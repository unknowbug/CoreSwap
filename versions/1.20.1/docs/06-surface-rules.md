# 6. 表面规则（surface.h）

## 功能目的

NOISE 阶段生成基础方块（stone/water/air）后，SURFACE 阶段按 biome/深度/噪声条件覆盖表层：
草地、泥土、沙子、雪、基岩、深板岩、水域/冰面等。

## 1.20.1 工作机制

### 规则树（buildOverworldRule）

```
finalRules（顺序匹配，第一个非 null 生效）：
  bedrock_floor（verticalGradient -64..-59 → bedrock）
  surface() → materialRule9        # 大规则树（含草/沙/雪/沼泽/山地等）
  deepslate（verticalGradient 0..8 → deepslate）
```

Java 的 `surface` 参数 = `condition(surface(), materialRule9)`（materialRule10），C++ 等价。

### 条件原语（MaterialRules）

| 条件 | 语义 |
|---|---|
| `biome(...)` | 当前块 biome 匹配 |
| `stoneDepth(offset, addSurfaceDepth, secondaryRange, ceiling)` | 见下 |
| `water(offset, mult, addStoneDepth)` | 见下 |
| `surface()` | `blockY >= estimateSurfaceHeight()` |
| `verticalGradient(name, from, to)` | y 渐变 + 随机（splitterFor(name).split(x,y,z).nextFloat() < d） |
| `noiseThreshold(name, min, max)` | 噪声值范围 |
| `not/and/or` | 组合 |
| `STONE_DEPTH_FLOOR` / `STONE_DEPTH_CEILING` | 快捷条件 |

### ⚠️ StoneDepth 语义（曾误判为 ==0）

```cpp
// Java 公式（MaterialRules.StoneDepthPredicate.test）
int i = ceiling ? stoneDepthBelow : stoneDepthAbove;
int j = addSurfaceDepth ? runDepth : 0;
return i <= 1 + offset + j + k;     // k = secondaryDepthRange 插值，通常 0
```

- `STONE_DEPTH_FLOOR` = `stoneDepth(0,false,0,FLOOR)` → `stoneDepthAbove <= 1`（不是 ==0！）
- `STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH` = `stoneDepth(0,true,0,FLOOR)` → `stoneDepthAbove + runDepth <= 1`
- 曾因此误判：`i <= 1+offset` 写成 `== offset`，导致草地只在 surface 那格生成、斜坡草皮丢失。

### Water 条件

```cpp
if (fluidHeight == INT32_MIN) return true;      // 无流体 → 恒真
return blockY + (addStoneDepth ? stoneDepthAbove : 0) >= fluidHeight + offset + runDepth * mult;
```

### buildSurface 列引擎（逐列从顶向下）

```
q = runDepth（连续非空气块计数）；r = fluidHeight（最高流体 y+1）；s = 下方第一个非默认块
vx = wy - s + 1
每块：isAir → q=0, r=MIN；isFluid → r=wy+1；default → q++, initVertical(q, vx, r, ...) → rule.apply(ctx)
```

- **s 语义**（Java 144-150）：从 wy-1 向下找第一个**非默认块**（默认块=stone），`s = v+1`。
- **initVerticalContext 参数顺序**：(stoneDepthAbove=q, stoneDepthBelow=vx, fluidHeight=r, x, y, z)。
- `default` 块才应用规则；`rule.apply` 返回 -1 保持原样。

### estimateSurfaceHeight（surface() 条件）

```cpp
return (int)floor(lerp2(fx, fz, 4角高度));   // 4 角 = chunk 四角 estimateSurfaceHeight（4 格对齐）
```

**⚠️ 无 `+ runDepth - 8` 偏移**——Java 源码有 surfaceMinY = k + runDepth - 8，但实测去掉后 100% 对齐
（该处 runDepth 语义与 buildSurface 的 q 不同，本实现以实测为准）。

### materialRule1..10 结构要点（1.20.1）

