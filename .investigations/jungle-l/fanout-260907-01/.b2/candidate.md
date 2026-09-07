# 候选 .b2（结构阶段差核查——价值最高候选）：mineshaft 组件 ±整件级差 / geode 差参与两臂分歧

- 状态：**draft（禁 confirmed）**
- 验证分层：**Degraded（数据审查层）**——纯静态数据解读 + 源码引证，无运行时探针（subagent 无 shell 约束，符合 §9 分层声明）
- 载体口径（§9.7 可比性三要素）：
  - **载体**：Chunky 双臂对称试验，seed 8576294172403134396，coreswap 臂 = stageMask=3（Rust 地形 + Java vanilla carver/特征），vanilla 臂 = loom vanilla Java。与 260907-01 采集同批，可比。
  - **覆盖面**：① Structures NBT = **仅 starts 键名 + status + id 三字段**（见盲区声明）；② 块级 diff = terrain-only 口径 per-chunk（3025 common chunks，799 chunk 有差）。
  - **与既有口径可比性**：与 F3 通道①/②归因（f3-twochannel-260906-09.md）同一载体族；与 native 全-Rust 臂（§4.2）**不可比**（不同执行语义，#36/#66）。
- 输入数据：`.tmp/jungle-l-260906/chunky/structures_nbt_260907-01.txt`、`.tmp/jungle-l-260906/chunky/diff_per_chunk_260907-01.txt`；执行体语义引证：`.investigations/jungle-l/f3-twochannel-260906-09.md` §2/§4.2、`versions/1.20.1/docs/10-timewise-archive.md` L3043。

---

## 1. starts NBT 层：±整件级差被否定（有明确盲区）

**采集事实**：两臂 starts 键频率逐项相同（mineshaft 13 / shipwreck 3 / ocean_ruin_warm 2 / ocean_ruin_cold 2 / monument 2 / ruined_portal_ocean 1 / ruined_portal 1），start-status 差异 = 0。

**判定**：在 starts 层（结构「有没有被判定生成」）两臂零差异 → **「mineshaft 整件级 ±差」在 NBT starts 层的表现被直接否定**。若两臂 mineshaft 组件组装（children/pieces 树）有差，块级签名应为：橡木木板/栅栏走廊段成段出现或缺失（连续数块～数百块）、轨道/蜘蛛网/火把随走廊同步 ±、且空间上沿组件 BoundingBox 连片分布、跨 chunk 延伸。

**量级相容性检验**：
- 一个 mineshaft 走廊组件通常数百块木板/栅栏级材料，且 13 个 mineshaft starts 中任一「整件 ±」都会在 3025 chunk 面积上留下连片数百块级签名。
- 实测结构特征块差合计：cobblestone 172 + mossy_cobblestone 101 + oak_planks ~23 + oak_fence ~11 + rail ~11 + cobweb ~12 + spawner 2 ≈ **330 块，散布在 ~8 个 chunk**（~0.09 块结构块/chunk），最大单 chunk 集中 63 块（cx=31,cz=-22）、130 块（cx=42,cz=-27/-28）。
- **不相容**：该量级与「整件级 mineshaft 组件 ±差」的预期签名（数百块连片 × 多 chunk）差一个数量级以上。且在 starts 完全一致的前提下，「同 start 下组装出不同 pieces 树」需要 children 级随机输入差，先验低（piece 组装在 StructureStart 生成时由 chunk 随机决定，两臂同 seed 同 Java 代码 → 同随机流）。

**盲区声明（§9.7 诚实项）**：本次采集**未对比 children/pieces 的 BoundingBox 明细**——「两臂 pieces 树逐件一致」未被直接验证，只验证了「starts 集合一致」。理论上存在「starts 相同但 children 组装不同」的残余可能，块级签名检验只能间接压制（量级+形态不符），不能封死。

## 2. geode 候选：不构成独立第 5 阶段，归入通道①

