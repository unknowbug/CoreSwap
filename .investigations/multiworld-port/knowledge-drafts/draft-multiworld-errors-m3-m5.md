# 草稿：multiworld-errors.md 追加 M3-M5（subagent 产出，主会话应用）

> **应用位置**：`.investigations/multiworld-port/multiworld-errors.md`——以下 M3/M4/M5 三节插入到「## 附：错误 → 根因 速查表」**之前**；速查表 3 行追加到现有表格末尾。追加不覆盖。
> 来源：多世界 Phase A/B/C（2026-08-30，commit 1102f58 + 9a3f7fa）。

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
2. 读 `worldgen_handle.rs` L144 附近 macro_sampler 构建：网格高度用的是 `settings height`（world height）而非 `noise.height` → 确认执行层未消费 noise_height。
3. 对照 C++ 时代同坑修复记录（docs/09 修复链表：「y 循环上限 noiseHeight——y 128-255 留 air——22% → 72%」）——**C++ 踩过一模一样的坑**，症状数字都对得上（C++ 22% ↔ Rust 23.77%）。

### 修复
- `worldgen_handle.rs`：handle 存 `noise_height`（= settings noise.height，nether 128），与 world height（256）分开。
- `terrain.rs fill_chunk`：加 `noise_height` 参数，`y ≥ noise_top` 直接留 Air，不进采样。
- 宏观采样器网格：只铺噪声高度（interp cell 按 128 高切，对齐 vanilla nether 网格）；est 循环同步收窄。
- 连带修 13 个探针 bin 的 `fill_chunk` 调用点补 `noise_height` 参数（perf_quant/badlands_probe/biome_fill/blocks_cmp/carver_probe/fillbench/fillmap/fillprofile/grass_probe/mt_fill/surface_probe/terracotta_probe/beard_cmp）。
- 修后：nether match 23.77% → **73.77% → 74.04%**（y≥128 四带 0% → **100%**），**超过 C++ 时代 71.97%**；overworld 基线 95.40% 零回归。证据 `.investigations/multiworld-port/cmd-output/nether_blocks_match_v{1,2_noiseheight}.txt`。

### 教训（可复用判错经验）
- **「分带签名」是垂直结构错的指纹**：match 按 y 分带出现「上半全 0% / 下界部分匹配」= 高度语义错（应 Air 的被填了 / 网格错位），不是噪声值差（值差是全高度均匀低分）。看到分带先查高度参数流，别先怀疑噪声公式。
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
3. 反向验证：手动排序 features 列表后两次运行逐位一致 → 根因坐实。

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
- MOD 游戏内调 `CppBridge.initDim` 报 **`UnsatisfiedLinkError`**；但检查 build 产物 target 目录下的 worldgen.dll **确实是含 initDim 导出的新版**——「dll 是新的但加载行为像旧的」。
- 相同 dll 用探针 bin 直接加载一切正常，只有 gradle 跑的 MOD 侧报错。

### 根因（机制）
- build.gradle 里 dll 同步写在 **`processResources` 的 `doFirst`** 里（拷 build-msvc 产物 → resources）。该任务**输入未变时 Gradle 判 UP-TO-DATE 直接跳过——`doFirst` 的副作用拷贝不在任务输入集，跳过时同步不执行** → resources 里躺着**旧 dll**，运行时 `System.load` 加载的是旧导出表 → 无 initDim 符号 → UnsatisfiedLinkError。
- 本质：**「构建脚本里的副作用拷贝」没有声明为任务输入依赖**，被增量构建静默跳过——这是 Gradle 增量模型与命令式 doFirst 的经典冲突。同批还修了一个 Mixin 侧隐患：`@Shadow` **够不到父类字段**（biomeSource 在父类），末地保护分支改用**缓存反射**读取（附记：@Shadow 只作用于当前 mixin 目标类声明的字段/方法，父类成员须反射）。

