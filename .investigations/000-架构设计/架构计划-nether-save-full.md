---
编号: 000
任务: nether 存档写入口径 Full 化（block_probe 双 seed 存档级逐位对比）
任务类型: 验证（worldgen 对齐）
模式档位: 轻量
状态: 已批准（260901-03 session，用户批准）
---

## 范围（含明确不做什么）
- 做：nether 维度存档写入口径的 Full 验证（vanilla 导出 → 清 world → Rust 重生成 → compare_save_region.py），双 seed 对齐 overworld 口径
- 不做：残差深挖（soul_sand/gravel/熔岩湖边界，等本任务新数据再动）；微漂移 M4 家族；overworld 重复验证

## 任务拆解
1. 开工验证（廉价独立验证，≤1 轮）：核对 build.gradle L85 `blockProbeDimension`，跑 `-PblockProbeDimension=nether` 小样本（4×4@0,0 seed -8248），确认导出格式与 seed 三查 → 验证 NEXT_SESSION 交接方向
2. vanilla nether 参照导出：双 seed（对齐 overworld：-2032… / 8576…），16 chunk
3. Rust 侧重生成：清 run\world → cppReplace/readWorldProbe 同参数
4. compare_save_region.py 对比 → 对齐率（同行声明 §9.7 口径三要素：载体=存档写入 / 覆盖面=nether 双 seed×16chunk / 与 96.44% 探针口径不可比）

## 验证方式
- Full 分层（存档级逐位）；seed 三查铁律全程（read_seed.py）；GRADLE_USER_HOME 必设

## 子角色介入点（预置，执行只核对不补排）
- scout: 否（开工点与流程已明确，无需勘探）
- worker: 若对比出现新差异 → 差异解读交 core.worker
- fan-out: 对比若现多互斥差异机制候选 → MUST fan-out（.bN），禁止主会话自推
- judge: candidate 授予 SHOULD judge；收尾交付 MUST judge（三源核对）
- knowledge: 结论落盘（docs/台账）MUST subagent 产出草稿（先读 SUBAGENT-KNOWLEDGE-GUIDE.md），主会话只应用 + 验证

## 交接验证纪律
NEXT_SESSION 中「BlockProbe 维度支持待确认」为方向性结论（非已 judge 定论）→ 第 1 步即廉价独立验证，验证通过才继承后续流程。
