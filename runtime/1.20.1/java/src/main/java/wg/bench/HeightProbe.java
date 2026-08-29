package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.world.Heightmap;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.chunk.ChunkGenerator;

/**
 * 结构高度探针：对比 ChunkGenerator.getHeightOnWorld（结构放置用，扫 density）
 * 与实际方块地表（列最高非空气）——定位「村庄帐篷悬空」（结构放 y=69 vs 地表 y=62）。
 * 用法：-Dheight.probe=true -Dheight.x=20 -Dheight.z=-468（默认 20,-468 玩家/塔楼位置）
 */
public class HeightProbe {
    public static void run(MinecraftServer server) {
        ServerWorld world = server.getOverworld();
        ChunkGenerator cg = world.getChunkManager().getChunkGenerator();
        int x = 20, z = -468;
        if (System.getProperty("height.x") != null) x = Integer.parseInt(System.getProperty("height.x"));
        if (System.getProperty("height.z") != null) z = Integer.parseInt(System.getProperty("height.z"));
        int cx = Math.floorDiv(x, 16), cz = Math.floorDiv(z, 16);
        System.out.println("=== HeightProbe 结构高度对比 (" + x + ", " + z + ") chunk(" + cx + "," + cz + ") ===");
        for (Heightmap.Type t : Heightmap.Type.values()) {
            try {
                int h = cg.getHeight(x, z, t, world, world.getChunkManager().getNoiseConfig());
                System.out.println(t + ": getHeight=" + h);
            } catch (Exception ex) {
                System.out.println(t + ": getHeightOnWorld threw " + ex);
            }
        }
        try {
            var col = cg.getColumnSample(x, z, world, world.getChunkManager().getNoiseConfig());
            System.out.println("getColumnSample: " + col);
            // 列 dump：y 55..75
            for (int y = 55; y <= 75; y++) {
                System.out.println("  col y=" + y + ": " + col.getState(y));
            }
        } catch (Throwable ex) {
            System.out.println("getColumnSample threw: " + ex);
            for (StackTraceElement e : ex.getStackTrace()) System.out.println("    at " + e);
        }
        // 实际方块地表：加载 chunk 读列最高非空气
        Chunk chunk = world.getChunk(cx, cz);
        int top = Integer.MIN_VALUE;
        int minY = chunk.getBottomY();
        int maxY = minY + chunk.getHeight();
        for (int y = maxY - 1; y >= minY; y--) {
            if (!chunk.getBlockState(new net.minecraft.util.math.BlockPos(x, y, z)).isAir()) { top = y; break; }
        }
        System.out.println("实际方块地表 top=" + top + " (minY=" + minY + " maxY=" + maxY + ")");
        // 扫整个 chunk 找村庄结构方块（white_wool=帐篷/建筑、oak_log=塔楼）的 y 分布
        int woolMin = Integer.MAX_VALUE, woolMax = Integer.MIN_VALUE, woolCount = 0;
        int logMax = Integer.MIN_VALUE;
        for (int bx = 0; bx < 16; bx++) {
            for (int bz = 0; bz < 16; bz++) {
                for (int y = maxY - 1; y >= minY; y--) {
                    var bs = chunk.getBlockState(new net.minecraft.util.math.BlockPos(x - (x & 15) + bx, y, z - (z & 15) + bz));
                    String nm = bs.getBlock().getTranslationKey();
                    if (nm.contains("white_wool")) {
                        woolCount++;
                        if (y < woolMin) woolMin = y;
                        if (y > woolMax) woolMax = y;
                    }
                    if (nm.contains("oak_log") && y > logMax) logMax = y;
                }
            }
        }
        System.out.println("chunk 结构方块: white_wool y=" + (woolMin == Integer.MAX_VALUE ? "无" : woolMin + ".." + woolMax) + " 数量=" + woolCount
                + "  oak_log 最高 y=" + (logMax == Integer.MIN_VALUE ? "无" : logMax));
    }
}
