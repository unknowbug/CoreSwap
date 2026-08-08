# Phase 13 — 最终综合裁决：Java 判水((-278,15..19,-240) water) vs C++ 全正机制

> 角色：recode.scout（只读勘探，本文件为唯一产出）
> 目标：裁决「Java 判水但 C++ 侧密度指标全正」的机制，重点核查 spaghetti 分支启用条件（slopedCheese>1.5625）与 raw noodle 语义
> 数据/源码：DensityFunctions.java（createSurfaceNoiseRouter/createCavesFunction/createCavesNoodleOverworldFunction）、overworld.json、noodle.json、noise_params.json、OctavePerlinNoiseSampler.java、DoublePerlinNoiseSampler.java、AquiferSampler.java、ChunkNoiseSampler.java、StructureWeightSampler.java、NoiseConfig.java、NoiseParametersKeys.java、RandomSplitter.java；C++ worldgen_api.cpp、density.h、density_builder.h、noise.h、xoroshiro.h；cmd-output/noodle_run.txt、rangechoice_run.txt、dp_aquifer_run.txt、dump_x-278_z-240.txt、vanilla_density_overworld_c-18_-15_b10_0.txt(_cns)、ref_col_-278_-240.txt、aqfin_aquifer.txt、m288_vanilla_cat.txt；analysis-phase10/11/12.md、phase8.md
> 日期：本次勘探

---

## 0. 结论摘要（TL;DR）

1. **raw noodle 语义裁决【确定·重大修正】**：noodle 是**低频**噪声（firstOctave=-8 → lacunarity=2^(-firstOctave)=2^8=0.00390625，采样坐标 ÷256，OctavePerlinNoiseSampler.java L88/L130-157 与 C++ noise.h L147 一致）。**phase11/12 的「noodle 高频（×256）」判断方向反了**——C++ noodle_run.txt 的 raw_n=-0.384~-0.396 **平滑抛物线是正确采样值**，不是高频丢失。
2. **cns 反射 idx1「不可信」论证失效【确定】**：phase11 §2.3 判 idx1 不可信的核心证据是「频率矛盾」（平滑 vs 应高频）。noodle 实为低频 → idx1 恒 -0.39 平滑**恰恰是真实 raw noodle 直采的合理形态**（ChunkNoiseSampler.java L786-788 外部 pos → delegate.sample 直采）→ **Java interp1(raw) ≈ -0.395 → when_in → 64 → Java noodle 树恒正 → Java 判水不来自 noodle 树 when_out**。phase12 的「判水负值必来自 noodle when_out」推断链断裂。
3. **finalDensity 树插值链 Java/C++ 逐位一致【确定·决定性新证据】**：Java 真实 CellCache（dp_aquifer_run.txt L485-502，y=8..24）= +0.042736~+0.068693，与 C++ aqfin_aquifer.txt **逐位一致（≤1e-6）**。→ noodle 树、左侧 squeeze、整棵 finalDensity 树的 Java/C++ 插值链输出相同 → **出路 B（raw 采样差）、出路 C（插值链差）在树层面排除**。
4. **slopedCheese 阈值核查【确定】**：rangechoice_run.txt 实测 DF14 range_choice 的 input（slopedCheese）在 grid 角点：(-280,8)=5.073、(-280,16)=4.077、(-280,24)=3.108、(-280,32)=2.143 全部 > 1.5625 → **y=8..32 走 when_out = createCavesFunction 完整 caves 树**（含 spaghetti_2d/entrances/pillars/cave_cheese）；y≥40 走 when_in=min(slopedCheese, 5×entrances)。
5. **判块快速交替是裁决支点【确定】**：ref_col y=15-19 water、y=20-22 stone、y=23 air、y=24 stone → 判块 density 需在 y=19/20、22/23、23/24 三次符号翻转（周期 ~8-10 格的中频）。低频分量（noodle when_out、thickness、ridge、spaghetti 主链、Beardifier 高斯）**无法**提供此快速交替 → 唯一中频来源 = **cave_cheese**（firstOctave=-8，最高有效 octave=-1 amplitude=2.0，yScale=0.6667 → y 方向周期 ~6 格）→ 判水负值必来自 **DF14 when_out 的 caves 树（cave_cheese 中频）**。
6. **最终裁决【强候选·未 100% 闭合】**：Java 判水机制 = **slopedCheese>1.5625 → caves 树（cave_cheese 中频）在块位置提供 density≤0 翻转**（水/空气夹层），C++ 侧 caves 树插值输出平滑正（角点 0.131~0.443，rangechoice_run.txt）→ **C++ 的 caves 树（cave_cheese 中频）插值链与 Java 判块所需不同**。出路 A 修正后成立为**唯一存活**机制；出路 B/C 排除。但 Java 侧 caves 树真实插值链值缺失（CellCache 被项目裁定污染，docs/09 L740），**机制链未完全闭合**。
7. **C++ 修复方向**：核查 C++ caves 树（cave_cheese 采样/DF14 when_out 组合/InterpolatedDF 对中频的 8 格 y cell 插值）在负坐标远端的值与 Java 的差；补 dump caves 树内部（cave_cheese raw + DF7 + min/max 组合）角点值。

