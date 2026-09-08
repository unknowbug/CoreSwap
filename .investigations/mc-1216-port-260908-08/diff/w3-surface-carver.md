# W3 语义级 diff：surface rule + carver（1.20.1 → 1.21.6）

- 置信度：**draft**
- 验证分层：Degraded（静态源码 diff，yarn sources 两树逐文件 `git diff --no-index -U6`；无运行时探针）
- A = `versions/1.20.1/data/mc_src_extract`，B = `versions/1.21.6/data/mc_src_extract`
- 辖区：`world/gen/surfacebuilder/{MaterialRules,SurfaceBuilder,VanillaSurfaceRules}.java`、`world/gen/carver/` 全目录 15 文件

## 0. 总量结论

辖区 15 个共有文件中 **仅 6 个有差异**，且差异极小（合计 ~40 行改动）：

| 文件 | 差异规模 | 实质语义 |
|---|---|---|
| surfacebuilder/MaterialRules.java | +13/−9 | 接口级（Codec→MapCodec、Identifier 工厂）+ 1 处算法级（isCold 传 seaLevel） |
| surfacebuilder/SurfaceBuilder.java | +9/−5 | 接口级 + getSeaLevel 新增 |
| surfacebuilder/VanillaSurfaceRules.java | **0** | 完全一致 |
| carver/Carver.java | +6/−5 | 接口级（MapCodec、setBlockState 重载） |
| carver/ConfiguredCarvers.java | +1/−1 | 装饰级（Identifier.ofVanilla） |
| carver/NetherCaveCarver.java | +1/−1 | 装饰级（setBlockState 重载） |
| carver/RavineCarverConfig.java | +1/−1 | 装饰级（Codecs.NONNEGATIVE_INT → NON_NEGATIVE_INT 改名） |
| 其余 9 个 carver 文件（CaveCarver/CaveCarverConfig/CarverConfig/CarverContext/ConfiguredCarver/RavineCarver/CarvingMask/CarverDebugConfig） | **0** | 完全一致 |

**carver 核心算法（Carver.carveSkape/CaveCarver/RavineCarver 形状与概率公式）1.20.1→1.21.6 零变化。概率/形状语义无任何改动。**

## 1. 逐处变更分类

### 1.1 surfacebuilder/MaterialRules.java

| # | 位置 | 1.20.1 | 1.21.6 | 分类 | 语义影响 |
|---|---|---|---|---|---|
| M1 | `verticalGradient()` | `new Identifier(id)` | `Identifier.of(id)` | 装饰级 | 工厂方法迁移（Identifier 构造 API 变更），id 字符串语义不变 |
| M2 | `register()` / `MaterialCondition.registerAndGetDefault()` / `MaterialRule.registerAndGetDefault()` | `Codec<? extends A>` | `MapCodec<? extends A>` | 接口级 | DFU 序列化 API 升级（1.21.x Codec→MapCodec 泛化）。注册的 id 列表逐条比对**完全一致**（biome/noise_threshold/vertical_gradient/y_above/water/steep/temperature/hole/ridge + bandlands/block/sequence/condition），无新增条件/规则类型 |
| M3 | `BiomeMaterialCondition.equals()` | 参数名 `object` | 参数名 `o` | 装饰级 | 纯改名，零语义 |
| M4 | `MaterialRuleContext` 构造器 | 参数名 `registry` | 参数名 `biomeRegistry` | 装饰级 | 纯改名；调用侧 `.get(RegistryKeys.BIOME)` → `.getOrThrow(...)`（见 S3） |
| M5 | **`MaterialRuleContext.getSeaLevel()` 新增** | 无 | `return this.surfaceBuilder.getSeaLevel()` | 接口级（算法级的前置） | context 暴露 seaLevel 给谓词 |
| M6 | **`BiomeTemperaturePredicate.test()`**（`temperature()` 条件底层） | `biome.isCold(pos)` | `biome.isCold(pos, context.getSeaLevel())` | **算法级** | 见 §1.3 |

**pale_garden 对 MaterialRules 的侵入：无。** 1.21.x 新 biome（pale_garden 等）对 surface 的适配全部走 `VanillaSurfaceRules.java`，而该文件两版**逐字节一致**（`git diff` 空，`pale` 关键词两版均无命中）——pale_garden 表面（pale moss block 常规 `block()` 规则）在 1.21.6 的 yarn 源里要么注册于数据层/其他文件，要么尚未进入本辖区；对本辖区（MaterialRules 谓词机制 + VanillaSurfaceRules）**零侵入**。BiomeMaterialCondition 谓词机制本身也未变（仅 equals 参数改名）。

### 1.2 surfacebuilder/SurfaceBuilder.java

| # | 位置 | 1.20.1 | 1.21.6 | 分类 | 语义影响 |
|---|---|---|---|---|---|
| S1 | 构造器 clay_bands | `randomDeriver.split(new Identifier("clay_bands"))` | `split(Identifier.ofVanilla("clay_bands"))` | 装饰级 | split key 字符串相同 → **terracotta band 随机派生种子语义不变** |
| S2 | `setstate`（方法论内部 ChunkAccessor） | `y >= getBottomY() && y < getTopY()`；`setBlockState(pos, state, false)` | `isInHeightLimit(y)`；`setBlockState(pos, state)` | 装饰级 | 等价重载/谓词改写，边界判定语义相同 |
| S3 | `applyMaterialRule()` | `context.getRegistryManager().get(RegistryKeys.BIOME)` | `.getOrThrow(RegistryKeys.BIOME)` | 接口级 | registry 缺失时由返回 null 改为抛异常；正常路径取到的 registry 相同。另 `getRegistryManager()` 访问路径在 1.21.x 动态 registry 管理下语义微调（接口级，非辖区行为变化） |
| S4 | **`getSeaLevel()` 新增** | 无 | `return this.seaLevel` | 接口级 | 支撑 M6/S5 |
| S5 | `bucketFrozenOceanSurface`（冰山下层判定） | `biome.shouldGenerateLowerFrozenOceanSurface(mutablePos.set(x, 63, z))` | `...set(x, this.seaLevel, z), this.seaLevel)` | **数据级→参数化** | 1.20.1 硬编码 y=63（= overworld 海平面）；1.21.6 用 `this.seaLevel`。**对 overworld（seaLevel=63）数值完全等价**；跨维度（非 63 海平面）才出现差异。下游 Biome 侧（辖区外佐证）：`getTemperature(pos, seaLevel)` 中高度加成阈值由硬编码 `80` 改为 `seaLevel + 17`——63+17=80，**overworld 逐位等价** |

