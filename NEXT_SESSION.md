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

---

## 2026-08-07 深夜追加：负坐标/大坐标块状根因已确认（InterpolatedDF 插值语义）

### 根因（决定性证据）

**C++ `InterpolatedDF`（density.h）插值「整棵 argument 树」——把 range_choice/min/squeeze 等非线性阶梯也平滑了；vanilla 的 `minecraft:interpolated` 只插值 argument 树中的「噪声节点」，非线性在插值后应用（保留阶梯）。**

证据（veinToggle，range_choice y 范围 -60..51，interpolated 包着）：
- y=52（max_exclusive=51，范围外）：vanilla=0.00000、C++=-0.02177 ❌（C++ 插值把噪声拖过边界）
- y=-60（min_inclusive 边界）：vanilla=-0.11003、C++=-0.05142 ❌（C++ 平滑拉向 0）
- y=-56/48（远离边界）：一致 ✅

影响面：
- **vein_toggle/vein_ridged**（range_choice 边界 -60..51）→ -288 区域 granite/diorite/tuff 缺失、gold_ore 错位
- **finalDensity**（sloped_cheese 的 range_choice 阈值 1.5625 + min/squeeze）→ 20000 区域 y40 差 0.29 + y52 符号翻转
- 3200「100%」= NOISE 状态旧参照（无 ORE/vein 产物）+ 该区域非线性恰好平缓（非坐标相关，位置巧合）

### 已排除（勿重查）
- yScale 第 4 参：javap 确认 1.20.1 DoublePerlinNoiseSampler 只有 3 参 sample（NoiseDF 3 参+乘 yScale 与 Java 一致）
- maintainPrecision：已对齐；超大坐标（134313,434419）b3d 差 6.6e-5（正常舍入，无折叠跳变）
- barrier/fluid/veinGap/continents/erosion：router 组件全 0 一致
- cns 反射链：interpolators 是 8 个组件插值器（get(0) 不是 finalDensity）；DensityInterpolator.sample 依赖 cns 遍历状态，反射输出不可信

### 修复方向（未开始）
把 InterpolatedDF 改为 vanilla 语义：遍历 argument 树，标记「噪声型节点」（minecraft:noise/shifted_noise/weird_scaled_sampler/old_blended_noise/spline 噪声），每个分配独立 cell 网格（角点采样该噪声），插值后重建树（非线性后置）。density_builder buildNode 层面做变换最省（不改每类 DensityFunction）。

### 新工具/诊断（本次会话）
- `WG_DBDEBUG`（worldgen_api.cpp fillOneChunk）：dump 指定列 densityBuf（**注意 chunk 内局部坐标**，世界坐标会越界读垃圾）
- `WG_COMPDUMP`：dump 全部 router 组件（barrier/fluid/vein 等）格式对齐 DensityProbe comps
- DensityProbe comps 扩展：barrierNoise/fluidLevelFloodednessNoise/fluidLevelSpreadNoise/lavaNoise/veinToggle/veinRidged/veinGap/initialDensity（yarn 方法名带 Noise 后缀！）
- got_export -nbDump 0=overworld；c2me MixinNoiseChunkGenerator.populateNoise 是权威遍历（cellX/cellZ/cbx/cbz 正向、cellY/vb 反向、blockX 世界坐标）
- 参照文件状态：-288 FULL（含 ORE/结构）、3200 NOISE（无 ORE）——正坐标回归需换 FULL 参照（20000/134304 已导出）

### 2026-08-08 凌晨追加：vein 调查（InterpolatedDF 整树插值确认正确）

**结论**：
- C++ InterpolatedDF **整树插值是正确的**（回滚后 chunk(-18,-16) 100%、3200 100% 恢复）。「噪声插值 + 非线性后置」改造（interpTransform/CellInterpRef）已回滚（git checkout density.h density_builder.h）——**勿再尝试**。
- veinToggle/veinRidged/veinGap 的 C++ InterpolatedDF 插值与 Java 游戏实际**逐点一致**（OreProbe vtI/vrI/vgI 对比，veinToggle 全 0 差异；veinRidged 差异是 OreProbe 自身 bug——unwrap 只解一层、lerp3Interp 对 add/max/abs 整树插值，vanilla 是 2 个 interpolated 独立插值——**C++ 正确**）。
- **vein 产物（granite/diorite/tuff）缺失的真正机制未定**：最可能是 **aquifer 与 vein 的交互顺序/决策**。

**Java 1.20.1 反编译确认（javap minecraft-unpicked.jar）**：
- `OreVeinSampler.create(...)` 返回 BlockStateSampler；`method_40547` = 逐块 `split(blockX,blockY,blockZ)` + veinToggle/veinRidged/veinGap 采样——**与 C++ ore_vein.h 逐行一致**（无 3×3 区域，常量全同）
- `ChunkNoiseSampler.sampleBlockState()` = `blockStateSampler.sample(cns)`（纯 vein）
- `NoiseChunkGenerator.getBlockState(...)` = 恒等（直接返回传入 state）——**aquifer 不在 populateNoise 里显式调用**
- **aquifer 应用位置未定**（可能在 ChunkNoiseSampler 构造的 blockStateSampler 组合里）——下一步：javap 查 ChunkNoiseSampler 构造函数的 blockStateSampler 初始化（putfield #blockStateSampler），确认 aquifer 与 vein 的组合顺序

