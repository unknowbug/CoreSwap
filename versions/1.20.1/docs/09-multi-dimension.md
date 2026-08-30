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

## 七、Rust 世界参数化（2026-08-29，对齐 C++ wg_create 多世界方向）

\WorldgenHandle::create(seed, wg_dir)\（overworld 便捷入口）拆出通用入口 \create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)\——支持任意维度加载，对齐 C++ \wg_create\ 多世界方向。

**参数化维度（非硬编码 overworld）**：
- \settings_name\：\
oise_settings/<settings_name>.json\（overworld / nether / end / mod 维度文件名）
- **dfNs** = settings_name 去 ".json"：决定 \density_function/<dfNs>/\ 目录 + \esolve_ref\ 命名空间前缀 \minecraft:<df_ns>/\（\DensityBuilder.set_df_ns\；修复 M1——惰性加载前缀原硬编码 \minecraft:overworld/\）
- **维度参数从 settings 读**：\min_y\ / \
oise.height\ / \sea_level\ / \quifers_enabled\（非硬编码 overworld 的 -64/384/63/true）
- \iome_params_file\：维度 biome 参数（overworld \iome_params.json\ / nether \iome_params_nether.json\ / mod 自定义）
- \world_height\：世界高度（overworld 384 / nether 256 / mod 按定义；0 = 从 noise.height 兜底，对齐 C++ \worldHeight>0?worldHeight:noiseHeight\）

**surface_rule 数据驱动（\SurfaceBuilder::parse_surface_rule\）**：
- overworld：保留已验证的代码规则（\uild_overworld_rule\）
- 非 overworld：用 \settings.surface_rule\ JSON 数据驱动（支持 sequence / condition / block + 各 cond：not / biome / y_above / stone_depth / noise_threshold / hole / steep / water / temperature / surface）——mod 维度无需改代码
- 对齐 C++ 方向：surface_rule 从 JSON 尾部读（数据驱动），主世界保留代码规则

**aquifers_enabled=false（下界）→ VanillaAquifer.enabled=false**：
- \classify\ 跳过真实 aquifer（无 water/lava），返回 Air（修复 M2——加 \nabled\ 字段破坏全部 struct-literal 构造点，用 \VanillaAquifer::new(aq)\ 收口）

