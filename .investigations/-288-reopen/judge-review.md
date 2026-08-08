# judge 审查意见：-288 课题破案（.investigations/-288-reopen/）

> 角色：core.judge（隔离子进程，只出意见，不改任何 status/文件）
> 审查对象：`.investigations/-288-reopen/`（analysis-phase2..13 + summary-final.md + patch + cmd-output + stats + scripts）与 `.artifacts/-288-reopen/index.yaml`
> 课题：seed=-8248318472910187742（-8248 世界），-288,-256 4×4 block_probe 95.7376%，用户质疑 8/8「结构/FEATURE 假 diff」结案
> 最终判定（待审）：C++ 核心无 bug；差异 = 岩石替换 49% + carvers 17% + 结构 3.6% + 树草 1% + surface 0.1%；FEATURE 范围决策（只做地形性）由用户拍板
> 日期：本次审查
> 状态标注：仅建议，不修改。推荐状态：**保持 candidate（待用户确认）**。

---

## 一、总体判定

**核心结论方向可信，但最终差异构成表不闭合（约 29% 差异块未归类），且对 phase13 的归纳存在失真。**
「C++ density/aquifer 核心在含水层列无 bug、含水层 water/air = carvers 阶段产物」有硬证据（AQF-APPLY 核心段 + chunk status 铁证），**同意**；但「-288 差异全部 = 范围外 FEATURE」的强表述**需补充**（构成缺口 + phase13 假设被推翻的因果链未明）。

**推荐状态：保持 candidate**（不升 confirmed——confirmed 留给用户；不降 draft——核心结论有铁证支撑）。

---

## 二、审查清单逐项结论

### 1. 证据链闭合性：核心闭合，构成不闭合 → 部分同意 / 需补充

**同意（核心结论）**：
- density/aquifer 逐位对齐：phase3（插值链 ≤4e-6）+ phase13（finalDensity 整树 CellCache=aqfin ≤1e-6）+ AQF-APPLY（y=12..23 与 C++ aqfin 逐位一致 ≤1e-6 且判 solid）三线互证，证据链闭合。
- 含水层 water = carvers 阶段产物：AQF-APPLY（NOISE 阶段 aquifer 判 solid）+ NOISE-BLK（chunk(-18,-15) status=`minecraft:carvers`，读到的 y=15-19 water 已是雕刻后状态）时序自洽。1.20.1 ChunkStatus 确实含 CARVERS（Noise=4 → Surface=5 → Carvers=6 → Features=7），铁证基石成立。
- 8/8 结案方向修正（「结构假 diff 为主」数据上不成立 → 差异主体是范围外 FEATURE）有 phase2 全量统计支撑。

**需补充（构成缺口，重要）**：
- summary-final 差异构成表（49%+17%+3.6%+1%+0.1%）合计约 **70.7%**，与 phase2 完整量化（A-H 合计 ≈100%）不闭合。未归类约 **1.9 万块（≈29%）**：
  - 矿脉 *_ore（phase2 B 组 ≈2900，4.3%）
  - gravel 类（phase2 C 组 ≈4900，7.3%，含海底 gravel 与深层 blob）
  - **海底边界 water→stone/dirt/sand 等（≈7400，11.6%）**——phase2 曾标「C++ 海底面系统性低」真差异，phase7 判定部分距结构 >24 格非 Beardifier → 真差异
  - 表面规则互换（phase2 G 组 ≈2900，4.3%）
  - 反向空气（phase2 F 组 ≈1260，1.9%）
- 若这些块（尤其海底边界）仍属 C++ 范围内差异，则「C++ 核心无 bug、差异全为范围外 FEATURE」的表述**覆盖不完整**；需补充说明这些块在 AQF-APPLY/status 证据后的最终去向（重新归因 or 仍待定位 or 属范围内微小项）。

### 2. 铁证有效性：AQF-APPLY 核心段有效，探针高位有垃圾值；status 铁证成立 → 同意（附技术标注）

**AQF-APPLY**：
- 【同意】核心段 (-278,12..23,-240) density 与 C++ aqfin 逐位一致（y=12: 0.055724 vs 0.055723、y=16: 0.068693 vs 0.068692、y=23: 0.068408 vs 0.068408，均 ≤1e-6），全部 `null(solid)`——两独立实现数值一致到 1e-6 不可能巧合，该段**确立「Java aquifer 在含水层列判 solid 与 C++ 一致」**。
- 【需补充】同文件高位段（y=319..256）density **恒为 -0.024995**——这正是 phase5 §2.3 记载的「CellCache 反射垃圾值（如固定 -0.024995）」。说明探针并非全高度真实遍历（或遍历后高位 cell 未填充即被反射采样）。**不影响已使用的核心段结论**，但建议补充探针实现说明（DensityProbe AQF-APPLY 如何驱动遍历、高位为何返回垃圾），使「cns 游戏同构遍历」的描述有据可查。
- 无直接反证：NOISE-BLK 读到的 water 与 AQF-APPLY 判 solid 不矛盾（阶段不同：前者 carvers 后、后者 NOISE 时）。

