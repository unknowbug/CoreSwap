# 草稿 2 —— knowledge/discovered/algorithm-fingerprints.md 新增发现 #15

> **目标文件**：`knowledge/discovered/algorithm-fingerprints.md`
> **插入位置**：追加到文件末尾（现有最后一条 = 发现 #14，文件末尾 L390 之后）
> **状态**：draft（subagent 产出，待主会话应用 + 一致性验证）
> **素材**：gpu-accel-errors.md D24 段 + 速查表 D24 行；架构计划 003 结论段；i-integration-record.md（I5-I8）
> **INDEX 同步**：INDEX.md 对 discovered/algorithm-fingerprints.md 已有文件级入口（L14），不逐条列发现——本发现无需改 INDEX。

---

## 发现 #15: GPU 批量加速的带宽死局：每点数据量 × 点数是可行性判据（split 全量上传 vs 网格角点）

**发现时间:** 2026-08-15
**发现者:** worker（GPU 块级生成立项 003 I6-I8 实测 → D24 负面结论，perf-rework GPU 集成课题）
**来源定位:** `.investigations/perf-rework/gpu-accel-errors.md` D24 段 + `.investigations/000-架构设计/架构计划-gpu-block-integration.md`（003 结论段）+ `.investigations/perf-rework/i-integration-record.md`（I5-I8）
**置信度:** candidate（「split 全量上传带宽死局」机制由实测坐实——24 chunks 11 分钟未完成 vs CPU 2.5 分钟；通用规律跨项目外推待更多案例验证）
**module:** perf

### 观察

GPU 引擎算 finalDensity 完整树需要**每个点的全部分解坐标**（`splitTotal=8672` floats/点，CPU 预拆分 double→int32 格点 + float 小数）。同引擎、同 shader，**采样密度直接决定每点数据总量，进而决定可行性**：

- **逐 block 完整树（不可行）**：98304 点/chunk × 8672 × 4B = **3.4GB split 数据/chunk** 需上传；分块 4096（显存限制）→ 24 次 dispatch/chunk × 142MB + readback → 24 chunks × 24 次 = 576 次大上传 = **82GB 数据搬运** → PCIe ~16GB/s → 分钟级。实测 24 chunks **11 分钟未完成 vs CPU 2.5 分钟**（慢 4 倍+）。**GPU 快在「算」（compute throughput），但被「喂数据」（host→device 带宽）完全主导**。
- **网格角点级（可行，22-39x）**：768 点/chunk × 8672 × 4B = **27MB/chunk**——GPU 批量有意义（wg_fill_density 实测 22-39x）。

**核心判据**：**「单点数据量小 + 点量大」是 GPU 批量加速的前提**。每点数据量 × 点数 = 上传总量，超过 PCIe/总线带宽的秒级承载量（~GB/s 级）即带宽死局——与 GPU 算力无关（本次算力 24-32x 充足，卡在喂数据）。

### 证据

- 实测吞吐（I7）：24 chunks（8576 区域）GPU 逐 block 路径 **11 分钟未完成**（主动终止）；CPU 基线同区域 **2.5 分钟**——GPU 块级路径比 CPU 慢 4 倍+（`cmd-output/` I7 运行记录 + gpu-accel-errors.md D24 段）。
- 带宽账：98304 点/chunk × 8672 floats/点 × 4B = 3.4GB/chunk；分块 4096 → 24 次 × 142MB/次；24 chunks × 24 = 576 次大上传 = 82GB；PCIe ~16GB/s → 分钟级。
- 对照组（I5）：wg_fill_density 网格角点批量 768 点/chunk × 8672 × 4B = 27MB/chunk → **22-39x**（吞吐探针实测落盘 `cmd-output/throughput-I5-*.txt`）——同引擎同 shader，仅采样密度不同，可行性翻转。
- 并发崩溃（P2-4 闭环）：I7 首次运行（无 mutex）`context=wg_fill_blocks_multi/fillOneChunk` `code=0xC0000005`，栈在 nvtfi（NVIDIA 驱动层）；fill() 加 `std::mutex fillMtx` 串行化后无崩溃——**多线程并发 GPU 调用（共享 buffer 上传/dispatch 无互斥）→ 驱动层崩溃，不是返回错误**。
- 回退验证（I8）：默认 CPU 路径 8576 **99.9994%** 零退化（与基线一致）；3200 沿用 99.9997%。

### 如何利用

1. **GPU 加速可行性先算「每点喂多少数据」，再谈「每点算多少」**：上传总量 = 每点数据量（拆分坐标/特征向量）× 点数。凡每点数据量是 KB 级 × 点数是万级+（如逐 block 98304 点），先做带宽账（总量 ÷ PCIe ~16GB/s），分钟级即不可行——**「单点数据量小 + 点量大」是 GPU 批量加速的前提**；正确形态是「GPU 算网格/角点（点量小）+ CPU 插值/后处理到逐点」（两阶段拆分）。
2. **吞吐探针结论有「采样密度域」**：同一引擎同一 shader 在某采样密度（网格角点 768 点）下实测 22-39x，**不能外推到更高密度（逐 block 98304 点）**——数据量 ∝ 点数，可行性随密度翻转。引用吞吐结论必须声明采样密度域。
3. **多线程并发 GPU 调用必须互斥**：共享 buffer 上传/dispatch 无锁并发 → 驱动层 0xC0000005 进程级崩溃（**不是返回错误**，是崩溃）——GPU 资源并发是硬约束，任何多线程宿主（线程池/自适应并行）接入 GPU 路径 MUST 先加互斥（mutex/串行化），再谈性能。
4. **负面结论也是知识（错误优先原则）**：接线正确（无崩溃、逻辑对）但吞吐不可行时，记录「为什么不可行」（带宽分析 + 数据账）比假装成功有价值——避免后人重复实现同一死局方案；负面结论同样要落五段式错误台账 + 时间线。
