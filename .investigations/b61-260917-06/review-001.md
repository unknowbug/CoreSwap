# B6-1 T1-T4 + Phase 2.5 审查意见（review-001，260917-06）

```yaml
status: opinion-only        # 只出意见；不修改任何产物 status
role: core.judge（isolated subagent）
block: 260917-06
review_scope: T1-T4 产物 + Phase 2.5 验证 + 落盘契约（T5 落地前问题抓取）
three_source_baseline:
  snapshot: .investigations/b61-260917-06/（switch-sources.json / candidates/.b1,.b2 / t4-adjudication.md / classification.yaml / switches-registry.yaml / phase25-verification.md / cmd-output/）
  git: HEAD=3e0d434 "feat(b61): switch mapping reconciliation T1-T4 + behavioral sentinels (260917-06)"；.tmp/b61-260917-06/ 为未入库临时区（符合 §八.13）
  verification: phase25-verification.md §2/§3 + cmd-output/vmargs-sentinel.log（一手复核）
independence_note: |
  本意见对 B 项做了一手独立取证（直读 versions/1.20.1/java/build.gradle 与全部 cmd-output 原文），
  未沿用 t4-adjudication.md §2 的自述表格。
verdict: PASS-with-conditions
```

---

## A. 数据一致性 — 通过（含 1 处文字口径瑕疵 → S1）

**一手复核结果（脚本计数，非目测）**：

| 核对项 | 实测 | 判定 |
|---|---|---|
| `switch-sources.json` consistent / dead / orphan | 112 / 4 / 24 | ✅ |
| M1_declared = consistent + dead | 112 + 4 = **116** ✓ 与 `counts.M1_declared=116` 一致 | ✅ |
| M2_consumed = consistent + orphan | 112 + 24 = **136** ✓ 与 `counts.M2_consumed=136` 一致 | ✅ |
| 差集总数 dead + orphan | 4 + 24 = **28** | ✅ |
| `switches-registry.yaml` rows | **140** = 112 + 28 ✓（`counts.total: 140` 与实解 140 行一致） | ✅ |
| registry 各 verdict 计数 | CONSISTENT 112 / DEFECT-MISSING-MAPPING 13 / DEFECT-DEAD-SWITCH 3 / SCOPED-DIRECT-D 11 / UNRESOLVED 1 | ✅ 与 `classification.yaml` 实条数逐一吻合 |
| `classification.yaml` 实条数 | 28 行（DEFECT-MM 13 / DEAD-SWITCH 3 / SCOPED 11 / UNRESOLVED 1） | ✅ |
| classification 行集 vs (orphan ∪ dead) | **missing=[] / extra=[]**，无重复（dups=[]） | ✅ 两分支 28 项覆盖无遗漏无重复 |
| `t4-adjudication.md` §5 计数闭合 | 13 + 3 + 11 + 1 = **28** ✓ | ✅ |

**⇒ A 项主体通过**：三处计数（抽取器 → 定性表 → 登记表）机械闭合，140 = 112 + 28 成立，28 项集合与两分支辖区完全对齐。

**瑕疵（不影响数值，影响可读性）**：
`classification.yaml:28` 的区段注释写 **`# ---- SCOPED-DIRECT-D（13）----`**，但该区段下实际只有 **11** 行（与 registry/§5 的 11 一致）。这是**注释遗留**（疑为早期草稿 13 未随改判同步），数值本身正确。
> **S1**：把 `classification.yaml:28` 区段注释 `（13）` 改为 `（11）`，与区段实条数及 §5 口径一致。

（另注：`t4-adjudication.md` §3.2 标题写「**裁决后 15 项**」，同节正文又称 `coreswap.exec`/`maxinflight` 已改判不再计，实为 11 项，§5 也记 11 —— 标题与 §5 自相矛盾，见 S2。）

---

## B. 裁决规则的合法性 — **一手核验后：族清单属实，但规则存在 1 处内部不一致（M1）**

**一手取证（直读 `versions\1.20.1\java\build.gradle`，逐行核对 §2 表格）**：

