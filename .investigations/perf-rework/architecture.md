# Phase 0 架构设计：Java 桥并发重写 + C++ 池并发化

> 依据：`.investigations/perf-rework/requirements-doc.md`（confirmed，2026-08-11）
> 目标：消除「性能反降」——JNI 批间串行（Java 锁 + C++ runMtx）+ chunk 写入串行 + 攒批 wait。
> 约束（BK-001/RQ-001 范围）：签名/对齐输出不变；C++ 对齐代码不改；宏观一致容忍只评估待议（RQ-006 不做）。

## 一、当前并发模型（问题）

```
MC worldgen executor（N worker 线程）
  → populateNoise mixin → CppBridge.fillChunk(chunk)
    → synchronized(BATCH_LOCK) 攒批 + wait(2ms)      [Java 全局锁：批间串行]
      → drainBatch（锁内）：JNI fillBlocks(16 chunk) → C++ CoreSwapPool（批内并行，批间 runMtx 串行）
        → writeChunk ×16（锁内串行写 157 万 setBlockState）
```

三层串行化：① Java BATCH_LOCK ② C++ runMtx ③ writeChunk 锁内循环。

## 二、目标并发模型（去锁，M=1 非空即处理）

```
MC worldgen executor（N worker 线程，各自独立）
  → populateNoise mixin → CppBridge.fillChunk(chunk)
    → 无全局锁：thread-local buffer → JNI fillBlocks(1 chunk, buf)   [M=1 直接调]
      → C++ CoreSwapPool（改造后：任务队列模型，多 run 并发安全，N worker 并行消费）
    → writeChunk(chunk, buf) 独立 Chunk 对象，无锁并行
```

- **无共享队列、无全局锁**：每 worker 独立调用 JNI（JNI 本身多线程安全）+ 独立写自己的 chunk
- **C++ 池改造后**：N 个并发 JNI 调用 → 任务全进池队列 → 池 worker 并行消费 = 真 N 核并行
- **BATCH 攒批整个删除**（M=1，用户拍板）——JNI 往返 1 次/chunk，靠池并行摊薄
- **thread-local buffer**（去 BATCH_BUFS 共享池，RQ-004）
- **stateById 进程级静态**（RQ-005）：`static volatile BlockState[4096]` 幂等懒填充（同 index 多线程写同值，无需锁）
- **feedBeardifier**：保持现状（反射），不进本次范围（P2-2 后续）

## 三、C++ CoreSwapPool 改造（RQ-001 范围：去 runMtx / per-caller 池）

**现状**（worldgen_api.cpp L922-1001）：单例池，`run()` 用共享成员 `fn/totalTasks/doneCount/nextTask/taskQueue` + `static std::mutex runMtx` 串行化批间（L959-960，32 视距崩溃修复）。

**改造方案：任务队列模型（并发 run 安全）**
- `run(count, f)` 提交 `count` 个任务 `{fn, shared_ptr<RunState>}` 到共享队列
- `RunState = {std::atomic<int> done, int total, std::mutex mtx, std::condition_variable cvDone}`（per-run）
- worker 循环：取任务 → 执行 `fn(taskId)` → `RunState.done++` → 完成时 `cvDone.notify`
- 调用方（JNI 线程）等自己的 `RunState.cvDone`，不阻塞其他 run
- 无共享 fn/计数器 → 并发 run 天然安全 → 删 runMtx
- `ensure(n)` 扩容逻辑保留；`shutdown()` 保留
- **签名不变**：`wg_fill_blocks_multi(handle, xs, zs, outs, count, threads)` 对 Java 完全透明
- **对齐输出不变**：fillOneChunkCore 不动

**风险**：多 run 并发 = 池任务超订（N 调用 × 每调用 count chunk）。缓解：`threads > count` 截断已存在（L1025）；池 worker 数 = physicalCoreCount（自适应），超订由操作系统调度兜底（用户拍板「崩了再说」测试策略）。

## 四、Java 侧改动清单（CppBridge.java）

| 改动 | RQ | 说明 |
|---|---|---|
| 删 BATCH_LOCK/PENDING/BATCH_BUFS/drainBatch/wait | RQ-003/004 | M=1 直接调 fillBlocks |
| thread-local buffer（每 worker 1 个 98304 int ≈ 384KB） | RQ-004 | `ThreadLocal<int[]>` |
| stateById 进程级静态 | RQ-005 | `static final BlockState[4096]` volatile 懒填充 |
| writeChunk 去锁（天然） | RQ-002 | 每 worker 写自己 chunk |
| noBatch 诊断路径保留为唯一路径 | RQ-003 | 原 noBatch 逻辑即新逻辑 |
| feedBeardifier 不动 | — | P2-2 后续 |

## 五、验证（RQ-004 用户拍板：随机抽种子对拍 + 统计差异分布）

1. **编译**：MSVC 重编 worldgen_core + block_probe + worldgen.dll
2. **block_probe 回归**：8576/3200 SURFACE 保持 99.999x%（对齐输出不变的铁证）
3. **随机种子对拍**：抽 1 个种子（排除 300515 类脏参照），block_probe 对拍 + 统计差异分布落盘（只统计留知识）
4. **JNI 并发回归**（AS-001）：runClient 多线程生成，确认无崩溃/无竞态污染（改造后池并发安全）
5. **体感验收**（BK-002）：用户传送场景体验，vanilla 对照

## 六、风险与回退

- 池超订：若并发 run 导致性能反而下降，回退方案 = Java 侧加轻量信号量限流并发 JNI 调用数（保留去锁收益）
- 32 视距崩溃回归：改造后需确认并发 run 不再有 fn 覆盖（RunState 隔离设计即修复）
- worldgen.dll 同步：MSVC 编译后同步 MC resources + sha256 校验（对齐铁律）

## 七、交付物

- C++：worldgen_api.cpp（CoreSwapPool 改造）
- Java：CppBridge.java（去锁重写）
- 验证：block_probe 回归输出 + 随机种子差异统计 + 运行日志
- 文档：10 时间线 + 07 篇性能章节更新（subagent 产出草稿）
