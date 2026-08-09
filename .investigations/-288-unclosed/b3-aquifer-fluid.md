# B3 假设判定：Java aquifer 液面/e 值修正项未复刻（海底边界 6710 块）

> 角色：core.worker（分析，沙箱无 shell，纯落盘证据）
> 课题：seed=-8248318472910187742 的 -288,-256 区域（4×4 chunk，block_probe 匹配率 95.7376%）未闭合差异
> 目标：判定 B3「Java aquifer 液面（fluidLevel/estimateSurfaceHeight）计算存在 C++ 未复刻的修正项 → calculateDensity e 非零 → 海底面判定差」
> 数据源：`AquiferSampler.java` / `ChunkNoiseSampler.java` / `VanillaBiomeParameters.java`（Java 1.20.1 源码）；`aquifer.h` / `surface.h` / `density.h` / `xoroshiro.h` / `worldgen_api.cpp`（C++）；`.investigations/-288-reopen/cmd-output/`（trace_aqf_1.txt、dump_x-244_z-256.txt、aqfj_blockprobe.txt、vanilla_density_*_cns.txt）+ `ref_col_-244_-256.txt` + `m288_pair_counts.txt` + `analysis-phase2.md` + `analysis-phase5.md`
> 状态：**draft**（AI 只写 draft，提升需审查+主会话裁决）
> retry: 0（部分支持，非失败轮——B3 是 fan-out 三候选之一，首次验证）
> 产物：`.investigations/-288-unclosed/b3-aquifer-fluid.md`

---

## 0. 一页判定

**B3 部分支持（机制成立，具体差异位置未 100% 闭合）。**

- **[确定]** C++ 与 Java 的 aquifer **density 输入同基准同值**（phase5：(-244,58,-256) 均 -0.0744，y=56/60 逐位一致）→ 判定差不在 density。
- **[确定]** 参照列 (-244,-256) vanilla 在 **y=58-61 存在 aquifer 浮岛实心**（y=58 stone + y=59-61 dirt，夹在 y=51-57 水洞与 y=62 海面之间），C++ 该 4 层**全判 water**（trace: y=55-62 全水，y=58 e=0）。
- **[确定]** 系统性统计：`got=water vs vanilla=stone/dirt/sand/sandstone/gravel` = **6710 块**（3117+2539+723+133+198，与任务背景数字逐项吻合），全部集中在 y=52-62 海底面附近；结构/FEATURE 仅占差量 ~3.6%（phase2）→ **非结构假 diff，是 aquifer 判定真差**。
- **[机制]** vanilla 浮岛实心只能由 aquifer 的 `density+e>0`（或 g/h 分支）翻转产生（d≤0 分支只返回 water/air，aquifer 无直接 stone 输出）。C++ e≡0（j=|fl2.y-fl3.y|=0）→ 缺翻转 → 判水。**B3 机制成立：Java 侧 e/g/h 项在海底面附近非零翻转，C++ 未复刻。**
- **[未闭合]** 具体差异位置在「随机点 o/p/q（splitter 派生）」与「液面网格输入（13 邻居 estimateSurfaceHeight/floodedness）」二选一——Java 侧中间量（o/p/q/d/fl.y/e）无可信数据（AQF-J 反射污染，phase5 判不可信）。**修复前需补 Java 侧一次中间量 dump。**

---

## 1. 参照列 ground truth（vanilla 实际生成，FULL 状态）

`ref_col_-244_-256.txt`（提取自 `vanilla_-8248318472910187742_4_-288_-256.blocks`）：

| y | 方块 | 解释 |
|---|---|---|
| 40..50 | stone | 主海床（海底在 y=50 顶） |
| 51..57 | water | **水下洞（aquifer 液面层）** |
| 58 | stone | **洞顶/浮岛底（density+e 翻转点）** |
| 59..61 | dirt | **浮岛体（surface 在 stone 上染 dirt）** |
| 62 | water | 海面 |
| 63.. | air | 大气 |

