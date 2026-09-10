# b1 候选：948ms 不是「依赖等待」，而是「单车道 worldgen 调度器 + 同步接管」把重活锁进了唯一串行通道

- 置信度：**draft**（纯静态源码推导 + 已实测事实的算术自洽，无新运行时探针）
- 验证分层：**Degraded**（只读源码 + 推理；未跑命令/探针）
- 结论倾向：**支持「编排是主体」，但父候选的机制命名需改写**——MC 世界生成路径上**不存在**任何「阻塞等依赖」；真正的机制是 `ChunkTaskScheduler` + `ConsecutiveExecutor("worldgen")` 组成的**单车道**，vanilla 靠 `populateNoise` 返回 pending future 把重活甩出车道，我们的 mixin 把重活留在车道内，于是车道成为硬上限。
- 一手源码根：`versions/1.21.6/data/mc_src_extract/`（yarn 名，下称 `<src>/`）
- 我方代码根：`runtime/1.21.6/java/src/main/java/wg/bench/`

---

## 候选主张

1. **硬结构上限（本候选的核心）**：1.21.6 里所有 chunk 生成推进（`ChunkLoader.run()`）都跑在**单车道** `SimpleConsecutiveExecutor("worldgen")` 上，且 `ChunkTaskScheduler` **同时只允许一个 entry 在飞**（下一个 entry 必须等当前 entry 的 task 全部跑完才会被 `poll`）。
   ⇒ **任何时刻全世界最多只有 1 个 `populateNoise` 接管在执行**，与线程池大小（23）无关。
   ⇒ worldgen 吞吐硬上限 = 1 / 接管耗时 ≈ 1/51ms ≈ 19 cps；wall 下限 = Σ(车道内每 chunk 串行工作)。
   ⇒ 用量给数字验算：`4225 × (51.0 + 1.3 + 3.1) ms = 234.06 s`，实测 **234 s**（差 0.04%）。即**实测 wall 就是结构下限本身**，并行度 ≈ 1.000。
2. **948ms gap 的性质**：不是等依赖、不是等锁、不是空转，而是**单车道喂 23 线程池**的必然结果——稳态下每个 Worker-Main 线程约每 `线程数/吞吐` 秒才轮到一次接管（≈1.27s；实测 gap 948ms + mixin 51ms = 999ms，同量级），其余 22 个线程**真正 parked（idle）**，所以进程只吃 ~1.7 核。
3. **对父候选措辞的修正**：本版本（1.21.6）代码里，loader 拿不到邻居时**从不阻塞**——全部走 `getNow(null)` 非阻塞探测 + `thenRun` 回调重排（见证据 B）。因此「在等依赖」这一机制**不成立**；成立的机制是「车道串行 + yield 点被取消」。
4. **对已有排除项的再确认**：Rust 引擎算力（native 39.9ms vs 实机 jni 47.2ms 自洽）、`-PcoreswapThreads=1`、per-chunk 日志、CA_MIN 缓存——在本机制下都不是主因；而且**在车道串行下，Rust 侧即使存在全局锁也不会被触发/被观测**（并发恒为 1），故「Rust 锁」不能解释 948ms，除非先证伪本候选（见反证条件）。

---

## 证据（file:line）

### A. 状态链与依赖半径（Q1）

**完整状态序列**（`<src>/net/minecraft/world/chunk/ChunkStatus.java:21-32`）：

```
EMPTY(0) → STRUCTURE_STARTS(1) → STRUCTURE_REFERENCES(2) → BIOMES(3) → NOISE(4)
 → SURFACE(5) → CARVERS(6) → FEATURES(7) → INITIALIZE_LIGHT(8) → LIGHT(9)
 → SPAWN(10) → FULL(11)
```

`populateNoise` 对应 **NOISE** 阶段——接线处 `<src>/net/minecraft/world/chunk/ChunkGenerationSteps.java:14-17`：

```java
.then(ChunkStatus.NOISE, builder -> builder
     .dependsOn(ChunkStatus.STRUCTURE_STARTS, 8)
     .dependsOn(ChunkStatus.BIOMES, 1)
     .blockStateWriteRadius(0)
     .task(ChunkGenerating::populateNoise))
```

