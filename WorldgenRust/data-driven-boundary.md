# Rust worldgen 数据驱动边界（跨版本设计）

> 目的：数据驱动本质是**跨版本方便**——升级 MC 版本时，版本相关数据尽量从 JSON/数据文件加载，减少代码改动。本文件记录数据驱动边界：哪些数据驱动、哪些代码硬编码、跨版本时怎么改。

## 数据驱动层（从 JSON/数据文件加载，跨版本换数据即可，不碰代码）

| 数据 | 文件 | 加载点 |
|---|---|---|
| density 函数树 | `density_function/*.json` | `DensityBuilder::set_external_loader` |
| noise settings / router | `noise_settings/overworld.json` | `WorldgenHandle::create` |
| 噪声参数 | `noise_params.json` | `load_noise_params_file` |
| biome 参数 | `biome_params.json` | `BiomeClassifier::load` |
| biome carvers/features | `biome/*.json` | `load_carvers` / `load_features` |
| block id | `blocks.json` | `BlockRegistry::load_from_json` |
| placed/configured feature | `placed_feature/*.json` + `configured_feature/*.json` | `FeatureCache` |
| configured carver | `configured_carver/*.json` | `carver_cache` |

**跨版本**：换对应 JSON 数据文件即可。block id 统一经 `blocks.json` 解析（`blocks.id("minecraft:stone")`），MC 很少改 block 名 → 代码稳定。

## 代码硬编码层（无数据源，版本相关）

| 项 | 位置 | 说明 | 跨版本处理 |
|---|---|---|---|
| carver replaceable（`#minecraft:overworld_carver_replaceables` tag） | `carver.rs build_overworld_replaceable` | 硬编码 block 名数组（tag 无独立数据源） | 已加注释：核对新版本 tag 展开 |
| feature RuleTest tag 展开（base_stone_overworld 等） | `feature.rs expand_tag` | 硬编码 tag 内容（无数据源） | 已加注释：核对新版本 tag |

**说明**：
- 这些硬编码的是 **block 名称字符串**（经 `blocks.id()` 解析成 id），不是数字 id → id 已数据驱动
- block 名称在 MC 版本间稳定（改名极少）；tag 内容可能变 → 跨版本只需核对 tag 展开
- 因当前数据目录不含 `tags/blocks` 数据（tag 在完整 jar 的 data/），暂无法完全数据驱动
- 若未来补充 tag 数据源，可进一步数据驱动（读 tags/blocks/*.json 展开）

## 跨版本升级检查清单

1. 换 `blocks.json` / `biome_params.json` / `noise_settings/*.json` / `density_function/*.json`
2. 检查 `carver.rs build_overworld_replaceable`（tag 变更）
3. 检查 `feature.rs expand_tag`（tag 变更）
4. 其余自动跟随数据文件

## 边界原则

- **block id 一律数据驱动**（经 `blocks.id` 或从 JSON config 解析），不硬编码数字
- **版本相关无数据源的 tag 展开**：代码硬编码但**集中管理 + 注释标注升级点**
- 算法/流程与版本无关的部分保持代码（不数据驱动）

## 多世界参数化（2026-08-29，对齐 C++ wg_create）

`WorldgenHandle::create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)` 支持任意维度：
- **settings_name**：`noise_settings/<settings_name>.json`（overworld / nether / end / mod 维度）
- **dfNs** = settings_name 去 ".json"，决定 `density_function/<dfNs>/` 目录 + resolve_ref 命名空间前缀（DensityBuilder.set_df_ns）
- **维度参数**从 settings 读：min_y / noise.height / sea_level / aquifers_enabled（非硬编码 overworld -64/384/63/true）
- **biome_params_file**：维度 biome 参数（overworld biome_params.json / nether biome_params_nether.json / mod 自定义）
- **surface_rule**：overworld 用代码规则（已验证）；其他维度用 settings.surface_rule JSON 数据驱动（`SurfaceBuilder::parse_surface_rule`，支持 sequence/condition/block + all conds）
- **aquifers_enabled=false**（下界）→ VanillaAquifer.enabled=false，classify 跳过真实 aquifer（无 water/lava）
- **mod 维度（如暮色森林）**：数据文件放 wgDir 对应路径（noise_settings/<mod_dim>.json + density_function/<mod_dim>/*.json + 对应 biome params），settings_name 指向即可加载
- **便捷入口** `create(seed, wg_dir)` = create_for_dim(seed, wg_dir, "overworld.json", "biome_params.json", 384)（overworld 兼容既有调用）
- 已验证：nether 加载（min_y=0）+ 生成 chunk；overworld 回归 95.40%
