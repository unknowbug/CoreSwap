# 8576-24blocks biome-fix #23/#24 温度链路对拍（第 2 步）

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6 chunks
> 范围：#23 (812,73,-337) C++ stone vs 参照 terracotta；#24 (815,89,-337) C++ grass vs 参照 terracotta
> 角色：anchor.worker 精确分析 subagent（只读；无 bash/exe 执行权限——read_only_task 实测全部被拦）
> 本文件：第 2 步（temperature 链路逐位对拍），承接 `analysis.md`（第 1 步）
> 日期：2026-08-09　状态：**draft**

---

## 0. 结论先行（修正 + 新发现）

- **修正 analysis.md 一处关键误判**：`analysis.md` §2.1 表格写 badlands 湿度 range `[-1000,1000]（命中）` **是错的**。已从 `biome_params.json`（Java BiomeParamProbe 导出，与 VanillaBiomeParameters 运行时一致）核对：
  - `badlands` / `eroded_badlands`：**humidity ∈ [-1.0,-0.35] 或 [-0.35,-0.1]** → toLong **≤ -1000**
  - `wooded_badlands`：humidity ∈ [0.1,0.3] 或 [0.3,1.0] → toLong **≥ +1000**
  - `forest`：humidity ∈ [-0.1,0.1] → toLong **[-1000,1000]**
  - ⇒ C++ compdump 的 vegetation(humidity)=**-0.095321 → toLong -953**：**badlands 系三族全部不命中**（离 badlands 边界 -1000 差 **47**），只有 forest 命中（距离 0）。
- **湿度差是 C++ 判 forest 的最强驱动**（修正 analysis.md「温度差 2 主导」的判断）：温度差仅 2 个 toLong 单位（5498 vs ≥5500，距离²=4），**湿度差 ≥47 个 toLong 单位（距离²=2209）**。C++ 判 forest 主要由湿度（vegetation）驱动，温度次之。
- **vanilla 判 plain badlands 的必要条件**（blocks 文件 biome 段 (812,-337)=badlands，biome-terracotta §1.3 已确认索引）：temperature ≥0.55（toLong ≥5500）**且** humidity ≤-0.1（toLong ≤-1000）。⇒ vanilla 与 C++ 在**同一选点坐标**的温度、湿度必有真实差：**湿度差 ≥0.0047（47/10000），温度差 ≥0.0002（2/10000）**。二者均远超 double/float 舍入 ulp（~1e-7）→ **不是浮点精度问题，是采样值本身的差**。
- **静态对拍穷尽后未定位到 C++ 链路 bug**：选点、采样坐标、shift 链路、噪声公式、RNG 派生、参数表、FlatCache/Cache2D、toLong float 全部与 Java 逐位一致（本文件 §2）。差异必在**运行时噪声采样值**（temperature/vegetation 噪声 sampler 的 origin/permutation，或选点坐标确认），需主会话运行时对拍（§4 命令）。
- **#24 悖论解释**：同列 y=87/88 判 badlands、y=89 判 forest 不矛盾——biomePickCell 选点含 py 扰动（jitter 的 e 偏移随 y 变），**不同 y 可能选出不同 (px,pz)**，因此 2D 温度/湿度虽 y 无关但**采样坐标随 y 变**，值随之变 → 同列不同 y 可翻转到不同侧。C++/vanilla 翻转位置差 1 格 = 选点坐标变化在边界上的 1-2 toLong 翻转。

---

## 1. 修正后的直接判定链（决定性）

compdump（`E:\tmp\compdump_812_-337.txt`，block 坐标 (812,·,-337)，%.6f 打印 double；**注意：biome 判定实际用选点坐标 (px<<2, py<<2, pz<<2)，z=pz<<2 ∈ {-340,-336}，与 -337 差 1-3 格**）：

