# W4: net\minecraft\world\gen\structure\ 语义 diff（1.20.1 → 1.21.6 yarn）

- 方法：`git diff --no-index` 逐文件 + DimensionPadding 消费点 grep。
- 树 A = versions\1.20.1\data\mc_src_extract，树 B = versions\1.21.6\data\mc_src_extract。
- 置信度：**draft**（静态审查，Degraded 分层；未运行时验证）。
- 对 1.20.1 Rust 复刻影响三态：**需改 / 不需改 / 新增可忽略**。

## 总览

| 文件 | 分类 | Rust 1.20.1 复刻 |
|---|---|---|
| DimensionPadding.java（新增） | **算法级**（结构放置新过滤判据） | 新增可忽略（1.20.1 无此概念） |
| Structure.java | 接口级（+少量算法级） | 大体不需改；Shipwreck 用到新 helper（见下） |
| Structures.java | 数据级（构造写法重构 + TrialChambers 新增） | 不需改（bootstrap 是 vanilla 数据面；Rust 从 JSON 加载） |
| StructureKeys.java | 数据级 | 不需改 |
| StructureType.java | 接口级（Codec→MapCodec） | 不需改（Rust 不复刻 DFU 序列化层） |
| JigsawStructure.java | **接口级 + 数据级**（新增 dimension_padding / liquid_settings / pool_aliases 字段） | 不需改（1.20.1 语义保持默认值） |
| 各具体结构类 ×15 | 装饰级（Codec→MapCodec 泛型签名）+ 个别例外 | 不需改 |
| ShipwreckStructure.java | **算法级（位置修正）** | 1.20.1 无此逻辑 → 不需改，标记升级点 |

---

## 逐文件清单

### DimensionPadding.java（新增）【重点】
**分类：算法级**（placement 行为变化，但 1.20.1 复刻不受影响）

语义：`record DimensionPadding(int bottom, int top)`。
- 常量：`NONE = DimensionPadding(0)`（bottom=top=0）。
- 构造重载 `DimensionPadding(int value)` → 上下同值。
- CODEC：either(int, object)——JSON 里 `dimension_padding: 10` 或 `{bottom:.., top:..}` 两种形式；int 形式仅在 top==bottom 时往返。

**消费点（grep 全树，仅 3 处真实消费）**：
1. `JigsawStructure.java`：字段 + codec 字段 `dimension_padding`（默认 NONE），传递给 `StructurePoolBasedGenerator.generate(...)`。
2. `structure\pool\StructurePoolBasedGenerator.java#method_65173`（jigsaw 拼装起始件后调用，L113）：
   ```java
   if (dimensionPadding == DimensionPadding.NONE) return false;
   int i = heightLimitView.getBottomY() + dimensionPadding.bottom();
   int j = heightLimitView.getTopYInclusive() - dimensionPadding.top();
   return blockBox.getMinY() < i || blockBox.getMaxY() > j;
   ```
   即：**中心件（start piece）的包围盒若侵入「世界底部+padding / 顶部-padding」保护区，整个结构直接放弃生成**（返回 Optional.empty()）。只判中心件，不递归判子件。NONE 时完全跳过（引用相等判断）。
3. `Structures.java`：唯一实际给非默认值的 vanilla 条目 = **TrialChambers 用 `new DimensionPadding(10)`**（bottom=top=10）。

**不在 ChunkGenerator/NoiseChunkGenerator/ChunkStatus 消费**：grep `gen\chunk\*.java` 无任何 DimensionPadding 引用；它纯是 jigsaw 结构放置侧的「维度边界保护区」概念，与 terrain 阶段无关。

**vanilla 1.21.6 使用现状**：除 TrialChambers 外所有 JigsawStructure 条目显式传 `JigsawStructure.DEFAULT_DIMENSION_PADDING`（=NONE），行为等价 1.20.1。

**对 1.20.1 移植**：**新增可忽略**。1.20.1 无此概念，jigsaw 起始件 Y 由 start_height/heightmap 决定、无边界拒生逻辑；1.20.1 结构在维度边界的处理 = 「不检查」——例如 y=-15 的 trail ruins 之类可贴地生成、Nether 顶层也可放件，不会因 padding 被拒。Rust 复刻 1.20.1 时**不应**实现此过滤（实现了反而引入跨版本行为差）。仅作为升级点记录：若未来升 1.21+，TrialChambers 需 `padding=10` + `method_65173` 判据。

