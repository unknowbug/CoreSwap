package wg;

/**
 * CoreSwap worldgen JNI 桥（与 jni_bridge.cpp 的 Java_wg_CppWorldgen_* 对应）。
 * 加载 C++ 编译的 worldgen.dll（路径可用 -Dcpp.worldgen.lib 覆盖）。
 */
public final class CppWorldgen {
    static {
        String lib = System.getProperty("cpp.worldgen.lib");
        if (lib == null) lib = "E:/python/MC/versions/1.20.1/cpp/build/bin/worldgen.dll";
        System.load(lib);
    }

    private CppWorldgen() {}

    /** 创建 worldgen 句柄（seed + worldgen JSON 数据目录） */
    public static native long init(long seed, String worldgenDir);

    /** 释放句柄 */
    public static native void destroy(long handle);

    /** 密度场批量求值（size×size chunks） */
    public static native int fillDensity(long handle, int minChunkX, int minChunkZ, int size, double[] out);

    /** 密度网格参数 {xzInterval, yInterval, minY, height} */
    public static native int densityParams(long handle, int[] out4);

    /**
     * 完整区块生成（方块层）：count 个 chunk，outs[i] = int[16*16*384]（vanilla raw block id，
     * 索引 (y-MIN_Y)*256 + z*16 + x）。threads <= 0 自适应。返回 count。
     */
    public static native int fillBlocks(long handle, int[] chunkXs, int[] chunkZs, int[][] outs, int threads);
}