| 分量 | C++ compdump（z=-337） | toLong | badlands 实际 range | 判定 |
|---|---|---|---|---|
| temperature | 0.549879 | **5498** | [5500,10000]（**差 2**） | forest [2000,5500] 命中 |
| vegetation(humidity) | -0.095321 | **-953** | badlands/eroded ≤-1000（**差 47**）；wooded ≥+1000（差 1953） | forest [-1000,1000] 命中 |
| continents | 0.015774 | 157 | [-0.11,0.03]∪[0.03,1.0]（命中） | forest（命中） |
| erosion | -0.441729 | -4417 | [-0.7799,-0.375]（命中） | forest（命中） |
| depth | y 线性 | y 依赖 | [0,0]/[1,1]（双方相同） | 不影响 forest↔badlands 相对胜负 |
| ridges(weirdness) | 未 dump | — | 未知（仅此维未验证） | — |

- **C++ 判 forest 的机制**：温度距离 0 + 湿度距离 0（badlands 系温 2、湿 47 → 总差 ≥2209），其余维双方 range 重叠 → forest 恒胜。
- **vanilla 判 badlands 的必然推论**：vanilla 在选点坐标湿度 ≤-1000 且温度 ≥5500（否则距离差 2209 无法弥补）。⇒ **C++/vanilla 湿度差 ≥47（0.0047）或温度差 ≥2（0.0002）真实存在**。
- 湿度 0.0047 与 docs/10 L481「temperature 差 0.005」量级吻合（该记载为 comps 直接采样 vs 游戏查表之差，方向一致但**不能**作为 C++ 与 vanilla 差的直接证据——见 §5 局限）。

---

## 2. 逐环节对拍表（本 worker 全部重新核过源码）

### 2.1 选点与坐标 —— ✅ 逐位一致（含 seed 来源）
| 项 | Java（权威） | C++ | 判定 |
|---|---|---|---|
| BiomeAccess seed | `ChunkRegion.java:102 new BiomeAccess(this, BiomeAccess.hashSeed(seed))`；`hashSeed = Hashing.sha256().hashLong(seed).asLong()` | `biomeAccessSeed = biomeHashSeed(seed)` biome.h:76-84（sha256(le8) 前 8 字节小端） | ✓ |
| getBiome 8 邻域 | BiomeAccess.java:30-64 | biome.h:121-149 | ✓ 逐行 |
| method_38106 | BiomeAccess.java:84-97：mixSeed(l,i),(m,j),(m,k),(m,i),(m,j),(m,k) → jitter → mix(m,l)×2 | biome.h:103-116 | ✓ 逐行（顺序一致） |
| method_38108 | BiomeAccess.java:99-102 `Math.floorMod(l>>24,1024)/1024.0` → (d-0.5)*0.9 | biome.h:94-100（`%1024` 负数补 1024） | ✓ 等价 floorMod |
| mixSeed | SeedMixer：`seed*(seed*6364136223846793005L+1442695040888963407L)+salt`（溢出） | biome.h:87-91（uint64 回绕） | ✓ |
| 采样点 | `MultiNoiseUtil.MultiNoiseSampler.sample` → `BiomeCoords.toBlock` = <<2 → UnblendedNoisePos | p.x=px<<2 等 | ✓ |

### 2.2 噪声公式 —— ✅ 逐位一致
| 项 | Java | C++ | 判定 |
|---|---|---|---|
| DoublePerlinNoiseSampler | DoublePerlinNoiseSampler.java:75-80：`(first.sample(x,y,z)+second.sample(x·1.0181268882175227, y·…, z·…))·amplitude`；amplitude=0.16666666666666666/createAmplitude(k-j) | noise.h:232-275 | ✓ 逐行（含 DOMAIN_SCALE 常量） |
| OctavePerlinNoiseSampler.sample | OctavePerlinNoiseSampler.java:143-167：`maintainPrecision(x·e)`、`d += amp·g·f`、e*=2、f/=2；useOrigin=false 路径 y 用 maintainPrecision(y·e) | noise.h:214-228 | ✓ 逐行 |
| maintainPrecision | `lfloor(v/3.3554432E7+0.5)·3.3554432E7`（lfloor 负向取整） | noise.h:123-129 `(long)(v/3.3554432E7+0.5)` | ⚠️ 见 §5.2（本任务坐标不触发折叠，非主因） |
| PerlinNoiseSampler.sample | PerlinNoiseSampler.java:33-63（origin+floor+fadeLocalY） | noise.h:50-75 | ✓ 逐行（含 yScale 分支 1.0E-7F） |
| sampleSection/lerp3/grad | PerlinNoiseSampler.java:86-105 / SimplexNoiseSampler.GRADIENTS | noise.h:82-110 / GRADIENTS[16][3] | ✓（历史逐位 0 差异佐证） |
| RNG 派生 | RandomSeed.createXoroshiroSeed(seed)=createUnmixedXoroshiroSeed().mix()；mixStafford13 | random.h:8-32 | ✓ 常量/顺序完全一致 |
| octave split | Xoroshiro128PlusPlusRandom.java:120-147 `split("octave_"+l)` → MD5 | xoroshiro.h:81-84 `split("octave_"+std::to_string(l))` | ✓（字符串拼接一致） |
| nextDouble/nextInt | `next(53)*1.110223E-16F`（float 精度）/ bound 拒绝采样 | xoroshiro.h:37-54 | ✓ |

