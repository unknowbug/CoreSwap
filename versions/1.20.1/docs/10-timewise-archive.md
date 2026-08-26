# 10 · 排查时间线（2026-08-06 起）

> **本文档 = 时间线/过程记录**（2026-08-08 知识库重构时从原 09 全文迁移）。时间线文档是必要的：保留完整推理过程、被推翻的假说、工具演进，防重走弯路。
> - **查结论** → 01-09 主题篇（结论检索）；**查过程/被推翻假说** → 本文档
> - 新排查记录**按日期追加**到本文档末尾，每条带状态标注（✅ 已解决 / ❌ 已排除 / 🔍 排查中 / 已结案）；积累到量大时按主题分组整理一次
>
> 以下为原 09 全文（保留完整历史，被推翻的结论以各篇「已验证结论」/09 篇为准）。

---

# 09 · 多维度通用引擎（下界跑通 72%）

> **文档定位（2026-08-08）**：本文档 = **排查时间线**（2026-08-06 起的每日进展/修正/工具），保留全部历史（铁律：追加不覆盖）。
> **已确认的结论/坑已按主题提炼到 01-08 各篇末尾**（「2026-08-08 已验证结论」章节）：02 随机（maintainPrecision/nextDouble float）、03 密度函数（Cache2DDF/WeirdScaledSampler/8 interpolated 映射）、04 含水层（est 两版一致/aquifer 链）、05 矿脉（FEATURE 假 diff）、06 表面规则（8 邻域/SurfaceCondC/terracotta 带）、07 流水线（并发崩溃/dll 对齐/seed 校验）。
> **查结论优先看 01-08 对应篇；看完整推理过程/时间线/工具演进看本文档。**

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

## 2026-08-07 决定性：下界 b3d 与 Java 游戏实际 deriver 逐位一致（b3d 彻底排除）

- ✅ **b3d 排除（最终）**：RouterProbe 反射 NoiseConfig.randomDeriver（游戏实际 deriver）+ 下界参数（0.25/0.375/80/60/8），采样 (0,y,0) 列 16 点——与 C++ 下界 b3d **逐位一致**（y24: -0.14815987141887240 vs -0.148160）——deriver 状态漂移假说也排除（rd2 反射可用）
- ❌ **DensityProbe 的 UnblendedNoisePos 路径不可靠**（坐实）：y24 final=0.0425 是 per-call CellCache 插值结果（≈0.5*(arg@16+arg@32)*0.64 squeeze），不是直接 arg@24（=-0.148）——**DensityProbe 下界 final 数据不再作为参照**
- **推论**：C++ 的 CellCache 网格（buildGrid 直接 arg 采样）与游戏实际路径一致（b3d 一致证明）——下界 72% 方块差**不在 density**，在 surface 规则（runDepth/hole/lava，之前已定位）
- **下一步**：surface 规则差异（lava 25365/洞穴——runDepth 洞内重置需先确认 MC 1.20.1 源码）

## 2026-08-07 潘多拉审计回应（证据精度 + 耗时占比 + noise-in-Java 开关）

**论点 1（位级一致证据精度）——已修正并实锤**：C++ nbDump 原 %.6f（6 位小数）不足以支撑「逐位一致」；改 %.17g + %a（hex float）双格式，Java RouterProbe 加 Double.toHexString 对照——**y0-60 全部 16 点 17 位有效数字 + 53 位尾数 hex 逐位相等**（含 %.17g 显示差 1 ulp 的 y12，hex 完全一致，确认是打印舍入）。CoreSwap 卖点「exact IEEE double」证据链：主世界 block 100% + b3d hex 位等。

**论点 2（耗时占比）——数据补钉**：WG_PROFILE 计时器（noise/spline 分项 ns 累计）实测 16 chunks：spline 99.4ms(100240 次, 单次 992ns) vs noise 30.6ms(37539 次, 单次 815ns)——**spline 总耗时是 noise 3.2 倍，单次 Perlin 不比单次 spline 贵**。「noise 不是热点」从次数对比升级为耗时占比铁证；把 noise 留 Java 无性能收益（noise 仅占 density 阶段小头，留 Java 还加 JNI 往返）。

**论点 3（noise-in-Java 迁移开关）——采纳为 v1.2 多版本迁移工具**：
- 运行时：默认全 C++（性能模式），不开开关
- 开发期：`-Dcpp.noiseInJava=true` 时，C++ 的 InterpolatedNoiseDF 采样改走 JNI 调 Java（游戏侧 old_blended_noise 永远对），C++ 只算确定性管线（渐变/常数/插值/含水层/表面）
- 新版本适配流程：先开开关跑通（noise 免复刻）→ 验证正确性 → 逐个复刻 noise 回 C++ 拿性能
- 双用途：多版本迁移脚手架 + 兼容模式兜底
- **实施时机：v1.2**（当前先修 surface 立确定性管线）

**论点 4（修 surface = 浇筑多版本稳定基础）——采纳为当前主线**：runDepth/hole/lava 是确定性逻辑（整数/规则判断），修完即确定性管线成型。

## 2026-08-07 surface 字段错位修复 + lava 差根源再定位

- ✅ **字段错位修复**（反编译 SurfaceBuilder 字节码确认）：C++ 把 sampleRunDepth（=Java getSurfaceDepth 列初始）错存进 ctx.runDepth——y_above/stone_depth 用 runDepth 碰巧对（Java 也用列值 surfaceDepth）；**hole 用错**（Java 用 runDepth 扫描计数器：空气→0、非空气非流体→++、流体→保持）。修复：surfaceDepth/runDepth 分离，hole 改 stoneDepthAbove（扫描计数），主世界 100% 保持
- ❌ **下界仍 72%**：字节码显示 Java buildSurface **跳过流体格**（goto 跳过规则应用）——**lava 不是 surface 规则生成**，来自 fillFromNoise 的流体填充（下界 fluid_level 组件）——C++ 的 3b 阶段下界无 aquifer 时跳过了流体填充 → lava 差 25365 根源
- **下一步**：C++ fillOneChunk 下界分支补流体填充（Java ChunkNoiseSampler 的 fluid 逻辑：fluidLevelFloodedness/fluidLevelSpread 组件）

## 2026-08-07 悬空结构根因（问题 1）

- **现象**：村庄塔楼/帐篷悬空（存档实测：帐篷地板 y=69，实际地表 y=62，悬空 7 格）
- **HeightProbe 实测**（seed -4763191261905561195，位置 20,-468）：Java getHeight=67（各 heightmap 类型一致）、getColumnSample 列 y55-66 stone（地表 66）；实际方块地表 62；C++ final@64=-0.000415（与存档一致）
- **根因**：Java 结构放置高度（getHeight/getColumnSample）用 dho 构建（ChunkNoiseSampler horizontalBlockCount=1 的 1-cell 网格 density 采样），实际方块生成用 4-cell 网格——塔楼位置 y63-66 两者差 ~0.0004 → **符号翻转**（Java 地表 66/67 vs 实际 62）→ 结构放高 4-5 格
- ❌ **未定**：vanilla（无 mod）同一位置村庄是否同样「高 5 格」（若是 MC 本身行为则非我们差异；若 vanilla 落地则结构放置路径有差）——需 vanilla 对照
- **下一步**：vanilla runServer 对照（无 cppReplace 同 seed 生成村庄看 start 高度）

## 2026-08-07 悬空结构 vanilla 对照结论（更新）

- ✅ **getHeight/getColumnSample 与 vanilla 完全一致**（67/66——纯 Java density 采样，不受 CoreSwap 影响）——结构 start 高度逻辑是 MC 原生行为
- ❌ vanilla 单 chunk 生成不含村庄 start（结构跨 chunk）——无法直接对比村庄落地；完整对比需预生成村庄 start 区域（成本高）
- **定性**：悬空结构非 CoreSwap 引入（高度逻辑没被改）；「MC 为什么 vanilla 不悬空」涉及结构模板落地机制/多 chunk blender——MC 深水区，暂缓
- **优先级调整**：先做问题 2（远处虚空——BATCH 攒批吞吐，性能/体验更实际）

## 2026-08-07 🔴 块状断裂根因坐实：负坐标 chunk C++ 生成差异（重大）

- **用户方案**（CoreSwap 存档 vs vanilla 存档同 seed=97 对比）直接暴露：**535 个差异 chunk，全部集中在负坐标（x<0/z<0）**，越负差越大（最大差 52 格）；正坐标区域逐位一致
- **模式**：CoreSwap 地表普遍比 vanilla 低（chunk(-18,-16) (0,37,89)：vanilla 89 vs CoreSwap 37）；chunk(-1,9) 差 2-11 格（用户坐标 12,156 附近可见的「块状」）
- **根因**：C++ 负坐标 NOISE+SURFACE 生成差异（got_export 生成 chunk(-1,9) 地表 y67 vs vanilla 76-78）——**block_probe 100% 只验证正坐标 3200 区域，负坐标从未被覆盖**（JniProbe 的 98.5% 是下界）
- **候选**：负坐标的 floor/取模语义（Java Math.floorDiv/floorMod vs C++ %）、hashXYZ、aquifer/surface 列缓存索引、InterpolatedDF 网格负坐标
- **下一步**：densityDump 负坐标 vs vanilla 定位具体函数

## 2026-08-07 负坐标 bug 定位进展（系统排查）

- ✅ **取模/移位/坐标运算系统排查**：核心代码负数语义正确（floorDiv 向下、算术右移=floor、gx 非负、`((y+i)%n+n)%n` 负数安全）——排除 % 类 bug
- 🔍 **新矛盾**：densityDump（finalDensity 树采样）与 fillOneChunk 的 densityBuf（方块生成的密度）**在正负坐标都结果矛盾**——densityDump 显示 chunk(0,9) final@y52-88 全正、chunk(-18,-16) final@96 正，但方块生成地表分别 66/65（y67+/y67+ 空气）——**两个路径采样同一 finalDensity 树却结果不同**
- **嫌疑**：InterpolatedDF 的 thread_local 缓存（densityDump 与 fillOneChunk 的采样顺序/缓存命中不同 → 缓存值差异）——或 fillOneChunk 的采样路径有偏差
- **下一步**：WG_SURFDUMP 在 fillOneChunk 内部 dump densityBuf（逐 4 格 finalDensity），与 densityDump 同列对比——定位「采样结果差异」的确切 pos 与缓存行为

## 2026-08-07 负坐标定位进展（二）

- ✅ **乌龙排除**：`-densityDump` 模式写死 `nether.json`（下界）——之前的「densityDump vs densityBuf 采样矛盾」是对比错维度（下界 density vs 主世界方块），不成立
- ✅ **正确路径**（WG_SURFDUMP，fillOneChunk 内部主世界）：final@列(-280,-248) y64 正、y68 转负 → 方块地表 66（**C++ 方块自洽**）；vanilla 地表 89 → vanilla final@88 正 vs C++ -0.385——**finalDensity 树在负坐标差异坐实**
- ✅ **b3d 排除**：nbDump 主世界 b3d 负坐标（-280,-248）无异常跳变（y60-96 平滑 -0.03~-0.25）
- ✅ **InterpolatedDF 排除**：负坐标 chunk 定位/gx 非负/cx 0..3/clamp 不触发——网格逻辑对
- 🔍 **嫌疑收窄**：spline（depth/continents/erosion）在负坐标——C++ 负坐标 y88 分量：depth=-0.1807、continents=0.1505、erosion=0.1338——**需 Java 侧（DensityProbe）对照分量**
- **下一步**：DensityProbe 加分量 dump（负坐标 -280,-248 y88）对比 C++——定位 spline 差异

## 2026-08-07 负坐标定位进展（三）：spline 分量差异坐实

- ✅ **DensityProbe 分量对照（负坐标 -280,-248 y88）**：continents C++ 0.1505 vs Java -0.2188（**差 0.37**）；depth -0.1807 vs -0.3113（差 0.13）；erosion 0.1338 vs 0.2781（差 0.14）——**spline 分量在负坐标差异坐实**
- ✅ continents = flat_cache(shifted_noise(continentalness, shift_x, shift_z))——纯噪声；排除 FlatCacheDF 网格定位（k/l 计算对）、ShiftDF（等价 Java shiftA/shiftB）、b3d（负坐标正常）
- 🔍 **嫌疑收窄**：shifted_noise 内部的 **continentalness 噪声或 shift_x/shift_z 在负坐标采样差**（Perlin 层）
- **下一步**：拆分 dump continentalness / shift_x / shift_z（C++ vs Java 对照）——定位具体噪声实例

## 2026-08-07 负坐标定位进展（四）：spline 三分量全差 → 嫌疑 biomeAt

- ✅ continents/erosion/depth **三个 spline 相关分量在负坐标都差**（0.37/0.14/0.13）——共同点 = spline（biome 参数）
  - ❌ **已更正（2026-08-08）**：router 组件（comps 可信路径）在 -288/20000 全部 0 差异——此为假象，biomeAt 嫌疑排除。见文末「2026-08-08 修正」段。
- ✅ shift_x 不在 NoiseRouter 上（Java 拿不到）；shift 本身是 spline（offset），差是结果非原因
- 🔍 **嫌疑**：biomeAt（MultiNoiseBiomeSource.find）在负坐标返回错误的 biome → spline 值差 → 地形差（535 chunk）
- **下一步**：验证 biomeAt 负坐标（C++ vs Java biome id 对照）——biome 坐标/hash 嫌疑

## 2026-08-07 负坐标定位进展（五）：Perlin 负坐标差异坐实

- ✅ **b3d 负坐标 C++ vs Java 差异坐实**（RouterProbe 负坐标采样，seed 97，位置 -280,-248）：y0 C++ 0.0274 vs Java 0.0895——x=0 时位级一致、负坐标差
  - ❌ **已更正（2026-08-08）**：RouterProbe rd2 漂移假象——`-nbDump` vs 游戏实际 b3d 全部 3e-5 级一致（含负坐标），base_3d_noise 彻底排除。见文末修正段。
- ✅ 实现逐行对比（floorD/map/grad/sampleSection/Octave sample/maintainPrecision）均与 Java 1.20.1 一致——**排除明显的负数语义错误**
- 🔍 **剩余嫌疑**：① 684.412f vs 684.412 浮点差在负坐标放大（d = -280*171.103 差 0.0034 → 噪声差 0.06）；但 continents（不用 684.412）也差 0.37 → 不是唯一原因 ② 更深层 Perlin 负数语义（需最小复现）
- **下一步**：① 快速验证 684.412f ② 或最小复现（C++ 与 Java 同构造 continentalness 噪声对比负坐标采样）

# 🔴 负坐标 bug 排查完整时间线（2026-08-07，重大主线）

**现象**：块状断裂地形（用户：「初始块内正常、往外块状断裂，整体起伏对但能观察到块状」）；村庄塔楼悬空（结构放置高度 vs 实际地表）
**突破**：CoreSwap 存档 vs vanilla 存档同 seed（97）对比 → **535 个差异 chunk 全部集中在负坐标（x<0/z<0），越负差越大（最大 52 格）**

**排查链**（每步 ✅/❌）：
1. ❌ 批量错位（BATCH_BUFS 对应正确，无 DIAG）
2. ❌ 取模/除法/移位负数语义（floorDiv/算术右移/gx 非负都正确）
3. ❌ aquifer->apply（density>0 直接 stone 逻辑对）
4. ❌「densityDump vs densityBuf 采样矛盾」——**乌龙**：-densityDump 写死 nether.json（下界），对比错维度
5. ✅ 正确路径 WG_SURFDUMP（fillOneChunk 内部主世界 final）：final@(-280,-248) y68 转负 → C++ 方块自洽（地表 66）；vanilla 地表 89 → **finalDensity 树负坐标差异坐实**
6. ❌ b3d（x=0 位级一致——但负坐标未验证）
7. ✅ **Perlin 负坐标差异坐实**：b3d(-280,-248) C++ 0.0274 vs Java 0.0895（RouterProbe 负坐标采样）
8. ✅ 实现逐行一致（floorD/map/grad/sampleSection/Octave/maintainPrecision）
9. 🔍 嫌疑：① 684.412f vs 684.412 浮点差（负坐标放大）② continentalness（不用 684.412）也差 0.37 → 独立问题

**下一步**：最小复现（同 deriver 构造同一噪声采样负坐标逐位对比）；先快速验证 684.412f

## 2026-08-07 负坐标定位进展（六）：684.412f 排除

- ✅ **684.412f 验证**：scaledXzScale/scaledYScale 改 (double)(float)684.412（对齐 Java 684.412F）——主世界 100% 保持；**负坐标 b3d(-280,-248) 不变（0.0274）** → 684.412f 不是该坐标差原因（保留改动：对齐 Java 语义）
- 🔍 Perlin 采样差（0.0274 vs 0.0895）——同 deriver 同参数同实现——**需最小复现**（dump Perlin 内部值 floorD/map/grad 逐位对比）

## 2026-08-07 负坐标定位进展（八）：Perlin.sample 本身差坐实（最后一步）

- ✅ **B3D 内部 dump（C++ WG_B3DDUMP + Java RouterProbe 反射）**：interp oct0（s=-598.86, t=0, u=-530.42, v=4.277, w=0）**输入逐位全同**（d/e/f/g/h/i/j/k 一致）——但 **C++ res=0.068549 vs Java res=-0.010214**——**PerlinNoiseSampler.sample 本身差坐实**（不是组合/输入/参数）
- ✅ sample 内部（floorD/map/grad/sampleSection/lerp3）逐行一致——**差在 sample 内部的某细节**（origin 或 yScale 分支或某浮点步）
- **下一步（最终）**：dump sample 内部（floorD 结果、小数、map 结果、每 grad、lerp3 各步）C++ vs Java 逐位——定位到具体一行

## 2026-08-07 负坐标定位进展（九）：C++ Perlin 内部 dump 完成

- ✅ C++ Perlin.sample 内部（interp oct0）：origin=(110.147561,36.856976,54.622212)、d=-488.71、e=36.86、f=-475.80、**i=-489、j=36、k=-476**、g=0.287、h=0.857、l=0.203、n=0
- **下一步**：Java 侧同样 dump（反射 origin + 手动算 i/j/k/g/h/l）对比——origin 差→deriver/构造；i/j/k/g/h/l 差→floorD/浮点；全同→map/grad/lerp3

## 2026-08-07 负坐标定位进展（十）：Perlin origin 差坐实（deriver 层）

- ✅ **origin 差坐实**：interp oct0 Perlin origin——C++ (110.147561,36.856976,54.622212) vs Java RouterProbe (68.458186,92.923998,198.372974)——完全不同
  - ❌ **已更正（2026-08-08）**：origin 差是 rd2 漂移假象（进展十一已证）；b3d 实际一致。见文末修正段。
- ⚠️ **待确认**：RouterProbe 用 rd2（NoiseConfig.randomDeriver 反射）——状态可能被构建消费（seed 97 漂移）；需从游戏实际 cns 的 b3d 实例反射 origin（最可靠）或 C++/Java fresh deriver 对比 nextDouble 序列
- **如果 C++ deriver 差坐实** → 负坐标 535 chunk 全部源于 deriver（XoroshiroRandom）构造/序列差异 → 修 deriver

## 2026-08-07 负坐标定位进展（十一）：origin 差 = RouterProbe rd2 漂移假象（关键推理）

- ✅ **B 结论**：正坐标存档逐位一致（scan_diff 535 chunk 全负坐标）——**若 C++ deriver 差则正坐标也差 → C++ deriver 对**
- ✅ **origin 差是假象**：RouterProbe 用 rd2（NoiseConfig.randomDeriver），1.20.1 split(long) 用 lo^seed（当前状态）——rd2 被构建消费 → origin 漂移 → RouterProbe 负坐标 b3d 参照（0.0895）**不可靠**
- 🔍 **负坐标差是坐标相关**（Perlin 采样负坐标 or 更深）——**需游戏实际参照**（cns 的 CellCache 网格——游戏实际路径的密度）
- **下一步**：从 cns 的 CellCache 网格拿负坐标密度（游戏实际），对比 C++ InterpolatedDF 网格——定位 Perlin 负坐标采样差异

## 2026-08-07 负坐标定位进展（十二）：负坐标差异普遍坐实（所有 seed）

- ✅ **seed -8248 负坐标 = 95.47%（非 100%）**（block_probe 对比 vanilla 参照，4×4 负坐标 chunk）——正坐标 100%、负坐标普遍差——**负坐标 bug 与 seed 无关，真实 bug**
  - ❌ **已更正（2026-08-08）**：95.47% 大部分是 FEATURE 假 diff（-288 参照为 FULL 状态）；真 bug（SURFACE 参照）在 20000 正坐标 0.59%、8576 3.31%——**正坐标超阈值同样块状**。见文末修正段。
- ✅ 排除「seed 97 特殊」——负坐标 bug 影响所有 seed（埋雷确认）
- 🔍 根因仍在：Perlin 负坐标采样 or final 树负坐标——RouterProbe 参照不可靠（rd2 漂移），需游戏实际参照（cns CellCache）或差异模式定位

## 2026-08-07 负坐标定位进展（十三）：差异模式 = 地表高度偏移（Perlin 微小差坐实）

- ✅ **差异 y 分布**（seed -8248 负坐标 16 chunks，71207 块）：**集中在 y0-71（海平面附近，峰值 y56-63=10991）**，y72+ 几乎无（23）——**地表高度偏移几格**（density 零点差），非全列错
- ✅ **Perlin 负坐标微小差坐实**：-8248 差几格、seed 97 特定位置放大到 52 格（同根源不同幅度）
- 🔍 输入/实现全验证一致——差异在 Perlin.sample 内部的「微小浮点差」（floorD 边界 or 某步）——需更深逐位对比，或考虑 noise-in-Java 开关兜底

## 2026-08-07 负坐标验证案例（用户实跑）

- **seed 8576294172403134396**，玩家降落 (731, 82, -404)——**z=-404 负坐标区域，地形 bug 特别明显**（用户确认）
- 1.0.10 候选验证：客户端模式（线程数=物理核-2 留核）可正常进入世界（dll 提取 readJarBytes 修复后）
- 发布铁律：dumpbin 验证 + 主世界 100% 回归 + jar 内 dll 哈希

## 2026-08-07 负坐标定位进展（十四）：maintainPrecision 反编译确认 + 修复

- ✅ **反编译确认 Java 1.20.1 maintainPrecision**：`(long)(v/33554432.0 + 0.5)`（+0.5 后向零截断）——**C++ 曾误写成纯向零截断**——修复（对齐 Java）
- ⚠️ **但**：maintainPrecision 折叠只在 |坐标×scaledXz|×2^r > 3.35e7 时触发（|x| > ~19.6 万）——**玩家位置（731,-404）和 -8248 测试区（|x|≤225）不触发**——负坐标小坐标差异根源在别处（Perlin.sample 其他细节）
- **下一步**：小坐标负坐标 Perlin 差（输入/实现全验证一致）——需 dump sample 内部每步 vs 游戏实际，或考虑 noise-in-Java 兜底
  - ✅ **已解决（2026-08-08）**：Perlin/b3d 实际一致（3e-5），此「下一步」作废；真正根因 = finalDensity 树内 factor/sloped_cheese spline 系统差 + range_choice 阈值跨越。见文末「2026-08-08 修正」段。

## 2026-08-08 修正（重大：推翻 08-07 部分中间结论，勿再重查）

> 本段修正 08-07 时间线中被后续证据推翻的中间结论。**旧结论文字保留**（铁律），以本段为准。

### ❌ 已推翻的中间结论（对应上文条目）

1. **「Perlin 负坐标差异坐实」（进展五/八/九/十，b3d(-280,-248) C++ 0.0274 vs Java 0.0895）**——**假象**。08-08 用 `-nbDump`（C++ 可信路径）vs DensityProbe 游戏实际 b3d（actualDensityFunctionCache）验证：**-8248 @ -18,-13 列 8,8 与 1250,1250 列 8,8 的 base_3d_noise 全部 3e-5 级一致**（含负坐标）。进展十一的「rd2 漂移假象」推理正确，但进展五/八/九/十未同步标注——**base_3d_noise（InterpolatedNoiseDF）彻底排除**。
2. **「spline 三分量全差 → 嫌疑 biomeAt」（进展四）**——**假象**。08-08 DensityProbe comps（router 方法，可信）验证：**barrier/fluid/veinGap/continents/erosion/depth/ridges 在 -288 与 20000 全部 0 差异**。biomeAt 嫌疑排除。
3. **「负坐标 bug 普遍坐实（95.47%）、与 seed 无关」（进展十二）**——**部分假象**。-288 参照是 **FULL 状态**（19:39 导出，含 FEATURE 产物：coal_ore/结构/granite blob 等），C++ 不做 FEATURE → **大部分 diff 是参照状态假 diff**。真正 bug 区（SURFACE 参照下）：20000 正坐标 0.59%、8576 玩家区 3.31%——**正坐标超过一定值也块状**（非纯负坐标）。
4. **「差异集中在 y0-71（地表高度偏移，Perlin 微小差）」**——**方向吻合但原因错误**。真 bug 区 = 地表带 y42-65（SURFACE 参照），但**不是 Perlin 差**（b3d 一致），是 **finalDensity 树内 factor/sloped_cheese spline 组合的系统差 + range_choice 阈值跨越**（见下）。

### ✅ 08-08 确凿结论（新）

1. **cns 反射不可信**：`ChunkNoiseSampler.interpolators` 是 **8 个组件插值器**（finalDensity 树内标记噪声），get(0) min=-∞ 非 finalDensity；DensityInterpolator.sample 依赖 cns 遍历状态。**勿再以 cns 反射作密度参照**。
2. **InterpolatedDF 整树插值 = 正确语义**（chunk(-18,-16) 100% 实证）。「噪声插值+非线性后置」重构（interpTransform/CellInterpRef）**已实现并回滚**（全区域变差）——**勿再尝试**。
3. **OreVeinSampler 与 Java method_40547 逐行一致**（javap 确认）；**vein 先/aquifer 后 与 aquifer 先/vein 后 结果逐位相同**（顺序无关）。granite/diorite/tuff 缺失 = FEATURE（ore_granite 等 placed feature）**非 vein bug**。
4. **level-seed 坑**：`java/run/server.properties` 的 level-seed 硬编码 -8248，`-PbenchSeed=X` 只设 Java 属性——**跑其他 seed 必须改 level-seed**（08-08 曾因 8576 参照错位误判「seed 派生差异」）。
5. **20000 无插值 finalDensity 角点差 0.127**（-densityDump 修复后可信）：`-densityDump`（wg_sample_density 无插值）vs vanilla grid——**-288 角点（y≡0 mod 8）逐位一致**、**20000 角点差 0.127（y48）**——**无插值层面就差**（非 InterpolatedDF 插值问题）。
6. **根因链（最终收敛）**：C++ finalDensity 树内 **factor/sloped_cheese（spline 组合 + shift）与 vanilla 有系统差**（dfreg 参考：factor 差 1.6、sloped_cheese 差 11.6、offset(spline) 差 0.02 恒定——待 cache 可信确认）→ **20000 的差跨 range_choice 阈值（sloped_cheese 1.5625）→ finalDensity 角点差 0.127 → 浅层 y42-65 符号翻转 → 块状**；-288 的差被「range_choice 同侧分支」吸收 → 100%（为何之前查不出）。
   - ❌ **已更正（2026-08-08 晚 2）**：真正的根因是 **Cache2DDF 缓存 key 用错粒度**（C++ chunk 级 vs Java block 级）——factor/offset 等的 FlatCache grid 查表值差（factor 3.99 vs 4.61）→ finalDensity 角点差 → 块状。spline 类本身与 Java 逐位一致（factor/offset spline GRID 对比 0 差异）。已修复，见文末「2026-08-08 晚（2）」段。
7. **-namedDump 可信**（与 -nbDump 逐位一致）；**dfreg 不可信**（DENSITY_FUNCTION registry 原始树 ≠ 游戏实际——base_3d_noise 0.0145 vs 游戏 0.0596）；cache（actualDensityFunctionCache）是游戏实际（b3d 从这里拿过，可信）。

### 工具（08-08 就绪）

`WG_DBDEBUG`（列 densityBuf）、`WG_COMPDUMP`（router 组件）、`-densityDump`（主世界无插值 finalDensity）、`-namedDump`（可信 registry）、DensityProbe cache/dfreg/comps 扩展、OreProbe 参数化。参照状态：-288 FULL（只用于 density/vein 分析）、3200/20000/8576 SURFACE（方块对比用）。

## 2026-08-08 晚：spline 差定位进展（factor 3.99 vs Java -0.61，差 4.6）

### 已坐实（可信数据）
- **factor（spline）差 4.6**：C++ -namedDump 3.9932 vs cache（actualDensityFunctionCache 游戏实际）Spline 实例 -0.610364（@20008,0,20008）——**spline 真差**
- **depth 差 0.0278**（C++ 0.417451 vs Java 0.389636）：depth 引用 offset（spline）——**同 spline bug，不同量级**
- **sloped_cheese 差 7.2（y0）**：C++ 6.73 vs cache -0.467——组成 = 4×quarter_negative((depth + jaggedness×half_negative(noise_jagged)) × factor) + base_3d_noise——depth/jaggedness/b3d 已一致 → 差在 factor/noise_jagged

### 已排除（静态审查 + debug 实证，全部与 Java 1.20.1 一致）
1. **SplineDF.apply**（Catmull-Rom）：`lerp(kd,nv,ov) + kd(1-kd)lerp(kd, p, q)` 逐行一致（mc-src2 Spline.java 核对）
2. **二分**：`findRangeForLocation = MathHelper.binarySearch(0, len, i -> x < locations[i]) - 1`（动态 predicate）== C++ 二分
3. **sampleOutsideRange**：Java `f==0.0 ? value : value + f*(point-loc)` == C++（derivative=0 等价）
4. **locations/derivatives 解析**：factor 顶层 locs=[-0.19,-0.15,-0.1,0.03,0.06] 与 JSON 一致
5. **cache_2d（chunk 级缓存）**：key=(x>>4)<<32^(z>>4) 与 Java ChunkPos.toLong 一致；debug 实证 10 次 miss 是 chunk 边界（20016=chunk 1251）正常交替，非 bug
6. **FlatCache 网格/查表**：5×5 角点 (chunkX*4+i)*4、clamp 语义一致

### 剩余嫌疑（下一步）
- **subSplines 嵌套值**（factor 的嵌套 spline：erosion 10 点、ridges 2 点等）——需逐环节对比（f/kd/nv/ov）
  - ✅ **已排除（2026-08-08 晚 2）**：SplineDF 类与 Java 逐位一致（factor/offset spline GRID 25 角点对比 0 差异）；spline 不是 bug 根源（根因 = Cache2DDF，见下段）
