# 知识库草稿 — mc-1216-features-takeover / 260909-02（subagent 产出，主会话应用）

> 日期标签：260909-02（真实日期已核 Get-Date 2026-09-09，本块为当日第 2 工作块；260909-01 为跨夜批次 D 块）。
> 状态：全部草稿 draft，置信度沿用各证据源（verdict 未 judge、confirmed 未授）；主会话应用前请复核编号无并发占用。
> 自检：已过 SUBAGENT-KNOWLEDGE-GUIDE §〇 价值门（逐条标注）+ §四清单。

## 目标文件清单（主会话应用点）

| # | 内容 | 载体 | 动作 |
|---|---|---|---|
| A1 | 发现 #91 注入点第三查 | knowledge/discovered/workflow-patterns.md | 末尾追加 |
| A2 | 发现 #92 噪声基线前置 + 共现相关分析 | knowledge/discovered/workflow-patterns.md | 末尾追加 |
| A3 | 发现 #93 contains 子串分发反模式 | knowledge/discovered/workflow-patterns.md | 末尾追加 |
| A4 | 发现 #94 简报字段名三连 → 一手源核对 + 双版交叉核对 | knowledge/discovered/workflow-patterns.md | 末尾追加 |
| A5 | 发现 #95 scout 结论前提廉价验证（前提错结论对形态） | knowledge/discovered/workflow-patterns.md | 末尾追加 |
| B1 | 发现 #43 (k,p,fid) 序列探针 | knowledge/discovered/build-tooling.md | 末尾追加 |
| B2 | 发现 #44 features mask 单 flag 双向切换语义 | knowledge/discovered/build-tooling.md | 末尾追加 |
| C1 | 260909-02 时间线块 | versions/1.21.6/docs/10-timewise-archive.md | **文件不存在，需新建**（1.21.6 下当前只有 data/ 与 rust/；格式参照 versions/1.20.1/docs/10-timewise-archive.md 条目风格，首部可加一行说明「1.21.6 时间线，格式同 1.20.1」） |
| D1 | INDEX.md 追加行 | knowledge/INDEX.md | 末尾追加（文本见 §四） |

注：编号 #91-#95 / #43-#44 为草稿建议值，基于「workflow-patterns 现最大 #90、build-tooling 现最大 #42」（本块已实读两文件末尾核实）；若主会话有并发追加请顺延。

---

## 一、workflow-patterns.md 追加草稿

### 发现 #91（最高价值·错误优先）: mixin cancel/接管注入点三查——「注入即失败面」之外 MUST 追加「cancel 影响域 = 方法体内全部副作用清单」（260909-02）

- **时间/置信度/module**：260909-02；candidate（v2 修复后双臂对拍 vault/trial_spawner 族差异消失，行为级证据；verdict judge 未做）；workflow-patterns / mixin 接管注入选型（#55 注入面家族的方法体内维度扩展，batchD-E1）。
- **来源定位**：`.investigations/mc-1216-features-takeover/batchD-record-260909-01.md` v2 节（五段式）+ `.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md` §一.1。
- **现象**：v1 `@Inject(ChunkGenerator.generateFeatures) HEAD + ci.cancel()` 编译/AP/冒烟全绿（接管哨兵命中、mask 回读正确），但双臂对拍实锤 Rust 臂**丢失全部结构件放置**——vault/trial_spawner/waxed copper 族/rails/chest/spawner/cobweb 大量差异（A vs B terrain 5.05M 中结构块显著）。
- **根因（机制）**：cancel 粒度 = **整方法**，而 `generateFeatures`（ChunkGenerator.java:360-423）内**结构方块放置段（:360-379，结构 `start.place`）与 feature 迭代同方法**——HEAD cancel 连坐把结构段一起砍掉。选型核对只做了「注入即失败面」检查（方法在哪个类声明：NoiseChunkGenerator 未声明该方法 → 改 @Mixin 基类），没查「方法体内还有什么」。
- **定位**：首轮双臂 region diff 按块家族分类 → 结构语汇块（vault/chest/spawner/rail）单侧缺失且坐标与结构实例吻合 → 回读 ChunkGenerator.java 方法体逐段核对副作用清单。
- **修复（v2）**：HEAD 仅做门控判定（ThreadLocal，不 cancel）+ `@Redirect` 精准拦截 `placedFeature.generate(...)` 调用点（:406）——结构段与 vanilla decorator seed 消费原样保留，仅 Java feature 逐调用 no-op。行为化哨兵 `[Mixin] placedFeature skipped count=N`（1089 chunks 拦截 65,536+ 次），结构族差异消失。
- **教训/检查单（可复用）**：mixin cancel/接管选型三查 MUST 齐备：① **方法在哪个类声明**（注入即失败面，@Mixin 目标不存在 = method not found）② **注入点存在性/命中证据**（编译过 ≠ 命中，#25/#55 家族）③ **cancel 影响域 = 方法体内全部副作用清单**（cancel 粒度是整方法；「主目标段在方法后半段」不构成前段豁免）。能用调用点级 @Redirect/@Inject 局部化的，不用整方法 HEAD cancel。
- **证据**：batchD-record v2 节五段式全段；ChunkGenerator.java:360/379/402/406 行号锚。

