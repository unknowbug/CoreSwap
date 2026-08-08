# Phase 8 — 「-288 含水层水体差异 = 矿井 Beardifier 负密度修正缺失」假设验证

> 角色：recode.scout（勘探，只读）
> 任务：验证假设「-288 区域含水层（stone→water / deepslate→water，~5051 块）的水体差异 = 矿井（mineshaft）结构的 Beardifier 负密度修正缺失」——C++ 无 StructureWeightSampler，Java 在矿井附近用 Beardifier 负修正拉低 density → 判水。
> 数据源：`m288_run1.txt`（全量 MISMATCH）、`m288_pair_counts.txt`、`noiseblk_run2.txt`（NOISE 阶段 4 列）、`vanilla_density_overworld_c-18_-15_b10_0.txt`（Java finalDensity）、`dump_x-278_z-240.txt`（C++ 剖面）、`blocks.json`、phase2/6/7 产物
> 日期：phase7 定案之后
> 置信度约定：**【确定】** = 坐标/计数/块 ID 直接可证；**【推测】** = 机制归因 / 数量估算

---

## 0. 一页摘要（TL;DR）

| # | 结论 | 置信度 |
|---|---|---|
| 1 | **-288 区域不存在矿井结构**。深部 y=-15~-19 的 cobblestone 结构实为**地牢（Dungeon，FEATURE）**：m288_run1.txt 深部含 **spawner(175)=1**（(-255,-18,-205)）+ **mossy_cobblestone(169)=46** + **chest(177)=2** + **cobblestone(12)=96**；而矿井特征块 **rail(197)/cobweb(122)/oak_log(46)/oak_planks(13)/oak_fence(254) 在深部零命中**（46/13/254 全部在村庄/沉船 y=62-69 层） | 确定 |
| 2 | **地牢是 FEATURE，不产生 Beardifier 权重**（Beardifier 只对 structure piece 生效）→ 深部结构完全不能作为「Beardifier 负修正」来源 | 确定（机制）/ 推测（归属） |
| 3 | **任务假设不成立**：(-278,-240) 列及含水层 C1/C2 距所有参与 Beardifier 的结构（村庄 y=62-70、沉船 y=63-68）**垂直距离 44-73 格 > 24**；距地牢最近也 26-39 格（且地牢不参与）。**Beardifier=0，含水层判水不可能由 Beardifier 负修正解释** | 确定（坐标计算） |
| 4 | **核心矛盾（density>0 却判 water）另有机制**：(-278,-240) 列 y=15-19 Java NOISE 判 water 且 y=23 是 air（洞穴夹层），Java finalDensity 全正（+0.0545/+0.0685/+0.0765/+0.0683）→ 含水层液体判定不是 `finalDensity>0 → solid` 的简单映射，需回查 Java `Aquifer` 内部（含水层 pocket 的独立形状噪声可覆盖 density>0 区域） | 确定（数据矛盾）/ 推测（替代机制） |
| 5 | **影响面**：含水层 5051 块（stone→water 4416 + deepslate→water 635）中 **Beardifier 可解释 = 0 块**；「Beardifier 负修正」对 -288 含水层**无修复收益**。phase6「结构 24 格内正向 water→solid 差异 ≈2000-4000 块」的 Beardifier 修复路径仍可保留（针对结构附近的 C++ water ↔ vanilla 实心），但与含水层无关 | 确定（0 块）/ 推测（数量） |

---

## 1. 矿井定位：-288 区域无矿井，深部结构实为地牢【确定】

### 1.1 任务预设 vs 数据事实

任务预设矿井特征块「橡木支撑柱（oak_log=46）+ 木板（oak_planks=13）+ 栅栏（oak_fence=254）+ 铁轨（rail=197）+ 蜘蛛网（cobweb=122）」应在 mismatch 深部出现（C++ 生成 stone/deepslate vs vanilla 结构块）。

**实测（m288_pair_counts.txt 全量计数）**：

| 块 ID | 名称 | 全量计数 | 深部（y<0）命中 | 说明 |
|---|---|---|---|---|
| 175 | spawner | **1** | 1（(-255,-18,-205)） | 地牢标志，矿井无 spawner |
| 169 | mossy_cobblestone | 46+18+5=69 | **46**（chunk(-17,-14)/(-16,-14)/(-17,-13)/(-16,-13) y=-19） | 地牢墙标志，矿井无 |
| 177 | chest | 2+2=4 | **2**（(-253,-18,-206)/(-253,-18,-203)） | 地牢 chest |
| 12 | cobblestone | 146+17+23+8=194 | **96**（y=-19~-15 一圈） | 地牢墙 |
| 197/119/120/423 | rail 系列 | **0** | 0 | 矿井必有，无 → 无矿井 |
| 122 | cobweb | **0** | 0 | 矿井必有，无 → 无矿井 |
| 46 | oak_log | 125+2=127 | **0**（全部在村庄/沉船 y=62-69，got=0 vanilla=46） | 矿井支柱缺失 |
| 13 | oak_planks | 139+64+14+4=221 | **0**（全部 y=62-69 村庄/沉船/水下木构） | 矿井地板缺失 |
| 254 | oak_fence | 46+6=52 | **0**（全部 y=62-68 村庄） | 矿井栅栏缺失 |

