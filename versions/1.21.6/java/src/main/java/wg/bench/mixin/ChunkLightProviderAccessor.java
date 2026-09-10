// 未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）
// 依赖 yarn 签名：
//   ChunkLightProvider#chunkProvider : protected final ChunkProvider（⚠️ 本文件 .tmp/light-yarn/ 未含
//   ChunkLightProvider.java 一手源，字段名按 yarn 1.20.1 惯例 "chunkProvider"——若 refmap/apply 报
//   accessor 找不到字段，改回反射兜底，见交付 notes 风险点 R1）
package wg.bench.mixin;

import net.minecraft.world.chunk.ChunkProvider;
import net.minecraft.world.chunk.light.ChunkLightProvider;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ChunkLightProvider.class)
public interface ChunkLightProviderAccessor {
    @Accessor("chunkProvider")
    ChunkProvider wgGetChunkProvider();
}
