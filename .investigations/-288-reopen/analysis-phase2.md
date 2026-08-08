# -288 区域 block_probe 差异构成分析（Phase 2）

- 分析对象：seed=-8248318472910187742，区域 -288,-256（4×4 chunk，cx∈[-18,-15], cz∈[-16,-13]）
- 数据源：`m288_run1.txt`（全量 MISMATCH 行）、`m288_pair_counts.txt`、`m288_vanilla_cat.txt`、`m288_chunk_counts.txt`、`m288_natural_rows.txt`
- 基准：vanilla 侧 FULL 状态（含结构/FEATURE 产物）；C++ 侧为 CoreSwap 当前输出
- 分析角色：recode.scout 只读勘探；本报告为解读，不修改代码
- 结论置信度：**[确定]** = 可由计数/坐标直接证明的统计事实；**[推测]** = 机制归因，需后续 density/feature 对比验证

---

## 0. 一页摘要

| 结论 | 置信度 |
|---|---|
| 8/8「结构/FEATURE 假 diff 为主」**不成立**（量化反驳，见 §1） | 确定 |
| 差异最大来源是 **OreFeature 缺失（浅层岩脉 + tuff + 矿脉 + gravel blob）**，约 3.75 万块 ≈ 56% | 确定（数量）/ 推测（机制） |
| **海域/河流海底边界差异**（water↔stone/dirt/sand 双向）约 1.3 万块 ≈ 19%，且 C++ 海底系统性偏低 | 确定（数量）/ 推测（机制） |
| **洞穴差异**（C++ 实心 vs vanilla 空气，多为深板岩层洞穴）6428 块 ≈ 9.6% | 确定（数量）/ 推测（机制） |
| 真正结构（村庄 + 沉船）约 2000-2500 块 ≈ 3%，**不是差异主体** | 确定 |
| 「C++ 的 density/surface 核心在负坐标已对齐」**部分成立、被夸大**：主体一致（95.7% 匹配 + 2 个 chunk 100% 匹配），但海底密度/表面规则、含水层、洞穴仍为系统性真差异 | 确定 |
| 区域为 4 生物群系混合：cold_ocean（主体）+ river + beach + plains；村庄在 plains、沉船在 cold_ocean | 确定 |

---

## 1. 结案判定：8/8「结构/FEATURE 假 diff 为主」是否成立？

### 1.1 统计事实（确定）

67042 个差异块按 vanilla 侧块名归类（`m288_vanilla_cat.txt`，脚本 `analyze_m288.py` 的分类规则）：

| vanilla 侧类别 | 块数 | 占比 | 含义 |
|---|---|---|---|
| natural | 55113 | 82.21% | 石头/深板岩/水/泥土/沙/砾石等**天然地形块**（脚本判定 C++ 应生成） |
| air | 6428 | 9.59% | vanilla 是空气（多为洞穴空洞） |
| structure_feature | 5296 | 7.90% | 脚本关键字命中（_ore/木制件/草/kelp 等） |
| unknown | 205 | 0.31% | 村庄器具 + 紫晶洞 + 发光苔藓（详见 §4） |

**判定：** 若「结构/FEATURE 为主」指 block 数量占比，则 **7.90% + 0.31% ≈ 8.2% 远称不上「为主」**。82.2% 的差异发生在 C++ 本应生成的自然块上。

### 1.2 关键纠偏：脚本的 structure_feature 类别被「矿脉」污染（确定）

脚本把 `*_ore` 一律归入 structure_feature（`STRUCTURE_KEYWORDS` 含 `"_ore"`），但 MC 中矿脉是 **OreFeature（天然生成）**，不是结构。structure_feature 5296 块中：
- 天然矿脉（coal/iron/copper/gold/lapis/redstone/diamond 及 deepslate 变体）≈ **2900 块**；
- 真正的结构/植被（村庄木制件、农田、草、kelp、seagrass、cobblestone、紫晶洞、发光苔藓）≈ **2400 块**。

因此「结构假 diff」的真实体量约为 2400/67042 ≈ **3.6%**，即使把矿脉也算「C++ 不做的 FEATURE」，最多 8.2%，仍非主体。

