# -288 区域 Beardifier 假设影响面验证（Phase 7）

> 角色：recode.scout（勘探，只读）
> 任务：验证「-288 水体边界 mismatch 是否全部聚簇在结构（村庄/沉船/矿井）附近」——判定「-288 差异 = 结构相关假 diff（Beardifier 缺失）」是否成立，还是存在远离结构的真 density/surface 差异。
> 数据源：`m288_run1.txt`（全量 MISMATCH 行）、`m288_pair_counts.txt`、`m288_chunk_counts.txt`、`blocks.json`、phase2/6 产物
> 日期：本次分析（phase6 定案之后）
> 置信度约定：**【确定】** = 坐标/计数直接可证；**【推测】** = 机制归因 / 估算比例

---

## 0. 一页摘要（TL;DR）

| 结论 | 置信度 |
|---|---|
| **「-288 差异全部 = 结构相关假 diff（Beardifier 缺失）」不成立** | 确定 |
| 最大反例 = **含水层**（stone→water 4416 块 + deepslate→water 635 块 ≈ 5051 块）：方向为「C++ 实心、vanilla 水」，**与 Beardifier 缺失方向相反**，且 y=11-23 深部远离所有结构（垂直距离 >24 格）→ **真差异，与 Beardifier 无关** | 确定（方向/位置）/ 推测（机制） |
| **chunk(-16,-16) 水体边界 364 块中约一半（z∈[-256,-249]，≈180 块）距最近村庄 25-32 格 > 24** → 非 Beardifier | 确定（坐标）/ 推测（比例） |
| **chunk(-15,-16) 水体边界部分约一半（z∈[-256,-249]）距村庄 >24 格** → 非 Beardifier | 推测（估算比例） |
| Beardifier 候选只覆盖**结构 24 格内**的「C++ water ↔ vanilla 实心」正向差异：沉船附近（chunk(-17,-14)/(-16,-14)）、村庄西缘（chunk(-18,-13)）等——是全部水体边界的一个子集 | 推测 |
| **-288 差异构成**：结构相关（Beardifier 缺失 + 结构/FEATURE 块缺失）≈ 30-40%；真 density/surface/含水层差异 ≈ 60-70%（含水层 ~5051 + 远离结构的海底边界 ~1000+ 洞穴 ~5400 等） | 推测 |

---

## 1. 结构块位置清单【确定】（vanilla 侧结构/FEATURE 块）

> 来源：`m288_run1.txt` 中 vanilla=结构特征 ID 的行（oak_log=46 / oak_planks=13 / oak_stairs=176 / oak_fence=254 / cobblestone=12 / mossy_cobblestone=169 / chest=177 / dirt_path=602 / farmland=184 / amethyst=903 等）。

| 结构 | 类型 | 位置（世界坐标） | y 层 | 判断 |
|---|---|---|---|---|
| **村庄**（plains） | village | x∈[-275,-233], z∈[-224,-193]，含 dirt_path/oak_log/oak_planks/oak_stairs/cobblestone/farmland/hay/wool/bed/bell 等 | 62-70 | 确定，1 个平原村庄，跨 chunk(-18..-15,-14..-13) |
| **沉船 A** | shipwreck | chunk(-17,-14) cold_ocean：x∈[-272,-257], z∈[-220,-212]（oak_log/oak_stairs/cobblestone） | 63-68 | 确定 |
| **沉船 B** | shipwreck | chunk(-16,-14) cold_ocean：x∈[-256,-248], z∈[-220,-214]（oak_log/oak_stairs/cobblestone） | 63-68 | 确定 |
| **沉船 C / 水下木构** | shipwreck? | chunk(-18,-13) cold_ocean：x∈[-288,-284], z=-197（oak_planks=13） | 62 | 确定存在橡木板水下结构 |
| **矿井（mineshaft）** | mineshaft | chunk(-17,-14)/(-16,-14)/(-17,-13)/(-16,-13) 交界：cobblestone 支柱 + chest(-253,-18,-206)/(-253,-18,-203) | -15~-19 | 确定，深板岩层废弃矿井 |
| **紫晶洞** | geode（FEATURE，**非 structure**，不参与 Beardifier） | chunk(-17,-13)/(-16,-13)：x∈[-260,-251], z∈[-197,-193] | -19~-28 | 确定 |
| **树** | tree（FEATURE） | chunk(-15,-13) beach：x=-230, z=-207 | 68-72 | 确定（单株） |
| **海底植被** | kelp/seagrass（FEATURE） | 多处海底：chunk(-17,-15) x∈[-272,-261] z∈[-229,-225] y=54-56；chunk(-18,-14) x∈[-288,-273] z∈[-224,-209] y=48-57 等 | 48-57 | 确定 |

