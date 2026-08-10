# 7.x FEATURE 实施：CARVERS + FEATURES 阶段（2026-08-10 Phase 1-5 实施 + 2026-08-11 树花植被废弃决策）

> **状态：candidate**（2026-08-11 验证基线确认；未经 judge 审查与主会话裁决，未授予 confirmed）
> **性质：追加式章节草稿**——不修改 07-block-pipeline.md 既有内容，独立成章供后续并入。
> **证据源**：`.investigations/feature-pipeline/pipeline-map.md`（管线地图）+ `cmd-output/*.txt`（各阶段实测）+ `cpp/worldgen/src/*.h`（代码注释锚点）+ `cpp/worldgen/deprecated-vegetation/README.md`（废弃决策归档）。
> **文风约定**：每条结论附验证方式；未验证推断明确标注为 candidate/待立项。

## 功能目的

补全 C++ worldgen 的 `SURFACE → CARVERS → FEATURES` 两阶段（Java ChunkStatus 链尾部）：

- **CARVERS**：洞穴雕刻（`cave`/`cave_extra_underground`/`canyon`）——把 aquifer 判定的实心块按洞穴体挖空，液面以下填水/岩浆。此前 -288 差异的「洞穴空气 + 含水层水」即缺此阶段（约 17% 差异构成，见 07 篇追加 4）。
- **FEATURES**：地形性装饰——岩石替换（ore_granite/tuff/diorite/andesite，Phase 3）、简单装饰（disk/spring/freeze_top_layer/underwater_magma，Phase 4）。树花植被（flower/random_patch/simple_block/tree）**已废弃**（Phase 5 验证未达标，2026-08-10 用户拍板，2026-08-11 代码迁移 deprecated-vegetation/）。
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
- **结果**（`phase4_result.txt`）：-288 FULL **97.8390%**（Phase3 97.8464% → -0.007%，magma 位置错引入 ~20 块）；300515 FULL **98.0975%**（Phase3 98.0948% → +0.003%，disk/spring 正确放置）；8576/3200 SURFACE 零退化保持。
- **HeightmapPlacementModifier 返回 top 不 +1**（`placement.h` L195-213）：
  - Java `Heightmap` 存 **topY + 1**（高度图语义），`HeightmapPlacementModifier.getPositions` 返回 `topY(heightmap, x, z)`（不额外 +1；k > bottomY 才返回）。
  - C++ 内部高度图存「块 y」（surface 内部消费需要 y 语义），HeightmapPlacementModifier 直接返回 C++ top（不 +1），与 Java 的 y+1 差 1。
  - **实测 +1 反而使 300515 降 0.12%**（disk/spring 变差）→ 保持 C++ y 语义（内部一致性优先）。生态装饰（花/草）已按拍板范围外移除，不依赖此语义差异。
  - 验证方式：`placement.h` L195-213 注释复盘 + `phase4_result.txt` 300515 +0.003%（disk/spring 正确放置）。
- **OCEAN_FLOOR_WG 高度图构建时机**：carver **前**（Java NOISE 阶段语义，挖洞不影响海底 top）——`worldgen_api.cpp` L1233-1234 注释锚点。

### Phase 5：树花植被——验证未达标 → 废弃（2026-08-10 拍板 + 2026-08-11 迁移 deprecated-vegetation/）

- **曾实现并接入**（2026-08-10 Phase 5）：SimpleBlockFeature / RandomPatchFeature（花/草）/ TreeFeature（oak/birch 直树 + fancy_oak 简化）/ RandomSelectorFeature。
- **验证未达标**（`deprecated-vegetation/README.md` 历史事实）：
  - **树只放 40%**：canGenerate 失败率高（origin ground 检查 / 树干空间检查失败）；
  - **300515 花爆炸**：dandelion C++ **533** vs 参照 **11**——树未实现 → 树冠区被当 air 放花；
- **废弃决策**（2026-08-10 用户拍板，README + feature_loader.h L67-70/L89-90 + worldgen_api.cpp L1360-1361 注释锚点）：
  1. **细节版本改动太多**——树/花/草植被在 MC 版本间差异大（1.20 → 1.21 大量变动），逐位对齐成本不可接受；
  2. **MOD 特别容易碰到**——实机 Mod 装饰主要挂 FEATURES 阶段，C++ 全接管会丢 Mod 花/草/树，兼容工作量不可接受。
- **2026-08-11 代码迁移**：实现代码剪出到 `cpp/worldgen/deprecated-vegetation/`（vegetation_features.h），主代码彻底移除接入点：`feature_loader.h` `generateOther` 对 flower/random_patch/simple_block/tree return false；`worldgen_api.cpp` random_selector return false；不参与编译、不接入调度。
- **恢复路径**：git 历史 c04768e 前的 feature.h 有完整版本；恢复需重新接入 feature_loader.h 分发 + worldgen_api.cpp 调度 + placement.h 植被 modifier，并重跑 Java 对拍。
- 验证方式：`deprecated-vegetation/README.md`（废弃状态 + 历史事实 + 禁用后基线）；代码锚点（generateOther return false / 不解析树花 config）。

## 验证基线（2026-08-11 实测，block_probe 逐位对照）

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

---
*草稿结束。待 judge 审查 + 主会话裁决后提升 candidate → confirmed；未授予 confirmed 前所有结论为 candidate。*
