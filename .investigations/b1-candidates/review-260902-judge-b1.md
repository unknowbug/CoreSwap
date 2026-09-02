# judge 审查意见：B1 四候选判别（draft→candidate，SHOULD 级）
审查基线：① 产物 = .tmp/ 脚本 3 + 数据 CSV（4096 列在盘、sanity 行自洽）② git diff **未核（judge 无 shell，主会话补）** ③ 验证记录 = surface-layer-preprocess.txt（数量可复算：26 单元、14/14 列、映射表与 overlap_check.py 硬编码一致）。

## 逐条结论
- **C1 [支持]**：air 签名 99.68% + §9.7 三要素同行声明齐全（脚本头注释）。存疑点：结论只引 air 签名，compare_noiseonly.py 同跑「全材质精确匹配」一项未引用——若材质匹配列数 <4083，说明 NOISE 层有 air 不变但材质变的微差，应在正文注明（与 C3「13/4096 单元级微差」口径对齐：13 列是 air 差异列还是材质差异列，措辞需消歧）。
- **C2 [支持，附不确定度声明]**：投票映射自洽（0→0=173760、256→5850=255343 等主票绝对优势），量级结论稳。但映射由**同一对拍数据自举**（循环性）：26 个差异单元同时是次票来源，映射错票方向只会**制造假差异**（保守方向，利于上界成立）；假匹配方向要求某 rust id 真分布双峰且主票巧合，次票占比 ≤0.02% 风险可忽略。建议候选前用独立 id 表（block registry）核对 7 对映射一次，把循环性声明写进结论。
- **C3 [存疑→补证后支持]**：闭合 13/14 有 1 列 only-surface 未解释——与「单因链」表述冲突。两种出路：① 该列差异恰为投票次票伪差异（taxonomy 中 0↔5854 类）则闭合成立，需指出具体列号；② 真有 surface 独立差，则「单因」降级为「主因」。正文必须写明该列身份，否则降措辞为「13/14 闭合 + 1 列待归因」。
- **C4 [支持]**：PRE CSV biome 字段直接核得 basalt_deltas；signature A 划在区外、不外推，边界诚实。
- **C5 [支持]**：三要素（限历史测量/保留项清单/单 seed 单 biome 4×4 外推边界）齐备，放大系数未量化已如实声明。存疑点：污染判定依赖「cppReplace 下 CARVERS/FEATURES 仍跑、stageMask 只控 Rust」机制断言，正文需给出该机制的落盘证据引用（log/代码行），目前仅结论转述。

## candidate 建议
**建议授予 candidate（条件项见下）**；confirmed 留人类。

## 主会话补做清单
1. git HEAD + 工作区 diff 三源核对（本 judge 无 shell 未做）。
2. compare_noiseonly 的全材质匹配数落盘并写进 C1 正文；消歧 C3 的 13 列口径（air vs 材质）。
3. 查明 overlap only-surface 那 1 列的列号与成因，写入 C3。
4. C5 污染机制断言补落盘证据引用（cppReplace/stageMask 出处）。
5. 结论登记 .artifacts/ + index.yaml（现仅存 .tmp 临时区，按 core.artifact 契约候选前须落盘）。
6. docs 落盘走 subagent 草稿流程（已规划，合规）。

## 触发点合规
candidate 授予 SHOULD judge——本次即审；confirmed 前 MUST judge 需另审（届时核对补做清单闭环）。
