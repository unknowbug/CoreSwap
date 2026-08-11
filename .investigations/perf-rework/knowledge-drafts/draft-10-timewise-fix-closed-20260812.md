# 10-timewise-archive.md 追加段草稿（2026-08-12 修复实施与闭环）

> **目标路径**：`versions/1.20.1/docs/10-timewise-archive.md`
> **应用方式（主会话）**：将下方 `---` 分隔线之后、`## 2026-08-12（补）...` 起的内容，**原样追加到该文件末尾**（文档铁律：追加不覆盖）。当前文件末尾（L1170）为 2026-08-12 根因定论条目最后一行「- 🔍 **Phase 2 修复中**：**per-chunk 多槽缓存**（主修复，低风险，采样值逐位不变，不破坏 BK-001 对齐；保留 k=4 边界命中语义 density.h L700-702）→ **线程亲和恢复**（后续，消除 thrashing）；改循环顺序不推荐（H1 非主因）。修复验证待闭环（以 08-12 同口径计数器复测 rebuild/spline 回落）。」，在它之后追加。
> **状态标注**：✅ 修复闭环 / 2026-08-12 用户验收——与 10 篇既有条目风格一致。
> **草稿状态**：draft（主会话应用；数字与 `.investigations/perf-rework/cmd-output/` 落盘文件核对一致）。

---

## 2026-08-12（补）：性能回归修复实施与闭环（16 槽 LRU 失败 → 上下文绑定成功）（✅ 修复闭环 / 用户验收）

> 承接上一条 2026-08-12 根因定论条目。修复经历两版演进：初版 16 槽 LRU 未消除蔓延 → 终版「当前生成 chunk 上下文绑定」与 Java per-chunk 实例语义完全对齐，验证达标 + judge 通过 + 用户验收。设计文档 `.investigations/perf-rework/fix-design.md`（§0 含实现演进注记）+ 审查 `.investigations/perf-rework/review-fix-delivery.md`，均已登记 `.artifacts/index.yaml`（kind: plan / review，status: candidate）。

### 实施演进：16 槽 LRU → 上下文绑定

- **初版（16 槽 LRU）**：FlatCacheDF/Cache2DDF 均改 thread_local 16 槽 LRU（`std::array<SubSlot,CAP>` key/grid/stamp，模拟 Java per-chunk 实例缓存）。实测 rebuild 36,252→**7,318**（5× 降）但**未消除蔓延**：rebuild **203/chunk** vs 期望 6、chunk 覆盖仍 **112**（splinedebug_8576_t1_fixed.txt；SPLINE 14,772/chunk）。→ **弃用原因**：16 槽 LRU 仍为「pos 推导的邻居 key」构建网格，只减少重建频率，**不改变「越界=重建」语义**。
- **关键洞察**：Java FlatCache 是 **per-chunk 实例**（构造时绑定 chunk、一次性预计算 25 角点、越界 delegate.sample 直算**永不构建邻居网格**，ChunkNoiseSampler.java L836-881）；C++ 是全局单例 DensityFunction 树，单槽/多槽缓存都做不到「越界不重建」——必须显式传入当前 chunk 上下文。
- **终版（当前 chunk 上下文绑定）**：thread_local `g_curChunkX/Z`（density.h L40-41）在 `fillOneChunkCore` 入口 RAII 设置、函数返回恢复 `INT32_MIN`（judge 修正项 ② RAII 恢复已闭环；诊断路径回退分支语义保留）；网格绑定当前 chunk，k/l 相对 startBiomeX 计算（`k=(pos.x>>2)-slot.cx*4`），越界 → `delegate.sample(pos)` 直算不重建。**Cache2DDF 保留 16 槽 LRU**（角点共享列可命中，无蔓延风险）。与 Java FlatCache 六维逐条对齐（review-fix-delivery.md 审查要点 1 表：实例绑定/网格构建/k-l 计算/界内查表/越界直算/边界共享 ✅）。
- 机理：buildGrid 角点 i=4 的 pos 采样时 `cx=g_curChunkX=当前 chunk` → `k=4 ∈ [0,5)` 命中本网格；更远越界 → 直算。**蔓延根除**。

### ✅ 验证数据（终版 ctx，2026-08-12 落盘）

