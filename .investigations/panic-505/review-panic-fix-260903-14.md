# core.judge 审查意见 — surface_rules.rs:505 panic 修复（260903-14）

> 审查人：core.judge subagent（b382e3a2，三源核对）；结论 **PASS**（candidate 维持，should-fix 补正后 confirmed 可提交用户）。以下为意见全文，should-fix 清偿状态由主会话标注。

## 结论：**PASS（candidate 可维持，confirmed 建议待 should-fix 补正后授予）**

根因证据链闭合、修复正确、零回归主张总体成立。2 项 should-fix（证据记录完整性 + 一处数值复算错误）、2 项 concern，均不阻塞 candidate；四臂 hash 记录补齐前不建议 confirmed。

## 逐项意见

### 1. 根因证据链 — ✅ 闭合（PASS）
- 代码行号实证一致：surface_rules.rs:505 expect / :1372 get_noise("minecraft:badlands_pillar_roof")（Java L214 对拍注释）/ :1371。
- git diff：worldgen_handle.rs L272-275 恰一处改动，与 scout 候选 a 逐字一致。
- 延迟触发自洽独立复核通过（block 8→9 边界 panic + backtrace 帧吻合）；候选 b/c 排除论证一致。

### 2. 零回归声明 — 大体成立，1 项 should-fix
- ✅ sweep hits/misses repro vs fixed block0-8 九组逐项相同；fixed 续跑 4096/4096 完成。
- ✅ hash f2b1a3932c6e589e 与 260903-13 confirmed 记录三方一致。
- ⚠️ should-fix（已清偿 ✅）：四臂 hash 落盘只有单臂 → 补齐 estopt-ab-4arms-260903-14.txt（四臂完整输出）。
- ⚠️ concern（已清偿 ✅）：+707~+1202 块改善幅度 < #10 散布带宽 ~2330 块 → verdict 已补「仅作无回归佐证，不具单修复归因力」限定。

### 3. §9.7 可比性声明 — ✅ 完备（PASS）

### 4. Full 层声明诚实性 — ✅ 如实（PASS，附 concern 已清偿 ✅：sweep 行补「崩溃回归载体，非正确性载体」标注）

### 5. 遗留项 — ✅ 无遗漏（PASS）

### 其他
- concern（轻微，已清偿 ✅）：verdict「commit 见 git」与未提交现场不符 → 改「diff 见 git 工作区，随 verdict 一并提交」。

## 数值复算

| 项 | 复算 | 判定 |
|---|---|---|
| run1/2/3 | 98.9969 / 99.0284 / 99.0067% | ✅ |
| 均值 | 99.0107%（已补入 verdict） | ✅ |
| 区间 vs C4 基线 98.9520% | 不重叠、方向向上 | ✅ |
| 各 run 块差 | +707 / +1202 / +861 | ✅ |
| run 间散布 | **495 块**（verdict 原「315」为复算错误，已修正；结论不变） | should-fix 已清偿 ✅ |
| sweep 计数 | repro/fixed block0-8 逐项相同（9/9） | ✅ |
