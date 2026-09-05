# .b2 — jungle_l 残差 −12055：J2（HashSet 桶序 tie）候选定界

- 角色：core.worker（subagent 隔离），候选编号 .b2
- 课题：J2 = TreeFeature HashSet 集合在 Y 稳定排序后的同 Y tie 序泄漏（scout A10/A11，R-1 落点）
- 状态：**draft**（静态定界 + 决定性实验设计；未授 status，运行时验证模板见 `b2-experiment.md`）
- 证据层：Degraded（静态一手源对读，无运行时 trace）
- 输入：260905-13 scout 产物 + Java 一手源（.tmp\scout-260905-08\mcsrc）+ worldgen-core\src\tree.rs

## 1. Java 语义精确刻画：tie 序从哪来、是否确定

### 1.1 泄漏路径

```
TreeFeature.generate :122-125  set/set2/set3 = Sets.newHashSet()   // java.util.HashSet（Guava 工厂，等价 new HashSet<>()）
  biConsumer2 → set2（log positions）:130-133
  blockPlacer  → set3（leaf positions）:134-145
TreeDecorator.Generator ctor :47-52
  new ObjectArrayList<>(set2/set3)          // 拷贝构造 = 以 HashSet.iterator() 序填入 list
  .sort(comparingInt(Vec3i::getY))          // List.sort = TimSort，稳定
```

稳定排序保序 ⇒ **list 中同 Y 元素的相对序 = HashSet.iterator() 序**。之后三个 decorator 全部按 list 迭代序消费：
- TrunkVine :21 `getLogPositions().forEach`（每 log 恒 4×nextInt(3)，西→东→北→南）
- LeavesVine :28 `getLeavesPositions().forEach`（每 leaf 恒 4×nextFloat(p) + placeVines 下行 0 消费）
- Cocoa :31-33 `list.get(0).getY()` 基线 + 同表 `filter(y-基线<=2).forEach`（stream 保 encounter order）

set/set4（root/decoration 集）不进 decorator；placeLogsAndLeaves :178/:194 的迭代序只影响 distance 属性重写（R-4 族），不在本候选内。

### 1.2 HashSet 迭代序的确定性（跨 JVM / 同 JVM）

HashSet = HashMap。迭代序 = 桶序：table 下标 `index = (cap-1) & (h ^ (h>>>16))`（HashMap.hash 的扰动函数，JDK 8 起至今未变），迭代按 table 数组下标升序扫描，桶内按插入序（JDK 8 改尾插 + resize split 保持桶内相对序）。键 hash 完全确定：`Vec3i.hashCode() = (y + z*31)*31 + x`（Vec3i.java:66-68，一手源已核）。

**结论**：对**相同插入序列**，HashSet 迭代序是**确定的**——同 JVM 重跑一致，跨 JVM/跨 Java 版本（8→21+）也一致（扰动函数、尾插、resize 保序均版本稳定；树化阈值 8/桶，本场景集合 ≤~200 元素、hash 分布良好，实际不会触发树化改变序）。**它不是随机序，而是「难复现的确定序」**：依赖插入序 + hash 布局 + table 容量增长史（逐次 ×2 resize），Rust 的 Vec 插入序与它只可能偶然一致。

- 推论 A（可复现性）：Java 侧同一 seed 同一树，dump 两次必然相同 → 「跑两次验证不稳定」的 scout 设想**不成立**，改为「Java dump 序 ≠ Rust 插入序」的直接比对。
- 推论 B（可修性）：由于完全确定，Rust 侧可精确复刻（模拟 HashMap 桶布局：按插入序 add + 容量 16 起倍增 + `(h^(h>>>16))&(cap-1)` 分桶 + 桶内插入序）——若 J2 实证成立，这是 1 个纯函数级修复，不需要猜。
- @anchor.idk("树化（TREEIFY_THRESHOLD=8/桶）在 jungle 树集合规模下是否实际触发未验证——集合 ≤200、Vec3i hash 分布良好，静态判断不触发", source="待：Java dump 时顺带打印 table 桶计数")

## 2. Rust 侧现状逐点核对（tree.rs）

