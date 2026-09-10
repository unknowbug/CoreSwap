package wg.bench.mixin;

import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import net.minecraft.world.gen.feature.FeaturePlacementContext;
import net.minecraft.world.gen.placementmodifier.SquarePlacementModifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;

/**
 * 260905-10 P2 逐树 RNG 对拍（b1 §4 模板）：in_square 两抽打点。
 * 门控 = sysprop wg.treediag（-Ptreediag=1 → build.gradle vmArg）或 env WG_TREEDIAG，默认关。
 * 双 ordinal @Redirect 捕获 x 抽 / z 抽结果并输出绝对坐标 [SQX]/[SQZ]
 * （不用 RETURN+setReturnValue：getPositions 非 cancellable，CancellationException 会崩 feature 放置——260905-10 实测）。
 */
@Mixin(SquarePlacementModifier.class)
public abstract class SquarePlacementModifierMixin {

    @Redirect(method = "getPositions", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/util/math/random/Random;nextInt(I)I", ordinal = 0))
    private int wg$sqX(Random random, int bound, FeaturePlacementContext context, Random random2, BlockPos pos) {
        int v = random.nextInt(bound);
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            // 260905-12 提速：按 pos 所在 chunk 过滤（同 THJ 家族）
            if (wg.bench.WgDiag.posAllowed(pos.getX(), pos.getZ())) {
                wg.bench.WgDiag.banner();
                System.out.println("[SQX] " + (v + pos.getX()));
            }
        }
        return v;
    }

    @Redirect(method = "getPositions", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/util/math/random/Random;nextInt(I)I", ordinal = 1))
    private int wg$sqZ(Random random, int bound, FeaturePlacementContext context, Random random2, BlockPos pos) {
        int v = random.nextInt(bound);
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            // 260905-12 提速：按 pos 所在 chunk 过滤（同 THJ 家族）
            if (wg.bench.WgDiag.posAllowed(pos.getX(), pos.getZ())) {
                wg.bench.WgDiag.banner();
                System.out.println("[SQZ] " + (v + pos.getZ()));
            }
        }
        return v;
    }
}
