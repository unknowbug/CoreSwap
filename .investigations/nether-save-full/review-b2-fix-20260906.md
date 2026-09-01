# Judge Review — nether_state_selector 预加载表修复 + SURFACE 口径残差量化（260901-04）

审查人：judge（subagent，只出意见不改 status）。基线：commit `92c2d99`（fix(nether): preload missing nether noise keys），工作区 diff 为空（改动已提交，非任务描述所称「未提交/部分 staged」——见 C5）。

## 逐项结论

### 1. 代码修复 — PASS
- 6 个新增 key（nether_state_selector / patch / soul_sand_layer / netherrack / nether_wart / gravel_layer）与 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise/*.json` 实际文件名逐一核对一致，无拼写错误。
- 完整性核验（judge 独立提取）：nether.json 中 surface rule 引用的 noise 参数恰好就是这 6 个（noise_router 里的 temperature/vegetation 不属于 surface rule，不需预加载）——补齐集合精确，无遗漏无冗余。
- `surface_rules.rs` noise_threshold_sample（L120-137）确为查不到 sampler 时 `unwrap_or(0.0)` 静默回退；nether_state_selector 的 `min_threshold: 0.0` 已在 nether.json 原文核实——「回退值使条件恒 true」的机制描述成立。

### 2. 验证证据 — PASS
- `b2-fix-rerun.log`：`match=984600/1048576 (93.8988%)` ✓；`[CppBridge] initNether seed=8576294172403134396 enabled=true` ✓（E1/E3 判据满足）；dll 同步行在位。
- `b2-surface-ref-export.log`：`[BlockProbe] worldSeed=8576294172403134396` 与存档口径 seed 一致 ✓；参照文件 `vanilla_8576294172403134396_4_3200_3208_nether.blocks` 落盘可引用。
- 算术复核全对：+0.348pp = 3649 块 ≈ 10×369；21296/1048576 → 97.9691%；812496/1048576 → 77.4857%；分族和 62,850+580+546 = 63,976 = 总 mismatch ✓；233,197+2,871+12 = 236,080 = 1,048,576−812,496 ✓。

### 3. docs 落盘 — PASS（附 C1/C2 建议）
- 09 篇两小节 + 10 篇 260901-04 条目 + 错误台账 E7（五段式完整，现象/根因/定位/修复/教训俱在）+ 速查表行，均核到。
- 数字跨文档一致（09 ↔ 10 ↔ E7：93.5508→93.8988、+0.348pp≈10×容差、77.4857%、21296/1048576=97.9691%、分族、top mismatch 157,658/35,031/15,678）。
- §9.7 三要素完整：两小节各有载体/覆盖面/可比性声明（L276-279、L312-315），三口径（77.4857 SURFACE / 93.8988 存档 / 77.43 纯Rust-FULL）明确分列且声明互不可比，包括对「数值接近属巧合」的防误并声明——到位。
- §15.4 supersedes 合规：原「待修」行保留未删，仅追加 `[supersedes 260901-04]` 注记（L249-250）；B1 行旧候选同样只加注（L193、L203）。

### 4. 声明诚实性 / 定级 — PASS
- Partial 分层声明诚实：存档口径是端到端对比 + 日志判据，非逐位 Full，两小节均已声明且说明「容差倍数判真改善」的 Partial 判定逻辑（E7 教训 3）。
- candidate 定级恰当：有落盘可引用的量化证据 + 超容差改善判据，未越权标 confirmed（L283 明示「confirmed 留用户」）。
- 「soul_soil 大头疑似 Java feature 阶段」保持 draft 语气（「疑似」「待 A1+B4 重估」）✓；「4×4 局部观察勿外推」在 09 L310 与 10 L2364 双处声明 ✓；WARN-4 备择排除给出的是架构事实（cppReplace 拦截面）而非推测，论证合规。

## CONCERN（均非阻塞）

- **C1**：E7 教训 1 自认「unwrap_or(0.0) 吞错误至少应 log-warn + 诊断开关报 unknown noise key」，但本次修复未在 `noise_threshold_sample` 加任何 warn/fail-fast——教训记了账，代码未兑现。同类隐患（未来维度 key 拼错）仍会静默。建议列为后续 todo。
- **C2**：预加载表仍是硬编码清单（E7 教训 2 已自我指出）——与「数据驱动架构铁律」存在张力。中期可改为从 surface_rule JSON 的 noise 引用动态收集/预加载，根除清单覆盖面问题。
- **C3**：SURFACE 对比证据链依赖 `.tmp/b2_surface_residual.py` 与 `.tmp/b1-rlib-blocks.bin`（docs L294 明文引用），`.tmp/` 是临时区不入库，rust dump 体积大可理解，但对比脚本建议迁 `.investigations/nether-save-full/` 或 cmd-output 留一份副本，防证据脚本丢失。
- **C4**：±369 块容差样本量 n=2（两次 run），「10× 容差」结论方向可靠（3649 ≫ 369），但容差本身置信度有限——docs 已用「实测」而非「上界」措辞，可接受，仅提示。
- **C5**（记录性）：任务描述称改动「未提交/部分 staged」，实际全部已在 `92c2d99` 提交、工作树对相关路径干净——三源核对无差异，仅表述与仓库状态不符。
- Rust 不在 Anchorlaw scanner 支持语言内（v0.16/17），本次以人工对拍替代静态扫描门禁——符合 v0.17 域声明，记录备查。

## 整体建议

**可授予 candidate（维持），无必须修正项。** 修复正确、证据链完整可引用、数字全部复核一致、口径声明与 supersedes 链合规、分层声明诚实。C1（unwrap_or 加 warn）建议作为独立小 todo 跟进后再谈 confirmed；confirmed 照例留给用户拍板。
