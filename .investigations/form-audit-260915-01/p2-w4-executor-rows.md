# P2-w4 — executor/线程放置图谱层错配矩阵行（跨段公共层）

- 工作块：260915-01（形态审计 P2 fan-out，w4 = executor/线程放置层）
- 角色：core-worker（对照 P1a/P1b 勘探产物 + 一手源抽核）
- 状态：**draft**（静态推演，验证分层 = Degraded/静态审查——无运行时 trace；所有量级标注假设）
- 日期锚：Get-Date 2026-09-15（工作块内）
- 一手源抽核补充：`wg/bench/mixin/NoiseChunkGeneratorMixin.java:60-253`（三分派臂 + 自有池定义逐行）、`worldgen-core/src/api.rs:24-183`（adaptive_threads + scope）、`.tmp/scout-260905-08/mcsrc/.../ChunkTaskPrioritySystem.java`（updateLevel/removeChunk/LevelPrioritizedQueue 语义确认：按 ticket level 排序 + 队列内可移除）。
- 分类法（同计划 §1）：【等价】形态不同但语义/性能等价 ｜【已修】已知错配已修 ｜【open】疑似错配 ｜【对齐】本就同形态。

---

## R1.【open】双池并存拓扑：自有池 + vanilla 共享池同机竞争（#122 池宽死参数家族再命中）

| 侧 | 形态 |
|---|---|
| vanilla | 单一物理 ForkJoinPool（main worker，宽 = clamp(cores-1,1,255)，asyncMode）承载 BIOMES/NOISE 体 + light 车道 + NBT 升级；SURFACE/CARVERS/FEATURES 在 worldgen 优先级车道内联（同池的逻辑队列） |
| 我方（exec 缺省开） | 自有池 `CoreSwap-Fill-N`：N=N=logical/2-2，LBQ(128)，CallerRuns，daemon；NOISE/surface/writeback 整链搬离共享池；BIOMES/light/序列化仍留在共享池 |

**错配代价推演（量级 + 假设）**：
1. **#122 家族命中确认**：`resolveExecPoolSize` / `resolveMaxInflight`（Mixin:105-116/151-164）与 Rust `adaptive_threads`（api.rs:24-44）三处共用「logical/2 - 2」启发式，来源 = 单机 bench（12 物理核最优），属同类死参数。**低核机退化**：4C8T → `max(1, 4-2)=… logical=8/2-2=2`（尚可）；**2C4T → logical=4/2-2=0 → clamp 1**：fill 吞吐被自有池钳到 1 线程，而 vanilla 共享池有 cores-1=1… 同为 1，2C 下等价；**4C（logical=4）→ 0→1 vs vanilla 3**：fill 主导段 wall 上限差 ~3×（假设：fill 是瓶颈段、共享池无其他饱和负载）。高核机（≥16 逻辑）该公式与物理核-2 收敛，退化不触发。
2. **线程总量**：自有池 N + 共享池 cores-1 同时存活（本机 24 逻辑：10 + 23 + 主线程 + IO 池）。可运行线程竞争同一组物理核，OS 负载均衡消化；由于 fill 已搬走，共享池只剩轻负载（BIOMES/light/NBT），**常态竞争代价 ≈ 可忽略**（假设：fill 占用 10 线程时共享池占用率 <30%）。缓存局部性：fill 的 Java 输出 buffer 是 ThreadLocal（P1a ⑤b），自有池固定 N 线程 ⇒ buffer 稳定复用，局部性**不劣于** vanilla FJP（FJP 线程集合也基本稳定）——此子项判【等价】。
3. 设计动因成立（#107/vivo 网格构建挤占修复，Mixin:118-123 注记）：搬离共享池是**有据已修**的错配修复——但池宽选择本身是新引入的 #122 同族参数。**净判定：open（子项 1），其余等价。**

## R2.【open】CallerRuns 回压 vs 优先级队列取消语义

- vanilla：`ChunkTaskPrioritySystem` 队列无界（Integer.MAX_VALUE）、按 ticket level 排序、`updateLevel`/`removeChunk` 支持**入队后重排/取消**（ChunkTaskPrioritySystem.java:101-133 一手核验）——玩家离开时未开跑的任务零成本撤销。
- 我方：LBQ(128) FIFO，**入队后不可取消、不可重排**；溢出时 CallerRuns 在 worldgen 车道线程内联执行被拒任务。

