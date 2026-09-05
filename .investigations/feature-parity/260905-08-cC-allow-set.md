# 260905-08 fan-out 候选 c-C：允许集语义差核实（core.worker）

- 角色：core.worker（fan-out 候选 c-C「允许集语义差」；沙箱无 shell，纯静态一手源对拍）
- 输入：scout-tree-rng.md + bB-set-diff.md + bA-stream-diff.md + SUBAGENT-KNOWLEDGE-GUIDE.md
- 状态：**draft**（Degraded 分层；未运行任何探针/命令——pwsh 只读 JSON 核对除外）
- **结论速览：c-C 主假设（允许集语义差）被证伪**——Rust Fix-2 门与 Java isFeatureAllowed 的集合语义**逐层等价**（同一 JSON、同一 placed-id 字符串、同一 step 全集并集）。但核实过程中抓到两个真差异：(i) Fix-1 biome_set 采样面 ⊊ Java 容器（含 trees_plains 被洞窟 biome 允许的实例）；(ii) **更重要的相邻发现：fid→p 全局索引的构建枚举序两侧不同**（Java registry 序 vs Rust BTreeMap 字典序），setDecoratorSeed(l,p,k) 用 p——p 错位 = 每次 feature 执行 seed 错位 = 双向残差，且 Fix-1 增加执行量会**按比例放大**该错位面——与「Fix-1 双向恶化」观测方向自洽。

## 1. 核实点逐条（file:line）

### 1.1 Java isFeatureAllowed 的精确集合 —— features steps **全集**，无 filteredFoo

`GenerationSettings.java`（.tmp/scout-260905-08/mcsrc/net/minecraft/world/biome/）：

- **:52** `features` = `List<RegistryEntryList<PlacedFeature>>`（per-step 列表，CODEC :44-47 直接反序列化 biome JSON 的 `features` 字段）。
- **:67-69** 构造器 memoize：
  ```java
  this.allowedFeatures = Suppliers.memoize(() -> (Set<PlacedFeature>)features.stream()
      .flatMap(RegistryEntryList::stream).map(RegistryEntry::value).collect(Collectors.toSet()));
  ```
  **= 全部 step 的 features 并集**（PlacedFeature 对象集），没有任何 filtered/裁剪逻辑；flowerFeatures（:59-66）才是 filtered（只 FLOWER 型），与允许集无关。
- **:88-90** `isFeatureAllowed = allowedFeatures.contains(feature)`——`feature` 是 BiomePlacementModifier 传入的**当前正执行的 PlacedFeature 实例**（BiomePlacementModifier.java:24-29），同 registry 对象，成员判定等价于 placed-id 成员判定。

### 1.2 Rust features_for 与 fid 的数据链 —— 同一 JSON、同一字符串形态

