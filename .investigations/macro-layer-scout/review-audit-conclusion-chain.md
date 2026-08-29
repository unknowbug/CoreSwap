# judge 审查意见：Rust 宏观采样重构 + 性能定位结论链审计

> 角色：core.judge（subagent，隔离子进程）。**只出审查意见，不改任何 status。** confirmed 由人类授予。
> 审查对象：本 session「Rust 宏观采样重构 + 性能定位」结论链（结论链 ①-⑦ 逐环核对）。
> 审查基线（三源核对）：① `.investigations/macro-layer-scout/` + `rust-mod-load/cmd-output/` + `perf-e2e/` 产物；② git HEAD=`e8fe8f2` + 工作区 diff；③ 原始探针代码 + cmd-output 数字。
> 审查标准：core.judge 清单（证据完整性/落盘/三源核对/置信度合法/产物契约/噪声卡/retry cap/模块边界）。
> 日期：2026-08-30。

---

## 0. 结论摘要（一句话）

**结论链主干正确、方向合理，但有 3 处需修正/补充的环节（② 产物落盘缺位、④ ShiftDF y 独立性证据与代码语义不符、⑤ noise AVX「1.36x」归因不实且 AVX 代码为死代码未接线），以及 aquifer 主瓶颈是「宏观子集减法导出」的 Partial 快照需限定其边界。整体置信度：candidate（建议人类确认后授予），非 confirmed。**

推荐状态：**保持 draft/candidate，建议 candidate**（各环节）；**confirmed 未达**（人类拍板前不可授）。

---

## 1. 逐环核对

### ① 顶层确认（NoiseChunk::fill cell grid 是真顶层）— ✅ 可靠（reader 级）

- 证据充分且为源码直读：`macro-layer-topness.md` 完整列出调度链（pyramid → tasks → noise stage → fill_from_noise）与采样机制层（`NoiseChunk::fill`+`fill_slice_into`+trilerp+combine），6 个疑似上层（blending/StaticCache2D/Beardifier/aquifer/dim settings/ColumnCache）逐一确认不构成采样层，均给源码落点。
- `macro-layer-map.md` 补充 Interpolated「竖切 channel 语义」+ Rust 52× 雪崩根因（采样点越界→懒建缓存级联），与 `rust-mod-load/cmd-output/macro_grid_*` 记录一致。
- **核实**：`terrain.rs` L134-143 注释确认「直接对 final_density 采样 corners 会触发内部 interpolated 雪崩（52×，见 macro_grid 记录）」→ 与 scout 结论互证。
- 结论：① 可靠，reader 级。此环对后续重构方向（multi-channel 竖切对齐 Java）是正确的锚定。**建议 candidate。**

### ② multi-channel 竖切重构（final_density → 5 channels + combine）— ⚠️ 正确性可信，但「重构」尚未进生产，且产物未落盘

- **代码正确性**：`density.rs macrolize_channels/macrolize_into`（L604-681）实现完整——DFS 遍历收集所有 `Interpolated` 的 inner 为独立 channel，树内 `Interpolated→ReadChannel{ch}` 全部替换，`sample_combine` 的 `ReadChannel` 分支读 `interp[ch]`（L464）。`macrolize_probe.rs` 验证 combine 树残留 Interpolated=0、ReadChannel≥1。
- **diff 0 的可信度**：`macro_sampler_probe.rs` L100-109 对比 n=54 点（x,z∈{4,8,12}，y∈{4,64,128,200,260,300}），平均差异 0.000000。**采样点仅 54，且全落在 chunk 内部 cell 边界平面（fx/fz=0 的角点上），未覆盖：(a) cell 内部任意点（0<fx,fz<1）；(b) chunk 边界/clamp 区域；(c) 负 Y 区间（-64..4 未测）；(d) 跨 cell 的插值路径。** diff 0 是**局部充分非全局证明**——在「已测子空间内正确」成立，但作为「multi-channel 语义正确」的完整证明不充分。
- **关键**：`DensityMacroSampler` **只在探针文件** `macro_sampler_probe.rs` 内定义；生产 `terrain.rs fill_chunk` **未接线**——默认仍是逐点 `dense.sample()`（L152），`MacroGrid`（对整树 corners 采样的错误做法）仅在 `WG_MACROGRID` env 时启用。**即「multi-channel 竖切重构」目前是探针层验证，未成为生产 fill 路径。** 结论链第 2 环表述「combine 树全部替换」容易误读为已进生产，需澄清为「重构函数/探针层完成，生产接线未做」。
- **产物契约违规**：`.artifacts/index.yaml` **无任何 macro-layer-scout / multi-channel / ShiftDF / noise 条目**；多结论仅在 `.investigations/cmd-output`（探针临时产物级，主会话可写）与 git commit message 中。**缺 index.yaml 落盘 = 产物契约未满足**（core.artifact / judge 清单第 5 条）。
- 结论：② 正确性 diff0 可信（在所测子空间），但「重构完成」表述需限定为探针层；产物未落盘契约。**建议 candidate（附限定），落盘缺失驳回。**
- **三源差异源**：工作区 `git status` 显示 `macrolize_probe.rs` 有 7 行未提交 diff（channels 纯性检查打印）+ `corner_sampling_breakdown.rs` 为 untracked——**部分探针产物未提交**，三源（产物/HEAD/工作区）不完全一致。建议先提交。

