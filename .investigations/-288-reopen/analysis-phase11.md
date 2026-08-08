# Phase 11 — Java noodle 洞穴树推算与 C++ 偏差定位（cns 插值器数据 + 反证）

> 角色：recode.scout（只读勘探，本文件为唯一产出）
> 目标列：(-278, -240)（chunk(-18,-15) 内块 x=10,z=0），y=8..24
> 日期：本次勘探
> 数据源：cmd-output/dp_aquifer_run.txt（InterpDiag）、vanilla_density_overworld_c-18_-15_b10_0_cns.txt（cns 8 插值器）、dump_x-278_z-240.txt（C++）、ref_col_-278_-240.txt（Java 块）、noise_params.json、noodle.json、overworld.json、ChunkNoiseSampler.java、DensityFunctionTypes.java、OctavePerlinNoiseSampler.java、DoublePerlinNoiseSampler.java、docs/03、docs/10-timewise-archive.md

---

## 0. 结论摘要（TL;DR）

1. **idx1-4 = noodle 树的 4 个 interpolated【确定】**：idx1=noodle 主噪声、idx2=thickness（含 -0.075/-0.025 系数）、idx3=noodle_ridge_a 原始噪声、idx4=noodle_ridge_b 原始噪声（InterpDiag delegate 类型直接证明）。**修正 phase10 §5 的「idx4-7」猜测**。
2. **ridge_a/b 的 firstOctave = -7（不是 -8）【确定】**：noise_params.json L29-30 + InterpDiag idx3/4 + C++ worldgen_api.cpp L106-107 三方一致 → **噪声参数 C++=Java**（phase10 的「全同」结论对，但依据 -8 是笔误）。
3. **cns.txt 的 idx1-4 反射值不可信【确定】**：按反射值算，noodle 树 = 64（when_in，主噪声恒 -0.39 负）或 when_out 恒正（+0.40~+1.07）→ 判 solid，**与 Java 判水铁证直接矛盾**。且 noodle 噪声是高频（采样坐标 ×256），反射值却恒 -0.39 → 反射值不是游戏实际插值链输出。docs/03 L93-95 已警告「CellCache/cns 反射不可信」。
4. **游戏实际 noodle 树（插值后）在 y=15-19 必须 ≤ 0【推导·强】**：判水 ⇒ density ≤ 0 = min(squeeze=+0.0655~0.0687, noodle_interp) ⇒ noodle_interp ≤ 0，且走 when_out 分支（主噪声插值 > 0）。
5. **C++ noodle 树插值输出未压低【确定】**：C++ finalDensity@y8..24 = squeeze(0.64×idx0)（0.0427~0.0687 逐位一致）→ C++ noodle 树 ≥ 0.0687@y16 ≠ Java（≤0）→ **偏差就在 noodle 树插值输出**。
6. **偏差节点候选（需 dump 定位，按概率排序）**：
   a. **noodle 4 噪声（noodle/thickness/ridge_a/ridge_b）的 DoublePerlinNoiseSampler 在负坐标网格角点采样差**（这是唯一未验证过负坐标的采样器组）
   b. noodle 树的 4 个 InterpolatedDF 实例构建参数/网格差（interp0 已验证一致 → 机制本身 OK，嫌疑低）
   c. noodle 树结构解析差（when_in/out 分支、add/max/abs 组合）——代码层面与 Java 一致，嫌疑低
   d. RangeChoice 判定差——语义与 Java 一致，嫌疑低
7. **C++ 必须 dump 的最小节点集**见 §6（WG_NOODLEDUMP 新增，不改生产逻辑）。

置信度总览：1/2【确定】；3【确定】（数学矛盾 + docs 双重证据）；4【推导·强】（apply 语义 + overworld.json + 判水铁证唯一推出）；5【确定】（C++=squeeze 逐位一致 + Java 判水）；6【候选】。

---

## 1. idx1-4 ↔ noodle 树节点映射确认【确定】

