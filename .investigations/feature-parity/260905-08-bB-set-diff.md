# 260905-08 fan-out 分支 B（.b2/.b5）——放置集合差核实（core.worker）

- 角色：core.worker（fan-out 分支 B：集合差 .b2 feature 集差 + .b5 heightmap 快照/越界拒绝）
- 输入：`.investigations/feature-parity/260905-08-scout-tree-rng.md`（scout 勘探产物）+ 一手源逐行核实
- 状态：**draft**（静态对拍，Degraded 分层；沙箱无 shell，未运行任何探针）
- Java 权威源：`.tmp/scout-260905-08/mcsrc/`（yarn 1.20.1+build.10-v2，版本已核）
- Rust：`worldgen-core/src/{worldgen_handle,placement,chunkrandom}.rs`
- 边界遵守：populationSeed 永久挂起域——本文所有修复均在 biome 采样/集合层，不触碰种子派生。

## 1. 逐行核实结果（.b2）

### 1.1 Java 3×3 邻域 biome 并集（scout :346-353 引用 → 属实 ✅）

一手源 `net/minecraft/world/gen/chunk/ChunkGenerator.java`：

- **:343-344** `chunkRandom = new ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))`；`l = setPopulationSeed(world.getSeed(), blockPos.getX(), blockPos.getZ())` ✅ 与 scout 接线图一致。
- **:345-353** `Set<RegistryEntry<Biome>> set = new ObjectArraySet<>()`；
  `ChunkPos.stream(chunkSectionPos.toChunkPos(), 1).forEach(chunkPosx -> { Chunk chunkx = world.getChunk(...); for (ChunkSection chunkSection : chunkx.getSectionArray()) { chunkSection.getBiomeContainer().forEachValue(set::add); } })`；
  `set.retainAll(this.biomeSource.getBiomes())`。
  **精确机制（scout 提问点的回答）**：
  - 遍历范围 = 中心 chunkSectionPos（bottom section）→ `.toChunkPos()` 还原为 chunk 坐标，半径 1 = **3×3 共 9 个 chunk**；
  - 每个 chunk 取 **全部 section（Y 全高）**，每 section 的 `BiomeContainer` 是 **4×4×4 cell（噪声格）**，`forEachValue` 把容器里**每个非空格的 biome entry 加入 set**（3D 容器，不只是地表层——洞窟生物群系如 lush/dripstone 若在容器中也入集；retainAll 只按 biomeSource 注册表过滤）；
  - 注意 chunk SectionPos.from(chunkPos, bottomSectionCoord) 的 Y 起点为最低 section，`.toChunkPos()` 丢弃 Y → 等价于「9 个 chunk 全高 biome 容器并集」。
- **:381-391** `intSet`：对 **set 中每个 biome** 取 `generationSettings.getFeatures()`，`k < list3.size()` 时把 `list3.get(k)`（该 biome 第 k 步的 placed feature 列表）map 成全局 index 经 `indexedFeatures.indexMapping` 后 `intSet.add` → **并集**。✅ scout「Java 是 3×3 并集」属实。
- **:393-402** `int[] is = intSet.toIntArray(); Arrays.sort(is)`；逐 p `setDecoratorSeed(l, p, k)` → `placedFeature.generate(...)`。每 feature 独立 reseed → **feature 集差异不产生 RNG 流污染，只改「哪些 feature 执行」**（scout 论证 ✅）。
- **:358** `int j = Math.max(GenerationStep.Feature.values().length, i)`——k 循环上限是 **全局 step 数**（i = 全局 indexed steps 数），**与中心 biome 无关**。❗scout 未列的追加差异：**Rust `worldgen_handle.rs:865` `let max_step = cur_features.len()`** 用的是当前 biome 的 features 列表长度——若某 biome 注册的 step 数少于全局（如仅到 VEGETAL_DECORATION 早停），Rust 会提前截断，漏掉 3×3 并集里其它 biome 在更高 step 的 feature。属 .b2 同族（集合差），并入修复。