### 1.3 若按「C++ 明确不做的东西」口径重估（推测）

8/8 的措辞可能指「差异主要由 C++ 设计上不做的 FEATURE 层构成」。若项目范围明确排除 **OreFeature（浅层岩脉+矿脉+gravel blob）**，则这部分 ≈ 3.75 万块（56%）可视为「范围外」；再加结构/植被 ≈ 5500（8%），合计约 64% 落在「范围外」。**即使按最宽松口径，仍有约 2.2 万块（33%）属于 density/surface 层面的真实差异**（水体边界 1.3 万 + 洞穴 0.64 万 + 表面规则 0.25 万），不能被「假 diff」覆盖。

### 1.4 结论

- **[确定]** 8/8「结构/FEATURE 假 diff 为主」**不成立**——最大单类（48.9%）是浅层岩脉/tuff（OreFeature 石质变体），其次（18.9%）是水体边界，两者合计已超三分之二且都发生在天然块上。
- **[确定]** 「C++ 的 density/surface 核心在负坐标已对齐」**部分成立**：匹配率 95.74%、(-18,-16)/(-17,-16) 两 chunk 0 差异、主体 stone/deepslate/water 分布一致，说明核心框架大体工作。
- **[确定]** 但「已对齐」被夸大：海底面（cold_ocean/river）、含水层、洞穴这三大块仍是系统性真差异（约 33%），必须修复或明确标注为范围外，否则 95.7% 的匹配率无法继续爬升。

---

## 2. 差异模式分类（数量排序 + 机制推测）

以下数量来自 `m288_pair_counts.txt`（got→vanilla 方向，名字为 blocks.json 反查）。

### 表 2-1：Top 25 (got, vanilla) 组合

| # | got | vanilla | 块数 | 归类 |
|---|---|---|---|---|
| 1 | stone | andesite | 8243 | 浅层岩脉 |
| 2 | deepslate | tuff | 8226 | 深板岩 tuff 区域 |
| 3 | stone | granite | 7016 | 浅层岩脉 |
| 4 | stone | diorite | 6614 | 浅层岩脉 |
| 5 | deepslate | air | 5411 | 洞穴空洞 |
| 6 | stone | water | 4416 | 含水层（vanilla 水） |
| 7 | water | stone | 3117 | 海底边界（C++ 水） |
| 8 | water | dirt | 2539 | 海底边界（C++ 水） |
| 9 | stone | gravel | 2135 | 海底表面 |
| 10 | stone | dirt | 2119 | 表面规则/海岸 |
| 11 | deepslate | gravel | 1802 | 深板岩 gravel blob/洞穴填充 |
| 12 | deepslate | diorite | 1131 | 浅层岩脉（深板岩段） |
| 13 | deepslate | granite | 1026 | 浅层岩脉（深板岩段） |
| 14 | stone | air | 775 | 洞穴/表面 |
| 15 | gravel | stone | 746 | 海底表面 |
| 16 | stone | coal_ore | 742 | 矿脉 |
| 17 | water | sand | 723 | 海底/海滩边界 |
| 18 | dirt | stone | 655 | 表面规则 |
| 19 | sandstone | stone | 638 | 海底砂岩层 |
| 20 | deepslate | water | 635 | 深板岩含水层 |
| 21 | stone | copper_ore | 541 | 矿脉 |
| 22 | water | grass_block | 540 | 海岸线/被淹 |
| 23 | stone | iron_ore | 452 | 矿脉 |
| 24 | sand | sandstone | 427 | 海滩表面 |
| 25 | deepslate | andesite | 321 | 浅层岩脉 |

### 表 2-2：机制分组汇总（数量为按 pair_counts 逐项归并的计算值，四舍五入）

