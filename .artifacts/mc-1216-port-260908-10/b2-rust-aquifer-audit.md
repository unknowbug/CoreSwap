---
candidate: b2
假设: Rust `worldgen-core/src/aquifer.rs` 存在相对 1.20.1 Java `AquiferSampler` 的潜在语义偏离（此前验证未覆盖的配置/分支），能够产生「某单元内 air 在 water 之上、而 vanilla 同位置是实心石头」的整列差异。
判定: REFUTES
置信度: candidate
module: swe
来源定位: |
  Rust: worldgen-core/src/aquifer.rs:9-11,13,229-230,244,250-288,316-338,350-370,437-450,452,454-529,
        531-565,567-589,591-602,604-630,632-661,663-675,677-688,691-697;
        worldgen-core/src/worldgen_handle.rs:306-312,440-446,634-646;
        worldgen-core/src/terrain.rs:229-241;
        worldgen-core/src/xoroshiro.rs:57-64,98-123
  Java 1.20.1: versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java:52-64,
        99-101,103-134,136-141,143-251,258-261,263-321,323-333,335-351,353-389,391-419,421-433,435-450;
        .../chunk/ChunkNoiseSampler.java:108,158-188,222-240;
        .../chunk/NoiseChunkGenerator.java:78-84;
        .../chunk/GenerationShapeConfig.java:46-48,54-58;
        .../biome/source/util/VanillaBiomeParameters.java:1206-1208;
        .../noise/NoiseConfig.java:54;
        .../util/math/random/Xoroshiro128PlusPlusRandom.java:42-44,129-140;
        .../dimension/DimensionType.java:45-47
  Java 1.21.6（参考）: versions/1.21.6/data/mc_src_extract/.../AquiferSampler.java:136-266,373-409,415;
        .../ChunkNoiseSampler.java:107,221-239,796-800;
        .../NoiseChunkGenerator.java:76-82;
        .../densityfunction/DensityFunctions.java:446,690-705;
        .../biome/source/util/VanillaBiomeParameters.java:1206-1208
  运行时数据（复用）: .tmp/p7-aqdump-260908-10/{points.txt,java-vanilla-aqdump.txt}、
        .tmp/p7-aqdump-rust-260908-10.log、.tmp/p7-aqdump-compare-260908-10.txt、
        .tmp/p7-cluster-result-260908-10.txt
  探针缺陷: runtime/1.20.1/java/src/main/java/wg/bench/mixin/AquiferDumpProbeMixin.java:109-115,302
        （runtime/1.21.6/java/.../AquiferDumpProbeMixin.java:302 同款）
---

# b2 — Rust `aquifer.rs` 相对 1.20.1 Java `AquiferSampler` 的语义偏离审计（只读静态）

> 口径声明（§9.7 三要素）：**载体** = 静态逐行对拍（Degraded/静态层）+ 复用既有运行时 dump（Partial：探针在 `density>0` 点打全 `na`）；
> **覆盖面** = `AquiferSampler.Impl` 全文件（apply/calculateDensity/getWaterLevel/getFluidLevel/getFluidBlockY/getNoiseBasedFluidLevel/getFluidBlockState）+ 全部外部输入（fluid level sampler / est DF / est 缓存 / splitter / method_43718 / field_35479 / 维度参数）+ 1.20.1↔1.21.6 版本差；
> **与既有口径可比性** = 与 `.investigations/residual-1830/`（b3 十七项静态对拍）同口径，不引入新数值口径。

## 0. 结论（先给判定）

**REFUTES**（对 H-b2 的**预测性主张**：存在能产生该整列签名的 aquifer 语义偏离）。

- 逐项对拍 **27 项**，26 项与 1.20.1 Java 逐位等价（含 1.20.1↔1.21.6 的版本差 = 仅 `needsFluidTick` 语义）。
- 找到 **1 处真实偏离**（此前未覆盖的分支）：water-over-lava 检查用**全链 `get_fluid_level`** 而非 Java 的**默认液面 sampler lambda**（`aquifer.rs:496` vs `AquiferSampler.java:215` / 1.21.6:225）。
  该偏离**只能返回 WATER、且必须 `block_y ≤ -10`** → 可以解释「深部 deepslate→water」子集，**不能**产生 air，也**不能**产生 y=-2..+2 的 water（§2 候选 1）。
