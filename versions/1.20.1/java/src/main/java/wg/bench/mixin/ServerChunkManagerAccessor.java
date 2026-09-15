package wg.bench.mixin;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.server.world.ChunkTicketManager;
import net.minecraft.server.world.ServerChunkManager;

/**
 * 形态审计探针（260915-03）用：暴露 {@code ServerChunkManager.ticketManager} 私有字段，
 * 供 FormProbe 虚拟玩家驱动调 {@code addTicketWithLevel/removeTicketWithLevel}
 * （一手源 ServerChunkManager.java:53 / ChunkTicketManager.java:173/177）。
 */
@Mixin(ServerChunkManager.class)
public interface ServerChunkManagerAccessor {
    @Accessor("ticketManager")
    ChunkTicketManager wgTicketManager();
}