数据文件：`cmd-output/regress_8576_raii.txt`、`regress_3200_raii.txt`、`wgprofile_8576_t1_ctx.txt`、`splinedebug_8576_t1_ctx.txt`（stat_ctx.py 统计）、`bench_8x8_noprof.txt`。

| 指标 | 修复前（08-12 定论） | 终版 | 结论 |
|---|---|---|---|
| FLATCACHE rebuild | 36,252（~1007/chunk，168×） | **216 = 6.0/chunk** | 期望 ~6 完全达标 ✓ |
| rebuild chunk 覆盖 | 112（36 生成 + 76 邻居） | **36** | 蔓延根除 ✓ |
| CACHE2D miss | 351,536 | **23,117** | ↓15× |
| SPLINE（SPLINEDEBUG 非 leaf 口径） | 66,682/chunk | **3,032/chunk** | 回旧基线 6,250 水平 ✓ |
| spline.sample（WG_PROFILE 全量） | 130,420/chunk | **5,906/chunk**（212,622/36） | ↓22× |
| 单线程 wall | 6,533ms（181ms/chunk） | **2,910ms** | 2.2× |
| bench_chunks 单线程 | ~181ms/chunk | **62.38ms/chunk** | 3× |
| 对齐 8576 / 3200 | 99.9994% / 99.9997% | **99.9994% / 99.9997%** | 零退化 ✓ |

- 口径注明（judge 修正项 ③ 已闭环）：SPLINEDEBUG `[SPLINE]` 为入口行（非 leaf）计数；WG_PROFILE `spline.sample` 为全量采样计数；wall/bench 为落盘文件数值（wgprofile_8576_t1_ctx.txt wall=2910.0ms；bench_8x8_noprof.txt threads=1 62.38ms/chunk）。
- 16 槽 LRU 对照：rebuild 7,318（203/chunk）、覆盖仍 112、bench 79.91ms/chunk（bench_fixed_ctx.txt）、wall 3,469ms（wgprofile_8576_t1_fixed.txt）——方向正确但未达标，弃用。

### ✅ judge 审查通过（review-fix-delivery.md）

- **主结论通过**：修复机制（FlatCacheDF 上下文绑定 + 越界直算不重建、Cache2DDF 16 槽 LRU）与 Java per-chunk 实例语义逐条对齐；边界 k=4 命中语义保留；buildGrid 角点 i=4 不再触发邻居网格重建（机理经代码路径推演成立，实测 rebuild 216/覆盖 36 吻合）；纯缓存路径改造零退化在数学上成立（双种子 99.9994%/99.9997% 落盘与修复前一致）；thread_local + fillOneChunkCore 单线程完整处理保证线程安全（无跨线程上下文污染）。
- **4 项修正已闭环**：① fix-design.md 补实现演进注记（§0）+ 登记 index.yaml ✅ ② fillOneChunkCore 末尾 RAII 恢复 g_curChunkX/Z=INT32_MIN + 注释修正（「未设置或已恢复时回退」）✅ ③ 性能数字口径注明（SPLINEDEBUG 非 leaf vs WG_PROFILE 全量；以落盘文件为准）✅ ④ retry 轮次记录缺失声明（修复为工程迭代，验证单轮完成）✅

### ✅ 用户验收 + 剩余课题

- ✅ **用户验收（2026-08-12）**：修复闭环确认，性能回归结案（rebuild 216=6.0/chunk 完全达期望、蔓延根除、双种子零退化）。
- 🔍 **剩余课题（独立于本次修复，待续）**：
  1. **多线程无加速**：bench threads=8 62.17ms/chunk ≈ 单线程 62.38ms——spline/cache 已非瓶颈，**aquifer+oreVein 阶段**（wgprofile_8576_t1_ctx.txt 20-52ms/chunk，远超 spline 贡献）成主导；需线程亲和（root-cause 方案 2）/ aquifer 并行化。
  2. **spline 单次 7,971ns**（WG_PROFILE ctx 口径）：调用量 ↓22× 后的单次成本，非本次修复引入的劣化（review 三源不一致 #2 已注明出处 = wgprofile_8576_t1_ctx.txt L80），与修复前 1,714ns 为不同测量口径。
  3. **aquifer 阶段 4× 级**（20-52ms/chunk vs 旧基线 6.5-8.9ms）——独立课题。