### 1.2 BiomePlacementModifier 语义（scout Q2 差异点 3 → 属实，且需修正一处表述 ✅⚠️）

一手源 `net/minecraft/world/gen/placementmodifier/BiomePlacementModifier.java:24-29`：

```java
protected boolean shouldPlace(FeaturePlacementContext context, Random random, BlockPos pos) {
    PlacedFeature placedFeature = ...;
    RegistryEntry<Biome> registryEntry = context.getWorld().getBiome(pos);
    return context.getChunkGenerator().getGenerationSettings(registryEntry).isFeatureAllowed(placedFeature);
}
```

- **回答 scout 核实问题「用 POI 还是 random」**：都不用。继承 `AbstractConditionalPlacementModifier`（:10-12 `shouldPlace ? Stream.of(pos) : Stream.of()`）——**0 次 RNG 消费**（scout 表格「biome: 0」✅）。
- **采样机制**：`context.getWorld()`（ChunkRegion）→ `WorldView.getBiome(pos)`（WorldView.java:45-46）→ `getBiomeAccess().getBiome(pos)` → `BiomeAccess.java:30-63`：以 `hashSeed(world.getSeed())`（sha256, :22-23）为种子做 **8 邻域 jitter 选点**（method_38106 加权最近格），最后 `storage.getBiomeForNoiseGen(px, py, pz)` 读 4×4×4 噪声格。**即 Java = jitter 采样（与 biome 源判定同源），非 4×4 对齐采样**。
- **判定语义**：`isFeatureAllowed` = **jitter 采样点所在 biome 的 GenerationSettings 是否包含该 placedFeature**（允许集成员判定），**不是**「== 中心 chunk biome」。
- **Rust 对应**（`placement.rs:308-317`）：`PlacementModifier::Biome` 臂——`(x>>2)<<2` **4×4 对齐采样**（:313，无 jitter）`== anchor_biome`（= 中心 chunk biome，`worldgen_handle.rs:921-922` 注释自认「260905-05 patch §3.8：锚定 biome = 当前 chunk biome」）。注意 `ctx.pos_to_biome`（jitter 版，`worldgen_handle.rs:849-853,917`）**已存在且已传入 fctx，但 Biome 臂没用它**——修复只差接线 + 允许集判定。
- **双重语义差确证**：① 对齐 vs jitter；② 单锚等值 vs 采样点 biome 允许集。✅ scout 差异点 3 属实（补充：Rust 的 anchor 等值判定是 patch §3.8 引入的适配近似，非等价实现）。

### 1.3 Rust 侧 cur_features 集合（scout :856-858 引用 → 属实 ✅）

`worldgen_handle.rs`：

- **:817** 注释自认：「简化：set = 当前 chunk biome（Java 是 3×3 chunk 所有 biome section）」✅。
- **:856-859** `cur_biome_id = biome_at_no_jitter(cx, cz)`（chunk 角无 jitter 采样）；`cur_features = biomesrc.bc.features_for(&cur_biome_id)`；`if cur_features.is_empty() { return 0 }` ✅。
- **:871-873** `int_set = indexer.int_set_for(&cur_features, k)`——只从 cur_features 并集。✅。
- 另两个附带点（同核实确认）：:865 `max_step = cur_features.len()`（见 1.1 ❗）；:905-907 `block_at` 越界返回 -1（→ .b5，见 §2）。

### 1.4 .b5 核实（scout 中置信 → 两腿均升为静态确证 ✅）

