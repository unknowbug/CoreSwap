# archive-draft-260912-02 —— 归档落盘草稿（7 段；供主会话应用 + 逐字校验）

> 角色：知识落盘 subagent（Phase 4 归档草稿）。**本件是本次唯一产出**；未修改任何现有文档（`docs/`、`knowledge/`、`.artifacts/` 一律未动）。
> 状态：draft（本件不授予 candidate/confirmed；`confirmed` 只能由人类授予）。
> 已获用户 confirmed 的范围（**只有 4 项**，2026-09-12 18:13）：① F1 修复有效性 ② 根因 b2a ③ 1.20.1「未观测到 F1 相关回归」 ④ 1.21.6 默认臂未退化。**不含**：nether/end 覆盖、性能结论、`BULKWB_ON` 翻转、出货决策。
> 依据：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（先读；错误优先 / 五段式 / 载体映射 / 记录价值门）+ `knowledge/INDEX.md`。
> 数字纪律：本稿所有数字均取自本稿**亲自读过**的文件，就近标注 `file:行` 或证据文件名。**不能从落盘证据复现的汇总数字已就地标注并要求按实际枚举改写**（段 1「16 组 → 实为 18 组」、段 1「五份 → 实为 4 份」、段 1「49/50 属另一半径臂」）。

---

## §0 七段总览（落点 / 锚点 / 校验样本数）

| 段 | 目标文件（相对工作区） | 落点 | 锚点（供精确定位） | 锚点唯一性 | 校验样本数 |
|---|---|---|---|---|---|
| 1 | `versions\1.20.1\docs\07-block-pipeline.md` | **追加到文件末尾**（现末行 = `:1363`） | 无需锚点（EOF 追加） | — | 3 |
| 1b | 同上 | **插入到 D3 节内**（该节标题行之后、空行之前） | `## 2026-09-12 D3 共享 Java 适配核（Wave 2 语义统一，commit \`997d40f\`）— ✅ confirmed（用户授予 2026-09-12 15:52；`（D3 标题全文） | **唯一**（`grep '^## 2026-09-12 D3 共享 Java 适配核'` = 1 命中，`:1336`） | 3 |
| 2 | `versions\1.20.1\docs\08-version-migration.md` | **插入到「已知坑速查（索引）」小节内**（该小节最后一条之后） | `- 流水线：块级插值顺序、线程默认自适应、多线程一致性验证（07）` | **唯一**（`grep '流水线：块级插值顺序'` = 1 命中，`:75`） | 3 |
| 3 | `versions\1.20.1\docs\10-timewise-archive.md` | **追加到文件末尾**（现 3212 行，末行 `:3212` = `---`；新块紧跟其后） | 无需锚点（EOF 追加） | — | 3 |
| 4 | `knowledge\discovered\workflow-patterns.md` | **追加到文件末尾**（现末行 = `:2386`，末块 = `### 发现 #14 补充案例（260912-01）`） | 无需锚点（EOF 追加）；新编号 = **#139**（现最大 = #138，`:2352`） | — | 3 |
| 5 | `knowledge\discovered\algorithm-fingerprints.md` | **追加到文件末尾**（现末行 = `:557`，末条 = `发现 #25 简记`） | 无需锚点（EOF 追加）；新编号 = **#26**（现最大 = #25，`:551`） | — | 3 |
| 6 | `framework-proposals\RE-FRAMEWORK-merge-index-list-root-proposal.md` | **新建文件全文**（该路径当前**不存在**；`glob framework-proposals/*` 仅命中两份先例） | — | — | 3 |
| 7 | `knowledge\INDEX.md` | **追加到文件末尾**（现末行 = `:155`，末条 = `> 260912-01 追加（…`；本行 = 同日不同块 260912-02，需可区分） | 无需锚点（EOF 追加） | — | 5 |

> 已应用记录（主会话回报，2026-09-12）：段 1 / 1b → `docs/07`（新节 `## 2026-09-12 D3 后续（260912-02）…` + D3 节标题行下就地取代指针）；段 2 → `docs/08:76`；段 3 → `docs/10:3206`；段 4 → `workflow-patterns.md:2390`（`## 发现 #139（最高价值）…`）；段 5 → `algorithm-fingerprints.md:561`（`## 发现 #26: …`）；段 6 → 新建 67 行。提交 `7a0fcff`（HOOK-D 裁决应用：record→confirmed / C-17′）+ `3cf009f`（归档 6 段）；21+ 样本 0 未命中。

> 应用顺序建议：先段 1b（1 行）→ 段 1 / 2 / 3（docs）→ 段 4 / 5（knowledge）→ 段 6（新文件）→ 段 7（INDEX 登记行，**最后**，因它引用段 4/5 的编号与位置）；每段应用后立刻用该段 ④ 的样本核对命中。

---

# 段 1 —— `versions\1.20.1\docs\07-block-pipeline.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\versions\1.20.1\docs\07-block-pipeline.md`（现 1363 行）

### ② 落点
**追加到文件末尾**（`:1363` 之后另起一段）。**不改动**既有 D3 节正文——D3 节的取代指针是**单独的段 1b**。

### ③ 待插入正文（逐字可用）

````markdown

## 2026-09-12 D3 后续（260912-02）：1.21.6 强制 bulk 写回的 storage 段帧契约缺陷定位与 F1 修复 — ✅ confirmed（用户授予 2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化）

> 范围与排除（照抄状态机口径，不得外推）：本节 confirmed **只覆盖上列 4 项**；**不含** nether/end 覆盖、性能结论、`BULKWB_ON` 翻转决策（三者均未获 confirmed）。
> 证据载体：`.investigations/shared-java-core-260912-02/`（`verify-260912-02.md` v1→v6 = 验证记录，**confirmed（范围受限** = 本节 4 项结论；范围外条目按原证据等级）；`judge-260912-02.md` = judge 三轮 + 交付前确认，PASS-with-conditions；`errors-260912-02.md` = W1–W13 五段式 + 速查表（台账，candidate）；`scout-map.md` + `scout-interpretation-A1.md` = 静态字节码与运行级解读；`evidence/` = 原始件）+ 冻结基线 `.investigations/shared-java-core-260912-01/evidence/`。提交 = `2c3be2a`（F1 本体，2026-09-12 17:13:50+0800，`evidence/javap-seam-260912-02.txt:37`）；`8974063 docs(260912-02): close review conditions, register artifacts, supersede E5`（2026-09-12 17:56:53+0800；两个提交号均已由本稿用 `git cat-file -t` 属主工具自核，符合 workflow-patterns #137）。
> 承接：本节取代上节（`## 2026-09-12 D3 共享 Java 适配核…`）中「1.21.6 强制 bulk 缺陷 = 仍开放 / 根因未定位」的表述；原节正文**不改**，取代指针以追加式插在该节标题行之下（见本节末「就地取代指针」）。

### 现象（260912-01 遗留缺陷的实测面）

