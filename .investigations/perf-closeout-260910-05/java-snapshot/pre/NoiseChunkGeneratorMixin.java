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
 * 行为更正确）。end 接管 = 下一里程碑（需先核 Rust end 管线 + end biome 判定）。
 */
@Mixin(NoiseChunkGenerator.class)
public abstract class NoiseChunkGeneratorMixin {

    /** 已打过的放行说明日志 id——防每 chunk 刷屏。 */
    private static final java.util.Set<String> wgLoggedReleases =
            java.util.concurrent.ConcurrentHashMap.newKeySet();

    /**
     * per-chunk 接管日志门控（260910-04 口径净化 / R1）。
     * 默认关（生产零成本）；诊断时 -Dcoreswap.mixlog=1 打开。
     * 背景：原先无条件 println 在 23 个 Worker-Main 线程并发下是 log4j（同步 appender）串行点——
     * 既污染性能基线口径（处理臂 8450 行 vs 对照臂 0 行），也是生产 jar 的真实成本。行为等价，仅日志。
     */
    private static final boolean MIXLOG = System.getProperty("coreswap.mixlog") != null;

    /**
     * R3（260910-04）异步填充通道：与 vanilla 的 {@code Util.getMainWorkerExecutor().named("wgen_fill_noise")}
     * 同池同命名形态——重活不再占用 worldgen 单车道。
     */
    private static final java.util.concurrent.Executor WG_FILL_POOL =
            net.minecraft.util.Util.getMainWorkerExecutor().named("coreswap_fill_noise");

    /** R3 A/B 回退：-Dcoreswap.syncfill=1 → 走旧的同步路径（默认关，默认即异步）。 */
    private static final boolean SYNCFILL = System.getProperty("coreswap.syncfill") != null;

    /** noise_settings 注册 id，如 "minecraft:overworld"；无 key（动态 entry）返回 "(unknown)"。 */
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
    // 260908-10 签名迁移（1.21.6）：populateNoise 去掉首参 Executor ——
    //   1.20.1: populateNoise(Executor, Blender, NoiseConfig, StructureAccessor, Chunk)
    //   1.21.6: populateNoise(Blender, NoiseConfig, StructureAccessor, Chunk)
    //   证据：versions/1.21.6/data/mc_src_extract/.../NoiseChunkGenerator.java:326（编译期 AP 亦报
    //   「Cannot find target method」= 旧描述符在 1.21.6 不存在，#25/#55 家族「编译过 ≠ 命中」的镜像面）
    @Inject(method = "populateNoise("
            + "Lnet/minecraft/world/gen/chunk/Blender;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/chunk/Chunk;)"
            + "Ljava/util/concurrent/CompletableFuture;",
            at = @At("HEAD"), cancellable = true)
    private void wgPopulateNoise(Blender blender,
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
            final Chunk target = chunk;
            final StructureAccessor structures = structureAccessor;
            // R3（260910-04，定责结论后修复）：把重活**移出 worldgen 单车道**。
            // 机制（b1/verdict §1）：1.21.6 世界生成只有一条 SimpleConsecutiveExecutor("worldgen")，
            // 且 ChunkTaskScheduler 同时只允许 1 个 entry 在飞；旧实现把 47ms 的 native 生成同步做在
            // populateNoise HEAD，导致全世界并行度 = 1（实测 inflight max=1、Σ车道工作≈wall）。
            // vanilla 形态 = 返回 pending future、重活在 Util.getMainWorkerExecutor()（"wgen_fill_noise"）上跑
            // （NoiseChunkGenerator.java:326-353）——本实现对齐该形态。
            // A/B 回退开关：-Dcoreswap.syncfill=1 → 旧的同步路径（用于同构建态对照）。
            java.util.function.Supplier<Chunk> work = () -> {
                long tt0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                wg.bench.ChunkTiming.enter(tt0);
                wg.bench.ChunkTiming.inflightEnter();
                try {
                    // Beardifier：vanilla 在 doFill 内构造 StructureWeightSampler（结构与 Java 同源），
                    // populateNoise 拦截后 vanilla 流程被跳过 → 必须在此喂 native（结构与 Java 同源、时机一致）
                    long tb0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                    CppBridge.feedBeardifier(target, structures);
                    wg.bench.ChunkTiming.addBeard(System.nanoTime() - tb0);
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
            if (SYNCFILL) {
                cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(work.get()));
            } else {
                cir.setReturnValue(java.util.concurrent.CompletableFuture.supplyAsync(work, WG_FILL_POOL));
            }
            return;
        }
        // 下界：形状匹配 + settings id = minecraft:nether + nether 句柄就绪
        // （末地 settings id = minecraft:end → 不再进入本分支，修复 issue #24 38% 误接管）
        if (zeroShape && CppBridge.netherActive() && wgSettingsId().equals("minecraft:nether")) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(nether) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            CppBridge.feedBeardifierNether(chunk, structureAccessor);
            CppBridge.fillChunkNether(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
            return;
        }
        // 末地：同形 0/256 + settings id + end 句柄就绪（260906-04 end 接管里程碑）
        if (endShape) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(end) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            CppBridge.feedBeardifierEnd(chunk, structureAccessor);
            CppBridge.fillChunkEnd(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
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
