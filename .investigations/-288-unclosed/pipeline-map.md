# -288 未闭合 ~23% 差异 — surface 管线勘探地图（recode.scout）

> 课题：seed=-8248318472910187742，-288,-256 4×4 chunk，block_probe 匹配率 95.7376%。
> 已闭合（范围外 FEATURE/STRUCTURE）≈73%；本文档聚焦**未闭合 ≈23%**：海底边界（water↔stone/dirt/sand ≈7400）、gravel（≈4900）、表面规则（sand/sandstone/dirt 互换 ≈2900）。
> 勘探范围：只读读文件 + 逐条对照，不跑命令不改代码。
> 产物格式：draft（管线地图 + C++/Java 对照 + 偏差点清单 + 候选机制）。

---

## 0. 关键文件索引

| 角色 | Java（1.20.1 yarn，包 `world.gen.surfacebuilder`） | C++ 复刻 |
|---|---|---|
| surface 规则引擎 | `data/mc_src_extract/net/minecraft/world/gen/surfacebuilder/MaterialRules.java` | `cpp/worldgen/src/surface.h`（MaterialRules 引擎 + 翻译） |
| vanilla 规则树 | `data/mc_src_extract/.../surfacebuilder/VanillaSurfaceRules.java` | `surface.h` `buildOverworldRule()` |
| surface builder 引擎 | `.../surfacebuilder/SurfaceBuilder.java` | `surface.h` `SurfaceBuilder::buildSurface` |
| density/海底液面 | `.../world/gen/chunk/ChunkNoiseSampler.java`（estimateSurfaceHeight L222） | `worldgen_api.cpp` fillOneChunk 3a/3b + `aquifer.h` |
| aquifer（判水） | ChunkNoiseSampler → AquiferSampler | `aquifer.h` |
| 噪声参数 | `data/noise_params.json`（Java 导出） | `worldgen_api.cpp` 硬编码表 + JSON 覆盖 |
| 先期结论 | — | `.investigations/-288-reopen/summary-final.md`（已破案边界） |

> ⚠️ 注意包名：任务描述写 `world/level/levelgen/surfacebuilders`，实际 1.20.1 yarn 是 `world/gen/surfacebuilder`（无 s，非 levelgen）。

---

## 1. Java 侧 surface 管线（流程）

```
ChunkStatus.NOISE
  └─ NoiseChunkGenerator.fillFromNoise
       ├─ density: CellCache(add(DensityInterpolator(finalDensity), Beardifier))
       │            → 逐块 finalDensity；y 上限 noiseHeight=128（上方 air）
       ├─ aquifer: ChainedBlockSource(aquifer.apply → oreVein.apply)
       │            density>0 → null(stone)；density≤0 → 按 fluidLevel 判 water/lava/air
       ├─ oreVein（范围外）
       └─ heightmap: WORLD_SURFACE_WG（NOT_AIR，**水也算表面**）

ChunkStatus.SURFACE（同一 chunk 顺序阶段）
  └─ NoiseChunkGenerator.buildSurface(L242)
       └─ NoiseConfig.getSurfaceBuilder().buildSurface(L266)
            └─ SurfaceBuilder.buildSurface(L72)
                 ├─ materialRule.apply(ctx) → BlockStateRule（规则树一次编译）
                 ├─ 逐列: o = WORLD_SURFACE_WG + 1（pillar 前）
                 │    biome = getBiome(m, useLegacyRandom?0:o, n)（仅 eroded_badlands 触发 pillar）
                 ├─ placeBadlandsPillar（eroded_badlands 专属）
                 ├─ p = WORLD_SURFACE_WG + 1（pillar 后）
                 │    ctx.initHorizontalContext(m, n) → runDepth = sampleRunDepth(x,z)
                 └─ for u = p .. bottomY:
                      ├─ isAir → q=0, r=MIN
                      ├─ isFluid（fluidState 非空）→ r = u+1（最高流体 y+1，仅首次）
                      ├─ else（default 块判定）:
                      │    s = 第一个非 default 块位置（u-1 向下找 air/fluid）
                      │    q++（stoneDepthAbove 扫描计数）
                      │    vx = u - s + 1（stoneDepthBelow）
                      │    ctx.initVerticalContext(q, vx, r, m, u, n)
                      │    if (state == defaultState[stone]) → blockStateRule.tryApply(x,u,z)
                      └─ （frozen_ocean 特殊 → placeIceberg）
```

