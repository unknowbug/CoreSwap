# 知识库草稿（subagent 产出，主会话应用+验证）：260907-01 工作块

> 产出者：知识库更新 subagent（core.worker 隔离）；依据 `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`。
> 素材：column-read-260907-01.md / convergence.md / .b1 + .b2 candidate.md / review-260907-01.md（judge N-1~N-10）。
> 置信度：全部 draft/candidate，禁止 confirmed。发现时间 = 260907-01（主会话已 Get-Date 锚定）。

---

## 一、knowledge/discovered/workflow-patterns.md 追加草稿（文件末尾追加，接 #70）

## 发现 #71: 「starts 层零差 + 块级签名量级先验对比」裁决结构差假设——无需 children dump 即可否定/压制「整件级差」，但 children 一致性是盲区须 §9.7 声明（260907-01）；candidate

- **发现时间/发现者**：260907-01，fan-out .b2 worker 判定 + judge N-1/N-8 通过；core.worker subagent 草稿 + 主会话应用。来源课题 jungle-l（chunky 地形差归因 fan-out）。
- **module**：workflow-patterns / 结构差量级先验判据
- **观察**：chunky 双臂（vanilla vs stageMask=3）region 对比中，799/3025 chunk 有地形差，其中结构语汇残余仅 ~330 块（cobblestone 172 + mossy 101 + oak_planks/fence/rail/cobweb/spawner 零头），散布 ~8 chunk；而两臂 Structures NBT starts 层（键名 + status + id 三字段）逐项零差（mineshaft 13 / shipwreck 3 / … 完全相同）。
- **证据/推理**：若存在「整件级结构 ±差」（某 mineshaft/ruined_portal 组件一臂生成一臂缺失），块级签名预期 = 数百块木板/栅栏连片 × 跨多 chunk（一个走廊组件即数百块材料）。实测 ~330 块/3025 chunk（~0.09 块/chunk）与该先验**差一个数量级以上** → 「整件级差」在 starts 层 + 量级先验双重否定。剩余 ~330 块 = feature 阶段相互作用差（geode 壳压 mineshaft 木板等，通道①/②下游），非独立第 5 通道。
- **盲区（§9.7 诚实项，复用本判据时 MUST 同步声明）**：starts 三字段对比**未覆盖 children/pieces BoundingBox 明细**——「两臂 pieces 树逐件一致」未被直接验证，量级+形态检验只能间接压制、不能封死「starts 相同但 children 组装不同」的残余可能。闭案需 children dump（本块列为未闭合采集项，不阻塞收口）。
- **如何利用（判据 MUST）**：
  1. 任何「结构阶段差」疑点，先做**量级先验对比**再决定是否深挖：实测结构特征块差总量 ÷ chunk 数 vs 「整件级差」先验签名（数百块连片 × 多 chunk）——差一个数量级以上即可否定/压制整件级假设，零成本免 children dump。
  2. starts 层零差（有无生成判定一致）与块级量级检验**两支齐备**才出否定结论；单支不裁决。
  3. 量级先验否定**不等于结构差闭案**：children 一致性盲区 MUST 显式声明（§9.7），并给出消解采集项（children BoundingBox dump + 与异常块簇 chunk 交叉定位）。
- **家族索引**：#66（执行体语义前提——本判据只在同执行体/已知语义两臂间有效）、#20（NBT 不信内坐标，region+slot 定位）、#26（chunky 双臂载体）。
- **证据**：.investigations/jungle-l/fanout-260907-01/.b2/candidate.md（§1 量级相容性检验 + 盲区声明）、convergence.md（.b2 证伪裁决）、review-260907-01.md（N-1/N-8）。

## 发现 #72: diff 统计「name 双桶相加」上界陷阱——两臂差异对的 name 计数直接相加是上界粗界；top-N 截断列表计数总和 vs 全量总数不自洽 = 口径疑点签名（260907-01）；candidate