- y=51-57 中空 + y=58-61 实心 + y=62 水：形态是 **aquifer 浮岛**（1.18+ aquifer 在水中生成的孤立实心），**不可能是 ocean_ruin 实心柱**（结构柱不会中间空；ocean_ruin 方块是 cobblestone/stone/gravel 大堆，且 phase2 统计结构总量仅 ~3.6%）。
- y=58-61 的实心是 vanilla「C++ 判 water、vanilla 判实心」的具体样本（任务背景 6710 块的典型列）。

## 2. C++ 判定链（trace_aqf_1.txt + phase5 §1.3，确定）

`trace_aqf_1.txt` (-244,-256)：

| y | density | nearest (o,p,q) | d=maxDistance(o,p) | bs | e | density+e | C++ 结果 |
|---|---|---|---|---|---|---|---|
| 55 | -0.043591 | 54,101,126 | -0.88 | 32(water) | —(d≤0) | — | water |
| 57 | -0.063950 | 82,107,109 | 0.00 | 32 | —(d≤0) | — | water |
| 58 | -0.074424 | 90,99,115 | 0.64 | 32 | **0.0000** | -0.0744 | **water** |
| 59 | -0.084882 | 75,106,118 | -0.24 | 32 | —(d≤0) | — | water |
| 62 | -0.116134 | 42,91,142 | -0.96 | 32 | —(d≤0) | — | water |

- 仅 y=58 走 e 分支（d>0），`calculateDensity` 因 `j=|fl2.y-fl3.y|=0`（相邻网格点液面相同 63）返回 0 → e=0 → `density+e<0` → water。
- y=55-57/59-62 因 `d≤0` 直接返回最近点液面（63 → water）。SURFTRACE 显示 C++ 该列海床顶在 **y=50**（stone→sand 染色，surface.h L771-775），y=51-62 全水。
- 注意：`d` 是 `1-|p-o|/25`，o/p 是**平方距离**（AquiferSampler.java L258-261 / aquifer.h L232-235 一致）。

## 3. C++ vs Java 液面/e 值逐项对照（带行号）

| # | 环节 | Java 1.20.1 | C++ aquifer.h | 一致性 |
|---|---|---|---|---|
| 1 | apply 入口 density>0 → null | AquiferSampler.java L149-151 | L74 | ✅ |
| 2 | 默认 fluidLevelSampler（y<-54 lava / 63 water） | AquiferSampler.java L153（外部 sampler）；NoiseChunkGenerator.createFluidLevelSampler | L77-82 `defaultFluidLevel` | ✅ |
| 3 | 3D 邻居 2×3×2 随机点（split(x,y,z)+nextInt(10/9/10)） | AquiferSampler.java L158-207 | L90-104 `getBlockPos` | ✅（源码一致；md5 实现未执行验证） |
| 4 | 最近/次近/第三近点 o/p/q 排序 | L168-207 | L90-104 | ✅（同上） |
| 5 | d=maxDistance(o,p)，d≤0 → 返回最近点液面 | L209-214 | L106-113 | ✅ |
| 6 | water+lava 下邻居分支 | **L215 用 `this.fluidLevelSampler.getFluidLevel(i,j-1,k)`（外部默认）** | **L114-117 用 `getFluidLevel(...)`（13 邻居细化）** | ⚠️ **源码差异**（本课题海底面无影响：blockY-1≥51>-54 时 Java 外部 sampler 恒 water 非 lava，分支不触发；但属潜在 bug） |
| 7 | e = d*calculateDensity(fl2,fl3)；density+e>0 → null | L219-224 | L119-126 | ✅（条件一致；差值在 fl2/fl3 值） |
| 8 | g = d*f*calculateDensity(fl2,fl4)；h = d*g2*calculateDensity(fl3,fl4) | L226-243 | L128-138 | ✅（条件一致；差值在 fl2/fl3/fl4 值） |
| 9 | calculateDensity：j=|fl.y-fl2.y|，j==0 → 0 | L269-272 | L244-246 | ✅ |
| 10 | calculateDensity 主体（e/f/o/q/r+barrierNoise） | L274-317 | L247-273 | ✅ |
| 11 | getWaterLevel/getWaterLevelAt 网格缓存（16×12×16 单元） | L335-351 | L279-287 | ✅ |
| 12 | getFluidLevel 13 邻居扫描（CHUNK_POS_OFFSETS / OFFSETS） | L353-389 | L290-319 | ✅（偏移表逐项一致：{0,0},{-2,-1},{-1,-1},{0,-1},{1,-1},{-3,0},{-2,0},{-1,0},{1,0},{-2,1},{-1,1},{0,1},{1,1}） |
| 13 | estimateSurfaceHeight（BiomeCoords 4 对齐；initialDensityWithoutJaggedness > 0.390625 自顶向下步长 8） | ChunkNoiseSampler.java L222-240（`initialDensityWithoutJaggedness` L234） | L145-164（`initialDensity`）；**worldgen_api.cpp L376 `{"initial_density","initial_density_without_jaggedness"}`** | ✅（入口函数正确；(-244,-256)=32 两测一致 aqfj L5-6） |
| 14 | getFluidBlockY（erosion/depth 判定 + floodedness + barrier） | L391-419 | L329-353 | ✅（数值等价；method_43718 条件已核实：VanillaBiomeParameters.java L1206-1208 `erosion<-0.225F && depth>0.9F` = C++ L335） |
| 15 | getNoiseBasedFluidLevel（spread 噪声 + est 截断） | L421-433 | L355-366 | ✅ |
| 16 | getFluidBlockState（lava 判定，-32512 无效液面） | L435-450 | L368-381 | ✅ |

