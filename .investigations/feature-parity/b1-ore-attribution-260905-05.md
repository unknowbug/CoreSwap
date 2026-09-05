# b1-ore-attribution-260905-05 — 矿石/替换层 blocks 差异规则层归因定位（core-worker，draft）

> 角色：core-worker（分析解读）。置信度：**全部 draft**（静态代码审读 + 数据 JSON 核对，Degraded——无运行时 probe，主会话可执行判别实验后升级）。
> 边界（用户拍板）：只分析数据/规则/placement modifier 层；若证据指向 populationSeed 随机派生层 → 登记「停手边界」不展开。
> 输入（已廉价验证可继承）：`.tmp/feature-parity-260905-05/verify-handoff-260905-05.md` 签名（diorite 9419 / andesite 8867 / granite 8215 / deepslate 2020，seed 8576294172403134396，655/3377 chunk 有差，Y=-1..-4 深层段 7000+ 实例）+ 两份 scout 产物。
> §9.7 口径声明：本文引用的差异签名 = V1 独立复算口径（全量 3377 chunk palette |Δ| 计数）；与 G2 judge 口径（447 逐块）不可比；判别实验设计默认同 V1 载体。

## 0. 一句话总览

**发现一个新的规则层高嫌疑落点：`HeightProvider`（carver.rs L84-105）不识别 `type: minecraft:trapezoid`，把全部 13 个主 ore 的 trapezoid 高度分布当 uniform 解析**——Y 分布错 + 随机消费次数可能不同（波及同 chunk 后续 feature 序列）。替换层（granite/diorite/andesite）与深层差的多个互斥候选见 §3。

## 1. 逐项映射：差 → Rust 落点

### 1.1 替换层差（granite/diorite/andesite 互增减 ~26k）

数据面事实（本 session 核对）：
- granite/andesite/diorite blob = **ore feature**（configured_feature/ore_granite.json / ore_andesite.json / ore_diorite.json，size 64，target = tag_match `base_stone_overworld`）——不是 ore_vein 产物（ore_vein.rs L31-32 只有 copper→granite / iron→tuff 两族，无 andesite 写入）。
- placed：`ore_granite_upper` / `ore_diorite_upper` / `ore_andesite_upper` = rarity_filter(6) + in_square + height_range **uniform(64..128)** + biome；`*_lower` = count(2) + in_square + height_range **uniform(0..60)** + biome。
- 即：**替换层 blob 的高度分布全是 uniform，不经过 trapezoid 路径**——1.2 的 trapezoid 缺口**不直接**解释 blob 差，但 blob 与主 ore 共享后续候选落点（biome modifier / discard / 随机序列）。
- blob 的 targets = `base_stone_overworld`（含 deepslate）→ blob 在 Y<0 也可能吃掉 deepslate → deepslate 差的合法规则层路径之一。

### 1.2 expand_tag 硬编码 vs vanilla tag 一致性（feature.rs L65-93）

- `stone_ore_replaceables` = stone/granite/diorite/andesite；`deepslate_ore_replaceables` = deepslate/tuff；`base_stone_overworld` = stone/granite/diorite/andesite/tuff/deepslate——与 1.20.1 vanilla `data/minecraft/tags/blocks/` 口径**逐项一致**（来源 = 知识记忆，非本 session 读 server jar；**判别实验见 .b1-5**，升级前保持 draft）。
- 结论：expand_tag **不是**替换层差的高嫌疑落点。@anchor.idk("tag 与 server jar 逐项一致为 memory 源，未做本地 diff 复核")

### 1.3 ore_vein（NOISE 阶段，域①挂起——只登记不动）

- ore_vein.rs L31-32：copper(granite, y 0..50) / iron(tuff, y -60..-8)。**Y=-2..-4 落在 iron vein 域内** → tuff/deepslate_iron_ore 写入差是 deepslate 差的强候选，但属挂起域①（260904-15 拍板），本文只登记不展开。

## 2. 新发现的规则层缺口（本 session 定位）

### 2.1 【高嫌疑】HeightProvider 丢 trapezoid 类型（carver.rs L84-105 + placement.rs L221-224）

- `PlacementModifier::parse` 的 `height_range` 分支（placement.rs L221-224）把 JSON 交给 `HeightProvider::parse`；后者**不看 `type` 字段**，只检查 `min_inclusive`/`max_inclusive` 键 → trapezoid JSON（键名恰好相同）被当 **uniform** 解析。
- 影响面（grep 实测）：placed_feature 中 `type: minecraft:trapezoid` 共 **13 个** = ore_coal_lower / ore_copper / ore_copper_large / ore_diamond*3 / ore_gold / ore_iron_middle / ore_iron_upper / ore_lapis / ore_redstone_lower / ore_emerald / ore_ancient_debris_large——**全部主 ore 高度分布**。
- 双重后果：
  1. **Y 分布差**：Java trapezoid（三角/梯形，峰在中央）vs Rust uniform（平坦）→ ore 在 Y 极值带多放、峰值带少放 → 规则层直接产生 ore↔stone 互增减。
  2. **随机消费次数疑点**：Java TrapezoidHeight 采样疑似消费 2 次 nextInt（uniform 路径消费 1 次）→ 若属实，每个 trapezoid ore 放置后**整个 chunk 后续随机序列偏移**。⚠️ 该后果与挂起域④（populationSeed 1/13 疑点）签名同形——**必须先做判别实验分离，分离前不得把此类残差归入挂起域**。Java 精确公式本 session 未核一手源 @anchor.idk("TrapezoidHeight 消费次数与精确公式需 Java 一手源核对，本产物仅登记疑点")
