# scout-b：1.21.6 features 数据包 vs worldgen-core 差距清单

- 日期/来源：mc-1216-features-takeover，scout 勘探（只读），status: **draft**
- 方法：PowerShell+Python 静态枚举（rg 对 gitignore 目录会 prune，未使用）；引擎支持面从 feature_loader.rs / placement.rs 静态读出；使用频率 = 64 个 biome JSON features 列表引用计数。
- 数据：configured_feature 224 个、placed_feature 258 个、biome 64 个。
- 验证分层：**Degraded（静态审查）**——未运行引擎解析，"已支持"指解析分发表覆盖，语义逐位对齐未验证。

## 1. 频次表

### configured_feature type（224 个，18 类被引用面覆盖）
| type | 数 | 判定 |
|---|---|---|
| minecraft:tree | 39 | ✅ |
| minecraft:ore | 30 | ✅ |
| minecraft:random_patch | 24 | ✅ |
| minecraft:random_selector | 21 | ✅ |
| minecraft:flower | 9 | ✅ |
| minecraft:vegetation_patch | 6 | ❌ |
| minecraft:nether_forest_vegetation | 6 | ⚠️ 误捕 |
| minecraft:spring_feature | 6 | ✅ |
| minecraft:disk | 5 | ✅ |
| minecraft:fallen_tree | 5 | ✅ |
| minecraft:block_pile | 5 | ❌ |
| minecraft:huge_fungus | 4 | ❌ |
| minecraft:simple_random_selector | 4 | ❌ |
| minecraft:simple_block | 4 | ✅ |
| minecraft:seagrass / bamboo / netherrack_replace_blobs / block_column / end_gateway / fossil / multiface_growth / iceberg / basalt_columns / random_boolean_selector / sculk_patch / twisting_vines | 各 2 | ❌ |
| 单例 12 类 | 各 1 | ❌（geode / basalt_pillar / blue_ice / bonus_chest / chorus_plant / waterlogged_vegetation_patch / delta_feature / desert_well / dripstone_cluster / end_island / end_platform / end_spike / glowstone_blob / huge_brown_mushroom / huge_red_mushroom / ice_spike / kelp / lake / large_dripstone / monster_room / root_system / sea_pickle / vines / void_start_platform / weeping_vines） |
| minecraft:scattered_ore | 2 | ✅ |
| minecraft:forest_rock | 1 | ⚠️ 误捕 |
| minecraft:freeze_top_layer / underwater_magma | 各 1 | ✅ |

**统计：✅ 154 / ⚠️ 误捕 2 类（7 个 JSON）/ ❌ 70 个 JSON（34 类）**

### placed_feature modifier 链 type（258 链，各元素频次）
✅：biome 204、in_square 191、count 126、heightmap 108、height_range 82、block_predicate_filter 58、rarity_filter 54、surface_water_depth_filter 25、random_offset 10、environment_scan 9、noise_based_count 4、surface_relative_threshold_filter 3
⚠️：count_on_every_layer 8 —— 被 `contains("count")` 吸进 count 分支（placement.rs count 臂只排 noise），**语义不同**（Java CountOnEveryLayerPlacement ≠ CountPlacement），静默语义腐蚀（#8 家族签名）
❌：noise_threshold_count 6（flower_cherry/flower_plains/patch_grass_meadow/patch_grass_plain/patch_tall_grass_2/wildflowers_meadow）、fixed_placement 1（end_platform）

**整链判定：251/258 全支持；7 链含❌ modifier；8 链含⚠️ count_on_every_layer 误配。**

## 2. 三分清单汇总

- **✅ 已支持**：上表 ✅ 项。注意"支持"= 分发臂覆盖；tree/patch/selector 内部字段（placer/size/foliage 等）对齐未在本勘探验证。
- **⚠️ 解析降级**（进 catch-all 或误分支）：
  1. `contains("ore")` 误捕 `nether_forest_vegetation`（6）与 `forest_rock`（1）→ 走 OreFeatureConfig 解析，配置全错，生成面必歪（substring 分发的固有缺陷）。
  2. `count_on_every_layer` → count 分支（8 链）。
  3. unknown configured type → eprintln 告警 + 空实体（feature_loader.rs:83），unknown placement modifier → eprintln（placement.rs:450）：70 个❌ JSON 全部静默降级为"可加载不生成"。