置信度总览：1/2/3/4/5【确定】；6【强候选：机制闭合方向】【未闭合：Java 侧 caves 树值缺失】；7【方向】。

---

## 1. raw noodle 语义裁决（出路 B 前置核查）

### 1.1 Java 的 lacunarity 公式【确定】

OctavePerlinNoiseSampler.java L88/L130-131：
```java
int j = -this.firstOctave;                       // firstOctave=-8 → j=8
this.lacunarity = Math.pow(2.0, -j);             // 2^-8 = 0.00390625
this.persistence = Math.pow(2.0, i - 1) / (Math.pow(2.0, i) - 1.0);
```
sample（L150-163）：`double e = this.lacunarity; ... maintainPrecision(x * e); e *= 2.0;` → 第一 octave 坐标 = x × 2^firstOctave = x / 256。

**负 firstOctave = 坐标缩小 = 低频**。noodle(-8)、thickness(-8)、ridge_a/b(-7)、cave_cheese(-8 多 octave)、cave_layer(-8) 全部低频或中频；**没有高频分量**。

### 1.2 C++ 实现一致【确定】

noise.h L134-150：`j = -firstOctave; lacunarity = pow(2.0, -j);`、`sample` 中 `maintainPrecision(x * e)`、`e *= 2.0` —— 与 Java 逐行一致（origin/permutation/gradient/lacunarity/maintainPrecision 均已在负坐标修复，noise.h L123-129）。

### 1.3 C++ noodle_run.txt 平滑是正确行为【确定】

DoublePerlin.sample(-278,16,-240)：first/second 各自 octave 坐标 = 块坐标 × 0.0039 → y=16↔17 坐标差 = 0.0039 → Perlin 变化 ≈ 梯度×0.0039 ≈ 0.002/块。实测 raw_n 相邻块差 0.000012~0.0009，同量级 → **-0.395 平滑抛物线 = 正确低频采样**。

### 1.4 phase11/12 的「高频」论证错误【确定】

- phase12 §3.2 用 dp_aquifer_run [CAVES-NOISE]（noodle 800 0 534=-0.056、1600 0 1068=+0.624）证明「raw 高频」——实为**坐标跨度大**（800↔1600 跨多个 Perlin 格）导致的低频采样差异，**不是高频**。
- phase11 §2.3 证据 2「noodle 高频（×256）→ 反射值平滑不可信」**方向反了**（实为 ÷256 低频）。

### 1.5 对 cns 反射 idx1 可信度的重新评估【强】

ChunkNoiseSampler.java L786-788：`sample(pos)` 当 `pos != ChunkNoiseSampler.this` → `delegate.sample(pos)`（直采）。idx1 delegate = RangeChoice(Y,-60,320,noise(noodle),-1)（phase11 §1.1）→ 外部 pos 反射 = **raw noodle 直采**。raw noodle 低频平滑 → **idx1=-0.395 平滑是可信的真实 raw noodle 直采**，phase11 §2.3 的「不可信」论证（数学矛盾 + 频率矛盾）在低频语义下不再成立。

**推论**：Java interp1（noodle 主噪声，低频 → 插值前后一致）≈ -0.395 < 0 → noodle 顶层 rangeChoice when_in → 64 → **Java noodle 树恒正 → 判水负值不来自 noodle when_out**。phase12 §2.2 的「判水必走 noodle when_out」推断链断裂。

---

## 2. finalDensity 树插值链 Java/C++ 逐位一致（出路 B/C 在树层面排除）【确定·决定性】

### 2.1 Java 真实 CellCache 值（dp_aquifer_run.txt L485-502）

