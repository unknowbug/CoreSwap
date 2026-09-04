# scout-aquifer-map — aquifer 流体判定全链路管线地图（recode-scout，260904，只读勘探）

> 任务：residual-1830 水洞群 (~700 块, x196-216 / y14-30 / z236-242)：CS=封闭小型干洞窟, vanilla=大型水洞窟（aquifer 淹没判定/流体水平系统性偏低）。
> 分解：water→air 314 / stone→water 217 / water→stone 182 / air→water 170。
> 状态：draft（勘探产物，只列机制与分歧候选，不下结论）。

## 0. 参照源码定位（本地有 Java 源码，docs 引用齐全）

| 侧 | 文件 | 说明 |
|---|---|---|
| Java 权威 | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java` | yarn mappings，Impl 全文 452 行已逐行读 |
| Java 权威 | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/ChunkNoiseSampler.java` | estimateSurfaceHeight L222-240 + surfaceHeightEstimateCache L47 |
| Java 权威 | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java` | createFluidLevelSampler L78-84（default fluid level 语义） |
| Rust | `WorldgenRust/src/aquifer.rs` | apply/est/get_fluid_level 全链（494 行已逐行读） |
| Rust | `WorldgenRust/src/terrain.rs` L215-236 | VanillaAquifer::classify（-1→Rock、下界 seaLevel、skip） |
| Rust | `WorldgenRust/src/worldgen_handle.rs` L243-249, L503-515, L614-616 | 采样器构建/每 chunk 新建/carver 共享 |
| Rust | `WorldgenRust/src/carver.rs` L405-409 | carver getState → aquifer.apply(pos, 0.0) |
| C++ 前身 | `versions/1.20.1/cpp/worldgen/src/aquifer.h`（未复读，Rust 由其移植） | docs/04 + 07 有对拍史 |
| docs | `versions/1.20.1/docs/04-aquifer.md`, `07-block-pipeline.md`, `10-timewise-archive.md` | -288 时代大量已结案结论（Beardifier/carver 归因等） |

## 1. 管线地图（两侧逐行结构对照——已确认同构的部分）

### 1.1 采样器构建（输入侧）
- Java：`Impl` 构造直接取 `noiseRouter.barrierNoise()/fluidLevelFloodednessNoise()/fluidLevelSpreadNoise()/lavaNoise()/erosion()/depth()`（AquiferSampler.java L113-118）；est 用 `noiseRouter.initialDensityWithoutJaggedness()`（ChunkNoiseSampler L187）。randomDeriver = `split("minecraft:aquifer").nextSplitter()`。
- Rust：`worldgen_handle.rs` L243-249 从同一 router build_node 同 7 个 key（barrier/fluid_level_floodedness/fluid_level_spread/lava/erosion/depth/initial_density_without_jaggedness）→ `Aquifer::new(..., splitter, cx*16, cz*16, min_y, height)`（L503-506）。splitter 同源 `split_str("minecraft:aquifer")`。
- 结论：输入 key 集合与来源一致。**这些 4 采样器走树求值（非 transpiler 通道）**——两侧同构，但运行时数值未在本残差域做过点对点对拍。

### 1.2 default fluid level（apply 入口的 lava 分 + getFluidLevel 的 default）
- Java：lambda `(x,y,z) -> y < min(-54, seaLevel) ? FluidLevel(-54, LAVA) : FluidLevel(seaLevel=63, defaultFluid=water)`（NoiseChunkGenerator L78-84）。
- Rust：`FluidLevel::default_level(block_y)`：`y < -54 → {-54, LAVA} else {63, WATER}`（aquifer.rs L69）＋ apply 入口内联同语义（L298-299）。
- 结论：overworld 下语义相等（seaLevel=63、defaultFluid=water）；sea_level 硬编码在 aquifer.rs，与 settings 无联动（见分歧点 D1）。

### 1.3 apply 主判定（AquiferSampler.java L145-251 ↔ aquifer.rs L294-341）
逐行对照：density>0 → null/Rock(-1)（同）；3×3 邻域 blob（floorDiv(x-5,16), floorDiv(y+1,12), floorDiv(z-5,16)，u∈[0,1] v∈[-1,1] w∈[0,1]，`next_int_bound(10)/(9)/(10)`，pack 语义）——逐位同构；最近/次近/第三 blob 三元组维护（o/p/q, r/s/t）同；maxDistance `1-|a-i|/25` 同；calculate_density（lava-water 2.0 特判、j==0 早退、e>0 分支 1.5/2.5 与 3.0/10.0、|q|≤2 才采样 barrier、MutableDouble 缓存 barrier）逐常量一致。Rust `d<=0.0 → bs`、water-over-lava 特判（L322 ↔ Java L215）同。

### 1.4 getWaterLevel/getFluidLevel（water_levels 缓存）
- Java：`waterLevels[o]` 判 null → miss 调 getFluidLevel（L335-351）；`blockPositions` 用 Long.MAX_VALUE 哨兵（L133,177）。
- Rust：`water_levels` 用 `FluidLevel{y: i32::MAX}` 哨兵（L173,409）——真实 FluidLevel.y 不可能为 MAX（getFluidBlockY 只产 default.y / -32512 / min(est,q)，est=MAX 时 min(MAX,q)=q）→ 哨兵无碰撞。get_block_pos 用 i64::MAX 哨兵（L172,281）同 Java。
- 结论：缓存机制同构。

### 1.5 getFluidLevel 13 邻域（L353-389 ↔ aquifer.rs L416-442）
CHUNK_POS_OFFSETS 13 项逐项一致（aquifer.rs L417 ↔ Java L99-101）；bl2（中心）`k>o → return default`；bl3 `j>o` 且非 air → return fl2；bl 标记；`i = min(est)`——控制流逐行一致。

### 1.6 estimateSurfaceHeight（est）
- Java（ChunkNoiseSampler L222-240）：坐标先 4 量化（BiomeCoords.fromBlock/toBlock = (x>>2)<<2），`surfaceHeightEstimateCache = Long2IntOpenHashMap`（L47，**per ChunkNoiseSampler = per chunk、无界**）；扫描 `l = min_y+height; l >= min_y; l -= verticalCellBlockCount(=4×size_vertical=8)`，`initialDensityWithoutJaggedness.sample(UnblendedNoisePos(4量化x, l, 4量化z)) > 0.390625` → 返回 l；**扫不到返回 Integer.MAX_VALUE**。
- Rust（aquifer.rs L343-377）：同样 4 量化（L344）、步长 -8（L367）、阈值 0.390625（L366）、扫不到返回 i32::MAX（L360 初值）→ **逐位同语义**。缓存：per-chunk 32×32 窗口（off 12/4，L77-79,345-348）+ 跨 chunk EstL2 精确值缓存（b1-b，L2 默认启用）——**L2 是纯函数精确值缓存，重算同值，语义零差**（BLEND_ACTIVE=false 闸门在位；blend 类 DF 皆常数，density.rs:626-628）。
- ⚠️ 澄清（对任务描述的修正）：**Java/Rust 两侧 est 都是「自顶向下扫描」，不存在 4 角插值**——「sh4 4 角」是 C++ 旧实现，docs/10 L513 已记「est 从 4 角插值改为扫描」且 L102「est 两版一致（同 seed）」。本残差域「est 插值输入差」假说按 docs 已结案，除非本 seed 域未覆盖。

### 1.7 getFluidBlockY / getNoiseBasedFluidLevel / getFluidBlockState
- erosion<-0.225 && depth>0.9 → d=e=-1（Java 走 `VanillaBiomeParameters.method_43718`，docs/04 追加2：float 常量比较已对齐；Rust L447 `-0.225f32 as f64` / `0.9f32 as f64`）；floodedness clamp±1；f = bl ? clampedMap(ii,0,64,1,0) : 0；map2(-0.3,0.8 / -0.8,0.4)；d/e 判定顺序（e>0→default / d>0→noise-based / else -32512）逐行一致（L391-419 ↔ L444-460）。
- getNoiseBasedFluidLevel：floorDiv/16, /40, n=l*40+20，spread×10 → roundDownToMultiple(,3)，min(est, q)——逐行一致（L421-433 ↔ L462-472）。
- getFluidBlockState：`fluid_level <= -10 && != -32512 && state != LAVA` → floorDiv(64,40,64) 采 lava noise，|d|>0.3 → LAVA——一致（L435-450 ↔ L474-485）。**注意：水洞 y14-30 域 fluid_level≈40-70，此 lava 分支大概率不触发**。

### 1.8 缓存/实例生命周期
- Java：`ChunkNoiseSampler`（含 AquiferSampler.Impl、surfaceHeightEstimateCache、waterLevels、blockPositions）**per chunk 新建**（NoiseChunkGenerator L95-97/102-111 getOrCreateChunkNoiseSampler），在 NOISE→SURFACE→CARVERS 各 status 间**同 chunk 复用**。
- Rust：`fill_chunk_blocks` 每 chunk 新建 Aquifer（L503），carver 同调用内共享 `&mut va.aq`（L614-616）→ 单次生成调用内生命周期等价。差异：**Rust 跨调用零残留**（waterLevels/blockPositions/est 窗口每调用冷）＋ EstL2 跨 chunk 共享（est 值精确，仅性能）。
- 结论：**缓存生命周期差异不产生语义差**（所有缓存均为精确值缓存），与「系统性偏低」的机制方向不符——偏低必须来自输入数值本身。

### 1.9 barrier(-1)→Rock 边界
- Java：apply 返回 null → ChainedBlockSource（aquifer null → oreVein → 默认 stone）。
- Rust：terrain.rs L232-234 `-1 → BlockKind::Rock`（260904-07 修复，对齐 C++ worldgen_api.cpp:1041-1043），ore_vein 仅在 Rock 替换（L548-551）。边界一致。

### 1.10 阶段顺序
Java：NOISE → SURFACE → CARVERS（→FEATURES）。Rust：fill_chunk(+ore_vein) → build_surface → carver（L490 注释、L614）。顺序一致，carver 与 Java 同在 surface 后。

## 2. 分歧点清单（每项带文件:行号 + 置信度）

> 总判断：**公式/结构层逐行对拍全部一致**（本图 §1 各节）。系统性的「淹没判定偏低」不可能由结构差异产生，只能来自**运行时输入数值**（树求值采样值）或**尚未对拍的坐标/边界情形**。以下按嫌疑排序。

| # | 分歧点 | 两侧落点 | 机制说明 | 置信度（作为根因候选） |
|---|---|---|---|---|
| **D1** | **floodedness/spread 树求值运行时值差** | Java AquiferSampler.java L402/429 ↔ aquifer.rs L452/468 输入 `self.flooded/spread`（worldgen_handle.rs L244-245 build_node 树） | e=d>0 → noise-based 液面；e>0 → default(63/air)。**floodedness 偏低 → e=g-h≤0 且 d=g-k≤0 → -32512 无效液面 → AIR（干洞）**——与「CS 干、vanilla 水」签名直接吻合。静态 17 项公式零偏离 ≠ 树求值零偏离（本路径未做逐点对拍） | **高（首选）** |
| **D2** | **est 数值差（initial_density_without_jaggedness 树求值）** | ChunkNoiseSampler.java L228-240 ↔ aquifer.rs L360-376 | est 偏低 → `k>o`（中心列 est+8 < blockY-12）早退 default（y14-30 → default=water@63？不，`k=o-…` 早退返回 default_fl = WATER@63，反而多水）；est 偏高 → getNoiseBasedFluidLevel 的 min(est,q) 偏高 → 液面偏高（多水）… 两个方向都需数值判别。同时 est 进 `bl/f` 淹没权重 | **中高** |
| **D3** | **erosion/depth 门（method_43718）运行时值** | Java L395 ↔ aquifer.rs L447 | 门命中（erosion<-0.225 && depth>0.9，即河/谷低地）→ d=e=-1 → 无效液面 AIR。树求值 erosion/depth 在水洞域若两侧差 → 淹没翻转。静态阈值已对齐（float 提升），采样值未对拍 | **中** |
| **D4** | **blob 邻域选择差（get_block_pos 随机偏移/split 链）** | Java L180-182 ↔ aquifer.rs L283-289 | -288 时代 Python 复现 8/8 逐位一致（docs/10），但本 seed/域未复验；r/s/t 选错 blob → 液面/距离场差 → 干/水互换（water→air 与 air→water 双向并存与「fl3/fl4 三元组差」签名吻合：本残差 water→air 与 air→water 双向共存） | **中** |
| **D5** | **density 输入 d 差（宏观插值 vs 逐点）** | Rust terrain.rs L275 宏观 chunk_density.sample ↔ Java InterpolatedDF 网格 | `density + e > 0 → -1(stone)` 与 `d<=0 → bs` 的边界对 d 敏感；若 Rust 宏观网格插值与 Java 网格在水洞域差 → water↔stone 双向（本残差 stone→water 217 / water→stone 182 双向并存） | **中** |
| **D6** | **default_fl 联动缺失（seaLevel/defaultFluid 硬编码）** | Java NoiseChunkGenerator L78-84（settings 驱动）↔ aquifer.rs L69（硬编码 -54/63/WATER） | overworld settings 恒 63/water → 当前零语义差；仅作防御性记录，**不解释本残差** | 低（当前无关） |
| **D7** | **EstL2 跨 chunk 缓存** | aquifer.rs L86-128, L349-375 | 纯函数精确值缓存 + BLEND_ACTIVE=false；若未来 blend 激活或 init 树含非纯状态（如内部缓存 DF 带 per-chunk 语义）→ 可能污染。当前判定语义安全 | 低（已闭） |
| **D8** | **UnblendedNoisePos vs NoisePos** | Java L392/429/443（Unblended）↔ Rust NoisePos（L445/467/480） | blend 关闭时二者等价（Rust blend 类 DF 全常数，density.rs:626-628）；blend 激活才分化 | 低（当前无关） |
| **D9** | **per-chunk 水位/blob 缓存冷语义** | Java per ChunkNoiseSampler 复用 ↔ Rust 每调用冷 | 均为精确值缓存，仅性能（Q-AQ1 课题）；无语义差 | 低（已闭） |

### 关键结构性观察（供定位阶段收敛）
残差四分（water→air 314 / air→water 170 / stone→water 217 / water→stone 182）**双向并存**。单向偏移（如仅 floodedness 低）应产生单向签名；双向并存指向：① blob 三元组选择差（D4，fl2/fl3/fl4 组合不同 → 有的点判水有的点判干）；② 或 e 值在 0 附近震荡的输入微差（D1/D2 的小幅数值差在阈值边界双向翻转）；③ 或 density 边界（D5）。**「系统性偏低」的任务假设与双向签名存在张力**——定位阶段应先用判别探针分辨「单向漂移」vs「边界震荡」。

## 3. 判别探针设计（每分歧点）

| 探针 | 判别目标 | 设计 |
|---|---|---|
| P-D1（floodedness/e 值 dump） | D1 | 在水洞域选 N 个分歧点（如 (200..216, y16..28, 236..242) 内 diff cell 中心），两侧分别 dump `fluid_floodedness.sample(blockX,blockY,blockZ)` 原值 + 13 邻域 est + `bl` + `f/h/k` + `d/e` + fl2/fl3/fl4 的 (y, block) 与最终判定。Java 侧走 DensityProbe 扩展（反射 getFluidLevel 私有链，注意 -288 AQF-DUMP 先例：反射 getWaterLevel 可信、CellCache 反射不可信）。逐点 diff → 单向偏移（均值系统性偏）vs 边界震荡（差值 ±0.0x 级随机符号）二分 |
| P-D2（est dump） | D2 | 同域 4 量化列，两侧 `estimateSurfaceHeight` 列值直采（Java surfaceHeightEstimateCache / Rust est+L2）对比；顺带验扫不到→MAX 路径 |
| P-D4（blob 三元组 dump） | D4 | 同点 dump r/s/t（pack 解包后 blob 坐标 + o/p/q 距离）逐位对比——o/p/q/r/s/t 任一不同即 D4 实锤；全同则 D4 出局 |
| P-D5（d 值 dump） | D5 | 同点 finalDensity（含 Beardifier）两侧对比（-288 已有 AQF-APPLY 同构探针可复用，换坐标） |
| P-D3（erosion/depth 门） | D3 | P-D1 的 dump 里附带 erosion/depth 样值 + 门命中布尔，一行即可判 |
| 隔离臂 | 全体 | `WG_SKIP_AQUIFER`/`WG_SKIP_OREVEIN`/carver 双臂已有 A/B bench 先例（qaq1_b2），可在本域重跑 2×2 定位差异归属阶段（fill-aquifer vs carver） |

执行顺序建议：P-D4（最廉价、纯整数逐位）→ P-D1（主判别）→ P-D2/P-D5 视 P-D1 结果补。seed 三查铁律适用于所有探针（先核 worldSeed 一致）。

## 4. deepslate→air 106（carver 域耦合，一行）

deepslate 域（y<0）不在本水洞残差带（y14-30），但同一耦合点：Rust carver getState（carver.rs L405-409）= Java Carver——`y<=lavaLevel(minY+8=-56)→lava` 否则 `aquifer.apply(pos,0.0)`，即 **carver 挖掉的 deepslate 格由 aquifer@density=0 决定 air/water**——若 aquifer 液面判定偏低，carver 域同样会少水多 air（deepslate→air 与水洞族可能是同一根因的 carver 侧投影）；106 块量级小，建议随 P-D1 顺带验证，不单独立案。

## 5. 待深入点 / 边界备注

- D1/D2/D3 共同前提「树求值运行时值未在本域对拍」是本次勘探唯一确认的**空白域**（静态公式已零偏离，运行时值无记录）。
- `VanillaBiomeParameters.method_43718` 本地未展开读（docs/04 记载阈值已对齐）——如 P-D3 命中需回读该方法源码（mc_src_extract 下应有）。
- Rust `estimate_surface_height` 的 32×32 窗口在 13 邻域外沿（off=-3 列）`in_c=false` 会绕过窗口缓存（仅性能）；L2 put 不受 in_c 门控（L371-375），无遗漏。
- water_levels 哨兵碰撞已论证安全（§1.4）。