### 发现 #92: 区域级大残差先跑「同代码双 run 噪声基线」，特征块共现相关分析可裁决「级联 vs 独立层分歧」（260909-02）

- **时间/置信度/module**：260909-02；candidate（#67/#51 家族量化延伸 + 本块双实测）；workflow-patterns / 对拍信号分解方法（#51/#67/#15 家族）。
- **来源定位**：`.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md` §一.2/.3 + `.investigations/mc-1216-features-takeover/fanout-260909-02/b2-feature-algo.md` §二。
- **观察**：① Java-vs-Java 同代码双 run 噪声基线：terrain=101,529 / veg=35,967（764/3410 chunk-slots）→ run 级非确定性只解释 Java-vs-Rust 信号（terrain 4.86M / veg 346k）的 **~2%**，信噪比 ≈48×，主残差为真实分歧；② stone 族 3.98M 残差中 **89.5% 与 ore 指示 section 共现、75.5% 与 disk/lake 共现，独立 section 仅 3.4%** → stone 族是特征位置偏移的**级联**（ore/disk 换位后原位回填/暴露差），非 NOISE/surface 层分歧（b3 候选消解）。
- **根因（方法论）**：区域级大残差有两种成因方向（测量噪声 vs 真实信号）与两种机制形态（独立层分歧 vs 上游分歧的下游级联），不分离就逐族瞎查。噪声基线 = 同代码双 run，成本低（复用双臂驱动，多跑一臂）；级联判别 = 对可疑块族做**与特征指示块的共现相关分析**（离线脚本，零采集成本）。
- **如何利用**：① 区域 diff 出大数字后，**第一动作先跑噪声基线**定信号下限（§9.7 口径：同载体同协议双 run），残差 ≈ 噪声量级即消解，不立案；② 「块 X 大量差异」先算 X 与各特征指示 section 的共现率——共现率高（>80%）判级联、独立率低（<5%）排除独立层分歧，归因直接收窄到上游特征位置层；③ 共现分析结论要配「独立 section 残差仍需解释」的剩余账（本例 3.4% 仍留 §9.7 单列）。
- **证据**：diff-noise-baseline.txt / diff-signal-v2.txt + correlate 脚本输出（.tmp/mc1216-closeout-260909-02/，不入库）；数量见 verdict §一.2/.3。

### 发现 #93: 注册名 contains 子串分发反模式——高频短子串误捕语义无关类型，静默配置全错（260909-02 收编批次 0）

