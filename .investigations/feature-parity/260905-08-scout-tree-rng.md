# 260905-08 scout — Java vs Rust 树放置 RNG 消费链勘探（feature-parity）

- 角色：recode.scout（只读静态对拍；本文档为唯一产物写入）
- 课题：确定性区域 dump（seed 8576294172403134396，2193 chunks）树族残差 119,198
  （oak_leaves rust多 38,250 / rust少 18,988；jungle_leaves rust少 22,611 / rust多 2,147；vine rust少 21,961）
- 状态：draft（静态对拍，Degraded 分层；未做任何运行时验证）
- 知识库前置：已读 knowledge/INDEX.md——特别遵守 workflow-patterns #43（随机派生/组合选择层双向残差签名优先怀疑）、#51（跨 run 噪声基线同阶）、#52（确定性 dump 载体）；上轮「RNG 流上游偏移 / 放置判定残差」假设**未当公理**，本次为一手源逐点接线。

## 0. 源码定位

- Java 权威源：yarn sources jar 已存在并解包：
  `E:\PYTHON\MC\versions\1.20.1\java\.gradle\loom-cache\minecraftMaven\net\minecraft\minecraft-merged-7787b014d4\1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2\...-sources.jar`
  → 解包树 `E:\PYTHON\CoreSwap\.tmp\scout-260905-08\mcsrc\`（.tmp 临时区，非目标代码修改）。
- 参照数据：`versions\1.20.1\data\worldgen\data\minecraft\worldgen\`（注意层级：data/worldgen/ 是旧壳，实际 JSON 在 data/worldgen/data/minecraft/ 下）。
- Rust：`worldgen-core\src\{feature_loader,placement,tree,feature,chunkrandom,worldgen_handle}.rs`。

## 1. 管线接线图（Java ↔ Rust）

```
Java ChunkGenerator.generateFeatures (ChunkGenerator.java:334-423)
  chunkRandom = ChunkRandom(Xoroshiro128PlusPlusRandom)            :343
  l = setPopulationSeed(worldSeed, chunkX*16, chunkZ*16)           :344  (ChunkRandom.java:54-61)
  set = 3×3 邻域 chunk 的 biome 并集（ChunkPos.stream(section,1)）  :346-353   ★Rust 只用当前 chunk biome
  for k in 0..max(stepCount):
    structures: setDecoratorSeed(l, m, k) 独立重置                 :364       （Rust 跳过——无流污染，因子 seed 独立）
    intSet = ∪(set 中各 biome 的 features[k]) → indexMapping(p)     :381-395   ★Rust 只算 cur_features
    for o: p = sortedIntSet[o]
      chunkRandom.setDecoratorSeed(l, p, k)                        :402  (ChunkRandom.java:75-78)
      placedFeature.generate(world, gen, chunkRandom, blockPos)     :406
        → PlacementModifier 链 flatMap（惰性 DFS）                  (PlacedFeature.java:48-63)
        → ConfiguredFeature.generate（同一 random 流）
Rust worldgen_handle.rs:862-879
  set_population_seed(seed, cx*16, cz*16) ✔ 逐行同 (chunkrandom.rs:148-155 ↔ ChunkRandom.java:54-61)
  set_decorator_seed(population_seed, p, k) ✔ 同 (chunkrandom.rs:158-161 ↔ ChunkRandom.java:75-78)
  k 循环/ int_set_for / step_features[k][p]  ✔ 结构同 (feature_loader.rs:88-164 ↔ PlacedFeatureIndexer)
