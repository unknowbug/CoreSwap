# A 组引擎 4 硬缺口 — Phase 1 勘察（260907-05 开工）

> 交接结论验证：NEXT_SESSION 开工点 1 的 4 缺口声明已逐条一手源码核对，全部实锤。

## 缺口核对结果（一手源码，260907-05）

| # | 缺口 | 位置（实测） | 现状 |
|---|------|-------------|------|
| 1 | 未知 block id 静默回填 AIR | `worldgen-core/src/blocks.rs:41` `id()` = `unwrap_or(AIR)` | mod/新版本方块查不到 → 静默写 AIR，无日志；blocks 表唯一来源 = `{wg_dir}/../blocks.json`（worldgen_handle.rs:335），无运行时注册通道 |
| 2 | default_block 硬编码 stone | `surface_rules.rs:1229,1406` + `worldgen_handle.rs:583,744` 共 4 处 `blocks.id("minecraft:stone")` | noise_settings JSON 有 `default_block` 字段未消费 |
| 3 | `data/minecraft` 命名空间硬编码 | `worldgen_handle.rs`（settings_path:184 / df_dir:244 / biome_dir:348 / carver:882 / base_3d_noise:219）+ `feature_loader.rs`（placed/configured feature 路径 6 处） | mod 维度数据放 `data/<modns>/worldgen/` 无法加载 |
| 4 | biome 表仅原版 | `worldgen_handle.rs:346-352`：biome_params_file 已参数化（对齐 C++），但 biome_dir 硬编码 `data/minecraft/worldgen/biome`；BiomeClassifier 参数集仅 vanilla biome_params.json | 与 #3 同根（命名空间），mod biome JSON 无法入表 |

## 与 JNI mod 方块注册的关系

- #1 的修复（运行时方块注册 API）是 mod 方块注册的 Rust 侧前置；mod 侧接线（JNI 回调/资源部署）仍为挂起项，本块只做 Rust 侧 API + 数据面，不动 mod Java 代码。

## 影响面

- 改 `blocks.rs`（注册 API）、`worldgen_handle.rs`（命名空间参数 + default_block 消费）、`surface_rules.rs`（default_block 传参）、`feature_loader.rs`（路径参数化）。
- API 面：`wg_create` 已有 settings_name/biome_params_file 参数；命名空间可由 settings_name 派生或新增参数（设计点 D1）。
- 判据（承 260907-04 先例）：fallback 必配一次性日志；改造后 golden 逐位等价或显式声明行为变化；workspace 全量 build（#23 机制面二）。

## 设计决策点（Phase 2 前定）

- **D1 命名空间来源**：(a) 从 settings_name 解析（`minecraft:nether.json` / `modid:dim.json` 形式）(b) 新增 wg_create 参数。倾向 (a)——不破 C ABI，向后兼容（无 `:` 前缀 → 默认 minecraft）。
- **D2 未知方块策略**：(a) 注册 API `wg_register_block`（分配 id ≥ 现有 max+1）(b) 保持 AIR + 一次性日志。倾向 (a)+(b)：注册前查到未知名 = 日志告警（不再静默）；注册后正常解析。
- **D3 default_block**：从 settings JSON `default_block` 读，缺失 fallback `minecraft:stone` + 一次性日志（对齐 tag 先例）。
