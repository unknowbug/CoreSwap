// 260912-01 D3 / S-1：本文件 = 1.21.6 侧副本（分版 mixin 件；代码/注解/成员与 1.20.1 版逐字相同，仅本头注释不同）。
// 依据（scout 已核）：两版 ChunkSection 字段名与类型同名——1.20.1 ChunkSection.java:21-24 / 1.21.6 :21-24；
//   intermediary 名两版逐字相同（1.21.6 mappings-base.tiny:88534-88538 = 1.20.1 refmap:29-32）
//   ⇒ @Accessor 字面量两版通用，无需改共享源。
// C 线（260911-05）交付：ChunkSection 私有字段访问器（主会话负责编译验证）。
// 依赖 yarn 1.20.1 签名（一手源 .tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/ChunkSection.java）：
//   :21-24  private short nonEmptyBlockCount / randomTickableBlockCount / nonEmptyFluidCount
//           private final PalettedContainer<BlockState> blockStateContainer
//   （1.21.6 侧同形，已核）
//
// 为什么需要它（两个独立理由，均有本案实证）：
//   ① 语义保真：ChunkSection 构造器 (:27-31) 会调 calculateCounts()，而 calculateCounts (:111-140) 与
//      增量 setBlockState (:60-93) 的计数语义**不一致**——calculateCounts 把「非空气且流体非空」的方块
//      重复计入 nonEmptyBlockCount，且 nonEmptyFluidCount 只计「流体自身 hasRandomTicks()」的格
//      （静水 WaterFluid.Still.hasRandomTicks()=false ⇒ 静水 section 该计数为 0，而增量路径为 4096）。
//      vanilla populateNoise 用增量路径，故「老路径计数」才是生成期 vanilla 语义；bulk 若走构造器
//      则 hasRandomFluidTicks() 等派生字段偏离（Tier 2 判据）。
//   ② 性能：calculateCounts 实测 72.5µs/section（占被替换 section 成本的 64%，见 [WG-BULKWB] 汇总）；
//      自算计数（去重遍顺带，O(distinct)）≈ 0。
//   ⇒ 复用旧 section 对象、只换容器 + 直接写三个计数，既不重算也不新建对象（生物群系容器原样保留）。
package wg.bench.mixin;

import net.minecraft.block.BlockState;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ChunkSection.class)
public interface ChunkSectionAccessor {

    @Mutable
    @Accessor("blockStateContainer")
    void wgSetBlockStateContainer(PalettedContainer<BlockState> container);

    @Mutable
    @Accessor("nonEmptyBlockCount")
    void wgSetNonEmptyBlockCount(short value);

    @Mutable
    @Accessor("randomTickableBlockCount")
    void wgSetRandomTickableBlockCount(short value);

    @Mutable
    @Accessor("nonEmptyFluidCount")
    void wgSetNonEmptyFluidCount(short value);

    // 读侧：Tier 2（派生字段正确性）判据——两臂逐 chunk 比对三个计数必须**全等**。
    // 注意：这些计数**不入存档**（ChunkSerializer:310-311 只写 block_states/biomes），
    // 故 region 对拍（Tier 3）看不到它们；唯一可见差异面 = 生成期到下次存读之间的内存态
    // （hasRandomFluidTicks() → 流体随机刻调度；nonEmptyBlockCount → isEmpty()/客户端包）。
    @Accessor("nonEmptyBlockCount")
    short wgGetNonEmptyBlockCount();

    @Accessor("randomTickableBlockCount")
    short wgGetRandomTickableBlockCount();

    @Accessor("nonEmptyFluidCount")
    short wgGetNonEmptyFluidCount();
}