### 1.1 InterpDiag delegate 类型（dp_aquifer_run.txt L178-185）

```
idx=1: DensityInterpolator, delegate=RangeChoice[input=YClampedGradient[-4064..4062],
      min=-60, max=321, whenInRange=Noise[noodle, xzScale=1.0, yScale=1.0], whenOutOfRange=-1.0]
idx=2: DensityInterpolator, delegate=RangeChoice[...,
      whenInRange=LinearOperation[ADD, LinearOperation[MUL, Noise[noodle_thickness, xz=1,y=1], -0.025], -0.075],
      whenOutOfRange=0.0]
idx=3: DensityInterpolator, delegate=RangeChoice[...,
      whenInRange=Noise[noodle_ridge_a, xzScale=2.6666666666666665, yScale=2.6666666666666665], whenOutOfRange=0.0]
idx=4: DensityInterpolator, delegate=RangeChoice[...,
      whenInRange=Noise[noodle_ridge_b, xzScale=2.6666666666666665, yScale=2.6666666666666665], whenOutOfRange=0.0]
```

对照 noodle.json 结构逐段一致：
- noodle.json L6-17：`interpolated(range_choice(y→noise noodle, else -1))` = **idx1**
- noodle.json L27-45：`interpolated(range_choice(y→add(-0.075, mul(-0.025, noise noodle_thickness)), else 0))` = **idx2**（LinearOperation 折叠后即 InterpDiag 所见）
- noodle.json L56-69：`interpolated(range_choice(y→noise noodle_ridge_a, else 0))` = **idx3**
- noodle.json L75-88：`interpolated(range_choice(y→noise noodle_ridge_b, else 0))` = **idx4**

注意：**abs 在 interpolated 外层**（noodle.json L54/L73），所以 idx3/idx4 是 ridge **原始（可负）**噪声插值值，`max(|idx3|,|idx4|)` 在组合层求。这与 InterpDiag delegate 只显示 RangeChoice/Noise（无 abs 包装）吻合。

### 1.2 与既有文档一致
docs/03-density-functions.md L96：「8 个 DensityInterpolator 映射：idx0=finalDensity 顶层（BlendDensity）、idx1-4=noodle 的 4 个（noodle 噪声/thickness/ridge_a/ridge_b）、idx5-7=ore_vein」。

### 1.3 关键参数修正：ridge_a/b firstOctave = -7
- noise_params.json L29-30：`"noodle_ridge_a": {"firstOctave": -7}`、`"noodle_ridge_b": {"firstOctave": -7}`
- InterpDiag idx3/4：`NoiseParameters[firstOctave=-7, amplitudes=[1.0]]`
- C++：worldgen_api.cpp L106-107（以及 noise_probe/router_probe/ore_probe/density_probe 均 -7）

→ **任务描述与 phase10 记的 -8 是笔误；Java/C++ 均 -7，参数一致【确定】**。

---

## 2. Java caves 树值推算【反射值不可信 + 判水反推】

### 2.1 noodle.json 语义（1.20.1）

```
noodle_tree = range_choice(
    input = interp1   // idx1，插值后的 noodle 主噪声（y∈[-60,321) 时）
    min_inclusive = -1e6, max_exclusive = 0,
    when_in  = 64.0,                                  // 主噪声 ≤0 → 无洞穴（64 岩石）
    when_out = interp2 + 1.5 × max(|interp3|, |interp4|)   // 主噪声 >0 → 洞穴判定
)
```

### 2.2 用 cns 反射 idx 值推算（展示语义，结论：不可信）

反射 idx 值（c-18_-15_b10_0_cns.txt，y=8..24）：

