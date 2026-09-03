# Java est（estimateSurfaceHeight）调用链地图 + Rust 对拍点清单

- 角色：recode-scout（只读勘探，不裁决）
- 状态：draft（勘探地图，非结论）
- 日期标签：260903（scout 产物，时间取任务批次，非自推）
- Java 源：`versions/1.20.1/data/mc_src_extract/`（yarn sources 解包）
- Rust 源：`WorldgenRust/src/`

## 1. Java 权威定义（ChunkNoiseSampler.java:222-240 精确摘录）

```java
// net/minecraft/world/gen/chunk/ChunkNoiseSampler.java
private final Long2IntMap surfaceHeightEstimateCache = new Long2IntOpenHashMap();   // L47

public int estimateSurfaceHeight(int blockX, int blockZ) {                          // L222-226
    int i = BiomeCoords.toBlock(BiomeCoords.fromBlock(blockX));
    int j = BiomeCoords.toBlock(BiomeCoords.fromBlock(blockZ));
    return this.surfaceHeightEstimateCache.computeIfAbsent(ColumnPos.pack(i, j), this::calculateSurfaceHeightEstimate);
}

private int calculateSurfaceHeightEstimate(long columnPos) {                        // L228-240
    int i = ColumnPos.getX(columnPos);
    int j = ColumnPos.getZ(columnPos);
    int k = this.generationShapeConfig.minimumY();
    for (int l = k + this.generationShapeConfig.height(); l >= k; l -= this.verticalCellBlockCount) {
        if (this.initialDensityWithoutJaggedness.sample(new DensityFunction.UnblendedNoisePos(i, l, j)) > 0.390625) {
            return l;
        }
    }
    return Integer.MAX_VALUE;
}
```

要点（纯事实摘录，非裁决）：
- **输入域**：blockX/blockZ 先做 biome 4×4 量化（`BiomeCoords.fromBlock = x >> 2`，`toBlock = << 2`）→ `(x>>2)<<2`。
- **缓存**：`surfaceHeightEstimateCache` 是 **sampler 实例级 Long2IntOpenHashMap，无界**，key = 量化后 ColumnPos.pack。ChunkNoiseSampler 每 chunk 新建（NoiseConfig/buildSurface 路径）→ 缓存生命周期 = 单 chunk，但 key 量化到 biome 列，跨采样点共享。
- **扫描域**：`min_y + height` → `min_y`，步长 `-verticalCellBlockCount`，采样 `initialDensityWithoutJaggedness`（注意：UnblendedNoisePos 用**量化后**的 i/j）。

## 2. Java 生产调用链（谁在什么阶段调用 est）

```
ChunkStatus.tasks (ChunkStatus.java:117)  — SURFACE 阶段
  → ChunkGenerator.buildSurface (ChunkGenerator.java:438, abstract)
    → NoiseChunkGenerator.buildSurface (NoiseChunkGenerator.java:242/252-266)
      → SurfaceBuilder.buildSurface (SurfaceBuilder.java:72)
        → materialRuleContext（SurfaceBuilder.java:166 placeIceberg 直调 estimateSurfaceHeight()）
        → MaterialRules.MaterialRuleContext:

  调用点 A — MaterialRules.java:488-516 estimateSurfaceHeight()（4 角插值，above_preliminary_surface 条件）:
    protected int estimateSurfaceHeight() {
        if (this.field_35679 != this.uniqueHorizontalPosValue) {
            int i = blockToChunkCoord(this.blockX);   // >> 4
            int j = blockToChunkCoord(this.blockZ);
            if (this.packedChunkPos != ChunkPos.toLong(i, j)) {   // 每 chunk 一次刷新 4 角
                this.estimatedSurfaceHeights[0] = sampler.estimateSurfaceHeight(chunkToBlockCoord(i),     chunkToBlockCoord(j));
                this.estimatedSurfaceHeights[1] = sampler.estimateSurfaceHeight(chunkToBlockCoord(i+1),   chunkToBlockCoord(j));
                this.estimatedSurfaceHeights[2] = sampler.estimateSurfaceHeight(chunkToBlockCoord(i),     chunkToBlockCoord(j+1));
                this.estimatedSurfaceHeights[3] = sampler.estimateSurfaceHeight(chunkToBlockCoord(i+1),   chunkToBlockCoord(j+1));
            }
            int k = MathHelper.floor(MathHelper.lerp2((blockX&15)/16.0F, (blockZ&15)/16.0F, e[0..3]));
            this.surfaceMinY = k + this.runDepth - 8;
        }
        return this.surfaceMinY;
    }
    消费点 A2 — MaterialRules.java:567-572 SurfacePredicate:
        get() => blockY >= estimateSurfaceHeight()      // abovePreliminarySurface

  调用点 B — AquiferSampler.java:363（getFluidLevel，NOISE/carver 期 aquifer 判定，非 SURFACE）:
        int n = this.chunkNoiseSampler.estimateSurfaceHeight(l, m);   // 9 邻域 CHUNK_POS_OFFSETS 循环内
```