**每 stage 的直接依赖环**（由 `ChunkGenerationStep.Builder.dependsOn/accumulateDependencies` 算法逐步展开：`<src>/net/minecraft/world/chunk/ChunkGenerationStep.java:72-132`；语义 = 数组下标 j 表示「距离 j 的邻居要达到的 status」，`directDependencies[0]` 恒为 previous status）：

| stage | 距离 0 | 距离 1 | 距离 2..8 |
|---|---|---|---|
| EMPTY | — | — | — |
| STRUCTURE_STARTS | EMPTY | — | — |
| STRUCTURE_REFERENCES | STRUCTURE_STARTS | 同左（半径 8 全是 STRUCTURE_STARTS） | STRUCTURE_STARTS |
| BIOMES | STRUCTURE_REFERENCES | STRUCTURE_STARTS | STRUCTURE_STARTS |
| **NOISE** | **BIOMES** | **BIOMES** | STRUCTURE_STARTS |
| SURFACE | NOISE | BIOMES | STRUCTURE_STARTS |
| CARVERS | SURFACE | STRUCTURE_STARTS | STRUCTURE_STARTS |
| FEATURES | CARVERS | CARVERS | STRUCTURE_STARTS |
| INITIALIZE_LIGHT | FEATURES | — | — |
| LIGHT | INITIALIZE_LIGHT | INITIALIZE_LIGHT | — |
| SPAWN | LIGHT | BIOMES | — |
| FULL | SPAWN | — | — |

**loader 实际扫描的方块半径**（`GenerationDependencies.getAdditionalLevel`：`<src>/net/minecraft/world/chunk/GenerationDependencies.java:35-42`；`ChunkLoader.getAdditionalLevel`：`<src>/net/minecraft/world/chunk/ChunkLoader.java:129-132`）。对 target=FULL 的 loader，我把 12 步 accumulated 数组逐步展开（方法同 `ChunkGenerationStep.java:111-132`），得到 FULL 的 accumulatedDependencies：

```
A_FULL = [SPAWN, FEATURES, CARVERS, INITIALIZE_LIGHT, STRUCTURE_STARTS × 8]   (半径 0..11)
```

⇒ 当 loader 走到某 status 层时，扫描半径 = max{j : A_FULL[j].index ≥ index(status)}：

| 层 | 扫描半径 i | holder 数 (2i+1)² |
|---|---|---|
| EMPTY / STRUCTURE_STARTS | 11 | 529 |
| STRUCTURE_REFERENCES / BIOMES | 3 | 49 |
| **NOISE / SURFACE / CARVERS / FEATURES / INITIALIZE_LIGHT** | **3** | **49** |
| LIGHT | 1 | 9 |
| SPAWN / FULL | 0 | 1 |

（这一段是**静态推导**，没有跑代码；推导链见 `ChunkGenerationStep.Builder` 的 `accumulateDependencies()`。⚠️ 若要与 `-D` 探针核对，建议先只核对 NOISE=3 / LIGHT=1 两个点。）

### B. 调度：单车道 + 单 entry 在飞 + 无阻塞等待（Q2）

**每 chunk 串行约束（CAS 强制逐级推进）**：`<src>/net/minecraft/world/chunk/AbstractChunkHolder.java:229-239`

```java
private boolean progressStatus(ChunkStatus nextStatus) {
    ChunkStatus chunkStatus = nextStatus == ChunkStatus.EMPTY ? null : nextStatus.getPrevious();
    ChunkStatus chunkStatus2 = this.currentStatus.compareAndExchange(chunkStatus, nextStatus);
    ... else throw new IllegalStateException("Unexpected last startedWork status: ...");
}
```

**车道构造（关键）**：`<src>/net/minecraft/server/world/ServerChunkLoadingManager.java:192,196`

```java
SimpleConsecutiveExecutor simpleConsecutiveExecutor = new SimpleConsecutiveExecutor(executor, "worldgen");
...
this.worldGenScheduler = new ChunkTaskScheduler(simpleConsecutiveExecutor, executor);   // 注意：不是 Throttled 版本
```

**任务投递入口**：`ServerChunkLoadingManager.java:668-682`