| y | idx1(主噪声) | idx2(thickness) | idx3(ridge_a) | idx4(ridge_b) | 分支 | 若 when_out |
|---|---|---|---|---|---|---|
| 8  | -0.392836 | -0.080962 | -0.714612 | +0.068121 | in→64 | -0.080962+1.5×0.714612=+0.990956 |
| 12 | -0.394182 | -0.081015 | -0.623247 | -0.027540 | in→64 | -0.081015+1.5×0.623247=+0.853856 |
| 15 | -0.395192 | -0.081054 | -0.554723 | -0.099285 | in→64 | -0.081054+1.5×0.554723=+0.751031 |
| 16 | -0.395529 | -0.081068 | -0.531881 | -0.123200 | in→64 | -0.081068+1.5×0.531881=+0.716754 |
| 19 | -0.394161 | -0.081068 | -0.452182 | -0.153752 | in→64 | -0.081068+1.5×0.452182=+0.597205 |
| 20 | -0.393705 | -0.081068 | -0.425615 | -0.163936 | in→64 | -0.081068+1.5×0.425615=+0.557355 |
| 23 | -0.392337 | -0.081068 | -0.345916 | -0.194487 | in→64 | -0.081068+1.5×0.345916=+0.437806 |
| 24 | -0.391881 | -0.081068 | -0.319350 | -0.204671 | in→64 | -0.081068+1.5×0.319350=+0.397957 |

按反射值：主噪声恒 -0.39（负）→ **when_in → noodle_tree = 64**（全 y）；即使误走 when_out 也恒正（+0.40~+0.99）。→ finalDensity = min(squeeze, 64) = squeeze 全正 → **判 solid**。

**与 Java 判水（ref_col：y=15-19 water、y=23 air）直接矛盾【确定】。**

### 2.3 矛盾归因：cns 反射值不可信【确定】

证据链：
1. **数学矛盾**：若 idx1-4 是游戏实际插值链真值，判水不可能（§2.2）。判水是铁的事实 → 反射值不是游戏实际值。
2. **频率矛盾**：OctavePerlinNoiseSampler 的 lacunarity = 2^(-firstOctave) = 2^8 = 256（OctavePerlinNoiseSampler.java L130），DoublePerlinNoiseSampler.sample 直接采样（L75-80）。noodle 主噪声在块坐标**高频**（y×256），y=8→24 跨 4096 噪声单位，块间值必然剧烈变化；而 cns 反射 idx1 恒 -0.3928~-0.3955（变化 <0.005）→ **不是块位置真实采样**。
3. **既有文档**：docs/03 L93-95「CellCache 反射有缓存污染不可信」「cns 反射不可信：DensityInterpolator.sample 依赖 cns 遍历状态」；docs/10 L317 同。ChunkNoiseSampler.java L786-788：`sample(pos)` 当 `pos != ChunkNoiseSampler.this` 时直接 `delegate.sample(pos)`——反射调用（非真实遍历）会拿到**非插值 delegate 直接采样**或**缓存垃圾**。

### 2.4 由判水反推游戏实际 noodle 树（真实目标值）【推导·强】

判水 ⇒ 游戏实际 CellCache 值 ≤ 0@y=15-19（AquiferSampler.java L149 `density > 0 → null`）。
CellCache 值 = finalDensity 树块位置完整采样 + Beardifier(0)（ChunkNoiseSampler.java L177-181，phase10 §3）。
finalDensity = min(squeeze(0.64×interp0), noodle_tree)（overworld.json L30-167，argument2=caves/noodle）。
interp0（插值后）= idx0 反射值可信（§5 验证：squeeze(0.64×idx0)=C++ 逐位一致）= 0.2048~0.2150@y15-19 → squeeze = 0.0655~0.0687（正）。

**⇒ noodle_tree(插值后) ≤ 0 @y=15-19【推导·强】**，且：
- 因 64 > 0，必走 **when_out** 分支 → **interp1(插值后) > 0 @y=15-19**；
- when_out = interp2 + 1.5×max(|interp3|,|interp4|) ≤ 0；interp2 = thickness ∈ [-0.158, +0.008]（恒负~-0.08）→ **max(|interp3|,|interp4|) ≲ 0.054 @y=15-19**（ridge 弱）。
- y=8/12/24（判 solid）：noodle_tree > squeeze 值或 when_out > 0（由 §5 角点一致推，noodle 未压低）。

