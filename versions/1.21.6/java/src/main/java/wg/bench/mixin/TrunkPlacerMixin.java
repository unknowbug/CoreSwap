package wg.bench.mixin;

import net.minecraft.util.math.random.Random;
import net.minecraft.world.TestableWorld;
import net.minecraft.world.gen.trunk.TrunkPlacer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * 260905-10 P2 判别（.bA 假设检验）：java 树高抽样值打点。
 * 门控 = sysprop wg.treediag / env WG_TREEDIAG（默认关）。
 * 输出 [THJ] i 与 rust 侧 tree.rs [TH] 同格式对拍。
 */
@Mixin(TrunkPlacer.class)
public abstract class TrunkPlacerMixin {

    @Inject(method = "getHeight(Lnet/minecraft/util/math/random/Random;)I", at = @At("RETURN"))
    private void wg$logHeight(Random random, CallbackInfoReturnable<Integer> cir) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            // 260905-12 提速：chunk 过滤（WG_DIAGCHUNK/wg.diagchunk），未设目标 = 旧行为全量
            if (wg.bench.WgDiag.curAllowed()) {
                wg.bench.WgDiag.banner();
                System.out.println("[THJ] " + cir.getReturnValue());
            }
        }
    }

    // j5-rng-trace-plan-260906-01 §2.2：setToDirt（static，TrunkPlacer.java:62-66）位置打点，
    // 消费 iff forceDirt||!canGenerate（分析推导，与 Rust [MJTD] 同口径）。
    @Inject(method = "setToDirt(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)V",
            at = @At("HEAD"))
    private static void wg$mjtd(TestableWorld world, java.util.function.BiConsumer<net.minecraft.util.math.BlockPos, net.minecraft.block.BlockState> replacer, Random random,
                                net.minecraft.util.math.BlockPos pos, net.minecraft.world.gen.feature.TreeFeatureConfig config, org.spongepowered.asm.mixin.injection.callback.CallbackInfo ci) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTD] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ") soil=? fd=?");
        }
    }

    // trySetState（TrunkPlacer.java:88-92）：canReplaceOrIsLog 层入口（拒绝时下方 MJTG 不出现）
    @Inject(method = "trySetState(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos$Mutable;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)V",
            at = @At("HEAD"))
    private void wg$mjtt(TestableWorld world, java.util.function.BiConsumer<net.minecraft.util.math.BlockPos, net.minecraft.block.BlockState> replacer, Random random,
                         net.minecraft.util.math.BlockPos.Mutable pos, net.minecraft.world.gen.feature.TreeFeatureConfig config, org.spongepowered.asm.mixin.injection.callback.CallbackInfo ci) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTT] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ")");
        }
    }

    // getAndSetState 5 参版（TrunkPlacer.java:68-70 委托 6 参）：ok=true ⇒ 消费 trunkProvider.get 1 次
    @Inject(method = "getAndSetState(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Z",
            at = @At("RETURN"))
    private void wg$mjtg(TestableWorld world, java.util.function.BiConsumer<net.minecraft.util.math.BlockPos, net.minecraft.block.BlockState> replacer, Random random,
                         net.minecraft.util.math.BlockPos pos, net.minecraft.world.gen.feature.TreeFeatureConfig config, CallbackInfoReturnable<Boolean> cir) {
        if (wg.bench.WgDiag.curAllowed() && (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null)) {
            System.out.println("[MJTG] p=(" + pos.getX() + ", " + pos.getY() + ", " + pos.getZ() + ") ok=" + cir.getReturnValue());
        }
    }
}
