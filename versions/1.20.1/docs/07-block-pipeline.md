# 7. 块级流水线与性能（worldgen_api.cpp）

## 功能目的

单 chunk 从 seed 到 16×16×384 方块的完整流程，以及性能优化记录（多线程 + 缓存）。

## 1.20.1 工作机制（fillOneChunk）

```
wg_create（一次性）：
  DensityBuilder（JSON → 密度树）+ blocks registry + biome + surfaceBuilder + overworldRule 预构建

fillOneChunk(cx, cz)（每 chunk）：
  1. aquifer = Aquifer(barrier, floodedness, spread, fluidType, initialDensity, ...,
                       split("minecraft:aquifer").nextSplitter(), sh4 4 角估计)
  2. oreVein = OreVeinSampler(vein_toggle, vein_ridged, vein_gap, split("minecraft:ore").nextSplitter())
  3. 块循环 by(0..383) → bz → bx：
       density = finalDensity.sample(x, y, z)          # 树内 InterpolatedDF 自行插值
       block = aquifer.apply(density)                  # -1 表示 null
       if (block < 0) block = oreVein.apply(x, y, z)   # ChainedBlockSource
       if (block < 0) block = stone                    # 默认
       heightmap 更新
  4. buildSurface（列引擎 + surface rules）覆盖表层
  5. memcpy → out
```

**ChainedBlockSource 顺序**：aquifer 先，返回 null 才轮到 oreVein，再 null 用默认 stone。
**高度图**：WORLD_SURFACE_WG（最高非空气块），SURFACE 阶段起始列顶。

## 性能基准（2026-08-06，seed -8248318472910187742，4×4 chunks）

> ⚠️ **已过时（2026-08-11 实测）**：下表为 2026-08-06 基线。2026-08-11 实测 SURFACE 吞吐已严重退化（单线程 98-182ms/chunk、多线程 108-239ms/chunk 无加速反降），并行 49.4ms/16chunk（3.1ms/chunk）**不再可达**。根因 = FlatCache/Cache2D 缓存失效（见下方「性能回归实测」小节）。历史保留，勿再引用为当前性能。

| 方案 | 耗时（16 chunks） | 备注 |
|---|---|---|
| 初始串行 | ~1800ms | aquifer 无缓存 |
| + aquifer 列缓存 | 1056ms | estimateSurfaceHeight ~2700 倍降幅 |
| + 多线程并行（自适应线程） | **110ms** | ~9.6×；100% 保持 |
| + 纯算法优化（无损）串行 | **450ms（28.1ms/chunk）** | 见下 |
| + 纯算法优化（无损）并行 | **49.4ms（3.1ms/chunk）** | 110→49.4ms（-55%） |

### 纯算法优化链（全部无损、100% 保持，2026-08-06 第二批）

**方法**：WG_PROFILE 剖析（阶段计时 + 计数器）→ 数据定位冗余 → 逐项修复。

1. **FlatCacheDF + Cache2DDF（最大结构性冗余！）**：Java ChunkNoiseSampler 对
   `minecraft:flat_cache`/`minecraft:cache_2d` 有真正缓存（FlatCache 5×5 网格预计算、Cache2D 列缓存），
   而 C++ 此前仅"语义委托"（每块重算）——**spline 采样 34900 → 6250 次/chunk**（大陆样条
   continents/erosion/ridges/factor/jaggedness/offset 每块重算是 Java 官方算法里已被缓存的冗余）。
   - FlatCache：per-instance thread_local 5×5 网格（Java: horizontalBiomeEnd+1=5，间距 4 块，y=0）；
     **边界点（x=cx*16+16）命中本 chunk 网格 k=4，不重建**（否则嵌套采样递归重建相邻 chunk 网格）。
   - Cache2D：per-instance thread_local 单列缓存（Java: lastSamplingColumnPos）。
2. **aquifer blocks->id() 预取（隐藏最大单点收益）**：apply 每块 4-7 次 `blocks->id("air"/"water"/"lava")`
   **std::map 字符串查找**（40-70 万次/chunk）——构造时预取三常量 → **aquifer 25ms→8.9ms（-64%）**。
3. **aquifer 列缓存 std::map → flat 数组**：estimateSurfaceHeight 13 邻居查询每块 13 次 map 查找
   → 32×32 数组（覆盖 ±48 格邻居范围）O(1)。
4. **oreVein y 范围预检查**：y∈[-60,50] 外提前返回（Java 采样 veinToggle 后同样返回 -1，结果一致）。
5. **surface 噪声列缓存**：NoiseThreshold（per-instance thread_local 单槽）/sampleRunDepth/getTerracottaBlock
   ——buildSurface 逐列处理，同列 2D 噪声采样位置相同 → 每列 1 次。

**陷阱**：surface 噪声缓存放 SurfaceContext 的 std::map 反而变慢（每块字符串查找）；
正确做法 = cond 实例的 thread_local 单槽（O(1)）+ 多线程安全。

## 性能回归实测（2026-08-11 发现 → 2026-08-12 根因定论）

> 发现：2026-08-11 用户实机传送后区块卡很久（vanilla 对照确认），SURFACE 吞吐严重退化。本次 Java 桥并发重写 + C++ CoreSwapPool 改造（perf-rework，RQ-001~005）**已排除为引入源**（stash 对照 + 旧提交对照均慢，见下）；根因为 8/6 优化链遗留的缓存/执行模型失配。
> **2026-08-12 根因定论**：主因（H2）= FlatCacheDF **单槽 thread_local 缓存** + buildGrid 角点 `i=4`/`j=4` 越界 → 嵌套 spline 的 FlatCache 收到**邻居 chunk key** → 单槽被污染 → 邻居网格重建**递归蔓延 112 chunk**（rebuild 36,252 = **168×** → spline 调用 **20×**）；放大器（H3）= 多线程 thread_local thrashing（单次 ×16）。结论已过 judge 审查（`.investigations/perf-rework/review-rootcause.md`）并经**用户拍板确认**。状态：✅ **修复闭环（2026-08-12 用户验收）**——修复方案（当前 chunk 上下文绑定，与 Java per-chunk 语义对齐）与验证数据见下方「修复方案（已实施并闭环）」/「修复闭环验证」小节；judge 审查：`.investigations/perf-rework/review-fix-delivery.md`（主结论通过，4 项修正已闭环）。

> ⚠️ **2026-08-16 影响标注（H3 ×16 需重新定性）**：本条「放大器（H3）= 多线程 thread_local thrashing（单次 ×16）」的 **×16 基数（mt 27,155ns）在 notify 丢失 bug 活跃期采集**（实际并行度=1，见下方 L97 标注 + `mt-scaling-errors.md` MT2）——H3 结论需修复后重测重新定性；**H2 主因（rebuild 168×）为单线程精确统计，不受影响，保留成立**。

### 吞吐数据（SURFACE 模式，2026-08-11）

| 场景 | 2026-08-06 基线 | 2026-08-11 实测 | 备注 |
|---|---|---|---|
| 串行 | 28.1ms/chunk（450ms/16chunk） | **98-182ms/chunk** | 退化 ~3.5-6.5× |
| 并行（8/22 线程） | 49.4ms/16chunk（3.1ms/chunk） | **108-239ms/chunk** | **无加速反降**；并行不随线程数伸缩 |
| density 阶段 | 8.5-11.7ms/chunk | **670-1000ms/chunk** | ~100×；根因所在 |

> ⚠️ **2026-08-16 影响标注（notify 丢失 bug，0a781e1 修复）**：本表「并行（8/22 线程）108-239ms/chunk 无加速反降」在 **notify 丢失 bug 活跃期（8/6-8/15）** 采集——[A] T>1 顺序跑下补建 worker 错过 notify 永久等待，**实际并行度=1（串行假象）**，「反降/无加速」幅度**不可信**（真实并行成本被 bug 伪影掩盖）。**H2 主因（FlatCache 单槽缓存 + buildGrid 角点越界 → rebuild 168×）为单线程精确统计（WG_SPLINEDEBUG），不受影响，保留成立**。影响面：`.investigations/worldgen-mt-scaling/notify-bug-impact.md`（§2 #3）+ `mt-scaling-errors.md`（MT1/MT2）；修复后重测数据见文末「2026-08-16 影响评估修正」。

### WG_PROFILE 计数器（density 阶段，2026-08-11）

| 指标 | 旧值（2026-08-06） | 2026-08-11 实测 | 含义 |
|---|---|---|---|
| spline 单次 | 992ns | **20,598ns** | ~21× 退化（08-11 多线程 thrashing 环境） |
| spline.sample | — | 338 万次 | 调用量 |
| FlatCache rebuild | — | **438,092 次 ≈ spline 调用数** | 每次 spline 采样都重建 5×5 网格（缓存命中率≈0） |
| Cache2D miss | — | **458,281 次** | 列缓存基本全 miss |

> ⚠️ **2026-08-16 影响标注（本表为 WG_PROFILE 计数器，含双污染）**：① **notify bug 污染**——「spline 单次 20,598ns（~21× 退化）」在 notify 丢失 bug 活跃期采集（[A] T>1 实际并行度=1，「多线程 thrashing 环境」实为单 worker + 扩池开销），需修复后重测（`mt-scaling-errors.md` MT2）；② **计时污染（MT4）**——WG_PROFILE/WG_STAGETIMER 每采样点 steady_clock + 原子计数，探针自身开销计入阶段耗时（density 460ms 伪影 = 真实 45ms；本表 density 670-1000ms 同为伪影），spline 单次耗时列不可直接引用。**FlatCache rebuild / Cache2D miss 为纯计数器（无计时语义），不受上述污染，保留**。详见 `mt-scaling-errors.md` MT2/MT4。

### 对照实验（排除本次改造引入）

- stash 本次改动（Java 桥重写 + C++ 池改造）后，HEAD 版 block_probe 8×8 仍 **10.2s**
- 连 07 篇基线提交 **86e4057** 也要 **8s**
- 结论：**吞吐退化在 8/6 优化链之后积累，非本次改造引入**；本次改造保持对齐（8576 99.9994% / 3200 99.9997% 零退化）且未恶化吞吐。具体引入提交待 git 二分（🔍）。

### 已确认根因（2026-08-12，用户拍板 + judge 通过）

> 根因分析全文：`.investigations/perf-rework/root-cause-draft.md`；judge 审查意见：`review-rootcause.md`。三组独立计数器数字闭环可复核。

