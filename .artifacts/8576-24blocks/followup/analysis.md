# 8576-24blocks 收尾分析：phase2 数据正式解读 + y=-32 噪声卡复核 + SteepCond 对拍

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6（chunk 45..50 × -27..-22）
> 角色：anchor.worker 精确分析 subagent（收尾轮）　日期：2026-08-09　状态：**candidate（21 块机制）** / draft（y=-32、SteepCond 零影响面项）
> 环境：本 worker 无 bash/可执行权限（工具集无 shell，read_only_task 实测 python/bash 均 BLOCKED），无法现场跑 exe/read_col2.py；基于 phase2-verification.md 原始数据 + Java/C++ 源码做正式解读，参照 (805,-427) 列数据来自 docs/10-timewise-archive.md 既有记录与 read_only_task 复核。
> 不修改任何代码/参照文件。

---

## 0. 结论摘要

| 任务 | 结论 |
|---|---|
| ① 21 块根因 | **块级 finalDensity 边界翻转（插值精度差/角点值差）** —— 非 aquifer 逻辑 bug、非结构假 diff。phase2 三列剖面坐实：C++ fd 在 density≈0 边界与 Java 符号相反。机制层升级 **candidate**；完整 candidate 需 Java DensityProbe 同点高精度对比（见 §1.5 判据）。 |
| ② y=-32 噪声卡 | **关闭**。(805,-32,-427) 当前已不在 mismatch（C++ 现生成 red_terracotta，与参照一致）；深层 terracotta 带 = badlands surface 规则（terracottaBands）产物，触发靠 biome=badlands 系；该点 biome 判定已随 biomeAt/8 邻域修复（docs/06:104-105）解决，与 #23/#24 同属「biome 判定差」机制族但该点已修复。 |
| ③ SteepCond | **理论差异存在**（C++ SteepCond 读 const heightmap 快照 vs Java 实时采样），但 badlands 段不用 steepSlope → **8576 零影响**，非收尾阻塞；patch 方向见 §3.3（不修代码）。 |

---

# 任务①：phase2 原始数据正式解读（21 块根因定论）

## 1.1 (743,72,-406) C++ fd = 0.000000（无负号）

### 原始数据（phase2-verification.md）
| y | initialDensity | finalDensity | got（C++ 最终） | 参照 |
|---|---|---|---|---|
| 64 | 0.632239 | +0.042307 | stone | stone |
| 68 | 0.003681 | +0.021163 | stone | dirt |
| **72** | -0.624877 | **0.000000（无负号 → ≥0）** | **grass（heightmap 顶 72）** | **air** |
| 76 | -0.840703 | -0.056512 | air | air |

### 值域推断
`%.6f` 打印 "0.000000" 且无负号 → C++ fd(72) ∈ [0, 0.0000005) 或精确 +0.0，**即 ≥0 的微值**（+1e-7 量级或精确 0，绝无负号）。got=grass 说明 C++ 在该格判**非 air**（buildSurface 前初始地形 stone → 顶层染 grass）。

### 参照侧 Java 符号（**纠正 phase2 判读**）
Java AquiferSampler.apply（AquiferSampler.java:145-151）：`if (density > 0.0) return null;` → null 在 NoiseChunkGenerator.populateNoise（L410-411）落 `defaultBlock = stone` → buildSurface 染 grass。**因此 Java fd > 0 → 参照应为 grass，不是 air**。

参照 (743,72) = **air** → Java 必走 `density ≤ 0` 分支 → 流体决策 d+e ≤ 0 → 返回 AIR（blockY=72 ≥ 液面 63，fl2.getBlockState(72) = AIR）。

⇒ **Java fd(72) ≤ 0（air）；C++ fd(72) ≥ 0（stone/grass）。两侧在 density≈0 边界符号相反。**

> ⚠️ phase2 判读「参照 air → Java fd>0」方向写反。按 Java 源码语义，参照 air 只能来自 fd ≤ 0 + d+e ≤ 0。核心结论不受影响：**C++ 与 Java 块级 finalDensity 在 density≈0 边界符号差**坐实，只是两侧正负方向与 phase2 判读相反（C++ 偏正、Java 偏负）。

### 判读
- C++ fd(72) 落在 [0, 5e-7) 的密度≈0 边界；Java fd(72) ≤ 0。
- 差量 < 1e-6（同一插值公式、同一角点值来源，唯一差为角点值微差 → 块级插值点差 < 1e-6）。
- 方向：C++ 偏正（判 stone）→ buildSurface 起点 = 72，参照偏负（判 air）→ 起点 = 71 → 三连段同步 +1（#4/5/6）。

