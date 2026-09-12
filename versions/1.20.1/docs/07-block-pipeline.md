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

> ⚠️ **补充（260911-05，C 线 bulk section 写回）**：本条结论**不推翻**——「写路径必须维护三计数」仍成立，且旧逐块路径保留为回退（`-Dcoreswap.bulkwb=0`）。C 线把机制升级为「**整段导入 + 原地换容器 + 经 mixin `@Accessor` 显式直写三计数**」：不再逐块 `setBlockState`，但**必须复刻同一套增量语义**（`calculateCounts` 是另一套定义，见本篇新增小节 + algorithm-fingerprints #23）；`@Accessor` 是**编译期 mixin**，不是本条禁止的「运行时反射改计数」（混淆名问题不存在）。判据不变：**任何绕过 `setBlockState` 的写路径都必须显式维护三个派生计数**。

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
- **C++ JNI 桥**：
ust_jni_bridge.cpp 加载 WorldgenRust.dll（LoadLibrary + GetProcAddress），导出 6 个 Java_wg_CppWorldgen_* 函数。JNI 桥 = **薄转发层**（JNI 数组 ↔ C 指针转换 + 调 wg_*），与 C++ jni_bridge.cpp 同构。
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

**「并发生成路径零锁」三件套**（详见 
unctional-errors.md F1-F3）：
- ① &mut 方法体实际只读 → 改 &self（签名谎报可变性 = 隐性锁来源）
- ② Option 高度图 None → 还原 Java 哨兵回退（getOceanFloorTopY 返回 min_y-1）
- ③ 「低频写 + 高频并发读」用 RwLock（读共享无争用）；持锁跨度最小化（读出来 clone 释放）

### 域/边界

- 验证分层 = Partial；对齐率 95.40% 为当前快照，用户指示只记录不展开差异。
- Beardifier 接入后探针无 beard 数据 → 对齐率不变（探针场景无结构区）。
- 错误台账：.investigations/rust-mod-load/functional-errors.md（F1-F3 五段式 + 速查表）。
- 对齐快照：.investigations/rust-mod-load/cmd-output/pipeline_alignment.txt。

## 2026-08-29 Rust worldgen 端到端性能定位（大样本修正：Rust 全管线反快；aquifer 宏观仍慢需优化）

> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链闭合后进入性能定位。本小节记性能定位结论与优化方向（中价值）；错误链条（双层 Interpolated 污染 / 诊断热路径污染 / Java 基准未热与缓存假象）见 `.investigations/perf-e2e/perf-e2e-errors.md`（P1-P5）。
> ⚠️ **本小节含重大修正**：早前「Rust 慢 Java 5 倍」结论（下方【历史快照】）基于「Java 8-9ms」错误基准，被大样本推翻。

### 端到端修正对比（大样本，region 200,200，2026-08-29）

- **Java FULL（256 chunks，含树花一切，充分预热）**：≈ **55ms/chunk**（稳定 54-57ms，avg 51.7 含冷启动）。
- **Java 宏观 NOISE（256 chunks）**：≈ **23-25ms/chunk**（avg 25.4，稳定 20-27ms）。
- **Rust 全管线（400 chunks，无树花）**：**45.48ms/chunk**。
- **Rust 宏观（400 chunks，density+aquifer）**：**34.66ms/chunk**（aquifer 增量 ~21.5ms）。
- ✅ **修正结论**：**Rust 全管线 45.48ms < Java FULL 55ms → Rust 反而快 ~1.2 倍**（尽管 Rust 无树花做更少工作）。「Rust 慢 5 倍」不成立。
- ⚠️ **但宏观专项 Rust 34.66 > Java 23-25 → aquifer 慢 ~1.4-1.5 倍**（真实差距，需优化）。
- 后续阶段（carver/features）Rust 应比 Java 更省（Java FULL 的宏观 ~25 + 后续 ~30ms）。

### 域/边界

- 验证分层 = Partial；数值为当前快照，随优化变化。端到端对比必须充分预热 Java + **大样本排除缓存/冷启动**（P5 教训，AGENTS.md 铁律）。

---

### 【历史快照 · 已被大样本推翻，勿再引用为当前结论】端到端对比（早期小样本）

> 早前结论，被 P5 推翻，保留作历史排除清单：
- ❌「Java 原版稳定 8-9ms/chunk」——16 chunks 小样本 + 相邻 chunk 缓存假象，真实 55ms（6 倍低估，见 P5）。
- ❌「Rust 44.9ms 慢 Java 5 倍」——基于错误基准的错误结论，已被大样本修正推翻。
- ⚠️「Java 60ms 是 JIT 未热错误基准」这一半仍成立（P3）；但「真实 Java 只有 8-9ms」这一半错误（P5 修正为 55ms）。

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

### 无污染 aquifer 内部精确定位（diag 方法，2026-08-29）——修正上述污染态构成

> ⚠️ **修正**：下表取代上节「aquifer 内部 profile（4 chunks）」的污染态读数。早前「calculate_density 52% = barrier.sample 无 Cache2D」是污染/粗糙归因（把 barrier.sample + fluid/提前返回混记），已被计数类硬证据推翻（见 perf-e2e-errors.md P4）；该小节历史读数保留但勿再引用为当前构成。

| aquifer 内部部分 | 耗时/chunk | 占比/备注 |
|---|---|---|
| get_fluid_level（含 estimate_surface_height） | 3.84ms | 22% |
| get_block_pos（3×3 邻域 18 次/点） | 2.57ms | 14% |
| calculate_density（fluid 逻辑） | ~0ms | **barrier.sample 仅 0.1%（346/393216），走提前返回几乎不触发** |
| get_water_level_at | ~0.9ms（污染态） | 小 |
| **合计可解释** | **~6.4ms** | — |
| **剩余 ~11ms（未解释）** | = apply 每点 98304 次调用的固定开销（函数调用 + 3×3 距离计算 + 分支 + 数组访问） |

- **barrier.sample 几乎为 0（0.1%）**——「barrier 加 Cache2D 缓存」方向已被实测推翻（错误方向，见 P4）。
- **aquifer 慢的根本** = apply 每点 98304 次调用的累积成本（~11ms 固定开销）+ get_fluid_level/get_block_pos（~36%），不是 barrier 采样。

### 根本洞察：Java 宏观网格采样 vs Rust 逐点采样（~80× 差）

- **Java**：宏观用 Interpolated 网格缓存（~1225 网格交点 + 三线性插值），aquifer/density 采样次数大幅减少。
- **Rust**：逐点采样（98304 点/chunk）——采样次数比 Java 多 ~80×（1225 vs 98304），主要影响 density 段。
- ⚠️ **归因修正（P5）**：早前「~80× 采样差 = Rust 慢 5 倍的根本」基于错误的「Java 8-9ms」基准，不成立。大样本修正后 **Rust 全管线反快 Java ~1.2 倍**；真实待优化点是 **aquifer 宏观（Rust 34.66 > Java 23-25ms，慢 ~1.4 倍）**——aquifer 是逐块独立采样器（Java 同样不插值），网格采样不覆盖，方向见下方优化方向。
- 早前「外层网格采样探针」实测 2000ms 是探针实现缺陷（走 internal interpolated 雪崩重建，P1 教训），不是方向错误；正确对齐 Java 网格架构（避免雪崩）是正解方向。

### 优化方向（candidate，修正）

1. **（修正，替代原「barrier 加 Cache2D」）fill_chunk 宏观采样对齐 Java Interpolated 网格架构**（~1225 网格点 + 三线性插值，降采样次数约 80×）——直接消除 apply 每点 98304 次的固定开销。需正确实现避免跨 chunk 雪崩重建（P1 教训），且对齐 MC「本就该插值」的语义。
2. **density**：单层 Interpolated 对 SplineDF 实测加速 70×（judge 已验证），是密度优化正解；需单层生产化验证。
3. **carver / surface**：相对小头，后置。

### 域/边界

- 验证分层 = Partial；数值为当前快照，随优化变化。端到端必须用充分预热的 Java 基准。

## 07 主题篇追加小节：Rust 宏观采样层重构 + 性能定位（macro-layer-scout）

> **[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]**。
> status（各环节）：**candidate**（judge 已审计，confirmed 由人类授予）。
> 依据：`.investigations/macro-layer-scout/`{macro-layer-map.md, macro-layer-topness.md, review-audit-conclusion-chain.md} + `cmd-output/` 实测记录 + judge 审计意见（review-audit-conclusion-chain.md）。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 multichannel-errors.md（M1/M2）；一次性数值快照按低价值不展开。**

---

## 2026-08-30 Rust 宏观采样重构（multi-channel 竖切）+ 性能定位（candidate，judge 已审计）

> 背景：Rust 全量 worldgen 功能链闭合后进入性能定位。本 session 完成三件事：① 确认宏观采样真正顶层（NoiseChunk cell grid，避免挖错层）；② multi-channel 竖切重构（对齐 SteelMC/Java 单层多 channel 语义）；③ 性能定位（noise 非全管线瓶颈；宏观 aquifer 是相对 Java 真差距）。judge 审计后标 **candidate**（各环节附限定）。

### 一、宏观采样的真正顶层 = NoiseChunk::fill 的 cell grid（中价值结论）

- **顶层确认**（`macro-layer-topness.md`，reader 级可靠）：SteelMC/Java 宏观采样最终控制点 = `NoiseChunk::fill` 的 cell grid（`fill_slice_into` corners 采样 + 块级 trilerp + `combine_interpolated`）。其下 `fill_cell_corner_densities`/`combine_interpolated`/`compute_noise_column` 是机制内部函数。
- **6 个疑似上层全部确认非采样层**（blending/StaticCache2D/Beardifier/aquifer/dimension settings/ColumnCache）——见 §7「顶层排除清单」。
- **对齐意义**：Rust 重构对齐目标应以**这一层**的 cell grid 语义为准，其上均为调度/装配，无需再找「更上层采样机制」。
- ⚠️ 与 Rust 现状差异：Rust 现 `MacroGrid`（对整树采样 corners）是**错误做法** → 52× 雪崩（采样点越界→内部 InterpolatedData 懒建网格反复重建）。正确方向 = 对齐 multi-channel 竖切（把 Interpolated 当独立 channel，避免「采样整树」触发自持缓存重建）。

**status：candidate（reader 级，judge 确认可靠✅）。**

### 二、multi-channel 竖切重构（final_density → 5 channels，**探针层验证完成，生产未接线**）中价值

- **结构**（`density.rs macrolize_channels/macrolize_into`，L604-681）：DFS 收集所有 `Interpolated` 的 inner 为独立 channel；`final_density` → **5 channels**（1 BlendDensity terrain + 4 RangeChoice noodle）；combine 树 `Interpolated→ReadChannel{ch}` 全部替换；`sample_combine` 的 `ReadChannel` 分支读 `interp[ch]`。channels inner 全「纯」（无嵌套 → 可独立采样，不触发雪崩）。
- **正确性**：`DensityMacroSampler` diff0（n=54 点，x,z∈{4,8,12}，y∈{4,64,128,200,260,300}，平均差异 0.000000）。**⚠️ 局部充分非全局证明**——54 点全落在 chunk 内部 cell 边界平面（fx/fz=0），未覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y / 跨 cell 插值路径（judge 限定）。
- **⚠️ 生产接线状态（judge 澄清，必标注）**：`DensityMacroSampler` **只在探针文件** `macro_sampler_probe.rs` 定义；生产 `terrain.rs fill_chunk` **未接线**（默认仍逐点 `dense.sample()`，`MacroGrid` 仅 `WG_MACROGRID` env 时启用）。**即「multi-channel 竖切重构 = 探针层验证完成，生产 fill 路径未接入优化。**
- **性能（标量，探针层）**：slices 构建 8.52ms + trilerp 0.3ms = 8.83ms vs 逐点 6.43ms（**标量结构正确但不省**）。`std::simd` (portable_simd) Rust 1.98 stable 不可用（需 nightly/intrinsics）；块级 trilerp SIMD 收益小（0.3ms）。
- **关键洞察**：Java 宏观高效的关键 = **ColumnCache（5×5 grid 缓存 xz 噪声值 O(1)）+ 批量 corners 采样（fill_cell_corner_densities_4x SIMD）**，不只是 SIMD。

**status：candidate（正确性在所测子空间可信；附「探针层未接线」限定）。生产接线为下一步待办。**

### 三、性能定位链（candidate，judge 修正归因）

