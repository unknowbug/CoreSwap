# 知识库草稿：幕帘线（260904-06，judge PASS-with-conditions / candidate）

> 产出者：core.worker 知识库 subagent（260904-06）。主会话负责应用；本文件仅草稿，不改任何正式文件。
> 依据：SUBAGENT-KNOWLEDGE-GUIDE.md（价值门 + 载体映射 + 自检清单）。
> 背景材料：curtain-verdict-260904-06.md（candidate，supersedes writer-verdict 结论 3）+ p4-reference-check-260904-06.md（事实底账）+ review-judge-curtain-supersedes-260904-06.md（judge，含误读源头定位）。

## 价值门筛选结论

| 候选 | 价值判定 | 载体 | 编号续接 |
|---|---|---|---|
| 候选 1：参照存在性判定单向盲区 | **高价值**（归因链整体建立在误读上 + 可复用判据 + 反模式） | knowledge/discovered/workflow-patterns.md | 发现 **#40** |
| 候选 2：「0<d」式机制断言未实测当公理 | **高价值**（与 #40 同一事故的对偶面：一处盲区 + 一处未实测断言共同坍塌整条课题；判据可独立复用） | knowledge/discovered/workflow-patterns.md | 发现 **#41**（与 #40 互引，不并入——机制不同：#40 是「参照读什么漏什么」，#41 是「断言跳过数据层实测」） |
| 候选 3：aquifer barrier margin 机制指纹 | **中价值（简写）**——算法/机制指纹，跨臂复用（「stone 但 d≤0 ≠ bug」直接免排查） | knowledge/discovered/algorithm-fingerprints.md | 发现 **#17** |

低价值项未收录：本列 32 块同型残差明细、grass/water 交替剖面、granite@y217 写者 idk——均为课题特定一次性结论，归 ore desync 课题与 verdict idk 登记，不进知识库。

---

## 一、workflow-patterns.md 追加文本（续接发现 #40，插到发现 #39 之后）