| 组 | 机制 | 估算块数 | 占比 | 主要 (got→vanilla) |
|---|---|---|---|---|
| A | **OreFeature 石质缺失**：浅层岩脉（andesite/granite/diorite）与 tuff | ≈ 32758 | 48.9% | stone/deepslate → andesite/tuff/granite/diorite |
| B | **OreFeature 矿脉缺失**：全部 *_ore | ≈ 2900 | 4.3% | stone/deepslate → coal/iron/copper/…_ore |
| C | **gravel 类**：深板岩层 gravel blob + 海底 gravel | ≈ 4900 | 7.3% | deepslate → gravel（1802，深层）；gravel↔stone（2881，海底） |
| D | **水体边界**：含水层 + 海底高度差 + 海岸被淹 | ≈ 13000 | 19.4% | stone→water（4416，含水层）；water→stone/dirt/sand（6380+，海底更低）；deepslate→water（635） |
| E | **空气差（C++ 实心 vs vanilla 空）**：洞穴为主 | 6428 | 9.6% | deepslate→air（5411）、stone→air（775）等 |
| F | **空气差（C++ 空 vs vanilla 实心）**：结构内部 + 表面 | ≈ 1260 | 1.9% | air→oak_stairs/oak_planks/cobblestone/grass/… |
| G | **表面规则互换**：sand/sandstone/dirt/grass_block/magma/smooth_basalt/calcite/cobblestone/cave_air 等 | ≈ 2900 | 4.3% | sandstone↔stone、sand↔sandstone、dirt↔stone、deepslate→smooth_basalt/calcite（紫晶洞外壳） |
| H | **结构+植被**：村庄、沉船、海底植物、树草、紫晶洞 | ≈ 2400 | 3.6% | 含于 F/G 及各 FEATURE 块 |

> 注：A-D-G 之间有小幅交叉（例如水中的 andesite/diorite/granite 差异 105 块、水中 ore 26 块已计入 D 或 B），分组为工程近似，合计 ≈ 67042 量级，用于量级判断而非精确划分。

### 2.3 各机制的空间证据（来自坐标采样，确定）

- **A 浅层岩脉**：stone→andesite 样本 y∈[9,36]，cold_ocean，斑块状（如 chunk(-16,-15) x∈[-247,-253] z∈[-225,-232] 连片、chunk(-17,-15) y=30-36 连片）；tuff 样本 y∈[-45,-12]，深板岩层。与 MC 1.18+ OreFeature 的浅层岩脉（y>0 石层）和 tuff 区域（y<0 深板岩层）位置吻合 → **[推测]** C++ 未实现 OreFeature 石质配置。
- **E 洞穴**：deepslate→air 样本 y∈[-45,-25]，连片大块（如 chunk(-18,-15) x∈[-273,-288] z∈[-225,-239] 整片空洞），位置在深板岩层 → **[推测]** C++ 的洞穴生成（carver / cheese cave 布尔体积）未对齐，vanilla 有洞而 C++ 填实。
- **D 水体**：stone→water 样本 y≈15-17（chunk(-18,-15) x∈[-273,-288] z∈[-225,-240] 大片）→ **[推测]** vanilla 的地下含水层，C++ 未生成水（实心）；water→stone/dirt 样本 y∈[49,61]（chunk(-16,-16) x∈[-241,-244]、chunk(-15,-16) river/cold_ocean）→ **[推测]** C++ 海底面系统性低于 vanilla 约 2-6 格，或 C++ 在海底边界仍判为水。
- **C 海底 gravel**：gravel→stone 样本 y∈[49,52]（chunk(-16,-16) cold_ocean + chunk(-15,-16) river/cold_ocean 大片）→ **[推测]** 海底 surface rule 的 gravel/stone 分布不一致（MC cold_ocean 海底 gravel 斑块由 surface rule + 随机决定，C++ 铺设位置不同）。
- **H 结构**：dirt_path 样本集中 plains y=62-64（chunk(-17,-14)/(-16,-14)/(-17,-13)/(-18,-13)）→ 村庄道路；oak_stairs/oak_planks 样本两处：(a) plains y=63-70（村庄房屋），(b) cold_ocean y=62-68（沉船，chunk(-16,-14) x∈[-256,-250] z∈[-214,-220]）；farmland 样本 chunk(-17,-14) y=63（村庄农田）→ **[确定]** 存在 1 个平原村庄 + 1 艘（或更多）沉船，C++ 完全未生成。

---

## 3. 坐标分布（空间形态）

### 3.1 chunk 级分布（确定，`m288_chunk_counts.txt`）

