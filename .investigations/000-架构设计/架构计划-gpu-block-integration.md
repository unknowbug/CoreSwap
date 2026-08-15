# 架构计划 003：GPU 参与块级生成（fillOneChunkCore 密度阶段 GPU 化）——2026-08-15

> 承接 002（I1-I5，confirmed）。用户 2026-08-15 拍板立项「端到端 GPU 跑世界」。
> 目标：让 block_probe / 真实世界生成的**块级密度计算**走 GPU，CPU 分支保持零退化。

## 现状（已核实）

- `fillOneChunkCore`（worldgen_api.cpp L645）密度阶段 L736-748：逐块（16×384×16=98304 点/chunk）
  `h->finalDensity->sample(fpos)` 填 densityBuf，beard 逐块加（L744）。
- `finalDensity->sample` 内部 = InterpolatedDF cell 网格插值（5×49×5 网格，x/z 每 4、y 每 8；
  只对 interpolated 节点插值，min/squeeze/mul 非线性在插值后应用——Java CellCache 语义，L679-681 注释）。
- GPU 引擎（GpuDensityEngine.fill）已能算任意坐标的 finalDensity（批量，22-39x，e2e 3.128e-07 逐位验证）。
- **gap**：fillOneChunkCore 不调 GPU；block_probe 走 wg_fill_blocks_multi → fillOneChunkCore（CPU）。

## 方案（轻量 ≤3 要点）

**I6：fillOneChunkCore 密度阶段 GPU 分支**
- `#ifdef CORESWAP_GPU_ENABLED` 且 `h->gpu` 存在时：
  1. 收集本 chunk 全部 98304 个 (x,y,z)（16×384×16，y = minY..minY+noiseHeight-1）
  2. `h->gpu->fill(coords, 98304, gpuOut)` 一次批量 dispatch
  3. gpuOut(float) → densityBuf(double)，beard 逐块仍 CPU 加（L744 不动）
- CPU 分支（无 GPU / 未启用）保持原样 = 零退化铁律。

**I7：语义对齐验证（关键）**
- GPU 算的是「finalDensity 完整树在任意坐标的值」（已 e2e 验证 vs CPU sample 3.128e-07）。
- fillOneChunkCore CPU 路径的 sample 走 InterpolatedDF 网格插值——两者语义应一致（同一棵 finalDensity 树），
  但需块级实测：WG_GPU_FILL=1 block_probe 8576 → 期望 ≥99.99%（GPU float 1e-7 级差在密度阈值附近可能翻极少数块）。
- **预期风险**：GPU float vs CPU double 的 1e-6~1e-7 差 → 密度判定阈值（0 边界）附近块可能翻转。
  实测对比 I4a（99.9994%）基线，量化损失。若损失可接受（<0.01%）→ 通过；否则评估 float 升级策略。

**I8：吞吐 + 零退化终验**
- 吞吐：block_probe 8576 总耗时 GPU vs CPU（含 pipeline 70-100s 一次性，剔除后对比稳态）。
- 零退化：CPU 分支（无 WG_GPU_FILL）8576/3200 与基线一致；GPU 分支与 CPU 分支块级对比。

## 角色 / 验证

- swe 主会话闭环（写码 + 编译 + block_probe 验证）；judge 收尾审查；知识库 subagent 产出草稿。
- 验证载体：block_probe（Full）、cmd-output/ 落盘；@anchor.test 沿用 densityBuf 锚点（source 不变）。

## 风险 / 回退

- GPU float 精度损失不可接受 → 回退 CPU 分支（默认路径不变，WG_GPU_FILL 只是开关）。
- pipeline 编译 70-100s 一次性（block_probe 单进程内）；mod 场景每用户首次（G6 已知约束）。
- fill() 无互斥（P2-4 遗留）：本方案单线程 fillOneChunkCore 调用，不引入并发。

## 结论（2026-08-15 实测后更新）——❌ 逐 block 方案不可行（D24）