Rust 用 `Vec` 按生成序收集（tree.rs:559-560），decorator 内 `sort_by_key(Y)` 稳定排序（:344/:366/:382）——与 Java 的稳定排序对齐，但 **tie 输入不同：Rust tie = 生成插入序，Java tie = HashSet 桶序**。受 tie 序影响的放置点全表：

| # | 位置 | tie 敏感性 | 机制 |
|---|------|-----------|------|
| R-① | TrunkVine :363-378 每 log 4 向 | **RNG 流不变**（每 log 恒 4×nextInt(3)，与序无关）；放置格集基本不变（两 log 共享目标格时后到者见非 air 跳过，幂等）；**vine face 布尔属性可差**（同一格先到者的 face 胜出） | 2×2 干同 Y 4 log 每 Y 必 tie（mega）；小树干单列每 Y 唯一不 tie；横向枝干相距 2 且中间格 air 时才可能同格竞争 |
| R-② | LeaveVine :379-399 每 leaf 4 向 + placeVines 下行 ≤4 | **RNG 流不变**（每 leaf 恒 4×nextFloat）；**放置格集可真差**：同列叶对（相距 2 的水平邻居、链穿越）按序先后会改变下行链长度（先短后长 vs 先长后短，净差 ±1..4 格/次碰撞）；face 属性同 R-① | 这是 J2 造成**块计数差（id 口径）**的唯一现实机制 |
| R-③ | Cocoa :337-361 | **基本免疫**（对 scout A11-① 的精化）：jungle_tree 干单列 → logPositions 每 Y 唯一 → 无 tie；`list.get(0).getY()` vs Rust `min_by_key(Y)` 取的都是最小 Y（所有同 Y 候选 Y 相等）→ **基线值恒等**；cocoa 是首 decorator，air 判定只依赖树本体块（序无关）；`nextInt(3)` 仅 air 时消费 → 无碰撞则流也不变 | mega_jungle 配置无 cocoa（TreeConfiguredFeatures:366-379），cocoa 碰撞场景在 jungle_l 不存在 |
| R-④ | Beehive :401-436 | 非 jungle_l 配置，本候选外 | — |

结论：J2 的可观测效应 = **vine/cocoa 附着集形状差 + vine face 属性差；RNG 流跨树不变**（除假想的 mega 分支 log 碰撞 cocoa，配置上不存在）→ J2 **不产生级联结构漂移**，与 J1（确定性缺失，流级联）机制性质完全不同。

## 3. 量级预测（上界论证）

- **tie 发生率**：jungle_tree（小树）trunk 单列 → logPositions 零 tie；leaves blob 每层多叶 → 必 tie。mega 2×2 干 → logPositions **每 Y 4 个必 tie**；树冠更大（约 60-100 叶/树）。
- **tie → 计数差的转化率**：tie 本身不必然产生差；需「同列碰撞几何」（相距 2 的叶对 + 链穿越，LeaveVine）或 face 竞争（仅 state 口径）。粗估每树 0-5 次有效碰撞（小树）/ 0-15 次（mega），每次 ±1..4 格 vine。
- **域上界**：每 chunk ~15 jungle 树 → 单 chunk vine 计数差上界 ~**几十格（≤~150）**，双向抵消后净差更小。
- **判据**：−12055 是 region 级。若 region 含 N chunks 且 12055/N ≫ 150，**J2 至多次要项**（主嫌疑回到 J1 确定性缺失 / J3 / J4 口径）。需主会话提供 region chunk 数核对（实验设计 §4 判据 C4）。

## 4. 与 scout A10/A11 的修正关系

- A10（集合 tie 序差存在）：**成立且比 scout 表述更强**——序确定可复现，非「不可复现」。
- A11-①（`list.get(0)` vs `min_by_key`）：**降级为无害**——基线只取 Y 值，同 Y tie 不影响 Y 值本身。
- A12/A13（消费序/恒定消费）：确认不变；J2 的 RNG 流不变性由此锁定。

## 5. 置信与遗留

