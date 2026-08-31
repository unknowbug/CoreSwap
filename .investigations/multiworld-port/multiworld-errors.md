# Rust worldgen 多世界参数化：错误与根因清单（重点记录）

> 载体：`.investigations/multiworld-port/multiworld-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录 Rust（WorldgenRust/）「Rust worldgen 多世界参数化」里程碑（2026-08-29，对齐 C++ `wg_create`）中定位并修复的错误。本 session 共 2 个错误（M1/M2）。多世界结论性架构见 09 主题篇追加小节；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。
> 背景：`WorldgenHandle::create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)` 参数化任意维度——dfNs 命名空间 + 维度参数（min_y/height/sea_level/aquifers_enabled）从 `noise_settings/<settings>.json` 读 + 非 overworld 维度用 JSON surface_rule 数据驱动。验证结果：nether 加载成功（min_y=0/height=256）+ 生成 chunk(0,0) 56307 非空气块；overworld 回归 95.40% 不变。

---

## M1. nether 加载 panic「unresolved density function ref: minecraft:nether/base_3d_noise」——resolve_ref 惰性加载硬编码 `minecraft:overworld/` 前缀

### 现象
- 用 `create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256)` 加载 nether 维度时，构建 final_density 树 panic：
  `unresolved density function ref: minecraft:nether/base_3d_noise`。
- overworld（`create` 便捷入口，settings_name="overworld.json"）不受影响，正常加载。

### 根因（机制）
- `DensityBuilder::resolve_ref` 的**惰性按需加载**分支硬编码了命名空间前缀：
  ```rust
  // 修复前（density_builder.rs）
  if key.starts_with("minecraft:overworld/") && self.external_loader.is_some() {
      let name = key["minecraft:overworld/".len()..].to_string();
      ...
  ```
- nether 的 density JSON 引用 `minecraft:nether/base_3d_noise`——**`minecraft:nether/` 前缀不匹配 `minecraft:overworld/`** → 惰性加载分支跳过 → 落到 `panic!("unresolved density function ref: {}", key)`。
- 本质：**命名空间是写死的 single-world overworld 假设**，未参数化维度。引擎本应从 settings_name 派生命名空间，却把「minecraft:overworld/」编进 lazy-load 判定。

### 定位（诊断链）
1. panic 信息 `unresolved density function ref: minecraft:nether/base_3d_noise` 直接点出**未解析的引用 key 前缀不对**——不是「文件缺失」而是「前缀不识别」。
2. 读 `DensityBuilder::resolve_ref`（density_builder.rs L202 附近）惰性加载分支 → 看到硬编码 `key.starts_with("minecraft:overworld/")` + `key["minecraft:overworld/".len()..]`——确认前缀写死。
3. 交叉核对 `nether.json` 的 final_density 引用前缀是 `minecraft:nether/` → 确定是命名空间不匹配，非文件路径/内容问题。

### 修复
- 给 `DensityBuilder` 增加 `df_ns: String` 字段（默认 `"overworld"`）+ `set_df_ns(&str)`（density_builder.rs）。
- `resolve_ref` 惰性加载分支改为**前缀从 df_ns 派生**：
  ```rust
  let prefix = format!("minecraft:{}/", self.df_ns);
  if key.starts_with(&prefix) && self.external_loader.is_some() {
      let name = key[prefix.len()..].to_string();
      ...
  ```
- `create_for_dim` 里 `df_ns = settings_name 去 ".json"`，`db.set_df_ns(&df_ns)` 同时决定 lazy-load 前缀与 external_loader 读的 `density_function/<dfNs>/` 目录（两处同源）。
- 修复后 nether 的 `minecraft:nether/base_3d_noise` 前缀匹配 → 惰性加载正常 → nether 加载 + 生成 chunk 成功。

### 教训（可复用判错经验）
- **多世界参数化的第一坑 = 「命名空间/维度数据不要硬编码」**——任何 `"minecraft:overworld/"` 字面前缀/目录都是 single-world 硬编码，遇到新维度必炸；命名空间前缀、噪声 settings 路径、sea_level/min_y 全部要从 settings_name / settings JSON 派生。
- **panic 「unresolved ... ref」先看前缀是否被代码识别**（写死 prefix vs 派生 prefix），再看文件是否存在——前缀识别错是「结构错」，文件缺失是「环境错」，两者定位路径完全不同。
- **惰性加载前缀 + 目录是两处同源**：`minecraft:<df_ns>/` 前缀与 `density_function/<dfNs>/` 目录必须同时参数化，只改一个会「前缀匹配但读错目录」或反之（本实例两处都在 create_for_dim / resolve_ref 同步改对）。

---

## M2. `VanillaAquifer` 加 `enabled` 字段后 20 个 probe 报「missing field enabled」——批量字段改动破坏所有 struct literal 构造点

### 现象
- 给 `VanillaAquifer`（terrain.rs）加 `enabled: bool` 字段（下界 aquifers_enabled=false 时跳过真实 aquifer）后，**约 20 个 probe**（badlands_probe / beard_cmp / biome_fill / blocks_cmp / carver_probe / fillbench / fillmap / fillprofile / grass_probe / mt_fill / perf_quant / surface_probe / terracotta_probe …）编译报：
  `error[E0063]: missing field 'enabled' in initializer of VanillaAquifer`。
- WorldgenRust 主 crate（worldgen_handle.rs）自身也报同样的 missing field（该处用 struct literal `VanillaAquifer { aq, ... }` 构造）。

### 根因（机制）
- **Rust 的 struct literal 是逐字段显式构造**：`pub struct VanillaAquifer { pub aq: Aquifer }` 原本单字段，任何构造点写 `VanillaAquifer { aq }`；给 struct 加一个**非 Default 的 `pub enabled: bool` 字段**后，**所有该 struct 的字段初始化表达式全部失效**（Rust 无隐式缺省字段默认值），编译器每个构造点都报 `missing field 'enabled'`。
- 波及面 ≈ **全部直接以 struct literal 构造 VanillaAquifer 的调用点**（主 crate + 各探针 bin），是「数据结构字段变更」的**结构性连锁破坏**，不是单一逻辑错。

### 定位（诊断方法）
- `cargo build`（或 clippy/check）报的 `error[E0063]: missing field 'enabled'` 直接列出**每个失效构造点的文件+行**——错误清单本身就是「受影响调用点」的完整位置索引。
- 数出 probe 数（约 20 个 bin）+ 主 crate → 确定是**结构性批量破坏**，须统一收口，而非逐个补字段。

### 修复
- 在 `VanillaAquifer` 加**便捷构造器** `VanillaAquifer::new(aq) -> Self { Self { aq, enabled: true } }`（terrain.rs）——把「默认 enabled=true」语义收敛到**一个构造入口**。
- 各 probe 的 `VanillaAquifer { aq }` → `VanillaAquifer::new(aq)`（约 20 处）；主 crate（worldgen_handle.rs）用显式 `VanillaAquifer { aq, enabled: self.aquifers_enabled }` 保留维度控制。
- 修复后 overworld 探针走 `new()` 默认 enabled=true（行为不变），nether 走显式 enabled=false。

### 教训（可复用判错经验）
- **加公共 struct 字段 = 结构性破坏点**：Rust 中给 struct 加非 Default 字段会让**所有 struct-literal 构造点**编译失败——这是「改一行数据结构，连带改全部调用点」的典型；批量改动前先 grep 所有 `StructName {` 构造点评估波及面。
- **默认值语义用显式构造器（`::new()`）收敛**：给 struct 加含默认值的字段时，优先提供 `::new()` 便捷构造器（默认值 + 参数化入口），让多数调用点只改 `StructName { f1 }` → `StructName::new(...)` 一处，避免在 20 个文件逐个写默认字段（重复 + 易漏）。
- **结构性连锁错误（E0063 / missing variant / 签名变更）用编译错误清单当「受影响点索引」**：错误列出的每个位置就是必须同步改的全集，不要手动回忆调用点（会漏）。改完字段后 MUST 全量编译，漏改的 probe 编译期即暴露，不留静默逻辑错。

---

## M3. nether 块级 match 仅 23.77%、y≥128 全 0%——noise_height 参数化停留在加载层，未进 fill_chunk/网格层

### 现象
- `multiworld_nether_blocks.rs`（fill_chunk_blocks vs vanilla nether 参照 WGB2 4×4@0,0 h256）首次跑出 **match 23.77%**；按 y 分带统计发现 **y≥128 四带全 0%**、y<128 带部分匹配——上半世界整体空白。
- overworld 基线 95.40% 正常，问题仅 nether。

### 根因（机制）
- nether.json 中 **noise.height=128 ≠ 世界高度 256**（噪声只在下半采样，上半本应留 Air）。M1/M2 的参数化把 min_y/height 读进了**加载层**（settings 解析、aquifer enabled），但 **`fill_chunk` 的 y 循环与宏观采样器网格仍用 world height（256）铺满**——采样层以为要算 256 高，噪声数据只有 128 高的语义 → ① y≥128 被噪声错误填充（应 Air）② y<128 采样网格错位（interp cell 布局按 256 高切，与 vanilla 128 高网格不对应）→ 双重偏差叠加出 23.77%。
- 本质与 M1 同族：**「维度参数从 settings 读」只做了一半**——加载层参数化了，执行层（fill 循环 / 采样网格 / est 循环）还写死单一高度。

### 定位（诊断链）
1. 23.77% + 「y≥128 全 0%」的**分带签名**直接指出垂直方向结构性错位（若是噪声值差，应全高度均匀低 match 而非上半全空）。
2. 读 `worldgen_handle.rs` macro_sampler 构建：网格高度用的是 world height 而非 `noise.height` → 确认执行层未消费 noise_height。
3. 对照 C++ 时代同坑修复记录（docs/09 修复链表：「y 循环上限 noiseHeight——y 128-255 留 air——22% → 72%」）——**C++ 踩过一模一样的坑**，症状数字都对得上（C++ 22% ↔ Rust 23.77%）。

### 修复
- `worldgen_handle.rs`：handle 存 `noise_height`（= settings noise.height，nether 128），与 world height（256）分开。
- `terrain.rs fill_chunk`：加 `noise_height` 参数，`y ≥ noise_top` 直接留 Air，不进采样。
- 宏观采样器网格：只铺噪声高度（interp cell 按 128 高切，对齐 vanilla nether 网格）；est 循环同步收窄。
- 连带修 13 个探针 bin 的 `fill_chunk` 调用点补 `noise_height` 参数（perf_quant/badlands_probe/biome_fill/blocks_cmp/carver_probe/fillbench/fillmap/fillprofile/grass_probe/mt_fill/surface_probe/terracotta_probe/beard_cmp）。
- 修后：nether match 23.77% → **73.77% → 74.04%**（y≥128 四带 0% → **100%**），**超过 C++ 时代 71.97%**；overworld 基线 95.40% 零回归。证据 `.investigations/multiworld-port/cmd-output/nether_blocks_match_v{1,2_noiseheight}.txt`。

### 教训（可复用判错经验）
- **「分带签名」是垂直结构错的指纹**：match 按 y 分带出现「上半全 0% / 下半部分匹配」= 高度语义错（应 Air 的被填了 / 网格错位），不是噪声值差（值差是全高度均匀低分）。看到分带先查高度参数流，别先怀疑噪声公式。
- **参数化必须贯穿全链路**：加载层读进 settings 的参数 ≠ 执行层在用——每个维度参数（min_y/height/noise_height/sea_level）要核对「从 JSON 读 → handle 存 → fill 循环 → 采样网格 → est」每一跳，断在任何一跳都等于没参数化。
- **C++ 时代已知坑清单（docs/09 修复链表）是移植 checklist**：C++ 踩过的坑 Rust 大概率原样再踩（本次 y 循环高度与 C++ 修复一字不差）——移植前逐条核对 09 篇修复链表，而不是重踩一遍再对号入座。
- **遗留差距（记录不修）**：熔岩海带 y=32..63（7.9%，流体填充缺失——C++ 时代也未解，见 09 篇 🔍 lava 项）；底部基岩错位（VerticalGradient 反锚序，C++ 有修复 Rust 未移植）。

---

## M4. nether features 两次运行漂移 2796 块——`BiomeClassifier` 用 HashMap，迭代序每进程随机

### 现象
- 修完 M3 后做确定性复验：同一 seed 同一坐标**两次运行 fill_chunk_blocks，结果差 2796 块**（nether features 区域）。overworld 同法复验逐位一致——问题只在走 features 放置的路径。

### 根因（机制）
- `biome.rs` `BiomeClassifier` 的 `features`/`carvers` 容器是 **`HashMap`**：`all_features_lists()` 每次 collect `HashMap.values()` 的顺序**随进程随机**（Rust HashMap 无序 + RandomState）。
- 该列表喂给 `PlacedFeatureIndexer` 编号——**迭代序变 → 每个 feature 的 index 变** → 放置顺序变 + decorator 的 index 参与 seed 派生 → **同 seed 同坐标放置结果运行间漂移**。
- Java vanilla 用的是注册表（确定性顺序）；Rust 侧随手用了 HashMap，把「容器选择」变成了「确定性破坏点」。

### 定位（诊断方法）
1. **skip 开关二分**：逐个关闭管线段复跑两次——carver no-op 后仍漂移、关 features 后逐位一致 → 漂移源锁定 features 放置。
2. 代码追 `PlacedFeatureIndexer` 的编号输入链：indexer ← `all_features_lists()` ← `HashMap.values()` collect——每进程迭代序不同，编号即不同。
3. 修复后两次运行逐位一致 → 根因坐实。

### 修复
- `biome.rs`：`BiomeClassifier` 的 `features`/`carvers` **HashMap → BTreeMap**（按键序确定迭代）。
- 修后两次运行**逐位一致**（2796 块差 → 0）。

### 教训（可复用判错经验）
- **「跨进程确定性」要求所有影响输出的容器迭代序确定——Rust `HashMap` 默认即不满足**（RandomState 每进程换种子）。**Registry 类容器（编号参与输出/seed 的）一律 `BTreeMap` 或 `Vec` + 显式排序**；HashMap 只允许用在纯查询、顺序不影响输出的场景。
- **「同一输入两次运行结果不同」= 确定性 bug，与数值精度 bug 完全不同族**：不要往浮点/精度方向查，直接二分管线段找「每进程会变的量」（迭代序 / 随机数种子 / 并发调度 / 地址依赖排序）。
- **indexer 编号是隐式输出**：给 feature/carver 编「放置顺序号」的任何 indexer，其输入容器顺序就是生成结果的一部分——审查确定性时把「编号来源」当数据流终点核对。

---

## M5. 游戏内 initDim 报 `UnsatisfiedLinkError` 但 target dll 是新的——Gradle `processResources` UP-TO-DATE 跳过 dll 同步

### 现象
- MOD 游戏内调 `CppBridge.initDim` 报 **`UnsatisfiedLinkError`**；但 target 目录下的 worldgen.dll **确实是含 initDim 导出的新版**——「dll 是新的但加载行为像旧的」。
- 相同 dll 用探针 bin 直接加载一切正常，只有 gradle 跑的 MOD 侧报错。

### 根因（机制）
- build.gradle 里 dll 同步写在 **`processResources` 的 `doFirst`** 里。该任务**输入未变时 Gradle 判 UP-TO-DATE 直接跳过——`doFirst` 的副作用拷贝不在任务输入集，跳过时同步不执行** → resources 里躺着**旧 dll**，运行时 `System.load` 加载旧导出表 → 无 initDim 符号 → UnsatisfiedLinkError。
- 本质：**「构建脚本里的副作用拷贝」没有声明为任务输入依赖**，被增量构建静默跳过——Gradle 增量模型与命令式 doFirst 的经典冲突。同批还修了一个 Mixin 侧隐患：`@Shadow` **够不到父类字段**（biomeSource 在父类 ChunkGenerator），末地保护分支改用**缓存反射**读取（附记：@Shadow 只作用于当前 mixin 目标类声明的成员，父类成员须反射）。

### 定位（诊断方法）
1. **三层 dll 导出表对照**：target/（构建产物）✅ 有 initDim → resources/（运行时实际加载）❌ 旧导出表 → 「目标层新、资源层旧」锁死同步链断点在 processResources。
2. 对照任务执行日志：processResources 标 UP-TO-DATE，doFirst 未执行 → 同步被跳过机制确认。
3. 运行日志 `UnsatisfiedLinkError: initDim`（而非 "no worldgen.dll"/路径错误）——符号级缺失，指向「加载了存在但版本旧的 dll」。

### 修复
- build.gradle：`processResources` 加 **`inputs.file(<dll 路径>)`**——dll 变化即任务失效重跑，doFirst 同步随之执行。
- 清掉 resources/ 陈旧 dll，全量重跑。
- 修后游戏内实证：`initNether enabled=true` + `[Mixin] populateNoise(nether) intercepted chunk(-1,-1)`（rust_nether_test4.log，摘录 `.investigations/multiworld-port/cmd-output/nether_ingame_intercept_20260830.txt`）。

### 教训（可复用判错经验）
- **「构建脚本里的副作用拷贝」必须声明为任务输入依赖**（Gradle `inputs.file`），否则增量构建静默跳过、产物层吃旧文件——任何写在 doFirst/doLast 里的拷贝/生成动作都要问一句「它的源文件在任务输入集里吗」。
- **「dll 是新的但行为像旧的」→ 立即做多层产物导出表对照**（构建产物 → 资源目录 → 运行时实际加载路径逐层解析导出表）——断点在哪一层一眼定位，不要在代码侧猜符号注册。
- **符号级 UnsatisfiedLinkError 排查序**：① 加载的到底是不是目标文件（路径+版本）② 该文件导出表有没有符号 ③ JNI 签名是否匹配——第 ② 步优先于第 ③（本例是 ②）。
- **@Shadow 父类字段不可达（附记）**：mixin @Shadow 只覆盖目标类自身声明的成员，父类字段须反射且**缓存 Field**（不要每 chunk 反射查找）。


## M6. nether 卡 74.04%、y32..63 带 7.9% 纹丝不动、legacy_random_source 零效果——JSON 布尔走 `as_f64()` 恒 None → `unwrap_or` 默认值静默生效（本轮最大根因）

### 现象
- nether 块级 match 卡在 **74.04%** 不再上升；按 y 分带：**y32..63 带仅 7.9%** 纹丝不动（熔岩海带带）、y0..31 也偏低。
- `legacy_random_source` 字段加了读取逻辑后**零效果**（legacy 分流从未激活）——「配置写了却不生效」的多字段聚簇。

### 根因（机制）
- `nether.json` 的 `"aquifers_enabled": false` 是 **JSON 布尔**；Rust 读取写的是：
  ```rust
  settings.get("aquifers_enabled").and_then(|v| v.as_f64()).map(|x| x != 0.0).unwrap_or(true)
  ```
- 自研 `json.rs` 的 `as_f64()` 只匹配 `JsonValue::Number`——**Bool 恒返回 None** → `and_then` 链断掉 → **`unwrap_or(true)` 的默认值静默生效**。
- 后果链：下界 aquifers_enabled 被错误当成 true → **下界被错误启用真实含水层** → 6.7 万块水（vanilla 是 air）。同款坑还埋着 `legacy_random_source`（默认 false 生效 → legacy 分流从未激活）和 feature.rs 的 `requires_block_below`。
- 本质：**「optional 读取 + unwrap_or 默认值」组合会把「字段类型不匹配」静默吞成默认行为**——不是「字段缺失」，是「字段在但类型读不到」，代码却按缺失处理走默认值，且默认值方向还恰好与 JSON 真实值相反（false → true）。

### 定位（诊断链）
1. **混淆对直方图（got→want Top 配对）**暴露 `id32=water` 大规模聚集——错误填充的是整层水，指向流体/含水层机制而非噪声值差。
2. **skip 开关二分**锁 stage：跳过 aquifer/流体相关阶段复跑 → 差异消失 → 锁定 stage 1（fill）内的流体填充路径。
3. 反查 classify 分支条件 → aquifer 启用状态的判定输入不对 → 下钻到 JSON 解析层 → 发现 `as_f64()` 对 Bool 恒 None、`unwrap_or(true)` 兜底。

### 修复
- `json.rs` 增加 **`as_bool()`**（Bool 直接读；Number 兼容 `!= 0`）。
- 三处读取（`aquifers_enabled` / `legacy_random_source` / feature.rs `requires_block_below`）由 `as_f64().map(!=0.0)` 改为 `as_bool()`。
- 修后：nether **74.04% → 82.69%**（y32..63 7.9% → **65.8%**，y0..31 59.5% → 79.6%）；overworld 95.40% 零回归。

### 教训（可复用判错经验）
- **「optional 读取 + unwrap_or 默认值」是静默默认值陷阱的标配组合**——默认值必须显式断言类型（读取后打一行日志或 assert 类型），新 JSON 字段接入时验证「读到的是什么」而不是「默认值是什么」。此坑跨语言跨项目通用（任何 self-parsed JSON/配置——Rust/Java/C++ 手写 parser——都会踩），已单独立 discovered 条目（见 knowledge/discovered/build-tooling.md 草稿）。
- **「多个配置字段同时零效果」是解析层错的聚簇签名**——单字段不生效可能是逻辑错，多字段同时「写了没反应」先怀疑共同的解析/读取层，不要逐字段查逻辑。
- **判错路径可复用**：块级混淆对直方图（got→want）定位「错的是什么」→ skip 二分定位「错在哪一段」→ 分支条件反推「输入状态错在哪」→ 才下钻解析层。层层收敛，不直接跳 JSON parser。

---

## M7. 下界熔岩的真正来源——aquifers_enabled=false 时走 `AquiferSampler.seaLevel()` 匿名实现（源码确认机制，docs/09 旧猜测证实）

### 现象
- M6 修复后 nether 熔岩仍与 vanilla 有残差——需要弄清 vanilla 下界熔岩的确切生成机制（docs/09 此前仅有「可能来自 fillFromNoise」的猜测）。

### 根因（机制，Java 源码确认）
- `aquifers_enabled=false` 时 `ChunkNoiseSampler` 用 **`AquiferSampler.seaLevel()` 匿名实现**（不是关闭 aquifer 就没有流体来源）：
  - `density > 0` → 返回 null（填 default_block）；
  - `density ≤ 0` → `FluidLevel(sea_level, default_fluid).getBlockState(y)` = **`y < sea_level ? lava : air`**（严格 `<`；无噪声参与；无上下界概念）。
- `buildSurface` **跳过流体格**（SurfaceBuilder L136 只记录液面、不应用表面规则）——表面规则不会覆盖熔岩。
- docs/09 旧猜测「熔岩来自 fillFromNoise」「buildSurface 跳过流体格」**均证实**（前者即 sea_level 实现，在 noise 填充阶段内生效）。

### 定位（诊断方法）
- 直接读 Java 源码（yarn sources）`ChunkNoiseSampler` / `AquiferSampler` / `SurfaceBuilder`，逐条落证据——机制类问题源码是权威，不做猜测性实验。

### 修复
- Rust 侧 `VanillaAquifer` 加 `sea_level`（从 settings 数据驱动读取）；`!enabled` 分支实现同语义：`y < sea_level → Lava else Air`。
- 修复后 nether 82.69%（y 分带详见 docs/08 增补段）；熔岩海带带 7.9% → 65.8% 的主要贡献源。

### 教训（可复用判错经验）
- **「开关关闭 ≠ 机制消失」**——vanilla 里 `aquifers_enabled=false` 不是「不跑流体逻辑」，而是**切换到 sea_level 简化实现**；移植开关语义前必须读 false 分支的实际实现，不能按名字直觉理解。
- 机制类定论（本条）由源码逐条证据支撑并落盘 `.investigations/multiworld-port/analysis-nether-lava-mechanism.md`——旧 docs 猜测（docs/09 🔍 lava 项）得以证实/结案，猜测→验证链条闭环。

---

## 附记（worker 发现，简记，未修单开课题）

1. **Hole 语义 Rust/Java 不一致**：Rust `SurfaceCond::Hole` 用 `surface_depth <= 0`；Java `HoleCondition` = `stoneDepthAbove <= 0`（C++ L251 写法才对）——Rust 侧注释声称「对齐 Java runDepth」是错的。影响 nether 的 lake/not(hole) 门控。未修，单开课题。
2. **三个已登记隐患**：① mixin `@Shadow` 够不到父类字段（biomeSource 在 ChunkGenerator）→ 用**缓存反射**（已用于末地保护，M5 附记同源）；② `parse_surface_rule` 未知 cond 走 `?` **静默吞掉整条分支**；③ surface rule 解析失败回退 `Block(0)` 会**写 id 0 进输出**——两者待加告警（静默降级是后续排查的隐形坑）。

---



---

## M8. Hole 条件「历史误判翻转」：当年注释声称「C++ 用错字段」，worker 源码核对证明 **C++ 才是对的**（JSON 布尔修复后顺带定案）

### 现象
- multiworld 收尾轮，worker 静态核对 nether surface 规则链时发现：Rust `surface_rules.rs` `SurfaceCond::Hole => ctx.surface_depth <= 0`，且注释**主动声称**「对齐 Java runDepth=sampleRunDepth 噪声；不照抄 C++ L251 的 `stoneDepthAbove <= 0`（C++ 用错字段，bug）」——即当年**有意**选择了与 C++ 不同的字段。
- nether 82.69% 阶段该条件的实际影响小（修后仅 +0.03pp），但「注释指导后人远离正确实现」的误导性是主要风险。

### 根因（机制）
- Java `MaterialRules.HoleMaterialCondition`（yarn `hole()`）：`return this.context.stoneDepthAbove <= 0;`——用的是 `initVerticalContext` 第一参的**垂直扫描 stoneDepthAbove**。
- C++ `surface.h` L251 `stoneDepthAbove <= 0` **与 Java 一致**。
- Rust 当年却用了 `surface_depth`（`sampleRunDepth(x,z)` 2D 噪声），并在注释中断言 C++ 是 bug——**对 Java 源码的核对当年没有做或做错了**，凭「名字像 runDepth」的直觉下了反转结论。
- `hole` 参与的规则：nether 熔岩 lake 判定、nether_wastes soul_sand/gravel 门控、overworld 水湖/熔岩湖边缘（`not(hole)` 门控）。

### 定位（怎么发现的）
- 非 hole 专项排查发现：multiworld 收尾轮 worker 做 nether surface 规则链静态核对时（M6 修复后的基岩/表面残差诊断），顺手核对 Java `HoleMaterialCondition` 源码 → 与 Rust 注释声称的语义矛盾 → 逐行确认 Java 用 `stoneDepthAbove`。
- 证据：`.investigations/multiworld-port/analysis-nether-bedrock-misalignment.md` §C（worker 源码核对）+ Java `MaterialRules.HoleMaterialCondition` 源码。

### 修复
- `surface_rules.rs`：`SurfaceCond::Hole => ctx.stone_depth_above <= 0`（对齐 Java/C++），注释更正。
- 修后：nether 82.69% → **82.72%**（微升，hole 在 nether 门控面小）；overworld 95.40% 零回归（hole 参与 badlands/湖缘，改动在 FULL 口径噪声内）。

### 教训（可复用判错经验）
- **「注释声称对齐 Java 但给出与 C++ 不同的语义」= 高危信号**：两个移植实现语义不同时，必有一错——正确动作是**当场读 Java 源码裁决**，而不是写注释为自己这边辩护。本条注释不但没触发核对，还主动把正确实现（C++）标记成 bug，误导持续至今。
- **历史误判的残留载体是注释，不是产物**：产物错会被测试抓，注释错只会在下一次「照注释实现」时复发——审查移植代码时，**对 Java 语义的断言注释要与源码抽查**，优先级高于对实现的抽查。
- **微小的差异也可能是语义分歧的暴露点**（+0.03pp 修正确认方向），不要因「影响小」跳过语义核对。

### 速查表追加 1 行
| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| Rust Hole 用 surface_depth，注释称「C++ 用错字段是 bug」（M8） | 历史误判翻转：Java `HoleMaterialCondition = stoneDepthAbove <= 0`，C++ L251 正确，Rust 当年用 runDepth 噪声且注释反指 C++ 为 bug；修后 nether 82.72%/overworld 零变化 | **两个移植实现语义冲突 = 必有一错，当场读 Java 源码裁决**；「注释声称对齐 Java」不等于核对过——对语义断言注释的源码抽查优先于实现抽查 |




---

## M9. legacy climate visitor 固定种子特例：逐语义正确移植后净效果为负（82.72% → 77.01%）——env 门控回退保留

### 现象
- WG_LEGACY_CLIMATE=1 启用 legacy climate visitor 特例（temperature/vegetation 固定种子 CheckedRandom(0)/(2)、offset 恒 0、blended CheckedRandom(0)）后 nether **82.72% → 77.01%**（y32..63 暴跌 65.78→22.37）；y96..127 61.03→65.64、y64..95 55.17→55.51 微升；nonAir 63.3→70.5 提升。

### 根因（机制）
- visitor 替换改变 climate 噪声 → biome 判定改变 → nether 3D 表面规则涂布（biome 条件 x5）连锁变化。
- **Rust 的 biome 判定本身仍是 nether_wastes 误判**（soul_sand 诊断，M10 链条）→ 正确的 climate × 误判的 biome = 涂布结果更差（负相关抵消）。

### 定位（诊断方法）
- 特例态消融（WG_LEGACY_CLIMATE=1 + WG_BIOMEDUMP=1）：t=+0.127~+0.150（**符号已对**——确认 legacy climate visitor 就是 biome 输入的真实机制）但 humidity ≈-0.01 vs Java -0.1533 → OctavePerlin createLegacy 构造/采样语义未对齐（yarn OctavePerlinNoiseSampler.java 已入档，静态核对为下轮开工点）。

### 修复
- 特例已实现并保留（env 门控默认关）：get_noise_sampler 特例（temperature/vegetation）+ old_blended_noise legacy 分支（CheckedRandom(0) 替代 split("terrain")）。

### 教训（可复用判错经验）
- **逐语义正确 ≠ 整体正确**——visitor 替换是全局耦合改动（climate→biome→表面三层连锁），必须消融验证子项而非直接看总分。
- env 门控回退 = 保留已实现工作 + 维持最佳默认态的标准做法。
- 净负结果不是白做——澄清了「legacy 下界的 worldSeed 无关性」疑点与 biome 采样的耦合关系。

## M10. 三层对拍校准：LCG/blended 全对齐，缺口隔离到 OctavePerlin createLegacy（humidity≈0 vs Java -0.16）

### 现象
- M9 门控回退后（默认 82.72%），soul_sand 残差（biome 判定 nether_wastes vs vanilla soul_sand_valley）根因未闭环。
- BIOME6（Java router 直采，yarn NoiseRouter）@ mismatch 坐标 y=1：**t=+0.077~+0.119（正）、h=-0.149~-0.175、c/e/d/w=0**。
- Rust 同坐标（无特例）：**t=-0.115（负）、h=-0.092**——temperature 符号相反 + humidity 幅度差 → biome 判定错（Rust h=-0.092 落 nether_wastes 盒，Java h≈-0.16 落 soul_sand_valley 盒）。

### 根因（三层对拍定位，各层独立验证）
- **S1 层（LCG 裸输出）**：LegacyRandom(0) 的 next(32)×8 / nextLong×4 / nextDouble×3 与 Java CheckedRandom(0) **逐位一致** → LCG 实现无错，排除。
- **S2 层（blended Octave 构造）**：new_legacy(-15,[1×16])×2 + (-7,[1×8]) 的 16+2 个 Octave origin 与 Java createLegacy **一致**（Java 打印序为反转方向，数值按随机消耗序对齐；尾数 ~4e-6 在 f32 打印噪声级）→ Octave legacy 构造的随机消耗序无错，排除。
- **S3 层（blended 采样）**：DoublePerlin legacy @ y=1/y=52 六个 mismatch 列与 Java **一致到 ~6e-6**（f32 噪声级）→ blended（old_blended_noise）不是 nether 密度形状差的来源。⚠️ 注意口径：S3 对拍的是 climate DoublePerlin(-7,[1,1])，blended Octave(-15) 的**采样**未单独对拍（见遗留）。
- **S4 层（router 组装/消融）**：WG_LEGACY_CLIMATE=1（特例启用）时 Rust t=+0.127~+0.150（**符号已对**——确认 legacy climate visitor 就是 biome 输入的真实机制）但 **humidity ≈-0.01 vs Java -0.16**；同时总分 82.72→77.01（y32..63 暴跌 65.8→22.4）。
- **缺口定位**：humidity ≈0 vs -0.16 → `DoublePerlinNoiseSampler::new_legacy`/`OctavePerlinNoiseSampler::new_legacy`(-7,[1,1]) 的**构造/采样语义**与 Java createLegacy（yarn `OctavePerlinNoiseSampler.java`，已入档 `.investigations/multiworld-port/OctavePerlinNoiseSampler.java`）有未对齐细节；temperature 同源 +0.05 偏差同因。四层排除后唯一不一致层即缺口所在——**Octave createLegacy 的采样语义**（非构造、非 LCG）。

### 定位（诊断方法——分段对拍设计，可复用）
1. **分段设计**：裸 LCG 输出（S1）→ 单 Octave 构造产物（S2）→ 复合采样（S3）→ router 集成消融（S4），逐层排除——每层一致即排除一层，最后不一致层就是缺口。本例三层全对齐后锁定 Octave createLegacy 采样语义。
2. **「一致性判据」定义精度口径**：~6e-6 的 f32 噪声级一致算「对齐」（两侧都有 f32 乘法路径，打印尾数必然抖动）；超出该量级即真差异。没有口径定义，「0.128817 vs 0.1288179」会被误判为不一致或漏判真差异。
3. **消融开关（WG_LEGACY_CLIMATE）+ 分带混淆对 + 6 维直采（BIOME6）三件套组合**：单轮即把「总分下降」拆到「哪一维（humidity）、哪一段（Octave 采样）、偏差多少（≈0 vs -0.16）」。S4 符号翻转（负→正）还顺带**确认了机制归属**——legacy climate visitor 确实是 biome 判定输入的真实路径。
4. Java 参照探针（BIOME6，yarn NoiseRouter 直采 @ 相同坐标）是校准的权威侧——没有它，每一步都是盲调。

### 修复
- **未修**（Octave createLegacy 语义静态核对为下轮开工点；yarn 权威源码已入档 `.investigations/multiworld-port/OctavePerlinNoiseSampler.java`）。
- 特例保持 **WG_LEGACY_CLIMATE 门控（默认关）**，82.72% 最佳默认态不受影响（M9 处置延续）。

### 教训（可复用判错经验）
- **对拍校准的分段设计**：裸随机源 → 单 Octave 构造产物 → 复合采样 → router 集成，逐层排除——每层一致即排除一层，最后不一致层就是缺口。逐层排除把「一个大差异」拆成「唯一缺口层」，比在整链上盲调快一个量级。
- **「一致性判据」要先定义精度口径**：f32 打印路径两侧都有乘法噪声，~6e-6 级一致算对齐；口径不定义，对拍结论本身不可靠。
- **消融开关 + 分带混淆对 + 6 维直采三件套**，单轮即可把总分下降拆到「哪一维、哪一段、偏差多少」；符号翻转本身就是机制归属的证据（符号都随开关翻转 → 该开关就是该输出的机制）。
- **Java 参照探针是校准权威侧**——没有 Java 侧同坐标直采，Rust 侧一切采样值都没有「对错」参照。

### 遗留（下轮开工点）
1. **OctavePerlin createLegacy 构造/采样逐行对照**（yarn `OctavePerlinNoiseSampler.java` 已入档 vs Rust `new_legacy`/Octave `sample`）——重点：legacy Octave 的 amplitudes 展开、permutation 消耗、sample 的 y smear/lacunarity 语义。
2. 修好后 **WG_LEGACY_CLIMATE 默认开启**，预期 humidity 对齐 → soul_sand 残差解决 → nether 冲 90%+。
3. bedrock roof 缺失（混淆对 `netherrack→bedrock 12195`@y96..）单独排查。
4. S3 层 blended Octave(-15) 的**采样**（非构造）未单独对拍——S3 只证明了 climate DoublePerlin(-7,[1,1]) 采样对齐，blended 密度形状差的排除不完整。

---

## 速查表追加 1 行（插表末）

---

### M10 补遗：第二轮对拍（blended 逐 octave + router 矛盾隔离，2026-08-30 深夜）

- **blended Octave 对拍（一致 ✓）**：Rust `new_legacy(LegacyRandom(0), -15, [1×16])` 的 16 个 Octave origin 与 Java `createLegacy(CheckedRandom(0), -15, [1×16])` 逐值一致（按消耗序；打印序差为 getOctave 映射方向）；blended 采样 y=1/y=52 与 Java 一致到 ~6e-6（f32 噪声级）。**blended（old_blended_noise）排除出 nether 密度形状差来源**。
- **S1/S3 复核**：LCG next(32)×8/nextLong/nextDouble 逐位一致；DoublePerlin createLegacy(CheckedRandom(0),(-7,[1,1])) @ 10 坐标一致到 ~5e-6。
- **矛盾收窄**：router.temperature/vegetation 直采（Java +0.0775/-0.1533 @ (12,1,0)）与「同构造直线采样」（Rust +0.1435/-0.010、Java CAL-S3 0.143525 ✓ 双方一致）**不一致** → 差异不在噪声构造层，在 **router 装配链**（temperature 的 ShiftedNoise 包装：shift_a/shift_b 的 offsetNoise 装配源 + wrapper 层 + visitor 时序）。
- 证据：`legacy_calibrate_rust_v3.txt` / `legacy_calibrate_java_oct.log` / `biome6cal3.log`（cmd-output/）。
- **下轮专项**：读 NoiseConfig.java 全文（已入档）的 shift_a/shift_b offsetNoise 装配链 + wrapper 层 + visitor 时序，定位 router.temperature 与直线特例构造的装配差异。5e-6 级残差按 f32 口径视为一致（两侧同走 f32 乘法路径）。


## M11【复发·第 3 次】跨工具对比 worldSeed 错位——烧掉整条「Octave createLegacy 缺口」结论链

### 现象
- BIOME6（Java yarn NoiseRouter 直采，跑在 run/server world，seed=**-2032795982907864146**）vs Rust multiworld_nether_blocks 探针（seed=**-8248318472910187742** 参照 seed）对比 6 维 climate 值（@ mismatch 同坐标）：
  - 「temperature 符号相反」：Java **+0.0775** vs Rust **-0.115**
  - 「humidity 完全错」：Java **-0.1533** vs Rust **≈0**
- 由此推导出的结论链：①「legacy climate visitor 是唯一对齐路径」②「OctavePerlin createLegacy 存在数值缺口」（M10 定论）③「v7 净负 = visitor 替换连锁」——**部分建立在错误对比上**（两侧根本不是同一个世界）。

### 根因
**跨工具对比用了不同 worldSeed**。派生噪声（`randomDeriver.split(temperature/vegetation/...)`）与 worldSeed 相关 → seed 不同则派生噪声值完全不同 → 「t 符号相反 / h 归零」全是 seed 错位假象，与 Rust 实现无关。固定种子特例（CheckedRandom(0)/(2)）不受 worldSeed 影响，所以 M9/M10 中「特例态符号一致」的结论碰巧成立——**混合正确与错误证据导致定位被带偏多个循环**（M10 四层对拍本身没问题，但对比基准侧的 seed 就是错的）。

### 定位（怎么发现的）
- Java 侧 ShiftedNoise 递归树 dump（BIOME6 探针反射，参数类型 NoisePos 匹配）：shiftX/Y/Z 全恒 0（OFFSET 特例生效 ✓）+ `noise.sample(3,0,0)=0.0919` 是派生值 → 逐层核对派生输入时发现两侧 worldSeed 不一致（-2032795982907864146 vs -8248318472910187742）。
- 诊断路径仍是「顺着采样链逐输入核对」——与 M10 分段对拍同思路，只是核对对象从噪声构造换到派生输入；核对到 worldSeed 这一层才暴露错位。

### 修复
- Rust 探针加 **`WG_SEED` env**（默认保持参照 seed，不改历史行为）→ 同 seed（-2032795982907864146）重比：
  - t：Java +0.0775 / Rust **+0.171**（残差 0.094，**符号一致**）
  - h：Java -0.1533 / Rust **-0.187**（残差 0.034，**量级一致**）
- 修正后的格局：
  - vegetation 采样链基本对齐（h 残差 0.034 在量级一致范围）；
  - temperature 残差 0.094 待查（候选：noise_params 表 vs 注册表 / shift_a 细节）；
  - soul_sand 残差定性重估：同 seed 下两边最近邻都判 nether_wastes → vanilla soul_sand 块**可能来自 features 层**而非表面/biome 层（新方向）。

### 教训（⚠️ 重点标记：复发错误——seed/坐标错位类第 3 次）
- **这是 seed/坐标错位类错误第 3 次复发**：
  1. ① #23/#24 采样坐标错位（-337 vs -336/-340，误判「湿度差 0.0054」）；
  2. ② 参照 seed 与 server.properties level-seed 混淆；
  3. ③ 本次跨工具 worldSeed 错位（Java 探针跑 server world / Rust 探针用参照 seed，两套 seed 无人核对）。
- **铁律早已存在**（AGENTS「探针/参照采集核对铁律」seed 三查）**但没被套用到「Java↔Rust 跨工具数值对比」场景**——三查约束的是参照导出流程，没约束「对比动作本身」。修正：**任何 Java↔Rust 数值对比的第一动作 = 核对两侧 worldSeed 一致**（在对比之前，不是得出怪结论之后）。
- **可复用判据**：跨工具对比出现「符号翻转 / 量级级（非残差级）差异」时，**先怀疑 seed/坐标错位，再怀疑实现**——seed 错位产生的差异恰好长得像实现 bug（符号反、量级差），与残差级偏差（0.03~0.09）完全不同族。
- **对比工具链应强制 seed 自检**：探针输出行自带 seed 并由对比脚本核对——人眼核对不可靠，已三犯。

### 遗留（被 seed 错位污染的旧结论逐条复核）
1. 「t 符号相反」（M10 现象）——**已反转**（同 seed 符号一致）。
2. 「OctavePerlin createLegacy 数值缺口」（M10 定论）——S3 对拍是**同 seed 固定种子**（CheckedRandom(0)）下做的，**仍成立**；但「湿度缺口」的定性需按新残差（h 0.034 量级一致）重估。
3. 「legacy climate visitor 净负」（M9）的连锁解读——需按修正后格局重审（正确的部分哪些来自 seed 假象、哪些真实）。
4. temperature 残差 0.094 具体差异（noise_params 表 vs 注册表 / shift_a 细节）——未查。
5. soul_sand 块来自 features 层假设——未验证。

---



## M12. legacy temperature 噪声种子源定案（try-seed sweep）：**worldSeed** 而非 sources 字面 0L；t 残差 0.094 → 0.0005

### 现象
- M11 seed 修正后同 seed（-2032795982907864146）对比：
  - t(12,1,0)：Java **+0.0775** vs Rust **+0.171**（残差 0.094）；
  - h：Java **-0.1533** vs Rust **-0.187**（残差 0.034）。
- 此时 shift 两边已实测恒 0（OFFSET 特例生效）、派生链 split 已对拍逐位一致、参数表一致——**残差来源待定**（M11 遗留第 4 条：noise_params 表 vs 注册表 / shift_a 细节，均被后续排除）。

### 根因（机制）
- yarn sources 的 `NoiseConfig` 源码字面写 `createLegacy(this.createRandom(0L), ...)`——但**运行时行为实测种子源 = worldSeed**。即：**sources 字面与运行行为不符**（真实装配链的种子传递路径与 sources 所示不同——具体传递机制待查，见遗留第 3 条）。
- Rust 之前**按 sources 字面 0L 实现** legacy temperature 特例 → 种子源错 → 采样值错 → t 残差 0.094。

### 定位（三层递进，每层有数据）
1. **ShiftedNoise 递归树 dump**（Biome6Probe 反射，参数类型 **NoisePos**——yarn 1.20.1 接口是 NoisePos 非 FunctionContext，反射时别按旧名找）：shiftX/Y/Z 全恒 0（OFFSET 特例生效 ✓）；temperature noise 是 ShiftedNoise(RegistryEntryHolder 包装) 内的 `DensityFunction.Noise` record。
2. **Noise record 反射（关键判读点）**：`noiseData` 字段显示注册表参数 **(-10, [1.5,0,1,0,0,0])**，**但实际 firstSampler 只有 2 个非空 octave = (-7, [1,1]) 特例形状** → record 的 noiseData 保留原引用、sampler 已被 visitor 替换为特例——「**params 字段 ≠ sampler 实际来源**」。
3. **S8 try-seed sweep（定案手段）**：Rust 构造 `createLegacy(LegacyRandom(seed), -7, [1,1])`，seed ∈ {0, worldSeed, worldSeed×2} 三候选对照——**seed=worldSeed 的 origins [21.877382, 5.365246, 138.552887] / [47.402641, 28.663731, 67.151535] 与 Java router 实测逐位一致** → 种子源 = worldSeed，定案。

### 修复
- `density_builder.rs get_noise_sampler`：legacy 下 `minecraft:temperature` → `DoublePerlinNoiseSampler::new_legacy(RsRandom::Legacy(LegacyRandom::new(world_seed)), -7, &[1.0, 1.0])`（种子源改 worldSeed）。
- 验证：t(12,1,0) Rust **+0.078** vs Java **+0.0775**（残差 **0.0005**，f32 精度级）；nether **82.72%** 中性；overworld **95.40%** 零回归。

### 教训（⚠️ 重点）
- **「同模式类推」翻车**：vegetation 套用 worldSeed 同款 → h=+0.078 与 t 完全相同（**同种子同参数输出必然相同**），而 Java t≠h → 立即暴露 vegetation 种子源不同 → **已回滚**（vegetation 保持派生，残差 0.034 更小）。**种子源必须逐维 try-seed 实测定案，不能类推。**
- **record 字段与实际来源可以不一致**：Noise record 的 noiseData（注册表参数）≠ sampler 实际构造来源（visitor 特例）——「打印字段值」可能误导，**「字段形状 + try-seed 行为对照」才是定案手段**。
- **sources 字面 ≠ 运行时行为**：yarn sources 与真实装配链存在版本/路径差——**实测行为优先级高于源码字面**（与「禁止直接信任 javap」同族的源码层告诫，多一条对照手段：try-seed）。

### 遗留（下轮开工点）
1. ✅ **vegetation 种子源已定案（2026-08-31 追加）**：worldSeed+1（S9 sweep 命中，origins 逐位一致）——已实施，h(12,1,0) = -0.153 vs Java -0.1533（残差 0.0003）✓；t/h 双维全部收敛。**总分 82.72→82.51（-0.21pp）**：t/h 修正改变部分 biome 判定边界的暂态，采样值正确性优先，其余层（features/soul_sand）修复后总分跟上。
2. h 收敛后 **soul_sand 残差重估**（biome 判定边界变化）。
3. 「sources 0L vs 运行时 worldSeed」**机制待查**（NoiseConfig 真实运行版本 vs yarn sources build.10 v2 的差异点）。

---


### M12 补遗二：同 seed 竖切定案——cell 角点全对齐，「density 残差」大部分是工具语义陷阱（2026-08-31，status: candidate）

## A 部分：追加小节正文

---

> status: candidate（数据源：cmd-output/nether_density_rust_same_seed.txt、nether_confusion_v3_postfix.txt）。
> `## 附：错误 → 根因 速查表` 之前；② 「B 部分」表格行追加到速查表末尾。
> `multiworld-errors.md` 的 M12 节（`## M12. legacy temperature 噪声种子源定案…`）之后、
> 产出者：knowledge 落盘 subagent（2026-08-31）。主会话应用方式：① 下面「A 部分」整段插入

| 同 seed 竖切 density 残差：cell 角点全 0、中间点 ±0.01~0.08、y≥128 恒值不饱和（M12 补遗二） | **工具语义陷阱**：Java DensityProbe 纯 `df.sample` 直采 vs Rust 竖切走生产插值路径——两侧采样语义不同，中间点/y≥128 的「残差」是语义差非实现错；旧「残差随 y 增长」结论 = seed 错位（M11）+ 语义差复合假象，已推翻 | **跨工具对比先声明两侧采样语义（纯函数/插值/网格）**——「⚠️ 探针采集核对铁律」的采样语义扩展：seed 一致 → 采样语义一致 → 逐节点公式，三层递进，前两层收敛就不进第三层；**cell 角点（插值端点）对齐 = 形状主体对齐的充分判据**，中间点差优先怀疑语义 |



---

## A 部分：追加小节正文

> 主会话应用方式：① 下面「A 部分」整段插入 **M12 补遗二 之后、`## 附：错误 → 根因 速查表` 之前**；② 「B 部分」表格行追加到速查表末尾。
> 本条是同日后续，且是本文件此前所有 nether 对齐工作的收口性突破。
> 产出者：knowledge 落盘 subagent（2026-08-31）。数据源：主会话 M13 实测记录（nether 82.51%→96.0568%、bedrock 残差 10288→4011、分带数据）。

# draft-multiworld-errors-m13 —— M13 追加 multiworld-errors.md（2026-08-31，status: candidate）
| nether 卡 82%、多带同时残差（y32..95 低至 55~66%）、bedrock roof/floor 整层缺失（混淆对 10288@y96..）（M13） | surface_rule JSON 数据驱动解析器不支持 `vertical_gradient` 条件类型 → nether.json 的 bedrock_floor/bedrock_roof 分支解析返回 None 被**整条静默跳过**（仅一行 WARN 被海量警告淹没）→ 顶部/底部 bedrock 层及规则链涂布全缺；附带：`below_top(N)` 的 height 误传 world_height(256) 致 roof 锚越界（须用 noise_height 128） | **「多带同时残差 + 特定块类缺失」优先查规则解析覆盖率，不逐带调参**——数据驱动解析器静默跳过分支是高危反模式，新数据文件接入 MUST 断言 [PARSE-WARN] 计数为 0，不能只看总分（总分被其它层掩盖，+13.5pp 一步修复）；**锚换算的 height 用逻辑生成高度（nether 128），不混用 world_height 256**（M3 同族） |


## 附：错误 → 根因 速查表（一页索引）
| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| nether 加载 panic `unresolved density function ref: minecraft:nether/base_3d_noise`（M1） | `DensityBuilder::resolve_ref` 惰性按需加载**硬编码** `key.starts_with("minecraft:overworld/")` + `key["minecraft:overworld/".len()..]`；nether 引用 `minecraft:nether/` 前缀不匹配 → 落 panic。命名空间写死，未参数化维度 | **多世界参数化 = 命名空间/维度数据不要硬编码**；`"minecraft:overworld/"` 字面前缀/目录都是 single-world 假设。panic「unresolved ref」先判「前缀是否被识别」再查「文件是否缺失」——前者是结构错后者是环境错。lazy-load 前缀 + `density_function/<dfNs>/` 目录是两处同源，须同步参数化 |
| `VanillaAquifer` 加 `enabled` 后约 20 个 probe 报 `error[E0063] missing field 'enabled'`（M2） | Rust struct literal 逐字段构造；给 struct 加**非 Default 的 pub 字段**后所有 `VanillaAquifer { aq }` 构造点全失效（无隐式默认值）——**数据结构字段变更 = 全体构造点结构性连锁破坏** | **加公共 struct 字段是结构性破坏点**：改前 grep 所有 `StructName {` 构造点评估波及面；默认值语义用显式 `::new()` 构造器收敛（一处 `new()` 取代 20 处手动补默认字段）；E0063 编译错误清单 = 受影响点全集，改完 MUST 全量编译（漏改编译期即暴露） |
| nether match 23.77% + y≥128 全 0%（M3） | noise_height 参数化停留在加载层；fill_chunk y 循环与宏观采样器网格仍按 world height 256 铺（噪声只有 128 高语义）→ 上半被错填 + 下半网格错位 | **「上半全 0%」分带签名 = 垂直结构错**，先查高度参数流不查噪声公式；参数化须核对「JSON→handle→fill→网格→est」每一跳；**C++ 已知坑清单（docs/09 修复链表）是移植 checklist**（C++ 22%↔Rust 23.77% 同坑） |
| 两次运行差 2796 块（M4） | `BiomeClassifier` features/carvers 用 HashMap，`all_features_lists()` 每进程随机迭代序 → PlacedFeatureIndexer 编号随机 → 放置序 + decorator seed 变 | **跨进程确定性要求所有影响输出的容器迭代序确定——Rust HashMap 默认不满足，Registry 类容器一律 BTreeMap/Vec 排序**；「同输入两次运行不同」= 确定性 bug，与精度 bug 不同族，二分管线段找「每进程会变的量」 |
| initDim UnsatisfiedLinkError 但 target dll 是新的（M5） | processResources 的 doFirst 同步不在任务输入集 → UP-TO-DATE 跳过时同步不执行 → resources 留旧 dll。附记：@Shadow 够不到父类字段 → 缓存反射 | **构建脚本里的副作用拷贝必须声明为任务输入依赖（inputs.file）**，否则增量构建静默跳过；「产物新行为旧」→ 三层导出表对照（target/resources/tmp）定位断点；@Shadow 只覆盖目标类自身成员 |
| nether 卡 74.04%、y32..63 带 7.9% 不动、legacy_random_source 零效果（M6） | `nether.json` 的 `false` 是 JSON 布尔；Rust 走 `as_f64()`（只匹配 Number，Bool 恒 None）→ `unwrap_or(true)` 默认值静默生效 → 下界被错误启用真实含水层（6.7 万块水 vs vanilla air）。同款坑：legacy_random_source、requires_block_below | **「optional 读取 + unwrap_or 默认值」会把「字段类型不匹配」静默吞成默认行为**——默认值必须显式断言类型/打日志验证「读到的是什么」；多配置字段同时零效果 = 先查共同解析层；判错路径：混淆对直方图 → skip 二分 → 分支条件反推 → 才下钻解析层 | | 下界熔岩分布与 vanilla 残差（M7） | `aquifers_enabled=false` 时 vanilla 用 `AquiferSampler.seaLevel()` 匿名实现：density≤0 → `y < sea_level ? lava : air`（严格 <，无噪声）；buildSurface 跳过流体格（SurfaceBuilder L136）。docs/09 旧猜测均证实 | **「开关关闭 ≠ 机制消失」**——false 分支是切换到简化实现而非跳过，移植前必读 false 分支实际源码；机制定论以源码逐条证据落盘（analysis-nether-lava-mechanism.md），猜测→验证闭环 |
| 下界熔岩分布与 vanilla 残差（M7） | `aquifers_enabled=false` 时 vanilla 用 `AquiferSampler.seaLevel()` 匿名实现：density≤0 → `y < sea_level ? lava : air`（严格 <，无噪声）；buildSurface 跳过流体格（SurfaceBuilder L136）。docs/09 旧猜测均证实 | **「开关关闭 ≠ 机制消失」**——false 分支是切换到简化实现而非跳过，移植前必读 false 分支实际源码；机制定论以源码逐条证据落盘（analysis-nether-lava-mechanism.md），猜测→验证闭环 |
| Rust Hole 用 surface_depth，注释称「C++ 用错字段是 bug」（M8） | 历史误判翻转：Java `HoleMaterialCondition = stoneDepthAbove <= 0`，C++ L251 正确，Rust 当年用 runDepth 噪声且注释反指 C++ 为 bug；修后 nether 82.72%/overworld 零变化 | **两个移植实现语义冲突 = 必有一错，当场读 Java 源码裁决**；「注释声称对齐 Java」不等于核对过——对语义断言注释的源码抽查优先于实现抽查 |
| legacy climate 特例逐语义移植后 nether 82.72% → 77.01%（y32..63 暴跌 65.78→22.37，两带微升，nonAir 提升）（M9） | 实现**逐语义正确**（源码逐条核对排除移植错）；净负 = 全局耦合：正确的 climate 噪声 × 仍误判的 biome（nether_wastes）→ 涂布离 vanilla 更远（此前错误 climate 碰巧压制部分 biome 条件反而多对）。假设标注：biome 修复前置与 legacy climate 存在顺序依赖 | **逐语义正确 ≠ 整体正确**——visitor 全局替换类改动必须消融验证子项（分带/分阶段），不能只看总分；「修对反而降分」可能是负相关抵消，处置 = 分带证据 + env 门控回退（保留工作、维持最佳默认态）+ 前置依赖课题，而非回滚正确修复；附推论：legacy 下界 climate/地形主干全固定种子 → worldSeed 无关性（未验证） |


| legacy climate 启用后 t 符号对但 humidity ≈0 vs Java -0.16，总分 82.72→77.01（M10） | 四层对拍（S1 LCG 逐位一致 / S2 Octave 构造一致 / S3 DoublePerlin 采样 ~6e-6 一致 / S4 router 消融 humidity 缺口）隔离出缺口 = `OctavePerlinNoiseSampler::new_legacy`/`DoublePerlin::new_legacy`(-7,[1,1]) 的**采样语义**与 Java createLegacy 未对齐（构造与 LCG 均无错）；未修，yarn 源码已入档待下轮 | **对拍校准分段设计**：裸随机源→单 Octave 构造→复合采样→router 集成，逐层排除，最后不一致层即缺口；**一致性判据先定义精度口径**（f32 路径 ~6e-6 算对齐）；消融开关+分带混淆对+6 维直采三件套单轮拆解「哪一维/哪一段/偏差多少」；符号随开关翻转 = 机制归属证据；Java 参照探针（BIOME6）是校准权威侧 |

| Java t 符号相反 / h≈0，推导出「Octave createLegacy 缺口」结论链，后被同 seed 重比推翻（M11【复发·第3次】） | 跨工具对比用了不同 worldSeed（Java 探针跑 server world -2032795982907864146，Rust 探针用参照 seed -8248318472910187742）；派生噪声与 worldSeed 相关 → seed 不同则派生值全不同，seed 错位假象长得像实现 bug；固定种子特例不受影响 → 混合正确与错误证据带偏定位多个循环 | **seed/坐标错位类错误第 3 次复发**——铁律（seed 三查）存在但没套到「Java↔Rust 跨工具数值对比」场景。修正：**任何跨工具数值对比的第一动作 = 核对两侧 worldSeed 一致**（对比前，非得出怪结论后）；判据：出现**符号翻转/量级级差异**先怀疑 seed/坐标错位再怀疑实现；对比工具链强制 seed 自检（探针输出自带 seed + 脚本核对，人眼核对已三犯不可靠） |
| M11 修正后 t 残差 0.094（shift/派生/参数表全对齐仍不收敛）（M12） | legacy temperature 特例种子源按 yarn sources 字面 0L 实现，而运行时实测为 **worldSeed**（S8 try-seed sweep：seed=worldSeed origins 与 Java router 逐位一致；seed=0/×2 不符）；修后 t 残差 0.094→0.0005（f32 级），nether 82.72% 中性、overworld 零回归 | **种子源必须逐维 try-seed 实测定案，不能同模式类推**（vegetation 套用 worldSeed 立即翻车：h=+0.078 与 t 全同而 Java t≠h，已回滚）；**record 字段 ≠ sampler 实际来源**（noiseData 显示注册表参数、firstSampler 已是 visitor 特例——字段形状+行为对照才是定案手段）；**sources 字面 ≠ 运行时行为**（实测优先于源码字面） |
| 同 seed 竖切 density 残差：cell 角点全 0、中间点 ±0.01~0.08、y≥128 恒值不饱和（M12 补遗二） | **工具语义陷阱**：Java DensityProbe 纯 `df.sample` 直采 vs Rust 竖切走生产插值路径——两侧采样语义不同，中间点/y≥128 的「残差」是语义差非实现错；旧「残差随 y 增长」结论 = seed 错位（M11）+ 语义差复合假象，已推翻 | **跨工具对比先声明两侧采样语义（纯函数/插值/网格）**——「⚠️ 探针采集核对铁律」的采样语义扩展：seed 一致 → 采样语义一致 → 逐节点公式，三层递进，前两层收敛就不进第三层；**cell 角点（插值端点）对齐 = 形状主体对齐的充分判据**，中间点差优先怀疑语义 |
| nether 卡 82%、多带同时残差（y32..95 低至 55~66%）、bedrock roof/floor 整层缺失（混淆对 10288@y96..）（M13） | surface_rule JSON 数据驱动解析器不支持 `vertical_gradient` 条件类型 → nether.json 的 bedrock_floor/bedrock_roof 分支解析返回 None 被**整条静默跳过**（仅一行 WARN 被海量警告淹没）→ 顶部/底部 bedrock 层及规则链涂布全缺；附带：`below_top(N)` 的 height 误传 world_height(256) 致 roof 锚越界（须用 noise_height 128） | **「多带同时残差 + 特定块类缺失」优先查规则解析覆盖率，不逐带调参**——数据驱动解析器静默跳过分支是高危反模式，新数据文件接入 MUST 断言 [PARSE-WARN] 计数为 0，不能只看总分（总分被其它层掩盖，+13.5pp 一步修复）；**锚换算的 height 用逻辑生成高度（nether 128），不混用 world_height 256**（M3 同族） |
