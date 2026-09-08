# W1: density/noise 域语义级 diff（1.20.1 → 1.21.6, yarn 源码）

- 状态: **draft**（静态审查，Degraded 分层——无运行时探针）
- 方法: `git diff --no-index -U8` 逐文件全量 diff + 关键点源码核对（MathHelper 签名等）
- 原始 diff: `cmd-output/w1-full.diff`（同目录）
- 日期标签: 260908-08 块（沿用任务命名）
- 分类体系: 算法级 / 数据级 / 装饰级 / 接口级

## 总览（先说结论）

**density/noise 数学本体（multi_noise / spline / 插值 / DoublePerlinNoise / Beardifier / OreVein）在 1.21.6 中零算法变更。**
唯一实质算法变更集中在 **AquiferSampler**（aquifer 邻居追踪 3→4 个 + needsFluidTick 判定语义重写）与 **BlendingData.getSurfaceBlockY 的检查顺序**（实际核对后语义等价）+ **Blender.hypot 走 float 重载**（精度路径变更，权重不可观测级差异）。web 勘探「density/noise 本体未变」结论**成立**；1.21.2 chunk 管线重构体现在 `populateNoise/populateBiomes/carve` 签名与 Executor 移除，属接口级，归 W2。

---

## 1. NoiseConfig.java（noise/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 1.1 | `new Identifier("aquifer"/"ore"/"terrain")` → `Identifier.ofVanilla(...)`（3 处） | **装饰级** | 静态工厂替代构造器，产生的 namespace:path 字符串完全相同 → **randomDeriver.split 的种子派生字符串不变**，Rust 端 split 派生逻辑无需改动 |

其余（LegacyNoiseDensityFunctionVisitor、usesLegacyRandom 分支、surfaceBuilder 构造）逐行一致。

**文件裁决：对 Rust 引擎零影响。**

## 2. NoiseParametersKeys.java（noise/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 2.1 | `of()` 内 `new Identifier(id)` → `Identifier.ofVanilla(id)` | 装饰级 | 同上 |

注意：diff 仅 1 行——**两版注册键列表完全相同，没有任何 noise parameters key 增删**。若 1.21.6 有新 noise 参数，不在本文件（本辖区证据：无）。

**文件裁决：零影响。** noise 参数数值本身在数据包 JSON（`worldgen/noise`），归数据侧核对（非本文件）。

## 3. DensityFunctions.java（densityfunction/）

| # | 变更 | 分类 |
|---|------|------|
| 3.1 | `of()` 内 Identifier 构造 → `ofVanilla` | 装饰级 |

**全部 overworld/caves/nether/end density function 的构建代码（bootstrap、CaveScaler、YClampedGradient 引用等）逐行一致。** 无任何默认值/公式变化。

**文件裁决：零影响。**

## 4. DensityFunctionTypes.java（densityfunction/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 4.1 | `registerAndGetDefault(Registry<Codec<...>>)` → `Registry<MapCodec<...>>`，`register()` 返回类型 Codec→MapCodec | 接口级 | DF codec 注册表改为 MapCodec 注册（DFU 升级配套），序列化格式（JSON tag 名）不变，行为不变 |
| 4.2 | `BinaryOperation.sample()` switch 中 MAX/MIN/MUL case 顺序重排（ADD,MAX,MIN,MUL → ADD,MUL,MIN,MAX） | 装饰级 | 每个分支表达式逐字符相同，switch 顺序无语义 |
| 4.3 | `BinaryOperation.fill()` 四个 case 块顺序重排（MAX,MUL,MIN → MUL,MIN,MAX） | 装饰级 | 各分支体逐行相同 |
| 4.4 | `BinaryOperationLike.create()` 中 min/max switch case 重排 | 装饰级 | 同上，MUL 的区间算式逐字符相同 |
| 4.5 | 多处构造器参数加 `final`（Type/RarityValueMapper/UnaryOperation.Type 等） | 装饰级 | 反编译器风格差异 |
| 4.6 | `RarityValueMapper.CODEC` 声明 `com.mojang.serialization.Codec` → 简化导入 `Codec` | 装饰级 |

**spline / interpolate / weird_scaled / beardifier / interpolated (old_blended_noise) 等类型定义零变更。**

**文件裁决：零影响。** Rust 端 BinaryOperation 的 min/max/mul 短路优化逻辑照旧。

