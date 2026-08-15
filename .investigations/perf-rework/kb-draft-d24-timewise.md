# 草稿 1 —— 10-timewise-archive.md 新增条目（2026-08-15 深夜段 003 立项）

> **目标文件**：`versions/1.20.1/docs/10-timewise-archive.md`
> **插入位置**：追加到文件末尾（现有最后一条 = 2026-08-15 晚段 I1-I5/D23 条目，文件末尾 L1404 之后）
> **状态**：draft（subagent 产出，待主会话应用 + 一致性验证）
> **素材**：gpu-accel-errors.md D24 段 + 速查表 D24 行；架构计划 003；i-integration-record.md（I6-I8）

---

## 2026-08-15（深夜段）：GPU 块级生成立项 003（I6-I8）——逐 block 完整树 GPU 化实测不可行（❌ D24 split 全量上传带宽死局 / ✅ P2-4 并发崩溃修复 / ✅ 回退默认 CPU 零退化）

> 承接 2026-08-15 晚段 I1-I5 条目（GPU 引擎接入 worldgen + D23 修复闭环）。架构：`.investigations/000-架构设计/架构计划-gpu-block-integration.md`（003，用户 2026-08-15 批准「端到端 GPU 跑世界」）。目标：让 block_probe / 真实世界生成的**块级密度计算**（fillOneChunkCore 密度阶段）走 GPU，CPU 分支保持零退化。D24 完整错误记录（五段式 + 速查表行）见 gpu-accel-errors.md D24 段；通用模式见 discovered/algorithm-fingerprints.md 发现 #15。

### ✅ I6：fillOneChunkCore 密度阶段 GPU 分支 + fill() mutex 并发崩溃修复（P2-4 闭环）

- 接线：`#ifdef CORESWAP_GPU_ENABLED` 且 `h->gpu` 存在时，收集本 chunk 全部 **98304 点**（16×384×16，y = minY..minY+noiseHeight-1）→ `h->gpu->fill(coords, 98304, gpuOut)` 批量 dispatch（显存限制**分块 4096 点** batch fill）→ gpuOut(float) 转 densityBuf(double)，beard 逐块仍 CPU 加（L744 不动）；CPU 分支（无 GPU / 未启用）原样 = 零退化铁律。
- **并发崩溃（0xC0000005 @ nvtfi）**：I7 首次运行 `context=wg_fill_blocks_multi/fillOneChunk`，`code=0xC0000005`，栈在 **nvtfi（NVIDIA 驱动层）**——block_probe 默认 `-threads` 自适应多线程并发调 `h->gpu->fill()` → 共享 buffer 上传/dispatch 竞争 → **驱动层崩溃（不是返回错误，是进程级 0xC0000005）**。**P2-4 预言实锤**。
- **修复**：fill() 加 `std::mutex fillMtx` 串行化 → 无崩溃（P2-4 闭环；正确性解决，但串行化进一步劣化吞吐——「多线程并发 GPU 调用必须互斥」是硬约束，不是「可能有问题」）。

### ❌ I7：实测吞吐负面结论——11 分钟未完成 vs CPU 2.5 分钟（性能不可行）

- 24 chunks（8576 区域）GPU 逐 block 路径运行 **11 分钟未完成**（主动终止）；CPU 基线同区域 **2.5 分钟**——GPU 块级路径比 CPU **慢 4 倍+**（且未跑完）。语义对齐验证因此无法进行（跑不完）。
- **为什么不可行（D24 根因 = split 全量上传带宽死局，非计算慢）**：
  - GPU shader 求 finalDensity 完整树需要**每个点的全部分解坐标**：`splitTotal=8672` floats/点（CPU 预拆分，double→int32 格点 + float 小数）。
  - 逐 block 方案：98304 点/chunk × 8672 × 4B = **3.4GB split 数据/chunk** 需上传 GPU。
  - 分块 4096（显存限制）→ **24 次 dispatch/chunk**，每次 upload **142MB** + readback → 24 chunks × 24 次 = **576 次大上传 = 82GB 数据搬运** → PCIe ~16GB/s → 分钟级。
  - **GPU 快在「算」（compute throughput），这里被「喂数据」（host→device 带宽）完全主导**——GPU 批量加速的前提是「单点数据量小 + 点量大」，逐 block 方案把 8672 floats/点 的「每点数据量」直接变成带宽死局。
- **定位链**：① I7 首次运行（无 mutex）崩溃 0xC0000005 @ nvtfi → 多线程并发 fill 竞争 → mutex 串行化修复；② mutex 后无崩溃但 11 分钟跑不完 → 性能灾难暴露；③ CPU 基线 2.5 分钟 vs GPU 11 分钟未完成 → 带宽分析定位「split 全量上传」为瓶颈。

### ✅ 正确方向（若未来继续）：GPU 算网格角点 + CPU 插值，非逐 block 完整树

- GPU 只算 InterpolatedDF 网格角点（**768 点/chunk**，wg_fill_density 已验证 **22-39x**；27MB/chunk）→ CPU 三线性插值到 98304 逐 block。
- 数据量对比：768 点/chunk × 8672 × 4B = **27MB/chunk** vs 逐 block 98304 点 × 8672 × 4B = **3.4GB/chunk**（~125 倍数据量差）——**GPU 只在「网格角点级」批量才有意义**。
- 工作量中等：fillOneChunkCore 密度阶段改「先 GPU 出网格 → CPU 插值」，未实施。

### ✅ I8：回退——默认 CPU 路径零退化（99.9994%）

- I6 代码保留（WG_GPU_FILL=1 走 GPU 分支），**默认关闭 = CPU 路径 99.9994% 零退化**（8576 口径与基线一致；3200 沿用 99.9997%）。
- 最终结论：**GPU 块级加速在「逐 block 完整树」方案下不可行**（D24 定性为**方案不可行，非代码 bug**——接线正确、无崩溃、逻辑对，但吞吐不可行）；回退 CPU 路径为默认。

### 教训（D24 综合，完整版见 gpu-accel-errors.md D24 段）

1. **GPU 加速先算「每点喂多少数据」，不是先算「每点算多少」**：split 全量（8672 floats/点）让「每点数据量」成为带宽死局——GPU 批量加速的前提 = 「单点数据量小 + 点量大」（网格角点 768 点 × 27MB 可行；逐 block 98304 点 × 3.4GB 不可行）。
2. **吞吐探针结论有采样密度域**：I5 的 22-39x 证明的是「网格角点批量」，**不能外推到「逐 block」**——同引擎、同 shader，采样密度决定可行性（数据量 ∝ 点数）。
3. **多线程并发 GPU 调用必须互斥**（P2-4）：共享 buffer 上传/dispatch 无锁 → 驱动层 0xC0000005（不是返回错误）——GPU 资源并发是硬约束。
4. **负面结论也是结论**：I6 的「接线」本身正确（无崩溃、逻辑对），但吞吐不可行——记录「为什么不可行」（带宽分析）比假装成功有价值（错误优先原则）。

### 🔍 遗留项（未立项）

- 正确方向（GPU 网格角点 + CPU 插值）未实施——需 fillOneChunkCore 密度阶段重构（「先 GPU 出网格 → CPU 插值」），工作量中等，待后续立项评估。