### 1.3 算法级变更汇总（唯一一处）

**温度高度加成的海平面参数化**（M6+S5+辖区外 Biome.java 联动）：
- 1.20.1：`computeTemperature(pos)` 高度阈值硬编码 `pos.getY() > 80`；surface/carver 调用链不传海平面。
- 1.21.6：全链传 `seaLevel`，阈值 `seaLevel + 17`，冰山判定采样 y = `seaLevel`。
- **overworld（seaLevel=63）下所有阈值数值逐位相同 → Rust 1.20.1 复刻的 `temperature()` 条件、冰冻海洋下层判定在 1.21.6 语义下不变**。这是给多维度参数化预留的接口级改动，不是行为变化。

### 1.4 carver/

| # | 文件 | 变更 | 分类 | 语义影响 |
|---|---|---|---|---|
| C1 | Carver.java | `Codec<ConfiguredCarver<C>>` → `MapCodec<...>`（字段/构造器/getCodec；`fieldOf("config").xmap(...)` 去 `.codec()` 尾调） | 接口级 | 序列化 API 升级，"config" 包装结构不变 |
| C2 | Carver.java `carveBlock` 等三处 | `chunk.setBlockState(pos, state, false)` → `setBlockState(pos, state)` | 装饰级 | 1.21.x Chunk 重载默认 moved=false，等价 |
| C3 | ConfiguredCarvers.java | `new Identifier(id)` → `Identifier.ofVanilla(id)` | 装饰级 | key 语义不变 |
| C4 | NetherCaveCarver.java | setBlockState 重载 | 装饰级 | 等价 |
| C5 | RavineCarverConfig.java | `Codecs.NONNEGATIVE_INT` → `Codecs.NON_NEGATIVE_INT` | 装饰级 | 常量改名，校验语义不变 |

**carver 概率/形状：CaveCarver.java、RavineCarver.java、CarverConfig.java、CaveCarverConfig.java、CarverContext.java、ConfiguredCarver.java、CarvingMask.java、CarverDebugConfig.java 两版全部逐字节一致。** carve 概率公式、tunnel/canyon 采样步进、taiga/terracotta 阈值均无改动。

## 2. mult=0 跨版本风险点核查

Rust 已知风险点 = VerticalGradientCondition / surface depth（secondaryDepth）相关乘法路径。核查结果：
- `verticalGradient` 工厂（M1）与 `VerticalGradientMaterialCondition` 类本体：**类本体零改动**，仅 Identifier 工厂调用改写 → **mult=0 风险点未被 1.21.6 触碰**。
- `sampleSecondaryDepth` / `surfaceSecondaryNoise`（SURFACE_SECONDARY 采样）：两版均无改动。
- `getSeaLevel()` 新增只读 accessor，不改变任何已有采样/乘法路径。

## 3. 移植影响裁决表

| 变更 | 分类 | 对 Rust 1.20.1 复刻的移植影响 | 裁决 |
|---|---|---|---|
| temperature 链 seaLevel 参数化（M6/S5/Biome.java `seaLevel+17`） | 算法级→参数化 | overworld 下数值逐位等价（80 = 63+17）；Rust 当前硬编码 80 与 seaLevel=63 均正确。仅未来多维度参数化时需改为 `sea_level + 17` | **无需改动**（多维度化时一并参数化） |
| MapCodec 迁移（M2/C1） | 接口级 | Rust 不走 DFU 序列化，规则为代码硬编码（数据驱动铁律下 overworld 保留代码规则）；注册 id 列表未变 | **无需改动** |
| pale_garden / 1.21.x 新 biome 侵入 | 无 | VanillaSurfaceRules 两版逐字节一致，MaterialRules 谓词机制无 biome 分支新增 | **无影响** |
| carver 概率/形状 | 无 | 9/11 个 carver 文件逐字节一致；有差异的 3 个均为 API 改名 | **无需改动** |
| NetherCaveCarver lava 阈值（min_y+31） | 无 | 两版逻辑一致，仅 setBlockState 重载 | **无需改动** |
| RavineCarverConfig Shape | 装饰级 | 字段/codec 字符串/校验全部不变（仅常量改名） | **无需改动** |
| Identifier 工厂（M1/S1/C3） | 装饰级 | Rust 侧 split key `"clay_bands"` 字符串不变 | **无需改动** |

**总裁决：本辖区 1.20.1 → 1.21.6 为「低噪声移植面」——唯一算法级改动（温度/冰冻海平面参数化）在 overworld 语义下逐位等价；carver 概率/形状与 surface rule 谓词机制零变化；mult=0 风险点未被触碰；pale_garden 等新 biome 对本辖区零侵入。Rust 工程无需为 1.21.6 兼容修改 surface rule / carver 任何代码逻辑。**
