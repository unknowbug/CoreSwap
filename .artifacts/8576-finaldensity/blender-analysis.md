# Blender（blend_density）分析 — 8576 seed chunk(50,-26)

> 承 `beardifier-analysis.md`（已否证 Beardifier）。本文件验证新候选：**Blender/blend_density 是否解释了 (810,76,-411) 的方块差异**。
> 矛盾现状：所有密度口径（无插值/插值/角点/cns 反射）一致给出 finalDensity ≈ -0.038（air），但真实游戏导出参照该点为 terracotta。
> 候选假设：BlockProbe 连带生成 6×6 时 Blender 被激活，blend_density 混合了邻居/旧世界密度，使参照值 ≠ 反射值。

---

## 1. Blender 激活条件（源码）

`net/minecraft/world/gen/chunk/Blender.java` L69-103：

```java
public static Blender getBlender(@Nullable ChunkRegion chunkRegion) {
    if (chunkRegion == null) {
        return NO_BLENDING;
    } else {
        ChunkPos chunkPos = chunkRegion.getCenterPos();
        if (!chunkRegion.needsBlending(chunkPos, BLENDING_CHUNK_DISTANCE_THRESHOLD)) {
            return NO_BLENDING;
        } else {
            // 收集 ±28 chunk 内所有 BlendingData（旧世界高度/密度/生物群系采样）
            ... if (blendingData == null) 跳过 ...
            return long2ObjectOpenHashMap.isEmpty() && long2ObjectOpenHashMap2.isEmpty()
                ? NO_BLENDING : new Blender(...);
        }
    }
}
```

- `BLENDING_CHUNK_DISTANCE_THRESHOLD` = `BiomeCoords.toChunk(110+3)` = **28 chunk**（L57-58）。
- `ChunkRegion.needsBlending`（`world/ChunkRegion.java` L109-110）→ `ThreadedAnvilChunkStorage.needsBlending` → `VersionedChunkStorage.needsBlending` → `StorageIoWorker.needsBlending`（`world/storage/StorageIoWorker.java` L46-73）：
  - 扫描 chunkPos ±28 范围内所有 region 文件的每个 chunk NBT，
  - `needsBlending(nbt)`（L123-127）：`DataVersion ≥ 3441` → 仅当含 `"blending_data"` 才 true；`DataVersion < 3441`（1.17 及更早）→ 一律 true。
- 即使 needsBlending=true，`BlendingData.getBlendingData(chunkRegion, l, m)` 对无旧数据的 chunk 返回 null → 表为空 → 仍 `NO_BLENDING`（L100）。

**→ 干净新世界（参照导出环境）中 Blender 恒为 `NO_BLENDING`**。Blender 是 1.17→1.18 世界过渡机制，只读「旧 chunk 磁盘数据 + blending_data」，与周边 chunk 是否已生成无关。

## 2. blend_density 的 sample 语义（源码原样）

`net/minecraft/world/gen/densityfunction/DensityFunctionTypes.java` L505-534：

```java
record BlendDensity(DensityFunction input) implements DensityFunctionTypes.Positional {
    @Override
    public double apply(DensityFunction.NoisePos pos, double density) {
        return pos.getBlender().applyBlendDensity(pos, density);
    }
}
```
`Positional`（L804-822）默认：
```java
default double sample(NoisePos pos) { return this.apply(pos, this.input().sample(pos)); }
default void fill(double[] densities, EachApplier applier) {
    this.input().fill(densities, applier);
    for (int i = 0; i < densities.length; i++) densities[i] = this.apply(applier.at(i), densities[i]);
}
```

`DensityFunction.NoisePos.getBlender()` 默认返回 `Blender.getNoBlending()`（`DensityFunction.java` L172-174）；`UnblendedNoisePos`（L177-178）用默认值；**ChunkNoiseSampler 自身实现 NoisePos 且 `getBlender()` 返回构造传入的 blender**（`ChunkNoiseSampler.java` L243-244）。

`Blender.applyBlendDensity`（`Blender.java` L155-195）：
```java
public double applyBlendDensity(NoisePos pos, double density) {
    ...
    if (d != Double.MAX_VALUE) return d;              // sampleClosest 命中旧数据
    ... closeBlendingData 加权平均（半径 2 biome 单位）...
    if (mutableDouble3 == +inf) return density;        // 无旧数据 → 恒等
    else return MathHelper.lerp(f, e, density);        // f = clamp(dx/3), e=旧世界密度
}
```
- `NO_BLENDING.applyBlendDensity` 直接返回 `density`（L45-47）。
- `NO_BLENDING.calculate` 返回 `BlendResult(1.0, 0.0)`（L40-42）。

**→ blend_density 在干净世界 = 恒等变换**。

## 3. finalDensity 树中 blend_density 的位置（确认其确实在最终路径上）

`net/minecraft/world/gen/densityfunction/DensityFunctions.java` L404-407, L453-458：
```java
private static DensityFunction applyBlendDensity(DensityFunction density) {
    DensityFunction densityFunction = DensityFunctionTypes.blendDensity(density);
    return DensityFunctionTypes.mul(DensityFunctionTypes.interpolated(densityFunction), DensityFunctionTypes.constant(0.64)).squeeze();
}
...
DensityFunction densityFunction14 = rangeChoice(slopedCheese, -1e6, 1.5625, ..., createCavesFunction(...));
DensityFunction densityFunction15 = min(applyBlendDensity(applySurfaceSlides(densityFunction14)), CAVES_NOODLE_OVERWORLD);
// finalDensity = densityFunction15
```
即 `finalDensity = squeeze( 0.64 × interpolated( blend_density( surface_slides( range_choice(...) ) ) ) )`，再与 noodle 取 min。**blend_density 在 finalDensity 树内、interpolated 内部**——任何真实采样都会经过它，其值完全由 `pos.getBlender()` 决定。

