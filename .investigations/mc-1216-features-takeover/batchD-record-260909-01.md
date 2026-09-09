# batchD 记录 — mc-1216 features 接管切换（Java mixin + stageMask 切换）

> 日期锚：260909-01（Get-Date 2026-09-09 02:36-02:45，跨夜工作块续 260908-16）。
> 状态：实装 + 冒烟核验完成（draft）；Phase 2.5 冻结序对拍 + judge 待做。
> 注：runtime/ 目录按仓库 .gitignore:95 不入库（既有设计）——本文件为批次 D 的追踪证据载体。

## 改动清单（工作区 runtime/1.21.6/java，未入库）

1. `src/main/java/wg/bench/mixin/ChunkGeneratorFeaturesMixin.java`（新增）：
   - @Mixin(ChunkGenerator) 注 `generateFeatures(StructureWorldAccess, Chunk, StructureAccessor)HEAD` cancellable；
   - 门控四重：`CppBridge.rustFeaturesTakeover()` ∧ `instanceof NoiseChunkGenerator` ∧ settings id ∈ {overworld,nether,end} ∧ 形状匹配 + 对应句柄 active（与 NoiseChunkGeneratorMixin.buildSurface 同口径）；
   - 行为化哨兵（#81）：`[Mixin] generateFeatures skipped (rust takeover) dim=... count=N`，首拦 + 每 4096 打点（AtomicLong）。
2. `CppBridge.java`：新增 `rustFeaturesTakeover()` = `enabled && mask!=0 && (mask&0b010)==0`——排除 mask=0（all=双跑对照意图）防误判；单 flag 双向切换，默认 0b011 现状不变，回退 = 删 -D 参数。
3. `coreswap.mixins.json`：注册 ChunkGeneratorFeaturesMixin（#40 纪律：json 与类同步）。

## 选型核对结论（scout-c 候选 a 修正）

- 候选 a 原样（@Mixin(NoiseChunkGenerator) 注 generateFeatures）**不可行**：NoiseChunkGenerator 未声明该方法（仅基类 ChunkGenerator.java:333 与 DebugChunkGenerator 声明，mc_src_extract grep 实证）→ method not found（#25/#55 家族「编译过 ≠ 命中」镜像面，此处是「注入即失败」）。
- 候选 b（ChunkGenerating.generateFeatures 步骤级）否决：连坐 Heightmap.populateHeightmaps（POST_CARVER 四图重建）+ tickLeavesAndFluids 副作用。
- **采用**：@Mixin(ChunkGenerator) HEAD + instanceof 门控——只 cancel features 本体，副作用面最小。

## 冒烟核验（runServer -PcppReplace -PrustStages=1，seed -8248318472910187742，fresh world，RCON forceload -64,-64→63,63）

| 判据 | 结果 |
|---|---|
| gradle build | BUILD SUCCESSFUL（defaultRequire=1 下 AP 通过 = 注入目标存在性编译期验证）|
| mask 传播 | `[CppBridge] init/initNether/initEnd ... stageMask=1` 三句柄全 1（bit1 清 = Rust features 执行）|
| Java 让位 | `[Mixin] generateFeatures skipped (rust takeover) dim=minecraft:overworld chunk(-6,4) count=1` 命中 |
| 既有接管不受扰 | populateNoise/buildSurface intercepted/skipped 照常 |
| 异常面 | 无 APPLY FAILED、无 mixin 异常、RCON stop 干净关服 |

验证分层声明（§9.7）：本核验 = **Partial**（接管生效行为化哨兵 + mask 回读；未做生成内容级对拍）。Rust features 执行体 = 与 native bin-diag 同代码路径（fill_chunk_blocks apply_features），其行为对齐由 Phase 2.5 冻结序对拍承担。

## 踩坑记录（五段式简记）

- **现象**：`gradle runServer --nogui` 失败「Unknown command-line option '--nogui'」。
- **根因**：#9 旧知识（--nogui 非 runServer CLI 选项）在本 project 依然成立——dedicated server 任务本就无 GUI。
- **教训**：开工前先查 knowledge/discovered/build-tooling.md #9——本轮先踩后查，检索顺序倒置。

## v2 修正（260909-02，Phase 2.5 首轮对拍实证）

- **缺陷（batchD-E1）**：v1 `@Inject HEAD cancellable ci.cancel()` 连坐了 generateFeatures 内的**结构方块放置段**（ChunkGenerator.java:360-379：结构 `start.place(...)` 与 feature 迭代同方法同循环，结构段在前）——mixin 生效臂丢失全部结构件放置。首轮双臂对拍实锤：vault/trial_spawner/waxed copper 族/rails/chest/spawner/cobweb 大量差异（A vs B terrain 5.05M 中结构块显著）。
- **根因（选型时盲区）**：选型核对只查了「方法在哪个类声明」（注入即失败面），没查「方法体内还有什么」——cancel 粒度 = 整方法，而方法不止 features。「注入即失败」检查单需追加第三查：**cancel 影响域 = 方法体内全部副作用清单**。
- **修复（v2）**：HEAD 仅做门控判定（ThreadLocal，不 cancel）+ `@Redirect` 精准拦截 `placedFeature.generate(...)` 调用点（:406）——结构放置段及 vanilla decorator seed 消费原样保留，Java feature 逐调用 no-op。build 绿（AP 通过），行为化哨兵 `[Mixin] placedFeature skipped (rust takeover) count=N` 命中（1089 chunks 拦截 65,536+ 次），vault/trial_spawner 族差异消失。
- 修正后残余信号（A vs B v2）：terrain 4.86M / veg 346k / air 189k；噪声基线（Java-vs-Java 同代码双 run）：terrain 101.5k / veg 36.0k → 信噪比 ≈48×，主残差为特征层真实分歧（候选分解 → fanout-260909-02/）。

## 遗留 → Phase 2.5

- 冻结序对拍（Java vanilla features 基线 vs Rust features，#67 判据）；
- 残差 9 项（缓装 5 + 嵌套暴露 4）在差集中按 §9.7 单列；
- 默认 mask 翻转（0b011→0b001）**待 judge + 用户 confirmed 后**再动（当前回退通路 = 删 -D 参数）。
