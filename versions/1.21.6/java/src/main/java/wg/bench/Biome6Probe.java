package wg.bench;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.Identifier;
import net.minecraft.world.gen.densityfunction.DensityFunction;
import net.minecraft.world.gen.noise.NoiseConfig;

/**
 * BIOME6 探针：nether router 六维直采 + ShiftedNoise 消融（字段遍历，名字无关）。
 * 触发：-Pbiome6=true（BenchMod SERVER_STARTED 时调用）。
 * 校准模式：-Pbiome6cal=true（CheckedRandom(0)+createLegacy(-7,[1,1]) 三段对拍输出）。
 */
public class Biome6Probe {
    public static void run(MinecraftServer server) {
        var netherKey = RegistryKey.of(RegistryKeys.WORLD, Identifier.of("the_nether"));
        // 260904-13：-Dbiome6.dim=overworld 支持（残 9 H2 Java 臂；默认 nether 保持旧行为）
        String dim6 = System.getProperty("biome6.dim", "nether");
        var world = dim6.equals("overworld") ? server.getOverworld() : server.getWorld(netherKey);
        if (world == null) { System.out.println("[BIOME6] no world dim=" + dim6); return; }
        NoiseConfig nc = world.getChunkManager().getNoiseConfig();
        var router = nc.getNoiseRouter();
        int[][] pts = {
            {5, 1, 0}, {12, 1, 0}, {10, 1, 1}, {14, 1, 2}, {7, 1, 3},
            {11, 1, 3}, {5, 1, 4}, {8, 1, 4}, {2, 1, 5}, {6, 1, 5}
        };
        // V5 残差对拍：-Dbiome6.points="x,y,z;x,y,z;..." 覆盖默认点位（主会话下发）
        String extra = System.getProperty("biome6.points", "");
        if (!extra.isEmpty()) {
            java.util.List<int[]> lp = new java.util.ArrayList<>();
            for (String s : extra.split(";")) {
                String[] p = s.split(",");
                if (p.length == 3) lp.add(new int[]{Integer.parseInt(p[0].trim()), Integer.parseInt(p[1].trim()), Integer.parseInt(p[2].trim())});
            }
            pts = lp.toArray(new int[0][]);
        }
        // 260904-13 残 9 H2 决胜：-Dbiome6.mnDump="qx,qy,qz" 直读 MultiNoiseSampler.sample 的 NoiseValuePoint 6 维
        //（生产语义：fillFromBiome 传 QuartPos.fromBlock 坐标；与 Rust/C++ NoisePos{px<<2} 对应关系由调用方核对）
        String mnDump = System.getProperty("biome6.mnDump", "");
        if (!mnDump.isEmpty()) {
            try {
                String[] mp = mnDump.split(",");
                int qx = Integer.parseInt(mp[0].trim()), qy = Integer.parseInt(mp[1].trim()), qz = Integer.parseInt(mp[2].trim());
                var nc2 = world.getChunkManager().getNoiseConfig();
                var sampler = nc2.getMultiNoiseSampler();
                var mSample = sampler.getClass().getMethod("sample", int.class, int.class, int.class);
                Object nvp = mSample.invoke(sampler, qx, qy, qz);
                Class<?> nvpCls = nvp.getClass();
                StringBuilder mnDbg = new StringBuilder("[MNDUMP] nvp class=" + nvpCls.getName() + " methods:");
                for (var mm : nvpCls.getMethods()) {
                    if (mm.getParameterCount() == 0 && mm.getReturnType() != void.class) {
                        mnDbg.append(' ').append(mm.getName()).append(':').append(mm.getReturnType().getSimpleName());
                    }
                }
                System.out.println(mnDbg);
                // 260904-13：NoiseValuePoint 实际为 long 定点域（temperatureNoise() 等）——dump 原始 long + toFloat 换算
                java.util.function.Function<String, Long> gl = name -> {
                    try { return (Long) nvpCls.getMethod(name).invoke(nvp); }
                    catch (Exception e) { throw new RuntimeException(e); }
                };
                Class<?> mnuCls = Class.forName("net.minecraft.world.biome.source.util.MultiNoiseUtil");
                var mToF = java.util.Arrays.stream(mnuCls.getMethods())
                        .filter(mm -> mm.getParameterCount() == 1 && mm.getParameterTypes()[0] == long.class
                                && mm.getReturnType() == float.class).findFirst().orElseThrow();
                java.util.function.Function<Long, Double> toF = l -> {
                    try { return (double) (float) mToF.invoke(null, l); }
                    catch (Exception e2) { throw new RuntimeException(e2); }
                };
                long tL = gl.apply("temperatureNoise"), hL = gl.apply("humidityNoise"), cL = gl.apply("continentalnessNoise");
                long eL = gl.apply("erosionNoise"), dL = gl.apply("depth"), wL = gl.apply("weirdnessNoise");
                System.out.println("[MNDUMP] raw longs: t=" + tL + " h=" + hL + " c=" + cL + " e=" + eL + " d=" + dL + " w=" + wL
                        + " (toFloat=" + mToF.getName() + ")");
                System.out.printf(java.util.Locale.ROOT,
                        "[MNDUMP] (%d,%d,%d) t=%.9f h=%.9f c=%.9f e=%.9f d=%.9f w=%.9f%n",
                        qx, qy, qz, toF.apply(tL), toF.apply(hL), toF.apply(cL), toF.apply(eL), toF.apply(dL), toF.apply(wL));
                // 同点距离核算（biome_params.json 两行：t 边界 0.2 / depth [0,0]）
                double t = toF.apply(tL), d = toF.apply(dL);
                // 同点距离核算（biome_params.json 两行：t 边界 0.2 / depth [0,0]）
                double tV = toF.apply(tL), dV = toF.apply(dL);
                double docean = Math.pow(Math.max(0.0, Math.max(tV - 0.2, -0.15 - tV)), 2) + dV * dV;
                double dluk = Math.pow(Math.max(0.0, Math.max(tV - 0.55, 0.2 - tV)), 2) + dV * dV;
                System.out.printf(java.util.Locale.ROOT, "[MNDUMP] dist deep_ocean=%.12e deep_lukewarm=%.12e%n", docean, dluk);
            } catch (Throwable t2) { System.out.println("[MNDUMP] failed: " + t2); }
        }
        // V5 裁决：-Dbiome6.cellDump="cx,cz[,cy]" 直读存档原始 biome cell（getBiomeForNoiseGen，无平滑）
        String cellDump = System.getProperty("biome6.cellDump", "");
        if (!cellDump.isEmpty()) {
            String[] cp = cellDump.split(",");
            int ccx = Integer.parseInt(cp[0].trim()), ccz = Integer.parseInt(cp[1].trim());
            int cellY = cp.length > 2 ? Integer.parseInt(cp[2].trim()) : 0;
            var chk = world.getChunk(ccx, ccz);
            System.out.println("[CELLDUMP] chunk(" + ccx + "," + ccz + ") cellY=" + cellY);
            for (int qz = 0; qz < 16; qz += 4) {
                StringBuilder row = new StringBuilder("[CELLDUMP] ");
                for (int qx = 0; qx < 16; qx += 4) {
                    var b = chk.getBiomeForNoiseGen(((ccx << 4) + qx) >> 2, cellY, ((ccz << 4) + qz) >> 2);
                    String n = b.getKey().map(k -> k.getValue().getPath()).orElse("?");
                    row.append(qx).append(',').append(qz).append('=').append(n.replace("minecraft:", "")).append(" | ");
                }
                System.out.println(row);
            }
        }
        // V5 裁决 2：-Dbiome6.colDump="x,z" 整列对比 storage(getBiomeForNoiseGen) vs BiomeAccess(world.getBiome)
        String colDump = System.getProperty("biome6.colDump", "");
        if (!colDump.isEmpty()) {
            String[] cp = colDump.split(",");
            int bx = Integer.parseInt(cp[0].trim()), bz = Integer.parseInt(cp[1].trim());
            var cpos = new net.minecraft.util.math.BlockPos.Mutable();
            System.out.println("[COLDUMP] block=(" + bx + "," + bz + ") cellY: storage | biomeAccess");
            for (int cy = 0; cy < 64; cy++) {
                var b1 = world.getChunk(bx >> 4, bz >> 4).getBiomeForNoiseGen(bx >> 2, cy, bz >> 2);
                var s1 = b1.getKey().map(k -> k.getValue().getPath()).orElse("?");
                var b2 = world.getBiome(cpos.set(bx, cy * 4 + 1, bz));
                var s2 = b2.getKey().map(k -> k.getValue().getPath()).orElse("?");
                if (!s1.equals(s2) || cy < 2)
                    System.out.println("[COLDUMP] y=" + (cy * 4 + 1) + ": " + s1.replace("minecraft:", "") + " | " + s2.replace("minecraft:", ""));
            }
            System.out.println("[COLDUMP] done（只打印 storage≠biomeAccess 的层 + 前 2 层）");
        }
        System.out.println("[BIOME6] nether router 6-dim @ mismatch pts:");
        System.out.println("[BIOME6] v2 (reflection ablation ON)");
        for (int[] p : pts) {
            var pos = new DensityFunction.UnblendedNoisePos(p[0], p[1], p[2]);
            double t = router.temperature().sample(pos);
            double h = router.vegetation().sample(pos);
            double c = router.continents().sample(pos);
            double e = router.erosion().sample(pos);
            double d = router.depth().sample(pos);
            double w = router.ridges().sample(pos);
            System.out.println(java.lang.String.format(
                java.util.Locale.ROOT, "[BIOME6] (%d,%d,%d) t=%.4f h=%.4f c=%.4f e=%.4f d=%.4f w=%.4f",
                p[0], p[1], p[2], t, h, c, e, d, w));
        }

        // ===== finalDensity 树结构 dump（限深 2）=====
        try {
            dumpDf("fd", router.finalDensity(), 0, 10);
        } catch (Throwable t) { System.out.println("[FD] failed: " + t); }
        // ===== blended 列对拍：InterpolatedNoiseSampler(LocalRandom(worldSeed)) @ (8,y,8) =====
        try {
            long wsb = world.getSeed();
            var bn = new net.minecraft.util.math.noise.InterpolatedNoiseSampler(
                    new net.minecraft.util.math.random.LocalRandom(wsb), 0.25, 0.375, 80.0, 60.0, 8.0);
            for (int y = 0; y < 128; y += 8) {
                double v = bn.sample(new DensityFunction.UnblendedNoisePos(8, y, 8));
                System.out.println(java.lang.String.format(java.util.Locale.ROOT, "[BLEND] y=%d %.6f", y, v));
            }
        } catch (Throwable t) { System.out.println("[BLEND] failed: " + t); }
        // ===== split 链对拍：LocalRandom(worldSeed).nextSplitter().split("minecraft:temperature") =====
        long ws = world.getSeed();
        var lr = new net.minecraft.util.math.random.LocalRandom(ws);
        var sp = lr.nextSplitter();
        long spSeed = -1;
        try {
            var fs = sp.getClass().getDeclaredField("seed");
            fs.setAccessible(true);
            spSeed = fs.getLong(sp);
        } catch (Throwable ignored) {}
        System.out.println("[SPLIT] worldSeed=" + ws + " splitterSeed=" + spSeed);
        var rt = (net.minecraft.util.math.random.CheckedRandom) sp.split(Identifier.of("minecraft:temperature"));
        StringBuilder sbt = new StringBuilder("[SPLIT] temp stream:");
        for (int i = 0; i < 4; i++) sbt.append(' ').append(rt.next(32));
        System.out.println(sbt);
        // ===== vegetation 递归 dump（同 temp 的 dumpDf）=====
        try {
            dumpDf("veg", router.vegetation(), 0, 3);
        } catch (Throwable t) { System.out.println("[ABL-VEG] failed: " + t); }
        // ===== 消融：递归 dump density 树 + 分量采样 =====
        try {
            var temp = router.temperature();
            dumpDf("temp", temp, 0, 3);
        } catch (Throwable t) {
            System.out.println("[ABL] failed: " + t);
        }
    }

