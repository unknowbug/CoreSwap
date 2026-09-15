# P2-W2 — C 段（光照接管，gate `-Dcoreswap.light.rust`）执行形态错配矩阵行

- 工作块：260915-01（形态审计 P2，worker W2 / core-worker）
- 计划：`.investigations/000-架构设计/架构设计-260914-04b-形态审计.md`
- 输入：p1a-coreswap-forms.md §2.3/§3.1/§3.3 + p1b-vanilla-forms.md §2/§3.6 + light-round3 record/probe-verdict（260914-04）
- 状态：**draft**（AI 不授 confirmed；分类「open 疑似错配」均为候选，排序供主会话/judge 裁定）
- 辖界：只写光照辖区（LIGHT 接管面）；INITIALIZE_LIGHT 让位归 vanilla（无 mixin，行为无改动）；nibble 写回 enqueueSectionData 语义归光照正确性课题（12 篇遗留），本文件只在形态侧记一行。
- 验证分层：**Degraded（静态源码对照 + 既有 round3 探针数据引用）**——本 worker 未做任何新运行时采集；所有量级为推演上限，标 ⚠️。

---

## 1. 五维度逐项对照矩阵

### 维度① 同步性

| 项 | vanilla | CoreSwap | 二选一裁定 |
|---|---|---|---|
| 1a | `light()` 异步两段 future：PRE_UPDATE 只 enqueue（propagateLight 入 pendingTasks），POST_UPDATE 完成后 setLightOn + release ticket（ServerLightingProvider:171-184）——调用 worldgen 线程立即返回 | mixin HEAD 完全同步内联：收集→JNI→写回→setLightOn→releaseLightTicket→`completedFuture`（Mixin:423-501），**全部工作在调用 LIGHT 任务的线程上** | **open 疑似错配**（见行 L1） |

**行 L1（open 疑似错配 · 嫌疑 ★高）**：同步内联使每个 chunk 的全部光照成本（round3 后 ON 串行 2.94ms/chunk = collect 0.29 + native 2.65）**直接占住一条 worldgen 车道线程**；vanilla 形态下同一线程只付出「入队 + 完成回调」的纳秒级成本，传播在 light 队列串行消化、worldgen 线程立即可跑下一个 ChunkStatus 任务。代价推演（⚠️ 上限推演，未实测）：假设 worldgen 池宽 W、稳态 N 个 chunk 流水，我方形态把光照从「旁路消化」变成「车道内串行段」，单车道吞吐损失 = 2.94ms/chunk 的占用；这与 260914-04「串行反超 1.9× 但 e2e 反慢 7%」的形态错配签名一致（串行口径赢、并行口径输 = 调度/占线形态问题而非算力问题）。归因候选互斥分叉见 §2.1。

### 维度② 线程放置

| 项 | vanilla | CoreSwap | 二选一裁定 |
|---|---|---|---|
| 2a | light TaskExecutor（TACR:188）与 worldgen 同一物理 ForkJoinPool（Util.getMainWorkerExecutor，宽 clamp(cores-1,1,255)）上的**两个逻辑优先级队列**；传播串行单线程 | 无自有池、无 supplyAsync——落点 = 调用线程（ChunkStatus.LIGHT 任务车道，即 worldgen 优先级 executor 线程 ⚠️ 具体线程名 P1a 未核） | **open 疑似错配**（行 L2） |
| 2b | 单线程串行传播 = LightStorage/传播队列**天然无并发写** | 全局单例静态 lightHandle（Mixin:66）+ Rust `LightEngine.scratch: RefCell`（light/mod.rs:64）非线程安全；依赖「MC 是否并行调 light()」——而 ChunkStatus 任务在 FJP 上**可并行** | **open 疑似错配（UB 面）**（行 L3，特别裁定项 2） |
| 2c | LightStorage 写者=light 线程、读者=主线程（serialize/发包）+ worldgen 线程，经 uncachedStorage 快照隔离（LightStorage:276-294） | 写回 enqueueSectionData 直接从调用线程发起，读侧隔离语义沿用 vanilla 类（未改） | **形态不同但等价（论证）**：CoreSwap 未改 LightStorage 读写隔离结构，enqueue 本身线程安全入队；正确性残差（再传播语义）是 12 篇遗留 @anchor.idk，非形态错配，归光照正确性课题 |

