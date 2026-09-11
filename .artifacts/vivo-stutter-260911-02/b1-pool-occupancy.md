---
artifact: b1-pool-occupancy
分支: b1 = H1「共享工作池占用形态差」
课题: vivo-stutter-260911-02（实机 VD32 集成服持续低帧）
status: draft
结论三态: 存疑（倾向：单纯「共用池」被既有反证前提压制；仅剩「每任务池占用时长」形态差候选，且与 H3 辖区重叠）
产出: fan-out worker（只读），2026-09-11
---

# b1 — 池占用形态差（H1）候选

> 范围声明：本文只写**可用 file:line 证明的 CoreSwap 侧形态**。vanilla 1.20.1 内部细节**一律标「需外部资料，未验证」**，不编造。
> 证据等级：`[证]` = 一手源码 file:line；`[注]` = 仓库注释/历史实测声明（非本文复现）；`[未验证]` = 需外部资料或实机探针。

## §1 CoreSwap 提交形态事实表（全部 `[证]`，除标注）

| 项 | 形态 | 证据 |
|---|---|---|
| 提交点 | `populateNoise` HEAD `@Inject` cancel → `cir.setReturnValue(wgDispatch(work))`；三分支同构（overworld/nether/end） | `NoiseChunkGeneratorMixin.java:172,204,233` |
| 任务粒度 | **每 chunk 一个任务**（1 Supplier = 1 chunk）；无合并、无批 | `Mixin.java:148-171`（work 定义）|
| 任务内容 | `CppBridge.feedBeardifier(target, structures)` + `CppBridge.fillChunk(target)` | `Mixin.java:155-156` |
| 池 | `Util.getMainWorkerExecutor()`，**static final 单例**，类加载时解析一次 | `Mixin.java:67-68` |
| 池是否与 vanilla 同 | 仓库注释声明「vanilla 同阶段同池（`NoiseChunkGenerator.populateNoise:348-349 "wgen_fill_noise"`）」 | `[注]` `Mixin.java:61-62`；vanilla 侧未独立核对 `[未验证]` |
| 提交频率 | 每 chunk NOISE 阶段一次；提交速率 = NOISE 阶段推进速率，**无 CoreSwap 侧限流** | `Mixin.java:172`（无信号量/无队列上限/无 in-flight 上限代码） |
| 背压 | **无**。裸 `CompletableFuture.supplyAsync(work, pool)`，走池的无界队列 | `Mixin.java:97` |
| 回调链 | 链上**只有这一个 future**，无 `thenApply/thenCompose`；返回值直接交回 MC 状态机（NOISE 阶段 future） | `Mixin.java:97,172` |
| 回调线程 | future 由**池线程**完成；后续 MC 侧依赖调度（谁在哪个线程消费完成信号）属 MC 内部，`[未验证]` | `[未验证]` |
| 异常处理 | `catch Throwable` → `println` + `printStackTrace` → **原样 rethrow**（不吞）；异常经 CompletableFuture 传给状态机 | `Mixin.java:158-163`（nether/end 同构 `:191-195,:220-224`）|
| `-Dcoreswap.syncfill=1` | `CompletableFuture.completedFuture(work.get())` = **在调用线程（worldgen 车道）内联执行**，池完全不参与 | `Mixin.java:71,94-96` |
| 默认臂 | `supplyAsync(work, WG_FILL_POOL)`，重活离 worldgen 车道 | `Mixin.java:97` |
| 计时门控 | `-Dcoreswap.chunktime=1` → 每 chunk 打 mixin 段/gap/inflight max（默认关，关闭时调用点仅一次布尔判断） | `ChunkTiming.java:19,37-71` |

**每 chunk 池占用链（全部在池线程内串行执行，`[证]`）**：

