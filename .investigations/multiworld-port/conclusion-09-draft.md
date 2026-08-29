# Rust worldgen 多世界参数化 —— 09 主题篇追加小节草稿

> 载体建议：`versions/1.20.1/docs/09-multi-dimension.md` 追加小节（09 = 多维度通用引擎/数据驱动任意维度）。
> 价值门：**中价值（简记）**——多世界架构 = 跨版本/跨维度复用（多版本升级时换 JSON 数据文件即可复用，通用引擎方向），按「记录价值门」中价值简记为「是什么」，不展开一次性对齐数值快照。
> 本文件是 subagent 产出草稿，主会话验证后应用到 09 篇（追加不覆盖）。
> 本 session（2026-08-29）完成：Rust `WorldgenHandle` 多世界参数化，对齐 C++ `wg_create`。验证：nether 加载 + 生成 chunk；overworld 回归 95.40% 不变。

---

## 七、Rust 世界参数化（2026-08-29，对齐 C++ `wg_create` 多世界方向）

`WorldgenHandle::create(seed, wg_dir)`（overworld 便捷入口）拆出通用入口 `create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)`——支持任意维度加载，对齐 C++ `wg_create` 多世界方向。

**参数化维度（非硬编码 overworld）**：
- `settings_name`：`noise_settings/<settings_name>.json`（overworld / nether / end / mod 维度文件名）
- **dfNs** = settings_name 去 ".json"：决定 `density_function/<dfNs>/` 目录 + `resolve_ref` 命名空间前缀 `minecraft:<df_ns>/`（`DensityBuilder.set_df_ns`；修复 M1——惰性加载前缀原硬编码 `minecraft:overworld/`）
- **维度参数从 settings 读**：`min_y` / `noise.height` / `sea_level` / `aquifers_enabled`（非硬编码 overworld 的 -64/384/63/true）
- `biome_params_file`：维度 biome 参数（overworld `biome_params.json` / nether `biome_params_nether.json` / mod 自定义）
- `world_height`：世界高度（overworld 384 / nether 256 / mod 按定义；0 = 从 noise.height 兜底，对齐 C++ `worldHeight>0?worldHeight:noiseHeight`）

**surface_rule 数据驱动（`SurfaceBuilder::parse_surface_rule`）**：
- overworld：保留已验证的代码规则（`build_overworld_rule`）
- 非 overworld：用 `settings.surface_rule` JSON 数据驱动（支持 sequence / condition / block + 各 cond：not / biome / y_above / stone_depth / noise_threshold / hole / steep / water / temperature / surface）——mod 维度无需改代码
- 对齐 C++ 方向：surface_rule 从 JSON 尾部读（数据驱动），主世界保留代码规则

**aquifers_enabled=false（下界）→ VanillaAquifer.enabled=false**：
- `classify` 跳过真实 aquifer（无 water/lava），返回 Air（修复 M2——加 `enabled` 字段破坏全部 struct-literal 构造点，用 `VanillaAquifer::new(aq)` 收口）

**验证结果**：
- nether：`create_for_dim(seed, wg_dir, "nether.json", "biome_params_nether.json", 256)` 加载成功（min_y=0 / height=256）+ 生成 chunk(0,0) **56307 非空气块**
- overworld 回归：`features_probe` match **95.40%** 不变
- mod 维度（如暮色森林）：数据文件放 wgDir 对应路径（`noise_settings/<mod_dim>.json` + `density_function/<mod_dim>/*.json` + biome params），settings_name 指向即可加载

**数据驱动边界更新**：详见 `WorldgenRust/data-driven-boundary.md`「多世界参数化」章节——block id / density / biome / feature 已数据驱动，跨版本换 JSON 数据文件即可；carver replaceable / feature tag 展开无数据源代码硬编码（已标注升级点）。AGENTS.md「数据驱动架构铁律」含本多世界方向（用户拍板）。
