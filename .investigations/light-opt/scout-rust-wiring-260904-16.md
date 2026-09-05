# scout：Rust 重写光照引擎——当前接线摸底（260904-16）

> 角色：scout（只读勘探）。状态：**全部 draft**（静态审查级，无运行时验证）。日期 260904-16（已按宿主核对流程接受任务方给定值）。
> 范围：光照当前归属 / 可拦截点 / chunk 上下文依赖 / 可复用资产 / 缺口清单。

---

## 1. 模块地图（当前光照在谁手里）

### 1.1 Java 侧（runtime\1.20.1\java\，盘上存在，fabric-loom 1.10.5 + MC 1.20.1 yarn 1.20.1+build.10）

- **全源码树 grep -i "light"：0 命中**（38 个 .java：probe/mixin/桥均无）。也无 `ChunkStatus` / `LIGHT` / `lighting` / `LightingProvider` 命中。
- **结论（draft）**：Java 侧接管代码**从未触碰 LIGHT 阶段**；光照 100% 仍由 vanilla 引擎持有
  （`ServerLightingProvider` / `ChunkLightingView` 一族，LOOM 提供的 MC 源码在本仓只读缓存外，未逐行核对——**降级声明**：vanilla 光照类名/位置为通用 MC 知识，非本仓验证）。
- 现有 mixin 只有两个：`mixin/NoiseChunkGeneratorMixin.java`（拦 `populateNoise` + `buildSurface`）、以及探针用 mixins（EstDump/NoiseDump/SurfaceDump/Blob/Aquifer/ColProf/ConfiguredFeature/DiagFeatureBiome）。
- `BenchMod.java`（fabric mod 入口，onInitialize）：SERVER_STOPPING → CppBridge.destroy；`-Dcpp.replace=1` → `CppBridge.init(seed)` + `initNether(seed)`；`-Dworldgen.bench` → `WorldGenBench.run(server)`；其余为各探针分发。
- mixin json 注册文件未逐条打开核对（draft，按目录内容推断）。

### 1.2 Rust 侧（WorldgenRust\）——**生产主线**

- Cargo.toml：crate-type `["cdylib","rlib"]`，唯一依赖 `jni 0.22.4`。产 `WorldgenRust.dll`。
- src/ 顶层模块（lib.rs L3-35）：noise / density / density_builder / aquifer / surface / surface_rules / ore_vein / biome / blocks / beardifier / spline / terrain / dfc_backend / gpu_ffi / api / json / verif / md5 / xoroshiro / legacy_random / chunkrandom / carver / worldgen_handle / placement / feature / feature_loader / **jni_bridge** / generated_density（build.rs transpiler 产物）。
- **当前接管阶段**（worldgen_handle.rs `fill_chunk_blocks`，L497 起）：①fill_chunk 宏观密度 → ②aquifer + ore_vein → ③build_surface → ④carvers（flag/env 可 skip）→ ⑤features（flag/env 可 skip）。**无 lighting 模块，无任何光照实现**（grep "light" 仅命中 terracotta 色名与 light_gray）。
- **可复用（光照相关辅助输入）**：
  - **heightmap**：`WORLD_SURFACE_WG` [256] int（索引 z*16+x），surface/features 均在用（surface_rules.rs L342 `column_heightmap`；worldgen_handle.rs L563/L890）。注意历史转置坑（surface_rules.rs L119 注释）。
  - **chunk section 数据结构**：`blocks.rs` BlockColumn（16×16×H 的 BlockId 列 + surface_height）；C++ 侧同构（worldgen_api.cpp 注释互引）。
  - C++ 侧 grep -il light（versions\1.20.1\cpp\worldgen\src）：仅 carver.h / surface.h / tbands_dump.cpp 的 terracotta 色名字符串——**C++ 无光照实现**（粗查口径，未深入，符合任务要求）。

### 1.3 集成方式（dll 链路）

- Java `wg.CppWorldgen`（native 方法：init/initDim/destroy/fillDensity/densityParams/fillBlocks/setBeardifier/setFlags/getFlags）→ JNI 符号 `Java_wg_CppWorldgen_*`。
- **权威 dll = Rust**：WorldgenRust\src\jni_bridge.rs 直接导出上述 JNI 符号（jni 0.22，内部调 api.rs 的 `wg_*` C ABI）。`rust_jni_bridge.cpp` 是旧 C++ 桥（对照用，已归档路线）。
- build.gradle L23-47：processResources **doFirst 强制同步** `WorldgenRust/target/release/WorldgenRust.dll` → resources/native/worldgen.dll（dll 为任务输入，防 UP-TO-DATE 旧包）。
- C++ jni_bridge.cpp / versions\1.20.1\cpp：已非主线（2026-08-30 转 Rust）。

## 2. 接线点清单（LIGHT 可拦截点）

现管线拦截模式（mixin HEAD + cancellable，按维度形状 minY/height 区分 overworld/nether，End 用 biomeSource 反射排除）：

