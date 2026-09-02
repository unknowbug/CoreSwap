# Judge 审查意见 — 极端坐标 FP 微差应力测试 verdict-260902（封存拍板的证据支撑）

> judge 角色（subagent 隔离）：只出意见，不改 status。用户已预先拍板「同意封存」，本审查 = 核对该拍板的证据支撑是否充分。
> 审查基线（三源）：① `.artifacts/extreme-coord-stress/verdict-260902.md` ② 证据抽查（cmp_*.out.txt ×3 + log_run2b/7/8 + server.properties）③ 批准计划 `.investigations/extreme-coord-stress/architecture-plan.md`。

## 逐项结论

**1. verdict 数字 vs cmp_*.out.txt — PASS**
抽查三区完全吻合：区① mismatch=16304 / agree=98.9634（cmp_run1 行5）✓；区④ 2295 / 99.8541（cmp_run4 行5）✓；对照 22156 / 98.5914（cmp_ctl 行5）✓。失配明细可读（如 run1 的 ref=1/34 → cpp=9 泥土带形态、run4 的 ref=1→cpp=0 擦边散簇），与 verdict 形态描述一致。

**2. 判据应用忠实度 — PASS**
95% 预登记参考线实际最差 98.59%（对照区），四极限区 98.85–99.85%，全部远高于参考线；形态判别（孤立格/小片 vs 连片翻转）与明细吻合（④最大簇 522 块，非整 chunk 翻转）；「先报数值再裁决」流程已履行——verdict 首节即列实际一致率并标注用户已拍板，裁决权在用户侧未被 AI 越权（confirmed 未被 AI 授予，状态链合法）。

**3. 归因推理（泥土带非极限坐标引起）— PASS（附一条已诚实声明的局限）**
对照设计质量良好：同 seed（+7159221168429822337，hdr 双侧核对一致 ✓）、同载体、同 4×4 口径，唯一变量 = 坐标（3200,3200 vs +29999936）。对照一致率 98.59% **低于**全部极限区，直接支持「失配主体不随坐标极端化放大」。坐标↔biome 混杂确实存在（对照区与极限区 biome 组成不同，泥土带是否随 biome 出现无法由本矩阵分离），verdict 对此是诚实的——遗留项标注「疑似 biome 驱动 surface rule 家族差异……未定位根因」，且归因结论措辞为「非坐标极端化引起」而非「已归因 biome」，边界划得准确。该混杂不影响封存主结论（封存的对象是「坐标极端化→地形颠覆」，对照已充分支撑其否定）。

**4. §9.7 三要素 — PASS**
verdict 头部量化指标同行声明：载体（BlockProbe WGB2 端到端，dll sha=68d7f401，与 B1/260902-10 同构建）+ 覆盖面（2 seed × 2 极限角 × 4×4 + 1 对照，每区 1,572,864 块）+ 可比性（与 260902-10 confirmed 同链路、overworld min_y=-64/height=384）——三要素齐备，与 B1 域差异已显式划界。

**5. 采样矩阵遗漏/污染风险 — PASS**
抽查 3 份日志：run2b（区① cpp）worldSeed=7159221168429822337 与 CppBridge init seed 一致、dll sha256=68d7f401 ✓、populateNoise intercepted 恰 16/16（chunk 1874996..1874999 × 1874996..1874999，含世界最后合法 chunk）✓；run8（区④ cpp）seed=-7159221168429822337、16/16 负极限 chunk（-1875000..-1874997）✓；run7（区④ ref）仅 BlockProbe worldSeed 行、无 CppBridge ✓。cmp 头部五要素（magic/seed/size/origin/miny/height）双侧一致 ✓。server.properties 当前 level-seed=8576294172403134396，已恢复默认 ✓（world 删除纪律以日志时间戳与逐跑 origin 各异间接印证，未逐跑核验属抽查范围内可接受）。

**6. 泥土带遗留项可见性 — PASS**
verdict 以独立 ⚠️ 章节 + 【遗留 · 未归因 · 与极限坐标无关】三重标签置顶级呈现，含 y 范围、失配量级、影响评估（地下不可见/表面零漂移）、未来触发条件（逐位 100% 对齐/地下依赖功能）——醒目程度达到「封存后不被遗忘」的要求。

## CONCERN（不阻塞封存，但需补齐）

**C1. 产物契约缺口：`.artifacts/extreme-coord-stress/` 无 index.yaml，根 `.artifacts/index.yaml` 亦无本课题条目**（core-artifact 落盘契约——登记缺失，verdict 目前处于「未入索引」状态）。

**C2. 计划 §6 预算的 facts-<date>.md 与 cmd-output/ 未见于本目录**；verdict 中「表面高度漂移 0/4096 列 / ④ 592 列 ±1~3 格」「④ 466 个散簇、最大 522 块」「泥土带 y 分带（①42-53/②32-47/③26-46/对照22-49）」等派生分析数字在 cmp_*.out.txt 中无直接落痕（cmp 文件止于 MM 明细，无统计汇总行）——推断由 cmp_region.py 临时分析得出但未落盘计算记录。这些数字不支撑封存主结论（主结论只依赖 agree% 与形态，已核对），但作为遗留项（泥土带）的描述性证据链不完整。

## 总体意见：**通过（封存拍板的证据支撑充分）**，附带 2 项需修正（均不改变结论方向）

需修正清单：
1. 补 `.artifacts/extreme-coord-stress/index.yaml` 并在根 `.artifacts/index.yaml` 登记本课题（C1）。
2. 补 facts 文件/cmd-output 落盘：记录表面漂移与簇统计的派生计算（脚本+输出），使 C2 数字可复现（C2）。

补充建议（可选）：泥土带遗留项在未来的入口 = verdict 已给（cmp 的 (ref,cpp) id 对分布）；若后续开「逐位 100% 对齐」课题，应先做 biome 归因 fan-out（对照矩阵当前无法分离坐标↔biome 混杂）。

> 推荐状态：维持 candidate（用户已拍板封存，confirmed 授予权在宿主人类——本意见仅为该拍板的证据充分性背书）。
