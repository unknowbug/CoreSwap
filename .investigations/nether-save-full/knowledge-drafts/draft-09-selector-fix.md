# 草稿：追加到 versions/1.20.1/docs/09-multi-dimension.md 的小节（draft，未应用）

> 本文件为 subagent 产出草稿，主会话审后应用。追加位置：09 篇「B1 定论」节之后。
> 置信度：**candidate**（待 judge + 用户拍板 confirmed）。验证分层：**Partial**（存档口径端到端对齐 + 日志判据核对，非逐位 Full）。
> §9.7 口径声明：93.8988% 是**存档口径**（Rust noise+surface + Java carvers/features），与纯 Rust 口径 77.43% **不可比**（载体不同）。

---

## nether_state_selector 预加载表修复（.b2 遗留项闭合，candidate，2026-09-06）

> supersedes：**本文取代本篇「B1 定论」节附带定论中的 .b2 待修注记**（「nether_state_selector 恒 0.0 → 修复值得做，待修后重测」）——该项已修复并重测，待修状态作废；原注记按 §15.4 不删。fan-out 候选记录：`.artifacts/.b2-nether-state-selector/`。

### 修复内容

`WorldgenRust/src/worldgen_handle.rs` step4 surface rules 噪声预加载表（L192-195 一带）原只含 overworld 噪声（surface / surface_secondary / clay_bands_offset / badlands_* / gravel / powder_snow / packed_ice / ice / surface_swamp），**缺全部 nether 噪声**。下游 `surface_rules.rs` 的 `noise_threshold_sample`（L120-137）查不到 sampler 时 `unwrap_or(0.0)` 静默回退 → `nether_state_selector`（min threshold = 0.0）条件恒 true → nether surface rule 恒走 basalt 分支。

修复 = 预加载表补 6 个 nether 噪声：`minecraft:nether_state_selector` / `patch` / `soul_sand_layer` / `netherrack` / `nether_wart` / `gravel_layer`（全部存在于 `versions/1.20.1/data/minecraft/worldgen/noise/*.json`）。

### 验证（存档口径，seed B = 8576294172403134396，4×4 @3200,3208，参照 = vanilla FULL）

| 项 | 数字 | 判读 |
|---|---|---|
| 修复前基线 | 93.5508%（上轮） | 同 dll 非确定性容差实测 ±369 块 ≈ ±0.035pp（B1 轮过程事实） |
| 修复后 | **93.8988%**（match = 984600/1048576） | **+0.348pp ≈ 10× 容差 → 超出非确定性噪声，真实改善** |
| E1/E3 判据核对 | 通过 | log `[CppBridge] initNether enabled=true` 且 seed 一致；log = `.investigations/nether-save-full/cmd-output/b2-fix-rerun.log` |

### 修复后分族（b1_family_split.py / b1_id_totals.py）

- 总 mismatch 63,976：solid_solid 62,850 / van_solid_rust_air 580 / van_air_rust_solid 546。
- soul_soil：ref 5474 vs save 1334（仍偏低）——**selector 噪声已生效**，但 soul_soil 大头疑似在 Java feature 阶段，属 B1 主导机制（feature 产物 × 基底地形差）的正常残差，**不是本 bug**。
- soul_sand 2457 vs 1471；quartz_ore / gold_ore / magma 仍偏高——ore features 归因（待 A1+B4 重估）。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照（WGB2）。
- 覆盖面：4×4 chunk 全高度（min_y=0, height=256）。
- 可比性：93.8988% 为**存档口径**（Rust noise+surface + Java carvers/features 端到端），与纯 Rust 口径 77.43% **不可比**；与修复前 93.5508% 同口径可比（容差 ±369 块已声明）。

### 状态

- 置信度 candidate（验证 = Partial：存档口径端到端，非逐位 Full）；confirmed 留用户。
- 过程 → 10 时间线 2026-09-06 条；错误 E7（隐式契约缺 key 静默回退）→ `.investigations/nether-save-full/nether-save-errors.md`。