- I6 接线实现完成：fillOneChunkCore 密度阶段 GPU 分支（分块 4096 点 batch fill）+ fill() 加 mutex（P2-4 并发崩溃修复）。
- **正确性侧**：mutex 修复后无崩溃（多线程并发 fill 驱动层崩溃 0xC0000005 已解决）。
- **吞吐侧（决定性负面）**：24 chunks GPU 逐 block 路径 **11 分钟未完成 vs CPU 2.5 分钟**（慢 4 倍+）——根因 = **split 全量上传带宽死局**：98304 点/chunk × 8672 floats/点 = 3.4GB/chunk 需上传（分块 24 次 × 142MB）。
- **正确方向**（若未来继续）：GPU 只算网格角点（768 点/chunk，wg_fill_density 已验证 22-39x）+ CPU 三线性插值到逐 block——非逐 block 完整树。
- **当前状态**：I6 代码保留（WG_GPU_FILL=1 走 GPU 分支），默认 CPU 路径零退化不受影响；D24 完整记录见 gpu-accel-errors.md。

## 方案 C 侦察结论（2026-08-15 深夜段，方向验证）——❌ 不可行（shader 角点分组结构限制）

- **方案 B（完整 finalDensity 网格插值）排除**：sim 实测网格加密到 step=(1,2,1) 误差仍 5e-2——完整树含非线性（min/squeeze/noodle），插值非线性不成立（结构性错误，非精度）。
- **方案 C（interp 内容树角点 + CPU 插值 + 外层非线性）验证 → 不可行**：
  - **语义正确性前提**：interp_N（8 角点 delegate + 插值）== 完整树 eval_df **逐位一致（maxDiff=0，sim 验证）**——interp_N 是 GPU 唯一正确形式。
  - **致命限制（角点分组噪声）**：GPU 的 interp 内容树噪声是「8 份冗余实例」结构（`continentalness@c0..c7` 等，参数完全相同，仅 `_key` 带角点后缀）——**每实例的 split 行 = 固定角点坐标的拆分**（sim 实证：不同 sIdx 同坐标值相同；corner 0-7 同坐标 range 1.6e-1~3.1e-1）。**内容树无法用「该坐标的拆分」算任意点**——实例绑定固定角点坐标。
  - **结论**：GPU 角点分组结构与「共享实例 + 每点坐标拆分」（CPU 语义）不兼容——1225 网格角点无法独立求值。
- **彻底裁决**：GPU 块级生成在当前 shader 结构下**无可行路径**。D24 从「带宽死局」深化为「**结构不兼容**」（角点分组 vs 共享网格）。要突破需重构生成器的 interp 噪声为「共享实例 + 坐标参数」（CPU 语义），工程量大且收益存疑（外层非线性仍 CPU）。
- **当前状态**：I6 代码保留（WG_GPU_FILL 开关，默认 CPU 零退化）；wg_fill_density 批量 API（22-39x）是 GPU 的实际可用成果。

## 004 候选方向：FP32 算子库（用户 2026-08-15 讨论，未立项）

**用户提议**：写一组简单 FP32 算子（噪声/spline/求值），把需要算的 FP32 运算批量扔 GPU、取回数据。

**可行性分析（主会话）**：
- ✅ **带宽可行**：`normal_noise(noiseIdx, sIdx)` 是独立函数（按实例查 NORMAL_PACK 参数表 + split 行）——天然单算子。单算子每点只需该实例的 split 行（12-几百 floats），vs 完整树 8672——**带宽降 1-2 个数量级**。
- ✅ **精度**：FP32 + 坐标预拆分 → e2e maxDiff 3.128e-07、I5 1e-6~8e-6——**近似用途可用**（非逐位对齐）。
- ✅ **运行时底座**：VkRuntime（init/createPipeline/upload/dispatch/readback）已通用，支持任意 shader/buffer 布局。
- ⚠️ **前提**：接受「1e-6 级近似精度」（放弃逐位对齐铁律）；逐位对齐场景仍需 CPU double。
- **最小实现**：算子 shader（如 op_noise.comp：n×3 int32 坐标 + 该算子 split 子集 → n float）+ 宿主 GpuOp 类（复用 VkRuntime）。
- **带宽实测账**（check_op_bandwidth.py）：10 万点批量，单算子（continentalness/erosion 60 floats/点）**4.8-77 MB** vs 完整树 **3469 MB**——**带宽降 45-723 倍**，算子模式完全可行。
- **价值场景**：mod 里任何「大量点密度/噪声查询 + 可接受近似」（地图预览/分析/统计/非对齐功能）。

