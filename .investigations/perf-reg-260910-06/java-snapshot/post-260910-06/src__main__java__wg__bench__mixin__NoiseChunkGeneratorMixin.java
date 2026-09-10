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
            java.util.function.Supplier<Chunk> work) {
        if (SYNCFILL) {
            return java.util.concurrent.CompletableFuture.completedFuture(work.get());
        }
        return java.util.concurrent.CompletableFuture.supplyAsync(work, WG_FILL_POOL);
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
            cir.setReturnValue(wgDispatch(work));
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
            cir.setReturnValue(wgDispatch(work));
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
            cir.setReturnValue(wgDispatch(work));
            return;
        }
        // 形状匹配但 settings 不在接管集 → 放行 vanilla，一次性说明（含 aether 等 mod 维度）
        if (overworldShape || zeroShape) {
            wgLogReleaseOnce(overworldShape ? "-64/384" : "0/256");
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
