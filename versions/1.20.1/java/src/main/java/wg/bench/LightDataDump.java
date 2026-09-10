// 未编译验证（260905-04 P2 Java 侧交付，主会话负责编译）
// 依赖 yarn 签名：
//   Registries.BLOCK（net.minecraft.registry.Registries）#getDefaultState()（yarn 标准 Iterable<Block>）
//   Block#getDefaultState() : BlockState（yarn 标准）
//   AbstractBlockState#getOpacity(BlockView, BlockPos) : int（⚠️ 未含一手 yarn 源，风险 R3——null view/pos
//     若运行时抛异常则该项记 -1 并列清单）；AbstractBlockState#getLuminance() : int（yarn 标准）
package wg.bench;

import net.minecraft.registry.Registries;

import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * light_data.json 数据表生成器（260905-04 P2，给 Rust lightCompute 用）。
 * 入口：-DlightDataDump=&lt;path&gt;（BenchMod SERVER_STARTED 钩子，build.gradle -PlightDataDump 映射）。
 * 遍历 Registries.BLOCK，每 block 取 getDefaultState()，记录 opacity 与 emission：
 * {"format":"corewap-light-1","default":{"opacity":0,"emission":0},"blocks":{"&lt;rawId&gt;":{"opacity":N,"emission":N},...}}
 * opacity ∉ {0,15} 的特殊值（水/叶等）另打 stdout 清单供人工核对。
 */
public final class LightDataDump {
    private LightDataDump() {}

    public static void run(String outPath) {
        List<String> specials = new ArrayList<>();
        StringBuilder sb = new StringBuilder(1 << 16);
        sb.append("{\"format\":\"corewap-light-1\",\"default\":{\"opacity\":0,\"emission\":0},\"blocks\":{");
        boolean first = true;
        for (var block : Registries.BLOCK) {
            int rawId = Registries.BLOCK.getRawId(block);
            var state = block.getDefaultState();
            int opacity;
            try {
                opacity = state.getOpacity(null, null);
            } catch (Throwable t) {
                opacity = -1; // R3：null view/pos 不合法的特殊 state，列清单人工核对
                specials.add(rawId + " " + Registries.BLOCK.getId(block) + " opacity threw: " + t);
            }
            int emission = state.getLuminance();
            if (!first) sb.append(',');
            first = false;
            sb.append('"').append(rawId).append("\":{\"opacity\":").append(opacity)
              .append(",\"emission\":").append(emission).append('}');
            if (opacity != 0 && opacity != 15) {
                specials.add(rawId + " " + Registries.BLOCK.getId(block) + " opacity=" + opacity + " emission=" + emission);
            }
        }
        sb.append("}}");
        try {
            Path out = Path.of(outPath).toAbsolutePath().normalize();
            if (out.getParent() != null) Files.createDirectories(out.getParent());
            try (PrintWriter w = new PrintWriter(Files.newBufferedWriter(out, StandardCharsets.UTF_8))) {
                w.print(sb);
            }
            System.out.println("[LightDataDump] wrote " + out + " blocks=" + Registries.BLOCK.size());
        } catch (Exception e) {
            System.out.println("[LightDataDump] FAILED: " + e);
        }
        if (!specials.isEmpty()) {
            System.out.println("[LightDataDump] special opacity values (review):");
            for (String s : specials) System.out.println("[LightDataDump]   " + s);
        }
    }
}
