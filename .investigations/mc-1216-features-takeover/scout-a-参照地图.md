# scout-A：1.20.1 / 共享引擎 Rust features 执行管线参照地图

> 角色：recode-scout（只读勘探）。任务：为 1.21.6 features 阶段从 Java vanilla 切换到 Rust 执行提供 1.20.1/worldgen-core（包名 WorldgenRust）参照实现地图。
> 置信度：candidate（静态勘探，所有引用为文件:行号一手核对；未做运行时验证——本勘探阶段降级声明：Degraded 静态审查）。
> 日期标签：未涉及（产物无日期命名）。

## 1. features 阶段执行入口链

| 环节 | 位置 | 说明 |
|---|---|---|
| C API 入口 | `worldgen-core/src/api.rs:131` `wg_fill_blocks_multi` | 批量块级管线入口；句柄来自 `wg_create`（api.rs:60）→ `WorldgenHandle::create/create_for_dim`（worldgen_handle.rs:159/176） |
| 单 chunk 入口 | `worldgen-core/src/worldgen_handle.rs:596` `fill_chunk_blocks(cx,cz)` | 流程注释 L594：fill_chunk(宏观)→BlockColumn→build_surface→carver→features；返回 16*16*height vanilla raw id |
| SKIP_FEATURES 判定 | `worldgen_handle.rs:609` | `flags & FLAG_SKIP_FEATURES != 0 \|\| env WG_SKIP_FEATURES`；`FLAG_SKIP_FEATURES = 1<<1`（L147）；flags 为 AtomicU32（L128 注释：bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE；0=未设置→回落 env）。JNI 侧 `wg_set_flags`（api.rs:113；1.21.6 壳 `versions/1.21.6/rust/src/jni_bridge.rs:109-120`，注释「Java CppBridge 在 init/initNether 后设 flag 关 Rust carver/features——双跑修复 2026-09-08」）。Java 侧 grep `stageMask` 在 versions/1.21.6 下**无命中**（该标识符不在仓库内，可能在 Java mod 源/外部） |
| features 调用 | `worldgen_handle.rs:610-611` → `apply_features(&mut col, cx, cz, &heightmap, &biome_at)`（定义 L961） | 注释 L957-959：装饰层，对齐 C++ applyCarversAndFeatures FEATURES 部分（worldgen_api.cpp L1584-1674）；structure 部分跳过 |
| biome/step 分发 | `apply_features` 内 L1030-1082 | 见下 |

按 biome/step 分发机制（apply_features，worldgen_handle.rs）：
- **biome 并集**（L1030-1045，Fix-1 .b2a）：3×3 邻域 chunk × 4×4 水平格 × Y∈{0,64,128} 切片近似 Java「全高 biome 容器并集」（ChunkGenerator.java:346-353；Y 全高并集以切片近似，IDK-1：洞窟 biome 地表 step feature 可能漏，L1032）。
- **feature entry**（L1046-1052）：每个 biome 的 per-step feature 表 `biomesrc.bc.features_for(b)`（不去重 entry 本身，对齐 Java :381）。
- **PlacedFeatureIndexer**（feature_loader.rs:96-208；预构建于 worldgen_handle.rs:449-453，从 `bc.all_features_lists()`）：featureIndex=首现递增（L119-129）；相邻 feature 建边 → (step,featureIndex) 比较器 DFS 拓扑 → 后序反转 → 按 step 分组（L134-186，260905-08 精确移植 Java collectIndexedFeatures——首现序与 Java p 不一致已实证，L113-116）；`lastIndexGetter`：last_index_map[step][fid]（L187-193）。
- **step 循环**（L1064-1082）：k=0..max_step；intSet = ∪ 各 entry 的 int_set_for(e,k)（feature_loader.rs:197-208，排序去重，Java :394-395）；p∈intSet → `fid = indexer.step_features[k][p]`（L1076-1078）；每个 (k,p) 先 `feat_random.set_decorator_seed(population_seed, p, k)`（L1079）。
- **两级 cache 查询**：placed = `feature_cache.placed.get(fid)`（L1084-1093）、configured = `feature_cache.configured.get(pf.configured_feature)`（L1158-1167）；MISS 打 WG_FEATURELOG 并 continue（不炸）。
- **放置执行**（L1094-1182）：FeaturePlacementContext（fctx）+ OreFeatureContext（octx）→ `pf.generate(fctx, feat_random, ...)`（placement.rs:474，深度优先 modifier 链）→ 闭包内 `feature_loader::generate_configured`（L1172-1181）。
- **跨 chunk 写（c-A，默认关）**：`WG_CA_MIN=1` 显式启用（L971）；c-A-write 越界写 → `pending_cross_writes` 缓冲（L990-996），features 前 overlay 本 chunk（L972-984）；性能代价 +68% 故默认关（L968-970 注释）。c-A 关闭时越界读返回 -1 = 保守拒绝（judge C-3 语义偏差，L1100/1117-1120）；开启时越界读走 `neighbor_terrain` 地形列缓存（L1111-1116，缓存只存 noise+surface+carver 无 feature，L938/137-141）。

