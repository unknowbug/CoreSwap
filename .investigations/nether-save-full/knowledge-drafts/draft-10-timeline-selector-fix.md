# 草稿：追加到 versions/1.20.1/docs/10-timewise-archive.md 的 2026-09-06 时间线条目（draft，未应用）

> 本文件为 subagent 产出草稿，主会话审后应用。追加位置：时间线末尾（2026-09-05 B1 定论条之后）。

---

## 2026-09-06 nether_state_selector 预加载表修复（.b2 遗留项闭合）✅

> 承接 2026-09-05 B1 定论条 fan-out .b2 遗留修复项（⚠️ 真实 bug 非主导，待修）。结论 → 09 篇追加小节（草稿 knowledge-drafts/draft-09-selector-fix.md）；错误 E7 → nether-save-errors.md。

### ✅ 一、修复
- `WorldgenRust/src/worldgen_handle.rs` step4 surface rules 噪声预加载表（L192-195 一带）补 6 个 nether 噪声：`minecraft:nether_state_selector` / `patch` / `soul_sand_layer` / `netherrack` / `nether_wart` / `gravel_layer`（全部存在于 `versions/1.20.1/data/minecraft/worldgen/noise/*.json`）。
- 机制：预加载表原只含 overworld 噪声 → `surface_rules.rs` noise_threshold_sample（L120-137）查不到 sampler 时 `unwrap_or(0.0)` → nether_state_selector（min threshold=0.0）恒 true → 恒 basalt 分支。

### ✅ 二、验证（存档口径，seed B = 8576294172403134396，4×4 @3200,3208）
- 修复前 93.5508% → 修复后 **93.8988%**（match=984600/1048576），+0.348pp ≈ 10× 同 dll 非确定性容差（±369 块 ≈ ±0.035pp）→ 真实改善。
- E1/E3 判据核对通过：log `[CppBridge] initNether enabled=true` 且 seed 一致（`.investigations/nether-save-full/cmd-output/b2-fix-rerun.log`）。
- 分族：总 mismatch 63,976（solid_solid 62,850 / van_solid_rust_air 580 / van_air_rust_solid 546）；soul_soil ref 5474 vs save 1334 仍偏低——selector 已生效，soul_soil 大头疑似 Java feature 阶段（B1 主导机制的正常残差，非本 bug）；quartz/gold/magma 偏高归 ore features（待 A1+B4 重估）。
- 口径声明（§9.7）：93.8988% = 存档口径（Rust noise+surface + Java carvers/features），与纯 Rust 77.43% 不可比。

### 📌 记录指引
- 结论 → 09 篇追加小节（supersedes：取代「B1 定论」节 .b2 待修注记，置信度 candidate）。
- 错误 E7（预加载表隐式契约缺 key 静默回退 0.0）→ `.investigations/nether-save-full/nether-save-errors.md`。
- 状态 ✅：修复完成、验证通过（candidate，judge/confirmed 留后续）。
