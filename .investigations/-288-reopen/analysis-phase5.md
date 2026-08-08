# Phase 5：C++/Java aquifer 判定冲突最终交叉判定（(-244,58,-256)）

> 角色：recode.scout（隔离子进程，只读勘探）
> 数据源：`.investigations/-288-reopen/cmd-output/`（aqfj_blockprobe.txt / trace_aqf_1.txt / dump_x-244_z-256.txt / vanilla_density_*_cns.txt）+ 源码（C++ `aquifer.h`/`density.h`/`worldgen_api.cpp`/`xoroshiro.h`；Java `AquiferSampler.java`/`ChunkNoiseSampler.java`/`DensityFunctionTypes.java`/`NoiseChunkGenerator.java`/`Xoroshiro128PlusPlusRandom.java`/`RandomSeed.java`）+ phase2/3/4 分析 + 会话历史（AQF-J 实现与知识库铁律）
> 参照 ground truth：`vanilla_-8248318472910187742_4_-288_-256.blocks`（Java 实际生成，ref_col_-244_-256.txt）
> 产出：本文档（唯一可写产物）
> 置信度标注：源码+数据闭合=**【确定】**；需执行/新数据=**【推测】**

---

## 0. 执行摘要（先给结论）

1. **C++ densityBuf 基准已确认**：`densityBuf[58] = h->finalDensity->sample(-244,58,-256)`（worldgen_api.cpp:619），与 `[SURF] finalDensity` 同基准（InterpolatedDF 块级插值链）；AQF 的 density 输入即 densityBuf 值（trace y=56/60 与 [SURF] 逐位一致：-0.053461/-0.095322）。**任务背景的疑问 a（AQF density ≠ [SURF] finalDensity）系 y 坐标错位（52 vs 58），实际同基准。**
2. **Java 判定可信度：AQF-J 的 `blockStateSampler.sample(cns) -> null` 不可信**——CellCache 反射污染（铁律 L750 复现）：AQF-J 反射设置 cellBlockX/Y/Z 后调用 sample，但 CellCache.cache 数组只在游戏主循环 `onSampledCellCorners`（ChunkNoiseSampler.java:349-351）填充；反射调用不触发重填，返回的是「最后遍历 8-cell」的错误缓存值。densFn 3 次运行值不同（0.037219/0.039371/0.048147）是污染的直接证据。**null 判定不能作为「Java aquifer 判 solid」的证据。**
3. **同基准密度对比（核心新证据）**：Java 噪声阶段 density(-244,58,-256) = **-0.0744 < 0**，与 C++ 逐位一致——由 cns idx0(58) = -0.233015（cns 反射文件）+ `squeeze(0.64×idx0)` 验算 = -0.074427 ≈ C++ -0.074424（差 ≤3e-6，phase3 同法）。Java aquifer 的 density 输入 = `CellCache(add(finalDensity, Beardifier=0)).sample` = finalDensity 块级值（ChunkNoiseSampler.java:177-181 + Beardifier 恒 0 确认）。**两测 density 输入同基准同符号（负）。**
4. **判定差定位**：C++ 判 water（density=-0.0744<0，e=0）与「Java 噪声阶段 density<0 → aquifer 判 water」**逻辑一致**。参照 y=58=stone（ground truth）与 density<0 的矛盾只有两条出路——(A) **Java e 值翻转**（density+e>0，需 Java 的 o/p/q 或 fl2/fl3 液面与 C++ 不同）；(B) **结构覆盖**（ocean_ruin 在 FEATURES 阶段放置 stone/dirt 覆盖 water）。**本次分析无法从现有数据 100% 闭合 A/B**，但提供了决定性判别方法（§5）。
5. **修复方向**：不修代码。补 Java 侧 (-244,55..62,-256) 的 aquifer 中间量（o/p/q/d/fl.y/e）一次对照即可收口；若 o/p/q 与 C++ 一致（splitter 派生链已在源码层面闭合），则 fl.y 差 → C++ 液面链 bug，fl.y 一致 → 结构覆盖 → 维持「结构假 diff」结案。

---

## 1. C++ densityBuf 基准确认（任务疑问 a）

