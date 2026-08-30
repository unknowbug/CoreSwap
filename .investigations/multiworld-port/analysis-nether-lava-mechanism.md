# analysis-nether-lava-mechanism — 1.20.1 下界熔岩填充精确机制（recode-scout 勘探）

- 状态：draft（静态源码勘探，Degraded 分层——未做运行时探针）
- 日期：2026-02（本 session）
- 源码根：`versions/1.20.1/data/mc_src_extract/`（yarn 反编译源）
- 参照数据：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/nether.json`

## 机制结论（一段话）

下界熔岩**不是单独的流体填充步骤，也不是 surface 规则产物**：它产生于 NOISE 阶段 `NoiseChunkGenerator.populateNoise`（即 fillFromNoise 路径）逐块的 `chunkNoiseSampler.sampleBlockState()` 调用。由于 `nether.json` 的 `aquifers_enabled=false`，`ChunkNoiseSampler` 构造器选择的是**极简 sea-level aquifer**（`AquiferSampler.seaLevel(fluidLevelSampler)`，一个匿名内部类，不是 `AquiferSampler.Impl`，无任何流体噪声参与）。该实现逻辑就是 docs/09 猜测的形态：**density > 0 → 返回 null（调用方填 default_block=netherrack）；density ≤ 0 → 返回 `FluidLevel.getBlockState(y)`，即 `y < sea_level(32) ? lava : air`**。docs/09 的两条猜测全部得到证实。

## 源码证据（逐条，源码确认）

### 1. aquifers_enabled=false → 选用 seaLevel 匿名实现，而非 Impl

`ChunkNoiseSampler.java` L158-174（构造器）：
```java
NoiseRouter noiseRouter = noiseConfig.getNoiseRouter();
NoiseRouter noiseRouter2 = noiseRouter.apply(this::getActualDensityFunction);
if (!chunkGeneratorSettings.hasAquifers()) {
    this.aquiferSampler = AquiferSampler.seaLevel(fluidLevelSampler);
} else {
    ... AquiferSampler.aquifer(...)  // Impl
}
```
【源码确认】

### 2. seaLevel 实现：density>0 → null；否则查 FluidLevel

`AquiferSampler.java` L32-45：
```java
static AquiferSampler seaLevel(AquiferSampler.FluidLevelSampler fluidLevelSampler) {
    return new AquiferSampler() {
        public BlockState apply(DensityFunction.NoisePos pos, double density) {
            return density > 0.0 ? null
                : fluidLevelSampler.getFluidLevel(pos.blockX(), pos.blockY(), pos.blockZ()).getBlockState(pos.blockY());
        }
        public boolean needsFluidTick() { return false; }
    };
}
```
- **无 floodedness/spread/lava/barrier 噪声参与**（下界 fluid 相关 router 值均为常量 0，见第 4 条——但在此路径根本不被采样）。
- `needsFluidTick()` 恒 false → 下界熔岩不进 `markBlockForPostProcessing`。【源码确认】

### 3. FluidLevel.getBlockState：严格小于 sea_level，y=0..31 为 lava

`AquiferSampler.java` L52-64：
```java
public static final class FluidLevel {
    final int y;
    final BlockState state;
    public BlockState getBlockState(int y) {
        return y < this.y ? this.state : Blocks.AIR.getDefaultState();
    }
}
```
比较运算符为**严格 `<`**：`y=31 → lava`，`y=32 → air`。**没有下界限制**（y=0 也返回 lava；不会自动停在 bedrock 上方——下界的 bedrock 由 bedrock floor/ceiling 阶段（`NetherBedrock` / bedrock 噪声，属后续 Stage）覆盖，非本阶段职责）。【源码确认】

### 4. fluidLevelSampler：下界实际返回恒定 FluidLevel(32, lava)

`NoiseChunkGenerator.java` L78-84：
```java
private static AquiferSampler.FluidLevelSampler createFluidLevelSampler(ChunkGeneratorSettings settings) {
    AquiferSampler.FluidLevel fluidLevel  = new AquiferSampler.FluidLevel(-54, Blocks.LAVA.getDefaultState());
    int i = settings.seaLevel();
    AquiferSampler.FluidLevel fluidLevel2 = new AquiferSampler.FluidLevel(i, settings.defaultFluid());
    AquiferSampler.FluidLevel fluidLevel3 = new AquiferSampler.FluidLevel(DimensionType.MIN_HEIGHT * 2, Blocks.AIR.getDefaultState());
    return (x, y, z) -> y < Math.min(-54, i) ? fluidLevel : fluidLevel2;
}
```
- 下界 `sea_level=32`（nether.json 已核实 `"sea_level": 32`，`"aquifers_enabled": false`，default_block=netherrack，default_fluid=lava）。
- `Math.min(-54, 32) = -54` → 分支条件 `y < -54` 在下界（min_y=0）**永不成立** → 采样器恒返回 `FluidLevel(32, lava)`。
- `-54` 的 lava 层与 `MIN_HEIGHT*2` 的 air 层是为主世界准备的，下界用不到。【源码确认】

### 5. 放置点：populateNoise 内联，null → default_block

`NoiseChunkGenerator.java` L404-423（`private Chunk populateNoise(...)` 内的三重循环）：
```java
BlockState blockState = chunkNoiseSampler.sampleBlockState();
if (blockState == null) {
    blockState = this.settings.value().defaultBlock();   // netherrack
}
blockState = this.getBlockState(chunkNoiseSampler, x, t, aa, blockState); // 恒等：直接 return state
if (blockState != AIR && ...) {
    chunkSection.setBlockState(y, u, ab, blockState, false);
    heightmap.trackUpdate(...);
    if (aquiferSampler.needsFluidTick() && !blockState.getFluidState().isEmpty()) {
        chunk.markBlockForPostProcessing(mutable);   // seaLevel 实现恒 false → 不触发
    }
}
```
`sampleBlockState()`（`ChunkNoiseSampler.java`）→ `blockStateSampler.sample(this)`，其链首即 L181：
```java
builder.add(pos -> this.aquiferSampler.apply(pos, densityFunction.sample(pos)));
```
其中 densityFunction = `cacheAllInCell(finalDensity + Beardifier)`。即**熔岩判断用的 density 就是最终密度（final_density，含 Beardifier 叠加）**。注意 `blockState != AIR` 判断：density≤0 且 y≥32 时返回 AIR → **该格不写入**（保持 chunk 初始为空），不是写入 air。【源码确认】

### 6. 下界 fluid 噪声 router 值：全 0（但 seaLevel 路径根本不采样）

nether.json `noise_router`：`fluid_level_floodedness=0`、`fluid_level_spread=0`、`lava=0`、`barrier=0`（常量 0）。这些只被 `AquiferSampler.Impl` 使用（`getFluidBlockY`/`getNoiseBasedFluidLevel`/`calculateDensity`/`getFluidBlockState`），下界走 seaLevel 路径，完全无关。【源码确认】

### 7. buildSurface 跳过流体格：证实

`SurfaceBuilder.java` L131-162（`buildSurface` 逐列扫描）：
```java
for (int u = p; u >= t; u--) {
    BlockState blockState = blockColumn.getState(u);
    if (blockState.isAir()) { q = 0; r = Integer.MIN_VALUE; }
    else if (!blockState.getFluidState().isEmpty()) {
        if (r == Integer.MIN_VALUE) { r = u + 1; }   // 记录液面+1，不套用 surface 规则
    } else {
        ...
        if (blockState == this.defaultState) {        // 仅当 = netherrack 才尝试规则
            BlockState blockState2 = blockStateRule.tryApply(m, u, n);
            ...
        }
    }
}
```
- 流体格走第二分支：只更新 `r`（surface depth 的流体顶参考），**不应用 surface 规则**——熔岩放置后不会被 surface 覆盖。
- 且 surface 规则只在 `blockState == defaultState`（非流体 netherrack）时尝试。【源码确认】docs/09 猜测证实。

## Rust 移植要点（伪代码级）

在 `fill_chunk` 的逐块循环中（密度已算出 final_density 的位置）：

```rust
// nether（泛化为 aquifers_enabled=false 的任意维度）:
const SEA_LEVEL: i32 = 32;              // 来自 nether.json sea_level
const DEFAULT_FLUID_IS_LAVA = true;     // 来自 nether.json default_fluid

