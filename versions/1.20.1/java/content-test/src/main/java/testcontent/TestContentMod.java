package testcontent;

import net.fabricmc.api.ModInitializer;
import net.minecraft.block.AbstractBlock;
import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.registry.Registries;
import net.minecraft.registry.Registry;
import net.minecraft.util.Identifier;

/**
 * 最小内容 mod 载体（260907-09 真实 mod 端到端实测）：
 * 注册 2 个自定义方块，方块 id 落在 vanilla 注册表尾部（Registries.BLOCK.size() 之后语义位置），
 * 供 CoreSwap CppBridge.registerModBlocks() 在 SERVER_STARTED 时扫描注册到 Rust 侧。
 * 自证打印 raw id 供冒烟日志与 [BLOCKS-REG] 行交叉核对。
 */
public class TestContentMod implements ModInitializer {
    public static final Block TEST_BRICK = new Block(AbstractBlock.Settings.create().strength(1.5f));
    public static final Block TEST_LAMP = new Block(AbstractBlock.Settings.create().strength(0.8f)
            .luminance((BlockState state) -> 15));

    @Override
    public void onInitialize() {
        Registry.register(Registries.BLOCK, new Identifier("testcontent", "test_brick"), TEST_BRICK);
        Registry.register(Registries.BLOCK, new Identifier("testcontent", "test_lamp"), TEST_LAMP);
        System.out.println("[TestContent] registered testcontent:test_brick raw="
                + Registries.BLOCK.getRawId(TEST_BRICK)
                + " testcontent:test_lamp raw=" + Registries.BLOCK.getRawId(TEST_LAMP));
    }
}
