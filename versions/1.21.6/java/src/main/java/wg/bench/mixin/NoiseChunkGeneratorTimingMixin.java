package wg.bench.mixin;

import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * 260910-04 chunk 级阶段计时（口径净化门控 -Dcoreswap.chunktime=1，默认关；行为零改动）。
 *
 * <p>动机：CHUNKTIME 实测「接管段 mixin=50.5ms（JNI 46.6）／段外 gap=867ms」——
 * 需要把段外时间拆到 MC 各阶段。本 mixin 只计时 {@code carve}（CARVERS 阶段，
 * 含 {@code ChunkNoiseSampler} 惰性创建），与 features 阶段计时（ChunkGeneratorFeaturesMixin）配合。
 */
@Mixin(NoiseChunkGenerator.class)
public abstract class NoiseChunkGeneratorTimingMixin {

    private static final ThreadLocal<long[]> WG_CARVE_T0 = ThreadLocal.withInitial(() -> new long[]{0L});

    @Inject(method = "carve", at = @At("HEAD"))
    private void wgCarveHead(CallbackInfo ci) {
        if (wg.bench.ChunkTiming.ON) WG_CARVE_T0.get()[0] = System.nanoTime();
    }

    @Inject(method = "carve", at = @At("RETURN"))
    private void wgCarveTail(CallbackInfo ci) {
        long t0 = WG_CARVE_T0.get()[0];
        if (wg.bench.ChunkTiming.ON && t0 != 0L) wg.bench.ChunkTiming.addCarve(System.nanoTime() - t0);
    }
}