- **时间/置信度/module**：260909-02 收编（原始发现 batch 0，修复已落地）；candidate；workflow-patterns / 分发反模式（#56「新分支落 catch-all 后」家族的**分发谓词**对偶形态）。**价值门：高（反模式 + 全量排查法 + 验收面）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/batch0-worker-delivery.md`（误捕面全量排查表 + E 段五段式）+ `scout-b-diff清单.md`。
- **现象**：`contains("ore")` 误捕 `forest_rock`（f-**ore**-st）与 `nether_forest_vegetation` → 走 OreFeatureConfig 解析、配置全错、无告警；placement 侧 `contains("count")` 吸进 `count_on_every_layer`（语义完全不同的 modifier，消费序不同）。
- **根因**：注册名分发用 contains 子串匹配，短高频子串（ore/count）是「子串碰撞富矿」；命中即静默走错分支，catch-all 哨兵收不到（没落 unknown）。
- **定位**：contains 各分支 × 一手注册表（1.21.6 Feature.java:27-121 / PlacementModifierType.java:8-30）全量排查矩阵，逐分支列出误捕全集。
- **修复/判据**：分发一律完整 `== "minecraft:xxx"` 精确匹配 + catch-all 告警；**新增类型时 catch-all 哨兵集合就是回归验收面**（每条已知 type 应在哨兵集合外各归其位）。
- **教训**：任何「按名字符串分发」的映射表接入新数据集时，先做「子串误捕全量排查」（分支谓词 × 数据集 type 全集），再谈逐分支语义正确。
- **证据**：batch0-worker-delivery 误捕面排查表（含 disk/spring/underwater_magma/freeze_top_layer 零误捕对照行）。

### 发现 #94（简记，三连实证）: 任务简报字段名/分类词不进一手源核对不可用——1.20.1/1.21.6 双版交叉核对升级形态（260909-02 收编批次 A/B/C）

- **时间/置信度/module**：260909-02 收编（batchA E-1 / batchB E-1 / batchC §四 同族三连）；candidate；workflow-patterns / 简报信息纪律（#90 转抄漂移家族的「简报→parse」形态）。**价值门：高（判错经验：凭记忆字段名写字段 → 恒 miss → 静默全默认值/全灭）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/batchA-worker-delivery.md` §E-1、`batchB-worker-delivery.md` §E-1、`batchC-worker-delivery.md` §四（E-3 教训 + E-4）。
- **要点**：三批独立复犯同型错误——简报凭记忆/旧版混杂给的字段名（「noise_level min/max、half_cost」「count 1..25 非 IntProvider」「selector 分类词」）与 1.21.6 一手 codec 不符（MultifaceGrowthFeatureConfig.java:20-36 / CountConfig.java:9-25 实为 IntProvider 等）；若照写 parse，字段恒 miss → config 全默认值或特征静默全灭，且难察觉。**判据：「先读 codec 再写字段」与「先读文件再写 patch」同级**；引擎目标 1.20.1 有本地 extract 时必须 1.20.1/1.21.6 双版都读（防 1.21.6 独有改动静默混入）。
- **证据**：三份 worker-delivery E 段（含双版行号锚）。

### 发现 #95: scout 结论的「事实前提」与「结论」要分开验证——前提部分不成立时结论可能仍成立，但证明链必须补全（260909-02 收编批次 B）