1. **主因（H2 成立）**：FlatCacheDF **单槽 thread_local 缓存**（density.h L683-704）+ buildGrid 嵌套采样递归。buildGrid 角点 `i=4`/`j=4` 时 `p.x=(chunkX*4+4)*4=(chunkX+1)*16` 指向**下一 chunk 首列**（L735），嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（L687 key=(x>>4,z>>4)）→ 单槽被污染 → 重建邻居网格 → **递归蔓延 112 chunk**（36 生成 + 76 邻居）→ **rebuild 36,252 次 = 每 chunk ~1007 次（期望 ~6 次）→ 168× 爆炸** → 直接驱动 spline 调用 **20× 爆炸**（130,420/chunk vs 旧 6,250）。
2. **放大器（H3 成立）**：thread_local 单槽缓存 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。调用量不变（4,703,488 ≈ 4,695,145），单次成本 ×16（多线程 27,155ns vs 单线程 1,714ns）；wall 多线程 8488ms > 单线程 6533ms（并行反而更慢）。

> ⚠️ **2026-08-16 影响标注（H3 ×16 需重新定性）**：本条「mt 27,155ns（×16）」在 **notify 丢失 bug 活跃期**采集——实际并行度=1，「多线程环境」实为单 worker + 扩池开销，**×16 的「多线程侧」基数不可信**，H3 结论**需修复后重测重新定性**（待办：mt 侧 spline 单次成本重测；若 mt≈t1 则 H3 为伪结论/降级，详见 `mt-scaling-errors.md` MT2）。**H2 主因（rebuild 168×，单线程 WG_SPLINEDEBUG 精确统计）不受影响，保留成立**。
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

> ⚠️ **2026-08-16 影响标注**：本表「spline 单次 **t1 1,714ns / mt 27,155ns**（×16 H3 放大器）」的 **mt 侧数值在 notify bug 活跃期采集**（实际并行度=1），×16 需修复后重测重新定性（`mt-scaling-errors.md` MT2）；**t1 1,714ns 为单线程精确统计，不受影响**。H2 行（rebuild 36,252 = 168×）为单线程数据，保留成立。

### 修复方案（已实施并闭环，2026-08-12）

> 设计文档：`.investigations/perf-rework/fix-design.md`（§0 含实现演进注记，已登记 `.artifacts/index.yaml`）；judge 审查：`.investigations/perf-rework/review-fix-delivery.md`（主结论通过 + 4 项修正闭环）。

- **终版：FlatCacheDF 改为「当前生成 chunk 上下文绑定」（与 Java per-chunk 实例语义完全对齐）**。thread_local `g_curChunkX/Z`（density.h L40-41）在 `fillOneChunkCore` 入口 RAII 设置、函数返回恢复 `INT32_MIN`；网格绑定当前 chunk，k/l 相对 `startBiomeX` 计算（`k=(pos.x>>2)-slot.cx*4`），越界 → `delegate.sample(pos)` **直算不重建**。与 Java `ChunkNoiseSampler.java` L836-881 FlatCache（构造时一次性预计算 25 角点、之后纯查表、**永不构建邻居网格**）六维逐条对齐（实例绑定/网格构建/k-l 计算/界内查表/越界直算/边界共享，见 review-fix-delivery.md 审查要点 1 表）。
- **机理（蔓延根除）**：buildGrid 角点 i=4 的 pos 不再用 pos 推导邻居 key——嵌套 spline 的 FlatCache 采样该 pos 时 `cx=g_curChunkX=当前 chunk` → `k=4 ∈ [0,5)` 命中本网格；更远越界 → arg 直算，亦不重建。
- **Cache2DDF**：保留 **16 槽 LRU**（角点共享列可命中，无蔓延风险；review-fix-delivery.md 已确认对齐安全）。
- **初版 16 槽 LRU 已弃用（关键教训）**：FlatCacheDF 也上 16 槽 LRU 时 rebuild 36,252→7,318（5× 降）**但未消除蔓延**（rebuild 203/chunk vs 期望 6，覆盖仍 112 chunk）——16 槽 LRU 仍会为 **pos 推导的邻居 key** 构建网格：多槽只减少重建频率，**不改变「越界=重建」语义**。上下文绑定从根上消除「越界→重建」，与 Java 语义一致，故终版采用（fix-design.md §0 演进注记）。
- **改循环顺序无效且不推荐**（H1 非主因：块级不触发 spline；且 aquifer/oreVein 同序读取 densityBuf，改动有未验证的对齐风险）。

### 修复闭环验证（2026-08-12，终版 ctx 数据）

> 数据文件：`cmd-output/regress_8576_raii.txt`、`regress_3200_raii.txt`、`wgprofile_8576_t1_ctx.txt`、`splinedebug_8576_t1_ctx.txt`（stat_ctx.py 统计）、`bench_8x8_noprof.txt`。数字口径：SPLINEDEBUG 为 `[SPLINE]` 非 leaf 入口行计数（每 chunk 3,032）；WG_PROFILE `spline.sample` 为全量采样计数（每 chunk 5,906），两口径并存，方向一致。

| 指标 | 修复前（08-12 定论） | 16 槽 LRU 初版 | **终版（上下文绑定）** | 结论 |
|---|---|---|---|---|
| FlatCache rebuild | 36,252（~1007/chunk，168×） | 7,318（203/chunk） | **216 = 6.0/chunk** | 完全达期望 ~6 ✓ |
| rebuild chunk 覆盖 | 112（36 生成 + 76 邻居） | 112（蔓延未除） | **36** | 蔓延根除 ✓ |
| CACHE2D miss | 351,536 | — | **23,117** | ↓15× |
| SPLINE（非 leaf 口径） | 66,682/chunk | 14,772/chunk | **3,032/chunk** | 回旧基线 6,250 水平 ✓ |
| spline.sample（WG_PROFILE 全量） | 130,420/chunk | — | **5,906/chunk**（212,622/36） | ↓22× |
| 单线程 wall | 6,533ms（181ms/chunk） | 3,469ms（wgprofile_8576_t1_fixed） | **2,910ms** | 2.2× ✓ |
| bench_chunks 单线程 | ~181ms/chunk（口径：旧 wall 6533/36） | 79.91ms/chunk（bench_fixed_ctx） | **62.38ms/chunk** | 3× ✓ |
| 对齐 8576 / 3200 | 99.9994% / 99.9997% | 同 | **99.9994% / 99.9997%** | 零退化 ✓ |


### 状态与下一步

- 🔍 **修复中（Phase 2 已启动）**：根因已定论（H2 主因 + H3 放大器，用户拍板 + judge 通过），per-chunk 多槽缓存修复方案已立项实施，验证待闭环（修复完成后以 08-12 同口径计数器复测 rebuild/spline 回落）。
- 相关：Java 桥并发重写（RQ-001~005，✅ 已实施）+ C++ 池改造（✅ 已实施）见 10 时间线 2026-08-11 条目；根因定论见 10 时间线 2026-08-12 条目；通用指纹见 knowledge/discovered/algorithm-fingerprints.md 发现 #10。

### 当前热点（串行 28.1ms/chunk，WG_PROFILE 数据）（2026-08-06 串行基线；2026-08-11 已退化，见性能回归实测）

| 阶段 | 耗时 | 构成 |
|---|---|---|
| density | 8.5-11.7ms | base_3d_noise 122 次/chunk（插值网格角点）、spline 6250 次（FlatCache 构建）、98k 块树遍历 |
| aquifer+oreVein | 6.5-8.9ms | 72% 块走 18 候选遍历（Java 同构，无法无损减少） |
| sh4+surface | 9-10.7ms | 98k 块规则遍历（Java 同构）+ 每块 VerticalGradient 随机 |

**结构上无法再无损压缩的**（Java 同构）：aquifer 18 候选遍历、surface 规则遍历、
VerticalGradient 每块 split(x,y,z)（依赖 y）。

### ⚠️ 为什么不做「base_3d_noise 网格插值」优化

Java 的 base_3d_noise **逐块重算 24 次 Perlin（无缓存）**。若 C++ 改为 cell 网格插值缓存，
会引入浮点误差**破坏 100% 逐位对齐**。多线程是唯一无损的大优化。
若未来追求进一步加速且可接受非逐位一致（如 ±1 块误差），需先经用户确认。

## 版本敏感点

- [ ] **fillOneChunk 的调用顺序**：ChainedBlockSource 的 sampler 顺序（aquifer→oreVein）随版本变（1.17 无 oreVein）。
- [ ] **fluidLevelSampler 默认**：`y < -54 lava else water`（主世界）——版本/维度敏感。
- [ ] heightmap 类型（WORLD_SURFACE_WG）与 buildSurface 起始。
- [ ] out 布局 `(y+64)*256 + z*16 + x` 与 vanilla raw id（air=0）——与 Java 导出格式绑定。

## 已验证的坑

- **块级插值顺序**：finalDensity 整树角点采样+手动插值是错的（非线性不可交换），必须块级直接采样（03 篇）。
- **线程数默认**：不要用单台机器的 hardware_concurrency 写死；API 默认自适应 + 调用方 `-threads` 可配。
- **验证多线程一致性**：block_probe 并行 vs got_export 串行，TOTAL 必须同为 100.0000%；
  任何差异说明有隐藏竞态（检查 mutable 缓存/懒构建）。

## Java 侧写入路径（CppBridge.fillChunk / writeChunk）的坑（2026-08-06 实测定稿）

> 现象：客户端地形只在 spawn 预生成区域（27×27 chunk ≈ 432×432 格）存在，之外全 air、
> 结构（村庄/冰山）悬浮。C++ 输出正常（buf 非空 27%、buf0=31=bedrock）、fillChunk 无异常。
> 根因是 **Java 写入层** 的连环 bug，与 C++ 无关。

### 坑 1：stateById 缓存不能预填 AIR

- writeChunk 用 `stateById[id]` 缓存 BlockState，`if (st == null)` 才查 `Registries.BLOCK.get(id)`
- 曾用 `Arrays.fill(stateById, AIR)` 做防御 → `st` 永远非 null → 从不查 registry → **所有方块写成 air**
- 教训：缓存数组必须 null 初始化（null = 未查过），不能填「默认值」占位（`st == null` 是哨兵）

### 坑 2：直写 PalettedContainer.set 不更新 nonEmptyBlockCount（核心根因）

- 性能优化曾直接 `container.set(x, sy, z, st)`（跳过 setBlockState 的 heightmap/blockEntity 开销）
- 字节码证据：`ChunkSection.isEmpty()` = `this.nonEmptyBlockCount == 0`（Field e:S）
- `PalettedContainer.set` 只改数据**不改 section 计数** → `isEmpty()` 永远 true
- `ProtoChunk.getBlockState` 反编译：`if (isOutOfHeightLimit(y)) return VOID_AIR; sec = getSection(e(y)); if (sec.isEmpty()) return AIR; ...`
- → **所有读路径（getBlockState/渲染/保存）返回空气**，FEATURES 结构照常生成 → 悬浮村庄/冰山
- **修复**：用 `ChunkSection.setBlockState(x, y, z, state)`（yarn 名，不是 `set`）——内部 = container.set + 计数更新，开销极小
- 不要反射改计数（运行时字段是混淆名，且 setBlockState 更干净）

### 坑 3：getSection(int) 语义用越界实测确认

