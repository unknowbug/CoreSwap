# vanilla 侧事实核对（260911-02）— javap 直读 loom 缓存

> 方法：`minecraft-merged.jar` = **官方混淆名** jar；`intermediary-v2.tiny`（official→intermediary）+ `yarn-1.20.1+build.10-v2.jar`（intermediary→named）三层可解出任意 named 类。
> 工具：`javap -p/-c -classpath .gradle-home\caches\fabric-loom\1.20.1\minecraft-merged.jar <official>`。
> 全部结论 = `[证]`（字节码一手）。

## 1. 主工作池（`Util.getMainWorkerExecutor`）

| 事实 | 证据 |
|---|---|
| `getMainWorkerExecutor` = method_18349 = official `ac.f()` | intermediary-v2.tiny `m ()Ljava/util/concurrent/ExecutorService; f method_18349` |
| 池创建：`Util.c(String)` = `Math.max(1, Math.min(availableProcessors-1, m()))` 的 **ForkJoinPool** | `ac.c` 字节码：`Runtime.availableProcessors → iconst_1 isub → iconst_1 → m() → apa.a(III)`（clamp）→ `new ForkJoinPool(int, factory, handler, true)`；`i>0` 否则 `MoreExecutors.newDirectExecutorService()` |
| `m()` = 读系统属性 `max.bg.threads`（1..255），**缺省 255** | `ac.m` 字节码：`System.getProperty("max.bg.threads")` → parse → 校验 1..255 → 否则 `sipush 255; ireturn` |
| ⇒ 池宽 ≈ `cores-1`（本机 24 逻辑 → 23），**不是 7** | 同上 |

## 2. 「客户端网格构建与 worldgen 共用同一池」——**已证**

| 事实 | 证据 |
|---|---|
| `WorldRenderer`(official `fjv`) 构造 `ChunkBuilder`(official `fmp`) 时传入 **`Util.f()`** | `javap -c fjv`：`new fmp` → `invokestatic ac.f:()Ljava/util/concurrent/ExecutorService;` → `fmp.<init>(few, fjv, Executor, Z, fjk)` |
| `ChunkBuilder` 持 `private final java.util.concurrent.Executor n` + `PriorityBlockingQueue` | `javap -p fmp` |
| `MinecraftServer` 也取 `Util.f()` | `javap -c net.minecraft.server.MinecraftServer`：`invokestatic ac.f()` ×2 |
| vanilla `NoiseChunkGenerator`(official `dhn`) 的 `populateNoise` 同样是**裸 `CompletableFuture.supplyAsync(supplier, executor)`** | `javap -c dhn`：`CompletableFuture.supplyAsync` ×2 |

**⇒ H1 的「共用池」不是差异**（vanilla 本来就共用，且提交形态同构）。差异只可能在**每任务占用形态/时长**或**池外的每 chunk 附加成本**。

## 3. 新嫌疑（CoreSwap 侧，待 A/B 证实）：单 chunk JNI 调用起「线程组」

| 事实 | 证据 |
|---|---|
| Java 侧 `THREADS`：客户端 = **-2**（服务端 -1），可被 `-Dcoreswap.threads=N` 覆盖 | `CppBridge.java:40-62` |
| Java 每 chunk 调 `fillBlocks(h, {cx}, {cz}, {buf}, THREADS)` ⇒ **count = 1** | `CppBridge.java:398-399`（nether/end 同构 `:435,:482`）|
| Rust `adaptive_threads(param, count)`：`count == 1` 时**不 clamp**（MT3 注释：防「池被 clamp 到 1 worker」）⇒ 返回 `threads.max(1)` = 物理核-2 | `worldgen-core/src/api.rs:24-41` |
| `wg_fill_blocks_multi` 随后 `std::thread::scope` **spawn `nthreads` 个线程**，闭包 `while i < count` ⇒ count=1 时**只有 t=0 干活，其余 (nthreads-1) 个空转即退** | `api.rs:146-169` |
| ⇒ 每 chunk **spawn (物理核-2) 个线程**（本机 10；用户机按核数），其中 9 个纯浪费 | 推论（上两行）|

**量级**：按 50-100 chunk/s ⇒ 500-1000 次无效线程创建/秒（Windows `CreateThread` + 2MB 栈 reserve + join）。对照：vanilla 无此形态。
**修法候选**：`adaptive_threads` 对 count=1 也应 clamp（`threads.min(count).max(1)`）——MT3 注释针对的是**已不存在的常驻池**（现为 per-call scoped threads），故 clamp 现在是安全且正确的。

**判别实验（本地，单变量）**：`CORESWAP_THREADS=1`（env 覆盖 → count=1 时只 spawn 1 线程）vs 默认，同 dll/seed/region 交错 4 臂 —— 预期：wall 不劣化、`cpuSec` 显著下降 ⇒ 证实「线程风暴纯浪费」。结果见 `ab-threads/results.txt`。

## 4. 方法沉淀（可复用）

- **javap 直读 vanilla 内部**（本仓库 loom 缓存自带三层映射）：named → (yarn v2) intermediary → (intermediary-v2.tiny) official → `javap`。用于任何「vanilla 到底怎么做的」核对，免反编译、免外部资料。
- 注意：`minecraft-merged.jar` 是**官方混淆名** jar（非 named/intermediary），只有 `DontObfuscate` 类保留可读名（如 `net/minecraft/server/MinecraftServer`）。
