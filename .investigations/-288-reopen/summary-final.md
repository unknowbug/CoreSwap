# -288 课题最终结论（2026-08-09 破案）

> 课题：seed=-8248318472910187742（-8248 世界），-288,-256 4×4 区域 block_probe 匹配率 95.7376%（用户实测质疑 8/8「结构/FEATURE 假 diff」结案）。
> **最终判定：8/8 结案方向正确（差异 = C++ 范围外功能），但机制描述不完整；C++ 核心（density/aquifer/surface/vein）全部正确，无 bug 需修复。**

## 一、调查过程（14 轮分析产物，见本目录 analysis-phase*.md）

| 阶段 | 结论 | 状态 |
|---|---|---|
| Phase 2 量化（phase2） | 67042 差异：natural 82.2% / air 9.6% / structure_feature 7.9%——「结构假 diff 为主」**数据上不成立**（结构仅 7.9%） | ✅ |
| Phase 3 密度层（phase3） | C++ 插值链与 Java cns 游戏实际链逐位一致（≤4e-6）；y=36 差 0.23 = 无插值/插值基准错配；base_3d_noise 排除 | ✅ |
| aquifer 层（phase4-5） | AQF-J null 判定不可信（CellCache 反射污染）；同基准 density 一致 | ✅ |
| NOISE 块直读（NOISE-BLK） | Java NOISE 阶段 (-244,-256) y=58-61 stone（岛非结构）、(-278,-240) y=15-19 water | ✅ 铁证 |
| Beardifier（phase6-8） | Java aquifer 输入含 StructureWeightSampler；但含水层区域实测 [BEARD]=0（ocean_ruin 无 terrain_adaptation 不参与） | ✅ 排除 |
| noodle/caves 树（phase10-13） | noodle 低频（firstOctave=-8 单 octave）；slopedCheese 3.1~5.1 >1.5625 → caves 树完整启用；C++ 树组件逐位一致 | ✅ |
| **AQF-APPLY 铁证** | **Java aquifer.apply 直接调用 (-278,12..23,-240) 全部判 solid，density 与 C++ 逐位一致（0.055724~0.068693）**——Java aquifer 判 solid，与 C++ 完全一致 | ✅ 决定性 |
| **chunk status 铁证** | chunk(-18,-15) status=`minecraft:carvers`——含水层 water/air = **洞穴雕刻（CaveCarver）阶段**产物，非 aquifer | ✅ 决定性 |

## 二、最终结论

### 1. C++ 核心全部正确
- **density/aquifer/surface/vein 逐位对齐**（AQF-APPLY + 分量对比 + 插值链对比三线互证）
- 之前所有「aquifer 判水 bug」「CellCache 语义差」「noodle 高频丢失」假设**全部推翻**（详见 phase5/6/11/13）

### 2. -288 差异构成（67042 块，2026-08-09 修正版——judge 审查后补充）
| 类别 | 数量 | 占比 | 机制 | 状态 |
|---|---|---|---|---|
| **岩石替换矿脉**（granite/tuff/diorite/andesite + coal_ore） | ~3.4 万 | **~51%** | ore_* placed feature（FEATURE 阶段）替换岩石层 | ✅ 已闭合（FEATURE） |
| **洞穴雕刻 carvers**（挖洞 + 液面填水） | ~1.2 万 | **~17%** | CaveCarver 阶段（NOISE 后雕刻 + 含水层液体）——含水层 water 非 aquifer | ✅ 已闭合（FEATURE，AQF-APPLY + status 铁证） |
| 结构（岛 Beardifier + 沉船/村庄） | ~0.24 万 | 3.6% | StructureStart（含 Beardifier 密度修正） | ✅ 已闭合（STRUCTURE） |
| 树/草/植被 | 少量 | ~1% | FEATURE 装饰 | ✅ 已闭合 |
| **海底边界**（water↔stone/dirt/sand 双向） | ~0.64 万 | **~11.6%** | **未完全定位**——候选：surface 海底 gravel/砂染色差、结构（岛）相关、C++ surface 微小项；phase2 曾标「C++ 海底系统性偏低」 | ⚠️ 未闭合（待后续） |
| **gravel**（海底表面 + 深层 blob） | ~0.49 万 | **~7.3%** | 部分 surface（海底 gravel 染色）+ 部分 ore_gravel FEATURE——未细分 | ⚠️ 部分闭合 |
| 表面规则（stone↔dirt 等） | ~0.29 万 | **~4.3%** | surface 规则（海岸/河岸），未逐项定位 | ⚠️ 未闭合（待后续） |

**诚实标注**：已闭合（范围外 FEATURE/STRUCTURE）≈ 73%；未完全闭合（海底边界/gravel/表面规则 ≈ 23%）机制待后续定位——**不归入「差异全为范围外 FEATURE」的覆盖**，judge 审查确认需补充。