**逐项对照结论**：apply 控制流、calculateDensity 公式、液面链 13 邻居结构、estimateSurfaceHeight 入口、getFluidBlockY 分支——**源码逐行等价**。唯一源码级差异是 #6（water+lava 分支的液面来源，对海底面无影响）。**没有发现 C++ 明显漏写某个「修正项」**——C++ 把 Java 的公式全部翻译了。差异必然来自**输入值**（fl2/fl3/fl4 的 y，或 o/p/q）。

### 3.1 为什么 C++ e≡0 而 Java 需要 e≠0（推理）

- 要判实心（null→stone），必须 `density+e>0`（或 g/h 分支）。d≤0 分支只返回 water/air。
- C++ 在 y=58 d=0.64>0 走 e 分支，但 fl2.y==fl3.y==63 → j=0 → e=0 → 水。
- 若 Java 的 fl2.y≠fl3.y（如候选 A：63 vs -32512 无效液面），j=32575 → e=blockY+0.5-d=16283>0 → o=4.5 → q=3.0（>2.0 → r=0 不采 barrier）→ calculateDensity=6.0 → e=d*6.0=3.84 → density+e=+3.77>0 → **null（stone）翻转**。
- **y=62 的不翻转是自洽的**：C++ trace y=62 o=42,p=91 → d=-0.96≤0 → 直接返回 water；若 Java o/p 相同也走 d≤0 → water（与 vanilla y=62 water 一致）。
- **但 y=59-61 的 vanilla 实心是难点**：C++ 该 3 层 d≤0（直接 water）；若 Java o/p 与 C++ 相同，Java 也 d≤0 无法走 e 分支 → 无法实心。**只有两种自洽出路**：
  - (a) **Java o/p/q 与 C++ 不同**（随机点 splitter 派生差：md5("minecraft:aquifer") 或 hashXYZ/nextInt 实现）→ d>0 → 走 e 分支 → 翻转；
  - (b) **Java 液面网格与 C++ 不同**（13 邻居 est/floodedness 输入链），使 fl2/fl3/fl4 的 y 组合产生非零 e/g/h。
- 二者都是 B3 家族（「液面/e 值相关输入未复刻」），但修复点不同。phase5 §4 曾判 splitter 派生链源码闭合（xoroshiro.h:75-79 vs Java XoroshiroRandom.Splitter.split；hashXYZ L13-23 vs MathHelper.hashCode），**md5 实现正确性未执行验证**——(a) 未彻底排除。

### 3.2 排除项（为什么不是这些）

