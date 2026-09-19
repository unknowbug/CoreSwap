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
6. ✅ **CppBridge 诊断增强**（~~保留~~ **已失效**，260918-06 注记）：-Dcpp.noBatch env 兜底 CORESWAP_NOBATCH——消费已随 C++ 归档消失，映射行已删（B6-1）

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

---

## 2026-08-29 Rust CARVERS 阶段移植（✅ 已结案）

**背景**：Rust 全量重写 worldgen 的 CARVERS 阶段（洞穴雕刻）。把 C++ carver.h（661 行，CaveCarver+RavineCarver）移植到 Rust（commit bf3d851）。

**过程**：
- 新增 chunkrandom.rs（CheckedRandom 48 位 LCG + ChunkRandom CHECKED/XOROSHIRO 分派 + setCarverSeed）
- 新增 carver.rs（CarvingMask、YOffset/HeightProvider/FloatProvider、CarverConfig/Cave/Ravine、CaveCarver/RavineCarver、ConfiguredCarver、mathSin/mathCos 查表）
- iome.rs 新增 load_carvers（biome/*.json carvers.air）+ biome_pick_cell（8 邻域 jitter）
- carver_probe.rs 接入块级管线（fill_chunk + build_surface 后 17×17 邻域）

**验证**（对拍 vanilla FULL 参照，seed=-8248318472910187742，4x4 origin -288,-256）：
- 无 carver：match=95.41%；有 carver：match=95.61%，挖洞重合 **90.88%**（5842/6428），0 块挖到地表以上
- 结论：Rust carver 挖洞位置与 vanilla 高度重合，功能完整

**错误台账**：.investigations/carver-port/carver-errors.md（C1-C4，Rust 移植 C++ 的借用/所有权典型坑：E0499 裸指针聚合、E0384 mut 按需、E0502 move 闭包、E0382 &self）

**归口**：07 篇「Rust CARVERS 阶段移植」小节 + 02 篇「CheckedRandom/ChunkRandom」小节。

---

## 2026-08-29 Rust worldgen 作为 mod 运行（✅ 关键里程碑）

> CoreSwap worldgen 全量重写为 Rust（WorldgenRust/）后，把 Rust 块级管线作为 Minecraft mod 运行。三层链路：**Rust cdylib（C ABI）→ C++ JNI 桥（worldgen.dll）→ mod 加载（Java_wg_CppWorldgen_*）**。配套：07 篇「Rust worldgen 作为 mod 运行」结论小节 + .investigations/rust-mod-load/ + 
ust-mod-errors.md 错误台账（M1-M4）。

### ✅ 一、Rust 块级管线封装 + C ABI（关键里程碑）
- worldgen_handle.rs：WorldgenHandle::create + ill_chunk_blocks（fill_chunk 宏观 → BlockColumn → build_surface → carver 17×17 邻域）。
- pi.rs：C ABI 导出 wg_create/wg_destroy/wg_fill_blocks_multi/wg_set_beardifier/wg_clear_beardifier/wg_density_*；Cargo.toml crate-type = ["cdylib", "rlib"]。
- wg_fill_blocks_multi 当前**串行生成**（裸指针跨线程 Send 问题，错误台账 M2）。

### ✅ 二、C++ JNI 桥（rust_jni_bridge.cpp → worldgen.dll）
- 加载 Rust WorldgenRust.dll（LoadLibrary + GetProcAddress 取 wg_*），导出 Java_wg_CppWorldgen_init/destroy/fillBlocks/setBeardifier/fillDensity/densityParams 六个 JNI 函数。
- JNI 桥 = **薄转发层**（JNI 数组 ↔ C 指针转换 + 调 wg_*），与 C++ jni_bridge.cpp 同构。

### ✅ 三、验证（三层递进）
- dll_test.c：wg_* 导出 OK（C ABI 层）。
- jni_dll_test.c：6 个 JNI 函数导出 OK（JNI 桥层）。
- handle_probe：WorldgenHandle vs vanilla 95.54%。
- **JniProbe（最终验证）**：JNI 加载 Rust dll 生成 64 chunks，match=**93.76%**（y=64..319 100%，地下 71-90%）——air 区 100% 证明桥接正确，地下差异来自 worldgen 已知边界（carver/FEATURE/Beardifier），非 JNI 桥引入。

### 🧰 四、工具演进（本轮新增）
- dll_test.c（C ABI 导出验证）、jni_dll_test.c（JNI 导出验证）、handle_probe.rs（WorldgenHandle 块级管线验证）、JniProbe（JNI 全链路验证）。

### 📌 记录指引（知识库归口）
- 错误台账：.investigations/rust-mod-load/rust-mod-errors.md（M1-M4 五段式）。
- 结论：07 篇「Rust worldgen 作为 mod 运行」小节。
- 过程：本节 + .investigations/rust-mod-load/。
- **域边界（保持）**：验证分层 = Partial（JNI 加载 Rust dll 对比 vanilla FULL 参照）；Rust 块级管线不含 Beardifier（@anchor.idk）；wg_fill_density 暂未实现（返回 0）。

---

## 2026-08-29 FEATURES 阶段移植到 Rust（🔍 功能完成，ore 位置待对齐）

> Rust 把 C++ FEATURES 阶段（feature.h/placement.h/feature_loader.h）移植到 WorldgenRust/src/{placement,feature,feature_loader}.rs（提交 6934ea4）。装饰层（Ore/Disk/Spring/FreezeTop/UnderwaterMagma）。配套：11 篇「FEATURES 阶段 Rust 移植」+ .investigations/features-port/ + eatures-errors.md 错误台账（F-1~F-3）。

### ✅ 一、FEATURES 移植（功能完成）
- placement.rs（IntProvider + 10 PlacementModifier + PlacedFeature.generate 深度优先）
- eature.rs（RuleTest + OreFeature 3D 矿脉 + ScatteredOre/Disk/Spring/FreezeTop/UnderwaterMagma）
- eature_loader.rs（ConfiguredFeature + PlacedFeatureIndexer + FeatureCache 懒加载）
- iome.rs（load_features + all_features_lists）
- worldgen_handle.rs（apply_features 接入，populationSeed + setDecoratorSeed + PlacedFeature.generate）

### 🔍 二、验证（ore 位置待对齐，未闭合）
- features_probe vs vanilla FULL：match≈95.50%（无 features 95.54% 略降 0.04%）
- **ore 放置位置与 vanilla 仅 1/13 匹配**——populationSeed/setDecoratorSeed 随机序列与 Java 不完全一致，**待对齐**（下次先对照 index/p/step 分量）
- freeze_top_layer 因温度 >=0 不冻结（无影响）

### 🧰 三、工具/错误台账
- eatures_probe.rs（对拍）、check_features.py（对比放置位置与 vanilla）
- 错误台账：.investigations/features-port/features-errors.md（F-1 随机源双重借用 / F-2 嵌套 fn 无法捕获 FnMut / F-3 Indexer 单 biome 构建 p 值错）

### 📌 记录指引
- 结论：11 篇「FEATURES 阶段 Rust 移植」。
- 过程：本节 + .investigations/features-port/。
- **域边界（保持）**：ore 位置 1/13 是未闭合已知问题非结论；树花植被范围外；邻域 chunk 方块读取简化。

---

## 2026-08-29 Rust worldgen 整体功能实现（✅ 关键里程碑）

> 从「Rust 块级管线跑通（mod-run）」推进到「FEATURES 功能真正接进生成管线 + 生成路径零锁」。用户明确「先整体功能实现 + 跑测试记录对齐程度，不纠结为什么没对齐」。配套：07 篇「Rust worldgen 整体功能实现」小节 + .investigations/rust-mod-load/ + unctional-errors.md 错误台账（F1-F3）。

### ✅ 一、功能链路完整接入（功能闭环）
- 09d85e8 补 **OCEAN_FLOOR_WG** 高度图（ocean_floor: None→Some）——水下 ore/disk/spring 按海底放置（F2）。
- 79daf17 接入 **ore_vein 矿脉**（vein_toggle/ridged/gap + split("minecraft:ore")；pply 改 &self 只读，F1）。
- a6a53f7 接入 **Beardifier**（eardifiers Mutex→RwLock，写读分离；fill 读 clone 不持锁，F3）。
- 4ac3a00 实现 **wg_fill_density**（finalDensity 网格采样，fillDensity API）。

### ✅ 二、生成路径零锁（perf 优化）
- ed59f50：feature_indexer/carver_cache/feature_cache 全部预加载只读共享；生成路径（fill_chunk_blocks）零锁。

### 🧪 三、功能验证（对齐快照，用户指示只记录）
- features_probe：**match 95.40%** / nonAir 85.84%（整体快照）。
- vein_probe：2295 矿脉块（1849 铜 + 19 生铜 + 427 深板岩铁）。
- fill_density_probe：3072 点全部非零。
- **对齐率只记录不纠结**（用户指令）：数值一次性，不展开差异，排查参考 .investigations/rust-mod-load/cmd-output/。

### 🧰 四、工具演进（本轮新增）
- ein_probe.rs（矿脉接入验证）、ill_density_probe.rs（finalDensity 网格采样验证）、features_probe（完整管线对齐快照）。

### 📌 记录指引（知识库归口）
- 错误台账：.investigations/rust-mod-load/functional-errors.md（F1-F3 五段式 + 速查表）。
- 结论：07 篇「Rust worldgen 整体功能实现」小节（中价值简记 + 低价值对齐快照）。
- 过程：本节 + .investigations/rust-mod-load/。

---

## 2026-08-29 Rust 多世界参数化（🔍 框架参数化完成）

> Rust WorldgenHandle 多世界参数化，对齐 C++ wg_create。从「定死 overworld」到「任意维度加载」（含 MOD 维度如暮色森林）。AGENTS.md 写入「数据驱动架构铁律」（含多世界方向，用户拍板）。

### ✅ 一、create_for_dim 维度参数化
- create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height) 支持任意维度
- dfNs = settings_name 去 .json → density_function/<dfNs>/ + resolve_ref 前缀（set_df_ns，修复 M1 前缀硬编码）
- 维度参数（min_y/height/sea_level/aquifers_enabled）从 settings 读
- SurfaceBuilder::parse_surface_rule JSON 数据驱动（非 overworld surface_rule）
- aquifers_enabled=false（下界）→ VanillaAquifer.enabled=false（修复 M2 字段连锁）

### 🧪 二、验证
- nether：加载成功（min_y=0/height=256）+ chunk(0,0) 56307 非空气块
- overworld 回归 95.40% 不变

### 📌 记录指引
- 结论：09 篇「七、Rust 世界参数化」+ WorldgenRust/data-driven-boundary.md 多世界章节。
- 错误台账：.investigations/multiworld-port/multiworld-errors.md（M1/M2 五段式）。
- **遗留**：fill_chunk_blocks 的 carver/features/ore_vein 仍是主世界逻辑，nether/MOD 维度的生成逻辑差异化（暮色森林接入时精化）。

---

## 2026-08-29 Rust worldgen 端到端性能定位（aquifer 最大头，慢 Java 5 倍）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节 + .investigations/perf-e2e/ + perf-e2e-errors.md 错误台账（P1-P3）。

### 🔍 一、density 方向修正（11f478f，judge 推翻）
- density_tree_profile：finalDensity 3710 节点（无指数膨胀，Spline 仅 9）。
- 原判「Interpolated 632ms → 放弃」被 judge 推翻：632ms 是双层 Interpolated 污染（内层 mesh 跨 chunk 雪崩重建 291×）。
- judge 实测单层 Interpolated 对 SplineDF = 70× 加速（83.74→1.19ms），Interpolated 是密度优化正解。

### 🔍 二、fill_chunk 内部定位（597e8d5）
- classify(aquifer) 43-64% 是 fill_chunk 最大头（有污染但相对占比可信）。

### ❌ 三、诊断代码热路径污染（P2，d9ff1e2）
- AQPROF atomic / Instant::now ×3 / env::var 每点执行（98304 次/chunk）→ 27% 退化（61.5→44.9ms），用户提醒「断点污染」坑后迁移到 chunk 级门控。

### ❌ 四、端到端基准重大修正（P3）
- 早期「Java 60ms」是 JIT 未热错误基准 → 误判 Rust 达标（「积累性差异」担忧）。
- 充分预热后 Java FULL 只要 ~8-9ms/chunk，Rust 44.9ms 慢 ~5 倍。

### ✅ 五、无污染重定位（本轮）
- base（fill_chunk+surface）29.4ms：aquifer 增量 ~17.5ms（60%）> density ~12ms > carver 14ms > surface ~4ms。
- 一个 aquifer（17.5ms）就比 Java 全部（8-9ms）慢 2 倍——优化应聚焦 aquifer。
- aquifer 内部：calculate_density 52%（barrier.sample 无 Cache2D 缓存 + fluid 逻辑）、get_block_pos 3×3 邻域 14%、get_water_level_at 2%。

### 🧰 六、铁律沉淀（用户拍板）
- AGENTS.md 新增「端到端性能对比铁律」（必须端到端对比充分预热的 Java；诊断代码不能放热路径每点执行）。

### 📌 记录指引
- 错误台账：.investigations/perf-e2e/perf-e2e-errors.md（P1-P3 五段式 + 速查表）。
- 结论：07 篇「Rust worldgen 端到端性能定位」小节。
- 域边界：端到端数字 = Partial 快照；优化方向（aquifer barrier 缓存 / 单层 Interpolated / DFC）= candidate 待立项验证。

##### ✅ 五-b、aquifer 内部精确无污染定位（2026-08-29，修正污染态构成）
- **barrier.sample 实测仅 0.1%**（346/393216 = 每 chunk ~86 次）——「barrier 是 aquifer 大头 / 加 Cache2D 缓存」方向被计数类硬证据推翻（错误方向，见 perf-e2e-errors.md P4）。
- **无污染精确构成**（diag 方法）：get_fluid_level 3.84ms（22%）+ get_block_pos 2.57ms（14%）+ get_water_level_at 小 + calculate_density fluid ~0ms → 合计可解释 ~6.4ms，**剩余 ~11ms 未解释 = apply 每点 98304 次调用固定开销**。
- **根本洞察**：Java 宏观 Interpolated 网格（~1225 交点）vs Rust 逐点 98304 次 → 采样差 ~80×（density 段）。
- **❌ 方向修正（judge 否决「宏观网格采样对齐」）**：judge 审查（
eview-fillchunk-grid-alignment.md）确认「80× = 根本」归因错误——真正最大成本是 **aquifer（Java 同样逐块不插值，网格覆盖不到）**；「宏观网格采样」不立项。
- **✅ 更优方向（judge 收益排序）**：① aquifer 每点开销优化（最大头）② 单层 Interpolated 应用于纯 SplineDF 子树（70× 实测）③ DFC 直排。

---

## 2026-08-29 运行环境迁移到 CoreSwap（免提权）+ Rust/Java 性能基准真正确认（大样本推翻「慢 5 倍」）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节修正 + .investigations/perf-e2e/ + perf-e2e-errors.md（P1-P5，本次新增 P5）。

### ✅ 一、运行环境迁移到 CoreSwap（免提权，b2b9bea + 50ba9a4）
- 原运行环境在 MC 侧（E:\PYTHON\MC\versions\1.20.1\java），每次 gradle 运行需 danger-full-access 提权（native-platform.dll 在 C:\Users\NDark\.gradle 外部）。
- 迁移三步：
  1. git mv versions/1.20.1/java → runtime/1.20.1/java（验证 client 独立 runtime，与数据/参考 versions 分离）。
  2. gradle home C:\Users\NDark\.gradle（2.6GB）→ CoreSwap\.gradle（robocopy 秒级），native-platform.dll + 依赖缓存在工作区内。
  3. GRADLE_USER_HOME=CoreSwap\.gradle → gradle classes 编译 + runServer 启动 + bench 探针全免提权。
- bench.out 默认改 CoreSwap；.gitignore 更新；run_rust_client.ps1 设 $runJava=runtime\1.20.1\java + GRADLE_USER_HOME。
- 原理：gradle home 放工作区 → native-platform.dll + 依赖缓存在沙箱可见区内 → 免提权。

### 🔄 二、端到端基准重大修正（P5，推翻「慢 5 倍」）
- 早前「Java FULL 8-9ms → Rust 慢 5 倍」是小样本（16 chunks）+ 相邻 chunk 缓存假象，与 P3（JIT 未热）同族——基准不可靠连续两次。
- 大样本修正（region 200,200）：Java FULL ≈ 55ms/chunk；Java 宏观 NOISE ≈ 23-25ms；Rust 宏观 34.66ms；Rust 全管线 45.48ms。
- ✅ Rust 全管线 45.48 < Java FULL 55 → Rust 反快 ~1.2 倍（「慢 5 倍」不成立）；但宏观专项 Rust 34.66 > Java 23-25 → aquifer 慢 ~1.4-1.5 倍（真差距需优化）。
- ❌ 早前「Rust 慢 5 倍」条目（下方旧 2026-08-29 条目）标注被大样本修正推翻。

### 📌 记录指引
- 错误台账：perf-e2e-errors.md P5（大样本缓存假象）。
- 结论：07 篇端到端小节修正（Rust 反快 / aquifer 宏观慢需优化）。
- 通用模式：knowledge/discovered/build-tooling.md「发现 #4」（gradle home 放工作区免提权）。
- 域边界：数字 = Partial 快照（随优化变化）；aquifer 宏观优化 = candidate 待立项。

---

## 2026-08-30 Rust 宏观采样重构（multi-channel 竖切）+ 性能定位（candidate）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节 + `.investigations/macro-layer-scout/`。本轮从「性能定位」深化到「宏观采样层重构正确性 + 性能归因修正」。

### ✅ 一、顶层确认（宏观采样的真正顶层 = NoiseChunk cell grid）
- `macro-layer-topness.md`（reader 级）完整列出调度链（pyramid → tasks → noise stage → fill_from_noise）与采样机制层（`NoiseChunk::fill` + `fill_slice_into` + trilerp + combine），**其上无采样机制层**。
- 6 个疑似上层（blending/StaticCache2D/Beardifier/aquifer/dim settings/ColumnCache）逐一确认非采样层，均给源码落点。
- 与 Rust 52× 雪崩根因互证：`terrain.rs` L134-143 注释 + `macro_layer_map` §3.2——「对 final_density 采样 corners 触发内部 interpolated 雪崩」。
- **结论**：① 可靠，reader 级，**judge 确认，建议 candidate**。

### 🧪 二、multi-channel 竖切重构（正确性 diff0，但生产未接线）
- `density.rs macrolize_channels/macrolize_into`（L604-681）：final_density → 5 channels（1 BlendDensity terrain + 4 RangeChoice noodle），combine 树 Interpolated→ReadChannel 全部替换，`macrolize_probe` 验证残留 Interpolated=0、ReadChannel≥1。
- `DensityMacroSampler` diff0（n=54，平均差异 0.000000）——**局部充分非全局**（只测 cell 边界平面 fx/fz=0，未覆盖 cell 内部/边界 clamp/负 Y/跨 cell）。
- **⚠️ 关键**：`DensityMacroSampler` **只在探针文件**定义；生产 `terrain.rs fill_chunk` **未接线**（仍逐点）。「重构完成」表述须限定为「探针层验证完成，生产接线未做」。
- 标量性能：slices 构建 8.52ms + trilerp 0.3ms = 8.83ms vs 逐点 6.43ms（不省）；`std::simd` stable 不可用（需 nightly/intrinsics）。

### 🔍 三、性能定位链（corners noise 89% / 全管线 aquifer 大头）
- corners 采样：ch#0（BlendDensity terrain, 3677 节点）3.60ms 绝对大头（1225 corners）；ch#1-4（noodle 小）合计 ~0.4ms。
- `tree_vs_noise`：ch#0 完整 3.34ms → 去 noise 0.38ms → **noise 采样 2.97ms（89%），树遍历 0.38ms（11%）**——修正早前「树解释器大头」判断（milestone_record 相反，以 tree_vs_noise 为准）。
- **两测量域不矛盾**：corners noise 89% = 「宏观 corners 采样内部构成」；全管线 aquifer 大头 = 「宏观 density+aquifer 跨阶段构成」。docs 需显式区分，避免误读「noise 慢=全管线瓶颈」。

### ❌ 四、noise AVX 归因不实（judge 修正，见 multichannel-errors.md M2）
- `sample_section_avx`（noise.rs L48-96）**从未被调用**（grep 全库仅定义处）+ **函数体非 SIMD**（标量 dot3/lerp/perlin_fade）——**死代码 + 非真 SIMD**。
- 「Perlin 26.56→19.55ns (1.36x)」= `bench_noise.rs` 在 `-C target-feature=+avx` 下**编译器 auto-vec**，非手工 AVX 路径。
- 「features_probe 95.40% 不变」平凡（AVX 没接线，生产仍标量）。
- **全管线 -1% 方向可靠**（45.47→45.01ms，400 chunks）→ noise 非瓶颈判断成立（决策不因归因修正改变）。

### ⚠️ 五、ShiftDF Cache2D 潜在 bug（M1，judge 发现—已保守修正）
- `shift_y_independent` 曾写「708 个 ShiftDF 全 y 独立」但探针**只测前 5 + 单列 + 不含负 Y**；`shift_y_confirmed` 补测后 708/708 全 y 独立（含负 Y + 4 列）——但 mode 分布仅 ShiftA+ShiftB（overworld 无 plain Shift）。
- **代码层**：缓存后 `Shift` 与 `ShiftA` 都落 `_ => (x,0,z)` 分支 → **plain Shift 被强置 y=0，偏离 C++/Java 参考（实际 y）**——潜在 bug。
- **保守修正**：plain Shift 不缓存（用实际 y，保持参考语义）；ShiftA/B 缓存安全（y 无关构造性保证）。features_probe 95.40% 保持。
- ⚠️ 风险：若未来维度用 plain `minecraft:shift`，需复核 y 语义——见 multichannel-errors.md M1。

### ❌ 六、aquifer 主瓶颈方向成立，但「21.5ms」是 Partial 快照（judge 限定边界）
- Rust 全管线 45.48 < Java FULL 55 → **Rust 反快 ~1.2×**；宏观专项 Rust 34.66 > Java 23-25 → **宏观 aquifer 慢 ~1.4-1.5×（相对 Java 真差距）**。
- 「21.5ms」= 减法导出（34.66 macro − 13.14 density），非 aquifer 独立计时；且 17.5ms（16ch）vs 21.5ms（400ch）是不同测量上下文，非精确单一值。
- aquifer 内部多次翻案（P4 推翻 barrier 大头）。下一步优化前须用无污染计数探针（aquifer_*_count）锁 apply 固定开销真实构成。

### 🧰 七、工具演进 / 产物
- 探针：`macrolize_probe.rs`（channels 纯性）、`macro_sampler_probe.rs`（diff0）、`corner_sampling_breakdown.rs`、`tree_vs_noise_breakdown`、`shift_y_dependence`（含 708 全集补测）、`bench_noise.rs`（AVX micro-bench）。
- ⚠️ 产物契约（judge 驳回项）：`.artifacts/index.yaml` 无本 session 条目；`macrolize_probe.rs` 有未提交 diff + `corner_sampling_breakdown.rs` untracked——主会话需补登记 + 提交（见 draft-artifacts-index-entries.md）。

### 📌 记录指引
- 结论 → 07 主题篇追加小节（draft-07-macro-layer-refactor-perfloc.md）。
- 过程 → 本节（10-timewise）。
- **错误台账 → `.investigations/macro-layer-scout/multichannel-errors.md`（M1/M2 五段式 + 速查表）。**
- 状态：各环节 candidate（confirmed 由人类授予）；生产接线未完成。




## 260901-03 nether 存档写入口径 Full 化（1.0.22 dll，双 seed，candidate）

> 承接 09 篇 nether 维度课题 + `.investigations/nether-save-full/`。目标：存档级（MCA 直解）Full 口径量化 Rust nether 接管质量。dll sha256=C5AC5309F3C59A044（1.0.22 M17），区域 4×4 @(3200,3208)。

### ❌ 一、首轮 run 三场全部无效（enabled=false 未察觉）——已作废
- ReadWorldProbe 新增 nether 支持（dim 属性 + 动态 min_y/height + `_nether` 参照后缀）后跑 seed A：gen1 内存 131 差 / gen2 内存 1 差 / gen2 存档 104 差 / reconfirm 读盘 1 差——5 条「矛盾观察」跨运行不一致。
- fan-out 三候选（b1 时序 / b2 管线 / b3 非确定）分析后，b2 日志取证倒查发现铁证：**三场 run CppBridge 全部 `enabled=false`、`[Mixin] intercepted` 0 条——dll 从未加载，全部 vanilla-vs-vanilla**。原 5 条观察**已作废，被 v2 Rust run 取代**（§15.4 取代记录；facts 文件正文不删不改，待主会话回填顶部 supersedes 标注——judge #20）。
- ⚠️ b2 论据更正一笔（judge #16）：其子候选①声称「seed 从 ref 文件内读天然防错位」**与代码不符**——seed 实来自 `-D` 属性拼文件名，header 读后丢弃不校验（fail-fast 建议优先级升高）。
- 根因/教训五段式见错误台账 `nether-save-errors.md` E1-E5（cppWorldgenDir 传错一层 / header 断言凭印象 / 未查接管标志 / 论据未指认代码行 / 矛盾先查前提）。

### ✅ 二、ctypes 直连定位 cppWorldgenDir 错层（数据层证据）
- ctypes 直连 `wg_create` 单变量复现：传错层（把 CppBridge 注释里的解压布局 `…/worldgen/data/…` 当 wg_dir）返回 0，传对层（含 `data/` 的层 = `versions/1.20.1/data/worldgen`）返回非 0——机制根因坐实，b2 早期「dll 提取失败/临时目录权限」推测降为表象。

### ✅ 三、v2 真 Rust 双 seed 三口径数据（judge 全 PASS，建议 candidate）
- seed A = -2032795982907864146：内存 = 存档读回 **99.9376%**（精确同值 1047922/1048576）；MCA 直解 **99.9278%**（1047819，差 103 = cave_air 簇，精确对账）。
- seed B = 8576294172403134396：三口径精确同值 **93.5156%**（980582/1048576）。
- 口径声明（§9.7 三要素）：载体 = MCA 存档直解 + ReadWorldProbe 内存读 vs vanilla 参照（WGB2）；覆盖面 = 4×4 chunk 全高度（nether min_y=0 height=256）；**与 docs/09 的 96.44% 探针口径不可比**（载体不同）。
- Rust 真实参与证明：两 seed v2 log 均 `enabled=true` + 64 条 intercepted（目标 4×4 + feature 蔓延邻域）。

### 🔍 四、残差分类（数据直读 PASS，机制解释保持 draft）
- seed A（757 块）：矿石 feature 差 84.5% > air↔cave_air 尾随簇 13.7% > magma 1.1% > 熔岩湖边界 0.7%。
- seed B（67,994 块）：basalt deltas / 表面规则三大宗石互换 76.6%（全部落在 y≤127 噪声高度内，y≥128=100%）> soul sand valley 8.4% > 矿石 3.9% > magma 2.5% > 熔岩湖 2.0%。
- 机制归属全部 candidate 以下（residual-interpretation §4 诚实声明 Partial/Degraded）。

### 🔍 五、未闭合待查项
- 103 cave_air 簇机制（v2 下内存=读回精确同值但 MCA 多 103——新形态矛盾，b1/b3 均未闭合，judge #14 确认保持 draft 正确）。
- basalt deltas 大宗互换（B1 surface rule 条件链）、nether 矿石 features 缺口（未实现 vs 错位）——深挖优先级见 residual-interpretation §3。

### 📌 记录指引
- 错误台账 → `.investigations/nether-save-full/nether-save-errors.md`（E1-E5 五段式 + 速查表）。
- 结论 → 09 篇（或 06/07 篇，主会话定）追加小节，草稿 `knowledge-drafts/docs-appendix-nether-save.md`。
- 过程 → 本节；judge 意见 → `.investigations/nether-save-full/judge-review.md`（#16/#17/#20 修正项待主会话落实：b2 论据更正、A2 引用作废数据改写、facts 文件回填 supersedes 标注）。
- 状态：数据与口径声明 candidate（judge 建议），confirmed 留人类；机制解释 draft。



## 260901-03 B1 定论：basalt deltas 三大宗互换 = feature 产物 × 两种基底地形（candidate）

> 承接 260901-03 nether 存档条目 B1 未闭合项（52,078 块 / 76.6%）。结论 → 09 篇「B1 定论」节；错误 E6 → nether-save-errors.md。

### ✅ 一、前置验证推翻交接假设（Hole 语义）
- 上轮遗留「Rust Hole 用 surface_depth<=0」为 M6（2026-08-30）修复前过时表述；开工前廉价独立验证：Rust surface_rules.rs L101 当前为 `Hole => stone_depth_above <= 0` 与 Java 一致，dll M17（sha C5AC5309）含修复——Hole 语义课题闭合（§15.4：09 篇原行加 supersedes 注记，不删）。
- 教训印证 AGENTS.md 交接结论验证纪律：交接里的「方向/待查假设」开工先验，本轮第一动作即排除一条假赛道。

### ✅ 二、机制定论（三方实验）
- 架构：cppReplace = Rust 只接管 populateNoise+buildSurface；vanilla carvers+features 仍在 Rust 地形上跑。宗石大宗（basalt_blobs/blackstone_blobs、large/small_basalt_columns、delta、basalt_pillar）本是 feature 阶段产物。
- 三方数据：纯 Rust（ctypes 直连 dll vs rlib 直跑 cell 级 0 差异）vs FULL = 77.43%（basalt→netherrack 157k）；存档（+Java carvers/features）= 93.5508%；WG_SKIP_SURFACE=1 = 55.18% 且 blobs 不触发（stone 基底非 netherrack → blackstone=0、quartz/gold ore=0）。
- 判读：互换主因 = 同一套 Java feature 在两种基底地形上的命中/形态差 + Rust surface 薄带残差；biome 分桶（互换 100% 落 vanilla basalt_deltas 列）排除 biome 源分配差。

### ❌ 三、fan-out 两候选裁决
- ❌ **.b1 surface_depth 带厚机制不成立**：带厚上限 ≤6 层，实测 40 层体块不可达（排除证据：`.artifacts/.b1-surface-depth/` 最终 verdict）。
- ⚠️ **.b2 nether_state_selector 恒 0.0 是真实 bug 但非主导**：`create_for_dim` step4 预加载表缺 nether 噪声（nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer → `unwrap_or(0.0)`）——只解释零星分支内翻转（证据：`.artifacts/.b2-nether-state-selector/`）。**修复待做**：一行预加载表补齐，预期闭合 soul_soil 子族等。

### 🔍 四、新过程事实与口径纪律
- **同 dll 非确定性容差**：同 dll 两次完整 run 相差 369 块（93.5156% → 93.5508%）——Java feature 阶段邻块写入调度非确定性；存档口径对齐指标 MUST 声明该容差。
- **对照口径澄清（§9.7）**：纯 Rust 口径（77.43%）与存档口径（93.55%）载体不同不可比；B1 深挖参照分两用——BlockProbe SURFACE 口径测 Rust surface 残差，存档口径测端到端。

### 📌 记录指引
- 结论 → 09 篇「B1 定论」节（草稿 knowledge-drafts/docs-09-b1-verdict-draft.md，含 L165 supersedes 标注文本）。
- 错误 E6（对照口径误置）→ `.investigations/nether-save-full/nether-save-errors.md` 追加。
- 通用模式 → knowledge/discovered/workflow-patterns.md 发现 #10（三阶段归因法）。
- 状态：机制定论 candidate（judge 审查通过建议），confirmed 留人类。

## 260901-04 nether_state_selector 预加载表修复（.b2 遗留项闭合）+ SURFACE 口径残差量化 ✅

> 承接 260901-03 B1 定论条 fan-out .b2 遗留修复项（⚠️ 真实 bug 非主导，待修）+ judge WARN-4 待排除备择。结论 → 09 篇追加两小节；错误 E7 → nether-save-errors.md。

### ✅ 一、selector 预加载表修复
- `WorldgenRust/src/worldgen_handle.rs` step4 surface rules 噪声预加载表（L192-195 一带）补 6 个 nether 噪声：`minecraft:nether_state_selector` / `patch` / `soul_sand_layer` / `netherrack` / `nether_wart` / `gravel_layer`（全部存在于 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise/*.json`）。
- 机制：预加载表原只含 overworld 噪声 → `surface_rules.rs` noise_threshold_sample（L120-137）查不到 sampler 时 `unwrap_or(0.0)` → nether_state_selector（min threshold=0.0）恒 true → 恒 basalt 分支。

### ✅ 二、验证（存档口径，seed B = 8576294172403134396，4×4 @3200,3208）
- 修复前 93.5508% → 修复后 **93.8988%**（match=984600/1048576），+0.348pp ≈ 10× 同 dll 非确定性容差（±369 块 ≈ ±0.035pp）→ 真实改善。
- E1/E3 判据核对通过：log `[CppBridge] initNether enabled=true` 且 seed 一致（`.investigations/nether-save-full/cmd-output/b2-fix-rerun.log`）。
- 分族：总 mismatch 63,976（solid_solid 62,850 / van_solid_rust_air 580 / van_air_rust_solid 546）；soul_soil ref 5474 vs save 1334 仍偏低——selector 已生效，soul_soil 大头疑似 Java feature 阶段（B1 主导机制的正常残差，非本 bug）；quartz/gold/magma 偏高归 ore features（待 A1+B4 重估）。

### ✅ 三、SURFACE 口径残差量化（judge WARN-4 排除）
- 采集：vanilla SURFACE 参照（BlockProbe 默认口径，无 carvers/features），FULL 参照备份 `.blocks.full`，hash 不同确认口径切换生效；对比脚本 `.tmp/b2_surface_residual.py` vs 纯 Rust rlib dump（`.tmp/b1-rlib-blocks.bin`）。
- 数据：SURFACE 参照 vs FULL 参照 diff 仅 21,296/1,048,576（97.9691% identical，本 4×4 区域 features 贡献 ~2%）；SURFACE 参照 vs 纯 Rust = **77.4857%** → **Rust surface 层自身残差 = 22.5%**，主导形态 basalt→netherrack 157,658 / blackstone→netherrack 35,031。
- 判读：①「薄带残差」实为 surface 层大宗差异（非薄带）；②存档口径 93.8988% 说明 Java features 在 Rust 基底上补齐大部分——与 B1 定论自洽；③「Rust 已实现 feature 与 Java feature 并存重复放置」备择按架构事实排除（cppReplace 只拦截 populateNoise+buildSurface，features 只由 Java 运行一次）。⚠️ FULL−SURFACE 差 ~2% 是 4×4 局部观察，勿外推为全局 features 占比。
- §9.7：77.4857%（SURFACE 口径）/ 93.8988%（存档口径）/ 77.43%（纯 Rust vs FULL）三口径载体互不可比，分列。

### 📌 记录指引
- 结论 → 09 篇「nether_state_selector 预加载表修复」+「SURFACE 口径残差量化」两小节（supersedes 取代「B1 定论」节 .b2 待修注记，candidate）。
- 错误 E7（预加载表隐式契约缺 key 静默回退 0.0）→ `.investigations/nether-save-full/nether-save-errors.md`。
- 状态 ✅：修复完成、验证通过（candidate，confirmed 留用户）。

### ✅ 追加（同日晚）：C1 修复 + 容差口径修正
- C1 落实（judge CONCERN）：`surface_rules.rs` noise_threshold_sample 未知 noise key 改为每 key warn 一次（全局去重，OnceLock+Mutex），不再静默回退（commit a3e9909）。两次回归均无 warn 触发 = 预加载表完备性运行时佐证。
- **容差口径修正（C4 属实）**：C1 回归两次 + 原修复轮一次 = 修复后 3 采样 {93.8988, 93.6767, 93.6765}，与修复前 {93.5156, 93.5508} 区间不重叠 → 改善保持成立，但下界修正为 +0.126pp；**同 dll 存档口径散布实测 ~2330 块（0.22pp），±369 块（n=2）系低估**。后续存档口径比对统一「区间不重叠 + 多次采样」判据。详见 09 篇「容差口径修正」段（log：cmd-output/c1-warn-regression{,2}.log）。




## 260902-01（nether-save-full 课题续）

- ✅ **C2 预加载表数据驱动化**（commit 709b006）：`worldgen_handle.rs` step4 新增 `collect_noise_keys()`，从 surface_rule JSON 构建期收集 noise_threshold 引用 key；overworld 保留静态清单（代码规则无 JSON 源）；nether 静态 6 key 清单删除（E7 手工修复的架构层收尾）。3 连跑 93.8988% 逐位同值无回归；judge C2 CONCERN 闭环。candidate。
- ✅ **P2 矿石归因重大转向——双重 feature 应用**（H_B'，judge PASS 建议 candidate）：发现 `wg_fill_blocks_multi` 内含 carver+feature 阶段（worldgen_handle.rs L442-449，WG_SKIP_CARVER/WG_SKIP_FEATURES env 门控）；存档链路 mixin 只拦 populateNoise + cancel buildSurface，Java CARVER/FEATURES 照跑 → 存档 = Rust+Java features 双跑。消融链：SKIP_FEATURES → 93.8988%→94.4241%（+5508），quartz 4478→2125（ref 1992）/ gold 1525→739（ref 728）/ magma 3814→1979（ref 1533）；SKIP_CARVER 仅再 +370。矿石 ~2.2× 偏高全额归因双跑。修正早前「features 只由 Java 运行一次（无双跑通道）」判断（09 篇原行加注记不删）。遗留：overworld 同路径双跑 vs 99.9% 对齐矛盾 → X1 FEATURELOG 裁决 🔍 进行中；修复方向 judge CONCERN = env 门进程全局，勿全局默认翻转，需句柄/调用级显式 flag。
- ✅ **B2 soul 家族定稿——上轮假设证伪**：V1 证伪「soul_soil 大头在 Java feature 阶段」（Rust 管线 soul_soil 1363 ≈ 存档 1334，缺口 4140 在 Rust 管线内）。V2 探针 180 点三签名：A biome 足迹偏移/收窄（valley 判 nether_wastes，聚簇 x≥3410 边界带）；B soul_soil 子分支失效（entered+selector<0 仍 applied=netherrack）；C floor 侧 soul_sand_layer 分支疑似缺失（组3 entered 0/60）。.b1a 结构差主导；.b1b 噪声值偏离 idk（缺 Java 同点对照）。Java features 对 soul_sand 净回补 +587。下一步：V3 Rust-vs-JSON 结构对拍（零成本最高优先）→ V4 RouterProbe 同点 selector → V5 biome 边界带。🔍 V3-V5 未做。
- 🔍 **X1 FEATURELOG 裁决**：overworld 双跑是否成立及为何对齐 99.9%，进行中，待回填。
- ⚠️ **环境坑 E8/E9**（详录 `.investigations/nether-save-full/nether-save-errors.md`）：E8 = 沙箱下 gradle runServer 提取 worldgen.dll AccessDeniedException → JAVA_TOOL_OPTIONS=-Djava.io.tmpdir 指工作区；E9 = WorldgenRust.dll mtime 因 fs::copy 保留时间戳不可信 → dll 新旧用二进制字符串探测；bin-diag bin 临时挪 src/bin/ 编译（init_vertical 需 pub 化）。


### ✅ X1 FEATURELOG 裁决回填（260902-01 深夜，裁决 overworld 双跑矛盾）
- runServer -PreadWorldProbe（overworld）+ WG_FEATURELOG：**8137 条 [FEATURE] 行 = Rust features 在 overworld 存档链路同样运行**（mixin 拦截范围两维度相同，X1 候选①「overworld 未装配 features」证伪）。
- 同 region 对齐 = 97.3537%（seed B，4×4 @3200,3208，FULL 参照 3219616B）——「overworld 99.9%」系 seed A 不同样本的口径记忆，同 region 从未测过 99.9%。双跑矛盾解除：overworld 同样双跑，只是 overworld feature 密度/基底差使其未显形为 2× 矿石（定性，未量化）。
- 教训补充 E9 同族：**历史对齐数字引用必须带 seed+region+口径三要素**，凭「维度印象」引用构成伪矛盾（本条即 X1 的成立前提）。
- 遗留：overworld 双跑的量化影响（对齐率/矿石计数）未测，需 seed B overworld 消融 run（SKIP_FEATURES）定性——下轮候选。

### ✅ 拍板回填（260902-01）
- 用户全部批准：六项 candidate → **confirmed**（selector 修复 / SURFACE 残差量化 / 容差口径修正 / C2 数据驱动化 / P2 双重 feature 归因 / P3 soul 缺口在 Rust 管线 + 三签名方向）。状态标注已回写 09 篇、decision-package、两份 artifacts verdict。修复类后续（双跑修复 / V3-V5）不在拍板范围。


## 260902-02（nether-save-full 课题续：双跑修复 + soul V3）

### ✅ 一、句柄级 wg_set_flags 修复双跑（judge PASS，candidate）

- 承接 09-07 P2 矿石归因 judge CONCERN（`WG_SKIP_*` env 门控进程全局，勿全局默认翻转）。
- 改动：`worldgen_handle.rs` AtomicU32 flags（bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE，**OR-env 语义**，0=回落 env 兼容）+ `api.rs` wg_set_flags/wg_get_flags + jni_bridge + Java CppWorldgen/CppBridge（**默认 mask=0b011**，`-Dcoreswap.rust.stages` 可覆盖）。
- 回归（C1 措辞修正）：同 region（seed B，nether 4×4@3200,3208）3 次复跑全部 **94.4241%**（990108/1048576，FULL 参照，ReadWorldProbe 存档口径；修复前 93.8988%）——验证确定性非覆盖面；ore per-id quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = **SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533）——与消融实验因果链重复。
- 设计：`.investigations/nether-save-full/design-wg-set-flags-20260908.md`；judge：`.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md`；日志：cmd-output/flags-regression-run4/5/6.log。

### ✅ 二、V3 结构对拍（draft，Degraded）

- nether.json surface_rule 全 10 种节点类型 Rust 解析器全支持、7 顶层分支逐节点一致。
- ❌ 签名 B（soul_soil 子分支失效）/ C（floor 侧 soul_sand_layer「分支缺失」）的**结构差解释不成立**——「分支缺失」假说被否定。
- 归因指向：①运行时输入差（V4：生产链路 soul 分支 ctx dump vs probe 输入对差）；②biome 分类层（签名 A 同源，V5）。产物：`.artifacts/.b2-soul/v3-structure-diff.md`。🔍 V4/V5 未做。

### ⚠️ 三、环境坑 E10（详录 `.investigations/nether-save-full/nether-save-errors.md`）

- 强杀 gradle daemon 后所有 gradle 调用报 `Failed to load native library 'native-platform.dll'`——根因 = `C:\Users\NDark\.gradle\native\**\native-platform.dll.lock` 拒绝访问（非 dll 本身，--stacktrace 定位到 .lock 文件级拒绝）；删锁被沙箱硬拒（工作区外，升级亦被拒）；最终修复 = **GRADLE_USER_HOME 指向工作区 `E:\PYTHON\CoreSwap\.gradle-home`**。

### 🔍 四、参数试错过程（run1-3 空跑教训）

- run2/run3 两次因 **bench 参照文件名四要素不一致**空跑（cppReplace + readWorldProbe + blockProbeDimension=nether + bench 参数须与 ref 文件名四要素一致）——完整命令模板以 flags-regression-run4.log 对应调用为准固化。

### 📌 记录指引

- 结论 → 09 篇追加「句柄级 wg_set_flags 修复双跑（candidate）」+「V3 结构对拍（draft，Degraded）」两小节，草稿 `knowledge-drafts/20260908-docs-09-dualrun-fix-and-v3.md`。
- 通用模式 → `knowledge/discovered/build-tooling.md` 发现 #7（E10：GRADLE_USER_HOME 沙箱策略 + 参照四要素核对），草稿 `knowledge-drafts/20260908-build-tooling-faxian7.md`。
- 错误 E10 五段式 → `.investigations/nether-save-full/nether-save-errors.md` 追加 + 速查表加行。
- 状态：双跑修复 candidate（judge PASS）；V3 draft（Degraded）；confirmed 留用户。


## 260902-03（soul V4/V5 课题：布尔解析 bug 根因 + 修复 + C4 消融）

### ✅ 一、V4 采集——输入差候选否定

- patch `WG_SOUL_CTX_DUMP`（env 门控点级 ctx dump，OnceLock 点集 + chunk 级门控，零热路径成本）+ bin-diag `soul_ctx_dump` 驱动生产 fill_chunk_blocks，180 点（V2 签名 B/C mismatch 点）180/180 全命中。
- 抽样点 probe CSV vs 生产 dump：biome / sda / sdb / surface_depth / selector **逐项全同**，整规则 apply 一致 → **「probe 复算输入 ≠ 生产 ctx」输入差候选被采集数据否定**。
- 新矛盾：biome=soul ∧ ceiling_ok ∧ selector<0 → applied=netherrack(256)，与 V3「进 soul 分支必得 soul_soil 兜底」结构推演冲突 → 矛盾收敛到求值/解析层。产物：`.investigations/soul-v4v5/v4-collection.md`。

### ✅ 二、树复现——假阴性 8 处

- bin-diag `soul_tree_repro` 直接 dump 解析产物树（修复前）：**8 处 `asd=false` 假阴性**（3 处 JSON 原文 true：soul ceiling/floor / basalt floor / wastes floor / gravel y_above 等；其余 8 处 JSON 原值即 false，恰成假阴性掩护）——「参数全对拍」对 JSON 原文肉眼核对全对，实为解析产物 ≠ JSON 语义。
- 3275,2,3201 裁决：bedrock_floor（above_bottom 0..5）生产侧随机判定先中即返（applied=31），从签名 B 证据集剔除。

### ✅ 三、根因定位 + 修复

- 根因（.b2-soul fan-out 裁决，候选 b）：`parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段 → Bool→None→**恒 false**（surface_rules.rs 三处：y_above L1079 / stone_depth L1093 / water L1116）；soul ceiling `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1` → 分支未进 → 穿透 netherrack 兜底。产物：`.artifacts/.b2-soul/v4-eval-conflict.md`。
- 修复：新增 `parse_bool_field`（`as_bool().or_else(as_f64 != 0).unwrap_or(false)`），三处替换；grep 复核无残留布尔误读。

### ✅ 四、四级回归（seed B = 8576294172403134396，nether 4×4 @3200,3208）

- 树复现（postfix）：5 处翻 true（soul ceiling/floor、gravel y_above 30/35、basalt floor），8 处保持 false=JSON 原值，无一误翻。
- 生产 dump（180 点）：netherrack 103→71，+soul_soil 18 / soul_sand 14；定点 3260,1,3200 applied 256→**258**。
- 存档口径 ×2：94.4241% → **run1 96.6215% / run2 96.5866%**（+2.20pp；run 间差 366 块在 #10 非确定带宽 ~2330 块内）。
- per-id：soul_soil 1334→5771（ref 5474）/ soul_sand 1471→2494（ref 2457）——soul 族闭合至 ref 邻域；gravel 674=ref 精确相等。
- 签名 C（soul_sand_layer entered 0/60）同 bug 源随修复闭合；basalt −3631→−1736（B1 家族收敛，无新负迁移）。

### ✅ 五、judge PASS + C4 overworld 消融量化 + C1 回写

- **judge（review-001）PASS，建议 candidate**：三重锁定（解析树 dump × 定点 apply × 生产 dump 逐位一致）成立；`parse_bool_field` 语义正确且替换完备；回归数字可归因；Degraded/§9.7 声明合规。4 项卫生处置（E9 临时 bin 删除 / index 登记 / supersedes 双指针 / 本 docs 草稿来源确认）→ 主会话清单。
- **C4 overworld 消融**（seed B，overworld 4×4 @3200,3208，存档口径）：默认 mask=0b011（不双跑）**98.9520%**（1556380/1572864）vs 旧双跑 mask=0 **97.3266%**（1530815/1572864）——双跑修复 **+1.6254pp**（+25565 块），差异集中 y=-64..63 features 活跃层（y≥64 两 run 100% 一致）；judge C4 CONCERN（overworld 默认 mask 行为变更未回归量化）量化闭合，无回归证据。⚠️ 单 region 单次，方向性量化非覆盖面结论。产物：`.investigations/soul-v4v5/c4-overworld-ablation.md`。
- **C1 措辞回写完成**：同 region 3 次复跑 = 验证确定性/可复现性（非多 region 覆盖面），判据措辞已按 C1 修正在 09 篇回归节与时间线落实。

### 🔍 残差

- basalt −1736 / blackstone −434（B1 家族遗留）；366 块非确定带宽；**V5 biome 边界带未做**（修复后残差图需重导，残差 ~3.4%）。

### 📌 记录指引

- 结论 → 09 篇追加「布尔字段解析 bug 修复签名 B/C（candidate）」小节，草稿 `knowledge-drafts/20260909-v4-boolfix-docs.md`（supersedes V3 节处置方向，原节不删）。
- 通用模式 → `knowledge/discovered/compiler-idioms.md` 发现 #8（布尔字段 as_f64 恒 false）+ `knowledge/discovered/workflow-patterns.md` 发现 #12（静态对拍须对拍解析产物）。
- 状态：根因 + 修复有效性 candidate（judge PASS 建议）；confirmed 留用户。

## 260902-04（实际 2026-09-02 15:45 = 本轮提交簇 git 时间戳锚；V5 残差排查）

- ✅ **存档口径复现**：nether 4x4@3200,3208 seed B = 96.6215%，与 confirmed 逐位一致（seed 三查 ✓；本轮三犯防住：权威 run 目录 = `runtime/1.20.1/java/run`，勿被 `E:\PYTHON\MC` 旧快照误导）。
- ❌→✅ **探针坐标 bug 假象（错误链，高价值）**：wBiome 误用 chunk 局部 x,z（0-15）调 world.getBiome → 实查 chunk(0,0)=warped_forest → 35426 列 biome 100% 单向假象 → 一步裁决探针 4 层收敛（biome 列对比→6 维对拍→storage cell dump→整列 storage vs biomeAccess）锁定 → 修正为世界坐标。教训沉淀 → workflow-patterns 发现 #13。fan-out .b6（存储填充异常）随修正**排除**。
- ✅ **残差归因闭合（candidate，judge 有条件 PASS）**：96.3% 残差列 biome 一致 → 残差主体 = basalt_deltas 同 biome 表面规则差（B1 家族本体）；签名 A（ssv↔basalt 互换）降级 1303 列 ≈ 3.7%；分类器 4 点 6 维逐位一致排除（.b1/.b2/.b4）。per-id：netherrack +1539 / basalt −1050 / blackstone −652 / soul_soil +297 / gravel 0。→ 09 篇新小节。
- 📝 **gradle -P 映射坑三犯**：biome6.points / biome6.cellDump / biome6.colDump 前两次忘加 -P→-D 映射行静默不生效 → build-tooling 发现 #8。
- 📝 **容差实例更新**：basalt −1736（run2）vs −1050（本轮），量级一致数值漂移 ~686，佐证 #10 非确定容差判据。
- 📝 **T4 工程加固**：worldgen_handle.rs parse_surface_rule 静默回退 → fail-fast（panic）；surface_rules.rs AboveY/Water mult 硬编码 0 跨版本风险标注；重编 dll 回归 96.6215% 不变（cargo check + 内容字符串验证）。
- 🔍 **下一步**：B1 下钻（nether_state_selector 采样 / delta 分支进出 / blackstone·basalt 分配）；签名 A 独立修（biome 边界/offset 精度）。

## 260902-05（实际 2026-09-02 15:5x–17:00；B1 下钻 Phase 1：口径修复 + surface 层收敛 + 残差主体改判）

- ✅ **overworld NoiseConfig 错位破案（错误链，高价值）**：RouterProbe 原 NOISEPT 对拍取 `server.getOverworld()` 的 NoiseConfig——overworld=XOROSHIRO 派生、nether=LEGACY，首轮「selector 场差 42%/patch 差 64%」全为该错位假象（发现 #13 同族）。修复：`-DnoisePoints.dim=nether`。
- ✅ **场一致**：selector/patch 噪声场 Java(nether)↔Rust 146 列零符号翻转，|diff| 中位 1e-5；派生链四中间种子 + origins 逐位相同 → selector 采样差候选正式排除（.b2 遗留 idk 闭合一半）。
- ✅ **BlockProbe FULL 预生成口径污染修复（错误链，高价值）**：原代码无条件预生成 FULL 邻域 → SURFACE 口径返回已提升 chunk，260901-04「SURFACE 残差 22.5%」参照实为 FULL 混合 → docs/09 supersedes 注记。修复：pregen 仅 FULL 口径执行。
- ✅ **大宗归属**：FULL vs 真 SURFACE diff 214,497 块（79.5% identical）；SURFACE basalt 15,065 vs FULL 172,704 → 大宗 basalt/blackstone 确为 features（blobs）产物。
- ✅ **surface 层收敛**：Rust surface-only vs 真 SURFACE 参照（02B94092）= **99.9423%**（mismatch ~600 块 soul 族散点）→ 260902-04 T2「同 biome 表面规则差」表述改判，surface 规则层基本正确。
- ✅ **残差定位**：存档 mismatch 35,426 块中 98.5%（34,865）落在两侧基底完全相同的块上 → 残差主体产生于特征阶段 blobs（H1 主导候选确立；H2 = 差基底放大待模拟验证）。
- 📝 纪律：inline `python -c` 双次被坑（GBK/BOM、解析歧义）→ 脚本文件化；探针/参照口径三查再次救场两次（NoiseConfig 维度、pregen 提升）。lib 零改动。
- 🔍 下一步：P1 blob origin 对拍 + P2 放大模拟（→ 260902-06）。

## 260902-06（实际 2026-09-02 17:02 起；B1 下钻 Phase 2：H1 机制链 candidate 定案）

- ✅ **P2 放大模拟排除 H2**：b1_blob_amp_sim.py rev1 verdict bug（20,000 采样 blobs_k=1122 误当 per-region 预算）→ rev2 修正；per-region 触碰 blob 期望 ≈3.7 个 × 上限 853 ≈ 3,150 块，对 34,246 缺口 shortfall ~10.8× → INSUFFICIENT；1,122 差基底中可种子仅 472 → H2 降为排除项。
- ✅ **P1 三层探针链对拍**（BlobProbeMixin + ConfiguredFeatureProbeMixin + SurfaceColDumpProbe + ColProfProbeMixin，lib 零改动，seed=8576294172403134396 两轮均验三查）：① PlacedFeature 入口两轮 1308 行全同（chunk 级随机分叉排除）；② ConfiguredFeature placedPos vanilla 20,327 / cpp 20,320，only_v=644/only_c=637（~3%）集中 blob/column/delta 族；③ 首分叉行 320 delta 同 x 异 z（judge 订正：初稿误标同 x,z 异 y；实为流内 skip 行对齐边界伪影）；④ 终态列 dump 9216 列 × 4 口径逐列全同（不含流体层）；⑤ **COLPROF 终审：10 列 diff 全部同构——V 含 `99|air→lava`（熔岩海面），C 缺失、代以 `100~104|air→netherrack`**。
- ✅ **H1 机制链五环 candidate 定案**：熔岩海缺失 → 转换面序列漂移（CountMultilayerPlacementModifier y 零随机语义）→ delta origin y 漂 → delta 流内级联（±1~5）→ blob 链式放大皮肤差（only_v=450/only_c=445 量级自洽）。修复落点 = Rust nether surface 填充规则；环节分级：1-2 环硬证据 / 4-5 环推理待 LAVAAUDIT。→ 09 篇新小节「B1 下钻 H1 定案（260902-06）」。
- 📝 **错误台账 8 条**（五段式详见 `.investigations/b1-downdrill/b1-errors.md` 新建）：后台 job 相对路径落空 ×2 / gradlew wrapper 不存在 / yarn PlacedFeature 实名 generate 非 place / mixin 非 private 静态成员被拒 / RegistryKey.getValue() 带命名空间 equals 恒 false（CSV 空 4 轮真根因，发现 #13 同族）/ BufferedWriter 未 flush 即 stop 丢数据 / cppWorldgenDir 必须显式 -P / BlobProbe 无 driver 不生成 chunk。
- 🔍 下一步：judge 审查 → LAVAAUDIT + Rust surface 熔岩海修复 → 回归三判据。




## 260902-07（实际 2026-09-02 18:36–19:1x；B1 下钻：H1 环1 证伪定案——id 标注误读）

- ❌→✅ **H1 环1 被证伪（本 session 主线转向，judge PASS）**：原计划 LAVAAUDIT 定性 → 熔岩海修复 → 回归；实际 LAVAAUDIT 探针新增（runtime ColProfProbeMixin `colprof.mode=lavaAudit` 模式：v1 只记 above=lava → v2 加记 below=lava 面向 lavaSurfY + `[LAUIDMAP]` 一次性 id 映射；build.gradle 补 colProfMode/colProfR -P→-D 映射）→ v1 指标盲区暴露（99.4% 一致率不记 above=air 转换，不构成世界一致证据，已废弃）→ `[LAUIDMAP]` id 映射实锤：**19319=blackstone 非 lava、5854=basalt 非 netherrack**（air=0 lava=96 water=80 netherrack=5850 basalt=5854 blackstone=19319 …）——昨日 COLPROF「`99|0->19319` = air→lava 熔岩海面」标注纯系误读。
- ✅ **COLPROF 10/25 列 diff 真相**：V 黑石底（y=99 恒平）vs C 玄武岩底（y=100~104 贴地形），两侧均实心材质；快照时间线（b1_colprof_firstsnap.py）证明 diff 在第一枚举快照即分叉（V#0/C#0 同构异材质，T 序列稳定），非 feature 事后改列伪影。
- ✅ **LAVAAUDIT v2 全扫**（11,443 公共列）：air→lava 面向**两侧均为零**——该区域 feature 阶段起点无任何熔岩面向；熔岩以流动态（96）存在于两轮相同位置（lavaTopY 分布峰值 23~24 一致）；lavaTopY 逐列差 329/11443（2.9%）、n 差 60 列（judge D1 补测，本 session 实测）。昨日「终态列 dump 9216 列逐列全同」不矛盾——该口径只记顶块（roof y=128），y=99~104 材质差不可见。
- ✅ **judge PASS（环1 证伪 + 转向）**：环 2~5（转换面漂移 → delta origin 漂 → 级联/blob 放大）作为现象保留（cfg 独立证据 delta y=111/119/121 vs 99）；CountMultilayerPlacementModifier y-零随机 findPos 语义只需「第一转换面不同」即可成立。judge 六项 CONCERN：lavaTopY 逐列已补（329 列）；only_v=10/only_c=56 覆盖缺口未解释；结论限 3200,3208 区域；SURFACE 99.9423%（4×4 固体表面顶块口径）vs 内部转换面差——口径三要素须显式声明；LAUIDMAP 只跑 vanilla 轮，cpp 轮待补；「标注三查」应入 NEXT_SESSION 开工检查项。
- 🔍 **下一轮方向（judge 设计，四候选判别 fan-out）**：(a) surface rule 材质分支差 (b) biome 判定输入差 (c) surface rule 随机序列差 (d) 前置地形形状差（NOISE/density 阶段列高度差——judge 指出若成立则 a/b/c 全降次生）；判别探针 = SURFACE 前/后逐列 dump（材质序列+biome id+顶面 y）；候选非严格互斥，按「判别目标」设计各自可独立排除；**先跑 (d)**。
- 📝 **错误台账 3 条**（五段式详见 b1-errors.md E-B1-9/10/11）：raw id 标注当公理继承（E-B1-9，重大——「标注三查」与 seed 三查同级）/ 探针指标盲区 lavaAudit 测不了 air→lava 面向（E-B1-10）/ grep 行首锚对带前缀 log 恒零命中假「零输出」（E-B1-11）。
- 📝 lib/dll 零改动（仅 runtime 探针 + .tmp 数据/脚本）；修复方案（熔岩流体填充）作废未执行。
- 🔍 下一步：判别探针 SURFACE 前/后逐列 dump → 四候选 fan-out 先跑 (d) → LAUIDMAP 补 cpp 轮。

---

## 260902-08/09（B1 下钻 Phase 3：四候选判别定案 + 假 100% 陷阱实证）

### 🔍→✅ 判别探针（fan-out 四候选）
(d) NOISE 宏观地形差排除：air 签名 99.68%（seed B，3200/3208，basalt_deltas）。✅
### ✅ surface 层对拍
99.66% 列一致（26/524288 单元），实现差上界 0.005%——非主体。✅
### ✅ 交集闭合
(a)(c) 排除（band 边缘对结构非系统/随机）；残余 = NOISE 微差 13 列 → band 边缘 ±1 平移 + 1 列黑石/熔岩边缘平移（51247,51375）。✅
### ✅ 历史残差改判
13.70% air / 22.5% SURFACE / 黑石·玄武岩底界差 = 测量口径阶段污染；真实 ~3.4% 主体归因 feature/carver 链路（放大系数未量化🔍）。
### ❌ java noise-only 路线证伪
stageMask 语义误解：-Dcoreswap.rust.stages 只控 Rust 内部阶段，cppReplace 下 Java CARVERS/FEATURES 照跑——「noise-only 存档」假象（判据：看存档内容而非 stageMask 日志）。
### 🔍 id 空间发现
Rust surface skip 输出统一 default block id=1，材质在 surface 层 → NOISE 材质级对拍不可做，air 签名可做。
### ⚠️ 假阴性两陷阱（假 100%）
① [128:] 切片空序列 → zip 空 → 0 差异；② mat= split(',')[3][4:] 逗号切散。两处本 session 实证；防范 = sanity 行强制打印序列长度+common 数。
### 附带
gradle --nogui 非 CLI 选项（runServer 失败）；build.gradle rustStages 缺 -P→-D 映射行（已补）。工具遗产：bin-diag b1_noiseonly/surfaceonly_dump.rs，.tmp/compare_*.py、overlap_check.py、judge_followup.py。


## 260902-10（feature/carver 放大系数量化 + ~3.4% 存档残差改判：参照口径阶段污染）

### 🔍→✅ 盘点发现区域缺口
C5 桥接（3200 区 26 种子 → 200 区 3.4% 残差）盘点证伪：200 区 vs 3200 区数据不同域；上轮 b1_blob_amp_sim（INSUFFICIENT 10.8×）输入 = 200 坐标旧 bug dump，结论作废未继承。✅

### ✅ 同域采集
3 次运行 seed 三查全过（worldSeed=8576294172403134396，log：.tmp/amp-van-ref-run.log / amp-cppreplace-run.log / amp-cppreplace-run200.log）；dll sha256 一致（68d7f401）；stageMask=3。benchOrigin 坑：benchOriginX/Z 是块坐标（chunk 3200 区传 51200/51328）。✅

### ✅ 3200 区 16 块全落种子列
fresh vanilla FULL vs cppReplace 存档 = 16/1048576（0.0015%），100% 落 B1 的 13 列 NOISE 微差 + 1 列 surface-only 差 → 放大系数 0.62 < 1，不存在（amp_step2_join.out.txt）。✅

### ❌→✅ 200 区 20% 异常 → 三方判别
200 区 fresh vs old ref = 214,474（20.4538%）异常高（amp_step3_region200.out.txt）；三方判别（amp_step4_crosscheck.out.txt）：old ref vs fresh = 20.45%、fresh vs cpp = 0.0000% → 异常收敛到 old ref 一侧。✅

### ✅ old ref = SURFACE 参照定性 → 改判
old ref（sha256 02b94092f917cb5d）内容指纹缺矿石 417/607/45、cave_air 730、basalt blob → 是 SURFACE 阶段参照被当 FULL 用，贯穿 M16→V5 多轮（96.62%/13.7%/22.5%/3.4% 全是口径污染链）→ ~3.4% 存档残差 = 跨阶段伪残差，改判 supersedes C5 条残差归因（verdict：.artifacts/b1-candidates/amplification-verdict-260902-10.md）。✅

### ✅ judge + 端到端水平
judge 0 BLOCKER，建议 candidate（review-260902-10-judge-amplification.md）；区域差异自洽（200 区无种子→无残差）补进 verdict 第 5 条；当前 dll 端到端真实对齐 = 两区合计 16/2097152 = 99.9992%。✅

### 📌 记录指引
- 结论 → 09 篇追加「feature/carver 放大系数定案」小节（supersedes C5 条，原节不删）。
- 教训 → 参照文件核对升级五要素（seed/size/origin/dim/stage 内容指纹）→ build-tooling 发现 #10；benchOrigin 块坐标坑同条附记。
- 状态：candidate（judge 建议授予）；confirmed 待用户。🔍 下一步：B1 NOISE 微差下钻（16 块，外推边界不变）。

## 260902-11/12（B1 NOISE 微差下钻：零面擦边符号翻转定案 + 封顶结案）

### 🔍→✅ scout 勘探 + 差异格 d_exact 逐点探针
13 差异格全部落密度零面擦边带（|d_exact| ∈ [3.7e-8, 2.27e-5]），符号与 air 归属 13/13 自洽（10 列 rust 多 air / 3 列 vanilla 多 air，方向不系统）。✅
### ✅ 全区普查封闭验证
4×4×128 = 524,288 单元 |d| 普查：擦边集 83 格（0.016%），12/13 差异格落入（第 13 格 +2.27e-5 略超 1e-5 阈值；阈值为后验，仅统计用途）。与存档残差 0.00076% 同量级。✅
### ❌→📝 scout 小样例推断被全量否定（高价值踩坑）
scout 5 样例推「x 等差 9 贯穿」→ 全量 13 列核对仅 4 列子簇成立——弱证据 draft 标注正确，消费前必须全量复核。📝
### 📝 census histogram 标签偏移坑（高价值踩坑）
b1_grazing_census.rs 桶打印标签错位一格，以 grazing 计数为准；复用脚本先修标签。📝
### ✅ judge 审查（0 BLOCKER，建议 candidate 有保留 C1-C4/N1-N4）
核心保留：C1 Java↔Rust 数值差从未配对实测（单侧数据不能定量言两侧差）/ C2 量级分流居间、归因推断 / C3 措辞越界（已改「非符号级/网格级结构错误；A1 vs A3 不可区分」）/ C4 A4 排除前提未静态核验（补 idk）。修订全部应用。✅
### ✅ 用户拍板三连 → confirmed
C3 措辞采纳授 candidate / 不补 Java 配对采样（接受机制类收敛）/ A1-A3 不再下钻，99.9992% 封顶结案。verdict + index.yaml 回写 confirmed。产物 noise-drill-verdict-260902-11.md；过程详录 .investigations/b1-noise-drill/。✅

## 260902-13（SIMD 静态核验闭合 A4 前提 + 知识库落盘）

- ✅ **A4 排除前提闭合（Degraded 级静态核验）**：versions/1.20.1/cpp 全部 .h/.hpp/.cpp 零 SIMD intrinsic / 零 pragma simd——「C++ dll 无显式向量化 noise 路径」成立（编译器自动向量化不在核验范围，保留声明）；Rust 侧 noise.rs 确有显式 AVX 路径 sample_section_avx（cfg target_feature="avx"）。verdict 第 3 条 idk 注记由本轮补证闭合。
- 📝 **知识库落盘（subagent 草稿 → 主会话应用）**：09 篇追加「B1 零面擦边定案」小节；可复用判据「零面擦边格签名判别法」→ workflow-patterns 发现 #15。
- 🔍 下一步（继承 NEXT_SESSION 优先级）：多 seed/多 biome 泛化重采样。

## 260902-14（极端坐标 FP 微差应力测试：±30M 无地形颠覆 + 泥土带系统差遗留 + 课题封存）

### ✅ 采样矩阵（4 极限 + 1 对照，每区 1,572,864 块）
2 seed（±7159…337）× ±30M 双极限角 + 普通坐标对照（chunk 200,200）；WGB2 端到端逐位，dll sha256=68d7f401 与 B1 同构建；每跑 seed 三查（server.properties 备份/删 world/worldSeed 日志核对），ref 跑无 CppBridge、cpp 跑 populateNoise intercepted 16/16。四极限区 98.85–99.85%（≫95% 预登记线），对照 98.5914%。✅

### ✅ 对照归因 → 封存
对照区一致率低于全部极限区 + 泥土带在对照区同样出现（17,754 vs 12k-18k）→ 失配主体非坐标极端化引起；排除泥土带后极限区只剩 B1 同族 FP 擦边散簇（区④ 466 个，最大 522）→ FP 微差不随坐标爆炸，课题封存。负轴区④为四区最佳 → 负坐标无结构崩坏（floorDiv 前科未复现）。✅

### ✅ judge 通过 + 用户拍板封存
judge 三源核对 PASS（数字抽查/判据忠实度/归因/§9.7/日志污染/遗留项可见性），CONCERN C1（index.yaml 登记）/C2（派生统计落盘）已闭环（derive_stats.out.txt 复现五区 y 分带与簇统计）。用户拍板：课题封存；泥土带系统差醒目标注、仅记录不下钻。✅

### 📝 4 个新坑（高价值，详录 facts-260902.md）
1. **gradle 沙箱坑复现**（= build-tooling #7）：`Failed to load native-platform.dll` → `GRADLE_USER_HOME=.gradle-home` + `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=.tmp\java-tmp` 修复。
2. **cppWorldgenDir 必须显式**：缺 `-PcppWorldgenDir` → `worldgen-data not found in mod resources`（runServer dev 模式 mod=classes 目录，jar 内解压路径失效）→ 显式传 `versions/1.20.1/data/worldgen`。
3. **WGB2 overworld 每 chunk id 数 = 98304**（256×height=384，非 nether 65536）——解析器按 65536 读 → stride 错位假坐标；正确格式 BlockProbe.java L477-485/L921-938。
4. **正极限原点越界**：chunk 1874999 为世界最后合法 chunk，原点 29999984 + size=4 会越界 → 取 29999936（chunk 1874996..1874999）。

### 📌 记录指引
- 结论 + ⚠️ 泥土带醒目小节 → 09 篇追加「极端坐标应力测试定论」小节（全新，无 supersedes）。
- 状态：candidate（用户已拍板封存）。🔍 遗留：泥土带系统差仅记录不下钻（开逐位 100% 对齐课题须先 biome 归因 fan-out）。

## 260903-04（lossless-accel 路线② FFI 工作包：spv 陈旧产物定案——「逐位一致」哨兵结论被陈旧二进制产物击穿）

### ✅ FFI shim + 三事实 + 决定性三路切分
W2 `gpu_ffi.cpp` C-ABI shim（build.ps1 `-Ffi`）+ W3 Rust 角点探针（bin-diag，LoadLibrary 动态加载零新依赖）。三事实实测（gpu-corner-probe-260903-04.txt）：① create ~64-75s，同 seed 第二实例无缓存同价（每次全量编译 pipeline）；② 串行 5.0µs/pt，双线程同 handle Mutex 反而 0.61×（GPU dispatch 异步流水，Mutex 不串行化 GPU 队列，readback 同步保正确性）；③ GPU vs DFC oracle 6144 点 f32_exact 仅 43.26% → 系统性 diff 非纯精度。tri-cut（同程序同坐标 CpuBackend.sample vs GpuDensityEngine.fill）：C++ CPU vs C++ GPU 自己就 major diff（16 点中 5 点，最大 0.502）→ **FFI/Rust 侧无罪，问题在引擎内部**。✅

### ✅ 三 worker fan-out（互斥候选并行）
.bA GPU fill 路径 / .bB CPU 参照单点采样（结论：CPU 参照正确——饱和值 -0.458333343 = DF_SQUEEZE clamp -1 属正常，错误在 GPU 侧；顺带发现 sampleInterpGrid y=320 grid[49] 越界读为独立真实缺陷，另立修复项）/ .bC 历史域考古（历史「逐位一致 maxDiff 3.1e-07」域 = seed …396 × x∈[0,63] × y∈[-64,-49]，新证据域外，结论保留须补域声明）。产物 .artifacts/lossless-accel/route2-tricut.bA/bB/bC.md。✅

### ✅ 决定性双 seed 切分 + 根因 = final_density.spv 陈旧产物（supersedes fan-out 归因方向）
已知值哨兵点 (784,160,-408) 历史验证 seed 下：旧 spv 输出 **0.0453032888 = 时间线 L1386 记录的 D23 修复前错误值**（正确 -0.458333343）——直接复现历史错值签名（tri-cut2）。证据链：① spv mtime 08-15 14:17 **早于** D23 修复提交 cc58e05（08-15 19:21）5 小时，commit 9de661e（19:22）提交的是修复前编译的 spv；② 08-23 `final_density.comp` 与 cpu_backend.h 同批重生成但 spv 未随之重编——生成器多产物部分更新失配。重编（gen_final_density.py → glslc → 部署，旧 spv 备份 .bak-pre-d23）后：**双 seed 23 点 major_diff=0**（tri-cut3），全量 6144 点 max_diff=9.18e-6（f32 ULP 级），rounded6 96.08%。✅ 已结案（根因闭合）

### 📌 记录指引
- 错误链五段式 → `.investigations/lossless-accel/lossless-accel-errors.md`（subagent 草稿应用）。
- 可复用判据（二进制产物无法从内容/时间戳判断新旧 + 逐位一致哨兵须配已知值哨兵点）→ build-tooling 发现 #12（260903-04）。
- 状态：candidate（judge 待过）；confirmed 待用户。

## 260903-06（lossless-accel P-A：ch0 跨语言通道级闭合——假残差定案 + transpiler 唯一缺陷定论）

### ✅ C++ CPU ch0 oracle 建立 + 三方对拍全绿
density_probe -dfDump（delegate 程序化提取自 overworld.json，避转录错）+ bin-diag ch0_gpu_dump：3 列 × 48 角点，GPU vs C++ major=0 max=1.795e-6（f32 ULP 级）；macro vs C++ major=0 max=5e-7 → GPU 与 macro 生产路径通道级均 = C++/Java 语义。✅

### ✅ 假残差定案（supersedes 260903-05 残留 idk）
「macro vs GPU ch0 残差 0.03-0.23」= 探针坐标混列假象（GPU 取 z=0、macro 取 z=16 误作同点）；C++ oracle 证实两点各自精确正确（(4,80,0)=-1.0966554 / (4,80,16)=-1.2163714）。✅ 已结案

### ✅ transpiler 缺陷复确认（正确坐标下）
transpiler vs C++ major=28/48，max=0.2299，y≥32 纯线性化步进 0.246875（YClampedGradient 线性分量）——缺陷确认在 transpiler 路径。✅

### ✅ P-B 真根因定案 + 修复（supersedes bA 闭包压平归因）
真根因 = worldgen_handle.rs NoiseSet 漏设 blended_noise（sample_blended_noise 返 0.0）；单点隔离探针（ch0_single_point.rs）证伪 cache_2d 闭包压平归因（生成函数全对、清缓存无效）。修复 = build_transpiler_noises()（blended_noise 数据驱动，两处构造共用）。修复后 transpiler≡macro 全列 diff=0；gpu_channel_probe 5×5 通道 major=0 + combine 抽样 major=0 → PASS，**WG_GPU_CHANNELS 生产门解锁**；fallback 改绑 DfcDensity 方案废弃。坑先前只记在 diag 注释侧（transpiler_slices_ch0.rs:33 等）未吸收进生产构造——错误台账 LL9。✅ 已结案

### 📌 记录指引
- 取代链条目（§15.4 两条）+ workflow-patterns 发现 #17（跨探针对比坐标钉死律 + 单点隔离复测补充要点）见 .artifacts/lossless-accel/pa-ch0-closure-260903-06.md。
- 状态：candidate（judge 待过）；WG_GPU_CHANNELS 门解锁待 judge + 用户确认。

## 260903-08（lossless-accel P-C：0.61× 复测定案 + 端到端三方对比 + GPU ON 判读）

### ✅ P-C2：0.61× 双线程异常未复现 = 测量伪影候选（supersedes 260903-04 [fact2]）
无探针复测（bin-diag/gpu_mt_wall_retest.rs）：n=8/n=6144 双口径 × 5 轮 S/P 交替 × 中位数 → 1.006×/0.989×，轮间波动 0.964-1.065×，原 0.61× 落分布外且原测系单 shot 无中位 → 判测量伪影，Mutex 真串行化成立。附带发现原探针 fill_n=8 口径错（注释「8 chunk 批量」vs 实参 8 点/次）→ 错误台账 LL10。P0「fill 全同步串行」Degraded→数据层升级：sync-check mismatch=0/6144。✅

### ✅ P-C1：端到端三方 256 chunks（§9.7 预声明，Full 层）
region(200,200) 16×16、单线程、区外预热、median 主判据：OFF 76.93/71.84/71.21（三跑）→ 零退化 ✓（env 门控未改生产代码，git diff 佐证）；ON 369.28ms/chunk（慢 OFF 4.8×）= 每 chunk 小批量 dispatch+readback 同步往返成本——用户预热假说已检验：预热收益仅 ~20-25%（P-C2 完全热态 ~172ms 仍高于 CPU 管线），非预热伪影；批量合并列独立优化工作包。negseed 判别 <3% → seed 非因素。✅

### 🔍 vs Java 33ms 慢 2.2× → 开问题 Q-PD1
同日有效对比：Rust 全管线（含 features）72-77ms vs Java FULL 33ms → 慢 ~2.2×。08-29「反快 1.2×」系 Rust 无树花口径不可比 → workflow-patterns 发现 #18。Q-PD1（draft）：features/carver 段疑似差距大头，独立排查。遗留 idk：Java 55→33 漂移未归因；features 段耗时分布未测。

### ✅ judge + 状态
review-001（三源核对）：无 BLOCKER，建议升 candidate；C1（计数构成）/C2（预热取窗口径）/C3（OFF 复跑存档）已清偿。**status = confirmed（260903-08 用户拍板）**。产物：.artifacts/lossless-accel/pc-results-260903-08.md + index.yaml；过程 .investigations/lossless-accel/{pc2-retest,pc1-e2e,review-001}-260903-08.md + cmd-output/*260903-08*。坑：误访问废弃 runtime 目录 E:\PYTHON\MC\versions\1.20.1\java（现行 = CoreSwap\runtime\1.20.1\java）→ build-tooling 环境坑补记；Rust 2021 闭包字段级捕获 → LL11。✅

## 260903-09（Q-PD1 包：Rust vs Java 2.2× 差距分阶段归因——大头在 aquifer，supersedes 260903-08 两个方向假设）

### ✅ 基线廉价独立复核（交接纪律先行）+ 附带发现死参数
Rust OFF 两跑 median 70.23/73.49ms（落 08 日 71-77 稳定带）✓；Java fresh（删 run\world 后）median≈32/total 10993（对 08 日 33/11067）✓ → 2.2× 有效。⚠️ 附带发现：`pc_e2e_bench.rs` L18 解析 WG_E2E_SEED 但 L22 恒用常量 SEED → 08 日 negseed「seed 判别」实际同 seed 跑两遍，「seed 非因素」证据无效（supersedes）。⚠️ 新坑：run\world 残留时 Java bench 走磁盘加载（total 764ms vs fresh 10993）——bench 前必须删 run\world（世界状态第四查）。✅

### ✅ 分阶段差分（WG_SKIP_* 门控，两轮稳定）
bin-diag/qpd1_stage_bench.rs（新，隔离区）：aquifer ~37ms/chunk（占 FULL ~60%，62ms 口径）；density/interp 底座 ~14.4ms（~23%）；surface ~5.5-6.7；carver ~5-6.5；orevein/features ≈0（噪声级）。两轮 ±1ms 级一致。✅

### 🔍 结论（confirmed 260903-09 用户拍板）
Q-PD1 归因：**差距大头 = aquifer 段**。supersedes pc1-e2e/pc-results-260903-08 两个方向假设（§15.4 双指针，原文不改）：①「features/carver 段疑似大头」②「negseed 判别 → seed 非因素」。Amdahl 读数：GPU density 优化端到端天花板 62→~47ms（仍慢 Java ~1.4×）→ 优化主攻转向 aquifer 段机制（邻居随机偏移/split/采样次数，复用 WG_AQUIFERCOUNT/WL/BP 计数器），新课题待立项。

### 📌 记录指引
- 新坑两条 → workflow-patterns 发现 #19（世界状态第四查 + 假象签名）/#20（死参数假判别 + 恒等式自检）。
- 产物：.artifacts/lossless-accel/qpd1-attribution-260903-09.md（candidate，judge review-260903-09 通过）；过程 .investigations/lossless-accel/{q-pd1,knowledge-draft,review}-260903-09.md + cmd-output/qpd1-*。


## 260903-10（Q-AQ1：aquifer 段 35ms/chunk 机制归因——est 冷扫描 × 全价 init 采样）

### 🔍 探针链（六步）
1. 基线锚定：FULL=60.43，aquifer 段 35.07（no-ore 48.84 − no-aquifer 13.77，落 Q-PD1 带）。
2. 计数器采数（新 WG_AQUIFERBP/WL/SURF + 现成 COUNT）：apply≈68k/chunk（bp 815,747÷12）、wl 110k、barrier 仅 ~10/chunk；两批线性稳定。
3. diag 微测：bp 2.5 + wl 3.7 + calcdensity ≈0 →「缺口 ~29ms」假设成立；冷缓存假设 A（surf est）被否（7342 采样×0.089µs 仅 0.66ms——该基线后被作废，见下）。
4. fan-out b1/b2：b1 生产循环镜像探针 T3−T2=32.20≈35.07（缺口在 fill 循环内复现）、T3−T4=26.65（每 chunk 新建 Aquifer 冷态成本）；b2 审计否证 carver 级联主因（雕刻点仅 ~756 apply/chunk），交错 bench A|Coff=33.47 独立交叉验证。
5. GRID_ARG_SAMPLES=0 反证 b1'「Interpolated 单槽抖动」（§4.2 凑数算术一并撤回）→ H* 重归因：est 冷扫描 × init 全价采样（重量叶 old_blended 24 octave 无缓存）。
6. judge R2 调和：est 单价三口径（3557 上界 / **2117 新鲜进程实测** / 1646 假冷）→ est ≈15.4ms，占 counter-free 冷态超额 22.70ms 的 ~68%。

### ✅ 结论（candidate，judge CONCERN 有条件通过，待用户 confirmed）
机制：每 chunk 新建 Aquifer → surface_cache 冷 → wl miss 链 → est 全量扫描（Java NoiseChunk est 列缓存 chunk 级持久 vs Rust 每 chunk 丢弃）＝ aquifer 段成本主体。分解：est ~15.4 + 冷 miss/杂项 ~6-8 + 暖 apply ~5.5。supersedes：F5 基线 0.089µs、F4 漏 t_fl 表述、b1' 抖动机制、首版 26.1ms 量化（R2）。修复方向（另立优化包）：est 查表化/列缓存跨 chunk 持久化。

### 📌 记录指引
- 新坑 → workflow-patterns #21（微测形态 40× 假基线）/#22（自由参数凑数）/#23（证据摘要漏行）/#24（多臂顺序 bench 假交互）；指纹 → algorithm-fingerprints #16。
- 产物：.artifacts/lossless-accel/qaq1-attribution-260903-10.md（candidate）+ index.yaml 登记；过程 .investigations/lossless-accel/{qaq1-evidence-pack,q-aq1,qaq1-b1-candidate,qaq1-b2-candidate,review-qaq1}-260903-10.md + cmd-output/qaq1-*（8 件）。
- 🔍 遗留 b4：carver 机械成本列状态反直觉（全 Air 列 carver-on 贵 22.97 vs 实地形 10.62ms，机制未定位）；后续 aquifer 实验一律 carver 双臂同关。顺手修复：pc_e2e_bench.rs seed 死参数（#20 族）。

## 260903-11（est 查表化优化包：est_at 共享 + 跨 chunk est L2，candidate @ 0949402）

> 承接 260903-10 Q-AQ1「修复方向（另立优化包）」；过程 `.investigations/lossless-accel/est-opt/`；结果快照 `.artifacts/lossless-accel/est-opt-result-260903-11.md`（candidate）。

- ✅ **P0 交接验证**：复跑 qaq1_surf_probe（新鲜进程），iterations 7342 / avg 34.35 / miss 2782 逐项一致，median 72.84 在方差内 → Q-AQ1 est 冷扫描量级可继承（§15.3 廉价独立验证）。
- ❌→✅ **P1 调研误读两例被核对推翻（高价值，详见 workflow-patterns #25）**：① G5「fill/carver 各自 Aquifer::new 不共享」系 subagent 引用 :547（诊断 API）错位，生产路径 :446 唯一构造——主会话差点按假差距点投入实现，被设计 worker 代码核对推翻（G5 supersedes 入 k3-k2-verdict）；② K3「Java 步长 4」系常量记忆错，实际 4×size_vertical=8，与 Rust 一致（疑点解除）。P1 文档原文不改，裁决记录 supersedes。
- ✅ **P2 fan-out：b2 主形态判死 → 分叉塌缩单线 b1**：b2 粗表逐位一致硬约束下不成立（唯一逐位安全形态 ⊂ b1-b）；K2 blend 旁路等价成立（blend 类 DF 全为 no-blending 常数，density.rs:626-628；原「blending_active 字段」引用系 b1 拟新增字段，judge R3 补正）；新增 D3 扫描域差异（est_at noise_height vs Aquifer height，仅 nether 有差，overworld 同 384）。
- ✅ **实现（commit 0949402，门控默认关）**：b1-b EstL2 精确值缓存（量化列 key / FIFO 131072 上限 / 代际挂 handle / blend 闸门；`WG_EST_L2` 门控）；b1-a est_at 共享（`WG_EST_SHARED`，对齐 Java ChunkNoiseSampler.java:222-226）；探针 bin-diag/estopt_ab.rs（四臂 hash A/B + L2 统计）。
- ✅ **四臂验证**（§9.7：载体=fill_chunk_blocks 全管线；覆盖面=64 chunks A/B + 256 chunks e2e region(200,200) seed 8576294172403134396；历史口径同 pc_e2e 260903-08 可比）：off 臂 == HEAD 基线（64-chunk 聚合 hash 相等 + stash 重建基线）；l2 臂 hash 逐位一致；16 chunks est 迭代 7342→1715（−76.6%）；**256 chunks e2e median 75.94→27.69ms（−63.5%）**。shared 臂 hash 变化（D1 角列量化修正 + D3 扫描域）——默认关，翻默认前 MUST Java 逐位验证。
- ✅ **judge 两次审查 PASS**：P2 选型（有条件）+ P5 交付（4 CONCERN 无 BLOCK，建议 candidate）：C1 L2 stats 口径修正（e2e 行未落盘，外推表述作废）/ C2 零回归证据载体标注（聚合 hash，非 block_probe 全量 diff）/ C3 未执行清单显式声明 / 代码抽查 8 项全过。
- 📝 **观察（不反推机制定论）**：e2e 收益（−48ms/chunk）超 est 微测上界（15.5ms）——生产冷路径 est 实际单价 ≈11µs/iter vs 微测 2117ns（working set 失配，workflow-patterns #21 补充案例）。
- 🔍 **未闭合**：shared 臂疑似修正既有 surface 错位 bug（需 Java 逐位裁决，独立小包）；b1-b 翻默认前置（mt_fill Mutex 基线 + 大 region 淘汰 + e2e l2 stats 落盘）；nether est_at 扫描域（D3）未收敛（生产仅 overworld，显式声明）。

### 📌 记录指引
- 通用模式 → workflow-patterns #25（静态调研结论失真两例）+ #21 补充案例（working set 维度）。
- 产物：.artifacts/lossless-accel/est-opt-result-260903-11.md（candidate + index.yaml 登记）；裁决/验证 k3-k2-verdict / p0-handover-verify / cmd-output/estopt-{ab-arms,perf}-260903-11.txt。
- 状态：candidate（judge 建议授予），confirmed 留用户；翻默认（WG_EST_L2 / WG_EST_SHARED）均不在授予范围。


## 260903-12（est shared 臂 Java 逐位裁决 + L2 翻默认三件套 + gpu-merge 重议——四臂课题收口轮）

> 承接 260903-11「未闭合」三项（shared 裁决 / 翻默认前置 / e2e 观察）；产物 `.artifacts/lossless-accel/{est-shared-verdict,est-l2-defaultflip-p2,gpu-merge-revisit}-260903-12.md`（均 candidate）；过程 `.investigations/lossless-accel/` + `cmd-output/*260903-12*`。

### ✅ P0 交接验证：四臂 hash 复现（§15.3 廉价独立验证先行）
复跑四臂 A/B（estopt-ab-arms-p0）：off `74f5dfc4` / shared `8bff4087` / l2==off + 命中 84.9%，与 260903-11 逐项一致 → 交接结论可继承。环境四查过（删 run\world、seed 8576294172403134396 三处一致、WG_* 默认关、dump 门控不影响 hash）。
**为什么**：260903-11 的 shared hash 变化只是「待裁决假设」，不验证不续推（M14/M11 纪律）。

### 🔍 scout：Java est 链勘探（subagent 隔离）
勘探产物 `.investigations/lossless-accel/est-shared-java-map/java-est-chain.md`（Java est 调用面 + mixin RETURN dump 路线）。
**为什么**：裁决 Java 语义需先摸清 est 调用链与可靠 dump 位置（#2：机制未明先勘探）。

### ✅ Java est dump 探针搭建 + 三方对比裁决（P1，Full 层）
Java 侧 EstDumpProbeMixin（RETURN dump）+ Rust WG_EST_DUMP 同 seed 同 region 对比：共同列（c0 原点角，64 chunk）**shared 64/64 与 Java 逐值一致；off 0/64 全偏（judge 复算 delta 恒 −1）**；角列敏感性 63/64（唯一敏感 chunk (201,200)：java@+16=56 / shared@+12=48 / off=55）。**裁决：shared=修正既有 est 错位，off=系统性偏离**（supersedes 260903-11「未裁决假设」）。
**为什么**：翻默认前置 = shared 语义必须 Java 逐位背书；共同列等值证明 + 敏感性探测双口径闭合（judge A2 精确化覆盖面表述）。

### ✅ 翻默认前置三件套（P2，Full 层）
① Mutex 争用基线（estopt_mt_bench T=1/2/4/8 交错双跑）：L2 加速比 2.55→3.12× 随线程不降反升，无争用退化（双跑偏差 <3%，judge B 修正「<2%」表述）；② 大 region sweep（64×64）：命中稳定 92±1%、evictions=0，触顶投影 ~7600+ chunk（judge B 复算修正 inserts ~40k / ~17 条每 chunk），typical region 远未触顶；③ e2e l2 stats 落盘（judge C1 清偿）。
**为什么**：260903-11 judge 预置的三项翻默认门控逐项清偿，量化声明全部可溯源。

### ✅ P2.4 剩余差归因：微测外推生产无效（#21 量化实锤）
est_price_probe 同代码 hot 60ns/iter vs cold 5.7µs/iter（形态差 ~95×）；跨 session 生产隐含单价 8.5/9.9µs 稳定 → 次级效应候选不构成互斥候选，fan-out 免触发（judge 认可作为收敛判定依据）。
**为什么**：e2e 收益与微测上界的 ~1.5× 剩余差要给 Partial 解释并声明，不能留缺口（§9.7）。

### ✅ gpu-batch-merge 重议（P3.1，决策建议）
est L2 落地后新基线：Rust l2 单线程 27.69 ms/chunk vs Java FULL ~33（**跨 bench 近似比较**，judge D 标注；保守口径 T=1 35.8 同量级）→ 立项目标（追平 Java）已被无损路径消除，8 线程 ~4.5 ≈ 7× Java（量级判断）。**建议降级/搁置 gpu-batch-merge**，待用户拍板。
**为什么**：GPU 路径 dispatch/readback 成本（369ms，260903-08）在小批量下是负收益；目标消失则工作包失去存在依据。

### ✅ 两轮 judge 审查（review-est-shared / review-p2-p3-final）
① shared 裁决：PASS + 3 CONCERN（A1 off 臂 −1 扫描偏移线索 / A2 覆盖面措辞 / C1 dump 缺 seed 头），复算零偏差，同意上报 confirmed；② P2/P3：无 BLOCK，judge B 数值修正（inserts/触顶投影）+ judge D 跨 harness 可比性标注，修正后可推荐 candidate→confirmed。
**为什么**：confirmed 前 MUST judge（AGENTS 强制触发点）；judge 复算产出 A1 新机制线索（见下新课题）。

### 🔍 新课题登记（本 session 新增，均另立验证，不阻塞上述 candidate）
1. **off 臂 −1 扫描偏移 bug（judge A1，生产 bug 线索）**：off 臂 `(min_y..min_y+noise_height).rev().step_by(8)` 半开区间 rev 首采样点 = 319，Java 从 320 起扫（319,311,… vs 320,312,…）——同时解释「c0 也偏离」与「delta 恒 −1」的规整性；**off 是当前默认臂，生产影响独立于翻默认决策**。
2. **surface_rules.rs:505 大 region panic**：`fill_chunk_blocks` 在 64×64 sweep 至 ~2304-2560 chunk 处 panic `missing noise sampler`（estopt-sweep 尾部原文在案）——疑似预加载噪声表缺项在特定 biome/区域触发，生产稳定性课题（数据截止于此，4096 chunk 全程未完成）。
3. **角参数 +15→+16 修正待办**：Rust 两臂 heights4 参数 `cx*16+15`（量化后 +12）≠ Java SURFACE 四角 +16——完全对齐需改 +16（两臂，独立小包），**翻 shared 默认的前置条件**。
4. **shared 翻默认待用户拍板**：前置四项 ✅（零回归/无争用/淘汰无风险/stats 落盘）+ 建议与 +16 修正联动一次到位。

### 📌 记录指引
- 通用模式 → workflow-patterns #25 补充案例（静态「恰好一致」断言必须显式算术）+ #21 补充案例（hot/cold 95× + 单价稳定性作收敛判据）；build-tooling 发现 #13（GRADLE_USER_HOME 复发）/ #14（watchdog 强杀 + dump 内嵌 seed 头）。
- 产物：三份 candidate artifact + index.yaml 登记；judge 意见两份；cmd-output 六件（est-compare-p13/p13b、estopt-ab-arms-p0、estopt-mt-baseline、estopt-sweep、est-price-p24）。
- 状态：三份产物均 candidate（两轮 judge 建议 confirmed 前清偿文档级修正），confirmed 留用户；翻默认动作不在本 session 执行范围。

## 260903-13（实际 2026-09-03；est off 臂扫描偏移 + 角参数 +16 两处修复——四臂同 hash 收口 + 翻默认实施）

> 承接 260903-12 新课题登记 #1（off 臂 −1 扫描偏移）/#3（角参数 +15→+16）。commit 3e2e67d（修复）+ 翻默认提交；judge PASS（`.investigations/lossless-accel/review-offscan-cornerfix-260903-13.md`，1 should-fix 已清偿）；结论 `.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md`（confirmed）。

### ✅ 一、P0 交接验证（先廉价验证再继承）

- 修复前四臂 hash 复跑与 260903-12 记录逐项一致（off 74f5dfc4eede8ef4 / shared 8bff408735f1560d / l2==off+84.9%）——交接结论继承合法（estopt-ab-arms-p0-260903-13.txt）。

### ✅ 二、off 臂扫描首点修复（judge A1 线索闭环）

- 现象：off 臂 est 对 Java 恒差 −1，c0 原点角也偏。
- 根因（为什么错）：Rust `(min_y..min_y+noise_height).rev().step_by(8)` 半开区间 rev → 首采样点 319；Java `NoiseChunk.computePreliminarySurfaceLevel`（forge official sources NoiseChunk.java:174）`for(l=minY+height; l>=minY; l-=cellHeight)` → 320..-64 含两端。首点差 1 + 下界包含性差，两个独立错位。
- 定位：judge A1 静态发现 + 本 session 抽取 forge official sources（Mojang 官方映射 jar）核对权威实现（非反编译推断）。
- 修复：扫描改闭区间含两端（首点 320、下界 -64）。通用模式 → compiler-idioms 发现 #10。

### ✅ 三、角参数 +15→+16（workflow-patterns #25 第三例实例修复）

- 根因：Java `MaterialRules.java:496-499` SURFACE 四角 `chunkToBlockCoord(i+1) = (i+1)<<4` = +16；Rust 两臂曾用 `cx*16+15`（量化后 +12 ≠ +16）。
- 修复：两臂 heights4 + dump corner_params 统一 +16。

### ✅ 四、验证与翻默认（Full 层）

- 修复后**四臂 hash 完全一致 `f2b1a3932c6e589e`**（off/shared × L2 开关）；est 优化语义零差。
- Java est 角列对比：off / shared 各 256/256 一致 0 diff；敏感 chunk (201,200) 一致（修复前 java@+16=56/shared@+12=48/off=55 → 修复后三方一致）。
- **翻默认（用户拍板 confirmed）**：WG_EST_SHARED / WG_EST_L2 默认启用、`=0` 反转关闭；默认臂 hash 同值 + L2 命中 84.9%，反转臂同 hash + L2 stats 归零（estopt-ab-defaultflip-260903-13.txt）。

### 📌 记录指引

- 结论 → 07 篇「est 优化收口」节后追加小节（260903-13）。
- 通用模式 → knowledge/discovered/compiler-idioms.md 发现 #10（半开区间 rev 复刻 Java 含两端递减 for 的 off-by-one）。
- 未结：surface_rules.rs:505 大 region panic（课题 #2，下轮立项 MUST recode-scout 前置）。


## 260903-14（实际 2026-09-03 深夜；surface_rules.rs:505 大 region panic 修复——预加载 noise key 清单缺项收口）

> 承接 260903-12 新课题登记 #2（sweep 至 ~2304-2560 chunk panic `missing noise sampler`）。过程产物 `.investigations/panic-505/`（错误台账 panic-errors.md E1-E3）；judge 审查 PASS（`.investigations/panic-505/review-panic-fix-260903-14.md`，2 should-fix 已清偿）；结论 `.artifacts/panic-505/panic-fix-verdict-260903-14.md`（candidate）。

### 🔍 现象

- estopt 大 region sweep 在 ~2304-2560 chunk 处 panic：`surface_rules.rs:505 missing noise sampler`（260903-12 sweep 尾部原文在案）；4096 chunk sweep 无法完成。
- 仅 eroded_badlands biome 列且侵蚀度 e>0 触发 → 极低频分支，64×64 sweep 才首次命中。

### ✅ 根因（为什么错）

- overworld 预加载 noise key 静态清单（`worldgen_handle.rs` L272）缺 `minecraft:badlands_pillar_roof`；`place_badlands_pillar`（`surface_rules.rs:1372`）运行时 `get_noise` → `expect` panic。预加载集合与运行时查询集合不同步——新增 expect 型查表调用点未同步预加载来源。

### ✅ 定位（怎么发现的）

- panic 点反查调用链：surface_rules.rs:505 `expect` ← place_badlands_pillar（:1372）get_noise ← 噪声 key 来自预加载清单——清单 grep `badlands_pillar_roof` 缺失即闭合。
- 触发条件（eroded_badlands + e>0）解释「小样本全绿、大 region 必崩」；过程与三错误（E1 worldgen-data marker 路径不一致 / E2 rustc --extern 误指 cdylib / E3 Tee 目标目录后建）→ panic-errors.md 五段式台账。

### ✅ 修复

- 预加载清单补 `minecraft:badlands_pillar_roof` 一行。通用模式 → workflow-patterns 发现 #26。

### ✅ 验证（Full 层）

- 4096 chunk sweep 全程无 panic（修复前 64×64 必崩于 ~2304-2560）。
- 四臂 hash `f2b1a3932c6e589e` 零回归（四臂完整落盘 estopt-ab-4arms-260903-14.txt）。
- 存档口径 3 采样 {98.9969, 99.0284, 99.0067}%（均值 99.0107%）vs 修复前历史 98.9520%：区间不重叠向上，散布 495 块在非确定带宽内（#10 同族判据；改善幅度在散布带内仅作无回归佐证）。

### 📌 记录指引

- 结论 → 07 篇末尾追加小节（260903-14）。
- 通用模式 → workflow-patterns 发现 #26（预加载/注册表与运行时查询集合同步 + 大 region sweep 暴露低频分支缺失）；build-tooling 发现 #15（run 存档口径照抄历史参数清单，`-PcppWorldgenDir` 必带——E1）。
- 产物：`.investigations/panic-505/`（panic-errors.md + knowledge-drafts/260903-14/ + cmd-output/）。
- 状态：修复验证完成 + judge PASS；confirmed 留用户拍板。

## 260903-15（启动期 noise key 机械校验 + CoreSwapFixHelper 解压根治）

> 承接 260903-14 发现 #26 判据 1（启动期机械校验）与 #15 根治方向（解压死路）。过程产物 `.investigations/preload-check/`（verdict + preload-check-errors.md E1-E3 + review + cmd-output/）；judge 审查 PASS with should-fix（2 项已清偿：panic 输出落盘 + 四臂 §9.7 口径补齐）。

### ✅ 结论定案（candidate，confirmed 待用户拍板）

- **启动期机械校验**：`collect_rule_noise_keys` 机械收集 rule 树 + `ENGINE_NOISE_KEYS` 就近维护引擎调用点清单 + `create_for_dim` 启动期断言（运行时引用 ⊆ 预加载集合，缺失即 panic）。盲区诚实声明：引擎路径不在树内，清单是唯一事实源。通用模式 → workflow-patterns 发现 #27。
- **解压根治**：routeRel 双兼容路由（data→原版 / minecraft→data/ 前缀）+ 资源整体重排 = 权威 `versions/1.20.1/data/worldgen`（845 文件）+ 顶层 4 json（共 849）。根因修正：#15 所载「布局不一致」只是次因，**主因 = 资源集不完整（只有 biome/，无 noise_settings），解压分支结构性死路**——历史全靠 `-PcppWorldgenDir` 绕过。通用模式 → build-tooling 发现 #16。
- **验证（Full 层）**：四臂 hash `f2b1a3932c6e589e` 与 260903-14 逐臂一致（零语义回归）；4096 chunk sweep 无 panic（16 block hits/misses 与基线逐项同）；负向测试删 calcite → 启动 panic exit=101 精确报缺 key；删 %TEMP% 缓存实测 marker 出现、849 文件齐；jar 内 dll sha256 = 最新构建。
- **错误**：E1 bin-diag 旧 exe 假阴性 ×2（`cargo build --release` 不编译 bin-diag——探针用前必单编/核时间戳）；E2 collect 循环 NoiseThreshold 臂无 break 死循环（负向测试立功）；E3 #15 根因不完整（资源集不完整为主因）。详见 preload-check-errors.md 五段式 + 速查表。
- **交付**：`runtime/1.20.1/java/build/libs/coreswap-1.20.1-1.0.22.jar`（内置完整 worldgen-data + 新 dll）。

### 📌 记录指引

- 通用模式 → workflow-patterns 发现 #27（启动期断言落地形态 + 负向测试假阴性危险度）、build-tooling 发现 #16（#15 根治复盘 + 死分支信号 + E1）。
- 状态：修复验证完成 + judge PASS（should-fix 已清偿）；confirmed 留用户拍板。


## 260904-03（翻默认后 block_probe 存档口径 Full 回归——遗留闭合 + 既有残差模式化）

> 承接 off-scan-cornerfix-verdict-260903-13.md:31-33 遗留（翻默认当时仅 Partial 声明）。采集 = 三 run（vanilla 参照 / cppReplace 默认开 / double-0 反转）+ 逐位对比。过程产物 `.investigations/lossless-accel/knowledge-drafts/260904-03/` + `cmd-output/cmp_full2-260904-03.txt` + `arm-compare-260904-03.txt`。

### ✅ 主结论（confirmed，260904-03 用户拍板）

- 翻默认后生产 dll 存档口径 **FULL 回归无回归**：default 臂 vs vanilla 参照 99.1526% 一致（13328/1572864 cell，16/16 chunk，aquifer stone↔water 互换族特征）；default==off 同 SHA256 与 260903-13 四臂零语义差自洽。§9.7 载体=WGB2 FULL 存档口径 4×4 单区域、覆盖面=16 chunk×98304 cell、与 260903-13 est 角列口径不可直接比数值。
- 生产 dll sha256 EC4A9AED…C8FC3（晚于全部生产改动）；参照四要素 + seed 三查核对通过。

### 🔍 既有残差登记（独立待查项，非本回归阻塞）

- stone↔water 等互换模式（ref=1→9 n=4165 等 top 对）为**新观察的量化模式**——量级已由 260903-14 记录（99.0107% 带），模式签名未记录过；机制候选：aquifer floodedness/流面高度/表面级联（未验证，下轮残差 y 分布直方图先行）。Run2==Run3 ⇒ 残差与 est 优化无关。
  - ⚠️ **supersedes（260904-05）**：本条 id 注读与三候选方向均被推翻（9=dirt 非 water；真签名 = blob 状 stone/gravel/花岗岩族→dirt/sand 全 y 均匀置换），改判记录见 07 篇「存档口径残差真签名改判」小节；「Run2==Run3 ⇒ 与 est 无关」结论保留成立。

### ⚠️ 流程瑕疵与局限（诚实声明）

- Run1 参照被 Run2 同名覆盖后 run1b 重采（LL12，无数据混入）；Run3 走复用 daemon，env 透传未验证 → 「default==off 同 hash」只作旁证（LL13/#32 族，死判别风险）；两 run 均 [AQF-J] NPE 每 chunk（两臂同现，Java 探针侧既有噪声，非 cppReplace 引入）。

### 📌 记录指引

- 结论 → `.artifacts/lossless-accel/p2full-regression-verdict-260904-03.md` + 260903-13 遗留项闭合指针。
- 残差登记 → 07 篇追加「存档口径残差模式化」小节（补充非取代 260903-14 记录）。
- 通用模式 → workflow-patterns 发现 #32（daemon env 死同值）；错误 → lossless-accel-errors.md LL12/LL13。

## 260904-09（residual-1830 破案：aquifer splitter 派生链一行缺失——1830→76）

> 过程产物 `.investigations/residual-1830/`（investigate-260904-09.md 全链 + scout-aquifer-map.md + 探针 bug 台账）。

- ✅ **廉价独立验证**：dll 三元组（0D247E03 双处一致）+ witness 坐标（205,22,239）双口径核对——交接结论验证纪律执行（§16.3）。
- ✅ **scout 勘探**（subagent）：aquifer 全链管线地图——静态公式 17 项零分歧、缓存臂全闭；关键结构发现：残差**双向并存** → 指 blob 三元组差或阈值震荡；纠正「est 4 角插值」过时假设（两侧均列扫描）。
- ✅ **判别探针**：Rust WG_AQDUMP + Java AquiferDumpProbeMixin，12 点判别 opq/r/s/t 全异 → D4（blob 邻域随机选择差）实锤。
- ✅ **根因 + 修复**：`worldgen_handle.rs` aquifer splitter 漏 `split_str("minecraft:aquifer").next_splitter()`（Java NoiseConfig.java:54 链）——一行修复。
- ✅ **decisive probe**（全新 world 重导，dll 5E2ACB7F，seed 三查 ✓）：**1830 → 76（99.995%）**；流体族/deepslate→air 全消。
- 🔍 **新残留 76 登记**：gravel→sand 49 / sand→gravel 24 / 零星 3——surface 材质微族，与 aquifer 无关，独立小课题（未立项）。
- 状态：修复 = **confirmed（用户实机确认 260904-09 21:23；judge PASS）**；现役 dll 基线 0D247E03 → 5E2ACB7F（resources 已同步）。
- 过程 bug 台账 4 条（门控死锁 / mixin 包禁嵌套类 / 中间名映射 / cmp 键含对比字段）见 investigate 文件；通用模式 → workflow-patterns #43、compiler-idioms #11/#12。

## 260904-10（残留 76 surface 微族收口）✅ 已结案（confirmed，用户拍板 260904-10）
- ✅ scout：D-1~D-6 勘探 + .b1~.b5 候选（.investigations/residual-1830/scout-residual76-260904-10.md）
- ✅ 盘上双臂直读 dump：76 签名三排除——dtop 全 0（❌.b1 列顶级联）/ biome 通道双侧同（❌.b3）/ surface_rules.rs 无 rev()（❌ off-by-one）
- ✅ fan-out 双 worker：.b2 噪声 patch 静态排除（VanillaSurfaceRules.java L263-270 深海分支无噪声条件）；.b4 腔底语义覆盖不了 3 块（❌），副产物炸出 docs/06 ==stone 口径失准
- ✅ P0 一手源码核对：SurfaceBuilder.java L181-183 isDefaultBlock = 非空非流体（≠docs/06 L62/L94）；Rust/C++ 本就一致，无需改码
- ✅ res76_probe 七通道探针：零星 3 块归因 ore_vein 域 1 / aquifer 域 2
- ✅ 真根因：Java surface 收 BiomeAccess（ChunkRegion hashSeed=sha256_asLong + 8 邻域 jitter，MaterialRules L464 块坐标采样），Rust surface 单元直读漏 zoom；C++ biomeCellKey 4 站点已 hashed——Rust↔C++ 实装分歧
- ✅ 修复 commit ab56706：biome_at_surface 仅接 build_surface + sha256 移植 biome_hash_seed；dll 基线 5E2ACB7F→6B31129E
- ✅ decisive 全新 world 重导：76→12（99.999%；sand→gravel 24 全闭合；gravel→sand 49→9）
- ✅ judge review：review-residual76-260904-10.md，建议 candidate（C-1/C-2 已补）；用户拍板 confirmed
- 🔍 剩余 12：ore_vein 1 + aquifer 2（归因已明未立案）+ gravel→sand 9（未立案）
- 🔍 open：Rust carver/feature biome_pick_cell(self.seed) 用裸 seed（C++ 4 站点全 hashed）——实装分歧，风险中，未动

## 260904-13（残 9 gravel→sand 收口，residual12→13 链条）✅ 已结案（candidate，judge 同意；confirmed 待用户拍板）
- probeA（260904-12）三排除→H2 唯一存活；probeB C++ 对照臂 12/12 逐位一致 + Java storage 9/9 deep_ocean → 锁死分类器层；mnDump 直读 NoiseValuePoint 得平局铁证 5041（1e-4 定点 long 域）
- 根因：Rust biome.rs f64 全精度未量化 → 平局点 1e-9 假严格差；修复 = noise_to_long + i64 距离 + 参数表序平局（dll 19D21219→561AFF49 注释终版）
- decisive 12→3（match 100.000%）；途中 -PblockProbe.full 假回归 78107 经旧 dll 同命令 A/B 隔离定责（78116−78107=9 恰为修复点）
- 剩余 3（ore_vein 1 + aquifer 2）各自立案；H1 zoom pick 全局普查 open（残 9 簇已间接覆盖）；bareseed 收口 confirmed 仍待用户

## 260905-03（光照课题：G2 根因转向 + P2 收口）✅ 判据链闭合（candidate，judge APPROVE-WITH-CONDITIONS；confirmed 待用户）

> 过程产物 `.investigations/light-opt/`（g2-convergence-260905-03.md + g2-fanout/b1|b2|b3 + d3-bench-260905-03.md）。

- ✅ **G2 假设推翻**（fan-out .b1 DENY）：「fallback 收不到 propagateLight」——残差 0/448 在 pregen 边界 + vanilla 拉取语义；交接结论廉价验证纪律（§16.3）生效（#36 第三例）。
- ✅ **G2 真根因**：worldgen feature 放置分歧（树叶/藤蔓/矿石/安山岩，judge 全量 447 chunk palette 对比）；光照内核 blocks 一致域无缺陷（G1 exact 100%）。
- ✅ **对比口径判据**：MCA 缺键语义两侧不对称（vanilla 隐式 15 / rust flag1 全 0），统一填充 1.47M 假差异 → workflow-patterns #46。
- ✅ **解析坑两枚**：1.20.1 chunk NBT 无 xPos（region+槽位推导）+ python unpack_from 不推进指针 → build-tooling #20。
- ✅ **round-trip 判据重立**：1.20.1 存档无 isLightOn + 每启动必 relight →「二次重启收敛 + 相对基线」口径；首载漂移 vanilla 17/2025 vs rust 123/2025（7× 待办）；源码参照版本疑点（.tmp/net 非 1.20.1）→ f5-bugs #5。
- ✅ **语义有损登记**：opacity:-1 → u8 clamp 0（18 条目，worldgen-core/src/light/mod.rs:121-125）→ compiler-idioms #14。
- ❌ **D3 双 FAIL**：内核 6.88ms/chunk（合成数据）、e2e 回退 2.4×（29.7 vs 12.4s/2025 chunks），主体 = 内核 BFS——性能待办。
- ✅ **G3c PASS**：nether/end 64+64 chunks 生成 + 高度守卫回退 vanilla（bottomY=0 span=16）+ 无新 crash。
- 🔍 open：rust 首载漂移 7× 待办；D3 性能优化未立项；feature parity 新课题（树叶/藤蔓/矿石/安山岩放置分歧，是否属既有挂起域待用户裁定）；101/447 影子传播未解释残差候选（推断级）。

## 260905-04（D3 光照优化 round1：口径复核 + 种子收缩实施）✅ 判据部分闭合（candidate，judge APPROVE-WITH-CONDITIONS；「e2e 不回退」严格判据 FAIL 待用户拍板）

> 过程产物 `.investigations/light-opt/`（d3-opt-calibration-260905-04.md + d3-opt-round1-260905-04.md + e2e-results.txt）。实际时间 2026-09-05 09:41-10:1x；收尾 commit 92d9b7b。

- ✅ **口径复核**：真实 blocks9（vanilla WGB2 4×4@200 抽 3×3）替代合成口径——内核 5.796ms/chunk（交接合成口径 6.88ms 量级成立但不可比，仅作对照）；「内核 BFS 为 e2e 开销主体」归因廉价验证后可继承（§9.7 三要素声明落盘）；残余 ≈5.6s = Java 收集循环 + JNI。
- ✅ **judge 方向定论 C1-C4**：C1 种子收缩路线（不用 out_flags，无边界正确性风险）/ C2 PhaseTimings 探针 / C3 内核 3 region + e2e 交替 4 臂 / C4 golden 逐位不变。
- ✅ **种子收缩实施**：sky BFS 只入队「边界 15」（内部 15 格零贡献 + 单调不动点与入队顺序无关论证）→ light_compute 委托 light_compute_inner + phased 探针（生产零开销）。
- ✅ **golden 冻结/比对**：pre rlib（96a33b09）冻结 4 用例 golden_pre.txt → post 复跑 golden_post.txt，FNV hash 全等 = PASS（逐位不变）。
- ✅ **e2e 四臂交替**：ON 23.497/23.429s，OFF 13.486/13.557s（复现 ±0.07s）→ ON median 23.46s vs OFF 13.52s = **1.74× 回退**（g3 基线 2.4×，收窄 29%；绝对开销 17.3→9.9s）。可比性三查过：模式行一致、dll 同版、fallback=0。
- ✅ **内核数据**：blocks9_real 5.796→3.723ms（1.56×，未达 ≥2× 判据）；phase 分解 sky_seed_bfs 48% / fill 24% / sky_fall 21% / export 6% / block_bfs 0.4%——剩余均为 O(N) 全域扫描（内存带宽型）。
- ✅ **judge 收尾**：APPROVE-WITH-CONDITIONS（C1-C4 全落实）；严格判据 FAIL 如实上报。
- ✅ commit 92d9b7b（种子收缩 + phased 探针 + bench/golden 产物）。
- 🔍 open：「e2e 不回退」严格判据待用户拍板；round2 候选（边界扫描融合 / fill 直读布局 / 均质 section 跳过）未排期；Java 收集循环 ≈5.6s 成 e2e 下一大头（secondary 方向）。

---

## 260905-04（D3 光照优化 round2：全扫融合 + 收集/JNI 直采）✅ 已结案（confirmed 2026-09-05 用户拍板；judge APPROVE-WITH-CONDITIONS 条件已全落实）

> 过程产物 `.investigations/light-opt/d3-opt-round2-260905-04.md`；结论 → 12 篇 round2 小节。

- ✅ **内核三项融合实施**：fill dom-major 重排 + col_max 同趟收集 / sky_fall 纯写趟（15-区间=[col_max+1,383]，逐位等价）/ 边界种子区间算术（2304 列×4 邻区间，与 round1 逐格扫描种子集合严格恒等）——内核 3.723→1.587ms/chunk。
- ❌→✅ **air 快路径 bug 一轮（golden 逐位门立功）**：首轮只判 id 位==0，漏 `id=0 + luminance>0` 合法光源（`15<<24`）→ 合成 golden 立即抓出；修复 = 判全字 v==0。修正后复测 1.587ms 不变（golden_pre 重冻自 HEAD 92d9b7b，C4 4 用例逐位 PASS）。
- ✅ **Java section 直采 + ThreadLocal + JNI thread_local/u8→i8 视图**：双采集对拍（4 chunk）= 4× MATCH 0/98304 diff，对拍后诊断移除复编通过（judge must 条件）。
- ✅ **e2e 四臂交替**：ON 中位 17.4s vs OFF 13.9s = **1.25× 回退**（round1 1.74×，绝对开销 ≈3.5s ≈ 内核份额预测 3.2s）——Java 收集/JNI 侧开销基本消除。
- ⚠️ **偏差记录（RCON）**：本轮 enable-rcon 已被重置 off → e2e_run stop 失败、世界被强杀；Done 计时在 stop 前完成，数值有效；已恢复 enable-rcon=true + rcon.password=coreswap（备份 run/server.properties.bak-g3，光课题收口时再复原）。
- ✅ **judge**（review-d3-round2-260905-04.md）：APPROVE-WITH-CONDITIONS；should-fix 三项均落实/声明；后用户拍板 **confirmed**。
- 📝 **降级声明**：Java/JNI 侧收益未单独微基准（运行时验证须实机），以 e2e 差值为证据；OFF 基线 ±1.5s 波动属噪声。
- 🔍 open：light 课题 .artifacts/index.yaml 登记（收口归档时补）；fill 仍 58%（palette 级批量展开 / opacity u8 表内联，未实施）；首载漂移 7× 待办（承 round1）。

## 260905-05（feature parity 课题：全天工作块——架构批准 → V1 验证 → fan-out → Phase 4 移植 → A/B 定案）✅ 判据链闭合（candidate，judge APPROVE-WITH-CONDITIONS；confirmed 已拍板）

> 过程产物 `.investigations/feature-parity/`（phase4-status + phase5-interim-260905-05.md）+ `.artifacts/feature-parity/`（b1/b2 候选 + judge-verdict-260905-05.md）+ `.tmp/feature-parity-260905-05/`（导出脚本 + v8_ab_verdict.py 判决脚本）。

- ✅ **架构批准**（Phase 0）：feature parity 立项独立课题（源自光照课题 G2 收敛的 feature 放置分歧转出）；tree 解禁 + 矿石规则层边界划定。
- ✅ **V1 遗产验证**：G2 遗产签名复现——vanilla vs 旧 dll（g1）= 155,470 差异实例 / 1333 可比 chunk（=117/chunk），主签名 leaves/vine/log（Y4-6，10k 级）+ 中等 blob。
- 🔍→✅ **双 scout + fan-out**：blob 差异定量切割后互斥候选 ≥2 → fan-out b1（count/IntProvider 采样）/ b2（modifier 链消耗 + discard_chance）。
- ❌ **b1 DENY**：count/IntProvider 均 JSON 常量，无可达作用面；算术不支持（+142.6 块 ≈ +2.23 blob vs 尝试 2.17/chunk）。
- ❌→📝 **b2 基本 DENY**：discard/count 子机制排除；set_decorator_seed 每 feature 独立播种，patch RNG 改动不传播（静态对拍结论仍有效，反向支持改动无大面积影响）。
- ✅ **S1 一手源 + Phase 4a/4b 应用**：tree.rs ~700 行移植 + biome filter + trapezoid height + BiasedToBottom IntProvider + feature_loader/placement/worldgen_handle 配套改动。
- ❌→✅ **「14.8× blob 回归」被取代（本工作块核心转折）**：初判「新 dll 2,298,206 实例 = 14.8× blob 爆炸回归」经三链互证推翻——①b1/b2 双 DENY 无可达作用面；②服务器考古（WG_FEATURELOG 硬开）：[CppBridge] init 发生在 Done 之后 → 启动区 = vanilla 装饰（knowledge #50）；③零 rust 参与正向对照：ab-vanilla vs g1-vanilla = 2,292,126 实例差、签名与「回归」完全一致 → **完成度伪影定案**（knowledge #49，g1 vanilla 参照缺 blob 装饰层 + 2044 空柱，3377 全域口径 60% 空对空不可比）。§15.4 取代记录落盘，原结论保留未改写。
- ✅ **opacity clamp 修复**：light_data.json 负 opacity 显式化（613-629 条 -1→15、954 条→0），light/mod.rs clamp 注释 + 防回归单测（cargo test light:: 2 passed，judge 实跑复核）。
- ✅ **受控 A/B 定案**：同方法论双臂（同 fresh world / 同 Done / 42-tile forceload 残差域 / region 收敛轮询 / 同停机拷贝，name 域口径）——ab-vanilla vs ab-takeover = **239,594 实例 / 1829 chunk（双侧完整 3009 可比）= 80/chunk** vs 旧口径 117/chunk，**净改善 ~32%**；主签名 = 树族（oak/jungle leaves + vine，Y4-6）+ 双向 ore 小残差（andesite/granite/diorite 各 1-1.2 万双向均衡）——与 idk-7 / R-1 已登记偏差源预期吻合，无 blob 爆炸。
- ✅ **judge**（judge-verdict-260905-05.md，MUST 级 + 独立复算）：APPROVE-WITH-CONDITIONS——v8 复算 3009/1829/239594 逐位一致；S1（证据包过期→已刷新）/S2（index.yaml→已补 FEA-1~5）/S4（原始输出→已归档 cmd-output/）核销；S3（light_data.json 被 .gitignore data/ 域忽略，版本控制策略待用户拍板，最重）未核销。
- 🔍 **残留 80/chunk 基线已立**：239,594/1829/3009 为 candidate 基线，后续 R-1/idk-7 收敛以此口径对表；per-feature 随机序列改变未逐 feature 定界（相对结论非绝对结论）。
- 📌 **open**：S3 data/ 管理策略（须用户拍板）；树族残差 R-1/idk-7 收敛立项；7× 首载漂移（承前）。


## 260905-06（实际 2026-09-05：树族残差收敛 — idk-7 实装 + 四 bug 修复 + 载体转折）✅ 判据链闭合（candidate）

> 过程产物 `.investigations/feature-parity/260905-06-errors.md` + `.tmp/feature-parity-260905-06/`；结论 → 13 篇 260905-06 小节；通用模式 → workflow-patterns #51/#52、build-tooling #23/#24（草稿）。

- ❌→✅ **cargo rlib 陈旧假绿一轮（定位链污染源）**：修复后 `cargo build -p worldgen` 多次 `Finished` 但 `libWorldgenRust.rlib` 未重编（mtime 数小时前）→ 新旧代码链同一陈旧 rlib 得「dump 逐字节一致」假象，浪费一轮对拍；字符串核验（新日志串缺失）实锤。修复 = 显式 `-p WorldgenRust` + mtime 核验判据（build-tooling #23 草稿）。教训：先破构建陈旧再谈行为一致。
- ✅ **噪声基线前置测量（课题裁决载体转折）**：同方法论两批纯 vanilla 臂互比（v9）= **241,080 实例**，与「接管 vs vanilla」信号（239,594）**同阶且签名同形**（air/leaves/双向 ore）→ 跨 run A/B 只能验「无回归」，无量级裁决力（workflow-patterns #51 草稿）。
- ✅ **确定性区域 dump 载体落地**：`worldgen-core/src/bin-diag/idk7_region_dump.rs`（bin-diag 隔离）纯实现生成整域 + 原始 id dump + blocks.json 映射 name 域 + SHA256 对拍（v10-v12 脚本链）——代码 diff 成为唯一变量（workflow-patterns #52 草稿）。
- ✅ **idk-7 实装 + 四 bug 修复**：generate_nested 接线（placed/configured 语义修正）、BlockPredicate `"predicate_type"`→`"type"`（谓词全灭根因，Discriminant 打标定位）、selector default 键修正、heightmap off-by-one——五段式台账 `.investigations/feature-parity/260905-06-errors.md`（含错误→根因速查表）。
- ✅ **jungle placers 实装**：jungle/mega jungle placer 补齐。
- ✅ **残差裁决**：树族残差 **171,442→118,697（−30.8%）**——确定性 dump 口径（载体 = region name 域对 vanilla06 快照 / 覆盖面 = 2193 chunks 全域 / 可比性 = 同快照同脚本，§9.7 三要素随行）；跨 run A/B 口径（80/chunk）降级为回归门，两口径不可互换。
- ✅ **gitignore 白名单修障**：`data/` 目录级规则 prune 使 `!` 重包含失效 + `data/*` 锚定语义坑——目录放行→内容重排除→文件白名单三段链式修复，`git check-ignore -v` 双向核验纪律（build-tooling #24 草稿；承接 260905-05 judge S3 阻塞项根因面）。
- 🔍 **open（残余清单更新）**：R-1 桶序（未核销）、vines feature（新）、distance 属性位（新）、树位置对齐（per-feature 随机序列定界）、anchor_biome 口径、fancy_oak 验证口径待拍板。
- 🔍 **open（流程）**：knowledge #51/#52、build-tooling #23/#24 草稿待主会话应用 + judge；13 篇小节待应用。

## 260905-10（实际 2026-09-05：P2 逐树对拍 → fan-out bA/bB → beehive 实装 → 三臂量化 → WG_CA_MIN 口径修正）✅ 根因链闭合（candidate）

> 过程产物 `.investigations/feature-parity/260905-10-{interim,oak-p2-bA,oak-p2-bB}.md` + `.tmp/feature-parity-260905-10/`；结论 → 13 篇 260905-10 小节；通用模式 → workflow-patterns #53、build-tooling #25（草稿）。

- ✅ **阶段 0 执行体三元组核对（#36 判据）**：release dll（18:12）不含 WG_CA_MIN 哨兵字节、rlib（19:17）含——ca0/ca1b 载体实为 bin-diag 单编 × rlib，不经 gradle runServer 的 dll；新登记风险：薄壳 dll 落后 rlib，runServer 采集前必须 `cargo build --offline -p worldgen --release` + processResources（rlib mtime > dll mtime 即红旗，#23 家族）。
- ✅ **P2 逐树对拍（决定性数据）**：p=20 trees_birch_and_oak 树1 base (602,-244) 两侧重合、树2 起全部错位 → 级联正面证据，分叉在树1 generate 内部（p2_first_tree.py 等）。
- ✅ **fan-out bA/bB 收口**：.bA（top-position/trunk 语义差）否证；.bB（selector/provider/replaceable）排除——**范围外高优先发现 = Rust 漏实现 BeehiveTreeDecorator**（每棵树必少抽 1 次 → getHeight 输入漂移链，同时解释 6vs7 + 树2 起漂移）。
- ✅ **BeehiveTreeDecorator 实装**：tree.rs Beehive 变体 + parse + generate（0.002 恒 1 次门 / bee_nest 朝南 / 蜂数两段消费）；修复后树1/树2 base 逐位重合；IDK-bee1/bee2 留档。
- ✅ **region 三臂量化**（#52 载体，2193 chunks，v18 口径）：bee-only oak +28886 / bee+CA 最优（oak +30807、jungle_l -12055、vine +1972、birch -973，较 ca1b birch 再收敛 739）；bee_nest 0/0 两侧一致；哈希留痕两份。
- ❌→✅ **WG_CA_MIN 口径修正（第二犯，判据升级）**：采集臂按「默认关」设计对照实验，实际 worldgen_handle.rs:876 `env_enabled` = 默认开，两臂 dump 全等（4D216088…）暴露——workflow-patterns #53 强制三查（消费点直读/声明对照/行为化哈希哨兵）。另修正 pfix「树1 7 根 log」读数错误（y=78 是 leaves）。
- ✅ **E-cA2 vine 镜像配对**：net +2127 掩盖 ~3.5 万块整体位移（excess 36033 / missing 33906）；依附配对分桶 a_log_both 3% / b_log_rust_only 28% / c_no_log 68% → 写/读腿时序域贡献可忽略（3% < 20% 阈值），vine 主体差与树结构级联同源（b1 域合流）。
- ✅ **WG_TREEDIAG Java 通道三坑**（build-tooling #25 草稿）：①JAVA_TOOL_OPTIONS -D 是 sysprop 非 env → -Ptreediag=1 → vmArg 映射 ②非 cancellable 方法 setReturnValue → CancellationException，改双 ordinal @Redirect/peek ③SEEDLOG 全量噪声 ~17min + spawn 预生成 20min+ → 整改方向 mixin chunk 过滤。
- 🔍 **open（残余清单）**：jungle_l -12055（beehive 不覆盖域）、树3+ 状态依赖短路（四角短路族）、R-1 HashSet 桶序、IDK-bee1/bee2、mixin chunk 过滤未实施。
- 🔍 **open（流程）**：workflow-patterns #53、build-tooling #25、13 篇 260905-10 小节草稿待主会话应用 + judge；candidate-beehive 待 judge + 用户拍板。

---

## 260905-12（实际 2026-09-05：feature parity——mixin chunk 过滤改造 + patch_grass 系内联 configured 修复 + badlands 参照区残差边界划分）✅ 修复完成 judge 通过（candidate，APPROVE-WITH-CONDITIONS 条件已核销；confirmed 待用户）

> 过程产物 `.investigations/feature-parity/260905-12-patch-grass-inline-fix.md`（架构计划 `.investigations/000-架构设计/架构计划-260905-12.md` 方案 C 已批准）；产物档案 `.artifacts/feature-parity/candidate-patchgrass-260905-12.md`（FEA-11）；通用模式 → compiler-idioms 发现 #16。

- ✅ **mixin chunk 过滤改造**：SEEDLOG/TREEDIAG Java 通道全量噪声整改落地（承接 260905-10 #25 整改方向，runtime/ local-only）：新增 wg.bench.WgDiag（ThreadLocal 当前 chunk + WG_DIAGCHUNK/wg.diagchunk 目标过滤 + 一次性 banner），population 行恒写 curChunk、THJ/BEE 走线程当前 chunk 门、CNT/SQX/SQZ 走 pos chunk 门，SEEDLOG population 行不滤（锚点）；编译绿。运行时效果验证（噪声 -99% + banner）并入下轮 java 采集（🔍 未验）。
- ✅ **patch_grass 系空 id miss 修复**：NEXT_SESSION 未闭合课题 #3（`generate_nested: unknown id (placed+configured miss): ` 空 id）——根因 = **24 内联点位**（18 patch_*.json + 6 flower*.json）的内嵌 `feature` 字段为**内联 configured 对象**（Holder.direct），`parse_inline` 只支持字符串形态 → 静默吞成空串 id → cache 双 miss。修复 = `PlacedFeature.inline_configured: Option<Box<ConfiguredFeature>>`（Box 断递归环）+ object 形态直接解析持有 + generate 侧直发优先于查表（generate_nested inline 分支加死防御注记：当前数据 0 个 placed 内联对象不可达）。基线对照（stash/pop 单变量，features_probe release × recheck 6×6 参照）：**miss 8766 → 0**（match 94.99% → 94.78%）。环境注意：debug 构建 chunkrandom.rs:169 溢出 panic 为既有问题非本修复引入；rlib mtime 红旗本轮为 #6 假阳性（fs::copy 保留 mtime），已用内容指纹核验。
- ✅ **badlands 参照区 −0.21% 残差边界划分（§9.7 + judge #3 因果标注）**：修复后 rust 在该参照区新放 7241 grass + 261 tall_grass、vanilla 全 0——**修复激活放置后暴露的上游差异，非本修复语义错误**（RNG 中性已证：BlockStateProvider Simple/Weighted 消费 0/1 同 Java，judge 专项核对 tree.rs:49-63），属「patch feature 选择 / biome 门」域；fan-out 候选 4 项（① biome feature 列表数据/映射差 ② dripstone_caves step9 patch_grass_plain + biome 门差 ③ patch_grass vs patch_grass_badlands 选型差 ④ unsupported placement modifier + features 顺序/global_index 差）→ 按分叉即 fan-out 纪律**不在主会话自推**，留待专项（可与 jungle_l 采集共用 java 侧证据）。match 口径声明：features_probe vs vanilla FULL 6×6，与 v18 口径（2193 chunks treediag）不可比。
- ✅ **judge 审查**：APPROVE-WITH-CONDITIONS（260905-12）——三源核对通过 + RNG 中性专项证明；条件三项（index 登记 FEA-11 / 「19 个」→24 计数更正 / generate_nested 死分支注记）当场核销。
- 🔍 **open**：patch 选型/biome 门专项未立项（FEA-11 残差 4 候选）；mixin 过滤运行时效果待验；confirmed 待用户拍板。

---

## 260906-03（实际 2026-09-06 14:12 起工作块：BUG-002/issue #24——勘探 → 廉价验证 → 卡片 v3 合并 → 方案批准 → 实现 → Forge+Connector 生产验证）✅ 修复完成 judge 通过（candidate，confirmed 待用户）

> 过程产物 `.investigations/multiworld-bug002/`（scout-map / worker-verdict / 方案 / cmd-output V1+V2）；通用模式 → workflow-patterns #55（草稿）；BUG 卡回写 → CoreSwap-Maint .artifacts/bugs/BUG-002-custom-dim-surface-skip.md（草稿待应用）。

- ✅ **scout 勘探**（scout-map-260906-03.md）：接管拦截面全景——维度判定只有两类「形状指纹」（-64/384→overworld、0/256 且 !wgIsEnd()→nether），零 registry 识别；Rust 硬编码点 H1-H10 清单（生产端 create_for_dim 已参数化，缺口在 Java 判定层 + mod 维度数据不在资源层）；末地子疑点 = wgIsEnd() 裸反射失败静默 false → end（同 0/256 形状）被 nether 句柄误接管。
- ✅ **worker 裁决 + V1 廉价实证**：worker-verdict-260906-03.md 判定「wgIsEnd() 反射在生产必失败」——字符串字面量 "biomeSource" 不被 remapper 重写，Forge 生产 SRG 成员名下 NoSuchFieldException → catch 静默 false；V1 grep ForgeGradle 缓存 tsrg 实锤 `f_62137_ biomeSource`（行 180533）——**judge 独立重跑勘误：首引 f_226623_ 为 Structure$GenerationContext 同名巧合，勿再引用**（cmd-output/v1-tsrg-biomeSource-260906-03.md）。
- ✅ **BUG 卡 v3 合并**：卡片「引擎 4 硬缺口」（blocks.rs:41 unwrap_or(AIR)、default_block minecraft:stone、data/minecraft 命名空间硬编码、biome 表仅原版）+ 本块末地误接管机制闭环 = 双机制互补；4 缺口归长期多世界数据驱动线，不阻塞短期放行策略。
- ✅ **方案批准**（方案-260906-03-放行策略.md，000-P2，用户拍板）：**资源感知自动放行**路线——按 noise_settings 注册 id 判定（settings RegistryEntry key，符号引用）替代形状指纹 + 裸反射；数据在资源层才接管、不在则放行 vanilla（mod 维度误接管机制性不可能）；end 暂放行 vanilla，end 接管 = 下一里程碑。
- ✅ **实现**：NoiseChunkGeneratorMixin 维度判定重构——wgSettingsId()（RegistryEntry key）+ 接管集 {minecraft:overworld, minecraft:nether}；裸反射 wgIsEnd() 整体删除（静默 catch 同步消除）；放行维度打一次性说明日志（Set 去重防刷屏）；buildSurface 同步收紧。
- ✅ **V2 生产验证**（cmd-output/v2-forge-connector-verification-260906-03.md，Forge 47.4.5 + Connector beta.49，seed 7691421705105351955 与报告者同种子）：7 判据全 PASS——① mixin 生产 APPLY 全绿（首启曾 APPLY FAILED：refmap 包路径笔误 world/gen/chunk/Chunk→world/chunk/Chunk，修正二启绿）② end `release to vanilla: settings=minecraft:end` ③ 末地零 nether-intercept（旧误接管指纹消失）④ end 地形 vanilla 化（y1=air/y55-58=end_stone，bedrock 未命中）⑤ nether 接管不回归 ⑥ overworld 接管不回归 ⑦ 因果端点闭合（V2 补齐 V1 遗留的运行时日志端点）。§9.7 降级声明随行：mod 维度未本地装载实体复测、未做报告者口径全维度 region A/B diff。
- ✅ **judge 审查**：三源核对（.investigations 快照 + 代码 diff + V1/V2 验证记录）通过；f_226623_ 误引勘误；V1 边界（因果端点待 V2）已由 V2 核销。
- 🔍 **open（降级/边界）**：mod 维度放行与 end 走同一代码路径（settings id ∉ 接管集）机制同源但未本地实体复测；报告者整合包 A/B 复测待报告者执行；end 接管（Rust end 管线 + biome 判定）= 下一里程碑；overworld 变体 settings（amplified 等）现被放行 vanilla（行为较此前被 overworld.json 错误接管更正确，边界已注记）。

---

## 260906-04（实际 2026-09-06 15:43 起 Get-Date 锚定：end 接管开发块——摸底 → 轻量架构 → G1-G3 开发 → 对拍 → 生产验证 → confirmed）✅ 已确认（用户授权确认；judge MUST review-002 推荐可 confirmed）

> 过程产物 `.investigations/end-takeover/`（00 摸底 verdict / 01 rust 现状 / 02 biome 规则 / 03 资源闭包 / 001 架构计划-dev / 260906-04-errors.md 五段式台账 / review-001 摸底 judge / review-002-final 收尾 judge）；通用模式 → workflow-patterns #56-#59、compiler-idioms #18/#19（subagent 草稿 → 主会话应用）。

- ✅ **摸底（scout + worker subagent，Partial 静态审查）**：NEXT_SESSION「create_for_dim 硬编码 overworld」方向描述经廉价验证判定**已过时**（260905-01 拆分后已全参数化，nether 实证跑通）——缺口定性中小，轻量架构 3 要点（Rust 内核 / Java 接管 / 验证闭环）经用户批准。硬缺口 = `minecraft:end_islands` DF 节点（含 SimplexNoiseSampler ~100 行新写）+ EndBiomeSource 位置判定分类器（非 MultiNoise，假设已验证：只读 erosion，阈值三段 + 中心 4096L，02 篇规则提取齐全）。
- ✅ **G1 Rust 内核**：SimplexNoiseSampler（复用 noise.rs GRADIENTS 表）+ EndIslands（CheckedRandom(seed)+skip(17292)，worldSeed 直传无 split）+ density_builder 注册 `minecraft:end_islands`；EndBiomeSource 分类器 + create_for_dim 按 settingsName=end 路由（BiomeClassifier::empty 分流，end 无 biome_params）。
- 🔍→✅ **错误链 E1-E3（台账五段式）**：E1 新 match arm 落 catch-all 后静默失效；E2 Java long 乘法回绕改 wrapping（debug 溢出断言=免费回绕审计器）；E3 空列 heightmap 哨兵 MIN 直入 build_surface（end 全空气列特有，消费点映射 min_y-1）。
- ✅ **E4/关键发现——插值 cell 尺寸硬编码**：DensityMacroSampler 硬编码 cell 4×8（overworld size 1/2 值），end size 2/1 应为 8×4 → 存档 A/B 岛面系统性 +1y（16385 air mismatch）。修复 = with_cells 参数化（settings.size_horizontal/vertical ×4），overworld/nether 参数恒等故行为恒等（judge PASS-3 静态复核）。
- ✅ **E5/生产坑——维度形状 ≠ noise.height**：end chunk 实形 0/256 与 nether 同形（noise.height=128 是域混淆）；settings id + endActive 双闸唯一判别（zeroShape 改名编码事实，workflow-patterns #58 详写）。另：Forge 维度存档路径 = `world/DIM1|DIM-1`（非 Fabric 式 `world/dimensions/<ns>/<dim>`）——清维度清错路径 = 静默 no-op，v1/v2/v3 存档污染根源。
- ✅ **三端对拍链（同 seed 7691421705105351955）**：组件级 SimplexNoise/EndIslands 4 seed 数值全等（Java EndProbe 自包含真值 vs Rust；含 -0.84375/0.5625 边界点；跨语言比对必须数值化——格式假阴性教训 #59）→ 引擎级 rust dump vs vanilla 存档：首跑 35520 差（cell +1y 为主）→ cell 修复后 18529 差（100% = vanilla 黑石柱/传送门 feature 域，地形层零差）→ **生产端到端（Forge mod jar vs vanilla 臂 DIM1 存档）：36/36 chunk 逐位 0 差异**（ab_regions.py palette↔id 逐块全等，y0..127 口径声明 §9.7）。
- ✅ **judge 终审（review-002-final，MUST 级）**：推荐 confirmed——三源核对 7 项全 ✅；**哈希更正声明：终版 dll sha256 = 1B5AA1DEA49445A2…（2160640B），早期记录 96AD0411 为 cell 修复前旧 dll**。
- ✅ **收尾跟进（用户 confirmed 前/中执行）**：① nether 共享路径回归——vanilla 臂 vs 1.0.25 基线 dll = 712 差、vs 1.0.26 新 dll = 914 差（basalt/blackstone 家族，同区域 36 chunk），差异量级在既有「同 jar 重跑非确定容差」（#10 家族）范围内，且三处共享改动对 nether 静态恒等（cell 参数恒等/wrapping 位模式恒等/哨兵映射 nether 不可达）——判无回归；② README.md / README.zh-CN.md 更新（end 接管 + Forge 生产级 + WorldgenRust→worldgen-core 路径 + 1.0.26）。
- 📌 **open**：报告者 1.0.26 复测反馈按 BUG 卡契约处理；D3 SHOULD judge 缺席记流程台账（被 MUST 覆盖）；对拍输出文件建议内嵌 seed/argv 头注（产物自证，下轮执行）。

---

## 260906-07（实际 2026-09-06 工作块：jungle-l mega 双向分歧 fan-out 收敛 + V1 决定性复验 + 实机载体身份厘清 + judge 审查）🔍 阶段性结案（B1/R1 升 candidate 命题限定版；#65 口径修正待取代记录落盘；E2a/E2b 未做）

> 过程产物 `.investigations/jungle-l/fanout-260906-07/`（.b1/.b2/.b3 / e0-neighbor-matrix / v1-vanilla-client-evidence / judge-verdict，脚本 `.tmp/jungle-l-260906/e0_neighbor_matrix_260906-07.py`）；通用模式 → workflow-patterns #63/#64/#65 + #13 家族补充案例（草稿待主会话应用 + judge）。

- ✅ **实机载体身份厘清（用户确认，最高优先发现）**：260906-05「实机 modded」实例只装 CoreSwap mod（Rust worldgen 接管）——原「实机 11 点真值」实为 **Rust ca_min=off** 生成数据，非 Java vanilla。后果：①「实机 vs pregen 2/11 双向分歧」= Rust vs Java 跨执行体对比（#62 口径违例根源，当时误当 Java 基线；#62 判据本身不变，表述待限定）；② **505 孤例消失**（=Rust ca_min=off 正常输出，on 才有 505 Y，.b2/.b3 归属竞赛随之消解）；③ .b2/M2「接管 mod 在实机端」坐实；④ j5-baseline:21「用户实机 vanilla 世界」需 §15.4 取代记录；⑤ V1 的 (469) 翻转证据是 Java-vs-Java，不受影响反而更纯粹。
- ✅ **V1 -Vanilla 客户端复验（用户实跑，决定性）**：loom+fabric-loader+fabric-api、vanilla worldgen、无 Rust dll，同 seed（8576294172403134396）旁观复验——(469,71,-230) 与 (505,72,-261) 均 Y（Targeted jungle_log + /seed 截图闭环）；对照 pregen（Java vanilla）469=N/505=Y → 「同 seed 同代码、不同加载载体 → mega 集单点翻转」运行时直证，B1/R1 升 candidate（命题限定版；judge N4：集成服 vs 专用服第二变量须声明；loom -PcppVanilla≠纯 vanilla）。
- ✅ **E0 邻居完成序矩阵**：pregen 内 25 chunk FEATURES 完成序 = 工作线程调度序非空间序，分歧 chunk 一个早跑邻未完成、一个极晚跑邻已完成——与交叉表形态吻合；judge 实跑 25/25 复现（可升 candidate，待修 N3 计数 3→4）。🔍 caveat：行号序≈时间序仅秒级精度（多 worker stdout 队列微扰），结论用远离序对故稳健。
- ✅ **judge 审查（judge-verdict-260906-07）**：无阻塞级问题；勘误 N1——.b3「(505) MJTD/MJTG 零命中」系转录失真（实测 31 命中，(469) 才是真零命中）——#13 家族补充案例；N2-N5（坐标标签/E0 计数/「纯 vanilla」表述+第二变量+n=1/截图原件落盘）待主会话回写。
- ✅ **观察误差成本实证**：用户一次未开旁观的目测复验即看错（后自纠）——B3 权重自判「弱」被 judge 回调至「中」；11 点表观察条件未确认旁观，精度降级标注（#64；叠加 #65 后该表为双重口径失真样本）。
- 🔍 **open（judge 下一步最小闭环，经 #65 厘清后重排）**：~~E2a 同协议 pregen 复跑（run 级确定性钉死）→ E2b 分批 forceload（B1 机制判别）；-Vanilla 客户端复跑 n=1→2；j5-baseline §15.4 取代记录落盘；N1-N5 勘误回写；ca_min 决策保持冻结待 E2b 同口径 A/B。~~实机 mods 清单（.b2 E1）/505 孤例归属~~（经 #65 消解/坐实，撤销）。（E2a/n=3/T4/F3 已于 260906-08 执行完毕，见下条。）

## 260906-08（实际 2026-09-06 20:15 起 Get-Date 锚定：jungle-l E2a 同协议复跑 → run 级非确定直证 → F3 执行语义判定 → 三项 confirmed 收口）✅ judge 通过 + 用户拍板三项 confirmed

> 过程产物 `.investigations/jungle-l/e2a-rerun-260906-08.md`（必读主文档，含 judge N-A/N-B/N-C 应用记录）；脚本 `.tmp/jungle-l-260906/j5_java_e2a_260906-08.ps1` + j5-java-e2a-260906-08.log（协议 = j5_java_baseline_260906-06.ps1 原样，逐行 diff 证实）+ j5_rust_caminoff_260906-08.ps1；通用模式 → workflow-patterns #66/#67/#68（subagent 草稿 → 主会话应用，日期自推 09-08 已实锚修正）。

- ✅ **E2a 同协议复跑 → run 级非确定直证**：同 seed（8576294172403134396）同代码同协议两 run，mega 集基线 15 vs E2a 19，交集仅 7；469/505 关键点在两 run 间即翻转。四查 PASS（含第四查 #62 Xoroshiro 判据复算逐位命中，双 population 行并存为已知双调用点签名）。E0 矩阵零成本复用：两 run FEATURES 完成序完全不同、8 邻先完成数 0~5 非平凡——调度序 run 间漂移直证，B1 机制表现面第二组数据层证据（#67）。
- ✅ **用户实机对拍 n=3（受控观察 #64 合规）**：-Vanilla 客户端第 3 样本对 7 稳定交集点命中 6、**479,71,-216 翻 N**（橡树）——「7 稳定交集」降级「6 稳定 + 1 波动」，Java 分布形状 = 小稳定核 + 大波动外围；实机 modded 样本命中 3 缺 4（后续 F3 重定性）。
- ✅ **T4 native off 受控重采**：`j5_tree_trace2.exe`（确定性顺序生成无调度噪声，exe 17:56 晚于全部源码，新鲜度哨兵 PASS）——Rust-off native 集 11 棵，**与 260905-05 预测清单逐位吻合**（含 469 Y / 505 Y）→ native off 生成确定性跨 run/跨月稳定；Rust-on（260906-06）16 棵与 off 交集仅 4。dll 版本确认：target/release/worldgen.dll 2,160,640 bytes，sha256 1B5AA1DE…，与 1.0.26 jar 内 native/worldgen.dll 逐字节一致（双重 hash 校对）。
- ✅ **缓存/env 嫌疑排除（用户核对客户端 log + 世界存档）**：实机世界 20:51 新建（全部 region 当日创建）、dll sha 同、无 WG_CA_MIN env、log stageMask=3 enabled=true——冲突迫使重查 wiring。
- ✅ **F3 fan-out 判定（candidate ~0.9，Degraded 静态审查；judge 四代码锚点逐一核实）**：① mixin 只拦 NOISE/SURFACE，feature/carver 阶段无拦截，Java vanilla 装饰器照常运行；② stageMask=3 = SKIP_CARVER|SKIP_FEATURES（worldgen_handle.rs:143-145），Rust 侧跳过自己的特征放置；③ native bin-diag 从不 set_flags → flags=0 特征全跑——**native 与 vivo 是两种执行语义（#36 家族最重形态，#66）**。综合：vivo live 树 = 纯 Java vanilla 树长在 Rust 地形上（第三种生成器）；中间误判段（执行体归属冲突）已被 F3 取代存档。judge N-B 降级：483 归因两通道（特征流执行序 vs 地形输入系统差）未分解，维持 draft 候选 → #68。
- ✅ **judge 审查（三源核对 + 数据抽查 5+ 点零失配）**：通过，非阻塞 3 项已应用——N-A 勘误作废 F3 前旧判读（「Rust-off 三点缺失利好 ca_min」彼时误将 vivo 当 Rust-off）；N-B 归因分解挂靠 #10/#14 家族；N-C MJT0 清单区域过滤口径声明。
- ✅ **用户拍板三项 confirmed**：① features 对齐非目标 + 出货架构（Rust 地形 + Java features/carver）= confirmed；② ca_min moot 化（挂起，材料保留给全接管配置）= confirmed；③ carver 维持 mask=3、接管列为 perf 候选课题（前置：残差归因——Rust carver 90.88% 挖洞重合未分离地形级联——+ 非 vanilla carver 检测回退 + replaceable 数据驱动化；触发条件 = profiling 占比值得）= confirmed。F3 wiring 判定 = candidate（N-B 开放）。
- ✅ **#65 再取代提案（candidate，confirmed 方向）**：实机 modded（mask=3 默认 0b011）实为 **Java 特征 × Rust 地形**，既非 Rust-off 也非 Java vanilla——260905-05 的 11 点真值同此重定性，保留「260905-05 当次 run 的 stageMask 待核」条件；走 §15.4 取代链。
- 🔍 **open**：#66-#68 知识库条目 confirmed 待用户复审；F3 两通道归因分解（廉价臂 = native Rust 地形 + Java 特征探针）；（可选 perf 课题）carver 残差归因 + 占比 profiling；E2b 分布对分布实验（或接受 run 级非确定已坐实而撤销）。

## 260906-09（实际 2026-09-06 23:39 Get-Date 锚定：jungle-l F3-1 判别臂两 run 采集 → 载体 PASS 复核 → 483 两通道归因 → judge 有条件通过）🔍 阶段性结案（通道② draft ~0.55；主判别臂未实施；500-234 直证待复现）

> 过程产物 `.investigations/jungle-l/f3-twochannel-260906-09.md`（worker 判定主文档，含 §9.7 口径声明）；日志 `.tmp/jungle-l-260906/j5-f3rust-260906-09-f3r1.log` / f3r2.log（参照 260905-13 stagemask3.log）。

- ✅ **F3-1 判别臂两 run 采集**：modded 载体（stageMask=3 = Rust 地形 + Java vanilla carver/特征）同 dll（1b5aa1de…，=1.0.26 出货）同 seed 同协议 f3r1/f3r2 两 run；载体四查全项 PASS（stageMask/dll sha/seed/population 1154 行/双 popseed 签名/时间窗），另补行为面论证——modded run 间集合相似 13/24≈54% 远高于 modded vs Java 8/30≈27%，行为上证明地形层为 Rust 系（judge 勘误原稿分母）。
- ✅ **通道①（特征流执行序 run 级非确定）存在性直证（~0.95）**：f3r1∩f3r2 仅 6/31（分歧 81%），run 对分歧率样本 46%–81% 且发现**非可交换性**（跨月跨 dll 共享度反高于同日同 dll）——mega 集由 chunk 完成序路径依赖主导，单点概率模型（含主会话 ~0.25）只可作粗界。
- ✅ **500,75,-234 地形漂移直证（数据层，通道②实证支点）**：同 seed 同点位 260905-13 modded h19 vs f3r2 h13（差 6 格）——新旧 dll 柱级地形差存在，dll 地形版本差由纯假设升格「已证实存在 × 影响面未测」活跃变量。
- ✅ **native 执行语义源码澄清**：j5_tree_trace 从不 set_flags → flags=0 → carver+features 全跑（worldgen_handle.rs L725/L562）——native 臂 = 三全 Rust，「native off ≈ Rust 地形代理」口径不成立（#66 代码级依据），native 483 Y 对 Java 谓词零证明力。
- ✅ **两通道归因结论（draft）**：通道②（Rust 地形输入差/dll 版本差）参与 483 类分歧倾向 YES 置信 ~0.55（judge 建议自 0.6 下调，已应用）——支撑：native 反证力削弱 + 今日 483 柱地形盲区 + 500-234 直证 + 1.0.26 overworld 回归未重跑（发版单自认）+ 版本相关翻转在纯通道①下无自然解释。
- ✅ **judge 有条件通过（260906-09）**：条件披露 = 架构计划 T1 主判别臂（native Rust 地形 + Java 特征探针）本轮未实施，实际采集为两次完整 vivo 载体重复——归因分解完成一半，建议地形直比（今 dll native vs Java block_probe，chunk (30,-15)/(31,-14)）出结果后再决定是否需要。
- ❌ **修正主会话三项观察**：①「~0.25 概率」量化不成立（改「双缺在通道①框架下不异常但非零信息量」）；②「共享点 h 全逐位同」需限定 Java 共享点位——modded 共享点存在 500-234 反例；③全日志前缀序列第三棵即分叉，「与 905-13 逐位同」仅区域过滤口径成立。
- 🔍 **open**：①modded n≥3 重复采样盯 483（≥5/5 N → 通道②升 candidate；顺手复现 500-234 h13）；②地形直比（机制级，一步闭合通道②地形前提）；③N-B 原案隔离特征执行体判别臂；④init 晚于 Done 时序机制探针 + 905-13 dll→1.0.26 overworld 变更清单审阅；⑤共享点 h 全同 vs 500-234 反例张力是否 fan-out，待主会话评估。

## 260906-09 深夜（实际 2026-09-06 深夜/07 凌晨工作块：jungle-l Chunky 双臂地形验证试验 → 载体转正 confirmed + 效率实测 → 首战区域级量化 → 三候选 fan-out 预置）✅ 载体/效率 confirmed；归因 draft

> 过程产物 `.investigations/jungle-l/chunky-trial-260906-09.md`；采集 `.tmp/jungle-l-260906/chunky/`（双臂日志 + region-{coreswap,vanilla} 各 6 mca + diff 脚本）；载体条目 → build-tooling #26（subagent 草稿 → 主会话应用）。

- ✅ **用户实机受控观察（liveobs-260906-09，截图已归档）**：483/500 焦点柱两臂**地表一致**（Rust live 483=Y jungle_log；500 两臂脚下地面一致）——通道②（地形输入差）参与 483 类分歧的证据面收窄，通道②降级走 §15.4 取代链（对 F3-1 通道② draft ~0.55 的口径修正，judge review-260906-09-liveobs.md PASS）。
- ✅ **Chunky 双臂试验（用户建议）**：gradle runServer 同实例对称双臂（vanilla=`-PcppVanilla=1` / coreswap=stageMask=3）+ Chunky jar + 控制台命令 + region 程序化 diff；管线核验 PASS（#36）。
- ✅ **载体转正 confirmed + 效率实测**：~4min/1089 chunks vs forceload ~27min/25 chunks（>40×）、零人力、region 可复用——转正为区域级地形验证标准载体（单点 sanity 保留 tp+F3）；使用前置三查 = 管线核验 / region+slot 定位 / 对比剔植被（build-tooling #26）。
- ✅ **首战量化结果（全 seed 区域 33×33）**：有差 chunks **946/3025**；terrain=113k / veg=86k / air=51k——「Rust 地形=Vanilla」逐块意义上不成立，但量级属已知残差域（≈15 块/chunk）+ 焦点柱表层一致 → 定性为**已知地形残差域首次全区域量化**（非新差通道发现）。
- 🔍 **三候选 fan-out 预置（互斥，未归因，禁止单通道结论）**：① blob 残差（andesite/diorite/granite ≈46k，已知残差域）② 洞穴级联（air/water——carver 级联 vs aquifer）③ **结构差 = 新开放面**（mineshaft 组件 ±整件级 + geode——mixin 拦 NOISE/SURFACE 对结构阶段影响未核，价值最高，查 Structures NBT）；矿石 ±1-3 疑为①下游非独立候选。待并行 worker（主会话不自推）。



## 260907-01（实际 2026-09-07 Get-Date 锚定：jungle-l 柱全列直读 → chunky 地形差归因 fan-out（.b1/.b2）→ judge 有条件通过）✅ judge PASS-with-conditions（条件已应用）；无代码改动

> 过程产物 `.investigations/jungle-l/column-read-260907-01.md`（柱直读）+ `.investigations/jungle-l/fanout-260907-01/`（convergence.md + .b1/.b2 candidate.md + review-260907-01.md）；数据 `.tmp/jungle-l-260906/chunky/diff_per_chunk_260907-01.txt` + structures_nbt_260907-01.txt；脚本 `.tmp/jungle-l-260906/column_read_260907-01.py` + collect_fanout_260907-01.py。**本块无任何 src 代码改动**（judge 三源核对 N：git diff 仅 .investigations/.artifacts/knowledge/docs）。通用模式 → workflow-patterns #71/#72/#73（subagent 草稿 → 主会话应用）。

- ✅ **柱全列直读（开工点 1，region NBT 程序化直读，Full 载体）**：柱 (483,-230) 全列 y=-64→319 **零差**（地形+植被逐位全等）；柱 (500,-234) 差异 21 处**全部在 y=74-94 feature 层**（vanilla 整根 jungle_log + 树下 grass_block→dirt，coreswap 无树干），y≤73 地形逐位全等。
- ✅ **交接结论廉价独立验证（STEP 1 纪律）**：通道②（Rust 地形输入差参与 483 柱分歧）draft ~0.55 → **支持降级**（483 柱全列地形零差直证地形输入一致）；blob 石残差 ≈15 块/chunk 不触及焦点柱。
- ✅ **§15.4 取代记录**：「500 柱地面低 6 格」（f3-twochannel-260906-09 §5 树基高度代理推定）被本块 column-read 取代——region 载体下地形基座一致，差异为植被层。原结论未改写；**反向指针待 f3-twochannel-260906-09 归档时补注**（judge N-5 条件）。
- ✅ **fan-out 双候选（互斥：地形差主力归属）**：.b1「blob 石残差域」**支持**——类型谱（granite/diorite/andesite 双向均衡 ≈48.7k）、量级带（≈15-16 块/chunk，judge N-6 降格「同量级带参照」：口径混合）、簇状空间分布三证据与 13 篇已知记录吻合，无需新机制。.b2「结构阶段独立第 5 通道」**证伪**——starts 层零差 + 块级签名 ~330 块/3025 chunk 与整件级差先验差一个数量级以上（判据 → #71）；geode 两臂均 Java vanilla feature 放置（mixin 只拦 NOISE/SURFACE 源码引证），geode 差 = 通道①/②下游表现面。children BoundingBox 盲区 §9.7 声明。
- ✅ **air/water 桶归属（开工点 3）**：water 846 → aquifer 挂起域（11 篇）；cave_air 548 三分（geode 壳 ~180-260 / 结构语汇残余 ~330 未闭合 / 洞穴 carve 下游）——无独立新通道。全区域地形差完整归账已开封通道，**无第 5 归因通道**。
- ✅ **judge 审查（review-260907-01，PASS-with-conditions）**：N-3「5.1k」修正为 name 双桶相加上界（→ #72）；N-4 口径疑点补第三成因（单 palette ±2048 近似 + section 跳过盲区）；N-5 取代链反向指针待补；N-6 量级表述降格；N-8 .b2 内部数字勘误（130→~200）。收敛主结论「无第 5 独立归因通道」建议 candidate。
- 🔍 **open（未闭合采集项，不阻塞收口）**：① 13 mineshaft + 2 ruined_portal children BoundingBox dump（消解 ~330 块 cobble/cobweb/spawner 簇归属）；② grass_block↔dirt y 分布核查（树 below-dirt vs 表层 rule 差）；③ 全量对表重算（top8 截断 + ±2048 近似 + section 跳过三成因合并消解）；④ column_read 脚本两处外观 bug（air 前缀永真比较 / sanity 无条件打印，不影响差异数据）。
- ✅ **用户拍板（260907-01 收口）**：① 本块收敛结论（.b1 支持 / .b2 证伪「无第 5 独立归因通道」+ air/water 桶归属 + 500 柱取代记录）**confirmed**（index.yaml 两条目已回写）；② **E2b 进入「撤销观察」状态**——不立即执行撤销，挂观察：后续若再出现需要 E2b（分批 forceload 载体）的判别场景则复评，无场景触发即按期正式撤销（材料：E2a + f3 双侧 run 级非确定直证）。



## 260910-06（实际 2026-09-10 20:16 起 Get-Date 锚定：1.20.1 R3 异步化移植（重活移出 worldgen 单车道）+ 同构建态 A/B + 内容指纹门 + 出测试 jar）🔍 candidate（judge 两轮 = PASS-with-conditions：J1-J12 + C1-C6 均已应用；用户未 confirmed）

> 过程产物 `.investigations/perf-reg-260910-06/`（`errors-260910-06.md` E1-E5 五段式台账 / `cmd-output/` 10 臂日志 + `results.txt` + `diffs/`（旧读法）+ **`diffs-fixed/`（规范读法，权威）** + `region-*` 快照 + `chunky-task-state/` + 驱动与对拍工具 / `java-snapshot/post-260910-06/` + `MANIFEST-sha256.txt`）；架构计划 `.investigations/000-架构设计/架构计划-260910-06.md`（含 **§14 执行中变更**：D-b 实测证伪）；判决 `.artifacts/perf-reg-260910-06/verdict-260910-06.md`（**candidate**）+ `judge-verdict-260910-06.md`；通用模式 → workflow-patterns #113/#114/#115、build-tooling #51/#52/#53、compiler-idioms #24（subagent 草稿 → 主会话应用）。

- ✅ **范围与形态移植**：三分支（overworld/nether/end）`populateNoise` 接管段由「传入车道上同步执行」改为 `wgDispatch`——默认 `supplyAsync(work, WG_FILL_POOL)`（= `Util.getMainWorkerExecutor()`；1.20.1 无 `NameableExecutor` 故**不加** `.named()`），`-Psyncfill=1` 回退为内联 `completedFuture(work.get())`；Rust/C++ 零改动（执行体与 260910-05 同源 dll，逐臂 `[CppBridge] dll=` 核对）。
- ❌ **首版镜像 vanilla 的「sections 外层锁 + `whenCompleteAsync` 解锁」被实测证伪**：两臂都卡死（`Processed: 0`、CPU≈0、等 40min 不复原）——同线程**非可重入 `LockHelper` 自锁死**（**E1**）；**排除对照 = 同步臂也卡** ⇒「异步分派 / 完成跳回」变量被排除，嫌疑收敛到两臂共有新增项（外层锁）；修复 = 删 `wgLockSections`/`wgUnlock` 与完成跳回，退化为 1.21.6 R3 已验证的无外层锁形态。
- ❌ **E1 证据保全失误**：卡死轮日志在后续成功复跑时被同名覆盖，dump 原文只剩引文 ⇒ 归档仅存 `results.txt` 2 条 STALL 行 + 源码级锁链（judge 已独立复核该链）；**教训 = 卡死轮日志 MUST 复跑前另存**。
- ✅ **诊断件（门控默认关，生产零成本）**：`StallWatch.java`（`-Dcoreswap.stallwatch=<秒>`，daemon 自打全线程栈，`[STALLWATCH]` 前缀）= 沙箱 `jstack`/`jcmd` 全拒访下**唯一**的停滞观测面（本块靠它一次 dump 读出阻塞链）；`ChunkTiming.java`（1.20.1 精简版，只打非恒 0 分项）；`CppBridge` 新增 `wgBufHash` + `[WG-CONTENT]` 逐 chunk 指纹（MIXLOG 门控）。
- ✅ **驱动三修正**：① `gradle runServer` → **`gradle :runServer`**（裸任务名级联 `:content-test:runServer` 第二服务器共用同一 run 目录，**E2**）；② 每臂清 `run\config\chunky\tasks`（Chunky 任务状态跨 run 持久化，**E3**）；③ 新增**早停探针**（`chunky progress` 轮询，`Processed` 恒 0 超 180s ⇒ 落盘诊断后退出，替代 40min 盲等）。
- ✅ **形态直证 + 量级（同构建态单变量 A/B，同 dll sha / 同 seed / 同 region 背靠背）**：`inflight max` **1 → 23**；共 10 臂（sync ×3 / async ×4 / vanilla ×1 / nether + end sanity）；vanilla 同批在位 ⇒ 得「同步慢于 vanilla、异步快于 vanilla」的**同批**读数（绝对值以判决表为准）；`sumMixin`/wall 同尺翻转（≈0.9 → ≈10.7）⇒ 延迟换 CPU（延迟结论 ≠ 效率结论）。
- ✅ **行为门三层**：**第一层（决定性、噪声无关）** = 每 chunk 原生输出 FNV-1a 64 指纹跨形态逐 chunk 比对 **4140/4140 全等、0 不一致**（judge 自写解析器全量 join 独立复核一致）；**第二层（旁证，检验力低已声明）** = region 逐块对拍（自比 0 / 正对照非零 / 同形态锚 vs 跨形态；读法与判定预登记）；**第三层** = 工具自检（自比 0 差 + common 非空 + 正对照灵敏度）。
- ❌ **对拍工具两次缺陷（同一主题的两个形态）**：① 手写 NBT reader 与已验证版不一致（TAG_Byte 读 8B）⇒ 全域只解析出 **760** chunk **且不报错**（**E4**）；② 与已验证版**逐行对齐后**仍静默丢 **46/7749 = 0.6%** chunk——两版共有的 `tag7`（TAG_Byte_Array）长度按 1 字节读，规范为 **TAG_Int(4B)**（**E5**，本块在知识库读码复核阶段抓出）；修复后**重跑全部对拍**：跨形态读数由旧读法的 0.058-0.065% 修正为 **0.030% 量级**（与 async 同形态锚同阶）⇒ **工具缺陷会直接改写判据解读**；复审进一步实测口径差**远超 chunk 数的 0.6%**：section 并集 85,047→**185,976**、块分母 348,352,512→**761,757,696**（×2.187）、sections/chunk 11.04→**24.00**，且新旧映射**非单调**⇒ **上游 260910-05（1.21.6，已 confirmed）的数字必须复算、禁按比例打折**（已在其 §8 加「补充收敛（E5）」注记，复算登记为待办）。
- ✅ **nether/end 只做「接管生效 sanity」**（全维行为门递延）：`intercepted` 与 `[WG-FILL]` 行数各 4761/4761、`inflight max=23`、无异常；判定域限定写进判决 §7.1。
- ✅ **交付（Phase 4）**：构建 1.20.1 **1.0.28** jar + **执行体三元组核验 MATCH**（jar sha / jar 内 `native/worldgen.dll` sha / `target/release/worldgen.dll` sha 三者一致，且与 9 个 coreswap 臂测量同源；vanilla 臂无 dll）；judge 条件 J9（源码注释编号）应用后**强制重编（`--rerun-tasks`，规避沙箱内 gradle VFS/文件监视失效的 `UP-TO-DATE` 假绿）仍逐字节相同**；实机测试点交用户（**观察级**，含 `-Dcoreswap.chunktime=1` 与 `-Dcoreswap.syncfill=1` 回退开关）。
- 🔍 **open（未核/降级/边界）**：① nether/end 全维行为门未做（指纹门只覆盖 overworld 4140/4225 = 98.0%）；② region 门检验力低（每形态 1-2 对、无置信区间；跨形态与 async 锚同阶但高于 sync 锚 +0.0124/+0.0127pp 的**形态相关分量未归因**）；③ 1.20.1 载体 run 级非确定量级本身未立项（sync 0.0174% / async 0.0292%，只登记未查成因）；④ **pre 源码快照缺失**（`runtime/` 被 gitignore，pre 以 1.0.27 jar 的 class 级证据承载）；⑤ **b3「写回/并发路径非确定」未被排除**（指纹取在 `writeChunk` 之前；未做锁/写回并发压力测试）；⑥ 85 个「Chunky 已处理但无接管行」chunk 与 621 格空白成因未查；⑦ 驱动/对拍工具为一次性件（`.tmp` 不入库，关键机制与判据已自足记录）；⑧ **流程**：judge = PASS-with-conditions（J1-J12 已应用），**confirmed 待用户**；⑨ ⚠️ **上游影响**：旧读法（`tag7`=1B）的 260910-05（1.21.6）`common`/差异百分比口径少计约 0.6% chunk ⇒ 已在该判决 §8 增「补充收敛」注记（原文不删不改），量级不变但**分母口径须声明**、**精确复算登记为后续项**。

## 260910-07（实际 2026-09-10 22:40–23:1x，日期锚 Get-Date 22:40：Java mod 工程迁出 `runtime/`（源码入库 + 运行环境原地，两版本 ×2））🔍 candidate（judge 未做 / 用户未 confirmed）

> 过程产物 `.investigations/perf-reg-260910-07/`（`migration-errors-260910-07.md` E1-E4 五段式 + 速查表 / `cmd-output/{equivalence-260910-07.txt,ignore-checks-260910-07.txt}` / `knowledge-draft-260910-07.md`）；架构计划 `.investigations/000-架构设计/架构计划-260910-07.md`（**HOOK-1 用户批准三项**：目标路径 `versions/<ver>/java`、环境原地 `runtime/<ver>/java/run`、入库范围含 worldgen-data + 散件隔离）；判决 `.artifacts/perf-reg-260910-07/verdict-260910-07.md`（**candidate**）；提交 `d5151a1`（2876 files changed）；通用模式 → workflow-patterns #116/#117（+ #28 补充案例）、build-tooling #54/#55/#56（subagent 草稿 → 主会话应用）。

- ✅ **范围与形态（结构重构，非语义改动）**：两个 MC 版本（1.20.1 / 1.21.6）的 loom dev 工程（= **出货 mod 源码本体**：`src/main/java/wg/**` **49 文件 ×2** + 构建定义 + `content-test` + `worldgen-data` 1019/1739 文件）由 `runtime/<ver>/java/` 迁 `versions/<ver>/java/`；**运行环境原地不动**（`runtime/<ver>/java/run/` 世界/mods/`server.properties`，698MB/56MB 零搬迁），由 `build.gradle` 的 `loom { runs { server/client { runDir "../../../runtime/<ver>/java/run" } } }` 指回。
- ✅ **V1 等价性门（决定性，全量无抽样）**：迁移前后构建 jar **逐条目 sha256 全等且 jar 整体 sha 逐字节不变**——1.20.1 **1077/1077**（only-old/only-new/content-diff 全 0）、1.21.6 **1797/1797**；jar 整体 sha 1.20.1 `297680e7…`、1.21.6 `16d5e5e7…` 前后同值 ⇒ **迁移零语义影响**（本稿 subagent 已独立复算 old/new jar sha + zip 条目数 1147/1908 + jar 内 dll sha，逐项一致）。
- ✅ **V3 三元组**：新 jar sha / jar 内 `native/worldgen.dll` sha / `target/release/worldgen{,1216}.dll` sha 两版本均 MATCH（`597e12ed…` / `abd7d889…`，一手复算一致）。
- ✅ **V2 路径回归（1.20.1 一臂）**：新路径 `gradle :runServer`（`-PcppReplace=true` + `-PcppWorldgenDir=…versions/1.20.1/data/worldgen` + `-Pchunktime=1`）跑通——Chunky `Processed: 4225`、`inflight max=23`、无 STALL、`run/` 仍在 `runtime/` 下被读写（`oa-r2` 38 s / wallgen 41.3 / 11.53 核）。
- ✅ **V4 入库双向核（8 条全对）**：源码 / `worldgen-data` / 构建定义 = **未忽略**；`run/`、`build/`、`.gradle/`、`native/`、`runtime/` = **忽略**（一手复跑 `git check-ignore -v` 与 `git ls-files` 计数 0 双重确认；无 `??` 噪声）。
- ✅ **V5 回滚**：反向 `Move-Item` + 还原 `.gitignore`/`runDir` 步骤自足（**记录即算**，未实跑）；旧 `runtime/<ver>/java/build/libs/*.jar` 与旧 `run/` **均原地未动**（历史产物/1.0.27 pre 证据，不得误删）。
- ❌ **E1（响亮失败）loom `runDir` 绝对路径被拼坏**：`CreateProcess error=267 目录名称无效`，错误里工作目录 = `…\versions\1.20.1\java\E:\PYTHON\…` ⇒ 根因 = `runDir` 是 **String 且按工程目录相对解析**（`javap -p` 实证，loom 1.10.5）；修复 = 相对路径 `../../../runtime/<ver>/java/run`；判据 = **读错误里解析后的路径**（→ build-tooling #55）。
- ❌ **E2 驱动脚本「工程目录 + `\run`」推导式失效**：`FAIL no Done` / `找不到 …\versions\1.20.1\java\run\server.properties` ⇒ 迁移后工程与环境分家；**有界扫描（已排除 `.tmp/`）漏掉 `.tmp/` 驱动**（路径推导写在变量拼接里，grep 易漏）⇒ 修复 = 驱动拆 `$run`（工程）/`$rd`（环境）两变量；判据 = **迁移后跑一次真实入口**比纯静态扫描更快暴露（→ workflow-patterns #117）。
- ❌ **E3 沙箱内 gradle `:build UP-TO-DATE` 假绿**：改了 Java 注释仍报 UP-TO-DATE、jar 未变，日志有 `File watcher server … NativeException: Couldn't open current thread, error = 5` ⇒ 文件监视失效使 VFS 陈旧；修复 = 验证性构建 `--rerun-tasks`（本块据此得到「注释级改动后 jar 逐字节相同」这一等价性门的一环；→ build-tooling #56）。
- ❌ **E4 首臂 66 s 离群（家族 37-38 s）——已排除迁移效应**：V1 证 jar 逐字节相同 ⇒ 搬位置不可能改变生成耗时；复跑 `oa-r2` 得 38 s 回到家族值 ⇒ 归因环境（同批并发文件操作 / 冷 gradle daemon / 预热），**非迁移效应**；如实登记为「单点离群先复跑再归因」，并补「有等价性硬证据时归因可直接闭到环境」（→ #28 补充案例）。
- 🔍 **open（未核 / 降级 / 边界）**：① judge **未做**（计划 §6 预置的 candidate-SHOULD / 收尾 FIN-MUST 均未签）、confirmed 未授予；② **证据归档缺口**：判决 §1 声明的证据含 `results.txt` / `logs/`，但二者实际只在 `.tmp/perf-reg-260910-07/`（**临时区，不入库，迟早灭失**）——建议复制进 `.investigations/perf-reg-260910-07/cmd-output/`（证据优先级高于工作区整洁，260910-06 E1 教训）；且 `results.txt` 中 `oa-r1` 同时存在两条 `FAIL_no_done` 与一条完整 66 s 行（**同名臂跨「失败轮/测量轮」复用**），引用需注明轮次；③ **V4 归档证据缺嵌套 prune 那一格**：`ignore-checks-260910-07.txt` 8 条中没有 `worldgen-data/data/**`（而 `.gitignore:109-110` 白名单链正是为它加的，每版本 1015 个 JSON）——本稿已独立复算确认其在库且未被忽略，建议补一行进证据文件；④ 等价性门**只覆盖 jar 产物**——不覆盖 dev-run 参数面 / `content-test` 子工程产物 / 1.21.6 运行回归（V2 仅 1.20.1 一臂），且**不证明 dev-run 行为完全一致**；⑤ `content-test/run`（0.1 MB）随子工程迁移（不影响主流程，已由 `versions/*/java/*/run/` 忽略）；⑥ `runtime/<ver>/java/{build,.gradle,.gradle-home,scripts}` 旧残留**原地保留**、gradle home 仍有多份（`$root\.gradle`、`$root\.gradle-home`、`runtime/...`）= 文件管理债，本块未动（登记后续清理）；⑦ `.investigations/**` 历史归档的路径**按历史读、不改写**（改写 = 篡改证据链），新块文档注明变更点；⑧ 上游影响：**260910-05（1.21.6）旧读法数字复算**仍挂账（260910-06 E5 主题），与本块无关但同批未闭；⑨ 本块跨两版本，时间线**只落 1.20.1**（与 260910-06 同做法）——1.21.6 时间线无 260910-06/07 块，是否加一行指针由主会话定。

---

- ✅ **用户实机观察（观察级，2026-09-10 23:1x 追记于 260910-07 块）**：用户用 `coreswap-1.20.1-1.0.28.jar`（sha `297680e7…`）**在自建实例上跑过三个世界（主世界 / 下界 / 末地），无问题**。⇒ 这是 **1.20.1 nether/end 异步化的首个 vivo 覆盖**（此前 260910-06 §7.1 仅壳内 sanity）。⚠️ **降级声明（#64/#65/#86）**：未受控观察（无坐标/无落盘/无重复/未声明对照基线/未量化/未声明 mod 列表）⇒ **只作方向性旁证，不构成定量结论**；未覆盖性能量化与特定 mod 兼容性。记录见 `.artifacts/perf-reg-260910-06/verdict-260910-06.md` §10。

---

## 260911-04（实际 2026-09-11 18:38–19:10 前后：vivo 卡顿收口——exec 缺省开转正拍板 → perfprofile 移除 + 1.0.29 出单全链 → judge PASS-with-conditions 四条件应用）✅ 用户已 confirmed（「EXEC 模式完美解决问题可以发 release 了」）；工单 pending 待 Maint 发布

> 过程产物 `.investigations/vivo-stutter-260911-02/fps-verdict-260911-04.md`（FPS 判读 + 拍板记录）；`.artifacts/releases/judge-verdict-260911-04.md`（judge 原文归档，N-2 落盘件）+ `RELEASE-1.0.29.md`（pending）+ `INDEX.md`（已登记 pending 行）；提交 `cf9fa54`。通用模式 → workflow-patterns #126（mtime 严格序倒挂，本块新判据）+ build-tooling #57（池宽语义换算）。

- ✅ **exec 转正拍板（重大方向，用户实机确认）**：实机验证「EXEC 模式完美解决问题」→ exec 模式缺省开转正（回退 = `-Dcoreswap.exec=0` 走 P1 信号量路径）；**池宽维持「物理核 − 2」用户语义不再测试**——现实现缺省 `logical/2 − 2` 与其 SMT2 下同源等价（引擎 `adaptive_threads` 同口径），池宽**零代码改动**。maxinflight 缺省值决策随转正自然关闭（仅回退路径使用，维持同源缺省）。
- ✅ **perfprofile 移除 + 1.0.29 出单全链**：`cf9fa54`（perfprofile 临时件全树移除 + version bump 1.0.29；exec 实现链 `0ecac5c`→`d20aac1`，Rust 零改动）→ 重编 final jar（18:55，`--rerun-tasks` 全量）→ **三元组 MATCH**：jar sha256 `b057fda216012647a0e0ea6bbe0d1b4967a5458950ddc5f208dc85f48c312239`；jar 内 `native/worldgen.dll` = `dd3b645f2c79d2cb54619e0ecb95f9b913b3fe02f30d74eba9ea5ee92886765d` = `target/release/worldgen.dll`（12:53 刷，早于 Java 改动，与 Rust 零改动一致）。judge 亲算复核一致。判读 + 拍板建议 candidate；AI 侧不授 confirmed 标签（用户实机确认即拍板，AI 仅记录）。
- ✅ **judge（MUST：重大方向 + 工单出单）**：节点① exec 转正 + 池宽维持 = PASS；节点② 出单 = **PASS-with-conditions**，四条件全部已应用——N-1 INDEX.md 登记 1.0.29 pending 行；N-2 judge 意见落盘至 `.artifacts/releases/judge-verdict-260911-04.md`；**N-3 mtime 严格序措辞**（jar mtime 18:55:17 早于 commit 时间戳 18:55:41 达 24 秒——「重编 > 删除 commit」严格不成立，工单改为「同分钟、commit 落盘于构建后」，sha 三元组以内容指纹为准 → 判据沉淀 workflow-patterns #126）；N-4 §4 补「关联 bug 卡回填：不适用」声明。其余核对：HEAD=`cf9fa54`、perfprofile 零残留、C1-C4 全核销（C3 三项落工单 §5 已知边界）、撤单 1.0.28 关系清楚。
- 🔍 **open（遗留）**：① **Maint 发布**——RELEASE-1.0.29.md pending，告知用户转交 Maint 会话（Maint 侧仍有第二重独立 hash 校对 + 逐次人工确认门）；② C1 补 run（wall 差定量）——用户实机已接受效果，wall 定量裁决不再需要；仅当后续需要性能定量口径时补单 run；③ 1.0.29 状态回写待 Maint 在其工作区 `status/` 落镜像后闭环。
- ✅ **Maint 发布回执（2026-09-11 19:14，260911-04 块追记）**：1.0.29 published——tag `coreswap-1.20.1-1.0.29`（按 1.0.26/1.0.27 惯例重定向至 master HEAD `1e7cda7`，PATCH refs + force）、release URL `https://github.com/unknowbug/CoreSwap/releases/tag/coreswap-1.20.1-1.0.29`、asset `coreswap-1.20.1-1.0.29.jar`；Maint 侧第二重独立 hash 校验三 MATCH（jar b057fda2…/inner dll dd3b645f…/target dll）。状态镜像 `E:\PYTHON\CoreSwap-Maint\.artifacts\releases\status\RELEASE-1.0.29.json`（主工作区 INDEX 已回写 published；镜像缺失误报一并澄清——镜像一直存在 Maint 工作区）。工单建议的 Release 标题未按仓库惯例命名由 Maint 纠正（规则已入 Maint 侧 AGENTS §5）。1.0.29 生命周期闭环。

---

## 260911-06（C 线 bulk section 写回；目录标签沿用 -05，实际 2026-09-11 20:56–22:3x）✅ confirmed（**用户授予 2026-09-11 22:58**；judge 已产出 `review-260911-05.md`：M1/M2/S8/S10 已响应）

> 过程产物 `.investigations/bulk-writeback-260911-05/`（`record-260911-05.md` §1-§9 含 §3.4 E4 五段式 + §5.1-§5.5 判决实验 / `plan-260911-05.md` §0 修订 R1-R4 / `api-probe-260911-05.md` / `scout-map.md` / `verify-design-draft-260911-05.md`（预注册判据 `:29-33` = `diff = 0`）/ `errors-260911-05.md`（E1/E2 五段式 + 速查表）/ `review-260911-05.md`（独立 judge，205 行）/ `knowledge-revision-260911-05-K1.md`（本块知识库修订稿）/ `evidence/`（六臂原始日志 + `wbc-*.txt` 抽取 + `bulk-summaries.txt` + `tier3-diff.txt` / `tier3-slice.txt` + 判据脚本 + `MANIFEST-sha256.txt` 40 行 + `impl-diff-260911-05.txt`））；判决 `.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`（candidate）+ `.artifacts/index.yaml`（`swe:bulk-writeback-260911-05:verdict`）；提交 `8dd9e71`（Java 实现；**Rust 零改动**，dll `838e89794a54e19d`）；通用模式 → algorithm-fingerprints #22/#23、compiler-idioms #24 补充案例、workflow-patterns #129（修订后）/ #25 补充案例 / #110 补充案例 / #14 补充案例（subagent 草稿 → 主会话应用）。

- ✅ **D1 落地（Java 侧 bulk section 写回，Rust 零改动）**：路线 = 公开 `readPacket(PacketByteBuf)`（原案 5 参构造器因第 3 参 `PalettedContainer.DataProvider` 是包私有 record 而不可调用）+ `ChunkSectionAccessor` mixin（4 写 + 3 读，`@Mutable`）**原地换容器 + 直写三计数**（不新建 section、不调 `calculateCounts`、生物群系容器原样保留）；消掉每 chunk 98,304 次 `setBlockState`；回退开关 `-Dcoreswap.bulkwb=0`（旧路径保留不删）。
- ✅ **等价性（决定性，噪声无关）**：Tier 1 写回后读回 FNV-1a 64 + Tier 2 三计数序列指纹，**607/625/625 逐 chunk 差均为 0**、两臂 chunk 集相同、零异常（覆盖 59,670,528 位置）；编码四支 SINGULAR/ARRAY/BI_MAP/**ID_LIST** 合成自检逐位读回全绿（自然生成 `max_distinct=7` ⇒ ID_LIST 生产不可达，刻意压测）。本稿**独立复算**自归档 `wbc-*.txt`（607/625/625 行、`only_old`/`only_bulk` = 0、hash|dh 值差 0）复核一致。
- ✅ **性能（同 run 同仪器单变量）**：写回分项 overworld 2,151,591→**652,202** ns/chunk（−69.7%）、nether 2,657,477→**859,626**（−67.7%）、end 2,809,444→**804,970**（−71.4%）；`counts_set` 由 `calculateCounts` 的 72,480 ns/section → 116 ns/section。**价值定位下调**：可维护性/可移植性为主，性能次要附带（写回占 chunk ~2-6% ⇒ 端到端在 ±10% 噪声带内**不主张**）。
- ❌ **E1（首跑 539 次 IAE）**：`bits==0` 单态 section 不得构造 `PackedIntegerArray`（`Validate 1..32`）——位宽 0 = **无 storage**；判据 = 复刻 MC switch 逐 case 过构造器前置条件；定位 = **先看两臂集合差（607 vs 68）再看异常原文**。
- ❌ **E2（最高价值：性能问题的解法暴露正确性问题）**：`calculateCounts()` 与增量 `setBlockState` 的计数语义**不一致**（流体重复计入 `nonEmptyBlockCount`；`nonEmptyFluidCount` 只计有随机刻的流体）⇒ 走构造器会静默改变 `hasRandomFluidTicks()` / 客户端包语义；生成期语义由**调用点**决定（vanilla 走增量）⇒ 改为复刻增量语义。三计数**不入存档**（`ChunkSerializer:310-311`）⇒ region 对拍看不到，唯一可见面 = 生成期内存态。
- ❌ **E3/E4（过程失败，judge S10 要求补记）**：E3 = 把 Chunky 的 `Processed: 441` 当成「写回 chunk 数」误推「重复接管存在」（实为 21×21；实测 607 行坐标全不重复）；E4 = ① **跑完 22:06–22:11 的同实现对照 run 却不回写结论**（输出只落在 gitignored 的 `.tmp/` ⇒ 等于不存在）② **把「判据未满足」写成「不可判」**（用降级措辞掩盖 FAIL）。五段式见 `record-260911-05.md` §3.3/§3.4。
- 🔍 **Tier 3 修订链（三拍；第一拍原文保留不改）**：
  - **❌ 21:5x 定稿（原记录，保留不改）**：原「region 逐块差 = 0」判据**降级为「不可判」（noise-floor-limited）**——理由是「全域 0.0277%，**从未走过本代码路径**的 2,583 chunk 也有 0.0260% ⇒ 残差为跨 run 固有抖动（FEATURES 产物 + 随机刻可改写方块的双向对称变化）」；口径外溢登记（含历史 live-server region 对拍保真度数字）拟写入其他载体的限定说明。
  - **❌ 22:0x judge 复算（`review-260911-05.md` M1/M2/S8/S10，独立复算 4 个已归档世界，不采信转述）**：**M1** = 交付对是 4 对中差分最高的一对、且超出集中在切片 A（同实现底 0.0239–0.0284% vs 交付对 0.0348%）⇒ 与 record / verdict / `index.yaml` 三处「残差非写回引起 / 不可判」措辞**直接矛盾**；且预注册判据 `diff = 0` **未满足**这一事实**未被登记**；**M2** = 「切片 A 低于切片 B 有内容子群 ⇒ 写回落在差分更低的一半」是**归因谬误**（该落差在同实现对里同样成立）；**S8** = 用本块噪声底去限定其他载体绝对值**越界**（载体不同、对方自身对照更低，且该限定从未落地）；**S10** = 「跑完对照 run 却未回写结论」「判据未满足的处置链」须按错误优先原则补记。
  - **🔍 22:2x 补跑判决实验 + 改判（本块定稿）**：补跑 `bulk × bulk`（2 run ⇒ 3 对）+ 关随机刻/天气/刷怪/火焰 run + 严格逐块归属；结果 = 交付对 0.0348% **落在 bulk 臂自身跨 run 区间 0.0272–0.0358% 之内** ⇒ **无「写回引入系统性存档层差异」的证据**；**改判 = Tier 3 判据未满足（FAIL）+ 残差未归因**。弱信号如实登记：含 bulk 的 5 对全域 0.0258–0.0284% vs 两 old 的 3 对 0.0238–0.0253%（两组不重叠）⇒ 最自然读法 = **bulk 臂自身跨 run 离散度更高（方差效应而非均值效应）**，n = 3/组、切片 A 区间仍重叠 ⇒ **仅提示性、未达显著**；机制候选并列 **A** 运行期调度（bulk 每 chunk 快 ~1.5 ms ⇒ 改变并行生成流水线中「邻块写特征」先后）/ **B** 容器编码差异（**不成立方向**：Tier 1/2 已证写回后状态逐位全等）/ **C** 开放。噪声底主体 = **vanilla FEATURES 阶段**（关掉全部随机源后仍 0.0259%，top pairs 仍是特征产物 ⇒ 随机刻只占 6–11%）。分母口径 = 313,589,760 块中 **58.1%（1854 chunk）从未生成内容**（纯稀释分母，两侧皆空差分**恰好 0**），100% 差分落在有内容的 **1336** chunk = **0.0661%**。**撤回**原「向其他载体写入限定说明」的动作 ⇒ 收窄为「region 层百分比受跨 run 抖动 + 分母含未生成 chunk 两项影响，**跨载体不可互引**；任何载体数值 MUST 由其**自身同形态对照**界定」。
- ✅ **R8/R9 论证**：R9 = 新路径不比老路径弱且在锁粒度/原子性上更强（`readPacket` 自带一对锁，每被替换 section 一对 ≈8.0/调用（上限 24/chunk）vs 老路径每非空气块一对；填私有容器后单次引用发布 ⇒ 无部分写可见窗口）；残余 = 引用非 `volatile`，未做并发压力专项（R9-b）。R8 = `BelowZeroRetrogen` 仅由旧存档 NBT 携带，本 harness 每臂删 world ⇒ flag 恒不置，且 C 不动生物群系/FEATURES ⇒ 无新增暴露面。
- 🔍 **open（未核/降级/边界）**：① R9-b 并发可见性专项未做；② ID_LIST 仅合成覆盖（生产不可达）；③ `MAX_ID=4096` 沿用老路径同款约束；④ **sync 形态未跑**（本块只 async）；⑤ 端到端 wall 不主张（噪声带内）；⑥ 共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）；⑦ **Tier 1/2 判据脚本的运行输出未落盘**（`cmp_wb.py` 与原始日志均在库，可复跑；本稿已用归档抽取件独立复算 PASS，建议补存一次输出）；⑧ **流程**：judge **已产出**（`review-260911-05.md`，205 行；M1/M2/S8/S10 已响应、知识库修订稿 K1 已产出），**confirmed 未授予**；⑨ **块号**：与同日 E5 复算块（19:37-20:52）同用 `260911-05` 标签，本行按 `-06` 记、目录名不改（见 `knowledge-update-draft-260911-05.md` §3）。

> **相对旧稿的唯一改动**：提交号归属按 `record-260912-01.md` §4.5 纠正——**A1b 实现与 A1a 同在 `b2b2f26`**（`6b90998` 实为 docs 提交 = plan + record）；完整链 = `b53b23f`(A1d) / `b2b2f26`(A1a+A1b) / `6b90998`(docs) / `8d8075a`(A2) / `e8decef`(judge C1-C12)。旧稿 §5-1「`6b90998` 零命中 ⇒ 存疑」**据此解除**。

## 260911-05（A 线：1.20.1 优化池 + nether/end 维度门；实际 2026-09-11 19:2x–20:5x）🔍 candidate（judge PASS-with-conditions，C1-C12 + I1-I4 已应用；confirmed 留人类）（追记于 260911-06 块之后；目录标签与同日 B1/C 块同用 -05）

> 过程产物 `.investigations/a1-opt-pool-260911-05/`（`record-260911-05.md`：A1a/A1b/A1c/A1d + 行为门 + §9.7 降级声明 / `a2-dim-gate-260911-05.md`：nether/end 全维行为门 / `review-260911-05.md`：judge 原文 + §7 C1-C12 处置表）；数据载体 `.tmp/a1-260911-05/`、`.tmp/a2-260911-05/`（驱动 + 比对脚本，临时区不入库）+ 日志 `.tmp/vivo-stutter-260911-02/ab-threads/logs/`、`.tmp/a2-260911-05/logs/`；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`，均 candidate）；提交 A1d `b53b23f` / **A1a+A1b `b2b2f26`（同提交）** / docs `6b90998` / A2 `8d8075a` / judge 修正 `e8decef`。
> ⚠️ **与 C 线的关系（勿写成「已消失」）**：C 线（260911-06）把默认写回改走 bulk section 路径后，A1a/A1b 所改的**旧逐块路径仍在库**——回退开关 `-Dcoreswap.bulkwb=0` 走旧路径，A1a 的跳空气写回与 A1b 的诊断门控即位于该路径（见 07 篇 C 线小节「回退开关 `-Dcoreswap.bulkwb=0`（旧逐块路径保留不删…）」）。

- ✅ **A1a 写回跳空气（`CppBridge.writeChunk` 对 raw id 0 跳过；回退 `-Dcoreswap.skipair=0`）**：**改了什么** = `writeChunk` 遇 raw id 0 不再逐格写回；开关为静态 final ⇒ 常量折叠（生产路径实际新增每格一次 `id==0` 判断，judge I4 修正措辞）。**判据（预登记）** = 跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；判据谓词经 judge C5 严格化为 **`!isOf(Blocks.AIR)`**——`isAir()` 会漏计持 `cave_air`(730)/`void_air`(729) 的格子，而跳过的写是「写 `minecraft:air` 默认态」。**vanilla 一手源依据（judge 补引）** = `ChunkSection.setBlockState`（1.20.1）只在 `isAir` 变化时增减 `nonEmptyBlockCount`、`PalettedContainer.swap` 同值时写回同一 palette index ⇒ 对空气格写 air 是语义 no-op。**实测（最终构建 + 严格谓词，三维全格计数）** = overworld **276,171,456** / nether **194,630,604** / end **154,752,709** 个空气格被跳过，`stale_nonair` **三维均 0**（judge C4 由「机制外推」补跑为三维实证：wb-ow/wb-n/wb-e）；机制依据 = 三维 mixin 均 `populateNoise` HEAD cancel（`NoiseChunkGeneratorMixin` 279/333/364）⇒ 目标 chunk 全新。**行为门** = 跨臂 + 跨 dll + 跨 session 归档臂逐 chunk 指纹全等（4140/4140，`hash_diff=0`）。**证据路径** = `record-260911-05.md` §2-§4 + 日志 `[WB-CHECK]` 行。**commit `b2b2f26`**。
- ✅ **A1a 收益不主张（Degraded）**：cpuSec **493→476（−3.4%）**——**落在 ±10% 机器噪声带内、单对跨 run ⇒ 只作趋势**；且两臂均开 `WBCHECK`（每空气格一次 `getBlockState` = **2.76 亿次**）⇒ 该数应读作**含诊断成本的收益下界**，要引用定量数字须关 WBCHECK 重跑（judge C6）。结论性质 = 「等价性成立 + 无回归」，非「性能提升 N%」。
- ✅ **A1b 诊断扫描门控（`CppBridge.fillChunk`）**：**改了什么** = `nzBuf` 全 buffer 扫描（**98,304 读/chunk**）与 nether/end 16 点读回**整块门控到 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。**判据** = 纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。**取舍声明（judge C2）** = 改前 `buf-all-air`/`buf-sparse` 是**无条件 println**（原记录「只门控 println」的理由句错误，已改正；类 javadoc 旧措辞作废，C10），`buf-sparse` 分级诊断随全量扫描一并移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。**证据路径** = `record-260911-05.md` §3 A1b + `CppBridge.java` javadoc。**commit `b2b2f26`（与 A1a 同提交）**。
- ❌ **A1c 6 高度图「全量重扫」——前提被一手源证伪，用户裁决放弃（已评估·不实施）**：一手源 `Heightmap.java:37-71` = vanilla `populateHeightmaps` **本就是单遍**（每列 (x,z) 只做**一次**下行扫描，6 个类型的谓词在**同一次扫描内**逐个匹配并移除，非「6 张各扫一遍」）⇒ 优化池该项描述**不成立**；残余候选只剩「读 `buf`（raw id）替代 `chunk.getBlockState`」，而代价/风险 > 收益（该项 0.42ms/chunk = 0.4%；`Heightmap.set` private、storage final ⇒ 外部写入须反射或自实现位打包）。**裁决** = 用户 HOOK-2 放弃实施。**证据路径** = `record-260911-05.md` §3 A1c + `review-260911-05.md` §3（judge 对照 `Heightmap.java` 核实）。
- ✅ **A1d `adaptive_threads` count=1 clamp（Rust 共享层 `worldgen-core/src/api.rs`，清理项）**：**改了什么** = `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`（批量路径 count>1 语义**逐字未变**）。**机制自证（运行期）** = `[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll，两条：a1-skipair0/1 的 `.log.err`）。**MT3 的历史前提已消失**——当时是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 per-call `std::thread::scope`（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ clamp 安全（judge 独立核对）。**无性能归因**（双臂都含 clamp，未做隔离 A/B）。**证据路径** = `record-260911-05.md` §3 A1d + `review-260911-05.md` §3。**commit `b53b23f`**。
- ✅ **A2 nether/end 全维行为门（开工发现真缺口 → 补载体）**：**改了什么** = `CppBridge.fillChunkEnd` 的 MIXLOG 块内补 `[WG-CONTENT]` 指纹行——首轮 end 臂 `intercepted=4761` 而 `contentLines=0` ⇒ 全维行为门在 end 维**载体缺失**（判据意义：**「接管生效」与「行为门可判」是两件事**）。**判据** = 三维「跨形态 + 跨 run 逐 chunk 指纹 sorted diff = 0」。**实测** = **overworld 4140/4140、nether 4761/4761、end 4761/4761 逐 chunk 指纹差 0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；**正对照** overworld vs nether `hash_diff=4140/4140`（门有检测力）；**分母语义** = 载体条数 = **经接管的 chunk 数（≠ Chunky Processed 4225）**，门判据不依赖任何分母（judge C3）。**证据路径** = `a2-dim-gate-260911-05.md` §1-§2 + 日志 `[WG-CONTENT]` 行。**commit `8d8075a`**（judge C9 改正早前误引的不存在提交号 `3c1f9c6`）。
- 🔍 **A2 附带读数（非门判据，Degraded）**：nether async **25s/24s** vs sync **69s**（形态效应 ~2.8×，cpuSec 212/202 vs 149）；end async **11s/8s** vs sync **20s**（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致（单车道同步形态在 nether/end 都是净亏损）；但跨 run 摆动 **±27%** ⇒ **不宣布定量收益**。
- ✅ **judge（candidate 授予 SHOULD 触发，隔离子进程）**：A1 / A2 均 **PASS-with-conditions**、无 FAIL，**全部量化数字复现一致**；C1-C12 + I1-I4 **逐条已应用**（处置表 = `review-260911-05.md` §7）——重点：C1 可比性改为「a1 双臂同 dll `838e8979`；三归档臂为 A1d 前构建 `dd3b645f`」并补「跨 dll 指纹全等 = A1d 输出中立性独立证据」、C3 分母语义声明、C4 由外推补跑为**三维实证**、C5 判据谓词严格化、C7 比对脚本加最小载体阈值断言（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）、C8 下游归因降为**未测外推**、C9 提交号改正、C12「85 缺口」说法作废。judge **未改任何 status**；**本块结论维持 candidate，confirmed 留人类**（judge 修正 commit `e8decef`）。judge 盲区如实登记：旧二进制 clamp 前 `nthreads` 只能推导（无日志行）、`dd3b645f` 是否确为 A1d 前构建按时间线推断、f2 成本拆分（0.42ms/chunk）本块未复核。
- 🔍 **open（未核/降级/边界）**：① **写回后内容指纹（post-write hash）仍未做**——三维均未覆盖（260910-06 open ⑤ 延续；A1a 以「前提全格实证」替代）；② overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」说法已废，降为 open 假设）；③ nether run 级非确定的**成因域**只排除「Rust 填充层」（本载体只覆盖 `buf`），下游层未测（judge C8 未测候选：Java carver/feature/装饰层、写回路径、存档序列化）；④ A1d「改前 nthreads=10」为公式推导非实测；⑤ `buf-sparse` 分级诊断已移除；⑥ 本项改动**尚未随任何 release 出货**（1.0.29 不含 A1 改动，出单评估见计划 §2 A4）；⑦ dll 硬门禁只比日志里 **16 hex 前缀**（非全 sha256，judge I2）；⑧ **块号**：与同日 B1（E5 复算）/C 线 bulk 块同用 `260911-05` 标签，本块按 `-05` 记、目录名不改。

---

## 260912-01（D3 共享 Java 适配核抽取：Wave 1 纯移动 + Wave 2 语义统一；实际 2026-09-12 14:0x–，日期锚 Get-Date）✅ confirmed（用户授予 2026-09-12 15:52；范围 = 出货线 1.20.1 生产行为不变 + 共享核结构抽取 + 1.21.6 默认路径冒烟；**不含** 1.21.6 强制 bulk 缺陷与内容等价性——仍开放、阻断 D-4(i)；Wave 1 V1-strict PASS；Wave 2 V1b = PASS-with-declarations（判定基线 = `post2`，三态表见 §4.6）；**V2/V3：1.20.1 双 PASS + 1.21.6 默认臂 PASS（§4.7.3 已回填）**；judge = **PASS-with-conditions**（`review-wave2-260912-01.md`；C1–C9 已全部响应/修正，见 record §4.7.7）/ record §6 已回填 / **用户已 confirmed 2026-09-12 15:52**（范围见本块标题）；**新发现确认缺陷：1.21.6 强制 `-Dcoreswap.bulkwb=1` 臂崩解（130 chunk `EntryMissingException: Missing Palette entry for index 2…8`，根因未定位）⇒ 不阻断 Wave 2（默认关）、但阻断 D-4(i)**）

> 过程产物 `.investigations/shared-java-core-260912-01/`（`record-260912-01.md`：§1 开工前交接核验 / §2 pre 冻结 + 构建确定性控制 + V0 接线预检 / §4.1 Wave 1 / §4.2 HOOK-2 用户裁决 / §4.3 scout-judge 条件响应 / §4.6 V1b 判定 + **构建三态表** / §4.7.0-§4.7.8（V2+V3 / **dll 血统事故** / 1.20.1 双 PASS / **1.21.6 回填 + 确认缺陷** / dll 归一化 / **证据落盘** / **两个过程发现的一手锚** / judge 条件响应 / 未闭合项）/ `scout-map.md` / `review-scout-260912-01.md`（scout-judge PASS-with-conditions：抽核 21 锚点 / 14 文件，✘0 / ⚠1，闭包 4 路证伪未遂）/ **`review-wave2-260912-01.md`**（本波 judge，verdict = PASS-with-conditions，条件 C1–C9 见 record §4.7.7）/ **`errors-260912-01.md`**（错误台账，五段式 E1 dll 血统 / E2 worktree CRLF / E3 javap 三陷阱 / E4 门数字未复算 / E5 1.21.6 强制 bulk 崩解 + 速查表）/ **`evidence/`**（**42 文件 + `MANIFEST.txt`**，含清单 / 差异输出 / 门控原样行 / 完整日志 / 复现工具；tracked、未被 gitignore））+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（§14 追加式补登，原正文不改）；工具 `.investigations/shared-java-core-260912-01/evidence/{tool-jar_manifest.py,tool-jar_manifest_diff.py,tool-fp_compare.py,tool-javap_method_diff.py}`；提交 `ce5286b`（Wave 1 纯移动）/ `997d40f`（Wave 2 语义统一）/ 后续 record+evidence+drafts 落盘 commit（judge C9）。通用模式 → knowledge/discovered：workflow-patterns **#138** + **#14 补充案例（260912-01）**、build-tooling **#59（与 A 线首例合并）/ #60 / #61**、compiler-idioms **#25**（subagent 草稿 → 主会话应用）。

- ✅ **Wave 0/1（机制接线 + 纯移动，V1-strict PASS）**：新建共享源目录 `java-core/src/main/java`（单一源），两版 `build.gradle` 各加 `sourceSets { main { java { srcDir '../../../java-core/src/main/java' } } }`（各 +11 行）。**V0 只接线（空 srcDir，仅 `.gitkeep` + README）**⇒ 两版 jar sha **逐字节不变**（`1027f4f6…` / `16d5e5e7…`；条目 1081/1081、1798/1798，差异 0、增删 0）⇒ **接线机制对 loom / mixin AP 惰性**（未证明 = 非空共享源时的行为，那才是 Phase 2 判定对象）。**Wave 1** 把 3 个**逐字节相同**的类（`wg/CppWorldgen.java`、`wg/bench/WgDiag.java`、`wg/bench/BenchMod.java`，两版源 sha 实测相同）`git mv` 进共享源、`git rm` 1.21.6 副本（「一个类只有一个家」）⇒ 两版 jar sha **仍逐字节不变** ⇒ **非空共享源下「移动 ⇒ 产物字节相同」成立**（提交 `ce5286b`）。
- ✅ **构建确定性控制（V1 判据有效性的前提）**：1.20.1 同源连跑两次（`--rerun-tasks`）jar sha 完全相同；1.21.6 现建 jar 与知识库 260910-07 迁移记录（KB #116）里的 `16d5e5e7…` **逐字节相同** ⇒ ① 构建确定（跨 2 天、跨调用）② 顺带证明 1.21.6 源码自 260910-07 起未变。⇒ **「抽取后条目 sha 变化」只能来自抽取本身，不来自构建抖动**。
- ✅ **V0 接线预检的价值边界（写死）**：证明了 ① 新增空 srcDir 不扰动 refmap / 条目集 / jar 字节（两版构建仍绿 `exit=0`）② V1 工具链在本载体端到端可用；**未证明**非空共享源的行为（单变量对照 = 仅添加空 srcDir）。
- ✅ **Wave 2（语义统一）交付面**：共享 `CppBridge`（超集合并：并入 1.21.6 独有项 + `WgCompat` 引用）、共享 `BulkWb`/`StallWatch`/`ChunkTiming`（超集）/`CoreSwapFixHelper`；分版缝类 `WgCompat` ×2（`BULKWB_ON`/`SKIPAIR_ON`：1.20.1 = true、1.21.6 = **false**；`WgCompat.flag(prop, def)`：设了 property 则「非 0 即真」⇒ 1.21.6 将来可 `-Dcoreswap.bulkwb=1` 强制开启而无需改码，翻转 = 改一个常量）；1.21.6 补 `mixin/ChunkSectionAccessor` + `coreswap.mixins.json` 24→25 条 + refmap 重建（HOOK-2 批准的 **S-1**；字段名两版同名 + intermediary 名两版逐字相同 ⇒ 静态可行，默认不被调用 ⇒ 无行为变更）。**HOOK-2 其余裁决**：S-2/S-3 默认常量（上）、S-5 StallWatch 接线进共享 `init`（property 门控、两版默认关）、S-6 **接受** 1.21.6 生产热路径去掉每 chunk 无条件全量扫描 + println（已批准的行为变更，由 V2/V3 门覆盖）、S-7 **接受** ChunkTiming 取超集（1.20.1 开 `[CHUNKTIME]` 时多打印恒 0 列）。
- **冻结 pre（同一主工作树构建，§2.1）= 1.20.1 `1027f4f6…`（1081 条目 / 57 类）、1.21.6 `16d5e5e7…`（1798 / 54）**。**Wave 2 构建三态（§4.6 表头，judge C6 修正：V1b 判定基线 = `post2`，不是 post1）**：

  | 态 | 构建内容 | 1.20.1 jar sha | 1.21.6 jar sha | 相对上一态的隔离差异（实测） |
  |---|---|---|---|---|
  | post1 | worker 首建（**含头注释**） | `41f4a551…` | `2be87e40…` | — |
  | **post2（V1b 判定基线）** | 删 `StallWatch.java`(1.20.1)/`CoreSwapFixHelper.java`(1.21.6) 头注释后重编 | **`0681ec03…`**（1082 / 58） | **`772d7a6e…`**（1803 / 59） | 恰 2 条：`CoreSwapFixHelper`、`StallWatch`（= 头注释致 `LineNumberTable` 位移，`post1-vs-post2`） |
  | post3（权威 dll） | `target/release/worldgen.*` 归一后重编 | `461baedc…` | `772d7a6e…`（同 post2） | 恰 1 条：`native/worldgen.dll` |
  | post4（C4 注释修补后） | `ChunkTiming.java` javadoc **逐行替换（总行数不变）** | `461baedc…`（≡ post3） | `772d7a6e…` | **0 条**（`evidence/post3-vs-post4-*.txt`：相同 1082、差异 0 / 非预期 0） |
- ⇒ **1.20.1 对冻结基线的 V1b 差异集 = 6 个变动 Java 条目 + 1 个新增 Java 条目**（`evidence/v1b-1.20.1.txt`：相同 1075 / 差异 6 / 新增 1 / 删除 0）；1.21.6 = 5 差异 + 5 新增（`v1b-1.21.6.txt`：相同 1793 / 差异 5 / 新增 5）；**dll 条目只在 post3 比对时出现 1 条**（构建输入、非 Java 面，§4.7.4）。两版 `gradle :build --offline --rerun-tasks` 均 **exit 0**（无 `error:`、无 `Cannot find target method` —— KB #25/#40 前兆判据；mixin AP 接受 1.21.6 的 `@Mutable` accessor = **由产物证实**（`javap -v -p` 显示 `ChunkSectionAccessor` 带 accessor ×7 + `@Mutable`），judge 指出 record 原引用的 build 日志中 `Mutable` 命中 0 ⇒ 证据指针不准、非结论错）。
- **V1b 判定 = PASS-with-declarations**（差异条目 **100% 逐条声明**；4 条经 `javap -c -p` 证明「指令完全相同、仅调试属性」；1 条为逐值等价的常量替换；其余为已批准超集/新增）。**V6 共享类布局检查** = `class_home_check.py` 交叠类**空** + 共享源内 mixin 包类**空** + 共享类 8 个 ⇒ PASS。
- **V1b 逐条声明（1.20.1：6 差异 + 1 新增）**：① `wg/bench/BulkWb.class` = **仅 `<clinit>` 的 `ON` 求值序列**（`!"0".equals(getProperty("coreswap.bulkwb"))` → `WgCompat.flag(prop, BULKWB_ON)`）；javap 逐指令对拍除该处外逐条相同（仅偏移位移），语义逐值等价（null→true、`0`→false、`1`/其它→true）；② `wg/bench/BulkWb$TL.class` = 仅调试属性（LineNumberTable；`javap -c -p` 输出**完全相同**）；③ `wg/bench/ChunkTiming.class` = 取超集（+7 `LongAdder`、+7 `addXxx`、+`featTick`；report 取 1.21.6 形态；计划 §4.2 已批准 **S-7**；javap 指令差异 697 行）；④ `wg/bench/CoreSwapFixHelper.class` = 仅调试属性（注释 3→6 行 ⇒ 行号位移；`javap -c -p` 输出**完全相同**）；⑤ `wg/bench/CppBridge.class` = 超集合并（并入 1.21.6 独有项 + `WgCompat` 引用）；⑥ `wg/bench/CppBridge$1.class` = 仅调试属性（匿名类行号随合并位移；`javap -c -p` 输出**完全相同**）；＋`wg/bench/WgCompat.class` = 新增（分版缝类，计划 §4.2）；另 `wg/bench/StallWatch.class` = **无变化（字节全等）**——逐字复制后条目 sha 全等；**曾因加 1 行头注释导致行号位移 ⇒ 删注释恢复字节锚**（「头注释纪律」的来源，→ KB compiler-idioms #25）。
- **方法级对拍（1.20.1 `CppBridge`，偏移 / 常量池序号 / `ldc_w` 宽度归一）**：32→33 方法，`same=28 / changed=4 / added=1`——`stateById`（内容路径）**源码逐字相同**（pre `:629-638` ≡ post `:725-734`）⇒ 该路径未变，字节差异仅常量池序号/顺序（归一后仍报 6 行 = **序号伪差**，已用源码逐字对照排除）；`writeChunk`（内容路径分派）唯一源码差异 = ① `BulkWb.ON` → `BULKWB`（1.20.1 缺省取值相同）② `populateHeightmaps` 前后 2 行计时钩子；`writeChunkPerBlock`（逐格写回本体）**不在差异集内** ⇒ 指令级不变（A1a 跳空气/WBCHECK 自检原样）；`fillChunk` 差异 = 计时钩子 + A1b 门控（1.20.1 侧 A1b 形态与 pre 相同）；＋`rustFeaturesTakeover` 新增（1.21.6 独有 mixin 的编译前置，1.20.1 无消费者 ⇒ 零行为影响）。⚠️ **`lambda$static$0` 不构成证据**：javap 的 lambda 名按**序号**命名，pre/post 的 `lambda$static$0` 可能对应不同 lambda（诚实声明；BUF/BUF_NETHER/BUF_END 声明已源码对照一致：`16*16*384`/`256`/`128`）。⚠️ **初版对拍曾报 4 处伪「语义变更」**（`stateById` = 「30/30 指令、6 行不同」、`lambda$static$0` = 「98→97、46 行不同」）——源码逐字对照（`evidence/src-pre-1.20.1-CppBridge.java` ↔ 现行 `java-core/.../CppBridge.java`）证明完全相同；归因 = **javap 三陷阱**（lambda 按序号命名 / 按行 zip 在指令数变化处级联误报 / 常量池序号与 `ldc`↔`ldc_w` 宽度未归一），一手锚 = record **§4.7.6** + `errors-260912-01.md` **E3**；judge 独立复算改用「`javap -c -p -constants` 文本逐字 + `-v -p` 属性分类」后确认 **4/4 条「仅调试属性」**成立。
- **V1b 逐条声明（1.21.6：5 差异 + 5 新增）**：① `coreswap.mixins.json` 24→25 条（+`ChunkSectionAccessor`；`ConvertFrom-Json` 复验合法）；② `coreswap1216-refmap.json` +`ChunkSectionAccessor` 段（mixin AP 重建，S-1）；③ `wg/bench/ChunkTiming.class` = **仅调试属性**（javadoc 行数变化；`javap -c -p` 输出**完全相同** ⇒ 并集未改 1.21.6 计时代码）；④ `wg/bench/CppBridge$1.class` = 仅调试属性（输出完全相同）；⑤ `wg/bench/CppBridge.class` = 超集合并（并入 A1a/A1b/A2/StallWatch 接线 + bulk 分派结构；含已批准 S-5/S-6 行为变更）——方法级：`writeChunk` 133→40（改为分派）+ `writeChunkPerBlock` 新增（取 1.20.1 版 128 指令）、`destroy` 14→43、`init` +1；＋`BulkWb`/`BulkWb$TL`/`StallWatch`/`WgCompat`/`mixin/ChunkSectionAccessor`（5 条）。
- ⚠️ **残留影响声明（诚实清单，3 条）**：① **1.20.1 生产路径新增 4 次 `System.nanoTime()` 求值/chunk**（`addJni/addScan/addWrite/addHmap` 的实参）——`ChunkTiming.addXxx` 内部 `if (ON)` 已门控 adder 写入，故未开 `-Dcoreswap.chunktime` 时唯一代价是这 4 次调用（**≈80ns/chunk 为算术估计、未实测**，相对 chunk 墙钟数百 ms 可忽略）；**保留 1.21.6 逐字形态，不做微优化**（不把未请求的优化混进等价波次）；② **1.21.6 `writeChunk` 写回实现被替换为 1.20.1 的 `writeChunkPerBlock`**（**结构变化、非仅门控**）⇒ 1.21.6 侧内容等价**不能靠指令级证明**；**该版写回替换的内容等价性目前仍无证据**（judge N-未证伪项：其 pre 无 `BulkWb` / 无读回门 ⇒ 无同层 pre 对照）——已具备的正向证据仅「§4.7.3 臂 A 默认臂干净（`[WG-CONTENT]` 625 + `[WG-CONTENT-WB]` 625、非 WMI 真实异常 0）+ 采用 1.20.1 已验证写回本体（指令级）+ `SKIPAIR=false` 保持「逐格写含空气」旧行为」，**不得表述为「已等价」**，留待 D-4(i) 自身 A/B（且须先修臂 B 缺陷）；③ S-6/S-7 的两版诊断面差异已按批准生效（1.21.6 去掉无条件扫描 + println；1.20.1 `[CHUNKTIME]` 多打印恒 0 列）。
- ⚠️ **❗ dll 血统事故（本波最重要的过程教训；V3 判定曾作废一次）**：**现象** = V2/V3 首轮两臂**实际执行的 dll 不同**（post `838e8979…` vs pre `dd3b645f…`）⇒ 对照被引擎差异污染、**V3 判定作废**（靠逐臂读取 `<CppBridge> dll= sha256=` **自证行**发现，非事后猜测）。**根因链** = ① `target/release/worldgen.dll` 当时是**早前 A/B 实验留在 target 的非权威产物**（`838e8979…`，A1d 后构建）；② A/B 驱动 `run_ab.ps1` 收尾**无条件** `Copy-Item $bak $targetDll`，把 `.tmp/.../worldgen-target.bak-597e12ed`（= **1.0.28 引擎 `597e12ed`**）写回 target；③ worktree 内 `gradle :build` 触发 dll 同步链，target 又被恢复为**权威 `dd3b645f`**（= 1.0.29 票记录值）⇒ **两臂各读一个 dll**。**处置** = 权威 dll 另存 `.tmp/.../w2/dll-canonical-1.20.1.dll`；**覆盖 harness 备份**（原备份另存 `…bak-597e12ed.historical-597e12ed`）使收尾 restore 退化为 **no-op**；两臂重跑。**三元组核验（事后）** = 1.0.29 票 dll `dd3b645f…` ≡ 当前 `target/release/worldgen.dll` ≡ 重编 jar 内 `native/worldgen.dll`；1.21.6 = `abd7d889…` 三处一致。**附带发现** = `gradle :build` 的 dll 同步 `doFirst` 会把 target 拉回权威产物（本次把非权威 `838e8979` 纠正为权威 `dd3b645f`）——构建链对 dll 有**自愈**作用，但**不能依赖**（若权威 dll 不存在或同步被 `UP-TO-DATE` 跳过则不自愈，KB #56/#96 家族）。**判据升级（可复用）** = **跨臂 / 跨 run 对照前 MUST 逐臂读「执行体自证行」核对，不得只看 target 的文件 sha**——target 会被「A/B 收尾 restore」与「实验暂存」改写 → KB build-tooling **#59**（与 A 线首例合并为同一条，本事故为**第二实例**，污染对象从「交付产物」升级为**对照实验本身**）；完整五段式台账 = `errors-260912-01.md` **E1**，judge 复算一致（`review-wave2-260912-01.md` §5.1）。
- ✅ **V2（run 回归）+ V3（行为门）——1.20.1 出货线双 PASS（judge C2/C3/C7/C8 修正后）**：**三臂同配方**（唯一变量 = Java 源码状态）：seed `417950215108767439`、`chunky world overworld` + `center -48 -11` + `radius 160`、`-PcppReplace=true`、`-Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1`、**同一 dll**；臂 post = 主工作树（HEAD `997d40f`）、臂 pre = `git worktree @ ce5286b`（`.tmp/w2/pre-wt`，其 `runtime` 用 **junction** 指回主仓 `runtime/` ⇒ 共享世界/配置，**pre 臂跑 2 次**）。结果：boot **55.8s / 54.2s、51.2s**；dll 自证门**逐臂读自证行**三臂 `dd3b645f` **OK**；生成 **441 chunks / chunky 自报 post 与 pre 臂均 5s（= 88.2 chunks/s；脚本打印的 10s 是轮询粒度、非 chunky 口径 —— C8）**；异常行（`Exception|Error:`）**实测 post 4 行（全部 WMI/COM 良性）/ pre 12 行（8 行 WMI 良性 + 4 条与 CoreSwap 无关：dev 测试 mod `testcontent` entrypoint 失败 `This registry can't create intrusive holders`）**——原记录的「0 / 0」已作废（门数字未按自身证据复算，`errors-260912-01.md` **E4**；真臂 mod 集合两臂一致 = 48 ⇒ 不影响对照结论）；`[WG-CONTENT]`（Rust buf 层）**607 / 607 / 607**；`[WG-CONTENT-WB]`（Java 写回读回层）**607 / 607 / 607**；**三臂两两比对（post vs pre-r1、post vs pre-r2、pre-r1 vs pre-r2）两族指纹全部 **multiset 全等**（`only-pre=0 / only-post=0`）** ⇒ ① 引擎层输出未变 ② **Java 写回结果逐 chunk 全等**；**§15.4 校正（judge 增量复审 D2，2026-09-12）**：原「与 sorted-sequence 双判全等／序列相等另证顺序」**已废**——六臂两两原始序列**无一对相同**（同 jar 重复跑 `pre-r1 vs pre-r2` 位置差 495/503）⇒ 顺序由异步流水线决定、**不承载判据**；判据只用 **multiset**（6 臂 `fp-*.txt` sha256 全同）。工具 `evidence/tool-fp_compare.py`（多重集 + 序列双判；含「同集合不同顺序」可辨）。⚠️ **覆盖面边界（judge C3：原文「逐格写回本体未变」= 过度声称，已作废）**：三臂 JVM 均未设 `-Dcoreswap.bulkwb` ⇒ 1.20.1 缺省 `BULKWB_ON=true` ⇒ 实际走 `BulkWb.writeSections` ⇒ 本门覆盖的是 **bulk 写回路径（= 1.20.1 生产路径）**；**`writeChunkPerBlock`（逐格写回 / 回退路径）在三臂均从未执行**，其「未变」只有 §4.6 指令级证据、**无运行期证据**。⚠️ **维度盲区（C7）**：本次只生成 overworld；被改的 `writeChunk` 同时服务 nether/end 调用点（`CppBridge.java:496/:525/:576` 邻域）⇒ **nether/end 零覆盖**（`[WG-CONTENT-NETHER/END]` 命中 0），依赖既有 C 线结论与后续 A2 门、不在本波证据面内。**§9.7 口径声明（record §4.7.2）**：载体 = live dev-server + chunky（overworld，center `-48 -11`，r160，441 chunks，seed `417950215108767439`，dll `dd3b645f`）；覆盖面 = 本次写回的 441 chunk / **607 个 chunk 的两层指纹**（Rust buf / Java 读回），**仅 overworld、仅 bulk 写回路径**；可比性 = 三臂同 dll / 同 seed / 同坐标 / 同门控，**唯一变量 = Java 源码状态**（pre 臂另有 `testcontent` 环境噪声，已声明——两臂 mod 环境非逐字节同）⇒ 本判据为**逐 chunk 精确等**，与既有 C 线「同实现两 run region 层噪声 0.024%」**不同层、不可混用**。
- 🔍 **V1b 基线的 dll 归一化说明**：V1b 的 **Java 面判定**用 `post2`（内含旧 dll `838e8979`）与冻结 pre（**同 dll**）比对 ⇒ dll 条目两侧相同，**不干扰 Java 面结论**；归一化影响已隔离验证：`post3`（权威 dll）vs `post2` = 1.20.1 **仅 1 条目差异**（`native/worldgen.dll`）、1.21.6 **0 条目差异** ⇒ dll 归一化对 class 条目**零影响**（V1b 声明不受影响）。**交付提醒**：`build/libs` 现产物 = 权威 dll 版本，但**与已发布 1.0.29 的 jar sha `b057fda2…` 不同**（发布 jar 早前被本地构建就地重写覆盖，§2.6「构建产物目录不是存档目录」）⇒ 任何 1.20.1 再发布 MUST 重跑全量回归 + 三元组重算。**C4 复算（注释修补不改字节）**：`ChunkTiming.java` javadoc 改动保持**总行数不变**（逐行替换）⇒ 其后的代码行号不变 ⇒ `LineNumberTable` 不变 ⇒ `.class` 字节不变；实测 `post4` vs `post3` = **差异 0 / 非预期 0**（相同 1082），两版 jar sha 相同（`461baedc…` / `772d7a6e…`）。
- 🔍 **open（未核/降级/边界）**：① **§4.7.3 已回填：1.21.6 默认臂（生产语义 `BULKWB_ON=false`）PASS**——boot **42.1s**；bridge init `seed=417950215108767439 worldgenDir=versions\1.21.6\data\worldgen enabled=true stageMask=3` 全对；dll 自证 `abd7d889…` ≡ 该版权威（票/target/jar 三处一致）；`[WG-CONTENT]` **625** + `[WG-CONTENT-WB]` **625**（**该版首次带上读回门**）、`[WG-BULKWB]` 0（缺省关，符合预期）；非 WMI 真实异常 **0**；jar 内 `coreswap.mixins.json` 25 条含 `ChunkSectionAccessor`、启动无 `Mixin apply failed` / `Cannot find target method`。**但强制 `-Dcoreswap.bulkwb=1` 臂崩解 = 确认缺陷**：130 个 chunk 写回**全部抛异常**（日志 132 次）`net.minecraft.world.chunk.EntryMissingException: Missing Palette entry for index 2…8`，栈 = `ArrayPalette.get` ← `PalettedContainer.get` ← `ChunkSection.getBlockState`（bulk 路径里的**读**操作），`[WG-CONTENT-WB]` **0 条**、流水线未完成；**根因未定位**（候选：1.21.6 palette/并发语义 ≠ 1.20.1 / `ChunkSectionAccessor` 取字段形态差异 / section 构造路径变化）⇒ **不阻断 Wave 2（默认关）、阻断 D-4(i)**（完整五段式 = `errors-260912-01.md` **E5**、未闭合项 = record §4.7.8）；② **judge 已做**（`review-wave2-260912-01.md` = **PASS-with-conditions**，C1–C9 已全部响应/修正，record §4.7.7）——record §6 已回填（该 C1 缺口已闭）、status 已由人类裁决；③ **用户已 confirmed 2026-09-12 15:52**（范围 = 出货线 1.20.1 行为不变 + 共享核抽取 + 1.21.6 默认路径冒烟；不含 1.21.6 强制 bulk 缺陷）；④ worktree 作**条目级基线**的换行物化伪差异（`worldgen-data/**` **1024 条**差异、**class 条目 0**）**已补一手锚** = record **§4.7.6** + `evidence/prewt-pseudodiff.txt`（→ KB build-tooling #60）；⑤ javap「按行 zip 对拍」级联误报**已补一手锚** = record **§4.7.6** + `evidence/src-pre-1.20.1-CppBridge.java` 与现行源码逐字对照（→ KB build-tooling #61 / `errors` E3）；⑥ **1.21.6 写回替换的内容等价性无证据**（该版 pre 无对照载体）⇒ **不得表述为「已等价」**；⑦ **`writeChunkPerBlock` 无运行期证据**（1.20.1 回退路径；低成本闭合方式 = 跑 `-Dcoreswap.bulkwb=0` 臂与生产臂对比 `[WG-CONTENT-WB]`，未做）；⑧ **nether/end 零覆盖**（本波被改的 `writeChunk` 共享于三维度，而本门仅 overworld）；⑨ 1.20.1 class 条目仅 4 条做 `javap -c -p` 全等证明 + 一次方法级对拍（judge 抽样复核 5/5 成立），**未做全量逐条 javap 对拍**；⑩ 4 次 `nanoTime/chunk`（≈80ns）为算术估计未实测；⑪ **门数字纪律**：本波曾把异常行写成「0 / 0」（未测量即断言）被 judge C2 复算推翻（实测 4/12）⇒ 门数字 MUST 复算 + 附可复现命令 + 原始输出落盘 + 定义口径（`errors-260912-01.md` E4）；⑫ `testcontent` 环境噪声（dev 测试 mod、与 CoreSwap 无关）使两臂 mod 环境非逐字节同 ⇒ 严格单变量需摘除后重跑（未做）；⑬ **块号/日期**：本块 260912-01 与计划目录标签一致（`Get-Date` 实取 2026-09-12 14:0x）。
- ❌ **排除清单（一行，防重走弯路）**：① 「jar 字节变了 ⇒ 行为一定变了」对**纯移动波次**不成立——Wave 1 字节全等 ⇒ **V2/V3 被蕴含**（同一执行体 ⇒ 行为必然相同），该判据已写进 record §4.1 **防后波次偷懒或重复劳动**；② 计划原表述「V1 对全范围统一」**不成立**（plan §14.1 追加式修正：纯移动 vs 语义统一在产物字节上性质不同）；③ 计划 §1 目标 1 / §2 表 / §3 D-3.1 把「exec 模式 / P1 信号量 / maxinflight」列为 1.20.1 超前项并暗示进共享核 —— **错**（这些住在 `mixin/NoiseChunkGeneratorMixin`，属**范围 C**；`CppBridge` 全文对 `exec|Semaphore|maxinflight|Executor` **零命中**）；`Identifier.of`/`getOrThrow` 改名同样属范围 B/C、**不在范围 A**（plan §14.3）；④ 计划 §6.1 预期差异清单**漏项**已按 scout-judge C2 补齐（补 `CppBridge$1.class`；补声明 1.20.1 侧 `BulkWb`/`BulkWb$TL` 因 `ON` 改走 `WgCompat.flag` 而字节变）；⑤ **「异常行 0 / 0」= 「未测量即断言」**（脚本没打印异常计数 ⇒ 当作 0），judge 复算实测 **4 / 12** ⇒ 该写法已作废（`errors-260912-01.md` E4）；⑥ **「默认关 ⇒ 该路径不存在」不成立**——共享化把 1.21.6 上**原本不存在**的 bulk 路径变成**可达**（该版 pre 无 `BulkWb`、property 被忽略），强制臂一开即崩（130 chunk `EntryMissingException`）⇒ **默认关不豁免冒烟**；且被改的 `writeChunk` 共享于三维度 ⇒ 冒烟须覆盖 nether/end（`errors-260912-01.md` E5）。

---
## 260912-02（实际 2026-09-12 17:0x–18:1x，日期锚 Get-Date 18:13：1.21.6 强制 bulk storage 段帧契约缺陷定位 + F1 分版缝修复）✅ 用户已 confirmed（2026-09-12 18:13；范围 = ①F1 修复有效性 ②根因 b2a ③1.20.1「未观测到 F1 相关回归」④1.21.6 默认臂未退化；**不含** nether/end 覆盖、性能结论、`BULKWB_ON` 翻转）

> 过程产物 `.investigations/shared-java-core-260912-02/`（`scout-map.md` / `scout-interpretation-A1.md` = 静态字节码 + C1 运行级解读 / `verify-260912-02.md` = v1→v6，**confirmed（范围受限** = 上列 4 项结论）；§0 载体声明 / §4.1 F1–F29 / §5 未闭项 / §8 硬门进度 / `judge-260912-02.md` = 三轮 + 交付前确认，均 PASS-with-conditions；§12 九项硬门 / §16.1 终态确认 / `errors-260912-02.md` = W1–W13 + 速查表 / `evidence/`（含 `javap-seam-260912-02.txt`、`arm-commands-260912-02.txt`、`cmp-260912-02{,-att}-raw.txt`、`manifest-*`、`TL-javapv-*`））+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-02-1.21.6强制bulk缺陷定位.md`；提交 `2c3be2a`（F1 本体，2026-09-12 17:13:50+0800）/ `8974063 docs(260912-02): close review conditions, register artifacts, supersede E5`（2026-09-12 17:56:53+0800）。通用模式 → workflow-patterns #139（最高价值）、algorithm-fingerprints #26。

- 目标：定位 260912-01 遗留的「1.21.6 强制 `-Dcoreswap.bulkwb=1` 每 chunk 抛 `EntryMissingException`（130 / 0 / 132）」根因并修复，且不得触碰 1.20.1 出货线。
- ✅ 做了（决定性判别先行，HOOK-B）：先跑判别臂确认单候选 b2a，而非直接 fan-out ⇒ 判定树坍缩（b2b 被三处逐条同形 + {2, 8} 单变量分布排除）⇒ fan-out 未触发（纪律说明：这是预登记判定树 + 判别臂的坍缩，非主会话自推取舍）。静态面补 A1 / A1b / A3 字节码（两版 `readPacket` 偏移 5/15/24/39/45 同形、唯第 39 条 `invokevirtual` 目标不同）+ 运行面 C1 合成自检（`PASS distinct=1` 之后 `FAIL branch distinct=3 i=0 got=granite want=stone`）。
- ✅ 判据 / 证据：根因 = storage 段帧契约变更（1.20.1 `readLongArray` 前缀帧 vs 1.21.6 `readFixedLengthLongArray` 定长帧；tiny 映射名 `mappings.tiny:17600 method_10789 = writeLongArray` / `:19802 method_68087 = writeFixedLengthLongArray`）；修复 F1 = 分版缝 `WgCompat.writeStorageLongs` + 共享核 2 调用点改调（`evidence/javap-seam-260912-02.txt` 证明缝体纯转发 ⇒ 1.20.1 写出帧同构）；修复后 1.21.6 bulk 臂 625 / 625、`EntryMissingException` = 0、WBTEST 4/4、多重集对比全等（本稿按 `cmp-*.txt` 枚举 = 18 组：8 主 + 8 自证 + 2 补）、条目级 3 变更 / 0 增 0 删、1.20.1 与冻结基线整文件全 64 位 sha 同一（同 sha 家族 1.20.1 = 10 份 / 1.21.6 = 4 份）。
- ✅ 方法学硬洞补救（本块自纠）：两臂指纹全同既可能是「真等价」，也可能是「两臂都走了同一路径」（臂变量没生效）⇒ 补 3 个 `-Dcoreswap.bulkwblog=1` 正 / 负生效自证臂（bulk 臂 `[WG-BULKWB] calls=625`；perblock 臂 `calls=0` + `[WG-PERBLOCK] calls=607`）⇒ 等价结论不再是假阳性风险。
- ✅ judge（三轮：初审 / 续审 / 交付前确认）：均 PASS-with-conditions；9 项硬门（MUST-1..5 + 续审 9.1/9.2/9.3 + MUST-6）经 judge 回一手文件独立复核后全部闭合（`judge-260912-02.md:468` 终态确认；`verify-260912-02.md:350` 的记录侧「唯一未过硬门 = MUST-6」已由 v6 划改并标 ✅ 已闭合）。judge 独立抓到的最高价值问题 = C-17 的「12 vs 4」实为计数载体缺陷（无冒号 runServer 同时命中 :runServer 与 :content-test:runServer，两次 JVM 运行写同一日志、计数对整份日志做）⇒ 逐 run 切分后两侧同为 4 WMI / 0 非 WMI；同机制解释了 260912-01 的 `WMI 4↔8` / `testcontent 2↔5`，且上一块 judge 复审已记录该机制而本块未继承（双重教训）。
- 🔍 数字更正过程（可复用教训）：① 「1.21.6 五份 fp 同一」→ 穷举定案为同 sha 家族 1.21.6 = 4 份 / 1.20.1 = 10 份（`judge-260912-02.md:451-453`）；② 计划原「修复前 49 / 50」是 r16 探针臂数字（不同臂，不得同格引用），r160 真值 = 130 / 0 / 132（`judge-260912-02.md:310-312`）；③ 本稿复核发现「16 组多重集对比」按 `cmp-*.txt` 枚举实为 18 组。
- ✅ 裁决：C-17 判据重述获批（载体 = 单次 JVM 运行；原字面 FAIL 留档不改）；`BULKWB_ON` 不翻转（nether / end 覆盖 = 0 为前置）；不出货（F1 对 1.20.1 为静态转发，无紧急重发需求；若出货须升版 1.0.30 + 新工单）。
- 🔍 残留 / 边界（如实）：nether / end 覆盖 = 0；1.21.6 写出帧字节未直采（由读端 4096 点 + 625 chunk 等价性代替）；1.20.1「未观测到 F1 相关回归」仅限 overworld / 写回内容层 / 单次 JVM 运行口径；性能结论不做；`verify` = confirmed（范围受限，仅上列 4 项）；`errors` = candidate。
- 📌 open（下一轮最小闭环）：`errors-260912-01.md` E5 的**就地** `superseded_by` 注记已补（`errors-260912-01.md:134` + 速查表 `:154`，judge §16 复核通过）；`ref_merge_index` 裸列表根缺陷经 `framework-proposals/` 上报（见段 6 提案，W13 + judge R4）；登记侧计数滞后（`index.yaml:1511` 与片段 `:69` 仍写「12 条（W1–W12）」，实为 13 条，judge §16.2 R5，1 分钟可闭合）。

## 260913-03（实际 2026-09-13 16:02 起，Get-Date 锚定：R9-b 可选加固——debug 门控并发冲突检测器 sentinel）✅ 用户已 confirmed（2026-09-13；范围 = record 全部结论；judge PASS-with-conditions：0 MUST / 2 SHOULD / 4 INFO，条件已应用）

> 过程产物：已批准计划 `.investigations/000-架构设计/架构计划-260913-03-R9b并发加固.md`（轻量档，含执行记录与 judge 条件响应）；编译日志 `.tmp\sentinel-compile-260913-03.log`；运行台 `.tmp\sentinel-260913-03\run_sentinel_1201.ps1`。

- ✅ 做了：`BulkWb.java` 新增 `-Dcoreswap.bulkwbsentinel` 门（默认关）的并发冲突检测器——恢复老路径 `LockHelper` 的「并发访问同一 chunk 即 crash」可观测性（260913-02 登记的永久回归面收口）：`SENTINEL_ACTIVE` map + `sentinelEnter/Exit/Crash`（CrashException + 双方线程 dump，与 LockHelper.crash 同构；检测域 = 在飞重叠 + 非可重入，不抓顺序双写）+ armed 一次性自证行；`writeSections` 拆包裹层，主体不动。编译绿（`gradle compileJava --offline --rerun-tasks`，29s，#56 watcher 噪声已知）。
- ✅ 两臂证据（隔离运行台：不杀 java 进程 / `:runServer` 修双命中 / forceload -128..127 触发接管管线）：门关臂 `armed=0 + wb=522 + crash=0`（bulk 真跑、sentinel 零介入）；门开臂 `armed=1 + wb=522 + crash=0`（自证命中、522 chunk 零误报）。首轮门关冒烟（wbLines=0）作废——bridge init 晚于 Done（#80），非本改动问题，加 forceload 后重跑。
- ✅ judge：隔离 subagent PASS-with-conditions，推荐 candidate——SHOULD-1 已应用（「门关零开销 = 编译期常量消除」是误称：`System.getProperty` 非常量表达式、javac 不内联，实际 = `<clinit>` 单赋值 + C2 运行期折叠 → compiler-idioms #26）；SHOULD-2（共享核跨版声明）/ INFO-2（sentinelKey 注释）已应用；INFO-1（chunk 级粒度系有意升级）已入注释。
- ⚠️ 降级声明：双写者违例路径未做运行时注入（无现成注入面）——以静态论证承载（同构 + 编译绿 + 消费面审查）；分类 = Degraded（局部）：门控行为 Full / 违例路径静态。
- 通用模式 → compiler-idioms #24 补充案例（260913-03 收口）+ #26；07 篇 R9-b 小节追加状态更新（另处落盘）。

## 260914-04（实际 2026-09-14 15:29 起，Get-Date 锚：光照性能 round3——探针定线 + B palette 展开 + C packed 直传）🔍 candidate 建议（judge 收尾 PASS 8/8；e2e 判据 FAIL 1.072 如实、用户裁决收尾；confirmed 待拍板）

> 过程产物 `.investigations/light-round3-260914-04/`（record / probe-verdict / review-final-judge / scout-map / b1-b3 / cmd-output 23 件）+ 计划 `.investigations/000-架构设计/架构设计-260914-04-光照round3.md`。

- 🔍 **探针定线（三源）**：K 内核 1482.6µs（同数据 vs 12 篇 1587µs sanity 过）/ T Java 段 collect 4.5ms、native 2.8ms / P palette bits 直方图（4:95%）；**round2 减法口径「Java 段≈0」证伪 30×**（→ 12 篇 L94 supersedes + workflow #147 判据）。
- ✅ **B（palette 级展开收集）**：writePacket 公有帧路径（@Accessor 撞私有 record 不可行 → compiler #27）；collect 4.527→1.257→0.288ms；DUAL ALL-MATCH。
- ✅ **C（JNI packed 直传）**：三数组直传 + Rust light_decode_packed + 同内核；native 2.770→2.651ms（游标免除法）；DUALP ALL-MATCH、fallback=0（paldump 0 节直证）。
- ⚠️ **W1 漏读长度前缀假绿**（最高价值过程错误）：singular 节免疫位流错位 → 对拍 4 chunk 假绿 + rc=-2 静默回退 + nativeAvg 反升伪装（→ build-tooling #151 三件套签名 + 位流负载用例判据）；W2 初值自关闭 / W3 脚本返回值污染 / W4 回退均值前提（record §5 五段式）。
- ❌ **e2e 判据 FAIL 1.072（n=6 六对交错，如实）**：OFF 极差 2.6s/15% + 跨批漂移 2s → 1s 级缺口不可判（→ workflow #148 载体灵敏度判据）；3 轮未满足触发 C-gate，**HOOK-3 用户裁决接受现状收尾**（非判据通过）。
- ✅ **净收**：ON 串行 7.3→2.94ms/chunk；e2e 1.25×→~1.07×；golden 4/4 逐位。剩余：解码-查表融合 / sky_fall 融合 / e2e 载体更换 / global palette 回退计数。
- 📌 通用模式 → compiler-idioms #27、build-tooling #151、workflow-patterns #147/#148（subagent 草稿 + 主会话应用）。

## 260915-01（实际 2026-09-15：形态审计——全接管面 × 执行形态矩阵，scout×2 → fan-out×4 → 汇总 → judge C-1..C-7 → HOOK-2 全批）✅ **confirmed（2026-09-15 用户「授权确认」；范围 = 候选池排序 + 矩阵等价判定 + D-hm 结案，Degraded 分层保留；各 CP 修复后行为级结论另走验收）**

> 计划 `.investigations/000-架构设计/架构设计-260914-04b-形态审计.md`（HOOK-1 已批，260914-04 尾声立项）；产物 `.artifacts/form-audit-260915-01/`（候选池 + judge-review）+ `.investigations/form-audit-260915-01/`（p1a/p1b + p2-w1..w4 六份）。本审计零运行时改动、零 src diff（judge 三源核对确认）。

- ✅ **P1 双勘探并行**：p1a 接管面清单 + 我方形态测绘（3+1 段 + executor 横切；carver/features/biome/序列化让位段显式出矩阵）；p1b vanilla 一手形态（.tmp/scout-260905-08/mcsrc 版本核验通过 MinecraftVersion + DataVersion 3465；1.21.6 因仓库无一手源降级只标注——genSources 禁跑声明）。
- ✅ **P2 fan-out ×4**（阶段分叉互斥）：W1（A fill + B surface，11 行）/ W2（C 光照，6 open + 3 等价 + 4 辖区外）/ W3（D 写回，8 行）/ W4（executor 横切，R1-R5）。二选一纪律（等价论证或错配代价证据）逐行执行。
- ✅ **P3 汇总**（主会话收敛，不重排 worker 行集）：交叉合并——**FIFO 优先级丢失三角度独立命中（W4-R3 = W1-A-②b2 = W3-D-② 随行）= 同一发现三 worker 独立命中，嫌疑加权**；光照粘线（W2-L1/L2 = W4-R5-light）同体两面；RefCell UB 与光照线程放置修复**耦合声明（解粘即暴露，必须同批）**。
- ✅ **P4 judge PASS-with-conditions（C-1..C-7）**：×9 与 13.8× 上限推演独立重算成立、无自由参数、无孤儿行。条件：C-1 T_fill 跨 worker 不一致（W1 ~10ms vs W4 50ms）→ 预验证 4 后统一回填，此前禁引绝对秒数；C-2 W2 补 retry 声明（已应用）；C-3 binding CP-4 G3 因果链 trace 前置绑定；C-4 CP-3 落地声明「UB 可达性未实证，防御性修复」；C-5 13.8× 声明池宽依赖（4C = 129×）；C-6 index.yaml 补登记（已应用）；C-7 噪声卡历史补查（已闭合：工作区无 noise_cards.json 在册，与历史 judge ⑥ 留档一致）。
- ✅ **D-③ 范本行**：bulk vs 逐块 = 形态不同但等价且更优，全场证据最硬（260911-05/260913-03 指纹门 + 计数语义复刻 + 并发哨兵），无需动作。
- ✅ **D-hm 条件闭合（judge 独立抽查）**：ProtoChunk 写路径增量维护 4 正式型 + FEATURES 步全量重算兜底 + 2 个 WG 型两侧同形态 → 等价条件成立，按等价结案（状态提升留人类）。
- ✅ **thread::scope 等价判定**：W1-A-②b 与 W4-R4 独立得出同一结论——per-call scope spawn 1 线程开销 0.04-0.1%，被 fill 主体淹没；R4 算术复核无误。
- ✅ **W3 前置澄清（防假错配）**：D 段参照系拆双重——vanilla populateNoise 内 section 写 = 同位参照（R1）；serialize 主线程 20/tick = 让位段下游交互（R2），拿 R2 对照我方 work 线程写回 = 假错配。
- ✅ **HOOK-2**：用户拍板「按建议执行序全批」→ 候选池升 candidate。执行序：CP-3 → CP-6 核对 → 预验证 1/2/3/4 → 按 probe 定 CP-1/2/4 → CP-5 随批；round4 纯算力项继续冻结。
- 📌 结论落盘：07 篇（矩阵摘要 + 候选池总表）+ 12 篇（光照错配族）；通用模式 → workflow-patterns #149/#150/#151（subagent 草稿 + 主会话应用）。

## 260915-02（实际 2026-09-15 16:23 起，Get-Date 锚：形态审计执行序第一批——CP-3 防御性修复 + CP-6 源码核对结案 + judge）✅ confirmed（2026-09-15 16:50 用户授权确认；judge PASS-with-conditions C1-C4 已应用）

> 过程产物 `.investigations/form-audit-260915-02/`（cp6-needssaving-verdict.md + review-001-cp3-cp6-judge.md）+ 架构计划 `.investigations/000-架构设计/`（260915-02，轻量档）。

- ✅ **CP-3（光照 scratch RefCell→Mutex，防御性修复）**：light/mod.rs 13 行四改动点（use Mutex / 字段+注释 / 构造 / `lock().unwrap_or_else(|e| e.into_inner())`）；poison→into_inner 合理（scratch 每调用开头 clear+resize）；单线程语义等价（无重入路径）。**#150 纪律合规**：防御性修复先行、CP-2 解粘未动；mod.rs:64-67 注释声明「UB 可达性未实证，防御性修复」（260915-01 judge C-4 落地）。workspace 全量构建绿 + 单测 14/14 绿 + 两版 dll 新鲜（judge C1 输出存档 cmd-output/build-test-260915-02.txt；C2 措辞：预期近零成本、未测量）。
- ✅ **CP-6 结案：不立项**——needsSaving 门控在（TACR:797-802）但标脏 = status 阶段完成统一置位（ChunkStatus:362→ProtoChunk:221），vanilla 生成期块写自身也不标脏 ⇒ bulk 原地替换与 vanilla 同标脏面，疑点不成立。judge 六点一手源抽查零漂移。范围边界：限生成管线期（非生成期 bulk 写另一条链，另核）。判据 → workflow-patterns **#152**。
- ✅ **judge review-001（CP-3+CP-6）PASS-with-conditions**：CP-3 条件 C1（cargo 输出落 cmd-output）/ C2（「近零成本」措辞改「预期近零、未测量」）/ C3（index.yaml 补登记）；CP-6 条件 C4（verdict 补范围边界）——均不阻塞 candidate 推荐，主会话已全部应用。
- 📌 落盘：07 篇 D-④-b 结案小节（追加，原 CP-6 行加结案指针）+ discovered #152；CP-3 为一次性工程修复不立项 discovered（#150 已覆盖其判据面，subagent 草稿 D 节取舍）。预验证探针 1-4 未做，留下一波。


## 260915-03（实际 2026-09-15，Get-Date 锚：形态审计执行序第二批——预验证探针 1-4 五臂采集 + 判读 + judge）🔍 candidate（judge PASS-with-conditions C1-C3 已应用；confirmed 待用户）

> 计划 `.investigations/000-架构设计/架构计划-260915-03-预验证探针.md`；判据预登记 `.investigations/form-audit-260915-03/probe-criteria.md`（先于任何采集定稿，#150 纪律）；结论 → 07 篇「260915-03 预验证探针」小节；通用模式 → workflow-patterns #153/#154/#155 + build-tooling #152/#153/#154。

- ✅ **采集**：五臂 A1/A2/B1/G1/G2，执行体三元组 dll `c86718e7ab300ac6`（CP-3 Mutex 构建，16:29）×4 boot 核对；seed 8576294172403134396（备份 .bak-formprobe、每臂删 world）；A1/A2 逐 chunk 4698 配对零缺。
- ✅ **判定**：(a) 否定（S=0.79）/ (b) 否定（O=0，单边声明）/ .b1 否定（U≈19%）/ .b2 判据带内否定（R_sticky 复算 6.7%）/ .b3 否定（F=0，限驱动窗）/ G1 5.83% ≥2.5% → (i) 时机形态主导、CP-1 升首 / T_fill 92.5ms **§15.4 取代** ~10ms/~50ms 两系回填 260915-01 C-1 / A3 条件臂已评估·不触发（C-2）。G2 **VOID**（fjp1 死参数，自证门抓住，未人工挑臂）。
- ✅ **CP 排序更新**：CP-1 升首候选 / CP-4 降后（C-3 驱动窗范围限定）/ CP-2 降后 / CP-5 随批。
- ❌→✅ **过程错误 E1-E6**（五段式全文见 record + interpretation-draft §4）：
  - **E1** run_g3 追加日志读取锚未重置（pos=0 误配 run1 的 Done）→ run2 提前 stop；修 = Popen 后 `pos=getsize`。→ build-tooling #152。
  - **E2** 沙箱 `taskkill /IM java.exe` Access denied → Get-Process java 按 StartTime 归属 + 定向 Stop-Process -Id（保留 daemon）。→ build-tooling #153。
  - **E3** fjp1 死参数——1.20.1 worldgen 主 worker = 专用 ForkJoinPool(cores-1)（Util.java:183），common pool parallelism 不接线；自证门（线程分布）判读前抓住 → VOID 而非假结论。→ workflow-patterns #153。
  - **E4** parse_fp.py 段切分全局排序 vs gapMs 同线程口径错配 → R_sticky 恒 0 假读数（与 max_run 4-7 自相矛盾暴露）；修 = 按线程分组段切分；复算 6.7%（judge 独立重算一致）。→ build-tooling #154。
  - **E5** 旁证臂缺位使 (i) 主导单腿站立——旁证臂立项时应与主判据一起做「旁证失败后结论可信度」预演。并入 #153。
  - **E6** 探针④判据三态映射漏「两系之外」分支（92.5ms 落全部映射外）→ 判读按精神执行 + 交 judge（N-4 取代裁决）。→ workflow-patterns #154。
- ✅ **attempt1 归档**：`G1.attempt1{,_light_before}.log/json`（#144 纪律，复跑前归档）。
- 📌 产物路径：`.investigations/form-audit-260915-03/{probe-criteria.md, record-260915-03.md, interpretation-draft.md, judge-review-260915-03.md, cmd-output/{A1,A2,B1,G1,G2}.log, metrics-260915-03.json, metrics-B1-resticky.json, G1/G2_light_{before,after}.json}`；解析器 `.tmp/formprobe-260915-03/parse_fp.py`。

## 260917-01（实际 2026-09-17，Get-Date 锚；G3 漂移基底归因回炉——三探针裁决 + E1 VOID 事故 + judge PASS-with-conditions）🔍 candidate（judge PASS-with-conditions C1/C2a/C2b 已应用/转入落盘；confirmed 待用户）

> 过程产物 `.investigations/g3-drift-basis-260917-01/`（criteria 预登记 + record + g3-drift-errors.md E1/E2 + judge-review + candidates/.b1/.b2/.b3 + cb1/cb2/control json + cmd-output 七轮日志）；承接 260916-01 CP-1 G3 三臂反常（L 5.93% / D 8.10% → 域批「劣化」误读回炉）。执行体 dll 6F7FA3AE…2337 全臂自证；seed 8576294172403134396；载具 snap_light.py（§9.7 全臂同载具）。

- ✅ **三探针裁决链（判据读法预登记写死，#112/#150 纪律）**：
  - **C-B2 离线对拍**：字面「不等分支」命中（D-only 28/44 = 63.6%），但 stable 对照 7.3% 噪声地板使字面判据无检验力——worker 按对照校准收窄为「基底跨臂系统性偏离（超地板 ~7-9×）成立、域批独有分量无证据 Fisher p≈0.22」，§9.7 三要素齐备（judge §J3 认可，非改判）。
  - **C-B1 settle 敏感性**：rL = 99.2%（119/120，judge 逐项重数）≥80% → **PERSISTENT**——「Done+20s 快照窗时序伪差」候选**证伪**（❌，60s 与 20s changed 集基本不变）。
  - **C-B4 run3 收敛性（追加预登记，时序锚 C1 已应用：criteria mtime 14:10:40 < run_g3_run3.py 创建 14:11:12 < 首臂日志 14:13:02）**：**两臂分裂**——legacy 臂 200 changed（9.88%）≥60 → M-b 持久不稳定；domain 臂 2 changed（0.10%）≤20 且交集 1/120 → M-a 收敛不动点。
- ✅ **判据裁决（综合）**：G3 drift 基底 = **legacy per-chunk 光照路径轮次级不收敛**（漂移集换血：run3 基底交集仅 48.3%）+ 域批路径一次重载收敛到不动点；「域批有害」方向**撤销**，改「域批收敛性优于 legacy per-chunk（本载具口径）」；C-1 的 2.5% 阈值系随载体回炉（legacy 臂结构性不可达 5.93→6.27→9.88 逐轮上升；domain 臂可达但量的不是质量）。
- ✅ **关键结构事实**：基底 120 chunk 100% 聚集 spawn 邻域箱 x[15,36]×z[-24,-4]、边缘环 0%（非区域边界效应，#80 spawn 时序家族语境）；vanilla 光照形态同协议 drift ~0.8% → 基底为**光照接管形态特有**。
- ❌→✅ **E1 VOID 事故（本轮最重要过程错误，五段式见 g3-drift-errors.md）**：G17 两臂漏传 `-PlightRust` → 实为 vanilla 光照形态冒充接管臂 → SELFCERT 硬门（lightInit=0）抓 VOID、不挑臂 → 换标签 G17b-* 重跑显式补开关；VOID 数据经口径声明（vanilla、60s、单对照、n=1）作跨形态噪声锚旁证，不进判据读法。→ 复测口径失传判据 → workflow-patterns #156；E2（对拍脚本 sections_diff 与哈希值结构不对表恒空）同文件。
- ✅ **judge 收尾 MUST 审查（judge-review-260917-01.md）：PASS-with-conditions，三源数字抽验 6 处全部吻合**。条件：**C1**（追加预登记补独立时序锚——已应用；后续追加预登记 MUST 留可核验时间标记）；**C2a**（两臂分裂逐臂读法为事后裁量，取代记录/引用 MUST 携带声明行——已转入取代记录正式文本）；**C2b**（取代记录新结论 MUST 内嵌 n=1/单 seed/单载具/candidate 限定；「5×5 覆盖」类机制推演不进正文——已应用）。CONCERN K1/K2 不阻塞：K1 run3 补算过程溯源、K2 R1 方向采纳前补接管形态同配置噪声锚（#111）。
- ✅ **§15.4 取代**：260915-03「G3 drift 5.83% 由时机形态主导」归因 + 260916-01 record §4「基底=Done+20s 快照窗完成时序边界」候选机制——被取代（取代记录正式文本见该文件小节，双指针登记、原文不删不改）。
- 🔍 **遗留**：① K1/K2（上）；② 机制候选 M-a'-α/β、M-c 保持 draft，待 P-α/P-β/P-path 探针链；③ CP-1 去留排序（R1 载体换轨 / R2 载体换型）待用户拍板——α/β 通道修复为 R1/R2 共同前置；④ C-B3 宽限期探针未执行（timeout=23 排程面开放）；⑤ 域批光照值 vs vanilla 直接对拍未做（E2 修复后前置）。
- 📌 记录指引：通用模式 → workflow-patterns #156/#157/#158（subagent 草稿 + 主会话应用）；错误台账 → g3-drift-errors.md（本块独立成篇）；取代记录正式文本 → 260915-03 归因小节 supersedes 注记 + 260916-01 record §4 注记（主会话应用）；CP 排序决策输入 → .b3 §3（judge §J7 认可中立性）。
- ✅ **追记（260917-02）**：用户拍板 **confirmed**（§15.4 取代记录 + 归因结论）；CP-1 = **R1 保留 .b1 + 载体换轨**（R1 前置 = 接管形态同配置噪声锚实测，K2 升决策前置）。正式裁决 → `.artifacts/g3-drift-basis-260917-01/verdict-260917-01.md`（含时序锚 C1 + 限定 C2a/C2b 内嵌）；结论落盘 → 12 篇「G3 漂移基底归因（confirmed）」小节；workflow-patterns 新增 #159（分裂逐臂展开三条件）+ build-tooling #8/#118 家族补充案例（复测口径失传指针 → #156）同批落盘。
## 260917-03（实际 2026-09-17 16:06 起，Get-Date 锚；K2 接管形态同配置噪声锚实测 + P-α 零成本探针 + 首轮 VOID 教训）🔍 candidate（judge PASS-with-conditions B5/B6 SHOULD 待应用；confirmed 待用户；verdict 仍 draft）

> 过程产物 `.investigations/k2-noise-anchor-260917-03/`（record + k2-noise-anchor-errors.md E1-E3 + p-alpha-result + review + cmd-output 六份日志 + changed_set json）+ `.artifacts/k2-noise-anchor-260917-03/verdict-260917-03.md`（draft）+ 计划 `.investigations/000-架构设计/架构计划-260917-03-R1载体换轨前置探针.md`（用户批准）。承接 260917-01 K2（接管形态同配置噪声锚 = R1 载体换轨前置）。执行体 dll sha256 6F7FA3AE…2337（target 与打包件开工前逐字节核对）；seed 8576294172403134396。

- ✅ **K2 噪声锚（domain 臂）**：同配置连续 3 对 run-to-run（K2-W/G17c-D3、D4/W、D5/D4）changed 均 **0/2025 = 0.0000%**；检测地板 1 chunk ≈ 0.049%。§9.7 三要素：载体 = snap_light 2025-chunk region 快照（45×45 spawn 区，light 逐 chunk 对比）；覆盖面 = 单 seed 单区域 Done+60s、n=3 对；可比性 = 与 G17c/G3 系同载体可直接比，与存档口径 / vanilla 形态 0.8% **不可比**（#111）。结论：domain 形态噪声带上界取检测地板 **≤0.05%**，低于 G17c-D3 的 0.10%——G3 的 run2→run3 残余 2 changed 属**信号非噪声**。K2-W 顺带直证 domain 重载收敛不动点（vanilla 污染 81 chunk 一次 boot 全洗回）。
- ✅ **C-1 R1 带宽度参数建议（输入，非定稿）**：带 = ≤1 chunk（0.05%，检测地板）或保守 ≤0.10%（2 chunk）灰区上限；正式重述 + 判据预登记 MUST 与采集脚本同批（#112），另步做。
- ✅ **P-α 零成本日志探针（无新 run）**：核打点节奏（`n<=8 || n%256==0` 无条件打印、不受 FormProbe.ON 门控，judge §A1 一手核对）→ 零 fallback 行 = 计数 0 → 全部有效 legacy 臂 boot（G17b-L60 ×2 + G17c-L3 ×1）fallback 总计 **0** → **M-a'-α（fallback 混合通道）排除**，M-b（循环依赖，仅存于 fallback 子路径）连带证伪；**β（静默空节瞬态读，mixin :211-216 无打点）升为唯一活着的时变输入通道候选**，F3（信 stored 未重算）并列未测（P-β/P-path 待做，非零成本）。
- ❌→✅ **E2 首轮 VOID（本块最重要过程错误，五段式见 errors 台账）**：K2-D4 首跑漏 tmpdir 修复——DSH 沙箱 TEMP 重定向 → JVM `extractWorldgenDir` AccessDeniedException → lightInit threw → **整臂静默 vanilla fallback**（SELFCERT lightInit=0/hook=0/sha=none，changed 81/2025=4% 伪装 legacy 量级）；#118 SELFCERT 硬门采集完成时即 VOID，未外泄无效结论（正面案例）。修复 = TEMP/TMP + java.io.tmpdir 固化工作区 + `gradle --stop`（#32 daemon 吞 env 成对）；三连 boot 全 PASS。→ build-tooling #42 家族补充案例。
- ❌→✅ **E3 裁决不断链**：GATE 只打印不退出 → boot1 VOID 后 boot2 已带 prev 启动，kill 留半截日志（归档 K2-D5-VOID2-partial，#144/#146 换标签）；教训回写脚本 GATE 不过即 `sys.exit(4)` 断链。→ workflow-patterns #160。E1（cmd-output 目录未建 FileNotFoundError，§八.5 家族）同台账。
- ✅ **judge 收尾（review-260917-03.md）：PASS-with-conditions，推荐 candidate**——MUST 项全过（打点节奏/判据对 .b3 预登记/§9.7 三要素/wash 轮使用/VOID 排除/台账五段式/index 无越权）；SHOULD 待应用：**B5**（[SELFCERT]/[GATE] 行本体未归档，需补录 cmd-output 或注记「分量可由日志逐项重建」）、**B6**（run_k2.py 在 .tmp 灭失即不可复现，需随课题归档副本）；B7 备注（收敛态幂等性解读边界，现文措辞已合规）。
- ❌ **被排除项**：K2-D4 首轮（VOID，沙箱 TEMP）；K2-D5 首次（VOID2-partial，链未断）；P-α 排除 M-a'-α 与 M-b（通道级证伪，非结构存在性）；G17-L60/D60 臂（VOID，vanilla 形态，#156）。
- 🔍 **遗留**：① verdict 补 B5/B6 后转 candidate、confirmed 待用户；② C-1 正式重述 + 判据预登记（与采集脚本同批）；③ P-β/P-path 分辨 β vs F3（需临时诊断打点）；④ K2 噪声锚引用保持「观测下界」措辞（单 seed 单区域 n=3）。
- 📌 记录指引：通用模式 → build-tooling #42 家族补充案例 + workflow-patterns #160 + #13 家族补充案例（subagent 草稿 + 主会话应用）；错误台账 → k2-noise-anchor-errors.md（本块独立成篇）；INDEX 尾注行同批落盘。

## 260917-04（实际 2026-09-17，Get-Date 锚；C-1 R1 正式重述执行 + C-6 A/B + legacy 对照冲突发现——历史 200-changed 判 confounded）🔍 candidate（judge PASS-with-conditions C1-C4 已应用；§15.4 取代 verdict-260917-01 的 run3 层 legacy 证据——待用户拍板）

> **✅ 追记（同日晚）：用户授权确认两项**——① C-1(R1) 判据结论 **confirmed**（保守带满足）；② **§15.4 取代生效**（verdict-260917-01 run3 层 legacy 证据判 confounded + 两臂对照叙事撤销；prev_after argv 单源依赖已知情确认）。正式裁决 → `.artifacts/c1-c6-260917-04/verdict-260917-04.md`（status confirmed）。下块开工点 = P-β/P-path 分辨探针（β 主嫌疑不变）。

> 过程产物 `.investigations/c1-c6-260917-04/`（criteria 预登记 + record + cmd-output/ + b1/b2 审计）+ `.artifacts/c1-c6-260917-04/verdict-260917-04.md`（candidate）；判据时序锚 = git be1c166 @18:13:04（criteria 与采集脚本同批先于任何采集，#112）。承接 260917-03 C-1 带宽度建议升格 + C-6 A/B 判据预登记。执行体 dll 6F7FA3AE…2337 全臂自证；seed 8576294172403134396。

- ✅ **C-1(R1) 判据轮（域臂，预登记二值判据）**：fresh world → run1(预热，changed 168 不判) → run2 → run3，各 Done+60s；**run2→run3 changed = 2/2025（0.0988%）→ 保守带满足（≤2），主带（≤1）未中**；逐 boot 自证全绿 ×3（lightInit ok / hook armed / dll 6f7fa3ae / fallback=0）。与 G17c-D3（=2）完全复现，域臂行为稳定。§9.7 三要素齐备（snap_light 2025-chunk 同载体，与 K2/G17c/G3 系可比）。
- ⚠️→✅ **计划外发现（本块最高价值）：C1R1-L（legacy 对照臂）fresh 干净链 run2→run3 = 0**——与 verdict-260917-01「legacy 轮次级不收敛（G17c-L3 = 200/9.88%）」直接冲突。fan-out 双 worker（强制触发 ≥2 互斥候选）收敛：**b1 成立（高置信）——G17c-L3 第三 boot 时盘上 world 已被 G17b-D60（domain 臂）rmtree 重建覆盖**（run_g3_settle.py:91-92 每臂删 world；260917-01 运行序 = G17b-L60 → G17b-D60 → G17c-L3；run_g3_run3.py 不删 world 且无 world 身份自证；G17c-L3.log Done 5.615s = 加载既有 region 形态）→ 其 prev_after（G17b-L60 run2 快照）与盘上 domain world 为**跨世界对比**，200-changed 不能证 legacy 自身不收敛。结构旁证：S200∩S166=77 > S200∩S127=63，200 集亲缘偏 domain 侧。b2：本轮 0 为真测量值（假 0 机制逐一排除，common=2025、run3 加载态、检测力 128 旁证）。
- ✅ **§15.4 取代登记（正式文本见 verdict-260917-04 §2）**：verdict-260917-01 中 run3 层 legacy 证据（G17c-L3 changed=200 + 换血 48.3%）及「legacy 持续漂移 vs 域批一次收敛」两臂对照叙事被取代；新结论（candidate，n=1 单 seed 单区域）= 受控 fresh 同 world 链上 legacy 在 run2 吸收首轮修正（~6%）后 run2→run3 收敛（0 changed），域批臂同层 = 2（保守带内）；M-b「持久不稳定」失去直接证据、M-a 获两臂正证据。不受影响：C-B1 rL=99.2%（run2 层）、spawn 箱聚集、vanilla 0.8% 锚等。原结论正文不删不改；**生效前提待用户知情确认 G17c-L3 prev_after argv 单源依赖（record-260917-01.md:15）**。
- ✅ **C-6 性能 A/B（记录项完成）**：FP-LIGHT 各 3703 calls；L 6.263 ms/call（=1 chunk）vs D 8.808 ms/call（~9-center 域批任务，task avgMs 8.853 → 折算 **~0.98 ms/chunk，调用层口径 ~6.4×**，非端到端结论、不构成回归证据）；FP-FILL P50 +9.8% 落 #103 ±10% 噪声带边缘不独立立信号（顺序采集未交错已声明）；C6-D timedOut task=115 / degraded=0（宽限期边界，如实记录）。
- ⚠️ **A1 判别降级声明**：预登记判别设计用 C6-D 臂内 lightTiming，但 LIGHTPROBE-T per-chunk 打点结构性不覆盖域批路径（D 臂 0 行）→ 正式判别不可达；只能用 L 臂占比 94.71% + D 臂 task avgMs 折算间接评估；正式判别须新增域批路径计时行后重采。
- 📌 通用模式 → workflow-patterns #161（跨臂 run3 复用型协议的盘上 world 身份陷阱，subagent 草稿 + 主会话应用）；证据指针 → verdict-260917-04 §5（b1 worker 4e305749 / b2 worker 3b275362 / 审计脚本 .tmp/c1c6-260917-04/）。

## 260917-05（实际 2026-09-17；P-β/P-path 分辨探针——β 与 F3 干净重载协议下均 QUIET，inputDiff23=46 立附带发现）🔍 candidate（judge PASS-with-conditions S1-S5 已应用；confirmed 留用户）

> 承接 260917-04 下块开工点（P-β/P-path 分辨探针，β 主嫌疑复核）；判据预登记 criteria-260917-05.md
> （时序锚 eaa0a5f @19:09:42 + 采集修复 eb3988f @19:10:37，均先于采集）。三轮采集：pbeta05a VOID
> （编译错 rc=1）、pbeta05b VOID（#42 家族第三犯：tmpdir 未固化 → lightInit threw → 整臂 vanilla，
> SELFCERT 正面拦截）、**pbeta05c 有效**（SELFCERT 三 boot 全绿：Done≥1 / lightInit ok / probe armed /
> fallback=0 / domain_hook=0 / dll 6f7fa3ae；path census 每 boot legacy-rust 529）。

- ✅ **C-β 判定（candidate，n=1）**：β（空节瞬态读）在干净重载协议下 **QUIET**——emptySec 逐 chunk
  两 run 恒等 + 全局 run2=run3=71871（run1=71900）；F3（信 stored 未重算）**未检出**（changed23=0，
  存在性不证伪，只证本载具本协议无现象）。与 260917-04 新基线（legacy 干净链 run2→run3=0）自洽。
- ⚠️ **独立推翻一次汇总定性（§16.3 交接验证）**：主会话机械交叉「inputDiff23=46/46 = packed↔blocks9
  ABI 切换」被 verdict worker 日志抽样推翻——全日志 `abi=blocks9` 0 行，10 个 diff chunk 三 run 均
  abi=packed、hash 两两不同、emptySec 恒等。46 重定性为**附带发现**：packed payload 跨重载不稳定、
  output-neutral、机制 open（palette 序/位打包/空节集合形态三候选未分辨，@anchor.idk 在案）。
- 📌 预登记分支覆盖缺口如实声明（#154 家族）：「inputDiff>0 且 changed=0 且输入恒等性可证」未单列，
  驱动机械退出码 1 的分支 3「第四通道」措辞失去对象，未开 fan-out（数据面无互斥分叉）。
- 📌 通用模式 → workflow-patterns 新发现一条（汇总交叉定性 MUST 回原始日志抽样 + 计数恒等≠集合恒等 +
  hash 探针随行打印口径；已应用定号 #162）+ build-tooling #42 补充案例第三犯一行。
- 过程产物 `.investigations/pbeta-260917-05/`（criteria + cmd-output/ pbeta05a/b/c + knowledge-drafts/ 三份
  subagent 草稿）；§9.7：载具 = snap_light+Done+60s（C-1 系可比）+ [LIGHT-BETA]/[LIGHT-PATH] 新口径
  （无历史可比）；n=1 单 seed 单区域，不外推。

> **（同块追记：框架升级执行面 — Anchorlaw v0.21→v0.22 / RE-Framework v2.5→v2.6 + 论文研究落地，B1→B7）** 🔍 candidate（内容 sha256 对账 + selfcheck 五段全绿；judge 待走 / confirmed 留用户）

- ✅ **B1 技能同步（全绿）**：`pwsh E:\PYTHON\RE-Framework\dsh\scripts\install.ps1` —— **首次因 v2.6 新增的 profile patch 事务备份写 `C:\Users\NDark\.dsh\profiles\web\cordis.patch.yml.bak-ref-install` 被沙箱拒绝（工作区外）→ install 中止 → escalation 后成功**（本块最重要环境事实，见下「过程错误」）；`pwsh E:\PYTHON\Anchorlaw\dsh\scripts\install.ps1 -Project E:\PYTHON\CoreSwap` → 11 anchor-\* 更新至 v0.22；**项目级 17 ref-\* 手工同步**（无脚本覆盖，见下）：8 个 drifted（core-artifact / core-fanout / core-judge / core-knowledge / core-plan / re-lift / ref-maintain / swe-guide）按**显式文件清单** Copy-Item（禁通配，§八.4）。**验证** = 递归逐文件 sha256 对账 `ALL GREEN (17 skills, recursive)` + 版本标记 36× v0.22 + `selfcheck.ps1` 五段 `ALL CHECKS PASSED`（新内容对账段：`preset content reconciled: 37 artifacts (0 missing, 0 drift, 0 orphan)` + `user-global content reconciled: 17 artifacts (0 missing, 0 drift, 0 orphan)`）。注：`.dsh/skills` 不入 git（本地安装产物），无提交，验证依据 = 内容 sha 对账 + selfcheck。
- ✅ **B7 AGENTS.md 基线 v0.21 → v0.22**：五处编辑（头部协议段 / skill 安装记录段 / 同步契约基线段含**宿主侧待办三项** / §〇 Anchorlaw 仓库行 / §〇.1 用户级技能行）+ 新增 §一 条款 9-12（§9.8 验证副作用与逆 / 判据前置集 / halt 即终止（PI-1 + PI-2 前提）/ 交接 claim 证据分级）+ §一.6/.7 引用更新为 v0.22 并补等价分层与无效声明清单。注：`AGENTS.md` 在 CoreSwap 被 .gitignore（本地运行时文件）→ 无提交，改动即生效。
- ✅ **B2 PI-2 前提枚举（协议 MUST 前置，首执行）**：产物 `.investigations/pbeta-260917-05/pi2-dependency-enumeration.md`——枚举 **4 份活跃判据 artifact**（K1 `c1-c6-260917-04` / K2 `cp1-light-form-260916-01` / K3 `g3-drift-basis-260917-01` / K4 `pbeta-260917-05`）及其依赖边；两条「判据/结论互引」边（K2→K3 系 G3 阈值、K4→K1 系 260917-04 新基线）**逐条判向均为单向后向引用** ⇒ **当前活跃判据依赖图无环**，PI-2 可按协议原样采用（无需改 "cycles permitted but registered"）。方法 = **边对象分类法**（边指向「外部事实」还是「另一条判据的结论」——只有后者有环风险）。登记的前提与盲区：覆盖面 = 4 份 artifact + verdict 引用面；未覆盖 = 无 criteria artifact 的历史课题 + **未来新增判据（不承担证明义务）**；未用形式化图工具（cheap enumeration 即协议要求）。
- ✅ **B3/B4 §9.8 + 判据前置集实操落地（首实例）**：`criteria-260917-05.md` 追加两节——**判据前置集**（5 项 key/expected/check：`dll_sha256` / `light_init` / `probe_armed` / `world_identity` / `seed` + 前置失效处置声明）+ **验证副作用与其逆**（5 条 effect→inverse 表，含 **2 条显式不可逆声明 + 理由**：world 删除（生成产物、删除即等价）/ `server.properties` 备份为**首次快照而非本轮前状态**）。AGENTS.md §一.9/.10/.12 为条款面；本文件为该条款在本工作区的**首个实例**。
- ✅ **B5 触发覆盖审计工具试跑（只读）**：`node E:\PYTHON\RE-Framework\dsh\tests\audit_trigger_coverage.mjs --all .investigations` —— 本课题（pbeta-260917-05）单独跑 exit 0（四项均为咨询性「未触发/需确认」）；全仓跑约 **90 课题 → 1 MISSING（exit 1）**，命中 `v5-residual`「存在 candidate 状态但无审查型产物」。**核实为假阳性**：该记录自述「candidate 已 judge：有条件 PASS 260902-04」，审查产物存放在**兄弟课题目录**（260902-04）而非课题自身目录 ⇒ 与上游实测 FP 率 **1.1%** 一致；**FP 主因 = 产物跨目录存放**（工具按课题目录边界扫描）。处置（不执行）：保持只读报告工具、**不接入 selfcheck**（上游自标 Unverified + 本仓 FP 已复现）；若未来接线，先解决「审查产物跨目录」的归属识别 + 课题内 `DECLARED-SKIP(reason)` 豁免。
- ❌→✅ **过程错误（本块升级面唯一错误链，五段式）**：**现象** = `install.ps1`（RE-Framework）首次执行写 `.bak-ref-install` 被沙箱拒绝（路径在 workspace 外）→ install 中止；**根因（机制）** = v2.6 新增的 profile patch **事务备份**（`install.ps1:104-119`，backup→restore，注释自述 *"The patch file is host-shared state … Back up first."*，与论文 §5.2.2 Algorithm 10 同源）把写入面从「目标文件」扩到「**备份路径**」——而该路径是**首次出现的新对象**，其可写性从未被历史命令验证过 ⇒ **「更安全」的机制引入新的环境前提，是风险转移而非纯收益**；**定位** = 读失败路径，其命名（`.bak-ref-install` 后缀）是升级前不存在的 ⇒ 先怀疑**新机制**而不是**老命令回归**；辅证 = 上游源码定位到新增段；**修复** = escalation 后重跑成功 + **纪律层**：升级后首次运行 MUST 视为 **escalation 窗口**（不放进无人值守/批处理链）；对上游每个新增「更安全」机制 MUST 问「它新增了哪些写入对象」并登记（本条登记物 = `cordis.patch.yml.bak-ref-install`）；升级后 MUST 检查「失败时是中止还是静默降级」（本例中止是**可接受**的——静默降级的「更安全」机制比没有更危险）；**教训** = **「老命令照跑」不成立**（命令没变、环境前提变了）——沙箱类失败先核「这次多写了什么」，不先怀疑权限配置回归。⚠️ **idk**：备份路径**已存在**时 `Copy-Item -Force` 覆盖是否仍被拒（创建新文件 vs 覆盖已有文件的沙箱策略差异）本块未单独复现。
- 📌 **通用模式 → 本轮新增五条可复用判据**（草稿见 `.investigations/pbeta-260917-05/knowledge-drafts/`，已应用定号 #163-#167）：① **门禁只判「存在/数量」= silently-green 门**（28 个目录一直绿而 8 个 ref-\* 内容 drift；上游 v2.6 恰把 selfcheck 第 3 段从 `$count -lt 17` 升级为逐件 sha256 内容对账，其注释自述与 preset 行解析门禁属**同一失败类**："a gate that silently goes green when it cannot execute is the same failure class as the incident it guards against"）；② **多副本安装树的「无脚本覆盖面」**（RE-Framework `install.ps1` **无 `-Project` 参数** ⇒ 项目级 17 ref-\* **无脚本写、无门验** = 漂移黑洞，而它恰是 Rank 100 最高优先命中的副本；判据 = 逐副本问「谁写、谁验」，答不出「谁验」即黑洞；覆盖面按**副本树**而非命名空间核对）；③ **逆登记 MUST NOT 伪造「可还原」**（§9.8：不可逆 + 理由才是合规形态）；④ **判据依赖图无环判定 MUST 分类边对象**；⑤ 配套 FP 形态：**按目录边界扫描的覆盖工具遇跨目录存放产物必假阳性**。
- 🔍 **遗留 / 待办（承接 AGENTS.md 宿主侧待办三项）**：① **PI-2 采用前的枚举已做（B2），但「新增 criteria artifact 时重跑枚举」尚无自动触发**——落地动作待排（PI-2 的 fan-out 汇聚纪律「`.bN` 候选在汇聚前不得被单独引用为结论」亦待写入 fan-out 收尾）；② **判据 artifact 的 `preconditions:` 字段**已在 criteria-260917-05.md 首实例化，采集脚本/verdict 的 `inverse` 字段**尚未全量接线**；③ **B6 项目侧接管面契约**（b1 透镜 B1/B2/B3 = 生产代码副作用面，协议不覆盖）**未执行**，仅出立项建议（优先级建议 B6-1「开关生效机械对账」：`build.gradle` 约 100 行手工 `-P`→`-D` 映射 × 各处 `System.getProperty` 消费点两份事实源零机械核对，#8/#19/#47/#56 四家族已致灾；与 v0.22 §1.3「presence ≠ satisfaction」同源）。三项均需走 core-plan Phase 0。
- 📌 **产物路径**：`.investigations/pbeta-260917-05/{upgrade-execution-record.md, pi2-dependency-enumeration.md, criteria-260917-05.md（末两节）, knowledge-drafts/（本轮 subagent 草稿三份）}` + `.investigations/paper-2608-25512/remaining-after-upgrade.md`（升级后复核：已吸收 vs 待执行）+ 上游一手 `E:\PYTHON\RE-Framework\dsh\scripts\{install.ps1, selfcheck.ps1}` 与 `E:\PYTHON\Anchorlaw\spec\protocol-v0.22.md`（§9.8 :950-964 / §15.4 PI-1·PI-2 :1606-1629）。

---

## 260918-01（实际 2026-09-18，Get-Date 锚 10:43；**B6-1 开关生效机械对账第二轮收尾**——union 口径缺陷自曝 + 1.21.6 侧缺口实测 + 13 项补映射发射实证闭合）🔍 draft（judge review-001/review-002 均 PASS-with-conditions 且条件已闭合；**confirmed 留用户**；1.21.6 侧为 static 抽取 Degraded）

> 承接 260917-06（B6-1 立项块：T1/T2 抽取 → fan-out b1/b2 → T4 汇聚裁决 → 三哨兵 → T5 落地，提交链 `3e0d434`→`1ff3a6f`→`7abed4d`→`56dc15c`）；本块（260918-01）处理其收尾遗留并自曝门禁口径缺陷。判据依据 = `.investigations/b61-260917-06/t4-adjudication.md`（族内对称性判据 + §3.4 豁免子句）+ `phase25-verification.md`（三哨兵）。
> **本块提交链**：`cf77963`（scripts/ gitignore 发现记录）→ `ab91a0e`（judge-002 条件闭合 + 脚本入库）→ `0781f98`（sentinel-D：13 项映射发射实证闭合）→ `0b0c7ac`（门禁 per-version 段修复）→ `b30d008`（1.21.6 确证 + B6-2 建议）。HEAD = `b30d008`。

- ✅ **门禁 per-version 口径修复（本块最重要产物，且是「门禁自曝缺陷」的实证）**：`scripts/check_switch_mapping.py` 新增 **per-version 缺口段**（按版本分别算 `consumed_ver ∪ consumed_shared ∖ declared_ver`），`--strict` 语义升级 = **DEAD 非空 或 任一版本有缺口** 即非零退出，并打印「union 口径的 ORPHAN 会掩盖单版本缺口——请以 per-version 段为准」警示行。→ 通用模式 **workflow-patterns #168**（commit `0b0c7ac`）。
- ⚠️→✅ **本块自曝缺陷：union 口径掩盖单版本缺口（五段式见草稿 #168）**：原实现对两版 `build.gradle` 的 `-D` 名取**并集**（M1）、消费面取并集（M2），`ORPHAN = M2 ∖ M1` ⇒ **任一版本声明过的名字对所有版本的缺口同时消失**，且**掩盖方向与「哪个载体先被修好」耦合**——本块修好 1.20.1 的 `blobProbe.*`/`colprof.*`/`surfacedump.dim`/`bulkwb*`（ORPHAN **24 → 11**，−13 与补映射项逐名吻合、judge 独立复算零回归）后，**1.21.6 的同名缺口 10 项静默消失**。判据 = 跨载体聚合门 MUST 同时给 per-carrier 视角 + `--strict` 判据域与「用户可踩到的路径」对齐（任一单版缺口即失败）。家族 = **#105 载体偏差的门禁口径形态 + #163 silently-green 门的聚合形态**。
- ⚠️ **1.21.6 侧同病实测（per-version，含共享 carrier，static 抽取 Degraded）**：1.20.1 声明 128 / 本版+共享消费 136 / **缺口 11**；1.21.6 声明 106 / 本版+共享消费 124 / **缺口 21**（另含独有缺口 `coreswap.stallwatch`）。21 项中 **10 项 = 本块刚在 1.20.1 修好的同名缺陷**。**消费点一手核准确证至少 3 项为真缺陷**（非旁路）：`coreswap.bulkwblog`（`BulkWb.java:91` + `CppBridge.java:628`，1.21.6 `build.gradle` 零命中 ⇒ 生产写回链诊断仪器不可达）、`coreswap.stallwatch`（`StallWatch.java:21` ⇒ 停滞看门狗无法经 `-P` 启用）、`coreswap.wbcontent`（同族写回读回层，同 #150 家族）。**未逐项定性**：其余多数疑为与 1.20.1 同类的合法旁路（`bench.*` / `height.*` / `chunkRandom.seed` / `java.io.tmpdir`），**不得按 21 个缺陷计**；1.21.6 boot **未实跑**（Degraded）。
- ✅ **13 项补映射发射实证闭合（原 7 项静态外推已消解）**：sentinel-D 一轮传齐**全部 13 个 `-P`**，实测 **13/13 发射无缺席**（`cmd-output/sentinel-D-all13.log` 原文：`-Dcoreswap.exec=0` / `-Dcoreswap.maxinflight=8` / `-Dcoreswap.bulkwblog=1` / `-Dcoreswap.wbcontent=1` / `-Dcoreswap.bulkwbtest=1` / `-Dcoreswap.bulkwbsentinel=1` / `-DblobProbe.chunkX=1` / `-DblobProbe.chunkZ=2` / `-DblobProbe.size=3` / `-DblobProbe.dim=minecraft:overworld` / `-Dcolprof.x=4` / `-Dcolprof.z=5` / `-Dsurfacedump.dim=minecraft:the_nether`）→ T5 §8 遗留项 4 闭合；分层仍为 **Partial**（止于参数发射面）。→ 通用模式 **workflow-patterns #170**（发射面直证法，commit `0781f98`）。
- ✅ **scripts/ gitignore 缺陷处置（原「仅记录」→ 本块落地）**：`.gitignore` 由 `scripts/`（整目录）改为 **`scripts/*` + 三条 `!` 白名单**（`check_switch_mapping.py` / `merge_index.py` / `scan_cpp_anchors.py`），三脚本入库（commit `ab91a0e`；`protocol/` 仍整目录忽略）。**根因** = **git 不支持在被排除的目录内 re-include** ⇒ `scripts/` + `!scripts/xxx.py` 是**静默失效**写法（`!` 行看着在、恒不生效）；此前 AGENTS.md 明文引用的门禁在仓库中**不存在**（只有本机有），换机/CI 即失。核验 = `git ls-files scripts/` 恰含三项。→ 通用模式 **build-tooling #155**（#24 同族第二形态）。
- 📌 **判据完整表述的合读纪律（judge M1 补，本块登记）**：**「族内对称性判据」单独引用会误判**——完整表述 = `t4-adjudication.md` **§2 规则 + §3.4 豁免子句**（**探针专用路径** ∧ **缺省权威**，二者须逐条登记 reason）；三条豁免项 = `bench.threads` / `bench.worldgen` / `colDump.targets`（族内虽有 `-P` 先例，但其先例服务的是「把探针输出导向文件」类通用参数，而该项是诊断性覆盖且缺省即权威 ⇒ SCOPED-DIRECT-D）。另一方法面 = **同一现象的两种相反解释用可 grep 的结构事实区分**：b1 持消费点 javadoc（明写 `-D` 用法）判旁路、b2 持族内映射事实判缺陷——双方各自持有对方没有的判据，裁决 = 两条合读（`colprof.*` 类「族内有先例」判缺陷；`height.*` 类「族内零先例」判旁路；b2 从未判 `height.x/z` 为缺陷，本动作属对其让渡项的确认而非推翻）。→ 通用模式 draft #171（INDEX 行已含；正文可选）。
- ❌ **被排除/让渡项（保留不删）**：① `max.bg.threads` = **UNRESOLVED**（消费方疑在 MC 发行 jar 内 `Util.getAvailableBackgroundThreads`，不在本仓库扫描面 ⇒ **不判缺陷也不判废弃**，**不得与 fjp1 同格引用**——机制不同：fjp1 = JVM 读了但被专用池架空，该项 = 消费方可能在别处）；② 死开关**只加注记未删**（`biome6oct` / `cpp.noBatch` / `ForkJoinPool.common.parallelism`）——删除改变用户可见开关面，属行为面变更，**待用户裁决**；③ 1.21.6 侧 b2 声明的 `BulkWb.java:81` 类注释与 `1.21.6/10-timewise-archive.md:112`「`BULKWB_ON` 已翻转」**状态不一致**（本分支未复核）⇒ 转 1.21.6 覆盖课题。
- 🔍 **遗留（交用户裁决 / 建议另立 B6-2）**：① **B6-2：1.21.6 侧开关映射补齐**（范围 = gate per-version 清单 → T4 族内对称性定性 → 补映射 → 哨兵验证；可立即做的最小项 = 补 `-Pstallwatch`/`-Pbulkwblog`/`-Pwbcontent`/`-Pbulkwbtest`/`-Pbulkwbsentinel` 五行）；② 死开关删除裁决；③ `max.bg.threads` 补核（需反编译或实测）；④ **端到端 Full 验证未做**（本块止于参数发射面）；⑤ 其余 ~6 项 DEFECT（探针族子参数未逐个补）按族代表 + 规则外推，未逐项哨兵。
- 📌 **记录指引**：通用模式 → workflow-patterns **#168/#169/#170**（+ #171 简记）+ build-tooling **#155**（subagent 草稿 + 主会话应用；草稿见 `.investigations/b61-260917-06/knowledge-drafts/`）；证据指针 → `t5-landing.md` §4.0/§7/§8 + `extension-per-version-gap.md` §2/§6 + `phase25-verification.md` §3/§4 + `review-001.md`/`review-002.md`（judge 独立复算 ORPHAN 24→11 零回归）；INDEX 尾注行同批落盘。


> **（同块追记：门禁有效性自证 — B6-1 round 5，注入式负向测试 + 输出口径缺陷自曝）** ✅ Partial（双注入实测 + 逐字节精确还原，`git diff` 为空；**未跑 JVM 端到端**；confirmed 留用户）

- ✅ **门禁有效性自证（本块 round 5 核心动作）：双负向测试，两个检测维度都能真失败**。判据来源 = **知识库 #163**（判据只落「存在/数量」的门恒绿，「绿」只证明门跑了、不证明被保护对象没坏）⇒ 推论 **一个不能失败的 gate 等于没有 gate**，故对同一 gate 的两个维度各注入一次：
  - **负向测试 A（主检测：漏映射 → per-version 缺口 / ORPHAN）**：删除 `versions/1.20.1/java/build.gradle` 中已修复的 `-PsurfaceDumpDim` → `-Dsurfacedump.dim` 映射行（模拟「有人误删」回归）。实测原文 `[INJECTED] removed: if (project.findProperty('surfaceDumpDim') != null) run.vmArg "-Dsurfacedump.dim` / `surfacedump.dim` / `[1.20.1] 声明 127 / 本版(+共享)消费 136 → 缺口 12` / `rc=1` ⇒ **四项自洽**（声明面 **128→127**、缺口 **11→12**、命中项 `surfacedump.dim` 出现在缺口清单、`--strict` **rc=1**）。
  - **负向测试 B（作用域检测 #47：`run.vmArg` 落在接收 `run` 的闭包之外）**：在文件末尾（`benchVmArgs` 闭包**之外**）追加一行 `run.vmArg "...-Dscope.violation=1"`。实测原文 `[OUT-OF-SCOPE] versions\1.20.1\java\build.gradle:293  if (project.findProperty("scopeViolationTest") != null) run.vmArg "-Ds` / `strict rc=1` ⇒ **作用域检测可失败** ✅。
- ✅ **两次注入均逐字节精确还原（自证的第三段，缺此则门后所有绿被污染）**：`[RESTORED] from .tmp/b61-260917-06/bg-negtest-backup.gradle` + `git diff --stat HEAD -- versions/1.20.1/java/build.gradle → （空）` + `CONSISTENT: 125 / [1.20.1] 缺口 11 / [1.21.6] 缺口 21` ⇒ 还原**逐字节精确**（`git diff` 为空），gate 回到基线；主会话侧复核另确认 `git status --porcelain` **为空** 且基线数字 **M1 129 / DEAD 4 / ORPHAN 11** 在位。测试脚本 `.tmp/b61-260917-06/negtest_round5.py`（**唯一临时区，不入库**）。
- ⚠️ **本块第二处自曝缺陷：输出口径未声明导致主会话误读（错误优先，含五段式）**——**现象**：本轮排查中主会话一度把 **per-version 段**的 `surfacedump.dim` 误读为「union ORPHAN 里仍有它」→ **短暂误判「gate 有 bug」**；实际 union ORPHAN **不含**它（1.20.1 侧已修），它**只在 1.21.6 段**的缺口清单里。**根因**：原输出中 union 段与 per-version 段**并列打印同名项**，两段标题强度相近（`-- ORPHAN --` vs `-- per-version 缺口 --`），且**未标注「同名不同义」**——`union ORPHAN` = 两版**都**未声明，`per-version 缺口` = **该版**未声明（可能另一版已声明）⇒ **判据的读数域与判据语义脱钩**（数字对，读出来的结论不是它说的那件事）。**定位**：误读当场被**自证矛盾**打断——若 union ORPHAN 真含该名，则与本块刚补过该映射的已知事实冲突 ⇒ 回头逐段核对标题与集合定义确认；**判错动作 = 读数与已知事实冲突时先核「看的是哪一段/哪个口径」，再怀疑工具**（本轮因此未走「立 gate bug 课题」的弯路）。**修复（三处，已落地）**：① 段标题加口径标记 `---- [union 段] … ----` / `---- [per-version 段] … ----`；② 开头加**口径提示**（`scripts/check_switch_mapping.py:165-167`，末句写入误读史「260918-01 实测：主会话曾误读一次」）；③ per-version 段**逐版量化掩盖量**（`:192-195`，打印「↑ 其中 N 项**不在 union ORPHAN 中**（= 另一版已声明，掩盖发生在此）」并逐项列出）—— 修复后实测 1.21.6 段显式报告「其中 **12 项不在 union ORPHAN 中**」，**把此前需人工集合运算才能得出的掩盖规模变成直接可读数字**。**教训**：**多口径工具的输出 MUST 显式声明同名项的口径差异并把差异量化**——否则读者（含 AI）会把不同口径的同名项当同一结论，制造**「工具坏了」的假警报**（本例）或**「一切正常」的假安全**（#168）；**判据的可读性属判据有效性的一部分**（机器不读提示 ⇒ 量化数字必须与清单**同格**出现）。
- 📌 **通用模式（本轮两条，均 subagent 草稿 + 主会话应用）**：**workflow-patterns #172**（门禁有效性 MUST 用注入式负向测试自证——判据形态三段 = **注入 → 观测命中 + 非零退出（期望值预登记）→ 精确还原**；**每个检测维度各注入一次**；**退出码不可省**（消费者是机器，#160 同源）；**注入本身是 in-place 副作用，逆为必备项**（备份件 + 双步还原验证）；与 **#163 分工**：#163 诊断（只判存在/数量的门恒绿）、本条开方（固定的证明动作）；分层 **Partial**；未注入 = 命名一致性维 / DEAD 类 / 1.21.6 侧，均如实声明）+ **workflow-patterns #173（错误优先）**（**多口径输出 MUST 声明同名项口径差异 + 量化**）；两条**独立成条**的核心理由：**#168 主体是被读方/聚合掩盖（防假安全）、#173 主体是读方/输出未声明口径（防假警报），判据不可互推——且本轮实证 per-version 段（#168 的修复产物）恰是制造本次误读的载体，即 #168 的修复制造了 #173 的暴露面**，两条宜并提而非合并。
- 📌 **覆盖面与降级声明（§9.7）**：**覆盖面** = A 覆盖「主检测的命中与退出码」、B 覆盖「作用域检测的命中与退出码」、还原覆盖「注入不残留」；**未覆盖** = ① 未跑 JVM 端到端（注入后未实跑 `runServer` 观测行为差异）⇒ 分层 **Partial，不得称 Full**；② 未测 1.21.6 侧注入还原；③ 未测 DEAD 类注入（DEAD 与 ORPHAN 共用集合运算路径，风险低——判断非实测，idk）；④ 未注入命名一致性维（#19）。数字口径提示：**注入态**（声明 127 / 缺口 12）与**基线态**（声明 128 / 缺口 11）分属两次运行，引用时不可混格。
- 📌 **§9.8 副作用与逆（本轮三条，独立字段不并入 `source=`）**：① 临时改 `build.gradle`（两次注入）——in-place，逆 = 备份件 `.tmp/b61-260917-06/bg-negtest-backup.gradle` + 还原后 `git diff` 为空实证 + git 本体可 revert；② 新脚本 `negtest_round5.py`——derived，identity（`.tmp` 不入库）；③ 改 `scripts/check_switch_mapping.py`（输出可读性）——in-place，逆 = git 提交（可 revert）。
- 📌 **证据指针**：`.investigations/b61-260917-06/gate-negative-tests.md`（§0 判据来源 / §1 负向测试 A 原文 / §2 负向测试 B 原文 / §3 还原验证 / §4 输出可读性缺陷五段式 / §5 覆盖面与降级 / §6 §9.8 副作用与逆表）+ 门禁实现 `scripts/check_switch_mapping.py`（union 段 `:168-175` / per-version 段 `:177-198` / #47 作用域 `:137-141` / #19 命名 `:142-147` / 口径提示 `:165-167` / `--strict` `:236-240`）+ 测试脚本与备份件 `.tmp/b61-260917-06/{negtest_round5.py, bg-negtest-backup.gradle}`；对照面 = `extension-per-version-gap.md` §2/§3（#168 的 union 掩盖）；本轮草稿见 `.investigations/b61-260917-06/knowledge-drafts/`（`*-r5.md` 三份）；INDEX 尾注行 = 260918-01 追加（二），同批落盘。

---

## 260918-07 块（实际 2026-09-18 20:0x-20:3x）：B6-3 S4 能力演示（判据前置集工具侧落地）

- 🔍 **任务**：收口 260918-06 judge 悬置条件 S4（「机械检出不可达前提」能力未演示，三条标注均人工静态）。架构 = 轻量（`.investigations/000-架构设计/架构计划-260918-07-B63-S4能力演示.md`，用户批准；比对器留临时区 + v1 备份 sha256 归档）。
- ✅ **实现**：v2 比对器 `.tmp/260918-07/b63_comparator_v2.py`——17 域级 `{key,expected,check}` 前提集（PRECONDS）+ 事实封闭集（6 键）静态求值器（log 行为化自证 + `--facts` env 覆盖）+ suspended 语义 + SUSPENDED-TOUCHED 触达矛盾 FAIL + UNKNOWN-FACT-KEY FAIL（#182）。
- ✅ **四测**（预登记，全过）：POS rc=0（触达 8 ⊆ 声明 17，v1 无回归）/ NEG-A rc=1（生产 mask 注入 × legacy 日志 → SUSPENDED-TOUCHED 7 键 = 矛盾检出）/ NEG-B rc=1（枚举外事实键）/ NEG-C rc=0（生产 mask × 干净日志 → suspended 16/17 + touched 0 = **「生产 mask 下不可达」人工标注被机械复现**，唯 sysprop 门域保留可达，对齐声明表 §一）。
- ✅ **judge（SHOULD）**：PASS-with-conditions（N1 paldump 域补 `light.rust.armed` 前提 / N2 启动期断言 PRECONDS 键 ⊆ FACT_KEYS）——均已应用，四测复跑无回归。**confirmed（2026-09-18 用户授予，同块内）**。
- 📌 **边界**：声明表三条人工标注中 ①mixin:644（状态机不可达）②bin-diag（结构性不可达）不在求值器域内（非门控关闭形态）；vanilla-min.log 为合成载体只证能力；事实封闭集仅 light 面，扩面先扩集。judge 备忘：domain 臂 DECLARED 缺 `wgLightDomainWriteBack` 三键 = v1/S2 继承盲区，随第二接管面扩面。
- 📌 **知识库**：workflow-patterns **#183**（subagent 草稿 + 主会话应用）；docs 07 篇零更新（能力演示无新管线结论）。
- **证据指针**：`.investigations/b63-260918-07/{record-260918-07.md, judge-review-260918-07.md}`；载体 `.tmp/260918-07/`（v2 比对器 + 夹具 + v1 备份，不入库）。

## 260918-10 块（实际 2026-09-18 21:04 起，Get-Date 锚；R9-b light reader × bulk writer SENTINEL 覆盖判定——三候选收敛「无正确性缺口」）🔍 candidate（judge PASS-with-conditions 条件已应用；confirmed 留用户；Degraded 全静态）

> 承接 260913-02/03 R9-b（单写者不变量 + sentinel 落地，均 confirmed）遗留的**读者侧**判定缺口。架构 = `.investigations/000-架构设计/架构计划-260918-10-R9b-light-reader-sentinel.md`（用户批准）。过程产物 `.investigations/r9b-light-260918-10/`（record + scout-map + .b1/.b2/.b3 + cmd-output）。
> 本块开场 hook：260918-09 定性 + 知识库 #186 confirmed 授予并回写（用户 2026-09-18 21:2x 拍板）。

- ✅ **scout 勘探**：读者清单 R1-R9（裸引用/门/获取途径逐站点分类）+ 时序交叉 W1-W6 + OQ-1~OQ-6 开放问题面（scout-map.md）；mcsrc 双树版本疑点登记（scout §5，后由 worker 消解）。关键负发现：无「Rust 光照线程经 JNI 直读 Java sections」形态——Rust light 是纯函数内核。
- ✅ **fan-out 三候选**（判定树 ≥2 互斥候选，强制触发）：.b1「已覆盖说」（volatile status 门独立构成 HB——成立）/ .b2「真缺口说」（不 join future = 无 HB——**不成立，自证伪**）/ .b3「vanilla 同构既有说」（义务承载三要素原样保留——成立，附限缩：R1/R2 为新增站点，同构义务论证不独立于 b1）。
- ❌→✅ **b2 自证伪（本块关键错误链，教训已沉淀 → workflow-patterns #187）**：b2 前提「status 普通写且无同步边」被一轮一手源实读直接证伪——读者门所读 `ProtoChunk.status` 是 **volatile** 字段（ProtoChunk.java:42，双树逐字一致）；b2 按诚实规则如实改判「HB 闭合」，并给出加固选项清单（读者登记 / future join / volatile 化——最后一项实为无操作）。**判错方法**：见「轮询门」先 grep 门所读字段声明（volatile/CAS/普通），再谈 HB 有无；**「检测面盲区 ≠ 正确性缺口」分开裁决**。
- ✅ **T2 收敛裁决**：① R1/R2 × bulk 写回无 data race（volatile 门 HB，传递边由 ChunkStatus.java:361 volatile 守卫读 / CF 依赖链双重承载）；② OQ-5 闭合（无半构造窗口，门后无并发写者）；③ SENTINEL 读者不登记 = 检测面盲区非正确性缺口（OQ-4）；④ OQ-2 结构性闭合（vanilla 读者更弱——连门都没有，ChunkLightProvider.java:72-77）。全文 → record.md §1。
- ✅ **mcsrc 版本疑点消解**：双通道（sha256 关键 7 文件 **7/7 一致** + 版本特征符 =1.20.x 且 <1.20.2）确证引用树 = 1.20.1（.b3 §0；复算留痕 `cmd-output/mcsrc-sha256.txt`，judge SHOULD-3）。
- ⚠️ **附带勘误（§15.4）**：07 篇「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 源冲突（ChunkStatus.java:158 已存在，judge 独立核实）——取代注已插原句后（原句不改）；根因 = 「1.21.6 拆站形态」与「状态存在性」混淆，教训 = 「新增」半句落笔前 MUST 对旧版本树做存在性 grep。
- ✅ **judge（MUST，收尾三源）**：PASS-with-conditions（0 MUST / 4 SHOULD / 3 INFO）——SHOULD-1（record HB 传递边明示）/ SHOULD-2（「无需 Degraded」措辞与分层声明矛盾修正）/ SHOULD-3（sha256 复算留痕）/ SHOULD-4（#143 全称否定残留边界句）均已应用；核心风险点（volatile release/acquire 反序窗口）judge 独立推演确认不存在。
- 📌 **决策点（用户拍板）**：OQ-4 scope——**sentinel 读者登记不实施**（检测面增强属 SHOULD 级可选项，非义务修复；实施需评估光照热路径开销）。OQ-6（INPLAY 期）维持超范围。
- 🔍 **诚实边界**：Degraded 全静态（无 behavior 证据）；R4/R5 TicketManager 装配细节未实读；候选收敛的 confirmed 待用户。
- 📌 **知识库**：workflow-patterns **#187**（subagent 草稿 + 主会话应用，judge 传递边要点已并入判据 1）；07 篇勘误取代注 + R9-b 追加小节 + 10 篇本条目；INDEX 同步。草稿：`.investigations/r9b-light-260918-10/knowledge-draft-260918-10.md`。

## 260918-11 块（实际 2026-09-18 22:02-22:13，判据时序锚 0e8083c @22:02:35）：legacy run3 收敛性补证（扩 n）——lr3a VOID（E1 gzip 门缺陷）→ 修门 → lr3b 新 seed 全绿 → 「legacy 干净链收敛」n=1→n=2 🔍 candidate（judge PASS-with-conditions 已应用；confirmed 留用户）

> 承接 260917-04 §2（§15.4 取代旧「legacy 不收敛 200-changed」结论后新立「legacy 干净链 run2→run3 收敛（=0）」，当时 n=1 单 seed candidate）的扩 n 补证。判据预登记 `.investigations/legacy-run3-260918-11/criteria-260918-11.md`（时序锚 git 0e8083c @2026-09-18 22:02:35，先于 lr3a/lr3b 采集，#112）；执行体 dll 6F7FA3AE…2337（与 260917-04 同一执行体跨块复用，#77 mtime 复核在案）；新 seed 8820042113345982107（区别于 n=1 的 8576294172403134396）。过程产物 `.investigations/legacy-run3-260918-11/`（criteria + errors E1 + judge review + cmd-output/ lr3a·lr3b 双臂留档）+ `.artifacts/legacy-run3-260918-11/verdict-260918-11.md`（candidate）+ index.yaml。

- ❌→✅ **lr3a 整臂 VOID（本块唯一错误链，E1 五段式见 errors-260918-11.md）**：测量面全绿（changed=0 / warmup=158）但 world 身份门 `seed_in_leveldat=false` 触发 VOID（rc=2）。**根因（机制）**：`level.dat` 是 gzip(NBT)，脚本直读压缩字节——BE int64 seed 模式在压缩流中结构性不存在，门恒 miss 是**载体读取层缺陷**而非 seed 错位（解压后 BE 在 offset 305 精确命中目标 seed，#36 判据本身正确）。**修复**：`gzip.open`；按 #118 不挑臂 / #144 失败轮留档 / #146 禁自动回退——lr3a 的 changed=0/warmup=158 **不计入结论**，换标签 lr3b 重采。**教训**：自证门的「载体读取方式」与「判据语义」是两层，判据对 ≠ 门能命中；新门首用前先对已知正样本做一次 sanity（本例一轮免掉整臂 15 min 重采）；judge INFO 补记：该门为包含性子串匹配非结构化 NBT 定位，严格度低于判据表述，照抄该门形态时勿高估。
- ✅ **lr3b 主判据 SATISFIED（新 seed 扩 n）**：干净链 run2→run3 **changed = 0/2025**（changed_set=[] 空集证据，非计数推断，#162；主会话独立重数三快照 2025×3 一致），与 n=1（260917-04 C1R1-L，=0）跨 seed 复现；**warmup_1to2 = 153**（历史 ~128 同族）正证检测器活性——「假 0 / 检测窗 / settle 时序」解释（判据 §4 分叉候选②）被旁证削弱；SELFCERT 全绿 ×3 boot（Done 3/3 / lightInit ok 3/3 / dll sha 6f7fa3ae 3/3 / domain_hook=0 / fallback=0），judge 侧对 lr3b.log 独立 grep 复核逐项一致。**world 身份三项闭合（#161 判据首次全链落地）**：① rmtree epoch 后 region 新鲜度（min_mtime 1789740845.58 > epoch 1789740532.67，+313s 物理自洽）② level.dat seed BE int64 命中 ③ common=2025 ≥1000——正是 260917-04 b1「盘上 world 被跨臂覆盖」教训的机械化，本轮若 G17c-L3 时代有此门即不会产生 200-changed 混杂。
- ✅ **结论升级（§15.4 追加补证指针，原正文不删不改）**：verdict-260917-04 §2「legacy 干净链 run2→run3 收敛（=0）」由 **n=1 单 seed → n=2 双 seed 复现 candidate**（8576294172403134396 + 8820042113345982107，各一条干净链，符合判据 §3 期望 n=2）；追加指针已落 verdict-260917-04 §6。**confirmed 留人类拍板**。
- ✅ **judge（MUST，收尾）：PASS-with-conditions，条件已应用**——MUST-1 补 `.artifacts/legacy-run3-260918-11/index.yaml`（已补）；MUST-2 git 面核对（0e8083c 存在且先于采集 + 本块 src 零改动，主会话已核）；SHOULD-1 rmtree epoch 补录 lr3b_result.json 可复算 / SHOULD-2 n=2 指针落 verdict-260917-04 §6（均已应用）；INFO-1 门严格度声明（已并入 E1 教训段）。
- 🔍 **边界（§9.7 如实声明）**：载体 = snap_light region 快照（与 260917-04 同工具谱系，E1 等价档位，共享观测 key 集 S = 两轮 common 快照面各 2025）；覆盖面 = 单 seed 单 spawn 区域 × Done+60s × 3 boot；**与存档写入口径 / 域批形态 / vanilla 均不可比**，不得外推至「legacy 光照整体收敛」或域臂；域臂对照不采（C-1(R1) 已 confirmed，判据 §3 明示不重复）。
- 📌 **知识库**：build-tooling **#151 追加实例行**（260918-11 E1，gzip 载体形态 sanity，subagent 草稿 + 主会话应用，不新增条目）；错误台账 = errors-260918-11.md（独立成篇 + 速查表）；10 篇本条目；INDEX 无新增条目不改。

## 260919-01（2026-09-19）A1 正式判别补通道——域批路径计时行后重采 🔍→✅（判别完成，candidate 待用户拍板）

> 承接 260917-04 §3 降级声明（A1 正式判别通道缺失：「须新增域批路径计时行后重采」）的执行块。架构计划轻量档 `.investigations/000-架构设计/架构计划-260919-01-A1正式判别补通道.md`（用户批准）；判据预登记 `.investigations/a1-260919-01/criteria-260919-01.md`（时序锚 git 6d79bc8 @2026-09-19 13:12:14，**先于打点实现与采集**，#112）。打点 = 纯 Java 观测（`wgLightDomainTaskRun` 三段分解 fill/native/wb，整块 LIGHT_TIMING 门控，#136），无 Rust/dll 改动（sha 6f7fa3ae 全程不变）；判据新增「成功 task 行必带 fillMs=」自证硬门（缺 → 臂 VOID，#160）+ #161 world 身份门（gzip level.dat，260918-11 台移植）+ #156 全使能开关口径。

- ❌ **C6R-L（legacy 臂首跑）宿主崩溃失败轮**：DSH 宿主崩溃带走整个进程树（驱动+gradle+server，log 截断于 13:17:43）——按 #144/#146 换标签留档（cmd-output/C6R-L.log 保留，禁自动回退），复跑 C6R-L2 全绿。
- ✅ **双臂重采 SELFCERT 全绿**：legacy 臂 LIGHTPROBE-T 18 行 / domain 臂 457 task 行全带 fillMs=（fillms 门 457/457，正自证打点真生效 #81）；fallback=0、degraded=0、dup=0、timedOut=115（与 260917-04 前例同量级，宽限期边界任务归因）；world 身份门两臂全过（seed_in_leveldat + region 新鲜度）。
- ✅ **A1 正式判定（判据 §2 三分支，读法写死）**：n=457，占比 P50=**0.9835** / mean=0.9774 同侧 ≥30% → 命中「≥30%」分支 = **A1（原生光照延迟优化）有改善面**；分解 P50：fill=0.302 / native=4.264 / wb=0.079 ms per-center——native JNI 调用全程占 ~92% 主导。主会话独立重数（独立实现 recount_a1.py）与驱动提取逐项一致，恒等式 avgMs≈三段和全 457 行成立（maxdev 0.08ms）。
- ❌→✅ **judge M1 更正（worker 复算错误，错误优先）**：worker 原稿「行 2160 恒等式偏差 7.99ms idk」系三段和误取 19.239（漏加 wb=18.501）——judge 一手实测该行三段和=27.239、dev=0.006ms，恒等式成立，idk 前提不成立注销；该行真实占比 0.32079 = 全分布 min。教训：**抽样复算的算术本身也要被复核**（worker 抽样 ≠ 免检；judge 独立复算抓出）。
- 🔍 **量级对照（E1 同载体跨轮）**：本轮 9-center 折算 ≈0.52 ms/chunk vs C6-260917-04 折算 ~0.98 ms/chunk，同量级无矛盾，预置 fan-out 未触发；域批 ≪ legacy 6.26 ms/chunk 方向两轮一致。
- ✅ **judge（MUST 收尾 + SHOULD candidate）：PASS-with-conditions，条件已应用**——M1 更正（更正记录形态落 verdict §3/§6）+ S1 判据 §3.1/§3.4 hook armed 读法澄清附注 + S2 驱动 fallback 硬门严于判据知悉附注 + S3 执行体三元组补核落盘（executor-triple-260919-01.txt：target dll = 资源 dll = in-log sha 三点一致 + mixin class 新编译含 fillMs；dev-run 口径下 build/libs jar 为红鲱鱼 #96）。verdict 升 **candidate**，confirmed 留用户。
- 📌 **知识库**：workflow-patterns 新增**发现 #188**（观测打点/判别数据通道必须覆盖全部执行形态——「打点在位 ≠ 判据可达」，生产侧前移版 #162；subagent 草稿 + 主会话应用）；10 篇本条目；错误记录 = M1 更正 + C6R-L 崩溃失败轮（本条目内，不独立成篇——单块单错误链未达独立台账门槛）。
- 📌 过程产物：`.investigations/a1-260919-01/`（criteria + cmd-output/ 两臂日志+result.json+失败轮留档+三元组补核 + judge-review + 知识草稿）、`.artifacts/a1-260919-01/verdict-260919-01.md`（candidate）+ root index 条目；打点 diff = ServerLightingProviderMixin.java（未提交，随本块收尾提交）。
- ✅ **confirmed 授予（2026-09-19 用户拍板，同日回写）**：verdict-260919-01（A1 判别 = 有显著改善面，native JNI 主导）升 confirmed；回写 = verdict front-matter/状态机行 + root index.yaml 条目 + knowledge INDEX #188 行 + 本条目。A1 是否立项实施 = 独立决策项，留待用户后续拍板（改善面上限口径，见 verdict §4 含义段）。

## 260919-02（2026-09-19）A1 native 段分解裁断（WG_LIGHTPHASE 门控打点，四臂 R2 重采）——.b1 算法量级主 + .b2 JNI 边界次 + .b3 测量轴排除 🔍 candidate（judge PASS-with-conditions M1/S1-S3 已应用；confirmed 留用户）

> 承接 verdict-260919-01（confirmed：native ~92% 主导）的下游分解。判据预登记 + 打点随 commit
> 32e126b @14:38 引入（时序锚链 32e126b→080d6e7→aab8b30，均早于全部采集 log 15:05–15:22）；执行体
> dll 修正版 **f1d30ee9**（第一版 60abb61b 缺陷构建数据留档不进判定）；seed 8576294172403134396；
> n=457 task/臂 × 四臂（off/on1/on2/on3）。正式裁决 → `.artifacts/a1-feas-260919-02/verdict-260919-02.md`。

- ❌→✅ **E1 打点字段误标（本块关键错误链，五段式见知识草稿 §1）**：第一版 `kernel_us` 打印的是
  dp-t1（outv 分配段 ~86μs）而非内核墙钟——数据自暴露（kernel_us ≪ Σphases 23ms，子段>母段物理
  不自洽）一轮抓出；修正计时窗 → 重编 60abb61b→f1d30ee9 → R2 四臂重采，P1-\* 四臂数据按 #144/#146
  留档不进判定。→ workflow-patterns **#189**（窗标签与计时代码一致性 + 自洽 sanity，#134 家族）。
- ❌→✅ **E2 并发 stderr 交错残缺行（第二错误链）**：`[LIGHTPHASE]` 行被 FP-FILL/FP-LIGHT 从中间切入
  产生缺 key 残缺行（on2 19/457=4.16%）→ 提取脚本 KeyError 崩溃；判据追补 §3.7 残缺行条款（全 key
  校验 + ≤5% 报告 / >5% VOID 双档），commit 080d6e7 时序实证先于采集；on2 贴线通过 + idk-4 复跑预案。
  → workflow-patterns **#190**。
- ✅ **三分支判定（P50 主读法，R2-on1 主读数）**：**.b1 算法量级满足（主）**——算法四相占 kernel
  77.7%（≥70%）、kernel/native 核算 75.9%（≥60%），kernel=27.04ms/任务、fill 13.51ms 单项最大
  （37.9%，9 中心全量域重算冗余主导）；**.b2 JNI 边界/搬移分配满足（次）**——非内核段 7.16ms
  （alloc_copyin 4.65 + subcopy 2.43 + copyout 0.07），拷入隐含带宽 2.11GB/s（<2.5，含分配/缺页/pin
  份额非纯 memcpy）；**.b3 测量/调度轴未满足**（trim 差 −7.68% ≤10%，低尾 timedOut 拉高全量均值）。
  三臂方向一致不翻转；未解释缺口出口不触发。
- ✅ **总量分解闭合（如实）**：计时三段+subcopy vs nativeMs×9 缺口 1.46–2.20ms（4.08–5.59%），归因
  Scratch::new+b9 分配不落分段 + 窗间缝隙；kernel 窗内未归因 3281–3904μs（outv ~90μs 在内，余未逐项
  → idk-2）。压缩面清单 C-1~C-5（收益上界口径 #124）：.b2 系（C-3 packed 直传 + C-4 缓冲复用）合计
  ~25% 低风险可先行；.b1 系（C-1 域共享增量化）上界最大（≤59%）但 MUST 先解 G3 一次重载收敛性约束。
- ✅ **judge（MUST 收尾）：PASS-with-conditions，条件已应用**——M1 index.yaml 登记补齐；S1 off 臂 sha
  改直证引用（off log 自含 f1d30ee9，证据强于原表述）；S2 窗内未归因补三臂量级带；S3 idk-3 三元组
  补核留待发布链。三源核对全过（判据时序锚实证 + judge 独立 python 重算逐项吻合 + 快照一致性）。
- 📌 **知识库**：workflow-patterns **#189/#190**（subagent 草稿 `.investigations/a1-feas-260919-02/
  knowledge-draft-260919-02.md` + 主会话应用）；错误台账随草稿 §1（E1/E2 五段式 + 速查表）；10 篇本
  条目；INDEX 尾注行同批落盘。分解数值本身按价值门不进 discovered（verdict 在案）。
- 📌 过程产物：`.investigations/a1-feas-260919-02/`（criteria + scout-map + record + judge-review +
  cmd-output/ P1-\* 缺陷对照留档 + R2-\* 四臂）+ `.artifacts/a1-feas-260919-02/verdict-260919-02.md`
  （candidate）+ root index 条目。

## 260919-03（2026-09-19）A1 .b2 系验收（C-3 packed 域批直传 + C-4 缓冲复用，四臂 off/on25/onpk/onpk2）——B-主未满足（净退步 ~17.7%）+ B-C3 满足 + B-C4 灰区 🔍 candidate（judge PASS-with-conditions S1-S3 已应用；confirmed 留用户）

> 承接 verdict-260919-02（candidate：.b2 系 ~25% 低优先先行）的验收块。时序链（#112）：a376b4c 判据+脚本预登记
> → ebcb393 judge SHOULD 审查（review-criteria-260919-03.md，M1/S1/S3 应用）→ 0615298 实现（C-4 thread_local +
> C-3 packed 域批 ABI）+ 新 dll **7519ddb8**（位等价单测 4/4 绿含新增 light_decode_packed_domain_roundtrip；
> Java compileJava 绿；check_switch_mapping rc=0）→ df2849d verdict+review+record 收尾批。seed 8576294172403134396；
> n=457 task/臂。正式裁决 → `.artifacts/a1-b2-260919-03/verdict-260919-03.md`。

- ✅ **实现要点（C-3/C-4）**：C-3 = 提交线程 packed 帧收集（任一 chunk 失败→内联路回退，
  WG_DOMAIN_INLINE 计数 = M1 门数据面）+ 新 JNI `lightComputeDomainPacked` + Rust 泛化解码
  `light_decode_packed_domain`（帧式 chunk 基址、后帧覆盖重叠 = 现 arraycopy 序语义；实现自检捕获并修正
  「全量清零抹先前帧」缺陷一次）；旧 `light_decode_packed` 收敛为包装防双路漂移；task 行 `packed=<n>` 负自证。
  C-4 = bridge b25/outv thread_local 复用（照 jni_bridge.rs:289-311 先例，无条件无开关）。
- ❌→✅ **onpk 臂 VOID → 换标签复采 onpk2**：onpk 残缺率 26/457=5.69% > 5%（#190 写死，非零退出）→
  留档不覆盖、换标签复采 onpk2（malformed 1.09%，coverage 0.9808 全绿）为判据臂（#144/#146）。
- ❌→✅ **E1 采集脚本 M1 门方向写反（本块关键错误链，五段式）**：判据「回退占比 >30% ⇒ VOID」被写成
  「coverage(=1−占比) > 0.30 ⇒ VOID」——同一数字两义（占比 vs 补数），方向未取反，优秀值 0.9808 反被判
  VOID；定位 = 门名与值并读不自洽；修复 = coverage<0.70 VOID / 0.70–0.80 灰区 + 注释。
  → workflow-patterns **#192**（占比门入脚本 MUST 注释罚哪一侧 + 已知优秀值负向测试先行）。
- ✅ **四臂判定（P50 主读法，判据读法写死）**：**B-主未满足**——nativeMs P50 4.012/3.410 = **1.177**（>0.95，
  packed 路端到端净退步 ~17.7%）；**B-C3 满足**——alloc_copyin 169/1091 = **0.155**（拷入降 ~6.5×；judge 注：
  packed 三数组 vs blocks25 宽 ABI 非同量工作，按 C-3 效应代理读）；**B-C4 灰区**——3.370/3.410 = **0.988**
  （0.97–1.00 带内，双值上报；idk-1：Scratch/b9 不在范围，归因受限）。行为等价门全绿（off 0.9972 /
  on25 1.0010 / onpk2 1.0015，均 ≥0.95）；位等价门绿。强制 fan-out 出口未触发（B-C3 满足）。
- ✅ **机制归因（证据绑定）**：拷入节省 0.92ms 被 kernel 窗解码成本 +6.75ms 反超（净 +5.8ms/task 与端到端
  退步量级自洽）；窗位代码实证 `jni_bridge.rs:575-588`（`light_decode_packed_domain` 落 kernel 计时窗内）；
  VOID 臂 kernel=34633 与 onpk2 同向（仅参考）。→ workflow-patterns **#191**（压缩/表示变换类优化必须双侧
  列账：消除侧节省 − 新增侧成本 = 净上界；「净退步」限定口径窗——idk-2：nativeMs 不含 Java 提交线程
  packed 收集成本，总管线需 tick/wall 口径另裁）。
- ✅ **judge（收尾 MUST）：PASS-with-conditions，S1-S3 已应用**——S1 coverage 读法 = M1 补数、方向为过；
  S2/S3 应用后 verdict candidate。三源核对过（数值独立重算零偏差）；A1-RAW 值差异实为两种 P50 取位法之差
  （驱动 stdout 单点右中位 vs 数组插值 median），非跨臂漂移（#90 不适用，record 已澄清）。
- 🔍 **结论建议（待用户拍板）**：① C-3 建议不采纳/回退缺省关（机制归因闭合；翻案须先解 idk-3
  解码/内核拆分打点 + 评估批量 palette 直映射，属新立项）；② C-4 建议保留（灰区非未满足，代码已并线无开关
  成本，E3 参照对旧构建 −14%）；③ 下一步回 .b1 系 C-1（上界 ≤59%，MUST 先解 G3 收敛性约束设计）；
  C-2 受 round4 纯算力冻结排序约束。idk-1~5 如实登记。
- 📌 **知识库**：workflow-patterns **#191/#192**（subagent 草稿，本块）；10 篇本条目；INDEX 尾注行同批落盘。
  §9.8 副作用登记：verdict/重算脚本（.tmp）均 derived；onpk VOID 臂换标签留档未覆盖。
- 📌 过程产物：`.investigations/a1-b2-260919-03/`（criteria + plan + record + review-criteria + judge-review +
  cmd-output/b3-\* 四臂）+ `.artifacts/a1-b2-260919-03/verdict-260919-03.md`（candidate）。
