package wg.bench;

import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;

/**
 * Fabric mod 入口（main：单机 integrated server 与服务端都会加载）。
 * - 替换模式（-Dcpp.replace=1）：C++ 生成 NOISE/SURFACE 阶段
 * - 探针模式（-PxxxProbe=true）：bench 工具
 */
public class BenchMod implements ModInitializer {
    @Override
    public void onInitialize() {
        ServerLifecycleEvents.SERVER_STOPPING.register(server -> wg.bench.CppBridge.destroy());
        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            // CoreSwap 替换模式：-Dcpp.replace=1（正常游玩，不跑探针）
            boolean replace = System.getProperty("cpp.replace") != null;
            if (replace) {
                wg.bench.CppBridge.init(server.getOverworld().getSeed());
            }
            if (System.getProperty("biome.probe") != null) {
                BiomeParamProbe.run(server);
            } else if (System.getProperty("block.probe") != null) {
                BlockProbe.run(server);
            } else if (System.getProperty("probe.count") != null) {
                NoiseProbe.run(server);
            } else if (System.getProperty("router.probe") != null) {
                RouterProbe.run(server);
            } else if (System.getProperty("ore.probe") != null) {
                OreProbe.run(server);
            } else if (System.getProperty("jni.probe") != null) {
                JniProbe.run(server);
            } else if (System.getProperty("readWorld.probe") != null) {
                ReadWorldProbe.run(server);
            } else if (replace) {
                // CoreSwap 替换模式：正常游玩（服务器保持运行，不自动关服）
                System.out.println("[BenchMod] CoreSwap replace mode: C++ worldgen active");
            } else {
                WorldGenBench.run(server);
            }
        });
    }
}