1. `feedBeardifier`：`StructureWeightSampler.createStructureWeightSampler` + 反射读 piece/junction（Box 6 方法/piece、JigsawJunction 3 方法/junction；`Method` 有 `ConcurrentHashMap` 缓存但**每次仍反射 invoke**）+ JNI `setBeardifier` 序列化 int[]。`CppBridge.java:296-354`（缓存 `:284-294`）
2. `fillChunk` → JNI `fillBlocks`：per-thread `ThreadLocal` buffer `16*16*384` int ≈ **384KB/线程**。`CppBridge.java:38-39,391-399`
3. **全量扫描** `buf` 98304 个 int 统计 nz（无条件执行，非门控）。`CppBridge.java:409-410`（MIXLOG 只门控 println，`[注]` `:363-368`）
4. `writeChunk`：**98,304 次 `sec.setBlockState(x,sy,z,st)`**（384×256）+ 逐 id `STATE_BY_ID` 查表；`setBlockState` 内部 = `PalettedContainer.swap` **逐次 lock/unlock**（非可重入）。`CppBridge.java:518-548`；锁语义声明见 `[注]` `Mixin.java:82-90`（历史实测 E1：`Worker-Main-20` 卡在 `LockHelper.lock ← PalettedContainer.swap`）
5. `Heightmap.populateHeightmaps` **6 种类型**一次性补齐。`CppBridge.java:552-558`
6. 完成后 future 完成 → 池线程释放

**形态要点**：池占用时长 = `JNI 生成 + 98304 次带锁写回 + 6 高度图 + 反射 Beardifier`，**不可抢占、无 yield、无分段**；`syncfill` 臂与默认臂唯一差别 = 这段重活在哪个线程跑（单变量）。

**辖区外（不展开，归属建议）**：JNI 内分配/线程数参数（`THREADS` = 客户端 -2 / 服务端 -1，`CppBridge.java:44-62`，且作为参数传给 `fillBlocks` `:398-399`）→ **归属建议 b3**；`PalettedContainer` 锁争用 → **归属建议 b3/其它**。

## §2 与 vanilla 的可辨差候选（vanilla 内部细节全部标未验证）

| # | 候选差异 | CoreSwap 侧（`[证]`） | vanilla 侧 | 判别力 |
|---|---|---|---|---|
| D1 | **每任务池占用时长** | 任务 = JNI 生成 + 98,304 次**逐次加锁**写回 + 6 高度图 + 反射 Beardifier（§1） | vanilla 同阶段是否把写回留在同一池任务内、是否用 `lock=false`/`swapUnsafe` 批量写 —— `[未验证]`（仓库注释声明 vanilla「先加锁 + `lock=false`」，`[注]` `Mixin.java:41-44,82-90`） | 若成立 ⇒ 同池任务平均驻留时间显著 > vanilla，池有效并发下降（队列更长、尾延迟更高） |
| D2 | **排队优先级/队列位置** | 裸 `supplyAsync` → 池的无界队列，**FIFO，无优先级**（`Mixin.java:97`） | vanilla 若经 `ChunkTaskScheduler` 优先级队列，客户端网格构建任务与 worldgen 任务的相对次序不同 —— `[未验证]` | 需先证「客户端网格构建与 worldgen 是否同池」`[未验证]`；不同池 ⇒ 本候选整体失效 |
| D3 | **背压/限流** | 无任何 CoreSwap 侧限流（§1） | vanilla 是否有 in-flight 上限 —— `[未验证]` | 弱：MC 状态机上游可能天然节流，两侧都受限 |
| D4 | **持续占用/无让出** | 单任务内无 yield/无分段，写回段纯 CPU+锁；`ChunkTiming.inflightEnter/Exit` 可直测同时在飞段数（`ChunkTiming.java:37-46`） | vanilla 对应段形态 —— `[未验证]` | 强（可直测）；但 inflight>1 只证「非串行」，**不证「比 vanilla 差」** |
| D5 | **每 chunk 恒定附加开销** | 98304 int 全量扫描 + 6 高度图 + 每 chunk 一次 JNI 往返（§1 步骤 2/3/5） | vanilla 无此三项 —— 但 vanilla 有等价的 density/aquifer/heightmap 工作，**不可直接比** `[未验证]` | 中：属「绝对开销」而非「池形态」 |