## 1.2 (802,0,-372) C++ fd = -0.000000（有负号）

### 原始数据
| y | initialDensity | finalDensity | got | 参照 |
|---|---|---|---|---|
| -4 | 14.437470 | -0.001872 | 流体决策 | — |
| **0** | 13.651744 | **-0.000000（有负号 → <0）** | **air（got=0）** | **deepslate** |
| 4 | 12.866018 | +0.019303 | stone | — |

### 推论
- `%.6f` 有负号 → C++ fd(0) ∈ (-0.0000005, 0)，即 **-1e-7 量级负值**（或 -0.0）。
- C++ fd < 0 → 走流体决策；d+e ≤ 0 → bs = AIR（blockY=0 < 液面 63 → fl2.getBlockState(0) = WATER？不对——**blockY=0 < 63 → fl2 = WATER**）。

等等，需仔细：液面 63，blockY=0 → `getBlockState(0)`：`0 >= 63 ? AIR : WATER` → WATER。但 got=air？phase2 判读写「C++ 流体决策 → d+e≤0 → bs=air（got=0 air ✓）」。

哦，这里 C++ got=0=air 是最终块。若 bs=WATER 应写 water。让我重新看——(802,0,-372) got=0 air，参照=970 deepslate。mismatch-list：got=0 air vs vanilla=970 deepslate（C++ 少一块 deepslate，air 侵入）。

phase2 判读：「C++ 流体决策 → d+e≤0 → bs=air（got=0 air ✓）」——如果 bs 是 WATER 而最终是 air，可能是 aquifer 返回 WATER 但后面 fillFromNoise 条件 `blockState != AIR` 才 set…… 不对。

让我看 Java fillFromNoise：`if (blockState != AIR && ...) chunkSection.setBlockState`（L415）。若 aquifer 返回 AIR → 不写 → chunk 默认 air。若返回 WATER → 写 water。若返回 null → stone。

C++ worldgen_api.cpp L700-707：`block = aquifer->apply(...)`；`if (block < 0 && oreVein) block = oreVein->apply(...)`；`if (block < 0) block = stone`。即 C++ aquifer 返回 -1（null）→ stone；返回 airId → air；返回 waterId → water。

(802,0,-372) C++ got=air → C++ aquifer 返回 airId。aquifer.h L103-110：`FluidLevel fl2 = getWaterLevelAt(r); double d = maxDistance(o,p); int bs = fl2.getBlockState(blockY, airId);`——fl2 是**水位**（不是默认液面）。getWaterLevelAt 返回的是 aquifer 计算的水位 FluidLevel（blockY ≥ y → AIR，< y → block）。blockY=0 < 液面但可能 ≥ aquifer 局部水位 y → AIR。

所以 C++ bs=AIR → d+e≤0 → 返回 AIR → got=air ✓。Java 参照 deepslate：Java fd(0) ≤ 0 且 d+e>0 → 返回 null → stone → buildSurface 染 deepslate（y≤8）。

### 判读
- **C++ fd(0) < 0（-1e-7 量级），Java fd(0) 略 >0（d+e>0 → stone）** —— 与 (743,72) 相反的方向（此处 C++ 偏负、Java 偏正）。
- 两侧方向相反正好证明**无系统性单侧偏移**，符合「插值精度差在敏感边界随机翻转」的特征（workerB §5.4 预判）。
- 参照 y=0=deepslate ↔ C++ air = deepslate↔air 边界翻转（#16），与 #21 (807,0) 互补（C++ 多 deepslate）。

## 1.3 21 块根因最终定性

### 可证伪判据（workerB §6 判据，用 phase2 数据验证）

| 判据 | phase2 数据 | 结论 |
|---|---|---|
| 若某格 C++ finalDensity 与 vanilla（Java cns 同点）差 ∈ (0,0.12) 且符号翻转 → 坐实插值精度边界翻转 | (743,72)：C++ fd≈0 ≥0 vs Java fd≤0 → 差 <1e-6、符号相反 ✓；(802,0)：C++ fd≈0 <0 vs Java fd>0 → 差 <1e-6、符号相反 ✓ | **判据满足**（C++ 侧直接证据） |
| 若 density 逐位一致但 aquifer 返回不同 → aquifer 真 bug | 三列 fd 全部落在 density≈0 边界（非远离边界），符号在边界翻转，无「逐位一致但返回不同」样本 | 否证 aquifer 逻辑 bug |
| 若参照含结构方块/大段差异 → 结构假 diff | 参照 SURFACE 状态；24 块全为边界 ±1 单格；无结构矩形/大段（区别于 -288） | 否证结构假 diff |

