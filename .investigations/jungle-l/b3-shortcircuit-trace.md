# .b3 — jungle_l 残差 −12055：J3（blob 四角 nextInt(2) 短路族）候选定界

- 候选：J3 / 角色：core.worker（subagent 隔离）/ 状态：**draft**
- 日期锚定：继承 260905-13 工作块（scout 产物同日）
- 输入：260905-13-scout-jungle-chain.md（A8/J3 行）+ Java 一手源（.tmp\scout-260905-08\mcsrc）+ worldgen-core\src\tree.rs + 台账/knowledge
- 验证分层：**Degraded（静态对读）**——结论全部为静态级，trace 级证据见 b3-experiment.md 待执行

## 一、任务1：Rust blob 四角短路 vs Java BlobFoliagePlacer.generate 逐行对读

### 1.1 Java 侧语义拆解（一手源）

`BlobFoliagePlacer.isInvalidForLeaves`（:55-57）：
```java
return dx == radius && dz == radius && (random.nextInt(2) == 0 || y == 0);
```
- **求值序**：两个 `&&` 在 `nextInt` 之前 → **仅真角（|dx|==r 且 |dz|==r）消费 1 次**；`||` 的右支 `y==0` 在 nextInt 之后 → y==0 也先消费再判。非角 0 消费。
- **dx/dz 已是绝对值归一化**：调用方 `FoliagePlacer.isPositionInvalid`（:84-96）非 giant 传 `Math.abs(dx)/Math.abs(dz)`，giant 传 min(|dx|,|dx-1|)。blob 小树 jungle_tree 走非 giant 分支。
- **y 语义 = 层内相对 y，非绝对**：`y` 参数 = `BlobFoliagePlacer.generate` 层循环变量 `i`（:43-46），从 `offset` 降到 `offset-foliageHeight`（jungle_tree: 0,-1,-2,-3），**可为负**；绝对 y 到 `placeFoliageBlock` 才经 `mutable.set(centerPos, j, y, k)`（:110）加上。故 `y==0` 指**顶层（i=offset 层）角必 invalid**，与世界高度无关。
- **radius 来源**：层循环里的 `radius` = 每层算术值 `j = max(radius + nodeRadius - 1 - i/2, 0)`（:44），**零随机消费**；IntProvider 的 `getRandomRadius = this.radius.get(random)`（FoliagePlacer.java:68-70）只在 `TreeFeature.generate`（:69）**每 node 一次、循环外**消费——jungle_tree 配置 radius=ConstantInt(2)（configured_feature/jungle_tree.json:27）→ 该消费恒 0。
- **负除法**：`i/2` Java 向零截断（-1/2=0, -3/2=-1）→ jungle_tree 层 j 序 = [max(2-1-0)=1, 1, 2, 2]（i=0,-1,-2,-3）→ **4 层全 j>0，每树角消费 = 4 层 × 4 角 = 16 次**。
  ⚠️ 顺带勘误：s1-semantics-260905-05.md:386 写 j 序=[1,1,0,0]→8 次，其算术漏了 i/2=-1 分支（2-1-(-1)=2≠0），**16 才对**；两侧实现同用 java_div 故对拍不受影响，但该台账数字需修正。

### 1.2 Rust 侧对应（tree.rs）

- `is_position_invalid` Blob 分支（tree.rs:247-249）：`if ax == r && az == r { random.next_int_bound(2) == 0 || y == 0 } else { false }` —— **短路点/求值序一致**（&&在 RNG 前，|| 在 RNG 后）。
- 归一化（:240-244）：非 giant `ax,az = |dx|,|dz|`，同 Java。
- y 传入（:184-186）：`ii = offset - i`，`j = max(radius + tree_node_radius - 1 - java_div(ii,2), 0)`，`java_div`（:269-271）向零截断同 Java——y 相对语义与负除法一致。
- 层序（:183-184）：`i in 0..=h, ii = offset - i` → 序列 offset→offset-h，同 Java 降序。
- radius 消费点：`get_radius` 在每 node 主链（getHeight→getRandomHeight→getRadius→trunk→逐 node offset，tree.rs:497-547）与 Java TreeFeature:67-81 同位，260905-08 scout §Q3 已核（260905-08-scout-tree-rng.md:66）。
- 角扫描序：generate_square dx 外层 dz 内层（:225-226），同 Java :107-108。

### 1.3 裁决

**「短路点一致」成立（静态，四个检查点全过）**：① 求值序（&&→nextInt→||）② y 相对语义（y==0=顶层）③ radius 消费点（每 node 循环外，ConstantInt 0 消费）④ 归一化/层序/负除法。**不升级为确定性候选**。