### ③ 「ch#0 corners 89% noise」与「全管线 aquifer 大头」是否矛盾 — ⚠️ 不矛盾，但二者是不同测量域，需明确解释

- **不矛盾，成立**，路径不同：
  - **corners 采样**（`tree_vs_noise_breakdown`/`corner_sampling_breakdown`）：测的是 **ch#0 (BlendDensity terrain) 的 cell-corner 采样** = 仅 density 树的 noise 采样部分（每 corner 求 Inner 树，3677 节点中 noise 采样 cost 主导）。该探针用 `macrolize_channels` 的 channel 采样，非生产逐点路径。
  - **全管线 aquifer**（`macro_java_vs_rust.txt` 34.66ms + `aquifer_*` 记录）：是生产 `fill_chunk_blocks` 的 density+aquifer 宏观，其中 aquifer 逐点 apply 每 chunk 98304 次的固定开销主导（`aquifer_internal_precise.txt` L17）。
  - 二者衡量的是**不同采样阶段/不同成本构成**：ch#0 89% noise 是「宏观 corners 采样内部构成」；aquifer 大头是「宏观 density+aquifer 的跨阶段构成」。不矛盾。
- **但结论链表述有跳跃**：从「ch#0 noise 89%」跳到「全管线 aquifer 是主瓶颈」缺一个「为何 noise 优化对全管线无大收益」的显式桥梁。该桥梁在 `noise_avx_eval.txt` L8-10（「density~13ms 里 noise 部分降 1.36x 被稀释」）+ `tree_vs_noise.txt`（noise 89% 但只占 macro 的一部分）才给出。**建议主会话把「corners 采样慢 ≠ 全管线瓶颈」的推理链写入 docs，避免读者误读。**
- 结论：③ 不矛盾，成立，但需在结论 docs 里显式区分「宏观 corners 采样成本」vs「全管线阶段构成」两个测量域。

### ④ ShiftDF Cache2D 是否真 y 无关 — ⚠️ 高风险：证据与代码语义不符（当前 overworld 低风险，但为潜在正确性隐患）

**这是审查中最重要的发现。** 三源核对发现严重不一致：

- **证据层（`shift_y_dependence.rs`）**：
  - 只测**前 5 个** ShiftDF（L43 `sh.iter().take(5)`），但 `shift_y_independent.txt` 写的「708 个 ShiftDF ... y_range_impact=0.00%」把「找到 708 个节点」与「实测 5 个」混为一谈。
  - 只测**单一 (x,z)** 列（`-288*16+4, -256*16+4`），y∈[0,320] step 8（**不含负 Y -64..0 区间**）。
- **代码层（`density.rs` L488-508）**：缓存后 `Shift` 与 `ShiftA` 都落入 `_ => (pos.x, pos.z, x, 0.0, z)` 分支 → **`Shift` mode 被强制 y=0**。
- **参考层（C++ `density.h` L247-257 + 缓存前 Rust）**：`Mode::SHIFT` 用**实际 y**（`case Mode::SHIFT: break`，y=pos.y），仅 `SHIFT_A` 置 y=0，`SHIFT_B` 做 (z,x,0) 交换。

**分歧**：缓存改动把 `Shift` mode 从「用实际 y」改为「y=0」。**该 cache 对 Shift mode 语义与 C++/Java 参考不符。**