**OreProbe 已参数化**（-PoreChunkX/-PoreChunkZ，build.gradle 已加传递）；dump 列 (chunkX*16+8, chunkZ*16+8) 的 vt/vr/vg raw+插值（lerp3Interp 复刻 DensityInterpolator，**对单个 interpolated 节点可信，对整树不可信**）。

**待验证实验**：C++ fillOneChunk 把顺序改成「vein 先、aquifer 后」（Java 疑似顺序），看 vein 产物是否恢复。

### 2026-08-08 凌晨追加 2：vein 顺序实验无影响 + 剩余疑点收敛

**顺序交换实验**（vein 先、aquifer 后）→ -288 95.4728%、3200 100% **逐位不变**——顺序不是 vein 缺失根因（两个顺序等价）。已改回原顺序（git 状态 worldgen_api.cpp 有顺序改动——**已手动还原**：`git checkout versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp` 需确认；或保留 vein 先顺序亦可，结果相同）。

**vein 决策链剩余疑点（唯一候选）**：
- **veinRidged 的符号**：ore_vein.h `if (veinRidged->sample(pos) >= 0.0) return -1;`——C++ veinRidged（2 个 interpolated 节点独立插值 + abs/max/add）vs **Java 游戏实际**（ChunkNoiseSampler 的 vein_ridged 插值器）——**没有可信的 Java 参照**（OreProbe 的 lerp3Interp 对整树插值是自身 bug，不可信）。
- 验证方法：cns 反射遍历 8 个 interpolators，找 vein_ridged 相关（min/max 特征：add(-0.08, max(abs,abs)) 的 min/max ≈ [-0.08-|vrmax|, |vrmax|]）；或 Java 侧新探针直接采样 ChunkNoiseSampler 的 vein_ridged 插值状态。

**已确认（勿重查）**：
- hashXYZ（split(int,int,int)）：aquifer 100% 实证负坐标一致
- veinToggle 插值：C++ == Java（OreProbe vtI 全 0 差异）
- OreVeinSampler 算法：与 Java method_40547 逐行一致（javap 确认）
- aquifer 与 vein 顺序：无影响
- InterpolatedDF 整树插值：正确（勿改）

**C++ 侧当前改动（未提交）**：worldgen_api.cpp 的 WG_DBDEBUG/WG_COMPDUMP 诊断 + vein 顺序改动；java 侧 DensityProbe（comps 扩展+cns 修复）、OreProbe（参数化）、BlockProbe 参数。git 提交建议：诊断工具独立提交（中文信息、author unknowbug）。

### 2026-08-08 追加 3：块状根因缩小（level-seed 坑 + 浅层插值符号翻转）

**重要坑（已踩）**：`java/run/server.properties` 的 `level-seed` **硬编码 -8248318472910187742**！`-PbenchSeed=X` 只设 `-Dbench.seed` 属性，**world 实际 seed 永远 -8248**——所有「8576 参照」（DensityProbe/BlockProbe）实际是 -8248 世界的（错位对比）。**跑其他 seed 前必须改 level-seed**（已验证改后 worldSeed 正确）。

**真实 8576（level-seed 修正后）**：玩家 (731,82,-404) 区域 SURFACE 参照对比 **3.31% diff**（错位假象是 9.59%）——**块状真实存在**（chunk(45,-26) 2.84%）。
- diff 全在**地表带 y42-65**（-8248 20000 也是 y42-65 峰值 y50-51——同模式）
- 配对：stone↔water、air↔stone、dirt/grass 错位——**aquifer/surface 浅层错**

**最深疑点（下一步）**：20000 列 (2,11) 剖面：vanilla y52-55 dirt + y56-62 water（海底）；C++ y52-58 stone + y59-61 dirt + y62 grass_block（陆地）——**C++ 插值密度在该列 y52-62 为正而游戏实际为负**——**C++ InterpolatedDF 插值 ≠ 游戏实际（20000 区域，-288 一致）**。候选：
1. vanilla cell 角点网格布局（cacheAllInCell）在特定坐标与 C++ buildGrids 差
2. 需要「游戏实际插值密度」参照——cns 反射的 interpolators 不含 finalDensity（get(0) min=-∞ 是某组件）；需找 ChunkNoiseSampler 的「当前密度」字段（非 interpolators）

**-8248 有效参照**：-288 FULL（19:39，含 FEATURE 假 diff——**只用于 density/vein 分析，方块 diff 不可信**）；20000 SURFACE（23:03，真 diff 0.59%）；3200 SURFACE（14:11，100% 基线）
