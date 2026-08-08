# Phase 12 — caves 树综合裁决：含水层列 (-278,-240) 的 C++/Java 差定位

> 角色：recode.scout（只读勘探，本文件为唯一产出）
> 目标：综合裁决 finalDensity 的 min 另一分支（caves/noodle 树）在含水层列 (-278,-240)（chunk(-18,-15)）的 C++/Java 差，定位 C++ 偏差节点
> 数据/源码：DensityFunctions.java（createSurfaceNoiseRouter/createCavesFunction/createCavesNoodleOverworldFunction）、overworld.json、noodle.json、ChunkNoiseSampler.java、AquiferSampler.java、DoublePerlinNoiseSampler.java、OctavePerlinNoiseSampler.java；C++ worldgen_api.cpp、density.h、noise.h、density_builder.h；cmd-output/aqfin_aquifer.txt、rangechoice_run.txt、dp_aquifer_run.txt、vanilla_density_overworld_c-18_-15_b10_0.txt(_cns/_comps)、ref_col_-278_-240.txt、dump_x-278_z-240.txt；analysis-phase10/11.md

---

## 0. 结论摘要（TL;DR）

1. **caves 树结构定论【确定】**
   - Java `finalDensity = min(squeeze(0.64×interp(blend(slides(DF14)))), noodle)`（overworld.json L30-167 / DensityFunctions.java L450-458）
   - `noodle`（caves/noodle）= `rangeChoice(input=插值后主噪声, -1e6, 0, when_in=64, when_out=thickness+1.5×max(|ridge_a|,|ridge_b|))`
   - `DF14` 内部另有 caves 树（spaghetti_2d/entrances/pillars/cheese），但**只当 slopedCheese > 1.5625 启用**
   - C++ 从 overworld.json/noodle.json **纯数据驱动 buildNode**（worldgen_api.cpp L360-362）→ 树结构逐节点同构【确定】
2. **Java 判水机制链【推导·强】**
   - 判水铁证（ref_col：y=15-19 water、y=20-22 stone、y=23 air、y=24 stone）⇒ AquiferSampler 门槛 `density ≤ 0`
   - 左侧 squeeze 恒正（+0.0655~+0.0687，C++/Java 多列验证；(-278,-240) 列差 1.6e-4 不翻符号）
   - ⇒ **负值由 noodle 树产生**：noodle ≤ 0 @ y=15-19,23，走 when_out（主噪声插值 > 0）且 when_out = thickness(≈-0.08 恒负) + 1.5×max(|ridge_a|,|ridge_b|) ≤ 0（ridge 弱 ≲0.054）
   - **spaghetti/pillars/entrances 不是本列判水根源**——它们在 DF14 caves 树分支（slopedCheese>1.5625 才启用），且左侧已验证恒正
3. **C++ 偏差定位【实测 + 候选】**
   - C++ finalDensity = squeeze（+0.0427~+0.0687 全正）→ 判 solid（aqfin_aquifer.txt）【实测】
   - C++ noodle input（rangechoice_run.txt）= **-0.395193/-0.395529/-0.391882**（y=15..24 全负、平滑抛物线、y≈16 最小）→ when_in → 64【实测】
   - **平滑抛物线 ≠ 高频噪声真实值**：Java raw noodle@(800,0,534)=-0.056004696、@(1600,0,1068)=+0.624240650（dp_aquifer_run.txt CAVES-NOISE）——raw 是高频随机【实测】
   - cns 反射 idx1 与 C++ input 逐位一致（1e-6 级）——**但 cns 反射不可信**（phase11 §2.3）→ C++ 与 Java 反射"一致地"给出非真实插值值
   - ⇒ **偏差落在 C++ noodle 主噪声采样/插值链（高频丢失→平滑 -0.395→when_in→64），与 Java 真实（>0→when_out→≤0）符号相反**
