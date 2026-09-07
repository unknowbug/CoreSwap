# .b1 候选：blob 石残差归因（有差 chunk 地形层主力 = 已知 blob 石残差域）

- 状态：**draft**（禁 confirmed；本分析 = **Degraded 静态/数据审查层**，无任何探针/运行时证据）
- worker：fan-out 260907-01 .b1（只读分析 + 本文件，未动主工作区）
- 数据源：`.tmp/jungle-l-260906/chunky/diff_per_chunk_260907-01.txt`（799/3025 chunk 地形差，每 chunk top8 差异对 + 全区域 top 名计数）
- 知识参照：`versions/1.20.1/docs/13-feature-parity.md`（L16：双向 ore 小残差 andesite/granite/diorite 各 ~1-1.2 万双向均衡）、`docs/11-features-stage.md`（L42-45：永久挂起 aquifer 2 + ore_vein 1，残差放大候选域）、`docs/07-block-pipeline.md`（L1217-1227：blob 状置换签名 + aquifer 挂起族）、`docs/04-aquifer.md`（blob 影响场）

## 1. 差异对分类（按 top 名计数，L2919-2944；注意计数口径见 §6 陷阱③）

| 类别 | 成员（top 名实例数） | 量级 | 归属判断 |
|---|---|---|---|
| **blob 石家族** | andesite 16973 + diorite 15885 + granite 15253 + tuff 595 | ≈48.7k 实例，**占 top 名总计 ~70%** | ✅ 主力，本候选域 |
| 矿石/矿石-石互换 | coal_ore 3737 + stone 3133 + deepslate_gold 290 + iron 203 + deepslate_iron 152 + copper 165 + gold 71 | ≈4.7k | 多为 blob/ore placement 下游（blob 簇内 coal↔stone 成对互换，L1401-1407、L1892-1899；纯 deepslate_ore↔ore 为 deepslate 过渡带微差，如 L59、L1310） |
| 表层 | grass_block 2271 + dirt 2868 | ≈5.1k | ⚠️ dirt 双重身份：`gravel↔dirt` 对属 blob 族；`grass_block↔dirt` 双向成对簇（L1318-1319、L1390-1394、L464-474）签名像树 placement 的 below-dirt/表面副作用泄漏进地形口径（grass_block/dirt 不在植被清单）——**建议让渡给 feature 层候选核查 y 分布后定夺**，本候选不计入 |
| 结构残留 | cobblestone 172 + mossy_cobblestone 101 + oak_planks/fence/rail（~50） | **<500，集中 3-4 个 chunk**（31,-22：L424-431；33,-24：L1915；39,-1：L2858-2866 等） | mineshaft 组件签名明确（cobble/mossy/oak_planks/rail 共现）→ **让渡 .b2 结构候选**，本候选不解释 |
| geode | smooth_basalt 103 + calcite 70 + amethyst_block（L488、L2861） | ≈200+，2-3 个 chunk（30,-21；42,-25；39,-1——注意 39,-1 同 chunk 内 geode+mineshaft 交叠） | geode 签名明确 → **让渡结构/geode 候选** |
| 地貌/水 | water 846 + sand 515 + clay 299 | ≈1.7k | water↔deepslate 大簇（17,-31：99；18,-31：110；18,-30：186，L94-105、L142-148）= **aquifer 固/液边界**，属 11 篇已挂起 aquifer 域（已知残差域，非新机制）；sand/clay 成对互换为湖/disk 类边缘差 |
| 洞穴边缘 | air 371 + cave_air 548（air↔deepslate 单块散布，如 L3、L11） | ≈0.9k | carver/洞穴边界擦边单块，量级小 |

## 2. 空间分布（粗粒度观察）

- 差异 chunk **明显成簇而非随机散布**：cz 每行内 cx 连续成段（如 cz=-34..-21 行聚集在 cx 11-31；cz=-25..-1 行聚集在 cx 32-46）；典型 blob 簇如 cx=32/-24（201 块纯 diorite↔andesite，L1911-1912）与 cx=33/-24（265 块，L1913-1914）相邻 chunk 同一对主导。
- 单对占绝对主导的超大 chunk（L36：40 块纯 granite↔andesite；L70：67 块纯 granite↔diorite；L1308-1309：118 块 diorite↔andesite）= 单个 blob 簇跨入单 chunk 的形态，与 blob 簇状生成一致。
- 散布的 1-3 块小 chunk 属洞穴/边界擦边单块，量级小不改变主体判断。
- 结论：空间签名 **支持 blob 簇状分布**，不支持随机散布的新机制。

