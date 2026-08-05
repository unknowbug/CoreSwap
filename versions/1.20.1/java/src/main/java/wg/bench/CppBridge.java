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
    private static final ThreadLocal<int[]> BUF = ThreadLocal.withInitial(() -> new int[16 * 16 * 384]);

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
        String tmpDir = System.getProperty("java.io.tmpdir");
        Path target = Path.of(tmpDir, "coreswap-data");
        Path wgDir = target.resolve("worldgen");
        try {
            Path marker = wgDir.resolve("data/minecraft/worldgen/noise_settings/overworld.json");
            if (!Files.exists(marker)) {
                // 幂等失败时残留旧结构 → 先清再解压
                if (Files.exists(target)) deleteRecursively(target);
                Files.createDirectories(wgDir);
                var container = net.fabricmc.loader.api.FabricLoader.getInstance().getModContainer("worldgen-bench").get();
                for (Path root : container.getRootPaths()) {
                    Path src = root.resolve("worldgen-data");
                    if (!Files.isDirectory(src)) continue;
                    try (var stream = Files.walk(src)) {
                        stream.filter(p -> Files.isRegularFile(p)).forEach(p -> {
                            Path rel = src.relativize(p);  // data/... 或 blocks.json / biome_params.json
                            Path dst = rel.startsWith("data") ? wgDir.resolve(rel.toString()) : target.resolve(rel.toString());
                            try {
                                Files.createDirectories(dst.getParent());
                                Files.copy(p, dst, java.nio.file.StandardCopyOption.REPLACE_EXISTING);
                            } catch (IOException e) {
                                throw new RuntimeException(e);
                            }
                        });
                    }
                }
                if (!Files.exists(marker)) {
                    throw new IllegalStateException("worldgen-data not found in mod resources");
                }
            }
            return wgDir.toString();
        } catch (RuntimeException e) {
            throw e;
        } catch (Exception e) {
            throw new RuntimeException("failed to extract worldgen-data", e);
        }
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
        long t0 = System.nanoTime();
        int[] buf = BUF.get();  // 复用（98304 ints/393KB，每 chunk 分配是 GC 压力）
        // threads=0：C++ 侧自适应（min(CPU 核心数, 任务数)）——不要写死线程数
        int n;
        try {
            n = CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz}, new int[][]{buf}, 0);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG fillBlocks threw for chunk(" + cx + "," + cz + "): " + t);
            return;
        }
        long t1 = System.nanoTime();
        if (n != 1) {
            System.out.println("[CppBridge] fillBlocks failed for chunk(" + cx + "," + cz + ")");
            return;
        }
        // 诊断：C++ 输出是否全 air（区分「C++ 输出 0」与「写入丢失」）
        int nz = 0;
        for (int i = 0; i < buf.length; i++) if (buf[i] != 0) nz++;
        if (nz == 0) System.out.println("[CppBridge] DIAG buf-all-air chunk(" + cx + "," + cz + ")");
        else if (nz < 1000) System.out.println("[CppBridge] DIAG buf-sparse chunk(" + cx + "," + cz + ") nz=" + nz);
        try {
            writeChunk(chunk, cx, cz, buf);
        } catch (Throwable t) {
            // 写入异常 = chunk 保持空气 → 后续结构（井/冰山）悬浮半空。必须暴露出来。
            System.out.println("[CppBridge] DIAG write threw chunk(" + cx + "," + cz + "): " + t);
            for (StackTraceElement e : t.getStackTrace()) {
                System.out.println("    at " + e);
                if (e.getMethodName().contains("writeChunk") || e.getMethodName().contains("fillChunk")) break;
            }
        }
        long t2 = System.nanoTime();
        if (DEBUG) System.out.printf("[CppBridge] chunk(%d,%d): C++=%dms write=%dms%n",
                cx, cz, (t1 - t0) / 1_000_000, (t2 - t1) / 1_000_000);
    }

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
