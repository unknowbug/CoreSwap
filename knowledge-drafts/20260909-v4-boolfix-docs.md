# 草稿：V4 布尔解析 bug 修复——docs/09 追加小节 + docs/10 时间线条 + discovered 两条（subagent 产出，主会话应用）

> **core.worker 知识库草稿（2026-09-09）**：已读 SUBAGENT-KNOWLEDGE-GUIDE.md；数字全部取自已定稿素材（v4-fix-verification / v4-eval-conflict / v4-collection / c4-overworld-ablation / review-001），无编造。
> **应用位置**：草稿 A → `versions/1.20.1/docs/09-multi-dimension.md`「V3 结构对拍」节之后（文件末尾）；草稿 B → `versions/1.20.1/docs/10-timewise-archive.md` 末尾追加 2026-09-09 条；草稿 C → `knowledge/discovered/compiler-idioms.md` 追加发现 **#8** + `knowledge/discovered/workflow-patterns.md` 追加发现 **#12**。追加不覆盖。
> **状态纪律**：只写草稿，不改 status，不 git 提交；confirmed 留用户。

---

## 草稿 A：docs/09-multi-dimension.md 追加小节

（接在「V3 结构对拍：……（draft，Degraded，2026-09-08）」节之后，文件末尾追加）

---

## 布尔字段解析 bug 修复签名 B/C（candidate，2026-09-09；judge PASS）

> 承接上节 V3 结构对拍的「归因指向」——V4 生产 ctx dump 先否定「输入差」候选（180 点生产 dump 与 probe 逐项全同），随后解析产物树 dump 锁定**求值层矛盾 = 解析器布尔字段 bug**。本节 supersedes V3 节的处置方向（原「到运行时输入找」的方向由本节取代，V3 原节不删）；V3 的「结构差不存在」排除结论维持成立，仅其「参数全对拍」子项为漏检（对拍的是 JSON 原文而非解析产物）。验证分层 **Partial**（bin-diag 解析树 dump + 定点 apply + 存档口径端到端，非逐位 Full）。

### 根因（supersedes：V3 节处置方向）

- `parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段 `add_surface_depth` / `add_stone_depth`——`JsonValue::Bool` 走 `as_f64()` 返回 `None` → **恒解析为 false**（surface_rules.rs 三处：y_above / stone_depth / water）。
- soul 分支 ceiling 条件 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1+0+0` → 分支该进未进 → 穿透 [7] 兜底 netherrack。V3 的「soul_soil 无条件兜底存在」推读本身正确，错在**解析产物树**；V3 静态对拍「参数全对拍」对拍的是 JSON 原文而非解析产物，被假阴性掩盖。
- 三处定位：L1079（y_above `add_stone_depth`）/ L1093（stone_depth `add_surface_depth`）/ L1116（water `add_stone_depth`）——`as_f64()` 对布尔返回 None；对照 `legacy_random_source` 的 `as_bool()` 读取为正确 API。

### 修复（surface_rules.rs）

- 新增 `SurfaceBuilder::parse_bool_field`：`as_bool().or_else(|| as_f64().map(|f| f != 0.0)).unwrap_or(false)`（bool 优先 / 数字 0/1 兼容 / 缺省 false，与 Java `GsonHelper.getAsBoolean(json, key, false)` 语义一致）。
- 三处调用替换：y_above `add_stone_depth` / stone_depth `add_surface_depth` / water `add_stone_depth`；grep 复核剩余 `as_f64` 命中均为纯数字字段，无残留布尔误读。
- overworld 不受影响（走代码规则树 build_overworld_rule，不经 parse_surface_cond）。

### 验证链（四级回归，seed B = 8576294172403134396）

