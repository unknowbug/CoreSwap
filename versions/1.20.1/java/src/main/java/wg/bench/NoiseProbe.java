package wg.bench;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.noise.DoublePerlinNoiseSampler;
import net.minecraft.world.gen.noise.NoiseConfig;

import java.util.List;

/**
 * 3a 噪声探针：与 C++ noise_probe 输出同格式数据用于逐点对比。
 * 用法：-Dprobe.count=<int>，对一组 noise key 采样。
 */
public class NoiseProbe {
    private static final List<String> KEYS = List.of(
            "temperature", "vegetation", "continentalness", "erosion", "ridge", "offset",
            "aquifer_barrier", "aquifer_fluid_level_floodedness", "aquifer_lava", "aquifer_fluid_level_spread",
            "pillar", "spaghetti_2d", "spaghetti_2d_elevation", "spaghetti_2d_modulator", "spaghetti_2d_thickness",
            "spaghetti_3d_1", "spaghetti_3d_2", "spaghetti_3d_rarity", "spaghetti_3d_thickness",
            "spaghetti_roughness", "spaghetti_roughness_modulator",
            "cave_entrance", "cave_layer", "cave_cheese",
            "ore_veininess", "ore_vein_a", "ore_vein_b", "ore_gap",
            "noodle", "noodle_thickness", "noodle_ridge_a", "noodle_ridge_b",
            "jagged", "surface", "surface_secondary", "clay_bands_offset",
            "badlands_pillar", "badlands_pillar_roof", "badlands_surface",
            "iceberg_pillar", "iceberg_pillar_roof", "iceberg_surface",
            "surface_swamp", "calcite", "gravel", "powder_snow", "packed_ice", "ice",
            "soul_sand_layer", "gravel_layer", "patch", "netherrack", "nether_wart", "nether_state_selector");

    public static void run(MinecraftServer server) {
        int count = Integer.parseInt(System.getProperty("probe.count", "64"));
        NoiseConfig noiseConfig = server.getOverworld().getChunkManager().getNoiseConfig();
        System.out.println("===WORLDSEED " + server.getOverworld().getSeed() + "===");
        StringBuilder splitterDbg = new StringBuilder();
        new net.minecraft.util.math.random.Xoroshiro128PlusPlusRandom(server.getOverworld().getSeed())
                .nextSplitter().addDebugInfo(splitterDbg);
        System.out.println("===DEBUG_SPLITTER " + splitterDbg + "===");

        // 采样点与 C++ noise_probe 完全一致
        double[] xs = new double[count], ys = new double[count], zs = new double[count];
        for (int i = 0; i < count; i++) {
            xs[i] = (i * 37) % 128;
            zs[i] = (i * 73) % 128;
            ys[i] = -64 + (i * 29) % 384;
        }

        StringBuilder sb = new StringBuilder();
        for (String key : KEYS) {
            RegistryKey<DoublePerlinNoiseSampler.NoiseParameters> k =
                    RegistryKey.of(RegistryKeys.NOISE_PARAMETERS, new Identifier(key));
            DoublePerlinNoiseSampler sampler = noiseConfig.getOrCreateSampler(k);
            if (key.equals("temperature")) {
                StringBuilder dbg = new StringBuilder();
                sampler.addDebugInfo(dbg);
                System.out.println("===DEBUG_TEMP " + dbg + "===");
            }
            for (int i = 0; i < count; i++) {
                double v = sampler.sample(xs[i], ys[i], zs[i]);
                sb.append("minecraft:").append(key).append(' ')
                  .append((int) xs[i]).append(' ').append((int) ys[i]).append(' ').append((int) zs[i])
                  .append(' ').append(String.format(java.util.Locale.ROOT, "%.17g", v)).append('\n');
            }
        }
        System.out.println("===NOISEPROBE_BEGIN===");
        System.out.print(sb);
        System.out.println("===NOISEPROBE_END===");
        server.stop(false);
    }
}
