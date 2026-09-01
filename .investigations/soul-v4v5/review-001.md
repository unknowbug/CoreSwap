# review-001 —— V4 soul 残差根因定论 + 布尔解析修复有效性（core.judge，260902-03）

> 审查对象：`.artifacts/.b2-soul/v4-eval-conflict.md`（draft）+ `v4-fix-verification.md`（candidate 前置）
> 三源核对：① 产物快照（两 md）② git 工作区 diff（surface_rules.rs + bin-diag/bin + docs）③ cmd-output 验证记录（前后各一对 dump/repro + run1/2 + mask log）
> 本意见只出结论，不改任何 status。

## 逐项意见

### 1. 根因证据链完整性 —— 通过
三重锁定成立且可复核：
- **解析产物树 dump**：`soul-tree-repro.txt`（修复前）[4] soul ceiling/floor StoneDepth 均实测 `asd=false`，与 nether.json 原文 `true`（产物 §1 表逐行带行号）冲突——中间层（解析器）直接锁定。
- **定点 apply 复现**：`3260,1,3200`（sdb=2, surface_depth=3）修复前 applied=256，修复后同 ctx applied=258，且 `2 ≤ 1+0+3` 求值路径与 `SurfaceCond::test` L84-93 逐行吻合。
- **生产 dump 逐位一致**：`soul-ctx-dump.stderr.txt` 生产 ctx（biome/sda/sdb/surface_depth/selector）与 probe CSV 全同（v4-collection），排除输入差备择；applied=256 与生产一致。
- 备择解释排除记录在案：输入差（V4 采集否定）、结构差（V3 静态对拍维持成立）、签名 B 3275,2,3201（bedrock_floor 先中，§2 有 ctx + 首中语义依据，剔除合理）。未发现未排除的合理备择。
- **唯一弱点**（不阻断）：生产 dump 的 ceiling_ok 镜像是「JSON 正确语义」的硬编码（`1+0+sd+0`），故 dump 的 ceiling_ok 与产物树语义天然不一致——产物 §0 已自证说明此点，判定诚实。

### 2. 修复正确性 —— 通过
- `parse_bool_field`（surface_rules.rs L1023-1026）：`as_bool().or_else(as_f64 != 0).unwrap_or(false)` —— 与 Java `GsonHelper.getAsBoolean(json, key, false)` 语义一致（bool 优先 / 缺省 false；数字兼容是超集，防御性合理）。
- **替换完备性已实测**：grep 当前工作区 `as_f64` 剩余命中均为纯数字字段（offset/secondary_depth_range/min-max_threshold/absolute anchor），三处布尔字段（L1089 y_above / L1103 stone_depth / L1126 water）已全部换用 helper，无残留 as_f64 读布尔。
- **产物树闭环**：postfix repro 5 处翻 true（soul ceiling/floor、gravel y_above 30/35、basalt floor），其余 8 处保持 false = JSON 原值——假阴性无一误翻。
- overworld 不受影响成立：overworld 走代码规则树（build_overworld_rule），不经 parse_surface_cond；diff 未触碰 overworld 路径。

### 3. 回归数字合理性 —— 通过
- run1 96.6215% / run2 96.5866%（log 原文核对，同 seed 8576294172403134396 两跑），差 366 块 << workflow-patterns #10 的 2330 块同 dll 非确定带宽，归因成立；94.4241%→96.6% 同参照/同 region/同口径，可比性三要素在产物 front-matter 已声明。
- soul 族闭合表述**有支撑但建议收敛措辞**：soul_soil 5771 vs ref 5474（+297，超 ref 约 5.4%）、soul_sand 2494 vs 2457（+37）——「闭合至 ref 邻域」成立，「完全闭合」不成立（soul_soil 仍偏高 ~300 块，落在多跑非确定带宽内但未单独证明）。v4-fix-verification 的表述（「闭合至 ref 邻域」）准确，保留该措辞即可。
- 生产 dump 分布 103→71(+18 soul_soil +14 soul_sand) 自洽（103=71+18+14）。

