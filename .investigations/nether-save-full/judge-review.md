# judge-review — nether 存档写入口径 Full 化（260901-03）

> judge 只出意见，不改任何 status；confirmed 留给人类。
> 三源核对：① 产物快照（架构计划 / facts / .b1-.b3 / residual-interpretation）② 工作区改动（runtime/1.20.1/java/.../ReadWorldProbe.java 逐行核对，BlockProbe.java 写入端对拍；git HEAD=dd24e1e 无已跟踪改动，与声明一致）③ 验证记录（.tmp/ 4 个 v2 log + 2 个 compare 输出，全部实读）。

## 一、交付声明审查（核心数据）

| # | 项 | 结论 | 依据 |
|---|---|---|---|
| 1 | seed A 内存=存档读回 99.9376% / MCA 99.9278% | **PASS** | v2 log L446 match=1047922/1048576；reconfirm log L234 同值 1047922；compare_nether_seedA_rust.txt L17 GRAND TOTAL 1047819/1048576=99.9278%。三口径互洽（1047922-1047819=103=cave_air 簇，精确对账） |
| 2 | seed B 三口径 93.5156% 精确同值 | **PASS** | v2 log L444、reconfirm log L234、compare L17 均 980582/1048576 |
| 3 | §9.7 口径三要素声明 | **PASS** | 载体（MCA+内存双载体）/覆盖面（4×4@3200,3208 双 seed）/与 96.44% 探针口径不可比——三要素齐备，架构计划 L17 预置项已兑现 |
| 4 | 真实 Rust 参与证明 | **PASS** | 两 seed v2 log 均 `enabled=true`（L235/237、L233/235）+ 64 条 `populateNoise(nether) intercepted`（覆盖目标 4×4 及 feature 蔓延邻域），与作废 run（enabled=false、0 条）形成有效区分 |

## 二、ReadWorldProbe.java nether 改动逐行核对

| # | 项 | 结论 | 说明 |
|---|---|---|---|
| 5 | header 跳过（L50-52：int+long+int×5） | **PASS** | 与 BlockProbe L476-482 写入序（magic/seed/size/originX/originZ/minY/height）逐字段对齐 |
| 6 | 索引换算 by=k/256, z=(k%256)/16, x=k%16（L57） | **PASS** | 写入端 BlockProbe L920-927 为 y 外层/z 中层/x 内层，读序一致；writeShort↔readUnsignedShort 对称 |
| 7 | 动态 min_y/height（L42-43）与层统计（L47,62-63,82-85） | **PASS** | layerTotal/layerMatch 按 world height 分配，`by` 上界=height-1（循环上界 16*16*height），无越界；log 实证 min_y=0 height=256、seed B y≥128=100% 合理 |
| 8 | biome 段跳过（L73，256×readUTF） | **PASS** | BlockProbe L929-934 每 chunk 写 256 UTF（y=100，nether 域内合法）；读侧不解释内容，序无关 |
| 9 | `_nether` 参照后缀（L33-34） | **PASS** | 与 BlockProbe L458 一致 |
| 10 | **header 字段未校验**（seed/minY/height 读后丢弃） | **WARN** | 若 `-Dbench.seed` 与 ref 文件不符或 ref 为 overworld 格式而 world 为 nether（height 384 vs 256 → layerTotal[yIdx] AIOOBE），前者**静默错比**、后者异常中止。b2 声称「seed/origin 从 ref 文件内读，天然防 seed 错位」与代码**不符**（文件名/seed 实际来自 -D 属性，L25-34）——见 #16。建议：读 header 后与 `bench.seed`/`world.getBottomY()/getHeight()` 断言，不符即 fail-fast |
| 11 | 末地/TheEnd 边界 | **WARN** | `dim` 非 "nether" 一律走 overworld 分支（L32,41）且 ref 无 _the_end 后缀——传 `the_end` 会静默用 overworld+错参照。建议显式守卫（非 overworld/nether 即报错退出） |
| 12 | 死常量 MIN_Y/HEIGHT（L22） | **INFO** | 已被动态值取代，建议删除防误用 |

**总评**：改动核心逻辑正确（#5-9 全过 + seed B 三口径同值 + seed A 内存=读回同值双重运行时背书），边界校验欠缺（#10-11）不阻塞本批数据有效性（本批 ref/world 均为同维度生成、seed 三查在场外核对）。

## 三、过程结论审查

| # | 项 | 结论 | 说明 |
|---|---|---|---|
| 13 | 结论① cppWorldgenDir 传错一层 → wg_create=0 → 三场 run 实为 vanilla、旧数据全部作废 | **建议 candidate** | b2 追加段有日志铁证（enabled=false + intercepted 0 条）+ ctypes 直连复现（错层返回 0/对层非 0），机制、推翻理由、取代关系齐备 |
| 14 | 结论③ MCA vs 内存差 103 cave_air 簇（尾随阶段候选，未闭合） | **保持 draft（正确）** | v2 下「内存=读回精确同值、MCA 多 103」是新形态矛盾（非旧 gen1/gen2 形态），b1/b3 均未闭合此条；residual-interpretation §3 #5 探针方向合理，挂待查诚实 |
| 15 | 残差解读分类占比与优先级 | **分类 PASS（数据直读，占比可复算）/ 机制解释保持 draft** | A1=640/757=84.5%、B1=52078/67994=76.6% 与 compare top-mismatch 逐条对得上；§4 诚实声明 Partial/Degraded 合规；优先级排序（块数×可定位性）理由成立。但见 #17 修正项 |