| 量 | 值（修复前） | 来源 |
|---|---|---|
| 1.21.6 强制 bulk 臂 `[WG-CONTENT]` | 130 | `shared-java-core-260912-01/evidence/arm-summary.txt:42` |
| 1.21.6 强制 bulk 臂 `[WG-CONTENT-WB]` | 0 —— **首要信号**（写回读回层一条都没有） | 同上 |
| `EntryMissingException` | 132 | 同上 |
| 异常类型 / 索引取值 | `DIAG write threw chunk(x,z): net.minecraft.world.chunk.EntryMissingException: Missing Palette entry for index 2.` 逐 chunk 抛；索引取值集合只有 {2, 8}（C1 判别臂实测 `index 2` ×41 + `index 8` ×9，无第三取值） | 同上 `:32-41`（抽录 10 条）；`scout-interpretation-A1.md:228` |

- 不是崩溃而是「整段移位」（关键性质）：异常只在越界位置抛出；位移本身静默（C1 臂 `PASS distinct=1` 之后 `FAIL branch distinct=3`，`i=0` 解出的正是 writer 位置 4 的值 = +4 个 4-bit 位置的逐位实证，`scout-interpretation-A1.md:255-265`）。
- 默认关不豁免：该路径在 1.21.6 上原本**不存在**（该版 pre 无 `BulkWb`、property 被忽略），共享核把它变成**可达**；强制臂一开即崩（`errors-260912-01.md` E5，`:110-132`）。

### 根因 b2a：storage 段帧契约随版本变更，共享核仍按 1.20.1 前缀帧写出

| 项 | 1.20.1 | 1.21.6 |
|---|---|---|
| `PalettedContainer.readPacket` storage 段读法 | `PacketByteBuf.readLongArray([J)` = VarInt 长度前缀 + 定长 | `PacketByteBuf.readFixedLengthLongArray([J)` = 定长、无前缀 |
| tiny 映射名（逐行核对） | `.gradle-home/caches/fabric-loom/1.20.1/…/mappings.tiny:17600` → `method_10789 = writeLongArray` | `…/1.21.6/…/mappings.tiny:19802` → `method_68087 = writeFixedLengthLongArray`（另 `:19781 method_68086` = `(ByteBuf, long[])` 重载同名） |
| 字节码判据（单变量差异） | 两版 readPacket 指令偏移 5/15/24/39/45 逐条相同，唯一差异 = 第 39 条 invokevirtual 的目标不同 | 同左（`scout-interpretation-A1.md:21-23`；`evidence/A1-javap-PalettedContainer-{1.20.1,1.21.6}.txt`） |

- 机制链：共享核写端 `BulkWb` 按 1.20.1 契约写「VarInt 长度 + longs」；1.21.6 读端只取 `values.length` 个 long ⇒ 那 2 字节前缀**不被消费**（ARRAY 支 `VarInt(256)` = `0x80 0x02`）⇒ 整段移位 2 字节 ⇒ 解码越界的 palette 索引只能是 {2, 8}（前缀 nibble `e15=8, e14=0, e13=0, e12=2`；`scout-interpretation-A1.md:261-271`）。
- 竞争候选 b2b（位宽协商语义差）已排除：读端 byte 消费路径、`getCompatibleData` 决策骨架、`createDataProvider` 实现三处逐条同形 + 运行级 {2, 8} 单变量分布（`scout-interpretation-A1.md:239-251`）。

### 修复 F1（commit `2c3be2a`）：分版缝收口写端帧

- 改法：分版缝（`versions/<ver>/java/src/main/java/wg/bench/WgCompat.java`，每版一个同名文件；共享源不得出现版本分支）新增 `writeStorageLongs(PacketByteBuf, long[])`；共享核 `BulkWb.buildContainer` 的 **2 个调用点**（原 `:247` `EMPTY_LONGS` 支 + `:262` 数据支）改为调用缝方法，两版各自调用本版正确的写出方法。
- jar 级证据（`evidence/javap-seam-260912-02.txt:4-34`）：两版缝体都是**纯转发** —— `invokevirtual #45` 调**同一个被调方法**（1.20.1 = `class_2540.method_10789`、1.21.6 = `class_2540.method_68087`）+ 同一实参（`aload_0` / `aload_1`），`pop` 丢弃返回值，**体内无任何写字节指令**；`BulkWb` 内该调用出现 2 次（`L386` / `L479`，同属 `buildContainer`）。
- 1.20.1 腿的性质：缝体是对原直接调用的静态转发，因此 1.20.1 写出帧逐字节同构（这是「1.20.1 未观测到 F1 相关回归」的机制依据，不是仅靠测试）。

### 验证（修复后实测）

| 判据 | 实测 | 来源 |
|---|---|---|
| 1.21.6 bulk 臂计数 | `[WG-CONTENT]` = `[WG-CONTENT-WB]` = 625（修复前 130 / 0 / 132） | `evidence/arm-summary-260912-02.txt:7` |
| 全目录 `EntryMissingException` | 0（8 臂全扫；仅修复前 C1 臂与常量池字节码文本命中） | `judge-260912-02.md:53`（A2） |
| WBTEST 合成自检 | 4/4 PASS（distinct = 1 / 3 / 20 / 300） | `evidence/arm-summary-260912-02.txt:29-33` |
| 按层多重集对比 | 全等（`only-pre=0` / `only-post=0`）——本稿逐组枚举：主对比 8 组（`cmp-260912-02-raw.txt`，8 个 `=====` 段）+ 自证臂对比 8 组（`cmp-260912-02-att-raw.txt:4-66`）+ 补跑 2 组（同文件 `:95-109`）= **18 组** | 三份 `cmp-*.txt` 原始输出 |
| 条目级对拍（jar manifest） | 相同 1079 / 1800、差异 3、增 0 删 0，`V1 verdict = PASS（无非预期差异）` | `evidence/manifest-diff-postF1-1.20.1.txt:6-9` 与 `:23`（1.21.6 同型） |
| 1.20.1 与冻结基线 | 整文件全 64 位 sha 同一 = `4b10f2be7f9d5d949dd0acd048b8c0b40ac0653bc09083b45d7968bf13ed9e21`（83804 B）；同 sha 家族 1.20.1 = 10 份 / 1.21.6 = 4 份 | `verify-260912-02.md:92`（C-13）+ `:175`（F28） |
| 1.21.6 默认臂未退化 | fix-default 与冻结默认臂逐层多重集全等（625 / 625） | `cmp-260912-02-raw.txt:29-53` |
| 臂变量生效自证（正 / 负成对） | `-Dcoreswap.bulkwblog=1`：bulk 臂 `[WG-BULKWB] calls=625`；perblock 臂 `[WG-BULKWB] calls=0` + `[WG-PERBLOCK] calls=607` | `cmp-260912-02-att-raw.txt:68-79` |

