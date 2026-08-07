package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.world.gen.densityfunction.DensityFunction;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * DensityProbe：导指定列（chunk 内 x,z）的 vanilla finalDensity 剖面。
 * 用法：-DdensityProbe=true -DdensityProbe.dimension=nether -DdensityProbe.chunkX=0
 *       -DdensityProbe.chunkZ=0 -DdensityProbe.x=8 -DdensityProbe.z=8
 * 输出 data/vanilla_density_<dim>_c<cx>_<cz>_b<bx>_<bz>.txt（每行 "y 值"，y 从 minY 每 4）。
 * 用于与 C++ 侧 finalDensity 逐点对比（下界 lava 差异定位）。
 */
public class DensityProbe {
    public static void run(MinecraftServer server) {
        try {
            String dim = System.getProperty("densityProbe.dimension", "nether");
            int cx = Integer.parseInt(System.getProperty("densityProbe.chunkX", "0"));
            int cz = Integer.parseInt(System.getProperty("densityProbe.chunkZ", "0"));
            int bx = Integer.parseInt(System.getProperty("densityProbe.x", "8"));
            int bz = Integer.parseInt(System.getProperty("densityProbe.z", "8"));
            ServerWorld world = dim.equals("nether")
                    ? server.getWorld(net.minecraft.world.World.NETHER)
                    : server.getOverworld();
            if (world == null) { System.out.println("[DensityProbe] world " + dim + " not found"); server.stop(false); return; }
            // NOISE 阶段：ChunkNoiseSampler 存活（finalDensity 可用）
            Chunk chunk = world.getChunk(cx, cz, ChunkStatus.NOISE, true);
            Field f = Chunk.class.getDeclaredField("chunkNoiseSampler");
            f.setAccessible(true);
            Object cns = f.get(chunk);
            if (cns == null) { System.out.println("[DensityProbe] cns null at NOISE stage — skipping cns chain (density/comps output still valid)"); }
            // RouterProbe 验证过的 yarn 路径：cm.getNoiseConfig().getNoiseRouter().finalDensity()
            net.minecraft.world.gen.noise.NoiseConfig nc = world.getChunkManager().getNoiseConfig();
            net.minecraft.world.gen.noise.NoiseRouter router = nc.getNoiseRouter();
            DensityFunction df = router.finalDensity();
            int wx = cx * 16 + bx, wz = cz * 16 + bz;
            int minY = world.getBottomY(), height = world.getHeight();
            StringBuilder sb = new StringBuilder();
            for (int y = minY; y < minY + height; y += 4) {
                double v = df.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                sb.append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
            }
            Path out = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
            String name = "vanilla_density_" + dim + "_c" + cx + "_" + cz + "_b" + bx + "_" + bz + ".txt";
            Files.writeString(out.resolve(name), sb.toString(), StandardCharsets.UTF_8);
            System.out.println("[DensityProbe] " + name + " -> " + out.resolve(name) + " (" + height / 4 + " points)");

            // 游戏实际路径：反射跑 cns 完整生成链（c2me MixinNoiseChunkGenerator 确认的流程）
            // sampleStartDensity → sampleEndDensity(cellX) → onSampledCellCorners(cellY,cellZ)
            // → interpolateY/X/Z → sampleBlockState（游戏实际方块）
            // 只采样列 (8,8)（cellX=2, cellBlockX=0, cellZ=2, cellBlockZ=0）对比 C++
            try {
                java.lang.reflect.Method mSS = cns.getClass().getDeclaredMethod("sampleStartDensity");
                java.lang.reflect.Method mSE = cns.getClass().getDeclaredMethod("sampleEndDensity", int.class);
                java.lang.reflect.Method mOS = cns.getClass().getDeclaredMethod("onSampledCellCorners", int.class, int.class);
                java.lang.reflect.Method mIY = cns.getClass().getDeclaredMethod("interpolateY", int.class, double.class);
                java.lang.reflect.Method mIX = cns.getClass().getDeclaredMethod("interpolateX", int.class, double.class);
                java.lang.reflect.Method mIZ = cns.getClass().getDeclaredMethod("interpolateZ", int.class, double.class);
                java.lang.reflect.Method mSB = cns.getClass().getDeclaredMethod("sampleBlockState");
                java.lang.reflect.Method mSW = cns.getClass().getDeclaredMethod("swapBuffers");
                for (java.lang.reflect.Method m : new java.lang.reflect.Method[]{mSS, mSE, mOS, mIY, mIX, mIZ, mSB, mSW}) m.setAccessible(true);
                int cell = 4;      // horizontalCellBlockCount
                int vcell = 8;     // verticalCellBlockCount（1.20.1 垂直 cell = 8 块）
                int cellHeight = 48;  // 384/8
                int minCellY = -8;    // -64/8
                mSS.invoke(cns);
                StringBuilder sbCns = new StringBuilder();
                // 严格复刻 c2me MixinNoiseChunkGenerator.populateNoise（= vanilla 1.20.1 ChunkNoiseGenerator.populateNoise）：
                // cellX/cellZ/cbx/cbz 全正向；cellY/vb 反向；blockX/blockZ 世界坐标（interpolateX 用它算 cellBlockX）
                for (int cellX = 0; cellX < cell; cellX++) {
                    mSE.invoke(cns, cellX);
                    for (int cellZ = 0; cellZ < cell; cellZ++) {
                        for (int cellY = cellHeight - 1; cellY >= 0; cellY--) {
                            mOS.invoke(cns, cellY, cellZ);
                            for (int vb = vcell - 1; vb >= 0; vb--) {
                                int blockY = (minCellY + cellY) * vcell + vb;  // 世界 y
                                double vy = (double) vb / (double) vcell;
                                for (int cbx = 0; cbx < cell; cbx++) {
                                    int blockX = cx * 16 + cellX * cell + cbx;   // 世界坐标（c2me: chunkStartX + ...）
                                    double vx = (double) cbx / (double) cell;
                                    for (int cbz = 0; cbz < cell; cbz++) {
                                        int blockZ = cz * 16 + cellZ * cell + cbz;  // 世界坐标
                                        double vz = (double) cbz / (double) cell;
                                        mIY.invoke(cns, blockY, vy);
                                        mIX.invoke(cns, blockX, vx);
                                        mIZ.invoke(cns, blockZ, vz);
                                        if ((blockX & 15) == bx && (blockZ & 15) == bz) {
                                            // 遍历 8 个 interpolators（ChunkNoiseSampler 的组件插值器：finalDensity/vein 等），
                                            // dump 每个的当前插值值——vein_ridged 的 ore_vein_a/b 在其中（找 min/max 特征匹配）
                                            java.lang.reflect.Field fInterps = null;
                                            try { fInterps = cns.getClass().getDeclaredField("interpolators"); }
                                            catch (NoSuchFieldException e) { fInterps = cns.getClass().getSuperclass().getDeclaredField("interpolators"); }
                                            fInterps.setAccessible(true);
                                            java.util.List<?> interps = (java.util.List<?>) fInterps.get(cns);
                                            if (sbCns.length() == 0) {  // 首行：min/max 特征
                                                for (int ii = 0; ii < interps.size(); ii++) {
                                                    Object dii = interps.get(ii);
                                                    System.out.println("[InterpDiag] idx=" + ii + " min="
                                                            + ((net.minecraft.world.gen.densityfunction.DensityFunction) dii).minValue()
                                                            + " max=" + ((net.minecraft.world.gen.densityfunction.DensityFunction) dii).maxValue()
                                                            + " class=" + dii.getClass().getName());
                                                }
                                            }
                                            java.lang.reflect.Method mDI0 = interps.get(0).getClass().getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class);
                                            for (int ii = 0; ii < interps.size(); ii++) {
                                                double dv2 = (double) mDI0.invoke(interps.get(ii), (net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos) cns);
                                                sbCns.append(blockY).append(' ').append(ii).append(' ')
                                                     .append(String.format(java.util.Locale.ROOT, "%.6f", dv2)).append('\n');
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    mSW.invoke(cns);
                }
                Files.writeString(out.resolve(name.replace(".txt", "_cns.txt")), sbCns.toString(), StandardCharsets.UTF_8);
                System.out.println("[DensityProbe] cns 游戏实际方块 -> " + name.replace(".txt", "_cns.txt"));
            } catch (Exception ex) {
                Throwable c = ex instanceof java.lang.reflect.InvocationTargetException ? ((java.lang.reflect.InvocationTargetException) ex).getTargetException() : ex;
                System.out.println("[DensityProbe] cns chain threw " + c);
                c.printStackTrace(System.out);
            }

            // base_3d_noise 分量：优先从 cns.actualDensityFunctionCache 拿 vanilla 实际函数（rd 构造易错）
            DensityFunction b3d = null;
            try {
                Field fc2 = cns.getClass().getDeclaredField("actualDensityFunctionCache");
                fc2.setAccessible(true);
                Object cache2 = fc2.get(cns);
                if (cache2 instanceof java.util.Map<?, ?>) {
                    java.util.Map<?, ?> cmap = (java.util.Map<?, ?>) cache2;
                    StringBuilder sbCache = new StringBuilder();
                    // dump 全部缓存 key（factor/sloped_cheese/offset 等 finalDensity 树内组件在 cache 里）
                    for (Object k : cmap.keySet()) {
                        Object v = cmap.get(k);
                        sbCache.append("KEY ").append(k).append('\n');
                        if (v instanceof DensityFunction) {
                            DensityFunction fc = (DensityFunction) v;
                            String ks = k.toString();
                            if (ks.contains("factor") || ks.contains("sloped_cheese") || ks.contains("offset")
                                    || ks.contains("base_3d_noise") || ks.contains("jaggedness")
                                    || ks.contains("RangeChoice") || ks.contains("Spline[spline")
                                    || ks.contains("ShiftedNoise") || ks.contains("Interpolated, wrapped")
                                    || ks.contains("minecraft:jagged") || ks.contains("Noise[noise=")) {
                                if (ks.contains("base_3d_noise") && b3d == null) b3d = fc;
                                // Noise 实例（含 scale 的 NoiseDF）：多 y 采样（对比 C++ -noiseDump）
                                if (ks.contains("Noise[noise=")) {
                                    for (int y : new int[]{-64, 0, 48}) {
                                        double vv = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                                        sbCache.append("NOISE ").append(y).append(' ')
                                               .append(String.format(java.util.Locale.ROOT, "%.9f", vv)).append('\n');
                                    }
                                }
                                // jagged Noise 实例：多 y 采样（y 无关 = xz_scale 大 y_scale=0 的 nj）
                                if (ks.contains("minecraft:jagged")) {
                                    for (int y : new int[]{-64, 0, 48}) {
                                        double vv = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                                        sbCache.append("JAGGED ").append(y).append(' ')
                                               .append(String.format(java.util.Locale.ROOT, "%.9f", vv)).append('\n');
                                    }
                                }
                                // 每个候选 dump y=-64,0,48（finalDensity 树内组件定位）
                                for (int y : new int[]{-64, 0, 48}) {
                                    double vv = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                                    sbCache.append(ks.substring(0, Math.min(90, ks.length())).replace('\n', ' '))
                                          .append(' ').append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", vv)).append('\n');
                                }
                                // Spline 实例：额外采样 5×5 FlatCache 网格角点（对比 C++ WG_SPLINEDEBUG）
                                if (ks.contains("Spline[spline=Implementation")) {
                                    int bx0 = wx & ~15, bz0 = wz & ~15;
                                    for (int gi = 0; gi < 5; gi++)
                                        for (int gj = 0; gj < 5; gj++) {
                                            double vv = fc.sample(new DensityFunction.UnblendedNoisePos(bx0 + gi * 4, 0, bz0 + gj * 4));
                                            sbCache.append("GRID ").append(bx0 + gi * 4).append(' ').append(bz0 + gj * 4)
                                                   .append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", vv)).append('\n');
                                        }
                                }
                            }
                        }
                    }
                    Files.writeString(out.resolve(name.replace(".txt", "_cache.txt")), sbCache.toString(), StandardCharsets.UTF_8);
                    System.out.println("[DensityProbe] cache -> " + out.resolve(name.replace(".txt", "_cache.txt")));
                }
            } catch (Exception ignored) {
            }
            StringBuilder sb2 = new StringBuilder();
            // 分量 dump（负坐标 spline 定位）：router.depth/continents/erosion vs C++ WG_SURFDUMP
            try {
                StringBuilder sbC = new StringBuilder();
                for (String comp : new String[]{"depth", "continents", "erosion", "shiftX", "shiftZ",
                        "barrierNoise", "fluidLevelFloodednessNoise", "fluidLevelSpreadNoise", "lavaNoise",
                        "veinToggle", "veinRidged", "veinGap", "initialDensity", "factor", "jaggedness", "ridges"}) {
                    try {
                        var m = router.getClass().getMethod(comp);
                        DensityFunction fc = (DensityFunction) m.invoke(router);
                        for (int y = minY; y < minY + height; y += 4) {
                            double v = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                            sbC.append(comp).append(' ').append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
                        }
                    } catch (NoSuchMethodException ex) {
                        // yarn 名可能是 getXxx()
                        String alt = "get" + Character.toUpperCase(comp.charAt(0)) + comp.substring(1);
                        try {
                            var m = router.getClass().getMethod(alt);
                            DensityFunction fc = (DensityFunction) m.invoke(router);
                            for (int y = minY; y < minY + height; y += 4) {
                                double v = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                                sbC.append(comp).append(' ').append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
                            }
                        } catch (Exception ex2) {
                            sbC.append(comp).append(" <no-method ").append(ex2).append(">\n");
                        }
                    } catch (Exception ex) {
                        sbC.append(comp).append(" <threw ").append(ex).append(">\n");
                    }
                }
                Files.writeString(out.resolve(name.replace(".txt", "_comps.txt")), sbC.toString(), StandardCharsets.UTF_8);
                System.out.println("[DensityProbe] comps -> " + out.resolve(name.replace(".txt", "_comps.txt")));

                // 从 DENSITY_FUNCTION registry 直取 factor/sloped_cheese/jaggedness/offset（finalDensity 树内部组件，
                // router 无方法；对比 C++ -namedDump "minecraft:overworld/<name>"）
                try {
                    var regMgr = world.getRegistryManager();
                    var dfReg = regMgr.get(net.minecraft.registry.RegistryKeys.DENSITY_FUNCTION);
                    StringBuilder sbR = new StringBuilder();
                    for (String dfName : new String[]{"factor", "sloped_cheese", "jaggedness", "offset", "base_3d_noise", "ridges", "ridges_folded", "continents", "erosion"}) {
                        DensityFunction fc = null;
                        for (var id : dfReg.getIds()) {  // 遍历匹配（registry key 形如 minecraft:overworld/factor）
                            if (id.getPath().endsWith("/" + dfName) || id.getPath().equals(dfName)) {
                                fc = dfReg.get(id); break;
                            }
                        }
                        if (fc != null) {
                            for (int y = minY; y < minY + height; y += 4) {
                                double v = fc.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                                sbR.append(dfName).append(' ').append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
                            }
                        } else {
                            sbR.append(dfName).append(" <registry-null>\n");
                        }
                    }
                    Files.writeString(out.resolve(name.replace(".txt", "_dfreg.txt")), sbR.toString(), StandardCharsets.UTF_8);
                    System.out.println("[DensityProbe] dfreg -> " + out.resolve(name.replace(".txt", "_dfreg.txt")));
                } catch (Exception ex) {
                    System.out.println("[DensityProbe] dfreg threw " + ex);
                }
            } catch (Exception ex) {
                System.out.println("[DensityProbe] comps threw " + ex);
            }
            if (b3d != null) {
                for (int y = minY; y < minY + height; y += 4) {
                    double v = b3d.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                    sb2.append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
                }
            } else {
                // fallback：RouterProbe 构造方式（下界参数 0.25/0.375/80/60/8）
                java.lang.reflect.Field rdField = net.minecraft.world.gen.noise.NoiseConfig.class.getDeclaredField("randomDeriver");
                rdField.setAccessible(true);
                var rd = (net.minecraft.util.math.random.RandomSplitter) rdField.get(nc);
                boolean nether = dim.equals("nether");
                var b3d2 = new net.minecraft.util.math.noise.InterpolatedNoiseSampler(
                        rd.split(new net.minecraft.util.Identifier("terrain")),
                        0.25, nether ? 0.375 : 0.125, 80.0, nether ? 60.0 : 160.0, 8.0);
                for (int y = minY; y < minY + height; y += 4) {
                    double v = b3d2.sample(new DensityFunction.UnblendedNoisePos(wx, y, wz));
                    sb2.append(y).append(' ').append(String.format(java.util.Locale.ROOT, "%.6f", v)).append('\n');
                }
            }
            String name2 = "vanilla_b3d_" + dim + "_c" + cx + "_" + cz + "_b" + bx + "_" + bz + ".txt";
            Files.writeString(out.resolve(name2), sb2.toString(), StandardCharsets.UTF_8);
            System.out.println("[DensityProbe] " + name2 + " -> " + out.resolve(name2));
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        System.out.println("[DensityProbe] DONE, stopping server");
        server.stop(false);
    }
}
