# supersede-draft — verdict-260920-05 §9 C2 升级建议措辞的 §15.4 结论取代登记（subagent 草稿，主会话应用）

> 应用纪律（§15.4）：原结论正文**不删不改**——verdict-260920-05.md 原文不动，仅在 §9 C2 条目（:74 行）之后**追加引用块形式的取代注记**（文本见本稿 §3）。新结论落点 = 本登记 + record-260920-06 §4 T5（confirmed）。status 全部 **draft/candidate**，confirmed 留人类。
>
> 前置核对（本草稿写作时已逐项确认）：
> 1. T5 已获用户 **confirmed（2026-09-21）**——`.investigations/k2a-t5t6-260920-06/record-260920-06.md` 第 4 行（judge PASS-with-conditions C1-C3 已应用）。
> 2. 被取代措辞 = `.artifacts/k2a-fullcov-260920-05/verdict-260920-05.md` :74（§9 第二条 bullet，judge-05 N8 收窄后版本），其中升档前置「一次运行时单验，如对单 chunk 施加 level 33 档对照臂」已由 260920-06 **fe2 双相运行时单验**满足（N 臂 lvl=34 status=initialize_light 停 light() 之前 / P 臂 lvl=33 status=full light() 被调，33/34 贴界最小差分对，E1，双 PASS）。
> 3. 不与 design-260920-05 §1.1 T5 行侧既有取代指针冲突：该指针取代的是 **T5 表行**（off-by-one 静态表述），本登记取代的是 **verdict :74 的 C2 升级建议措辞**——对象不同、指向同一条 260920-06 证据链，互为补充。

---

## 1. 取代登记正文（§15.4 格式）

- **supersedes**: `.artifacts/k2a-fullcov-260920-05/verdict-260920-05.md` :74 —「C2 升级建议（judge N8 收窄后措辞）：light() 覆盖 = 驱动 workload/ticket 覆盖面的直接函数（D1 直接证明面）；level≤32 机制层出自 T5/T6 静态推演，保持 I 标注不随本证据升档（升档前置 = 一次运行时单验，如对单 chunk 施加 level 33 档对照臂）」——其中**「level≤32 机制层保持 I 标注」的分档部分被取代**（C2 主命题「覆盖面直接函数」不变，仅机制层措辞升档，见 §2）。
- **superseded-by**:
  - 新结论落点 = 本登记 §2 新 C2 措辞（`.investigations/c2-upgrade-260921-01/supersede-draft.md`，主会话应用后生效）；
  - `.investigations/k2a-t5t6-260920-06/record-260920-06.md` §4 T5（judge C2 限定证据强度后版本；**confirmed 用户授予 2026-09-21**）；
  - `.investigations/k2a-t5t6-260920-06/judge-review-260920-06.md`（PASS-with-conditions C1-C3，条件已应用；C2 = 措辞限定证据强度）。
- **一行升档理由**：升档前置「运行时单验（如对单 chunk 施加 level 33 档对照臂）」已满足——fe2 双相单验以 33/34 贴界最小差分对直证边界两侧（唯一变量 = 票档，E1，双 PASS），T5 获 confirmed（2026-09-21），原「保持 I 标注」的保留条件失效。
- **status**: 本登记与新 C2 措辞 = **confirmed（用户授予 2026-09-21；judge PASS 2026-09-21，附 B1/B2 非阻塞建议已应用）**。

## 2. 新 C2 措辞（升档后，按 judge-06 C2 条件限定证据强度）