### 定位（诊断方法）
1. **三层 dll 导出表对照**：target/（构建产物）✅ 有 initDim → resources/（运行时实际加载）❌ 旧导出表 → gradle tmp 解包目录 ❌ 同旧——「目标层新、资源层旧」锁死同步链断点在 processResources。
2. 对照任务执行日志：processResources 标 `UP-TO-DATE`，doFirst 未执行 → 同步被跳过的机制确认。
3. 运行日志 `UnsatisfiedLinkError: initDim`（而非 "no worldgen.dll"/路径错误）——符号级缺失，指向「加载了存在但版本旧的 dll」。

### 修复
- build.gradle：`processResources` 加 **`inputs.file(<dll 路径>)`**——dll 变化即任务失效重跑，doFirst 同步随之执行。
- 清掉 resources/ 与 tmp 里的陈旧 dll，全量重跑。
- 修后游戏内实证：`initNether enabled=true` + `[Mixin] populateNoise(nether) intercepted chunk(-1,-1)`（rust_nether_test4.log，摘录 `.investigations/multiworld-port/cmd-output/nether_ingame_intercept_20260830.txt`）。

### 教训（可复用判错经验）
- **「构建脚本里的副作用拷贝」必须声明为任务输入依赖**（Gradle `inputs.file` / `inputs.dir`），否则增量构建静默跳过、产物层吃旧文件——任何写在 doFirst/doLast 里的拷贝/生成动作都要问一句「它的源文件在任务输入集里吗」。
- **「dll 是新的但行为像旧的」→ 立即做多层产物导出表对照**（构建产物 → 资源目录 → 运行时实际加载路径逐层 dumpbin /exports）——断点在哪一层一眼定位，不要在代码侧猜符号注册。
- **符号级 UnsatisfiedLinkError 排查序**：① 加载的到底是不是目标文件（路径+版本）② 该文件导出表有没有符号 ③ JNI 签名是否匹配——第 ② 步优先于第 ③（本例是 ②）。
- **@Shadow 父类字段不可达（附记）**：mixin @Shadow 只覆盖目标类自身声明的成员，父类字段须反射且**缓存 Method/Field**（不要每 chunk 反射查找）。

---

## 速查表追加 3 行（追加到 `.investigations/multiworld-port/multiworld-errors.md` 现有速查表末尾）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| nether match 23.77% + y≥128 全 0%（M3） | noise_height 参数化停留在加载层；fill_chunk y 循环与宏观采样器网格仍按 world height 256 铺（噪声只有 128 高语义）→ 上半被错填 + 下半网格错位 | **「上半全 0%」分带签名 = 垂直结构错**，先查高度参数流不查噪声公式；参数化须核对「JSON→handle→fill→网格→est」每一跳；**C++ 已知坑清单（docs/09 修复链表）是移植 checklist**（C++ 22%↔Rust 23.77% 同坑） |
| 两次运行差 2796 块（M4） | `BiomeClassifier` features/carvers 用 HashMap，`all_features_lists()` 每进程随机迭代序 → PlacedFeatureIndexer 编号随机 → 放置序 + decorator seed 变 | **跨进程确定性要求所有影响输出的容器迭代序确定——Rust HashMap 默认不满足，Registry 类容器一律 BTreeMap/Vec 排序**；「同输入两次运行不同」= 确定性 bug，与精度 bug 不同族，二分管线段找「每进程会变的量」 |
| initDim UnsatisfiedLinkError 但 target dll 是新的（M5） | processResources 的 doFirst 同步不在任务输入集 → UP-TO-DATE 跳过时同步不执行 → resources 留旧 dll。附记：@Shadow 够不到父类字段 → 缓存反射 | **构建脚本里的副作用拷贝必须声明为任务输入依赖（inputs.file）**，否则增量构建静默跳过；「产物新行为旧」→ 三层导出表对照（target/resources/tmp）定位断点；@Shadow 只覆盖目标类自身成员 |