> ⚠️ 数字复核（本稿，两处汇总数字按实际枚举更正）：① 指令 / 计划口径的「16 组多重集对比」在落盘证据中**无法复现** —— 按 `cmp-*.txt` 逐组枚举为 **18 组**（8 + 8 + 2），本节按 18 记；② 「1.21.6 五份 fp 同一」为汇总层笔误，穷举定案为 **1.21.6 = 4 份 / 1.20.1 = 10 份**（`verify-260912-02.md:10` v4 修订、`judge-260912-02.md:451-453`）。

### 残留 / 边界（如实写，不得当已闭合）

| 项 | 状态 | 说明 |
|---|---|---|
| nether / end 覆盖 | 0（零覆盖） | 属 `BULKWB_ON` 翻转的**前置**（**非本结论前置**）；被改的 `writeChunk` 共享三维度 |
| 1.21.6 写出帧字节直采 | 未做 | 由读端 4096 点读回（WBTEST 4/4）+ 625 chunk 等价性代替 |
| 1.20.1「未观测到 F1 相关回归」口径 | 受限 | 仅限 **overworld / 写回内容层 / 单次 JVM 运行** 口径 |
| 性能结论 | 不做 | 本块不出性能主张（bulk 与逐块计时同 run 含诊断成本，不作收益读数） |
| `BULKWB_ON` 翻转 | 未做 | nether / end 覆盖为 0 即前置未达 |
| 出货 | 不出货 | F1 对 1.20.1 为静态转发、无紧急重发需求；若出货须升版 1.0.30 + 新工单 |
| `verify` / `errors` 状态 | verify = **confirmed（范围受限**，仅标题所列 4 项；范围外条目按原证据等级）；errors = candidate（台账） | confirmed 只覆盖 4 项范围，不外推 |

### 可复用判据

- 跨版本帧契约变更 ⇒ 写端必须走分版缝：凡「共享核 + 分版缝」结构，任何写 wire 格式的调用点都 MUST 收敛到单一分版缝函数，禁止散落的直接调用（本块修复即此形态）。
- 指纹对写回内容面敏感，但观察面有限：`[WG-CONTENT]`（Rust buf 层）对 Java 侧写回改动结构性不敏感；`[WG-CONTENT-WB]`（写回后读回层）能捕获写帧错误。两层指纹只可排除「写回内容发生变化」，不可单独支撑「无回归」——必须与单运行口径异常面同档、条目级 diff、构建绿并列（`verify-260912-02.md:176`，F29）。
- 等价性结论 MUST 与臂变量生效自证配对（→ `knowledge/discovered/workflow-patterns.md` 发现 #139）；跨版本帧契约指纹 → `knowledge/discovered/algorithm-fingerprints.md` 发现 #26。

### 就地取代指针（供主会话插入到上节 D3 节内，见段 1b）

> ⚠️ 260912-02 取代指针（2026-09-12）：本节的「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」已被取代 —— 见本文件末尾「D3 后续（260912-02）」节：根因 = storage 段帧契约变更、修复 F1 = commit 2c3be2a、运行级验证已过。原文保留不改。
````

### ④ 逐字校验样本（3 条，应用后 `grep` 应各命中 1 次）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `不是崩溃而是「整段移位」（关键性质）：异常只在越界位置抛出；位移本身静默` |
| 2 | `两版 readPacket 指令偏移 5/15/24/39/45 逐条相同，唯一差异 = 第 39 条 invokevirtual 的目标不同` |
| 3 | `缝体是对原直接调用的静态转发，因此 1.20.1 写出帧逐字节同构` |

---

# 段 1b —— 就地取代指针（同文件 `07-block-pipeline.md` 的 D3 节内）

### ① 目标文件
`E:\PYTHON\CoreSwap\versions\1.20.1\docs\07-block-pipeline.md`

### ② 落点
**插入到既有 D3 节标题行之后**（`:1336` 之后、`:1337` 空行之前），成为该节第二行（渲染为标题下的引用行）。原文**一字不改**。

### ③ 待插入正文（逐字可用；1 行）

```markdown
> ⚠️ 260912-02 取代指针（2026-09-12）：本节的「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」已被取代 —— 见本文件末尾「D3 后续（260912-02）」节：根因 = storage 段帧契约变更、修复 F1 = commit 2c3be2a、运行级验证已过。原文保留不改。
```

### 锚点（唯一性已核）
- 锚点原文片段（`:1336` 行首）：`## 2026-09-12 D3 共享 Java 适配核（Wave 2 语义统一，commit `997d40f`）— ✅ confirmed（用户授予 2026-09-12 15:52；`
- **唯一性**：`grep '^## 2026-09-12 D3 共享 Java 适配核'` → 1 命中（`:1336`）✓
- 插入位置说明：紧跟该整行（该行以 `…confirmed 留用户）` 结束）之后新增本行。

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `本节的「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」已被取代` |
| 2 | `根因 = storage 段帧契约变更、修复 F1 = commit 2c3be2a、运行级验证已过。原文保留不改。` |
| 3 | `见本文件末尾「D3 后续（260912-02）」节` |

---

# 段 2 —— `versions\1.20.1\docs\08-version-migration.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\versions\1.20.1\docs\08-version-migration.md`（现 83 行）

### ② 落点
**插入到「已知坑速查（索引）」小节内**，作为该小节新的一条（放在最后一条之后、`## 数据/工具链` 之前）。该小节现为 `:68-75`。

### ③ 待插入正文（逐字可用；一条）

````markdown
- 分块容器「写帧」契约会随版本变（跨版本坑，260912-02）：1.20.1 的 `writeLongArray` 是 VarInt 长度前缀 + 定长 longs，1.21.6 的 `writeFixedLengthLongArray` 是定长无前缀 —— `PalettedContainer.readPacket` 的 storage 段读法同步换成了定长版。症状：帧不匹配时不是崩溃而是整段移位解出越界 palette 索引（本块实测越界值只出现 {2, 8}），且 `[WG-CONTENT-WB]` = 0（写回读回层全空）是首要信号。检查动作：升级版本时 MUST 对「写 storage 的每一处」核对读端 `readPacket` 用的是哪种读法（`javap` 比第 39 条 `invokevirtual` 的目标 + `.gradle-home/.../mappings.tiny` 的 yarn 方法名，两者互证），并把写端收敛到单一分版缝函数（禁用散落的直接 writeLongArray 调用）。证据：`.investigations/shared-java-core-260912-02/`（`verify-260912-02.md` / `judge-260912-02.md` / `errors-260912-02.md` / `evidence/javap-seam-260912-02.txt`）。
````

### 锚点（唯一性已核）
- 锚点原文（`:75`，小节末条）：`- 流水线：块级插值顺序、线程默认自适应、多线程一致性验证（07）`
- **唯一性**：`grep '流水线：块级插值顺序'` → 1 命中（`:75`）✓
- 插入位置说明：紧跟该行之后新增本行（缩进与列表符号 `- ` 与相邻条目一致）。

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `分块容器「写帧」契约会随版本变（跨版本坑，260912-02）` |
| 2 | `帧不匹配时不是崩溃而是整段移位解出越界 palette 索引` |
| 3 | `并把写端收敛到单一分版缝函数（禁用散落的直接 writeLongArray 调用）` |