| 族 | §2 自述 | 一手实测（行号） | 判定 |
|---|---|---|---|
| surfaceDump | 5/6（dim 缺） | `:190` 父门 `-Dsurfacedump.probe=1` + `:192-196` **chunkX/chunkZ/size/out/outPost = 5 个子参数**；`dim` 无 | ✅ **属实** |
| colProf | 3/5 | `:187-189` **probe/mode/r = 3**；`x`/`z` 无 | ✅ **属实** |
| blobProbe | 2/6 | `:179` 父门 `-Dblob.probe=1` + `:181` **blobProbeOut = 1 个子参数**（连父门共 2 行） | ✅ 属实（口径为含父门则 2） |
| heightProbe | 0/2 | `:130` 仅 `-Dheight.probe=true`，**无任何子参数映射** | ✅ **属实** |
| exec 族 | execpool 有 / exec 无 | `:162` `-Pexecpool` → `-Dcoreswap.execpool=`；**`coreswap.exec` / `coreswap.maxinflight` 在 1.20.1 build.gradle 全文零映射行** | ✅ **属实** |
| chunkRandom | 1/3 | `:139` 仅 `-PchunkRandomProbe` → `-DchunkRandom.probe=true` | ✅ 属实 |

**⇒ §2 表格的族映射清单经一手核验属实，无错记。** 裁决规则的两侧判据（族内 ≥1 先例 ⇒ DEFECT；族内零先例 ⇒ SCOPED）在上述数据上均成立，`height.x/z` 判 SCOPED 的族内事实基础正确，`surfacedump.dim`/`colprof.*`/`blobProbe.*`/`exec` 判 DEFECT 的族内事实基础亦正确。

**但规则**自身有 1 处**内部不一致**（这是本审查最重要的实质发现）：

规则表述为「族内存在 ≥1 个兄弟参数 `-P` 映射 ⇒ 同族未映射项 = DEFECT」。按此表述机械套用：
- `bench.threads` / `bench.worldgen` —— `bench` 族 `bench.seed/size/originX/originZ/out` 在 `build.gradle:74-78` **全部有映射**（一手确认），**族内先例充分**，按规则**应判 DEFECT-MISSING-MAPPING**；但最终定性为 **SCOPED-DIRECT-D**（`classification.yaml:29-30`）。
- `colDump.targets` —— `:183-186` `-PsurfaceColDump` → `-DsurfaceColDump=1` **且** `-DcolDump.out` 有映射，**族内先例存在**；b2 在 `.b2:250` 明写「**形态同 3.2/3.4（族内不对称）**……本分支倾向 DEFECT」，最终仍判 SCOPED。
- 反向：`coreswap.light.blockabi`/`oldcollect` 判 SCOPED 的理由是「light 族**无任何 `-P` 映射先例**」（§3.2 说明）——与规则自洽 ✅。

即：**规则在 3 项（bench.threads / bench.worldgen / colDump.targets）上未被机械执行**，实际是按「消费点在**探针专用路径** + 缺省值即权威路径 + 缺『用户会自然尝试 `-P`』证据」另行豁免。这**未必错**（b2 的让渡理由有据），但**裁决规则没有把这条豁免写进判据**，导致规则与结果表不一致——规则自称「机械可判」，实际含一条未声明的例外。

> **M1（MUST，T5 落地前）**：在 `t4-adjudication.md` §2 的族内对称性判据上**补一条显式豁免子句**（例：「族内虽有先例，但消费点属探针专用路径且缺省值即权威路径、且全仓文档零宣传 `-P` 用法者，判 SCOPED-DIRECT-D」），并把 `bench.threads` / `bench.worldgen` / `colDump.targets` 三行在 §3.1/§3.2 的归属**按该子句显式落一条理由**；否则 T5 补映射时无法机械复算「哪些族该补、哪些不补」，且该 3 项与 §2 表格表面冲突可被后续误读为漏项。

---

## C. 行为化验证强度 — **不通过（M2 MUST）**

**一手复核 `cmd-output/vmargs-sentinel.log`**（全文 grep `BEGIN|END`、`^VMARG: -D`、`height`）：

| 实测 | 结果 |
|---|---|
| `===B61-VMARGS-BEGIN===` 标记出现次数 | **1**（仅一次运行） |
| 全部 `VMARG: -D` 行 | `bench.seed/size/originX/originZ/out`、`surfacedump.probe`、`surfacedump.out`（= **哨兵 A**） |
| 日志中 `height` 命中数 | **0** |
| 日志结局 | `> Task :runServer FAILED` / `BUILD FAILED in 13s`（FabricLoader `Failed to remap mods!` → java 非零退出 1） |
| `sentinel-surfacedump.log` 中 `B61-VMARGS` 标记 | **0**（该日志是另一次 boot 记录，无哨兵输出） |

