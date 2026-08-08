# Phase 9 — Aquifer.apply 决策链逐行对比：C++ vs Java（-288 判水/判 solid 机制定位）

> 角色：recode.scout（勘探，只读）
> 任务：逐行对比 C++ 与 Java 的 Aquifer.apply 决策链，定位「Java 在 density>0 时仍判 water、C++ 判 solid」的具体机制差异
> 种子：seed=-8248318472910187742；双列锚点 (-278,-240)（Java water y=15-19 / C++ solid）与 (-244,-256)（Java solid 岛 y=58-61 / C++ water）
> 数据源：Java `AquiferSampler.java`、`ChunkNoiseSampler.java`、`NoiseChunkGenerator.java`、`StructureWeightSampler.java`；C++ `aquifer.h`、`worldgen_api.cpp`、`density.h`；cmd-output 全套（noiseblk_run2/noiseblk_blockprobe/aqfj_blockprobe/trace_aqf_1/dump_x-278_z-240/vanilla_density_*）；phase6/7/8 产物
> 日期：phase8 之后
> 置信度约定：【确定】= 源码逐行 / 数据直接可证；【推测】= 机制归因 / 数量估算；【需补数据】= 现有探针无法证实，必须补 Java 中间量 dump

---

## 0. 一页摘要（TL;DR）

| # | 结论 | 置信度 |
|---|---|---|
| 1 | **C++ `Aquifer::apply` 决策链与 Java `AquiferSampler.Impl.apply` 逐行一致**：density>0→null 分支、3×3×3 blob 遍历、d≤0 分支、d+e 判据、e=calculateDensity、fl.y 液面、getFluidLevel(13 邻居)/getFluidBlockY/getNoiseBasedFluidLevel/getFluidBlockState 全部对齐，**无逻辑差异** | 确定（逐行） |
| 2 | **两列差异的公共根因 = aquifer 收到的 density 数值不同，不是决策逻辑不同**。Java aquifer 输入 = `CellCache(Add(插值 finalDensity, StructureWeightSampler))`（ChunkNoiseSampler L177-181），C++ 输入 = `finalDensity->sample`（worldgen_api L619，**无 Beardifier**） | 确定 |
| 3 | **(-244,-256) 岛（Java solid / C++ water）机制已定位【确定】**:Java 插值 finalDensity(58)=-0.0744（负）→ 判 water；Java 真实判 solid → 必须有额外正贡献 `StructureWeightSampler(58)≈+0.08~+0.11`（村庄/沉船 piece 下方 12 格内，j<piece底 → **正贡献**，StructureWeightSampler L101-104 + L140-149）。C++ 缺该项 → density 保持 -0.0744 → 判 water。**分叉点 = apply 入口 `density>0` 判定；根因 = Beardifier 缺失**（与 phase6 一致） | 确定（机制）+ 推测（贡献量级） |
| 4 | **(-278,-240) 含水层（Java water / C++ solid）机制未完全定位【需补数据】**：Java 判 water → Java CellCache 输入在 y=15..19 必 ≤0；但 Java 无插值直采 finalDensity 与 C++ 插值同点全正（+0.054~+0.077）。距结构 >12 格（Beardifier 有效半宽 = 12，StructureWeightSampler INDEX_OFFSET=12）→ Beardifier≈0 无法解释。**候选 = C++ 树/插值在 (-278,-240) 与 Java 存在局部差异**（如 jaggedness 恒 0），或存在未发现结构。**必须补 Java `[CellCache]` 真实值 dump** | 确定（数据矛盾）/ 推测（机制） |
| 5 | **任务描述勘误**：`vanilla_density_overworld_c-18_-15_b10_0.txt` 是 **Java 无插值直采** finalDensity（DensityProbe L64-69，`UnblendedNoisePos` 每 4 格），非「router.finalDensity().sample 插值」；AQF-J 的 densFn 值被项目文档裁定 **CellCache 缓存污染不可信**（docs/09-multi-dimension.md L740）——不能当作 aquifer 输入 | 确定 |
| 6 | **修复方向**：①实现/注入 StructureWeightSampler（Beardifier）并接入 worldgen_api L619 → 修复 (-244,-256) 岛及全部结构 12 格内正贡献类差异；②(-278,-240) 含水层需先补 Java CellCache dump 定位 C++ 树/插值差异，再做针对性修复 | 确定（方向）/ 推测（工程量） |