### 1.1 densityBuf 填充来源【确定】
worldgen_api.cpp:610-622：
```cpp
std::vector<double> densityBuf((size_t)h->dim.worldHeight * 256);
for (int by = 0; by < h->dim.noiseHeight; by++) {
    int wy = h->dim.minY + by;
    for (int bz = 0; bz < 16; bz++)
        for (int bx = 0; bx < 16; bx++) {
            fpos.x = chunkX * 16 + bx; fpos.y = wy; fpos.z = chunkZ * 16 + bz;
            densityBuf[by * 256 + bz * 16 + bx] = h->finalDensity->sample(fpos);   // ← L619
        }
}
```
apply 调用（worldgen_api.cpp:712）：
```cpp
block = aquifer->apply(chunkX * 16 + bx, wy, chunkZ * 16 + bz, densityBuf[by * 256 + bz * 16 + bx]);
```

**→ C++ AQF 的 density 输入 = densityBuf = `finalDensity->sample(块)`（InterpolatedDF 块级插值），与 `[SURF] finalDensity`（dump 打印 `h->finalDensity->sample`，worldgen_api.cpp:673-674）完全同基准。**

### 1.2 AQF density 与 [SURF] finalDensity 的关系【确定】
trace_aqf_1.txt 与 dump_x-244_z-256.txt 交叉验证：

| y | [SURF] finalDensity（dump L1186-1188） | [AQF] density（trace L5-12） | 差 |
|---|---|---|---|
| 56 | -0.053461 | -0.053461 | 0 |
| 58 | （dump 步长 4 无此层） | -0.074424 | — |
| 60 | -0.095322 | -0.095322 | 0 |
| 62 | （dump 步长 4 无此层） | -0.116134 | — |

**→ AQF density 与 [SURF] finalDensity 同基准逐位一致（y=56/60 直接相等）。任务背景的「AQF density(-0.0744) ≠ [SURF] finalDensity(-0.0139)」是 y 坐标错位——-0.0139 是 y=52 的值（dump L1185），不是 y=58。y=58 的 [SURF] finalDensity 未打印（步长 4），其块级值即 -0.074424。**

### 1.3 C++ 判定链（y=58）【确定】
aquifer.h:70-137 apply：
- `density=-0.074424 ≤ 0` → 不走 `density>0` 分支
- fluidLevelSampler（L74-78）：y>-54 → 水液面 63
- 13 邻居扫描（L81-101）：o=90, p=99, q=115（平方距离）
- `d = maxDistance(o,p) = 1-|99-90|/25 = 0.64 > 0` → 进入 e 计算（L118）
- `calculateDensity`：`j=|fl2.y-fl3.y|=0`（相邻网格点液面相同 63）→ 返回 0（L241-243）→ **e=0**
- `density+e = -0.074424 < 0` → 不触发 `density+e>0` 的 null → 返回 bs=water(32) → **FLUID**

C++ 55..62 全判 water 的路径：y=55-57/59-62 因 `d≤0` 直接 `return bs`（L110，trace 无 [AQF-e] 行印证）；仅 y=58 走 e 且 e=0。

---

## 2. Java 判定可信度：AQF-J null 判定被 CellCache 反射污染（任务疑问 b 前半）

### 2.1 AQF-J 的调用机制【确定】（会话历史 message 186 源码片段）
BlockProbe.java 通过反射设置 cns 的 `startBlockX/Y/Z` + `cellBlockX/Y/Z` + `isInInterpolationLoop=true`，然后调用 `blockStateSampler.sample(cns)`（ChainedBlockSource → aquifer.apply(pos, densityFunction.sample(pos))）。

### 2.2 为什么反射调用返回垃圾值【确定】
`densityFunction` = `CellCache(add(finalDensity, Beardifier))`（ChunkNoiseSampler.java:177-181）。CellCache.sample 走缓存路径（L665-685）：
```java
int i = cellBlockX, j = cellBlockY, k = cellBlockZ;
return i>=0 && j>=0 && k>=0 && i<4 && j<8 && k<4
    ? this.cache[((8-1-j)*4+i)*4+k]     // ← cache 数组
    : this.delegate.sample(pos);
```
而 **cache 数组只在游戏主循环 `onSampledCellCorners`（L342-355）里由 `cellCache.delegate.fill(cellCache.cache, this)` 填充**（对「当前 8-cell」的 4×8×4=128 个子位置采样）。反射调用**不会触发重填**——返回的是**最后一次遍历的 8-cell** 的缓存值，与目标位置 (-244,58,-256) 无关。