**与 cns 反射 idx3(|ridge_a|≈0.53@y16) 相差 10 倍 → 再次证明反射值不是插值后真值。**

### 2.5 无插值基准（vanilla_density.txt）与插值链不矛盾【确定】

Java vanilla_density_overworld_c-18_-15_b10_0.txt 是**无插值**基准（docs/10 L751「.txt 无插值」）：router.finalDensity().sample(pos) 中 interpolated 直接采样 delegate。
- y=8=0.042577、y=16=0.068530、y=24=0.068347（y=8/16/24 是 cell 角点，但**x/z 方向仍插值** → 无插值 ≠ 插值链，phase10「角点处本应逐位相等」的推理不完整）。
- 无插值 noodle@16 = 0.068530（正，when_out）与插值 noodle_interp@16 ≤ 0 **可以共存**：noodle 高频（×256），插值（4×4×8 网格 lerp3）前后差异巨大，这正是「interpolated 防 alias」的设计意图。**不矛盾**。

---

## 3. C++ 偏差定位候选【候选，按概率排序】

### 3.1 确定事实
- C++ finalDensity@y8..24 = 0.042736/0.055723/0.068692/0.068530/0.068367 = **逐位等于 squeeze(0.64×idx0)**（§5 验证）→ **C++ noodle 树从未压低 min**（noodle ≥ 0.0687@y16）。
- Java noodle_interp ≤ 0@y15-19 → **C++ noodle 树插值输出与 Java 符号/量级相反**。

### 3.2 候选分析

| # | 候选节点 | 证据/排除情况 | 概率 |
|---|---|---|---|
| a | **noodle 4 噪声的 DoublePerlinNoiseSampler 负坐标采样差** | 参数（-8/-7）与 Java 一致【确定】；sampler 实现与 Java 一致（DOMAIN_SCALE/lacunarity/maintainPrecision 已修复负坐标语义，noise.h L123-129）【代码一致】；但 **noodle 组采样器从未与 Java 游戏实际逐值对比过**（base_3d_noise 排除的是 InterpolatedNoiseSampler 另一类；blend 树 cave_cheese 等通过 interp0 一致排除的是另一组实例）。负坐标网格角点（x=-288..-272, z=-256..-240, y=-64..）是未验证区域 | **高** |
| b | noodle 4 个 InterpolatedDF 构建参数/网格差 | InterpolatedDF 机制已通过 interp0（blend）验证一致【确定】（squeeze 逐位）；若 noodle 的 4 个实例 minY/height 传参错 → 网格错。需 dump 角点确认 | 中 |
| c | noodle 树结构解析差（when_in/out、add/max/abs） | density_builder.h L70-164 解析正确；BinaryOperation 折叠/优化（density.h L80-141）与 Java 语义一致；RangeChoice（L277-305）判定一致 | 低 |
| d | seed 派生顺序差（全局性） | 若全局错，正坐标洞穴也大量错；-288 区总体 95.7% 匹配、chunk(-18,-16)/(-17,-16) 100% 匹配 → 非全局性 | 低 |

### 3.3 推论：为什么候选 a 最可能
- interp0（blend 插值后）与 C++ 逐位一致 → C++ 的 InterpolatedDF 网格机制、y 网格范围、插值公式都 OK；
- interp0 的 delegate（blend_density 树）里也含高频 noise（cave_cheese、cave_layer、spaghetti 等）在**同一负坐标网格角点**采样且一致 → 说明**这些** noise 采样器负坐标 OK；
- 但 noodle 4 噪声是**独立的采样器实例、独立的 seed 派生链**（NoiseConfig 按 registry 顺序创建）。若 C++ 在创建 noodle 采样器时的 random 派生顺序/次数与 Java 差一步 → noodle 组整体错位。**这是唯一没验证过的环节**。