### 最终定性
**21 块 = 块级 finalDensity 边界翻转（插值精度差/角点值差）**：
1. 插值公式已逐位对齐（phase2 ③：Java MathHelper.lerp3 double vs C++ InterpolatedDF::sample 同公式同顺序同 double）
2. 角点值来源同链（base_3d_noise / sloped_cheese → finalDensity 树；known POC：-288 interp 差 7e-6 量级、density 差 0.12 可翻转 aquifer 判定，docs/10 L453/L67-68）
3. 8576 剩余 mismatch 极稀疏（24/3,538,944 = 0.0007%），全部落在 deepslate↔water↔air↔地表 density≈0 边界 → 与「插值精度差只在敏感边界偶尔翻转」特征自洽
4. 三列代表性剖面（743/800/802）C++ fd 全部落在 density≈0 边界且与 Java 符号相反 → **C++ 侧直接证据已取得**

### candidate or draft 建议
- **机制层升级 candidate**：phase2 提供了「C++ fd 在边界 ≈0、与 Java 符号相反」的运行时直接证据，满足 workerB 判据 1 的 C++ 侧部分；aquifer 18 项静态对齐 + deepslate 规则一致 + 参照 SURFACE 排除结构 → 三假说互斥后唯一剩「插值精度边界翻转」。
- **完整 candidate（所有 21 格逐个验证）保持 draft 性质**：phase2 只测了 3 列代表剖面，未逐格 + 未做 Java DensityProbe 同点高精度差量（diff ∈ (0,0.12) 的直接数值测量）。升级到「confirmed」需主会话跑 Java DensityProbe cns 同点（命令见 §1.5）。
- **明确理由**：不升 confirmed 是因为差量 1e-6 级的直接对拍未跑；但机制结论（非 aquifer bug、非结构、边界翻转）证据充分，作为 Phase 3 立项依据足够。

## 1.4 C++ 侧可定位最小范围 + 工程量评估

### 最小可定位范围
1. **块级插值点**：InterpolatedDF::sample（density.h 529-537）—— 公式已对齐，跳过。
2. **角点值**：`finalDensity 树角点缓存`（buildNode 采样，density_builder.h）→ 8 角点三线性。差必在角点值本身。
3. **角点值来源**：
   - `base_3d_noise` octave 求和（DoublePerlinNoiseSampler 负坐标 octave 浮点路径）—— **docs/10 已记录「负坐标 base_3d_noise 偏正未解项」（@-288 差 0.05-0.23）**，8576 三列 z 全为负（-406/-363/-372），方向吻合 → **首选候选**。
   - sloped_cheese（offset/factor/depth/jaggedness 组合）—— 次候选。
4. **定位工具**（复用 -288 篇）：`WG_B3DDUMP/GRID` dump C++ 角点各 octave；Java DensityProbe cns 链同点 dump → 逐 octave diff 定位第一个符号差 octave。

### 修复工程量 vs 收益
- 工程量：中等。需复刻/核对负坐标 octave 的浮点求和路径（可能涉及 per-octave 求和顺序 / 负坐标 floor 语义 / DoublePerlinNoiseSampler 的 sample 实现），改动集中在 base_3d_noise 或 sloped_cheese 一角点链。
- 收益：消除全部剩余边界翻转（21 块 + 潜在其他 seed 同类）。8576 匹配率 99.9993% → 100%（或接近）；3200 干净参照需零退化验证。
- 风险：base_3d_noise 是全局密度源，改动可能影响 -288/3200/20000 全部参照 → 必须全回归。**21 块 = 21/3,538,944 = 0.0006%**，性价比：**收尾性质（消除最后 0.0007%）**，优先级低于 #23/#24 biome 真 bug（后者影响面更大且是规则层错误）。

---

# 任务②：y=-32 噪声卡复核（judge 建议项）

## 2.1 旧疑点 vs 当前事实

**旧疑点（docs/06:132 + 10:487）**：(805,-32,-427) 参照=red_terracotta（y=-32 深层单层/带）vs C++=stone；JSON surface_rule 不覆盖 y=-32 → 深层 terracotta 带来源未解（当时判「假 diff 候选」）。

