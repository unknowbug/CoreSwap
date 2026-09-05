# scout2-random-derivation — feature 放置随机派生链两侧对照（scout2，只读勘探）

> 编号：scout2（区别于首轮 scout-random-chain-260905-05——该轮时一手 Java 源尚未入工作区、tree 族未实现）。
> 置信度：**draft**。验证分层：**Degraded→Partial 静态**（本轮一手源核对 = `versions\1.20.1\data\mc_src_extract\` 静态对拍，无运行时探针）。
> 状态基线：**Phase 4a/4b 已应用后的工作区现状**（tree.rs 新建、placement.rs/carver.rs/feature_loader.rs/worldgen_handle.rs 已改，未提交——对照对象是含 patch 的代码，不是 HEAD）。
> 只读勘探：不改任何代码；不判 populationSeed 挂起项。

## 0. 一手源可用性声明（本轮最大增量）

首轮 scout（scout-random-chain §6）标注「Java 一手源不在工作区、docs 引用为二手记录」。**现状已变**：`versions\1.20.1\data\mc_src_extract\`（S1 一手源，phase4-status #2 用户指路）含全套 random 类与调度类。本轮派生链 Java 侧全部改为**一手 file:line 核对**：

- `util/math/random/ChunkRandom.java`（setPopulationSeed L54-61 / setDecoratorSeed L75-78 / setCarverSeed L87-93 / next(bits) L29-32 / setSeed L35-39）
- `util/math/random/BaseRandom.java`（nextInt(bound) L15-30 / nextLong L33-38 / nextDouble L51-56）
- `util/math/random/Xoroshiro128PlusPlusRandom.java`（setSeed→createXoroshiroSeed L47-50 / next(bits) L116-118）
- `util/math/random/RandomSeed.java`（createXoroshiroSeed L23-31 / mixStafford13 L17-21 / 常量 L11-12）
- `world/gen/chunk/ChunkGenerator.java` generateFeatures L334-412（结构+feature 双循环）
- `world/gen/feature/PlacedFeature.java` generate L48-63（flatMap 惰性）
- `world/gen/feature/util/PlacedFeatureIndexer.java` collectIndexedFeatures L61-156 + lastIndexGetter L154

## 1. 派生链两侧对照表（Java 步骤 ↔ Rust 实现/缺失）

| # | Java 步骤（一手源 file:line） | Java 语义要点 | Rust 实现位置 | 对齐状态 |
|---|---|---|---|---|
| D1 | ChunkGenerator.java L343：`new ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))` | FEATURES 阶段基类 = Xoroshiro（非 carver 的 CheckedRandom LCG）；构造 seed 为 `RandomSeed.getSeed()`（uniquifier+nanoTime）——**值无关紧要**，L344 setPopulationSeed 立即重播种 | worldgen_handle.rs L862 `ChunkRandom::xoroshiro()` | ✅（构造 seed 差异被 set_population_seed 的 set_seed(worldSeed) 覆盖，无影响） |
| D2 | ChunkGenerator.java L344：`setPopulationSeed(world.getSeed(), blockPos.getX(), blockPos.getZ())`，blockPos = chunk 首块 (cx*16, cz*16) | ChunkRandom.java L54-61：setSeed(worldSeed); l=nextLong()\|1; m=nextLong()\|1; n=(blockX*l + blockZ*m) ^ worldSeed; setSeed(n) | chunkrandom.rs L148-155 + worldgen_handle.rs L864 | ✅ 公式逐项一致（含 `\|1`、`^` 优先级：Java `a*b+c*d ^ e` 中 `^` 低于乘加，Rust L152 括号等价）。**populationSeed 精度判定本身属永久挂起域，此处只声明公式静态对齐，不判逐位等价** |
| D3 | ChunkRandom nextLong（BaseRandom.java L33-38）= `(next(32)<<32) + next(32)`，next(bits) 经 ChunkRandom L31 按基类分发 | Xoroshiro 基类时 next(32) = `(int)(nextLong() >>> 32)`（Xoroshiro128PlusPlusRandom.java L116-118），即 2 次 xoroshiro 步取高 32 位；有符号拼接（MC-239059） | chunkrandom.rs L96-109（next/next_long） | ✅（`>>` 算术移位 + `as i32` 截断与 Java `(int)(x >>> (64-bits))` 在 bits≤32 时逐位等价——移位量 ≥32，移位结果落在 int 值域，符号差异被截断消除） |
| D4 | Xoroshiro setSeed（Xoroshiro128PlusPlusRandom.java L47-50）= `new Impl(RandomSeed.createXoroshiroSeed(seed))` | RandomSeed.java：unmixed = (seed^SILVER, lo+GOLDEN) → mixStafford13×2；零种子卫兵 Impl L31-34 | xoroshiro.rs L6-30 + L43（mix_stafford13 常量 0xBF58476D1CE4E5B9/0x94D049BB133111EB = Java 负常量无符号形式，unmixed ^SILVER 0x6A09E667F3BCC909 +GOLDEN 0x9E3779B97F4A7C15，零卫兵同序） | ✅ **首轮 C8 疑点就此闭合**（首轮时无从核对，本轮一手坐实路径与常量一致） |
| D5 | ChunkGenerator.java L345-353：3×3 邻 chunk 各 section biome 并入 `ObjectArraySet`，`retainAll(biomeSource.getBiomes())` | feature 候选集 = **3×3 邻域全部 biome** 的并集（无序 set） | worldgen_handle.rs L857-858：**只用当前 chunk 角 biome**（cur_biome_id）取 features | ⚠️ **疑点 N1**（见 §3）——邻域 biome 并集 vs 单 biome；影响 feature 成员集，不影响已放 feature 的随机序列 |
| D6 | ChunkGenerator.java L382-395：`IntSet`（IntArraySet）收集 set 内各 biome features[k] 的 `indexMapping(p)` → `toIntArray()` → `Arrays.sort()` 升序遍历 | 每 step 按 p 升序放置；p = lastIndex | worldgen_handle.rs L871-879 + feature_loader.rs `int_set_for`（L152-167 排序后返回） | ✅ 排序语义一致；**indexMapping 本体见 D7** |
| D7 | PlacedFeatureIndexer.java L84（featureIndex 首现递增）+ L154（`Util.lastIndexGetter(features,…)`）+ L140-147（拓扑排序后按 step 过滤） | p = feature 在**该 step 全局列表**中的**最后出现索引**（indexMapping = lastIndexGetter，map.put 覆盖语义） | feature_loader.rs L83-150（PlacedFeatureIndexer::build：last_index_map lastIndex + step_features 按 featureIndex 升序分组） | ✅ 结构一致（F-3 修复后全局构建）；⚠️ 逐位等价仍无独立数值验证记录（沿用首轮 C6，**非本轮新增**） |
| D8 | ChunkGenerator.java L360-379：**structure 循环**（每 step 先跑，`setDecoratorSeed(l, m++, k)`） | structure 用独立重播种，place 消耗的随机**不污染** feature 序列——因每个 feature 放置前 L402 无条件 setDecoratorSeed(l,p,k) 重置 | Rust 跳过 structure（worldgen_handle.rs 无对应循环） | ✅ 数学上无害（重播种吞掉一切前序消耗）；⚠️ 仅当未来引入「放置时读世界状态」依赖 structure 方块时才需补（登记不展开） |
| D9 | ChunkGenerator.java L402-406：每 feature `setDecoratorSeed(l, p, k)` → `placedFeature.generate(...)` | ChunkRandom.java L75-78：seed = populationSeed + index + 10000*step，直接 setSeed | chunkrandom.rs L158-161 + worldgen_handle.rs L879 | ✅（p/k 取值链同 D6/D7） |
| D10 | PlacedFeature.java L48-63：`Stream.of(pos)` 逐 modifier `flatMap` → Java Stream **惰性**：forEach 拉取时每个位置**深度优先**走完全链才到下一个；末段 L57-61 `configuredFeature.generate` 与 modifier **共用同一 random 流** | 消费顺序 = modifier 链 DFS + feature 内部续流 | placement.rs L447-467（visit 递归 DFS + placed Cell）——与一手源语义一致 | ✅（首轮 S3 的「可疑」在本轮对照一手源后升级为对齐；注意 Java forEach 的实际拉取顺序 = flatMap 链惰性求值，Rust DFS 是其忠实展开） |
| D11 | RNG 原语（BaseRandom.java L15-30）：nextInt(bound) 幂 2 = `bound*next(31)>>31`；非幂 2 = 有符号拒绝采样 `i%bound`，`i-j+(bound-1)<0` 重试；**经 ChunkRandom.next(bits) 分发**（Xoroshiro 基类时取 nextLong 高 31 位） | 注意：Xoroshiro128PlusPlusRandom 自身的无符号 nextInt 覆写（L58-75）在 ChunkRandom 路径**不被使用**（ChunkRandom extends CheckedRandom 走 BaseRandom 默认实现） | chunkrandom.rs L114-128（next_int_bound，对两种基类同一公式） | ✅——Rust 公式与 BaseRandom 默认路径一致；首轮未识别的「Xoroshiro 分支该用哪套 nextInt」疑点就此澄清：**BaseRandom 默认（有符号、next(31)）是权威路径** |
| D12 | 邻域方块读取（feature 内部 setBlock/check） | Java ChunkRegion 带 2-chunk 边框，可读写邻 chunk（后写覆盖，algorithm-fingerprints #8） | worldgen_handle.rs L900-910 `region_col_at: None / pending_cross: None`；11 篇 L35 已知限制 | ⚠️ 沿用首轮 S9 简化（本课题 R 系登记项，非本轮新发现） |

## 2. 矿石 vs 树木：随机消耗模式差异点（问题 3）

**矿石（OreFeature/ScatteredOre，feature.rs L213-397）**：
- placement 链典型形态 `count → in_square → height_range → biome_filter`：随机消耗全部发生在 modifier 段（Count 的 IntProvider 1 次、Square 2 次 nextInt(16)、HeightRange 的 HeightProvider 若干次），biome_filter 0 次。
- feature 段消耗**结构性固定**：起点角 3 次（L215/223-224）+ generateVeinPart 内 size/方向/place 判定逐位推进——无提前退出式条件随机（should_not_discard / get_spread 固定调用点）。⇒ 矿石随机序列 = 「per-feature 一次性派生 + 固定节奏消耗」，**modifier 段公式错 1 次消耗即全链漂移**（Phase 4a 已修 Trapezoid/BiasedToBottom，phase4-status #5 一手坐实 2×nextBetween）。
- ScatteredOre 额外 get_spread（L394）——消耗点固定。

**树木（tree.rs，Phase 4a 新建）**：
- placement 链典型形态 `count(?) → in_square → heightmap → block_predicate_filter / environment_scan / surface_water_depth_filter`——**过滤器段 0 消耗**，消耗集中在 in_square 与 feature 段。
- feature 段消耗**数据依赖、可提前退出**：每站点 is_position_invalid / can_replace 判定失败 → 跳过该树但随机已被消耗；树高/半径/树冠逐块随机；decorators（L283-284+L491-493）共用同一流按 config 顺序追加消耗。⇒ 树木 = 「**变量长度、条件分支消耗**」，一处判定语义偏差（如 isSolid 近似 placement.rs L236、Replaceable≈air L241）不漂移序列本身，但改变**分支走向**后使后续消耗次数分歧 → 与 Java 的序列**在第一棵失败树之后即失同步**。
- **派生链层面对比结论（draft）**：矿石链对「派生公式+消耗次数」敏感（少一次消耗=整体平移）；树木链对「判定语义+消耗次数」双敏感（分支不同=序列叉开）。两侧共用的派生头（D1-D11）一旦对齐，矿石更易先收敛；树木收敛依赖 block 判定语义逐位一致——与架构「先矿石/替换层后树木」分层判据自洽。

## 3. 疑点清单（全部 draft，供 Phase 3 fan-out 取材）

- **N1（本轮新识别，范围/成员集）**：Java D5 用 3×3 邻域 biome 并集定 feature 候选；Rust 用当前 chunk 角单 biome（worldgen_handle.rs L857）。chunk 边界两侧 biome 不同时，Rust 漏放属于邻 chunk biome 的 feature（如树在隔壁沼泽）。**随机派生本身不受影响**（每 feature 独立重播种），但放置成员集与 Java 不同 → 直接的 palette 差源。归属：放置管线层，非 RNG 层。
- **N2（沿袭首轮 C6，升级为有对照但未数值验证）**：p=lastIndex（D7）结构对齐一手源，但 `Util.lastIndexGetter` 的「覆盖语义」与 Rust `last_index_map` 构建（feature_loader.rs L145-150）缺逐位数值对照（可用 D9 打点 WG_FEATURELOG 对比 Java 探针，属数据层验证，超勘探职责）。
- **N3（沿袭 R-1，tree 内部）**：tree decorator 同 Y 序 = Java HashSet 桶序近似（phase4-status §3.5/3.6 遗留 idk）——属 D10 段内消耗顺序的已知未闭合点。
- **N4（沿袭 idk-7）**：selector/patch generate 公式占位禁入对拍（phase4-status §风险移交）。
- **N5（本轮静态确认保留观察）**：D3/D11/D4 三项首轮疑点（C3/C4/C8）经一手源核对后**闭合/澄清**：C8（xoroshiro setSeed 路径）✅ 一致；C4（Trapezoid）已由 Phase 4a 修复（2×nextBetween 一手坐实）；C3（BiasedToBottom）同批实装。本条仅作首轮疑点收口记录，不构成新风险。
- **N6（判定语义近似，tree 分支敏感源）**：placement.rs L236 isSolid≈非空气、L241 Replaceable≈air 两处近似（注释自登记）——不消耗随机但改变树分支走向（§2 树木敏感机制），palette 差候选。

## 4. populationSeed 边界标注（问题 4，不深入）

- **状态**：⛔ 永久挂起（260904-15 用户拍板；NEXT_SESSION.md L32；11-features-stage.md L39-45 supersedes 记录）。挂起 ≠ confirmed ≠ closed；归因保持 candidate。
- **本链边界位置**：populationSeed 是 D2 的**输出值**、D9 setDecoratorSeed 的**输入 l**。本勘探对 D2 仅声明「公式静态对齐一手源」（L54-61 逐项一致），**不做逐位精度判定、不把对齐它列为前置动作**（11 L41 已取代）。D1/D3-D12 全部可独立于挂起项推进——若后续定位收口于 D2 输出值本身，即触架构 R2「停手报用户」。
- **与矿石疑点关系**：挂起项④的「ore 1/13 匹配」疑点在链上即 D2→D9→矿石消耗链；架构边界拍板（260905-05）= 矿石只修规则层，归因落派生层即停手。

## 5. 与「POST 排序对齐」（7× 首载漂移候选根因）的相关性标注（问题 1 附加，不混线）

- POST 排序属**光照课题**的首载漂移候选根因（NEXT_SESSION L27；10-timewise L2897/2901）。
- 本链唯一交点：feature 放置写邻 chunk（D12）时，**后写覆盖**结果依赖「哪些 chunk 先完成 decoration」——即 chunk 装饰调度顺序（Java ChunkStatus FEATURES 的跨 chunk 编排）。RNG 派生（D1-D11）**完全不受调度顺序影响**（每 chunk 独立 populationSeed）。
- 结论：feature parity 课题定位 palette 差时，**不要**把差归因到 POST 排序；反之首载漂移若与 feature 写入序相关，引用本表 D12 而非 D1-D11。两课题共享的只有 D12 一格。

## 6. 知识库覆盖与声明

- 一手源核对：mc_src_extract 下 ChunkRandom / BaseRandom / Xoroshiro(Impl) / RandomSeed / ChunkGenerator / PlacedFeature / PlacedFeatureIndexer（file:line 见 §1 表）。
- Rust 现状：chunkrandom.rs / xoroshiro.rs / placement.rs / feature_loader.rs / worldgen_handle.rs / tree.rs / feature.rs（含 260905-05 patch 后工作区状态，未提交）。
- 沿袭文档：scout-random-chain-260905-05.md（首轮 11 站点）、s1-semantics-260905-05.md、phase4-status-260905-05.md、11-features-stage.md、NEXT_SESSION.md。
- 需外部/运行时项：N2（p 值数值对照）、populationSeed 逐位精度（挂起域）——均需探针级验证，超本勘探职责。
