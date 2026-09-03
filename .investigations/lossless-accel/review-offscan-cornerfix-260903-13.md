# Judge 审查意见 — 260903-13「off 臂 −1 扫描偏移修复 + 角参数 +15→+16」candidate

- 审查对象：`.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md`（candidate）
- 审查模式：三源交叉核对（§15.4）+ 语义怀疑姿态核对
- 本意见只出分级结论，不改任何 status（confirmed 留人类）。

## 总评

**PASS（建议维持 candidate；1 项 should-fix 后不影响用于翻默认前置）**

修复代码、验证数据、结论文档三者一致；语义逐点对齐 Java 权威源成立；证据链完整可信。唯一实质性问题是 verdict 前置头 `verification: full` 与正文「Partial」声明自相矛盾（should-fix，非 blocking——机械判据三类均未触发）。

## 分级发现

### PASS

1. **三源一致**：工作区 diff（`WorldgenRust/src/worldgen_handle.rs`）与结论声明逐项一致——off 臂 `for..rev` 半开区间改 `let mut y = min_y + noise_height; while y >= min_y { ...; y -= 8; }`；`surface_heights4` 与 dump `corner_params` 同步 +15→+16；注释更新。无夹带代码改动（index.yaml 追加 4 条目属产物台账预期项，见 info #3）。
2. **P0 交接验证合法（§16.3）**：`estopt-ab-arms-p0-260903-13.txt` 四臂与 260903-12 记录逐项一致——off `74f5dfc4eede8ef4`、shared `8bff408735f1560d`、l2==off（hit_rate 84.9%）。交接结论先廉价独立验证后继承，合规。
3. **零语义差核心证据成立**：`estopt-ab-arms-p1fix-260903-13.txt` 四臂 agg hash 同值 `f2b1a3932c6e589e`，seed 一致（8576294172403134396）。
4. **est 对比数据与结论一致**：`est-compare-p1fix-260903-13.txt`——java entries=11877 conflicts=0；off/shared 各 match=256 diff=0 java-missing=8；敏感 chunk (201,200) 两臂均 c0:48 c1:56 c2:48 c3:56，与 Java 一致（旧 off=55 确为 −1 扫描伪差）。
5. **off 臂扫描语义逐点对齐**（`worldgen_handle.rs:533-539` vs `NoiseChunk.java:169-181`）：首点 `min_y+noise_height`（overworld = -64+384 = **320** ✓）；`while y >= min_y` 含下界（末点恰为 **-64** ✓）；步长 **8**（= cellHeight ✓）；未命中回退 `i32::MAX` = Java `Integer.MAX_VALUE` ✓；阈值 0.390625 ✓（该行未改动，与 Java `initialDensityNoJaggedness` 对应关系维持 260903-11 既有结论）。
6. **+16 角参数两臂等价 Java**：`MaterialRules.java:496-499` `chunkToBlockCoord(i+1) = (i+1)<<4 = cx*16+16`；off 臂直采 `(cx*16+16)` 与 Java `estimateSurfaceHeight` 输入一致；shared 臂 `aquifer.rs:343-344` 先 `(x>>2)<<2` 量化，+16 为 16 倍数量化不变 → 两臂在 +16 下语义等价，且由 256/256×2 实测闭环佐证。
7. **无 +15 残留**：grep `+ 15` / `16 + 15` 于 `worldgen_handle.rs` 零匹配。
8. **对比脚本逻辑可靠**（`.tmp/estdump/compare_260903_13.py` 只读复核）：Java 表以量化 `(qx,qz)` 为键并计 conflicts（=0，无键覆盖歧义，last-wins 无影响）；Rust 角列 `q(px),q(pz)` 匹配 Java 键，坐标语义一致（两侧同为 4 倍数量化块坐标，无 #23/#24 类错位风险）；missing 计数与 diff 明细落盘，无选择性比对。
9. **§9.7 可比性声明在场**：载体（WG_EST_DUMP 角列）/覆盖面（64 chunk×4 角+敏感点）/与 260903-12 口径可比性三要素齐备。
10. **遗留声明诚实**：block_probe 存档口径 Full 回归未跑已明确声明（ Partial+est 全角列组合口径如实交代）；surface_rules.rs:505 panic 课题如实移交下轮。status 标注 candidate 合法，无违规 confirmed。
11. **产物契约**：verdict 落盘 + `.artifacts/lossless-accel/index.yaml` 已登记（id `offscan-cornerfix-260903-13`，status: candidate）。

### CONCERN（should-fix，不阻塞）

1. **前置头与正文分层声明矛盾**：verdict frontmatter `verification: full`，正文却写「声明 Partial+est 全角列组合验证；未做全量 block_probe 回归」。按验证分层定义（Full=block_probe 逐位），本轮应为 Partial+组合证据。**建议将 frontmatter 改为 `verification: partial`（或 `partial+composite` 并注一行口径）**——矛盾声明若原样进 docs 会污染 §9.7 口径纪律。机械判据三类（测试失败/编译失败/声称与实现矛盾）中此项接近第③类但属自声明内部矛盾而非实现矛盾，按 §15.4 归 should-fix。
2. **hash 一致性证明力边界**：四臂同 hash 证明「四臂彼此零差」，Java 对齐由 256 角列独立证明——两者叠加结论「向 Java 收敛的纯性能优化」成立，但覆盖面限于 c0 区 64 chunk + est 角列通路；surface lerp 后续链路（block 级）未经本轮直接验证（已由遗留声明覆盖，此处仅记录提醒翻默认决策时知悉）。

### INFO

1. 声明「+13/-5」与 git stat（worldgen_handle.rs +18/-5）差 5 行为注释行，口径差无实质问题。
2. 8 java-missing 的「预热 chunk (400..402,400) 区外」归因未在脚本内断言，但数量小、两臂对称、不影响 256/256 主结论；如需可由 rust CSV 原始行一行复核。
3. 工作区 diff 另含 `.artifacts/lossless-accel/index.yaml`（+16 行）——任务书称「只应含 worldgen_handle.rs」，经核为本次 verdict + 260903-12 三份结论的 index 补登记，属产物契约预期项，非夹带代码改动。
4. noise_height/height、cellHeight=8 的 overworld 同值前提已在 260903-12 D3 声明限定（收敛仅限 overworld），本次修复未扩大该边界，nether 路径不在本轮结论范围。

## 推荐状态

**维持 candidate**（修复证据充分、语义对齐成立）；建议人类在采纳 CONCERN #1 修正 frontmatter 后再考虑作为 WG_EST_SHARED/L2 翻默认的前置引用；翻默认落地如需存档口径 Full 证据，按遗留声明下轮补 block_probe 回归。