**NOISE-BLK / chunk status**：
- 【同意】chunk(-18,-15) status=`minecraft:carvers` 是 1.20.1 合法状态（外部确认 ChunkStatus 含 CARVERS）；NOISE-BLK 请求 NOISE 但读到 carvers 阶段块 → y=15-19 water 是雕刻后状态。与 AQF-APPLY 时序自洽，支持「含水层 = carvers 液体填充」。
- 【同意】(-244,-256) 列（chunk(-16,-16) status=`noise`）y=58-61 stone 是真实 NOISE 阶段块——岛非结构覆盖，属 Beardifier/密度抬升范畴（phase6-9），与 summary 结构 3.6% 归因一致。
- 反证排查：chunk(-18,-14) status=`initialize_light`、chunk(-17/-16,-16) status=`noise`——各 chunk 进度不同，探针打印 status 恰好证明其意识到了阶段差异，无系统性误读。

### 3. 置信度标注：合理 → 同意

- index.yaml 全部 status: candidate；summary 明示「confirmed 待用户拍板」；无越权标 confirmed。
- 无运行时证据却标 candidate 的情况：有（AQF-APPLY/NOISE-BLK 均为实际运行产物，cmd-output 落盘），符合「以实际执行为准」。

### 4. 落盘契约：产物落盘 ✓，index 覆盖不全 → 部分同意 / 需补充

- 【同意】所有结论均已落盘（无只留对话的结论）：analysis-phase2..13、summary-final、cmd-output/ 全套、m288_* 统计、分析脚本、ref_col 参照列、patch 描述均存在。
- 【需补充】`.artifacts/-288-reopen/index.yaml` **未登记以下落盘产物**（raw-data 条目仅指向 `cmd-output/`，未覆盖根目录）：
  - `m288_pair_counts.txt`、`m288_vanilla_cat.txt`、`m288_chunk_counts.txt`、`m288_natural_rows.txt`（关键统计）
  - `analyze_m288.py`、`read_col2_m288.py`、`compare_comps.py`、`filter_rc.py`（分析脚本）
  - `comps_diff.txt`、`comps_diff2.txt`、`ref_col_-242_-256/-244_-256/-278_-240.txt`（对照数据）
  建议补充 index 条目或说明归属，以满足 core.artifact 契约。

### 5. 模块边界 / retry cap / 职责分离 → 同意（边界情况标注）

- 【同意·模块边界】产物引用的是 Java 源码（MC 工程）、本模块 C++ 源码（versions/1.20.1/cpp）与同模块 docs（03/05/07/09/10），未引用其他领域模块 skill 正文，无违规。
- 【边界情况·retry cap】phase10-13 连续 4 轮围绕「noodle/caves 树负值来源」假设：phase11 判定 noodle「高频」方向在 phase13 被证实为「反了」（实为低频）。若按「同假设验证失败 ≤3 次换方向」严格计数，该子方向已超 3 轮；但每轮有新数据（InterpDiag、cns idx、raw noodle、slopedCheese 实测）且最终由 AQF-APPLY（新勘探方向）闭合，可辩护为递进调查。**建议**在产物中记录轮次控制依据（哪些轮是新证据驱动、哪轮是假设修正），避免未来被质疑 retry cap。
- 【同意·职责分离】采集在主会话（运行探针/生成 cmd-output）、解读在 subagent（phase 文档），符合「采集主会话、解读 subagent」；探针 patch 描述由 scout 产出、由主会话应用，职责清晰。

### 6. 三源核对（docs 提炼 vs .investigations 产物）→ 同意，附一处失真

- 【同意】docs/07 追加 4、docs/05 决策更新、docs/10 2026-08-09 条目与 summary-final 一致：含水层=carvers、构成表、FEATURE 范围决策、8/8 修正、8576 独立课题均无夸大。
- 【同意】C++ 诊断扩展（WG_NOODLEDUMP、WG_AQF_YMIN/YMAX、octave dump）为纯诊断：noise.h sample 中 `dumpOct` 仅 `fprintf` 不改变返回值 `d`；worldgen_api.cpp WG_NOODLEDUMP/WG_SURFDUMP 等均为 dump 分支，未改生成逻辑。✓
- 【失真·需说明】docs/10 与 summary 均将 phase13 归为「✅ C++ 树组件逐位一致」，但 phase13 原文结论 6 是「Java 判水机制 = caves 树（cave_cheese 中频）翻转【强候选·未 100% 闭合】」、结论 7 是「核查 C++ caves 树…在负坐标远端的差」——**phase13 并未证明 C++ caves 树组件一致**，而是留下未闭合假设。summary 的「逐位一致」应明确限定为 phase13 结论 3（整树插值链 CellCache=aqfin），并补充说明 phase13 的「caves 树中频判水」假设如何被 AQF-APPLY（aquifer 判 solid）+ status 铁证（water 来自 carvers）**取代/推翻**——否则 8/8 链从「caves 树疑似差」到「C++ 核心无 bug」的转折缺因果记录。