- 复用既有数据层证据：68/68 共有点 **density 符号翻转**（Java d ∈ [+0.004, +2.215] → stone；Rust d ∈ [−0.0002, −0.05] → 进 aquifer），同号大差异 0 个 → 该簇差异**可由 density 输入充分解释**，无需 aquifer 内部偏离（§3 硬反证）。
- 结论：H-b2 不是残差根因；**残差根因在 aquifer 的 density 入参（final_density / 插值 / beardifier 一侧）**——属让渡桶（见 §3 让渡清单）。water-over-lava 偏离是**独立的、有界的真 bug**，需单独修复（§2 候选 1）。

## 1. 逐项对拍表（Rust vs 1.20.1 Java，含行号）

| # | 项目 | Rust（aquifer.rs / 其它） | 1.20.1 Java | 结论 |
|---|---|---|---|---|
| 1 | 方块语义载体 | `AIR/WATER/LAVA = 0/1/2`（:9-11）；`-1` = null → `Rock`（terrain.rs:241） | `BlockState`（:52-64）；`null` → ChunkNoiseSampler 默认块 stone（:181 + NoiseChunkGenerator:68） | 等价 |
| 2 | 单元范围 start/size | :316-325（`floor_div` :13） | :120-130 | 等价 |
| 3 | 单元索引 | :350-353 `(j*size_z+k)*size_x+i` | :136-141 同式 | 等价 |
| 4 | `waterLevels` 哨兵 | `{y:i32::MAX, block:AIR}`（:333）+ 判 `fl.y != i32::MAX`（:597） | `null`（:131,344）；`y` 不可能为 MAX（getFluidBlockY 只回 default.y/-32512/noise 值） | 等价（哨兵不可达） |
| 5 | `blockPositions` 哨兵 | `i64::MAX`（:332,441） | `Long.MAX_VALUE`（:133,177） | 等价 |
| 6 | pack/unpack | :354-370 | `BlockPos.asLong/unpackLongX/Y/Z`（:181,185-187） | 等价（y 12 位 / x,z 26 位符号扩展一致） |
| 7 | 单元定位 l/m/n | :469-471 `floor_div(x-5,16) / (y+1,12) / (z-5,16)` | :158-160 | 等价 |
| 8 | 邻域遍历 12 格 | :474 `u 0..=1 / v -1..=1 / w 0..=1` | :168-170 | 等价 |
| 9 | blob 随机位置 | :443-447 `split_xyz` + `next_int_bound(10/9/10)` + `x*16+rx …` | :180-181 `split(x,y,z)` + `nextInt(10/9/10)` | 等价（xoroshiro.rs:77-90 无偏 bound 与 Java `nextInt(bound)` 同构） |
| 10 | splitter 派生链 | worldgen_handle.rs:440-446 `split_str("minecraft:aquifer").next_splitter()` | NoiseConfig:54 + Xoroshiro128PlusPlusRandom:42-44 | 等价（1.20.1 `new Identifier("aquifer")` ≡ 1.21.6 `Identifier.ofVanilla("aquifer")` = `minecraft:aquifer`） |
| 11 | 三近邻排序（含 `>=` 并列语义） | :481-483 | :189-204 | 等价（逐分支同构） |
| 12 | maxDistance | :452 `1 - |a-i|/25` | :258-261 | 等价 |
| 13 | `density > 0` 早退 | :464 → `-1`（Rock） | :149-151 → `null`（stone） | 等价 |
| 14 | lava 早退 | :465-467 `block_y < -54 → LAVA` | :153-156 `fluidLevelSampler(i,j,k).getBlockState(j) isOf LAVA` = `j < -54`（NoiseChunkGenerator:78-84，seaLevel 63） | 等价（阈值 `min(-54, seaLevel)` = -54） |
| 15 | `d <= 0` 返回 | :491-492 返回 `bs = fl2.get_block_state(block_y)` | :211-214 返回 `blockState` | 等价 |
| 16 | **water-over-lava 检查** | :493-499 `self.get_fluid_level(block_x, block_y-1, block_z)`（**全 13 offset 链**） | :215-217 `this.fluidLevelSampler.getFluidLevel(i,j-1,k)`（**默认液面 lambda**） | **偏离（唯一）** |
| 17 | calculateDensity | :567-589 | :263-321 | 等价（lava/water 混合 `2.0`；`j==0 → 0.0`；`e>0` 两分支系数 1.5/2.5/3/10；barrier 单次采样 + 复用） |
| 18 | barrier 单次缓存 | :581-584 `md.has` 标志 | :304-311 `Double.isNaN(getValue())` | 等价（barrier 不返回 NaN） |
| 19 | e/g/h 三连与顺序 | :501-526 | :220-243 | 等价（含 `f>0` / `g2>0` 门与 `d*f`、`d*g2` 权重） |
| 20 | est 量化与缓存键 | :532-536 `(x>>2)<<2` + 32×32 相对索引 | ChunkNoiseSampler:222-225 `BiomeCoords.toBlock(fromBlock)` + per-chunk `Long2IntMap` | 等价（缓存键同量化列；Rust 覆盖域略小，见 §3 盲区 3） |
| 21 | est 扫描域/步长/阈值/未命中 | :549-556 `min_y+height → min_y` step 8，`>0.390625`，未命中 `i32::MAX` | ChunkNoiseSampler:230-239 同（`verticalCellBlockCount = 4*size_vertical = 8`） | 等价（Java `MAX+8` 与 Rust release 回绕同为 −2147483641，后续 `default_level` 判定同） |
| 22 | est 输入 DF | worldgen_handle.rs:312 `initial_density_without_jaggedness` | ChunkNoiseSampler:187,234 | 等价 |
| 23 | est 纯净性（L2 跨 chunk 缓存） | :244 `BLEND_ACTIVE=false`；:538-563 | （Java 只做 per-chunk 缓存） | 语义中性（见 §3 盲区 2 + 摘录 [D]） |
| 24 | per-cell 液面缓存 | :591-602 | :335-351 | 等价 |
| 25 | get_fluid_level 13 offset + 短路 | :604-630（`bl2 && k>o` 早退；`bl3||bl2` 采样；`bl` 置位；`i=min`） | :353-389 | 等价（Rust 用 `default_level(o)` 替代 `fluidLevelSampler(l,o,m)`：两者在 o≥-54 均 WATER@63、o<-54 均 LAVA@-54，`getBlockState(o)` 逐值相同） |
| 26 | get_fluid_block_y | :636-643 gate（`er < -0.225f32 as f64 && dv > 0.9f32 as f64`，短路）；:648-656 f/h/kk/d/e；:658-660 三出口 | :395-397 `method_43718`（**同 f32 字面量**，:1206-1208）+ :399-406 + :410-416 | 等价（`-0.225F` 提升 double 与 Rust `f32 as f64` 逐位相同；`MathHelper.map` 无 clamp 但 f∈[0,1] → Rust `map2` 的 clamp 为恒等） |
| 27 | noise-based 液面 + 方块态 | :663-675 `min(est, n+p)`；:677-688 `≤-10 && != -32512 && !=LAVA && |d|>0.3` | :421-433 `Math.min(est, q=n+p)`；:435-450（`field_35479 = MIN_HEIGHT<<4 = -32512`，DimensionType:45-47） | 等价 |
| — | 版本差（1.20.1→1.21.6） | Rust 未改（薄壳） | 1.21.6 `apply` 仅 top-3→top-4（第 4 槽只喂 `needsFluidTick`，:213-264）+ `method_43718`→`inDeepDarkParameters` 改名 | **方块语义等价**（b1 的「版本语义变化改变落方块」不成立） |