- **❌ Unsupported**（70/224 = 31%，按 biome 引用频次排序的 top）：monster_room(108)、lake_lava(106)、amethyst_geode(54)、glow_lichen(54)、glowstone_extra(10)、kelp(6)、forest_flowers(5)、vines(4)、seagrass 族(7)、fossil(6)、iceberg 族(4)、blue_ice(2)、bamboo(2)、vegetation_patch 族(4)……其余见上表。
  - **overworld 影响面：biome 引用的 configured placement 总计 2642 次，命中 ⚠️+❌ = 408 次（15.4%）**；❌ 内 top4（monster_room/lake/geode/glow_lichen）合计 322 次，是接管后可见差异的主来源。

## 3. placed→configured 引用完整性

- 三形态分布（1.21.6）：**id 字符串 258 / 内联对象 0 / 缺失 0**——发现 #16 的三形态在 1.21.6 顶层 placed 全部是字符串形态；引擎 260905-12 已支持三形态，无缺口。
- 死链：**0**（258 个引用按命名空间剥离后全部存在于 configured_feature 目录）。
- 注意：内联对象形态可能出现在 random_patch/selector **内层**（未在本轮逐个展开计数，引擎有递归补载路径）。

## 4. #87 判据检查（Java 枚举映射 catch-all）

| 映射点 | 引擎实现 | 1.21.6 Java 常量 | 差异 |
|---|---|---|---|
| StructureTerrainAdaptation | beardifier.rs:217 + worldgen_handle.rs:509：ordinal 1-4 精确映射，`_ => TerrainAdaptation::None` catch-all | NONE/BURY/BEARD_THIN/BEARD_BOX/ENCAPSULATE（5 个，StructureTerrainAdaptation.java） | **无缺失**（ENCAPSULATE=4 已接，1.21.6 与 1.20.1 常量集相同）；catch-all 仍在但当前不可达 |
| Heightmap.Types | placement.rs:311/369：`contains("OCEAN_FLOOR") ? ocean_floor : world_surface` **两桶塌缩** | 7 个常量（WORLD_SURFACE_WG/WORLD_SURFACE/OCEAN_FLOOR_WG/OCEAN_FLOOR/MOTION_BLOCKING/MOTION_BLOCKING_NO_LEAVES/…） | ⚠️ 非枚举映射而是字符串塌缩：MOTION_BLOCKING(_NO_LEAVES) 全部落 world_surface；tree.rs:982 对 MOTION_BLOCKING_NO_LEAVES 做了独立近似（自带 IDK 注释）——接管 features 前需逐 type 审 |
| GenerationStep.Feature（decoration step） | 无硬编码枚举——step 数与列表从 biome JSON features 数据驱动（feature_loader.rs PlacedFeatureIndexer） | 11 个常量（1.21.6 与 1.20.1 同集） | 无 catch-all 面，**无升级风险** |
| tree anchor（placement Y 锚） | biome modifier 已 Fix-2（jitter 采样），anchor_biome 近似已删 | — | 无 `_ => None` 残留 |

结论：#87 判据下**当前唯一活口风险 = Heightmap 两桶塌缩**；TerrainAdaptation catch-all 形存实无。

## 5. 解析面风险注记（静态）

1. **substring 分发**（feature_loader.rs:52-60 contains 系列）：任何未来 type 名含 "ore"/"disk"/"spring" 子串即误捕（已实证 2 例）——1.21.6 接管建议改精确匹配。
2. **catch-all 静默降级**：❌70 JSON 可加载、eprintln 一次、生成恒 false——S4 全量加载门会"绿"，必须以「unknown type 计数=0」为验收判据。
3. **count_on_every_layer 误配**：非告警路径，完全静默——比❌更危险（#8 家族签名：解析成功但语义错）。
4. **feature JSON 结构形态**：1.21.6 placed 顶层全部 id 字符串形态；配置内层（patch 的 tries/xz_spread 等）字段形态未逐文件 diff 1.20.1，接管批次 B 应先跑字段形态 diff。
5. biome JSON features 列表数据驱动，11 step 无枚举硬编码——**调度层升级风险低，风险集中在 configured type 实装面**。

## 排批次建议（供 worker/judge 参考，draft）

- 批次 A（低垂，覆盖 top 差异）：monster_room、lake、geode、glow_lichen（合计 322/408 = 79% 的❌引用量）+ noise_threshold_count modifier（6 花田链）。
- 批次 B：count_on_every_layer 修正（⚠️ 转正确实现）+ contains→精确匹配改造 + seagrass/kelp/vines 等水下植被族。
- 批次 C：nether/end 专用族（huge_fungus/basalt_columns/delta/chorus/end_* 等），overworld 无引用，优先级最低。

—— scout-b，产物即本文件；枚举脚本：.tmp/scout-b-enumerate.py / scout-b-usage.py / scout-b-dead.py
