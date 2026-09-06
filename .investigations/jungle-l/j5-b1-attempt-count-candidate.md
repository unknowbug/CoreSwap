# J5 fan-out 候选 .b1：tree attempt/selector 计数差（j5-b1-attempt-count-candidate）

> 角色：core.worker（re-code/swe 混合，fan-out .b1）；状态：**draft**。
> 验证分层：**Degraded（静态）**——一手源码比对 + 既有 trace 日志复读，未跑任何运行时探针；全部结论待 §4 判别探针升级。
> 课题：J5，seed 8576294172403134396，chunk (29,-16)。继承事实（已 judge/主会话核实，见 j5-trace-verdict-260906-01 附录）：Java 目标 chunk 2 棵 mega（464,71,-252 / 479,71,-249，均 h=27）；Rust native 载具（bin-diag/j5_tree_trace.rs）目标 chunk 零棵、5×5 邻域 11 棵；T5 形状级无结构性差（原 §3 结论因载具更正需重做，但方向性结论——分歧先于 trunk generate——由附录 #2 数据层支持）。

## 1. Java 一手参照：mega jungle 树的 placement 链（逐环节）

数据链（versions/1.20.1/data，均为一手 JSON）：
- biome `jungle.json` L78-90：vegetal_decoration（step index 9）含 `minecraft:trees_jungle`（L81）。
- placed_feature `trees_jungle.json` L3-34：placement 链 = **count(weighted_list 50:9 / 51:1) → in_square → surface_water_depth_filter(0) → heightmap(OCEAN_FLOOR) → biome**。
- configured_feature `trees_jungle.json` L1-20：**random_selector**：fancy_oak_checked 0.1 / jungle_bush 0.5 / **mega_jungle_tree_checked 0.33333334**，default = jungle_tree。
- placed_feature `mega_jungle_tree_checked.json` L3-16：仅 `block_predicate_filter(would_survive jungle_sapling)`。

代码链（E:\PYTHON\MC\data\mc_src_extract，yarn 一手源）：
1. `AbstractCountPlacementModifier.java:13-15`：`IntStream.range(0, getCount(random,pos))`——count.get **恒 1 次消费**，产出 n 个同 pos。
2. `WeightedListIntProvider`（经 CountPlacementModifier.java:29-31）：nextInt(totalWeight) 选项——数据内全是常量 → 消费恒 1。
3. `SquarePlacementModifier.java:19-23`：`nextInt(16)+x`（第一次给 X）、`nextInt(16)+z`（第二次给 Z），恒 2 次消费。
4. `SurfaceWaterDepthFilterPlacementModifier.java:28-32`：OCEAN_FLOOR 与 WORLD_SURFACE 双高度图差 ≤0，0 消费。
5. `HeightmapPlacementModifier.java:27-32`：`context.getTopY(OCEAN_FLOOR, x, z)`，0 消费；k ≤ bottomY 整支丢弃。⚠️ 该链用的是 **OCEAN_FLOOR（非 WG 型）**——FEATURES 阶段经 ChunkRegion.getTopY 读**实况（含本阶段已放方块更新）**高度图（260905-06 修正注释已引用 ChunkRegion L422-425 sampleHeightmap+1）。
6. `BiomePlacementModifier.java:24-29`：BiomeAccess jitter 采样 → GenerationSettings.isFeatureAllowed，0 消费。
7. `RandomFeature.java:23-27`（selector）：**逐 entry `random.nextFloat() < chance`，命中即返（后续 entry 不消费）；全落空走 default（不抽选择 RNG）**。chance 存 float。
8. `PlacedFeature.java:48-63`：flatMap **惰性深度优先**（第 1 个 pos 走完整链再第 2 个）；同一 Random 流贯穿全链与内层 placed feature（generateUnregistered）。
9. 每个 placed feature 的装饰随机流独立：`setDecoratorSeed(populationSeed, p, k)`（p=全局 placed feature index，k=step）。

