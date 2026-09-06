# 草稿 B：versions/1.20.1/docs/10-timewise-archive.md 追加时间线条目（260906-03）

> 用法：追加到 10-timewise-archive.md 末尾（对齐现有「## YYMMDD-##（实际 …：主题）✅ …」条目格式）。主会话应用。

---

## 260906-03（实际 2026-09-06 14:12 起工作块：BUG-002/issue #24——勘探 → 廉价验证 → 卡片 v3 合并 → 方案批准 → 实现 → Forge+Connector 生产验证）✅ 修复完成 judge 通过（candidate，confirmed 待用户）

> 过程产物 `.investigations/multiworld-bug002/`（scout-map / worker-verdict / 方案 / cmd-output V1+V2）；通用模式 → workflow-patterns #55（草稿）；BUG 卡回写 → CoreSwap-Maint .artifacts/bugs/BUG-002-custom-dim-surface-skip.md（草稿待应用）。

- ✅ **scout 勘探**（scout-map-260906-03.md）：接管拦截面全景——维度判定只有两类「形状指纹」（-64/384→overworld、0/256 且 !wgIsEnd()→nether），零 registry 识别；Rust 硬编码点 H1-H10 清单（生产端 create_for_dim 已参数化，缺口在 Java 判定层 + mod 维度数据不在资源层）；末地子疑点 = wgIsEnd() 裸反射失败静默 false → end（同 0/256 形状）被 nether 句柄误接管。
- ✅ **worker 裁决 + V1 廉价实证**：worker-verdict-260906-03.md 判定「wgIsEnd() 反射在生产必失败」——字符串字面量 "biomeSource" 不被 remapper 重写，Forge 生产 SRG 成员名下 NoSuchFieldException → catch 静默 false；V1 grep ForgeGradle 缓存 tsrg 实锤 `f_62137_ biomeSource`（行 180533）——**judge 独立重跑勘误：首引 f_226623_ 为 Structure$GenerationContext 同名巧合，勿再引用**（cmd-output/v1-tsrg-biomeSource-260906-03.md）。
- ✅ **BUG 卡 v3 合并**：卡片「引擎 4 硬缺口」（blocks.rs:41 unwrap_or(AIR)、default_block minecraft:stone、data/minecraft 命名空间硬编码、biome 表仅原版）+ 本块末地误接管机制闭环 = 双机制互补；4 缺口归长期多世界数据驱动线，不阻塞短期放行策略。
- ✅ **方案批准**（方案-260906-03-放行策略.md，000-P2，用户拍板）：**资源感知自动放行**路线——按 noise_settings 注册 id 判定（settings RegistryEntry key，符号引用）替代形状指纹 + 裸反射；数据在资源层才接管、不在则放行 vanilla（mod 维度误接管机制性不可能）；end 暂放行 vanilla，end 接管 = 下一里程碑。
- ✅ **实现**：NoiseChunkGeneratorMixin 维度判定重构——wgSettingsId()（RegistryEntry key）+ 接管集 {minecraft:overworld, minecraft:nether}；裸反射 wgIsEnd() 整体删除（静默 catch 同步消除）；放行维度打一次性说明日志（Set 去重防刷屏）；buildSurface 同步收紧。
- ✅ **V2 生产验证**（cmd-output/v2-forge-connector-verification-260906-03.md，Forge 47.4.5 + Connector beta.49，seed 7691421705105351955 与报告者同种子）：7 判据全 PASS——① mixin 生产 APPLY 全绿（首启曾 APPLY FAILED：refmap 包路径笔误 world/gen/chunk/Chunk→world/chunk/Chunk，修正二启绿）② end `release to vanilla: settings=minecraft:end` ③ 末地零 nether-intercept（旧误接管指纹消失）④ end 地形 vanilla 化（y1=air/y55-58=end_stone，bedrock 未命中）⑤ nether 接管不回归 ⑥ overworld 接管不回归 ⑦ 因果端点闭合（V2 补齐 V1 遗留的运行时日志端点）。§9.7 降级声明随行：mod 维度未本地装载实体复测、未做报告者口径全维度 region A/B diff。
- ✅ **judge 审查**：三源核对（.investigations 快照 + 代码 diff + V1/V2 验证记录）通过；f_226623_ 误引勘误；V1 边界（因果端点待 V2）已由 V2 核销。
- 🔍 **open（降级/边界）**：mod 维度放行与 end 走同一代码路径（settings id ∉ 接管集）机制同源但未本地实体复测；报告者整合包 A/B 复测待报告者执行；end 接管（Rust end 管线 + biome 判定）= 下一里程碑；overworld 变体 settings（amplified 等）现被放行 vanilla（行为较此前被 overworld.json 错误接管更正确，边界已注记）。
