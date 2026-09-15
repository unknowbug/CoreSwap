package wg.bench.mixin;

import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.world.ChunkRegion;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.StructureAccessor;
import net.minecraft.world.gen.chunk.Blender;
import net.minecraft.world.gen.chunk.ChunkGeneratorSettings;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import net.minecraft.world.gen.noise.NoiseConfig;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import wg.bench.CppBridge;

/**
 * CoreSwap：用 native 生成替换 vanilla 的 NOISE（方块）与 SURFACE（表面规则）阶段。
 *
 * 维度识别（BUG-002 修复，260906）：按本 generator 的 noise_settings 注册 id
 * （ChunkGeneratorSettings RegistryEntry key）判定——仅 "minecraft:overworld" /
 * "minecraft:nether" 接管；末地（"minecraft:end"）与 mod 维度（"aether:*" 等）
 * 一律放行 vanilla（issue #24：末地 38%、aether 39% 等裸 bedrock 根因修复）。
 * end 已加入接管集（260906-04：Rust end 管线接通后加回，接管集 = {overworld, nether, end}）。
 *
 * 旧的「chunk 形状指纹 + biomeSource 裸反射」已删除：反射的字符串字面量不被
 * mixin/Connector remap 重写，Forge 生产 SRG 命名下 ChunkGenerator.biomeSource
 * 实际名为 f_62137_（srg_to_official_1.20.1.tsrg 行 180533），getDeclaredField
 * ("biomeSource") 生产必 NoSuchFieldException → catch 静默 false → 末地豁免失效
 * 被 nether 句柄误接管。settings RegistryEntry 是符号引用，remap 正确处理。
 *
 * 已知边界：mod 维度若直接复用 "minecraft:overworld"/"minecraft:nether" settings id
 * 仍会被接管（维持 overworld 既有行为）；overworld 变体（amplified/large_biomes/
 * floating_islands）id 不同 → 放行 vanilla（此前被 overworld.json 错误接管，本次起
 * 行为更正确）。
 *
 * R3 异步化（260910-06，1.21.6 260910-04/05 同款形态移植）：三分支的重活从「传入车道 executor
 * 上同步执行」改为「{@code Util.getMainWorkerExecutor()} 上异步执行」，把重活移出 worldgen 单车道；
 * 同构建态 A/B 回退开关 {@code -Dcoreswap.syncfill=1}（{@code -Psyncfill=1}）。
 * ⚠️ **与 vanilla 1.20.1 的形态差异（有意）**：vanilla 在异步 fill 前会对 sections 加锁，但那只与
 * vanilla 自己的 {@code setBlockState(..., lock=false)} 写路径配套；本工程的写回（{@code CppBridge.writeChunk}
 * → {@code ChunkSection.setBlockState} → {@code PalettedContainer.swap}）**自带逐次 lock/unlock**，
 * 外层再加锁 = 同线程自锁死（实测 E1：Chunky {@code Processed: 0}）。故这里不加外层锁。
 */
@Mixin(NoiseChunkGenerator.class)
public abstract class NoiseChunkGeneratorMixin {

    /** 已打过的放行说明日志 id——防每 chunk 刷屏。 */
    private static final java.util.Set<String> wgLoggedReleases =
            java.util.concurrent.ConcurrentHashMap.newKeySet();

    /**
     * per-chunk 接管日志门控（260910-04 口径净化 / R1，1.21.6 同款同步）。
     * 默认关（生产零成本）；诊断时 -Dcoreswap.mixlog=1 打开。行为等价，仅日志。
     */
    private static final boolean MIXLOG = System.getProperty("coreswap.mixlog") != null;