**⇒ 哨兵 B（`-PheightProbe` 对照）的原始输出在落盘证据中不存在。** `phase25-verification.md` §3 引用的 `===B61-VMARGS-BEGIN=== / VMARG: -Dheight.probe=true / ===B61-VMARGS-END===` 三行**在任何 cmd-output 文件中都无对应记录**（唯一的 marker 对属于哨兵 A）。

逐点结论：
1. **是否构成独立于源码复述的证据？** **部分是。** 读 `t.jvmArgs` 确是**发射点本体**，强于复述 `build.gradle` 源码——这一方法设计**成立且值得保留**，`phase25-verification.md` §1 的定位准确。但**证据的落盘链断了**（见 2/3），方法优点未能转化为可引用证据。
2. **哨兵 A 是否足以支撑「`-PsurfaceDumpDim` 静默无效」？** **不足以单独支撑，且现记录存在两处削弱**：
   - (a) 该次 `runServer` **FAILED**（mod remap 失败，进程非零退出）。jvmArgs 打印发生在 `doFirst`，**参数组装面已成立**，因此「`-Dsurfacedump.dim` 未发射」这一**窄结论**在逻辑上仍站得住（发射面在进程退出前已完成）；
   - (b) **但**：既然 `-PsurfaceDumpDim=minecraft:the_nether` 被传入而未出现在发射面，正确的**排除性对照**是——同一命令**去掉** `-PsurfaceDumpDim` 时发射面应当**逐字相同**（证明该 `-P` 确实被忽略），以及**换一个已映射的 `-PsurfaceDumpChunkX`** 时发射面应当**多出**对应行（证明发射面确实会响应 `-P`）。**这两个对照均未做**，因此「静默无效」的因果链缺少「发射面对 `-P` 敏感」的正向标定。当前证据只证明「本次发射面里没有 dim 行」。
3. **哨兵 B 是否真有判别力？** **无法评估——其原始输出缺失。** 且哨兵 B 的设计本身判别力偏弱：它证明「heightProfbe 族只发射父门」，这对「族内零先例 ⇒ SCOPED」是**同义复述**（发射面与 `build.gradle:130` 是同一事实的两个视角），**不构成独立判据**；真正有判别力的对照是「族内有先例时发射面**确实**包含那些子参数行」（即用一条 surfaceDump 正例证明发射面会随 `-P` 变），而这条恰恰没做。

> **M2（MUST，Phase 2.5 补做后方可进 T5）**：
> ① 重跑哨兵 B（`-PheightProbe`）并**落盘原始输出**到 `cmd-output/`（与哨兵 A 同格式，带 BEGIN/END 标记）；
> ② 补做**正向标定对照**：同一 `-PsurfaceDump` 命令加 `-PsurfaceDumpChunkX=1` 重跑，断言发射面**新增** `-Dsurfacedump.chunkX=1`（证明发射面对 `-P` 敏感）；
> ③ 在 `phase25-verification.md` §2 中**如实记录**该次 `runServer` 以 `BUILD FAILED`（mod remap 失败）结束，并说明「参数发射在 `doFirst` 完成，失败发生在之后的进程启动」——现文 §5 只提「boot 已 Done 3.651s / probe 未产出属预期」，**未披露 A 次运行的 FAILED**，属**选择性陈述**，须修正。

---

## D. 改判的痕迹与依据 — **依据成立，但「改判」的事实描述有误（S3 SHOULD，含 M3）**

**一手复核**：
- `.b2-defect-classification.md:44`（总览计数表）与 `:252-253`（逐项表第 7/8 行）**明确把 `height.x`/`height.z` 列入 `NOT-A-DEFECT（让渡 b1）`**，并在 `:240` 节标题写「NOT-A-DEFECT 清单（让渡 b1 辖区）= 12 项」、`:258` 计数闭合式亦把该 2 项计入**让渡**而非 DEFECT。
- `.b2:252` 的理由行逐字写：「**按任务书第 4 条判据 = 旁路**。**让渡 b1** 正式定性」。

**⇒ b2 从未把 `height.x/z` 判为 DEFECT**，它把这两项**让渡给 b1**，且**自己已倾向「旁路」**。而 `t4-adjudication.md:46` 节标题写「DEFECT-MISSING-MAPPING（13 项 → 裁决后 11 项确认 + **2 项改判 SCOPED**）」、`:64` 写「**改判 2 项 → SCOPED-DIRECT-D**（裁决规则推翻 b2 原判）……**b2 该 2 项误判**」。

