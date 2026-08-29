# 【草稿】`.artifacts/index.yaml` 应追加条目建议（macro-layer-scout）

> **[DRAFT — knowledge subagent 产出建议，待主会话应用 + 校验 + 提交]。**
> 背景：judge 审计（review-audit-conclusion-chain.md §3.5）判定**产物契约不满足**——`.artifacts/index.yaml` 无任何 macro-layer-scout / multi-channel / ShiftDF / noise 条目。以下为建议追加项（**status: candidate**，confirmed 由人类授予）。
> 注意：主会话应用前先**提交未提交探针**（`macrolize_probe.rs` 有 7 行 diff + `corner_sampling_breakdown.rs` untracked，judge §1② 指出），再补登记；确认 `path` 与最终文件实际位置一致。

---

## 建议追加 entries（插到 `.artifacts/index.yaml` entries 末尾，缩进对齐现有结构）

```yaml
  # === macro-layer-scout（Rust 宏观采样重构 + 性能定位）2026-08-30，status: candidate，judge 已审计 ===
  - id: 're-code:macro-layer-scout:topness'
    # 宏观采样真正顶层 = NoiseChunk::fill cell grid；6 个疑似上层非采样层
    path: '../.investigations/macro-layer-scout/macro-layer-topness.md'
    kind: analysis
    status: candidate
    # 结论摘要：宏观采样最终控制点 = NoiseChunk::fill 的 cell grid（slice SoA corners 采样 + 块级 trilerp + combine），其上仅为调度/装配，无采样机制层。
    # 6 个疑似上层（blending/StaticCache2D/Beardifier/aquifer/dim settings/ColumnCache）全部确认非采样层。reader 级可靠（judge ①✅）。

  - id: 're-code:macro-layer-scout:map-semantics'
    path: '../.investigations/macro-layer-scout/macro-layer-map.md'
    kind: analysis
    status: candidate
    # 结论摘要：Java/SteelMC = 单层宏观网格 multi-channel SoA（inner corners 采样 + outer combine）；Rust 现状 = 每 interp 独立自持 chunk 网格 + 逐点整树采样；「外层采样整树」与「内部独立缓存」结构叠加 → 52× 雪崩。正确方向 = multi-channel 竖切。

  - id: 're-code:macro-layer-scout:multichannel-refactor'
    # 建议 path：应用后改为最终落盘的 07 主题篇小节（如 '../versions/1.20.1/docs/07-block-pipeline.md#2026-08-30-Rust-宏观采样重构'）
    path: '../.investigations/macro-layer-scout/macro-layer-map.md'
    kind: analysis
    status: candidate
    # 结论摘要：multi-channel 竖切（final_density → 5 channels：1 BlendDensity + 4 RangeChoice noodle，combine 树 Interpolated→ReadChannel 全替换）正确性 diff0（n=54 局部，未盖 cell 内/边界/负Y）。⚠️ 生产 fill_chunk 未接线（DensityMacroSampler 仅在探针文件）；探针层验证完成，生产接线为下一步待办。

  - id: 're-code:macro-layer-scout:shiftdf-cache'
    path: '../.investigations/macro-layer-scout/cmd-output/shift_y_confirmed.txt'
    kind: evidence
    status: candidate
    # 结论摘要：ShiftDF Cache2D（ShiftA/B y 独立可靠：708/708 全 y 独立含负Y+多列）；plain Shift 保守不缓存（用实际 y，保持参考语义，修复 M1 y=0 偏离）。features_probe 95.40% 保持。

  - id: 're-code:macro-layer-scout:perf-loc-chain'
    path: '../.investigations/macro-layer-scout/cmd-output/tree_vs_noise.txt'
    kind: analysis
    status: candidate
    # 结论摘要：corners 采样 ch#0(BlendDensity terrain) 3.60ms 大头；ch#0 3.34ms 里 noise 采样 2.97ms(89%)、树遍历 0.38ms(11%)——噪声非树遍历。全管线 noise AVX 后仅 -1%（45.47→45.01ms）→ noise 非全管线瓶颈。

  - id: 're-code:macro-layer-scout:aquifer-gap'
    # 建议 path：应用后改为最终落盘的 07 主题篇小节（同 multichannel-refactor）
    path: '../.investigations/macro-layer-scout/cmd-output/noise_avx_eval.txt'
    kind: analysis
    status: candidate
    # 结论摘要：宏观 aquifer 是相对 Java 的待优化差距（Rust 全管线 45.48 < Java FULL 55，反快 1.2×；但宏观 Rust 34.66 > Java 23-25，aquifer 慢 ~1.4-1.5×）。⚠️「21.5ms」= 宏观子集减法 Partial 快照（34.66−13.14），非 aquifer 独立计时；表述限定为「相对 Java 待优化差距」，非「全管线绝对瓶颈」。

  - id: 're-code:macro-layer-scout:noise-avx-correction'
    path: '../.investigations/macro-layer-scout/multichannel-errors.md#M2'
    kind: errors
    status: candidate
    # 结论摘要（归因修正，M2）：noise AVX「1.36x」归因不实——`sample_section_avx` 为未接线死代码且非真 SIMD；实为编译 target-feature=+avx 下 auto-vec 微基准。「95.40% 不变」平凡（死代码未接线）。全管线 -1% 方向可靠 → noise 非瓶颈决策不因归因修正改变。

  - id: 're-code:macro-layer-scout:errors-m1-m2'
    path: '../.investigations/macro-layer-scout/multichannel-errors.md'
    kind: errors
    status: candidate
    # 结论摘要：错误台账 M1/M2 五段式——① ShiftDF Cache2D 对 plain Shift 强置 y=0 偏离参考（已保守改不缓存）；② noise AVX 归因不实（死代码+非真 SIMD，把 auto-vec 当手工 SIMD）。含错误→根因速查表。

  - id: 're-code:macro-layer-scout:review-audit'
    path: '../.investigations/macro-layer-scout/review-audit-conclusion-chain.md'
    kind: review
    status: candidate
    # 结论摘要：judge 审计意见——结论链主干正确、方向合理，但有 3 处修正/补充（②产物落盘缺、④ShiftDF 证据与代码语义不符、⑤noise AVX 归因不实）+ aquifer 减法 Partial 快照限定。整体推荐 candidate（非 confirmed）。
```

> **[DRAFT END]**——主会话应用：手动将上述 entries 追加到 `.artifacts/index.yaml`，调整 `path` 相对路径（建议指向最终落盘的 docs/07 小节或 investigation 文件，避免指向 knowledge-drafts 中间草稿；如结论已进 07 主题篇，`path` 宜改为 `'../versions/1.20.1/docs/07-block-pipeline.md#2026-08-30...'`），再 `pwd` 校验相对路径可达。
