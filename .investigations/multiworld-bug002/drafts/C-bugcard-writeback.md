# 草稿 C：BUG 卡回写文本（E:\PYTHON\CoreSwap-Maint\.artifacts\bugs\）

> ⚠️ 路径勘误：任务给的 `BUG-002-custom-dim-skip.md` 实际不存在，卡片真实文件名为
> `BUG-002-custom-dim-surface-skip.md`（本草稿按真实文件名产出）。回写方式 = 原卡片头部状态行
> 更新 + 新增「修复回写（260906-03）」小节（追加不覆盖）；INDEX.md 对应条目同步 done。
> 状态前置：修复已完成、judge 通过（candidate）——**待用户 confirmed 后才置 done**；本回写文本
> 以「状态: done（待 confirmed）」形态给出，主会话应用时可按拍板结果调整。

---

## 头部状态行（替换原第 3 行）

状态: done（待用户 confirmed）/ 2026-09-06-03 工作块修复完成，judge 审查通过（candidate）
修复载体: coreswap-1.20.1-1.0.24.jar（mixin 维度判定重构版）；主工作区验证记录 .investigations/multiworld-bug002/cmd-output/v2-forge-connector-verification-260906-03.md

## 新增小节（追加到卡片末尾）

### 修复回写（260906-03，主工作区 BUG-002 课题，judge 通过 candidate）

**机制定论**（双机制互补，取代原卡「根因」小节的单一定位）：
1. **末地 38% 裸 bedrock（原卡未解释的主因）**：mixin 旧 wgIsEnd() 用 `getDeclaredField("biomeSource")`（Yarn 字面量）做末地豁免——字符串字面量不被 remapper 重写，Forge 生产 SRG 命名下 ChunkGenerator.biomeSource 实际名为 `f_62137_`（srg_to_official_1.20.1.tsrg 行 180533 实证）→ NoSuchFieldException → 旧 catch 静默 false → 末地豁免失效，末地（min_y 0/height 256 与 nether 同形状）被 nether 句柄用 nether.json 误接管生成。dev 环境（全 Mojmap 名）反射恰好成功 → dev 测试不可见。
2. **mod 维度（aether 39%/糖果 34%/豆腐 34% 等）**：原卡「4 硬缺口」结论维持有效且为必要补全——形状不匹配的 mod 维度被 mixin 放行 vanilla 后仍损坏，根因即引擎 4 硬缺口（blocks.rs:41 unwrap_or(AIR)、default_block minecraft:stone、data/minecraft 命名空间硬编码、biome 表仅原版）；形状恰好 = 0/256 的 mod 维度则同末地路径被 nether 误接管。共同上游根因：维度识别缺失（形状指纹 + 裸反射，零 registry 识别）。

**修复描述**：NoiseChunkGeneratorMixin 维度判定从「形状指纹 + biomeSource 裸反射」重构为「noise_settings 注册表 entry key（符号引用，remap 安全）」——接管集 = {minecraft:overworld, minecraft:nether}；末地与 mod 维度一律放行 vanilla 并打一次性放行日志（`release to vanilla: settings=...`）；裸反射路径整体删除（静默 catch 同步消除）。文件：runtime/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java（头注释含机制说明）。

**验证记录**（主工作区 .investigations/multiworld-bug002/）：
- `cmd-output/v1-tsrg-biomeSource-260906-03.md`——SRG 字段名 tsrg 静态实证（f_62137_；初引 f_226623_ 系 Structure$GenerationContext 同名巧合，已勘误）。
- `cmd-output/v2-forge-connector-verification-260906-03.md`——Forge 47.4.5 + Sinytra Connector beta.49 生产环境，seed 7691421705105351955（与报告者同种子）：mixin 生产 APPLY 全绿、end `release to vanilla` + 末地零 nether-intercept、end 地形探针 vanilla 化（y55-58=end_stone、bedrock 未命中）、nether/overworld 接管不回归——7 判据全 PASS。

**降级声明**（诚实边界，§9.7）：
- mod 维度（aether 等）**未本地实体复测**——其放行与 end 走同一代码路径（settings id ∉ 接管集），机制同源但未在本地装载 mod 维度验证。
- 未做报告者口径的全维度 region 逐方块 A/B diff——完整 A/B 验收需**报告者以修复版复测**（或本地补装 aether 级 mod 重跑）；本环境为 vanilla 三维度 + mod 加载器环境。
- 引擎 4 硬缺口（本卡原「根因」小节）未在本轮修复——属长期「多世界数据驱动」工作线（mod 方块 JNI 注册为关键项），放行 vanilla 后 mod 维度由原版管线生成，不再依赖这些缺口。

**建议后续**：① 报告者复测确认后回填 A/B 数据；② end 接管 = 下一里程碑（Rust end 管线 + end biome 判定先行核对）；③ 长期多世界数据驱动线接续原卡「修复提案-长期」。
