# Phase 10 — 裁决：Java NOISE 阶段判水 vs density 全正的硬矛盾

> 角色：recode.scout（只读勘探，本文件为唯一产出）
> 目标列：(-278, -240)（chunk(-18,-15) 内，块 x=10,z=0）
> Java 块：y=15-19 water、y=23 air、其余 stone（ref_col_-278_-240.txt）
> 日期：本次勘探

---

## 0. 结论摘要（TL;DR）

**Java 判水是「finalDensity 的 min(noodle) 分支取负」驱动的，不是 density 插值路径错，也不是 Beardifier，也不是 apply 分支顺序。**

裁决链（每环都有源码/实测支撑）：

1. **aquifer 输入 = CellCache 值 = `add(finalDensity, Beardifier)` 在块位置的完整采样**（ChunkNoiseSampler.java L177-181）。判水 ⇒ 该值 ≤ 0（AquiferSampler.java L149 `if (density > 0.0) return null` 是无条件的第一行，语义完整）。
2. **finalDensity = `min(squeeze(mul(interpolated(...), 0.64)), caves/noodle)`**（DensityFunctions.java L404-406 + L456-458 + overworld.json L30-167）。
3. **cns idx0 只是 finalDensity 内部 interpolated 节点的输出**（正 0.133~0.215），`squeeze(0.64×idx0)` 与 C++ 逐位一致 ⇒ **interp 部分 Java=C++，不是问题所在**。
4. Beardifier=0（任务实测 + 无结构区域）⇒ 不是问题所在。
5. **⇒ Java 的 noodle 在 y=15-23 必须为负**（唯一能把正 interp 压到 ≤0 的项）。
6. **C++ finalDensity 全正（0.0427~0.0687 = squeeze(0.64×idx0)）⇒ C++ 的 noodle 未压低**（≥0.069 或走 when_in_range=64）。
7. 实测差异实锤：Java `vanilla_density` y=8=0.042577 vs C++ y=8=0.042736（角点处本应逐位相等）——**Java 与 C++ 的 noodle 值确实不同**。

**C++ 缺的一环 = noodle 子树（`minecraft:overworld/caves/noodle`）没有在洞穴处产生负值。**
noodle 树的噪声参数、JSON 结构、InterpolatedDF 语义在代码层面均与 Java 一致，具体偏差环节需新增探针逐分量定位（见 §6）。

置信度：裁决链 1-7 中，1-4 为【确定】（源码行号），5-7 为【推导】（强：由 1-4 合取 + 实测数据唯一推出）。

---

## 1. 裁决问题 1：Java apply 完整流程与分支顺序

**源码行号=【确定】**（AquiferSampler.java L143-251）：

```
apply(pos, density):
  1. L149-151  if (density > 0.0) { needsFluidTick=false; return null; }   ← 无条件第一行
  2. L153      fluidLevel = fluidLevelSampler.getFluidLevel(x,y,z)         （默认：y<min(-54,63)→LAVA，否则 WATER(63)）
  3. L154-157  if (fluidLevel.getBlockState(y) 是 LAVA) return LAVA
  4. L158-207  12 邻居随机流体点，取最近 3 个（o/p/q 距离平方）
  5. L209-214  fluidLevel2 = getWaterLevel(r)；d = maxDistance(o,p)；
               blockState = fluidLevel2.getBlockState(y)
               if (d <= 0.0) return blockState                              ← 无 barrier 影响区直接判
  6. L215-217  if (blockState 是 WATER 且下方邻居是 LAVA) return blockState
  7. L219-247  barrier 修正：density+e/g/h > 0 → return null；否则 return blockState
```

**结论**：`density>0 → null` 在**最前面**，phase5 引用无遗漏。**判水必须 density ≤ 0**。C++ aquifer.h apply（L70-140）分支顺序与 Java 逐行一致（L74 `density > 0.0 → -1`）。**apply 本身不是矛盾来源**。

---

## 2. 裁决问题 2：含水层 pocket 机制

**源码行号=【确定】**（AquiferSampler.java L391-419）：

含水层（y=15-19 water 带）由 **「洞穴空腔（density ≤ 0）+ aquifer 液面机制」联合驱动**，没有独立于 density 的水逻辑：

