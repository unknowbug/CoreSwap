# discovered/algorithm-fingerprints — 已确认的算法/协议指纹（跨版本通用）

> 从 versions/1.20.1/docs/ 与排查中提炼的 MC 算法特征。写入格式见 knowledge/INDEX.md。

## 发现 #1: weird_scaled_sampler 的 rarity_value scale 语义

**发现时间:** 2026-08-08
**发现者:** worker（spaghetti_2d 修复）
**来源定位:** MC 1.20.1 density_function JSON（caves/spaghetti_2d 等）
**置信度:** confirmed（修复后 8576 98.67% → 99.60%）
**module:** re-code

### 观察
`weird_scaled_sampler` 的 scale 处理：C++ 曾把 rarity_value 的缩放语义实现错（差 1.5 倍）→ weird 值差（0.3701 vs 0.0679）→ spaghetti_2d 差 → entrances 差 → when_out_of_range 差 → 8576 块状。

### 证据
- 修复链记录（10 时间线）：scale 错 1.5 → weird 值差 → 8576 块状

### 如何利用
- 还原 weird_scaled_sampler 时先确认 Java 的 rarity/scale 公式（`rarityValue * (x/z scale)` 还是 `x/z scale / rarityValue`——按 Java 源码）
- 相关文件：caves/spaghetti_2d、caves/entrances、caves/noodle_ridge 等

## 发现 #2: final_density 树的 range_choice 阈值（sloped_cheese 1.5625）

**发现时间:** 2026-08-08
**发现者:** worker（8576 finalDensity 排查）
**来源定位:** overworld.json final_density / sloped_cheese.json
**置信度:** candidate（分支翻转敏感区已定位，具体差未最终确认）
**module:** re-code

### 观察
`final_density = min(squeeze(0.64 × interpolated(blend_density(0.1171875 + yclamp×(...range_choice(sloped_cheese, when_in=min(sloped_cheese, 5×entrances), when_out=...))))), noodle)`。
- range_choice 阈值：**sloped_cheese = 1.5625**
- when_in = min(sloped_cheese, 5×entrances)——**entrances 噪声在角点约束 when_in**（如 (808,72,-412) sloped_cheese=0.398 但 5×entrances=0.147 → when_in=0.147）
- squeeze(x) = clamp(x,-1,1) 后 `d/2 - d³/24`（C++ 实现，与 Java 需核对）
- y=72 时 initialDensity 树退化为 `0.1171875 + yclamp×(...) = when_in`（yclamp(-64,-40)=1、yclamp(240,256)=1）

### 证据
- C++ GRID 角点 dump：cacheId=12（init 树 interpolated）8 角点 = 0.1471/0.0522/-0.2658/-0.3200/0.0476/-0.0854/-0.3258/-0.3774；三线性 initInterp=-0.1188（fx=0.5,fy=0.5,fz=0.25）
- 0.64×initInterp=-0.076 与 densityDump -0.038 差 2 倍（squeeze 语义未最终核对）

### 如何利用
- 1.20.1 final_density 是 range_choice(sloped_cheese) 结构，**不是** 1.18/1.19 的 `-0.703125 + 4×qneg(depth×factor)` clamp 公式——跨版本迁移时必须换
- est 用 noise_router.initial_density_without_jaggedness（clamp 公式），方块判定用 final_density（range_choice 公式）——**两个树不同，别混用**
- 分支翻转区（sloped_cheese 接近 1.5625）是插值敏感区，角点值微差会导致地形完全不同

## 发现 #3: InterpolatedDF 网格结构（4×4×8 cell）

**发现时间:** 2026-08-08
**发现者:** worker（density.h 实现）
**来源定位:** density.h InterpolatedDF / Java ChunkNoiseSampler
**置信度:** confirmed
**module:** re-code

### 观察
InterpolatedDF 以 chunk 为单位建 5×49×5 网格（x/z 每 4、y 每 8，含边界）；插值在 interpolated 内容（如 blend_density(init 树)）角点，**外层组合（0.64×、squeeze、min、noodle）在插值后计算**；noodle 树内有 4 个独立 interpolated（noodle/thickness/ridge_a/ridge_b）。

### 证据
- C++ density.h 与 Java ChunkNoiseSampler 结构一致（-288/3200 逐位验证）

### 如何利用
- 三线性只对 interpolated 内容做，不能对 finalDensity 角点做（本 session 手算踩过）
- 垂直网格 y 步 8：cellY=floorDiv(y+64,8)，fy=(y+64)%8/8

## 发现 #4: placeBadlandsPillar（eroded_badlands 支柱 air→stone 填充）

**发现时间:** 2026-08-08
**发现者:** worker（8576 terracotta 带破案）
**来源定位:** Java SurfaceBuilder.placeBadlandsPillar L208-234（mc_src_extract）
**置信度:** candidate（block_probe 实测闭环：8576 99.9993%、3200 零退化）
**module:** re-code

### 观察
eroded_badlands 每列在 buildSurface 规则应用前，先用 2D 噪声算 pillar 顶 j = 64 + min(e²·2.5, ceil(h·50)+24)（e=min(|badlands_surface(x,0,z)*8.25|, badlands_pillar(x*0.2,0,z*0.2)*15.0)，h=|badlands_pillar_roof(x*0.75,0,z*0.75)*1.5|），表面≤j 时把 y≤j 的 **air 先填成 stone** → heightmap（WORLD_SURFACE_WG）抬升到 j+1 → 主循环起点变高（j+2）→ badlands 段规则（blockY+q≡j+1 恒真）→ terracottaBands 染色。

### 证据
- NOISE 阶段 (810,76,-411)=air、SURFACE 后=terracotta（Diag810 实测）
- 修复后 chunk(50,-26) 797 块差异全消（8576 99.9993%）
- 3 个噪声：badlands_surface（原始坐标×8.25）/ badlands_pillar（×0.2）/ badlands_pillar_roof（×0.75），y 采样恒 0.0

### 如何利用
- **C++ buildSurface 对 air 跳过规则是「缺前置步骤」的信号**——某 biome 在 NOISE 是 air 但 SURFACE 有方块时，先查 Java 的 pillar/iceberg 类前置填充（placeIceberg 是 frozen ocean 同类，同样缺失）
- 跨版本：1.18+ 都有 badlands pillar（1.20.1 公式 64+min(...)）
- 3200 参照污染教训：**server level-seed 固定 8576 时，BlockProbe 重导的 blocks 文件是 8576 世界**——不能只看文件名/header 的 benchSeed，导出后核对 worldSeed

## 发现 #5: StructureWeightSampler（Beardifier）算法指纹——24³ 权重表 + fastInverseSqrt 位操作 + sample 四分支

**发现时间:** 2026-08-09
**发现者:** worker（Beardifier 实现，-288 海底边界闭合）
**来源定位:** MC 1.20.1 StructureWeightSampler.java / C++ beardifier.h（versions/1.20.1/cpp/worldgen/src/beardifier.h）
**置信度:** confirmed（t_beard3 17/17 逐位一致 + -288 闭合 +10777 块 + 8576/3200 零退化）
**module:** re-code

### 观察
Beardifier 是结构密度修正（StructureWeightSampler）：`ChunkNoiseSampler.getActualDensityFunction` L469-470 把 `Beardifier.INSTANCE`（恒 0）替换为真实 beardifying → density 链 = CellCache.add(DensityInterpolator(finalDensity), Beardifier)。算法四要点：
- **24³=13824 float 权重表**：惰性预计算 `(float)Math.pow(Math.E, -squaredMagnitude(x, y+0.5, z)/16)`——**Java 用 Math.pow(Math.E,...)（fdlibm pow 通用路径）不是 Math.exp**；表索引 `TABLE[k*576 + i*24 + j]`（k=z+12, i=x+12, j=y+12，越界 0）
- **getMagnitudeWeight = clampedMap(magnitude(x, y/2, z), 0, 6, 1, 0)**——clampedLerp 链（getLerpProgress=(v-s)/(e-s) 不 clamp → clampedLerp 才 clamp）
- **getStructureWeight = 表查找 + `-d * fastInverseSqrt(e/2)/2` 因子**（d=yy+0.5, e=squaredMagnitude(x,d,z)）
- **fastInverseSqrt 位操作**：`l = 6910469410427058090LL - (l >> 1)`（Double.doubleToRawLongBits → 有符号算术右移 1 → longBitsToDouble）→ `x*(1.5 - 0.5*x_orig*x*x)` Newton 一步
- **sample 四分支**（按 piece.terrain）：NONE 跳过 / BURY 累加 getMagnitudeWeight / BEARD_THIN·BEARD_BOX 累加 getStructureWeight×0.8；junction 循环累加 getStructureWeight(r,l,m,l)×0.4；垂直项 q 按 terrain：BURY/BEARD_THIN = blockY-o、BEARD_BOX = max(0, max(o-blockY, blockY-maxY))
- piece 记录：box（minX/minY/minZ/maxX/maxY/maxZ 含边界）+ terrain（StructureTerrainAdaptation 序数 NONE=0/BURY=1/BEARD_THIN=2/BEARD_BOX=3）+ groundLevelDelta

### 证据
- t_beard3：(-244,50..66,-256) C++ vs Java BEARD-244 **17/17 逐位一致**（.investigations/-288-unclosed/cmd-output/t_beard3_run.txt）
- block_probe -288：TOTAL 95.7379%→96.4221%（+10777 块闭合，MISMATCH 67039→56275，.investigations/-288-unclosed/cmd-output/bp288_beard_run.txt）
- 结构布局参照：.investigations/-288-unclosed/cmd-output/beard_m288.txt（16 chunks：135 pieces + 506 junctions）
- verdict：.investigations/-288-unclosed/beardifier-verdict.md（density 链等式 8/8 点 ≤3e-6 闭环）

### 如何利用
- **跨版本迁移（1.18/1.19）**：
  - piece 记录字段与 terrainAdaptation 枚举**序数**各版本可能不同——迁移时 diff 各版本 StructureWeightSampler.java 的 Piece 字段 / StructureTerrainAdaptation 序数（按序数传，别硬编码字符串名）
  - 结构布局数据走「vanilla 同源构造（createStructureWeightSampler）+ 反射提取」喂入——**不需要复刻 Java 结构生成器**，跨版本只改构造入口
