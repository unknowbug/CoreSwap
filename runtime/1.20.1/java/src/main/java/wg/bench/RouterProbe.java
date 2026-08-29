package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.world.gen.noise.NoiseConfig;
import net.minecraft.world.gen.noise.NoiseRouter;

import java.util.Locale;

/**
 * Router 分量探针：输出 noiseRouter 各分量在指定采样点的值，供 C++ 逐分量对比。
 * 用法：-Dprobe.count=<int>（采样点固定网格）
 */
public class RouterProbe {
    public static void run(MinecraftServer server) {
        int count = System.getProperty("router.yFrom") != null
                ? (int)((Double.parseDouble(System.getProperty("router.yTo", "100"))
                        - Double.parseDouble(System.getProperty("router.yFrom")))
                        / Double.parseDouble(System.getProperty("router.yStep", "2")) + 1)
                : 16;
        ServerWorld world = server.getOverworld();
        ServerChunkManager cm = world.getChunkManager();
        NoiseConfig noiseConfig = cm.getNoiseConfig();
        NoiseRouter router = noiseConfig.getNoiseRouter();
        long seed = world.getSeed();

        // terracotta 带数组导出（对比 C++ SurfaceBuilder 192 带）
        try {
            Object sb = noiseConfig.getSurfaceBuilder();
            java.lang.reflect.Field fBands = sb.getClass().getDeclaredField("terracottaBands");
            fBands.setAccessible(true);
            Object bands = fBands.get(sb);
            StringBuilder bsb = new StringBuilder("TBANDS");
            for (int bi = 0; bi < java.lang.reflect.Array.getLength(bands); bi++) {
                Object bs = java.lang.reflect.Array.get(bands, bi);
                int bid = net.minecraft.registry.Registries.BLOCK.getRawId(
                        ((net.minecraft.block.BlockState) bs).getBlock());
                bsb.append(' ').append(bid);
            }
            System.out.println(bsb);
            System.out.println("TBANDS_COUNT=" + java.lang.reflect.Array.getLength(bands));
        } catch (Throwable ex) {
            System.out.println("[RouterProbe] tbands dump failed: " + ex);
        }

        // 采样点：列模式（x=0, z=0, y=0..count*4）——下界 b3d 列对比（C++ densityDump 同列）
        double[] xs = new double[count], ys = new double[count], zs = new double[count];
        for (int i = 0; i < count; i++) {
            xs[i] = System.getProperty("router.x") != null ? Double.parseDouble(System.getProperty("router.x")) : 0.0;
            zs[i] = System.getProperty("router.z") != null ? Double.parseDouble(System.getProperty("router.z")) : 0.0;
            ys[i] = System.getProperty("router.y") != null
                    ? Double.parseDouble(System.getProperty("router.y"))
                    : (System.getProperty("router.yFrom") != null
                        ? Double.parseDouble(System.getProperty("router.yFrom")) + i * Double.parseDouble(System.getProperty("router.yStep", "2"))
                        : i * 4);
        }

        String[] names = {
                "barrier", "temperature", "vegetation", "continents", "erosion", "depth",
                "ridges", "initial_density", "final_density", "vein_toggle", "vein_ridged", "vein_gap",
                "fluid_level_floodedness"
        };
        net.minecraft.world.gen.densityfunction.DensityFunction[] fns = {
                router.barrierNoise(), router.temperature(), router.vegetation(), router.continents(),
                router.erosion(), router.depth(), router.ridges(), router.initialDensityWithoutJaggedness(),
                router.finalDensity(), router.veinToggle(), router.veinRidged(), router.veinGap(),
                router.fluidLevelFloodednessNoise()
        };
        // 用 server 的 ChunkNoiseSampler 派生 NoisePos？直接构造最小 NoisePos
        SimplePos pos = new SimplePos();

        // base_3d_noise：NoiseConfig 的 randomDeriver（游戏实际）反射 + 下界参数
        java.lang.reflect.Field rdField2;
        net.minecraft.util.math.random.RandomSplitter rd2;
        try {
            rdField2 = NoiseConfig.class.getDeclaredField("randomDeriver");
            rdField2.setAccessible(true);
            rd2 = (net.minecraft.util.math.random.RandomSplitter) rdField2.get(noiseConfig);
        } catch (Exception ex) {
            throw new RuntimeException("cannot get randomDeriver", ex);
        }
        String dim = System.getProperty("router.dim", "nether");
        double ys_ = dim.equals("overworld") ? 0.125 : 0.375;
        double yf = dim.equals("overworld") ? 160.0 : 60.0;
        var b3d = new net.minecraft.util.math.noise.InterpolatedNoiseSampler(
                rd2.split(new net.minecraft.util.Identifier("terrain")),
                0.25, ys_, 80.0, yf, 8.0);

        StringBuilder sb = new StringBuilder();
        sb.append("#seed ").append(seed).append('\n');
        // continentalness/offset 噪声直接采样（对比 C++ continents 树内部）
        try {
            java.lang.reflect.Method mSampler = net.minecraft.world.gen.noise.NoiseConfig.class.getDeclaredMethod("getOrCreateSampler", net.minecraft.registry.RegistryKey.class);
            mSampler.setAccessible(true);
            var continentalness = (net.minecraft.util.math.noise.DoublePerlinNoiseSampler) mSampler.invoke(noiseConfig, net.minecraft.world.gen.noise.NoiseParametersKeys.CONTINENTALNESS);
            var offsetNoise = (net.minecraft.util.math.noise.DoublePerlinNoiseSampler) mSampler.invoke(noiseConfig, net.minecraft.world.gen.noise.NoiseParametersKeys.OFFSET);
            sb.append(String.format(Locale.ROOT, "continentalness_noise %.17g %.17g %n", continentalness.sample(800.0, 0.0, 804.0), continentalness.sample(800.0, 0.0, 804.0)));
            sb.append(String.format(Locale.ROOT, "continentalness_tree %.17g %n", continentalness.sample(798.354203, 0.0, 805.729138)));
            sb.append(String.format(Locale.ROOT, "offset_noise %.17g %n", offsetNoise.sample(200.0, 0.0, 201.0)));
            sb.append(String.format(Locale.ROOT, "offset_tree_shiftx %.17g %n", offsetNoise.sample(800.0, 0.0, 804.0) * 4.0));
            sb.append(String.format(Locale.ROOT, "offset_tree_shiftz %.17g %n", offsetNoise.sample(804.0, 800.0, 0.0) * 4.0));
        } catch (Throwable ex) {
            sb.append("noise sampler ERR ").append(ex).append('\n');
        }
        for (int i = 0; i < count; i++) {
            pos.x = (int) xs[i];
            pos.y = (int) ys[i];
            pos.z = (int) zs[i];
            sb.append(String.format(Locale.ROOT, "P %d %d %d", pos.x, pos.y, pos.z)).append('\n');
            for (int f = 0; f < names.length; f++) {
                if (fns[f] == null) continue;
                double v = fns[f].sample(pos);
                sb.append(String.format(Locale.ROOT, "%s %.17g", names[f], v)).append('\n');
            }
            // ESH：模拟 cns.estimateSurfaceHeight（initialDensityWithoutJaggedness > 0.390625 扫描，步长 8）
            if (System.getProperty("router.esh") != null) {
                var idwj2 = router.initialDensityWithoutJaggedness();
                for (int[] pt : new int[][]{{-244, -256}, {-260, -256}, {-248, -248}, {-244, -244},
                    {-256, -256}, {-240, -256}, {-256, -240}, {-240, -240},
                    {-252, -256}, {-244, -252}, {-236, -256}, {-244, -260}, {-248, -260}, {-240, -260}, {-252, -260}, {-244, -268}, {-236, -268}}) {
                    int est = Integer.MAX_VALUE;
                    for (int y = 320; y >= -64; y -= 8) {
                        if (idwj2.sample(new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(pt[0], y, pt[1])) > 0.390625) { est = y; break; }
                    }
                    sb.append(String.format(Locale.ROOT, "ESH %d %d est=%d%n", pt[0], pt[1], est));
                    for (int y : new int[]{64, 60, 56, 52, 48, 44, 40, 36, 32}) {
                        double v2 = idwj2.sample(new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(pt[0], y, pt[1]));
                        sb.append(String.format(Locale.ROOT, "ESH-ID %d %d y=%d %.6f%n", pt[0], pt[1], y, v2));
                    }
                }
            }
            if (System.getProperty("router.b3dDump") != null) {
                try {
                    dumpB3dInternal(b3d, pos);
                } catch (Throwable ex) {
                    System.out.println("b3dDump threw " + ex);
                }
            }
            sb.append(String.format(Locale.ROOT, "base_3d_noise %.17g %s%n", b3d.sample(pos), Double.toHexString(b3d.sample(pos))));
            // biome 采样（该点 6 维；采样位置 = floor(block/4)*4）
            {
                SimplePos bp = new SimplePos();
                bp.x = (pos.x >> 2) << 2;
                bp.y = (pos.y >> 2) << 2;
                bp.z = (pos.z >> 2) << 2;
                float t = (float) router.temperature().sample(bp);
                float hum = (float) router.vegetation().sample(bp);
                float cont = (float) router.continents().sample(bp);
                float ero = (float) router.erosion().sample(bp);
                float dep = (float) router.depth().sample(bp);
                float w = (float) router.ridges().sample(bp);
                sb.append(String.format(Locale.ROOT, "B %d %d %d %.6f %.6f %.6f %.6f %.6f %.6f%n",
                        bp.blockX(), bp.blockY(), bp.blockZ(), t, hum, cont, ero, dep, w));
                // 表面规则实际 biome：ChunkRegion.getBiomeAccess().getBiome(BlockPos)（8 邻域选点）等价复刻
                try {
                    net.minecraft.world.biome.source.BiomeSource bs2 =
                            ((net.minecraft.world.gen.chunk.NoiseChunkGenerator) server.getOverworld().getChunkManager().getChunkGenerator()).getBiomeSource();
                    net.minecraft.world.biome.source.BiomeAccess ba = new net.minecraft.world.biome.source.BiomeAccess(
                            (bx, by, bz) -> bs2.getBiome(bx, by, bz, noiseConfig.getMultiNoiseSampler()),
                            net.minecraft.world.biome.source.BiomeAccess.hashSeed(seed));
                    Object biomeEntry2 = ba.getBiome(new net.minecraft.util.math.BlockPos(pos.x, pos.y, pos.z));
                    String bid2 = ((net.minecraft.registry.entry.RegistryEntry<net.minecraft.world.biome.Biome>)biomeEntry2)
                            .getKey().map(k -> k.getValue().toString()).orElse("?");
                    sb.append(String.format(Locale.ROOT, "SURFBIOME %d %d %d %s%n", bp.blockX(), bp.blockY(), bp.blockZ(), bid2));
                } catch (Throwable ex) {
                    sb.append("SURFBIOME ERR ").append(ex).append('\n');
                }
                // 对照：无 8 邻域（直接 floor）biomeSource 采样
                try {
                    net.minecraft.world.gen.chunk.NoiseChunkGenerator gen =
                            (net.minecraft.world.gen.chunk.NoiseChunkGenerator) server.getOverworld().getChunkManager().getChunkGenerator();
                    net.minecraft.world.biome.source.BiomeSource bs = gen.getBiomeSource();
                    java.lang.reflect.Method mSampler = net.minecraft.world.biome.source.BiomeSource.class
                            .getMethod("getBiome", int.class, int.class, int.class, net.minecraft.world.biome.source.util.MultiNoiseUtil.MultiNoiseSampler.class);
                    Object biomeEntry = mSampler.invoke(bs, bp.blockX() >> 2, bp.blockY() >> 2, bp.blockZ() >> 2,
                            noiseConfig.getMultiNoiseSampler());
                    String bid = ((net.minecraft.registry.entry.RegistryEntry<net.minecraft.world.biome.Biome>)biomeEntry)
                            .getKey().map(k -> k.getValue().toString()).orElse("?");
                    sb.append(String.format(Locale.ROOT, "BIOME %d %d %d %s%n", bp.blockX(), bp.blockY(), bp.blockZ(), bid));
                } catch (Throwable ex) {
                    sb.append("BIOME ERR ").append(ex).append('\n');
                }
            }
        }
        System.out.println("===ROUTERPROBE_BEGIN===");
        System.out.println(sb);
        System.out.println("===ROUTERPROBE_END===");

        // density 纯采样计时：模拟 16 chunk 的 density 网格（4x4x8 = 12288 点）
        long t0 = System.nanoTime();
        double acc = 0;
        int n = 0;
        for (int c = 0; c < 16; c++) {
            int cx0 = 200 + c % 4, cz0 = 200 + c / 4;
            for (int y = 0; y < 48; y++) {
                for (int z = 0; z < 4; z++) {
                    for (int x = 0; x < 4; x++) {
                        pos.x = cx0 * 16 + x * 4;
                        pos.z = cz0 * 16 + z * 4;
                        pos.y = -64 + y * 8;
                        acc += router.finalDensity().sample(pos);
                        n++;
                    }
                }
            }
        }
        long t1 = System.nanoTime();
        System.out.println("===DENSITY_TIMING " + n + " points " + String.format(Locale.ROOT, "%.2f", (t1 - t0) / 1e6)
                + " ms, acc=" + acc + "===");
        server.stop(false);
    }