4. **角点 1.6e-4 差**：Java txt（无插值基准）y=16=0.068530 vs C++ y=16=0.068692——**左侧 squeeze 链**的位置相关微差（phase3 的 (-244,-256) 列逐位一致 ≤2e-6，本列有 1.6e-4）；量级不翻符号，**与判水矛盾无直接因果**，但与 noodle 偏差可能同源（高频噪声采样器组在负坐标远端的累计差）
5. **C++ dump 规划**见 §5（WG_NOODLEDUMP：插值值 + grid 角点 + raw 噪声三重 dump，与 Java 真实插值链对比）

置信度总览：1【确定】；2【推导·强】（判水铁证 + apply 语义唯一推出）；3【实测：C++ 现象】【候选：根因】；4【确定：现象】【候选：机制】。

---

## 1. caves 树结构定论

### 1.1 Java finalDensity 树（createSurfaceNoiseRouter，DensityFunctions.java L414-495）

```
finalDensity  = densityFunction15 = min(
                    applyBlendDensity(applySurfaceSlides(false, densityFunction14)),   // = squeeze(0.64×interp(blend(slides(DF14))))
                    entryHolder(CAVES_NOODLE)                                           // caves/noodle 树
                )
```

其中（L446-458）：
```
DF14 = rangeChoice(
    slopedCheese,                                  // = SLOPED_CHEESE_OVERWORLD（L447-449）
    -1e6, 1.5625,                                  // 当 slopedCheese ≤ 1.5625
    min(slopedCheese, 5 × entrances),              // when_in 分支
    createCavesFunction(lookup, noise, slopedCheese)  // when_out 分支（slopedCheese > 1.5625）
)
```

### 1.2 createCavesFunction（caves 树，DensityFunctions.java L378-402）——仅当 slopedCheese > 1.5625

```
DF4 = 4 × square(noise(cave_layer, xz=1, y=8))              // 频率参数见 InterpDiag：NoiseParameters[-8,[1.0]]
DF5 = noise(cave_cheese, xz=1, y=0.6667)                     // NoiseParameters[-8,[0.5,1,2,1,2,1,0,2,0]]
DF6 = clamp(0.27 + DF5, -1, 1) + clamp(1.5 - 0.64×slopedCheese, 0, 0.5)
DF7 = DF4 + DF6
DF8 = min( min(DF7, entrances), spaghetti_2d + spaghetti_roughness )
DF10 = rangeChoice(pillars, -1e6, 0.03, -1e6, pillars)
caves = max(DF8, DF10)
```

### 1.3 noodle 树（caves/noodle，createCavesNoodleOverworldFunction L308-346 + noodle.json）

```
verticalRangeChoice(yFn, f, min, max, out) = interpolated(rangeChoice(yFn, min, max, f, constant(out)))

interp1 = verticalRangeChoice(Y, noise(noodle, xz=1, y=1),       -60, 320, -1)   // 插值后主噪声
interp2 = verticalRangeChoice(Y, -0.075 + (-0.025)×noise(noodle_thickness, 1,1), -60, 320, 0)
interp3 = verticalRangeChoice(Y, noise(noodle_ridge_a, xz=2.6667, y=2.6667),     -60, 320, 0)
interp4 = verticalRangeChoice(Y, noise(noodle_ridge_b, xz=2.6667, y=2.6667),     -60, 320, 0)

noodle = rangeChoice(interp1, -1e6, 0,
                     64.0,                                        // 主噪声插值 ≤ 0 → 岩石
                     interp2 + 1.5 × max(|interp3|, |interp4|))   // 主噪声插值 > 0 → 洞穴判定
```

### 1.4 C++ 同构性

- C++ `wg_create`（worldgen_api.cpp L360-362）：`finalDensity = builder->buildNode(*router.final_density)`，noodle 树经 externalLoader/预注册读 `data/minecraft/worldgen/density_function/overworld/caves/noodle.json`（L338-345）→ **树结构由 JSON 保证逐节点同构**，RangeChoice/BinaryOperation/abs 语义在 density_builder.h L70-164 解析【确定】
- C++ 的噪声参数表（worldgen_api.cpp L104-107）与 Java noise_params.json 一致：noodle -8 [1.0]、thickness -8 [1.0]、ridge_a/b -7 [1.0]【确定】（phase11 §1.3 修正 ridge 为 -7）
- C++ DoublePerlinNoiseSampler 构造（noise.h L249-267）与 Java（DoublePerlinNoiseSampler.java L38-65）一致：amplitude = 0.1666…/createAmplitude(k-j)【确定】

