package wg.bench;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.Identifier;
import net.minecraft.world.chunk.ChunkStatus;

/**
 * CoreSwap 诊断（H1）：-Dblob.probe=1 时对 nether 目标 chunk 区（默认 (3200,3208) 起 4×4，
 * 与 BlockProbe 同参数源 bench.originX/bench.originZ/bench.size）强制 FULL 生成，
 * 触发 FEATURES 阶段 → BlobProbeMixin 落盘 origins。chunk 区外扩 1 圈防边界 blob 丢失。
 */
public final class BlobProbe {
    public static void run(MinecraftServer server) {
        RegistryKey<net.minecraft.world.World> key =
                RegistryKey.of(RegistryKeys.WORLD, new Identifier("the_nether"));
        ServerWorld nether = server.getWorld(key);
        if (nether == null) {
            System.out.println("[BLOB-PROBE] nether world not found");
            return;
        }
        System.out.println("[BLOB-PROBE] start worldSeed=" + nether.getSeed());
        int cx0 = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int cz0 = Integer.parseInt(System.getProperty("bench.originZ", "3208"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        for (int x = cx0 - 1; x <= cx0 + size; x++) {
            for (int z = cz0 - 1; z <= cz0 + size; z++) {
                try {
                    nether.getChunk(x, z, ChunkStatus.FULL, true);
                } catch (Throwable t) {
                    System.out.println("[BLOB-PROBE] chunk(" + x + "," + z + ") failed: " + t);
                }
            }
        }
        int calls = -1, written = -1;
        try {
            java.lang.reflect.Field fc = Class.forName("wg.bench.mixin.BlobProbeMixin").getDeclaredField("CALLS");
            fc.setAccessible(true);
            calls = fc.getInt(null);
            java.lang.reflect.Field fw = Class.forName("wg.bench.mixin.BlobProbeMixin").getDeclaredField("WRITTEN");
            fw.setAccessible(true);
            written = fw.getInt(null);
        } catch (Throwable t) {
            System.out.println("[BLOB-PROBE] stats read failed: " + t);
        }
        System.out.println("[BLOB-PROBE] done region chunk(" + cx0 + "," + cz0 + ") size=" + size
                + " mixinCalls=" + calls + " written=" + written);
        server.stop(false);
    }
}
