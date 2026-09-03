# Java 1.20.1 NoiseChunk "est" 列缓存确切语义（core-worker，只读调研）

- **status: draft**
- 置信度：candidate（纯静态阅读 Java 参考源 + Rust 生产源，Degraded/静态审查分层，未运行验证——本任务为语义核对，无数值对拍需求）
- 日期标签：260903（随父 session Q-AQ1 课题）
- 映射说明：工作区 Java 参考源为 **Mojang 官方映射名**——yarn 的 `NoiseChunk` = `net.minecraft.world.gen.chunk.ChunkNoiseSampler`；yarn 的 `NoiseChunkAquifer` = `net.minecraft.world.gen.chunk.AquiferSampler`（含 `AquiferSampler.Impl`）。

---

## 1. "est" 缓存的数据结构 / 粒度 / 生命周期

### 1.1 数据结构

`ChunkNoiseSampler` 的实例字段（`E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\gen\chunk\ChunkNoiseSampler.java:47`）：

```java
private final Long2IntMap surfaceHeightEstimateCache = new Long2IntOpenHashMap();
```

哈希 map（fastutil Long2IntOpenHashMap），key = `ColumnPos.pack(i, j)`（**4 格生物组量化后的列坐标打包 long**，见 §2.1），value = 估算表面高度 int。

### 1.2 入口与粒度

`estimateSurfaceHeight`（ChunkNoiseSampler.java:222-226）：

```java
public int estimateSurfaceHeight(int blockX, int blockZ) {
    int i = BiomeCoords.toBlock(BiomeCoords.fromBlock(blockX));   // 4 格量化 (x>>2)<<2
    int j = BiomeCoords.toBlock(BiomeCoords.fromBlock(blockZ));
    return this.surfaceHeightEstimateCache.computeIfAbsent(ColumnPos.pack(i, j), this::calculateSurfaceHeightEstimate);
}
```

- **粒度 = 每列（column），且列坐标按 4×4 生物格量化**——同一 4×4 方块格内任意方块坐标命中同一条缓存。这与 Rust 侧 `surface_cache` 的 `(bx>>2, bz>>2)` 索引粒度一致（`WorldgenRust/src/aquifer.rs:284-285`）。
- 调用方：`AquiferSampler.Impl.getFluidLevel` 对 13 个 offset 邻居列逐列调用（AquiferSampler.java:360-385，`this.chunkNoiseSampler.estimateSurfaceHeight(l, m)`，行 363）。

### 1.3 生命周期（Q4 的核心答案）

- `ChunkNoiseSampler` 由 `NoiseChunkGenerator.createChunkNoiseSampler` 创建（NoiseChunkGenerator.java:102-111），经 `chunk.getOrCreateChunkNoiseSampler(...)` **懒创建并挂在 Chunk 对象上**（NoiseChunkGenerator.java:95-97 / 261-262 / 294-297 / 360-361）。
- 该 sampler 在同一 chunk 的 NOISE → CARVERS（NoiseChunkGenerator.java:294-320）→ SURFACE/FEATURE（360-361）各阶段间复用——即 **chunk 生成管线全程存活，随 ProtoChunk 生命周期一起消亡**。
- **不存在任何跨 chunk 复用**：map 是 sampler 的实例字段，sampler 是 per-chunk 的；相邻 chunk 各自新建 sampler → 各自冷缓存。Java 世界里没有「est 缓存池 / 全局 surface 高度表」这类对应物。
- **结论（Q4）：Java 的 est 列缓存也是 per-chunk 重建、chunk 内列间复用；「跨 chunk 持久化」在 Java 里没有对应物。** Q-AQ1 中「Java 侧 est 列缓存是 chunk 级持久」应精确表述为「**chunk 生成管线内持久**（跨阶段），不是跨 chunk」。Rust 每 chunk 丢缓存在这一点上与 Java **语义一致，并无结构性劣势**；Rust 的劣势是**每 chunk 冷启动时首列邻域的全价重算量与 Java 相同、但 Rust 若想优化只能引入 Java 没有的跨 chunk 结构**（详见 §4）。

### 1.4 同一 Impl 内的另一层缓存（顺带核对）

