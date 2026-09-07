# Scout A：worldgen CPU 侧分阶段耗时盘点（260908-02）

> 角色：recode.scout（只读勘探）。目的：为 GPU 课题重入提供「CPU 侧阶段耗时分布」事实底表，标注哪些阶段已被 GPU 课题评估过、哪些从未评估。
> 本文只列事实 + 出处，不做可行性判断。
> 已否线路（不重复研究，见 `.investigations/perf-rework/gpu-accel-errors.md`）：noise/density 现有引擎形态 GPU 接入（D24 逐块 fill 带宽死局 / D25 interp 角点方案 C 结构不兼容 / D27 攒批摊销封顶 13%）；光照不立项。

## 一、主表：阶段 → 实测耗时（ms/chunk）

### A. Rust 载体（当前主线，worldgen.dll）

| 阶段 | 实测耗时 | 口径 | 出处 |
|---|---|---|---|
| **Rust 全管线** | 45.48 ms/chunk | 400 chunks、单线程、无树花 feature | versions/1.20.1/docs/07-block-pipeline.md L794；10-timewise-archive.md L2222 |
| **Rust 宏观（density+aquifer）** | 34.66 ms/chunk | 400 chunks 单线程 | 07 篇 L795 |
| **aquifer 段** | ~37 ms/chunk（占 FULL ~60%，FULL=62ms 口径）；另一口径 35.07（= no-ore 48.84 − no-aquifer 13.77，FULL=60.43） | bin-diag/qpd1_stage_bench.rs 隔离 bench，region(200,200)，两轮 ±1ms 一致 | 10-timewise-archive.md L2657、L2670 |
| **density/interp 底座** | ~14.4 ms/chunk（~23%） | 同上 qpd1_stage_bench | 10-timewise L2657 |
| **surface** | ~5.5-6.7 ms/chunk | 同上 | 10-timewise L2657 |
| **carver** | ~5-6.5 ms/chunk | 同上（⚠️ 机械成本反直觉：全 Air 列 carver-on 22.97 vs 实地形 10.62ms，机制未定位，见 b4 遗留 L2683） | 10-timewise L2657、L2683 |
| **orevein / features** | ≈0（噪声级） | 同上 | 10-timewise L2657 |
| **aquifer 内部（est 冷扫描）** | est ~15.4 + 冷 miss/杂项 ~6-8 + 暖 apply ~5.5 ms/chunk | Q-AQ1 归因（260903-10），Java est 列缓存跨 chunk 持久 vs Rust 每 chunk 丢弃 | 10-timewise L2678 |
| **est L2 优化后新基线（全管线）** | 单线程 27.69；8 线程 ~4.5 ms/chunk（vs Java FULL ~33，跨 bench 近似比较 judge D 标注） | region(200,200) 16×16、区外预热、median | 07 篇 L1137；10-timewise L2729 |
| **corners 宏观采样（NoiseChunk cell grid）** | 3.61 ms/chunk（ch#0 BlendDensity 3.60 为大头；含首次缓存构建 ~8.5ms） | corner_sampling_breakdown，预热后 | 07 篇 L898 |
| **cell grid 构建 / fill** | 构建 17 ms/chunk（优化前 443ms）；fill 单次缓存热 13μs | 性能复测 | 07 篇 L1003 |
| **biome** | **无实测**（未见任何 biome 阶段独立计时记录；bench 口径均不含 biome 阶段行） | — | — |
| **feature（树花）** | **无实测**（Rust 全管线口径「无树花」；qpd1 的 "features ≈0" 指 Rust 已有 feature 段，非 Java 全量树花成本） | — | — |

### B. Java 参照（端到端，大样本）

| 项 | 实测 | 口径 | 出处 |
|---|---|---|---|
| Java FULL | ≈55 ms/chunk（稳定 54-57，avg 51.7 含冷启动） | 256 chunks、充分预热、含树花一切 | 07 篇 L792 |
| Java 宏观 NOISE | ≈23-25 ms/chunk | 256 chunks | 07 篇 L793 |

### C. C++ 载体（历史参照，已归档 versions/1.20.1/cpp）

| 阶段 | T=1 | T=8 | 口径 | 出处 |
|---|---|---|---|---|
| density | 34-42 ms | 400-412 ms（11×） | WG_PHASETICK（QPC 单次零污染），block_probe | .investigations/worldgen-mt-scaling/density-latency-rootcause.md L8-13 |
| aquifer+ore | 8 ms | 25-28 ms | 同上 | 同上 |
| surface | 7 ms | 25-38 ms | 同上 | 同上 |
| total | 50 ms | 462 ms | 同上 | 同上 |
| （旧基线）density | 8.5-11.7 ms | — | WG_PROFILE 串行（2026-08-06 基线，计时列有污染风险，仅计数可靠） | 07 篇 L77、L153-158 |
| （旧基线）aquifer+oreVein | 6.5-8.9 ms | — | 同上 | 07 篇 L158 |
| 单线程复核 | density 47-57 + aquifer 32-61 + surface 8-24 = 总 90-120 ms/chunk | — | WG_STAGETIMER block_probe -threads 1 | .investigations/worldgen-mt-scaling/scout-map.md L80 |