---

## 2. Java 判水机制链（y=15-19 water / y=23 air）

### 2.1 判水门槛（AquiferSampler.Impl.apply，AquiferSampler.java L145-151）

```java
if (density > 0.0) { return null; }   // solid
else { ... 水位锚点 + 距离惩罚，density + penalty ≤ 0 才判流体 ... }
```
- 判水铁证（ref_col_-278_-240.txt）：y=15-19 water、y=20-22 stone、y=23 air、y=24 stone、y=4-14 stone
- water/air 由 `fluidLevelSampler.getFluidLevel(...)` 的水位决定（y < 水位 → water，y ≥ 水位 → air）；**只需 density ≤ 0 即进入判定**，water/air 的分布不反映密度梯度

### 2.2 由判水反推 noodle 值【推导·强】

- density = finalDensity 树块位置完整采样 + Beardifier(0)（ChunkNoiseSampler.java L177-181）→ density = finalDensity
- finalDensity = min(squeeze, noodle)（§1.1）
- 左侧 squeeze 恒正（+0.0655~+0.0687；phase11 §5 / aqfin vs txt）→ **y=15-19,23 处 noodle ≤ 0；y=20-22,24 处 noodle > 0**
- noodle = rangeChoice(interp1, -1e6, 0, 64, when_out)；64 > 0 → **判水必走 when_out ⇒ interp1（主噪声插值后）> 0 @ y=15-19,23**
- when_out = interp2 + 1.5×max(|interp3|,|interp4|)；interp2 = thickness ∈ [-0.158, +0.008]（实测插值后 ≈ -0.08，恒负）→ **需 max(|interp3|,|interp4|) ≲ 0.054（ridge 弱）→ when_out ≤ 0**
- **结论：产生负值的是 noodle 树的 when_out 分支（thickness 负主导 + ridge 弱）；spaghetti_2d / entrances / pillars / cave_cheese 不在本列判水链上**（它们在 DF14 的 caves 树分支，slopedCheese > 1.5625 才启用；本列左侧 squeeze 恒正说明 DF14 未贡献负值）

### 2.3 为什么 cns 反射的"noodle=64"不矛盾【确定】

- cns 反射 idx1-4 显示主噪声恒 -0.39、noodle 走 when_in=64 → 判 solid，与判水铁证直接矛盾
- 归因：cns 反射不可信（phase11 §2.3 证据链：数学矛盾 + 频率矛盾 + docs/03 L93-95「CellCache 反射不可信」+ ChunkNoiseSampler.java L786-788 反射直接 delegate.sample）
- 本 phase 新增证据：**Java raw noodle 是高频**（§3.2），cns 反射的平滑 -0.39 不可能是 raw，也不可能是真实插值

---

## 3. C++ 偏差定位

### 3.1 C++ 现象（实测）

| 量 | C++ 值（y=15..24） | 含义 |
|---|---|---|
| finalDensity（aqfin_aquifer.txt） | +0.065452/+0.068692/…/+0.068367 全正 | 判 solid（非 Java 判水） |
| noodle RANGECHOICE input（rangechoice_run.txt） | -0.395193/-0.395529/…/-0.391882 全负 | when_in → 64 |
| squeeze 左侧（与 Java txt 比） | 差 1.6e-4 @y16 | 微差，不翻符号 |

### 3.2 Java raw noodle 是高频（实测，dp_aquifer_run.txt L700-705）

```
[CAVES-NOISE] noodle 800 0 534 -0.056004696
[CAVES-NOISE] noodle 800 -64 534 -0.338286008
[CAVES-NOISE] noodle 1600 0 1068 0.624240650
```
- 同名称噪声不同坐标值差巨大（-0.34 ↔ +0.62）→ raw 高频随机（x×256、y×256，OctavePerlinNoiseSampler.java L148-167 lacunarity=2^8）【确定】
- cns 反射 idx1 / C++ input 的平滑抛物线（y≈16 最小 -0.3955，两侧 -0.392）**不可能是高频真实值** → 两者都在非真实路径上

