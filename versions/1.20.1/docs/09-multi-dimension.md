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