    /**
     * R3（260910-06）异步填充通道：把重活**移出 worldgen 单车道**（1.21.6 R3 的同款形态移植）。
     * 池 = {@code Util.getMainWorkerExecutor()}——**这正是 vanilla 1.20.1 同一阶段用的池**
     * （{@code NoiseChunkGenerator.populateNoise} :348-349 的 {@code "wgen_fill_noise"}）。
     * ⚠️ 不能照抄 1.21.6 的 {@code .named("coreswap_fill_noise")}：1.20.1 的
     * {@code getMainWorkerExecutor()} 返回 {@code ExecutorService}（{@code Util.java:229}），
     * 带 {@code named()} 的 {@code NameableExecutor} 是 1.21.6 才有的 API。
     */
    private static final java.util.concurrent.Executor WG_FILL_POOL =
            net.minecraft.util.Util.getMainWorkerExecutor();

    /** R3 A/B 回退：-Dcoreswap.syncfill=1 → 内联执行（改造前的同步形态），默认即异步。 */
    private static final boolean SYNCFILL = System.getProperty("coreswap.syncfill") != null;

    /**
     * P1（260911-02）in-flight 限流：恢复**引擎设计并发度**。
     *
     * <p>Phase 1 实测（`.investigations/vivo-stutter-260911-02/f2-phase1-cost-split.md`）：
     * vivo 路径每 chunk 调一次 native fill，引擎侧线程参数（客户端 physical-2）只 spawn 空转线程，
     * **真实并发度 = Java 池宽 = cores-1（本机 23，实测 {@code inflight max=23}）** ⇒ 超订 12 物理核：
     * 每 chunk fill 50 ms（单线程真值）→ 96 ms（1.9×），总 CPU +42%（331→471 cpuSec），
     * 而 wall 在池宽 11 已与 23 持平（41 vs 38 s）——**超订不换吞吐，只烧 CPU**（客户端上与网格构建
     * 共用同一池 ⇒ 饿死渲染线程）。
     *
     * <p>规模：{@code -Dcoreswap.maxinflight=N} 显式覆盖；缺省 = 物理核-2（与引擎
     * {@code adaptive_threads} 同源：{@code logical/2 - 2}）；{@code N<=0} = 不限（改造前行为）。
     */
    private static final int WG_MAX_INFLIGHT = resolveMaxInflight();

    /** 限流规模一次性自证日志（#81 行为化：参数生效必须可从日志核对）。 */
    private static final java.util.concurrent.atomic.AtomicBoolean wgInflightLogged =
            new java.util.concurrent.atomic.AtomicBoolean();

    private static void wgLogInflightOnce() {
        if (wgInflightLogged.compareAndSet(false, true)) {
            System.out.println("[WG-INFLIGHT] max_inflight="
                    + (WG_MAX_INFLIGHT > 0 ? String.valueOf(WG_MAX_INFLIGHT) : "unlimited")
                    + " logical=" + Runtime.getRuntime().availableProcessors()
                    + " syncfill=" + SYNCFILL);
        }
    }

    /** 许可在**调用线程**获取、任务完成释放 ⇒ 池内只跑已获许可的任务（不阻塞池线程）。 */
    private static final java.util.concurrent.Semaphore WG_FILL_PERMITS =
            WG_MAX_INFLIGHT > 0 ? new java.util.concurrent.Semaphore(WG_MAX_INFLIGHT) : null;

    private static int resolveMaxInflight() {
        String p = System.getProperty("coreswap.maxinflight");
        if (p != null) {
            try {
                return Integer.parseInt(p.trim());
            } catch (NumberFormatException e) {
                System.out.println("[WG-INFLIGHT] bad -Dcoreswap.maxinflight=" + p + " → 用缺省");
            }
        }
        int logical = Runtime.getRuntime().availableProcessors();
        return Math.max(1, logical / 2 - 2);
    }