- `mr`（grass/dirt）：`sequence(condition(water(0,0), grass_block), dirt)`——**表层草皮**。
- `mr4`：山峰/海滩/干旱系列（stony_peaks/stony_shore/windswept/sandstone/dripstone），无通用 fallback。
- `mr7`：**结尾 = MANGROVE_SWAMP→MUD + DIRT fallback**（不是 taiga/mushroom！曾误放 mr8 分支导致草皮泄漏）。
- `mr8`：frozen/snowy/jagged/grove/windswept + taiga/ice_spikes/mangrove/mushroom + **mr（grass fallback）**。
- `mr9`：STONE_DEPTH_FLOOR 段（badlands/湿地）+ 海洋段（water/frozen/sand）+ gravel fallback。

## 版本敏感点

- [ ] **materialRule1..10 的分支归属**：新版本直接 diff VanillaSurfaceRules.java 的 materialRule7/8 等定义——**每个规则的行号/嵌套顺序都变**，必须逐规则对照，不能平移。
- [ ] **StoneDepth 公式**（`i <= 1+offset+j+k`）与快捷条件参数。
- [ ] **buildSurface 的 s 语义**（非默认块 vs 空气的判定集合）。
- [ ] **estimateSurfaceHeight 的 4 角插值**与 biome 对齐。
- [ ] surface() 与 STONE_DEPTH 的层级关系（1.19+ surface rules 重构过）。

## 已验证的坑

- **mr7 误放 mr8 分支**：C++ 曾把 taiga/ice_spikes/mushroom/mr 塞进 mr7 结尾，导致非表面位置也生成 grass_block（dirt→grass 200 块）——**对照 Java 时逐行核对规则归属，别只比对分支数**。
- **s 判定集合**：Java `isDefaultBlock`（==stone）vs C++ 早期只认 air/water/lava——非默认块（gravel 等）的处理集合必须一致。
- 验证方法：`[sf2]` 打印 before/after + biome 对照；或直接对差异块驱动 buildSurface（08 篇）。

## 2026-08-08 已验证结论（自 10 时间线归档提炼，完整过程见 10-timewise-archive.md）

### ✅ BiomeAccess 8 邻域选点（已修复）——8576 99.8473%→99.8892%
- Java `BiomeAccess.getBiome(BlockPos)` **不是 floor 采样**：pos-2 → 8 邻域角点 (l,l+1)×(m,m+1)×(n,n+1) + seed 哈希扰动距离选最近（method_38106）
- `hashSeed(seed)` = `Hashing.sha256().hashLong(seed).asLong()`（Guava putLong 小端 → SHA-256 → 前 8 字节小端）
- `mixSeed(seed, salt)` = `seed*(seed*6364136223846793005L + 1442695040888963407L) + salt`（64 位无符号回绕）
- `method_38108(l)` = `(floorMod(l>>24, 1024)/1024 - 0.5) * 0.9`；method_38106 六次 mixSeed 扰动选最近角点
- C++ biomeAt 曾直接 `(x>>2)<<2` floor → 判错 biome（savanna vs eroded_badlands）→ 不产 terracotta；修复后 (805,64,-432)=eroded_badlands 与 Java SURFBIOME 一致
- **biomeAtCached 缓存 key 必须用选点坐标 packed**（原 (x>>2,y>>2,z>>2) 会错误复用：同 4 格内不同 y 的 8 邻域选点不同）

### ✅ above_preliminary_surface（SurfaceCondC）公式修复——8576 99.8892%→99.9768%
- Java（MaterialRules.java:567-572）= `blockY >= estimateSurfaceHeight()`，其中 `estimateSurfaceHeight()`（488-516）= `floor(lerp2(4 角 est)) + runDepth - 8`
  - 4 角 est = chunk 4 角 cns.estimateSurfaceHeight；lerp2 参数序 `lerp2((blockX&15)/16, (blockZ&15)/16, e00, e10, e01, e11)`
- C++ 旧公式 `blockY + surfaceDepth + 4 >= est` **完全不等价**（缺 4 角插值 + runDepth-8）→ 修复 `blockY >= k + surfaceDepth - 8`（k = floor(lerp2 4 角 est)）
- 验证：8576 99.8892%→**99.9768%**（chunk(50,-23) 99.59%→100%）、3200 99.8814%→**99.9995%**

