# review-001 —— 260914-01 内存口径工作块 judge 审查意见（core.judge，只出意见不改 status）

- 审查对象：`record-260914-01.md`（draft，candidate 建议）
- 三源：① record + .tmp 原始数据 ② git HEAD 436d95d + 工作区 diff ③ 本 judge 独立重算（gc log / csv / err 逐项）
- 日期锚：宿主实际时间核对（审查轮，无新命名产物）

## 逐项核对

### 1. record 数字 vs 原始数据 —— 通过（全部独立复算吻合）

- **peak 表**：csv 逐行核对——r4 行5 `1454.0/1645.6`、r5 行6 `1556.4/1756.5`、r6 行6 `1515.6/1738.5`，与 record 完全一致（各 csv 内即最大样本）。
- **极差**：peakPriv 1756.5−1645.6=110.9 ✓；110.9/1645.6=6.74% ✓。peakWS 1556.4−1454.0=102.4 ✓；102.4/1454.0=7.04% ✓。lastUsedAfter 502−480=22、22/502=4.4% ✓。
- **GC 表**：judge 按「Pause Young 行」口径独立重算——pauses 49/47/48、minUsed 12/12/12、maxUsed 861/864/872、lastUsed 502/493/480，**与 record 逐格吻合**。注意：record 未写明该计数口径（Remark/Cleanup 停顿被排除；全停顿口径为 69/69/70）——建议在 record 的 GC 表加一行口径注记（不阻塞）。
- **末段抽核**：r4 `GC(55..58)` 733M->437M(799M)…757M->502M(825M) 与 record 引文逐字一致 ✓；Concurrent Mark Cycle + 堆容量动态扩属实 ✓。
- **自证门**：r4/r5/r6 `.log.err` 各含 **2 条** `Picked up JAVA_TOOL_OPTIONS: … -Xmx2G -Xlog:gc:…` pickup 行（daemon+server，stderr 实证）→ xmxApplied=2 ✓；stdout `armed` 行=1 ✓、wb 行=576 ✓；`crash` 仅命中 fabric 模块清单行 `fabric-crash-report-info-v1`（非真实崩溃），sentinelCrash=0 合理 ✓。
- **VOID 留档**：gc-*r6-18220.log = 160B 空壳盘上现存 ✓；r1-r3 err 6884B vs r4-r6 7042B 差 ≈2 条 pickup 行量级，与「第一版门只扫 stdout」根因叙事自洽 ✓。

### 2. 置信度合法性 —— 通过

- status=draft，candidate 为建议且明示待 judge+用户拍板，全文无 confirmed 越权 ✓。
- §9.7 三要素齐备（载体/覆盖面/与 #119 历史口径不可比已显式声明），无跨口径引用数字、无夸大 ✓。Degraded 声明（无 jcmd 直读、5s 粒度峰值归因未知）如实 ✓。

### 3. 边界声明 —— 通过

单 seed 单 region n=3、无 vanilla 臂、~60s 短窗、peakPriv 含 native 未分解、proc_id 列瑕疵——六条边界全部如实落入第六节，与本 judge 独立看到的 csv 事实（proc_id 确为属性对象字符串）一致 ✓。

### 4. 错误记录五段式 —— 通过

r1-r3 VOID：现象（160B 空壳+门扫错流）/ 根因（`${}` 词法歧义 + pickup 流向）/ 定位 / 修复（含换标签重跑、VOID 不删）/ 教训（可复用判据：注入通道产物非空+pickup 流实证）——五段完整，教训超出一次性的价值，可入 discovered 候选 ✓。

### 5. 判据设计缺陷 —— 有条件通过（两条 condition）

- 摆动带 ~7% 计算口径自洽（相对最小值，表头已标）✓。
- **无 #112 式「读法未预登记」**……**部分存在**：极差带是 n=3 事后归纳，且「本底噪声带」与「被测 run」未来将来自同一生成过程——判据草案未预登记**带的再校准规则**（新 run 落带内是否并入带、并多少跑一次重校、固定 once-and-for-all 还是滚动）。
- **存在 #129 式「终态含糊」轻度残留**：「超出摆动带才算信号」未定义——单次 run 超带即信号，还是 n≥3 中 k 次超带？超带判定对带缘的裕量是多少（110.9MB 极差本身是 n=3 的小样本估计，第 4 个 run 落在带外 5MB 算不算）？建议在判据升 candidate 前补一段「信号判定操作定义」。
- n=3 极差作为噪声估计的统计脆弱性已在边界节隐含（覆盖面窄），但未点明「极差随 n 单调增长」——建议措辞补一句。

### 6. git diff 核对 —— 有条件通过（1 条 condition）

- 无任何源码/引擎/门控改动（tracked 文件 diff 为空，HEAD 436d95d）✓，与「纯测量课题」声明一致。
- **越界残留**：`versions/1.21.6/java/,uptime`、`,uptime.0`、`,uptime.1` 三个未跟踪杂散文件——命名特征与 VOID 轮 gc log 路径缺陷（`$gcl:time` → `-Xlog:gc:file=` 路径畸变，`,uptime` 为 options 段落入文件名）高度吻合，属 VOID 轮遗留垃圾，落在 record 声明的「只有 .investigations 新增 + 计划文件」范围之外。要求：确认后清理或在 record 补记一句来源声明（judge 不动手）。

## 总裁定：**PASS-with-conditions**

数字、自证门、错误记录、边界声明、置信度纪律全部核实通过（judge 独立重算零偏差）；不构成驳回。条件（均不阻塞 draft→candidate 建议，但应在用户拍板前或判据首次实际使用前闭环）：

1. **C1（判据）**：补「信号判定操作定义」——单次/多数超带、带缘裕量、n=3 极差的再校准规则（消 #129 终态含糊残留 + 预登记读法）。
2. **C2（现场）**：清理或书面声明 `versions/1.21.6/java/,uptime*` 三个 VOID 轮遗留文件（git 声明范围对齐）。
3. **C3（建议，非条件）**：GC 表加一行计数口径注记（pauses=Young 停顿，不含 Remark/Cleanup）；教训 1/2 值得作为「测量有效性自证门」签名进 knowledge/discovered 候选（按流程由 knowledge subagent 产出草稿）。

推荐状态：**建议 candidate**（附上述条件）；confirmed 留给用户。

*本意见为建议非命令；未修改任何文件 status。审查意见落盘本文件。*