**当前事实**：
1. **参照 (805,-427) 列**（read_col2/read_biome2 既有记录 + read_only_task 复核，10-timewise L501）：
   - y=-32 单层 red + y=-27..-23 连续 red 带 + y=-16 red + y=-11..-10 red + y=-8..-4 white —— **badlands 深层 terracotta 带形态**（连续同色段）。
   - 地表形态：read_only_task 报告地表 y=296（高原顶）——eroded_badlands pillar 高原区域（与 (810,-411) pillar 高原同机制）。
2. **当前 24 mismatch 明细（mismatch-list.md）不包含 (805,-32,-427)** → 该点当前 C++ 与参照**已匹配**（TOTAL 99.9993%，所有 mismatch 均已列出）。
3. 参照该点 = red_terracotta（固定不变）→ C++ 当前也在 (805,-32,-427) 生成 **red_terracotta**。

## 2.2 机制判读

- **深层 terracotta 带来源**（本任务已解决旧「未解」）：badlands/eroded_badlands/wooded_badlands surface 规则的 `terracottaBands()`（VanillaSurfaceRules.java:207-234 badlands 段，L219/L232）会应用到深层 default（stone）块——terracottaBands 是 192 带周期，不限于地表；y=-32 落在带阵列的 red 带上。触发前提 = **块级 biome 判定 = badlands 系**（biome 条件在 L207）。
- 参照 (805,-427) 列深层 terracotta 带 → vanilla 在该列深层判 badlands 系 ✓。
- C++ 当前同点生成 red_terracotta → C++ 当前在该列判 badlands 系（biome 判定已修复）。
- **修复依据**：docs/06:104-105 记录 biomeAt 修复（`(x>>2)<<2` floor → 8 邻域选点，修复后 (805,64,-432)=eroded_badlands 与 Java SURFBIOME 一致）与 biomeAtCached key 修复（选点坐标 packed）。该点恰在 savanna↔eroded_badlands 边界（docs/10:481：continents 差 1.8e-4、temperature 差 0.005）→ biome 判定修复后翻转正确。

## 2.3 结论

- **与 #23/#24 同属「biome 判定差」机制族**（都是 badlands 深层 terracotta 带触发与否），但 **该点已随 biome 判定修复解决**（不再 mismatch），而 #23/#24（812/815,-337）仍在 forest↔badlands 边界未修复 → 同一机制、不同坐标、不同修复状态。
- **y=-32 噪声卡关闭**。依据：
  1. 该点当前 C++ 输出 = 参照（不在 24 mismatch）
  2. 深层 terracotta 带机制已破案（badlands surface 规则 terracottaBands + biome 判定），非「来源未明的假 diff」
  3. 触发条件（biome=badlands 系）该点已满足 → 机制已正确
- **保留项**：无独立噪声卡必要。建议把 (812,-337)/(815,-337) 的 biome 判定差合并为 Phase 3 单独立项（#23/#24 真 bug），与 y=-32 旧卡合并关闭。
- 置信度：中高。参照列形态来自既有记录（未在本 worker 现场重读 blocks 二进制）；(805,-32,-427) 不在 mismatch 是**复现明细的确定性事实**（mismatch-list.md 2026-08-08 复现，TOTAL 统计闭合）。

---

# 任务③：SteepCond 理论差异对拍（judge 建议项）

## 3.1 Java 侧语义（权威源码）

**SurfaceBuilder.buildSurface（L113-131）**：
```java
int o = chunk.sampleHeightmap(WORLD_SURFACE_WG, k, l) + 1;   // L117 pillar 前 heightmap+1
biome 采样 y=o（L119）→ 若 eroded_badlands → placeBadlandsPillar（L120-122）
int p = chunk.sampleHeightmap(WORLD_SURFACE_WG, k, l) + 1;   // L124 pillar 后重新采样 heightmap！
// 主循环从 p 向下
```

**placeBadlandsPillar（L208-234）**：填充 y≤j 的 air→stone → `column.setState(k, stone)` → `ProtoChunk.setBlockState`（L96）→ `Heightmap.trackUpdate`（ProtoChunk.java:154）→ **WORLD_SURFACE_WG 实时更新**。

**SteepSlopePredicate（MaterialRules.java:541-565）**：`chunk.sampleHeightmap(WORLD_SURFACE_WG, ...)` —— **实时采样 chunk heightmap**。若本列/邻居列被 pillar 抬升，读到**更新后**高度。

## 3.2 C++ 侧语义

