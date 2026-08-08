# SurfaceBuilder 分析 — 为什么 NOISE 阶段是 air 的位置在 SURFACE 阶段变成 terracotta

> 项目：CoreSwap（C++ 重写 MC 1.20.1 世界生成，逐位对齐 vanilla）
> 场景：seed=8576294172403134396，chunk(50,-26)，代表列 (810,-411)（eroded_badlands）
> Diag810（Java BlockProbe 实测）：NOISE 后 y=60-73 stone、y=74-120 air；SURFACE 后 y=60 light_gray_terracotta、y=61-68 stone、y=69-118 terracotta 带（全带色）、y=119+ air
> 任务：解释 Java buildSurface 在「NOISE 是 air」的 y=74-118 上生成 terracotta 的机制，并列出 C++ surface.h 的差异
> 结论先行：**Java 的 `SurfaceBuilder.placeBadlandsPillar`（eroded_badlands 专属）先把支柱顶以下的 air 全部填成 defaultState（stone），随后主循环把这些新填的 stone 当 default 块应用 badlands surface 规则 → `terracottaBands()` 染成 terracotta。C++ surface.h 完全没有这一步。**

---

## 1. Java SurfaceBuilder.buildSurface 完整机制（源码行号）

文件：`net/minecraft/world/gen/surfacebuilder/SurfaceBuilder.java`

### 1.1 逐列遍历：起点 = heightmap(WORLD_SURFACE_WG)+1，且 **pillar 之后重采样**

```java
// L113-114  外层双循环：k/l = chunk 局部 x/z
for (int k = 0; k < 16; k++) {
    for (int l = 0; l < 16; l++) {
        // L117  o = 当前列 WORLD_SURFACE_WG 高度图 + 1（pillar 之前采样）
        int o = chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l) + 1;
        // L119  biome 采样高度 = o（eroded_badlands 判定用）
        RegistryEntry<Biome> registryEntry = biomeAccess.getBiome(mutable2.set(m, useLegacyRandom ? 0 : o, n));
        // L120-122  eroded_badlands → 先执行 placeBadlandsPillar（见 §1.4）
        if (registryEntry.matchesKey(BiomeKeys.ERODED_BADLANDS)) {
            this.placeBadlandsPillar(blockColumn, m, n, o, chunk);
        }
        // L124  p = 高度图 + 1（**pillar 之后重采样**——pillar 填的 stone 会抬升高度图！）
        int p = chunk.sampleHeightmap(Heightmap.Type.WORLD_SURFACE_WG, k, l) + 1;
        // L129  t = 世界底 y（overworld -64）
        int t = chunk.getBottomY();
        // L131  主循环：从 p 向下遍历到世界底
        for (int u = p; u >= t; u--) {
```

要点：
- **起点 p 不是固定值**。placeBadlandsPillar 先把支柱区 air 填成 stone，stone 经 `ProtoChunk.setBlockState` → `Heightmap.trackUpdate`（见 §1.5）把 WORLD_SURFACE_WG 抬升到 pillar 顶+1，因此 L124 重采样得到 `p = pillar顶 + 2`。**主循环遍历范围由此覆盖整个 pillar 填充区**。
- `Heightmap.Type.WORLD_SURFACE_WG` = `NOT_AIR`（`Heightmap.java:24,143`：`state -> !state.isAir()`），即「最高非空气块」高度。

### 1.2 对 air / 流体 / default 的处理（L132-162）

```java
// L132  取当前块
BlockState blockState = blockColumn.getState(u);
if (blockState.isAir()) {                    // L133  air：仅重置计数器
    q = 0;                                   // q = 连续非空气块数（stoneDepthAbove）
    r = Integer.MIN_VALUE;                   // r = 最高流体 y+1（fluidHeight）
} else if (!blockState.getFluidState().isEmpty()) {   // L136  流体
    if (r == Integer.MIN_VALUE) r = u + 1;
} else {                                     // L140  非 air 非流体
    if (s >= u) {                            // L141  s = 下方第一个非 default 块位置
        s = DimensionType.field_35479;
        for (int v = u - 1; v >= t - 1; v--) {
            if (!this.isDefaultBlock(blockColumn.getState(v))) { s = v + 1; break; }
        }
    }
    q++;
    int vx = u - s + 1;                      // stoneDepthBelow
    materialRuleContext.initVerticalContext(q, vx, r, m, u, n);
    if (blockState == this.defaultState) {   // L156  只有 default(=stone) 块应用规则
        BlockState blockState2 = blockStateRule.tryApply(m, u, n);
        if (blockState2 != null) blockColumn.setState(u, blockState2);   // L159 写方块
    }
}
```