- **density 差**：已证同基准同值（phase5 §3.2，≤3e-6）。❌
- **estimateSurfaceHeight 入口**：(-244,-256)=32 两测一致；C++ 用的 initial_density 映射到 initial_density_without_jaggedness（worldgen_api.cpp L376），与 Java L234 同函数。❌（13 邻居的 est 未对照，属于 (b) 候选）
- **结构覆盖（ocean_ruin）**：phase5 曾留 (B) 出路。本分析用**形态 + 统计**排除：y=51-57 中空柱不可能；结构/FEATURE 总量仅 3.6%（phase2 §1.2），而 6710 块系统性遍布海底面附近（phase2 §2 表 2-2 组 B 海域边界）。❌ 作为主解释。
- **surface 规则把海床染色位置放低**：SURFTRACE 显示 C++ 在 y=50 把 stone 染 sand，说明 surface 只是**反映**底层实心位置（y=50 海床顶）；vanilla 主海床也在 y=50（y=40-50 stone）。表面染色高度不是差源——**差源是 y=58-61 浮岛的 aquifer 实心缺失**（surface 无可染基底）。❌

---

## 4. 可解释块数估算

目标差块（`got=water vs vanilla 实心`，y=52-62 海底面附近）：

| vanilla 实心 | got=water 块数 | 来源 |
|---|---|---|
| stone | 3117 | m288_pair_counts.txt L8 |
| dirt | 2539 | L9 |
| sand | 723 | L18 |
| sandstone | 198 | L32 |
| gravel | 133 | L44 |
| **合计** | **6710** | 与任务背景 6710 吻合 |

**估算**：
- B3 机制（e/g/h 翻转缺失 → 浮岛/海床判定差）是这 6710 块的**直接判定机制**——vanilla 实心全部依赖 aquifer 的 `density+e>0` 翻转，C++ 无翻转即全水。
- **可解释区间：~4000-6710 块**（保守 60% / 乐观 100%）。合理中点 **~5000-5500 块**。剩余不确定性来自：部分块可能同时涉水洞边界液面（-32512→air）路径（若 d≤0 分支液面差异使 Java 返回 AIR，则表面层可能因后续 FEATURE 变化而非 aquifer 直接实心——但 AIR 非实心，最终仍要 aquifer 实心才出现 stone/dirt）。
- **反向差** `got=stone vs vanilla=water` = 4416 块（含水层，phase2 表 2-2 组 D）+ `deepslate vs water` 635 = 5051 块，可能是同一 e 翻转的另一侧表现（Java 在该处 density+e>0 不成立 → 水，C++ e=0 判实心），**计入潜在收益但置信度低于正向**（含洞穴/gravel 干扰，不单独计数）。
- 与 B3 无关的差（不在本机制范围）：gravel 类 4900（OreFeature blob + 海底 gravel，surface/ore 层）、表面规则 2900、洞穴 6428、OreFeature 石质变体 3.3 万。

**结论：B3 修复最大可闭合 ~6710 块（海底边界主体）+ 可能部分反向 5051 块，占总差量 67042 的 ~10%（6710/67042）— 17.5%（(6710+5051)/67042）。**

---

## 5. 范围内待修建议（交主会话/审查裁决）

**定性：C++ bug（aquifer 判定链输入未复刻 Java），非基准差异。** 参照（vanilla .blocks，FULL 状态）为 ground truth：y=58-61 实心是 vanilla 真实输出，C++ 判水是错误；density 已证一致，错误只在 aquifer 内部。

待修（按优先级，需先补数据闭合）：

1. **补 Java 侧 aquifer 中间量 dump（决定性，先做）**：在 Java 真实遍历内打印 (-244,55..62,-256) 的 `o/p/q/d/fl2.y/fl3.y/fl4.y/e/g/h/density+e`（对齐 C++ trace [AQF]/[AQF-e] 行）。**注意不得再用 AQF-J 反射**（phase5 §2 判 CellCache 污染不可信）——必须在 onSampledCellCorners/interpolate 真实遍历内取。
   - 判定树：o/p/q 与 C++ 一致 + fl.y 不同 → **(b) 液面网格链**（修 getFluidLevel/getFluidBlockY 13 邻居输入：est 缓存边界、floodedness/fluidSpread 采样位置、或 -32512 无效液面传播）；o/p/q 不同 → **(a) 随机点 splitter**（验证 md5("minecraft:aquifer")、hashXYZ、nextInt 重掷逻辑）。