- **C++ 移植硬约束**：
  - `Math.pow(Math.E,...)` 不能换成 `Math.exp`（数学等价但 fdlibm pow 路径位级可能差 1 ulp）——C++ 用 `std::pow(M_E,...)` 或字面量 2.718281828459045
  - 权重表是 **float 截断**（`(float)` 强转）：先 double 算完整再截 float，不能全程 double
  - fastInverseSqrt 的 **MSVC long=32 位坑**：必须 int64_t/long long（Java long 有符号算术右移 >>）
  - clampedMap 链两步不能合并：getLerpProgress 不 clamp、clampedLerp 才 clamp
- **判定用**：`Beardifier.INSTANCE.sample()` 恒 0 但 cns 替换为真实 beardifying——**只看静态实现会漏掉真实 Beardifier**（density 链必须看 getActualDensityFunction 的替换逻辑）

## 发现 #6: Java stream.flatMap 惰性 → positions 链必须深度优先（BFS 消费 RNG 错序）

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 3 ore 定位）
**来源定位:** MC 1.20.1 `feature/PlacedFeature.java:44-63`（generate）+ `.investigations/feature-pipeline/pipeline-map.md` §2.2
**置信度:** confirmed（2026-08-10 已验证；-288 FULL 96.67%、300515 FULL 96.59%、granite 56.2%→修复后 phase3.5 88.3%）
**module:** re-code

### 观察
Java `PlacedFeature.generate` 用 `Stream.flatMap` 串联 placement modifiers：
```
Stream<BlockPos> stream = Stream.of(pos);
for (PlacementModifier pm : placementModifiers)
    stream = stream.flatMap(p -> pm.getPositions(context, random, p));
```
`flatMap` 是**惰性**的：对当前位置 p 立即完整执行该 modifier 的 getPositions 得到子位置流，再递归展开到叶子，然后才处理下一个兄弟位置 —— 即 **深度优先**。且所有 placement modifier 与 feature 内部共用**同一个 `chunkRandom`（已 setDecoratorSeed）RNG 流**。因此 positions 链的遍历顺序直接决定 RNG 消费顺序 → 决定每个位置。

C++ 初版用「vector 先收集全部位置、再逐层展开」的 **BFS**：同一个 count/in_square/height_range 链，BFS 与 DFS 的 RNG 消费顺序不同 → 位置全部错 → ore 球体位置不重合（granite 匹配率仅 56.2%）。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase3_ore_result.txt`：`root cause = BFS vs DFS positions chain`；-288 FULL 96.67%、300515 96.59%、granite 56.2%
- `pipeline-map.md` §2.2（PlacedFeature.generate flatMap 语义 + 共用 RNG 流）+ §2.4（PlacedFeatureIndexer 决定 setDecoratorSeed 的 index）
- 修复后 phase35_crosschunk_result.txt：granite 88.3%、diorite 85.7%、tuff 87.8%、dirt 92.7%

### 如何利用
- 复刻 positions 链：用**递归/栈式 DFS**（当前位置 → 立即跑完整个 modifier 链到叶子 → 下一个），不要「收集再展开」BFS
- 跨版本通用：1.18/1.19/1.20.x 的 PlacedFeature.generate 都是同一 flatMap 惰性模式；凡「一个 modifier 输出 0..n 个位置、下一个 modifier 再变换」的链都是 DFS
- RNG 必须贯穿 positions 链与 feature.generate 全程同一个流，中途不得重建/重播种

---

## 发现 #7: carver 随机基类是 CheckedRandom（48 位 LCG），不是 Xoroshiro——FEATURES 才是 Xoroshiro

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 2 carvers）
**来源定位:** MC 1.20.1 `util/math/random/CheckedRandom.java` / `ChunkRandom.java setCarverSeed L87-93` / `carver/CaveCarver.java carveTunnels L124-219`；C++ `versions/1.20.1/cpp/worldgen/src/chunkrandom.h`、`carver.h`
**置信度:** confirmed（2026-08-10 已验证；挖洞重合 12%→69%，-288 FULL 93.4462%→93.9442%）
**module:** re-code

### 观察
MC 1.18+ 生成两阶段 RNG 基类**不同**，混用即全错：
- **CARVERS 阶段**：`ChunkRandom(new CheckedRandom(RandomSeed.getSeed()))` —— CheckedRandom 是 **48 位 LCG**：`seed = (seed * 0x5DEECE66D + 0xB) & ((1<<48)-1)`；`next(bits) = seed >>> (48-bits)`（Java `next(int bits)` 语义，高 32 位返回）。`setCarverSeed(worldSeed)` 走 LCG 递推（`nextLong()` = 两次 next(32) 有符号拼接，见 MC-239059）。
- **递归洞穴分支**：`carveTunnels/carveRavine` 内部 `Random.create(seed)` **默认也是 CheckedRandom**（LCG）——不是 Xoroshiro。
- **FEATURES 阶段**：`ChunkRandom(new Xoroshiro128PlusPlusRandom(...))` —— 与 carver 完全不同，C++ `random.h` 已有。

C++ 曾把 carver 递归分支误用 XoroshiroRandom → 漂移序列全错 → 洞穴挖洞位置不重合（仅 12%，2042/16668）；改回 CheckedRandom 后重合 69%（11929/17573）。挖洞位置一旦错位，后续一切依赖洞穴形状的方块（含水层、FEATURE 的 carving_mask、underwater_magma）全部连锁错位。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase2_carvers_result.txt`：根因「carveTunnels/carveRavine 内部 Random.create(seed) = CheckedRandom（48 位 LCG）；C++ 曾误用 XoroshiroRandom → 漂移序列全错 → 挖洞位置不重合（仅 12%）」；修复后 69%；SURFACE 模式零退化 8576 99.9994%/3200 99.9997%
- `chunkrandom.h` 头部注释：CheckedRandom 48 位 LCG 常量（MULTIPLIER=0x5DEECE66D、INCREMENT=0xB、SEED_MASK=(1<<48)-1）；setCarverSeed 语义 + MC-239059 有符号拼接
- `pipeline-map.md` 附录 A：CheckedRandom（LCG）next(bits)=seed>>>(48-bits)；Xoroshiro128PlusPlus 是 FEATURES；`Random.create(seed)` 默认 Xoroshiro **但 carver 隧道分支例外是 CheckedRandom**

### 如何利用
- 复刻 carver 前先确认 RNG 基类：**carver = LCG（CheckedRandom），feature = Xoroshiro128PlusPlus**，两套必须分别实现且绝不混用
- 递归子分支 `Random.create(seed)` 的默认实现会随上下文不同（carver 内是 CheckedRandom）——不要按「默认 = Xoroshiro」一刀切，逐调用点核对
- 挖洞位置错位是「用错 RNG」的强信号：若洞穴与参照不重合且洞穴数量量级匹配（挖洞总量接近但位置不对），先查 RNG 基类，再查 MathHelper.sin/cos 查表（65536 项，非 std::sin）

---

## 发现 #8: 两阶段 FEATURE + pendingCross 跨 chunk 写入——复刻「后写覆盖」语义需区域缓存 + 待应用队列

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 3.5 cross-chunk）
**来源定位:** MC 1.20.1 `ChunkGenerator.generateFeatures`（按 chunk 序遍历）+ `OreFeature.generateVeinPart`（椭球跨 chunk 边界）；C++ `versions/1.20.1/cpp/worldgen/src/block_probe.cpp`（两阶段 `wg_fill_blocks_multi_phase`）
**置信度:** confirmed（2026-08-10 已验证；-288 FULL 96.67%→97.8464%、300515 96.59%→98.0948%）
**module:** re-code

### 观察
Java 世界按 **chunk 序**生成：chunk A 先生成时，跨 chunk feature（如 size 大的球体 ore 椭球、紫晶洞、树冠）的方块会**直接写入相邻 chunk B 的区域**；之后 chunk B 自己生成 feature 时再写一遍 → 语义是 **「后写覆盖」（last-write-wins）**，且 A 的跨 chunk 方块在 B 生成时是可见的（B 的 feature 判定/放置会读到它们）。

C++ `fillOneChunk` 是单 chunk 独立生成、输出即 memcpy 走：处理 B 时 A 的方块已不在内存 → 既看不到 A 的跨 chunk 写入，也无法产生「A 先写、B 后写覆盖」。复刻方案（phase3.5 采用）：**两阶段生成** —— ① surface+carvers 全部 chunk 先生成并缓存 regionCols；② features 阶段逐 chunk 串行，把跨 chunk 写入暂存为 pendingCross 队列，等相邻 chunk 生成后按「后写覆盖」应用。块级提升：-288 FULL 96.67%→97.8464%（nonAir 93.6490%），granite 56.2%→88.3%、dirt 92.7%。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase35_crosschunk_result.txt`：`two-phase (surface+carvers store regionCols -> features pendingCross apply A-overwrites-B)`；-288 FULL 97.8464%、300515 98.0948%；granite 88.3% diorite 85.7% tuff 87.8% dirt 92.7%
- `block_probe.cpp L243-246`：`WG_GEN_MODE=full` 时两阶段 `wg_fill_blocks_multi_phase(h,...,1)` + `(...,2)`（阶段 1 surface+carvers 存 regionCols；阶段 2 features 串行跨 chunk 写）
- `pipeline-map.md` §6.1：Java CARVERS/FEATURES 独立 ChunkStatus、跨 chunk 邻域读取（generateFeatures 读 3×3）——单 chunk 生成会丢邻域越界部分

### 如何利用
- 凡复刻「世界按 chunk 序 + feature 可跨界写」的生成器（MC 1.18+ 均如此），**MUST 两阶段**：先无 FEATURE 阶段全量落 regionCols，再 FEATURE 阶段串行逐 chunk 处理，跨 chunk 写入走待应用队列
- 后写覆盖语义 = 以「最后生成该位置的 chunk 的写入」为准，不能各 chunk 独立求解（独立求解会丢 A 写入 B 的块或产生重复）
- 区域缓存只需覆盖 feature 的跨界半径（ore 椭球/geode 小；树冠/大 feature 大），先按 3×3 邻域缓存 + pending 队列即可复现绝大多数跨界
- 判定「跨 chunk 差异」前先确认参照是 FULL（含 FEATURE）还是 SURFACE——FULL 才有此语义（见 workflow-patterns 发现 #4）

---

## 发现 #9: blocks 参照文件每 chunk 后跟 256 项 biome 段（2+len 结构）——读取脚本 MUST 跳过，否则 chunk 坐标错位（int16 溢出假象）

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 2 参照读取修复）
**来源定位:** C++ `versions/1.20.1/cpp/worldgen/src/block_probe.cpp L194-235`（参照 blocks 文件解析）
**置信度:** confirmed（2026-08-10 已验证；修复后 SURFACE 零退化 + carver 闭合）
**module:** swe

### 观察
BlockProbe 导出的 vanilla 参照 blocks 文件（大端）chunk 记录格式为：
```
header: magic(4) + seed(8) + size(4) + originX(4) + originZ(4) + minY(4) + height(4)
每 chunk:
  cx(4) + cz(4)
  BPC = 16*16*height 个 uint16 方块状态
  biome 段：256 项（16×16），每项 = 2 字节长度前缀(blen) + blen 字节 UTF 字符串（writeUTF）
