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
            // 发布默认：无任何探针参数 = CoreSwap 正常游玩（服务端/客户端都保持运行，不自动停服）
            boolean replace = System.getProperty("cpp.replace") != null;
            boolean wgBench = System.getProperty("worldgen.bench") != null;
            boolean anyProbe = System.getProperty("biome.probe") != null
                    || System.getProperty("block.probe") != null
                    || System.getProperty("probe.count") != null
                    || System.getProperty("router.probe") != null
                    || System.getProperty("ore.probe") != null
                    || System.getProperty("jni.probe") != null
                    || System.getProperty("readWorld.probe") != null
                    || wgBench;
            boolean active = replace || !anyProbe;  // 显式探针参数才跑探针，否则默认启用 CoreSwap
            if (active) {
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
            } else if (System.getProperty("noise.probe") != null) {
                NoiseParamProbe.run(server);
            } else if (wgBench) {
                WorldGenBench.run(server);
            } else {
                // 默认：CoreSwap 正常游玩模式（服务器保持运行）
                System.out.println("[BenchMod] CoreSwap replace mode: C++ worldgen active");
            }
        });
    }
}
