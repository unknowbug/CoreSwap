# B5+B6 勘探报告 — fallen_tree feature + place_on_ground decorator（260908-15）

> scout 只读勘探（re-code 路由，yarn 一手源码为权威）。产物仅本文；未改任何代码。
> 状态：draft。验证分层：Degraded（静态源码/数据核对，无运行时探针）。

## 0. 源码树定位

- **一手源码树根**：`versions/1.21.6/data/mc_src_extract/`（yarn mappings sources extract，与 260908-13 P5 verdict 引用同一根）。关键文件：
  - `net/minecraft/world/gen/feature/Feature.java`（L29：`FALLEN_TREE = register("fallen_tree", new FallenTreeFeature(FallenTreeFeatureConfig.CODEC))`）
  - `net/minecraft/world/gen/feature/FallenTreeFeature.java`（130 行）/ `FallenTreeFeatureConfig.java`（60 行）
  - `net/minecraft/world/gen/treedecorator/TreeDecoratorType.java`（L16：`PLACE_ON_GROUND = register("place_on_ground", PlaceOnGroundTreeDecorator.CODEC)`）
  - `net/minecraft/world/gen/treedecorator/PlaceOnGroundTreeDecorator.java`（86 行）
  - `net/minecraft/world/gen/treedecorator/AttachedToLogsTreeDecorator.java`（B5 数据依赖，见 §3）
  - `net/minecraft/world/gen/feature/TreeFeature.java`（L231-244：`getLeafLitterPositions`）；`TreeDecorator.java`（Generator）
- **版本证据**：无 DataVersion 文件实存（数据目录顶层只有 blocks.json/biome_params 等）；blocks.json 含 `minecraft:leaf_litter`(1052)、`minecraft:wildflowers`(1051) —— 1.21.6 新方块实存，佐证数据集为 1.21.6。1.21.6 Rust 薄壳实存：`versions/1.21.6/rust/src/`（jni_bridge.rs 等）。
- 注：`.tmp/scout-260905-08/mcsrc/` 是另一份 extract（1.20.1 侧），与 `versions/1.20.1/data/mc_src_extract` 对应；本次只以 1.21.6 侧为权威。

## 1. B5 机制 — FallenTreeFeature（一手源码，FallenTreeFeature.java 全文已读）

### 1.1 输入（JSON 形态，FallenTreeFeatureConfig.CODEC）
`trunk_provider`（BlockStateProvider）+ `log_length`（IntProvider，校验 0..16）+ `stump_decorators`（TreeDecorator 列表）+ `log_decorators`（TreeDecorator 列表）。generate 恒返回 true（不依赖放置成功）。

### 1.2 放置算法（generate L38-47）
1. `generateStump`：在 origin 放 trunkProvider 方块（原 axis），对 stump 单点集跑 stump_decorators。
2. `direction = Direction.Type.HORIZONTAL.random(random)`。
3. `i = log_length.get(random) - 2`（日志段数 = 抽取值−2）。
4. 起点 = `pos.offset(direction, 2 + random.nextInt(2))`（离桩 2~3 格）→ `moveToGroundPos`：上移 1 格后向下找最多 6 格，直到 `canReplace && 下方方块 isSideSolidFullSquare(UP)`；6 格内没找到就停在最后位置。
5. `canPlaceLog`：从起点沿 direction 逐格检查（确定性，无 RNG）：任一格 `!TreeFeature.canReplace` → 整段放弃；下方不实心的格数累计 >2 → 放弃。检查后 pos 回退 `length` 格复位。
6. `generateLog`：逐格放 trunk 方块并以 `state.withIfExists(PillarBlock.AXIS, direction.getAxis())` 旋转轴；每格 `markBlocksAboveForPostProcessing`。完成后对全部 log 位置集跑 log_decorators。
- 依赖的方块机制：`canReplace`（同 TreeFeature）、下方实心判定 `isSideSolidFullSquare`、PillarBlock AXIS 属性、`Block.NOTIFY_ALL` 写入。**无 leaf_litter/wildflowers 直接依赖**（其新方块依赖经 decorator 间接，见 §3）。

