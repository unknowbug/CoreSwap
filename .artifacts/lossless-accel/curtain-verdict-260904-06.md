# 幕帘课题裁决 v2：幕帘 = vanilla 正常机制，非共有偏离（candidate，260904-06）

> **supersedes**: `writer-verdict-260904-06.md` 结论 3「真根因上移 NOISE：幕帘 = C++/Rust 共有 vanilla 偏离」（原文不删不改）
> **superseded-by**: （无）
> **推翻理由一行**: P4 参照复验实测 vanilla 参照列 (195,199) y180-319 含 99 stone 族块（granite/copper 交替同幕帘形态）——「vanilla y≥201 无 stone」为参照误读；幕帘在三方（vanilla/C++/mod）均存在，且 C++ 生产密度实测全负（d≤0，stone 来自 aquifer barrier margin，b3 已证三方 aquifer 零偏离），幕帘 = vanilla 正常机制产物。
> **status**: candidate（judge 审查见 review-curtain-supersedes-260904-06.md；confirmed 留人类）
> **§9.7 口径**: 载体 = block_probe WG_DBDEBUG 生产密度 dump + density_probe 点采（%.17g）+ 参照/导出 blocks 逐块重读（同 seed 8576294172403134396、4×4 @ chunk(200,200)、列 (195,199)）；覆盖面 = 单判别列 + 生产密度全域剖面（y −64..319）；与 260904-06 幕帘线此前口径同域同 seed，但推翻了其参照解读（可比性声明：本裁决的参照读数以 p4-reference-check 实测为准）。

## 裁决

1. **幕帘存在性（candidate）**：aquifer 高位水口袋（~16 间距）+ barrier margin stone + 高 est 区 = vanilla 同构机制（aquifer.h:121-137 / aquifer.rs:327-338 三方零偏离），幕帘非任何一臂的偏离。原「0<d≤0.39 幕帘」描述作废（实测 C++ 幕帘带 d=−0.02/−0.46 全负）。
2. **课题结案**：幕帘「共有 vanilla 偏离」课题结案（对象不存在）。b1 臂密度抬升候选失去靶子；b2/b3 排除结论维持有效。
3. **真残差移交**：列 (195,199) y185-231 实测 32 块同型 `C++ stone ← vanilla granite/copper_ore/iron_ore` → 并入 **ore 族 desync 课题**（NEXT_SESSION 课题 3，底账 tuff→deepslate 1496 / andesite→stone 1270 / granite→stone 1175 / stone→coal_ore 974 / 22653 mismatch @98.56%）。新 idk：vanilla y 217-229 granite 写者身份（超 oreVein y≤50 硬门与 feature 上界 128/112）。
4. **idk 登记**：列剖面 grass/water 交替（y 67-214，两臂逐位一致）成因未查（真实地形 or raw id 映射 #8 家族嫌疑）；C++ margin stone 在幕帘带的占比未定量。

## 证据
`.investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md`（Facts/解释分离）+ `.tmp/p2full/` 五件探针产物（路径见该文）。
