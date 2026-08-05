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
                // 自测：PalettedContainer.set 直写是否生效（复用 writeChunk 的写法）
                try {
                    var world = server.getOverworld();
                    var chunk = world.getChunk(0, 0);
                    var pc0 = chunk.getSection(0).getBlockStateContainer();
                    net.minecraft.block.BlockState orig = pc0.get(0, 0, 0);
                    pc0.set(0, 0, 0, net.minecraft.block.Blocks.STONE.getDefaultState());
                    net.minecraft.block.BlockState g0 = pc0.get(0, 0, 0);
                    net.minecraft.block.BlockState viaChunk =
                            chunk.getBlockState(new net.minecraft.util.math.BlockPos(0, -64, 0));
                    System.out.println("[TEST-WRITE] section0(0,0,0): 原=" + orig
                            + " set=stone 读回=" + g0 + " chunk读=" + viaChunk);
                    // 模拟 writeChunk 的完整写法（24 sections 都 set 一个标记方块）
                    for (int secIdx = 0; secIdx < 24; secIdx++) {
                        chunk.getSection(secIdx).getBlockStateContainer().set(1, 1, 1,
                                net.minecraft.block.Blocks.DIAMOND_BLOCK.getDefaultState());
                    }
                    net.minecraft.block.BlockState g1 = chunk.getSection(0).getBlockStateContainer().get(1, 1, 1);
                    System.out.println("[TEST-WRITE] 24-sections set(1,1,1,diamond) 读回 section0=" + g1);
                } catch (Throwable t) {
                    System.out.println("[TEST-WRITE] error: " + t);
                }
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
