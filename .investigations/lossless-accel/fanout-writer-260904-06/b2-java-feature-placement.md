# .b2 候选：OOB dirt/sand 写者是 Java vanilla features（条件性放置差）

> **置信度：draft**（静态审查，Degraded 分层——无运行时验证）
> 产物：fanout-writer-260904-06 / b2-java-feature-placement.md（260904-06 fan-out，只读分析，无命令执行）
> Java 源码参照：`E:\PYTHON\MC\data\mc_src_extract\net\minecraft`（yarn 1.20.1）；
> 数据参照：`E:\PYTHON\MC\versions\1.20.1\java\src\main\resources\worldgen-data\data\minecraft\worldgen\`（下称 `<wg>`）。

---

## 1. 结论一句话

**b2 只能解释一小部分 OOB 样本（y∈[0,160] 的 ore_dirt 类翻转 + 树 trunk-base setToDirt 翻转），无法解释三组关键形态**：
① y≥216 周期 ~16 dirt 簇；② (244,-60..-54,244) 单列 7 深 gravel→sand；③ 负 y 段 dirt（-41..-9）。
**周期 16 竖列形态 b2 明确无法解释——无任何 vanilla feature 具备竖向周期性放置**（判别关键，诚实声明见 §3.1）。
另发现一个**上游混淆项**：A（mod 链 = Rust NOISE+SURFACE）vs B（C++ NOISE+SURFACE 探针）的 diff 混入了 **Rust↔C++ 引擎分叉 + Java carvers + structures** 四个非 feature 来源，A−B 的 5.7% 不能全记在 feature 头上（§5）。

---

## 2. 前置确认：Java features 的执行点（必做分析 #3）

**确认**：`E:\PYTHON\MC\versions\1.20.1\java\src\main\java\wg\bench\mixin\NoiseChunkGeneratorMixin.java` 只有两个注入点：
- `populateNoise` 拦截：**L24-30**（@Inject HEAD cancellable），命中后 `CppBridge.fillChunk(chunk)`（L48）+ `cir.setReturnValue`（L49）跳过 Java NOISE；
- `buildSurface` 拦截：**L54-58**（@Inject HEAD + `ci.cancel()` L63）。

**FEATURE 阶段（以及 CARVERS 阶段）无任何 mixin 注入 → 由 vanilla Java 全量执行**。即 mask=3 链路下 mod 世界 = **Rust NOISE+SURFACE + vanilla Java CARVERS + vanilla Java FEATURES + vanilla structures**。
`CppBridge.fillChunk`→`writeChunk`（CppBridge.java **L196-229**）只写方块 section + 重算高度图（**L222-228** `Heightmap.populateHeightmaps` 全 6 种），**不写 biome**——biome 由 vanilla `createBiomes`（BIOMES status，先于 NOISE）按同 seed 算出。

---

## 3. 排查：哪些 vanilla overworld feature 能放 dirt/sand/gravel 到 y>200 或 y<-32（必做分析 #1）

### 3.1 dirt/sand/gravel 写者完整清单（configured_feature 全量扫描 `"Name": "minecraft:(dirt|sand|gravel)"`）

| 写者 | 方块 | 放置（placed json） | 能否到 y>200 / y<-32 |
|---|---|---|---|
| `ore_dirt` | dirt（target=tag base_stone_overworld，size 33；`<wg>/configured_feature/ore_dirt.json`） | count 7 + in_square + **height_range uniform [0,160]** + biome | ❌ 两头都不行（上限 160） |
| `ore_gravel` | gravel（同 target，size 33） | count 14 + **height_range [above_bottom 0, below_top 0] = 全域 -63..319** | ✅ 两头都行——但写的是 **gravel**，不是 dirt/sand |
| `disk_sand` | sand/sandstone（**target=matching_blocks dirt/grass_block**，half_height 2） | count 3 + **heightmap OCEAN_FLOOR_WG** + block_predicate_filter matching_fluids water + biome | 高度跟 ocean floor；**竖向跨度 ≤5**（DiskFeature.java L23-25: `j=i+half, k=i-half-1`）；**只能在已有 dirt/grass_block 上替换** |
| `disk_gravel` | gravel（target 同上 dirt/grass_block，half_height 2） | count 无 + heightmap OCEAN_FLOOR_WG + water filter | 同上，≤5 深，仅替换 dirt/grass |
| `disk_grass` | dirt(grass_block)（target=**dirt/mud**） | heightmap OCEAN_FLOOR_WG + water filter | dirt→dirt 等值替换/翻草，**净增 dirt ≈ 0** |
| **树（全部 tree 配置）** | dirt（trunk 基座） | TrunkPlacer.java **L58-66** `setToDirt`：`config.forceDirt \|\| !canGenerate(pos)` 时写 `dirtProvider`（=dirt）；canGenerate L58-60 = 基座是 soil 且非 grass/mycelium | 树 origin 由 heightmap 给出（单值/列，HeightmapPlacementModifier.java **L27-32**）；单棵树每 (x,z) 至多 1 个 dirt 基座 |
| `lake_lava(_underground/_surface)` | **lava+stone**（LakeFeature.java L60/L106；configured 只有 lava/stone） | underground：rarity 9 + height [0,top] + environment_scan down 32 + surface_relative_threshold(OCEAN_FLOOR_WG ≤ -5)；surface：rarity 200 + heightmap WORLD_SURFACE_WG | 高度可达高处/深处，但**写的是 lava/stone/CAVE_AIR，非 dirt/sand/gravel** → 排除 |
| `spring_water/lava` | 流体 | height uniform **[..192]** 等 | 只写流体 → 排除 |
| `freeze_top_layer` | ice/snow | world_surface 高度图 | 只写 ice/snow → 排除 |
| `forest_rock` | mossy_cobblestone | — | 排除 |
| `bamboo` / `delta` / `underwater_magma` / `geode` / `dripstone` | 竹/玄武岩/岩浆块/非目标 | — | 排除 |
| **`fossil`** | — | **`<wg>/placed_feature/fossil.json` 不存在**（1.20.1 fossil 是**结构**非 feature） | 排除出 b2（属结构写者） |
| **buried 系列** | — | buried treasure 是 structure，仅箱子 | 排除出 b2 |

**判别核心声明（诚实项）**：
1. **周期 ~16 竖向 dirt 簇（216, 229/232, 246-248, 262-264, 278-280, 294-295，且 y≡6..8 mod 16）无任何 vanilla feature 机制可产生**——所有 feature 的放置输入是 ①绝对高度随机（height_range，每 attempt 单 y、attempt 间独立随机无周期）或 ②heightmap 单值/列；单列 (195,199) 上 6+ 个互不相连 dirt 层要求 6 次独立 attempt 精确命中同一 (x,z)，概率≈0；树的 dirt 基座一树一个、基座在 heightmap 表面。**b2 对该形态无解释力**。
2. **(244,-60..-54,244) 单列 7 深 gravel→sand**：disk_sand 竖向跨度 ≤5 且 target 只认 dirt/grass_block（不能替换 stone/gravel）；无其他 feature 写 sand。**b2 无解释力**。（注：7 深 > disk 最大 5，单 disk 也不够。）
3. **负 y dirt（-41,-40,-27..-24,-10,-9）**：ore_dirt 下限 0；disk 需已存 dirt/mud 目标；树基座不会在 y<0。**b2 无解释力**。

### 3.2 反向异常（削弱「feature 清单完备性」自身）：vanilla ref 的深 dirt 也无 feature 解释

vanilla ref 列 (195,199) 自有 dirt @ **-59..-42, 184, 200**——其中 **-59..-42 段同样超出所有 feature dirt 写者的可达域**（ore_dirt ≥0）。ref 是 FULL vanilla（无 Rust、无 mixin），这说明参照导出里存在**本清单之外的 dirt 写者**（头号嫌疑：**structures**——A−B、ref−B 两条 diff 都含 vanilla structures；fossil/trail_ruins/ocean_ruins 类结构含 gravel/dirt/sand 类方块且随地形埋设）。**在结构写者被排查前，不能把该列任何 OOB dirt 直接归因 b1 或 b2**——这是本轮判别的一个方法论级发现。

---

## 4. mod↔vanilla 的 feature 输入差（必做分析 #2）——b2 真正能翻转什么

同 seed ⇒ `setDecoratorSeed` 随机序列、feature 顺序、biome 全同 ⇒ **每个 feature 的 attempt 原点在 ref 与 mod 中逐一相同（绝对高度类）或同样尝试（heightmap 类）**。会翻转的只有**状态耦合谓词**：

| 输入差 | 机制 | 受影响 feature | file:line |
|---|---|---|---|
| ① **heightmap 输入差** | mod 高度图由 Rust 地形重算（CppBridge.java **L220-228**）⇒ OCEAN_FLOOR_WG / WORLD_SURFACE_WG 与 ref 不同 | disk_*（origin 移位）、树（基座位置/基座下方块改变 → setToDirt 翻转）、lake_lava_surface | HeightmapPlacementModifier.java L27-32 |
| ② **方块状态谓词差** | Rust 地形与 vanilla 地形块状态不同（水面、含菌/含水、carver 掏空位置不同——Java carvers 在两世界都跑但地形不同 ⇒ 洞穴位置不同） | ore target 匹配（base_stone vs 空气/非 base）、disk 的 matching_fluids water、`setToDirt` 的 soil 检查（TrunkPlacer.java L62-66）、lake 的 isSolid/isLiquid 有效性检查（LakeFeature.java L74-86） | 见左 |
| ③ **BiomeFilter** | **不会翻转**——mod 不写 biome（CppBridge.writeChunk L196-229 无 biome 写入），biome 由同 seed vanilla createBiomes 算出 ⇒ ref/mod 同点位同 biome | BiomePlacementModifier.java **L24-29** | — |

→ b2 可解释的 mod-only dirt 形态仅限：**y∈[0,160] 的 ore_dirt 翻转**（如观测的 y=7：ref 侧该点被 carver 掏空或状态不匹配 → 未放；mod 侧 stone 在 → 放了）与**树基座 dirt 因高度图/基座 soil 检查翻转**。**解释不了 §3.1 的 ①②③ 形态。**
⚠️ 任务书原表述「mod 世界生物群系与 vanilla 不同」**不成立**（③）——b2 的翻转通道是高度图与方块状态，不是 biome。

---

## 5. 混淆项声明（证据包口径风险）

A = mod 链（**Rust** NOISE+SURFACE + Java carvers + Java features + structures）；B = **C++** block_probe NOISE+SURFACE。
A−B = **Rust↔C++ 引擎分叉 + carvers 有无 + Java features + structures** 四源叠加。把 A−B 的 OOB 差直接做「写者归属」会系统性高估 b2。更干净的归因对是 **ref−A**（两者同 structures 同 carvers 机制、同 seed）与 **E1 写者直接指认探针（§6）**。

---

## 6. 可判定实验（命令模板交主会话执行；加法消融，禁归零式 A/B）

**E1【决定性】OOB 写者指探（纯日志加法，零行为改变）**——mixin hook 在 `ChunkRegion.setBlockState`（或 `StructureWorldAccess` 实现的 setBlockState 调用点），当 `y>200 || y<-32` 且新状态 ∈ {dirt, sand, gravel} 时打印：
```
[FWriter] (x,y,z) <blockId> writer=<top-3 stack frames> thread=<t>
```
- stack 顶帧直接区分 `net.minecraft.world.gen.feature.*`（b2）/ `net.minecraft.world.structure.*`（结构写者，§3.2 嫌疑）/ 其他（surface/carver 路径 = b1 域）。
- 触发条件建议用系统属性门控（`-Dcoreswap.featureLog=1`）+ 坐标窗口过滤（列 (195,199)、(244,244) ±32 chunk）防日志爆炸（诊断 chunk 级判断一次，防热路径污染——AGENTS「测量污染铁律」）。
- 同 seed 双跑 ref/off 各一次，grep 列坐标。

**E2【高度图核验】** 在 off 链路 FEATURES 状态完成后，打印列 (195,199)/(244,244) 的 WORLD_SURFACE_WG 与 OCEAN_FLOOR_WG 值（探针已有 ChunkRegion 可取 `chunk.sampleHeightmap`）。若 ≈200 而 OOB dirt 在 216+，直接否掉「树基座/heightmap 驱动」解释，坐实周期 16 非特征产物。

**E3【结构写者核查】** 对 ref−B 的深 dirt（-59..-42）打印同位置 `StructureAccessor` 命中的 StructureStart piece 包围盒（或复用现有 structure 日志）；若命中结构（fossil/trail_ruins/ocean_ruins 类）→ 开「b3：structures」候选。fossil 在 1.20.1 是结构不是 feature（§3.1）。

**消融纪律**：E1/E2/E3 全部为加法日志（mask 恒 3、dll 不换），不引入「关掉某阶段」的归零式对比（#34 谓词耦合禁令）。

---

## 7. 自检清单

- [x] dirt/sand/gravel 写者按 configured+placed JSON 全量枚举，非凭记忆
- [x] 周期 16 / 负 y dirt / 7 深sand 三形态 **明确声明 b2 无解释力**（未强行归因）
- [x] ref 深 dirt 反常（§3.2）如实暴露——它是「清单外写者存在」的证据，指向结构候选
- [x] biome 差异假设被纠正（同 seed ⇒ BiomeFilter 不翻转，§4③）
- [x] A−B 混淆项声明（§5）
- [x] 实验为加法消融，符合 #34 禁令
- [x] 置信度 draft，未做任何运行时验证（Degraded 静态审查声明）
- [ ] 运行时验证（E1/E2/E3）——待主会话执行

## 附：关键 file:line 索引
- NoiseChunkGeneratorMixin.java L24-30（populateNoise 拦截）、L54-58（buildSurface 拦截）→ features/carvers 不拦
- CppBridge.java L196-229（writeChunk：只写方块+高度图，不写 biome）、L220-228（高度图重算）
- DiskFeature.java L17-38、L40-55（halfHeight 跨度 ≤2*half+1）
- TrunkPlacer.java L58-66（setToDirt 条件）
- BiomePlacementModifier.java L24-29；HeightmapPlacementModifier.java L27-32；HeightRangePlacementModifier.java L38-40
- LakeFeature.java L74-86（状态有效性检查）、L88-131（lava/stone only）
- `<wg>/placed_feature/{ore_dirt,ore_gravel,disk_sand,disk_gravel,lake_lava_underground}.json`；`<wg>/configured_feature/{ore_dirt,disk_sand,disk_gravel,disk_grass,lake_lava}.json`
