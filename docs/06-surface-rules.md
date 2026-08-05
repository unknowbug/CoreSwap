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