```
**关键坑**：每 chunk 的方块数据后**紧跟 256 项 biome 段**（-288/300515 参照一直含 biome 段）。若读取脚本不跳过（或按 `blen<128` 截断读），后续 chunk 的 `cx/cz` 实际读到的是 biome 名 UTF 字节拼接出的值——坐标瞬间变成超大/负值，表现为「**int16 坐标溢出假象**」，实际是**流错位**：后续所有 chunk 的坐标与方块数据全部错位，对比结果无意义（可能伴随栈越界写，如 `buf[8]` 容纳 blen 超 8 字节的 biome 名）。

### 证据
- `block_probe.cpp L199` 注释：`biome 段字符串 blen < 128，必须 ≥ 128（曾用 8 导致栈越界写）`
- `block_probe.cpp L226-231`：显式跳过 256 项 biome 段（每项先读 2 字节长度，再读 blen 字节）；L230 `writeUTF 长度无上界（biome 名 ≤ 64，但安全读全部）`
- `.investigations/feature-pipeline/cmd-output/phase2_carvers_result.txt` 关键修复链 #1：`block_probe biome 段跳过 bug（blen<128 截断）→ 参照读取错误 → 修复（参照一直含 biome 段）`

### 如何利用
- 解析此类参照 blocks 文件：读方块数据后 **MUST 按 256 项 ×（2 字节长度 + blen 字节）跳过 biome 段**，再读下一 chunk
- 长度前缀是大端 uint16；blen 用「读 2 字节得长度 → 读满长度字节」安全读，**不要假设长度上界**（栈缓冲 ≥ 128 或按长度动态分配）
- 诊断信号：chunk 坐标读出来是异常值/负值（看似 int16 溢出）而 header 的 origin/size 正常时——先怀疑**流错位**（漏跳/错跳变长字段），不是坐标本身溢出
- 用参照文件做对比的脚本（任何语言）都要按此格式解析；跨版本格式若加字段同样适用「未知变长段 MUST 先确认结构再跳」

## 发现 #10: thread_local 缓存与「每 chunk 跨线程」执行模型冲突 → 缓存命中率归零的性能回归指纹

**发现时间:** 2026-08-11（2026-08-12 根因定论确认）
**发现者:** worker（perf-rework 性能回归调查）
**来源定位:** MC 1.20.1 主世界密度求值缓存（versions/1.20.1/docs/07-block-pipeline.md 2026-08-06 纯算法优化链 FlatCache/Cache2D）+ `.investigations/perf-rework/`（WG_PROFILE/WG_SPLINEDEBUG 实测 2026-08-11/08-12）
**置信度:** confirmed（机制已 WG_PROFILE/WG_SPLINEDEBUG 实测坐实；2026-08-12 根因定论经 judge 通过 + 用户拍板；2026-08-12 修复闭环验证达标（rebuild 216=6.0/chunk、覆盖 36）+ judge 通过 + 用户验收）
**module:** perf

> **2026-08-12 修正（judge 审查要点 4 + 主因升级）**：08-11 实测（rebuild 438,092 / spline 单次 20,598ns）与 08-12 实测（rebuild 36,252 / 单次 1,714ns）差异巨大——两轮测量口径不同（多线程 thrashing 环境粗计数器 vs 单线程精确统计），见「观察」节口径说明；核心指纹结论不变，叠加因素升级为主因机制（H2），见「主因机制」节。

### 观察
性能优化常引入「局部缓存」把重复计算降为 O(1)（如 8/6 把 spline 采样 34900 → 6250 次/chunk，靠 FlatCache 5×5 网格 + Cache2D 列缓存）。**这类缓存的收益依赖「缓存生命周期 ⊇ 重复访问窗口」**：

- 原设计假设（8/6，单线程串行）：同一 chunk 生成期间大量 spline 采样重复 → per-instance（per-DensityFunction）**thread_local** 缓存命中。
- 当执行模型变为「线程池并行消费 chunk 任务」（每 chunk 可能由不同线程处理、线程跨 chunk 迁移）时，thread_local 缓存与 chunk 生命周期**不匹配**：每线程独立缓存 → 每 chunk 首访即 miss → 命中率归零 → 每次访问都走完整重建路径。

**指纹信号**：缓存重建/失效计数 ≈ 缓存访问总数（命中率≈0），且原 O(1) 路径变成重建热点（单次成本放大一个量级）；伴随「多线程不加速甚至反降」（并行只放大重建并发，不摊薄重复访问）。本次实测（2026-08-11，多线程 8/22 线程 thrashing 环境下计数器）：FlatCache rebuild 438,092 ≈ spline 调用数、Cache2D miss 458,281 次、spline 单次 992ns → 20,598ns、density 阶段 8.5-11.7ms → 670-1000ms/chunk——正是此指纹。

**08-11 vs 08-12 口径说明（judge 审查要点 4）**：两个测量口径不同，不构成矛盾——
- **08-11**：多线程（8/22 线程）thrashing 环境下的粗粒度计数器。每 chunk 跨线程迁移 → 单槽缓存全 miss + 多线程重建并发 → rebuild 计数 ≈ spline 调用数（命中率≈0 的表象）、spline 单次被 thrashing 放大到 20,598ns。
- **08-12**：单线程（-threads 1）WG_SPLINEDEBUG 精确统计，剥离 thrashing 后暴露真实主因结构：rebuild 36,252 次 = 每 chunk ~1007（期望 ~6）→ **168×**；rebuild 仅占 spline 调用（4,695,145 = 130,420/chunk）的 **0.77%**；spline 单次 t1 1,714ns（mt 27,155ns，16× thrashing）。
- **核心结论不变**：thread_local 单槽缓存 vs 跨线程执行模型失配。08-12 数据把放大链精确化为「rebuild 168× × 13.36 spline/miss」的级联（而非 08-11 表象的「rebuild ≈ 访问总数」）。

**主因机制（2026-08-12 定论，H2 新指纹）**：FlatCache 网格构建含**嵌套采样递归**。buildGrid 角点 `i=4`/`j=4` 时 `p.x=(chunkX*4+4)*4=(chunkX+1)*16` 指向**下一 chunk 首列** → 嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（key=(x>>4,z>>4) chunk 级）→ 单槽缓存被污染 → 重建邻居网格 → 递归蔓延（实测 112 chunk = 36 生成 + 76 邻居，含左下对角 (44,-28)）→ rebuild 36,252 = 每 chunk ~1007 vs 期望 ~6（**168×**）→ spline 调用 20× 爆炸（130,420/chunk vs 旧 6,250）。

**H2 指纹信号**：缓存 key 由采样坐标派生（chunk 级），而采样点存在**越出当前上下文范围的角点**（buildGrid i=4/j=4）时，单槽缓存必然收到非本 chunk key → miss + 重建 + 递归蔓延。排查特征 = 重建计数的 chunk 覆盖**超出生成范围**（112 = 36+76 邻居）。**修复（2026-08-12 已实施并闭环）= 当前 chunk 上下文绑定**：thread_local `g_curChunkX/Z` 绑定当前生成 chunk（fillOneChunkCore 入口 RAII 设置、返回恢复 `INT32_MIN`），k/l 相对 startBiomeX 计算，越界 → `delegate.sample(pos)` **直算不重建**——即 Java per-chunk 实例语义（ChunkNoiseSampler.java L836-881：构造时预计算 25 角点、之后纯查表、永不构建邻居网格）的 C++ 模拟。**关键教训：per-chunk 多槽 LRU 不足以根除**（初版 16 槽 LRU 仍为 pos 推导的邻居 key 构建网格，rebuild 仅 36,252→7,318，覆盖仍 112）——必须消除「越界→重建」语义本身；**改循环顺序无效**（块级不触发 spline，H1 非主因）。

### 证据
- WG_PROFILE（2026-08-11，density 阶段，多线程 thrashing 环境）：spline 单次 992ns → 20,598ns；spline.sample 338 万次；FlatCache rebuild 438,092 次 ≈ spline 调用数；Cache2D miss 458,281 次；density 阶段 670-1000ms/chunk（旧 8.5-11.7ms）
- WG_PROFILE/WG_SPLINEDEBUG（2026-08-12，单线程 -threads 1 精确统计）：spline 4,695,145 次（130,420/chunk，旧 6,250 → **20×**）；FlatCache rebuild **36,252** 次 / 112 chunk（每 chunk ~1007，期望 ~6 → **168×**）；CACHE2D miss 351,536 次（4 个 cacheId，= 14,061 rebuild × 25 角点 ✓）；spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×** thrashing）；放大链 = rebuild 168× × 13.36 spline/miss ✓
- 修复后验证（2026-08-12 终版 ctx，数据 `.investigations/perf-rework/cmd-output/`）：FLATCACHE rebuild **216 = 6.0/chunk**（期望 ~6 完全达标）、覆盖 **36**（蔓延根除）；CACHE2D miss **23,117**（旧 351,536）；SPLINE **3,032/chunk**（SPLINEDEBUG 非 leaf 口径，旧 66,682，回旧基线 6,250 水平；WG_PROFILE 全量 spline.sample **5,906/chunk**）；单线程 wall 6,533→**2,910ms**（2.2×）；bench 单线程 **62.38ms/chunk**（旧 ~181，3×）；8576 **99.9994%** / 3200 **99.9997%** 零退化
- 初版 16 槽 LRU 对照（已弃用）：rebuild 36,252→7,318（203/chunk）、覆盖仍 112（splinedebug_8576_t1_fixed.txt）——多槽只降频率不除「越界→重建」语义
- 吞吐（SURFACE）：07 篇旧基线串行 28.1ms/chunk、并行 49.4ms/16chunk（3.1ms/chunk）→ 2026-08-11 实测单线程 98-182ms/chunk、多线程（8/22）108-239ms/chunk **无加速反降**
- 对照实验排除「本次改造引入」：stash 本次改动（Java 桥重写 + C++ 池改造）后 HEAD 版 block_probe 8×8 仍 10.2s；07 篇基线提交 86e4057 也要 8s → 回归在 8/6 优化链之后积累
- 数据载体：`.investigations/perf-rework/`（requirements-doc.md / static-audit.md / architecture.md / random-seed-sampling.md）+ 10 时间线 2026-08-11 条目 + 07 篇「性能回归实测」小节草稿

### 如何利用
- **设计缓存前先明确「缓存生命周期 vs 执行模型」是否匹配**：thread_local 只适合「线程内连续消费同一上下文」（如单线程完整生成一个 chunk）；线程池并行 + 任务迁移时，用 **per-chunk 键索引缓存**（缓存随 chunk 生命周期）或按调用上下文显式传入，**不要依赖线程亲和**
- **性能回归排查第一手段 = 缓存计数器**：看 rebuild/miss 与命中之比；命中率≈0 即缓存失效（本次正是靠 WG_PROFILE 计数器坐实）
- **优化计数器要结合真实执行模型验证**：8/6 的 spline 34900 → 6250 次/chunk 是单线程串行模型下的计数，未覆盖多线程并行/线程迁移——「优化后计数器下降」不等于「目标执行模型下收益」
- **git 二分定位引入点**：stash/checkout 旧提交对照（本次 8s 级退化用 stash 实验证明非本次引入，具体引入提交待二分）
- **越界角点指纹（H2，2026-08-12 定论，修复已闭环）**：缓存 key 由采样坐标派生（如 `(x>>4,z>>4)` chunk 级）且采样点可能越出当前上下文范围（buildGrid 角点 i=4 → 下一 chunk 首列）时，单槽缓存必然被邻居 key 污染 → 递归重建蔓延。排查 = 检查重建计数的 chunk 覆盖是否超出生成范围（本次 112 chunk = 36 生成 + 76 邻居实锤）；**修复 = 当前 chunk 上下文绑定**（thread_local 显式传入当前 chunk 键 + 越界直算不重建，模拟 Java per-chunk 实例语义；实测 rebuild 216=6.0/chunk、覆盖 36）——**多槽 LRU 不够**（本次初版 16 槽 LRU 仍为邻居 key 建网格、覆盖仍 112），必须消除「越界→重建」语义
- 跨版本/跨项目通用：任何「局部缓存 + 并行执行」组合都适用此检查

## 发现 #11: spline 树扁平化模式（递归 shared_ptr 树 → 连续数组 + 非虚采样）——单线程 -24% 零退化，但多线程 latency-bound 不改善

**发现时间:** 2026-08-13
**发现者:** worker（perf-rework 无损优化 Phase 1）
**来源定位:** C++ `versions/1.20.1/cpp/worldgen/src/density.h`（SplineDF → 连续 nodes[]/locations[]/derivatives[]/subIdx[] + sampleNode）+ `.investigations/perf-rework/`（phase1-design.md / review-aae119d.md）
**置信度:** candidate（实测单线程 -24% + judge 逐行核对 Hermite 公式逐位等价 + 零退化落盘；多线程课题未闭合）
**module:** perf

### 观察
MC 密度函数的 SplineDF（spline 节点）原生是「递归 shared_ptr 子节点树 + 虚调用采样」：每次 `apply` 经 `locationFunction->sample`（虚调用）→ 二分查 locations → `subSplines[k]->sample`（虚调用递归嵌套）→ Hermite 插值，每层 2 次虚指针间接跳转 + 二分，cache miss 高。

扁平化把递归树展开为连续节点数组（nodes/locations/derivatives/subIdx/locationFunctions 池）+ 整数索引，采样改为非虚递归 `sampleNode`。Hermite 插值公式（`lerp(kd,nv,ov)+kd(1-kd)lerp(kd,p,q)`）逐位不变。

**实测**：单线程 density wall 61.7→47.1ms（-23.7%）、吞吐 92.08→71.68ms/chunk（-22.2%）；零退化 8576 99.9994% / 3200 99.9997%。

**关键边界**：扁平化只解决 spline 子树自身的间接寻址 cache miss。多线程 8t 下 density 460.8→478.3ms **不改善**——多线程膨胀根因在 InterpolatedDF::buildGrid 的 1225 角点树遍历**整体**（spline 递归 + FlatCache 查表 + noise 的 cache miss 叠加），不在 spline 递归这一层。要改善多线程 latency 需 **DFC（整个 DF 树扁平化）**，非仅 spline 子树。

### 证据
- `analyze_stagetimer.py` 现场复跑（n=128）：phase0_baseline density median 61.7 / phase1_splineflat 47.1；[A] threads=1 92.08→71.68
- `regress_8576_aae119d.txt` 3538922/3538944（99.9994%）、`regress_3200_aae119d.txt` 1572860/1572864（99.9997%）零退化
- 8t density median：phase0 460.8 → phase1 478.3（不降反略升，未兑现 phase0-quantify 的「10× 膨胀有望大幅回落」预估）
- judge review-aae119d.md 要点 1：Hermite 公式逐位等价（含 n==1/i<0/i==n−1/min-max 全边界）

### 如何利用
- 复刻/优化 MC density 树的 spline 时，优先把「递归指针子节点 + 虚调用采样」扁平化为连续数组 + 整数索引 + 非虚采样：单线程 cache miss 立减，收益大且零退化可保（纯布局重排，采样公式逐位不变）。
- 但**别指望子树扁平化解决多线程 latency-bound**：先定位耗时大头在哪一层（用 WG_PROFILE/临时计数器拆 buildGrid 角点采样 vs 块级插值），若大头是「上层树遍历触发 + 多层 cache miss 叠加」（如 InterpolatedDF::buildGrid 1225 角点 × arg->sample），则需整个 DF 树扁平化（DFC）而非局部子树。
- 通用规律：指针链深递归 + 每层虚调用的数据结构，扁平化是低风险高收益的无损优化；但收益上限受「遍历触发点是否集中在该结构」约束——若遍历大头是调用方触发的多层组合，单层扁平化收益有限。

## 发现 #12: 边界角点复用收益小的根因——缓存构建触发点不集中在角点，跳过角点省不了树遍历

**发现时间:** 2026-08-13
**发现者:** worker（perf-rework 无损优化 Phase 2）
**来源定位:** C++ `versions/1.20.1/cpp/worldgen/src/density.h`（InterpolatedDF::buildGrid 1225 角点 / FlatCacheDF::buildGrid 5×5=25 角点）+ `.investigations/perf-rework/`（review-aae119d-followup.md §3）
**置信度:** candidate（实测 -1.7% 收益坐实 + 根因机制推演）
**module:** perf

### 观察
InterpolatedDF 网格 x/z 边界角点（gx=0 列 = 左邻 gx=4 列）与相邻 chunk 坐标重合，phase1-design 预估「x/z 双向边界复用可减 -36% buildGrid 采样 → -28% density」。但实测只做 x 方向左邻列复用（上限 245/1225=20%），density 47.1→46.3ms（-1.7% 接近噪声）、吞吐 71.68→72.06（+0.5% 无改善）。

**根因**：InterpolatedDF::buildGrid 的 1225 角点采样耗时大头**不集中在可跳过的边界角点**，而在「每 chunk 每实例 1 次的 FlatCache buildGrid 构建触发」+「spline 树遍历」：
- FlatCache buildGrid 只在**首个角点**触发一次（其后 24 角点查表命中），跳过 gx=0 列只是把首次触发从 gx=0 移到 gx=1，**省不了这次触发**；
- gx=0 列其余 244 角点是 FlatCache 查表命中（快），跳过省不了多少。

即：边界复用优化了「角点采样次数」这个**错误目标**——真正的耗时是「树遍历触发点」而非「角点数量」。

### 证据
- 实测：phase2_edgereuse density median 46.3（vs phase1 47.1，-0.8ms）、[A] threads=1 72.06（vs 71.68，+0.5%）——端到端总 wall 反升
- review-aae119d.md 要点 2/3：实现只做 x 方向左邻列（非 x/z 双向）；density -0.8ms 在 min-max 波动（±38ms）内
- review-aae119d-followup.md §3：根因 = FlatCache buildGrid 只在首个角点触发一次，gx=0 列其余 244 角点查表命中

### 如何利用
- 优化「重复采样」前先拆「每次采样的成本构成」：若某采样点集里只有**首次**采样触发昂贵构建、其余是便宜查表命中，则「减少采样点数量」省不了多少（触发点不随数量减少而消失）——要优化的是「构建触发次数」而非「采样次数」。
- 边界角点复用类优化的收益上限 = 可跳过的采样里「昂贵采样」的比例，不是「采样总数」的比例。本例 245 角点里只有 1 个（首个）是昂贵构建触发，其余 244 是查表命中 → 收益上限远低于「-36% 采样数」的朴素估算。
- 通用规律：缓存/预计算类的「重复计算」里，先用计数器区分「首次构建触发（昂贵）」vs「查表命中（便宜）」的占比，再决定是否值得做「去重复」——若昂贵部分不集中在可跳过的边界，去重复收益必然低于按总数比例的估算。

## 发现 #13: GPU 驱动编译时间——编译期常量下标进数据驱动函数 = 常量传播展开陷阱；「编译慢根因结论」有版本域（const 表 vs SSBO）

**发现时间:** 2026-08-15
**发现者:** worker（G4 A 方案实施 + A5 减法二分）
**来源定位:** `.investigations/perf-rework/a-plan-ssbo-implementation.md`（A5 节）+ `.investigations/perf-rework/gpu-accel-errors.md` D21/D22 + `.investigations/000-架构设计/架构计划-gpu-spline-fix.md`（001 修订版）；复现数据 `cmd-output/compile_bench-A5-*.txt`
**置信度:** confirmed（2026-08-15 用户拍板：减法二分证据链 + 3 次实测 67.4/71.4/101.8s 均 <120s 达标 + 正确性逐位一致 maxDiff=3.128e-07 + judge 审查通过）
**module:** perf

### 观察
GPU 驱动编译时间（vkCreateComputePipelines 的 SPIR-V→机器码）对「数据驱动函数」的索引形态极度敏感，两个互相关联的通用模式：

1. **编译期常量下标进数据驱动函数 = 常量传播展开陷阱**：spline_coord 的 `switch(coordType)` 使每个 case 内 `NOISE_SLOT_BASE[0]` 成为编译期常量下标 → 常量传播进 normal_noise（数据驱动函数，参数表在 const 数组）→ `NORMAL_PACK` 读取被静态化 → 驱动逐 case 循环展开（单次调用 +37~75s）。对照：eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]`（索引完全动态）→ 驱动放弃展开（快）。**同一批数据、同一个求值函数，仅「索引在编译期是否可解析」的差异 → 编译时间 350.6s vs 37.2s（~10×）级差**。
2. **「编译慢根因结论」有版本域**：D21 在 const 表版实证「动态 node 索引是主因」（固定 node=0 → 903.4→31.0s）；SSBO 化后做同一实验（fixed_node=0）→ 361.0s ≈ full 350.6s（无收益）——**同一个「固定动态索引」实验，const 表版成立、SSBO 版不成立**（SSBO 已把动态索引变成运行时 buffer 读，驱动无从展开，因此固定它也没有额外收益）。

