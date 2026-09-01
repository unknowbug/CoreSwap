---
id: review-judge-20260908
topic: c2-p2-ore-attribution
title: Judge 审查——存档双跑修复（wg_set_flags 句柄级 flag）+ V3 soul rule 结构对拍（260902-02）
status: draft
审查性质: 只出审查意见，不改任何 status；confirmed 留待宿主人类
审查基线: 三源核对（① 产物快照 design-wg-set-flags-20260908.md + v3-structure-diff.md + index.yaml ② git status/diff ③ 回归日志 run4/5/6）
date: 260902-02
---

# Judge 审查意见（260902-02）：双跑修复 + V3 对拍

## 一、证据完整性（run4/5/6）

- **判据被日志实锤：PASS（带限定）**。run4/5/6 三轮独立日志均含 `stageMask=3`（init + initNether 两行）、seed=8576294172403134396、`match=990108/1048576 (94.4241%)`、mixin 拦截面（populateNoise intercepted / buildSurface skipped）在案。回归数字带 seed+region+口径三要素（nether 4x4@3200,3208，ReadWorldProbe 存档口径，FULL 参照 `.tmp/ref-full/`）。
- **CONCERN-C1（判据措辞）**：设计文档判据「≥3 采样区间一致」——实测三轮是**同一 seed 同一 region（198..205）重复三次**，验证的是确定性/可复现性，**不是 3 个独立采样区间**。若判据本意是区域覆盖面（不同 region 的存档口径一致），则该子项**未满足**，建议改判据措辞为「同 region 三次复跑一致」或补跑 2 个不同 region。
- **CONCERN-C2（94.4241% == SKIP_FEATURES 消融上界的解读）**：逐位一致的聚合匹配数（990108/1048576，连 nonAir 430408 都一致）是相当强的间接证据——若 features 仍在双跑，匹配数几乎必然偏离消融上界。但这是**聚合口径推断**，非 features 双跑已消除的直接观测。补一个低成本直接佐证即可闭合：对比修复后存档 vs 消融上界 run 的 **ore per-id 计数**（或直接对比两组非空点集合 diff=0）。在补齐前，支持「双跑已消除」为 **candidate 证据充分性：偏弱但可接受**；不构成 blocker。
- 历史对照链完整（修复前 ~93.8988% C2 → +5508 → 94.4241% 上界）。

## 二、代码改动正确性（git diff 静态核对）

- **Rust diff 与设计声明一致：PASS**。worldgen_handle.rs 加 `pub flags: AtomicU32`（create 置 0）+ 三个 FLAG 常量 + 三处判定改为 `flags & bit == 0 && env 未设`（skip 方向 OR，flags=0 行为与旧版完全一致）；api.rs 新增 `wg_set_flags/wg_get_flags`（null handle 防御）；jni_bridge.rs 新增两个 JNI 导出。**wg_create 5 参签名未动，FFI 契约无破坏：PASS**。
- **AtomicU32 并发：PASS（附低优先级注记）**。Relaxed store/load 对纯开关 flag 语义足够（无复合不变量依赖）；严格内存模型上 setFlags（主线程 JNI）与 worker 线程 load 无 happens-before，理论上有读到 stale 0 的窗口——实践中 setFlags 先于 region 准备且 worker 后创建，风险可忽略。建议注释里记一句（低优先级）。
- **CONCERN-C3（git 三源核对降级声明）**：`runtime/` 整目录在 .gitignore（`/runtime/`），**Java 侧 CppWorldgen.java/CppBridge.java 与 resources dll 二进制不在 git 追踪内**——「未提交改动清单含 Java/dll」的说法与 git 实况不符；本次审查对 Java/dll 侧只能做**当前文件态静态核对（Degraded）**，无法 diff HEAD。声明：本条审查为降级核对。Java 侧静态核对结果：`resolveStageMask()`（默认 0b011、`all`→0、int 直通）、init/initNether 后 setFlags+getFlags 日志回读、native `setFlags` 声明——与设计一致。低优先级：`Integer.parseInt` 解析失败静默回落 0b011（拼写错误无提示，建议 log warn）。
- **CONCERN-C4（默认 mask=0b011 影响面）**：overworld 句柄（init）同样默认 skip carver/features，设计文档 §风险 已声明、并给出 `-Dcoreswap.rust.stages=all` 对照通道——**声明本身合格，但该声明是「已知行为变更」而非「已验证无害」**：本轮回归只盯 nether 判据，overworld 存档行为变更未跑任何回归。风险已在清单 #3 量化课题挂账，可接受；但晋升 candidate 以上前，overworld 消融量化应完成（与 C2 补证同批）。

