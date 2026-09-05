# 260905-08 fan-out 分支 A：RNG 流内差三候选核实（.b1/.b3/.b4）

- 角色：core.worker（分支 A「流内差」，静态源码分析）
- 输入：260905-08-scout-tree-rng.md（scout 勘探产物）+ 一手源逐行核实
- 状态：**draft**（Degraded 分层：纯静态对拍，未运行任何探针/命令；沙箱无 shell）
- Java 参照：yarn 1.20.1+build.10-v2 sources（.tmp/scout-260905-08/mcsrc/，与 scout 同源）
- 结论速览：**三候选全部属实但只有 .b1/.b3 有树族解释力；.b4 对 oak/jungle_leaves 签名 ≈ 零贡献；三者均解释不了 oak_leaves +38,250**（该残差指向分支 B「集合差」）。

---

## 1. 逐候选核实

### 1.1 .b1 MegaJungle cos/sin：libm f32 vs MathHelper 查表 —— ✅ 属实

**Rust 侧（worldgen-core/src/tree.rs，mega_jungle_trunk）**：
- tree.rs:585-593 属实（引文行号偏 2 行内）：`:588 let f = random.next_float() as f32 * two_pi;`（two_pi = `std::f32::consts::PI * 2.0` = 6.2831855f32，与 Java `(float)(Math.PI * 2)` 位级相等——double 2π 截断 f32 与 f32 常量运算结果一致，已核）；`:592 j = (1.5f32 + f.cos() * l as f32) as i32;` `:593 k = (1.5f32 + f.sin() * l as f32) as i32;` —— **libm f32::cos/f32::sin**。
- 分支循环消费序 `height-2-nextInt(4)` / 每支 1×nextFloat / 5×getAndSetState（:586-601）与 Java 一致 ✔——差异**仅在三角函数实现**，不在消费结构。

**Java 侧（MegaJungleTrunkPlacer.java:37-49）**：
- `:43 j = (int)(1.5F + MathHelper.cos(f) * l);` `:44 k = (int)(1.5F + MathHelper.sin(f) * l);` —— **MathHelper = 65536 项 SINE_TABLE 查表**（MathHelper.java：`SIN_TABLE[(int)(value * 10430.378F + 16384.0F) & 65535]`）。

**接线现状**：carver.rs:17-36 已有 `pub fn math_sin/math_cos`（查表复刻，注释自认「carve 漂移逐位对齐关键」），**tree.rs 未接线**（grep 证实 tree.rs 无 math_sin 引用）。scout 引用全部属实。

**语义差真实性**：查表值 = sin(格点)（格距 2π/65536），对真值误差 ≤ ~9.6e-5；libm f32 误差 ≤ 1ulp（~6e-8 量级）。两者对同一 f 的差可达 ~1e-4；乘 l≤4 放大到 ~4e-4；`(int)` 截断在 frac 距整数边界 < 4e-4 时翻转 → **j/k 差 1**。翻转是确定性的（同 f 两侧结果可静态算出），但分布双向（f 落点在边界两侧均匀），单树方向不可预测。

### 1.2 .b3 decorator 集合迭代序：HashSet 桶序 vs Y 稳定序 —— ✅ 属实

**Rust 侧**：tree.rs:341（cocoa：`sorted.sort_by_key(|p| p[1])`，注释 `:339` 自认「同 Y 内序 = Java HashSet 桶序，遗留 idk R-1，此处用插入序近似」）、:363（TrunkVine）、:379（LeaveVine）——属实。

**Java 侧**：TreeFeature.java:122-125：`Sets.<BlockPos>newHashSet()` ×4（trunk/leaves/foliage/decorator）——HashSet 迭代序 = 桶序，与插入序/Y 序无对应。TreeDecorator.Generator（TreeFeature.java:153）直接持有 set2/set3 引用，各 decorator（CocoaBeans/TrunkVine/LeavesVine）遍历该集合 → **迭代序即 RNG 消费序**。

**波及范围修正（重要）**：Rust decorator 执行条件 = config.decorators 非空。查 placed/configured 链：**oak 平原树（trees_plains）无 decorator** → .b3 对 oak_leaves 残差**零贡献**。带 decorator 的树链 = jungle_tree（cocoa+vine）、mega_jungle_tree（trunk_vine+leave_vine）、swamp/oak_fancy（vine）等。**jungle 域限定 + vine/cocoa 专属**。

