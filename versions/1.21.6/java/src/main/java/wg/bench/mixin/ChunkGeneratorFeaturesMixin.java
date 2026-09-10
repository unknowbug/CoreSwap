package wg.bench.mixin;

import net.minecraft.world.StructureWorldAccess;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.StructureAccessor;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import net.minecraft.world.gen.feature.PlacedFeature;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import wg.bench.CppBridge;

import java.util.concurrent.atomic.AtomicLong;

/**
 * CoreSwap batchD-E1（mc-1216 features 接管修正）：Rust features 接管时跳过 Java vanilla 装饰器。
 *
 * v2 注入点修正（260909-02，Phase 2.5 对拍实证）：
 *  - v1（@Inject HEAD cancellable ci.cancel()）连坐了 generateFeatures 内的**结构方块放置段**
 *    （ChunkGenerator.java:360-379：per-step `start.place(...)` 与 feature 迭代同方法同循环）——
 *    HEAD cancel 导致 Rust features 臂丢失全部结构件放置（Chunky 双臂对拍实锤）。
 *  - v2：HEAD 仅做门控判定（ThreadLocal，不 cancel）；@Redirect 精准拦截
 *    `placedFeature.generate(...)` 调用点（:406）——结构放置段及其 vanilla decorator seed
 *    消费（setDecoratorSeed l,m,k / l,p,k）原样保留，Java feature 逐调用 no-op。
 *
 * 接管语义：takeover() = enabled && mask!=0 && bit1 清；维度/形状/句柄门控与
 * NoiseChunkGeneratorMixin.buildSurface 同口径；mod 生成器放行 vanilla。
 * 切换：-Dcoreswap.rust.stages=1（bit1 清）→ Rust features + Java 让位；默认 0b011 → 本 mixin
 * 不生效（redirect 原样转发）。回退 = 删 -D 参数。
 *
 * P-b1 序列探针（260909-02）：env WG_FEATURESEQ=1 门控，chunk(0,0) dump
 *  - [JFEATURE]    setDecoratorSeed 特征侧调用点（:402，ordinal=1，结构侧 :364 = ordinal 0 不打）
 *  - [JFEATURE-FID] placedFeature.generate 调用点的 fid（映射 p→fid 用）
 * MixinExtras 编译期不可用（runtime-only）→ 纯 Mixin @Redirect + ThreadLocal 传 chunk 坐标。
 */
@Mixin(ChunkGenerator.class)
public abstract class ChunkGeneratorFeaturesMixin {

    /** 行为化哨兵计数（发现 #81）：首拦 + 每 4096 打点。 */
    private static final AtomicLong WG_FEATURES_SKIP_COUNT = new AtomicLong();

    /** P-b1 序列探针开关（env WG_FEATURESEQ=1；启动期读一次——发现 #11：激活判据与数据初始化分离）。 */
    private static final boolean FEATURE_SEQ_ENABLED = "1".equals(System.getenv("WG_FEATURESEQ"));

    /** 当前 chunk 是否由 Rust features 接管（HEAD 判定 → generate 调用点消费；RETURN 清除）。 */
    private static final ThreadLocal<Boolean> WG_TAKEOVER_ACTIVE = ThreadLocal.withInitial(() -> Boolean.FALSE);

    /** P-b1 探针：当前线程是否正处理 chunk(0,0)（HEAD 置 / RETURN 清；null = 非 0,0 或未启用）。 */
    private static final ThreadLocal<int[]> WG_PROBE_CHUNK = new ThreadLocal<>();

    /** 260910-04 chunk 级阶段计时（门控 -Dcoreswap.chunktime=1）：features 阶段起止。 */
    private static final ThreadLocal<long[]> WG_FEAT_T0 = ThreadLocal.withInitial(() -> new long[]{0L});

