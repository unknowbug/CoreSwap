package wg.bench.mixin;

import net.minecraft.world.gen.chunk.ChunkNoiseSampler;
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
 * CoreSwap 诊断（260903-12，P1.2 shared 臂裁决参照）：hook ChunkNoiseSampler.estimateSurfaceHeight(II)I，
 * 逐次调用 dump（raw 输入 blockX/blockZ + 返回值）。est 是量化列上的纯函数——调用方
 * （SURFACE 4 角 / aquifer 9 邻域）不影响值，全量 dump 后按量化列 (x>>2)<<2 归并即权威参照表。
 * 门控：-Destdump.probe=1；输出 -Destdump.out=.tmp/estdump/java-est.csv；
 * chunk 过滤 -Destdump.chunkX/-Destdump.chunkZ/-Destdump.size（按输入坐标 >>4 判定，默认不过滤）。
 * 只读旁观，不缓存/不改返回值。
 */
@Mixin(ChunkNoiseSampler.class)
public abstract class EstDumpProbeMixin {

    @Unique private static final boolean EST_DUMP = System.getProperty("estdump.probe") != null;
    @Unique private static final String EST_OUT = System.getProperty("estdump.out", ".tmp/estdump/java-est.csv");
    @Unique private static final String EST_CX = System.getProperty("estdump.chunkX");
    @Unique private static final String EST_CZ = System.getProperty("estdump.chunkZ");
    @Unique private static final int EST_SIZE = Integer.parseInt(System.getProperty("estdump.size", "0"));

    @Inject(method = "estimateSurfaceHeight(II)I", at = @At("RETURN"), require = 1)
    private void wgEstDump(int blockX, int blockZ, CallbackInfoReturnable<Integer> cir) {
        if (!EST_DUMP) return;
        int cx = blockX >> 4, cz = blockZ >> 4;
        if (EST_CX != null) {
            int cx0 = Integer.parseInt(EST_CX), cz0 = Integer.parseInt(EST_CZ == null ? "0" : EST_CZ);
            if (cx < cx0 || cx >= cx0 + EST_SIZE || cz < cz0 || cz >= cz0 + EST_SIZE) return;
        }
        int qx = (blockX >> 2) << 2, qz = (blockZ >> 2) << 2;
        String line = "EST," + blockX + "," + blockZ + "," + qx + "," + qz + "," + cir.getReturnValue() + "\n";
        try {
            Path out = Path.of(EST_OUT).toAbsolutePath().normalize();
            if (out.getParent() != null) Files.createDirectories(out.getParent());
            synchronized (EstDumpProbeMixin.class) {
                Files.write(out, line.getBytes(StandardCharsets.UTF_8),
                        StandardOpenOption.CREATE, StandardOpenOption.APPEND);
            }
        } catch (Throwable t) {
            System.out.println("[ESTDUMP] failed: " + t);
        }
    }
}
