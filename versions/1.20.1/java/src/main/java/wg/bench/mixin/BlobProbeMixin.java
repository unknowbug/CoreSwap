package wg.bench.mixin;

import net.minecraft.registry.Registry;
import net.minecraft.registry.RegistryKey;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.ChunkRegion;
import net.minecraft.world.StructureWorldAccess;
import net.minecraft.world.gen.chunk.ChunkGenerator;
import net.minecraft.world.gen.feature.PlacedFeature;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.io.BufferedWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

/**
 * CoreSwap 诊断（H1 blob 放置差）：dump placed feature 每个 origin（blob 起始点）。
 * -Dblob.probe=1 启用（默认关闭：static final 布尔短路，零开销）。
 * 输出格式：一行一 origin：x,y,z,featureId,dim
 * 只读性：不碰 Random、不改世界、不消费任何随机序列——纯旁观。
 * 方法匹配说明：yarn 1.20.1 PlacedFeature 实名方法为 generate（非 place，260902-06 踩坑），
 * 公开签名 boolean generate(StructureWorldAccess, ChunkGenerator, Random, BlockPos)，
 * 另有私有重载故必须带完整描述符；require = 1 表示至少命中 1 个目标方法，否则 mixin 应用
 * 阶段启动即崩（fail-fast）。
 */
@Mixin(PlacedFeature.class)
public abstract class BlobProbeMixin {

    private static final boolean BLOB_PROBE = System.getProperty("blob.probe") != null;
    private static volatile BufferedWriter BLOB_OUT;
    @Unique private static volatile int CALLS = 0;
    @Unique private static volatile int WRITTEN = 0;
    @Unique private static final java.util.Set<String> DIMS_SEEN = new java.util.HashSet<>();
    // mixin 禁止非 private 静态成员（本版本连 @Unique public static 方法都拒）；
    // BlobProbe 用反射读取这两个字段。

    @Inject(method = "generate(Lnet/minecraft/world/StructureWorldAccess;"
            + "Lnet/minecraft/world/gen/chunk/ChunkGenerator;"
            + "Lnet/minecraft/util/math/random/Random;"
            + "Lnet/minecraft/util/math/BlockPos;)Z", at = @At("HEAD"), require = 1)
    private void wgBlobProbeOrigin(StructureWorldAccess world, ChunkGenerator generator,
                                   net.minecraft.util.math.random.Random random,
                                   BlockPos pos, CallbackInfoReturnable<Boolean> cir) {
        if (!BLOB_PROBE) return;
        CALLS++;
        if (CALLS == 1) {
            System.out.println("[BLOB-PROBE] handler active worldClass="
                    + (world == null ? "null" : world.getClass().getName()));
        }
        try {
            String dim;
            if (world instanceof ChunkRegion region) {
                RegistryKey<?> k = region.toServerWorld().getRegistryKey();
                dim = k.getValue().toString();
            } else {
                return; // 非 chunk 装饰上下文不记
            }
            String dimFilter = System.getProperty("blobProbe.dim", "minecraft:the_nether");
            synchronized (BlobProbeMixin.class) {
                if (DIMS_SEEN.add(dim)) {
                    System.out.println("[BLOB-PROBE] dimSeen " + dim);
                }
            }
            if (!dim.equals(dimFilter)) return;
            String cxp = System.getProperty("blobProbe.chunkX");
            if (cxp != null) {
                int cx0 = Integer.parseInt(cxp);
                int cz0 = Integer.parseInt(System.getProperty("blobProbe.chunkZ", "0"));
                int size = Integer.parseInt(System.getProperty("blobProbe.size", "4"));
                int cx = pos.getX() >> 4, cz = pos.getZ() >> 4;
                if (cx < cx0 || cx >= cx0 + size || cz < cz0 || cz >= cz0 + size) return;
            }
            Registry<PlacedFeature> reg = world.getRegistryManager().get(RegistryKeys.PLACED_FEATURE);
            Identifier key = reg.getId((PlacedFeature) (Object) this);
            String id = key != null ? key.toString() : "direct:" + this.getClass().getSimpleName();
            String line = pos.getX() + "," + pos.getY() + "," + pos.getZ() + "," + id + "," + dim;
            Path out = Path.of(System.getProperty("blobProbe.out",
                    ".tmp/blob-probe/origins.csv")).toAbsolutePath().normalize();
            synchronized (BlobProbeMixin.class) {
                if (out.getParent() != null) {
                    Files.createDirectories(out.getParent());
                }
                Files.write(out, (line + System.lineSeparator()).getBytes(StandardCharsets.UTF_8),
                        StandardOpenOption.CREATE, StandardOpenOption.APPEND);
                WRITTEN++;
            }
        } catch (Throwable t) {
            System.out.println("[BLOB-PROBE] dump failed: " + t);
        }
    }
}
