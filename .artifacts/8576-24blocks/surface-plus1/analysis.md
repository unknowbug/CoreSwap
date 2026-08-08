# 8576 剩余 24 mismatch — 地表三连错位类（#4-6,10-11,18-20 及顺带 #12）根因分析

> anchor.worker 精确分析 subagent 产物
> seed=8576294172403134396，区域 720,-432 6×6（chunk 45..50 × -27..-22）
> 范围：地表三连错位类。主攻 #4/5/6 (743,-406)、#18/19/20 (800,-363)、#10/11 (754,-403)；顺带 #12 (771,-410) 水边界。
> 状态：**draft**（未实测 density 值，结论为强推理链；详见 §5 置信度）
> 不修改任何代码/参照文件。

---

## 1. 摘要

**根因：不在 surface 规则层（est / runDepth / buildSurface 主循环全部与 Java 对齐），而在 buildSurface 之前的初始地形（NOISE/aquifer 阶段）。**

C++ 在 `(743,72,-406)`、`(800,77,-363)`、`(754,61,-403)` 的 buildSurface 前初始地形比 vanilla 高 **1 格**：C++ finalDensity 在这些列判 `stone`，Java 判 `air`（754 列为判 `stone` 而参照为 `water`）。由于 buildSurface 从「最高非空气块」向下铺层（grass → dirt×sd → stone），初始地形高 1 导致整列 stone→dirt→grass 三连段同步 +1。

- **真 bug**（非 FEATURE 假 diff）：参照列 grass 顶上方均为纯 air、无树/湖/草方块覆盖迹象；est/runDepth 实测与 Java 一致，三列 z 全为负（-406/-363/-403），与 `docs/10-timewise-archive.md` 记录的负坐标 base_3d_noise 偏正未解项方向吻合。
- 修复方向：对 `(743,72,-406)` 等点用 `WG_SURFDUMP`（C++ finalDensity）vs Java cns（DensityProbe）实测 finalDensity 符号，定位负坐标密度偏差源（候选 `base_3d_noise` 负坐标 octave / InterpolatedDF 负坐标插值）。

---

## 2. 对拍表（Java 权威源码 vs C++ 实现）

### 2.1 estimateSurfaceHeight（est 4 角 + 插值）

| 环节 | Java | C++ | 结论 |
|---|---|---|---|
| 单列 est 扫描 | ChunkNoiseSampler.java:222-240：`(x>>2)<<2` 4 格对齐，从顶 `minY+height` 向下步长 8，`initialDensityWithoutJaggedness > 0.390625` 返回首个 l | aquifer.h:142-161：`(blockX>>2)<<2`、步长 8、`initialDensity->sample(pos)>0.390625` | 一致 |
| 4 角坐标 | MaterialRules.java:491-500：`blockToChunkCoord(blockX)<<4` = `chunkX*16` | worldgen_api.cpp:747-750：`estimateSurfaceHeight(chunkX*16, chunkZ*16)` 等 | 一致 |
| lerp2 插值 | MaterialRules.java:502-511：`MathHelper.lerp2(fx,fz,e00,e10,e01,e11)`，fx=(x&15)/16 | surface.h:268-273：同参数序展开 | 一致 |
| floor | `MathHelper.floor(lerp2(...))` | `(int)std::floor(...)` | 一致 |
| 阈值 | `surfaceMinY = k + runDepth - 8`（MaterialRules.java:512） | surface.h:277：`blockY >= k + surfaceDepth - 8` | 一致 |

**实测**（历史 WG_ESTDUMP，20260807 会话）：`chunk(46,-26) sh4=64 64 56 40`。
(743,-406) 的 k = `floor(lerp2(0.4375, 0.625, 64,64,56,40))` = `floor(54.625)` = **54**。
→ above_preliminary_surface 阈值 = 54 + 3 - 8 = **49**（sd=3，见 2.2）——远低于 grass 顶 71/72。

**结论：est 不是 +1 来源**（若 est 差 1，阈值也只影响 49/50 附近的分层，不可能把 grass 顶从 71 抬到 72）。