`AquiferSampler.Impl` 自带两个 per-chunk 数组缓存（AquiferSampler.java:88-89、131-133）：
- `waterLevels: FluidLevel[]`——aquifer cell（16×12×16）粒度的水位缓存，`getWaterLevel` 惰性填充（335-351）；
- `blockPositions: long[]`——同粒度的随机锚点缓存，填充 `Long.MAX_VALUE` 哨兵（133、177-183）。

Rust `Aquifer` 的 `water_levels` / `block_positions`（aquifer.rs:93-94、121-122）与之一一对应，粒度/哨兵值均一致。

---

## 2. 缓存写入 / 命中 / 重算条件（Q2）

### 2.1 写入

- **惰性写入**：首次对某量化列调用 `estimateSurfaceHeight` 时 `computeIfAbsent` 触发 `calculateSurfaceHeightEstimate`（ChunkNoiseSampler.java:228-240）：从 `minY + height` 顶到底按 **4 格垂直步长（verticalCellBlockCount）** 下降，对 `initialDensityWithoutJaggedness.sample(UnblendedNoisePos(i, l, j))` 全价采样，首个 `> 0.390625`（= 25/64）的 `l` 即返回；全空则返回 `Integer.MAX_VALUE`（**该哨兵同样被写进缓存**，computeIfAbsent 不区分）。
- 写入后**本 sampler 生命周期内永不失效、永不重算**——没有失效钩子、没有容量淘汰（OpenHashMap 无界，但 9×9 生物列 ≈ 81 条上限，实际有界）。

### 2.2 命中

- 同一量化列的后续任何调用（无论来自哪个 aquifer 邻居 offset、哪个阶段）直接 O(1) 命中 map。
- 典型命中场景：13 offset 邻域中重叠列（如 {0,0} 与相邻 pocket 查询共享）、同 chunk 内多次 `getFluidLevel`（每个 block-density 采样点都会走到）。

### 2.3 aquifer surface 计算路径中的使用点

`getFluidLevel`（AquiferSampler.java:353-389）：
1. 13 个 `CHUNK_POS_OFFSETS` 邻居列（AquiferSampler.java:99-101）逐列取 `estimateSurfaceHeight`（363）；
2. `o = n + 8` 与当前 y 比较做 flooded 判定（364-382）；
3. `i = Math.min(i, n)` 聚合最小表面（384）→ 传入 `getFluidBlockY`（387，行 391-419）→ 可能进入 `getNoiseBasedFluidLevel`（421-433，用 surfaceHeightEstimate 做 `Math.min(surfaceHeightEstimate, q)` 上限，行 432）。
- 即**一次 `getFluidLevel`（一个 aquifer cell 的水位解析）最多触发 13 列 est 计算**；冷缓存时每列 ≈ 96 次初始密度采样（384 高 / 4 步），这就是 Q-AQ1 的 13×~34~96 采样成本来源（Java 冷 chunk 同样付这笔，见 §4）。

---

## 3. Rust 侧对应实现与 Java 语义差距点清单

Rust 生产实现：`E:\PYTHON\CoreSwap\WorldgenRust\src\aquifer.rs`（`Aquifer`，每 chunk 在 `worldgen_handle.rs:446`（fill_chunk_blocks 主路径）与 `worldgen_handle.rs:547`（carver 路径）各 `Aquifer::new` 一次）。

| # | 语义点 | Java（ChunkNoiseSampler/AquiferSampler.Impl） | Rust（aquifer.rs） | 差距 |
|---|---|---|---|---|
| G1 | est 缓存载体 | sampler 实例 `Long2IntOpenHashMap`（ChunkNoiseSampler.java:47），独立于 Aquifer | `Aquifer.surface_cache: Vec<i32>`（:95、:123），内嵌在 Aquifer | 结构差异无行为影响；但 Rust 把 est 缓存绑死在 Aquifer 上，Aquifer 每 chunk 重建 → 缓存每 chunk 丢（与 Java 同） |
| G2 | 列量化 | `BiomeCoords.fromBlock` = `>>2`（:223-224） | `bx>>2`（:284-285） | **一致** ✅ |
| G3 | 扫描步长 | 4 格垂直步长、阈值 0.390625、从顶到底（:233-237） | 4 格步长、0.390625（:292-295） | **一致** ✅（垂直步长核对：Rust 以 4 步进，与 `verticalCellBlockCount=4` 一致） |
| G4 | MAX_VALUE 哨兵缓存 | 空列 `Integer.MAX_VALUE` 也入缓存（:239，computeIfAbsent） | Rust 哨兵 `i32::MIN` 表示未算；**val 会被缓存**（:297），需确认空列返回值也被缓存且与哨兵不冲突——静态读 :287-297 未见冲突，但空列返回值语义需对拍 | ⚠️ 低风险对拍点 |
| G5 | 生命周期 | sampler 挂 Chunk，管线全阶段复用（NOISE/CARVERS/SURFACE 同一实例，NoiseChunkGenerator.java:294/360） | `fill_chunk_blocks`（:446）与 carver 路径（:547）**各自 new 一个 Aquifer** → 两阶段间 water_levels/block_positions/surface_cache **不共享**；Java 是共享的 | **真实差距点**：同 chunk 双份冷缓存 = 额外重复采样（Java 无此重复）。这是「cache 复用」维度上 Rust 比 Java 差的地方，但发生在**chunk 内阶段间**，不是跨 chunk |
| G6 | 跨 chunk 持久化 | **无此物**（§1.3） | 也无 | **一致**——「跨 chunk est 缓存」是 Java 不存在的优化自由度（见 §5） |

