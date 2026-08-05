package wg.bench;

import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.world.gen.densityfunction.DensityFunction;
import net.minecraft.world.gen.densityfunction.DensityFunctionTypes;
import net.minecraft.world.gen.noise.NoiseConfig;
import net.minecraft.world.gen.noise.NoiseRouter;

import java.util.Locale;

/**
 * Ore 矿脉插值探针：对照 C++ ore_probe。
 * 复刻 ChunkNoiseSampler.DensityInterpolator 的 cell 角点采样 + 三线性插值。
 * 用法：-Dore.probe=1
 */
public class OreProbe {
    public static void run(MinecraftServer server) {
        ServerWorld world = server.getOverworld();
        ServerChunkManager cm = world.getChunkManager();
        NoiseConfig noiseConfig = cm.getNoiseConfig();
        NoiseRouter router = noiseConfig.getNoiseRouter();
        long seed = world.getSeed();

        DensityFunction vt = router.veinToggle();
        DensityFunction vr = router.veinRidged();
        DensityFunction vg = router.veinGap();

        // 解包 interpolated → argument（与 ChunkNoiseSampler.getActualDensityFunction 一致）
        DensityFunction vtArg = unwrap(vt);
        DensityFunction vrArg = unwrap(vr);
        DensityFunction vgArg = unwrap(vg);

        StringBuilder sb = new StringBuilder();
        sb.append("#seed ").append(seed).append('\n');

        // 与 C++ ore_probe 相同的采样网格：chunk(200,200) 4×4 列 × y 每 4
        SimplePos pos = new SimplePos();
        for (int col = 0; col < 4; col++) {
            for (int row = 0; row < 4; row++) {
                int bx = 200 * 16 + row * 4;
                int bz = 200 * 16 + col * 4;
                for (int by = -64; by <= 100; by += 4) {
                    double vtRaw = vt.sample(pos.set(bx, by, bz));
                    double vrRaw = vr.sample(pos.set(bx, by, bz));
                    double vgRaw = vg.sample(pos.set(bx, by, bz));
                    double vtInterp = lerp3Interp(vtArg, bx, by, bz);
                    double vrInterp = lerp3Interp(vrArg, bx, by, bz);
                    double vgInterp = lerp3Interp(vgArg, bx, by, bz);
                    sb.append(String.format(Locale.ROOT, "P %d %d %d vt=%.6f vr=%.6f vg=%.6f vtI=%.6f vrI=%.6f vgI=%.6f%n",
                            bx, by, bz, vtRaw, vrRaw, vgRaw, vtInterp, vrInterp, vgInterp));
                }
            }
        }

        // vanilla 矿脉坐标精确诊断：列 (3211,3204) 矿脉段 y=4..56
        sb.append("#mine column (3211,3204)\n");
        for (int by = -8; by <= 60; by++) {
            double vtV = vt.sample(pos.set(3211, by, 3204));
            double vrV = vr.sample(pos.set(3211, by, 3204));
            double vgV = vg.sample(pos.set(3211, by, 3204));
            double vtI = lerp3Interp(vtArg, 3211, by, 3204);
            double vrI = lerp3Interp(vrArg, 3211, by, 3204);
            double vgI = lerp3Interp(vgArg, 3211, by, 3204);
            sb.append(String.format(Locale.ROOT, "M %d %d %d vt=%.6f vr=%.6f vg=%.6f vtI=%.6f vrI=%.6f vgI=%.6f%n",
                    3211, by, 3204, vtV, vrV, vgV, vtI, vrI, vgI));
        }
        System.out.println("===OREPROBE_BEGIN===");
        System.out.print(sb);
        System.out.println("===OREPROBE_END===");
        server.stop(false);
    }

    private static DensityFunction unwrap(DensityFunction f) {
        // DensityFunctionTypes.Wrapping 是 protected 接口，反射调用 wrapped()
        try {
            for (Class<?> c : f.getClass().getInterfaces()) {
                if (c.getSimpleName().equals("Wrapping")) {
                    java.lang.reflect.Method m = c.getMethod("wrapped");
                    return (DensityFunction) m.invoke(f);
                }
            }
        } catch (Exception e) {
            throw new RuntimeException("unwrap failed: " + f.getClass(), e);
        }
        return f;
    }

    /** Java DensityInterpolator 的三线性插值（cell 角点采样 + lerp） */
    private static double lerp3Interp(DensityFunction arg, int bx, int by, int bz) {
        int minCellY = Math.floorDiv(-64, 8);
        int cx = Math.floorDiv(bx, 4);
        int cz = Math.floorDiv(bz, 4);
        int gy = Math.floorDiv(by, 8) - minCellY;
        double fx = (bx - cx * 4) / 4.0;
        double fy = (by - (gy + minCellY) * 8) / 8.0;
        double fz = (bz - cz * 4) / 4.0;
        SimplePos p = new SimplePos();
        double x0y0z0 = arg.sample(p.set(cx * 4, (gy + minCellY) * 8, cz * 4));
        double x1y0z0 = arg.sample(p.set((cx + 1) * 4, (gy + minCellY) * 8, cz * 4));
        double x0y1z0 = arg.sample(p.set(cx * 4, (gy + 1 + minCellY) * 8, cz * 4));
        double x1y1z0 = arg.sample(p.set((cx + 1) * 4, (gy + 1 + minCellY) * 8, cz * 4));
        double x0y0z1 = arg.sample(p.set(cx * 4, (gy + minCellY) * 8, (cz + 1) * 4));
        double x1y0z1 = arg.sample(p.set((cx + 1) * 4, (gy + minCellY) * 8, (cz + 1) * 4));
        double x0y1z1 = arg.sample(p.set(cx * 4, (gy + 1 + minCellY) * 8, (cz + 1) * 4));
        double x1y1z1 = arg.sample(p.set((cx + 1) * 4, (gy + 1 + minCellY) * 8, (cz + 1) * 4));
        return lerp3(fx, fy, fz, x0y0z0, x1y0z0, x0y1z0, x1y1z0, x0y0z1, x1y0z1, x0y1z1, x1y1z1);
    }

    private static double lerp(double delta, double start, double end) {
        return start + delta * (end - start);
    }

    private static double lerp3(double dX, double dY, double dZ,
                                double x0y0z0, double x1y0z0, double x0y1z0, double x1y1z0,
                                double x0y0z1, double x1y0z1, double x0y1z1, double x1y1z1) {
        double x0y0 = lerp(dX, x0y0z0, x1y0z0);
        double x1y0 = lerp(dX, x0y1z0, x1y1z0);
        double x0y1 = lerp(dX, x0y0z1, x1y0z1);
        double x1y1 = lerp(dX, x0y1z1, x1y1z1);
        double x0 = lerp(dY, x0y0, x1y0);
        double x1 = lerp(dY, x0y1, x1y1);
        return lerp(dZ, x0, x1);
    }

    static final class SimplePos implements DensityFunction.NoisePos {
        int x, y, z;

        SimplePos set(int x, int y, int z) { this.x = x; this.y = y; this.z = z; return this; }

        @Override public int blockX() { return x; }
        @Override public int blockY() { return y; }
        @Override public int blockZ() { return z; }
        @Override public net.minecraft.world.gen.chunk.Blender getBlender() {
            return net.minecraft.world.gen.chunk.Blender.getNoBlending();
        }
    }
}