- `isDefaultBlock`（L181-183）= `!state.isAir() && state.getFluidState().isEmpty()`（主世界 default = stone）。
- **对 air 本身不写方块**——air 分支只重置 q/r。**但是**：air 已被 `placeBadlandsPillar` 先一步变成 stone（default），于是 L156 的 default 判定命中 → 规则被应用。**「在 air 上写 terracotta」的真相 = 先在 air 上写 stone（pillar），再把 stone 染成 terracotta（规则）。**
- 写入走 `BlockColumn.setState` → `chunk.setBlockState(pos, state, false)`（L93-101）。

### 1.3 规则树入口与 surface() 外层条件

- `VanillaSurfaceRules.java:281-282`：`materialRule10 = condition(MaterialRules.surface(), materialRule9)`，`builder.add(surface ? materialRule10 : materialRule9)`。主世界（surface=true）时 **badlands 整段被 `surface()`（= above_preliminary_surface）包住**。
- `surface()` = `SurfaceMaterialCondition` → `surfacePredicate`（`MaterialRules.java:743-756, 413`）= `blockY >= estimateSurfaceHeight()`（`MaterialRules.java:567-572`）。
- `estimateSurfaceHeight()`（`MaterialRules.java:488-516`）= `floor(lerp2((blockX&15)/16, (blockZ&15)/16, 4角est)) + runDepth - 8`，其中 4 角 est = `ChunkNoiseSampler.estimateSurfaceHeight`（基于 initialDensityWithoutJaggedness 扫描）。`runDepth` = `sampleRunDepth(blockX, blockZ)`（每列固定，`MaterialRules.java:459`）。
- **该条件是 y<surfaceMinY 时整个 mr9 被跳过的原因**（见 §2.3）。

### 1.4 placeBadlandsPillar —— 核心机制（air→stone）

```java
// L208-234
private void placeBadlandsPillar(BlockColumn column, int x, int z, int surfaceY, HeightLimitView chunk) {
    double e = Math.min(Math.abs(this.badlandsSurfaceNoise.sample(x, 0.0, z) * 8.25),
                        this.badlandsPillarNoise.sample(x * 0.2, 0.0, z * 0.2) * 15.0);
    if (!(e <= 0.0)) {                                  // L211  e>0 才触发
        double h = Math.abs(this.badlandsPillarRoofNoise.sample(x * 0.75, 0.0, z * 0.75) * 1.5);
        double i = 64.0 + Math.min(e * e * 2.5, Math.ceil(h * 50.0) + 24.0);
        int j = MathHelper.floor(i);                    // L216  pillar 顶 y
        if (surfaceY <= j) {                            // L217  表面 ≤ pillar 顶
            for (int k = j; k >= chunk.getBottomY(); k--) {   // L218-227  检查：遇 water 则整体返回
                BlockState blockState = column.getState(k);
                if (blockState.isOf(this.defaultState.getBlock())) break;   // 遇 stone 停（校验）
                if (blockState.isOf(Blocks.WATER)) return;
            }
            for (int k = j; k >= chunk.getBottomY() && column.getState(k).isAir(); k--) {
                column.setState(k, this.defaultState);  // L230  **把 air 填成 stone！**
            }
        }
    }
}
```

- 三个 2D 噪声：`badlands_surface`、`badlands_pillar`、`badlands_pillar_roof`（`SurfaceBuilder.java:46-48, 64-66`；噪声键 `NoiseParametersKeys.BADLANDS_PILLAR` 等）。
- 在本场景（(810,-411)）：pillar 顶 j≈118 ≥ 表面 74 → **y=74..118 全部由 air 变为 stone**。这与 Diag810 完全吻合（NOISE 后 74-120 air → SURFACE 后 74-118 被染色，119+ 仍 air：pillar 只填到 j=118，119/120 保持 air）。
- `placeIceberg`（L236-275，frozen ocean）是同类「直接在 air/water 上写方块」的机制，本项目区域无关，但 C++ 同样缺失时会造成同构差异。

### 1.5 pillar 填的 stone 如何抬升高度图（主循环起点变高的依据）

