# b1-feature-blob：特征阶段（feature placement）blob 放置差 —— 候选假设分析

> 状态：**draft**（置信度：draft，验证分层：Degraded=静态审查 + 数据签名核对，无运行时探针——subagent 沙箱无 shell）
> 课题：FULL 存档口径残差 13328 cell / 4×4 chunk @ (200,200)，seed 8576294172403134396（99.1526% 一致）
> 数据源：`.tmp/p2full/cmp_yhist-260904-04.txt`、`.tmp/p2full/cmp_spatial-260904-04.txt`
> 结论取代链：旧「aquifer」解读已被证伪（block id 误读，id 9=dirt 非 water）——本候选属取代后的新 fan-out 分支 .b1

---

## 1. 机制候选

Java 的 dirt/gravel 矿囊（ore_dirt/ore_gravel，OreFeature）与 disk_sand/gravel/clay/grass（DiskFeature）在 C++/Rust 侧的
放置位置/数量/随机序列与 Java 分歧 → 非 Java 侧多放/错放 dirt/sand/gravel blob。

### 三个可区分的子机制（b1a/b1b/b1c）

**b1a —— BiomeFilter 缺失 + 单 biome 枚举（结构性多放，方向不对称）**
- C++ `BiomePlacementModifier` 是**直通**（不过滤）：`versions\1.20.1\cpp\worldgen\src\placement.h:220-231`
  （注释原文「简化：直接返回（biome 过滤由调用方预判）」）；Rust 同样直通：`WorldgenRust\src\placement.rs:173-178`。
- Java `BiomeFilter` 对**每个 blob 原点**判定位置 biome 是否含该 feature，不含则丢弃。
- 且 C++ 特征枚举只用**当前 chunk 单 biome**（`versions\1.20.1\cpp\worldgen\src\worldgen_api.cpp:1585-1600`，
  注释自认「简化（Phase 3）：set = 当前 chunk biome（Java 是 3×3 chunk 所有 biome section）」）。
- 推论：biome 边界附近，非 Java 侧会放置 Java 已过滤掉的 blob → **单向多放**（ref stone→cpp dirt 为主，反向少）。
  这与残差签名（stone→dirt 4165 主导、sand→dirt 反向仅 259）**方向不对称性吻合**。纯 RNG desync 应产生双向近似对称的置换对。

**b1b —— intSet/p 索引与 decorator seed 序列分歧（RNG desync）**
- `worldgen_api.cpp:1596-1609`：setDecoratorSeed(populationSeed, p, k)，p 来自 `intSetFor(curBiome, k)`。
- Java 的 intSet 由 3×3 chunk 全部 biome section 的 features 并集计算（PlacedFeatureIndexer）；
  只要 3×3 内 biome 与 chunk 中心不同，**p 索引集合不同 → 每个 feature 的 seed 不同 → 全部 blob 位置漂移**（双向置换）。
- 另 `feature.h:8` 注释明确 OreFeature 用 `Math.sin/Math.cos（标准库，非查表）`——若 C++/Rust 用了查表或不同 libm，blob 形状/位置漂移（b1b 变体）。

**b1c —— disk 系（sand 残差来源候选）**
- disk_sand/gravel/clay/grass：`placed_feature/disk_*.json` 全部走 `heightmap OCEAN_FLOOR_WG` + `matching_fluids: water` 过滤。
- C++ HeightmapPlacementModifier 有**已知 +1 语义偏差记录**（`placement.h:208-211`：C++ 高度图存 y、Java 存 y+1，
  实测改 +1 使 300515 降 0.12%、disk/spring 变差——当时拍板保持内部一致）。disk 原点 y 差 1 会改变 target 判定命中，
  造成砂/粘土 disk 的错放（gravel→sand / sand→dirt 类置换的部分来源）。
- `BlockPredicateFilter`（matching_fluids）依赖 `ctx.blockAt`（`placement.h:249-268`）；null 时直通——若原点读块失败会多放。

