package wg.bench.mixin;

import java.util.function.BooleanSupplier;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.server.world.ServerWorld;
import wg.bench.FormProbe;

/**
 * 形态审计探针（260915-03）：ServerWorld.tick HEAD 挂 FormProbe 虚拟玩家票据驱动。
 * 仅在 -Dcoreswap.formdrive=1 时有行为（加/移自建 formprobe 票据），默认零操作。
 */
@Mixin(ServerWorld.class)
public abstract class ServerWorldFormProbeMixin {
    @Inject(method = "tick(Ljava/util/function/BooleanSupplier;)V", at = @At("HEAD"))
    private void wgFormProbeTick(BooleanSupplier shouldKeepTicking, CallbackInfo ci) {
        if (FormProbe.DRIVE || FormProbe.GRID || FormProbe.EDGE) {
            FormProbe.serverTick((ServerWorld) (Object) this);
        }
    }
}