```java
private void schedule(ChunkLoader loader) {
    AbstractChunkHolder holder = loader.getHolder();
    this.worldGenScheduler.add(() -> {
        CompletableFuture<?> f = loader.run();          // ← 生成推进全在这里面
        if (f != null) f.thenRun(() -> this.schedule(loader));
    }, holder.getPos().toLong(), holder::getCompletedLevel);
}
@Override public void updateChunks() { this.loaders.forEach(this::schedule); this.loaders.clear(); }
```

**调度器 = 单 entry 在飞**：`<src>/net/minecraft/server/world/ChunkTaskScheduler.java:44-84`

```java
public void add(Runnable runnable, long pos, IntSupplier levelGetter) {
    this.dispatcher.send(new PrioritizedTask(2, () -> {
        this.queue.add(runnable, pos, levelGetter.getAsInt());
        if (this.pollOnUpdate) { this.pollOnUpdate = false; this.pollTask(); }   // pollOnUpdate 门
    }));
}
protected void pollTask() {
    this.dispatcher.send(new PrioritizedTask(3, () -> {
        Entry entry = this.poll();                          // 一次只取一个 (level, chunkPos) 条目
        if (entry == null) this.pollOnUpdate = true; else this.schedule(entry);
    }));
}
protected void schedule(Entry entry) {
    CompletableFuture.allOf(entry.tasks().stream()
        .map(r -> this.executor.executeAsync(future -> { r.run(); future.complete(Unit.INSTANCE); }))
        .toArray(CompletableFuture[]::new))
      .thenAccept(v -> this.pollTask());                    // ← 下一个 entry 必须等本 entry 全部跑完
}
```

- `entry` 的键是 `(level, chunkPos)`（`<src>/net/minecraft/server/world/LevelPrioritizedQueue.java:42-47,68-84,94`）——**一个 entry 只属于一个 chunk**。
- `pollOnUpdate` 只在**单车道 dispatcher** 里读写，无并发窗口 ⇒ **严格 one entry in flight**。
- `this.executor` = 上面那个 `"worldgen"` 单车道（`ChunkTaskScheduler.java:24-29` 注入）。

**单车道语义（关键）**：`<src>/net/minecraft/util/thread/ConsecutiveExecutor.java:37-49,51-58,70-74,112-122`

```java
private boolean runOnce() { ... Runnable runnable = this.queue.poll(); Util.runInNamedZone(runnable, this.name); return true; }
public void run() { try { this.runOnce(); } finally { this.sleep(); this.scheduleSelf(); } }   // 每次调度只跑 1 个任务
public void send(T runnable) { this.queue.add(runnable); this.scheduleSelf(); }
private boolean wakeUp() { return this.status.compareAndSet(SLEEPING, RUNNING); }              // 单飞门
```
`SimpleConsecutiveExecutor`：`<src>/net/minecraft/util/thread/SimpleConsecutiveExecutor.java:6-14`。

**排除「4 路并发」误读**：`new PrioritizedConsecutiveExecutor(4, dispatchExecutor, "dispatcher")`（`ChunkTaskScheduler.java:27`）里的 `4` 是**优先级档数**，不是车道数——`<src>/net/minecraft/util/thread/PrioritizedConsecutiveExecutor.java:7-11`（`super(new TaskQueue.Prioritized(priorityCount), executor, name)`），仍是**单** ConsecutiveExecutor。

**「等邻居」不阻塞 worker，而是把 worker 放回池**（这是 vanilla 高效的关键，也是被我们去掉的那个 yield 点）：`<src>/net/minecraft/world/chunk/ChunkLoader.java:134-173`

```java
private boolean load(ChunkStatus targetStatus, boolean allowGeneration, AbstractChunkHolder h) {
    CompletableFuture<OptionalChunk<Chunk>> future = h.generate(step, mgr, chunks);
    OptionalChunk<Chunk> now = future.getNow(null);         // ← 非阻塞探测，绝不 join
    if (now == null) { this.futures.add(future); return true; }   // 未完成 → 记账后继续扫下一个 holder
    ...
}
@Nullable private CompletableFuture<?> getLatestPendingFuture() {   // LIFO：本层全部完成才进下一层
    while (!this.futures.isEmpty()) { ... if (now == null) return future; ... }
}
```
`ChunkLoader.java:41-55`（`run()` 的 while 循环 + `getLatestPendingFuture`）、`113-127`（`loadAll` 按半径扫区域）。