### 2.3 shift 链路 —— ✅ 逐位一致
| 项 | Java | C++ | 判定 |
|---|---|---|---|
| ShiftedNoise | DensityFunctionTypes.java:995-1001 `d=bx·xz+shiftX.sample(pos)`… | density.h:259-269 | ✓ 逐行 |
| ShiftA/B | DensityFunctionTypes.java:937-977 `sample(bx,0,bz)` / `sample(bz,bx,0)` → Offset.sample(x·0.25)·4 | density.h:236-249 | ✓ |
| Noise | DensityFunctionTypes.java:751-753 `noise.sample(bx·xz, by·y, bz·xz)` | density.h:215-224 | ✓ |
| shift_x/shift_z JSON | shift_x.json = flat_cache(cache_2d(shift_a(offset))) | density_builder.h:220-230（ShiftDF SHIFT_A + WrappingDF） | ✓（Wrapping 纯委托无损） |
| overworld temperature/vegetation | noise_settings/overworld.json:248-265 shifted_noise(xz_scale=0.25, y_scale=0.0, shift_y=0.0) | worldgen_api.cpp:369-376 buildNode 同 JSON | ✓ |

### 2.4 缓存层 —— ✅ 逐位一致（本次重点复核）
| 项 | Java ChunkNoiseSampler | C++ | 判定 |
|---|---|---|---|
| FlatCache 查表 key | FlatCache.sample（L858-864）：`BiomeCoords.fromBlock(blockX)` = **blockX>>2（算术右移）** − startBiomeX | FlatCacheDF.sample（density.h:698）：`pos.x>>2 − slot.cx*4` | ✓（负坐标 -337>>2=-85 同语义） |
| FlatCache 网格 | FlatCache 构造（L840-855）：5×5，点 (chunkX*4+i)<<2，(chunkZ*4+j)<<2，y=0 | density.h:727-738（p.x=(chunkX*4+i)*4, y=0） | ✓ |
| Cache2D | ChunkNoiseSampler.Cache2D：block 级 (x,z) 复用（值不变） | density.h:629-667 key=block x,z | ✓（缓存不改值，纯无损） |
| 采样 y 语义 | FlatCache 填网格 y=0；ShiftA 内部 y 硬编码 0 → 与 biome 采样 y=py<<2 等价 | ShiftDF SHIFT_A 置 y=0；ShiftedNoise e=pos.y·0.0=0.0 | ✓（temperature 的 e 恒 0，与 Java 一致） |

### 2.5 参数表与 toLong —— ✅ 一致
| 项 | Java | C++ | 判定 |
|---|---|---|---|
| temperature 噪声参数 | noise_params.json:58 firstOctave=-10, amplitudes [1.5,0,1,0,0,0] | worldgen_api.cpp:74 同 | ✓ |
| vegetation 噪声参数 | noise_params.json:60 firstOctave=-8, amplitudes [1.0,1.0,0,0,0,0] | worldgen_api.cpp:75 同 | ✓ |
| offset 噪声参数 | noise_params.json:32 firstOctave=-3, amplitudes [1.0,1.0,1.0,0] | worldgen_api.cpp（同 JSON） | ✓ |
| toLong | MultiNoiseUtil.toLong = `(long)(value·10000.0F)`（float 乘） | biome.h:152 `(long)(v·10000.0F)` | ✓ |
| double→float 转换点 | MultiNoiseUtil.MultiNoiseSampler.sample L227-233 `(float)…sample(pos)` | worldgen_api.cpp:487/727 `(float)…sample(q)` | ✓ |
| 参数表 | VanillaBiomeParameters → biome_params.json | biome.h:191-212 loadFromJson | ✓（forest/badlands/wooded/eroded 湿度区间已逐条核对，见 §1） |

