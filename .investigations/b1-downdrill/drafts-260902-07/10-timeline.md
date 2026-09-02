# 草稿：10-timewise-archive.md 追加条目（260902-07）

> 载体：`versions/1.20.1/docs/10-timewise-archive.md` 末尾追加（接 260902-06 条之后）。主会话应用。

---

## 260902-07（实际 2026-09-02 18:36–19:1x；B1 下钻：H1 环1 证伪定案——id 标注误读）

- ❌→✅ **H1 环1 被证伪（本 session 主线转向，judge PASS）**：原计划 LAVAAUDIT 定性 → 熔岩海修复 → 回归；实际 LAVAAUDIT 探针新增（runtime ColProfProbeMixin `colprof.mode=lavaAudit` 模式：v1 只记 above=lava → v2 加记 below=lava 面向 lavaSurfY + `[LAUIDMAP]` 一次性 id 映射；build.gradle 补 colProfMode/colProfR -P→-D 映射）→ v1 指标盲区暴露（99.4% 一致率不记 above=air 转换，不构成世界一致证据，已废弃）→ `[LAUIDMAP]` id 映射实锤：**19319=blackstone 非 lava、5854=basalt 非 netherrack**（air=0 lava=96 water=80 netherrack=5850 basalt=5854 blackstone=19319 …）——昨日 COLPROF「`99|0->19319` = air→lava 熔岩海面」标注纯系误读。
- ✅ **COLPROF 10/25 列 diff 真相**：V 黑石底（y=99 恒平）vs C 玄武岩底（y=100~104 贴地形），两侧均实心材质；快照时间线（b1_colprof_firstsnap.py）证明 diff 在第一枚举快照即分叉（V#0/C#0 同构异材质，T 序列稳定），非 feature 事后改列伪影。
- ✅ **LAVAAUDIT v2 全扫**（11,443 公共列）：air→lava 面向**两侧均为零**——该区域 feature 阶段起点无任何熔岩面向；熔岩以流动态（96）存在于两轮相同位置（lavaTopY 分布峰值 23~24 一致）；lavaTopY 逐列差 329/11443（2.9%）、n 差 60 列（judge D1 补测，本 session 实测）。昨日「终态列 dump 9216 列逐列全同」不矛盾——该口径只记顶块（roof y=128），y=99~104 材质差不可见。
- ✅ **judge PASS（环1 证伪 + 转向）**：环 2~5（转换面漂移 → delta origin 漂 → 级联/blob 放大）作为现象保留（cfg 独立证据 delta y=111/119/121 vs 99）；CountMultilayerPlacementModifier y-零随机 findPos 语义只需「第一转换面不同」即可成立。judge 六项 CONCERN：lavaTopY 逐列已补（329 列）；only_v=10/only_c=56 覆盖缺口未解释；结论限 3200,3208 区域；SURFACE 99.9423%（4×4 固体表面顶块口径）vs 内部转换面差——口径三要素须显式声明；LAUIDMAP 只跑 vanilla 轮，cpp 轮待补；「标注三查」应入 NEXT_SESSION 开工检查项。
- 🔍 **下一轮方向（judge 设计，四候选判别 fan-out）**：(a) surface rule 材质分支差 (b) biome 判定输入差 (c) surface rule 随机序列差 (d) 前置地形形状差（NOISE/density 阶段列高度差——judge 指出若成立则 a/b/c 全降次生）；判别探针 = SURFACE 前/后逐列 dump（材质序列+biome id+顶面 y）；候选非严格互斥，按「判别目标」设计各自可独立排除；**先跑 (d)**。
- 📝 **错误台账 3 条**（五段式详见 b1-errors.md E-B1-9/10/11）：raw id 标注当公理继承（E-B1-9，重大——「标注三查」与 seed 三查同级）/ 探针指标盲区 lavaAudit 测不了 air→lava 面向（E-B1-10）/ grep 行首锚对带前缀 log 恒零命中假「零输出」（E-B1-11）。
- 📝 lib/dll 零改动（仅 runtime 探针 + .tmp 数据/脚本）；修复方案（熔岩流体填充）作废未执行。
- 🔍 下一步：判别探针 SURFACE 前/后逐列 dump → 四候选 fan-out 先跑 (d) → LAUIDMAP 补 cpp 轮。
