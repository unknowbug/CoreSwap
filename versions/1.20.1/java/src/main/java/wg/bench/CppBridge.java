package wg.bench;

import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.Registries;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.noise.NoiseConfig;
import wg.CppWorldgen;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * CoreSwap worldgen 全局桥：持有 C++ 句柄，把 C++ 生成的整块写入 Chunk。
 * 启用：-Dcpp.replace=1（由 BenchMod 在 server started 时 init）。
 */
public final class CppBridge {
    private static volatile long handle;
    public static volatile boolean enabled;
    private static final boolean DEBUG = System.getProperty("cpp.debug") != null;

    private CppBridge() {}

    public static void init(long seed) {
        String dir = System.getProperty("cpp.worldgen.dir");
        if (dir == null) dir = extractWorldgenDir();
        handle = CppWorldgen.init(seed, dir);
        enabled = handle != 0;
        System.out.println("[CppBridge] init seed=" + seed + " worldgenDir=" + dir + " enabled=" + enabled);
    }

    /**
     * 从 mod 内 worldgen-data/ 解压 C++ 所需 JSON 数据到临时目录（幂等：已存在即复用）。
     * 目标布局（对齐 C++ wg_create 的路径约定）：
     *   <tmp>/coreswap-data/worldgen/data/minecraft/worldgen/...  （JSON 数据）
     *   <tmp>/coreswap-data/blocks.json / biome_params.json      （wgDir/../ 查找）
     */
    private static String extractWorldgenDir() {
        // Forge+Connector 兼容：原 getRootPaths() 在 Forge UnionFileSystem 下不可遍历，
        // 改用 CoreSwapFixHelper 多级定位 jar（codeSource → ModOrigin.getPaths → classloader）后 JarFile 提取。
        return CoreSwapFixHelper.extractWorldgenDir();
    }

    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) return;
        try (var stream = Files.walk(path)) {
            stream.sorted(java.util.Comparator.reverseOrder()).forEach(p -> {
                try { Files.deleteIfExists(p); } catch (IOException ignored) {}
            });
        }
    }

    /**
     * 用 C++ 结果整块填充 Chunk（NOISE 阶段的方块 + 高度图）。
     * 性能：直接写 PalettedContainer（跳过 setBlockState 的 heightmap/blockEntity 开销），
     * 高度图用 populateHeightmaps 一次批量重算——98304 次 setBlockState → 直写。
     */
    public static void fillChunk(Chunk chunk) {
        long h = handle;  // 本地快照：destroy 后置 0，拦截后续调用（不 use-after-free）
        if (!enabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        boolean drain;
        synchronized (BATCH_LOCK) {
            PENDING.addLast(new Object[]{chunk, cx, cz});
            if (PENDING.size() >= BATCH) {
                drain = true;  // 攒满一批 → 本线程处理
            } else {
                // 没攒满：短暂等待其他线程攒批（wait 释放锁，其他 Worker 可继续入队）
                try {
                    BATCH_LOCK.wait(BATCH_TIMEOUT_MS);
                } catch (InterruptedException ignored) {
                }
                // 超时后：若队列仍非空（可能被其他线程处理空）则本线程处理
                drain = !PENDING.isEmpty();
            }
        }
        if (drain) drainBatch(h);
    }

    private static void drainBatch(long h) {
        // 锁内处理：fillBlocks + writeChunk 都用 BATCH_BUFS（复用池），必须互斥
        synchronized (BATCH_LOCK) {
            int n = PENDING.size();
            if (n == 0) return;
            Object[][] batch = PENDING.toArray(new Object[0][]);
            PENDING.clear();
            int[] cxs = new int[n];
            int[] czs = new int[n];
            for (int i = 0; i < n; i++) {
                cxs[i] = (Integer) batch[i][1];
                czs[i] = (Integer) batch[i][2];
            }
            long t0 = System.nanoTime();
            int got;
            try {
                // 批量 fillBlocks：threads=0 → C++ 自适应 min(核数, n)；批量摊薄 JNI 边界 + 并行生成
                // 注意：JNI 校验 outs.length == count，BATCH_BUFS 固定 16 → 必须 copyOf 到 n（引用数组，开销可忽略）
                got = CppWorldgen.fillBlocks(h, cxs, czs, java.util.Arrays.copyOf(BATCH_BUFS, n), 0);
            } catch (Throwable t) {
                System.out.println("[CppBridge] DIAG batch fillBlocks threw n=" + n + ": " + t);
                return;
            }
            long t1 = System.nanoTime();
            if (got != n) {
                System.out.println("[CppBridge] DIAG batch fillBlocks got=" + got + " want=" + n);
                got = Math.min(got, n);
            }
            for (int i = 0; i < got; i++) {
                int[] buf = BATCH_BUFS[i];
                Chunk c = (Chunk) batch[i][0];
                // 诊断：C++ 输出是否全 air（区分「C++ 输出 0」与「写入丢失」）
                int nz = 0;
                for (int k = 0; k < buf.length; k++) if (buf[k] != 0) nz++;
                if (nz == 0) System.out.println("[CppBridge] DIAG buf-all-air chunk(" + cxs[i] + "," + czs[i] + ")");
                else if (nz < 1000)
                    System.out.println("[CppBridge] DIAG buf-sparse chunk(" + cxs[i] + "," + czs[i] + ") nz=" + nz);
                try {
                    writeChunk(c, cxs[i], czs[i], buf);
                } catch (Throwable t) {
                    System.out.println("[CppBridge] DIAG write threw chunk(" + cxs[i] + "," + czs[i] + "): " + t);
                }
            }
            long t2 = System.nanoTime();
            if (DEBUG) System.out.printf("[CppBridge] batch n=%d: C++=%dms write=%dms%n",
                    n, (t1 - t0) / 1_000_000, (t2 - t1) / 1_000_000);
        }
    }

    // 批量攒批参数：攒满 BATCH 或超时 BATCH_TIMEOUT_MS 即处理（锁内串行 drain，buf 池安全复用）
    private static final int BATCH = 16;
    private static final long BATCH_TIMEOUT_MS = 2;
    private static final Object BATCH_LOCK = new Object();
    private static final java.util.ArrayDeque<Object[]> PENDING = new java.util.ArrayDeque<>();
    private static final int[][] BATCH_BUFS = new int[BATCH][16 * 16 * 384];

    // 直写 PalettedContainer（跳过 chunk.setBlockState 的 heightmap/blockEntity 开销）
    // Chunk.getSection(int) 参数是 0-based section index（0..23 = 世界 y -64..319）
    // 抽出独立方法便于 try-catch：写入异常 = chunk 保持空气 → 后续结构悬浮半空
    private static void writeChunk(Chunk chunk, int cx, int cz, int[] buf) {
        BlockState[] stateById = new BlockState[4096];  // null = 未查过 registry（不能填 AIR：st==null 判断会永远 false 导致全写空气——历史根因）
        BlockState air = Blocks.AIR.getDefaultState();
        net.minecraft.world.chunk.ChunkSection[] sections = new net.minecraft.world.chunk.ChunkSection[24];
        // 1.20.1 Chunk.getSection(int) 是 0-based 索引（0..23 = y -64..319）——已验证（getSection(-4) 越界）
        for (int secIdx = 0; secIdx < 24; secIdx++) sections[secIdx] = chunk.getSection(secIdx);
        for (int by = 0; by < 384; by++) {
            net.minecraft.world.chunk.ChunkSection sec = sections[by >> 4];
            int sy = by & 15;
            for (int z = 0; z < 16; z++) {
                int base = by * 256 + z * 16;
                for (int x = 0; x < 16; x++) {
                    int id = buf[base + x];
                    if (id < 0 || id >= 4096)
                        throw new IllegalArgumentException("bad id " + id + " chunk(" + cx + "," + cz + ")");
                    BlockState st = stateById[id];
                    if (st == null) {
                        st = id == 0 ? air : Registries.BLOCK.get(id).getDefaultState();
                        stateById[id] = st;
                    }
                    // 必须用 ChunkSection.setBlockState（内部=container.set + nonEmptyBlockCount 更新）：
                    // 直写 container.set 不更新计数 → isEmpty() 误判 true → 全部读成空气（历史根因）
                    sec.setBlockState(x, sy, z, st);
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
    }

    public static void destroy() {
        // 只标记禁用并摘除句柄；真实释放由 shutdown hook 完成
        // （防止「保存并退出」时异步 chunk 生成还在 fillBlocks 里用已释放句柄 → use-after-free）
        enabled = false;
        handle = 0;
    }

    // 分量对照探针：用 vanilla NoiseConfig 的 density function registry 采样指定坐标的分量
    private static volatile boolean compProbed = false;
    public static boolean didCompProbe() { return compProbed; }

    public static void compProbe(NoiseConfig noiseConfig) {
        compProbed = true;
        try {
            int bx = Integer.parseInt(System.getProperty("comp.x"));
            int bz = Integer.parseInt(System.getProperty("comp.z"));
            int by = Integer.parseInt(System.getProperty("comp.y", "31"));
            var router = noiseConfig.getNoiseRouter();
            var names = new String[]{"finalDensity", "depth",
                    "continents", "erosion", "ridges", "initialDensityWithoutJaggedness",
                    "fluidLevelFloodedness", "fluidLevelSpread", "barrier", "lava"};
            var noisePos = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(bx, by, bz);
            for (String n : names) {
                java.lang.reflect.Method m = router.getClass().getMethod(n);
                net.minecraft.world.gen.densityfunction.DensityFunction df =
                        (net.minecraft.world.gen.densityfunction.DensityFunction) m.invoke(router);
                if (df != null) {
                    System.out.println("[COMP] " + n + "(" + bx + "," + by + "," + bz + ")=" + df.sample(noisePos));
                } else {
                    System.out.println("[COMP] " + n + "=<null>");
                }
            }
            // 提取 finalDensity 树里的 InterpolatedNoiseSampler（base_3d_noise 唯一节点）
            final net.minecraft.world.gen.densityfunction.DensityFunction finalDensity = router.finalDensity();
            final Object[] found = new Object[1];
            finalDensity.apply(new net.minecraft.world.gen.densityfunction.DensityFunction.DensityFunctionVisitor() {
                public net.minecraft.world.gen.densityfunction.DensityFunction apply(
                        net.minecraft.world.gen.densityfunction.DensityFunction df) {
                    if (found[0] == null &&
                            df instanceof net.minecraft.util.math.noise.InterpolatedNoiseSampler) {
                        found[0] = df;
                    }
                    return df;
                }
                public net.minecraft.world.gen.densityfunction.DensityFunction.Noise apply(
                        net.minecraft.world.gen.densityfunction.DensityFunction.Noise noise) { return noise; }
            });
            if (found[0] != null) {
                net.minecraft.world.gen.densityfunction.DensityFunction df =
                        (net.minecraft.world.gen.densityfunction.DensityFunction) found[0];
                System.out.println("[COMP] base_3d_noise(" + bx + "," + by + "," + bz + ")=" + df.sample(noisePos));
            } else {
                System.out.println("[COMP] base_3d_noise=<not found>");
            }
        } catch (Throwable t) {
            System.out.println("[COMP] probe error: " + t);
        }
    }

    static {
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            long h = handle;
            handle = 0;
            if (h != 0) CppWorldgen.destroy(h);
        }, "coreswap-destroy"));
    }
}
