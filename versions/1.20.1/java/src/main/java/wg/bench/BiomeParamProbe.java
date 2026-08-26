package wg.bench;

import net.minecraft.registry.Registries;
import net.minecraft.registry.Registry;
import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.Identifier;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.biome.source.MultiNoiseBiomeSourceParameterList;
import net.minecraft.world.biome.source.util.MultiNoiseUtil;
import com.mojang.datafixers.util.Pair;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * 导出 vanilla multi_noise biome 参数表（1.20.1 硬编码在 VanillaBiomeParameters，非 JSON）：
 * 用运行时 preset "minecraft:overworld" 导出六维参数 → data/biome_params.json，供 C++ 加载。
 * 用法：-Dbiome.probe=1
 */
public class BiomeParamProbe {
    public static void run(MinecraftServer server) {
        Registry<MultiNoiseBiomeSourceParameterList> reg =
                server.getRegistryManager().get(RegistryKeys.MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST);
        // preset 可配（-Dbiome.preset=nether 导下界参数）；overworld 输出 biome_params.json，其他输出 biome_params_<preset>.json
        String presetName = System.getProperty("biome.preset", "overworld");
        String outName = presetName.equals("overworld") ? "biome_params.json" : "biome_params_" + presetName + ".json";
        MultiNoiseBiomeSourceParameterList preset = reg.get(
                RegistryKey.of(RegistryKeys.MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST, new Identifier(presetName)));
        if (preset == null) {
            System.err.println("[BiomeParamProbe] preset " + presetName + " not found");
            server.stop(false);
            return;
        }
        List<Pair<MultiNoiseUtil.NoiseHypercube, RegistryEntry<Biome>>> entries = preset.getEntries().getEntries();
        System.out.println("[BiomeParamProbe] biomes=" + entries.size());

        StringBuilder sb = new StringBuilder("[\n");
        for (int i = 0; i < entries.size(); i++) {
            Pair<MultiNoiseUtil.NoiseHypercube, RegistryEntry<Biome>> p = entries.get(i);
            MultiNoiseUtil.NoiseHypercube c = p.getFirst();
            String id = p.getSecond().getKey().map(k -> k.getValue().toString()).orElse("?");
            double temp = p.getSecond().value().getTemperature();
            sb.append("  {\"biome\":\"").append(id).append("\",\"temperature\":").append(temp).append(",\"parameters\":{");
            sb.append("\"temperature\":[").append(MultiNoiseUtil.toFloat(c.temperature().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.temperature().max())).append("],");
            sb.append("\"humidity\":[").append(MultiNoiseUtil.toFloat(c.humidity().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.humidity().max())).append("],");
            sb.append("\"continentalness\":[").append(MultiNoiseUtil.toFloat(c.continentalness().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.continentalness().max())).append("],");
            sb.append("\"erosion\":[").append(MultiNoiseUtil.toFloat(c.erosion().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.erosion().max())).append("],");
            sb.append("\"depth\":[").append(MultiNoiseUtil.toFloat(c.depth().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.depth().max())).append("],");
            sb.append("\"weirdness\":[").append(MultiNoiseUtil.toFloat(c.weirdness().min())).append(",")
              .append(MultiNoiseUtil.toFloat(c.weirdness().max())).append("],");
            sb.append("\"offset\":").append(MultiNoiseUtil.toFloat(c.offset()));
            sb.append("}}");
            if (i < entries.size() - 1) sb.append(",");
            sb.append("\n");
        }
        sb.append("]\n");
        try {
            Path out = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
            Files.createDirectories(out);
            Files.writeString(out.resolve(outName), sb.toString(), StandardCharsets.UTF_8);
            System.out.println("[BiomeParamProbe] " + outName + " -> " + out.resolve(outName));
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        System.out.println("[BiomeParamProbe] DONE, stopping server");
        server.stop(false);
    }
}