**关键限定**：D1–D5 中**没有一条**能支持「CoreSwap 选了不同池」——既有反证前提（vanilla 同阶段同池）成立时，H1 的「共用池」命题本身不解释差异，只剩 D1/D4「占用形态」支线，且 D1 的 vanilla 对照必须外部资料。**D1/D4 与 H3（接管尾延迟形态差）辖区重叠，收敛时需分账。**

## §3 判别实验表（用户侧可执行，低成本优先）

| # | 实验 | 若 H1 成立 → 预期读数 | 若 H1 不成立 → 预期读数 |
|---|---|---|---|
| E1 | `-Dcoreswap.syncfill=1`（同 jar，重活回到 worldgen 车道）vs 默认 async，同 VD32 同路径飞行 | 卡顿**明显变化**（async 更卡 ⇒ 池争用/占用差是变量；或 sync 更卡 ⇒ 车道被占满） | 两臂**无差**（帧率曲线重合）⇒ 池占用形态非变量，H1 被证伪 |
| E2 | `-Dcoreswap.chunktime=1` 看 `inflight max`（async 臂） | `inflight max` 显著 > 1（多任务并发驻留池）且 `mixin perChunk` 大（数十 ms 级） | `inflight max = 1`（重活实际被串行化 ⇒ 不是池并发问题，指向上游/单点瓶颈）；或 mixin 段极小 |
| E3 | 固定 VD 12（其余不变）对照 | 卡顿**仍存在**（池争用与视距无关度不高）；帧率提升有限 | 卡顿**消失** ⇒ 负载驱动（H2 成立倾向），H1 非主因 |
| E4 | spark profiler 30s（飞行段），看 `Worker-Main-*` 线程热点 | 热点集中在 `CppBridge.writeChunk` / `ChunkSection.setBlockState` / `PalettedContainer.swap` / `fillBlocks`（CoreSwap 代码占池线程时间大头） | 热点集中在 Render thread / 主线程，`Worker-Main-*` 大面积空闲或等锁 ⇒ H1 被证伪 |
| E5 | 上一版 jar（297680e7）同场景对照（H4 排除，顺带看池面） | 上一版同样卡 ⇒ 与 ca_min 微修无关（H4 排除），H1 仍可能 | 上一版不卡 ⇒ 指向本块改动（H4），H1 降级 |
| E6 | 同机 vanilla + VD32 + 同路径（H2 一票判据） | vanilla **不卡**（用户已报告）⇒ 差异真实存在，H1 仍在候选集 | vanilla 同样卡 ⇒ H2 成立，H1 无观测对象 |

**H1 的可证伪点（读到即推翻/降级）**：
1. **E1 两臂无差** ⇒ 池占用形态不是变量，H1 证伪（最强、最廉价）。
2. **E2 `inflight max = 1`** ⇒ 重活并未在池上并发，H1 的「池占用」前提不成立。
3. **E4 池线程热点不是 CoreSwap 代码**（Render thread 占满、Worker-Main 空闲/等锁）⇒ H1 证伪。
4. **D1 vanilla 对照**（外部资料或 vanilla 侧探针）显示 vanilla 同阶段同样做逐块写回且不 `lock=false` ⇒ D1 可辨差归零，H1 仅剩 D2（需先证同池）⇒ 实质证伪。
5. **E3 VD12 卡顿消失** ⇒ 差异由负载而非池形态解释 ⇒ H1 非主因（H2 优先）。

**未验证项（诚实声明）**：① vanilla 1.20.1 `populateNoise` 池与写回形态（本文未读 MC 源码）；② 集成服客户端区块网格构建走哪个池；③ `ChunkTaskScheduler` 优先级在本场景的实际作用；④ 实机帧率不可量化（#64/#65 观察侧纪律）⇒ E1–E6 均需用户侧受控执行 + 读数落盘才可裁。