- **f 的 float 精度**：Java Spline 的 locationFunction 是 ToFloatFunction（applyAsFloat 返回 float），C++ locationFunction->sample 返回 double——float vs double 差 1e-7 级，但 f 落 location 边界时可能跳区间（当前 f=-0.0091 远离边界，暂排除）
  - ✅ **已排除（2026-08-08 晚 2）**：GRID 对比 0 差异证明 float/double 精度不是问题（8576 剩余差另查 noise_jagged/cave 逻辑）

### 工具
WG_SPLINEDEBUG（SplineDF f/result/locations/locFn + Cache2DDF miss + FlatCacheDF grid dump）。

## 2026-08-08 晚（2）：块状 bug 主因修复——Cache2DDF 缓存 key（chunk 级 → block 级）

### 根因（最终）
**C++ Cache2DDF 的缓存 key 用 chunk 级** `(x>>4)<<32 ^ (z>>4)`；**Java 1.20.1 是 block 级** `ChunkPos.toLong(blockX, blockZ)`（javap 反编译确认：单槽 lastSamplingColumnPos，key 是 block 原值）。
- 影响：FlatCache 的 5×5 角点（同 chunk 不同 x,z）——Java 每个角点独立采样（block 级 key 不命中），C++ 被 chunk 级缓存**错误共享** → 25 个角点只采样少数几个 → **grid 值错** → factor/offset/erosion/ridges（都是 FlatCache[Cache2D[...]]）查表值差 → finalDensity 角点差 → 浅层符号翻转 → 块状
- **为何 -288 100%、20000 块状**：块循环（fillFromNoise y→z→x）同列连续采样，chunk 级与 block 级命中率都 100%（无差）；FlatCache buildGrid 才暴露差异——-288 恰好 grid 查表值同侧不翻转，20000 翻转
- 修复后块循环命中率不变（同列连续）→ **性能无损**

### 成效（block_probe 回归）
| 区域 | 修复前 | 修复后 |
|---|---|---|
| 20000 SURFACE | 99.4115% | **99.9850%**（角点密度 0.127→≤2e-6）|
| -288 FULL | 95.4728% | 95.7111%（剩余 = FULL 参照 FEATURE 假 diff）|
| 3200 NOISE | 100% | 100% |
| 8576 玩家区 | 96.69% | 98.67% |

### 剩余（8576 玩家区 1.33% = 47000 块地形差）
- **sloped_cheese 值差**（C++ 12.7 @8576 y-8 vs vanilla range_choice 分支不同）——range_choice 阈值 1.5625 附近分支选择差
- 组件（depth/factor/b3d/jaggedness/continents/erosion）修复后全一致；qn/hn 与 Java 一致（mc-src2 核对 `x>0 ? x : 0.25x`）；**唯一未定位：noise_jagged（xz_scale=1500）或 when_out_of_range 的 cave 逻辑**
- jagged 噪声 firstOctave=-16；20000 的 jagged@30012000 触发 maintainPrecision 折叠（C++ -0.023052 vs Java 疑似 +0.023028——**符号差待确认**）；8576 不折叠（C++ -0.1373）——待 Java 直接采样确认

### 工具（本轮新增）
WG_SPLINEDEBUG（spline f/result/locations/locFn + Cache2D miss + FlatCache grid）、block_probe -mismatch（差块明细）、got_export -noiseDump（wg_sample_noise 直接采样噪声）、DensityProbe cache GRID（Spline 25 角点）、buildSpline 构建期 dump。

## 2026-08-08 晚（3）：8576 剩余差收窄——组件/噪声全一致，收缩到组合层

### 已排除（全部逐位实证，8576 玩家区）
- nj（jagged noise，xz=1500）：C++ -0.1372547 vs Java cache -0.137255 ✅
- factor spline：C++ 5.183928740 vs Java 5.183929 ✅（此前「factor 差 0.07」是角点对比错位）
- cave_layer（xz=1,y=8）：C++ 0.070025/0.360630 vs Java 0.070025/0.360630 ✅
- cave_cheese（xz=1,y=0.667）：C++ 0.388347/0.258653 vs Java 0.388347/0.258653 ✅
- -1e6 误判排除：caves/pillars 的 when_in_range=-1e6 是 JSON 合法常量（非 bug）
- final_density 结构：argument1 = squeeze(0.64×interp(blend))（blend 内嵌 range_choice(input=sloped_cheese, min=-1e6, max=1.5625, in=min(sloped_cheese,5×entrances), out=cave 逻辑)）；argument2 = 1.25×interp(caves/noodle)；树内直接噪声只有 cave_layer/cave_cheese

### 剩余（下一步）
- when_out_of_range 的组合（min/max/add/clamp/square 嵌套）——C++ 输出 0.114@y-8 vs vanilla 推断 -0.00184（推断依赖假设，需 vanilla 直接值：cache 加 y-8 采样）
- caves/noodle 引用（argument2）——C++ y-56=0.763（64 是合法 when_out_of_range 常量），需 vanilla 对比
- y-8 的 InterpolatedDF 插值交互

## 2026-08-08 晚（4）：8576 差锁定 when_out_of_range 组合（轴顺序已排除）

- **轴顺序确认**：MC = 长高宽（X 长/Y 高/Z 宽）；NoisePos/索引/采样参数全部正确（20000 角点逐位一致是铁证）
- **final_density 结构**：min(squeeze(0.64×interp(blend)), caves/noodle 引用)；blend 内嵌 range_choice(input=sloped_cheese, when_out_of_range=cave 逻辑)
- **8576 y-8 差**：-densityDump 0.0128 vs vanilla -0.0023 = min(squeeze(0.64×rc), noodle) 差——反推 when_out_of_range C++≈0.04 vs vanilla≈-0.0072——**when_out_of_range（cave 逻辑组合）差**；noodle=64（-namedDump）可能一致
- **已排除**：cave_layer/cave_cheese 噪声（逐位一致）、nj、factor spline、全部组件
- **下一步**：逐个对比 when_out_of_range 的 caves 引用（pillars/spaghetti_2d/spaghetti_roughness_function）或拆 MIN 链

## 2026-08-08 晚（5）：8576 差 = when_out_of_range@y-8；「地下正常」观察分析

### cache 实锤（8576）
- **final_density range_choice 输出**：y-8 = **-0.00726**（vanilla）vs C++ when_out_of_range ≈ **0.04**——**差 0.047，y-8 特异地差**（y0/y48 一致 0.039/0.051）
- **cave_layer/cave_cheese 全 y（含 y-8）逐位一致**（cave_layer y-8: 0.161222/0.1612217；cave_cheese y-8: 0.267454/0.2674543）
- 差收缩到：**when_out_of_range 的 caves 引用（pillars/spaghetti_2d/spaghetti_roughness_function——interp 包装）或组合层（min/max/add/clamp）**
- C++ caves 引用值（y-8）：pillars -0.1066、spaghetti_2d 0.2874、spaghetti_roughness 0.0075（合理，待 vanilla 对比）

### 用户观察「地下（y<0）正常、地上异常」分析（2026-08-08 用户确认，透视 + 多位置）
- 8576 密度差分布：y<-28 ≤0.0018（不翻转→方块全石头→正常）；y-20..0 0.005-0.015（接近翻转）；y60-100 0.013-0.061（块状暴露区）
- **深层「正常」= 密度差小 + 石头掩盖**（方块对 density 符号不敏感），**非轴错**（轴顺序已逐项确认）
- when_out_of_range 的 cave 逻辑 y 范围 -60 起（y<-60 简单分支）——深层天然一致——与观察吻合
- **结论**：剩余差 = when_out_of_range（cave 逻辑）在浅层（y>-60）的组合/引用差，非结构性

### 下一步
C++ 加 debug 拆 when_out_of_range 的 MIN/MAX/ADD/CLAMP 每层 + caves 引用值（对比 vanilla 推断），或从 cache 的 Interpolated 实例拿 vanilla 的 spaghetti/pillars 值。

## 2026-08-08 晚（6）：when_out_of_range 差组成拆解（interp(blend)@y-8 确认差）

- InterpolatedDF debug（8576 @728,-8,-408）：interp(blend)@y-8 = **0.040084**（C++）vs cache #0（interp(blend) 角点，y-64=0.11719 证实）= **-0.00726**——差 0.047 确认
- 另一个 caves interp = **-0.192846** == cache #7（-0.19285）逐位一致（排除）
- when_out_of_range 差不在：cave_layer/cave_cheese（全 y 一致）、#7 interp（一致）——剩：**pillars/spaghetti_2d/spaghetti_roughness interp（cache #1-#6 待对应）或 min/max 选边差**
- cache Interpolated 实例 y-8 值：-0.28558/-0.04597/0.38682/-0.24612/-0.13824/-0.07872（#1-#6，待与 C++ caves interp 对应）

## 2026-08-08 晚（7）：when_out_of_range 差 = 组合/树结构层（叶子全一致）

- GRID debug（8576 @728,-8,-408）：when_out_of_range 的 4 个 caves interp 全部与 cache 逐位一致
  （-0.192846=#7、-0.078724=#6、-0.285579=#1、0.386824=#3）；interp(blend)=0.040084（=when_out_of_range C++）
- MIN 链：when_out_of_range = min(0.788729, 0.040084) → 0.040084；finalDensity = min(0.012826, noodle=64) = 0.012826
- **叶子全一致但组合差（0.040084 vs -0.00726）→ 树构建结构差嫌疑**：
  ① 常量折叠（add/mul+Constant → LinearOperation，Java 不折叠——sample 等价但 minValue/maxValue 缓存时机差）
  ② LazyRef 未填充时的 minValue/maxValue 缓存（-inf/+inf）影响 MIN/MAX 选边
  ③ when_out_of_range 的 min/max 嵌套层级解析
- cache #2/#4/#5（-0.04597/-0.24612/-0.13824）是 vanilla 侧 C++ 未采样的 interp（采样路径不同）
- **下一步**：打印 when_out_of_range 完整树（JSON）对比 C++ 构建；或加 MAX debug 查选边

## 2026-08-08 晚（8）：entrances 差——when_out_of_range 差根源

- **链条**：when_out_of_range@y-8 差 = max(min(min(add(4×square(cave_layer), clamp(0.27+cave_cheese)+clamp(1.5-0.64×sloped_cheese)), entrances), spaghetti_2d+roughness), pillars)——min 选 entrances
- **entrances@y-8：C++ 0.040084 vs vanilla -0.00726（差 0.047）**
- entrances = cache_once(min(arg1, arg2))：
  - arg1 = 0.37 + cave_entrance(xz=0.75,y=0.5) + ycg(-10,30,0.3→0) = 0.788729——cave_entrance 一致（0.1337285/0.133729）
  - **arg2 = add(spaghetti_roughness_function, clamp(add(max(weird1, weird2), add(-0.0765,...)))) 差**
- **已排除**：cave_entrance、spaghetti_3d_rarity（-0.1304857/-0.130486）、WeirdScaledSampler 实现（scale 阈值+公式逐行一致）、ycg、cave_layer/cave_cheese、4 个 caves interp
- **剩余嫌疑**：weird（spaghetti_3d_1/2 噪声，C++ 0.1076275/0.0261934 @728,-8,-408）或 spaghetti_roughness_function 或 clamp 的 add(-0.0765,...) 组合
- **瓶颈**：cache 无 spaghetti_3d_1/2 纯 Noise 实例（WeirdScaledSampler 内未单独缓存）——需 Java 直接采样 noise registry 或反推

## 2026-08-08 晚（9）：第二个根因修复——WeirdScaledSampler rarity 解析 bug

- **根因**：density_builder.h 的 rarity 判断 `"type2"`（漏下划线）vs JSON 的 "type_2"——**CAVES 的 weird_scaled_sampler 全部误判 TUNNELS**（scale 1.5 vs 1.0）
- **链条**：spaghetti_2d 的 weird（scale 错 1.5）→ weird 值差（0.3701 vs 0.0679）→ spaghetti_2d 差（0.2874 vs -0.014777）→ entrances 差 → when_out_of_range 差 → 8576 块状
- **修复**：`rarity == "type_2"`（Java 1.20.1 带下划线）
- **成效**：8576 98.67%→99.60%；**密度角点全部对齐（0 差）**；剩余 0.4% = InterpolatedDF 插值差（非角点，已知 POC 现象，y60 陡峭地形翻转）；-288/20000/3200 无回归
- **工具**：WEIRD/UNARY/CLAMP/YCG/NOISE/GRID 节点级 debug

## 2026-08-08 晚（3）：8576 剩余 0.4% 深挖——continents 差 → biome 边界翻转 → 表层 terracotta 差

### 根因链（8576 玩家区表层差）
C++ continents（0.031236）vs Java（0.028145）差 0.003 → biome 六维参数（continents 等）在边界位置微差 → **biome 判定翻转（savanna ↔ eroded_badlands）** → 恶地表层规则不触发/触发 → 表层 stone↔terracotta 差块。差块集中在 **z=5 列 + x=810-815**（biome 边界线）。

### 已验证（本轮决定性）
- **biome 六维参数全一致**（temperature/vegetation/continents/erosion/depth/ridges @(728,-408) 0 差异；@(800,-428) 有 0.0007-0.004 差——**continents 差 0.003 是主因**）
- **offset 噪声一致**（C++ -0.450812887 == Java -0.450812887 @(800,0,-428)，getOrCreateSampler 路径）
- **continentalness 噪声一致**（C++ 0.033976100 == Java 0.033976100 @(200,0,-107)）
- **Java 实际 shift_a/shift_b = -1.234233 / -0.157350**（cache 的 Cache2D[ShiftA/ShiftB] 实例）——**≠ C++ 修前的 -0.695174/0.096050**（×0.25×4 缩放）**也不等于 offset 噪声直接采样**（-0.4508@(800,0,-428)）——**shift_a 的 offsetNoise 来源未明（既非 getOrCreateSampler 也非特判恒 0）**
- ShiftDF 的 ×0.25×4 尝试去掉（Java 字节码 sample(blockX,0,blockZ) 无缩放）→ **-288 变差（95.71→95.62）→ 已回滚**；修后 20000 的 continents 也仍差（0.031746）
- 差块 C++ biome=eroded_badlands 与 Java 一致（biome 判定在多数差块一致——**表层规则差是「biome 边界少数的翻转」**）

### 待解之谜
Java 的 shift_a（-1.234233）的 offsetNoise 参数/派生：usesLegacyRandom=false（OFFSET 特判不触发）、getOrCreateSampler(OFFSET)=-0.4508（≠-1.234）、特判 NoiseParameters(0,0.0,[]) 恒 0（≠-1.234）——三个来源都不匹配。下一步：DensityProbe 反射 shift_a 的 offsetNoise 的 noise 字段（看实际 sampler 的参数/派生）。

### 工具
DensityProbe 扩展：[OFFSET-NOISE]/[CONT-NOISE]/[SHIFT_X]/[SHIFT_Z] 直接采样（getOrCreateSampler + registry 函数）；block_probe -mismatch 带 C++ biome 输出。

## 2026-08-08 晚（4）：shift_a 之谜破解——cns 的 shift 与 C++ 完全一致；新根因候选 estimateSurfaceHeight

### 决定性（反射 + 直接采样）
- **cns 的 Cache2D 的 shift_a/shift_b（delegate 直接采样 @(800,-428)）= -0.695174451 / 0.096050446**——**与 C++ 的 ShiftDF（×0.25×4）完全一致**（逐位）——**C++ ShiftDF 正确（回滚正确）**
- **cache 文件里的 -1.234233/-0.157350 是「另一个 Cache2D[ShiftA/B] 实例」**（DensityFunctionTypes$Cache2D——nc 的 router 的？）——**非 continents 实际用的**（continents 用 cns 的 ChunkNoiseSampler$Cache2D——-0.695）
- comps（DensityProbe 的 nc.router.continents()）是 **Wrapping(FLAT_CACHE).sample = 直接采样**（非查表）——**而游戏实际（cns 的）是查表**——comps 与 C++（查表）存在「实现差」（comps 的 0.0281 vs C++ 查表 0.0245——位置/实现差，非 bug）
- 差块位置 (805,-427)：continents 差仅 1.8e-4（查表值 vs 直接值）、temperature 差 0.005（spline 放大）——**biome 边界翻转的真正输入是 temperature/erosion 的微小差**

### 新根因候选：estimateSurfaceHeight 实现差
- **Java cns.estimateSurfaceHeight(x,z)**（javap）：`(x>>2)<<2` biome 格对齐 + 从顶向下扫描 `initialDensityWithoutJaggedness > 0.390625`（间隔 8）
- **C++ surface.h estimateSurfaceHeight()**：`lerp2((blockX&15)/16, (blockZ&15)/16, surfaceHeights4[4])`——**4 角插值**——**实现完全不同**（06 篇检查清单待办）
- surfaceHeights4（C++ 的 4 角）来源待查（fillOneChunk）——若 4 角值/插值公式与 Java 差 → steep 条件/表层判定差 → 表层 stone↔grass/terracotta 差
- 差块 (805,-32,-427) Java=red_terracotta（y=-32 深层）vs C++=stone——JSON 规则 y_above(74) 不覆盖 y=-32——**参照的深层 terracotta 来源待解**（可能与 estimateSurfaceHeight/stoneDepth 相关）

### 下一步
验证 estimateSurfaceHeight（C++ vs Java @(805,-427)）+ surfaceHeights4 来源（fillOneChunk）——对比 cns 的扫描实现。

## 2026-08-08 晚（5）：surface 规则条件链全验证——runDepth/stoneDepth/split/hashXYZ 全部一致

### 已确认一致（逐位/公式）
- **runDepth**：Java `sampleRunDepth = (int)(surface*2.75+3.0+split(x,0,z).nextDouble()*0.25)` == C++（公式+hashXYZ 一致）
- **aboveY**：Java `y + stoneDepthAbove >= anchor + runDepth*mult` == C++（C++ 用 ctx.surfaceDepth==Java runDepth 同值）
- **stoneDepth**：Java `i(stoneDepthAbove/Below) <= 1+offset+j(addSurfaceDepth?runDepth)+k` == C++（C++ 的 ctx.surfaceDepth==runDepth）
- **estimateSurfaceHeight（模拟扫描）**：Java `initialDensityWithoutJaggedness > 0.390625 从顶向下`（(805,-427)=64）——**C++ 的 sh4（aquifer 4 角插值）待对比**

### 参照的 y=-32 terracotta 带之谜（未解）
(805,-427) 列：地表 296（高原顶）、y=-32 单层 red + y=-27..-23 带 red + y=-16 red + y=-11..-10 red + y=-8..-4 white——**bandlands 带（连续同色段）**。但 JSON 规则（badlands 段在 STONE_DEPTH_FLOOR 内）**不覆盖 y=-32**（stoneDepthAbove=328 > 1 不满足 STONE_DEPTH_FLOOR）——**参照的深层 terracotta 带来源不明**（非 JSON surface_rule——可能假 diff 或另有机制）

### 8576 差分类
1. **terracotta 带差（1554 块）**：参照深层带来源不明
2. **表层 stone↔grass/dirt（~12500 块，savanna 为主）**：真正的主差——**候选：C++ surface 循环起点（heightmap+1=297）vs Java（cns 的 fill 起点——待查）** → stoneDepthAbove 差 → 表层规则判定差

### 下一步
javap cns.fillFromNoise 的 surface 阶段（起点：estimateSurfaceHeight 还是高度图）+ sh4（C++ aquifer）vs Java est 对比

## 2026-08-08 晚（6）：est 修复（扫描）+ 8576 根因收敛到 finalDensity 微差

### 修复（C++ surface.h）
**above_preliminary_surface 的 est 从「4 角插值（aquifer sh4）」改为「扫描」**（Java cns 语义：从顶向下 initial_density_without_jaggedness > 0.390625，间隔 8，列缓存）。20000/-288 无回归；8576 略降（99.60→99.576）——**est 修复正确但 8576 主根因在别处**。

### 8576 根因收敛（决定性）
- (742,-427) 大洞穴（参照 62..257 air）：C++ finalDensity(y=64)=0.006（solid）vs Java（air）——**finalDensity 微差（0.006 级）在密度边界翻转 → 洞穴/地形差 → est 差 → 表层差**
- C++ initial_density(y=64)=0.76（>0.390625→est=64）vs Java（洞穴处 <0.390625→est 更低）——**initial_density 差同源**
- 之前 compaction 记录：8576 修复后 density 仍差（25/96 点、max 0.0607@y60、4 符号相反）——**8576 的 finalDensity 差（0.06@y60）是主根因**（组件差未定位）

### 下一步
定位 8576 的 finalDensity 差（0.06@y60）的组件：comps @(742,-427)（cns 查表版）C++ vs Java。

## 2026-08-08 晚（7）：est 验证一致 + 洞穴底 dirt 之谜（8576 差分类收敛）

### est 决定性
- **Java initial_density_without_jaggedness（模拟）@(742,64,-427) = 0.818289、@(739,64) = 0.679866**——**C++（WG_SURFDUMP）0.76**（差 0.058——**查表 vs 直接实现差**）
- **Java est（扫描）= 64 == C++（修复后 64）**——**est 一致**（est 修复正确但非 8576 主差）
- cns 的 initialDensityWithoutJaggedness = router.initialDensityWithoutJaggedness()（apply(getActualDensityFunction) 后——**查表版**）——est 扫描（UnblendedNoisePos）用查表版

### 洞穴底 dirt 之谜（未解）
参照 (739,-427) 大洞穴（57-60 air）洞穴底 y=56 dirt（C++ stone）——但 est=64（56<64 不满足 above_preliminary）→ **JSON 规则树（bedrock/above_preliminary/deepslate 3 条）不覆盖 56** → 参照的洞穴底 dirt 来源矛盾（可能假 diff 或 Java 另有机制）

### 8576 差分类（收敛）
1. 洞穴底 dirt（stone↔dirt——~6289 块）——**est 之外**（参照 dirt 来源未明）
2. stone↔grass（2164 块）——表层 grass 覆盖差
3. stone↔terracotta 带（2950 块）——参照深层带（假 diff 候选）
4. finalDensity 微差（(742,64) C++ 0.006 vs Java air——0.006 级洞穴翻转）

### 结论
est 修复（Java 语义）正确保留（20000/-288 无回归）；8576 主差在 surface 规则（洞穴底/表层）与 finalDensity 微差——**参照导出状态需验证**（洞穴底 dirt 可能假 diff）

## 2026-08-08 晚（8）：spaghetti_2d 排除（旧 dll 假象）+ 8576 主差收敛到洞穴底 dirt（假 diff 候选）

### spaghetti_2d 排除（MAXDBG @(728,-8,-408)）
- **add(weird+thickness) = -0.014777、cube = -0.393684、max = -0.014777**——**与 vanilla -0.014777 完全一致**（之前的 0.287444 是旧 dll/推断错）
- weird（树内）= 0.0679（与 -namedDump 的 spaghetti_2d 噪声一致——**无双实例**）——**spaghetti_2d 正确排除**

### 8576 主差收敛
- est 修复（扫描）正确（Java 语义）但 8576 略降（99.60→99.576）——**差块重排（est 不是主差）**
- 主差 = **洞穴底 dirt（stone↔dirt ~6289 块）**：参照 (739,-427) 大洞穴底 y=56 dirt vs C++ stone——**但 est=64（56<64 不满足 above_preliminary）→ Java 的规则树也不该产 dirt → 参照 dirt 疑似假 diff**（但 20000 无此差——8576 洞穴密集）
- initial_density（C++ 0.76@(742,64) vs Java 模拟 0.818——查表 vs 直接实现差 0.058）；est（Java 模拟 64 == C++ 64）

### 待决
验证参照（BlockProbe SURFACE 导出）的洞穴底 dirt 是否假 diff（游戏实际 vs 参照）——若假 diff，8576 真差更小

## 2026-08-08 晚（9）：⚠️ 崩溃修复——CoreSwapPool 并发 run 竞争（32 视距崩溃根因）

### 用户崩溃报告（1.0.11-pre）
- 创建世界 32 视距 → 99% 崩溃 + 改视距进图崩溃（hs_err：EXCEPTION_ACCESS_VIOLATION 读地址 0）
- 栈：CppBridge.fillChunk → drainBatch → CppWorldgen.fillBlocks（JNI）→ worldgen.dll+0x1b930 → msvcp140.dll+0x12c10（读 0）——Worker-Main-16（MC worldgen 线程池）

### 根因（代码审查确认）
**CoreSwapPool::run 的共享成员 fn/totalTasks/doneCount/nextTask/taskQueue**——MC 的多个 Worker 线程**并发调 fillBlocks → wg_fill_blocks_multi → run**——并发 run 互相覆盖（A 的 run 尾 fn=nullptr 被 B 的 workers 读空 → 调用空 std::function → 读地址 0 崩溃）
- 之前测试（block_probe）单批 run 不触发；MC 32 视距多 Worker 并发 fillBlocks 触发

### 修复
run 开头加 `static std::mutex runMtx`（整个 run 串行化——内部线程池仍并行 fillOneChunk，性能影响小）
- 回归：3200 100% / 20000 99.985% 无回归
- 待打包新版本（含此修复）供用户测试

## 2026-08-08 晚（10）：⚠️ 新 BUG——16 格宽「地貌同构划线」异常（runClient 实测）

### 用户实测（1.0.12-pre，runClient）
- ✅ 之前的地形 BUG（块状断裂）**已消失**（est 修复生效）
- ❌ **新 BUG**：在「之前修复的地形分界差异的同样位置」，出现**地貌同构的划线异常**——石头/雪地/黏土的块分布出现**明显的线状过渡分区（约 12-16 格宽，用户目测未精确计数）**

### 初步嫌疑（16 格宽 = FlatCache 网格单元）
- 12-16 格 ≈ 3-4 个 biome 格（4×4 块/格）——**接近 FlatCache 5×5 网格的一个单元**（buildGrid 角点 → 每角点覆盖 4×4 格 = 16 块）；宽度未精确（12-16）
- 地貌（stone/snow/clay）= **biome 相关**（温度/降水）——划线 = **biome 采样差**（某个 FlatCache 网格角点值 C++ vs Java 差 → 该角点覆盖的 16 块区域 biome 差 → 带状划线）
- cache_2d 修复（块状）后残留——**FlatCache 网格值（buildGrid 角点）仍有差（特定位置）**
- 与「8576 的 finalDensity 微差（0.006@洞穴）」「continents 网格值差（0.003）」可能是同源（FlatCache 网格值）

### 下一步
定位 16 格宽划线位置的 biome 参数（C++ vs Java 的 FlatCache 网格角点值）——确认是否是 FlatCache 网格值差（特定位置）

---

## 2026-08-08 晚补充：8576 剩余差（99.58%→99.8473%）+ 用户崩溃（内存损坏）

### 8576 对齐提升链（block_probe 逐位）
- **heightmap 索引 x/z 交换修复**（ad81342）：buildSurface 遍历 heightmap[k*16+l] 应为 heightmap[l*16+k]（z*16+x）——-288 95.47→95.72%、8576 99.58→99.80%
- **above_preliminary_surface 语义**：Java 实测 est=64 的列 y58/y63/y64 都产 grass/terracotta → 语义 = `blockY + surfaceDepth + 4 >= est`（试过 >=est/ +1/+sd/+sd+4，+4 最佳 99.8473%）。3200 保持 100%、-288 不变
- **est 两版一致**：C++ nc 直接版 initial_density 与 Java cns 查表版在 (738,64) 都 = 0.574（est=64）——FlatCacheDF 直用崩（RAX=0 多线程）已回滚
- **terracotta 带 y57/58 错位 1 未解决**（lround 正确，floor 更差）——疑带数组差或 biome 差（参照 savanna 列有 terracotta=假 diff 疑点，需 Java 真实 biome 验证）

### 用户崩溃（仅 XMing_Glamorgan，1.0.11-pre→1.0.17 都崩）
- 已修：CoreSwapPool run fn 并发覆盖（1.0.12）、derivedSplitters 并发写（1.0.14）
- 1.0.15+：崩溃日志 handler（vectored exception + StackWalk64 + crash-coreswap-*.txt + dll sha256）
- 1.0.17 崩溃：RIP=堆地址 0x28F57AF5057（call 到堆执行=use-after-free/函数指针覆盖）；data[0x34000]=0x854800014F721D8B（异常值）；MEM-CHK 未报（写坏在 fillOneChunk 外或校验位不对）
- **0xEFE1 call [0x34001] 之谜**：.rdata 0x34000+1（奇数地址未对齐 call）——静态值垃圾——正常应 call memset——需 CE/dumpbin 确认运行时值
- 用户机器疑有内存/驱动问题（0x40010006 异常像被 patch）——但需先排除我们代码

### 工具/脚本（data/）
- read_col2.py（列方块）、read_biome2.py（biome）、pe_probe.py/dis_efe1_16.py/find_pat.py/iat_probe.py/parse_map.py（PE 分析）
- BlockProbe 参数是 benchOriginX（不是 blockProbeOriginX）；EstDiag 条件 wx==45 && wz==-27

### 工具/脚本（data/）
- read_col2.py（列方块）、read_biome2.py（biome）、pe_probe.py/dis_efe1_16.py/find_pat.py/iat_probe.py/parse_map.py（PE 分析）
- BlockProbe 参数是 benchOriginX（不是 blockProbeOriginX）；EstDiag 条件 wx==45 && wz==-27

## 2026-08-08 深夜：✅ BiomeAccess 8 邻域选点——8576 剩余差根因（99.8473%→99.8892%）
（本段覆盖旧条目「terracotta 带 y57/58 错位 1 未解决——疑带数组差或 biome 差（参照 savanna 列有 terracotta=假 diff 疑点）」：那个 savanna 列有 terracotta 的疑点**已解**——不是假 diff，是 C++ biome 判定缺 8 邻域选点）

### 根因（Java 源码逐层确认）
- 参照列 (805,-432)：biome 段（y=100 采样）=savanna，但列内有 terracotta 带（y=58-71，badlands 专属特征）→ 矛盾疑点
- Java 表面阶段 biome 判定真实链路：`NoiseChunkGenerator.buildSurface` → `region.getBiomeAccess()` → `BiomeAccess.getBiome(BlockPos)`
  - `ChunkRegion.biomeAccess = new BiomeAccess(this, hashSeed(seed))`，ChunkRegion 实现 `Storage.getBiomeForNoiseGen` → `world.getGeneratorStoredBiome` → `biomeSource.getBiome(x,y,z, noiseConfig.getMultiNoiseSampler())`（实时采样，非 chunk 存储）
  - **`BiomeAccess.getBiome(BlockPos)` 不是 floor 采样**：pos-2 → 8 邻域角点 (l,l+1)×(m,m+1)×(n,n+1) + seed 哈希扰动距离选最近（method_38106）