**Beardifier 结构源（StructureWeightSampler 只对 structure 生效，FEATURE 不参与）**：
- 村庄、沉船 A/B/C、矿井 —— **是**结构，其 ±24 格影响区才可能产生 Beardifier 非零。
- 紫晶洞、树、kelp/seagrass、ore —— FEATURE，**不产生 Beardifier 权重**。

---

## 2. 水体边界 mismatch 与结构的距离分布

### 2.1 方向拆解：一半水体边界「反向」，与 Beardifier 无关【确定】

Beardifier 缺失的效应方向是固定的：Java 侧 `finalDensity + StructureWeightSampler`（结构附近抬 density 更容易判实心），C++ 无此项 → density 更低 → **C++ 更倾向判 water**。

因此：
| pair 方向 | 块数 | 与 Beardifier 缺失的相容性 |
|---|---|---|
| got=32(water) ↔ vanilla=1/9/34/8/99/37（**C++ 水、vanilla 实心**） | ≈ 3117+2539+723+540+198+133+160(water→dirt_path) ≈ **7400** | **相容**（Beardifier 候选，还需看是否在结构 24 内） |
| got=1/970(stone/deepslate) ↔ vanilla=32（**C++ 实心、vanilla 水**） | 4416+635+9 ≈ **5060**（含水层） | **不相容**（Beardifier 缺失只会让 C++ 更判水，不会更判实心） |

**【确定】含水层 5060 块不可能由 Beardifier 缺失造成**。phase6「13000 块水体边界 = Beardifier 缺失」**按块数口径偏大**，至少 5060 块（≈39%）不属于 Beardifier。

### 2.2 含水层（stone→water / deepslate→water）距结构距离【确定】

含水层分布（grep 采样）：
- **stone→water**（4416）：chunk(-18,-14) y=11-18、chunk(-18,-15) y=15-16、chunk(-17,-15) y=17-23、chunk(-16,-15) y=1-6 → **y=11-23 石层深部**
- **deepslate→water**（635）：chunk(-18,-15) y=-3~0、chunk(-16,-15) y=-2~6、chunk(-15,-15) y=-2~4 → **y=-3~6 深板岩层**

距结构（切比雪夫距离，取最近结构块）：
| 含水层块示例 | 最近结构 | 切比雪夫距离 | 结论 |
|---|---|---|---|
| (-281,15,-220)（chunk-18,-14） | 村庄 (-275,62,-197) | max(6,47,23)=47 > 24 | 远离【确定】 |
| (-283,15,-218) | 矿井 (-258,-15,-209) | max(25,30,9)=30 > 24 | 远离【确定】 |
| (-277,15,-232)（chunk-18,-15） | 村庄 (-275,62,-224) | max(2,47,8)=47 > 24 | 远离【确定】 |
| (-262,17,-233)（chunk-17,-15） | 矿井 (-258,-15,-209) | max(4,32,24)=32 > 24 | 远离【确定】 |
| (-246,1,-231)（chunk-16,-15） | 矿井 (-252,-15,-209) | max(6,16,22)=22 ≤ 24 | **在矿井 24 内**（但方向反向，仍非 Beardifier） |
| (-236,-2,-235)（chunk-15,-15） | 矿井 (-252,-15,-201) | max(16,13,34)=34 > 24 | 远离【确定】 |

**结论【确定】**：含水层绝大多数（估 >85%）距所有结构 >24 格（主要是 y 深差 26-59 格主导）。且即使个别块（如 chunk-16,-15 靠矿井一侧）落在矿井 24 内，方向也是「C++ 实心、vanilla 水」——**与 Beardifier 缺失相反**，仍不能归因 Beardifier。