## 4. DensityProbe vs BlockProbe 的 Blender 状态判定

| | DensityProbe（反射 cns，单独生成） | BlockProbe（参照导出，连带 6×6） |
|---|---|---|
| NoisePos | UnblendedNoisePos / 探针 pos | ChunkNoiseSampler（getBlender=this.blender） |
| Blender 来源 | `ChunkStatus.NOISE` 任务：`Blender.getBlender(chunkRegion)`（`ChunkStatus.java` L88-98，传入 `populateNoise` → `createChunkNoiseSampler` → `ChunkNoiseSampler.create`） | 同一路径，仅 chunkRegion 覆盖面不同 |
| 激活判定 | needsBlending(±28) 扫磁盘旧数据 | 同左 |
| 干净新世界结果 | **NO_BLENDING** | **NO_BLENDING** |

- 两者走**完全相同的激活逻辑**，差异只在 chunkRegion 的 chunk 集合大小——而激活条件是「磁盘/内存中存在旧格式（DataVersion<3441 或含 blending_data）的 chunk」，与周边 chunk 是否已生成**无关**。
- 参照为「干净重导」世界（任务背景确认）：其中所有 chunk 都是 1.20.1 新格式、无 blending_data → **needsBlending=false → NO_BLENDING**。
- 连带生成时新生成的 chunk 即使已存在，也不会产生 blending_data（`BlendingData.getBlendingData` 对无标记 chunk 返回 null）。

**→ DensityProbe 与 BlockProbe 的 Blender 状态无差异：均为 NO_BLENDING，blend_density 均恒等。**

## 5. 定量论证

- 若 Blender = NO_BLENDING：`finalDensity(810,76,-411)` 真实生成值 = 反射值 ≈ -0.038（air），**blend_density 完全不改变它**；Beardifier 前已否证 → 游戏实际仍判 air，与参照 terracotta 矛盾依旧。
- 若要 Blender 把 -0.038 拉成正（>0），需要 `applyBlendDensity` 的 `lerp(f, e, density)` 中旧世界密度 e 显著为正且 f 足够大（f=clamp(dx/3) 需 > 0.038/(e+0.038)）。但 **e 来自旧世界 BlendingData，干净新世界不存在任何 BlendingData**（无 1.17 数据），该分支（L163-193）根本不会进入；`sampleClosest` 也返回 MAX_VALUE → 直接返回 density。
- 关于「alpha 在 chunk 内部是否为 0」：`BlendResult.alpha`（`Blender.calculate` L110-146）用于 **blend_alpha / blend_offset 密度函数**（surface rule 的地形混合与偏移），**不在 finalDensity 树内**，且 NO_BLENDING 时 calculate 恒返回 (1.0, 0.0)。密度混合走的是 `applyBlendDensity`（closeBlendingData），同样依赖旧数据。两项在干净世界均无作用。

**→ Blender 无法解释 (810,76,-411) 的 -0.038 → terracotta 翻转。**

## 6. 结论

**Blender（blend_density）根因否证（candidate 级）。**

理由链：
1. Blender 只在「1.17 旧世界升级」场景激活（needsBlending 检测 DataVersion<3441 或 blending_data，`StorageIoWorker.java` L123-127）；干净新世界恒为 `NO_BLENDING`。
2. 即使周边 chunk 已生成（BlockProbe 连带生成），新生成 chunk 不带 blending_data → Blender 表仍空 → NO_BLENDING（`Blender.java` L100）。
3. `blend_density` 节点 = `pos.getBlender().applyBlendDensity(pos, input)`，NO_BLENDING 时恒等（L45-47），真实生成 finalDensity 与反射值一致（-0.038）。
4. DensityProbe 与 BlockProbe 激活路径相同（`ChunkStatus.NOISE` → `Blender.getBlender(chunkRegion)`），状态无差异。

### 需进一步验证（才能彻底关闭）
- **运行时/数据**：确认参照导出的世界确为干净新世界（无 pre-1.18 chunk 磁盘数据、chunk NBT 无 `blending_data` 字段）。代码层面必然为 NO_BLENDING，但若 BlockProbe 使用了带旧数据的存档，需实测 `Blender.getBlender(chunkRegion) != NO_BLENDING`。
- 备查：若 Blender 被意外激活，`blend_offset`（`getBlendOffset`，L148-153，基于旧高度）会影响 initialDensity 的 shift，但同样需要旧高度数据。
- 前序否证（Beardifier）结论不变；差异仍指向 **badlands 方块替换 surface rule（terracotta band）** 及少量 **aquifer/洞穴判定** 差异（详见 `beardifier-analysis.md` §4-5）。

## 置信度
**candidate**（否证方向）。Blender 激活条件、blend_density 语义、NO_BLENDING 恒等均为确定性源码事实；唯一前提是「参照世界干净全新」，与任务背景一致，最终由用户拍板。
