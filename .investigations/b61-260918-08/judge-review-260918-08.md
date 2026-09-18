# Judge Review — B6-2 收尾工作块 260918-08（core-judge，Anchorlaw §15/§16）

```yaml
status: draft
role: judge
block: 260918-08
review_scope: 三源核对（产物快照 / git diff / 验证记录复现）
verdict: PASS-with-conditions
```

## 0. 三源核对执行记录（judge 基线）

1. **产物快照**：已读 `.investigations/b61-260918-08/record.md`（§5 汇聚裁决）+ `.candidates/.b1-defect-bulkwb-skipair.md` + `.candidates/.b2-scoped-bulkwb-skipair.md`（均 draft-candidate，未采纳/保留，无 status 越权）。
2. **git 工作区**：`git status --porcelain` = 仅两处 untracked：`.investigations/b61-260918-08/`（本块产物）与 `.investigations/000-架构设计/架构计划-260918-08-….md`（Phase 0 计划）；`git diff HEAD --stat` 为空（无任何 tracked 文件改动）。**「脚本零改动、无映射补」与工作区事实一致**（见 SHOULD-3 关于计划文件超出声称范围）。
3. **验证记录复现**：`python scripts/check_switch_mapping.py` 复现 **1.20.1 缺口 13 / 1.21.6 缺口 11，全部 union-orphan，wrapper 捕获恰 `coreswap.bulkwb` + `coreswap.skipair`**；`python .tmp/260918-08/pv_gaps.py` 复现全清单逐项与 record.md §2 一致。实测数字与声称完全吻合。

## 1. 审查面 A — 汇聚裁决证据链（维持 260918-04 confirmed，无取代）

**结论：成立。**