---

## 1. 数据源盘点与勘误【确定】

### 1.1 数据源语义（DensityProbe.java 逐行）

| 文件 | 语义 | 行号 |
|---|---|---|
| `vanilla_density_*.txt`（无后缀） | **Java 无插值直采** finalDensity（`df.sample(new UnblendedNoisePos(wx,y,wz))`，y 每 4） | DensityProbe L64-69 |
| `*_cns.txt` | Java 游戏实际路径：8 个 DensityInterpolator 的插值 sample（y 每块 + 8 列） | DensityProbe L116-164 |
| AQF-J 的 densFn / `[CellCache]` | CellCache.sample 反射值——**污染不可信**（项目文档明示 L740；phase5 裁定；值在 3 次运行间漂移） | docs/09-multi-dimension.md L740 |
| C++ `dump_x-278_z-240.txt` | C++ `finalDensity->sample`（InterpolatedDF 插值）每 4 格 + 分量 | worldgen_api L663-688 |
| C++ `trace_aqf_1.txt` | C++ `Aquifer::apply` 逐块 trace（density/o/p/q/d/e） | aquifer.h L106-122 |

### 1.2 关键数值对照（双列）

**(-244,-256) 列**（Java 岛 y=58-61 stone / C++ trace 全判 water）：

| y | Java 无插值直采 | C++ 插值 trace | Java NOISE 块 | C++ aquifer 判定 |
|---|---|---|---|---|
| 56 | -0.053463 | -0.053461 | water | FLUID |
| 57 | （无） | -0.063950 | water | FLUID |
| 58 | （无） | **-0.074424** | **stone** | **FLUID（错）** |
| 59 | （无） | -0.084882 | stone | FLUID（错） |
| 60 | -0.100940 | -0.095322 | stone | FLUID（错） |
| 61 | （无） | -0.105740 | stone | FLUID（错） |

→ C++ 插值 finalDensity(58) = -0.0744 与 Java（phase6 验算 squeeze(0.64×-0.233015)=-0.0744）一致；**Java 判 solid 只能来自额外正贡献（Beardifier）**。【确定】

**(-278,-240) 列**（Java water y=15-19 / C++ solid）：

| y | Java 无插值直采 | C++ 插值 SURF | Java NOISE 块 | C++ aquifer 判定 |
|---|---|---|---|---|
| 12 | +0.054475 | +0.055723 | stone | SOLID |
| 15 | （无） | （插值≈正） | **water** | SOLID |
| 16 | +0.068530 | +0.068692 | **water** | SOLID |
| 19 | （无） | （插值≈正） | **water** | SOLID |
| 20 | +0.076488 | +0.068530 | stone | SOLID |
| 23 | （无） | （插值≈正） | **air** | SOLID |
| 24 | +0.068347 | +0.068367 | stone | SOLID |

→ Java 在 y=15..19 判 water、y=23 判 air ⇒ **Java aquifer 输入在这些 y ≤ 0**；C++ 同点插值全正 ⇒ 判 solid。**Java 与 C++ 的 density 输入数值矛盾（相差 ≥0.06），且方向与 (-244,-256) 相反**。【确定（数据）/ 机制未定】

---

## 2. 对比 1：apply 入口分支【确定·逐行一致】

| 环节 | Java（AquiferSampler L143-152） | C++（aquifer.h L70-71） | 判定 |
|---|---|---|---|
| density>0 → null（默认方块） | `if (density > 0.0) { return null; }` | `if (density > 0.0) return -1;` | **一致** |
| density≤0 进入流体决策 | else 分支 | 继续执行 | **一致** |
| **density>0 的「提前返回 null」分支存在且条件相同** | ✓ | ✓ | **一致** |

→ **「density>0 → solid（null）」简单映射在 Java 与 C++ 都存在且一致**。Java 判 water 的唯一路径是 density ≤ 0 进入流体决策。**因此任务描述的「Java 在 density>0 时仍判 water」不成立——Java 判 water 时其 density 输入必 ≤ 0，只是该输入 ≠ 探针/无插值直采/ C++ 插值（三者都正）**。【确定】

