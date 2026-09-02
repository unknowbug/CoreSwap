# Java vanilla 1.20.1 nether biome 选择语义（对拍基准）

status: candidate（证据=yarn 1.20.1+build.10-v2 反编译源码 + vanilla data JSON，静态审查 = Degraded 分层声明：无运行时 trace）
来源：`versions/1.20.1/data/mc_src_extract/`（已解包的 loom minecraft-merged sources jar，
jar 原件 `runtime/1.20.1/java/.gradle/loom-cache/minecraftMaven/net/minecraft/minecraft-merged-7787b014d4/1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2/minecraft-merged-7787b014d4-1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2-sources.jar`）

## 0. 架构总述（1.20.1 无 NetherBiomeSource 类）

1.20.1 里 nether biome 参数**不在 Java 代码的独立 NetherBiomeSource**，而是：
- 硬编码 preset：`MultiNoiseBiomeSourceParameterList.Preset.NETHER`（net/minecraft/world/biome/source/MultiNoiseBiomeSourceParameterList.java，L60-76）——5 个 hypercube 硬编码于代码；
- 数据侧只有引用：`data/minecraft/worldgen/multi_noise_biome_source_parameter_list/nether.json` = `{"preset":"minecraft:nether"}`（preset id 反查回上面的代码表）。
- 通用选择引擎：`MultiNoiseBiomeSource.getBiome(x,y,z,sampler)` → `MultiNoiseUtil.MultiNoiseSampler.sample` → `MultiNoiseUtil.Entries.get(NoiseValuePoint)` → `SearchTree`。

## a. nether 5 个 biome 的 6 维参数盒 + offset（第 7 维）

`createNoiseHypercube(T,H,C,E,D,W,offset)`，每个分量变成 ParameterRange `[min,max]`（toLong = `(long)(f*10000.0F)` 截断，非四舍五入）：

| biome | T | H | C | E | D | W | offset |
|---|---|---|---|---|---|---|---|
| nether_wastes | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| soul_sand_valley | 0 | **-0.5** | 0 | 0 | 0 | 0 | 0 |
| crimson_forest | **0.4** | 0 | 0 | 0 | 0 | 0 | 0 |
| **warped_forest** | 0 | **0.5** | 0 | 0 | 0 | 0 | **0.375** |
| **basalt_deltas** | **-0.5** | 0 | 0 | 0 | 0 | 0 | **0.175** |

关键语义：
- 单值 `v` → 退化为点盒 `[v,v]`；`ParameterRange.getDistance(noise)` = 点在盒外时的盒外距离（`noise>max → noise-max`；`noise<min → min-noise`；盒内=0）。
- **offset 维是纯常数惩罚项**：采样点的第 7 维恒为 0（`NoiseValuePoint.getNoiseValueList()` 第 7 项硬编码 `0L`），距离公式里 `MathHelper.square(this.offset)` 与采样点无关——warped_forest 恒加 `3750² = 14,062,500`，basalt_deltas 恒加 `1750² = 3,062,500`，其余 3 个加 0。即 warped/basalt 需要 6 维距离小于该惩罚才可能胜出。
- nether 里 depth 恒 0、C/E/W 盒全为 0 → 只有 T/H 两维 + offset 惩罚实际起作用。climate 值范围：6 维采样点各 `toLong(f)`；不 clamp 到 [-1,1]（ParameterRange codec 才 clamp）。

## b. climate 采样点：block→cell，无随机 offset

调用链：`NoiseChunkGenerator.populateBiomes` → `Chunk.populateBiomes` → 每 section `ChunkSection.populateBiomes(biomeSupplier, sampler, x, y, z)`，其中 x/z=chunk 起始 biome 坐标、y=section 的 biome 坐标；对 section 内 4×4×4 每个 (j,k,l)：`biomeSupplier.getBiome(x+j, y+k, z+l, sampler)`（ChunkSection.java L182-192）。

- `MultiNoiseBiomeSource.getBiome(bx,by,bz,sampler)` → `sampler.sample(bx,by,bz)`；`MultiNoiseUtil.MultiNoiseSampler.sample`（MultiNoiseUtil.java L222-235）：`BiomeCoords.toBlock(coord) = coord << 2`，用 `(i,j,k)=(cellX*4, cellY*4, cellZ*4)` 构造 `DensityFunction.UnblendedNoisePos` 采 6 个 density function。
- **即：cell 角点直接采样（biome 坐标 <<2 的 block 坐标），没有任何随机 offset / cell 内插值**（Bedrock 式随机 pick 不存在于 Java）。y 同样是 cell 角（*4），但 nether temperature/vegetation `y_scale=0`，y 完全不影响。
- 第 7 维（offset）不是采样出来的：采样点恒 0，它是每个 biome 参数盒的静态属性（见 a）。
- 注意 `UnblendedNoisePos`：不做 aquifer/blending 偏移，就是原始 block 坐标。

