# W4 辖区：world/biome 包语义 diff（1.20.1 → 1.21.6）

- 置信度：**draft**（静态 diff 审查，Degraded 分层；未做运行时验证）
- 树 A = `versions\1.20.1\data\mc_src_extract\net\minecraft\world\biome`
- 树 B = `versions\1.21.6\data\mc_src_extract\net\minecraft\world\biome`
- diff stat 总量：23 files, +497/-338。逐文件清单如下。
- 分类标签：**算法级**（公式/流程变）/ **数据级**（常量/参数表/注册数据变）/ **接口级**（签名/类型变但语义不变）/ **装饰级**（注释/格式/日志/命名）。

---

## 1. ★ 重点结论：MultiNoiseUtil 定点域分类器（Rust biome 路由是否需改）

**结论：1.21.6 对 MultiNoiseUtil 的 SearchTree / Node / quantize / distance / square-distance 完全没有任何改动（逐字节相同）。定点 long 域（TO_LONG_FACTOR = 10000.0F，`(long)(value * 10000.0F)`）、`ParameterRange.getSquaredDistance`、`SearchTree.TreeNode` 分割/距离最小化、`getSoundestBiome` 逻辑全部不变。Rust 复刻的 1.20.1 biome MultiNoise 路由**不需改**。**

全文件唯一语义变化在 `FittestPositionFinder.calculateFitness`（**仅影响 `/locate biome` 命令**，不影响 worldgen 逐块 biome 判定）：

```java
// 1.20.1:
double d = MathHelper.square(2500.0);
long l = (long)(MathHelper.square(10000.0F) * Math.pow((sq(x)+sq(z)) / d, 2.0));
long m = min over noises of getSquaredDistance(point);
return Result(pos, l + m);                       // fitness = 距离惩罚平方表 + biome 距离

// 1.21.6:
long l = min over noises of getSquaredDistance(point);   // biome 距离
long m = sq((long)x) + sq((long)z);                      // 平面距离平方
return Result(pos, l * sq(2048L) + m);                   // fitness = biomeDist * 2048^2 + r^2
```

→ 算法级变化但作用域是 locate 搜索 fitness 排序；搜索树采样（`findFittest` 的 2048/512、512/32 递归步进）不变。**对 Rust worldgen biome 路由：不需改**；若未来复刻 /locate，需按新公式。

`VanillaBiomeParameters.java`：
- **数据级（影响 overworld 路由表）**：`nearMountainBiomes[2][4]` = `DARK_FOREST` → `PALE_GARDEN`（1.21.4+ 新生物群系挤占山地过渡表一格）。**若 Rust 复刻要升 1.21.6 路由数据，此表需同步**；维持 1.20.1 行为则不改。
- 接口级/装饰级：`method_43718` → 改名 `inDeepDarkParameters`（公式不变，`erosion < -0.225 && depth > 0.9`）；`getWeirdnessParameters`/`method_40015` 两名互换（数值表不变）；`getWrapperOrThrow` → `getOrThrow`（仅 debug writeDebug 路径）。
- 参数范围表（temperature/humidity/continentalness/erosion/weirdness 全部 `ParameterRange.of(...)` 表）**无任何数值变化**。

## 2. biome\ 逐文件

| 文件 | 分类 | 关键变化 | Rust 移植影响 |
|---|---|---|---|
| Biome.java | 接口级 + 算法级（局部） | ① `getPrecipitation/isCold/doesNotSnow/computeTemperature` 全链新增 `int seaLevel` 参数；高温衰减阈值从硬编码 `pos.getY() > 80` 改为 `> seaLevel + 17`（主世界 seaLevel=63 → 80，**默认行为等价**，但非主世界维度会变）。② `canSetSnow/canSetIce` 高度检查改 `world.isInHeightLimit`（语义同旧 bottom/top 判断）。③ `getGrassColorAt` 拆出 `getGrassColor()`，逻辑等价。④ **新增 `getDryFoliageColor()`**（dry_foliage_color，默认 `DryFoliageColors.getColor(t,d)`）。⑤ `getMusic()` 返回 `Optional<Pool<MusicSound>>` + 新增 `getMusicVolume()`。⑥ `Precipitation` 枚举实现 StringIdentifiable + CODEC（序列化层）。 | ①② 对 1.20.1 复刻不需改（80 == 63+17）。③⑤⑥ 仅新增内容可忽略。④ 仅当移植 1.21.6 叶色时需加 dry foliage 通道 |
| BiomeKeys.java | 装饰级 + 数据级 | `register` → `keyOf`（`Identifier.ofVanilla`），**新增 `PALE_GARDEN`** 一个 key；其余 63 个 key 名称/顺序不变 | 仅新增内容可忽略（升 1.21.6 数据时加 pale_garden） |
| BuiltinBiomes.java | 数据级 | `DARK_FOREST` 注册改走 `createDenseForest(..., false)`，**新增 `PALE_GARDEN = createDenseForest(..., true)`** | 仅新增内容可忽略（1.20.1 行为不变） |
| OverworldBiomeCreator.java | 数据级（大） | ① `createBiome(...)` 链加 `dryFoliageColor` 参数（null 直通，1.20.1 等价）。② SpawnEntry 构造 API 变化：weight 移到 `builder.spawn(group, weight, entry)`（数值逐一对应，语义不变）。③ `createDarkForest` → `createDenseForest(pale)`。④ 新增生物：wolves 进 jungle/savanna/snowy 变体、**armadillo** 进 savanna/badlands、badlands 加 farm animals + 低 creature 概率（0.03/0.04）。⑤ 特性表微调：`addDefaultVegetation(lb, true)`、`addBushes` 进 plains/ windswept、`addDesertDeadBushes` → `addDesertDryVegetation`、冷/温海洋移除 `addSeagrassOnStone`、birch 加 wildflowers、plains 非 sunflower 分支重构（sugar cane/pumpkin patch 归入 addDefaultVegetation）。⑥ `DEFAULT_DRY_FOLIAGE_COLOR = 8082228` 新常量。**sky color 公式 `getSkyColor`、temperature/downfall、DEFAULT_* 色值全部不变** | 对 1.20.1 复刻**不需改**（全部是 1.21.x 内容新增/等价重构）；升 1.21.6 时按新 feature/ spawn 表重导数据 |
| TheEndBiomeCreator.java | 数据级 | `createTheEnd` 新增 `END_PLATFORM` TOP_LAYER_MODIFICATION feature（返回平台） | 仅新增内容可忽略 |
| TheNetherBiomeCreator.java | 接口级 + 数据级 | carver API：`carver(GenerationStep.Carver.AIR, ...)` → `carver(...)`（**Carver 步骤枚举被移除**，见 GenerationSettings）；spawn API 机械重排（数值不变）；无语义变化 | 不需改 |
| GenerationSettings.java | **接口级（结构变）** | carvers 存储从 `Map<GenerationStep.Carver, RegistryEntryList>` 改为单一 `RegistryEntryList`；`getCarversForStep()` 无参化；`GenerationStep.Carver` 枚举删除。**这是序列化/数据布局变化**：1.21.6 biome JSON 里 carvers 不再按步骤分桶。运行时 carver 应用顺序（ChunkGenerator carve 阶段统一跑全部）语义等价于旧的仅 AIR 步骤（1.20.1 实际只有 AIR 步被填充） | 1.20.1 复刻**不需改**；升 1.21.6 数据时 carver 解析需单列化 |
| BiomeEffects.java | 接口级 + 数据级 | 新增 `dry_foliage_color`（optional）、`music_volume`（float, 默认 1.0）、`music` 从 `Optional<MusicSound>` 改 `Optional<Pool<MusicSound>>`；`noMusic()` builder。旧字段/顺序兼容（optional 字段追加在中间但按名字匹配） | 不需改（新增字段可忽略） |
| SpawnSettings.java | **接口级** | `SpawnEntry` 从 `Weighted.Absent` 类改 **record**，weight 移出 entry 进 `Pool`（`builder.spawn(group, weight, entry)`）；MISC→PIG 替换逻辑保留；min>max 校验移到 `.validate`。**序列化格式不变**（weight 仍在 spawners JSON 的 weight 字段，由 Pool codec 承担） | 不需改 |