for y in 0..height_world {              // 世界绝对 y，下界 0..128
    let density = final_density(x, y, z);           // 含 beardifier 叠加（若已移植）
    if density > 0.0 {
        set(netherrack);                            // default_block
    } else if y < SEA_LEVEL {                       // 严格 <；y=31 lava, y=32 air
        if y >= world_min_y { set(Lava); }          // y=0 也是 lava，无下界 clamp
        // 注意：不 mark post-processing（needsFluidTick 恒 false）
    } else {
        // 返回 AIR → Java 不写入该格。若 Rust 块数组预填了某种值，
        // 必须与 Java「保持空」语义一致（大概率应填 Air）
        set(Air);
    }
}
```

要点清单：
1. **条件三分**，不是二分：`d>0 → rock`、`d≤0 && y<32 → lava`、`d≤0 && y≥32 → air`。Rust 现状「lava 海带 y=32..63 全错」与 `y < 32` 边界一致（猜 Rust 侧边界写成了 `<=` 或范围错位，先核对）。
2. **无需 aquifer 复杂路径**：下界禁用 aquifers 时不存在 Impl 的 16×12×16 邻居随机偏移、floodedness/spread/lava 噪声、barrier 计算——全部是主世界专属。
3. **y 域是绝对坐标**：`y < SEA_LEVEL` 比较用世界绝对 y（下界 0..127），非 chunk 内相对 y。
4. **AIR 不写入**（Java `blockState != AIR` 守卫），heightmap 也不 track；若 Rust 高度图对齐 Java，需同样跳过。
5. **surface 阶段跳过流体**：Rust surface 规则应用处若已跳过非 default_block / 流体格则无需改；否则熔岩会被覆盖出错。
6. 泛化公式（任何 aquifers=false 维度）：`sampler = |y| -> FluidLevel(sea_level, default_fluid)`；`d>0→default_block`，`d≤0→ (y < sea_level ? default_fluid : air)`。end 结束「lava 特判不需要，default_fluid 即 lava」。

## 未解/不确定项

1. **Beardifier 叠加是否参与下界 density**【源码确认存在、运行时未验证】：L178 `finalDensity + Beardifier.INSTANCE` 对所有维度一致；下界无结构起始点时 Beardifier 恒 0，理论无影响，但若 Rust 下界 fill 尚未叠加 beardifier，需确认下界是否真有非零 beardifier 贡献（推断：无，降级声明）。
2. **Rust「lava 海带 y=32..63」的具体错位根因**（边界 `<=` vs `<`、相对 y vs 绝对 y、air 分支缺失）需在 Rust 侧 fill_chunk 代码核对——本勘探只定 Java 真值。
3. 下界 bedrock（0 层与 127 层）由 bedrock 阶段处理，不在本机制范围（未展开勘探）。

## docs/09 勘误小结

- 「lava 来自 fillFromNoise」✅ 证实（精确：populateNoise → sampleBlockState → seaLevel aquifer）。
- 「buildSurface 跳过流体格」✅ 证实（SurfaceBuilder L136 分支）。
- 补充：比较为严格 `<`；air 分支不写入；fluid 噪声 router 全 0 但根本不被采样。