**行 L2（open 疑似错配 · 嫌疑 ★★★最高）**：物理池相同、逻辑形态相反——vanilla 把光照放到**独立逻辑队列**（同池但经 ChunkTaskPrioritySystem 独立排序，worldgen 任务不被单个 chunk 的光照长任务粘住）；我方把光照**焊死在 worldgen 车道内**。同池意味着池宽是共享资源：我方每 chunk 占线 2.94ms × N chunk = 挤占同池 NOISE/FEATURES 任务的算力；vanilla 同样的传播工作也在同池跑，但 (i) 批 ≤1000 混批摊薄每任务开销、(ii) 优先级系统可插队、(iii) POST_UPDATE 先行让 chunk 流水不等传播细节。这是「e2e 慢 7%」的首席候选（与 L1 同体两面，互斥归因见 §2.1 .b2）。

**行 L3（open 疑似错配 · UB 面 · ⚠️ 未运行时验证）**：vanilla 传播**按设计串行**（单 "light" 队列 drain），LightStorage/队列无锁是建立在串行前提上的；CoreSwap 的 mixin 入口 = `ServerLightingProvider.light(Chunk, boolean)` 被 ChunkStatus.LIGHT generationTask 调用，该任务经 worldGenExecutor 投到 FJP（TACR:637-680）——**不同 chunk 的 LIGHT 任务可并行执行** ⇒ 多线程可同时进入同一静态 lightHandle 的 `lightComputePacked` → Rust `RefCell<Scratch> borrow_mut` 无同步 ⇒ **数据竞争 UB 面（静态推理成立）**。实际并发可达性评估：LIGHT 任务的 taskMargin=1 + 邻 chunk status≥FEATURES 门（Mixin 回退链）会收窄并行度，但无结构性串行保证（vanilla 的串行在 light 队列 drain 侧，不在 light() 调用侧——调用侧只是 enqueue）。修复形态候选：Mutex 包 RefCell（成本≈零，scratch 单次持有跨整个 compute）/ 或复刻 vanilla 串行队列。**注意：此 UB 面可能正是 G3 首载漂移 7× 的未排查共因**（见 §2.3）。

### 维度③ 批粒度

| 项 | vanilla | CoreSwap | 二选一裁定 |
|---|---|---|---|
| 3a | ≤1000 待办任务/chunk 间**混批**：runTasks 一批内全 PRE_UPDATE → 一次 doLightUpdates → 全 POST_UPDATE（ServerLightingProvider:195-218） | 单 chunk / 次调用，输入 3×3 邻域 9 chunk（blocks9 884,736 int / packed 216 节），输出仅中心 | **open 疑似错配**（行 L4） |

**行 L4（open 疑似错配 · 嫌疑 ★中）**：vanilla 混批把「跨 chunk 的传播合并成一次 BFS drain」——邻 chunk 已亮/待亮的传播边在同一批内自然收敛，不为 chunk 边界重复支付；CoreSwap 每 chunk 独立全量域，无批概念。代价推演（⚠️）：混批收益主要体现在**边界带重复传播**的摊销（vanilla 每 chunk 只为本 chunk 种子入队，边界越界传播被批合并）；我方以「每 chunk 重算整 3×3」的更粗粒度替代——量级上被维度④的 ×9 掩盖，单列修复价值低于④，但**批化（多 chunk 一次 JNI + 一次域合并）是④增量化之前的廉价中间形态**（修复形态候选：邻域共享计算 / 3×3 域中心批量产出 9 chunk 输出）。

### 维度④ 增量 vs 全量（核心）

