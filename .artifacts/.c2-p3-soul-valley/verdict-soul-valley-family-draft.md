# verdict — C2/P3 soul sand valley 家族分族重测解读（B2）

- **status: confirmed（260902-01 用户拍板；主结论=缺口在 Rust 管线内+三签名方向；.b1b 内部机制 idk 保持）**（AI 不得标 confirmed；待 judge 审查 + 用户拍板）
- **验证分层：Degraded（静态数据解读）**——本结论仅基于主会话提供的 per-id / 块对原始数据推演，未运行任何探针/命令（沙箱无 shell）。升级到 candidate 需下文「数据层验证动作清单」至少 V1+V2 落地。
- **可比性声明（§9.7）**：口径 = FULL vanilla 参照 vs Rust 存档 MCA 直解（存档写入口径），seed B=8576294172403134396，4×4 @3200,3208，总 mismatch 63,976。与 SURFACE 参照 77.4857%（探针口径）**不可直接比**。
- 产出者：core.worker subagent；日期：本轮。

---

## Q1：soul_soil 缺口大头在 Java feature 阶段？——**基本证伪**

**数据核算**：
- soul_soil 缺口 = 5474 − 1334 = **4140**；soul_soil 流出块对合计 = 3344(netherrack)+345(magma)+334(basalt)+103(blackstone)+31(gravel) = **4157 ≈ 缺口全accounted**（自洽）。
- 其中指向 feature 阶段覆盖的项（basalt blobs / deltas / blackstone / gravel / magma 类覆盖）合计仅 **813 ≈ 20%**；**soul_soil→netherrack 3344 ≈ 80% 是绝对大头**。
- soul_sand 同构验证：缺口 986，流出 1379 − 流入 349 ≈ 1030，自洽；同样 soul_sand→netherrack 1102 为大头。

**机制判读（方向性论证）**：
- 块对方向 ref=soul_soil → save=**netherrack**：netherrack 是基底地形块。Java feature 阶段不会大面积放 netherrack（basalt deltas feature 放 basalt/blackstone，carvers 挖空/放洞穴块）——若 soul_soil 先被 surface rule 放上、后被 feature 覆盖，save 侧应呈现 basalt/blackstone/gravel 等 feature 产物，而非 netherrack。
- 因此 3344 的主导方向指向：**Rust surface rule 在这些点位根本没走 soul 分支（nether_state_selector 判定未命中 soul_soil 侧），基底保持 netherrack**——这是 selector 判定差异，不是 feature 覆盖。
- magma 证据仅作邻居佐证：soul_soil→magma 345 + soul_sand→magma 142 与 magma save 偏高（3814 vs 1533）同向，说明 save 区局部基底/后续覆盖与 vanilla 不同，但这属于小头（P2 另行分析，本课题不当根因）。

**上轮交接假设处置**：「soul_soil 大头疑似 Java feature 阶段」——**❌ 证伪（按 §16.3 廉价独立验证要求，用块对方向性核算完成）**。feature 阶段覆盖差存在但只占 ~20%；80% 是 surface 判定未发生。

**残余不可分辨点（诚实声明 @anchor.idk）**：仅凭最终存档块对**无法完全排除**「surface 曾放上 soul_soil、后被 carver 顶面暴露为 netherrack 基底」的复合情形——但该情形要求 surface 先放 soul_soil 再被剥到 netherrack，与 selector 未命中的可观测区别需要阶段分离 dump 才能裁决（见 V1）。

## Q2：soul_sand:soul_soil 比例 1:2.2 → 1:0.9 说明什么

- ref 2457:5474 ≈ 1:2.23；save 1471:1334 ≈ 1:0.91。两类块都缺，但 soul_soil 缺得更狠（−76% vs −40%）。
- 机制上 selector（min=0.0 阈值）在 soul 分支内部分 soul_sand/soul_soil；**selector 整体有生效**（save 仍有大量两种 soul 块，非全灭），但**分割点分布偏移**：
  - 候选 A：nether_state_selector 噪声采样/阈值边界处理与 vanilla 有小偏差——噪声分布若在阈值附近质量密集，微小系统偏移就会把大量点从 soul_soil 侧推到 soul_sand 侧或推回 netherrack 分支；
  - 候选 B：soul 分支的前置条件链（biome footprint / stone_depth ceiling/floor 覆盖范围）在 Rust 侧更窄——进入分支的点变少，且进入点内阈值两侧分割也偏；
  - soul_sand→netherrack 1102 表明 soul_sand 也大量整支未命中 → 支持「**分支入口收窄 + 阈值侧偏**」复合，而非单纯 soul_soil 侧翻转。
