# Phase 1 攒批摊销曲线实测（260907-11 开测；judge 修订 260908-01，跨午夜注记：曲线测量与首轮分析实际时刻 2026-09-07 23:4x-09-08 00:0x，judge 条件应用与复跑 2026-09-08 00:1x-00:3x）

- 状态：draft → **candidate 建议**（judge PASS-with-conditions 260908-01，5 条件已应用；review 落盘 `.investigations/perf-rework/review-gpu-phase1-amortization-260907-11.md`；confirmed 留用户）
- 口径（§9.7）：载体 = gpu_ffi_fill 同步 fill（含 readback）；seed=-8248318472910187742（同 260903-04/08/16/17 可比）；corner 口径 768 点/chunk；严格串行（#28）；探针 = `.tmp/gpu_n_scan.exe`（bin-diag/gpu_n_scan.rs，rustc 单编 rlib 96a33b09，exe LastWriteTime 2026-09-07 23:21——#29 实值，与测量自洽）；设备 RTX 4060 Laptop。
- 原始输出：首轮尾部读数（对话采集）+ **复跑全量落盘 `.investigations/perf-rework/cmd-output/gpu-n-scan-fresh-rerun.log`（260908-01，judge 条件 2）**；复跑 256 chunks per-pt 0.2197（首轮 0.2211，偏差 <1%，结构一致可复现）。
- 前置：gpu_ffi.dll fresh 重出（sha 14096A5A...B9D53，build.ps1 -Ffi）+ 三件套复验全绿（corner 96.08%/9.18e-6；channels 逐通道 major=0 + combine major=0 跨 3 域）。

## 数据（7 轮中位数）

| n（点） | chunks | median | per-pt | per-chunk(768pt 折算) |
|---|---|---|---|---|
| 6144 | 8 | 1593.5ms | 0.2594ms | 199.2ms |
| 49152 | 64 | 11959.5ms | 0.2433ms | 186.9ms |
| 196608 | 256 | 43464.1ms | 0.2211ms | 169.8ms |

边际成本（相邻档差分）：768→6144 = 0.2534ms/pt；6144→49152 = 0.2410；49152→196608 = 0.2137。与 260903-16 旧读数 0.233ms/pt 结构一致（fresh 复现确认）。

## 结论（收敛型，单假设，证据三重）

**攒批摊销不可达；瓶颈在解释器式 kernel 的单点求值成本（~0.22ms/pt 主导）**——微观成分（纯算术吞吐 vs 解释器分派/sync/workgroup/f64 残留）本轮数据未分解（judge A 条件收窄），但无论哪个成分主导，攒批均救不了。

1. 攒批摊销收益封顶 ~13%（0.259→0.221ms/pt，768→256 chunks）——dispatch/readback 固定成本在 n=768 时已基本摊没，不是主项。
2. 带宽排除：n=196608 上传 6.8GB / 43.5s ≈ 0.16GB/s ≪ PCIe ~16GB/s——split 上传远未饱和，D24 带宽死局在**角点批量**形态下不是当前瓶颈（瓶颈更上游：计算）。
3. 门槛核算（judge 条件 1，两口径二分）：
   - **摊销口径**（<1ms/chunk 含计算总预算）→ 需 per-pt <1.3μs = 当前 **~170×**；
   - **重议门槛原义**（摊销 <1ms **且** 计算不劣于 CPU 4.5ms/chunk）→ 计算侧需 <5.86μs/pt = 当前 **~38×**；
   - 定性结论两口径一致：均远超「接入现有引擎」范畴，等价 C2ME 式专用 kernel 重设计（新立项级）。
4. 对照：GPU 169.8ms/chunk vs Rust CPU 8 线程 ~4.5ms/chunk = **GPU 慢 ~38×**；vs 单线程 27.69ms = 慢 ~6×。channels 路线更差（n=1 实测 345-361ms/chunk，5 通道 ×5 spv）。

## 判定含义（待用户拍板）

- 重议触发条件的量化门槛（dispatch+readback 摊销 <1ms/chunk 且计算不劣于 CPU）**计算侧不满足**：需要 ~170× kernel 提速，等价于放弃解释器式通用 shader、走 C2ME 式专用 kernel 重设计（每 DF 类型专项化 + 网格预填充原生实现）——新立项级别，非本课题「接入」范畴。
- 若维持「接入现有引擎」，负面结论成立：GPU 全树/channels 两路线相对现 Rust CPU 均为负收益，应记录后维持搁置（D24/D25/D27 家族负面结论第三例）。

## 附带采集

- Phase 0.5 复验（同日）：dll fresh sha 14096A5AB545891BA8B307550BA2100E3BF4E2B2AF943950FBD01526934B9D53；corner probe rounded6=96.08% max_diff=9.179e-6（§9.7：探针口径，与历史 96.06% 同载体可比）；channels probe 全通道 major=0（含大坐标域 104852/-104856）+ combine major=0。
- 环境：测量期间用户游戏已暂停（23:4x 前一度并行跑过一轮作废，本轮为清洁重跑，7 轮方差小、与历史同构）。
