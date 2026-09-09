# fanout .b2 候选：部分特征族算法级实现分歧（b2-feature-algo）

> **judge 勘误（260909-02，C2，§15.4 追加不改上文）**：本文档中「emerald_ore 0 RNG 消费 ⇒ 下游特征流错位 / 让渡 b1」论述不成立——b1 已证每 feature `setDecoratorSeed` 完全重置 RNG（ChunkRandom.java:75-78），种子间零耦合，emerald 缺失仅造成 emerald 自身直接损失，无下游错位。该论断以本勘误为准，不进主题篇。

- 状态：**draft**（静态审查，Degraded 分层）
- 会话：mc-1216-features-takeover fanout-260909-02（core-worker b2）
- 输入口径（§9.7 验证可比性声明）：
  - 载体：`.tmp/mc1216-closeout-260909-02/diff-signal-v2.txt`（seed -8248318472910187742，region 0,0 r=16，3410 chunks，Java-vs-Java 噪声 terrain=101,529 / veg=35,967；Java-vs-Rust 信号 terrain=4,857,306 / veg=346,025，信噪比≈48×）
  - 覆盖面：仅 worldgen features 阶段静态源码对拍（Rust `worldgen-core/src` vs Java 一手源 `.tmp/scout-260905-08/mcsrc/.../gen/feature/`），**零运行时探针**——全部结论 Degraded。
  - 与既有口径可比性：与 M16 探针口径（96.06%）、存档写入口径（82.16%）**均不可比**；本候选只做族级「有/无静态分歧」判定，不输出对齐百分比。
- 辖区声明：本候选只管**族内算法**（放置判据/RNG 用法/边界条件）。全局 RNG 消费序错位归 b1；跨族巨量地形差归属建议见 §五（不解释）。

---

## 一、静态可证成立的算法级分歧（4 族 + 1 处实施缺失）

### 1.1 Disk 族（clay/gravel/dirt/sand）——列内 break 语义错（PROVEN）

- **现象对应**：clay 206,276 / gravel 158,880 / dirt 120,832 / sand 31,515 delta。
- **证据**：
  - Rust `worldgen-core/src/feature.rs:461-476`（DiskFeature::generate）：每列自顶向下扫描，**首个 target 命中 `set_block` 后 `break`**（:471），一列至多放 1 块。
  - Java `DiskFeature.java:44-52`（placeBlock）：`for (int i = topY; i > bottomY; i--)` 内**无 break**——target 匹配的**每一层都替换**（disk 半高即厚度），且逐块消费 `stateProvider.get(random, pos)`（:47）。
- **根因（机制）**：移植时把「找该列放置面」当成了「找首个匹配」，把 Java 的逐层替换（thickness = half_height 层语义）折叠成了单层 disk。disc 族形状整体变薄 → 大体量 clay/dirt/gravel 差。
- **附带偏差**：Java 每放置 1 块消费 1 次 provider RNG（SimpleBlockStateProvider 不消费，vanilla disk 全用 simple，实际 RNG 序无差——已核对 vanilla disk 配置，风险留档）。
- **五段式**：
  - 现象：信号 diff clay/gravel/dirt/sand 三位数量级差，且坐标成片（如 clay 例 (-1,0,766)/(-1,0,767) 连片）。
  - 根因：feature.rs:467-473 的 `break` 相对 Java DiskFeature.java:44-52 的全层替换是算法级删减。
  - 定位：逐族源码对拍（本候选 §方法，Degraded 静态审查）。
  - 修复（建议）：去掉 break，列内所有 target 层全替换（保持 `placed` 语义 = 任一放置即 true）。
  - 教训：对拍「循环内放置」语义时必须核对 Java 是否 break/continue——形状类 feature 的「首个命中即停」假设未经源码核对不得引入。
- **探针 P1**（廉价可判别）：取 clay 例坐标区块（chunk 起点附近 (-1,0,766)→chunk (-1,0)），对比两侧输出该列 y∈[origin-1, origin+1] 的 clay 层数：Java ≥2 层/列、Rust =1 层/列 ⇒ 判据成立。无需改代码，直接 diff 两份现成 region 导出。

### 1.2 Geode 族——isAir 判定不含 cave_air（PROVEN）+ 噪声流构造待 probe（UNPROVEN）