| 项 | vanilla | CoreSwap | 二选一裁定 |
|---|---|---|---|
| 4a | 增量 BFS：blockPositionsToCheck + 增/减双队列；运行时改方块 checkBlock 单点登记（ChunkLightProvider:29-50,144-160） | 无对应面（无 checkBlock 接管；gate 只拦 light() 首亮路径）——运行时增量仍走 vanilla（gate 关时）或不可用（gate 开时 mixin 只管首亮，checkBlock 未动 ⇒ vanilla 增量机制仍在） | **形态不同但等价（论证）**：gate 只接管首亮（light() HEAD），checkBlock/setSectionStatus 未被 mixin 触及，运行时增量化保留 vanilla。⚠️ 但注意：首亮结果由我方 enqueueSectionData 写入后，vanilla 增量引擎在其上继续运作的正确性 = 12 篇遗留（G3/idk），归正确性课题非形态 |
| 4b | **首亮种子化**：block 光只把全部发光方块入队；sky 光读自身+4 邻 heightmap，heightmap 以上直填 15 **仅边界打包方向位入队**（ChunkBlockLightProvider:112-121 / ChunkSkyLightProvider:292-343）——每格平均 BFS 触达远小于全域 | **3×3×384 域全量重算**：fill opacity → block BFS → sky 柱状直落+种子 BFS → 导出中心（light/mod.rs:273-431），文件头自声明「全量重算无增量」 | **open 疑似错配（最大嫌疑）**（行 L5） |

**行 L5（open 疑似错配 · 嫌疑 ★★★核心 · 量级推演）**：

**总量 ×9 推演（⚠️ 上限口径）**：N 个 chunk 全亮，我方总计算 = N ×（48×48×384 全域 fill+BFS）= **9N 个 chunk 域**；vanilla 总计算 ≈ N ×（本 chunk 种子化 BFS + 直填 15 的 O(柱) 扫描 + 边界种子）≈ **N 个 chunk 域且每域实际 BFS 触达 ≪ 全域**（sky 直填不进队列、block 光源稀疏）。即**结构总量比 ≥ 9×，考虑 vanilla BFS 稀疏性后有效工作量比可达 ~9×–数10×**。

**与 round3 实测的拼合**：ON native 2.65ms/chunk（含 ×9 冗余）vs vanilla 整体 ~5.6ms/chunk（含调度/批开销）——我方单核算力优势（Rust 直排 vs Java 对象 BFS）吸收了 9× 结构冗余后仍串行反超 1.9×。**推论**：纯算力冗余在串行口径已被消化，e2e 7% 回退更可能来自调度形态（.b2）或依赖时机（.b3）而非剩余算力差——但 ×9 冗余是「算力预算被浪费」的上限证据，任何并行/池竞争场景下它直接乘进池占用（与 L2 复合）。

**修复形态候选（供候选池，不实施）**：(i) 3×3 域批中心化——一次计算产出 9 邻位输出（×9→×1，需邻域 status 同步点）；(ii) vanilla 形态复刻——heightmap 直填 + 边界种子 BFS + 跨 chunk LightStorage 式共享；(iii) 混批 JNI（多 chunk 一次调用摊固定成本）。

### 维度⑤ 缓存层（跨 chunk 光照状态复用）

| 项 | vanilla | CoreSwap | 二选一裁定 |
|---|---|---|---|
| 5a | LightStorage ChunkToNibbleArrayMap：跨 chunk 共享 nibble 存储（带 cache/uncached 快照），邻 chunk 光照值**读现成结果**，越界传播不重算邻 chunk | **无跨 chunk 光照缓存**——同一邻 chunk 被中心地位重算 1 次 + 作为邻居结构性参与 8 次重算（P1a §2.3④ 结构性事实） | **open 疑似错配**（行 L6，与 L5 同根，缓存缺失是 ×9 的机制表达） |
| 5b | sky 首亮复用 4 邻 heightmap（非 3×3 全量数据） | 邻 chunk 以 blocks9/packed 全方块输入参与，不用 heightmap 通道信息 | **open 疑似错配**（行 L6 附注） |
| 5c | 2 槽 chunk 缓存（doLightUpdates 末清） | Scratch RefCell 跨调用复用（同线程内）+ Java/Rust ThreadLocal 缓冲（Mixin:91-109 / jni_bridge:294-309） | **形态不同但等价（论证）**：两者都是「单次计算内的工作缓冲」，非跨 chunk 状态复用，语义等价 |

**行 L6（open 疑似错配 · 嫌疑 ★★，L5 的机制面）**：vanilla 的 LightStorage 是「每格光照值只算一次、跨 chunk 读者共享」的持久层；CoreSwap 的等价层不存在——光照状态只在单次 3×3 域计算内有效，出域即弃。这是 L5 ×9 冗余的缓存侧表述；修复形态与 L5 绑定（引入跨调用 nibble 缓存 = 增量化的前置）。