### Structure.java
**分类：接口级**，两处小算法级：
1. `createCodec` 返回 `Codec<S>` → `MapCodec<S>`（RecordCodecBuilder.create→mapCodec）——接口级。
2. `createStructureStart` 新增形参 `RegistryEntry<Structure> structure, RegistryKey<World> dimension` + JFR profiling（`FlightProfiler.startStructureGenerationProfiling` / `Finishable`）——纯观测，接口级/装饰级。
3. **新增 `public static int getAverageCornerHeights(...)`**（四角高度取平均）——算法级新 helper，仅被 ShipwreckStructure 消费（见下）。
4. `Structure.Config` 新增 `DEFAULT` 常量 + `Builder`（装饰级重写，字段与 codec 语义不变；`terrain_adaptation` 默认仍 NONE）。
- `getStructurePosition` / `getValidStructurePosition` / `isBiomeValid` / `getMinCornerHeight` / `getShiftedPos` / `Context` / `StructurePosition` **逐行无语义变化**（Context 的 ChunkRandom 构造 setCarverSeed 不变——placement 随机流未动）。
- **对 Rust**：不需改；`getAverageCornerHeights` 记入升级点（与 Shipwreck 绑定）。

### Structures.java
**分类：数据级**（vanilla 内置结构注册表 bootstrap，是数据面不是算法）：
- `createConfig` 静态工厂三重载删除，改为 `Structure.Config.Builder` 链式 / `new Structure.Config(biomes)` 单参构造——纯写法重构，**全部条目的 biomes/spawns/step/terrain_adaptation 值逐一等价**。
- `SpawnSettings.SpawnEntry` 构造从 `(type, weight, min, max)` 变 `(type, min, max)`（weight 移入 Pool.builder().add(entry, weight)）——影响面在 SpawnSettings/Pool 侧（W3/w-其他 section 范畴），数值 weight/min/max 全部不变（如 guardian weight=1 min=2 max=4）。
- `new Identifier("...")` → `Identifier.ofVanilla("...")`——装饰级。
- **新增条目：TRIAL_CHAMBERS**（1.21 新结构，见 JigsawStructure 节 + 下）：
  ```java
  Builder(biomes).step(UNDERGROUND_STRUCTURES).terrainAdaptation(ENCAPSULATE)
    .spawnOverrides(all groups → PIECE, empty pool).build(),
  pool=TrialChamberData.CHAMBER_END_POOL_KEY, size=20,
  UniformHeightProvider(fixed(-40), fixed(-20)), expansionHack=false,
  project=null, maxDist=116, aliases=TrialChamberData.ALIAS_BINDINGS,
  DimensionPadding(10), liquidSettings=IGNORE_WATERLOGGING
  ```
- **常量仍在代码里**：spacing/separation 等 placement 参数不在此文件（它们在 datapack JSON，1.20.1 与 1.21.6 同为数据级）；本文件内无 placement salt/spacing 硬编码变化。
- **对 Rust**：不需改（1.20.1 数据面已对齐；TrialChambers 是 1.21 新增可忽略）。

### StructureKeys.java
数据级：新增 `TRIAL_CHAMBERS = of("trial_chambers")`；`new Identifier(id)` → `Identifier.ofVanilla(id)`。**对 Rust：不需改。**

### StructureType.java
接口级：`codec()` 返回 `Codec<S>` → `MapCodec<S>`（与各结构 CODEC 改 MapCodec 配套，DFU 层等价变换）。注册条目集合无增删（jigsaw/mineshaft/…全部同名）。**对 Rust：不需改。**

### JigsawStructure.java
**分类：接口级 + 数据级**：
- 字段新增：`poolAliasBindings`（pool_aliases，1.20.5+ 引入）、**`dimensionPadding`**、**`liquidSettings`**（`StructureLiquidSettings`，默认 APPLY_WATERLOGGING）。
- codec 字段 `size` 范围 `intRange(0,7)` → `intRange(0,20)`（MAX_GENERATION_DEPTH=20）；validate 中 terrainAdaptation switch 新增 `ENCAPSULATE`（1.21 新枚举值，+12 同 BURY）。
- `getStructurePosition`：传递 `StructurePoolAliasLookup.create(bindings, blockPos, seed)` + padding + liquidSettings 给 `StructurePoolBasedGenerator.generate`——jigsaw 装配算法本体不变（变化在 pool 侧文件，非本 section）。
- **对 Rust 1.20.1**：不需改——1.20.1 无 pool_aliases/padding/liquid_settings，默认值全部等价旧行为；size 上限 7 对 1.20.1 复刻保持。

### 各具体结构类
**分类：装饰级**（全部仅 `Codec<X> CODEC = createCodec(...)` → `MapCodec<X>`，逻辑零变化）：
BuriedTreasure / DesertPyramid / EndCity / Igloo / JungleTemple / OceanMonument / Stronghold / SwampHut / WoodlandMansion / BasicTemple(无 diff)。