## 2. 支持面清单

### 2.1 configured feature 类型（feature_loader.rs:34-86 parse + 374-474 generate_configured，catch-all 全列）
已支持：`*ore*`（contains；scattered_ore 分流 ScatteredOreFeature，L384-390）｜`*disk*`｜`*spring*`｜`*underwater_magma*`｜`*freeze_top_layer*`｜`minecraft:tree`（精确匹配，防 azalea_tree 误伤；azalea_tree 数据 type 也是 minecraft:tree 走同分支，L62-65）｜`minecraft:random_selector`｜`minecraft:random_patch`/`minecraft:flower`｜`minecraft:simple_block`｜`minecraft:fallen_tree`（B5 260908-15）。
**catch-all**：parse 未知 type → `eprintln!("[feature-loader] unknown configured feature type: ...")` 静默丢弃不 panic（L81-84）；generate 未知 type → 返回 false 无日志（L471-473）。generate 侧配置缺失（如 tree without config）→ 告警+false（L400-403/467-470）。
**未支持即不可能出现在 1.20.1 数据中的其余 vanilla type**（no_bonemeal/vein 类/bamboo/coral/delta/basalt_columns/glow_lichen/multiface_growth/root_system/geode/huge_fungus/huge_brown_mushroom/huge_red_mushroom/monster_room/ore_vein/pointed_dripstone/replace_spring/twisting_vines/weeping_vines/kelp/seagrass/end_* /nether_forest_vegetation 等均走 catch-all）——1.20.1 biome features 实际引用面已由预加载覆盖（见 §4），catch-all 告警在 native/bin-diag 口径验证时为已知告警面。

### 2.2 placement modifier（placement.rs:257-279 enum + 389-452 parse，尾部 catch-all 全列）
已支持：count（IntProvider）｜rarity_filter｜in_square（Square）｜height_range（HeightProvider）｜heightmap（OCEAN_FLOOR*→ocean_floor 表，其余→world_surface，L310-321；off-by-one top+1 已修 L316-318）｜biome（0 RNG 消费，8 邻域 jitter + features_for 允许集判定，L322-334）｜random_offset（xz_spread/y_spread）｜surface_water_depth_filter（L339-350）｜environment_scan（L351-364）｜block_predicate_filter（谓词树，L365-367；BlockPredicate enum placement.rs:145-253：matching_blocks/matching_fluids/has_sturdy_face/solid/all_of/would_survive/inside_world_bounds/always_true——子类型清单见 enum 定义）｜surface_relative_threshold_filter（L368-378）｜noise_based_count（**简化：noise 恒 0.0，noise_name/scale 被丢弃**，L379-385 注释「Phase 3 简化」）。
**catch-all**：parse 未识别 → `eprintln!("unknown placement modifier type")` + None 丢弃（L449-451）。parse 子句用 `contains` 匹配（如 "count" 且非 noise，L391）。

### 2.3 tree.rs 支持面（MC 1.20.1 树族）
- TrunkPlacer（tree.rs:69-95）：straight_trunk_placer｜fancy_trunk_placer(LargeOak)｜mega_jungle_trunk_placer(MegaJungle)；其余 → `Unsupported{type_name}` + 告警（L91-94），generate 时 Unsupported return false（L627）。
- FoliagePlacer（tree.rs:113-121）：blob_foliage_placer｜fancy_foliage_placer｜bush_foliage_placer｜jungle_foliage_placer；Unsupported + 告警（L129 起分支）。
- TreeDecorator（tree.rs:314-368）：cocoa｜trunk_vine｜leave_vine｜beehive（260905-10 补抽）｜place_on_ground（B6 260908-15）｜attached_to_logs（B5 260908-15）；其余 → Unsupported + 告警（L367-368），apply 时消费 0 RNG 并打 WG_TREEDIAG [BEE-MISS] 标记（L484-487）。
- FeatureSize（tree.rs:508 起）：TwoLayers 系（two_layers_feature_size）+ Unsupported 兜底；BlockStateProvider：Simple/Weighted（tree.rs:22-52）。
- 嵌套 feature 配置：RandomSelectorConfig（tree.rs:890）/RandomPatchConfig（917）/SimpleBlockConfig（937）/FallenTreeConfig（1083，stump+log decorators）。