### 2.2 runDepth / sampleRunDepth（surfaceDepth）

| 环节 | Java | C++ | 结论 |
|---|---|---|---|
| 列初始值 | MaterialRules.java:459 `initHorizontalContext`: `runDepth = sampleRunDepth(blockX,blockZ)` | surface.h:718 `ctx.surfaceDepth = sampleRunDepth(m,n)` | 一致 |
| 公式 | SurfaceBuilder.java:172-175：`(int)(surface*2.75 + 3.0 + split(x,0,z).nextDouble()*0.25)` | surface.h:377-389：同公式 | 一致（docs/10 已逐位验证 (804,-368)=4） |
| 分层语义 | dirt 层数 = `stoneDepthAbove <= 1 + surfaceDepth`（STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH，mr7→dirt） | surface.h:228-234 StoneDepthCond + 规则树 mc10 段 | 一致 |

**实测**：三列 dirt 层数两侧一致——
- 743：参照 dirt 68-70（3 层）vs C++ dirt 69-71（3 层）
- 800：参照 dirt 72-75（4 层）vs C++ dirt 73-76（4 层）
- 754：参照 dirt 56-60（5 层）vs C++ dirt 57-61（5 层）

**结论：runDepth/surfaceDepth 不是 +1 来源**（两侧 sd 完全相同，故 dirt 层厚度相同，整段只是整体平移 1）。

### 2.3 buildSurface 主循环

| 环节 | Java | C++ | 结论 |
|---|---|---|---|
| 起点 | SurfaceBuilder.java:117/124：`p = sampleHeightmap(WORLD_SURFACE_WG)+1` | surface.h:707/717：`p = columnH + 1`（heightmap 最高非空气） | 一致 |
| 列状态 | q=0 空气归零；r=最高流体 y+1；s=下方首个非默认块 | surface.h:735-758 同语义 | 一致 |
| 顶层 | q=1 的 default 块经 mr9→mr8→mr→water(0,0)→grass | 同 | 一致 |
| dirt 层 | q=2..1+sd 经 mc10→stoneDepth(0,true,0,false)→mr7→dirt | 同 | 一致 |
| 深层 | q>1+sd 不替换 → 保持 stone | 同 | 一致 |

**结论：buildSurface 主循环与分层判定全部对齐。** grass 顶 = buildSurface 前最高非空气块；差异只能来自 buildSurface 前初始地形。

---

## 3. 三列形态与初始地形推导

> 参照 = 参照 blocks 最终状态（buildSurface 后）；「buildSurface 前」由列形态反推（surface 只把顶层 stone 换 dirt/grass/water，不改 air↔stone 边界）。

| 列 | 参照（buildSurface 后） | C++（buildSurface 后） | 参照初始地形 | C++ 初始地形 | 差 |
|---|---|---|---|---|---|
| (743,-406) | stone≤67 / dirt 68-70 / grass 71 / air≥72 | stone≤68 / dirt 69-71 / grass 72 / air≥73 | stone 顶 71（72 起 air） | stone 顶 72（73 起 air） | **C++ +1** |
| (800,-363) | stone≤71 / dirt 72-75 / grass 76 / air≥77 | stone≤72 / dirt 73-76 / grass 77 / air≥78 | stone 顶 76（77 起 air） | stone 顶 77（78 起 air） | **C++ +1** |
| (754,-403) | water 50-53 / stone 54-55 / dirt 56-60 / water 61-62 / air≥63 | water 50-52 / stone 54-56 / dirt 57-61 / water 62 / air≥63 | stone 顶 60、水面顶 62 | stone 顶 61、水面顶 62 | **C++ 水底 +1（水面一致）** |

关键点（754 列）：**水面顶两侧一致（62）**，只有 stone/水底边界 C++ 高 1——说明 aquifer 液面准确，差在 density（stone↔air）边界本身。三列均是「C++ 在 y 判 stone、Java 判 air」的单格边界翻转。

`#12 (771,-410)`（顺带）：参照 stone 35-40 / water 41+；C++ stone 38-41 / water 42+——C++ 地表/水面整体 +1，同机制（水边界类另一列，与 #2/#3 等深板岩/水类形态相关，可并入后续 aquifer 液面立项）。