### 2.6 判定查找 —— ⚠️ tie-break 候选（次要）
| 项 | Java | C++ | 判定 |
|---|---|---|---|
| getValueSimple（线性，`m < l` 严格小于取首个） | MultiNoiseUtil.java:122-139（仅测试用） | biome.h:221-234（严格 `<` 取首个） | ✓ 一致 |
| getValue（SearchTree，运行时实际） | MultiNoiseUtil.java:146-152 + SearchTree 树序 | —（C++ 用线性） | ⚠️ **平局时可能不同** |
| 本次是否平局 | — | C++ 温度距离 0 + 湿度距离 0 vs badlands 温 2 + 湿 47 → **非平局，压倒性 forest** | — |

---

## 3. 根因（文件:行 + 精确差异）

### 3.1 定性结论（高置信）
- **C++ 在 (812,-337)/(815,-337) 选点坐标的判定输入 ≠ vanilla**。从参照 biome=badlands + badlands 硬约束（温度≥0.55 **且** 湿度≤-0.1）可推出：**vanilla 温度 ≥5500、湿度 ≤-1000；C++ 温度 ≈5498（若选点坐标与 compdump 相近）、湿度 ≈-953**。
- **差量**：湿度 ≥0.0047（47 toLong）、温度 ≥0.0002（2 toLong）。两者都远超浮点舍入 ulp → **真实采样差**。
- **驱动分量**：湿度（vegetation）主导（差 47 vs 温度 2）。**修正 analysis.md 的「temperature 首要嫌疑」→ 湿度（vegetation）应升为首要嫌疑，temperature 次之。**

### 3.2 未能静态定位（诚实说明，置信度受此限制）
本 worker 逐行对拍了 §2 全部环节，**未发现任何静态可解释 ~0.0047 差异的 bug**。剩余可能性按概率排序：
1. **temperature/vegetation 噪声 sampler 的运行时 origin/permutation** 与 Java 不同（静态推导一致，但该分量从未运行时验证过；offset 噪声历史逐位一致只能证明 offset 的 octave 派生链，不直接证明 temperature/vegetation 的——虽机制相同，但需实证）。
2. **选点坐标确认缺失**：compdump 用 z=-337，判定用 z=pz<<2（-340 或 -336）。若 scout 的 -biomeDump 数据与判定路径一致则成立，但**选点 pz 未运行确认**（静态 biomePickCell 一致，但未实测选出的 (px,py,pz)）。
3. **spline double vs float**（density.h:791-812 vs Spline.java float）：影响 continents/erosion/ridges（temperature/vegetation 不用 spline）；历史 GRID 0 差异 → 低概率，但 continents 差 1.8e-4 的历史记载与该维度吻合。
4. find tie-break：非平局，排除。

### 3.3 具体文件:行（供运行时验证聚焦）
| 环节 | 文件:行 | 说明 |
|---|---|---|
| vegetation 分量采样 | worldgen_api.cpp:489（`samp("vegetation", p)`）/ 729-730 | **首要验证对象**（湿度差 47 主导） |
| temperature 分量采样 | worldgen_api.cpp:489（`samp("temperature", p)`）/ 729 | 次验（温度差 2） |
| vegetation 噪声 sampler | noise.h:249-261（构造）/ 269-274（sample） | 反射 Java getOrCreateSampler 对拍 origin/permutation |
| shift 链路（offset 噪声） | density.h:252-272 + getNoiseSampler("minecraft:offset") | 已历史逐位一致（docs/10 L463），但 (812,-337) 未复验 |
| FlatCache 查表 | density.h:681-702 | 负坐标 >>2 语义已对齐（本次确认） |