1. **树复现**（soul_tree_repro，nether.json × surface_rules.rs 解析产物树 dump）：修复前解析树 8 处 `asd=false` 假阴性（其中 3 处 JSON 原文为 true；通用 floor 段等 8 处 JSON 原值即 false，恰成假阴性掩护）；修复后 5 处翻 true（soul ceiling/floor StoneDepth、gravel patch y_above 30/35、basalt floor stone_depth），其余 8 处保持 false = JSON 原值，无一误翻。产物：`.investigations/soul-v4v5/cmd-output/soul-tree-repro-postfix.txt`。
2. **生产 ctx dump**（soul_ctx_dump，180 点生产链，nether 4×4 @3200,3208）：netherrack 103→71，新增 soul_soil=18 / soul_sand=14（103=71+18+14 自洽）；定点 3260,1,3200（sdb=2, sd=3, selector<0）applied 256→**258(soul_soil)**，与 V3 语义推演逐位一致。产物：`cmd-output/soul-ctx-dump-postfix.stderr.txt`。
3. **nether 存档全量回归**（run4 模板照抄，FULL 参照，ReadWorldProbe 存档口径，4×4 @3200,3208）：94.4241%（990108/1048576）→ **run1 96.6215%（1013150/1048576）/ run2 96.5866%（1012784/1048576）**，+2.20pp。run1/run2 差 366 块，在已知同 dll 重跑非确定容差带内（workflow-patterns 发现 #10，实测散布 ~2330 块）——归因成立。
4. **soul 族 per-id 佐证**（save MCA vs FULL ref，同 seed/region/口径）：soul_soil 修复前 1334 → **5771**（ref 5474，偏高 +297，闭合至 ref 邻域，「完全闭合」不成立）；soul_sand 1471→2494（ref 2457）；quartz 2095（ref 1992）/ gold 711（728）/ magma 1543（1533）/ gravel 674（674，精确相等）。

### 附带闭合（同 bug 源次生影响面）

- **签名 C**：nether_wastes「soul_sand_layer 分支 entered 0/60」同 bug 源（floor 段 `add_surface_depth` 误读），随本修复闭合（soul_sand 2494 vs ref 2457 佐证）。
- nether **gravel patch 高度带**（y_above asd，锚 30/35）整体修正（gravel per-id 674=ref 覆盖）。
- **3275,2,3201 从签名 B 证据集剔除**：该点 y=2 ∈ bedrock_floor 随机带（above_bottom 0..5），生产侧 bedrock 先中即返（applied=31），本不用于 soul 表面判定；vanilla 同点亦先判 bedrock_floor。

### 残差（idk / 遗留）

- basalt −1736 / blackstone −434：**B1 surface 残差家族**（已知遗留，非本次范围）；修复前后对照（同 seed/region/口径）：basalt save-ref −3631（2026-09-08）→ −1736——asd 翻转无新负迁移，B1 家族反而收敛。
- 366 块非确定带宽（run1/run2）；**V5 biome 边界带（vs vanilla 足迹）未做**——修复后残差图需重导（readWorldProbe mismatch 全集），残差降至 ~3.4%。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解（ReadWorldProbe）vs vanilla FULL 参照 × 解析产物树 dump（bin-diag）× 生产 ctx dump（stderr）；覆盖面：nether 4×4 @3200,3208 全高度 + 180 点签名集；与既有口径可比（94.4241% 历史值同参照/同 region/同口径），与 SURFACE 77.49% / 纯 Rust 77.43% 口径不可比，分列。
- 本节数字全部带 seed+region+口径三要素（seed B = 8576294172403134396；region 4×4 @3200,3208；存档/树 dump/dump 口径分列）。

### 状态

- **candidate（judge PASS，review-001）**；supersedes 双指针：v3-structure-diff.md 参数对拍子项被 `.artifacts/.b2-soul/v4-eval-conflict.md` 细化取代。confirmed 待用户拍板。过程 → 10 时间线 2026-09-09 条。

---

## 草稿 B：docs/10-timewise-archive.md 追加 2026-09-09 条

（文件末尾追加）

---

## 2026-09-09（soul V4/V5 课题：布尔解析 bug 根因 + 修复 + C4 消融）

### ✅ 一、V4 采集——输入差候选否定

