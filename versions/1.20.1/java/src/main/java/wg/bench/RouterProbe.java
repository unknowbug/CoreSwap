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
        int count = 16; // 临时固定（probe.count 读取异常）
        ServerWorld world = server.getOverworld();
        ServerChunkManager cm = world.getChunkManager();
        NoiseConfig noiseConfig = cm.getNoiseConfig();
        NoiseRouter router = noiseConfig.getNoiseRouter();
        long seed = world.getSeed();

        // 采样点：列模式（x=0, z=0, y=0..count*4）——下界 b3d 列对比（C++ densityDump 同列）
        double[] xs = new double[count], ys = new double[count], zs = new double[count];
        for (int i = 0; i < count; i++) {
            xs[i] = 0.0;
            zs[i] = 0.0;
            ys[i] = i * 4;
        }

        String[] names = {
                "barrier", "temperature", "vegetation", "continents", "erosion", "depth",
                "ridges", "initial_density", "final_density", "vein_toggle", "vein_ridged", "vein_gap"
        };
        net.minecraft.world.gen.densityfunction.DensityFunction[] fns = {
                router.barrierNoise(), router.temperature(), router.vegetation(), router.continents(),
                router.erosion(), router.depth(), router.ridges(), router.initialDensityWithoutJaggedness(),
                router.finalDensity(), router.veinToggle(), router.veinRidged(), router.veinGap()
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
        for (int i = 0; i < count; i++) {
            pos.x = (int) xs[i];
            pos.y = (int) ys[i];
            pos.z = (int) zs[i];
            sb.append(String.format(Locale.ROOT, "P %d %d %d", pos.x, pos.y, pos.z)).append('\n');
            for (int f = 0; f < names.length; f++) {
                double v = fns[f].sample(pos);
                sb.append(String.format(Locale.ROOT, "%s %.17g", names[f], v)).append('\n');
            }
            sb.append(String.format(Locale.ROOT, "base_3d_noise %.17g%n", b3d.sample(pos)));
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
}