---

## 3. 对比 2：computeSubstance 流体决策【确定·逐行一致】

### 3.1 主决策链（Java L153-251 ↔ C++ L73-136）

| 环节 | Java | C++ | 判定 |
|---|---|---|---|
| 液面基准 | `fluidLevelSampler.getFluidLevel(i,j,k)`（NoiseChunkGenerator L78-84：y<min(-54,63) LAVA else WATER@63） | `defaultFluidLevel`（y<-54 lava else water@63） | **一致**（主世界 sea=63） |
| lava 早退 | `getBlockState(j).isOf(LAVA) → return LAVA` | `fluidBlock==lavaId → return` | **一致** |
| blob 网格原点 | `floorDiv(i-5,16), floorDiv(j+1,12), floorDiv(k-5,16)` | 同 | **一致** |
| 3×3×3 最近 3 blob（o/p/q, r/s/t） | 三重循环 + 三档插入 | 同 | **一致** |
| `d = maxDistance(o,p) = 1-\|o-p\|/25` | L210 | L229-232 | **一致** |
| `blockState = fluidLevel2.getBlockState(j)`（最近 blob 液面） | L211 | L105 | **一致** |
| `d ≤ 0 → return blockState` | L212-214 | L110 | **一致** |
| water 且下方 lava → return water | L215-217 | L111-114 | **一致** |
| `e = d * calculateDensity(fl2, fl3)`；`density+e > 0 → null` | L219-224 | L116-123 | **一致** |
| `f = maxDistance(o,q)`；`g = d*f*calc(fl2,fl4)`；`density+g>0 → null` | L226-234 | L125-130 | **一致** |
| `g2 = maxDistance(p,q)`；`h = d*g2*calc(fl3,fl4)`；`density+h>0 → null` | L236-243 | L131-135 | **一致** |
| 兜底 return blockState | L246 | L136 | **一致** |

### 3.2 calculateDensity（Java L263-321 ↔ C++ L235-273）【确定·逐行一致】

- `lavaWater` 判断（两液面一水一岩）→ return 2.0 ✓
- `j = abs(fl.y - fl2.y)`；`j==0 → 0.0` ✓
- `d = 0.5*(y1+y2)`；`e = blockY+0.5-d`；`f = j/2`；`o = f - |e|` ✓
- e>0：`p=0+o; q = p>0 ? p/1.5 : p/2.5`；e≤0：`p=3+o; q = p>0 ? p/3 : p/10` ✓
- `q∈[-2,2]` 时取 `barrierNoise`（MutableDouble 缓存），否则 0；`return 2*(r+q)` ✓
- 唯一形式差异：Java 用 `pos`（ChunkNoiseSampler.this，坐标 = blockX/Y/Z）采 barrierNoise，C++ 用 `NoisePos{blockX,blockY,blockZ}`——坐标一致 ✓

### 3.3 getFluidLevel / 液面（Java L353-450 ↔ C++ L287-391）【确定·一致】

- 13 邻居偏移表完全一致（CHUNK_POS_OFFSETS）✓
- `estimateSurfaceHeight`：Java 用 `initialDensityWithoutJaggedness`（ChunkNoiseSampler L234），C++ 用 `R["initial_density"]`（worldgen_api L376 注册的是 `initial_density_without_jaggedness`）→ **一致** ✓
- `getFluidBlockY`：`erosion < -0.225f && depth > 0.9f → d=e=-1`；否则 `f = bl?lerpClamp(i,0,64,1,0):0`、`g = clamp(floodedness)`、`h/k = map(f,1,0,-0.3/0.8 / -0.8/0.4)`、`d=g-k, e=g-h`；`e>0→default.y; d>0→noiseBased; else -32512` ✓
- `getNoiseBasedFluidLevel`：`floorDiv(x/16,y/40,z/16)`、`l*40+20`、`spread*10`、`roundDownToMultiple(3)`、`min(est,q)` ✓
- `getFluidBlockState`：`fluidLevel<=-10 && !=-32512 && state!=lava → fluidType abs>0.3 → lava` ✓