---

# 段 3 —— `versions\1.20.1\docs\10-timewise-archive.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\versions\1.20.1\docs\10-timewise-archive.md`（现 3203 行，末行 = `---`）

### ② 落点
**追加到文件末尾**（`:3203` 的 `---` 之后另起新块节）。块节风格照既有块（标题 + `> 过程产物` + `- ✅ / 🔍 / 📌` 条目 + 状态标注）。

### ③ 待插入正文（逐字可用）

````markdown
## 260912-02（实际 2026-09-12 17:0x–18:1x，日期锚 Get-Date 18:13：1.21.6 强制 bulk storage 段帧契约缺陷定位 + F1 分版缝修复）✅ 用户已 confirmed（2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化；**不含** nether/end 覆盖、性能结论、`BULKWB_ON` 翻转）

> 过程产物 `.investigations/shared-java-core-260912-02/`（`scout-map.md` / `scout-interpretation-A1.md` = 静态字节码 + C1 运行级解读 / `verify-260912-02.md` = v1→v6，**confirmed（范围受限** = 上列 4 项结论）；§0 载体声明 / §4.1 F1–F29 / §5 未闭项 / §8 硬门进度 / `judge-260912-02.md` = 三轮 + 交付前确认，均 PASS-with-conditions；§12 九项硬门 / §16.1 终态确认 / `errors-260912-02.md` = W1–W13 + 速查表 / `evidence/`（含 `javap-seam-260912-02.txt`、`arm-commands-260912-02.txt`、`cmp-260912-02{,-att}-raw.txt`、`manifest-*`、`TL-javapv-*`））+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-02-1.21.6强制bulk缺陷定位.md`；提交 `2c3be2a`（F1 本体，2026-09-12 17:13:50+0800）/ `8974063 docs(260912-02): close review conditions, register artifacts, supersede E5`（2026-09-12 17:56:53+0800）。通用模式 → workflow-patterns #139（最高价值）、algorithm-fingerprints #26。

- 目标：定位 260912-01 遗留的「1.21.6 强制 `-Dcoreswap.bulkwb=1` 每 chunk 抛 `EntryMissingException`（130 / 0 / 132）」根因并修复，且不得触碰 1.20.1 出货线。
- ✅ 做了（决定性判别先行，HOOK-B）：先跑判别臂确认单候选 b2a，而非直接 fan-out ⇒ 判定树坍缩（b2b 被三处逐条同形 + {2, 8} 单变量分布排除）⇒ fan-out 未触发（纪律说明：这是预登记判定树 + 判别臂的坍缩，非主会话自推取舍）。静态面补 A1 / A1b / A3 字节码（两版 `readPacket` 偏移 5/15/24/39/45 同形、唯第 39 条 `invokevirtual` 目标不同）+ 运行面 C1 合成自检（`PASS distinct=1` 之后 `FAIL branch distinct=3 i=0 got=granite want=stone`）。
- ✅ 判据 / 证据：根因 = storage 段帧契约变更（1.20.1 `readLongArray` 前缀帧 vs 1.21.6 `readFixedLengthLongArray` 定长帧；tiny 映射名 `mappings.tiny:17600 method_10789 = writeLongArray` / `:19802 method_68087 = writeFixedLengthLongArray`）；修复 F1 = 分版缝 `WgCompat.writeStorageLongs` + 共享核 2 调用点改调（`evidence/javap-seam-260912-02.txt` 证明缝体纯转发 ⇒ 1.20.1 写出帧同构）；修复后 1.21.6 bulk 臂 625 / 625、`EntryMissingException` = 0、WBTEST 4/4、多重集对比全等（本稿按 `cmp-*.txt` 枚举 = 18 组：8 主 + 8 自证 + 2 补）、条目级 3 变更 / 0 增 0 删、1.20.1 与冻结基线整文件全 64 位 sha 同一（同 sha 家族 1.20.1 = 10 份 / 1.21.6 = 4 份）。
- ✅ 方法学硬洞补救（本块自纠）：两臂指纹全同既可能是「真等价」，也可能是「两臂都走了同一路径」（臂变量没生效）⇒ 补 3 个 `-Dcoreswap.bulkwblog=1` 正 / 负生效自证臂（bulk 臂 `[WG-BULKWB] calls=625`；perblock 臂 `calls=0` + `[WG-PERBLOCK] calls=607`）⇒ 等价结论不再是假阳性风险。
- ✅ judge（三轮：初审 / 续审 / 交付前确认）：均 PASS-with-conditions；9 项硬门（MUST-1..5 + 续审 9.1/9.2/9.3 + MUST-6）经 judge 回一手文件独立复核后全部闭合（`judge-260912-02.md:468` 终态确认；`verify-260912-02.md:350` 的记录侧「唯一未过硬门 = MUST-6」已由 v6 划改并标 ✅ 已闭合）。judge 独立抓到的最高价值问题 = C-17 的「12 vs 4」实为计数载体缺陷（无冒号 runServer 同时命中 :runServer 与 :content-test:runServer，两次 JVM 运行写同一日志、计数对整份日志做）⇒ 逐 run 切分后两侧同为 4 WMI / 0 非 WMI；同机制解释了 260912-01 的 `WMI 4↔8` / `testcontent 2↔5`，且上一块 judge 复审已记录该机制而本块未继承（双重教训）。
- 🔍 数字更正过程（可复用教训）：① 「1.21.6 五份 fp 同一」→ 穷举定案为同 sha 家族 1.21.6 = 4 份 / 1.20.1 = 10 份（`judge-260912-02.md:451-453`）；② 计划原「修复前 49 / 50」是 r16 探针臂数字（不同臂，不得同格引用），r160 真值 = 130 / 0 / 132（`judge-260912-02.md:310-312`）；③ 本稿复核发现「16 组多重集对比」按 `cmp-*.txt` 枚举实为 18 组。
- ✅ 裁决：C-17 判据重述获批（载体 = 单次 JVM 运行；原字面 FAIL 留档不改）；`BULKWB_ON` 不翻转（nether / end 覆盖 = 0 为前置）；不出货（F1 对 1.20.1 为静态转发，无紧急重发需求；若出货须升版 1.0.30 + 新工单）。
- 🔍 残留 / 边界（如实）：nether / end 覆盖 = 0；1.21.6 写出帧字节未直采（由读端 4096 点 + 625 chunk 等价性代替）；1.20.1「未观测到 F1 相关回归」仅限 overworld / 写回内容层 / 单次 JVM 运行口径；性能结论不做；`verify` = confirmed（范围受限，仅上列 4 项）；`errors` = candidate。
- 📌 open（下一轮最小闭环）：`errors-260912-01.md` E5 的**就地** `superseded_by` 注记已补（`errors-260912-01.md:134` + 速查表 `:154`，judge §16 复核通过）；`ref_merge_index` 裸列表根缺陷经 `framework-proposals/` 上报（见段 6 提案，W13 + judge R4）；登记侧计数滞后（`index.yaml:1511` 与片段 `:69` 仍写「12 条（W1–W12）」，实为 13 条，judge §16.2 R5，1 分钟可闭合）。
````

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `1.21.6 强制 bulk storage 段帧契约缺陷定位 + F1 分版缝修复` |
| 2 | `无冒号 runServer 同时命中 :runServer 与 :content-test:runServer` |
| 3 | `先跑判别臂确认单候选 b2a，而非直接 fan-out` |

---

# 段 4 —— `knowledge\discovered\workflow-patterns.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\knowledge\discovered\workflow-patterns.md`（现 2386 行；末块 = `### 发现 #14 补充案例（260912-01）`）