| cz＼cx | -18 | -17 | -16 | -15 | 行合计 |
|---|---|---|---|---|---|
| -16 | **0** | **0** | 364 | 1390 | 1754 |
| -15 | 2529 | 2090 | 2868 | 2487 | 9974 |
| -14 | 6918 | **8806** | 7248 | 5842 | 28814 |
| -13 | 5685 | 7543 | 7084 | 6188 | 26500 |

- **(-18,-16) 与 (-17,-16) 为 0 差异（100% 匹配）**——纯深海、无结构、无浅层岩脉命中，证明 C++ 核心在该处完全一致。
- **z=-14 行最差**（合计 28814，单 chunk 峰值 (-17,-14)=8806）：该行是**平原村庄 + 沉船 + 海岸/河流混合区**，结构差异叠加海底差异。
- z=-13 行次差（26500）：村庄 + 海洋边界。
- z=-15 行（9974）：深海，主要是浅层岩脉 + 洞穴 + 含水层。
- z=-16 行最浅（1754）：纯深海，仅海底边界零星差异。

### 3.2 y 层分布规律（确定，来自各组合采样）

| y 区间 | 主要差异 | 机制 |
|---|---|---|
| 62-70 | dirt_path / oak_planks / oak_stairs / farmland / hay_block 等 | 村庄 + 沉船 + 海平面附近 |
| 49-61 | water↔stone/dirt/sand、gravel↔stone | 海底边界 + 海底表面规则 |
| 9-36 | stone→andesite/granite/diorite | 浅层岩脉（石层） |
| 0~-45 | deepslate→tuff / deepslate→air / deepslate→gravel / deepslate→amethyst | tuff 区域 + 洞穴 + gravel blob + 紫晶洞 |
| -63~-59 | deepslate→gravel | 深板岩底部填充 |

### 3.3 区域生物群系（确定，从 biome 字段采样）

cold_ocean（主体，占样本绝大多数）+ river（chunk(-15,-16) 一带）+ beach（chunk(-15,-14)/(-15,-13)/(-16,-14) 一带）+ plains（chunk(-18,-13)/(-17,-13)/(-16,-13)/(-17,-14)/(-16,-14) 东北角）。**这是一个海岸-河口-平原村庄混合区，而非单一深海**——这解释了为何差异并非均匀散布而是按生物群系/结构聚簇。

### 3.4 空间结论

- 差异呈**结构化聚簇**而非随机散布：村庄聚簇（plains）、沉船聚簇（cold_ocean 一处）、海底边界聚簇（chunk(-16,-16)/(-15,-16)）、洞穴聚簇（chunk(-18,-15)/(-17,-15)/(-18,-14) 深板岩层）、紫晶洞聚簇（chunk(-17,-13)/(-16,-13) x≈-257~-260 z≈-193~-197 y≈-19~-28）。
- 纯深海 chunk 已 100% 一致，说明**问题不在基础密度函数本身，而在海域/结构/洞穴/Feature 等下游阶段**。

---

## 4. unknown 205 块：是什么（确定）

`m288_vanilla_cat.txt` 的 unknown 明细经 blocks.json 反查确认，全部是 C++ 未生成的 FEATURE/结构件，**没有一个是未知的天然块**：

| ID | 名称 | 块数 | 来源 |
|---|---|---|---|
| 903 | amethyst_block | 74 | **紫晶洞**壁（chunk(-17,-13)/(-16,-13) y≈-19~-28，geode FEATURE） |
| 904 | budding_amethyst | 8 | 紫晶洞 |
| 905/906 | amethyst_cluster / large_amethyst_bud | 2 | 紫晶洞 |
| 184 | farmland | 27 | 村庄农田 |
| 477 | hay_block | 22 | 村庄 |
| 318 | glow_lichen | 24 | 洞穴发光苔藓（FEATURE） |
| 311 | glass_pane | 14 | 村庄 |
| 130/134 | white/yellow_wool | 16 | 村庄（含地毯 482） |
| 103 | white_bed | 2 | 村庄 |
| 773/834/778 | loom / composter / fletching_table | 5 | 村庄 |
| 783 | bell | 1 | 村庄 |
| 147/149/152/158 | dandelion / poppy / azure_bluet / cornflower | 9 | 村庄花 + 自然花 |