**这是事实误记**：T4 把「b2 的让渡项」重述为「b2 判 DEFECT 后被推翻」，凭空给 b2 记了一次误判。`classification.yaml:38` 与 `switches-registry.yaml` 对应行的 reason（「裁决改判（原 b2 判缺陷）」）**继承了该误记**，会污染后续引用。

**依据本身是否成立？** **成立**：族内零先例（一手确认 `build.gradle:130` 仅父子门）+ javadoc 明写 `-Dheight.x=` 直传（b1 `.b1:62-63` 引自 `HeightProbe.java:12`，b2 `:252` 独立复核同一行且一致）——**两条独立分支对同一 javadoc 一手证据一致**，改判结论正确。

**留痕是否合规？** **部分**：`t4-adjudication.md:42` 保留了「b1 对 height.* 定性正确」的正确面归因 ✅，但**未保留「b2 对该 2 项的处置原文（让渡）」**，反而以「推翻 b2 原判」覆盖之——违反错误优先/取代链精神（§15.4：被推翻的定性应留取代记录，而非重述对方立场）。

> **S3（SHOULD）**：把 `t4-adjudication.md:46` 标题与 `:64` 的「改判/推翻 b2 原判/b2 误判」改为**事实准确**表述：「`height.x/z` 为 **b2 让渡项**（b2 自身即倾向旁路），T4 据此**确认** SCOPED-DIRECT-D」，并注明改判依据 = 族内零先例 + javadoc 直传（双分支一致一手证据）；同步修正 `classification.yaml:38-39` 与生成表中对应 reason 的「原 b2 判缺陷」措辞。
>
> **M3（MUST，T5 落地前）**：`switches-registry.yaml` 是 T5 的唯一事实源且由脚本从 `classification.yaml` 生成——修正 `classification.yaml` 的 reason 后**必须重跑 `gen_registry.py` 重新生成**（不得手改 registry），否则登记表与定性表分叉。

---

## E. 降级与边界声明 — **基本诚实，1 处需补强（S4）**

| 声明项 | 一手核对 | 判定 |
|---|---|---|
| `static: Degraded` | `phase25-verification.md:8` 与 `t4-adjudication.md:10` 一致标注静态审查 | ✅ |
| `behavioral: Partial`；`NOT_DONE: Full` | `:9-10` 显式列出，并说明「补映射后端到端等价属 T5 之后」 | ✅ 分层诚实，未冒充 Full |
| 覆盖面（2 哨兵 vs 26 项外推） | §5 ① 明写「其余 26 项未逐个哨兵……**属外推非实证**」 | ✅ **诚实且明确**（符合 §9.7 覆盖面声明要求） |
| 未覆盖面 ②③ | 明确列出「未验证补映射后行为」「未跑端到端世界生成」 | ✅ |
| `max.bg.threads` 的 UNRESOLVED 处置 | `t4-adjudication.md:85` 与 `classification.yaml:26` 均标 UNRESOLVED，并**显式写「不得与 fjp1 同格引用」**，理由（消费方可能在 MC 发行 jar 内）说明清楚 | ✅ **诚实**——且 b1 `.b1:44` 独立保留意见「形态上更像 SCOPED-EXTERNAL」，与 UNRESOLVED 相容 |

**唯一缺口**：`phase25-verification.md` **未声明本轮验证动作的 in-place 副作用与逆**（§9.8 verification temporality）——本次跑动在 `runtime/1.20.1/java/run/` 下启动了 runServer 并触发 FabricLoader remap（日志显示对 `.fabric\processedMods\*.jar` 执行了 `deleteIfExists` 且**抛 IOException**），且落盘了 `.tmp/b61-260917-06/jtmp/` 与 `sd.txt` 类产物。这些属「改共享执行环境 / 覆盖缓存」形态的副作用，**未见逆登记，也未见不可逆声明**。