### ② 落点
**追加到文件末尾**（`:2386` 之后）。编号 = **#139**（现最大 = #138，`:2352`）。格式照既有「发现 #N」体例（发现时间 / 发现者 / 置信度 / module → 来源定位 → 观察 → 根因 → 判据 → 家族索引）。

### ③ 待插入正文（逐字可用）

````markdown

---

## 发现 #139（最高价值）: 等价性结论 MUST 与「臂变量生效自证」成对出现，且汇总数字 MUST 回底本复算（260912-02）

- 发现时间 / 发现者 / 置信度 / module：260912-02；主会话（补 3 个生效自证臂 + 穷举复算）+ judge subagent（三轮 + 交付前确认，独立抓出计数载体缺陷）；confirmed（用户授予 2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化；不含 nether/end 覆盖、性能结论、`BULKWB_ON` 翻转）；workflow-patterns / 等价性门（#81 + #138 家族的「自证」面）。
- 来源定位：`verify-260912-02.md` §1.C-11 / §1.C-17 / §6.1-§6.4 / §8.2；`judge-260912-02.md:53`（A2）/ `:299-301`（C-17 处置）/ `:307-308`（MUST-3）/ `:310-312`（MUST-4）/ `:451-453`（§15 穷举定案）/ `:468`（9 项硬门终态）；`errors-260912-02.md` W6（等价性 × 生效自证）/ W7（计数载体）/ W8（汇总数字未复算），速查表 `:379-381`；证据 `evidence/cmp-260912-02-att-raw.txt:68-79`（正 / 负自证原文）、`.investigations/shared-java-core-260912-01/evidence/arm-summary.txt:42`（修复前 130 / 0 / 132）、`evidence/cmd-260912-02-judge-requested.txt` §0（逐 JVM 运行切分）/ §5c（gradle 任务图直证）、`evidence/cmp-260912-02-raw.txt`（8 组主对比）、`evidence/cmp-260912-02-att-raw.txt`（8 组自证 + 2 组补跑）。
- 观察（现象）：

  1. 等价性 × 生效自证：修复后 1.21.6 bulk 臂与 default 臂两层指纹全同（625 / 625，`cmp-260912-02-raw.txt:3-27`）——但两臂指纹全同既可能是「真等价」，也可能是「两臂都走了同一路径」（臂变量没生效）。本块补 3 个 `-Dcoreswap.bulkwblog=1` 生效自证臂后才闭合：bulk 臂 `[WG-BULKWB] calls=625`；perblock 臂 `[WG-BULKWB] calls=0` + `[WG-PERBLOCK] calls=607`（`cmp-260912-02-att-raw.txt:68-79`）。
  2. 计数载体缺陷（假告警）：1.20.1 各臂异常行 12 vs 冻结基线 4 一度被读成「环境噪声 / 复跑波动」，实为统计载体错位 —— 运行台以**无冒号** `gradle runServer` 调用，同时命中 `:runServer` 与子工程 `:content-test:runServer`，两次 JVM 运行写同一份日志、计数对整份日志做（异常行 = 4/4 + 4/0）；按 run 切分后两侧同为 4 WMI / 0 非 WMI（`judge-260912-02.md:299-301`）。
  3. 汇总数字未回底本复算（复发族）：本块两处 —— 「1.21.6 五份 fp 同一」（穷举实为 4 份；1.20.1 = 10 份）与「修复前 49 / 50」（属 r16 探针臂，不得与 r160 行同格引用；r160 真值 = 130 / 0 / 132）。同一判据上一块已立（`errors-260912-01.md` E4），本块第二次复发。
- 根因（机制）：① 指纹全同是「对称」结论 —— 它对「变量失效」与「真等价」给出同一读数（门对两侧同路径结构性不敏感）；② 计数若不绑定「单次 JVM 运行」这一载体，多次运行的聚合就与单次运行的指标混用，差值会伪装成噪声；③ 汇总数字是多手转录的产物，与底本（台账 / 日志）之间没有任何机制保证一致，且跨臂数字常被同格引用（半径 16 探针臂的数字被填进半径 160 的表）。
- 判据（可复用）：

  1. 等价性 × 生效自证成对：两臂指纹全同 ⇒ MUST 附逐臂 in-log 生效证据（正 / 负成对），如 `-Dcoreswap.bulkwblog=1` 打印 `[WG-BULKWB] state_ids_size=…` / `calls=…` 与 `[WG-PERBLOCK] calls=…`；缺它则等价结论只能标「假阳性风险」。
  2. 统计前 MUST 声明并切分「计数载体」（与 §9.7 可比性三要素同源）；判别信号 = 同一判据的旧诊断 / 旧机制必须继承（本块上一轮 judge 复审已记录 `runServer` 双命中机制，本块未继承 ⇒ 双重教训）；低成本直证 = `gradle --dry-run <task>` 的任务图（带冒号只列 `:runServer`，无冒号列出 `:content-test:runServer`）。
  3. 任何进入记录的数字 MUST 当场回底本复算一次，并附 `file:line` + 臂标签；跨臂数字不得同格引用（份数 / 计数的口径必须分开计）。
- 家族索引：#81（臂变量生效自证）、#138（等价性门分档）、#130（载体存在性与空载体自检）、#127（口径换代后旧基线复算）、#132（跨臂门的分母语义）、`errors-260912-01.md` E4（门数字复算）/ E6（判据越界：多重集 ≠ 序列）。
````

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `等价性结论 MUST 与「臂变量生效自证」成对出现` |
| 2 | `也可能是「两臂都走了同一路径」（臂变量没生效）` |
| 3 | `按 run 切分后两侧同为 4 WMI / 0 非 WMI` |

---

# 段 5 —— `knowledge\discovered\algorithm-fingerprints.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\knowledge\discovered\algorithm-fingerprints.md`（现 557 行；末条 = `发现 #25 简记`）

### ② 落点
**追加到文件末尾**（`:557` 之后）。编号 = **#26**（现最大 = #25，`:551`）。格式照既有「发现 #N」体例（时间 / 置信度 / module → 来源定位 → 指纹 → 判据 → 家族索引）。

### ③ 待插入正文（逐字可用）

````markdown

---