## 四、发现的问题（需处理）

| # | 项 | 级别 | 说明与建议 |
|---|---|---|---|
| 16 | b2 子候选①一处论据与代码不符 | **FAIL（局部）** | b2 声称 ReadWorldProbe「seed/origin 从 ref 文件内读（非命令行拼），天然防 seed 错位 ✓」——实际文件名与 seed 全部来自 `-D` 属性（ReadWorldProbe L25-34），header 读后丢弃（L50-52）。其「改动排除」总结论方向仍成立（#5-9 + 运行时背书），但该条论据必须更正；同时正因无 header 校验，#10 的 fail-fast 建议优先级升高 |
| 17 | residual-interpretation §1-A2 引用作废 run 数据 | **WARN** | A2 段「gen1/gen2 内存态不一致 → 跨运行不确定性（M4 家族嫌疑）」引用的是**已作废 vanilla run** 的现象（CppBridge=false），对 Rust 生成无证据力；v2 双 seed「内存=读回精确同值」反而**削弱** M4 嫌疑。建议该段改写：跨运行不确定性信号降级为「作废 run 遗留观察，不构成 Rust 侧证据」；M4 复核降为可选 |
| 18 | seed 三查在 reconfirm run 记录不完整 | **WARN** | v2 gen log 有 CppBridge init seed 回显（=bench.seed 属性）；但 reconfirm log 无任何 seed 回显，ref header seed 仅在 compare 输出回显，level.dat/server.properties 核对无直接落盘记录——目前靠「ref seed 同值 + 99.9% 匹配率不可能错 seed」间接背书。本批**接受**，但按探针/参照核对铁律（seed 三犯前科），建议后续 run 在 probe 输出加一行 seed/level.dat 回显 |
| 19 | 错误台账缺失 | **WARN** | SUBAGENT-KNOWLEDGE-GUIDE 要求错误独立成篇 `<课题>-errors.md`（五段式+速查表）；本课题 cppWorldgenDir 传错层 + 「run 后未检查 enabled 标志」两错误只散记于 b2 追加段（非五段式、无独立台账、无速查表行）。尤其「未查 enabled」违反探针/参照核对铁律精神，五段式教训（**含 CppBridge 静默降级不 fail-fast 的管线级教训**）值得完整沉淀，建议补 errors 台账 |
| 20 | §15.4 取代链只有单向指针 | **WARN** | facts 文件（原结论）头部无「已被 v2 run 取代（supersedes→b2 追加段/v2 log）」回填标注，取代关系仅存在于引用方文档。按双指针要求建议在 facts 文件顶部补一行取代标注（原正文不改） |
| 21 | WGB2 header 教训（min_y=0 height=256 vs 猜 128） | **INFO（简记即可）** | 价值门中低档：错在开工前未读 DimensionType 而靠猜；代码已动态化（world.getBottomY()/getHeight()）部分结构化该教训。建议并入 #19 的 errors 台账一行判错经验（「维度高度/最小 Y 勿猜，从 DimensionType/world 运行时读」），不单独进 knowledge/discovered |
| 22 | b2 探针判据「恰好 16 条 intercepted」与实测不符 | **INFO** | 实测正确 run 为 64 条（4×4 目标 + feature 蔓延邻域均拦截）。该判据照本宣科会误杀正确 run，建议改为「≥16 且覆盖目标 4×4 chunk」 |
| 23 | 本课题无 .artifacts/index.yaml 条目 | **INFO** | 结论均为过程性/候选级，落 .investigations 可接受；若 #13/#15 升 candidate，建议补 index.yaml 登记以满足 core.artifact 契约 |

## 五、结论状态建议汇总（供主会话/人类裁决，judge 不改 status）

- **可升 candidate**：双 seed Full 口径数据 + §9.7 声明（#1-4）；ReadWorldProbe nether 核心改动正确性（#5-9，附 #10-11 建议项）；过程结论①（#13）；残差分类占比本身（数据直读部分）。
- **保持 draft**：103 cave_air 尾随簇机制（#14，挂待查正确）；B1 surface rule 机制解释、A1「未实现 vs 错位」归属（residual-interpretation §1 各机制段）。
- **必留 draft/待查**：M4 家族在 Rust 接管后的风险登记（b3 追加段定位正确——降级为风险登记而非现象解释，本 judge 认可）。
- **前置修正后再议**：#16（b2 论据更正）、#17（A2 表述修正）不阻塞 candidate 但应在下一版文档中体现。
