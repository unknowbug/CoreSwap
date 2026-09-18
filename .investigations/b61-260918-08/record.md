# B6-2 续块：开关缺口收口（260918-08）

```yaml
status: draft
block: 260918-08
role: 主会话
verification_layer: Partial（门禁实测 + wrapper 消费面一手核对；补映射后发射面验证待做）
```

## 1. 前提验证（交接结论验证纪律，已执行）

- 交接记录（NEXT_SESSION 260918-07 / b61 extension-per-version-gap.md）称 1.21.6 缺口 **21** 项。
- 廉价独立验证（`python scripts/check_switch_mapping.py`，260918-08 20:4x）：**1.21.6 缺口实测 11 / 1.20.1 缺口 13**。
- 判定：交接数字已漂移（260918-02 B6-2 前置块 + 260918-03/04 后续扩面已补 10 项、且门禁消费面新增 wrapper 捕获），**以门禁当前实测为权威清单**。

## 2. 当前 per-version 缺口全清单（.tmp/260918-08/pv_gaps.py 实测）

```
[1.20.1] gap=13（全部 union-orphan）
  bench.threads / bench.worldgen / chunkRandom.seed / chunkRandom.seed288 / colDump.targets /
  coreswap.bulkwb(wrapper) / coreswap.light.blockabi / coreswap.light.oldcollect / coreswap.skipair(wrapper) /
  coreswap.wbcheck / height.x / height.z / java.io.tmpdir
[1.21.6] gap=11（全部 union-orphan）
  = 上列去掉 coreswap.light.blockabi / coreswap.light.oldcollect（1.21.6 已声明）
wrapper-captured: coreswap.bulkwb, coreswap.skipair
```

## 3. 定性状态对照

| 项 | 既有定性 | 来源 |
|---|---|---|
| bench.threads / bench.worldgen / colDump.targets | SCOPED（豁免子句①②） | b61 t4-adjudication §3.4 |
| chunkRandom.seed / seed288 / coreswap.wbcheck / java.io.tmpdir / height.x / height.z | SCOPED | b61 t4-adjudication §3.2 |
| coreswap.light.blockabi / oldcollect | SCOPED（light 探针族零 -P 先例 + javadoc -D 直传） | b61 t4-adjudication §3.2 注 |
| （1.21.6 侧同批 9 项） | SCOPED（#174 记载：缺口 21→9 全 SCOPED） | b62-260918-02/t4-adjudication-1216.md |
| **coreswap.bulkwb / coreswap.skipair** | **未定性——本轮 T4 对象**（260918-04 门禁 wrapper 捕获后新进消费面；260918-02 时 :188 诚实边界「消费面未计」） | 本块 |

## 4. T4 分叉（fan-out 预置触发）

bulkwb/skipair 落在判据分叉点：
- **DEFECT 侧**：bulkwb 族四项两版均有 -P 先例（1.20.1 build.gradle:176-179 / 1.21.6:152-155），#174 跨载体延伸 ⇒ 对称性指向 DEFECT；bulkwb=0 是生产写回路径即时回退开关（260913-01 翻转 judge 条件「回退开关在位」）。
- **SCOPED 侧**：豁免子句（诊断/实验专用 + 缺省权威——BULKWB_ON/SKIPAIR_ON 常量默认 + flag(prop,def)「未设→默认」）；历史全部 A/B 实验清一色 `-D` 直传（07 篇 1343/1345）。

⇒ ≥2 互斥候选，按计划预置 fan-out：.b1（DEFECT）/ .b2（SCOPED）并行 worker，主会话汇聚裁决。
- b1: 518d27f8-dbcc-4c53-9576-293ba64c66d8
- b2: f9376754-5e62-4d07-877f-d294be8cec8b

## 5. 汇聚裁决（260918-08，主会话 T4）

### 5.1 输入

- .b1（DEFECT 双项）：`.candidates/.b1-defect-bulkwb-skipair.md`
- .b2（SCOPED 双项）：`.candidates/.b2-scoped-bulkwb-skipair.md`
- **既有 confirmed 结论**：`.investigations/b61-flag-coverage-260918-04/record-260918-04.md` §定性/§judge —— 两项在 260918-04 已定性 **SCOPED-DIRECT-D（ORPHAN 显形非缺陷）**，judge PASS-with-conditions（SHOULD-1..5 闭合），用户 2026-09-18 授权 confirmed。b2 回复中「从未定性」的说法不准（260918-04 已定性，彼时状态即含 judge+confirmed）；本轮实为「新证据复核既有 confirmed」。

### 5.2 裁决：维持 SCOPED-DIRECT-D（无取代，§15.4）