**机制细节**：LeaveVine placeVines 向下续藤最长 4 格（tree.rs:386-392 ↔ Java 同）——先迭代的 leaf 会占据后迭代 leaf 也能到达的格子（air 检查失败）→ 迭代序差不仅改 vine 位置，还改**vine 总数**（非中性重排）。vine 大头 rust 少 21,961 与「序差导致数量差」吻合，但方向性（为何系统性 rust 少）静态不可判——留实验。

### 1.3 .b4 placement modifier 抽法差 —— ✅ 属实，但对树族签名 ≈ 零贡献

**RarityFilter**：placement.rs:286-288 `random.next_int_bound(*chance) == 0` vs Java RarityFilterPlacementModifier.java:26 `random.nextFloat() < 1.0F / this.chance`——属实。同为 1 次消费但接受集不同（chance=100：Java 接受 float∈[0,0.01) ≈ 167,772/2^24 个值；Rust 只接受 nextInt(100)==0 精确 1/100——两接受集不相等的样本不同）。

**RandomOffset**：placement.rs:392-395 解析成 `RandomOffset(xz, Constant(0), y_spread)`；get_positions :319-321 消费 `ox.get + const0.get(0 消费，placement.rs:24-26 证实 Constant 不耗随机) + oz.get` = **2 次**；Java RandomOffsetPlacementModifier.java:41-45 消费 `spreadXz, spreadY, spreadXz` = **3 次**（xz 消费两次）——属实。

**波及面（grep 全部 placed_feature JSON，58 处命中）**：
- **rarity_filter**：bamboo_light、蘑菇 6 种、花 7 种、patch_*（cactus/melon/pumpkin/sugar_cane/tall_grass/berry 等 12 种）、ore_×4（花岗/闪长/安山上层、大钻石）、lake_lava×2、fossil×2、iceberg×2、desert_well、amethyst_geode、seagrass_simple、sea_pickle、end 系列、**trees_meadow（chance=100，唯一树链）**。
- **random_offset**：cave_vines、disk_grass、ice_patch、lush_caves×3、pointed_dripstone、spore_blossom、end_gateway_return、**rooted_azalea_tree（唯一树链，xz_spread=0/y_spread=-1）**。
- **核心树链全不用**：trees_jungle（count weighted_list→in_square→surface_water_depth→heightmap→biome）、mega_jungle_tree_checked（仅 block_predicate_filter）、trees_plains（+would_survive filter）——grep 证实三链均无 rarity_filter/random_offset。

**归因判定**：每 placed feature 在 setDecoratorSeed(l,p,k) 独立重 seed（scout Q1 已证，:402）→ 这些 modifier 差**不污染其它 feature 的流**。因此 .b4 只影响：
1. trees_meadow（meadow 生物群系，chance=100 稀疏树——若 dump 区域含 meadow，对 oak_leaves 有**极小**边际贡献，量级 ≤ 数十块，解释不了 +38,250）；
2. rooted_azalea_tree（azalea_leaves，不在残差签名族内）；
3. 全部植被/蘑菇/矿/湖类（不在树族残差内）。
**结论：.b4 修不修都不动 oak +38,250 / jungle -22,611 主签名**——它是「顺手对齐项」，不是主因候选。scout 给的中强度偏高（对树族课题而言应降为低）。

---

## 2. 签名预测表（修复后残差如何变化）

| 候选 | oak_leaves (+38,250) | jungle_leaves (−22,611 / +2,147) | vine (−21,961) | cocoa | 域限定 | 预测量级 |
|---|---|---|---|---|---|---|
| .b1 接线 math_cos/sin | **0**（oak 链不触发） | 双向收敛：残差净额部分收敛（j/k 翻转位置差 → TreeNode 移位 → 树冠位移），**方向混合**，对 −22,611 净额收敛幅度静态不可精确判 | 小（枝干端点连带 leave_vine 载点变化，jungle 域） | 0 | **jungle 域**（mega_jungle_trunk 独占） | 翻转率 ~1e-3/评估点 × 每树 2-3 支 × 5 点 → 少数百分比树受影响；受影响树每树 ±数十 leaves 块。**量级够到部分解释，能否独扛 22k 存疑** |
| .b3 对齐 HashSet 序 | **0**（trees_plains 无 decorator） | 0 直接（leaves 集合本身不迭代——只 decorator 迭代 trunk/leaves 集）| **主解释候选**：对齐后 vine 残差应显著收敛（数量+位置双向） | 同步收敛 | jungle/swamp 域 | 若 .b3 主导 vine 残差，对齐后 vine 残差应降一个台阶；leaves 族签名基本不动 |
| .b4 nextFloat + 3 消费 | ≈0（仅 meadow 边际） | ≈0 | ≈0 | ≈0 | 全域植被 | 树族签名不动；预期蘑菇/花/patch 族残差（另有台账）变化 |