- 附带发现（低影响面）：`IntProvider::Trapezoid`（placement.rs L32-42）parse 从 JSON 读 `plateau` 键（L79），而 1.20.1 trapezoid JSON **无 plateau 键**（默认 0）→ L40 `f/0.0` **除零**（NaN/i32::MAX 路径）。但 height_range 不走此路径；grep 13 个 trapezoid JSON 全在 height 位置，未发现 IntProvider 级 trapezoid 用户——影响面待 grep placed JSON 的 count 字段复核 @anchor.idk("IntProvider::Trapezoid 除零的实际触发面未全量 grep count 字段")

### 2.2 buried discard 语义差（feature.rs isExposedToAir + apply_features L907-908 None）

- ore_coal_lower → **ore_coal_buried**（discard_chance_on_air_exposure=1.0）；diamond/lapis/gold buried 同族。
- scout 已登记：越界读回 -1 ≈ 非空气（feature.rs L349-350），方向与 Java ChunkSectionCache 相反 → Rust 在 chunk 边界/洞穴邻接处**少 discard**。规则层候选（.b1-3）。

## 3. 互斥候选清单（.b1 编号，各带判别实验；主会话执行，worker 不自下结论）

### .b1-1 HeightProvider trapezoid→uniform（含序列偏移分离）
- 机制：13 个主 ore Y 分布错 + 可能的随机消费次数差。
- 判别实验（可执行）：
  a. **分布判别（不动随机层）**：写 `.tmp/feature-parity-260905-05/trapz_probe.py`——对 `ore_coal_lower` 的 height 用 Java 已知 trapezoid 公式与 Rust uniform 各采样 N=100k（同 nextLong 流先分离、只比 Y 边际分布），对比 V1 差异的 Y 直方图形状：差实例在**峰带（中段）少/极值带多** → 支持 .b1-1。
  b. **序列分离判别**：单 chunk 对拍（WG_FEATURELOG，worldgen_handle.rs L650/880/918 现成钩子）：A/B 两个 Rust 构建（现状 vs 临时改 trapezoid 正确解析，工程修复不计数）各跑同一 chunk；若仅改 Y 解析、随机消费次数改齐后 ore 位置仍 1/13 匹配 → 残差归挂起域（停手）；若匹配率显著上升 → .b1-1 成立为主因。
- 预计判别力：a 便宜（纯静态采样）；b 需主会话构建，判定力强。

### .b1-2 IntProvider::Trapezoid plateau 除零
- 判别：全量 grep 231 placed JSON 的 `count`/`xz_spread`/`y_spread` 字段有无 IntProvider 级 trapezoid；有则对每个构造最小调用打印 get() 结果（NaN/i32::MAX 即触发）。
- 判别力：直接；预计影响面 0~小。

### .b1-3 buried discard 语义（coal_buried 族）
- 判别：取 V1 差异实例中 `coal_ore` 的具体坐标（V1 脚本可扩为记录位置），判 6 邻域空气率；把差异实例分「chunk 边界 3 格内/外」「邻洞穴/不邻洞穴」分层统计：差实例显著富集于边界/洞穴邻接层 → 支持 .b1-3。
- 交叉验证：diamond_buried/lapis_buried/gold_buried 差应同向富集；若只有 coal 差而其他 buried 无差 → 弱化。

### .b1-4 biome 特征集单 chunk vs 3×3 + Biome modifier no-op
- 机制：worldgen_handle.rs L817 单 chunk 角 biome；placement.rs L173-178 biome 过滤直通。对 all-biome 共有 ore（underground_ores 主族）应无影响；只影响 biome 专属 feature（emerald=山地族、extra ore peaks 等）。
- 判别：V1 差异实例按 biome 分组（biome 数据两侧一致为前提，G1 100%）；若差实例在普通 biome 与山地 biome 均匀分布 → 排除 .b1-4；若 emerald/peak 专属差占比异常 → 支持。
- 判别力：对替换层 blob 差**预期排除**（blob 全 biome 共有），仍列以防归因盲区。

### .b1-5 expand_tag 与 server jar tag diff（低嫌疑复核）→ **已执行，排除（260905-05 主会话）**
- 判别结果：feature.rs expand_tag 8 个 tag 展开与 1.20.1 vanilla 权威口径逐项一致（base_stone_overworld=stone/granite/diorite/andesite/tuff/deepslate；stone_ore_replaceables=stone/granite/diorite/andesite；deepslate_ore_replaceables=deepslate/tuff；base_stone_nether=netherrack/basalt/blackstone；sand=+red_sand+suspicious_sand；dirt=9 项含 moss_block/mud/muddy_mangrove_roots）。静态一手核对级（judge J8 要求提前执行，已消除候选链地基风险）；该 §1.2 的 idk 就此收口。
- 判别力：直接；**排除**。