## 5. ChunkNoiseSampler.java（chunk/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 5.1 | blockStateSampler 组装：`ImmutableList.builder()` → `ArrayList` + 显式 cast + `ChainedBlockSource(list.toArray(...))` | 装饰级/接口级 | sampler 链构造方式变，链内容不变（aquifer → 可选 oreVein），顺序不变 |
| 5.2 | `interpolators.forEach(lambda)` → 显式 for 循环（4 处：onSampledCellCorners/interpolateY/X/Z） | 装饰级 | 无语义差异 |
| 5.3 | 内部类构造器加 `final`（CacheOnce/CellCache/DensityInterpolator/FlatCache） | 装饰级 |

**核心插值机制（DensityInterpolator 4 角缓冲、FlatCache、CellCache、CacheOnce 的 uniqueIndex 失效逻辑、cell 尺寸计算）逐行一致。**

**文件裁决：零影响。** Rust 的 4 角插值/cell cache 语义照旧。

## 6. AquiferSampler.java（chunk/）★ 本域唯一实质算法变更

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 6.1 | `FluidLevel` 类 → `record FluidLevel(int y, BlockState state)` | 装饰级 | 等价重构；record 的 equals 用于 6.3 的新判定 |
| 6.2 | **邻居搜索从追踪最近 3 个 fluid cell 点扩展到最近 4 个**：距离 `o,p,q` + 第四个 `r`(int dist)，位置 `s,t,u,v`；shift 链相应多一层 | **算法级** | 结构性重写。但**用于 density 计算的配对集合不变**：仍是 (1st,2nd)/(1st,3rd)/(2nd,3rd) 三对 calculateDensity，maxDistance 配对 (o,p)/(o,q)/(p,q) 与 1.20.1 完全相同——第 4 点只喂给 needsFluidTick 判定。**地形形状（返回的 BlockState/density 抑制）不变** |
| 6.3 | **needsFluidTick 判定语义重写**：① `d<=0` 分支从 `needsFluidTick = d >= THRESHOLD` 改为「达到阈值**且**第 1/第 2 邻居 FluidLevel 不同才 true，否则 false」；② 收尾分支从无条件 `true` 改为三条件（(1,2) 层不同 ∨ (2,3) 距离达阈值且层不同 ∨ (1,3) 距离达阈值且层不同），全不满足时再看 (1,4) 组合 | **算法级** | 影响的是流体 **scheduled tick 触发频率**（泉水/含水层流动表现），**不改变方块放置本身**。Rust 若复刻了 needsFluidTick 语义（影响存档 tick 队列）需移植；只复刻地形形状则可忽略 |
| 6.4 | `VanillaBiomeParameters.method_43718(...)` → 具名 `inDeepDarkParameters(...)` | 装饰级 | deep dark 流体禁用判定逻辑不变（`d=e=-1.0` 分支一致） |

**aquifer 核心数学未动**：fluidLevelFloodedness/aquiferBarrier/aquiferAquifer 采样、clampedMap、`i-5/16`、`j+1/12` 的 cell 网格、`x*16+nextInt(10), y*12+nextInt(9)` 随机点派生——全部逐行一致。

**文件裁决：算法级但低影响——地形形状零变化；仅 needsFluidTick 语义变化，视 Rust 是否复刻 tick 行为决定是否移植。**

## 7. GenerationShapeConfig.java（chunk/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 7.1 | `trimHeight`: `world.getTopY()` → `world.getTopYInclusive() + 1` | 接口级 | 1.21.2 HeightLimitView API 改名（exclusive top 移除），`getTopYInclusive()+1 == 旧 getTopY()`，数学等价 |

**文件裁决：零影响。** min_y=-64/height=384 等 overworld 默认值未变（本文件内无数据级变更）。

## 8. NoiseChunkGenerator.java（chunk/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 8.1 | `CODEC`: `Codec`+`RecordCodecBuilder.create` → `MapCodec`+`mapCodec`；`getCodec()` 同 | 接口级 | 序列化格式不变 |
| 8.2 | `populateBiomes(Executor, ...)` → 去掉 Executor 参数；`Util.debugSupplier("init_biomes",...)` → `executor.named("init_biomes")` | 接口级 | 1.21.2 管线重构：executor 由调度器管理；仍跑在 mainWorkerExecutor，语义等价 |
| 8.3 | `populateNoise(Executor, ...)` → 去掉 Executor；section unlock 从 `whenCompleteAsync(executor)` 改为同 future 内 `try/finally` | 接口级 | 锁释放时机从回调挪到同任务 finally，行为等价（甚至更严格） |
| 8.4 | `carve(...)` 去掉 `GenerationStep.Carver carverStep` 参数；`getOrCreateCarvingMask(carverStep)` → `getOrCreateCarvingMask()`；`getCarversForStep()` 无参 | **数据级/管线级**（归属建议：**W2 chunk 管线**） | 1.21.2 合并 carving step（AIR/LIQUID 两步合一，每 chunk 单张 carving mask）。carver 种子派生 `setCarverSeed(seed + l, ...)`、-8..8 邻域循环逐行不变 → carve 数学不变，变的是 step 组织方式。**超出 W1 辖区，只标归属不展开** |
| 8.5 | `getDebugHudText` → `appendDebugHudText` | 装饰级 | 更名 |
| 8.6 | `region.getRegistryManager().get(...)` → `getOrThrow(...)`（2 处）；`region.getTopY()-1` → `getTopYInclusive()` | 接口级 | 等价 |
| 8.7 | populateBiomes 私有方法内部（MultiNoiseSampler 构造、BelowZeroRetrogen、aquifer/beardifier/sampler 装配） | 无 diff | 完全一致 |