### 证据
- 减法二分链（DFC_DIAG 诊断开关 + compile_bench 秒级测）：full 350.6s / fixed_node 361.0s（动态 node 索引非 SSBO 版主因）/ coord_const 37.2s（coord 表达式贡献 ~313s）/ coord_slot0 302.3s（排除实例数因素）/ coord_case0 74.8s（1 次 normal_noise 调用 +37s）/ no_spline 17.2s（eval_df 内同函数调用不慢）。
- 修复验证：coordType 运行时查表（`COORD_SLOT_TABLE[coordType]`）+ fold 特例（`if (coordType == 2)`）后，pipeline 编译 350.6s → 67.4s（e2e 内计时）/ 71.4s（compile_bench 单独）/ 101.8s（第 3 次）——3 次均 <120s；正确性逐位一致（maxDiff=3.128e-07 / avgDiff=1.097e-08，seed 8576294172403134396，N=1024）；fp64 次因自动作废（no_old 只省 ~8.5s——fp64 成本是「与 spline 展开的交互效应」）。
- 历史对照：const 表版动态 node 索引 903.4s（D21）→ SSBO 版 350.6s → 查表修复 67.4s；spline 子系统 ~885s → ~50s（-94%）。

### 如何利用
- **排查 GPU 驱动编译慢时，先查「数据驱动/查表函数的所有索引是否运行时不可解析」**——switch/case 把下标常量化、函数参数折叠成常量、const 数组编译期已知下标，都会触发常量传播 + 展开；把下标改成「运行时查表」让驱动无法静态化，是编译时间分水岭。
- **「固定某索引 / 去掉某子系统的减法二分」是最快定位手段**：一次实验排除一个候选（本次 coord_case0 单次调用定位 +37s），比机制猜测快；配合 DFC_DIAG 类诊断开关 + 秒级编译计时器使用。
- **根因结论必须声明版本域**：凡是「在某结构版本下成立的编译慢根因」（如动态 node 索引），结构改变后（const 表→SSBO/数据 buffer）**必须重新验证，不能直接复用**——「动态索引」在 const 表里是驱动展开的输入，在 SSBO 里已经是运行时读，同一表述指代完全不同的编译行为。
- 跨项目通用：任何 GPU compute shader / OpenCL kernel 的驱动编译时间优化（C2ME 类内核分发、pipeline 预编译分发）都适用「索引可解析性」与「版本域」两条检查。

