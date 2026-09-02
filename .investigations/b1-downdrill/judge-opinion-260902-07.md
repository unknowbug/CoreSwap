# core.judge 审查意见（H1 环1 证伪转向，260902-07）

> judge subagent（ca4ab08a-f7e7-4c3b-92d5-860a1d8a21bb）已独立核实 colprof-cmp-out.txt / colprof-firstsnap-out.txt / lavaaudit-v2-cmp-out.txt / b1_lavaaudit_cmp2.py / facts-06/07 / LAUIDMAP 原始行。只出意见，未改任何 status。

**A. 「H1 环1 被证伪」证据链充分性：PASS（附 1 CONCERN）**
三组证据正交收敛：① LAUIDMAP（Java STATE_IDS 权威）——19319=blackstone、5854=basalt，直接推翻原标注；旁证：原标注物理可疑（1.20.1 主熔岩海面应在 y≈31，lavaTopY 峰值 23~24 与之吻合）。② 快照时间线（firstsnap V#0/C#0 即异构）排除 feature 事后改列；`4|96->5854`（lava→basalt 接触面）两侧同构说明熔岩行为一致、差异只在固相材质与转换面 y。③ LAVAAUDIT v2 全扫 11,443 列：air→lava 面向两侧同为空——「V 有熔岩海而 C 缺失」前提在数据层不成立。替代解释（id 映射错 / 扫描漏测 / 熔岩海在区外）逐一排除。
CONCERN：LAUIDMAP 只在 v3 vanilla 轮，cpp 轮待补（封死「cpp 侧 id 空间不同」退路）。

**B. 昨日 judge 有条件 PASS 条件失效判定：PASS**
失效①：环4-5 的 LAVAAUDIT/回归升级路径连同条件 PASS 作废；失效②：回归判据「99|air->lava 恢复」整组须重写。保留：seed 三查声明仍为待补项。环2~5 作为现象成立（COLPROF 10 列 + cfg y=111/119/121 vs 99）；findPos y-零随机语义只需「第一转换面不同」，黑石 vs 玄武岩(+y 100~104) 满足——只需重定位因果入口，链体不推翻。同意候选降级 + §15.4 取代记录形态。

**C. fan-out 候选设计：CONCERN（可执行，两点须修）**
① (a)(b)(c) 非严格互斥（b→a 因果包含；c 与 a/b 同签名可能）——候选表述改「判别目标」，各 worker 判别探针须能独立给出排除/成立数据。② 遗漏第四候选 **(d) 前置地形形状差**（NOISE/density 列高度差）：V 黑石底恒 99 全平 vs C 玄武岩底 100~104 贴地形——若纯材质分支差 y 应一致，y 分布形态差暗示 C 侧地形顶面本身不同，材质差可能是次生。若 NOISE 后即分叉，(a)(b)(c) 全降次生。判别探针：SURFACE 前后逐列 dump（材质序列+biome id+顶面 y），一次采集分诊四候选。

**D. 风险与遗漏（CONCERN 级，不阻塞）**
① lavaTopY 逐列验证（已补：329/11443=2.9% 差、n 差 60 列）；② only_v=10/only_c=56 覆盖缺口未解释（§9.7 声明）；③ 结论限 3200,3208 区域，外推须声明；④ SURFACE 99.9423% vs 内部转换面差——取代记录按 §9.7 写明口径三要素；⑤ E-B1-9「标注三查」入 NEXT_SESSION 开工检查项；⑥ 首分叉行 320 伪影订正与本次转向一致。

**总评**：转向「H1 环1 被证伪、因果入口重定位」证据充分，同意按 §15.4 出取代记录候选（原 H1 正文不删不改）；环2~5 现象保留。下一轮四候选 fan-out，先跑 (d) 判别。confirmed 留人类拍板。