### 2.4 RuleTest / tag 展开（feature.rs）
- RuleTest::test/parse（feature.rs:27-62）；`expand_tag`（L64）走 block_tags JSON，`expand_tag_fallback`（L71）硬编码兜底（260907-04 数据驱动改造，见 §6）。
- should_place 在 generate_vein_part 内逐 target 判定（feature.rs:327-333）。

## 3. RNG 消费机制（冻结顺序生成 = 有）

- 随机源：`ChunkRandom::xoroshiro()`（worldgen_handle.rs:1056，Xoroshiro128PlusPlus；与 carver 的 CHECKED 不同，L1055 注释）。
- **setPopulationSeed**（chunkrandom.rs:163-171）：`setSeed(worldSeed); l=nextLong()|1; m=nextLong()|1; n=(blockX*l+blockZ*m)^worldSeed（wrapping 回绕）; setSeed(n)`。调用点 worldgen_handle.rs:1058，输入 `(self.seed, cx*16, cz*16)`。
- **setDecoratorSeed**（chunkrandom.rs:174-177）：`l = populationSeed + index(p) + 10000*step(k)`，setSeed 直设。每个 (step k, index p) 一条独立种子流（worldgen_handle.rs:1079）；structure 的 decorator seed 独立重置、Rust 跳过 structure 不影响 feature 序（feature_loader.rs:95 注释）。
- **流共享**：同一 feat_random 流贯穿 placed modifier 链（深度优先：位置1 走完全部 modifier → 位置2，placement.rs:471-494；广度优先会致 RNG 消费序错 = granite y 全错的实证注释）→ generate_configured → 树 trunk/foliage/decorators（tree.rs:575 注释「random = feat_random 同一 RNG 流」；随机只经 &mut 参数传递不存字段，tree.rs:4 F-1 教训）。
- **JDK Random 语义**：next_int_bound/next_float/next_double（chunkrandom.rs:126-160，legacy LCG 同款；Xoroshiro 分支同接口）。
- **shuffle 面**：树 decorator 内——attached_to_logs 用 Java Util.copyShuffled Fisher-Yates 降序（tree.rs:1053-1058，消费世界流）；beehive 候选的 Collections.shuffle 用自有 Random **不消费世界流、顺序不确定** → Rust 取确定序首候选（RNG 流不受影响，idk-bee2，tree.rs:458-460）。RandomPatch：每 try 恒 6 次 nextInt 差分三角分布（feature_loader.rs:428-439）。random_selector：逐项 nextFloat()<chance 即选即返，全落空走 default 不抽选择 RNG（feature_loader.rs:407-421）。
- **冻结**：feature_cache/feature_indexer 创建时预载后只读无锁（worldgen_handle.rs:121-124/463-466），运行时顺序确定；多 chunk 并发下跨 chunk 写时序近似为无（IDK-cA1，L137-141）。

## 4. 数据加载面（发现 #26/#27：预加载集合 vs 运行时查询集合）