### 辖区外归属建议

| 项 | 归属 |
|---|---|
| enqueueSectionData 再传播语义 / G3 正确性判据 | 光照正确性课题（12 篇遗留 @anchor.idk），非形态审计 |
| INITIALIZE_LIGHT（setSectionStatus 批登记） | vanilla 保留（无 mixin），形态无差异，不立行 |
| collect 段 Java palette 展开（0.29ms/chunk）性能 | 光照 round4 纯算力域（已冻结至本审计后，计划 §1.3） |
| heightmap 6 种补写（写回侧） | D 段（W-写回 worker）辖区 |

---

## 2. 特别裁定输入项（整理证据，非本 worker 裁定）

### 2.1 项 1：「e2e 慢 7% 但串行反超 1.9×」互斥归因候选（.b1/.b2/.b3）

事实基线（260914-04，§9.7 口径声明：串行 per-chunk vs e2e wall 不可换算）：ON 串行 2.94ms/chunk vs vanilla ~5.6ms/chunk（1.9× 优）；e2e（boot Done 载体，n=6，OFF 极差 15% 噪声带）ON 慢 7%。三互斥候选：

- **.b1（3×3 全量重算总量）**：上限推演——9× 结构冗余 × N chunk 在并行池内 = 池算力占用 ×9，若 worldgen 池宽 W 已被 NOISE/FEATURES 饱和，光照增量直接挤占 → e2e 回退。**反驳证据**：串行口径 2.94 < 5.6 说明单 chunk 全部成本已低于 vanilla 单 chunk，若仅 .b1 生效且池未饱和，e2e 应为净赢 ⇒ .b1 单独不充分，需与 .b2 复合（×9 乘进占线时长）。可分辨探针：池利用率计数（光照占线 ms / 池宽 / wall）。
- **.b2（同步内联阻塞 worldgen 线程 vs vanilla light 独立逻辑队列）**：推演——L1+L2 形态：每 chunk 2.94ms 焊死在 worldgen 车道；vanilla 同等工作在 light 队列按优先级消化、worldgen 车道零占用（enqueue 纳秒级）。同物理池下 .b2 的差异 = **车道粘滞 + 优先级失灵**（我方光照不可被更高优先级 chunk 任务插队）。可分辨探针：mixin 内记录线程 id + 调用间隔分布（是否长任务粘线）、对照 OFF 时 LIGHT 任务的车道占用。
- **.b3（首亮种子化时机差）**：推演——我方要求 3×3 邻 status≥FEATURES 才接管（否则回退 vanilla），等价于把 chunk 完成条件从「LIGHT margin=1」收紧为「8 邻 FEATURES」；vanilla sky 光只需 4 邻 heightmap。流水线尾部（玩家视野边缘）chunk 因此多等一轮邻域成熟 → boot Done 时点后移，与「光照算得更快但 e2e 更慢」签名相容。可分辨探针：回退计数（Mixin 回退链触发次数）+ chunk 完成时间线对齐 OFF/ON。
- **worker 倾向（供 judge 参考，非裁决）**：.b2 为主 + .b1 放大 + .b3 边缘贡献的复合解释与全部观测自洽；但三者互斥性不完美（.b1/.b2 可复合），若 judge 要求严格互斥集，建议以「可分辨探针」作为分叉判据交 fan-out 实测。

### 2.2 项 2：LightEngine.scratch RefCell 并发 UB 面（⚠️ 静态推理，未运行时验证）