    /** 最小 NoisePos（与 WorldGenBench.SimpleNoisePos 相同） */
    static final class SimplePos implements net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos {
        int x, y, z;

        @Override
        public int blockX() { return x; }

        @Override
        public int blockY() { return y; }

        @Override
        public int blockZ() { return z; }

        @Override
        public net.minecraft.world.gen.chunk.Blender getBlender() {
            return net.minecraft.world.gen.chunk.Blender.getNoBlending();
        }
    }

    /** b3d 内部 dump（WG_B3DDUMP 对照）：反射 lower/upper/interpolation，手动复刻 sampleImpl 循环。 */
    private static void dumpB3dInternal(net.minecraft.util.math.noise.InterpolatedNoiseSampler b3d, net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos pos) throws Exception {
        net.minecraft.util.math.noise.OctavePerlinNoiseSampler lower = null, upper = null, interp = null;
        for (java.lang.reflect.Field f : net.minecraft.util.math.noise.InterpolatedNoiseSampler.class.getDeclaredFields()) {
            f.setAccessible(true);
            if (f.getType() == net.minecraft.util.math.noise.OctavePerlinNoiseSampler.class) {
                Object v = f.get(b3d);
                if (lower == null) lower = (net.minecraft.util.math.noise.OctavePerlinNoiseSampler) v;
                else if (upper == null) upper = (net.minecraft.util.math.noise.OctavePerlinNoiseSampler) v;
                else interp = (net.minecraft.util.math.noise.OctavePerlinNoiseSampler) v;
            }
        }
        if (lower == null || upper == null || interp == null) throw new IllegalStateException("cannot locate octave fields");
        double scaledXz = 684.412F * 0.25;  // 主世界参数（RouterProbe 构造）
        double scaledY = 684.412F * 0.125;
        double d = pos.blockX() * scaledXz, e = pos.blockY() * scaledY, f = pos.blockZ() * scaledXz;
        double g = d / 80.0, h = e / 160.0, i = f / 80.0;
        double j = scaledY * 8.0, k = j / 160.0;
        System.out.println("[J-B3D] pos=(" + pos.blockX() + "," + pos.blockY() + "," + pos.blockZ() + ") d=" + d + " e=" + e + " f=" + f + " g=" + g + " h=" + h + " i=" + i + " j=" + j + " k=" + k);
        double l = 0, m = 0, n = 0, o = 1.0;
        for (int p = 0; p < 8; p++) {
            net.minecraft.util.math.noise.PerlinNoiseSampler pn = interp.getOctave(p);
            if (pn != null) {
                if (p < 2) {
                    java.lang.reflect.Field fo = net.minecraft.util.math.noise.PerlinNoiseSampler.class.getDeclaredField("originX");
                    fo.setAccessible(true);
                    double ox = (double) fo.get(pn);
                    fo = net.minecraft.util.math.noise.PerlinNoiseSampler.class.getDeclaredField("originY");
                    fo.setAccessible(true);
                    double oy = (double) fo.get(pn);
                    fo = net.minecraft.util.math.noise.PerlinNoiseSampler.class.getDeclaredField("originZ");
                    fo.setAccessible(true);
                    double oz = (double) fo.get(pn);
                    double dd = g * o + ox, ee = h * o + oy, ff = i * o + oz;
                    int ii = (int) Math.floor(dd), jj = (int) Math.floor(ee), kk = (int) Math.floor(ff);
                    System.out.println("[J-P] interp oct=" + p + " origin=(" + ox + "," + oy + "," + oz + ") d=" + dd + " e=" + ee + " f=" + ff
                            + " i=" + ii + " j=" + jj + " k=" + kk + " g=" + (dd - ii) + " h=" + (ee - jj) + " l=" + (ff - kk));
                }
                double r0 = pn.sample(g * o, h * o, i * o, k * o, h * o);
                System.out.println("[J-B3D] interp oct=" + p + " res=" + r0 + " contrib=" + r0 / o);
                n += r0 / o;
            }
            o /= 2.0;
        }
        double q = (n / 10.0 + 1.0) / 2.0;
        boolean bl2 = q >= 1.0, bl3 = q <= 0.0;
        o = 1.0;
        for (int r = 0; r < 16; r++) {
            double s = net.minecraft.util.math.noise.OctavePerlinNoiseSampler.maintainPrecision(d * o);
            double t = net.minecraft.util.math.noise.OctavePerlinNoiseSampler.maintainPrecision(e * o);
            double u = net.minecraft.util.math.noise.OctavePerlinNoiseSampler.maintainPrecision(f * o);
            double v = j * o;
            if (!bl2) {
                net.minecraft.util.math.noise.PerlinNoiseSampler pn = lower.getOctave(r);
                if (pn != null) {
                    double r0 = pn.sample(s, t, u, v, e * o);
                    System.out.println("[J-B3D] lower oct=" + r + " s=" + s + " t=" + t + " u=" + u + " res=" + r0 + " contrib=" + r0 / o);
                    l += r0 / o;
                }
            }
            if (!bl3) {
                net.minecraft.util.math.noise.PerlinNoiseSampler pn = upper.getOctave(r);
                if (pn != null) {
                    double r0 = pn.sample(s, t, u, v, e * o);
                    System.out.println("[J-B3D] upper oct=" + r + " s=" + s + " t=" + t + " u=" + u + " res=" + r0 + " contrib=" + r0 / o);
                    m += r0 / o;
                }
            }
            o /= 2.0;
        }
        System.out.println("[J-B3D] l=" + l + " m=" + m + " q=" + q);
    }
}