## 3. 对照已知残差清单

- **类型谱吻合**（13 篇 L16）：granite 族三成员双向均衡互换（文件中 `A<->B` 双向均大量出现，如 L19 与 L32 方向互逆），与「双向 ore 小残差、andesite/granite/diorite 各 1-1.2 万」签名一致——本区域量级 ~1.5-1.7 万/成员，同量级带（跨载体口径不同，见 §6③，只做量级带比较）。
- **量级吻合**：blob 族 ≈46-49k ÷ 3025 chunk ≈ **15-16 块/chunk**，与 260906-09 已知残差域「≈15 块/chunk」（10 时间线 L3069）吻合。
- **挂起域不扩容**：aquifer water 簇对应 11 篇 L42 挂起的 aquifer 固/液边界族（(198,18,198) water→dirt、(237,42,224) gravel→water 同签名），非新发现。

## 4. 裁决

**支持（支持该候选，status 停在 draft，建议主会话收敛后交 judge；本 worker 不授予 candidate 以上）**：

> 799 个有差 chunk 的地形层差异主力（粗界：60-75% 实例量）= 已知 blob 石残差域（Rust blob 石生成 vs Java 的已知双向 granite 族残差），类型谱、量级带（≈15 块/chunk）、空间簇状分布三方面均与知识库已知残差记录吻合，**无需引入新机制**。

让渡清单（不属于本候选解释范围）：
- mineshaft 结构残留（<500 块，3-4 chunk）→ .b2 结构候选
- geode 签名（~200 块，2-3 chunk，含与 mineshaft 同 chunk 交叠的 39,-1）→ 结构/geode 候选
- aquifer water 簇（~1k 块，3-4 chunk 大簇）→ 11 篇已挂起 aquifer 域（非新候选）
- grass_block↔dirt 表层簇（~5k 实例）→ 待 y 分布核查后归 feature 副作用或 surface 微差，暂挂本候选外
- coal_ore/stone 互换中不能被 blob 覆盖的部分 → ①的下游核查项

## 5. 未验证假设清单

1. 「grass_block↔dirt 簇 = 树 placement below-dirt 副作用泄漏」——仅凭签名推断，需 y 分布（是否全在 surface 附近）+ 与植被差 chunk 空间重合度验证。
2. 「coal_ore↔stone 互换是 blob 置换的下游」——未做 y 坐标/空间包含关系核对；不排除 ore placement 独立残差。
3. 「blob 族差异 = 残差域而非 Rust blob 生成新 bug」——本分析只验证了与已知签名/量级吻合，未做 Rust 侧 blob origin 对拍（无 shell，探针不可行）。
4. 水域簇「= aquifer 边界」——位置未与 aquifer blob 网格核对（也可能是 carver 水系）。
5. 799/3025 chunk 有差中「无差 2226 chunk」两侧地形逐位全等——继承自背景陈述，未独立验证。
6. deepslate 过渡带 ore 变体互换（deepslate_iron↔iron 等，~1k）归 deepslate 边界微差——未验证。

## 6. 口径陷阱声明（§9.7 自查）

① 本分析载体 = Chunky region 存档口径（terrain-only，剔植被/空气），与 13 篇 treediag 口径、11 篇 blockProbe 口径**不可比**，仅做签名/量级带参照；② 结构/geode 归属为量级标注与归属建议，不构成结构结论（明示让渡 .b2）；③ **数据内部口径疑点**：top 名计数总和 ≈7 万实例，若每差异块计入两侧各一次则推得 ~3.5 万块，与 terrain=113k 不符——疑似「top8 截断的 per-chunk 对列表」与「全量名计数」来源不一致，本文件所有比例只作粗界使用，精确分账需重算全量对表；④ 概率/占比一律区间表述，无单点数（#70）。