## 2. 可产生该签名的偏离候选（按可能性排序）

1. **water-over-lava 检查用全链而非默认液面（唯一真偏离；实锤，可解释子集）**
   - Rust `aquifer.rs:496`：`self.get_fluid_level(block_x, block_y-1, block_z)` → 13 offset 全链，返回的 `block` 可为 LAVA（当 `p ≤ -10 && |fluid_type| > 0.3`）。
   - Java `AquiferSampler.java:215`（1.21.6:225）：`this.fluidLevelSampler.getFluidLevel(i,j-1,k)` → 默认液面 lambda，`getBlockState(j-1) isOf LAVA` 当且仅当 `j-1 < -54`，即 **j ≤ -54**。
   - 触发域：Rust 分支需 `bs == WATER` 且 `get_block_state(block_y-1) == LAVA`，后者要求 `block_y-1 < p ≤ -10` → **block_y ≤ -10**；返回值是 `bs`（WATER），**永不返回 AIR**。
   - 能解释：y ≤ -10 的 `deepslate→water`（如 `(222,-25,238) V=deepslate R=water`）；不能解释：`deepslate→air`（6,347 处主对）与 y=-2..+2 的 water。
2. est 跨 chunk L2 缓存污染（:538-563）——**排除**：est 是 (x,z) 纯函数。证据链：`trimHeight` 对满高 chunk 恒等（GenerationShapeConfig:54-58，bottomY −64/topY 320）；router 的 `initial_density_without_jaggedness` 顶层是 `lerp/y_clamped_gradient`（DensityFunctions:690-705，纯）；内层 `cache2d(factor)` 键为 (x,z)、delegate 为 2D 纯函数（DensityFunctions:446）；即使被 `getActualDensityFunction` 换成 `DensityInterpolator`，其 `sample(pos)` 对非 sampler pos 直委派原始 DF（ChunkNoiseSampler:796-800）→ est 采样永不走插值。与既有结论（四臂 hash 同一）一致。
3. per-chunk `surface_cache` 覆盖缺口（:533-536，`ix<0` 时 `in_c=false`，如 blob 落在 chunk 西侧 16 格内）——**只影响性能**：命中失败仅重算同值；L2 以绝对 (x,z) 兜底正确性。
4. 其余可能（splitter 少一级、DF 绑定错位、常量/阈值/floorDiv/pack-unpack 错、`bl` 语义错、`default_level` 阈值错）——**逐项排除**：第 10/22 项与 `NoiseConfig:54` 对齐（260904 已修的 `split_str` 家族）；`default_level` 阈值 = `min(-54, seaLevel)`（NoiseChunkGenerator:78-84）与 Rust `-54` 逐值相同；`method_43718` 的 f32 字面量两侧一致。