- **现象对应**：amethyst_block 22,123 / calcite 27,456 / smooth_basalt 33,164 / budding_amethyst + amethyst_cluster/buds。
- **证据 A（静态成立）**：Java `GeodeFeature.java:62-63` 用 `blockState.isAir()`——含 air/cave_air/void_air 全族；Rust `feature.rs:2006` 只认 `st == blocks::AIR`。Rust 世界里 lake 等 feature 会写入 cave_air（feature.rs:869），洞穴邻近的 geode 在 Rust 侧会把 cave_air 计为 invalid，`invalid > 1` 即整 geode abort（:2008），Java 则正常继续。方向与 delta 一致（Rust 缺 geode）。
- **证据 B（待运行时判别）**：噪声采样器种子构造——Java `GeodeFeature.java:42` = `ChunkRandom(new CheckedRandom(worldSeed))`；Rust `feature.rs:1986-1988` = `RsRandom::Legacy(LegacyRandom::new(world_seed))`，注释声称「同 LCG 等价」但 CheckedRandom 与 LegacyRandom 的 setSeed 预处理（乘数扰动/warm-up）是否逐位一致**本文未验证**。若不等价，**所有** geode 的分层噪声全错（不只是洞穴内）——可解释 smooth_basalt 33k > amethyst 22k 的量级比例。
- **探针 P2**（廉价）：对同一 geode 例坐标（amethyst 例 chunk (-1,0)，y≈159 区域）单 chunk 对拍：若 Rust 整 geode 缺失且该处无洞穴 ⇒ 指向证据 B（噪声流）；若仅洞穴相邻 geode 缺失 ⇒ 指向证据 A。辅助插桩：统计 geode abort 次数（Rust 侧一行 eprintln 计数即可）。

### 1.3 Ore 族——邻 chunk 暴露判定边界条件（PROVEN，幅度中等）

- **现象对应**：coal_ore 55,539 / copper 17,752 / iron 11,426 / deepslate ore 族。
- **证据**：
  - Java `OreFeature.java:110,143,174`：`ChunkSectionCache` 读方块（含**邻 chunk section**，feature 阶段邻 chunk 已生成），`shouldPlace` 的 `isExposedToAir`（:174）用真实邻块判 air。
  - Rust `feature.rs:358-369`（is_exposed_to_air）：邻块经 `local_idx` 出 chunk 返回 -1（:365），`-1 != 0` ⇒ **邻域一律视为非 air，永不 discard**；region_col_at/block_at_ext 有钩子（:176-188）但该函数未走 `block_at`，走 `col.at` 直读。
- **根因**：边界一环内的矿块，Java 可能因邻 chunk 空腔 discard，Rust 不 discard ⇒ Rust 边界矿偏多。
- **主体结论**：OreFeature 算法主体（generate/generateVeinPart/RNG 消费序/sin vs 查表 sin/ceil 边界）已逐行对齐（feature.rs:229-347 vs OreFeature.java:22-165，含 `l = ((MathHelper.sin +1)*h+1)/2` 查表调用 :273 vs :83）——ore 族差异**主源不是全局流**，是边界读。ScatteredOreFeature 逐行一致（feature.rs:386-411 vs ScatteredOreFeature.java:24-51，含 `Math.round((f-f)*spread)` :410 vs :50）。
- **探针 P3**（廉价，零代码）：对 ore diff 例坐标统计「到最近 chunk 边（x%16∈{0,15} 或 z%16∈{0,15}）距离」分布：Java-缺失矿显著集中在边界 1-2 格环带 ⇒ 判据成立；均匀分布 ⇒ 转向 b1。

### 1.4 Lake 族——isSolid 谓词语义不等价（PROVEN 谓词，幅度待判）

- **现象对应**：water 114,621（+lava/岩浆块族）。
- **证据**：Java `LakeFeature.java:80,122` 用 `BlockState.isSolid()` = **Material#isSolid（blocksMovement）**——树叶、含树叶方块按 Material 均 solid；Rust 复用 `tree::is_solid_id`（tree.rs:969-981），其 APPROX_NON_SOLID 排除表**把全部树叶类当非 solid**。校验 pass（feature.rs:843-862）「u<4 邻接位非实心且≠fluid → abort」在树冠覆盖的湖上判定结果两侧不同 ⇒ Rust 该放不放/Java 该 abort 不 abort 分歧。
- **附带**：canSetIce 用 chunk 级温度近似 per-column biome（feature.rs:803,904 声明）；block_at 不可读保守 abort（:844, idk-3）。均为已声明近似，量级小于 isSolid 项。
- **探针 P4**（廉价）：插桩统计 lake 校验 pass abort 计数（Java 侧用同 seed 单测或对 lake 例坐标区块直接对拍水面完整性）；树叶覆盖湖（森林/沼泽 biome 例坐标）优先。

### 1.5 实施缺失：minecraft:emerald_ore 走 catch-all（直接损失 + b1 让渡）