- C++ biomeAt 直接 `(x>>2)<<2` floor 采样 → 判错 biome（savanna）→ 不产 terracotta；Java 8 邻域在该处判 eroded_badlands
- Java 实测对照：SURFBIOME（8 邻域）@(804,64,-432)=**eroded_badlands** vs BIOME（floor）=savanna
- 参照列 terracotta y=58-71 ↔ Java 8 邻域判 eroded_badlands 的 y 区间；grass 地表 y=76-78 ↔ savanna —— 完全吻合

### Java 算法要点（已复刻进 C++）
- `BiomeAccess.hashSeed(seed)` = `Hashing.sha256().hashLong(seed).asLong()`（Guava：putLong 小端 8 字节 → SHA-256 → 取前 8 字节小端）
- `SeedMixer.mixSeed(seed, salt)` = `seed * (seed * 6364136223846793005L + 1442695040888963407L) + salt`（64 位无符号回绕）
- `method_38108(l)` = `(floorMod(l>>24, 1024)/1024 - 0.5) * 0.9`
- `method_38106(seed, q,r,s, d,e,f)`：6 次 mixSeed（seed→q→r→s→q→r→s）得 m → g=38108(m), m=mixSeed(m,seed), h=38108(m), m=mixSeed(m,seed), n=38108(m) → 距离 = (f+n)²+(e+h)²+(d+g)²
- `getBiome(pos)`：i=x-2, j=y-2, k=z-2 → l=i>>2, m=j>>2, n=k>>2 → d=(i&3)/4 等 → 8 邻域选最小距离角点 → `storage.getBiomeForNoiseGen(px,py,pz)` → 采样位置 = (px<<2, py<<2, pz<<2)（Java sample 内部 ×4）

### C++ 修复
- **biome.h**：新增 SHA-256（`biomeHashSeed`）、`mixSeed`、`biomeJitter`（method_38108）、`biomeCellDistance`（method_38106）、`biomePickCell`（8 邻域选点）
- **worldgen_api.cpp**：WorldgenHandle 加 `seed`/`biomeAccessSeed`；biomeAt 与 wg_sample_biome 先 `biomePickCell` 选点 → p=(px<<2,py<<2,pz<<2) → 6 维采样 → find
- **surface.h**：biomeAtCached 缓存 key 改 `biomeCellKey`（选点坐标 packed）——原 `(x>>2,y>>2,z>>2)` key 错误（同 4 格内不同 y 的 8 邻域选点不同，会错误复用）

### 验证
- C++ -biomeDump (805,64,-432)=eroded_badlands（原 savanna）与 Java SURFBIOME 一致；(805,56,-432)/(805,100,-432)=savanna 一致
- **8576 TOTAL：99.8473% → 99.8892%**（差 0.1527% → 0.1108%，修复约 27% 剩余差）

### 剩余差（未解，下一轮主线）
- chunk(50,-23) 99.59%、chunk(50,-22) 99.98%：C++ 在 y=56 产 brown_terracotta，Java 参照列 (804,-368) terracotta 带在 y=60-73——**带 y 偏移约 6**
- clay_bands_offset 采样值 C++/Java 一致（(804,0,-368) v4=1.984，JavaRound=CppLround=2，diff=0）→ 带偏移不是 offset 噪声或 round 差异，疑规则条件（stoneDepth/STONE_DEPTH_FLOOR 窗口）或带数组生成——待续

