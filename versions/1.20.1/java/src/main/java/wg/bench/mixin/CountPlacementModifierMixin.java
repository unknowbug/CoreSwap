package wg.bench.mixin;

import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import net.minecraft.world.gen.placementmodifier.CountPlacementModifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * 260905-10 P2 逐树 RNG 对拍（b1 §4 模板）：count 抽样打点。
 * 门控 = env WG_TREEDIAG 或系统属性 wg.treediag（默认关）。
 * 输出 [CNT] n 与 Rust 侧 placement.rs Count 打点同格式对拍。
 */
@Mixin(CountPlacementModifier.class)
public abstract class CountPlacementModifierMixin {

    @Inject(method = "getCount", at = @At("RETURN"))
    private void wg$logCount(Random random, BlockPos pos, CallbackInfoReturnable<Integer> cir) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            // 260905-12 提速：按 pos 所在 chunk 过滤（同 THJ 家族）
            if (wg.bench.WgDiag.posAllowed(pos.getX(), pos.getZ())) {
                wg.bench.WgDiag.banner();
                System.out.println("[CNT] " + cir.getReturnValue());
            }
        }
    }
}