**执行体语义核对（源码引证）**：
- `10-timewise-archive.md` L3043（F3 判定，judge 已核四代码锚点）：**CoreSwap mod mixin 只拦 NOISE/SURFACE（populateNoise + buildSurface），feature/carver 阶段无拦截，Java vanilla 装饰器/features 照常运行**；stageMask=3 只控 **Rust 内部**阶段（worldgen_handle.rs:143-145，Rust 跳过自己的特征放置）。
- `f3-twochannel-260906-09.md` §4.2：native 臂 flags=0 全跑是另一种执行体，与本载体不可比。
- **结论：两臂的 geode 都由 Java vanilla configured feature 放置，不存在「Rust 缺 geode」候选**——Rust 侧 geode 代码在 stageMask=3 下根本不参与，且 mod mixin 未触碰 feature 阶段。该子候选直接证伪。

**那么 geode 差是什么语义**：同一段 Java feature 代码、同 seed，在两臂放置结果不同 → 输入地形不同（Rust 地形 vs vanilla 地形的基座/湿度/谓词通过差异，通道②语义）或执行序/调度非确定（通道①语义）。两者都是 F3 已立项通道，**geode 差是通道①/②在结构块上的下游表现面，不是独立第 5 阶段**。

**块级证据（geode 参与 diff 的形态）**：
- cx=39,cz=-1：`oak_planks↔air 18`、`oak_planks↔smooth_basalt 12`、`oak_planks↔calcite 8`、`oak_fence↔calcite 5`、`rail↔air 4` + 巨型 geode 壳差（smooth_basalt 84 / calcite 49 / amethyst 44）——**一臂 geode 压掉了 mineshaft 走廊的木板**，另一臂 geode 未成或形态不同。这是「geode（feature 阶段，UNDERGROUND_DECORATION 步序晚于结构放置）覆盖 mineshaft（UNDERGROUND_STRUCTURES 步序）」的相互作用差，全部发生在 feature 阶段内部。
- cx=30,cz=-21：amethyst 9 / calcite 8 ↔ cave_air——geode 壳放置差；cx=42,cz=-25：smooth_basalt↔cave_air 7 同类。
- geode 壳特征块合计约 smooth_basalt 103 + calcite 70 + amethyst ~53 ≈ **226 块，集中在 3-4 个 chunk**——量级 = 少数 geode 的成败/形态差，不是系统性。

## 3. tuff 595 的拆分：多数非 geode 壳

按「tuff 差异 chunk 是否与 smooth_basalt/calcite/amethyst chunk 重合」粗判：
- **与 geode 共现的 tuff**（geode 壳内层语义）：cx=29/30,cz=-21（tuff↔granite 42+20，邻 amethyst/calcite chunk）、cx=31,cz=-22（cobble/mossy↔tuff 7，邻 cobblestone 壳 chunk）、cx=21,cz=-18（4）、cx=18,cz=-30（water↔tuff 19，含水环境）。约 **~90-100 块**。
- **孤立 tuff（无 calcite/basalt 共现）**：cx=14,cz=-22（diorite↔tuff 24）、cx=14,cz=-21（28）、cx=30,cz=-29（granite↔tuff 42）、cx=41/42,cz=0（water↔tuff 16，aquifer 面）等 ≈ **110+ 块**——这是 deepslate 层 blob 石族 / aquifer 面，属已知残差域（fan-out 预置候选①的辖区，见 10-timewise-archive L3070）。
- 判定：**tuff 差异主体（~2/3）来自 blob/含水层族，geode 壳只占少数**。且上述 water↔tuff/water↔deepslate 大面（846 water 差）本属洞穴级联候选（预置候选②）。无论哪一族，都不经过结构阶段。

## 4. 结论（draft，三选一）