### .b1-4b BiasedToBottom（judge J4 补录，已排除）
- judge 代 grep：全 231 placed JSON 中 `biased_to_bottom` 仅 glowstone_extra + spring_lava_*（very_biased_to_bottom，且 IntProvider::parse 无该分支 → 落「未知 type 静默丢弃」面，随 b2 S2 告警修复覆盖）；**主 ore 族不用** → 排除。

### .b1-6【停手边界登记，不展开】随机派生层 + ore_vein 域①
- 内容：populationSeed/setDecoratorSeed 派生差（挂起域④，1/13 匹配）+ ore_vein vein pre 态与 iron vein tuff/deepslate 写入（域①，Y=-60..-8 覆盖深层段）。
- 处置：.b1-1..b1-5 判别后仍不能解释的残差**归此域收口**；本文不做任何派生层分析（边界遵守声明见 §5）。

## 4. 第 3 问初判：Y=-1..-4 段 7000+ 实例归属

- **Y 语义已裁决（judge J1 + 主会话，supersedes 下面两行的 idk）**：V1 脚本 Y = **section 索引**（`sec["Y"]` 有符号修正），桶宽 16 block Y；「Y=-1..-4」= block Y −64..−1；「Y=3..6」= block Y 48..111。不存在 16 倍单点位移。按族×section 重切（j1_j14_recut.py）结果：**coal_ore 差全部在 section 0..4（block Y ≥0），负 Y 无 coal 差**（下面第 3 条「coal 负 Y 路径」问题消解）；deepslate 差集中 section −1..−4 = block Y −64..−1，落 iron vein 域① + blob targets 合法路径。深层段主混计成分 = diorite/andesite/granite（blob，section −1..3）+ deepslate/clay/moss_block/cave_vines（lush_caves/lake 族，非 ore）。
- 若 Y = block Y（-1..-4）：vanilla coal 数据面 trapezoid min=0 → **Java 在负 Y 产生 coal_ore 的合法路径不存在**；深层段 stone↔coal_ore 差初判：
  1. **ore_iron_middle（trapezoid -24..56，合法负 Y）**差被直方图混计——即 7000+ 或主要属 iron/deepslate_iron_ore 族而非 coal（V1 报表按族分开则此条可立即判）。idk，需按族重切直方图。
  2. **deepslate 2020**：Y=-2..-4 在 iron vein 域（-60..-8）内 → tuff/deepslate_iron_ore 写入差（**域①挂起，登记**）为首要候选；次选 blob（base_stone_overworld 含 deepslate）placement Y 差（.b1-1 间接）。
  3. 若按族重切后确有 coal_ore 负 Y 差：规则层无合法产生路径 → 直接判为**配对标签/统计口径问题或停手边界（.b1-6）**，不做规则层深挖。
- 置信度：初判（draft + idk），全部待 §3 判别实验收口。

## 5. 停手边界声明（用户拍板边界遵守）

- 本文止步于：数据 JSON、tag 展开一致性、placement modifier 解析/语义、discard 规则、biome 特征集归属。**未做**：populationSeed 公式分析、setDecoratorSeed 序列对齐、ore_vein sampler 内部数学（域①）、任何随机派生层修因建议。
- 交叉点提示：.b1-1 的「序列偏移后果」与挂起域④签名同形，判别实验 .b1-1b 是分离两者的唯一手段——分离完成前，ore 位置类残差**不得**单方面归入挂起域（防 M14 式假设当公理）。

## 6. 过程记录（判错经验，按 SUBAGENT-KNOWLEDGE-GUIDE）

- 现象：初读 scout 产物时按其 §4.11 把替换层差候选押在「ore_vein vs surface rule」。定位：核 configured_feature/ore_granite.json 等 3 个 blob 配置后，发现 blob = ore feature（base_stone_overworld target），ore_vein 无 andesite——scout 地图该条为 idk 占位，非结论。教训：**候选落点判定必须以 targets/state JSON 为锚，不沿用上游勘探的开放性问题措辞**。
- 现象：第一版分析以为 height_range 走 placement.rs IntProvider::Trapezoid（除零疑点），影响面看似巨大。定位：读 placement.rs L221-224 确认 height_range 委托 carver.rs HeightProvider（独立实现，无除零但有丢类型问题）。教训：**同名概念（trapezoid）在本仓库有两套实现（IntProvider vs HeightProvider/FloatProvider），定位影响面前必须先确认调用路径**。

## 7. 产物自检（GUIDE §四）

- [x] 价值门：本文 = 过程性中间产物（.investigations/ 载体，正确）；无直接写 docs/discovered。
- [x] 所有候选互斥列出、判别实验可由主会话执行、无自下结论。
- [x] 停手边界显式声明；挂起域只登记。
- [x] 不确定处全部标 @anchor.idk；置信度全文 draft。
- [x] 只读分析，未改任何代码。
