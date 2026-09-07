# children 零差 + dirt y 分布解读（260907-02，core.worker 草稿）

> **状态：draft**（置信度建议见文末；AI 不授 confirmed）。
> 输入采集（主会话 260907-02）：
> - `.tmp/jungle-l-260906/chunky/children_dump_260907-02.txt`（两臂 7 starts children id+BB 有序对比 + 簇覆盖核对）
> - `.tmp/jungle-l-260906/chunky/ydist_dirt_260907-02.txt`（全区域 dirt 涉及差异对 × y 直方图）
> - 参照：`.tmp/jungle-l-260906/chunky/diff_per_chunk_260907-01.txt`（簇 chunk 原始 name 对）
> - 上下文：`.investigations/jungle-l/fanout-260907-01/convergence.md`、`review-260907-01.md`
> 知识库引用：#71（starts 零差裁决 + children 盲区 MUST §9.7 声明）、#72（name 双桶相加上界陷阱）。

## §9.7 验证可比性声明（载体 / 覆盖面 / 可比性）

- **载体**：Chunky 双臂 region NBT 静态解析（#26 confirmed 载体），Degraded 分层（无运行时探针）。
- **覆盖面**：children dump = 本区域 **7 个 starts**（mineshaft×1 + monument + ocean_ruin_cold/warm + ruined_portal + ruined_portal_ocean + shipwreck），seed 8576294172403134396，A=coreswap、B=vanilla。**注意：上块「13 mineshaft starts 全同」的区域口径与本块 7 starts 的 dump 口径不同——本 dump 未覆盖全部 13 mineshaft starts**（脚本只读 chunk structures 的 starts 键、未读 references 键，见盲区）。
  > **口径修正指引（260907-02 采集后补注，指向下方 §B1/B2 消解结果 B2'）**：本文所有「7 个 starts / 13 mineshaft starts（上块口径）」表述均为**按 template 名合并的假口径**——mineshaft 实为 13 个独立 start 实例，7-starts dump 的 mineshaft 项实为一个模板实例且 children 只保留了最大实例（n=219 为合并产物）。逐实例正确键 = (template, ChunkX, ChunkZ)；原文 7-starts 数字不改写，引用时以本注记 + B2' 为准。
ydist = 全区域 3025 common chunk 全量差异对按 (name 对, y) 分桶。
- **与既有口径可比性**：本块 dirt 数值全部为**差异对口径**（每对只计一次方向，如 dirt↔grass_block 1134+1129=2263 双向合计），与 judge N-3 的「5.1k 双桶 name 相加上界」**不可直接比**（#72）；与 N-3 的「净化下界 ≈2.3k」同口径可比（见 Q2 对账）。ydist 桶宽 8（y0-7/8-15/…），top-y 为单 y 精确值，两口径在文件内混排但互不相加。

---

## Q1：~330 块结构语汇簇归属

### 证据链

1. **children 有序列表两臂 IDENTICAL**：7 starts 全部（n=219/1/3/6/1/1/1），含 id+BB。结构 piece 布局生成两臂确定性一致 → .b2「结构阶段非独立通道」进一步加固。
2. **簇 chunk 无 starts 引用**：簇 chunk（42,-27/-28、30/31,-23，及 28,-22、31,-17）不在任何已 dump start 的 chunks 引用集内（7 个 starts 的 `covers cluster: []` 全空）。
3. **BB 坐标系异常是普遍现象，不止 mineshaft**（本块新发现，修正 prompt 中「唯 mineshaft 不吻合」的初判）：
   - 吻合：monument startChunk=(2,-22) ↔ BB x3-60/z-381..-324（世界系）；ruined_portal (44,-18) ↔ BB x693-704/z-288..-279（世界系）；ruined_portal_ocean、shipwreck、ocean_ruin_cold 同样吻合。
   - **不吻合 ×2**：mineshaft startChunk=(4,-30) ↔ BB x111-243/z84-255；ocean_ruin_warm startChunk=(22,-36) ↔ BB x418-456/z-17..22。
   - 推断（draft）：mineshaft 与 ocean_ruin_warm 的 piece BB 存储在**非世界坐标系**（piece 局部/累积生成系，生成期未经最终 offset 平移），因此 **BB 不能用于把块级簇归位到具体 start**——这同时废掉「用 BB 反推簇归属」的路径，也解释了为何「mineshaft 不吻合」不能作为排除 mineshaft 的证据。