### 1.3 随机数消费序（单一 Random 流，逐调用点）
| # | 调用点 | 消费 |
|---|---|---|
| 1 | `Direction.Type.HORIZONTAL.random` = Util.getRandom(facingArray) (Direction.java:612-614, Util.java:863-865) | `nextInt(4)` |
| 2 | `log_length.get(random)`（IntProvider uniform） | uniform 语义的 nextInt（移植需与 Rust ChunkRandom 对齐） |
| 3 | `random.nextInt(2)`（离桩偏移） | 1 次 |
| 4 | stump：`trunkProvider.get(random,pos)`（simple_state_provider 恒 0 次消费；weighted 才消费） | 0~N |
| 5 | stump_decorators 依序（本数据集 = trunk_vine）：每位置恒 4×nextInt(3)（西→东→北→南，与 Rust tree.rs 现有注释一致） | 4 次/位置 |
| 6 | generateLog 每格：`trunkProvider.get(random,pos)` | 0~N/格 |
| 7 | log_decorators 依序（本数据集 = attached_to_logs）：先 `Util.copyShuffled(logPositions, random)`（Fisher-Yates 整洗牌消费），后每位置恒 `Util.getRandom(directions)`=nextInt(len) + `nextFloat()<=prob`（无条件消费，即使失败）；成功再 `blockProvider.get`（weighted → 消费） | 见 AttachedToLogsTreeDecorator.java:34-43 |
- 顺序要点：stump decorators **先于** log 生成消费；attached_to_logs 的洗牌在方向抽取之前；decorator 共用同一 random（TreeDecorator.Generator 持引用）。

## 2. B6 机制 — PlaceOnGroundTreeDecorator（全文已读）

- 参数（CODEC L17-25）：`tries`（默认128）、`radius`（默认2）、`height`（默认1）、`block_state_provider`。
- generate（L44-76）：
  1. `list = TreeFeature.getLeafLitterPositions(generator)`：root 空 → 全 logPositions；root/log 都非空且首元素同 Y → log+root；否则 root。（list 来自 Generator 构造时按 Y 升序稳定排序的 logPositions/rootPositions，TreeDecorator.java:48-53。）
  2. list 空 → 直接返回，**零 RNG 消费**。
  3. 取 list 首元素 Y = i，在 `y==i` 的子集上求 XZ 包围盒 → `BlockBox.expand(radius, height, radius)`。
  4. `tries` 次循环，每次**恒 3 次** `random.nextBetween(min,max)`（X/Y/Z 各 1，无条件）。
- 单点放置（L78-85，三条全过才放）：`pos.up()` 是 air 或 vine；`generator.matches(pos, isOpaqueFullCube)`（pos 本体为不透明全方块 = 地面）；`getTopPosition(MOTION_BLOCKING_NO_LEAVES, pos).y <= pos.up().y`。成功才 `blockStateProvider.get(random, pos.up())`（weighted → 消费）+ replace(pos.up)。
- 注意：放置目标 = pos 的**上一格**，不是 pos 本身；两处实现都调 provider 但失败路径不消费。

## 3. 数据文件形态（盘上实存核对）

- `versions/1.21.6/data/worldgen/data/minecraft/worldgen/configured_feature/fallen_{birch,jungle,oak,spruce,super_birch}_tree.json` **5 个全部实存**；对应 placed_feature 5 个也实存。
- 形态（fallen_oak_tree.json 全读）：`type=minecraft:fallen_tree`；`log_length={type:uniform,min_inclusive:4,max_inclusive:7}`；`stump_decorators=[trunk_vine]`（oak/jungle 有，birch/spruce/super_birch **无 stump_decorators**）；`log_decorators=[attached_to_logs(probability 0.1, directions:["up"], weighted red/brown_mushroom)]`；`trunk_provider=simple_state_provider(oak_log axis=y)`。
- `*leaf_litter*.json`：9 个文件名匹配（7 CF + patch_leaf_litter + trees_birch_and_oak_leaf_litter）。**实际引用 `place_on_ground` decorator 的是 7 个 CF**（birch / birch_bees_0002 / dark_oak / fancy_oak / fancy_oak_bees_0002 / oak / oak_bees_0002 的 _leaf_litter，各 2 处 decorator 实例）；patch_leaf_litter 与 trees_birch_and_oak_leaf_litter 未 grep 到 place_on_ground（课题描述「9 处 CF 引用」与盘上 7 不符——诚实声明，可能把这两个间接引用计入）。
- 形态（oak_leaf_litter.json 全读）：`type=minecraft:tree` 普通 tree config + decorators 数组含 2 个 `place_on_ground`：实例1 tries=96/radius=4/height=2（leaf_litter segment_amount 1-3 × facing 4 向 weighted 各 weight1）；实例2 tries=150/radius=2/height=2（segment_amount 1-4）。`leaf_litter` 方块属性 = `facing`(N/E/S/W) + `segment_amount`(1-4)。

## 4. Rust 侧接线点与改动清单（worldgen-core）

现状：`grep fallen_tree|place_on_ground|leaf_litter` 在 worldgen-core **零命中**（两侧零支持，与 c1-exemption-list.md L25 一致）。

1. **feature type 分发**：`worldgen-core/src/feature_loader.rs`
   - `ConfiguredFeature::parse`（L34-78）：type 链 else 落 `eprintln! unknown configured feature type`（L75，显式告警非静默）→ 需加 `== "minecraft:fallen_tree"` 分支 + 新 `fallen_config: Option<FallenTreeConfig>` 字段。
   - `generate_configured`（L366-459）：同样 else→false → 需加 fallen_tree 分发。
   - `preload_all` 递归补载链（selector/patch 内嵌）无需改（fallen 不嵌 placed）。