---

## 4. 根因结论

**真 bug（density 层），非 surface 规则层，非 FEATURE 假 diff。**

- **不是** est/runDepth/buildSurface 的 bug：§2 三表 + §3 实测全对齐。
- **不是** FEATURE 假 diff：参照 grass 顶上方均为纯 air（743:72+ air、800:77+ air、754:63+ air），无树（无 log/leaves）、无湖泊（无 water 替换）、无草方块覆盖；若有 FEATURE 抬高/改变地形，方向应让参照 ≠ C++，但这里参照只低 1 且是纯自然 air 边界。
- **是** C++ 在 buildSurface 前（NOISE/aquifer 阶段）于 `(743,72,-406)`、`(800,77,-363)`、`(754,61,-403)` 判 `stone`，Java 判 `air`（754 判 water）——finalDensity 符号在临界列翻转。
- **三列 z 均为负**（-406/-363/-403）：与 `docs/10-timewise-archive.md` 记录过的「负坐标 base_3d_noise 偏正（@-288 差 0.05-0.23，未定位，后被结构假 diff 掩盖）」方向一致——**主候选 = C++ 密度引擎负坐标（z<0）微小正偏差**，在 finalDensity 接近 0 的临界列导致判 stone。

### 具体定位（待实测确认）

无法在本次 subagent 环境运行 block_probe（无 shell；read_only_task 确认 bash 禁用），故**未直接实测 finalDensity 值**。定位为强推理：
- C++ 文件：`versions/1.20.1/cpp/worldgen/src/density.h`（base_3d_noise / InterpolatedDF 负坐标路径）或 `density_builder.h`（buildNode 采样）
- 修复方向（下一轮 worker）：
  1. `$env:WG_SURFDUMP=1; WG_SURFDUMP_X=743; WG_SURFDUMP_Z=-406` 跑 block_probe → 看 (743,72,-406) C++ finalDensity 符号
  2. Java 侧 DensityProbe cns 链实测 `finalDensity(743,72,-406)` 符号（参照 air 应 < 0）
  3. 若 C++ 偏正：对比 base_3d_noise 各 octave / InterpolatedDF 插值在负坐标的值（`-288` 篇工具 WG_B3DDUMP/GRID 复用），定位符号差源头
  4. 修复后回归 8576（应 99.9993% → 100% 或接近）+ 3200 干净参照零退化

---

## 5. 置信度与局限

- **置信度：draft**（AI 不写 confirmed）。
- 结论强度：est/runDepth/buildSurface 对拍与列形态为**实测证据**（历史 block_probe/read_col2 + 源码），「+1 在初始地形」为**强推理**（buildSurface 逻辑 + 列形态反推）；「负坐标 density 偏差」为**主候选假设**（未实测 finalDensity 符号）。
- 局限：subagent 无 shell，未能现场跑 WG_SURFDUMP/WG_DENDUMP 拿 (743,72,-406) 等点的 finalDensity 实值；下一轮 worker 需补测后把本条目从 draft 升 candidate。

---

## 6. 产物引用

- 本文件：`.artifacts/8576-24blocks/surface-plus1/analysis.md`
- 明细/剖面：`.investigations/8576-24blocks/mismatch-list.md`、`.investigations/8576-24blocks/column-profiles.md`
- 历史实测：`E:/tmp/bp8576_run.txt`（block_probe 24 mismatch）、`E:/tmp/cpp_fd_810_-411.txt`（finalDensity 剖面样例）
- 源码：`MaterialRules.java`（488-516, 459, 567-572）、`ChunkNoiseSampler.java`（222-240）、`SurfaceBuilder.java`（113-175）、C++ `surface.h`（177-194, 261-278, 377-389, 701-780）、`aquifer.h`（139-161）、`worldgen_api.cpp`（746-760）
- 先验：`versions/1.20.1/docs/06-surface-rules.md`、`docs/04-aquifer.md`、`docs/10-timewise-archive.md`
