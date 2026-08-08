# Beardifier（结构密度修正）分析 — 8576 seed chunk(50,-26)

> 项目：CoreSwap（C++ 逐位对齐 vanilla 1.20.1 世界生成）
> seed=8576294172403134396，区域 6×6 @ (720,-432)，代表列 (810,-411)（biome=eroded_badlands）
> 任务：验证「结构 Beardifier（密度修正）未实现」是否为 chunk(50,-26) 797 块方块差异的根因
> 产物日期：本次调查

---

## 1. blockStateSampler 组成（源码确认）

`net/minecraft/world/gen/chunk/ChunkNoiseSampler.java` 构造段 L176-186：

```java
Builder<ChunkNoiseSampler.BlockStateSampler> builder = ImmutableList.builder();
DensityFunction densityFunction = DensityFunctionTypes.cacheAllInCell(
        DensityFunctionTypes.add(noiseRouter2.finalDensity(), DensityFunctionTypes.Beardifier.INSTANCE)
    )
    .apply(this::getActualDensityFunction);
builder.add(pos -> this.aquiferSampler.apply(pos, densityFunction.sample(pos)));
if (chunkGeneratorSettings.oreVeins()) {
    builder.add(OreVeinSampler.create(noiseRouter2.veinToggle(), noiseRouter2.veinRidged(), noiseRouter2.veinGap(), noiseConfig.getOreRandomDeriver()));
}
this.blockStateSampler = new ChainedBlockSource(builder.build());
```

**结论：游戏实际方块判定 = `aquifer(finalDensity + Beardifier)`，无其他密度项。**
- 唯一附加项是 `OreVeinSampler`（`oreVeins=true` 时存在），只生成矿石脉方块（花岗岩/闪长岩/安山岩/凝灰岩/深板岩/铜/铁），**不产生 terracotta**，非本差异来源。
- L469-471：`getActualDensityFunction` 把 `Beardifier.INSTANCE` 替换为真实实例：
  ```java
  if (function == DensityFunctionTypes.Beardifier.INSTANCE) {
      return this.beardifying;
  }
  ```
- **真实 chunk 生成路径**（`NoiseChunkGenerator.java` L102-111）传入的 beardifying 是结构感知实现：
  ```java
  StructureWeightSampler.createStructureWeightSampler(world, chunk.getPos())
  ```
- 注意：`NoiseChunkGenerator.sampleHeightmap`（L199-209，用于 getHeight/高度图查询）用的是恒 0 的 `DensityFunctionTypes.Beardifier.INSTANCE`，**不影响 chunk 方块生成**。

→ 任务假设的入口成立：C++ 未实现 Beardifier 时，其密度 = 纯 finalDensity，而游戏实际 = finalDensity + StructureWeightSampler。

---

## 2. Beardifier 公式（`net/minecraft/world/gen/StructureWeightSampler.java`，源码原样）

### piece 贡献（L86-106）
```java
BlockBox blockBox = piece.box();
int l = piece.groundLevelDelta();
int m = Math.max(0, Math.max(blockBox.getMinX() - i, i - blockBox.getMaxX())); // X 向到 box 距离
int n = Math.max(0, Math.max(blockBox.getMinZ() - k, k - blockBox.getMaxZ())); // Z 向到 box 距离
int o = blockBox.getMinY() + l;   // 结构"地面线"
int p = j - o;                    // 相对地面线的 Y 偏移

int q = switch (piece.terrainAdjustment()) {
    case NONE -> 0;
    case BURY, BEARD_THIN -> p;
    case BEARD_BOX -> Math.max(0, Math.max(o - j, j - blockBox.getMaxY()));
};

d += switch (piece.terrainAdjustment()) {
    case NONE -> 0.0;
    case BURY -> getMagnitudeWeight(m, q, n);
    case BEARD_THIN, BEARD_BOX -> getStructureWeight(m, q, n, p) * 0.8;
};
```

### junction 贡献（L110-116）
```java
d += getStructureWeight(r, l, m, l) * 0.4;   // r,l,m = 相对 jigsaw 连接点的偏移
```