```markdown
## 发现 #40: 参照存在性判定的单向盲区——「参照无 X」断言只查差异族从不查背景族，整条归因链可建立在误读上（260904-06）

- **发现时间**：260904-06（锚 git 924e934）。**发现者**：主会话 decisive probe（P4）+ judge 误读源头定位。**置信度**：candidate（单轮事故实锤，judge 三源核对通过；confirmed 留人类）。**module**：通用方法论（残差归因 / 参照解读）。
- **来源定位**：`.investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md`（Facts #3）+ `review-judge-curtain-supersedes-260904-06.md` §2.4（误读源头：convergence 轮 `ref_col_check_260904-06.py` 只查 dirt/sand/gravel 差异族，从未核对 stone 族）+ `.artifacts/lossless-accel/curtain-verdict-260904-06.md`（取代裁决）。

### 观察（现象）

幕帘课题前提「vanilla y≥201 无 stone」源自 convergence 轮对同一参照 .blocks 的解读——该轮对比脚本（ref_col_check_260904-06.py）只数了 dirt/sand/gravel（差异族），从未数 stone（背景族）。实际 vanilla 参照列 (195,199) y180-319 含 **131 个 stone 族块**（stone 99 + granite 24 + copper_ore 7 + iron_ore 1，复验脚本 `ref_check_p4_260904-06.py` 输出）。「vanilla 无 stone」被整条课题当成公理，推导出「幕帘 = C++/Rust 共有 vanilla 偏离、真根因上移 NOISE」的 writer-verdict 结论 3——P4 一次存在性复验即推翻整个课题前提（§15.4 取代裁决）。

### 证据

- judge 独立重跑 ref_check_p4 + xcheck_p4 复现关键数字（review-judge §3）：stone:99/granite:24/copper:7/iron:1，32 diffs 精确等于带内 ore 族总数；
- 参照 .blocks 本身无解析错误（idx = 3*(H*16)+7*H+(y−(−64))，seed 双臂同 8576294172403134396）——误读属「读了但看错对象」（只查差异族），不是解析 bug；
- 正向教训：b1 worker 的 idk「vanilla 无 stone 引用自 convergence，本臂未独立复验」（`.investigations/lossless-accel/fanout-writer-260904-06/b1-surface-oob-writer.md` §6）事前命中该 bug——诚实声明纪律起到风险标记作用（judge §6 确认）。

### 根因（机制）

残差归因常用「vanilla 无 X」式存在性判定，但对比脚本天然沿「差异族」清单查（本次 diff 是什么就查什么），**从不查「背景族」**（stone 在本课题是假设的背景块，无人想到要数它）。存在性断言一旦从「mod ≠ 参照在差异族上」间接反推出「参照无 X」，就绕过了任何直接观测——参照里本来就有 X 时，归因链的地基（幕帘是偏离）整体悬空。#39 核「参照输出对不对」，本条核「参照里到底有没有被断言『无』的东西」——同一「参照解读」风险面的存在性维度。

### 如何利用

1. **判据（写进归因检查单）**：任何「参照无 Y」断言 MUST 用独立脚本/独立口径做一次存在性复验（直接数参照里 Y 到底有没有），**禁止从「mod ≠ 参照」反推存在性**。成本一轮以内（本例一个 ~20 行脚本），漏做的代价 = 整条课题归因作废。
2. 复验脚本要**族级别全覆盖**（至少列出参照列的 id 直方图），不能只数当前差异族——差异族清单本身依赖被检验的前提。
3. 家族索引：#39（参照自身缺陷辨识——对偶面）/ #13（探针输出 sanity check）/ #3（块级真相验证法——本条是其「先看参照本身」分支）/ #18（跨 session 数字不可续推——本条是其「参照解读结论」特例）。

## 发现 #41: 「0<d」式机制断言未经数据层实测当公理续推——一轮生产密度 dump 即证伪（260904-06）

- **发现时间**：260904-06（锚 git 924e934）。**发现者**：P-B1-2 判别探针执行中顺藤发现（负值）→ 主会话 P4 复验。**置信度**：candidate（生产 dump + 点采双载体交叉证伪；confirmed 留人类）。**module**：通用方法论（机制断言验证纪律；与 #40 同一事故的对偶面）。
- **来源定位**：`.investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md`（Facts #1/#2）+ `.artifacts/lossless-accel/curtain-verdict-260904-06.md` §裁决 1。

### 观察（现象）

「幕帘带 = 0<d≤0.39」作为机制描述贯穿幕帘课题（writer-verdict 结论 3 及其上游归因），从未对生产执行体实测过 d 值。WG_DBDEBUG 生产密度 dump 一轮实测：C++ 幕帘带 d **全负**（y256-318 = −0.024995，y192-255 = −0.458333，y180-319 无一正值），density_probe 点采 %.17g 与生产一致——前提当轮证伪；stone 实际来自 aquifer barrier margin（`density+e>0` 翻转，aquifer.h:121-137 / worldgen_api.cpp:1040 消费环）。

### 证据

- 生产 dump（`.tmp/p2full/cpp-dbdebug-195-199-260904-06.txt`）与点采（`cpp-fdcol-195-199.txt`）双载体交叉一致；
- 负值量级与 b1 静态理论地板（−0.025/−0.458）逐位吻合（judge §2.1）——静态推导能预测正确值，恰说明「d 的符号」这类**可实测命题**不该停留在静态描述。

### 根因（机制）

「0<d」是从 Java 源码静态读出的符号级断言，在课题内被降格为背景描述反复引用，无人把它当「待实测假设」——机制断言的置信度应当与证据层匹配：静态读码 ≠ 数据层实测（trace/probe），而下游归因把它用成了公理。与 #40 叠加形成双故障：一处参照误读（#40）+ 一处未实测断言（本条），共同支撑了「幕帘 = 共有偏离」的错误结论。

### 如何利用

1. **符号级机制断言（「X 带内 d>0」「此路径恒 return」类）进入归因链前 MUST 一次数据层实测**（env 门控 dump / 点采探针，一轮以内）；实测与静态读数冲突时以实测为准并回溯引用点。
2. 课题结案/取代时检查：被依赖的机制描述里有几条从未实测过？未实测的逐条标注「static-only」再结案。
3. 家族索引：#40（参照存在性盲区——本事故另一面）/ #25（Java 常量必须追取值源头——同「静态结论需锚定」家族）/ #17（跨探针对比坐标钉死律）/ AGENTS「交接结论验证纪律」（§16.3 廉价独立验证的机制断言特例）。
```

