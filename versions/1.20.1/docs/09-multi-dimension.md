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
