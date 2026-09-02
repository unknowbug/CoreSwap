# B1 下钻四候选判别 —— 判别探针采集记录（260902-08）

> 日期 260902-08（实际 2026-09-02 晚）。承接 260902-07 H1 环1 证伪转向。
> 本文件记录判别探针（SURFACE 前/后逐列 dump）的实现与采集过程 + 关键发现。

## 探针实现

### SurfaceDumpProbeMixin（hook SurfaceBuilder.buildSurface HEAD/RETURN）
- `runtime/1.20.1/java/src/main/java/wg/bench/mixin/SurfaceDumpProbeMixin.java`
- HEAD = 「前」（NOISE 产物），RETURN = 「后」（SURFACE 产物）
- dump 每列：材质序列（raw id 自顶向下）+ biome id + 顶面 y（WORLD_SURFACE_WG）+ useLegacyRandom 标志
- 门控 `-Dsurfacedump.probe=1`，chunk 过滤 `-Dsurfacedump.chunkX/Z/size`

### NoiseDumpProbeMixin（hook NoiseChunkGenerator.populateNoise RETURN）
- `runtime/1.20.1/java/src/main/java/wg/bench/mixin/NoiseDumpProbeMixin.java`
- 意图：对 vanilla 与 cppReplace 都生效的 NOISE 后 dump
- **失败**：cppReplace 模式下 `NoiseChunkGeneratorMixin.wgPopulateNoise`（HEAD cancellable）提前 return，RETURN 注入点不触发；vanilla 模式 populateNoise 异步（CompletableFuture），RETURN 时块未填充

## 关键发现（数据直读）

### 1. 坐标语义澄清（重要）
- `bench.originX=3200` 是 **chunk 坐标**（非 block 坐标）——BlobProbe 用 `nether.getChunk(3200, 3208)` 直接传 chunk 坐标
- 区域 = chunk(3200,3208) 起 4×4 = block(51200,51328) 起 64×64
- 参照文件名 `vanilla_8576294172403134396_4_3200_3208_nether.blocks` 的 "3200_3208" 是 chunk 坐标

### 2. nether legacy_random_source = true（确认）
- `noise_settings/nether.json` L13 `"legacy_random_source": true`
- → `usesLegacyRandom()` = true → biome 判定用 **y=0**（恒平），非顶面 y
- 探针实测：`biomeY=0, legacy=true`（vanilla 轮）

### 3. vanilla NOISE（PRE）地形 = 均匀（候选 d 关键证据）
- 4096 列全部 `topY=127`（WORLD_SURFACE_WG = 基岩顶 y=127，非地形面）
- 4096 列全部 `biome=minecraft:basalt_deltas`
- 材质 = 纯 netherrack（5850）+ 基岩顶/底
- **结论：NOISE 阶段地形无形状变化（均匀 y=127），候选 (d) 前置地形形状差无「形状」可差**

### 4. vanilla SURFACE（POST）规则输出（候选 a/b/c 参照）
- netherrack → **basalt(5854)** 上层（y=5~114，峰值 y=79-82）
- netherrack → **blackstone(19319)** 下层（y=17~39，峰值 y=21-23）
- netherrack → **bedrock(79)** 顶（y=123-127）+ 底（y=0-4）
- netherrack → **118**（未识别，y=26-33，疑 gravel）
- 完整分布见 `.tmp/analyze_surface_rule.py` 输出

### 5. cppReplace 架构事实（探针设计约束）
- cppReplace 模式：`populateNoise` 被 Rust `fillChunkNether` 接管（noise+surface+features 一起），`buildSurface` 被 skip
- → `SurfaceBuilder.buildSurface` hook 在 cpp 轮**不触发**（无 cpp surfacedump 数据）
- → 判别探针的「SURFACE 前/后」概念在 cpp 侧不成立，需改走「读存档」路径

## 采集数据清单

- vanilla PRE：`.tmp/surfacedump/vanilla-pre-c3200-3208.csv` 等 16 文件（4096 列）
- vanilla POST：`.tmp/surfacedump/vanilla-post-c3200-3208.csv` 等 16 文件（4096 列）
- cpp 轮：无 surfacedump 数据（buildSurface 被 skip）
- 分析脚本：`.tmp/analyze_vanilla_pre.py` / `analyze_surface_rule.py`

## 下一步（待主会话决策）

1. **候选 (d) 判别需「noise-only cpp」对照**：vanilla PRE（noise-only）vs cpp 存档（noise+surface+features）阶段不同，air pocket 对比（13.70% 匹配）被 surface/features 填充污染，不构成 (d) 的干净判别。正确做法 = cpp 轮加 `WG_SKIP_SURFACE=1 WG_SKIP_FEATURES=1`（wg_set_flags bit0|bit1）跑 noise-only，再与 vanilla PRE 对比 air pocket 签名。
2. **cpp 侧 surface 输出已读**（`read_cpp_save.py`）：basalt 主导（11k-18k/chunk）+ blackstone（1.2k-4.4k）+ netherrack（0.8k-5.9k）+ lava（0.1k-5.4k）——与 vanilla POST（basalt 上层 + blackstone 下层）的差异即 B1 家族残差本体。
3. 候选 (a/b/c) 判别：对比 vanilla POST vs cpp 存档 surface 输出（材质序列逐列对拍）。

## 关键语义澄清（沉淀）

- `WORLD_SURFACE_WG` 高度图 = **基岩顶 y=127**（nether 基岩顶），非地形面——nether 地形形状在 air pocket（cave）分布，不在高度图。
- nether 地形 = 实心 netherrack 到 y=127（noise height 128），基岩顶 y=127-128、基岩底 y=0-4。
- 「V 黑石底 y=99 vs C 玄武岩底 y=100~104」是 **surface 规则层**（basalt/blackstone 替换 netherrack 的深度），非 NOISE 地形。