| y | Java CellCache | C++ aqfin | 差 |
|---|---|---|---|
| 8  | 0.042736 | 0.042736 | 0 |
| 12 | 0.055724 | 0.055723 | 1e-6 |
| 15 | 0.065453 | 0.065452 | 1e-6 |
| 16 | 0.068693 | 0.068692 | 1e-6 |
| 19 | 0.068571 | 0.068570 | 1e-6 |
| 20 | 0.068530 | 0.068530 | 0 |
| 23 | 0.068408 | 0.068408 | 0 |
| 24 | 0.068367 | 0.068367 | 0 |

**Java 真实 CellCache（判块输入 cacheAllInCell(add(finalDensity, Beardifier))，ChunkNoiseSampler.java L177-181）在 y=8..24 与 C++ densityBuf 逐位一致（≤1e-6）**。→ 树结构（含 noodle 树 when_in→64、左侧 squeeze）、噪声采样、cell 插值全部一致。

### 2.2 排除结论

- **出路 B（raw noodle 采样差）【排除】**：raw/interp1 一致（-0.395，when_in→64，noodle=64）【§1+§2.1】
- **出路 C（noodle 插值链差）【排除】**：noodle 低频 → 插值前后一致；且 Java/C++ 全树插值链逐位一致

### 2.3 判块矛盾的本质

Java CellCache = +0.068（正）→ AquiferSampler.apply L149 `density > 0 → null(solid)` → 不可能判 water。**但 ref_col 判 water 是铁证**。→ **判块真实 density ≠ CellCache dump 值（+0.068）**。phase9 已裁定「CellCache 反射缓存污染不可信」（docs/09-multi-dimension.md L740）。→ 判块真实 density 必含 +0.068 之外的**让 y=15-19,23 为负**的分量，而所有反射/dump 路径（CellCache/cns/无插值 txt）都未捕获该分量。

---

## 3. slopedCheese 阈值核查（出路 A 前提）【确定】

### 3.1 DF14 range_choice 结构（Java 源码 L453-455 + overworld.json L72-155）

```java
DF14 = rangeChoice(slopedCheese, -1e6, 1.5625,
                   min(slopedCheese, 5×entrances),   // when_in
                   createCavesFunction(...))          // when_out
```

### 3.2 实测 slopedCheese（rangechoice_run.txt，grid 角点，列 x=-280/-276, z=-240）

| y 角点 | (-280,z) input | (-276,z) input | 分支 |
|---|---|---|---|
| -56 | 13.02 | 12.99 | out |
| -16 | 7.89 | 7.89 | out |
| 0   | 6.04 | 6.04 | out |
| 8   | **5.073** | 5.060 | out |
| 16  | **4.077** | 4.080 | out |
| 24  | **3.108** | 3.101 | out |
| 32  | **2.143** | 2.118 | out |
| 40  | 1.225 | 1.214 | **in** |
| 64  | -0.229 | -0.232 | in |

**y=8..32 全部 > 1.5625 → DF14 = when_out = 完整 caves 树（spaghetti_2d/entrances/pillars/cave_cheese/cave_layer）**。块位置 (-278,15..24,-240) 的 slopedCheese 由角点插值，同样 > 1.5625 → **caves 树在目标列启用**。

### 3.3 C++ 侧 caves 树 when_out 输出（rangechoice_run.txt 同 dump）

| y 角点 | (-280) caves | (-276) caves |
|---|---|---|
| 8  | 0.131267 | 0.135995 |
| 16 | 0.211507 | 0.218496 |
| 24 | 0.210840 | 0.217122 |
| 32 | 0.436919 | 0.442805 |

**C++ caves 树角点全正平滑（0.13→0.21→0.21→0.44）**。

---

## 4. 判块快速交替 → 中频来源定论（裁决支点）【确定】

### 4.1 判块模式（ref_col_-278_-240.txt，单列 x=10,z=0）

```
y=0-3 deepslate、y=4-14 stone、y=15-19 WATER、y=20-22 stone、y=23 AIR、y=24-30 stone
```
→ 判块 density 需：y=15-19 ≤ 0（water）、y=20-22 > 0（solid）、y=23 ≤ 0（air）、y=24 > 0（solid）→ **在 y=19/20、22/23、23/24 三次符号翻转（周期 ~8-10 格中频）**。

### 4.2 频率清单