- **corners 采样构成**（`corner_sampling_breakdown`）：ch#0（BlendDensity terrain，3677 节点）3.60ms/chunk（1225 corners）绝对大头；ch#1-4（RangeChoice noodle 小）各 ~0.1ms（合计 0.4ms）；所有 channels corners 总计 3.61ms/chunk（预热后，含首次缓存构建 ~8.5ms）。
- **tree_vs_noise（修正里程碑）**：ch#0 完整 corners 3.34ms → 去 noise 0.38ms → **noise 采样贡献 2.97ms（89%）**；树遍历仅 0.38ms（11%）。真正大头 = noise 采样（3D Perlin：Noise/ShiftedNoise/InterpolatedNoise/WeirdScaled），**非树解释器**（DFC 编译只优化 11% 树遍历，收益有限）。⚠️ 该修正与早前 `milestone_record.txt`（树解释器大头判断）相反，以 tree_vs_noise 为准。
- **noise AVX 评估（judge 修正归因）**：`sample_section_avx` 是**死代码**（从未被调用，生产走标量 `sample_section`）**且非真 SIMD**（函数体是标量 dot3/lerp/perlin_fade）。「Perlin 26.56→19.55ns（1.36x）」是 `bench_noise.rs`（标量 `sample()`）在 `-C target-feature=+avx` 下**编译器自动向量化**的微基准，**非手工 AVX 路径功劳**。「features_probe 95.40% 不变」是平凡事实（AVX 没接线）。**全管线 -1% 方向可靠**（45.47→45.01ms，400 chunks），说明 noise 非全管线瓶颈。→ **噪声 AVX 归因修正见 §7 M2。**
- **全管线差距定位（judge 限定）**：Rust 全管线 45.48ms < Java FULL 55ms → **Rust 反快 ~1.2×**；**但宏观专项 Rust 34.66ms > Java 23-25ms → 宏观 aquifer 慢 ~1.4-1.5×（相对 Java 真差距，唯一明确待优化点）**。⚠️ 「21.5ms」是**宏观子集减法 Partial 快照**（34.66 macro − 13.14 density），非 aquifer 独立计时；且 aquifer 内部曾多次翻案（barrier 大头已被计数证据推翻 P4）。→ 表述限定为「**宏观 aquifer 是相对 Java 的待优化差距**」，非「全管线绝对瓶颈」（Rust 全管线已快于 Java）。

**status：candidate（judge 确认「corners noise 89%」与「全管线 aquifer 大头」两测量域不矛盾，但需在 docs 显式区分「宏观 corners 采样成本」vs「全管线阶段构成」）。**

### 四、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ **ch#0 corners 采样大头 = 树解释器/树遍历**——被 `tree_vs_noise` 推翻：noise 采样 89%，树遍历仅 11%。
- ❌ **noise AVX 手工实现带来 1.36x**——`sample_section_avx` 死代码/非 SIMD，1.36x 是编译器 auto-vec。
- ❌ **Rust 慢 Java 5 倍 / Java 8-9ms**——大样本修正后 Rust 反快 ~1.2×（已在 07 篇既有小节记录，不重复）。
- ❌ **`MacroGrid` 对整树采样 corners 是正解**——52× 雪崩（越界→自持缓存重建），正确方向是 multi-channel 竖切。
- ❌ **barrier.sample 是 aquifer 大头 / 加 barrier Cache2D**——已被 P4 计数证据推翻（07 篇既有小节已记）。
- ⚠️ **Shift mode 也 y 无关（可缓存）**——见 §7 M1，plain `minecraft:shift` 未验证 y 独立，cache 对 Shift 曾强置 y=0 偏离参考（已保守改不缓存）。

### 五、优化方向（candidate）

1. **（宏观 aquifer，相对 Java 真差距）** 优化 apply 每点 98304 次固定开销（~11ms 未解释）+ get_fluid_level/get_block_pos（~36%）；**优化前用无污染计数探针（aquifer_*_count）锁定 apply 固定开销真实构成**（防再翻案；勿直接上 barrier Cache2D，P4 已推翻）。
2. **（multi-channel 生产接线）** 把探针层验证完成的 multi-channel 竖切接入生产 `fill_chunk`（消除「采样整树」→ 雪崩）；补 ColumnCache + 批量 corners 采样（对齐 Java 宏观高效关键）。
3. **（noise）** 非当前瓶颈，后置；若之后 density 成瓶颈再考虑（真正的 AVX __m256d dot 需真实现 + 接线热路径，当前框架未做）。

### 六、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；数值为当前快照，随优化变化。
- status：**candidate**（confirmed 由人类授予）；生产接线未完成，正确性证明局部（54 点）。
- ⚠️ **后续已接线生产**：transpiler 现已接入生产（TranspilerDensity + WG_TRANSPILER 门控），见本文末尾「2026-08-30 transpiler 4 公式修复 + 生产接线」M12 节（此行为历史快照，保留不删）。

### 七、顶层排除清单 + 错误台账

- **顶层排除清单**（6 个疑似上层均非采样层，reader 确认）：① blending（树内部：BlendAlpha→常量 1.0/BlendOffset→常量 0.0，BlendedNoise 为 corners 叶采样，非外层包装）② StaticCache2D/ChunkHolder（调度层，仅 Beardifier 结构引用解析）③ Beardifier（块级、combine 之后加，非 corners 采样）④ aquifer（宏观采样之后逐 block 消费层，独立液面 cell grid，不重采样地形 density）⑤ dimension settings（cell 尺寸参数，非独立层）⑥ ColumnCache（采样机制内部性能缓存）。
- **错误台账独立成篇**：`.investigations/macro-layer-scout/multichannel-errors.md`（M1 ShiftDF 缓存 y=0 潜在 bug；M2 noise AVX 归因不实，五段式 + 速查表）。

---

## 2026-08-30 build-time transpiler 探索（shift 引用 bug 修复 + 对齐验证）（candidate，judge 已审计）

> **[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]**。
> status（各环节）：**candidate**（judge 已审计，confirmed 由人类授予）。
> 依据：`.investigations/macro-layer-scout/`{buildtime-compile-map.md, review-transpiler-perf.md, transpiler-errors.md} + `cmd-output/`{transpiler_continents_after_shiftfix.txt, transpiler_finaldensity_after_shiftfix.txt, transpiler_alignment_status.txt, tree_vs_noise.txt, transpiler_grid_compare.txt, transpiler_grid_calls.txt} + judge 审计意见（review-transpiler-perf.md）。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 transpiler-errors.md（M1-M9，独立成篇，不重复展开）；一次性数值快照按低价值不展开。**

### 一、背景（build-time transpiler 探索）

- **目标**：把 density 树（3677 节点）编译成 native 代码，避免运行时 enum match 递归解释（`buildtime-compile-map.md`：SteelMC 式 specialized 内联函数生成器，非数据驱动解释器）。
- **本 session 动作**：judge 审计（`review-transpiler-perf.md`）发现 transpiler 有重大缺陷（shift 引用 bug + continents 对齐被污染 + 价值判断被削弱），本 session 修复 shift 引用 bug 并重测对齐。

### 二、修复：shift 引用 bug（M7，中价值判据）

- **bug 机制**：`minecraft:shift_x`/`minecraft:shift_z` 是 **vanilla 内建 density 函数**（不在 `density_function/overworld` 子目录），transpiler 的 registry 只从 overworld 目录收集（`build/density.rs` `collect_json`）无法 resolve，被静默替换为 `0.0 /* unresolved ref */`（生成代码 55 处）。运行时 `density_builder.rs` L176-189 有正确特殊处理（`shift_df(ShiftMode::ShiftA/ShiftB)` 采样 `minecraft:offset` noise），transpiler 缺此处理 → 同一内建函数两种语义。
- **修复**：`build/density.rs` L104-108 特殊处理 `minecraft:shift_x`/`shift_z`，对齐运行时 ShiftA/ShiftB：shift_x → `noises.sample_noise("minecraft:offset", x*0.25, 0.0, z*0.25)*4.0`，shift_z → `noises.sample_noise("minecraft:offset", z*0.25, x*0.25, 0.0)*4.0`。修后 `grep "unresolved ref"` 生成代码 = 0（原 55）。
- **可复用判据**：**「unresolved: 0」计数是假信号**——置零也算 0（`0.0 /* unresolved ref */` 被当成已处理，但语义错）。计数必须数「含 unresolved 注释的占位符」。**「从数据目录收集」≠「覆盖所有函数」**——内建函数（shift_x/shift_z/shift_a/shift_b/shift）必须单独特殊处理；**transpiler 与运行时必须逐函数核对内建函数处理**，不能只对齐「数据目录里的函数」。

### 三、对齐验证（continents 0.000000 / final_density 0.43）

- **continents 对齐 0.0088 → 0.000000**（n=54 全对齐，`transpiler_continents_after_shiftfix.txt`）：**证明 transpiler 核心（noise/spline）正确**。原 0.0088 被 shift 引用 bug 污染（transpiler 置零 shift → 未偏移 vs 已偏移差异），非干净核心正确性测试。
- **final_density 对齐 0.44 → 0.432843**（n=54，`transpiler_finaldensity_after_shiftfix.txt`）：修 shift bug 后无显著变化。**剩余差异来自「channel inner 采样」**——transpiler 竖切（`fill_cell_corner_densities` 每点采样完整树）vs 运行时 Interpolated cell grid 插值语义差异，**非 shift bug**。
- **可复用判据**：**对齐测试要选「不含已知 bug 引用」的干净函数**——被测函数若含 transpiler 置零的 shift 引用，对齐差异是「bug 误差」不是「核心正确性」。**小对齐值 ≠ 核心正确**（0.0088 小只说明「shift 置零误差在当前测试点小」）；**污染测试比无测试更危险**（被污染的「通过」测试会让人误信错误实现）。

### 四、价值评估：transpiler 价值被「noise 89% 主导」削弱（M9，中价值判据）

- **ch#0 corners 采样构成**（`tree_vs_noise.txt`）：noise 采样占 **89%**（2.97ms/3.34ms），树遍历仅 **11%**（0.38ms/3.34ms）。
- **transpiler 优化的是树遍历（11%），不是 noise 采样（89%）**——即使完美 transpiler（含缓存）也只能省 ~11% 的 ch#0 成本。**build-time 编译（消除 enum match）的收益上限 = 被编译部分占比（树遍历 11%）**。
- **可复用判据**：**性能优化要盯「真正的瓶颈」（noise 89%），不是「顺手的」（树遍历 11%）**——先做占比分解（去 noise 后测剩余）再决定优化哪个；**「编译成 specialized 函数」的收益上限 = 被编译部分占比**。

### 五、性能定位：无缓存抵消编译优势（M1-M6，中价值判据）

- **transpiler cell grid 构建 41.79ms vs 运行时 Interpolated grid 构建 8.14ms（慢 5 倍）**（`transpiler_grid_compare.txt`/`transpiler_grid_calls.txt`）：两者都是 1225 corners（采样量相同），差异在**单次 fill 成本**——transpiler 每 corner 采样 5 channels 完整树**无缓存（缓存冷 34μs）**，运行时 Interpolated **有缓存复用**（grid 建一次 + 块内插值）。
- **核心判断**：**编译优化与缓存优化是正交的两条线，缺一不可**——编译成 specialized 函数 ≠ 快，若每点采样完整树（无缓存复用），可能比运行时（有缓存）更慢。
- **可复用判据**：**性能定位先排除「采样量」再查「单次成本」**（1225=1225 一致 → 差异必在单次 fill 缓存状态）；**区分「缓存热 vs 缓存冷」**（同一函数不同调用场景单次成本可差 5 倍，用单次成本估算总成本前先确认测量场景缓存状态一致）；**「看起来慢」≠「是瓶颈」**（noise 查表数组优化后 1.02x 无差异 → 排除，用「优化后是否有显著收益」的对照实验判断瓶颈）。

### 六、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ **transpiler 完整实现 / references 全 resolve（unresolved: 0）**——假，实际 55 个未 resolve shift 引用（M7，已修复）。
- ❌ **continents 0.0088 证明 transpiler 核心正确**——被 shift 引用 bug 污染，非干净测试（M8，修后 0.000000 才证明核心正确）。
- ❌ **final_density 0.44 全部来自 channel inner 采样**——归因不完整，0.44 含 shift bug 贡献（M7，修后 0.432843 才分离出纯 channel inner 差异）。
- ❌ **noise 查表（HashMap）是性能主因**——数组优化后 1.02x 无差异，主因是 cell grid 无缓存（M4）。
- ❌ **「深入缓存」是下一步正解**——缓存建立在有 shift bug 的代码上，先修 bug + 重新评估价值（M7/M9，judge ⑥）。

### 七、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；数值为当前快照，随优化变化。
- status：**candidate**（confirmed 由人类授予）；transpiler 加缓存方向未落地（仅记录方向，不标 confirmed）；生产接线未完成。
- ⚠️ **后续已接线生产 + 加缓存已落地**：transpiler 已补缓存（对齐 ColumnCache）并接入生产（M11 + M12 节），见本文末尾 M12 节（此行为历史快照，保留不删）。
- 错误链条完整记录见 `.investigations/macro-layer-scout/transpiler-errors.md`（M1-M9，五段式 + 速查表），本节不重复展开。

---

## 2026-08-30 transpiler 补缓存 + 探针污染修复（candidate，judge 已审计）

> **[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]**。
> status（各环节）：**candidate**（judge 已审计，confirmed 由人类授予）。
> 依据：`.investigations/macro-layer-scout/`{buildtime-compile-map.md, transpiler-errors.md} + `cmd-output/`{transpiler_grid_after_cache.txt, transpiler_fill_after_cache.txt, transpiler_fill_cold_after_cache.txt} + 代码锚点（`WorldgenRust/build/density.rs` L226-234、`WorldgenRust/src/density.rs` L334-335）。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 transpiler-errors.md（M10/M11，独立成篇，不重复展开）；一次性数值快照按低价值不展开。**

### 一、背景（承接 build-time transpiler 探索）