## 发现 #14: 边界外推遇嵌套 value 必须递归（spline 边界分支的「执行不到」类 bug）+ 单域 e2e 验证是盲区制造机

**发现时间:** 2026-08-15
**发现者:** worker（block_probe 集成立项 I5 吞吐对比 → D23 定位，perf-rework GPU 集成课题）
**来源定位:** `.investigations/perf-rework/gpu-accel-errors.md` D23 段（含最终合并版）+ `.investigations/perf-rework/i-integration-record.md` + `.investigations/perf-rework/review-003-d23-integration.md`；复现/验证数据 `cmd-output/domain-probe-D23-fixed-20260815.txt` / `cmd-output/e2e-A5-20260815-135509.txt` / `verify_p11_recursive.py`
**置信度:** confirmed（2026-08-15 用户拍板：GPU+sim 双修 + domain probe 全域 clean + e2e 零回归 maxDiff=3.128e-07 + 显式栈 vs 递归参照 1344 组合 0 mismatch + judge 4 P1 全闭合）
**module:** re-code（发现于 perf-rework GPU 集成，规律本体为复刻算法正确性 + 验证覆盖方法）

### 观察

GPU 引擎（spline_eval 显式 while 栈）与 CPU 参照（DensityBuilder）在 e2e 验证域（x≤63, y∈[-64,-49], z≤4）逐位一致（maxDiff=3.128e-07），但在域外大坐标 chunk 域系统性错值（(784,160,-408) gpu=0.045 vs cpu=-0.458，量级级差异非浮点舍入）。根因 = **spline_eval 边界外推（coord < loc[0] / coord > loc[n-1]）对端点 value 写成 `(kind==0 ? valF : 0.0f)`——嵌套 value（kind==1）直接返回 0，未递归求值**；vanilla `Spline.apply` L259/261 的边界外推是 `value[0]+der[0]*(x-loc[0])`，端点 value 为嵌套样条时**必须递归求值**。触发条件：spline55 的 coord（continentalness@c0）= 0.060231412 **恰好 > 最后 loc 0.06** → 右边界 → 嵌套 value 返回 0 → 上层链错（参照该点应递归得 factor=4.524）。由此提炼四个跨版本/跨项目通用规律：

1. **边界分支「执行不到」类 bug 指纹**：边界外推（coord 超出 locs 范围）分支只在特定坐标域触发——单域验证（e2e 小域）永远测不到——「逐位一致」只证明**被覆盖的域**；性能/吞吐探针必须顺带做多 chunk/多 cell/多 y 层 diff 抽查。同类还有 C12（range_choice 常数分支吸收误差）——「采样点没覆盖有效路径」的假正确是通用陷阱。
2. **模拟器与 GPU 同源产物同错**：模拟器复现 GPU 错值（sim=GPU=0.045303285）＝生成器+解释器**共同逻辑 bug**（非 GPU 特有）——定位先做「GPU 特有 vs 共同逻辑」二分（sim 能复现 → 直接排除 GPU kernel/驱动层）；但 sim 只能证明「生成器产物内部一致」，**必须与第三方参照（DensityBuilder）对拍**才能发现生成器级错误。
3. **显式栈移植的返回地址/恢复点纪律**：显式栈的「返回地址（outSlot）」与「父帧恢复点（stage）」是两套状态——压帧时各设一次，回填时**只写数据槽**；任何「回填时顺带改父帧 stage」的优化破坏等待语义（跳 v1 求值 → Hermite 用 0）。
4. **对照 vanilla 逐行是最终手段**：Spline.apply 边界外推是递归求值（L259/261），不是取 0——「生成器里留的 stub/简化占位」是语义差头号嫌疑（D17 ws→0.0f 同教训），对照原版逐行才收口。

### 证据

- 决定性单点：(784,160,-408) 修复前 gpu=0.045303289 vs cpu=-0.458333333（diff 5.036e-01）→ 修复后 gpu=-0.458333343（diff 9.9e-9，`cmd-output/domain-probe-D23-fixed-20260815.txt`）
- 错误域模式：z-scan（y=160 x=784）z=-432..-412 对 / z=-408,-404 错（cz=2/3 格）；y-scan（x=784 z=-408）y=-64 对 / y∈[-56,248] 几乎全错 / y≥256 对（无地形常数分支 -0.02499）——**常数分支层吸收差异 = 假正确**（C12 同款陷阱）
- 根因证据链：sim 复现 0.045303285（与 GPU 完全一致）→ 排除 GPU kernel 特有；node[54]（roughness@c0）拆分采样 == CpuBackend 直接采样逐位一致（coord 正确）；node[22]/[33] SPLINE 大坐标域算出 0；spline55 数据（locs=[-0.19,-0.15,-0.1,0.03,0.06]）coord=0.060231412 > 0.06 触发右边界
- 修复验证：e2e maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线逐位一致（零回归，`cmd-output/e2e-A5-20260815-135509.txt`）；显式栈 spline_eval_py vs 递归版 Spline.apply 参照 **1344 组合 0 mismatch**（`verify_p11_recursive.py`，覆盖边界触发域坐标）
- 候选排除（❌）：H1 角点序 / H2 cell 推导 / H3 split 数值均验证无差；中间误判（「缺 noodle_ridge_b 拆分行」「双索引错位」）被 check_split_base.py + check_two_alloc.py + check_meta_vs_splitbase.py 证伪——**对账必须基于当前生成产物**（旧 comp/spv dump 会误读索引，多花数轮）

### 如何利用