**Worker 池**：`<src>/net/minecraft/util/Util.java:103,204-235,237-239,262-264` → `ForkJoinPool(clamp(cores-1,1,255))`，线程名 `Worker-Main-N`；`NameableExecutor.named()`（`<src>/net/minecraft/util/thread/NameableExecutor.java:11-31`）在 dev（`SharedConstants.isDevelopment`）下会**临时把线程改名**为 `wgen_fill_noise` —— 探针设计时不要用线程名做跨段关联（非 dev 下是 no-op 直通）。

**不是 worldgen 的那个 throttle（排除干扰）**：`new ThrottledChunkTaskScheduler(taskExecutor, executor, 4)` 出现在 `<src>/net/minecraft/server/world/ChunkLevelManager.java:49-56`，其 `maxConcurrentChunks=4` 只管**玩家 ticket / 等级推进**那条线（`ThrottledChunkTaskScheduler.poll()` 在 `chunks.size() ≥ 4` 时返回 null，`<src>/net/minecraft/server/world/ThrottledChunkTaskScheduler.java:28-38`），**不 gate worldgen**。⚠️ 但它可能构成「上游请求限流」这一竞争解释，列入建议测量 2 并记入 @idk。

**ChunkSection「锁」不是互斥锁（排除锁串行解释）**：`NoiseChunkGenerator.java:336-349` 在**异步任务内部**对每个 section 调 `lock()/unlock()`；实现是 `<src>/net/minecraft/world/chunk/ChunkSection.java:56-61` → `<src>/net/minecraft/world/chunk/PalettedContainer.java:38-53,160-162` → `<src>/net/minecraft/util/thread/LockHelper.java:31-54`：它是**跨线程访问检测器**（Semaphore(1) + `tryAcquire` 失败即 `acquire()` 后抛 CrashException「Accessing PalettedContainer from multiple threads」），**没有把它当普通互斥锁使用的语义** ⇒ 不可能造成吞吐被锁串行化。且我们的 mixin 在 `@At("HEAD")` 就 return，`ChunkSection.lock()` 整段**根本没执行**（`NoiseChunkGeneratorMixin.java:112-113`）。

### C. 同步 vs 异步的差异落点（Q3）

**vanilla**：`<src>/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:326-353`

```java
public CompletableFuture<Chunk> populateNoise(Blender b, NoiseConfig nc, StructureAccessor sa, Chunk chunk) {
    ... int k = ...;
    return k <= 0 ? CompletableFuture.completedFuture(chunk)
                  : CompletableFuture.supplyAsync(() -> {            // ← 唯一 yield 点
                        ... this.populateNoise(b, sa, nc, chunk, j, k);   // 47ms 重活
                    }, Util.getMainWorkerExecutor().named("wgen_fill_noise"));
}
```

**我们**：`runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:76-113`

```java
@Inject(method = "populateNoise(Lnet/minecraft/world/gen/chunk/Blender;...Ljava/util/concurrent/CompletableFuture;", at = @At("HEAD"), cancellable = true)
private void wgPopulateNoise(...) {
    ...
    CppBridge.feedBeardifier(chunk, structureAccessor);
    CppBridge.fillChunk(chunk);                     // 47.2ms JNI + 3.5ms 写回，全在当前线程
    cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));   // ← 已完成的 future
    return;
}
```
SURFACE 同样被 cancel：`NoiseChunkGeneratorMixin.java:140-157`（vanilla 的 `buildSurface` **本来就是同步的**，`ChunkGenerating.buildSurface` 返回 `completedFuture`，`<src>/net/minecraft/world/chunk/ChunkGenerating.java:102-110`——所以 SURFACE 从来就在车道里，不是新增串行）。

