package wg.bench;

import net.minecraft.block.Block;
import net.minecraft.registry.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.world.ServerChunkManager;
import net.minecraft.server.world.ServerWorld;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkStatus;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;

import java.io.BufferedOutputStream;
import java.io.DataOutputStream;
import java.io.FileOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * vanilla 方块参照导出：
 * 1. FULL 生成 size×size chunk，导出 16×16×384 方块（vanilla block 注册表 id）
 * 2. 导出方块 id→name 映射（blocks.json），供 C++ 侧使用同一张表
 * 用法：-Dblock.probe=1 -Dbench.seed=<seed> -Dbench.size=<n> -Dbench.originX=<x> -Dbench.originZ=<z>
 */
public class BlockProbe {
    private static final int MIN_Y = -64;
    private static final int HEIGHT = 384;

    public static void run(MinecraftServer server) {
        long seed = Long.parseLong(System.getProperty("bench.seed", "-8248318472910187742"));
        int size = Integer.parseInt(System.getProperty("bench.size", "4"));
        int originX = Integer.parseInt(System.getProperty("bench.originX", "3200"));
        int originZ = Integer.parseInt(System.getProperty("bench.originZ", "3208"));

        Path dataDir = Path.of(System.getProperty("bench.out", "data")).toAbsolutePath().normalize();
        try {
            Files.createDirectories(dataDir);
        } catch (Exception e) {
            throw new RuntimeException("无法创建输出目录: " + dataDir, e);
        }

        ServerWorld world = server.getOverworld();
        ServerChunkManager chunkManager = world.getChunkManager();
        ChunkGenerator generator = chunkManager.getChunkGenerator();
        if (!(generator instanceof NoiseChunkGenerator)) {
            System.err.println("错误：期望 NoiseChunkGenerator，实际为 " + generator.getClass().getName());
            server.stop(false);
            return;
        }

        System.out.println("[BlockProbe] seed=" + seed + " size=" + size + " origin=(" + originX + "," + originZ + ")");

        // 预热
        for (int i = 0; i < 2; i++) {
            world.getChunk(i, 0, ChunkStatus.FULL, true);
        }

        Path blocksFile = dataDir.resolve("vanilla_" + seed + "_" + size + "_" + originX + "_" + originZ + ".blocks");
        try (DataOutputStream out = new DataOutputStream(
                new BufferedOutputStream(new FileOutputStream(blocksFile.toFile())))) {
            out.writeInt(0x57474232); // "WGB2"
            out.writeLong(seed);
            out.writeInt(size);
            out.writeInt(originX);
            out.writeInt(originZ);
            out.writeInt(MIN_Y);
            out.writeInt(HEIGHT);

            BlockPos.Mutable pos = new BlockPos.Mutable();
            for (int cz = 0; cz < size; cz++) {
                for (int cx = 0; cx < size; cx++) {
                    int wx = originX / 16 + cx;
                    int wz = originZ / 16 + cz;
                    ChunkPos chunkPos = new ChunkPos(wx, wz);
                    long t0 = System.nanoTime();
                    // SURFACE 状态 = NOISE + SURFACE（不含 structures/carvers/features，与 C++ 方块层对齐）
                    Chunk chunk = world.getChunk(wx, wz, ChunkStatus.SURFACE, true);
                    long t1 = System.nanoTime();
                    System.out.println("[BlockProbe] chunk (" + wx + "," + wz + ") FULL in " + (t1 - t0) / 1_000_000 + " ms");
                    out.writeInt(wx);
                    out.writeInt(wz);
                    for (int y = MIN_Y; y < MIN_Y + HEIGHT; y++) {
                        for (int z = 0; z < 16; z++) {
                            for (int x = 0; x < 16; x++) {
                                Block block = chunk.getBlockState(pos.set(x, y, z)).getBlock();
                                out.writeShort(Registries.BLOCK.getRawId(block));
                            }
                        }
                    }
                }
            }
        } catch (Exception e) {
            throw new RuntimeException(e);
        }

        // 导出方块 id→name 映射（全注册表）
        Map<String, Integer> idToName = new TreeMap<>();
        for (Block block : Registries.BLOCK) {
            idToName.put(Registries.BLOCK.getId(block).toString(), Registries.BLOCK.getRawId(block));
        }
        StringBuilder sb = new StringBuilder("{\n");
        List<Map.Entry<String, Integer>> entries = new ArrayList<>(idToName.entrySet());
        for (int i = 0; i < entries.size(); i++) {
            Map.Entry<String, Integer> e = entries.get(i);
            sb.append("  \"").append(e.getKey()).append("\": ").append(e.getValue());
            if (i < entries.size() - 1) sb.append(",");
            sb.append("\n");
        }
        sb.append("}\n");
        try {
            Files.writeString(dataDir.resolve("blocks.json"), sb.toString(), StandardCharsets.UTF_8);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }

        System.out.println("[BlockProbe] blocks -> " + blocksFile);
        System.out.println("[BlockProbe] DONE, stopping server");
        server.stop(false);
    }
}
