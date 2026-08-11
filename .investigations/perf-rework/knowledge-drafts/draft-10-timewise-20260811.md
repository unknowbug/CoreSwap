# 10-timewise-archive.md 追加段草稿（2026-08-11）

> **应用方式（主会话）**：将下方 `---` 分隔线之后、`## 2026-08-11 ...` 起的内容，**原样追加到 `versions/1.20.1/docs/10-timewise-archive.md` 末尾**（该文档铁律：追加不覆盖）。
> **整体状态标注**：🔍（性能回归根因待修，未结案）——条目内已解决子项用 ✅、未解决子项用 🔍，与 10 篇既有条目风格一致。
> 草稿状态：draft — candidate（主会话应用后 + 根因修复闭环后再升）。

---

## 2026-08-11 性能回归调查 + Java 桥并发重写 + C++ 池改造（🔍 性能根因未结案）

> 承接 2026-08-10 深夜拍板（树花植被不做）后，用户实机发现**性能反降**：`-PcppReplace=1` 传送后区块生成卡很久才出现，纯 vanilla（`-PcppDisable=off`）对照确认——启动 perf-rework 调查（`.investigations/perf-rework/`）。
> 结论已提炼方向：requirements-doc.md（confirmed）+ static-audit.md（Java 桥并发静态审查）+ architecture.md（Phase 0 架构设计）+ 07 篇性能章节修正草稿（subagent 产出，待应用）+ discovered 模式 #10（thread_local 缓存冲突指纹）。本条保留完整推理链。

### 起因：实机性能反降（🔍 → ✅ 定位 Java 桥并发层）

- 🔍 **现象**：`-PcppReplace=1` 传送后区块卡很久才出现；纯 vanilla 对照确认 C++ 接管反而更慢（需求文档背景，2026-08-11）。
- ✅ **静态审查定位**（static-audit.md，审查对象 `CppBridge.java` 1.20.1-1.0.18；审查时 git 快照 MC HEAD=`78b615b` / CoreSwap HEAD=`0b92c62`，行号对审查时工作区 362 行）：
  - **P0-1**：JNI `fillBlocks` 被 `synchronized(BATCH_LOCK)` 全局锁串行化（noBatch L158 / 攒批 L182-197 / drainBatch L202）——对 JNI 多线程语义的认知错误（JNI 允许 native 被任意多线程并发调用，线程安全由 native 负责；C++ `wg_fill_blocks_multi` 设计即多线程）。
  - **P0-2**：writeChunk 锁内串行写 16 chunk（drainBatch L228-242 for 循环全程锁内）——157 万次 setBlockState 串行 + 阻塞攒批线程。
  - **P1-1**：攒不满 BATCH=16 时 `BATCH_LOCK.wait(2ms)`（L188-195）——低并发每 chunk 固定 +2ms，「区块卡很久」的直接体感来源。
  - **P1-2**：BATCH_BUFS 共享复用池（静态 `int[BATCH][98304]` ≈ 384KB/chunk）强制锁（L250-254）。
  - **P2-1**：writeChunk 每 chunk `new BlockState[4096]`（L260）——进程级静态可消除。
  - **P2-2**：feedBeardifier 每 chunk 全反射（15 次 Method.invoke）——P2-2 后续，不进本次范围。
- ✅ **runMtx 实证（Judge 第 2 轮 C1，worldgen_api.cpp L954-976）**：`CoreSwapPool::run` 内置 `static std::mutex runMtx` 锁住整个 run 生命周期（共享成员 fn/totalTasks/doneCount/nextTask/taskQueue 被并发 run 覆盖 → 读空 `std::function` 崩溃，**32 视距崩溃根因修复**）——即「批内并行（CoreSwapPool 多线程）、批间串行（runMtx）」；「C++ 耗时随线程数伸缩」在改造前不可达。
- ✅ **三层串行化定性**（architecture.md）：① Java BATCH_LOCK ② C++ runMtx ③ writeChunk 锁内循环。

### Java 桥去锁重写 + C++ 池改造（✅ 已实施，RQ-001~005）

