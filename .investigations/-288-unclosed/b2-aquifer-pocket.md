# B2 —「Java Aquifer 存在独立 pocket 形状场导致含水层判定≠density>0→solid」假设验证

> 角色：core.worker（分析，只读文件——沙箱无 shell）
> 任务：验证候选假设 B2：「Java Aquifer 内部存在独立于 finalDensity 的 pocket 形状噪声/液面逻辑，使 aquifer 判定 ≠ density>0→solid 的简单映射——C++ aquifer.h 未实现该形状场，导致水/实心判定差」
> 数据源：`versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java`、`ChunkNoiseSampler.java`、`NoiseChunkGenerator.java`、`carver/CaveCarver.java`、`carver/CarverContext.java`；`versions/1.20.1/cpp/worldgen/src/aquifer.h`；`.investigations/-288-reopen/`（analysis-phase8.md、summary-final.md、cmd-output/aqfapply_run.txt、noiseblk_run2.txt、dump_x-278_z-240.txt）
> 日期：2026-08（-288 未闭合差异定位轮）
> 状态：draft

---

## 0. 一页摘要（TL;DR）

| # | 结论 | 置信度 |
|---|---|---|
| 1 | **Java `AquiferSampler.Impl.apply` 的第一铁律是 `density > 0 → null(solid)`（L149-151）**。pocket/液面形状逻辑（floodedness/spread/barrier）只存在于 **density≤0 分支**（L153 之后），用于细分水/空气/实心，**不能推翻 density>0→solid** | 确定（源码） |
| 2 | **C++ aquifer.h 完整实现了全部 pocket/液面场**：fluidFloodedness/fluidSpread/fluidType/barrierNoise 字段（L167）+ getFluidBlockY floodedness 逻辑（L341-346）+ getNoiseBasedFluidLevel spread 逻辑（L355-366）+ calculateDensity barrier 逻辑（L260-273）——与 Java 一一对应。B2 核心指控「C++ 未实现形状场」**不成立** | 确定（源码对照） |
| 3 | **「AQF-APPLY 判 solid」vs「NOISE-BLK 判 water」矛盾澄清**：AQF-APPLY 直接调 `aquifer.apply`（NOISE 阶段单块判定）→ (-278,12..23,-240) 全判 solid；NOISE-BLK 读的是 **carvers 阶段之后**的方块状态 → y=15-19 water / y=23 air。两者**不冲突**：aquifer 判 solid，water/air 是 CaveCarver 雕刻 + 液面填水的产物（chunk status=carvers 铁证） | 确定（探针语义 + status） |
| 4 | **B2 无法解释任何差异**：4416 深层含水层 = carvers 产物（已闭合）；6710 海底边界（C++ water vs vanilla solid）方向**相反**，更不是 aquifer pocket 缺失可解释 | 确定（方向 + 阶段） |
| 5 | **修复方向**：aquifer.h **无需修改**；若要闭合「地形性 carvers」需实现 CaveCarver（雕刻 + 液面填水），其填水依赖 `AquiferSampler.getFluidLevel`——C++ 已具备该能力 | 确定（机制） |

---

## 1. Java `AquiferSampler.Impl.apply` 完整判定逻辑（带行号）【确定】

文件：`AquiferSampler.java`，行号以 mc_src_extract 为准。

### 1.1 apply 入口（L143-251）

```
L143-145  public BlockState apply(NoisePos pos, double density)
L149-151  if (density > 0.0) { needsFluidTick=false; return null; }   ← 第一铁律：density>0 → 实心（null = 默认方块）
L153-157  density≤0 分支：取 fluidLevelSampler.getFluidLevel(i,j,k)；若该液面是 LAVA → 判 LAVA
L158-207  网格 pocket 采样：l=floorDiv(i-5,16) / m=floorDiv(j+1,12) / n=floorDiv(k-5,16)
          18 邻居（u∈{0,1}, v∈{-1,0,1}, w∈{0,1}）经 randomDeriver.split(x,y,z) + nextInt(10/9/10)
          得随机 pocket 点；维护 3 最近距离 o/p/q（平方距离）
L209-214  fluidLevel2 = getWaterLevel(r)（最近 pocket 液面）；d = maxDistance(o,p)；
          bs = fluidLevel2.getBlockState(j)（液面下 → 水）；d ≤ 0 → 判 bs（水）
L215-217  若 bs 是水且下方邻居液面是 LAVA → 判水
L219-221  e = d * calculateDensity(pos, md, fluidLevel2, fluidLevel3)   ← barrierNoise + 液面间形状
L222-225  if (density + e > 0.0) → null(solid)                          ← 形状场可把 density≤0 抬成实心
L226-243  第二/第三近邻形状修正 f/g（d*g / d*f*g / d*g*h），density+g>0 / density+h>0 → null(solid)
L245-247  否则判 bs（水）
```

**结论**：aquifer 判定 = `density>0 → solid`（硬顶）；`density≤0` 时 pocket 形状场决定水/空气/实心细分。**不存在「density>0 仍判水」的路径**。