- `apply` 的 density ≤ 0 分支才查液面；
- 液面由 `getFluidBlockY` 决定：
  - `e = floodedness − h`（floodedness = `fluidLevelFloodednessNoise`，h 由表面高度 + `bl` 映射）→ e>0 → 液面 = default 63；
  - 否则 `d = floodedness − k` → d>0 → `getNoiseBasedFluidLevel`（`fluidLevelSpreadNoise` 的 spread 液面，20~60 之间）；
  - 否则 → `DimensionType.field_35479`（-32512，恒 AIR）。
- `fluidLevel2.getBlockState(y)`：y < 液面 → WATER，否则 AIR。

**y=15-19 water vs y=23 air 的边界**：同一列表面高度 32（comps.txt ESTDUMP），bl=true（同列 surface 处非空）。由 floodedness/spread 噪声随 y 变化产生液面分界。C++ 已完整复刻该逻辑（aquifer.h L289-353）。

**推论**：只要 density 在 y=15-23 正确为负，C++ 的液面逻辑会自动给出 water/air 分界（若液面子树本身正确，需后续验证）。

---

## 3. 裁决问题 3：CellCache 值 vs cns idx0 —— 两者不同，idx0 只是 interp 节点

**源码行号=【确定】**：

- ChunkNoiseSampler.java L177-181：
  ```java
  densityFunction = cacheAllInCell(add(noiseRouter2.finalDensity(), Beardifier.INSTANCE))
                       .apply(getActualDensityFunction);
  builder.add(pos -> aquiferSampler.apply(pos, densityFunction.sample(pos)));
  ```
- CellCache（L652-701）：cache 数组大小 = `hcc*hcc*vcc` = **4×4×8 = 128（每块一个）**，`sample` 直接按 `((vcc-1-j)*hcc+i)*hcc+k` 索引，**不是角点插值**；值在 `onSampledCellCorners`（L342-355）用 `delegate.fill(cache, this)` 逐块采样（isSamplingForCaches=true 时 DensityInterpolator.sample 走 lerp3，L786-808）。
- **所以 aquifer 输入 = finalDensity 树在块位置的完整值 = `min(squeeze(0.64×interp), noodle) + beardifier`**，其中 `interp` 是 idx0（interpolated 节点 lerp3 输出）。
- cns idx0（vanilla_density_..._cns.txt）：y=8..24 = 0.1336~0.2150（正）。**但 idx0 ≠ CellCache 值**——CellCache 值还要过 squeeze(×0.64)、min(noodle)、+beardifier。
- **noodle 也是 entryHolder**（RegistryEntryHolder.apply L896-898 **递归展开内部并应用 visitor**），其 4 个 interpolated 也实例化为 DensityInterpolator（cns 共 8 个插值器，cns.txt 证实）。

**关键修正（推翻 phase5/6 可能的误读）**：aquifer 实际拿到的值不是 idx0，也不是「插值后的 idx0」，
而是 **`min(squeeze(0.64×idx0), noodle)`（+beardifier）**。

---

## 4. 裁决问题 4：Java 判水机制链 与 C++ 缺口

### 4.1 Java 判水链（推导，强）

```
y=15-23 块：
  interp（cns idx0）        = +0.1336~0.2150          （实测）
  squeeze(0.64×interp)      = +0.0427~0.0690          （实测，正）
  beardifier                = 0                       （实测）
  noodle（主噪声>0 且 thickness+1.5×max(|ridge|)<0）  = 负  ←【必须是负】
  finalDensity = min(正, noodle)                      = 负（≤0）
  aquifer.apply(density ≤ 0) → 液面分支
      y=15-19：液面 > blockY → WATER
      y=23：   液面 ≤ blockY → AIR
```

- noodle 结构（DensityFunctions.java L308-346 + noodle.json）：`rangeChoice(主噪声, -1e6..0 → 64, thickness + 1.5×max(|ridge_a|,|ridge_b|))`；thickness = `add(-0.075, mul(-0.025, noise))`（恒负 [-0.1,-0.05]）。
- y=15-23 是**面条洞穴空腔**（noodle 主噪声 > 0、ridge 弱），noodle 负 → 压过 interp → 判水/空气。

### 4.2 C++ 缺口（推导，强）

