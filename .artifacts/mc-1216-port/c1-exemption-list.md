# C1 豁免清单（mc-1216 移植已知差异 · 权威载体）

> **性质**：本文件是 1.21.6 移植课题 C1「已知差异/豁免清单」的**单一权威载体**（此前散落在各 verdict，260908-12 收口归一）。
> **纪律**：追加不覆盖（§15.4——条目若被推翻以取代记录表达，正文不改）；每条带机制/影响域/处置区/首次取证来源。
> **处置区分账**：
> - **A 区** = 真 bug/对齐缺口，须修复（有 commit/verdict 指针）。
> - **B 区** = 「维持旧行为」预期差异（版本新增内容未支持/超出接管域），**勿误读为 bug**；修复与否属独立立项决策。
> - **IDK** = 未归因，仅登记。

## 状态：confirmed（用户拍板 260908-12；judge SHOULD 审查 PASS-with-conditions 两条件已应用）

---

## 条目清单

### B 区（维持旧行为，预期）

| # | 条目 | 机制 | 影响域 | 首次取证 | 本块核验（260908-12） |
|---|------|------|--------|----------|----------------------|
| B1 | Shipwreck Y 重定位 | 结构放置 Y 语义版本差 | shipwreck 结构 | 架构计划-260908-09 C1 清单 | 继承项，状态以 260908-09/10 verdict 为准，本块未重验 |
| B1 | Shipwreck Y 重定位 | 结构放置 Y 语义版本差 | shipwreck 结构 | 架构计划-260908-09 C1 清单 | 继承项，状态以 260908-09/10 verdict 为准，本块未重验 |
| B2 | HugeMushroom / Disk / EndSpike / Geode 4 项 feature | 未支持 feature 类型 | 各自结构/装饰 | 架构计划-260908-09 C1 清单 | 继承项；1.20.1 侧同类未支持属「非回归」（对照臂同样报，p7-e2e-verdict-260908-10 §4） |
| B3 | SPAWN barrier 差异 | spawn 区 barrier 放置 | spawn 区 | 架构计划-260908-09 C1 清单 | 继承项；P6 探针挂账未做，本块不变 |
| B4 | `DimensionPadding`（jigsaw 结构放置 Y 约束） | 1.21.6 结构放置边界机制（DimensionPadding.java:9、StructurePoolBasedGenerator.java:113-132 的 fit 检查）；**Java 侧机制，Rust 不执行结构放置，双臂均走 vanilla Java 结构管线** → 预期零差异 | 结构放置 Y 边界 | 架构计划-260908-09 C1 清单 | ✅ 本块判定（静态）：不涉 Rust 接管域，无缺口；原 C1 清单核对项在此交代去向（judge MUST-C1） |
| B5 | **`minecraft:fallen_tree` configured feature（5 变体）未支持** | 1.21.6 新增 feature type `Feature.FALLEN_TREE`（Feature.java:29）；数据实存 5 个 configured feature：fallen_birch/jungle/oak/spruce/super_birch_tree.json；Rust 引擎（worldgen-core tree.rs/feature 层）与 Java 探针侧均无该 type 引用 | 倒木装饰（features 由 Java vanilla 在 Rust 地形上跑的场景不受影响；Rust 接管 feature 展开或对照臂 unsupported 报告时体现） | p7-e2e-verdict-260908-10 §4（候选记录） | ✅ 本块静态核验一致：数据 5 变体实存 + 两侧零支持 |
| B6 | **`minecraft:place_on_ground` tree decorator 未支持** | 1.21.6 新增 decorator type `TreeDecoratorType.PLACE_ON_GROUND`（TreeDecoratorType.java:16，PlaceOnGroundTreeDecorator）；数据实存 9 个 configured feature 引用（oak/birch/dark_oak/fancy_oak 及 bees 变体的 \*leaf_litter 族）；两侧均无引用 | 叶子层 ground 装饰（同 B5 分账逻辑） | p7-e2e-verdict-260908-10 §4（候选记录） | ✅ 本块静态核验一致：数据 9 处实存 + 两侧零支持 |

### 取代记录与勘误登记（260908-14）

> 依 §15.4：追加不覆盖，原条目正文不删不改；以下登记为唯一权威补充。

**B5/B6 状态更新注记（260908-15，features 接管块）**：B5 (fallen_tree 5 变体) 与 B6 (place_on_ground decorator) 已在 Rust 侧实装支持（b5b6-verdict-260908-15，candidate 待 judge→260909-02 块纳入收口），B5/B6 的「两侧零支持」状态已被取代——两行正文按 §15.4 不改，以本注记为准。

**B 区追加：features 接管缓装 5 项（260909-02 登记，来源 = 批次 C 交付文档 §〇.2，`.investigations/mc-1216-features-takeover/batchC-worker-delivery.md`）**：

| # | 条目 | 机制 | 影响域 | 缓装理由 / 处置建议 |
|---|------|------|--------|---------------------|
| B7 | `dripstone_cluster` 未实装 | 需三基建（高度图 probe 面/双向 grow 逻辑/风化层级） | 滴水石簇装饰（1.21.6 drippy 区域） | 基建批立项；当前 Rust 接管臂 unknown 哨兵覆盖 |
| B8 | `large_dripstone` 未实装 | 同 B7 基建依赖 | 大型滴水石 | 并入 B7 基建批 |
| B9 | `iceberg` 未实装 | 大工作量（体积雕刻 + 冰变体状态机） | 冻结海洋冰山 | 独立小批立项 |
| B10 | `sculk_patch` 未实装 | 需传播子系统（sculk spread） | 深暗之域表面 | 行为层立项（传播机制跨 feature 边界） |
| B11 | `fossil` 未实装 | 需 NBT 结构模板加载层 | 骨矿（结构模板类 feature） | 结构模板层立项（与 structures 数据面共用决策） |
| B12 | 嵌套暴露 4 项（coral_claw/mushroom/tree + pointed_dripstone） | 批次 C 起从父 feature 内嵌展开，行为已解锁——差集出现为**行为解锁非回归** | 珊瑚/滴水石尖锥 | 非豁免项，登记防误读；对拍差集出现时按 §9.7 单列 |

