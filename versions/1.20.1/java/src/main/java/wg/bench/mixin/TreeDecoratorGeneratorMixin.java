package wg.bench.mixin;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.gen.treedecorator.TreeDecorator;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * 260905-13 jungle_l .b2 决定性实验（b2-experiment.md §2）：
 * TreeDecorator.Generator 构造器末尾 dump Y 稳定排序后的最终 list 序。
 * 与 rust 侧 tree.rs [TREESET]/[TREELEAF]（插入序）对拍 → 同 Y tie 序比对。
 * 门控 = sysprop wg.treediag / env WG_TREEDIAG；chunk 过滤 = WgDiag.curAllowed()。
 * 格式：[TREESET] t0=(x,y,z) trunk=x:y,z|x:y,z...（t0 = 排序后首 log，供配树参考）
 */
@Mixin(TreeDecorator.Generator.class)
public abstract class TreeDecoratorGeneratorMixin {

    @Shadow @Final private ObjectArrayList<BlockPos> logPositions;
    @Shadow @Final private ObjectArrayList<BlockPos> leavesPositions;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void wg$dumpSets(CallbackInfo ci) {
        if (Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null) {
            if (wg.bench.WgDiag.curAllowed()) {
                wg.bench.WgDiag.banner();
                StringBuilder logs = new StringBuilder();
                for (BlockPos p : this.logPositions) {
                    if (logs.length() > 0) logs.append('|');
                    logs.append(p.getX()).append(':').append(p.getY()).append(',').append(p.getZ());
                }
                StringBuilder leaves = new StringBuilder();
                for (BlockPos p : this.leavesPositions) {
                    if (leaves.length() > 0) leaves.append('|');
                    leaves.append(p.getX()).append(':').append(p.getY()).append(',').append(p.getZ());
                }
                String t0 = this.logPositions.isEmpty() ? "none"
                        : "(" + this.logPositions.get(0).getX() + "," + this.logPositions.get(0).getY()
                          + "," + this.logPositions.get(0).getZ() + ")";
                System.out.println("[TREESET] t0=" + t0 + " trunk=" + logs);
                System.out.println("[TREELEAF] t0=" + t0 + " leaves=" + leaves);
            }
        }
    }
}