**关键事实：Java 全路径只有一张 est 表**——ChunkNoiseSampler 实例上的 `surfaceHeightEstimateCache`。SURFACE（调用点 A）与 aquifer（调用点 B）调用的是**同一个 sampler 的同一张缓存 map**；SURFACE 阶段每 chunk 预取 4 角（chunk 坐标 `<<4` 即 chunk 原点角，非 +15），后续同 chunk 任意列经 biome 量化后命中同一张 map。

## 3. Rust 侧对应代码

### 3a. 生产门控点 — `WorldgenRust/src/worldgen_handle.rs:517-550`

```rust
// b1-a（260903-11，默认关）：WG_EST_SHARED → est_at 复用 va.aq 的 surface_cache
//（对齐 Java：SURFACE 阶段走 sampler.estimateSurfaceHeight 同一张 map，ChunkNoiseSampler.java:222-226）。
//  D1 角列量化：旧路径 +15 直采；共享路径 (x>>2)<<2 量化（Java 语义）
//  D3 扫描域：旧路径 min_y+noise_height；共享路径 min_y+height（overworld 384 同值；nether 128<256…）
let est_shared = std::env::var("WG_EST_SHARED").is_ok();
let mut est_at = |x: i32, z: i32| -> i32 {
    if est_shared {
        va.aq.estimate_surface_height(x, z)                       // 共享臂
    } else {
        let mut est = i32::MAX;
        for y in (min_y..min_y + self.noise_height).rev().step_by(8) {
            if self.init.sample(&NoisePos { x, y, z }) > 0.390625 { est = y; break; }   // off 臂
        }
        est
    }
};
let surface_heights4 = vec![
    est_at(cx * 16, cz * 16), est_at(cx * 16 + 15, cz * 16),
    est_at(cx * 16, cz * 16 + 15), est_at(cx * 16 + 15, cz * 16 + 15),
];
```

### 3b. 共享臂实现 — `WorldgenRust/src/aquifer.rs:343-377`

```rust
pub fn estimate_surface_height(&mut self, block_x: i32, block_z: i32) -> i32 {
    let bx = (block_x >> 2) << 2; let bz = (block_z >> 2) << 2;   // biome 量化（D1 量化点）
    let ix = (bx >> 2) - self.cache_cx * 4 + CACHE_OFF_X;          // per-chunk 16×(DIM) 邻域缓存
    ... if in_c { hit return; }  ← L2（WG_EST_L2）miss 回填 ...
    let mut val = i32::MAX;
    let mut l = self.min_y + self.height;                          // 扫描顶 = min_y+height（D3 扫描域）
    while l >= self.min_y {
        if self.initial_density.sample(&NoisePos { x: bx, y: l, z: bz }) > 0.390625 { val = l; break; }
        l -= 8;
    }
    if in_c { self.surface_cache[ci] = val; }                      // chunk 级缓存 + 可选跨 chunk L2
    val
}
```

- aquifer 自身调用点（对应 Java 调用点 B）：`aquifer.rs:426` `let n = self.estimate_surface_height(l, mm);`（get_fluid_level 9 邻域循环）。
- off 臂另有第三条独立实现：`surface_rules.rs:367-391`（SurfaceContext 单槽 thread_local 缓存、非 heights4 时 fallback 路径，world_min_y+world_height 扫描）——生产 SURFACE 走 heights4 时被绕过。

## 4. 对拍点清单（函数 / 输入域 / 缓存范围）

