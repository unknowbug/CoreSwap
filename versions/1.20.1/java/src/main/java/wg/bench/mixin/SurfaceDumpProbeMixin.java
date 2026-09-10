package wg.bench.mixin;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.Registry;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.Heightmap;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.gen.chunk.ChunkNoiseSampler;
import net.minecraft.world.gen.noise.NoiseConfig;
import net.minecraft.world.gen.surfacebuilder.MaterialRules;
import net.minecraft.world.gen.surfacebuilder.SurfaceBuilder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

/**
 * CoreSwap 诊断（B1 下钻四候选判别）：SURFACE 前/后逐列 dump。
 * hook SurfaceBuilder.buildSurface(NoiseConfig,BiomeAccess,Registry,boolean,HeightContext,Chunk,ChunkNoiseSampler,MaterialRule)
 * 的 HEAD（「前」= NOISE 产物）与 RETURN（「后」= SURFACE 产物），逐列 dump：
 *   材质序列（raw id 自顶向下）+ biome id + 顶面 y（WORLD_SURFACE_WG）+ useLegacyRandom 标志。
 * 判别映射：
 *   (d) 前置地形形状差 → 「前」dump 的顶面 y / 材质序列 vanilla vs cpp 是否已不同
 *   (a) 材质分支差 → 「后」vs「前」材质序列差（surface rule 改写了哪些块）
 *   (b) biome 判定输入差 → biome id 对比（useLegacyRandom=true → 判定 y=0 恒平）
 *   (c) 随机序列差 → 材质序列的随机选择漂移（间接）
 * 门控：-Dsurfacedump.probe=1；维度过滤 -Dsurfacedump.dim=minecraft:the_nether；
 * chunk 过滤 -Dsurfacedump.chunkX/-Dsurfacedump.chunkZ/-Dsurfacedump.size。
 * 只读性：不碰 Random、不改世界、不消费随机序列——纯旁观 dump。
 * 注意：必须在 vanilla 模式运行（不启用 -Dcpp.replace），否则 NoiseChunkGeneratorMixin 会 cancel surface 流程。
 */
@Mixin(SurfaceBuilder.class)
public abstract class SurfaceDumpProbeMixin {

    @Unique private static final boolean SURFACE_DUMP = System.getProperty("surfacedump.probe") != null;
    @Unique private static final String DIM_FILTER = System.getProperty("surfacedump.dim", "minecraft:the_nether");
    @Unique private static final String CHUNK_X = System.getProperty("surfacedump.chunkX");
    @Unique private static final String CHUNK_Z = System.getProperty("surfacedump.chunkZ");
    @Unique private static final int CHUNK_SIZE = Integer.parseInt(System.getProperty("surfacedump.size", "4"));
    @Unique private static final String OUT_PRE = System.getProperty("surfacedump.out", ".tmp/surfacedump/pre");
    @Unique private static final String OUT_POST = System.getProperty("surfacedump.outPost", ".tmp/surfacedump/post");
    @Unique private static volatile boolean MAPPING_DONE = false;
    @Unique private static volatile boolean DIM_SEEN = false;

    @Inject(method = "buildSurface(Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/biome/source/BiomeAccess;"
            + "Lnet/minecraft/registry/Registry;"
            + "ZLnet/minecraft/world/gen/HeightContext;"
            + "Lnet/minecraft/world/chunk/Chunk;"
            + "Lnet/minecraft/world/gen/chunk/ChunkNoiseSampler;"
            + "Lnet/minecraft/world/gen/surfacebuilder/MaterialRules$MaterialRule;)V",
            at = @At("HEAD"), require = 1)
    private void wgSurfaceDumpPre(NoiseConfig noiseConfig,
                                  net.minecraft.world.biome.source.BiomeAccess biomeAccess,
                                  Registry<Biome> biomeRegistry,
                                  boolean useLegacyRandom,
                                  net.minecraft.world.gen.HeightContext heightContext,
                                  Chunk chunk,
                                  ChunkNoiseSampler chunkNoiseSampler,
                                  MaterialRules.MaterialRule materialRule,
                                  CallbackInfo ci) {
        if (!SURFACE_DUMP) return;
        dumpColumns("PRE", chunk, biomeAccess, useLegacyRandom, OUT_PRE);
    }

