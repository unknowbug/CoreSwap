package wg.bench.mixin;

import net.minecraft.world.gen.treedecorator.BeehiveTreeDecorator;
import net.minecraft.world.gen.treedecorator.TreeDecorator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * 260905-10 P2 判别（.bB 发现：rust 漏实现 BeehiveTreeDecorator，每棵 bees 树少抽 1 次）：
 * java 侧 beehive decorator 消费点打点。
 * 门控 = sysprop wg.treediag / env WG_TREEDIAG（默认关）。输出 [BEE] nextFloat 前标记。
 */
@Mixin(BeehiveTreeDecorator.class)
public abstract class BeehiveDecoratorMixin {

    @Inject(method = "generate", at = @At("HEAD"))
    private void wg$logBee(TreeDecorator.Generator generator, CallbackInfo ci) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            // 260905-12 提速：chunk 过滤（同 THJ）
            if (wg.bench.WgDiag.curAllowed()) {
                wg.bench.WgDiag.banner();
                System.out.println("[BEE]");
            }
        }
    }
}