    /** 递归 dump：类名/字段 + DF 样对象采样 */
    private static void dumpDf(String path, Object obj, int depth, int maxDepth) {
        if (obj == null || depth > maxDepth) return;
        var cls = obj.getClass();
        java.lang.reflect.Method ms = null;
        for (var mm : cls.getMethods()) {
            if (mm.getName().equals("sample") && mm.getParameterCount() == 1
                    && mm.getParameterTypes()[0].getName().endsWith("NoisePos")) { ms = mm; break; }
        }
        if (ms != null) {
            try {
                double at12 = (double) ms.invoke(obj, new DensityFunction.UnblendedNoisePos(12, 1, 0));
                double at3 = (double) ms.invoke(obj, new DensityFunction.UnblendedNoisePos(3, 0, 0));
                System.out.println(java.lang.String.format(java.util.Locale.ROOT,
                        "[ABL] %s (%s): f(12,1,0)=%.6f f(3,0,0)=%.6f", path, cls.getSimpleName(), at12, at3));
            } catch (Exception e) { System.out.println("[ABL] " + path + ": sample threw " + e); }
        } else {
            System.out.println("[ABL] " + path + " (" + cls.getName() + "): no NoisePos sample");
        }
        if (cls.getName().endsWith("DensityFunction$Noise")) {
            try {
                var fn = cls.getDeclaredField("noise");
                fn.setAccessible(true);
                Object dpn = fn.get(obj);
                var dpCls = dpn.getClass();
                var m3 = dpCls.getMethod("sample", double.class, double.class, double.class);
                double v300 = (double) m3.invoke(dpn, 3.0, 0.0, 0.0);
                double v3025 = (double) m3.invoke(dpn, 3.0, 0.25, 0.0);
                var fpa = dpCls.getDeclaredField("amplitude");
                fpa.setAccessible(true);
                var ffs = dpCls.getDeclaredField("firstSampler");
                ffs.setAccessible(true);
                Object octFirst = ffs.get(dpn);
                var fOct = octFirst.getClass().getDeclaredField("octaveSamplers");
                fOct.setAccessible(true);
                Object[] samplers = (Object[]) fOct.get(octFirst);
                for (int i = 0; i < samplers.length; i++) {
                    if (samplers[i] == null) { System.out.println("[ABL-OCT] " + path + " oct" + i + ": <null>"); continue; }
                    var fx = samplers[i].getClass().getDeclaredField("originX");
                    var fy = samplers[i].getClass().getDeclaredField("originY");
                    var fz = samplers[i].getClass().getDeclaredField("originZ");
                    fx.setAccessible(true); fy.setAccessible(true); fz.setAccessible(true);
                    System.out.println(java.lang.String.format(java.util.Locale.ROOT,
                            "[ABL-OCT] %s oct%d: origin=(%.6f,%.6f,%.6f)", path, i,
                            fx.getDouble(samplers[i]), fy.getDouble(samplers[i]), fz.getDouble(samplers[i])));
                }
            } catch (Throwable t) { System.out.println("[ABL-NOISE] " + path + ": " + t); }
        }
        if (depth < maxDepth) {
            for (var ff : cls.getDeclaredFields()) {
                if (java.lang.reflect.Modifier.isStatic(ff.getModifiers())) continue;
                ff.setAccessible(true);
                try {
                    Object v = ff.get(obj);
                    if (v == null) continue;
                    String tn = v.getClass().getName();
                    if (tn.startsWith("java.") || tn.startsWith("com.mojang.") || tn.startsWith("net.minecraft.util.dynamic")
                            || tn.contains("$Type") || tn.contains("Codec")) continue;
                    dumpDf(path + "." + ff.getName(), v, depth + 1, maxDepth);
                } catch (Exception ignored) {}
            }
        }
    }