### 2.3 chunk(-16,-16)：无结构块，约一半水体边界 >24 格【确定】

chunk(-16,-16) mismatch = **364 块**，全部为 got=32(water) ↔ vanilla=1/9/8（stone/dirt/grass_block），cold_ocean 海底边界，坐标：
- x∈[-244,-241]（4 列），z∈[-256,-241]（16 列），y=50-62

该 chunk **不含任何结构块**（grep oak/cobble/chest/dirt_path/kelp 零命中）。最近结构 = 村庄（chunk(-16,-14) 的 dirt_path，最近点 (-241,63,-224)）。

距离计算（切比雪夫，对村庄最近点 (-241,62~63,-224)）：
| 位置 | 切比雪夫距离 | 是否 ≤24 |
|---|---|---|
| (-241,62,-241)（东北角） | max(0,1,17)=17 | 是 |
| (-244,50,-256)（西南角） | max(3,13,32)=32 | **否** |
| (-241,62,-248) | max(0,1,24)=24 | 是（边界） |
| (-244,50,-249) | max(3,13,25)=25 | **否** |

→ **z∈[-248,-241]（8 列）在村庄 24 格内；z∈[-256,-249]（8 列）在 24 格外**。
364 块按 x/y/z 均匀分布估计：约 **180 块（z∈[-256,-249]）远离结构 >24 格**【推测比例 / 确定坐标事实】。
这 180 块是「C++ water ↔ vanilla 实心」正向差异但**不在任何结构 24 内** → **不是 Beardifier 假 diff，是真海底高度/density 差**【推测机制】。

### 2.4 chunk(-15,-16)：海底边界 + 表面规则，部分 >24 格【推测】

chunk(-15,-16) mismatch = **1390 块**（river/cold_ocean/beach 海底），构成以 got=37(gravel)↔vanilla=1(stone)、got=32(water)↔vanilla=1/9、got=99(sandstone)↔vanilla=1、got=34(sand)↔vanilla=1 为主（海底 gravel/water/sand ↔ 石），y=47-52+。
- 无结构块（grep 零命中）。
- 最近结构 = 村庄 dirt_path（chunk(-15,-14) x∈[-240,-233] z∈[-224,-209]）。
- x 差 ≤14，z 差 17-32，y 差 10-17 → z 主导：z∈[-256,-249] 部分 >24 格。
- 估水体边界相关（water↔solid）约 500-800 块，其中约一半（z∈[-256,-249]）远离结构**【推测估算】**。

### 2.5 沉船/村庄附近的水体边界：Beardifier 候选【推测】

- **chunk(-17,-14)**：got=32(water)↔vanilla=1(stone) 大量（y=52-55, x∈[-272,-258], z∈[-224,-209]）——在沉船 A（x∈[-272,-257], z∈[-220,-212]）附近，切比雪夫 ≤16 ✓ **24 内**。另 water↔kelp/dirt_path（y=62）在沉船/村庄件上。
- **chunk(-18,-13) 村庄西缘**：got=32(water)↔vanilla=1/9/4/13 大量（x∈[-276,-273], z∈[-208,-193], y=54-62）——紧贴村庄（dirt_path x∈[-275,-273] z∈[-197,-195]），切比雪夫 ≤11 ✓ **24 内**。
- 这些是「C++ water、vanilla 实心」且距结构 ≤24 → **Beardifier 缺失的最可信候选**【推测】。
- 但注意：其中 water→kelp/seagrass（vanilla=126/127/678/679）属 FEATURE 块差异（C++ 未生成植被），与 Beardifier 无关，单独计为 FEATURE 假 diff。

---

## 3. 远离结构的差异聚簇清单（潜在真 bug 目标）

> 判定规则：切比雪夫距离到最近**结构**（村庄/沉船/矿井）> 24 格，且不属 FEATURE 块（kelp/ore/geode/tree）。