4. **语汇归因（最强独立证据）**：簇内结构语汇 = spawner / cobweb / rail / oak_planks / cave_air 双向位移互差（diff_per_chunk 260907-01：cx=30/31,-23、cx=28,-22、cx=31,-17）。**spawner+cobweb+rail 组合是 mineshaft 专属语汇**（洞穴蜘蛛刷怪笼+蛛网+铁轨）；ruined_portal 语汇为 cobblestone/mossy_cobblestone/stone bricks/obsidian/gold_block/magma，**不含 spawner/cobweb/rail**。候选 (b)（vanilla 装饰器 cascade）机制上不产生 spawner/rail/cobweb（decorator 不放置这些方块）。
5. **双向位移式互差**（A 有 spawner 处 B 是 cave_air 且邻近互翻）：与「同一结构 piece 布局、放置与洞穴/地形交界面的交错表现」一致——同一走廊网格在两臂与略差的洞穴空洞相交，暴露/被覆的结构方块不同，呈互翻而非单侧缺失。这与 (a) 的「布局一致、结果因上游输入差位移/暴露」表述吻合，也与 (b) 的「交界面差异」不冲突——**语汇来源与差异机制可分层**。

### 判定（draft）

**语汇归属：mineshaft（候选 a 的前半段），不支持 ruined_portal，亦不支持纯装饰器 cascade（候选 b 主体）**——由证据 4 一票定：三件套 spawner+cobweb+rail 在本区域语境下唯一指向 mineshaft。

**通道归属：倾向 (a) 但不闭合**。「某个经 references 引用放置（或未被本 7-starts dump 覆盖的其余 mineshaft start）的 mineshaft，piece 布局两臂一致、块级差为结构-地形交界面表现」是当前最合理假说；但本采集存在三项未消解盲区（下），不足以把「簇 ← 具体哪个 mineshaft、经哪条放置路径」闭合。**不做硬闭合，候选 (a) 与 (b) 不构成本轮需要 fan-out 的互斥竞争**（语汇证据已把 (b) 的独立机制版本压到低概率；残留分叉是「哪个 start / 哪条路径」，属采集消解型，不是机制竞争型）。

### 盲区显式声明（#71 MUST）与消解采集项

| # | 盲区 | 消解采集项 |
|---|------|-----------|
| B1 | 采集脚本只读 `structures.starts`，未读 **`references` 键** → 「经 references 放置的 mineshaft」路径完全未观测 | 重跑 dump 增加 references 键（chunk→start 引用图），核对簇 chunk 是否落在某 mineshaft 的 references 集内 |
| B2 | 本 dump 仅覆盖 7 starts，区域实有 13 mineshaft starts（上块口径）；**簇归属可能落在未 dump 的 6 个 mineshaft starts 之一** | dump 范围扩到全部 starts（不限 7 个） |
| B3 | mineshaft / ocean_ruin_warm 的 BB 坐标系未定（局部/累积生成系假设未验证）→ 无法用 BB 把簇 chunk 归位到具体 piece | 任取一 mineshaft start，用其 startChunk 世界坐标 + piece 相对 offset 复算一条走廊的预期世界 BB，与 NBT 存储值对拍，定坐标系换算式 |

（候选排序：(a) > (c-混合：语汇=mineshaft、机制=交界面表现) > (b)。若 B1/B2 消解后发现簇 chunk 与任何 mineshaft 引用集都不相交，则 (b)/语汇污染假说重开，届时再议 fan-out。）

## §B1/B2 消解结果（260907-02 主会话采集）

> 本节为主会话 260907-02 新采集结果的落盘转记（core.worker 草稿；产物：`.tmp/jungle-l-260906/chunky/children_per_start_260907-02.txt`（summary: identical=24 diff=0）与 `references_dump_260907-02.txt`）。上方盲区表三项中 B1/B2 就此关闭，B3 保持未消解（见末条声明）。

1. **B1 关闭——references 键全空**：两臂 chunk `structures.references` 键全空（该区域无任何引用型结构放置记录）。「经 references 放置的 mineshaft」路径在本区域**不存在**，原「完全未观测」盲区消解为「已观测且为空」。
2. **B2' 关闭——start 实例口径修正**：mineshaft 实为 **13 个独立 start 实例**，两臂 startChunk + nchildren **逐实例全同**。先前「7 starts / n=219」口径系**按 template 名合并的假口径**——一个模板多个实例被合并、children 只保留了最大实例（见 §9.7 补注指引）。B2 原文「区域实有 13 mineshaft starts」的部分猜中被证实，且 13 个实例两臂无任何一例差异。
3. **B2'' 关闭——children 逐实例全同**：按 (template, ChunkX, ChunkZ) 逐实例对比，**24/24 实例 children（id+BB 有序列表）两臂 IDENTICAL**（`children_per_start_260907-02.txt`：summary identical=24 diff=0）。#71 盲区声明的「starts 相同但 children 组装不同」残余可能被直接排除：**piece 布局层两臂零差**。
4. **消解效果——通道 (a) 升 candidate 建议**：B1/B2 关闭 + children 零差 → 「通道 (a)：语汇=mineshaft 且 piece 布局层两臂零差，~330 块簇差为放置/暴露面级 cascade 表现」**建议升 candidate**（judge 审查后由人类确认）。原判定节的「不做硬闭合」理由（三项盲区）已消解两项。
5. **B3 保持未消解声明**：BB 坐标系非世界系问题（mineshaft 与 ocean_ruin_warm 的 BB 与 startChunk 世界坐标不吻合，monument/ruined_portal 吻合，成因未查）**未消解**；但已不阻塞主结论——原「mineshaft BB 不吻合不能作排除证据」的判断仍成立，且簇归属闭合不再依赖 BB 归位（语汇证据 + children 零差已足够支撑通道 (a)）。