- **为何当前 overworld 没爆**：
  1. overworld 密度函数只用 `minecraft:shift_x`/`shift_z`（→ ShiftA/ShiftB），**无 plain `minecraft:shift`**（扫描 continents/erosion/ridges JSON 确认只有 shift_x/shift_z）。
  2. ShiftA/ShiftB 的 y 独立性是**由 mode 语义构造性保证**的（ShiftA 本就 y=0、ShiftB 本就 z=0）——`shift_y_dependence` 探针测出 y_independent=0 对 ShiftA/ShiftB 是**必然结果，不能证明 offset noise 本身 y 无关**。
  3. `features_probe` 95.40% 对齐在缓存前后不变（b4014d8/e8fe8f2 均称保持）→ 当前 overworld 种子无回归。
- **风险**：若未来 nether/end/自定义维度用 plain `minecraft:shift`，cache 的 y=0 会与 Java 的实际-y 语义分歧 → 潜在地形错。这是**潜在正确性隐患**，当前未触发，但证据链（5 节点/单列/仅 ShiftAB）无法支撑「Shift mode 也 y 无关」的泛化。

**审查意见**：④ 的「ShiftAB y 无关 + 当前 overworld 对齐保持」成立（低风险）；但「ShiftDF 完全 y 独立（708 节点）」的表述**过强且证据不足**，且 cache 对 plain `Shift` mode 的 y=0 强制是**代码语义偏离参考的潜在 bug**。建议：探针补测 plain Shift-mode 节点的 y 独立性（或补一个负 Y + 多点位 + 覆盖 708 全集的抽查），并为 cache 的 Shift mode 分支与参考对齐（或显式注释为「仅 ShiftA/B 适用，Shift 保留实际 y」）。

### ⑤ noise AVX「全管线 -1%」是否可靠 — ⚠️ 数字方向可靠，但「AVX 实现/1.36x」归因不实（重大偏差）

**这是第二关键发现。** 三源核对揭示 `sample_section_avx` 是**死代码且未实现 SIMD dot**：

- `noise.rs` L48-96 `sample_section_avx`：
  - **从未被调用**（`grep sample_section_avx` 全库仅噪声.rs 自身定义处命中；生产 `sample()`/`sample_section()` 从未调它）。注释 L46 自认「生产仍走标量 sample_section」。
  - **函数体不是 SIMD**：L72-76 创建 `_mm256_set1_pd` 但注释 L73 明说反了并用 `_ = vx` 丢弃；L77 注释「先用标量 dot 跑通流程」；L79-92 全部是**标量 `dot3`/`lerp`/`perlin_fade`**，与 `sample_section` 相同。
- 因此：
  - **「Perlin 26.56→19.55ns (1.36x)」不是 `sample_section_avx` 的功劳** —— 是 `bench_noise.rs`（标量 `sample()`）在 `-C target-feature=+avx` 下被**编译器自动向量化**的微基准改善。AVX **框架只是空壳，未做**。
  - **「features_probe 95.40% 不变」是平凡事实**：AVX 路径没接线，生产仍是标量 → 对齐当然不变，不能作为「AVX 正确」的证据。
- **全管线 -1% 方向可靠**：45.47→45.01 (400chunks) 与 bench_single 的噪声级数据（+avx 全编译对全管线）一致，说明「noise 非全管线瓶颈」成立。
- **影响**：结论 ⑤ 的**核心决策（noise 非优先，aquifer 才是）仍成立**，因为该决策基于「+avx 编译后全管线仅 -1%」（编译器层面已能代表 AVX 收益上限）。但 commit message / noise_avx_eval.txt 宣称的「AVX __m256d 手工实现 1.36x」**不实**，属归因错误，需修正文案。

### ⑥ 噪声 AVX 评估（Perlin 26.56→19.55ns 1.36x、全管线-1%、95.40%保持）— 同⑤：数字部分可靠，归因不实

- 同 ⑤，不重复。`noise_avx_eval.txt`/commit 描述「AVX __m256d: ... Perlin 1.36x」把「编译器 auto-vec 微基准」与「手工 AVX 路径」混淆。**95.40% 保持是（死代码导致的）平凡结论，不能用于证明 AVX 正确。**

### ⑦ aquifer 是主瓶颈（fill_chunk_blocks 45ms 的大头 = aquifer 21.5ms）— ⚠️ 方向成立，但「21.5ms」是 Partial 减法快照，需限定边界