    @Inject(method = "generateFeatures("
            + "Lnet/minecraft/world/StructureWorldAccess;"
            + "Lnet/minecraft/world/chunk/Chunk;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + ")V",
            at = @At("HEAD"))
    private void wgJudgeTakeover(StructureWorldAccess world, Chunk chunk, StructureAccessor structureAccessor,
                                 CallbackInfo ci) {
        boolean takeover = false;
        if (CppBridge.rustFeaturesTakeover()
                && (((Object) this) instanceof NoiseChunkGenerator)) { // mod 生成器放行 vanilla
            String settingsId = ((NoiseChunkGenerator) (Object) this).getSettings()
                    .getKey().map(k -> k.getValue().toString()).orElse("(unknown)");
            boolean overworld = chunk.getBottomY() == -64 && chunk.getHeight() == 384
                    && settingsId.equals("minecraft:overworld") && CppBridge.enabled;
            boolean nether = chunk.getBottomY() == 0 && chunk.getHeight() == 256
                    && settingsId.equals("minecraft:nether") && CppBridge.netherActive();
            boolean end = chunk.getBottomY() == 0 && chunk.getHeight() == 256
                    && settingsId.equals("minecraft:end") && CppBridge.endActive();
            takeover = overworld || nether || end;
        }
        WG_TAKEOVER_ACTIVE.set(takeover);
        if (wg.bench.ChunkTiming.ON) {
            long t = System.nanoTime();
            WG_FEAT_T0.get()[0] = t;
            wg.bench.ChunkTiming.featTick(t);
        }
        if (FEATURE_SEQ_ENABLED) {
            WG_PROBE_CHUNK.set((chunk.getPos().x == 0 && chunk.getPos().z == 0) ? new int[]{0, 0} : null);
        }
    }

    @Inject(method = "generateFeatures("
            + "Lnet/minecraft/world/StructureWorldAccess;"
            + "Lnet/minecraft/world/chunk/Chunk;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + ")V",
            at = @At("RETURN"))
    private void wgClearTakeover(StructureWorldAccess world, Chunk chunk, StructureAccessor structureAccessor,
                                 CallbackInfo ci) {
        WG_TAKEOVER_ACTIVE.set(Boolean.FALSE);
        WG_PROBE_CHUNK.set(null);
        long t0 = WG_FEAT_T0.get()[0];
        if (wg.bench.ChunkTiming.ON && t0 != 0L) wg.bench.ChunkTiming.addFeat(System.nanoTime() - t0);
    }

    @Redirect(method = "generateFeatures",
            at = @At(value = "INVOKE",
                    target = "Lnet/minecraft/util/math/random/ChunkRandom;setDecoratorSeed(JII)V",
                    ordinal = 1))
    private void wgLogFeatureSeq(net.minecraft.util.math.random.ChunkRandom random,
                                 long populationSeed, int index, int step) {
        random.setDecoratorSeed(populationSeed, index, step);
        if (FEATURE_SEQ_ENABLED && WG_PROBE_CHUNK.get() != null) {
            System.out.println("[JFEATURE] chunk(0,0) l=" + populationSeed + " p=" + index + " k=" + step);
        }
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    @Redirect(method = "generateFeatures",
            at = @At(value = "INVOKE",
                    target = "Lnet/minecraft/world/StructureWorldAccess;setCurrentlyGeneratingStructureName(Ljava/util/function/Supplier;)V",
                    ordinal = 1))
    private void wgLogPlacedFeatureKey(StructureWorldAccess world, java.util.function.Supplier supplier) {
        if (FEATURE_SEQ_ENABLED && WG_PROBE_CHUNK.get() != null && !WG_TAKEOVER_ACTIVE.get()) {
            System.out.println("[JFEATURE-FID] chunk(0,0) fid=" + supplier.get());
        }
        world.setCurrentlyGeneratingStructureName(supplier);
    }

    @Redirect(method = "generateFeatures",
            at = @At(value = "INVOKE",
                    target = "Lnet/minecraft/world/gen/feature/PlacedFeature;"
                            + "generate(Lnet/minecraft/world/StructureWorldAccess;"
                            + "Lnet/minecraft/world/gen/chunk/ChunkGenerator;"
                            + "Lnet/minecraft/util/math/random/Random;"
                            + "Lnet/minecraft/util/math/BlockPos;)Z"))
    private boolean wgSkipPlacedFeature(PlacedFeature placedFeature,
                                        StructureWorldAccess world, ChunkGenerator generator,
                                        Random random, BlockPos pos) {
        if (WG_TAKEOVER_ACTIVE.get()) {
            long n = WG_FEATURES_SKIP_COUNT.incrementAndGet();
            if (n == 1 || n % 4096 == 0) {
                System.out.println("[Mixin] placedFeature skipped (rust takeover)"
                        + " count=" + n);
            }
            return true;
        }
        return placedFeature.generate(world, generator, random, pos);
    }
}