- **消费点切分事实一手核对（PASS）**：共享树 `java-core\src\main\java\wg\bench\` 一手 grep——`BulkWb.java:90` = `ON = WgCompat.flag("coreswap.bulkwb", WgCompat.BULKWB_ON)`（本体，wrapper 形态）；`:91/:95/:97` = `bulkwblog/bulkwbtest/bulkwbsentinel` 字面量 `System.getProperty`（仪器链）；`CppBridge.java:642` = `BULKWB` 本体第二消费点、`:621` = `SKIPAIR`。b2 声称的「仪器消费点 :91-97 / 本体 :90/:642」切分**与源码逐行如实**。
- **#174 判据原文核对**：`knowledge/discovered/workflow-patterns.md:2995-2999` 明文「任一载体已为**该消费点**建立 `-P` 先例 ⇒ DEFECT；仅当**全部载体**均零先例才落 SCOPED 侧」。b2 将「该消费点」读为精确 property 消费点（非族粒度）有原文支撑；b1 的「族先例跨消费点外推」超出 #174 字面。主会话采纳 b2 切分 = 判据原文内的读法，非新造规则。
- **§15.4 取代链核对**：260918-04 record :22-25/:34/:44 证实两项既有定性 SCOPED-DIRECT-D + judge PASS-with-conditions + 用户 2026-09-18 授权 confirmed。b1 未提供新数据层证据（其最弱环节自认未实测，见 .b1 §5），**无取代条件成立**，维持既有 confirmed 是唯一合 §15.4 的裁决。
- **（备注）审查任务书路径勘误**：任务书写消费点在 `versions\1.20.1\java\src\main\java\wg\bench\BulkWb.java`——该路径**不存在**（两版 java 树下均无此文件）；实际权威路径 = 共享树 `java-core\src\main\java\wg\bench\`（两候选文件用的正是此路径，正确）。此为任务书笔误，非产物缺陷。

## 2. 审查面 B — 豁免子句合读

**结论：类比成立，dissent 已如实登记。**

- t4-adjudication §3.4（:67-86）核对：豁免先例 `bench.threads` 的豁免理由 = 「诊断性覆盖，非运行必经参数」；record §5.2 将 bulkwb/skipair 类比于此（A/B 覆盖开关 + 缺省权威 + 历史全 `-D` 直传有案）——与 §3.4 原文先例形态同构，类比成立。
- b1 的 dissent（豁免①对 bulkwb 依赖扩展读法：本体本质是生产回退开关，严格字面读可能落 DEFECT、届时仅豁免②单腿）在 record §5.3 **保真转述**——与 .b1 §2.1 原文立场一致，且保留了「后续可选加固：一轮发射面实测可同时闭合 weakest 与读法张力」的出路。

## 3. 审查面 C — 引用完整性抽查（v0.22 §1.3 anchor 档，属主方式）

抽查 3 处，全对：
- **#174**（workflow-patterns.md:2995）——「该消费点」取证域表述与 b2 引用一致（见 §1）。
- **#171**（:2897）——「判据与其豁免子句 MUST 合读」确实存在且方向与 b2 引用一致。
- **#131**（:2266）——「等价证据形态 = 前提全格实证 + A/B 双臂」与 b2 对 skipair 证据形态的引用一致。
- **260918-04 SHOULD-4**（record-260918-04.md:25）——「族内分界须显式登记，防同族两判疑问」；本块 §5.3 分界细化（先例不可跨消费点借用，边界 = 消费点切分）是对 SHOULD-4 的正当并入，非篡改。

## 4. 审查面 D — 覆盖面与降级声明

**结论：无越界。** record.md yaml `verification_layer: Partial`；全文无「回退通道当前可达/不可达」类未实测断言——§5.3 明确将发射面实测列为「后续可选加固（非本轮义务）」并归用户口径决策。b1/b2 均 Degraded 且各自声明诚实边界。门禁运行记录与 per-version 复现脚本均可重放（本 judge 已重放，数字一致）。

## 5. 审查面 E — 反方处理（.b1 不采纳可审计性）

**结论：可审计。** record §5.2 对 b1 三个论点逐一给出驳回理由（①#174 取证域字面不支持族粒度外推；②豁免①「非既适用读法」承认字面张力但按 bench.threads 在案先例读法；③weakest 环节 b1 自认未实测）。.b1 候选保留不删、dissent 落 §5.3——符合 §15.4「原结论/候选不删不改」与 fan-out 汇聚纪律。

## 6. 意见列表

### MUST（无）

无 MUST 级问题。三源一致、判据引用属实、取代链合 §15.4、无 status 越权、无结论越界。

### SHOULD-1：record §5.2 对 b1 论点③的反驳措辞略强于证据

- **依据**：§5.2 称「b1 论点③无证据且被在案记录反驳：260913-01 九臂经裸 `-D` 直传完成 A/B，通道有运行记录」。在案历史 A/B 走 `-D` 直传是事实，但**历史用法记录不等于「裸 `-D` 进 fork JVM（gradle runServer 单命令口径）可达」的同口径证明**——b1 的不可达论断本身也承认未实测，双方在同一未实测点上各执一侧。
- **建议动作**：record §5.2 该行补一句口径声明（§9.7 形态）：「在案 `-D` 实验记录与『gradle run 单命令裸 -D 可达性』非同口径，本块两侧均未实测，不影响定性（DEFECT/SCOPED 判据不依赖可达性），仅影响严重度叙事」。不阻塞。

### SHOULD-2：「豁免任一 vs 合读 AND」措辞分歧未在汇聚层登记

- **依据**：t4 §3.4 原文（:71）为「满足下列**任一**」，任务书/#171 为「两条合读（AND）」；b1 §5 已登记此分歧为次弱项，但 record.md §5 汇聚裁决未再登记（§5.2 直接按合读口径裁决）。本轮两项在两种读法下结论相同（b1 自己也承认），无实质影响。
- **建议动作**：record §5.3 追加一行登记该措辞分歧「两种读法下本轮结论不变，留待判据文本下次维护时统一」。不阻塞。

### SHOULD-3：git 工作区新增面与声称范围有一处超出

- **依据**：本块声称新增限 `.investigations/b61-260918-08/**` 与 `.tmp/260918-08/**`；实际另有 untracked `.investigations/000-架构设计/架构计划-260918-08-….md`（Phase 0 计划文件，属本块但不在声称清单内）。无 tracked 改动，「脚本零改动」核心声称不受影响。
- **建议动作**：声称清单补列该计划文件，或后续 commit 时一并入库。不阻塞。

### SHOULD-4（备注级）：收口结论无 .artifacts/index.yaml 条目

- **依据**：index.yaml 有 260918-03 条目但无 260918-04/260918-08——本块延续 260918-04 的「仅 .investigations 落盘」先例，非本块新引入的违规。
- **建议动作**：B6-2 终局归档（docs/ 时间线 + 台账）时一并决定是否为 04/08 两块的定性结论补 index 条目。

## 7. 总评

**PASS-with-conditions**。

- 汇聚裁决（维持 260918-04 confirmed、无 §15.4 取代）**证据链完整且经一手核对成立**：消费点切分与源码逐行相符，#174 原文支持该读法，b1 无新数据层证据。
- 门禁数字（13/11、全 union-orphan、wrapper 恰两项）已由 judge 独立复现，与声称一致；git 工作区与「零改动」声称一致（除 SHOULD-3 一处计划文件清单外溢）。
- 4 条 SHOULD 均为措辞/登记级，不改变任何定性，不阻塞收口；confirmed 授予权留给宿主人类。
