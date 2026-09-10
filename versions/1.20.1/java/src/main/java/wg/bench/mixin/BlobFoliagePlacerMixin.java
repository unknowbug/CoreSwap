package wg.bench.mixin;

import net.minecraft.util.math.random.Random;
import net.minecraft.world.gen.foliage.BlobFoliagePlacer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * 260905-13 jungle_l .b3 决定性实验（b3-experiment.md）：blob 四角 nextInt(2) 消费打点。
 * 仅在真角（dx==radius && dz==radius，Java && 短路在 RNG 前）时打 1 行 = 1 次消费，
 * 与 rust tree.rs [CORN-BLOB] 同格式对拍：y=层内相对 y, r=radius, dx/dz=归一化前原值。
 * 注意：BushFoliagePlacer 覆写本方法 → 由 BushFoliagePlacerMixin 单独覆盖。
 */
@Mixin(BlobFoliagePlacer.class)
public abstract class BlobFoliagePlacerMixin {

    @Inject(method = "isInvalidForLeaves(Lnet/minecraft/util/math/random/Random;IIIIZ)Z", at = @At("HEAD"))
    private void wg$logCorner(Random random, int dx, int y, int dz, int radius, boolean giantTrunk,
                              CallbackInfoReturnable<Boolean> cir) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            if (dx == radius && dz == radius && wg.bench.WgDiag.curAllowed()) {
                wg.bench.WgDiag.banner();
                System.out.println("[CORN-BLOB] y=" + y + " r=" + radius + " dx=" + dx + " dz=" + dz);
            }
        }
    }
}