## 3. biome\source\ 逐文件

| 文件 | 分类 | 关键变化 | Rust 移植影响 |
|---|---|---|---|
| BiomeSource.java | 接口级 + 装饰级 | `getCodec()` 返回 `Codec` → `MapCodec`；`getTopY()` → `getTopYInclusive()+1`（等价边界重写）；`locateBiome` 搜索流程不变 | 不需改 |
| MultiNoiseBiomeSource.java | 接口级 | CODEC 从 `.xmap(...).codec()` 改直接暴露 `MapCodec`；**CUSTOM_CODEC/PRESET_CODEC、 Either 结构、biomeEntries、getBiome 全部不变** | 不需改 |
| MultiNoiseBiomeSourceParameterList.java | 装饰级 | `new Identifier(id)` → `Identifier.ofVanilla(id)`、lambda 参数改名；NETHER/OVERWORLD preset 内容不变 | 不需改 |
| MultiNoiseBiomeSourceParameterLists.java | 装饰级 | 同上 Identifier 工厂替换 | 不需改 |
| TheEndBiomeSource.java | 接口级 | Codec → MapCodec（`RecordCodecBuilder.create` → `mapCodec`）；**end island 噪声逻辑（getBiome 的 height/thickness 分支）逐行不变** | 不需改 |

## 4. 新增颜色类（仅存在于 B，粗看）

`BiomeColors.java`（新接口）= 把旧 `GrassColors/FoliageColors.getColor(t, d)` 里的 colormap 索引公式（`downfall *= temperature; i=(1-t)*255; j=(1-d)*255; k=j<<8|i; colormap[k]`，越界回退 fallback）**抽取为共享静态方法**；`GrassColors/FoliageColors` 改为薄委托（fallback: grass=-65281，foliage=-12012264），常量 SPRUCE/BIRCH/DEFAULT/MANGROVE 不变。**算法与 1.20.1 Biome 层色值逻辑逐位等价**。`DryFoliageColors` 是新 colormap 通道（fallback -10732494，pale garden 枯叶色），仅被 `Biome.getDryFoliageColor` 消费。这些类都在原 `net.minecraft.client.color.world` 移到 server 端 `world.biome` 包（客户端/服务端去耦重构），色值映射算法本身零变化。

## 5. 汇总对 Rust 复刻的判断

1. **biome MultiNoise 路由核心（定点量化/距离/搜索树）在 1.21.6 零变化 → 不需改。** 定点域分类器明确结论：quantize `(long)(value*10000.0F)`、`ParameterRange.getSquaredDistance`、`SearchTree.TreeNode` 逐字节与 1.20.1 相同。
2. 唯一路由数据变化：`VanillaBiomeParameters.nearMountainBiomes` 的 DARK_FOREST→PALE_GARDEN 一格（数据级，升版本才需要）。
3. `/locate biome` 的 FittestPositionFinder fitness 公式改写（算法级，但不进 worldgen 热路径，可忽略/未来复刻 locate 时再改）。
4. 其余全部为：序列化层 Codec→MapCodec 重构、SpawnEntry record 化、carver 步骤枚举移除、dry foliage/music pool 新增、spawn/feature 表内容新增 —— 对维持 1.20.1 行为的 Rust 工程**均不需改**。