**错配代价推演**：
- **不可取消浪费**：快速移动场景 ticket 撤销时，我方已入队 ≤128+N 个 fill 仍会跑完。量级：128 排队 × ~50ms/chunk ÷ 池宽 10 ≈ **0.64s 纯浪费 CPU**（假设：单 chunk fill 50ms、池宽 10、瞬间全弃）。vanilla 同场景 = 0。绝对量小，但发生在首载/传送的敏感窗口。
- **CallerRuns 语义核对（本 worker 修正一个直觉性误判）**：提交序 = 优先级序（任务从优先级队列按 level 出队后才调 populateNoise → wgDispatch），故 LBQ 里是最早 = **最高**优先级的 128 个；被 CallerRuns 拒收的是**当前最新提交**（当下最高优先级）任务，在车道内联跑它本身不是优先级倒置，代价 = 车道被占 ~1 个 fill 时长（~50ms 量级，假设 count=1 单线程 fill），推迟的是其后**更低**优先级任务的出队。判：**常态无害，突发窗口车道阻塞有界**（judge C2 260911-03 已修正过的断言，本行维持其结论）。
- **运行中不可中断**：vanilla supplyAsync 体同样不可中断 ⇒ 此子项【对齐】。
- 净判定：open（不可取消浪费 + 溢出车道阻塞），量级均 ≤ 秒级/50ms 级，属低嫌疑。

## R3.【open】ticket level 优先级在我方自有池的丢失——G3 首载漂移 7× 家族推演

核心差异：vanilla 未开跑任务随 ticket level 变化**持续重排**（玩家附近 chunk 永远插队）；我方 FIFO 一次定序，入队后 level 变化无效。

**互斥归因竞争（≥2 分支，各出上限推演）**：

- **分支 (a) FIFO 队深延迟**：首载时 spawn 半径 chunk 与玩家视距 chunk 几乎同时入场，提交序近似 ticket 建立序而非最终优先级序；近玩家 chunk 最坏排在 128+N 深度队尾。上限：(128+10)/10 ≈ **13.8× 单 chunk 延迟**（最后完成的近玩家 chunk 相对 vanilla 插队首位的延迟倍数；假设：全队列等价 chunk、池均匀排水、50ms/chunk ⇒ 最坏 ~0.69s 额外等待）。**量级足以解释 G3 7× 漂移的大部分**——但依赖「提交序与最终优先级序显著错位」假设（未验证）。
- **分支 (b) 优先级重排缺失**：玩家移动过程中 frontier chunk 的相对优先级动态变化，vanilla 持续把新最近 chunk 推到队首；我方已入队的旧远 chunk 不会让位。上限：同 (a) 的队深量级，但发生在**移动中**而非首载瞬间——对 G3（首载漂移）贡献应为次要。
- 两分支非互斥排除项：光照/writeback 车道（R5）也会造成首载串行漂移——归 w-light/w-writeback 行，本行不抢归因。

判定：open，**嫌疑度高**（G3 家族最强候选之一）；上限 ~13.8×、典型预计 <7×；需运行时 trace（首载期 fill 完成序 vs chunk 距玩家距离）定夺——静态层无法闭合。

## R4.【等价】Rust per-call `thread::scope` 每 chunk 线程域开销

一手核验（api.rs:143-181）：生产路径 count=1 → `adaptive_threads` clamp 到 1 → `thread::scope` spawn **恰好 1** 线程/chunk（260911-05 A1d 已消除 10 线程 9 空转风暴）。

**量级推演**：Windows 线程创建+join ≈ 20-50µs；每调用额外堆分配 = Arc<&handle> + SendOut Vec ≈ 2 alloc ≈ 0.1µs 级。对照单 chunk fill ~50ms ⇒ **开销占比 0.04-0.1%**，判等价。残余优化点（nthreads==1 时跳过 spawn 直接调）收益 <0.1%，登记不修级别。批量 ABI（count>1）交错分块形态与 vanilla 无对应物（vanilla 无批概念），不构成错配。