- **验证覆盖设计**：GPU/加速内核接入集成/吞吐探针时，正确性抽查 MUST 覆盖多 chunk（含 chunk 0 外）/多 cell（cy≥1、cz≥2）/多 y 层（含常数分支层——常数分支吸收差异是假正确）；「单域逐位一致」不能作为全域正确性证据
- **性能/吞吐探针默认带 diff**：只测时间不测正确性 = 只能发现慢不能发现错（本次 16/64 chunks 正是靠附带 diff 抽查才暴露 D23）
- **「GPU 特有 vs 共同逻辑」二分**：模拟器能复现 → 生成器/解释器共同 bug，先排除 GPU kernel 特有路径，再与第三方参照逐分量对拍（registry 分量探针 getRegistryEntry 采样 factor/sloped/entrances 最快）
- **显式栈移植**：返回地址与恢复点分离管理——压帧设一次、回填只写数据；边界/特殊路径用显式 stage（等边界 v0/等边界 vn），不重载普通 Hermite 状态
- **数据驱动树/表（spline 类）的边界语义**：外推端点 value 为嵌套结构时递归求值（vanilla 语义），任何「简化取 0」都是潜伏的域相关 bug——跨版本（1.18/1.19 Spline.java 同构，边界外推同为 `value[0]+der[0]*(x-loc[0])` 递归）同样适用

## 发现 #15: GPU 批量加速的带宽死局：每点数据量 × 点数是可行性判据（split 全量上传 vs 网格角点）

**发现时间:** 2026-08-15
**发现者:** worker（GPU 块级生成立项 003 I6-I8 实测 → D24 负面结论，perf-rework GPU 集成课题）
**来源定位:** `.investigations/perf-rework/gpu-accel-errors.md` D24 段 + `.investigations/000-架构设计/架构计划-gpu-block-integration.md`（003 结论段）+ `.investigations/perf-rework/i-integration-record.md`（I5-I8）
**置信度:** candidate（「split 全量上传带宽死局」机制由实测坐实——24 chunks 11 分钟未完成 vs CPU 2.5 分钟；通用规律跨项目外推待更多案例验证）
**module:** perf

### 观察

GPU 引擎算 finalDensity 完整树需要**每个点的全部分解坐标**（`splitTotal=8672` floats/点，CPU 预拆分 double→int32 格点 + float 小数）。同引擎、同 shader，**采样密度直接决定每点数据总量，进而决定可行性**：

- **逐 block 完整树（不可行）**：98304 点/chunk × 8672 × 4B = **3.4GB split 数据/chunk** 需上传；分块 4096（显存限制）→ 24 次 dispatch/chunk × 142MB + readback → 24 chunks × 24 次 = 576 次大上传 = **82GB 数据搬运** → PCIe ~16GB/s → 分钟级。实测 24 chunks **11 分钟未完成 vs CPU 2.5 分钟**（慢 4 倍+）。**GPU 快在「算」（compute throughput），但被「喂数据」（host→device 带宽）完全主导**。
- **网格角点级（可行，22-39x）**：768 点/chunk × 8672 × 4B = **27MB/chunk**——GPU 批量有意义（wg_fill_density 实测 22-39x）。

**核心判据**：**「单点数据量小 + 点量大」是 GPU 批量加速的前提**。每点数据量 × 点数 = 上传总量，超过 PCIe/总线带宽的秒级承载量（~GB/s 级）即带宽死局——与 GPU 算力无关（本次算力 24-32x 充足，卡在喂数据）。

### 证据

- 实测吞吐（I7）：24 chunks（8576 区域）GPU 逐 block 路径 **11 分钟未完成**（主动终止）；CPU 基线同区域 **2.5 分钟**——GPU 块级路径比 CPU 慢 4 倍+（`cmd-output/` I7 运行记录 + gpu-accel-errors.md D24 段）。
- 带宽账：98304 点/chunk × 8672 floats/点 × 4B = 3.4GB/chunk；分块 4096 → 24 次 × 142MB/次；24 chunks × 24 = 576 次大上传 = 82GB；PCIe ~16GB/s → 分钟级。
- 对照组（I5）：wg_fill_density 网格角点批量 768 点/chunk × 8672 × 4B = 27MB/chunk → **22-39x**（吞吐探针实测落盘 `cmd-output/throughput-I5-*.txt`）——同引擎同 shader，仅采样密度不同，可行性翻转。
- 并发崩溃（P2-4 闭环）：I7 首次运行（无 mutex）`context=wg_fill_blocks_multi/fillOneChunk` `code=0xC0000005`，栈在 nvtfi（NVIDIA 驱动层）；fill() 加 `std::mutex fillMtx` 串行化后无崩溃——**多线程并发 GPU 调用（共享 buffer 上传/dispatch 无互斥）→ 驱动层崩溃，不是返回错误**。
- 回退验证（I8）：默认 CPU 路径 8576 **99.9994%** 零退化（与基线一致）；3200 沿用 99.9997%。

### 如何利用

1. **GPU 加速可行性先算「每点喂多少数据」，再谈「每点算多少」**：上传总量 = 每点数据量（拆分坐标/特征向量）× 点数。凡每点数据量是 KB 级 × 点数是万级+（如逐 block 98304 点），先做带宽账（总量 ÷ PCIe ~16GB/s），分钟级即不可行——**「单点数据量小 + 点量大」是 GPU 批量加速的前提**；正确形态是「GPU 算网格/角点（点量小）+ CPU 插值/后处理到逐点」（两阶段拆分）。
2. **吞吐探针结论有「采样密度域」**：同一引擎同一 shader 在某采样密度（网格角点 768 点）下实测 22-39x，**不能外推到更高密度（逐 block 98304 点）**——数据量 ∝ 点数，可行性随密度翻转。引用吞吐结论必须声明采样密度域。
3. **多线程并发 GPU 调用必须互斥**：共享 buffer 上传/dispatch 无锁并发 → 驱动层 0xC0000005 进程级崩溃（**不是返回错误**，是崩溃）——GPU 资源并发是硬约束，任何多线程宿主（线程池/自适应并行）接入 GPU 路径 MUST 先加互斥（mutex/串行化），再谈性能。
4. **负面结论也是知识（错误优先原则）**：接线正确（无崩溃、逻辑对）但吞吐不可行时，记录「为什么不可行」（带宽分析 + 数据账）比假装成功有价值——避免后人重复实现同一死局方案；负面结论同样要落五段式错误台账 + 时间线。

## 发现 #16: aquifer est 冷扫描机制指纹——per-chunk 新建 Aquifer × 全价 init 采样 ≈ 15.4ms/chunk（冷态主体）
- **时间/置信度/module**：260903-10，confirmed（260903-10 用户拍板），MC worldgen 性能指纹。
- **指纹**：每 chunk 新建 Aquifer → surface_cache 冷 → get_water_level_at miss(~158/chunk) → get_fluid_level 13 offset 列（横跨 ≤5 x-chunk）→ 每列 estimate_surface_height ~34 次 initial_density 全价采样（SURF 计数器口径 214×34.35=7342 次/chunk × **2117ns/iter（R2 调和：新鲜进程实测）≈ 15.4ms，占 counter-free 冷态超额 22.70ms 的 ~68%**）；重量叶 = depth→sloped_cheese→base_3d_noise（old_blended_noise，24 octave/次，InterpolatedNoiseData::sample 无缓存）。对照：Java NoiseChunk 的 est 列缓存随 chunk 生命周期持久。init 子树无 interpolated 节点（GRID_ARG_SAMPLES=0 反证）。（首版「3557ns/26.1ms」口径被 judge R2 调和取代——原始 cmd-output：qaq1-r2-reconcile-260903-10.txt）
- **如何利用**：冷−暖差大（~20ms+/chunk）签名 → 先查 est 扫描而非 apply 单价；修复方向 est 查表化/列缓存跨 chunk 持久化 > surface_cache 单独持久化。证据：.artifacts/lossless-accel/qaq1-attribution-260903-10.md。

## 发现 #17: aquifer barrier margin 机制指纹——|d|≈0.02 微负带是 margin stone 高发区，「stone 但 d≤0」≠ bug（260904-06）

- **时间/置信度/module**：260904-06，candidate（生产密度 dump + 静态结构互证，均布局无关；confirmed 留人类），MC worldgen 机制指纹。
- **指纹**：aquifer `apply` 在 d≤0 时唯一把 block 翻成 stone 的路径是 barrier margin（`density+e>0` 三连 -1 翻转，aquifer.h:121-137 / aquifer.rs:327-338，三方零偏离——b3 十七项静态对拍）→ **|d| 极小（≈0.02，如 −0.024995 带）的 stone 带是 margin 高发区的签名**；「该处有 stone 但 d≤0」不是 density 写者 bug，先查 margin。C++ 消费环：worldgen_api.cpp:1040（block<0 → stone）。实测锚：WG_DBDEBUG 生产密度 (195,199) 列 y192-318 全负（−0.025/−0.458 带，按 y 直印、布局无关）。
- **如何利用**：stone 带排查先看 d 剖面——d 微负（|d|≈0.02）+ 带状 → 直接归 margin（aquifer 高位水口袋 ~16 间距伴生），跳过「density 写者缺失」方向；d 显著正的 stone 才查 surface/ore 写者。注意 Rust 侧已知分叉：margin→air 丢 barrier stone（b3 发现，只影响幕帘构成）。证据：.investigations/lossless-accel/incident-layout-260904-06.md（硬数据存活清单）+ p4-reference-check-260904-06.md Facts #1。

## 发现 #18: MC 1.20.1 MultiNoise biome 分类器是 1e-4 定点 long 域运算——f64 全精度复刻在精确平局点出 ~1e-9 假严格差，分类翻转（260904-13）
- 时间/置信度/module：260904-13，candidate（Java mnDump NoiseValuePoint 直读 + 三源同点互证 + decisive 12→3；confirmed 留人类），MC worldgen 算法指纹。
- 指纹：MultiNoiseUtil.toLong(v)=(long)(v*10000F)（先过 float），NoiseValuePoint 字段（temperatureNoise() 等）为 long 定点 getter、无 double getter；SearchNode 距离比较在 1e-4 定点 long 域，定点域精确平局由树遍历序取先（参数表行序靠前者胜）。f64 全精度复刻且不量化输入 → 同一输入定点域本应精确平局的两候选出 ~1e-9 假严格差 → 分类翻转。
- 实例：残 9 gravel→sand（seed 8576294172403134396，pick cell (59,9,62)，t=0.200031787）：定点域 dist(deep_ocean)=dist(deep_lukewarm)=5041 精确平局 → 树序取 deep_ocean（gravel ✓）；f64 穷举差 1.01e-9 → deep_lukewarm（sand ✗）。
- 判据：分类边界残差 + dbg_dist 呈「定点量化粒度」量级（如 5041 原始 i64、或 1e-9~1e-5 f64 微差）→ 先怀疑量化语义缺失，再查采样链。
- 如何利用：任何 MultiNoise 分类器复刻必须复刻 toLong 量化（含 float 中转）+ long 域距离 + 平局取树序先——「数学更精确」的浮点距离 = 更错。证据：.artifacts/lossless-accel/residual9-verdict-260904-13.md + .tmp/p2full/res13-mndump4-260904-13.log。


