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
| + 纯算法优化（无损）串行 | **450ms（28.1ms/chunk）** | 见下 |
| + 纯算法优化（无损）并行 | **49.4ms（3.1ms/chunk）** | 110→49.4ms（-55%） |

### 纯算法优化链（全部无损、100% 保持，2026-08-06 第二批）

**方法**：WG_PROFILE 剖析（阶段计时 + 计数器）→ 数据定位冗余 → 逐项修复。

1. **FlatCacheDF + Cache2DDF（最大结构性冗余！）**：Java ChunkNoiseSampler 对
   `minecraft:flat_cache`/`minecraft:cache_2d` 有真正缓存（FlatCache 5×5 网格预计算、Cache2D 列缓存），
   而 C++ 此前仅"语义委托"（每块重算）——**spline 采样 34900 → 6250 次/chunk**（大陆样条
   continents/erosion/ridges/factor/jaggedness/offset 每块重算是 Java 官方算法里已被缓存的冗余）。
   - FlatCache：per-instance thread_local 5×5 网格（Java: horizontalBiomeEnd+1=5，间距 4 块，y=0）；
     **边界点（x=cx*16+16）命中本 chunk 网格 k=4，不重建**（否则嵌套采样递归重建相邻 chunk 网格）。
   - Cache2D：per-instance thread_local 单列缓存（Java: lastSamplingColumnPos）。
2. **aquifer blocks->id() 预取（隐藏最大单点收益）**：apply 每块 4-7 次 `blocks->id("air"/"water"/"lava")`
   **std::map 字符串查找**（40-70 万次/chunk）——构造时预取三常量 → **aquifer 25ms→8.9ms（-64%）**。
3. **aquifer 列缓存 std::map → flat 数组**：estimateSurfaceHeight 13 邻居查询每块 13 次 map 查找
   → 32×32 数组（覆盖 ±48 格邻居范围）O(1)。
4. **oreVein y 范围预检查**：y∈[-60,50] 外提前返回（Java 采样 veinToggle 后同样返回 -1，结果一致）。
5. **surface 噪声列缓存**：NoiseThreshold（per-instance thread_local 单槽）/sampleRunDepth/getTerracottaBlock
   ——buildSurface 逐列处理，同列 2D 噪声采样位置相同 → 每列 1 次。

**陷阱**：surface 噪声缓存放 SurfaceContext 的 std::map 反而变慢（每块字符串查找）；
正确做法 = cond 实例的 thread_local 单槽（O(1)）+ 多线程安全。

### 当前热点（串行 28.1ms/chunk，WG_PROFILE 数据）

| 阶段 | 耗时 | 构成 |
|---|---|---|
| density | 8.5-11.7ms | base_3d_noise 122 次/chunk（插值网格角点）、spline 6250 次（FlatCache 构建）、98k 块树遍历 |
| aquifer+oreVein | 6.5-8.9ms | 72% 块走 18 候选遍历（Java 同构，无法无损减少） |
| sh4+surface | 9-10.7ms | 98k 块规则遍历（Java 同构）+ 每块 VerticalGradient 随机 |

**结构上无法再无损压缩的**（Java 同构）：aquifer 18 候选遍历、surface 规则遍历、
VerticalGradient 每块 split(x,y,z)（依赖 y）。

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