## 发现 #26: 跨版本「分块容器 storage 写帧」契约差 —— VarInt 长度前缀 vs 定长无前缀（260912-02，最高价值）

- 发现时间 / 发现者 / 置信度 / module：260912-02；主会话（静态字节码 + 运行级判别 + F1 修复）+ scout subagent（`scout-interpretation-A1.md` 静态解读）+ judge subagent（三轮 PASS-with-conditions，9 项硬门闭合）；confirmed（用户授予 2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化）；algorithm-fingerprints / MC 序列化编码契约（**#22 的写端邻接面**：本条 = 写帧契约的版本差）。
- 来源定位：1.20.1 `PalettedContainer.readPacket` 的 storage 段走 `PacketByteBuf.readLongArray([J)`（VarInt 长度前缀，指令 #39；`evidence/A1-javap-PalettedContainer-1.20.1.txt:394`）；1.21.6 走 `readFixedLengthLongArray([J)`（定长无前缀，指令 #39；`…-1.21.6.txt:423`）；两版指令偏移 5/15/24/39/45 逐条相同（`scout-interpretation-A1.md:21-23`）。yarn 映射名逐行核对：`.gradle-home/caches/fabric-loom/1.20.1/…/mappings.tiny:17600 method_10789 = writeLongArray`、`…/1.21.6/…/mappings.tiny:19802 method_68087 = writeFixedLengthLongArray`（另 `:19781 method_68086` 为 `(ByteBuf, long[])` 重载同名）。失配症状实测：`.investigations/shared-java-core-260912-01/evidence/arm-summary.txt:42`（`[WG-CONTENT]=130` / `[WG-CONTENT-WB]=0` / `EntryMissing=132`）+ `scout-interpretation-A1.md:228`（`index 2` ×41 + `index 8` ×9）。
- 指纹（契约差）：

  | 版本 | readPacket 的 storage 段读法 | 写端对应方法 | 帧形态 |
  |---|---|---|---|
  | 1.20.1 | `PacketByteBuf.readLongArray([J)` | `writeLongArray`（`method_10789`） | VarInt 长度前缀 + 定长 longs |
  | 1.21.6 | `PacketByteBuf.readFixedLengthLongArray([J)` | `writeFixedLengthLongArray`（`method_68087`） | 仅定长 longs（无前缀） |

  - 1.21.6 仍保留前缀版 `readLongArray`（只是 `readPacket` 不再用它）⇒ 该变更是「读到点的换用」，不是「能力移除」。
- 失配症状（识别签名）：① 抛出类型 = `EntryMissingException: Missing Palette entry for index <小整数>`，不是帧格式错 / 长度错；② 读回层指纹 `[WG-CONTENT-WB]` 归零为首要信号；③ 整段移位 —— reader 位置 k 解出 writer 位置 k+4 的值（4-bit 打包；`PASS distinct=1` 之后 `FAIL branch distinct=3`，`i=0 got=granite want=stone`），越界取值集合由前缀 nibble 决定（`VarInt(256)` = `0x80 0x02` ⇒ 只能是 {2, 8}），且 {2, 8} 是越界值集合、不是受影响位置集合（`scout-interpretation-A1.md:255-273`）；④ 两版 `readPacket` 指令布局相同、仅 1 条 `invokevirtual` 目标不同。
- 判据（通用动作）：① 写端 MUST 收敛到单一分版缝函数（本项目形态 = 分版件 `WgCompat.writeStorageLongs` + 共享核调用点改调；`evidence/javap-seam-260912-02.txt:4-34` 证明缝体纯转发、`pop` 丢返回值、无写字节指令 ⇒ 未变更版本写出帧同构）；② 升级版本时对每个写 storage 的点核对读端读法（`javap` 比第 39 条 `invokevirtual` 目标 + tiny 映射名，两者互证）；③ 越界索引只有 {2, 8} 这类离散小集合时，优先怀疑整段字节移位，而不是位宽协商语义差（后者预测索引散布）。
- 家族索引：algorithm-fingerprints #22（同代码点的读端逐字节契约）、#24（「规范编码字节」= storage elementBits）、#23（同 section 派生计数两套语义）、workflow-patterns #139（等价性 × 生效自证）、#14 补充案例（门必须与被测变更同层）。
````

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `跨版本「分块容器 storage 写帧」契约差` |
| 2 | 写端 MUST 收敛到单一分版缝函数（本项目形态 = 分版件 |
| 3 | 抛出类型 = `EntryMissingException: Missing Palette entry for index <小整数>` |

---

# 段 6 —— `framework-proposals\RE-FRAMEWORK-merge-index-list-root-proposal.md`（**新文件全文**）

### ① 目标文件
`E:\PYTHON\CoreSwap\framework-proposals\RE-FRAMEWORK-merge-index-list-root-proposal.md` —— **当前不存在**（`glob framework-proposals/*` 只有两份先例：`ANCHORLAW-supersession-and-comparability-proposal.md`、`RE-FRAMEWORK-knowledge-compaction-proposal.md`）。

### ② 落点
**新建文件**（整份写入；不追加、不改任何现有文件）。头部 schema 与分节风格照两份先例（`# 提案：…` + 提案对象 / 提案人 / 状态 / 实证来源 + 编号小节）。

### ③ 待插入正文（逐字可用；文件全文）

`````markdown
# 提案：merge_index 片段顶层契约容错（裸列表根 + 逐文件隔离 + 注释保全）

- 提案对象：RE-Framework 工具链 `scripts/merge_index.py`（= `ref_merge_index` 工具的实现；`core-artifact` §5.1 配套）
- 提案人：CoreSwap 260912-02 session（2026-09-12）
- 状态：draft（待维护 agent 评估）
- 实证来源：CoreSwap 错误台账 `.investigations/shared-java-core-260912-02/errors-260912-02.md` **W13** + judge 交付前确认残留 **R4**（`.investigations/shared-java-core-260912-02/judge-260912-02.md:404`）；本块实测（`--dry-run` 复现 + 源码三点定位 + 逐文件读 legacy 片段 + 工具 sha 比对）

## 1. 问题陈述

`ref_merge_index` 在 CoreSwap 仓库**当前完全不可用**：任何一次合并（含 `--dry-run`）都在 `norm_entries` 处抛异常并整体 fail-fast，退出码 1、**不写任何文件**。后果是 `core-artifact` 的产物登记（`.artifacts/` 条目）在工具路径上被完全阻断，项目只能人工绕过。

三层缺口叠加（任一层单独存在都不致命）：

1. **片段顶层契约无类型分支**：`norm_entries()` 假定片段顶层是 mapping（`schema_version` / `project` / `module` / `entries:`），对解析结果直接调 `.get('entries', [])`。
2. **历史遗留片段形态不合契约**：仓库内存在 **5 个「裸列表根」片段**（顶层是 YAML 序列，没有 `entries:` 包裹）。
3. **采集无 per-file 容错**：`collect_fragments()` 递归 glob 全部 `index-entry.yaml`，任一文件不合规即让整个项目的合并中止（fail-fast 无隔离）。

此外有**同源副作用**：写回路径是「parse → 白名单化 → 重建整个文档 → dump」，只保留 `{id, path, kind, status}` 四字段 ⇒ **注释必然丢失**（YAML 加载器不保留注释是语言层面事实；`ENTRY_KEYS` / `norm_entries` / 写回 dict 三处叠加），而本项目 `.artifacts/index.yaml` 的惯例是**注释承载结论摘要与证据指针**。

## 2. 证据（可复现）

### 2.1 现象（一手运行记录）

```
AttributeError: 'list' object has no attribute 'get'
  栈：main():95 → norm_entries():52 → data.get('entries', [])
  退出码 1，未写任何文件