- ✅ **目标架构**：去锁、M=1 非空即处理——每 worker 独立 thread-local buffer → JNI fillBlocks(1 chunk, buf) → 无锁 writeChunk 自己的 chunk；BATCH 攒批整个删除（用户拍板 M=1）；靠池并行摊薄 JNI 往返。
- ✅ **C++ CoreSwapPool 任务队列模型**：`run(count, f)` 提交 `{fn, shared_ptr<RunState>}` 到共享队列；RunState = `{atomic done, total, mtx, cvDone}`（per-run）；worker 循环取任务执行；调用方等自己 run 的 cvDone，不阻塞其他 run；删 runMtx。签名/对齐输出不变（`wg_fill_blocks_multi` 对 Java 透明）。风险：多 run 并发 = 池任务超订，操作系统调度兜底（用户拍板「崩了再说」测试策略）。
- ✅ **Java 侧改动（CppBridge.java）**：删 BATCH_LOCK/PENDING/BATCH_BUFS/drainBatch/wait；thread-local buffer（RQ-004）；stateById 进程级静态（RQ-005）；writeChunk 天然无锁（RQ-002）；noBatch 诊断路径保留为唯一路径（RQ-003）；feedBeardifier 不动。
- ✅ **随机种子对拍零退化**（random-seed-sampling.md，2026-08-11 改造后验证）：
  - `-8248318472910187742` 134304,434416 4×4 = TOTAL **99.9992%**（13 块差异）
  - `8576294172403134396` 200,200 8×8 = TOTAL **99.9997%**（22 块差异）
  - 与 2026-08-10 基线（99.9994%/99.9997%）同量级，差异均为既有插值课题类，**非本次引入**。只统计留知识，不修复（客户拍板）。

### 🔍 性能回归根因：FlatCache/Cache2D thread_local 缓存失效（未修，待立项）

- 🔍 **2026-08-11 吞吐实测（SURFACE 模式）**：单线程 **98-182ms/chunk**、多线程（8/22 线程）**108-239ms/chunk**——**无加速反降**；07 篇旧基线记录串行 28.1ms/chunk、并行 49.4ms/16chunk（3.1ms/chunk）。退化 ~3.5-6.5×（单线程）且并行不随线程数伸缩。
- 🔍 **WG_PROFILE 实测（density 阶段 670-1000ms/chunk，旧 8.5-11.7ms）**：
  - spline 单次 **20,598ns**（旧 992ns，~21×）
  - spline.sample **338 万次**
  - FlatCache rebuild **438,092 次 ≈ spline 调用数**——每次 spline 采样都重建 5×5 网格（缓存命中率≈0）
  - Cache2D miss **458,281 次**
- 🔍 **对照实验（排除本次改造引入）**：stash 本次改动后 HEAD 版 block_probe 8×8 仍 **10.2s**；连 07 篇基线提交 **86e4057** 也要 **8s** → **回归在 8/6 优化链之后积累，非本次改造引入**（本次改造保持对齐 8576 99.9994%/3200 99.9997%，未恶化吞吐；吞吐退化是独立预存问题，具体引入提交待 git 二分）。
- 🔍 **疑似根因（candidate 待验证）**：FlatCache/Cache2D 的 per-instance **thread_local** 缓存与「每 chunk 跨线程」执行模型冲突——多线程并行时每线程独立缓存 → 每 chunk 跨线程迁移 → 命中率归零、每 chunk 重建多次；叠加 buildGrid **嵌套采样递归**（边界点 x=cx*16+16 命中本 chunk 网格 k=4 才不重建，失配时触发相邻 chunk 网格重建递归）→ density 阶段 ~100 倍级恶化。
- 🔍 **待修状态**：根因修复未验证。候选方向：缓存按 chunk 键索引 / 按调用上下文显式传入 / 恢复线程亲和；需 git 二分定位 8/6 后引入提交。**未结案**。

### 决策：优化转向（已结案，2026-08-11 用户拍板）

- ✅ **放弃噪声 100% 对齐目标，转向优化优先**：有损容忍度 = **宏观一致**（地形/洞穴大体一致、允许方块级差异，肉眼基本看不出；用户实测地下也几乎看不出差异）。
- ✅ **300515 种子差异 = 非本项目问题**（BK-003）：参照含废弃前脏数据（花爆炸/树失败为废弃前实测），用户实测 vanilla 对照确认，不追责。
- ✅ **性能验收 = 体感**（BK-002）：游戏内「传送后区块出现时间」不采量化基线，验收凭用户体感。
- ✅ **RQ-006（C++ 有损加速，如 base_3d_noise 网格插值缓存）**：仅评估+用户逐项拍板后实施，不默认开（边界内待议）。
