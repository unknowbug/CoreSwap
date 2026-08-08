# 09 · 多维度通用引擎（数据驱动任意维度）

> **文档定位**：多维度引擎 = 用户初始目标（**通用引擎，不是白名单 vanilla 三界**）。本文记录通用架构、下界引擎（第一个非主世界验证实例）、多维度规划。
> **主世界 1.20.1 的排查记录**（负坐标/8576/崩溃/工具演进）不属本主题：结论已提炼到 01-08 各篇末尾「2026-08-08 已验证结论」章节；完整时间线在 [10-timewise-archive.md](10-timewise-archive.md)（只读归档，勿当活跃文档）。

---

## 一、定位：通用引擎（数据驱动任意维度）

1.18+ 所有维度（主世界/下界/末地/暮色等）共用同一套底层：`ChunkNoiseSampler` + `DensityFunction` 树 + noise_settings JSON，**差异只是数据**。CoreSwap 的 C++ 是「密度求值引擎」（JSON 解析 + 树求值），主世界只是第一个应用实例。

**通用结构（wg_create 纯数据驱动）**——不再有「维度」概念：

```cpp
wg_create(seed, dataDir, settingsName, biomeParamsFile, worldHeight);
// 引擎从 noise_settings/<settingsName>.json 读：minY / noiseHeight / aquifersEnabled
// settingsName 决定 density namespace/目录（"overworld.json"→overworld；mod 维度传自己的设置名）
// biomeParamsFile：biome_params.json / biome_params_nether.json（Java BiomeParamProbe 导出的参数表）
// worldHeight：Java 侧传（维度定义的世界高度；overworld 384 / nether 256）
```

- mod 维度只要 Java 侧把数据准备好（noise_settings JSON + biome 参数 + 世界高度）传给 wg_create 就能生成
- 例外（少数不通用）：自定义 BiomeSource 类 / 完全自定义生成器类的 mod 走 vanilla
- **双高度**：世界高度（buffer/写入，nether 256）vs 噪声高度（采样，nether 128）——`fillOneChunk` y 循环用噪声高度，上方留 air
- **维度分支**：下界跳过 aquifer/oreVein（R 缺组件不 fail）；surface_rule 在 JSON 尾部（数据驱动）

## 二、下界引擎（第一个非主世界验证）

**状态**：TOTAL 71.97% / nonAir 75.41%（chunk 0-3，2026-08-07）——**主世界 100% 每次改动保持回归**。

**修复链**：

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

**下界差异分析（已排除 vs 未解）**：
- ❌ **b3d 彻底排除**：RouterProbe 反射 NoiseConfig.randomDeriver（游戏实际 deriver）+ 下界参数（0.25/0.375/80/60/8）采样 (0,y,0) 列 16 点——与 C++ 下界 b3d **逐位一致**（y24: -0.14815987141887240 vs -0.148160）
- ❌ **684.412f 精度排除**：模拟 `(double)(float)684.412` 后下界 final 完全无变化（已回滚）
- ❌ **maintainPrecision 排除**（后修复语义）：C++ `lfloor(v/3.35e7+0.5)` → Java `(long)(v/3.35e7)`（向零截断），下界 o 从 1.0 递减、e*o 最大 6159 不触发折叠（语义必须对，主世界 100% 保持）
- ❌ **DensityProbe 的 UnblendedNoisePos 路径不可靠**（坐实）：下界 final 数据不再作参照（per-call CellCache 插值结果，非直接采样）
- 🔍 **lava 差根源（未解，下一步）**：Java buildSurface **跳过流体格**（goto 跳过规则应用）——**lava 不是 surface 规则生成**，来自 fillFromNoise 的流体填充（下界 fluid_level 组件）——C++ 的 3b 阶段下界无 aquifer 时跳过了流体填充 → lava 差 25365 根源。**下一步：C++ fillOneChunk 下界分支补流体填充**（Java ChunkNoiseSampler 的 fluid 逻辑：fluidLevelFloodedness/fluidLevelSpread 组件）
- ❌ **runDepth 洞内重置**：回滚（主世界 100% 铁律；lava 的 hole 机制需先确认 MC 1.20.1 SurfaceBuilder 源码，不要猜）

## 三、多维度规划

1. **下界收尾**：流体填充（fluidLevelFloodedness/fluidLevelSpread）→ 72% → 目标 100%
2. **末地**：末地生成器结构差异大（无噪声路由器，自定义生成器类）——走 vanilla 或专项 C++ 化（未启动）
3. **mod 维度**：数据驱动路径已通（见定位节），需实际 mod 验证（未启动）
4. **多版本**（1.18/1.19）：迁移流程见 08 篇（diff 流程 + 版本敏感点）；**noise-in-Java 开关**（v1.2 规划）：`-Dcpp.noiseInJava=true` 时 C++ 的 InterpolatedNoiseDF 采样改走 JNI 调 Java（游戏侧 old_blended_noise 永远对），C++ 只算确定性管线——新版本适配先开开关跑通 → 验证正确性 → 逐个复刻 noise 回 C++ 拿性能（多版本迁移脚手架 + 兼容兜底）

## 四、多维度相关工具

- **DensityProbe.java**（Java）：`-PdensityProbe=true -PdensityProbeDimension=nether -PdensityProbeChunkX/CZ/X/Z` 导 vanilla finalDensity 剖面（`vanilla_density_<dim>_c<cx>_<cz>_b<bx>_<bz>.txt`，y 每 4）
  - **拿 finalDensity 的正确路径**：`cm.getNoiseConfig().getNoiseRouter().finalDensity()`（yarn）
  - ❌ 反射 cns 的 finalDensity 字段不存在；`initialDensityWithoutJaggedness` 下界是常量 0（无用）
- **got_export -densityDump cx cz bx bz**（C++）：同格式 dump（-dimension 1 切下界）
- **wg_sample_density / wg_sample_named / wg_sample_noise API**（直接采样 finalDensity / 注册 df / 原始噪声）
- **RouterProbe**：反射 NoiseConfig.randomDeriver（游戏实际 deriver）+ 下界 b3d 参数采样

## 五、多维度已知坑（勿重蹈）

- **runDepth 洞内重置会破坏主世界**（99.86%——2268 块差异）——主世界 100% 是铁律；lava 的 hole 机制先确认源码再动
- nether/base_3d_noise 参数**不在 NOISE_PARAMETERS 注册表**（是 old_blended_noise 内联）——noise_params.json 只含 38 个 minecraft:noise 型参数
- 下界 y_scale(0.375)/y_factor(60) ≠ 主世界(0.125/160)——old_blended_noise 分支默认值写主世界的，必须从 JSON 读
- 密度采样用 `UnblendedNoisePos` 直接调 `router.finalDensity().sample` 有效（RouterProbe 验证过）
- **参照导出铁律**：重新导出 vanilla 前必须删 `run/world/region/`（或删 world）——否则 `getChunk` 复用旧 chunk 缓存 → 参照含假差异（谜 A+B 真相）
- **level-seed 坑**：`java/run/server.properties` 的 level-seed 硬编码，`-PbenchSeed=X` 只设 Java 属性——跑其他 seed 必须改 level-seed
- **残留 java 进程**：gradle runServer 崩溃后可能残留（占 world/端口）——先 `Stop-Process -Name java`，仍失败删 `run/world` 再重跑

## 六、关联文档

- 01 架构映射（C++ 引擎结构）、08 版本迁移方法论（跨版本流程）
- 主世界 1.20.1 排查：01-08 各篇末尾「2026-08-08 已验证结论」+ [10-timewise-archive.md](10-timewise-archive.md)（完整时间线归档）
