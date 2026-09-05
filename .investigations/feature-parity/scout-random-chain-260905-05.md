# scout-random-chain-260905-05 — feature 放置随机派生链勘探（只读，recode-scout）

> 置信度：**draft**（勘探产物，只定位不解读不承担）。验证分层：Degraded（静态知识库 + 源码 grep，无运行时探针）。
> 任务：MC 1.20.1 worldgen feature 放置 random source 派生链在 Java 参照（知识库记录）与 Rust 实现（worldgen-core）两侧的对应现状。背景 = G2 残差根因 feature 放置分歧（树/藤蔓/矿石，g2-convergence-260905-03）。

## 1. 派生链总图（Java 参照侧既有结论，源=知识库，未猜）

Java 侧参照在本工作区以文档/知识库形式存在（无一手 .java 源；docs 引用的 Java 行号来自历史排查记录）：

1. **根**：`worldSeed`（docs/02-random.md L11-16：`XoroshiroRandom(seed)` → `nextSplitter()` → NoiseConfig.randomDeriver；**注意**：feature 阶段不用此 deriver，用 ChunkRandom 包装 Xoroshiro，见 02 L97-99）。
2. **chunk 装饰随机**：`ChunkRandom(new Xoroshiro128PlusPlusRandom(...))`（**FEATURES 阶段基类 = Xoroshiro，不是 carver 的 CheckedRandom LCG**——algorithm-fingerprints 发现 #7 L151-176，confirmed）。
3. **populationSeed**：`setPopulationSeed(worldSeed, blockX, blockZ)` = setSeed(worldSeed); l=nextLong()|1; m=nextLong()|1; n=(blockX*l + blockZ*m) ^ worldSeed; setSeed(n)（02 L103 附近 setCarverSeed 类似形态；Rust 侧 chunkrandom.rs L146-155 注释有完整 Java 公式）。坐标输入 = **chunk 块坐标**（cx*16, cz*16，见 Rust worldgen_handle.rs L864）。
4. **setDecoratorSeed(l, p, k)**：`l = populationSeed + index + 10000*step`，其中 p = feature 在 `features[step]` 的 **lastIndex**（Java `Util.lastIndexGetter` / indexMapping），**不是全局 featureIndex**（11-features-stage.md L21）。p 错 → 该 feature 随机序列全偏。structure 分支 `setDecoratorSeed(l,m,k)` 独立重置，不污染 feature 序列（同处）。
5. **消费链**：`PlacedFeature.generate` = flatMap 惰性 **深度优先**，全部 placement modifier + feature 内部共用**同一 RNG 流**（algorithm-fingerprints 发现 #6 L120-148，confirmed；02 篇后写覆盖语义见发现 #8）。
6. **对齐铁律**：feature 阶段逐位对齐 = 派生公式 + p/k/index 取值 + RNG 基类 + 消费顺序四者同时一致。

### populationSeed 永久挂起项（边界声明，不展开分析）

- **状态**：⛔ **永久挂起（260904-15 用户拍板）**，见 NEXT_SESSION.md L32 与 11-features-stage.md L39-45。
- **挂起内容（仅记录）**：「ore 放置位置与 vanilla 仅 1/13 匹配」疑点归因 populationSeed/setDecoratorSeed 随机序列与 Java 不完全一致（11 L33 原文；supersedes 记录 11 L41）。
- **挂起语义**：挂起 ≠ confirmed ≠ closed；归因保持 candidate，坐标与归因保留不删（11 L43）。
- **边界声明（本勘探遵守）**：本产物只登记挂起状态与涉链站点，**不对 populationSeed 公式精度展开分析、不把「对齐 populationSeed」列为前置动作**（11 L41 已明确取代该指令）。
- **口径声明（§9.7）**：现 match=100% 口径 = 单 seed 8576294172403134396 / 4×4@200 / 存档写入口径；换 seed 出现同类残差不得误读为回归（11 L44）。

## 2. Rust 侧现状（worldgen-core）

- **随机核**：`worldgen-core\src\chunkrandom.rs`（172 行）——`ChunkRandom` enum {Checked(LCG), Xoroshiro} + set_seed/next/next_long/next_int_bound/next_float/next_double + 三个种子派生：`set_population_seed`(L148)、`set_decorator_seed`(L158)、`set_carver_seed`(L165)。
- **调度**：`worldgen_handle.rs` `apply_features`（L~815-937）：L862 `ChunkRandom::xoroshiro()`；L864 `set_population_seed(seed, cx*16, cz*16)`；L869 全局 `feature_indexer`；L879 `set_decorator_seed(population_seed, p, k)`；L914-915 generate_configured 闭包传同一 `feat_random`。
- **放置链**：`placement.rs`（310 行）——IntProvider（uniform/trapezoid/biased_to_bottom/weighted_list/clamped）+ 10 PlacementModifier + `PlacedFeature.generate` 递归 DFS（L289-309，与 Java flatMap 惰性语义对齐）。
- **特征实现**：`feature.rs`——Ore/ScatteredOre（generateVeinPart）、Disk、Spring、FreezeTopLayer、UnderwaterMagma。**树/花/藤蔓/植被不解析**：feature_loader.rs L50、L257「2026-08-10 用户拍板范围外」。
- **单测**：`worldgen-core\src\bin\chunkrandom_test.rs` L47-69（populationSeed 值比对 -3665859634238804548 + setDecoratorSeed index 0/1/2 序列偏移自检——**只验证公式自洽，无 Java 侧逐位参照**）。