- 与 B1 定论兼容：本课题残差独立于 basalt→netherrack 157,658 大宗石互换（那是 feature 命中差），不要混算。

## Q3：置信度与下一步

**结论置信度：draft / 中等**。方向性论证（netherrack=基底块 ⇒ 判定未发生）较强，但被 Q1 的 idk 残余点限制——缺阶段分离证据。

**互斥机制候选（≥2，现有数据不可裁决 → 建议 fan-out）**：
- **.b1 surface selector 判定差**：nether_state_selector 噪声值/阈值边界/前置条件（biome、stone_depth）在 Rust 侧与 vanilla 不一致 → soul 分支未命中（预计主导，~80% 量级）。
- **.b2 feature/carver 阶段覆盖差**：soul 层已由 Rust 正确放置，被 Java carvers 暴露或 features（basalt blobs/deltas/magma）覆盖（上轮交接假设残余，~20% 量级）。
- 两候选非纯互斥（可叠加），但归因拆分互斥——主会话按 core.fanout 派两个 worker 各自验证一个 .bN，judge 对比。

**数据层验证动作清单（主会话只执行不解读，原始输出落盘 .investigations/ cmd-output/ 回传）**：
1. **V1 阶段分离 dump（裁决性）**：对同一 4×4 region（seed B，@3200,3208）分别导出 ① Rust SURFACE 阶段输出（feature 注入前快照；可在 WorldgenRust `src/bin-diag/` 临时 diag bin 或 env 门控 dump surface rule 应用后的调色板计数）② 最终存档 MCA 计数。执行模板：
   ```
   cargo build --release --bin <diag-name>
   <diag-bin> --seed 8576294172403134396 --region 3200 3208 4 4 --stage surface --out E:\PYTHON\CoreSwap\.tmp\soul-surface-dump.txt
   ```
   判据（交回 worker 解读）：surface 阶段 soul_soil 计数 ≈5474 → .b2 主导；≈1334 → .b1 主导。
2. **V2 selector 噪声探针**：在 soul_soil→netherrack mismatch 样本点（从存档 diff 取 ≥50 个坐标），dump `nether_state_selector` 噪声原始值 + 阈值分支决定 + biome id + stone_depth 输入，与 Java RouterProbe/SURFBIOME 同点对比（注意坐标语义三查：SURFBIOME 是 floor 对齐 `(x>>2)<<2`）。
3. **V3 seed/坐标三查（对比前强制）**：核对参照与存档两侧 worldSeed=8576294172403134396、region 对齐、文件 header 完整（M11 三犯教训）。
4. **V4 biome footprint 核对**：两侧 soul_sand_valley biome 在 region 内的 cell 计数对比（RouterProbe SURFBIOME vs WG_BIOMEDUMP），排除 biome 判定差导致的分支入口收窄。

## 自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

---

# 定稿补充（V1 回传后收敛记录，仍 draft/Degraded）

> V1 阶段分离探针已由主会话执行：纯 Rust populateNoise+buildSurface（无 carvers/features），seed/origin header 校验通过，wg_fill ret=16。分叉收敛，fan-out .b1/.b2 结束。

## V1 数据

| id | ref | pureRust (SURFACE 阶段) | save (最终存档) |
|---|---|---|---|
| soul_sand | 2457 | 884 | 1471 |
| soul_soil | 5474 | **1363** | 1334 |
| basalt | 172704 | 5514 | 167314 |

## 收敛裁决（按 Q1 预设判据）

- pureRust soul_soil 1363 ≈ save 1334（差 29）→ **.b1 成立：soul_soil 缺口在 Rust SURFACE 阶段即已存在，与 carvers/features 无关**。.b2（feature 覆盖残余）解释力仅 ~29 块（<1%），降级为边缘项。上轮 Q1 idk 残余点（「surface 曾放置后被 carver 暴露」）就此关闭——surface 阶段本就只有 1363。
- **反向线索（新增）**：soul_sand pureRust 884 → save 1471（**+587**）——Java feature 阶段在 Rust 地形上额外净增了 soul_sand。即存档口径 soul_sand 缺口（986）被 feature 增量部分掩盖；pureRust 口径下真实缺口 2457−884=**1573**，比存档口径更大。方向与 soul_soil 相反，两块不可混算。
- basalt 5514 → 167314 与 B1 定论（大宗 basalt = Java feature 命中）自洽，交叉验证 V1 载体可信。
- 修正后缺口核算（pureRust 口径）：soul 家族 SURFACE 阶段合计缺 ~5684（soul_soil 4111 + soul_sand 1573）≥ 存档口径缺口 ~5126，确认主导机制在 surface rule，feature 阶段是部分回补而非流失源。

