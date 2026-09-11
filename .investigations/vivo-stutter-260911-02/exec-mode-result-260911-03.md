# exec 模式（自有有界执行器）测量记录 — 260911-03

> 状态：**candidate**（本地测量完成，judge 待跑，用户实机 FPS 数据未回收）
> 执行体：dll `dd3b645f…`（零改动）；Java 侧 `NoiseChunkGeneratorMixin` 新增 exec 分派 + 缺省开（preview 起）。
> §9.7 口径声明：载体 = Chunky radius 500（4225 chunks，seed 417950215108767439，同驱动 run_ab.ps1 同日串行三臂）；与 260911-02 的 cap 曲线为**跨 run 数字，只作趋势参照不作裁量**（#51）；本表内三臂同 run 族可直接比。

## 结果

| 臂 | 配置 | wall | cps | cpuSec | fill 均/chunk | maxWsMB |
|---|---|---|---|---|---|---|
| cap0-fresh | shared 池不限（同日基线） | 40 s | 105.6 | 488 | 102.8 ms | 3957 |
| **exec-def** | `-Dcoreswap.exec=1`，池宽缺省 10（logical/2−2） | **35 s（−12.5%）** | **120.7** | **455（−6.8%）** | **70.1 ms（−32%）** | 6034 |
| exec-16 | `execpool=16` | 35 s | 120.7 | 492 | 87.2 ms | 3875 |

- 行为门：三臂 `[WG-CONTENT]` 指纹各 4140 条，sorted diff = **0**（逐条全同）。
- 行为化自证：`[WG-EXEC] mode=own-pool pool=N queue=128 rejectedPolicy=CallerRuns` 三臂均命中；缺省开 sanity（零 env）boot 命中 `pool=10` 且无 `[WG-INFLIGHT]` 行（exec 分支短路信号量路径）。
- 池宽敏感性：**10 优于 16**（fill 70 vs 87 ms，cpuSec 455 vs 492，wall 持平）——24 逻辑核机上 fill 池 10 + 其它工作占用的结构更优，与 260911-02 超订曲线「拐点在 12 物理核边界」一致。

## 与 P1（信号量限流）对比（本地口径）

- P1 cap12（260911-02，跨 run 趋势参照）：fill −23%，wall 无损。
- exec-def（本次，同 run 族 vs 基线）：fill −32%，**wall −12.5%**，cpu −6.8%。
- 机制差异：P1 fill 仍占共享池线程槽（只限个数）；exec 把 fill 整体搬出自有池 ⇒ ChunkBuilder 网格构建拿回全池（vivo 低帧根因 = 共享池争用，javap 已证同池 + 用户光影 mod 缓解观察）。

## 交付物

- preview jar：`.tmp/vivo-stutter-260911-02/coreswap-1.20.1-1.0.28-p2-exec-preview.jar`
  sha256 = `ec21b650…`（全值见提交/工单时补全）；jar 内 dll = target = `dd3b645f…` **三元组 MATCH**。
- 缺省：exec **开**（`-Dcoreswap.exec=0` 回退 shared+P1）；`execpool` 缺省 10。
- 代码：`versions/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`（EXEC_MODE/WG_EXEC_POOL_SIZE/wgOwnPool/wgDispatch 分支）。

## 待办 / 边界

- perfprofile 临时计时器**仍在**（默认关）——正式出单前 MUST 移除（继承 260911-02 纪律）。
- P1 信号量代码保留（`maxinflight` 缺省值决策悬置：exec 转正后 P1 变回退路径）。
- 用户实机待回收：最低 FPS + 主观分（preview vs 1.0.28，vanilla 渲染器无光影）。
- 内存注记：exec-def maxWs 6.0 GB（vs 基线 3.9 GB）——自有池线程栈+队列 chunk 常驻，vivo 集成服内存敏感场景需在用户实机观察。