- 1.20.1 `Chunk.getSection(int)` 是 **0-based 数组索引**（0..23 = y -64..319）
- 曾误判为「世界 y>>4 坐标」（-4..19）改 `getSection(secIdx - 4)` → 全部 AIOOBE（`Index -4 out of bounds for length 24`）→ 回退
- 教训：索引语义用「越界异常」实测，不要靠推理/文档猜

### 坑 4：readback 验证的陷阱

- `chunk.getBlockState(y=-64)` 可能因 minY 边界特判返回 air（误报）
- 坑 2 存在时：即使 container 有数据，`chunk.getBlockState` 也会因 `section.isEmpty()` 返回 air
- 可靠验证：`chunk.getSection(0).getBlockState(0, 0, 0)`（section 层）或生成完成后 `getChunk(x,z).getBlockState(...)`（WorldChunk 层）

### 坑 5：view-distance 复现法

- 客户端 `options.txt` renderDistance=32/simulationDistance=32 → 服务端 `server.properties` 也要 32 才能复现
- view-distance=10 时行为不同（曾误判「10 生效 32 不生效」，实际是日志输出截断误读）
- 修复后与 view-distance 无关；实测 64 无崩溃、无写入异常（threw=0、2699 chunk 正常），仅 spawn 预生成性能变慢（16384 chunk 需数分钟，原版同样）

### 反编译工具链（定位这类问题的关键）

- `javap -c -p -cp C:\Users\NDark\.gradle\caches\fabric-loom\1.20.1\minecraft-merged.jar <类>`（loom jar 是 mojang 混淆名）
- 混淆名对照：Chunk=`ddx`、ProtoChunk=`des`、ChunkSection=`dej`、PalettedContainer=`deq`
- yarn 1.20.1 文档：https://maven.fabricmc.net/docs/yarn-1.20.1-rc1+build.1/
- 看「字段/方法/是否更新计数」类逻辑，读字节码比读源码快

## 2026-08-08 已验证结论（自 10 时间线归档提炼，完整过程见 10-timewise-archive.md）

### ✅ 崩溃修复链（32 视距 / 并发）
- **CoreSwapPool::run 并发竞争**（1.0.11-pre 32 视距 99% 崩溃）：共享成员 fn/totalTasks/doneCount/nextTask/taskQueue 被 MC 多 Worker 并发调 → A 的 run 尾 fn=nullptr 被 B 读空 → 调用空 std::function → 读地址 0。**修复：run 开头 static mutex 串行化**（内部线程池仍并行 fillOneChunk）
- **derivedSplitters 并发写**（1.0.14）
- **mod id 漏改**（CppWorldgen.java:36 的 getNamespace 用旧 worldgen-bench）→ 1.0.5 修复；社区 PR #2 独立发现
- **out 越界写**：BLOCK_COUNT(98304) → 维度大小（nether 65536）——下界崩溃根因

### ✅ 崩溃日志 handler（1.0.15+，全局铁律）
- vectored exception + StackWalk64 + crash-coreswap-*.txt + dll sha256 打印；不吞异常（JVM hs_err 照常）

### ✅ worldgen.dll 对齐铁律（反复踩坑后制度化）
- **唯一权威 = cpp/build-msvc/bin/worldgen.dll**；每次编译后同步到 java/src/main/resources/native/worldgen.dll；对比/打包前 sha256 校验
- **DensityProbe 必须禁用 CppBridge**（densityProbe 不在 BenchMod.anyProbe → 默认启用 C++ 接管 → 参照被污染）——DensityProbe.run 开头 `CppBridge.enabled=false`
- gradle runServer 崩溃后 java 进程可能残留（占 world/端口）——先 `taskkill /F /IM java.exe`

### ✅ 参照导出/seed 校验
- server.properties `level-seed` 硬编码，`-PbenchSeed=X` 只设 Java 属性——跑其他 seed 必须改 level-seed + 删 world
- simulation-distance=2 + 删 world 可保 cns 存活（NOISE 状态），否则 spawn 预生成连带推进 → cns null

### ⚠️ 坑
- 混淆名对照：Chunk=`ddx`、ProtoChunk=`des`、ChunkSection=`dej`、PalettedContainer=`deq`（loom jar 是 mojang 混淆名）
- javap：`javap -c -p -cp <loom jar> <类>`

---

## 2026-08-08 已验证结论（追加 2）：24 块 mismatch 收尾分类 + finalDensity 边界翻转课题（candidate 待立项）

**8576（seed 8576294172403134396，720,-432 6×6）24 块 mismatch 收尾分类**（pillar 修复后剩余）：

| 类别 | 数量 | 根因/状态 |
|---|---|---|
| 深板岩/水边界 | 12 | 块级 finalDensity 边界翻转（candidate 待立项） |
| 地表三连错位 | 9 | 同上（=21 块课题） |
| river | 1 | 同机制（与 20000 river/taiga 边界差同族） |
| forest terracotta（#23/#24） | 2 | ✅ 已修复（biome 判定 tie-break，见 06 篇追加 3） |

- **根因假设**：块级 finalDensity 边界翻转 + 插值精度差——final_density 树在 range_choice 分支切换陡峭区（sloped_cheese≈1.5625 阈值，见 knowledge/discovered/algorithm-fingerprints.md 发现 #2）对网格角点值微差敏感，单块判 air/方块翻转；非 biome/terracotta 机制问题（biome tie-break 已修）。**candidate 待立项验证**。
- **修复后剩余**：8576 99.9993%→**99.9994%**（24→22）；3200 99.9997% 零退化；-288 95.7376% 结案基线（结构/FEATURE 假 diff，不属本课题）。
- **20000 基线修正（重要）**：8/7 深夜记录的 99.9997% 已过时——8/8 HEAD 实测 **99.9989%**；git stash 实验确认 18 块差异在 8/8 HEAD 就存在（非本批改动引入），与 river/taiga 边界插值差同类 → **并入 21 块 finalDensity 课题，不新立方向**。

---

## 2026-08-09 已验证结论（追加 4）：-288 课题破案 + FEATURE 范围决策

> 完整调查链（14 轮：量化→密度层→aquifer→Beardifier→caves 树→AQF-APPLY）见 `.investigations/-288-reopen/`（analysis-phase2..13 + summary-final.md）。本节只记结论。

### ✅ -288 破案：C++ 核心无 bug（AQF-APPLY 铁证）
- **Java aquifer.apply 直接调用**（cns 游戏同构遍历 + CellCache 真实值）(-278,12..23,-240) **全部判 solid**，density 与 C++ 逐位一致（0.055724~0.068693）——Java aquifer 与 C++ 完全一致，「aquifer 判水 bug」假设推翻
- **含水层 water/洞穴 air 来自洞穴雕刻（CaveCarver）阶段**：NOISE-BLK chunk status 显示 chunk(-18,-15)=`minecraft:carvers`——carvers 挖洞（y=23 air）+ 液面以下填水（y=15-19 water）；C++ 未实现 carvers → 判 stone
- **-288 差异构成（67042 块）**：已闭合（范围外 FEATURE/STRUCTURE）≈73%——岩石替换矿脉 ~51%（ore_granite/tuff/diorite/andesite + coal_ore placed feature）+ 洞穴雕刻 carvers ~17%（挖洞 + 液面填水）+ 结构 ~3.6%（含 Beardifier 抬 density 的岛）+ 树/草 ~1%；**未完全闭合 ≈23%**（judge 审查确认）：海底边界 ~11.6%（water↔stone/dirt/sand，候选 surface 海底 gravel/砂染色、结构岛相关）、gravel ~7.3%、表面规则 ~4.3%——待后续定位
- **8/8 结案修正**：方向正确（差异 = C++ 范围外功能），机制补充完整（含水层 = carvers 液体填充，05 篇 L86 早有记载；岛 = 结构 Beardifier；granite/tuff = ore_* placed feature）；时间线 L670「base_3d_noise 负坐标差」再次确认 = RouterProbe 独立构建假象（03 篇 L100 deriver 排除）

### ✅ FEATURE 复刻范围决策（2026-08-09 用户拍板）
- **只做地形性 FEATURE：carvers（洞穴雕刻）+ 岩石替换（ore_granite/tuff/diorite/andesite）**——影响玩家可见地形（洞穴可进入、岩层外观），原「FEATURE 全放弃（装饰影响小）」低估
- 矿石（coal/iron/copper）、树/草/花、结构（村庄/神庙/矿井）：**暂缓**
- **暂缓实施**（用户明确不急着做）——数据已就绪（worldgen/data 有 configured_carver/configured_feature/placed_feature），实施需 Phase 0 架构设计

### ⚠️ 方法沉淀
- **NOISE-BLK 探针**（BlockProbe NOISE 阶段 chunk.getBlockState 直读）是「块级真相」权威来源——反射（CellCache/AQF-J）受缓存污染不可信，NOISE-BLK 直读 chunk 状态不受污染
- **AQF-APPLY 探针**（cns 游戏同构遍历 + aquifer.apply 直接调用）验证 aquifer 判定——反射 CellCache 值不可信（L750 铁律），但 cns 遍历填 cache 后直接调 apply 判定可信
- **chunk status 检查**（NOISE-BLK 打印 getStatus）——确认读的是哪个阶段（noise/carvers/surface），防止把 carvers/FEATURES 产物误当 aquifer 判定

> 完整二分排查链（症状→线程数→攒批→fillChunk→wg_create 阶段→对照→VEH 根因）见 10-timewise-archive.md「2026-08-08 晚」条目。本节只记结论。

### ✅ 根因结论：AddVectoredExceptionHandler（VEH）在 JVM 进程不可用
- 崩溃日志铁律（全局崩溃捕获）用 `AddVectoredExceptionHandler` 装 VEH + StackWalk64/打印——**干扰 JVM 的硬件异常处理**：
  - JVM 的 JIT null-check、GC guard page、写屏障都是 SEH 异常（**预期异常**，正常控制流的一部分）；VEH 在 SEH 之前执行（第一顺位），且 VEH 里做 StackWalk64/打印重活 → JVM 内存被破坏
  - 崩坏形态：Server thread 堆损坏（Java 对象字段变垃圾、metadata 被当代码执行、栈被 0xDEADDEAF 覆盖）、jvm.dll 连锁崩溃
- **独立原生进程（block_probe/got_export）不崩**——无 JVM 异常模式，VEH 可安全使用
- **用户机器 D:\MC 的 0x34001 崩溃 = 同根因**（1.0.17 客户端 = C++ 接管 + VEH）

### ✅ jvm.dll 检测规则（worldgen_api.cpp wg_create L292）
```cpp
// 独立进程装 VEH（block_probe/got_export 崩溃日志）；JVM 进程不装（jvm.dll 已加载检测）
if (!GetModuleHandleA("jvm.dll")) wg::installCrashHandler();
```
- **判定依据 = jvm.dll 是否已加载**（`GetModuleHandleA("jvm.dll")`），不靠进程名/命令行
- JVM 进程 = jvm.dll 已加载 → 不装 VEH；独立原生进程 = jvm.dll 未加载 → 装 VEH