**判定**：深部结构 = **单个地牢（Dungeon）**，范围 x∈[-258,-252]、z∈[-209,-201]、y∈[-19,-15]（7×9×4 房间：y=-19 为 mossy_cobblestone 屋顶，y=-18~-15 为 cobblestone 墙，中心 (-255,-18,-205) 为 spawner，(-253,-18,-206)/(-253,-18,-203) 为 chest）。

### 1.2 地牢 = FEATURE，不参与 Beardifier【确定机制 / 推测归属】

- `StructureWeightSampler`（Beardifier）只对 **structure**（StructureStart + pieces + jigsaw junctions）产生权重；村庄、沉船、矿井、要塞等是 structure。
- **地牢（DungeonFeature）是 FEATURE（与紫晶洞、树、矿脉同级），生成于块填充后，不进入 populateNoise 的 `add(finalDensity, Beardifier)`** → 地牢 24 格内的 Beardifier 恒 0。
- phase6 已确认：Java 块生成路径 Beardifier = `StructureWeightSampler.createStructureWeightSampler(...)`（ChunkNoiseSampler.java L102-111），其 piece 集来自 structure start；地牢不在其中【推测，依 phase6 源码链】。

### 1.3 -288 区域参与 Beardifier 的结构清单

| 结构 | 类型 | 参与 Beardifier | 范围 | y | 来源 |
|---|---|---|---|---|---|
| 村庄（plains） | village | ✓ | x∈[-275,-233], z∈[-224,-193] | 62-70 | phase7【确定】 |
| 沉船 A | shipwreck | ✓ | x∈[-272,-257], z∈[-220,-212] | 63-68 | phase7【确定】 |
| 沉船 B | shipwreck | ✓ | x∈[-256,-248], z∈[-220,-214] | 63-68 | phase7【确定】 |
| 沉船 C | shipwreck | ✓ | chunk(-18,-13) x∈[-288,-284], z≈-197 | 62 | phase7【确定】 |
| **地牢** | dungeon | **✗（FEATURE）** | x∈[-258,-252], z∈[-209,-201] | -19~-15 | 本报告【确定】 |
| 紫晶洞 | geode | ✗（FEATURE） | x∈[-260,-251], z∈[-197,-193] | -19~-28 | phase7【确定】 |

---

## 2. 距离验证【确定】

### 2.1 核心矛盾列 (-278,-240)（y=15-19 含水层）

到各候选结构/结构的切比雪夫距离（取最近结构块）：

| 候选 | 最近点 | dx | dy | dz | 切比雪夫 | ≤24？ |
|---|---|---|---|---|---|---|
| 村庄 | (-275,62,-224) | 3 | 47 | 16 | **47** | 否 |
| 沉船 A | (-272,63,-212) | 6 | 48 | 28 | **48** | 否 |
| 沉船 B | (-256,63,-214) | 22 | 48 | 26 | **48** | 否 |
| 地牢（若不参与也已超） | (-258,-15,-201) | 20 | 30 | 39 | **39** | 否 |

→ (-278,-240) 距任何结构 >24 格，**Beardifier=0**。【确定】

### 2.2 含水层 C1（stone→water，4416 块，y=11-23）

分布（phase7）：chunk(-18,-14) y=11-18、chunk(-18,-15) y=15-16、chunk(-17,-15) y=17-23、chunk(-16,-15) y=1-6。

垂直距离主导：
| 含水层区域 | 到村庄/沉船（y=62-70）dy | 到地牢（y=-15~-19）dy | 结论 |
|---|---|---|---|
| chunk(-18,-14)/(-18,-15)/(-17,-15) y≥11 | 39-59 >24 | 26-38 >24 | 全部 >24【确定】 |
| chunk(-16,-15) y=1-6 | 56-69 >24 | 16-25（部分 ≤24，但地牢不参与） | Beardifier=0【确定】 |

→ C1 **0 块**在参与 Beardifier 的结构 24 格内。【确定】

### 2.3 含水层 C2（deepslate→water，635 块，y=-3~6）

分布（phase7）：chunk(-18,-15) y=-3~0、chunk(-16,-15) y=-2~6、chunk(-15,-15) y=-2~4。