> **S4（SHOULD）**：在 `phase25-verification.md` 增一小节「§9.8 副作用与逆」——登记：① `.fabric\processedMods\` remap 缓存被删改（逆 = 无需还原，下次构建自然重建；但须声明**未能确认删除是否成功**——日志有 IOException）；② `runtime/.../run/` 世界/日志未被写入（或如实说明）；③ `.tmp/b61-260917-06/jtmp`、`sd.txt` 为 derived 产物（天然合规）。并加一句「本次未改写任何生产代码/配置」的显式声明（§6 已有一句，但未覆盖执行环境）。
> 注：按 §9.8 **不得要求自动回退**，本项只要求**登记**。

---

## F. 落盘契约与 status 纪律 — **通过**

| 核对项 | 实测 | 判定 |
|---|---|---|
| 全部产物 status | `switch-sources.json` 无 status 字段（数据产物）；`t4-adjudication.md:4` = `draft`；`phase25-verification.md:4` = `draft`；candidates 内 `.b1/.b2` 条目级 `candidate`（有源码 file:line 一手证据，合法） | ✅ |
| 越权 candidate/confirmed | **未发现**任何产物自我标 `confirmed`；`t4-adjudication.md:106` 明写「本裁决为 draft，candidate/confirmed 由后续 judge + 用户定」 | ✅ **纪律良好** |
| 登记表生成器不伪装 | 直读 `.tmp/b61-260917-06/gen_registry.py:41`：非 CONSISTENT 组 default = **`("UNCLASSIFIED", "")`**；`:70-71` 对 `UNCLASSIFIED` 计数并打印「⚠️ …不得当作已定性」 | ✅ **真实生效**（本次因 classification 已覆盖 28/28，`counts` 中无 UNCLASSIFIED，说明无漏定性——该分支未触发但逻辑存在且正确） |
| 生成器 default 分支覆盖 | `cls.get(n, default)` 仅对**未在 classification.yaml 出现**的名字生效；28 项全覆盖已由 A 项机械验证 | ✅ |
| 临时区纪律 | `.tmp/b61-260917-06/`（抽取器/生成器/init script）未入库；`git status` 显示工作区干净 | ✅ 符合 §八.13 |
| 三源核对 | 快照 vs HEAD `3e0d434` vs `cmd-output/` 验证记录——**C 项发现快照（§3 哨兵 B）与原始记录不一致**，其余一致 | ⚠️ 见 M2 |

**⇒ F 项通过**（唯一扣分项源自 C 项的证据链，非 status 纪律问题）。

---

## G. T5 落地前风险提示 — 建议纳入（S5/S6/S7）

| 风险 | 一手核对 | 建议 |
|---|---|---|
| **G1 `-P` 命名惯例二义**（点分 `-Pbiome6.colDump` vs 驼峰 `-PblockProbeFull` vs 无点 `-PsurfaceDumpChunkX`） | `build.gradle:125-126`（`biome6.cellDump`/`biome6.colDump` 点分）vs `:115-116`（`blockProbeDimension`/`blockProbeFull` 驼峰）vs `:192-196`（`surfaceDumpChunkX` 驼峰）——**三种形态并存，属实** | **S5**：T5 补映射前，先从 `build.gradle` 机械导出一份「每族 `-P` 命名形态表」，规定「新补项与该族既有兄弟**逐字同形**」，并对 `surfacedump.dim`（该族已全驼峰：`surfaceDumpChunkX/SurfaceDumpSize/...`）明确应命名 `-PsurfaceDumpDim`（**不得**写成 `-Psurfacedump.dim`）——#19 已三犯，须机械防住。 |
| **G2 `benchVmArgs` 闭包作用域（#47）** | `build.gradle:150-151` 既存注释**已显式防住**：`⚠️ 必须写在 benchVmArgs 闭包内（run 只在此作用域可见；写进 tasks.matching{} 配置期即失败，#47）` | **S6**：T5 所有新增映射行**必须落在 `benchVmArgs` 闭包内**（`:190-197` 等区段同级缩进）；建议 T5 后跑一次 `-P<新名>` 哨兵（复用 `print_vmargs.gradle`）逐项确认发射，而非仅静态 review。 |
| **G3 1.21.6 侧是否同病** | `switch-sources.json` coverage 声明扫描面**含** `versions/1.21.6/java/build.gradle` 与 `src`；`t4-adjudication.md:86` 明确把「1.21.6 `BulkWb.java:81` 类注释 vs `10-timewise-archive.md:112`『BULKWB_ON 已翻转』状态不一致」**转出**为 1.21.6 覆盖课题 | **S7**：T5 前明确 1.21.6 的处置口径——是「同批补映射」还是「另开课题」；若同批，须按 1.21.6 自己的族先例重算（不得沿用 1.20.1 的族表外推，§9.7 等价档位 E2/E3 须声明）。另 `max.bg.threads` 仅存在于 1.21.6（`b1:40`），其 UNRESOLVED 与 1.21.6 覆盖面重叠。 |

---

## MUST / SHOULD 条件编号清单

