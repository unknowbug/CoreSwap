package wg.bench.mixin;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.gen.feature.FeaturePlacementContext;
import net.minecraft.world.gen.placementmodifier.CountMultilayerPlacementModifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.StringJoiner;

/**
 * CoreSwap 诊断（H1 终审 (e)）：对首分叉邻域列，只读复刻 findPos 的转换面序列。
 * @Mixin(CountMultilayerPlacementModifier) @Inject(getPositions HEAD)——不消费 Random，
 * 不改返回；对 colprof.x/z 半径 colprof.r 内、且落在本 chunk 的列 dump：
 * MOTION_BLOCKING top + 自顶向下全部「spawn(air/water/lava)->非spawn(非bedrock)」转换面
 * [y,aboveRawId->belowRawId] + 转换数 n。两轮 diff：T[] 差 = (e) 实锤，diff 项直接指认残差本体。
 * -Dcolprof.probe=1 门控。
 */
@Mixin(CountMultilayerPlacementModifier.class)
public abstract class ColProfProbeMixin {

    @Unique private static final boolean COLPROF = System.getProperty("colprof.probe") != null;
    @Unique private static final int CX = Integer.parseInt(System.getProperty("colprof.x", "51178"));
    @Unique private static final int CZ = Integer.parseInt(System.getProperty("colprof.z", "51319"));
    @Unique private static final int CR = Integer.parseInt(System.getProperty("colprof.r", "2"));
    // lavaAudit 模式：-Dcolprof.mode=lavaAudit——每 chunk 只扫一次（去重），全 16x16 列，
    // 仅输出含 lava 转换的列：[LAVAAUDIT] x,z,lavaTopY,nTransitions（r=64 无意义，按 chunk 全扫）
    @Unique private static final boolean LAVA_AUDIT = "lavaAudit".equals(System.getProperty("colprof.mode"));
    @Unique private static final java.util.Set<Long> AUDIT_DONE =
            java.util.Collections.newSetFromMap(new java.util.concurrent.ConcurrentHashMap<Long, Boolean>());
    @Unique private static boolean MAPPING_DONE = false;

