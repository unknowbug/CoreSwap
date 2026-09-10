package wg.bench;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.Identifier;
import net.minecraft.world.World;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.world.Heightmap;

/**
 * CoreSwap 诊断（H1 decisive probe ③）：nether 目标区每列 dump
 * WORLD_SURFACE_WG / OCEAN_FLOOR_WG 两个 heightmap 口径的顶 y 与自顶向下首个非空气 block rawId。
 * -DsurfaceColDump=1 启用；输出一行：x,z,wsY,wsBlock,ofY,ofBlock
 * 区分 (b) 表面输入差 vs (c) heightmap 查询口径差：两口径一同一异可直接锁定类型。
 */
public final class SurfaceColDumpProbe {
    public static void run(MinecraftServer server) {
        RegistryKey<World> key = RegistryKey.of(RegistryKeys.WORLD, Identifier.of("the_nether"));
        ServerWorld nether = server.getWorld(key);
        if (nether == null) {
            System.out.println("[COL-PROBE] nether world not found");
            return;
        }
        System.out.println("[COL-PROBE] start worldSeed=" + nether.getSeed());
        int cx0 = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int cz0 = Integer.parseInt(System.getProperty("bench.originZ", "3208"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        StringBuilder sb = new StringBuilder();
        String targets = System.getProperty("colDump.targets");
        if (targets != null) {
            // 定点剖面模式：x:z,x:z,... 每列 dump 全 y 的 blockId|fluid（(e)/(f) 终审）
            for (String t : targets.split(",")) {
                String[] p = t.split(":");
                int tx = Integer.parseInt(p[0]), tz = Integer.parseInt(p[1]);
                for (int dx = -2; dx <= 2; dx++) {
                    for (int dz = -2; dz <= 2; dz++) {
                        int wx = tx + dx, wz = tz + dz;
                        try {
                            nether.getChunk(wx >> 4, wz >> 4, ChunkStatus.FULL, true);
                        } catch (Throwable ignored) {
                        }
                        Chunk ch = nether.getChunk(wx >> 4, wz >> 4, ChunkStatus.FULL, false);
                        if (ch == null) continue;
                        int lx = wx & 15, lz = wz & 15;
                        for (int y = ch.getBottomY(); y < ch.getBottomY() + ch.getHeight(); y++) {
                            var st = ch.getBlockState(new net.minecraft.util.math.BlockPos(lx, y, lz));
                            var fs = ch.getFluidState(new net.minecraft.util.math.BlockPos(lx, y, lz));
                            sb.append(wx).append(',').append(y).append(',').append(wz).append(',')
                              .append(net.minecraft.block.Block.STATE_IDS.getRawId(st)).append('|')
                              .append(fs.isEmpty() ? "-" : fs.toString()).append('\n');
                        }
                    }
                }
            }
            writeOut(sb, "profile");
            server.stop(false);
            return;
        }
        for (int x = cx0 - 1; x <= cx0 + size; x++) {
            for (int z = cz0 - 1; z <= cz0 + size; z++) {
                try {
                    nether.getChunk(x, z, ChunkStatus.FULL, true);
                } catch (Throwable t) {
                    System.out.println("[COL-PROBE] chunk(" + x + "," + z + ") failed: " + t);
                }
            }
        }
        Heightmap.Type[] types = {Heightmap.Type.WORLD_SURFACE_WG, Heightmap.Type.OCEAN_FLOOR_WG,
                Heightmap.Type.MOTION_BLOCKING, Heightmap.Type.MOTION_BLOCKING_NO_LEAVES};
        for (int cx = cx0 - 1; cx <= cx0 + size; cx++) {
            for (int cz = cz0 - 1; cz <= cz0 + size; cz++) {
                Chunk chunk;
                try {
                    chunk = nether.getChunk(cx, cz, ChunkStatus.FULL, false);
                } catch (Throwable t) {
                    continue;
                }
                if (chunk == null) continue;
                Heightmap ws = chunk.getHeightmap(types[0]);
                Heightmap of = chunk.getHeightmap(types[1]);
                Heightmap mb = chunk.getHeightmap(types[2]);
                Heightmap mbnl = chunk.getHeightmap(types[3]);
                for (int lx = 0; lx < 16; lx++) {
                    for (int lz = 0; lz < 16; lz++) {
                        int wx = cx * 16 + lx, wz = cz * 16 + lz;
                        int wsY = ws.get(lx, lz);
                        int ofY = of.get(lx, lz);
                        int mbY = mb.get(lx, lz);
                        int mbnlY = mbnl.get(lx, lz);
                        int wsBlock = topNonAir(chunk, lx, lz, wsY);
                        int ofBlock = topNonAir(chunk, lx, lz, ofY);
                        sb.append(wx).append(',').append(wz).append(',')
                          .append(wsY).append(',').append(wsBlock).append(',')
                          .append(ofY).append(',').append(ofBlock).append(',')
                          .append(mbY).append(',').append(mbnlY).append('\n');
                    }
                }
            }
        }
        writeOut(sb, "done");
        server.stop(false);
    }

    private static void writeOut(StringBuilder sb, String tag) {
        try {
            java.nio.file.Path out = java.nio.file.Path.of(
                    System.getProperty("colDump.out", ".tmp/blob-probe/cols.csv"))
                    .toAbsolutePath().normalize();
            if (out.getParent() != null) java.nio.file.Files.createDirectories(out.getParent());
            java.nio.file.Files.writeString(out, sb.toString());
            System.out.println("[COL-PROBE] " + tag + " rows=" + (sb.length() == 0 ? 0 : sb.toString().split("\n").length - 1)
                    + " out=" + out);
        } catch (Throwable t) {
            System.out.println("[COL-PROBE] write failed: " + t);
        }
    }

    /** 自顶向下（从 y 起不含）找首个非空气 block rawId；找不到返回 -1。 */
    private static int topNonAir(Chunk chunk, int lx, int lz, int y) {
        net.minecraft.world.chunk.ChunkSection[] secs = chunk.getSectionArray();
        for (int yy = Math.min(y, chunk.getBottomY() + chunk.getHeight() - 1); yy >= chunk.getBottomY(); yy--) {
            var state = chunk.getBlockState(new net.minecraft.util.math.BlockPos(lx, yy, lz));
            if (!state.isAir()) {
                return net.minecraft.block.Block.STATE_IDS.getRawId(state);
            }
        }
        return -1;
    }
}
