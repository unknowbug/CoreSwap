# 草稿：knowledge/discovered/compiler-idioms.md 发现 #10

目标落盘位置：`knowledge/discovered/compiler-idioms.md` 末尾追加（发现 #9 之后；编号经核对：现有最后条目为发现 #9，本条 = **#10**）；同时按 INDEX.md 纪律在 `knowledge/INDEX.md` compiler-idioms 条目摘要同步补一行（主会话应用时处理）。

---

## 发现 #10: Rust 半开区间 rev().step_by() 复刻 Java 含两端递减 for 循环的 off-by-one

- **发现时间**：260903-13；**发现者**：core.worker 草稿（lossless-accel off-scan+cornerfix 课题）+ 主会话应用；**来源定位**：commit 3e2e67d + `.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md` + `.investigations/lossless-accel/review-offscan-cornerfix-260903-13.md`（Rust 侧 `WorldgenRust` est 扫描；Java 侧 forge official sources `NoiseChunk.java:174` `computePreliminarySurfaceLevel`）；**置信度**：candidate（修复后两臂四臂 hash 完全一致 f2b1a3932c6e589e + Java 角列 256/256 0 diff，judge PASS，confirmed 待用户拍板）；**module**：re-code / swe（跨语言循环移植）。

### 观察

Java `for(l=top; l>=bottom; l-=step)` 是**含两端**的递减扫描；移植 Rust 时写成 `(bottom..top_exclusive).rev().step_by(step)` 会引入**两个独立的错位**：① 半开区间上端 `top_exclusive` 使 rev 首点 = top−1（本例 319 vs Java 320）；② 下端 `bottom`（exclusive）使 rev 末点 = bottom+1 起、实际扫到 bottom+step−step 即**下界漏扫 step 的端点语义差**（本例 Java 扫到 −64 含端）。首点差与下界包含性是**两个独立参数**，只对齐其一（如仅把上端改 top+1）修不完整。

### 证据

- 本例签名：修复前 off 臂 est 角列对 Java **恒差 −1**（64/64 全偏、delta 恒 −1，含 c0 原点角——规整性系统偏移而非随机差）；敏感角 (201,200) 值 55 vs Java 56。
- 修复（扫描对齐「首点值 + 下界包含性」）后：两臂四臂 hash 完全一致（`f2b1a3932c6e589e`）；Java est 角列 off/shared 各 256/256 一致 0 diff。

### 如何利用

- **判据**：跨语言移植递减扫描循环时，必须显式对齐**「首点值」+「下界包含性」两个独立参数**，逐一与 Java 源码核对（`l>=bottom` 含端 vs Rust `..` 半开），禁止凭「看起来等价」直译。
- **签名**：结果相对参照**恒差固定小量（如 −1）且全样本规整偏移** = 扫描/索引 off-by-one 类错位，优先核对循环端点语义，不是精度/随机性问题。
- 等价复刻形态：Java `for(l=top; l>=bottom; l-=step)` → Rust `(bottom..=top).rev().step_by(step)`（含端 RangeInclusive），并核对 `top−bottom` 可被 step 整除时的末点行为。
- 交叉引用：workflow-patterns #25（静态调研/直译结论失真——本例 +15 角参数即其第三例实例）；compiler-idioms 发现 #7（锚换算端点 off-by-one 同族：端点语义 inclusive/exclusive 是跨语言移植的第一易错点）。
