# 1.21.6 时间线（格式同 versions/1.20.1/docs/10-timewise-archive.md）

## 260909-02（实际 2026-09-09 Get-Date 锚定：mc-1216 features 接管 Phase 2.5——双臂对拍 → batchD-E1 发现修复 → 噪声基线 → 共现相关 → fan-out → 序列探针根因定位 → mask 不翻转建议）draft（verdict judge 未做，confirmed 未授）

> 过程产物 `.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md`（verdict 主文档）+ `.investigations/mc-1216-features-takeover/fanout-260909-02/{b1-rng-wiring,b2-feature-algo}.md`（fan-out 双候选）+ batchD-record-260909-01.md v2 节；数据 `.tmp/mc1216-closeout-260909-02/`（不入库）；mixin 探针 runtime/1.21.6/java（不入库，batchD-record 为追踪载体）；通用模式 → workflow-patterns #91/#92 + build-tooling #43（subagent 草稿 → 主会话应用）。

- ✅ **双臂对拍 v1 → batchD-E1 发现（P1）**：v1 HEAD cancel 连坐 generateFeatures 内结构方块放置段（ChunkGenerator.java:360-379 与 feature 迭代同方法）→ Rust 臂丢全部结构件（vault/trial_spawner/waxed copper/rails/chest/spawner/cobweb 大量差异）。v2 = HEAD 仅门控 + @Redirect placedFeature.generate（:406）精准让位，结构族差异消失。注入点第三查沉淀 → workflow-patterns #91。
- ✅ **噪声基线（#67/#51 家族量化）**：Java-vs-Java 同代码双 run terrain=101,529 / veg=35,967 → run 级非确定只解释 ~2% 残差，信噪比 ≈48×。
- ✅ **共现相关分析 → b3 候选消解**：stone 族 3.98M 中 89.5% 与 ore 指示共现、75.5% 与 disk/lake 共现、独立仅 3.4% → 级联非独立层分歧（方法论 → #92）。
- ✅ **fan-out 双候选（fanout-260909-02）**：b1 RNG 接线——公式/类型/迭代序三层静态同构（population seed 有 Java 数值锚），输入域层保留 registry_order 回退残差（部分排除，Degraded）；b2 特征算法——4 族静态 PROVEN（Disk break 过早 / Geode isAir 缺 cave_air / Ore 邻 chunk 暴露判定 / Lake isSolid 含树叶）+ emerald_ore catch-all（0 RNG 消费）。
- ✅ **P-b1 序列探针 → 根因定位（decisive）**：chunk(0,0) 双侧 (k,p,fid) 对拍，step 1-8 共 40 条完全一致；step 9 特征集相同（13 fid 一致）但 p 全体错位（trees_water J=50/R=28 等）→ decorator seed=f(l,p,k) 域错位 → 植被整体偏移+级联。根因 = Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致（runtime 直接证据：probe-rustfeat-err-snapshot.log:13/65/78 biome_registry_order.json 缺失 → 字典序回退实录生效）。探针做法 → build-tooling #43。
- ⚠️ **次级实现缺陷 5 项（b2，静态 PROVEN，位置修复后逐一可验）**：Disk break（feature.rs:467-473）/ Geode isAir（feature.rs:2006）/ Ore isExposedToAir（feature.rs:358-369）/ Lake isSolid（tree.rs:969-981）/ emerald_ore catch-all——详见 fanout b2 §一（emerald「下游错位」论断已按 judge C2 勘误：无下游错位，仅直接损失）。
- ✅ **mask 翻转建议 = 不翻转（0b011 维持）**：step 9 p 错位未修复前翻转 = 全域植被换位 + 4 个已知 feature 族缺陷上线。翻转前置 = indexer p 域对齐 + b2 五缺陷修复 + 重对拍信噪比回噪声基线量级。
- 🔍 **open**：indexer p 域根因修复（生成 1.21.6 biome_registry_order.json——1.20.1 文件缺 pale_garden 等新 biome，覆盖面逐项核对 / feature list 构建序）；b2 五缺陷逐一修复+探针验证；b2 探针 P1-P6 清单；残差 9 项 §9.7 单列（dripstone_block=237 + pointed_dripstone=34 本 region 实测；iceberg/fossil/sculk/large_dripstone 未出现）；confirmed 留用户。
- 口径声明（§9.7）：双臂/基线 = Chunky region 存档口径（全量三分分类）；序列探针 = 行为化日志（#81，单 chunk 覆盖面声明：step 1-8 对全 region 有代表性，step 9 在 chunk(0,0) biome 邻域成立）；b1/b2 静态结论 = Degraded。
