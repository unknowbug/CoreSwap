# 8576-24blocks biome-fix：#23/#24 forest↔badlands 判定差精确根因

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6 chunks（chunk 45..50, z -27..-22）
> 范围：#23 (812,73,-337) C++ stone vs 参照 terracotta；#24 (815,89,-337) C++ grass vs 参照 terracotta
> 参照：`versions/1.20.1/data/vanilla_8576294172403134396_6_720_-432.blocks`（SURFACE 状态）
> 角色：anchor.worker 精确分析（只读，不改代码/参照；patch 建议仅供主会话）　日期：2026-08-09　状态：**draft**

---

## 0. 结论先行

- **#23/#24 共同根因（高置信）**：C++ 六维 biome 分量在 (812,-337)/(815,-337) 的 biomePickCell 采样点与 vanilla 存在**微小采样差**，而该区域恰好紧贴 badlands↔forest hypercube 边界（temperature≈0.55、humidity≈-0.1），toLong 量化后落在边界不同侧 → 最近邻翻转。
- **决定性旁证**：`docs/10-timewise-archive.md` L462/L481 已记录同 seed 同区域实测——「biome 六维 @(728,-408) **0 差异**；@(800,-428) 有 0.0007-0.004 差（continents 差 0.003 主因）；@(805,-427) continents 差 1.8e-4、**temperature 差 0.005（spline 放大）——biome 边界翻转的真正输入是 temperature/erosion 的微小差**」。
- **#23 具体机制（中高置信）**：C++ compdump temperature@(812,-337)=**0.549879** → toLong **5498**；vanilla 判 badlands 的硬约束是 temperature∈[0.55,1.0]（toLong ≥5500，`VanillaBiomeParameters.getBadlandsOrRegularBiome: temperature==4`）。C++ 差 2 个 toLong 单位落在 forest 区间 [0.2,0.55]（≤5500）→ forest 温度距离 0、badlands 温度距离 2 → 在湿度/weirdness 等其余维度双方距离相等时 forest 恒胜。
- **#24 具体机制（中高置信）**：(815,-337) vanilla 在 y<90 判 badlands、y≥90 判 forest（翻转 y=90）；C++ 翻转在 y=89（早 1 格）。biome 六维中**唯一 y 依赖分量是 depth**（y_clamped_gradient + offset 2D；temperature/vegetation/continents/erosion/ridges 全部 y_scale=0 是 2D）→ 翻转位置差 = depth 采样值差（或 temperature 边界上 1-2 单位差叠加）。
- **已排除**（源码逐行 + 历史实测）：选点（biomePickCell == BiomeAccess.getBiome）、采样坐标（×4 对齐）、ShiftA/ShiftB/ShiftedNoise/Noise/spline 公式、hashSeed 字节序、Cache2DDF key 粒度（历史已修复：`cpp_comps_8576_45_-26_fix.txt` depth 与 vanilla 完全一致）、参数表结构（biome_params.json 与 VanillaBiomeParameters 一致）、terracotta 带公式。

---

## 1. 三步对拍表

### 步骤 1：biomePickCell 选点对拍 —— ✅ 一致（源码逐行）