## 三、落盘契约

- **PASS（带一条建议）**：设计文档（.investigations）、V3 对拍表 frontmatter 齐全（id/status=draft/验证分层 Degraded/可比性声明/日期）、`.artifacts/.b2-soul/index.yaml` 已登记、回归日志 run1-6 落盘 cmd-output/、回归数字三要素齐。建议：回归判据的**判定记录本身**（94.4241% vs ≥94.42% 的通过判定 + C1 的判据措辞问题）最好有一份 .artifacts 结论条目登记，避免散在日志里（低优先级）。

## 四、V3 对拍表方法学

- **PASS**。Degraded 分层声明明确（纯静态、无运行时证据、与 V1/V2 动态口径不可比——可比性声明三要素齐）；「结构差不可解释」判定链自洽：结构完整 + Seq 末 soul_soil 兜底存在 → entered 且 selector<0 必然命中兜底、不可能穿透到 netherrack → 残差只能归运行时输入差（分支未进入 / selector 实例差）。这是正确的排除式论证，且明确不越界回答「运行时输入差」。
- **V4 建议是最小裁决探针：PASS**——生产链路 soul 分支入口一次性 ctx dump（或复用 soul_selector_probe 直连生产构造路径），直接裁决「分支未进入 vs selector 实例差」，无过度工程。
- 附注两点合理：mult 硬编码 0 判「非本次因果」的论证成立（nether 全 mult=0）；「解析失败静默回退 overworld 规则」改 fail-fast 的改进建议恰当。**低优先级遗留**：vertical_gradient 解析块重复两份（代码卫生）、default_block 占位 stone 的边界注释——对拍表已自标，无需行动。

## 五、PASS / CONCERN / FAIL 清单

| # | 级别 | 项 |
|---|---|---|
| P1 | PASS | 回归判据主数字被 run4/5/6 实锤，stageMask=3 + seed + 口径三要素齐 |
| P2 | PASS | Rust diff 与设计一致；wg_create 5 参未动；OR 语义/AtomicU32/null 防御正确 |
| P3 | PASS | V3 对拍分层声明/可比性声明/判定链自洽；V4 为最小裁决探针 |
| P4 | PASS | 落盘契约齐（frontmatter/index.yaml/日志/三要素） |
| C1 | CONCERN | 「≥3 采样区间」实为同 region 三次复跑，措辞与实测不符——改措辞或补 region |
| C2 | CONCERN | 「双跑已消除」为聚合口径推断；建议补 ore per-id 计数（或非空点集合 diff=0）作直接佐证 |
| C3 | CONCERN | Java/dll 不在 git 追踪（runtime/ gitignored），三源核对对该部分降级为静态核对——交接/台账应声明 |
| C4 | CONCERN | overworld 默认 mask=0b011 行为变更已声明但未回归验证；candidate 以上前需 overworld 消融量化 |
| L1 | LOW | AtomicU32 Relaxed 跨 JNI 线程窗口补注释 |
| L2 | LOW | resolveStageMask 解析失败静默回落建议 log warn |
| L3 | LOW | vertical_gradient 解析块重复（V3 已标）；fail-fast 改造建议在案 |
| — | FAIL | 无 |

## 六、推荐状态

- 双跑修复：**支持保持/授予 candidate**（证据充分性偏弱但可接受；C2 补证可与 overworld 量化同批低成本补齐）。
- V3 对拍：**保持 draft（Degraded）**——正确地未自授 candidate；V4 动态对照后另行评估。
- confirmed：留待宿主人类。