## 2. 证据（文件:行号）

### 2.1 实现与调用点
| 侧 | 位置 | 内容 |
|---|---|---|
| C++ | `versions\1.20.1\cpp\worldgen\src\feature.h:126-278` | OreFeature.generate/generateVeinPart/shouldPlace/shouldNotDiscard |
| C++ | `feature.h:280-310` | ScatteredOreFeature |
| C++ | `feature.h:420-460` | DiskFeature.generate + targetMatches |
| C++ | `feature_loader.h:22-49,79-81` | 分发 ore/scattered_ore/disk/spring/freeze_top_layer/underwater_magma |
| C++ | `worldgen_api.cpp:1584-1671` | FEATURES 阶段主循环：populationSeed→intSetFor→setDecoratorSeed→PlacedFeature.generate→generateConfigured |
| C++ | `worldgen_api.cpp:1660-1661` | **树花植被（tree/random_patch 等）范围外移除**——若 Java 植被 feature 在 ore 步之前消费 RNG 会级联 desync（注意 Java 步序 vegetal_decoration 最后，对 ore 影响有限，但 step 0-5 内未实现 feature 需逐一核对） |
| C++ | `worldgen_api.cpp:1690-1740` | placed_feature JSON 懒加载（modifiers 解析齐全：count/rarity/in_square/height_range/heightmap/biome/random_offset/block_predicate_filter） |
| Rust | `WorldgenRust\src\feature.rs:210-365`（OreFeature）、`436-470`（DiskFeature） | 与 C++ 同构 |
| Rust | `WorldgenRust\src\feature_loader.rs:152-200` | placed_feature/configured_feature JSON 懒加载 |
| Rust | `WorldgenRust\src\placement.rs:147-208` | modifier 链执行（Biome 直通 L173-178） |
| Rust | `WorldgenRust\src\worldgen_handle.rs:871-901` | apply_features 调用 |