**结论：C++ aquifer 内部决策链与 Java 逐行一致，无逻辑差异【确定】。差异完全在输入 density 数值。**

---

## 4. 对比 3：density 输入语义【核心差异】

### 4.1 Java 侧（ChunkNoiseSampler L176-181 + CellCache L652-701）

```
builder.add(pos -> this.aquiferSampler.apply(pos, densityFunction.sample(pos)));
densityFunction = CellCache( Add( InterpolatedDF(finalDensity), Beardifier ) ).apply(getActualDensityFunction)
```

- CellCache 在 `onSampledCellCorners` 对 cell 内 4×4×8=128 个块位置填充
- 填充值 = `InterpolatedDF.sample`（isSamplingForCaches=true → **lerp3(8 角点, 块位置比例)**，L792-806）+ `beardifying.sample`（结构权重**直采**）
- 角点 buffer 在 `sampleStartDensity/sampleEndDensity` 时对 **finalDensity 整棵树**直采（interpolator.fill → wrapped().fill）
- **Java aquifer 输入 = 插值 finalDensity（逐块）+ StructureWeightSampler（逐块）**，且 `beardifying` 在块生成路径是真实 StructureWeightSampler（NoiseChunkGenerator L102-111 传入，非恒 0 占位符）【确定】

### 4.2 C++ 侧（worldgen_api L612-622）

```cpp
densityBuf[by*256 + bz*16 + bx] = h->finalDensity->sample(fpos);
...
block = aquifer->apply(chunkX*16+bx, wy, chunkZ*16+bz, densityBuf[...]);
```

- `h->finalDensity` = `buildNode(overworld.json final_density)`（worldgen_api L362），顶层 `interpolated` → InterpolatedDF（density_builder L163）
- InterpolatedDF 对整棵树做 cell 角点采样 + lerp3（density.h L471-591）——**语义与 Java DensityInterpolator 一致**（phase6 已验 (-244,58) 一致）
- **没有 +StructureWeightSampler 项**【确定】（grep structure/junction 零匹配，phase6 L157）

### 4.3 数值差异汇总

| 位置 | Java 输入（CellCache） | C++ 输入（densityBuf） | 差 |
|---|---|---|---|
| (-244,58,-256) | -0.0744 + Beardifier(≈+0.11) = **≈+0.037（正，判 solid）** | -0.0744（负，判 water） | **+0.11 = Beardifier** |
| (-278,15..19,-240) | **≤0（判 water 的必要条件）** | ≈+0.06（正，判 solid） | **≥-0.06（来源未定）** |

→ **(-244,-256) 差异 = Beardifier 缺失（确定）**；**(-278,-240) 差异 = Java 输入 ≤0 vs C++ 输入 +0.06，且 Beardifier≈0，来源未定（需补数据）**。

---

## 5. 对比 4：含水层液体来源（1.20.1 全清单）【确定】

Java 1.20.1 aquifer 判水/判岩的液体来源（AquiferSampler.Impl + 外部组件）：

| # | 来源 | Java 代码 | C++ 是否实现 |
|---|---|---|---|
| 1 | 外部 fluidLevelSampler（默认 y<-54 lava / 否则 water@seaLevel） | NoiseChunkGenerator L78-84；apply L153-157 | ✓ 硬编码 defaultFluidLevel（主世界一致） |
| 2 | 3×3×3 blob 网格最近 blob r 的液面（getWaterLevelAt/waterLevels 缓存） | L158-211 | ✓ blockPositions/waterLevels |
| 3 | 13 邻居 surface 扫描液面（estimateSurfaceHeight + fluidLevelFloodednessNoise） | getFluidLevel L353-389 + getFluidBlockY | ✓ |
| 4 | fluidLevelSpreadNoise（噪声液面） | getNoiseBasedFluidLevel L421-433 | ✓ |
| 5 | fluidTypeNoise（lava 判定） | getFluidBlockState L435-450 | ✓ |
| 6 | barrierNoise（e 值过渡） | calculateDensity L263-321 | ✓ |
| 7 | **StructureWeightSampler（density 输入加项，非 aquifer 内部）** | ChunkNoiseSampler L177-181 | **✗ 缺失** |
| 8 | 洞穴场（caves/entrances、noodle 等） | 已内嵌于 finalDensity 树，经插值进入 CellCache | ✓（树内） |

