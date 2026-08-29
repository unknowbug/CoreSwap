# 【草稿】07 主题篇追加小节：Rust 宏观采样层重构 + 性能定位（macro-layer-scout）

> **[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]**。
> status（各环节）：**candidate**（judge 已审计，confirmed 由人类授予）。
> 依据：`.investigations/macro-layer-scout/`{macro-layer-map.md, macro-layer-topness.md, review-audit-conclusion-chain.md} + `cmd-output/` 实测记录 + judge 审计意见（review-audit-conclusion-chain.md）。
> 载体：追加到 `versions/1.20.1/docs/07-block-pipeline.md` 末尾（追加不覆盖）。**本节只列中价值结论；错误链条见 §7 独立错误台账 multichannel-errors.md（M1/M2）；一次性数值快照按低价值不展开。**

---

## 2026-08-30 Rust 宏观采样重构（multi-channel 竖切）+ 性能定位（candidate，judge 已审计）

> 背景：Rust 全量 worldgen 功能链闭合后进入性能定位。本 session 完成三件事：① 确认宏观采样真正顶层（NoiseChunk cell grid，避免挖错层）；② multi-channel 竖切重构（对齐 SteelMC/Java 单层多 channel 语义）；③ 性能定位（noise 非全管线瓶颈；宏观 aquifer 是相对 Java 真差距）。judge 审计后标 **candidate**（各环节附限定）。

### 一、宏观采样的真正顶层 = NoiseChunk::fill 的 cell grid（中价值结论）

- **顶层确认**（`macro-layer-topness.md`，reader 级可靠）：SteelMC/Java 宏观采样最终控制点 = `NoiseChunk::fill` 的 cell grid（`fill_slice_into` corners 采样 + 块级 trilerp + `combine_interpolated`）。其下 `fill_cell_corner_densities`/`combine_interpolated`/`compute_noise_column` 是机制内部函数。
- **6 个疑似上层全部确认非采样层**（blending/StaticCache2D/Beardifier/aquifer/dimension settings/ColumnCache）——见 §7「顶层排除清单」。
- **对齐意义**：Rust 重构对齐目标应以**这一层**的 cell grid 语义为准，其上均为调度/装配，无需再找「更上层采样机制」。
- ⚠️ 与 Rust 现状差异：Rust 现 `MacroGrid`（对整树采样 corners）是**错误做法** → 52× 雪崩（采样点越界→内部 InterpolatedData 懒建网格反复重建）。正确方向 = 对齐 multi-channel 竖切（把 Interpolated 当独立 channel，避免「采样整树」触发自持缓存重建）。

**status：candidate（reader 级，judge 确认可靠✅）。**

### 二、multi-channel 竖切重构（final_density → 5 channels，**探针层验证完成，生产未接线**）中价值

- **结构**（`density.rs macrolize_channels/macrolize_into`，L604-681）：DFS 收集所有 `Interpolated` 的 inner 为独立 channel；`final_density` → **5 channels**（1 BlendDensity terrain + 4 RangeChoice noodle）；combine 树 `Interpolated→ReadChannel{ch}` 全部替换；`sample_combine` 的 `ReadChannel` 分支读 `interp[ch]`。channels inner 全「纯」（无嵌套 → 可独立采样，不触发雪崩）。
- **正确性**：`DensityMacroSampler` diff0（n=54 点，x,z∈{4,8,12}，y∈{4,64,128,200,260,300}，平均差异 0.000000）。**⚠️ 局部充分非全局证明**——54 点全落在 chunk 内部 cell 边界平面（fx/fz=0），未覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y / 跨 cell 插值路径（judge 限定）。
- **⚠️ 生产接线状态（judge 澄清，必标注）**：`DensityMacroSampler` **只在探针文件** `macro_sampler_probe.rs` 定义；生产 `terrain.rs fill_chunk` **未接线**（默认仍逐点 `dense.sample()`，`MacroGrid` 仅 `WG_MACROGRID` env 时启用）。**即「multi-channel 竖切重构 = 探针层验证完成，生产 fill 路径未接入优化。**
- **性能（标量，探针层）**：slices 构建 8.52ms + trilerp 0.3ms = 8.83ms vs 逐点 6.43ms（**标量结构正确但不省**）。`std::simd` (portable_simd) Rust 1.98 stable 不可用（需 nightly/intrinsics）；块级 trilerp SIMD 收益小（0.3ms）。
- **关键洞察**：Java 宏观高效的关键 = **ColumnCache（5×5 grid 缓存 xz 噪声值 O(1)）+ 批量 corners 采样（fill_cell_corner_densities_4x SIMD）**，不只是 SIMD。

**status：candidate（正确性在所测子空间可信；附「探针层未接线」限定）。生产接线为下一步待办。**

### 三、性能定位链（candidate，judge 修正归因）