| 分量 | firstOctave | 最高有效 octave | y 频率 | 可 3 次翻转？ |
|---|---|---|---|---|
| noodle 主噪声 | -8 [1.0] | -8 | ÷256 超低频 | ✗ |
| noodle thickness/ridge | -8/-7 | -8/-7 | ÷256/÷128 | ✗ |
| spaghetti_2d 等 | -7/-11 | 低 | 超低频 | ✗ |
| cave_layer | -8 [1.0] | -8，yScale=8 | ÷32 | ✗ |
| **cave_cheese** | -8 [0.5,1,2,1,2,1,0,2,0] | **octave -1 (amplitude 2.0)**，yScale=0.6667 | **坐标 ×0.333，周期 ~6 格** | **✓** |
| Beardifier（StructureWeightSampler） | — | 高斯×线性符号 | 平滑 | ✗（phase8 已排除：距结构垂直 44-73 格>24） |

**唯一中频 = cave_cheese（octave -1，amplitude 2.0）**。而 cave_cheese **只在 DF14 when_out（caves 树，slopedCheese>1.5625）参与** → 判水负值必来自 **caves 树（cave_cheese 中频）**。

### 4.3 Beardifier 排除（phase8 §1-3 确认）

- -288 区域无矿井（rail/cobweb/oak_log 深部零命中）；深部结构=地牢（FEATURE，不参与 Beardifier）
- (-278,-240) 距村庄/沉船垂直 44-73 格 > 24 → **Beardifier=0**（StructureWeightSampler INDEX_OFFSET=12，结构上方 hollow 负修正不覆盖本列）
- 本 phase 复核 StructureWeightSampler.java L101-105：修正符号依赖 y 相对结构地面，平滑高斯，无法 3 次快速翻转，且本列无结构 → 排除

---

## 5. A/B/C 裁决

| 出路 | 结论 | 依据 |
|---|---|---|
| **A. spaghetti/caves 分支**（修正为：**caves 树中频缺失/差**） | **成立·强候选** | slopedCheese>1.5625 实测启用 caves 树；判块 3 次翻转只能由 cave_cheese 中频提供；C++ caves 树角点平滑正（0.13~0.44）无中频翻转 → C++ 与 Java 判块所需不同 |
| **B. raw noodle 语义差** | **排除** | noodle 低频；C++ raw=-0.395 平滑正确；finalDensity 全树插值链 Java/C++ 逐位一致 |
| **C. 插值链差** | **排除** | 低频插值前后一致；Java CellCache=C++ densityBuf 逐位一致 |

**机制链（A，推断·强，未 100% 闭合）**：
```
slopedCheese = 3.1~5.1 > 1.5625 @ y=8..32
→ DF14 = createCavesFunction（完整 caves 树）
→ cave_cheese（octave -1, amp 2.0, yScale 0.6667）中频在 y=15..24 提供密度≤0 翻转
→ 左侧 squeeze(0.64×interp(blend(caves))) 在 y=15-19,23 ≤ 0 → 判 water/air
→ y=20-22,24 > 0 → stone
C++：caves 树插值输出平滑正（无中频翻转）→ 左侧恒正 +0.065~+0.068 → 全判 solid
```

**未闭合点（诚实标注）**：
1. Java 侧 caves 树（DF14 when_out）真实插值链值缺失——现有 Java 数据（CellCache/cns/无插值 txt）都被污染或为整树，无 caves 树分量值
2. 「Java CellCache=+0.068（正）却判 water」的矛盾，依赖「CellCache 污染」裁定解释，未直接采样到判块真实 density
3. C++ caves 树角点平滑 vs Java 判块所需中频翻转的量级差异（0.2-0.5 级）巨大，指向 cave_cheese 采样链在负坐标远端的差，但**该差尚未在 Java/C++ 之间逐点定位**

---

## 6. C++ 修复方向（若 A 成立）

1. **补 dump C++ caves 树内部**（DF14 when_out 的分量，仿 WG_NOODLEDUMP）：
   - cave_cheese raw（DoublePerlin 直接采样，负坐标远端）
   - DF7 = 4×square(cave_layer) + clamp(0.27+cave_cheese) + clamp(1.5-0.64×slopedCheese)
   - min(min(DF7,entrances), spaghetti+roughness) 与 pillars 的 max 组合角点值
   - 对照 Java 同点（Java 侧需在真实遍历内取 caves 树分量值，勿用反射）