### 4. 声明合规 —— 通过（附一处 index 缺口，见 §6）
- Degraded 声明到位：v4-eval-conflict front-matter 明确「Degraded 静态阅读 + Partial（bin-diag 复现，未逐位对拍）」——V3 静态对拍降级与 V4 Partial 边界均如实标注，未冒充 Full。
- 可比性声明三要素（载体/覆盖面/历史口径）在 front-matter 完整，且明确「与 V1/V2 存档口径、V3 纯静态口径不可比」。
- 残差声明：basalt/blackstone B1 家族、366 块带宽、V5 残差图未重导均显式列为遗留（idk 性质），诚实。
- retry/证据饱和：V3→V4 每轮均有新数据层证据（静态→生产 dump→解析树 dump），无超限迹象。

### 5. 残留风险（gravel/basalt 分支覆盖）—— 通过，附一条点名建议
- gravel：全量回归 + per-id gravel 674 vs ref 674 精确相等，y_above 修正已被数据覆盖。
- basalt：修复使其 floor stone_depth asd 翻 true，回归后 basalt −1736 / blackstone −434 仍为残差且已声明归属 B1 遗留家族——**建议在升 candidate 前补一句显式对照**：「修复前后 basalt/blackstone per-id 数值变化量（或不变）」，证明 asd 翻转未对 basalt 分支引入新的负迁移（当前记录只给了残差绝对值，没给修复前后对比）。

## 三源核对差异源标注
- 产物快照 vs diff：一致（diff = 声明的诊断门控 + 三处替换 + soul_dump_points，无越界改动）。
- 产物快照 vs 验证记录：一致（log/dump 原文数字与产物引用逐项相符）。
- **发现的不一致（处理项，不影响技术结论）**：
  1. **E9 违规**：临时 bin 副本 `WorldgenRust/src/bin/soul_ctx_dump.rs`、`src/bin/soul_tree_repro.rs` 未删（bin-diag 正本已在）。v4-collection 明说「临时挪 src/bin 编译」，用完未清——发版全量绿检查会被这两个 bin 阻塞（1.0.22 前科同型）。
  2. **产物契约缺口**：`.artifacts/.b2-soul/index.yaml` 未登记 `v4-eval-conflict` 条目（仍停在 260902-02 的 v3 条目）。
  3. **supersedes 双指针缺失**（v0.20 §15.4）：v4-eval-conflict 单向声明「细化 V3」，但 v3-structure-diff.md 侧无被取代标注/指向 v4 的指针（原结论不可改写 ≠ 不可加取代标注；应补 supersedes 记录而非改 v3 正文）。
  4. docs/09-multi-dimension.md + 10-timewise-archive.md 已有工作区改动——升 candidate/归档前核对该 diff 是否由知识库 subagent 草稿产出（工作区存在 `knowledge-drafts/`，疑似合规，需主会话确认草稿来源）。

## 总体结论

**PASS（建议：根因定论 + 修复有效性可升 candidate）**——技术证据链三重锁死、修复语义正确且替换完备、回归数字可归因、声明合规；上述 4 项为卫生/流程缺口，不否定技术结论，但 **candidate 授予前建议处置 1-3 项**（临时 bin 删除 + index 登记 + supersedes 补标）。confirmed 仍留待用户拍板。

## 需主会话处置清单
1. 删除 `WorldgenRust/src/bin/soul_ctx_dump.rs`、`WorldgenRust/src/bin/soul_tree_repro.rs`（E9）。
2. `.artifacts/.b2-soul/index.yaml` 登记 v4-eval-conflict（含验证分层与可比性声明）。
3. 补 §15.4 supersedes 双指针：v3-structure-diff 加「被 v4-eval-conflict 细化取代（参数对拍子项）」标注。
4. 确认 docs/09、10 改动的 subagent 草稿来源；不合规则补派知识库 subagent。
5.（建议，非阻断）修复验证记录补 basalt/blackstone 修复前后 per-id 对照一行。