## 3. 取随机站点对照清单（文件:行号级）

| # | 站点 | Java 参照（知识库记录） | Rust 实现位置 | 状态 |
|---|------|------------------------|--------------|------|
| S1 | chunk 装饰随机初始化 | ChunkRandom(Xoroshiro) + setPopulationSeed（02 L97-99, L103） | chunkrandom.rs L75-78/L148-155；worldgen_handle.rs L862-864 | ✅ 已实现（公式与基类对齐；**populationSeed 精度属永久挂起域，此行不做判定**） |
| S2 | 每 feature 装饰种子 | setDecoratorSeed(populationSeed, lastIndex, step)（11 L21；07 L441） | chunkrandom.rs L158-161；worldgen_handle.rs L879；feature_loader.rs PlacedFeatureIndexer | ✅ 已实现（F-3 修复后用全局 all_features_lists 构建）；⚠️ p=lastIndex 与 Java indexMapping 的逐位等价无独立验证记录 |
| S3 | placement modifier 链消费 | flatMap 惰性 DFS，同一 RNG 流（指纹 #6） | placement.rs L147-209 + PlacedFeature.generate L289-309 | ✅ 已实现（DFS）；⚠️ 可疑点见 §5 C3/C4 |
| S4 | Count/HeightRange IntProvider 取整 | UniformIntProvider / TrapezoidIntProvider / HeightProvider | placement.rs L24-61（Trapezoid L32-42、BiasedToBottom L43-46） | ⚠️ BiasedToBottom L45 注释自认「近似（Java 更复杂）」；Trapezoid 实现与注释公式存疑（L34 plateau+1 vs 注释 L33 0..plateau-1） |
| S5 | RarityFilter / in_square | rarity nextInt(chance)；in_square nextInt(16)×2 | placement.rs L155、L158 | ✅ 已实现 |
| S6 | ore 每次放置随机 | OreFeature.generate 端点角 + generateVeinPart 内部随机 | feature.rs L215/223-224/256/359/371/394/625 | ✅ 已实现；⚠️ 逐位精度属 populationSeed 挂起域辐射（1/13 匹配疑点即此链），**挂起，不判** |
| S7 | 树形状随机（每棵树） | Java tree/RandomSelector/TreeFeature 内部随机序列 | **不存在**（feature_loader.rs L50/L257 范围外，2026-08-10 拍板） | ❌ 缺失（范围外决策，非 bug）；G2 残差树冠分歧的直接对应缺口 |
| S8 | 藤蔓随机 | LeavesVineVineDecorator / TrunkVineVineDecorator | **不存在**（同上，tree 家族一并范围外） | ❌ 缺失（范围外决策）；G2 残差 vine 分歧直接缺口 |
| S9 | 邻域 chunk 读取 | ChunkSectionCache 惰性生成邻域 + 后写覆盖（指纹 #8） | worldgen_handle.rs L907-908 `region_col_at: None / pending_cross: None`；11 L35「只处理当前 chunk 内」 | ⚠️ 缺失/简化（11 L35 已知限制）；影响跨界树冠/ore 一致性 |
| S10 | Biome placement 过滤 | posToBiome 8 邻域 jitter 判定 | placement.rs L173-177（直接透传，注释「过滤由调用方预判」） | ⚠️ 可疑（简化实现，随机不消费故不偏序列，但放置判定域可能与 Java 不同） |
| S11 | NoiseBasedCount | count + floor(noise*maxCount) | placement.rs L201-207（noise 硬编码 0.0，L204「Phase 3 简化」） | ⚠️ 简化（噪声恒 0 → 计数可能偏；随机消费 count.get 本身仍在流内） |

## 4. 已知 feature 分歧输入（12-lighting.md + g2-convergence-260905-03 摘录）

