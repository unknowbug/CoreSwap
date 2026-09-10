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
                    || System.getProperty("densityProbe") != null
                    || System.getProperty("blob.probe") != null
                    || System.getProperty("estdump.probe") != null
                    || System.getProperty("aqdump.probe") != null
                    || System.getProperty("surfaceColDump") != null
                    || System.getProperty("colprof.probe") != null
                    || wgBench;
            // cpp.vanilla（-Vanilla，260904-08）：纯 vanilla worldgen A/B 模式——跳过原生 dll 初始化
            //（旧逻辑 active = replace || !anyProbe：无参启动 anyProbe=false → 照样 init → temp dll 解包失败即崩）
            boolean vanilla = System.getProperty("cpp.vanilla") != null;
            boolean active = (replace || !anyProbe) && !vanilla;  // 显式探针参数才跑探针，否则默认启用 CoreSwap；vanilla 显式关闭
            if (active) {
                wg.bench.CppBridge.init(server.getOverworld().getSeed());
                // 多世界（2026-08-30）：nether 维度句柄（失败不阻断主世界）
                wg.bench.CppBridge.initNether(server.getOverworld().getSeed());
                // end 维度句柄（260906-04 end 接管；失败不阻断主世界）
                wg.bench.CppBridge.initEnd(server.getOverworld().getSeed());
                // mod 方块注册冒烟（260907-07 方案 C）：-Dcpp.blockRegister 门控，默认关
                wg.bench.CppBridge.registerModBlocks();
            } else if (vanilla) {
                System.out.println("[BenchMod] vanilla mode: CoreSwap worldgen disabled (A/B comparison)");
            }
            if (System.getProperty("lightDataDump") != null) {
                wg.bench.LightDataDump.run(System.getProperty("lightDataDump"));
            } else if (System.getProperty("biome.probe") != null) {
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
            } else if (System.getProperty("height.probe") != null) {
                HeightProbe.run(server);
            } else if (System.getProperty("densityProbe") != null) {
                DensityProbe.run(server);
            } else if (System.getProperty("chunkRandom.probe") != null) {
                ChunkRandomProbe.run(server);
            } else if (wgBench) {
                WorldGenBench.run(server);
            } else if (false) {
            } else if (System.getProperty("biome6cal") != null) {
                wg.bench.Biome6Probe.calibrate();
            } else if (System.getProperty("biome6") != null) {
                wg.bench.Biome6Probe.run(server);
            } else if (System.getProperty("surfaceColDump") != null) {
                wg.bench.SurfaceColDumpProbe.run(server);
            } else if (System.getProperty("blob.probe") != null) {
                wg.bench.BlobProbe.run(server);
            } else if (System.getProperty("estdump.probe") != null) {
                wg.bench.EstDumpProbe.run(server);
            } else if (System.getProperty("aqdump.probe") != null) {
                wg.bench.AquiferDumpProbe.run(server);
            } else if (System.getProperty("wg.diagNether") != null) {
                wg.bench.DiagNetherProbe.run(server);
            } else {
                // 默认：CoreSwap 正常游玩模式（服务器保持运行）
                System.out.println("[BenchMod] CoreSwap replace mode: C++ worldgen active");
            }
        });
    }
}



