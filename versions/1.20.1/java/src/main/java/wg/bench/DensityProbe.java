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
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        System.out.println("[DensityProbe] DONE, stopping server");
        server.stop(false);
    }
}