- 结论 1（tie 序确定、泄漏路径、RNG 流不变、cocoa 免疫）：Degraded 静态，逐行一手源对读，置信 candidate 级证据待实验确认后申请。
- @anchor.idk("J2 对 −12055 的实际占比未定界——需 b2-experiment 的两侧单树集合 dump + region chunk 数", source="待：b2-experiment.md §3/§4")
- @anchor.idk("HashMap 树化是否实际触发（见 §1.2）", source="待：Java dump 桶计数")

## 五、裁决（draft，260905-13 b2b3 数据：Rust b2b3-rust-treediag-chunk29-16.log + Java wgdiag-run-v2.log v3/E5 批）

对拍键声明：Java `t0=(x, y, z)` y=最低 log；Rust `t=(bx, by, bz)` by=表面基点（最低 log = by）——**y 锚差 1 属语义差**，配对只用 (x,z)。

### 5.1 逐 Y 组内序比对结果

| 侧 | TREESET 树 | 备注 |
|---|---|---|
| Java v3 | (473,-249) trunk 473:70..79（10 log 纯单列）；(477,-250) trunk 477:70..76（7 log 纯单列） | THJ 共 14（1×5, 5,6,7×3,8,9,12,27,28），Generator 仅 2 棵（其余树无 decorator 或放置失败） |
| Rust | 15 棵（10 bush + 2 小树 + 2 带横向 offset log 的 h7 + 1 mega 468,-245） | (x,z) 集合与 Java 2 棵**完全不相交** |

**配对结果 = 0**：Java 两棵 TREESET 树 (473,-249)/(477,-250) 在 Rust 同 chunk 无任何同位成功树。同位尝试点存在但树高不同——Rust (473,-249) TH=1（bush）vs Java 同位 10-log 树；Rust (477,-250) TH=8 vs Java 7。**同位异高 ⇒ 两侧树级 RNG 流在该点之前已漂移**。

### 5.2 判据裁决

- **C1/C2/C3 均不可判**（判据输入 = 同一棵树的两侧 dump；配对数 = 0）。J2 在 chunk (29,-16) 上**当前不可归因**——不是被排除，是被**上游差遮蔽**（C5 降级）。
- 结构前提核对：Java 两棵树叶层同 Y 大量元素（如 y=77 层 22 叶）+ trunk 单列零 tie——与 Rust 侧小树形态一致的 tie 几何**确实两侧俱在**；一旦上游对齐，C1 评估可立即执行（模拟方案 §3/③ 不变）。
- 新增锚定证据：
  1. 🚩 **上游差候选（新，建议立 .b4）**：同位异高（473,-249: 1 vs ~10；477,-250: 8 vs 7）+ 树位集整体不相交 + Rust TH 尝试 ~50 次 vs Java THJ 14——尝试数/树高/位置三级漂移，全部先于 decorator 阶段。
  2. 🚩 **Rust 小树横向 offset log 实证为异常**：Java 两棵 7/10-log 树 trunk 均纯单列（StraightTrunkPlacer 语义），Rust h7 树 trunk 含 (472,74,-246)/(479,74,-247)/(479,74,-246) 类横向 log——trunk_placer 路由或 trunk_set.push 位置错，单独立项核对。
  3. 🚩 Rust 空 trunk 树 (471,71,-247)（leaves 21 块）：Java 无对应状态；Cocoa `list.get(0)` 在 Java 该状态会 IndexOutOfBounds ⇒ vanilla 不可达，Rust 守卫差（A16 实证）。
- Java THJ 27/28 两棵 mega 级无 TREESET 的判读：THJ 在 getHeight 后即打 ⇒ 尝试在本 chunk；Generator 探针在 generate 成功路径 ⇒ **放置失败（域守卫 ④/⑥ 或 canReplace 全拒）**，非邻 chunk（邻 chunk 树不会在本 log 打 THJ）。@anchor.idk("具体失败守卫未定位", source="待：Java 侧失败分支打点")

### 5.3 结论（draft）

J2 机制本身（tie 序确定、泄漏路径、RNG 流不变、cocoa 免疫、vine 链碰撞）维持 §1-§3 静态结论；**运行时归因被上游树流漂移阻断**——先修 .b4（上游差）+ trunk 异常（发现 2）+ 守卫差（发现 3），然后重跑本实验（判据不变）。
