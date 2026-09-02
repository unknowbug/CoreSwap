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
- **dfNs** = settings_name 去 ".json"：决定 \density_function/<dfNs>/\ 目录 + \
esolve_ref\ 命名空间前缀 \minecraft:<df_ns>/\（\DensityBuilder.set_df_ns\；修复 M1——惰性加载前缀原硬编码 \minecraft:overworld/\）
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
  - **[supersedes 260901-03]** 本行已过时（M6 修复前表述）：当前 Rust surface_rules.rs L101 Hole => stone_depth_above <= 0 与 Java 一致，dll M17（C5AC5309）含修复——Hole 语义课题闭合。依据见本文「B1 定论」节（§15.4 取代链，原行保留不删）。
- 末地引擎未启动（同前）。


## nether 存档写入口径 Full 化（1.0.22 dll，双 seed）（数据/口径/写盘无损 = confirmed；机制解释 = draft，260901-03）

> 载体：Rust nether 接管 gen（cppReplace）→ 存档落盘 → MCA 直解（compare_save_region.py）vs vanilla BlockProbe 参照（WGB2）+ ReadWorldProbe 内存读交叉验证。dll sha256=C5AC5309F3C59A044（1.0.22 M17）。
> **口径声明（v0.20 §9.7 三要素）**：① 载体 = MCA 存档直解 + 内存读，vs vanilla 参照；② 覆盖面 = 4×4 chunks @(3200,3208) 全高度（nether min_y=0，height=256，动态读取）；③ **与 96.44% 探针口径（docs/09 既有数字）不可比**——载体不同（存档/内存 vs 探针直采），数值禁止直接互比。seed 三查：server.properties ↔ level.dat ↔ ref header 全同值。
> 过程错误（首轮三场 run enabled=false 全作废、cppWorldgenDir 错层等）见 `.investigations/nether-save-full/nether-save-errors.md`（独立台账，不在此重复）。

### 双 seed 三口径数字表（candidate）

| seed | 内存读（ReadWorldProbe） | 存档读回（reconfirm，从盘读） | MCA 直解（compare_save_region） | 残差块数 |
|---|---|---|---|---|
| A = -2032795982907864146 | 99.9376%（1047922/1048576） | 99.9376%（**与内存精确同值**） | 99.9278%（1047819，差 103 = cave_air 簇） | 757（MCA）/ 655（内存） |
| B = 8576294172403134396 | 93.5156%（980582/1048576） | 93.5156%（精确同值） | 93.5156%（精确同值） | 67,994 |

- Rust 真实参与证明：v2 log `enabled=true` + 64 条 `populateNoise(nether) intercepted`（4×4 目标 + feature 蔓延邻域）。**验收判据：enabled 标志 + intercepted 覆盖目标 chunk，缺一 run 作废**（首轮教训，见错误台账 E1/E3）。
- seed B 残差全部落在 y≤127（layerMatch：y0..31=82% / 32..63=88% / 64..95=88% / 96..127=90%；**y≥128=100%**）——与「noise_height=128、y≥128 留 air」（09 篇 M3 教训）自洽；本次 run buildSurface 被 Mixin skip，存档表层全来自 Rust 生成，残差 = vanilla surface rule 输出 vs Rust 生成的差异。

### 残差机制分类占比（分类 = 数据直读，candidate；机制解释 = draft）

| seed | 类别 | 块数 | 占比 | 机制候选（draft） |
|---|---|---|---|---|
| A | A1 nether 矿石 feature 差（quartz/gold/debris，方向全为「vanilla 有矿→存档无矿」） | 640 | 84.5% | Rust nether ore feature 未放置或错位（与 B4 同家族；09 篇已知缺口「fill_chunk_blocks nether 逻辑差异化」的存档级量化） |
| A | A2 air↔cave_air 尾随簇（单 chunk(203,200) y70-72） | 104 | 13.7% | **未闭合**（见下） |
| A | A3/A4 magma 点差 / 熔岩湖边界点差 | 13 | 1.8% | seed B 大类的缩微版 |
| B | B1 basalt deltas 三大宗石互换（basalt↔blackstone↔netherrack，成片双向） | 52,078 | 76.6% | surface rule 条件链系统性偏差（biome 判定 / noise 阈值 / Hole 语义 `surface_depth<=0` vs `stoneDepthAbove<=0` 的下游表现之一，未验证）**[supersedes 260901-03]** 本行旧候选方向作废，定论见本文「B1 定论」节 |
| B | B2 soul sand valley 涂布边界 | 5,720 | 8.4% | 吻合 09 篇已知遗留（y=1..2 soul_sand_valley 表面残差），块数放大 |
| B | B4 矿石（与 A1 同家族） | 2,629 | 3.9% | 同 A1 |
| B | B5 magma / B3 熔岩湖边界 | 3,069 | 4.5% | magma：underwater_magma/邻接判定归属未定；湖边界：M7 seaLevel 机制已修、边界条件残差（已知遗留） |
| B | 未分类（top15 以下散点） | 4,498 | 6.6% | — |

### 未闭合待查项（全部 draft/待查）

1. **103 cave_air 簇机制**：v2 下 seed A 内存 = 存档读回**精确同值**（无 cave_air），MCA 直解却多 103 块 air→cave_air（同 chunk 同簇，y69=0/70=4/71=23/72=53）——「同一次落盘、两种读取口径不同」的新形态矛盾；Rust 全代码零写 cave_air，b1（时序）/b3（非确定）候选均未闭合。探针方向：M4（biome.rs BTreeMap）复核、禁 carvers/features 重跑定位尾随写入者、save 前后 hook dump。
2. **basalt deltas 大宗互换（B1）**：76.6% 大头，层位/形状已锁定（y≤127、按区域成片）——surface rule 单列对拍 + 按 biome 分桶可定位。
   - **[已结案 260901-03]** 机制定论见本文「B1 定论」节：feature 阶段产物（blobs/columns/delta/pillar）在两种基底地形上的命中/形态差 + Rust surface 薄带残差；surface_depth 带厚候选被排除，nether_state_selector bug 另案（非主导）。
3. **矿石 features 缺口（A1+B4，3,269 块）**：「未实现」vs「放置错位」归属未定——feature 阶段 A/B diff 出 Rust 实际矿位 vs 参照矿位集合即可裁决（若错位，按发现 #6 查 PlacedFeatureIndexer 编号链）。

### 下一步深挖优先级（块数 × 可定位性，residual-interpretation §3）

