// 未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）
// 依赖 yarn 签名（一手源 .tmp/light-yarn/LightingProvider.java）：
//   LightingProvider#blockLightProvider : @Nullable private final ChunkLightProvider<?,?>（L16）
package wg.bench.mixin;

import net.minecraft.world.chunk.light.ChunkLightProvider;
import net.minecraft.world.chunk.light.LightingProvider;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(LightingProvider.class)
public interface LightingProviderAccessor {
    @Accessor("blockLightProvider")
    ChunkLightProvider<?, ?> wgGetBlockLightProvider();
}