- **腿 1：Java heightmap 实时更新**。`HeightmapPlacementModifier.java:27-32`：`k = context.getTopY(this.heightmap, i, j)` → `ChunkRegion.java:422-423`：`getChunk(sectionCoord).sampleHeightmap(...) + 1` → `Chunk.java:187-198` 读 chunk 的 heightmaps 表。而 `ProtoChunk.java:108,154`：`setBlockState` 内对每个 heightmap 类型执行 `trackUpdate(m, j, o, state)` → `Heightmap.java:73-99`（y>=当前值→set(y+1)；顶块被替换→向下重扫）。**结论：FEATURES 阶段每放一个方块，本 chunk（及邻 chunk 经 ChunkRegion.getChunk）的 heightmap 即时更新，后续 feature 的 heightmap 读取取到的是含先前树/feature 的实况**。Rust `worldgen_handle.rs:826-842` 的 `ocean_floor` 在全部 feature 前一次性快照、全程只读、**从不更新**——静态确证语义差。✅（scout「中置信」可升「静态确证」）
- **腿 2：越界保守拒绝**。`worldgen_handle.rs:902-908`：`block_at` 出本 chunk（或 Y 越界）返回 -1；谓词读 -1（非 air/非 dirt）→ 过滤拒绝。Java `ChunkRegion` 的 getBlockState/setBlockState 读**邻 chunk 实况**（ProtoChunk 3×3 region）。random_offset（xz_spread 常为 ±2~±8）可把树干推到 chunk 外 → Java 能判定并放置（跨 chunk 写入），Rust 直接拒绝。✅ 静态确证。
- 附带：`placement.rs:301` 邻域 heightmap 查询返回 `min_y-1` → k=min_y → `vec![]`（:305 拒绝）；:330/:357 同类邻域直通保留——同属「邻 chunk 数据缺失」家族。

## 2. 机制→签名量化预测

| 候选 | 机制方向 | 受益族 | 预测残差签名（修复后收敛方向） |
|---|---|---|---|
| .b2a 3×3 并集缺失 | Java 在「中心非 X 但 3×3 内有 X」的边界 chunk 额外执行 X 的 feature；Rust 全漏 | jungle_edge/oak 等所有交界族 | **rust 少**（单向，缺整树）：jungle −22,611 的边界分量；oak 的森林/平原交界带负分量 |
| .b2-maxstep 截断 | k ≥ cur_features.len() 提前停 | 高 step 注册 biome 的 feature | rust 少（量级小，需 WG_FEATURELOG 验证实际截断面） |
| .b2b Biome modifier 语义 | rust 多：对齐采样==锚（中心同质 chunk 内）即放行，Java 要求 jitter 点 biome 允许该 feature——混合 chunk（含 river/沼泽 cell）内 rust **过放**；rust 少：中心≠锚的 chunk 内 rust 全拒，Java 只拒非允许 cell | oak（允许集宽：forest/plains/birch/flower_forest 等 → 过放面大）；jungle（允许集窄≈仅 jungle → 过放面小、拒出面小） | **oak 双向但净 +**（过放主导 → 可解释 oak +38,250 的主体）；jungle 净效应小 |
| .b5 快照静态 | Java 树叠树（后树落前树顶）；Rust 后树落原地 → 谓词/重叠拒绝 | 密集林区（oak 密林、jungle） | rust 少（密集区）；叠加越界拒绝 → chunk 边缘树 rust 少 |

**综合签名判断**：
- **jungle −22,611**：.b2a（jungle 允许集窄，交界 chunk 系统性整树缺席）为主集合差来源；jungle 域的过放抵消面小。注意分支 A 的 .b1（MegaJungle cos/sin）与 .b3（vine 迭代序）同为 jungle 负贡献——集合差修复后 jungle 残差不会归零，剩余归流内差。
- **oak +38,250**：.b2b 过放机制（宽允许集 + 锚等值放行）+ .b2a 的 oak 负分量互相抵消后仍净 +，机制上成立；但**无法静态排除** .b4（RarityFilter/RandomOffset 抽法）与 biome 判定联合贡献，须 A/B 实验定份额。
- .b5 修复预期对 oak/jungle 都是「密林区收敛」，量级次级。

## 3. 精确修复方案（diff 级，全部 biome 采样层，不碰 populationSeed）

### Fix-1（.b2a + maxstep）：3×3 biome 并集

`worldgen-core/src/worldgen_handle.rs` apply_features：