**完成 future 会内联触发后续链（三处内联级联）**：
1. `<src>/net/minecraft/world/chunk/ChunkGenerationStep.java:23-30`：`task.doWork(...).thenApply(finalizeGeneration)` —— 已完成的 future ⇒ `thenApply` **在调用线程内联**执行（写 status）。
2. `<src>/net/minecraft/world/chunk/AbstractChunkHolder.java:63-80`：`chunkLoadingManager.generate(...).handle(...)` —— 同上，`handle` 内联执行 `completeChunkFuture`（`:172-194`）——即**在车道线程内联把这个 chunk 的状态 future 完成掉**，而不是回到池里。
3. `<src>/net/minecraft/world/chunk/ChunkLoader.java:134-155`：`future.getNow(null)` **非 null** ⇒ `load()` 直接 `return true` ⇒ `loadAll` 的 for 循环**继续扫同一层的下一个 holder**，并在同一个车道任务里把它的 NOISE 也同步做完，然后继续下一层（直到遇到真正 pending 的层）。

⇒ 我们的一个重要「副作用」：**一个 entry 任务不再只是 1 个 chunk 的一层，而是「一个 chunk 的 EMPTY..FEATURES 整条链（含 3×3 区域内邻居的 NOISE/SURFACE/CARVERS/FEATURES）」**，全部压在一辆车道上。

**哪些阶段仍会 yield（所以车道任务边界在哪里）**：`populateBiomes` 仍是 async（`NoiseChunkGenerator.java:85-90`，`init_biomes`）、`initializeLight`/`light` 返回光照 pipeline 的 future（`ChunkGenerating.java:144-157`）、`convertToFullChunk` 走主线程（`ChunkGenerating.java:169-197`）⇒ 车道任务 ≈ 「structure 段」+「NOISE..FEATURES 重活段」，后者就是 55.4ms/chunk 的来源。

**其余仍走 Java 的阶段**：`carve`（`ChunkGenerating.java:112-129`，同步）、`generateFeatures`（`131-142`，同步）——两者在 vanilla 里也已同步在车道内（实测 carve=1.3 / feat=3.1ms 与之一致）。

### D. 数字自洽性（用父给的实测数字，未新造数字）

- 每 chunk 车道串行工作 = mixin 51.0 + carve 1.3 + feat 3.1 = **55.4 ms**
- 4225 chunks × 55.4 ms = **234.06 s**，实测 **234 s**（同一批 n=1024 采样给出的三个分量；若实测 N 或 wall 有小数/含尾段，请按建议测量 1 复核）
- 每线程周期 = 948(实测 gap) + 51 = **999 ms**；若 23 线程都在轮转，理论周期 = 23/18.06cps = 1273ms（观测略小 ⇒ 实际参与轮转的线程数约 17–20，量级一致）
- 池占用 = 23 × 55.4/999 ≈ **1.27 核（worldgen 重活）**，实测进程 ~1.7 核（余量 = light/IO/save/主线程）——自洽
- 对照臂：52s/4225 = **12.3 ms/chunk wall**，而每 chunk 重活量同量级（jni 47.2ms 仅是**单线程**成本）⇒ vanilla 的有效并行度 ≈ 4~4.5，coreswap ≈ **1.000**

---

## 机制推演

### 1) 完整调用链（唯一入口，单一车道）

```
ChunkHolder.updateFutures → makeChunkAccessible/getRegion → AbstractChunkHolder.load
  → createLoader (ChunkLoader) → ServerChunkLoadingManager.loaders
  → updateChunks() → schedule(loader) → worldGenScheduler.add(runnable)
  → ChunkTaskScheduler.queue → dispatcher(poll, 单车道) → pollTask → schedule(entry)
  → SimpleConsecutiveExecutor("worldgen")  ← 【唯一车道】
  → ChunkLoader.run() → loadAll(layer, radius) → AbstractChunkHolder.generate(step)
  → ServerChunkLoadingManager.generate (:629-659) → ChunkGenerationStep.run
  → ChunkGenerating.populateNoise (:78-100) → NoiseChunkGenerator.populateNoise  ← mixin 命中点
```
`populateNoise` 只有这一个调用者（全树 grep：`NoiseChunkGenerator` / `ChunkGenerating` / `ChunkGenerator` 抽象 / Flat / Debug），且 `ChunkGenerating.populateNoise` 只被 `ChunkGenerationSteps.GENERATION` 的 NOISE 步骤引用 ⇒ **不存在第二条绕过车道的入口**。