- `placeBadlandsPillar` 用 `column.setState` → `chunk.setBlockState(pos, state, false)`。
- `ProtoChunk.setBlockState`（`ProtoChunk.java:108-162`）在 **L153-155** 对 `this.getStatus().getHeightmapTypes()` 全部类型调 `heightmap.trackUpdate(m, j, o, state)`。
- SURFACE 状态（`ChunkStatus.java:114-119`）的 `getHeightmapTypes` = `PRE_CARVER_HEIGHTMAPS`（`ChunkStatus.java:33`）= `{OCEAN_FLOOR_WG, WORLD_SURFACE_WG}`。
- `trackUpdate`（`Heightmap.java:73-100`）：新块 y ≥ 当前高度图值-1 时更新为 y+1。pillar 从顶向下填，**第一个填充（y=j≥74）把高度图从 74 抬到 j+1**，后续填充 y≤j-1 ≤ (j+1)-2 不再触发。→ 高度图最终 = j+1，主循环 `p = j+2`，从 pillar 顶上方 1 格开始向下遍历，覆盖全部新填 stone。

---

## 2. terracotta 带规则触发条件分析（为什么 y=69-118 触发）

### 2.1 badlands 段规则结构（`VanillaSurfaceRules.java:206-237`）

```java
condition(biome(BADLANDS, ERODED_BADLANDS, WOODED_BADLANDS), sequence(
    condition(STONE_DEPTH_FLOOR,                    // ① q <= 1（仅表层第一块）
        sequence(mc2→ORANGE, mc4→sequence(mc16/17/18→TERRACOTTA, terracottaBands()),
                 mc8→(RED_SANDSTONE/RED_SAND), not(mc11)→ORANGE, mc10→WHITE, mr3)),
    condition(mc3,                                  // ② aboveYWithStoneDepth(63,-1)
        sequence(mc7→condition(not(mc4), ORANGE), terracottaBands())),
    condition(STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH, // ③ q <= 1+runDepth
        condition(mc10, WHITE_TERRACOTTA))
))
```

关键条件（`VanillaSurfaceRules.java:57-69, 167-169`；C++ 同定义 `surface.h:461-471, 524-526`）：

| 条件 | 定义 | Java 判定（`MaterialRules.java`） |
|---|---|---|
| mc2 | `aboveY(fixed(256), 0)` | `blockY >= 256` |
| mc3 | `aboveYWithStoneDepth(fixed(63), -1)` | `blockY + q >= 63 - runDepth`（AboveY.test L160-163） |
| mc4 | `aboveYWithStoneDepth(fixed(74), 1)` | `blockY + q >= 74 + runDepth` |
| mc7 | `aboveY(fixed(63), 0)` | `blockY >= 63` |
| STONE_DEPTH_FLOOR | `stoneDepth(0,false,0,FLOOR)` | `q <= 1`（StoneDepthPredicate L729-736） |
| STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH | `stoneDepth(0,true,0,FLOOR)` | `q <= 1 + runDepth` |
| mc16/17/18 | `noiseThreshold(SURFACE, …)` | surface 噪声窗口 → plain terracotta |
| terracottaBands() | 无条件 | `getTerracottaBlock(x,y,z)`（`MaterialRules.java:783-785` → `SurfaceBuilder.java:323-326`） |

（`runDepth` = `sampleRunDepth`，每列固定 ≈ -3..9；注意与 buildSurface 的 q 无关。）

### 2.2 关键几何事实：`blockY + q ≡ j + 1`（支柱填充区恒为常数）

主循环从 pillar 顶 j 开始逐块向下数 q（q 仅在遇 air 时清零；pillar 把 74..118 填成 stone，中间无 air）：

- y=j：q=1 → `blockY + q = j+1`
- y=j-1：q=2 → `j-1+2 = j+1`
- … y=74：q=j-73 → `74 + (j-73) = j+1`
- … y=69：q=j-68 → `69 + (j-68) = j+1`

对 j≈118，**`blockY + q` 恒等于 119**，远超 63±runDepth、74±runDepth。

因此 pillar 填充区内：
- **mc3 恒真**（119 ≥ 63-runDepth，runDepth 最大约 9，63-9=54）。
- **mc4 恒真**（119 ≥ 74+runDepth，74+9=83）。
- STONE_DEPTH_FLOOR（q≤1）只对 y=j（表层第一块）命中 → 走 ① 的 mc4 → terracotta 带；
- 其余 y（q≥2）STONE_DEPTH_FLOOR false → 走 ② mc3 段：mc7（blockY≥63）只影响 ORANGE 分支的取舍，`terracottaBands()` 是无条件兜底 → **命中带色 terracotta**（diag 的 white/orange/plain/light_gray/brown/red 全带色即来自 `getTerracottaBlock` 的 192 带数组，`SurfaceBuilder.java:277-326`）。
- mc16/17/18 只在 surface 噪声恰好落入窗口时给出 plain terracotta（diag 列里也有 `minecraft:terracotta` 项，即此路径）。