**features 接管对拍现状注记（260909-02，Phase 2.5 首轮）**：Chunky 双臂对拍（region 0,0 r=16）信号/噪声 = 4.86M/0.10M terrain ≈48×，主残差为特征层真实分歧（候选分解见 `.investigations/mc-1216-features-takeover/fanout-260909-02/`）；默认 mask 翻转建议**未提出**（confirmed 前置未满足）。

**取代记录（B3）**：
- **supersedes** → `.artifacts/mc-1216-port/p5-verdict-260908-13.md`（P5 判定 confirmed 260908-13，B3 ⊆ P5 覆盖面，无独立机制成分）；
- **superseded-by** → `.artifacts/mc-1216-port-260908-14/p6-b3-closeout-260908-14.md`（本取代记录）；
- 一行推翻理由（judge 条件 1 原文措辞）：「B3 机制实为 1.21.6 ChunkStatus.SPAWN 邻居依赖 0→ring-1@BIOMES 的调度层依赖 barrier，非『spawn 区 barrier 放置』——原行机制/影响域列系转抄漂移；运行时探针无观测对象，按取代记录关账」。
- 重开判据：沿 P5——出现「邻 chunk 状态不足」类失败且定位到依赖推进 → 重开（取代记录不豁免）。

**勘误登记（judge 条件 2，三项）**：
1. **B3 转抄漂移**：本清单 B3 行机制列「spawn 区 barrier 放置」→ 实为「邻居推进依赖 barrier（SPAWN.dependsOn(BIOMES,1)）」；影响域「spawn 区」→「任意 chunk 推进到 SPAWN 的调度条件」（全管线第 11 阶段）。首次取证：w2-chunk-pipeline.md:30/51/64（260908-08）+ port-list-260908-08.md:15。
2. **B1 整行重复**：本清单「条目清单」B 区第 20-21 行 B1「Shipwreck Y 重定位」整行逐字重复（260908-12 转录引入），有效条目以其中一条为准。
3. **p7-e2e-verdict 路径出入**：该文件实际位于 `.artifacts/mc-1216-port-260908-10/`，本清单此前引用的目录名（`mc-1216-port`）有出入；文件名与 §节号无误，引用以实际路径为准。

> 原 B3 行正文不改；勘误删行与否由用户另行拍板，当前仅登记。

### A 区（已修复真缺口，登记备查）

| # | 条目 | 修复 | 来源 |
|---|------|------|------|
| A1 | aquifer water-over-lava 检查点用全价噪声链（aquifer.rs:493-499） | F1 修复（默认液面 sampler 语义），confirmed 260908-11 | p7-e2e-verdict-260908-10 §5.1 → verdict-f1f2-chunky-260908-11 |
| A2 | AquiferDumpProbeMixin `wgNoiseBasedFluidLevel` 漏 `+ base`（探针 bug，非产品） | F2 修复（gitignored runtime/，f2-note.md 为权威描述），confirmed 260908-11 | p7-e2e-verdict-260908-10 §5.2 |
| A3 | `StructureTerrainAdaptation.ENCAPSULATE`（trial_chambers 等）：1.21.6 新增枚举常量，Rust beardifier 原映射 `_ => None` 静默关断（1.21.6 新常量落 catch-all 家族，workflow #87） | 260908-10 P2b 修复（beardifier.rs ordinal 4 + q 分支 + 权重 f64 版），judge 通过 | workflow #87 / p7-e2e-verdict-260908-10 §1-3（首次取证：发现 #87，knowledge/discovered/workflow-patterns.md） |

### IDK（未归因）

| # | 条目 | 状态 |
|---|------|------|
| I1 | P7 修复后 41 格残差（无簇零散，0.00065%）未逐点归因（feature 漂移 vs carver/aquifer 边界微差） | idk 声明保留（p7-e2e-verdict-260908-10 §4）；1.21.6 Chunky 区域级基线已建（625/3698，三家族同构） |

---

## 覆盖面 / 口径声明（§9.7）

- B5/B6 本块核验为**静态核对**（数据文件实存 + 两侧代码零引用 grep），非运行时行为验证——「未支持导致的行为差异量级」未量化（Chunky 1.21.6 基线 625/3698 中 veg=6,473 块差含本两项贡献的可能性未分解，属 I1 同族下钻项）。
- B1/B2/B3 为继承项，本块未重验，状态以来源 verdict 为准；B4（DimensionPadding）为本块静态判定（不涉 Rust 接管域）。
- 数据目录 versions/1.21.6/data 为 gitignored → 证据以本清单 + 来源 verdict 文字为准。
- **260908-14 静态关账口径追加**：B3 本轮按 §15.4 取代记录关账（`.artifacts/mc-1216-port-260908-14/p6-b3-closeout-260908-14.md`），验证分层 = Degraded（静态双源核对）+ §9.7 覆盖面声明——**未做运行时行为化探针**，理由 = 观测对象结构性不存在（双臂同走 vanilla Java 调度层），非「预期零差=验证通过」；见末尾「取代记录与勘误登记（260908-14）」节。