2. **若 (a)**：核对 C++ `md5.h` 与 Java `RandomSeed.java:33-38` 对 "minecraft:aquifer" 的 MD5（RFC1321）一致性；补 C++ blockPositions dump 与 Java 反射对照（一次性）。
3. **若 (b)**：逐项核对 13 邻居列的 estimateSurfaceHeight（当前只验证了 (-244,-256) 单点）与 fluidLevelFloodedness 采样位置（C++ getFluidBlockY L341 `fluidFloodedness->sample(pos)` 传块坐标 vs Java UnblendedNoisePos——确认 C++ raw DF 采样与 Java unblended 语义一致）。
4. **附带修复（独立小 bug，非本课题主因）**：aquifer.h L114-117 的 water+lava 分支用 13 邻居细化 `getFluidLevel`，Java L215 用外部 `fluidLevelSampler`——改为 `defaultFluidLevel(blockY-1)` 对齐 Java（对海底面无影响，但消除潜在 lava 判定差）。

---

## 6. 置信度汇总

| # | 结论 | 置信度 |
|---|---|---|
| 1 | density 同基准同值，判定差不在 density | 确定（phase5 已证） |
| 2 | 参照列 y=58-61 是 aquifer 浮岛实心（形态排除 ocean_ruin 柱） | 确定（ref_col + 结构统计） |
| 3 | 6710 块「C++ 水 vs vanilla 实心」系统性分布在海底面附近，非结构假 diff | 确定（m288_pair_counts + phase2） |
| 4 | 实心只能由 `density+e>0`（e/g/h 非零）产生；C++ e≡0 → 水；**B3 机制成立** | 确定（apply 控制流推理） |
| 5 | C++/Java 液面公式源码逐行等价，未发现漏写修正项；差异在输入值（fl2/fl3/fl4 或 o/p/q） | 确定（源码对照表 §3） |
| 6 | 具体差异位置：随机点 (a) vs 液面网格输入 (b)，现有数据无法 100% 闭合 | 推测（需 Java 中间量 dump） |
| 7 | 可解释块数 ~4000-6710（主目标 6710 的大部分） | 推测（机制直接、比例未测） |
| 8 | water+lava 分支液面来源差异（aquifer.h L114-117 vs Java L215）为独立潜在 bug，海底面无影响 | 确定（源码对照） |

---

## 附：证据索引

- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java`（apply L145-251、calculateDensity L263-321、getFluidLevel L353-389、getFluidBlockY L391-419）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/ChunkNoiseSampler.java`（estimateSurfaceHeight L222-240、aquifer 构造 L160-174、initialDensityWithoutJaggedness L187/234）
- `versions/1.20.1/data/mc_src_extract/net/minecraft/world/biome/source/util/VanillaBiomeParameters.java`（method_43718 L1206-1208）
- `versions/1.20.1/cpp/worldgen/src/aquifer.h`（apply L70-140、estimateSurfaceHeight L145-164、getFluidLevel L290-319、getFluidBlockY L329-353）
- `versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp`（L376 router 映射、L590-597 aquifer 构造）
- `versions/1.20.1/cpp/worldgen/src/xoroshiro.h`（hashXYZ L13-23、Splitter.split L75-79）
- `.investigations/-288-reopen/cmd-output/trace_aqf_1.txt`（C++ e=0、SURFTRACE）
- `.investigations/-288-reopen/cmd-output/aqfj_blockprobe.txt`（est=32 一致、CppCmp/CppCmpS）
- `.investigations/-288-reopen/ref_col_-244_-256.txt`（vanilla 列 ground truth）
- `.investigations/-288-reopen/m288_pair_counts.txt`（6710 块统计）
- `.investigations/-288-reopen/analysis-phase2.md`（海域边界 19%、结构 3.6%）
- `.investigations/-288-reopen/analysis-phase5.md`（density 同基准、AQF-J 不可信、判别树）
