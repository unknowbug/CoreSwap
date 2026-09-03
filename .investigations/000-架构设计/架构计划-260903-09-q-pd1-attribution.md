---
编号: 260903-09
任务: Q-PD1 归因——Rust vs Java 端到端 ~2.2× 差距分阶段定位
任务类型: 性能归因（测量 + 分析）
模式档位: 轻量
状态: 待批准
---

## 范围（含明确不做什么）

- **做**：对 Rust 全管线（worldgen chunk 生成）按阶段（density / aquifer / ore / surface / carver / features …）拆分耗时，与 Java 对应阶段对比，定位 2.2× 差距大头（候选段）。
- **不做什么**：不改任何生产代码路径（零退化铁律，env 门控诊断不进热路径）；不做 GPU 优化实现（gpu-batch-merge 后续包）；不解 Java 55→33ms 漂移（遗留 idk，仅顺带记录）。

## 任务拆解（子任务 → 预期产物）

1. **基线廉价独立复核（交接结论验证纪律，MUST 先做）**：上轮「Rust OFF ~72ms / Java 33ms / 2.2×」是交接方向性结论——先复跑 `pc_e2e_bench`（256 chunks，同 seed 同口径 §9.7 声明）确认基线仍成立，再谈归因。产物：cmd-output 复跑记录。
2. **分阶段计数采集**：优先用「无探针整批 wall + 调用次数计数」口径；WG_PHASETICK（QPC 单次、默认关、不污染）作为单线程分阶段补充。Rust 侧按 stage mask 逐段关闭/开启做差分（stages 计数法），Java 侧引用已有探针数据/按需补采。产物：cmd-output + .investigations/lossless-accel/q-pd1-260903-09.md 过程记录。
3. **归因分析（收敛型，主会话）**：阶段耗时表 → 定位占比最大段 → 出结论 artifact（draft→candidate）。
   - ⚠️ fan-out 预置：若分叉出 ≥2 个互斥根因候选（如「features 段慢」vs「carver 段慢」且机制解释互斥），MUST 并行 fan-out worker 出 .bN 候选，禁止主会话自推。

## 验证方式

- 复跑基线与分段测量的 seed 三查（worldSeed=8576294172403134396）+ §9.7 可比性三要素声明（载体/覆盖面/历史口径）。
- 差分自洽检查：各段之和 ≈ 全管线 wall（不自洽 = 测量工具问题，先修工具）。

## judge 预置

- 收尾交付 MUST judge（三源核对：artifacts 快照 + git diff + 验证记录）。
- candidate 授予 SHOULD judge。

## fan-out 预置

- 触发点：归因分析若出现互斥候选分叉 → core.fanout 并行 worker，.bN 候选落 .artifacts。

## 知识库更新

- 结论性 docs（10 时间线 260903-09 节）/discovered（若有新判错/判据）→ **subagent 产出草稿**（prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md 指引）+ 主会话应用验证。

## 子角色介入点

- scout: 否（管线阶段结构已知，Q-PD1 是测量归因而非机制未明勘探；若分段结果与已知管线图冲突则升级为 scout 勘探）
- worker: fan-out 触发时的候选分析 worker；知识库草稿 worker
- fan-out: 归因候选分叉 ≥2 互斥机制时 MUST
- judge: 收尾 MUST + candidate SHOULD
- knowledge: 结论定稿后 subagent 产出落盘

## 后续衔接（不在本包范围，仅备忘）

Q-PD1 结论 → 决定 gpu-batch-merge 的 Amdahl 上限与 n 扫描/①prefetch 取舍。