- C++ densityBuf = `finalDensity->sample`（worldgen_api.cpp L619）全正 0.0427~0.0687 = 恰为 `squeeze(0.64×idx0)`（dump_x-278_z-240.txt [SURF] 行）。
- ⇒ **C++ 的 min(noodle) 从未压低**：noodle ≥ 0.069（或走 when_in_range=64），与 Java noodle（负）不同。
- 实测：Java vanilla_density y=8=0.042577 vs C++ 0.042736 —— 角点处（y=8 是 cell 角点，插值=直接采样）本应逐位相等，差异实锤 **Java noodle < C++ noodle**。

**已排除的原因**（不是矛盾来源）：
- apply 分支顺序：Java/C++ 逐行一致（§1）
- Beardifier：无结构区域 = 0（任务实测 + createChunkNoiseSampler L106 的 StructureWeightSampler）
- interp（cns idx0）：与 C++ 逐位一致
- noodle 噪声参数：Java BuiltinNoiseParameters L54-57 vs C++ density_probe.cpp L56-59 全同
- noodle JSON 结构：noodle.json 与 Java 代码版（verticalRangeChoice L669-673 = interpolated(rangeChoice)）一致
- InterpolatedDF 语义：只在 interpolated 节点做 cell 插值，min/squeeze/mul 在插值后应用（density.h L569-571 注释 + L473-593 实现）——与 Java 一致

**未定位到具体行**：noodle 树 4 个插值器（主噪声 / thickness / ridge_a / ridge_b）中，C++ 具体哪个节点的采样值偏大。cns.txt 显示 8 个插值器（idx4-7 应为 noodle 的 4 个，值域与主噪声/thickness/ridge 匹配，见 §5），但需逐 idx 探针确认对应关系与偏差环节。

---

## 5. cns.txt 8 个插值器数据（-278,-240，y=8..24 节选）

| y | idx0 (final interp) | idx1 | idx2 (thickness?) | idx3 | idx4 (noodle主?) | idx5 | idx6 (ridge?) | idx7 (ridge?) |
|---|---|---|---|---|---|---|---|---|
| 8  | +0.1336 | -0.3928 | -0.0810 | -0.7146 | +0.0681 | -0.1228 | -0.5162 | +0.0492 |
| 12 | +0.1743 | -0.3942 | -0.0810 | -0.6232 | -0.0275 | -0.1243 | -0.5767 | +0.0707 |
| 15 | +0.2048 | -0.3952 | -0.0811 | -0.5547 | -0.0993 | -0.1254 | -0.6220 | +0.0867 |
| 16 | +0.2150 | -0.3955 | -0.0811 | -0.5319 | -0.1232 | -0.1258 | -0.6371 | +0.0921 |
| 19 | +0.2146 | -0.3942 | -0.0811 | -0.4522 | -0.1538 | -0.1274 | -0.5829 | +0.1416 |
| 23 | +0.2141 | -0.3923 | -0.0811 | -0.3459 | -0.1945 | -0.1296 | -0.5107 | +0.2078 |
| 24 | +0.2140 | -0.3919 | -0.0811 | -0.3194 | -0.2047 | -0.1301 | -0.4926 | +0.2243 |

- idx0 = applyBlendDensity interp（squeeze(0.64×idx0) 与 C++ 逐位一致，**确认**）。
- idx2 ≈ -0.081 恒定 → 值域符合 thickness（[-0.1,-0.05]）；其余 idx 的**精确归属（哪个是 noodle 主噪声/ridge_a/ridge_b）无法仅凭值域确定**，需探针。
- **注意**：若按「深度优先遍历」推断，noodle 的 4 个插值器应为 idx4-7（slide 树内部 entry 先展开贡献 idx1-3）。但 idx4 在 y=11+ 转负 → 若它是 noodle 主噪声则 y=12-24 全 solid，与判水矛盾 → **说明 idx 归属比预想复杂（可能 slide 树内部 entry 贡献了更多插值器）**，必须探针确认。

---

## 6. 修复方向与下一步（C++ 需改/需验）

### 6.1 第一步（定位，不改代码）：新增 noodle 子树探针

在 `worldgen_api.cpp` fillOneChunk 的 WG_SURFDUMP 分支（L663-688）增加 dump（-278,-240，y=8..24 每块）：

