# 7. 块级流水线与性能（worldgen_api.cpp）

## 功能目的

单 chunk 从 seed 到 16×16×384 方块的完整流程，以及性能优化记录（多线程 + 缓存）。

## 1.20.1 工作机制（fillOneChunk）

```
wg_create（一次性）：
  DensityBuilder（JSON → 密度树）+ blocks registry + biome + surfaceBuilder + overworldRule 预构建

fillOneChunk(cx, cz)（每 chunk）：
  1. aquifer = Aquifer(barrier, floodedness, spread, fluidType, initialDensity, ...,
                       split("minecraft:aquifer").nextSplitter(), sh4 4 角估计)
  2. oreVein = OreVeinSampler(vein_toggle, vein_ridged, vein_gap, split("minecraft:ore").nextSplitter())
  3. 块循环 by(0..383) → bz → bx：
       density = finalDensity.sample(x, y, z)          # 树内 InterpolatedDF 自行插值
       block = aquifer.apply(density)                  # -1 表示 null
       if (block < 0) block = oreVein.apply(x, y, z)   # ChainedBlockSource
       if (block < 0) block = stone                    # 默认
       heightmap 更新
  4. buildSurface（列引擎 + surface rules）覆盖表层
  5. memcpy → out
```

**ChainedBlockSource 顺序**：aquifer 先，返回 null 才轮到 oreVein，再 null 用默认 stone。
**高度图**：WORLD_SURFACE_WG（最高非空气块），SURFACE 阶段起始列顶。

## 性能基准（2026-08-06，seed -8248318472910187742，4×4 chunks）

| 方案 | 耗时（16 chunks） | 备注 |
|---|---|---|
| 初始串行 | ~1800ms | aquifer 无缓存 |
| + aquifer 列缓存 | 1056ms | estimateSurfaceHeight ~2700 倍降幅 |
| + 多线程并行（自适应线程） | **110ms** | ~9.6×；100% 保持 |

### 热点分布（优化前）

- **density 采样 ~12ms/chunk（12%）**：base_3d_noise 逐块 24 次 Perlin。
- **aquifer+oreVein 59-562ms/chunk（88%）**：根因是 `estimateSurfaceHeight` 无缓存
  （每块 13 邻居 × 最多 49 次 initialDensity 采样 ≈ 3200 万次/chunk）。

### 优化手段（全部无损，结果逐位一致）

1. **aquifer 列缓存**：`estimateSurfaceHeight` per-chunk `map<列key, 高度>`，key = `((x>>2)<<2, (z>>2)<<2)`（Java surfaceHeightEstimateCache 同款）。
2. **chunk 级多线程**：`wg_fill_blocks_multi`，std::thread 池 + 原子任务索引；`threads<=0` 自适应 `min(hw, count)`，clamp 不超过任务数。
3. **线程安全**（多线程前提）：
   - `InterpolatedDF` 缓存 → per-instance `thread_local`（O(1) ID 索引 vector，非 std::map）
   - `overworldRule` 预构建到 wg_create（消除懒构建竞态）
   - aquifer/SurfaceContext/oreVein 均 per-chunk 局部对象 ✅
   - `split()`/`split(name)` const 纯函数 ✅；`nextSplitter()` 有状态 → 每线程独立派生

### ⚠️ 为什么不做「base_3d_noise 网格插值」优化

Java 的 base_3d_noise **逐块重算 24 次 Perlin（无缓存）**。若 C++ 改为 cell 网格插值缓存，
会引入浮点误差**破坏 100% 逐位对齐**。多线程是唯一无损的大优化。
若未来追求进一步加速且可接受非逐位一致（如 ±1 块误差），需先经用户确认。

## 版本敏感点

- [ ] **fillOneChunk 的调用顺序**：ChainedBlockSource 的 sampler 顺序（aquifer→oreVein）随版本变（1.17 无 oreVein）。
- [ ] **fluidLevelSampler 默认**：`y < -54 lava else water`（主世界）——版本/维度敏感。
- [ ] heightmap 类型（WORLD_SURFACE_WG）与 buildSurface 起始。
- [ ] out 布局 `(y+64)*256 + z*16 + x` 与 vanilla raw id（air=0）——与 Java 导出格式绑定。

## 已验证的坑

- **块级插值顺序**：finalDensity 整树角点采样+手动插值是错的（非线性不可交换），必须块级直接采样（03 篇）。
- **线程数默认**：不要用单台机器的 hardware_concurrency 写死；API 默认自适应 + 调用方 `-threads` 可配。
- **验证多线程一致性**：block_probe 并行 vs got_export 串行，TOTAL 必须同为 100.0000%；
  任何差异说明有隐藏竞态（检查 mutable 缓存/懒构建）。