### 新增工具
- RouterProbe 加 8 邻域等价复刻输出 `SURFBIOME`（BiomeAccess 直接构造 + biomeSource.getBiome）与 floor 对照 `BIOME`；参数 routerY/routerYFrom/routerYTo/routerYStep（probe.count 会触发 NoiseProbe 分支，勿用）
- Java 源码提取：`E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\gen\surfacebuilder\`（MaterialRules/SurfaceBuilder/VanillaSurfaceRules）、`world\biome\source\`（BiomeAccess/SeedMixer）、`world\ChunkRegion.java`

## 2026-08-08 深夜（续）：✅ 关键差异确认（8 邻域已修 + nextDouble float 精度）+ -288 负坐标 bug 定位进展（根因未定）

### ✅ 已确定差异（Java 源码逐行 + 实测双确认）
1. **BiomeAccess 8 邻域选点缺失**（已修复，见上段「✅ BiomeAccess 8 邻域选点」）：8576 99.8473%→99.8892%；跨 seed/坐标验证：3200=100%、8576@200,200(64chunk)=99.9998%、-8248@20000,20000=99.9997%、-8248@134304,434416=99.9940%——**不是个别点碰巧，三维（含 Y 轴）逻辑全对**。Y 轴：depth 分量 = y_clamped_gradient(-64→1.5, 320→-1.5) 连续 y 函数 + 8 邻域 y 方向选点（j=y-2, m=j>>2, (j&3)/4），每层 y 重选。
2. **nextDouble float 精度差异**（xoroshiro.h 已修）：
   - Java `Xoroshiro128PlusPlusRandom.nextDouble() = next(53) * 1.110223E-16F`——**float 常量**（53 位值被舍入到 ~24 位）
   - C++ 原实现 `(next()>>11) * 1.1102230246251565E-16`（**double** 常量，53 位全保留）
   - 影响：PerlinNoiseSampler 的 originX/Y/Z（`nextDouble()*256`）差 ~5e-7，在 maintainPrecision 折叠边界（±3.3554432E7）可能被放大；实测 base_3d_noise 差 ~7e-6（微小但确定是差异，已改 float 对齐 Java）
3. **blocks.json 的 vanilla=1 = minecraft:stone**（不是 air！air=0）——此前误读 mismatch 的 vanilla=1 为 air，实际是 stone

### ✅ 已确认一致（排除项，避免重复排查）
- InterpolatedDF cell 大小：`verticalCellBlockCount = BiomeCoords.toBlock(size_vertical) = 2×4 = 8`，C++ CELL_Y=8 **正确**（不是 16）
- Java CellCache（cache_all_in_cell）缓存同 pos 同值，C++ 纯委托**等价**（无损）
- Java 1.20.1 PerlinNoiseSampler.sample **无 512 归一化**（1.18 前的旧版才有），C++ 直接 floorD 一致
- OctavePerlinNoiseSampler legacy 构造 random 消费顺序一致（firstPN + kx 循环 + skipCalls=262）
- Xoroshiro128PlusPlusRandom.nextInt(bound) = Lemire 乘法（`l*bound` 高 32 位 + 拒绝采样），C++ 逐行一致
- `XoroshiroRandom(seed)` 单参数构造已做 RandomSeed.createXoroshiroSeed（SHA-256 混合，random.h 46 行），与 Java 一致
- Java 1.20.1 InterpolatedNoiseSampler.sample 与 C++ InterpolatedNoiseDF 逐行一致（8 次 interp + 16 次 lower/upper + clampedLerp）

### -288 负坐标 bug 定位进展（现象确定，根因未定）
- **现象**：参照列 (-244,-256)：y=40-50 stone、51-57 water、**58 stone + 59-61 dirt（岛）**、62 water、63+ air；C++ 同列 y=51-62 **全 water**（岛缺失）
- **确定**：C++ finalDensity 插值后 @(-244,56,-256)=-0.053 vs Java cns 反射（DensityProbe 真实生成链）=-0.668——**C++ 偏正 ~0.6 → 挖洞不足 → stone 岛缺失 → 表面规则在错误位置产 gravel/terracotta**
- **注意基准**：DensityProbe .txt（router.finalDensity().sample 未插值）≠ cns（插值后）≠ RouterProbe b3d（独立构建，可能与真实不同）——对比必须同基准
- **未定位**：base_3d_noise @(244,58,256) C++=+0.0889 vs Java RouterProbe=+0.0384（差 0.05）、@(-244,58,-256) 差 0.23——但 RouterProbe b3d 是独立构建，需用 cns 真实链对比后才能定论；正坐标 3200=100% 说明主链正确，-288 是负坐标特有
- 下一步：cns 反射采真实 base_3d_noise 对比；或直接对比 C++/Java 的 sloped_cheese 分量

### 新增工具
- `tbands_test.cpp`（一次性）：clay_bands_offset + 原始噪声 + base_3d_noise 正负坐标对比；WG_B3DDUMP 环境变量 dump base_3d_noise 中间值（interp/lower/upper 各 octave）
- BlockProbe 加 ColDiag：chunk(50,-23) 列 (804,-368) y=50-80 表面后方块 dump（对比参照）
- RouterProbe：SURFBIOME（8 邻域复刻）/BIOME（floor 对照）；参数 routerY/routerYFrom/routerYTo/routerYStep

## 2026-08-08 深夜（终）：✅ above_preliminary_surface 公式修复（SurfaceCondC）——8576 99.8892%→99.9768%

### ✅ 根因（Java 源码铁证 + 子进程独立审查交叉确认）
**`surface.h:263` SurfaceCondC（above_preliminary_surface）公式错误**：
- Java（`MaterialRules.java:567-572` SurfacePredicate）= `blockY >= estimateSurfaceHeight()`
- `estimateSurfaceHeight()`（`MaterialRules.java:488-516`）= `floor(lerp2(4 角 est)) + runDepth - 8`
  - 4 角 est = chunk 4 角 `cns.estimateSurfaceHeight`（`BiomeCoords` 对齐 (x>>2)<<2，扫描 initialDensityWithoutJaggedness > 0.390625，步长 8）
  - lerp2 参数序：`lerp2((blockX&15)/16, (blockZ&15)/16, e00, e10, e01, e11)`（MathHelper.lerp2 = lerp(deltaY, lerp(deltaX,e00,e10), lerp(deltaX,e01,e11))）
  - Java runDepth = sampleRunDepth = C++ surfaceDepth
- C++ 旧公式 `blockY + surfaceDepth + 4 >= est` **完全不等价**（缺 4 角插值 + runDepth-8 项）
- **修复**：`blockY >= k + surfaceDepth - 8`，k = floor(lerp2 4 角 est)（C++ 已传 `surfaceHeights4` 但此前未用）

### ✅ 实测验证（block_probe 逐位，seed 8576 = run/server.properties level-seed）
| 区域 | 旧公式 | 新公式 |
|---|---|---|
| 8576（720,-432 6×6）| 99.8892% | **99.9768%**（chunk(50,-23) 99.59%→100%）|
| 3200（8576 世界重导）| 99.8814% | **99.9995%**（差 8 块）|

**3200 旧参照（-8248 世界）已被 8/8 重导覆盖**——server.properties `level-seed=8576294172403134396` 固定 8576，BlockProbe 重导的 blocks 文件都是 8576 世界；-288 参照（8/6 19:39）是 -8248 世界（C++ -8248 匹配 95.74%）。**⚠️ 教训：对照 block_probe 前必须确认参照文件实际 seed（`[BlockProbe] worldSeed=` 打印），不能只看文件名/header 的 benchSeed。**

### ✅ 已排除（本问题无关）
- terracottaBands 192 带数组：C++/Java 逐位一致（tbands_dump + RouterProbe TBANDS 对比）
- clay_bands_offset、sampleRunDepth、biome 判定（@804,56,-368=badlands、@804,64,-368=savanna）均一致
- est 值：C++ sh4（aquifer）与 Java cns 4 角在**同 seed** 下一致（8576 seed：48/56 系；-8248：32 系）——之前「est 不同」是 seed 混淆假象
- RouterProbe 修正：names/fns 数组错位 bug（initial_density 列实际打了 veinGap）已对齐

### 新增工具
- C++ block_probe：`-biomeDump`、`WG_SURFDUMP`（列剖面+est）、`WG_ESTDUMP`（sh4 4 角+k）、`WG_DENDUMP`（buildSurface 前列）、`WG_SURFTRACE`（逐列 q/vx/s/biome 轨迹）
- `tbands_dump.exe`：复刻 Java createTerracottaBands 导出 192 带（对比 RouterProbe TBANDS）
- RouterProbe：修正 fns/names 对齐 + continentalness/offset 噪声直接采样

---

## 2026-08-08（深夜终）：✅ -288 负坐标「bug」= 结构/FEATURE 假 diff（非 density bug）

### 排查链条（现象 → 猜测/排除 → 验证 → 发现）
**现象**：-288（seed -8248318472910187742，4×4）95.74%。参照列 (-244,-256)：y=40-50 stone、51-57 water、58 stone + 59-61 dirt（「岛」）、62 water；C++ 同列 y=58-61 全 water（「岛缺失」）。

**猜测 1：est 差** → ❌ 排除
- C++ WG_ESTDUMP 测 (-244,-256)/(-241,-253)/(-243,-254)=32
- Java RouterProbe ESH（router.initialDensityWithoutJaggedness 扫描）+ BlockProbe EstDiagN（cns 查表版 estimateSurfaceHeight）17 点全 32（含岛区）——**查表版=无插值版=C++ 版**

**猜测 2：分量差** → ❌ 排除
- C++ WG_SURFDUMP vs Java RouterProbe @(-244,58,-256)：barrier -0.305444/-0.305447、erosion 0.246871/0.246878、depth -0.076875 一致、fluid_level_floodedness 0.0191（RouterProbe 新增该分量）、continents -0.206056 一致

**猜测 3：finalDensity 角点差** → ❌ 排除
- C++/Java @(-244,56,-256) = -0.053461/-0.053463（8 倍数角点逐位一致）

**猜测 4：InterpolatedDF 插值差** → ❌ 排除
- C++ GRID 打印 interp0(-244,58,-256)=-0.233008、interp1=-0.237669 vs Java cns 链（DensityProbe cns.txt）interp0=-0.233015、interp1=-0.237671——**完全一致**
- C++ InterpolatedDF 实例 6 个 vs Java DensityInterpolator 8 个——**但 Java idx5-7 是 ore_vein 的**（OreVeinSampler 用，不在 finalDensity 树）——**finalDensity 树内 interpolated 数量一致（5 个）**

**猜测 5：Beardifier（结构密度修正）** → ❌ ~~排除~~ → **2026-08-09 推翻重开**：`DensityFunctionTypes.Beardifier.INSTANCE.sample()` = 恒 0.0 属实（源码 290-312 行），但 ChunkNoiseSampler L469-470 的 `getActualDensityFunctionImpl` 把 INSTANCE **替换为真实 `beardifying`（StructureWeightSampler）**——只看 INSTANCE 静态实现导致误判。verdict-04（2026-08-09）实测 (-244,-256) 真实 Beardifier 非零（峰值 +0.166@60、y=58 +0.092），**海底边界 6710 块根因 = C++ 缺失 Beardifier**（AQF-APPLY dCC = C++ finalDensity + Java Beardifier，8/8 点 ≤3e-6 闭环）

**猜测 6：aquifer 判定差（e 值）** → ❌ 排除（结构发现后不再需要）
- 全部分量/est/邻居一致 → e=0 → 两边都该判 water——矛盾 → 转向结构

**✅ 最终发现：island = ocean ruin 结构（STRUCTURE_STARTS 阶段）**
- 参照 y=58 层地图：x=-244..-241（4 格宽）× z=-256..-241（16 格长）**完全规则矩形 stone 柱 + dirt 顶**——自然地形不可能 4×16 完全对齐
- cold_ocean 的 ocean ruin 用普通 stone（warm 用 sandstone）——buildSurface 在结构 stone 上产 dirt（y=59-61）
- 结构在 NOISE 之后（STRUCTURE_STARTS → NOISE → SURFACE）——aquifer（NOISE）判 water 被结构 stone 覆盖
- 参照含 FEATURE/结构证据：copper_ore 564、iron_ore 465、oak_log 127、cobblestone 290、chest 2（chunk(-16,-13) 沉船）
- **C++ 只到 SURFACE 不做 STRUCTURE_STARTS/FEATURE → island 缺失 = 结构假 diff**

### 结论
- **-288 的 95.74% 差 = 结构（ocean ruin/沉船）+ 矿脉 + 树/草等 FEATURE 假 diff 为主**——C++ 的 density/surface 核心在负坐标已对齐（est/分量/角点/插值全一致）
- **8576/3200 的剩余差（0.0232%/0.0005%）同样可能是小结构/FEATURE**——8576 的 826 块待验证是否结构区
- **验证参照状态铁律**：BlockProbe 导出表面是 SURFACE（594 行 getChunk）但实际含 FEATURE/结构（连带推进）——对比前必须过滤 FEATURE/结构方块，或参照导出时禁 spawn 预生成（server.properties simulation-distance=2 + 删 world）

### 新增工具/证据（本 session）
- C++ `WG_AQFDUMP`（aquifer apply 邻居距离/e 值）、`[BUILD] InterpolatedDF instances`（构造计数）
- Java BlockProbe `EstDiagN`（cns 查表版 est + cns-ini 列）、`AQF-J`（blockStateSampler.sample 反射——注意 CellCache 缓存污染不可信）、DensityProbe `InterpDiag delegate`（8 个 interpolated 的 delegate 类型）、`[CellCache]`（真实遍历态 density——同样污染）
- **DensityProbe 导出状态**：`E:\PYTHON\MC\data\vanilla_density_overworld_c-16_-16_b12_0.txt`（无插值）、`_cns.txt`（游戏实际插值链 8 interpolators）、`_cache.txt`（actualDensityFunctionCache dump）
- **关键文件路径**：C++ `E:\PYTHON\MC\versions\1.20.1\cpp\worldgen\src\aquifer.h`（getFluidLevel/estimateSurfaceHeight）、`density.h`（InterpolatedDF/Cache2DDF/FlatCacheDF）、`density_builder.h`（buildNode）、Java `src\main\java\wg\bench\BlockProbe.java`（EstDiagN/AQF-J）、`DensityProbe.java`（InterpDiag/CellCache）、参照 `E:\PYTHON\MC\data\vanilla_-8248318472910187742_4_-288_-256.blocks`

---

## 2026-08-08 晚：8576 terracotta 带破案 + 3200 参照污染 + 框架流程首跑（状态 ✅）

**链条**（框架 Phase 0-3，scout/worker/judge 全部 subagent 隔离）：
1. est 一致排除（est=64，C++/Java 一致）→ noodle=64 排除 → sloped_cheese 1.5625 阈值（(808,-412) y64=1.5654 恰过线）→ squeeze 公式确认一致（Java DensityFunctionTypes 1161-1164 = C++ density.h:154，d/2-d³/24）
2. **Diag810**（BlockProbe 新增诊断）：NOISE 阶段 (810,76,-411)=air、y=74-120 全 air；SURFACE 阶段 y=69-118 terracotta 带——**矛盾在 buildSurface 不在密度**（此前所有密度口径 air 全部解释通）
3. **worker4 破案**：Java SurfaceBuilder.placeBadlandsPillar（L208-234）eroded_badlands 专属——2D 噪声算 pillar 顶 j，air→stone 填充 + heightmap 抬升 + 主循环起点变高 + badlands 段恒真 → terracottaBands 染色（surfacebuilder-analysis.md）
4. **worker5 实现** C++ placeBadlandsPillar（surface.h，@anchor PILLAR#001）；主会话修复编译参数（heightmap const → 局部变量 columnH）
5. **验证**：8576 99.9768%→**99.9993%**（820→24）；3200 干净参照 **99.9997%**（零退化）
6. **3200 参照污染**（worker6 诊断）：anilla_-8248318472910187742_4_3200_3208.blocks 8/8 00:02 被 8576 世界重导覆盖（level-seed 固定 8576）——89.89% 假象；已重导干净参照（worldSeed=-8248 核对）+ 污染备份 E:\tmp\vanilla_-8248_3200_POLLUTED.bak.blocks
7. **judge 审查**（core.judge subagent）：7 项通过、无阻塞；建议保持 candidate（用户可考虑授予 confirmed）、SteepCond 理论差异（零影响）、剩余 24 mismatch 立项、y=-32 噪声卡关闭
8. **框架流程首跑完成**：Phase 0 架构计划（.investigations/000-架构设计/）→ Phase 1 scout（squeeze 确认）→ Phase 2 worker1-6 → Phase 2.5 block_probe 双回归 → Phase 3 judge → 知识库更新

---

## 2026-08-08（深夜终 2）：✅ 24 块 mismatch 收尾——#23/#24 forest terracotta 破案（SearchTree 移植 3 版迭代）+ finalDensity 课题归类 + 20000 基线修正

### ✅ 24 块分类（8576 seed 8576294172403134396，720,-432 6×6）
- **finalDensity 边界翻转课题**（candidate 待立项）：深板岩/水边界 12 + 地表三连错位 9（=21 块）+ river 1（同机制）——根因假设 = 块级 finalDensity 边界翻转（插值精度差），与 20000 的 river/taiga 边界差同族
- **forest terracotta 2（#23/#24）**——本轮破案修复（biome 判定 tie-break，见下）

### ✅ #23/#24 根因：biome 判定平局 tie-break 差（C++ vs vanilla SearchTree）
- C++ 线性 `find` 用严格 `<` 取 entries 首个命中（→ forest）；vanilla `MultiNoiseUtil.SearchTree` 按树序遍历，**平局（等 cost）取 badlands** → 参照产 terracotta 带而 C++ 判 forest
- 🔍 排查中曾误判「湿度差 0.0054」——实为坐标错位（-337 vs -336/-340 探针语义不同，见数据采集教训）

### ✅ 修复：移植 MultiNoiseUtil.SearchTree（searchtree.h）——3 版迭代
- ❌ v1：**空指针崩溃**（crash-coreswap-20260808-*.txt 一连串）
- ❌ v2：**异常崩溃**（makeBranch throw）
- ✅ v3：根因 = **MSVC long 32 位（Windows LLP64）**：`long bestCost = INT64_MAX` 被截断为 -1 → `bestCost > cost` 恒 false → bestBatches 恒空 → makeBranch 抛异常 → 崩溃；**改 `long long`（64 位）后修复**
- 验证：(812,73,-337) forest→badlands ✓（与 Java SURFBIOME 一致）；门禁 scan_cpp_anchors.py invalid=0（searchtree.h 新增 @anchor.test SURFBIOME#003）

### ✅ 顺手对齐（judge 建议，与 tie-break 同批）
- aquifer.h：`-0.225`→`-0.225f`、`0.9`→`0.9f`（Java float 常量提升）；`fluidLevel != INT32_MAX`→`!= -32512`（Java field_35479 无效液面常量）
- surface.h：buildSurface heightmap 改可变副本 + pillar 写回（SteepCond 读 pillar 后高度，对齐 Java trackUpdate）

### ✅ y=-32 噪声卡关闭
- (805,-32,-427) 深层 terracotta = badlands terracottaBands 产物，biome 判定已随 8 邻域修复解决（当前匹配）——**噪声卡关闭**，与 #23/#24 同机制族（选点/tie-break），不再独立排查

### ✅ 回归（block_probe 逐位）
- 8576 99.9993%→**99.9994%**（24→22）；3200 **99.9997%** 零退化；20000 **99.9989%**；-288 **95.7376%**（结案基线，结构/FEATURE 假 diff 不动）
- **20000 过期基线修正**：8/7 深夜记录的 20000 99.9997% 已过时（当时非干净 HEAD）——git stash 实验确认 18 块差异在 8/8 HEAD 就存在，与 river/taiga 边界插值差同类 → **并入 21 块 finalDensity 课题，不新立方向**

### ✅ 数据采集教训（已写入 AGENTS.md 四·探针/参照数据采集核对铁律）
- **探针采样坐标语义三套**：RouterProbe `B`/`SURFBIOME` = floor 对齐 `(x>>2)<<2`（SURFBIOME 打印 bp 对齐坐标、判定输入原始 BlockPos）；C++ `-biomeDump`/`WG_BIOMEDUMP` = 8 邻域选点后 `(px<<2,py<<2,pz<<2)`；`WG_COMPDUMP` = 原始块坐标直采——**跨工具同点对比 MUST 先确认语义**
- **参照文件完整性**：2×2 导出曾混入范围外 chunk（chunk(65515,65515) int16 溢出坐标）——导出后查 header/范围/TOTAL
- **seed 三查**：改 server.properties level-seed 前备份 → 删 run/world 强重新生成 → 输出核对 #seed / [BlockProbe] worldSeed / blocks file seed

---

## 2026-08-08 晚：✅ spawn 预生成后 native 崩溃根因——AddVectoredExceptionHandler 干扰 JVM 硬件异常（VEH vs JVM 冲突）

> **结论已提炼** → docs/07 追加 3（worldgen 运行时的崩溃日志机制）+ knowledge/discovered/compiler-idioms.md 发现 #5（VEH 在 JVM 进程不可用）+ knowledge/discovered/build-tooling.md（gradle 三坑）。本条保留完整二分链。

### 症状
- `gradle runServer`（seed 8576，replace 模式 C++ 接管）→ spawn 预生成完成（Done）后 ~2 秒 native 崩溃
- 崩溃线程 = JVM "Server thread"；RIP 指向 JVM metadata；RAX 是 Java Object[] oop；后续 jvm.dll 连锁崩溃；栈被 0xDEADDEAF 覆盖
- 崩溃 handler（dll 的 VEH）打印：RIP=堆地址（每次不同）、rw=read 0x8/0x24、寄存器小数字（0x11AFA/0x76AC）、stack-window 返回地址区全垃圾

### 二分排查链（每条带状态标注）
1. ❌ **线程数**：CORESWAP_THREADS=1（C++ 内部线程池单线程）→ 仍崩 → **排除** C++ 线程池并发
2. ❌ **攒批**：-Dcpp.noBatch=1（单 chunk 直调 fillBlocks）→ 仍崩 → **排除** BATCH 攒批机制
3. ✅ **fillChunk 计数**：CppBridge 加日志 → **fillChunk 0 次调用**（spawn 预生成在 init 之前完成；mixin 拦截条件 CppBridge.enabled 此时 false）→ **排除** C++ 生成相关（崩溃与生成无关）
4. ✅ **wg_create 阶段**：分段日志 → 4 阶段（finalDensity/noiseSamplers/biomeSource/surfaceRule）全部 OK → **排除** wg_create 内部
5. ✅ **对照实验**：BenchMod `active = replace`（禁无条件 init C++）→ **不崩** → 崩溃与 wg_create/init 相关（收窄）
6. ✅ **二分 VEH**：注释 installCrashHandler → **不崩** → **根因 = AddVectoredExceptionHandler**

### 根因
- `AddVectoredExceptionHandler`（崩溃日志铁律的 VEH 实现）**干扰 JVM 的硬件异常处理**：JIT null-check、GC guard page、写屏障都是 SEH 异常，VEH 先于 SEH 执行（StackWalk64/打印重活）→ JVM 内存被破坏（Server thread 堆损坏：Java 对象字段变垃圾、metadata 被当代码执行、栈 poison 0xDEADDEAF）→ 崩溃
- **block_probe/got_export 独立进程不崩**（无 JVM 异常模式）；**用户机器 D:\MC 的 0x34001 崩溃 = 同根因**（1.0.17 客户端 = C++ 接管 + VEH）
- 崩溃日志铁律（全局崩溃捕获）与 JVM 进程**冲突**——VEH 捕获一切异常（含 JVM 预期异常）并做重活

### 修复（worldgen_api.cpp wg_create L292）
```cpp
// 独立进程装 VEH（block_probe/got_export 崩溃日志）；JVM 进程不装（jvm.dll 已加载检测）
if (!GetModuleHandleA("jvm.dll")) wg::installCrashHandler();
```
- JVM 侧崩溃由 JVM 自带 hs_err（含 native 栈 dll 偏移）兜底——仍满足「崩溃可定位」铁律
- 验证：修复后服务器稳定运行（>5 分钟无崩溃），8576/3200 回归零变化

### 过程中发现/顺带修复的次问题（一并记录）
1. ✅ **build.gradle dll 同步源错误**：processResources 的 `../cpp/build-msvc` 指向 MC 侧历史旧 cpp（非 CoreSwap）→ 打包旧 dll（1.0.2/1.0.6 同款坑复发）；修复：改 `E:/PYTHON/CoreSwap/versions/1.20.1/cpp/build-msvc/bin/worldgen.dll`
2. ✅ **processResources UP-TO-DATE 不重同步**：doFirst 的 copy 不算 task input，dll 更新后 gradle 判定 UP-TO-DATE 跳过 → 服务器加载旧 dll（sha 不匹配排查半天）；规避：手动 Copy resources 或 --rerun-tasks
3. ✅ **gradle daemon env 缓存**：$env:CORESWAP_THREADS 传给 gradle daemon 不重启不生效（fork 的 JVM 继承 daemon 启动时 env）→ 用 -P 属性（vmArg 映射）或重启 daemon
4. ✅ **gradle 8.13 -D 参数解析**：`gradle runServer -Dcpp.replace=1` 被拆成任务（`.replace=1 not found`）→ 用 build.gradle 的 -PcppReplace → vmArg 映射
5. ✅ **crash handler 增强**（本次加，保留）：module base 打印（崩溃 RVA 定位）、stack-window 打印（RSP±0x50 qword + 0xDEADDEAF poison 标记）、WG_FBLOG（fillBlocks 批次日志 env 开关）
6. ✅ **CppBridge 诊断增强**（保留）：-Dcpp.noBatch env 兜底 CORESWAP_NOBATCH

### 通用模式（已入 knowledge/）
- **VEH 在 JVM 进程（jvm.dll 已加载）不可用** → knowledge/discovered/compiler-idioms.md 发现 #5（检测 GetModuleHandleA("jvm.dll")）
- **gradle 三坑**（processResources doFirst copy 不算 input → UP-TO-DATE；daemon env 缓存；8.13 -D 拆任务） → knowledge/discovered/build-tooling.md 发现 #1-#3

---

## 2026-08-09：✅ -288 课题破案——C++ 核心无 bug，差异 = 范围外 FEATURE（含洞穴雕刻 carvers）

> **结论已提炼** → docs/07 追加 4（-288 破案 + FEATURE 范围决策）+ docs/05 更新（岩石替换 = 地形性 FEATURE）。本条保留调查链。

### 起因
- 用户实测：block_probe -288（seed -8248318472910187742，-288,-256 4×4）仍 95.7376%，质疑 8/8「结构/FEATURE 假 diff」结案（「唯一需要担心的反而是那个」）
- 8/8 结案依据仅单点排除（岛=ocean ruin）+ 参照 FEATURE 计数（≈1448 < 差异 67042）——从未系统量化

### 调查链（14 轮分析，产物 .investigations/-288-reopen/analysis-phase2..13）
1. ✅ **量化**（phase2）：67042 差异中 natural 82.2% / structure_feature 仅 7.9%——「结构为主」数据上不成立 → 结案质疑成立
2. ✅ **密度层排除**（phase3）：C++ 插值链 vs Java cns 游戏实际链逐位一致（≤4e-6）；y=36 差 0.23 = 无插值/插值基准错配；base_3d_noise 再排除
3. ✅ **aquifer 层**（phase4-5）：AQF-J null 判定不可信（CellCache 反射污染 L750 铁律）；同基准 density 一致
4. ✅ **NOISE-BLK 探针**（新增，BlockProbe chunk.getBlockState 直读）：Java NOISE 阶段 (-244,-256) y=58-61 stone（岛非结构）、(-278,-240) y=15-19 water——块级真相
5. ✅ **Beardifier**（phase6-8）：Java aquifer 输入含 StructureWeightSampler；含水层区域实测 [BEARD]=0（ocean_ruin 无 terrain_adaptation 不参与）；CellCache 假设推翻（逐块插值无损）
6. ✅ **noodle/caves 树**（phase10-13）：noodle 低频（firstOctave=-8 单 octave，phase11/12「高频」方向反了）；slopedCheese 3.1~5.1 >1.5625 → caves 树完整；C++ octave 累加正确（persist 归一化后高 octave 贡献小正常）
7. ✅ **AQF-APPLY 铁证**（DensityProbe 扩展）：cns 游戏同构遍历 + aquifer.apply 直接调用 (-278,12..23,-240) **全部判 solid**，density 与 C++ 逐位一致——Java aquifer 与 C++ 完全一致
8. ✅ **chunk status 铁证**：chunk(-18,-15) status=`minecraft:carvers`——含水层 water/air = **洞穴雕刻（CaveCarver）阶段**产物（挖洞 + 液面填水），非 aquifer

### 最终结论
- **C++ 核心（density/aquifer/surface/vein）全部正确，无 bug 需修复**
- -288 差异 = 岩石替换矿脉 49%（ore_granite/tuff/diorite/andesite）+ 洞穴雕刻 carvers 17% + 结构 3.6%（Beardifier 岛）+ 树草 ~1% + C++ surface 微小项 ~0.1%
- 8/8 结案**方向正确**（差异 = 范围外功能），机制描述补充完整；时间线 L670「base_3d_noise 差 0.05-0.23」= RouterProbe 独立构建假象（03 篇 L100 排除，本次再确认）
- 8576 21 块课题独立（22 块清单无 carvers 差异：深板岩/水边界 + 地表分层错位 + terracotta 带）

### FEATURE 范围决策（用户拍板）
- **只做地形性 FEATURE：carvers（洞穴雕刻）+ 岩石替换（granite/tuff/diorite/andesite）**——影响玩家可见地形；矿石/树草/结构暂缓
- **暂缓实施**（用户「不急着做」）——数据已就绪（worldgen/data 有 configured_carver/configured_feature/placed_feature），实施需 Phase 0 架构设计

### 方法沉淀（已入 07 篇）
- NOISE-BLK（NOISE 阶段 chunk.getBlockState 直读）= 块级真相权威来源（反射 CellCache 污染不可信）
- AQF-APPLY（cns 游戏同构遍历 + aquifer.apply 直接调用）= aquifer 判定权威验证
- chunk status 检查（noise/carvers/surface）防阶段误判

---

## 2026-08-09 -288 未闭合 ~23% 差异定位（Phase 1-3，draft）

### 课题
- judge 审查要求补齐 -288 差异构成缺口 ≈23%：海底边界（C++ water vs vanilla solid ≈6710）+ gravel（≈4900）+ 表面规则（≈2900）

### Phase 1 勘探（recode.scout，pipeline-map.md）
- Java surface 管线：ChunkStatus.SURFACE → NoiseChunkGenerator.buildSurface → SurfaceBuilder.buildSurface（MaterialRules 引擎）；海底高度本体 = density/aquifer（逐位一致）
- C++ surface.h 偏差点：P1 StoneDepthCond secondaryDepth 映射（Java (int)map(sec,-1,1,0,range) 不 clamp+截断 vs C++ floor(lerpClamp) clamp）；P2 HoleCond 字段错（-288 不触发）；P3 s 未找到；P4 isFluid

### Phase 2 归类量化（classify_m288.py / colview_m288.py）
- seabed 11135 = 含水层 stone→water 4416（carvers 已闭合）+ 海底边界 water→solid 6710（y=52-62）
- gravel 4881（深层 deepslate→gravel 1802 = ore_gravel FEATURE + 浅层 2881 surface rule）；surface_rules 4675（beach 1876）
- **关键判定：C++ 海底系统性低 4-10 格（非 ±1e-6 翻转）→ 独立机制，不并入 8576 21 块课题**

### Phase 3 机制定位（fan-out 3 worker + 主会话验证）
- **B1 Beardifier：推翻**——StructureWeightSampler 非零区 = structure bbox 外 11 格（±1 口径）+ y 基准 ±12（phase6/7「24 格」= TABLE 尺寸误读）；(-244,-256) 距村庄 32 > 11 → Beardifier=0
- **B2 aquifer pocket：推翻**——AquiferSampler L149 density>0→null 硬铁律；C++ 已完整实现形状场；含水层 = carvers（AQF-APPLY + chunk status 铁证）
- **B3 aquifer 液面/e 值：部分支持（机制成立）**——C++ trace e=0.0000（fl2.y==fl3.y==63 → j=0）→ density+e<0 → 判水；vanilla 浮岛实心只能由 density+e>0 翻转产生；(a) splitter 派生排除（Python 复现 8/8 o/p/q 逐位一致，verify_splitter2.txt）；(b) 液面网格输入值未闭合（需 Java 真实遍历中间量 dump 判别，AQF-J 反射污染不可信）
- P1 表面规则：beach RANGE_6 sandstone 层边界差（可解释块数待量化，非直接 2900）

### 状态
- ✅ 海底边界机制定位（e 翻转缺失）＝ candidate（judge 建议，机制级）
- 🔍 (b) 液面网格输入值判别（Java dump）= 下一步最高优先
- ⏸️ P1 修复（前置量化收益后再改）、carvers 后重测海底 gravel

### 方法沉淀
- splitter 派生链 Python 独立复现（md5/mixStafford13/hashXYZ/nextInt 拒绝采样/floorDiv）= 判别 C++ vs Java 随机派生差异的可复现手段（verify_splitter2.py 8/8 逐位一致）
- AQF-J 反射污染 → e 值判别必须 Java 真实遍历中间量 dump（禁反射）

---

## 2026-08-09 -288 未闭合差异定位 + 300515 判定 + 重归因（draft→candidate）

### 课题主线
- judge 审查要求补齐 -288 差异构成缺口 ≈23%（海底边界 6710 + gravel 4900 + 表面规则 2900）
- Phase 1-3（fan-out B1/B2/B3 + splitter 验证）：**海底边界定位 = aquifer e 值翻转缺失**（B3 部分支持），(a) splitter 派生 8/8 逐位一致排除，(b) 液面网格输入值待 Java dump

### 用户洞察（重大转向）
- 用户提出：「冰山在无陆地时也生成 = FEATURE 独立生成实心块」，质疑 (-244,-256) 岛是否 FEATURE 产物
- **决定性验证**：NOISE-BLK 铁证（status=noise 打印验证）(-244,-256) y=58-61 NOISE 阶段已 stone + Java cns 权威密度负 → **e 翻转（B3）成立，岛非 FEATURE**；AQF-J densFn +0.037 = CellCache 反射垃圾（phase5 L750 铁律）
- **重归因**：-288 的 67042 块 FULL 差异中 **FEATURE 占 74.2%**（岩石替换 33k + 矿石 3k + 村庄方块 + carvers 洞穴 6684 + 含水层 5051）；真核心 17251 块（e 翻转 ~7250 + surface 规则 ~9979）

### 300515 判定（用户提供新 seed）
- seed 3005152118058349760 + 坐标 (-1320400,-198049)：FULL 差异 **94.13%** = 全部范围外 FEATURE（陆地 flower_forest/plains，无 e 翻转）；SURFACE 状态 99.9986% 对齐 → **C++ 核心无 bug 强证据 + e 翻转确认为海洋/含水层专属**

### 参照状态审计（check_ref_status.py）
- **8576/3200 参照 = SURFACE 状态**（FEATURE 产物 1773/0）→ **21 块插值课题是纯核心差异，不混 FEATURE，无需重归因**（重要！）
- **-288/300515 参照 = FULL 状态**（混 FEATURE 74.2%/94%）→ 差异需按 NOISE 状态拆分

### 用户拍板（ask 工具）
- **FEATURE 实施范围 = 扩展：carvers + 岩石替换 + 装饰层（树草/矿石/团块）**——放弃原「只做 carvers+岩石替换、暂缓装饰层」
- 理由：-288 74.2% + 300515 94.1% 差异来自 FEATURE；含装饰层才能闭合 300515 实机差异

### 方法沉淀
- **参照状态三查**：blocks 参照导出后 MUST 检查 FEATURE 产物（岩石替换/ore/草/村庄方块）判定 SURFACE vs FULL 状态——不同状态差异构成完全不同（8576/3200=SURFACE 纯核心 vs -288/300515=FULL 混 FEATURE）
- **FEATURE 独立于地形**：冰山/村庄/紫晶洞在无 density 支撑处生成实心块（用户早期 bug 观察验证）——海底/陆地差异 MUST 先排除 FEATURE 方块再归因核心
- **NOISE-BLK 状态验证**：getChunk(x,z,NOISE,true) 请求后 MUST 立即打印 chunk.getStatus() 验证——主循环后续 getChunk(SURFACE/FULL) 会连带推进，NOISE 状态只在请求后立即读才可靠
- **SURFACE 参照导不出**：主循环 getChunk(SURFACE) 被连带推进到 FULL（stat 验证新参照仍含岩石替换）——SURFACE 状态参照不可用，NOISE-BLK 直读是唯一可靠的阶段隔离手段

### 待办（交下 session）
- FEATURE 实施（扩展范围：carvers + 岩石替换 + 装饰层）——Phase 0 架构设计先行
- 海底边界 e 翻转 (b) 判别（Java 真实遍历中间量 dump）——优先级下调（真核心仅 ~7250 块）
- 21 块插值课题（8576/3200 参照纯净，可继续）
- P1 surface secondaryDepth（terrain 互换 ~9979 块中部分）

### ⚠️ 知识库冲突裁决记录（2026-08-09 审计发现，交下 session）

**04 篇已 confirmed 结论**：「-288 岛区 e=0（fl2/fl3 液面全 63）→ 两侧判定一致——岛缺失不是 aquifer bug（是 ocean ruin 结构覆盖）」【04-aquifer.md L108】

**本次 session 验证发现的前提漏洞**：
- 「e=0 两侧一致」中 **Java 侧 e 值从未实测**——trace_aqf_1.txt 只有 C++ 的 e=0.0000；Java 侧 e 值（fl2.y/fl3.y 液面输入）是**假设**，非测量
- B3 (b) 子候选 =「Java 液面网格输入值 ≠ C++（如 fl3.y=-32512 无效液面 → j≠0 → e≠0 → density+e>0 翻转判 solid）」——**从未被 Java 真实遍历中间量 dump 直接验证**
- NOISE-BLK 铁证（status=noise 打印验证）：(-244,-256) y=58-61 **NOISE 阶段已是 stone**（FEATURE 之前）——与「Java aquifer 判 water」矛盾；若 Java 真判 water，stone 只能来自非 aquifer 的 NOISE 产物（oreVein 不可能形成 4 格厚岛）→ **矛盾未解**
- 04 篇「ocean ruin 结构覆盖」是 phase6/7 排除法推断（aquifer 无 bug → 岛另有来源），**未直接验证 structure start**

**裁决点（下 session 最高优先）**：Java 真实遍历内 dump (-244,55..62,-256) 的 o/p/q/d/fl2.y/fl3.y/fl4.y/e/g/h（DensityProbe 扩展，禁反射）——
- 若 Java e≠0（fl2.y≠fl3.y）→ e 翻转成立，04 篇「不是 aquifer bug」结论**推翻**（需修订）
- 若 Java e=0 且判 water → NOISE-BLK 的 stone 另有来源（需查 NOISE 阶段非 aquifer 产物）
- 若 Java e=0 且判 solid → density 输入差（反查插值链）
- **无论哪种，04 篇 L108 的「ocean ruin 结构覆盖」结论都需重新验证**（该结论无 structure start 直接证据）

### 2026-08-09 框架同步（e4e88c4）：fan-out 强制触发点（第三条）
- 发现驱动：CoreSwap -288 未闭合课题——B3 (a)/(b) 子候选主会话自推多轮（splitter 复现→液面链→est→r/s/t 点），用户两次提醒「派 worker」「启动 judge」；AGENTS.md 审计发现 fan-out 仅描述性文字无 MUST 触发规则（judge/scout 均有）
- RE-Framework 更新（commit e4e88c4，我们这边发现反馈后官方落地）：
  - spec §4.5 执行强制链扩为三条：judge / scout / **fan-out**（判定树分叉 ≥2 互斥候选 MUST 并行 fan-out，禁止主会话逐个自推；场景四：多疑点冲突/同一现象多机制/旧结论 vs 新证据/子假设再分叉；原则：不因候选小自推、自推成本 > 派 worker 隔离成本；自检提示：第二轮仍无定论自查已分叉）
  - core.fanout 触发条件升级（3 个以上 → ≥2 个互斥候选，MUST 语言 + 场景表 + 自检提示）
  - core.plan 轻量模板加 fan-out 预置节、重量模板加第 8 节
- CoreSwap 同步：install.py v2.0.0 重装（16 skills + 4 模块声明，framework.json source_commit=e4e88c4）；AGENTS.md 补自检提示；knowledge 发现 #7（上一条）
- ✅ 结案：fan-out 从「可选工具」升级为「强制触发点」，三触发点并列独立（scout 勘探→fan-out 分叉→judge 审查）

---

## 2026-08-09：✅ Beardifier 实现（StructureWeightSampler 结构密度修正）——-288 海底边界闭合 +10777 块（已结案）

> **结论已提炼** → docs/04 追加 3（海底边界结案）+ docs/06 追加 5（surface 级联）+ discovered 算法指纹 #5。本条保留完整链条。

### 起因
- verdict-04（2026-08-09）裁决：海底边界 ≈6710 块根因 = **C++ 缺失 Beardifier**（StructureWeightSampler 结构密度修正，NOISE 阶段 density 链 CellCache(add(finalDensity, Beardifier)) 缺项）——04 篇「ocean ruin 结构覆盖」归因推翻，B3「aquifer 液面链待修」撤销
- 用户拍板：**列入范围内待修**（结构相关但影响 NOISE 阶段 density 链）

### 架构设计（Java 喂数据方案）
- 结构布局（pieces/junctions）由 Java 侧 vanilla 机制构造：`NoiseChunkGenerator.populateNoise → doFill` 拦截处调 `StructureWeightSampler.createStructureWeightSampler(world, chunkPos)`——**C++ 不复刻结构生成器**，只收数据 + 移植纯算法
- `CppBridge.feedBeardifier`：vanilla 同源构造 + 反射提取 piece/junction → int[] → JNI `CppWorldgen.setBeardifier` → C++ per-chunk `beardifiers` map（key = chunkX<<32^chunkZ）；无输入则不加、行为不变（8576/3200 零退化保证）

### 算法移植（beardifier.h）
- **24³ float 权重表惰性预计算**：`(float)calculateStructureWeight = Math.pow(Math.E, -squaredMagnitude(x, y+0.5, z)/16)`——**Java 用 pow(Math.E,...) 非 exp**
- `getMagnitudeWeight` = clampedMap(magnitude(x, y/2, z), 0, 6, 1, 0)；`getStructureWeight` = 表查找 + `-d * fastInverseSqrt(e/2)/2`（d = yy+0.5）
- **fastInverseSqrt 位操作**：`l = 6910469410427058090LL - (l >> 1)` + Newton 一步（**MSVC long=32 位 → 必须 int64_t**）
- sample 四分支：NONE 跳过 / BURY getMagnitudeWeight / BEARD_THIN·BEARD_BOX getStructureWeight×0.8；junction 循环 getStructureWeight(r,l,m,l)×0.4

### 接入（per-chunk + JNI + mixin）
- worldgen_api.cpp：`wg_set_beardifier`（pieces 每 8 int：bbox6 + terrain + groundLevelDelta；junctions 每 3 int）+ fillOneChunk 3a 段 densityBuf = finalDensity + beard
- MC 工程（本地 M）：CppWorldgen.setBeardifier native 声明 + NoiseChunkGeneratorMixin.populateNoise 拦截处喂数据 + BlockProbe BEARD-DUMP 段

### 验证（全链）
- **算法对拍 17/17 逐位一致**：t_beard3（C++ Beardifier）vs BEARD-244 真实参照（beard244_run1.txt）(-244,50..66,-256) 全 17 点（含 y=50=0、y=58 翻转、y=60 峰值、y=63 翻负）
- **block_probe -288 闭合**：TOTAL 95.7379% → **96.4221%（+10777 块，MISMATCH 67039 → 56275）**；闭合点 86% 在海底边界 y=52..62（9280 块），与 verdict-04 预期 6710 吻合且超预期（村庄 12 格内其他传导差异也闭合）
- **零退化**：8576 99.9994% / 3200 99.9997%（无 beard 输入时行为不变）；scan_cpp_anchors invalid=0
- judge 审查 → **用户拍板 candidate**（2026-08-09）

### 坑（勿重蹈）
- **BEARD-DUMP 初版 cns null 静默跳过**：z=-13 时 chunk 连带推进导致 cns 生命周期问题 → dump 缺失且无报错；修复 = 不依赖 cns，直接 `createStructureWeightSampler(structureAccessor, pos)` 同源构造（BEARD-DUMP 与实机 CppBridge.feedBeardifier 同源）
- **t_beard2 臆造占位值误报**：测试脚本用臆造占位值 0 当参照 → 误报 y=50..54/64..66 MISMATCH（实为参照错）；用真实参照（beard244_run1.txt）重测 17/17 全过——**对拍参照必须来自真实导出，不能臆造**

### 产物
- verdict：`.investigations/-288-unclosed/beardifier-verdict.md`
- 结构布局参照：`.investigations/-288-unclosed/cmd-output/beard_m288.txt`（16 chunks：135 pieces + 506 junctions）
- 算法对拍：`.investigations/-288-unclosed/cmd-output/t_beard3_run.txt`
- block_probe：`.investigations/-288-unclosed/cmd-output/bp288_beard_run.txt`

---

## 2026-08-10 FEATURE 实施（Phase 1-5）

> 承接 2026-08-09 判定与拍板：-288/300515 的 FULL 差异 = **范围外 FEATURE**（岩石替换 33k + 矿石 3k + carvers 洞穴 6684 + 含水层 5051 = 74.2%；300515 = 94.13%）；用户扩展范围 = **carvers + 岩石替换 + 装饰层（树草/矿石/团块）**。
> 结论已提炼方向：docs/07 追加 FEATURE 章节 + discovered 模式（positions 链深度优先 / LCG 48 位 / 两阶段跨 chunk / biome 段跳过读 blocks）——待 Phase 6 收尾应用。本条保留完整推理链条。

### Phase 0/1：管线地图 + 基线 + RNG 对拍（🔍 → ✅）

- ✅ **管线地图**（pipeline-map.md，43KB 8 节）：ChunkStatus 链 `NOISE → SURFACE → CARVERS → FEATURES`；CARVERS 基类 = `ChunkRandom(CheckedRandom)`（48 位 LCG）、FEATURES 基类 = `ChunkRandom(Xoroshiro128PlusPlus)`——**两阶段随机基类不同（复刻最易错点）**；附 PlacedFeature 数据流 / PlacedFeatureIndexer / Feature 类优先级清单 / C++ 接入点
- ✅ **基线实测**（phase0/phase1 txt）：
  - 8576 SURFACE 99.9994%（nonAir 99.9986%）；3200 SURFACE 99.9997%——**零退化铁律基线**
  - -288 SURFACE（beard）96.4219%；300515 SURFACE 94.1326%（nonAir 81.8918%）——SURFACE 模式不跑 FEATURE 的起点
  - `-features` stub 空跑 = 与 SURFACE 逐位一致（管线接入不破坏 SURFACE 路径的预验证）
- ✅ **RNG 层逐位对拍**（chunkrandom_probe_run1.txt，Java ChunkRandomProbe）：CheckedRandom LCG 的 next/nextInt/nextLong、Xoroshiro 的 nextLong/nextFloat、ChunkRandom(Xoroshiro) 的 setPopulationSeed→setDecoratorSeed（step=0..1 × index=0..2 的 nextLong/nextInt(64)/nextFloat）、ChunkRandom(CheckedRandom) 的 setCarverSeed + shouldCarve——**全部逐位一致**，RNG 层先于机制层锁死

### Phase 2：CARVERS 引擎（LCG 根因）✅

**排查链（完整）**：carver 挖洞位置 **90% 不重合** → 排除种子/轨迹/mask/参照（曾误判「参照损坏」——biome 段未跳过，读取脚本 MUST 跳过每 chunk 后 256 个 biome 条目，否则后续 chunk 坐标错位）→ **根因 = CheckedRandom LCG**（shouldCarve 后的 nextFloat 序列错）→ 修复后 **69% 重合**。

- ❌ **先排除的假说**：种子公式（setCarverSeed 逐位对拍通过）、洞穴轨迹（mathSin/mathCos 65536 项 SINE_TABLE + MathHelper.sin 全程 float π）、CarvingMask 语义、参照完整性（修复 biome 段跳过后仍不重合）
- 🔍→✅ **根因（关键）**：`carveTunnels`/`carveRavine` 内部 Java `Random.create(seed)` = **CheckedRandom（48 位 LCG）**，不是 pipeline-map 勘探假设的 Xoroshiro！C++ 曾误用 XoroshiroRandom → 漂移序列全错 → **修复前挖洞重合仅 12%**（2042/16668）；修复（carver.h 内部 XoroshiroRandom → CheckedRandom）后：
  - 挖洞量匹配：C++ 17300 vs 参照洞 17573；重合 **11929（69%）**
  - -288 FULL 93.9442%（超过 SURFACE 93.4462%，carver 闭合 +0.5%）
  - **LCG 公式**：`seed = (seed*0x5DEECE66D+0xB) & ((1<<48)-1)`；nextInt 用高 32 位
- ✅ **配套修复**：BlockProbe 预生成 17×17 邻域（此前逐 chunk 生成 carver 静默跳过 → 参照才含 carver 产物）；carveCave 范围判断用 targetChunkX/Z；getState density=0.0 走液面链（carver 首次暴露该路径，已验证 d 逐位一致）
- ⚠️ **遗留（candidate 待续）**：剩余 **31% 挖洞差异**（挖多 5371 / 挖少 5644，对称，浅层 y=8-43）——carveRegion 边界微差或 mask 交互，非机制级；canyon 在 -288 区域无贡献（prob 0.01 低）
- ✅ 零退化：8576 99.9994% / 3200 99.9997%

### Phase 3：FEATURE 调度链（p=lastIndex + 深度优先 positions 链）✅

**排查链（完整）**：granite 位置 **3%** → p=lastIndex 修复无效 → **根因 = positions 链深度优先**（Java `stream.flatMap` 惰性消费随机，广度优先消费错序）→ granite **56%** → 跨 chunk 两阶段 → **88%**。

- ✅ **p（setDecoratorSeed 的 index）= PlacedFeatureIndexer.lastIndex**：`object2IntMap` 首个 entry index=0、后续递增；C++ `index[fid]=next++` 同构。**p=lastIndex 修复单独无效**（数字正确但仍错位）
- 🔍→✅ **根因（关键）**：Java `PlacedFeature.generate` 的 placement modifiers 是 `stream.flatMap` **惰性链**——每个 modifier 按前一级的输出位置逐个消费随机，等价**深度优先**遍历；C++ 原用 vector 逐 modifier 整批展开（广度优先）→ 随机消费顺序错 → 改深度优先 visit 递归后 **-288 91.24% → 96.67%**、granite 56.2%（Phase 3 落地数字：-288 FULL 96.67%、300515 FULL 96.59%）
- ⚠️ 剩余：跨 chunk 球体仍错位（granite 56% 封顶）→ 交 Phase 3.5

### Phase 3.5：跨 chunk 两阶段（FULL 首破 97.8%）✅

- 🔍→✅ **根因**：Java OreFeature 放**整个球体**（可跨 chunk 边界），C++ 单 chunk 内部生成只放本地部分 → 跨 chunk 球体缺失
- ✅ **两阶段 FEATURE**（`wg_fill_blocks_multi_phase`）：**phase 1** surface+carvers 并行 → 存 `regionCols`；**phase 2** features 串行 + `pendingCross` 跨 chunk 写（A 后生成覆盖 B = Java 语义）——granite 56% → **88.3%**（diorite 85.7% / tuff 87.8% / dirt 92.7%）
- ✅ 落地数字（phase35_crosschunk_result.txt）：**-288 FULL 97.8464%**（nonAir 93.65%）、**300515 FULL 98.0948%**（94.06%）

### Phase 4：简单装饰 + canyon/Heightmap 修复 ✅

- ✅ 实现：`DiskFeature` / `SpringFeature` / `FreezeTopLayerFeature` / `UnderwaterMagmaFeature`（CaveSurface 语义）+ `block_predicate_filter` + `surface_relative_threshold_filter` + IntProvider uniform value 嵌套修复
- ✅ 落地数字（phase4_result.txt，Phase 4 完成时中间快照）：-288 FULL **97.8390%**（Phase3 97.8464% → -0.007%，magma 位置错引入 ~20 块）、300515 FULL **98.0975%**（Phase3 98.0948% → +0.003%，disk/spring 正确放置）；8576/3200 SURFACE 零退化保持。**演进注**：97.8390% → 最终基线 97.8460%（+0.007% ≈ 110 块）来自 Phase 5 禁用 random_selector 树分支（worldgen_api.cpp JUDGE-DIAG 注释）
- ✅ **canyon 两处修复**：`carveRavine` 终点缩回 `l+1`、`carveTunnels` 分支递归传 `targetChunkX/Z`（否则分支洞穴落点错 chunk）
- ✅ **Heightmap 修复**：`HeightmapPlacementModifier` 返回 **top 不 +1**；`oceanFloorHeightmap` 构建时机移到 **carver 前**（Java NOISE 阶段语义，carver 后构建会导致特征偏一层）
- ✅ 其他：IntProvider `uniform` 的 min/max 在 `value` 子对象 → `count=uniform(44,52)` 修复（magma 0 → 43）；UnderwaterMagmaFeature 重写为 `CaveSurface.getFloorHeight` + `Box.stream` 语义（isValidPosition 全石头包围）
- ⚠️ **已知限制（记录）**：
  - **magma 位置与参照重合 0**：Java `BiomePlacementModifier` 过滤（cold_ocean）C++ 简化不过滤 + origin 依赖洞穴水位置（Phase 2 carver 31% 差异连锁）——影响 0.004%
  - disk `state_provider` 规则（sandstone 分支）未实现（简化 fallback）
  - FreezeTopLayer 用 OCEAN_FLOOR_WG 近似 MOTION_BLOCKING（-288 温度高无冻结，无影响）

### Phase 5：树草植被（失败 → 用户拍板不做）❌

- ❌ **树只放 40%**：oak_log C++ 114 vs 参照 273（参照含 y=-49..-39 地下 22 格 = 结构产物，地面树 ~251）；C++ 仅 y=73..91 地表——**canGenerate 失败多**（WG_TREELOG_ALL 诊断：origin 的 ground 检查 / 树干空间检查失败率高）；需 Java probe 对拍树 origin（count weighted_list → in_square → surface_water_depth_filter → heightmap OCEAN_FLOOR → biome 随机消费序列）未完成
- ❌ **300515 花爆炸**：dandelion C++ 533 vs 参照 11（树未实现 → 树冠区被当 air 放花——**树是花爆炸根因**）；noise_provider 简化 states[0]
- ⚠️ 禁用收场：feature_loader.h `generateOther` 对 flower/random_patch/simple_block/tree `return false`；worldgen_api.cpp random_selector `return false`——**禁用后基线实测**（见下节，树花不影响基线）

### 2026-08-10 深夜：用户拍板 + 收拾烂摊子（已结案）

- ✅ **用户拍板：树花植被不做**（2026-08-10 拍板 → 当晚重申）：
  1. **细节版本改动太多**——树/花/草植被在 MC 版本间差异大（1.20 → 1.21 大量变动），逐位对齐成本不可接受
  2. **MOD 特别容易碰到的位置**——实机 Mod 装饰主要挂 FEATURES 阶段，C++ 全接管会丢 Mod 花/草/树，兼容工作量不可接受
- ✅ **代码迁移 `deprecated-vegetation/`**（`versions/1.20.1/cpp/worldgen/deprecated-vegetation/`，2026-08-10 深夜从 feature.h 剪出）：SimpleBlockFeature / RandomPatchFeature / TreeFeature（straight+blob，fancy_oak 简化）/ RandomSelectorFeature 归档；**不参与编译、不接入调度**；恢复需重新接入 feature_loader.h 分发 + worldgen_api.cpp 调度 + placement.h 植被 modifier，并重跑 Java 对拍（git 历史 c04768e 前的 feature.h 有完整版本）
- ✅ **git 已提交状态（交接「未提交」已过时）**：NEXT_SESSION.md（2026-08-10 晚）记「git 提交本 session 进度（未提交！）」——已过时；实际 `c04768e` = **`feat(feature): implement CARVERS + FEATURES pipeline (phases 1-4)`** 已提交（author 固定 unknowbug，2026-08-10 深夜入 git，含 CARVERS + FEATURE 引擎 + 两阶段 + 装饰层；HEAD 当前仍停在 c04768e，2026-08-10 深夜植被迁移为 c04768e 之后的工作区修改，未提交，交下 session）
- ✅ **禁用后基线实测确认**（deprecated-vegetation/README，2026-08-10 深夜重跑）：8576 SURFACE **99.9994%** / 3200 SURFACE **99.9997%** / -288 FULL **97.8460%** / 300515 FULL **98.0975%**——树花不影响基线（参照的树/花方块 = 已知预期差异）；对照 NEXT_SESSION 最终基线表：-288 FULL 97.8460%（+1.42% vs SURFACE 96.4219%）、300515 FULL 98.0975%（+3.96% vs SURFACE 94.1326%）
- ⚠️ **MC 工程烂摊子**：`ChunkRandomProbe.java` **15 行单行格式已被破坏（-replace 事故）**——GRANITE/UNDERWATER_MAGMA 段仍可用（L5 用字面量 seed 能编译）；**勿再改格式**（交接待办 4 保留）；BlockProbe FULL 参照导出能力已提交

### 产物清单（新文件，2026-08-10 session）

- `chunkrandom.h`：CheckedRandom（48 位 LCG）+ ChunkRandom 双基类分发（Java BaseRandom nextLong 有符号拼接 MC-239059）+ setPopulationSeed/setDecoratorSeed/setCarverSeed；`chunkrandom_test.cpp` **33 断言逐位过**
- `carver.h`：CarverConfig/CaveCarverConfig/RavineCarverConfig + CarvingMask + CarverContext + CaveCarver（carveTunnels 分支树 / carveCave）+ RavineCarver（carveRavine / canyon 水平拉伸）+ FloatProvider/HeightProvider/IntProvider/YOffset 解析
- `feature.h`：OreFeature/ScatteredOreFeature + RandomPatchFeature + SimpleBlockFeature + DiskFeature + SpringFeature + FreezeTopLayerFeature + UnderwaterMagmaFeature + TreeFeature + RandomSelectorFeature（植被部分后迁移 deprecated-vegetation/）
- `feature_loader.h`：ConfiguredFeature.parse 分发（type → config）
- `placement.h`：PlacedFeature.generate（**深度优先 positions 链**）+ CountPlacementModifier（weighted_list/clamped）+ RarityFilter + Square + HeightRange + Heightmap + Biome + block_predicate_filter + surface_relative_threshold_filter + random_offset + noise_based_count
- `worldgen_api.cpp`：applyCarversAndFeatures + getPlacedFeature/getConfiguredFeature 懒加载 + **两阶段 FEATURE**（regionCols + pendingCross 跨 chunk）+ OceanFeatureContext
- 诊断 env：WG_FEATURELOG / WG_CARVERLOG / WG_CARVE_TRACE / WG_TREELOG / WG_TREELOG_ALL / WG_CARVER_SKIP / WG_CARVERDUMP

### 验证方法与方法沉淀（交 Phase 6 应用）

- 验证方法：C++ 独立测试（granite_cpp3.exe 球体对拍 3274 块、chunkrandom_test.exe 33 断言）+ Java ChunkRandomProbe（GRANITE/UNDERWATER_MAGMA 段逐位参照）+ block_probe `-save` 输出脚本对比（cmp_ore2/cmp_p4b/cmp_p5 在 E:\tmp）
- **待沉淀模式**（docs/07 + discovered）：① positions 链深度优先（Java stream.flatMap 惰性 = DFS，BFS 消费错序）② CheckedRandom LCG 48 位（carveTunnels/carveRavine 内部 Random.create = LCG，**非** pipeline-map 假设的 Xoroshiro——勘探假设被实测推翻）③ 两阶段跨 chunk（regionCols + pendingCross，A 后生成覆盖 B）④ 参照 blocks 的 biome 段跳过（每 chunk 后 256 个 biome 条目）
## 2026-08-11 性能回归调查 + Java 桥并发重写 + C++ 池改造（🔍 性能根因未结案）

> 承接 2026-08-10 深夜拍板（树花植被不做）后，用户实机发现**性能反降**：`-PcppReplace=1` 传送后区块生成卡很久才出现，纯 vanilla（`-PcppDisable=off`）对照确认——启动 perf-rework 调查（`.investigations/perf-rework/`）。
> 结论已提炼方向：requirements-doc.md（confirmed）+ static-audit.md（Java 桥并发静态审查）+ architecture.md（Phase 0 架构设计）+ 07 篇性能章节修正草稿（subagent 产出，待应用）+ discovered 模式 #10（thread_local 缓存冲突指纹）。本条保留完整推理链。

### 起因：实机性能反降（🔍 → ✅ 定位 Java 桥并发层）

- 🔍 **现象**：`-PcppReplace=1` 传送后区块卡很久才出现；纯 vanilla 对照确认 C++ 接管反而更慢（需求文档背景，2026-08-11）。
- ✅ **静态审查定位**（static-audit.md，审查对象 `CppBridge.java` 1.20.1-1.0.18；审查时 git 快照 MC HEAD=`78b615b` / CoreSwap HEAD=`0b92c62`，行号对审查时工作区 362 行）：
  - **P0-1**：JNI `fillBlocks` 被 `synchronized(BATCH_LOCK)` 全局锁串行化（noBatch L158 / 攒批 L182-197 / drainBatch L202）——对 JNI 多线程语义的认知错误（JNI 允许 native 被任意多线程并发调用，线程安全由 native 负责；C++ `wg_fill_blocks_multi` 设计即多线程）。
  - **P0-2**：writeChunk 锁内串行写 16 chunk（drainBatch L228-242 for 循环全程锁内）——157 万次 setBlockState 串行 + 阻塞攒批线程。
  - **P1-1**：攒不满 BATCH=16 时 `BATCH_LOCK.wait(2ms)`（L188-195）——低并发每 chunk 固定 +2ms，「区块卡很久」的直接体感来源。
  - **P1-2**：BATCH_BUFS 共享复用池（静态 `int[BATCH][98304]` ≈ 384KB/chunk）强制锁（L250-254）。
  - **P2-1**：writeChunk 每 chunk `new BlockState[4096]`（L260）——进程级静态可消除。
  - **P2-2**：feedBeardifier 每 chunk 全反射（15 次 Method.invoke）——P2-2 后续，不进本次范围。
- ✅ **runMtx 实证（Judge 第 2 轮 C1，worldgen_api.cpp L954-976）**：`CoreSwapPool::run` 内置 `static std::mutex runMtx` 锁住整个 run 生命周期（共享成员 fn/totalTasks/doneCount/nextTask/taskQueue 被并发 run 覆盖 → 读空 `std::function` 崩溃，**32 视距崩溃根因修复**）——即「批内并行（CoreSwapPool 多线程）、批间串行（runMtx）」；「C++ 耗时随线程数伸缩」在改造前不可达。
- ✅ **三层串行化定性**（architecture.md）：① Java BATCH_LOCK ② C++ runMtx ③ writeChunk 锁内循环。

### Java 桥去锁重写 + C++ 池改造（✅ 已实施，RQ-001~005）

- ✅ **目标架构**：去锁、M=1 非空即处理——每 worker 独立 thread-local buffer → JNI fillBlocks(1 chunk, buf) → 无锁 writeChunk 自己的 chunk；BATCH 攒批整个删除（用户拍板 M=1）；靠池并行摊薄 JNI 往返。
- ✅ **C++ CoreSwapPool 任务队列模型**：`run(count, f)` 提交 `{fn, shared_ptr<RunState>}` 到共享队列；RunState = `{atomic done, total, mtx, cvDone}`（per-run）；worker 循环取任务执行；调用方等自己 run 的 cvDone，不阻塞其他 run；删 runMtx。签名/对齐输出不变（`wg_fill_blocks_multi` 对 Java 透明）。风险：多 run 并发 = 池任务超订，操作系统调度兜底（用户拍板「崩了再说」测试策略）。
- ✅ **Java 侧改动（CppBridge.java）**：删 BATCH_LOCK/PENDING/BATCH_BUFS/drainBatch/wait；thread-local buffer（RQ-004）；stateById 进程级静态（RQ-005）；writeChunk 天然无锁（RQ-002）；noBatch 诊断路径保留为唯一路径（RQ-003）；feedBeardifier 不动。
- ✅ **随机种子对拍零退化**（random-seed-sampling.md，2026-08-11 改造后验证）：
  - `-8248318472910187742` 134304,434416 4×4 = TOTAL **99.9992%**（13 块差异）
  - `8576294172403134396` 200,200 8×8 = TOTAL **99.9997%**（22 块差异）
  - 与 2026-08-10 基线（99.9994%/99.9997%）同量级，差异均为既有插值课题类，**非本次引入**。只统计留知识，不修复（客户拍板）。

### 🔍 性能回归根因：FlatCache/Cache2D thread_local 缓存失效（未修，待立项）

- 🔍 **2026-08-11 吞吐实测（SURFACE 模式）**：单线程 **98-182ms/chunk**、多线程（8/22 线程）**108-239ms/chunk**——**无加速反降**；07 篇旧基线记录串行 28.1ms/chunk、并行 49.4ms/16chunk（3.1ms/chunk）。退化 ~3.5-6.5×（单线程）且并行不随线程数伸缩。
- 🔍 **WG_PROFILE 实测（density 阶段 670-1000ms/chunk，旧 8.5-11.7ms）**：
  - spline 单次 **20,598ns**（旧 992ns，~21×）
  - spline.sample **338 万次**
  - FlatCache rebuild **438,092 次 ≈ spline 调用数**——每次 spline 采样都重建 5×5 网格（缓存命中率≈0）
  - Cache2D miss **458,281 次**
- 🔍 **对照实验（排除本次改造引入）**：stash 本次改动后 HEAD 版 block_probe 8×8 仍 **10.2s**；连 07 篇基线提交 **86e4057** 也要 **8s** → **回归在 8/6 优化链之后积累，非本次改造引入**（本次改造保持对齐 8576 99.9994%/3200 99.9997%，未恶化吞吐；吞吐退化是独立预存问题，具体引入提交待 git 二分）。
- 🔍 **疑似根因（candidate 待验证）**：FlatCache/Cache2D 的 per-instance **thread_local** 缓存与「每 chunk 跨线程」执行模型冲突——多线程并行时每线程独立缓存 → 每 chunk 跨线程迁移 → 命中率归零、每 chunk 重建多次；叠加 buildGrid **嵌套采样递归**（边界点 x=cx*16+16 命中本 chunk 网格 k=4 才不重建，失配时触发相邻 chunk 网格重建递归）→ density 阶段 ~100 倍级恶化。
- 🔍 **待修状态**：根因修复未验证。候选方向：缓存按 chunk 键索引 / 按调用上下文显式传入 / 恢复线程亲和；需 git 二分定位 8/6 后引入提交。**未结案**。

### 决策：优化转向（已结案，2026-08-11 用户拍板）

- ✅ **放弃噪声 100% 对齐目标，转向优化优先**：有损容忍度 = **宏观一致**（地形/洞穴大体一致、允许方块级差异，肉眼基本看不出；用户实测地下也几乎看不出差异）。
- ✅ **300515 种子差异 = 非本项目问题**（BK-003）：参照含废弃前脏数据（花爆炸/树失败为废弃前实测），用户实测 vanilla 对照确认，不追责。
- ✅ **性能验收 = 体感**（BK-002）：游戏内「传送后区块出现时间」不采量化基线，验收凭用户体感。
- ✅ **RQ-006（C++ 有损加速，如 base_3d_noise 网格插值缓存）**：仅评估+用户逐项拍板后实施，不默认开（边界内待议）。


## 2026-08-12：性能回归根因定论（H1/H2/H3 假设验证 + judge 通过 + 用户拍板）（✅ 根因定论 / 🔍 修复中 Phase 2）

> 承接 2026-08-11 条目（性能回归根因 candidate 未结案）。2026-08-12 主会话采集新数据（wgprofile_t1/mt + splinedebug 537MB，36 chunks 6×6，seed 8576294172403134396），H1/H2/H3 假设全部验证，根因定论过 judge 审查并经用户拍板确认。完整分析落盘 `.investigations/perf-rework/root-cause-draft.md`（analysis, candidate）+ `review-rootcause.md`（review, candidate），已登记 `.artifacts/index.yaml`。本条保留验证链与定论过程。

### 数据采集（2026-08-12 主会话，勿重复实验）

- 命令：`block_probe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks`（36 chunks 6×6）；MSVC 强制重编，TOTAL 99.9994% 对齐保持（纯性能问题，无功能退化）。
- 原始输出：`cmd-output/wgprofile_8576_t1.txt`、`wgprofile_8576_mt.txt`、`splinedebug_8576_t1.txt`（537MB）；摘要 `cmd-output/collect-summary.md`。

### ✅ 假设验证（三组独立计数器数字闭环）

- **H1（y 主序 → Cache2DDF 单槽 100% miss）：部分成立（非主因）**。y 主序循环属实（worldgen_api.cpp L669-672 `for by{for bz{for bx}}`）且与 density.h L630 注释「同列连续 384 次采样」矛盾；但 splinedebug 全部 SPLINE/CACHE2D 行 **y=0**（grep `pos=(x,非0,z)` 零匹配）→ spline 只在 buildGrid 角点被采样，块级 densityBuf 98,304 次采样被 InterpolatedDF 插值 + FlatCache 查表挡掉（0 次 spline）→ 对爆炸贡献 ≈ 0。改循环顺序无效且不推荐（aquifer 同序读取有对齐风险）。
- **H2（FlatCacheDF 单槽 + buildGrid 嵌套递归 → 邻居网格重建）：成立（主因）**。density.h L735 `p.x=(chunkX*4+i)*4`，i=4 → `(chunkX+1)*16` = **下一 chunk 首列** → 嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（L687 key=(x>>4,z>>4)）→ 单槽被污染 → 重建邻居网格 → **递归蔓延 112 chunk**（36 生成 + 76 邻居，含左下对角 (44,-28)）。**rebuild 36,252 = 每 chunk ~1007 vs 期望 ~6 → 168× 爆炸** → 直接驱动 spline 调用 **20×**（4,695,145 = 130,420/chunk vs 旧 6,250）。
- **H3（多线程 thread_local thrashing）：成立（放大器）**。单槽 thread_local（density.h L660-663/L718-721）+ 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×**）；调用量不变（4,703,488 ≈ 4,695,145）；wall mt 8488ms > t1 6533ms（并行反而更慢）。
- **数字闭环**（三组独立计数器互相印证）：CACHE2D miss 351,536 = 14,061 rebuild × 25 角点 ✓；spline 4,695,145 ≈ 2,400,550（SPLINEDEBUG 非 leaf）× 1.96 ✓ ≈ 351,536 miss × 13.36 spline/miss ✓；130,420/chunk = 9,765 miss/chunk × 13.36 ✓；36,252 ÷ 36 = 1,007 ✓。
- **08-11 vs 08-12 数据口径**：08-11（rebuild 438,092 / 单次 20,598ns）为多线程 thrashing 环境粗计数器；08-12（36,252 / 1,714ns）为单线程精确统计。不构成矛盾，放大链实为「rebuild 168× × 13.36 spline/miss」。

### ✅ judge 审查通过（review-rootcause.md）

- **主结论通过**：H2 主因（FlatCacheDF 单槽 + buildGrid 角点 i=4 越界 → 嵌套递归蔓延 112 chunk，rebuild 168×）、H3 放大器（thread_local thrashing 16×）、H1 非主因（y 主序注释矛盾已实证不触发 spline），机制与代码一致，数字闭环可复核，置信度标注合法，修复方向（per-chunk 多槽缓存）不破坏 BK-001（采样值逐位不变）。
- **7 项修正/澄清建议**（已处理或已声明）：① CACHE2D 第 4 个 cacheId 来源（spline locationFunction 可能为 Cache2D，列入 root-cause §6 不确定点）② 08-11 vs 08-12 数据差异（口径说明已补入 07 篇 + 发现 #10 修正）③ collect-summary Java 循环顺序断言修正（root-cause §4.1 独立核对为 y 外层，非 x→z→y）④ index.yaml 登记 root-cause-draft/review-rootcause（本次完成）⑤ retry 记录缺失（H1/H2/H3 单轮验证 + 数字闭环已声明）⑥ 噪声卡历史无法核对（工作区无 noise_cards.json，留档）⑦ wall 时间 6448.0 vs 6533.3 来源注明（取 collect-summary）。

### ✅ 用户拍板 + 修复启动

- ✅ **根因定论（用户拍板确认）**：H2 主因（FlatCacheDF 单槽缓存 + buildGrid 角点越界 → 嵌套 FlatCache 邻居 key 污染 → 递归蔓延，rebuild 168× → spline 20×）+ H3 放大器（thread_local thrashing 单次 ×16）+ H1 非主因（块级不触发 spline）。
- 🔍 **Phase 2 修复中**：**per-chunk 多槽缓存**（主修复，低风险，采样值逐位不变，不破坏 BK-001 对齐；保留 k=4 边界命中语义 density.h L700-702）→ **线程亲和恢复**（后续，消除 thrashing）；改循环顺序不推荐（H1 非主因）。修复验证待闭环（以 08-12 同口径计数器复测 rebuild/spline 回落）。


## 2026-08-12（补）：性能回归修复实施与闭环（16 槽 LRU 失败 → 上下文绑定成功）（✅ 修复闭环 / 用户验收）

> 承接上一条 2026-08-12 根因定论条目。修复经历两版演进：初版 16 槽 LRU 未消除蔓延 → 终版「当前生成 chunk 上下文绑定」与 Java per-chunk 实例语义完全对齐，验证达标 + judge 通过 + 用户验收。设计文档 `.investigations/perf-rework/fix-design.md`（§0 含实现演进注记）+ 审查 `.investigations/perf-rework/review-fix-delivery.md`，均已登记 `.artifacts/index.yaml`（kind: plan / review，status: candidate）。

### 实施演进：16 槽 LRU → 上下文绑定

- **初版（16 槽 LRU）**：FlatCacheDF/Cache2DDF 均改 thread_local 16 槽 LRU（`std::array<SubSlot,CAP>` key/grid/stamp，模拟 Java per-chunk 实例缓存）。实测 rebuild 36,252→**7,318**（5× 降）但**未消除蔓延**：rebuild **203/chunk** vs 期望 6、chunk 覆盖仍 **112**（splinedebug_8576_t1_fixed.txt；SPLINE 14,772/chunk）。→ **弃用原因**：16 槽 LRU 仍为「pos 推导的邻居 key」构建网格，只减少重建频率，**不改变「越界=重建」语义**。
- **关键洞察**：Java FlatCache 是 **per-chunk 实例**（构造时绑定 chunk、一次性预计算 25 角点、越界 delegate.sample 直算**永不构建邻居网格**，ChunkNoiseSampler.java L836-881）；C++ 是全局单例 DensityFunction 树，单槽/多槽缓存都做不到「越界不重建」——必须显式传入当前 chunk 上下文。
- **终版（当前 chunk 上下文绑定）**：thread_local `g_curChunkX/Z`（density.h L40-41）在 `fillOneChunkCore` 入口 RAII 设置、函数返回恢复 `INT32_MIN`（judge 修正项 ② RAII 恢复已闭环；诊断路径回退分支语义保留）；网格绑定当前 chunk，k/l 相对 startBiomeX 计算（`k=(pos.x>>2)-slot.cx*4`），越界 → `delegate.sample(pos)` 直算不重建。**Cache2DDF 保留 16 槽 LRU**（角点共享列可命中，无蔓延风险）。与 Java FlatCache 六维逐条对齐（review-fix-delivery.md 审查要点 1 表：实例绑定/网格构建/k-l 计算/界内查表/越界直算/边界共享 ✅）。
- 机理：buildGrid 角点 i=4 的 pos 采样时 `cx=g_curChunkX=当前 chunk` → `k=4 ∈ [0,5)` 命中本网格；更远越界 → 直算。**蔓延根除**。

### ✅ 验证数据（终版 ctx，2026-08-12 落盘）

数据文件：`cmd-output/regress_8576_raii.txt`、`regress_3200_raii.txt`、`wgprofile_8576_t1_ctx.txt`、`splinedebug_8576_t1_ctx.txt`（stat_ctx.py 统计）、`bench_8x8_noprof.txt`。

| 指标 | 修复前（08-12 定论） | 终版 | 结论 |
|---|---|---|---|
| FLATCACHE rebuild | 36,252（~1007/chunk，168×） | **216 = 6.0/chunk** | 期望 ~6 完全达标 ✓ |
| rebuild chunk 覆盖 | 112（36 生成 + 76 邻居） | **36** | 蔓延根除 ✓ |
| CACHE2D miss | 351,536 | **23,117** | ↓15× |
| SPLINE（SPLINEDEBUG 非 leaf 口径） | 66,682/chunk | **3,032/chunk** | 回旧基线 6,250 水平 ✓ |
| spline.sample（WG_PROFILE 全量） | 130,420/chunk | **5,906/chunk**（212,622/36） | ↓22× |
| 单线程 wall | 6,533ms（181ms/chunk） | **2,910ms** | 2.2× |
| bench_chunks 单线程 | ~181ms/chunk | **62.38ms/chunk** | 3× |
| 对齐 8576 / 3200 | 99.9994% / 99.9997% | **99.9994% / 99.9997%** | 零退化 ✓ |

- 口径注明（judge 修正项 ③ 已闭环）：SPLINEDEBUG `[SPLINE]` 为入口行（非 leaf）计数；WG_PROFILE `spline.sample` 为全量采样计数；wall/bench 为落盘文件数值（wgprofile_8576_t1_ctx.txt wall=2910.0ms；bench_8x8_noprof.txt threads=1 62.38ms/chunk）。
- 16 槽 LRU 对照：rebuild 7,318（203/chunk）、覆盖仍 112、bench 79.91ms/chunk（bench_fixed_ctx.txt）、wall 3,469ms（wgprofile_8576_t1_fixed.txt）——方向正确但未达标，弃用。

### ✅ judge 审查通过（review-fix-delivery.md）

- **主结论通过**：修复机制（FlatCacheDF 上下文绑定 + 越界直算不重建、Cache2DDF 16 槽 LRU）与 Java per-chunk 实例语义逐条对齐；边界 k=4 命中语义保留；buildGrid 角点 i=4 不再触发邻居网格重建（机理经代码路径推演成立，实测 rebuild 216/覆盖 36 吻合）；纯缓存路径改造零退化在数学上成立（双种子 99.9994%/99.9997% 落盘与修复前一致）；thread_local + fillOneChunkCore 单线程完整处理保证线程安全（无跨线程上下文污染）。
- **4 项修正已闭环**：① fix-design.md 补实现演进注记（§0）+ 登记 index.yaml ✅ ② fillOneChunkCore 末尾 RAII 恢复 g_curChunkX/Z=INT32_MIN + 注释修正（「未设置或已恢复时回退」）✅ ③ 性能数字口径注明（SPLINEDEBUG 非 leaf vs WG_PROFILE 全量；以落盘文件为准）✅ ④ retry 轮次记录缺失声明（修复为工程迭代，验证单轮完成）✅

### ✅ 用户验收 + 剩余课题

- ✅ **用户验收（2026-08-12）**：修复闭环确认，性能回归结案（rebuild 216=6.0/chunk 完全达期望、蔓延根除、双种子零退化）。
- 🔍 **剩余课题（独立于本次修复，待续）**：
  1. **多线程无加速**：bench threads=8 62.17ms/chunk ≈ 单线程 62.38ms——spline/cache 已非瓶颈，**aquifer+oreVein 阶段**（wgprofile_8576_t1_ctx.txt 20-52ms/chunk，远超 spline 贡献）成主导；需线程亲和（root-cause 方案 2）/ aquifer 并行化。
  2. **spline 单次 7,971ns**（WG_PROFILE ctx 口径）：调用量 ↓22× 后的单次成本，非本次修复引入的劣化（review 三源不一致 #2 已注明出处 = wgprofile_8576_t1_ctx.txt L80），与修复前 1,714ns 为不同测量口径。
  3. **aquifer 阶段 4× 级**（20-52ms/chunk vs 旧基线 6.5-8.9ms）——独立课题。


## 2026-08-13：spline 扁平化 + 边界列复用（无损优化）+ 多线程膨胀重新定性 latency-bound（✅ spline 扁平化闭环 / 🔍 边界列复用收益小 / 🔍 多线程根因待续）

> 承接 2026-08-12 修复闭环条目。本轮在「多线程内存带宽饱和优化」课题下做两个无损优化 + 一次根因重新定性。commit aae119d（density.h / density_builder.h）+ ae9a3b9（phase0-2 调查产物）+ 5ec4f07；judge 审查 `.investigations/perf-rework/review-aae119d.md` + 跟进 `review-aae119d-followup.md`。

### ✅ spline 扁平化闭环（主要收益）

- 递归 `shared_ptr<SplineDF>` 树 → 连续节点数组（nodes/locations/derivatives/subIdx/locationFunctions 池）+ 整数索引 + 非虚 `sampleNode`，Hermite 插值公式逐位不变。
- 单线程 density wall 61.7→47.1ms（**-23.7%**）、[A] threads=1 吞吐 92.08→71.68ms/chunk（**-22.2%**）（analyze_stagetimer 聚合 n=128）。
- 零退化：8576 99.9994% / 3200 99.9997%（`regress_8576_aae119d.txt` / `regress_3200_aae119d.txt`，本轮针对 aae119d 补落盘，闭合 judge 证据链缺口）。

### 🔍 边界列复用收益小（-1.7% 接近噪声）

- thread_local edge 缓存复用左邻 gx=4 列作 gx=0 列（CELL_X=4 坐标对齐，无损）。density 47.1→46.3ms（-1.7%）、吞吐 71.68→72.06（+0.5% 无改善）。
- 根因：buildGrid 耗时大头不集中在 gx=0 列（FlatCache buildGrid 只在首个角点触发一次，跳过 gx=0 只移到 gx=1；其余 244 角点查表命中）——优化了错误目标（角点采样次数而非树遍历触发点）。

### 🔍 多线程膨胀重新定位 latency-bound（DDR5）

- 用户纠正内存 DDR5-5600 双通道 → 旧「DDR4 带宽饱和 ~17.8GB/s」定论失效。
- 重新定性 latency-bound：8t spline 单次 10× vs noise 1.3× 不对称膨胀 = cache miss 延迟，非带宽对称争用。
- spline 扁平化后单线程 -24% 但 8t density 460.8→478.3ms 无改善 → 多线程根因在 InterpolatedDF::buildGrid 1225 角点树遍历整体（spline + FlatCache 查表 + noise 的 cache miss 叠加），不在 spline 递归本身。**待解决方向 = DFC（整个 DF 树扁平化）**。

### judge 审查（保持 draft）

- 代码语义无损通过（Hermite 逐位等价 + 边界复用坐标对齐）；零退化证据链缺口 → 跟进补 regress 落盘；-1.7% 选择性报告 → 补总 wall 口径；术语「FlatCache buildGrid」→ 修正为「InterpolatedDF::buildGrid」。
- 状态保持 draft（多线程膨胀课题未闭合，需重新定位 buildGrid 树遍历 cache miss 构成后再评估 DFC）。

## 2026-08-15：G4 编译时间修复——A 方案（spline 6 表 SSBO 化）实施 + A5 coord 查表根因 + 达标（✅ 性能/正确性双达标 / 🔍 遗留 P2/P3）

> 承接 2026-08-14 D21 条目（903.4s 根因 = spline 动态 node 索引 + 方案决策点）。用户拍板 **A 先行**（spline 数据表 const→真 SSBO，架构计划 001 修订版）。实施 + 二分 + 修复完整记录落盘 `.investigations/perf-rework/a-plan-ssbo-implementation.md`（A5 节）。

### ✅ A 方案实施（A1a-A4a 完成）

- dfc_gen.py `_spline_ssbo_glsl` 重写：6 张 spline 表（NODE_PACK/LOCS/DERS/VAL_F/VAL_KIND/VAL_NODE）const→`layout(set=0, binding=6..11, std430) buffer` SSBO；spline_eval 恢复 b1a 设计的 while 栈显式栈后序求值（帧 {node,i,coord,stage,v0,v1}，stage 0/1/3，32 深）；spline_find_range 恢复 while 二分；新增 `self.spline_layout` 导出 → gen_cpu 输出 7 个 spline 成员（**D19 合规：宿主零硬编码**）。
- dfc_final_backend_e2e.cpp：descriptor 5→12 binding，新增 6 个 spline SSBO buffer 创建/上传/绑定/释放（binding 6-11）；compile_bench descriptor 8→12；新增 `gen_spline_diag.py`（spline 剩余成本二分变体生成器）。

### ✅ A4b 性能（compile_bench / e2e pipeline 计时）

| 变体 | const 表版（D21） | SSBO 版（A 初版） | 修复后 |
|---|---|---|---|
| 完整 | 903.4s | 350.6s（-61%） | **67.4s**（-92.5%，**达标 <2min**） |
| no_old（去 fp64） | 591.8s | 278.8s | **58.9s**（fp64 交互 ~310→~72→**~8.5s**） |
| no_spline | 17.6s | 17.2s | 17.2s |
| no_old+no_spline | 7.3s | 8.1s | — |
| **spline 子系统** | **~885s** | **~333s**（-62%） | **~50s**（-94%） |

- **达标判定 ✅**：67.4s < 2min 目标（架构计划 §5 拍板 HOOK）。3 次测量 67.4/71.4/101.8s 有波动，均 <120s。数字口径（judge 审查项）：71.4s = compile_bench 单独测 vkCreateComputePipelines；67.4s = 同 spv 在 e2e 内 pipeline 计时；同一 spv 两工具差 ~4s 属测量上下文/噪声；final 确认值见 cmd-output/compile_bench-A5-*.txt。
- **fp64 次因自动作废**：修复后 no_old 只省 ~8.5s——fp64 成本本质是「与 spline 展开的交互效应」，coord 查表修复后消失（NEXT_SESSION 待办 2 不再需要）。

### ✅ A4a 正确性（与基线逐位一致）

- maxDiff=**3.128e-07** / avgDiff=**1.097e-08**，与基线（D17/D19 修复后 while 栈 + const 表版）**逐位一致**（e2e N=1024 seed 8576294172403134396；TOP 差异点 i=1004 pos=(44,-49,0) 同点位同值）。
- ref_probe 分量：factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105。
- 结论：SSBO 化 + 查表修复语义零影响（spline 数据收集逻辑未动，只改输出形态）。

### ✅ A5 根因二分 + 修复（coordType 查表，本日最重要新知识）

- **二分证据链（减法二分，非猜测）**：fixed_node（361.0s ≈ full）排除「动态 node 索引」在 SSBO 版是主因（**D21 结论有版本域：const 表版成立、SSBO 版不成立**——SSBO 已把动态索引变运行时读）；coord_const（37.2s）定位 coord 表达式贡献 ~313s；coord_slot0（302.3s）排除「不同实例数」因素；coord_case0（74.8s）定位 1 次 normal_noise 调用 +37s；no_spline（17.2s）排除 eval_df 内同函数调用慢。
- **机制**：spline_coord 的 `switch(coordType)` 让每个 case 内 `NOISE_SLOT_BASE[0]` 成为**编译期常量下标** → 常量传播进 normal_noise 数据驱动函数 → NORMAL_PACK 读取静态化 → **循环展开**（每次调用 +37~75s）。eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]` 索引完全动态 → 驱动放弃展开（快）。
- **修复**：spline_coord 改「coordType 运行时查表」——`const int COORD_SLOT_TABLE[N] = int[](...)` + `int slot = COORD_SLOT_TABLE[coordType];` → normal_noise 实例索引运行时不可解析；fold 包装（coordType==2 的 abs 链）提取为 `if (coordType == 2)` 特例；非标准形态 fallback 原 switch。
- **教训（可复用）**：①「动态 node 索引」结论有版本域 ② 编译期常量下标进数据驱动函数 = 常量传播展开陷阱（switch/case 常量化 vs 动态索引是编译时间分水岭）③ 减法二分（coord_case0 单次调用定位 +37s）比猜快。
- 错误台账完整条目：gpu-accel-errors.md D22；通用模式：knowledge/discovered/algorithm-fingerprints.md 发现 #13。

