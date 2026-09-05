# s1-semantics-260905-05 — tree/植被 feature 族 S1 一手源语义提取（draft）

> 角色：core-worker（feature parity 课题 Phase 4a / S1）。置信度：**draft**（公式均有一手源行号，未运行验证）。
> 验证分层：**Degraded**（纯静态源码审读，无 probe）。
> 一手源 = `versions\1.20.1\data\mc_src_extract\`（官方 mappings，1.20.1）。下文 `file:line` 均相对该目录。
> 裁决对象 = b2-tree-plan-260905-05 §2.3 的 7 条 idk + §6 补录第 8 条（cocoa）。逐条给出：Java 源码摘录（file:line）+ 随机消费次数/顺序 + Rust 复刻公式。

---

## 0. 基础随机原语（后续所有公式的前提）

**`Random.java`（net\minecraft\util\math\random\Random.java）**：

- L72-74：`int nextInt();` / `int nextInt(int bound);` —— 每次调用消费 **1 次**底层 draw。
- L76-77：
  ```java
  default int nextBetween(int min, int max) {
      return this.nextInt(max - min + 1) + min;
  }
  ```
  → `nextBetween(a,b)` = **1 次消费**，等价 `nextInt(b-a+1)+a`。
- L84：`float nextFloat();` —— 1 次消费（24-bit draw，Rust 对应 `chunkrandom::ChunkRandom::next_float()`，chunkrandom.rs L133 同语义 `(next(24) as f32) * 5.9604645E-8`）。
- 边界：`nextInt(bound)` 要求 bound ≥ 1（bound=0 抛异常）；`nextBetween(a,a)` = `nextInt(1)+a` 仍**消费 1 次**（不短路）——**消费次数判定只看代码路径，不看参数范围**。

---

## 1. idk 逐条裁决

### idk-1（b2 §2.3 第 1 条）StraightTrunkPlacer：高度公式与消费序

**摘录 1 — 高度（基类 TrunkPlacer.getHeight）** `net\minecraft\world\gen\trunk\TrunkPlacer.java:54-56`：
```java
public int getHeight(Random random) {
    return this.baseHeight + random.nextInt(this.firstRandomHeight + 1) + random.nextInt(this.secondRandomHeight + 1);
}
```
→ **消费 2 次，顺序固定**：先 `nextInt(height_rand_a + 1)`，后 `nextInt(height_rand_b + 1)`。b2 的「base + nextInt(a+1) + nextInt(b+1)」形态**裁决为正确**。

**摘录 2 — StraightTrunkPlacer.generate** `net\minecraft\world\gen\trunk\StraightTrunkPlacer.java:30-40`：
```java
public List<FoliagePlacer.TreeNode> generate(
    TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random, int height, BlockPos startPos, TreeFeatureConfig config
) {
    setToDirt(world, replacer, random, startPos.down(), config);

    for (int i = 0; i < height; i++) {
        this.getAndSetState(world, replacer, random, startPos.up(i), config);
    }

    return ImmutableList.of(new FoliagePlacer.TreeNode(startPos.up(height), 0, false));
}
```

**摘录 3 — setToDirt** `TrunkPlacer.java:62-66`：
```java
protected static void setToDirt(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random, BlockPos pos, TreeFeatureConfig config) {
    if (config.forceDirt || !canGenerate(world, pos)) {
        replacer.accept(pos, config.dirtProvider.get(random, pos));
    }
}
```
- `canGenerate`（TrunkPlacer.java:58-60）= `Feature.isSoil(state) && !GRASS_BLOCK && !MYCELIUM`（纯 world 读，**0 消费**）。
- `dirtProvider.get(random, pos)`：`simple_state_provider` 的 get **不消费随机**（常量 state）；`weighted_state_provider` 的 get 消费 1 次（本数据集 S5 四树全部 simple —— oak/birch/jungle_tree/fancy_oak.json 实测 dirt_provider 均为 simple_state_provider）。**b2 推断「setDirtAt 不消费」裁决为：对 simple provider 成立，但代码路径存在消费点（provider.get），非无条件不消费。**

**摘录 4 — getAndSetState** `TrunkPlacer.java:68-86`：
```java
protected boolean getAndSetState(TestableWorld world, BiConsumer<BlockPos, BlockState> replacer, Random random, BlockPos pos, TreeFeatureConfig config) {
    return this.getAndSetState(world, replacer, random, pos, config, Function.identity());
}
protected boolean getAndSetState(..., Function<BlockState, BlockState> function) {
    if (this.canReplace(world, pos)) {
        replacer.accept(pos, (BlockState)function.apply(config.trunkProvider.get(random, pos)));
        return true;
    } else {
        return false;
    }
}
```
- `canReplace`（TrunkPlacer.java:94-96）→ `TreeFeature.canReplace`（TreeFeature.java:53-55）= `isAir() || isIn(REPLACEABLE_BY_TREES)`（0 消费）。
- **canReplace 不过时 trunkProvider.get 不调用**（短路）——对 simple provider 无随机差异，但数据边界上要注意。

**Rust 复刻（straight trunk，h = 传入 height）**：
1. `set_to_dirt(pos.down())`：`if force_dirt || !(below is soil-dirt && !grass_block && !mycelium) { set(dirt_provider_state) }`（simple provider：0 消费）
2. `for i in 0..h { if can_replace(pos.up(i)) { set(trunk_provider_state) } }`（0 消费/格）
3. 返回 TreeNode `{ center: pos.up(h), foliage_radius: 0, giant_trunk: false }`

### idk-2（b2 §2.3 第 2 条）BlobFoliagePlacer：radius/offset 取法与收缩公式

**摘录 1 — TreeFeature 主流程取 j/l（foliageHeight / radius）** `TreeFeature.java:66-69`：
```java
int i = config.trunkPlacer.getHeight(random);
int j = config.foliagePlacer.getRandomHeight(random, i, config);
int k = i - j;
int l = config.foliagePlacer.getRandomRadius(random, k);
```

**摘录 2 — BlobFoliagePlacer.getRandomHeight / FoliagePlacer.getRandomRadius** `BlobFoliagePlacer.java:50-52`、`FoliagePlacer.java:68-74`：
```java
@Override
public int getRandomHeight(Random random, int trunkHeight, TreeFeatureConfig config) {
    return this.height;                      // 常量，0 消费
}
public int getRandomRadius(Random random, int baseHeight) {
    return this.radius.get(random);          // IntProvider，消费次数随 provider 类型
}
private int getRandomOffset(Random random) {
    return this.offset.get(random);          // IntProvider
}
```
→ radius/offset 不是「radius + nextInt(offset+1)」复合式（b2 推测形态**裁决为错误**）；各自独立走 IntProvider。本数据集 S5 四树的 radius/offset 均为 **JSON 纯数字（constant）→ 0 消费**（oak.json L14-15 等）。若未来数据出现 uniform，则各消费 1 次（`nextInt(max-min+1)+min`，UniformIntProvider）。

**摘录 3 — Blob 层循环（收缩公式）** `BlobFoliagePlacer.java:32-47`：
```java
protected void generate(..., int trunkHeight, FoliagePlacer.TreeNode treeNode, int foliageHeight, int radius, int offset) {
    for (int i = offset; i >= offset - foliageHeight; i--) {
        int j = Math.max(radius + treeNode.getFoliageRadius() - 1 - i / 2, 0);
        this.generateSquare(world, placer, random, config, treeNode.getCenter(), j, i, treeNode.isGiantTrunk());
    }
}
```
→ 层数 = foliageHeight + 1（i 从 offset 到 offset-foliageHeight 递减）；层半径 `j = max(radius + nodeRadius - 1 - i/2, 0)`。**⚠️ `i/2` 是 Java int 除法（向零截断）**：i=0→0, -1→0, -2→-1, -3→-1（不是 floor 的 -1,-1,-2）。Rust：`java_div2(i) = if i >= 0 { i/2 } else { -((-i)/2) }`。

**摘录 4 — generateSquare（逐层方块）** `FoliagePlacer.java:101-115`：
```java
protected void generateSquare(..., BlockPos centerPos, int radius, int y, boolean giantTrunk) {
    int i = giantTrunk ? 1 : 0;
    for (int j = -radius; j <= radius + i; j++) {
        for (int k = -radius; k <= radius + i; k++) {
            if (!this.isPositionInvalid(random, j, y, k, radius, giantTrunk)) {
                mutable.set(centerPos, j, y, k);
                placeFoliageBlock(world, placer, random, config, mutable);
            }
        }
    }
}
```
→ 非 giant：dx,dz ∈ [-radius, radius]（方形，行序 j 外层 x、k 内层 z）。radius=0 时仅中心 1 格。

**摘录 5 — 角落排除（随机消费点！）** `BlobFoliagePlacer.java:55-57` + `FoliagePlacer.java:84-96`：
```java
@Override
protected boolean isInvalidForLeaves(Random random, int dx, int y, int dz, int radius, boolean giantTrunk) {
    return dx == radius && dz == radius && (random.nextInt(2) == 0 || y == 0);
}
protected boolean isPositionInvalid(Random random, int dx, int y, int dz, int radius, boolean giantTrunk) {
    int i; int j;
    if (giantTrunk) { i = Math.min(Math.abs(dx), Math.abs(dx - 1)); j = Math.min(Math.abs(dz), Math.abs(dz - 1)); }
    else            { i = Math.abs(dx); j = Math.abs(dz); }
    return this.isInvalidForLeaves(random, i, y, j, radius, giantTrunk);
}
```
→ **消费裁决**：仅当 `|dx|==radius && |dz|==radius`（正方形四角）时 `nextInt(2)` **消费 1 次**（`y==0` 是 `||` 的右支，短路在 nextInt 之后 → y==0 也先消费）。非角位置 **0 消费**。判定为 invalid（角落被跳过）当：`nextInt(2)==0` 或 `y==0`。b2 推断「逐叶 nextInt 判跳角」**裁决为：只判四角，非逐叶**。

**摘录 6 — placeFoliageBlock** `FoliagePlacer.java:165-177`：
```java
protected static boolean placeFoliageBlock(TestableWorld world, FoliagePlacer.BlockPlacer placer, Random random, TreeFeatureConfig config, BlockPos pos) {
    if (!TreeFeature.canReplace(world, pos)) {
        return false;
    } else {
        BlockState blockState = config.foliageProvider.get(random, pos);
        if (blockState.contains(Properties.WATERLOGGED)) { ... } // worldgen 处按 fluid 判定
        placer.placeBlock(pos, blockState);
        return true;
    }
}
```
→ simple provider 0 消费；**canReplace 不过 → 不进 provider.get**（simple 无随机差异）。oak_leaves JSON 自带 `distance:7`，放置后再由 placeLogsAndLeaves 重算 distance（见 idk-3 附注）。
→ 带 chance 的重载（FoliagePlacer.java:155-163，`nextFloat() > chance → false`）**blob 路径不经过**（那是 generateSquareWithHangingLeaves 用，cherry 系），blob 消费树里无 nextFloat。

### idk-3（b2 §2.3 第 3 条）TreeFeature 主流程放置顺序与跳过条件

**摘录 — 公开入口** `TreeFeature.java:117-165`（节选关键行）：
```java
public final boolean generate(FeatureContext<TreeFeatureConfig> context) {
    ...
    Set<BlockPos> set = Sets.newHashSet();   // root positions（无 rootPlacer 时空）
    Set<BlockPos> set2 = Sets.newHashSet();  // trunk（log）positions
    final Set<BlockPos> set3 = Sets.newHashSet();  // leaves positions
    Set<BlockPos> set4 = Sets.newHashSet();  // decoration positions
    ...（4 个 BiConsumer 各自 setBlockState + 记入集合）
    boolean bl = this.generate(structureWorldAccess, random, blockPos, biConsumer, biConsumer2, blockPlacer, treeFeatureConfig);
    if (bl && (!set2.isEmpty() || !set3.isEmpty())) {
        if (!treeFeatureConfig.decorators.isEmpty()) {
            TreeDecorator.Generator generator = new TreeDecorator.Generator(structureWorldAccess, biConsumer3, random, set2, set3, set);
            treeFeatureConfig.decorators.forEach(decorator -> decorator.generate(generator));
        }
        return BlockBox.encompassPositions(...).map(box -> {
            VoxelSet voxelSet = placeLogsAndLeaves(structureWorldAccess, box, set2, set4, set);
            StructureTemplate.updateCorner(structureWorldAccess, 3, voxelSet, box.getMinX(), box.getMinY(), box.getMinZ());
            return true;
        }).orElse(false);
    } else {
        return false;
    }
}
```

**摘录 — 私有 generate（真实放置序）** `TreeFeature.java:57-90`：
```java
int i = config.trunkPlacer.getHeight(random);            // ① 2 次消费
int j = config.foliagePlacer.getRandomHeight(random, i, config); // ② blob: 0
int k = i - j;
int l = config.foliagePlacer.getRandomRadius(random, k);  // ③ radius IntProvider
BlockPos blockPos = config.rootPlacer.map(...).orElse(pos); // 无 rootPlacer → 原点，0 消费
int m = Math.min(pos.getY(), blockPos.getY());
int n = Math.max(pos.getY(), blockPos.getY()) + i + 1;
if (m >= world.getBottomY() + 1 && n <= world.getTopY()) {            // ④ 高度域守卫
    OptionalInt optionalInt = config.minimumSize.getMinClippedHeight();
    int o = this.getTopPosition(world, i, blockPos, config);           // ⑤ 站点探测（0 消费）
    if (o >= i || !optionalInt.isEmpty() && o >= optionalInt.getAsInt()) {  // ⑥ 高度验收
        if (config.rootPlacer.isPresent() && !...) { return false; }   // 无 rootPlacer 跳过
        else {
            List<FoliagePlacer.TreeNode> list = config.trunkPlacer.generate(world, trunkPlacerReplacer, random, o, blockPos, config); // ⑦ 树干
            list.forEach(node -> config.foliagePlacer.generate(world, blockPlacer, random, config, o, node, j, l));               // ⑧ 树叶
            return true;
        }
    } else { return false; }
} else { return false; }
```

**放置顺序裁决**：**dirt 与 trunk 同在 trunkPlacer.generate 内部**（straight：先 `setToDirt(pos.down)` 再逐层 log）——不存在独立「先全图 dirt 再 trunk」阶段。全流程 = 高度随机（①-③）→ 域守卫 → getTopPosition → trunk（含 dirt）→ foliage → decorators → placeLogsAndLeaves（distance 重算）。b2 问题「先收集 foliage 再 trunk 还是交替」**裁决为：trunk 先、foliage 后，不交替**。

**getTopPosition（站点探测，0 消费）** `TreeFeature.java:92-109`：
```java
for (int i = 0; i <= height + 1; i++) {
    int j = config.minimumSize.getRadius(height, i);
    for (int k = -j; k <= j; k++) {
        for (int l = -j; l <= j; l++) {
            mutable.set(pos, k, i, l);
            if (!config.trunkPlacer.canReplaceOrIsLog(world, mutable) || !config.ignoreVines && isVine(world, mutable)) {
                return i - 2;      // 首个失败层 → o = i-2（可为负）
            }
        }
    }
}
return height;
```
- `getRadius`（TwoLayersFeatureSize.java:38-40）：`y < limit ? lowerSize : upperSize`。fancy_oak limit=0 → 每层 j=upperSize=0 → 单点探测。
- `canReplaceOrIsLog`（TrunkPlacer.java:98-100）= canReplace || isIn(LOGS)。
- **⑥ 验收**：`o >= i`（o 未受挫）或 min_clipped_height 存在且 `o >= minClippedHeight`（fancy_oak = 4）。o 可为负 → 一般直接 false。

**随机消费时序总表（oak/birch/jungle_tree：straight + blob + simple providers + constant radius/offset）**：
| # | 站点 | 消费 |
|---|---|---|
| 1 | getHeight | 2（nextInt(rand_a+1) → nextInt(rand_b+1)） |
| 2 | getRandomHeight/getRandomRadius/getRandomOffset | 0（blob 常量 height；constant IntProvider×2） |
| 3 | getTopPosition / 域守卫 | 0 |
| 4 | setToDirt | 0（simple dirt provider） |
| 5 | 每格 trunk log | 0（simple trunk provider） |
| 6 | foliage 每层每角（\|dx\|==j && \|dz\|==j） | 1× nextInt(2) |
| 7 | decorators 按 config 顺序（见 idk-6/7/8） | 见各条 |

### idk-3 附注（补充发现）placeLogsAndLeaves / TreeNode / distance 重算

**TreeNode 语义** `FoliagePlacer.java:188-213`：`(center, foliageRadius, giantTrunk)`；straight 返回 `(pos.up(height), 0, false)`。foliageRadius 参与 blob 层半径公式（idk-2 摘录 3 的 `treeNode.getFoliageRadius()`）；giantTrunk 影响 generateSquare 边界（+1）与 isPositionInvalid 的 normalize（FoliagePlacer.java:87-89）——S5 四树恒为 `false`。

**placeLogsAndLeaves** `TreeFeature.java:167-229`：树干/树冠放置完成后，从 trunkPositions 出发对**已放置的 leaves 块**做 7 级 BFS（`LeavesBlock.getOptionalDistanceFromLog` 只对含 DISTANCE_1_7 属性的方块非空），把每片叶子的 `distance` 重写为到最近 log 的实际距离（k=1..7；L198-200：k≠0 时 `blockState.with(DISTANCE_1_7, k)`）。decoration 集合（vine/cocoa）只进入 voxelSet（L178-182），**不会被 BFS 展开**（vine/cocoa 无 distance 属性 → 永不进队列 → 永不改写）。→ **Rust 必须复刻**：叶块终态 distance 不是 JSON 里的 7，而是 BFS 距离（对 palette 逐位对比是可见差异源）。0 随机消费。

### idk-4（b2 §2.3 第 4 条）TrunkVine / LeavesVineTreeDecorator 遍历序

**摘录 1 — Generator 构造（排序！）** `net\minecraft\world\gen\treedecorator\TreeDecorator.java:43-53`：
```java
this.rootPositions = new ObjectArrayList<>(rootPositions);
this.logPositions = new ObjectArrayList<>(logPositions);
this.leavesPositions = new ObjectArrayList<>(leavesPositions);
this.logPositions.sort(Comparator.comparingInt(Vec3i::getY));
this.leavesPositions.sort(Comparator.comparingInt(Vec3i::getY));
this.rootPositions.sort(Comparator.comparingInt(Vec3i::getY));
```
→ **只按 Y 升序排序（稳定排序）**；同 Y 内部顺序 = `ObjectArrayList(HashSet)` 的拷贝序 = **Java `HashSet<BlockPos>` 迭代序（hash 桶序）**。这是当前最大的复刻风险点：同 Y 多格（树冠）的遍历序直接影响 vine/cocoa 站点与随机消费分布。→ **遗留 idk R-1**（见 §2）：需复刻 BlockPos.hashCode + HashSet 桶迭代，或以对拍数据裁决近似序（如 xyz 序）的可接受度。b2 的「BlockPos 迭代 = xyz 序？」**裁决为：不是——是「Y 升序 + 同 Y hash 桶序」**。

**摘录 2 — TrunkVineTreeDecorator** `TrunkVineTreeDecorator.java:19-50`：
```java
generator.getLogPositions().forEach(pos -> {
    if (random.nextInt(3) > 0) { BlockPos blockPos = pos.west();  if (generator.isAir(blockPos)) { generator.replaceWithVine(blockPos, VineBlock.EAST); } }
    if (random.nextInt(3) > 0) { ... pos.east()  ... VineBlock.WEST  ... }
    if (random.nextInt(3) > 0) { ... pos.north() ... VineBlock.SOUTH ... }
    if (random.nextInt(3) > 0) { ... pos.south() ... VineBlock.NORTH ... }
});
```
→ **每 log 恒消费 4 次** `nextInt(3)`（无论结果），顺序固定 西→东→北→南；通过（>0，即 1 或 2）且目标为 air 才放 vine（放 vine 0 额外消费）。概率 = 2/3。

**摘录 3 — LeavesVineTreeDecorator** `LeavesVineTreeDecorator.java:26-70`：
```java
generator.getLeavesPositions().forEach(pos -> {
    if (random.nextFloat() < this.probability) { BlockPos blockPos = pos.west();  if (generator.isAir(blockPos)) { placeVines(blockPos, VineBlock.EAST, generator); } }
    if (random.nextFloat() < this.probability) { ... east ... }
    if (random.nextFloat() < this.probability) { ... north ... }
    if (random.nextFloat() < this.probability) { ... south ... }
});
private static void placeVines(BlockPos pos, BooleanProperty faceProperty, TreeDecorator.Generator generator) {
    generator.replaceWithVine(pos, faceProperty);
    int i = 4;
    for (BlockPos var4 = pos.down(); generator.isAir(var4) && i > 0; i--) {
        generator.replaceWithVine(var4, faceProperty);
        var4 = var4.down();
    }
}
```
→ 每 leaf **恒消费 4 次** `nextFloat()`（顺序 西→东→北→南）；通过且 air → 放 vine 并向下最长 4 格续藤（续藤只做 air 判定，0 消费）。b2 的「nextFloat() < probability」形态**裁决为正确**。

### idk-5（b2 §2.3 第 5 条）would_survive 判定语义

**摘录 1 — WouldSurviveBlockPredicate** `net\minecraft\world\gen\blockpredicate\WouldSurviveBlockPredicate.java:26-28`：
```java
public boolean test(StructureWorldAccess structureWorldAccess, BlockPos blockPos) {
    return this.state.canPlaceAt(structureWorldAccess, blockPos.add(this.offset));
}
```

**摘录 2 — SaplingBlock 无 canPlaceAt 重写**（`net\minecraft\block\SaplingBlock.java` 全文 66 行，grep 确认无 `canPlaceAt`/`canPlantOnTop`）→ 走父类链。

**摘录 3 — PlantBlock** `net\minecraft\block\PlantBlock.java:16-18, 30-33`：
```java
protected boolean canPlantOnTop(BlockState floor, BlockView world, BlockPos pos) {
    return floor.isIn(BlockTags.DIRT) || floor.isOf(Blocks.FARMLAND);
}
@Override
public boolean canPlaceAt(BlockState state, WorldView world, BlockPos pos) {
    BlockPos blockPos = pos.down();
    return this.canPlantOnTop(world.getBlockState(blockPos), world, blockPos);
}
```
→ **裁决：无光照判定**（光照只出现在 SaplingBlock.randomTick L34 的成熟检查，与 worldgen 的 would_survive 无关）。Rust 复刻 = `block_below ∈ DIRT tag ∪ {farmland}`。**数据边界声明**：`BlockTags.DIRT` 成员是 Java tag 数据（`versions\1.20.1\data\minecraft\tags\block\dirt.json`），Rust 侧应以 tag 数据驱动展开，不硬编码 id 列表。
→ b2 猜测「下方块为 dirt 系 + 光照？」**裁决为：dirt 系 tag + farmland，无光照**。

**「notTraceable」语义**：1.20.1 一手源中**不存在** `notTraceable` 符号（TreeFeature.java 全文核过）。b2 该词对应物最接近的是 `TreeFeature.canReplace`（TreeFeature.java:53-55，`isAir() || isIn(BlockTags.REPLACEABLE_BY_TREES)`）。**保留 @anchor.idk**：notTraceable 出处待澄清（可能为旧 mapping 名或其他版本符号），在获得出处前 Rust 不实现同名语义；canReplace 的公式以上行为准。

### idk-6（b2 §2.3 第 6 条 → 本轮升级）CocoaBeansTreeDecorator（概率消费 + 放置逻辑）

**摘录全文** `net\minecraft\world\gen\treedecorator\CocoaBeansTreeDecorator.java:28-45`：
```java
public void generate(TreeDecorator.Generator generator) {
    Random random = generator.getRandom();
    if (!(random.nextFloat() >= this.probability)) {          // ① 门：nextFloat() < probability，恒 1 消费
        List<BlockPos> list = generator.getLogPositions();
        int i = ((BlockPos)list.get(0)).getY();               // ② 最低 log 层 Y
        list.stream().filter(pos -> pos.getY() - i <= 2).forEach(pos -> {   // ③ 底部 3 层 log
            for (Direction direction : Direction.Type.HORIZONTAL) {          // ④ 水平 4 向循环
                if (random.nextFloat() <= 0.25F) {            // ⑤ 恒 1 消费/向（注意 <=）
                    Direction direction2 = direction.getOpposite();
                    BlockPos blockPos = pos.add(direction2.getOffsetX(), 0, direction2.getOffsetZ());
                    if (generator.isAir(blockPos)) {
                        generator.replace(blockPos, Blocks.COCOA.getDefaultState()
                            .with(CocoaBlock.AGE, random.nextInt(3))          // ⑥ air 时再 1 消费
                            .with(CocoaBlock.FACING, direction));
                    }
                }
            }
        });
    }
}
```
→ 消费序：门 1 次 nextFloat → 每个「底部 3 层」log × 4 向各 1 次 nextFloat（**注意判定是 `<= 0.25`，与 leave_vine 的 `<` 严格不等**）→ 命中且目标 air 时追加 1 次 `nextInt(3)`（cocoa age）。
→ **水平方向枚举序**：`Direction.Type.HORIZONTAL`（Direction.java:49-52）= 按 `idHorizontal` 升序 = **NORTH(0) → WEST(1) → SOUTH(2) → EAST(3)**（Direction.java:35-36 实证 WEST idHorizontal=1、EAST idHorizontal=3，NORTH=0/SOUTH=2 由枚举常量序推出——见 §3 保留项 R-2）。cocoa 的 FACING=direction（朝向树干），放置位 = log 沿 **direction 的反向**偏移一格（贴在 log 北侧的 cocoa 在 log 的 north 方向）。

### idk-7（b2 §2.3 第 7 条，本轮范围外但一并核）RandomSelector / RandomPatch

本轮 S1 范围（任务书 §1 覆盖清单）未含 RandomSelector/RandomPatch 源码裁决；b2 §2.3 第 6/7 条（selector 遍历、patch tries 偏移）**保留 idk，未裁决**，S6/S7 动工前补一轮同规格取证（`RandomSelectorFeature.java` / `RandomPatchFeature.java` 在同目录，路径已确认存在）。

### idk-8（b2 §6 第 8 条）cocoa 消费序与 vine 判别力

已由 idk-6 裁决。jungle_tree decorators 顺序 = `[cocoa(0.2), trunk_vine, leave_vine(0.25)]`（jungle_tree.json L4-16），三者共用同一 random（Generator 构造传入的同一个 Random 实例，TreeFeature.java:153）。**消费总序**：cocoa 门 nextFloat → cocoa 底 3 层 log ×4 向 nextFloat(+age) → trunk_vine 全部 log ×4 nextInt(3) → leave_vine 全部 leaf ×4 nextFloat。跳过 cocoa = 从第一个 vine 站点起系统性偏移——b2 §6 裁定 cocoa 必须随 L1 实现正确（本文档公式已给足）。

---

## 2. 遗留 idk / 保留项（本轮无法从源码确证）

- **R-1（高优）@anchor.idk**：`HashSet<BlockPos>` 同 Y 内部迭代序（影响 trunk_vine/leave_vine/cocoa 在树冠同层的站点序与随机消费分布）。复刻选项：① 实现 Java `Vec3i.hashCode`（x*3129871 ^ z*116129781 ^ y? 以 Vec3i.java 为准）+ HashSet 桶序模拟（成本高、精确）；② 以「Y 升序 + 同 Y xyz 序」近似，用单 seed palette 对拍 vine 位置检验近似误差。S5 验证时二选一并声明。
- **R-2（低优）@anchor.idk**：Direction.NORTH/SOUTH 的 idHorizontal 字面值（0/2）由枚举序推断，摘录未直击行（Direction.java:33-34 行未在本轮摘录内）；实现前读一次 Direction.java L30-34 即可闭合。
- **R-3**：`BlockTags.REPLACEABLE_BY_TREES` / `BlockTags.DIRT` / `BlockTags.LOGS` 的 Rust 侧数据源 = `versions\1.20.1\data\minecraft\tags\block\*.json`（存在性未在本轮核，S2/S4 接线时确认）。
- **R-4**：decorator 生成的 vine/cocoa 状态的属性名（east/west/north/south 布尔面、age、facing）需 BlockRegistry 支持对应属性写入；Rust 侧 state 编码方式（整型 id）如何携带属性 = 接线时对拍。

---

## 3. S5 四树配置实测（验证消费表用）

| 树 | trunk_placer | base/a/b | foliage | height/offset/radius | decorators | min_size |
|---|---|---|---|---|---|---|
| oak | straight | 4/2/0 | blob | 3/0/2 | [] | two_layers(1,0,1) |
| birch | straight | 5/2/0 | blob | 3/0/2 | [] | two_layers(1,0,1) |
| jungle_tree | straight | 4/8/0 | blob | 3/0/2 | cocoa(0.2), trunk_vine, leave_vine(0.25) | two_layers(1,0,1) |
| fancy_oak | **fancy**（LargeOakTrunkPlacer） | 3/11/0 | **fancy**（LargeOakFoliagePlacer，blob 子类） | 4/4/2 | [] | two_layers(0,0,0)+min_clipped_height=4 |

- **fancy_oak 特殊性**：trunk = LargeOakTrunkPlacer（`net\minecraft\world\gen\trunk\LargeOakTrunkPlacer.java:38-88`，分支公式：`g = f*(nextFloat()+0.328)` + `h = nextFloat()*2π`，**每候选分支恒消费 2 次**，L57-58）+ foliage = LargeOakFoliagePlacer（`LargeOakFoliagePlacer.java:26-46`，层半径 `j = radius + (i!=offset && i!=offset-foliageHeight ? 1 : 0)`，角落判定改圆盘 `square(dx+0.5)+square(dz+0.5) > radius²` **无随机消费**）。b2 §6 J9① 说「straight+blob 可覆盖 fancy_oak」**裁决为不准确**：fancy 是独立 placer 对，L1 需带 LargeOak* 实现或将 fancy_oak 移出 S5 判据（两难移交 patch 文档 §7 处置）。
- S5 三棵 straight 树每棵树叶层角落 nextInt(2) 消费次数 = Σ_layers 4×[j_layer>0]（每层 4 角）；j=0 层 0 次。oak/birch/jungle：radius=2, offset=0, nodeRadius=0 → j 序 = [max(2-1-0/2)=1, 1, 0, 0]（i=0,-1,-2,-3 → i/2=0,0,-1,-1 → j=1,1,0,0）→ **每树角落消费 = 1 层?** 核：i=0→j=1（4 次），i=-1→j=1（4 次），i=-2→j=0（0），i=-3→j=0（0）→ **每树 8 次 nextInt(2)**。

---

## 4. 结论

- b2 §2.3 七条 idk：**5 条已裁决清零**（idk-1/2/3/5/6），**2 条显式保留**（idk-7 selector/patch 留 S6/S7；idk-4 的同 Y 遍历序遗留 R-1）。idk-8（cocoa）已裁决。
- b2 §6 J9① 的「fancy_oak 由 straight+blob 覆盖」被一手源推翻（§3）——按 §15.4 记录取代：**fancy_oak 需要 LargeOakTrunkPlacer + LargeOakFoliagePlacer 独立实现**，公式已摘录（LargeOakTrunkPlacer.java:38-193 / LargeOakFoliagePlacer.java:26-46），可入 L1 或移 L3，由主会话拍板。
- L1（straight + blob + 三 decorator）编码的随机消费前置条件已满足；R-1（HashSet 序）为 vine 位置对拍的已知偏差源，须在验证口径中声明。

（draft，交主会话/judge；本文件不改 src。）
