# fan-out 收敛判定：chunky 地形差归因（260907-01）

> 状态：主会话收敛（收敛门 v0.13：候选齐备后单线收敛），draft → judge 审查中。
> 输入：.b1 candidate.md（支持）、.b2 candidate.md（证伪）、diff_per_chunk_260907-01.txt、structures_nbt_260907-01.txt、column-read-260907-01.md。

## 双候选裁决综合

| 候选 | 裁决 | 要点 |
|---|---|---|
| .b1 blob 石残差归因 | **支持** | 地形差主力（粗界 60-75%）= 已知 blob 石残差域：granite/diorite/andesite 双向均衡互换 ≈48.7k、≈15-16 块/chunk（judge N-6：口径混合，降格为「同量级带参照」）、簇状空间分布，三证据与知识库已知记录（13 篇）吻合。无需新机制。 |
| .b2 结构阶段差 | **证伪**（非独立第 5 通道） | starts 层两臂零差（13 mineshaft 等 starts/status/id 全同）；块级结构签名 ~330 块/3025 chunk 与「整件级差」先验差一个数量级以上；geode 两臂均 Java vanilla feature 放置（mixin 只拦 NOISE/SURFACE，源码引证），geode 差 = 通道①/②下游表现面。children BoundingBox 盲区已 §9.7 声明。 |

## air / water 桶归属判定（开工点 3，主会话数据核对：grep diff_per_chunk 全量 water/cave_air 行）

**water（846）→ aquifer 域**（11 篇已挂起课题，非新通道）：
- 主力 = water↔deepslate 大簇（99/110/186/124/46/45/38/…），集中少数 chunk（17,-31 / 18,-30/-31 等）= aquifer 水囊边界差签名。
- 散点 water↔stone/gravel/dirt 单块 ~数十处 = aquifer/blob 下游。
- 两臂 aquifer 均由各自地形阶段计算（Rust vs Java），此域即已知挂起的 aquifer 残差课题辖区。

**air（cave_air 548）→ 三分**（无独立新通道）：
1. **geode 壳差** ~180-260 块（cave_air↔smooth_basalt/calcite/amethyst 大簇：84+49+44、8+9+8）= 同一 Java geode feature 在不同地形基座/执行序下成败差 → 通道①/②下游（.b2 定性）。
2. **结构语汇残余** ~200-330 块（cave_air↔rail/cobweb/oak_planks/oak_fence/spawner，cx=42,-27/-28 簇 + cx=30/31,-23 spawner/cobweb）→ **未闭合点**，.b2 建议 children BoundingBox dump 消解（mineshaft vs 2 个 ruined_portal starts 语汇归属未定）。
3. **cave_air↔deepslate/stone 大簇**（83、52、47、22+20…）= 洞穴/结构 carve 与地形残差交叠的下游差 → 通道①/②下游，量级小，不立新通道。

## 综合结论（draft candidate，待 judge + 用户）

1. chunky 全区域地形差可完整归账到**已开封通道**：已知 blob 石残差域（主力）+ aquifer 挂起域（water）+ 通道①/② feature 下游（geode/grass_block↔dirt 表层簇 ~5.1k 疑似树 below-dirt 泄漏）+ 小额未闭合结构残余（~330 块）。
2. **无第 5 独立归因通道**（.b2 证伪）；F3 通道①主归因（~0.9）不受动摇；通道②维持降级（column-read 直证 483 柱地形零差）。
3. 剩余未闭合点（不阻塞本轮收口，列为后续采集项）：
   - 13 mineshaft + 2 ruined_portal starts 的 children BoundingBox dump（消解 ~330 块 cobble/cobweb/spawner 簇归属）。
   - grass_block↔dirt 5.1k 簇的 y 分布核查（定树 below-dirt 泄漏 vs 表层 rule 差）。**judge N-3 修正：5.1k 为双桶 name 计数相加的上界粗界**（dirt 2868 含 blob 族 gravel↔dirt 成分），树 below-dirt 真实量级 ≈2.3k 起。
   - .b1 口径疑点：top8 截断列表 vs 全量计数 ≈7 万 vs 113k 不自洽——精确分账需全量对表重算（粗界已够本轮裁决）。**judge N-4 补充成因：采集脚本单 palette section 的 ±2048 "unknown split" 近似 + 跨臂 section 不齐整段跳过盲区**，一并并入全量重算项。
4. 「500 柱地面低 6 格」由本块 column-read 取代：region 载体下地形基座一致，差异为植被层（§15.4 取代记录，见 column-read-260907-01.md）。

## 验证分层声明

- 本收敛 = Degraded（region NBT 数据静态审查），载体 = Chunky 双臂 region（#26 confirmed），seed 8576294172403134396，覆盖 3025 chunk 全区域、两焦点柱全列。
- 全部比例声明为粗界（#70 非可交换性 + .b1 口径疑点）。