### 🔍 遗留项（P2/P3，未立项）

- z 采样覆盖 / binding 号导出 / gen_split_shaders 宿主适配 / binding 2 死代码 / block_probe 终验（8576/3200 零退化终验）——均未立项。

## 2026-08-15（下午段）：知识库流程改进——错误记录强化方案 C 落地 + RE-Framework 同步申请（✅ 项目侧已落地 / 🔍 框架侧待评估）

> 承接 2026-08-15 上午段 D22 条目（kb-draft-d22.md 产出——实证「草稿质量靠 prompt 显式要求兜底，非 skill 自动保证」）。用户提问「知识库 subagents 的 skills 有没有写明错误记录要求」→ 主会话核对 core-knowledge skill → 拍板方案 C → 三文件落地；转交材料落盘 `.investigations/000-架构设计/framework-sync-request-error-recording.md`，供 RE-Framework 维护侧评估框架层同步。流程改进类条目（非错误结论），记录 触发→诊断→决策→落地 全链。

### 触发（用户提问）

- 用户问：「知识库 subagents 的 skills 有没有写明错误记录要求」→ 主会话核对 core-knowledge skill（框架通用层，项目副本 `E:\PYTHON\CoreSwap\.dsh\skills\core-knowledge`）。

### 诊断（skill 内容核对——通用层与项目级要求的缝隙）

