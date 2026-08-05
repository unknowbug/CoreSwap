package wg.bench;

import wg.CppWorldgen;

import java.io.BufferedInputStream;
import java.io.DataInputStream;
import java.io.FileInputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * JNI 验证：Java 经 JNI 调 C++ 生成 4×4 chunks（3200..3263），
 * 与 data/vanilla_<seed>_<size>_<ox>_<oz>.blocks 参照逐块对比，期望 100%。
 * 用法：gradle runServer -PjniProbe=true
 */
public class JniProbe {
    private static final int MIN_Y = -64, HEIGHT = 384;

    public static void run(net.minecraft.server.MinecraftServer server) {
        long seed = Long.parseLong(System.getProperty("bench.seed", "-8248318472910187742"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        int originX = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int originZ = Integer.parseInt(System.getProperty("bench.originZ", "3208"));
        String worldgenDir = System.getProperty("bench.worldgen", "E:/python/MC/data/worldgen");
        String dataDir = System.getProperty("bench.out", "E:/python/MC/data");

        Path vanilla = Path.of(dataDir, "vanilla_" + seed + "_" + size + "_" + originX + "_" + originZ + ".blocks");
        if (!Files.exists(vanilla)) {
            System.out.println("[JniProbe] missing vanilla reference: " + vanilla);
            return;
        }

        long h = CppWorldgen.init(seed, worldgenDir);
        if (h == 0) {
            System.out.println("[JniProbe] init failed");
            return;
        }
        int n = size * size;
        int[] cxs = new int[n], czs = new int[n];
        for (int i = 0; i < n; i++) {
            cxs[i] = originX / 16 + i % size;
            czs[i] = originZ / 16 + i / size;
        }
        int[][] outs = new int[n][16 * 16 * HEIGHT];
        int threads = Integer.parseInt(System.getProperty("bench.threads", "1")); // JNI 验证先用单线程排除多线程因素
        long t0 = System.nanoTime();
        int count = CppWorldgen.fillBlocks(h, cxs, czs, outs, threads);
        long t1 = System.nanoTime();
        System.out.printf("[JniProbe] C++ via JNI: %d chunks, %.1fms (%.2fms/chunk)%n",
                count, (t1 - t0) / 1e6, (t1 - t0) / 1e6 / n);

        // 读 vanilla 参照（头 32 字节 + 每 chunk 8 字节坐标 + 16*16*384 short）
        long total = 0, match = 0, totalNonAir = 0, matchNonAir = 0;
        long[] layerTotal = new long[HEIGHT], layerMatch = new long[HEIGHT];
        List<String> mismatches = new ArrayList<>();
        // 调试：把 JNI 输出写 raw 文件（对比 got.bin 用）
        try (java.io.DataOutputStream jout = new java.io.DataOutputStream(
                new java.io.BufferedOutputStream(new java.io.FileOutputStream(
                        Path.of(dataDir, "jni_out.bin").toFile())))) {
            for (int i = 0; i < n; i++) {
                jout.writeInt(cxs[i]); jout.writeInt(czs[i]);
                for (int k = 0; k < 16 * 16 * HEIGHT; k++) jout.writeInt(outs[i][k]);
            }
        } catch (Exception e2) {
            System.out.println("[JniProbe] write jni_out failed: " + e2);
        }
        try (DataInputStream in = new DataInputStream(new BufferedInputStream(new FileInputStream(vanilla.toFile())))) {
            in.readInt(); // WGB2
            in.readLong(); // seed
            in.readInt(); in.readInt(); in.readInt(); // size originX originZ
            in.readInt(); in.readInt(); // MIN_Y HEIGHT
            for (int i = 0; i < n; i++) {
                int wx = in.readInt(); int wz = in.readInt(); // wx wz
                if (i == 0) System.out.println("[JniProbe] ref chunk(" + wx + "," + wz + ") want (" + cxs[0] + "," + czs[0] + ")");
                for (int k = 0; k < 16 * 16 * HEIGHT; k++) {
                    int v = in.readUnsignedShort();
                    int c = outs[i][k];
                    total++;
                    int yIdx = k / 256;
                    layerTotal[yIdx]++;
                    if (v != 0) { totalNonAir++; if (v == c) matchNonAir++; }
                    if (v == c) { match++; layerMatch[yIdx]++; }
                    else if (mismatches.size() < 10) mismatches.add(cxs[i] * 16 + k % 16 + ", " + (MIN_Y + yIdx) + ", " + (czs[i] * 16 + (k % 256) / 16));
                }
                // 跳过每 chunk 末尾的 biome 数据（BlockProbe 写了 256 个 writeUTF）
                for (int b = 0; b < 256; b++) in.readUTF();
            }
        } catch (Exception e) {
            System.out.println("[JniProbe] read failed: " + e);
            CppWorldgen.destroy(h);
            return;
        }
        System.out.printf("[JniProbe] TOTAL: match=%d/%d (%.4f%%) nonAir match=%d/%d (%.4f%%)%n",
                match, total, 100.0 * match / total, matchNonAir, totalNonAir,
                totalNonAir == 0 ? 0 : 100.0 * matchNonAir / totalNonAir);
        System.out.print("[JniProbe] layerMatch%: ");
        for (int yIdx = 0; yIdx < HEIGHT; yIdx += 32) {
            long mt = 0, tt = 0;
            for (int yy = yIdx; yy < Math.min(yIdx + 32, HEIGHT); yy++) { mt += layerMatch[yy]; tt += layerTotal[yy]; }
            System.out.printf("y=%d..%d:%.0f%% ", MIN_Y + yIdx, MIN_Y + Math.min(yIdx + 31, HEIGHT - 1), 100.0 * mt / tt);
        }
        System.out.println();
        if (!mismatches.isEmpty()) System.out.println("[JniProbe] first mismatches (x,y,z): " + mismatches);
        CppWorldgen.destroy(h);
    }
}
