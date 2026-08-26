# Rust density_builder.rs 完成 + 与 C++ 参照对齐（candidate）

> 2026-08-24 | 任务：修复并完成 WorldgenRust/src/density_builder.rs 的 DensityBuilder（原 3 编译错误 + &self 内改 mut 设计问题 + build_node 分派不完整 + mn/mx 占位 + resolve_ref POC）
> 状态：candidate（已与 C++ 参照逐位验证，待 judge 审查 + 用户拍板）

## 结论

WorldgenRust 的 DensityBuilder/buildNode 现已**逐位对齐 C++ `density_builder.h`（buildNode）**（⚠️ 对齐基准 = C++ buildNode，**非 Java vanilla 逐块**——见「域/边界」）：
对 overworld **16 个密度函数文件**（10 顶层：base_3d_noise / continents / erosion / ridges / ridges_folded / factor / offset / jaggedness / depth / sloped_cheese + 6 caves：entrances / noodle / pillars / spaghetti_2d / spaghetti_2d_thickness_modulator / spaghetti_roughness_function），在 **10 个采样点（160 采样值）+ min/max** 上，Rust 输出与 C++ `rust_ref_check.exe` 参照**数值完全一致**（8 位小数逐位相同，含全部 min/max）。

**验证方式**：Rust `overworld_probe.rs`（external_loader 读盘 + resolve_ref 递归 + 采样）vs C++ `rust_ref_check.cpp`（用同 seed + 同 density_builder.h 构建同批 DF + 同点采样），输出规范化后差分 = 0（仅 Rust probe 末尾多一行 summary，非数值差）。seed = 8576294172403134396。验证分层 = **Full（逐位）**，对齐基准 = C++ buildNode。已冻结快照 + 哈希 + 复现命令于 `.investigations/rust-density-builder/verification-record.md`。

**采样覆盖**：10 点/函数覆盖跨 min_y..max_y 与跨 xz；但 factor/jaggedness/offset 在选定点近退化（factor=3.95 左端平段、jaggedness=0.0、offset 部分同值）——**未触 spline 拐点区**，逐位断言以「160 采样点 + min/max」为界（judge P1-① 记录）。

## 追加里程碑：full overworld finalDensity 端到端（✅ 已对齐 C++）

用**纯 Rust** 读 `noise_settings/overworld.json` → `noise_router.final_density` → 经同一个 buildNode 构建**整棵最终密度树**（big `min(squeeze(mul(0.64, interpolated(blend_density(...)))), caves/noodle)` 链 + `overworld/...` + caves 引用）。**与 C++ `rust_ref_check` 在 10 点 + min/max 上数值逐位一致**（final_density 规范化差分 = 0）。

- 结果抽样：`final_density min=-0.45833333 max=0.45833333`（跨 min_y..max_y 与正负 xz）；(0,0,0)=0.06590008、(8,64,8)=-0.25433936、(200,40,200)=-0.00016419…（C++ 全同）。
- 若干点恒 -0.45833333（不同 chunk/coords）——经 C++ 确认是 **vanilla 真值**（非 Rust bug；interpolated/range_choice 在该区段塌缩到常量）。
- 探针：`WorldgenRust/src/bin/finaldensity_probe.rs`；已冻结 `rust_fd_out.txt / cpp_fd_out.txt / finaldensity_probe.rs`（SHA 见 verification-record.md）。

**意义**：Rust 的 buildNode 不再限于单密度函数，而是能把**整个 overworld 最终密度树**逐位复刻出来——这是 Rust 全量重写「能替代 production density」的核心能力里程碑。

## 本批变更（WorldgenRust/）