Java 目标 chunk 预期 attempt 轮廓：trees_jungle 每 chunk 恰 1 次执行、count=50/51、50 次SQ 内 pos、每次 selector 抽 0-3 次 nextFloat。

## 2. Rust 侧逐环节核对（worldgen-core\src）

| 环节 | Java | Rust（证据行） | 判定 |
|---|---|---|---|
| step/调度 | biome features[9]，每 placed feature setDecoratorSeed(p,k) | worldgen_handle.rs L1011-1035：set_population_seed → 遍历 step → intSet(p 排序去重) → set_decorator_seed | 同构 ✓（p 编号依赖 registry_order，biome.rs L236-239 已修 E-C2；错位风险归 .b2） |
| count | 1 消费，weighted 50/51 | placement.rs L292-296 `count.get(random)` 1 次；L54-62 weighted next_int_bound(total) 累减 | 同构 ✓ |
| in_square | 2 消费（x 先 z 后） | placement.rs L300-305 dx=first, dz=second | 同构 ✓ |
| water_depth | 双高度图差 ≤0，出界真读邻 chunk | placement.rs L339-349；**越界直通放行**（L346 注释已登记） | ⚠️ 微差（SQ 恒 chunk 内，不影响本课题，低） |
| heightmap | **实况 OCEAN_FLOOR**（feature 阶段动态更新），k>bottomY | placement.rs L310-320：**预计算静态快照** `ocean_floor`；k=top+1，k≤min_y 丢弃 | **差异 D1（见 §3）** |
| biome 门 | jitter + isFeatureAllowed | placement.rs L322-334 Fix-2 已接 biome_allows | 同构（接线质量待验证，中低） |
| selector | 逐项 `<chance` 即选即返；float | feature_loader.rs L396-416 `random.next_float() < *chance`；tree.rs L854 `as f32` | 同构 ✓（恒等语义逐行对上） |
| 链序/惰性 | 深度优先 flatMap | placement.rs L474-494 visit 递归（深度优先，注释明确对拍） | 同构 ✓ |
| would_survive | sapling→PlantBlock.canPlaceAt | placement.rs L234-239：下方 ∈ 硬编码 dirt 列表（L190-193） | 同构近似（idk-5 已登记） |

**已核为一致的计数语义**（.b1 的「字面计数」层未发现差异）：count 消费次数、SQ 消费次数与赋值序、selector 即选即返 + float 精度、深度优先遍历、decorator seed 每放置特征独立。

## 3. 差异点清单（.b1 范围内，按置信度排）

