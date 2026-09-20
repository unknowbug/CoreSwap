# judge review-260920-01 — E-3a verdict 审查（260920-01）

**结论：PASS-with-conditions**（条件 N1-N3/N6-N8 已于同日应用；推荐 candidate，confirmed 留用户）

- N1（判据旁路风险）：P09v2 降级实质改写「前置失效 → 数据不进判读」语义，高风险 amend——缓解 = 总体结论未变 + 1518 仅登记为数据层证据。
- N2：amend 留痕与声称不符 → P09v2/P11 已回写 e3a-criteria.md §5（原文不改 + 加注）。
- N3：[SELFCERT] 原始行未落盘（stdout only）→ verdict P01-P06 行已显式声明，读数以 result json 为准。
- N4：「全部可观测面」→「可观测面（限 S=1518）」已改。
- N5：P09 降级诚实性合规（idk-E3a2 登记，两可能同处置）。
- N6：补稳健性声明——coverage 判定对 P07 v1/v2 口径均 <80%，不可判结论口径不敏感（criteria §5）。
- N7：取代链第二指针补入库载体（10-timewise-archive 260920-01 块）。
- N8：「结构性缺席」→「两臂系统性缺席观测」措辞修正。
- N9：FA-2 维持 suspended、保守默认行落点正确。

审查对象 = verdict-260920-01.md；三源核对（快照/git diff 仅打点 2 文件/两臂 log 3703 行 ×2）实测通过。