- patch `WG_SOUL_CTX_DUMP`（env 门控点级 ctx dump，OnceLock 点集 + chunk 级门控，零热路径成本）+ bin-diag `soul_ctx_dump` 驱动生产 fill_chunk_blocks，180 点（V2 签名 B/C mismatch 点）180/180 全命中。
- 抽样点 probe CSV vs 生产 dump：biome / sda / sdb / surface_depth / selector **逐项全同**，整规则 apply 一致 → **「probe 复算输入 ≠ 生产 ctx」输入差候选被采集数据否定**。
- 新矛盾：biome=soul ∧ ceiling_ok ∧ selector<0 → applied=netherrack(256)，与 V3「进 soul 分支必得 soul_soil 兜底」结构推演冲突 → 矛盾收敛到求值/解析层。产物：`.investigations/soul-v4v5/v4-collection.md`。

### ✅ 二、树复现——假阴性 8 处

- bin-diag `soul_tree_repro` 直接 dump 解析产物树（修复前）：**8 处 `asd=false` 假阴性**（3 处 JSON 原文 true：soul ceiling/floor / basalt floor / wastes floor / gravel y_above 等；其余 8 处 JSON 原值即 false，恰成假阴性掩护）——「参数全对拍」对 JSON 原文肉眼核对全对，实为解析产物 ≠ JSON 语义。
- 3275,2,3201 裁决：bedrock_floor（above_bottom 0..5）生产侧随机判定先中即返（applied=31），从签名 B 证据集剔除。

### ✅ 三、根因定位 + 修复

