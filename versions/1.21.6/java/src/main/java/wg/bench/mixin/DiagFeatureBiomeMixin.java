package wg.bench.mixin;

import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.util.math.ChunkPos;
import net.minecraft.world.ChunkRegion;
import net.minecraft.world.StructureWorldAccess;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.chunk.Chunk;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.gen.StructureAccessor;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.HashSet;
import java.util.Set;

/**
 * CoreSwap 诊断（M14 下界怪异城）：feature 装饰阶段 biome 上下文 dump。
 * -Dwg.dumpbiome=1 启用；只打印，不改行为。
 * 产出：每次 generateFeatures 的 world registryKey / 中心 chunk / 3×3 邻 chunk
 * biome 容器并集（retainAll 前）∩ biomeSource 后集合 / 各邻 chunk 单独集合。
 */
@Mixin(ChunkGenerator.class)
public abstract class DiagFeatureBiomeMixin {

    private static volatile java.lang.reflect.Field wgDiagBiomeSourceField;

    private Object wgDiagBiomeSource() {
        try {
            if (wgDiagBiomeSourceField == null) {
                java.lang.reflect.Field f = ChunkGenerator.class.getDeclaredField("biomeSource");
                f.setAccessible(true);
                wgDiagBiomeSourceField = f;
            }
            return wgDiagBiomeSourceField.get(this);
        } catch (Throwable t) {
            return null;
        }
    }

    private static String wgBiomeId(RegistryEntry<Biome> b) {
        return b.getKey().map(k -> k.getValue().toString()).orElse("direct");
    }

    @Inject(method = "generateFeatures(Lnet/minecraft/world/StructureWorldAccess;"
            + "Lnet/minecraft/world/chunk/Chunk;"
            + "Lnet/minecraft/world/gen/StructureAccessor;)V",
            at = @At("HEAD"))
    private void wgDumpFeatureBiomes(StructureWorldAccess world, Chunk chunk,
                                     StructureAccessor structureAccessor, CallbackInfo ci) {
        if (System.getProperty("wg.dumpbiome") == null) return;
        try {
            String dim = "?";
            if (world instanceof ChunkRegion region) {
                RegistryKey<?> k = region.toServerWorld().getRegistryKey();
                dim = k.getValue().toString();
            } else if (world != null) {
                dim = world.getClass().getSimpleName();
            }
            ChunkPos center = chunk.getPos();
            // 3×3 邻 chunk biome 容器并集（复刻 ChunkGenerator#method_39787 输入侧）
            Set<RegistryEntry<Biome>> union = new HashSet<>();
            StringBuilder perChunk = new StringBuilder();
            ChunkPos.stream(new ChunkPos(center.x - 1, center.z - 1), new ChunkPos(center.x + 1, center.z + 1)).forEach(pos -> {
                Set<RegistryEntry<Biome>> one = new HashSet<>();
                try {
                    Chunk c = world.getChunk(pos.x, pos.z);
                    for (ChunkSection sec : c.getSectionArray()) {
                        if (sec != null && !sec.isEmpty()) {
                            sec.getBiomeContainer().forEachValue(one::add);
                        }
                    }
                } catch (Throwable t) {
                    perChunk.append(" chunk(").append(pos.x).append(',').append(pos.z).append(") ERR=").append(t);
                    return;
                }
                for (RegistryEntry<Biome> b : one) {
                    perChunk.append(' ').append(wgBiomeId(b));
                }
                perChunk.append(" @(").append(pos.x).append(',').append(pos.z).append(')');
                union.addAll(one);
            });
            Object src = wgDiagBiomeSource();
            Set<RegistryEntry<Biome>> retained = new HashSet<>(union);
            if (src instanceof net.minecraft.world.biome.source.BiomeSource bs) {
                retained.retainAll(bs.getBiomes());
            }
            System.out.println("[WG-DUMPBIOME] dim=" + dim
                    + " center=(" + center.x + "," + center.z + ")"
                    + " status=" + chunk.getStatus()
                    + " biomeSource=" + (src == null ? "null" : src.getClass().getSimpleName()));
            System.out.println("[WG-DUMPBIOME] unionBeforeRetain=(" + union.size() + "):" + perChunk);
            StringBuilder rb = new StringBuilder();
            for (RegistryEntry<Biome> b : retained) rb.append(' ').append(wgBiomeId(b));
            System.out.println("[WG-DUMPBIOME] afterRetain=(" + retained.size() + "):" + rb);
        } catch (Throwable t) {
            System.out.println("[WG-DUMPBIOME] dump failed: " + t);
        }
    }
}