## 3. 反证 / 盲区（§9.7 覆盖面声明）

**硬反证（数据层，Partial 口径）**
- 既有 AQDUMP 对比：`common=68`，**68/68 density 符号翻转**（Java d ∈ [+0.003965, +2.215056] → `null`(stone)；Rust d ∈ [−0.0002, −0.05] → 进 aquifer 得 `BLOCK:0/1/null:*`），`[SAME-SIGN LARGE DELTA >0.05] 0`。
- 逻辑反证：y > −10 的 water mismatch（如 `(223,-2..+2,237)`）上，唯一偏离分支**结构上不可达**（需 y ≤ −10 且只返回 WATER）→ 这些 mismatch **只能**来自 `density` 入参。反向（vanilla air/water → Rust stone）在残差表中无成规模出现（`p7-cluster-result` 仅 glow_lichen/water 个位数反向）。

**盲区（诚实声明）**
1. **Java 侧内部链在 mismatch 点不可见**：探针 `AquiferDumpProbeMixin:109-115` 在 `density > 0` 时早退打全 `na`；而 68 个共有点的 Java 侧 density 全 > 0 → 这些点的 Java aquifer 内部字段（est/opq/r-s-t/fl*/d/cd*）**未被采集**，字段级对拍缺一半（靠静态对拍覆盖）。
2. **探针自身 bug（非产品）**：`AquiferDumpProbeMixin:302`（1.20.1 与 1.21.6 同款）`return Math.min(surfaceHeightEstimate, prnd);` **漏 `+ base`**（对照 `AquiferSampler.java:427,431-432`：`q = n + p`）。凡走 noise-based 液面路径，探针复算的 `fl*` 整体偏 `spreadBase`；任何字段级对拍必须先修或按 `spreadBase` 补偿。
3. **两臂点集天然不齐**：Rust `terrain.rs:230` 在 `d>0` 时短路不调 `aq.apply` → `WG_AQDUMP` 只覆盖 `d≤0`（68/164）；Java 每块都调（ChunkNoiseSampler:181，164/164）。
4. **nether 参数潜在偏离（本课题不触发）**：`Aquifer::new` 的 `height` 取 `world_height`（worldgen_handle.rs:223，下界 256），Java 取 `generationShapeConfig.height()`（噪声高 128，ChunkNoiseSampler:171）；下界 `aquifers_enabled=false` → `enabled=false` 不消费该参数，暂不构成偏差，但属未覆盖配置。
5. **未做新的 Full 层验证**：本审计未跑新的 block_probe 全量（subagent 无 shell）；§4 给出主会话可执行的一锤定音探针。
6. **让渡清单（不属本候选账）**：该簇的 `density` 差异（final_density 树 / 插值 / beardifier）→ density 桶（建议新候选 b3）；结构/feature 残留、`deepslate_gold_ore→deepslate` 等 → b1/结构桶；本候选只声明「aquifer 语义」域，不认领 density 域差异。

## 4. 一锤定音的运行时探针（主会话执行）

> 目标：在**同一坐标**上对拍 aquifer **全链字段**（density 无关的字段），把「aquifer 内部是否等价」从静态推断升级为字段级实测。
> 前置（必做，否则结果不可读）：修 `AquiferDumpProbeMixin` 两处 —— ①:302 改 `Math.min(surfaceHeightEstimate, base + prnd)`；②新增 `-Daqdump.force=1` 时**跳过** :109-115 的 `density > 0` 早退（改为照常复算全链，仅把 `density` 字段按真实值打印）。