**文件裁决：noise 填充数学零变化；签名/调度变更归 W2 管线。carve step 合并归 W2。**

## 9. ChunkGenerator.java（chunk/，仅 noise/sampler 相关部分）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 9.1 | `getCodec()/getCodecKey()` Codec→MapCodec | 接口级 | 同 8.1 |
| 9.2 | 抽象 `populateNoise/populateBiomes` 去 Executor；`carve` 去 carverStep | 接口级 | 归 W2 |
| 9.3 | `setStructureStarts(...)` 新增 `RegistryKey<World> dimension` 参数，透传到 `structure.createStructureStart(...)` | 接口级（结构域） | 结构生成域，非 noise/sampler——1.21.5+ 结构按维度判定相关。归属建议：结构移植项，W1 不展开 |
| 9.4 | `getStructurePresence(...)` 新增 `placement` 参数 | 接口级（结构域） | 同上 |
| 9.5 | `appendDebugHudText` 更名；`initializeIndexedFeaturesList()` 新增公开方法 | 接口级 | 后者是管线预热钩子，归 W2 |
| 9.6 | getHeight/getHeightOnGround/getHeightInGround、sampleHeightmap 路径 | 无 diff | noise 高度图采样数学一致 |

**文件裁决：noise/sampler 相关部分零变化；其余签名变更分别归 W2（管线）与结构域。**

## 10. Blender.java（chunk/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 10.1 | `blendHeight`: `MathHelper.hypot(i-biomeX, j-biomeZ)`（int→double 提升）→ `hypot((float)(...), (float)(...))`——**命中 1.21.6 新增的 `hypot(float,float)` 重载**（已核对 MathHelper：新版同时保留 double 版并新增 float 版） | **算法级（浮点精度路径）** | 距离平方和改在 float 域累加再 sqrt。输入是 int 差值（≤ ~50），float 域内平方和精确（无舍入损失），结果与 double 版**位级一致概率极高**；严格非逐位等价，实际不可观测 |
| 10.2 | `blendDensity`: `MathHelper.magnitude(int,int,int)` → `magnitude((double), (double), (double))` | 装饰级 | 原本就经 double 提升，显式化，位级等价 |
| 10.3 | `blendBiome`: 同 10.1 的 float hypot | 算法级（精度路径，同 10.1） | 同上 |
| 10.4 | `setBlendingMask`: `Stream.of(GenerationStep.Carver.values()).map(getOrCreateCarvingMask)` → 单 `chunk.getOrCreateCarvingMask()` | 接口级（归 W2） | 配合 carve step 合并；mask predicate 数学（OFFSET_NOISE 采样、<4.0）逐行不变 |
| 10.5 | `tickLeavesAndFluids`: `getTopY()-1` → `getTopYInclusive()` | 接口级 | 等价 |

**文件裁决：老世界 blending 权重数学实质不变（10.1 理论精度路径变更，判定为不可观测）；mask 组织变化归 W2。**

## 11. BlendingData.java（chunk/）

