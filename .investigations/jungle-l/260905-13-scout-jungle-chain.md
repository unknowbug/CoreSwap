# 260905-13 scout — jungle_l 域 jungle 树族全链对拍勘探

- 日期锚定：Get-Date = 2026-09-05 23:06（标签 260905-13 合法）
- 角色：recode.scout（只读；本文件为唯一产物）
- 残差背景：jungle_l 域 −12055（region 树族块计数差，.artifacts/feature-parity/candidate-beehive-260905-10.md）；beehive 不覆盖此域

## 一、勘探范围

jungle 树族全链：jungle_tree（小树）/ mega_jungle_tree / jungle_bush 配置，trunk/foliage placer，trunk_vine + leave_vine + cocoa decorator，TreeFeature 集合构建与 decorator 驱动层；与 Rust worldgen-core/src/tree.rs 逐点对齐。

**Java 一手源定位（yarn mappings sources，非 javap）**：`.tmp\scout-260905-08\mcsrc\`（260905-08 session 解包的 yarn 1.20.1 sources 缓存，本 session 逐文件核读，行号一手）。以下引用省略前缀。
另核：`.tmp\net\minecraft\...` 为 mojmap 反编译缓存（仅作旁证，未采信）。

## 二、链路图（类/方法 → RNG 消费序）

```
TreeFeature.generate(FeatureContext)                    TreeFeature.java:117-165
  set/set2/set3/set4 = Sets.newHashSet()                :122-125   ← R-1 入口（HashSet）
  biConsumer2 → set2 = log positions                    :130-133
  blockPlacer  → set3 = leaf positions                  :134-145
  generate(...)（私有）：FoliagePlacer.getRandomHeight   :67   0 消费（jungle/blob 恒常量）
            FoliagePlacer.getRandomRadius               :69   radius.get(random)
            TrunkPlacer.generate                        :80
            foliagePlacer.generate × node               :81   offset.get(random) 1 次
  decorators 非空 → new Generator(set2,set3,set)        :153
  decorators.forEach(generate)                          :154   共用同一 random

MegaJungleTrunkPlacer.generate                          MegaJungleTrunkPlacer.java:31-53
  super = GiantTrunkPlacer.generate                     GiantTrunkPlacer.java:31-50
    setToDirt ×4（dirtProvider.get 仅实际替换时消费）     :34-37
    2×2 柱：每 log getAndSetState → canReplace 门 → trunkProvider.get 1 消费/根  :40-47
  分支循环：i = h-2-nextInt(4)，步 2+nextInt(4)          MegaJungle:37,50（各 1 消费）
    f = nextFloat()*2π                                  :38（1 消费/支）
    5 × getAndSetState（cos/sin 查表 65536 项）          :42-47（canReplace 才消费 1/log）
    TreeNode(startPos.add(j,i,k), -2, false)            :49

JungleFoliagePlacer.generate                            JungleFoliagePlacer.java:29-46
  i = giant ? foliageHeight : 1 + nextInt(2)            :40（非 giant 1 消费）
  层循环 k = radius + nodeRadius + 1 - j                 :42-44
  → FoliagePlacer.generateSquare                        FoliagePlacer.java:101-115
    isPositionInvalid（giant 归一化 min(|dx|,|dx-1|)）   :84-96
    isInvalidForLeaves(jungle)：dx+dz>=7 ‖ dx²+dz²>r²，0 消费   JungleFoliage:54-56
    placeFoliageBlock：canReplace 门 → foliageProvider.get 1 消费/块   FoliagePlacer:165-177

jungle_tree（小树）= StraightTrunkPlacer(4,8,0) + BlobFoliagePlacer(const2, const0, h3)
                                          TreeConfiguredFeatures.java:121-123, 355-363
  getHeight：nextInt(a+1)+nextInt(b+1) 各 1 消费          TrunkPlacer.java:54-56
  Blob 角判：dx==r && dz==r && (nextInt(2)==0 ‖ y==0)     BlobFoliagePlacer.java:55-57
    ← nextInt 仅在真角时消费（&& 短路在 RNG 之前）＝「树3+ 四角短路族」

