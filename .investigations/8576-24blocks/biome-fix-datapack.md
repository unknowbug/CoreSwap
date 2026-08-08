# #23/#24 biome 判定差 — 运行时数据包（主会话采集，待 worker 解读）

> 主会话只做工具采集（block_probe / gradle RouterProbe / BlockProbe 导出），以下全是**原始事实**，不含结论。解读/根因/修复方案由 worker 产出。

## 1. 参照可信性（已确认事实）
- CppBridge 默认关闭：`CppBridge.java:18`（`-Dcpp.replace=1` 才 init，`enabled` 由 `handle != 0` 决定）、`BenchMod.java:17`（`System.getProperty("cpp.replace") != null` 才 replace）
- 本次两次 gradle 运行均未传 `-Dcpp.replace` → 纯 vanilla 生成
- 8/7 23:23 旧参照（vanilla_8576294172403134396_6_720_-432.blocks，CoreSwap data/）与 8/8 18:35 新参照（E:\tmp\vanilla_8576294172403134396_2_800_-352.blocks，BlockProbe worldSeed=8576 核对 ✓）在 (812,73,-337) 都是 terracotta（494）→ **terracotta 是 vanilla 稳定产物**
- 新参照对比 C++（block_probe -mismatch）：(812,73,-337) got=1 vanilla=494、(815,89,-337) got=8 vanilla=494、(816,90,-337) got=8 vanilla=437，且 (812,-336) 区域 chunk(50,-21) 有**大片 terracotta**（y=73-85，x=811-815，z=-336--331，425/426/433/437/439/494 系）→ 参照 badlands 带真实存在
- 注：新参照 2×2 文件含 chunk(65515,65515)（=int16 的 -21，即 chunk(-21,-21)，在参照范围外）数据，block_probe 对比时 TOTAL 90.89%（4 chunk 有效）——新参照文件**仅用于局部确认**，非正式回归参照

## 2. C++ 判定输入（-biomeDump + WG_BIOMEDUMP，block_probe 本地）
- (812,73,-337)：pick=(203,18,-84) → sample=(812,72,-336)；t=0.550060272、hum=-0.094668537、cont=0.016117165、ero=-0.444270968、dep=0.103940725、w=-0.541882336 → forest
- (812,100,-337)：pick=(203,24,-84) → sample=(812,96,-336)；同 2D 分量（t/hum/cont/ero/w 相同），dep=-0.083559275 → forest
- (815,100,-337)：pick=(203,24,-85) → sample=(812,96,-340)；t=0.548055351、hum=-0.096356198 → forest
- (728,-408)（对照组）：sample=(728,-408,-4) t=0.484735548 → lukewarm_ocean
- (800,-428)（对照组）：sample=(800,-428,0) t=0.548235238 → stony_shore

## 3. Java 6 维分量（RouterProbe B 行，纯 vanilla router 分量直采，坐标=(pos>>2)<<2 floor 对齐）
- B (812,64,-340)：t=0.548046 hum=-0.096363 cont=0.015770 ero=-0.441729 dep=0.156736 w=-0.534382
- B (812,72,-340)：t=0.548046 hum=-0.096363 dep=0.094236（2D 分量全 y 相同）
- B (812,88,-340)：dep=-0.030764；B (812,100,-340)：dep=-0.124514
- **与 C++ 同坐标（-340）对比：t 差 ~9e-6（0.548055 vs 0.548046）、hum 差 ~7e-6**——浮点级小差

## 4. Java SURFBIOME（RouterProbe 手动 BiomeAccess：`new BiomeAccess((bx,by,bz)->bs2.getBiome(bx,by,bz,multiNoiseSampler), BiomeAccess.hashSeed(seed))`，判定输入=原始 BlockPos (x,y,z)）
- @ (812,y,-337)：y=64 savanna、68 forest、72 savanna、76 savanna、80 savanna、84 forest、88 forest、92 savanna、96 forest、100 forest、104 savanna
- @ (815,y,-337)：y=64-80 forest、**84 badlands、88 badlands**、92 forest、**96 badlands**、100 forest、104 forest
- **注意**：SURFBIOME（savanna/forest）与参照实际方块（terracotta=badlands 带）**矛盾**——SURFBIOME 路径（手动 BiomeAccess seed/选点）可信度存疑，需要判定

## 5. Java SurfaceBuilder 源码（mc_src_extract，L113-167）
- L117：`o = chunk.sampleHeightmap(WORLD_SURFACE_WG, k, l) + 1`（pillar 前表面高度）
- L119：`registryEntry = biomeAccess.getBiome(m, useLegacyRandom ? 0 : o, n)`（**y=o**）→ 仅用于 L120 pillar 判定 + L165 iceberg 判定
- L124：pillar 后重采样 `p = sampleHeightmap + 1`
- 主循环 L131-163：逐块 u，`initVerticalContext(q, vx, r, m, u, n)`（**逐块 y=u**）→ MaterialRules biome 条件逐块采样
- 对照 C++ surface.h：711 行 pillarBiome=biomeAtCached(m,o,n)（y=o ✓ 对齐 L119）；760 行主循环 biomeAtCached(m,wy,n)（逐块 ✓ 对齐 MaterialRules）——**两处 y 语义都对齐**（此条为事实对照，无结论）

## 6. 主会话已改代码（swe 顺手对齐，已编译 + 8576 99.9993% / 3200 99.9997% 零退化回归）
- aquifer.h:332 `-0.225` → `-0.225f` / `0.9` → `0.9f`（Java float 常量）
- aquifer.h:367 `fluidLevel != INT32_MAX` → `!= -32512`（Java field_35479）
- surface.h：buildSurface heightmap 改本地可变副本（heightmapIn → 副本 + pillar 写回，SteepCond 读 pillar 后高度）
- worldgen_api.cpp：wg_sample_biome 加 WG_BIOMEDUMP 诊断输出（选点+6 维）

## 7. 参照文件位置
- 新参照：`E:\tmp\vanilla_8576294172403134396_2_800_-352.blocks`（2×2，(800,-352)，581628 字节）
- 旧参照：`versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks`（6×6，(720,-432)）
- RouterProbe 原始输出：`.investigations\8576-24blocks\routerprobe\routerprobe_812_-337.txt`、`routerprobe_815_-337.txt`（全量 stdout）
- 新参照 block_probe 对比输出：`E:\tmp\bp_newref.txt`（Tee 未存，如需要重跑；关键 mismatch 行已列于 §1）