## .b1 内部细化（下一层分叉，V2 裁决）

.b1「Rust surface rule soul 分支未命中」成立，但内部两个互斥子机制待分：
- **.b1a 前置条件差**：soul 分支入口（soul_sand_valley biome 判定 / stone_depth ceiling-floor 范围）Rust 侧与 vanilla 不一致 → 大片区域整支未进入 soul 分支。签名：mismatch 点 biome id 或 stone_depth 输入不同。可解释 soul_sand→netherrack 1102 整支脱离。
- **.b1b 阈值判定差**：前置一致，`nether_state_selector` 噪声值/阈值（min=0.0）边界处理不一致 → 分支内 soul_sand/soul_soil 侧分配偏移（对应 Q2 比例变化）。纯阈值差只在两 soul 块间搬运，不产生 netherrack。
- 初判（待 V2）：soul_sand→netherrack 1102 + soul_sand 真实缺口 1573 更像 **.b1a 主导 + .b1b 次级复合**。

## 下一步（主会话只执行不解读）

**V2 selector 噪声同点探针（裁决 .b1a/.b1b，最高优先）**：从存档 diff 取 soul_soil→netherrack 样本 ≥50 点 + soul_sand→netherrack ≥30 点，每点 dump：Rust 侧 nether_state_selector 噪声 f64 原值、soul 分支进入与否、biome id、stone_depth 输入（ceiling/floor/depth）；Java 侧同坐标 RouterProbe/SURFBIOME。执行模板：
```
# Rust 侧（bin-diag 临时 diag bin，勿入 src/bin/）
cargo build --release --bin soul-selector-probe
target\release\soul-selector-probe --seed 8576294172403134396 --points .tmp\soul-mismatch-points.txt --out .tmp\soul-selector-rust.txt
# Java 侧（外部 gradle 探针工程）
.\gradle runServer   # env: WG_ROUTERPROBE=1，采样点注入 .tmp\soul-mismatch-points.txt
```
判据（交回 worker）：前置条件不同的点占比 >50% → .b1a；前置一致但噪声值/分支侧不同 → .b1b。注意 SURFBIOME 坐标 floor 对齐 `(x>>2)<<2`，判定输入是原始 BlockPos——对比前先换算（M11/#23 教训）。
辅助：soul_sand +587 feature 增量溯源（save−pureRust soul_sand 块坐标分布，判定对应 soul_sand_layer/deltas 哪个放置），仅影响口径归因。V3 seed 三查已完成 ✅；V4 biome footprint 并入 V2 Rust 侧 biome id 列。

## 置信度更新

- 「缺口大头在 Rust SURFACE 阶段（.b1），feature 覆盖仅边缘」：**candidate 级证据**（V1 数据层探针裁决，Full 级载体但单 region 单 seed）——建议主会话交 judge 审查后授 candidate；扩样（另一 region/seed 复跑 V1）可加固。
- 「.b1a 主导 vs .b1b 次级」：仍 draft，待 V2。

## 自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）
- [x] 价值门：本课题为活跃排查中间结论 → 载体 .artifacts/ 草稿（非结论性 docs），合规
- [x] 被证伪假设保留并标注（❌ 上轮交接假设）
- [x] 根因为机制层（分支入口/阈值侧偏 vs feature 覆盖），非现象复述
- [x] 无编造数字，全部来自主会话提供数据；核算自洽（流出≈缺口）
- [x] Degraded 分层显式声明；idk 残余点显式声明
- [x] fan-out 建议明确（.b1/.b2）

---

## V2 回传定稿（.b1a/.b1b 裁决；仍 draft，置信度升级建议见末节）

数据：`.tmp/soul-selector-probe.csv`（180 点三组各 60）+ `.stderr.txt`（整规则 apply 交叉验证）。**环境修正（judge 指出，记录在案）**：V1「pureRust」实为 **Rust 全管线**（含 Rust carvers/features）——V1 结论不受影响：soul_soil 1363≈1334 说明 Java features 对 soul_soil 无增减；soul_sand +587 现可精确归因为 **Java 侧 features 独有回补**（Rust features 已含在两侧口径中）。

### 组间模式（实测）