```rust
// 删除 :856-859：
//   let cur_biome_id = biome_at_no_jitter(cx, cz);
//   let cur_features = self.biomesrc.bc.features_for(&cur_biome_id).to_vec();
//   if cur_features.is_empty() { return 0; }
// 替换为（ChunkGenerator.java:346-353 对齐）：
let mut biome_set: Vec<String> = Vec::new();           // 有序去重集
for nx in (cx - 1)..=(cx + 1) {
    for nz in (cz - 1)..=(cz + 1) {
        // 每 chunk 4×4 噪声格 x/z（对应 Java BiomeContainer 的 x/z 平面）；
        // Y 全高并集在 overworld 地表 feature 域退化为采样若干 Y 切片（见 IDK-1）
        for gx in 0..4 { for gz in 0..4 {
            let wx = nx * 16 + gx * 4;
            let wz = nz * 16 + gz * 4;
            for wy in [0, 64, 128] {                    // Y 切片近似，见 IDK-1
                let b = biome_at_no_jitter_at(wx, wy, wz);   // 需把 biome_at_no_jitter 泛化出 y 参数
                if !biome_set.contains(&b) { biome_set.push(b); }
            }
        }}
    }
}
// 注意保留 cur_biome_id（:922 anchor_biome 在 Fix-2 落地前仍需要；落地后可删）
// :865 改：
let max_step = indexer.step_features.len();   // Java :358 max(values().length, i) ≥ 全局步数
// :871-873 改（intSet = ∪ set 中各 biome 的 step k）：
let mut int_set: Vec<i32> = Vec::new();
for b in &biome_set {
    let feats = self.biomesrc.bc.features_for(b);
    for p in indexer.int_set_for(feats, k as i32) {
        if !int_set.contains(&p) { int_set.push(p); }
    }
}
int_set.sort_unstable(); int_set.dedup();     // Java :394-395 toIntArray+sort
for p in int_set { ... }                       // 其余循环体不变（:874 起）
```

### Fix-2（.b2b）：Biome modifier 改 jitter 采样 + 允许集

- `worldgen-core/src/worldgen_handle.rs`：fctx（:910-923）新增字段
  `biome_allows: Option<&dyn Fn(i32, i32, i32, &str) -> bool>`，
  构造闭包：`|x, y, z, fid| self.biomesrc.bc.features_for(&biome_at_jitter(x, y, z)).iter().any(|p| indexer 由 p 得 fid 命中)`——实现上建议预构建 `HashSet<fid>` per 闭包捕获（features_for 直接给 fid 列表即可 contains(fid)，无需 index 反查）。
- `worldgen-core/src/placement.rs:308-317`：

```rust
PlacementModifier::Biome => {
    // Java BiomePlacementModifier.java:24-29：BiomeAccess jitter 采样 →
    // 采样点 biome 的 GenerationSettings.isFeatureAllowed(placedFeature)
    match (&ctx.biome_allows, &ctx.feature_id) {
        (Some(allows), Some(fid)) => {
            if allows(x, y, z, fid) { vec![[x, y, z]] } else { vec![] }
        }
        _ => vec![[x, y, z]],   // 未接线时直通（保持现防御姿态）
    }
}
```
  需同步给 FeaturePlacementContext 加 `feature_id: Option<String>`（:878 已有 `fid`，直接传入）；落地后删除 `anchor_biome`（:921-922）与对齐采样分支（避免双口径残留）。

### Fix-3（.b5 腿 1）：heightmap 实时更新

- `worldgen_handle.rs`：`ocean_floor` 改 `mut`，在 feature 放置写块的统一出口（octx 写块处 / tree set_state 处）后追加 `Heightmap.trackUpdate` 语义：

```rust
// Heightmap.java:73-82（放置分支：y >= 当前 → set(y+1)）
fn track_update(hm: &mut [i32], min_y: i32, lx: i32, lz: i32, y: i32) {
    let i = hm[(lz * 16 + lx) as usize];
    if y >= i { hm[(lz * 16 + lx) as usize] = y + 1; }   // OCEAN_FLOOR predicate=固体，放置必为固体
}
```
  world_surface（`heightmap` 参数）同理需 mut 化（Java 对 WORLD_SURFACE_WG/OCEAN_FLOOR_WG 同步 trackUpdate）。注意：被替换方块的下移重扫分支（Heightmap.java:83-96）在纯放置场景不触发，登记 idk 即可。
