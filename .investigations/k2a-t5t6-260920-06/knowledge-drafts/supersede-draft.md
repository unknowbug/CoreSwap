# supersede-draft — record-260920-06 C1/C2 补丁 + design-260920-05 §1.1 T5 §15.4 取代登记（subagent 草稿，主会话应用）

> 应用纪律：原结论正文不删不改（§15.4）——C1 为**插入**、C2 为**整段替换且替换后旧文本以引用块形式保留原位**（见 a-2 具体写法）、b 为**原行保留 + 追加取代指针**。

---

## a-1. record-260920-06.md —— C1 补丁（插入；judge 条件 C1 + info-1）

**插入位置**：`## 2.1 辅助读数` 小节末尾（第三条 bullet「**T6 环档分布**…不绑定主判据。」之后、`## 3. 判据逐条判定` 标题之前），插入以下小节：

```markdown
### 2.2 N 窗归窗缝隙近似补登（judge C1，落盘补登；不改判据不改判读）

- **缝隙存在**：probe N 打印与 remove(34)/add(33) 发生在**同一 serverTick 内、probe 之后数行**（FormProbe.java edgeTick case 2）——故「N 窗 = t < 60598.0」与「N 窗 = P 切票之前」之间存在一个**无 t 字段可分辨的同 tick 缝隙**（probe 行到切票行之间）。[WBQ] 行无 t 字段、按 log 位置归窗，若 light 行恰好落在缝隙内会被误归 P 窗，理论掩盖方向 = N FAIL（判据 N 方向）。
- **本 run 三重闭合旁证（缝隙风险未实际触碰）**：
  1. probe N `status=minecraft:initialize_light` 为边界时刻的运行时自证——light 未跑；
  2. 目标 chunk 唯一的 [FP-LIGHT] call 带 t=70860.6 ≫ 60598.0（切票后 10s 窗内）；
  3. WBQ enter 行（log #5537，无 t）之前夹有 P 相 ring 扩张的邻 chunk fill 行（#5489-5536，t=60617–60788，33 票 FULL 族扩环触发）——「enter 发生在切票后」有独立旁证。
- **行号计数基注记（judge info-1）**：本 record §1/§2 所引 log 行号存在系统性 ±1 偏差（疑 0 基/1 基计数差，如 probe N 记 #5487 / 实测 #5488；WBQ enter 记 #5536 / 实测 #5537；FP-LIGHT 记 #5541 / 实测 #5542；FP-FILL 记 5458/5461/5480 / 实测 5459/5462/5481）——相对次序不变、不影响归窗结论；归窗依据 = 行间相对位置，行号引用以本注记的计数基声明为准。
```

## a-2. record-260920-06.md —— C2 补丁（T5 结论表述改写为限定证据强度版本；judge 条件 C2）

**操作**：`## 4. T5/T6 结论表述` 小节第一条 bullet（原文起 `- **T5（修正后表述，draft→candidate 建议）**：` 至 `§15.4 取代链登记由 judge 后主会话落。`）**替换**为以下两条（原表述压缩保留为引用行，正文不丢）：

```markdown
- **T5（修正后表述，candidate；judge C2 限定证据强度后版本，取代本 bullet 原稿措辞）**：
  - **运行时直证面（Full 层，E1：同构建态 dll=cc4e39fe、单 run 双相、单 chunk (160,96) 单 seed 单维）**：**level 34 档 = INITIALIZE_LIGHT，停在 light() 之前；level 33 档 = FULL，light() 被调**——33/34 贴界最小差分对（N=34 停 light 前 + P=33 进 light，唯一变量 = 票档）运行时直接证实 33/34 为边界两侧。
  - **「level ≤ 33 其余档（≤32）」= 间接背书面（非运行时直证）**：静态链 A5（`level < 33 → FULL`，ChunkLevels.java:11-13 一手已核）+ A8（level 22 DRIVE_LEVEL 海量实测旁证）。不称全域直证；§9.7 不外推声明（§5）继续覆盖档位/seed/点组合外推。
  - 取代对象：design-260920-05 §1.1 T5 原表述「level 33 档目标 = INITIALIZE_LIGHT 停 light 前」——该表述为静态推演链 off-by-one（把 33 当第一个非 FULL 档；一手源核正见 design-260920-06 头号发现）。取代登记见 design-260920-05 §1.1 T5 行侧 §15.4 指针。
  - （原稿措辞存档：「chunk level ≤ 33 ⇒ 目标档含 LIGHT ⇒ light() 运行；level 34 档 = INITIALIZE_LIGHT，停在 light() 之前」——其中「≤33」全域直证措辞按 judge C2 收窄为上述两层。）
- **T6（辅助读数）**：本 run 未实现 ring=1 补充 probe（可选实现未做），环档 Chebyshev 线性加距预测维持 **I 标注**，独立降权、不随 T5 升档。
```

（注意：第二条 T6 bullet 原文保留不动；上面代码块把它一并列出仅为锚定替换边界——实际操作只替换第一条 bullet。）

## b. design-260920-05.md —— §1.1 T5 行的 §15.4 取代登记（原行保留不改 + 追加取代指针）

**操作**：§1.1 表中原行**保留不改**：

```
| T5 | **light() 运行的充要条件（推论，静态推演标 I）**：LIGHT 在 DISTANCE_TO_STATUS 中无独立档（T2），level 33 档目标 = INITIALIZE_LIGHT（**停在 light() 之前**，T3）；目标含 LIGHT 仅当 status 推进越过 INITIALIZE_LIGHT，即目标 = FULL = **level ≤ 32** | T1+T2+T3 联立，I-静态 |
```

**在该表行之后（紧接的下一行）追加取代指针**：

```markdown
> **§15.4 取代（k2a-t5t6-260920-06，260920-06）**：上行 T5 原表述已被取代——superseded-by `.investigations/k2a-t5t6-260920-06/record-260920-06.md` §4（judge C2 限定版）+ `judge-review-260920-06.md`（PASS-with-conditions C1-C3）。一行推翻理由 = **off-by-one**：`ChunkLevels.getStatus(33)` → `byDistanceFromFull(0)` = **FULL**（ChunkLevels.java:11-13 + ChunkStatus.java:303-308/203-216），LIGHT 在 FULL 前必须完成 ⇒ light() 被调；恰停 INITIALIZE_LIGHT 的档 = **34**（byDistanceFromFull(1)）。运行时双相直证 = fe2（N=34 停 light 前 status=initialize_light / P=33 进 light status=full，双 PASS）。上行原表述保留不改，仅本指针生效。
```