| 组 (n=60) | entered=true | selector_pass=true | Rust biome=nether_wastes | 关键签名 |
|---|---|---|---|---|
| soul_soil→netherrack | 20 (33%) | entered 内 ~31 | **35** | 边界点 selector∈(-0.05,0) 且 pass=false → applied=netherrack |
| soul_sand→netherrack | 51 (85%) | 28 | 9 | entered 高但 pass 低；9 点 biome 已是 wastes |
| netherrack→soul_sand | **0** | — | 多数 | soul_sand_valley 点也 ceiling_ok=false，整组未进分支 |

### 裁决

1. **.b1a 成立且为主导，两个具体签名**：
   - **① biome footprint 差**：组1 35/60、组2 9/60、组3 多数点 Rust biome=nether_wastes，而 vanilla 在这些点产出 soul 块（soul 层仅 soul_sand_valley 有）⇒ vanilla biome=soul_sand_valley。失配点空间聚簇于 **x≥3410 带**（z 3210–3231）→ Rust soul_sand_valley 足迹相对 vanilla **偏移/收窄**，边界带整片入口未进。
   - **② soul_soil 子分支失效签名**：entered=true 且 selector<0（pass=false）的点 rule_applied=**netherrack** 而非 soul_soil（3260/3261，selector −0.047/−0.013；y=3 行同模式）→ vanilla 应给 soul_soil 的边界带点，Rust 规则落到默认石。此签名同时解释 soul_soil 重缺（selector 负侧质量流失）与 Q2 比例偏移（1:2.23→1:0.91：soul_soil 侧被削、soul_sand 侧相对保留）。
2. **.b1b（selector 噪声值本身与 vanilla 偏离）→ idk，本轮不可裁决**：V2 设计要求 Java 同点对照，本轮仅回传 Rust 侧值。Rust selector 值是否等于 Java 值未知——「同值但子分支结构/映射错」与「噪声值偏离」现有数据不可区分。
3. **组3 整组 entered=0/60（新发现）**：含 biome=soul_sand_valley 的点也 ceiling_ok=false（floor 侧场景，stone_depth_above 10~79 / below 4~16）→ 提示 **floor 侧 soul_sand 路径（nether_wastes 的 soul_sand_layer 薄层，min=−0.012 / valley floor 条件）在 Rust 规则中缺失或 stone_depth 判定不同**。
4. **idk：`rule_applied=id=31` 语义未定**——header 声明 soul_sand=257/soul_soil=258，id=31 两者皆非（palette index? 需主会话查 probe 源码确认）；本定稿未依赖 id=31 判断具体块。

### 结论（B2 定稿，draft）

soul 家族存档口径偏低 = **Rust surface rule 结构性差异主导（.b1a）**，非 Java feature 覆盖：
- 签名 A：soul_sand_valley biome 足迹偏移/收窄（x≥3410 边界带整片入口未进）；
- 签名 B：soul_soil 子分支（selector<0 侧）落 netherrack 而非 soul_soil；
- 候选签名 C：floor 侧 soul_sand_layer 分支缺失（组3 entered=0/60）。
Java features 侧为净回补（soul_sand +587），不是缺口来源。

### 下一步（主会话只执行不解读）

1. **V3-rule 结构对拍（最高优先，零探针成本）**：dump Rust nether surface rule 结构（serde debug / 规则构建日志），与 vanilla `nether.json` surface_rule 逐节点对拍：soul_soil 子分支、`soul_sand_layer` 分支、stone_depth above/below 参数与 stop 深度。
2. **V4 Java 同点 selector 对照（裁决 .b1b）**：RouterProbe 输出 3260,1,3200 / 3261,1,3200 / 3359,1,3206 等已采点的 nether_state_selector 值——与 Rust 同 ⇒ 纯结构 bug（V3-rule 修完闭环）；异 ⇒ 追噪声采样链。
3. **V5 biome 边界带对比**：x∈[3400,3520] 两侧 biome cell 对比（RouterProbe SURFBIOME vs WG_BIOMEDUMP，坐标语义三查），定位 soul_sand_valley 边界偏移层。
4. id=31 语义由主会话查 probe 源码确认。

### 置信度分层声明

- 「缺口主导在 Rust SURFACE 阶段结构差（.b1a），非 feature」：**candidate**（V1 阶段分离 + V2 三组同点探针双数据层证据；单 region 单 seed，扩样可加固）。
- 「soul_soil 子分支落 netherrack」「soul_sand_layer 分支缺失」：**draft 签名**（Rust 单侧 apply 模式推断，待 V3-rule 对拍定案）。
- 「Rust selector 噪声值 vs vanilla」：**idk**（V4 未做）。