| 阶段 | 拦截点 | 现状 |
|---|---|---|
| NOISE | `NoiseChunkGenerator.populateNoise` HEAD，cir.setReturnValue | 已接管（CppBridge.fillChunk → writeChunk 直写 ChunkSection + `Heightmap.populateHeightmaps` 补 6 种高度图，CppBridge.java L285-326） |
| SURFACE | `NoiseChunkGenerator.buildSurface` HEAD，ci.cancel | 已跳过（C++/Rust 内部含 surface） |
| CARVERS/FEATURES | 未 mixin；Java vanilla 照跑，Rust 侧双跑由句柄 flag 关闭（`setFlags` stageMask bit0=SKIP_CARVER bit1=SKIP_FEATURES，默认 0b011；build.gradle `-PrustStages`） | Java 持有 |
| **LIGHT** | **无任何拦截**。候选模式：`ServerLightingProvider` 方法级 mixin，或 `ChunkStatus.LIGHT` 对应的 `light()` 步骤（1.20.1 为 `ChunkStatus.LIGHT` task，经 `LightingProvider`/`ServerLightingProvider.lightChunk`）HEAD cancellable——**具体方法签名未在本仓核实（需 loom 生成源码确认），标注未验证** | vanilla 持有 |

- LIGHT 阶段输入（draft，通用 MC 知识 + 本仓旁证）：已完成 FEATURES 的 chunk（sections 方块）+ 邻居 chunk（光照跨界传播，≥1 ring）；heightmap 非直接输入但已由 writeChunk 补齐。
- LIGHT 下游读者：mob spawning（SpawnHelper）、渲染、方块更新光照回调。**这些均在本仓源码树外，未核实**——接管 LIGHT 后需确认 chunk 完成协议（`ChunkStatus.LIGHT` 的 future 链）不被破坏，风险点：mixin cancel 后必须自行 setReturnValue 已"点亮"的 chunk 并保持 `LightingProvider` 状态一致（`setLightEnabled`/section light storage），否则下游 NPE/全黑。

## 3. chunk 上下文依赖与既有教训（可直接引用的知识条目）

- `knowledge/discovered/workflow-patterns.md`：
  - 发现 #10：cppReplace 存档口径三阶段归因法（接管部分阶段时残差是多阶段混合产物）。
  - 发现 #11：嵌套接管双跑风险（内层全管线 × 外层分步拦截）——LIGHT 接管若 Rust 侧也算光、Java 又跑，即复刻此坑。
  - 「getChunk 阶段语义」「接管单阶段后的后续阶段上下文依赖（2026-08-31）」条目：接管后 feature 装饰拿到的上下文被污染案例——LIGHT 接管同理需审计 chunk 上下文（status 推进、邻居依赖）。
- stageMask 语义（#14 家族）：`-Dcoreswap.rust.stages` 只控 Rust 内部阶段；扩展 LIGHT 需新增 flag 位（如 bit3）+ Java 侧 mixin 双侧协同。

## 4. bench 基础设施（端到端性能对比复用）

- **Java 侧端到端**：`wg/bench/WorldGenBench.java`，入口 `-Dworldgen.bench`（build.gradle `-PbenchProbe`）；参数 `-Dbench.seed/size/originX/originZ/out`（build.gradle L54-63，默认 seed=-8248318472910187742、size=8、out=versions/1.20.1/data）。大样本纪律（≥256 chunks、稳定中位数、排除冷启动）见 AGENTS.md 端到端铁律。
- **Rust 侧**：src/bin/ 有 pipeline_bench.rs、bench_single.rs、bench_threads.rs、perf_probe*.rs、macro_grid_density_bench.rs 等（cargo run 级，不入 Java 链路）。
- 测量纪律：并发下禁用阶段计时探针（WG_PROFILE 等污染），只信无探针整批 wall + 计数（AGENTS.md 测量污染铁律）；诊断代码不得进热路径（每点 env 查询 27% 退化案例）。
- 无现存光照基准——LIGHT bench 需新建（缺）。

## 5. 缺口清单（接管 LIGHT 前必须补）

1. **vanilla 光照引擎结构未勘探**：1.20.1 `ServerLightingProvider.lightChunk` / `LightingProvider` / `ChunkLightStorage` 的方法签名与 chunk 完成协议——需对 loom 生成的 MC 源码做二次勘探（本仓只有 yarn 依赖坐标，无源码树）。
2. **Java mixin json 注册机制**：resources 下 mixin 配置文件未核对（新增 LIGHT mixin 需在此注册）。
3. **Rust 侧光照算法全缺**：sky/block light 传播、§ 有损容差设计、跨 chunk 边界传播的邻域协议（现 wg_* C ABI 全部单 chunk 进出、无邻 chunk 读通道——fillBlocks 只传 chunk 坐标数组）。
4. **接口边界扩展**：api.rs/jni_bridge.rs/CppWorldgen.java 三处需同步新增 light API；output buffer 布局（block light + sky light nibble per section）需定义。
5. **下游兼容验证通道**：存档口径 light 数据（MCA 中的 skylight/blocklight arrays）与 ReadWorldProbe/compare_save_region.py 工具是否读光照段——未核实。
6. **End 维度**：现有拦截全靠形状判断 + biomeSource 排除；LIGHT mixin 若按同法需考虑 End sky light=0 特例（未核实 vanilla 行为）。

## 6. 不确定项汇总（不编造声明）

- vanilla 光照类名/方法签名/ChunkStatus.LIGHT 上下文：通用知识，未在本仓核实（无 MC 源码树）。
- mixin 注册 json 路径/内容未打开。
- WorldGenBench 内部计时口径未逐行读（仅确认入口与参数映射）。
- C++ 侧仅粗查（任务口径）。
- 「建议切 Session」不适用（本产物为一次性勘探交付，无续推假设）。