### 2.3 下边界（y=69）由 surface() 截断 —— 唯一能解释 y=61-68 保持 stone 的机制

- 若 surface() 为真，y=61-68（q=51..58，blockY+q 恒 119）同样必走 mc3 段 terracottaBands → terracotta。**diag 显示 stone，唯一解释是这些 y 的 `surface()` 为假**：`blockY < surfaceMinY = floor(lerp2(4角est)) + runDepth - 8`，整段 mr9 跳过 → 保持 NOISE 阶段的 stone。
- 由此反推 surfaceMinY = 69（y=69 是第一个 ≥ surfaceMinY 的块）⇒ 4 角 est 插值 ≈ 69 + 8 - runDepth ≈ 77（runDepth≈0 时）。
- **上层语义**：eroded_badlands 的 pillar 是 finalDensity（jagged 尖峰）的地形，而 `estimateSurfaceHeight` 基于平滑的 initialDensityWithoutJaggedness（阈值 0.390625 向下扫描），两者高度差（≈118 vs ≈77）正是「NOISE 是 air、SURFACE 是 pillar+terracotta」的密度/表面分离。Diag810 的 BeardDiag（finalDensity=-0.0397/-0.1188 <0）与 est 无关，不矛盾。
- y=119+ 仍 air：surface() 为真但主循环里它们是 air（pillar 只填到 j=118），air 分支不写方块。

### 2.4 未解点：y=60 light_gray_terracotta

同一列 surfaceMinY 是常数，y=60 < 69 时 surface() 应为假 → 应保持 stone。diag 显示 light_gray 与 y=61-68 的 stone 无法用纯 buildSurface 机制自洽解释（已排除 biome 变化、mc3/mc4 恒真性、pillar 只填 air）。最可能是 Diag810 的 SURFACE dump 被连带推进（BlockProbe 文档已知：SURFACE 导出可能含 FEATURE/结构污染）或相邻列/边界效应。**标注为次要待验证项，不影响 §3 的核心结论。**

---

## 3. C++ surface.h 对照差异清单

文件：`versions/1.20.1/cpp/worldgen/src/surface.h` + `worldgen_api.cpp`

| # | 差异点 | Java | C++ |
|---|---|---|---|
| 1 | **placeBadlandsPillar（air→stone 填充）** | `SurfaceBuilder.java:208-234`，eroded_badlands 每列在规则应用前执行 | **完全缺失**。`grep` 全 cpp 目录无 pillar 逻辑（仅注册了 `badlands_pillar`/`badlands_pillar_roof`/`badlands_surface` 噪声，`worldgen_api.cpp:390-391`，无使用方） |
| 2 | **主循环起点 p** | pillar 后重采样 heightmap+1 = pillar顶+2（`SurfaceBuilder.java:124`） | 固定为 NOISE 后 heightmap+1（`surface.h:701`），**不覆盖 pillar 填充区** |
| 3 | **对 air 的处理** | air 分支只重置 q/r，不写；但 pillar 已先填 stone | `surface.h:721-723`：`isAir → q=0, r=INT32_MIN; continue`（跳过规则）——**无 pillar 前置时行为等价，但缺失 pillar 导致该区永远不被规则访问** |
| 4 | **规则树本体**（badlands 段、mc3/mc4/mc7/mc16-18、terracottaBands、STONE_DEPTH_*） | `VanillaSurfaceRules.java:206-237` | **已对齐**（`surface.h:590-614`，条件定义 `surface.h:461-471,524-526`；StoneDepthCond `surface.h:228-234`；TerracottaBandsRule `surface.h:292-294`）——规则存在但无 pillar 产的 stone 可染 |
| 5 | **surface()（above_preliminary_surface）** | `blockY >= floor(lerp2(4角est)) + runDepth - 8` | **已对齐**（`surface.h:261-278` SurfaceCondC；4 角 est 来自 `aquifer->estimateSurfaceHeight`，`worldgen_api.cpp:746-750`）——该条件 C++/Java 一致，y<surfaceMinY 时两边都跳过 mr9 |
| 6 | s（stoneDepthBelow）判定集合 | `isDefaultBlock` = 非 air 且 fluidState 空 | `surface.h:736` 用 `!= air/water/lava`；主世界默认只有 water/lava 流体时等价；其他维度/自定义流体存在理论差异（影响 CEILING 类规则，与 badlands 无关） |
| 7 | 列起点以上越界 | HeightLimitView 返回 AIR | `surface.h:714-715` 显式 air——一致 |

