package wg.bench.mixin;

import net.minecraft.util.math.random.ChunkRandom;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * 260905-08 E-C2 判别：p-index 对拍钩子。
 * 打点 setPopulationSeed / setDecoratorSeed 的入参与返回值，门控 = env WG_SEEDLOG 或系统属性 WG_SEEDLOG。
 * 用途：与 Rust 侧 WG_FEATURELOG 的 [FEATURE] (k,p,fid) 对拍，验证 indexer 构建序差（registry 序 vs 字典序）。
 */
@Mixin(ChunkRandom.class)
public abstract class ChunkRandomSeedLogMixin {
    private static boolean wg$seedlog() {
        return Boolean.getBoolean("wg.seedlog") || System.getenv("WG_SEEDLOG") != null;
    }

    @Inject(method = "setPopulationSeed", at = @At("RETURN"))
    private void wg$logPop(long seed, int x, int z, CallbackInfoReturnable<Long> cir) {
        // 260905-12 提速：population 行是每 chunk 恒一条的锚点，恒写线程当前 chunk（廉价）；
        // 自身不滤（非噪声源），供 THJ/BEE 等无坐标打点做 chunk 门。
        wg.bench.WgDiag.setCurrent(x, z);
        if (wg$seedlog()) {
            System.out.println("[SEEDLOG] population " + cir.getReturnValue() + " chunk " + x + " " + z);
        }
    }

    @Inject(method = "setDecoratorSeed", at = @At("RETURN"))
    private void wg$logDeco(long populationSeed, int index, int step, CallbackInfo ci) {
        if (wg$seedlog()) {
            System.out.println("[SEEDLOG] decorator p=" + index + " k=" + step + " pop=" + populationSeed);
        }
    }
}