且注意：blob 角消费**完全不含世界状态依赖**（判角在 placeFoliageBlock/can_replace 之前，纯 j 序决定）→ 若静态一致成立，J3 在 jungle_tree 域的期望漂移 = **0**。残余风险仅剩「静态对读漏看」一级，故 J3 应**降级为低先验候选**，用廉价 trace 收口而非继续静态深钻。

## 二、任务2：历史台账核对

- **标签出处**：`NEXT_SESSION.md:29` + `13-feature-parity.md:114` + `10-timewise-archive.md:2976`——「树3+ 状态依赖短路差」首见于 **260905-10 beehive judge 残余清单**（260905-10-judge-beehive.md:108），承接 260905-10 interim P2 逐树对拍（树1 重合、树2 起错位）后的未闭合残余。
- **当时证据域 = oak/birch**（trees_birch_and_oak，p=20，chunk(37,-16)，beehive 级联课题）；oak/birch foliage 也是 blob（BlobFoliagePlacer 同类）→ 该域**在族内**。
- **「状态依赖」原始所指** = can_replace 世界状态依赖（bB-3，260905-10-oak-p2-bB.md:33「6vs7 最可能形态」），**不是** blob 角 nextInt；scout 260905-08 §Q3 与 260905-13 A8 把该标签重述为「四角短路族」——两轮静态对读均 ✅ 未找到短路差实证。
- **jungle（Blob h3 小树）当时未覆盖**：jungle_tree 是同族**外推域**，无独立静态/trace 证据。→ J3 作为残余标签在 jungle 域合法但**无任何方向正证据**；beehive 修复后 oak 域该残余是否仍存在也未复测（@anchor.idk）。

## 三、任务3：量级预测

密度自查（versions/1.20.1/data worldgen JSON）：
- jungle biome：trees_jungle count = weighted_list(50 w9 / 51 w1) ≈ **50.1 attempts/chunk**；random_selector（configured_feature/trees_jungle.json）：fancy_oak 0.1 + jungle_bush 0.5 + mega 1/3 → **default jungle_tree 概率 = 0.0667 → ≈3.3 棵小树/chunk**（attempts 过 heightmap/biome 门后更少）。
- jungle_bush ≈ 25/chunk（bush 角判同族，A9 已静态对齐——若 J3 型差存在于 bush 同样成立）。
- sparse_jungle：trees_sparse count = weighted(2 w9/3 w1) ≈ 2.1 attempts → 小树 ≈0.14 棵/chunk，**贡献可忽略**。
- 每棵 jungle_tree 角消费 = 16 次（§1.1 勘误后）；每棵 jungle_bush = 层 j 全>0、角消费亦同量级（radius 2, h2 → j=2+1-1-i=2,1 → 8 次）。

**可解释上界**：
- 期望贡献（静态对齐成立）：**≈ 0**——角消费无状态依赖，两侧同构即零漂移。
- 若 trace 揭示任一处消费差（如 y==0 边界差一层 → 每树 4 次漂移）：单树 4-16 次消费漂移会在同 chunk population 流内**级联**（beehive 案例实证：每树 1 次缺抽即致树2 起全错位）→ 每 chunk 3.3 棵小树 × 15-45 leaves/vine 块/树 → 区域级可累积至 **−12055 全量在原理上可覆盖**（上界=整域）。
- 判读：J3 量级潜力大但**先验极低**（两轮静态 ✅）——不排除、不值得先钻；−12055 主嫌疑仍应是 J1（mega 枝干 trunk_set 缺失，确定性 ❌）+ J2（HashSet 同 Y tie 序）。

## 四、决定性实验设计

见 `.investigations/jungle-l/b3-experiment.md`（单树 RNG 消费计数 trace + 三行判据）。

## 四a、Rust 侧 trace 预判读（260905-13 补，cmd-output/b2b3-rust-treediag-chunk29-16.log，J1 修复后载体）

- [CORN-BLOB] 32 行 = **2 棵 × 16 行**，两棵 (y,r,dx,dz) 序列完全一致：y=0 r=1、y=-1 r=1、y=-2 r=2、y=-3 r=2（各 4 角）→ **与基线 j 序 [1,1,2,2] 逐层吻合** ✔
- [CORN-BUSH] 90 行 = **10 棵 × 9 行**，序列一致：y=1 **r=0（dx=0,dz=0 退化角，1 消费）**、y=0 r=1（4）、y=-1 r=2（4）→ **每棵 9 行而非 §三 预估 8**（bush 基线勘误：r=0 层 0==0 成立仍消费 1 次）；blob 不受影响（j 序无 r=0 层）
- 结构注意：[TH] 50 行 vs 仅 2 blob + 10 bush 到 foliage——38 棵直干树未到 foliage（selector 分支/门），不影响 J3 判读但 Java 侧应复现同比例
- Java 比对方案确认：按 [THJ] 树高 + chunk 内序对树；blob 树 = 后随 16 行 [CORN-BLOB] 者；比对键 = (y,r,dx,dz) 元组序列逐行一致（next_outputs 不落日志，叶子落点差由 block 级对比覆盖）；⚠️ Java BushFoliagePlacerMixin 必须同样覆盖 r=0 退化角（dx==0&&dz==0 时 `dx==radius&&dz==radius` 成立仍 nextInt 1 次），否则 9 行基线对不上

