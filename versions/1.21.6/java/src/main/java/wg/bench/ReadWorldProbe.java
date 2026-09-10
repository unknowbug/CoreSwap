package wg.bench;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.registry.Registries;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.World;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;

import java.io.BufferedInputStream;
import java.io.DataInputStream;
import java.io.FileInputStream;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * 读回已生成的 world（C++ 替换模式产物），对比 vanilla 参照验证一致性。
 * 用法：gradle runServer -PreadWorldProbe=true（world 须已由 -PcppReplace=true 生成）
 */
public class ReadWorldProbe {
    private static final int MIN_Y = -64, HEIGHT = 384;

    public static void run(net.minecraft.server.MinecraftServer server) {
        long seed = Long.parseLong(System.getProperty("bench.seed", "-8248318472910187742"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        int originX = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int originZ = Integer.parseInt(System.getProperty("bench.originZ", "3208"));
        String dataDir = System.getProperty("bench.out", "E:/PYTHON/CoreSwap/versions/1.20.1/data");
        // nether 支持（2026-09-04，复用 -DblockProbe.dimension）：维度 world + 动态 min_y/height + _nether 参照后缀
        String dim = System.getProperty("blockProbe.dimension", "overworld");
        boolean nether = dim.equals("nether");
        Path vanilla = Path.of(dataDir, "vanilla_" + seed + "_" + size + "_" + originX + "_" + originZ
                + (nether ? "_nether" : "") + ".blocks");
        if (!Files.exists(vanilla)) {
            System.out.println("[ReadWorldProbe] missing vanilla reference: " + vanilla);
            server.stop(false);
            return;
        }

        World world = nether ? server.getWorld(World.NETHER) : server.getOverworld();
        int minY = world.getBottomY();
        int height = world.getHeight();
        System.out.println("[ReadWorldProbe] dim=" + dim + " min_y=" + minY + " height=" + height);
        BlockPos.Mutable pos = new BlockPos.Mutable();
        long total = 0, match = 0, totalNonAir = 0, matchNonAir = 0;
        long[] layerTotal = new long[height], layerMatch = new long[height];
        int shown = 0;
        // mismatch 全集导出（V5 残差图，-Dbench.mismatchOut=<csv>）：x,y,z,vanillaRawId,worldRawId
        java.io.PrintWriter mismatchOut = null;
        String mismatchPath = System.getProperty("bench.mismatchOut", "");
        if (!mismatchPath.isEmpty()) {
            try {
                mismatchOut = new java.io.PrintWriter(Files.newBufferedWriter(Path.of(mismatchPath)));
                mismatchOut.println("#seed=" + seed + " dim=" + dim + " size=" + size + " origin=" + originX + "," + originZ);
            } catch (Exception e) {
                System.out.println("[ReadWorldProbe] mismatch csv open failed: " + e);
            }
        }
        try (DataInputStream in = new DataInputStream(new BufferedInputStream(new FileInputStream(vanilla.toFile())))) {
            in.readInt(); in.readLong();
            in.readInt(); in.readInt(); in.readInt();
            in.readInt(); in.readInt();
            for (int ci = 0; ci < size * size; ci++) {
                int wx = in.readInt(), wz = in.readInt();
                Chunk chunk = world.getChunk(wx, wz, ChunkStatus.FULL, true);
                // 文件布局 = 块数据在前 + chunk 尾 256 列 biome 名；先整 chunk 读入内存再对比
                int[] vVals = new int[16 * 16 * height];
                for (int k = 0; k < 16 * 16 * height; k++) vVals[k] = in.readUnsignedShort();
                String[] vBiomes = new String[256];
                for (int b = 0; b < 256; b++) vBiomes[b] = in.readUTF();
                for (int k = 0; k < 16 * 16 * height; k++) {
                    int by = k / 256, z = (k % 256) / 16, x = k % 16;
                    int v = vVals[k];
                    BlockState st = chunk.getBlockState(pos.set(x, minY + by, z));
                    int raw = Registries.BLOCK.getRawId(st.getBlock());
                    total++;
                    int yIdx = by;
                    layerTotal[yIdx]++;
                    if (v != 0) { totalNonAir++; if (v == raw) matchNonAir++; }
                    if (v == raw) { match++; layerMatch[yIdx]++; }
                    else {
                        if (shown < 15) {
                            shown++;
                            String vb = v == 0 ? "air" : Registries.BLOCK.getId(Registries.BLOCK.get(v)).toString();
                            String wb = raw == 0 ? "air" : Registries.BLOCK.getId(Registries.BLOCK.get(raw)).toString();
                            System.out.println("[ReadWorldProbe] mismatch chunk(" + wx + "," + wz + ") (" + (chunk.getPos().getStartX() + x) + "," + (minY + by) + "," + (chunk.getPos().getStartZ() + z) + ") vanilla=" + vb + " world=" + wb);
                        }
                        if (mismatchOut != null) {
                            // ⚠️ biome 查询必须用世界坐标（260902-04 修复：曾误用 chunk 局部 x,z，
                            // 全部查到 chunk(0,0) 区域 biome = warped_forest → 35426 行全 warped 假象）
                            String wBiome = world.getBiome(pos.set(chunk.getPos().getStartX() + x, minY + by, chunk.getPos().getStartZ() + z)).getKey()
                                    .map(k2 -> k2.getValue().toString()).orElse("?");
                            mismatchOut.println((chunk.getPos().getStartX() + x) + "," + (minY + by) + "," + (chunk.getPos().getStartZ() + z) + "," + v + "," + raw + "," + vBiomes[z * 16 + x] + "," + wBiome);
                        }
                    }
                }
            }
        } catch (Exception e) {
            System.out.println("[ReadWorldProbe] failed: " + e);
        }
        if (mismatchOut != null) { mismatchOut.flush(); mismatchOut.close(); System.out.println("[ReadWorldProbe] mismatch csv: " + mismatchPath); }
        System.out.printf("[ReadWorldProbe] world-vs-vanilla: match=%d/%d (%.4f%%) nonAir=%d/%d (%.4f%%)%n",
                match, total, 100.0 * match / total, matchNonAir, totalNonAir,
                totalNonAir == 0 ? 0 : 100.0 * matchNonAir / totalNonAir);
        System.out.print("[ReadWorldProbe] layerMatch%: ");
        for (int yIdx = 0; yIdx < height; yIdx += 32) {
            long mt = 0, tt = 0;
            for (int yy = yIdx; yy < Math.min(yIdx + 32, height); yy++) { mt += layerMatch[yy]; tt += layerTotal[yy]; }
            System.out.printf("y=%d..%d:%.0f%% ", minY + yIdx, minY + Math.min(yIdx + 31, height - 1), tt == 0 ? 0 : 100.0 * mt / tt);
        }
        System.out.println();
        server.stop(false);
    }
}