```

来源：`errors-260912-02.md:326-327`（`--dry-run` 即崩溃）。本项目 `scripts/merge_index.py` 副本与 RE-Framework 上游版 **sha256 前 16 位同为 `299598f46ebe1269`** ⇒ 非项目侧改动所致。

### 2.2 源码三点定位（本项目副本行号）

| 点 | 位置 | 内容 |
|---|---|---|
| ① 无类型分支 | `scripts/merge_index.py:52` | `for e in data.get('entries', []) or []:` —— `data` 为 `list` 时直接 AttributeError |
| ② 递归采集无容错 | `:59-60`（调用点 `:95`） | `glob('.artifacts/**/index-entry.yaml', recursive=True)`，无 try / except、无跳过 |
| ③ 写回丢注释 | `:37` + `:55` + `:129-139` | `ENTRY_KEYS = ('id','path','kind','status')` → `{k: e.get(k,'') …}` 白名单化 → 重建 `{schema_version, project, module, entries}` 后 `yaml.safe_dump` **覆写**根文件 |

### 2.3 5 个 legacy 裸列表根片段（逐文件实读）

`.artifacts/8576-24blocks/{aquifer-wateredge,biome-fix,biome-terracotta,followup,surface-plus1}/index-entry.yaml` —— 均为「注释行 + 直接进入序列」（如 `biome-fix/index-entry.yaml:4` 起即 `- id: …`，**无 `entries:` 映射键**）；行数 12 / 43 / 12 / 13 / 12。⇒ 解析结果为 `list`，命中缺口 ①。

### 2.4 影响面量化（注释即权威载体）

本项目 `.artifacts/index.yaml` 中**以 `#` 开头的注释行占相当比例**（260912-02 会话实测：全文 **1531 行、其中 `#` 注释行 504 行**，约 33%），且根 `index.yaml:1190` 已就地写下「勿跑该工具」警告 ⇒ 一旦工具被采用并写回，带注释的权威条目会被降级为 4 字段裸条目、叙事静默丢失。

### 2.5 本块的正当绕行（非项目侧缺陷）

① 主会话**手工**把 260912-02 的 4 条（带注释）追加进根 `.artifacts/index.yaml:1462-1524`；② 本地幂等校验脚本 `.tmp/shared-java-core-260912-02/check_index.py` 比对「片段 ↔ 根 index」的 `id/path/kind/status` 逐字段一致 + 幂等（实测 `IDEMPOTENT-CONSISTENT`）；③ 片段改置**标准采集路径** `.artifacts/shared-java-core-260912-02/index-entry.yaml`（工具修好后可直接采集，届时因 id / path / status 相同判「已存在，跳过（幂等）」）。

## 3. 建议（最小修复，按优先级）

1. **`norm_entries` 加 list 根容错**（一行）：`if isinstance(data, list): entries = data` 再取 entries；对既非 mapping 也非 list 的顶层给明确报错（不得静默当空）。
2. **采集按文件 try / except，跳过并报告**：单文件不合规 → 记入跳过清单并在输出打印「本次跳过 N 个片段（路径 + 原因）」，退出码仍可为 0（不静默、不 fail-fast）。
3. **写回保留注释 / 改「只追加缺失条目」策略**：可选 `--preserve-comments`；或把写回从「重建整档」改为「只追加缺失条目 + 原地保留原文」（本项目注释即权威载体的前提）。
4. **文档化片段顶层契约 + 给出合规样例**（`entries:` 包裹示例）并写入 `core-artifact` §5.1；输出行同时打印「片段总数 / 新增 / 幂等跳过 / 冲突 / 跳过文件」。

## 4. 影响面

- 工具在 `re-framework` preset 内嵌、**跨项目复用** ⇒ 缺陷不是 CoreSwap 局部问题：任何含历史遗留片段的项目都会遇到「工具完全不可用」。
- 报错指向 **YAML 内部类型**（`'list' object has no attribute 'get'`），不含片段路径与修复指引 ⇒ 使用者倾向怀疑「自己的片段写错了」，自查成本高（本项目已两次独立遇到同一坑：260910-04 与 260912-02）。
- 写回丢注释是**静默数据降级**（无告警），比崩溃更隐蔽。

## 5. 验收判据（建议）

