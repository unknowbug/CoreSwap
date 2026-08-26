# core.judge 审查意见 — rust-density-builder 对齐

> 审查对象：`E:\PYTHON\CoreSwap\.artifacts\rust-density-builder\rust-density-builder-alignment.md`（candidate，index.yaml 首条 `re-code:rust-density-builder:alignment`）
> 日期：2026-08-26 | 审查人：core.judge（subagent，只出意见，不改 status）
> 三源核对：① 产物快照（.artifacts 对齐 md + index.yaml）② git HEAD/工作区 diff（WorldgenRust）③ 验证记录（rust_out.txt / cpp_out.txt）

## 结论

**有条件通过（conditionally pass）**——推荐维持 candidate，禁止在未解决 P1-①/P1-② 前升 confirmed。

核对三项均成立到「当前稳定快照」程度：
- 数值差分 = 0（我以 `[System.IO.File]::ReadAllLines` + `Get-Content` 双法、单次读取现文件，跑了逐行/逐列正则 diff：**16 条函数行全部 OK**，含 10 个 overworld 函数与 6 个 caves/* 函数，含 `caves/noodle min=-0.1583`，两端一致）。
- Rust 仅比 C++ 多一行末尾的 `sloped_cheese min=-114.516889 max=250.071120` summary（非数值差），与产物所述一致。
- ABS/SQUARE mn bug 修复与 C++ 参照**静态逐行吻合**（见 P1 下方证据）。

但产物声称的「逐位对齐」超出取样证据强度，且整个验证是 Rust-vs-C++（两套自洽复现），非 Rust-vs-vanilla；git diff 基线缺失、验证输出非冻结态。故结论是有条件通过。

---

## 一、先独立验证的核心事实（支撑上面三源核对）

1. **数值差分**：`E:\PYTHON\CoreSwap\versions\1.20.1\cpp\build-msvc\bin\rust_out.txt`（17 行）vs `cpp_out.txt`（16 行）。现行文件：
   - rust_out.txt LastWrite=2026-08-26 16:22:59.674（md5 29976716B504B8AA99D6ED355CEA4203）
   - cpp_out.txt LastWrite=2026-08-26 16:22:29.541（md5 440A2512AF1B4A63541CB1F087FEDB0D）
   - 逐列 diff：**全部 16 行 OK，TOTAL MISMATCH = 0**。含 `ridges_folded min=-14.1429 max=1.0000`（两端一致）、`caves/noodle min=-0.1583`（两端一致）。
2. **C++ 参照 UnaryOperation::create**：`E:\PYTHON\CoreSwap\versions\1.20.1\cpp\worldgen\src\density.h` L181-189：
   ```
   181: static DF create(UnaryOp op, DF input) {
   182:   double imin = input->minValue(), imax = input->maxValue();
   183:   double mn = applyUnary(op, imin), mx = applyUnary(op, imax);
   184:   if (op == UnaryOp::ABS || op == UnaryOp::SQUARE) {   // 184
   185:     mn = std::max(0.0, imin);                          // 185
   186:     mx = std::max(applyUnary(op, imin), applyUnary(op, imax));  // 186
   187:   }
   188:   if (mn > mx) std::swap(mn, mx);                      // 188
   189:   return std::make_shared<UnaryOperation>(op, input, mn, mx);
   ```
   产物引用「C++ create L184-188」准确。Rust `WorldgenRust\src\density_builder.rs` `un()` L209-219：`mn = imin.max(0.0)`（= `max(0.0, imin)`）、`mx = apply_unary(op, imin).max(apply_unary(op, imax))`、`if mn>mx swap`——**与 C++ 完全一致**。claim「应为 max(0.0, imin) 而非 |imin|」正确。
3. **Rust density_builder.rs 当前内容**（非 diff，直接读）：`build_node` 分派已含 shifted_noise / shift_a/b/shift / range_choice / y_clamped_gradient / weird_scaled_sampler / blend_alpha/offset/density / cache_once+cache_all_in_cell / old_blended_noise；mn/mx 用 `input.min_value()`/`get_max_value()`；`resolve_ref` 真实化（registry + shift_x/shift_y/zero + overworld/<name> 惰性 Lazy 占位 + external_loader）；`build_node`/`build_spline` 为 `&mut self`；**无任何 `unsafe` 残留**。
4. **factor=3.95000005 解释**：`...\density_function\overworld\factor.json` 外层 spline 首点 `location=-0.19, value=3.95`（derivative=0，左端平段）。三个采样点 continents = -0.65267760 / -0.62737422 / -0.46576604，**全部 < -0.19**，故 spline 恒取 3.95；顶层 `add(10, mul(blend_alpha, add(-10, spline)))`，blend_alpha=1.0 → 3.95。产物判断「非 bug、为左端平段」正确（与 C++ rust_ref_check 一致，且与既有 ref_probe factor=3.950000048 一致）。**但**该判断同时意味着这三个点根本没测到 factor 的嵌套 erosion/ridges spline 段（location -0.15..0.06），故是「证据不足以支撑全域逐位」而非 bug——见 P1-①。

---

## 二、P1（阻 confirmed，必须处置）

### P1-①  claim「逐位对齐」超出取样证据强度
- **问题**：「对 10 个函数在 3 点 + min/max 上逐位对齐」不足以证明「buildNode 已逐位对齐」。每函数仅 3 个点，而这些函数是连续 Perlin 噪声 + 多层嵌套 spline（factor.json 890 行，含十余个 knot、多处嵌套 ridges/ridges_folded），其值域是连续流形。
- **证据/理由**：① 3 点/函数对连续噪声函数是极稀疏采样；② 多个函数在选定点**退化**：factor 三点全 = 3.95000005（同一平段）；jaggedness 三点全 = 0.00000000；offset 三点两值相同。即「3 点」对 factor/jaggedness 实际只提供了一个非零/非平段的约束，未触及 spline 拐点区、未触及噪声的动态中间区；③ min/max 多数是解析界（如 base_3d_noise ±87.5515、factor 0.625..6.300），只验证端点层折叠逻辑，不验证中间采样路径。
- **建议处置**：二选一——
  (a) 扩充采样：随机/网格 >100 点/函数（覆盖 continents 在 [-0.19, 0.06+]，erosion 在 [-0.6, 0.45]，ridges/ridges_folded 各 knot，含负/大坐标、y 在 [-64, 320]），跑 diff 仍 = 0 后才可称「逐位对齐」；或
  (b) 降级表述为「在 10 个 overworld 函数的 3 个采样点 + min/max 端点、8 位小数上，Rust buildNode 与 C++ 参照一致」——这是当前证据可支撑的最强说法。

### P1-②  验证是 Rust-vs-C++（两套自洽复现），非 Rust-vs-vanilla
- **问题**：对照物是 C++ `density_builder.h` 这一**独立复现**，不是 MC vanilla。两者读同一批 JSON、同一噪声参数子表、同一 seed、同一坐标。若 Rust 与 C++ 采有**共同错误**（同一 JSON 语义解读、同一 octave 消费顺序、同一硬编码噪声参数表），diff 永远 = 0，但两者都偏离 vanilla。
- **证据/理由**：产物自身在「域/边界」列出「未做：full finalDensity（noise_router）端到端、与 Java vanilla blocks 逐块对齐」。故当前验证只证 Rust==C++ buildNode，**不证 Rust==vanilla**。产物 headline/结论用词「逐位对齐」需被读作「逐位对齐 C++ buildNode」，否则易被误读为「worldgen 已对齐 vanilla」。
- **建议处置**：把「对齐基准 = C++ density_builder.h」写进结论显式限定；明确本交付物在 Rust-worldgen 对齐 vanilla 链路中只占「buildNode 与 C++ 参照一致」这一步；vanilla 端到端/逐块对齐属后续阶段（产物已声明，保留即可）。

---

## 三、P2（不阻 candidate，但影响严谨与可信，建议处理）

### P2-①  三源核对第 ② 源（git diff）不可用 —— WorldgenRust 全树 untracked
- **问题**：`git -C E:\PYTHON\CoreSwap ls-files WorldgenRust` 为空，`git status` 显示 `WorldgenRust/` 整体为 **untracked**（无任何已跟踪基线）。因此产物声称「对 density_builder.rs/density.rs/noise.rs 检查改动（git diff）」在仓库里**无 diff 可查**——不存在 HEAD 基线。
- **证据/理由**：`git ls-files WorldgenRust` 返回空；`git diff --stat WorldgenRust` 空。无从验证「原 3 编译错误 + &self 内改 mut + 分派不全 + mn/mx 占位 + resolve_ref 占位”这些『修复前的缺陷』确曾存在并被修复——只能审当前文件终态。我已直接审 density_builder.rs 终态（分派全/&mut self/无 unsafe），与产物所述完成态一致，但这不构成「改动 diff」意义上的核对。
- **建议处置**：① 提交一个基线（或整树）到 git，使 diff 维度可查；或 ② 明确声明 WorldgenRust 为全新/untracked 树，本核对退化为「整文件终态审"，并如实标注。注意：CoreSwap 主工作区有提交纪律（git 强制），WorldgenRust 长期 untracked 本身也是该交付物未入版本库的问题。

### P2-②  验证输出非冻结态（评审期间被再生成；观测到一次瞬态数值分叉）
- **问题**：评审期间我反复读证文件得到**不一致内容**——read 工具首次快照显示 10 行(仅 10 个 overworld)、随后 `Get-Content` 显示 10/16、再后 `ReadAllLines`+`Get-Content` 均显示 17/16；并一度观测 `caves/noodle min` **rust=-0.0083 vs cpp=-0.1583** 分叉，最终稳定在两端 -0.1583（diff=0）。且两文件 LastWrite 不同（rust 16:22:59.674 vs cpp 16:22:29.541）。
- **证据/理由**：证文件是**运行再生成产物、非不可变快照**；若评审/复验发生在不同再生成瞬间，diff 可能不为 0。当前稳定快照 diff=0，但该「0」只对当前字节串成立，未冻结。
- **建议处置**：把 rust_out.txt / cpp_out.txt + 生成命令（`cargo run --bin overworld_probe` vs `rust_ref_check.exe`）+ 探针源哈希 + 二进制哈希，**复制固化**到 `.investigations/rust-density-builder/`（如 `cmd-output/` + `regression-record.md`），使「diff=0」可复现、可回溯。同时确认无后台进程再改动这些文件（我 job_list 无任务，但可能属发起方 agent 的构建循环）。

### P2-③  noise_params 表：硬编码 vs noise_params.json
- **问题**：Rust `build_noise_params()` 与 C++ `rust_ref_check.cpp` `buildNoiseParams()` 均为**硬编码完整内置表**（我已对比二者——共享键逐值一致，且当前 C++ 表已扩到全表，与 Rust 对齐）。两者都**不读** `minecraft:noise_params.json`（产物已在域/边界注明「noise_params.json 文件读取后续」）。
- **证据/理由**：硬编码表若任一键值有误，两实现共同继承 → diff 仍 = 0，但偏离 vanilla。
- **建议处置**：将该硬编码表与真实 `noise_params.json` 做一次逐键核对（或下一步改为读文件）；把「当前用硬编码表」写入产物边界，免得把它当 vanilla 权威。

---

## 四、P3（文档/细节，不阻通过）

### P3-①  覆盖范围与产物文本不符（保守方向）
- **问题**：现行探针源 `overworld_probe.rs`（36 行）与 `rust_ref_check.cpp`（115 行）均迭代 **16 个函数**（10 overworld + 6 caves/*），且 diff=0 对这 16 个均成立（含 `caves/entrances/noodle/pillars/spaghetti_2d/spaghetti_2d_thickness_modulator/spaghetti_roughness_function`）。产物只声称/描述 10 个 overworld，域/边界写「仅验证 density == C++ buildNode（overworld）」。
- **证据/理由**：当前 diff 行包括 L11-L16 六个 caves/* 行，全部 OK。
- **建议处置**：在产物里补一句「现行探针已延展到 16 个函数（含 6 个 overworld/caves/*），diff 仍 = 0」；caves/* 虽未在「10 个」命名列表，但已实测对齐，值得补记（避免文档滞后于探针）。

### P3-②  @anchor.test 标注与验证分层
- **问题**：验证为真运行时证据（真实 exe 产出真实输出），故 candidate 合法；但产物未标注 Rust 侧是否带 `@anchor.test(source=...)`（本任务为 Rust 文件，Anchorlaw v0.17 声明 Rust 不在支持语言，参考 Rust 侧 anchor 标注暂缺属预期）。
- **证据/理由**：产物未列 `@anchor.test` 条目。
- **建议处置**：既然 Rust 协议外，无需 Rust 侧 anchor；但建议在产物「验证方式」里明确本交付物属哪个分层（Full/Partial/Degraded），并标注实际执行者与执行环境。当前未声明分层。

---

## 五、审查意见（只意见，不改 status）

- **推荐状态**：**维持 candidate**（不升 confirmed；`confirmed` 只能是用户拍板）。
  理由：有真运行时证据（16 函数 diff=0）+ 代码终态与产物一致 + ABS/SQUARE mn 修复与 C++ 静态逐行吻合，candidate 资格成立。
- **到 confirmed 前的必要条件**：① P1-① 二选一（扩充到 >100 点/函数网格，或把 claim 降级为「在选定采样点+min/max 上对齐」）；② P1-② 在结论显式限定「对齐基准 = C++ buildNode」（vanilla 对齐属后续）；③ P2-① WorldgenRust 入 git（或声明 untracked+整文件审）；④ P2-② 固化证文件为不可变快照 + 命令/哈希日志。
- **我认同的判断**：factor=3.95000005 为 factor 外层 spline 左端平段（location=-0.19, value=3.95，三点 continents 均 < -0.19）→ **非 bug**；该判断证据充分（factor.json L23-24 + 三点 continents 值 + C++ 同值）。mn/mx 对齐充分（ridges_folded min=-14.1429 max=1.0000 两端一致，证实 ABS-range 修复生效；16 函数 min/max 列全对齐）。
- **待用户确认的风险点**：① 3 点采样对 factor/jaggedness 实际近乎退化（覆盖不足）；② Rust-vs-C++ 非 Rust-vs-vanilla；③ WorldgenRust 未入 git；④ 硬编码 noise_params 表未对噪声参数文件。

> 说明：本次审查**未改动**任何状态；`.artifacts/index.yaml` 的 `re-code:rust-density-builder:alignment` 仍为 candidate。