- **时间/置信度/module**：260909-02 收编（batchB E-2）；candidate；workflow-patterns / 交接结论验证纪律（§16.3 / STEP 1 廉价独立验证的「前提/结论分离」细化形态）。**价值门：高（可复用判错姿势）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/batchB-worker-delivery.md` §E-2 + §一.0.3 证明链。
- **要点**：scout-b 报告「feature 阶段只有两张高度图 → MOTION_BLOCKING 塌缩不可修」——前提半错（CARVERS 起实挂 POST_CARVER 四图，ChunkStatus.java:33-35），但**塌缩结论本身成立**（特征时点地形无树叶/植被态，MOTION_BLOCKING 与 WORLD_SURFACE 的差集在引擎方块分类下为空集，两图逐列同值）。教训：① 继承 scout 报告的机制方向前，把「事实前提」与「结论」拆开各自做廉价验证（本次 = ChunkStatus/Heightmap 两文件核对）；② 「前提错但结论对」时**证明链必须写全**（逐 type 等价表），否则下次还会再验一遍；③ 「不可修」类断言的权重全部压在前提上，前提错则断言整个失效，优先验前提。
- **证据**：batchB-worker-delivery §E-2 + §一.0.3 逐 type 表。

---

## 二、build-tooling.md 追加草稿

### 发现 #43: (k,p,fid) 单 chunk 序列探针——decorator seed 域错位的决定性判据；MixinExtras 不可用时的纯 Mixin 替代（260909-02）

- **时间/置信度/module**：260909-02；candidate（step 1-8 共 40 条完全一致 + step 9 p 错位实锤定位根因，behavior 级探针证据）；build-tooling / 探针工具（#81 行为化日志家族的序列化形态 + #62 seed 判据的调用点级落地）。**价值门：高（判据 + 工具做法直接复用）**。
- **来源定位**：`.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md` §一.4 + `.investigations/mc-1216-features-takeover/fanout-260909-02/b1-rng-wiring.md` §4（探针设计）。
- **做法**：单 chunk 双侧对拍 `(population_seed l, step k, index p, fid)` 全序列：Java 侧 mixin 在 ChunkGenerator.generateFeatures :402 `setDecoratorSeed(l, p, k)` 处打印四元组 + registryKey（本块实现 = **@Redirect placedFeature.generate 调用点**顺带打印——MixinExtras 编译期不可用时的纯 Mixin 替代；另 `setCurrentlyGeneratingStructureName` 供应商重定向可拿结构 fid，本块结构段走种子隔离不需）；Rust 侧 `WG_FEATURELOG=1` 现成钩子（worldgen_handle.rs:1080）。
- **判据（三层直接命中）**：`l` 不一致 → seed 接线差（worldSeed/公式）；`l` 一致、`(p, fid)` 序列有差 → **p 域输入差**（registry 序文件/biome feature list 构建序）；序列完全一致 → RNG 接线整体排除，归因转特征实现层。本块实测：step 1-8（40 条）完全一致，step 9 特征集相同（13 fid 一致）但 p 错位（trees_water J=50/R=28、flower_default J=57/R=53 等）→ 根因钉死为 **Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致**。
- **教训**：① 「全局位置整体错开」类症状，序列探针一步区分「种子域错位」vs「算法实现差」，先于任何逐族算法对拍；② p 索引差（数值不同但 fid 集合相同）是**种子域错位的决定性签名**——特征集合一致恰恰排除了「缺 feature」候选；③ 静态源码对读（b1 候选 Degraded）已把公式/类型/迭代序三层核到同构后，剩余疑点收敛到「输入数据域」，探针设计应直接对准该域。
- **证据**：probe-{javafeat-log,rustfeat-err}-snapshot.log（.tmp/mc1216-closeout-260909-02/）；mixin 代码 runtime/1.21.6/java ChunkGeneratorFeaturesMixin（不入库，batchD-record 为追踪载体）。

### 发现 #44（简记）: 接管开关单 flag 双向切换语义——mask=0 必须显式排除出「接管生效」判定（260909-02 收编 batchD）

- **时间/置信度/module**：260909-02 收编（batchD 改动 2）；candidate；build-tooling / 接管开关设计。**价值门：中（简记——跨版本复制接管开关时的语义陷阱）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/batchD-record-260909-01.md` 改动清单 2。
- **要点**：`rustFeaturesTakeover() = enabled && mask!=0 && (mask&0b010)==0`——**mask=0（全 Java = 双跑对照意图）必须显式排除**，否则「无 mask 参数」与「mask=0」两种语义被合并，对照臂误走接管路径；单 flag 双向切换（默认 0b011 现状不变，回退 = 删 -D 参数）。翻转前置（verdict §二）：p 域对齐 + 已知 feature 族缺陷修复 + 重对拍信噪比回噪声量级，**残差未收敛前不翻转**。另：接管开关生效判定要配行为化哨兵计数（首拦 + 周期打点），纯布尔回读不构成生效证据（#81 家族）。
- **证据**：batchD-record 改动清单 + verdict §二（mask 翻转不建议，前置清单）。

---

## 三、时间线追加草稿（C1）

> ⚠️ 目标文件 `versions/1.21.6/docs/10-timewise-archive.md` **当前不存在**（1.21.6 下只有 data/ 与 rust/，主会话 glob/pwsh 双核实）。建议：新建该文件，首部加一行「# 1.21.6 时间线（格式同 versions/1.20.1/docs/10-timewise-archive.md）」后接下方内容。若主会话决定 1.21.6 时间线归口到别处（如并入 1.20.1 归档），请自行改挂——以下条目文本两种归口通用。