① 对含 5 个裸列表根片段的仓库跑 `--dry-run`：**不再抛异常**，输出「跳过 5 个片段 + 路径清单」，退出码 0；② 修复后对合规片段合并**幂等**（二次运行 新增 = 0 / 跳过 = N）；③ 若仍采用写回路径，根 `index.yaml` 的注释行数**不减少**，或工具显式声明「不保全注释」且默认不写盘。
`````

### ④ 逐字校验样本（3 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | `提案：merge_index 片段顶层契约容错` |
| 2 | 假定片段顶层是 mapping（`schema_version` / `project` / `module` / `entries:`） |
| 3 | `任一文件不合规即让整个项目的合并中止（fail-fast 无隔离）` |

---

## §7 本稿自检（对照 `SUBAGENT-KNOWLEDGE-GUIDE.md` §四）

- [x] 先过价值门：段 1/3 = 高价值结论与过程（错误链条 + 判据）；段 4/5 = 高价值可复用判据（错误优先 + 指纹）；段 6 = 高价值框架缺陷（含可复现证据）；段 2 = 中价值跨版本坑（简记式一条）。**未写**：一次性数值快照、性能读数（本块本就不出）。
- [x] 错误优先：段 1 写「现象 → 根因机制 → 定位 → 修复 → 边界」，未写成「已修复」；段 3 专列「数字更正过程」「judge 抓到的最高价值问题（计数载体缺陷）」「双重教训」；段 4 三个子要点均可独立复用。
- [x] 根因是机制层面（帧契约位移 / 计数载体错位 / 变量未生效与真等价的对称性），非现象复述。
- [x] 定位含诊断方法（`javap` 比第 39 条 `invokevirtual` 目标 + tiny 映射名；`gradle --dry-run` 任务图；逐 run 切分；按 sha256 穷举分组）。
- [x] 被排除假说有标注：段 1 记 b2b 已排除（不删）；段 1 残留表如实列 6 项未闭合 + 1 项文件状态。
- [x] 载体正确：结论 → `docs/07`；跨版本坑 → `docs/08`；过程 → `docs/10`；通用 → `knowledge/discovered/`；框架缺陷 → `framework-proposals/`。
- [x] 数字全部来自本稿实读文件并附来源；**不能复现的汇总数字（16 组 / 五份 / 49-50）已就地标注按实际枚举更正**，无编造、无占位符。
- [x] 格式与各目标文件末尾现状对齐（先读末尾再写）。
- [x] 引用锚自核（workflow-patterns #137）：`2c3be2a` 与 `8974063` 均由本稿 `git cat-file -t` / `git log -1` 核实存在（前者 `fix(java-core): route bulk storage frame through per-version seam (F1)` 17:13:50+0800；后者 `docs(260912-02): close review conditions, register artifacts, supersede E5` 17:56:53+0800）。
- [ ] 段 6 的提案属**跨仓库上报**：应用后需按 RE-Framework 维护惯例（`ref-maintain`）转交，本稿不改框架仓库。

---

# 段 7 —— `knowledge\INDEX.md`

### ① 目标文件
`E:\PYTHON\CoreSwap\knowledge\INDEX.md`（现 155 行；末行 `:155` = `> 260912-01 追加（**A 线 260911-05 知识补录 + D3 共享 Java 适配核 Wave 2**；…`）

### ② 落点
**追加到文件末尾**（`:155` 之后另起一个 `>` 引用行块）。与末行同属 2026-09-12 但**不同工作块**：末行 = `260912-01`（共享核 Wave 2 抽取），本行 = `260912-02`（其遗留缺陷的定位与 F1 修复）——本行首句显式声明「同批不同块，本行 = -02」，避免与末行混淆。

### ③ 待插入正文（逐字可用；1 行）

````markdown
> 260912-02 追加（**D3 后续：1.21.6 强制 bulk storage 段帧契约缺陷定位 + F1 分版缝修复**；与同日 260912-01 的共享核 Wave 2 **同批不同块**，本行 = -02，承接其「1.21.6 强制 bulk 缺陷＝仍开放/根因未定位」）：**workflow-patterns 新增 #139（最高价值）**（① **等价性结论 MUST 与「臂变量生效自证」成对**——两臂指纹全同 ≠ 真等价，也可能是「两臂都走了同一路径」（变量没生效）⇒ MUST 附逐臂 in-log 正/负成对自证（本块 `-Dcoreswap.bulkwblog=1`：bulk 臂 `[WG-BULKWB] calls=625` vs perblock 臂 `[WG-BULKWB] calls=0` + `[WG-PERBLOCK] calls=607`），缺它则等价结论只能标「假阳性风险」（#81 家族）；② **统计前 MUST 声明并切分「计数载体」**（与 §9.7 可比性三要素同源）——无冒号 `runServer` 同时命中 `:runServer` 与 `:content-test:runServer` ⇒ 一次日志含两次 JVM 运行、计数对整份日志做（12 = 4/4 + 4/0），逐 run 切分后两侧同为 **4 WMI / 0 非 WMI**；**同一判据的旧诊断必须继承**（上一块 judge 复审已记录该机制而本块未继承 = 双重教训），低成本直证 = `gradle --dry-run <task>` 任务图；③ **汇总数字入记录前 MUST 回底本复算 + 绑臂标签 + 跨臂数字不得同格引用**（本族复发：260912-01 E4 → 本块 W8；本块实例：计划口径「16 组」按 `cmp-*.txt` 枚举 = **18 组**、「1.21.6 五份 fp 同一」穷举 = 4 份（1.20.1 = 10 份）、「修复前 49/50」属 **r16 探针臂**而 r160 真值 = **130/0/132**））+ **algorithm-fingerprints 新增 #26（最高价值）**（跨版本**分块容器 storage 写帧**契约差——1.20.1 `PalettedContainer.readPacket` 走 `readLongArray`（**VarInt 长度前缀**）vs 1.21.6 走 `readFixedLengthLongArray`（**定长无前缀**，指令 #39；yarn 名 `method_10789` / `method_68087` 已逐行核对）；失配症状 = **不是帧格式错而是整段移位** ⇒ `EntryMissingException: Missing Palette entry for index <小整数>`（本块实测只出现 **{2, 8}**，来源 = 前缀 nibble 而非位宽）、读回层指纹 `[WG-CONTENT-WB]` **归零**为首要信号；判据 = 越界索引只有离散小集合时先怀疑**整段字节移位**（位宽协商语义差预测索引散布），写端 MUST **收敛到单一分版缝函数**；**与 #22 的关系 = 同代码点的邻接面**：#22 是**读端**逐字节编码契约（1.20.1 一手源），本条是**写帧契约的版本差**（#22 的**写端**邻接面））；框架侧工具缺陷上报 → `framework-proposals/RE-FRAMEWORK-merge-index-list-root-proposal.md`（`merge_index.py` 裸列表根崩溃，W13 + judge R4）。来源：`.investigations/shared-java-core-260912-02/`（`verify-260912-02.md` / `judge-260912-02.md` / `errors-260912-02.md` W6–W8 / `evidence/javap-seam-260912-02.txt`）+ 计划件 `.investigations/000-架构设计/架构计划-260912-02-1.21.6强制bulk缺陷定位.md`（**confirmed 2026-09-12 18:13**：①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化；**不含** nether/end 覆盖、性能结论、`BULKWB_ON` 翻转）。
````

### ④ 逐字校验样本（5 条）

| # | 样本（≥20 字符连续片段） |
|---|---|
| 1 | 两臂指纹全同 ≠ 真等价，也可能是「两臂都走了同一路径」（变量没生效） |
| 2 | `无冒号 runServer 同时命中 :runServer 与 :content-test:runServer` |
| 3 | 上一块 judge 复审已记录该机制而本块未继承 = 双重教训 |
| 4 | `判据 = 越界索引只有离散小集合时先怀疑` |
| 5 | `本族复发：260912-01 E4 → 本块 W8` |

### 编号未被占用的核查方式（本稿）

1. `workflow-patterns.md`：`grep '^## 发现 #1[0-9][0-9]'` → 全量枚举后**最大 = #138**（`:2352`），**无任何 #139**；应用后 `:2390` 的 #139 = 本块条目。
2. `algorithm-fingerprints.md`：`grep '^## 发现 #'` → 最大 = **#25**（`:551`），**无任何 #26**；应用后 `:561` 的 #26 = 本块条目。（注：该文件 **#21** 是 `### ` 级标题、仍占号，故 #25 + 1 = #26 正确。）
3. `INDEX.md`（应用前）：`grep '#139'` = **0 命中**；`grep '#26'` 只命中 **build-tooling** 的 #26（`:79`）——各 discovered 文件的编号是**各自文件的命名空间**（同号在不同文件并存是本库既有常态，如 workflow-patterns #26 `:337` 与 build-tooling #26 `:569`），故不构成 `algorithm-fingerprints #26` 占用。
