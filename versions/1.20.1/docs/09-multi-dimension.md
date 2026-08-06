# 09 · 多维度通用引擎（下界跑通 72%）

> 2026-08-06 追加（主世界 1.20.1 全部 100% 后新增）。本文是 docs/01-08 的延续——**多维度定位**（用户初始目标：通用引擎，不是白名单 vanilla 三界）。
> 配套：docs/01 架构映射（C++ 引擎结构）、docs/08 版本迁移方法论（跨版本流程）。

## 定位：通用引擎（数据驱动任意维度）

1.18+ 所有维度（主世界/下界/末地/暮色等）共用同一套底层：`ChunkNoiseSampler` + `DensityFunction` 树 + noise_settings JSON，**差异只是数据**。CoreSwap 的 C++ 是「密度求值引擎」（JSON 解析 + 树求值），主世界只是第一个应用实例。

**通用结构（wg_create 纯数据驱动）**——不再有「维度」概念：

```cpp
wg_create(seed, dataDir, settingsName, biomeParamsFile, worldHeight);
// 引擎从 noise_settings/<settingsName>.json 读：minY / noiseHeight / aquifersEnabled
// settingsName 决定 density namespace/目录（"overworld.json"→overworld；mod 维度传自己的设置名）
```

- mod 维度只要 Java 侧把它的数据准备好（noise_settings JSON + biome 参数 + 世界高度）传给 wg_create 就能生成
- 例外（少数不通用）：自定义 BiomeSource 类 / 完全自定义生成器类的 mod 走 vanilla

## 下界跑通（72% 全匹配 / 75.4% 非 air 匹配）

**修复链**（每步主世界 100% 回归保持）：

| 修复 | 内容 |
|---|---|
| surface_rule JSON 解析器 | sequence/condition/block/vertical_gradient/not/y_above/biome/stone_depth/noise_threshold/hole/steep/water/temperature/surface 全节点 → C++ 规则树（mod 维度通用） |
| VerticalGradient 反锚序 | 先 false 后 true（nether bedrock_roof：trueAt=顶部 > falseAt=顶下 5）——主世界锚序正常两序一致 |
| out 越界写 | `BLOCK_COUNT`(98304) → 维度大小（nether 65536）——崩溃根因 |
| y 循环上限 | noiseHeight（nether 128）——y 128-255 留 air——22% → 72% |
| BlockProbe 维度化 | 导下界 vanilla 参照（`vanilla_*_nether.blocks`，高度 256） |

**下界关键数据**（nether.json）：
- min_y=0、noise.height=128、世界高 256（**两者不同**——y 循环用 noiseHeight，上方留 air）
- aquifers_enabled=false、ore_veins_enabled=false（**无 aquifer/oreVein**——跳过）
- sea_level=32、default_block=netherrack、default_fluid=lava
- noise_router 仅 5 组件有值（barrier/continents/depth/erosion=0.0 常量 + final_density 内联；temperature/vegetation=shifted_noise）
- final_density 引用 `minecraft:nether/base_3d_noise` = **old_blended_noise**（参数内联：y_scale 0.375/y_factor 60——与主世界 0.125/160 不同）

## 密度级对比工具（新）

- **DensityProbe.java**（Java）：`-PdensityProbe=true -PdensityProbeDimension=nether -PdensityProbeChunkX/CZ/X/Z` 导 vanilla finalDensity 剖面（`vanilla_density_<dim>_c<cx>_<cz>_b<bx>_<bz>.txt`，y 每 4）
  - **拿 finalDensity 的正确路径**：`cm.getNoiseConfig().getNoiseRouter().finalDensity()`（yarn）
  - ❌ 反射 cns 的 finalDensity 字段不存在；`initialDensityWithoutJaggedness` 下界是常量 0（无用）
- **got_export -densityDump cx cz bx bz**（C++）：同格式 dump（固定下界）
- **wg_sample_density(handle, x, y, z)** API（直接采样 finalDensity）

## 剩余 28% 差异（下界）= density（非 surface 解析器）

- lava（33）差异集中在 y 2-31；air 差异集中在 y 32-63（洞上方——C++ 填了、vanilla air）
- **C++ finalDensity 在 y 48-80 微负**（~0.04），vanilla 微正——洞未形成 → hole 条件不命中 → lava 规则空转
- 密度对比（chunk 0,0 列 0,0）：**y=0 完全一致**（0.458333）、y≥4 振荡差（±0.02~0.2）——**像 base_3d_noise 的 y 方向采样差**，但参数已确认读对（[NB] y_scale=0.375 y_factor=60）
- **下一步**：base_3d_noise 分量直接对比（C++ vs Java 的 InterpolatedNoiseSampler 采样——RouterProbe 已有 b3d 构造可扩展）

