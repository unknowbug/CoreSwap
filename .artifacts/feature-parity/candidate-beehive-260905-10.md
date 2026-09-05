# Candidate c-B（BeehiveTreeDecorator 实装）—— 260905-10

- **status: candidate（judge APPROVE-WITH-CONDITIONS，C1-C4 已核销；confirmed 待用户拍板）**
- judge 产物：.investigations/feature-parity/260905-10-judge-beehive.md
- **C1 核销**：候选方向集修正为 {E,S,W}（java GENERATE_DIRECTIONS = HORIZONTAL N,E,S,W 去 NORTH，Direction.java:499）；注释流序同步更正；修复后单 chunk 复跑无回归（树1/2 仍逐位重合）
- **C2 核销**：两臂 region signed 输出已落盘 .investigations/feature-parity/260905-10-cmd-output/{v25-bee-ca-region.txt,v25b-bee-only-region.txt}；「bee_nest 0/0」口径改为「signed 差 0（位置级不保证）」
- **C3 核销**：worldgen_handle.rs:105/:875 注释已按 §15.4 更新（supersedes FEA-9「WG_CA_MIN 默认关」表述，实证 :876 env_enabled 默认开）；**默认开长期态 = 待用户裁决项**
- **C4 声明**：runtime/1.20.1/java 侧 mixin（TrunkPlacer/BeehiveDecorator/Square/Count/coreswap.mixins.json/build.gradle）为 untracked local-only（基线 commit 0fc44d3 后未入库），git diff 不覆盖；提交时需一并 git add
- 代码：`worldgen-core/src/tree.rs`（TreeDecorator::Beehive 变体 + parse + generate；WG_TREEDIAG 诊断打点见 placement.rs/tree.rs）
- 载体：#52 确定性 region dump（seed 8576294172403134396，2193 chunks，idk7_region_dump 同源 bin）

## 1. 根因链（P2 逐树对拍，Full 分层）

- fan-out .bA（top-position/trunk 语义差）**否证**：四点逐行对拍同构（260905-10-oak-p2-bA.md）
- fan-out .bB（selector/provider/replaceable 差）**排除**；范围外发现 rust 漏实现 BeehiveTreeDecorator（260905-10-oak-p2-bB.md）
- 决定性数据：java p=20 树1 `SQ(602,-244) → THJ=6 → [BEE]`；rust 修复前树1 decorator 0 消费 → 树2 起流漂移；修复后树1、树2 base 逐位重合（`.tmp/feature-parity-260905-10/p2_first_tree.py`）
- 实装语义（BeehiveTreeDecorator.java:43-75 对拍）：恒 1 次 nextFloat 门（probability 0.002）→ i 推导 → 候选（y==i logs × W/S/E）→ findFirst 空气位放置 bee_nest（朝南）→ `2+nextInt(2)` + `ix×nextInt(599)` 蜂数消费。注：java `Collections.shuffle` 用自有 Random（非世界流）；rust 确定序取首候选（放置点可偶差、流不受影响，idk-bee2）

## 2. 验证记录（region 全量 signed vs vanilla，v18 同口径）

| 臂 | oak_l | jungle_l | vine | jungle_log | birch_l | 备注 |
|---|---|---|---|---|---|---|
| pfix 基线（260905-08） | +29830 | -15151 | -16829 | -2453 | -1964 | |
| c-A（ca1b，260905-09） | +31662 | -12027 | +2134 | -2129 | -1712 | |
| **bee-only（CA=0）** | +28886 | -15151 | -16840 | -2453 | -1921 | jungle 域零变化（jungle 树无 beehive，机制自洽）|
| **bee+CA（默认态）** | +30807 | **-12055** | **+1972** | -2153 | **-973** | 最优臂；birch 较 ca1b 再收敛 739 |

- bee_nest 0/0 两侧一致（gate 0.002 极少触发）
- dump 哈希留痕：bee+CA = 4D216088…C48FD（region_bee.bin ≡ region_bee_ca.bin，二次证明两臂同 env 态）；bee-only(CA=0) = 19BA41F4…953E474
- chunk 级判别：修复后 p20 树1/树2 base 逐位一致，树3 起仍有状态依赖消费差（四角短路族，未闭合——见残余）

## 3. 残余与 IDK

- IDK-bee1：leaves[0]/logs[0] 取「生成序首元素」，rust 用 y 极值近似
- IDK-bee2：java shuffle 自有 Random 非确定序 vs rust 确定序（放置点可偶差，流不影响）
- 残余主项：jungle_l -12055（jungle 树域，beehive 不覆盖）；树3+ 状态依赖短路与 R-1（HashSet 序）继续持有
- 交接口径修正（本轮廉价验证推翻交接两处）：① WG_CA_MIN 实际默认**开**（worldgen_handle.rs:876 用 env_enabled，unset→true）——FEA-9「默认关」表述不成立；② pfix「树1 7 根 log」为主会话读数错误（y=78 是 leaves）

## 4. 采集效率整改（本轮新增，已识别未实施）

- mixin chunk 过滤（population 行携带 x/z → static curChunk，噪声 -99%）；spawn point 预置目标 chunk 缩短预生成；结构性方案 = 单 chunk 直驱 harness（免 runServer）
