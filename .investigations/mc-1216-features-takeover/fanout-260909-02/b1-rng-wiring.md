# fanout-260909-02 / b1：RNG/seed 消费序全局错位（候选）

- **状态**：draft（静态审查，Degraded 分层——无运行时 trace/probe，仅源码对读）
- **产物归属**：fanout .b1 候选，主会话收敛/judge 前不升级
- **§9.7 口径声明**：本结论载体 = **静态源码对读**（Java yarn 源 vs worldgen-core/src），覆盖面 = features 阶段 RNG 接线公式/索引域/迭代序三轴，**不含**运行时 RNG 值对拍；与 batchD 存档写入口径、与后续 WG_FEATURELOG 运行时口径**均不可直接比**。
- **session**：260909-02（worker b1，只读无 shell）

## 1. Java 特征 RNG 消费序精确语义（一手源）

源：`.tmp/jungle-l-260906/mcsrc/net/minecraft/world/gen/chunk/ChunkGenerator.java:334-423` + `net/minecraft/util/math/random/ChunkRandom.java:54-78`

1. **RNG 实例**：`ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))`（ChunkGenerator.java:343）——baseRandom 是 **Xoroshiro**（非 Legacy LCG）。
2. **population seed**（:344）：`l = setPopulationSeed(world.getSeed(), blockPos.getX(), blockPos.getZ())`，blockPos = chunk 最小角 `(cx*16, cz*16)`。公式（ChunkRandom.java:54-61）：`setSeed(worldSeed); l = nextLong()|1; m = nextLong()|1; n = blockX*l + blockZ*m ^ worldSeed; setSeed(n); return n`。
3. **step 主循环**（:360）：`k = 0 .. j`，`j = max(GenerationStep.Feature.values().length, i)`（i = 全局 IndexedFeatures 步数）。
4. **结构体种子**（:361-378）：每个 step 内，`map = registry 按 getFeatureGenerationStep().ordinal() 分组`（:340-341），step 内序号 `m` 从 0 递增（:361,377），`setDecoratorSeed(l, m, k)`。
5. **feature 种子**（:381-412）：`intSet = ∪（3×3 邻域 chunk 全 section biome 容器并集 ∩ biomeSource.getBiomes()）各 biome` GenerationSettings.getFeatures()[k] 的 `indexMapping` 值（:346-353, :384-391）；**toIntArray + Arrays.sort**（:394-395）；遍历有序 p，`setDecoratorSeed(l, p, k)`（:402）后 `placedFeature.generate`。
6. **p 的来源**（IndexedFeatures 语义，:342 `indexedFeaturesListSupplier`）：p = 该 feature 在 step 内拓扑排序列表的 **lastIndex**（PlacedFeatureIndexer.collectIndexedFeatures：featureIndex 首现递增 → biome 内相邻建边 → (step, featureIndex) 比较器拓扑排序 → 按 step 分组 → Util.lastIndexGetter map.put 覆盖取末位索引）。**不是首现序，不是 registry key 序**。
7. **关键隔离性质**：每次 `setDecoratorSeed` 都 `setSeed(l)` **完全重置** RNG（ChunkRandom.java:75-78）——结构体种子与 feature 种子、以及 feature 之间互不共享序列，跳过任何一个 feature 的种子不影响后续 feature 的 RNG 状态。

## 2. Rust 侧 feature 执行通路 RNG 派生与消费序

源：`worldgen-core/src/worldgen_handle.rs:1030-1180`、`chunkrandom.rs:159-189`、`feature_loader.rs:243-362`、`feature.rs:1980-1988`、`biome.rs:233-239,549-558`

1. **RNG 实例**：`ChunkRandom::xoroshiro()`（worldgen_handle.rs:1056，注释明确与 carver 的 Checked 区分）——与 Java:343 同构。
2. **population seed**：`set_population_seed(self.seed, cx*16, cz*16)`（:1058）→ chunkrandom.rs:163-171，公式逐项同 Java（wrapping_mul 防溢出，:168）。**有 Java 数值回归锚**：`src/bin/chunkrandom_test.rs:50-51` 断言 populationSeed = -3665859634238804548（worldSeed=8576294172403134396, block=(720*16, -432*16)），及 :60-69 decorator seed 单调性。
3. **decorator seed**：`set_decorator_seed(population_seed, p, k)`（:1079）→ chunkrandom.rs:173-178，`l = pop + index + 10000*step` 同 Java:76。每个 feature 前重置，消费序 = p 升序（:1072 sort+dedup）。
4. **p 域**：`PlacedFeatureIndexer::build`（feature_loader.rs:271-348）= Java collectIndexedFeatures 精确移植：首现 featureIndex → 相邻建边（:288-303）→ BTreeSet 升序邻居 DFS 后序反转（:304-333）→ 按 step 分组 → lastIndex map（:341-347）。`int_set_for`（:351-362）= Java indexMapping 取值。
5. **biome 枚举序**（p 域的输入）：`all_biome_features` 按 `registry_order` 输出（biome.rs:549-558），`registry_order` 从 `biome_registry_order.json` 加载（:403-414，E-C2 修复，260905-08）；**文件缺失时回退字典序**（:414 eprintln + 字典序）——回退态下 p 域与 Java 错位。
6. **OreFeatureContext.world_seed**（:1157）：仅喂 geode 独立噪声流（feature.rs:1986-1988，`LegacyRandom(world_seed)` + DoublePerlin(-4,[1.0])，注释锚 Java GeodeFeature :41-42），**独立流不占 decorator RNG**——与 Java 一致，不构成消费序错位。
7. **结构体种子跳过**：feature_loader.rs:249 注释 + worldgen_handle 本通路无结构体循环——因 Java 每次 setDecoratorSeed 完全重置（§1.7），跳过结构体种子**不污染** feature RNG（batchD v2 结构体放置走独立通路）。