## 发现 #19: jungle 树 blob/bush 叶层角消费基线——bush 每棵 9 次（r=0 退化角仍消费 1 次 nextInt(2)）（260905-13，简记）

- jungle_tree blob（h=4，j 序 [1,1,2,2]）：每棵 **16 次**角消费（4 层×4 角；勘误 s1-semantics:386 的 8 次——i/2 向零截断漏算负分支，已被两侧 trace 证实）。
- jungle_bush（r 序 [0,1,2]）：每棵 **9 次**——r=0 层 `dx==radius&&dz==radius` 在 dx=dz=0 时**仍成立**，退化角照样消费 1 次 nextInt(2)；Java 侧 mixin 对拍必须同样覆盖 r=0 退化角，否则 9 行基线对不上。
- trace 比对键：(y, r, |dx|, |dz|) 元组序列逐行一致（Java mixin 打 |dx|，Rust 打带符号——口径差声明，§9.7）。
- 证据：`.investigations/jungle-l/b3-shortcircuit-trace.md` §四a/§五（Java 45 行=5 棵×9、Rust 90 行=10 棵×9，逐棵同构 → J3 短路族两域核销）。

## 发现 #20 简记: MJT0 树基高度 h 指纹作「两侧地形等价性」廉价探针（260906-09）；candidate

- **指纹**：`[MJT0] t=(x,y,z) h=N` 行中的树基高 h = 放置点地表高度的可观测代理——**共享点位 h 逐位相同 = 局部地形等价的零成本直证**（无需 block_probe diff）；**单柱 h 版本间漂移 = dll 地形版本漂移的直证签名**（实例：500,75,-234 h19@260905-13 旧 dll → h13@f3r2 新 dll，同 seed 同 x,z 差 6 格）。
- **边界**：h 只在「该 run 放了树」的柱上可观测——未放树柱（如 483,71,-230 于 f3r1/f3r2）是盲区，h 指纹不能外推到盲区柱（.investigations/jungle-l/f3-twochannel-260906-09.md §4.1-2）。
- **证据**：f3-twochannel-260906-09.md §2.2-2/§4.3；Java 共享点 439 h23 / 443 h28 / 456 h14 / 451 h26 / 511 h16 / 500,-226 h11 逐位复核同。

### 发现 #21 简记: MC 维度确定性指纹——同载体同 seed 下 run 级非确定是**维度属性**：overworld ~0.009%、nether ~0.14%（~15×）、end = 0（比对集内两 run 逐块零差）（260910-05）

- **发现时间 / 发现者 / 置信度 / module**：260910-05；主会话（region 全域逐块对拍，每维同形态两 run）；**candidate**（每形态 1 对点估计）；algorithm-fingerprints / 维度指纹（载体选择与判门前提）。
- **来源定位**：`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md` §3.1/§3.2/§5.3；同目录 `verdict-c7-260910-05.md` §2.1；一手 `.investigations/perf-closeout-260910-05/cmd-output/`：`diff-N_async_vs_async2.txt:74`（nether 同形态 `0.1406%`）、`diff-E_async_vs_async2.txt:74`（end 同形态 **0**）、`diff-r3-r1_vs_r3-r2.txt:74`（overworld async 同形态 `0.0092%`）。
- **指纹（同载具 1.21.6 + Chunky radius 500 / 同 seed / 同 region 中心 / 同工具修订，region 全域逐块普查）**：

  | 维度 | 同形态 run-to-run（async） | 跨形态（async × sync） | 接管 vs vanilla | 读数 |
  |---|---|---|---|---|
  | overworld | **0.0092%**（sync 0.0086%） | 0.0122% / 0.0130% | 0.0111-0.0148% | 低噪声维 |
  | nether | **0.1406%** | 0.1219% | 0.1020% | **~15× overworld** |
  | end | **0** | **0**（块差 0） | **7 块 / 494M** | **比对集内两 run 逐块零差** |

- **判据（MUST）**：① **维度间不可互引噪声基线**（#33 的维度面）——选载体 / 判门 MUST **先测该维自身** run 级噪声，禁拿 overworld 的 0.009% 去判 nether 的 0.12%；② **确定性维度上「非零差异即信号」**——end 噪声 = 0（两 run 逐块零差）时，等价门可升级为**位级门**（本案跨形态 0 块差是比「≤ 噪声」更强的证据）；③ 反之在 nether 这类高噪声维上，0.10-0.12% 级差异**落在噪声带内不构成任何结论**，措辞须写「未超出 run 级噪声」而**不得**写「已验证/已对齐」；④ 本指纹**机制未定**（候选：nether 结构/熔岩湖等按 chunk 完成序放置，或 region 内含非本臂生成的 chunk）⇒ 指纹可作判据使用，成因标 open（不阻塞使用）。
- **家族索引**：#33（载具/维度可比性）、#51（噪声基线与信号同阶）、#67（run 级非确定）、#83（分母/场景）、#26（载具）。

> **⛔ 数值取代 + 成因域收窄（260911-05，§15.4 取代记录，原结论正文不删不改）**——本条 `:456` 标题的「~15×」与 `:459` / `:464` / `:465` 的全部读数系**旧读法**（`tag 7` 长度按 1 字节读）产物；规范读法同载体复算（旧 → 新）：overworld async 同形态 `0.0092% → 0.0169%`（sync `0.0086% → 0.0133%`）、跨形态 `0.0122%/0.0130% → 0.0173%/0.0177%`、接管 vs vanilla `0.0111-0.0148% → 0.0192-0.0198%`；nether 同形态 `0.1406% → 0.3759%`、跨形态 `0.1219% → 0.2978%`、接管基线 `0.1020% → 0.2703%`；end **7/0/0 不变**（分母 `494,010,368 → 496,041,984`、`common` `7567 → 7569`）。标题倍数 **~15× → ~22×**（0.3759% / 0.0169%）；`common`（overworld）7959 → **8004**。
> ⇒ **判据 ①②③ 方向全部不变**（① 维度间不可互引噪声基线；② 确定性维上非零即信号 / end 可升位级门；③ nether 高噪声维上「未超出 run 级噪声」措辞禁令），**end = 0 的结论亦不变**（仅分母更新）。
> ⇒ **判据 ④ 的机制候选须标「成因域收窄」**：**不在 Rust 填充层**（同批对的新旧差纯为**读法效应**——同工具、同改法、同 region，唯一变量是 `tag 7` 长度字段；旧读法造成的 sections/chunk 失真 `11.52→24.0000` 是**解析层**失真，不是维度机制差异；Rust 填充在比对集内表现为 per-chunk 确定性），**下游未测**（原两条候选——Java 装饰/结构按 chunk 完成序放置、region 内含非本臂生成的 chunk——**均未被本块检验**）⇒ 成因标 **open（收窄：非解析层、非 Rust 填充层；下游未测）**。取代链：`record-260911-05.md` §3/§4/§5 **S4**。

---

## 发现 #22 简记: MC 1.20.1 `PalettedContainer.readPacket` 逐字节编码契约 + `BLOCK_STATE` 位宽映射指纹（含 `bits==0` 无 storage）（260911-05）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线 API 前置探针 + 实施）；**confirmed**（用户授予 2026-09-11 22:58）；algorithm-fingerprints / MC 序列化编码契约。
- **来源定位**：vanilla 一手源 `.tmp/scout-260905-08/mcsrc/`：`PalettedContainer.java:203-215`（`readPacket`）/ `:398-404`（`record DataProvider` 包私有 + `bits == 0 ? EmptyPaletteStorage : PackedIntegerArray`）/ `:420-430`（`BLOCK_STATE` 的 switch）/ `:465-467`（`computeIndex`）；`PacketByteBuf.java:859-867`（`writeLongArray` 带 VarInt 长度前缀）/ `:924-932`（`readLongArray` 仅在长度相符时复用传入数组）；`SingularPalette.java:67-68`、`ArrayPalette.java:85-90`、`BiMapPalette.java:74-79`、`IdListPalette.java:45-46`（空实现）；`PackedIntegerArray.java:257`（`Validate 1..32`）/ `:261`（`elementsPerLong = 64/bits`）/ `:266`（`longs = ceil(size/elementsPerLong)`）。一手产物 `.investigations/bulk-writeback-260911-05/api-probe-260911-05.md` §2/§3。
- **观察（逐字节契约）**：`readPacket(buf)` = `byte 请求位宽 i`（`:207`）→ `getCompatibleData(prev, i)`（按 i 选 provider）→ `palette.readPacket(buf)` → `buf.readLongArray(storage.getData())`（`:210`）→ 换 `this.data`。

  | 请求位宽 i | provider | palette 段字节 | storage 位宽 | storage 值域 |
  |---|---|---|---|---|
  | 0 | SINGULAR | `VarInt rawStateId` | 0（`EmptyPaletteStorage`，**无 storage 对象**） | —（写 0 个 long） |
  | 1-4 | ARRAY | `VarInt size` + size × `VarInt rawStateId` | **恒 4** | 局部 palette 索引 |
  | 5-8 | BI_MAP | 同上 | = i | 局部 palette 索引 |
  | ≥9 | ID_LIST | **无字节**（空实现） | `ceilLog2(idList.size())` | **全局 state id**（`idList.getRawId`） |

  - 实测自证（overworld/nether/end 三维一致）：`state_ids_size=24137`、`idlist_bits=15`、`max_distinct` = 7 / 7 / 2、`idlist_hits=0`。
  - **零布局转换**：`computeIndex(x,y,z) = (y<<4|z)<<4|x` 与 Rust buf 的 y-major 切片布局逐位相同 ⇒ section s 恰是连续切片 `buf[s*4096,(s+1)*4096)`。
  - ⚠️ **`readLongArray` 的静默丢弃**：`readPacket` **丢弃返回值**；`readLongArray(toArray)` 仅在 `toArray.length == VarInt 长度` 时复用传入数组，否则**新建数组并返回**（读进来的值被丢掉，storage 保持原值）——**长度不符不抛异常、不报错**（对比 `PackedIntegerArray(elementBits,size,long[])` 构造器在长度不符时抛 `InvalidLengthException`）。⇒ 自建 buf 时 storage long 数必须精确等于 `ceil(size/(64/bits))`（**推导值**：4bit → 256 longs、15bit → 1024 longs）。