## 已知坑（勿重蹈）

- **runDepth 洞内重置会破坏主世界**（99.86%——2268 块差异）——主世界 100% 是铁律；lava 的 hole（runDepth<=0）机制需先确认 MC 1.20.1 SurfaceBuilder 源码，不要猜
- nether/base_3d_noise 参数**不在 NOISE_PARAMETERS 注册表**（是 old_blended_noise 内联）——noise_params.json 只含 38 个 minecraft:noise 型参数
- 下界 y_scale(0.375)/y_factor(60) ≠ 主世界(0.125/160)——old_blended_noise 分支默认值写主世界的，必须从 JSON 读
- 密度采样用 `UnblendedNoisePos` 直接调 `router.finalDensity().sample` 有效（RouterProbe 验证过）

## 2026-08-06 晚补充：1.0.5 崩溃修复（mod id 漏改）

**现象**：1.0.4 客户端进入世界崩溃 `CppWorldgen.java:36`（NoSuchElementException）。

**根因**：mod 改名 worldgen-bench→coreswap 时只改了 `CppBridge.java:51`，**`wg/CppWorldgen.java:36` 的 `getModContainer("worldgen-bench")` 漏改**。CppWorldgen 用独立的 `%TEMP%\coreswap-native` 缓存（CppBridge 是 coreswap-data）——本地测试只删了 coreswap-data 未暴露。

**修复**：1.0.5（改 coreswap + 全项目 grep worldgen-bench 清零）。社区 PR #2（dustinmoon78）独立发现并修复同一处，已合并。

**⚠️ 改名类改动验证铁律**：全项目 grep 旧名清零 + 删 %TEMP%\coreswap-data 和 %TEMP%\coreswap-native **两个**缓存目录（缺一不可）再模拟全新环境。

## 2026-08-06 晚补充：Forge+Connector 兼容（1.0.6）+ 已解决项标注

**CoreSwapFixHelper**（wg/bench/，合入社区 PR #3 dustinmoon78 思路）：
- 多级定位 mod jar：codeSource → FabricLoader.getAllMods()/ModOrigin.getPaths() → classloader 兜底，JarFile 提取
- 原因：原 getRootPaths() 在 Forge UnionFileSystem（Sinytra Connector）下不可遍历
- 增强：dev classpath（目录）也支持（Fabric 开发环境回归通过——codeSource/classloader 是目录时走 Files.walk）
- **1.0.6 发布**（Pre-release，Forge 需 Sinytra Connector）；PR #3 已关闭（感谢）

### ✅ 已解决项标注（不删除历史，只标注状态）
- ✅ **1.0.5 崩溃**（mod id 漏改 CppWorldgen.java:36）——已解决（1.0.5 发布 + 双缓存验证铁律）
- ✅ **y_scale 参数**（nether 0.375/60）——已确认读对（[NB] 打印），非参数问题
- ✅ **out 越界写 / y 循环上限**——已解决（下界 22% → 72%）
- ❌ **runDepth 洞内重置**——未解决（回滚；主世界 100% 铁律；lava 的 hole 机制待 MC 1.20.1 SurfaceBuilder 源码确认）
- ❌ **下界 28% 密度差**（y 48-80 微负）——进行中（base_3d_noise 分量对比工具已就绪：wg_sample_named + got_export -nbDump）

## 2026-08-06 深夜补充：base_3d_noise 分量判定

- ✅ **C++ 的 base_3d_noise 正确**：主世界 b3d(0,0,0) = -0.318090（主世界 100% 证明）、下界 b3d(0,0,0) 同 = -0.318090（deriver/octave 一致，y0 缩放 0 坐标同）——下界 b3d 采样无问题
- ❌ **Java 侧参照不可靠**：cns.actualDensityFunctionCache 里 key 含 base_3d_noise 的函数采样 -0.080 ≠ 真实 -0.318（拿到的不是 old_blended_noise 本身）；rd 反射构造同样 -0.080（同源问题）——**b3d 分量对比走不通，别再用 Java 侧 b3d 采样当参照**
- ❌ **final 差（y 4-36）根源未定**：b3d 已排除（C++ 对），嫌疑收窄到 CellCache 网格值/渐变/常数等 final 树内部环节——需 Java 侧网格级对比（下阶段）

## 2026-08-06 深夜补充 2：684.412 精度排除（final 差根源收窄）