    /**
     * exec 模式（260911-03）：CoreSwap fill 改投<b>自有有界执行器</b>，完全退出 vanilla 共享池
     * （{@code Util.getMainWorkerExecutor()} 同时承载 ChunkBuilder 网格构建——javap 已证同池，
     * vivo 低帧根因 = fill 挤占网格构建线程）。与 P1 信号量互补：
     * P1 只限制「在共享池上同时飞多少个 fill」，fill 仍占共享池线程槽位；exec 把 fill
     * 整体搬走 ⇒ 网格构建拿回全池，车道不阻塞。
     *
     * <p>开关：260911-03 preview 起<b>缺省开</b>；{@code -Dcoreswap.exec=0} 回退 shared 池
     * （+P1 信号量路径）。池宽：{@code -Dcoreswap.execpool=N} 显式覆盖；缺省与 P1 同源
     * （{@code logical/2 - 2}）。形态：{@code ThreadPoolExecutor(N, N, keepAlive,
     * LinkedBlockingQueue(128), namedFactory, CallerRunsPolicy)}——队列满（128）背压时经
     * CallerRuns 回退调用线程（worldgen 车道）内联执行 fill，常态不阻塞车道、不丢任务
     * （judge C2 260911-03：原「车道不阻塞」断言与 CallerRuns 路径矛盾，已修正）；
     * 固定 N 线程 + 有界队列 = in-flight 上限即池宽，无需再加信号量。
     * 分派优先级（judge C4）：SYNCFILL &gt; EXEC_MODE &gt; P1 信号量——exec 开时
     * {@code coreswap.maxinflight} 被忽略（此时无 [WG-INFLIGHT] 行属预期）。
     */
    private static final boolean EXEC_MODE = resolveExecMode();

    private static boolean resolveExecMode() {
        String p = System.getProperty("coreswap.exec");
        if (p != null) {
            p = p.trim();
            if (p.equals("0") || p.equalsIgnoreCase("false")) return false;
            if (p.equals("1") || p.equalsIgnoreCase("true")) return true;
            System.out.println("[WG-EXEC] bad -Dcoreswap.exec=" + p + " → 用缺省(开)");
        }
        return true;
    }

    /** exec 池宽（缺省 = logical/2-2，与 WG_MAX_INFLIGHT 同源口径）。 */
    private static final int WG_EXEC_POOL_SIZE = resolveExecPoolSize();

    private static int resolveExecPoolSize() {
        String p = System.getProperty("coreswap.execpool");
        if (p != null) {
            try {
                int n = Integer.parseInt(p.trim());
                if (n > 0) return n;
                System.out.println("[WG-EXEC] bad -Dcoreswap.execpool=" + p + " → 用缺省");
            } catch (NumberFormatException e) {
                System.out.println("[WG-EXEC] bad -Dcoreswap.execpool=" + p + " → 用缺省");
            }
        }
        int logical = Runtime.getRuntime().availableProcessors();
        return Math.max(1, logical / 2 - 2);
    }

    /** exec 模式一次性自证日志（#81 行为化：开关生效必须可从日志核对）。 */
    private static final java.util.concurrent.atomic.AtomicBoolean wgExecLogged =
            new java.util.concurrent.atomic.AtomicBoolean();

    /**
     * 自有有界执行器（exec 模式）。惰性初始化（首次 dispatch 才建，避免非接管形态空建池）；
     * 线程命名 CoreSwap-Fill-N 便于 jstack 归因；daemon=true 不阻停机。
     */
    private static volatile java.util.concurrent.ThreadPoolExecutor wgOwnPool;

    private static java.util.concurrent.ThreadPoolExecutor wgOwnPool() {
        java.util.concurrent.ThreadPoolExecutor p = wgOwnPool;
        if (p != null) return p;
        synchronized (NoiseChunkGeneratorMixin.class) {
            if (wgOwnPool == null) {
                final int n = WG_EXEC_POOL_SIZE;
                java.util.concurrent.atomic.AtomicInteger seq = new java.util.concurrent.atomic.AtomicInteger();
                wgOwnPool = new java.util.concurrent.ThreadPoolExecutor(
                        n, n, 60L, java.util.concurrent.TimeUnit.SECONDS,
                        new java.util.concurrent.LinkedBlockingQueue<>(128),
                        r -> {
                            Thread t = new Thread(r, "CoreSwap-Fill-" + seq.incrementAndGet());
                            t.setDaemon(true);
                            return t;
                        },
                        new java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy());
                wgOwnPool.allowCoreThreadTimeOut(true);
            }
            p = wgOwnPool;
        }
        return p;
    }