**证伪——「结构阶段差」不是独立第 5 归因通道，证据全部归并入既有通道**：
1. starts 层零差 → 结构判定/组装在两臂一致（受盲区 §1 限定）；
2. 块级结构签名量级（~330 块/3025 chunk）与整件级差先验不相容；
3. 块级结构签名的形态是「feature 阶段相互作用差」：geode 壳压掉 mineshaft 木板（cx=39,cz=-1）、geode 壳放置成败差（cx=30,-21 / 42,-25）→ 通道①/②语义；孤立 tuff → blob/aquifer 已知残差域；「Rust 缺 geode」子候选被执行体语义直接证伪。
4. F3 wiring 图无需新增第 5 阶段：结构 starts 判定与 piece 放置在两臂均为纯 Java（mixin 未触碰），差异只能从地形输入或执行序进入——与通道①/②框架自洽。

**残余异常点（诚实披露，未闭合）**：cx=42,cz=-27/-28 的 `stone↔cobblestone 68 + mossy_cobblestone 47 + cave_air 79 + rail 3`（~200 块，两 chunk 连片）与 cx=30/31,cz=-23 的 `spawner/cobweb↔cave_air`。这是唯一不完全被 geode-overwrite 语义覆盖的簇：cobblestone+mossy+cave_air 组合的归属（mineshaft 与 ruined_portal 均含 cobble/mossy 语汇，本 region 有 2 个 ruined_portal starts；mineshaft 走廊是否用 cobblestone 本 worker 无源码在手，不臆断）需要 children BoundingBox 与之交叉定位才能归因。**不构成独立通道证据，但闭此案前应消解。**

**需要的采集（若主会话决定闭合残余异常点）**：
1. **children/pieces BoundingBox dump**：对 13 个 mineshaft + 2 个 ruined_portal starts，两臂各导出 children 数组（piece type + BoundingBox min/max），与 cx=42,cz=-27/-28、cx=30..31,cz=-23 做 chunk 交叉。零差 → 残余异常点归入 feature 阶段覆盖差，本案完全闭合。
2. **mixin 拦截点清单核对（建议）**：CoreSwap 自有 mixin 源码目录 = `runtime/1.20.1/java/src/main/java/wg/bench/mixin/`（本 worker glob 实证）。清单内 **没有任何结构阶段拦截点**（无 StructureStart/StructurePiece/MineshaftPieces 类 mixin；NoiseChunkGeneratorMixin = NOISE 层、SurfaceDumpProbeMixin 族 = SURFACE 层；其余为探针/装饰器诊断 mixin）。`versions/1.20.1/data/C2ME-fabric/` 下的 MixinMineshaftGenerator* / MixinStructureStart* 等是 **C2ME 参照数据源码，不是本 mod 安装面**，勿误当作拦截点。核对建议：确认 mod 的 mixins json 实际加载集合与上述目录一致即可——若一致，结构阶段「执行体差」通道在接线层面即不存在，进一步支持本案证伪。

## 未验证假设清单（本 candidate 全部遗留假设）

1. 「两臂 children/pieces 树逐件一致」——未经直接验证（starts 三字段对比的盲区，§1）。
2. 「piece 组装随机流两臂一致」——基于「同 seed 同 Java 代码 → 同随机」的先验推理，无 trace/probe 证据（Degraded 层）。
3. 「mineshaft 走廊组件使用 cobblestone 与否」——未查 Java 源码（MineshaftPieces 语汇），cx=42,-27/-28 簇的归属依赖此点，未闭合。
4. 「geode feature 步序（UNDERGROUND_DECORATION）晚于 mineshaft piece 放置（UNDERGROUND_STRUCTURES）」——1.20.1 decoration 步序的常识性引用，未逐条核对 data/minecraft/worldgen 源。
5. tuff 壳/blob 拆分为 chunk 共现粗判（~90 vs ~110 块），未做逐块空间定位；数值为量级估计非精确分账。
6. 「cx=39,cz=-1 木板差 = geode 覆盖走廊」是从块对方向推断的形态学解读，未用 BoundingBox 定位证实。
7. mixins json 实际加载集合 = glob 所见目录——未读 json 文件本身核对。