### 2.3 污染证据【确定】
- densFn(-244,58,-256) 3 次运行值不同：0.037219 / 0.039371 / 0.048147（aqfj_blockprobe.txt L19/40）；y=52 三次也不同（0.036956/0.041260/0.058800）→ 同一位置 3 个值，cache 状态每运行不同。
- 知识库铁律 L750（历史 9 篇时间线）：「blockStateSampler.sample / CellCache.sample 在非真实遍历状态返回缓存垃圾值（如固定 -0.024995）——勿以反射作密度参照；必须用 DensityProbe 的完整 cns 链（sampleStartDensity→interpolateY/X/Z）在真实遍历内取值」。
- `null` 3 次「稳定」不构成可信证据：3 次运行 chunk 生成流程相同 → 最后遍历的 8-cell 相同 → cache 值相同 → null 稳定，但该 null 对应的是**错误 cell 的密度**（若错误 cell 密度 >0 → aquifer apply 的 `density>0` 分支 → null）。

**→ AQF-J 的 `-> null` 不能作为「Java aquifer 在 (-244,58,-256) 判 solid」的证据。**

---

## 3. 同基准密度对比（核心新证据，任务疑问 b 后半）

### 3.1 Java aquifer density 输入链【确定】
ChunkNoiseSampler.java:176-181：
```java
Builder<BlockStateSampler> builder = ImmutableList.builder();
DensityFunction densityFunction = DensityFunctionTypes.cacheAllInCell(
        DensityFunctionTypes.add(noiseRouter2.finalDensity(), DensityFunctionTypes.Beardifier.INSTANCE)
    )
    .apply(this::getActualDensityFunction);
builder.add(pos -> this.aquiferSampler.apply(pos, densityFunction.sample(pos)));
```
- `Beardifier.INSTANCE` 恒返回 0.0（DensityFunctionTypes.java:294-296）；NoiseChunkGenerator.java:205 传入的就是 INSTANCE → **Beardifier 贡献 = 0**。
- CellCache 缓存值 = `add(finalDensity, Beardifier)` 在「当前 8-cell 内 4×8×4 子位置」的 8-cell 三线性插值 = **finalDensity 块级值**（cns.fill L313-328 + DensityInterpolator isSamplingForCaches 分支 L792-806 语义闭合）。
- **→ Java aquifer 的 density 输入 = finalDensity 块级值 = C++ densityBuf 值（同基准同值）。**

### 3.2 y=58 的 Java 块级密度验算【确定】
phase3 未直接测 y=58（dump 步长 4），本次用 cns 反射文件闭合：
- `vanilla_density_overworld_c-16_-16_b12_0_cns.txt` L2089：**idx0(58) = -0.233015**（cns 8 个 interpolators 第 0 个 = finalDensity 树 interpolated 部分，phase3 §3.1 已确认 idx0 语义）
- `squeeze(0.64 × idx0(58))`：0.64×(-0.233015) = -0.149130；squeeze(-0.149130) = -0.149130/2 − (−0.149130)³/24 ≈ -0.074565 + 0.000138 = **-0.074427**
- C++ `finalDensity(58) = -0.074424`（trace L7）→ 差 **3e-6**（打印位内一致，phase3 同法误差 ≤4e-6）

**→ Java 噪声阶段 density(-244,58,-256) = -0.0744 < 0，与 C++ 逐位一致。两测同基准、同符号（负）。**

### 3.3 符号确认【确定】
| 基准 | density(58) | aquifer 逻辑结果 |
|---|---|---|
| C++ densityBuf / [SURF] | -0.074424（负） | density≤0 → 流体路径 → water |
| Java cns 插值链（squeeze(0.64×idx0)） | -0.074427（负） | 同输入 → 应判流体（除非 e 翻转） |
| AQF-J 反射（污染） | 0.037219/0.039371/0.048147（正） | 不可信（污染垃圾值） |

---

## 4. aquifer 判定差定位：四个候选环节逐项裁决