### ✅ hs_err 兜底（仍满足「崩溃可定位」铁律）
- JVM 侧崩溃由 JVM 自带 hs_err 文件兜底（含 native 栈 dll 偏移，可定位崩溃点）
- VEH 增强（module base RVA + stack-window + 0xDEADDEAF poison 标记）保留给独立进程；JVM 进程不需要

### ✅ 顺带修复/增强（本次调试中保留）
- **build.gradle dll 同步源错误**：processResources 的 `../cpp/build-msvc` 指向 MC 侧历史旧 cpp（非 CoreSwap）→ 打包旧 dll（1.0.2/1.0.6 同款坑复发）；改 `E:/PYTHON/CoreSwap/versions/1.20.1/cpp/build-msvc/bin/worldgen.dll`
- **processResources UP-TO-DATE 不重同步**：doFirst 的 copy 不算 task input → dll 更新后 gradle 跳过 → 服务器加载旧 dll（sha 不匹配排查半天）；规避：手动 Copy resources 或 --rerun-tasks
- **gradle daemon env 缓存**：$env:CORESWAP_THREADS 传给 daemon 不重启不生效（fork 的 JVM 继承 daemon 启动时 env）→ 用 -P 属性（vmArg 映射）或重启 daemon
- **gradle 8.13 -D 参数解析**：`gradle runServer -Dcpp.replace=1` 被拆成任务（`.replace=1 not found`）→ 用 build.gradle 的 -PcppReplace → vmArg 映射
- **crash handler 增强**（本次加，保留）：module base 打印（崩溃 RVA 定位）、stack-window 打印（RSP±0x50 qword + 0xDEADDEAF poison 标记）、WG_FBLOG（fillBlocks 批次日志 env 开关）
- **CppBridge 诊断增强**（保留）：-Dcpp.noBatch env 兜底 CORESWAP_NOBATCH

---

## 2026-08-10 已验证结论（追加 5）：FEATURE 实施（CARVERS + FEATURES 阶段 Phase 1-5 + 树花植被废弃）

> 状态：candidate（2026-08-10 深夜验证基线确认；未经 judge 审查，未授予 confirmed）
> **证据源**：`.investigations/feature-pipeline/pipeline-map.md`（管线地图）+ `cmd-output/*.txt`（各阶段实测）+ `cpp/worldgen/src/*.h`（代码注释锚点）+ `cpp/worldgen/deprecated-vegetation/README.md`（废弃决策归档）。
> **文风约定**：每条结论附验证方式；未验证推断明确标注为 candidate/待立项。

## 功能目的

补全 C++ worldgen 的 `SURFACE → CARVERS → FEATURES` 两阶段（Java ChunkStatus 链尾部）：

- **CARVERS**：洞穴雕刻（`cave`/`cave_extra_underground`/`canyon`）——把 aquifer 判定的实心块按洞穴体挖空，液面以下填水/岩浆。此前 -288 差异的「洞穴空气 + 含水层水」即缺此阶段（约 17% 差异构成，见 07 篇追加 4）。
- **FEATURES**：地形性装饰——岩石替换（ore_granite/tuff/diorite/andesite，Phase 3）、简单装饰（disk/spring/freeze_top_layer/underwater_magma，Phase 4）。树花植被（flower/random_patch/simple_block/tree）**已废弃**（Phase 5 验证未达标，2026-08-10 用户拍板，2026-08-10 深夜代码迁移 deprecated-vegetation/）。
- **模式隔离**：默认 `SURFACE` 模式不进入 FEATURES（8576/3200 零退化铁律）；`-features`（`WG_GEN_MODE=full`）才对照 FULL 状态参照（-288/300515）。

## 1.20.1 工作机制（Java 类 → C++ 文件映射）

### CARVERS 数据流

```
NoiseChunkGenerator.carve（L278-327）
  → ChunkRandom(new CheckedRandom(RandomSeed.getSeed()))   ← 基类 = CheckedRandom（48 位 LCG）
  → 17×17 邻域 chunk 循环（j,k ∈ [-8,8]）
    → 每邻域查 biome → GenerationSettings.getCarversForStep(AIR)
    → setCarverSeed(worldSeed + l, cx2, cz2)               ← l = carver 列表序号
    → shouldCarve（nextFloat() <= probability）
    → CaveCarver/RavineCarver.carve → carveRegion 逐点：
        getState → aquifer.apply（液面判定：y <= lavaLevel 直接放岩浆）→ materialRule 补丁
  → CarvingMask（per carverStep air/liquid 各一，BitSet(256*height)）
```

C++ 映射（`worldgen_api.cpp applyCarversAndFeatures` + `carver.h`）：

| Java 类 | C++ 文件 | 说明 |
|---|---|---|
| `CheckedRandom`（48 位 LCG） | `carver.h`（`CheckedRandom`） | `setSeed` 截断 48 位；`next(bits)=(seed*0x5DEECE66D+0xB)&mask48 >>> (48-bits)` |
| `ChunkRandom`（setCarverSeed） | `carver.h`（`ChunkRandom`） | Checked/Xoroshiro 双基类路径，`nextLong` 两次取高 32 位拼接 |
| `CaveCarver`（carveTunnels 递归） | `carver.h` `CaveCarver` | 递归子分支 `Random.create(seed)` = **CheckedRandom**（根因见 §3） |
| `RavineCarver`（canyon） | `carver.h` `RavineCarver` | `createHorizontalStretchFactors`/`getVerticalScale`/`isPositionExcluded` |
| `CarvingMask` | `carver.h` `CarvingMask` | FEATURES 阶段 `carving_mask` modifier 跨阶段读取 |
| `CarverContext`（surface 补丁） | `carver.h` `CarverContext` | materialRule 单点求值（复用 surface 规则） |

**种子公式**（ChunkRandom.java:87-93）：`setSeed(worldSeed); l=nextLong(); m=nextLong(); n=chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)`。验证方式：`chunkrandom_probe_run1.txt` 中 `setCarverSeed` 输出与 Java 对拍。

### FEATURES 数据流

```
ChunkGenerator.generateFeatures（L334-423，不在 NoiseChunkGenerator）
  → blockPos = (chunkX*16, bottomY, chunkZ*16)
  → setPopulationSeed(worldSeed, blockX, blockZ)            ← Xoroshiro128PlusPlus 基类（与 carver 的 Checked 不同！）
  → 收集 3×3 邻域 biome（C++ 简化 = 当前 chunk biome）
  → i = PlacedFeatureIndexer 结果长度
  → for k（step 0..10）:
      intSet = 各 biome 的 step k features → indexMapping（lastIndex）去重
      排序 → for p : intSet:
        setDecoratorSeed(l, p, k)                            ← p = indexMapping lastIndex（非 featureIndex！）
        placedFeature.generate(...)                          ← positions 链深度优先 flatMap
```

C++ 映射（`worldgen_api.cpp` FEATURES 段 + `feature_loader.h` + `placement.h` + `feature.h`）：

| Java 类 | C++ 文件 | 说明 |
|---|---|---|
| `ChunkRandom.setPopulationSeed/setDecoratorSeed` | `feature_loader.h`/`worldgen_api.cpp` | Xoroshiro 基类：`next(bits)=(int)(base.nextLong() >>> 64-bits)`；`setPopulationSeed` 里 `nextLong()` 两次取高 32 位拼接（共 4 轮 Xoroshiro 输出） |
| `PlacedFeatureIndexer` | `feature_loader.h` `PlacedFeatureIndexer` | featureIndex（首现递增）+ stepFeatures + lastIndexMap；`p = lastIndex` |
| `PlacedFeature.generate`（flatMap 链） | `placement.h` `PlacedFeature::generate` | **深度优先**递归 visit（见 §3） |
| 15 个 `PlacementModifier` | `placement.h` | count/in_square/height_range/heightmap/random_offset/carving_mask... |
| `OreFeature`/`ScatteredOreFeature` | `feature.h` | 椭球矿脉/撒点 |
| `DiskFeature`/`SpringFeature`/`FreezeTopLayerFeature`/`UnderwaterMagmaFeature` | `feature.h` | Phase 4 简单装饰 |
| `ConfiguredFeature`（type 分发） | `feature_loader.h` `ConfiguredFeature` | ore/disk/spring/freeze/underwater_magma 走 generate/generateOther |

**种子公式**（ChunkRandom.java:54-78）：
- `setPopulationSeed`: `l = nextLong()|1L; m = nextLong()|1L; n = blockX*l + blockZ*m ^ worldSeed; setSeed(n)`（**|1L 保证奇数**）
- `setDecoratorSeed(pop, index, step)`: `setSeed(pop + index + 10000*step)`（C++ 展开 `(long)k*65713L + 11L + (long)p*985L + l`）

**GenerationStep.Feature 顺序**（ordinal 0..10）：raw_generation / lakes / local_modifications / underground_structures / surface_structures / strongholds / underground_ores / underground_decoration / fluid_springs / vegetal_decoration / top_layer_modification。

## 关键根因与修复（按 Phase）

### Phase 0：基线（8576/3200 SURFACE 零退化铁律）

- **铁律**：SURFACE 模式（`WG_GEN_MODE` 未设 full）**绝不调用** `applyCarversAndFeatures`（`worldgen_api.cpp` L864-867 注释 + `fillOneChunkCore` runFeatures 分支）。任何 FEATURE 改动不得影响 8576/3200。
- 验证方式：每个 Phase 结束跑 `block_probe` 8576/3200 SURFACE 对照，TOTAL 必须保持 99.9994%/99.9997% 不变（`phase1_baseline.txt`、`phase4_result.txt` 均记录零退化）。

### Phase 1：`-features` FULL 模式与 SURFACE 模式隔离

- `block_probe -features` → `_putenv_s("WG_GEN_MODE", "full")`；C++ `wg_create` 读 env 选生成模式（0=SURFACE 默认 / 1=FULL + CARVERS→FEATURES）。
- **Phase 1 验证**（`phase1_baseline.txt`，2026-08-10）：seed 8576 SURFACE 99.9994% 与 FULL `-features`（stub 空，FEATURE 无产出）**逐位一致**——证明 FULL 模式开启本身不破坏 SURFACE 路径；-288 同理（96.4219% 与 SURFACE 一致）。此即「stub 空 = 与 SURFACE 一致」的隔离验证。
- 验证方式：`phase1_baseline.txt` 同 seed 双模式对照逐位一致。

### Phase 2：CARVERS——CheckedRandom 48 位 LCG（carver 挖洞错位根因）

