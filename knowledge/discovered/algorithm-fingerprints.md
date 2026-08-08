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
