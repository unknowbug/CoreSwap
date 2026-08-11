# 剩余课题调查记录：多线程无加速 + spline 单次 8µs（2026-08-12）

> 状态：🔍 未结案（FlatCache 修复闭环后剩余独立课题）
> 数据来源：pool_test.exe（CMake target，src/pool_test.cpp 临时诊断）、bench_8x8_noprof.txt、wgprofile_8576_t1_ctx.txt

## 1. 现象（实测数据）

| 测量 | 单线程 | 多线程 | 期望 | 实际 |
|---|---|---|---|---|
| pool_test 32 chunks | T=1: 90.60ms/chunk | T=2: 76.55 / T=4: 72.70 / T=8: 82.21 | 4-8× 加速 | **仅 1.25×，T=8 反降** |
| bench 64 chunks | T=1: 62.38ms/chunk | T=8: 62.17ms/chunk | 8× 加速 | **完全无加速** |
| block_probe 36 chunks（无 profile） | 94.04ms/chunk | 8 线程 87.61ms/chunk | 8× | 仅 7% |
| spline 单次（WG_PROFILE t1） | 7,971ns | — | 旧基线 992ns | **8× 慢** |
| concTest 8 线程 640 chunks | — | 49.8s（78ms/chunk） | ~6s | **≈完全串行** |

**关键结论**：
1. 池逻辑正确（CoreSwapPool 任务队列模型，RQ-001 已实施）——问题在 fillOneChunkCore 内部
2. 多线程几乎无加速（1.25× max）→ **fillOneChunkCore 内部存在跨 chunk 串行/争用点**
3. spline 单次 7,971ns vs 旧基线 992ns = 8× 慢 → **spline 树采样路径本身退化**（非 FlatCache 缓存问题，修复前后单次均高）

## 2. 已排除的假设

- ❌ **WG_PROFILE 多线程数据（原子争用假象，显式排除）**：wgprofile_8576_mt_ctx.txt 显示多线程单 chunk density=758ms（单线程 44ms，17×）——此为 profile 计时器污染：SplineDF::sample 每次采样都 `wg_profSplineNs.fetch_add` 原子操作，多线程 21 万次争用同一原子变量 → cache-line ping-pong 假象。**真实性能以无 profile bench 为准**（62.17ms/chunk 8 线程 vs 62.38 单线程）。
- ❌ **CRT 堆分配锁**：alloc_test 8 线程 0.07ms/iter vs 单线程 0.02ms——分配非瓶颈
- ❌ **池 worker 数不足**：ensure(poolThreads) 逻辑正确（L1053），concTest 8/8 线程全部批次成功
- ❌ **beardifierMtx**：SURFACE 模式 beard 空 map，锁内只 find（L665），快
- ❌ **splitterFor 锁**（surface.h L153）：derivedSplittersMtx 是 SurfaceContext 成员（每 chunk 局部对象），不跨线程争用
- ❌ **regionColsMtx/pendingCrossMtx**：SURFACE 模式不触发（storeRegion=false）

## 3. 待验证假设（candidate）

- **H-A：spline 树构建/采样路径退化**（8/6 优化链引入）：spline 单次 8µs 修复前后均存在 → 8/6「纯算法优化」提交（86e4057）之后某个提交改了 spline 树结构或采样实现。多线程无加速可能同源（spline 树共享 mutable 状态？）
- **H-B：fillOneChunkCore 内隐藏全局写**：densityBuf 循环 L673-681 的 h->finalDensity->sample 树内是否有非 thread_local 的共享 mutable（已查 FlatCacheDF/Cache2DDF/InterpolatedDF 均为 thread_local；未查 SplineDF 子树内的 noise sampler 等）
- **H-C：MEM-CHK 诊断**（L592-611）：每 chunk GetModuleHandleA + IsBadReadPtr + static 变量读写——static baseline/haveBase **无锁多线程读写 = 数据竞争（UB）**，且每 chunk 一次系统调用，疑低效但非主因

## 4. 建议下一步

1. **git 二分 8/6 优化链**（86e4057 之后 → HEAD）定位 spline 单次退化引入提交（用户已同意继续追剩余课题；二分曾因 FlatCache 主因拍板跳过，现主因已修，剩余课题需二分）
2. 检查 SplineDF 子树（noise sampler、shifter）是否有共享 mutable 状态（H-B 静态排查）
3. MEM-CHK static 数据竞争修复（RAII/atomic，低优先级）

## 6. 2026-08-12 补充调查（git bisect 不可行 + spline 单次假象）

### 6.1 git bisect 不可行（实证）
- **data 目录不在 git 中**（`git ls-files versions/1.20.1/data` 空；86e4057 ls-tree 也无）——data 是本地外部资源
- checkout 86e4057 后强制 /utf-8 /DNOMINMAX 编译成功，但运行 `wg_create` 失败（"The system cannot find the path specified"，无 wg_create failed 输出）——旧提交代码期望的 JSON 结构/路径与当前 data 不兼容
- **结论**：无法为 bisect 候选提交提供匹配的 data → git bisect 不可行（NEXT_SESSION 待办 1 的二分方案需废弃/换数据驱动）

### 6.2 spline 单次 8µs 假象（86e4057 vs HEAD 代码对比）
- 86e4057 版 `SplineDF::sample`：仅 `wg_profSpline.fetch_add(1)`，**无耗时计时**
- HEAD 版：`wg_profSplineNs` 计时（每层 2×steady_clock::now() + 2×fetch_add，递归嵌套累加）
- **结论**：07 篇 992ns 是纯采样耗时口径；HEAD 的 7,971-9,735ns 含计时器开销（嵌套累加）→ **spline 单次 8µs 主要（或全部）是 WG_PROFILE 计时器污染假象**，非真实退化
- 真实性能以无 profile bench 为准：单线程 62.38ms/chunk（修复前 181ms，3× 改善）

### 6.3 剩余课题重估
- **density 阶段 44-47ms/chunk（真实 wall，旧 8.5-11.7ms，4× 慢）才是剩余主瓶颈**：
  - aquifer+oreVein 修复前 125-166ms → 修复后 26-35ms（4-5× 改善，**FlatCache 修复已解决 aquifer 慢**）
  - density 修复前后几乎不变（44-47ms）——spline 调用量已回基线（5,906/chunk）但 density 仍慢
  - 构成未明：疑 InterpolatedDF buildGrid 网格角点采样（6 实例 × 每 chunk 网格角点数）或 profile 污染残留
- **spline 单次 8µs 假象确认**：86e4057 版无耗时计时器，HEAD 版 wg_profSplineNs 嵌套累加 → 7,971-9,735ns 含计时器污染；spline 总计时 57.5ms/chunk > density wall 44ms 即证据（递归嵌套重复计时）
- **多线程无加速仍为真问题**（无 profile bench 8 线程 ≈ 单线程 62ms/chunk；pool_test T=4 仅 1.25×）——待定位 fillOneChunkCore 串行点
- git bisect 不可行（data 不兼容），替代 = 数据驱动（无 profile 分阶段计时 / perf 采样）或静态分析 density 阶段构成

## 5. 附：本次调查产物

- `.investigations/perf-rework/pool_test.cpp` + `alloc_test.cpp`（临时诊断，已注册 CMake target pool_test）
- `cmd-output/bench_8x8_noprof.txt`、`conctest8.txt`、`wgprofile_8576_t1_ctx.txt`