## R5. 各段线程归属表（我方 vs vanilla）

| 段 | 我方实际线程 | vanilla | 判定 |
|---|---|---|---|
| BIOMES | （未接管）共享 FJP worker | 共享 FJP worker（硬编码 main worker，NoiseChunkGenerator.java:87-92） | 对齐 |
| fill/NOISE 体（含 density/aquifer/ore） | CoreSwap-Fill-N（自有池）；CallerRuns 时 worldgen 车道；exec=0 时共享 FJP + P1 信号量；syncfill 时车道内联 | 共享 FJP worker（supplyAsync "wgen_fill_noise"） | 【已修】#107 搬离共享池（有据）；exec=0 臂 = 对齐 vanilla + 限流 |
| SURFACE | Rust fill 管线内（worldgen_handle.rs:824-834），随 fill 同线程 = CoreSwap-Fill-N | worldgen 优先级车道线程内联（ChunkStatus.java:114-119） | 形态不同（线程池 vs 车道）但语义等价【等价】——两者都是「某个生成 worker 线程内联」 |
| CARVERS/FEATURES/SPAWN | （让位）worldgen 车道 | 同左 | 对齐 |
| light（gate 关=缺省） | vanilla：light 车道（同物理池，传播串行批 ≤1000 混批） | 同左 | 对齐 |
| light（gate 开） | **调用线程内联全链**（Mixin:421-501 同步），调用线程 = ChunkStatus.LIGHT 任务的 worldgen 优先级车道线程 ⚠️（P1a §3.3-1 未核实具体线程名，但 generationTask 默认落点 = worldgen 优先级 executor，P1b §1） | light 独立逻辑队列 + 批处理，**不占 worldgen 车道** | 【open·高嫌疑】我方把 3×3×384 全量重算（~9 chunk 输入的重活）**同步压在 worldgen 优先级车道上**，每个 light chunk 阻塞车道一个全量重算时长 ⇒ 其他 chunk 的 generationTask（含高优先级近玩家 chunk）全部排队。量级：车道每 light chunk 阻塞 ≈ light_compute 时长（假设数十 ms 级）× 首载 chunk 数；与 R3 分支叠加可放大 G3 漂移。另 P1a §2.3② 记录的 RefCell 并发 UB 面在此形态下**反而不触发**（车道串行化是事实上的锁）——但代价是串行 |
| writeback（blocks→sections+heightmap） | 随 fill 同线程（CoreSwap-Fill-N / CallerRuns 车道），BulkWb per-thread TL | populateNoise 体内 = 共享 FJP worker；FULL 转换/serialize = 主线程（我方同让位） | 【等价】写者线程类别一致（生成 worker），读者（主线程序列化）两侧相同 |

---

## 汇总

- **错配行数**：5 行（R1 open / R2 open / R3 open·高 / R4 等价 / R5 表内含 1 open·高 = light 车道内联 + 2 等价 + 其余对齐/已修）。
- **open 嫌疑摘要**（按嫌疑度降序）：
  1. **R3 ticket 优先级丢失 + FIFO**（G3 首载 7× 家族最强候选，上限 13.8×，需首载 trace 闭合；双分支 (a) 队深延迟 / (b) 动态重排缺失 上限同量级）。
  2. **R5-light 车道内联全量重算**（gate 开时 worldgen 车道被逐 chunk 阻塞，与 R3 可叠加放大首载漂移；侧面「消除」了 RefCell 并发 UB）。
  3. **R1 #122 池宽死参数三处同族**（logical/2-2，低核机 fill 钳 1 线程，上限 ~3× @4C；高核机不触发）。
  4. **R2 不可取消 + CallerRuns**（浪费 ≤0.64s/突发窗口、车道阻塞 ~50ms 有界，低嫌疑，登记级）。
- 产物路径：`.investigations/form-audit-260915-01/p2-w4-executor-rows.md`（本文件，draft）。
- retry 轮次：1（首轮静态对照，无 evidence saturation 消耗；建议闭合手段 = 首载期 fill 完成序 trace + jstack 车道占用采样）。