2. **decorator 分发**：`worldgen-core/src/tree.rs`
   - `TreeDecorator` enum（L314-338）：现有 Cocoa/TrunkVine/LeaveVine/Beehive/Unsupported；parse else → `Unsupported` 告警。**需新增 `PlaceOnGround` 与 `AttachedToLogs` 两个变体**（后者 B5 数据强制依赖，B6 数据也走同一 parse 入口）。
   - `TreeDecorator::generate`（L340-446）：现有实现范式 = 逐位置、恒定 RNG 消费注释、WG_TREEDIAG 缺消费标记——新变体沿用此范式。
   - 位置集 plumbing：Java Generator 持 log/leaves/root 三集 + Y 排序；fallen 只传单集（stump={stump}，leaves/roots=∅）。place_on_ground 需要 root/log 双集合并逻辑（getLeafLitterPositions）——现有 tree.rs decorator 调用点（L593-602）只传单一位置集，需扩上下文。
3. **配套能力**：
   - IntProvider（uniform）解析：placement/tree 侧已有（patch/leaf config 在用），可复用。
   - weighted_state_provider：需确认 tree.rs 是否已有 parse（B5 蘑菇/B6 leaf_litter 均用）；若无是新增点。
   - 地面查询：fallen 的 `isSolidBelow`（isSideSolidFullSquare UP）与 place_on_ground 的 isOpaqueFullCube/MOTION_BLOCKING_NO_LEAVES heightmap——feature 层现有邻 chunk 地形列缓存（worldgen_handle.rs c-A 路径 L1112）可复用，但 heightmap 查询需接线。
   - log AXIS 旋转放置（x/z 轴）：现有树只放 axis=y log，需确认 state property 覆写支持。
4. **1.21.6 薄壳**：`versions/1.21.6/rust/` 实存且纯 ABI 适配；core 改完薄壳无需结构性改动（数据 worldgen/data 已在 versions/1.21.6/data）。

## 5. 执行域判定（关键结论）

- **1.21.6 运行时：features 阶段由 Java vanilla 执行，Rust 侧被 flag 关闭**。证据：`versions/1.21.6/rust/src/jni_bridge.rs:109`「Java CppBridge 在 init/initNether 后设 flag 关 Rust carver/features」+ `worldgen_handle.rs` FLAG_SKIP_FEATURES(bit1, L147)/apply_features skip 判定（L609）。C1 豁免清单（c1-exemption-list.md L24-25）同口径：「features 由 Java vanilla 在 Rust 地形上跑的场景不受影响」。
- **推论（B3 教训适用）**：生产双臂路径下 B5/B6 **无 Rust 侧观测对象**——Java vanilla 自己放倒木/leaf_litter，Rust 引擎是否实现不影响产物一致性。验证载体只有两种：① Degraded 静态（本轮已做）；② 运行时探针必须在 **Rust 独跑模式**（不设 skip flag / WG_SKIP_FEATURES 未设的 bin-diag 路径）下做 Rust-vs-Java feature 层对拍，或等 features 接管课题立项。**勿在生产路径跑无观测对象探针。**
- 实现价值定位：为「Rust features 接管」前置铺路 + 1.20.1↔1.21.6 引擎同构；不是当前生产一致性缺口。

## 6. 风险与盲区（诚实声明）

- @anchor.idk：`Direction.Type.HORIZONTAL` facingArray 的**具体元素顺序**未读（Direction.java L612 只确认 nextInt(4) 消费次数）；移植方向→序号映射时需读该数组字面量。
- @anchor.idk：`IntProvider.get`（uniform）在 1.21.6 Random 上的精确消费次数未逐一核（Rust 侧已有 uniform 移植可对拍，风险低）。
- @anchor.idk：课题描述「9 处 CF 引用 place_on_ground」与盘上 grep（7 CF）差 2——patch_leaf_litter / trees_birch_and_oak_leaf_litter 是否经其他机制引用未深挖。
- @anchor.idk：`Util.copyShuffled`（Fisher-Yates）的精确 RNG 消费序未读 Util.java 实现——attached_to_logs 洗牌对齐移植时必须核。
- Rust 侧 weighted_state_provider / log AXIS 覆写 / heightmap 接线的现状仅 grep 间接推断，未逐文件确认（改动清单开工时先核这三点）。
- 1.21.6 其他新树系机制（pale_oak/pale_moss/creaking_heart、mangrove attached_to_logs 族等）不在本勘探范围，但同走 tree.rs decorator parse else→Unsupported，可顺带盘点。
