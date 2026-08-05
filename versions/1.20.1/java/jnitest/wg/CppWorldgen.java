package wg;

import java.io.DataInputStream;
import java.io.FileInputStream;

/**
 * CoreSwap worldgen JNI 桥测试：
 * C++ 生成密度场（大块数据一次交换）→ 与 vanilla 参照文件逐点对比 + 计时。
 * 用法: java -Djava.library.path=<cpp\build\bin> -cp <out> wg.CppWorldgen <seed> <wgDir> <vanilla.density>
 */
public class CppWorldgen {
    static { System.loadLibrary("worldgen"); }

    static native long init(long seed, String worldgenDir);
    static native void destroy(long handle);
    static native int fillDensity(long handle, int minChunkX, int minChunkZ, int size, double[] out);
    static native int densityParams(long handle, int[] out4);

    public static void main(String[] args) throws Exception {
        long seed = Long.parseLong(args[0]);
        String wgDir = args[1];
        String densityPath = args[2];

        long handle = init(seed, wgDir);
        if (handle == 0) { System.err.println("init failed"); System.exit(1); }

        int[] params = new int[4];
        densityParams(handle, params);
        System.out.printf("density grid: xz=%d y=%d minY=%d height=%d%n", params[0], params[1], params[2], params[3]);

        // 读 vanilla 参照（DataInputStream 默认大端，与 Java DataOutputStream 写入一致）
        int size, xzI, yI;
        double[][] vanilla; // [chunkIdx][points]
        int[] chunkX, chunkZ;
        try (DataInputStream in = new DataInputStream(new FileInputStream(densityPath))) {
            int magic = in.readInt();
            long vseed = in.readLong();
            size = in.readInt();
            xzI = in.readInt();
            yI = in.readInt();
            System.out.printf("vanilla file: magic=0x%08X seed=%d size=%d xzI=%d yI=%d%n", magic, vseed, size, xzI, yI);
            vanilla = new double[size * size][];
            chunkX = new int[size * size];
            chunkZ = new int[size * size];
            for (int c = 0; c < size * size; c++) {
                int cx = in.readInt(), cz = in.readInt();
                int sx = in.readInt(), sy = in.readInt(), sz = in.readInt();
                int minY = in.readInt(), height = in.readInt();
                chunkX[c] = cx; chunkZ[c] = cz;
                double[] pts = new double[sx * sy * sz];
                for (int i = 0; i < pts.length; i++) pts[i] = in.readDouble();
                vanilla[c] = pts;
            }
        }

        // C++ 生成同样 region（4x4 chunks）并计时（含 JNI 开销）
        double[] out = new double[size * size * 768];
        long t0 = System.nanoTime();
        int pointsPerChunk = fillDensity(handle, chunkX[0], chunkZ[0], size, out);
        long t1 = System.nanoTime();
        double ms = (t1 - t0) / 1e6;
        System.out.printf("C++ fillDensity (JNI): %d chunks in %.2f ms (%.2f ms/chunk)%n", size * size, ms, ms / (size * size));

        // 逐点对比
        long match = 0, total = 0;
        double maxErr = 0;
        for (int c = 0; c < size * size; c++) {
            for (int i = 0; i < vanilla[c].length; i++) {
                double v = vanilla[c][i];
                double got = out[c * 768 + i];
                double err = Math.abs(got - v);
                total++;
                if (err < 1e-9) match++;
                if (err > maxErr) maxErr = err;
            }
        }
        System.out.printf("match=%d/%d (%.4f%%) maxErr=%.9g%n", match, total, 100.0 * match / total, maxErr);

        destroy(handle);
        System.out.println(match == total ? "JNI BRIDGE OK (100%)" : "JNI BRIDGE DIFF");
    }
}
