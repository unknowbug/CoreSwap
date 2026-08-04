package wg.bench;

import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;

/**
 * Fabric mod 入口：server 启动完成后跑 WorldGenBench，然后关服。
 */
public class BenchMod implements DedicatedServerModInitializer {
    @Override
    public void onInitializeServer() {
        ServerLifecycleEvents.SERVER_STARTED.register(server -> {
            if (System.getProperty("probe.count") != null) {
                NoiseProbe.run(server);
            } else {
                WorldGenBench.run(server);
            }
        });
    }
}
