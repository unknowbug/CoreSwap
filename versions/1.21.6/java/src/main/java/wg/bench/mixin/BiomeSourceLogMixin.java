package wg.bench.mixin;

import net.minecraft.registry.entry.RegistryEntry;
import net.minecraft.world.biome.Biome;
import net.minecraft.world.biome.source.BiomeSource;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Set;

/**
 * 260905-08 E-C2 修复配套：打印 biomeSource.getBiomes() 的迭代序（= ChunkGenerator 构建
 * features 全局索引的枚举序）。门控同 ChunkRandomSeedLogMixin（wg.seedlog）。
 */
@Mixin(BiomeSource.class)
public abstract class BiomeSourceLogMixin {
    private static boolean wg$seedlog() {
        return Boolean.getBoolean("wg.seedlog") || System.getenv("WG_SEEDLOG") != null;
    }
    private static boolean wg$logged = false;

    @Inject(method = "getBiomes", at = @At("RETURN"))
    private void wg$logBiomeOrder(CallbackInfoReturnable<Set<RegistryEntry<Biome>>> cir) {
        if (wg$seedlog() && !wg$logged) {
            wg$logged = true;
            int i = 0;
            for (RegistryEntry<Biome> e : cir.getReturnValue()) {
                String key = e.getKey().map(k -> k.getValue().toString()).orElse("<anonymous>");
                System.out.println("[BIOMELOG] " + i + " " + key);
                i++;
            }
        }
    }
}
