// 未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）
// 依赖 yarn 签名（一手源 .tmp/light-yarn/ThreadedAnvilChunkStorage.java）：
//   ThreadedAnvilChunkStorage#releaseLightTicket(ChunkPos) : protected void（L684）
package wg.bench.mixin;

import net.minecraft.server.world.ThreadedAnvilChunkStorage;
import net.minecraft.util.math.ChunkPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(ThreadedAnvilChunkStorage.class)
public interface ThreadedAnvilChunkStorageAccessor {
    @Invoker("releaseLightTicket")
    void wgReleaseLightTicket(ChunkPos pos);
}