- **如何利用（判据）**：
  1. 要**整段导入** `PalettedContainer` 时用公开 `readPacket`（自建 buffer），不必逐点 `set`；但 MUST 自复刻 `BLOCK_STATE` 的 switch（provider 类型**包私有**，包外不可命名）。
  2. **位宽 0 = 无 storage，不是「0 位 storage」**——不得对 0 位构造 `PackedIntegerArray`（E1：`Validate 1..32` 抛 IAE，首跑 539 次）。
  3. 复刻 MC 的 switch **逐 case 过目标构造器前置条件**（第 0 支没有 storage 对象）。
  4. 手写该 buffer 的**长度字段必须按契约对表**（palette 段有无、VarInt 前缀），错一位即「静默丢数据 + 无异常」（与 build-tooling #58 同族：单点长度笔误 → 结构级失真）。
- **家族索引**：algorithm-fingerprints #23（同案计数语义差）、compiler-idioms #6（raw id vs state id 双域——本契约 ≥9 支走 state id 域）、compiler-idioms #24 补充案例（`readPacket` 自带锁）、build-tooling #58（长度字段对表）。

---

## 发现 #23: `ChunkSection` 三个派生计数在 vanilla 内部有**两套语义**——`calculateCounts()`（全量重算）≠ 增量 `setBlockState`，且三计数**不入存档**（260911-05，最高价值·错误优先）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线实施期，为绕过 72.5µs/section 读 `calculateCounts` 源码时发现）；**confirmed**（用户授予 2026-09-11 22:58）（Tier 2 逐 chunk 全等 + 一手源对表；confirmed 留人类）；algorithm-fingerprints / MC section 派生状态语义（完整错误链见 `.investigations/bulk-writeback-260911-05/errors-260911-05.md` E2）。
- **来源定位**：`ChunkSection.java:27-31`（构造器自动 `calculateCounts()`）/ `:60-93`（增量 `setBlockState`）/ `:111-140`（`calculateCounts`）；`NoiseChunkGenerator.java:416`（生成期走增量、`lock=false`）；`ChunkSerializer.java:310-311`（只写 `block_states`/`biomes`）；实现 `BulkWb.buildContainer` ②段 + `ChunkSectionAccessor`（4 写 + 3 读）。
- **观察（逐条对表）**：

  | 计数 | 增量 `setBlockState` | `calculateCounts` |
  |---|---|---|
  | `nonEmptyBlockCount` | 非空气 +1 | 非空气 +1 **且流体非空再 +1**（同格重复计入） |
  | `randomTickableBlockCount` | 非空气 ∧ `hasRandomTicks` | 同 |
  | `nonEmptyFluidCount` | **流体非空** +1 | 仅**流体自身 `hasRandomTicks()`**（静水 `WaterFluid.Still.hasRandomTicks()=false` ⇒ 静水 section = 0，增量 = 4096） |

  - 实测成本：构造器路线 `calculateCounts` = **72,480 ns/section**（占被替换 section 成本 64%）；自算（去重遍顺带，O(distinct)）= **116 ns/section**。
  - 生成期语义由**调用点**决定：vanilla `populateNoise` 走增量 ⇒ 增量语义才是生成期 vanilla 语义（也是老逐块路径语义）。
  - **三计数不入存档** ⇒ region 对拍看不到；唯一可见面 = 生成期到下次存读之间的**内存态**：`hasRandomFluidTicks()`（→ 流体随机刻调度）、`isEmpty()` / `nonEmptyBlockCount`（→ 客户端包 `toPacket`）。
  - 等价性验证：Tier 2 派生字段指纹（`dh=` = 三计数逐 section 序列）**607/625/625 逐 chunk 差 0**（三维持；本稿独立复算自归档抽取件）。
- **如何利用（判据）**：
  1. **同一语义在 vanilla 内部有两套实现时 MUST 逐条对表**；生成期语义按**调用点**定，不按「哪个 API 更现成」定。
  2. **「构造器会自动算好」是危险直觉**——它算的是另一套定义（走 `new ChunkSection(pc,biome)` 会静默改变 `hasRandomFluidTicks()` / 客户端包语义）。
  3. **等价门必须分层**：状态层指纹（方块 id 序列）对派生字段语义差**结构性不敏感** ⇒ 派生字段必须**独立成门**。
  4. 判定某派生字段「不入存档」时 MUST 同时声明其**唯一可见面**，否则会被误读成「region 对拍过了 = 没问题」。
- **家族索引**：compiler-idioms #24 补充案例（同 section 的锁语义）、algorithm-fingerprints #22（同案编码契约）、workflow-patterns #129（本案 Tier 3 判据降级）、workflow-patterns #14 补充案例（层不匹配的门零判别力）。

---

## 发现 #24 简记: vanilla 分块容器「规范编码字节」= **storage 的 elementBits**，不是请求位宽——「非规范但解码等价」= 对 switch 宽容度的不必要依赖（260911-05）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线实施 + judge S5 复核并扩大）；**confirmed**（用户授予 2026-09-11 22:58；一手源逐行直读 + 本块实测偏差登记）；algorithm-fingerprints / MC 序列化编码规范性。
- **来源定位**：vanilla 一手源 `.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/PalettedContainer.java`：`:383-387`（`Data.writePacket`：`buf.writeByte(this.storage.getElementBits())` ← **规范字节的唯一来源**）/ `:420-430`（`BLOCK_STATE.createDataProvider` 的 switch）/ `:125-130`（`getCompatibleData` → `createDataProvider(this.idList, bits)`：**解码侧把收到的字节原样喂回同一个 switch**）/ `:203-215`（`readPacket` 读 `byte i`）；实现侧 `BulkWb.java:243`（写 `pb.writeByte(bits)`，`bits = MathHelper.ceilLog2(distinct)`，见 `:234`）与 `:249-250`（`storageBits = idList ? ceilLog2(Block.STATE_IDS.size()) : (bits <= 4 ? 4 : bits)`）。
- **观察（规范值 vs 本块写值，逐支）**：

  | 分支 | `createDataProvider` 分支（`:423-428`） | storage 实际位宽 | **规范字节**（`writePacket:384`） | 本块 `BulkWb.java:243` 写出值 |
  |---|---|---|---|---|
  | SINGULAR | `case 0 → DataProvider(SINGULAR, 0)` | 0（`EmptyPaletteStorage`，无 storage 对象） | **0** | 0 ✔ |
  | ARRAY | `case 1,2,3,4 → DataProvider(ARRAY, **4**)` | **恒 4**（与请求位宽无关） | **4** | **1..4**（= `ceilLog2(distinct)`）✘ 非规范 |
  | BI_MAP | `case 5,6,7,8 → DataProvider(BI_MAP, bits)` | = 请求位宽 | = 请求位宽 | 5..8 ✔ |
  | ID_LIST | `default（≥9）→ DataProvider(ID_LIST, ceilLog2(idList.size()))` | **15**（`ceilLog2(Block.STATE_IDS.size())`；实测 `state_ids_size=24137`） | **15** | **9/10**（= `ceilLog2(distinct)`）✘ 非规范 |

- **为什么「解码等价」**：`readPacket:207-208` 把读到的字节 `i` 原样交给 `getCompatibleData(:125-130)` → `createDataProvider(idList, i)` ⇒ 同一个 switch 把 **1..4 全映到 `ARRAY(4)`**、把 **≥9（含 9/10）全映到 `ID_LIST(15)`** ⇒ 写 1..4（ARRAY 支）或写 9/10（ID_LIST 支）**解码结果完全等价**（本块 Tier 1/2 逐 chunk 全等亦旁证）。
- **为什么仍是脆弱点（判据）**：这是**对 switch 宽容度的不必要依赖**——「今天无缺陷、移植时咬人」：1.21.6 或未来版本若**收窄 switch 分支**、或改动 `getCompatibleData` 的「可复用 previousData」判定，同一字节流会**静默变行为**（无异常）。
  - **判据（MUST）**：**编码器 MUST 写 storage 位宽**（与构造 `PackedIntegerArray` 用的 `storageBits` **同源**），不得写「请求位宽 / 局部 distinct 位宽」。
  - **精确修法**（`BulkWb.java:243` 处，与 `:249-250` 同源）：
    `int storageBits = (bits == 0) ? 0 : (idList ? MathHelper.ceilLog2(Block.STATE_IDS.size()) : (bits <= 4 ? 4 : bits)); pb.writeByte(storageBits);`
- **本块处置（登记不修）**：偏差 = ARRAY 支写 1..4、ID_LIST 支写 9/10（`record-260911-05.md` §7 遗留 7）；judge 最小上呈条件不含 S5，且改码会使 6 臂证据全部失效 ⇒ **留待 1.21.6 移植随共享 Java 适配核一并修正**。⚠️ 因该偏差存在，本块「编码四支全绿」的合成自检**不构成规范性证据**（自检 ARRAY 例 distinct=3 恰好覆盖该非规范路径，见 workflow-patterns #110 补充案例 其二）。
- **交叉引用（不重复展开）**：`bits == 0`（SINGULAR）时 storage = `EmptyPaletteStorage`，**不得构造 `PackedIntegerArray`**（elementBits=0 触发 `Validate 1..32` 抛 IAE，260911-05 首跑 539 次）——见 `发现 #22` 与错误台账 E1，此处不重复。
- **家族索引**：algorithm-fingerprints #22（**同代码点的解码侧**逐字节契约——本条为**编码侧规范性**）、#23（同 section 的派生计数两套语义）、compiler-idioms #6（raw id vs state id 双域——ID_LIST 支走 state id 域）、compiler-idioms #15（位打包标量域的「部分位判零」——同属「位宽/掩码约定」家族）、workflow-patterns #14 补充案例（门必须与被测变更同层）。