1. **B1 表面规则大宗互换**（52,078，可定位性高：biome 分桶 + 单列 surface rule 逐步对拍）
2. **A1+B4 nether 矿石 feature**（3,269，中高：一次探针双 seed 受益，A1 占 seed A 残差 84.5%）
3. **B2 soul sand valley**（5,720，中高：已知遗留，限 y∈[0,4] 切片 diff）
4. **B3 熔岩湖边界**（1,375，中：y 直方图看是否聚 sea_level 附近）
5. **A2 cave_air 簇**（104，价值在排除 M4 家族复发而非块数）
6. **B5 magma**（1,694，中低：与 #1 同脚本分桶）

> 状态：**数据、口径声明、写盘无损结论（三口径同值）= confirmed（用户拍板 260901-03，judge-review #1-4/#15 通过）**；残差分类占比数据 confirmed、机制解释与待查项 = draft。
> 关联：`.investigations/nether-save-full/`（facts / .b1-.b3 / residual-interpretation / judge-review / nether-save-errors.md）。

## B1 定论：basalt deltas 三大宗石互换 = feature 阶段产物在两种基底地形上的命中/形态差（candidate，260901-03）

> 承接上节「nether 存档写入口径 Full 化」B1 未闭合项（52,078 块 / 76.6%）。本轮三方实验 + fan-out 两候选裁决后机制定论。过程与被推翻假说见 10 时间线 260901-03 条；错误 E6 见 `.investigations/nether-save-full/nether-save-errors.md`。

### 机制定论（B1 主导，candidate）

- **架构事实**：cppReplace 模式下 Rust 只接管 populateNoise + buildSurface（NoiseChunkGeneratorMixin.java）；vanilla carvers + features 仍在 Rust 地形上运行。
- **机制**：nether 的 basalt_deltas / soul_sand_valley 宗石大宗（basalt_blobs / blackstone_blobs = netherrack_replace_blobs、large/small_basalt_columns、delta、basalt_pillar）**本是 feature 阶段产物**，不是 surface rule 产物。同一套 Java feature 在两种基底地形（vanilla surface vs Rust surface）上运行，命中/形态不同 → 大宗互换；叠加 Rust surface 薄带残差。
- **biome 源分配差排除**：互换块 100% 落在 vanilla basalt_deltas 列内（soul_sand_valley 家族单列）——feature 的 biome 源分配两侧一致，排除。

### 三方实验证据（数据直读）

| 口径 | 数字 | 判读 |
|---|---|---|
| 纯 Rust 输出（ctypes 直连 dll vs rlib 直跑）vs FULL 参照 | **77.43%**（basalt→netherrack 157k）；ctypes/rlib cell 级 **0 差异**（确定性） | Rust surface 薄带 + 纯 Rust 口径下 blobs/columns 缺失的叠加表现 |
| 存档（Rust noise+surface + Java carvers/features）vs FULL 参照 | **93.5508%** | feature 阶段产物补回大头 |
| WG_SKIP_SURFACE=1 重跑 | **55.18%**，且 blobs 不触发（可用基底面大幅减少 + target=netherrack 依赖 → blackstone=0、quartz/gold ore=0） | **surface 是实心块主来源**，且证明 blobs 是 feature 阶段依赖 netherrack 基底 |

**待排除备择（judge WARN-4）**：Rust surface 薄带残差在 52k 中的量级未单独量化（待 SURFACE 口径参照重测）；「Rust 自身已实现 feature 与 Java feature 并存重复放置」（Rust 侧 ore_magma 等已实现）未显式排除——二者均列为下轮待查。
**369 块容差注记（judge）**：数值（两次 run 相差 369 块）= 过程事实；「Java feature 邻块写入调度非确定」的机制解释 = 推断（draft），未经隔离实验证明。

### 对照口径澄清（v0.20 §9.7）

- 纯 Rust 口径（77.43%）与存档口径（93.55%）**不可比**（载体不同，§9.7）。
- B1 深挖的正确参照分两用：**BlockProbe SURFACE 口径**（无 carvers/FEATURE）测 Rust surface 残差；**存档口径**测端到端。
- **同 dll 非确定性容差（新过程事实）**：同 dll 两次完整 run 相差 369 块（93.5156% → 93.5508%）——Java feature 阶段邻块写入调度非确定性所致。**存档口径对齐指标必须声明该容差**（同 dll 重跑差 ≤ 百分级块数属正常，非实现回归）。

### 附带定论与遗留

- ✅ **Hole 语义不一致已闭合**（取代本篇前文遗留课题中的 Hole 行）：docs/09 前文「Rust Hole 用 surface_depth<=0」为 M6（2026-08-30）修复前的过时表述；当前 Rust surface_rules.rs L101 `Hole => stone_depth_above <= 0` 与 Java 一致，dll M17（sha C5AC5309）含修复。见上方 supersedes 标注。
- ❌ surface_depth 带厚机制（fan-out .b1）：不成立——带厚上限 ≤6 层，实测 40 层体块不可达（`.artifacts/.b1-surface-depth/` verdict）。
- ⚠️ nether_state_selector 恒 0.0（fan-out .b2）：**真实 bug**（`create_for_dim` step4 预加载表缺 nether 噪声：nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer → `unwrap_or(0.0)`），但只解释零星分支内翻转，**非 B1 主导**（`.artifacts/.b2-nether-state-selector/`）。修复值得做（一行预加载表补齐），预期闭合 soul_soil 子族等——**待修，修复后重测**。
  **[supersedes 260901-04]** 上行「待修」状态作废：已修复并重测，见下节。

## nether_state_selector 预加载表修复（.b2 遗留项闭合，candidate，260901-04）

> 置信度 **candidate**；验证分层 **Partial**（存档口径端到端对齐 + 日志判据核对，非逐位 Full）。§9.7：93.8988% 为**存档口径**，与纯 Rust 口径不可比。

### 修复内容

`WorldgenRust/src/worldgen_handle.rs` step4 surface rules 噪声预加载表（L192-195 一带）原只含 overworld 噪声（surface / surface_secondary / clay_bands_offset / badlands_* / gravel / powder_snow / packed_ice / ice / surface_swamp），**缺全部 nether 噪声**。下游 `surface_rules.rs` 的 `noise_threshold_sample`（L120-137）查不到 sampler 时 `unwrap_or(0.0)` 静默回退 → `nether_state_selector`（min threshold = 0.0）条件恒 true → nether surface rule 恒走 basalt 分支。