---

## Q2：dirt↔grass_block y 分布定性

### 数据（差异对口径，#72 合规）

- dirt↔grass_block 双向合计 **2263**（1134 + 1129），双向近似对称（50.1% / 49.9%，差 5 块）。
- y 分布：y64-71 ≈ 75.4%（866+841）；y72-79 ≈ 18.1%；y56-63 ≈ 5.2%（118）；y80+ ≈ **29**（5+3+11+10）。y62-79 合计 ≈98.7%（judge N-4 勘误：原写 ≈97%）。峰值 y68-71（279+157+143+110 / 271+162+137+103）。

### 论证：支持「树 below-dirt 泄漏」为主，表层 rule 差不是主力

1. **双向对称性**：系统性 surface rule 差（如草判定阈值/深度差）会产生**方向偏置**的系统性层差（一臂整体多草或少草，且 y 轮廓随高度图连续展宽），而非两臂 1134:1129 的近乎完美对分。对称对分的最自然解释 = 两臂各自「有树的一侧」把树下 grass_block 换成 dirt，而对方同位置保持 grass_block——每个位点单向，全区域对分（树放置差由上游地形差随机决定哪臂有树）。
2. **y 带状集中**：y64-71 峰值带 = 本区域丛林地表高度带，树干底部 below-dirt 恰落于此。surface rule 差若是机制，应同时在 grass↔stone/浅层材料出现同量级配对差——未见（grass↔dirt 仅 4，dirt↔stone 合计 22）。
3. **伴随语汇**：oak_leaves↔dirt（y69-72，5 块）、jungle_log↔dirt（y70）、vine↔dirt（y70）、fern↔dirt（y71）、cocoa↔air 散点遍布同一 chunk 带——全部是植被 feature 语汇，与树放置差同源，交叉印证树域而非表层 rule 域。
4. **与 judge N-3 口径对账**：N-3 指出 5.1k 是双桶 name 相加上界（dirt 总桶 2868 混入 gravel↔dirt 等 blob 族成分），净化下界 ≈2.3k。本采集按差异对口径直接分账：树候选主对（dirt↔grass_block）= **2263**，与 ≈2.3k 净化下界**吻合（落在其下沿）**——N-3 的净化方向被数据确认；剩余 dirt 涉及对（gravel↔dirt 266 深层 y3-57、sand↔dirt 162 y34-69、各类 ore↔dirt 合计 ≈145（judge N-4 勘误：原写 ~135））y 域与表层不重叠，归 blob/矿化域，不计入树账。
5. **混合成分判定**：不能排除少量表层 rule 差混入（y62-63 的 ~118 块中，靠近水面的低地/滩面位点可能是表层 builder 在边缘地形的差；y62-63 也是洞穴顶穿出的 cave_air↔dirt/stone 交界带，见 diff_per_chunk cx=27,-22 等洞穴簇），但量级 ≤ 百块级，不改变主归因。

### 边缘带解释

- **y80+（~29 块）**：y88-95（21 块）= 高地/山脊地表上的树 below-dirt（丛林大型树可放在抬升地形），仍属树域；与 stone↔dirt y80-95 的 ~9 块相邻出现，是地表高度高的柱位，非独立机制。
- **y62-63（~118 块）**：三成分——①低地/近水面地形（地表本身在 y62-63）的树 below-dirt；②洞穴顶穿出交界（cave_air↔deepslate/dirt 簇的边缘互翻，如 cx=27,-22 的 6+6）；③ vine/藤下泄漏散点。无证据显示需要新机制。

### 判定（draft）

**主归因 = 树 below-dirt 泄漏**（两臂 vanilla 树放置因 Rust 地形上游差导致放置位点/成败差，树干下 dirt 互翻），量级 ≈2.26k（差异对口径上界，天然即净化后账目）；**表层 surface rule 差非主力**（如有混入 ≤ 百块级）。judge N-3 的「5.1k 上界 → ≈2.3k 净化下界」修正链被本采集闭环。

## 验证分层声明

- 全程 Degraded（region NBT 静态审查），无运行时探针；载体/覆盖面/口径见 §9.7 块。
- 本文件全部判定为 **draft**；未做 fan-out（理由见 Q1：残留分叉为采集消解型，非互斥机制竞争型）。
