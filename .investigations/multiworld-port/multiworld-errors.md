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

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| nether 加载 panic `unresolved density function ref: minecraft:nether/base_3d_noise`（M1） | `DensityBuilder::resolve_ref` 惰性按需加载**硬编码** `key.starts_with("minecraft:overworld/")` + `key["minecraft:overworld/".len()..]`；nether 引用 `minecraft:nether/` 前缀不匹配 → 落 panic。命名空间写死，未参数化维度 | **多世界参数化 = 命名空间/维度数据不要硬编码**；`"minecraft:overworld/"` 字面前缀/目录都是 single-world 假设。panic「unresolved ref」先判「前缀是否被识别」再查「文件是否缺失」——前者是结构错后者是环境错。lazy-load 前缀 + `density_function/<dfNs>/` 目录是两处同源，须同步参数化 |
| `VanillaAquifer` 加 `enabled` 后约 20 个 probe 报 `error[E0063] missing field 'enabled'`（M2） | Rust struct literal 逐字段构造；给 struct 加**非 Default 的 pub 字段**后所有 `VanillaAquifer { aq }` 构造点全失效（无隐式默认值）——**数据结构字段变更 = 全体构造点结构性连锁破坏** | **加公共 struct 字段是结构性破坏点**：改前 grep 所有 `StructName {` 构造点评估波及面；默认值语义用显式 `::new()` 构造器收敛（一处 `new()` 取代 20 处手动补默认字段）；E0063 编译错误清单 = 受影响点全集，改完 MUST 全量编译（漏改编译期即暴露） |