    @Inject(method = "getPositions", at = @At("HEAD"), require = 1)
    private void wgColProf(FeaturePlacementContext context,
                           net.minecraft.util.math.random.Random random, BlockPos pos,
                           CallbackInfoReturnable<java.util.stream.Stream<BlockPos>> cir) {
        if (!COLPROF) return;
        if (LAVA_AUDIT) {
            int baseX = pos.getX(), baseZ = pos.getZ();
            long ck = ((long)(baseX >> 4) << 32) | ((baseZ >> 4) & 0xFFFFFFFFL);
            if (!AUDIT_DONE.add(ck)) return; // 每 chunk 只扫一次
            if (!MAPPING_DONE) { // 一次性打印关键方块 raw id 映射（防 id 标注误读）
                MAPPING_DONE = true;
                System.out.println("[LAUIDMAP] air=" + Block.STATE_IDS.getRawId(Blocks.AIR.getDefaultState())
                        + " lava=" + Block.STATE_IDS.getRawId(Blocks.LAVA.getDefaultState())
                        + " water=" + Block.STATE_IDS.getRawId(Blocks.WATER.getDefaultState())
                        + " netherrack=" + Block.STATE_IDS.getRawId(Blocks.NETHERRACK.getDefaultState())
                        + " basalt=" + Block.STATE_IDS.getRawId(Blocks.BASALT.getDefaultState())
                        + " blackstone=" + Block.STATE_IDS.getRawId(Blocks.BLACKSTONE.getDefaultState())
                        + " soulsand=" + Block.STATE_IDS.getRawId(Blocks.SOUL_SAND.getDefaultState())
                        + " soulsoil=" + Block.STATE_IDS.getRawId(Blocks.SOUL_SOIL.getDefaultState())
                        + " magma=" + Block.STATE_IDS.getRawId(Blocks.MAGMA_BLOCK.getDefaultState())
                        + " bedrock=" + Block.STATE_IDS.getRawId(Blocks.BEDROCK.getDefaultState()));
            }
            try {
                for (int x = baseX; x < baseX + 16; x++) {
                    for (int z = baseZ; z < baseZ + 16; z++) {
                        int m = context.getTopY(net.minecraft.world.Heightmap.Type.MOTION_BLOCKING, x, z);
                        int lavaTopY = Integer.MIN_VALUE;
                        int lavaSurfY = Integer.MIN_VALUE;
                        int n = 0;
                        BlockPos.Mutable mutable = new BlockPos.Mutable(x, m, z);
                        BlockState above = context.getWorld().getBlockState(mutable);
                        for (int y = m; y >= context.getWorld().getBottomY() + 1; y--) {
                            mutable.setY(y - 1);
                            BlockState below = context.getWorld().getBlockState(mutable);
                            boolean spawnAbove = above.isAir() || above.isOf(Blocks.WATER) || above.isOf(Blocks.LAVA);
                            boolean spawnBelow = below.isAir() || below.isOf(Blocks.WATER) || below.isOf(Blocks.LAVA);
                            if (!spawnBelow && spawnAbove && !below.isOf(Blocks.BEDROCK)) {
                                n++;
                                if (above.isOf(Blocks.LAVA) && lavaTopY == Integer.MIN_VALUE) lavaTopY = y;
                                if (below.isOf(Blocks.LAVA) && lavaSurfY == Integer.MIN_VALUE) lavaSurfY = y;
                            }
                            above = below;
                        }
                        if (lavaTopY != Integer.MIN_VALUE || lavaSurfY != Integer.MIN_VALUE) {
                            System.out.println("[LAVAAUDIT] " + x + "," + z + "," + lavaSurfY + "," + lavaTopY + "," + n);
                        }
                    }
                }
            } catch (Throwable t) {
                System.out.println("[LAVAAUDIT] failed: " + t);
            }
            return;
        }
        try {
            int baseX = pos.getX(), baseZ = pos.getZ();
            for (int dx = -CR; dx <= CR; dx++) {
                for (int dz = -CR; dz <= CR; dz++) {
                    int k = CX + dx, l = CZ + dz;
                    if ((k >> 4) != (baseX >> 4) || (l >> 4) != (baseZ >> 4)) continue;
                    if (k < baseX || k >= baseX + 16 || l < baseZ || l >= baseZ + 16) continue;
                    int m = context.getTopY(net.minecraft.world.Heightmap.Type.MOTION_BLOCKING, k, l);
                    StringJoiner tj = new StringJoiner(";");
                    int n = 0;
                    BlockPos.Mutable mutable = new BlockPos.Mutable(k, m, l);
                    BlockState above = context.getWorld().getBlockState(mutable);
                    for (int y = m; y >= context.getWorld().getBottomY() + 1; y--) {
                        mutable.setY(y - 1);
                        BlockState below = context.getWorld().getBlockState(mutable);
                        boolean spawnAbove = above.isAir() || above.isOf(Blocks.WATER) || above.isOf(Blocks.LAVA);
                        boolean spawnBelow = below.isAir() || below.isOf(Blocks.WATER) || below.isOf(Blocks.LAVA);
                        if (!spawnBelow && spawnAbove && !below.isOf(Blocks.BEDROCK)) {
                            tj.add(y + "|" + Block.STATE_IDS.getRawId(above) + "->" + Block.STATE_IDS.getRawId(below));
                            n++;
                        }
                        above = below;
                    }
                    System.out.println("[COLPROF] " + k + "," + l + " top=" + m + " n=" + n + " T=" + tj);
                }
            }
        } catch (Throwable t) {
            System.out.println("[COLPROF] failed: " + t);
        }
    }
}