- ❌ **684.412 float/double 精度差排除**：模拟 Java 684.412f（`(double)(float)684.412`）后下界 final **完全无变化**（主世界 100% 保持）——已回滚。坐标差（~7e-4@y24）不足以产生噪声差
- 排除清单：b3d（✅ C++ 对，deriver/octave 一致）、684.412（✅ 排除）、y_scale 参数（✅ 读对）
- **嫌疑收窄**：CellCache 网格值 / YClampedGradient（下界 from_y -8 负锚）/ 插值实现——需 Java 侧网格级对比或 C++ 侧分量采样 API（YClampedGradient 是内联节点，无注册名）

## 排查方法论（密度差定位，勿重复踩）

1. **b3d 分量对比的 Java 参照陷阱**：cns.actualDensityFunctionCache 里 key 含 base_3d_noise 的函数 ≠ old_blended_noise 本身（采样 -0.080 vs 真实 -0.318）；rd 反射构造同源错——**判定用「主世界交叉验证」**（主世界 100% 证明的值当基准：b3d(0,0,0) 主世界=下界=-0.318090）
2. **排除法顺序**（每步主世界 100% 回归）：b3d（✅）→ y_scale 参数（✅）→ 684.412 精度（✅ 模拟 float 无变化）——嫌疑收窄 CellCache 网格/y 方向大坐标
3. **下界 y 方向坐标放大**：y_scale 0.375 使 octave 采样坐标达 2000+（主世界 0.125 只有 ~700）——大坐标浮点是下阶段重点

## 2026-08-07 凌晨补充：maintainPrecision 排除 + 嫌疑锁定 CellCache 网格

- ❌ **maintainPrecision 排除**：C++ `lfloor(v/3.35e7+0.5)`（四舍五入）vs Java 疑似截断——但主世界 100%（大坐标 1.8e10 也折叠）+ 下界折叠值小数 <0.5（2.007）两实现同；大坐标（y 24 → 6159）double 精度足够（ulp ~9e-13），非浮点问题
- ✅ 排除清单完整：b3d / y_scale / 684.412 / maintainPrecision / 浮点精度
- **嫌疑锁定**：CellCache 网格（C++ InterpolatedDF vs Java DensityInterpolator）——final 差网格点（y 0 一致、y 8 差 0.014、y 24 差 0.09 峰值）与非网格点同趋势，且 b3d/渐变/常数均已排除 → **网格构建/插值环节**（y 方向）
- 下一步：Java 侧反射 cns 的 CellCache/DensityInterpolator 网格值，与 C++ InterpolatedDF 网格逐点对比

## 2026-08-07 补充：b3d 差坐实 + 两个诊断 bug

- ✅ **maintainPrecision 修复**：C++ `lfloor(v/3.35e7+0.5)`（四舍五入）→ Java `(long)(v/3.35e7)`（向零截断）。主世界折叠值小数<0.5 未暴露（100% 保持）；下界 o 从 1.0 递减、e*o 最大 6159 不触发折叠（本次 chunk 无影响，但语义必须对）
- ✅ **nbDump 维度 bug**：`atoi("-dimension")=0` → 之前「下界 b3d 正确」判定全基于主世界 b3d！修复后真下界 b3d：y0=-0.318、y8=-0.2226、y24=-0.1482（与 final 反推 -0.148 完全一致，内部自洽）
- ❌ **C++ 下界 b3d@24=-0.148 vs Java 反推+0.133（差 0.28）坐实**——deriver/参数/octave/实现全核对过无差；剩两个嫌疑：① 684.412f vs double（e 差 4.3e-4@y24，之前模拟无变化但可能没生效）② Java 侧 UnblendedNoisePos 采样路径（interpolated 直接 arg 的语义）待核实

## 2026-08-07 状态总览（继续前必读）

- **下界 72% 已可用**（TOTAL 71.97% / nonAir 75.41%，chunk 0-3）；主世界 100% 铁律保持（每次改动回归）
- **b3d 差坐实**：C++ 真下界 b3d@24=-0.1482（final 反推一致，内部自洽）；Java 侧 DensityProbe final@24=0.0425 反推 b3d≈+0.133，差 0.28
- **已排除**：b3d 实现/deriver/octave 参数/scale factor 参数/684.412f（e 差 4e-4→噪声差 4e-4 量级，数学排除）/maintainPrecision（下界 o 递减不触发）
- **关键疑点**：DensityProbe 用 UnblendedNoisePos 采样，游戏实际走 BlendedNoisePos（CellCache 网格）——两条路径语义可能不同；cns cache(-0.073) 与 RouterProbe(-0.080) 两参照互不一致，均不可靠 → **需游戏实际路径（CellCache 网格）的 b3d 真值**
- **下一步（选 1）**：修 RouterProbe 的 deriver 来源（与 NoiseConfig 一致）或反射 CellCache 网格值，拿游戏实际路径的下界 b3d@24 真值对比 C++（-0.1482）