    /** 对拍校准：CheckedRandom(0)+createLegacy(-7,[1,1]) 三段输出（S1 LCG / S2 origin / S3 采样） */
    public static void calibrate() {
        // S1: LCG 裸输出
        var r = new net.minecraft.util.math.random.CheckedRandom(0L);
        StringBuilder s1 = new StringBuilder("[CAL-S1] ");
        for (int i = 0; i < 8; i++) s1.append(r.next(32)).append(' ');
        System.out.println(s1);
        var r2 = new net.minecraft.util.math.random.CheckedRandom(0L);
        StringBuilder s1l = new StringBuilder("[CAL-S1L] ");
        for (int i = 0; i < 4; i++) s1l.append(r2.nextLong()).append(' ');
        System.out.println(s1l);
        var r3 = new net.minecraft.util.math.random.CheckedRandom(0L);
        StringBuilder s1d = new StringBuilder("[CAL-S1D] ");
        for (int i = 0; i < 3; i++) {
            int i26 = r3.next(26); int i27 = r3.next(27);
            long l = ((long) i26 << 27) + i27;
            s1d.append(String.format(java.util.Locale.ROOT, "%.10f ", l * 1.110223E-16F));
        }
        System.out.println(s1d);
        // S2: Octave createLegacy 构造产物（origin 反射）
        var random = new net.minecraft.util.math.random.CheckedRandom(0L);
        var oct = net.minecraft.util.math.noise.OctavePerlinNoiseSampler.createLegacy(
                random, -7, it.unimi.dsi.fastutil.doubles.DoubleList.of(1.0, 1.0));
        try {
            java.lang.reflect.Field fOct = net.minecraft.util.math.noise.OctavePerlinNoiseSampler.class.getDeclaredField("octaveSamplers");
            fOct.setAccessible(true);
            Object[] samplers = (Object[]) fOct.get(oct);
            for (int i = 0; i < samplers.length; i++) {
                if (samplers[i] == null) { System.out.println("[CAL-S2] octave" + i + ": <null>"); continue; }
                java.lang.reflect.Field fx = samplers[i].getClass().getDeclaredField("originX");
                java.lang.reflect.Field fy = samplers[i].getClass().getDeclaredField("originY");
                java.lang.reflect.Field fz = samplers[i].getClass().getDeclaredField("originZ");
                fx.setAccessible(true); fy.setAccessible(true); fz.setAccessible(true);
                System.out.println(java.lang.String.format(java.util.Locale.ROOT,
                        "[CAL-S2] octave%d: origin=(%.6f,%.6f,%.6f)", i, fx.getDouble(samplers[i]), fy.getDouble(samplers[i]), fz.getDouble(samplers[i])));
            }
        } catch (Throwable t) { System.out.println("[CAL-S2] reflection failed: " + t); }
        // S3: DoublePerlin legacy sample
        var dp = net.minecraft.util.math.noise.DoublePerlinNoiseSampler.createLegacy(
                new net.minecraft.util.math.random.CheckedRandom(0L),
                new net.minecraft.util.math.noise.DoublePerlinNoiseSampler.NoiseParameters(-7, 1.0, 1.0));
        int[][] pts = {
            {5, 1, 0}, {12, 1, 0}, {10, 1, 1}, {14, 1, 2}, {7, 1, 3},
            {11, 1, 3}, {5, 1, 4}, {8, 1, 4}, {2, 1, 5}, {6, 1, 5}
        };
        System.out.println("[CAL-S3] DoublePerlin legacy sample:");
        for (int[] p : pts) {
            double v = dp.sample(p[0] * 0.25, 0.0, p[2] * 0.25);
            System.out.println(java.lang.String.format(java.util.Locale.ROOT, "(%d,%d,%d) -> %.6f", p[0], p[1], p[2], v));
        }
    }
}




















