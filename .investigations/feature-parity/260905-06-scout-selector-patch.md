# 260905-06 scout 取证：random_selector / random_patch / placed-feature / vine decorator 管线地图

- 角色：recode.scout（只读勘探；不承担分析解读；置信度 = **draft**，验证分层 = **Degraded（纯静态读源码，无 trace/probe）**）
- 源码参照权威：yarn 映射源码树 `versions\1.20.1\data\mc_src_extract\`（1.20.1）；缺类补位见 §0。
- 结论状态：draft。供 core.worker 分析解读用，不作为对拍依据。

## 0. 站点清单（类存在性 + 一手源路径）

| 取证对象 | 找到？ | 一手源路径 | 映射系 |
|---|---|---|---|
| RandomSelectorFeature | ✅（不在 mc_src_extract） | `E:\PYTHON\CoreSwap\.tmp\net\minecraft\world\level\levelgen\feature\RandomSelectorFeature.java` | **mojmap**（levelgen 包名；其余为 yarn）。mc_src_extract 无此类（yarn 树缺该文件） |
| RandomPatchFeature | ✅ | `versions\1.20.1\data\mc_src_extract\net\minecraft\world\gen\feature\RandomPatchFeature.java` | yarn |
| PlacedFeature | ✅ | `...\mc_src_extract\net\minecraft\world\gen\feature\PlacedFeature.java` | yarn |
| CountOnEveryLayerPlacementModifier（mojmap 名） | ✅（yarn 名 = **CountMultilayerPlacementModifier**，type id `COUNT_ON_EVERY_LAYER`，@Deprecated） | `...\placementmodifier\CountMultilayerPlacementModifier.java` | yarn |
| CountPlacementModifier / AbstractCountPlacementModifier / SquarePlacementModifier / HeightmapPlacementModifier / BiomePlacementModifier | ✅ | 同目录 placementmodifier\ | yarn |
| LeavesVineVineDecorator（任务用名） | ✅（yarn 名 = **LeavesVineTreeDecorator**） | `...\treedecorator\LeavesVineTreeDecorator.java` | yarn |
| TrunkVineVineDecorator（任务用名） | ✅（yarn 名 = **TrunkVineTreeDecorator**） | `...\treedecorator\TrunkVineTreeDecorator.java` | yarn |
| TreeDecorator（Generator 收集容器） | ✅ | `...\treedecorator\TreeDecorator.java` | yarn |
| TreeFeature（decorator 调用链 + 放置条件） | ✅ | `...\feature\TreeFeature.java` | yarn |
| random_patch placed/configured JSON 实例 | ✅ | `versions\1.20.1\data\worldgen\data\minecraft\worldgen\placed_feature\patch_grass_plain.json`、`crimson_fungi.json`；`configured_feature\patch_grass.json` | 官方数据 |

**找不到/换名清单**：`RandomSelectorFeature` 不在 mc_src_extract（在 `.tmp` mojmap 树找到）；`LeavesVineVineDecorator`/`TrunkVineVineDecorator`/`CountOnEveryLayerPlacementModifier`/`TreeFeature`→yarn 对应 `LeavesVineTreeDecorator`/`TrunkVineTreeDecorator`/`CountMultilayerPlacementModifier`/`TreeFeature`（无独立 TrunkPlacer 类名差异，TrunkPlacer/FoliagePlacer 调用点在 TreeFeature.generate 内，本文引用）。

## 1. RandomSelectorFeature（mojmap 树）

```java
for(WeightedPlacedFeature weightedplacedfeature : randomfeatureconfiguration.features) {
   if (randomsource.nextFloat() < weightedplacedfeature.chance) {
      return weightedplacedfeature.place(worldgenlevel, chunkgenerator, randomsource, blockpos);
   }
}
return randomfeatureconfiguration.defaultFeature.value().place(worldgenlevel, chunkgenerator, randomsource, blockpos);
```

语义要点：
- 遍历 `features`（List，JSON 顺序），每项消费 **1 次 nextFloat**：`nextFloat() < chance` 即选中并**立即 return**（后续项不消费 RNG）。
- 全部落空 → `defaultFeature` 分支：**不额外消费 RNG 做选择**，直接 place（RNG 消费进入内层 feature）。
- RNG 流来源：`FeaturePlaceContext.random()`——与外层 placed modifier 同一个 Random 实例，连续流，无 reseed。
- 内层 `place` 是 **PlacedFeature.place**（mojmap = yarn PlacedFeature.generate，见 §3），即内层还有自己的 placement 链。

## 2. RandomPatchFeature（yarn）

```java
int i = 0;
int j = randomPatchFeatureConfig.xzSpread() + 1;
int k = randomPatchFeatureConfig.ySpread() + 1;
for (int l = 0; l < randomPatchFeatureConfig.tries(); l++) {
   mutable.set(blockPos,
      random.nextInt(j) - random.nextInt(j),   // X 偏移
      random.nextInt(k) - random.nextInt(k),   // Y 偏移
      random.nextInt(j) - random.nextInt(j));  // Z 偏移
   if (config.feature().value().generateUnregistered(world, generator, random, mutable)) i++;
}
return i > 0;
```

语义要点：
- **tries 来自 feature JSON 字段 `tries`**（`RandomPatchFeatureConfig.tries()`），不是 placement 的 Count。
- 每次 try 消费 **6 次 nextInt**，顺序固定：x、x、y、y、z、z（`nextInt(j) - nextInt(j)`，j = spread+1 ⇒ 三角分布，值域 [-spread, +spread]）。
- 每次成功放置（内层 generateUnregistered 返回 true）计数 +1；返回 `i > 0`。
- **内层走 generateUnregistered**：内层 PlacedFeature 的 placement 链执行，但 `FeaturePlacementContext.placedFeature = Optional.empty()` ⇒ **内层不能用 biome modifier**（会 throw，见 §3）——内嵌 placed JSON 只允许 block_predicate_filter 类无注册依赖的 modifier（实例 patch_grass.json 即 `block_predicate_filter air`）。
- RNG 流：与外层同一 Random 连续消费；内层 modifier（如 in_square）会继续从同一流抽。

## 3. PlacedFeature.generate 与 modifier 链

```java
private boolean generate(FeaturePlacementContext context, Random random, BlockPos pos) {
   Stream<BlockPos> stream = Stream.of(pos);
   for (PlacementModifier m : this.placementModifiers)
      stream = stream.flatMap(posx -> m.getPositions(context, random, posx));
   stream.forEach(placedPos -> { if (cf.generate(world, generator, random, placedPos)) ok.setTrue(); });
   return ok.isTrue();
}
```

- modifier **按 JSON 顺序** flatMap；**惰性流**——forEach 逐位置触发下游 generate 时，modifier 内的 RNG 抽取按「流被消费」的顺序发生（非全部预生成后统一抽）。这是 RNG 消费顺序对拍的关键点：Rust 复刻不能把所有位置先算完再放块，需与 forEach 的逐位置语义一致。
- 各 modifier RNG 消费（同注释 §2）：
  - `CountPlacementModifier`（AbstractCount）：`IntStream.range(0, count.get(random))` ⇒ **先抽 1 次 count**，再复制 pos N 份（不额外抽）。
  - `SquarePlacementModifier`（in_square）：`nextInt(16)+x, nextInt(16)+z` ⇒ **2 次 nextInt，先 x 后 z**。
  - `HeightmapPlacementModifier`：0 次 RNG（查 heightmap；k<=bottomY 时输出空流截断）。
  - `BiomePlacementModifier`（biome）：0 次 RNG；要求 context 有 placedFeature（Optional.empty 即抛异常）。
  - `CountMultilayerPlacementModifier`（count_on_every_layer，@Deprecated 但 1.20.1 数据仍在用）：do-while 层循环 `i=0..`，每层 `count.get(random)` 次（**先抽 count**）→ `nextInt(16)+x, nextInt(16)+z`（先 x 后 z）→ heightmap MOT_BLOCKING → `findPos` 向下扫第 `i` 层可放置面（0 次 RNG），命中则输出 pos；有任何命中则 `i++` 继续下一层，整层无命中终止。
- **random_patch 的 tries 计数裁决**：来自 **feature JSON `tries` 字段**（§2），与 placed JSON 的 `count`/`count_on_every_layer` 无关。patch 的 placed JSON 典型链（patch_grass_plain：noise_threshold_count → in_square → heightmap(WORLD_SURFACE_WG) → biome）作用于「外层 patch 的锚点数」；patch 内层嵌套 placed 的 placement（block_predicate_filter）作用于每次 try 的偏移点。

## 4. R-1 取证：vine decorator

- **收集容器**：TreeFeature.generate 建 4 个 `HashSet<BlockPos>`：set=logs(root)/set2=logs(trunk)/set3=leaves/set4=decoration（L122-125）。generate 成功后（`bl && (!set2.isEmpty() || !set3.isEmpty())`）构造 `TreeDecorator.Generator(world, biConsumer3(放 vine 计入 set4), random, set2, set3, set)`（L153），`decorators.forEach(d -> d.generate(generator))`（L154，JSON 顺序）。
- **Generator 构造器排序**（TreeDecorator.java L47-52）：`Set` → `ObjectArrayList` 拷贝后 **`sort(Comparator.comparingInt(Vec3i::getY))`**（log/leaves/root 各自）——**只按 Y 升序，同 Y 内顺序 = HashSet 迭代序（不稳定、非坐标序）**。稳定对拍需注意：同 Y 顺序依赖 Java HashSet 桶序（与 BlockPos hash 实现绑定）。
- **LeavesVineTreeDecorator**：per 叶子位置 4 个方向 west/east/north/south，每个方向先 `nextFloat() < probability`（**每方向 1 次 nextFloat，固定 4 次/叶**，即使前面方向未命中也继续抽）→ 命中且 `generator.isAir(neighbor)` 才 `placeVines`：放 vine 后向下最多 4 格，每格 `isAir` 才继续（下降循环本身 0 次 RNG）。isAir = `testBlockState(isAir)`。
- **TrunkVineTreeDecorator**：per 原木位置 4 方向，每方向 `nextInt(3) > 0`（**每方向 1 次 nextInt(3)，固定 4 次/原木**，概率 2/3）→ isAir 才 `replaceWithVine`（不向下延伸）。
- **RNG 来源**：`generator.getRandom()` = **TreeFeature 的同一个 Random**（trunk/foliage 消费完后继续），连续流，无 reseed。
- 调用链上游：TreeFeature.generate（yarn L66-82）= trunkPlacer.getHeight → foliagePlacer.getRandomHeight → getRandomRadius → (rootPlacer?) → trunkPlacer.generate → foliagePlacer.generate；返回 true 后才跑 decorators（即 vine 在树体块全部放置之后）。

## 5. 放置条件（isAir / isReplaceable 类）判断点（简要）

- `TreeFeature.isAirOrLeaves`（L45-47）：`isAir || BlockTags.LEAVES`——叶/干放置常规守卫。
- `TreeFeature.canReplace`（L53-55）：`isAir || BlockTags.REPLACEABLE_BY_TREES`（1.20.1 的 "replaceable" tag 判定，非泛用 isReplaceable 属性）——树放置可替换判定。
- vine decorator 的判断点是 `TreeDecorator.Generator.isAir`（严格 `AbstractBlockState.isAir`）。
- patch 内嵌 placed 常用 `block_predicate_filter(matching_blocks air)`（JSON 数据层守卫，见 patch_grass.json）。
- CountMultilayer.findPos 用 `blocksSpawn = isAir || WATER || LAVA`（数据层，不属树）。

## 6. Rust 侧对照定位（只定位，不判定）

- `worldgen-core\src\feature_loader.rs` L308-329：`random_selector` 占位（自标 idk-7，逐项 nextFloat<chance 即选即返 + default——**与 Java 一致形态**，待 worker 裁决）；L330-346：`random_patch`/`flower` 占位（tries 循环在，**偏移合成未落地**（L340 `let _ = (sx,sy,sz)`），且 Rust 现用 `IntProv::get` 3 次而非 Java 的 6 次 nextInt(±) 差分形态——**消费次数不同**，仅记录事实）；L365-380：generate_nested 占位（未接线）。
- `worldgen-core\src\tree.rs` L647-689：`RandomSelectorConfig`（L650-668，features vec<(f32, PlacedFeature)> + default）/ `RandomPatchConfig`（L673-688，tries: i32, xz_spread/y_spread: IntProv, feature: PlacedFeature）。parse 与 JSON 字段对齐（tries/xz_spread/y_spread/feature/default_feature/features.chance）。

## 7. RNG 消费序速查（本次取证核心输出）

```
外层 placed 链（按 JSON 顺序逐 modifier）→ 选中位置
→ configured.generate:
   random_selector: [nextFloat × 已跳过项数] + 内层 placed
   random_patch:    tries × [nextInt(j)×2(x) → nextInt(k)×2(y) → nextInt(j)×2(z)
                             → 内层 placed.generateUnregistered（其链继续同流）]
   tree:            getHeight/Height/Radius(+root offset) → trunk → foliage
                    → 成功后 decorators(JSON 顺序): TrunkVine 原木Y升序 ×4×nextInt(3)；
                      LeavesVine 叶Y升序 ×4×nextFloat(prob)，placeVines 向下≤4 0 RNG
```

---

*置信度：draft（静态 Degraded；所有行号为 yarn/mojmap 源文件实际行号，可直接回读复核）。*
