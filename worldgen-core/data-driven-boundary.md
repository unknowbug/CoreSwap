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
| 方块 tag（#minecraft:xxx 展开） | `tags/blocks/*.json`（170 文件，server jar 提取） | `BlockTagRegistry`（block_tags.rs，260907-04） |

**跨版本**：换对应 JSON 数据文件即可。block id 统一经 `blocks.json` 解析（`blocks.id("minecraft:stone")`），MC 很少改 block 名 → 代码稳定。

## 代码硬编码层（无数据源，版本相关）

| 项 | 位置 | 说明 | 跨版本处理 |
|---|---|---|---|
| ~~carver replaceable（`#minecraft:overworld_carver_replaceables` tag）~~ | ~~carver.rs~~ | **260907-04 已数据驱动**（block_tags JSON 优先；硬编码表降级为 fallback + golden 基准） | 换 tag JSON 即可 |
| ~~feature RuleTest tag 展开（base_stone_overworld 等）~~ | ~~feature.rs~~ | **260907-04 已数据驱动**（同上，`expand_tag_fallback` 为兜底） | 换 tag JSON 即可 |

**残留说明（260907-04）**：
- fallback 硬编码表**保留不删**（数据缺失不炸生成，用户拍板缺失策略 = fallback + 一次性日志）；golden 测试（block_tags.rs tests）保证 JSON 展开 ≡ fallback 集合。
- `minecraft:netherrack` 在 1.20.1 **无 tag 文件**（feature 消费点走 fallback 硬编码——1.20.1 该 tag 确实不存在，非数据缺失）。
- Java 侧部署：CoreSwapFixHelper 双 marker（noise_settings + tags/blocks）判缓存新鲜度，旧 tmp 缓存自动重解压。

## 跨版本升级检查清单（260907-04 修订）

1. 换 `blocks.json` / `biome_params.json` / `noise_settings/*.json` / `density_function/*.json` / `tags/blocks/*.json`（server jar 重新提取，脚本 `.investigations/tag-datadriven-260907-04/extract_tags.py`）
2. 其余自动跟随数据文件（carver replaceable / feature tag 已 JSON 驱动，fallback 只是兜底基准，不需手改）

## 边界原则

- **block id 一律数据驱动**（经 `blocks.id` 或从 JSON config 解析），不硬编码数字
- **tag 展开已数据驱动**（260907-04）：JSON 优先，硬编码 fallback 表集中管理并作为 golden 等值基准
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
