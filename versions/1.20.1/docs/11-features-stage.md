# 11. FEATURES 阶段（装饰层）Rust 移植

> Rust 把 C++ FEATURES 阶段（feature.h/placement.h/feature_loader.h）移植到 `WorldgenRust/src/{placement,feature,feature_loader}.rs`（提交 6934ea4）。
> Java 参照：`world/gen/feature/*` + `world/gen/placementmodifier/*`。

## 功能目的

FEATURES 阶段（Java `world/gen/feature`）在 NOISE→CARVERS→SURFACE 之后填充方块：矿脉（ore/scattered_ore）、圆盘（disk）、泉水（spring）、顶层冻结（freeze_top_layer）、水下岩浆（underwater_magma）等。Rust 移植实现见 `WorldgenRust/src/{placement,feature,feature_loader}.rs`，对应 C++ `versions/1.20.1/cpp/worldgen/src/{placement,feature,feature_loader}.h`。

## 1.20.1 工作机制（含代码位置）

- **调度链**（`worldgen_handle.rs apply_features`，carver 后调用）：3×3 biome set → `PlacedFeatureIndexer` 全局索引 → `setDecoratorSeed(l,p,k)` → `PlacedFeature.generate`。
- **`placement.rs`**：`IntProvider`（uniform/trapezoid/biased_bottom/weighted_list/clamped）+ 10 个 `PlacementModifier` + `PlacedFeature.generate`。
- **`feature.rs`**：`RuleTest`（tag_match/block_match/random_block_match）、`OreFeatureConfig`/`OreFeatureContext`、`OreFeature`（3D 矿脉 `generateVeinPart`）、`ScatteredOreFeature`、`DiskFeature`、`SpringFeature`、`FreezeTopLayerFeature`、`UnderwaterMagmaFeature`。
- **`feature_loader.rs`**：`ConfiguredFeature` 解析（type 分发到各 config）+ `PlacedFeatureIndexer`（全局 lastIndex p 值）+ `FeatureCache` 懒加载。
- **`biome.rs`**：`load_features`（读 `biome/*.json` 的 `features[step][]`）+ `all_features_lists`。
- **惰性语义关键**：Java Stream flatMap 是**深度优先惰性**（位置1 走完所有 modifier → 位置2…），Rust 必须按序深度展开，否则**随机数消费顺序不同 → height_range y 全错**（granite 位置错）。

## 版本敏感点

- **`setDecoratorSeed(l,p,k)`**：`l = populationSeed + index + 10000*step`，p = feature 在 `features[step]` 的 `lastIndex`（Java `indexMapping`/`Util.lastIndexGetter`），**不是全局 featureIndex**。p 错 → 该 feature 的随机序列全偏（位置对但分布/数量错）。structure 的 `setDecoratorSeed(l,m,k)` 独立重置，不影响 feature 随机序列，Rust 可跳过 structure。
- **`PlacedFeatureIndexer` 必须以所有 biome features（`all_features_lists`）全局构建**，不能只从当前 biome 子集构建（详见 F-3）。
- 冻结温度：`freeze_top_layer` 用 `world_surface` 高度图；温度 -288（实测）>=0 不冻结，故无影响。

## 已验证的坑（见 `.investigations/features-port/features-errors.md`）

- **F-1**：OreFeatureContext 不能持有 `&mut random`（与 PlacedFeature.generate 双重借用）→ random 改为各 generate 函数参数传入。
- **F-2**：嵌套 fn 无法捕获 FnMut → 闭包递归 + Cell 存 placed 标志（嵌套 fn 全显式参数化最干净）。
- **F-3**：Indexer 从单 biome 构建导致 p 值错 → 改全局 `all_features_lists()` 构建。

## 已知限制（未闭合，供后续 session/决策）

- **ore 放置位置与 vanilla 仅 1/13 匹配**：`populationSeed`/`setDecoratorSeed` 随机序列与 Java 不完全一致（待对齐）。这是**未闭合已知问题**，不是结论。下次处理此阶段前先对齐这两个随机序列分量（index/p/step 对照），再判公式精度。
- 树花植被（flower/random_patch/simple_block/tree/random_selector）在范围外（2026-08-10 用户拍板）。
- 邻域 chunk 方块读取简化：只处理当前 chunk 内（Java ChunkSectionCache 惰性生成邻域）。

> 对齐率（95.50%/95.54%）为当前快照，属低价值不写 docs 主内容，排查时参考 `.investigations/features-port/cmd-output/`。