| 项 | Java（权威） | C++ | 判定 |
|---|---|---|---|
| 入口 | `BiomeAccess.getBiome(BlockPos)` BiomeAccess.java:30-64 | `biomePickCell` biome.h:121-149 | 逐行对应 |
| 减 2 + >>2 | `i=x-2; j=y-2; k=z-2; l=i>>2; m=j>>2; n=k>>2`（L31-36） | `i=blockX-2; j=blockY-2; k=blockZ-2; l=i>>2; m=j>>2; n=k>>2`（L122-127） | ✓（负坐标 >>2 均算术右移，-339>>2=-85） |
| 余数 | `d=(i&3)/4.0; e=(j&3)/4.0; f=(k&3)/4.0`（L37-39） | `d=(i&3)/4.0; e=(j&3)/4.0; f=(k&3)/4.0`（L128-130） | ✓（负数 & 3 同语义） |
| 8 邻域 | `bl=(p&4)==0; bl2=(p&2)==0; bl3=(p&1)==0`（L44-46） | 同（L134-136） | ✓ |
| 扰动距离 | `method_38106(seed,q,r,s,d,e,f)`（L53） | `biomeCellDistance`（L103-116） | ✓（mixSeed×6 + jitter×3 + 平方和） |
| 选点 | `(o&4)==0?l:l+1; (o&2)==0?m:m+1; (o&1)==0?n:n+1`（L60-62） | 同（L146-148） | ✓ |
| hashSeed | `Hashing.sha256().hashLong(seed).asLong()`（L22-24） | `biomeHashSeed` biome.h:76-84 | ✓（Guava putLong 小端；前序 worker 已确认字节序） |

→ 采样点 = biome 坐标 (px,py,pz) ×4 回 block：(px<<2, py<<2, pz<<2)。

### 步骤 2：六维分量采样对拍 —— 采样坐标 ✅ 一致；分量值 ⚠️ 存在历史差

**坐标链路一致**：
| 项 | Java | C++ | 判定 |
|---|---|---|---|
| 采样点 | `BiomeAccess.getBiomeForNoiseGen(px,py,pz)` → `MultiNoiseBiomeSource.getBiome(px,py,pz,sampler)` → `MultiNoiseSampler.sample(px,py,pz)` → `BiomeCoords.toBlock` = `<<2`（BiomeCoords.java:45-47）→ `UnblendedNoisePos` | `p.x=px<<2; p.y=py<<2; p.z=pz<<2` worldgen_api.cpp:484 / 722-724 | ✓（×4 同语义） |
| 6 分量 | `MultiNoiseUtil.MultiNoiseSampler.sample` MultiNoiseUtil.java:222-235：temperature/humidity/continentalness/erosion/depth/weirdness 同 pos | `samp("temperature"|"vegetation"|"continents"|"erosion"|"depth"|"ridges")` worldgen_api.cpp:489-491 / 729-734 | ✓（字段映射：humidity↔vegetation、weirdness↔ridges） |
| ShiftA/B | `DensityFunctionTypes.ShiftA.sample(pos)=offset(x*0.25,0,z*0.25)*4`；`ShiftB=offset(z*0.25,x*0.25,0)*4`（DensityFunctionTypes.java:943-966） | `ShiftDF::SHIFT_A=y=0`；`SHIFT_B=x=z;y=x;z=0`（density.h:236-246） | ✓ 逐位 |
| ShiftedNoise | `d=x*xz+shiftX; e=y*ys+shiftY; f=z*xz+shiftZ; noise.sample(d,e,f)`（L996-1001） | `ShiftedNoiseDF::sample`（density.h:259-269） | ✓ 逐位 |
| Noise | `noise.sample(x*xz, y*ys, z*xz)` | `NoiseDF::sample`（density.h:215-224） | ✓ 逐位 |
| Spline | **全程 float**（Spline.java:253-275：`float f=locFn.apply`、`float k=(f-g)/(h-g)`、float p/q/lerp） | **double**（density.h:791-812：`double f=locationFunction->sample`、double kd/p/q） | ⚠️ 精度差 1e-7 级；历史 GRID 对比 0 差异，已判定非主因 |
| 2D 包装 | continents/erosion/ridges = flat_cache(cache_2d(shifted_noise y_scale=0))；offset = flat_cache(cache_2d(spline)) | `FlatCacheDF`/`Cache2DDF`（density.h:629-744） | ✓ 修复后一致（Cache2DDF block 级 key，density.h:642） |