- **根因**：`CaveCarver.carveTunnels` / `RavineCarver.carveRavine` 内部 Java `Random.create(seed)` = **CheckedRandom（48 位 LCG）**，不是 Xoroshiro。C++ 曾误用 XoroshiroRandom → 漂移序列全错 → 挖洞位置不重合（修复前重合仅 **12%**，2042/16668）。
- **修复**：`carver.h` carveTunnels/carveRavine 内部 `XoroshiroRandom → CheckedRandom`（L489/L553 注释锚点）。
- **成果**（seed=-8248318472910187742, -288,-256 4×4，FULL 参照含 carver）：
  - SURFACE 模式（无 carver）：93.4462%；FULL 模式（carver 开启）：**93.9442%**（carver 闭合 +0.5%）
  - 挖洞对比：我们挖 17300 vs 参照洞 17573（量匹配），重合 11929（**69%**）
  - 剩余差异：挖多 5371 / 挖少 5644（对称，浅层 y=8-43，carveRegion 边界微差 candidate）
- **修复链其他项**：block_probe biome 段跳过 bug（blen<128 截断）→ 参照读取错误；BlockProbe 预生成 17×17 邻域（逐 chunk 生成 carver 静默跳过）；carveCave 范围判断用 targetChunkX/Z（Java carveRegion 内部 chunk.getPos()）；mathSin/mathCos 查表（65536 项 SINE_TABLE）；MathHelper.sin 参数 float π（3.1415927F 全程 float）；getState density=0.0 走液面链（3b density>0 直接 solid，carver 首次暴露液面链路径——已验证 d 逐位一致）。
- 验证方式：`phase2_carvers_result.txt` + `chunkrandom_probe_run1.txt`（CheckedRandom next/nextLong/nextInt 输出与 Java 对拍）；挖洞重合率从 12% → 69% 量化。

### Phase 2 附属：canyon 两处修复（RavineCarver）

- **修复 1**：`createHorizontalStretchFactors` 的 `fs[j] = f * f`——Java RavineCarver.java L122 是平方，C++ 曾漏平方 → ravine 挖更宽（`carver.h` L592 注释锚点）。
- **修复 2**：`carveRavine` 内部 `Random.create(seed)` = **CheckedRandom**（与 carveTunnels 同根因；`carver.h` L553 注释锚点）——RNG 漂移直接决定 canyon 走向与宽度。
- 验证方式：代码注释锚点 + `phase2_carvers_result.txt` 记录 canyon 在 -288 区域无贡献（prob 0.01 低，需在 canyon 概率高区域另设验证，candidate）。

### Phase 3：Ore——positions 链深度优先（Java stream.flatMap 惰性）

- **现象**：-288 FULL 96.67%、300515 96.59%，granite 匹配仅 **56.2%**（`phase3_ore_result.txt`）。
- **根因**：Java `PlacedFeature.generate` 是 `Stream.of(pos)` 链式 **惰性 flatMap**——「位置 1 走完所有 modifier → 位置 2 走完所有 modifier」= **深度优先**；C++ 若「modifier 全展开再下一个」= 广度优先 → 随机消费顺序不同 → `height_range` 的 y 全错（granite 位置错）。
- **修复**：`placement.h` `PlacedFeature::generate` 改为递归 `visit(mi, x, y, z)`——先取当前 modifier 的 getPositions，对每个位置递归进入下一个 modifier（L324-339 注释锚点 + 实现）。
- 验证方式：`phase3_ore_result.txt`（granite 56.2% 定位）+ 修复后 `phase35_crosschunk_result.txt`（granite **88.3%**、diorite 85.7%、tuff 87.8%、dirt 92.7%）。

### Phase 3 附属：p = PlacedFeatureIndexer.lastIndex

- Java `Util.lastIndexGetter`：`p = indexMapping(feature)` = feature 在 `stepFeatures[step]` 中的 **lastIndex**（`map.put` 覆盖 → 最后出现索引），**不是 featureIndex**（全局首现递增号）。
- C++：`feature_loader.h` `PlacedFeatureIndexer` 三表（index / stepFeatures / lastIndexMap）构建 lastIndexMap（`lastIndexMap[st][stepFeatures[st][i2]] = i2`），`intSetFor` 返回 lastIndex 集合，`setDecoratorSeed(populationSeed, p, k)` 的 p 用 lastIndex（`worldgen_api.cpp` L1296-1302 注释锚点 + `feature_loader.h` L99-100）。
- **关键**：Java 拓扑排序（TopologicalSorts）保证 vanilla 无 cycle → featureIndex 升序；C++ 按 biome 列表序 + step 升序近似。若未来全量 JSON 引入 cycle 会崩溃（DataFixer 校验），需与 Java 的 indexMapping 数值一致。
- 验证方式：代码锚点（feature_loader.h L99-100）+ 与 Java 参照对拍 p 值。

### Phase 3.5：两阶段 FEATURE + pendingCross 跨 chunk

- **问题**：FEATURE（如 ore 椭球）跨 chunk 读写，单 chunk 局部生成读不到邻域已写方块 → granite 等 target 判定错。
- **方案**（`worldgen_api.cpp wg_fill_blocks_multi_phase` + `feature.h` `OreFeatureContext`）：
  - **phase 1**：surface+carvers 并行全部完成后，每 chunk 的 col 存 `regionCols`（`map<pair<int,int>, vector<int32_t>>`，mutex 保护）；
  - **phase 2**：features 阶段**强制串行**（`threads = 1`）重跑，`regionColAt(cx,cz)` 从区域缓存取邻域 col 做 target 判定读；跨 chunk 写入走 `pendingCross`（`map<pair<int,int>, vector<pair<int,int32_t>>>`）记录 `(idx, state)`；
  - 全部 fill 完成后统一应用 pending：**A 后生成覆盖 B**（Java 语义）——`for (auto& [key, list] : pendingCross) for c in count: if match → o[idx] = state`（L1044-1082）。
- **成果**：-288 FULL **97.8464%**（nonAir 93.65%）、300515 **98.0948%**（94.06%）、granite 88.3% / diorite 85.7% / tuff 87.8% / dirt 92.7%（`phase35_crosschunk_result.txt`）。
- 验证方式：`phase35_crosschunk_result.txt` + `worldgen_api.cpp` L1044-1082（两阶段实现注释）。

### Phase 4：简单装饰 + HeightmapPlacementModifier 返回 top 不 +1

- **实现**：DiskFeature / SpringFeature / FreezeTopLayerFeature / UnderwaterMagmaFeature（CaveSurface 语义）+ block_predicate_filter + surface_relative_threshold_filter + IntProvider uniform **value 嵌套修复**（JSON `{"type":"minecraft:uniform","value":{...}}`——min/max 在 value 子对象，修复前 count=uniform(44,52) 被错误解析 → magma 0 → 43）。
- **结果**（`phase4_result.txt`，Phase 4 完成时中间快照）：-288 FULL **97.8390%**（Phase3 97.8464% → -0.007%，magma 位置错引入 ~20 块）；300515 FULL **98.0975%**（Phase3 98.0948% → +0.003%，disk/spring 正确放置）；8576/3200 SURFACE 零退化保持。
- **演进注**：Phase 4 快照 97.8390% → 最终基线 **97.8460%**（+0.007% ≈ 110 块）来自 Phase 5 禁用 `random_selector`（trees_*）分支（`worldgen_api.cpp` JUDGE-DIAG 注释）——禁用树后其错误位置生成消失，FULL 微升；最终基线见下节验证基线表。
- **HeightmapPlacementModifier 返回 top 不 +1**（`placement.h` L195-213）：
  - Java `Heightmap` 存 **topY + 1**（高度图语义），`HeightmapPlacementModifier.getPositions` 返回 `topY(heightmap, x, z)`（不额外 +1；k > bottomY 才返回）。
  - C++ 内部高度图存「块 y」（surface 内部消费需要 y 语义），HeightmapPlacementModifier 直接返回 C++ top（不 +1），与 Java 的 y+1 差 1。
  - **实测 +1 反而使 300515 降 0.12%**（disk/spring 变差）→ 保持 C++ y 语义（内部一致性优先）。生态装饰（花/草）已按拍板范围外移除，不依赖此语义差异。
  - 验证方式：`placement.h` L195-213 注释复盘 + `phase4_result.txt` 300515 +0.003%（disk/spring 正确放置）。
- **OCEAN_FLOOR_WG 高度图构建时机**：carver **前**（Java NOISE 阶段语义，挖洞不影响海底 top）——`worldgen_api.cpp` L1233-1234 注释锚点。

### Phase 5：树花植被——验证未达标 → 废弃（2026-08-10 拍板 + 2026-08-10 深夜迁移 deprecated-vegetation/）

- **曾实现并接入**（2026-08-10 Phase 5）：SimpleBlockFeature / RandomPatchFeature（花/草）/ TreeFeature（oak/birch 直树 + fancy_oak 简化）/ RandomSelectorFeature。
- **验证未达标**（`deprecated-vegetation/README.md` 历史事实）：
  - **树只放 40%**：canGenerate 失败率高（origin ground 检查 / 树干空间检查失败）；
  - **300515 花爆炸**：dandelion C++ **533** vs 参照 **11**——树未实现 → 树冠区被当 air 放花；
- **废弃决策**（2026-08-10 用户拍板，README + feature_loader.h L67-70/L89-90 + worldgen_api.cpp L1360-1361 注释锚点）：
  1. **细节版本改动太多**——树/花/草植被在 MC 版本间差异大（1.20 → 1.21 大量变动），逐位对齐成本不可接受；
  2. **MOD 特别容易碰到**——实机 Mod 装饰主要挂 FEATURES 阶段，C++ 全接管会丢 Mod 花/草/树，兼容工作量不可接受。
- **2026-08-10 深夜代码迁移**：实现代码剪出到 `cpp/worldgen/deprecated-vegetation/`（vegetation_features.h），主代码彻底移除接入点：`feature_loader.h` `generateOther` 对 flower/random_patch/simple_block/tree return false；`worldgen_api.cpp` random_selector return false；不参与编译、不接入调度。
- **恢复路径**：git 历史 c04768e 前的 feature.h 有完整版本；恢复需重新接入 feature_loader.h 分发 + worldgen_api.cpp 调度 + placement.h 植被 modifier，并重跑 Java 对拍。
- 验证方式：`deprecated-vegetation/README.md`（废弃状态 + 历史事实 + 禁用后基线）；代码锚点（generateOther return false / 不解析树花 config）。

## 验证基线（2026-08-10 深夜实测，block_probe 逐位对照）

| 场景 | seed | 坐标 | 模式 | TOTAL | nonAir | 备注 |
|---|---|---|---|---|---|---|
| 8576 | 8576294172403134396 | 720,-432 | SURFACE | **99.9994%** | 99.9986% | 零退化铁律（含 FULL -features stub 逐位一致） |
| 3200 | -8248318472910187742 | 3200,3208 | SURFACE | **99.9997%** | 99.9992% | 零退化铁律 |
| -288 | -8248318472910187742 | -288,-256 | FULL（-beard） | **97.8460%** | 93.6490% | 含 CARVERS + 岩石替换 + 简单装饰；参照 FULL 状态 |
| 300515 | 3005152118058349760 | -1320400,-198064 | FULL | **98.0975%** | 94.0641% | 陆地 flower_forest/plains 区域 |