- 承接既有「2026-08-30 build-time transpiler 探索」小节：transpiler 把 density 树编译成 native 代码，但性能未达（cell grid 构建慢 5 倍，M1-M6 定位为「无缓存」）。本 session 落地两件事：① 补缓存（对齐 Java/SteelMC ColumnCache，M11）；② 修复性能探针污染（M10，声称测缓存冷实为缓存热，低估 20 倍）。

### 二、补缓存（对齐 Java/SteelMC ColumnCache，M11，中价值判据）

- **根因（MVP 简化，非架构决策）**：transpiler 是「先验证链路」的临时简化版，把 `flat_cache`/`cache_2d`/`cache_once`/`cache_all_in_cell` 直接内联 inner（`build/density.rs` L224-228「MVP 不做缓存，语义等价」），每 corner 重算完整树。查提交历史确认：初始版本（1838a22）缓存节点未处理，后续（e4a56c8）延续「不做缓存」——是实现者自己定的临时简化，**非用户拍板架构决策**。
- **修复**：
  - `build/density.rs` L226-234 对缓存节点生成 `transpiler_cache_2d(id, x, z, || inner)` 调用（运行时 thread_local 缓存，key 按 (x,z)，y 无关，对齐运行时 `Cache2DData` 语义）。
  - 运行时 `density.rs` L334-335 加 `transpiler_cache_2d` 函数（先查缓存，未命中 **drop 借用后重算**，避免嵌套借用 RefCell panic——闭包可能嵌套调用同一 C2D_CACHE RefCell）。
- **性能复测**：cell grid 构建 **443ms → 17ms/chunk**（慢 54 倍 → 慢 2 倍）；fill 单次（缓存热）**438μs → 13μs**。
- **对齐验证**：continents **0.000000**（不变，缓存语义正确）、final_density **0.43**（基本不变）。
- **可复用判据**：**「MVP 简化」要记录决策来源**——临时简化（先验证链路）若未记录，后续会误以为是架构决策，导致「照抄 Java 却丢了缓存」的困惑；**缓存是性能关键**——补缓存后 cell grid 构建 443ms → 17ms（25 倍），证明「无缓存每 corner 重算完整树」是性能主因（M1-M6 已定位，M11 落地修复）。

### 三、探针污染修复（M10，中价值判据）

- **污染**：`transpiler_fill_noise_share.rs` 声称测「缓存冷」，但坐标 `px = -288*16 + (i % 16) * 4`、`pz = -256*16 + (i % 16) * 4` 用**同一个 `(i % 16)`**，导致 (px, pz) 只有 **16 种组合**（不是 16×16=256 种）——缓存（按 (x,z) key）命中这 16 种组合 → 探针实际测的是**缓存热**，低估 **20 倍**（13μs vs 真实缓存冷 262μs）。
- **定位**：对比 `transpiler_fill_noise_share`（13μs）与 `transpiler_fill_cost`（缓存热 12.3μs）几乎相同 → 触发怀疑「声称缓存冷，实际缓存热」；写新探针 `transpiler_fill_cold`（每 corner 不同 (x,z)）→ 262μs，确认探针污染。
- **修复**：探针坐标改为每 corner 不同 (x,z)（px/pz 用不同取模），修后测出真正缓存冷 **263μs**（与独立探针 `transpiler_fill_cold` 一致）。
- **可复用判据**：**探针坐标设计要避免循环命中缓存**——测「缓存冷」时坐标必须每点不同（px/pz 用不同取模），否则缓存命中，声称缓存冷实为缓存热；**对比探针测量值交叉验证**——声称测缓存冷的探针若与缓存热探针几乎相同，则探针污染（坐标循环命中）。

### 四、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ **transpiler 无缓存是用户拍板的架构决策**——查提交历史（1838a22 初始未处理缓存节点、e4a56c8 延续「不做缓存」）确认是 MVP 临时简化，非架构决策（M11）。
- ❌ **`transpiler_fill_noise_share` 测的是缓存冷**——坐标 px/pz 用同一 `(i % 16)` 循环命中缓存，实为缓存热，低估 20 倍（M10，修后 263μs 才测出真缓存冷）。

### 五、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；数值为当前快照，随优化变化。
- status：**candidate**（confirmed 由人类授予）；补缓存已落地（cell grid 构建 443ms → 17ms）；**生产接线已完成**（TranspilerDensity + WG_TRANSPILER 门控，见下节「2026-08-30 transpiler 4 公式修复 + 生产接线」）；transpiler 整体价值仍受「noise 89% 主导」削弱（M9，见既有小节）。
- 错误链条完整记录见 `.investigations/macro-layer-scout/transpiler-errors.md`（M10/M11，五段式 + 速查表），本节不重复展开。

---

## 2026-08-30 transpiler 4 公式修复 + 生产接线（M12，candidate，judge 已审计）

> **已应用（commit 649a2b2）+ 已审计**：本节为结论性记录（candidate，confirmed 由人类授予），证据落盘见 `.investigations/macro-layer-scout/` 及 `cmd-output/`。
> status（各环节）：**candidate**（judge 已审计 + 补证后复审，confirmed 由人类授予）。
> 依据：`.investigations/macro-layer-scout/`{transpiler-errors.md M12, review-transpiler-prod.md} + `cmd-output/`{transpiler_finaldensity_after_unaryfix.txt, transpiler_prod_density_98304.txt, transpiler_slices_ch0_after_bnfix.txt, transpiler_prodblocks_after_unaryfix.txt, transpiler_prod_vanilla_full.txt, macrosampler_prod_vanilla_full_baseline.txt, transpiler_prod_perf_multi.txt}。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 transpiler-errors.md（M12，独立成篇，不重复展开）；一次性数值快照按低价值不展开。**

### 一、M12：4 个数学公式生成错误（final_density 0.432843 的真正根因）

- **背景**：transpiler 接入生产时发现 final_density 对齐 0.432843（judge 曾误判为「channel inner 采样语义差异」并接受）。逐 channel 对比（`transpiler_prod_combine6`）发现 ch1-4（noodle）diff=0.000000 完全对齐，唯独 ch0（terrain）差 3.14 → 分解出 4 个公式生成错误：
  - **squeeze**：生成成 `v - v³/3`，正确是 `clamp(v,-1,1)/2 - d³/24`（对齐运行时 `apply_unary`）。
  - **half_negative**：生成成 `-0.5*x`，正确是 `if x > 0 { x } else { 0.5*x }`。
  - **quarter_negative**：生成成 `-0.25*x`，正确是 `if x > 0 { x } else { 0.25*x }`。
  - **weird_scaled_sampler**：把 rarity 分段阶梯（`scale_value`：0.75/1.0/1.5/2.0/3.0）错生成成 `d*2.0`/`d*3.0` 常数乘。
- **修复**：`build/density.rs` 四处生成公式对齐运行时 `apply_unary`/`WeirdScaled::scale_value` 语义。
- **结果**：final_density 对齐 **0.432843 → 0.000000**（n=54，`{:.6}` 舍入下 max_diff <5e-7，非 bit 级 0）。
- **可复用判据**：**「对齐值中等偏小（0.4）≠ 语义差异固有」**——对齐未达 0 时不要给差异找架构性借口，逐 channel/逐步骤分解直到差值消失；**公式类 bug 用「手算对照」定位最快**（把 interp 值代入错式与对式手算，与实测输出对上即锁定）；**transpiler 每个节点类型的生成公式必须逐一对齐运行时对应实现**，不能凭记忆写数学式。

### 二、接入生产（TranspilerDensity + WG_TRANSPILER 门控）

- **实现**：`terrain.rs` 泛化 `ChunkDensity`/`DensitySource`/`fill_chunk`（新增 `ChunkDensitySampler` trait）+ `TranspilerDensity`（transpiler 生成代码采样 cell grid + 块级插值 + compute）；`worldgen_handle.rs` `WG_TRANSPILER` env 门控构建 NoiseSet+TranspilerDensity，`fill_chunk_blocks` 按此分派。
- **零风险切换**：env 未设时 `transpiler_density = None` → 走 `DensityMacroSampler`，行为与改动前完全一致。
- **验证**：TranspilerDensity vs DensityMacroSampler 全 chunk 98304 点 **max_diff=0.000000**（`{:.6}` 舍入下 <5e-7 浮点残差内，探针 `transpiler_prod_density.rs` L51-63 逐点遍历；`transpiler_prod_density_98304.txt`）；块级一致 **99.30%**（非 100%，~0.7% 块因密度近 0 边界浮点残差翻转分类，修前 78.48%，`transpiler_prodblocks_after_unaryfix.txt`）；vs vanilla FULL **94.20%**（基线 DensityMacroSampler 95.40%，transpiler 较基线**略低 1.2pp**，nonAir 83.06% vs 86.89% 低 3.83pp，经 carver/features 级联放大；两路径同为 partial match，基线亦未达 100%，`transpiler_prod_vanilla_full.txt` + `macrosampler_prod_vanilla_full_baseline.txt`）。差异来源待归因（cache_all_in_cell 点级缓存效率低于 Java cell 级为文档化假设——既有的正确性保守事实，非已证根因；channel 采样语义细微差异亦属可能方向）。
- **性能**：5 次 release 运行 **0.96-1.05x，均值 ~1.00x = 与基线持平**（`transpiler_prod_perf_multi.txt`）。cell grid 构建慢（47ms vs 8.14ms）不是端到端瓶颈（density 阶段占完整管线比例小；judge M9 已证 noise 89% 才是真瓶颈）。
- **价值定位（最终）**：transpiler 的价值在**正确性对齐**（final_density 与基线 DensityMacroSampler 对齐至浮点残差 <5e-7、块级一致 99.30%，worldgen 上层可查由头），性能与基线持平即可。

### 三、探针教训（2026-08-30 新立）

- **排障先核对探针自身前置条件**：`transpiler_slices_ch0` 探针漏 `set_blended_noise`（`noise.rs` L315，`NoiseSet` 方法）→ ch0 内 `sample_blended_noise` 返回 0 → 误判为「cache id 污染」（134/1225 diff）。补 `set_blended_noise` 后 max_diff=0.000000。**排查「两实现不一致」时，第一步核对两边初始化等价（NoiseSet 的 blended_noise/seed 派生/注册表），第二步才是怀疑缓存/污染/公式**。
- **cache id 空间隔离（防御性）**：transpiler cache id 起始改为 1_000_000（与运行时 `NEXT_CACHE_ID` 从 0 递增隔离，两者共用 `C2D_CACHE` 数组）——防御性修复（本次矛盾另有根因，但隔离正确）。

### 四、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ **final_density 0.43 来自「channel inner 采样语义差异」**——实际是 4 个公式生成错误叠加（M12，修后 0.000000）。
- ❌ **transpiler 与运行时 cache id 空间冲突导致 134/1225 diff**——实为探针漏 `set_blended_noise`（补后 0.000000）；cache id 隔离是防御性修复，非本次矛盾根因。

### 五、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；数值为当前快照，随优化变化。
- status：**candidate**（confirmed 由人类授予）；`cache_all_in_cell` 用点级 (x,y,z) 缓存（正确性保守，非 bug，但缓存效率低于 Java cell 级）；n=54 覆盖局限（cell 边界平面，未覆盖 cell 内部/边界 clamp/负 Y）。
- 错误链条完整记录见 `.investigations/macro-layer-scout/transpiler-errors.md`（M12，五段式 + 速查表），本节不重复展开。

---

## 2026-08-30 扩大对齐样本（judge 建议项 7）+ transpiler flat_cache 量化语义修复（M13，confirmed）

