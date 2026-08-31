package wg.bench;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.Identifier;
import net.minecraft.world.chunk.ChunkStatus;

/**
 * CoreSwap 诊断（M14 下界怪异城）：-Dwg.diagNether=1 时对下界 chunk(-5,-3)
 * 周边 3×3..7×7 范围强制 FULL 生成（触发完整管线：NOISE(mixin)→SURFACE→CARVERS→FEATURES），
 * 配合 -Dwg.dumpbiome=1 观察 feature 装饰阶段的 biome 上下文。跑完自动停服。
 */
public final class DiagNetherProbe {
    public static void run(MinecraftServer server) {
        RegistryKey<net.minecraft.world.World> key =
                RegistryKey.of(RegistryKeys.WORLD, new Identifier("the_nether"));
        ServerWorld nether = server.getWorld(key);
        if (nether == null) {
            System.out.println("[WG-DIAGNETHER] nether world not found");
            return;
        }
        System.out.println("[WG-DIAGNETHER] start worldSeed=" + nether.getSeed());
        // overworld 也要在 SERVER_STARTED 后强制生成（spawn 区在 init 之前已被 vanilla 生成）
        ServerWorld over = server.getOverworld();
        if (over != null) {
            for (int x = 9; x <= 11; x++) {
                for (int z = 9; z <= 11; z++) {
                    try {
                        over.getChunk(x, z, ChunkStatus.FULL, true);
                        System.out.println("[WG-DIAGNETHER] overworld chunk(" + x + "," + z + ") FULL ok");
                    } catch (Throwable t) {
                        System.out.println("[WG-DIAGNETHER] overworld chunk(" + x + "," + z + ") failed: " + t);
                    }
                }
            }
        }
        int cx = -5, cz = -3, r = 1;
        for (int x = cx - r; x <= cx + r; x++) {
            for (int z = cz - r; z <= cz + r; z++) {
                try {
                    nether.getChunk(x, z, ChunkStatus.FULL, true);
                    System.out.println("[WG-DIAGNETHER] chunk(" + x + "," + z + ") FULL ok");
                } catch (Throwable t) {
                    System.out.println("[WG-DIAGNETHER] chunk(" + x + "," + z + ") failed: " + t);
                }
            }
        }
        System.out.println("[WG-DIAGNETHER] done");
        server.stop(false);
    }
}
