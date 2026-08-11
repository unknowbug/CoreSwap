# 07-block-pipeline.md 「性能回归实测」小节更新草稿（根因定论版 2026-08-12）

> **目标路径**：`versions/1.20.1/docs/07-block-pipeline.md`
> **范围**：L64-101 整个「性能回归实测（2026-08-11）」小节（含吞吐表、WG_PROFILE 表、对照实验、疑似根因、状态与下一步）。
> **应用方式（主会话）**：用下方「替换块」的**新文本**整体替换 marker 块（L64 标题行 → L101 末行）。08-11 实测表（吞吐 + WG_PROFILE）原样保留，只追加 08-12 确认数据与定论。
> **与旧草稿关系**：`draft-07-performance-fix.md`（08-11 版）已应用后即当前小节；本草稿是其定论更新版，不覆盖旧草稿文件。
> **草稿状态**：draft（主会话应用 + 用户已拍板；修复验证待 Phase 2 闭环）。

---

## 定位 marker（现有文本，整块替换）

```markdown
## 性能回归实测（2026-08-11）

> 发现：2026-08-11 用户实机传送后区块卡很久（vanilla 对照确认），SURFACE 吞吐严重退化。本次 Java 桥并发重写 + C++ CoreSwapPool 改造（perf-rework，RQ-001~005）**已排除为引入源**（stash 对照 + 旧提交对照均慢，见下）；根因为 8/6 优化链遗留的缓存/执行模型失配。状态：🔍 待修（未结案）。

### 吞吐数据（SURFACE 模式，2026-08-11）

| 场景 | 2026-08-06 基线 | 2026-08-11 实测 | 备注 |
|---|---|---|---|
| 串行 | 28.1ms/chunk（450ms/16chunk） | **98-182ms/chunk** | 退化 ~3.5-6.5× |
| 并行（8/22 线程） | 49.4ms/16chunk（3.1ms/chunk） | **108-239ms/chunk** | **无加速反降**；并行不随线程数伸缩 |
| density 阶段 | 8.5-11.7ms/chunk | **670-1000ms/chunk** | ~100×；根因所在 |

### WG_PROFILE 计数器（density 阶段，2026-08-11）

| 指标 | 旧值（2026-08-06） | 2026-08-11 实测 | 含义 |
|---|---|---|---|
| spline 单次 | 992ns | **20,598ns** | ~21× 退化 |
| spline.sample | — | 338 万次 | 调用量 |
| FlatCache rebuild | — | **438,092 次 ≈ spline 调用数** | 每次 spline 采样都重建 5×5 网格（缓存命中率≈0） |
| Cache2D miss | — | **458,281 次** | 列缓存基本全 miss |

### 对照实验（排除本次改造引入）

- stash 本次改动（Java 桥重写 + C++ 池改造）后，HEAD 版 block_probe 8×8 仍 **10.2s**
- 连 07 篇基线提交 **86e4057** 也要 **8s**
- 结论：**吞吐退化在 8/6 优化链之后积累，非本次改造引入**；本次改造保持对齐（8576 99.9994% / 3200 99.9997% 零退化）且未恶化吞吐。具体引入提交待 git 二分（🔍）。

### 疑似根因（candidate 待验证）

1. **FlatCache/Cache2D 的 per-instance thread_local 缓存与「每 chunk 跨线程」执行模型冲突**：多线程并行时每线程独立缓存 → 每 chunk 跨线程迁移 → 缓存命中率归零 → 每 chunk 重建多次（rebuild 438,092 ≈ spline 调用数）。
2. **buildGrid 嵌套采样递归**：FlatCache 网格构建含嵌套采样；缓存失配时边界点（x=cx*16+16）不再命中本 chunk 网格 k=4，触发重建相邻 chunk 网格的递归——单次重建成本高（spline 992ns → 20,598ns）。
3. 8/6 优化的计数器（spline 34900 → 6250 次/chunk）在**单线程串行模型**下测出，未覆盖多线程并行/线程迁移场景。

### 状态与下一步

- 🔍 **未结案**：根因机制已实测坐实（缓存命中率≈0、density 阶段 ~100 倍级恶化），修复方案未验证。
- 候选方向：缓存按 chunk 键索引（每 chunk 独立缓存，而非 thread_local 线程亲和）/ 按调用上下文显式传入 / 恢复线程亲和。
- 相关：Java 桥并发重写（RQ-001~005，✅ 已实施）+ C++ 池改造（✅ 已实施）见 10 时间线 2026-08-11 条目；通用指纹见 knowledge/discovered/algorithm-fingerprints.md 发现 #10。
```