- **skill 通用层已有基线**：「错误 > 正确」原则（错误链条先写、已排除不删、INDEX 置顶）+ 错误账本条目格式（`knowledge/errors/error-NNN-*.md`，四段式：错误现象 / 诊断过程（含结论根因）/ 排除后的正确认识 / 诊断方法论沉淀）。
- **缺项目级强化三处**（缝隙 = 详实度 / 载体 / 判错经验未达项目要求）：
  1. **五段式 vs 四段式**：项目要求「现象→根因→定位→修复→教训」五段完整（AGENTS.md 三-2、2026-08-13 用户明确）；skill 四段式**无独立「修复（改了什么）」段**，且未写「不得只记『已修复』而不记『为什么错』」。
  2. **判错经验沉淀**：项目要求「符号级错误一定是结构错不是精度错，先查公式/索引/坐标，别在精度上纠结」类**可复用判错方法必须沉淀**（比单条错误更有价值）；skill 仅有通用「诊断方法论沉淀」段（下次遇到类似症状 → 第一步做什么），**未强化到项目级 MUST 强调度**。
  3. **载体写死**：skill 固定 `knowledge/errors/error-NNN-*.md` 独立文件（每条一个文件）；项目实际载体 = `.investigations/perf-rework/gpu-accel-errors.md` 等**独立成篇 + 末尾「错误→根因」速查表**（A-G/D 系列一个文件）——skill 未说明「项目可自定义错误台账载体」。
- **为什么这是问题（实证）**：错误优先原则项目早有、skill 有通用版，但 2026-08-15 上午 D22 草稿质量达标是靠派知识库 subagent 的 prompt **显式要求**「按现象→根因→定位→修复→教训格式（参照 D21）」兜底（kb-draft-d22.md 即产物）——**不是 skill 自动保证**；每次派 subagent 都需人肉强调，漏一次即退化。

### 决策（方案 C 拍板）

- 主会话给出三个候选方案，用户拍板 **方案 C**：新建项目级规范文件承载强化（方案 C 内容见转交材料 §二；被否方向的核心顾虑：仅靠 prompt 兜底不可靠——本次实证；直接改只读框架 skill 越界）。
- 方案 C 三件套：① 项目级规范文件 `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`；② AGENTS.md §九新增「错误记录强化」强制行；③ 转交材料供 RE-Framework 维护侧评估框架层同步。

### 落地（✅ 三文件）

- **`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（新建，项目级知识库产出须知，68 行）**：
  - 一、错误优先原则（错误 > 正确、被排除假说保留、判错经验尤其要记）；
  - 二、五段式格式表（现象/根因/定位/修复/教训 + 反模式三条：只写已修复 / 现象=根因 / 无定位过程）；
  - 三、知识库载体映射（错误台账 → gpu-accel-errors.md 等独立成篇 + 速查表；结论 → 01-09 主题篇；过程 → 10 时间线；通用 → discovered/）+ 载体纪律；
  - 四、产出检查清单 10 项（subagent 交付前自检）；
  - 五、与 core-knowledge skill 关系（冲突时项目级文件优先，同 AGENTS.md 优先级规则）。
- **AGENTS.md §九「知识库更新强制触发点」新增「错误记录强化」行**：派知识库 subagent 的 prompt MUST 包含一行 `先读 E:\PYTHON\CoreSwap\knowledge\SUBAGENT-KNOWLEDGE-GUIDE.md，按其中格式与载体要求产出草稿`，并写明理由（skill 通用层无「不得只记已修复 / 判错经验沉淀 / 项目自定义错误台账载体」三处强化，靠 prompt 兜底不可靠——2026-08-15 实证 D22）。
- **`framework-sync-request-error-recording.md`（转交材料，.investigations/000-架构设计/，44 行）**：背景（三处缺口 + D22 实证）→ CoreSwap 侧落地（方案 C）→ 建议框架层同步（core-knowledge skill 增「项目级错误记录强化（可选适配）」节：五段式、不得只记已修复、判错经验、载体灵活、被排除保留）→ 同步边界建议（框架保持通用基线；项目级强化归项目侧文件，框架提供「可被项目覆盖」说明；若框架内置五段式，建议把 skill「诊断过程」段改/补为「定位（诊断方法/工具）」+ 加「教训（可复用判错经验）」段对齐）。

### 🔍 框架侧待评估（RE-Framework 维护侧）

- 转交材料已就位，待 RE-Framework 维护侧评估是否在 core-knowledge skill / 模板层同步增强（五段式、判错经验、载体灵活、同步边界四条建议）。
- 项目侧已闭环 ✅：规范文件 + AGENTS.md 强制行 + 转交材料三件套完成；后续派知识库 subagent 的 prompt 一律带「先读 SUBAGENT-KNOWLEDGE-GUIDE.md」行（AGENTS.md 九强制，随 todo 预置纪律同款）。

## 2026-08-15（晚段）：block_probe 集成立项 I1-I5——GPU 引擎接入 worldgen + D23 spline 边界 bug（✅ I1-I5 集成闭环 / ✅ D23 GPU+sim 双修 / ✅ judge 4 P1 全闭合 / ✅ 用户 2026-08-15 拍板 confirmed）

> 承接 2026-08-15 上午段 D22 条目（A 方案 SSBO 化 + coord 查表达标）。架构：`.investigations/000-架构设计/架构计划-gpu-integration.md`（002，用户 2026-08-15 批准）。目标：DFC + CpuBackend + Vulkan 运行时接入 worldgen，8576/3200 零退化终验 + 吞吐对比。集成记录 `.investigations/perf-rework/i-integration-record.md`；judge 审查 `review-003-d23-integration.md`（4 个 P1，P1-1/P1-2 已闭环、P1-3 已重跑落盘、P1-4 由本知识库更新闭环）；D23 完整错误记录 gpu-accel-errors.md D23 段（含最终合并版 + 判错经验补充段）。

### ✅ I1：Vulkan 运行时封装（vulkan_runtime.h）

- header-only 组件（复制到 `worldgen/src/vulkan_runtime.h`）；接口 init / createPipeline(spv) / createBuffer / upload / makeDescriptorSet<N> / dispatch / readback / destroy / destroyBuffer
- 语义与 e2e 内联版逐位一致：**12 binding storage buffer 布局**（binding 2 已删 OriginBuf 但保留占位）、host-visible+coherent memory、单 command buffer + fence、256 work items/组
- 驱动一次性 pipeline 编译 ~70-100s（domain probe 标注）；e2e 改用组件后 maxDiff=3.128e-07 / avgDiff=1.097e-08 与内联版逐位一致，pipeline 90.9s 达标

### ✅ I2：GpuDensityEngine PIMPL + worldgen 接入

- `vulkan-proto/gpu_density_engine.h/.cpp`（PIMPL，复制到 `worldgen/src/`）；接口 GpuDensityEngine(seed, spvPath) / fill(coords, n, out) / sample / splitTotal / perSample / splineBindBase
- **PIMPL 原因（集成期新坑）**：cpu_backend.h → density.h 的 static 成员定义（InterpolatedDF::nextId 等 L937-942）**非 inline**，多 TU include 会 **LNK2005**（worldgen_core 恰好单 TU 持有定义未触发；引擎引入第二 TU 暴露）→ **修复**：density.h L937-942 static 定义加 `inline`（C++17 inline 变量，语义与单 TU 完全一致，零运行时影响）
- 引擎验证（gpu_fill_probe）：maxDiff=3.128e-07 / avgDiff=1.097e-08 与 DensityBuilder 参照逐位一致；splitTotal=8672 / perSample=352 / splineBindBase=6 对齐生成器（D19 合规：宿主零硬编码）
- worldgen 接入（worldgen_api.cpp）：WorldgenHandle 加 `gpu` 字段（`#ifdef CORESWAP_GPU_ENABLED` 条件）；wg_create 尾部 env `WG_GPU_FILL=1` 时构造引擎（spv 从 gpu-assets 读，缺文件 CPU fallback）；wg_fill_density GPU 分支（批量坐标 → fill → float 转 double 输出）/ CPU 分支（默认，零退化）

### ✅ I3：生成器产物纳入构建（gpu-assets）

- 目录约定 `worldgen/gpu-assets/`（cpu_backend.h + final_density.spv）；gen_final_density.py 同步 cpu_backend.h 到 gpu-assets（spv 由 glslc 编译后复制/脚本化）
- CMake：worldgen_core 加 gpu_density_engine.cpp / vulkan_runtime.h；`if(DEFINED ENV{VULKAN_SDK})` 条件加 Vulkan include/lib + CORESWAP_GPU_ENABLED 定义（无 SDK 时 CPU-only 构建）

### ✅ I4：零退化（8576 CPU 路径 + GPU 接入不破坏）

- **I4a**：8576 CPU 路径 99.9994% 与基线一致（block_probe CPU 路径实测；3200 零退化沿用 2026-08-12 回归口径 99.9997%）
- **I4b**：GPU 引擎接入不破坏——块级生成（fillOneChunkCore）**恒走 CPU finalDensity->sample**，GPU 引擎（WG_GPU_FILL=1）仅构造 + wg_fill_density 批量接口生效，块级路径不受影响（fallback 机制 + WG_GPU_FILL=1 下 block_probe 运行不崩溃）
- ⚠️ **范围修正（judge P1-2）**：I4b 不是「GPU 参与块生成的逐位验证」——块级正确性由 CPU 路径保证；GPU 引擎自身的逐位正确性由 e2e（3.128e-07）+ domain probe（9.9e-9）验证（i-integration-record 表述已修正）

### ✅ I5：吞吐对比——GPU 24-32x，吞吐探针带 diff 抽查 → 发现 D23

- gpu_throughput_probe（chunk 批量 1/4/16/64）实测：**GPU 24-32x**（1/4/16 chunks）
- **意外收获**：探针顺带做同点 diff 抽查 → **16/64 chunks maxDiff 飙到 2.02e-01 / 4.45e-01**（应 ~1e-7 量级），1/4 chunks 正常（1.04e-06 / 1.33e-06）→ 发现 GPU 引擎在 e2e 验证域外系统性错值 → 引出 D23（**吞吐探针若只测时间不测 diff 就漏了**）
- D23 修复后：I5 各 chunk diff **1e-6~4e-6**（正确性恢复），吞吐 24-32x 保持

### ✅ D23：spline 边界外推遇嵌套 value 直接返回 0（GPU+sim 双修，judge P1-1 追补闭环）

> 完整错误记录（五段式 + 速查表）见 gpu-accel-errors.md D23 段；通用模式见 discovered/algorithm-fingerprints.md 发现 #14。此处时间线式记录推理过程（保留被排除候选与中间误判）。

**现象**：I5 吞吐探针 16/64 chunks 带 diff 抽查发现 GPU 引擎在 e2e 验证域外系统性错值——决定性单点 (784,160,-408) gpu=0.045303289 vs cpu=-0.458333333（diff 5.036e-01，量级级差异非浮点舍入）；而 e2e 域（x≤63, y∈[-64,-49], z≤4）maxDiff=3.128e-07 全过——**e2e 域是 D23 盲区**。

**根因（最终锁定）**：`spline_eval` 边界外推（coord < loc[0] / coord > loc[n-1]）写成 `(splineValKind[valB]==0 ? splineValF[valB] : 0.0f)`——**嵌套 value（kind==1）直接返回 0.0，未递归求值**。vanilla `Spline.apply` L259/261 边界外推是 `value[0]+der[0]*(x-loc[0])`，端点 value 为嵌套样条时**必须递归求值**。触发：(784,160,-408) 的 spline55（factor 的 spline，locs=[-0.19,-0.15,-0.1,0.03,0.06]）coord（continentalness@c0）=0.060231412 **恰好 > 最后 loc 0.06** → 右边界 → vn=嵌套(spline54) → 0.0（参照应递归得 factor=4.524）→ 上层 entrances 链错 → fd 错。**e2e 域为何对**：域内 spline coord 全在 locs 范围内 → 正常 Hermite → 对；大坐标域 coord 恰好跨出末 loc → 边界嵌套 → 0。**D17 修复后遗留**（D17 只修 node_idx/val_begin 陈旧索引，未处理边界嵌套 value 的递归）。

**定位链（域扫描二分，非猜测）**：
1. throughput probe 16 chunks → top diff @ (784,160,-408)：先定位到「大坐标 chunk 域」（x=784 > e2e 的 x≤63）
2. domain probe 定点对比 → (784,-64,-408) 对、(784,160,-416) 对、(720,160,-432) 对 → **错误依赖具体 (x,z,y) 组合，不是简单坐标域**
3. z-scan（y=160 x=784）：z=-432..-412 全对、**z=-408/-404 错**（cz=2/3 格错）
4. y-scan（x=784 z=-408）：y=-64 对、y∈[-56,248] 几乎全错、y≥256 对（= 无地形常数分支 -0.02499）——**错误域 = 「y 中间层 + cz≥2」组合；正确域 = 常数分支层或 cz≤1**
5. 🔍 **y=72 反例（新嫌疑，后被根因解释）**：y=72 (cy=17) cz=2 对、y=160 (cy=28) cz=2 错——同 cz 同 cx 仅 cy 不同，若拆分/读取全对不应差异（未收敛于拆分/索引层）

**候选 fan-out 排除（❌，各一行）**：
- ❌ **H1 角点序**：interp 角点 delegate 顺序 GPU=sim 一致，排除
- ❌ **H2 cell 推导**：cx/cy/cz（整数除法 vs floorDiv）逐位核对无差，排除
- ❌ **H3 split 数值**：gpu_split_probe（纯 CPU）拆分数据无 NaN/无越界/cz 变化小数正确区分，排除
- ❌ **初判「缺 noodle_ridge_b 拆分行」**（grep 实证 split() 在 normals[191] 结束）——**证伪**：check_split_base.py 实证 192 个 normal 拆分实际生成（normals[160]=noodle@c0 base=8288）——误报来源 = 用全量序号对比纯 normal 的 normals[]
- ❌ **「双索引错位」**（gen_cpu 纯 normal 序号 0..191 vs gen_shader 全量 0..199，splitBase 错位 8）——**证伪**：数据来自**旧版 final_density.comp** dump（P2 修改前产物）；当前重新生成后 NORMAL_PACK[168]=8288 与 split 写、normal_meta 三方一致（check_two_alloc.py 0 处不一致 / check_meta_vs_splitbase.py 全 YES）——教训 ⑧：**对账必须基于当前生成产物，不能依赖旧 comp/spv 的 dump**

**求值分叉定位（决定性）**：sim（dbg_full_sim.py 复刻解释器）对 (784,160,-408) = 0.045303285 **与 GPU 完全一致** → **生成器产物 + 解释器共同逻辑 bug（不是 GPU kernel 特有）**；分量参照（DensityBuilder）：错点参照 sloped=-2.664 / factor=4.524，GPU/sim sloped 角点值 -0.0165（差 160 倍，结构性错）→ 嫌疑收敛 spline 链；node[54]（roughness@c0）拆分采样 -0.113109157 == CpuBackend 直接采样**逐位一致**（coord 正确）→ 分叉在 node[54] 之后：**node[22]/[33] SPLINE 大坐标域算出 0** → 对照 vanilla Spline.apply 逐行 → 边界外推分支的嵌套 value 用 0.0f 占位 → 最终锁定。

**修复（GPU 侧，dfc_gen.py `_spline_ssbo_glsl`）**：while 栈边界分支（i<0 / i>=n-1）遇嵌套 value 不再直接 0.0，改压子帧递归求值（新增 **stage 4=等边界 v0 / stage 5=等边界 vn**，回填后用子帧值做外推；与普通 Hermite 路径共用同一栈帧回填机制，无新增数组）。

**修复（sim 侧，dbg_full_sim.py 回归工具）**：显式栈移植同样的边界递归（stage 6/7 对应 GPU stage 4/5），但踩了两个**显式栈回填机制**的坑（GPU while 栈直接 outVal 回填无此问题）：
1. **outSlot 返回地址被覆盖**：压子帧时 `outSlot[sp]=-1` 清掉本帧自己的返回地址 → 深层嵌套完成时结果不回填祖父帧。修复：只改 stage 不覆盖 outSlot
2. **父帧 stage 被回填覆盖**：子帧完成回填 `stageStack[ps>>1]=2` 无条件覆盖 → 压 v0 子帧时父帧 stage 已设 1（等 v1），回填后被改成 2 → **跳过 stage 1（v1 求值）→ v1Stack 恒 0 → Hermite 用错值**。修复：父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），回填只写值不覆盖 stage
- **judge P1-1 追补**：审查发现 stage 6/7 完成路径仍保留原 L289/302 的 `stageStack[ps>>1]=2`（正是声称已修的同类 bug，normal-range 父帧的 v0 子帧为边界嵌套帧时仍会算错）→ 删除全部 5 处 `stageStack[ps>>1]=2`（grep 确认 0 残留）→ **verify_p11_recursive.py 显式栈 vs 递归版 Spline.apply 参照（vanilla 语义直译）1344 组合 0 mismatch**（覆盖边界触发域坐标 (784,160,-408)/(720,160,-432) 等）

**验证（seed 8576294172403134396，gpu_domain_probe / e2e）**：
- (784,160,-408)：0.045303289（错）→ **-0.458333343（对，diff 9.9e-9）**
- z-scan（y=160 x=784, z=-432..-404）：全部 diff 9.9e-9（原 z=-408/-404 错 0.5）
- y-scan（x=784 z=-408, y=-64..312）：y=80-120 diff 5e-7~3e-6（float 精度，原 0.03-0.5）；y≥128 全 9.9e-9；y≥256 常数分支 1.1e-9
- e2e 回归：maxDiff=3.128e-07 / avgDiff=1.097e-08 **与基线逐位一致（零回归）**（e2e-A5 落盘：pipeline 80.1s、TOP00 i=1004 pos=(44,-49,0) diff=3.128e-07；D23 修复验证记录 pipeline 94.4s，均达标）
- sim：eval_df(784,160,-408)=-0.458333333 ✓；sim vs e2e-A5 全量对拍 maxDiff=5.7e-9 ✓ 无回归；dbg_full_sim 四点全对齐
- I5 复测：各 chunk diff 1e-6~4e-6（正确性恢复），吞吐 24-32x 保持

**教训（D23 综合，完整版见 gpu-accel-errors.md D23 段 + discovered #14）**：
1. **e2e 单域验证是盲区制造机**：域内全过 ≠ 域外正确；吞吐/性能探针必须顺带做 diff 抽查（多 chunk / 多 cell / 多 y 层）
2. **边界分支是「执行不到」类 bug 的温床**：e2e 域触发不到的分支（边界外推、嵌套边界）必须用跨域采样覆盖
3. **模拟器复现 0.045 = 生成器+解释器共同逻辑 bug**（不是 GPU 特有）——「GPU 特有 vs 共同逻辑」二分法先做
4. 与 vanilla 逐行对照是最后手段也是最终手段：**Spline.apply 的边界外推是递归求值，不是取 0**
5. **显式栈移植纪律**：「返回地址（outSlot）」与「父帧恢复点（stage）」是两套状态——压帧时各设一次，回填时只写数据槽，任何「回填时顺带改父帧 stage」的优化破坏等待语义

### 🔍 遗留项（未立项 / 待复核）

- **judge P1-3 复核**：I5 吞吐已重跑落盘 cmd-output/throughput-I5-*.txt（1/4/16/64 chunks，64 chunks 档位 ~10min+），复核数字后闭合
- **judge P2-2（低危，遗留 NEXT_SESSION 待办 2）**：shaderFloat64 未启用 + GpuDensityEngine 构造失败 `exit(1)` 无 CPU fallback（wg_create 已 try/catch 返回 nullptr 走 CPU；引擎内部 exit 需复核；shader 无 fp64 需求因 CPU 预拆分）
- ✅ **confirmed（用户 2026-08-15 拍板）**：I1-I5 集成 + D23 修复 + sim 回归工具 + 知识库闭环（本条目 + discovered #14 + gpu-accel-errors.md D23 判错经验段）全部确认；.artifacts 9 条升 confirmed

## 2026-08-15（深夜段）：GPU 块级生成立项 003（I6-I8）——逐 block 完整树 GPU 化实测不可行（❌ D24 split 全量上传带宽死局 / ✅ P2-4 并发崩溃修复 / ✅ 回退默认 CPU 零退化）

> 承接 2026-08-15 晚段 I1-I5 条目（GPU 引擎接入 worldgen + D23 修复闭环）。架构：`.investigations/000-架构设计/架构计划-gpu-block-integration.md`（003，用户 2026-08-15 批准「端到端 GPU 跑世界」）。目标：让 block_probe / 真实世界生成的**块级密度计算**（fillOneChunkCore 密度阶段）走 GPU，CPU 分支保持零退化。D24 完整错误记录（五段式 + 速查表行）见 gpu-accel-errors.md D24 段；通用模式见 discovered/algorithm-fingerprints.md 发现 #15。

### ✅ I6：fillOneChunkCore 密度阶段 GPU 分支 + fill() mutex 并发崩溃修复（P2-4 闭环）

- 接线：`#ifdef CORESWAP_GPU_ENABLED` 且 `h->gpu` 存在时，收集本 chunk 全部 **98304 点**（16×384×16，y = minY..minY+noiseHeight-1）→ `h->gpu->fill(coords, 98304, gpuOut)` 批量 dispatch（显存限制**分块 4096 点** batch fill）→ gpuOut(float) 转 densityBuf(double)，beard 逐块仍 CPU 加（L744 不动）；CPU 分支（无 GPU / 未启用）原样 = 零退化铁律。
- **并发崩溃（0xC0000005 @ nvtfi）**：I7 首次运行 `context=wg_fill_blocks_multi/fillOneChunk`，`code=0xC0000005`，栈在 **nvtfi（NVIDIA 驱动层）**——block_probe 默认 `-threads` 自适应多线程并发调 `h->gpu->fill()` → 共享 buffer 上传/dispatch 竞争 → **驱动层崩溃（不是返回错误，是进程级 0xC0000005）**。**P2-4 预言实锤**。
- **修复**：fill() 加 `std::mutex fillMtx` 串行化 → 无崩溃（P2-4 闭环；正确性解决，但串行化进一步劣化吞吐——「多线程并发 GPU 调用必须互斥」是硬约束，不是「可能有问题」）。

### ❌ I7：实测吞吐负面结论——11 分钟未完成 vs CPU 2.5 分钟（性能不可行）

- 24 chunks（8576 区域）GPU 逐 block 路径运行 **11 分钟未完成**（主动终止）；CPU 基线同区域 **2.5 分钟**——GPU 块级路径比 CPU **慢 4 倍+**（且未跑完）。语义对齐验证因此无法进行（跑不完）。
- **为什么不可行（D24 根因 = split 全量上传带宽死局，非计算慢）**：
  - GPU shader 求 finalDensity 完整树需要**每个点的全部分解坐标**：`splitTotal=8672` floats/点（CPU 预拆分，double→int32 格点 + float 小数）。
  - 逐 block 方案：98304 点/chunk × 8672 × 4B = **3.4GB split 数据/chunk** 需上传 GPU。
  - 分块 4096（显存限制）→ **24 次 dispatch/chunk**，每次 upload **142MB** + readback → 24 chunks × 24 次 = **576 次大上传 = 82GB 数据搬运** → PCIe ~16GB/s → 分钟级。
  - **GPU 快在「算」（compute throughput），这里被「喂数据」（host→device 带宽）完全主导**——GPU 批量加速的前提是「单点数据量小 + 点量大」，逐 block 方案把 8672 floats/点 的「每点数据量」直接变成带宽死局。
- **定位链**：① I7 首次运行（无 mutex）崩溃 0xC0000005 @ nvtfi → 多线程并发 fill 竞争 → mutex 串行化修复；② mutex 后无崩溃但 11 分钟跑不完 → 性能灾难暴露；③ CPU 基线 2.5 分钟 vs GPU 11 分钟未完成 → 带宽分析定位「split 全量上传」为瓶颈。

### ✅ 正确方向（若未来继续）：GPU 算网格角点 + CPU 插值，非逐 block 完整树

- GPU 只算 InterpolatedDF 网格角点（**768 点/chunk**，wg_fill_density 已验证 **22-39x**；27MB/chunk）→ CPU 三线性插值到 98304 逐 block。
- 数据量对比：768 点/chunk × 8672 × 4B = **27MB/chunk** vs 逐 block 98304 点 × 8672 × 4B = **3.4GB/chunk**（~125 倍数据量差）——**GPU 只在「网格角点级」批量才有意义**。
- 工作量中等：fillOneChunkCore 密度阶段改「先 GPU 出网格 → CPU 插值」，未实施。

### ✅ I8：回退——默认 CPU 路径零退化（99.9994%）

- I6 代码保留（WG_GPU_FILL=1 走 GPU 分支），**默认关闭 = CPU 路径 99.9994% 零退化**（8576 口径与基线一致；3200 沿用 99.9997%）。
- 最终结论：**GPU 块级加速在「逐 block 完整树」方案下不可行**（D24 定性为**方案不可行，非代码 bug**——接线正确、无崩溃、逻辑对，但吞吐不可行）；回退 CPU 路径为默认。

### 教训（D24 综合，完整版见 gpu-accel-errors.md D24 段）

1. **GPU 加速先算「每点喂多少数据」，不是先算「每点算多少」**：split 全量（8672 floats/点）让「每点数据量」成为带宽死局——GPU 批量加速的前提 = 「单点数据量小 + 点量大」（网格角点 768 点 × 27MB 可行；逐 block 98304 点 × 3.4GB 不可行）。
2. **吞吐探针结论有采样密度域**：I5 的 22-39x 证明的是「网格角点批量」，**不能外推到「逐 block」**——同引擎、同 shader，采样密度决定可行性（数据量 ∝ 点数）。
3. **多线程并发 GPU 调用必须互斥**（P2-4）：共享 buffer 上传/dispatch 无锁 → 驱动层 0xC0000005（不是返回错误）——GPU 资源并发是硬约束。
4. **负面结论也是结论**：I6 的「接线」本身正确（无崩溃、逻辑对），但吞吐不可行——记录「为什么不可行」（带宽分析）比假装成功有价值（错误优先原则）。

### 🔍 遗留项（未立项）

- 正确方向（GPU 网格角点 + CPU 插值）未实施——需 fillOneChunkCore 密度阶段重构（「先 GPU 出网格 → CPU 插值」），工作量中等，待后续立项评估。

---

## 2026-08-16：线程池 notify 丢失修复（0a781e1）+ C1 回滚（8966ba9）+ 影响评估 + clamp 发现 + MT 错误台账（✅ 修复闭环 / ↩️ 回滚 / 🔍 H3 待重测 / ⚡ clamp candidate 待实机验证）