- **证据**：feature_loader.rs 的 type 分发（:118-235 及 generate 侧 :561-745）**无 `minecraft:emerald_ore` 分支** → parse 与 generate 双双落 catch-all（:235-237 / :748-751），generate 返回 false 且 **0 RNG 消费**；Java `EmeraldOreFeature`（next 循环放置+随机 y）正常消费。信号 diff 有 emerald_ore 例（(0,0,887) 等）为直接损失证据。
- **让渡（发现 #73，归 b1）**：catch-all 0 消费 ⇒ 该 feature 之后**同 chunk 全部后续 feature 的 RNG 流错位**——这不是 b2 算法差异，但会让 b1 的全局错位量化被污染。建议主会话先补 emerald_ore 或确认它已在「缓装单列」内。核实手段：`WG_FEATURE_UNKNOWN_LOG=1` 重跑看 unknown 集合是否含 emerald_ore（预期残差清单在代码注释外，本文无法核对——待主会话执行）。
- **探针 P5**：跑 `WG_FEATURE_UNKNOWN_LOG=1` 一次，对照 expected 残差清单（known9/缓装 9 项之外是否多出 emerald_ore）。

## 二、判定为「可被全局错位/其他候选解释、b2 不认领」的族

| delta 家族 | 判定 | 归属建议 |
|---|---|---|
| stone 1,283,593 / deepslate 900,678 / tuff 485,832 / andesite 447,730 / diorite 433,581 / granite 426,898（合计≈3.98M，占 terrain 信号 82%） | **超出 feature 阶段各族能量级**：ore 全族≤15 万、geode 数万、lake 数万——即使本候选 §一 全部成立也凑不出 4M。基岩面块身份差指向 surface-rule noise patch / ore_vein（NOISE 阶段）或 b1 全局流 | 建议**另立候选 b3（surface-rule/ore_vein noise 族）**；b1 只吃「流错位」部分 |
| spruce_leaves 70,246 / birch_leaves / moss_block 50,273 / short_grass 26,197 / cave_vines_plant 14,290 | 树/patch 主体由 decorator 链+全局流主导；random_patch tries 循环 Rust 已逐行对齐（feature_loader.rs:599-627 vs RandomPatchFeature.java:22-32，含 nextInt(j)-nextInt(j) 差分序）；cave_vine 经 block_column 走（placement.rs:22 声明）。未见族内算法级静态分歧 | b1 全局流；若 b1 收敛后残差仍在再回 b2 |
| kelp 73,158+14,079 / seagrass 8,840+16,121 | 算法主体对齐（feature.rs:966-1047 vs KelpFeature.java/SeagrassFeature.java，含 l==k AGE 消费序）；但 **offset ±7/±8 跨 chunk 时 Rust 高度查询走邻域下扫近似+不可读即跳过**（feature.rs:928-950），Java 可读邻 chunk 且直接写（无 isValidForSetBlock 门）→ 边界带差异静态可证但量级小 | 边界带部分算 b2 边界条件子项（P6：veg diff 到 chunk 边距离分布）；主体归 b1 |
| chest/spawner/cobweb/mossy_cobblestone/rail | monster_room/forest_rock 已实装（feature.rs:697-783/1688-1730），未见静态分歧 | 暂归 b1；forest_rock 若残留再单查 |

## 三、结论（b2 候选主句）

**成立**：4 个族存在静态可证的算法级分歧——① Disk 列内 break（最确定、量级最大）② Geode isAir 不含 cave_air（+噪声流构造待 P2 判别）③ Ore 邻 chunk 暴露判定永不 discard ④ Lake isSolid 谓词把树叶当非实心；另 1 处实施缺失（emerald_ore catch-all，含 b1 让渡 #73）。这些差异与全局 RNG 错位**独立**（各自有确定方向与局部位置特征），合计可解释信号中 ore/disk/geode/lake 家族的大部；**不能**解释 4M 级基岩面块差（见 §二 b3 建议）。

## 四、探针清单（主会话执行，全部 ≤ 一轮）

| # | 探针 | 判据 | 成本 |
|---|---|---|---|
| P1 | clay 例坐标列厚对拍（现成两份导出直接 diff） | Java≥2 层/列 vs Rust=1 层 ⇒ §1.1 | 极低 |
| P2 | 单 geode chunk 对拍 + Rust abort 计数插桩 | 区分 §1.2A（洞穴）vs §1.2B（噪声流） | 低 |
| P3 | ore diff 距 chunk 边分布统计（离线脚本） | 边界 1-2 格环带集中 ⇒ §1.3 | 极低 |
| P4 | lake abort 计数/树叶湖对拍 | 森林湖水面完整性差 ⇒ §1.4 | 中 |
| P5 | `WG_FEATURE_UNKNOWN_LOG=1` 重跑 | unknown 集合 vs 预期残差清单（emerald_ore?） | 低 |
| P6 | veg diff 距 chunk 边分布 | 边界 ±8 带集中 ⇒ kelp/seagrass 边界子项 | 极低 |

## 五、辖区外证据（只标归属，不解释）

- stone/deepslate/tuff/andesite/diorite/granite 4.0M 级 → **建议 b3**（surface-rule noise patch / ore_vein NOISE 阶段族），与 b1 全局 RNG、b2 族内算法均正交。
- emerald_ore catch-all 的下游 RNG 流错位 → b1 让渡清单（发现 #73），修复后 b1 的错位量化需重做一轮。