**分量值历史实测（决定性）**：
| 位置 | 结果 | 出处 |
|---|---|---|
| (728,-408) | temperature/vegetation/continents/erosion/depth/ridges **0 差异** | docs/10:462 |
| (800,-428) | 0.0007-0.004 差（continents 差 0.003 主因） | docs/10:462 |
| (805,-427) | continents 差 1.8e-4、**temperature 差 0.005（spline 放大）——biome 边界翻转的真正输入** | docs/10:481 |
| (812,-337) | C++ temperature=**0.549879**（compdump），紧贴 badlands 边界 0.55 | 本任务 compdump |
| (812,-337) | C++ vegetation=**-0.095321**，紧贴 humidity 边界 -0.1 | 本任务 compdump |
| (728,-408) | 修复前 C++ depth 差 0.004107；**修复后与 vanilla 完全一致**（cpp_comps_8576_45_-26_fix.txt depth@0=0.561976 == vanilla） | 本任务对拍 + docs/10:323 |

### 步骤 3：find / 距离对拍 —— 公式 ✅ 一致；tie-break ⚠️ 候选（次要）

| 项 | Java | C++ | 判定 |
|---|---|---|---|
| 距离 | `ParameterRange.getDistance(noise)`：`l=noise-max; m=min-noise; l>0?l:max(m,0)`（MultiNoiseUtil.java:362-366）+ `MathHelper.square(...)` 6 维 + offset²（L287-295） | `NoiseHypercube::rangeDistance`（biome.h:166-170）+ `getSquaredDistance`（biome.h:172-180） | ✓ 逐行一致 |
| 参数表 | vanilla 运行时 `VanillaBiomeParameters` → `MultiNoiseBiomeSourceParameterLists` | `biome_params.json`（BiomeParamProbe 导出）→ `BiomeSource::loadFromJson`（biome.h:191-212） | ✓ 结构一致（badlands/forest 条目已核对） |
| 查找 | 游戏实际 `SearchTree.getValue`（MultiNoiseUtil.java:146-152, 520-526） | 线性遍历 `find`（biome.h:221-234） | ⚠️ 平局 tie-break 可能不同（C++ 严格 < 取 entries 首个；SearchTree 按树遍历序） |

---

## 2. 根因（文件:行 + 精确差异）

### 2.1 直接判定链（C++ 实测输入）

compdump（`E:\tmp\compdump_812_-337.txt`，block 坐标 (812,·,-337)，y 无关分量恒定）：

| 分量 | C++ 值 | toLong | badlands range | forest range |
|---|---|---|---|---|
| temperature | 0.549879 | **5498** | [5500,10000]（**差 2**） | [2000,5500]（命中，差 0） |
| vegetation(humidity) | -0.095321 | -953 | [-1000,1000]（命中） | [-1000,1000]（命中） |
| continents | 0.015774 | 157 | [-1100,300]（命中） | [-1100,3000]（命中） |
| erosion | -0.441729 | -4417 | [-7799,-3750]（命中） | [-7799,-3750]（命中） |
| depth | y 线性 | 随 y | [0,0]/[1,1] | [0,0]/[1,1] |
| ridges(weirdness) | 未 dump | — | [-9333,-7666] | [-10000,-9333]/[-9333,-7666] |

**硬约束**（VanillaBiomeParameters.java:1034-1036）：`getBadlandsOrRegularBiome: temperature == 4 ? getBadlandsBiome : getRegularBiome`——vanilla **只有在 temperature index=4（[0.55,1.0]）时才可能判 badlands 系**。

→ 若 (812,-337) 采样点温度值 = C++ compdump 的 0.549879（5498），无论其余 5 维如何，badlands 温度距离 ≥2（平方 4），而 forest 温度距离 0；其余维度双方 range 大多重叠 → **C++ forest 距离恒 ≤ badlands 距离**。
→ vanilla 参照该列判 badlands（blocks biome 段 y=100=badlands + y=73 terracotta 带）→ **vanilla 采样点温度必须 ≥0.55（toLong ≥5500）**，即 vanilla 温度分量与 C++ 有 ≥1-2 个 toLong 单位差（float 层面 ≥ ~0.00005-0.00012）。

