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
