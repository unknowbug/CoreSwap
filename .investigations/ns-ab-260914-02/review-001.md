# review-001 —— 260914-02 ns/section A/B 配对 record 审查（core.judge）

> 审查对象：record-260914-02.md（draft → 建议 candidate）
> 三源核对：① record/落盘产物 ② 原始验证记录 .tmp/ns-ab-260914-02/（7 log + 7 err + 脚本）③ git status/diff HEAD
> 本意见只出审查结论，不改任何 status。

## 逐项结果

### 1. record 数字 vs 原始 log 汇总行 —— ✅ 通过（全 6 正式臂 + smoke1 抽核，零偏差）
- bulk r1/r4/r5：`[WG-BULKWB] calls=576 sections_replaced=5143 air_sections_skipped=8681 build=569729(8679)/375489(11955)/392615(12710) counts_set=136/189/112 ns/call=5449008/3474419/3536385` —— 与 record §五表逐格一致。
- perblock r2/r3/r6：`[WG-PERBLOCK] calls=576 ns/call=7597723/7217233/7344082` —— 一致。smoke1=6961872 一致。
- 汇总行内自带「与 [WG-BULKWB] ns/call(total) 同仪器」声明 —— record §二声称核实成立。
- 算术独立重算：7217233/3536385=2.041（~2.04×）✓；bulk 剔 r1 均值 3505402 vs perblock 均值 7386346 → 2.107（~2.11×）✓；perblock 极差 5.27% ✓；bulk 全量极差 36.3%（以 r1 为基，声明一致）✓；剔 r1 极差 1.78% ✓。
- wbLines：6 正式臂 log 内 `[WG-CONTENT-WB] chunk(` 计数均 =576，逐臂核验一致 ✓（smoke1=0，与其 wbcontent 缺失声明自洽——smoke1 stderr 无 `-Dcoreswap.wbcontent=1`，正式臂均有）。

### 2. 自证链 —— ✅ 通过（一处措辞条件见 C1）
- calls 正负成对：bulk 臂 576/0、perblock 臂 0/576（含 smoke1 0/576），log 原文核验 ✓。
- pickup 行：每臂 stderr 恰 2 条 `Picked up JAVA_TOOL_OPTIONS`；perblock 臂 2 条均含 `-Dcoreswap.bulkwb=0`，bulk 臂 2 条均不含（且含 `-Dcoreswap.bulkwb=1` 语义侧默认）→ **record「手工补验 pickHit=2/臂」与盘上 log 一致** ✓。
- 脚本 pickHit 双重重置 bug 属实：run_nsab_1216_260914-02.ps1 行 64-70 计数后行 72 无条件 `$pickHit = 0` 覆盖 → 恒 0。record 瑕疵 1 描述（现象/根因/定位/修复/教训五段）与盘上证据吻合 ✓。
- 单变量确认：bulk/perblock 臂 JAVA_TOOL_OPTIONS 仅差 `-Dcoreswap.bulkwb=0` 一项 ✓；同 dll（脚本 -PcppLib 固定 target/release/worldgen1216.dll）✓。

### 3. 过程瑕疵声明完整性 —— ✅ 通过
- 瑕疵 2（判据未预登记 #112 违例）：已按五段式如实登记，保守读法补救路径合理 ✓。
- 瑕疵 3（rcon ConnectionRefused ×3 open）：登记属实；**补充证据**（judge 发现，供 open① 参考非结论）：脚本含**两次** `rcon stop`（行 55 与行 74），第二次 stop 时服务器已在退出流程中 → ConnectionRefused 的机制候选高度可疑就是脚本双 stop，可低成本收敛 open①。
- 冒烟臂不入账 ✓；260913-06 旧数据弃用 ✓（§六明确不进任何对比）。

### 4. 同 workload 前提与分母语义 —— ✅ 通过
- bulk 三臂 sections_replaced/air_skipped 完全一致（5143/8681）✓（perblock 臂不开 bulkwb 日志故无该行，record 以「—」呈现，诚实）。
- calls=576 vs 625 分母语义差（#132）已声明并列不连线 ✓；576=256 chunk region section 数口径声明 ✓。
- 交错序 A,B,B,A,B,A 与 log 时间戳吻合（r1 14:25→r6 14:31，smoke 14:18）✓。

### 5. §9.7 三要素与外推边界 —— ✅ 通过
- 载体（dll sha + 同仪器 destroy 汇总行）/覆盖面（单 seed × 256 chunk overworld region × 每臂 3 重复交错）/可比性（仅同批配对臂可比、260913-06 与 625 口径均不可比、绝对值不外推）三要素齐备；§九边界（单 seed/region/维度/单机）明确 ✓。

### 6. git diff 无越界 —— ✅ 通过
- `git status --porcelain`：仅 2 项 untracked（`.investigations/000-架构设计/架构计划-260914-02-ns配对A-B.md` + `.investigations/ns-ab-260914-02/`）；**零 tracked 文件改动、零源码改动** ✓。计划文件确在（承接声明成立）。

## 条件（建议 candidate 前应用，均不动原始数据）

- **C1（判读措辞修正，必改）**：§六「保守，取 bulk **不剔 r1 的最差臂** vs perblock 最好臂：3.536M vs 7.218M → ~2.04×」措辞自相矛盾——r1 未剔时 bulk 最差臂是 r1=5.449M（对应比值仅 ~1.32×），3.536M（r5）实为**剔 r1 口径下的最差臂**。~2.1× 结论本身由 2.11× 敏感性读法独立支撑、不受影响，但「不剔 r1 仍得 2.04×」的表述高估了保守度，必须改写（例如改为：「剔 r1 口径最差臂对比 ~2.04×；剔 r1 均值敏感性 ~2.11×；若强保 r1 则最差对比 ~1.32×，主判据采前两者」）。record 头部「最差臂对比 ~2.04×，剔首臂敏感性 ~2.11×」的概括同样需按此口径标注。
- **C2（脚本注记，建议）**：脚本头注「bulk 臂 calls=625」为上批口径残留（本批 576）；双 stop 结构记入瑕疵 3 教训行——两者随脚本修正/归档时处理即可，不影响本批数据。
- **C3（落盘契约，升 candidate 时）**：本 record 目前仅存在于 .investigations/，.artifacts/index.yaml 尚无对应条目；升 candidate 时按 core.artifact 补 index 登记（kind=record/status=candidate）。

## 总裁定：**PASS-with-conditions**

数据层零偏差、自证链成立、单变量前提扎实、瑕疵声明诚实完整、git 无越界——证据质量支持 draft → candidate 升级建议；条件 C1 为措辞级必改（不触碰数据与结论方向），C2/C3 为流程级。confirmed 仍留待宿主人类拍板。