---

## 4. patch 建议（不修代码，供主会话）

### 4.1 运行时定位（第一步，决定性）
主会话补跑（本 worker 无执行权限）：
```
# C++ 侧（选点坐标 + 6 维分量 + biome）
block_probe.exe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks -biomeDump 812 73 -337
block_probe.exe ... -biomeDump 815 89 -337 / 815 90 -337
WG_COMPDUMP=1 WG_COMPDUMP_X=812 WG_COMPDUMP_Z=-337 block_probe.exe ...   # 注：z=-337 非选点坐标，仅供对比
# 需在 block_probe 增加/复用一个输出「biomePickCell 选点 (px,py,pz) + 选点后 6 维 float 值」的诊断（或读 surface.h biomeAtCached 的日志）

# Java 侧
java ... RouterProbe -Prouter.x=812 -Prouter.z=-337 -Prouter.yFrom=60 -Prouter.yTo=100 -Prouter.yStep=1（SURFBIOME：8 邻域选点后 6 维 + biome）
java ... RouterProbe -Prouter.x=815 -Prouter.z=-337 ...
```
对比项：选点 (px,py,pz)、选点后 temperature/vegetation/continents/erosion/depth/ridges 的 float 值、最终 biome。

### 4.2 若确认 temperature/vegetation 噪声值差
- 对拍 C++ `WG_B3DDUMP`（Perlin 输入/输出 %.17g）vs Java RouterProbe 反射 DoublePerlinNoiseSampler（各 octave origin/permutation/sample）。
- 重点：`noise.h:269-274` sample 的 `x·DOMAIN_SCALE` 与 octave 求和顺序；`noise.h:139-145` octave 派生；`random.h:30-32` createXoroshiroSeed。
- 修复方向：若发现 origin/permutation 差 → 查 RNG 派生链；若发现某 octave 求和顺序差 → 对齐 Java 循环。

### 4.3 若确认选点坐标差
- 复跑 biomePickCell 手算/加日志；比对 Java SURFBIOME 输出选点。
- 若选点一致但 compdump 的 z=-337 值误导 → 以选点坐标为准重采 6 维，可能发现 C++ 在选点坐标已跨过边界（假 bug）。

### 4.4 若六维全一致但仍不同 biome（最后手段）
- 复刻 Java SearchTree 遍历序（MultiNoiseUtil.java:379-604）到 C++ find，或确认 entries 顺序一致。低优先级（本次非平局）。

### 4.5 回归要求
任何修复必须全量回归 **-288 / 3200 / 20000 / 8576** 四套参照；3200 保持 100% 铁律零退化。

---

## 5. 风险提示（3200 铁律影响面）

### 5.1 温度/湿度链路修复对 3200 的影响判断
- **影响机制**：temperature/vegetation 是 2D 分量，**全局参与所有 biome 判定**。但 MultiNoise 最近邻判定对噪声值**不敏感**——只有落在 hypercube 边界（toLong 量化边界）± 若干单位的点才会翻转。远离边界的点，噪声值微小变化不改变最近 biome。
- **3200 当前 100%**：说明 3200 区域所有 biome 判定与 vanilla 一致（无边界翻转点）。
- **修复影响**：
  1. 若修复是把 C++ 噪声值**逐位对齐** vanilla（正确方向）：已一致的 biome 判定保持正确（值变正确但仍在正确侧）；只有恰好落在边界 ±Δ 的 3200 点可能翻转——但那些点在修复前若 C++ 与 vanilla 同侧则不翻，若不同侧则说明 3200 也隐含该 bug 但未暴露。**逐位对齐不会破坏已正确的判定**。
  2. **风险来源不是「修复本身」而是「修复方案引入新误差」**：若方案改了采样公式/顺序（非精确对齐），可能把正确值改成错误值 → 3200 新边界翻转。**必须零退化验证**。
- **结论**：温度/湿度链路修复对 3200 风险 = **低-中**。低：逐位对齐方向安全；中：全局参与 + 3200 是铁律参照 → 修复必须全量回归且 diff=0。