### 2.2 JSON 数据源（两侧行为的共同输入）
| 文件（`versions\1.20.1\data\worldgen\data\minecraft\worldgen\`） | 关键参数 |
|---|---|
| `placed_feature\ore_dirt.json` | **count 7/chunk**，in_square，height_range **uniform absolute 0..160**，biome |
| `configured_feature\ore_dirt.json` | **size 33**，discard_on_air 0.0，target=tag `base_stone_overworld` → **dirt** |
| `placed_feature\ore_gravel.json` | **count 14/chunk**，height_range **uniform above_bottom 0 .. below_top 0 = y -64..319 全量程** |
| `configured_feature\ore_gravel.json` | size 33，base_stone_overworld → **gravel** |
| `placed_feature\disk_sand.json`（及 disk_gravel/clay/grass） | count 3（sand），heightmap **OCEAN_FLOOR_WG**，matching_fluids water |
| `configured_feature\disk_sand.json` | half_height 2，radius uniform 2..6，target=dirt/grass_block（**底层 sandstone rule**：上方 air 时给 sandstone） |

C++ YOffset/HeightProvider 解析（`carver.h:74-116`）对 absolute/above_bottom/below_top 三种语义实现正确
（below_top = minY+height-1-value）；Rust 同（`WorldgenRust\src\carver.rs:60-100`）。
⚠️ 但 `worldgen_api.cpp:267-268` 存在**第二个** YOffset 求值实现，below_top 缺 `-1`（minY+height-value=320，出界 1 层）——
需核对该副本的调用方是否涉及 feature（疑似 carver/surface 用，影响面待查）。

### 2.3 C++/Rust feature 清单覆盖核对
两侧行为同构且均从 JSON 驱动（数据驱动架构铁律合规）。对 dirt/gravel/sand 残差相关的 feature：
ore_dirt / ore_gravel / disk_sand / disk_gravel / disk_clay / disk_grass 全部有 JSON 且两侧解析/分发路径存在。
**未见「清单缺失」型分歧**；分歧必然在 placement 执行语义（biome 过滤/高度/seed 序列），不在数据覆盖。

## 3. y 分布自洽性核算（对照 cmp_yhist-260904-04.txt）

| 证据 | 与 .b1 的自洽性 |
|---|---|
| 残差全 y 均匀（-64..319 每 y≈35，32-band 每带 1041-1210） | ✅ ore_gravel count 14/chunk × **全量程 uniform（-64..319）** 天然产生全 y 均匀的 gravel blob；位置 desync/多放后残差全 y 铺开 |
| stone→dirt 在 mid（-32..199）n=2505 | ✅ 落在 ore_dirt 的 0..160 范围内（199 边缘含 vein 纵向展宽，量级合理） |
| **stone→dirt deep(<-32) n=298、surface(200+) n=1362** | ❌ **ore_dirt 放置域 0..160 无法解释**——y<-32 与 y>200 的 cpp 侧 dirt 不能来自正确实现的 ore_dirt（vein 纵向展宽仅数格，够不到 ±100 格）。同样 spatial 数据 chunk(13,13) y -64..-55、chunk(13,14) top-comp y-range=-64..191 内含 stone→dirt |
| gravel→sand 1687 / gravel→dirt 1359（ref gravel） | ✅ ref 侧 gravel=ore_gravel（全量程），cpp 侧该位置被 dirt/sand blob 占据 → 双 blob 序列分歧的交叠产物 |
| water→air 339 | ⚠️ 与 blob 无关倾向（aquifer/carver 域），属其他候选（.bN） |
| sand→dirt 反向 259 | ⚠️ ref 侧 sand（surface rule/disk 产物）被 cpp dirt 覆盖——若 cpp ore_dirt target 判定正确（base_stone_overworld 不含 sand），此处 cpp 的 dirt 应另有来源 → 与上一条「出界 dirt」同族异常 |

**要点：y 分布「部分自洽」**——gravel 全量程解释了 y 均匀性的主体，但**出界 dirt（约 1660+ cell，占残差 ~12%）是 .b1 纯机制解释不了的硬缺口**，
指向：① 某处 height provider 语义误用（如 worldgen_api.cpp:267-268 那个缺 -1 的第二实现被 feature 路径引用）；
② dirt 来自非 feature 阶段（surface rule 高度差/carver 回填）；③ 比对引擎侧（C++ vs Rust 未在数据文件中区分）存在另一条 dirt 写入路径。此缺口须用探针裁决，不得在 .b1 内含糊带过。

## 4. 量级核算

- 残差密度：13328 cell / 16 chunk ≈ **833 cell/chunk**（题面 52 cell/chunk 是按 256 chunk 折算的全局均值；本对比仅 16 chunk，实际局部密度 833/chunk）。
- Java 每 chunk 矿囊期望替换量（量级）：
  - ore_dirt：7 blob × size 33 = 231 次放置尝试；vein 部分越界/落空（target 未命中 base_stone、越 chunk），经验命中 ~30-60% → **~70-140 dirt cell/chunk**
  - ore_gravel：14 blob × 33 ≈ 462 次尝试 → **~140-280 gravel cell/chunk**
  - disk_*：disk_sand 3 次（需 OCEAN_FLOOR+water），尺寸 π·r²·5，r∈2..6 → 每片 ~60-560 cell，但水面 biome 才触发
- 若位置完全 desync，dirt+gravel 可贡献 **~200-400 mismatch cell/chunk**（含 ref 侧 blob 变孤儿 + cpp 侧 blob 多放）。
- 实测 833 cell/chunk（16 chunk 局部）：略高于 dirt+gravel 满失同步上限，需 disk（r 大，单片可达数百 cell，spatial top-comp n=553/478/387 的巨型连通域与 disk/vein 聚簇形态吻合）+ 其他 feature 贡献补足。
- **量级判定：吻合（同一数量级），不排除 .b1，但满失同步假设偏紧，残差可能还混有第二机制**。

spatial 形态佐证：blob 状连通域（最大 615 cell 跨 y -64..191、跨 200+ 层）与 OreFeature vein（数格级）不匹配，
但与「多条 vein + disk 沿同一 desync 的随机路径连成簇」或「单个大 disk」吻合；chunk(12,14) 出现 water→air n=62 的独立 comp（aquifer 域，另立候选）。

## 5. 判别实验建议（供主会话执行；探针/日志指令模板）

subagent 沙箱无 shell，以下由主会话执行（原始输出落 `.investigations/lossless-accel/fanout-residual-260904-04/cmd-output/`，回传解读）：

1. **origin 审计（判 b1 高层 + 出界 dirt）**：现日志只打印 ore_granite/underwater_magma（`worldgen_api.cpp:1613,1649`）。
   临时把 `[ORIGIN]` 过滤扩到 `ore_dirt|ore_gravel|disk_`，`WG_FEATURELOG=1` 跑同 seed/origin 的 block_probe。
   - 判据 A：ore_dirt origin y 是否全部 ∈[0,160]；**任何出界 origin = height provider/第二实现 bug 实锤（裁决 §3 硬缺口）**。
   - 判据 B：统计每 chunk ore_dirt/ore_gravel/disk 实际执行次数 vs JSON count（7/14/3）——数量偏差 = biome 过滤/枚举差实锤。
2. **Java 侧 seed 序列对照（判 b1b）**：Java 探针工程加 decorator log（populationSeed、每 (p,k) 的 setDecoratorSeed 参数 + placedFeature id + origin y），
   与 C++ `[FEATURE]` 行逐行对拍——p 索引/seed/id 列表首个分歧点即 desync 源头。
3. **BiomeFilter A/B（判 b1a）**：临时实现 per-position 过滤（posToBiome(x,y,z) 的 biome feature 列表含 fid 才保留），
   重跑同参数 FULL 对比：残差若显著下降（预期削掉「单向多放」部分）→ b1a 成立。
4. **feature 归零实验（量化 .b1 总贡献）**：仿 `WG_CARVER_SKIP`（`worldgen_api.cpp:1561` 已有先例）加 `WG_FEATURE_SKIP=ore_dirt,ore_gravel,disk_`，
   跳过后残差中 dirt/sand/gravel 置换对的消失量 = feature blob 贡献的直接测量，剩余残差归其他候选。
5. **出界 dirt cell 取样**：从 cmp 原始 diff 重导出 10 个 (y<-32 或 y>200) 的 stone→dirt cell 坐标，在 cpp 侧 `setBlock(state==dirt)` 处加条件日志，
   回溯写入者（ore feature / surface rule / pendingCross）——直接回答「dirt 从哪来」。

## 6. 结论（draft）

- .b1「feature blob 放置差」与残差签名**大体自洽**：量级同阶、y 均匀性主体可由 ore_gravel 全量程解释、置换对方向不对称与 BiomeFilter 直通 + 单 biome 枚举的结构性简化吻合。
- **但存在硬缺口**：y<-32 / y>200 的 stone→dirt（~1660 cell）超出 ore_dirt 放置域，纯 .b1 不能解释——须先跑实验 1（origin 审计）裁决出界 dirt 来源，再定 .b1 的 candidacy 升降。
- 已定位的可疑代码点：`placement.h:220-231` / `placement.rs:173-178`（Biome 直通）、`worldgen_api.cpp:1585`（单 biome 枚举）、`worldgen_api.cpp:267-268`（below_top 缺 -1 的第二 YOffset 实现）、`placement.h:208-211`（heightmap +1 已知偏差）。
- 结论状态：**draft**，不下 candidate/confirmed；验证分层 Degraded（静态审查 + 数据签名核对，无运行时探针）。