→ **含水层内部液体机制 C++ 全部实现；唯一缺失是第 7 项（density 输入链的 Beardifier）**。【确定】

---

## 6. 机制链定位

### 6.1 (-244,-256) 岛：Java solid / C++ water【机制已定位·确定】

```
C++：densityBuf(-244,58) = finalDensity 插值 = -0.0744 < 0
  → aquifer.apply：density ≤ 0 → 流体决策 → d=0.64>0, bs=water, e=0（两 blob 液面同高）
  → density+e = -0.0744 ≤ 0 → 返回 water → NOISE 填 water ✗（Java 是 stone）

Java：CellCache(-244,58) = 插值 finalDensity(-0.0744) + StructureWeightSampler(+0.11) ≈ +0.037 > 0
  → aquifer.apply：density > 0 → return null → NOISE 填 defaultBlock(stone) ✓

分叉点：apply 入口的 density 符号判定（L149 vs L71）
统一根因：C++ 缺 StructureWeightSampler 项（worldgen_api L619 未加 beardifier）
验证：StructureWeightSampler.sample 在结构下方（j < piece底 → p<0 → f>0）返回正；(-244,58) 在村庄/沉船 piece 下方 12 格内（phase8 dy≈4-12）→ 正贡献 +0.08~+0.11【推测量级】
```

### 6.2 (-278,-240) 含水层：Java water / C++ solid【机制未完全定位·需补数据】

```
Java：NOISE y=15-19 water、y=20-22 stone、y=23 air、y=24+ stone
  → aquifer 在 y=15..19 返回 water（= fluidLevel2.getBlockState(blockY)=WATER，density ≤ 0）
  → aquifer 在 y=23 返回 AIR（最近 blob 液面低于 23，density ≤ 0）
  → aquifer 在 y=20-22/24+ 返回 null（density > 0）→ stone
  ⇒ Java CellCache 输入在 y=15..19、y=23 ≤ 0；y=20-22、y=24+ > 0（符号在 cell 边界内快速翻转）

C++：densityBuf 同列全正（+0.0557@12 / +0.0687@16 / +0.0685@20 / +0.0684@24，插值）→ 全判 solid ✗

矛盾：Java 无插值直采（+0.0685@16 等）与 C++ 插值全正，但 Java 判 water ⇒ Java CellCache ≤ 0 在 y=15..19
Beardifier：(-278,-240) 距村庄/沉船/地牢均 >12 格（StructureWeightSampler 有效半宽 = INDEX_OFFSET 12；
  z 方向距村庄包围盒 16 格 >12 → 越界返回 0）⇒ Beardifier≈0【确定】→ 无法用 Beardifier 解释
候选机制：
  (a) C++ 树/插值在 (-278,-240) 与 Java 存在局部差异（docs「全一致」是全局结论；
      y=20 处 C++ 插值 0.0685 vs 直采 0.0765 已见 0.008 差异；jaggedness 恒 0 可疑——SURF dump y=31=0.000000，
      jaggedness 参与 offset 可把 finalDensity 在特定 y 拉低到 ≤0）【推测，需角点/树对比】
  (b) 存在 phase8 未枚举的结构提供 Beardifier 负贡献（结构上方挖空带）【推测，低可能】
  (c) 探针/块状态数据误读【已排除：NOISE-BLK 与参照 .blocks 双证 water】
```

---

## 7. 修复方向与工程量评估

### 7.1 修复 1：(-244,-256) 岛及全部结构 12 格内差异 = 实现 StructureWeightSampler【确定】

- **接入点**：worldgen_api.cpp L619 `densityBuf[...] = h->finalDensity->sample(fpos)` → 改为 `+ beardifier(x,y,z)`
- **需要**：structure starts → pieces（bbox + terrainAdaptation + groundLevelDelta + jigsaw junctions）数据——C++ 目前无结构系统（worldgen 到 SURFACE 为止）
- **推荐路径**（phase6 已建议，本次复核支持）：
  1. **短期验证**：从 Java/参照提取 -288 区域结构 piece 列表硬编码/查表注入，验证 y=58-61 翻 solid（预期 +0.08~+0.12）
  2. **长期**：实现 structure starts 枚举 + StructureWeightSampler（表长 24³、calculateStructureWeight = e^(-d²/16)）接入
