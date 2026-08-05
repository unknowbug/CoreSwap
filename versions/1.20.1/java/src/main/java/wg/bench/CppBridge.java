package wg.bench;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.Registries;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import net.minecraft.world.chunk.Chunk;
import wg.CppWorldgen;

/**
 * CoreSwap worldgen 全局桥：持有 C++ 句柄，把 C++ 生成的整块写入 Chunk。
 * 启用：-Dcpp.replace=1（由 BenchMod 在 server started 时 init）。
 */
public final class CppBridge {
    private static long handle;
    public static boolean enabled;
    private static final boolean DEBUG = System.getProperty("cpp.debug") != null;
    private static final ThreadLocal<int[]> BUF = ThreadLocal.withInitial(() -> new int[16 * 16 * 384]);

    private CppBridge() {}

    public static void init(long seed) {
        String dir = System.getProperty("cpp.worldgen.dir", "E:/python/MC/data/worldgen");
        handle = CppWorldgen.init(seed, dir);
        enabled = handle != 0;
        System.out.println("[CppBridge] init seed=" + seed + " worldgenDir=" + dir + " enabled=" + enabled);
    }

    /**
     * 用 C++ 结果整块填充 Chunk（NOISE 阶段的方块 + 高度图）。
     * 性能：直接写 PalettedContainer（跳过 setBlockState 的 heightmap/blockEntity 开销），
     * 高度图用 populateHeightmaps 一次批量重算——98304 次 setBlockState → 直写。
     */
    public static void fillChunk(Chunk chunk) {
        if (!enabled) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        long t0 = System.nanoTime();
        int[] buf = BUF.get();  // 复用（98304 ints/393KB，每 chunk 分配是 GC 压力）
        int n = CppWorldgen.fillBlocks(handle, new int[]{cx}, new int[]{cz}, new int[][]{buf}, 1);
        long t1 = System.nanoTime();
        if (n != 1) {
            System.out.println("[CppBridge] fillBlocks failed for chunk(" + cx + "," + cz + ")");
            return;
        }
        // id → BlockState 预映射（raw id 上限 < 1024；air=0）
        BlockState[] stateById = new BlockState[1024];
        java.util.Arrays.fill(stateById, Blocks.AIR.getDefaultState());
        BlockState air = Blocks.AIR.getDefaultState();
        // 直写 PalettedContainer（跳过 chunk.setBlockState 的 heightmap/blockEntity 开销）
        // 注意：Chunk.getSection(int) 参数是 section 的世界 Y 坐标（-4..19），不是 0-based index
        net.minecraft.world.chunk.ChunkSection[] sections = new net.minecraft.world.chunk.ChunkSection[24];
        for (int secY = -4; secY < 20; secY++) sections[secY + 4] = chunk.getSection(secY);
        for (int by = 0; by < 384; by++) {
            net.minecraft.world.chunk.PalettedContainer<BlockState> container =
                    sections[by >> 4].getBlockStateContainer();
            int sy = by & 15;
            for (int z = 0; z < 16; z++) {
                int base = by * 256 + z * 16;
                for (int x = 0; x < 16; x++) {
                    int id = buf[base + x];
                    BlockState st = stateById[id];
                    if (st == null) {
                        st = id == 0 ? air : Registries.BLOCK.get(id).getDefaultState();
                        stateById[id] = st;
                    }
                    container.set(x, sy, z, st);
                }
            }
        }
        // 补设高度图（原版 populateNoise 只设 WORLD_SURFACE_WG；buildSurface 被跳过，
        // 需一次性补齐全部，否则 FULL 后的生物生成/寻路/光照依赖错乱）
        Heightmap.populateHeightmaps(chunk, java.util.Set.of(
                Heightmap.Type.WORLD_SURFACE_WG,
                Heightmap.Type.WORLD_SURFACE,
                Heightmap.Type.OCEAN_FLOOR_WG,
                Heightmap.Type.OCEAN_FLOOR,
                Heightmap.Type.MOTION_BLOCKING,
                Heightmap.Type.MOTION_BLOCKING_NO_LEAVES));
        long t2 = System.nanoTime();
        if (DEBUG) System.out.printf("[CppBridge] chunk(%d,%d): C++=%dms write=%dms%n",
                cx, cz, (t1 - t0) / 1_000_000, (t2 - t1) / 1_000_000);
    }

    public static void destroy() {
        if (handle != 0) CppWorldgen.destroy(handle);
        handle = 0;
        enabled = false;
    }
}