- 根因（.b2-soul fan-out 裁决，候选 b）：`parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段 → Bool→None→**恒 false**（surface_rules.rs 三处：y_above L1079 / stone_depth L1093 / water L1116）；soul ceiling `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1` → 分支未进 → 穿透 netherrack 兜底。产物：`.artifacts/.b2-soul/v4-eval-conflict.md`。
- 修复：新增 `parse_bool_field`（`as_bool().or_else(as_f64 != 0).unwrap_or(false)`），三处替换；grep 复核无残留布尔误读。

### ✅ 四、四级回归（seed B = 8576294172403134396，nether 4×4 @3200,3208）

- 树复现（postfix）：5 处翻 true（soul ceiling/floor、gravel y_above 30/35、basalt floor），8 处保持 false=JSON 原值，无一误翻。
- 生产 dump（180 点）：netherrack 103→71，+soul_soil 18 / soul_sand 14；定点 3260,1,3200 applied 256→**258**。
- 存档口径 ×2：94.4241% → **run1 96.6215% / run2 96.5866%**（+2.20pp；run 间差 366 块在 #10 非确定带宽 ~2330 块内）。
- per-id：soul_soil 1334→5771（ref 5474）/ soul_sand 1471→2494（ref 2457）——soul 族闭合至 ref 邻域；gravel 674=ref 精确相等。
- 签名 C（soul_sand_layer entered 0/60）同 bug 源随修复闭合；basalt −3631→−1736（B1 家族收敛，无新负迁移）。

### ✅ 五、judge PASS + C4 overworld 消融量化 + C1 回写

- **judge（review-001）PASS，建议 candidate**：三重锁定（解析树 dump × 定点 apply × 生产 dump 逐位一致）成立；`parse_bool_field` 语义正确且替换完备；回归数字可归因；Degraded/§9.7 声明合规。4 项卫生处置（E9 临时 bin 删除 / index 登记 / supersedes 双指针 / 本 docs 草稿来源确认）→ 主会话清单。
- **C4 overworld 消融**（seed B，overworld 4×4 @3200,3208，存档口径）：默认 mask=0b011（不双跑）**98.9520%**（1556380/1572864）vs 旧双跑 mask=0 **97.3266%**（1530815/1572864）——双跑修复 **+1.6254pp**（+25565 块），差异集中 y=-64..63 features 活跃层（y≥64 两 run 100% 一致）；judge C4 CONCERN（overworld 默认 mask 行为变更未回归量化）量化闭合，无回归证据。⚠️ 单 region 单次，方向性量化非覆盖面结论。产物：`.investigations/soul-v4v5/c4-overworld-ablation.md`。
- **C1 措辞回写完成**：同 region 3 次复跑 = 验证确定性/可复现性（非多 region 覆盖面），判据措辞已按 C1 修正在 09 篇回归节与时间线落实。

### 🔍 残差

- basalt −1736 / blackstone −434（B1 家族遗留）；366 块非确定带宽；**V5 biome 边界带未做**（修复后残差图需重导，残差 ~3.4%）。

### 📌 记录指引

- 结论 → 09 篇追加「布尔字段解析 bug 修复签名 B/C（candidate）」小节，草稿 `knowledge-drafts/20260909-v4-boolfix-docs.md`（supersedes V3 节处置方向，原节不删）。
- 通用模式 → `knowledge/discovered/compiler-idioms.md` 发现 #8（布尔字段 as_f64 恒 false）+ `knowledge/discovered/workflow-patterns.md` 发现 #12（静态对拍须对拍解析产物）。
- 状态：根因 + 修复有效性 candidate（judge PASS 建议）；confirmed 留用户。

---

## 草稿 C：knowledge/discovered 两条

### C-1 → `knowledge/discovered/compiler-idioms.md`（取下一个编号 = **发现 #8**；文件当前最大 #7）

---

## 发现 #8: JSON 布尔字段经 as_f64 读取恒 false——分型标量 API 下的「静默语义腐蚀」签名

- **发现时间**：2026-09-09；**发现者**：core.worker 草稿（soul-v4v5 课题 .b2-soul fan-out 裁决）+ 主会话应用；**来源定位**：`.artifacts/.b2-soul/v4-eval-conflict.md` + `.investigations/soul-v4v5/v4-fix-verification.md`（修复位置 `WorldgenRust/src/surface_rules.rs` parse_surface_cond 三处 + `parse_bool_field`）；**置信度**：candidate（三级数据层证据实锤，confirmed 待用户拍板）；**module**：re-code / swe（数据驱动解析器 / 跨语言 JSON 语义）。
- **观察**：自定义 JSON 包装层若按标量分型提供 API（`as_f64` / `as_bool` / `as_str` 各只对同型返回 Some），则 `x.as_f64().map(|f| f != 0.0).unwrap_or(false)` 读布尔字段**恒得 false 且无任何告警**——不是兼容读取，是静默语义腐蚀。Java `GsonHelper.getAsBoolean(json, key, false)` 是 bool 优先/缺省 false 的类型感知读取，两端 API 语义不等价，直译即错。本例：surface_rule 三处布尔字段（add_surface_depth/add_stone_depth）恒 false → soul 分支条件 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1` → 分支该进未进穿透兜底，nether 存档对齐被压 2.20pp（94.42%→96.62% 修复）。
- **证据**：nether.json L293 `"add_surface_depth": true`（布尔）vs 解析产物树 dump 实测 `asd=false`（soul-tree-repro，8 处假阴性中 3 处为真阳性翻转）；定点 3260,1,3200（sdb=2, surface_depth=3）`2 ≤ 1+0+0`=false 复现 applied=256，修复后 `2 ≤ 1+0+3`=true → applied=258 与 V3 语义推演逐位一致；生产 180 点 dump netherrack 103→71；存档 94.4241%→96.6215%/96.5866%（seed B，4×4@3200,3208，存档口径）。
- **如何利用**：
  - **规则**：分型标量 API 下读布尔一律 `as_bool().or_else(|| as_f64().map(|f| f != 0.0)).unwrap_or(false)`（类型感知 + 数字 0/1 兼容 + 缺省 false），禁止「万能 as_f64 转 bool」；移植/翻译 Java 数据驱动解析器时，逐字段核对 Gson getXxx 的类型容忍面与目标语言 API 的分型行为是否等价。
  - **签名**：「条件永远不成立但无任何告警」+ 解析期零 WARN（读取成功返回 false，不是解析跳过）——凡「分支看起来存在却从不进入」先 dump 解析产物核对布尔字段；与发现 #7 同族（都是「单维度/单分支未覆盖即潜伏」的解析器坑）。
  - 交叉引用：对拍方法教训见 workflow-patterns 发现 #12（对拍解析产物而非 JSON 原文——本发现的假阴性正是被 #12 缺口掩盖的）。

