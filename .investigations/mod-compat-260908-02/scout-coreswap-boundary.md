# scout 盘点：CoreSwap worldgen 接管的边界面（第三方 worldgen mod 撞点）

> 日期标签 260908-02（scout 只读盘点，产物仅本文件；不做兼容性结论——收敛归主会话）。
> 证据类型标注：【实测】= 端到端/运行时实证 artifact；【代码】= 源码/文档静态出处；【纸面】= 文档声明但无本地实体复测。

## 〇、背景快照（谁在拦、怎么拦）

- **Java 侧接管判定**：260906-03 起从「形状指纹」改为「按 noise_settings 注册 id 判定」（资源感知自动放行：settings id ∉ 接管集 → release to vanilla）。**接管集当前硬编码**，无 dimensions.json 类声明入口（NEXT_SESSION.md 开工点 5 原文）。已知边界：overworld 变体 settings（amplified 等）现被放行 vanilla；mod 维度放行与 end 同代码路径，**未本地实体复测**【纸面，10-timewise-archive.md L2997-3004】。
- **Rust 侧维度参数化**：`WorldgenHandle::create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)` 已支持任意维度；`create(seed, wg_dir)` 便捷入口 = overworld 硬编码（"overworld.json"/"biome_params.json"/384）【代码，worldgen_handle.rs L159-161, L176-230】。
- **数据驱动边界权威**：worldgen-core/data-driven-boundary.md（block id / density / noise_settings / biome 参数 / carver+feature JSON / 方块 tag JSON 全数据驱动；fallback 硬编码表为兜底 + golden 基准）。

## 一、对照表：第三方 mod 改动类型 → CoreSwap 现状

