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
            if (cns == null) { System.out.println("[DensityProbe] cns null at NOISE stage"); server.stop(false); return; }
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

            // base_3d_noise 分量：优先从 cns.actualDensityFunctionCache 拿 vanilla 实际函数（rd 构造易错）
            DensityFunction b3d = null;
            try {
                Field fc2 = cns.getClass().getDeclaredField("actualDensityFunctionCache");
                fc2.setAccessible(true);
                Object cache2 = fc2.get(cns);
                if (cache2 instanceof java.util.Map<?, ?>) {
                    for (Object k : ((java.util.Map<?, ?>) cache2).keySet()) {
                        if (k.toString().contains("base_3d_noise")) {
                            Object v = ((java.util.Map<?, ?>) cache2).get(k);
                            if (v instanceof DensityFunction) { b3d = (DensityFunction) v; break; }
                        }
                    }
                }
            } catch (Exception ignored) {
            }
            StringBuilder sb2 = new StringBuilder();
            // 分量 dump（负坐标 spline 定位）：router.depth/continents/erosion vs C++ WG_SURFDUMP
            try {
                StringBuilder sbC = new StringBuilder();
                for (String comp : new String[]{"depth", "continents", "erosion", "shiftX", "shiftZ"}) {
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