INDEX.md 同步行（追加到 260904-06 追加段之后）：

```markdown
> 260904-06 追加（二）：workflow-patterns 新增**发现 #40/#41**（参照存在性判定的单向盲区——「参照无 X」MUST 独立复验禁止从差异族反推；「0<d」式机制断言未经数据层实测当公理续推，一轮 dump 证伪——幕帘课题前提坍塌两对面，源自 curtain-verdict-260904-06 取代裁决）。
```

---

## 二、algorithm-fingerprints.md 追加文本（中价值简写，续接发现 #17）

```markdown
## 发现 #17: aquifer barrier margin 机制指纹——|d|≈0.02 微负带是 margin stone 高发区，「stone 但 d≤0」≠ bug（260904-06）

- **时间/置信度/module**：260904-06，candidate（生产 dump + 静态结构互证；confirmed 留人类），MC worldgen 机制指纹。
- **指纹**：aquifer `apply` 在 d≤0 时唯一把 block 翻成 stone 的路径是 barrier margin（`density+e>0` 三连 -1 翻转，aquifer.h:121-137 / aquifer.rs:327-338，三方零偏离——b3 十七项）→ **|d| 极小（≈0.02，如 −0.024995 带）的 stone 带是 margin 高发区的签名**；「该处有 stone 但 d≤0」不是 bug，先查 margin 而非density 写者。C++ 消费环：worldgen_api.cpp:1040（block<0 → stone）。
- **如何利用**：高 y stone 带排查时先看 d 剖面——d 全负 + 幕帘形态 → 直接归 margin（aquifer 高位水口袋 ~16 间距伴生），跳过「density 写者缺失」方向；d 显著正的 stone 才查 surface/ore 写者。注意 Rust 侧已知分叉：margin→air 丢 barrier stone（b3 发现，只影响幕帘构成）。证据：.investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md + .artifacts/lossless-accel/curtain-verdict-260904-06.md。
```

INDEX.md 同步行（algorithm-fingerprints 分类行末追加）：

```markdown
、aquifer barrier margin 机制指纹——|d|≈0.02 微负带 = margin stone 高发区，「stone 但 d≤0」≠ bug（发现 #17，260904-06）
```

---

## 三、草稿自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：候选 1/2 高价值详写，候选 3 中价值简写；一次性结论（32 块明细/grass-water 剖面/granite idk）未写知识库
- [x] 五段式：#40/#41 均含观察（现象具体：131 块构成 / d 全负数值）/根因（机制层面：差异族查法盲区 / 断言置信度错配）/定位（复验脚本 + judge 重跑）/教训（判据进如何利用）
- [x] 根因为机制层非现象复述
- [x] 定位含诊断方法（ref_check_p4 独立脚本、WG_DBDEBUG dump、judge 独立重跑）
- [x] 判错经验沉淀：「参照无 Y MUST 独立复验」「机制断言 MUST 数据层实测」两条可复用判据
- [x] 被排除假说标注：writer-verdict 结论 3 以 supersedes 记录保留（原文不删不改），b3 P-4 预测标 ❌ 属 judge C5（主会话收敛时执行）
- [x] 载体正确：#40/#41 → workflow-patterns；#17 → algorithm-fingerprints；无 docs 主题篇内容（课题结论归 verdict/ore 课题，非本草稿职责）
- [x] 数字全部来自主会话提供的实测记录（verdict / p4 / judge 三文），无编造无占位符
- [x] 格式与目标文件末尾现状对齐（#37-#39 与 algorithm-#16 格式逐字段比对；编号续接 #40/#41/#17 已核空位）
- [x] 未改任何正式文件（本草稿为唯一产出）
