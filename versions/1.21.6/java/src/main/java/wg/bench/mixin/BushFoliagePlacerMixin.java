package wg.bench.mixin;

import net.minecraft.util.math.random.Random;
import net.minecraft.world.gen.foliage.BushFoliagePlacer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * 260905-13 jungle_l .b3：jungle_bush 角消费打点（Bush 覆写 Blob.isInvalidForLeaves，独立 mixin）。
 * 覆盖 r=0 退化角（dx=dz=0==radius，b3 勘误：9 行/棵基线含此层 1 消费）。
 * 与 rust tree.rs [CORN-BUSH] 同格式对拍。
 */
@Mixin(BushFoliagePlacer.class)
public abstract class BushFoliagePlacerMixin {

    @Inject(method = "isInvalidForLeaves(Lnet/minecraft/util/math/random/Random;IIIIZ)Z", at = @At("HEAD"))
    private void wg$logCorner(Random random, int dx, int y, int dz, int radius, boolean giantTrunk,
                              CallbackInfoReturnable<Boolean> cir) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            if (dx == radius && dz == radius && wg.bench.WgDiag.curAllowed()) {
                wg.bench.WgDiag.banner();
                System.out.println("[CORN-BUSH] y=" + y + " r=" + radius + " dx=" + dx + " dz=" + dz);
            }
        }
    }
}
