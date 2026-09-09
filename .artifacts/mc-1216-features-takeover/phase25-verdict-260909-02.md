# features 接管 Phase 2.5 对拍 verdict（260909-02 · draft，judge PASS-with-conditions 四条件已应用，confirmed 未授）

> 日期锚：Get-Date 2026-09-09（13:3x）；git 锚 bf37814（Rust 侧无新提交；Java mixin 探针改动在 runtime/ 不入库）。
> 载体：Chunky 双臂 + 噪声基线 + 序列探针（seed -8248318472910187742，region 0,0 r=16，1089 chunks/臂，各 run mask/seed 三查全过）。

## 一、结论

1. **batchD-E1（P1，已修复）**：v1 mixin HEAD cancel 连坐 generateFeatures 内结构方块放置段（ChunkGenerator.java:360-379，结构 `start.place` 与 feature 迭代同方法）→ Rust 臂丢全部结构件。v2 = HEAD 仅门控判定 + `@Redirect placedFeature.generate`（:406）精准让位；vault/trial_spawner 族差异消失。选型盲区教训：注入点检查需加第三查「cancel 影响域 = 方法体全部副作用清单」。
2. **噪声基线（#67/#51）**：Java-vs-Java 同代码双 run terrain=101,529 / veg=35,967（764/3410 chunk-slots）→ run 级非确定只解释 ~2% 残差。
3. **信号分解**：Java-vs-Rust terrain=4.86M / veg=346k（1389/3410）。相关分析：stone 族 3.98M 中 89.5% 与 ore 指示 section 共现、75.5% 与 disk/lake 共现，独立 section 仅 3.4% → **stone 族是特征位置偏移的级联，非 NOISE/surface 层分歧**（b3 候选消解）。因果边界声明（judge C3）：共现 ≠ 因果，本判读由「共现 + 机制可解释（ore/disk 置换 stone）」联合支撑；独立 section 的 3.4%（135,826 块）未逐块归因，留修复后复验。
4. **根因定位（decisive，P-b1 序列探针）**：chunk(0,0) 双侧 (k,p,fid) 对拍——step 1-8 共 40 条完全一致（证据行区间：probe-javafeat-log-snapshot.log `[JFEATURE]` 50 行 / probe-rustfeat-err-snapshot.log `[FEATURE] chunk(0,0)` 50 行，顺序对齐）；**step 9 (VEGETAL_DECORATION) 特征集合相同（13 fid 一致）但 p 索引不同**（trees_water J=50/R=28、flower_default J=57/R=53、seagrass_normal J=101/R=94 等）→ decorator seed=f(l,p,k) 全体错位 → 植被整体偏移 + 级联。根因 = **Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致**——输入差已由 runtime 直接证据证实：probe-rustfeat-err-snapshot.log:13/65/78 实录 `biome_registry_order file missing (...1.21.6\data\worldgen/biome_registry_order.json), falling back to lexicographic order`（字典序回退在本探针 run 实际生效）。**修复前置 = 生成 1.21.6 版 biome_registry_order.json（现有 1.20.1 文件缺 pale_garden 等 1.21.6 新 biome，覆盖面须逐项核对）**。
5. **次级实现缺陷（b2，静态 PROVEN，位置修复后逐一可验）**：① Disk break 过早（feature.rs:467-473，Java 全层替换）② Geode isAir 缺 cave_air/void_air（feature.rs:2006）③ Ore isExposedToAir 邻 chunk -1 视为非 air（feature.rs:358-369，Java ChunkSectionCache）④ Lake isSolid 排除树叶（tree.rs:969-981）⑤ emerald_ore 走 catch-all（直接损失，emerald 全缺；judge C2 勘误：因每 feature seed 完全重置（ChunkRandom.java:75-78），emerald 0 RNG 消费**不产生下游流错位**——b2 文档「下游错位/让渡 b1」论述由本勘误取代，不进主题篇）。
6. **已知残差 9 项**（§9.7 单列）：本轮 diff 实测 dripstone_block=237 + pointed_dripstone=34；iceberg/fossil/sculk/large_dripstone 本 region 未出现。

## 二、mask 翻转建议

**不建议翻转（0b011 维持）**——step 9 植被 p 错位未修复前翻转 = 全域植被换位 + 4 个已知 feature 族缺陷上线。翻转前置 = indexer p 域对齐 + b2 五缺陷修复 + 重对拍（信噪比回到噪声基线量级）。

## 三、验证分层与口径（§9.7）

- 双臂/基线对拍 = Chunky region 存档口径（#26 载体，剔除植被口径未启用——本块用全量三分分类）；
- 序列探针 = 行为化日志（#81），单 chunk(0,0) 覆盖面声明：step 1-8 结论对全 region 有代表性（同 seed 同输入），step 9 p 错位在 chunk(0,0) biome 邻域成立，其他 biome 邻域的错位幅度未逐 chunk 验证；
- b1/b2 静态结论 = Degraded（源码对读），探针证据 = behavior 级。

## 四、产物索引

- 对拍脚本/驱动：.tmp/mc1216-closeout-260909-02/（chunky_arm、diff_*、correlate；.tmp 不入库）
- 相关分析输出（judge C4）：.tmp/mc1216-closeout-260909-02/correlate-output.txt（b3「surface/NOISE 层分歧」候选由主会话该相关分析消解——共现 89.5% + 独立 section 3.4%，无独立 .b3 产物，消解路径记录于此）
- diff 输出：diff-signal-v2.txt / diff-noise-baseline.txt；探针快照：probe-{javafeat-log,rustfeat-err}-snapshot.log
- fan-out：.investigations/mc-1216-features-takeover/fanout-260909-02/{b1-rng-wiring,b2-feature-algo}.md
- batchD-E1 记录：batchD-record-260909-01.md v2 节；台账：.artifacts/mc-1216-port/c1-exemption-list.md B5/B6 注记 + B7-B12
- 探针代码：runtime/1.21.6/java ChunkGeneratorFeaturesMixin（v2 接管 + WG_FEATURESEQ 序列探针；不入库）

## 五、judge 审查记录（260909-02）

- core-judge（subagent）：**PASS-with-conditions**，三源核对通过（数字逐位吻合）；四条件已应用：
  - C1 ✅ §一.4 补引 runtime 直接证据（probe-rustfeat-err-snapshot.log:13/65/78 字典序回退实录）→「indexer 输入序」从静态推断升格数据层证实；修复前置明确 = 生成 1.21.6 biome_registry_order.json；
  - C2 ✅ b2「emerald 下游错位」论断按 §15.4 勘误（b2 文档头 + 本 verdict §一.5）；
  - C3 ✅ §一.3 补因果边界声明（共现+机制联合支撑；3.4% 独立残差留复验）；
  - C4 ✅ §四补 correlate-output.txt 位置 + b3 消解路径记录。
- 附注（日志行区间）：step 1-8「40 条」对应 probe 快照 `[JFEATURE]`/`[FEATURE] chunk(0,0)` 各 50 行中的 k∈{1,2,3,6,7,8} 区段（快照 grep 截断，judge 抽验通过）。