- 12-lighting.md L8-10：G2 残差 = worldgen feature 放置分歧（树叶/藤蔓/矿石/安山岩），judge 全量 447 chunk palette 对比归因；G1 exact 100% 仅覆盖树冠一致区 16 chunks。
- g2-convergence-260905-03.md：
  - L13（b2 事实）：「翻转集中 **Y=3..4 地表树冠段**、显式差剖面 rust=vanilla−1~2（树冠顶多一档衰减）→ blocks9 树冠输入差签名」。
  - L18（决定性探针）：worst 6 chunk palette 对比 = 树木/藤蔓放置分歧——`jungle_leaves`/`oak_leaves`/`vine`/`jungle_log` 增减各 80-350 处/chunk，另 granite↔andesite 少量。
  - L25：346/447 chunk 有 blocks 差，签名含 stone↔coal_ore、granite↔andesite（**矿石/替换层必须入范围**）；是否属既有挂起域待用户裁定。
- **与随机派生链的关系线索（scout 判断，draft）**：树/藤蔓整族未实现（S7/S8），其「分歧」主因是**缺失而非随机序列错**——Rust 根本不生成这些方块，故 Y=3..4 树冠差无法用 S1-S6 的随机链误差解释；但 granite↔andesite / stone↔coal_ore 差落在 ore 链（S6），恰与 populationSeed 挂起域（1/13 匹配）+ S4 近似 + S11 简化叠加区重合。**注意**：单 seed 口径下 ore 已 100%（11 L44），新 seed/新区域 ore 残差是否复现未验证——idk。

## 5. 可疑点清单（按优先级，全部 draft / @anchor.idk 性质）

- **C1（范围缺失，非随机错）**：树/花/藤蔓/植被整族未实现（feature_loader.rs L50/L257）→ Y=3..4 树冠 + vine 残差的结构性缺口。2026-08-10「范围外」拍板早于 G2 残差归因；新 feature parity 课题需用户重新裁定范围。**本条不是随机派生链问题，是存在性问题。**
- **C2（挂起域辐射）**：ore 链逐位精度属 populationSeed 永久挂起（11 L42 ④），本勘探不展开；但 G2 的 granite↔andesite / stone↔coal_ore 差是否即挂起项 ④ 在该 seed 下的表现 → **@anchor.idk（需换 seed 复现探针，属数据层验证，超出勘探职责）**。
- **C3**：`BiasedToBottom` placement.rs L43-46 自注释「近似（Java 更复杂）」——Java `BiasedToBottomIntProvider.get = min + nextInt(bound) + nextInt(bound-min)/2` 形态（凭 docs 记忆，**需外部资料核一手 Java 源**）；每调用消耗随机次数若与 Java 不同会整体偏移后续序列。
- **C4**：`Trapezoid` placement.rs L32-42：注释 L33 写 `nextBetween(0, plateau-1)`、代码 L34 用 `next_int_bound(plateau + 1)`，且 L37 `i` 计算后 `let _ = i` 弃用——实现与注释不自洽，疑似未完成的修复。**下一个随机消费点若在 trapezoid 链上（height_range 常用 trapezoid），y 值与序列双双可偏。可疑度高。**
- **C5**：`HeightProvider`（carver.rs，placement.rs L137/L161 引用）的随机消费实现本勘探未逐行核对 → **idk（未读，非「无问题」）**。
- **C6**：`set_decorator_seed` 的 p=lastIndex 语义（int_set_for 返回集合，worldgen_handle.rs L872-879 遍历）与 Java `Util.lastIndexGetter` 的逐位等价缺独立验证记录（chunkrandom_test.rs 只测自洽偏移）。
- **C7**：S9 邻域简化 + S10 Biome 透传 + S11 noise 恒 0——三处简化不直接错随机序列（S10 不消费随机；S11 消费 count.get 但 count 值偏），但使放置判定域与 Java 不同，属 feature parity 课题范围内的已知偏差源。
- **C8**：Xoroshiro 基类 `set_seed`（chunkrandom.rs L87-91）走 `create_xoroshiro_seed(seed as u64)`——seed 转 u64 后经 SHA-256 混合，与 Java `Xoroshiro128PlusPlusRandom(long seed)` 构造路径的一致性未在本工作区 docs 中找到独立验证记录 → idk（02 篇仅验证了单参构造 = RandomSeed.createXoroshiroSeed 一致，未点名 ChunkRandom 包装路径）。

## 6. 知识库覆盖声明

- 已查：knowledge/INDEX.md（入口）、knowledge/discovered/algorithm-fingerprints.md（#6/#7/#8 相关）、versions/1.20.1/docs/02-random.md、07-block-pipeline.md、11-features-stage.md、12-lighting.md、10-timewise-archive.md（L2096-2100）、NEXT_SESSION.md L32、.investigations/light-opt/g2-convergence-260905-03.md。
- **需外部资料项**：① Java 一手源（PlacedFeature/BiasedToBottomIntProvider/TrapezoidIntProvider/ChunkRandom/ChunkSerializer）不在工作区，docs 引用为二手记录；② ChunkRandom(Xoroshiro) 构造路径（C8）无知识库条目。
- 本产物只读勘探，不改代码；结论不改变任何挂起项状态；populationSeed 边界声明见 §1。