```markdown
## 260909-02（实际 2026-09-09 Get-Date 锚定：mc-1216 features 接管 Phase 2.5——双臂对拍 → batchD-E1 发现修复 → 噪声基线 → 共现相关 → fan-out → 序列探针根因定位 → mask 不翻转建议）draft（verdict judge 未做，confirmed 未授）

> 过程产物 `.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md`（verdict 主文档）+ `.investigations/mc-1216-features-takeover/fanout-260909-02/{b1-rng-wiring,b2-feature-algo}.md`（fan-out 双候选）+ batchD-record-260909-01.md v2 节；数据 `.tmp/mc1216-closeout-260909-02/`（不入库）；mixin 探针 runtime/1.21.6/java（不入库，batchD-record 为追踪载体）；通用模式 → workflow-patterns #91/#92 + build-tooling #43（subagent 草稿 → 主会话应用）。

- ✅ **双臂对拍 v1 → batchD-E1 发现（P1）**：v1 HEAD cancel 连坐 generateFeatures 内结构方块放置段（ChunkGenerator.java:360-379 与 feature 迭代同方法）→ Rust 臂丢全部结构件（vault/trial_spawner/waxed copper/rails/chest/spawner/cobweb 大量差异）。v2 = HEAD 仅门控 + @Redirect placedFeature.generate（:406）精准让位，结构族差异消失。注入点第三查沉淀 → workflow-patterns #91。
- ✅ **噪声基线（#67/#51 家族量化）**：Java-vs-Java 同代码双 run terrain=101,529 / veg=35,967 → run 级非确定只解释 ~2% 残差，信噪比 ≈48×。
- ✅ **共现相关分析 → b3 候选消解**：stone 族 3.98M 中 89.5% 与 ore 指示共现、75.5% 与 disk/lake 共现、独立仅 3.4% → 级联非独立层分歧（方法论 → #92）。
- ✅ **fan-out 双候选（fanout-260909-02）**：b1 RNG 接线——公式/类型/迭代序三层静态同构（population seed 有 Java 数值锚），输入域层保留 registry_order 回退残差（部分排除，Degraded）；b2 特征算法——4 族静态 PROVEN（Disk break 过早 / Geode isAir 缺 cave_air / Ore 邻 chunk 暴露判定 / Lake isSolid 含树叶）+ emerald_ore catch-all（0 RNG 消费）。
- ✅ **P-b1 序列探针 → 根因定位（decisive）**：chunk(0,0) 双侧 (k,p,fid) 对拍，step 1-8 共 40 条完全一致；step 9 特征集相同（13 fid 一致）但 p 全体错位（trees_water J=50/R=28 等）→ decorator seed=f(l,p,k) 域错位 → 植被整体偏移+级联。根因 = Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致（候选输入差：biome_registry_order 覆盖面 biome.rs:414 字典序回退 / biome feature list 构建序）。探针做法 → build-tooling #43。
- ⚠️ **次级实现缺陷 5 项（b2，静态 PROVEN，位置修复后逐一可验）**：Disk break（feature.rs:467-473）/ Geode isAir（feature.rs:2006）/ Ore isExposedToAir（feature.rs:358-369）/ Lake isSolid（tree.rs:969-981）/ emerald_ore catch-all——详见 fanout b2 §一。
- ✅ **mask 翻转建议 = 不翻转（0b011 维持）**：step 9 p 错位未修复前翻转 = 全域植被换位 + 4 个已知 feature 族缺陷上线。翻转前置 = indexer p 域对齐 + b2 五缺陷修复 + 重对拍信噪比回噪声基线量级。
- 🔍 **open**：indexer p 域根因修复（biome.rs:414 回退面 / feature list 构建序）；b2 五缺陷逐一修复+探针验证；b2 探针 P1-P6 清单；残差 9 项 §9.7 单列（dripstone_block=237 + pointed_dripstone=34 本 region 实测；iceberg/fossil/sculk/large_dripstone 未出现）；judge 审查 verdict；confirmed 留用户。
- 口径声明（§9.7）：双臂/基线 = Chunky region 存档口径（全量三分分类）；序列探针 = 行为化日志（#81，单 chunk 覆盖面声明：step 1-8 对全 region 有代表性，step 9 在 chunk(0,0) biome 邻域成立）；b1/b2 静态结论 = Degraded。
```