2. **重点核查**：
   - cave_cheese 采样器（octave -1 中频）在负坐标网格角点的值与 Java 是否一致（candidate：seed 派生/octave 索引/坐标精度）
   - InterpolatedDF 对中频的 8 格 y cell 插值是否与 Java DensityInterpolator 完全一致（Java L763-783 interpolateY/X/Z 的分层 lerp vs C++ L525-539 一次性 trilinear——两者数学等价但浮点路径不同，中频下可能放大差）
3. **若确认 caves 树采样差** → 修 cave_cheese 采样链；**若确认插值路径差** → 修 InterpolatedDF 分层插值；**若确认 caves 树本身一致** → 回查判块 density 的 Beardifier 输入（虽然 phase8 排除，需复核结构定位）

---

## 7. 现有数据无法裁决的部分 + 下一步数据需求

**现有数据可裁决**：noodle 低频语义（B/C 排除）、slopedCheese 启用 caves 树、判块中频来源=cave_cheese、C++ caves 树平滑正。

**不可裁决**：Java caves 树（cave_cheese 中频）在 (-278,8..24,-240) 的真实值是否确实 ≤0（判水）及翻转模式。

**下一步数据（按优先级）**：
1. **Java 真实遍历内 finalDensity 逐块值**（勿用反射/CellCache；BlockProbe 在 sampleStartDensity→interpolate 循环内对 cacheAllInCell 之外再取块密度，或直接读 populateNoise 的 blockState 判定路径）——一次性裁决「判块 density 是否=+0.068」
2. **Java caves 树分量**：cave_cheese raw（DoublePerlin 直采，格式对齐 C++ [NOODLE]）@(-278,8..24,-240) + DF14 when_out 输出（对齐 C++ RANGECHOICE out）——裁决「Java caves 树是否负/中频」
3. **C++ dump caves 树内部**（§6.1）——对齐 Java 后逐点定位差
4. **复核结构定位**：m288_vanilla_cat 显示 -288 区有 5296 个 structure_feature 块（coal_ore/copper_ore/oak 木构/kelp 等）——若存在 phase8 未识别的水下结构（如埋藏宝藏/水下废墟），Beardifier 需重查

---

## 8. 引用清单

**Java 源码**：DensityFunctions.java L308-346（noodle）、L378-402（createCavesFunction）、L414-495（createSurfaceNoiseRouter，L453-458 finalDensity/min）；overworld.json L30-167（final_density 数据驱动树，L72-155 DF14 when_out=完整 caves 树）；OctavePerlinNoiseSampler.java L88/L130-157（lacunarity=2^-8）；DoublePerlinNoiseSampler.java；AquiferSampler.java L145-251（L149 density>0→null）；ChunkNoiseSampler.java L177-181（判块 density=cacheAllInCell(add(finalDensity,Beardifier))）、L786-808（反射直采）；StructureWeightSampler.java L101-105/L160（修正符号）；NoiseConfig.java L52-58/L143-149（seed 派生）；NoiseParametersKeys.java L83；RandomSplitter.java L16-18
**C++ 源码**：worldgen_api.cpp L69-130（buildNoiseParams）、L193-203（fillNoiseSamplers）、L360-362（buildNode finalDensity）、L705-730（WG_NOODLEDUMP）、L712-724（raw_n 采样）；density.h L50/L114-141（BinOp）、L277-305（RangeChoice dump）、L469-593（InterpolatedDF）；density_builder.h L64-128/L244-304（JSON 构建/惰性加载）；noise.h L134-150（lacunarity）、L214-228（OctavePerlin.sample）、L249-274（DoublePerlin）；xoroshiro.h L81-84（split(label) MD5 派生）
**数据/探针**：noodle_run.txt L4-34（raw_n/t/a/b 平滑+tree=64）；rangechoice_run.txt L1406-1480/L6448-6985（slopedCheese out/in、caves out 值）；dp_aquifer_run.txt L176-191（InterpDiag）、L485-502（Java CellCache y=8..24=+0.0427~+0.0687）、L700-705（[CAVES-NOISE] raw noodle 坐标跨度差异）；vanilla_density_overworld_c-18_-15_b10_0.txt（无插值）；dump_x-278_z-240.txt（[SURF] C++ 剖面、[COMP] 分量）；ref_col_-278_-240.txt（判块铁证）；aqfin_aquifer.txt（C++ densityBuf）；m288_vanilla_cat.txt（structure_feature 5296 块）；analysis-phase8.md（结构定位/Beardifier 排除）、phase10/11/12.md（前序结论与修正点）