### 3.3 偏差结论

- **C++ noodle 主噪声采样/插值链未得到真实高频值**：C++ input（插值后）平滑 -0.395（与 cns 反射一致），而 Java 真实插值必须 > 0 @ y=15-19,23（§2.2）→ **C++ 与 Java 符号相反**，分支翻转（C++ when_in=64 vs Java when_out≤0）
- 1.6e-4 的左侧差无法解释 +0.05 量级翻符号 → **根因在 noodle 树本身，不在 squeeze**

### 3.4 根因候选（按概率，需 dump 实证）

| # | 候选 | 证据/排除 |
|---|---|---|
| a | **C++ noodle 主噪声采样链高频丢失**（InterpolatedDF cacheId=1 的 buildGrid 角点采样 / NoiseDF→DoublePerlin 链 / seed 派生） | 平滑抛物线 = 高频丢失的形态；noodle 是 -8 单 octave 高频，cave_cheese 同 -8 但经 interp0 验证正确——**区别在 noodle 链的包装/实例** |
| b | C++ InterpolatedDF（noodle 4 实例）grid 构建/参数差 | interp0 机制已验证逐位，但 noodle 实例参数/角点未验证；需 dump 角点 |
| c | noodle 树结构解析差（when_in/out、add/max/abs、verticalRangeChoice 的 -1/0 常数） | 数据驱动同构 + 代码与 Java 语义一致，嫌疑低 |
| d | seed 派生顺序差（全局性） | -288 区 95.7% 匹配、chunk(-18,-16)/(-17,-16) 100% → 非全局性；且 cns 与 C++ 反射一致暗示 raw 采样链一致，嫌疑低 |

**倾向解释**：C++ 与 Java cns 反射"一致地"走出平滑 -0.395（可能两者在块坐标直接采样时都遭遇同一类坐标/缓存问题，或 C++ 复刻了反射路径的错误语义），而 Java 游戏真实 ChunkNoiseSampler 遍历（interpolated 防 alias 机制）给出高频真实插值（>0）。**必须用真实遍历内 dump 定位**（§5）。

---

## 4. 角点 1.6e-4 差分析

- Java txt（无插值基准，vanilla_density_overworld_c-18_-15_b10_0.txt）y=16 = **+0.068530**；C++（aqfin_aquifer.txt）y=16 = **+0.068692** → 差 1.6e-4
- phase3 在 (-244,-256) 列角点逐位一致（≤2e-6）→ 本列差是**位置相关**的（负坐标远端 / 含水层列特有）
- 该值属于**左侧 squeeze 链**（finalDensity 在 txt 无插值模式下 = min 取 squeeze 侧；noodle 未压低），**不是 noodle 树的值**
- 量级 1.6e-4 vs 判水需要 ≥0.05 翻符号 → **与判水矛盾无因果**
- 候选机制：高频噪声（noodle 同组或 cave_cheese 组）在负坐标 grid 角点的累计精度差；或 InterpolatedDF 远端角点插值差。**与 §3.4 的 noodle 偏差可能同源**（同一高频采样器组在负坐标远端未逐位），需 dump 角点值定位

---

## 5. C++ dump 规划（主会话实施）

### 5.1 新增 WG_NOODLEDUMP（worldgen_api.cpp fillOneChunk，仿 WG_SURFDUMP L663-688 / WG_DBDEBUG L604-640）

**取引用**（wg_create L360-362 处，不改生产逻辑）：
- `h->noodleTree` = buildNode(overworld.json final_density 的 argument2)（或注册表查 "minecraft:overworld/caves/noodle"）
- `h->squeezeInterp` = finalDensity 的 argument1
- noodle 4 个 InterpolatedDF：InterpolatedDF 增加 debug 名（构造时注入 noodle_main/thickness/ridge_a/ridge_b）或用 cacheId（Java cns 同序 idx0=blend, idx1-4=noodle）

