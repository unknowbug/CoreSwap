# 草稿：260903-09（Q-PD1 包）知识库更新草稿（core.worker subagent 产出，主会话应用）

> 依据：SUBAGENT-KNOWLEDGE-GUIDE.md（价值门 + 五段式 + 载体映射）。
> 来源材料：.investigations/lossless-accel/q-pd1-260903-09.md + .artifacts/lossless-accel/qpd1-attribution-260903-09.md + cmd-output/{qpd1-stage-bench, qpd1-baseline-rustoff, qpd1-java-recheck}-260903-09.*。
> 价值门预判：坑 A / 坑 B = 高价值（判错经验/可复用判据）→ discovered/ 详记；结论摘要 = 中价值简记 → 10 时间线。

---

## 一、discovered/ 草稿条目

**载体建议**：两条均归 `knowledge/discovered/workflow-patterns.md`（判错经验/实验设计家族，与发现 #12/#13/#14/#18 同族），**不归 build-tooling.md**（build-tooling 编号当前至 #12，都是构建/工具链坑；坑 B 虽发生在 bench 工具源码，本质是「判别实验设计」方法论，不是工具链环境坑）。当前 workflow-patterns.md 最大编号 = **#18**（跨 session 基准数字不可直接续推），故本两条为 **发现 #19、#20**。

### 发现 #19: Java bench 前必须删 run\world——世界状态第四查；「快一个量级 + min≈0」是缓存假象签名（260903-09）

- **发现时间**：260903-09
- **发现者**：core.worker(subagent)
- **来源定位**：`.investigations/lossless-accel/cmd-output/qpd1-java-recheck-260903-09.md`（run A vs run B）；`.investigations/lossless-accel/q-pd1-260903-09.md` Step 3
- **置信度**：candidate（两轮实测复现：run A 764ms total vs run B fresh 10993ms；与 260903-08 Java 基线 33/11067/43.2 吻合）
- **观察（现象）**：Java WorldGenBench 在 `run\world` 残留时跑 region(200,200)：`total=764ms avg=2.98ms min=0 max=285`；同命令删 `run\world` 后 fresh 生成：`total=10993ms median≈32ms`。第一次测出「比 Rust 快 ~60×」的假象级数字。
- **根因**：`run\world` 残留时 WorldGenBench 走服务器 chunk 系统，region 内 chunk 昨日已生成 → 从磁盘加载（~1ms/chunk）而非走 worldgen 管线——测的是 IO 缓存不是生成性能。`benchSeed` 只改 bench 参数，不改变 level.dat 里的世界状态（与 seed 三查中「benchSeed 不改变世界 seed」同源）。
- **定位**：run A 输出 min=0 + total 764ms 偏离前日基线一个量级 → 对 260903-08 基线（33/11067）触发复核；对照 fresh 重跑即闭合。签名 = **同 region 二次 bench 快一个量级 + min 接近 0**。
- **修复**：bench 前 `Stop-Process java` + `Remove-Item run\world` 强制 fresh 生成。
- **教训/如何利用**：**判据升级：探针/参照数据采集核对四查**（原 seed/坐标/文件三查 + 新增第四查 **世界状态**）——任何依赖 `run\world` 的 Java 侧测量（bench/探针/导出），数据用于对比前必须确认 world 目录状态（fresh 生成，非残留）。假象识别签名：同 region 二次测量快一个量级 + min≈0 → 立即怀疑缓存/残留，不作结论。同族：AGENTS.md「残留 java 进程占 world/端口」条目（同一 `run\world` 问题的性能测量面）。

### 发现 #20: 死参数制造假判别——判别实验必须验证「自变量真被改变」（260903-09）