| # | 聚簇 | 坐标范围 | 估算数量 | 类型 | 方向 vs Beardifier |
|---|---|---|---|---|---|
| C1 | **石层含水层** | chunk(-18,-14)/(-18,-15)/(-17,-15) y=11-23，x∈[-288,-257], z∈[-240,-209] | 3000-4000 | got=stone vanilla=water | **反向（非 Beardifier）** |
| C2 | **深板岩含水层** | chunk(-18,-15)/(-16,-15)/(-15,-15) y=-3~6，x∈[-288,-225], z∈[-240,-225] | 400-600 | got=deepslate vanilla=water | **反向（非 Beardifier）** |
| C3 | **海底边界（深海）** | chunk(-16,-16) z∈[-256,-249] x∈[-244,-241] y=50-62 | ~180 | got=water vanilla=stone/dirt/grass | 正向（但 >24 格，非 Beardifier） |
| C4 | **海底边界/表面规则（river/cold_ocean）** | chunk(-15,-16) z∈[-256,-249] x∈[-240,-226] y=47-52 | ~300-400（估算） | got=water/gravel/sand↔stone | 正向（但 >24 格，非 Beardifier） |
| C5 | **深板岩洞穴（已单列）** | chunk(-16,-15)/(-18,-15) y=-45~-25，x∈[-288,-241], z∈[-240,-225] | ~5400 | got=deepslate vanilla=air | 独立机制（carver），与 Beardifier 无关 |

> C1/C2 合计 ≈ **5000+ 块**是最大非 Beardifier 差异源；C3+C4 ≈ **500-600 块**；C5（洞穴）≈ 5400 块单列（phase2 已归洞穴机制）。

---

## 4. 判定：Beardifier 假设影响面 vs 真差异

### 4.1 结案判定

- 【确定】「-288 差异 = 结构相关假 diff（Beardifier 缺失）」**不成立**：
  1. **含水层 5060 块方向相反**（C++ 实心、vanilla 水），Beardifier 缺失在机制上不可能造成；
  2. 含水层几乎全部距结构 >24 格（y 深差主导）；
  3. chunk(-16,-16)/(-15,-16) 有约 500-600 块水体边界距最近村庄 >24 格。
- 【确定】phase6「13000 块水体边界 = Beardifier」**按块数口径偏大**，需收窄为「**结构 24 格内的正向 water→solid 差异**」：
  - 沉船 A/B 附近、村庄西缘、chunk(-16,-16) 东北 8 列（z∈[-248,-241]）等；
  - 估算 Beardifier 可解释的水体边界 ≈ **2000-4000 块**（而非 13000）。
- 【推测】-288 差异的最终构成（按 67042 总块）：
  - 结构/FEATURE 假 diff（结构块 2400 + ore 2900 + kelp 400 + geode 200 + 树 等）≈ 6000-7000（9-10%）；
  - Beardifier 缺失（结构 24 内正向水体边界）≈ 2000-4000（3-6%）；
  - **真差异**（含水层 ~5000 + 海底边界 ~500-1000 + 洞穴 ~5400 + 浅层岩脉 ~32000 + 表面规则 ~3000 等）≈ 80-85% 主体；
  - 即「-288 差异」的主因仍与 phase2 一致（OreFeature 缺失 48.9% + 真 density/surface/洞穴/含水层），Beardifier 只是其中**新增、且只能解释部分水体边界**的一个机制，不是全部水体边界的解释。

### 4.2 对 phase6 的修正建议（交主会话裁决）

- 【架构变更建议·需裁决】phase6「13000 块水体边界 = Beardifier 缺失」**应收窄为「结构 24 格内的 water→solid 正向差异（≈2000-4000 块）」**；含水层 stone→water（4416）与 deepslate→water（635）应作为**独立真 bug**（C++ 在深部判实心而 vanilla 判水）定位，与 Beardifier 无关。
- 若仍按「13000 全部 Beardifier」推进实现，会**高估 Beardifier 修复收益**：注入 StructureWeightSampler 后最多消掉 2000-4000 块，含水层/海底边界/洞穴/岩脉 6 万块中的大头仍不匹配。

### 4.3 关键坐标复核