**核心理解（对应任务 A3）**：
- **海底高度不由 surface 规则决定**。海底 stone 顶由 density/aquifer 层决定（density>0→stone；density≤0→aquifer 判水）。surface 规则只对已是 default(stone) 的块染色。
- 海底列高度图 = 水面（水是 NOT_AIR）→ buildSurface 从水面+1 向下扫 → 遇水(fluid) 记 r=fluidHeight → 遇 stone(default) 应用规则。
- sea level=63 的判定在 **aquifer 的 fluidLevelSampler**：`y < 63 → water`（Java ChunkNoiseSampler L161 `AquiferSampler.seaLevel(fluidLevelSampler)`；C++ `aquifer.h` `defaultFluidLevel`: `blockY < -54 ? lava(-54) : water(63)`）。
- **surface 规则只替换 default block（stone）**。海底染色实际发生在 stone 列上，不是"建海底"。

---

## 2. 海底 gravel/sand/sandstone 规则逐条对照（对应任务 A1/A2）

### 2.1 规则定义（Java VanillaSurfaceRules L71-72 / C++ surface.h L481-482）

| 规则 | Java | C++ | 一致 |
|---|---|---|---|
| mr2（sand/sandstone） | `sequence(condition(STONE_DEPTH_CEILING, SANDSTONE), SAND)` | `sequence({condition(stoneDepth(0,false,0,true), B("sandstone")), B("sand")})` | ✅ |
| mr3（stone/gravel） | `sequence(condition(STONE_DEPTH_CEILING, STONE), GRAVEL)` | `sequence({condition(stoneDepth(0,false,0,true), B("stone")), B("gravel")})` | ✅ |

- `STONE_DEPTH_CEILING = stoneDepth(0,false,CEILING)`：`stoneDepthBelow <= 1 + 0 + 0`。
- **语义**：海底列从水面往下扫，最顶 1 格 stone（vx==1）→ STONE/STONE 面；往下 vx>1 → GRAVEL/SAND。**MC cold_ocean 海底表面即"1 格 stone + 下方 gravel"混合斑块**。
- 注：**海底表层其实是 STONE，gravel 在 stone 之下**（仅 vx<=1 时才表层 stone；vx>1 时当前块直接 gravel）。这与"海底 gravel 斑块"观察一致。

### 2.2 应用链（Java mr9 海底段 L263-270 / C++ surface.h L639-644）

```java
condition(STONE_DEPTH_FLOOR, sequence(
    condition(biome(FROZEN_PEAKS, JAGGED_PEAKS), STONE),                       // 山峰
    condition(biome(WARM_OCEAN, LUKEWARM_OCEAN, DEEP_LUKEWARM_OCEAN), mr2),    // 暖海沙
    mr3                                                                        // cold_ocean 等 → stone/gravel
))
```

C++ `surface.h` L639-644 逐条同构。**-288 区域 biome = cold_ocean（主体）+ river + beach + plains** → 海底走 mr3（stone/gravel）；beach 走 mc14 → RANGE_6 sandstone（见 2.4）。

### 2.3 条件变量核对（Java MaterialRules / C++ surface.h）

| 条件 | Java 公式 | C++ 公式 | 判定 |
|---|---|---|---|
| runDepth/surfaceDepth | `sampleRunDepth = (int)(surface(x,0,z)*2.75 + 3.0 + randomDeriver.split(x,0,z).nextDouble()*0.25)` | `surfaceDepth` 同公式（surface.h L377） | ✅ |
| stoneDepth | `i <= 1 + offset + j + k`，`i=stoneDepthAbove/Below`，`j=addSurfaceDepth?runDepth:0`，`k=map(secondaryDepth,-1,1,0,range)` | 同构，**但 k 用 `lerpClamp` + `floor`（见偏差 P1）** | ⚠️ P1 |
| water(offset,mult,addStoneDepth) | `fluidHeight==MIN \|\| blockY+stoneDepthAbove >= fluidHeight+offset+runDepth*mult` | 同构（surface.h L223） | ✅ |
| noiseThreshold | `sample(blockX, 0.0, blockZ)`（y 恒 0，列缓存） | `sample(ctx.blockX, 0.0, ctx.blockZ)`（列缓存） | ✅ |
| above_preliminary_surface（surface()） | `blockY >= floor(lerp2(4 角 est)) + runDepth - 8` | `blockY >= k + surfaceDepth - 8`（surface.h L261） | ✅ |
| hole() | **`context.runDepth <= 0`（sampleRunDepth！）** | **`ctx.stoneDepthAbove <= 0`（扫描计数）** | ❌ **P2** |
| biome / steep / temperature | 同 | 同（steep 读 heightmap ±4） | ✅ |