| # | 维度 | Java | Rust off 臂（默认） | Rust shared 臂 | 差异候选 |
|---|------|------|--------------------|----------------|----------|
| 1 | 定义函数 | `ChunkNoiseSampler.estimateSurfaceHeight` | 闭包 `est_at`（worldgen_handle.rs:525-535） | `Aquifer::estimate_surface_height`（aquifer.rs:343） | — |
| 2 | 列量化 D1 | `(x>>2)<<2` 在 est 入口（sampler 内） | **无量化**，调用传 `cx*16+15` 直采（角 +15） | 入口量化 `(x>>2)<<2`（aquifer.rs:344） | **D1：off 臂角列 x/z = cx*16+15 未量化，Java 是 (cx*16+15>>2)<<2 = cx*16+12** → off/shared 值可不同（shared 与 Java 一致形态） |
| 3 | 扫描域 D3 | `min_y+height` → `min_y`（generationShapeConfig，overworld 320+64=384） | `min_y+noise_height` → min_y | `min_y+height`（aquifer.rs:361） | **D3：off 臂用 noise_height；overworld noise_height==height==384 时同值，异维度即分叉** |
| 4 | 步长 | `verticalCellBlockCount`（=4×size_vertical） | 硬编码 8 | 硬编码 8 | overworld size_vertical=2 → 8，一致；硬编码非数据驱动（见 §5） |
| 5 | 采样函数 | `initialDensityWithoutJaggedness.sample(UnblendedNoisePos(i,l,j))`，i/j=量化列 | `self.init.sample(NoisePos{x, y, z})`（未量化 x/z） | `self.initial_density.sample`（量化 bx/bz） | off 臂 x/z 未量化（伴生 D1）；采样函数是否同一实现待对拍（scout 未验证数值） |
| 6 | 缓存范围 | sampler 实例级 **无界 Long2IntOpenHashMap**，生命周期=单 chunk（sampler 每 chunk 新建），key=量化列 pack | **无列缓存**（每角现算，且只算 4 角一次/chunk） | per-chunk `surface_cache: Vec<i32>` 有限邻域（CACHE_DIM）+ 可选跨 chunk L2（WG_EST_L2，blend 闸门） | 缓存粒度不同（无界 map vs 有限邻域），命中语义等价性（同列同值纯函数）不改变值本身；L2 开启时跨 chunk 复用 = Java 无此语义（Java sampler 每 chunk 新建 → L2 与 Java 生命周期不等价，仅值等价） |
| 7 | SURFACE 4 角取点 | chunk 坐标 `<<4`（= cx*16，chunk **原点**角） | `cx*16+15`（角 +15） | 同 off 臂调用参数，但入口量化后 = cx*16+12 | **角参数 +15 vs 原点**：Java 预取的是 (i,j)/(i+1,j+1) chunk 角，量化后 00 角=chunk 原点，11 角=+12（60,60 量化→60）。Rust +15 量化后=+12 —— shared 臂量化后恰好与 Java (i+1) 角一致；off 臂 +15 直采不一致 |
| 8 | 4 角插值消费 | `MathHelper.lerp2((x&15)/16f,(z&15)/16f,...)` → floor，`surfaceMinY = k + runDepth - 8` | surface_rules.rs:209-224 `lerp2` + floor + `k + surface_depth - 8` | 同左（消费端不分臂） | 消费端一致 |
| 9 | aquifer 共用 | SURFACE 与 aquifer 共用同一张 map | off 臂 SURFACE 独立现算，aquifer 走 surface_cache → **两表分离** | SURFACE 复用 aquifer 表 → 同一张表 | off 臂本质差异：SURFACE est 与 aquifer est 同列值可能不同（量化/扫描域/采样 x,z 不同） |

## 5. est 步长常量取值源头（#25 教训）

- Java：`ChunkNoiseSampler.verticalCellBlockCount`（L129）= `generationShapeConfig.verticalCellBlockCount()`；`GenerationShapeConfig.java:17` 显示 `size_vertical` 是 JSON 字段（`Codec.intRange(1, 4)`）。
- JSON 数据源：`worldgen/noise_settings/overworld.json` 的 `"size_vertical": 2` → 步长 = 4 × 2 = **8**。**常量 8 = `4 * size_vertical`，取值源头是 noise_settings JSON，不是协议常量**；nether/end 等 settings 的 size_vertical 不同（nether size_vertical=1 → 步长 4，扫描域亦异）。
- Rust：off/shared 两臂均硬编码 `8`/`-= 8`（worldgen_handle.rs:530、aquifer.rs:367、surface_rules.rs:384）——overworld 生产路径数值正确，但未从 settings 读（跨维度升级点，数据驱动边界标注候选）。

## 6. scout 观察（不下裁决）

- shared 臂在「列量化（D1）、扫描域（D3）、与 aquifer 共表」三处均呈 Java 同构形态；off 臂三处均偏离（+15 直采、noise_height 扫描、独立表）。**但 off 臂仅对 4 角现算各一次/chunk**——两臂数值差异只会在「量化/扫描域造成角列值不同」时显形，即 hash 变化的机制候选集中在 D1/D3（与代码注释一致），本 scout 不判权重。
- Java 的 `estimatedSurfaceHeights` 4 角按 chunk 刷新一次（`packedChunkPos` 比对），Rust heights4 每 chunk 重建一次——刷新粒度一致。
- 未验证项（诚实声明）：① Rust `initial_density` 与 Java `initialDensityWithoutJaggedness` 逐位一致性（另有结论链，未复查）；② nether/other-dimension 路径两臂行为（本勘探只覆盖 overworld 生产路径）。