- **D1【中高，静态】heightmap 动态 vs 静态**：Java `heightmap(OCEAN_FLOOR)`（trees_jungle.json L29，非 WG 型）在 FEATURES 阶段读**被本阶段放置更新的实况高度图**（先放的树/植被抬高地表，影响后续 attempt 的 y 与 would_survive 落点）；Rust（placement.rs L310-320 + worldgen_handle.rs L1084 `ocean_floor: Some(&ocean_floor)`）用 terrain 预计算**静态快照**。后 49 个 attempt 的 y 逐个漂移 → 部分树落点/拒绝集不同 → 树位集不同。这是「attempt 相同但每 attempt 选位/被拒原因不同」的机制候选，属 .b1（位置选择器语义差），与 .b2（RNG 相位）正交。@anchor.idk("静态推断：未验证 ChunkRegion.getTopY 在 vanilla FEATURES 阶段对 OCEAN_FLOOR 型是否真的吃到本阶段 setBlock 更新（高度 map 挂 chunk 且 setBlockStatus 更新的链路未逐行取证）；若实际读到的是 pre-feature 快照则 D1 消解") —— 需 §4 探针 J 侧字段 HMY 直接判。
- **D2【中，静态+日志旁证】mega 的 soil 判定大面积 false**：j5-rust-native-trace.log 目标 chunk 组（L12341-12581）16 次 [MJTD] 中 15 次 `soil=false`（仅 L12541 (471,70,-247) soil=true），邻 chunk 组同样以 false 为主。Java 在同类地表（grass_block）放树成功，若两侧 p 点地表一致，Rust 的 soil 判定（giant trunk 底座 dirt 检查 / would_survive 的 dirt_ids 硬编码表 placement.rs L190-193 + block_at 实况读）存在系统性误拒候选——**同一位置「Java 接受 / Rust 拒绝」即 attempt 通过性计数差，.b1 核心**。但注意：soil=false 的语义未在打点端注明（是 would_survive 拒、还是 giant trunk 底座拒、还是直接跳 mega），判读受限。
- **D3【中，矛盾点待澄清】(464,71,-252) 的 [TH] 27 与「零 mega」统计冲突**：目标 chunk 组首个 attempt `[SQ] 464,-252 → [TH] 27 @ (464,71,-252)`（L12342-12343）与 Java 树 A **同位同高**，但该 attempt 无 [MJTD]/[TREESET] 跟随，且附录 #2 判目标 chunk 零 mega。两种解释：① selector 选了 mega、getHeight=27 与 Java 一致，但后续放置层（soil/canReplace/pending 跨界写）把它丢了 → **D2 同族（通过性差）且说明 attempt/selector 本身可能无差**；② [TH] 行归属其他 feature（组边界判读错）。**若 ① 成立，.b1 的「attempt 计数/selector」子假设大幅收窄，重心移向 D1/D2；若 ②，组归属方法需先修。** 主会话澄清 [TH] 打点语义即可低成本裁决。
- **D4【低，登记】noise_based_count noise 恒 0**（placement.rs L379-385 `Phase 3 简化 0`）：patch/grass 类 attempt 数两侧系统性不同。不直接影响 trees_jungle（每 placed feature 独立 decorator seed，流不串），但会放大整 chunk 方块 diff、干扰以「邻域树数」为指标的对拍口径。邻域缺陷，非本课题根因候选。
- **D5【低，登记】越界高度图语义**：Rust heightmap 越界 = min_y-1 → 恒拒（placement.rs L315），Java 真读 5×5 region 邻 chunk。SQ 恒 chunk 内 → 本课题不受影响；登记为其他树型（environment_scan/height_range 系）的通用偏差。

## 4. 判别探针设计（主会话可执行，最小打点）

目标：两侧同格式打出「每 chunk 的树 attempt 序列（pos + count + selector 选择 + 被拒环节）」，一次采集同时裁决 .b1（D1/D2/D3）与移交 .b2（相位）。

**判据**：对目标 chunk 的 trees_jungle 一次执行——
- 若两侧 `[CNT]` 值（50/51）与 `[SQ]` 序列逐项相同 → RNG 流相位一致 → 相位差排除（.b2 收窄），分歧必在 selector 之后的门层（D1/D2）→ .b1 得证且子分支定位到门；
- 若 `[SQ]` 序列第 i 项起分叉 → 相位差先行（.b2），.b1 仅保留 D1 类「门层独立差异」待相位修复后复测。

### Rust 侧（env 门控，复用 treediag_enabled()，placement.rs L283-286 已有）
在 worldgen_handle.rs feature 循环（L1026-1136）+ placement.rs 扩展打点，统一 CSV 到 stderr：
```
[T1] cx,cz,k,p,fid                 （feature 开始：step k、全局 p、placed feature id）
[CNT] fid,n                        （count 结果；现 [CNT] 增 fid 前缀）
[SQ] fid,i,x,z                     （第 i 个 attempt 的 in_square 结果）
[HMY] fid,i,hm_type,y,in_chunk     （heightmap 修正后 y；in_chunk=越界标记）→ D1 判据
[BIOME] fid,i,ok                   （biome 门结果）
[WATER] fid,i,ok                   （water depth 门结果）
[WS] fid,i,ok,block_below_id       （would_survive 结果 + 下方方块 id）→ D2 判据
[SEL] fid,i,idx,feature_id         （selector 每次抽选：第 idx entry、命中 id，含未命中行）→ D3/selector 判据
[TREE] fid,i,x,y,z,result          （attempt 终局：placed / rejected:<环节>）
```
每行恒含 fid/i，保证跨流可归组；门控用现成 `treediag_enabled()`（OnceLock，热路径零成本），不打点分支不进生产语义（只读 env，不改任何判定）。