> 承接 2026-08-15 深夜段（I6-I8）之后；提交时间 8/15 23:50-23:59，排查/评估/台账 8/16。完整五段式错误记录：`.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（MT1-MT7 + 判错经验 + 速查表）；影响评估：`notify-bug-impact.md`；勘探：`scout-map.md`；docs 影响标注：07-block-pipeline.md「2026-08-16 影响评估修正」。

### ✅ notify 丢失 bug 修复（0a781e1，8/15 23:50）

- **bug**：CoreSwapPool ensure()（L1057-1098）锁内建 worker + run() 入队后 notify_all()（L1125）竞争 → 补建 worker 错过通知永久等待（tasks 空 + stop false）→ 只有老 worker 干活 = **串行假象**（经典丢失唤醒）。引入 252d988（8/6 20:11 扩容支持），**活跃约 9 天**。
- **现象**：bench [A] T>1 顺序跑「反降 +19-29%」（T=1 73.23 / T=8 87.51 / T=12 89.92 / T=22 94.35 ms/chunk，bench-C2-20260815.txt）；WG_TASKTIME 实证补建 worker 全空闲（顺序跑 done_by 恒老 worker；**单独跑完美并行 = 池无增长时正确，bug 只在扩容路径暴露**）。
- **修复**：readyCount 原子（worker 进 wait 自增 / 拿任务自减）+ run() 入队前等 `readyCount >= workers.size()`（L1110-1118）。
- **影响**：8/11-8/15 所有 [A] T>1 顺序跑数据作废（串行假象）；**单线程数据、H2 主因（rebuild 168×）不受影响**（单线程精确统计）。
- **修复后验证**：64-chunk 8×8 前台重测（bench-notifyfix-8x8-20260816.txt）：[A] T=1 98.02 / T=8 89.88（**-8.3% 不再反降**）/ T=12 90.39 / T=22 97.76——收益仍被「每 chunk 并发下慢」吞掉（第二阶段课题）。

### ↩️ C1 thread_local 复用回滚（8966ba9，8/15 23:59）

- C1 候选验证（tl_col/tl_densityBuf 复用，消除每 chunk 1.2MB 堆分配/释放）→ **单线程慢 9%（71.68→77.93）+ MT 反降依旧** → 回滚；**C1 排除结论保留**（堆分配非 MT 反降主因，负面验证结果本身是资产）。

### ⚠️ 影响评估（8/16，notify-bug-impact.md）

- **H3「thrashing ×16」（mt 27,155ns vs t1 1,714ns）**：mt 侧数据在 bug 活跃期采集（实际并行度=1）→ **×16 需重新定性（🔍 待修复后重测）**；H2（rebuild 168×）保留。
- **WG_PROFILE/WG_STAGETIMER 计时污染揭穿**：density 460ms 伪影（真实 45ms）——独立污染源（探针自身开销），非 notify bug；探针已分离修复（cc93c50）。

### ⚡ threads clamp 发现（[B]/实机 M=1 结构性串行，candidate 待实机验证）

- `wg_fill_blocks_multi` L1189 `if (threads > count) threads = count;`（**66e05f5，8/5「方块层多线程并行」引入**；池化 c792e9d 后语义失效）→ count=1 时 clamp 到 1 → ensure(1) → **池恒 1 worker**。
- **实机链路铁证**：CppBridge.java L170-171（count=1 + THREADS）→ jni_bridge.cpp L93（`(int)count, (int)threads` 原样透传）→ L1189 clamp → L1193 ensure(1) → **实机 mod「多线程」可能从未真正并行**（结构性串行；与 notify bug 独立——notify 只影响 [A] 批量，clamp 影响 [B]/实机 M=1）。
- **修复待办**：clamp 改 `if (threads > count && count > 1)`（count=1 保留 THREADS）或实机改批量调用（未实施，记录待办）。

### ✅ MT 错误台账建立（mt-scaling-errors.md）

- **MT1** notify 丢失（✅ 已修复 0a781e1）| **MT2** H3 ×16 污染（🔍 待重测）| **MT3** clamp 结构性串行（🔍 待定性 + ⚡ candidate）| **MT4** 计时污染（✅ 已修复 cc93c50）| **MT5** C1 thread_local 退化（↩️ 已回滚 8966ba9）| **MT6** 修复后验证缺失（✅ 已补充 64-chunk 重测）| **MT7** runMtx「排队」未留痕（✅ 已核对留痕）+ 判错经验 9 条 + 速查表 11 行。
- **MT7 演进链核对（git log -S "runMtx" 实证）**：c792e9d 持久池（8/6）→ 252d988 扩容+shutdown（8/6）→ **e388ab4 runMtx 全局互斥（8/7，32 视距崩溃补丁 = 用户记忆的「排队」）** → **6e2c7ea per-run RunState 隔离取代 runMtx（8/11，批间真并行）**——「加了又去掉」只留一半痕（演进记录散在 10 时间线 L567/L1112/L1118，6e2c7ea 只改代码注释未显著标注旧方案作废；09 篇无「排队」字样，初稿说法已修正），本台账已留痕。

### 🔍 遗留项（未立项 / 待复核）

- 🔍 H3 ×16 修复后重测（mt 侧 spline 单次成本；若 mt≈t1 则 H3 降级/删除）
- ⚡ 实机实跑对比（clamp 推论最后验证——实机多线程生成时 C++ 侧 worker 数 / 吞吐与单线程无差）
- 🔍 scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」vs 8x8 数据（T=1 98.02）矛盾（中间状态混测，单线程基差 +37% 待同机同状态对照）
- 🔍 「每 chunk 并发下慢 7.5 倍」真实性（WG_MTTRACE fprintf stderr 锁竞争污染）——需无 fprintf 计数器测量
- 07 篇 L74/L97/L109 影响标注 + 文末「2026-08-16 影响评估修正」小节（本批次落盘）

---

## 2026-08-16（追加）：density 11× 真实 + spline 树遍历是根源——「并发下慢 7.5×」重定性（WG_PHASETICK 定论）

> ⚠️ **纠正**：先前 subagent 草稿声称「并发下慢 7.5× 不存在（探针污染）」——**错误**（基于 WG_DENSITYTICK 重复循环 bug 的假象）。本条目按 WG_PHASETICK（干净测量 + 补全 SplineDF 遍历）**定论：density 11× 真实**。
> 完整记录：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md` + `mt-scaling-errors.md` MT8。

### ✅ 定论：density 11× / 每 chunk 并发下慢 9× 真实（WG_PHASETICK）

| 阶段 | T=1 | T=8 | 放大 |
|---|---|---|---|
| density | 34-42ms | 400-412ms | **11×** |
| aquifer+ore | 8ms | 25-28ms | ~3× |
| surface | 7ms | 25-38ms | ~4× |
| total | 50ms | **462ms** | **9×** |

- **自洽验证**：462ms × 8 并行（64chunks = 8 批）≈ 3696 + 批间 = 4618ms = wall ✅
- **关键概念**：bench `med/N`（wall/64=72ms）是**吞吐均值**；每 chunk 真实耗时 = 462ms（8 worker 并行，wall 4618ms 处理 64 chunks）——**吞吐均值掩盖单 chunk 延迟**（之前把 72ms 当每 chunk 耗时 → 误判「只慢 8%」）。

### 🔍 spline 是 density 11× 的根源（补全遍历确认）
- **finalDensity 树含 6 个 SplineDF**（WG_SPLINESTATS：splineInst=6、537 节点、17KB 表、195 locationFunction）
- **之前误判「无 spline」**——typeid 遍历漏了 BlendDensityDF/WrappingDF（spline 经 blend_density 引用 continents/erosion/depth 分量）
- **spline 单次重推**：T=1 density 34ms / 2154 次 ≈ **15.8μs**；T=8 density 409ms / 2160 次 ≈ **190μs** —— **spline 单次并发下慢 12×**
- **spline 表 17KB（驻留 L2）——非 L3 miss 容量**；慢在 **spline 树递归（90 节点/实例）+ 195 locationFunction 虚调用 + 并发 I-cache/cache-line 争用**
- **优化方向 = C2ME 式 DFC 编译直排**（消除每点树遍历虚调用）

### ⚠️ 探针污染链（部分成立，非全部）
- WG_PROFILE/WG_STAGETIMER 的 density 34→400ms **与 WG_PHASETICK 一致（真实）**——不是探针污染，density 11× 真。
- WG_MTTRACE fprintf 锁竞争：部分成立（470ms 有打印污染，但量级接近真实 462ms）。
- WG_DENSITYTICK 6.95ms：**重复循环 bug，假象**（曾误导「并发正常」——已纠正）。

### ↩️ 作废清单（建立在 WG_DENSITYTICK bug / 概念混淆上）
- ~~「并发下慢 7.5× 不存在」~~（subagent 草稿 + 初稿 MT8——基于 WG_DENSITYTICK 假象，**错误**）
- ~~「density 11× 作废（探针污染）」~~（同上，**错误**——WG_PHASETICK 证实 density 11× 真）
- ~~**git 527cade「conclusively rule out per-chunk concurrency slowdown」**~~（错误结论提交，基于 WG_DENSITYTICK 假象 + 吞吐均值/每 chunk 耗时混淆）——**已由 fcbdad1 纠正**（density 11× 真实）。

### ✅ 保留结论
- **notify 丢失 bug（0a781e1 已修）**：真 bug（串行假象），独立于 density 11×。
- **density 11× = spline 树遍历虚调用 + 并发争用**（新定位，真实）。
- **Threads clamp（MT3）**：独立问题（[B]/实机 M=1 结构性串行）。

### 教训（第 6 个测量/探针案例，纠偏）
1. **区分吞吐均值（wall/N）与每 chunk 真实耗时**：wall/64=72ms（吞吐）≠ 462ms（延迟）。多线程下吞吐均值掩盖单 chunk 延迟。
2. **测量工具 bug 会给出「看似干净实则错误」数据**：WG_DENSITYTICK 6.95ms 看似 QPC 干净，实则重复循环 bug → 误导「并发正常」。**用「阶段耗时 × 并行批次 ≈ wall」自洽检查**（462×8≈3696+批间=4618 自洽；6.95×8≈55 ≪ 4618 不自洽 → bug）。
3. **不要用「探针污染」解释数据**——先验证测量工具自身（自洽性），再怀疑真实计算慢。初稿「所有探针都污染」是**过度泛化**。
4. **遍历要覆盖所有 DF 容器类型**：typeid 遍历漏 BlendDensityDF/WrappingDF 导致「无 spline」误判——遍历完整性必须验证。


## 2026-08-23（追加）：locationFunction 嵌套 SplineDF 证伪 + DFC 收益天花板——11× 多线程课题重大转向与完整 DFC C++ 立项

> 🔄 **重大转向**：08-16「density 11× = spline 树遍历虚调用（locFn 嵌套 SplineDF 递归膨胀）」的**根因定论翻车**——权威 JSON 数据源证明所有 spline coordinate 全是纯噪声 DF（无一嵌套 SplineDF）。同时 **DFC 收益天花板被钉死 ~5%**（无法消除主导的 shift_noise 噪声计算），**DFC 理论上不可能消除 11×**。用户据此拍板：不再把「实现 DFC」当作证明指针追逐的手段，**投入完整 DFC C++ 实现**（连续化 195 个多态 locFn，正确性底层目标 —— 性能另说）。
> 完整记录：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 一/二节）+ `mt-scaling-errors.md` MT11 + `concurrent-density-probe-scout.md` + `.investigations/perf-rework/dfc_cpu_mapping.md` + `dfc_grid_cache_design.md` + `verif_grid_cache_correctness.md` + `NEXT_SESSION.md`（7 节）。

### ❌ 一、证伪「locationFunction 嵌套 SplineDF」（推翻 08-16 旧论）
- **权威数据源**：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`（continents/erosion/ridges/ridges_folded/depth/factor/offset/jaggedness/sloped_cheese/base_3d_noise）。
- **事实**：所有 spline `coordinate` 全为**纯噪声 DF**——continents/erosion/ridges = `flat_cache(shifted_noise(...))`；ridges_folded = 纯 mul/add/abs 链。**无一嵌套 SplineDF**。
- **spline 嵌套真实位置**：仅存在于 `points[].value` 数据表，最多 3 层（≠ 树节点/coordinate 字符串引用嵌套）。
- → **08-16 旧论「优化 = 消除嵌套密度树递归（指数膨胀）」是误读** ❌。195 个 locationFunction 仍保留（散布指针追逐），但「嵌套 SplineDF 递归膨胀」这一根因描述不成立。

### ❌ 二、DFC 收益天花板 ~5%（DFC 理论上不可能消除 11×）
- **DFC 显式栈只消除两样东西**：虚调用（dispatch）+ `shared_ptr` 引用计数。
- **MVP 实测收益 ~5%** —— 因为**主导成本是 shift_noise 噪声计算本身**（每点都在算），不是分派。
- → **DFC 理论收益上限 ≈ 5%，不可能消除 11×**。08-16「DFC 是消除 11× 的良药」定位 **⚠️ 待重审**（见作废清单）。

### ✅ 三、11× 真实复现（干净测量，重确认）
- **工具**：`conc_density_probe` + `WG_PHASETICK`，同批 chunk、**无 warmup**。
- **数据**（avg density）：T=1 39.31ms → T=8 331.04ms = **8.4×**。
- **单 chunk(-6,-6)**：42.69ms → 391.41ms = **9.2×**（单 chunk 视角更陡）。
- **延迟随线程线性**：T=1/2/4/8 = 1×/2×/4.6×/9×（每加一份并发，单 chunk 延迟近似翻倍 → 共享资源争用特征）。
- **吞吐正常**：69 → 73ms/chunk（吞吐不受影响，慢在**延迟**——单 chunk 处理时间被并发拉长）。
- **概念区分再次证实**：`bench med/N`（吞吐）≈ 不降；每 chunk 延迟（wall / 并行 batch）显著上升——**吞吐均值掩盖单 chunk 延迟**。

### 🔍 四、根因收窄（scout 勘探，still candidate）
- **Tier-1（主）**：SplineDF 长串行依赖链 + locFn 散布堆指针追逐的**共享内存延迟**（多线程下 cache-line 伪共享 / 内存延迟放大）。
- **Tier-2（次）**：I-cache 争用（同批 chunk 并发遍历同一棵 spline 树，指令缓存互踩）。
- **已排除**（静态/实验）：a-1 yield 空转、b 超线程、d 硬锁、e Beardifier、c-3 17KB 表共享读（容量不构成 L3 miss）。

### ⚠️ 五、MVP 决定性对照**未复现 11×**（MT11 —— MVP 性能外推无效）
- **对照实验**：`mvp_spline_eval` 线程扫描（3 形态，全 amp=0.2x）→ **完美扩展**（线性无退化）。
- **为何复现不了**：MVP 表小（无真实 537 节点/17KB/195 locFn）、无真实堆指针追逐——MVP 的访存足迹与 production 完全不在一个量级。
- **结论（MT11）**：**MVP 只能验证算法正确性，不能外推性能/机制**。08-16「放大 MVP 验证 DFC」路径 **❌ 作废**。⚠️ 连带：MVP 的 ~5% 天花板收益**只在 MVP 的简化模型下成立**，production 的指针追逐/内存延迟可能比 MVP 更高——真正的收益必须在 production 上测。
- **教训**：MVP（简化/降级）回答「算法对不对」，回答不了「production 性能/机制」。性能结论必须 production 数据，不可由 MVP 外推。

### ✅ 六、用户拍板：投入完整 DFC C++ 实现
- **打破循环依赖**：之前误以为「必须实现 DFC 才能坐实指针追逐」→ 反正 DFC 是正确性目标（无论性能）就先做。用户明确**投入完整 DFC**（连续化 195 个多态 locFn）。
- **新目标定位**：DFC = **正确性底层实现**（替代 SplineDF 的确定性重写），性能是否改善**另测**——不再假定 DFC 能消除 11×。

### ✅ 七、DFC C++ 实现里程碑（Phase 0-3 达成 + 路径 C 可行性确认）
- **Phase 1**：`GLSL→C++` 映射表（`dfc_cpu_mapping.md`，23 种 DF 类型分派）。
- **Phase 2**：`gen_cpu` 扩展（`gen_cpu_sampling`）生成 C++ 采样函数（`CpuBackend.h`：`eval_density`/`eval_df`/`spline_eval` 显式栈 / `spline_coord`/`normal_noise`/`interp`）。
- **Phase 2c**：DFC C++ vs `dbg_full_sim`(＝GPU) **maxdiff = 2.06e-08**。
- **Phase 3**：DFC C++ vs production `finalDensity` **maxdiff = 6.52e-07**（128 点）→ **DFC C++ 可替代 SplineDF（正确性达成）** ✅。
- **Phase 4a-1**：**路径 C（grid 缓存）可行性验证** —— **语义成立，需改生成器（split 翻转）**：
  - grid 节点值唯一 `max|diff| = 0`；
  - 8 份同参实例等价；
  - edgeCol 复用无损。

### 📌 八、记录指引 / 作废标注（知识库归口）
- **完整记录**：
  - `.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 一/二节：证伪 + DFC 天花板 + 根因收窄 + 拍板）
  - `.investigations/worldgen-mt-scaling/mt-scaling-errors.md` **MT11**（MVP 未复现 11× —— 判错案例）
  - `.investigations/worldgen-mt-scaling/concurrent-density-probe-scout.md`（11× 复现 + scout 根因收窄）
  - `.investigations/perf-rework/dfc_cpu_mapping.md`（GLSL→C++ 映射）
  - `.investigations/perf-rework/dfc_grid_cache_design.md`（路径 C grid 缓存设计）
  - `.investigations/perf-rework/verif_grid_cache_correctness.md`（路径 C 正确性验证）
  - `NEXT_SESSION.md`（7 节）
- **被推翻假说 / 作废清单**：
  - ❌ 08-16「locationFunction 嵌套 SplineDF（递归膨胀）」——**误读，证伪**。
  - ⚠️ 08-16「DFC 是 11× 良药」——**待重审**（DFC 天花板 ~5%，不可能消除 11×）。
  - ❌ 08-16「放大 MVP 验证 DFC」——**作废**（MVP 复现不了 production 共享延迟，MT11）。
  - ❌ 「必须先实现 DFC 才能坐实指针追逐」——打破（用户拍板改为直接实现 DFC）。

### 🧭 九、判错经验沉淀（本 session 最重要资产）
1. **吞吐 vs 延迟**：并发下吞吐（wall/N）不变并不代表「并发没问题」——单 chunk 延迟（wall/并行批）才是被并发拉长的指标。11× 是延迟现象，吞吐正常（69→73ms/chunk）完全不矛盾。
2. **静态排除要干净实验**：仅「看着像嵌套」不足以定论，必须落到**权威数据源**（JSON）上逐一核对 coordinate 结构，才能排出「嵌套 SplineDF」。
3. **MVP 复现不了真实共享延迟**：MVP 简化模型的访存足迹 / 表大小 / 指针追逐与 production 差一个量级，**MVP 只回答算法正确性**，性能/机制结论必须 production 数据。
4. **先钉主导成本再立项**（MT11 直接教训）：DFC 立项前应先坐实「主导成本是可消除的分派/引用计数，还是不可消除的 shift_noise 噪声计算」——后者（噪声）是主导，DFC 天花板 ~5%，**先量化再立项，避免把优化方向建在错误的主导成本假设上**。


## 2026-08-23（追加）：DFC C++ 实现完整成果——消除 11× 并发放大实证 + 逐位对齐 + 性能优化链 + 未解问题