### 7. 遗留问题诚实性 → 同意

- 【同意】surface 微小项 ~0.1% 标注「待后续核查（gravel 染色边界等）」诚实，未掩盖。
- 【同意】FEATURE 范围决策记录完整（用户拍板「只做地形性 FEATURE」+ 暂缓实施），且「用户拍板」与 docs/07 追加 4、docs/05 L88 一致。
- 【需补充】遗留问题清单未覆盖「构成缺口 ~29% 的最终去向」（见 §1）与「AQF-APPLY 探针高位垃圾值机制」，建议并入 NEXT_SESSION。

---

## 三、逐条结论速查表

| # | 审查点 | 判定 | 关键依据 |
|---|---|---|---|
| 1a | C++ density/aquifer 核心无 bug（含水层列） | **同意** | AQF-APPLY y=12..23 与 C++ aqfin 逐位一致 ≤1e-6 + phase3/13 三线互证 |
| 1b | 差异构成表（49/17/3.6/1/0.1%） | **需补充** | 合计约 70.7%，矿脉/gravel/海底边界/表面规则约 29%（≈1.9 万块）未归类 |
| 2a | AQF-APPLY 铁证有效性 | **同意（附标注）** | 核心段逐位一致判 solid；高位段（y≥256）恒 -0.024995 = 已知反射垃圾值，探针遍历范围需说明 |
| 2b | chunk status=carvers 铁证 | **同意** | 1.20.1 ChunkStatus 含 CARVERS；NOISE-BLK 读到的 water 为雕刻后状态 |
| 3 | 置信度标注 candidate | **同意** | confirmed 留给用户，无越权 |
| 4 | 落盘 + index.yaml 契约 | **部分同意** | 产物全落盘 ✓；index.yaml 未登记 m288_*、脚本、ref_col、comps_diff ✗ |
| 5 | retry cap / 职责 / 模块边界 | **同意（边界标注）** | phase10-13 四轮 noodle/caves 子方向超 3 轮上限，但有新证据驱动 + AQF-APPLY 新方向闭合；建议记录轮次依据 |
| 6 | 三源核对 | **同意（一处失真）** | docs/07/05/10 与产物一致；C++ 诊断纯打印不改逻辑；phase13 归纳需注明「假设被 AQF-APPLY 取代」 |
| 7 | 遗留问题诚实性 | **同意（补两项）** | surface 0.1% 诚实；FEATURE 决策完整；补构成去向与探针垃圾值机制 |

---

## 四、建议动作（非命令，交主会话/用户裁决）

1. **补充差异构成映射**：将 phase2 完整量化（A-H ≈100%）映射到最终定性，明确矿脉 4.3%、gravel 7.3%、海底边界 ≈11.6%、表面规则 4.3%、反向空气 1.9% 的去向（重新归因 or 仍待定位）。这直接影响「C++ 核心无 bug」结论的覆盖范围。
2. **补充 phase13 → 最终结论的因果链**：说明 phase13「caves 树中频判水【未闭合】」如何被 AQF-APPLY + status 铁证取代（判水不在 aquifer 阶段而在 carvers 阶段）。
3. **补充 AQF-APPLY 探针实现说明**：高位段（y≥256）为何返回恒定 -0.024995（CellCache 反射垃圾值）；确认核心段 y=12..23 确在真实遍历状态内采样。
4. **补 index.yaml 条目**：登记 m288_*、分析脚本、ref_col、comps_diff（或说明归属）。
5. **记录 retry cap 依据**：phase10-13 轮次控制说明（新证据驱动/假设修正/新方向跳出）。

## 五、推荐状态

**保持 candidate**。理由：核心结论（C++ aquifer/density 无 bug、含水层=carvers 产物）有铁证支撑，不降 draft；但构成表缺口与 phase13 归纳失真未闭合前，不宜由审查方升 confirmed——confirmed 本应留给用户拍板（用户已拍板 FEATURE 范围，但未确认「破案」本身）。若上述 1-3 项补齐，可建议用户确认。

---

*本意见仅为审查建议，不含任何 status 修改动作；与 Anchorlaw §12 规则挑战正交。*
