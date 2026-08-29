# 【草稿】10 时间线追加：2026-08-30 Rust 宏观采样重构 + 性能定位（macro-layer-scout）

> **[DRAFT — knowledge subagent 产出草稿，待主会话应用]。** status：candidate（judge 审计后）。
> 追加到 `versions/1.20.1/docs/10-timewise-archive.md` 末尾。过程性推理进时间线，结论性内容进 07 主题篇（见 draft-07-macro-layer-refactor-perfloc.md）。
> 依据：`.investigations/macro-layer-scout/` 各文档 + cmd-output/ 记录 + review-audit-conclusion-chain.md（judge 审计意见）。

---

## 2026-08-30 Rust 宏观采样重构（multi-channel 竖切）+ 性能定位（candidate）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节 + `.investigations/macro-layer-scout/`。本轮从「性能定位」深化到「宏观采样层重构正确性 + 性能归因修正」。

### ✅ 一、顶层确认（宏观采样的真正顶层 = NoiseChunk cell grid）
- `macro-layer-topness.md`（reader 级）完整列出调度链（pyramid → tasks → noise stage → fill_from_noise）与采样机制层（`NoiseChunk::fill` + `fill_slice_into` + trilerp + combine），**其上无采样机制层**。
- 6 个疑似上层（blending/StaticCache2D/Beardifier/aquifer/dim settings/ColumnCache）逐一确认非采样层，均给源码落点。
- 与 Rust 52× 雪崩根因互证：`terrain.rs` L134-143 注释 + `macro_layer_map` §3.2——「对 final_density 采样 corners 触发内部 interpolated 雪崩」。
- **结论**：① 可靠，reader 级，**judge 确认，建议 candidate**。

### 🧪 二、multi-channel 竖切重构（正确性 diff0，但生产未接线）
- `density.rs macrolize_channels/macrolize_into`（L604-681）：final_density → 5 channels（1 BlendDensity terrain + 4 RangeChoice noodle），combine 树 Interpolated→ReadChannel 全部替换，`macrolize_probe` 验证残留 Interpolated=0、ReadChannel≥1。
- `DensityMacroSampler` diff0（n=54，平均差异 0.000000）——**局部充分非全局**（只测 cell 边界平面 fx/fz=0，未覆盖 cell 内部/边界 clamp/负 Y/跨 cell）。
- **⚠️ 关键**：`DensityMacroSampler` **只在探针文件**定义；生产 `terrain.rs fill_chunk` **未接线**（仍逐点）。「重构完成」表述须限定为「探针层验证完成，生产接线未做」。
- 标量性能：slices 构建 8.52ms + trilerp 0.3ms = 8.83ms vs 逐点 6.43ms（不省）；`std::simd` stable 不可用（需 nightly/intrinsics）。

### 🔍 三、性能定位链（corners noise 89% / 全管线 aquifer 大头）
- corners 采样：ch#0（BlendDensity terrain, 3677 节点）3.60ms 绝对大头（1225 corners）；ch#1-4（noodle 小）合计 ~0.4ms。
- `tree_vs_noise`：ch#0 完整 3.34ms → 去 noise 0.38ms → **noise 采样 2.97ms（89%），树遍历 0.38ms（11%）**——修正早前「树解释器大头」判断（milestone_record 相反，以 tree_vs_noise 为准）。
- **两测量域不矛盾**：corners noise 89% = 「宏观 corners 采样内部构成」；全管线 aquifer 大头 = 「宏观 density+aquifer 跨阶段构成」。docs 需显式区分，避免误读「noise 慢=全管线瓶颈」。

### ❌ 四、noise AVX 归因不实（judge 修正，见 multichannel-errors.md M2）
- `sample_section_avx`（noise.rs L48-96）**从未被调用**（grep 全库仅定义处）+ **函数体非 SIMD**（标量 dot3/lerp/perlin_fade）——**死代码 + 非真 SIMD**。
- 「Perlin 26.56→19.55ns (1.36x)」= `bench_noise.rs` 在 `-C target-feature=+avx` 下**编译器 auto-vec**，非手工 AVX 路径。
- 「features_probe 95.40% 不变」平凡（AVX 没接线，生产仍标量）。
- **全管线 -1% 方向可靠**（45.47→45.01ms，400 chunks）→ noise 非瓶颈判断成立（决策不因归因修正改变）。

### ⚠️ 五、ShiftDF Cache2D 潜在 bug（M1，judge 发现—已保守修正）
- `shift_y_independent` 曾写「708 个 ShiftDF 全 y 独立」但探针**只测前 5 + 单列 + 不含负 Y**；`shift_y_confirmed` 补测后 708/708 全 y 独立（含负 Y + 4 列）——但 mode 分布仅 ShiftA+ShiftB（overworld 无 plain Shift）。
- **代码层**：缓存后 `Shift` 与 `ShiftA` 都落 `_ => (x,0,z)` 分支 → **plain Shift 被强置 y=0，偏离 C++/Java 参考（实际 y）**——潜在 bug。
- **保守修正**：plain Shift 不缓存（用实际 y，保持参考语义）；ShiftA/B 缓存安全（y 无关构造性保证）。features_probe 95.40% 保持。
- ⚠️ 风险：若未来维度用 plain `minecraft:shift`，需复核 y 语义——见 multichannel-errors.md M1。

### ❌ 六、aquifer 主瓶颈方向成立，但「21.5ms」是 Partial 快照（judge 限定边界）
- Rust 全管线 45.48 < Java FULL 55 → **Rust 反快 ~1.2×**；宏观专项 Rust 34.66 > Java 23-25 → **宏观 aquifer 慢 ~1.4-1.5×（相对 Java 真差距）**。
- 「21.5ms」= 减法导出（34.66 macro − 13.14 density），非 aquifer 独立计时；且 17.5ms（16ch）vs 21.5ms（400ch）是不同测量上下文，非精确单一值。
- aquifer 内部多次翻案（P4 推翻 barrier 大头）。下一步优化前须用无污染计数探针（aquifer_*_count）锁 apply 固定开销真实构成。

### 🧰 七、工具演进 / 产物
- 探针：`macrolize_probe.rs`（channels 纯性）、`macro_sampler_probe.rs`（diff0）、`corner_sampling_breakdown.rs`、`tree_vs_noise_breakdown`、`shift_y_dependence`（含 708 全集补测）、`bench_noise.rs`（AVX micro-bench）。
- ⚠️ 产物契约（judge 驳回项）：`.artifacts/index.yaml` 无本 session 条目；`macrolize_probe.rs` 有未提交 diff + `corner_sampling_breakdown.rs` untracked——主会话需补登记 + 提交（见 draft-artifacts-index-entries.md）。

### 📌 记录指引
- 结论 → 07 主题篇追加小节（draft-07-macro-layer-refactor-perfloc.md）。
- 过程 → 本节（10-timewise）。
- **错误台账 → `.investigations/macro-layer-scout/multichannel-errors.md`（M1/M2 五段式 + 速查表）。**
- 状态：各环节 candidate（confirmed 由人类授予）；生产接线未完成。

> **[DRAFT END]**——主会话应用时删除 DRAFT 标记，追加到 10-timewise 末尾。