> **C2（升档版，confirmed——用户授予 2026-09-21）**：light() 覆盖 = 驱动 workload/ticket 覆盖面的直接函数（机制 = **level≤33 档位语义**：ticket 目标档 ≤33 ⇒ 档位链含 LIGHT ⇒ light() 被调；**level 34 档 = INITIALIZE_LIGHT 停 light() 之前**）。
>
> 证据强度限定（judge-review-260920-06 C2 条件）：
> - **33/34 贴界对 = 运行时直证**（fe2 双相单验：N 臂 lvl=34 status=initialize_light / P 臂 lvl=33 status=full，同 run 双相单 chunk 自对照，唯一变量 = 票档，E1，双 PASS）。
> - **≤33 其余档（≤32）= 间接背书面，非全域直证**：静态链 A5（`level < 33 → FULL`，ChunkLevels.java:11-13 一手已核）+ A8（level 22 DRIVE_LEVEL 海量实测旁证）。
> - **T6（Chebyshev 环分布）维持 I 标注不变**（ring=1 probe 未实现，独立降权，不随本升档联动）。

该措辞取代 verdict-260920-05 :74 中「level≤32 机制层出自 T5/T6 静态推演、保持 I 标注」的分档（仅该分档；「覆盖面直接函数」主命题与 judge-05 N8 收窄措辞一致，未被推翻，只是机制层从 I 升为 confirmed 承载的档位语义）。

## 3. verdict-260920-05.md 附录取代指针文本（追加于 :74 条目之后，引用块形式；原文不删不改）

```markdown
> **§15.4 取代（k2a-t5t6-260920-06 + c2-upgrade-260921-01，2026-09-21）**：上行 C2 升级建议中「level≤32 机制层保持 I 标注（升档前置 = 一次运行时单验）」的分档已被取代——superseded-by `.investigations/c2-upgrade-260921-01/supersede-draft.md` §2（新 C2 措辞）+ `.investigations/k2a-t5t6-260920-06/record-260920-06.md` §4 T5（confirmed 用户授予 2026-09-21）+ `judge-review-260920-06.md`（PASS-with-conditions C1-C3）。一行升档理由 = **升档前置已满足**：fe2 双相运行时单验以 33/34 贴界最小差分对直证边界两侧（N=34 停 light 前 status=initialize_light / P=33 进 light status=full，唯一变量 = 票档，E1，双 PASS），T5 confirmed 2026-09-21。新措辞 = 「机制 = level≤33 档位语义：ticket 目标档 ≤33 ⇒ 档位链含 LIGHT ⇒ light() 被调；level 34 档 = INITIALIZE_LIGHT 停 light() 之前」；证据强度限定：33/34 运行时直证，≤32 其余档为静态链 A5+A8 间接背书，非全域直证；T6 环分布维持 I 不变。上行原文保留不改，仅本指针生效。
```

## 4. 置信度与 §9.7 口径声明

- **置信度状态**：本登记 = **confirmed（用户授予 2026-09-21）**；新 C2 措辞 = **confirmed**（judge PASS + 用户拍板，2026-09-21）。
- **证据分层**：33/34 直证面 = **Full**（运行时行为化日志直读）；≤32 其余档 = Degraded（静态一手源链）；C2 主命题「覆盖面直接函数」维持 verdict-260920-05 原证据面（log 机械复算 Full 数据层 + 机制解释）。
- **§9.7 口径（fe2 单验，照 record-260920-06 §5）**：载体 = server stdout 行为化日志（[WBQ]/[FP-LIGHT]/[FP-EDGE]）；覆盖面 = **单 chunk (160,96) 单 seed 8576294172403134396 单维 overworld 1.20.1**；等价档位 = **E1**（同构建态 dll=cc4e39fe、同 run 双相、唯一变量 = 票据档位 34→33）；共享观测 key 集 S = {(160,96)} 三面。不外推 1.21.6、其他 seed/点/档位组合；判据面依赖 lightRust 开（Mixin :1061 gate）。
- **判据前置（§15.1）**：本登记引用的全部读数依赖 record-260920-06 Q1-Q8（全绿）+ verdict-260920-05 Q1-Q7/QG1-QG2（全过）；任一后续复核失效 ⇒ 对应判据 suspended、本登记标 premise-expired（status 永不自动变更）。