> **已应用（含修后回归全量落盘）+ confirmed 由用户授予（2026-08-30，MOD 实跑进游戏肉眼无差异）**；A 语义判定（`analysis-flatcache-semantics.md`）随本节一并 confirmed。
> status：**confirmed**（用户 2026-08-30 授予；judge 审查 review-m13-flatcache-jni.md 3×PASS 在前）。
> 依据：`.investigations/macro-layer-scout/`{analysis-flatcache-semantics.md, transpiler-errors.md#M13} + `cmd-output/`{transpiler_exactpoint_verify.txt, transpiler_ch0_decompose.txt, transpiler_ch0_census.txt, transpiler_exactpoint_verify_after_flatcachefix.txt, transpiler_ch0_decompose_after_flatcachefix.txt, transpiler_alignment_expanded_after_flatcachefix.txt, transpiler_prod_density_98304_after_flatcachefix.txt, transpiler_prodblocks_after_flatcachefix.txt, transpiler_prod_vanilla_full_after_flatcachefix.txt}。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 transpiler-errors.md（M13，五段式 + 速查表），不重复展开；一次性数值快照按低价值不展开。**

### 一、主线：扩大对齐样本（judge 建议项 7）→ flat_cache 量化语义 bug 发现与修复（4 探针链）

- **发现**：按 judge 建议项 7（覆盖 cell 内部任意点/chunk 边界 clamp/负 Y）扩大对齐样本，`transpiler_exactpoint_verify`（对比1=channel inner 精确采样+combine 精确点、对比2=vs tree.sample 插值，n=3584）暴露：**对比1 max_diff=0.027855**（diff>1e-9=250/3584，诊断分组自 y=64 起）、ch0 内部点 (3,3) t=0.133102 vs r=0.068001（diff 0.065101），而 4 corner 两侧逐位一致、生产 98304 点恒 0.000000——差异只藏在「精确点诊断采样」域。
- **4 探针链**：① `transpiler_exactpoint_verify`（扩样对比1/对比2）→ ② `transpiler_ch0_decompose`（分解实验：量化检查三点 / corner 双线性自洽性 / y 密扫 / 跨 x 扫描）→ ③ `transpiler_ch0_census`（节点普查：ch0 Interpolated 残留=0、FlatCache=363，疑点收敛到 flat_cache 节点簇）→ ④ `transpiler_alignment_expanded`（修后扩样回归：精确点核心对比 + 生产路径对比 interior/边界/负 Y）。fan-out 双 worker（A/B）超时失败后改走「分解实验+census 普查采数据 → core-worker 判读」收敛路径，判定报告 `analysis-flatcache-semantics.md`（A 语义裁决）。
- **判定（A 成立）**：vanilla `minecraft:flat_cache` = **4×4 格点量化缓存**（`ChunkNoiseSampler.java` L836-881 FlatCache：per-chunk **5×5 网格 y=0 预计算** + `(blockX>>2)` **量化索引**，cell 内匿名点共享左下角格点值，越界直算）；同文件 **Cache2D（L557-579）才是精确列键**。**transpiler 把 flat_cache 与 cache_2d 合并生成精确键 `transpiler_cache_2d` 是语义 bug**（「y 无关」被偷换成「逐点精确」）；CoreSwap 运行时（FlatCacheData）与 C++ FlatCacheDF 为 Java 语义忠实复刻，**本就正确**。
- **修复**：`build/density.rs` 拆分两分支——`flat_cache` 生成量化封装（x/z 用 i64 算术右移量化到 4 的倍数、y 变量遮蔽置 0、按量化键缓存），`cache_2d` 保持精确键；运行时/C++ 未动。生成侧注释更新记录两类节点语义差异。
- **修后全量回归**（`cmd-output/*_after_flatcachefix.txt`）：内部点 ch0 0.068001 vs 0.068001 **diff=0.000000**；量化签名复刻（transpiler(1,1)=0.062840 与 runtime 同值）+ y 密扫/跨 x 扫描全 0.000000；**对比1 max_diff=0.000000**（n=3584，残余 >1e-9 的 160/3584 点均 <5e-7 浮点残差，`{:.6}` 舍入口径非 bit 级 0）；vs tree.sample 0.043514 维持（既有插值语义差异，对比1=0 证实非 bug）；生产路径对比 n=1480 max_diff=0.000000（119/1480 <5e-7）；生产回归 98304 点 **0.000000**（均值/正值分布与修前逐值相同）+ 块级 **99.30% 持平**（1561802/1572864，与修前逐值相同）+ vs vanilla FULL **94.20% → 94.27%**（+0.07pp，1482808/1572864）。

### 二、中价值结论（可复用）

1. **flat_cache vs cache_2d 语义区分（vanilla 同一文件就有两类缓存语义）**：`cache_2d` = 透明缓存（block 级精确 (x,z) 列键，本点计算本点取）；`flat_cache` = **量化采样器**（5×5 网格 y=0 预计算、`(x>>2)` 量化索引、cell 内喂 cell 左下角格点值、越界直算）。判据：**拿到 cache 类节点先问「命中时返回的值第一次在哪算的」**——cache_2d 在本点算（透明），flat_cache 在 cell 角、y=0 算（不透明，改变返回值）。「y 无关」只支撑「缓存 key 不含 y」，不支撑「任意 (x,z) 取本点值」。
2. **诊断方法：corner 双线性自洽性检验**——检验 cell 内部点相对自身 4 corner 双线性插值的偏离：全精确实现应自洽（transpiler 偏离 −0.004，树本征非线性残差），被量化实现不自洽（runtime 偏离 −0.069）——把「谁在量化」归属到侧后，用 Java 反编译源裁决忠实侧。廉价、单假设、无歧义。
3. **量化签名（quantization footprint）**：corner 处 diff=0（量化索引恰命中槽位、值=精确值）+ 内部点系统性偏离双线性 + 差异随 y 线性轮廓（量化值经树内 y 相关线性结构放大）——三特征同现即「量化缓存语义差」；修后 transpiler 复刻同一签名（三点同值）即修复到位的互证。
4. **corner-only 生产域会掩盖缓存语义 bug**：corner 处量化值≡精确值（y 无关树），生产 98304 点恒 0——语义 bug 只暴露在扩样后的内部点。**诊断（精确点）域与生产（corner-only）域互补**。生产不受影响的两个前置条件：① flat_cache inner 保持 y 无关（climate 树）；② 生产采样保持 corner-only——本修复消除了对这两个前置的长期依赖。

### 三、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ 「Java 精确键、CoreSwap 运行时自行量化」——与 ChunkNoiseSampler.java L858-864 直接矛盾 + 与 C++/Rust 双侧独立复刻史矛盾（判定报告 §二）。
- ❌ ShiftDF 是第二处偏差源——运行时 ShiftDF 核实为精确 (x,z) 缓存（字段名 cx/cz 误导，实存精确坐标），双边一致非混淆源。
- ❌ 「runtime 偏离角双线性 = runtime 有 bug」——正是 flat_cache 量化 + 树内其它精确项合成的预期签名。
- ❌ Interpolated 节点未编译残留——census 残留=0。

### 四、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；判定报告 = Degraded（静态审查）+ 运行时探针解读；数值为当前快照。
- **已知边界**：Java FlatCache 是 per-chunk 实例（查表区间 = 采样器自身 chunk，越界相对自身 startBiome 则 delegate 直算）；无状态 transpiler 按 pos 推导量化角，跨 chunk 直采诊断点与 Java per-chunk 上下文可能仍有差（生产 in-chunk 恒界内不受影响，诊断跨 chunk 抽查时注意）。对比2 vs tree.sample（InterpolatedDF 插值）0.043514 为既有「运行时插值 vs transpiler 直采」采样语义差异，非 bug 残留。
- status：**confirmed**（用户 2026-08-30 MOD 实跑授予）。
- 错误链条完整记录见 `.investigations/macro-layer-scout/transpiler-errors.md`（M13，五段式 + 速查表），本节不重复展开。




## 2026-09-03 est 优化收口（追加小节草稿，260903-12）：shared 臂裁决 candidate + 翻默认前置达成 + gpu-batch-merge 降级建议

> 承接本篇「Q-PD1/Q-AQ1」归因与 260903-11 est 查表化优化包（est_at 共享 + 跨 chunk est L2，commit 0949402，门控默认关）。本轮 Full 层运行时证据裁决 shared 臂语义 + 清偿翻默认前置。产物：`.artifacts/lossless-accel/{est-shared-verdict,est-l2-defaultflip-p2,gpu-merge-revisit}-260903-12.md`（均 candidate）；judge 两轮（review-est-shared / review-p2-p3-final，建议 confirmed 前清偿文档级修正）。

### ✅ shared 臂裁决结论（candidate）：shared = 修正，off = 系统性偏离

- **验证分层 Full**：Java `ChunkNoiseSampler.estimateSurfaceHeight` mixin RETURN dump vs Rust `WG_EST_DUMP` 角值 dump，同 seed 8576294172403134396 同 region (200,200) 64 chunks（§9.7 三要素见 verdict 头部）。
- **结论**：共同列（c0 原点角，Java dump 列 residue 无 residue-12，其余 3 角列无对应列——覆盖面精确表述 per judge A2）**shared 64/64 与 Java 逐值一致；off 0/64 全偏且 delta 恒 −1**；角列敏感性 63/64（唯一敏感 chunk (201,200)：java@+16=56 / shared@+12=48 / off=55）。Java 表 11877 条 conflicts=0（est 为量化列纯函数）。
- **⚠️ 待办（翻 shared 默认的前置）**：Rust 两臂 heights4 参数 `cx*16+15`（量化 +12）≠ Java SURFACE 四角 `(i+1)<<4`=+16——改 +16（两臂独立小包）后完全对齐；量化敏感 chunk 约 1.6%~4.7%。
- **⚠️ 附带生产 bug 线索（judge A1，另立验证）**：off 臂扫描 `(min_y..min_y+noise_height).rev().step_by(8)` 半开区间 rev 首采样点 = 319 vs Java 320——**off 是当前默认臂，−1 系统偏移独立于翻默认决策**。

### ✅ 翻默认前置达成情况（est-l2-defaultflip-p2，candidate）

四项门控（260903-11 judge 预置）逐项清偿：

| 前置 | 结果 |
|---|---|
| 默认路径零回归 | ✅ off 臂 hash == HEAD 基线（74f5dfc4，stash 重建复跑） |
| Mutex 争用 | ✅ 无退化：L2 加速比 2.55→3.12× 随线程不降反升（T=1/2/4/8 交错双跑，偏差 <3%） |
| 大 region 淘汰 | ✅ 64×64 sweep 命中 92±1%、evictions=0；触顶投影 ~7600+ chunk（inserts ~40k，judge B 修正），typical region 远未触顶 |
| e2e l2 stats 落盘 | ✅ T=1 l2 hits/misses/inserts 逐条可溯源（89.8%） |

- 剩余差归因（P2.4）：est_price_probe 同代码 hot ~60ns/iter vs cold 5.7µs/iter（形态差 ~95×，workflow-patterns #21 量化实锤）；跨 session 生产隐含单价 8.5/9.9µs 稳定 → 剩余 ~1.5×（生产侧 aquifer/缓存压力）为 Partial 解释，已声明。
- ⚠️ sweep 在 ~2304-2560 chunk 处 panic（`surface_rules.rs:505 missing noise sampler`）——数据截止于此，panic 另立课题，不影响 typical region 结论。

### 📉 gpu-batch-merge 降级为低优先级保留（用户拍板 260903-12）

- 新基线（§9.7 同 region/size，256 chunks median；**Java 侧为跨 bench 近似比较**，judge D 标注）：Rust off 75.94 / **l2 27.69** / l2 8 线程 ~4.5 ms/chunk vs Java FULL ~32-33。
- est L2 无损优化已消除立项时目标差距（260903-08：Rust 72-77 vs Java 33，慢 2.2×）→ 单线程恢复快 ~1.2× 方向（260903-10 大样本同向），8 线程 ≈ 7× Java（量级判断）。GPU 路径 dispatch/readback 成本（369ms/chunk）在小批量下为负收益。
- **建议（用户拍板 260903-12）**：gpu-batch-merge **降级为低优先级保留，不删除**——未来确有「超越 Java 10×+」目标，届时重新立项。重议触发条件与量化门槛见 `.artifacts/lossless-accel/gpu-merge-revisit-260903-12.md`：10× Java ≈ 3.3ms/chunk，需 GPU 批次摊销后 readback+dispatch < ~1ms/chunk 且计算侧不劣于 CPU（~56 chunk 批次级摊销起算）。

> 过程（P0 复现 → scout → 探针搭建 → 裁决 → 三件套 → P2.4 → 重议 → 两轮 judge）→ 10 时间线 260903-12 条；新课题四项（off −1 偏移 / surface_rules.rs:505 panic / +16 角修正 / 翻默认拍板）→ 10 时间线「新课题登记」。

## 2026-09-03 est 两处偏差修复：off 臂扫描偏移 + 角参数 +15→+16（260903-13，四臂同 hash，confirmed + 翻默认已实施）

> 承接上节「est 优化收口」遗留的新课题 #1（off 臂 −1 扫描偏移）与 #3（角参数 +16 修正待办）。修复 commit 3e2e67d；judge review 见 `.investigations/lossless-accel/review-offscan-cornerfix-260903-13.md`（PASS）；结论 `.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md`（judge PASS + 用户拍板批准翻默认 → confirmed）。

### ✅ 修复一：off 臂扫描首点 off-by-one（半开区间 rev）

- **根因（机制）**：Rust `(min_y..min_y+noise_height).rev().step_by(8)` 是半开区间 rev——首采样点 = **319**；Java `NoiseChunk.computePreliminarySurfaceLevel`（forge official sources NoiseChunk.java:174）为 `for(l=minY+height; l>=minY; l-=cellHeight)`——**320..-64 含两端**。半开区间 rev 使首点差 1、下界差 step（编译器惯用法条目，见 knowledge/discovered/compiler-idioms.md 发现 #10）。
- **修复**：扫描改为闭区间含两端对齐 Java（首点 320、下界 -64 均含）。

### ✅ 修复二：角参数 +15→+16

- **根因（机制）**：Java `MaterialRules.java:496-499` SURFACE 四角用 `chunkToBlockCoord(i+1) = (i+1)<<4` 即 **+16**；Rust 两臂曾用 `cx*16+15`（经量化 +12 ≠ +16）——系 workflow-patterns #25「静态调研结论失真」第三例的实例修复（上节已登记为待办）。
- **修复**：两臂（off/shared）heights4 角参数统一改 +16。

### ✅ 验证（Full 层，§9.7 口径见 verdict 头部）

- **修复后四臂 hash 完全一致 `f2b1a3932c6e589e`**（off/shared × L2 开关，64 chunks A/B）。
- Java est 角列对比：off / shared 各 **256/256 一致，0 diff**；敏感 chunk (201,200) 角值一致（java@+16=56）。

### ✅ 翻默认已实施（260903-13 用户拍板 confirmed）

- `WG_EST_SHARED` / `WG_EST_L2` **默认启用**，`env_enabled()` 助手 = 默认开、显式设 `0` 反转关闭（诊断用）。
- 验证：默认臂 shared=true l2=true hash 同值 + L2 命中 84.9%；双 `0` 反转臂同 hash + L2 stats 归零（`estopt-ab-defaultflip-260903-13.txt`）。
- est 优化语义零差达成 → 翻默认由语义裁决问题变为纯性能决策，已落地。

> 过程 → 10 时间线 260903-13 条；编译器惯用法 → compiler-idioms 发现 #10。


## 2026-09-03 surface_rules.rs:505 大 region panic 修复：overworld 预加载 noise key 清单缺项（260903-14，confirmed 待用户拍板）

> 承接 260903-12 新课题登记 #2（estopt-sweep 至 ~2304-2560 chunk 处 panic `missing noise sampler`）与本篇 260903-12 节内 ⚠️ 附注。修复 = 清单补一行；judge 审查 PASS（`.investigations/panic-505/review-panic-fix-260903-14.md`，2 should-fix 已清偿）；结论 `.artifacts/panic-505/panic-fix-verdict-260903-14.md`（candidate）。过程与错误台账 → `.investigations/panic-505/`（panic-errors.md E1-E3）。

### ✅ 根因（机制）

- overworld 预加载 noise key 静态清单（`worldgen_handle.rs` L272）缺 `minecraft:badlands_pillar_roof`；`place_badlands_pillar`（`surface_rules.rs:1372`）运行时 `get_noise` → `expect` panic。
- 触发面极窄：仅 eroded_badlands biome 列且侵蚀度 e>0 时走到该 rule → 64×64 sweep 至 ~2304-2560 chunk 才首次命中，小样本测试全绿掩盖缺失。

### ✅ 修复

预加载清单补 `minecraft:badlands_pillar_roof` 一行（与运行时查询集合同步）。

### ✅ 验证（Full 层，§9.7 口径见验证记录）

- 4096 chunk sweep 全程无 panic（修复前 64×64 sweep 必崩于 ~2304-2560）。
- 四臂 hash `f2b1a3932c6e589e` 零回归（off/shared × L2 开关，四臂完整落盘 `estopt-ab-4arms-260903-14.txt`）。
- 存档口径 3 采样 {98.9969, 99.0284, 99.0067}%（均值 99.0107%），vs 修复前历史 98.9520%——区间不重叠向上；散布 495 块在非确定带宽内（同族判据见 workflow-patterns #10），改善幅度在散布带内仅作无回归佐证。

> 通用模式 → workflow-patterns 发现 #26（预加载/注册表与运行时查询集合同步——expect 型查表缺失在低频分支才触发，大 region sweep 是暴露手段）；build-tooling 发现 #15（run 存档口径照抄历史参数清单，`-PcppWorldgenDir` 必带）。


## 存档口径残差模式化（260904-03）

> 补充既有「存档口径 3 采样」（260903-14 节）记录——**量级已记录，本节补模式签名**；无 §15.4 取代关系（量级读数互相兼容）。

### 🔍 现象量化（seed 8576294172403134396，4×4 @ chunk(200,200)，WGB2 FULL 存档口径）

- cppReplace（Rust dll）vs vanilla 参照：**13328/1572864 cell 差，99.1526% 一致**；差异遍布全 16 chunk（414–1347 cell/chunk）。
- top 互换对（ref→cpp）：**1(stone)→9(water) n=4165**、37→34 n=1687、1→34 n=1581、37→9 n=1359、32→0 n=520、2→9 n=439、34→9 n=399——主体为 stone/deepslate 族 ↔ water/air 互换，**aquifer/深板岩族特征**（液面判定与深板岩分层交界处方块身份互换，非坐标错位形态）。
  - ⚠️ **supersedes（260904-05）**：本行 id 注读有误——blocks.json 实测 9=dirt、32=water、34=sand、37=gravel（数字 n 本身无误）：top 对实为 **stone→dirt / gravel→sand / stone→sand / gravel→dirt / water→air / granite→dirt / sand→dirt**，「aquifer 液面互换」定性被推翻。真签名 = blob 状 stone/gravel/花岗岩族→dirt/sand 置换（最大连通域 615 cell，全 y 均匀每 y≈35），指针见下方「存档口径残差真签名改判」小节。
- est off 臂（double-0，判别力受限见 verdict 局限①）与默认臂逐位一致 ⇒ **残差与 est 优化/翻默认无关**（既有差异，est 语义无关性独立复证）。
- 载体/覆盖面/可比性（§9.7）：WGB2 FULL 存档口径 4×4 单区域；16 chunk×98304 cell；与 260903-14 存档口径 3 采样同族载体但区域不同，不可直接数值比（99.15% 落其 99.01% 同族带附近）。

### 机制候选（均未验证，续推前 MUST 廉价独立验证）

1. aquifer floodedness 判定输入/语义差（stone→water 单向占大头，指向流体判定未触发：floodedness 采样 / barrier 噪声 / 邻居 blob 选择）。
2. 流面高度/est 网格在深板岩过渡带边界翻转（#15 零面擦边格签名族）。
3. surface/carver 级联（低先验，需残差 y 分布数据）。

> ⚠️ **supersedes（260904-05 探针裁决轮）**：上述三候选**全部封闭**——残差 y 分布全高均匀（每 y≈35）与液面/est 边界带/surface 阶段均不符；真机制 = blob 状置换（见「存档口径残差真签名改判」小节），本候选清单仅存档。

### 下轮探针

残差 y 分布直方图先行（最廉价）→ AQF-APPLY 型配对（注意 AQF-J NPE 既有噪声 + 07 篇 L291 反射 CellCache 不可信铁律）→ 互斥候选 ≥2 按 fan-out 纪律并行。

## 存档口径残差真签名改判（260904-04 探针裁决 + 260904-05 落盘）

> supersedes 本篇「存档口径残差模式化（260904-03）」小节的「aquifer/深板岩族特征」定性与三机制候选（§15.4 取代链：原文不删不改，取代记录已就地标注）。状态：candidate（探针裁决轮 + judge 待过）。裁决 verdict 落点建议：`.artifacts/lossless-accel/residual-signature-verdict-260904-04.md`（欠账待补登记）。

### ✅ 改判内容

- **id 注读纠错**：260904-03 top 对「1(stone)→9(water)」系 id 映射误读——blocks.json 实测 **9=dirt、32=water、34=sand、37=gravel、2=granite**。top 对真实含义 = stone→dirt n=4165 / gravel→sand n=1687 / stone→sand n=1581 / gravel→dirt n=1359 / water→air n=520 / granite→dirt n=439 / sand→dirt n=399。
- **真签名**：blob 状 stone/gravel/花岗岩族 → dirt/sand **置换**（最大连通域 615 cell）；y 分布全高均匀（每 y≈35）——与 aquifer 液面（应有 y 带）、est 深板岩过渡边界带、surface 阶段（应集中浅层）均不符。
- **三候选封闭**：❌ aquifer floodedness / ❌ 流面 est 翻转 / ❌ surface-carver 级联（y 均匀分布直接排除后两者；分布形态与液面带不符排除前者）。
- §9.7 口径：同 260904-03（WGB2 FULL 存档口径 4×4 @ chunk(200,200)，16 chunk×98304 cell）。
- ❌ 已挂起（260904-15 用户拍板）：本篇挂起族 = aquifer 域 2（(198,18,198) water→dirt、(237,42,224) gravel→water，固/液边界真实方块级残差）+ ore_vein 域 1（(237,41,224)）——永久挂起，归因保持 candidate、坐标不删；光照/流体课题可引用为已知差异源；多世界新 seed 流下此族为残差放大候选（详注见 11 篇）。

## 2026-09-11 C 线：Java 侧 bulk section 写回（D1，Rust 零改动）— candidate（judge PASS-with-conditions；M1/M2/M3 已应用 / confirmed 留用户）

> 载体与依据：`.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`（candidate）+ `.investigations/bulk-writeback-260911-05/record-260911-05.md`（§3 两个错误 / §4 Tier 1-4 / §5 Tier 3 判据未满足（FAIL）+ 未归因 / §6 R8/R9）+ `plan-260911-05.md` §0（修订 R1-R6）+ `evidence/`（MANIFEST sha 清单）。提交 `8dd9e71`（判后修订 `cfd2036`/`ea545d1`/`e6c900b`）。通用模式 → knowledge/discovered：algorithm-fingerprints #22/#23、compiler-idioms #24 补充案例、workflow-patterns #129 / #25 补充案例 / #110 补充案例 / #14 补充案例（subagent 草稿 → 主会话应用）。

### 形态（D1 落地）
- 把「Rust 填好的扁平 `int[]`（y-major raw block id）」一次性转成 section 级 `PalettedContainer`，取代**每 chunk 98,304 次 `ChunkSection.setBlockState`**；调用点与时机不变（NOISE 阶段接管点内、高度图之前），**Rust/JNI/数据驱动边界零改动**。
- 路线 = 公开 `PalettedContainer.readPacket(PacketByteBuf)`（自建 buffer；原案 5 参构造器因第 3 参 `DataProvider` 是包私有 record 而不可调用）+ `ChunkSectionAccessor` mixin（4 写 + 3 读）**原地换容器 + 直写三计数**（不新建 section、不调 `calculateCounts`、生物群系容器原样保留）。
- 回退开关 `-Dcoreswap.bulkwb=0`（旧逐块路径**保留不删**，即时降级 + A/B 单变量）；诊断 `-Dcoreswap.bulkwblog` / `-Dcoreswap.wbcontent` / `-Dcoreswap.bulkwbtest`（默认全关，生产零成本）。

### 等价性（Full，噪声无关；§9.7 三要素）
- **载体**：同 JVM 内、写回**之后**读回全部 section 算 FNV-1a 64（`[WG-CONTENT-WB] hash=`）+ 三计数序列指纹（`dh=`）。
- **覆盖面**：写回调用 607（overworld）/ 625（nether）/ 625（end），各 24/16/8 section × 4096 位置（overworld = 59,670,528 位置）。
- **可比性**：同构建态、同 dll（`838e89794a54e19d`）、同 seed/区域、背靠背两臂，**唯一变量 = `-Dcoreswap.bulkwb`**。
- **结果**：Tier 1（状态层）与 Tier 2（派生层）**逐 chunk 差均为 0**，两臂 chunk 集相同（`only_old=0` / `only_bulk=0`）、零异常；编码四支 SINGULAR/ARRAY/BI_MAP/**ID_LIST** 合成自检逐位读回全绿（自然生成 `max_distinct=7` / `idlist_hits=0` ⇒ ID_LIST 生产不可达，必须刻意压测）。
- ⚠️ `[WG-CONTENT]`（Rust **buf** 层指纹）**与本门不同层**：buf 不变则 hash 必相同 ⇒ 对 Java 侧消费的变更**不敏感**，C 的等价性只能由本门承载（判据 → workflow-patterns #14 补充案例）。

### 两个实施期错误（最高价值，五段式见错误台账）
1. **E1**：`bits==0` 的单态 section **不得构造 `PackedIntegerArray`**（`Validate 1..32` 抛 IAE，首跑 539 次）——位宽 0 = **无 storage**，不是「0 位 storage」；复刻 MC 的 switch 必须逐 case 过构造器前置条件。
2. **E2**：`calculateCounts()` 与增量 `setBlockState` 的**计数语义不一致**（流体重复计入 `nonEmptyBlockCount`；`nonEmptyFluidCount` 只计有随机刻的流体）⇒ 走构造器会**静默改变** `hasRandomFluidTicks()` / 客户端包语义；已改为复刻**增量**语义（生成期 vanilla 语义由调用点决定）。三计数**不入存档**（`ChunkSerializer:310-311`）⇒ region 对拍看不到，唯一可见面 = 生成期内存态。

### 性能（Partial：分项计时，跨 run 只作趋势）

| 维度 | 旧逐块 ns/chunk | bulk ns/chunk | 变化 |
|---|---|---|---|
| overworld | 2,151,591 | **652,202** | −69.7% |
| nether | 2,657,477 | **859,626** | −67.7% |
| end | 2,809,444 | **804,970** | −71.4% |

- 分项（定稿轮）：`build=75,666 ns/section`（含 `readPacket=5,963`）、`counts_set=116 ns/section`（原 `calculateCounts` 72,480 ns/section）；`sections_replaced=4856` / `air_sections_skipped=9712`（= 607×24）。
- ⚠️ **价值定位**：**可维护性/可移植性为主**（消掉逐块循环与 per-version 诊断，为共享 Java 适配核与 1.21.6 收敛铺路）；性能为次要附带，写回仅占 chunk 时间约 2-6% ⇒ **端到端落在 ±10% 机器噪声带内，不予主张**（本 run wall/CPU 因指纹门重载不可用于端到端）。

### Tier 3 判据未满足（FAIL）+ 残差未归因（judge M1/M2 修订；原「降级」定性作废）
- **预注册判据**（`verify-design-draft-260911-05.md:29-33`）= **`diff = 0`**（**不是**「≤ 噪声基线」）；实测全域 `blocks=313,589,760 diff=86,841 = **0.0277%**`（`sections same/diff = 74822/1738`）⇒ **判据未满足（FAIL）**。**不得**写成「不可达 / 不可判 / 降级为噪声底受限」——那是把 FAIL 掩盖成测量能力问题。
- **补跑变更臂自身对照（`bulk × bulk`，2 个新 run；同 dll / 同参数 / 唯一变量 = run）**——接管坐标集跨 4 run 完全相同（607，差 0），切片可跨对直接比较。切片 A = 607 个走过 bulk 写回的 chunk（59,670,528 块）；全域 = 313,589,760 块：

  | 对 | 性质 | 切片 A | 全域 |
  |---|---|---|---|
  | old × old2 | 同实现(old) | 0.0249% | 0.0242% |
  | old × old3 | 同实现(old) | 0.0239% | 0.0253% |
  | old2 × old3 | 同实现(old) | 0.0284% | 0.0238% |
  | **bulk × bulk2** | 同实现(bulk) | **0.0352%** | 0.0258% |
  | **bulk × bulk3** | 同实现(bulk) | **0.0358%** | 0.0284% |
  | **bulk2 × bulk3** | 同实现(bulk) | 0.0272% | 0.0263% |
  | **old × bulk（交付对）** | 单变量=写回路径 | **0.0348%** | 0.0277% |
  | old × bulk（关随机刻/天气/刷怪/火焰） | 单变量 | 0.0311% | 0.0259% |

- **三条定稿结论**：① 预注册判据 `diff = 0` ⇒ **未满足（FAIL）**（全域 0.0277%）；② 交付对 0.0348% **落在 bulk 臂自身跨 run 区间 0.0272–0.0358% 之内** ⇒ **无「写回引入系统性存档层差异」的证据**；弱信号如实登记（含 bulk 的 5 对全域 0.0258–0.0284% vs 两 old 的 3 对 0.0238–0.0253%，两组不重叠 ⇒ 最自然读法 = **bulk 臂自身跨 run 离散度更高（方差效应而非均值效应）**；n = 3/组、切片 A 区间仍重叠 ⇒ **仅提示性、未达显著**）；机制候选并列：**A** 运行期调度（bulk 每 chunk 快 ~1.5 ms ⇒ 改变并行生成流水线中「邻块写特征」先后）/ **B** 容器编码差异（**不成立方向**：Tier 1/2 已证写回后状态逐位全等）/ **C** 开放；③ **噪声底主体在 vanilla FEATURES 阶段**——关掉全部 live-server 随机源后差分仍 **0.0259%**、top pairs 仍是特征产物 ⇒ **随机刻只占 6–11%**（0.0277→0.0259；切片 A 0.0348→0.0311）。
- **分母口径**：313,589,760 块中 **58.1%（1854 个 chunk）从未生成内容**（`ChunkSerializer` 对光照范围内每个 section 无条件写盘 ⇒ 未走到 NOISE 阶段的部分生成 chunk，**纯稀释分母**）；两侧皆空 chunk 差分**恰好 0**，100% 的差分落在有内容的 **1336** 个 chunk 上 = **0.0661%**。凡后续用 region 层百分比做判据，MUST 声明「分母含未生成 chunk」。
- ❌ **排除清单（judge M2）**：「切片 A（0.0348%）低于切片 B 有内容子群（≈0.0922%）⇒ 写回落在差分更低的一半」——**归因谬误**，该落差在**同实现对照**里同样成立（old × old2 0.0249% vs 0.0853%、old × old3 0.0239% vs 0.0908%、old2 × old3 0.0284% vs 0.0805%），属**切片群体构成属性**，与写回无关。
- **等价性主张边界**：C 线的等价性**只由 Tier 1/2 承载**（写回点内存态逐位全等，决定性）；**存档层不主张等价**。判据沉淀 → `knowledge/discovered/workflow-patterns.md` 发现 #129（三条可复用判据：终态二分 / 变更臂自身对照不可替代 / 切片归因适用条件）。

### R8/R9 论证（judge 前置）
- **R9（锁语义）——❌ 原结论「新路径不比老路径弱且在锁粒度/原子性上更强」已被 judge（S1/A12）推翻，更正如下**：`readPacket` 锁的是**刚构造、尚未发布、其他线程不可达的私有容器** ⇒ 对**共享容器零互斥**；老路径锁的是**已发布的活容器**，其锁兼作「多线程访问同一容器」的 crash **检测器**（`LockHelper`）——该检测能力**被移除**。「每 chunk 24 次锁 vs 老路径数万次」**不是同一件事的强弱**，且 24 只是**上界**（实测均值 **8.0 次/chunk** = `sections_replaced 4856 / 607`）。**正确表述**：正确性完全**外移到「每 chunk 单写者」这一外部保证**（老路径也依赖，但其锁附带检测）；该保证**未验证**（R9-b）。仍成立的子结论：**发布原子性**方向成立（`PalettedContainer.data` 是 `volatile`，`readPacket` 在发布前完成全部内部写 ⇒ 读者只见旧或新）；但 section 数组里的容器引用本身非 `volatile`，形式上仍是 data race。
- **R8（`BelowZeroRetrogen`）**：1.20.1 该 flag 仅由**旧存档 chunk NBT** 携带（`ChunkSerializer:185/281-284`）；本 harness 每臂删 world 重新生成 ⇒ flag 恒不置；且 C 不动生物群系容器、不动 FEATURES 阶段 ⇒ 即便触发也无新增暴露面。

### 遗留 / 未覆盖
- **遗留 / 未覆盖**：R9-b 并发可见性专项未做；`plan §10.3`（消费者缓存前置）已收口——**原地换容器** ⇒ 缓存 **section 实例**的消费者安全，缓存**容器**的消费者会读到**孤儿容器**（残留风险：modpack/未来版本若新增此类消费者，本实现会静默失效）；**编码非规范位宽（judge S5，登记不修）**：`BulkWb.java:243` 写**请求位宽**，vanilla `Data.writePacket:384` 写的是 `storage.getElementBits()` ⇒ ARRAY 支写 1..4（规范恒 4）、ID_LIST 支写 9/10（规范恒 15）；**解码等价、今天无缺陷**，留待 1.21.6 移植随适配核修正；ID_LIST 仅合成覆盖（生产不可达）；`MAX_ID=4096` 沿用老路径同款约束；**sync 形态未跑**；端到端 wall 不主张；**Tier 3 弱信号未闭合**（bulk 臂自身离散度略高，n=3 未达显著）；共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）。

## 2026-09-11 A 线：Java 侧写回优化池（A1a 跳空气 / A1b 诊断门控）+ Rust 共享层线程 clamp（A1d）+ nether/end 全维行为门（A2）— candidate（judge PASS-with-conditions；C1-C12 + I1-I4 已应用 / confirmed 留用户）

> 载体与依据：`.investigations/a1-opt-pool-260911-05/record-260911-05.md`（A1a/A1b/A1c/A1d + 行为门 4140/4140 + §9.7 降级声明）+ `a2-dim-gate-260911-05.md`（nether/end 各 4761/4761）+ `review-260911-05.md`（judge 原文 + §7 处置表）；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`）。提交 A1a+A1b `b2b2f26`（**同提交**）/ A1d `b53b23f` / A2 `8d8075a` / judge 修正 `e8decef` / docs `6b90998`。
> ⚠️ **与 C 线小节的路径关系**：C 线（本篇「2026-09-11 C 线」小节）把默认写回改走 bulk section 路径；**A1a/A1b 所在的旧逐块路径保留不删**（回退 `-Dcoreswap.bulkwb=0`）⇒ 本节结论对该回退路径**仍然有效**，不得读作「已消失 / 已被 C 线取代」。

### A1a 写回跳空气（`CppBridge.writeChunk`，回退 `-Dcoreswap.skipair=0`）
- **形态**：`writeChunk` 遇 raw id 0（= `minecraft:air`，`data/blocks.json` 0）不再逐格 `setBlockState`；开关静态 final ⇒ 常量折叠，生产路径实际新增每格一次 `id==0` 判断（judge I4）。
- **判据（预登记）**：跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；谓词取 **`!isOf(Blocks.AIR)`**（`isAir()` 会漏计 `cave_air` 730 / `void_air` 729，而跳过的写是「写 air 默认态」）。
- **vanilla 依据**：`ChunkSection.setBlockState` 只随 `isAir` 变化增减 `nonEmptyBlockCount`；`PalettedContainer.swap` 同值写回同一 palette index ⇒ 对空气格写 air 为语义 no-op（judge 补引；机制指纹 → algorithm-fingerprints #25）。
- **等价性（Full，噪声无关；§9.7 三要素）**：
  - **载体**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），`[WB-CHECK]` 全格计数。
  - **覆盖面**：三维全格计数，非抽样——overworld 276,171,456 / nether 194,630,604 / end 154,752,709 空气格被跳过，`stale_nonair` **三维均 0**；覆盖面分母（经接管 chunk 数）见 A2 段（overworld 4140 / nether·end 4761）。⚠️ 该项为**单 run 单维、无负对照、与计时同 run**（judge 核对表）。
  - **可比性**：a1 双臂同 dll `838e89794a54e19d`、同 seed/区域中心/工具修订、背靠背串行；三归档臂（exec-def/cap0-fresh/exec-16）为 **A1d 前构建** `dd3b645f2c79d2cb`，跨 dll 逐 chunk 指纹全等（4140/4140）⇒ 兼作 **A1d 输出中立性独立证据**。
  - **结果**：等价性成立（三维实证）；行为门跨臂 + 跨 dll + 跨 session 归档臂指纹 `hash_diff=0`。
- **收益：Degraded（不主张定量）**——cpuSec 493→476（**−3.4%**），单对跨 run 且**落在 ±10% 机器噪声带内 ⇒ 只作趋势**；两臂均开 `WBCHECK`（2.76 亿次 `getBlockState`）⇒ **含诊断成本的收益下界**，引用定量数字须关 WBCHECK 重跑。结论性质 = 「等价性成立 + 无回归」。

### A1b 诊断扫描门控（`CppBridge.fillChunk`）
- **形态**：`nzBuf` 全 buffer 扫描（98,304 读/chunk）+ nether/end 16 点读回**整块移入 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。
- **判据**：纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。
- **取舍声明**：改前 `buf-all-air`/`buf-sparse` 为**无条件 println**（原「只门控 println」理由句错误、类 javadoc 旧措辞作废）；`buf-sparse` 分级诊断随全量扫描移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。

### A1d `adaptive_threads` count=1 clamp（Rust 共享层，清理项）
- **形态**：`worldgen-core/src/api.rs` 的 `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`；批量路径（count>1）语义逐字未变。
- **机制自证（运行期）**：`[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll 两条日志行）。
- **与上文 2026-08-16 clamp 小节的关系（本条为该待办在 Rust 共享层的收口）**：8-16 发现的历史前提是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 **per-call `std::thread::scope`**（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ 前提消失、clamp 安全（judge 独立核对）。
- **归口理由**：本篇已承载 clamp 课题的发现与待办（上文「[B]/实机 M=1 结构性串行（threads clamp 发现，candidate）」），且 `03-density-functions.md` 无 threading/池宽章节（`线程池|adaptive_threads|池宽` 零命中）⇒ **不归 03 篇**。
- **无性能归因**（双臂都含 clamp，未做隔离 A/B）；**改前 nthreads=10 为公式推导非实测**（旧 dll 无自证行）。

### A2 nether/end 全维行为门（`CppBridge.fillChunkEnd` 补指纹载体）
- **开工缺口（judge 核实属实）**：end 路径原本**没有 `[WG-CONTENT]` 指纹行**——首轮 end 臂 `intercepted=4761` 而 `contentLines=0`；判据意义 = **「接管生效」与「行为门可判」是两件事**，门禁类课题 MUST 先核「该维是否有载体行」再谈门值。修复 = MIXLOG 块内补指纹行。
- **判据（三条可复用）**：① 该维有载体行（否则先补载体）② 跨形态 diff=0（证 exec/异步化不改内容）③ 跨 run diff=0（证该维 Rust 输出确定性）。
- **结果（Full，噪声无关；§9.7 三要素）**：
  - **载体 / 覆盖面**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），**经接管的全部 chunk 均进门、非抽样**——overworld 4140、nether 4761、end 4761；**分母 = 接管调用数 ≠ Chunky Processed**（门判据不依赖分母）。
  - **可比性**：9 条维度臂 dll 全为 `838e89794a54e19d`、同 seed/区域中心/工具修订；nether 臂与 end 臂**不同 Java 构建态**（end 在补指纹行后重跑）已声明——`git show 8d8075a --stat` = 2 files/57 insertions，Java 仅 +4 行且全在 `fillChunkEnd`，不影响 nether/overworld 路径。
  - **结果**：三维跨形态 + 跨 run 逐 chunk 指纹差 **0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；正对照 overworld vs nether `hash_diff=4140/4140`（门有检测力）；比对脚本已加**最小载体阈值断言**（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）。
  - **打印位置**：overworld 的 `[WG-CONTENT]` 在 `writeChunk` **之前**、nether/end 在其**之后**；三者都对**只读 `buf`** 计算 ⇒ hash 语义一致。
- **附带读数（非门判据，Degraded）**：nether async 25s/24s vs sync 69s（~2.8×，cpuSec 212/202 vs 149）；end async 11s/8s vs sync 20s（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致；跨 run 摆动 **±27%** ⇒ 不宣布定量收益。
- **❌ 排除清单（一行，防重走弯路）**：① 「nether run 级非确定来自 Rust 填充层」以外的下游归因（Java carver/feature/装饰层、写回路径、存档序列化）= **未测外推**，不得当结论引用（judge C8）；② 早前「85 个 Chunky 已处理但无接管行」算式**不成立**——期望集不是 Processed 集（judge C12）；③ A1c「6 张高度图全量重扫」= 前提被 vanilla 一手源证伪（`Heightmap.java:37-71` 本就单遍）⇒ 已评估·不实施（用户裁决）。

### 遗留 / 未覆盖（A 线）
- **写回路径未覆盖（降级）**：指纹取在 `writeChunk` 前/后但**只哈希只读 `buf`**，不覆盖写回结果；**写回后内容指纹（post-write hash）三维均未做**（260910-06 open ⑤ 延续；→ 260912-01 的 `[WG-CONTENT-WB]` 层部分回补该缺口）。
- overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」已废，降为 open 假设）；nether run 级非确定**成因域未测**（只排除 Rust 填充层）。
- A1d 改前 `nthreads=10` 为公式推导非实测；`dd3b645f` 是否确为 A1d 前构建按时间线推断（未反汇编）。
- 1.0.29 **不含 A1 改动**（尚未随任何 release 出货）；1.21.6 侧同构问题属 Phase B 范围。

## 2026-09-12 D3 共享 Java 适配核（Wave 2 语义统一，commit `997d40f`）
> ⚠️ 260912-02 取代指针（2026-09-12）：本节的「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」已被取代 —— 见本文件末尾「D3 后续（260912-02）」节：根因 = storage 段帧契约变更、修复 F1 = commit 2c3be2a、运行级验证已过。原文保留不改。— ✅ confirmed（用户授予 2026-09-12 15:52；范围 = 出货线 1.20.1 生产行为不变 + 共享核抽取 + 1.21.6 默认路径冒烟；不含 1.21.6 强制 bulk 缺陷/等价性（仍开放）；V1b PASS-with-declarations；1.20.1 V2+V3 双 PASS；**1.21.6 默认臂 PASS + 强制 bulk 臂发现确认缺陷（阻断 D-4(i)）**；judge = PASS-with-conditions（C1–C9 已响应）/ confirmed 留用户）

> 载体与依据：`.investigations/shared-java-core-260912-01/record-260912-01.md`（§2 pre 冻结 + 构建确定性 / §2.4 V0 接线预检 / §4.1 Wave 1 / §4.2 HOOK-2 / §4.6 V1b 判定 + **构建三态表** / §4.7.0-§4.7.8（V2+V3 / dll 血统事故 / 1.20.1 双 PASS / **1.21.6 回填 + 确认缺陷** / dll 归一化 / 证据落盘 / 一手锚补正 / judge 条件响应 / 未闭合项））+ `review-wave2-260912-01.md`（judge，verdict = PASS-with-conditions）+ `errors-260912-01.md`（E1–E5 + 速查表）+ `evidence/`（42 文件 + `MANIFEST.txt`，tracked）+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（§14 追加式补登）；提交 `ce5286b`（Wave 1 纯移动）+ `997d40f`（Wave 2 语义统一）。通用模式 → knowledge/discovered：workflow-patterns #138 / #14 补充案例（260912-01）、build-tooling #59（合并）/ #60 / #61、compiler-idioms #25。

### 形态（共享核抽取）
- `java-core/src/main/java` = **单一源**；两版 `build.gradle` 各加 `sourceSets { main { java { srcDir '../../../java-core/src/main/java' } } }`（各 +11 行），`git mv` 1.20.1 版入共享、`git rm` 1.21.6 副本（**一个类只有一个家**）。
- **分版差异收进分版缝类 `WgCompat` ×2**：`BULKWB_ON`/`SKIPAIR_ON` 默认常量（1.20.1 = true / 1.21.6 = **false**）；`WgCompat.flag(prop, def)` 语义 = 设了 property 则「非 0 即真」（`0` → false，其它非空 → true，未设 → 默认）⇒ 1.21.6 可用 `-Dcoreswap.bulkwb=1` 强制开启而**不改码**，翻转 = 改一个常量。
- 共享超集：`CppBridge`（并入 1.21.6 独有项 + `WgCompat` 引用）、`BulkWb`/`StallWatch`/`ChunkTiming`（超集）/`CoreSwapFixHelper`；1.21.6 补 `mixin/ChunkSectionAccessor` + `coreswap.mixins.json` 25 条 + refmap 重建（S-1）。
- 回退/门控形态不变：`-Dcoreswap.bulkwb=0`（旧逐块路径保留不删）、`-Dcoreswap.skipair=0`、`-Dcoreswap.chunktime`（默认关）、`-Dcoreswap.bulkwbtest` 等仍有效；1.21.6 `BULKWB_ON=false` ⇒ 该版**默认不调用** bulk 写回。

### 判据（分档，可复用；→ workflow-patterns #138）
- **V1-strict**（**纯移动子集**）：条目级 sha 全等（最好整文件 sha 不变）——Wave 1 实测两版 jar sha **逐字节不变**（1081/1081、1798/1798）。**反向蕴含**：字节全等 ⇒ 同一执行体 ⇒ 行为必然相同，**V2/V3 免跑**（防后波次偷懒/重复劳动）。
- **V1b**（**语义统一波**）：① **未变更侧**条目 MUST 逐字节不变（两侧都变者无字节锚，MUST 显式声明理由）；② 变更条目 MUST **逐条声明**（文件 + 变更性质 + 依据；`jar_manifest_diff.py --expect` 只做归类，**归类 ≠ 声明**）；③ **任何 class 字节发生变化的版本 MUST 补跑 V2 + V3**（「jar 不同」不再蕴含「行为相同」）；④ **判定基线 MUST 钉死构建态**（本波三态：post1 含头注释 → **post2 = V1b 判定基线** → post3 权威 dll；用错态会把「头注释行号位移」或「dll 归一」混进 Java 面差异 —— judge C6）；⑤ **运行期门 MUST 覆盖被测路径与维度**（本波 V3 只覆盖 bulk 写回路径 = 1.20.1 生产路径 + 仅 overworld ⇒ `writeChunkPerBlock` 与 nether/end **无运行期证据** —— judge C3/C7）；⑥ **「默认关」的共享路径 MUST 至少跑一次冒烟**——1.21.6 原本不存在 bulk 路径、共享化后变成可达，强制臂一开即崩（130 chunk `EntryMissingException`、根因未定位、阻断 D-4(i)）⇒ 「默认关」不等于「该路径不存在」（`errors-260912-01.md` E5）。
- **V2/V3 运行期门（两层指纹齐跑）**：`[WG-CONTENT]` = **Rust buf 层**（引擎输出，对 Java 侧改动不敏感）、`[WG-CONTENT-WB]` = **Java 写回读回层**（覆盖写回代码变更）；判据 = 两族指纹 **multiset 判等**（多重集证集合；序列不作判据——异步流水线完成顺序不确定，六臂两两无一对序列相同）；**§15.4 校正（judge 增量复审 D2，2026-09-12）**：原「与 sorted-sequence 双判全等／序列相等另证顺序」**已废**——六臂两两原始序列**无一对相同**（同 jar 重复跑 `pre-r1 vs pre-r2` 位置差 495/503）⇒ 顺序由异步流水线决定、**不承载判据**；判据只用 **multiset**（6 臂 `fp-*.txt` sha256 全同）。**只跑 buf 层 = 假安全感**（→ workflow-patterns #14 补充案例 260912-01）。

### 结论（1.20.1 出货线：行为不变）
- **V2 + V3 双 PASS**（**三臂**同配方、唯一变量 = Java 源码状态、同一 dll `dd3b645f`；pre 臂跑 **2 次**）：441 chunks；异常行实测 **post 4（全 WMI 良性）/ pre 12（8 WMI 良性 + 4 条 `testcontent` 无关噪声）**（原「0 / 0」写法已作废 —— 未测量即断言，E4）；两层指纹各 **607/607**（三臂两两比对）且 multiset 全等（`only-pre=0 / only-post=0`；序列不作判据）；**§15.4 校正（judge 增量复审 D2，2026-09-12）**：原「与 sorted-sequence 双判全等／序列相等另证顺序」**已废**——六臂两两原始序列**无一对相同**（同 jar 重复跑 `pre-r1 vs pre-r2` 位置差 495/503）⇒ 顺序由异步流水线决定、**不承载判据**；判据只用 **multiset**（6 臂 `fp-*.txt` sha256 全同）⇒ ① 引擎层输出未变 ② **Java 写回结果逐 chunk 全等**。⚠️ **覆盖面**：三臂均未设 `-Dcoreswap.bulkwb` ⇒ 走 bulk（= **1.20.1 生产路径**）；**`writeChunkPerBlock`（逐格/回退路径）从未执行** ⇒ 其「未变」仅有指令级证据、无运行期证据；**nether/end 零覆盖**（被改的 `writeChunk` 共享于三维度）。**§9.7 口径**：仅 overworld / 仅 bulk 路径；与 C 线 region 层噪声（0.024%）**不同层、不可混用**，本判据为逐 chunk 精确等，且同实现跨 run 在**本层无噪声**（pre-r1 vs pre-r2 全等）。
- **class 条目变化 100% 逐条声明**：1.20.1 = 6 差异 + 1 新增（4 条经 `javap -c -p` 证明「指令完全相同、仅调试属性」`LineNumberTable`；`BulkWb.class` = 仅 `<clinit>` 常量求值序列改走 `WgCompat`，逐值等价；`ChunkTiming` = 已批准超集；`CppBridge` = 超集 + 分派/计时钩子；`StallWatch` = **字节全等**）；1.21.6 = 5 差异 + 5 新增（mixin json/refmap 各 +1 条、2 条仅调试属性、`CppBridge` 超集合并、5 条新增）。
- **1.21.6 侧**（默认 `BULKWB_ON=false`）：**默认臂 PASS** —— boot 42.1s；bridge init（seed / worldgenDir / `stageMask=3`）全对；`[WG-CONTENT]` **625** + `[WG-CONTENT-WB]` **625**（**该版首次带上读回门**）、`[WG-BULKWB]` 0（符合预期）；非 WMI 真实异常 **0**；jar 内 `coreswap.mixins.json` 25 条含 `ChunkSectionAccessor`、启动无 `Mixin apply failed` / `Cannot find target method`。**但**：① **写回替换的内容等价性仍无证据**（该版 pre 无 `BulkWb` / 无读回门 ⇒ 无同层 pre 对照）；② **强制 `-Dcoreswap.bulkwb=1` 臂崩解 = 确认缺陷**（130 chunk 全抛 `EntryMissingException: Missing Palette entry for index 2…8`，栈 `ArrayPalette.get` ← `PalettedContainer.get` ← `ChunkSection.getBlockState`；`[WG-CONTENT-WB]` **0**、流水线未完成；**根因未定位**）⇒ 不阻断 Wave 2（默认关）、**阻断 D-4(i) 翻转**（须先定位 palette 未填充注入点，再跑「默认 / `bulkwb=0` / `bulkwb=1`」三臂逐 chunk diff=0，且覆盖 nether/end）。
- **可复用过程判据（→ KB）**：① 跨臂/跨 run 对照前 **MUST 逐臂读「执行体自证行」**（`[CppBridge] dll= sha256=`）核对，**不得只看 target 文件 sha**（本轮 target 被「A/B 收尾 restore」与「实验暂存」两次改写 ⇒ 首轮对照执行了不同引擎、判定作废；build-tooling #59 + `errors-260912-01.md` E1）；② **fresh `git worktree` 不能当「条目级」基线**（检出文本被 `core.autocrlf` 物化为 CRLF ⇒ `worldgen-data/**` **1024 条**伪差异、**class 条目 0**；条目级基线 MUST 同工作树，跨树比对只可用于类文件/运行期判据；build-tooling #60 + `errors` E2 + 一手锚 record **§4.7.6**）；③ **需要字节锚的逐字复制文件不得加头注释**（`LineNumberTable` 随源码行号位移 ⇒ 条目 sha 变、指令不变；compiler-idioms #25）；④ **注释修补保行数即保字节**（`ChunkTiming.java` javadoc 逐行替换 ⇒ `post4` ≡ `post3`、差异 0；compiler-idioms #25 的正向对偶）；⑤ **门数字 MUST 复算 + 附可复现命令 + 原始输出落盘 + 定义口径**（本波异常行曾写 0/0、实测 4/12；`errors` E4）；⑥ **javap 差异计数 ≠ 语义差异**（三陷阱：lambda 序号 / zip 级联 / 常量池未归一；`errors` E3 + build-tooling #61）。

### 遗留 / 未覆盖（Wave 2）
- **1.21.6 强制 bulk 缺陷（确认缺陷）**：`EntryMissingException: Missing Palette entry for index 2…8` 根因未定位 ⇒ **阻断 D-4(i)**；默认臂干净 ⇒ **不阻断 Wave 2**（`errors-260912-01.md` E5 + record §4.7.8）。
- **内容等价性缺口**：① 1.21.6 写回替换无证据（该版 pre 无对照载体）；② `writeChunkPerBlock` 无运行期证据（1.20.1 回退路径；低成本闭合 = 跑 `-Dcoreswap.bulkwb=0` 臂与生产臂对比 `[WG-CONTENT-WB]`，未做）；③ **nether/end 零覆盖**（本波门仅 overworld，而被改的 `writeChunk` 共享三维度）。
- **judge 已做**（`review-wave2-260912-01.md` = PASS-with-conditions，C1–C9 已全部响应/修正，record §4.7.7）；**record §6 已回填 / 用户已 confirmed 2026-09-12 15:52**；1.20.1 class 条目**未做全量 javap 逐条对拍**（4 条 `javap -c -p` 全等证明 + 一次方法级对拍；judge 抽样复核 5/5 成立）；`stateById` 归一后 6 行残差已用**源码逐字对照**排除（序号伪差）。
- 1.20.1 生产路径新增 **4 次 `System.nanoTime()`/chunk**（≈80ns/chunk 为算术估计、未实测；保留 1.21.6 逐字形态、不做微优化）；1.21.6 S-6/S-7 诊断面差异已按批准生效。
- **证据落盘（judge C5）**：判据已从 `.tmp/` 复制到 tracked **`evidence/`**（42 文件 + `MANIFEST.txt`：清单 / 差异输出 / 门控原样行 / 完整日志 / 复现工具；jar 本体与中间 dump 不入库、只入 sha）⇒ 判据可从仓库复现（此前「只在 `.tmp`」的状态已终结）。
- **再发布提醒**：`build/libs` 现产物与已发布 1.0.29 的 jar sha `b057fda2…` **不同**（发布 jar 已被本地构建就地覆盖，§2.6「构建产物目录不是存档目录」）⇒ 再发布 MUST 重跑全量回归 + 三元组重算。

## 2026-09-12 D3 后续（260912-02）：1.21.6 强制 bulk 写回的 storage 段帧契约缺陷定位与 F1 修复 — ✅ confirmed（用户授予 2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化）

> 范围与排除（照抄状态机口径，不得外推）：本节 confirmed **只覆盖上列 4 项**；**不含** nether/end 覆盖、性能结论、`BULKWB_ON` 翻转决策（三者均未获 confirmed）。
> 证据载体：`.investigations/shared-java-core-260912-02/`（`verify-260912-02.md` v1→v6 = 验证记录，**confirmed（范围受限** = 本节 4 项结论；范围外条目按原证据等级）；`judge-260912-02.md` = judge 三轮 + 交付前确认，PASS-with-conditions；`errors-260912-02.md` = W1–W13 五段式 + 速查表（台账，candidate）；`scout-map.md` + `scout-interpretation-A1.md` = 静态字节码与运行级解读；`evidence/` = 原始件）+ 冻结基线 `.investigations/shared-java-core-260912-01/evidence/`。提交 = `2c3be2a`（F1 本体，2026-09-12 17:13:50+0800，`evidence/javap-seam-260912-02.txt:37`）；`8974063 docs(260912-02): close review conditions, register artifacts, supersede E5`（2026-09-12 17:56:53+0800；两个提交号均已由本稿用 `git cat-file -t` 属主工具自核，符合 workflow-patterns #137）。
> 承接：本节取代上节（`## 2026-09-12 D3 共享 Java 适配核…`）中「1.21.6 强制 bulk 缺陷 = 仍开放 / 根因未定位」的表述；原节正文**不改**，取代指针以追加式插在该节标题行之下（见本节末「就地取代指针」）。

### 现象（260912-01 遗留缺陷的实测面）

| 量 | 值（修复前） | 来源 |
|---|---|---|
| 1.21.6 强制 bulk 臂 `[WG-CONTENT]` | 130 | `shared-java-core-260912-01/evidence/arm-summary.txt:42` |
| 1.21.6 强制 bulk 臂 `[WG-CONTENT-WB]` | 0 —— **首要信号**（写回读回层一条都没有） | 同上 |
| `EntryMissingException` | 132 | 同上 |
| 异常类型 / 索引取值 | `DIAG write threw chunk(x,z): net.minecraft.world.chunk.EntryMissingException: Missing Palette entry for index 2.` 逐 chunk 抛；索引取值集合只有 {2, 8}（C1 判别臂实测 `index 2` ×41 + `index 8` ×9，无第三取值） | 同上 `:32-41`（抽录 10 条）；`scout-interpretation-A1.md:228` |

- 不是崩溃而是「整段移位」（关键性质）：异常只在越界位置抛出；位移本身静默（C1 臂 `PASS distinct=1` 之后 `FAIL branch distinct=3`，`i=0` 解出的正是 writer 位置 4 的值 = +4 个 4-bit 位置的逐位实证，`scout-interpretation-A1.md:255-265`）。
- 默认关不豁免：该路径在 1.21.6 上原本**不存在**（该版 pre 无 `BulkWb`、property 被忽略），共享核把它变成**可达**；强制臂一开即崩（`errors-260912-01.md` E5，`:110-132`）。

### 根因 b2a：storage 段帧契约随版本变更，共享核仍按 1.20.1 前缀帧写出

| 项 | 1.20.1 | 1.21.6 |
|---|---|---|
| `PalettedContainer.readPacket` storage 段读法 | `PacketByteBuf.readLongArray([J)` = VarInt 长度前缀 + 定长 | `PacketByteBuf.readFixedLengthLongArray([J)` = 定长、无前缀 |
| tiny 映射名（逐行核对） | `.gradle-home/caches/fabric-loom/1.20.1/…/mappings.tiny:17600` → `method_10789 = writeLongArray` | `…/1.21.6/…/mappings.tiny:19802` → `method_68087 = writeFixedLengthLongArray`（另 `:19781 method_68086` = `(ByteBuf, long[])` 重载同名） |
| 字节码判据（单变量差异） | 两版 readPacket 指令偏移 5/15/24/39/45 逐条相同，唯一差异 = 第 39 条 invokevirtual 的目标不同 | 同左（`scout-interpretation-A1.md:21-23`；`evidence/A1-javap-PalettedContainer-{1.20.1,1.21.6}.txt`） |

- 机制链：共享核写端 `BulkWb` 按 1.20.1 契约写「VarInt 长度 + longs」；1.21.6 读端只取 `values.length` 个 long ⇒ 那 2 字节前缀**不被消费**（ARRAY 支 `VarInt(256)` = `0x80 0x02`）⇒ 整段移位 2 字节 ⇒ 解码越界的 palette 索引只能是 {2, 8}（前缀 nibble `e15=8, e14=0, e13=0, e12=2`；`scout-interpretation-A1.md:261-271`）。
- 竞争候选 b2b（位宽协商语义差）已排除：读端 byte 消费路径、`getCompatibleData` 决策骨架、`createDataProvider` 实现三处逐条同形 + 运行级 {2, 8} 单变量分布（`scout-interpretation-A1.md:239-251`）。

### 修复 F1（commit `2c3be2a`）：分版缝收口写端帧

- 改法：分版缝（`versions/<ver>/java/src/main/java/wg/bench/WgCompat.java`，每版一个同名文件；共享源不得出现版本分支）新增 `writeStorageLongs(PacketByteBuf, long[])`；共享核 `BulkWb.buildContainer` 的 **2 个调用点**（原 `:247` `EMPTY_LONGS` 支 + `:262` 数据支）改为调用缝方法，两版各自调用本版正确的写出方法。
- jar 级证据（`evidence/javap-seam-260912-02.txt:4-34`）：两版缝体都是**纯转发** —— `invokevirtual #45` 调**同一个被调方法**（1.20.1 = `class_2540.method_10789`、1.21.6 = `class_2540.method_68087`）+ 同一实参（`aload_0` / `aload_1`），`pop` 丢弃返回值，**体内无任何写字节指令**；`BulkWb` 内该调用出现 2 次（`L386` / `L479`，同属 `buildContainer`）。
- 1.20.1 腿的性质：缝体是对原直接调用的静态转发，因此 1.20.1 写出帧逐字节同构（这是「1.20.1 未观测到 F1 相关回归」的机制依据，不是仅靠测试）。

### 验证（修复后实测）

| 判据 | 实测 | 来源 |
|---|---|---|
| 1.21.6 bulk 臂计数 | `[WG-CONTENT]` = `[WG-CONTENT-WB]` = 625（修复前 130 / 0 / 132） | `evidence/arm-summary-260912-02.txt:7` |
| 全目录 `EntryMissingException` | 0（8 臂全扫；仅修复前 C1 臂与常量池字节码文本命中） | `judge-260912-02.md:53`（A2） |
| WBTEST 合成自检 | 4/4 PASS（distinct = 1 / 3 / 20 / 300） | `evidence/arm-summary-260912-02.txt:29-33` |
| 按层多重集对比 | 全等（`only-pre=0` / `only-post=0`）——本稿逐组枚举：主对比 8 组（`cmp-260912-02-raw.txt`，8 个 `=====` 段）+ 自证臂对比 8 组（`cmp-260912-02-att-raw.txt:4-66`）+ 补跑 2 组（同文件 `:95-109`）= **18 组** | 三份 `cmp-*.txt` 原始输出 |
| 条目级对拍（jar manifest） | 相同 1079 / 1800、差异 3、增 0 删 0，`V1 verdict = PASS（无非预期差异）` | `evidence/manifest-diff-postF1-1.20.1.txt:6-9` 与 `:23`（1.21.6 同型） |
| 1.20.1 与冻结基线 | 整文件全 64 位 sha 同一 = `4b10f2be7f9d5d949dd0acd048b8c0b40ac0653bc09083b45d7968bf13ed9e21`（83804 B）；同 sha 家族 1.20.1 = 10 份 / 1.21.6 = 4 份 | `verify-260912-02.md:92`（C-13）+ `:175`（F28） |
| 1.21.6 默认臂未退化 | fix-default 与冻结默认臂逐层多重集全等（625 / 625） | `cmp-260912-02-raw.txt:29-53` |
| 臂变量生效自证（正 / 负成对） | `-Dcoreswap.bulkwblog=1`：bulk 臂 `[WG-BULKWB] calls=625`；perblock 臂 `[WG-BULKWB] calls=0` + `[WG-PERBLOCK] calls=607` | `cmp-260912-02-att-raw.txt:68-79` |

> ⚠️ 数字复核（本稿，两处汇总数字按实际枚举更正）：① 指令 / 计划口径的「16 组多重集对比」在落盘证据中**无法复现** —— 按 `cmp-*.txt` 逐组枚举为 **18 组**（8 + 8 + 2），本节按 18 记；② 「1.21.6 五份 fp 同一」为汇总层笔误，穷举定案为 **1.21.6 = 4 份 / 1.20.1 = 10 份**（`verify-260912-02.md:10` v4 修订、`judge-260912-02.md:451-453`）。

### 残留 / 边界（如实写，不得当已闭合）

| 项 | 状态 | 说明 |
|---|---|---|
| nether / end 覆盖 | 0（零覆盖） | 属 `BULKWB_ON` 翻转的**前置**（**非本结论前置**）；被改的 `writeChunk` 共享三维度 |
| 1.21.6 写出帧字节直采 | 未做 | 由读端 4096 点读回（WBTEST 4/4）+ 625 chunk 等价性代替 |
| 1.20.1「未观测到 F1 相关回归」口径 | 受限 | 仅限 **overworld / 写回内容层 / 单次 JVM 运行** 口径 |
| 性能结论 | 不做 | 本块不出性能主张（bulk 与逐块计时同 run 含诊断成本，不作收益读数） |
| `BULKWB_ON` 翻转 | 未做 | nether / end 覆盖为 0 即前置未达 |
| 出货 | 不出货 | F1 对 1.20.1 为静态转发、无紧急重发需求；若出货须升版 1.0.30 + 新工单 |
| `verify` / `errors` 状态 | verify = **confirmed（范围受限**，仅标题所列 4 项；范围外条目按原证据等级）；errors = candidate（台账） | confirmed 只覆盖 4 项范围，不外推 |

### 可复用判据

- 跨版本帧契约变更 ⇒ 写端必须走分版缝：凡「共享核 + 分版缝」结构，任何写 wire 格式的调用点都 MUST 收敛到单一分版缝函数，禁止散落的直接调用（本块修复即此形态）。
- 指纹对写回内容面敏感，但观察面有限：`[WG-CONTENT]`（Rust buf 层）对 Java 侧写回改动结构性不敏感；`[WG-CONTENT-WB]`（写回后读回层）能捕获写帧错误。两层指纹只可排除「写回内容发生变化」，不可单独支撑「无回归」——必须与单运行口径异常面同档、条目级 diff、构建绿并列（`verify-260912-02.md:176`，F29）。
- 等价性结论 MUST 与臂变量生效自证配对（→ `knowledge/discovered/workflow-patterns.md` 发现 #139）；跨版本帧契约指纹 → `knowledge/discovered/algorithm-fingerprints.md` 发现 #26。

### 就地取代指针（供主会话插入到上节 D3 节内，见段 1b）

> ⚠️ 260912-02 取代指针（2026-09-12）：本节的「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」已被取代 —— 见本文件末尾「D3 后续（260912-02）」节：根因 = storage 段帧契约变更、修复 F1 = commit 2c3be2a、运行级验证已过。原文保留不改。