1. **编译修复**：`NoiseParameters` 加 `#[derive(Clone)]`；`let mut base`；`DensityBuilder::new(seed, min_y, noise_height)`（删 `new2` 及 `md:&[minY;0]` 笔误）。
2. **&mut self 贯穿**：`build_node`/`build_spline`/`build_spline_node` 全改 `&mut self`，删除 `unsafe self_mut` 不健全设计（原先 &self 内改 mut 调 get_noise_sampler）。
3. **density.rs 扩展**：新增 `ShiftMode`/`WeirdRarity` 枚举、`InterpolatedNoiseData`（old_blended_noise 三 octave sampler，Rc 包裹保 Clone）、`ShiftDF`/`ShiftedNoise`/`RangeChoice`/`YClampedGradient`/`WeirdScaled`/`BlendAlpha`/`BlendOffset`/`BlendDensity`/`Wrapping`/`InterpolatedNoise`/`Lazy` 变体 + sample/min_value/max_value。
4. **noise.rs 扩展**：`OctavePerlinNoiseSampler::new_legacy`（直接消费 random，含 skip(262)）、`range_closed_amplitudes`、`get_octave`、`method_40556`、`maintain_precision` 改 pub。
5. **build_node 分派补全**：shifted_noise / shift_a/b/shift / range_choice / y_clamped_gradient / weird_scaled_sampler / blend_alpha/offset/density / cache_once+cache_all_in_cell（Wrapping）/ old_blended_noise；数字/字符串裸节点处理（C++ buildNode L31-47）。
6. **mn/mx 精确**：NoiseDF maxValue = noise.getMaxValue()；BinaryOperation::create 符号 mn/mx（MUL 全符号区间）+ add/mul 常量折叠→LinearOp；UnaryOperation::create（ABS/SQUARE mn=max(0,imin)）。
7. **resolve_ref 真实化**：registry 查找 + 特殊键 shift_x/shift_z/y/zero + overworld/<name> 惰性加载（Lazy placeholder 循环引用保护 + external_loader）+ register_function/parse_file/set_external_loader。

## 关键 bug 修复（错误优先记录，均在对拍中抓到并修复）

1. **UnaryOperation ABS mn 错**：`mn = |imin|`（初版 `mn.max(0.0)`）→ 应 `max(0.0, imin)`。症状：`abs(ridges)` 范围塌缩成常量 5.7143 → `ridges_folded` min==max=-14.1429（退化）。C++ 参照 L184-188 确认。
2. **`minecraft:clamp` 读字段错**：读 `"argument"` → 应 `"input"`（C++ buildObject L92 用 `arg("input")`）。症状：caves/entrances 等含 clamp 的文件 build_node 收到 `Null` 报 unsupported。
3. **`InterpolatedDF` min/max 委托错**：`-arg.max_value()` → 应 `arg.min_value()/arg.max_value()`（C++ L560-561）。症状：caves/noodle min_value 差 0.15. 注意 `InterpolatedNoiseDF`（old_blended_noise）的 `-maxVal` 是另一类型，不改。

## 验证证据

- Rust 探针输出：`versions/1.20.1/cpp/build-msvc/bin/rust_out.txt`（overworld_probe，16 DF）
- C++ 参照输出：`versions/1.20.1/cpp/build-msvc/bin/cpp_out.txt`（rust_ref_check，16 DF）
- C++ 参照源码：`versions/1.20.1/cpp/worldgen/src/rust_ref_check.cpp`
- 参照值 `factor=3.95000005` 与 C++ 既有 `ref_probe factor=3.950000048` 一致

## 域/边界

- 仅验证 density == C++ buildNode（overworld 16 DF + full finalDensity）。未做：vanilla 逐块对齐（block_probe/ref density 数据）、多线程/性能、`Rc<RefCell>→Arc/thread_local` 生产化。
- ✅ noise_params 已从硬编码表切到权威 `noise_params.json`（`load_noise_params_file`），消除两端共享硬编码表的转录风险（judge P2-e 收口）；file-loaded 后 finalDensity 仍与 C++ 数值逐位一致。
- `old_blended_noise` 已对齐（base_3d_noise min/max == ±87.5515 与 C++ 一致）。