**P-A（决定性 · 字段级，推荐）**：同一 164 点集，两臂各跑一次，逐字段 diff。

```
# 1) 点集沿用既有（3 簇列 + 对照列，y=-34..6）：
#    .tmp/p7-aqdump-260908-10/points.txt
#    内容形如 `223 -34 237`（# 注释，x y z）

# 2) V 臂（1.21.6 vanilla + Java 探针；-PaqDump 已映射到 -Daqdump.probe=1，build.gradle:176-180）
cd runtime/1.21.6/java
gradle runServer -PcppVanilla=true `
  -PaqDump=true `
  -PaqDumpPoints=E:\PYTHON\CoreSwap\.tmp\p7-aqdump-260908-10\points.txt `
  -PaqDumpOut=E:\PYTHON\CoreSwap\.tmp\p7-aqdump-260908-10\java-forced-aqdump.txt `
  -PbenchSize=8 -PbenchOriginX=200 -PbenchOriginZ=200 *> ..\..\..\.tmp\p7-aqdump-java-forced-260908-10.log
#    （并在 vmArg 追加 -Daqdump.force=1；或在 mixin 里直接去掉早退）

# 3) R 臂（Rust 接管 + WG_AQDUMP；-PaqDumpRust → env WG_AQDUMP，build.gradle:208-212）
gradle runServer -PcppReplace=true `
  -PcppLib=E:\PYTHON\CoreSwap\target\release\worldgen1216.dll `
  -PcppWorldgenDir=versions/1.21.6/data/worldgen `
  -PaqDumpRust=E:\PYTHON\CoreSwap\.tmp\p7-aqdump-260908-10\points.txt `
  -PblockProbe=true -PblockProbeFull=true `
  -PbenchSize=8 -PbenchOriginX=200 -PbenchOriginZ=200 *> ..\..\..\.tmp\p7-aqdump-rust-forced-260908-10.log

# 4) 逐字段 diff（脚本可复用 .tmp/p7_aqdump_cmp_260908-10.py 的解析，扩到全字段）
#    比较键 = (x,y,z)；字段 = density,est,bl,f,opq,r,s,t,fl2,fl3,fl4,d,fq,gpq,cdE,e,cdG,g,cdH,h,barrier,floodedRaw,spreadRaw,spreadRnd,spreadBase,gate,erosion,depth
```

**判读标准**
- 全部字段逐点相同（允许 `decision` 字符串格式差：Java `null` ≡ Rust `null:*`/`-1`）→ **H-b2 实测驳倒**（aquifer 在该批单元逐位等价），残差归 density 桶。
- 任一字段系统性不等 → 该字段即偏离定位：`est` → 扫描域/DF/量化；`opq`/`r,s,t` → splitter/blob/排序；`fl2,fl3,fl4` → `get_fluid_level`/`get_fluid_block_y`/`get_fluid_block_state`；`d,fq,gpq,cdE,e,cdG,g,cdH,h,barrier` → `calculate_density`；`floodedRaw/spreadRaw/erosion/depth/gate` → 输入 DF 绑定。
- 特别核对点（本审计的偏离分支）：在 `block_y ≤ -10` 且 Rust `fl2` 的 blob 为 WATER 的点，看 Java `fl3`（= `getWaterLevel(s)`）与 Rust 是否同值 —— 若同值而 decision 不同，即 water-over-lava 分支实锤（Rust 提前 return WATER）。

**P-B（零改动 · 定性兜底）**：点集换到「两臂 d≤0」的区域做同链对拍（cluster 列地表以上 `y=64..120` step 4 + 1 列浅水如 `200 62 200`；此域 Java 探针不打 na）。
```
gradle runServer -PcppVanilla=true -PaqDump=true -PaqDumpPoints=<air-points.txt> -PaqDumpOut=<out> ...
gradle runServer -PcppReplace=true -PaqDumpRust=<air-points.txt> ...
```
- 同字段全等 → aquifer 在「空气/水」域的链等价（补强 P-A）；不等 → 立刻定位偏离字段。
- 局限：该域单元行（`m=floorDiv(y+1,12)`）与簇内深部单元不同，不能直接覆盖簇内单元（故 P-A 优先）。

**P-C（零改动 · 符号分层统计）**：把 P7 全量 mismatch 按「两臂 density 符号」分层（需 density 探针在 mismatch 点取值）——统计「同号但 decision 不同」的点数。当前 dump 观察为 0；若 >0，取该点做 P-A，即为 aquifer 偏离的直接铁证。