### 两个权重函数
```java
// BURY：正权重（把结构区域填实/埋藏），最大 1.0，线性衰减
getMagnitudeWeight(x, y, z) = MathHelper.clampedMap(MathHelper.magnitude(x, y / 2.0, z), 0.0, 6.0, 1.0, 0.0);

// BEARD_THIN / BEARD_BOX / junction
getStructureWeight(x, y, z, yy):
    i = x + 12; j = y + 12; k = z + 12;
    if (i,j,k 任一不在 [0,24)) return 0.0;
    d = yy + 0.5;
    e = MathHelper.squaredMagnitude(x, d, z);          // x²+d²+z²
    f = -d * MathHelper.fastInverseSqrt(e / 2.0) / 2.0; // = -d / (2·√(e/2))
    return f * STRUCTURE_WEIGHT_TABLE[k*576 + i*24 + j];
// 表项 = exp(-(x²+(y+0.5)²+z²)/16)（calculateStructureWeight / structureWeight，L162-169）
```

### 数学表达与几何
对固定 XZ（在结构 piece box 内或其 ±12 格内），沿 Y 的偏移为：

- **BEARD_THIN / BEARD_BOX**（×0.8）：
  `B(y) = 0.8 · [-(p+0.5) / (2·√((x²+(p+0.5)²+z²)/2))] · exp(-(x²+(q+0.5)²+z²)/16)`
  其中 p = y − (box.minY+Δ)。符号：**y 在地面线下（p<0）→ 正偏移（抬升）**；**地面线上（p>0）→ 负偏移（挖空）**。
  - 影响半径 ±12 格（24×24×24 查找表）；XZ 距离 m,n 超过 12 → 0。
  - 轴线上（x=z=0）|f| ≡ 0.7071，最大正偏移 ≈ 0.7071 × 1.0 × 0.8 ≈ **+0.566**（恰好在地面线下方 1 格）；向上/向下指数衰减（exp(-d²/16)），到 12 格处 ≈ 0。
- **BURY**：`B(y) = clamp01(1 − ‖(m, (y−o)/2, n)‖/6)`，box 内中心最大 **+1.0**，半径约 6（y 半轴）~12 衰减到 0。
- **junction**：`B(y) = 0.4 · getStructureWeight(...)`，最大 ≈ +0.283（仅 jigsaw 结构存在）。

> 结论：Beardifier 的**正偏移只出现在结构 piece box 地面线（minY+Δ）之下约 12 格内**，幅度上限 +0.566（BEARD）/ +1.0（BURY）；上方为负（挖空）。这是判断"能否把 (810,76,-411) 的 -0.038 拉成正"的关键几何约束。

---

## 3. (810,-411) 结构判定

### 3a. 静态配置证据（badlands 区域）

`eroded_badlands` biome 在 1.20.1 参与 `worldgen/structure` 配置的结构（`net/minecraft/world/gen/structure/Structures.java` bootstrap）：

| 结构 | terrain_adaptation | 是否参与 Beardifier |
|---|---|---|
| mineshaft / mineshaft_mesa（badlands 标配） | `NONE`（L76 / L85） | ❌ 不参与 |
| ruined_portal_desert（badlands 有 tag） | `NONE`（L281） | ❌ 不参与 |
| **stronghold**（badlands 有 tag） | **`BURY`**（L131） | ✅ 唯一候选 |
| village_savanna / village_desert / village_plains / pillager_outpost | `BEARD_THIN` | ❌ biome tag 不含 badlands（savanna/desert/plains 等专属 biome 列表） |
| ancient_city | `BEARD_BOX` | ❌ deep dark |

- **badlands 中唯一参与 Beardifier 的结构是 stronghold（BURY）**；废弃矿井是 NONE（挖洞的"地下结构"并不用 Beardifier）。
- stronghold 几何：结构深埋地下（起始高度约 y=0~40）。其 BURY 权重 `getMagnitudeWeight(m, q, n)` 的 y 参数 q = p = y − box.minY ≈ 36+（对 y≈76 而言），`magnitude(0, q/2, 0) ≥ 18 > 6` → clampedMap 截断为 **0**。**stronghold 权重几何上无法到达 y≈76+**。
- 因此：**静态配置层面，(810,76,-411) 处 Beardifier 正偏移不存在**（唯一候选 stronghold 贡献为 0）。

### 3b. 参照反推（特征校验）

- Java 参照 (810,-411) 列 y≈0-118 为 terracotta 系（band 结构），**无结构方块**（无橡木原木/圆石/木板/铁轨等）→ 没有村庄/outpost piece box 覆盖该列（若有，参照会出现结构方块）。
- BEARD 抬升的形态学特征应是"box 底高度的**台地**"（box 覆盖的 XZ 区域整体抬到同一高度）；而参照特征为**单列柱/带**（118 高原仅个别列，周边 74/87/108 急降）→ 与结构台地特征不符。
- （此为参照反推，强弱受限于既有导出数据；精确确认仍需运行时结构查询。）