- **发现时间/发现者**：260907-01，fan-out .b1 worker §6③ 自查 + judge N-3/N-4 实证修正；core.worker subagent 草稿 + 主会话应用。来源课题 jungle-l（chunky 地形差归因 fan-out）。
- **module**：workflow-patterns / 统计口径陷阱（#59 跨语言数值对拍 / #18 跨口径数字不可续推 家族的 diff 统计形态）
- **现象**：① convergence 将 grass_block 2271 + dirt 2868 两桶 name 计数相加得「~5.1k 疑似树 below-dirt 泄漏」，被 judge N-3 修正为**上界粗界**——dirt 2868 混入 blob 族的 `gravel↔dirt` 成分，与 grass_block↔dirt 对无关；树 below-dirt 真实量级 ≈2.3k 起（grass_block 侧基本纯净）。② .b1 top 名计数总和 ≈7 万 vs terrain 全量 113k 不自洽——成因不止 top8 截断：judge N-4 补出第三成因 = 采集脚本对「单 palette 无 data 的 section 差异」按 ±2048 "unknown split" 近似计入 + 跨臂 section 不齐（Y 只在一臂存在）整段跳过的采集盲区。
- **根因（机制）**：per-block diff 的「按 block name 分桶计数」与「按差异对 (A↔B) 分桶计数」是两种口径——同一 block name 可出现在多个差异对中（dirt 同属 grass_block↔dirt 与 gravel↔dirt），name 桶计数不等于任何单一机制的贡献量；两桶相加 = 对共享成分重复计账。截断/近似/跳过类采集近似则制造「局部可加、全局对不上」的缺口。
- **如何利用（判据 MUST）**：
  1. 归因量化（「某机制贡献 ~X 块」）**禁止用 name 桶计数直接相加**——要么按差异对口径统计，要么显式声明「上界粗界，含他族共享成分」，并给出净化下界（如本块 ≈2.3k）。
  2. **「top-N 截断列表计数总和 vs 全量总数不自洽」本身是口径疑点签名**：发现即声明（不先修数据），把精确分账列为采集项；裁决若只依赖量级带/类型谱则粗界够用，需单点数字的结论必须等全量重算。
  3. 采集脚本口径三查（写入采集器注释/交付声明）：截断（top-N）、近似（无 data section ±2048 计入）、盲区（跨臂 section 不齐整段跳过）——三者都会进 total，引用 total 前先问三者是否在声明中。
- **家族索引**：#59（数值化比较禁字符串比对——采集层口径）、#18（跨口径数字不可续推）、#70（概率/占比一律区间粗界，非可交换性）。
- **证据**：.investigations/jungle-l/fanout-260907-01/review-260907-01.md（N-3/N-4 全推理链）、.b1/candidate.md（§1 dirt 双重身份 + §6③）、convergence.md L27/L31-32（修正后表述）。

## 发现 #73（简记，中价值）: fan-out worker 交付「让渡清单」实践——候选对辖区外证据只标归属建议不解释，主会话收敛时统一分账（260907-01）；draft

- **发现时间/发现者**：260907-01，fanout-260907-01 .b1/.b2 实践提炼；core.worker subagent 草稿 + 主会话应用。
- **module**：workflow-patterns / fan-out 交付契约
- **是什么**：各 worker 候选在 §裁决 附**让渡清单**——凡不属本候选解释范围的差异簇，只标「签名 + 量级 + 建议归属桶」，不做解释、不计入本候选账（.b1 例：结构残留→.b2、geode→结构候选、aquifer→11 篇挂起域、grass_block↔dirt 暂挂待 y 分布核查）。主会话收敛时按清单统一分账，judge 核对「无同一块簇被两个桶同时认领」（N-2 通过项）。
- **如何利用**：fan-out 候选模板增列「让渡清单」小节；收敛/审查第一步核让渡清单 ↔ 归属桶一一对应，防候选间越界归因与重复计账。与 #10 三阶段归因（通道归口）互补：#10 管「差归哪个通道」，本条管「候选间谁不许碰哪块账」。
- **证据**：.investigations/jungle-l/fanout-260907-01/.b1/candidate.md（§4 让渡清单）+ convergence.md（air/water 桶归属）+ review-260907-01.md（N-2）。

---

## 二、knowledge/INDEX.md 追加行草稿（文件末尾追加）

```
> 260907-01 追加：workflow-patterns 新增**发现 #71**（「starts 层零差 + 块级签名量级先验对比」裁决结构差假设——无需 children dump 即可否定「整件级差」（~330 块/3025 chunk vs 数百块连片先验差一个数量级以上），children 一致性盲区 MUST §9.7 声明 + 消解采集项随判据给出）+ **发现 #72**（diff 统计「name 双桶相加」上界陷阱——dirt 桶混入 gravel↔dirt 成分，5.1k 上界 vs ≈2.3k 净化下界；「top-N 计数总和 vs 全量总数不自洽」= 口径疑点签名，成因含单 palette section ±2048 近似 + 跨臂 section 跳过盲区）+ **发现 #73 简记**（fan-out worker 交付「让渡清单」实践——辖区外证据只标归属建议不解释，收敛统一分账防越界归因）。来源：.investigations/jungle-l/fanout-260907-01/（柱全列直读 + 双候选 fan-out + judge N-1~N-10 有条件通过）。
```

---

## 三、versions/1.20.1/docs/10-timewise-archive.md 追加条草稿（文件末尾追加）