### 2) 两臂对照

- **vanilla**：车道任务 = 「扫区域 + 调 `holder.generate` 提交 `supplyAsync` 并立刻返回 pending future + surface/carve/features 内联」。车道任务单价 ≈ µs~ms 级，`allOf` 立刻完成 ⇒ `pollTask` 立刻取下一个 entry ⇒ 车道每秒能甩出成百上千个 noise 任务到 FJ 池 ⇒ 23 线程并行啃 47ms/chunk ⇒ 宏观 ~4.5 核 busy、81 cps。**pending future 就是「把 worker 还回池」的 yield 点。**
- **coreswap**：同一个车道任务里，`getNow(null)` 永远非 null ⇒ `ChunkLoader.run()` 的 while 一路把 EMPTY→FEATURES 全部跑完（含区域内邻居），51ms×N 全在车道线程内 ⇒ `allOf` 要等这段跑完 ⇒ 下一个 entry 才能被 poll。**车道吞吐 = 1/55.4ms = 18.06 cps，正好实测 18.1 cps。**
- **为什么 23 个线程看着「闲」**：车道每次只 `executor.execute(this)` 提交 1 个任务（`ConsecutiveExecutor.scheduleSelf`），FJ 池把它派给某个空闲 worker；任务 55ms 结束，线程回池 parked，下一次提交再派（可能换线程）。于是每个线程 1 秒左右才被喂一次 ⇒ per-thread gap 948ms，进程 1.7 核。**gap 是「没人喂」，不是「等邻居」。**

### 3) 为什么这是「结构下限」而不是「某个参数没调好」

车道属性 ⇒ wall ≥ Σ(车道内串行工作) = 234.06s（**与池大小、JNI 优化、日志门控无关**）。实测 wall = 234s ⇒ 实测 wall 已被这条下限解释干净。推论：
- 加线程/调 `-PcoreswapThreads`/改 Rust 并行 → 无效（除非重活被移出车道）。
- Rust 再快 2× → 也只能到 ~120s（下限随 jni 线性下降），仍远落后 vanilla 52s。
- **Rust 侧的全局锁在这条链上是「不可观测」的**：并发恒为 1，锁不会被争用 ⇒ 不能拿它解释 948ms（只有先证伪「车道饱和」，才轮到 Rust 锁）。

### 4) 对「依赖等待」措辞的证伪（父候选原表述）

- loader 遇到未满足依赖时**不阻塞**：`getNow(null)` + `thenRun(()->schedule(loader))`（`ChunkLoader.java:144-147`、`ServerChunkLoadingManager.java:671-675`）。
- 强制链上**没有任何 `join()`/`.get()`**：`ServerChunkLoadingManager.java` 里唯一的 `join()` 在 `crash()` 的调试快照里（`:349`，且前置 `isDone()` 守卫）和存档路径（`:801`）。
- ⇒ 「等依赖」若成立，应该看到 worker **BLOCKED/WAITING 在 future 上**、CPU 里含大量 park 但**任务并发 > 1**；实测是「任务并发 = 1 且 wall = Σ串行工作」，对应**车道**而非依赖阻塞。

---

## 反证/证伪条件

