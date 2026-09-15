# P2-W1 — A 段（NOISE/fill）+ B 段（SURFACE）执行形态错配矩阵行

- 工作块：260915-01（形态审计 P2 fan-out worker W1，计划 000-260914-04b）
- 角色：core-worker（对照 P1a/P1b scout 产物 + 一手源核对）
- 输入：`p1a-coreswap-forms.md` / `p1b-vanilla-forms.md`；核对源：`worldgen-core/src/api.rs:24-44,134-183`、`versions/1.20.1/java/.../mixin/NoiseChunkGeneratorMixin.java:39-235`（grep 核对，行号见下）
- 状态：**draft**
- 验证分层：**Degraded（静态源码对照 + 量级推演；无运行时测量）**——所有代价推演的数值均为假设推演，假设已逐条注明
- 辖区：仅 fill（A）+ surface（B）；carver/features/light/writeback 证据只标「归属建议」（#73 让渡清单）

---

## 0. 行分类统计

- 总行数：**11**（A 段 6 + B 段 5）
- 已对齐：3（A-④、B-③、B-④）
- 已知错配已修（#107 单车道 6.64× 相关）：1（A-①，含 A-② 的历史修复注记）
- 疑似错配（open）：2（A-② 主嫌疑、A-⑤ ChunkNoiseSampler 重建嫌疑〔归属建议〕）
- 形态不同但等价：5（A-③、A-②b、B-①、B-②、B-⑤）

---

## 1. A 段（NOISE/fill，populateNoise 接管）矩阵行

### 行 A-① 同步性/异步

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 异步单跳 future：`supplyAsync("wgen_fill_noise", …, Util.getMainWorkerExecutor())`（NoiseChunkGenerator.java:330-357），section 先 `lock()`、`whenCompleteAsync(unlock, worldgen优先级executor)` 收尾（:348-355）——**unlock 回优先级队列排序执行** |
| 我方形态 | 异步单跳 future：拦截点立即返回 pending future，重活（feedBeardifier+fill+writeChunk）在 work lambda 内异步（Mixin:298-327；supplyAsync(wgDispatch)，Mixin:227-253）；完成即 future 完成，writeChunk 同线程顺带（CppBridge.java:494-500） |
| 分类 | **已知错配已修（#107 单车道 6.64× 相关）**——Mixin:39 注释明载「上同步执行」→「getMainWorkerExecutor() 上异步执行，把重活移出 worldgen 单车道」，后演进为自有池（260911-03 EXEC_MODE 缺省开） |
| 等价论证 | 语义等价：双方向调用方（ChunkStatus.NOISE 任务）返回 future、由 MC 状态机阻塞等待，完成时序语义同。**残余微差**（不构成错配）：① vanilla 的 unlock 收尾经优先级 executor 重排序（section 锁释放与后续任务排序耦合）；我方无 section 锁概念（整块一次性写回，BulkWb 整段替换），无此需求 ⇒ 差异被写回形态吸收（writeChunk 等价性已由 CppBridge.java:612-615 论证：接管点=HEAD cancel，chunk 全新，无部分填充态需解锁）。② vanilla future 收尾回 worldgen 队列、我方在 fill 池线程直接完成——只影响后续任务落点线程，不影响结果。 |
| 置信度 | 高（静态对照；#107 修复记录 + 6.64× 数据既有） |

