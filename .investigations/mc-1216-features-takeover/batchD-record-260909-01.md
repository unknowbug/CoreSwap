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

## 遗留 → Phase 2.5

- 冻结序对拍（Java vanilla features 基线 vs Rust features，#67 判据）；
- 残差 9 项（缓装 5 + 嵌套暴露 4）在差集中按 §9.7 单列；
- 默认 mask 翻转（0b011→0b001）**待 judge + 用户 confirmed 后**再动（当前回退通路 = 删 -D 参数）。
