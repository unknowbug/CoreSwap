# 草稿 A：knowledge/discovered/workflow-patterns.md 追加条目（发现 #55）

> 用法：追加到 knowledge/discovered/workflow-patterns.md 末尾（现最大号 #54，本条 = #55），并同步 INDEX.md。主会话应用 + 验证。

---

## 发现 #55: mixin 对父类字段的 getDeclaredField 字符串字面量不被 remapper 重写，Forge 生产 SRG 命名下必失败；跨加载器生产安全的维度/资源识别应使用注册表 entry key（符号引用）而非反射字符串（260906-03）

- **时间/置信度/module**：260906-03；candidate（源码链完整 + tsrg 映射表双源实证 + Forge+Connector 生产环境修复后行为日志全链 PASS，§9.7 边界声明随行——mod 维度未本地实体复测）；workflow-patterns / modding 环境坑（SRG/refmap 家族，与 BUG-001 Fabric intermediary 反射失败同域不同机制）。

### 现象（现象→根因）
BUG-002（issue #24）：替换模式下非主世界维度大面积损坏——末地 38% 裸 bedrock、aether 39%、糖果/豆腐 34% 等（同种子 7691421705105351955 region 逐方块 A/B）。**dev 环境测试全绿，仅生产环境（Forge 47.4.5 + Sinytra Connector）复现**。末地 chunk 被打上 `populateNoise(nether) intercepted`（nether 句柄误接管）。

### 根因（机制层面）
`NoiseChunkGeneratorMixin.wgIsEnd()` 用 `getDeclaredField("biomeSource")`（NoiseChunkGeneratorMixin.java 旧 L30，Yarn 名）做末地豁免。**mixin/Connector 的 remap 管线只重写常量池符号引用（类/字段/方法引用、注解方法签名），不重写方法体内的字符串字面量**——`"biomeSource"` 以 Yarn 拼写原样到达生产运行时。而 Forge 1.20.1 生产命名 = 官方类名 + **SRG 成员名**（`f_XXXXX_`/`m_XXXXX_`；srg_to_official_1.20.1.tsrg 行 180533 实证 `f_62137_ biomeSource`），运行时不存在名为 `biomeSource` 的字段 → `NoSuchFieldException` → 旧代码 `catch (Throwable) { return false; }` **静默**返回 false → 末地豁免失效，末地（min_y 0/height 256 与 nether 同形状）被 nether.json 句柄用 nether 的 noise_settings/surface_rule 生成 = 裸 bedrock。dev 环境（loom/ForgeGradle dev runtime = 全 Mojmap 名）`biomeSource` 拼写恰好与 Mojmap 同名 → 反射成功 → **dev 全绿不构成生产证据**。mod 维度分支同根因的另一面：形状不匹配 → 侥幸 vanilla 放行（水晶 0.8%/混沌 0.02% 正常 = 侥幸正确而非机制正确）；形状恰好 = 0/256 的 mod 维度将复现末地式灾难。

### 定位（怎么发现的）
① scout 勘探（只读源码全链）发现维度判定只有形状指纹 + 裸反射、零 registry 识别（scout-map-260906-03.md）；② worker 静态裁决：SRG 机制链 + 「dev 恰好同名所以 dev 看不见」精确解释 dev/生产分歧（worker-verdict-260906-03.md）；③ V1 廉价实证：grep ForgeGradle 缓存 tsrg，`f_62137_ biomeSource` 一锤定音（**勘误教训：首次引用误取 f_226623_——那是 Structure$GenerationContext 同名字段巧合，judge 独立重跑类段归位后勘误；tsrg 未收窄 grep 会命中多类同名字段，必须归位到目标类段再引用**）；④ V2 生产复验：修复 jar 在 Forge+Connector 真机，`release to vanilla: settings=minecraft:end` + 末地零 intercepted + end_stone 探针（v2-forge-connector-verification-260906-03.md）。

### 修复
维度识别改用 **noise_settings 注册表 entry key（符号引用）**：`((NoiseChunkGenerator)(Object)this).getSettings().getKey().map(k -> k.getValue().toString())`，仅 `minecraft:overworld`/`minecraft:nether` 在接管集；末地与 mod 维度放行 vanilla 并打一次性放行说明日志（防刷屏 Set 去重）。裸反射 wgIsEnd() 整体删除（连同静默 catch，「不吞异常」违规同步消除）。RegistryEntry key 是符号引用，remap 管线正确处理——机制上不可能再被命名环境击穿。

### 教训/判据
1. **反模式签名（MUST）**：mixin/mod 代码中对（父类）字段做 `getDeclaredField("<字面量>")` = 生产定时炸弹——字符串字面量不被 remapper 重写，Forge 生产 SRG 成员名下 `NoSuchFieldException` 必现；若 catch 静默吞掉则 dev 永远测不出来。Fabric intermediary 环境（BUG-001）同理：任何「硬编码映射名 + 反射」组合都跨不过生产命名。
2. **验收判据（MUST）**：接管类/注入类修复的验收 = **生产命名环境（SRG 载体，boot 日志含 `server-...-srg.jar` 即确认）运行时行为日志**；dev 环境全绿不构成生产证据。带生产风险的判别可先用 tsrg 映射表 grep 做零成本静态实证（V1 模式），再上真机（V2 模式）。
3. **生产安全的识别方式**：跨加载器/跨 remap 环境做维度/资源识别，用注册表 entry key / ResourceKey 等**符号引用**，不用反射字符串字面量；识别不到就放行 vanilla + 打日志，禁止静默默认。
4. **refmap 包路径笔误教训**：mixin 重写时目标方法签名的包路径笔误（`world/gen/chunk/Chunk` 应为 `world/chunk/Chunk`）→ refmap 无法重映射该类名 → 生产首启 **APPLY FAILED** 整个 mixin 失效——refmap 错误只在生产（需要重映射的）环境暴露，dev 不重映射所以不报；mixin 改动后生产首启必查 APPLY 结果。
5. **tsrg 引用纪律**：tsrg grep 同名字段会命中多个类段（`biomeSource` 至少 4 类），引用具体 `f_NNNNN_` 编号前必须归位到目标类段，否则张冠李戴（f_226623_ 误引实录）。

### 证据
`.investigations/multiworld-bug002/scout-map-260906-03.md` + `worker-verdict-260906-03.md` + `方案-260906-03-放行策略.md`（f_62137_ 勘误后引用）+ `cmd-output/v1-tsrg-biomeSource-260906-03.md`（tsrg 行 180533）+ `cmd-output/v2-forge-connector-verification-260906-03.md`（7 判据 PASS + §9.7 降级声明）；修复代码 `runtime/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`（头注释含机制说明）。