## c. 最近邻规则

- 距离 = 7 维**平方距离和**（`NoiseHypercube.getSquaredDistance`，MultiNoiseUtil.java L287-295）：`Σ square(盒外距离) + offset²`，全 long 运算（toLong 截断后的整数）。
- 参考实现 `getValueSimple`（@VisibleForTesting）：**严格 `m < l` 才替换 → 平局时先来先得（列表顺序：wastes, ssv, crimson, warped, basalt）**。
- 生产实现走 `SearchTree`（Entries.getValue）：KD 树近似搜索 + **ThreadLocal `previousResultNode` 缓存**——上次查询的叶节点作为本次起点 alternative，平局时 alternative 保留（branch 节点严格 `l > m` 才下钻/替换）。**副作用：平局/近似平局结果依赖查询历史顺序**（即同一 climate 点在遍历顺序不同的上下文里可能判不同 biome）——Rust 侧若没有复刻 previousResult 缓存，边界点会出现残差（项目已有 WG_SEARCHTREE_CACHE 开关对拍此事）。平局在 nether 实际很难精确触发（toLong 截断 + offset² 奇偶），但「树搜索 + 缓存起点」对**非平局的最近邻也可能给出与线性扫描不同**的结果吗——不会：树保证返回的是真正最小距离叶（alternative 只在不严格更小时保留，最终 n 是真实距离比较），差异仅出现在 equal-distance 平局时谁被保留。
- **推荐 Rust 对拍口径：线性扫描 + 列表顺序 first-wins（getValueSimple），但要意识到 Java 生产实际走树 + previousResult 缓存；若残差点都落在 warped/basalt 边界带，优先核对 offset² 惩罚与 T/H 的 toLong 截断**。

## d. nether.json 六个 climate density function

`versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/nether.json` noise_router：

| 维 | 节点 |
|---|---|
| temperature | `shifted_noise { noise:"minecraft:temperature", shift_x:"minecraft:shift_x", shift_y:0.0, shift_z:"minecraft:shift_z", xz_scale:0.25, y_scale:0.0 }` |
| vegetation | 同型：`shifted_noise { noise:"minecraft:vegetation", shift_x:"minecraft:shift_x", shift_y:0.0, shift_z:"minecraft:shift_z", xz_scale:0.25, y_scale:0.0 }` |
| continents | 字面量 `0.0` |
| erosion | 字面量 `0.0` |
| depth | 字面量 `0.0` |
| ridges | 字面量 `0.0` |

即 nether 只采样 T/H 两维：都是 `shifted_noise`——Perlin noise（minecraft:temperature / minecraft:vegetation，各自 noise json），坐标先加 shift_x/shift_z 两个低频 Perlin（`minecraft:shift_x/shift_z` noise，xz_scale 默认 1）再以 xz_scale=0.25 采样，y 完全不参与。**Rust 判 warped_forest 而 Java 判 basalt/ssv 的残差，机制上只能来自 T/H 两维的数值差（含 shifted_noise 的 shift 采样差）或最近邻口径差（c 节）。**

## Rust 残差排查要点（对本任务直接可用）

1. warped_forest 带 offset 惩罚 3750²、basalt 带 1750²（long 整数域）——漏掉或用 float 平方都可能翻转边界带。
2. toLong 是 `(long)(f*10000.0F)`（Java float 乘法 + 向零截断），先乘后转。
3. 采样点是 cell 角 `coord<<2`，无随机 offset；nether 无 y 依赖。
4. 残差若只在树缓存启用时消失 → previousResultNode 顺序效应（平局类）。

## 主会话执行清单

无需解包：sources 已解包于 `versions/1.20.1/data/mc_src_extract/`（本次只读使用）。若未来需要重解包（1.20.1 yarn v2）：
```powershell
Expand-Archive -Path "E:\PYTHON\CoreSwap\runtime\1.20.1\java\.gradle\loom-cache\minecraftMaven\net\minecraft\minecraft-merged-7787b014d4\1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2\minecraft-merged-7787b014d4-1.20.1-net.fabricmc.yarn.1_20_1.1.20.1+build.10-v2-sources.jar" -DestinationPath "E:\PYTHON\CoreSwap\versions\1.20.1\data\mc_src_extract" -Force
```
（jar 实为 zip；只覆盖该目录，勿改其它。）