基线数据来源：`phase0_baseline_m288.txt`（-288 FULL 97.8460%/93.6490%）、`phase0_baseline_300515.txt`（300515 98.0975%/94.0641%）、`phase1_baseline.txt`（8576/3200 SURFACE 99.9994%/99.9997%）、`deprecated-vegetation/README.md`（禁用后基线确认）。各 Phase 演进见 §3；-288/300515 的剩余差异构成见「版本敏感点/已知限制」。

## 版本敏感点 / 已知限制

### 版本敏感点（升级 1.21 必须复查）

- [ ] **随机数基类语义**：CARVERS 用 `CheckedRandom`（48 位 LCG）、FEATURES 用 `Xoroshiro128PlusPlus`——两者 `setSeed` 对 worldSeed 的消化不同（LCG 截断 48 位 / createXoroshiroSeed），C++ `ChunkRandom` 双基类路径必须分别实现、勿混用（`pipeline-map.md` ⚠ 块 + 附录 A）。
- [ ] **`setPopulationSeed` 的 `|1L`**：保证 l/m 为奇数——漏写会导致 feature 随机序列整体漂移（candidate 已验证到 Xoroshiro 输出轮次，见 pipeline-map L213）。
- [ ] **Heightmap 语义差**：Java 高度图存 y+1，C++ 存块 y——当前 HeightmapPlacementModifier 不 +1 且实测正确；若未来接入依赖「高度图 y+1」的生态装饰（花/草），必须重新评估（已按拍板范围外移除）。
- [ ] **PlacedFeatureIndexer 拓扑序**：C++ 按 biome 列表序近似 Java 拓扑排序；若 JSON 数据引入 feature order cycle（DataFixer 校验），indexMapping 会不一致——需与 Java 对拍或导出 indexMapping。
- [ ] **`carving_mask` 跨阶段状态**：FEATURES 阶段读 CARVERS 的 mask（ProtoChunk 持有）——C++ 需保持 per-chunk mask 存活到 FEATURES。
- [ ] **structure 部分跳过**：generateFeatures 的结构阶段（setDecoratorSeed(l, m, k)）C++ 未实现（-288 深海无结构影响）；村庄/矿井区域需补 structure 序号语义。

### 已知限制（candidate 记录，非 bug）

| 限制 | 影响 | 说明 |
|---|---|---|
| carver 31% 剩余差异 | 挖多 5371 / 挖少 5644，浅层 y=8-43 | 对称，carveRegion 边界微差或 mask 交互，非机制级；待新区域验证 |
| canyon 覆盖不足 | -288 区域无贡献（prob 0.01 低） | canyon 两处修复已在代码层，需高概率区域对拍（待立项） |
| magma 位置重合 0 | -288 FULL -0.007%（~20 块） | Java BiomePlacementModifier 过滤（cold_ocean）C++ 不过滤 + origin 依赖洞穴水位置（Phase 2 carver 差异 31% 连锁） |
| disk state_provider 简化 | 有限 | sandstone 分支未实现（简化 fallback） |
| FreezeTopLayer 用 OCEAN_FLOOR_WG 近似 MOTION_BLOCKING | -288 温度高无冻结，无影响 | 其他温度带需验证 |
| noise_based_count 简化 | Phase 3 简化 noise=0 | 依赖 `minecraft:foliage` 噪声参数，未注册时 count 偏差 |
| 树花植被已废弃 | 参照的树/花方块 = 已知预期差异 | 用户拍板范围外；树 40% 失败 + 300515 花爆炸（dandelion C++533 vs 参照 11）为废弃前实测 |

## 方法沉淀（本课题新增铁律/探针）

- **FEATURE 探针**：`block_probe -features`（FULL 模式）+ `WG_FEATURELOG`/`WG_CARVERLOG`（origin/mods 日志）+ `-save`（生成 blocks 文件对比）。RNG 层先验证（CheckedRandom/Xoroshiro 输出），再 placement 位置，最后方块结果。
- **参照状态审计**：8576/3200 参照 = SURFACE 状态（纯核心差异）；-288/300515 参照 = FULL 状态（混 FEATURE）——对比前必须判定参照状态，不同状态差异构成完全不同（07 篇追加 4 已记）。
- **两阶段验证**：FULL 模式跨 chunk 用 `wg_fill_blocks_multi_phase`（phase1 存 regionCols / phase2 串行 + pendingCross）——A 后生成覆盖 B（Java 语义），不要用单阶段逐 chunk。

## 2026-08-13 spline 扁平化 + 边界列复用（无损优化 + 多线程膨胀重新定性）

> 状态：draft（judge 语义无损通过 + 零退化已落盘；多线程课题未闭合）
> 来源：`.investigations/perf-rework/`（phase0-quantify / phase0-hotspot-analysis / phase0-interp-measurement / phase1-design / static-audit-c2me-steel）+ commit aae119d（density 代码）/ ae9a3b9（phase0-2 产物）+ `cmd-output/phase0_baseline_8x8.txt` / `phase1_splineflat_8x8.txt` / `phase2_edgereuse_8x8.txt` / `regress_8576_aae119d.txt` / `regress_3200_aae119d.txt` + judge `review-aae119d.md` / `review-aae119d-followup.md`

承接 2026-08-12 修复闭环（FlatCache 上下文绑定，spline 调用量回 5,906/chunk、单线程 wall 2,910ms）。本轮在「多线程内存带宽饱和优化」课题下做两个无损优化 + 一次根因重新定性。

### 优化 1：SplineDF 树扁平化（主要收益，单线程 -24% 零退化）

**改动**：SplineDF 从递归 `shared_ptr<SplineDF>` 树改为连续节点数组（`nodes/locations/derivatives/subIdx/locationFunctions` 池）+ 整数索引，采样从递归虚调用 `apply` 改为非虚递归 `sampleNode`。Hermite 插值公式逐位不变（judge 逐行核对 n==1 / i<0 / i==n−1 / min-max 全边界等价）。

**实测**（`bench_chunks 8×8`，analyze_stagetimer 聚合 n=128）：

| 指标（单线程） | 基线 | 扁平化后 | 变化 |
|---|---|---|---|
| density wall（median） | 61.7ms | 47.1ms | **-23.7%** |
| [A] threads=1 吞吐 | 92.08 ms/chunk | 71.68 ms/chunk | **-22.2%** |

**零退化**（block_probe 单线程逐位）：8576 SURFACE 99.9994%（3538922/3538944）、3200 SURFACE 99.9997%（1572860/1572864）——`regress_8576_aae119d.txt` / `regress_3200_aae119d.txt` 落盘。

### 优化 2：InterpolatedDF 边界列复用（收益小，-1.7% 接近噪声）

**改动**：thread_local edge 缓存复用左邻 chunk 的 gx=4 列作为当前 gx=0 列（CELL_X=4 坐标对齐，采样纯函数 → 逐位无损）。

**实测**（单线程）：

| 指标 | 扁平化后 | 边界复用后 | 变化 |
|---|---|---|---|
| density wall（median） | 47.1ms | 46.3ms | -1.7%（接近噪声） |
| [A] threads=1 吞吐 | 71.68 ms/chunk | 72.06 ms/chunk | +0.5%（无改善） |

**根因（为什么收益小）**：InterpolatedDF::buildGrid 耗时大头是「每 chunk 每实例 1 次的 FlatCache buildGrid 构建触发 + spline 树遍历」，**不集中在 gx=0 列**——FlatCache buildGrid 只在首个角点触发一次，跳过 gx=0 列只是把触发点移到 gx=1 列，省不了；gx=0 列其余 244 角点是 FlatCache 查表命中（快）。边界复用优化了错误的目标（角点采样次数，而非树遍历触发点）。且实现只做 x 方向左邻列（上限 245/1225=20%），未达 phase1-design 预估的「x/z 双向 -36%」。

### 多线程膨胀重新定性：bandwidth-bound → latency-bound（DDR5）

**旧定论失效**：此前「8 线程 ~17.8GB/s ≈ DDR4 带宽上限 → 带宽饱和」基于错误的内存类型假设。用户纠正内存为 **DDR5-5600 双通道**（~85GB/s 有效；CPU Ryzen 9 7845HX 12 物理核）后，17.8GB/s 远低于有效带宽 → 非 bandwidth-bound。

**重新定性 latency-bound（cache miss 延迟）**：8t 下 spline 单次 10×（深递归指针链 cache miss 高）vs noise 仅 1.3×（噪声参数表相对局部）——**不对称膨胀**。若带宽饱和两者应同比例排队；实际只有 spline 膨胀 → 符合随机指针链 cache miss 延迟，非带宽对称争用。

**关键结论（扁平化未解决多线程）**：spline 扁平化后单线程 -24%，但多线程无改善（8t density median 460.8→478.3ms，不降反略升）——**多线程膨胀根因在 InterpolatedDF::buildGrid 的 1225 角点树遍历整体**（spline 递归 + FlatCache 查表 + noise 的 cache miss 叠加），不在 spline 递归本身。**待解决方向 = DFC（整个 DF 树扁平化；C2ME 1.21.3+ 引入，Rust SteelMC 静态分派等价物）**，非仅 spline 子树扁平化。

### 状态

- 代码语义无损：成立（SplineDF Hermite 公式逐位等价 + 边界复用 CELL_X=4 坐标对齐，judge 逐行核对通过）。
- 零退化：成立（regress_8576/3200_aae119d 落盘）。
- 状态：**保持 draft**——spline 扁平化单线程 -24% 是真实收益，但「多线程膨胀」课题未闭合，需重新定位 InterpolatedDF::buildGrid 树遍历的 cache miss 构成后再评估 DFC。

---

## 2026-08-16 影响评估修正：notify 丢失 bug 污染面 + 修复后重测 + clamp 发现

> 状态：draft（结论性落盘）| 来源：`.investigations/worldgen-mt-scaling/`
> 完整错误台账（五段式 + 判错经验 + 速查表）：`mt-scaling-errors.md`（MT1-MT7）；影响评估：`notify-bug-impact.md`；勘探：`scout-map.md`；本修正对应上文 L74/L97/L109 三处 ⚠️ 标注。

### notify 丢失 bug（0a781e1 修复）影响面摘要