| 环节 | 结论 | 证据 |
|---|---|---|
| density 输入差 | **【确定】无差**（同基准同值 -0.0744） | §3.2 验算 ≤3e-6 |
| Beardifier | **【确定】无贡献**（恒 0） | DensityFunctionTypes.java:294 + NoiseChunkGenerator.java:205 |
| e 值（calculateDensity / fl2/fl3 液面） | **【推测】唯一可能的判定差**：C++ e≡0（j=|fl2.y-fl3.y|=0）；若 Java fl2.y≠fl3.y（如 63 vs -32512 无效液面），j≠0 → calculateDensity 非零 → e 非零 → `density+e>0` → null（stone） | C++ e=0（trace L8）；Java 侧 fl2/fl3 无数据 |
| o/p/q（getBlockPos split 派生） | **【推测·源码闭合】**：splitter 派生链已确认一致（xoroshiro.h:75-79 vs Xoroshiro128PlusPlusRandom.java:130-134 同为无状态 `(hashXYZ^seedLo, seedHi)`；hashXYZ L13-23 vs MathHelper.hashCode 补码语义一致；md5 均为 RFC1321 标准 → RandomSeed.java:33-38 vs md5.h）；**若 md5 实现无误，o/p/q 必与 C++ 一致（90/99/115）**，则 d=0.64 一致，e 链唯一剩余变量是液面 | splitter 闭合；md5 实现正确性需执行验证（唯一未闭合点） |
| 液面网格（getFluidLevel / estimateSurfaceHeight） | **【推测】e 翻转的首选嫌疑**：C++ getFluidLevel（aquifer.h:287-363）与 Java（AquiferSampler.java:353-450）逐行一致；estimateSurfaceHeight(-244,-256)=32 两测一致（aqfj L5-6）；**但 13 邻居跨 ±48 格（5×5 chunk）的液面输入链无 Java 对照数据** | C++/Java 源码一致；est 单点一致；邻居液面未对照 |

### 4.1 核心矛盾（必须诚实呈现）【确定 + 推测】
- 【确定】Java 噪声阶段 density(58) = -0.0744 < 0 → 若 aquifer 判定链与 C++ 完全一致，Java 应判 water。
- 【确定】参照（游戏实际导出，ground truth）y=58 = stone。
- 【推测】两条出路：
  - **(A) Java e 值翻转**：Java 的 fl2/fl3（或 fl2/fl4、fl3/fl4）液面不同 → calculateDensity≠0 → e 或 g/h 使 `density+e>0` → null（stone）。这需要 Java 的液面网格输入（13 邻居 estimateSurfaceHeight/floodedness）与 C++ 有差异。
  - **(B) 结构覆盖**：参照 y=58 stone 是 FEATURES 阶段 ocean_ruin 结构放置（覆盖噪声阶段 water）。phase4 §2.2 以「ChunkStatus 顺序 + y=50=stone（q=5 → SURFACE 阶段 y=58-61 solid）」排除结构假设；**但该论证依赖「y=50=stone 是 SURFACE 阶段未染色」**，若 y=50 的 stone 也是结构覆盖（ocean_ruin 覆盖 SURFACE 阶段染的 gravel），排除不成立。ref_col y=40-50 全 stone（无海底 surface 染色的 gravel/sand）与「正常海底有 surface 染色」不符，**对 (B) 有一定支持**。

### 4.2 AQF-J 对两个假设的判别力【确定】
AQF-J 的 null 不能判别 (A)/(B)——它连「Java 实际判什么」都没测到（§2）。**任何基于 AQF-J null 的「Java 判 solid」结论（含主会话 message 201 的断言）均不可采信。**

---

## 5. 修复方向建议（交主会话裁决）

### 5.1 决定性对照实验（一次收口，最高优先）
在 Java 侧补打 (-244,55..62,-256) 的 aquifer apply 中间量：**o/p/q（最近邻/次近邻距离）、d=maxDistance(o,p)、fl2.y/fl3.y（getWaterLevelAt(r/s) 液面）、e、density+e**（对应 C++ trace 的 [AQF]/[AQF-e] 行，一次 run 即可逐项 diff）。

判定树：
1. **o/p/q 与 C++ 一致（90/99/115@y=58）且 fl.y 一致（63/63）** → Java 的 e=0 同 C++ → Java 也应判 water → **参照 y=58 stone 必为结构覆盖 (B)** → 维持「结构假 diff」结案（C++ 无 aquifer bug），并记录为结构样本（建议用结构探测确认 ocean_ruin 存在）。
2. **o/p/q 一致但 fl.y 不同（如 63 vs -32512）** → j≠0 → Java e≠0 → density+e 翻转 → **(A) e 翻转成立** → C++ 液面链 bug：定位 `getFluidLevel/getFluidBlockY`（aquifer.h:286-363）在 13 邻居上的液面计算（首选 suspect：estimateSurfaceHeight 缓存边界 CACHE_DIM=32/CACHE_OFF 或 fluidFloodedness/fluidSpread 采样位置）。
3. **o/p/q 不同** → getBlockPos splitter 派生差 → 验证 C++ md5("minecraft:aquifer") 与 Java MD5 是否一致（md5.h vs RandomSeed.java:33-38；或补 C++ blockPositions dump 对照）。