### ✅ terracotta 带相关（全部一致/排除）
- terracottaBands 192 带数组：C++/Java 逐位一致（tbands_dump + RouterProbe TBANDS 对比）
- clay_bands_offset、sampleRunDepth 一致；带 y57/58 错位疑点 = BiomeAccess 8 邻域问题（已解）

### ✅ 表面规则条件链全验证一致
- runDepth：`(int)(surface*2.75+3.0+split(x,0,z).nextDouble()*0.25)` == C++
- aboveY：`y + stoneDepthAbove >= anchor + runDepth*mult` == C++
- stoneDepth：`stoneDepthAbove/Below <= 1+offset+(addSurfaceDepth?runDepth)+k` == C++
- estimateSurfaceHeight（扫描）== C++（见 04 篇）

### ✅ heightmap 索引修复
- buildSurface 遍历 heightmap[k*16+l] 应为 heightmap[l*16+k]（z*16+x）——-288 95.47→95.72%、8576 99.58→99.80%

### ✅ 参照状态/seed 校验铁律（对比前必读）
- BlockProbe 导出表面是 SURFACE（594 行 getChunk）但实际含 FEATURE/结构（连带推进）——**对比前必须过滤 FEATURE/结构方块，或参照导出时 simulation-distance=2 + 删 world**
- **参照文件实际 seed 看 `[BlockProbe] worldSeed=` 打印**，不能只看文件名/header 的 benchSeed（server.properties level-seed 硬编码 8576；-288 参照是 -8248 世界）

### ❌ 未解（下一轮候选）
- 8576 剩余 826 块 = terracotta 带边缘（y=100-108 地表带，C++ 判 air vs Java terracotta）
- 参照深层 terracotta 带（y=-32 单层/带，badlands 段 STONE_DEPTH_FLOOR 不覆盖）来源未明（假 diff 候选）
- 洞穴底 dirt（参照 (739,-427) 洞穴底 y=56 dirt vs C++ stone，est=64 不满足 above_preliminary）来源未明（假 diff 候选）
- 16 格宽「地貌同构划线」（biome 相关，疑 FlatCache 网格角点值特定位置差）

---

## 2026-08-08 已验证结论（追加 2）：placeBadlandsPillar 修复（eroded_badlands terracotta 带）

- **现象**：8576 chunk(50,-26) 797 块差（C++ air vs Java terracotta 带），参照（SURFACE 导出）y=69-118 terracotta，NOISE 阶段 y=74-118 是 air。
- **破案链**（10 时间线 2026-08-08）：squeeze 一致排除 → Beardifier 否证（badlands 无结构贡献）→ Blender 否证（无旧世界数据=恒等）→ Diag810（NOISE=air vs SURFACE=terracotta）→ **worker 定位 Java SurfaceBuilder.placeBadlandsPillar（L208-234）：eroded_badlands 每列先算 pillar 顶 j=64+min(e²·2.5, ceil(h·50)+24)，把 y≤j 的 air 填成 stone → heightmap 抬升到 j+1 → 主循环起点变高 → badlands 段规则（blockY+q≡j+1 恒真）→ terracottaBands 染色**。
- **C++ 修复**（surface.h placeBadlandsPillar，@anchor PILLAR#001）：补 air→stone 填充 + heightmap 抬升 + 主循环起点重采样；规则树本体原已对齐。
- **验证**：8576 99.9768% → **99.9993%**（820→24 mismatch）；3200 干净参照回归 **99.9997%**（4 mismatch，零退化）。
- **3200 参照污染（重要）**：anilla_-8248318472910187742_4_3200_3208.blocks 在 8/8 00:02 被 8576 世界重导覆盖（server level-seed 固定 8576 但 benchSeed=-8248）——**不能只看文件名/header 的 benchSeed**；已重新导出干净参照（16:16，worldSeed=-8248 核对）。
- **剩余 24 mismatch**（8576）：散落边缘（forest terracotta×2、savanna 水/深板岩 ~20、river），非 pillar 范围，待立项。