修复 = 预加载表补 6 个 nether 噪声：`minecraft:nether_state_selector` / `patch` / `soul_sand_layer` / `netherrack` / `nether_wart` / `gravel_layer`（全部存在于 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise/*.json`）。

### 验证（存档口径，seed B = 8576294172403134396，4×4 @3200,3208，参照 = vanilla FULL）

| 项 | 数字 | 判读 |
|---|---|---|
| 修复前基线 | 93.5508%（上轮） | 同 dll 非确定性容差实测 ±369 块 ≈ ±0.035pp |
| 修复后 | **93.8988%**（match = 984600/1048576） | **+0.348pp ≈ 10× 容差 → 超出非确定性噪声，真实改善** **[supersedes 260901-04]** 单点倍数表述作废，见下方「容差口径修正」 |
| E1/E3 判据核对 | 通过 | `[CppBridge] initNether enabled=true` 且 seed 一致；log = `.investigations/nether-save-full/cmd-output/b2-fix-rerun.log` |

### 修复后分族（b1_family_split.py / b1_id_totals.py）

- 总 mismatch 63,976：solid_solid 62,850 / van_solid_rust_air 580 / van_air_rust_solid 546。
- soul_soil：ref 5474 vs save 1334（仍偏低）——**selector 噪声已生效**，soul_soil 大头疑似在 Java feature 阶段，属 B1 主导机制（feature 产物 × 基底地形差）的正常残差，**不是本 bug**。
- soul_sand 2457 vs 1471；quartz_ore / gold_ore / magma 仍偏高——ore features 归因（待 A1+B4 重估）。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照（WGB2）；覆盖面：4×4 chunk 全高度（min_y=0, height=256）。
- 可比性：93.8988% 与纯 Rust 口径 77.43% 不可比；与修复前 93.5508% 同口径可比（容差 ±369 块已声明）。

### 容差口径修正（candidate，260901-04，C1 修复回归暴露）

> supersedes：本段取代上方验证表中「+0.348pp ≈ 10× 容差」的容差倍数表述——n=2 容差样本低估散布（judge CONCERN C4 属实），改善量表述修正为区间下界；原表行不删，加注记。

背景：C1 修复（`surface_rules.rs` 未知 noise key 每 key warn 一次，行为中性）后两次回归 rerun（log = `.investigations/nether-save-full/cmd-output/c1-warn-regression.log` / `c1-warn-regression2.log`）暴露容差样本量问题。

| 代码版本 | run 结果 | 判读 |
|---|---|---|
| 修复后（3 次） | **93.8988%**（984600）/ **93.6767%**（982271）/ **93.6765%**（982269） | 后两次仅差 2 块（近确定），与前一次差 ~2330 块（≈0.222pp） |
| 修复前（2 次） | 93.5156% / 93.5508% | 即上轮 ±369 块容差的来源（n=2） |

1. **「修复真实改善」结论保持成立**：两组区间不重叠（修复后最小 93.6765 > 修复前最大 93.5508）。改善量表述修正为——**下界 = +0.126pp（≥3.6× 旧容差），点估计随调度波动（+0.126~+0.348pp）**。
2. **同 dll 存档口径散布实测可达 ~2330 块（≈0.22pp），此前 ±369 块（n=2）是低估**——n=2 样本无法覆盖调度非确定性的真实散布。
3. **后续存档口径比对判据统一为「区间不重叠 + 多次采样」**，不再用「单次差值 vs ±369 块」判定改善/回归。
4. **C1 代码验证通过**：行为中性、构建绿；两次回归均无 SURFACE-WARN 触发 = 预加载表完备性同时得到运行时佐证（`initNether enabled=true`）。

### 状态

- candidate（Partial 分层 + 容差倍数判真改善）；confirmed 留用户。
- 过程 → 10 时间线 260901-04 条；错误 E7 → `.investigations/nether-save-full/nether-save-errors.md`。

## SURFACE 口径残差量化：Rust surface 层自身残差 = 22.5%，主导形态 = basalt/blackstone 位放 netherrack（candidate，260901-04）

> 承接「B1 定论」节 judge WARN-4 待排除备择。验证分层 Partial（SURFACE 参照口径 = BlockProbe 无 carvers/features）。

### 采集与口径

- vanilla SURFACE 参照：BlockProbe 默认口径（无 carvers/features；`-PblockProbe -PblockProbeDimension=nether`，**不带** blockProbe.full），seed B = 8576294172403134396，4×4 @3200,3208。export log = `.investigations/nether-save-full/cmd-output/b2-surface-ref-export.log`（`BlockProbe worldSeed=` 核对一致）。
- FULL 参照已备份 `.blocks.full`，hash 不同确认口径切换生效（SURFACE 270D6E97… vs FULL 1DDE3B09…）。
- 对比脚本：`.tmp/b2_surface_residual.py`；纯 Rust 侧 = rlib dump（`.tmp/b1-rlib-blocks.bin`）。

### 数据（数据直读）

| 对比 | 数字 | 判读 |
|---|---|---|
| SURFACE 参照 vs FULL 参照 | diff 仅 21,296/1,048,576（**97.9691% identical**） | 本 4×4 区域 features 贡献 ~2%；黑石/玄武岩大宗主体在 surface rules 层（SURFACE 参照 basalt = 173,073 vs FULL 172,704） |
| SURFACE 参照 vs 纯 Rust rlib dump | **77.4857%**（match = 812496/1048576） | Rust surface 层自身残差 = 22.5%（SURFACE 口径） |
| 分族 | solid_solid 233,197 / ref_solid_rust_air 2,871 / ref_air_rust_solid 12 | 残差几乎全是实心块互换 |
| top mismatch | basalt→netherrack 157,658 / blackstone→netherrack 35,031 / cave_air→netherrack 15,678 | 主导形态 = vanilla surface 放 basalt/blackstone 处 Rust 放 netherrack |

### 结论（candidate）

1. **Rust surface 层自身残差 = 22.5%**（SURFACE 口径）——B1 定论中的「薄带残差」实为 surface 层大宗差异（非薄带）。
2. **存档口径 93.8988% 说明 Java features 在 Rust 基底上补齐了其中大部分**——与 B1 定论「feature 产物 × 两种基底地形差」自洽。
3. **judge WARN-4 备择「Rust 已实现 feature 与 Java feature 并存重复放置」排除**：cppReplace 架构只拦截 populateNoise + buildSurface，features 只由 Java 运行一次（无双跑通道）；SURFACE 口径 Rust 侧残差形态与存档口径收敛方向一致。
4. **⚠️ 外推限制**：FULL−SURFACE 差 ~2% 是 4×4 局部观察（basalt deltas 宗石恰好 surface 主导），**勿外推为全局 features 贡献占比**。

### 口径声明（§9.7 三要素）

- 载体：SURFACE 参照 = BlockProbe 默认口径（无 carvers/features）vs 纯 Rust rlib dump；覆盖面：4×4 chunk 全高度，seed B。
- 可比性：**77.4857%（SURFACE 口径）与 93.8988%（存档口径）载体不同不可比，分列**；与 B1 节纯 Rust 口径 77.43%（FULL 参照）亦不同载体，分列——数值接近纯属本区域 features 占比低的巧合，不构成口径可合并的证据。

> **[supersedes 260902-05]** 本节「SURFACE 口径残差 = 22.5%」结论作废——参照口径污染：BlockProbe 原实现无条件预生成 FULL 邻域 chunk，SURFACE 口径请求 getChunk 返回的实为已提升的 FULL 混合数据（SURFACE 参照 basalt=173k 实为 features 产物混入），22.5% 残差量化基于被污染参照，不成立。修复后真 SURFACE 参照（hash 02B94092）重导：Rust surface-only vs 真 SURFACE = **99.9423%**，surface 规则层基本正确；残差主体改判特征阶段 blobs。见本文「B1 下钻 H1 定案（260902-06）」节 + 10 时间线 260902-05 条。原节数据与结论不删不改，仅作历史口径记录（§15.4 取代链）。

## C2 预加载表数据驱动化：nether 噪声 key 从 surface_rule JSON 构建期收集（confirmed，260902-01 用户拍板，commit 709b006）

> 承接「E7 修复」节：E7 修复 = 手工补齐 nether 6 key 清单；本节 = 同一问题的架构层收尾（数据驱动化，对齐 AGENTS.md 数据驱动铁律）。

### 改动内容

- `worldgen_handle.rs` step4 预加载表数据驱动化：新增 `collect_noise_keys()`，从 surface_rule JSON 构建期自动收集 noise_threshold 引用的 noise key，预加载表不再依赖手工清单。
- **overworld 保留静态清单**：overworld 为代码规则（`build_overworld_rule`，无 JSON 源），数据驱动化不适用，静态清单保留（已验证代码规则不动）。
- **nether 静态 6 key 清单删除**（E7 手工补的清单由 JSON 收集取代）。

### 验证（存档口径，同 E7 口径）

- 3 连跑 **93.8988% 逐位同值**，无回归。
- judge C2 CONCERN 已闭环。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照（同 E7/B1 口径）；覆盖面：4×4 chunk 全高度（min_y=0, height=256）。
- 可比性：与 E7 修复后基线 93.8988% 同口径可比；本节为重构性质（行为不变验证），非改善量声明。

### 状态

- **confirmed（260902-01 用户拍板）**。过程 → 10 时间线 260902-01 条。

---

## 矿石归因定论：双重 feature 应用（confirmed，260902-01 用户拍板；judge PASS）

### 机制（H_B'）

- `wg_fill_blocks_multi` 内含 **carver + feature 阶段**（`worldgen_handle.rs` L442-449，`WG_SKIP_CARVER` / `WG_SKIP_FEATURES` env 门控）。
- 存档链路 mixin 只拦 **populateNoise + cancel buildSurface**，Java 侧 CARVER / FEATURES 步骤照跑 → **存档 = Rust features + Java features 双跑**。
- 修正早前结论：09 篇「SURFACE 口径残差量化」节曾写「cppReplace 架构只拦截 populateNoise + buildSurface，features 只由 Java 运行一次（无双跑通道）」——该判断对 mixin 拦截范围描述正确，但漏了 Rust 侧 `wg_fill_blocks_multi` 内含 feature 阶段这一半，双跑通道实存。**[注 260902-01]** 原行不删，以本节为准。

### 消融证据（seed B，4×4 @3200,3208，存档口径）

| 实验 | match | 矿石计数变化 |
|---|---|---|
| 基线 | 93.8988% | quartz 4478 / gold 1525 / magma 3814 |
| +WG_SKIP_FEATURES=1 | **94.4241%**（+5508 块） | quartz 2125（ref 1992）/ gold 739（ref 728）/ magma 1979（ref 1533） |
| +WG_SKIP_CARVER | 仅再 +370 | — |

### 结论（candidate）

1. 矿石 ~2.2× 偏高**全额归因 Rust features 双跑**（SKIP_FEATURES 后三族矿石均落回 ref 邻域）。
2. carver 双跑贡献小（+370），非主导。
3. **遗留（idk）**：overworld 同路径理论上也双跑，但 overworld 存档对齐 99.9%——是否同样双跑及为何不显形，待 X1 FEATURELOG 裁决（进行中，结论回填时间线，不在本节预写）。
4. **修复方向 judge CONCERN**：`WG_SKIP_*` 是 env 门控（进程全局），修复勿用全局默认翻转——需句柄/调用级显式 flag，避免污染其它调用方。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照；覆盖面：4×4 chunk 全高度，seed B。
- 可比性：消融各列同为存档口径，列间可比；与 SURFACE 口径 77.49% / 纯 Rust 口径 77.43% 不可比，分列。

### 状态

- **confirmed（260902-01 用户拍板）**。

---

## soul sand valley 归因三签名（B2 定稿，260902-01）

### 上轮假设证伪（supersedes 注记）

> **supersedes**：本节推翻 docs/09 早前小节「soul_soil 大头疑似在 Java feature 阶段，属 B1 主导机制的正常残差」表述（原行不删，加注记）。

- **证伪证据（V1）**：Rust 管线自身 soul_soil 1363 ≈ 存档 1334——Java feature 阶段并未产出 soul_soil 大头；缺口 4140（ref 5474）**在 Rust 管线内部**（surface 层机制缺失），非 Java 侧残差。

### V2 探针三签名（180 点，三签名）

| 签名 | 现象 | 判读 |
|---|---|---|
| A | biome 足迹偏移/收窄：valley 点 Rust 判 nether_wastes，聚簇 x≥3410 边界带 | biome 判定边界带差 |
| B | soul_soil 子分支失效：entered + selector<0 仍 applied=netherrack | surface rule 子分支未生效 |
| C | floor 侧 soul_sand_layer 分支疑似缺失：组3 entered 0/60 | 候选缺失（待结构对拍确认） |

- 候选分派：**.b1a（结构差）主导**；**.b1b（噪声值偏离）idk**——缺 Java 同点对照，诚实标注未决。
- Java features 对 soul_sand 为**净回补 +587**（与矿石双跑偏高方向相反，单独记）。

### 下一步（优先级序）

1. **V3：Rust-vs-JSON rule 结构对拍**（零成本，最高优先）；
2. V4：RouterProbe 同点 selector 对比；
3. V5：biome 边界带对比。

### 口径声明（§9.7 三要素）

- 载体：V2 = Rust 管线探针 + 存档读回 + ref 三方；覆盖面：180 采样点（含边界带聚簇）。
- 可比性：V1 Rust 管线口径 vs 存档口径载体不同，分列引用；.b1b 无 Java 同点对照，不构成可比结论（idk）。

### 状态

- **confirmed（260902-01 用户拍板：缺口在 Rust 管线内 + 三签名方向）**；.b1b 内部机制 idk 保持（不阻塞本定论）。过程 → 10 时间线 260902-01 条。




## 句柄级 wg_set_flags 修复 cppReplace 存档链路 Rust features/carver 双跑（confirmed，260902-02；judge PASS；用户拍板 260902-03）

> 承接「矿石归因定论」节结论 4 的 judge CONCERN（`WG_SKIP_*` 为进程全局 env 门控，勿全局默认翻转）。修复验证分层 **Partial**（存档口径端到端 + ore per-id 消融值佐证，非逐位 Full）。§9.7：94.4241% 为存档口径，与 SURFACE/纯 Rust 口径不可比。

### 修复内容

- **worldgen_handle.rs**：`AtomicU32 flags` 句柄级标志位——bit0=SKIP_CARVER、bit1=SKIP_FEATURES、bit2=SKIP_SURFACE；**OR-env 语义**（句柄 flag 与 `WG_SKIP_*` env 任一置位即生效）；flags=0 时回落 env 兼容行为（存量调用方零影响）。
- **api.rs**：新增 `wg_set_flags(handle, mask)` / `wg_get_flags(handle)`。
- **jni_bridge + Java**：`CppWorldgen` / `CppBridge` 透传；存档链路默认 **mask=0b011**（SKIP_CARVER|SKIP_FEATURES，即存档链路不再双跑），可用 `-Dcoreswap.rust.stages` 系统属性覆盖。

### 回归验证（seed B = 8576294172403134396，nether 4×4 @3200,3208，FULL 参照，ReadWorldProbe 存档口径）

| 项 | 数字 | 判读 |
|---|---|---|
| 修复前（消融轮） | 93.8988% | 存档 = Rust+Java features 双跑基线 |
| 修复后（同 region 3 次复跑，seed B 4×4@3200,3208，存档口径） | **全部 94.4241%**（990108/1048576） | 同 region 复跑零散布 = 确定性/可复现（非多 region 覆盖面）；与消融轮 SKIP_FEATURES 值逐位一致 = 双跑通道闭合 |
| ore per-id 直接佐证 | quartz 4478→2125 / gold 1525→739 / magma 3814→1979 | **= SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533），三族矿石全部落回 ref 邻域 |

- 判据（C1 措辞修正）：**同 region 3 次复跑确定性**（seed B，nether 4×4@3200,3208 同一 region，3 次全新 run 全部 94.4241% 零散布——验证的是确定性/可复现性，非多 region 覆盖面）+ **ore per-id 消融值佐证**（quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = SKIP_FEATURES 消融值，ref 邻域 1992/728/1533）——修复后值与手工 `WG_SKIP_FEATURES=1` 消融值逐位相同（重复了消融实验的因果链）。
- 日志：`.investigations/nether-save-full/cmd-output/flags-regression-run4/5/6.log`。

### 设计与审查记录

- 设计文档：`.investigations/nether-save-full/design-wg-set-flags-20260908.md`。
- judge 意见：`.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md`（PASS，建议 candidate）。
- confirmed 留用户拍板。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解（ReadWorldProbe 口径）vs vanilla FULL 参照；覆盖面：4×4 chunk 全高度，seed B。
- 可比性：与消融轮 94.4241%（SKIP_FEATURES）/ 基线 93.8988% 同口径可比；与 SURFACE 口径 77.49%、纯 Rust 口径 77.43% 不可比，分列。

### 状态

- **confirmed（260902-03 用户拍板；judge PASS）**；过程 → 10 时间线 260902-02 条。

---

## V3 结构对拍：nether surface_rule 解析器全节点一致，「分支缺失」假说否定（draft，Degraded，260902-02）

> 承接「soul 三签名」节下一步第 1 项（V3 结构对拍，零成本最高优先）。验证分层 **Degraded**（静态结构对拍，无运行时证据），MUST 声明降级。

### 对拍结果（排除式论证）

- **解析器能力面**：nether.json surface_rule 的全 **10 种节点类型** Rust 解析器全部支持；**7 个顶层分支逐节点一致**（节点类型/参数/嵌套结构与 JSON 语义等价）。
- **排除结论**：
  - ❌ **签名 B（soul_soil 子分支失效）**与 **签名 C（floor 侧 soul_sand_layer「分支缺失」）**的**结构差解释不成立**——结构层逐节点一致，不存在「分支没解析出来」。
  - → 签名 C 的「分支缺失」假说被否定；签名 B 的机制必须到**运行时输入**找。
- **归因指向（候选，未验证）**：
  1. **运行时输入差**（V4：生产链路 soul 分支 ctx dump vs probe 输入对差）——probe 的 V2 采样路径与生产链路输入可能不同源；
  2. **biome 分类层**（签名 A 同源，V5 边界带对比）。

### 口径声明（§9.7 三要素）

- 载体：静态结构对拍（nether.json surface_rule vs Rust 解析规则树）；覆盖面：7 顶层分支 × 10 节点类型全量。
- 可比性：结构层一致**不构成**运行时行为一致的证据（Degraded）——V2 三签名的运行时现象不被本节解释，仅排除结构差候选。

### 状态

- **draft（Degraded）**：排除结论（结构差不存在）数据直读可信；归因指向两候选均未验证。下一步 V4（ctx dump 对差）/ V5（biome 边界带）。
- 产物：`.artifacts/.b2-soul/v3-structure-diff.md`。过程 → 10 时间线 260902-02 条。


## 布尔字段解析 bug 修复签名 B/C（confirmed，260902-03；judge PASS；用户拍板 260902-03）

> 承接上节 V3 结构对拍的「归因指向」——V4 生产 ctx dump 先否定「输入差」候选（180 点生产 dump 与 probe 逐项全同），随后解析产物树 dump 锁定**求值层矛盾 = 解析器布尔字段 bug**。本节 supersedes V3 节的处置方向（原「到运行时输入找」的方向由本节取代，V3 原节不删）；V3 的「结构差不存在」排除结论维持成立，仅其「参数全对拍」子项为漏检（对拍的是 JSON 原文而非解析产物）。验证分层 **Partial**（bin-diag 解析树 dump + 定点 apply + 存档口径端到端，非逐位 Full）。

### 根因（supersedes：V3 节处置方向）

- `parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段 `add_surface_depth` / `add_stone_depth`——`JsonValue::Bool` 走 `as_f64()` 返回 `None` → **恒解析为 false**（surface_rules.rs 三处：y_above / stone_depth / water）。
- soul 分支 ceiling 条件 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1+0+0` → 分支该进未进 → 穿透 [7] 兜底 netherrack。V3 的「soul_soil 无条件兜底存在」推读本身正确，错在**解析产物树**；V3 静态对拍「参数全对拍」对拍的是 JSON 原文而非解析产物，被假阴性掩盖。
- 三处定位：L1079（y_above `add_stone_depth`）/ L1093（stone_depth `add_surface_depth`）/ L1116（water `add_stone_depth`）——`as_f64()` 对布尔返回 None；对照 `legacy_random_source` 的 `as_bool()` 读取为正确 API。

### 修复（surface_rules.rs）

- 新增 `SurfaceBuilder::parse_bool_field`：`as_bool().or_else(|| as_f64().map(|f| f != 0.0)).unwrap_or(false)`（bool 优先 / 数字 0/1 兼容 / 缺省 false，与 Java `GsonHelper.getAsBoolean(json, key, false)` 语义一致）。
- 三处调用替换：y_above `add_stone_depth` / stone_depth `add_surface_depth` / water `add_stone_depth`；grep 复核剩余 `as_f64` 命中均为纯数字字段，无残留布尔误读。
- overworld 不受影响（走代码规则树 build_overworld_rule，不经 parse_surface_cond）。

### 验证链（四级回归，seed B = 8576294172403134396）

1. **树复现**（soul_tree_repro，nether.json × surface_rules.rs 解析产物树 dump）：修复前解析树 8 处 `asd=false` 假阴性（其中 3 处 JSON 原文为 true；通用 floor 段等 8 处 JSON 原值即 false，恰成假阴性掩护）；修复后 5 处翻 true（soul ceiling/floor StoneDepth、gravel patch y_above 30/35、basalt floor stone_depth），其余 8 处保持 false = JSON 原值，无一误翻。产物：`.investigations/soul-v4v5/cmd-output/soul-tree-repro-postfix.txt`。
2. **生产 ctx dump**（soul_ctx_dump，180 点生产链，nether 4×4 @3200,3208）：netherrack 103→71，新增 soul_soil=18 / soul_sand=14（103=71+18+14 自洽）；定点 3260,1,3200（sdb=2, sd=3, selector<0）applied 256→**258(soul_soil)**，与 V3 语义推演逐位一致。产物：`cmd-output/soul-ctx-dump-postfix.stderr.txt`。
3. **nether 存档全量回归**（run4 模板照抄，FULL 参照，ReadWorldProbe 存档口径，4×4 @3200,3208）：94.4241%（990108/1048576）→ **run1 96.6215%（1013150/1048576）/ run2 96.5866%（1012784/1048576）**，+2.20pp。run1/run2 差 366 块，在已知同 dll 重跑非确定容差带内（workflow-patterns 发现 #10，实测散布 ~2330 块）——归因成立。
4. **soul 族 per-id 佐证**（save MCA vs FULL ref，同 seed/region/口径）：soul_soil 修复前 1334 → **5771**（ref 5474，偏高 +297，闭合至 ref 邻域，「完全闭合」不成立）；soul_sand 1471→2494（ref 2457）；quartz 2095（ref 1992）/ gold 711（728）/ magma 1543（1533）/ gravel 674（674，精确相等）。

### 附带闭合（同 bug 源次生影响面）

- **签名 C**：nether_wastes「soul_sand_layer 分支 entered 0/60」同 bug 源（floor 段 `add_surface_depth` 误读），随本修复闭合（soul_sand 2494 vs ref 2457 佐证）。
- nether **gravel patch 高度带**（y_above asd，锚 30/35）整体修正（gravel per-id 674=ref 覆盖）。
- **3275,2,3201 从签名 B 证据集剔除**：该点 y=2 ∈ bedrock_floor 随机带（above_bottom 0..5），生产侧 bedrock 先中即返（applied=31），本不用于 soul 表面判定；vanilla 同点亦先判 bedrock_floor。

### 残差（idk / 遗留）

- basalt −1736 / blackstone −434：**B1 surface 残差家族**（已知遗留，非本次范围）；修复前后对照（同 seed/region/口径）：basalt save-ref −3631（260902-02）→ −1736——asd 翻转无新负迁移，B1 家族反而收敛。
- 366 块非确定带宽（run1/run2）；**V5 biome 边界带（vs vanilla 足迹）未做**——修复后残差图需重导（readWorldProbe mismatch 全集），残差降至 ~3.4%。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解（ReadWorldProbe）vs vanilla FULL 参照 × 解析产物树 dump（bin-diag）× 生产 ctx dump（stderr）；覆盖面：nether 4×4 @3200,3208 全高度 + 180 点签名集；与既有口径可比（94.4241% 历史值同参照/同 region/同口径），与 SURFACE 77.49% / 纯 Rust 77.43% 口径不可比，分列。
- 本节数字全部带 seed+region+口径三要素（seed B = 8576294172403134396；region 4×4 @3200,3208；存档/树 dump/dump 口径分列）。

### 状态

- **confirmed（260902-03 用户拍板；judge PASS，review-001）**；supersedes 双指针：v3-structure-diff.md 参数对拍子项被 `.artifacts/.b2-soul/v4-eval-conflict.md` 细化取代。过程 → 10 时间线 260902-03 条。

## V5 残差归因：主体 = 同 biome 表面规则差（B1 家族本体），签名 A 降级 3.7%（candidate 待 judge，260902-04）

> 承接「布尔字段解析 bug 修复」节遗留的 V5 biome 边界带课题。本节 supersedes 前置假设「biome 分类器/存储填充异常（fan-out .b6）」——该假设由探针坐标 bug 假象制造（见 10 时间线 260902-04 条 + workflow-patterns 发现 #13），修正后排除。验证分层 **Partial**（ReadWorldProbe biome 列对比 + BIOME6/biome6_dump 6 维对拍 + storage cell dump，非逐位 Full）。

### 残差真图景（修正 wBiome 坐标 bug 后，mismatch-nether-run6.csv）

- 35426 mismatch 列中 **96.3% biome 完全一致**（basalt→basalt 32817 列 + ssv→ssv 1306 列）→ 残差主体 = **basalt_deltas biome 内 surface rule 分支判定差**（vanilla 写 basalt/blackstone 的列，Rust 写 netherrack 或 basalt/blackstone 互换）——与 B1 家族（basalt −1736/blackstone −434）同族本体，下钻候选：nether_state_selector 采样 / delta 分支进出条件 / blackstone·basalt 分配。
- **签名 A（biome 边界带）降级为次要项**：biome 真差仅 1303 列 ≈ 残差 3.7%（ssv→basalt 676 + basalt→ssv 627，边界互换）——可单独修（biome 6 维边界/offset 距离精度），非残差主体。

### 分类器排除证据链

- Java BIOME6 与 Rust biome6_dump 同 4 点 6 维值**逐位一致**（t=-0.5598/h=-0.2920 等）；两侧数学均判 basalt_deltas（dist 0.119 vs warped 1.080，非平局）→ 排除 offset 维语义（.b1）、shift/Perlin 种子差（.b2）、SearchTree 平局（.b4）。
- Rust 独立分类器（生产同路径组件重组）判定与 vanilla 参照一致；fillChunkNether 只写方块无 biome 写入；BIOMES 阶段 biome 容器由 Java 填充。

### per-id 量化（vanilla→save 净差，match 行精确等价法，judge 独立复核正负归零 ±1965）

netherrack +1539 / basalt −1050 / blackstone −652 / soul_soil +297 / soul_sand +37 / gravel 0；tail：quartz_ore +82 / lava −109 / air −128 / gold_ore −26 / magma +9 / red_mushroom +1（id→名 = data/blocks.json；basalt+blackstone −1702 ≈ netherrack+soul_soil +1836，delta 族互换闭环）。basalt −1736（run2）vs −1050（本轮）漂移在 #10 非确定容差内。

### 口径声明（§9.7 三要素）

- 残差/对齐：存档读回 vs vanilla 参照 blocks（B/4/3200,3208/nether），与 M16 起 96.62% 口径可比；本轮复现 96.6215% 与 confirmed 逐位一致（seed 三查 ✓）。
- biome 对比：ReadWorldProbe CSV（vanilla 列 biome @y=100 world.getBiome 平滑值 vs save 同方法）+ BIOME6/biome6_dump 原始 router 采样（UnblendedNoisePos 直采）——两口径分列，不混比。
- 覆盖面：单 seed B、单 4x4 域，占比外推性有限。

### 状态

- **candidate（judge 有条件 PASS 260902-04，P1-P5 已落实；confirmed 留用户）**。过程 → 10 时间线 260902-04 条；探针坐标 bug 教训 → workflow-patterns 发现 #13。

> **[supersedes 260902-05]** 本节结论「残差主体 = basalt_deltas 同 biome 表面规则差（surface rule 分支判定差，B1 家族本体）」部分改判：BlockProbe FULL 预生成污染修复 + 布尔修复后重导，**surface 规则层 Java(nether)↔Rust = 99.9423% 一致**——「同 biome 表面规则差」作为残差主体的表述作废；残差主体改判为**特征阶段 blobs 在相同基底上的放置差**（存档 mismatch 35,426 块中 98.5% 落在两侧基底完全相同的块上）。本节的分类器排除证据链（biome 真差 3.7%、6 维逐位一致）维持成立。见「B1 下钻 H1 定案（260902-06）」节。

## B1 下钻 H1 定案：熔岩海缺失 → 转换面漂移 → delta/blob 链式放大（candidate，judge 审查中，260902-06）

> **[supersedes 260902-06]** 本节第 1 环（Rust surface 熔岩海缺失——netherrack 实心兜底）**被数据证伪**：`[LAUIDMAP]` Java STATE_IDS 权威映射实测 `19319 = blackstone`、`5854 = basalt`（air=0 lava=96 water=80 netherrack=5850 basalt=5854 blackstone=19319 soulsand=5851 soulsoil=5852 magma=12402 bedrock=79）——COLPROF 10/25 列 diff 真相 = **V 黑石底（y=99 恒平）vs C 玄武岩底（y=100~104 贴地形）**，两侧均为实心材质，无任何熔岩缺失（LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零）。基于该环的「熔岩流体填充」修复方案作废未执行。环 2~5（转换面漂移 → delta origin 漂 → 级联/blob 放大）作为**现象**维持成立（cfg 探针独立证据 delta y=111/119/121 vs 99；CountMultilayerPlacementModifier y-零随机 findPos 语义只需「第一转换面不同」即可触发），因果入口需重定位。范围限定与口径声明（§9.7 三要素）：本证伪基于**已扫描区域**（seed=8576294172403134396，3200,3208 size=4+外扩环，11,443 公共列）；「99.9423%」为 4×4 固体表面顶块口径，与本节 10/25 列内部转换面差（y=99~104，顶块以下）载体/覆盖面不同、不可直接比较。新机制候选（材质分支差 / biome 输入差 / 随机序列差 / 前置地形形状差）待四候选 fan-out 判别。见 10 时间线 260902-07 条 + `.investigations/b1-downdrill/facts-260902-07.md` + judge-opinion-260902-07.md。原节数据与结论不删不改，仅作历史记录（§15.4 取代链）。

> 承接 260902-05 开工点 H1/H2。本 session 完成 P2 放大模拟排除 H2 + P1 三层探针链对拍，H1 机制链 candidate 定案。验证分层 Partial（COLPROF/cfg 对拍数据层证据 + 量级自洽推理，非逐位 Full）。事实链全文：`.investigations/b1-downdrill/facts-260902-06.md`。

### 机制链五环

1. **Rust surface 熔岩海缺失**：nether surface 以 netherrack 实心兜底填充，未按 vanilla 熔岩海规则填流体（y=99 lava 面）——成片结构性，非随机散点。
2. **CountMultilayerPlacementModifier 转换面序列漂移**：y 零随机语义（x,z 随机 + MOTION_BLOCKING 起点找第 i 个 air/water/lava → 实心转换；sources.jar 已核）——熔岩海缺失使转换面序列系统性漂移。
3. **delta origin y 漂移**：首分叉行 320，`delta` V=(51178,99,51319) C=(51178,99,51310)——同 x、**异 z**（judge 订正：初稿误标「同 x,z 异 y」；且该行 V/C 分属不同 decorated chunk，是流内 skip 的行对齐边界伪影，不构成同点 y 对比证据；y 漂有效证据 = COLPROF cpp 侧 y=100~104 替代 99 + cfg-cmp C 侧 delta y=111/119/121）。
4. **delta 流内级联**：per-placed-feature RandomSequence 隔离 → 差异只污染自身，per-feature 计数差 ±1~5（basalt_blobs ±1 / blackstone_blobs ±1 / delta −3 / small_basalt_columns −5 / direct ±1）。
5. **blob 链式放大皮肤差**：only_v=644 / only_c=637（xyz+featureId，~3%）→ 存档皮肤差 only_v=450/only_c=445 量级自洽。

### COLPROF 证据（终审）

首分叉邻域 25 列枚举时刻转换面：**10 列 diff 全部同构**——V 含 `99|0->19319`（air→lava，熔岩海面 y=99），C 同转换缺失、代以 `100~104|0->5854`（air→netherrack，各列 y 不一）；其余 15 列逐项相同。前置层证据：PlacedFeature 入口两轮 1308 行全同（随机序列 chunk 级分叉排除）；ConfiguredFeature placedPos vanilla 20,327 / cpp 20,320；终态列 dump 9216 列 × 4 口径逐列全同（口径不含流体层 → 流体差隐形，恰与 COLPROF 互补）。

### 已排除

- ❌ H2「差基底放大」：per-region 触碰 blob 期望 ≈3.7 个 × 上限 853 ≈ 3,150 块，对 34,246 缺口 shortfall ~10.8× → INSUFFICIENT；且 1,122 差基底中可种子口径仅 472（~650 块 soul 侧不可触发 blob 链）、「触碰即翻转」不物理。
- ❌ 随机序列 chunk 级/全局分叉（入口全同 + 转换面漂移签名）；❌ biome 过滤差（链尾过滤不碰随机）。

### 环节分级（judge ① 项）

- **第 1-2 环（熔岩海缺失 / 转换面漂移）= COLPROF 数据层硬证据**；
- 第 3 环（delta origin 漂）= cfg 数据（首分叉）+ 伪影订正后仍成立；
- **第 4-5 环（流内级联 / blob 链式放大）= 推理 + 量级自洽，非独立探针——confirmed 前须 LAVAAUDIT / 修复回归升级为证据**。

### 修复落点与回归判据

- 修复落点 = **Rust nether surface 填充规则**（按 vanilla 熔岩海规则填流体，禁止实心兜底）；Java/Mixin 侧无错。
- 回归判据：① COLPROF 25 列两轮逐项相等（`99|air->lava` 恢复）；② cfg 对拍 delta y 收敛全 99、blob 族计数差归零、only_v/only_c 归零；③ blob 对拍 20327==20327 全同 + 存档口径（vs FULL 1DDE3B09）不重叠区回归 + overworld 基线不破。
- LAVAAUDIT 补充验收（不阻塞授级）：colprof 加 `-Dcolprof.mode=lavaAudit -Dcolprof.r=64` 两轮 diff → 熔岩海缺失列数 + 空间分布（成片=高度/区域规则错；按 biome 分块=biome 输入差）。

### 口径声明（§9.7 三要素）

- 99.9423% 是「4×4 区内固体表面」口径；600 块残差清单**漏区外熔岩海残差**；熔岩海缺失（成片）与 soul 散点（散点替换）不同类，修复分模式处理。
- seed 三查声明（judge B 项）：cpp 轮各 log 有 CppBridge init seed=8576294172403134396 直证；vanilla colprof 轮无直接 worldSeed 行，以同参数命令行 benchSeed + 输出一致性作间接声明。

### 状态

- candidate（judge 审查中）；confirmed 留用户。过程 → 10 时间线 260902-05/06 条；错误 → `.investigations/b1-downdrill/b1-errors.md`。

---

## B1 四候选判别定案：残差主体 = NOISE 单元格微差驱动的 band 边缘平移，历史「surface rule 差 / feature blobs 差」判为口径污染（candidate，260902-08/09）

> **[supersedes 260902-09，§15.4 双指针]** 本节取代：①「B1 定论」节（260901-03）「feature 阶段产物在两种基底地形上的命中/形态差」；②「B1 下钻 H1 定案」（260902-06）「转换面漂移 → delta/blob 链式放大」的因果入口方向；③历史残差观测（13.70% air、22.5% SURFACE、黑石/玄武岩底界差）判为**测量口径阶段污染**。被取代节原正文不删不改。

### 判别结论（fan-out 四候选，seed 8576294172403134396，chunk(3200..3203,3208..3211)，basalt_deltas）

- **(d) NOISE 宏观地形差排除**：air 签名 99.68% 一致（4083/4096 列，同阶段对拍）。
- **surface 层实现差上界 0.005%**：99.66% 列一致（26/524288 单元，id 投票映射后）——非主体。
- **(a)(c) 排除**：差异呈 band 边缘对结构、非系统/随机。
- **残余定论**：NOISE 单元格微差（13 列）驱动 band 边缘 ±1 平移 + 1 列 blackstone/lava 边缘平移（51247,51375，y20-24）——非规则/feature 机制差。
- 真实存档残差（~3.4%）主体归因 feature/carver 链路差（**放大系数未量化，idk**）。
- **保留项（不属污染）**：signature A（biome 3.7% 真差）；soul_soil V1 缺口。
- **新架构事实**：Rust fill_chunk_blocks 在 surface skip 时输出统一 default block（id=1），材质分配在 surface 层；vanilla NOISE 已是真材质 → **NOISE 层材质级对拍不可做，air 签名可做**。

### 口径声明（§9.7 三要素）

- 载体 = 阶段化逐列 dump（air 签名/surface 列对拍）+ 存档口径；覆盖面 = 单 seed / 单 biome（basalt_deltas）/ 4×4 chunks——**外推边界：多 seed/多 biome 未验证**；与 13.70% air、22.5% SURFACE 等历史口径不可比（后者已被判污染），与 96.6% 存档口径载体不同分列。

### 后续探针（指名）

- ① 多 seed / 多 biome 同阶段重采样（破单点外推）；② feature/carver 放大系数量化（~3.4% 残差归因闭合）。

### 状态

- **confirmed（260902-09 用户拍板；外推边界单 seed/单 biome/4×4 不变）**。产物 = `.artifacts/b1-candidates/four-candidate-verdict-260902-09.md`；审查 = `.investigations/b1-candidates/review-260902-judge-b1.md`。