    private static void wgLogExecOnce() {
        if (wgExecLogged.compareAndSet(false, true)) {
            System.out.println("[WG-EXEC] mode=own-pool pool=" + WG_EXEC_POOL_SIZE
                    + " queue=128 rejectedPolicy=CallerRuns"
                    + " logical=" + Runtime.getRuntime().availableProcessors()
                    + " syncfill=" + SYNCFILL);
        }
    }

    /**
     * 三分支共用的分派器（R3 形态，260910-06）：
     * <ul>
     *   <li>默认：{@code supplyAsync(work, WG_FILL_POOL)}——重活离开 worldgen 车道
     *       （对齐 1.21.6 R3 的已验证形态）；</li>
     *   <li>{@code -Dcoreswap.syncfill=1}：内联执行 work（改造前形态，同构建态 A/B 对照臂）。</li>
     * </ul>
     * 两臂唯一差别 = 重活在哪跑（单变量 A/B）。
     *
     * <p>⚠️ **本实现不自行加 sections 锁**（与 vanilla 1.20.1 :337-346 的形态不同，**声明差异**）：
     * vanilla 在自己的 fill 路径里持锁后改用 {@code setBlockState(..., lock=false)}（{@code swapUnsafe}），
     * 而本工程的写回走 {@code CppBridge.writeChunk} → {@code ChunkSection.setBlockState(x,y,z,st)} →
     * {@code PalettedContainer.swap()} —— **swap 自己会 lock/unlock 同一把 {@code LockHelper} 信号量
     * （非可重入）** ⇒ 外层再持锁 = 同线程自锁死。实测证据（260910-06 错误台账 E1）：
     * {@code Worker-Main-20} 卡在 {@code LockHelper.lock ← PalettedContainer.swap ← ChunkSection.setBlockState
     * ← CppBridge.writeChunk}，Chunky 任务 {@code Processed: 0}。故此处不加外层锁；逐次写的锁语义由
     * write 路径自带（改造前的并发模型即如此：{@code CppBridge} 注释「每个 mixin worker 线程直接调
     * JNI fillBlocks… writeChunk 写独立 Chunk 对象天然并行」）。
     */
    private static java.util.concurrent.CompletableFuture<Chunk> wgDispatch(
            java.util.function.Supplier<Chunk> work, String fpDim, int fpCx, int fpCz) {
        // 形态审计探针（260915-03）：sub/sta/end 事件（ON=false 时零成本直返）
        if (wg.bench.FormProbe.ON) wg.bench.FormProbe.fillSub(fpCx, fpCz, fpDim);
        final boolean fpCallerRuns = EXEC_MODE; // CallerRuns 候选标记（执行线程名非 CoreSwap-Fill-* 时由 wrapper 复核）
        java.util.function.Supplier<Chunk> wrapped = () -> {
            if (wg.bench.FormProbe.ON) wg.bench.FormProbe.fillSta(fpCx, fpCz, fpDim);
            long fpT0 = wg.bench.FormProbe.ON ? System.nanoTime() : 0L;
            try {
                return work.get();
            } finally {
                if (wg.bench.FormProbe.ON) {
                    String th = Thread.currentThread().getName();
                    wg.bench.FormProbe.fillEnd(fpCx, fpCz, fpDim, System.nanoTime() - fpT0,
                            fpCallerRuns && !th.startsWith("CoreSwap-Fill-"));
                }
            }
        };
        if (SYNCFILL) {
            return java.util.concurrent.CompletableFuture.completedFuture(wrapped.get());
        }
        if (EXEC_MODE) {
            wgLogExecOnce();
            // exec：fill 投自有有界执行器（池宽即 in-flight 上限），不经共享池、不加信号量。
            return java.util.concurrent.CompletableFuture.supplyAsync(wrapped, wgOwnPool());
        }
        final java.util.concurrent.Semaphore permits = WG_FILL_PERMITS;
        if (permits == null) {
            wgLogInflightOnce();
            return java.util.concurrent.CompletableFuture.supplyAsync(wrapped, WG_FILL_POOL);
        }
        wgLogInflightOnce();
        // P1：在调用线程（worldgen 车道）取许可 ⇒ 池内最多 WG_MAX_INFLIGHT 个 fill 同时在飞。
        permits.acquireUninterruptibly();
        java.util.concurrent.CompletableFuture<Chunk> f;
        try {
            f = java.util.concurrent.CompletableFuture.supplyAsync(wrapped, WG_FILL_POOL);
        } catch (Throwable t) {
            permits.release();   // 提交失败不得泄漏许可（否则许可耗尽 = 永久停顿）
            throw t;
        }
        return f.whenComplete((r, e) -> permits.release());
    }