**#24 的 y 翻转**：六维中只有 depth 是 y 依赖（`overworld/depth.json` = `y_clamped_gradient(-64,320,1.5,-1.5) + flat_cache(offset)`；temperature/vegetation/continents/erosion/ridges 全部 y_scale=0 2D——已核对 overworld.json:248-265 / continents.json / erosion.json / ridges.json）。badlands↔forest 的 depth range 相同（[0,0]/[1,1]），depth 只通过**距离**影响翻转位置；depth（offset 部分）若差 ~0.004（toLong 41），翻转位置偏移 ~0.5-1 格（depth 每格 -0.0078125），与「(815,-337) C++ 翻转早 1 格」量级吻合。

### 2.2 精确文件:行

| 环节 | C++ 位置 | 说明 |
|---|---|---|
| biome 采样入口 | `worldgen_api.cpp:475-497`（`wg_sample_biome`，`-biomeDump` 走此）+ `worldgen_api.cpp:714-737`（`fillOneChunk` 内 `biomeAt`，surface 逐块判定走此） | `biomePickCell` 选点 → `p.x=px<<2` → 6 分量 `samp(...)` → `biomeSource.find(...)` |
| temperature 分量 | `worldgen_api.cpp:489`（`samp("temperature", p)`）/ `worldgen_api.cpp:729` | shifted_noise，值紧贴 0.55 边界；**首要嫌疑** |
| vegetation 分量 | `worldgen_api.cpp:489`（`samp("vegetation", p)`）/ `worldgen_api.cpp:730` | 值紧贴 -0.1 边界；**次嫌疑** |
| depth 分量 | `worldgen_api.cpp:491`（`samp("depth", p)`）/ `worldgen_api.cpp:733` | 唯一 y 依赖；影响 #24 翻转位置 |
| ShiftedNoise/Shift | `density.h:252-272`（ShiftedNoiseDF）/ `density.h:230-249`（ShiftDF） | 已与 Java 逐位一致；**若 temperature 差经 shift 传导，查 offset 噪声 ulp** |
| Noise | `density.h:209-227`（NoiseDF）；噪声本体 `noise.h:232-275`（DoublePerlinNoiseSampler） | temperature/vegetation/ridge 噪声 |
| Spline | `density.h:791-812`（SplineDF::apply 用 double） | Java 全程 float（Spline.java:253-275）；历史 GRID 0 差异 → 非主因但保留复查 |
| find | `biome.h:221-234`（严格 < 线性遍历） | 平局时与 Java SearchTree 可能不同 |

### 2.3 根因排序（置信度）

1. **temperature 分量在 (812,-337) 采样点 C++/vanilla 微小差（≥1-2 toLong 单位）**——由历史 (805,-427) temperature 差 0.005、(800,-428) 差 0.0007-0.004 佐证，且 (812,-337) 恰在 0.55 边界。**主因候选**。
2. **depth（offset 部分）采样差**——影响 #24 翻转位置早 1 格。历史 (728,-408) 修复前差 0.004107（offset 差）；修复后对齐。需确认 (812,-337)/(815,-337) 当前值。
3. **find 平局 tie-break**（SearchTree vs 线性遍历）——若 vanilla 温度恰 = 5500 与其他维度形成平局，C++/Java 返回不同条目。**次要候选**。
4. spline double vs float 精度、noise ulp——已基本排除（历史 GRID/噪声逐位 0 差异）。

---

## 3. 修复方向（不修代码，供主会话/后续 worker）

1. **首要（定位差分量）**：Java RouterProbe SURFBIOME（= BiomeAccess 8 邻域复刻）@ (812,-337)/(815,-337)，y=73/89/100，输出 6 维分量 + 实际 biome + biomePickCell 选点 (px,py,pz)。与 C++ `-biomeDump` + `WG_COMPDUMP`（注意：compdump 用原始 block 坐标，**biome 判定用 (px<<2,py<<2,pz<<2)**，需在选点后重采 6 维）逐项对拍。
   ```
   block_probe.exe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks -biomeDump 812 73 -337
   WG_COMPDUMP=1 WG_COMPDUMP_X=812 WG_COMPDUMP_Z=-337 block_probe.exe ...
   ```
   Java 侧：`RouterProbe -Prouter.x=812 -Prouter.z=-337 -Prouter.yFrom=60 -Prouter.yTo=100`（SURFBIOME）。