| 区域 | 到村庄/沉船 dy | 到地牢 dy | 结论 |
|---|---|---|---|
| 全部 | 56-73 >24 | 9-25（靠 x∈[-264,-241] z∈[-233,-225] 的部分 ≤24） | 地牢不参与 Beardifier → Beardifier=0【确定】 |

→ C2 **0 块**在参与 Beardifier 的结构 24 格内。【确定】

### 2.4 距离结论

**含水层 5051 块（4416+635）中，Beardifier 可解释 = 0 块**。任务假设的「矿井 Beardifier 负修正」在 -288 区域无作用对象（无矿井），且即使把地牢误当结构，C1 主体（y≥11）也距地牢 >24 格。【确定】

---

## 3. 假设验证：不成立【确定】

### 3.1 假设链逐环节裁定

| # | 假设环节 | 裁定 | 证据 |
|---|---|---|---|
| (a) | 候选结构 = 矿井（y=-15~-19，含 oak 支柱/rail/cobweb） | **推翻** | 深部无 rail/cobweb/oak_log/oak_planks；实为地牢（spawner/mossy_cobblestone）【确定】 |
| (b) | 矿井是 structure，参与 Beardifier | 不适用（无矿井） | — |
| (c) | 含水层在矿井 24 格内（Beardifier 非零） | **推翻** | C1 全部距结构 >24；(-278,-240) 距一切结构 39-48 格【确定】 |
| (d) | Beardifier 负修正把 Java density 拉低 → 判 water | 不成立（权重恒 0） | — |
| (e) | C++ 无 Beardifier → density 保持正 → 判 solid → mismatch | 部分成立（C++ 确判 solid、Java 判 water），但**归因错误** | C++ finalDensity 全正（dump L1174-1177）；差异机制不在 Beardifier【确定】 |

### 3.2 与 phase6 Beardifier 结论的关系【重要澄清】

- phase6 的 Beardifier 缺失结论针对 **(-244,58,-256) 岛**：Java 判 solid、finalDensity=-0.0744 负 → 需要 Beardifier **正贡献**（+0.08~+0.12）抬 density 判实心。该位置在村庄/沉船 Beardifier 24 格内（y≈58 vs 村庄 y=62-70，dy≈4-12 ✓）。
- 本次任务对象 (-278,-240) 方向**相反**：Java 判 **water**、finalDensity **全正** → 需要 Beardifier **负贡献**。但该位置距一切结构 >24 格，Beardifier=0 → **phase6 的机制不能直接迁移到含水层**。
- 结论：Beardifier 缺失确实存在（phase6，结构 24 格内正贡献），但**不能解释含水层**（结构 24 格外）。

### 3.3 核心矛盾的替代解释方向【推测，供主会话】

(-278,-240) 列 NOISE 剖面（noiseblk_run2.txt L286-295）：y=15-19 water、y=20-22 stone、y=23 **air**、y=24-29 stone。Java finalDensity 全正。若 aquifer 判定是 `density>0 → solid`，则 y=23 也应为 stone——但实为 air，说明**该列存在洞穴（carver/cheese cave），且含水层是洞穴+液面系统产物**，不是纯 `finalDensity` 判据。

候选机制（需 Java `Aquifer` 源码确认）：
1. **Aquifer pocket 独立形状噪声**：1.18+ 含水层由各向异性噪声定义形状（`Aquifer` 的 boundary/flooded 场），pocket 内部可覆盖 density>0 的实心区，判为水/空气（y=15-19 water + y=23 air = 同一空洞带的液面上下）【推测】。
2. **洞穴密度修正**：NoiseChunk 的洞穴密度（`NoiseCave`/`Aquifer` 前处理）会在 `finalDensity` 上叠加洞穴场，使 y=15-23 实际空洞（但探针 vanilla_density 文件采的是 finalDensity，未含洞穴层）【推测】。
3. **探针路径差异**：vanilla_density 文件若来自高度图路径（sampleHeightmap，Beardifier=INSTANCE 恒 0），与块生成路径的 aquifer 输入（含 StructureWeightSampler，但此处=0）一致——因此**探针与块生成在此列 Beardifier 均 0**，无法解释判水【确定探针语义 / 推测归因】。

→ 修复方向应转向 **C++/Java 含水层（Aquifer）液面与洞穴系统对比**（phase7 已列「含水层液面逐列对比」为优先项），而非 Beardifier。

---

## 4. 影响面重估【确定 + 推测】

### 4.1 含水层 5051 块归属

| pair | 数量 | 距参与 Beardifier 结构 ≤24？ | Beardifier 可解释 | 归属 |
|---|---|---|---|---|
| stone→water | 4416 | 0 块 | **0** | 真差异（含水层/洞穴机制）【确定】 |
| deepslate→water | 635 | 0 块 | **0** | 真差异（含水层/洞穴机制）【确定】 |
| 合计 | 5051 | 0 | **0** | 真差异 |

