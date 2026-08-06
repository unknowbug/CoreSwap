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
        } catch (Exception ex) {
            System.out.println("getColumnSample threw " + ex);
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
        // 附近几列地表（结构周围 5x5）
        System.out.println("周围 5x5 列地表高度（x 步进 4, z 步进 4）:");
        for (int dz = -8; dz <= 8; dz += 4) {
            StringBuilder sb = new StringBuilder("  z=" + (z + dz) + ": ");
            for (int dx = -8; dx <= 8; dx += 4) {
                int t = Integer.MIN_VALUE;
                for (int y = maxY - 1; y >= minY; y--) {
                    if (!chunk.getBlockState(new net.minecraft.util.math.BlockPos(x + dx, y, z + dz)).isAir()) { t = y; break; }
                }
                sb.append(t == Integer.MIN_VALUE ? "A " : t + " ");
            }
            System.out.println(sb);
        }
    }
}