### 1.2 形状场来源（L83-86 构造 + L353-450 使用）

| 场 | Java 字段/构造 | 使用位置 | 作用 |
|---|---|---|---|
| barrierNoise | L83 `noiseRouter.barrierNoise()` | L306 calculateDensity | 含水层间岩石屏障噪声 |
| fluidLevelFloodednessNoise | L84 | L402 getFluidBlockY | 液面 floodedness（决定液面是否高于默认水位） |
| fluidLevelSpreadNoise | L85 | L429 getNoiseBasedFluidLevel | 液面散布噪声（决定噪声液面高度） |
| fluidTypeNoise（lavaNoise） | L86 | L443 getFluidBlockState | 深层液面 lava 判定 |
| erosionDensityFunction / depthDensityFunction | L91-92 | L395 method_43718 | 深海区液面覆盖（erosion<-0.225 && depth>0.9 → d=e=-1） |

- `getFluidBlockY`（L391-419）：`d = g - k` / `e = g - h`（floodedness g、surface 距离 f 的映射），e>0→默认液面、d>0→噪声液面、否则无效层（-32512）→ AIR。
- `getNoiseBasedFluidLevel`（L421-433）：spread 噪声 ×10 → roundDownToMultiple(3) → 与 surfaceHeightEstimate 取 min。
- `calculateDensity`（L263-321）：液面间线性形状 q（±2 内）+ barrierNoise，`2*(r+q)`。

→ **Java 确实存在「独立于 finalDensity 的形状/液面场」**——但只作用于 density≤0 分支。

## 2. C++ aquifer.h 对照——形状场已实现【确定】

文件：`cpp/worldgen/src/aquifer.h`

| Java | C++ | 一致 |
|---|---|---|
| apply L149 `density>0→null` | L74 `if (density>0) return -1` | ✅ |
| apply L153-247 全部 pocket/形状逻辑 | L76-139 逐条对应（邻居 18、o/p/q、d/e/f/g/h 全部） | ✅ |
| calculateDensity L263-321 | L238-276（barrier 缓存 MutableDouble、q 分支、2*(r+q)） | ✅ |
| getFluidLevel L353-389（13 邻居 CHUNK_POS_OFFSETS） | L290-319（OFFSETS[13][2] 同序） | ✅ |
| getFluidBlockY L391-419（floodedness d/e、method_43718） | L329-353（`erosion<-0.225f && depth>0.9f`、clamp/map2 同式） | ✅ |
| getNoiseBasedFluidLevel L421-433（spread×10 → roundDown3 → min(surface)） | L355-366 | ✅ |
| getFluidBlockState L435-450（fluidTypeNoise |d|>0.3 → LAVA） | L368-381 | ✅ |
| estimateSurfaceHeight（ChunkNoiseSampler 列缓存） | L145-164（flat 数组 + initialDensity>0.390625） | ✅ |

- C++ 头部注释（L1-2）「412 行 Java 翻译」；成员字段 L167 含全部 4 个形状 DF + erosion/depth。
- anchor.test 标注（L68/L219/L237/L328）记录了已做过的对齐验证（apply、getBlockPos、calculateDensity、getFluidBlockY）。

→ **B2 的「C++ aquifer.h 未实现该形状场」指控在源码层面不成立**。

## 3. 「AQF-APPLY 判 solid」vs「NOISE-BLK 判 water」矛盾澄清【确定】

### 3.1 两个探针语义不同

| 探针 | 测什么 | (-278,12..23,-240) 结果 | 来源 |
|---|---|---|---|
| **AQF-APPLY** | 直接调 `AquiferSampler.Impl.apply(pos, density)`（NOISE 阶段单块 aquifer 判定） | y=12..23 **全部 density>0（0.055724~0.068693）→ null(solid)** | aqfapply_run.txt L1090-1124 |
| **NOISE-BLK** | Java BlockProbe 读 chunk 内**已写入的方块**（populateNoise + carve + surface 之后的状态） | y=15-19 **water**、y=20-22 stone、y=23 **air**、y=24-28 stone | noiseblk_run2.txt L286-301 |

### 3.2 阶段定位（Java 流程）

- `NoiseChunkGenerator.fillFromNoise`（L359-430）：逐 cell `sampleBlockState()` → `blockStateSampler.sample` = **ChainedBlockSource（aquifer.apply + OreVein）**（ChunkNoiseSampler L176-186）；`aquifer.apply` 输入 = `cacheAllInCell(finalDensity + Beardifier)`（L177-180）。**此阶段 aquifer 判 solid**。
- `carve(...)`（NoiseChunkGenerator L279-327）在 **populateNoise 之后**独立运行（GenerationStep.Carver）：`configuredCarver.carve(carverContext, chunk, ..., aquiferSampler, chunkPos2, carvingMask)`（L320）——把 solid 挖成空并**用 aquifer 液面填水**（CaveCarver L24-67 传 aquiferSampler 到 carveRegion）。
- **chunk status 铁证**：summary-final 记录 chunk(-18,-15) status=`minecraft:carvers`；NOISE-BLK 探针读到的是该 status 后的状态。