### 行 A-② 线程放置与池归属【主 open 嫌疑，含 3 个互斥归因候选】

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 硬编码 `Util.getMainWorkerExecutor()`（ForkJoinPool，宽 = clamp(cores-1,1,255)，asyncMode=true，Util.java:182-231）；任务投递经 ChunkTaskPrioritySystem——**按 chunk ticket level 排序 + 支持取消**（TACR:637-680, :643；ChunkTaskPrioritySystem，队列上限 MAX_VALUE）。同池还载 BIOMES supplyAsync / NBT 升级 / ChunkBuilder 网格构建 |
| 我方形态 | 自有有界池 `wgOwnPool`：`ThreadPoolExecutor(N,N,60s,LinkedBlockingQueue(128),CallerRunsPolicy,daemon)`（Mixin:174-197），N 缺省 = `logical/2-2`（Mixin:151-164；本机 24 逻辑核 → N=10）；FIFO 队列、无优先级、无取消；队列满 128 → CallerRuns 回退调用线程（worldgen 车道）内联。exec=0 回退臂 = vanilla 共享池 + P1 信号量（Mixin:86-116/243-252），缺省不启用 |
| 分类 | **疑似错配（open）**——池宽/优先级/取消三轴与 vanilla 均不同形态；代价归因存在互斥候选竞争（下 b1/b2/b3），需区分实验裁决 |
| 代价推演（互斥候选，各出上限） | **b1 稳态吞吐（池宽归因）**：我方 N=10 vs vanilla FJP 宽 23（本机 cores-1）。若 fill 为 CPU-bound 瓶颈且需求满载（传送/大量预生成），稳态并发上限差 **≈2.3×**（23/10）。假设：① fill 单 chunk CPU-bound 无锁串行段 ② 到达率足够饱和 ③ Rust 单 chunk fill 与 Java 同量级耗时（未测量，T_r/T_j 未知——若 T_r ≪ T_j，实际需求并发远低于 10，b1 不触发）。**b2 排队延迟/不可取消（优先级归因）**：玩家移动/传送时 vanilla 高 ticket chunk 插队 + 弃 chunk 任务可取消；我方 FIFO queue(128) → 新优先 chunk 最坏排队 **128×T_fill**（假设 T_fill≈10ms 量级〔未测量，声明〕→ 最坏 ~1.3s 额外延迟）；unload 弃置 chunk 的已入队 fill 不可取消 → 浪费算力上限 **≤128×T_fill**。**b3 CallerRuns 回压（过载归因）**：queue 满时 fill 在 worldgen 车道内联执行 → 车道被占 T_fill/任务，后续所有 ChunkStatus 任务（含让位段 carver/features 的调度）排队——**#107 前形态的局部重演**；触发条件 = 爆发负载（b1 的饱和场景），最坏持续到队列 drain。**互斥性说明**：三者作用于不同现象轴（稳态吞吐 / 响应延迟 / 过载车道阻塞），但若只观测到一个「e2e 慢/卡顿」总量，归因互斥竞争。**区分实验建议**（供 P3 排序，非本 worker 辖区）：固定负载稳态吞吐 → 裁 b1；移动/传送场景首 chunk 延迟 → 裁 b2；queue 水位监控 + CallerRuns 命中计数 → 裁 b3。 |
| 置信度 | 中（形态差异为代码事实·高置信；代价量级为推演·低-中，T_fill 无实测） |

### 行 A-②b Rust 内并行形态（per-call thread::scope）

| 字段 | 内容 |
|---|---|
| vanilla 形态 | fill 体跑在常驻 ForkJoinPool worker 上，任务级零线程创建开销 |
| 我方形态 | 每次 JNI 调用 `std::thread::scope` 重建线程域（api.rs:158-181）；生产 Java 侧单 chunk/次（count=1）→ `adaptive_threads` clamp 到 1（api.rs:38-44，260911-05 A1d 修正：消除 count=1 时 spawn 物理核-2 个线程、9 个空转退出的风暴）→ **每 chunk spawn 1 线程 + join**；另每调用 `Arc::new(h)` + Arc 克隆（api.rs:154-163，atomic refcount 数次） |
| 分类 | **形态不同但等价** |
| 等价论证 | 量级推演：Windows 线程 create+join ≈ 20-60µs（CreateThread+TLS+调度，推演值）；Arc 分配/refcount ≈ 1µs 级。对比 chunk fill 主体（density 插值+aquifer+surface，ms 级；假设 T_fill ≥ 5ms〔保守下限假设〕）→ 附加开销 **<1-2%**。确定性无影响（每线程写固定 out，api.rs:129-132 注记）。结论：单 chunk 粒度下 per-call scope 与常驻池的差额被 fill 主体淹没；仅当批量化（count>1）时 scope spawn N-1 线程/批仍远小于 vanilla FJP 常驻形态的差距，但同属 <2% 量级 |
| 置信度 | 中-高（形态为代码事实；开销占比为推演，依赖 T_fill 下限假设——若 T_fill 实测 <1ms 需复核） |

### 行 A-③ 批粒度

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 1 chunk / generationTask（ChunkStatus 状态机粒度，ChunkStatus.java:346-371） |
| 我方形态 | 1 chunk / JNI 调用（CppBridge.java:459-460 单元素数组）；多 chunk 批量 ABI（wg_fill_blocks_multi count>1）存在但生产不用 |
| 分类 | **形态不同但等价**（粒度同为 1 chunk，差异仅在「批量 ABI 闲置」这一未启用通道） |
| 等价论证 | 双方任务粒度一致，无调度形态错配。JNI 固定开销（参数编组+跨界）µs 级 vs fill ms 级，占比可忽略（同 A-②b 假设）。**附注（登记不判错配，归 P3 候选池）**：批量化是 terrain_cache 读路径的激活前提——count=1 时 ca_min 门控下 terrain_cache 只写不读被跳过（P1a §2.1⑤c，worldgen_handle.rs:136-146/620-626）；批量化 + 邻 chunk 缓存读是已知可选优化通道，属收益项而非错配 |
| 置信度 | 高 |