- **工程量**：中等偏大（结构系统前置）；注入式验证 = 小
- **只改 aquifer 输入，不改 aquifer 逻辑**；estimateSurfaceHeight / surface 阶段不用改（Java 高度图路径 Beardifier=0，phase6 L229）

### 7.2 修复 2：(-278,-240) 含水层【需补数据后定】

- **必须先补**：DensityProbe 在 chunk(-18,-15) block(10,0) 的 `[CellCache]` 真实遍历值（y 逐块）+ 8 角点 dump
  - 若 CellCache(y=15..19) 确认 ≤0：对比 C++ InterpolatedDF 同点角点（[GRID] dump）→ 定位树/插值差异（优先查 jaggedness：C++ SURF 显示 y=31 恒 0，1.20.1 jaggedness 参与 offset，量级可达 ±0.1+）
  - 若 CellCache 实测 ≈+0.06（正）：则 Java 判 water 另有来源（需重查块生成/探针路径）
- **工程量**：未定（取决于差异点；若 jaggedness 实现缺失 = 中等）

### 7.3 决策建议（交主会话）

1. **不要**为含水层提前实现 Beardifier（(-278,-240) Beardifier=0，收益 0，phase8 一致）
2. **先跑补数探针**（DensityProbe [CellCache] @ c-18_-15）再决定修复 2
3. Beardifier 实现（修复 1）对结构 12 格内差异（岛/水体边界）有效，可并行推进

---

## 8. 置信度与需补数据清单

- 【确定】：apply/computeSubstance/calculateDensity/getFluidLevel 逐行一致；Java 输入链 = CellCache(Add(插值 finalDensity, Beardifier))；C++ 缺 Beardifier；(-244,58) Java 判 solid 必有正贡献；(-278,-240) Beardifier≈0（>12 格）；Java 判 water ⇒ Java 输入 ≤0（逻辑必然）；块状态双证（NOISE-BLK + 参照 .blocks）
- 【推测】：Beardifier 贡献量级 +0.08~+0.11（phase6）；(-278,-240) 差异归因（树/插值局部差异 vs 未发现结构）；jaggedness 恒 0 的影响
- 【需补数据】：
  1. **Java DensityProbe [CellCache] 逐块 dump @ chunk(-18,-15) block(10,0)**（y=0..40）——判定 (-278,-240) 的 Java 真实 aquifer 输入符号【最高优先】
  2. Java 同列 8 角点 / C++ [GRID] 角点 dump 对比——定位插值差异
  3. C++ jaggedness 直采剖面（y 方向）——确认是否恒 0
  4. -288 4×4 区域 structure starts 全枚举（排除未发现结构）——消除 Beardifier 假说残余
- 本报告只做解读，未运行任何命令（铁律遵守）；产出仅写入 `.investigations/-288-reopen/analysis-phase9.md`

---

## 附：关键源码行号速查

- Java aquifer 输入链：ChunkNoiseSampler L176-181（`add(finalDensity, Beardifier)` 包 cacheAllInCell）
- Java apply：AquiferSampler L143-251（density>0→null L149；blob 遍历 L168-207；d≤0 L212；e L221；g/h L229/L238）
- Java calculateDensity：L263-321
- Java getFluidLevel/getFluidBlockY/getNoiseBasedFluidLevel/getFluidBlockState：L353-450
- Java StructureWeightSampler：L23（INDEX_OFFSET=12）、L86-120（sample）、L140-149（getStructureWeight 越界 0）、L162-169（e^(-d²/16)）
- Java createFluidLevelSampler：NoiseChunkGenerator L78-84
- Java 块生成 null→defaultBlock：NoiseChunkGenerator L409-412
- C++ apply：aquifer.h L70-137；calculateDensity L235-273；getFluidLevel L287-316；estimateSurfaceHeight L142-161
- C++ densityBuf：worldgen_api L612-622、L706-720
- C++ InterpolatedDF：density.h L471-591