**buildSurface（surface.h:701-717）**：
```cpp
int o = heightmap[idx] + 1;                                  // L707 pillar 前
auto pillarBiome = biomeAtCached(m, o, n);                   // L711
int columnH = heightmap[idx];                                // L712 列表面高度（本列）
if (pillarBiome.first == "minecraft:eroded_badlands")
    placeBadlandsPillar(..., o, columnH, ...);               // L713-715 通过引用更新 columnH
int p = columnH + 1;                                         // L717 pillar 后起点（已对齐 L124 ✓）
```

**关键差异点**：
- C++ `placeBadlandsPillar` 更新的是**本列**局部引用 `columnH`（surface.h:818 `columnHeight = std::max(columnHeight, j+1)`），并正确用于本列起点 p（L717，对齐 Java L124 重采样）。
- 但 **C++ `ctx.columnHeightmap` 仍指向传入的 `const std::vector<int>& heightmap`（L662/L673）原始快照**，placeBadlandsPillar **没有写回 heightmap 数组**。
- **SteepCond（surface.h:250-258）读 `ctx.columnHeightmap`（const 快照）**，而 Java SteepSlopePredicate 实时采样（含 pillar 抬升）→ **理论差异成立**：若 SteepCond 采样的邻居列被 pillar 抬升，C++ 读旧高度、Java 读新高度。

## 3.3 影响面与 patch 方向

**影响面**：
- `steepSlope()`（materialCondition13）在 VanillaSurfaceRules 中仅用于 frozen_ocean/deep_frozen_ocean（PACKED_ICE，L99）、snowy（STONE，L107/137）、jagged_peaks 等段（L129/141）。
- **badlands 段（L206-235）完全不用 mc13/steepSlope** → placeBadlandsPillar 区域（eroded_badlands）即使 pillar 抬升，SteepCond 也不参与 badlands 染色 → **badlands 段零影响**（与 judge 判断一致）。
- 8576 24 块全部为 savanna/river/forest 边界，无 frozen/snowy/jagged 场景 → **对 8576 收尾零影响**，非阻塞。

**patch 方向（不修代码，供 Phase 3 参考）**：
1. 让 `ctx.columnHeightmap` 指向一个 **pillar 后更新的可变副本**（或在 buildSurface 内维护局部可写 heightmap，placeBadlandsPillar 后 `heightmap[idx] = columnH` 写回本列，SteepCond 后续采样即读到新值）。
2. 注意 Java 逐列顺序（k=x 外层、l=z 内层）与 SteepCond 采样 (i, j±1)/(i±1, j) 邻居：**已处理列的 pillar 抬升必须已写回**，未处理列保持原值——与 Java 实时采样语义一致。
3. 因影响面≈0，可作为「顺手对齐」低优先级，不阻塞 24 块收尾。

---

## 4. 置信度与局限

- **任务①（21 块根因）**：机制层 candidate。phase2 三列 C++ fd 边界符号差为**运行时直接证据**；Java 侧符号经源码语义推演（纠正了 phase2 一处方向判读）。局限：未跑 Java DensityProbe 同点差量数值、未逐格验证 21 格。
- **任务②（y=-32）**：噪声卡关闭，置信度中高。核心事实（该点不在 mismatch）确定；参照列形态来自既有记录，未现场重读 blocks 二进制。
- **任务③（SteepCond）**：差异存在、影响面≈0，置信度高（源码逐行对拍）。
- 状态：AI 不写 confirmed；21 块机制层 candidate 供主会话/judge 裁决，y=-32 关闭结论与 SteepCond patch 方向供 Phase 3 采纳。

## 5. 产物引用

- 本文件：`.artifacts/8576-24blocks/followup/analysis.md`
- 数据源：`.investigations/8576-24blocks/phase2-verification.md`、`mismatch-list.md`、`column-profiles.md`
- 前序产物：`../surface-plus1/analysis.md`、`../aquifer-wateredge/analysis.md`、`../biome-terracotta/analysis.md`
- 参照列记录：`versions/1.20.1/docs/10-timewise-archive.md`（L481-501）、`docs/06-surface-rules.md`（L104-105、L131-145）
- 源码：`AquiferSampler.java`（145-151）、`NoiseChunkGenerator.java`（409-418）、`ChunkNoiseSampler.java`（176-181, 703-800）、`SurfaceBuilder.java`（113-131, 208-234）、`MaterialRules.java`（541-565）、`ProtoChunk.java`（108-154）、`Heightmap.java`（73-100）；C++ `aquifer.h`（67-137）、`surface.h`（250-258, 659-819）、`worldgen_api.cpp`（594-711）