带额外小改的：
- **MineshaftStructure**：接口级（CODEC RecordCodecBuilder.create→mapCodec）+ 装饰级（`ValueLists.createIdToValueFunction`→`createIndexToValueFunction` 更名，语义不变）。placement/piece 逻辑无变化。
- **NetherFortressStructure**：数据级——`MONSTER_SPAWNS` Pool 写法 `Pool.of(entry(weight,min,max))` → `Pool.builder().add(entry(min,max), weight)`；五组 weight/min/max 数值逐一相同。**对 Rust：不需改。**
- **NetherFossilStructure**：接口级（mapCodec）。
- **OceanRuinStructure**：接口级 + 装饰级（新增 `@Deprecated ENUM_NAME_CODEC`，主 CODEC 不变）。
- **RuinedPortalStructure**：接口级 + **一处算法级签名变化**：`isColdAt(pos, biome)` → `isColdAt(pos, biome, chunkGenerator.getSeaLevel())`，`Biome.isCold` 增加 seaLevel 形参（温度判定基准面显式化，语义应等价——seaLevel 由 generator 提供；对 overworld 1.20.1 行为等价，但**机制依赖 Biome.isCold 侧 diff 确认**，本 section 未读 biome 包）。`new Identifier`→`ofVanilla` 装饰级。piece 放置/随机流无变化。**对 Rust：需核对 Biome.isCold 变化是否为纯重构；若是则不需改。**
- **ShipwreckStructure —— 本 section 唯一真算法变化**：
  ```java
  ShipwreckGenerator.Piece piece = ShipwreckGenerator.addParts(...);
  if (piece.isTooLargeForNormalGeneration()) {
      if (beached) j = findGroundedY(minCornerHeight, random);
      else j = Structure.getAverageCornerHeights(context, box...);
      piece.setY(j);
  }
  ```
  1.21 对「过大残骸」做 Y 重定位（固定 y=90 起始 → 最小角高度/四角均值落地）。**1.20.1 无此逻辑 → 对 1.20.1 Rust 复刻：不需改**；标记为已知跨版本行为差（升级点，依赖 ShipwreckGenerator.Piece.isTooLargeForNormalGeneration 的定义，structure 包外）。

---

## DimensionPadding 专项结论

1. **是什么**：`record DimensionPadding(int bottom, int top)`，jigsaw 结构专用的「维度垂直边界保护区」，codec 支持 int（上下同值）与 {bottom,top} 对象两种 JSON 形式；常量 NONE=(0,0)。
2. **谁消费**：仅 JigsawStructure（字段→codec→StructurePoolBasedGenerator.generate）→ `method_65173`：中心 piece 包围盒超出 `[worldBottom+bottom, worldTopInclusive-top]` 即整结构放弃（返回 empty，debug 日志）。**NoiseChunkGenerator / ChunkStatus / gen\chunk 全无引用**——它是放置期过滤，不是 chunk 生成管线概念。
3. **vanilla 用法**：仅 TrialChambers 用 `DimensionPadding(10)`；其余全部默认 NONE（行为=1.20.1）。
4. **1.20.1 边界行为**：无任何 padding 检查——jigsaw 起始件可贴/越维度上下界生成，不拒生。Rust 1.20.1 复刻**不实现**此判据即与 1.20.1 对齐；升级 1.21+ 时需补 `method_65173` + TrialChambers 的 padding=10/liquidSettings=IGNORE_WATERLOGGING/ENCAPSULATE/pool_aliases 全套 1.21 语义。
5. 关联但独立的概念：`StructureLiquidSettings`（APPLY_WATERLOGGING=1.20.1 等价默认 / IGNORE_WATERLOGGING）在 piece 液体处理层，属 structure\structure 包范畴，本 section 仅登记入口。

## 移植影响汇总（Rust 1.20.1 worldgen）

- **需改**：无（本 section 内无任何要求 1.20.1 复刻跟进的算法变化）。
- **待核对（跨 section）**：`Biome.isCold(pos, seaLevel)` 新签名是否纯重构（影响 RuinedPortal 判冷）；`ShipwreckGenerator.Piece.isTooLargeForNormalGeneration / findGroundedY / setY`（仅 1.21 行为差，不影响 1.20.1）。
- **新增可忽略**：DimensionPadding（1.20.1 无此概念，不实现即为对齐）、TrialChambers、pool_aliases、ENCAPSULATE、getAverageCornerHeights、JFR profiling 包装。
- **不需改**：其余全部（含 Structures/StructureKeys/StructureType/15 个具体结构类的 Codec→MapCodec 泛型层、Config Builder 重写、SpawnEntry 构造参数重排——数值全等价）。
