# B5+B6 worker 交付 — fallen_tree feature + place_on_ground / attached_to_logs decorator（260908-15）

> core-worker 交付。状态：**未编译验证**（subagent 无 shell）。验证分层：Degraded（静态源码对拍，无运行时探针）。
> 改动文件：`worldgen-core/src/feature_loader.rs`、`worldgen-core/src/tree.rs`（直接 edit，无补丁文件）。

## 1. 改动文件清单

| 文件:函数 | 行为 |
|---|---|
| feature_loader.rs:ConfiguredFeature（struct） | 新增 `fallen_config: Option<FallenTreeConfig>` 字段 + parse 初始化 None |
| feature_loader.rs:ConfiguredFeature::parse | 新增 `== "minecraft:fallen_tree"` 精确分支（位于 catch-all **之前**，未触碰 catch-all） |
| feature_loader.rs:generate_configured | 新增 fallen_tree 分发：Some→FallenTreeConfig::generate 恒 true；None→告警 false |
| tree.rs:TreeDecorator（enum） | 新增 `PlaceOnGround { tries, radius, height, provider }` 与 `AttachedToLogs { probability, provider, directions }` 变体 |
| tree.rs:TreeDecorator::parse | 签名加 `blocks: &BlockRegistry`（新变体需解析 BlockStateProvider）；新增两分支（catch-all 之前）；directions 空 → 告警+Unsupported（对齐 Java nonEmptyList） |
| tree.rs:TreeDecorator::generate | 签名 `trunk_set/leaves_set` 改 `&[[i32;3]]` 并新增第三参 `root_set: &[[i32;3]]`（Java Generator 三集）；新增两 match 臂 |
| tree.rs:TreeFeatureConfig::generate | 唯一既有调用点同步：`d.generate(ctx, random, &trunk_set, &leaves_set, &[])`（现树无 root placer） |
| tree.rs:新自由函数 dir_vector / next_between / is_solid_ish / top_motion_blocking_no_leaves_y / generate_place_on_ground / generate_attached_to_logs | 见下 |
| tree.rs:FallenTreeConfig（struct + parse + generate + apply_decorators） | B5 主体 |

**被改调用点全列表**（decorator 上下文扩 root 集后）：
1. tree.rs:563 `TreeFeatureConfig::parse`（parse 加 blocks 参）
2. tree.rs:643 `TreeFeatureConfig::generate` decorator 循环（+`&[]`）
3. tree.rs:1170 `FallenTreeConfig::apply_decorators`（新）
4. tree.rs:1098/1101 `FallenTreeConfig::parse`（新）
grep 确认全仓无其它 `TreeDecorator::parse`/`.generate(ctx, random` 调用点；versions/1.21.6/rust 薄壳零引用，无需改。

## 2. 两个 idk 源码定论（一手源码已全文读）

- **idk-① Fisher-Yates 洗牌**：`Util.copyShuffled(List, Random)` = `Util.shuffle`（Util.java:1184-1191）：
  `for (j = n; j > 1; j--) { k = random.nextInt(j); swap(k, j-1); }`——降序 Fisher-Yates，j 从 n 到 2，每次 1 消费 nextInt(j)；n=0/1 零消费。attached_to_logs 逐位置消费在洗牌**全部完成之后**（AttachedToLogsTreeDecorator.java:37-43：nextFloat 门无条件消费，isAir 在消费后才评估，双门全过才 provider.get）。
- **idk-② facingArray 顺序**：`Direction.Type.HORIZONTAL`（1.21.6 Direction.java:601）= `[NORTH, EAST, SOUTH, WEST]`；`.random(random)` = `Util.getRandom(facingArray)` = `array[nextInt(4)]`（Util.java:863-865）→ Rust `HORIZONTAL_FACING = [(0,0,-1),(1,0,0),(0,0,1),(-1,0,0)]`。

## 3. scout 三待核点结论

1. **weighted_state_provider**：tree.rs 已有（BlockStateProvider::Weighted，get 1 次 next_int_bound(total_weight)）→ 直接复用，未新增。
2. **log AXIS=x/z 覆写**：引擎 state=i32 块 id 无属性位，**不可编码**——与 cocoa age（R-4）同族已知偏差；放置坐标/RNG 序不受影响，palette 对比时 x/z 向 log 会显示缺 axis 属性。代码注释已声明（FallenTreeConfig::generate ⑦）。
3. **heightmap/地面查询接线**：引擎无 MOTION_BLOCKING_NO_LEAVES heightmap（ocean_floor/world_surface 是 NOISE 阶段 WG 图，语义不等价）→ 选 **逐列扫描近似** `top_motion_blocking_no_leaves_y`（从世界顶向下首个 solid||water 且非树叶）。`isSideSolidFullSquare(UP)`/`isOpaqueFullCube` 无形状数据 → `is_solid_ish` 近似（非 air/water/REPLACEABLE/树叶；APPROX_NON_SOLID 表）。两近似均已注释声明数据边界。性能：place_on_ground 每候选 O(height) 扫描 × tries ≤150，仅 decorator 路径，可接受。

## 4. RNG 消费序对照表（Java ↔ Rust 逐点）

