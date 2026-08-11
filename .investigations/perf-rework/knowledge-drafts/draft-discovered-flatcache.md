# discovered/algorithm-fingerprints.md 追加草稿（发现 #10）

> **应用方式（主会话）**：将下方 `---` 分隔线之后的内容，**追加到 `knowledge/discovered/algorithm-fingerprints.md` 末尾**（发现 #9 之后，编号 #10）。
> **INDEX.md 无需改动**：algorithm-fingerprints.md 已在 INDEX 分类表「已确认的算法/协议指纹」中，追加发现不改变分类入口。
> 草稿状态：draft — candidate（机制已实测，根因修复未验证；修复闭环后再升 confirmed）。

---

## 发现 #10: thread_local 缓存与「每 chunk 跨线程」执行模型冲突 → 缓存命中率归零的性能回归指纹

**发现时间:** 2026-08-11
**发现者:** worker（perf-rework 性能回归调查）
**来源定位:** MC 1.20.1 主世界密度求值缓存（versions/1.20.1/docs/07-block-pipeline.md 2026-08-06 纯算法优化链 FlatCache/Cache2D）+ `.investigations/perf-rework/`（WG_PROFILE 实测 2026-08-11）
**置信度:** candidate（FlatCache 失效机制已 WG_PROFILE 实测坐实：命中率≈0；根因修复未验证）
**module:** perf

### 观察
性能优化常引入「局部缓存」把重复计算降为 O(1)（如 8/6 把 spline 采样 34900 → 6250 次/chunk，靠 FlatCache 5×5 网格 + Cache2D 列缓存）。**这类缓存的收益依赖「缓存生命周期 ⊇ 重复访问窗口」**：

- 原设计假设（8/6，单线程串行）：同一 chunk 生成期间大量 spline 采样重复 → per-instance（per-DensityFunction）**thread_local** 缓存命中。
- 当执行模型变为「线程池并行消费 chunk 任务」（每 chunk 可能由不同线程处理、线程跨 chunk 迁移）时，thread_local 缓存与 chunk 生命周期**不匹配**：每线程独立缓存 → 每 chunk 首访即 miss → 命中率归零 → 每次访问都走完整重建路径。

**指纹信号**：缓存重建/失效计数 ≈ 缓存访问总数（命中率≈0），且原 O(1) 路径变成重建热点（单次成本放大一个量级）；伴随「多线程不加速甚至反降」（并行只放大重建并发，不摊薄重复访问）。本次实测：FlatCache rebuild 438,092 ≈ spline 调用数、Cache2D miss 458,281 次、spline 单次 992ns → 20,598ns、density 阶段 8.5-11.7ms → 670-1000ms/chunk——正是此指纹。

叠加因素：FlatCache 网格构建含**嵌套采样递归**（边界点 x=cx*16+16 命中本 chunk 网格 k=4 才不重建，失配时触发相邻 chunk 网格重建递归）——缓存 miss 时单次重建成本被递归放大，进一步恶化。

### 证据
- WG_PROFILE（2026-08-11，density 阶段）：spline 单次 992ns → 20,598ns；spline.sample 338 万次；FlatCache rebuild 438,092 次 ≈ spline 调用数；Cache2D miss 458,281 次；density 阶段 670-1000ms/chunk（旧 8.5-11.7ms）
- 吞吐（SURFACE）：07 篇旧基线串行 28.1ms/chunk、并行 49.4ms/16chunk（3.1ms/chunk）→ 2026-08-11 实测单线程 98-182ms/chunk、多线程（8/22）108-239ms/chunk **无加速反降**
- 对照实验排除「本次改造引入」：stash 本次改动（Java 桥重写 + C++ 池改造）后 HEAD 版 block_probe 8×8 仍 10.2s；07 篇基线提交 86e4057 也要 8s → 回归在 8/6 优化链之后积累
- 数据载体：`.investigations/perf-rework/`（requirements-doc.md / static-audit.md / architecture.md / random-seed-sampling.md）+ 10 时间线 2026-08-11 条目 + 07 篇「性能回归实测」小节草稿

### 如何利用
- **设计缓存前先明确「缓存生命周期 vs 执行模型」是否匹配**：thread_local 只适合「线程内连续消费同一上下文」（如单线程完整生成一个 chunk）；线程池并行 + 任务迁移时，用 **per-chunk 键索引缓存**（缓存随 chunk 生命周期）或按调用上下文显式传入，**不要依赖线程亲和**
- **性能回归排查第一手段 = 缓存计数器**：看 rebuild/miss 与命中之比；命中率≈0 即缓存失效（本次正是靠 WG_PROFILE 计数器坐实）
- **优化计数器要结合真实执行模型验证**：8/6 的 spline 34900 → 6250 次/chunk 是单线程串行模型下的计数，未覆盖多线程并行/线程迁移——「优化后计数器下降」不等于「目标执行模型下收益」
- **git 二分定位引入点**：stash/checkout 旧提交对照（本次 8s 级退化用 stash 实验证明非本次引入，具体引入提交待二分）
- 跨版本/跨项目通用：任何「局部缓存 + 并行执行」组合都适用此检查
