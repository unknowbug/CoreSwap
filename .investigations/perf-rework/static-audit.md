# 静态审查记录（regression-record）：Java 桥并发设计问题

> 本文件是 requirements-doc.md 中 RQ-001~005 的 source 载体（judge 审核缺失信息 #1 的补录）。
> 审查对象：`E:\PYTHON\MC\versions\1.20.1\java\src\main\java\wg\bench\CppBridge.java`（1.20.1-1.0.18）
> 审查方式：静态读码（read_file），未运行。审查日期 2026-08-11，审查者 Reasonix 主会话。
> **审查时 git 快照（C6 补录）**：MC 工程 HEAD=`78b615b`（BlockProbe FULL-status export），工作区含未提交改动（build.gradle/CppWorldgen.java/BenchMod.java/BlockProbe.java/CppBridge.java 等，CppBridge.java 工作区 362 行 ≠ HEAD 303 行）；CoreSwap HEAD=`0b92c62`。行号引用对**审查时工作区**有效，后续提交会漂移。

## P0-1：JNI fillBlocks 被全局锁串行化（对 JNI 多线程语义的认知错误）

- **证据**：`fillChunk` noBatch 路径 L158 `synchronized (BATCH_LOCK)` 包裹整个 JNI fillBlocks + writeChunk；攒批路径 L182-197 同样 `synchronized (BATCH_LOCK)`；`drainBatch` L202 `synchronized (BATCH_LOCK)` 包裹全部处理。
- **事实**：JNI 规范允许 native 方法被任意多线程并发调用（JNIEnv per-thread），线程安全由 native 实现负责；C++ 侧 `wg_fill_blocks_multi`（worldgen_api.cpp L1006-1042）用 `CoreSwapPool` 线程池 + thread_local 缓存，设计即多线程。
- **补充实证（Judge 第 2 轮 C1，worldgen_api.cpp L954-976）**：`CoreSwapPool::run` 内置 `static std::mutex runMtx`（L959-960）锁住整个 run 生命周期——并发 run 会互相覆盖共享成员 `fn/totalTasks/doneCount/nextTask/taskQueue` → 读空 `std::function` 崩溃（**32 视距崩溃根因修复**）。即：**批内并行（CoreSwapPool 多线程）、批间串行（runMtx）**。「C++ 耗时随线程数伸缩」在不改造 C++ 并发层前不可达。
- **决策（2026-08-11 客户拍板）**：C++ 并发层改造纳入 RQ-001 范围——去 runMtx 或 per-caller 池，签名/对齐输出不变，批间真并行，不碰对齐语义。
- **后果**：C++ 并行仅体现在「一批 16 chunk 内部」，批与批串行；Java 侧本可并发的多个 worker 全部阻塞等锁。

## P0-2：writeChunk 锁内串行写 chunk

- **证据**：`drainBatch` L228-242 `for` 循环逐 chunk `writeChunk`，全程在 `synchronized(BATCH_LOCK)` 内。
- **事实**：`writeChunk`（L259-294）仅操作入参 Chunk 的 sections + 方法内局部 `stateById` 数组，无跨 chunk 共享写状态；不同 Chunk 写入天然隔离（vanilla populateNoise 同构并行）。
- **后果**：16 chunk 写入（157 万次 setBlockState）串行化，且阻塞所有攒批线程。

## P1-1：攒批 wait(2ms) 强制延迟

- **证据**：L188-195：攒不满 BATCH=16 时 `BATCH_LOCK.wait(BATCH_TIMEOUT_MS=2)`。
- **后果**：低并发（传送后请求流不够密）每 chunk 固定 +2ms 等待 → 「区块卡很久才出现」的直接体感来源。

## P1-2：BATCH_BUFS 共享复用池强制锁

- **证据**：L250-254：`BATCH_BUFS = new int[BATCH][16*16*384]` 静态共享池 + `BATCH_LOCK` 保护；drainBatch L218 `Arrays.copyOf(BATCH_BUFS, n)` 复用。
- **事实**：98304 int ≈ 384KB/chunk；per-thread 分配/池可消除「为 buf 安全而锁」。
- **注意**：若保留批量语义（每批最多 16 chunk），per-thread 池容量为 16×384KB ≈ 6MB/线程（judge 缺失信息 #3 指出文档 AS-003 低估）。

## P2-1：writeChunk 每 chunk 重建 stateById 缓存

- **证据**：L260：每次 writeChunk `new BlockState[4096]`；L275-277 首次命中走 `Registries.BLOCK.get(id)`。
- **事实**：vanilla 注册表运行期冻结（Fabric 1.20.1 无新增注册项假设，待确认）；id→BlockState 映射确定性、BlockState 不可变 → 进程级静态缓存安全。

## P2-2：feedBeardifier 每 chunk 全反射

- **证据**：L93-145：每 chunk 反射创建 StructureWeightSampler + 15 个 Method.invoke 提取 pieces/junctions。
- **后果**：深海区 pieces 常为空但仍建 sws + 反射遍历；结构密集区开销可观。

## 与知识库交叉引用

- 07 篇 L83「并行 vs 串行 TOTAL 必须同为 100.0000%（任何差异说明隐藏竞态）」——C++ 内部并行验证记录；**不覆盖跨 JNI 调用并发**（AS-001 待验证的证据层级说明）。
- 07 篇 L160-163 对齐基线表（8576/3200/-288/300515）——BK-001 出处。
