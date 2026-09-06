# review-001 — end 接管摸底 verdict SHOULD 级审查（core.judge）

> 审查对象：00-recon-verdict.md（主）+ 01-rust-status.md + 02-end-biome-rules.md + 03-resource-closure.md
> 级别：SHOULD（摸底结论 / 轻量架构建议 candidate 前置）。只出意见，不改任何 status。
> 审查方式：四份产物通读 + 关键断言廉价独立验证（grep/read 一手源码与数据文件）。

## 一、独立抽查结果（judge 亲手核对）

| # | 断言 | 抽查方法 | 结果 |
|---|---|---|---|
| ① | density_builder.rs 无 `minecraft:end_islands` 分支 | grep 全 worldgen-core/src `end_islands` → 0 命中；density_builder.rs L312-356 通读确认 match 分支集 | ✅ 属实 |
| ② | noise_params.json 无 minecraft:end_islands 条目 | grep `versions/1.20.1/data/noise_params.json` → 0 命中（注：实际路径是 `data/noise_params.json`，01/03 未写全路径但结论成立） | ✅ 属实 |
| ③ | create_for_dim 参数化（worldgen_handle.rs） | 通读 L140-354：min_y/height @L164-166、aquifers @L168、legacy @L170-171/:211、df_ns @L151-154/:217、sea_level 兜底 @L325、非 overworld 走 settings.surface_rule JSON + fail-fast @L329-345、base_3d overworld 硬编码 @L192 —— 行号与 01 表逐条吻合 | ✅ 属实 |
| ④ | 02 篇 Java 行号 | TheEndBiomeSource.java（L15 类声明/L16-25 CODEC/L32-40 createVanilla/L66-87 getBiome/阈值 L73/79/81/84/采样点 L76-77）逐行核对一致；DensityFunctionTypes.java L626-682（-0.9F@L628、CheckedRandom+skip(17292)@L631-635、sample@L637-661、/128 归一@L665、min/max@L670/675）逐行核对一致；NoiseConfig.java L105/L115 一致；ChunkSectionPos.getSectionCoord@L98-100 一致 | ✅ 属实（内部一致性 PASS） |

## 二、审查意见（分条）

### PASS-1 证据链主体验证充分
00 verdict 的三条「已验证事实」均可回溯到一手源码/数据证据，01/02 的行号引用经独立抽查全部命中、无虚引。02 篇边界归属（d==0.25→midlands 等）与源码三元逻辑逐字一致，且已显式标注——这类边界细节常被漏，质量高。

### PASS-2 01/02 的「节点数」表面矛盾已在 00 中正确合并
01 §摸底结论说「管线侧缺口 = 1 个新 density function 节点」，02/01 §2 说需「新写 SimplexNoiseSampler + EndIslands」——不矛盾：1 个 JSON 节点类型（end_islands）的实现 = 节点分支 + SimplexNoiseSampler + CheckedRandom skip 三件套，00 已合并为 G1 并写明组成，自洽。

### PASS-3 置信度合法性
四份产物全部 draft + Partial（纯静态）声明，无 confirmed 越权；02 §4 idk 清单（提取链版本/end.json 逐字节/运行时行为/SimplexNoiseSampler 全文）具体、非敷衍，符合 @anchor.idk 特异性要求。降级声明恰当。

### PASS-4 审查遗漏项覆盖
seed 三查（00 架构要点 3 + 02 §3.4）、§9.7 口径声明、#55 生产命名环境验收均已在计划中体现；02 §3.3 明确 End 分类器与 MultiNoise BiomeClassifier 并列不复用，模块边界清晰；WG_TRANSPILER/DFC/GPU env 互斥与「end 对拍禁开」已在 01 §2.3 non-blocker 备注——00 风险节虽未重述，G2/要点 1 隐含覆盖，可接受。

### CONCERN-1 01/03 对 end.json 的两处事实性错误（不改方向，但须修正后再升 candidate）
1. **sea_level**：01 表写「sea_level 无字段（end）⚠️ 兜底 63.0」，00/G2 沿用「sea_level=63 对 end 无害性确认」——**实测 end.json L134 明确有 `"sea_level": 0`**，Rust L325 会读到 0 而非兜底 63。实际风险比产物描述的更小（0 + default_fluid=air 更无害），但证据链上的断言本身错误，属于「读数据文件不仔细」，必须更正表述。
2. **闭包**：03 说「无 minecraft:reference 外部 df 文件；sloped_cheese.json 在盘但未被 end.json 引用（冗余文件）」——**实测 end.json L66/L66-67 final_density 内有字符串引用 `"minecraft:end/sloped_cheese"`**（经 df_ns 解析的引用，非 `minecraft:reference` 显式类型节点）。sloped_cheese.json **在闭包内**，03 的「引用闭包自包含」结论碰巧仍成立（文件在盘），但「未被引用/冗余」是错误判定；若未来有人按 03 清理「冗余文件」会直接打断 end final_density。01 §3 沿用了同一错误表述。**sloped_cheese.json 的引用形态（namespace 字符串引用 vs minecraft:reference 节点）在 Rust density_builder resolve 路径下是否正确解析，摸底未验证，应补一条 idk/待验项。**

### CONCERN-2 架构建议遗漏点：非 overworld 的 biome 加载与 carvers/features 路径
G2 写了「biome 加载 5 key」，但 create_for_dim L309-316 是无条件 `BiomeClassifier::load(biome_params_file)`——end 无 biome_params 文件（03 实测），end 接入时该路径如何处理（空 classifier？按 settingsName 分流到 EndBiomeSource？）在 00 三要点中未落到具体改动点。nether 跑通不能外推 end（nether 有 biome_params_nether.json，end 没有）。属计划细化项，不阻塞轻量架构判定，但正式开发计划必须显式覆盖。

### CONCERN-3 02 篇引用路径小瑕疵
ChunkSectionPos.java 实际在 `net/minecraft/util/math/`（02 证据索引只写类名+行号，行号正确）；noise_params.json 实际在 `versions/1.20.1/data/`（01/03 未给全路径）。行号/结论均正确，仅路径检索性略降。不影响通过。

### 备注-1 index.yaml
四份产物均在 .investigations/（中间排查记录），按 core.artifact 契约不强制 index.yaml 登记；若后续 verdict 升 candidate 并作为开发计划输入，建议在 .artifacts/ 侧补正式登记。

## 三、推荐

- 四份产物**维持 draft**（与现状态一致）；00 verdict 的缺口定性与轻量 3 要点架构**方向成立**，建议主会话先落实 CONCERN-1 的两处更正（sea_level=0 实测值、sloped_cheese 在闭包内）再交用户批准架构。
- CONCERN-2 作为正式开发计划的必列项；CONCERN-3 顺手修正。
- 本意见不改变任何产物置信度；confirmed 留待人类拍板。