### Java 侧（mixin，最低保障集——吸取 verdict §8.3 LocalCapture 教训，只用 HEAD/RETURN 注入）
- `AbstractCountPlacementModifier#getPositions` RETURN：n + fid（经 context.getPlacedFeature().map(PlacedFeature::toString)）。
- `SquarePlacementModifier#getPositions` RETURN：i, j。
- `HeightmapPlacementModifier#getPositions` RETURN：k, heightmap 类型名 → 对应 [HMY]。
- `BiomePlacementModifier#shouldPlace` RETURN：boolean → [BIOME]。
- `RandomFeature#generate` 每次抽选前/命中后（HEAD+参数，无 locals capture）→ [SEL]。
- 开关**双域**（`-Dwg.treediag` + env `WG_TREEDIAG`，verdict §8.2/E6 教训）；流拓扑按 §8.1 以 stdout 为载体。

### 采集与对拍
1. Rust：`WG_TREEDIAG=1 cargo run --release --bin j5_tree_trace > rust-t1.log 2> rust-t1.err`（现载具即可）。
2. Java：gradle runServer 注入 mixin + 双域开关，chunk 过滤 (29,-16) ±2。
3. 对拍脚本（.tmp/ 下一次性 python）：两侧按 fid=trees_jungle 归组，逐字段 diff（CNT → SQ 序列 → 每 attempt 的 HMY/BIOME/WATER/WS/SEL），输出第一个分叉字段 = 分歧环节定位。**首次对拍前先三查 seed**（AGENTS §三.3）。

## 5. 与其他候选的边界

- RNG 派生相位（setDecoratorSeed p/k 错位、population seed、splitter）→ **.b2**，本候选不深钻；本探针的 CNT/SQ 序列判据恰是两者的分界。
- 树内 trunk/叶机制 → 已由 T5/verdict 排除结构性差，不属 .b1。
- 「mega 放置成功性差（C-R1）」→ 与 D2 同族；verdict 已降优先级，若探针 [WS]/soil 字段坐标坐实误拒，D2 吸收之。

## 6. 诚实声明

- 本文档全部静态结论为 **Degraded（静态）**：Java 源 = yarn sources（一手），Rust 源 = 当前 worktree 一手；但 D1 的 Java 运行时高度图动态性、D2 的 soil=false 语义、D3 的 [TH] 归属均未运行时验证。
- @anchor.idk 清单：
  - @anchor.idk("D1：vanilla FEATURES 阶段 ChunkRegion.getTopY(OCEAN_FLOOR 非 WG) 是否含本阶段已放置方块的更新，未逐行取证")
  - @anchor.idk("D2：[MJTD] soil=false 的判定层（would_survive / giant trunk 底座 / 跳 mega）打点端未注明，证据链缺一环")
  - @anchor.idk("D3：j5-rust-native-trace.log 的 [TH] 打点是否等同 attempt 终局放置成功，语义未取证")
  - @anchor.idk("Java WeightedListIntProvider 数据集全为 ConstantInt 的断言仅核对 trees_jungle 一例，未全量核对（若存在非常量项则 Rust 直接返回 data 为近似）")
- 未修改任何生产代码；本文档与探针设计均为草案（draft）。

## 7. 置信度小结

| 结论 | 置信度 |
|---|---|
| 字面计数/selector/遍历序语义两侧一致（§2 同构表） | 较高（静态逐行对拍） |
| D1 heightmap 动静态差是 .b1 内最强机制候选 | 中（需探针 HMY 判） |
| D2 soil 系统性误拒 | 中（日志旁证强、机制待定位） |
| D3 矛盾点（[TH]27 vs 零 mega）必须先澄清再下 .b1 总结论 | 高（澄清成本低） |