- phase6 锚点列 (-244,-256) 恰在 **chunk(-16,-16) 无结构块区域**，距村庄最近点 (-241,-224) 的 z 差 = 32 > 24。phase6 用该列证明 Java NOISE 阶段 solid、C++ water 且插值一致（-0.0744）→ 推断需要 Beardifier 正贡献 → **但该列距结构 >24 格，Beardifier 应为 0**（STRUCTURE_WEIGHT_TABLE 24 格查表外恒 0）。
  - 若 phase6 的 -0.0744 验算无误，则该列「Java solid」**另有原因**（含水层/海底液面处理、或 Java 在深海水域的结构权重覆盖了该区域的其他 structure start，需 Java 端结构清单复核）——【推测】此矛盾是下一轮必须解决的入口。
  - 若 phase6 的 CellCache/插值验算有误（phase5 已证反射探针不可信），则需重算该列真实 density。【推测】

---

## 5. 下一步建议（交主会话裁决）

按「修复收益 × 定位成本」排序：

1. 【优先】**含水层 stone→water / deepslate→water（C1/C2，~5000 块，y=11-23 与 y=-3~6）**
   - 锚点：chunk(-18,-14) y=11-18 x∈[-288,-273] z∈[-224,-209]；chunk(-18,-15) y=15-16；chunk(-17,-15) y=17-23；chunk(-16,-15) y=1-6。
   - 动作：对上述区域做 **C++/Java 含水层液面逐列对比**（Aquifer 液面解算、underground water level），确认为何 C++ 判实心而 vanilla 判水（方向：C++ density 偏高 / Java 液面覆盖）。与 phase2 §5.1 的「海域海底边界」不同源，单独立项。
2. 【次优先】**深海水域海底边界（C3/C4，~500-600 块，chunk(-16,-16)/(-15,-16) z∈[-256,-249]）**
   - 锚点：(-244,50,-256) 一带、chunk(-15,-16) z≤-249。
   - 动作：C++/Java 海底面高度逐列对比（phase2 §5.1 同类），确认是密度差还是 surface rule 差；**不要**归入 Beardifier。
3. 【复核】**phase6 锚点 (-244,-256) 矛盾**：Java 端检查该列 24 格内是否真有 structure start（村庄/沉船/其他 piece 的 bbox），若没有则 phase6 的「Beardifier 抬 density」归因在本列不成立，需回到插值/含水层方向重新定位。
4. 【决策】Beardifier 实现（phase6 §5.3）**保留但不作为 -288 对齐主路径**：预期收益 ≈ 2000-4000 块（3-6%），且需要结构系统前置。可在含水层修复后重测匹配率再决定。

---

## 6. 置信度与边界说明

- 【确定】：结构块坐标清单（grep 直接证据）；含水层方向（pair_counts 全量 + Beardifier 方向逻辑）；chunk(-16,-16) 坐标范围与距离计算（364 块全量读取）；含水层 y 层与结构 y 层垂直距离 >24（多例计算）；phase6 锚点列距结构 >24（坐标计算）。
- 【推测】：数量占比估算（C3/C4 的 180/300-400 块为按分布均匀假设估算，非逐块计数）；Beardifier 可解释块数上界 2000-4000（未做逐块距离统计）；含水层/海底边界的机制归因。
- 本报告只做解读，不修改代码；所有结论基于现有数据文件，未运行任何命令。

### 附：参考数据速查

- chunk 级 mismatch：(-18,-16)/(-17,-16)=0；(16,-16)=364；(-15,-16)=1390；(-18,-15)=2529；(-17,-15)=2090；(-16,-15)=2868；(-15,-15)=2487；(-18,-14)=6918；(-17,-14)=8806；(-16,-14)=7248；(-15,-14)=5842；(-18,-13)=5685；(-17,-13)=7543；(-16,-13)=7084；(-15,-13)=6188。
- 水体边界主要 pair：water→stone 3117；stone→water 4416；water→dirt 2539；water→sand 723；water→grass 540；deepslate→water 635；water→sandstone 198；water→gravel 133；water→dirt_path 160；water→kelp/seagrass 259+66+31+28。
- 结构特征 ID：oak_log=46、oak_planks=13、oak_stairs=176、cobblestone=12、chest=177、dirt_path=602、amethyst_block=903。