| 编号 | 级别 | 条件（一句可执行修正） |
|---|---|---|
| **M1** | MUST | 在 `t4-adjudication.md` §2 族内对称性判据上补一条**显式豁免子句**（族内有先例但消费点属探针专用路径 / 缺省值即权威路径 / 全仓零宣传 `-P` 用法 ⇒ 仍判 SCOPED），并对 `bench.threads`、`bench.worldgen`、`colDump.targets` 三项按该子句显式落理由，消除规则与 §3.2 结果表的表面冲突。 |
| **M2** | MUST | 补做并落盘 Phase 2.5 行为化证据：①重跑哨兵 B（`-PheightProbe`）并把原始输出写入 `cmd-output/`；②加做**正向标定对照**（`-PsurfaceDump -PsurfaceDumpChunkX=1` 断言发射面新增 chunkX 行，证明发射面对 `-P` 敏感）；③在 `phase25-verification.md` 如实披露哨兵 A 次运行以 `BUILD FAILED`（mod remap 失败）结束。 |
| **M3** | MUST | 修正 `classification.yaml:38-39`（及经其生成的 `switches-registry.yaml`）中「原 b2 判缺陷」的误记措辞后，**重跑 `.tmp/b61-260917-06/gen_registry.py` 重新生成登记表**（禁止手改 registry）。 |
| **S1** | SHOULD | 把 `classification.yaml:28` 区段注释 `SCOPED-DIRECT-D（13）` 改为 `（11）`，与其下 11 行实条数及 `t4-adjudication.md` §5 对齐。 |
| **S2** | SHOULD | 把 `t4-adjudication.md` §3.2 标题「裁决后 **15 项**」改为「**11 项**」，与同节正文（已剔除 exec/maxinflight）及 §5 一致。 |
| **S3** | SHOULD | 把 `t4-adjudication.md:46/:64` 的「2 项改判 / 推翻 b2 原判 / b2 该 2 项误判」改为准确表述：「`height.x/z` 系 **b2 让渡项**（b2 自身已倾向旁路），T4 予以**确认**为 SCOPED-DIRECT-D」。 |
| **S4** | SHOULD | 在 `phase25-verification.md` 增「§9.8 副作用与逆」小节，登记 runServer 对 `.fabric\processedMods\` 的删改（含 IOException 未确认成功）、`.tmp/` derived 产物，并声明未改写生产代码/配置（只登记，**不要求回退**）。 |
| **S5** | SHOULD | T5 补映射前先机械导出「每族 `-P` 命名形态表」，规定新补项与该族既有兄弟**逐字同形**（如 `surfacedump.dim` 应为 `-PsurfaceDumpDim`），防 #19 四犯。 |
| **S6** | SHOULD | T5 所有新增映射行 MUST 落在 `benchVmArgs` 闭包内（#47），落地后用 `print_vmargs.gradle` 哨兵逐项确认发射，而非仅静态审查。 |
| **S7** | SHOULD | T5 前明确 1.21.6 侧处置口径（同批补映射 or 另开课题）；若同批，须按 1.21.6 自身族先例重算并声明 §9.7 等价档位，不得沿用 1.20.1 族表外推。 |

---

## 推荐状态

- **产物快照（`t4-adjudication.md` / `classification.yaml` / `switches-registry.yaml` / `phase25-verification.md`）：保持 `draft`。**
  理由：A/B/D 项已具备 candidate 级静态证据基础，但 **C 项行为化证据链不完整（M2）**、**B 项裁决规则存在未声明例外（M1）**——两点均为 T5 落地的直接输入，未闭合前不建议升 `candidate`。
- **`candidates/.b1-bypass-classification.md` / `.b2-defect-classification.md`：保持现状（条目级 candidate）**。
  两者一手证据（源码 file:line）质量良好，b1 对弱项（`chunkRandom.seed/seed288`）主动降为 `draft`、b2 对 `max.bg.threads` 拒给 candidate——**自我降级纪律执行到位**，是本轮产物质量的最大亮点。
- **不授予 `confirmed`**（本角色无此权限；`confirmed` 留人类）。

---

## 最终结论

**PASS-with-conditions** —— 计数体系（140 = 112 + 28，28 项无遗漏无重复）与 §2 族映射一手事实均经独立复核属实、status 纪律与生成器防伪装机制合规，但裁决规则存在 1 条未声明例外（M1）、哨兵 B 原始证据缺失且 A 次运行 FAILED 未披露（M2）、`height.x/z` 的「改判」事实描述误记了 b2 立场（M3）——三条 MUST 须在 T5 落地前闭合。