### 行 A-④ 增量化 vs 全量

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 全量：整 chunk 无增量噪声（BelowZeroRetrogen 特例 ChunkStatus.java:100-108 不在接管路径） |
| 我方形态 | 全量：整 chunk 16×16×384 从 density 起整块生成（P1a §2.1④）；跨 chunk 状态 = aquifer split 种子纯函数重导 + beardifier 显式喂入 |
| 分类 | **已对齐** |
| 等价论证 | 双方同为全量单遍；跨 chunk 依赖的等价性已由 beardifier 喂入（A'）与 aquifer 纯函数重导覆盖（既有验证链，aquifer 对齐历史不在本行重证） |
| 置信度 | 高 |

### 行 A-⑤ 缓存层

| 字段 | 内容 |
|---|---|
| vanilla 形态 | **ChunkNoiseSampler 挂 chunk**（Chunk.java getOrCreateChunkNoiseSampler，NoiseChunkGenerator.java:94-100）：NOISE 阶段首次创建（缓存写），SURFACE/CARVERS/FEATURES 全部复用（缓存读）——跨阶段状态（noise sampler、aquifer、双 WG heightmap）一物贯通 |
| 我方形态 | 无 ChunkNoiseSampler 对应物：fill 单遍内完成 noise+surface+heightmap（无跨阶段复用需求，A 段内自洽）；Rust 侧缓存 = est_l2（aquifer 二级缓存）、carver/feature cache、噪声 key 预载表；Java 侧 = STATE_BY_ID / per-thread 输出 buffer / 反射缓存（P1a §2.1⑤a-f） |
| 分类 | **疑似错配（open）——但主要代价落在辖区外** |
| 代价推演 | 我方 cancel 掉 populateNoise 与 buildSurface 后，chunk 上的 ChunkNoiseSampler **从未创建**；而让位段 **vanilla CARVERS 仍要它**（carve 用 ChunkNoiseSampler 的 aquifer 交互，NoiseChunkGenerator.java:297，P1b §3.4）→ CARVERS 首次调用 getOrCreateChunkNoiseSampler 触发**全量重建**（cell 采样器整套），只为 carver 服务一次。代价量级 = 一次 sampler 构建/chunk（推演：与 NOISE 阶段采样器初始化同量级，占 fill 总时长的分数级；无实测，声明推演）。次级影响：FEATURES 段 vanilla 读 heightmap（我方已补 6 种，CppBridge.java:660-666，覆盖）。**归属建议**：此行的验证与定论归 carver/features 让位段 worker（#73 让渡清单）——本 worker 只从 fill 侧缓存维度登记「我方未喂 chunk 级缓存、下游 vanilla 重建」这一事实 |
| 置信度 | 中（ChunkNoiseSampler 重建路径为源码事实·高；代价量级未测·低） |

---

## 2. B 段（SURFACE，buildSurface 纯 cancel）矩阵行

### 行 B-① 同步性/异步

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 同步内联：SimpleGenerationTask 直接在 worldgen 任务线程执行 buildSurface（ChunkStatus.java:114-119 → NoiseChunkGenerator.java:242-276），复用 chunk 上 ChunkNoiseSampler（:261-263） |
| 我方形态 | 同步零工作：`ci.cancel()`（Mixin:415）纯跳过；实际 surface 计算前移至 A 段 Rust fill 管线内（worldgen_handle.rs:824-834，FLAG_SKIP_SURFACE 门控 :825），在 fill 池线程上随 fill 一遍完成 |
| 分类 | **形态不同但等价** |
| 等价论证（时序等价性重点核对） | vanilla 时序：NOISE future 完成 → SURFACE 任务独立调度（读 NOISE 产物：ChunkNoiseSampler + est + heightmap + biome jitter）→ CARVERS。我方时序：surface 在 fill 内完成 → writeChunk → future 完成 → SURFACE 任务 cancel（空跳）→ CARVERS。等价性论证：① **输入确定性**——surface rule 判定输入（est 列、heightmap、surface noise、biome jitter）全部是 chunk 坐标的确定函数，在 fill 遍内可同遍取得（已实现 biome_at_surface jitter，worldgen_handle.rs:817-820），不依赖 NOISE 之后、SURFACE 之前任何外部状态突变（此窗口内无第三方写者：ChunkStatus margin 隔离保证邻域任务不写本 chunk）⇒ **前移不改变输出**。② **无锁/无共享冲突**——同一 chunk 串行（fill future 未完成则 SURFACE 任务不调度）。③ 顺带收益：省一次 pipeline 任务 hop（vanilla SURFACE 的队列往返）。残余微差：vanilla surface 与 carver 之间理论上的观察点（mod hook）消失——非本项目关切。 |
| 置信度 | 高（时序论证基于 ChunkStatus margin 隔离 + 输入确定性，静态可证；输出对齐另有既有 block_probe 验证链兜底） |