- **corners 采样构成**（`corner_sampling_breakdown`）：ch#0（BlendDensity terrain，3677 节点）3.60ms/chunk（1225 corners）绝对大头；ch#1-4（RangeChoice noodle 小）各 ~0.1ms（合计 0.4ms）；所有 channels corners 总计 3.61ms/chunk（预热后，含首次缓存构建 ~8.5ms）。
- **tree_vs_noise（修正里程碑）**：ch#0 完整 corners 3.34ms → 去 noise 0.38ms → **noise 采样贡献 2.97ms（89%）**；树遍历仅 0.38ms（11%）。真正大头 = noise 采样（3D Perlin：Noise/ShiftedNoise/InterpolatedNoise/WeirdScaled），**非树解释器**（DFC 编译只优化 11% 树遍历，收益有限）。⚠️ 该修正与早前 `milestone_record.txt`（树解释器大头判断）相反，以 tree_vs_noise 为准。
- **noise AVX 评估（judge 修正归因）**：`sample_section_avx` 是**死代码**（从未被调用，生产走标量 `sample_section`）**且非真 SIMD**（函数体是标量 dot3/lerp/perlin_fade）。「Perlin 26.56→19.55ns（1.36x）」是 `bench_noise.rs`（标量 `sample()`）在 `-C target-feature=+avx` 下**编译器自动向量化**的微基准，**非手工 AVX 路径功劳**。「features_probe 95.40% 不变」是平凡事实（AVX 没接线）。**全管线 -1% 方向可靠**（45.47→45.01ms，400 chunks），说明 noise 非全管线瓶颈。→ **噪声 AVX 归因修正见 §7 M2。**
- **全管线差距定位（judge 限定）**：Rust 全管线 45.48ms < Java FULL 55ms → **Rust 反快 ~1.2×**；**但宏观专项 Rust 34.66ms > Java 23-25ms → 宏观 aquifer 慢 ~1.4-1.5×（相对 Java 真差距，唯一明确待优化点）**。⚠️ 「21.5ms」是**宏观子集减法 Partial 快照**（34.66 macro − 13.14 density），非 aquifer 独立计时；且 aquifer 内部曾多次翻案（barrier 大头已被计数证据推翻 P4）。→ 表述限定为「**宏观 aquifer 是相对 Java 的待优化差距**」，非「全管线绝对瓶颈」（Rust 全管线已快于 Java）。

**status：candidate（judge 确认「corners noise 89%」与「全管线 aquifer 大头」两测量域不矛盾，但需在 docs 显式区分「宏观 corners 采样成本」vs「全管线阶段构成」）。**

### 四、已排除假说（❌ 排除清单，保留一行，防重走弯路）

- ❌ **ch#0 corners 采样大头 = 树解释器/树遍历**——被 `tree_vs_noise` 推翻：noise 采样 89%，树遍历仅 11%。
- ❌ **noise AVX 手工实现带来 1.36x**——`sample_section_avx` 死代码/非 SIMD，1.36x 是编译器 auto-vec。
- ❌ **Rust 慢 Java 5 倍 / Java 8-9ms**——大样本修正后 Rust 反快 ~1.2×（已在 07 篇既有小节记录，不重复）。
- ❌ **`MacroGrid` 对整树采样 corners 是正解**——52× 雪崩（越界→自持缓存重建），正确方向是 multi-channel 竖切。
- ❌ **barrier.sample 是 aquifer 大头 / 加 barrier Cache2D**——已被 P4 计数证据推翻（07 篇既有小节已记）。
- ⚠️ **Shift mode 也 y 无关（可缓存）**——见 §7 M1，plain `minecraft:shift` 未验证 y 独立，cache 对 Shift 曾强置 y=0 偏离参考（已保守改不缓存）。

### 五、优化方向（candidate）

1. **（宏观 aquifer，相对 Java 真差距）** 优化 apply 每点 98304 次固定开销（~11ms 未解释）+ get_fluid_level/get_block_pos（~36%）；**优化前用无污染计数探针（aquifer_*_count）锁定 apply 固定开销真实构成**（防再翻案；勿直接上 barrier Cache2D，P4 已推翻）。
2. **（multi-channel 生产接线）** 把探针层验证完成的 multi-channel 竖切接入生产 `fill_chunk`（消除「采样整树」→ 雪崩）；补 ColumnCache + 批量 corners 采样（对齐 Java 宏观高效关键）。
3. **（noise）** 非当前瓶颈，后置；若之后 density 成瓶颈再考虑（真正的 AVX __m256d dot 需真实现 + 接线热路径，当前框架未做）。

### 六、域/边界

- 验证分层 = Partial（探针可复现，非 @anchor.test）；数值为当前快照，随优化变化。
- status：**candidate**（confirmed 由人类授予）；生产接线未完成，正确性证明局部（54 点）。

### 七、顶层排除清单 + 错误台账

- **顶层排除清单**（6 个疑似上层均非采样层，reader 确认）：① blending（树内部：BlendAlpha→常量 1.0/BlendOffset→常量 0.0，BlendedNoise 为 corners 叶采样，非外层包装）② StaticCache2D/ChunkHolder（调度层，仅 Beardifier 结构引用解析）③ Beardifier（块级、combine 之后加，非 corners 采样）④ aquifer（宏观采样之后逐 block 消费层，独立液面 cell grid，不重采样地形 density）⑤ dimension settings（cell 尺寸参数，非独立层）⑥ ColumnCache（采样机制内部性能缓存）。
- **错误台账独立成篇**：`.investigations/macro-layer-scout/multichannel-errors.md`（M1 ShiftDF 缓存 y=0 潜在 bug；M2 noise AVX 归因不实，五段式 + 速查表）。

> **[DRAFT END]**——主会话应用时：删除本节头尾 DRAFT 标记，追加到 07 篇末尾。