### C-2 → `knowledge/discovered/workflow-patterns.md`（取下一个编号 = **发现 #12**；文件当前最大 #11）

---

## 发现 #12: 静态对拍必须对拍解析产物而非输入原文——「参数全对拍」假阴性掩盖真 bug（2026-09-09）

- **发现时间**：2026-09-09（soul-v4v5 课题 V3→V4；judge PASS 建议 candidate）；**module**：workflow / 验证方法 / 数据驱动解析。
- **观察**：V3 静态结构对拍把「解析器产物树 vs JSON」的对拍做成「肉眼核对 JSON 原文参数」——节点结构逐项一致、参数「全对拍」通过，结论「结构差不存在」。但**中间层（解析器）本身是嫌疑对象**：布尔字段被解析器读成 false，JSON 原文上是 true，「肉眼对拍 JSON」天然查不出——8 处假阴性中多处 JSON 原值恰为 false，进一步掩护。真 bug（as_f64 读布尔恒 false，见 compiler-idioms 发现 #8）被假阴性压制一轮（V3 draft）才在 V4 由解析产物树 dump 锁定。
- **证据**：修复前解析产物树 dump（soul-tree-repro）实测 8 处 `asd=false`，与 nether.json 原文行号逐项对拍——3 处 JSON=`true`（真阳性）、多处 JSON=false（假阴性掩护）；「JSON 原文 / 解析产物树 / 运行时行为」三方对拍闭合后单轮定位根因（`.artifacts/.b2-soul/v4-eval-conflict.md` §1 表）。
- **如何利用**：
  - **规则**：凡对拍「解析器/转换器/中间层」的正确性，对拍物必须是**parse 产物树 dump**，不是输入原文——原文对拍只能证「输入长什么样」，证不了「中间层把它变成了什么」。
  - **工具化**：把 parse 产物树 dump 固化为 bin-diag 常备诊断（本例 `soul_tree_repro`），带 JSON 行号列，逐节点对拍；「参数全对拍」类结论必须注明对拍对象是原文还是产物。
  - **三方纪律**：probe「复算一致」只证 probe 与生产同源，不证与 JSON 规范同源——JSON 规范 / 解析产物 / 运行时三方对拍缺一不可（本例三方各自自洽、互相矛盾，矛盾点即中间层 bug）。

---

## 自检（SUBAGENT-KNOWLEDGE-GUIDE §4）

- [x] 价值门：docs 小节 = 已裁决修复结论（高价值详记）；时间线 = 过程归口 10；discovered 两条 = 跨项目可复用惯用法坑 / 工作流模式（高价值）。
- [x] 根因为机制层面（as_f64 分型语义 / 对拍对象错置），非现象复述；定位含可复用诊断工具（soul_tree_repro / soul_ctx_dump）。
- [x] 被排除假说保留：输入差（V4 否定）❌、结构差（V3 维持成立）❌、3275,2,3201 bedrock 先中剔除；残差 idk 显式。
- [x] 全部数字带 seed+region+口径三要素；来源均为已定稿素材，无编造、无占位符。
- [x] 格式与目标文件末尾对齐（09 篇 wg_set_flags/V3 节风格、10 时间线 ✅/🔍 条目式、compiler-idioms #6/#7 与 workflow-patterns #10/#11 模板）；编号已核对（compiler-idioms → #8，workflow-patterns → #12）。
- [x] 未改 status、未 git 提交、未写 docs 正文（本文件仅为草稿）。