### 2.4 beach sand/sandstone 链（对应任务 A2）

Java mr9 海底 mc10 段（water(-6,-1)）：
```java
condition(mc10 /* waterWithStoneDepth(-6,-1) */, sequence(
    condition(STONE_DEPTH_FLOOR, condition(mc12, condition(mc11, WATER))),                     // frozen ocean hole
    condition(STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH, mr7),                                       // 水下 6 格内陆地风格表面
    condition(mc14, condition(STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH_RANGE_6, SANDSTONE)),        // warm_ocean/beach/snowy_beach
    condition(mc15, condition(STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH_RANGE_30, SANDSTONE))))      // desert
```
C++ surface.h L632-638 同构。**-288 有 beach（mc14）→ RANGE_6 sandstone 路径在场**（desert 不在场）。

### 2.5 海底"系统性偏低 2-6 格"的判定（对应任务 A3/B2）

- **结论（强证据）**：`summary-final.md` 已用 AQF-APPLY + chunk-status 铁证证明 **C++ density/aquifer 与 Java 逐位一致**（aquifer 判 solid 一致；含水层 water=carvers 产物非 aquifer）。因此 **"海底 stone 顶偏低"不来自 density/aquifer 层**。
- 残余 water↔stone 双向差异（got=water/vanilla=stone 3117 + got=stone/vanilla=water 4416 等）主体是 **carvers（已闭合）与 surface 染色的交互**：vanilla carver 挖洞后改变了列内 air/water 分布 → buildSurface 的 q/r/s/fluidHeight 不同 → surface 规则输出不同（见候选 C）。
- phase2「C++ 海底系统性偏低 2-6 格」为早期观察，在 AQF-APPLY 铁证后应**修正为 surface 染色 / carver 交互**，非海底高度本体的系统性偏移。

---

## 3. C++ 覆盖度与偏差点清单（对应任务 B）

### 3.1 C++ 已实现（覆盖度 ✅）
- MaterialRules 引擎全部条件：Biome/AboveY/Water/StoneDepth/NoiseThreshold/Hole/Steep/Surface/Temp/VerticalGradient/Not
- buildSurface 循环（q/r/s/vx 逐块对齐）、sampleRunDepth、getSecondaryDepth、estimateSurfaceHeight（4 角 lerp2 + runDepth-8）
- 规则树 mr/mr2..mr9 与 Java 逐条同构；噪声参数表与 noise_params.json 一致（gravel -8 {1,1,1,1}；surface -6 {1,1,1}；surface_secondary -6 {1,1,0,1}）
- 随机派生链一致：Java `randomDeriver = provider.create(seed).nextSplitter()`；C++ `builder->randomDeriverPublic()` 同源。sampler 派生：Java `randomDeriver.split(key)` → sampler；C++ `splitter.split("minecraft:"+k)` → sampler；octave：Java `random.nextSplitter().split("octave_"+l)`；C++ 同。✅
- 海底规则 mr3/mr2 条件参数与顺序逐条一致

### 3.2 偏差点清单（按 -288 区域影响排序）