**验证结果**：
- nether：\create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256)\ 加载成功（min_y=0 / height=256）+ 生成 chunk(0,0) 56307 非空气块
- overworld 回归：\eatures_probe\ match 95.40% 不变
- mod 维度（如暮色森林）：数据文件放 wgDir 对应路径（\
oise_settings/<mod_dim>.json\ + \density_function/<mod_dim>/*.json\ + biome params），settings_name 指向即可加载

**数据驱动边界更新**：详见 \WorldgenRust/data-driven-boundary.md\「多世界参数化」章节。AGENTS.md「数据驱动架构铁律」含本多世界方向（用户拍板）。

**错误台账**：\.investigations/multiworld-port/multiworld-errors.md\（M1 前缀硬编码 / M2 字段连锁）。

---

## 八、Rust 多世界落地（2026-08-30 Phase A/B/C，commit 1102f58 + 9a3f7fa）

> status: **candidate**。Phase A = Rust 引擎 nether 块级验证 + 两修复；Phase B/C = MOD 游戏内接线。
> 错误台账：`.investigations/multiworld-port/multiworld-errors.md`（M1-M5，含速查表）。

### Phase A：nether 块级验证 + 双高度/确定性两修复

**探针**：`WorldgenRust/src/bin/multiworld_nether_blocks.rs`（fill_chunk_blocks vs vanilla nether 参照 WGB2 4×4@0,0 h256）。

**修复 1（双高度，M3）**：`worldgen_handle.rs` 存 `noise_height`（settings noise.height，nether 128）≠ world height（256）；`terrain.rs fill_chunk` 加 noise_height 参数（y≥noise_top 留 Air）；宏观采样器网格只铺噪声高度；13 个探针 bin 调用点同步补参。
- **nether match 23.77% → 73.77% → 74.04%**（y≥128 四带 0% → 100%），**超 C++ 时代 71.97%**；overworld 基线 95.40% 零回归。

**修复 2（确定性，M4）**：`biome.rs` BiomeClassifier features/carvers HashMap → BTreeMap——原 `all_features_lists()` 每进程随机迭代序 → PlacedFeatureIndexer 编号随机 → nether features 放置运行间漂移 2796 块。修后两次运行逐位一致。

**遗留差距（记录不修）**：熔岩海带 y=32..63（7.9%，流体填充缺失——C++ 时代也未解）；底部基岩错位（VerticalGradient 反锚序，C++ 有修复未移植）。证据：`.investigations/multiworld-port/cmd-output/nether_blocks_match_v{1,2_noiseheight}.txt`。

### Phase B/C：MOD 游戏内接线

- **JNI 层**：`jni_bridge.rs` initDim（5 参 wg_create 映射）；`CppWorldgen.java` initDim 声明。
- **CppBridge**：netherHandle + initNether + fillChunkNether（16×16×256 buffer）+ feedBeardifier 泛化 handle 参数 + writeChunk 泛化维度高度。
- **Mixin 按维度分派**：下界拦截分支（min_y=0/h=256 且 netherActive）+ **末地保护**（End 与下界同形状，靠 biomeSource 反射区分——@Shadow 够不到父类字段是坑，用缓存反射）+ buildSurface 从全局 cancel 收紧为**按维度**（修掉「末地表层会被误 skip」的现存隐患）。
- **构建接线（M5）**：`build.gradle processResources inputs.file dll`（根因修复：UP-TO-DATE 跳过 doFirst 同步 → resources 里旧 dll → initDim UnsatisfiedLinkError）。
- **实证**：`initNether enabled=true` + `[Mixin] populateNoise(nether) intercepted chunk(-1,-1)`（rust_nether_test4.log，摘录 `.investigations/multiworld-port/cmd-output/nether_ingame_intercept_20260830.txt`）。

### 遗留课题
- lava 流体填充（Rust 与 C++ 同未解，Phase A 遗留差距同源）；
- 底部基岩 VerticalGradient 反锚序移植；
- **末地引擎未启动**（Mixin 保护已就位，Rust/C++ 末地生成都未做）。

### 补遗（2026-08-30 深夜）：JSON 布尔读取修复 + 熔岩机制 + 数字更新

> 错误台账 M6/M7：`.investigations/multiworld-port/multiworld-errors.md`（含速查表）；熔岩机制源码证据：`.investigations/multiworld-port/analysis-nether-lava-mechanism.md`。

**修复 3（JSON 布尔，M6）**：`nether.json` 的 `"aquifers_enabled": false` 是 JSON 布尔，Rust 读取走 `as_f64()`（自研 json.rs 只匹配 Number，Bool 恒 None）→ `unwrap_or(true)` 默认值静默生效 → 下界被错误启用真实含水层（6.7 万块水 vs vanilla air）。同款坑连带 `legacy_random_source`（legacy 分流从未激活）与 feature.rs `requires_block_below`。修复：json.rs 加 `as_bool()`（Bool 直读 + Number 兼容 !=0），三处读取改 as_bool。

**修复 4（熔岩机制，M7，源码确认）**：vanilla 下界熔岩 = `aquifers_enabled=false` 时 `ChunkNoiseSampler` 走 `AquiferSampler.seaLevel()` 匿名实现——density≤0 → `y < sea_level ? lava : air`（严格 <，无噪声参与）；buildSurface 跳过流体格（SurfaceBuilder L136 只记录液面）。docs/09 旧猜测「来自 fillFromNoise」「buildSurface 跳过流体格」均证实。Rust：VanillaAquifer 加 settings 数据驱动的 sea_level，`!enabled` 分支同语义实现。

**数字更新（本节此前数字作废，以本条为准）**：
- nether match **82.69%**（M6/M7 修复前 74.04%）：y≥128 **100%** / y0..31 **79.6%** / y32..63 **65.8%**（修复前 7.9%）/ y64..95 55.2% / y96..127 61.0%。
- overworld **95.40% 零回归**；两次运行**逐位一致**（确定性保持）。

**遗留课题更新**（第八节原「遗留课题」之上追加）：
- soul_sand_valley 表面残差（y=1..2）；
- legacy 分流激活验证（as_bool 修复后读取已通，块级输出仍未变）；
- Hole 语义不一致（Rust `surface_depth <= 0` vs Java `stoneDepthAbove <= 0`，C++ L251 才对——Rust 注释声称对齐 Java 是错的，影响 nether lake/not(hole) 门控，单开课题）；
- 末地引擎未启动（同前）。