**差异 1/2 是 y=74-118「C++ air vs Java terracotta」的唯一来源**（badlands 规则本体两方一致，仅缺「先填 stone」这一步）。与已有记录吻合：`docs/06-surface-rules.md:131`「8576 剩余 826 块 = terracotta 带边缘，C++ 判 air vs Java terracotta」；`beardifier-analysis.md` 已否证 Beardifier 为该差异根因、并指出差异形态是「base 方块 vs 染色 terracotta」——正指向 surface rule 染色而非密度。

---

## 4. 结论

**机制（确定性，源码逐行）：** Java 在 SURFACE 阶段对 eroded_badlands 列先执行 `placeBadlandsPillar`（`SurfaceBuilder.java:208-234`）——用 2D 噪声算出 pillar 顶 j，若表面≤j 则**把 j 以下全部 air 填成 defaultState(stone)**（L229-231），该写入经 `ProtoChunk.setBlockState → Heightmap.trackUpdate`（`ProtoChunk.java:153-155`，SURFACE 状态含 WORLD_SURFACE_WG）**抬升高度图到 j+1**；随后主循环从重采样的 `p=j+2` 向下遍历（L124,131），这些新填 stone 满足 `blockState == defaultState`（L156）→ 应用规则树。badlands 段（`VanillaSurfaceRules.java:206-237`）中 mc3 段因 `blockY + q ≡ j+1` 恒真而兜底命中 `terracottaBands()` → **NOISE 阶段的 air 变成带色 terracotta**。下边界由外层 `surface()`（above_preliminary_surface = `blockY >= floor(lerp2(4角est)) + runDepth - 8`）截断：y<surfaceMinY 时整段规则跳过 → 本例 y=61-68 保持 stone、y=69-118 染色。

**「Java 在 air 上生成方块」的准确表述**：Java buildSurface 对 air 本身同样跳过规则应用；terracotta 是「air→stone（pillar 前置）→规则染色」两步的结果。

**C++ 需改哪里（对齐方向）：**
1. 在 `SurfaceBuilder::buildSurface` 每列主循环前，对 biome=eroded_badlands 的列实现 `placeBadlandsPillar` 等价逻辑：`e = min(|badlands_surface(x,0,z)*8.25|, badlands_pillar(x*0.2,0,z*0.2)*15.0)`；`j = 64 + min(e²*2.5, ceil(|badlands_pillar_roof(x*0.75,0,z*0.75)*1.5|*50)+24)`；`heightmap+1 <= j` 时从 j 向下把 air 填 stone（遇 water 整列跳过；遇 stone 停）。
2. **同步把该列 `heightmap[l*16+k]` 更新为 `j+1`**（Java trackUpdate 等效；j≥原 heightmap 时生效），主循环起点随之抬高到 j+2。
3. 时序对齐 Java：biome 采样用 pillar 前 heightmap（o），pillar 填充后重采样 p。
4. 需要 `badlands_surface`/`badlands_pillar`/`badlands_pillar_roof` 三个 2D 噪声的可采样句柄（C++ 已注册到噪声表，取用即可）。
5. （可选，同类）frozen ocean 的 `placeIceberg`（`SurfaceBuilder.java:236-275`）同样缺失，属另一个 biome 的同类差异。

**判断**：C++ 当前行为是「缺前置步骤」而非规则错误；Java 行为是 vanilla 预期。**C++ 应补 pillar，而不是改 air 跳过逻辑。**

## 置信度

**candidate**（机制核心 = 源码确定性证据，接近 confirmed；边界数值需运行时复核）：

- 确定性：§1 全部为 mc_src_extract 源码行级引用；§2.2 的 `blockY+q≡j+1` 为代数恒等；§3 差异 1/2 已与 06 文档、beardifier 分析交叉印证。
- 需运行时验证的缺口：
  1. `WG_ESTDUMP` 验证 (810,-411) 的 4 角 est 插值 + runDepth - 8 是否恰为 69（验证 §2.3 的 surfaceMinY=69 反推）。
  2. Diag810 复跑确认 y=60 light_gray 的来源（SURFACE dump 连带推进/FEATURE 污染？），以及 pillar 顶 j 实际值（应为 ≈118，可由 badlands 噪声复算验证）。
  3. 修复实现后对比 mismatch 数量（预期 chunk(50,-26) 的「C++ air/stone vs Java terracotta」差异消失，整体 826 块缺口收窄）。
