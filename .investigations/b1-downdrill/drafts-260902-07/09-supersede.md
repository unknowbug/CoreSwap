# 草稿：09-multi-dimension.md「B1 下钻 H1 定案（260902-06）」节取代注记（§15.4）

> 载体：`versions/1.20.1/docs/09-multi-dimension.md` L568 节标题下方、节首引导段之前插入注记块。
> 正文不删不改（原结论保留为历史记录）。主会话应用。

---

> **[supersedes 260902-06]** 本节「B1 下钻 H1 定案」第 1 环（Rust surface 熔岩海缺失——netherrack 实心兜底）**被数据证伪**：`[LAUIDMAP]` Java STATE_IDS 权威映射实测 `19319 = blackstone`、`5854 = basalt`（air=0 lava=96 water=80 netherrack=5850 basalt=5854 blackstone=19319 soulsand=5851 soulsoil=5852 magma=12402 bedrock=79），COLPROF 10/25 列 diff 真相 = **V 黑石底（y=99 恒平）vs C 玄武岩底（y=100~104 贴地形）**——两侧均为实心材质，无任何熔岩缺失（LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零）。基于该环的「熔岩流体填充」修复方案作废未执行；环 2~5（转换面漂移 → delta origin 漂 → 级联/blob 放大）作为**现象**维持成立（cfg 探针独立证据 delta y=111/119/121 vs 99；CountMultilayerPlacementModifier y-零随机 findPos 语义只需「第一转换面不同」即可触发），但其因果入口需重定位。范围限定与口径声明（§9.7 三要素）：本证伪基于**已扫描区域**（seed=8576294172403134396，3200,3208 size=4+外扩环，11,443 公共列）；「99.9423%」为 4×4 固体表面顶块口径，与 10/25 列内部转换面差（y=99~104，顶块以下）不可直接比较。新机制候选（材质分支差 / biome 输入差 / 随机序列差 / 前置地形形状差）待四候选 fan-out 判别。见 10 时间线 260902-07 条 + `.investigations/b1-downdrill/facts-260902-07.md`。原节数据与结论不删不改（§15.4 取代链）。

---

## 应用位置说明

- 目标节标题：`## B1 下钻 H1 定案：熔岩海缺失 → 转换面漂移 → delta/blob 链式放大（candidate，judge 审查中，260902-06）`（L568）。
- 注记插在标题行之后、`> 承接 260902-05 开工点 H1/H2.…` 引导段之前，与 L566 既有 supersedes 注记同形态。
- 建议（非本草稿范围）：节标题末尾可加「（环1 已证伪，见上方 supersedes 注记）」——是否改标题由主会话裁决；注记本身已足以表达取代关系。