```

## 2. 逐问题回答

### Q1 population 遍历与种子派生
- Java：ChunkGenerator.java:343-344（setPopulationSeed）、:402（setDecoratorSeed(l,p,k)）、:358（k 循环上限=max(step 枚举数, 全局 step 数)）。index 排序 = intSet 升序 :393-395；p = indexMapping = Util.lastIndexGetter（step 内最后索引）。
- Rust：worldgen_handle.rs:862-879 结构一致；feature_loader.rs:105-164 的 build/int_set_for 与 PlacedFeatureIndexer 语义对齐（首现递增 featureIndex、lastIndex 覆盖写）。
- **差异点（候选 b2）**：Java `set` 是 3×3 邻域 biome 并集（:346-353），Rust 只有当前 chunk biome（worldgen_handle.rs:856-858，注释自认「简化：set = 当前 chunk biome」）。intSet 并集多出的邻 biome feature 会**额外执行**（Java :398-412 每 feature 独立 setDecoratorSeed，故不污染其它 feature 的流——影响的是「有没有放」，不是流偏移）。边界 chunk 少放邻 biome 树 → rust少 方向。
- **挂起域（只定界）**：populationSeed 数值派生两侧逐行一致；Xoroshiro 本体 nextLong/nextInt(bound) 的位级等价性未在本次勘探重验（此前已有验证载体），不深钻。

### Q2 placed 链 modifier 消费序
Java 各 modifier 每位置 RNG 消费（yarn 一手源）：
| modifier | 消费 | 源 |
|---|---|---|
| count(IntProvider) | 随 IntProvider：weighted_list 1、uniform 1、constant 0 | CountPlacementModifier.java + AbstractCountPlacementModifier |
| in_square | 2×nextInt(16)，先 x 后 z | SquarePlacementModifier.java:24-27 |
| surface_water_depth_filter | 0 | SurfaceWaterDepthFilterPlacementModifier |
| heightmap | 0（k>bottomY→空流） | HeightmapPlacementModifier.java:30-34 |
| block_predicate_filter | 0 | BlockFilterPlacementModifier |
| biome | 0 | BiomePlacementModifier.java:26-32 |
| rarity_filter | 1×nextFloat()<1/chance | RarityFilterPlacementModifier.java:24-26 |
| random_offset | **3 次**：xz.get, y.get, xz.get | RandomOffsetPlacementModifier.java:41-45 |

Rust（placement.rs）逐项对拍结果：
- Count :282-285 ✔、Square :289-291 ✔（x 先 z 后）、Heightmap :296-307 ✔（+1 修正 260905-06）、Biome :308-316 ✔0 消费、SurfaceWaterDepth :323-334 ✔、EnvironmentScan ✔。
- **差异点 1（候选 a-证据，placement.rs:286-288）**：RarityFilter 用 `next_int_bound(chance)==0`，Java 用 `nextFloat() < 1/chance`——同为 1 次消费但**抽法不同**，接受集不同（如 chance=4：Java 接受 float<0.25；Rust 接受 int∈{0}）。树链本体不用 rarity_filter，但同 step 的其它植被 feature 会。
- **差异点 2（候选 a-证据，placement.rs:392-395 + :319-321）**：RandomOffset Rust 解析成 `(xz, const0, y)` 且 get_positions 只消费 2 次（xz 一次）；Java 消费 3 次（xz 两次）。xz_spread 为非常量 IntProvider 的 feature（部分 patch/花）流必错。
- **差异点 3（候选 e-证据，placement.rs:308-316）**：Rust Biome modifier 判定 = `(x>>2)<<2 4×4 对齐采样 == anchor_biome(当前 chunk biome)`；Java = `world.getBiome(pos)`（BiomeAccess jitter）∈ feature 允许集。两处语义都不同（对齐采样 vs jitter、单 biome vs 允许集）→ 放置判定残差。

### Q3 树 placer 内 RNG 消费
主链顺序（TreeFeature.java:66-81）getHeight(2)→getRandomHeight(0)→getRandomRadius(0/1)→trunkPlacer.generate→逐 node foliage.generate（**offset 每 node 抽 1 次**，FoliagePlacer.java:48-52）→decorators。Rust tree.rs:497-547 顺序一致 ✔；offset per-node（tree.rs:179）✔；blob/bush/jungle isInvalidForLeaves 的 nextInt(2) 位置与短路序（tree.rs:245-263 ↔ FoliagePlacer.java:84-96/Blob:63-64/Bush:47-49）✔；MegaJungle 分支循环 `height-2-nextInt(4)`、每支 nextFloat×1 + 5×getAndSetState（tree.rs:585-603 ↔ MegaJungleTrunkPlacer.java:39-51）消费次数 ✔；setToDirt/getAndSetState 只在放置时消费 provider（tree.rs:568-574, :623-637 ↔ TrunkPlacer.java:54-66）✔；LargeOak 每候选分支 2×nextFloat 顺序 ✔（tree.rs:654-676 ↔ LargeOakTrunkPlacer.java）。

发现的差异点：
- **差异点 4（候选 c-证据，tree.rs:585-593）**：MegaJungle 分支 `j=(int)(1.5+cos(f)*l)` Rust 用 `f32::cos/f32::sin`；Java 用 **MathHelper.cos/sin（SinLUT 查表）**。非线性查表 vs libm → j/k 位置不同 → 枝干端点、TreeNode 位置、jungle_foliage 团位置系统性偏移。**jungle_leaves rust少 22,611 的强候选**（mega_jungle 树冠半径大，单树偏差放大）。Rust 仓库已有 `carver::math_sin`（MathHelper.sin 查表复刻）却未在此接线。
- **差异点 5（候选 c-证据，tree.rs:331-400 ↔ TreeFeature.java:151-154）**：TreeDecorator 遍历序。Java trunk/leaves 集是 `Sets.newHashSet()`（HashSet 迭代序 = 桶序），Rust 用「插入序按 Y 稳定排序」近似（tree.rs:341, :363, :379 注释自认遗留 idk R-1）。jungle_tree/mega_jungle_tree 带 cocoa+trunk_vine+leave_vine：每 leaf 4×nextFloat（leave_vine）在同一 set 迭代序下消费——**迭代序不同 → 流从此点分叉 → 同树后续 decorator 甚至无影响但同流后续树已重 seed（独立）**。注意：decorator 在树尾、树间 seed 独立重置 → 分叉只影响本树 decorator 放置（vine/cocoa 数量位置），不外溢。vine rust少 21,961 与此吻合（Java HashSet 序放置的 vine ≠ Rust 序的 vine，块数差双向但 vine 大头 rust少）。
- 另登记：树叶 `distance` 属性重写 no-op（tree.rs:548-552）——按 block id 家族对拍无影响，声明已知。

### Q4 worst chunk (29,-16) 一带的可用数据
`.tmp/feature-parity-260905-06/`：
- `dumpnew/region_new.bin` + `dumpold/region_old.bin`：确定性区域全量块 dump（2193 chunks，header 含 seed，SHA256 见 dump-hashes.txt，260905-06 C-2 生成）——**唯一可靠对拍锚点**（载体 #52）。
- `tree_chunk_probe.bin`：header `IDK7` + seed 8576294172403134396 ✔ + 边界 (29,29,-16,-16,-64..)——**单 chunk (29,-16) 的历史 idk7 exe 产物，格式无在库生成者源**（全仓 grep 无 producer），不可直接复用，仅证明 worst chunk 曾被单点采样。
- `ab-takeover.log`：gradle mixin 接管日志（populateNoise/buildSurface 逐 chunk 行），**无 per-feature RNG/放置日志**。
- `v10/v11/v12/*.out.txt`：残差分解（树族双向表、top deltas、Y 分布、worst chunk 表）——已提炼于本文开头。
- 可用而未用：Rust 侧已有 `WG_FEATURELOG` env（worldgen_handle.rs:880-882，逐 chunk 逐 step/p/fid 打点）——判别实验现成钩子。

### Q5 互斥差异候选（.bN 预留）

| 编号 | 候选 | 证据定位 | 强度 | 双向残差解释力 |
|---|---|---|---|---|
| **.b1** | 树 placer 内：MegaJungle MathHelper.cos/sin（查表）vs Rust libm f32.cos/sin | tree.rs:585-593 ↔ MegaJungleTrunkPlacer.java:41-43；math_sin 已在 carver.rs 有复刻未接线 | **高**（一手源可静态确证实现不同；jungle 树冠大、放大效应强） | jungle_leaves 少 22,611 + 多 2,147；jungle_log 差 |
| **.b2** | feature 集差：Rust 只用当前 chunk biome，Java 3×3 邻域并集 + Biome modifier 语义差（4×4 对齐采样 vs BiomeAccess jitter、允许集） | worldgen_handle.rs:856-858,922；placement.rs:308-316 ↔ ChunkGenerator.java:346-353,381-395；BiomePlacementModifier.java:26-32 | **高**（简化注释自认；边界 chunk 系统性少放邻 biome 树） | oak/jungle 双向残差（交界带双向） |
| **.b3** | decorator 迭代序：HashSet 桶序 vs Rust Y 稳定插入序 | tree.rs:341,363,379 ↔ TreeFeature.java:122-154 | 中（vine/cocoa 局部；树间流独立不外溢） | vine 少 21,961、cocoa 差 |
| **.b4** | placement modifier 抽法差：RarityFilter nextInt==0 vs nextFloat<1/chance；RandomOffset 2 vs 3 次消费 | placement.rs:286-288, :392-395+319-321 ↔ RarityFilterPlacementModifier.java:24-26, RandomOffsetPlacementModifier.java:41-45 | 中（树链本体不触发；同 step 植被/patch feature 受影响） | 植被类残差（非树主因） |
| **.b5** | 放置判定残差：heightmap 快照静态（Rust 预建 OCEAN_FLOOR 不随 feature 放置更新；Java chunk heightmap setBlockState 即更新）+ 越界 block_at=-1 保守拒绝 vs Java 读邻 chunk 实况 | worldgen_handle.rs:826-842（只建一次）；placement.rs:296-307,905-907 ↔ ChunkRegion.getTopY/Chunk heightmap update | 中（密集树区/water 邻域偏差；worst chunk 也是地表/stone 族双高大户） | 树堆叠/水边树双向 |
| .b6（排除候选） | populationSeed/setDecoratorSeed 数值派生、featureIndex/lastIndex 排序 | chunkrandom.rs:148-161、feature_loader.rs:88-164 ↔ ChunkRandom.java:54-78、ChunkGenerator.java:381-402 | 低——**静态逐行一致**，且 b2 的流独立性论证（per-feature reseed）进一步削弱「index 序差致流偏移」：p 不同只改本 feature 的 seed 起点，属于「本 feature 错位」而非流污染 | — |

互斥性：.b1 与 .b3/.b4 是**不同消费点**的独立差异（可同时为真，非严格互斥）；与 .b2/.b5 是「放置集合差」与「流内差」两类。真正互斥的顶层分叉 = **流内差（.b1/.b3/.b4）vs 集合差（.b2/.b5）**——两者残差签名不同（集合差=整树缺席双向；流内差=树形/局部块差）。

## 6. 最小判别实验建议

1. **判 .b1（最便宜、优先）**：worst chunk (29,-16) 单 chunk，Rust 临时把 mega_jungle 分支 cos/sin 换 `carver::math_sin`（查表），确定性 dump 重生成 → jungle_leaves 残差变化量。预期若 b1 主导，残差显著收敛；换回即回滚（单点 A/B，#20 死参数自检：先确认 (29,-16) 邻域确有 mega_jungle biome——WG_FEATURELOG 打点验证 fid=mega_jungle 相关 feature 实际执行）。
2. **判 .b2**：挑 1 个「中心 chunk = plains、3×3 内有 jungle」的边界 chunk（dump 里 vanilla 有 jungle 树、rust 无的位置），Java 侧 mixin 打 setDecoratorSeed(l,p,k) 逐 feature 日志 + Rust 开 WG_FEATURELOG 同 chunk 对拍 p 序列与 feature 集——直接看集合差。
3. **判 .b3**：单树隔离（构造单 chunk 单树 seed 或直接读 Java HashSet 序——在 Java 探针工程打印 decorator 的 set 迭代序一次，与 Rust 排序序对照）。
4. **判 .b4**：静态可确证，不需实验；修 implementing 时顺手统一为 nextFloat 口径。
5. 噪声基线：任何 A/B 用同一确定性 dump 载体重生成（#52），不要用跨 run 存档对比（#51）。

## 7. 边界声明

- 静态对拍（Degraded）；未运行任何探针；置信度=draft。
- 未重验 Xoroshiro 本体位级等价、biome features JSON 加载序（b2 的 index 部分依赖 biome 枚举序——若 Rust biome 枚举序与 Java registry 序不同，p 值整体漂移，此点并入 .b2 实验一并暴露）。
- Java 参照源为 yarn 1.20.1+build.10-v2 sources jar（版本已核，f5-bugs #5 教训）。