- **真实现状**：aquifer 确实是宏观 density+aquifer 阶段的大头（`macro_java_vs_rust.txt` 34.66ms 中 aquifer 增量 ~21.5ms；`aquifer_internal_precise` 指出 apply 每点 98304 次固定开销主导）。
- **证据分层**：
  - 21.5ms = **减法导出**（34.66 macro - 13.14 density），非 aquifer 独立计时；且**宏观子集数字本身被 AGENTS.md「端到端性能对比铁律」标为可疑信号**（「宏观子集 > 完整路径」是基准不可靠信号之一）。这里宏观子集（34.66）用于隔离 macro，是**有意为之**，但需知它是 Partial 快照。
  - `aquifer_internal_precise` 自己承认「合计可解释 ~6.4ms，aquifer 总 17.5ms，剩余 ~11ms 未解释（apply 固定开销）」→ 21.5ms/17.5ms 数值在不同记录间不一致（16ch 17.5ms vs 400ch 21.5ms），是**不同测量上下文**，不可当作精确单一值。
  - 该环节的 aquifer 数字经历过多次修正（base_breakdown 被标污染 → 17.5ms → 21.5ms macro 增量），`perf-e2e-errors.md` P4 还推翻过「barrier 是 aquifer 大头」的方向。
- **决策层面合理**：全管线 Rust 45.48ms < Java FULL 55ms（Rust 反而快 ~1.2×），但宏观专项 Rust 34.66 > Java 23-25（aquifer 慢 ~1.4-1.5×）。**so「aquifer 是宏观待优化点」方向正确**；但若说「aquifer 是全管线主瓶颈」需谨慎——Rust 全管线已快于 Java，唯一明确待优化差距就在宏观 aquifer。**表述应限定为「宏观 aquifer 是相对 Java 的待优化差距」，非「全管线绝对瓶颈」。**
- **下一步 aquifer 优化合理性**：合理（相对 Java 有真差距），但需注意 aquifer 内部已多次翻案（barrier 非大头 → apply 固定开销），优化前应先用无污染计数探针（`aquifer_*_count`）锁定 apply 固定开销的真实构成，避免再走错方向。**不建议直接上「barrier Cache2D」（已被 P4 推翻）。**

---

## 2. 三源核对表（不一致项汇总）

| 环节 | .investigations 记录 | git HEAD/工作区 | cmd-output 数字 | 一致性 |
|---|---|---|---|---|
| ① 顶层 | macro-layer-topness reader 级 | 已 commit (7ee5ec4) | 无命令（纯源码） | ✅ |
| ② multi-channel | multichannel_progress/corner breakdown | macrolize_channel 已 commit；**DensityMacroSampler 仅探针文件，未进生产** | diff0 (n=54) 局部 | ⚠️ 生产未接线 + 落盘缺 + 未提交 diff |
| ③ noise89% vs aquifer | tree_vs_noise (修正里程碑) | 6755b0b already commit | 3.34/0.38/2.97ms | ✅（但两域未显式区分） |
| ④ ShiftDF | shift_y_independent "708 y独立" | **cache y=0 for Shift 偏离参考** | 探针仅测 5 节点/单列 | ⚠️ 证据与语义不符 |
| ⑤⑥ noise AVX | noise_avx_eval "1.36x AVX" | **sample_section_avx 死代码/非 SIMD** | 26.56/19.55, -1% | ⚠️ 归因不实 |
| ⑦ aquifer | perf-e2e conclusion + docs/07 | 21.5/34.66 已 commit docs | 17.5 vs 21.5 不一致 | ⚠️ Partial 快照，需限定 |

---

## 3. judge 审查清单结论