- .b5 腿 2（越界读邻 chunk 实况）：**本轮不动**（需 3×3 region 列缓存，改动面大，独立成后续候选）；登记为残留，用判别实验量化其贡献后另立候选。

### 声明

- 以上 diff 未编译验证（沙箱无 shell）；类型/借用自检：`biome_at_no_jitter` 泛化 y 需同步改签名及 ：845 调用点；`biome_allows` 闭包捕获 `&self` + `indexer` 引用，fctx 生命周期同域（与 ：901 block_at_col 同一并发不变量，judge C-4 论证可复用）。
- `@anchor.idk` 候选：IDK-1——Java set 是 3D biome 容器全量并集（含洞窟生物群系），Rust 用 Y 切片近似；洞窟 biome 的 features 若含地表 step（如 lush 的 flower），近似会漏。可用 WG_FEATURELOG 对拍确认截断面后决定是否全量化。

## 4. 最小判别实验（主会话执行）与排除判据

**E-B1（判 .b2，优先，单点 A/B）**：
1. 用 WG_FEATURELOG 锁定一个「中心=plains/forest、3×3 内含 jungle」的边界 chunk（从确定性 dump 树族残差表中挑 jungle 双向差最大的边界 chunk；单 chunk 代价最小）。
2. 现状跑一次：Java 侧 mixin 打 `setDecoratorSeed(l,p,k)` + fid 日志（scout §6.2 方案），Rust 开 WG_FEATURELOG——对比同 chunk 的 p 集合。**判据：Rust 缺失的 p 恰为 jungle biome 独有 feature → .b2a 实锤**；顺带暴露 biome 枚举序差（p 值整体漂移即 index 层问题，scout §7 已预留）。
3. 应用 Fix-1，重生成确定性区域 dump（同一 seed 8576294172403134396、同一载体），重跑树族残差表。**收敛判据：边界带 jungle 树出现 + jungle_leaves −22,611 显著收敛（预计收敛量 = 边界 chunk jungle 树份额，可由 FEATURELOG 统计缺席 feature 数 × 平均树冠块数估算）；oak +38,250 若反而增大 → 证实 oak 残差主体在 .b2b（Fix-1 只补负分量）**。
4. 应用 Fix-2，重生成。**收敛判据：oak 双向残差同时收敛（过放块消失）；若 oak + 残差基本不动 → oak 主因不在 .b2b，转分支 A/.b4 复核**。
5. 应用 Fix-3，重生成。**判据：密林区（top-delta chunk 表中地表/stone 双高 chunk）残差收敛；不收敛则 .b5 降权**。
6. 排除判据：E-B1.2 中 p 集合完全一致（无缺失）→ .b2a 排除；Fix-2 后 oak 双向均无变化 → .b2b 排除。
7. 全程同一确定性 dump 载体（workflow-patterns #52），禁跨 run 对比（#51）。

**执行序建议**：Fix-1 → E-B1.2/3 → Fix-2 → E-B1.4 → Fix-3 → E-B1.5（单变量逐次，保回滚点）。

## 5. .b6 一句话复核

排除理由成立：`chunkrandom.rs:148-161` 的 set_population_seed/set_decorator_seed 与 Java ChunkRandom.java:54-78 逐式一致（`|1`、`^seed`、`+index+10000*step` 全同），且 Java :402 每 feature 独立 reseed → p 集差异只改「执行哪些 feature」（归 .b2a），不产生 RNG 流污染。

## 6. 边界与置信度

- 全文 Degraded（静态）；未运行任何命令/探针；status: draft，candidate 需 E-B1 数据。
- populationSeed 域未触碰：Fix-1/2/3 全部在 biome 采样、集合运算、heightmap 层；种子派生代码零改动。
- Java 参照：yarn 1.20.1+build.10-v2 sources（路径 scout §0，已复核存在）。