jungle_bush = StraightTrunkPlacer(1,0,0) + BushFoliagePlacer(const2, const1, 2)
                                          TreeConfiguredFeatures.java:416-428
  Bush 角判：dx==r && dz==r && nextInt(2)==0              BushFoliagePlacer.java:41-43

Decorators（配置序：cocoa 0.2 → trunk_vine → leave_vine 0.25）
                                          TreeConfiguredFeatures.java:360（jungle_tree）
                                          :377（mega_jungle：trunk_vine → leave_vine）
  TreeDecorator.Generator 构造                           TreeDecorator.java:36-53
    logPositions / leavesPositions = new ObjectArrayList<>(Set)
    .sort(comparingInt(Y))  ← 稳定排序：同 Y 内序 = Java HashSet 桶序泄漏点（R-1 核心）
  CocoaBeansTreeDecorator.generate                       CocoaBeansTreeDecorator.java:28-45
    门 nextFloat()<p（1 消费）；list.get(0).getY() 基线；y-基线<=2 层；
    每向 nextFloat()<=0.25（恒 1 消费/向）；air 时再 nextInt(3)（age）
  TrunkVineTreeDecorator.generate                        TrunkVineTreeDecorator.java:19-50
    每 log 恒 4 次 nextInt(3)>0（西→东→北→南字面序）；air 才放
  LeavesVineTreeDecorator.generate                       LeavesVineTreeDecorator.java:26-70
    每 leaf 恒 4 次 nextFloat()<p（西→东→北→南字面序）；air → placeVines（向下 ≤4，0 消费）