1. **最直接的证伪**：读 `[CHUNKTIME]` 末行的 `sumMixin / sumCarve / sumFeat` 与 `n`。若 `sumMixin+sumCarve+sumFeat` **明显小于** wall（例如 < 150s，或 < 60% wall），则车道内还有别的东西、或重活并未全在车道 ⇒ **本候选死**（转向「上游请求限流」或「Rust 侧锁」）。
2. **车道占用率**：若 `ChunkLoader.run()` 级别的车道 busy 累计 / wall ≪ 1（例如 coreswap 只有 40%），说明车道经常在等上游（ticket/Chunky/主线程）⇒ 本候选降级为「非主因」。
3. **并发反证**：线程栈 dump（jcmd Thread.print / 看门狗）若显示 **≥2 个 Worker-Main 同时停在 `CppBridge.fillChunk`/JNI 内**，或大量 Worker-Main 处于 **BLOCKED on monitor**（而非 WAITING/park）⇒ 「单车道 + 无锁」前提不成立，本候选死。
4. **池大小不敏感反证**：coreswap 臂 `-Dmax.bg.threads=1` 与 `=23` 若 wall 差异 >20%，说明池并行度仍在起作用（车道没饱和）⇒ 本候选降级。
5. **vanilla 侧反证**：`-Dmax.bg.threads=1` 的 vanilla 臂若仍 ≈52s（不退化到 200s+），说明 vanilla 的 81 cps 并不依赖池并行度 ⇒ 两臂的差异机制需要重写。
6. **Rust 并行度反证（只在 1/2 成立后才有意义）**：native 多线程 bench 若完全不可 scale（单全局锁），则 Rust 侧是共因；但在车道串行前提下它不解释 948ms，只能作为「移出车道后」的第二道瓶颈记录。

---

## 建议测量（信息量排序）

> 纪律：全部只加**计时/计数**，默认门控关闭、chunk 级一次判断（对齐项目「测量/探针污染铁律」与 ChunkTiming 现有写法）。**不要**为了 A/B 把 mixin 改成异步提交（语义改动，且会改变写回线程归属与可见性）。

1. **零改动、数据已在手**：把 `-Dcoreswap.chunktime=1` 输出的**最后一行** `[CHUNKTIME]` 全文拿来（`runtime/1.21.6/java/src/main/java/wg/bench/ChunkTiming.java:73-86` 已经打印 `sumMixin / sumGap / sumCarve / sumFeat / n`）。
   - 判据：`Δ = wall − (sumMixin+sumCarve+sumFeat)`；`Δ/wall ≤ 5%` ⇒ **并行度 1.000，本候选的关键量化证据成立**。
   - 同时核对 `sumGap/wall ≈ 参与轮转线程数`（预计 15~23；若 ≈1 说明只有一个线程在跑 ⇒ 与「vanilla 式均匀 23 线程」不同，是车道轮转签名）。
2. **车道占用探针（判据最硬）**：给 `ChunkLoader.run()`（或 `ChunkTaskScheduler.schedule` 的 entry task）加 env 门控的进入/退出 `System.nanoTime()` **LongAdder 累加**（非逐点、默认关）⇒ 打印 `ΣlaneBusy` 与 wall、以及 `inFlight`（AtomicInteger）的 max。
   - 预测：coreswap `ΣlaneBusy/wall ≈ 1.00`，`inFlight.max == 1`；vanilla `ΣlaneBusy/wall ≈ 0.3~0.45`，`inFlight.max == 1`。
   - 这是唯一能**直接区分**「车道饱和」与「上游请求限流」的量（1 只能给不等式，2 给等号）。
3. **上游是否限流**：把 worldGenScheduler 队列深度打出来（`ConsecutiveExecutor` 已内建 `worldgen-queue-size` sampler：`ConsecutiveExecutor.java:107-110`），另加 `ServerChunkLoadingManager.loaders.size()` 与 ticket pipeline 的 `ChunkLevelManager.scheduler` 深度（`ChunkLevelManager.java:199-201` 的 `toDumpString`）。
   - 预测：coreswap 长期 **queue>0 或 loaders 有积压**（= 车道是瓶颈）；若 queue 长期为 0/loaders 空 ⇒ 上游喂不进来（竞争解释成立）。
4. **线程状态 dump（零代码改动）**：生成期间每秒 `jcmd <pid> Thread.print`，统计 23 个 `Worker-Main-*` 的状态分布与栈顶。
   - 预测（b1）：绝大多数 `WAITING`（FJ 池 park，栈在 `ForkJoinPool.getTask`），≤1 个 `RUNNABLE` 且在 JNI/`fillChunk` 里。
   - 竞争解释（Rust 全局锁）：多个 `BLOCKED`/在 `JNI` 上排队。