**战略含义**：oak_leaves +38,250 **三者皆解释不了**——oak 树链既无 cos/sin（.b1）也无 decorator（.b3）也无 rarity/random_offset（.b4）。oak 残差几乎必然在分支 B（.b2 邻域 biome 集 / .b5 高度图快照）或更上游（生物群系判定本身）。分支 A 只对 jungle_leaves/vine 负责。

---

## 3. 精确修复方案（diff 级）

### .b1 —— tree.rs:592-593（Java 参照 MegaJungleTrunkPlacer.java:43-44）

```diff
--- worldgen-core/src/tree.rs
+++ worldgen-core/src/tree.rs
@@ -589,9 +589,9 @@
             let mut j = 0i32;
             let mut k = 0i32;
             for l in 0..5i32 {
-                j = (1.5f32 + f.cos() * l as f32) as i32;
-                k = (1.5f32 + f.sin() * l as f32) as i32;
+                // MegaJungleTrunkPlacer.java:43-44：MathHelper.cos/sin = 65536 项查表，非 libm
+                j = (1.5f32 + crate::carver::math_cos(f) * l as f32) as i32;
+                k = (1.5f32 + crate::carver::math_sin(f) * l as f32) as i32;
```
（math_cos/math_sin 已 pub，carver.rs:29-36；无其它调用点需要改。）

### .b4a —— placement.rs:286-288（Java 参照 RarityFilterPlacementModifier.java:26）

```diff
@@ -286,7 +286,7 @@
             PlacementModifier::RarityFilter(chance) => {
-                if *chance <= 0 || random.next_int_bound(*chance) == 0 { vec![[x, y, z]] } else { vec![] }
+                // RarityFilterPlacementModifier.java:26：nextFloat() < 1.0F / chance
+                if *chance <= 0 || random.next_float() < 1.0f32 / *chance as f32 { vec![[x, y, z]] } else { vec![] }
             }
```
注意 Java `1.0F / int chance` 是 float 除法；Rust `1.0f32 / *chance as f32` 等价。

### .b4b —— placement.rs:319-321（Java 参照 RandomOffsetPlacementModifier.java:41-45）

最小 diff：parse（:392-395）不动（oz 槽位即 y_spread），只改 get_positions 消费序——**xz 消费两次**：

```diff
@@ -319,8 +319,9 @@
             PlacementModifier::RandomOffset(ox, oy, oz) => {
-                vec![[x + ox.get(random), y + oy.get(random), z + oz.get(random)]]
+                // RandomOffsetPlacementModifier.java:41-45：
+                // i = x + spreadXz.get; j = y + spreadY.get; k = z + spreadXz.get —— xz 消费两次
+                let _ = oy; // parse 侧中槽为 Constant(0)（:395），Java y 槽实际用 parse 的 y_spread(oz)
+                vec![[x + ox.get(random), y + oz.get(random), z + ox.get(random)]]
             }
```
（若嫌槽位语义脏，可后续把 enum 改两元组 RandomOffset(xz, y)——行为等价，非本次必需。）

### .b3 —— 暂不出 diff（诚实声明）

精确复刻 Java HashSet 桶序需要复刻 BlockPos.hashCode + HashMap 桶布局 + 插入历史（HashSet 迭代序依赖扩容历史），成本高且只影响 vine/cocoa。建议先跑 §4.3 判别实验确认 .b3 对 vine 残差的解释力再决定是否投入。若确认，实现方案 = Rust 侧用 `Vec<(hash, seq)>` 模拟 Java HashMap：`IterateOrderSort::key = (BlockPoshashCode(pos) 桶索引, 插入序)`——但 Java HashSet<Long>（BlockPos.asLong boxed？实为 BlockPos 对象，hashCode = BlockPos.hashCode）桶序 = `(hash & 0x7FFFFFFF) % capacity` 升序桶 + 桶内链序——容量随扩容 16→32→…变化，需按实际插入数模拟 resize。复杂度中，收益域窄。