### 4.2 -288 差异构成的修正（相对 phase7）

phase7 已估「Beardifier 可解释水体边界 ≈2000-4000 块（结构 24 格内正向 water→solid）」。本报告**不改变该估计**（那些是结构附近的 C++ water ↔ vanilla 实心，方向与 phase6 岛一致，仍是 Beardifier 候选）；**但明确排除**：含水层 5051 块不是 Beardifier 候选，Beardifier 修复对其**零收益**。

- 结构/FEATURE 假 diff（村庄/沉船结构块 + 地牢块 + 紫晶洞 + 矿脉 + 植被）≈ 6000-7000【推测】
- Beardifier 缺失（结构 24 内正向水体边界）≈ 2000-4000【推测，phase7】
- **真差异**：含水层 ~5051 + 洞穴 ~5411+160 + 海底边界 ~500-1000 + 浅层岩脉 ~32000 + 表面规则 ~3000 ≈ 80%+【推测】

### 4.3 地牢块单独计入

深部地牢（cobblestone 96 + mossy 46 + chest 2 + spawner 1 + 内部 air/结构相关）≈ 145 块为 **FEATURE 缺失假 diff**（C++ 未生成地牢），与 Beardifier 无关，与含水层无关（地牢在 y=-19~-15，含水层 C1 在 y≥11）【确定计数 / 推测归类】。

---

## 5. 最终定性（为 Phase 4 修复决策提供依据）

1. 【确定】**-288 区域无矿井**；深部 cobblestone 结构是地牢（FEATURE），不参与 Beardifier。
2. 【确定】**「-288 含水层差异 = 矿井 Beardifier 负密度修正缺失」不成立**：含水层 5051 块全部距参与 Beardifier 的结构 >24 格，Beardifier=0。
3. 【确定】**核心矛盾（(-278,-240) finalDensity>0 却判 water，且 y=23 air）是含水层/洞穴系统内部机制**，与 Beardifier 无关。
4. 【决策建议·交主会话裁决】
   - **不要**为「修复 -288 含水层」实现/注入 Beardifier（收益 0）。
   - Beardifier 实现（phase6 §5.3）**保留**，但仅覆盖结构 24 格内的正向 water→solid 差异（预期 2000-4000 块），且需要结构系统前置。
   - **含水层修复应转向 Java `Aquifer` 源码级对比**：确认 `Aquifer.sample` 在 density>0 时是否仍可判水（pocket 形状噪声）、洞穴场叠加方式、液面（fluid level）计算——锚点 (-278,-240) y=15-23（water/stone/air 夹层）+ chunk(-18,-14) y=11-18。

---

## 6. 置信度与边界说明

- 【确定】：块 ID 计数（m288_pair_counts 全量）；深部结构=地牢（spawner/mossy_cobblestone 标志 + 无 rail/cobweb/oak）；含水层分布（phase7 坐标 + noiseblk 列）；(-278,-240) 与 C1/C2 到各结构切比雪夫距离 >24；Java finalDensity 全正 vs NOISE 判水（数据矛盾）；C++ 判 solid（dump 剖面）。
- 【推测】：地牢不参与 Beardifier 的归属（基于 phase6 源码链推理，未直接读 Java StructureWeightSampler 此处 piece 集）；核心矛盾的替代机制（Aquifer pocket/洞穴场）；数量占比估算。
- 本报告只做解读，不修改代码；未运行任何命令（铁律遵守）。
- 遗留问题（交主会话）：Java `Aquifer.sample` 源码路径需人工/后续 phase 读取以确认含水层判水的真实输入；-288 区域是否还有 4×4 之外的矿井结构 piece 靠近含水层（本报告仅覆盖 origin=(-288,-256) 4×4 区域内的结构块证据）。

---

## 附：关键参考数据速查

- 核心矛盾列：(-278,-240) NOISE y=15-19 water / y=20-22 stone / y=23 air / y=24+ stone（noiseblk_run2.txt L286-295）；Java finalDensity y=12/16/20/24 = +0.054475/+0.068530/+0.076488/+0.068347（vanilla_density c-18_-15_b10_0）；C++ finalDensity y=12/16/20 = +0.055723/+0.068692/+0.068530（dump_x-278_z-240 L1175-1177）。
- 含水层 pair：stone→water 4416、deepslate→water 635、dirt→water 9（m288_pair_counts）。
- 地牢：spawner(-255,-18,-205)、chest(-253,-18,-206)/(-253,-18,-203)、cobblestone/mossy_cobblestone y=-19~-15 x∈[-258,-252] z∈[-209,-201]。
- 参与 Beardifier 结构：村庄 y=62-70、沉船 A/B/C y=63-68（phase7）。