### 行 B-② 线程放置与池归属

| 字段 | 内容 |
|---|---|
| vanilla 形态 | worldgen 优先级队列线程（ChunkTaskPrioritySystem 排序后执行） |
| 我方形态 | 无独立线程动作；实际计算在 A 段 fill 池线程（CoreSwap-Fill-N）内 |
| 分类 | **形态不同但等价** |
| 等价论证 | 计算落点线程从 worldgen 队列线程移到 fill 池线程——无共享状态（见 B-① ②），无优先级交互（surface 无独立任务排队需求，其完成被 fill future 覆盖）。唯一耦合：fill 池过载（A-②b3 CallerRuns）时 surface 随 fill 一起承受回压——该代价已计入 A-② 行，不在本行重复计 |
| 置信度 | 高 |

### 行 B-③ 批粒度

| 字段 | 内容 |
|---|---|
| vanilla / 我方 | 双方均为 1 chunk（我方随 A 段 fill 粒度，逐列 surface rule 判定） |
| 分类 | **已对齐** |
| 等价论证 | 粒度同构，无批形态差 |
| 置信度 | 高 |

### 行 B-④ 增量化 vs 全量

| 字段 | 内容 |
|---|---|
| vanilla / 我方 | 双方全量：每列判定 surface rules，无增量（输入含 est/heightmap/biome，本 chunk 全量） |
| 分类 | **已对齐** |
| 等价论证 | 同为全量单遍；我方在 fill 同遍内做，还省了 vanilla 的 ChunkNoiseSampler 复用读取一趟（结果不变，见 B-①） |
| 置信度 | 高 |

### 行 B-⑤ 缓存层

| 字段 | 内容 |
|---|---|
| vanilla 形态 | 读 chunk 级 ChunkNoiseSampler（NOISE 阶段写入的跨阶段缓存） |
| 我方形态 | biome_at_surface = BiomeAccess jitter 采样（worldgen_handle.rs:817-820）+ surface 噪声 key 预加载表（surface_rules.rs ENGINE_NOISE_KEYS，worldgen_handle.rs:160-165）+ est_l2 等 A 段缓存共用 |
| 分类 | **形态不同但等价** |
| 等价论证 | 功能对应物齐全：vanilla 的「复用 ChunkNoiseSampler」在我方形态下无意义（surface 与 noise 同遍同线程，中间态直接在寄存器/栈上传递，无需 chunk 级缓存物化）；跨调用缓存（噪声 key 预载、est_l2）为进程级等价物。无丢缓存导致的重复计算 |
| 置信度 | 高 |

---

## 3. open 嫌疑摘要（供 P3 汇总）

1. **A-② 池/优先级/取消三轴错配（主嫌疑）**：自有池 N=logical/2-2 + FIFO queue128 + CallerRuns + 不可取消，vs vanilla FJP(cores-1) + ticket-level 排序 + 可取消。互斥归因候选：b1 稳态吞吐上限差 ~2.3×（饱和场景）/ b2 新优先 chunk 最坏排队 128×T_fill + 弃 chunk 算力浪费（T_fill 未实测，假设 ~10ms → ~1.3s）/ b3 CallerRuns 回压局部重演 #107 形态（爆发负载）。需区分实验裁决。
2. **A-⑤ ChunkNoiseSampler 重建（归属建议 → carver/features 让位段 worker / #73）**：我方接管 NOISE+SURFACE 后未建 chunk 级 sampler，vanilla CARVERS 被迫全量重建只为 carve 服务——辖区外定论，本行只登记事实。

## 4. 自检声明

- retry 轮次：1（首轮静态对照即闭合，无数据层采集需求——本任务为形态对照非机制排查）。
- 所有代价数值均为推演且已注明假设（T_fill 未实测为最大假设依赖，A-②b1/b2/A-②b 置信度因此压至中/低-中）。
- 辖区外证据（ChunkNoiseSampler 在 CARVERS 的重建代价、light/回退臂 exec=0 形态）只标归属建议，未越权定论。
- 未写 confirmed；status: draft 待 P3 汇总 + P4 judge。