| # | Java（FallenTreeFeature.java 行号） | 消费 | Rust |
|---|---|---|---|
| 1 | generateStump→setBlockStateAndGetPos→trunkProvider.get（L38-39,111） | 0~N | trunk_provider.get |
| 2 | applyDecorators(stump)（L39,63）→ 本数据集 trunk_vine | 4/位置 | apply_decorators → TrunkVine 臂 |
| 3 | HORIZONTAL.random（L40,612） | nextInt(4) | next_int_bound(4) |
| 4 | logLength.get−2（L41） | uniform 1 | log_length.get + wrapping_sub(2) |
| 5 | nextInt(2)（L42） | 1 | next_int_bound(2) |
| 6 | moveToGroundPos / canPlaceLog（L43-44,49-59,66-87） | 0 | 同（确定性） |
| 7 | generateLog 每格 trunkProvider.get（L92-93,111） | 0~N/格 | 循环 provider.get |
| 8 | log_decorators：copyShuffled（Util:1184） | Σ_{j=2..n} 1 | (2..=n).rev() next_int_bound(j)+swap |
| 9 | 逐 log：Util.getRandom(directions)（Util:883-885） | nextInt(len) | next_int_bound(len) |
| 10 | nextFloat()<=prob（L40） | 恒 1（无条件） | next_float() |
| 11 | provider.get（仅双门过） | weighted 1 | provider.get |

place_on_ground（PlaceOnGroundTreeDecorator.java:44-85）：list 空→零消费短路（L46）✓；每 try 恒 3 次 nextBetween(min,max)=nextInt(max−min+1)+min（min==max 仍 1 消费，Rust next_int_bound(1) 同）✓；provider.get 仅三合一条件全过后（L83）✓；放置目标 = pos.up() ✓；BlockBox.expand 半径六面外扩 ✓；getLeafLitterPositions 三分支（TreeFeature.java:231-244）✓；Generator ctor Y 升序稳定排序（TreeDecorator.java:48-53）→ Rust clone+sort_by_key（稳定）✓。

## 5. 静态自检清单（强制五项）

1. **类型宽度**：全部 i32（Java int），f32（Java float：probability/next_float）。无 Java long 参与（本域无 64 位量）；bbox/坐标运算 i32，与 MC 世界坐标域一致。
2. **wrapping 算术点**：`log_length.get().wrapping_sub(2)`；`2.wrapping_add(next_int_bound(2))`；方向 offset `wrapping_mul/wrapping_add`；moveToGround `py.wrapping_add(1)/wrapping_sub(1)`；canPlaceLog/generateLog 游标 `wrapping_add`；bbox `wrapping_sub/wrapping_add(radius/height)`；place_on_ground `py.wrapping_add(1)`；next_between `wrapping_sub/wrapping_add`；Fisher-Yates 无算术溢出面（索引 usize 转换前 j∈[2,n]）。`suspended.wrapping_add(1)`。
3. **move/所有权/空容器**：parse 两新分支 `BlockStateProvider::parse(...)?` 失败→告警+Unsupported（不 panic）；directions 空→告警+Unsupported；place_on_ground list 空先短路再索引 `list[0]`（守卫后索引）；attached directions 由 parse 非空保证，仍以 Unsupported 兜底；`logs/roots` if-else 互斥 move 合法；`&[]` 字面量作 root_set 实参（&[[i32;3]] 协变）。无 unwrap/expect 新增。
4. **panic 路径**：新增代码零 unwrap/expect；`random.next_int_bound(bound)` 内部对 bound≤0 的行为与既有用法一致（本域 bound 恒 ≥1：nextInt(4)/(2)/(j≥2)/nextBetween(max−min+1≥1)/weighted 有 total_weight≤0 守卫（既有））；`directions[...]` 索引有非空前置；`list[0]` 有 is_empty 短路；`list.swap(k, j-1)` k<j≤n 界内。
5. **对拍点清单**：见 §4 对照表 + 数据文件（fallen_oak/birch_tree.json、oak_leaf_litter.json 全读核对字段名 tries/radius/height/block_state_provider/block_provider/directions/log_length/stump_decorators/log_decorators）。已知近似（声明）：①AXIS 属性位 ②is_solid_ish 三谓词近似 ③heightmap 逐列扫描 ④B6 leaf_litter facing/segment_amount 属性位（同 R-4，weighted 抽样/RNG 序不受影响）。

## 6. 建议编译/冒烟步骤（主会话执行）

```powershell
cargo build --offline -p WorldgenRust --release     # core 包名 WorldgenRust；dll 薄壳单独 -p worldgen
# 若 core 未单发：cargo build --offline -p worldgen --release（连带 core）
# 冒烟思路（Rust 独跑模式，勿设 WG_SKIP_FEATURES / skip flag——1.21.6 生产双臂下 Java 跑 features，无观测对象）：
#   bin-diag 加载 1.21.6 数据目录，断点调用 ConfiguredFeature::parse(fallen_oak_tree.json) →
#   断言 fallen_config.is_some()、decorators 解析出 PlaceOnGround/AttachedToLogs 非 Unsupported；
#   generate 固定 seed 跑 fallen_oak_tree，WG_TREEDIAG 打点核对 RNG 消费序列数（§4 表逐项计数）。
```
