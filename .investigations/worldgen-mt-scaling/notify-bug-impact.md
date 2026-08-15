# notify 丢失 bug 影响范围评估（2026-08-16）

> 状态：draft（主会话临时排查记录；结论性落盘待知识库 subagent 草稿）
> 课题：CoreSwapPool notify 丢失 bug（0a781e1 修复）对历史数据/结论的影响面

## 0. Bug 定义与生命周期

- **现象**：多线程 bench「反降」——T=8/12/22 无加速甚至更慢；WG_TASKTIME 实证补建 worker 全空闲（只有老 worker 干活 = 串行假象）
- **根因**：CoreSwapPool::ensure()（worldgen_api.cpp L1057-1098）在 mtx 锁内补建 worker；新 worker 启动后要拿 mtx 进 `cvTask.wait`；`run()` 入队后 `notify_all()`（L1125）时新 worker 可能还没进 wait → **notify 丢失 → 永久等待**（tasks 空 + stop false）→ 只有老 worker 干活
- **修复**：`readyCount` 原子 + `run()` 入队前等 `readyCount >= workers.size()`（L1110-1118）
- **引入**：`252d988`（8/6 20:11，per-run 隔离 + 扩容支持）——`ensure()` 补建 worker 逻辑
- **修复**：`0a781e1`（8/15 23:50）
- **活跃期**：8/6 20:11 → 8/15 23:50（约 9 天）

## 1. 触发条件

**同一进程内线程数递增**（顺序跑 {1,8,12,22}）：
- bench [A]：wg_fill_blocks_multi(count=64/256, threads=T)，T=1 先建 1 worker → T=8 ensure(8) 补建 7 个 → 补建的错过 notify → 本批 64 任务全由 1 个老 worker 串行执行
- T=8 **单独跑**（进程内首个调用）时：8 个 worker 全部新建但 `run()` 的 `if (workers.empty()) ensure(count)` 分支 + 无老 worker → 实测完美并行（WG_TASKTIME）——因为首次 ensure 后所有 worker 都抢锁进 wait，run 入队时它们已就绪（时序上更接近）
- **不触发**：固定线程数进程（mod 实机每 worker count=1 直接调，不递增）；单线程

## 2. 受影响的数据/结论

| # | 数据/结论 | 位置 | 影响 | 处理 |
|---|---|---|---|---|
| 1 | bench [A] T>1 顺序跑数据：「反降 +19-29%」 | bench-C2-20260815.txt / bench_8x8_noprof.txt | ❌ **串行假象**（补建 worker 空闲，实际并行度=1） | 作废；修复后重测 |
| 2 | scout-map C1-C7 排查链中 T>1 对比基线 | scout-map.md L73-99 | ⚠️ 反降幅度被高估（真并行度=1），但「C1-C7 非主因」结论仍成立（那些是单线程+结构分析） | 保留结论，标注幅度不可信 |
| 3 | 07 篇 08-11「并行反降 108-239ms」 | 07-block-pipeline.md L74 | ⚠️ 反降部分混入 notify bug（当时并行度=1）；H2 主因（单线程可复现 rebuild 168×）**不受影响** | H2 保留；H3 待重测 |
| 4 | H3「thrashing ×16」（mt 27,155ns vs t1 1,714ns） | 07-block-pipeline.md L97/L109 | ⚠️ mt 侧数据在 bug 活跃期采集（实际串行环境）→ ×16 需重新定性 | 重测（修复后 mt 单次） |
| 5 | 「多线程无加速/反降」被当作性能现实反复排查 9 天 | scout-map 全篇 | ❌ 排查方向被误导（C1-C7 全是表象） | 教训已记录 |
| 6 | WG_STAGETIMER/WG_PROFILE 计时污染数据（density 460ms 等） | scout-map L82/L112、07 篇 L77-84 | ⚠️ 独立污染源（探针自身开销），非 notify bug | 已揭穿（L112）；07 篇 L77-84 计数保留但耗时列作废 |

## 3. 不受影响（真实数据）

- **单线程数据全部**：62.38 / 73.23 / 79.91 / 181→62ms 修复链（T=1 无补建，notify 无关）
- **H2 主因**（rebuild 36,252 = 168×）：单线程 WG_SPLINEDEBUG 精确统计，独立成立 ✅
- **8/12 FlatCache 修复闭环验证**（单线程 wall 2910ms、62.38ms/chunk）：单线程口径 ✅
- **mod 实机正确性**：notify bug 只影响性能假象，不产生错误方块（任务仍被执行，只是串行）✅
- **08-06 基线表**（池改造前）：不涉及池 ✅

## 4. 修复后验证（2026-08-16 重跑）

`bench-notifyfix-8x8-20260816.txt`（64 chunk 8x8, reps=2, seed 8576）：

```
[A] threads=  1   98.02 ms/chunk
[A] threads=  8   89.88 ms/chunk   （-8.3%：不再反降，轻度加速）
[A] threads= 12   90.39 ms/chunk
[A] threads= 22   97.76 ms/chunk
[B] workers=  1   86.80 ms/chunk（[B] 段 120s cap 截断，未跑完）
```

- **结论**：修复后 [A] T=8 不再反降（比 T=1 快 8%），但**远未到 8× 加速** →「每 chunk 并发下慢」仍存在（见下）
- ⚠️ 与 scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」**矛盾**——需核实 L110 那次运行是否 C1 版/计时污染/不同机器状态；本数据为 C1 回滚 + notify 修复后最终状态
- **[B] 段结构性串行**（见错误台账 #3）：count=1 + `threads>count` clamp → 池恒 1 worker——实机 M=1 模型多线程并行可能从未生效（candidate，待实机验证）

## 5. 遗留问题

1. scout-map L110 与本次数据矛盾（71.40 vs 98.02 单线程基差 +37%）——单线程波动大（49-67ms vs 44-46ms 已在复核清单），需同机同状态对照
2. **L110「修复后仍反降」数据版本不明**（教训实证）：71.40/84.24 与今天重跑（98.02/89.88 不再反降）矛盾——L110 未记录确切代码版本（C1 版？notify 修复前？）/时间/环境，无法判定。**这正是「错误未详细记录」的代价**：数据没带版本戳就下结论。以今天重跑（C1 回滚 + notify 修复最终态 8966ba9 之后）为准：[A] T=8 89.88 < T=1 98.02，不再反降
2. 「每 chunk 并发下慢 7.5 倍」真实性问题：WG_MTTRACE 的 fprintf stderr 锁竞争污染（L123）——需无 fprintf 计数器测量
3. H3 ×16 重新定性（修复后 mt 单次 spline 成本）
4. 实机 M=1 + clamp 结构性串行 → 实机并行从未生效？→ 需实机验证（CppBridge THREADS 传参 + wg_fill_blocks_multi count=1 路径）

## 6. 教训（一句话版）

> 多线程性能「反降」先查**线程池实现正确性**（notify/worker 就绪竞争），再查内存/调度/带宽——C1-C7 全排查是表象（scout-map L107）。错误不详细记录 → notify bug 活跃 9 天反复踩坑（本台账即是对此的纠正）。