- **发现时间**：260903-09
- **发现者**：core.worker(subagent)
- **来源定位**：`.investigations/lossless-accel/q-pd1-260903-09.md` Step 1 附带发现；`.artifacts/lossless-accel/pc-results-260903-08.md`（被 supersedes 的「negseed 判别 <3% → seed 非因素」）
- **置信度**：candidate（源码证据：pc_e2e_bench.rs L18 解析 `WG_E2E_SEED` 后 L22 恒用常量 `SEED`；实测 negseed 运行差 <3% 恰因两侧同 seed）
- **观察（现象）**：260903-08 P-C1 用 `WG_E2E_SEED` 跑 negseed 判别，结果差 <3%，读作「seed 非因素」并记入 confirmed artifact。260903-09 bin 源码审查发现该 env 解析后根本没被使用，两次运行跑的是同一个 seed——「差 <3%」是同变量重复测量的必然结果，不是判别结果。
- **根因**：死参数——CLI/env 自变量被解析但未接到实际使用点（常量遮蔽），实验在机制上未改变自变量；「两结果相近」被误读为「自变量无关」，实际是「自变量从未变化」。属「对照实验对照组≠处理组」的设计失效，且失效静默（无告警、输出格式完全正常）。
- **定位**：交接纪律规定的「廉价独立验证」（基线复核）顺带做 bin 源码审查时发现 L18/L22 脱节；若按参数回显/探针恒等式自检（打印实际生效 seed 并比对）也能当轮发现。
- **修复**：supersedes 260903-08「seed 判别」结论（§15.4 双指针，见本文第三节）；判别实验前置自检——运行时输出实际生效的自变量值并断言处理组≠对照组。
- **教训/如何利用**：**通用判据：任何「变量 X 是否影响 Y」的判别实验，先验证「X 真被改变」再读结果**——手段 = 探针恒等式（打印生效值/中间产物断言两侧不同）或最小已知效应 sanity（给 X 一个已知强效应值跑一遍，若结果仍「无差异」则实验设计坏）。判别实验「差 <3%」这类近零结果尤其可疑：它同时兼容「真无影响」与「假判别」两种解释，须用恒等式自检排除后者再下结论。同族：workflow-patterns #14（stageMask 只控本侧、对侧静默须独立验证）、#18（跨 session 基准口径核对）——都是「对照/判别实验的前提假设未经验证」家族；本条补「自变量生效性」维度。

---

## 二、10 时间线条目草稿（追加到 `versions/1.20.1/docs/10-timewise-archive.md` 末尾）

> 结论状态 = **draft（judge 中）**；条目格式对齐末尾既有「## 260903-06/08」风格。

```markdown
## 260903-09（Q-PD1 包：Rust vs Java 2.2× 差距分阶段归因——大头在 aquifer，supersedes 260903-08 两个方向假设）

### ✅ 基线廉价独立复核（交接纪律先行）+ 附带发现死参数
Rust OFF 两跑 median 70.23/73.49ms（落 08 日 71-77 稳定带）✓；Java fresh（删 run\world 后）median≈32/total 10993（对 08 日 33/11067）✓ → 2.2× 有效。⚠️ 附带发现：`pc_e2e_bench.rs` L18 解析 WG_E2E_SEED 但 L22 恒用常量 SEED → 08 日 negseed「seed 判别」实际同 seed 跑两遍，「seed 非因素」证据无效。⚠️ 新坑：run\world 残留时 Java bench 走磁盘加载（total 764ms vs fresh 10993）——bench 前必须删 run\world（世界状态第四查）。✅

### ✅ 分阶段差分（WG_SKIP_* 门控，两轮稳定）
bin-diag/qpd1_stage_bench.rs（新，隔离区）：aquifer ~37ms/chunk（占 Rust FULL ~60%）；density/interp 底座 ~14.4ms（~23%）；surface ~5.5-6.7；carver ~5-6.5；orevein/features ≈0（噪声级）。段和−FULL=0.0%（构造恒等），真 sanity = 段值两轮 ±1ms 级一致。🔍 Java 侧 run A 无效测量已识别并由 run B fresh 复核替代。✅

### 🔍 结论（draft → judge 中，未 confirmed）
Q-PD1 归因：**差距大头 = aquifer 段**。supersedes pc1-e2e-260903-08 两个方向假设（§15.4 双指针，原文不改）：①「features/carver 段疑似大头」（实测 features≈0、carver ~8%）；②「seed 判别：negseed 差 <3% → seed 非因素」（死参数，未换 seed）。Amdahl 读数：GPU density 优化端到端天花板 62→~47ms（仍慢 Java ~1.4×）→ 优化主攻转向 aquifer 段机制（邻居随机偏移/split/采样次数，复用 WG_AQUIFERCOUNT/WL/BP 计数器），新课题待立项。

### 📌 记录指引
- 新坑两条（run\world 世界状态第四查 + 死参数假判别）→ workflow-patterns 发现 #19/#20（subagent 草稿已出，待应用）。
- 状态：draft（judge SHOULD→candidate 待审）；时间线随结论状态更新。
```