- 数据根：`<wg_dir>/data/<ns>/worldgen/{placed_feature,configured_feature}/<short>.json`（feature_loader.rs:213-216 data_path；id 无冒号默认 minecraft）。实际数据：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/`（另有 testcontent ns）；placed_feature 231 个、configured_feature 194 个（实测计数）。biome features 列表来自 `biome/*.json` load_features（worldgen_handle.rs:375-379，minecraft + mod ns 两趟）。
- **预加载**（WorldgenHandle::create，worldgen_handle.rs:463-466）：`placed_ids = bc.all_feature_ids()`（= 所有 biome features 引用的 placed 全集）→ `FeatureCache::preload_all`（feature_loader.rs:269-366）：① 逐个加载 placed JSON + 其引用的 configured（L269-299）；② placed.feature 为内嵌对象（Holder.direct）时登记其 configured（L300-315，260905-12 内联对象 19 个 patch_grass 系）；③ **selector 内层 placed 递归补载到不动点**（L318-365）——内层 placed（如 jungle_bush）不在任何 biome features 列表，不补载则运行时 miss。
- **运行时查询集合**：apply_features 按 indexer intSet→fid 查 placed/configured cache（L1084/1158），均只读（cache 不再写，L1169-1171 注释）；generate_nested（feature_loader.rs:483-516）查 nested placed→configured，双 miss 告警（L505/514）。
- **集合差**：预加载 = biome 引用 ∪ selector 递归闭包；运行时查询 = biome 引用经 indexer + selector/patch 内层（已被闭包覆盖）。patch 内联 configured（inline_configured）直发不查 cache（feature_loader.rs:441-444）。generate_nested 死防御分支（内联 placed 直发绕过 placement 链）当前数据不可达（judge WARN 260905-12，L492-496）。
- **懒加载 API**（get_placed/get_configured，L230-265）仍保留但生产路径不触发写。

## 5. biome 过滤与 heightmap 输入来源（全部 Rust 内部，Java 只传 worldSeed/flags/handle）

- **biome 输入**：`biomesrc.biome(&NoisePos)`——Rust 内部 MultiNoise 分类器（biome_params 从 `biome_params.json` 加载，data-driven-boundary.md:12）。三个变体：
  - 无 jitter 直采（worldgen_handle.rs:1016-1019）→ biome 并集 / cur_biome；
  - 8 邻域 jitter（L1021-1025，`biome::biome_pick_cell(self.biome_access_seed,...)` + `<<2` 对齐）→ pos_to_biome / biome_allows（Fix-2 .b2b，L1122-1126：jitter 点 biome 的 features_for 含 fid 才放行）；
  - fctx.biome_at（fill_chunk_blocks 传入，L603-606：`(x>>2)<<2` 对齐采样）。
- **heightmap 输入**：两套都在 apply_features 内 Rust 自算，非 Java 传入：
  - OCEAN_FLOOR_WG：逐列自顶向下扫 col 跳过 air/water/lava 取第一个固体（L998-1014；carver 前构建语义）；
  - WORLD_SURFACE_WG：`fill_terrain_column` 的 build_surface 输出 heightmap（L600 参数传入）。
  - placement 层经 fctx.ocean_floor / fctx.world_surface 消费（placement.rs:310-321/343-349/369-377）；越界列 = min_y-1（placement.rs:315）或直通（L346/373 邻域保留）。
- **block_at 输入**：本 chunk 直读 col 裸指针（L1101-1121，并发不变量 judge C-4）；越界 c-A 关=-1 保守拒、c-A 开=neighbor_terrain 缓存。
- biome_temperature 静态表（surface_rules::biome_temperature，L1026）供 freeze_top_layer。

## 6. 已知缺口 / 语义偏差注记（升级点清单，只列事实）

| # | 项 | 位置 | 现状 |
|---|---|---|---|
| 1 | carver replaceable / feature RuleTest tag 展开 | data-driven-boundary.md:25-31 | **260907-04 已数据驱动**（tags/blocks JSON 170 文件优先，硬编码 fallback 保留为 golden 基准 + 一次性日志）；`minecraft:netherrack` 1.20.1 无 tag 文件走 fallback（L30） |
| 2 | noise_based_count noise 恒 0 | placement.rs:379-385 | noise sampler 未接，count = count.get(random)+0（影响分布密度，不炸） |
| 3 | 越界读保守拒绝 | worldgen_handle.rs:1100/1117-1120 | judge C-3 语义偏差：Java 读邻 chunk 实况；c-A-min 开启时走地形列缓存近似 |
| 4 | 跨 chunk 写默认关 | worldgen_handle.rs:971（WG_CA_MIN） | c-A-write 性能 +68% 未翻默认；IDK-cA2：先于 center 生成的邻 chunk 收不到 overlay（L141） |
| 5 | 3×3 biome 并集 Y 切片近似 | worldgen_handle.rs:1030-1032 | IDK-1：洞窟 biome 地表 step feature 可能漏（Y 只采 {0,64,128}） |
| 6 | 树 decorator 集合序近似 | tree.rs:383/450-460 | 同 Y 内序 = Java HashSet 桶序近似（idk R-1）；beehive 候选极值近似（idk-bee1）+ shuffle 确定序（idk-bee2）；放置点可能偶差、RNG 流不受影响 |
| 7 | unsupported 面 | feature_loader.rs:83/450-451、tree.rs:92/367 | catch-all 全部「告警+丢弃/false」，无静默；WG_TREEDIAG 可标 [BEE-MISS] 漏消费点（tree.rs:484-487） |
| 8 | 1.21.6 壳现状 | versions/1.21.6/rust/src/jni_bridge.rs:109-132 | 已有 setFlags/getFlags JNI（mask 直通 wg_set_flags）；仓库内 grep `stageMask` 无命中（该词不在本仓库，Java 侧调用方在仓库外/mod 源） |
| 9 | flags 原子读 | worldgen_handle.rs:601/128 | flags=0 未设置 → 完整执行（不 skip）；SKIP 语义 = 跳过 Rust 侧执行 |
| 10 | 单 biome features 双 ns 加载 | worldgen_handle.rs:375-379 | minecraft + mod data ns 两趟 load_features；feature id 自带命名空间路径解析（feature_loader.rs:211-216，260907-05） |

## 待深入点（交 worker）
- `all_feature_ids` / `features_for` / `all_features_lists` 的 biome.rs 侧实现（本次未展开，biome.rs）。
- `wg_fill_blocks_multi` 的多 chunk 循环与 flags 传递路径（api.rs:131-175，本次只定位未逐行）。
- Java 侧 stageMask 0b011 的设置点不在本仓库（外部 mod 源），接管方案需主会话确认 Java 侧调用面。
