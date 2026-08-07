package wg.bench;

import net.minecraft.block.Block;
import net.minecraft.registry.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * vanilla 方块参照导出：
 * 1. FULL 生成 size×size chunk，导出 16×16×384 方块（vanilla block 注册表 id）
 * 2. 导出方块 id→name 映射（blocks.json），供 C++ 侧使用同一张表
 * 用法：-Dblock.probe=1 -Dbench.seed=<seed> -Dbench.size=<n> -Dbench.originX=<x> -Dbench.originZ=<z>
 */
public class BlockProbe {
    /** 驱动真实 ChunkNoiseSampler 插值循环到目标块，打印真实 blockState 与 veinToggle 插值 */
    private static void driveCnsTo(Object cns, int bx, int by, int bz, net.minecraft.world.gen.noise.NoiseConfig nc) throws Exception {
        Class<?> cls = cns.getClass();
        java.lang.reflect.Method mStart = cls.getMethod("sampleStartDensity");
        java.lang.reflect.Method mEnd = cls.getMethod("sampleEndDensity", int.class);
        java.lang.reflect.Method mOn = cls.getMethod("onSampledCellCorners", int.class, int.class);
        java.lang.reflect.Method mY = cls.getMethod("interpolateY", int.class, double.class);
        java.lang.reflect.Method mX = cls.getMethod("interpolateX", int.class, double.class);
        java.lang.reflect.Method mZ = cls.getMethod("interpolateZ", int.class, double.class);
        java.lang.reflect.Method mSwap = cls.getMethod("swapBuffers");
        java.lang.reflect.Method mSample = cls.getDeclaredMethod("sampleBlockState");
        mSample.setAccessible(true);
        java.lang.reflect.Method mStop = cls.getMethod("stopInterpolation");

        int ox = 3200, oz = 3200;   // chunk(200,200) 起点
        int cellX = (bx - ox) / 4;
        int cellZ = (bz - oz) / 4;
        int minCellY = -8;
        int cellY = (by + 64) / 8;

        // 反射拿 veinToggle 的 DensityInterpolator：遍历 interpolators 打印全部结构
        java.lang.reflect.Field fInterps = cls.getDeclaredField("interpolators");
        fInterps.setAccessible(true);
        java.util.List<?> interps = (java.util.List<?>) fInterps.get(cns);
        Object vtInterp = null;
        Object fdInterp = interps.isEmpty() ? null : interps.get(0);   // finalDensity 的 DensityInterpolator（BlendDensity）
        for (Object it : interps) {
            try {
                java.lang.reflect.Field fDel = it.getClass().getDeclaredField("delegate");
                fDel.setAccessible(true);
                Object del = fDel.get(it);
                System.out.println("[InterpList] " + describe(del, 2));
                if (del != null && describe(del, 2).contains("xz=1.5")) { vtInterp = it; }
            } catch (Exception ignore) { }
        }

        mStart.invoke(cns);
        outer:
        for (int o = 0; o <= cellX; o++) {
            mEnd.invoke(cns, o);
            for (int p = 0; p <= cellZ; p++) {
                for (int r = 47; r >= 0; r--) {
                    mOn.invoke(cns, r, p);
                    for (int s = 7; s >= 0; s--) {
                        int t = (minCellY + r) * 8 + s;
                        double d = s / 8.0;
                        mY.invoke(cns, t, d);
                        for (int w = 0; w < 4; w++) {
                            int x = ox + o * 4 + w;
                            double e = w / 4.0;
                            mX.invoke(cns, x, e);
                            for (int z = 0; z < 4; z++) {
                                int aa = oz + p * 4 + z;
                                double f = z / 4.0;
                                mZ.invoke(cns, aa, f);
                                Object bs = mSample.invoke(cns);
                                if (x == bx && t == by && aa == bz) {
                                    double vtVal = vtInterp != null
                                            ? (double) vtInterp.getClass().getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class)
                                                    .invoke(vtInterp, (net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos) cns)
                                            : Double.NaN;
                                    // 反射读 8 角点字段
                                    StringBuilder corners = new StringBuilder();
                                    if (vtInterp != null) {
                                        // 直接采样 delegate 于角点坐标（对照 buffer）
                                        java.lang.reflect.Field fDel = vtInterp.getClass().getDeclaredField("delegate");
                                        fDel.setAccessible(true);
                                        Object del = fDel.get(vtInterp);
                                        corners.append("delClass=").append(del.getClass().getSimpleName()).append(" ");
                                        // BlendDensity 的 input 结构（递归 describe）
                                        try {
                                            java.lang.reflect.Method mInput = del.getClass().getMethod("input");
                                            Object in = mInput.invoke(del);
                                            corners.append("input=").append(describe(in, 3)).append(" ");
                                        } catch (Exception e4) {
                                            corners.append("inputErr=").append(e4.getMessage()).append(" ");
                                        }
                                        // noiseConfig 原始 veinToggle 结构
                                        try {
                                            Object rawVt = nc.getNoiseRouter().veinToggle();
                                            corners.append("rawVt=").append(rawVt.getClass().getSimpleName()).append(" ");
                                            java.lang.reflect.Method mVal = rawVt.getClass().getMethod("function");
                                            Object func = mVal.invoke(rawVt);
                                            Object v = func.getClass().getMethod("value").invoke(func);
                                            corners.append("val=").append(v.getClass().getSimpleName()).append(" ");
                                            // 递归打印 value 结构（最多 3 层）
                                            Object cur = v;
                                            for (int depth = 0; depth < 4; depth++) {
                                                String cn = cur.getClass().getSimpleName();
                                                corners.append("L").append(depth).append("=").append(cn).append(" ");
                                                if (cn.contains("Wrapping")) {
                                                    java.lang.reflect.Method mW = cur.getClass().getMethod("wrapped");
                                                    cur = mW.invoke(cur);
                                                } else if (cn.contains("RangeChoice")) {
                                                    java.lang.reflect.Method mWhen = cur.getClass().getMethod("whenInRange");
                                                    cur = mWhen.invoke(cur);
                                                } else if (cn.contains("BlendDensity")) {
                                                    java.lang.reflect.Method mIn = cur.getClass().getMethod("input");
                                                    cur = mIn.invoke(cur);
                                                } else {
                                                    break;
                                                }
                                            }
                                        } catch (Exception e5) {
                                            corners.append("rawErr=").append(e5.getMessage()).append(" ");
                                        }
                                        var np0 = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3208, 0, 3204);
                                        var np1 = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3212, 0, 3204);
                                        var np2 = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3208, 8, 3204);
                                        var np3 = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3212, 8, 3204);
                                        java.lang.reflect.Method mDelSample = net.minecraft.world.gen.densityfunction.DensityFunction.class.getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class);
                                        corners.append("dir(3208,0,3204)=").append(String.format(java.util.Locale.ROOT, "%.4f", (double) mDelSample.invoke(del, np0))).append(" ");
                                        corners.append("dir(3212,0,3204)=").append(String.format(java.util.Locale.ROOT, "%.4f", (double) mDelSample.invoke(del, np1))).append(" ");
                                        corners.append("dir(3208,8,3204)=").append(String.format(java.util.Locale.ROOT, "%.4f", (double) mDelSample.invoke(del, np2))).append(" ");
                                        corners.append("dir(3212,8,3204)=").append(String.format(java.util.Locale.ROOT, "%.4f", (double) mDelSample.invoke(del, np3))).append(" ");
                                        for (String fn : new String[]{"x0y0z0", "x1y0z0", "x0y1z0", "x1y1z0", "x0y0z1", "x1y0z1", "x0y1z1", "x1y1z1", "result"}) {
                                            try {
                                                java.lang.reflect.Field fld = vtInterp.getClass().getDeclaredField(fn);
                                                fld.setAccessible(true);
                                                corners.append(fn).append("=").append(String.format(java.util.Locale.ROOT, "%.4f", fld.getDouble(vtInterp))).append(" ");
                                            } catch (Exception ignore) { }
                                        }
                                        try {
                                            java.lang.reflect.Field fsbx = cls.getDeclaredField("startBlockX");
                                            fsbx.setAccessible(true);
                                            java.lang.reflect.Field fsbz = cls.getDeclaredField("startBlockZ");
                                            fsbz.setAccessible(true);
                                            java.lang.reflect.Field fsby = cls.getDeclaredField("startBlockY");
                                            fsby.setAccessible(true);
                                            corners.append("cnsSBX=").append(fsbx.getInt(cns)).append(" SBY=").append(fsby.getInt(cns)).append(" SBZ=").append(fsbz.getInt(cns));
                                        } catch (Exception ignore) { }
                                    }
                                    System.out.println(String.format(java.util.Locale.ROOT,
                                            "[VeinDiag] (%d,%d,%d) block=%s veinToggle=%.6f %s",
                                            bx, by, bz, bs, vtVal, corners));
                                    // 反射采样 beardifying（StructureWeightSampler）与 finalDensity
                                    try {
                                        java.lang.reflect.Field fBeard = cls.getDeclaredField("beardifying");
                                        fBeard.setAccessible(true);
                                        Object beard = fBeard.get(cns);
                                        var npb = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(bx, by, bz);
                                        double bv = (double) net.minecraft.world.gen.densityfunction.DensityFunction.class.getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class).invoke(beard, npb);
                                        double fdv = nc.getNoiseRouter().finalDensity().sample(npb);
                                        double fdInterpV = fdInterp != null
                                                ? (double) fdInterp.getClass().getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class)
                                                        .invoke(fdInterp, (net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos) cns)
                                                : Double.NaN;
                                        System.out.println(String.format(java.util.Locale.ROOT,
                                                "[BeardDiag] (%d,%d,%d) beard=%.6f finalDensity(raw)=%.6f finalDensityInterp=%.6f",
                                                bx, by, bz, bv, fdv, fdInterpV));
                                    } catch (Exception e7) {
                                        System.out.println("[BeardDiag] failed: " + e7);
                                    }
                                    mStop.invoke(cns);
                                    break outer;
                                }
                            }
                        }
                    }
                }
            }
            mSwap.invoke(cns);
        }
    }

    /** 递归描述 DensityFunction 结构（最多 depth 层，带关键参数） */
    private static String describe(Object f, int depth) {
        if (f == null || depth <= 0) return f == null ? "null" : f.getClass().getSimpleName();
        StringBuilder sb = new StringBuilder(f.getClass().getSimpleName());
        sb.append("[");
        try {
            String cn = f.getClass().getSimpleName();
            if (cn.contains("Wrapping")) {
                sb.append(describe(f.getClass().getMethod("wrapped").invoke(f), depth - 1));
            } else if (cn.contains("RangeChoice")) {
                sb.append("min=").append(f.getClass().getMethod("minInclusive").invoke(f))
                  .append(",max=").append(f.getClass().getMethod("maxExclusive").invoke(f))
                  .append(",whenIn=").append(describe(f.getClass().getMethod("whenInRange").invoke(f), depth - 1));
            } else if (cn.contains("BlendDensity")) {
                sb.append(describe(f.getClass().getMethod("input").invoke(f), depth - 1));
            } else if (cn.contains("Noise")) {
                java.lang.reflect.Method mNoise = f.getClass().getMethod("noise");
                Object rec = mNoise.invoke(f);
                java.lang.reflect.Field fSamp = rec.getClass().getDeclaredField("noise");
                fSamp.setAccessible(true);
                Object samp = fSamp.get(rec);
                java.lang.reflect.Method mXz = f.getClass().getMethod("xzScale");
                java.lang.reflect.Method mY = f.getClass().getMethod("yScale");
                sb.append("samp=").append(samp == null ? "null" : samp.getClass().getSimpleName())
                  .append(",xz=").append(mXz.invoke(f)).append(",y=").append(mY.invoke(f));
            } else if (cn.contains("LinearOperation") || cn.contains("Binary")) {
                for (String am : new String[]{"argument1", "argument2"}) {
                    try {
                        sb.append(am).append("=").append(describe(f.getClass().getMethod(am).invoke(f), depth - 1)).append(",");
                    } catch (Exception ignore) { }
                }
            } else if (cn.contains("Constant")) {
                sb.append(f.getClass().getMethod("value").invoke(f));
            } else if (cn.contains("RegistryEntryHolder")) {
                Object fn = f.getClass().getMethod("function").invoke(f);
                Object v = fn.getClass().getMethod("value").invoke(fn);
                sb.append(describe(v, depth - 1));
            } else if (cn.contains("UnaryOperation") || cn.contains("Squeeze") || cn.contains("Abs")) {
                sb.append(describe(f.getClass().getMethod("input").invoke(f), depth - 1));
            } else if (cn.contains("Clamp")) {
                sb.append("min=").append(f.getClass().getMethod("minValue").invoke(f))
                  .append(",in=").append(describe(f.getClass().getMethod("input").invoke(f), depth - 1));
            } else if (cn.contains("Y")) {
                sb.append("Y");
            } else {
                sb.append(cn);
            }
        } catch (Exception e) {
            sb.append("err=").append(e.getMessage());
        }
        sb.append("]");
        return sb.toString();
    }

    private static final int MIN_Y = -64;
    private static final int HEIGHT = 384;

    public static void run(MinecraftServer server) {
        long seed = Long.parseLong(System.getProperty("bench.seed", "-8248318472910187742"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        int originX = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int originZ = Integer.parseInt(System.getProperty("bench.originZ", "3208"));

        Path dataDir = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
        try {
            Files.createDirectories(dataDir);
        } catch (Exception e) {
            throw new RuntimeException("无法创建输出目录: " + dataDir, e);
        }

        String dim = System.getProperty("blockProbe.dimension", "overworld");
        ServerWorld world = dim.equals("nether")
                ? server.getWorld(net.minecraft.world.World.NETHER)
                : server.getOverworld();
        if (world == null) {
            System.out.println("[BlockProbe] world " + dim + " not found, stopping");
            server.stop(false);
            return;
        }
        int worldMinY = world.getBottomY();
        int worldHeight = world.getHeight();
        ServerChunkManager chunkManager = world.getChunkManager();
        ChunkGenerator generator = chunkManager.getChunkGenerator();
        if (!(generator instanceof NoiseChunkGenerator)) {
            System.err.println("错误：期望 NoiseChunkGenerator，实际为 " + generator.getClass().getName());
            server.stop(false);
            return;
        }

        System.out.println("[BlockProbe] seed=" + seed + " size=" + size + " origin=(" + originX + "," + originZ + ")");
        System.out.println("[BlockProbe] worldSeed=" + world.getSeed());
        // 诊断：DimensionType.field_35479（Aquifer 无效液面常量，C++ 用 INT32_MAX 对应）
        try {
            java.lang.reflect.Field f35479 = net.minecraft.world.dimension.DimensionType.class.getDeclaredField("field_35479");
            f35479.setAccessible(true);
            System.out.println("[DimDiag] field_35479=" + f35479.getInt(null));
        } catch (Exception e) {
            System.out.println("[DimDiag] failed: " + e);
        }

        // 诊断：verticalGradient deepslate 的 random 值（对照 C++）
        try {
            net.minecraft.world.gen.noise.NoiseConfig nc = chunkManager.getNoiseConfig();
            for (int y : new int[]{2, 3, 4, 5}) {
                double rnd = nc.getOrCreateRandomDeriver(new net.minecraft.util.Identifier("deepslate"))
                        .split(3200, y, 3200).nextFloat();
                long h = net.minecraft.util.math.MathHelper.hashCode(3200, y, 3200);
                System.out.println(String.format(java.util.Locale.ROOT, "[VgDiag] y=%d rnd=%.6f hash=%d", y, rnd, h));
            }
            net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos np =
                    new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3217, -36, 3200);
            System.out.println(String.format(java.util.Locale.ROOT, "[CppCmp] floodedness=%.6f erosion=%.6f depth=%.6f",
                    nc.getNoiseRouter().fluidLevelFloodednessNoise().sample(np),
                    nc.getNoiseRouter().erosion().sample(np),
                    nc.getNoiseRouter().depth().sample(np)));
            net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos np2 =
                    new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3218, -19, 3192);
            System.out.println(String.format(java.util.Locale.ROOT, "[CppCmpS] floodedness=%.6f erosion=%.6f depth=%.6f",
                    nc.getNoiseRouter().fluidLevelFloodednessNoise().sample(np2),
                    nc.getNoiseRouter().erosion().sample(np2),
                    nc.getNoiseRouter().depth().sample(np2)));
        } catch (Exception e) {
            System.out.println("[VgDiag] failed: " + e);
        }

        // 预热
        for (int i = 0; i < 2; i++) {
            world.getChunk(i, 0, ChunkStatus.FULL, true);
        }

        // RuleDiag：主循环前单独请求 chunk(50,-23) NOISE（未被连带推进），dump surface 前列 + 手动复刻 buildSurface 循环
        if (System.getProperty("rule.diag") != null) {
            try {
                Chunk rchunk = world.getChunk(50, -23, ChunkStatus.NOISE, true);
                System.out.println("[RuleDiag] chunk class=" + rchunk.getClass().getName() + " status=" + rchunk.getStatus());
                // applyMaterialRule 单点规则判定（真实 biomeAccess，固定 q=vx=1）
                try {
                    net.minecraft.world.gen.surfacebuilder.SurfaceBuilder sb = world.getChunkManager().getNoiseConfig().getSurfaceBuilder();
                    java.lang.reflect.Method mApply = net.minecraft.world.gen.surfacebuilder.SurfaceBuilder.class.getDeclaredMethod(
                            "applyMaterialRule",
                            net.minecraft.world.gen.surfacebuilder.MaterialRules.MaterialRule.class,
                            net.minecraft.world.gen.carver.CarverContext.class,
                            java.util.function.Function.class,
                            Chunk.class,
                            net.minecraft.world.gen.chunk.ChunkNoiseSampler.class,
                            net.minecraft.util.math.BlockPos.class,
                            boolean.class);
                    mApply.setAccessible(true);
                    net.minecraft.world.biome.source.BiomeAccess ba = world.getBiomeAccess();
                    // chunk 存储 biome（populateBiomes 填入 biome 数组）vs 实时采样——表面规则实际用哪个？
                    try {
                        for (int yy : new int[]{52, 56, 60, 63, 64, 67}) {
                            int by = yy >> 2;
                            var stored = rchunk.getBiomeForNoiseGen(804 >> 2, by, -368 >> 2);
                            String sStored = stored.getKey().map(k -> k.getValue().toString()).orElse("?");
                            var live = world.getGeneratorStoredBiome(804 >> 2, by, -368 >> 2);
                            String sLive = live.getKey().map(k -> k.getValue().toString()).orElse("?");
                            System.out.println("[RuleDiag] biomeCellY=" + by + " (y=" + yy + ") stored=" + sStored + " live=" + sLive);
                        }
                    } catch (Throwable ex4) {
                        System.out.println("[RuleDiag] biome cmp ERR " + ex4);
                    }
                    try {
                        net.minecraft.world.gen.surfacebuilder.SurfaceBuilder sb2 = world.getChunkManager().getNoiseConfig().getSurfaceBuilder();
                        Class<?> mrc = Class.forName("net.minecraft.world.gen.surfacebuilder.MaterialRules$MaterialRuleContext");
                        java.lang.reflect.Constructor<?> mrcCtor = mrc.getDeclaredConstructor(
                                net.minecraft.world.gen.surfacebuilder.SurfaceBuilder.class,
                                net.minecraft.world.gen.noise.NoiseConfig.class,
                                Chunk.class,
                                net.minecraft.world.gen.chunk.ChunkNoiseSampler.class,
                                java.util.function.Function.class,
                                net.minecraft.registry.Registry.class,
                                net.minecraft.world.gen.HeightContext.class);
                        mrcCtor.setAccessible(true);
                        // q/vx 用 RuleDiag 循环实测值：(y,q,vx) 从模拟循环
                        int[][] qv = {{74,1,1},{73,2,1},{72,3,1},{71,4,1},{70,5,1},{69,6,1},{68,7,1},{67,8,5},
                                      {66,9,4},{65,10,3},{64,11,2},{63,12,1},{62,13,1},{61,14,1},{60,15,31},
                                      {59,16,30},{58,17,29},{57,18,28},{56,19,27},{55,20,26},{54,21,25}};
                        net.minecraft.world.gen.surfacebuilder.MaterialRules.MaterialRule rule =
                                net.minecraft.world.gen.surfacebuilder.VanillaSurfaceRules.createDefaultRule(true, false, true);
                        for (int[] qvi : qv) {
                            int yy = qvi[0];
                            Object ctx = mrcCtor.newInstance(sb2, world.getChunkManager().getNoiseConfig(), rchunk,
                                    null, (java.util.function.Function<net.minecraft.util.math.BlockPos, net.minecraft.registry.entry.RegistryEntry<net.minecraft.world.biome.Biome>>) ba::getBiome,
                                    world.getRegistryManager().get(net.minecraft.registry.RegistryKeys.BIOME),
                                    new net.minecraft.world.gen.HeightContext(
                                            (net.minecraft.world.gen.chunk.NoiseChunkGenerator) world.getChunkManager().getChunkGenerator(), rchunk));
                            java.lang.reflect.Method mInitH = mrc.getDeclaredMethod("initHorizontalContext", int.class, int.class);
                            mInitH.setAccessible(true);
                            mInitH.invoke(ctx, 804, -368);
                            java.lang.reflect.Method mInitV = mrc.getDeclaredMethod("initVerticalContext", int.class, int.class, int.class, int.class, int.class, int.class);
                            mInitV.setAccessible(true);
                            mInitV.invoke(ctx, qvi[1], qvi[2], Integer.MIN_VALUE, 804, yy, -368);
                            java.lang.reflect.Method mApply2 = rule.getClass().getMethod("apply", mrc);
                            Object blockStateRule = mApply2.invoke(rule, ctx);
                            java.lang.reflect.Method mTry = blockStateRule.getClass().getMethod("tryApply", int.class, int.class, int.class);
                            Object res = mTry.invoke(blockStateRule, 804, yy, -368);
                            System.out.println("[RuleDiag] manual q=" + qvi[1] + " vx=" + qvi[2] + " y=" + yy + " -> " +
                                    (res == null ? "null(保持)" : net.minecraft.registry.Registries.BLOCK.getId(((net.minecraft.block.BlockState) res).getBlock()) + " " + net.minecraft.registry.Registries.BLOCK.getRawId(((net.minecraft.block.BlockState) res).getBlock())));
                        }
                    } catch (Throwable ex3) {
                        System.out.println("[RuleDiag] manual ERR " + ex3);
                        if (ex3.getCause() != null) ex3.getCause().printStackTrace(System.err);
                    }
                } catch (Throwable ex2) {
                    System.out.println("[RuleDiag] applyMaterialRule ERR " + ex2);
                    if (ex2.getCause() != null) ex2.getCause().printStackTrace(System.err);
                }
                BlockPos.Mutable rpos = new BlockPos.Mutable();
                StringBuilder rsb = new StringBuilder();
                for (int y = 50; y <= 80; y++) {
                    net.minecraft.block.Block bb = rchunk.getBlockState(rpos.set(4, y, 0)).getBlock();
                    rsb.append(y).append('=').append(net.minecraft.registry.Registries.BLOCK.getId(bb))
                       .append(' ').append(net.minecraft.registry.Registries.BLOCK.getRawId(bb)).append(" | ");
                }
                System.out.println("[RuleDiag] (804,-368) col(NOISE, pre-loop): " + rsb);
                int hm = rchunk.getHeightmap(net.minecraft.world.Heightmap.Type.WORLD_SURFACE_WG).get(4, 0);
                System.out.println("[RuleDiag] heightmap(4,0)=" + hm);
                // 手动复刻 buildSurface 循环：(804,-368) 列
                int m = 804, n = -368;
                int minY = rchunk.getBottomY();
                int q = 0, r = Integer.MIN_VALUE, s = Integer.MAX_VALUE;
                int p = hm + 1;
                for (int u = p; u >= minY; u--) {
                    net.minecraft.block.BlockState st = rchunk.getBlockState(rpos.set(m % 16, u, n % 16));
                    int raw = net.minecraft.registry.Registries.BLOCK.getRawId(st.getBlock());
                    if (st.isAir()) {
                        q = 0; r = Integer.MIN_VALUE;
                    } else if (!st.getFluidState().isEmpty()) {
                        if (r == Integer.MIN_VALUE) r = u + 1;
                    } else {
                        if (s >= u) {
                            s = net.minecraft.world.dimension.DimensionType.field_35479;
                            for (int v = u - 1; v >= minY - 1; v--) {
                                net.minecraft.block.BlockState st2 = rchunk.getBlockState(rpos.set(m % 16, v, n % 16));
                                if (!st2.isOf(net.minecraft.block.Blocks.STONE)) { s = v + 1; break; }
                            }
                        }
                        q++;
                        int vx = u - s + 1;
                        if (m == 804 && n == -368) {
                            System.out.println("[RuleDiag] y=" + u + " q=" + q + " vx=" + vx + " r=" + r + " s=" + s + " raw=" + raw);
                        }
                    }
                }
            } catch (Throwable ex) {
                System.out.println("[RuleDiag] ERR " + ex);
            }
        }

        Path blocksFile = dataDir.resolve("vanilla_" + seed + "_" + size + "_" + originX + "_" + originZ
                + (dim.equals("nether") ? "_nether" : "") + ".blocks");
        try (DataOutputStream out = new DataOutputStream(
                new BufferedOutputStream(new FileOutputStream(blocksFile.toFile())))) {
            out.writeInt(0x57474232); // "WGB2"
            out.writeLong(seed);
            out.writeInt(size);
            out.writeInt(originX);
            out.writeInt(originZ);
            out.writeInt(worldMinY);
            out.writeInt(worldHeight);

            BlockPos.Mutable pos = new BlockPos.Mutable();
            for (int cz = 0; cz < size; cz++) {
                for (int cx = 0; cx < size; cx++) {
                    int wx = originX / 16 + cx;
                    int wz = originZ / 16 + cz;
                    ChunkPos chunkPos = new ChunkPos(wx, wz);
                    long t0 = System.nanoTime();
                    // 先生成到 NOISE（ChunkNoiseSampler 存活期），驱动插值诊断，再补 SURFACE
                    Chunk chunk = world.getChunk(wx, wz, ChunkStatus.NOISE, true);
                    if (wx == 45 && wz == -27) {
                        // EstDiag（8576 的 chunk(45,-27)——bench origin 720,-432）
                        try {
                            java.lang.reflect.Field fCnsDiag = Chunk.class.getDeclaredField("chunkNoiseSampler");
                            fCnsDiag.setAccessible(true);
                            Object cnsD = fCnsDiag.get(chunk);
                            java.lang.reflect.Method mEst = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getMethod("estimateSurfaceHeight", int.class, int.class);
                            System.out.println("[EstDiag] (45,-27) chunk est(738,-421)=" + mEst.invoke(cnsD, 738, -421));
                            for (int[] pt : new int[][]{{739, -427}, {742, -427}, {805, -427}, {728, -408}, {800, -431}, {742, 64}, {739, 56}, {738, -421}, {805, -432}, {808, -432}, {803, -432}}) {
                                System.out.println("[EstDiag] (" + pt[0] + "," + pt[1] + ") estimateSurfaceHeight=" + mEst.invoke(cnsD, pt[0], pt[1]));
                            }
                            java.lang.reflect.Field fIni = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getDeclaredField("initialDensityWithoutJaggedness");
                            fIni.setAccessible(true);
                            Object ini = fIni.get(cnsD);
                            var npX = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3200, 3200 >> 4 * 0 + 0, 3211);
                            for (int yy : new int[]{64, 56, 48, 40, 32, 24}) {
                                npX = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3200, yy, 3211);
                                double iv = (double) net.minecraft.world.gen.densityfunction.DensityFunction.class.getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class).invoke(ini, npX);
                                System.out.println(String.format(java.util.Locale.ROOT, "[EstDiag] initialDensity(3200,%d,3211)=%.6f", yy, iv));
                            }
                            // (738,-421) 的 initial_density 列（cns 查表版——网格覆盖判定）
                            for (int yy : new int[]{72, 64, 56, 48}) {
                                npX = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(738, yy, -421);
                                double iv = (double) net.minecraft.world.gen.densityfunction.DensityFunction.class.getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class).invoke(ini, npX);
                                System.out.println(String.format(java.util.Locale.ROOT, "[EstDiag] cns-ini(738,%d,-421)=%.6f", yy, iv));
                            }
                        } catch (Exception e9) {
                            System.out.println("[EstDiag] failed: " + e9);
                        }
                        try {
                            java.lang.reflect.Field fCns = Chunk.class.getDeclaredField("chunkNoiseSampler");
                            fCns.setAccessible(true);
                            Object cns = fCns.get(chunk);
                            if (cns != null) {
                                driveCnsTo(cns, 3211, 4, 3204, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3211, 40, 3204, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3211, -30, 3204, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3215, -26, 3200, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3220, -32, 3200, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3214, 31, 3212, chunkManager.getNoiseConfig());
                                driveCnsTo(cns, 3201, 56, 3202, chunkManager.getNoiseConfig());
                            } else {
                                System.out.println("[VeinDiag] chunkNoiseSampler null after NOISE");
                            }
                        } catch (Exception e2) {
                            if (e2.getCause() != null) {
                                e2.getCause().printStackTrace(System.err);
                            } else {
                                System.out.println("[VeinDiag] failed: " + e2);
                            }
                        }
                    }
                    // EstDiag2：4 角 estimateSurfaceHeight（Java 4 角 lerp2 插值用）——支持 8576 chunk(50,-23) 与 3200 chunk(200,201)
                    if (wx == 50 && wz == -23) {
                        try {
                            java.lang.reflect.Field fCns2 = Chunk.class.getDeclaredField("chunkNoiseSampler");
                            fCns2.setAccessible(true);
                            Object cns2 = fCns2.get(chunk);
                            if (cns2 != null) {
                                java.lang.reflect.Method mEst2 = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getMethod("estimateSurfaceHeight", int.class, int.class);
                                System.out.println("[EstDiag2] (50,-23) 4角 est: " + mEst2.invoke(cns2, 800, -368) + " " + mEst2.invoke(cns2, 816, -368) + " " + mEst2.invoke(cns2, 800, -352) + " " + mEst2.invoke(cns2, 816, -352));
                                float fx = (804 & 15) / 16.0f, fz = (-368 & 15) / 16.0f;
                                double e0 = (double) mEst2.invoke(cns2, 800, -368), e1 = (double) mEst2.invoke(cns2, 816, -368);
                                double e2 = (double) mEst2.invoke(cns2, 800, -352), e3 = (double) mEst2.invoke(cns2, 816, -352);
                                double k = Math.floor(net.minecraft.util.math.MathHelper.lerp2(fx, fz, e0, e1, e2, e3));
                                System.out.println(String.format(java.util.Locale.ROOT, "[EstDiag2] k(804,-368)=%.6f (lerp2 fx=%.2f fz=%.2f)", k, fx, fz));
                            } else {
                                System.out.println("[EstDiag2] (50,-23) chunkNoiseSampler null");
                            }
                        } catch (Exception e10) {
                            System.out.println("[EstDiag2] (50,-23) failed: " + e10);
                        }
                    }
                    if (wx == 200 && wz == 201) {
                        try {
                            java.lang.reflect.Field fCns2 = Chunk.class.getDeclaredField("chunkNoiseSampler");
                            fCns2.setAccessible(true);
                            Object cns2 = fCns2.get(chunk);
                            if (cns2 != null) {
                                java.lang.reflect.Method mEst2 = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getMethod("estimateSurfaceHeight", int.class, int.class);
                                System.out.println("[EstDiag2] (200,201) 4角 est: " + mEst2.invoke(cns2, 3200, 3216) + " " + mEst2.invoke(cns2, 3216, 3216) + " " + mEst2.invoke(cns2, 3200, 3232) + " " + mEst2.invoke(cns2, 3216, 3232));
                                // (3214,3227)：fx=(3214&15)/16=14/16=0.875, fz=(3227&15)/16=11/16=0.6875
                                float fx = (3214 & 15) / 16.0f, fz = (3227 & 15) / 16.0f;
                                double e0 = ((Number) mEst2.invoke(cns2, 3200, 3216)).doubleValue(), e1 = ((Number) mEst2.invoke(cns2, 3216, 3216)).doubleValue();
                                double e2 = ((Number) mEst2.invoke(cns2, 3200, 3232)).doubleValue(), e3 = ((Number) mEst2.invoke(cns2, 3216, 3232)).doubleValue();
                                double k = Math.floor(net.minecraft.util.math.MathHelper.lerp2(fx, fz, e0, e1, e2, e3));
                                System.out.println(String.format(java.util.Locale.ROOT, "[EstDiag2] k(3214,3227)=%.6f (lerp2 fx=%.2f fz=%.2f)", k, fx, fz));
                                // initialDensityWithoutJaggedness 直接采样（C++ 对比：same pos）
                                java.lang.reflect.Field fIni2 = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getDeclaredField("initialDensityWithoutJaggedness");
                                fIni2.setAccessible(true);
                                Object ini2 = fIni2.get(cns2);
                                java.lang.reflect.Method mSample = net.minecraft.world.gen.densityfunction.DensityFunction.class.getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class);
                                for (int ly : new int[]{64, 56, 48, 40, 32, 24}) {
                                    var np = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(3200, ly, 3216);
                                    double iv = (double) mSample.invoke(ini2, np);
                                    System.out.println(String.format(java.util.Locale.ROOT, "[EstDiag2] ini(3200,%d,3216)=%.6f", ly, iv));
                                }
                            } else {
                                System.out.println("[EstDiag2] (200,201) chunkNoiseSampler null");
                            }
                        } catch (Exception e11) {
                            System.out.println("[EstDiag2] (200,201) failed: " + e11);
                        }
                    }
                    // PreDiag：NOISE 阶段 surface 前的列（(804,-368) 局部 4,0）+ heightmap + sampleRunDepth
                    if (wx == 50 && wz == -23) {
                        try {
                            net.minecraft.world.gen.surfacebuilder.SurfaceBuilder sb = world.getChunkManager().getNoiseConfig().getSurfaceBuilder();
                            java.lang.reflect.Method mRd = net.minecraft.world.gen.surfacebuilder.SurfaceBuilder.class.getDeclaredMethod("sampleRunDepth", int.class, int.class);
                            mRd.setAccessible(true);
                            int rd = (int) mRd.invoke(sb, 804, -368);
                            int hm = chunk.getHeightmap(net.minecraft.world.Heightmap.Type.WORLD_SURFACE_WG).get(4, 0);
                            System.out.println("[PreDiag] runDepth(804,-368)=" + rd + " heightmap(4,0)=" + hm);
                            StringBuilder psb = new StringBuilder();
                            for (int y = 50; y <= 80; y++) {
                                net.minecraft.block.Block bb = chunk.getBlockState(pos.set(4, y, 0)).getBlock();
                                psb.append(y).append('=').append(net.minecraft.registry.Registries.BLOCK.getId(bb))
                                   .append(' ').append(net.minecraft.registry.Registries.BLOCK.getRawId(bb)).append(" | ");
                            }
                            System.out.println("[PreDiag] (804,-368) col(NOISE): " + psb);
                        } catch (Throwable ex) {
                            System.out.println("[PreDiag] ERR " + ex);
                        }
                    }
                    chunk = world.getChunk(wx, wz, ChunkStatus.SURFACE, true);
                    if (wx == 50 && wz == -23) {
                        // ColDiag：dump (804,-368) 列（局部 4,0）Java 表面后方块，对比参照
                        StringBuilder csb = new StringBuilder();
                        for (int y = 50; y <= 80; y++) {
                            net.minecraft.block.Block bb = chunk.getBlockState(pos.set(4, y, 0)).getBlock();
                            csb.append(y).append('=').append(net.minecraft.registry.Registries.BLOCK.getId(bb))
                               .append(' ').append(net.minecraft.registry.Registries.BLOCK.getRawId(bb)).append(" | ");
                        }
                        System.out.println("[ColDiag] (804,-368) col: " + csb);
                    }
                    long t1 = System.nanoTime();
                    System.out.println("[BlockProbe] chunk (" + wx + "," + wz + ") FULL in " + (t1 - t0) / 1_000_000 + " ms");
                    out.writeInt(wx);
                    out.writeInt(wz);
                    for (int y = worldMinY; y < worldMinY + worldHeight; y++) {
                        for (int z = 0; z < 16; z++) {
                            for (int x = 0; x < 16; x++) {
                                Block block = chunk.getBlockState(pos.set(x, y, z)).getBlock();
                                out.writeShort(Registries.BLOCK.getRawId(block));
                            }
                        }
                    }
                    // biome 采样（每列 y=100，用于验证 C++ biome 查找）
                    for (int z = 0; z < 16; z++) {
                        for (int x = 0; x < 16; x++) {
                            net.minecraft.registry.entry.RegistryEntry<Biome> biome =
                                    world.getBiome(pos.set(chunkPos.getStartX() + x, 100, chunkPos.getStartZ() + z));
                            out.writeUTF(biome.getKey().map(k -> k.getValue().toString()).orElse("?"));
                        }
                    }
                }
            }
        } catch (Exception e) {
            throw new RuntimeException(e);
        }

        // 导出方块 id→name 映射（全注册表）
        Map<String, Integer> idToName = new TreeMap<>();
        for (Block block : Registries.BLOCK) {
            idToName.put(Registries.BLOCK.getId(block).toString(), Registries.BLOCK.getRawId(block));
        }
        StringBuilder sb = new StringBuilder("{\n");
        List<Map.Entry<String, Integer>> entries = new ArrayList<>(idToName.entrySet());
        for (int i = 0; i < entries.size(); i++) {
            Map.Entry<String, Integer> e = entries.get(i);
            sb.append("  \"").append(e.getKey()).append("\": ").append(e.getValue());
            if (i < entries.size() - 1) sb.append(",");
            sb.append("\n");
        }
        sb.append("}\n");
        try {
            Files.writeString(dataDir.resolve("blocks.json"), sb.toString(), StandardCharsets.UTF_8);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }

        System.out.println("[BlockProbe] blocks -> " + blocksFile);
        System.out.println("[BlockProbe] DONE, stopping server");
        server.stop(false);
    }
}