### 3c. 需运行时验证项（明确标注）

结构实例位置由 seed + structure set placement 计算，静态数据无法枚举全部实例。**无法静态确认的缺口**：
1. `structureAccessor.getStructureStarts(chunk(50,-26), s → s.getTerrainAdaptation() != NONE)` 是否为空（chunk 的 STRUCTURE_REFERENCES 及 8-chunk 半径内结构 start）。
2. 若存在 stronghold，其 box 精确范围（确认权重确实衰减为 0）。

---

## 4. 定量：Beardifier 能否 > 0.038？

`(810,76,-411)` 要判方块，需 `finalDensity(−0.038) + Beardifier > 0`，即 **Beardifier > 0.038**（任务给定口径）。

按第 2 节公式逐一排除候选：

1. **stronghold（BURY）**：`getMagnitudeWeight(m, q, n)`，q≈36+ → 权重 **0**。✗
2. **邻 savanna 的村庄/outpost（BEARD_THIN）**：若其 box 扩展 12 格恰好覆盖该列，box 地面线 o ≈ savanna 地表（≈70-74）< 76 → p>0 → 贡献为**负**（挖空方向），只会让密度更负。✗（且若覆盖，参照应含结构方块，未观察到）
3. **任意 BEARD 结构使 y=76 有正偏移**：要求 box 地面线 o ≥ 77（即结构底部高于周围地表 44+ 格）——不存在此类结构。即使存在，正偏移半径仅 12 格，无法同时解释参照 0-118 的 terracotta 带。✗
4. **junction（0.4 权重）**：仅 jigsaw 结构（村庄/outpost）存在 junction，badlands 不生成。✗

**论证结论：该列 Beardifier ≤ 0（或 = 0），无法满足 > 0.038。**

附加佐证（差异模式）：
- `mismatch_8576*.txt` 中 chunk(50,-26) z=-411 列的差异主要是 **C++ base 方块（stone/gravel/deepslate 等）vs Java 染色 terracotta（426/433/437/439/494）**，另有少量 **C++ air vs Java terracotta**。
- Beardifier 只改变"方块 vs 空气"判定，**不改变 stone→terracotta 的染色**。大规模"未染色 vs 染色"差异属于**方块替换 surface rule（badlands band）**范畴，与 Beardifier 特征不符。

---

## 5. 结论

**Beardifier（结构密度修正缺失）不是 chunk(50,-26) 差异的根因 —— 否证（candidate 级）。**

理由链：
1. 游戏实际 = finalDensity + Beardifier，C++ 未实现 Beardifier 属实（功能缺口真实存在），但 **本例位置 Beardifier 贡献几何上为 0 或为负**：badlands 唯一非 NONE 结构是 deep 的 stronghold（BURY 权重衰减为 0），BEARD 结构在 badlands 不存在，且 118 高原形态与"结构台地"不符、该列无结构方块。
2. 需要 +0.038 的判定点 (810,76,-411) 无法被任何结构提供该偏移；要解释参照 0-118 全 terracotta 带更需 +偏移跨 49 格（远超 12 格衰减半径）——单一结构 box 几何上不可能。
3. 差异模式（base 方块 vs 染色 terracotta、整列大跨度 band）指向 **方块替换 surface rule（eroded_badlands terracotta band）** 及少量 **aquifer/洞穴判定** 差异，而非结构密度修正。

### 需进一步验证（才能彻底关闭）
- **运行时**：`StructureAccessor` 查 chunk(50,-26)（含 8-chunk 半径引用）确认无 `terrainAdaptation != NONE` 的 StructureStart；若存在 stronghold 打印其 box 验证权重为 0。
- **surface rule**：对比 C++ 与 Java 在 badlands 的方块替换实现（`overworld.json` surface_rule 的 badlands/eroded_badlands 分支、band 垂直范围）；用"仅 finalDensity 生成 base 方块 vs 参照最终方块"对比验证差异是否仅染色。
- 说明：与 -288 岛（Beardifier 相关候选）不是同一结构环境，不可互证；-288 岛若在 ancient_city/stronghold 附近仍可能为 Beardifier，需单独验证。

## 置信度
**candidate**（否证方向）。核心源码事实（Beardifier 公式、badlands 结构配置、几何衰减、blockStateSampler 组成）为确定性证据；参照列形态与 mismatch 模式依赖既有导出/历史数据，需运行时复验后由用户拍板。