1. **证据完整性（@anchor.test source）**：本 session 探针（macro_sampler/tree_vs_noise/shift_y_dependence）**均为独立可编译 Rust 探针**，非 `@anchor.test` 标注函数；它们本身就是验证载体（probe 对比参照值）。source 语义弱（无 @anchor.test 标注），但探针代码可复现，属可接受。**未发现伪造证据。**
2. **证据落盘**：原始 cmd-output 落盘于 `.investigations/*/cmd-output/`（探针级，合规）；**但结论级落盘缺**——`docs/07` 有 aquifer 宏观结论，**multi-channel 重构进度/ShiftDF cache/noise AVX 结论链未写入 docs/07 或 10-timewise**，且 `.artifacts/index.yaml` 无条目。**证据链未完整落盘。**
3. **三源核对**：见上表。④⑤ 是三源明显不一致，⑦ 是数值限定问题。
4. **置信度合法**：产物标 draft/candidate 合法，未发现 AI 自标 confirmed。但「候选结论」多处表述（如「708 y独立」「AVX 1.36x」「multi-channel 重构完成」）**证据不足以支撑其强度**，属于置信度标注偏乐观。
5. **产物契约**：**不满足**——缺 index.yaml 条目，结论未完整进 docs。
6. **噪声卡历史**：目标（density/aquifer 性能）无未解决噪声卡记录（该 session 为性能定位，非运行时失败累积）。
7. **retry cap**：ShiftDF/aquifer 环节有多次方向修正，但多为「新数据层证据」（新的计数/探针）驱动，非「无证据空转」；**未发现连续 3 轮无新证据的违规**。aquifer 曾因 P4 barrier 方向被计数证据推翻——这正是证据饱和机制的正当触发。
8. **模块边界**：无跨模块 skill 正文引用违规。

---

## 4. 审查意见汇总

| 环节 | 审查结论 | 推荐状态 |
|---|---|---|
| ① 顶层确认 | ✅ 可靠（reader） | 建议 candidate |
| ② multi-channel 重构 | ⚠️ diff0 局部可信，但生产未接线 + 落盘缺 | 建议 candidate（附「探针层完成」限定）；落盘驳回 |
| ③ corners noise vs aquifer | ✅ 不矛盾（两测量域），需 docs 显式区分 | 建议 candidate |
| ④ ShiftDF Cache2D | ⚠️ 高风险：证据（5节点/单列/仅ShiftAB）不足以支撑「708 全 y 独立」；cache 对 Shift mode y=0 偏离参考 | 建议 candidate（附局限），潜在 bug 需修 |
| ⑤⑥ noise AVX | ⚠️ 决策（noise 非优先）可靠；但「AVX 手工实现 1.36x」归因不实（死代码/非 SIMD），文案需修正 | 建议 candidate（决策），归因需修正 |
| ⑦ aquifer 主瓶颈 | ⚠️ 方向成立（宏观相对 Java 有真差距）；「21.5ms」为减法 Partial 快照，需限定；下一步 aquifer 优化合理但防再翻案 | 建议 candidate（方向 + 限定边界） |

**整体置信度：candidate（需人类确认）。confirmed 未达。**

---

## 5. 下一步建议（给主会话/人类）

1. **修正 ⑤ 归因（必做）**：把「noise AVX 手工实现 1.36x」改为「编译 target-feature=+avx 下 bench_noise 微基准快 1.36x（编译器 auto-vec），手工 `sample_section_avx` 为未接线死代码且未实现 SIMD dot」，避免误导后续「AVX 已实现」。
2. **修正 ④ 风险（建议）**：探针补测 plain `Shift`-mode 节点的 y 独立性（覆盖负 Y + 多列 + 覆盖>5 节点），或将 cache 的 Shift 分支与 C++/Java 参考对齐（保留实际 y）；在 cache 注释声明「仅验证 ShiftA/B，Shift 语义需复核」。
3. **澄清 ② 生产接线状态（必做）**：在 docs 里明确「multi-channel 竖切 = 探针层验证完成，生产 fill_chunk 未接线（仍逐点）」，避免把探针正确性误读为已生效优化。
4. **限定 ⑦ 边界（建议）**：docs 07 把「aquifer 21.5ms = 宏观子集减法 Partial 快照」标注清楚；优化前用无污染计数探针（aquifer_*_count）锁定 apply 固定开销真实构成，勿再直接上 barrier Cache2D（P4 已推翻）。
5. **补齐产物契约（必做）**：`.investigations/macro-layer-scout/` 结论 → 派 knowledge subagent 产出 docs 07/10-timewise 草稿；补 `.artifacts/index.yaml` 条目（macro-layer 顶层 / multi-channel 重构 / ShiftDF cache / noise-AVX-eval / aquifer-bottleneck）；提交未提交探针（macrolize_probe + corner_sampling_breakdown untracked）。
6. **noise AVX 的 AVX 路径若要真做**：真正实现 `sample_section_avx` 的 __m256d dot（而非标量占位），并接线到 `sample()`/`sample_section()` 热路径（按 env 门控 chunk 级一次判断），再复测全管线；当前「AVX 框架已加」的说法不成立。
