# Rust 重写进度记录（2026-08-24 本 session）

> 主会话临时记录（过程）。交接详见 NEXT_SESSION.md「当前主线」节。规划见 rust-rewrite-plan.md。

## 已完成且编译通过（WorldgenRust/）
- **noise.rs**：Perlin/Octave/Double Perlin + 辅助（floorD/lerp/perlinFade/GRADIENTS/maintainPrecision）——**逐位对齐 C++/Java**（md5 + next_double float 语义修复后，Rust == C++）。
- **xoroshiro.rs**：XoroshiroRandom/128PlusPlus/Splitter/hashXYZ/mix_stafford13/nextDouble（f32 语义）/nextInt(bound)。
- **md5.rs**：RFC1321（+big-endian lo/hi——create_xoroshiro_seed_str）。
- **density.rs**：**enum DensityFunction**（Constant/Noise/LinearOp/BinaryOp/UnaryOp/Clamp/Spline(SplineData)/Interpolated(InterpolatedData)/Cache2D/FlatCache——sample/min/max；Spline Hermite 递归；Interpolated grid 缓存；Cache2D 16 槽 LRU；FlatCache 5x5）。
- **json.rs**：手写 JSON parser。
- **data-driven 链验证**：`buildnode_test.rs`（JSON→buildNode→DF→sample——noise 逐位对齐）跑通。

## 当前未编译（density_builder.rs——DensityBuilder 完整版，3 处设计/编译错误）
1. `DensityBuilder::new(seed, md:&[minY;0])`——**签名笔误**（应 `new(seed, min_y, noise_height)`，`new2` 已存在）。
2. `build_node(&self)` 里 `unsafe self_mut`——**&self 里 mut 调 get_noise_sampler**——设计问题（应 `&mut self`，或 noise_samplers 用 `RefCell`/内部可变性）。
3. `resolve_ref`/lazy_refs——**POC 占位**（返回 Constant 0；真实 lazyRef + external_loader + parseFile 懒加载未实现）。

## 下一 session 接手（连续完整件，别一小点）
1. 修 density_builder.rs（3 错误）：&mut self + resolve_ref/lazyRef + parseFile（JSON→DF 懒加载）+ register_function（dfFiles 预注册——overworld 15 文件）。
2. 补全 build_node type（shifted_noise/range_choice/blend_*/old_blended_noise/weird_scaled/y_clamped_gradient/shift_a/b）。
3. mn/mx 精确（noise 真实范围 + BinaryOp 符号计算——C++ create）。
4. noise_params.json 读取（worldgen_api buildNoiseParams L142-158）。
5. 生产化：Rc<RefCell> → Arc + thread_local（多线程缓存——Rust 并发安全）。
6. 对齐测试：Rust DF vs Java 参照（真实世界 seed）。

## 关键（rust-porting-notes.md）
逆向对齐 Rust 移植：**逐位复刻 C++ 语义（含精度损失），用 f32（float）不用 f64，wrapping_*（溢出），as（截断），>>（算术）**——`next_double` float 是实测案例。

## 参照/探针
- noise_check_cpp.cpp（C++ noise 参照，已对齐 Java）/ mlp_probe.rs（Rust 软流）。
- buildnode_test.rs / noise_check.rs（Rust 探针——修好 density_builder.rs 后跑）。

## 2026-08-24 晚段（density_builder 完成 + C++ 对齐）

> 状态：✅ density_builder.rs 编译通过 + **16 个 overworld 密度函数**（10 顶层 + 6 caves）与 C++ 参照**数值逐位一致**。本段覆盖前文「下一 session 接手」1–3 项。

**已完成且编译通过**：
- **density_builder.rs**（DensityBuilder）：
  - `build_node` 全 overworld 分派补全（shifted_noise/range_choice/blend_*/old_blended_noise/weird_scaled/y_clamped_gradient/shift_a/shift_b/cache_once+cache_all_in_cell/clamp）；
  - 数字/字符串裸节点；
  - `mn`/`mx` 符号精确（noise 真实范围 + BinaryOp 符号计算 + 常量折叠→LinearOp）；
  - 真 `lazyRef`/`register_function`/`parse_file`/`set_external_loader`（JSON→DF 懒加载 + dfFiles 预注册）；
  - `resolve_ref` 特殊键 shift_x/shift_z/y/zero。
- **density.rs**：新增 10 个 DensityFunction 变体 + `InterpolatedNoiseData`。
- **noise.rs**：新增 `new_legacy`/`range_closed_amplitudes`/`get_octave`/`method_40556`（OctavePerlinNoiseSampler legacy 构造）+ `build_noise_params` 补全为 BuiltinNoiseParameters 全表。

**验证**：
- `overworld_probe`（Rust）vs `rust_ref_check`（C++ 参照）——16 个 overworld 密度函数 × 3 点 + min/max，**数值逐位一致（exact match）**。
- 测试 seed：`8576294172403134396`。

