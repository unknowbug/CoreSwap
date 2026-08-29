package wg.bench;

import net.minecraft.registry.Registry;
import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.Identifier;

import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;

/**
 * NoiseParamProbe：导出 noise 参数（-Dnoise.probe=true）。
 * 默认导出全部（NOISE_PARAMETERS 注册表；含主世界+下界共享的 38 个）；
 * 也可 -Dnoise.names=a,b,c 指定（逗号分隔），-Dnoise.suffix 控制输出名（noise_params.json / noise_params_<suffix>.json）。
 * C++ 侧 buildNoiseParams 加载合并（数据驱动：主世界硬编码表 + JSON 补充）。
 */
public class NoiseParamProbe {
    public static void run(MinecraftServer server) {
        try {
            String names = System.getProperty("noise.names", "");
            String suffix = System.getProperty("noise.suffix", "");
            String outName = suffix.isEmpty() ? "noise_params.json" : "noise_params_" + suffix + ".json";
            Registry reg = server.getRegistryManager().get(RegistryKeys.NOISE_PARAMETERS);
            StringBuilder sb = new StringBuilder("{\n");
            List<String> keys;
            if (names.isEmpty()) {
                keys = new ArrayList<>();
                for (Object k : reg.getKeys()) {
                    keys.add(((RegistryKey<?>) k).getValue().toString());  // minecraft:xxx
                }
                java.util.Collections.sort(keys);
            } else {
                keys = new ArrayList<>();
                for (String n : names.split(",")) keys.add(n.trim());
            }
            for (int i = 0; i < keys.size(); i++) {
                String name = keys.get(i);
                Identifier id = Identifier.tryParse(name.contains(":") ? name : "minecraft:" + name);
                Object np = reg.get(RegistryKey.of(RegistryKeys.NOISE_PARAMETERS, id));
                if (np == null) {
                    System.err.println("[NoiseParamProbe] " + name + " not found, skipped");
                    continue;
                }
                int firstOctave = (int) invoke(np, new String[]{"firstOctave", "getFirstOctave"});
                Object ampsObj = invoke(np, new String[]{"amplitudes", "getAmplitudes"});
                List<?> amps = (ampsObj instanceof List<?> l) ? l : new ArrayList<>();
                sb.append("  \"").append(name).append("\": {\"firstOctave\": ").append(firstOctave)
                  .append(", \"amplitudes\": [");
                for (int j = 0; j < amps.size(); j++) {
                    if (j > 0) sb.append(", ");
                    sb.append(amps.get(j));
                }
                sb.append("]}");
                if (i < keys.size() - 1) sb.append(",");
                sb.append("\n");
            }
            sb.append("}\n");
            Path out = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
            Files.createDirectories(out);
            Files.writeString(out.resolve(outName), sb.toString(), StandardCharsets.UTF_8);
            System.out.println("[NoiseParamProbe] " + outName + " -> " + out.resolve(outName));
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
        System.out.println("[NoiseParamProbe] DONE, stopping server");
        server.stop(false);
    }

    private static Object invoke(Object target, String[] candidates) throws Exception {
        for (String m : candidates) {
            try {
                Method method = target.getClass().getMethod(m);
                return method.invoke(target);
            } catch (NoSuchMethodException ignored) {
            }
        }
        throw new NoSuchMethodException("no method in " + String.join("/", candidates) + " on " + target.getClass());
    }
}