合计 205 块 ≈ 0.31%，全为结构/FEATURE。**它们被脚本标 unknown 只因分类集合未覆盖，不影响任何结论。**

---

## 5. 定位建议（下一步）

按「修复收益 × 定位成本」排序（建议主会话裁决）：

### 5.1 优先：海域/河流海底边界（D 组，约 1.3 万块）
- 锚点：chunk(-16,-16)（364 块但纯海底）、chunk(-15,-16) river/cold_ocean（x∈[-226,-240], z∈[-241,-256], y∈[49,61]）；chunk(-17,-14)/(-16,-14) 海洋部分。
- 动作：对上述区域做 **C++/Java 海底面高度逐列对比**（每列取第一个实心块 y），检查是否系统性低 2-6 格；同时对比海底 surface rule 的 gravel/stone/dirt 铺放。重点核对：`Aquifer` 液面（sea level=63 后的含水层解算）、`OceanFloor` surface builder、以及 density 在海底处的 interpolation 是否复用 Java 的 `ScalablePerlinNoise` 采样格。

### 5.2 次优先：洞穴（E 组，6428 块，其中 deepslate→air 5411）
- 锚点：chunk(-18,-15)/(-17,-15) 深板岩层（y∈[-45,-25]）连片空洞，chunk(-18,-14) 亦有大片。
- 动作：对比 C++ 与 Java 的洞穴布尔体积（cheese cave / spaghetti cave / noodle 的半径与位置），确认是「洞穴随机种子/噪声未对齐」还是「carver 完全未实现」。可先取 chunk(-18,-15) 内一个 8×8 竖直切面对比。

### 5.3 决策项：OreFeature（A+B+C 深层，约 3.6 万块）
- **这是量级最大的一组（56%）**，但若项目范围明确不含 ore feature，则应在对齐目标中剔除并显式记录，避免后续反复 reopen。
- 若纳入：浅层岩脉/tuff 是 OreFeature 的 `ore_andesite/granite/diorite/tuff` 配置，坐标由 `PlacementModifier`（RarityFilter + HeightRange）决定，建议对照 Java 的 `PlacedFeatures` 逐配置核对随机数流顺序。

### 5.4 顺带：含水层（stone→water 4416，y≈15-17）
- 锚点：chunk(-18,-15) x∈[-273,-288] z∈[-225,-240] y=15-17 大片。
- 动作：属 `Aquifer` 的 underground water level 解算，可与 5.1 合并排查。

### 5.5 范围确认：结构（村庄 + 沉船 + 紫晶洞 ≈ 2500 块）
- 已确认 C++ 完全未生成（村庄在 plains、沉船在 cold_ocean、紫晶洞在深板岩层）。若范围不含结构，作为已知差异记录即可。

### 5.6 方向性提示
- **不要再往 base_3d_noise 找**（03 篇知识库已排除）：z=-16 行 100% 匹配已再次证明基础 density 函数在负坐标正确。
- 建议下一轮定位从 **density 之后、块填充之前的阶段**（Aquifer → SurfaceBuilder → 洞穴 carver → OreFeature）入手，与上述 5.1/5.2 的坐标锚点做 Java 端 `debuggable` 对比。

---

## 6. 置信度与边界说明

- **[确定]**：所有计数（pair/类别/chunk 汇总）、(-18,-16)/(-17,-16) 零差异、biome 集合、结构块的空间位置、unknown 块名映射。
- **[推测]**：各机制归因（浅层岩脉=OreFeature、deepslate→air=洞穴、water↔stone=海底高度差、gravel↔stone=surface rule、stone→water=含水层）——方向由坐标形态强烈支持，但确认需 5.1-5.4 的 C++/Java 逐列对比。
- **口径提示**：组 A-C 的「块数」是依据 pair_counts 逐项归并的工程近似，交叉项（水中 ore/水中岩石变体）已按主项归属，总误差 <1.5%。
- 本报告为勘探产物，不改代码；若主会话决定把 OreFeature/结构列为范围外，建议更新对齐目标后再评估匹配率，届时「假 diff 占比」会显著上升，8/8 结案可按新口径重估。