## 3. 对比判定：b1 是否成立

**结论：b1 的"公式/实现层"不成立——两侧 RNG 链已同构；b1 的"输入域层"保留一个可证伪残差（p 域输入），判"部分排除"。**

| 轴 | Java | Rust | 判定 |
|---|---|---|---|
| population seed 公式 | setSeed(ws); 2×nextLong\|1; bx*l+bz*m^ws | chunkrandom.rs:163-171 同式，有数值回归锚 | ✅ 同构 |
| base RNG 类型 | Xoroshiro128PlusPlus | ChunkRandom::xoroshiro + create_xoroshiro_seed | ✅ 同构（含 next(bits)/nextLong MC-239059 语义） |
| decorator seed 公式 | pop + p + 10000*k | chunkrandom.rs:174-177 同式 | ✅ 同构 |
| 迭代序（step 内 p 升序） | sort（:395） | sort+dedup（:1072） | ✅ 同构 |
| p 索引域（topo+lastIndex） | PlacedFeatureIndexer | feature_loader.rs:271-348 精确移植 | ⚠️ 算法同构，**输入依赖 registry_order.json 完整性**；缺失即回退字典序 → 全局 p 错位（biome.rs:414 回退路径存在） |
| 结构体种子隔离 | setDecoratorSeed 每次重置 | 跳过但零污染（重置语义） | ✅ 不构成错位 |
| biome 集成员（intSet 成员） | 3×3 chunk 全 section 容器 | Y 切片 [0,64,128] 近似（IDK-1，:1039） | ⚠️ 影响哪些 feature 执行，不影响 seed 数值——属特征集差，非 RNG 链错位 |

**推理**：b1 原假设"所有随机放置位置整体错开"要求 seed 数值层错位。静态证据显示公式/类型/迭代序三层均已同构且 population seed 有 Java 数值锚；p 错位只可能经由 **①registry_order.json 缺失/不全回退字典序**、**②Java registry 枚举序数据与实际运行时序不一致**、**③worldSeed 传入差**（已被 terrain 低噪声基线间接排除：noise/carver 同 seed 派生链已对齐）。若 p 域错位，表现应为"同一 fid 种子不同 → 树/矿位置系统性错开"——与主残差（features 层 4.86M 差）症状兼容，故不能纯静态排除，需探针。

## 4. 廉价判别探针建议（主会话执行）

**探针 P-b1：单 chunk (k, p, fid) 序列对拍**

1. Rust：`WG_FEATURELOG=1`（worldgen_handle.rs:1080 已有钩子）跑 seed -8248318472910187742 chunk (0,0)，收集 `[FEATURE] chunk(0,0) step=k p=p fid=X` 全序列，并单独打印 population_seed（建议临时加一行 log 或复用 :1058 断点）。
2. Java：injector/mixin 在 ChunkGenerator.generateFeatures :402 处打印 `(l, k, p, registryKey)`（chunk 0,0），一次 runServer 即可。
3. **判据**：
   - `l` 不一致 → seed 接线差（worldSeed 传入/公式），b1 复活为最强候选；
   - `l` 一致、`(p, fid)` 序列有差 → **p 域输入差**（registry_order.json 与 Java 实际枚举序不符，或 Rust all_biome_features biome 集不全）——先核对 biome_registry_order.json 覆盖面（end/nether biome 是否在表内，biome.rs:381 注释暗示 end 5 biome 特例）；
   - `(p, fid)` 序列完全一致 → b1 **整体排除**，残差归因转向特征实现层（放置/decorator 消费序），与既有 §9.7 单列 9 项分流。
4. 成本：Rust 一次 CLI run（毫秒级）；Java 一次 gradle runServer 单 chunk probe（分钟级）。判别力：直接命中三层（l / p / fid）。

## 5. 错误记录（五段式）——本次无新踩坑，仅登记既有修复线索

无新错误产生（只读对读）。关联既有修复（非本次发现，引用备查）：E-C2「p 首现序 vs Java lastIndex/topo 序」（260905-08 重写 PlacedFeatureIndexer::build，feature_loader.rs:267-270 注释自证 amethyst_geode java p=2 vs rust p=0 曾错位）——该修复即 b1 假设的主修复，本次对读确认修复后公式层同构。判错经验沉淀：**「全局位置整体错开」类症状，先分层查 seed 接线三轴（population 公式 / 索引域输入 / 迭代序），索引域错位往往不在算法而在算法的输入数据（registry 序文件）**。

## 6. 速查

- 判定：b1 公式层 ✅ 排除；输入域层 ⚠️ 保留（registry_order.json 回退路径 + biome 集近似）→ **部分排除，待 P-b1 探针收敛**
- 置信度：draft（静态 Degraded）
- 关键证据行号：ChunkGenerator.java:343/344/360/402/406；ChunkRandom.java:54-61/75-78；worldgen_handle.rs:1056/1058/1064-1079/1157；chunkrandom.rs:163-178（测试锚 bin/chunkrandom_test.rs:50-69）；feature_loader.rs:271-348/249；biome.rs:403-414/549-558