| # | mod 改动类型 | CoreSwap 现状 | 撞点 / 边界 | 实测程度 | 证据 |
|---|---|---|---|---|---|
| 1 | **加自定义 biome（含 climate/multi-noise 点位）** | biome 判定 = MultiNoise 近似，参数行从 `biome_params.json` 读（box 范围 + offset），SearchTree 自动建树——**加行即数据驱动**；biome 目录按命名空间合并（`data/<modid>/worldgen/biome`，id 带 modid: 前缀），carvers/features 随之加载 | ① biome_params 行须**手工**追加到 `biome_params.json`（Rust 不读 mod 的 datapack multi_noise JSON 源文件，格式是 CoreSwap 自有 box 格式）；② Java 侧把 mod biome 数据放进 wgDir 是接入方责任；③ **mod 自带 biome_params 完整场景未实测**（NEXT_SESSION 遗留 idk ①，参数化前置） | 机制【代码】；mod 场景【纸面】 | worldgen_handle.rs L365-382；biome.rs L242, L385-399（load boxes→tree）, L424/498（load_carvers/features 按 ns）；NEXT_SESSION.md L36, L40 |
| 2 | **换/改 noise_settings（自定义密度函数 JSON）** | **已数据驱动**：noise_settings/<settings>.json 整个 router（final_density + barrier/floodedness/spread/lava/erosion/depth/initial + 6 climate DF + vein 3 件）从 JSON 建；density_function/<dfNs>/*.json 经 external_loader；cell 尺寸 size_horizontal/vertical 从 settings 读 | ① settings_name 必须在 Java 侧接管集内才会进 Rust（见 §〇），mod 自定义 settings id 不在集内 → 放行 vanilla（行为正确但「未接管」）；② dfNs 派生规则 = settings_name 去 .json（modid:name 语法支持，260907-05）；③ overworld 同 id 覆盖（mod 改 overworld.json 本体）时 df_ns=="overworld" → **仍走代码版 overworld surface rule**，mod 的 surface_rule JSON 被忽略 | 机制【代码】+ nether/end 加载【实测】；mod 自定义 settings【纸面】 | worldgen_handle.rs L181-189, L191-224, L274-327, L341-354；data-driven-boundary.md 表；docs/09-multi-dimension.md |
| 3 | **加自定义维度** | create_for_dim 全参数化：min_y/noise.height/sea_level/aquifers_enabled/legacy_random_source 从 settings 读；surface_rule 非 overworld 走 settings.surface_rule JSON 解析（fail-fast，解析失败 panic 不回退）；biome 目录支持 mod 命名空间 | ① **world_height 是显式参数**（Java 传维度定义；0 兜底 noise.height）——维度 chunk 形状 ≠ noise.height（end 实形 0/256，发现 #58），新维度首查形状碰撞；② **mod 自带 biome_params 未实测**（遗留 idk ①）；③ 数据文件必须按 `data/<ns>/worldgen/...` 布局放进 wgDir（mod jar 内数据不在资源层，Java 侧搬运是前置）；④ 空 air 列 chunk 压力形态须覆盖（发现 #57 判据）；⑤ end 专用判定（end_mode 位置法）与 nether legacy_random 专用路径都是 df_ns 硬编码特判（"end"/"overworld" 字符串比较） | nether/end【实测】（nether 生成 chunk + overworld 回归 95.40%；end 接管 judge+confirmed）；第三方 mod 维度【纸面】（docs/09 自注「需实际 mod 验证（未启动）」） | worldgen_handle.rs L210-223, L368, L399-417；data-driven-boundary.md「多世界参数化」节；docs/09-multi-dimension.md L60；knowledge/INDEX.md 260906-04 段（#57/#58）；10-timewise L3004 |
| 4 | **加自定义方块（内容 mod）** | **realmod 已端到端实测**（Forge 47.4.5 + Sinytra Connector 生产口径 PASS，用户 confirmed）：`wg_register_block_id` 显式 java_raw 同域注册（候选 B，消除映射表）；default_block 消费点惰性按名解析（#79） | 已知坑（全部有判据签名）：① **`-Dcpp.blockRegister` 门控默认关**，生产裸 java 无携带 → 注册链下半断 → 整片 AIR（#32，修复 = user_jvm_args.txt，limit=1 只注册首个 mod 块）；② 创建期解析 mod 方块名 miss → 惰性解析是必要条件（#79）；③ 注册时同域化前提 = 注册时可拿权威侧 id 且域无碰撞（#21/#22）；④ dev 载体须 devlibs jar、生产须 remapped jar（#31） | **【实测】**——dev e2e（260907-09）+ Forge+Connector 生产 e2e（260907-10，test_brick 672 命中/144 sections，三元组 sha 一致） | knowledge/INDEX.md 260907-08/09/10 段；compiler-idioms #6/#21/#22；build-tooling #31/#32；workflow-patterns #79/#80；.artifacts/realmod-e2e-260907-09.md、forge-prod-e2e-260907-10.md |
| 5 | **改地形高度 / min_y** | min_y/noise.height 从 settings JSON 读（数据驱动）；height（世界实高）= Java 显式传参；carver YOffset above_bottom/below_top、surface 锚点、aquifer 层带、placement 高度限全部参数化消费 ctx.min_y/height | ① world_height 参数错 = chunk 形状错（同 #58）；② est/interpolated 域按 min_y+noise_height 扫描——noise.height 与维度实高不一致时行为依赖调用方传对参数；③ 无 overworld 专用 min_y 残留（诊断 bin 里硬编码 -64/384 属 bin-diag，非生产路径） | min_y/height 参数化【代码】+ nether(min_y=0)【实测】；非 vanilla 高度组合（如 mod 超高维度）【纸面】 | worldgen_handle.rs L210-223；carver.rs L66-96；placement.rs L307-360；surface_rules.rs L1090-1096（锚点）；data-driven-boundary.md |
| 6 | **自定义 feature / 放置** | placed_feature/configured_feature **JSON 懒加载，路径按 id 命名空间**（`data/<ns>/worldgen/<kind>/<short>.json`，260907-05）；feature tag 展开（RuleTest base_stone_overworld 等）260907-04 起数据驱动（tags/blocks JSON 优先，fallback 兜底 + golden 等值测试）；**但 feature 类型分派是代码硬编码白名单**：tree/random_selector/random_patch/flower/simple_block 等——**mod 自定义 feature 类型（Java 代码 feature）无法消费** | ① mod feature 若最终落到支持类型的 configured_feature（JSON 定义）→ 理论可走通【纸面】；② Java 代码实现的 feature 类型 → 不支持（解析走告警兜底）；③ `minecraft:netherrack` 1.20.1 无 tag 文件，该消费点恒走 fallback 硬编码（已知例外）；④ 内联 placed/configured 对象形态已修（发现 #16/#8 家族，260905-12）；⑤ biome_temperature 表硬编码 vanilla 名单，mod biome 默认 0.5（→TempCond false）——mod 冷 biomes 的 surface 温度分支判定失真【纸面】 | vanilla feature JSON 链【实测】；mod 命名空间 feature 路径解析【代码】（260907-05 缺口 3，无 e2e）；自定义 feature 类型【硬撞点，代码白名单】 | feature_loader.rs L60-71, L203-207；data-driven-boundary.md（tag 数据驱动 + netherrack 例外）；surface_rules.rs L1488-1517（biome_temperature 硬编码表）；compiler-idioms #16 |
| 7 | **数据包驱动的 worldgen 资源（datapack 形态）** | CoreSwap 消费的是 **wgDir 解包目录布局**（`<dir>/data/<ns>/worldgen/...` + 平级 noise_params.json/biome_params.json/blocks.json），不是 datapack zip，也不读 pack.mcmeta/registry JSON（如 multi_noise.json 源文件、dimension JSON、world_preset） | ① mod datapack 形态的 worldgen 资源需**外置转换/搬运**成 wgDir 布局 + CoreSwap 自有格式（biome_params box 格式）；② noise_params.json / blocks.json 为共享平级文件，多维度共用，mod 追加条目须合并进同文件；③ dimension/world_preset/.chunk generator 类型（flat/debug 等）不在 Rust 接管范围 | 布局消费【代码+实测】（vanilla 数据目录 + nether/end）；datapack 直读【不支持，代码无此入口】 | worldgen_handle.rs L171-175, L262-263, L358-360；docs/09-multi-dimension.md L93-112 |

## 二、横切撞点（跨改动类型）

- **接管集硬编码（Java 侧）**：settings id → 接管/放行判定清单在代码里，无声明入口。任何「mod 覆盖 vanilla 同 id 数据」场景（改 overworld 高度、换 overworld surface）现状 = 放行 vanilla 或被 overworld 代码规则截和——**这是开工点 5「摘硬编码/opt-in 清单」的核心对象**【10-timewise L3000-3004；NEXT_SESSION L40】。
- **overworld 代码 surface rule 特权**：df_ns=="overworld" 恒走 build_overworld_rule()（已验证 95%+ 对齐），settings.surface_rule JSON 被忽略——mod 覆盖 overworld surface_rule 的场景有结构性撞点【worldgen_handle.rs L399-417】。
- **biome_temperature 硬编码** + **overworld noise key 静态预载清单**（ENGINE_NOISE_KEYS，启动期断言兜底）：均为 overworld 代码规则伴生硬编码【surface_rules.rs L1488-1517；worldgen_handle.rs L150-155, L336-347】。
- **fallback 策略统一形态**：tag 展开 / default_block 缺失 = JSON 优先 + 硬编码 fallback + 一次性日志 + golden 测试——mod 数据缺失不炸生成但静默降级，判读须看日志【data-driven-boundary.md 残留说明】。
- **生产口径参数清单**：mod 方块注册链需 `-Dcpp.blockRegister`（+ limit 值）生产必带；参数清单按口径分组维护（#32/#28）【build-tooling #32】。

## 三、已实测兼容 vs 纸面推断（分账）

**已实测兼容（有端到端/运行时 artifact）**：
- overworld 全管线（基线 95.40%+ 各期口径）、nether 维度（create_for_dim，min_y=0）、end 维度接管（260906-04 judge+confirmed）
- 内容 mod 自定义方块：dev e2e + Forge+Connector 生产 e2e（260907-09/10，confirmed，judge PASS-with-conditions 条件已应用）
- 方块 tag JSON 数据驱动（260907-04，golden 等值测试）

**纸面推断（机制在、无 mod 场景实体复测）**：
- mod 自定义 noise_settings / density_function / surface_rule JSON 全链
- mod 自定义维度端到端（docs/09 自注「需实际 mod 验证（未启动）」）
- mod 自带 biome_params 完整场景（遗留 idk ①，显式在案）
- mod 命名空间 feature/biome JSON 路径解析（260907-05 代码缺口闭合，无 e2e）
- mod 维度放行路径（与 end 同源机制，未实体复测）

**硬撞点（代码白名单/特权，非数据缺失）**：
- 自定义 feature 类型（Java 代码 feature）不可消费
- overworld 同 id 覆盖时 surface_rule JSON 被代码规则取代
- 接管集硬编码 → mod 自定义 settings id 默认放行 vanilla（非崩溃，但「不接管」）
