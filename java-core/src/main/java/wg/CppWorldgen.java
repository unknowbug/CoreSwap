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
        // Forge+Connector 兼容：多级定位 jar（codeSource → ModOrigin → classloader）后 JarFile 提取
        return wg.bench.CoreSwapFixHelper.extractNativeDll();
    }

    /** 当前加载的 dll 路径（版本排查用）。 */
    public static String getNativeLibraryPath() {
        return wg.bench.CoreSwapFixHelper.extractNativeDll();
    }

    /** 创建 worldgen 句柄（seed + worldgen JSON 数据目录） */
    public static native long init(long seed, String worldgenDir);

    /** 多世界：按维度创建句柄（settingsName 如 "nether.json"；biomeParamsFile 如 "biome_params_nether.json"；worldHeight 维度世界高度） */
    public static native long initDim(long seed, String worldgenDir, String settingsName, String biomeParamsFile, int worldHeight);

    /** 释放句柄 */
    public static native void destroy(long handle);

    /**
     * mod 方块注册（260907-07，C1 闭环前置）：把名字注册进 Rust BlockRegistry，返回 Rust 动态 id。
     * ⚠️ 返回值是 Rust 侧动态 id（blocks.json max+1 起），≠ Java raw id——写回错位风险，方案 C 冒烟验证点。
     */
    public static native int registerBlock(long handle, String name);

    /**
     * mod 方块显式 id 注册（260907-08，候选 B——id 错位写回修复）：把名字以 javaRawId 注册进 Rust
     * BlockRegistry，Rust 内部 id 与 Java raw id 同域，写回 Registries.BLOCK.get(id) 直查即对齐。
     * 返回：生效 id（对齐成功 = javaRawId）；handle/name 无效、id 冲突或越界返回 -1。
     */
    public static native int registerBlockId(long handle, String name, int javaRawId);

    /** 密度场批量求值（size×size chunks） */
    public static native int fillDensity(long handle, int minChunkX, int minChunkZ, int size, double[] out);

    /** 密度网格参数 {xzInterval, yInterval, minY, height} */
    public static native int densityParams(long handle, int[] out4);

    /**
     * 完整区块生成（方块层）：count 个 chunk，outs[i] = int[16*16*384]（vanilla raw block id，
     * 索引 (y-MIN_Y)*256 + z*16 + x）。threads <= 0 自适应。返回 count。
     */
    public static native int fillBlocks(long handle, int[] chunkXs, int[] chunkZs, int[][] outs, int threads);

    /**
     * 设置指定 chunk 的 Beardifier（StructureWeightSampler）输入（wg_set_beardifier 的 JNI 包装）。
     * pieces 每 8 int：{minX,minY,minZ,maxX,maxY,maxZ,terrain(0-3),groundLevelDelta}；
     * junctions 每 3 int：{sourceX,sourceGroundY,sourceZ}。在 fillBlocks 之前调用。
     */
    public static native void setBeardifier(long handle, int chunkX, int chunkZ,
                                            int[] pieces, int pieceCount,
                                            int[] junctions, int junctionCount);

    /**
     * 句柄级阶段开关（双跑修复 2026-09-08）：bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE。
     * flags=0 时行为与旧版一致（env 门兜底）。存档链路在 init 后设置，standalone 工具不受影响。
     */
    public static native void setFlags(long handle, int mask);

    /** 读回句柄 flags（诊断用）。 */
    public static native int getFlags(long handle);

    // ==== 光照接管 D4 协议（260905-04 P2，未编译验证；与 Rust 侧共享 ABI 契约） ====

    /** 加载光照数据表（light_data.json），返回句柄，0 = 失败。 */
    public static native long lightInit(String dataJsonPath);

    /** 释放光照句柄。 */
    public static native void lightDestroy(long handle);

    /**
     * 光照计算：blocks9 = 9 chunk raw id（chunkIdx = dz*3+dx，dx/dz 0..2 → 邻 (cx-1+dx, cz-1+dz)；
     * chunk 内 (y+64)*256 + z*16 + x，总长 884736）。outBlock/outSky = 24 section × 2048 B = 49152；
     * outFlags = 48 B，section s 双字节 [blockFlag, skyFlag]：0=数据有效,1=全0,2=全15。
     * Rust 独立重算 3×3 只回中心。返回 0 成功，负数错误。
     */
    public static native int lightCompute(long handle, int[] blocks9, byte[] outBlock, byte[] outSky, byte[] outFlags);

    /**
     * 光照计算（packed ABI，260914-04 候选 C）：sectionMeta = 216 节 × 2 = 432 项
     * （节序 = 9 chunk c=dz*3+dx × 24 section；每节 [bits, psz]；bits=0&psz=0 空节哨兵、
     * bits=0&psz=1 singular 均质节）；paletteData = 逐节拼接 ABI 编码表（rawId | lum<<24）；
     * storage = 逐节拼接 PackedIntegerArray longs（LSB-first 不跨 long，索引序 = y<<8|z<<4|x）。
     * 输出契约同 lightCompute。返回 0 成功；-2 = 长度/解码错（调用方整 chunk 回退 blocks9 ABI）。
     * bits 域 = 4..14；global（ID_LIST bits≥15）不进本 ABI，调用方遇之整 chunk 回退。
     */
    public static native int lightComputePacked(long handle, int[] sectionMeta, int[] paletteData,
            long[] storage, int paletteLen, int storageLen, byte[] outBlock, byte[] outSky, byte[] outFlags);
}