| # | 变更 | 分类 | 说明 |
|---|------|------|------|
| 11.1 | 序列化重构：`BlendingData.CODEC` + `validate` 移入新 `record Serialized`，新增 `fromSerialized()/toSerialized()`；`optionalFieldOf` → `lenientOptionalFieldOf`；**`max_section` 序列化值从 `getTopSectionCoord()` 改为 `getTopSectionCoord()+1`** | **数据级**（序列化格式） | 存档中 blending 数据的 `max_section` 字段值整体 +1（旧字段语义 exclusive→inclusive 迁移）；lenient 使畸形 heights 不再硬失败。**只影响旧世界升级读取，不影响新世界生成**。Rust 若写存档需对齐；只读参照可忽略 |
| 11.2 | `getBlendingData`: `chunk.method_51526().isAtLeast(BIOMES)` → `!chunk.getMaxStatus().isEarlierThan(BIOMES)` | 接口级 | 逻辑等价 |
| 11.3 | `getSurfaceBlockY`: 高度图取值 `min(sample+1, getTopY())` → `min(sample, getTopYInclusive())`；扫描循环「先 move(DOWN) 再检查」→「先检查再 move(DOWN)」 | **装饰级（经核对等价）** | 两种写法首个被检查的 Y 相同（都是 sample 高度本身），cap 边界等价（exclusive/inclusive 换算抵消）——语义无变化，疑似对旧 off-by-one 风险的显式化重写 |
| 11.4 | `calculateCollidableBlockDensityColumn` 起点 `getTopY()` → `getTopYInclusive()+1` | 接口级 | 数值相同，等价 |
| 11.5 | `acceptBiomes`: `biomeY < fromBlock(getTopY())` → `biomeY <= fromBlock(getTopYInclusive())` | 装饰级（经核对等价） | `(incl+1)>>2 == (incl>>2)+1` 恒成立，两条件集合相同 |
| 11.6 | `HORIZONTAL_BIOME_COUNT` private→package-private | 装饰级 |
| 11.7 | heights 判空：`DoubleStream.of(...).anyMatch(...)` → 循环 `bl` 标志；`DoubleArrays.copy` 防御性拷贝 | 装饰级 |

**文件裁决：新世界生成零影响；唯一实质点 11.1 是存档序列化格式变化（旧世界升级路径），归数据/存档兼容侧。**

---

## 1.21.6 移植影响裁决表（W1: density/noise 域）

| 文件 | 算法级 | 数据级 | 接口级 | 装饰级 | Rust 引擎动作 |
|---|---|---|---|---|---|
| NoiseConfig | — | — | — | Identifier.ofVanilla | **无**（split 种子字符串不变） |
| NoiseParametersKeys | — | — | — | ofVanilla | **无**（键表零增删） |
| DensityFunctions | — | — | — | ofVanilla | **无** |
| DensityFunctionTypes | — | — | MapCodec 注册表 | switch 重排/final | **无** |
| ChunkNoiseSampler | — | — | ChainedBlockSource 数组 | ArrayList/显式循环 | **无** |
| AquiferSampler | **6.2 邻居 3→4；6.3 needsFluidTick 重写** | — | — | record | **地形形状无需改**；若复刻 tick 队列 → 移植 6.3 判定 |
| GenerationShapeConfig | — | — | TopY API 改名 | — | **无** |
| NoiseChunkGenerator | — | — | Executor 移除；carve step 合并（→W2） | 更名 | **W1 内无**；carve 组织归 W2 |
| ChunkGenerator | — | — | MapCodec；setStructureStarts+dimension（结构域） | — | **W1 内无** |
| Blender | **10.1/10.3 float hypot 精度路径（判定不可观测）** | — | 单 carving mask（→W2） | — | **无**（建议保留 double 实现 + 备注差异声明） |
| BlendingData | — | **11.1 max_section 序列化值 +1** | Serialized record 拆分 | 扫描顺序重写（等价） | **无**（写存档才需对齐） |

### 裁决总结

1. **Rust 已逐位复刻的 1.20.1 density/noise/spline/插值/aquifer-est/surface-rule 数学在 1.21.6 全部原样**——移植时这部分引擎代码**零改动**，换数据（noise parameters / density function JSON，另案核对）即可，符合数据驱动架构。
2. **唯一需要决策的算法点 = AquiferSampler needsFluidTick 语义**（6.3）：只影响流体 tick 调度，不影响方块形状；若 Rust 引擎不含 tick 复刻则完全免移植。
3. **Blender float-hypot** 为理论上非逐位、实际不可观测的精度路径变更，建议 Rust 维持 double 实现并在 anchor 注记该已知差异（@anchor.idk 级声明）。
4. chunk 管线（Executor 移除、carve step 合并、populateNoise 调度、Serialized 拆分）→ **W2**；结构 dimension 参数化 → 结构域另案。
5. 置信度 **draft**：静态源码 diff（Degraded 分层），未经 Java 1.21.6 运行时探针验证；建议后续若真做移植，用 1.21.6 block_probe 类探针抽验 aquifer/needsFluidTick 与 blending 两点。

*草稿次序差异说明：NoiseConfig MD5 两版不同即源本项目预估的 1.1 类变更，实际仅 ofVanilla 静态工厂替换。*