---

## 替换后的新文本（整体替换上述 marker）

```markdown
## 性能回归实测（2026-08-11 发现 → 2026-08-12 根因定论）

> 发现：2026-08-11 用户实机传送后区块卡很久（vanilla 对照确认），SURFACE 吞吐严重退化。本次 Java 桥并发重写 + C++ CoreSwapPool 改造（perf-rework，RQ-001~005）**已排除为引入源**（stash 对照 + 旧提交对照均慢，见下）；根因为 8/6 优化链遗留的缓存/执行模型失配。
> **2026-08-12 根因定论**：主因（H2）= FlatCacheDF **单槽 thread_local 缓存** + buildGrid 角点 `i=4`/`j=4` 越界 → 嵌套 spline 的 FlatCache 收到**邻居 chunk key** → 单槽被污染 → 邻居网格重建**递归蔓延 112 chunk**（rebuild 36,252 = **168×** → spline 调用 **20×**）；放大器（H3）= 多线程 thread_local thrashing（单次 ×16）。结论已过 judge 审查（`.investigations/perf-rework/review-rootcause.md`）并经**用户拍板确认**。状态：🔍 **修复中（Phase 2 已启动）**。

### 吞吐数据（SURFACE 模式，2026-08-11）

| 场景 | 2026-08-06 基线 | 2026-08-11 实测 | 备注 |
|---|---|---|---|
| 串行 | 28.1ms/chunk（450ms/16chunk） | **98-182ms/chunk** | 退化 ~3.5-6.5× |
| 并行（8/22 线程） | 49.4ms/16chunk（3.1ms/chunk） | **108-239ms/chunk** | **无加速反降**；并行不随线程数伸缩 |
| density 阶段 | 8.5-11.7ms/chunk | **670-1000ms/chunk** | ~100×；根因所在 |

### WG_PROFILE 计数器（density 阶段，2026-08-11）

| 指标 | 旧值（2026-08-06） | 2026-08-11 实测 | 含义 |
|---|---|---|---|
| spline 单次 | 992ns | **20,598ns** | ~21× 退化（08-11 多线程 thrashing 环境） |
| spline.sample | — | 338 万次 | 调用量 |
| FlatCache rebuild | — | **438,092 次 ≈ spline 调用数** | 每次 spline 采样都重建 5×5 网格（缓存命中率≈0） |
| Cache2D miss | — | **458,281 次** | 列缓存基本全 miss |

### 对照实验（排除本次改造引入）

- stash 本次改动（Java 桥重写 + C++ 池改造）后，HEAD 版 block_probe 8×8 仍 **10.2s**
- 连 07 篇基线提交 **86e4057** 也要 **8s**
- 结论：**吞吐退化在 8/6 优化链之后积累，非本次改造引入**；本次改造保持对齐（8576 99.9994% / 3200 99.9997% 零退化）且未恶化吞吐。具体引入提交待 git 二分（🔍）。

### 已确认根因（2026-08-12，用户拍板 + judge 通过）

> 根因分析全文：`.investigations/perf-rework/root-cause-draft.md`；judge 审查意见：`review-rootcause.md`。三组独立计数器数字闭环可复核。

1. **主因（H2 成立）**：FlatCacheDF **单槽 thread_local 缓存**（density.h L683-704）+ buildGrid 嵌套采样递归。buildGrid 角点 `i=4`/`j=4` 时 `p.x=(chunkX*4+4)*4=(chunkX+1)*16` 指向**下一 chunk 首列**（L735），嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（L687 key=(x>>4,z>>4)）→ 单槽被污染 → 重建邻居网格 → **递归蔓延 112 chunk**（36 生成 + 76 邻居）→ **rebuild 36,252 次 = 每 chunk ~1007 次（期望 ~6 次）→ 168× 爆炸** → 直接驱动 spline 调用 **20× 爆炸**（130,420/chunk vs 旧 6,250）。
2. **放大器（H3 成立）**：thread_local 单槽缓存 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。调用量不变（4,703,488 ≈ 4,695,145），单次成本 ×16（多线程 27,155ns vs 单线程 1,714ns）；wall 多线程 8488ms > 单线程 6533ms（并行反而更慢）。
3. **H1 部分成立（非主因）**：y 主序循环与 L630 注释矛盾属实，但 spline/cache_2d **全部来自 buildGrid 角点（y=0）**，块级 densityBuf 98,304 次采样被 InterpolatedDF 插值 + FlatCache 查表挡掉（0 次 spline），对爆炸直接贡献 ≈ 0。

**08-11 vs 08-12 数据口径说明（judge 审查要点 4）**：两个测量口径不同，不构成矛盾——08-11 为**多线程（8/22 线程）thrashing 环境**下粗粒度计数器（rebuild ≈ spline 调用数、单次 20,598ns 被 thrashing 放大）；08-12 为**单线程（-threads 1）WG_SPLINEDEBUG 精确统计**（剥离 thrashing 后暴露真实主因结构：rebuild 36,252 仅占 spline 调用 0.77%，放大链 = rebuild 168× × 13.36 spline/miss）。

### 2026-08-12 确认数据（单线程精确统计，WG_SPLINEDEBUG + WG_PROFILE）

| 指标 | 2026-08-11 实测（多线程环境） | 2026-08-12 确认（单线程） | 结论 |
|---|---|---|---|
| spline.sample | 338 万次 | **4,695,145**（= 130,420/chunk；旧基线 6,250/chunk） | **20× 爆炸 = 主因现象** |
| FlatCache rebuild | 438,092（≈ spline 调用数） | **36,252**（= 每 chunk ~1007，期望 ~6） | **168×**，直接驱动 20× spline |
| Cache2D miss | 458,281 | **351,536**（= 14,061 rebuild × 25 角点 ✓，4 个 cacheId） | 角点采样 miss 级联 |
| spline 单次 | 20,598ns | **t1 1,714ns / mt 27,155ns** | 多线程 thrashing ×16（H3 放大器） |
| rebuild chunk 覆盖 | — | **112 chunk**（36 生成 + 76 邻居） | 递归蔓延实锤 |

### 修复方向（2026-08-12 定论）

- **主修复：per-chunk 多槽缓存**（FlatCacheDF/Cache2DDF 单槽 → 最近 4-8 chunk 的网格/值 map，key 不变）。低风险：纯缓存，采样值逐位不变，**不破坏 BK-001 对齐**（8576/3200 零退化铁律）；需保留 k=4 边界命中语义（density.h L700-702）。预期 rebuild 回落 ~6/chunk → spline 回 ~1,000/chunk。
- **后续：线程亲和恢复**（每 chunk 固定线程 / per-thread 缓存绑定 chunk 生命周期）→ 消除 H3 thrashing（单次 27,155ns → ~1,714ns）。
- **改循环顺序无效且不推荐**（H1 非主因：块级不触发 spline；且 aquifer/oreVein 同序读取 densityBuf，改动有未验证的对齐风险）。

### 状态与下一步

- 🔍 **修复中（Phase 2 已启动）**：根因已定论（H2 主因 + H3 放大器，用户拍板 + judge 通过），per-chunk 多槽缓存修复方案已立项实施，验证待闭环（修复完成后以 08-12 同口径计数器复测 rebuild/spline 回落）。
- 相关：Java 桥并发重写（RQ-001~005，✅ 已实施）+ C++ 池改造（✅ 已实施）见 10 时间线 2026-08-11 条目；根因定论见 10 时间线 2026-08-12 条目；通用指纹见 knowledge/discovered/algorithm-fingerprints.md 发现 #10。
```