| # | 偏差点 | Java | C++ | 影响区域 | 严重度 |
|---|---|---|---|---|---|
| **P1** | StoneDepthCond 的 secondaryDepth 映射 | `(int)MathHelper.map(sec, -1, 1, 0, range)` — **不 clamp**，`(int)` 截断 | `(int)std::floor(lerpClamp(sec, -1,1, 0, range))` — **clamp [0,1]** + floor | **beach RANGE_6 sandstone（mc14）**、desert RANGE_30、warm ocean | **高**（beach 在 -288 在场） |
| **P2** | HoleCond 用错字段 | `hole() = runDepth <= 0`（sampleRunDepth 2D 噪声值） | `ctx.stoneDepthAbove <= 0`（垂直扫描计数 q） | frozen ocean hole、badlands 段（**-288 无 frozen/badlands → 当前不贡献，但真实 bug**） | 中-高（真实 bug） |
| **P3** | buildSurface 的 s 未找到默认值 | 循环未找到非 default → `s = -32512`（field_35479）→ vx 巨大正 | 未找到 → `s = INT32_MAX` → vx 巨大负 | 仅"u 到世界底全 default 无 air/fluid"列（海底有 lava 一般不会触发） | 低-中 |
| **P4** | isFluid 判定 | `!state.getFluidState().isEmpty()`（含水方块算流体） | `state==water \|\| state==lava` | 含水方块（overworld 海底少见） | 低 |
| P5 | estimateSurfaceHeight fallback 无 4 对齐 | `BiomeCoords.toBlock(fromBlock())`（4 格对齐） | fallback 单列扫描未对齐（正常走 surfaceHeights4 已对齐） | 不触发（surfaceHeights4 恒提供） | 低 |
| P6 | 深度语义命名混乱 | MaterialRuleContext.runDepth = sampleRunDepth；stoneDepthAbove = 扫描 q | SurfaceContext.runDepth = 扫描 q；surfaceDepth = sampleRunDepth | 仅 HoleCond 用错（P2）；其余正确 | 记录 |

---

## 4. 候选机制清单（对应任务 C，按优先级）

### 候选 1：P1 — secondaryDepth 映射 clamp/截断差异 → beach 砂岩层（sand/sandstone 互换核心）
- **机制**：`STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH_RANGE_6` 的 k = `map(surface_secondary,-1,1,0,6)`。Java map 不 clamp（surface_secondary 超出 [-1,1] 时映射可出 [0,6]），`(int)` 向零截断；C++ lerpClamp 钳制 + floor 向负无穷。surface_secondary = -6 {1,1,0,1} 噪声，值域常超 [-1,1]，**在 beach 海底/沿岸列会改变 sandstone 层厚度边界** → sand↔sandstone 互换（pair 统计 sand→sandstone 427、sandstone→sand 134、sandstone→stone 638、sand→stone 273、water→sand 723 等 ≈1500-2000 块）。
- **证据支持度**：代码级确认差异真实；-288 含 beach（mc14 在场）；phase2 已列 sand/sandstone 互换。
- **验证方法**：
  1. 对 chunk(-15,-14)/(-15,-13)/(-16,-14) 的 beach 列，用 C++ probe 输出 `getSecondaryDepth` 与 Java 反射对比，找出 `sec > 1 || sec < -1` 的列；
  2. 对这些列比较 C++ 当前规则输出 vs Java map 版本输出（模拟 `unclamped map + trunc`），统计 block 级一致率提升；
  3. 修复后重跑 block_probe 对 -288 的 sand/sandstone 类 pair 计数应显著下降。
- **修复方向**：C++ `StoneDepthCond::test` 的 k 改 `(int)unclampedMap(sec, -1, 1, 0, range)`（不 clamp；截断语义对齐 Java `(int)`，注意负值向零 vs floor 差异）。

### 候选 2：P2 — HoleCond 字段错（真实 bug，-288 暂不贡献）
- **机制**：Java hole() 判定 `runDepth<=0`（surface 2D 噪声派生的列值），C++ 误用 `stoneDepthAbove<=0`（垂直扫描计数）。frozen_ocean 坑洞填水（AIR/ICE/WATER 链）与 badlands 段受影响。
- **证据支持度**：代码级铁证（Java MaterialRules L536 `NegativeRunDepthPredicate` vs C++ surface.h L249）。
- **验证方法**：任意 frozen_ocean/badlands 区域（seed 采样 1 个含 frozen_ocean 的 chunk）对比 hole 触发列；或单列断言：选 surface noise 很负（runDepth 可能 ≤0）的列，检查 C++ hole 结果与 Java 是否一致。
- **修复方向**：`HoleCond::test` 改 `ctx.surfaceDepth <= 0`。
- ⚠️ 影响面：修复会改变 frozen ocean 表面/坑洞；**需在改动前确认 -288 无 frozen ocean（已确认 biome 无 frozen_ocean）**，避免误改主场景。

