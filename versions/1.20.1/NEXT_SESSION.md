# CoreSwap 下一会话交接（2026-08-06 深夜切会话）

> 主主线：**负坐标 bug**。本文档是唯一权威交接，先读这个再动手。

## 当前唯一主主线：负坐标 bug（块状断裂地形）

**现象**：负坐标区域 C++ 生成的地形断裂/浮空（用户验证 seed `8576294172403134396`，玩家降落 (731,82,-404)）。正坐标 100% 逐位一致，负坐标才触发。

### 已确认的事实（勿重复排查）

1. **差异模式**：C++ 生成比 vanilla 地表低/断裂。seed -8248 负坐标 chunk(-18,-16) 列 (8,8)（世界 -280,-248）：
   - cns 游戏实际密度（DensityInterpolator.sample）：y 48 = +0.213（正）、y 52 = -0.010（负）——**过零 51-52**
   - C++ 方块：y 40-51 实心 ✅、**y 52-60 实心 ❌（应空气）**、y 61-64 空气 ✅、**y 65-99 全 stone ❌（应空气，cns y 65=-0.40）**
2. **排除项**（全部验证过，别重查）：
   - Perlin 实现（c2me 源码确认 vanilla 原样，C++ 已核对）
   - maintainPrecision（已修复：Java 是 `(long)(v/3.35e7+0.5)` 截断语义，C++ 已对齐；小坐标不触发折叠）
   - FlatCacheDF/Cache2DDF 缓存（key `(uint32)x<<32 ^ z` 负坐标唯一；网格索引 kc/lc 用算术右移一致）
   - InterpolatedDF 插值（gx/gy/cz = pos - chunk*16 均非负，负坐标与正坐标同路径）
   - 取模/移位/GRADIENTS 表/deriver
3. **A 方案（cns 游戏实际参照）已跑通**——这是当前最强诊断工具：
   - DensityProbe.java 反射 cns 完整链：`sampleStartDensity()` → 循环 `sampleEndDensity(cellX)` → `onSampledCellCorners(cellY,cellZ)` → `interpolateY/X/Z(世界坐标, progress)` → **`DensityInterpolator.sample(cns)`**（interpolators 字段 get(0)——字段名 `interpolators` 不是 `interps`）
   - **注意**：不能调 `sampleBlockState`（aquifer 单 chunk 探针越界 `Index 358`——探针缺周围 chunk 上下文）；必须用 DensityInterpolator.sample
   - cell 尺寸：水平 4、垂直 8；cellHeight=48；minCellY=-8；blockY = (minCellY+cellY)*8+vb（世界 y）；blockX/blockZ 必须世界坐标（chunkStartX + cellX*4 + cbx）
   - 跑法：`gradle runServer --no-daemon -PdensityProbe=true -PdensityProbeDimension=overworld -PdensityProbeChunkX=-18 -PdensityProbeChunkZ=-16 -PdensityProbeX=8 -PdensityProbeZ=8 -PbenchSeed=-8248318472910187742`
   - 输出 `data\vanilla_density_overworld_c-18_-16_b8_8_cns.txt`

### 下一步（按优先级）

1. **dump C++ 的 densityBuf 原始值**（fillOneChunk 内部，不经 aquifer/surface）对比 cns——**区分「density 错」vs「aquifer/surface 错」**
   - 在 fillOneChunk 的 densityBuf 填充处（worldgen_api.cpp ~534 行）加 WG_DBDEBUG 环境变量条件打印列 (bx,bz) 的原始密度
   - 对比 cns 反射值：若 density 一致 → 错在 aquifer/surface；若 density 就差 → 错在密度树（负 x/z 的某分量）
2. **修复 got_export 的 -densityDump**：它**硬编码下界**（`wg_create(..., "nether.json", "biome_params_nether.json", 256)`，忽略 dimension 参数）——主世界 dump 必须用 `-namedDump final_density -18 -16 8 8 -dimension 0`（但 namedDump 目前全 0——final_density 的 registry 名不对，需查 builder 注册的 key）
3. **WG_SURFDUMP 诊断**：worldgen_api.cpp ~547 行已有 dump 列表面高度/initial/final 剖面——可对比 cns
4. **若 aquifer/surface 错**：负坐标的 aquifer 表面估计（estimateSurfaceHeight）或 surface 规则遍历
5. **兜底**：noise-in-Java 开关（v1.2 迁移工具，docs/09 有设计）——噪声交 Java 立即解决（但不优先）

## 工具与命令速查

- C++ 构建（MSVC，禁止 MinGW）：`cmd /c "call \"D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat\" && set PATH=\"...Ninja\";%PATH% && cmake --build build-msvc"`（改 .h 后需 touch worldgen_api.cpp 强制重编——ninja 头依赖未跟踪）
- 主世界回归：`block_probe -8248318472910187742 E:\python\MC\data\worldgen E:\python\MC\data\vanilla_-8248318472910187742_4_3200_3208.blocks` → 必须 100%
- 负坐标参照：`data\vanilla_-8248318472910187742_4_-288_-256.blocks`（格式：32B 头 magic/seed/size/originX/originZ/minY/height + 每 chunk 8B pos + 16*16*384 short 大端）
- c2me 源码已 clone 到 `E:\python\MC\data\c2me-fabric`（MixinNoiseChunkGenerator 有完整 populateNoise 链，MixinChunkNoiseSampler 有 cacheAllInCell 语义）
- 线程：`-PcoreswapThreads=N` → `-Dcoreswap.threads` → C++ `physicalCoreCount()`（Windows API 物理核）+ `CORESWAP_THREADS` env

## 已发布版本（勿重复发）

1.0.4/1.0.5/1.0.6/1.0.7/1.0.8 均已发布。1.0.8 = dll 版本化（哈希对比自动替换缓存 dll，修 XuanRikka 的更新不替换问题）。当前 build.gradle version = 1.20.1-1.0.8（若改代码需 bump + 发布按铁律：MSVC dll + dumpbin 导入表 + block_probe 100% 回归 + jar 内 dll 验证）。

## 重要铁律（勿违反）

- 知识库 docs/ 追加式更新，禁止覆盖（用户明确）
- 提交 author 必须 unknowbug，中文提交信息
- 主世界 100% 是铁律——任何改动后必须回归
- 不在 GitHub Issue 直接回复（除非用户明确指示）
- 全版本覆盖是真实目标（对外文档禁止「不计划」措辞）
