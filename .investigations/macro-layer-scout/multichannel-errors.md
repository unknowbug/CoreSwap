# Rust 宏观采样重构 + 性能定位：错误与根因清单（重点记录）

> 载体：`.investigations/macro-layer-scout/multichannel-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「Rust 宏观采样层重构 + 性能定位」里程碑（2026-08-30）中定位并处理/发现错误的条目。本 session 共 2 类（M1/M2）。
> 背景：本里程碑（multi-channel 竖切重构 + 顶层确认 + 性能定位，judge 审计为 candidate）中，judge 三源核对发现 2 处需修正的错误/隐患——① ShiftDF Cache2D 对 plain Shift 强置 y=0 偏离参考（潜在正确性 bug，已被保守修正）；② noise AVX 归因不实（`sample_section_avx` 死代码 + 非真 SIMD，把编译器 auto-vec 当手工 SIMD 功劳）。
> 结论性架构见 07 主题篇追加小节（draft-07）；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。数字来自 `cmd-output/` 实测记录 + judge 审计意见。

---

## M1. ShiftDF Cache2D 缓存把 plain `Shift` mode 强置 y=0，偏离 C++/Java 参考（实际 y 语义）

### 现象
- 为加速 ch#0 corners 采样（3.60ms 大头）实现 ShiftDF Cache2D（xz-keyed 缓存；`shift_y_independent` 曾判「708 个 ShiftDF 全 y 独立」）。
- **judge 三源核对**（`.investigations/macro-layer-scout/review-audit-conclusion-chain.md` ④）发现**代码层与参考层不一致**：
  - **代码层**（`density.rs` L488-508）：缓存后 `Shift` 与 `ShiftA` 都落入 `_ => (pos.x, pos.z, x, 0.0, z)` 分支 → **`Shift` mode 被强制 y=0**。
  - **参考层**（C++ `density.h` L247-257 + 缓存前 Rust）：`Mode::SHIFT` 用**实际 y**（`case Mode::SHIFT: break`，y=pos.y）；仅 `SHIFT_A` 置 y=0，`SHIFT_B` 做 (z,x,0) 交换。
- 即：缓存改动把 Shift mode 从「用实际 y」改成「y=0」，**与 Java/C++ 参考语义不符**。

### 根因（机制）
- **缓存实现把「ShiftA/ShiftB 的 y 独立性」错误泛化到 plain `Shift` mode**：为统一写 cache getter，把 `Shift` 也归到 `_ => (x, 0, z)` 分支，**丢失了 shift offset noise 依赖实际 y 的语义**。
- 更深一层：**「探针实测某些 mode y 独立」≠「该 cache 对所有 mode 的 y 强制合理」**。探针能证明的是「被测 mode 在实测坐标域 y 无关」，不能推出「缓存把未测 mode 强置 y=0 仍与参考一致」——语义依赖 mode 定义，cache 强制不能替代 mode 语义。

### 定位（诊断方法）
- **judge 基准三源核对**（不是运行时崩溃，是静态语义核对）：
  1. reader 读缓存分支 `_ => (pos.x, pos.z, x, 0.0, z)` → 见 `Shift` 落入 y=0 分支。
  2. 对照 C++ `density.h` `case Mode::SHIFT: break`（保留实际 y）→ 见分歧。
  3. reader 扫 continents/erosion/ridges JSON 确认 overworld 只用 `shift_x`/`shift_z`（→ ShiftA/B），**无 plain `minecraft:shift`** → 当前没爆是「恰好没用 plain Shift」，非语义正确。
- 关键判据：**语义分歧不一定立即爆（当前 overworld 无 plain Shift），但它是潜在正确性隐患**——不是「没报错=对」。

### 修复
- **保守修法**（已做，见 `shift_y_confirmed.txt`）：
  - `ShiftA`/`ShiftB` 缓存安全（y 独立性由 mode 语义**构造性保证**：ShiftA 本就 y=0、ShiftB 本就 z=0）——保留缓存。
  - **plain `Shift` 改为不缓存、用实际 y**（保持参考语义），避免 y=0 偏离。
- 补测：`shift_y_dependence` 增强版覆盖 ch#0 **全部 708 个 ShiftDF**（非仅前 5）+ 含**负 Y（-64..320）** + 每节点 4 列 (4,4)/(8,8)/(12,4)/(0,0) → **708/708 全部 y 独立**（mode 分布仅 ShiftA+ShiftB，**确认 overworld 无 plain Shift**，验证 judge 判断）。
- features_probe 对齐 **95.40% 保持**（当前 overworld 种子无回归）。

### 教训（可复用判错经验）
- **「实测若干 mode 性质」≠「同类缓存对所有 mode 适用」**：缓存/泛化分派时，**逐个核对 mode 定义**（本案例 Shift 用实际 y vs ShiftA y=0 vs ShiftB z=0），别把「某 subset 的性质」当成「cache 对全集的强制」——尤其当探针只测子集 + 单列 + 不含负 Y 时。
- **语义分歧用静态三源核对（代码 vs 参考 vs 数据）而非等运行时报错**：mode 语义偏离参考可能「恰好当前 config 不用该 mode 而不爆」，**不爆 ≠ 正确**。给 cache/mode 分派加注释声明「仅验证 ShiftA/B，Shift 语义需复核」。
- **探针证据强度 vs 表述强度匹配**：写「708 全 y 独立」前确认探针真的覆盖 708 全集（早前只测前 5 + 单列 + 无负 Y 就标「708」，overclaim，judge 抓出）。**补测覆盖负 Y + 多列 + 全集是判定「真 y 无关」的必要条件**（offset noise 可能 y 频率 0 / y 用 0 导致表面无关，需多坐标验证）。

---

## M2. noise AVX 归因不实：`sample_section_avx` 死代码 + 非真 SIMD，把编译器 auto-vec 当手工 AVX 功劳

### 现象
- `noise_avx_eval.txt` / commit 描述宣称「AVX __m256d 手工实现：Perlin 26.56→19.55ns (快 1.36x)，features_probe 95.40% 不变，全管线 -1%」。
- **judge 三源核对**（review-audit-conclusion-chain.md ⑤⑥）发现 `sample_section_avx` **从未被调用**且**函数体不是 SIMD**：
  - `noise.rs` L48-96 `sample_section_avx`：**grep 全库仅定义处命中，生产 `sample()`/`sample_section()` 从未调它**；注释 L46 自认「生产仍走标量 sample_section」。
  - **函数体非 SIMD**：L72-76 创建 `_mm256_set1_pd` 但注释 L73 明说反了并用 `_ = vx` 丢弃；L77 注释「先用标量 dot 跑通流程」；L79-92 全是**标量 `dot3`/`lerp`/`perlin_fade`**，与 `sample_section` 相同。

### 根因（机制）
- **「AVX 框架已加（sample_section_avx 门控）」的表述不成立**：这个函数是**未接线的占位/半成品**（建了 intrinsic 变量但没用于 dot，注释自认标量跑通），不是真实 SIMD 路径。
- **测量归因张冠李戴**：26.56→19.55ns (1.36x) 是 **`bench_noise.rs`（标量 `sample()`）在 `-C target-feature=+avx` 下被编译器自动向量化**的微基准改善，**与 `sample_section_avx` 函数无关**。把「编译器 auto-vec 的收益」写成「手工 AVX 实现的收益」= 归因错误。
- **「95.40% 不变是平凡结论」**：AVX 路径没接线，生产仍走标量 → 对齐当然不变，**不能作为「AVX 正确」的证据**（死代码的"零回归"是空洞的）。

### 定位（诊断方法）
- **代码层**：`grep sample_section_avx` 全库 → 仅定义处命中（无调用点）→ 判死代码；读函数体 → 见标量 dot/注释自认 → 判非 SIMD。
- **测量层**：对 `bench_noise.rs`（标量 `sample()`）+ `RUSTFLAGS='-C target-feature=+avx'` 复测 → 26.56→19.55ns 是 auto-vec 微基准，非手工 AVX 路径。
- **判别关键**：**「编译参数（target-feature=+avx）引起的快」与「某函数实现引起的快」不是一回事**——前者可能是编译器对现有标量代码的自动向量化，后者才证明该手工路径有效。**归因时要核实「宣称优化的函数是否真的被调用/真的实现 SIMD」**。

### 修复
- **文案归因修正（必做，judge 建议）**：把「noise AVX 手工实现 1.36x」改为「**编译 target-feature=+avx 下 bench_noise 微基准快 1.36x（编译器 auto-vec）**；手工 `sample_section_avx` 为**未接线死代码且未实现 SIMD dot**」。避免误导后续「AVX 已实现」。
- **决策保留**：全管线 -1%（45.47→45.01ms，400 chunks）方向可靠（编译器层面已代表 AVX 收益上限），「noise 非全管线瓶颈 / aquifer 才是」判断**不因归因修正改变**。
- **若真要 AVX**：真正实现 `sample_section_avx` 的 `__m256d` dot（而非标量占位），按 env 门控 chunk 级一次判断接线到 `sample()`/`sample_section()` 热路径，再复测全管线。

### 教训（可复用判错经验）
- **测量归因要核实「实际执行路径」，不只微基准数字**：宣称某优化带来加速前，**三查——① 该函数是否被调用（死代码检测）；② 函数体是否真的实现该优化（intrinsic 是否用于计算，还是占位/丢弃）；③ 数字是「编译器层面（编译参数）变化」还是「函数实现变化」**。微基准快 ≠ 目标路径快，更 ≠ 目标路径真的优化了。
- **「对齐率不变」作为优化正确性证据是陷阱**：当优化路径没接线（死代码），「不变」是平凡的；只有「接线后仍不变」才能证明优化本身无损。
- **死代码/未接线检测方法**：`grep <函数名>` 全库看调用点，无调用 = 死代码；函数体内 `_ = intrinsic` 丢弃 / 注释自认标量跑通 = 半成品占位。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| judge 三源核对发现 ShiftDF Cache2D 把 plain `Shift` 强置 y=0，偏离 C++/Java（实际 y）；当前 overworld 无 plain Shift 所以没爆（M1） | cache getter 统一 `_ => (x,0,z)` 分支把 Shift 也强置 y=0，**丢失 shift offset noise 依赖实际 y 的语义**；「实测 ShiftA/B y 无关」被错误泛化成「cache 对 Shift 全 mode y 强制合理」 | **语义分歧不一定立即爆**（恰好 config 不用该 mode）——不爆≠正确。缓存/泛化分派**逐个对 mode 定义**（Shift=实际y / ShiftA=y=0 / ShiftB=z=0），别把 subset 性质当全集。证据强度匹配表述：写「708 全 y 独立」前确认探针真覆盖 708 全集 + 负 Y + 多列 |
| 宣称「AVX __m256d 手工实现 Perlin 1.36x；95.40% 不变证明 AVX 正确」被 judge 揭穿（M2） | `sample_section_avx` 是**未接线死代码**（grep 无调用）+**非真 SIMD**（标量 dot/占位丢弃 intrinsic）；1.36x 是 `bench_noise` 标量代码在 `-C target-feature=+avx` 下**编译器 auto-vec**，归因张冠李戴；95.40% 不变是死代码导致平凡 | **测量归因要核实实际执行路径**三查：①函数是否被调用（死代码）②函数体是否真实现优化（intrinsic 用于计算）③数字是编译参数(auto-vec)还是函数实现变化。死代码的「零回归」不能证明优化正确——只有接线后仍不变才行 |

> [DRAFT — knowledge subagent 产出，待主会话应用。] 主会话应用：保留本文件（错误台账独立成篇，符合 SUBAGENT-KNOWLEDGE-GUIDE §三），末尾速查表已含 M1/M2 各一行。
