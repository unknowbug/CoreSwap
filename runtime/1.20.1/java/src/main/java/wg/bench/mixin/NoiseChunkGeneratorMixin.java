package wg.bench.mixin;

import net.minecraft.world.ChunkRegion;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.StructureAccessor;
import net.minecraft.world.gen.chunk.Blender;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import net.minecraft.world.gen.noise.NoiseConfig;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import wg.bench.CppBridge;

/**
 * CoreSwap：用 C++ 生成替换 vanilla 的 NOISE（方块）与 SURFACE（表面规则）阶段。
 * 启用条件：-Dcpp.replace=1 且 CppBridge.init 成功（wg_create 非 0）。
 */
@Mixin(NoiseChunkGenerator.class)
public abstract class NoiseChunkGeneratorMixin {

    // 末地保护：End 也走 NoiseChunkGenerator 且形状同为 min_y=0/height=256，
    // 用 biomeSource（父类 ChunkGenerator 字段，@Shadow 够不到 → 缓存反射）区分 TheEndBiomeSource
    private static volatile java.lang.reflect.Field wgBiomeSourceField;

    private boolean wgIsEnd() {
        try {
            if (wgBiomeSourceField == null) {
                java.lang.reflect.Field f = net.minecraft.world.gen.chunk.ChunkGenerator.class.getDeclaredField("biomeSource");
                f.setAccessible(true);
                wgBiomeSourceField = f;
            }
            return wgBiomeSourceField.get(this) instanceof net.minecraft.world.biome.source.TheEndBiomeSource;
        } catch (Throwable t) {
            return false;  // 反射失败 → 不排除（保持旧行为，nether 形状拦截仍成立）
        }
    }

    // NOISE 阶段：整块 C++ 生成（方块 + 高度图），跳过 Java 的 density/aquifer/oreVein
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
        boolean overworld = chunk.getHeightLimitView().getBottomY() == -64
                && chunk.getHeightLimitView().getHeight() == 384;
        boolean netherShape = chunk.getHeightLimitView().getBottomY() == 0
                && chunk.getHeightLimitView().getHeight() == 256;
        // 主世界：minY=-64 + height=384
        if (overworld) {
            System.out.println("[Mixin] populateNoise intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            // Beardifier：vanilla 在 doFill 内构造 StructureWeightSampler（结构与 Java 同源），
            // populateNoise 拦截后 vanilla 流程被跳过 → 必须在此喂 C++（结构与 Java 同源、时机一致）
            CppBridge.feedBeardifier(chunk, structureAccessor);
            CppBridge.fillChunk(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
            return;
        }
        // 下界（多世界 2026-08-30）：minY=0 + height=256 且 nether 句柄就绪（末地同形状 → biomeSource 排除）
        if (netherShape && !wgIsEnd() && CppBridge.netherActive()) {
            System.out.println("[Mixin] populateNoise(nether) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            CppBridge.feedBeardifierNether(chunk, structureAccessor);
            CppBridge.fillChunkNether(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
        }
    }

    // SURFACE 阶段：C++ 已生成表面（surface rules 在 wg_fill_blocks 内部），跳过 Java 实现。
    // 只对已接管的维度 cancel（主世界 / nether 接管时），末地等其他维度放行 vanilla。
    @Inject(method = "buildSurface(Lnet/minecraft/world/ChunkRegion;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/chunk/Chunk;)V",
            at = @At("HEAD"), cancellable = true)
    private void wgBuildSurface(ChunkRegion region, StructureAccessor structures,
                                NoiseConfig noiseConfig, Chunk chunk, CallbackInfo ci) {
        if (!CppBridge.enabled) return;
        boolean overworld = chunk.getHeightLimitView().getBottomY() == -64
                && chunk.getHeightLimitView().getHeight() == 384;
        boolean netherShape = chunk.getHeightLimitView().getBottomY() == 0
                && chunk.getHeightLimitView().getHeight() == 256;
        if (overworld || (netherShape && !wgIsEnd() && CppBridge.netherActive())) {
            System.out.println("[Mixin] buildSurface skipped chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            ci.cancel();
        }
    }
}
