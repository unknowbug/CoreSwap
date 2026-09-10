package wg.bench.mixin;

import net.minecraft.registry.Registry;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.ChunkRegion;
import net.minecraft.world.StructureWorldAccess;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.feature.ConfiguredFeature;
import net.minecraft.world.gen.feature.util.FeatureContext;
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
 * CoreSwap 诊断（H1 blob 放置差，第二层）：dump configured feature 实际放置点 placedPos。
 * PlacedFeature.generate 的入参只是 chunk 级装饰基准点（实测两轮 64 个且完全一致），
 * blob 真实位置在 placement modifier 流产出的 placedPos 上——在本方法 HEAD 捕获。
 * -Dblob.probe=1 门控；输出一行：x,y,z,configuredFeatureId,dim。只读、不碰 Random。
 */
@Mixin(ConfiguredFeature.class)
public abstract class ConfiguredFeatureProbeMixin {

    @Unique private static final boolean CFG_PROBE = System.getProperty("blob.probe") != null;
    @Unique private static volatile int CFG_CALLS = 0;
    @Unique private static volatile int CFG_WRITTEN = 0;
    @Unique private static final java.util.Set<String> CFG_DIMS = new java.util.HashSet<>();

    @Inject(method = "generate(Lnet/minecraft/world/StructureWorldAccess;"
            + "Lnet/minecraft/world/gen/chunk/ChunkGenerator;"
            + "Lnet/minecraft/util/math/random/Random;"
            + "Lnet/minecraft/util/math/BlockPos;)Z", at = @At("HEAD"), require = 1)
    private void wgCfgFeatureProbe(StructureWorldAccess world, ChunkGenerator generator,
                                   net.minecraft.util.math.random.Random random,
                                   BlockPos pos, CallbackInfoReturnable<Boolean> cir) {
        if (!CFG_PROBE) return;
        CFG_CALLS++;
        if (CFG_CALLS == 1) {
            System.out.println("[CFG-PROBE] handler active worldClass="
                    + (world == null ? "null" : world.getClass().getName()));
        }
        try {
            if (!(world instanceof ChunkRegion region)) return;
            String dim = region.toServerWorld().getRegistryKey().getValue().toString();
            synchronized (ConfiguredFeatureProbeMixin.class) {
                if (CFG_DIMS.add(dim)) {
                    System.out.println("[CFG-PROBE] dimSeen " + dim);
                }
            }
            if (!dim.equals("minecraft:the_nether")) return;
            Registry<ConfiguredFeature<?, ?>> reg =
                    world.getRegistryManager().get(RegistryKeys.CONFIGURED_FEATURE);
            Identifier key = reg.getId((ConfiguredFeature<?, ?>) (Object) this);
            String id = key != null ? key.toString() : "direct:" + this.getClass().getSimpleName();
            String line = pos.getX() + "," + pos.getY() + "," + pos.getZ() + "," + id + "," + dim;
            Path out = Path.of(System.getProperty("blobProbe.out",
                    ".tmp/blob-probe/origins.csv")).toAbsolutePath().normalize();
            out = out.resolveSibling(out.getFileName().toString().replace(".csv", "-cfg.csv"));
            synchronized (ConfiguredFeatureProbeMixin.class) {
                if (out.getParent() != null) {
                    Files.createDirectories(out.getParent());
                }
                Files.write(out, (line + System.lineSeparator()).getBytes(StandardCharsets.UTF_8),
                        StandardOpenOption.CREATE, StandardOpenOption.APPEND);
                CFG_WRITTEN++;
            }
        } catch (Throwable t) {
            System.out.println("[CFG-PROBE] dump failed: " + t);
        }
    }
}