**关键 bugs（均已在对拍中抓到并修复）**：
1. ABS/SQUARE 的 mn 边界错（`max(0,imin)` vs `|imin|`）——`abs(ridges)` 范围曾塌缩成常量（5.7143）、`ridges_folded` min==max=-14.1429；已改为 `imin.max(0.0)`。
2. `minecraft:clamp` 读字段 `"input"` 而非 `"argument"`（C++ buildObject L92 用 arg("input")）。
3. `InterpolatedDF` min/max 应**委托 arg**（`arg->minValue()/maxValue()`，C++ L560-561），非 `-arg.max()`。
见 rust-porting-notes.md「UnaryOperation ABS/SQUARE 的 mn 边界」。

**新探针**：
- Rust：`WorldgenRust/src/bin/overworld_probe.rs`。
- C++ 参照：`versions/1.20.1/cpp/worldgen/src/rust_ref_check.cpp`。

**下一批（未做）**：
1. full finalDensity（noise_router）端到端；
2. 与 Java vanilla blocks 对齐；
3. 生产化（`Rc<RefCell>` → `Arc`/`thread_local`）；
4. `noise_params.json` 读取。

## 2026-08-24 深夜（full finalDensity 端到端）

> 状态：✅ Rust 能构建**完整 overworld finalDensity（noise_router）**并与 C++ 参照**数值逐位一致（final_density 规范化差分=0）**。本段覆盖前文「下一批（未做）」第 1 项（full finalDensity 端到端）。

**已完成且编译通过**：
- **finaldensity_probe.rs**（`WorldgenRust/src/bin/`）：纯 Rust 读 `noise_settings/overworld.json` → `noise_router.final_density` → `build_node` 构建整棵最终密度树 → 采样（10 点）+ min/max；`external_loader` 从 `density_function/overworld/` 懒加载被引用文件（caves/overworld refs）。
- **rust_ref_check.cpp**（C++ 参照）扩展：在既有 16 函数对拍之后追加一段，`parseFile` 构建 overworld.json 的 `noise_router.final_density` 整树并同点采样。
- 冻结快照 `.investigations/rust-density-builder/{rust_fd_out.txt, cpp_fd_out.txt, finaldensity_probe.rs}`（SHA256 见 verification-record.md v1.1 表）。

**验证**：
- Rust `finaldensity_probe` vs C++ `rust_ref_check` 的 final_density 段落——10 点 + min/max **数值逐位一致**（规范化差分=0）。
- 实测：`final_density min=-0.45833333 max=0.45833333`；`(0,0,0)=0.06590008`、`(8,64,8)=-0.25433936`、`(200,40,200)=-0.00016419`（`rust_fd_out.txt` 与 `cpp_fd_out.txt` 第 17 行逐字一致，Rust 多冒号标签 + 一行 summary，非数值差）。
- 测试 seed：`8576294172403134396`（沿用上段）。

**关键点**：
- **Rust buildNode 不再限于单密度函数**——能自底向上复刻整棵 overworld 最终密度树（`noise_router.final_density` 的 big `min(squeeze(mul(0.64, interpolated(blend_density(...)))), caves/noodle)` 链 + `overworld/*` 与 `caves/*` 引用 + lazyRef 懒加载贯通）。这是「**Rust 能替代 production density**」的核心能力里程碑。
- 若干采样点恒 `-0.45833333`（如 (4,120,4)/(72,240,72)/(-200,96,96)/(0,200,-16)，不同 chunk/coords）——为 **vanilla 真值**（final_density clamp 下界），**C++ 参照同样输出该值，互证 vanilla**；曾一度被疑为 bug，经 C++ 对拍排除（❌ 排除记录：非 Rust 插值/range_choice bug，是 min(...) 下界在该区段的真值）。

**新探针**：
- Rust：`WorldgenRust/src/bin/finaldensity_probe.rs`。
- C++ 参照：`versions/1.20.1/cpp/worldgen/src/rust_ref_check.cpp`（final_density 段落）。

**下一批（未做）**：
1. vanilla 逐块对齐（`block_probe`/ref density，Rust vs vanilla，非 C++ 自洽对拍）；
2. 生产化（`Rc<RefCell>` → `Arc`/`thread_local`，多线程缓存并发安全）；
3. `noise_params.json` 读取（对齐基准从硬编码表切到文件）。

## 相关文档
rust-rewrite-decision.md / rust-rewrite-plan.md / rust-porting-notes.md / rust-mlp-validation.md / rust-install-guide.md。

## 2026-08-24 深夜2（noise_params.json 读取，judge P2-e 收口）
- ✅ `build_noise_params_from_file(path)`：读权威 `versions/1.20.1/data/noise_params.json`（BuiltinNoiseParameters 1.20.1 导出）构建噪声参数表；`DensityBuilder::load_noise_params_file(path)` 覆盖硬编码表。
- 验证：finaldensity_probe 用文件加载噪声参数后，finalDensity 输出与 C++ 参照**仍数值逐位一致**（硬编码表与权威文件等价，转录风险消除）。
- 意义：对齐基准从硬编码表切到权威 `noise_params.json`（judge P2-e）——两端不再共享"可能写错"的硬编码表。
