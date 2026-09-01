# 草稿：追加到 .investigations/nether-save-full/nether-save-errors.md 的 E7 五段式条目（draft，未应用）

> 本文件为 subagent 产出草稿，主会话审后应用。追加位置：E6 之后、「未闭合待查项」之前；速查表末尾加一行。

---

## E7. surface rules 噪声预加载表缺 nether 噪声：unwrap_or(0.0) 静默回退使 nether_state_selector 恒 true（.b2 遗留 bug，已修复）

- **编号**：E7（fan-out .b2 候选判定为真实 bug，2026-09-06 修复；建议 candidate）
- **现象**：nether 存档口径验证中 nether surface rule 恒走 basalt 分支（basalt deltas 相关宗石零星分支内翻转，selector 条件失效）；`surface_rules.rs` noise_threshold_sample 对 `minecraft:nether_state_selector` 等 nether 噪声 key 全部取到 0.0。
- **根因**：**隐式契约断裂 + 缺省值吞错误**——「surface rules 引用的噪声 key 必须在 step4 预加载」这一约束没有任何静态检查；`worldgen_handle.rs` step4 预加载表（L192-195 一带）只硬编码了 overworld 噪声清单（surface/surface_secondary/clay_bands_offset/badlands_*/gravel/powder_snow/packed_ice/ice/surface_swamp），nether 的 6 个噪声（nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer）全部缺失；下游 `noise_threshold_sample`（surface_rules.rs L120-137）查不到 sampler 时 `unwrap_or(0.0)` 静默回退——而 nether_state_selector 的 min threshold 恰为 0.0，回退值使条件恒 true，错误被完全吞掉，只在输出块差异里显形。
- **定位**：B1 大宗互换排查 fan-out 两候选中，.b2 候选沿 noise key 数据流（surface rule JSON 引用 → step4 预加载表 → noise_threshold_sample 查表）逐段对拍发现表缺 key（证据：`.artifacts/.b2-nether-state-selector/`）；judge 裁决「真实 bug 非主导」（B1 主导 = feature 产物 × 基底地形差，见 09 篇 B1 定论）。
- **修复**：step4 预加载表补 6 个 nether 噪声 key（全部存在于 `versions/1.20.1/data/minecraft/worldgen/noise/*.json`，非自造）。重测（seed B = 8576294172403134396，4×4 @3200,3208，存档口径）：93.5508% → **93.8988%**（+0.348pp ≈ 10× 同 dll 非确定性容差 ±369 块 ≈ ±0.035pp，超出非确定性噪声，真实改善）；E1/E3 判据核对通过（`initNether enabled=true` 且 seed 一致，`cmd-output/b2-fix-rerun.log`）。
- **教训**：
  1. **隐式契约（引用方与加载方的 key 一致性）必须有静态检查或 fail-fast**：本 bug 从 nether 接管上线起潜伏多轮，唯一显形通道是输出块差异——B1 排查绕了一圈才定位。凡「查不到就回退默认值」的路径（`unwrap_or(0.0)` / `get(key).unwrap_or` 家族）在数据驱动的 JSON 引用链上都是吞错误反模式，**跨语言通用**（Java/Rust/C++ 同罪）；至少应 log-warn 一次 + 诊断开关可直接报「unknown noise key」。
  2. **新增维度/数据域时，硬编码清单类代码是天然遗漏点**：预加载表按 overworld 清单写死后，nether 接管时无人提醒补齐——数据驱动边界评审时应专门过一遍「清单是否覆盖所有已启用维度」。
  3. **修复验证用容差倍数判真改善**：+0.348pp 远超 ±0.035pp 容差（10×），不需要逐位 Full 即可判定改善真实（Partial 分层 + 容差声明即可，§9.7）。

---

### 速查表追加行

| 错误（现象签名） | 根因 | 一句话教训 |
|---|---|---|
| E7 nether surface rule 恒 basalt 分支（noise key 全取 0.0） | step4 预加载表只含 overworld 噪声，nether key 缺失 → noise_threshold_sample `unwrap_or(0.0)` 静默回退（threshold=0.0 使条件恒 true） | **隐式契约要有静态检查；unwrap_or(0.0) 吞错误是跨语言通用反模式**；新维度上线先核硬编码清单覆盖面 |