- **bug**：CoreSwapPool ensure() 锁内建 worker + run() 入队后 notify_all() 竞争 → 补建 worker 错过通知永久等待（tasks 空 + stop false）→ 只有老 worker 干活 = **串行假象**（[A] T>1 顺序跑实际并行度=1）。引入 252d988（8/6 20:11），修复 0a781e1（8/15 23:50），**活跃约 9 天**。
- **影响**：8/11-8/15 所有 [A] T>1 顺序跑数据作废（含本文件 L74「108-239ms 反降」、L97/L109「×16」）；**单线程数据全部不受影响**（T=1 无补建）；**H2 主因（rebuild 168×）保留成立**（单线程精确统计）。
- **触发边界**：只影响 [A] 批量模式（count=N 线程数递增 → 补建 worker 空闲）；[B]/实机 count=1 不补建不触发（其「无并行」是 clamp 问题，见下）。

### 修复后重测（64-chunk 8×8 前台，bench-notifyfix-8x8-20260816.txt）

```
[A] threads=  1   98.02 ms/chunk
[A] threads=  8   89.88 ms/chunk   （-8.3%：不再反降，轻度加速）
[A] threads= 12   90.39 ms/chunk
[A] threads= 22   97.76 ms/chunk
[A] threads=  0   96.30 ms/chunk
[B] workers=  1   86.80 ms/chunk   （[B] 段 120s cap 截断，不影响 [A] 结论）
```

- **结论**：notify 修复后 [A] T=8 不再反降（比 T=1 快 8%），但**远未到 8× 加速**——「每 chunk 并发下慢」仍存在（第二阶段课题：fillOneChunkCore 并发下每 chunk 耗时随并发增长，WG_MTTRACE 证明 8 worker 真并行但批间 525ms ≈ 8×65ms；fprintf stderr 锁竞争污染待无 fprintf 计数器复测）。
- ⚠️ 与 scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」**矛盾**（中间状态 C1 版/计时污染混测，单线程基差 +37%）——待同机同状态对照（notify-bug-impact.md §5 #1）。

### [B]/实机 M=1 结构性串行（threads clamp 发现，candidate）

- **发现**：`wg_fill_blocks_multi` L1189 `if (threads > count) threads = count;`（**66e05f5，8/5 引入**，池化 c792e9d 后语义失效）→ count=1 时 clamp 到 1 → ensure(1) → **池恒 1 worker**。
- **实机推论（代码链路铁证，待实机实跑对比）**：CppBridge.java L170-171（count=1 + THREADS）→ jni_bridge.cpp L93（透传）→ clamp → ensure(1)：**实机 mod 每 worker 调 count=1 时即使传 THREADS=12 也被 clamp 到 1 → 实机「多线程」可能从未真正并行**（结构性串行）。
- **与 notify bug 独立**：notify 只影响 [A] 批量；clamp 影响 [B]/实机 M=1。
- **状态/待办**：candidate（代码链路已闭环，唯一剩余验证 = 实机实跑对比）；修复待办（clamp 改 `if (threads > count && count > 1)` 或实机改批量调用）见 `mt-scaling-errors.md` MT3。

---

## （追加小节）production density 并发 11× 争用归因 = 长串行依赖链 + 内存子系统 latency QoS

> 承接本主题既有多线程性能记录：2026-08-12 根因（H2 主因 FlatCache 单槽缓存 + buildGrid 角点越界 → 嵌套递归蔓延）/ H3 放大器（thread_local thrashing）为**旧课题**；本节是 **density 11×（SplineDF/InterpolatedDF 采样在并发下的每 chunk 延迟膨胀）** 的归因。DFC 已在 DFC CPU 移植失败定论中作废（600× 慢，净作用为负）；locFn 连续化（Plan A）在 A/B 中证伪（放大比持平，非主导）。

### 一、现象（核心数据）

production density 单 chunk 延迟随线程数线性暴涨（conc_density_probe + WG_PHASETICK，12 固定 chunk，median density）：

| T | density 耗时 | 相对 T=1 |
|---|---|---|
| 1 | 37.83~39.31ms | 1× |
| 2 | 74.01ms | 2× |
| 4 | 174.33ms | 4.6× |
| 8 | 331.04~346.26ms | **8.4×~9×（单 chunk 9.2× / density 11×）** |

**关键区分（AGENTS.md 早已警告，本课题反复犯）**：
- **每 chunk 延迟** = density 阶段耗时 = 42.69 → 391.41ms（**9.2×**）——真实暴涨；
- **整批吞吐** = wall/chunk = 69 → 73ms/chunk——**几乎不变**；
- 多线程下 **wall/N 是吞吐均值，不是每 chunk 耗时**——吞吐正常 ≠ 并发无问题。
- **单点 0.4μs·快**（thread_local grid 懒建 + 每点纯 trilinear），并发才是问题。

### 二、排除链（production 模型确证级，同一探针 conc_density_probe / 同一 wg_worker pool / 只差一项改动）

| 试验 | 改动 | 放大比 | 结论 |
|---|---|---|---|
| BASE | — | 10.32× | 基线 |
| SERIAL | spline.locFn 存储连续化 | 10.25× | ❌ 存储非争用 |
| NOSPLIT | spline 递归→显式栈 | 9.9× | ❌ 递归非争用 |
| DEVIRT | spline.locFn 虚分派 devirtualize | 10.05× | ❌ locFn 虚分派非争用 |
| spline-only | 绕 wrapper 直采 spline（WG_SPLINE_FILL） | 1.62× | spline 无碍（占时间仅 9%） |
| warm | 预建 grid 排除 buildGrid | 10.10× | ❌ buildGrid 无碍 |
| **WG_FLAT_TOP** | 去 min/squeeze/mul 虚分派（4→2，**block_probe SHA256 逐位一致**） | 10.55× | ❌ **虚分派数无碍** |

**排除清单（一行式）**：11× 争用 **不是** 存储（SERIAL）、**不是** 递归（NOSPLIT）、**不是** locFn 虚分派（DEVIRT）、**不是** buildGrid 深链（warm）、**不是** 顶层 min/squeeze/mul 虚分派（WG_FLAT_TOP），**不是** spline 本身（spline-only 1.62×），**不是** 内存带宽（C7 DDR 1-2% 未饱和）、**不是** SMT（T=8 ≤ 12 物理核，各占独立核）、**不是** 写乒乓（共享全 const 只读）。

⇒ **11× 争用 = interp/noodle 采样内部**（内存访问模式）。

### 三、latency QoS 机制（candidate/推断）

scout 访存分析（interp-memory-access.md，dcf85758）确证：interp grid 全 **thread_local**（density.h:576-578，跨线程独立不共享）；跨线程共享全为**只读 const**（noiseSamplers/SplineDF 表 17KB/GRADIENTS 192B/finalDensity 节点字段），**无写共享/ping-pong**；C7 带宽 DDR 1-2% 未饱和；C4/C2 SMT 对 T=8 不触发（12 物理核无 core 共享）；noise 1.15×/spline 1.62× 证明共享读不是 10× 放大器。

⇒ **最一致机制 = 长串行依赖链 + 内存子系统 latency QoS**：
- 每点串行链：interp#1 grid（8 读）→ noodle 顶 range_choice 判定 → interpA（8）→ [out_range] interpB/C/D（24）→ 各级 range_choice/add/mul/abs/max 数学。**每级 load 结果喂下一级**（数据依赖）。
- 8 线程同时灌入这些长链 → 共享内存子系统（L2 到 L3 / OOO 窗口 / load-store 队列）的**延迟 QoS** → 每级 load 延迟被**非线性放大** → 串行链延迟膨胀（~10×）。这发生在**无锁 + 读共享 const + 真并行**三条件下。
- **是延迟（latency）而非吞吐（throughput）**被共享资源排队放大——不是共享读带宽饱和（C7 已否），不是写 ping-pong（表全只读）。

### 四、修复方向（latency QoS 下）

**提升 MLP**（打破长依赖链形态：并行多独立点 / 软件流水 / 分块减少每级数据依赖），**不是**减虚调用/存储/递归（已排除）。

**⚠️ 边界（关键）**：DFC 式全扁平直排能在 CPU 上消除并发放大（11×→1.3×），但**每点绝对成本 238μs → 整 chunk 600× 慢 → 净作用为负，已作废**。故「提升 MLP」必须在 **production 自身形态**上做（保留单点 0.4μs 快），**不是算法重写**（DFC 教训）。

### 五、待验证（M3）

执行 M3（interp-only grid-hit 隔离）验证「长链 latency QoS」。**M3 探针（wg_sample_interp）目前因自身 bug 未完成干净验证**（hit 慢 850× vs production 0.34μs/点，探针自身需 perf 定位：thread_local slots resize/坐标跨 cell/每次 buildGrid）。latency QoS 归因基于排除链 + 结构自洽（**间接**），**待 M3 或等价干净测量直接证实**。

- 若 M3 低（争用不在 grid 读）→ 指向长链依赖 → MLP 方向。
- 若 M3 高（争用在 InterpolatedDF 机制）→ 另查 thread_local vector / cacheId 索引 / allocator。

### 六、引用文件

- `.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`（主过程日志）
- `.investigations/worldgen-mt-scaling/wrapper-chain-measurement.md`（§6-8：spline-only / warm / WG_FLAT_TOP + 对拍）
- `.investigations/worldgen-mt-scaling/interp-memory-access.md`（scout 访存分析）
- `.investigations/worldgen-mt-scaling/wrapper-buildgrid-structure.md` / `topwrapper-sample-logic.md`（scout 结构）
- `.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（历史 11× 机制 + DFC 作废）
- `.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（错误台账本体；新增 ①-⑥ 见 `knowledge-drafts/draft-mt-errors-11x.md`）

## 2026-08-29 Rust CARVERS 阶段移植（WorldgenRust，commit bf3d851）

> Rust 全量重写 worldgen 的 CARVERS 阶段（洞穴雕刻）。把 C++ `carver.h`（661 行，CaveCarver+RavineCarver）移植到 Rust。语义要点与 C++ 完全一致（见上方 Phase 2 的 C++ 根因），此处只记 Rust 移植新增/复用的关键语义。

### Rust 移植关键语义（高价值，与 C++ 同源）