## 五、正式裁决（260905-13，Java `.tmp\jungle-l-260905-13\wgdiag-run-v2.log` v3/E5 后 vs Rust `cmd-output/b2b3-rust-treediag-chunk29-16.log`，J1 修复后同载体同 chunk(29,-16)）

### 5.1 blob —— **J3 排除（消费计数级），按 b3-experiment.md 判据 2 核销**

- 两侧各 32 行 = 2 棵 × 16；层序列逐层一致：y=0 r=1 / y=-1 r=1 / y=-2 r=2 / y=-3 r=2（各层 4 角），棵内一致、跨侧一致。
- 元组比对口径（§9.7 声明）：Java mixin 打 |dx|,|dz|（四角呈 dx=1/dz=2 形式），Rust 打带符号值 → 比对键 = (y, r, |dx|, |dz|) + 每层 4 角计数——**该口径下逐行一致**（消费计数/层结构级，非带符号位置级；带符号落点差归 block 级对比覆盖）。
- 判据 2（排除）命中：同树行数与层分布逐树一致 → **短路族在 jungle_tree blob 域零 RNG 消费差，J3 核销**。

### 5.2 bush —— 机制级一致，数量差另案

- Java 45 行 = **5 棵 × 9**：每棵 y=1 r=0（退化角 1 消费）+ y=0 r=1（4）+ y=-1 r=2（4）——与 Rust 10 棵 × 9 的 (y,r) 分组与每棵序列**完全同构** → bush 角短路机制两侧一致、零消费差。

### 5.3 🚩 新候选 J5 登记：jungle_bush 执行数差（rust 10 vs java 5 棵，同 chunk 同载体）

- 定性：**feature 执行数差（上游 selector/attempt 域），超出 J3 短路族判据范围**；bush 与 jungle_tree 经同一 trees_jungle selector，同域树数/高度分布差一并归 J5 证据（Rust [TH] 50 行含重复位 vs Java THJ 分布 1×5,6,7×3,8,9,12,27,28 的树数归组差）。
- 方向性预测：每棵 bush ≈15-25 leaves + 1-3 log → rust 多 5 棵/chunk → rust **多**放 jungle_leaves/log（signed 差为正）。对照残差符号：jungle_l −12055 = rust−vanilla 为负（rust 亏，candidate-beehive §2 口径）→ **J5 是反向抵消项，非 −12055 贡献方向**；修 J5 后 −12055 预计先扩大，真负向项须另找（J1 级联/J2/未明）。
- 定界手段：两侧 [CNT]/[SQ] attempt 级对拍 + selector 分支消费流对拍。
- @anchor.idk("jungle_bush 执行数 2× 差（rust 10 vs java 5）根因未定界——selector 分支序/attempt 门/can_replace 前置差三候选", source="待：[CNT]/[SQ] attempt 级对拍")

## 六、@anchor.idk

- ~~J3 blob trace 级消费计数未采~~ → **已采，J3 核销（§5.1）**；s1-semantics:386 的 8 次勘误维持（16 才对，已被两侧 trace 证实）。
- @anchor.idk("树3+ 残余在 beehive 修复后的 oak/birch 域是否仍存在未复测", source="待：oak 域复测或台账核销")

## 七、引用

- Java：foliage/BlobFoliagePlacer.java:32-57、foliage/FoliagePlacer.java:38-115、feature/TreeFeature.java:66-81
- Rust：worldgen-core/src/tree.rs:174-271（blob 循环/generate_square/is_position_invalid/java_div/place_foliage_block）、:497-547（主链消费序）
- 数据：worldgen/configured_feature/{trees_jungle,jungle_tree,trees_sparse_jungle}.json、placed_feature/{trees_jungle,trees_sparse_jungle}.json、biome/jungle.json:81
- 台账：260905-10-judge-beehive.md:108、260905-10-oak-p2-bB.md、260905-08-scout-tree-rng.md:66、NEXT_SESSION.md:29、13-feature-parity.md:114