    @Inject(method = "buildSurface(Lnet/minecraft/world/gen/noise/NoiseConfig;"
            + "Lnet/minecraft/world/biome/source/BiomeAccess;"
            + "Lnet/minecraft/registry/Registry;"
            + "ZLnet/minecraft/world/gen/HeightContext;"
            + "Lnet/minecraft/world/chunk/Chunk;"
            + "Lnet/minecraft/world/gen/chunk/ChunkNoiseSampler;"
            + "Lnet/minecraft/world/gen/surfacebuilder/MaterialRules$MaterialRule;)V",
            at = @At("RETURN"), require = 1)
    private void wgSurfaceDumpPost(NoiseConfig noiseConfig,
                                   net.minecraft.world.biome.source.BiomeAccess biomeAccess,
                                   Registry<Biome> biomeRegistry,
                                   boolean useLegacyRandom,
                                   net.minecraft.world.gen.HeightContext heightContext,
                                   Chunk chunk,
                                   ChunkNoiseSampler chunkNoiseSampler,
                                   MaterialRules.MaterialRule materialRule,
                                   CallbackInfo ci) {
        if (!SURFACE_DUMP) return;
        dumpColumns("POST", chunk, biomeAccess, useLegacyRandom, OUT_POST);
    }

    @Unique
    private static void dumpColumns(String phase, Chunk chunk,
                                    net.minecraft.world.biome.source.BiomeAccess biomeAccess,
                                    boolean useLegacyRandom, String outPath) {
        try {
            // 维度过滤：nether 才 dump（chunk 无直接维度引用，用 chunk 坐标过滤 + 调用方保证）
            int cx = chunk.getPos().x, cz = chunk.getPos().z;
            if (CHUNK_X != null) {
                int cx0 = Integer.parseInt(CHUNK_X);
                int cz0 = Integer.parseInt(CHUNK_Z == null ? "0" : CHUNK_Z);
                if (cx < cx0 || cx >= cx0 + CHUNK_SIZE || cz < cz0 || cz >= cz0 + CHUNK_SIZE) return;
            }
            if (!MAPPING_DONE) {
                MAPPING_DONE = true;
                System.out.println("[SURFDUMP-LAUIDMAP] air=" + Block.STATE_IDS.getRawId(Blocks.AIR.getDefaultState())
                        + " lava=" + Block.STATE_IDS.getRawId(Blocks.LAVA.getDefaultState())
                        + " water=" + Block.STATE_IDS.getRawId(Blocks.WATER.getDefaultState())
                        + " netherrack=" + Block.STATE_IDS.getRawId(Blocks.NETHERRACK.getDefaultState())
                        + " basalt=" + Block.STATE_IDS.getRawId(Blocks.BASALT.getDefaultState())
                        + " blackstone=" + Block.STATE_IDS.getRawId(Blocks.BLACKSTONE.getDefaultState())
                        + " soulsand=" + Block.STATE_IDS.getRawId(Blocks.SOUL_SAND.getDefaultState())
                        + " soulsoil=" + Block.STATE_IDS.getRawId(Blocks.SOUL_SOIL.getDefaultState())
                        + " magma=" + Block.STATE_IDS.getRawId(Blocks.MAGMA_BLOCK.getDefaultState())
                        + " bedrock=" + Block.STATE_IDS.getRawId(Blocks.BEDROCK.getDefaultState()));
            }
            int startX = chunk.getPos().getStartX();
            int startZ = chunk.getPos().getStartZ();
            int bottomY = chunk.getBottomY();
            Path out = Path.of(outPath + "-c" + cx + "-" + cz + ".csv").toAbsolutePath().normalize();
            StringBuilder sb = new StringBuilder();
            BlockPos.Mutable mutable = new BlockPos.Mutable();
            for (int k = 0; k < 16; k++) {
                for (int l = 0; l < 16; l++) {
                    int wx = startX + k, wz = startZ + l;
                    int topY = chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l);
                    // biome 判定（复刻 SurfaceBuilder L119：useLegacyRandom ? 0 : topY+1）
                    int biomeY = useLegacyRandom ? 0 : (topY + 1);
                    RegistryEntry<Biome> entry = biomeAccess.getBiome(mutable.set(wx, biomeY, wz));
                    String biomeId = entry.getKey().map(key -> key.getValue().toString()).orElse("?");
                    // 材质序列：自顶向下（topY 到 bottomY），记录 raw id
                    StringBuilder mat = new StringBuilder();
                    for (int y = topY; y >= bottomY; y--) {
                        BlockState st = chunk.getBlockState(mutable.set(wx, y, wz));
                        if (mat.length() > 0) mat.append(',');
                        mat.append(Block.STATE_IDS.getRawId(st));
                    }
                    sb.append(phase).append(',').append(wx).append(',').append(wz)
                            .append(",topY=").append(topY)
                            .append(",biomeY=").append(biomeY)
                            .append(",legacy=").append(useLegacyRandom)
                            .append(",biome=").append(biomeId)
                            .append(",mat=").append(mat)
                            .append('\n');
                }
            }
            synchronized (SurfaceDumpProbeMixin.class) {
                if (out.getParent() != null) Files.createDirectories(out.getParent());
                Files.write(out, sb.toString().getBytes(StandardCharsets.UTF_8),
                        StandardOpenOption.CREATE, StandardOpenOption.APPEND);
            }
        } catch (Throwable t) {
            System.out.println("[SURFDUMP] " + phase + " failed: " + t);
        }
    }
}