1. `min` 两分支：`squeeze(mul(0.64, interp))` 与 **noodle 树输出**；
2. noodle 树中间值：主噪声插值器输出（=InterpolatedDF 采样）、`range_choice` 判定（in/out）、thickness 值、`1.5×max(|ridge_a|,|ridge_b|)` 值；
3. 与 cns.txt 的 idx4-7（或实际对应 idx）逐块对比，**定位偏差节点**（主噪声？thickness？ridge？range_choice 边界？）。

### 6.2 第二步（修复候选）

- 若 noodle 主噪声插值器偏大/偏小：检查 `InterpolatedDF` 对该插值器的构建（minY/noiseHeight 参数、buildGrid 网格角点）；
- 若 range_choice 判定错（y 范围 / `minecraft:y` 语义）：检查 `resolveRef("minecraft:y")`（density_builder.h L232-237）与 `RangeChoice`（L277-305）；
- 若 thickness/ridge 值错：检查 add/mul/abs/max 折叠与 `Constant` 折叠（density_builder.h L102-109 的 LinearOperation 折叠）；
- 若 InterpolatedDF 实例数与 Java 的 8 不一致：检查 registry entry 是否都正确展开（density_builder.h resolveRef/externalLoader）。

### 6.3 判水目标

修好后，C++ finalDensity 在 y=15-23 应出现 ≤0（noodle 负），aquifer.apply 自动进入液面分支：
- y=15-19 液面在块上 → WATER（需顺带验证 C++ getFluidBlockY 液面，aquifer.h L329-353）；
- y=23 液面 ≤ 块 → AIR；
- y=8/24+ 保持 solid。

---

## 7. 与既有结论的关系

- **density.h L470 @anchor.idk「Beardifier 未实现 → -288 岛缺失根因」**：本列（-278,-240）无结构、beardifier=0，**不是本矛盾原因**；但 Beardifier 仍是另一个待办（结构区域缺口），不冲突。
- **phase5「apply L145-151 density>0→null」**：语义完整无遗漏（§1 确认）。
- **phase6「aquifer 输入 = cacheAllInCell(add(finalDensity, StructureWeightSampler)).sample」**：正确，但需补「**该值 ≠ cns idx0，中间还有 squeeze(×0.64) 与 min(noodle)**」。

---

## 8. 引用清单

**Java 源码**：
- AquiferSampler.java L143-251（apply 完整流程）
- AquiferSampler.java L353-419（getFluidLevel/getFluidBlockY 液面）
- ChunkNoiseSampler.java L158-188（noiseRouter2 / aquifer 构造 / densityFunction 装配）
- ChunkNoiseSampler.java L342-355（onSampledCellCorners 填 CellCache）
- ChunkNoiseSampler.java L442-470（getActualDensityFunction 替换）
- ChunkNoiseSampler.java L652-701（CellCache per-block cache）
- ChunkNoiseSampler.java L786-808（DensityInterpolator.sample isSamplingForCaches→lerp3）
- DensityFunctions.java L404-406（applyBlendDensity = squeeze(mul(interpolated,0.64))）
- DensityFunctions.java L456-458（finalDensity = min(applyBlendDensity, noodle)）
- DensityFunctions.java L308-346（noodle 公式）/ L669-673（verticalRangeChoice=interpolated(rangeChoice)）
- DensityFunctionTypes.java L884-898（RegistryEntryHolder.apply 递归展开）
- NoiseRouter.java L50-68（apply 字段顺序，finalDensity 最后）
- NoiseChunkGenerator.java L359-435（populateNoise 主循环）
- BuiltinNoiseParameters.java L54-57（noodle 噪声参数）

**C++ 源码**：
- aquifer.h L70-140（apply）；worldgen_api.cpp L612-622（densityBuf=finalDensity->sample）、L705-720（aquifer 调用）
- density.h L473-593（InterpolatedDF）；density_builder.h L79-81（min）、L160-164（interpolated）、L232-237（y）

**数据/探针**：
- ref_col_-278_-240.txt（Java 块：15-19 water、23 air）
- vanilla_density_overworld_c-18_-15_b10_0.txt（Java density dump：y=8=0.042577, y=16=0.068530）
- vanilla_density_overworld_c-18_-15_b10_0_cns.txt（cns 8 插值器）
- dump_x-278_z-240.txt（C++ finalDensity：y=8=0.042736, y=16=0.068692）
- vanilla_density_overworld_c-18_-15_b10_0_comps.txt（router 分量全对齐）
