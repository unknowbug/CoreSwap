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