---

## 4. 最小判别实验设计（主会话执行）

通用：全部用同一确定性 dump 载体（#52：同 seed 8576294172403134396、同 2193 chunk 区域重生成），不做跨 run 对比（#51）。先跑 `WG_FEATURELOG` 确认 dump 区域 jungle 块确实执行了 mega_jungle 相关 feature（否则 .b1/.b3 实验无效）。

1. **判 .b1（最便宜，优先）**：应用 §3 .b1 diff → `cargo build --offline -p worldgen --release` → 重生成确定性 dump A/B。
   - 预测 hash 变化范围：**仅 jungle 域 chunk 变**（cos/sin 只在 mega_jungle_trunk 执行；oak/plains chunk 的任何字节不变——包括 RNG 流下游，因 j/k 翻转改变的是块位置与 can_replace 分支，流消费次数可能变 → 同树内后续消费偏移，但树间 per-feature 重 seed 不外溢）。oak_leaves 残差必须**分毫不动**，否则说明有未建模耦合，立即回查。
   - 量化预期：jungle_leaves 双向残差表（−22,611 / +2,147）部分收敛；vine 残差小动。
2. **判 .b3**：Java 探针工程一次性打印 TreeDecorator.Generator 对 set2/set3 的实际迭代序（单棵 jungle 树），与 Rust Y 稳定序对照（scout §6.3 同方案）。若两序对多数树一致 → .b3 排除 vine 主因地位；不一致 → 做 §3 .b3 完整实现后 A/B，预期 vine 残差（−21,961）显著收敛、leaves 族不动。
3. **判 .b4**：静态已可确证（不用实验定真伪），修完跑全量 dump 确认**树族签名三族（oak/jungle/vine）残差不变**——变了即有未建模耦合。 vegetation 族残差若另有台账则顺带核。
4. **oak +38,250 岔路指引**：分支 A 实验全部做完后 oak 残差预计不动——主会话应把 oak 残差归入分支 B（.b2/.b5）排序，不等分支 A 收敛。

## 5. 排除判据

| 候选 | 排除条件 |
|---|---|
| .b1 | WG_FEATURELOG 确认 mega_jungle feature 已执行，且接线后 jungle_leaves 双向残差表**逐格不变**（hash 不变）→ .b1 排除（说明 j/k 从未翻转或翻转无块级效应）。若 hash 变了但 jungle 残差净额收敛 <10% → .b1 真实但非主因，降级。 |
| .b3 | Java 单树 decorator 迭代序与 Rust Y 稳定序一致（多数树）→ 排除；或对齐实现后 vine 残差不变 → 排除。 |
| .b4 | 修复后树族三签名（oak/jungle_leaves/vine）残差不变 → 对本课题排除（静态已预判 ≈0；此实验是防「未建模耦合」的保险闸）。 |

## 6. 推荐执行序（供主会话）

1. WG_FEATURELOG 确认 jungle 域 feature 执行（否则换含 jungle 的采样窗）。
2. .b1 diff（3 行）→ A/B dump → 看 jungle_leaves 收敛率（同时验证 oak 不动 = 健全性检查）。
3. .b4a+.b4b diff（5 行）→ 全量 dump 回归（预期树族签名不动）。
4. .b3 判别（Java 迭代序探针）→ 决定是否投入 HashSet 序复刻。
5. oak +38,250 → 转分支 B（.b2/.b5），不阻塞于分支 A。

## 7. 边界声明

- Degraded：纯静态源码对拍，未运行任何命令/探针（沙箱无 shell）；所有量化预测为静态推算，status=draft。
- 未重验项：BlockPos.hashCode 具体公式（.b3 实现时需要，本次未读）、Java 侧 `(int)` 截断对负值的语义（1.5+cos*l 理论可 <0，两侧都是 Java/Rust `as i32` 截断语义一致，粗核无差异）。
- 依赖 scout Q1 的「per-feature 独立重 seed」结论——本分析沿用（其 chunkrandom.rs:158-161 ↔ ChunkRandom.java:75-78 静态一致已由 scout 一手核过，非转引假设）。
