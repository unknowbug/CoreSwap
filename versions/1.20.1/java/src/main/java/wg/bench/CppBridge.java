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
    // 方块注册表缓存（RQ-005）：进程级静态，vanilla 注册表运行期冻结。
    // null = 未查过 registry（不能填 AIR：st==null 判断会永远 false 导致全写空气——历史根因）；
    // AtomicReferenceArray 保证并发可见性（多 worker 线程同时写同 id 同值，幂等安全）。
    private static final int MAX_ID = 4096;
    private static final java.util.concurrent.atomic.AtomicReferenceArray<BlockState> STATE_BY_ID =
            new java.util.concurrent.atomic.AtomicReferenceArray<>(MAX_ID);
    private static final BlockState AIR = Blocks.AIR.getDefaultState();
    // per-thread 输出 buffer（RQ-004）：M=1 无锁模型，每 worker 线程一个 16*16*384 int（~384KB）
    private static final ThreadLocal<int[]> BUF =
            ThreadLocal.withInitial(() -> new int[16 * 16 * 384]);
    /** 生成线程数（-Dcoreswap.threads=N 显式覆盖；否则模式自适应：
     *  服务端全核(-1)、客户端留 2 核(-2) 给渲染/主线程——Issue #7 + 用户设计） */
    private static final int THREADS = resolveThreads();

    private static int resolveThreads() {
        String explicit = System.getProperty("coreswap.threads");
        if (explicit != null) {
            try {
                return Integer.parseInt(explicit);
            } catch (NumberFormatException e) {
                return -1;  // 非法值兜底：服务端全核
            }
        }
        try {
            // 反射拿 Fabric 环境（编译期无 fabric-loader API 依赖；Forge+Connector 也可能没有）
            Object loader = Class.forName("net.fabricmc.loader.api.FabricLoader")
                    .getMethod("getInstance").invoke(null);
            Object env = loader.getClass().getMethod("getEnvironmentType").invoke(loader);
            return "SERVER".equals(env.toString()) ? -1 : -2;
        } catch (Throwable t) {
            return -1;  // 非 Fabric/未知：服务端全核兜底
        }
    }

    private CppBridge() {}

    public static void init(long seed) {
        String dir = System.getProperty("cpp.worldgen.dir");
        if (dir == null) dir = extractWorldgenDir();
        handle = CppWorldgen.init(seed, dir);
        enabled = handle != 0;
        System.out.println("[CppBridge] init seed=" + seed + " worldgenDir=" + dir + " enabled=" + enabled);
        // 打印 dll 版本信息（排查旧缓存：用户加载的 dll 应与 jar 内的一致）
        try {
            java.nio.file.Path dllPath = java.nio.file.Path.of(CppWorldgen.getNativeLibraryPath());
            byte[] loaded = java.nio.file.Files.readAllBytes(dllPath);
            String sha = java.security.MessageDigest.getInstance("SHA-256").digest(loaded).length > 0
                    ? java.util.HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(loaded)) : "";
            System.out.println("[CppBridge] dll=" + dllPath + " size=" + loaded.length + " sha256=" + sha.substring(0, 16) + "...");
        } catch (Exception e) {
            System.out.println("[CppBridge] dll info failed: " + e);
        }
    }

    /**
     * 从 mod 内 worldgen-data/ 解压 C++ 所需 JSON 数据到临时目录（幂等：已存在即复用）。
     * 目标布局（对齐 C++ wg_create 的路径约定）：
     *   <tmp>/coreswap-data/worldgen/data/minecraft/worldgen/...  （JSON 数据）
     *   <tmp>/coreswap-data/blocks.json / biome_params.json      （wgDir/../ 查找）
     */
    private static String extractWorldgenDir() {
        // Forge+Connector 兼容：多级定位 jar（codeSource → ModOrigin → classloader）后 JarFile 提取
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
     * 喂 Beardifier（StructureWeightSampler）输入到 C++：在 populateNoise 拦截处、fillChunk 之前调用。
     * 用 vanilla createStructureWeightSampler 构造（结构与 Java 同源），反射提取 piece/junction 列表，
     * 序列化 int[] 传给 wg_set_beardifier。失败时降级：不喂数据（Beardifier=0，与现状一致），不阻断生成。
     */
    public static void feedBeardifier(Chunk chunk, net.minecraft.world.gen.StructureAccessor structures) {
        long h = handle;
        if (!enabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        try {
            net.minecraft.world.gen.StructureWeightSampler sws =
                    net.minecraft.world.gen.StructureWeightSampler.createStructureWeightSampler(structures, chunk.getPos());
            java.util.ArrayList<int[]> pieces = new java.util.ArrayList<>();
            java.util.ArrayList<int[]> junctions = new java.util.ArrayList<>();
            java.lang.reflect.Field fPieces = net.minecraft.world.gen.StructureWeightSampler.class.getDeclaredField("pieceIterator");
            java.lang.reflect.Field fJunctions = net.minecraft.world.gen.StructureWeightSampler.class.getDeclaredField("junctionIterator");
            fPieces.setAccessible(true);
            fJunctions.setAccessible(true);
            it.unimi.dsi.fastutil.objects.ObjectListIterator<?> pit = (it.unimi.dsi.fastutil.objects.ObjectListIterator<?>) fPieces.get(sws);
            while (pit.hasNext()) {
                Object piece = pit.next();
                Object box = piece.getClass().getMethod("box").invoke(piece);
                java.lang.reflect.Method gML = box.getClass().getMethod("getMinX");
                java.lang.reflect.Method gMY = box.getClass().getMethod("getMinY");
                java.lang.reflect.Method gMZ = box.getClass().getMethod("getMinZ");
                java.lang.reflect.Method gXL = box.getClass().getMethod("getMaxX");
                java.lang.reflect.Method gXY = box.getClass().getMethod("getMaxY");
                java.lang.reflect.Method gXZ = box.getClass().getMethod("getMaxZ");
                Object terrain = piece.getClass().getMethod("terrainAdjustment").invoke(piece);
                int terrainOrd = ((Enum<?>) terrain).ordinal();
                int delta = (Integer) piece.getClass().getMethod("groundLevelDelta").invoke(piece);
                pieces.add(new int[]{
                        (Integer) gML.invoke(box), (Integer) gMY.invoke(box), (Integer) gMZ.invoke(box),
                        (Integer) gXL.invoke(box), (Integer) gXY.invoke(box), (Integer) gXZ.invoke(box),
                        terrainOrd, delta});
            }
            it.unimi.dsi.fastutil.objects.ObjectListIterator<?> jit = (it.unimi.dsi.fastutil.objects.ObjectListIterator<?>) fJunctions.get(sws);
            while (jit.hasNext()) {
                Object jj = jit.next();
                int sx = (Integer) jj.getClass().getMethod("getSourceX").invoke(jj);
                int sy = (Integer) jj.getClass().getMethod("getSourceGroundY").invoke(jj);
                int sz = (Integer) jj.getClass().getMethod("getSourceZ").invoke(jj);
                junctions.add(new int[]{sx, sy, sz});
            }
            if (pieces.isEmpty() && junctions.isEmpty()) {
                CppWorldgen.setBeardifier(h, cx, cz, null, 0, null, 0);  // 清空该 chunk（防上一批残留）
            } else {
                int[] p = new int[pieces.size() * 8];
                for (int i = 0; i < pieces.size(); i++) System.arraycopy(pieces.get(i), 0, p, i * 8, 8);
                int[] j = new int[junctions.size() * 3];
                for (int i = 0; i < junctions.size(); i++) System.arraycopy(junctions.get(i), 0, j, i * 3, 3);
                CppWorldgen.setBeardifier(h, cx, cz, p, pieces.size(), j, junctions.size());
            }
        } catch (Throwable t) {
            System.out.println("[CppBridge] feedBeardifier failed chunk(" + cx + "," + cz + "): " + t);
            // 降级：不喂数据（Beardifier=0）
        }
    }

    /**
     * 用 C++ 结果整块填充 Chunk（NOISE 阶段的方块 + 高度图）。
     * 并发模型（RQ-001~004 改造，2026-08-11）：M=1 非空即处理，无全局锁——
     * 每个 mixin worker 线程直接调 JNI fillBlocks（JNI 本身多线程安全；C++ 池已改任务队列
     * 模型，批间也真并行）；writeChunk 写独立 Chunk 对象天然并行。per-thread buffer 消除共享池。
     */
    public static void fillChunk(Chunk chunk) {
        long h = handle;  // 本地快照：destroy 后置 0，拦截后续调用（不 use-after-free）
        if (!enabled || h == 0) return;
        int cx = chunk.getPos().x, cz = chunk.getPos().z;
        int[] buf = BUF.get();  // per-thread buffer（ThreadLocal，~384KB/worker，RQ-004）
        int got = 0;
        try {
            got = CppWorldgen.fillBlocks(h, new int[]{cx}, new int[]{cz},
                    new int[][]{buf}, THREADS);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG fillBlocks threw chunk(" + cx + "," + cz + "): " + t);
            return;
        }
        if (got != 1) {
            System.out.println("[CppBridge] DIAG fillBlocks got=" + got + " chunk(" + cx + "," + cz + ")");
            return;
        }
        // 诊断：C++ 输出是否全 air（区分「C++ 输出 0」与「写入丢失」）
        int nz = 0;
        for (int k = 0; k < buf.length; k++) if (buf[k] != 0) nz++;
        if (nz == 0) System.out.println("[CppBridge] DIAG buf-all-air chunk(" + cx + "," + cz + ")");
        else if (nz < 1000)
            System.out.println("[CppBridge] DIAG buf-sparse chunk(" + cx + "," + cz + ") nz=" + nz);
        try {
            writeChunk(chunk, cx, cz, buf);
        } catch (Throwable t) {
            System.out.println("[CppBridge] DIAG write threw chunk(" + cx + "," + cz + "): " + t);
        }
    }

    // 直写 PalettedContainer（跳过 chunk.setBlockState 的 heightmap/blockEntity 开销）
    // Chunk.getSection(int) 参数是 0-based section index（0..23 = 世界 y -64..319）
    // 抽出独立方法便于 try-catch：写入异常 = chunk 保持空气 → 后续结构悬浮半空
    private static void writeChunk(Chunk chunk, int cx, int cz, int[] buf) {
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
                    if (id < 0 || id >= MAX_ID)
                        throw new IllegalArgumentException("bad id " + id + " chunk(" + cx + "," + cz + ")");
                    BlockState st = STATE_BY_ID.get(id);
                    if (st == null) {
                        st = id == 0 ? AIR : Registries.BLOCK.get(id).getDefaultState();
                        STATE_BY_ID.set(id, st);  // 幂等 set（并发同 id 同值，无锁安全）
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
