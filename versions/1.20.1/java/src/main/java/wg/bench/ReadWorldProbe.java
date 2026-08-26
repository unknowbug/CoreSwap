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
        String dataDir = System.getProperty("bench.out", "E:/python/MC/data");
        Path vanilla = Path.of(dataDir, "vanilla_" + seed + "_" + size + "_" + originX + "_" + originZ + ".blocks");
        if (!Files.exists(vanilla)) {
            System.out.println("[ReadWorldProbe] missing vanilla reference: " + vanilla);
            server.stop(false);
            return;
        }

        World world = server.getOverworld();
        BlockPos.Mutable pos = new BlockPos.Mutable();
        long total = 0, match = 0, totalNonAir = 0, matchNonAir = 0;
        long[] layerTotal = new long[HEIGHT], layerMatch = new long[HEIGHT];
        int shown = 0;
        try (DataInputStream in = new DataInputStream(new BufferedInputStream(new FileInputStream(vanilla.toFile())))) {
            in.readInt(); in.readLong();
            in.readInt(); in.readInt(); in.readInt();
            in.readInt(); in.readInt();
            for (int ci = 0; ci < size * size; ci++) {
                int wx = in.readInt(), wz = in.readInt();
                Chunk chunk = world.getChunk(wx, wz, ChunkStatus.FULL, true);
                for (int k = 0; k < 16 * 16 * HEIGHT; k++) {
                    int by = k / 256, z = (k % 256) / 16, x = k % 16;
                    int v = in.readUnsignedShort();
                    BlockState st = chunk.getBlockState(pos.set(x, MIN_Y + by, z));
                    int raw = Registries.BLOCK.getRawId(st.getBlock());
                    total++;
                    int yIdx = by;
                    layerTotal[yIdx]++;
                    if (v != 0) { totalNonAir++; if (v == raw) matchNonAir++; }
                    if (v == raw) { match++; layerMatch[yIdx]++; }
                    else if (shown < 15) {
                        shown++;
                        String vb = v == 0 ? "air" : Registries.BLOCK.getId(Registries.BLOCK.get(v)).toString();
                        String wb = raw == 0 ? "air" : Registries.BLOCK.getId(Registries.BLOCK.get(raw)).toString();
                        System.out.println("[ReadWorldProbe] mismatch chunk(" + wx + "," + wz + ") (" + (chunk.getPos().getStartX() + x) + "," + (MIN_Y + by) + "," + (chunk.getPos().getStartZ() + z) + ") vanilla=" + vb + " world=" + wb);
                    }
                }
                for (int b = 0; b < 256; b++) in.readUTF();
            }
        } catch (Exception e) {
            System.out.println("[ReadWorldProbe] failed: " + e);
        }
        System.out.printf("[ReadWorldProbe] world-vs-vanilla: match=%d/%d (%.4f%%) nonAir=%d/%d (%.4f%%)%n",
                match, total, 100.0 * match / total, matchNonAir, totalNonAir,
                totalNonAir == 0 ? 0 : 100.0 * matchNonAir / totalNonAir);
        System.out.print("[ReadWorldProbe] layerMatch%: ");
        for (int yIdx = 0; yIdx < HEIGHT; yIdx += 32) {
            long mt = 0, tt = 0;
            for (int yy = yIdx; yy < Math.min(yIdx + 32, HEIGHT); yy++) { mt += layerMatch[yy]; tt += layerTotal[yy]; }
            System.out.printf("y=%d..%d:%.0f%% ", MIN_Y + yIdx, MIN_Y + Math.min(yIdx + 31, HEIGHT - 1), tt == 0 ? 0 : 100.0 * mt / tt);
        }
        System.out.println();
        server.stop(false);
    }
}