---

## 4. 方案 a 可行性判定：「est 列缓存跨 chunk 持久化」

**判定：语义上 Java 没有对应物，方案 a（跨 chunk 持久 est 缓存）不是「复刻 Java」而是「超越 Java 的新优化」。** 它在语义上是安全的（est 是纯函数：initialDensityWithoutJaggedness 只依赖 (x,z) 与世界种子/噪声参数，无 chunk 局部状态——注意 Blender 边界 chunk 除外，`cachedBlendAlphaDensityFunction` 等 blend 输入是 per-chunk 的，ChunkNoiseSampler.java:52-53、142-154，跨 chunk 缓存须对 blend chunk 排除或以 blend 状态为 key），可作为纯性能优化引入而不改变输出。但必须明确：它不是对齐项，不能作为「补 Java 已有的缓存」来叙述。

**同时修正 Q-AQ1 表述**：Java est 缓存 = chunk 管线内持久（per-chunk 实例、跨阶段复用），非跨 chunk；Rust 与 Java 的真实差距是 **G5（NOISE/carver 两阶段各建一份 Aquifer，缓存不共享）**，而非「Java 跨 chunk、Rust 每 chunk 丢」。

**对拍点清单**（若推进优化/验证）：
1. **G4 空列哨兵**：Rust 空列（全列 ≤0.390625 → Java 返回 Integer.MAX_VALUE）路径的缓存写入/读取与 Java 行为逐位对拍（选一个海洋深处列验证）。
2. **G5 阶段共享**：确认 carver 路径（worldgen_handle.rs:547）与 fill 路径对同一 chunk 的 aquifer 输出逐位一致后，评估共享一个 Aquifer（或至少共享 surface_cache）的成本——这是 Java 有、Rust 无的 chunk 内复用。
3. **方案 a 正确性闸门**：跨 chunk 缓存 key 必须含 (worldSeed, noiseRouter 参数, blend 状态)；blend chunk（有 neighbors 的 ProtoChunk，Blender 非 `Blender.getNoBlending()`）必须旁路，否则 surface 估算被 blend density 污染（ChunkNoiseSampler.java:142-154 blend 缓存为 per-chunk 预填）。
4. **性能口径**：优化前后按端到端大样本基准（AGENTS.md §四 端到端铁律），并遵守验证可比性声明（§9.7）——探针口径与整批 wall 口径分开声明。

---

## 来源索引

| 结论 | 来源 |
|---|---|
| est 缓存结构/粒度 | mc_src_extract ChunkNoiseSampler.java:47, 222-226 |
| est 计算（4 格步长、0.390625、MAX_VALUE） | ChunkNoiseSampler.java:228-240 |
| 13 offset 邻域调用 est | AquiferSampler.java:99-101, 353-389（363/364/384/387/432） |
| sampler per-chunk、管线跨阶段复用 | NoiseChunkGenerator.java:95-97, 102-111, 294-320, 360-361 |
| aquifer waterLevels/blockPositions per-chunk | AquiferSampler.java:88-89, 120-134, 335-351 |
| blend 缓存 per-chunk（方案 a 闸门依据） | ChunkNoiseSampler.java:52-53, 142-154 |
| Rust 对应实现 | WorldgenRust/src/aquifer.rs:81-127, 284-297; worldgen_handle.rs:446-453, 547 |
