# tag 数据驱动化（260907-04 · swe · status: confirmed）

> 任务：carver replaceable + feature RuleTest tag 从硬编码转 JSON 数据驱动（架构计划 000 轻量，用户批准，缺失策略 = fallback + 一次性日志）。
> 时间锚：2026-09-07 16:56 开工（Get-Date）。
> 状态链：draft → judge PASS-with-conditions（条件 2/3/4 已应用 + compileJava 绿；条件 1 余项 runServer 冒烟留发布前）→ 用户「授权确认」confirmed（2026-09-07 17:24）。

## 改动清单

| # | 文件 | 内容 |
|---|---|---|
| 1 | `versions/1.20.1/data/worldgen/data/minecraft/tags/blocks/*.json`（170 文件） | server jar 提取（loom minecraft-merged.jar，zip 提取脚本 `.tmp/extract_tags_260907-04.py`） |
| 2 | `runtime/1.20.1/java/src/main/resources/worldgen-data/data/minecraft/tags/blocks/`（同 170 文件） | mod 资源侧副本（processResources 自动打包，build.gradle 无需改） |
| 3 | `worldgen-core/src/block_tags.rs`（新） | BlockTagRegistry：全局 RwLock 注册表（create_for_dim init）、懒加载 + 缓存、`#tag` 递归展开（环检测 depth>16 + visited）、required/optional 条目语义、未知名跳过 + 一次性日志；`expand_tag(tag, blocks, out) -> bool`（false = 数据缺失 → 调用方 fallback） |
| 4 | `worldgen-core/src/feature.rs` | `expand_tag` = JSON 优先 + `expand_tag_fallback`（原硬编码 8 tag 保留） |
| 5 | `worldgen-core/src/carver.rs` | `build_overworld_replaceable` = JSON 优先 + `fallback_overworld_replaceable`（原 47 名单保留） |
| 6 | `worldgen-core/src/worldgen_handle.rs` | create_for_dim 在 blocks 加载后 `block_tags::init(&wg_dir)`（load_carvers/features 解析前） |
| 7 | `runtime/.../CoreSwapFixHelper.java` | 缓存新鲜度单 marker → 双 marker（+tags/blocks/overworld_carver_replaceables.json）——旧 tmp 缓存无 tags 会被静默跳过重解压，数据驱动失效（走 fallback 不炸但退化为硬编码） |
| 8 | `worldgen-core/data-driven-boundary.md` | 边界文档更新（tag 移入数据驱动层 + 跨版本清单修订） |
| 9 | 6 个预存破损 bin 修复（EndIslands 非穷尽 match，260906-04 end 接管遗留）：density_tree_profile / transpiler_ch0_census / channel_probe(×2 处) / ch0_tree_analysis / macrolize_probe / tree_vs_noise_breakdown |

## 语义要点

- **缺失策略**（用户拍板）：tag 文件缺失/嵌套 tag 缺失/环 → `expand_tag` 返回 false → 调用方 fallback 硬编码集 + 一次性 stderr 日志；不 fail（跨版本数据未跟上不炸生成）。
- **`minecraft:netherrack` 在 1.20.1 无 tag 文件**（实测 jar 内 MISSING）——feature 消费点走 fallback 硬编码，属正常（非数据缺失）。
- 未知名条目：JSON 路径跳过（不 push AIR，防「tag 含 air」假语义）；fallback 路径保留旧 `blocks.id()` 行为（AIR 兼容，与改动前一致）。
- "replace" 字段忽略（datapack 合并语义，vanilla 单源数据无意义）。

## 验证（§9.7 口径声明）

- **golden 等值（Full，确定性）**：`cargo test -p WorldgenRust --lib` 5/5 PASS——① carver replaceables JSON 展开 ≡ fallback 集（排序去重逐位）；② 6 个 feature tag 同；③ 负向：不存在目录 → None（fallback 协议生效）。载体 = 仓库数据文件 + cargo test；覆盖面 = 本块全部消费 tag；与既有口径可比（同 blocks.json）。
- **构建绿（全量）**：`cargo build --offline --release` Finished（含 worldgen.dll 2,192,896 B @17:07，晚于全部源码改动）；dll 内容哨兵 `block_tags`/`hardcoded fallback` 命中（#23 家族：mtime+内容双验）。
- **行为恒等论证**：tag 内容 JSON ≡ fallback（golden 直证）→ 生成行为恒等 → 无需 Chunky/存档回归（架构计划批准的验证方式）。
- **降级声明**：Java 侧改动（双 marker）仅静态审查，未跑 gradle 编译/生产 runServer 复验（改动 3 行、条件合取，风险低）；mod 维度自定义 namespace tag 未实测（结构已支持 data/<ns>/tags/blocks 路径）。

## 顺手修复（非本块目标）

- 6 个 src/bin EndIslands 非穷尽 match（纪律 13a：全量绿）——end 接管（260906-04）新增 DensityFunction::EndIslands variant 后这些诊断 bin 一直编不过，属预存破损，本块修复后 workspace 全量恢复绿。

## 关联

- 数据驱动边界：`worldgen-core/data-driven-boundary.md`（本块修订）
- AGENTS.md 数据驱动铁律（2026-08-29）；carver 接管 perf 课题前置条件之一（260906-08 confirmed 触发条件）现已解锁