---

## 4. 负坐标噪声嫌疑评估【确定（现象解释）+ 候选（根因）】

1. **idx1 恒 -0.39 不是「负坐标 bug」证据**：反射值不可信（§2.3），且真实 noodle 是高频——恒定恰证明它是**非插值/垃圾值**而非真实采样。
2. **负坐标确实是历史高发区**：noise.h L123-129 记录 maintainPrecision 曾用向零截断，负坐标折叠值差 1 → 噪声差 → 「负坐标地形偏移」——**已修复**。同类风险（Perlin 的 floor 渐变、origin、lacunarity 放大负坐标）仍需 dump 实证。
3. **采样频率放大效应**：noodle 主噪声采样坐标 = 块坐标 × 256（lacunarity）。负坐标 × 256 后绝对值巨大（-278×256≈-71168），PerlinNoiseSampler 的 floor/取整/精度处理若与 Java 有微差，会被 ×256 放大 → 角点值差 → 插值后差。**这是候选 a 的具体机制假设**。
4. 结论：**「C++ noodle 噪声负坐标采样差」仍是首要嫌疑，但必须用网格角点 dump 与 Java 游戏实际对比确认**（单靠代码静态分析无法排除/证实）。

---

## 5. C++ dump 规划【主会话执行】

### 5.1 新增 WG_NOODLEDUMP（worldgen_api.cpp fillOneChunk 内，仿 WG_SURFDUMP L664-688 结构）

**位置**：fillOneChunk 的 WG_SURFDUMP 块之后（约 L688 后），或独立块；改 `fillFromNoise` 之前先取得树引用。

**取节点引用的实现建议**（不改生产逻辑，仅诊断）：
- 在 `wg_create`（L360-365）构建 finalDensity 时，额外保留：
  - `h->noodleTree` = buildNode("minecraft:overworld/caves/noodle") 的引用（与 finalDensity argument2 同一实例，通过 LazyRef/registry 查 `"minecraft:overworld/caves/noodle"`）；
  - `h->squeezeInterp` = finalDensity argument1（squeeze(0.64×interp0)）的引用；
- 4 个 noodle InterpolatedDF 实例：通过 `InterpolatedDF::getInstanceCount()` 与 cacheId 枚举（构造顺序 idx0=blend, idx1-4=noodle——与 Java cns 同序），或更好：给 InterpolatedDF 加 debug 名（构造时注入 "noodle_main"/"thickness"/"ridge_a"/"ridge_b"）。

**输出内容（列 (-278,-240)，y=8..24 每块）**：

```
[NOODLE] (x,y,z) interp1=%.6f thickness=%.6f ridgeA=%.6f ridgeB=%.6f
         whenOut=%.6f rc=%s noodle=%.6f squeeze=%.6f final=%.6f
```
- interp1/thickness/ridgeA/ridgeB：4 个 noodle InterpolatedDF.sample(pos)（插值后）；
- whenOut = thickness + 1.5×max(|ridgeA|,|ridgeB|)（组合层重算）；
- rc = interp1 在 [-1e6,0) ? "in" : "out"；
- noodle = RangeChoice 输出（when_in→64 / when_out）；
- squeeze = h->squeezeInterp->sample(pos)；
- final = h->finalDensity->sample(pos)。

**网格角点 dump（定位采样器）**：对 chunk(-18,-15) 的 noodle 主噪声 InterpolatedDF，打印 buildGrid 角点值（可复用 density.h L514-524 已有 `[GRID]` debug 的写法，条件放宽到 noodle 实例 + 目标 chunk）。角点坐标：x∈{-288,-284,-280,-276,-272}，z∈{-256,-252,-248,-244,-240}，y∈{-64,-56,...,320}（只需 y=8..24 覆盖的 y∈{-64..24} 段）。