```
## 260907-01（实际 2026-09-07 Get-Date 锚定：jungle-l 柱全列直读 → chunky 地形差归因 fan-out（.b1/.b2）→ judge 有条件通过）✅ judge PASS-with-conditions（条件已应用）；无代码改动

> 过程产物 `.investigations/jungle-l/column-read-260907-01.md`（柱直读）+ `.investigations/jungle-l/fanout-260907-01/`（convergence.md + .b1/.b2 candidate.md + review-260907-01.md）；数据 `.tmp/jungle-l-260906/chunky/diff_per_chunk_260907-01.txt` + structures_nbt_260907-01.txt；脚本 `.tmp/jungle-l-260906/column_read_260907-01.py` + collect_fanout_260907-01.py。**本块无任何 src 代码改动**（judge 三源核对 N：git diff 仅 .investigations/.artifacts/knowledge/docs）。通用模式 → workflow-patterns #71/#72/#73（subagent 草稿 → 主会话应用）。

- ✅ **柱全列直读（开工点 1，region NBT 程序化直读，Full 载体）**：柱 (483,-230) 全列 y=-64→319 **零差**（地形+植被逐位全等）；柱 (500,-234) 差异 21 处**全部在 y=74-94 feature 层**（vanilla 整根 jungle_log + 树下 grass_block→dirt，coreswap 无树干），y≤73 地形逐位全等。
- ✅ **交接结论廉价独立验证（STEP 1 纪律）**：通道②（Rust 地形输入差参与 483 柱分歧）draft ~0.55 → **支持降级**（483 柱全列地形零差直证地形输入一致）；blob 石残差 ≈15 块/chunk 不触及焦点柱。
- ✅ **§15.4 取代记录**：「500 柱地面低 6 格」（f3-twochannel-260906-09 §5 树基高度代理推定）被本块 column-read 取代——region 载体下地形基座一致，差异为植被层。原结论未改写；**反向指针待 f3-twochannel-260906-09 归档时补注**（judge N-5 条件）。
- ✅ **fan-out 双候选（互斥：地形差主力归属）**：.b1「blob 石残差域」**支持**——类型谱（granite/diorite/andesite 双向均衡 ≈48.7k）、量级带（≈15-16 块/chunk，judge N-6 降格「同量级带参照」：口径混合）、簇状空间分布三证据与 13 篇已知记录吻合，无需新机制。.b2「结构阶段独立第 5 通道」**证伪**——starts 层零差 + 块级签名 ~330 块/3025 chunk 与整件级差先验差一个数量级以上（判据 → #71）；geode 两臂均 Java vanilla feature 放置（mixin 只拦 NOISE/SURFACE 源码引证），geode 差 = 通道①/②下游表现面。children BoundingBox 盲区 §9.7 声明。
- ✅ **air/water 桶归属（开工点 3）**：water 846 → aquifer 挂起域（11 篇）；cave_air 548 三分（geode 壳 ~180-260 / 结构语汇残余 ~330 未闭合 / 洞穴 carve 下游）——无独立新通道。全区域地形差完整归账已开封通道，**无第 5 归因通道**。
- ✅ **judge 审查（review-260907-01，PASS-with-conditions）**：N-3「5.1k」修正为 name 双桶相加上界（→ #72）；N-4 口径疑点补第三成因（单 palette ±2048 近似 + section 跳过盲区）；N-5 取代链反向指针待补；N-6 量级表述降格；N-8 .b2 内部数字勘误（130→~200）。收敛主结论「无第 5 独立归因通道」建议 candidate。
- 🔍 **open（未闭合采集项，不阻塞收口）**：① 13 mineshaft + 2 ruined_portal children BoundingBox dump（消解 ~330 块 cobble/cobweb/spawner 簇归属）；② grass_block↔dirt y 分布核查（树 below-dirt vs 表层 rule 差）；③ 全量对表重算（top8 截断 + ±2048 近似 + section 跳过三成因合并消解）；④ column_read 脚本两处外观 bug（air 前缀永真比较 / sanity 无条件打印，不影响差异数据）。
```

---

## 四、自检（SUBAGENT-KNOWLEDGE-GUIDE §4 对照）

- [x] 价值门：#71/#72 高价值（可复用判据/口径陷阱签名）详记；#73 中价值简记；候选 D 未过门不写（500 柱取代 = 一次性结论，归 10 时间线；反向指针经验已被 #66 N-A 覆盖）。
- [x] 根因为机制层面（name 桶与差异对口径混同 / 量级先验与整件签名不相容），非现象复述。
- [x] 定位含可复用方法（top-N vs 全量不自洽 = 口径疑点签名；starts 零差 + 量级先验两支齐备）。
- [x] 被排除假说保留：.b2 结构差通道证伪（❌ 非第 5 通道）完整入 10 时间线；children 盲区未删除。
- [x] 数字全部来自素材实测记录（48.7k / ~330 块 / 2271+2868 / ≈7 万 vs 113k / 846 / 548），无编造、无占位符。
- [x] 载体正确：判据 → workflow-patterns #71-73；过程 → 10 时间线；INDEX 末尾日期追加条。
- [x] 置信度全部 draft/candidate，无 confirmed。
- [x] 格式与 workflow-patterns 末尾（#66-#70 体例）及 10 时间线末尾（260906-09 深夜条体例）对齐。