### D. 光照（Rust 重写，用户已裁决不立项）

| 项 | 实测 | 口径 | 出处 |
|---|---|---|---|
| 光照内核 | 5.796 → 3.723 → 1.587 ms/chunk | light_golden_dump/light_bench_real，blocks9_real 真实数据，256 chunks 批 wall | versions/1.20.1/docs/12-lighting.md L47、L73 |
| 光照 e2e | 回退 2.4×（29.7 vs 12.4 s / 2025 chunks，gate ON vs OFF） | 全量 wall | 12-lighting.md L13 |

## 二、GPU 课题评估覆盖标注

| 阶段 | GPU 课题是否评估过 | 依据 |
|---|---|---|
| noise / density（含 interp/corners/DFC/GLSL） | ✅ **已评估并否掉**（D24 逐块 fill 带宽死局 / D25 interp 角点结构不兼容 / D27 攒批摊销封顶 13%；F1 单 chunk 往返 144μs、批量 256 chunk 摊薄吞吐 ~10000 chunk/s 曾为正结论，后经 D24/D27 收口） | .investigations/perf-rework/gpu-accel-errors.md；gpu-accel-findings-summary.md F1；.artifacts/perf-rework/gpu-phase1-verdict-260908-01.md；07 篇 L1138-1139（gpu-batch-merge 降级保留，重议门槛 = 10× Java ≈ 3.3ms/chunk，见 .artifacts/lossless-accel/gpu-merge-revisit-260903-12.md） |
| 光照 | ✅ **已评估，用户裁决不立项** | 12-lighting.md；任务背景 |
| **aquifer**（Rust 全管线最大头，~35-37ms、优化后仍为宏观真差距 1.4-1.5×） | ❌ **从未被 GPU 课题评估**（仅有 CPU 侧 est L2/归因课题） | 本表 A；gpu-accel-errors.md 无 aquifer 条目（grep 确认该文件主题为 density/noise/光照族） |
| **surface**（Rust ~5.5-6.7ms；C++ 7ms(T1)/25-38(T8)） | ❌ 从未评估 | — |
| **carver**（Rust ~5-6.5ms，机械成本反直觉未定位） | ❌ 从未评估 | — |
| **biome** | ❌ 从未评估，且**无实测耗时** | — |
| **feature/树花**（Java FULL 含树花 55ms vs Rust 无树花 45.48 —— 隐含差值 ~10ms 量级，未直接实测） | ❌ 从未评估，无独立实测 | 07 篇 L792/L794 |
| **orevein** | ❌ 从未评估（实测 ≈0，量级上无留白） | 10-timewise L2657 |
| **离线预生成场景**（批量摊销） | ⚠️ 部分评估过但仅在 density 线路上（F1 批量甜点 + D27 攒批封顶 13%）；非 density 阶段的批量/GPU 摊销从未评估 | gpu-accel-findings-summary.md F1；任务背景 D27 |

## 三、事实性附注（无判断）

1. Rust 当前最大单阶段 = aquifer（qpd1/Q-AQ1 双口径 35-37ms；est L2 优化已消一块，剩余构成见 Q-AQ1 分解）。
2. 全管线口径混乱提示（§9.7 可比性）：C++ PHASETICK 口径（T=1 total 50ms）、Rust 62ms FULL 口径、Rust 45.48/27.69 无树花口径、Java 55/33 FULL 口径互不可直接比数值。
3. surface/carver 各 ~5-7ms（单线程），合计与 aquifer 差一个数量级；carver 存在未定位的机械成本反常（全 Air 列更贵，b4）。
4. biome 阶段在所有 bench 口径中均无独立计时行——「无实测」是确证（grep versions/1.20.1/docs + .investigations 无 biome 耗时记录）。

## 出处文件清单

- versions/1.20.1/docs/07-block-pipeline.md（L785-831 Rust 端到端、L898 corners、L1003 grid、L1137-1139 gpu-batch-merge）
- versions/1.20.1/docs/10-timewise-archive.md（L2651-2678 Q-PD1/Q-AQ1、L2729 est L2 基线）
- versions/1.20.1/docs/12-lighting.md（光照内核/e2e）
- .investigations/worldgen-mt-scaling/density-latency-rootcause.md（WG_PHASETICK 表）
- .investigations/worldgen-mt-scaling/scout-map.md（L80 单线程复核）
- .investigations/perf-rework/gpu-accel-findings-summary.md（F1-F7 + 分层方案）
- .investigations/perf-rework/gpu-accel-errors.md（D24/D25/D27 否决线路）
- .artifacts/perf-rework/gpu-phase1-verdict-260908-01.md（GPU phase1 结论）
- .artifacts/lossless-accel/gpu-merge-revisit-260903-12.md（重议门槛）

— scout-a（260908-02），只读勘探，不含可行性判断。