**env**：`WG_NOODLEDUMP=1 WG_NOODLEDUMP_X=-278 WG_NOODLEDUMP_Z=-240`（列）+ `WG_NOODLEDUMP_CHUNK=-18,-15`（角点）。

### 5.2 期望值判据（对照 Java）
- 目标：C++ interp1 > 0 且 whenOut ≤ 0 @y=15-19（ridge 弱、thickness≈-0.08）；
- 若 C++ interp1 ≤ 0 → 候选 a/b（interp1 插值错）；
- 若 interp1 > 0 但 whenOut > 0 → 候选 c（组合错，ridge/thickness 值大）；
- 若 noodle 树输出 64（when_in）→ 主噪声插值恒负 → 网格角点值整体错（采样器差或 seed 差）。

### 5.3 Java 侧补充重跑（如需）
- 现有 DensityProbe InterpDiag 只给 delegate 类型不给值。要拿**游戏实际** noodle 分量真值，需在**真实遍历内**取值（docs/10 L111：必须 DensityProbe 完整 cns 链 sampleStartDensity→interpolateY/X/Z 在真实遍历内取，勿用反射）。
- 若 C++ dump 显示角点值 vs Java 需要逐位对比，则 Java 侧在 BlockProbe 驱动 cns 真实遍历时对 noodle 的 4 个 InterpolatedDF 输出各 y 值（与 cns.txt 同格式，但确保真实遍历状态）。

---

## 6. 与既有结论的关系
- **phase10 §0「Java noodle 必须为负」**：方向对，但把 cns idx 值当真值推算的路径不可行（§2.3）；正确结论由判水铁证反推（§2.4）：**noodle_tree(插值后) ≤ 0@y15-19**。
- **phase10 §5「idx4-7 是 noodle」**：错，实际 idx1-4（§1.1）。
- **phase10「噪声参数全同」**：结论对（ridge 为 -7 而非 -8），修正笔误。
- **phase10 §0 第 7 点「角点处本应逐位相等」**：不完整——y 角点处 x/z 方向仍插值，无插值基准 ≠ 插值链（§2.5）。
- **density.h L470 anchor「Beardifier 未实现 → -288 岛缺失根因」**：本列无结构 beardifier=0，**不是本矛盾根因**；但 C++ noodle 树插值输出与 Java 不一致是本列判水缺口的**直接根因候选**（候选 a/b/c）。

## 7. 架构变更建议
- 本 phase 无架构变更；**不修改目标代码**。
- 建议：InterpolatedDF 增加 debug 名称（供 dump 区分实例），属诊断增强，交主会话裁决。

## 8. 引用清单
**Java 源码**：noodle.json；overworld.json L30-167（finalDensity）；noise_params.json L28-31；ChunkNoiseSampler.java L177-181/L786-808（interpolator sample/fill）；DensityFunctionTypes.java L1161-1164（squeeze=x/2-x³/24）；OctavePerlinNoiseSampler.java L130/L143-167（lacunarity=2^8、sample）；DoublePerlinNoiseSampler.java L75-80（DOMAIN_SCALE=1.0181268882175227、amplitude）。
**C++ 源码**：worldgen_api.cpp L104-107（noodle 参数 -8/-7）、L360-365（finalDensity 构建）、L604-640（fillFromNoise/WG_DBDEBUG）、L663-704（WG_SURFDUMP）；density.h L211-229（NoiseDF）、L277-305（RangeChoice）、L469-593（InterpolatedDF）、L80-144（BinaryOperation 折叠/优化）；noise.h L113-275（OctavePerlin/DoublePerlin）。
**数据/探针**：dp_aquifer_run.txt L178-185（InterpDiag）；vanilla_density_overworld_c-18_-15_b10_0_cns.txt；vanilla_density_overworld_c-18_-15_b10_0.txt（无插值）；dump_x-278_z-240.txt（C++）；ref_col_-278_-240.txt（Java 判块）；docs/03-density-functions.md L90-100；docs/10-timewise-archive.md L317/L751。