```

## 三、对拍点清单表

Rust 侧 = `worldgen-core\src\tree.rs`。状态：✅已对齐（静态一手源对读）/ ⚠️疑似差 / ❌Rust 缺失 / @anchor.idk 未定界。

| # | 位置（Java） | Rust 对应 | 状态 | 预期效应量级 | 验证手段 |
|---|---|---|---|---|---|
| A1 | TrunkPlacer.getHeight :54-56（nextInt+1 各 1） | tree.rs getHeight 等价（straight 分支 ⑤⑥ 前消费） | ✅ | — | tree_dump 单树 trace |
| A2 | GiantTrunk 2×2 柱序 + 顶列缺 +1 | tree.rs:619-626 | ✅ | — | 同上 |
| A3 | setToDirt ×4 :34-37 | tree.rs:607-610 | ✅ | — | — |
| A4 | MegaJungle 分支循环消费序（nextInt(4)/nextFloat/nextInt(4)） | tree.rs:630-646 | ✅（cos/sin 查表已修，260905-08 .b1） | — | 单树 RNG 流 trace |
| **A5** | **MegaJungle 分支 log：getAndSetState → biConsumer2 → set2（trunk 集）** MegaJungleTrunkPlacer.java:46 | **tree.rs:640-643 仅 set_block，缺 `trunk_set.push`** | **❌ Rust 缺失（新发现，J1）** | 每横向枝 5 个 log 不入 trunk_set → TrunkVine decorator 每枝少 5×4=20 次 nextInt(3) 消费 + 枝干 vine 全缺 → **RNG 流级联**，mega_jungle 每树 1-4 支，域级可上千 | 单树 trunk_set 内容 diff + RNG 流对拍（tree_dump 加 dump 即可） |
| A6 | JungleFoliagePlacer 非 giant 首层 nextInt(2) :40 | tree.rs:206-208 | ✅ | — | — |
| A7 | Jungle isInvalidForLeaves dx+dz≥7/圆盘（归一化坐标） JungleFoliage:54-56 + FoliagePlacer:84-96 | tree.rs:259-262（ax,az 归一化后判） | ✅ | — | — |
| A8 | Blob 四角 nextInt(2) 短路族（jungle_tree 小树） BlobFoliagePlacer:55-57 | tree.rs:247-249（`ax==r&&az==r → nextInt(2)==0‖y==0`，短路点一致） | ✅静态；@anchor.idk：与「树3+ 状态依赖短路差」存量候选的关系未定界（需 trace 级证据） | 若真有差：小树每树 0-4 消费漂移 | block_probe 逐位单树对比 |
| A9 | Bush 角判（jungle_bush） BushFoliagePlacer:41-43 | tree.rs:256-258 | ✅ | — | — |
| A10 | TreeFeature 集合构建 = HashSet :122-125 + Generator 稳定 Y 排序 :50-52 | tree.rs:559-560 = Vec 插入序 + decorator 内稳定 Y 排序（tree.rs:343-344,365-366,381-382） | ⚠️ **疑似差（J2 = R-1 落点）**：同 Y tie 序 = HashSet 桶序 vs 插入序 | decorator 遍历序差 → air 判定结果不同 → 附着 vine/cocoa 放置集不同（RNG 流不变——每位置消费数恒定，但放置输出差；仅 cocoa 门内 nextInt(3) 依赖局部状态） | 构造单树 dump：两侧 leaves/log 集合逐位置比对同 Y 内序 |
| A11 | CocoaBeans 门 + 每向 nextFloat<=0.25 + air→nextInt(3) Cocoa:30-43 | tree.rs:337-361 | ⚠️ 部分：①`list.get(0)` = Y 稳定排序后首元素（tie 时 HashSet 序）vs Rust `min_by_key`（插入序首 min）——A10 子项；②cocoa AGE/FACING 属性位未编码（R-4，占位块 id） | 属性位差：若 region 计数按 block-state，cocoa 全差；按 id 则 0 | 核对 region 计数口径（状态 vs id）@anchor.idk |
| A12 | TrunkVine 每 log 恒 4×nextInt(3)，西东北京字面序 TrunkVine:21-49 | tree.rs:363-378 | ✅（序/消费一致；vine face 位未编码同 R-4） | — | — |
| A13 | LeavesVine 每 leaf 恒 4×nextFloat<p + placeVines 向下≤4 LeavesVine:28-70 | tree.rs:379-399 | ✅（0 消费下行一致） | — | — |
| A14 | decorator 执行序 = config.decorators 序 :154；条件 = set2‖set3 非空 :151 | tree.rs:586-591 | ✅ | — | — |
| A15 | placeLogsAndLeaves BFS distance 重写 :157-161,167-229 | tree.rs:592-596 占位 no-op（属性位未接线，已在案） | ⚠️ 已知（旧账 R-4 族） | leaves distance 属性差 | 属性位接线后回归 |
| A16 | TreeFeature 私有 generate 的域/藤校验循环 :71-115（含 ignoreVines isVine 检查 :101） | tree.rs:549-557 仅实现 ④⑤⑥ 简化守卫 | @anchor.idk 未定界：Rust 是否等价覆盖 isVine/ignoreVines 分支未逐行核对 | 0 RNG 消费，只影响 yes/no 与起始位 | 逐行读 TreeFeature:57-115 vs tree.rs 生成入口 |
| A17 | jungle_tree 配置本身（TreeConfiguredFeatures:355-363：cocoa 0.2 + trunk_vine + leave_vine 0.25；TwoLayersFeatureSize(1,0,1)+ignoreVines） | 数据驱动 JSON + tree.rs parse | ✅静态（decorator 集与序一致） | — | — |

## 四、互斥候选分叉清单

⚠️ 本域候选为**互补叠加**而非互斥机制（多条可同时贡献 −12055），但分叉 ≥2、各候选定界手段独立 → **建议 fan-out**：并行派 worker 各验一条，各产 .bN 定界证据，judge 汇总叠加归因。

| 候选 | 机制 | 预测（若成立） | 定界手段 |
|---|---|---|---|
| **J1（新发现，确定性 ❌）** | mega jungle 横向枝干 log 未入 trunk_set（tree.rs:640-643） | TrunkVine 每枝少 20 次 nextInt(3)；RNG 流自第一支起级联偏移 → 后续所有树结构整体漂移；修复=1 行 push | 单树 RNG 流 trace（修复前后对拍），**无需 fan-out 即可先修**（确定性缺失，非假设） |
| **J2（=R-1 定界）** | HashSet 桶序 vs Rust 插入序（A10/A11）：Java `newHashSet` + Y 稳定排序，同 Y tie 序不可复现 | vine/cocoa 附着集形状差、RNG 流不变；块计数差量级 = 每树个位数~十位数 vine | 单树集合逐位置序 dump 对比；不可复现性本身可用 Java 侧 dump 同 Y 序多次运行验证（HashSet 序跨 JVM 稳定但与插入序不同） |
| **J3（树3+ 短路族残余）** | blob 四角 nextInt(2) 短路在 jungle_tree（小树，Blob foliage）域内仍有差 | 小树每树 0-4 消费漂移，仅 jungle_tree 密度区贡献 | block_probe 单树逐位 + RNG 消费计数 |
| **J4（cocoa/vine 属性位 R-4）** | AGE/FACING/vine-face/distance 属性位未编码 | 计数口径含 state 时整族差；仅 id 时 0 | 核对 region 计数口径（id vs state）——**先查口径再定界，成本最低** |

建议执行顺序：J1 先修（确定性）→ 口径核对（J4 廉价）→ J2/J3 fan-out 并行定界。

## 五、未定界声明（@anchor.idk 风格）

- @anchor.idk("jungle_l region 计数口径（block id vs block-state）未确认——J4 归因前置", source="待：region 计数脚本口径核对")
- @anchor.idk("TreeFeature.java:57-115 私有 generate 的 isVine/ignoreVines 校验与 Rust 守卫等价性未逐行核对（A16）", source="待：逐行对读")
- @anchor.idk("Blob 四角短路族（J3）在 jungle 域的实际贡献未定界（静态已对齐，缺 trace 级证据）", source="待：单树 RNG 流 trace")
- @anchor.idk("HashSet 同 Y tie 序在真实 JVM 的具体桶序未采样（J2 影响幅度定界需要）", source="待：Java 侧单树 dump")

## 六、引用一手源清单

均在 `.tmp\scout-260905-08\mcsrc\net\minecraft\world\gen\`：
- `feature\TreeFeature.java`（:53-55 canReplace, :57-115 私有 generate, :117-165 驱动层, :167-229 placeLogsAndLeaves）
- `feature\TreeConfiguredFeatures.java`（:99-106 builder, :121-123 jungle(), :355-363 JUNGLE_TREE, :365 NO_VINE, :366-379 MEGA_JUNGLE_TREE, :416-428 JUNGLE_BUSH）
- `trunk\TrunkPlacer.java`（:54-56, :62-92）、`trunk\GiantTrunkPlacer.java`（:29-65）、`trunk\MegaJungleTrunkPlacer.java`（:31-53）
- `foliage\FoliagePlacer.java`（:38-115, :155-177）、`foliage\JungleFoliagePlacer.java`（:29-56）、`foliage\BlobFoliagePlacer.java`（:32-57）、`foliage\BushFoliagePlacer.java`（:23-43）
- `treedecorator\TreeDecorator.java`（:28-86）、`LeavesVineTreeDecorator.java`、`TrunkVineTreeDecorator.java`、`CocoaBeansTreeDecorator.java`（全文）

Rust 侧：`worldgen-core\src\tree.rs`（:67-266 placers, :305-449 decorators, :545-649 生成驱动 + mega_jungle_trunk）。