### 5.2 附带发现（非本任务主因，记录在案）
- **C++ maintainPrecision 潜在 bug（noise.h:127 `(long)` vs Java `lfloor`）**：`(long)(v/3.3554432E7+0.5)` 在 v/3.3554432E7+0.5 < 0（即 v < -16.78M）时向零截断 ≠ lfloor 负向取整 → 折叠结果差 1。**temperature/vegetation 采样坐标量级（block×2^10 ≈ 百万）不触发折叠，非本任务主因**；但这是与 vanilla 逐位对齐的隐患（极远坐标/极大 firstOctave 才暴露），建议后续独立修复。
- docs/10 L481「temperature 差 0.005（spline 放大）」是 Java comps（直接采样）vs 游戏实际（cns 查表）的内部实现差（L480 已注明），**不能**直接当作 C++ 与 vanilla 的差——本任务结论不依赖该记载。

---

## 6. 置信度与局限

- **高置信**：选点/坐标/shift/noise 公式/RNG/参数/cache/toLong 全部逐位一致（本次源码级复核）；badlands 湿度硬约束 ≤-0.1 从 biome_params.json 证实；C++ 湿度 -953 差 47 是判 forest 的压倒性驱动。
- **中高置信**：C++/vanilla 在选点坐标湿度差 ≥0.0047 或温度差 ≥0.0002（由参照 biome=badlands 反推，逻辑闭合）。
- **局限**：
  1. 无执行权限，未运行时验证 temperature/vegetation 噪声 sampler 值与选点坐标。
  2. compdump 的 z=-337 值 ≠ 判定实际用 z=pz<<2（-340/-336），±1-3 格噪声梯度可能改变具体 toLong，但不改变「贴边界 + vanilla 需温度≥0.55 且湿度≤-0.1」的定性。
  3. ridges(weirdness) 分量 compdump 未输出（唯一未知维）；若 vanilla 判 badlands 系还依赖 weirdness（badlands weirdness [-0.9333,-0.7666] 等），需运行时补齐。但湿度差 2209 已足以解释 forest 胜，weirdness 非必需。
- 状态：**draft**（精确到分量需 §4.1 运行时对拍；AI 不写 confirmed）。

---

## 7. 产物引用

- 本文件：`.artifacts/8576-24blocks/biome-fix/analysis2.md`
- 前序：`.artifacts/8576-24blocks/biome-fix/analysis.md`（第 1 步，其中 humidity range 误判由本文件 §1 修正）
- 相关：`.artifacts/8576-24blocks/biome-terracotta/analysis.md`（scout 实测 C++ forest / 参照 badlands）、`.artifacts/8576-24blocks/followup/analysis.md`（biome 判定修复史、21 块插值边界翻转）
- 数据：`E:\tmp\compdump_812_-337.txt`；`versions/1.20.1/data/biome_params.json`（forest L238-259/L378-399/L516-541/L1182-1225、badlands L1262-1295、wooded_badlands L684-727/L1340-1381/L1896+、eroded_badlands L4310+）
- 源码：Java `BiomeAccess.java`、`ChunkRegion.java`（L102 seed 来源）、`ChunkNoiseSampler.java`（L190-198 createMultiNoiseSampler、L836-881 FlatCache）、`MultiNoiseUtil.java`（L222-235 sample、L66-68 toLong）、`DensityFunctionTypes.java`（L739-779 Noise、L916-1025 Shift/ShiftA/ShiftB/ShiftedNoise）、`DoublePerlinNoiseSampler.java`、`OctavePerlinNoiseSampler.java`、`PerlinNoiseSampler.java`、`RandomSeed.java`、`Xoroshiro128PlusPlusRandom.java`、`Xoroshiro128PlusPlusRandomImpl.java`、`noise_settings/overworld.json`（L248-265）；C++ `biome.h`（L76-84/94-149/152/221-234）、`noise.h`（L28-275）、`xoroshiro.h`、`random.h`、`density.h`（L209-272/629-744/791-812）、`density_builder.h`（L16-20/100-118/215-230/295-304）、`worldgen_api.cpp`（L74-75/356-376/464-497/625-652/714-737）