### 3. 8/8 结案修正
- **方向正确**：差异确为 C++ 范围外功能（FEATURE/结构）
- **机制补充**：含水层 water = carvers 液体填充（05 篇 L86 早有记载「deepslate→air = 洞穴雕刻（FEATURE）」）；岛 = 结构 Beardifier 抬 density；granite/tuff = ore_* placed feature
- **旧记录纠错**：时间线 L670「base_3d_noise 负坐标差 0.05-0.23 未定位」= RouterProbe 独立构建假象（03 篇 L100 deriver 验证已排除，本次再次确认）
- **phase13 归纳因果链（judge 发现 #2 补充）**：phase13 原文结论是「caves 树中频判水【强候选·未 100% 闭合】」（建议核查 C++ caves 树）——该假设**被 AQF-APPLY + chunk status 铁证取代**：AQF-APPLY 证明 Java aquifer 判 solid（与 C++ 一致）→ 判水不是 aquifer/caves 树问题，而是 carvers 阶段产物。早期「caves 树差」方向是**正确排除链的一部分**（排查过但被更强的直接证据取代），非最终结论。

### 4. FEATURE 范围决策（2026-08-09 用户拍板）
- **只做地形性 FEATURE：carvers（洞穴雕刻）+ 岩石替换（ore_granite/tuff/diorite/andesite）**——影响玩家可见地形
- 矿石（coal/iron/copper 等）、树/草、结构：**暂缓**
- **暂缓实施**（用户明确「不急着做」）——本 session 只记录决策，不开始实现

## 三、产物清单（.investigations/-288-reopen/）

- `analysis-phase2.md` ~ `analysis-phase13.md`：14 轮分析（量化/密度层/aquifer/Beardifier/caves 树/octave）
- `noise_blk_patch.md` / `beardifier_patch.md`：Java 探针 patch 设计
- `cmd-output/`：全部原始数据（m288_run1.txt、aqfapply_run.txt、noiseblk_run3.txt、noodle2_run.txt、dump_x*.txt、ref_col_*.txt 等）
- `m288_pair_counts.txt` / `m288_vanilla_cat.txt` / `m288_chunk_counts.txt` / `m288_natural_rows.txt`：差异统计
- `analyze_m288.py` / `read_col2_m288.py` / `compare_comps.py` / `filter_rc.py`：分析脚本
- Java 探针改动（MC 工程，本地 M 状态）：DensityProbe（AQF-APPLY/CellCache n）、BlockProbe（NOISE-BLK v2 多列 + BEARD + status）

## 四、后续待办（NEXT_SESSION）

1. **carvers + 岩石替换实现**（用户拍板范围，暂缓）：需 Phase 0 架构设计；数据已就绪（worldgen/data 有 configured_carver/configured_feature/placed_feature）
2. **8576 21 块课题**（独立机制）：22 块清单无 carvers 差异（深板岩/水边界 + 地表分层错位 + terracotta 带）——finalDensity 边界翻转，与原计划一致
3. **未闭合差异定位（judge 发现 #1）**：海底边界 ~0.64 万（11.6%）+ gravel ~0.49 万（7.3%）+ 表面规则 ~0.29 万（4.3%）——候选 surface 海底 gravel/砂染色、ore_gravel FEATURE、C++ surface 微小项——后续定位
4. **诊断工具清理**：WG_AQF_YMIN/YMAX（aquifer.h）、WG_NOODLEDUMP（worldgen_api.cpp/noise.h octave dump）为本次调查新增——保留（通用诊断）或按需清理
5. **retry cap 记录（judge 发现 #5）**：phase10-13 连续 4 轮围绕 noodle/caves 子方向（phase11「高频」在 phase13 被证反）超 3 轮上限——每轮有新证据（低频修正/角点差/octave dump），最终由 AQF-APPLY 新方向跳出；已记录轮次依据

## 五、judge 审查记录（2026-08-09）

- judge 结论：**推荐保持 candidate**（核心有铁证支撑，不降 draft）；无越权标 confirmed；无只留对话的结论
- 同意：C++ density/aquifer 核心无 bug（AQF-APPLY 与 phase3/13 三线互证）、carvers 铁证成立、置信度/职责/模块边界合规
- 需补充（已修正）：① 差异构成闭合（§2 已补未闭合项）② phase13 归纳因果链（§3 已补）③ AQF-APPLY 高位垃圾值说明（见下）④ index.yaml 覆盖（§6）⑤ retry cap 记录（见四-5）
- **AQF-APPLY 高位垃圾值说明**：y≥256 段恒 -0.024995 = CellCache 反射垃圾值（phase5 L750 铁律记载）——核心段 y=12..23 每块独立 dump 不受影响

## 六、产物索引补登记（judge 发现 #4）

- `m288_pair_counts.txt` / `m288_vanilla_cat.txt` / `m288_chunk_counts.txt` / `m288_natural_rows.txt`：差异统计（phase2 输入）
- `analyze_m288.py` / `read_col2_m288.py` / `compare_comps.py` / `filter_rc.py`：分析脚本
- `ref_col_*.txt` / `comps_diff2.txt`：参照列形态 / 分量对比
- 已登记 `.artifacts/-288-reopen/index.yaml`（raw-data 条目扩展至全部）