- 静态事实链（P1a §2.3② + 本文件 L3）：全局单例静态 handle（Mixin:66）→ JNI 原始指针（jni_bridge.rs:368）→ `RefCell<Scratch> borrow_mut`（light/mod.rs:64/288-302）无同步。
- 并发可达性（结合 P1b）：vanilla 的串行保证在 **light 队列 drain 侧**（runTasks 单线程消化），**不在 light() 调用侧**——调用侧是 ChunkStatus.LIGHT 任务，经 worldGenExecutor 投 FJP（TACR:637-680），FJP asyncMode 宽 clamp(cores-1,1,255) ⇒ 多 chunk LIGHT 任务并行调用 light() **可达**。我方 mixin HEAD cancel 后的全部重活在调用线程 ⇒ 多线程并发进入同一 RefCell 可达 ⇒ UB 面（Scratch 内 opacity/block_light/sky_light/queue/col_max 竞争写 → 错值/崩溃）。
- 缓解因素（不消除）：LIGHT taskMargin=1 + 我方邻域 status 门限收窄并行窗口；Java 侧缓冲全 ThreadLocal（不竞争）。
- 修复形态候选：`Mutex<Scratch>`（单次 compute 全程持有，锁开销可忽略——串行语义等价 vanilla）或 per-thread handle。**与 G3 首载漂移的可能共因见 §2.3**。

### 2.3 项 3：G3 首载漂移 7×（12 篇遗留）与本段形态同源性核对

- 现象（260905-03 g3-roundtrip，confirmed 口径）：首载漂移 rust 6.07% vs vanilla 0.84%（≈7×），二次重启 0/2025 收敛——**一次性、首载特有、之后稳定**。
- 同源性核对：
  - (i) **邻域时机形态**（L5/.b3 同族候选）：我方以「3×3 邻 FEATURES 时刻的方块快照」全量定值；vanilla 以共享 LightStorage 增量收敛（后亮的邻 chunk 会回头传播修正先前 chunk 的边界光）。首载时邻域成熟度次序不同 ⇒ 边界带光照值可系统性偏移；重载时 nibble 已持久化、直接读旧值 ⇒ 0 漂移。**与「一次性 + 收敛」签名相容** ✅。
  - (ii) **RefCell UB 面**（§2.2）：若首载并行 window 内偶发竞争写 → 错值定值 → 持久化后同样表现为「一次性漂移」。与签名也相容 ⚠️——**两候选当前不可分**，需探针分辨（如单线程强制复跑 gate ON 首载：漂移消失 ⇒ UB 主导；不变 ⇒ 时机形态主导）。
  - (iii) 12 篇/feature-parity 侧的 POST 排序候选（feature 写入序相关）：属 feature 课题 D12 交叉面，与本段形态不同源（那是方块输入差，不是光照执行形态差）——**排除同源**（引用 scout2-random-derivation §5 标注）。
- 结论倾向（供 judge）：G3 7× 与本段「全量重算 + 无共享存储」形态**疑似同源（(i) 为主候选，(ii) 未排除）**，修复 L5/L6（跨 chunk 状态共享或复刻 vanilla 收敛语义）可能顺带收敛该正确性残差——这是候选池排序时「一箭双雕」权重项。

---

## 3. 汇总

- **错配行数**：6 行（L1 同步粘线 / L2 线程放置 / L3 RefCell UB / L4 批粒度 / L5 全量 ×9 / L6 缓存缺失）+ 3 行「形态不同但等价」论证（2c/4a/5c）+ 4 项辖区外归属。
- **open 嫌疑排序（worker 倾向，供 judge）**：
  1. **L5+L6（全量 ×9 + 无跨 chunk 缓存）**——结构性最大冗余，且疑似 G3 7× 同源（正确性+性能双收益）；
  2. **L2+L1（同步内联 + 车道粘滞）**——e2e 7% 回退首席归因候选（.b2）；
  3. **L3（RefCell UB 面）**——正确性风险（潜在崩溃/错值），修复成本近零（Mutex），优先级按风险而非性能排；
  4. **L4（批粒度）**——被 L5 掩盖，作为 L5 修复的附属形态。
- **互斥候选列表**：.b1 总量冗余 / .b2 线程粘滞 / .b3 时机差（§2.1，各附可分辨探针）；G3 同源 (i) 时机 / (ii) UB（§2.3，附单线程复跑分辨法）。
- 产物：本文件（draft，Degraded 静态对照分层）。
- retry 轮次：1（首轮静态对照即闭合；引用的运行时数据全部来自 260914-04 round3 既有探针，本 worker 无新采集）。自检：所有行号经 P1a/P1b + light/mod.rs 与 Mixin 源核对；量级数字均为推演且已注明假设；未授 confirmed/candidate。（judge C-2 补，260915-01）