## 004 实施验证（2026-08-15 晚段，op_probe 实测）——✅ 算子库成立

**实现**：`vulkan-proto/op_noise.comp`（FP32 单算子 normal_noise，复用生成器 GLSL 函数体）+ `op_probe.cpp`（VkRuntime + CpuBackend，提取目标实例 split 行）。

**实测（10 万点，实例 0 = continentalness@c0）**：
- **精度**：GPU FP32 vs CPU double（同 split 行）**maxDiff=6.9e-7 / avgDiff=1.34e-7**——FP32 舍入级，近似用途完全够。
- **吞吐**：GPU **1186 万点/s** vs CPU double 内联参照 309 万点/s = **3.8x**；带宽紧凑 43.2MB vs 全量 3469MB = **80x 降**。
- **关键修正**：初版 maxDiff=1.085 是参照 bug（CPU 用原始坐标 sample，split 是角点对齐坐标——两算的不是同一点）；改「同 split 行 double 参照」后 6.9e-7 ✓。
- **吞吐低于 I5 的 22-39x 原因**：CPU 参照是纯 double 计算无 split 开销（32ms）；真实场景 CPU 完整路径含 split（25.6s）→ GPU 算子实际增益更大。
- **结论**：FP32 算子库（近似精度 + 小带宽 + 3.8x+ 吞吐）**成立**——「大量点噪声/密度查询」的 GPU 正确用法。
- **对照**：本质 = C2ME OpenCL 架构的简化版（不重构生成器，只暴露单算子入口）；C2ME 全程 fp64 实时算，我们走 fp32 + 预拆分（精度 1e-6 够近似用）。
- **未立项**：待用户拍板（今天讨论，未实施）。

## C2ME 对照总结（2026-08-15 深夜段核实）——为什么我们走错了路

**C2ME 实际架构**（源码核实，E:\PYTHON\MC\data\C2ME-fabric）：
- **主加速 = CPU 多线程**（README：「taking advantage of multiple CPU cores」）——99% 收益来源。
- **OpenCL GPU 只是可选实验模块**（c2me-opts-accel-opencl，非默认）。
- **OpenCL 架构**（对我们有对照价值）：
  - 精度：**GPU 内 fp64 全程实时算**（maintainPrecision double 折叠 L117-119，与 Java 一致）——**无 CPU 预拆分**。
  - 带宽：每点只传 **3 int32 坐标（12 字节）** + 噪声参数表（const_data 一次性）——无 8672 floats/点。
  - interpolated 两阶段：角点预填充内核按网格 dispatch（每角点 1 次 delegate 调用，几十字节）→ 主内核读 buffer 插值（L442-474）。
  - 拆 7 内核 + 预编译二进制随 mod 分发（tar.zst，秒级加载）。

**我们 vs C2ME 三个架构分岔**：
| | 我们（D2 起） | C2ME OpenCL |
|---|---|---|
| 精度策略 | CPU 预拆分 8672 floats/点 → 上传 | GPU 内 fp64 实时（3 int32/点） |
| 角点结构 | 8 份冗余实例（D25 死局） | 共享 delegate + 坐标参数 |
| 插值 | 单 pass 8 角点内联 | 两阶段（预填充 + 插值） |

**根本教训**：我们为了 fp32 性能选 split 预拆分架构，代价 = 带宽死局（D24）+ 角点结构死局（D25）。C2ME 证明「GPU fp64 实时 + 共享 delegate」能绕开——但**消费 GPU fp64 吞吐低（RTX 4060 ≈ 1/64 fp32）且 OpenCL 模块是实验性的**，所以 C2ME 也没默认开。**GPU 块生成的收益天花板就是「C2ME 都只能做实验模块」的水平**——这也是我们收尾 GPU 块生成课题、转向「FP32 算子库（近似用途）」的依据。