- **CheckedRandom 内部递归**：`CaveCarver::carveTunnels` / `RavineCarver::carveRavine` 内部 `Random.create(seed)` = **48 位 LCG（CheckedRandom）**，非 Xoroshiro（C++ 2026-08-10 已知根因，Rust 移植必须同样用 `CheckedRandom::new(seed)`）。可复用判据：MC 里 `Random.create(seed)` 默认实现是 `new CheckedRandom(seed)`（48 位 LCG），**不是** Xoroshiro——凡看到 `Random.create(...)` 派生内部随机源，先确认是 LCG 而非 Xoroshiro。
- **mathSin/mathCos 查表**：MC `MathHelper.sin/cos` 是 **65536 项 SINE_TABLE 查表**（`table[(int)(value * 10430.378F) & 65535]`），**不是** `std::sin`。`mathCos(value) = table[(int)(value * 10430.378F + 16384.0F) & 65535]`。任何用 `std::sin`/`f64::sin` 替代的实现都会在长循环里累积漂移（111 步漂移数格）。
- **carveTunnels 里 `(float)Math.PI` 全程 float**：`d = 1.5 + mathSin(3.1415927F * j / branchCount) * width`——`(float)Math.PI = 3.1415927F`（float π），不是 double π。float π 与 double π 在查表索引上差 1 位 → 漂移累积。
- **setCarverSeed 派生**：`setSeed(worldSeed); l=nextLong(); m=nextLong(); n=chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)`。`nextLong()` = `(long)next(32) << 32 + next(32)`（**有符号拼接**，MC-239059：j<0 时高 32 位被 0xFFFFFFFF 填充，非无符号位拼接）。
- **CarvingMask 索引**：`index = (x & 15) | ((z & 15) << 4) | ((y - bottomY) << 8)`，256*height 位集。
- **carveRegion 两套坐标**：洞穴中心 x/y/z 用**邻域 chunk**（chunkX/chunkZ 参数仅用于范围判断）；carveRegion 写方块用 **targetChunkX/Z（当前 chunk）**——两套坐标易混。
- **getState**：`y <= lavaLevel.getY(minY+8=-56)` → lava；否则 `aquifer.apply(pos, 0.0)`（density=0.0）。replaceable tag `#minecraft:overworld_carver_replaceables`（**含 water！**）。

### Rust 移植验证（candidate · Partial）

- 对拍 vanilla FULL 参照 `vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks`（seed=-8248318472910187742，4x4 origin -288,-256）：
  - 无 carver（surface-only）：match=95.41%，nonAir=86.89%
  - 有 carver：match=95.61%，nonAir=86.34%，Rust carved=8430，vanilla carved=6428，挖洞重合 **90.88%**（5842/6428）
  - Rust carver 挖洞 0 块在地表以上（正确，carver 只挖地下）
- 验证记录：`.investigations/carver-port/cmd-output/carver_probe.txt`（一次性数值，不写 docs）
- 错误台账：`.investigations/carver-port/carver-errors.md`（C1-C4，Rust 移植 C++ 的借用/所有权典型坑）

## 2026-08-29 Rust worldgen 作为 mod 运行（关键里程碑）

> Rust 全量重写 worldgen 后，把 Rust 块级管线作为 Minecraft mod 运行。三层链路：**Rust cdylib（C ABI）→ C++ JNI 桥（worldgen.dll）→ mod 加载（Java_wg_CppWorldgen_*）**。

### 架构（三层链路）

`
Rust WorldgenRust.dll（cdylib，导出 wg_* C ABI）
  ↑ LoadLibrary + GetProcAddress
C++ rust_jni_bridge.cpp → worldgen.dll（导出 Java_wg_CppWorldgen_* JNI 函数）
  ↑ JNI
Java wg.CppWorldgen（mod 加载，调用 init/fillBlocks/setBeardifier/densityParams）
`

- **Rust 侧**：worldgen_handle.rs（WorldgenHandle::create + fill_chunk_blocks，fill_chunk 宏观 → BlockColumn → build_surface → carver 17×17 邻域）+ pi.rs（C ABI 导出 wg_*）。Cargo.toml crate-type = ["cdylib", "rlib"]。
- **C++ JNI 桥**：ust_jni_bridge.cpp 加载 WorldgenRust.dll（LoadLibrary + GetProcAddress），导出 6 个 Java_wg_CppWorldgen_* 函数。JNI 桥 = **薄转发层**（JNI 数组 ↔ C 指针转换 + 调 wg_*），与 C++ jni_bridge.cpp 同构。
- **mod 加载**：Java 侧 JNI 调用 init/fillBlocks/setBeardifier/densityParams，与 C++ worldgen.dll 加载路径同构。

### 验证（三层递进，Partial 分层）

| 验证 | 结果 |
|---|---|
| dll_test.c（C ABI 导出） | wg_* 导出 OK |
| jni_dll_test.c（JNI 导出） | 6 个 JNI 函数导出 OK |
| handle_probe（WorldgenHandle vs vanilla） | 95.54% |
| **JniProbe（JNI 加载 Rust dll 生成 64 chunks）** | **match=93.76%**（y=64..319 100%，地下 71-90%） |

- **可复用判据**：「air 区 100% + 地下带 70-90%」签名 = 桥接正确 + 地下差异来自 worldgen 已知边界（carver 剩余差异 / FEATURE 范围外 / Beardifier 结构区），**非 JNI 桥引入**。与「air 区吻合 + ground 带全错 = 参照/种子配置错」签名互补。
- **逐层验证**：先证 C ABI（dll_test）→ 再证 JNI 导出（jni_dll_test）→ 最后全链路（JniProbe）——任一层失败先修该层，不跨层猜。

### 关键语义（可复用）

- Rust edition 2024 的 C ABI 导出：#[no_mangle] 需 #[unsafe(no_mangle)]。
- 裸指针跨线程 Send：*mut i32 不实现 Send，edition 2024 下 SendPtr 包装不生效，改串行生成。
- gradle 需 danger-full-access（native-platform.dll 加载）。
- MSVC 编译含中文的 UTF-8 源文件需 /utf-8（code page 936 错解）。

### 域/边界

- 验证分层 = **Partial**（JNI 加载 Rust dll 对比 vanilla FULL 参照，非逐位 Full）。
- Rust 块级管线不含 Beardifier 结构密度修正（@anchor.idk 已知边界）。
- 地下带差异（y<64 71-90%）与 C++ worldgen 已知边界同源，非 JNI 桥引入。
- wg_fill_density 当前返回 0（Rust 侧暂未实现完整 density 网格，fillDensity 用）——已知未实现项。

### 排除清单

- ❌ 「JNI 桥有 bug」——air 区（y=64..319）100% 吻合证明桥接数据传递正确。
- ❌ 「Rust cdylib C ABI 导出失败」——dll_test 验证 wg_* 导出 OK。
- ❌ 「JNI 桥导出失败」——jni_dll_test 验证 6 个 Java_wg_CppWorldgen_* 导出 OK。

### 记录指引

- 错误台账：.investigations/rust-mod-load/rust-mod-errors.md（M1-M4 五段式）。
- 验证记录：.investigations/rust-mod-load/cmd-output/jniprobe_rust.txt。

## 2026-08-29 Rust worldgen 整体功能实现（功能链路闭合 + 生成路径零锁）

> Rust worldgen 从「块级管线跑通（mod-run）」推进到「FEATURES 功能真正接进生成管线 + 生成路径零锁」。用户明确「先整体功能实现 + 跑测试记录对齐程度，不纠结为什么没对齐」。

### 功能链路闭合（提交映射）

| 提交 | 功能 | 错误 |
|---|---|---|
| ed59f50 | 锁清理（生成路径零锁） | 中价值 |
| 09d85e8 | OCEAN_FLOOR_WG 高度图（ocean_floor: None→Some） | F2 |
| 79daf17 | ore_vein 矿脉接入（apply 改 &self 只读） | F1 |
| a6a53f7 | Beardifier 接入（RwLock 写读分离） | F3 |
| 4ac3a00 | wg_fill_density（finalDensity 网格采样） | — |

### 功能验证（对齐快照，用户指示只记录不纠结）

| 验证 | 结果 |
|---|---|
| features_probe（完整管线） | match **95.40%** / nonAir 85.84% |
| vein_probe（矿脉） | 2295 矿脉块（1849 铜 + 19 生铜 + 427 深板岩铁） |
| fill_density_probe | 3072 点全部非零 |

### 关键设计语义（可复用）

**「并发生成路径零锁」三件套**（详见 unctional-errors.md F1-F3）：
- ① &mut 方法体实际只读 → 改 &self（签名谎报可变性 = 隐性锁来源）
- ② Option 高度图 None → 还原 Java 哨兵回退（getOceanFloorTopY 返回 min_y-1）
- ③ 「低频写 + 高频并发读」用 RwLock（读共享无争用）；持锁跨度最小化（读出来 clone 释放）

### 域/边界

- 验证分层 = Partial；对齐率 95.40% 为当前快照，用户指示只记录不展开差异。
- Beardifier 接入后探针无 beard 数据 → 对齐率不变（探针场景无结构区）。
- 错误台账：.investigations/rust-mod-load/functional-errors.md（F1-F3 五段式 + 速查表）。
- 对齐快照：.investigations/rust-mod-load/cmd-output/pipeline_alignment.txt。

## 2026-08-29 Rust worldgen 端到端性能定位（aquifer 是最大头，整体慢 Java 5 倍）

> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链闭合后进入性能定位。本小节记性能定位结论与优化方向（中价值）；错误链条（双层 Interpolated 污染 / 诊断热路径污染 / Java 基准未热）见 .investigations/perf-e2e/perf-e2e-errors.md（P1-P3）。

### 端到端对比（Java 充分预热）

- **Java 原版（WorldGenBench FULL 含树花植被，充分预热 JIT）**：稳定后 ~8-9ms/chunk（排除首个冷启动）。
- **Rust（fill_chunk_blocks 无树花，清理诊断污染后）**：44.9ms/chunk → **慢 ~5 倍**。
- ⚠️ 早期「Java 60ms」是 JIT 未热的错误基准，据此误判 Rust 达标；真实 Java 只要 8-9ms。**端到端必须对比充分预热的 Java**（AGENTS.md「端到端性能对比铁律」）。

### 无污染重定位：fill_chunk+surface base 29.4ms 内部构成（region 200,200 单线程）

| 组成部分 | 增量 | 占比/备注 |
|---|---|---|
| **aquifer（含水层 classify）** | **~17.5ms** | **60%（最大头）** |
| density（finalDensity 采样，含内部 Interpolated 网格首建） | ~12ms | 次大头 |
| carver / surface | ~14 / 4ms | carver 属完整管线阶段 |

### aquifer 内部 profile（4 chunks）

| 部分 | 耗时/chunk | 占比 |
|---|---|---|
| **calculate_density** | 19.68ms | **52%（最大头）** |
| get_block_pos（3×3 邻域） | 5.30ms | 14% |
| get_water_level_at | 0.89ms | 2% |

- **calculate_density 是 aquifer 内部最大头**：barrier.sample（1 个 3D Noise 节点，无 Cache2D 缓存）+ fluid 逻辑 + 最多 3 次调用。

### 优化方向（candidate）

1. **aquifer 的 barrier.sample 跨点缓存 / 减少 calculate_density 3 次调用 / fluid 逻辑优化**——aquifer 是 base 内最大单头（17.5ms，60%）。
2. **density**：单层 Interpolated 对 SplineDF 实测加速 70×（judge 已验证），是密度优化正解；需单层生产化验证。
3. **carver / surface**：相对小头，后置。

### 域/边界

- 验证分层 = Partial；数值为当前快照，随优化变化。端到端必须用充分预热的 Java 基准。
