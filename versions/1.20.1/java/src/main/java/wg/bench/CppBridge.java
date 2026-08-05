package wg.bench;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.Registries;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.chunk.Chunk;
import wg.CppWorldgen;

/**
 * CoreSwap worldgen 全局桥：持有 C++ 句柄，把 C++ 生成的整块写入 Chunk。
 * 启用：-Dcpp.replace=1（由 BenchMod 在 server started 时 init）。
 */
public final class CppBridge {
    private static long handle;
    public static boolean enabled;

    private CppBridge() {}

    public static void init(long seed) {
        String dir = System.getProperty("cpp.worldgen.dir", "E:/python/MC/data/worldgen");
        handle = CppWorldgen.init(seed, dir);
        enabled = handle != 0;
        System.out.println("[CppBridge] init seed=" + seed + " worldgenDir=" + dir + " enabled=" + enabled);
    }

    /**
     * 用 C++ 结果整块填充 Chunk（NOISE 阶段的方块 + 高度图）。
     */
    public static void fillChunk(Chunk chunk) {
        if (!enabled) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        int[] buf = new int[16 * 16 * 384];
        int n = CppWorldgen.fillBlocks(handle, new int[]{cx}, new int[]{cz}, new int[][]{buf}, 1);
        if (n != 1) {
            System.out.println("[CppBridge] fillBlocks failed for chunk(" + cx + "," + cz + ")");
            return;
        }
        BlockPos.Mutable pos = new BlockPos.Mutable();
        BlockState air = Blocks.AIR.getDefaultState();
        for (int by = 0; by < 384; by++) {
            int y = -64 + by;
            for (int z = 0; z < 16; z++) {
                for (int x = 0; x < 16; x++) {
                    int id = buf[by * 256 + z * 16 + x];
                    BlockState st = id == 0 ? air : Registries.BLOCK.get(id).getDefaultState();
                    chunk.setBlockState(pos.set(x, y, z), st, false);
                }
            }
        }
    }

    public static void destroy() {
        if (handle != 0) CppWorldgen.destroy(handle);
        handle = 0;
        enabled = false;
    }
}