---

## 三、supersedes 标注文本草稿（§15.4 双指针，原结论不改）

**取代目标定位**（`E:\PYTHON\CoreSwap\.artifacts\lossless-accel\pc-results-260903-08.md`，confirmed artifact，正文一字不改）：
- **目标 1**：L15 结论 4 末句 ——「开问题 Q-PD1：features/carver 段疑似大头，独立排查。」
- **目标 2**：L15 结论 4 中段 + 10 时间线 260903-08 节 P-C1 行 ——「negseed 判别 <3% → seed 非因素。」

**建议标注方式**：在 pc-results-260903-08.md 文末（**追加，不改正文**）加 supersedes 指针小节；同时在 qpd1-attribution-260903-09.md 补回指指针（该文件「supersedes」行已有单向指针，补「取代自」双指针完整化）。

追加文本草稿（pc-results-260903-08.md 文末）：

```markdown
---
**§15.4 supersedes 指针（260903-09 追加，正文未改动）**：
- 本文件结论 4 中「开问题 Q-PD1：features/carver 段疑似大头」——已被 qpd1-attribution-260903-09.md 推翻：实测 features ≈0（噪声级）、carver 仅 ~8%，差距大头为 aquifer 段（~60%）。
- 本文件 P-C1 中「negseed 判别 <3% → seed 非因素」——证据无效（superseded）：pc_e2e_bench.rs WG_E2E_SEED 解析后未使用（L22 恒用常量 SEED），negseed 运行实际未换 seed，同 seed 重复测量必然差 <3%。
- 取代记录：.artifacts/lossless-accel/qpd1-attribution-260903-09.md（status: draft，judge 中）
```

回指文本草稿（qpd1-attribution-260903-09.md，supersedes 行后补一行）：

```markdown
- 取代自（双指针）：pc-results-260903-08.md 结论 4（Q-PD1 方向假设 + negseed seed 判别）——该文件文末已加 §15.4 指针。
```

---

## 四、subagent 自检清单

- [x] 价值门：坑 A/#19 高价值（判错经验+可复用签名）；坑 B/#20 高价值（实验设计判据）；结论摘要中价值简记（时间线）；无低价值内容混入。
- [x] 五段式完整（现象/根因/定位/修复/教训），根因为机制层（磁盘加载路径 / 死参数未改自变量），非现象复述。
- [x] 判错经验沉淀：#19 假象签名（快一个量级+min≈0）、#20 恒等式自检判据。
- [x] 编号核对：workflow-patterns.md 当前最大 #18 → 本两条 #19/#20（已 grep 全部 discovered 文件确认；build-tooling.md 至 #12，未占用）。
- [x] supersedes 双指针、原结论不改、目标行已实际定位（L15 两处）。
- [x] 时间线条目格式对齐 10 末尾（260903-06/08 节风格：## 日期标题 + ✅/🔍 小节 + 📌 记录指引），状态如实标 draft/judge 中。
- [x] 数字全部来自主会话提供的记录，无编造。
