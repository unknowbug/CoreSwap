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
        // 只拦截主世界：主世界特征 = minY=-64 + height=384（下界 minY=0/256、末地 TheEndGenerator 不在此类、多数维度 mod 高度不同）
        // 注意：极端情况下维度 mod 若用 NoiseChunkGenerator 且同为主世界高度，会被误拦——后续可加 -Dcoreswap.dimensions 白名单
        if (CppBridge.enabled
                && chunk.getHeightLimitView().getBottomY() == -64
                && chunk.getHeightLimitView().getHeight() == 384) {
            System.out.println("[Mixin] populateNoise intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            CppBridge.fillChunk(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
        }
    }

    // SURFACE 阶段：C++ 已生成表面（surface rules 在 wg_fill_blocks 内部），跳过 Java 实现
    @Inject(method = "buildSurface(Lnet/minecraft/world/ChunkRegion;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/chunk/Chunk;)V",
            at = @At("HEAD"), cancellable = true)
    private void wgBuildSurface(ChunkRegion region, StructureAccessor structures,
                                NoiseConfig noiseConfig, Chunk chunk, CallbackInfo ci) {
        if (CppBridge.enabled) {
            System.out.println("[Mixin] buildSurface skipped chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            ci.cancel();
        }
    }
}
