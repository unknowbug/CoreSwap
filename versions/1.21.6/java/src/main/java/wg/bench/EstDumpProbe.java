package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.world.chunk.ChunkStatus;

/**
 * CoreSwap 诊断（260903-12，P1.2 shared 臂裁决参照驱动）：预生成指定 chunk region（FULL，
 * 触发完整管线含 SURFACE/aquifer 的 estimateSurfaceHeight 调用），配合 EstDumpProbeMixin 采集
 * Java 权威 est 值。用完即停服。
 * 用法：-Destdump.probe=1 -Destdump.originX=200 -Destdump.originZ=200 -Destdump.size=8
 */
public class EstDumpProbe {
    public static void run(MinecraftServer server) {
        int originX = Integer.parseInt(System.getProperty("estdump.originX", "200"));
        int originZ = Integer.parseInt(System.getProperty("estdump.originZ", "200"));
        int size = Integer.parseInt(System.getProperty("estdump.size", "8"));
        var world = server.getOverworld();
        System.out.println("[EstDumpProbe] seed=" + world.getSeed() + " region=(" + originX + "," + originZ
                + ") size=" + size + " (chunk coords)");
        for (int cz = 0; cz < size; cz++) {
            for (int cx = 0; cx < size; cx++) {
                world.getChunk(originX + cx, originZ + cz, ChunkStatus.FULL, true);
            }
        }
        System.out.println("[EstDumpProbe] done, " + (size * size) + " chunks");
        server.stop(false);
    }
}