    /** noise_settings 注册 id，如 "minecraft:overworld"；无 key（dynamic entry）返回 "(unknown)"。 */
    private String wgSettingsId() {
        return ((NoiseChunkGenerator)(Object)this).getSettings()
                .getKey()
                .map(k -> k.getValue().toString())
                .orElse("(unknown)");
    }

    /** 放行 vanilla 时一次性说明日志（仅形状恰好落在接管形状区间时才值得记录）。 */
    private void wgLogReleaseOnce(String shapeTag) {
        String id = wgSettingsId();
        if (wgLoggedReleases.add(id)) {
            System.out.println("[Mixin] release to vanilla: settings=" + id + " shape=" + shapeTag
                    + " (takeover set = {minecraft:overworld, minecraft:nether, minecraft:end})");
        }
    }

    // NOISE 阶段：整块 native 生成（方块 + 高度图），跳过 Java 的 density/aquifer/oreVein
    @Inject(method = "populateNoise(Ljava/util/concurrent/Executor;"
            + "Lnet/minecraft/world/gen/chunk/Blender;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/chunk/Chunk;)"
            + "Ljava/util/concurrent/CompletableFuture;",
            at = @At("HEAD"), cancellable = true)
    private void wgPopulateNoise(java.util.concurrent.Executor executor, Blender blender,
                                 NoiseConfig noiseConfig, StructureAccessor structureAccessor,
                                 Chunk chunk,
                                 CallbackInfoReturnable<java.util.concurrent.CompletableFuture<Chunk>> cir) {
        // 分量对照探针（-Dcomp.probe=true -Dcomp.x=... -Dcomp.z=... [-Dcomp.y=31]）
        if (System.getProperty("comp.probe") != null && !CppBridge.didCompProbe()) {
            CppBridge.compProbe(noiseConfig);
        }
        // 形态审计探针（260915-03）：vanilla 对照臂（CppBridge.enabled=false）的 fill 提交事件；
        // CS 接管臂的 sub/sta/end 由 wgDispatch wrapper 负责（两者互斥，不双计）。
        if (wg.bench.FormProbe.ON && !CppBridge.enabled) {
            wg.bench.FormProbe.fillSub(chunk.getPos().x, chunk.getPos().z,
                    chunk.getBottomY() == -64 ? "ow" : "ne");
        }
        if (!CppBridge.enabled) return;
        boolean overworldShape = chunk.getBottomY() == -64 && chunk.getHeight() == 384;
        boolean zeroShape = chunk.getBottomY() == 0 && chunk.getHeight() == 256;
        // ⚠️ end 维度类型高度 = 256（min_y 0），与 nether 同形 0/256（噪声高度才是 128）——
        // end 只能靠 settings id 区分（260906-04 E5：首版用 0/128 判形状致 end 走放行分支）。
        boolean endShape = zeroShape && CppBridge.endActive() && wgSettingsId().equals("minecraft:end");
        // 主世界：形状匹配 + settings id = minecraft:overworld
        if (overworldShape && wgSettingsId().equals("minecraft:overworld")) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            // R3（260910-06，1.21.6 260910-04 同款形态）：把重活移出 worldgen 单车道。
            // 机制：本 @Inject 在 HEAD cancel，原实现把整个 native 生成同步做在传入 executor 车道上
            // （ChunkStatus.NOISE task 内）⇒ 该车道被生成工作量占满；vanilla 形态 = 返回 pending future、
            // 重活跑在 Util.getMainWorkerExecutor()（NoiseChunkGenerator.java:348-355）。本实现对齐该形态。
            final Chunk target = chunk;
            final StructureAccessor structures = structureAccessor;
            java.util.function.Supplier<Chunk> work = () -> {
                long tt0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                wg.bench.ChunkTiming.enter(tt0);
                wg.bench.ChunkTiming.inflightEnter();
                try {
                    // Beardifier：vanilla 在 doFill 内构造 StructureWeightSampler（结构与 Java 同源），
                    // populateNoise 拦截后 vanilla 流程被跳过 → 必须在此喂 native（结构与 Java 同源、时机一致）
                    CppBridge.feedBeardifier(target, structures);
                    CppBridge.fillChunk(target);
                    return target;
                } catch (Throwable t) {
                    // 不吞异常（崩溃日志铁律）：打印现场后原样抛出，由 MC 状态机按失败处理
                    System.out.println("[CppBridge] R3 async fill FAILED chunk("
                            + target.getPos().x + "," + target.getPos().z + "): " + t);
                    t.printStackTrace();
                    throw t;
                } finally {
                    if (wg.bench.ChunkTiming.ON) {
                        long tx = System.nanoTime();
                        wg.bench.ChunkTiming.exit(tx, tx - tt0);
                        wg.bench.ChunkTiming.inflightExit();
                    }
                }
            };
            cir.setReturnValue(wgDispatch(work, "ow", chunk.getPos().x, chunk.getPos().z));
            return;
        }
        // 下界：形状匹配 + settings id = minecraft:nether + nether 句柄就绪
        // （末地 settings id = minecraft:end → 不再进入本分支，修复 issue #24 38% 误接管）
        if (zeroShape && CppBridge.netherActive() && wgSettingsId().equals("minecraft:nether")) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(nether) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            // R3 推广（260910-06）：与 overworld 分支同构（1.20.1 侧 nether 接管历史更久，本块只做 sanity，
            // 全维行为门见 .investigations/000-架构设计/架构计划-260910-06.md §7 R3）。
            final Chunk target = chunk;
            final StructureAccessor structures = structureAccessor;
            java.util.function.Supplier<Chunk> work = () -> {
                long tt0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                wg.bench.ChunkTiming.enter(tt0);
                wg.bench.ChunkTiming.inflightEnter();
                try {
                    CppBridge.feedBeardifierNether(target, structures);
                    CppBridge.fillChunkNether(target);
                    return target;
                } catch (Throwable t) {
                    System.out.println("[CppBridge] R3 async fill FAILED (nether) chunk("
                            + target.getPos().x + "," + target.getPos().z + "): " + t);
                    t.printStackTrace();
                    throw t;
                } finally {
                    if (wg.bench.ChunkTiming.ON) {
                        long tx = System.nanoTime();
                        wg.bench.ChunkTiming.exit(tx, tx - tt0);
                        wg.bench.ChunkTiming.inflightExit();
                    }
                }
            };
            cir.setReturnValue(wgDispatch(work, "ne", chunk.getPos().x, chunk.getPos().z));
            return;
        }
        // 末地：同形 0/256 + settings id + end 句柄就绪（260906-04 end 接管里程碑）
        if (endShape) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(end) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            final Chunk target = chunk;
            final StructureAccessor structures = structureAccessor;
            java.util.function.Supplier<Chunk> work = () -> {
                long tt0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                wg.bench.ChunkTiming.enter(tt0);
                wg.bench.ChunkTiming.inflightEnter();
                try {
                    CppBridge.feedBeardifierEnd(target, structures);
                    CppBridge.fillChunkEnd(target);
                    return target;
                } catch (Throwable t) {
                    System.out.println("[CppBridge] R3 async fill FAILED (end) chunk("
                            + target.getPos().x + "," + target.getPos().z + "): " + t);
                    t.printStackTrace();
                    throw t;
                } finally {
                    if (wg.bench.ChunkTiming.ON) {
                        long tx = System.nanoTime();
                        wg.bench.ChunkTiming.exit(tx, tx - tt0);
                        wg.bench.ChunkTiming.inflightExit();
                    }
                }
            };
            cir.setReturnValue(wgDispatch(work, "en", chunk.getPos().x, chunk.getPos().z));
            return;
        }
        // 形状匹配但 settings 不在接管集 → 放行 vanilla，一次性说明（含 aether 等 mod 维度）
        if (overworldShape || zeroShape) {
            wgLogReleaseOnce(overworldShape ? "-64/384" : "0/256");
        }
    }

    // 形态审计探针（260915-03）：vanilla 对照臂的 fill 完成事件（RETURN 时挂完成回调，
    // 回调线程 = 真正执行 fill 的工作线程；CS 接管臂不进此路径）。
    @Inject(method = "populateNoise(Ljava/util/concurrent/Executor;"
            + "Lnet/minecraft/world/gen/chunk/Blender;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/chunk/Chunk;)"
            + "Ljava/util/concurrent/CompletableFuture;",
            at = @At("RETURN"))
    private void wgFormProbeFillEnd(java.util.concurrent.Executor executor, Blender blender,
                                    NoiseConfig noiseConfig, StructureAccessor structureAccessor,
                                    Chunk chunk,
                                    CallbackInfoReturnable<java.util.concurrent.CompletableFuture<Chunk>> cir) {
        if (!wg.bench.FormProbe.ON || CppBridge.enabled) return;
        if (cir.getReturnValue() instanceof java.util.concurrent.CompletableFuture) {
            java.util.concurrent.CompletableFuture<?> f = (java.util.concurrent.CompletableFuture<?>) cir.getReturnValue();
            final int cx = chunk.getPos().x;
            final int cz = chunk.getPos().z;
            final String dim = chunk.getBottomY() == -64 ? "ow" : "ne";
            f.whenComplete((r, e) -> wg.bench.FormProbe.fillEnd(cx, cz, dim, 0L, false));
        }
    }

    // SURFACE 阶段：native 已生成表面（surface rules 在 wg_fill_blocks 内部），跳过 Java 实现。
    // 只对已接管的维度 cancel；其余维度放行 vanilla。
    @Inject(method = "buildSurface(Lnet/minecraft/world/ChunkRegion;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/chunk/Chunk;)V",
            at = @At("HEAD"), cancellable = true)
    private void wgBuildSurface(ChunkRegion region, StructureAccessor structures,
                                NoiseConfig noiseConfig, Chunk chunk, CallbackInfo ci) {
        if (!CppBridge.enabled) return;
        boolean overworld = chunk.getBottomY() == -64 && chunk.getHeight() == 384
                && wgSettingsId().equals("minecraft:overworld");
        boolean nether = chunk.getBottomY() == 0 && chunk.getHeight() == 256
                && CppBridge.netherActive() && wgSettingsId().equals("minecraft:nether");
        boolean end = chunk.getBottomY() == 0 && chunk.getHeight() == 256
                && CppBridge.endActive() && wgSettingsId().equals("minecraft:end");
        if (overworld || nether || end) {
            if (MIXLOG) System.out.println("[Mixin] buildSurface skipped chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            ci.cancel();
        }
    }
}