5. **纯配置对照臂（零语义改动，最有说服力）**：vanilla 臂 `-Dmax.bg.threads=1 / 2 / 4 / 23`（`Util.java:237-242`），coreswap 臂 `1 / 23`。
   - 预测：vanilla N=1 ⇒ wall 升到 ~200s+（≈coreswap）、CPU ~1 核；N=4 ⇒ ~60-70s；N=23 ⇒ 52s。
   - 预测：coreswap N=1 与 N=23 **都 ≈234s**（对池大小不敏感 = 车道饱和的签名）。
   - ⚠️ N=1 时 light/IO 与 worldgen 共用 1 线程会抬高两边读数，看**量级/斜率**不看绝对值。
6. **vanilla 单线程每 chunk 成本（补上缺失的对照数）**：在 vanilla 臂给 `NoiseChunkGenerator.populateNoise` 的**私有重载**（`NoiseChunkGenerator.java:355-431`）加同款 chunk 级计时（或复用 ChunkTiming 的 ThreadLocal 写法）。
   - 预测：Java 单线程 ≈ 45~55ms/chunk（与 Rust jni 47.2 同量级）⇒ 两臂**每 chunk 工作量相同、唯一差异是并行度 4.5 → 1.0**，这一步把「Rust 慢」彻底排除。
7. **Δ 归属**：再加 `generateStructures` / `generateStructureReferences` 的 chunk 级计时（`ChunkGenerating.java:31-65`，车道内同步段），确认 Δ 是否被它们吃掉（并顺带确认 bench 的 `shouldGenerateStructures()`，`:35`）。
8. **臂无关尺子**（已在代码里，值得读）：`ChunkTiming.featTick` 的 `featInterval`（`ChunkTiming.java:30-45,84`）两臂都打点，直接对比「同线程两次到 FEATURES 的间隔」：预计 vanilla ≈ 数十 ms 级、coreswap ≈ 1s 级。

---

## @idk 清单

- **@idk：`[CHUNKTIME]` 末行的 `sumMixin/sumCarve/sumFeat/sumGap` 未读到**（父给的只有 n=1024 的 per-chunk 均值）。「并行度=1.000」目前是**推导**（用均值 × 4225 与实测 234s 相减），不是实测。需要的一手证据 = 该次运行的 `[CHUNKTIME]` 全部行原文（尤其最后一行）。
- **@idk：N=4225 与 wall=234s 的精确取值**（是否含 warmup/最后一次 report 到进程结束的尾段、Chunky 完成日志的时间戳）。需要 Chunky 完成输出 + 进程起止时间戳。
- **@idk：`write=3.5ms` 的写回是否走了 `setBlockState(..., lock=false)` 路径**（vanilla 的 `ChunkSection.lock()`（`NoiseChunkGenerator.java:336-349`）在我们的 mixin 下被整段跳过）。这只影响**跨线程可见性/检测**（`LockHelper` 是检测器，不是互斥锁），不影响本候选的吞吐结论；但要定性必须读 `CppBridge.fillChunk` 的 Java 写回实现。
- **@idk：车道任务里 STRUCTURE_STARTS / STRUCTURE_REFERENCES / BIOMES(非 async 部分) 的真实耗时**——它们是 Δ 的候选归属，未测。
- **@idk：gap 的 per-thread 分布**（是否集中在 ~17 个线程、是否轮转）。当前只有全局均值 + ThreadLocal 累加；需要 per-thread 计数直方图才能把「17 线程轮转」从推算变实测。
- **@idk：Chunky 的 chunk 请求路径**（是否被 `ThrottledChunkTaskScheduler(maxConcurrentChunks=4)` 所在的 ticket/等级管线限流）。我未读 Chunky 源码，**无法排除「上游请求限流」是主因**——这是本候选最主要的竞争解释，由建议测量 2/3 判定。
- **@idk：`max.bg.threads=1` 的 vanilla 臂读数会因 light/IO 与 worldgen 共用单线程而偏高**，因此建议测量 5 只能看量级趋势，不能当精确预测值。
- **@idk：我推导的「FULL-target loader 扫描半径表」未与运行时核对**（静态展开 `ChunkGenerationStep.Builder.accumulateDependencies` 得到，NOISE=3 / LIGHT=1 / SS=11）。若要引用该表，请用一条打印实际半径的探针交叉核验。
