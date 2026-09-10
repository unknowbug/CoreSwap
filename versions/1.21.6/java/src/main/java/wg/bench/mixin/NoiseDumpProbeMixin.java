package wg.bench.mixin;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.StructureAccessor;
import net.minecraft.world.gen.chunk.Blender;
import net.minecraft.world.gen.chunk.NoiseChunkGenerator;
import net.minecraft.world.gen.noise.NoiseConfig;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

/**
 * CoreSwap 诊断（B1 下钻四候选判别）：populateNoise RETURN 逐列 dump（NOISE 后状态）。
 * hook NoiseChunkGenerator.populateNoise RETURN——对 vanilla 与 cppReplace 都生效：
 *   - vanilla：NOISE 阶段产物（density/aquifer/oreVein 后、surface 前）
 *   - cppReplace：Rust fillChunkNether 产物（noise+surface 一起，surface 已做）
 * dump 每列：顶面 y（WORLD_SURFACE_WG 高度图）+ 材质序列（raw id 自顶向下）。
 * 判别映射：
 *   (d) 前置地形形状差 → vanilla populateNoise RETURN 的 topY vs cpp populateNoise RETURN 的 topY
 *       （高度图 = NOISE 地形顶，surface 不改高度；若两侧 topY 不同 → 地形形状已分叉）
 * 门控：-Dnoisedump.probe=1；chunk 过滤 -Dnoisedump.chunkX/-Dnoisedump.chunkZ/-Dnoisedump.size。
 * 只读性：不碰 Random、不改世界——纯旁观 dump。
 */
@Mixin(NoiseChunkGenerator.class)
public abstract class NoiseDumpProbeMixin {

    @Unique private static final boolean NOISE_DUMP = System.getProperty("noisedump.probe") != null;
    @Unique private static final String CHUNK_X = System.getProperty("noisedump.chunkX");
    @Unique private static final String CHUNK_Z = System.getProperty("noisedump.chunkZ");
    @Unique private static final int CHUNK_SIZE = Integer.parseInt(System.getProperty("noisedump.size", "4"));
    @Unique private static final String OUT = System.getProperty("noisedump.out", ".tmp/noisedump/noise");
    @Unique private static volatile boolean MAPPING_DONE = false;

    // 260908-10 签名迁移（1.21.6）：populateNoise 去掉首参 Executor（同 NoiseChunkGeneratorMixin）
    @Inject(method = "populateNoise("
            + "Lnet/minecraft/world/gen/chunk/Blender;"
            + "Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/gen/StructureAccessor;"
            + "Lnet/minecraft/world/chunk/Chunk;)"
            + "Ljava/util/concurrent/CompletableFuture;",
            at = @At("RETURN"), require = 1)
    private void wgNoiseDump(Blender blender,
                             NoiseConfig noiseConfig, StructureAccessor structureAccessor,
                             Chunk chunk,
                             CallbackInfoReturnable<java.util.concurrent.CompletableFuture<Chunk>> cir) {
        if (!NOISE_DUMP) return;
        try {
            int cx = chunk.getPos().x, cz = chunk.getPos().z;
            if (CHUNK_X != null) {
                int cx0 = Integer.parseInt(CHUNK_X);
                int cz0 = Integer.parseInt(CHUNK_Z == null ? "0" : CHUNK_Z);
                if (cx < cx0 || cx >= cx0 + CHUNK_SIZE || cz < cz0 || cz >= cz0 + CHUNK_SIZE) return;
            }
            if (!MAPPING_DONE) {
                MAPPING_DONE = true;
                System.out.println("[NOISEDUMP-LAUIDMAP] air=" + Block.STATE_IDS.getRawId(Blocks.AIR.getDefaultState())
                        + " lava=" + Block.STATE_IDS.getRawId(Blocks.LAVA.getDefaultState())
                        + " netherrack=" + Block.STATE_IDS.getRawId(Blocks.NETHERRACK.getDefaultState())
                        + " basalt=" + Block.STATE_IDS.getRawId(Blocks.BASALT.getDefaultState())
                        + " blackstone=" + Block.STATE_IDS.getRawId(Blocks.BLACKSTONE.getDefaultState())
                        + " bedrock=" + Block.STATE_IDS.getRawId(Blocks.BEDROCK.getDefaultState()));
            }
            int startX = chunk.getPos().getStartX();
            int startZ = chunk.getPos().getStartZ();
            int bottomY = chunk.getBottomY();
            Path out = Path.of(OUT + "-c" + cx + "-" + cz + ".csv").toAbsolutePath().normalize();
            StringBuilder sb = new StringBuilder();
            BlockPos.Mutable mutable = new BlockPos.Mutable();
            for (int k = 0; k < 16; k++) {
                for (int l = 0; l < 16; l++) {
                    int wx = startX + k, wz = startZ + l;
                    int topY = chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l);
                    StringBuilder mat = new StringBuilder();
                    for (int y = topY; y >= bottomY; y--) {
                        BlockState st = chunk.getBlockState(mutable.set(wx, y, wz));
                        if (mat.length() > 0) mat.append(',');
                        mat.append(Block.STATE_IDS.getRawId(st));
                    }
                    sb.append(wx).append(',').append(wz)
                            .append(",topY=").append(topY)
                            .append(",mat=").append(mat)
                            .append('\n');
                }
            }
            synchronized (NoiseDumpProbeMixin.class) {
                if (out.getParent() != null) Files.createDirectories(out.getParent());
                Files.write(out, sb.toString().getBytes(StandardCharsets.UTF_8),
                        StandardOpenOption.CREATE, StandardOpenOption.APPEND);
            }
        } catch (Throwable t) {
            System.out.println("[NOISEDUMP] failed: " + t);
        }
    }
}