### 5.2 低优先/辅助
- 结构探测：确认参照列 (-244,-256) 及 x∈[-244,-241], z∈[-256,-250] 是否落在 ocean_ruin 结构范围内（MC 结构日志 / structure accessor），直接判别 (B)。
- y=50=stone 的 gravel 差：若 (B) 成立，C++ 染 gravel 是「对非结构状态忠实」的结果，应归结构假 diff，不修。
- `splitter` md5 验证：补 C++ 打印 blockPositions 与 Java 反射 blockPositions 对照（一次性）。

---

## 6. 置信度汇总表

| # | 结论 | 置信度 |
|---|---|---|
| 1 | C++ densityBuf = finalDensity->sample（worldgen_api.cpp:619），与 [SURF] finalDensity 同基准；AQF density 与之逐位一致（y=56/60=0 差） | 【确定】 |
| 2 | 任务背景疑问 a 系 y 坐标错位（52 vs 58），非密度基准差异 | 【确定】 |
| 3 | Java aquifer density 输入 = CellCache(add(finalDensity, Beardifier=0)).sample = finalDensity 块级值 | 【确定】 |
| 4 | Java 噪声阶段 density(-244,58,-256) = squeeze(0.64×idx0(58)) = -0.074427 ≈ C++ -0.074424（≤3e-6） | 【确定】 |
| 5 | AQF-J null 判定不可信（CellCache 反射污染；densFn 3 次不同 + L750 铁律 + cache 不重填机制） | 【确定】 |
| 6 | C++ 判 water 与 Java 噪声阶段 density<0 逻辑一致；矛盾在参照 y=58=stone | 【确定】 |
| 7 | 参照 y=58 stone 的两条出路：(A) Java e 翻转（fl.y 差）／(B) 结构覆盖（ocean_ruin）；现有数据无法闭合 | 【推测】 |
| 8 | splitter 派生链源码闭合（hashXYZ/split 无状态/md5 标准）；md5 实现正确性未执行验证 | 【确定（源码）/推测（md5 实现）】 |
| 9 | phase4 的「q=5 → SURFACE 阶段 solid」排除结构论证依赖「y=50=stone 非结构」，若 y=50 亦为结构覆盖则排除失效；ref_col y=40-50 全 stone（无海底 surface 染色）对结构假设有一定支持 | 【推测】 |

---

## 7. 影响架构的变化（交主会话裁决）

> 显式标注「架构变更建议」，本角色不实施。

1. **AQF-J 探针口径修正（高优先）**：`blockStateSampler.sample(cns)` 反射不可用作 aquifer 判定证据（L750 铁律已有，本次为再确认）。后续 aquifer 判定对照必须用「游戏真实遍历状态」的中间量 dump（在 onSampledCellCorners/interpolate 循环内打 o/p/q/d/fl.y/e），或直接用最终 .blocks 参照列判 ground truth。
2. **参照列语义确认（高优先）**：.blocks 为 FULL 状态（含结构/FEATURE，phase2 口径）。凡参照列出现「噪声密度为负但参照是实心」的样本，必须先用结构探测排除结构覆盖，再下 aquifer bug 结论——否则会重蹈「误判 ocean_ruin」的翻案循环。
3. **无 C++ 代码修改建议**：本次分析未发现可闭合的 C++ 源码级 bug（aquifer.h/surface.h/xoroshiro.h 与 Java 对照一致）；唯一未闭合点是 md5 实现与 13 邻居液面输入链，需 §5.1 数据。

---

## 附：待深入点清单

| # | 项目 | 状态 | 置信度 | 建议 |
|---|---|---|---|---|
| 1 | Java (-244,55..62,-256) aquifer 中间量 dump（o/p/q/d/fl.y/e） | 未跑 | - | §5.1 决定性实验 |
| 2 | C++ md5("minecraft:aquifer") vs Java MD5 一致性 | 未验证 | 推测 | 补 blockPositions 对照 |
| 3 | 参照列 (-244,-256) 是否 ocean_ruin 结构范围 | 未探测 | 推测 | 结构 accessor 探测 |
| 4 | phase4 预测 rd=sampleRunDepth(-244,-256)==2 | 未验证 | 推测 | 与 §5.1 一并验证 |
| 5 | AQF-J 探针口径（cellBlockX/Y/Z 反射） | 已判不可信 | 确定 | 归档为不可用路径 |
