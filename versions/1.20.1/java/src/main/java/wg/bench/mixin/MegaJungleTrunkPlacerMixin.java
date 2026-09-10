package wg.bench.mixin;

import java.util.List;
import java.util.function.BiConsumer;
import net.minecraft.block.BlockState;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import net.minecraft.world.TestableWorld;
import net.minecraft.world.gen.feature.TreeFeatureConfig;
import net.minecraft.world.gen.foliage.FoliagePlacer;
import net.minecraft.world.gen.trunk.MegaJungleTrunkPlacer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

/**
 * j5-rng-trace-plan-260906-01：mega 树逐消费 [MJT0]/[MJTI]/[MJTF]/[MJTS]/[MJTX]。
 * 门控 = sysprop wg.treediag / env WG_TREEDIAG（默认关）；chunk 过滤走 WgDiag.curAllowed()。
 * 对拍对象 = Rust tree.rs mega_jungle_trunk [MJT*] 打点（格式逐字段对齐，含逗号后空格）。
 */
@Mixin(MegaJungleTrunkPlacer.class)
public abstract class MegaJungleTrunkPlacerMixin {
    private static boolean wg$on() {
        return Boolean.getBoolean("wg.treediag") || System.getenv("WG_TREEDIAG") != null;
    }
    private static boolean wg$gate() {
        if (!wg$on()) return false;
        if (wg.bench.WgDiag.curAllowed()) { wg.bench.WgDiag.banner(); return true; }
        return false;
    }

    // generate 入口：树起点 + height
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At("HEAD"))
    private void wg$mjt0(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJT0] t=(" + startPos.getX() + ", " + startPos.getY() + ", " + startPos.getZ() + ") h=" + height);
        }
    }

    // [MJTI]/[MJTF]/[MJTS] capture 注入已删（260906-01：LVT incompatible ×2，回退最低保障集预案
    // j5-rng-trace-plan-260906-01 §5.2；i/f 值以 Rust 侧为全量参照，Java 侧形状判据走 T5 + MJTG 序列）。

    // 收尾：node 数（几何细节由 MJTI/MJTB 对拍，不取 TreeNode 内部字段降低编译风险）
    @Inject(method = "generate(Lnet/minecraft/world/TestableWorld;Ljava/util/function/BiConsumer;Lnet/minecraft/util/math/random/Random;ILnet/minecraft/util/math/BlockPos;Lnet/minecraft/world/gen/feature/TreeFeatureConfig;)Ljava/util/List;",
            at = @At("RETURN"))
    private void wg$mjtx(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random,
                         int height, BlockPos startPos, TreeFeatureConfig config,
                         CallbackInfoReturnable<List<FoliagePlacer.TreeNode>> cir) {
        if (wg$gate()) {
            System.out.println("[MJTX] nodes=" + cir.getReturnValue().size() + " t=(" + startPos.getX() + ", " + startPos.getY() + ", " + startPos.getZ() + ")");
        }
    }
}