> 承接上文「2026-08-23：locationFunction 嵌套 SplineDF 证伪 + DFC 收益天花板 + 完整 DFC C++ 立项」（08-16 重大转向）。本节记录 **DFC C++ 实现的落盘成果**（对齐 + 并发放大 + 性能链 + 未解问题），已达成「可替代 SplineDF 的正确性」且实证「几乎消除 production 的 11× 并发争用」。
> 完整记录：`.investigations/perf-rework/vulkan-proto/dfc_cpp_conc.cpp`（并发放大）、`dfc_cpp_verif.cpp`、`dfc_cpp_vs_prod.cpp` + `.investigations/perf-rework/dfc_cpu_mapping.md` / `dfc_grid_cache_design.md` / `sample-splittop-optimization.md` + `.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 三节）。

### ✅ 一、DFC 消除 11× 并发放大（核心价值实证）
- **生产并发放大（对照）**：production density 单 chunk 39.31ms（T=1）→ 331.04ms（T=8）= **8.4×**（单 chunk 9.2×、density 11×）。
- **DFC C++（thread_local grid 缓存，每线程 chunk 内采样）**：T=1/2/4/8 per-sample = 882.7/905.3/1021.5/1157.6 μs（初版）→ splitTop 后 251.7/260.3/296.0/327.8 μs → 闭包优化后 238/314 μs。
- **并发放大 T=8 vs T=1 = 1.30-1.31×（各版本都保持）**——**DFC 几乎消除了并发争用**（production 8.4×/11×）。这正是 DFC 核心价值：消除 SplineDF 指针追逐/共享延迟导致的 11×。
- 意义：MT 课题 11× 的根源（SplineDF 递归虚调用 + locFn 散布堆指针追逐）在 DFC 直排上不再放大——佐证 Tier-1（共享内存延迟/指针追逐）是 production 11× 主因的判别方向。

### ✅ 二、DFC C++ 逐位对齐（正确性达成）
- `dfc_cpp_verif`（vs dbg_full_sim = GPU 蓝本）：**maxdiff=2.06e-08**。
- `dfc_cpp_vs_prod`（vs production finalDensity）：**maxdiff=9.57e-07**（768 点、跨多 cell）。
- **结论**：DFC C++（数据驱动直排 + grid 缓存）**已逐位对齐 production**——「能否替代 production SplineDF」的**正确性达成** ✅（性能另说）。

### ⚠️ 三、性能优化链（部分达成）
1. **splitTop 优化（✅ 3.5×）**：`sample()` 每点整树 `split()`（200 条）→ `splitTop`（只 interp delegate 的 @c0，25 行 = 200 的 1/8）——882→251μs；对齐保持（9.57e-07 / 2.06e-08）。
2. **闭包优化（D26，⚠️ 提速仅 5%）**：`eval_df_base`/`eval_df` 从遍历全 DF_NODES(163) → 各用闭包子集（interp 1-4 只 ~17-21 节点，顶层 ~21）。**正确性保持（对齐 9.57e-07/2.06e-08）但提速仅 251→238μs（~5%）**——**低于预估 2-4×，说明每点慢主因不是孤儿 delegate**（D26 详记）。

### 🔍 四、未解问题（待重诊断）
- **每点 238μs 真实主因未明**：闭包优化只降 5% → 主因不是孤儿 delegate。候选：① grid 构建摊销（buildInterpGrid 每 chunk 首访建 5×768 cell 全量 split）② sample() 每次仍调 splitTop ③ eval_density 结构成本。**下轮重诊断**（干净无探针整批 wall + 调用次数计数）。
- **DFC 每点 238μs > production 0.4μs/点**（绝对值仍慢约 600×，但并发放大 1.30× 很好）——整 chunk 生成仍可能慢/超时（需主因优化后实测）。

### ✅ 五、接入（完成）
- `worldgen_api.cpp` 加 `std::unique_ptr<CpuBackend> dfcBackend` + `WG_DFC_CPU=1` env 门控 + `fillOneChunkCore` density 阶段三路分支。
- **默认（WG_DFC_CPU 关）dfcBackend=nullptr → production（零退化逻辑保证）**；`WG_DFC_CPU=1` 用 `dfcBackend->sample`（对齐 9.57e-07）。
- **注意**：`CpuBackend` 表是 **overworld 专用**（DFC 仅适合 overworld；nether 等维度 minY/height 不同，生成器当前硬化）。

### 记录指引（知识库归口）
- 错误台账：`.investigations/perf-rework/gpu-accel-errors.md` **D26**（闭包优化提速仅 5% 五段式 + 判错经验 + 速查表行）。
- 根因/实证：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 三节：DFC 消除 11× 实证 + 未解问题）。
- 性能链/未解：本节 + `NEXT_SESSION.md`（若换 session 前更新）。

### 被推翻假设 / 作废标注（保持）
- 「DFC 是 11× 良药」⚠️——**本节实证澄清**：DFC 并不「降低每点绝对成本」（天花板 ~5%，每点 238μs > production 0.4μs），但 **DFC 消除了「并发下的放大倍数」（11×→1.3×）**——两维度独立，DFC 核心价值 = 消除并发争用（非消除每点慢）。


## 2026-08-23（追加）：DFC CPU 移植失败定论（❌ 绕圈无果，作废）+ 转下一真课题（production 并发争用无损修复）

> 承接上文「2026-08-23（追加）：DFC C++ 实现完整成果」。本节为 DFC CPU 移植方向**结案**：**不是「性能待优化」，而是「方向不可行，作废」**。整个 DFC 移植绕了一圈回到「没作用」——本节为准。
> 完整记录：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 四节：失败定论）+ `.investigations/perf-rework/gpu-accel-errors.md` D26 + `.investigations/perf-rework/vulkan-proto/dfc_cpp_conc.cpp` + `NEXT_SESSION.md` §8。

### ❌ 一、DFC CPU 移植失败定论（作废）

- **每点 600× 慢（硬伤，不可行）**：DFC CpuBackend `sample()` 每点 **238μs** vs production **0.4μs/点**（约 600×）。整 chunk = 98304 点 × 238μs ≈ **23.4s** vs production **39ms**（约 600×）。`dfc_fill_compare` 120s 超时——**任何实际场景不可用**。
- **核心矛盾（净作用为负）**：DFC「消除并发放大 1.30-1.31×（vs production 8.4×/11×）」是用**更大的新问题（整 chunk 慢 600×）**换掉旧问题（并发 11×）——**净作用为负**。
- **DFC 是 GPU 性质设计搬到 CPU（用错工具）**：`split-precompute`（每点重算 8672 floats）+ `grid 构建摊销`（buildInterpGrid 每节点全量 split）+ `eval_density 结构成本`（splitTop 每点 25 条 + eval_df 闭包遍历）——三者是为 GPU「无 fp64 + 并行摊销 prefetch」定制的妥协；**CPU 串行每点付全额** → 每点慢根源。
- **立论证伪（为什么从根上就错了）**：核心假设「虚调用是 11× 元凶」已证伪——权威 JSON 证实所有 spline coordinate 均为纯噪声 DF（无一嵌套 SplineDF），DFC 消除虚调用只 ~5%，**不可能解决 11×**。整个方向建立在错误前提上。

### ✅ 二、真收获（非无用，是本次课题最贵资产）

1. **证伪「虚调用是元凶」**——避免未来继续在「消除虚调用」上投入（本次 DFC 方向的最大价值）。
2. **确认 production 单点 0.4μs 很快（并发才是问题）**——把问题从「单点慢」重新定位到「并发争用」。
3. **完整 DFC 对齐链**（逐位对齐 9.57e-07 vs production / 2.06e-08 vs GPU 蓝本，证明 CpuBackend 正确）——但「正确的但无用」（每点慢使价值归零，仅保留作对齐参照）。

### 🚫 作废标注

| 项 | 状态 |
|---|---|
| DFC CPU 移植作为性能方案 | ❌ 作废（绕圈无果） |
| DFC CPU 移植作为正确性/对齐参照 | ✅ 保留（CpuBackend 正确） |
| WG_DFC_CPU 接入 | ⚠️ 默认关（保留代码，非生产路径） |

### 🔍 三、下一真课题：production 并发争用的无损修复（非 DFC）

- **真问题**：production 单点 0.4μs（快），但并发 11×（SplineDF 指针追逐 + locFn 散布堆共享内存延迟放大）。**目标：保留单点快（0.4μs），修复并发争用**——**不是 DFC**（它在 CPU 不可行，600× 慢）。
- **候选（待 scout/勘探，均为生产自身可修复点）**：
  - SplineDF `locationFunctions`（散布堆 locFn 指针追逐）→ **locFn 连续化/去 shared_ptr**（保留多态但布局紧凑）。
  - production 的 thread_local grid 缓存已做部分；**找其余共享可变状态**（并发 11× 的可修复点）。
- **关键**：**不要再用「算法重写」**——DFC 证明这是绕圈。**聚焦 production 自身的并发争用可修复点**（共享可变状态/内存布局/I-cache），这是无损修复（保留单点快）的战场。

### 教训（判错经验，最重要的资产）

1. **不要用「算法重写」解决并发争用**：MC density 树已是「一个对象 + 实例数据」形态，DFC 重写成 C2ME 数据驱动直排只是在 CPU 造了个 600× 慢的「更正确」版本。并发 11× 的战场在 production 自身争用点。
2. **先 benchmark 钉住主导成本再立项**（D26）：闭包化砍 87% 节点遍历只换 ~5% 提速；DFC 立项前应先钉死主导成本（shift_noise 噪声计算），避免把优化方向建在「可消除的分派/引用计数」错误假设上。
3. **正确性达成 ≠ 性能达成**：DFC 逐位对齐达成但性能目标未达——两个指标独立衡量、分别验收。
4. **吞吐 vs 延迟必须分开**：并行性能看「每 chunk 延迟（阶段耗时）」，不是 wall/N 吞吐均值。

### 被推翻假设 / 作废标注（结案）

| 假设 | 状态 | 依据 |
|---|---|---|
| 「DFC 是 11× 良药」（C2ME 式 DFC 编译直排） | ❌ **正式作废** | 每点 600× 慢 + 立论证伪（虚调用非元凶）+ 净作用为负 |
| 「DFC 消除虚调用可解决 11×」 | ❌ **证伪** | 权威 JSON 证实 coordinate 全纯噪声，虚调用只 ~5%，主导为 shift_noise |
| 「DFC 对齐链」（CpuBackend 正确性） | ✅ **保留** | 逐位对齐 9.57e-07 / 2.06e-08 |
| 「生产并发争用的无损修复」（next） | 🔍 **下一课题** | production 单点 0.4μs 快，争用为战场，候选 locFn 连续化/其它共享可变状态 |

## 2026-08-23（追加）：locFn 连续化 A/B 非主导确认——下一真课题=长串行依赖链

> 承接上文「DFC CPU 移植失败定论（作废）+ 转下一真课题（production 并发争用无损修复）」。本节为 next 真课题的首个**决定性 A/B**：**locFn 连续化（SERIAL）vs BASE 的 T=1/T=8 放大比持平（10.25× vs 10.03×）→ locFn 连续化不能修复 11× → ❌ Plan A 不做**，确认 scout 的候选判断：**真实主导 = 长串行依赖链（~90 节点/实例）的 load 延迟膨胀**。
> 完整记录：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 五节）+ `locfn-serialization-ab.md`（A/B 代码 + 测量方法）+ `production-contention-scout.md`（scout 勘探，§6 A/B 判据来源）。

### ❌ 一、locFn 连续化 A/B 无效（放大比持平）

| 变体 | T=1 | T=8 | 放大比（T8/T1） |
|---|---|---|---|
| **BASE**（`vector<DF>` 散布堆） | 35.11ms | 352.12ms | **10.03×** |
| **SERIAL**（locFn 连续池 + 索引 + kind switch） | 34.76ms | 356.35ms | **10.25×** |

- **SERIAL 放大比（10.25×）与 BASE（10.03×）持平**——locFn 连续化**不能修复 11×**。
- 单线程 T=1 仅微降（34.76 vs 35.11ms）——distinct locFn 只 4-6 个 + L2 热，指针追逐绝对成本本来就小。
- **判读**（scout §6 判据）：SERIAL 放大比未向 DFC 的 1.3× 靠拢 → **A（指针追逐）非主导** → Plan A 不做。

### ✅ 二、确认真实主导 = 长串行依赖链

- **真实 11× 主导 = 长串行依赖链（~90 节点/实例 + 嵌套 spline 跳转）的 load 延迟膨胀**（只读共享广播，非 cache-line 写乒乓）。
- locFn 连续化只消除「每节点 ×1 次 L2 命中 deref」（A 类），不改变依赖链（B）/I-cache（C）——**放大比不降即证明 A 非主导**。
- scout 的候选判断（locFn 大概率非主导）**由最小 A/B 实证确认**——把「locFn 是 11× 主导」这条打 ❌。

### 🔍 三、下一真课题：深挖长串行依赖链

- locFn 连续化（Plan A）**不做**作为 11× 主修复；转长链方向：
  - 提升 MLP / 打破依赖链形态（预取、分块、减少每级数据依赖）；
  - I-cache 争用（C 类叠加）；**不再算法重写**（DFC 教训）。
- 目标：**保留单点 0.4μs 快，修并发 11×**（无损修复）。

### 教训（判错经验）

1. **最小 A/B 验证主导——DFC 教训成功应用**：DFC「静态推断（虚调用元凶）→ 大投入 → 失败」；本次先用最小 A/B（SERIAL 连续化，BASE 不变）在落地前钉死「locFn 非主导」——投入极小、风险极低、结论清晰。**`先钉死主导再动` 有效（MT11 教训 4）。**
2. **放大比 = 11× 判据，非绝对耗时**：SERIAL 绝对耗时微降 ≠ 修好 11×（放大比持平），并行性能看每 chunk 延迟的放大比（T8/T1）。
3. **隔离变量才有可信判据**：只改 locFn 存储、其余不动，才能从 A/B 隔离 A 的贡献。

### 记录指引（知识库归口）
- 根因/结论：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（2026-08-23 五节）。
- A/B 代码/测量：`.investigations/worldgen-mt-scaling/locfn-serialization-ab.md`。
- scout 勘探（§6 A/B 判据来源）：`.investigations/worldgen-mt-scaling/production-contention-scout.md`。

### 被推翻假设 / 作废标注
- ❌ **「locFn 连续化是 11× 主导 / 无损主修复」**——证伪（放大比持平，A 非主导）。locFn 连续化本身仍是有独立价值的低风险小优化（绝对耗时微降），仅不作为 11× 主方案。


---

## 2026-08-23 / 08-24（追加）：production density 并发 11× 争用定位 —— 排除法收敛到「interp/noodle 采样内部」，归因 latency QoS

> **承接**：2026-08-23「locFn 连续化 A/B 非主导」（10-timewise L1697-1739）。上一节把「locFn 连续化（Plan A）」从 11× 主修复除名，真实方向指向「长串行依赖链」。本节在其基础上，**用 production 模型（conc_density_probe + wg_fill_blocks_multi 填 chunk 线程池）做完整排除链**，逐项排除「存储/递归/虚分派/buildGrid/顶层包装虚分派」，最终把 11× 争用**收窄到 interp/noodle 采样内部**，并由 scout 访存分析定论 = **长串行依赖链 + 内存子系统 latency QoS**（每级 load 结果喂下一级，8 线程灌入共享内存子系统排队 → 每级延迟非线性膨胀）。
>
> **统一口径**：所有「放大比」= `median density(T8)/median density(T1)`，12 固定 chunk + WG_PHASETICK（QPC 单次，AGENTS.md 测量污染铁律），禁 WG_PROFILE/WG_STAGETIMER（并发污染）。
> **关键前提（既有）**：production density 单 chunk T=1 ~39ms → T=8 ~331ms = 8.4×（单 chunk 9.2× / density 11×）真实；纯 noise 1.07×（无争用）；「11×」= 放大比 median(T8)/median(T1)。**单点 0.4μs·快**（thread_local grid 懒建 + 每点纯 trilinear），并发才是问题。

---

### 一、起点与背景（为什么进入这条线）

- **Dfc 失败定论**（前 session）：DFC C# 移植（CpuBackend split 预拆分）失败 = 600× 慢，且是「实现（split=GPU 设计）错」不是「直排方向（去递归/去虚调用/去寻址）错」。scout 明确「DFC 直排仍对，只是 split 实现错」。
- **用户拍板**：研究 production 并发争用的**无损修复**（保留单点 0.4μs 快，修 11× 并发；**不是 DFC**——它 600× 慢，CPU 上不可行）。
- **教训**：不要在「算法重写」上立项，除非先钉死主导成本（DFC 是绕圈）。

---

### 二、试验 1（SERIAL）：locFn 存储连续化 —— ❌ 存储非争用

**为什么做**：scout 候选 A = 散布堆 locFn 指针追逐（每 spline 节点 shared_ptr deref + 虚调用）。想验证「locFn 存储连续化能否解 11×」。

**怎么做**：`density.h` SplineDF 加 SERIAL 路径（`WG_SERIAL_LOCFN` env）：`locationFunctions` 从 `vector<DF>`（shared_ptr 散布堆）→ 按类型连续池（flatCachePool/cache2dPool/binopPool 实体）+ `LocFnRef{kind,index}`。`sampleNode` 经 `sampleSerialLocFn`（kind-switch 选池）+ 用 `static_cast<const DensityFunction&>(pool[i]).sample()`。**保留递归 + 虚调用 + thread_local grid 缓存 + registry 共享 cacheId**（只去 deref + 池连续）。

**数据**（conc_density_probe 12 固定 chunk）：BASE 10.03× / SERIAL 10.25×（T1 只微降 35.11→34.76，<1%）。

**结论**：SERIAL ≈ BASE（10.25× vs 10.03×，持平）→ **locFn 存储布局非争用**（A 排除）。

**⚠️ 教训①（本次最重要）**：SERIAL 的 `sampleSerialLocFn` kind-switch 后 `static_cast<const DensityFunction&>(pool[i]).sample()` 转回基类引用 → **仍是虚调用**！所以 SERIAL 只去掉「shared_ptr deref + 存储连续化」，**从未去虚分派**。A/B 只能证明「存储非争用」，**不能**证明「虚分派非争用」。**`static_cast<const DensityFunction&>(obj).sample()` = 强制虚调用，不是去虚调用。**

---

### 三、measurement 修正：production SplineDF 不是深链（浅而宽）

**为什么做**：早期「深链递归」推测需要实测确认（scout d5bb8c50「测量生产 SplineDF 树结构」）。

**数据**：production SplineDF 是**浅而宽**——递归深度仅 3 边 / 4 级，但节点多（factor 135/node、offset 254/node，共 ~433 节点，表 13.8KB）。**非深链** → 早期「深链递归」推测修正。

**机制（scout 测量确认）**：无跨实例 SplineDF 嵌套（coordinate 全解析为噪声/二进制）；真实争用 = 每采样点长 DF wrapper 虚调用链（InterpolatedDF.grid→blend_density→...→spline，15-20 层）+ spline 宽递归 → 8 线程灌同一缓存层级 → 每级延迟膨胀（15.8→190μs）。

---

### 四、对照确认差异在 spline/wrapper 链：noise vs density

**为什么做**：确认「11×」差异来自 spline/wrapper 链（不是 noise 或公共基础设施）。

**怎么做**：conc_sample_probe（density/noise 单点对照）。

**数据**：noise 1.07× vs density 8.4× → **差异在 spline/wrapper 链的 load 争用**（noise 无争用）。

---

### 五、试验 2（NOSPLIT）：spline 递归 → 显式栈 —— ❌ 递归非争用

**为什么做**：scout 候选 B = spline 递归串行依赖链（latency-bound）。想验证「去递归能否解 11×」。

**怎么做**：SplineDF 加 `sampleNodeStack`（递归→显式栈 128 帧）+ `WG_DFC_NOSPLIT` env。保 production 表（nodes/locations/derivatives/subIdx）+ locFn 虚调用。

**数据**：NOSPLIT T1 34.91 → T8 345.78 = **9.9×** vs BASE **10.38×**（持平）。

**结论**：**去递归无效**（递归非争用）。NOSPLIT 保留了 locFn 虚调用 + wrapper 链虚调用（未动）。

**⚠️ 教训②**：NOSPLIT/SERIAL 都**没去「虚调用本身」**——递归和存储都改了，但虚调用还在。虚调用是剩余候选。

---

### 六、试验 3（DEVIRT）：去 spline.locFn 虚分派 —— ❌ locFn 虚分派非争用

**为什么做**：剩余候选 = 虚调用。先隔离 spline.locFn 虚调用（次要那份）。

**怎么做**：先改 `sampleSerialLocFn` 去掉 `static_cast<const DensityFunction&>`（3 case），具体类型直接调 `.sample()`（by-value 池，语义保证 devirtualize，O2）。env `WG_SERIAL_LOCFN=1`（DEVIRT）。**这是对教训①的直接修正**（去掉转基类引用 cast）。

**数据**（conc_density_probe，12 chunk）：BASE 33.54/346.26 = **10.32×** / DEVIRT 34.03/342.06 = **10.05×**。

**结论**：DEVIRT ≈ BASE（10.05× vs 10.32×，降 2.6% 噪音内）→ **spline.locFn 虚分派非争用**。① 排除。

---

### 七、wrapper 链隔离 —— 决定性转向主靶

**为什么做**：做了①（spline.locFn 虚分派）无效，但怀疑主靶是 wrapper 链。要隔离 wrapper 链。

#### 7.1 探针实现（worker 交付，先 scattered 失真）
- `density_builder.h` `getSplines()/splineCount()` + SplineDF 捕获；`worldgen_api` `wg_sample_spline`（直接采样单 SplineDF，绕 wrapper）；`conc_sample_probe` spline 模式；build.ps1 加 `conc_sample_probe`。

**⚠️ 教训③（探针失真）**：conc_sample_probe spline 模式初用 scattered 坐标（`x=3200+(i*17)%2048`，跨 128 chunk）。spline 的 locFn（FlatCacheDF）grid 按 chunk 懒建，scattered 坐标 → **每换 chunk 重建 grid** → per-sample = 440552ns（0.44ms），**比 production 慢 1000 倍，完全失真**（grid 重建主导，非生产路径）。

#### 7.2 修正：固定同 chunk（grid 命中）
**为什么**：生产 fillOneChunkCore 是「同 chunk grid 命中」访问模式。改 conc_sample_probe 固定 x,z 同 chunk（3200-3215/3224-3239）、y 扫 → grid 命中 → per-sample 4493.5ns（快 98×）→ 可靠。

**spline 并发放大**（conc_sample_probe，std::thread，固定同 chunk）：[0] 1.22× / [2] 1.21×。

**⚠️ 教训④（线程模型混淆；关键）**：conc_sample_probe 用 **std::thread**（各线程独立循环），production 争用（10.32×）用 **wg_worker pool**（wg_fill_blocks_multi 填 chunk）。**线程模型不同** → spline 1.2× **不能**独立证明「spline 在 production 下无争用」（std::thread 下多入口都低放大：noise 1.15×/spline 1.2×，可能 std::thread 本身无争用）。**spline 1.2× 仅作辅证**。

#### 7.3 决定性：WG_SPLINE_FILL（production 模型严格对照）
**为什么**：消除线程模型混淆，用 production 线程池（wg_fill_blocks_multi）测 spline 绕 wrapper。

**怎么做**：worldgen_api.cpp `fillOneChunkCore` 加 `WG_SPLINE_FILL=which` → density 采样绕 wrapper，直接 `spl[which]->sample(fpos)`（production 线程池）。

**数据**（conc_density_probe，同一探针/线程池，只差 wrapper）：

| | T1 | T8 | 放大比 | 占时间 |
|---|---|---|---|---|
| 全 tree（含 wrapper） | 33.54 | 346.26 | **10.32×** | 100% |
| spline-only[2]（绕 wrapper） | 3.015 | 4.895 | **1.62×** | 9% |

**结论（决定性）**：wrapper 链把 1.62× 拉到 10.32×（6.4× 放大贡献）+ 占 91% 时间 → **wrapper 链是主争用**，spline 自身几乎无争用。

**⚠️ 还要**：wg_sample_density（whole tree 单点）**无 grid 缓存（每点 buildGrid 6ms）→ std::thread 20000 点 120s 超时**（探针入口需 grid 缓存）。

---

### 八、warm vs cold（区分 buildGrid vs 顶层逐点）

**为什么做**：wrapper 链分「buildGrid 深链（每 chunk 首点一次性）」vs「顶层逐点包装（98304 点 × 虚调用）」。要区分谁主争用。

**怎么做**：fillOneChunkCore 加 `WG_WARM_GRID=1` 预建 grid（对 chunk 中心点调 finalDensity->sample 触发懒建），排除 buildGrid 深链，只剩顶层逐点包装。

**数据**：cold（含 buildGrid）10.32× / warm（排除 buildGrid）10.10×。

**结论**：warm ≈ cold（差 0.22×）→ **buildGrid 深链无碍**。

**⚠️ 教训⑤（修正 scout）**：scout（83c9d1b0「勘探 buildGrid 链虚调用结构」）断言「buildGrid 深链=91% 主争用，顶层逐点每层浅、次要」**有误**。warm 证明 buildGrid 无碍；顶层逐点包装才是主争用。

---

### 九、scout 顶层 wrapper sample 逻辑（7e49cc07）

深挖 finalDensity 顶层：`min(squeeze(mul(0.64, InterpolatedDF#1)), noodle)`。
- a 链（terrain）= BinaryOperation(MIN) → UnaryOperation(SQUEEZE) → LinearOperation(MUL,0.64) → InterpolatedDF#1（唯一 terrain 插值）→ 其下 arg=blend_density(add(...))。
- 每点虚分派：a 链 **4 虚分派/点**（MIN、squeeze、mul、interp#1），3 层有计算。98304 点 × ≈80万-150万次。
- **纯委托层**（BlendDensityDF/WrappingDF/LazyRef）**全部在 InterpolatedDF 网格之下（buildGrid 冷路径）**→ **温暖 per-point 链零纯委托层** → 只剥纯委托对 11× 收益≈0。
- **最小改法（scout candidate）**：数据驱动化温暖 a 链 min/squeeze/mul → a 链每点 4→2 虚分派。**量级 = candidate（需实测）**。

---

### 十、试验 4（WG_FLAT_TOP）：数据驱动化 4→2 虚分派 —— ❌ 虚分派数无碍（最终排除）

**为什么做**：验证 scout candidate「数据驱动化 min/squeeze/mul 降 11×」。

**怎么做**：worldgen_api.cpp 3 处 edit：
1. WorldgenHandle 加 `FlatTop` 成员（enabled/mul_c/interp/b/bmin）。
2. wg_create dynamic_cast 识别 `finalDensity == BinaryOperation(MIN,[UnaryOperation(SQUEEZE,[LinearOperation(MUL,c, interp)])], b)` → 存 flatTop（mul_c=0.64、interp、b、bmin）。
3. fillOneChunkCore 加 `WG_FLAT_TOP` 分支：`double da = applyUnary(SQUEEZE, mul_c * interp->sample(fpos)); fd = da < bmin ? da : std::min(da, b->sample(fpos));`

**逐位一致依据**（与生产 sample 同算术）：mul=`x*c`（LinearOperation L71）、squeeze=`applyUnary(SQUEEZE)`（L165 clampD(x,-1,1)/2 - clampD^3/24）、min=`da<bmin?da:min(da,b->sample)`（BinaryOperation L129）。

**数据**：生产 10.32× / WG_FLAT_TOP 10.55×。

**✅ 对拍通过**：用 block_probe `-save`（WG_FLAT_TOP=0/1 同参照 `vanilla_8576294172403134396_6_720_-432.blocks`），`out_prod.bin` vs `out_flat.bin` **SHA256 完全一致（identical: True）** → WG_FLAT_TOP **逐位一致**（保正确）。

**结论（关键负面）**：WG_FLAT_TOP ≈ 生产（10.55× vs 10.32×，持平甚至略高）→ **减少虚分派层数（4→2）不降 11×**。scout 的「数据驱动化 min/squeeze/mul 降 11×」candidate **被证伪**。**11× 争用不是虚分派层数多导致**。

**⚠️ 教训⑥（纪律）**：改生产路径（WG_FLAT_TOP）后**必须 block_probe 对拍（SHA256 identical 确认逐位一致）才下性能结论**——同算术理论一致但需实证；本次对拍在此负面结论之前完成，保证「减少虚分派不降 11×」结论可信。

---

### 十一、排除链汇总（全部 production 模型 = conc_density_probe，同一探针/线程池，可靠）

| 试验 | 改动位置 | 改动 | 放大比 | 结论 |
|---|---|---|---|---|
| BASE | — | — | 10.32× | 基线 |
| SERIAL | spline.locFn 存储 | locFn 存储连续化 | 10.25× | ❌ 存储非争用 |
| NOSPLIT | spline | 递归→显式栈 | 9.9× | ❌ 递归非争用 |
| DEVIRT | spline.locFn | 虚分派 devirtualize | 10.05× | ❌ locFn 虚分派非争用 |
| spline-only | 绕 wrapper | 直采 spline（WG_SPLINE_FILL） | 1.62× | spline 无碍 |
| warm | wrapper buildGrid | 预建 grid 排除 buildGrid | 10.10× | ❌ buildGrid 无碍 |
| **WG_FLAT_TOP** | 顶层 wrapper | 去 min/squeeze/mul 虚分派（4→2，逐位一致） | 10.55× | ❌ **虚分派数无碍** |

⇒ **11× 争用 = interp/noodle 采样内部**（内存访问模式），**非** 虚调用数、buildGrid、spline、min/squeeze/mul 虚分派、存储、递归。

> 注：BASE 基线在早期 SERIAL A/B 为 10.03×（另一 run），本轮权威基线 = 10.32×。各 A/B 均保留**各自同步的 BASE 对照**；合并排除链用 10.32× 为准。

---

### 十二、scout 访存分析（dcf85758，interp-memory-access.md）—— 排除带宽/SMT，定论 latency QoS

**确证（源码行号）**：
- interp grid **thread_local**（density.h:576-578），跨线程独立，**不共享**。
- interp#1 命中后每点读 **8 角点 double（64B）+ 3 lerp，0 虚调用**（L537-548）。grid = 5×49×5=1225 ×8B=**9800B**/实例/线程。
- noodle 内层 = **InterpolatedDF 包 range_choice 包 noise**（非 InterpolatedNoiseDF old_blended_noise——**更正任务标注 @anchor.idk**）。每点最多 **32 角点/256B** grid 读（thread_local），RangeChoice + interp#A/B/C/D。
- 跨线程共享**全为只读 const**（noiseSamplers/SplineDF 表 17KB/GRADIENTS 192B/finalDensity 节点字段），**无写共享/ping-pong**。
- 机器 **12 物理核/24 逻辑**；pool 默认 = `physicalCoreCount()`=12 物理核；**无 SetThreadAffinityMask/pinning**。**T=8 ≤ 12 物理核 → 各占独立物理核，不触发 SMT**。

**判断（推断，@anchor.idk，需 M3 钉死）——排除 带宽/SMT**：
- **C7 内存带宽**：并发 540MB/s = DDR **1-2%** → 带宽远未饱和。
- **C4/C2 SMT**：T=8 ≤ 12 物理核无 core 共享；频率归一化后 10× 远超 SMT 理论上限(~1.5×)。
- **共享读便宜**：noise 1.15×、spline-only 1.62×（都读共享 const）→ 共享读本身不是 10× 放大器。
- **最一致机制 = 长串行依赖链 + 内存子系统 latency QoS**：每点链（interp#1 grid 8 读 → noodle range_choice → interpA(8) → out_range interpB/C/D(24) → 各级数学）**每级 load 结果喂下一级**（数据依赖）；8 线程灌入长链 → 共享内存子系统排队 → **每级 load 延迟非线性膨胀** → 链延迟 ~10×。与「无锁 + 读共享 const + 真并行 + 单 chunk 膨胀 10×」自洽。**是延迟（latency）非吞吐（throughput）被共享资源排队放大**。

**⚠️ 关键区分**：这是**延迟 QoS**（latency，每级 load 排队放大），**不是**吞吐带宽饱和（C7 已否）、**不是**写乒乓（全只读）、**不是**虚调用、**不是** buildGrid/spline/存储/递归（已排除）。grid 全 thread_local + 共享读全 const + 只读无写 → 三者与「无锁+读共享+真并行+膨胀10×」自洽。

**可测量方法（scout 推荐执行序）**：
- **M3【决定性】interp-only grid-hit 隔离**：conc_sample_probe 加 interp-only 模式（预建 grid，只测 8 角点读 + 3 lerp），T=1 vs T=8。**低 → 争用不在 grid 读，在长链依赖（latency QoS H3）；高 → 在 InterpolatedDF 机制本身**。最便宜最判别。
- M1（pin 物理核）/M2（per-thread perm 副本）——大概率确认否定（与 C2/C4/C7 一致）。
- M4（MLP 提升，并行多独立点链段）——M3 显示长链主导时验证。

---

### 十三、M3 interp-only 探针 —— 执行遇阻（wg_sample_interp 采样慢，未干净隔离 trilinear）

**实现**（5 处编辑）：WorldgenHandle 加 `interpTop`（Dynamic_cast 捕获 a 链 InterpolatedDF#1）+ `wg_sample_interp(handle,x,y,z)`（worldgen_api.h/.cpp）+ conc_sample_probe `interp` 模式（固定同 chunk 坐标，wg_sample_interp 采样）。

**探针故障链路（详细）**：
1. **初版 interp 模式 N=20000 超时（120s）**。
2. **诊断 N=5**：每采样 **1.1s**（wall 5.5s）——interp#1->sample 极慢。
3. **根因假设**：wg_sample_interp **未设 g_curChunkX/Z**（InterpolatedDF 懒建 grid 的 buildGrid 怪物树里，FlatCacheDF/Cache2DDF 的 grid/缓存 key 依赖 g_curChunkX/Z；fillOneChunkCore 的 CurChunkGuard 会设，wg_sample_interp 不设则它们回退 pos>>4 推导，逐点/跨 y 反复重建 → 慢）。
4. **修复：wg_sample_interp 设 g_curChunkX = x>>4, g_curChunkZ = z>>4**（仿 CurChunkGuard，RAII 恢复）→ N=5 per-sample **5.9ms（快 187 倍）**，含 interp#1 buildGrid（怪物树建 grid ≈ 25ms，production density 的大头）。
5. **N=20000**：per-sample **292μs**（wall 5847ms）——**仍比 production 0.34μs/点慢 850×**。

**结论 / 遇阻**：wg_sample_interp 未干净隔离到「grid 命中 trilinear」——per-sample 292μs 远高于预期的 trilinear（<1μs），可能是每次采样重建 grid 或 buildGrid 摊薄不足。**M3 探针未能干净测「interp#1 grid 命中」的并发放大**，latency QoS 假说**未直接验证**（需修探针或另法）。

**已有数据（间接指向 latency QoS）**：
- warm（production 预建 grid，去 buildGrid）10.10× → buildGrid 无碍
- spline-only 1.62×（绕 wrapper+interp+spline）→ 绕全部后低
- WG_FLAT_TOP（去 min/squeeze/mul）10.55× → 虚分派数无碍
- → 争用集中在 **interp#1 trilinear + noodle 长链**（非 buildGrid/spline/虚分派），与 scout 的「长串行依赖链 + latency QoS」一致（但未经 M3 直接证实）。

**M3 探针诊断进展（更新）**：
- **N=1**：wall 27.9ms → interp#1->sample 单次 = **buildGrid 怪物树 ≈27.9ms**（production density 大头）。
- **N=20000**：per-sample 292μs = (27.9ms + 19999×hit)/20000 → **hit ≈ 291μs/采样**。
- **矛盾（探针 bug 铁证）**：production 33ms/chunk 含 98304 点（interp#1 hit + noodle + min/squeeze/mul）→ 每点仅 **0.34μs**；wg_sample_interp 的 hit（291μs）**比 production 慢 850×**。同 chunk 的 interp#1 trilinear（8 角点 grid 读 + 3 lerp）不可能 291μs。
- **结论**：wg_sample_interp 命中慢 850× 是**探针自身 bug**（非 11× 机制）。候选根因：① thread_local slots 每采样 resize/allocator 行为；② 坐标覆盖 256 个不同 (x,z) cell 的 cache 局部性；③ g_curChunk 设置引入的额外路径。**需 perf 分析钉死**（探针调试，非 11× 机制）。

> **教训**：interp#1->sample 单点即触发 buildGrid（怪物树 27.9ms）——探针测「hit」必须先预建 grid；且 wg_sample_interp 的 hit 慢 850× vs production，探针自身需 perf 调试（thread_local slots/坐标/allocator）。

---

### 十四、结论汇总（归档时核对，状态标注）

| 结论 | 状态 | 依据 |
|---|---|---|
| **排除链**：存储/递归/虚分派/buildGrid/顶层包装虚分派均非 11× 争用 | ✅ **production 模型确证级**（同探针 conc_density_probe ±10×，各 A/B 保留同步 BASE 对照） | 11x-contention-log §10 / wrapper-chain-measurement §8 |
| **争用收窄到 interp/noodle 采样内部** | ✅ **确证级**（排除法收敛） | §11 排除链 |
| **11× 归因 = 长串行依赖链 + 内存子系统 latency QoS** | 🔍 **candidate/推断**（@anchor.idk，需 M3 干净验证） | 12x/12 节 scout 访存分析 |
| M3 探针（wg_sample_interp）自身 bug（hit 慢 850×） | 🔍 **已记录，需 perf 定位**（探针调试，非 11× 机制） | §13 |

**修复方向（latency QoS 下）**：**提升 MLP**（打破长依赖链：并行多独立点/DFC 式全扁平直排/软件流水），**不是**减虚调用/存储/递归（已排除）。**但注意**：DFC 式全扁平直排在 CPU 上已证 600× 慢（净作用为负），故「提升 MLP」需在 production 自身形态上做（保留单点 0.4μs 快），**不是算法重写**。

---

### 记录指引（知识库归口）
- 主过程日志（最全）：`.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`
- 主测量记录：`.investigations/worldgen-mt-scaling/wrapper-chain-measurement.md`（§6 spline-only / §7 warm-cold / §8 WG_FLAT_TOP + 对拍）
- scout 访存分析：`.investigations/worldgen-mt-scaling/interp-memory-access.md`（dcf85758）
- scout buildGrid 结构：`.investigations/worldgen-mt-scaling/wrapper-buildgrid-structure.md`
- scout 顶层 wrapper 逻辑：`.investigations/worldgen-mt-scaling/topwrapper-sample-logic.md`
- 历史 11× 机制：`.investigations/worldgen-mt-scaling/density-latency-rootcause.md`
- SERIAL A/B：`.investigations/worldgen-mt-scaling/locfn-serialization-ab.md`
- locFn 非主导勘探：`.investigations/worldgen-mt-scaling/production-contention-scout.md`
- 错误台账（新增 ①-⑥）：本目录 `draft-mt-errors-11x.md`

---

## 2026-08-24（追加）：Rust worldgen 重写 density_builder 完成 + 逐位对齐 C++ buildNode（✅ 关键里程碑）

> CoreSwap worldgen 正在全量重写为 Rust（WorldgenRust/）。本节记录 `density_builder.rs` 产物：buildNode 全分派 + mn/mx + lazyRef，使 Rust 能把 overworld JSON 构建成密度树，并验证与 C++ `density_builder.h`（rust_ref_check）逐位一致。配套：03 篇「Rust 重写 buildNode 对齐 C++」结论小节 + `.investigations/rust-density-builder/` + `rust-errors.md` 错误台账（R1-R4）。

### ✅ 一、Rust 重写 density_builder 完成 + 对齐 C++（关键里程碑）
- **16 个 overworld 密度函数**（10 顶层：base_3d_noise/continents/erosion/ridges/ridges_folded/factor/offset/jaggedness/depth/sloped_cheese；+ 6 caves/*：entrances/noodle/pillars/spaghetti_2d/spaghetti_2d_thickness_modulator/spaghetti_roughness_function）× **10 采样点（160 值）** + 各函数 min/max，与 C++ `rust_ref_check` 输出**逐位一致（规范化差分=0）**。
- **数据驱动实现**：Rust 用 `enum DensityFunction`（match 全分派），等价 C++ 多态虚调用 DF 树（无虚调用/无指针追逐）。新增变体：ShiftDF/ShiftedNoise/RangeChoice/YClampedGradient/WeirdScaled/BlendAlpha/BlendOffset/BlendDensity/Wrapping/InterpolatedNoise/Lazy。
- **对齐基准**：C++ `density_builder.h` buildNode（不含 Beardifier，见域边界）。

### ✅ 二、noise_params.json 读取（对齐基准从硬编码表切到权威文件，judge P2-e 收口）
- **对齐基准切换**：噪声参数不再用硬编码表，改为读权威 `noise_params.json`——judge P2-e 收口（对齐基准单一事实源化）。
- 意义：消除了「参数表与权威 JSON 漂移」这一潜在对齐差异来源（噪声参数是 octave/振幅，漂移会整树错位）。

### ✅ 三、完整 finalDensity 端到端（Rust 构建 noise_router.final_density 整树，与 C++ 逐位）
- Rust 读 overworld.json 构建 `noise_router.final_density` **整树**，10 点 + min/max 与 C++ **逐位一致**（min=-0.45833333, max=0.45833333）。
- 验证分层 = **Full**（逐位），seed = 8576294172403134396。

### 🔍→❌→✅ 四、块级 y-column 填充对比参照坑（错误台账 R1——被推翻假说）
**现象**：块级 y-column 填充（chunk(45,-26) row(8,8)→(728,-408) 列 384 点）与历史参照对比出现差异，一度疑似 Rust bug。
**根因（机制）**：**参照文件配置错误，非 Rust 代码错**——历史 `cpp_density_*` 参照**含 Beardifier**（属完整 worldgen 配置），而对拍目标 buildNode **不含 Beardifier**；二者在结构附近差 ~0.015。
**定位（诊断方法）**：经**当前 C++ 重编译的 rust_ref_check** 对拍排除 Rust bug；改用**当前 C++ 列 dump** 作参照。
**修复**：对齐参照切到当前 C++ 列 dump → **384/384 一致**，maxDiff=3.58e-9。
**教训（判错经验）**：**跨实现/跨版本对拍必须同时确认「参照的配置语义」与「目标的配置语义」一致**——参照含 Beardifier 而目标不含时，结构附近的差异是配置差，不是实现 bug；「参照错位」应先于「实现 bug」被排除。

**❌ 被推翻假说**：「Rust 插值 / range_choice 在 (-40,240) 有 bug」——**证伪，❌**；真实因 = 参照文件配置（含 Beardifier）。后续对拍一律以当前 C++ 重编译的 rust_ref_check + 当前 C++ 列 dump 为参照，**不再沿用 `cpp_density_*` 历史文件**。

### 🧰 五、工具演进（本轮新增）
- **rust_ref_check**（C++ 参照，cl 直链）：对外对拍的 C++ 权威参照。
- **overworld_probe.rs**（Rust 探针）：overworld 密度函数层探针。
- **finaldensity_probe.rs**（Rust 探针）：final_density 整树端到端探针。
- **chunkfill_probe.rs**（Rust 探针）：块级 y-column 填充探针。

### 📌 记录指引（知识库归口）
- 错误台账：`.investigations/rust-density-builder/rust-errors.md`（R1 参照坑五段式 + R2/R3/R4 对齐 bug 五段式）。
- 结论：03 篇「Rust 重写 buildNode 对齐 C++」小节。
- 过程：本节 + `.investigations/rust-density-builder/`。
- **域边界（保持）**：align = C++ buildNode，不含 Beardifier（`@anchor.idk`）；vanilla 逐块对齐未做。