---

## 四、INDEX.md 追加行草稿（D1，追加到文件末尾，不改既有行）

```markdown
> 260909-02 追加：workflow-patterns 新增**发现 #91（最高价值·错误优先）**（mixin cancel/接管注入点三查——「注入即失败面」外 MUST 追加「cancel 影响域 = 方法体内全部副作用清单」：generateFeatures 结构放置段与 feature 迭代同方法，HEAD cancel 连坐丢全部结构件，v2 = @Redirect placedFeature.generate 调用点精准让位；batchD-E1）+ **发现 #92**（区域级大残差先跑同代码双 run 噪声基线——本例 run 级非确定仅解释 ~2%；特征块共现相关分析裁决「级联 vs 独立层分歧」：stone 族 89.5% 与 ore 指示共现 → 级联，独立 3.4% → NOISE 层候选消解）+ **发现 #93**（注册名 contains 子串分发反模式——contains("ore") 误捕 forest_rock/nether_forest_vegetation、contains("count") 吸 count_on_every_layer，分发一律完整注册名精确匹配 + catch-all 哨兵集合作回归验收面；收编 batch 0）+ **发现 #94 简记**（任务简报字段名/分类词不进一手源核对不可用，batchA/B/C 三连实证；升级形态 = 1.20.1/1.21.6 双版交叉核对）+ **发现 #95**（scout 结论的事实前提与结论分开验证——前提错结论可对，证明链必须补全；batchB E-2）；build-tooling 新增**发现 #43**（(k,p,fid) 单 chunk 序列探针——Java @Redirect 调用点 + setCurrentlyGeneratingStructureName 供应商重定向（MixinExtras 不可用时的纯 Mixin 替代）+ Rust WG_FEATURELOG；p 索引差 = 种子域错位决定性判据，本块一步钉死 PlacedFeatureIndexer 植被段 lastIndex 错位）+ **发现 #44 简记**（接管开关单 flag 双向切换——mask=0 显式排除出接管判定，翻转前置 = 残差收敛到噪声量级）。来源：.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md + .investigations/mc-1216-features-takeover/{batchD-record-260909-01.md,fanout-260909-02/,batch{0,A,B,C}-worker-delivery.md}（verdict draft，judge 未做）。时间线 → versions/1.21.6/docs/10-timewise-archive.md 260909-02 块（新建文件）。
```

---

## 五、价值门处置记录（筛选说明）

| 候选 | 处置 | 理由 |
|---|---|---|
| 1 注入点第三查 | 入 #91（详写，最高价值） | 反模式 + 检查单，batchD-E1 五段式齐备 |
| 2 噪声基线 + 共现相关 | 入 #92 | 可复用判据/方法，两段实测量化 |
| 3 (k,p,fid) 序列探针 | 入 build-tooling #43（按载体指定） | 工具做法 + 决定性判据 |
| 4① contains 子串分发 | 入 #93 | 反模式 + 全量排查法（batch0 误捕面表），高价值 |
| 4② 简报字段名三连 | 入 #94（简记，三连合并） | 三批独立复犯同型，判错经验高价值；量入简记避免与 #90 家族重复展开 |
| 4③ 双版 diff 判语义 | **并入 #94 升级形态**（batchC E-4：diff 必须逐条判语义，改名型 vs 行为型） | 与 4② 同属「双版交叉核对」方法，分开则各自单薄；若主会话愿单列可拆 #96 |
| 4④ scout 前提廉价验证 | 入 #95 | §16.3 的「前提/结论分离」细化，有独立证明链形态 |
| 4⑤ mask 双向切换 | 入 build-tooling #44（简记） | 中价值——语义陷阱 + 翻转前置清单，简记即可 |
| 5 时间线 | 入 C1 | 过程记录，状态标注齐（✅/⚠️/🔍），draft 状态如实标注 |