2. **若确认 temperature 差**：对拍 C++/Java 的 temperature 噪声采样链——`WG_B3DDUMP`（C++ Perlin 输入/输出）vs Java RouterProbe 反射 DoublePerlinNoiseSampler；重点 `noise.h:269-274`（DoublePerlinNoiseSampler::sample 的 `x*DOMAIN_SCALE` 与 octave 求和）与 shift 链路（`shift_x`/`shift_z` = offset 噪声，检查 C++ `getNoiseSampler("minecraft:offset")` 与 Java getOrCreateSampler(OFFSET) 逐位）。
3. **若确认 depth 差**：对拍 (812,-337) offset 2D 值（C++ FlatCache 查表 vs Java 游戏 cns 查表）；查 `density.h:673-744`（FlatCacheDF buildGrid/查表）与 `overworld/offset.json` spline 树构建。
4. **若六维全一致但 biome 不同**：复刻 Java SearchTree 平局语义或确认 `biome_params.json` entries 顺序与 Java `MultiNoiseBiomeSourceParameterLists` 完全一致（C++ `biome.h:221-234` 严格 `<` 取首个；SearchTree 按树序）。低优先级。
5. **回归**：任何修复必须全量回归 -288/3200/20000/8576 参照（biome 判定影响 terracotta 带 + 表层规则面）。

---

## 4. 置信度与局限

- **高置信**：选点/采样坐标/Shift/Noise/spline 公式/参数表结构/Cache2D 全部对齐（源码逐行 + 历史 0 差异实测）；(812,-337) temperature=0.549879 贴 0.55 边界 + vanilla badlands 硬约束 temperature≥0.55 → 该列 C++/vanilla 温度值必有 ≥1-2 toLong 单位差。
- **中高置信**：#23 根因在 temperature（或 temperature+vegetation 组合）微小差；(812,-337)/(815,-337) 恰处 temperature 0.55 与 humidity -0.1 双边界交叉。
- **局限**：本 worker 无执行权限，未能现场跑 Java RouterProbe 拿 (812,-337) 选点后 6 维 vanilla 真值；compdump 的 temperature 0.549879 是原始 block 坐标（z=-337）采样，biome 判定实际用 (px<<2,·,pz<<2)（z=-340/-336 等），值可能有 ±几 toLong 单位差，但不改变「紧贴边界 + vanilla 需 ≥0.55」的定性结论。
- 状态：**draft**（AI 不写 confirmed；精确到分量需 §3 第 1 步 Java 对拍）。

## 5. 产物引用

- 本文件：`.artifacts/8576-24blocks/biome-fix/analysis.md`
- 前序：`.artifacts/8576-24blocks/biome-terracotta/analysis.md`（#23/#24 形态与带公式排除）
- 历史实测：`versions/1.20.1/docs/10-timewise-archive.md`（L462、L478-481、L323）
- 数据：`E:\tmp\compdump_812_-337.txt`；`versions/1.20.1/data/cpp_comps_8576_45_-26.txt` / `cpp_comps_8576_45_-26_fix.txt` / `vanilla_density_overworld_c45_-26_b8_8_comps.txt`
- 源码：Java `BiomeAccess.java`、`MultiNoiseBiomeSource.java`、`MultiNoiseUtil.java`、`BiomeCoords.java`、`DensityFunctionTypes.java`（ShiftA/B、ShiftedNoise、Spline）、`Spline.java`、`VanillaBiomeParameters.java`；C++ `biome.h`、`worldgen_api.cpp`、`density.h`、`density_builder.h`、`noise.h`
