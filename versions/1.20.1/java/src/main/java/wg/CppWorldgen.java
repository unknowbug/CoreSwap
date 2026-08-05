package wg;

import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * CoreSwap worldgen JNI 桥（与 jni_bridge.cpp 的 Java_wg_CppWorldgen_* 对应）。
 * 加载 C++ 编译的 worldgen.dll：
 * 1. -Dcpp.worldgen.lib 显式指定（开发调试用绝对路径）
 * 2. mod jar 内 native/worldgen.dll（解压到临时目录后 System.load）
 */
public final class CppWorldgen {
    static {
        String lib = System.getProperty("cpp.worldgen.lib");
        if (lib != null) {
            System.load(lib);
        } else {
            System.load(extractNativeDll());
        }
    }

    private CppWorldgen() {}

    private static String extractNativeDll() {
        String tmpDir = System.getProperty("java.io.tmpdir");
        Path dir = Path.of(tmpDir, "coreswap-native");
        Path dll = dir.resolve("worldgen.dll");
        try {
            if (!Files.exists(dll)) {
                Files.createDirectories(dir);
                var container = FabricLoader.getInstance().getModContainer("worldgen-bench").get();
                for (Path root : container.getRootPaths()) {
                    Path src = root.resolve("native/worldgen.dll");
                    if (Files.isRegularFile(src)) {
                        Files.copy(src, dll, StandardCopyOption.REPLACE_EXISTING);
                        break;
                    }
                }
            }
            if (!Files.exists(dll)) {
                throw new IllegalStateException("worldgen.dll not found in mod native/");
            }
            return dll.toString();
        } catch (IOException e) {
            throw new RuntimeException("failed to extract worldgen.dll", e);
        }
    }

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
