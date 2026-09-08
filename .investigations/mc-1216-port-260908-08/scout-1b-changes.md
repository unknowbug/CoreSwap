# scout-1b: MC 1.20.1 → 1.21.6 worldgen 机制变更底册（web 勘探轮）

- 角色: recode.scout（只读勘探，产物仅 .investigations/）
- 方法: web_search（官方 changelog / wiki / modding primer）。本沙箱无直接网页抓取能力（Invoke-WebRequest SSL 被拒），来源均为搜索结果 URL，**未能逐页核对原文**——置信度按此降级。
- 反幻觉纪律: 每条给来源 URL；查不到的标「未查到，需一手源码核对」。置信度: 中=官方/社区多源一致但未逐页核对；低=单源或推断；未查到=留待 Java 源码 diff（1.20.1 vs 1.21.6 yarn/sources）核对。
- 状态: **draft**

## 1. noise / density function（multi_noise、spline、插值）

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| multi_noise / spline / 插值核心语义在 1.21.x 系列未发现公开的语义变更记录（官方 changelog 与 modding 资料均未提及 density function 数学子系统重写） | 未知（倾向「无算法级变更」） | [1.21 官方公告](https://www.minecraft.net/en-us/article/minecraft-java-edition-1-21#1) | 低（changelog 证据缺省，非等价证明；需源码核对） |
| density function / noise router 的 JSON schema 字段名与结构 1.20.1→1.21.x 未见破坏性变更报告（社区 datapack 工具 [syldium/worldgen](https://github.com/syldium/worldgen#1) 声明面向 1.21.4 生成 worldgen JSON，字段沿用） | 数据级（schema 兼容倾向） | 同上 | 低 |
| 待核对: `final_density` / surface 之上是否有 per-biome 分支注入（如 pale garden 的路径改写）——**未查到，需一手源码核对** | 未知 | — | 未查到 |

## 2. biome 参数与 MultiNoise 分类器

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| 1.21.4 新增 `pale_garden` 生物群系（overworld multi_noise 参数空间新增一个参数点 + 配套 surface/feature 数据），SearchTree/分类算法本身未见变更报告 | 数据级 | [The Garden Awakens 内容汇总](https://craftdex.net/updates/the-garden-awakens) / [minecraft-france 1.21.4](https://www.minecraft-france.fr/minecraft-1-21-4-the-garden-awakens/#comments#1) | 中 |
| temperature/humidity/continentalness/erosion/depth/weirdness 六参数分类器语义 1.20.1→1.21.6 未查到变更记录 | 未知（倾向不变） | [planetminecraft worldgen 文档（参数语义说明，1.21.x 仍适用）](https://www.planetminecraft.com/blog/custom-world-generation-documentation/) | 低 |
| 待核对: multi_noise biome source JSON 是否新增可选字段（如 spawn cost / 参数点格式）——未查到 | 未知 | — | 未查到 |

## 3. surface rule / carver / aquifer / feature 放置 / structure

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| 1.21 新增 Trial Chambers 结构 + 试炼刷怪笼等 feature（纯数据新增） | 数据级 | [1.21 官方公告](https://www.minecraft.net/en-us/article/minecraft-java-edition-1-21#1) | 中 |
| 1.21.5 新增 leaf litter / bush / firefly bush / wildflowers 等植被 feature（纯数据新增） | 数据级 | [1.21.5 官方公告](https://www.minecraft.net/fr-ca/article/minecraft-java-edition-1-21-5#1) | 中 |
| 1.21.6 新增 dry grass（badlands 表面放置，纯数据新增） | 数据级 | [1.21.6 pre-release 公告](https://www.minecraft.net/sv-se/article/minecraft-1-21-6-pre-release-1#1) | 中 |
| surface rule / carver / aquifer 的放置算法（条件树、carving、aquifer 邻域随机）1.20.1→1.21.6 未查到语义变更记录 | 未知（倾向不变） | — | 未查到，需一手源码核对 |
| structure placement（random_spread / concentric_rings 算法）变更未查到 | 未知 | — | 未查到，需一手源码核对 |

## 4. chunk 生成阶段与状态机

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| 1.21.2 存在 chunk 生成管线内部重构（NeoForge 官方发布 1.21.2 primer 专文覆盖，modding 层面确认内部有实质变动）；具体是状态枚举重排还是依赖图重构**未查到细节** | 算法级（倾向） | [NeoForge 1.21.2 primer](https://github.com/neoforged/.github/blob/main/primers/1.21.2/index.md#10) | 低（primer 存在性=中；内容细节未核对） |
| 1.21.1 之后随机刻逻辑出现不一致报告（说明 1.21.2+ chunk/level 内部行为确有变动），间接佐证上条 | 算法级 | [NeoForge issue #2679](https://github.com/neoforged/NeoForge/issues/2679#issue-3475154123#1#1) | 中（现象存在；与 worldgen 阶段关系未知） |
| ChunkStatus 序列 / NOISE→CARVERS→SURFACE→FEATURE 阶段顺序语义变更未查到 | 未知 | — | 未查到，需一手源码核对 |

## 5. 数据包 JSON schema（worldgen/*.json）

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| 每个 1.21.x 小版本均抬升 datapack pack_format（worldgen 注册表随版本重序列化/严格校验趋严是惯例），具体各版本号未逐条查到 | 数据级 | [1.21.4 官方公告（含 format 说明）](https://www.minecraft.net/en-us/article/minecraft-java-edition-1-21-4#2) | 中（趋势）；低（逐版本号） |
| 1.21.4 公告含 worldgen 相邻 schema 字段说明（如 placed feature offset 语义），提示存在局部字段级调整 | 数据级 | 同上 | 低 |
| worldgen/*.json 顶层结构破坏性重构未查到 | 未知（倾向无） | — | 未查到，需一手源码核对 |

## 6. dimension / noise_settings（min_y/height/shape）

| 变更条目 | 影响级别 | 来源 | 置信度 |
|---|---|---|---|
| min_y=-64 / height=384 / overworld 形状参数 1.20.1→1.21.6 未查到变更记录 | 未知（倾向不变） | [Lithosphere datapack 兼容 1.20–1.21.x 同一份 worldgen](https://modrinth.com:8443/datapack/lithosphere#1) | 低 |
| 待核对: overworld noise_settings 的 spline 数据点是否有数值微调（非结构变更）——未查到 | 未知 | — | 未查到，需一手源码核对 |

## 对 Rust 引擎的预估冲击面

**只换数据（versions/1.20.1 → versions/1.21.6 新薄壳 + 新 JSON）**：
- 新生物群系/feature/structure 数据（pale_garden、trial chambers、1.21.5/6 植被）——现有数据驱动管线直接吃新 JSON。
- datapack pack_format / worldgen 注册表序列化版本号。
- 以上为主。当前证据下 **90%+ 差异面落在数据层**。

**可能动 worldgen-core 算法（需 worker 核对后才能定）**：
1. **chunk 生成阶段/状态机（1.21.2 重构，最大嫌疑）**——若 ChunkStatus 管线语义/顺序变化，影响 `worldgen-core` 的阶段调度骨架。优先核对项。
2. **final_density / surface 的 per-biome 分支**（若 pale garden 走了非纯数据的表面规则注入）。
3. multi_noise 分类器/spline 插值：**大概率无算法变更**，一次源码 diff 核对即可关闭。

**核对方式建议**（交主会话排程）：对 1.20.1 与 1.21.6 的 yarn/sources（或 mojang mappings 源 jar）做 worldgen 包级 diff（`world/gen/density`、`world/gen/surface`、`world/level/chunk/status`、`world/level/biome`），比 web changelog 更权威；本角色无网络抓取能力，需主会话或下一 worker 承担。

## 待深入点清单（→ worker / 后续 scout）
1. NeoForge 1.21.2 primer 正文（chunk 管线细节）——最高优先。
2. 1.21.4 官方公告 worldgen 字段段（offset 语义）。
3. 1.20.1 vs 1.21.6 源码 diff（density/surface/chunk-status/biome 四包）。