**输出 A——列值**（(-278, y∈[8,24], -240) 每块）：
```
[NOODLE] (x,y,z) interp1=%.6f thickness=%.6f ridgeA=%.6f ridgeB=%.6f
         whenOut=%.6f rc=%s noodle=%.6f squeeze=%.6f final=%.6f
```

**输出 B——grid 角点**（定位采样器；density.h L576-592 buildGrid 加 dump，条件 noodle 实例 + chunk(-18,-15)）：
```
[GRID-N] cacheId=%d pos=(%d,%d,%d) value=%.6f
```
角点坐标：x∈{-288,-284,…, -272}，z∈{-256,-252,…,-240}，y∈{-64,-56,… ,24}（覆盖 y=8..24 的 cell）

**输出 C——raw 噪声**（不经 InterpolatedDF，直接 DoublePerlin.sample）：
```
[RAW-N] noodle (x,y,z) = %.9f
```
坐标 (-278, 8..24, -240)，对照 dp_aquifer_run.txt 的 [CAVES-NOISE] 格式（Java 侧补同坐标 raw 即可直接逐位比）

**env**：`WG_NOODLEDUMP=1 WG_NOODLEDUMP_X=-278 WG_NOODLEDUMP_Z=-240 WG_NOODLEDUMP_CHUNK=-18,-15`

### 5.2 Java 侧补充（拿真实插值链真值）

- 现有 InterpDiag 只给 delegate 类型不给值；需在 BlockProbe 驱动 cns **真实遍历内**（sampleStartDensity→interpolateY/X/Z）对 noodle 4 个 InterpolatedDF 输出各块值（勿用反射；docs/10 L111）
- 输出格式与 [NOODLE] 对齐，供逐位对比

### 5.3 期望值判据

| C++ dump 结果 | 结论 |
|---|---|
| interp1 > 0 且 whenOut ≤ 0 @ y=15-19 | 与 Java 一致 → 排查 §3.4 之外（如 aqua 判定/水位） |
| interp1 ≤ 0 @ y=15-19（当前 -0.395） | 候选 a/b：主噪声采样/插值错 → 再看 RAW-N vs Java raw 与 GRID-N 角点形态 |
| RAW-N 与 Java raw 逐位一致、GRID-N 角点也高频 | 插值公式/网格参数错（候选 b） |
| RAW-N 与 Java raw 不一致 | 采样器/seed/参数错（候选 a/d） |
| interp1 > 0 但 whenOut > 0 | 组合错（候选 c，ridge/thickness 值异常） |

---

## 6. 架构变更建议

- 本 phase 无架构变更；不修改目标代码
- 诊断增强建议（交主会话裁决）：InterpolatedDF 增加 debug 名；buildGrid 的 [GRID-N] dump 条件化

## 7. 引用清单

**Java 源码**：DensityFunctions.java L378-402（createCavesFunction）、L404-407（applyBlendDensity）、L414-495（createSurfaceNoiseRouter）、L308-346（noodle）；AquiferSampler.java L145-151（判水门槛）；ChunkNoiseSampler.java L177-181/L786-808；DoublePerlinNoiseSampler.java L38-65/L75-80；OctavePerlinNoiseSampler.java L84-133/L148-167（lacunarity=2^8）
**C++ 源码**：worldgen_api.cpp L104-107（噪声参数）、L338-362（数据驱动构建 finalDensity）、L604-640/L663-704（dump 样例）；density.h L211-229（NoiseDF）、L277-305（RangeChoice）、L469-593（InterpolatedDF/grid）；noise.h L113-228（OctavePerlin/lacunarity）、L231-275（DoublePerlin）
**数据/探针**：aqfin_aquifer.txt（C++ finalDensity 全正）；rangechoice_run.txt L269454-295485（noodle input 全负走 in）；dp_aquifer_run.txt L176-191（InterpDiag idx0-7）、L700-705（Java raw noodle 高频）；vanilla_density_overworld_c-18_-15_b10_0.txt/_cns.txt/_comps.txt；ref_col_-278_-240.txt（判块铁证）；analysis-phase10/11.md