- **biome JSON 格式（实测 versions/1.20.1/data/worldgen/data/minecraft/worldgen/biome/{plains,jungle}.json）**：`features` 是**嵌套字符串数组**，元素即 placed id（`"minecraft:trees_plains"`、`"minecraft:trees_jungle"` 等，`minecraft:` 前缀齐全；1.20.1 数据无内嵌对象/两元组形态）。
- **加载**：`biome.rs:456-488` load_features——`f.as_str()`（:476）逐元素收字符串，key = biome id 原样（含前缀，:483）。placed id 直接就是 JSON 字符串，**无 configured↔placed 换算**（biome JSON 引用的本就是 placed id）。
- **features_for**：`biome.rs:491-493`——`features.get(biome)`，key 形态 = 采样返回的 biome id。
- **biome id 形态对齐（关键键位核实）**：biome 树叶值来自 `biome.rs:375-388` load（`versions/1.20.1/data/biome_params.json` 的 `biome` 字段）——**实测 7593 行、53 个唯一 biome、全部 `minecraft:` 前缀、0 个缺 biome/*.json 数据文件**。树值（:134/:152 `e.biome.clone()`）与 features key（:483 `id.clone()`）同一来源同一形态 → `features_for(&biome_at_jitter(...))` 查找键对齐，无 miss。
- **fid 形态**：`feature_loader.rs:112-140`——indexer 的 fid 全部来自同一批 biome JSON 字符串；`worldgen_handle.rs:908` `fid = step_list[p]`；placed cache 以同字符串为 key（:914，preload :219-268 同形态 strip 前缀找文件、**以带前缀 id 为 key 插入**）。**判定式 ：941-943 `f == fid` 是同源字符串全等，无 bare-vs-prefixed false-negative/positive 面**。

### 1.3 集合语义对拍 —— 等价

| | Java | Rust |
|---|---|---|
| 允许集 | `features` 全 step 并集 → `Set<PlacedFeature>`（GenerationSettings.java:67-69） | `features_for(b)` 全 step 遍历 `any(f == fid)`（worldgen_handle.rs:943） |
| 成员对象 | placed id（registry 实例） | placed id（字符串，同 JSON） |
| 采样 biome | BiomeAccess jitter → `getGenerationSettings(entry)`（BiomePlacementModifier.java:24-29） | `biome_at_jitter`（worldgen_handle.rs:849-853，biome_pick_cell 8 邻域） |
| 门位置 | 链中 `minecraft:biome` modifier 处 | `placement.rs:311-323` Biome 臂，同链位 |

**判定：c-C 主假设证伪**——只要 biome JSON 数据一致且 biome 采样一致，两侧允许集判定给出相同结果。剩余差异全部落在两个相邻层（§2）。

### 1.4 mega_jungle 无门（jungle 方向修正）

实测 placed_feature JSON 修饰链：`trees_plains`/`trees_jungle`/`trees_birch`/`trees_flower_forest` 都含 `minecraft:biome`（门生效）；**`mega_jungle_tree_checked` 只有 block_predicate_filter——无 biome 门**。含义：Fix-2 门对 mega_jungle（jungle 树冠大头）完全不生效；jungle 残差不能指望 Fix-2 收敛，jungle 缺失方向归执行集差（.b2a）+ 流内差（bA .b1/.b3）。

## 2. 核实中发现的真实差异（c-C 相邻层）

### 2.1 Fix-1 biome_set 采样面 ⊊ Java（IDK-1 的实例化，oak 相关）

- Java set（ChunkGenerator.java:346-353）= 9 chunk × **全 section 全 cell** 的 3D biome 容器并集（每 chunk 4×4×96 cell）。
- Rust Fix-1（worldgen_handle.rs:861-873，**当前已临时撤回只留 Fix-2，:876-882 E-B1.4b 实验态**）= 每 chunk 4×4 xz × **3 个 Y 切片**（wy∈{0,64,128}→cell y∈{0,16,32}）= 48/1536 cell。采样点是 Java 容器 cell 的子集 → **Rust set ⊆ Java set，恒不超集**。
- **oak 实例**：实测 `trees_plains` 被 **deep_dark.json 与 dripstone_caves.json 允许**（Select-String 全 biome 目录证实）。Java 容器并集含地下洞窟 biome cell → 这些 chunk 的执行集会**多出 oak 树执行**；Rust 3 切片若未命中洞窟 cell 则漏 → **oak rust少 分量**（不是 oak 过放方向）。同族：lush_caves 等洞窟 biome 的地表 step feature。

### 2.2 【强候选】fid→p 全局索引枚举序两侧不同 → setDecoratorSeed seed 错位

- Java indexer（ChunkGenerator.java:102）built from `biomeSource.getBiomes()`——**registry/BIOME source 迭代序**；:381-395 intSet/indexMapping 基于该全局序。
- Rust（feature_loader.rs:105-140 build）built from `all_features_lists()`（biome.rs:496-498）= `self.features` **BTreeMap 按 biome id 字典序**迭代 → first-occurrence 全局 index。
- **两侧枚举序几乎必然不同**（registry 序 vs 字典序）→ 同一 fid 的全局 p 不同 → `set_decorator_seed(population_seed, p, k)`（worldgen_handle.rs:909 ↔ ChunkGenerator.java:402）**seed 不同** → 每个执行 feature 的随机流起点错位 → 树放置双向偏差。
- **与恶化观测自洽**：p 错位是「执行越多、错得越多」的放大器——Fix-1 并集新增执行量 → 新增错位执行 → oak +29,708 恶化；Fix-2 门砍掉部分放置 → 部分回收（+26,385）。**「并集引入的新执行 feature 是恶化源」细化为「并集引入的新执行 + p 错位 seed」。**
- scout .b6 只核了 seed 派生算法逐行一致（属实），但 **p 的输入序未核**——本条把 .b6 的「排除」范围收窄为：算法一致 ✅，全局 index 输入序 ❌ 未对齐（新证据，b6 结论应被取代记录，非删除）。
- 注意边界：若 Rust 单 biome chunk 的 p 恰与 Java 一致（该 biome 恰为两序首现），则基线（无 Fix-1）部分对——与基线已有 119,198 残差不矛盾（不同 biome chunk 错位程度不同）。

### 2.3 jitter 采样保真度（bB 域，仅指界）

biome_at_jitter（worldgen_handle.rs:849-853）= biome_pick_cell + `<<2` 回读；Java = BiomeAccess.method_38106 加权最近格 + getBiomeForNoiseGen。若 biome_pick_cell 与 Java 有偏差，Fix-2 门在错误的 biome 上判允许集——归 bB 候选，本文不定界。

## 3. 量化预测（静态推算，draft）

| 假设 | 方向 | 预测 |
|---|---|---|
| 2.1 洞窟 cell 漏采 | oak/jungle **rust少** | 修复（Y 切片→全 cell 或加密采样）后：含洞窟 biome 的 chunk 出现 oak（dripstone/deep_dark→trees_plains）执行增量；残差变化量 = 该类 chunk 数 × trees_plains 平均树块，量级预计 ≪ 万级（洞窟 cell 只占容器小部分） |
| 2.2 p 枚举序 | **双向**，随执行量放大 | 若 WG_FEATURELOG 对拍 fid 集一致而 p 值不同（见 §4 E-C2），则修复（对齐 Java registry 序建 index）后全部 step/全部 biome 的树族残差系统性收敛；Fix-1 恶化现象应消失（并集后执行序与 Java 同序同 seed） |

## 4. 最小判别实验（主会话执行）

**E-C1（静态，判 2.1）**：python 脚本对 dump 区域逐 chunk 复算 Rust Fix-1 采样 recipe（48 点×3 切片）与 Java 全容器模拟（1536 cell）的 biome 并集差集，统计「Java 有 Rust 无」的 biome 及其 feature 是否含树族（重点 dripstone_caves/deep_dark→trees_plains）。
- 判据：差集非空且含洞窟 biome → 2.1 实锤（oak rust少 分量）；差集为空 → 2.1 对本区域无贡献。

**E-C2（判 2.2，一石二鸟，最优先）**：同一 chunk，Java mixin 打 `setDecoratorSeed(l,p,k)` 的 (p, fid)，Rust 开 `WG_FEATURELOG`（现成钩子 worldgen_handle.rs:910-911）对拍：
- **fid 集一致而 p 值不同 → 2.2 实锤**（seed 全错位，修 p 序为最高优先级修复）；
- fid 集也不一致 → 缺口部分归 2.1（执行集差）；p 一致 → 2.2 排除，恶化源回到 jitter/流内差。
- Java p 序可用一手规则静态推导：`biomeSource.getBiomes()` 迭代序 = registry 序（可从 Java 探针工程打印一次固定）。

**排除判据**：E-C2 中 p 与 fid 逐项一致 + E-C1 差集为空 → c-C 及其相邻两差异全部排除，恶化源只剩 jitter 采样（2.3，转 bB）与流内差（bA）。

## 5. 给主会话的行动建议

1. 先跑 E-C2（WG_FEATURELOG + Java mixin，单 chunk，最便宜且同时判 p 序与执行集）。
2. 若 p 错位实锤：修法 = feature_indexer 构建序改为 Java registry 序（需从 Java 侧导出 biome 迭代序一次，数据驱动落地），**优先级高于 Fix-1 恢复**——否则并集修复在错序 index 上会继续放大恶化。
3. Fix-1 当前处于撤回态（worldgen_handle.rs:876-882 TODO(restoring)），恢复前先修 2.2。
4. .b6 结论按 §15.4 取代链处理：原「p 差异不产生实质影响」收窄，supersedes 指向本文件 §2.2。

## 6. 边界与置信度

- 全文 Degraded（静态对拍 + 只读 JSON 实测）；status: draft。
- 未重验：biome_pick_cell 与 BiomeAccess 的位级等价（bB 域）；`biomeSource.getBiomes()` 的具体迭代序内容（需 Java 侧一次打印）。
- 数据核对面：biome_params.json 选了生产路径 versions/1.20.1/data/biome_params.json（.tmp 下另有 3 份副本未当生产）。