### 候选 3：carver 交互 — gravel↔stone 双向残余（海底 gravel ≈4900 的核心候选）
- **机制**：mr3 的 STONE_DEPTH_CEILING 判定依赖 `stoneDepthBelow = u - s + 1`，s 由列内向下找第一个非 default(air/fluid) 确定。vanilla carver 在列中挖洞/填水 → s 更浅 → vx 更小 → 更可能判定"表层 1 格 STONE"；C++ 未实现 carvers → 列内无洞 → s 更深 → vx 更大 → 更多 GRAVEL。**这正是 phase2「C++ 海底 gravel 斑块与 vanilla 不一致」的可能根因之一**，且与已闭合 carvers 纠缠（deepslate→gravel 1802 已确认 ore_gravel FEATURE，海底 gravel↔stone 2135+746 可能部分来自此交互）。
- **证据支持度**：机制推演强；carvers 已证实是 -288 洞穴/含水层差异主体；但 surface-carver 交互未被直接量化。
- **验证方法**：
  1. 取含 carver 洞的 vanilla 列（如 dump_x-278_z-240 已知含洞），在 C++ 中**先模拟 carver 挖洞再跑 buildSurface**，对比 gravel↔stone 输出；
  2. 或反推：对 got=gravel/vanilla=stone 的样本列，检查 vanilla 列该处是否 carver 洞边缘（有洞 → s 浅 → STONE），C++ 无洞 → GRAVEL。
- **性质**：此候选若成立，是"surface 正确 + 前置 carver 缺失"的传导差异，属于**范围外 FEATURE（carvers）已决策待实现**的一部分，或需与候选 1 合并评估。

### 候选 4：waterlogged/流体判定（P4）— 低
- 含水方块在 Java 算流体（不染色、计入 fluidHeight），C++ 只认 water/lava。overworld 海底/beach 区域含水方块少见 → 影响小。列为低优先级记录。

### 候选 5：deepslate gradient 边界（垂直梯度 bedrock/deepslate）
- 规则树一致（C++ surface.h L650-654 同构）；海底/beach 不涉及 deepslate（y>0）。无需动作。记录排除。

---

## 5. 影响架构的变化（交主会话裁决）

1. **P1 修复（surface.h StoneDepthCond::test k 计算）**：建议修复，属于 C++ surface 层单点改动，预计可闭合 sand/sandstone 互换主体（≈2900 块中相当部分）。需先用验证方法 1 量化收益再改。
2. **P2 修复（surface.h HoleCond::test）**：真实 bug 但 -288 无触发场景。**建议先记录、不立即修复**（避免在无验证数据下改动 frozen ocean/badlands 行为）；或作为独立小任务在验证后修复。
3. **carver 交互（候选 3）**：与「carvers + 岩石替换实现」主计划（summary-final 待办 1）重叠；建议在实现 carvers 后重跑 block_probe，用 gravel↔stone 计数验证是否自动闭合，**不单独改 surface**。
4. **P3/P4/P5**：低风险，随 P1 修复一并核对即可；无架构影响。

---

## 6. 待深入点（后续勘探/定位输入）

- [ ] P1 收益量化：beach 列 `getSecondaryDepth` 分布 + map vs lerpClamp 差异块统计（需 C++ probe 输出）
- [ ] P2 确认：frozen_ocean 区域样本（若未来课题涉及）
- [ ] carver 交互量化：模拟 carver 后 buildSurface 的 gravel↔stone 变化（属 carvers 实现阶段）
- [ ] 海底水↔sand 边界（water→sand 723）归因：beach RANGE_6 sandstone 链（候选 1）vs 结构与 carver（已闭合）
- [ ] 噪声 seed 链复核（gravel -8 {1,1,1,1} 已与 noise_params.json 一致，无需动作）

---

*勘探角色：recode.scout · 只读 · 产物仅 .investigations/ · 不修改目标代码 · 2026 勘探记录*