### 3.3 结论

- **AQF-APPLY 是 aquifer 的真实判定**（NOISE 阶段），判 solid，与 C++ 逐位一致。
- **NOISE-BLK 读到的是 carvers 之后的方块**（洞穴雕刻 + 液面填水），不是 aquifer 输出。
- 矛盾是**阶段差异**，不是 aquifer 语义差异。phase8「核心矛盾」（density>0 却判 water）由此闭合：water 不是 aquifer 判的，是 **CaveCarver** 挖洞 + `AquiferSampler.getFluidLevel`（液面 y=63 默认水）填水判的。

## 4. B2 假设判定【确定】

| 假设环节 | 裁定 | 依据 |
|---|---|---|
| (a) Java Aquifer 存在独立于 finalDensity 的 pocket 形状/液面逻辑 | **成立但仅限 density≤0 分支** | L153-247；floodedness/spread/barrier 场（L83-86） |
| (b) 该逻辑使 aquifer 判定 ≠ density>0→solid | **不成立**——density>0 恒判 solid（L149）是硬铁律 | AQF-APPLY y=12..23 实测全 solid |
| (c) C++ aquifer.h 未实现该形状场 | **不成立**——全部字段与逻辑一一对应已实现 | §2 对照表 |
| (d) 该差异能解释深层含水层 4416 块 | **不能**——4416 块 = carvers 阶段产物（NOISE-BLK water/air 夹层 + status 铁证 + AQF-APPLY solid 三方互证），aquifer 两语言一致判 solid | §3 |
| (e) 该差异能解释海底边界 6710 块（C++ water vs vanilla solid） | **不能**——方向相反：若 aquifer pocket 缺失导致 C++ 判水，则 AQF-APPLY 应显示 C++/Java 判定不同，但实测逐位一致；且 6710 块是 C++ 多水而 vanilla 实心，机制应在 surface/结构侧（summary-final 候选：surface 海底 gravel/砂染色、结构岛、C++ surface 微小项） | summary-final §2 + AQF-APPLY |

**B2 总判定：推翻（作为含水层差异归因）。** 正确认知内核保留：Java Aquifer 确实有形状/液面场（pocket），但它是 density≤0 分支的细分器，不是「density>0 判水」的来源，也不是 C++ 缺失项。

## 5. 可解释块数

| 差异类别 | 数量 | B2 可解释 | 实际机制 |
|---|---|---|---|
| 深层含水层 stone→water | 4416 | **0** | CaveCarver 雕刻 + 液面填水（FEATURE，已闭合） |
| deepslate→water | 635 | **0** | 同上 |
| 海底边界 water↔solid | ~6710 | **0**（方向相反） | 未完全定位（surface/结构候选） |

→ B2 修复收益 **0 块**；aquifer.h 无 bug，无需修改。

## 6. 范围内待修建议【机制层】

1. **不做**：修改 `aquifer.h` 判定（任何「pocket 覆盖 density>0 判水」的改动都会破坏逐位对齐——AQF-APPLY 铁证反证）。
2. **可做（若实施「地形性 carvers」FEATURE）**：实现 **CaveCarver**（configured_carver 配置 + carving 算法 + `AquiferSampler.getFluidLevel` 液面填水）。C++ `Aquifer.getFluidLevel`（L290-319）已具备 carver 填水所需液面查询能力——该能力正是 Java carve() 阶段传给 carver 的 `aquiferSampler`（NoiseChunkGenerator L320）所暴露的。
3. **海底边界 6710 块**：转 surface 侧定位（海底 gravel/砂染色、C++ surface 微小项），与 B2 无关（summary-final 待办 3）。
4. **诊断**：若需进一步证实 carver 填水，可加探针在 Java carve 后读 carvingMask + 液面填水点（对照 NOISE-BLK water 集合）。

---

## 7. 置信度与边界

- 【确定】：源码行号（AquiferSampler L149/L153-247、ChunkNoiseSampler L176-186、NoiseChunkGenerator L279-327）；AQF-APPLY 实测（aqfapply_run.txt L1090-1124）；NOISE-BLK 实测（noiseblk_run2.txt L286-301）；C++ dump（dump_x-278_z-240.txt L1175-1179）；chunk status 铁证（summary-final 引）。
- 【推测】：海底边界 6710 块的最终机制（未在本轮定位，引用 summary-final 候选）。
- 本报告只做解读，不修改代码；未运行任何命令（沙箱无 shell，铁律遵守）。
- retry 轮次：0（本轮首次验证 B2；此前 phase8 曾推测 pocket 机制——本轮以源码 + AQF-APPLY 铁证判定其不能解释差异）。
- 自检：产物仅 draft；未自我审查（审查走 core.judge）；未跑命令（无 shell）。
