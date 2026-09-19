# SHOULD-judge 审查记录：criteria-260919-03（T1.5，时序锚阶段）

- 审查者：judge subagent（6bdcc521，2026-09-19）
- 对象：.investigations/a1-b2-260919-03/criteria-260919-03.md（a376b4c 版）
- 判定：**PASS-with-conditions**（0 blocking / 2 M / 3 S）
- M1（C-3 门控回退自证盲区）：WG_DOMAIN_INLINE 无门 + packed=<n> 无数量下限门
  → onpk 臂可能 packed 路未被行使而全门通过，B-主归因被内联回退路稀释。
- S1：每臂最小成功 task 数无下限门。S2（并入 M1）。S3：补 idk-4（thread_local 驻留触发面）。
- info：B-C3 的 alloc_copyin 在 onpk 为 packed 三数组语义（C-3 效应代理，非同量工作）；
  B-C4 依据混入 idk-1 范围外份额，贴灰区上沿时 verdict 须引用 idk-1。

**应用记录（主会话，2026-09-19）**：M1 → preconditions 增 packed_coverage 门
（inline 占比 ≤30%，>30% VOID / 20-30% 灰区）+ packed 帧数下限（packed=<6 视同回退计数）；
S1 → preconditions 增 min_tasks 门（<100 判据 suspended 灰区）；S3 → idk-4。
修订后重走时序锚 commit；判据状态保持 draft（confirmed 留用户）。
