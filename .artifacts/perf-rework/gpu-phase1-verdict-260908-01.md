# GPU Phase 1 摊销曲线 verdict（260908-01）

- id: `re-code:gpu-integration:phase1-amortization-verdict-260908-01`
- session: 260908-01（开测 260907-11 深夜，跨午夜按实际时刻归属）
- status: **confirmed**（用户「授权确认」260908-01；范围 = **noise/density 管线 GPU 接入**——负面结论落盘，维持搁置（三选一中的 ①）；光照侧不在本 verdict 范围，仅另有「不立项简记」（gpu-accel-errors.md）。judge PASS-with-conditions 5 条件已应用）
- 口径注记（260908-01 补，用户质询后核实）：本 verdict 引用的 CPU 分母「8 线程 ~4.5ms/chunk」来自 260903-12 estopt_mt_bench 扫描网格（T=1/2/4/8）上限档，**非生产配置**；生产自适应 = 物理核-2（api.rs adaptive_threads，本机 12C/24T → T=10，bench_threads 实测物理核-2 最优）。T=10 吞吐 ≥ T=8 → GPU 门槛（38×/170×）为**下界**，负面结论方向更稳；口径瑕疵不影响结论（用户拍板确认 260908-01）。
- 课题：GPU 管线接入（260907-10 用户拍板启动，目标 = 10× Java，重议 260903-12 搁置的 gpu-batch-merge）

## 结论

**攒批摊销不可达；瓶颈 = 解释器式 kernel 单点求值成本（~0.22ms/pt 主导，微观成分未分解但不妨碍决策）。**

- 摊销收益封顶 ~13%（per-pt 0.259→0.221ms，n=768→196608）：dispatch/readback 非主项。
- 带宽排除：角点形态上传 0.16GB/s ≪ PCIe——对 D24 是**形态限定而非推翻**（D24 = 逐块 3.4GB/chunk 死局；本次 = 角点 26.6MB/chunk 未饱和，瓶颈更上游）。
- 门槛核算（两口径二分，judge 条件 1）：摊销口径（<1ms/chunk 总预算）需 ~170×；重议门槛原义（计算不劣于 CPU 4.5ms/chunk）需 ~38×。两口径定性一致 = 现引擎形态下无解，达门槛等价 C2ME 式专用 kernel 重设计（新立项级）。
- 对照：GPU 169.8ms/chunk vs CPU 8 线程 4.5ms = 慢 ~38×；vs 单线程 27.69ms = 慢 ~6×。channels 路线更差（345-361ms/chunk @n=1）。

## 验证（§9.7）

- 载体：gpu_ffi_fill 同步 fill（含 readback）；seed=-8248318472910187742（同 260903-04/08/16/17 可比）；corner 768 点/chunk；严格串行。
- 前置：gpu_ffi.dll fresh 重出（sha 14096A5A...B9D53）+ 三件套复验全绿（corner 96.08%/9.18e-6 探针口径；channels major=0 跨 3 域）。
- 可复现性：独立两轮 256 chunks per-pt 0.2211/0.2197（<1% 偏差）；与 260903-16 旧读数 0.233 结构一致。
- 原始数据：`.investigations/perf-rework/cmd-output/gpu-n-scan-fresh-rerun.log`（全量 7 轮 ×4 档）。

## 判定含义（待用户拍板三选一）

1. **负面结论落盘，维持搁置**（回 CPU 主线；8 线程 ~7× Java 已在盘）；
2. **新立项：专用 kernel 重设计**（C2ME 式每类型专项化 + 网格预填充原生实现；~38× 起步门槛，工程量大，另走 Phase 0）;
3. 其他目标重定义（如 GPU 只服务特定子管线/离线预生成场景）。

关联：.investigations/perf-rework/gpu-phase1-amortization-260907-11.md（数据正文）+ review-gpu-phase1-amortization-260907-11.md（judge）+ gpu-accel-errors.md（D24/D25/G 系列）+ gpu-merge-revisit-260903-12.md（门槛出处）+ 架构计划-260907-11.md。