- **b1 论点①（族先例封死 SCOPED）不成立**：#174 先例取证域 = 该消费点。bulkwb 族 4 个 -P 先例（1.20.1:176-179 / 1.21.6:152-155）服务仪器消费点（BulkWb.java:91-97）；本体消费点（BulkWb.java:90 / CppBridge.java:642 路径选择器）两版全载体零 -P 先例 ⇒ 落 #174「全载体零先例 → SCOPED 侧」。b2 的消费点切分有 #174 原文支撑。
- **b1 论点②（豁免①不成立）字面成立但非既适用读法**：T4 §3.4 豁免先例 `bench.threads` 同为「诊断性覆盖、非运行必经参数」而非探针输出参数——bulkwb/skipair 同构（A/B 覆盖开关 + 缺省权威 + 历史全部实验 `-D` 直传有案：07:1240/1343/1345、260913-01 三臂/九臂）。豁免①②合读成立。
- **b1 论点③（回退通道不可达）无证据且被在案记录反驳**：260913-01 九臂经裸 `-D` 直传 bulkwb 族完成 A/B，通道有运行记录；b1 自认该承重推理未实测（weakest 环节）。⚠️ §9.7 口径声明（judge S1）：历史 `-D` 实验记录的直传通道 = **vmArg/JAVA_TOOL_OPTIONS 注入形态**，与「gradle 单命令行裸 `-D` 进 fork JVM」**不同口径、不可互证**——本裁决只证「回退通道在 vmArg 口径可达」，单命令裸 `-D` 可达性未实测（可选加固见 §5.3）。
- **b1 对 260918-04 SHOULD-4 的批评系误读**：#171 分工 = 结构事实定缺陷边界、文档定豁免边界；「javadoc 未宣传」用于豁免侧与 #171 一致，非 T4 已否决的「javadoc 定缺陷」判据。
- **结论**：b1 未提供足以取代 confirmed 的新证据，**无 §15.4 取代**；.b1 候选保留不采纳。

### 5.3 dissent 与分界细化（b1 候选的保留价值）

- **豁免子句读法分歧登记（judge S2）**：t4 §3.4 原文「满足**下列任一**」vs 任务书/#171「合读（AND）」措辞存在分歧，汇聚层按 AND 读法执行（本两项在 AND 下：skipair ①②均过、bulkwb ①扩展读法+②过）；若按任一（OR）读法结论不变（b1 自己亦确认「AND/OR 读法下结论相同」）。措辞统一属 t4 判据文本维护项，非本块义务。

- **dissent 注记**：bulkwb 豁免①依赖「A/B 实验专用」扩展读法（本体本质是生产回退开关）——若未来按字面严格读，本体可能落 DEFECT（届时仅豁免②单腿）。本轮按在案 bench.threads 先例读法维持 SCOPED。
- **分界细化（并入 260918-04 SHOULD-4 族内分界）**：bulkwb 族 -P 先例**不可跨消费点借用**——先例（仪器读取开关 :91-97）不覆盖本体（路径选择器 :90/:642）；「同族两判」的边界 = 消费点切分，非开关名前缀。
- **后续可选加固**（非本轮义务）：一轮发射面实测（`-Pbulkwb` 补映射与否的行为化判别 / 裸 `-D` 进 fork JVM 直证）可同时闭合 b1 weakest 与豁免①读法张力；属用户口径决策（260918-04 既定：是否补映射属用户口径决策）。

### 5.4 B6-2 总收口

- **本块产物清单（judge S3 补列）**：`.investigations/b61-260918-08/`（record + candidates/.b1 + candidates/.b2 + judge-review）+ `.investigations/000-架构设计/架构计划-260918-08-B6-2开关缺口定性补齐.md` + `.tmp/260918-08/pv_gaps.py`（derived）。

- 当前 per-version 缺口（1.20.1=13 / 1.21.6=11）**逐项全部已有定性，零 DEFECT 残留**：
  - 9 项（1.21.6）/ 11 项（1.20.1）SCOPED = b61 t4 §3.2/§3.4 + b62-260918-02 t4-adjudication-1216（#174 记载缺口 21→9 全 SCOPED）
  - 2 项（bulkwb/skipair，双版共有）SCOPED = 260918-04 confirmed + 本块复核维持
  - 1.20.1 独有 2 项（light.blockabi/oldcollect）SCOPED = b61 t4 §3.2 注
- **无映射需要补；门禁维持报告级；脚本零改动**（260918-04 口径：是否补映射属用户口径决策，维持报告级不入 --strict 修复范围）。
- NEXT_SESSION 260918-07 开工点 2「B6-2：1.21.6 侧开关映射补齐」的前提（缺口 21、含 stallwatch 等真缺陷）**已由 260918-02/03/04/06 各块陆续闭合**，本块为终局对账与 T4 分叉收口。
- 交接数字漂移链：260918-01 记 21 → 260918-02 补 10 项后余 9（+wrapper 显形 2 = 11）→ 本块实测 11。
