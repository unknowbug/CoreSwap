package wg.bench;

import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.world.gen.chunk.Blender;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.chunk.ChunkGeneratorSettings;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import net.minecraft.world.gen.densityfunction.DensityFunction;
import net.minecraft.world.gen.noise.NoiseConfig;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * vanilla 基准 harness：
 * 1. 对 (0,0) 起 size×size 个 chunk 做 FULL 生成并计时（毫秒/chunk）
 * 2. 对每个 chunk 导出 finalDensity 采样（4 列/方块间隔，即 cell 采样点）
 * 3. 输出 data/vanilla_<seed>_<size>.density（二进制）+ data/vanilla_<seed>_<size>.json（计时）
 *
 * 用法：-Dbench.seed=<long> -Dbench.size=<int>
 */
public class WorldGenBench {
    // 采样密度场的垂直分辨率：每几格采样一次（vanilla 内部为 4x4x8 cell）
    private static final int DENSITY_Y_INTERVAL = 8;
    private static final int DENSITY_XZ_INTERVAL = 4;

    public static void run(MinecraftServer server) {
        long seed = Long.parseLong(System.getProperty("bench.seed", "-8248318472910187742"));
        int size = Integer.parseInt(System.getProperty("bench.size", "8"));
        int originX = Integer.parseInt(System.getProperty("bench.originX", "200"));
        int originZ = Integer.parseInt(System.getProperty("bench.originZ", "200"));

        Path dataDir = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
        try {
            Files.createDirectories(dataDir);
        } catch (Exception e) {
            throw new RuntimeException("无法创建输出目录: " + dataDir, e);
        }

        ServerWorld world = server.getOverworld();
        ServerChunkManager chunkManager = world.getChunkManager();
        ChunkGenerator generator = chunkManager.getChunkGenerator();
        NoiseConfig noiseConfig = chunkManager.getNoiseConfig();

        if (!(generator instanceof NoiseChunkGenerator)) {
            System.err.println("错误：期望 NoiseChunkGenerator，实际为 " + generator.getClass().getName());
            server.stop(false);
            return;
        }
        NoiseChunkGenerator noiseGenerator = (NoiseChunkGenerator) generator;
        RegistryEntry<ChunkGeneratorSettings> settingsEntry = noiseGenerator.getSettings();
        ChunkGeneratorSettings settings = settingsEntry.value();

        System.out.println("[WorldGenBench] seed=" + seed + " size=" + size + " origin=(" + originX + "," + originZ + ")");
        System.out.println("[WorldGenBench] settings=" + settingsEntry.getKey().orElse(null));

        List<Long> chunkTimesMs = new ArrayList<>();
        Path densityFile = dataDir.resolve("vanilla_" + seed + "_" + size + ".density");

        // 预热：生成 2 个 chunk 触发 JIT/懒加载
        for (int i = 0; i < 2; i++) {
            world.getChunk(i, 0, ChunkStatus.FULL, true);
        }
        System.out.println("[WorldGenBench] warmup done");

        try (DataOutputStream out = new DataOutputStream(
                new BufferedOutputStream(new FileOutputStream(densityFile.toFile())))) {
            // 文件头：magic + seed + size + 采样分辨率
            out.writeInt(0x57474231); // "WGB1"
            out.writeLong(seed);
            out.writeInt(size);
            out.writeInt(DENSITY_XZ_INTERVAL);
            out.writeInt(DENSITY_Y_INTERVAL);

            for (int cz = 0; cz < size; cz++) {
                for (int cx = 0; cx < size; cx++) {
                    int wx = originX + cx;
                    int wz = originZ + cz;
                    ChunkPos pos = new ChunkPos(wx, wz);
                    long t0 = System.nanoTime();
                    Chunk chunk = world.getChunk(wx, wz, ChunkStatus.FULL, true);
                    long t1 = System.nanoTime();
                    long ms = (t1 - t0) / 1_000_000;
                    chunkTimesMs.add(ms);
                    System.out.println("[WorldGenBench] chunk (" + wx + "," + wz + ") FULL in " + ms + " ms");

                    // 导出 finalDensity 采样：直接用最小 NoisePos 逐点采样
                    DensityFunction finalDensity = settings.noiseRouter().finalDensity();
                    int minY = settings.generationShapeConfig().minimumY();
                    int height = settings.generationShapeConfig().height();
                    int sx = (int) Math.ceil(16.0 / DENSITY_XZ_INTERVAL);
                    int sz = (int) Math.ceil(16.0 / DENSITY_XZ_INTERVAL);
                    int sy = (int) Math.ceil((double) height / DENSITY_Y_INTERVAL);
                    out.writeInt(wx);
                    out.writeInt(wz);
                    out.writeInt(sx);
                    out.writeInt(sy);
                    out.writeInt(sz);
                    out.writeInt(minY);
                    out.writeInt(height);
                    SimpleNoisePos densityPos = new SimpleNoisePos();
                    for (int y = 0; y < sy; y++) {
                        for (int z = 0; z < sz; z++) {
                            for (int x = 0; x < sx; x++) {
                                densityPos.x = wx * 16 + x * DENSITY_XZ_INTERVAL;
                                densityPos.z = wz * 16 + z * DENSITY_XZ_INTERVAL;
                                densityPos.y = minY + y * DENSITY_Y_INTERVAL;
                                out.writeDouble(finalDensity.sample(densityPos));
                            }
                        }
                    }
                }
            }
        } catch (Exception e) {
            throw new RuntimeException("生成/导出失败", e);
        }

        // 计时报告
        long totalMs = chunkTimesMs.stream().mapToLong(Long::longValue).sum();
        long minMs = chunkTimesMs.stream().mapToLong(Long::longValue).min().orElse(0);
        long maxMs = chunkTimesMs.stream().mapToLong(Long::longValue).max().orElse(0);
        double avgMs = chunkTimesMs.stream().mapToLong(Long::longValue).average().orElse(0);
        StringBuilder sb = new StringBuilder();
        sb.append("{\n");
        sb.append("  \"seed\": ").append(seed).append(",\n");
        sb.append("  \"size\": ").append(size).append(",\n");
        sb.append("  \"chunks\": ").append(size * size).append(",\n");
        sb.append("  \"totalMs\": ").append(totalMs).append(",\n");
        sb.append("  \"avgMsPerChunk\": ").append(String.format(java.util.Locale.ROOT, "%.3f", avgMs)).append(",\n");
        sb.append("  \"minMsPerChunk\": ").append(minMs).append(",\n");
        sb.append("  \"maxMsPerChunk\": ").append(maxMs).append(",\n");
        sb.append("  \"densityFile\": \"").append(densityFile.getFileName()).append("\",\n");
        sb.append("  \"densityXzInterval\": ").append(DENSITY_XZ_INTERVAL).append(",\n");
        sb.append("  \"densityYInterval\": ").append(DENSITY_Y_INTERVAL).append("\n");
        sb.append("}\n");
        try {
            Files.writeString(dataDir.resolve("vanilla_" + seed + "_" + size + ".json"), sb.toString(), StandardCharsets.UTF_8);
        } catch (Exception e) {
            throw new RuntimeException("写计时报告失败", e);
        }
        System.out.println("[WorldGenBench] total=" + totalMs + "ms avg=" + avgMs + "ms min=" + minMs + "ms max=" + maxMs + "ms");
        System.out.println("[WorldGenBench] density -> " + densityFile);
        System.out.println("[WorldGenBench] DONE, stopping server");
        server.stop(false);
    }

    /** 最小 NoisePos 实现：直接对世界坐标采样 DensityFunction。 */
    static final class SimpleNoisePos implements DensityFunction.NoisePos {
        int x, y, z;

        @Override
        public int blockX() {
            return x;
        }

        @Override
        public int blockY() {
            return y;
        }

        @Override
        public int blockZ() {
            return z;
        }

        @Override
        public Blender getBlender() {
            return Blender.getNoBlending();
        }
    }
}
