# 260912-01 知识库成稿草稿（docs 载体：10 时间线追加块 + 07 主题篇小节）

> ⚠️ **§15.4 注记（应用后校正，2026-09-12）**：本稿 `multiset + sorted-sequence 双判全等` 的表述**已被 judge 增量复审 D2 证伪**（六臂两两原始序列无一对相同，同 jar 重复跑亦然）⇒ **落盘正文已就地校正为「只用 multiset；序列不承载判据」**（见 record §4.7.9、errors E6）。**本稿正文不再代表落盘内容**，以落盘文件与上述锚点为准（原稿按 §15.4 不删不改）。

> **产出角色**：`core.worker` 知识库落盘 subagent。**本文件是草稿**——只给「可直接粘贴的成稿正文 + 插入锚点 + 取代指针建议」；主会话负责应用 + 一致性验证。**本 worker 未修改** `versions/1.20.1/docs/` 任何既有文件。
> **本版定位**：**覆盖** A 线旧稿 `.investigations/a1-opt-pool-260911-05/knowledge-draft-260911-05-docs.md`（其正文已按 record §4.5 的提交号纠正重录，见 §1.4/§2.4）**并合入**本轮 Wave 2（workspace 块 260912-01）新增。旧稿应用后即可废弃；**勿两份都应用**（同一 10 块 / 07 小节会重复）。
> **源材料**：`record-260912-01.md` **§4.6 / §4.7（含 §4.7.2-§4.7.8：三态表 / judge 复算后的门数字 / 1.21.6 回填 + 确认缺陷 / 证据落盘 / 一手锚补正 / judge 条件响应 / 未闭合项）**（Wave 2 唯一素材主源，本稿只读这两节 + §4.1/§4.2/§2.1-§2.6 作背景）+ 本波 judge `review-wave2-260912-01.md`（PASS-with-conditions / C1–C9）+ 错误台账 `errors-260912-01.md`（E1–E5 + 速查表）+ 证据目录 `evidence/`（42 文件 + `MANIFEST.txt`）+ 计划 `.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（§14 补登）+ A 线三件（`record-260911-05.md` / `a2-dim-gate-260911-05.md` / `review-260911-05.md`，经旧稿转录）。两项过程实测的一手锚（worktree 换行伪差异 / javap zip 对拍级联误报）**已由 record §4.7.6 补齐**（`evidence/prewt-pseudodiff.txt`；`evidence/src-pre-1.20.1-CppBridge.java` 与现行源码逐字对照），另见 `errors-260912-01.md` E2/E3。
> **本版修订记录**：2026-09-12 依 judge `review-wave2-260912-01.md` 的 **C1–C9** 与回填后的 record 正文**原地修订**（三态 jar sha / 异常行 4-12 / V3 覆盖面边界 / 1.21.6 回填与确认缺陷 / 证据落盘 / 一手锚改引 / judge 状态）；结构与编号未变，无任何条目升格。
> **状态纪律**：A 线部分 = **candidate**（judge PASS-with-conditions，条件已应用；confirmed 留人类）；**Wave 2 部分 = draft**（judge 已做 = PASS-with-conditions C1–C9 已响应、record §6 已回填、用户未 confirmed、1.21.6 强制 bulk 缺陷根因未定位 ⇒ 不得写成 candidate/confirmed）。
> **锚点实测（本稿实读，2026-09-12）**：`versions/1.20.1/docs/10-timewise-archive.md` **total = 3154 行**（末行 3154 = 260911-06 块的 open 行）；`versions/1.20.1/docs/07-block-pipeline.md` **total = 1289 行**（末行 1289 = C 线小节「遗留 / 未覆盖」长行）⇒ **旧稿锚点仍有效（未被后续改动推走）**。⚠️ **行号会漂移，应用时一律以「锚点文本」grep 定位为准**。

---

## 0. 摘要（应用清单）

| # | 载体 | 追加块 / 小节 | 状态 | 追加顺序 |
|---|---|---|---|---|
| A-1 | `versions/1.20.1/docs/10-timewise-archive.md` | `## 260911-05（A 线：1.20.1 优化池 + nether/end 维度门…）` | candidate | 第 1 个 |
| A-2 | 同上 | `## 260912-01（D3 共享 Java 适配核抽取：Wave 1 纯移动 + Wave 2 语义统一…）` | **draft**（V1b PASS-with-declarations（基线 `post2`）；1.20.1 V2+V3 双 PASS + 1.21.6 默认臂 PASS；judge PASS-with-conditions C1–C9 已响应；强制 bulk 臂确认缺陷（阻断 D-4(i)）；用户未 confirmed） | 第 2 个（A-1 之后） |
| B-1 | `versions/1.20.1/docs/07-block-pipeline.md` | `## 2026-09-11 A 线：Java 侧写回优化池…` | candidate | 第 1 个 |
| B-2 | 同上 | `## 2026-09-12 D3 共享 Java 适配核（Wave 2 语义统一，commit 997d40f）` | **draft**（同 A-2） | 第 2 个（B-1 之后） |
| S-* | 取代指针（§15.4，原句不删不改） | 见 §3（A 线 S-1~S-4 + Wave 2 S-5/S-6） | 建议应用 | 就地追加 |

**两个插入锚点（速览）**
1. `10-timewise-archive.md` → **文件末尾**，末行 `3154: - 🔍 **open（未核/降级/边界）**：① R9-b 并发可见性专项未做；…⑨ **块号**：与同日 E5 复算块（19:37-20:52）同用 `260911-05` 标签，本行按 `-06` 记、目录名不改（见 `knowledge-update-draft-260911-05.md` §3）。` 之后追加（格式：空行 + `---` + 空行 + `##` 标题）。
2. `07-block-pipeline.md` → **文件末尾**，末行 `1289: - **遗留 / 未覆盖**：R9-b 并发可见性专项未做；…共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）。` 之后追加（格式：空行 + `##` 标题）。A1d 取代指针可选打在 `07:601`（见 §2.4）。

---

## 1.【成稿 A】`versions\1.20.1\docs\10-timewise-archive.md` 追加块（两个，按序追加在 EOF）

### 1.1 插入锚点（精确）

- **位置**：文件**末尾**（当前 total = 3154 行）。
- **顺序**：先 A-1（260911-05 A 线），再 A-2（260912-01 Wave 2）——两块的**实际时点**都晚于文件现有末块 260911-06（20:56–22:3x）中的 A 线部分、早于/晚于 C 线部分不等，故按本文件既有「**追记**」实践一律追加在 EOF，并在块首声明时序（A-1 的时序声明旧稿已写；A-2 无需声明）。
- **插入模板**（与既有块一致）：`3154` 行之后 → 空行 → `---` → 空行 → `## 260911-05（…）` → （A-1 全文）→ 空行 → `---` → 空行 → `## 260912-01（…）` → （A-2 全文）。

### 1.2 可直接粘贴正文 A-1（A 线 260911-05；整段复制，从 `##` 行起）

> **相对旧稿的唯一改动**：提交号归属按 `record-260912-01.md` §4.5 纠正——**A1b 实现与 A1a 同在 `b2b2f26`**（`6b90998` 实为 docs 提交 = plan + record）；完整链 = `b53b23f`(A1d) / `b2b2f26`(A1a+A1b) / `6b90998`(docs) / `8d8075a`(A2) / `e8decef`(judge C1-C12)。旧稿 §5-1「`6b90998` 零命中 ⇒ 存疑」**据此解除**。

## 260911-05（A 线：1.20.1 优化池 + nether/end 维度门；实际 2026-09-11 19:2x–20:5x）🔍 candidate（judge PASS-with-conditions，C1-C12 + I1-I4 已应用；confirmed 留人类）（追记于 260911-06 块之后；目录标签与同日 B1/C 块同用 -05）

> 过程产物 `.investigations/a1-opt-pool-260911-05/`（`record-260911-05.md`：A1a/A1b/A1c/A1d + 行为门 + §9.7 降级声明 / `a2-dim-gate-260911-05.md`：nether/end 全维行为门 / `review-260911-05.md`：judge 原文 + §7 C1-C12 处置表）；数据载体 `.tmp/a1-260911-05/`、`.tmp/a2-260911-05/`（驱动 + 比对脚本，临时区不入库）+ 日志 `.tmp/vivo-stutter-260911-02/ab-threads/logs/`、`.tmp/a2-260911-05/logs/`；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`，均 candidate）；提交 A1d `b53b23f` / **A1a+A1b `b2b2f26`（同提交）** / docs `6b90998` / A2 `8d8075a` / judge 修正 `e8decef`。
> ⚠️ **与 C 线的关系（勿写成「已消失」）**：C 线（260911-06）把默认写回改走 bulk section 路径后，A1a/A1b 所改的**旧逐块路径仍在库**——回退开关 `-Dcoreswap.bulkwb=0` 走旧路径，A1a 的跳空气写回与 A1b 的诊断门控即位于该路径（见 07 篇 C 线小节「回退开关 `-Dcoreswap.bulkwb=0`（旧逐块路径保留不删…）」）。

- ✅ **A1a 写回跳空气（`CppBridge.writeChunk` 对 raw id 0 跳过；回退 `-Dcoreswap.skipair=0`）**：**改了什么** = `writeChunk` 遇 raw id 0 不再逐格写回；开关为静态 final ⇒ 常量折叠（生产路径实际新增每格一次 `id==0` 判断，judge I4 修正措辞）。**判据（预登记）** = 跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；判据谓词经 judge C5 严格化为 **`!isOf(Blocks.AIR)`**——`isAir()` 会漏计持 `cave_air`(730)/`void_air`(729) 的格子，而跳过的写是「写 `minecraft:air` 默认态」。**vanilla 一手源依据（judge 补引）** = `ChunkSection.setBlockState`（1.20.1）只在 `isAir` 变化时增减 `nonEmptyBlockCount`、`PalettedContainer.swap` 同值时写回同一 palette index ⇒ 对空气格写 air 是语义 no-op。**实测（最终构建 + 严格谓词，三维全格计数）** = overworld **276,171,456** / nether **194,630,604** / end **154,752,709** 个空气格被跳过，`stale_nonair` **三维均 0**（judge C4 由「机制外推」补跑为三维实证：wb-ow/wb-n/wb-e）；机制依据 = 三维 mixin 均 `populateNoise` HEAD cancel（`NoiseChunkGeneratorMixin` 279/333/364）⇒ 目标 chunk 全新。**行为门** = 跨臂 + 跨 dll + 跨 session 归档臂逐 chunk 指纹全等（4140/4140，`hash_diff=0`）。**证据路径** = `record-260911-05.md` §2-§4 + 日志 `[WB-CHECK]` 行。**commit `b2b2f26`**。
- ✅ **A1a 收益不主张（Degraded）**：cpuSec **493→476（−3.4%）**——**落在 ±10% 机器噪声带内、单对跨 run ⇒ 只作趋势**；且两臂均开 `WBCHECK`（每空气格一次 `getBlockState` = **2.76 亿次**）⇒ 该数应读作**含诊断成本的收益下界**，要引用定量数字须关 WBCHECK 重跑（judge C6）。结论性质 = 「等价性成立 + 无回归」，非「性能提升 N%」。
- ✅ **A1b 诊断扫描门控（`CppBridge.fillChunk`）**：**改了什么** = `nzBuf` 全 buffer 扫描（**98,304 读/chunk**）与 nether/end 16 点读回**整块门控到 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。**判据** = 纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。**取舍声明（judge C2）** = 改前 `buf-all-air`/`buf-sparse` 是**无条件 println**（原记录「只门控 println」的理由句错误，已改正；类 javadoc 旧措辞作废，C10），`buf-sparse` 分级诊断随全量扫描一并移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。**证据路径** = `record-260911-05.md` §3 A1b + `CppBridge.java` javadoc。**commit `b2b2f26`（与 A1a 同提交）**。
- ❌ **A1c 6 高度图「全量重扫」——前提被一手源证伪，用户裁决放弃（已评估·不实施）**：一手源 `Heightmap.java:37-71` = vanilla `populateHeightmaps` **本就是单遍**（每列 (x,z) 只做**一次**下行扫描，6 个类型的谓词在**同一次扫描内**逐个匹配并移除，非「6 张各扫一遍」）⇒ 优化池该项描述**不成立**；残余候选只剩「读 `buf`（raw id）替代 `chunk.getBlockState`」，而代价/风险 > 收益（该项 0.42ms/chunk = 0.4%；`Heightmap.set` private、storage final ⇒ 外部写入须反射或自实现位打包）。**裁决** = 用户 HOOK-2 放弃实施。**证据路径** = `record-260911-05.md` §3 A1c + `review-260911-05.md` §3（judge 对照 `Heightmap.java` 核实）。
- ✅ **A1d `adaptive_threads` count=1 clamp（Rust 共享层 `worldgen-core/src/api.rs`，清理项）**：**改了什么** = `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`（批量路径 count>1 语义**逐字未变**）。**机制自证（运行期）** = `[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll，两条：a1-skipair0/1 的 `.log.err`）。**MT3 的历史前提已消失**——当时是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 per-call `std::thread::scope`（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ clamp 安全（judge 独立核对）。**无性能归因**（双臂都含 clamp，未做隔离 A/B）。**证据路径** = `record-260911-05.md` §3 A1d + `review-260911-05.md` §3。**commit `b53b23f`**。
- ✅ **A2 nether/end 全维行为门（开工发现真缺口 → 补载体）**：**改了什么** = `CppBridge.fillChunkEnd` 的 MIXLOG 块内补 `[WG-CONTENT]` 指纹行——首轮 end 臂 `intercepted=4761` 而 `contentLines=0` ⇒ 全维行为门在 end 维**载体缺失**（判据意义：**「接管生效」与「行为门可判」是两件事**）。**判据** = 三维「跨形态 + 跨 run 逐 chunk 指纹 sorted diff = 0」。**实测** = **overworld 4140/4140、nether 4761/4761、end 4761/4761 逐 chunk 指纹差 0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；**正对照** overworld vs nether `hash_diff=4140/4140`（门有检测力）；**分母语义** = 载体条数 = **经接管的 chunk 数（≠ Chunky Processed 4225）**，门判据不依赖任何分母（judge C3）。**证据路径** = `a2-dim-gate-260911-05.md` §1-§2 + 日志 `[WG-CONTENT]` 行。**commit `8d8075a`**（judge C9 改正早前误引的不存在提交号 `3c1f9c6`）。
- 🔍 **A2 附带读数（非门判据，Degraded）**：nether async **25s/24s** vs sync **69s**（形态效应 ~2.8×，cpuSec 212/202 vs 149）；end async **11s/8s** vs sync **20s**（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致（单车道同步形态在 nether/end 都是净亏损）；但跨 run 摆动 **±27%** ⇒ **不宣布定量收益**。
- ✅ **judge（candidate 授予 SHOULD 触发，隔离子进程）**：A1 / A2 均 **PASS-with-conditions**、无 FAIL，**全部量化数字复现一致**；C1-C12 + I1-I4 **逐条已应用**（处置表 = `review-260911-05.md` §7）——重点：C1 可比性改为「a1 双臂同 dll `838e8979`；三归档臂为 A1d 前构建 `dd3b645f`」并补「跨 dll 指纹全等 = A1d 输出中立性独立证据」、C3 分母语义声明、C4 由外推补跑为**三维实证**、C5 判据谓词严格化、C7 比对脚本加最小载体阈值断言（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）、C8 下游归因降为**未测外推**、C9 提交号改正、C12「85 缺口」说法作废。judge **未改任何 status**；**本块结论维持 candidate，confirmed 留人类**（judge 修正 commit `e8decef`）。judge 盲区如实登记：旧二进制 clamp 前 `nthreads` 只能推导（无日志行）、`dd3b645f` 是否确为 A1d 前构建按时间线推断、f2 成本拆分（0.42ms/chunk）本块未复核。
- 🔍 **open（未核/降级/边界）**：① **写回后内容指纹（post-write hash）仍未做**——三维均未覆盖（260910-06 open ⑤ 延续；A1a 以「前提全格实证」替代）；② overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」说法已废，降为 open 假设）；③ nether run 级非确定的**成因域**只排除「Rust 填充层」（本载体只覆盖 `buf`），下游层未测（judge C8 未测候选：Java carver/feature/装饰层、写回路径、存档序列化）；④ A1d「改前 nthreads=10」为公式推导非实测；⑤ `buf-sparse` 分级诊断已移除；⑥ 本项改动**尚未随任何 release 出货**（1.0.29 不含 A1 改动，出单评估见计划 §2 A4）；⑦ dll 硬门禁只比日志里 **16 hex 前缀**（非全 sha256，judge I2）；⑧ **块号**：与同日 B1（E5 复算）/C 线 bulk 块同用 `260911-05` 标签，本块按 `-05` 记、目录名不改。

---

### 1.3 可直接粘贴正文 A-2（Wave 2 260912-01；整段复制，接在 A-1 之后）

## 260912-01（D3 共享 Java 适配核抽取：Wave 1 纯移动 + Wave 2 语义统一；实际 2026-09-12 14:0x–，日期锚 Get-Date）🔍 draft（Wave 1 V1-strict PASS；Wave 2 V1b = PASS-with-declarations（判定基线 = `post2`，三态表见 §4.6）；**V2/V3：1.20.1 双 PASS + 1.21.6 默认臂 PASS（§4.7.3 已回填）**；judge = **PASS-with-conditions**（`review-wave2-260912-01.md`；C1–C9 已全部响应/修正，见 record §4.7.7）/ record §6 已回填 / 用户未 confirmed；**新发现确认缺陷：1.21.6 强制 `-Dcoreswap.bulkwb=1` 臂崩解（130 chunk `EntryMissingException: Missing Palette entry for index 2…8`，根因未定位）⇒ 不阻断 Wave 2（默认关）、但阻断 D-4(i)**）

> 过程产物 `.investigations/shared-java-core-260912-01/`（`record-260912-01.md`：§1 开工前交接核验 / §2 pre 冻结 + 构建确定性控制 + V0 接线预检 / §4.1 Wave 1 / §4.2 HOOK-2 用户裁决 / §4.3 scout-judge 条件响应 / §4.6 V1b 判定 + **构建三态表** / §4.7.0-§4.7.8（V2+V3 / **dll 血统事故** / 1.20.1 双 PASS / **1.21.6 回填 + 确认缺陷** / dll 归一化 / **证据落盘** / **两个过程发现的一手锚** / judge 条件响应 / 未闭合项）/ `scout-map.md` / `review-scout-260912-01.md`（scout-judge PASS-with-conditions：抽核 21 锚点 / 14 文件，✘0 / ⚠1，闭包 4 路证伪未遂）/ **`review-wave2-260912-01.md`**（本波 judge，verdict = PASS-with-conditions，条件 C1–C9 见 record §4.7.7）/ **`errors-260912-01.md`**（错误台账，五段式 E1 dll 血统 / E2 worktree CRLF / E3 javap 三陷阱 / E4 门数字未复算 / E5 1.21.6 强制 bulk 崩解 + 速查表）/ **`evidence/`**（**42 文件 + `MANIFEST.txt`**，含清单 / 差异输出 / 门控原样行 / 完整日志 / 复现工具；tracked、未被 gitignore））+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（§14 追加式补登，原正文不改）；工具 `.investigations/shared-java-core-260912-01/evidence/{tool-jar_manifest.py,tool-jar_manifest_diff.py,tool-fp_compare.py,tool-javap_method_diff.py}`；提交 `ce5286b`（Wave 1 纯移动）/ `997d40f`（Wave 2 语义统一）/ 后续 record+evidence+drafts 落盘 commit（judge C9）。通用模式 → knowledge/discovered：workflow-patterns **#138** + **#14 补充案例（260912-01）**、build-tooling **#59（与 A 线首例合并）/ #60 / #61**、compiler-idioms **#25**（subagent 草稿 → 主会话应用）。

- ✅ **Wave 0/1（机制接线 + 纯移动，V1-strict PASS）**：新建共享源目录 `java-core/src/main/java`（单一源），两版 `build.gradle` 各加 `sourceSets { main { java { srcDir '../../../java-core/src/main/java' } } }`（各 +11 行）。**V0 只接线（空 srcDir，仅 `.gitkeep` + README）**⇒ 两版 jar sha **逐字节不变**（`1027f4f6…` / `16d5e5e7…`；条目 1081/1081、1798/1798，差异 0、增删 0）⇒ **接线机制对 loom / mixin AP 惰性**（未证明 = 非空共享源时的行为，那才是 Phase 2 判定对象）。**Wave 1** 把 3 个**逐字节相同**的类（`wg/CppWorldgen.java`、`wg/bench/WgDiag.java`、`wg/bench/BenchMod.java`，两版源 sha 实测相同）`git mv` 进共享源、`git rm` 1.21.6 副本（「一个类只有一个家」）⇒ 两版 jar sha **仍逐字节不变** ⇒ **非空共享源下「移动 ⇒ 产物字节相同」成立**（提交 `ce5286b`）。
- ✅ **构建确定性控制（V1 判据有效性的前提）**：1.20.1 同源连跑两次（`--rerun-tasks`）jar sha 完全相同；1.21.6 现建 jar 与知识库 260910-07 迁移记录（KB #116）里的 `16d5e5e7…` **逐字节相同** ⇒ ① 构建确定（跨 2 天、跨调用）② 顺带证明 1.21.6 源码自 260910-07 起未变。⇒ **「抽取后条目 sha 变化」只能来自抽取本身，不来自构建抖动**。
- ✅ **V0 接线预检的价值边界（写死）**：证明了 ① 新增空 srcDir 不扰动 refmap / 条目集 / jar 字节（两版构建仍绿 `exit=0`）② V1 工具链在本载体端到端可用；**未证明**非空共享源的行为（单变量对照 = 仅添加空 srcDir）。
- ✅ **Wave 2（语义统一）交付面**：共享 `CppBridge`（超集合并：并入 1.21.6 独有项 + `WgCompat` 引用）、共享 `BulkWb`/`StallWatch`/`ChunkTiming`（超集）/`CoreSwapFixHelper`；分版缝类 `WgCompat` ×2（`BULKWB_ON`/`SKIPAIR_ON`：1.20.1 = true、1.21.6 = **false**；`WgCompat.flag(prop, def)`：设了 property 则「非 0 即真」⇒ 1.21.6 将来可 `-Dcoreswap.bulkwb=1` 强制开启而无需改码，翻转 = 改一个常量）；1.21.6 补 `mixin/ChunkSectionAccessor` + `coreswap.mixins.json` 24→25 条 + refmap 重建（HOOK-2 批准的 **S-1**；字段名两版同名 + intermediary 名两版逐字相同 ⇒ 静态可行，默认不被调用 ⇒ 无行为变更）。**HOOK-2 其余裁决**：S-2/S-3 默认常量（上）、S-5 StallWatch 接线进共享 `init`（property 门控、两版默认关）、S-6 **接受** 1.21.6 生产热路径去掉每 chunk 无条件全量扫描 + println（已批准的行为变更，由 V2/V3 门覆盖）、S-7 **接受** ChunkTiming 取超集（1.20.1 开 `[CHUNKTIME]` 时多打印恒 0 列）。
- **冻结 pre（同一主工作树构建，§2.1）= 1.20.1 `1027f4f6…`（1081 条目 / 57 类）、1.21.6 `16d5e5e7…`（1798 / 54）**。**Wave 2 构建三态（§4.6 表头，judge C6 修正：V1b 判定基线 = `post2`，不是 post1）**：

  | 态 | 构建内容 | 1.20.1 jar sha | 1.21.6 jar sha | 相对上一态的隔离差异（实测） |
  |---|---|---|---|---|
  | post1 | worker 首建（**含头注释**） | `41f4a551…` | `2be87e40…` | — |
  | **post2（V1b 判定基线）** | 删 `StallWatch.java`(1.20.1)/`CoreSwapFixHelper.java`(1.21.6) 头注释后重编 | **`0681ec03…`**（1082 / 58） | **`772d7a6e…`**（1803 / 59） | 恰 2 条：`CoreSwapFixHelper`、`StallWatch`（= 头注释致 `LineNumberTable` 位移，`post1-vs-post2`） |
  | post3（权威 dll） | `target/release/worldgen.*` 归一后重编 | `461baedc…` | `772d7a6e…`（同 post2） | 恰 1 条：`native/worldgen.dll` |
  | post4（C4 注释修补后） | `ChunkTiming.java` javadoc **逐行替换（总行数不变）** | `461baedc…`（≡ post3） | `772d7a6e…` | **0 条**（`evidence/post3-vs-post4-*.txt`：相同 1082、差异 0 / 非预期 0） |
- ⇒ **1.20.1 对冻结基线的 V1b 差异集 = 6 个变动 Java 条目 + 1 个新增 Java 条目**（`evidence/v1b-1.20.1.txt`：相同 1075 / 差异 6 / 新增 1 / 删除 0）；1.21.6 = 5 差异 + 5 新增（`v1b-1.21.6.txt`：相同 1793 / 差异 5 / 新增 5）；**dll 条目只在 post3 比对时出现 1 条**（构建输入、非 Java 面，§4.7.4）。两版 `gradle :build --offline --rerun-tasks` 均 **exit 0**（无 `error:`、无 `Cannot find target method` —— KB #25/#40 前兆判据；mixin AP 接受 1.21.6 的 `@Mutable` accessor = **由产物证实**（`javap -v -p` 显示 `ChunkSectionAccessor` 带 accessor ×7 + `@Mutable`），judge 指出 record 原引用的 build 日志中 `Mutable` 命中 0 ⇒ 证据指针不准、非结论错）。
- **V1b 判定 = PASS-with-declarations**（差异条目 **100% 逐条声明**；4 条经 `javap -c -p` 证明「指令完全相同、仅调试属性」；1 条为逐值等价的常量替换；其余为已批准超集/新增）。**V6 共享类布局检查** = `class_home_check.py` 交叠类**空** + 共享源内 mixin 包类**空** + 共享类 8 个 ⇒ PASS。
- **V1b 逐条声明（1.20.1：6 差异 + 1 新增）**：① `wg/bench/BulkWb.class` = **仅 `<clinit>` 的 `ON` 求值序列**（`!"0".equals(getProperty("coreswap.bulkwb"))` → `WgCompat.flag(prop, BULKWB_ON)`）；javap 逐指令对拍除该处外逐条相同（仅偏移位移），语义逐值等价（null→true、`0`→false、`1`/其它→true）；② `wg/bench/BulkWb$TL.class` = 仅调试属性（LineNumberTable；`javap -c -p` 输出**完全相同**）；③ `wg/bench/ChunkTiming.class` = 取超集（+7 `LongAdder`、+7 `addXxx`、+`featTick`；report 取 1.21.6 形态；计划 §4.2 已批准 **S-7**；javap 指令差异 697 行）；④ `wg/bench/CoreSwapFixHelper.class` = 仅调试属性（注释 3→6 行 ⇒ 行号位移；`javap -c -p` 输出**完全相同**）；⑤ `wg/bench/CppBridge.class` = 超集合并（并入 1.21.6 独有项 + `WgCompat` 引用）；⑥ `wg/bench/CppBridge$1.class` = 仅调试属性（匿名类行号随合并位移；`javap -c -p` 输出**完全相同**）；＋`wg/bench/WgCompat.class` = 新增（分版缝类，计划 §4.2）；另 `wg/bench/StallWatch.class` = **无变化（字节全等）**——逐字复制后条目 sha 全等；**曾因加 1 行头注释导致行号位移 ⇒ 删注释恢复字节锚**（「头注释纪律」的来源，→ KB compiler-idioms #25）。
- **方法级对拍（1.20.1 `CppBridge`，偏移 / 常量池序号 / `ldc_w` 宽度归一）**：32→33 方法，`same=28 / changed=4 / added=1`——`stateById`（内容路径）**源码逐字相同**（pre `:629-638` ≡ post `:725-734`）⇒ 该路径未变，字节差异仅常量池序号/顺序（归一后仍报 6 行 = **序号伪差**，已用源码逐字对照排除）；`writeChunk`（内容路径分派）唯一源码差异 = ① `BulkWb.ON` → `BULKWB`（1.20.1 缺省取值相同）② `populateHeightmaps` 前后 2 行计时钩子；`writeChunkPerBlock`（逐格写回本体）**不在差异集内** ⇒ 指令级不变（A1a 跳空气/WBCHECK 自检原样）；`fillChunk` 差异 = 计时钩子 + A1b 门控（1.20.1 侧 A1b 形态与 pre 相同）；＋`rustFeaturesTakeover` 新增（1.21.6 独有 mixin 的编译前置，1.20.1 无消费者 ⇒ 零行为影响）。⚠️ **`lambda$static$0` 不构成证据**：javap 的 lambda 名按**序号**命名，pre/post 的 `lambda$static$0` 可能对应不同 lambda（诚实声明；BUF/BUF_NETHER/BUF_END 声明已源码对照一致：`16*16*384`/`256`/`128`）。⚠️ **初版对拍曾报 4 处伪「语义变更」**（`stateById` = 「30/30 指令、6 行不同」、`lambda$static$0` = 「98→97、46 行不同」）——源码逐字对照（`evidence/src-pre-1.20.1-CppBridge.java` ↔ 现行 `java-core/.../CppBridge.java`）证明完全相同；归因 = **javap 三陷阱**（lambda 按序号命名 / 按行 zip 在指令数变化处级联误报 / 常量池序号与 `ldc`↔`ldc_w` 宽度未归一），一手锚 = record **§4.7.6** + `errors-260912-01.md` **E3**；judge 独立复算改用「`javap -c -p -constants` 文本逐字 + `-v -p` 属性分类」后确认 **4/4 条「仅调试属性」**成立。
- **V1b 逐条声明（1.21.6：5 差异 + 5 新增）**：① `coreswap.mixins.json` 24→25 条（+`ChunkSectionAccessor`；`ConvertFrom-Json` 复验合法）；② `coreswap1216-refmap.json` +`ChunkSectionAccessor` 段（mixin AP 重建，S-1）；③ `wg/bench/ChunkTiming.class` = **仅调试属性**（javadoc 行数变化；`javap -c -p` 输出**完全相同** ⇒ 并集未改 1.21.6 计时代码）；④ `wg/bench/CppBridge$1.class` = 仅调试属性（输出完全相同）；⑤ `wg/bench/CppBridge.class` = 超集合并（并入 A1a/A1b/A2/StallWatch 接线 + bulk 分派结构；含已批准 S-5/S-6 行为变更）——方法级：`writeChunk` 133→40（改为分派）+ `writeChunkPerBlock` 新增（取 1.20.1 版 128 指令）、`destroy` 14→43、`init` +1；＋`BulkWb`/`BulkWb$TL`/`StallWatch`/`WgCompat`/`mixin/ChunkSectionAccessor`（5 条）。
- ⚠️ **残留影响声明（诚实清单，3 条）**：① **1.20.1 生产路径新增 4 次 `System.nanoTime()` 求值/chunk**（`addJni/addScan/addWrite/addHmap` 的实参）——`ChunkTiming.addXxx` 内部 `if (ON)` 已门控 adder 写入，故未开 `-Dcoreswap.chunktime` 时唯一代价是这 4 次调用（**≈80ns/chunk 为算术估计、未实测**，相对 chunk 墙钟数百 ms 可忽略）；**保留 1.21.6 逐字形态，不做微优化**（不把未请求的优化混进等价波次）；② **1.21.6 `writeChunk` 写回实现被替换为 1.20.1 的 `writeChunkPerBlock`**（**结构变化、非仅门控**）⇒ 1.21.6 侧内容等价**不能靠指令级证明**；**该版写回替换的内容等价性目前仍无证据**（judge N-未证伪项：其 pre 无 `BulkWb` / 无读回门 ⇒ 无同层 pre 对照）——已具备的正向证据仅「§4.7.3 臂 A 默认臂干净（`[WG-CONTENT]` 625 + `[WG-CONTENT-WB]` 625、非 WMI 真实异常 0）+ 采用 1.20.1 已验证写回本体（指令级）+ `SKIPAIR=false` 保持「逐格写含空气」旧行为」，**不得表述为「已等价」**，留待 D-4(i) 自身 A/B（且须先修臂 B 缺陷）；③ S-6/S-7 的两版诊断面差异已按批准生效（1.21.6 去掉无条件扫描 + println；1.20.1 `[CHUNKTIME]` 多打印恒 0 列）。
- ⚠️ **❗ dll 血统事故（本波最重要的过程教训；V3 判定曾作废一次）**：**现象** = V2/V3 首轮两臂**实际执行的 dll 不同**（post `838e8979…` vs pre `dd3b645f…`）⇒ 对照被引擎差异污染、**V3 判定作废**（靠逐臂读取 `<CppBridge> dll= sha256=` **自证行**发现，非事后猜测）。**根因链** = ① `target/release/worldgen.dll` 当时是**早前 A/B 实验留在 target 的非权威产物**（`838e8979…`，A1d 后构建）；② A/B 驱动 `run_ab.ps1` 收尾**无条件** `Copy-Item $bak $targetDll`，把 `.tmp/.../worldgen-target.bak-597e12ed`（= **1.0.28 引擎 `597e12ed`**）写回 target；③ worktree 内 `gradle :build` 触发 dll 同步链，target 又被恢复为**权威 `dd3b645f`**（= 1.0.29 票记录值）⇒ **两臂各读一个 dll**。**处置** = 权威 dll 另存 `.tmp/.../w2/dll-canonical-1.20.1.dll`；**覆盖 harness 备份**（原备份另存 `…bak-597e12ed.historical-597e12ed`）使收尾 restore 退化为 **no-op**；两臂重跑。**三元组核验（事后）** = 1.0.29 票 dll `dd3b645f…` ≡ 当前 `target/release/worldgen.dll` ≡ 重编 jar 内 `native/worldgen.dll`；1.21.6 = `abd7d889…` 三处一致。**附带发现** = `gradle :build` 的 dll 同步 `doFirst` 会把 target 拉回权威产物（本次把非权威 `838e8979` 纠正为权威 `dd3b645f`）——构建链对 dll 有**自愈**作用，但**不能依赖**（若权威 dll 不存在或同步被 `UP-TO-DATE` 跳过则不自愈，KB #56/#96 家族）。**判据升级（可复用）** = **跨臂 / 跨 run 对照前 MUST 逐臂读「执行体自证行」核对，不得只看 target 的文件 sha**——target 会被「A/B 收尾 restore」与「实验暂存」改写 → KB build-tooling **#59**（与 A 线首例合并为同一条，本事故为**第二实例**，污染对象从「交付产物」升级为**对照实验本身**）；完整五段式台账 = `errors-260912-01.md` **E1**，judge 复算一致（`review-wave2-260912-01.md` §5.1）。
- ✅ **V2（run 回归）+ V3（行为门）——1.20.1 出货线双 PASS（judge C2/C3/C7/C8 修正后）**：**三臂同配方**（唯一变量 = Java 源码状态）：seed `417950215108767439`、`chunky world overworld` + `center -48 -11` + `radius 160`、`-PcppReplace=true`、`-Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1`、**同一 dll**；臂 post = 主工作树（HEAD `997d40f`）、臂 pre = `git worktree @ ce5286b`（`.tmp/w2/pre-wt`，其 `runtime` 用 **junction** 指回主仓 `runtime/` ⇒ 共享世界/配置，**pre 臂跑 2 次**）。结果：boot **55.8s / 54.2s、51.2s**；dll 自证门**逐臂读自证行**三臂 `dd3b645f` **OK**；生成 **441 chunks / chunky 自报 post 与 pre 臂均 5s（= 88.2 chunks/s；脚本打印的 10s 是轮询粒度、非 chunky 口径 —— C8）**；异常行（`Exception|Error:`）**实测 post 4 行（全部 WMI/COM 良性）/ pre 12 行（8 行 WMI 良性 + 4 条与 CoreSwap 无关：dev 测试 mod `testcontent` entrypoint 失败 `This registry can't create intrusive holders`）**——原记录的「0 / 0」已作废（门数字未按自身证据复算，`errors-260912-01.md` **E4**；真臂 mod 集合两臂一致 = 48 ⇒ 不影响对照结论）；`[WG-CONTENT]`（Rust buf 层）**607 / 607 / 607**；`[WG-CONTENT-WB]`（Java 写回读回层）**607 / 607 / 607**；**三臂两两比对（post vs pre-r1、post vs pre-r2、pre-r1 vs pre-r2）两族指纹全部 multiset 与 sorted-sequence 全等（`only-pre=0 / only-post=0`）** ⇒ ① 引擎层输出未变 ② **Java 写回结果逐 chunk 全等**；同实现跨 run 在**本层无噪声**（pre-r1 vs pre-r2 全等已证）。工具 `evidence/tool-fp_compare.py`（多重集 + 序列双判；含「同集合不同顺序」可辨）。⚠️ **覆盖面边界（judge C3：原文「逐格写回本体未变」= 过度声称，已作废）**：三臂 JVM 均未设 `-Dcoreswap.bulkwb` ⇒ 1.20.1 缺省 `BULKWB_ON=true` ⇒ 实际走 `BulkWb.writeSections` ⇒ 本门覆盖的是 **bulk 写回路径（= 1.20.1 生产路径）**；**`writeChunkPerBlock`（逐格写回 / 回退路径）在三臂均从未执行**，其「未变」只有 §4.6 指令级证据、**无运行期证据**。⚠️ **维度盲区（C7）**：本次只生成 overworld；被改的 `writeChunk` 同时服务 nether/end 调用点（`CppBridge.java:496/:525/:576` 邻域）⇒ **nether/end 零覆盖**（`[WG-CONTENT-NETHER/END]` 命中 0），依赖既有 C 线结论与后续 A2 门、不在本波证据面内。**§9.7 口径声明（record §4.7.2）**：载体 = live dev-server + chunky（overworld，center `-48 -11`，r160，441 chunks，seed `417950215108767439`，dll `dd3b645f`）；覆盖面 = 本次写回的 441 chunk / **607 个 chunk 的两层指纹**（Rust buf / Java 读回），**仅 overworld、仅 bulk 写回路径**；可比性 = 三臂同 dll / 同 seed / 同坐标 / 同门控，**唯一变量 = Java 源码状态**（pre 臂另有 `testcontent` 环境噪声，已声明——两臂 mod 环境非逐字节同）⇒ 本判据为**逐 chunk 精确等**，与既有 C 线「同实现两 run region 层噪声 0.024%」**不同层、不可混用**。
- 🔍 **V1b 基线的 dll 归一化说明**：V1b 的 **Java 面判定**用 `post2`（内含旧 dll `838e8979`）与冻结 pre（**同 dll**）比对 ⇒ dll 条目两侧相同，**不干扰 Java 面结论**；归一化影响已隔离验证：`post3`（权威 dll）vs `post2` = 1.20.1 **仅 1 条目差异**（`native/worldgen.dll`）、1.21.6 **0 条目差异** ⇒ dll 归一化对 class 条目**零影响**（V1b 声明不受影响）。**交付提醒**：`build/libs` 现产物 = 权威 dll 版本，但**与已发布 1.0.29 的 jar sha `b057fda2…` 不同**（发布 jar 早前被本地构建就地重写覆盖，§2.6「构建产物目录不是存档目录」）⇒ 任何 1.20.1 再发布 MUST 重跑全量回归 + 三元组重算。**C4 复算（注释修补不改字节）**：`ChunkTiming.java` javadoc 改动保持**总行数不变**（逐行替换）⇒ 其后的代码行号不变 ⇒ `LineNumberTable` 不变 ⇒ `.class` 字节不变；实测 `post4` vs `post3` = **差异 0 / 非预期 0**（相同 1082），两版 jar sha 相同（`461baedc…` / `772d7a6e…`）。
- 🔍 **open（未核/降级/边界）**：① **§4.7.3 已回填：1.21.6 默认臂（生产语义 `BULKWB_ON=false`）PASS**——boot **42.1s**；bridge init `seed=417950215108767439 worldgenDir=versions\1.21.6\data\worldgen enabled=true stageMask=3` 全对；dll 自证 `abd7d889…` ≡ 该版权威（票/target/jar 三处一致）；`[WG-CONTENT]` **625** + `[WG-CONTENT-WB]` **625**（**该版首次带上读回门**）、`[WG-BULKWB]` 0（缺省关，符合预期）；非 WMI 真实异常 **0**；jar 内 `coreswap.mixins.json` 25 条含 `ChunkSectionAccessor`、启动无 `Mixin apply failed` / `Cannot find target method`。**但强制 `-Dcoreswap.bulkwb=1` 臂崩解 = 确认缺陷**：130 个 chunk 写回**全部抛异常**（日志 132 次）`net.minecraft.world.chunk.EntryMissingException: Missing Palette entry for index 2…8`，栈 = `ArrayPalette.get` ← `PalettedContainer.get` ← `ChunkSection.getBlockState`（bulk 路径里的**读**操作），`[WG-CONTENT-WB]` **0 条**、流水线未完成；**根因未定位**（候选：1.21.6 palette/并发语义 ≠ 1.20.1 / `ChunkSectionAccessor` 取字段形态差异 / section 构造路径变化）⇒ **不阻断 Wave 2（默认关）、阻断 D-4(i)**（完整五段式 = `errors-260912-01.md` **E5**、未闭合项 = record §4.7.8）；② **judge 已做**（`review-wave2-260912-01.md` = **PASS-with-conditions**，C1–C9 已全部响应/修正，record §4.7.7）——但 **record §6 仍写「待回填」**、status 由人类裁决；③ **用户未 confirmed**；④ worktree 作**条目级基线**的换行物化伪差异（`worldgen-data/**` **1024 条**差异、**class 条目 0**）**已补一手锚** = record **§4.7.6** + `evidence/prewt-pseudodiff.txt`（→ KB build-tooling #60）；⑤ javap「按行 zip 对拍」级联误报**已补一手锚** = record **§4.7.6** + `evidence/src-pre-1.20.1-CppBridge.java` 与现行源码逐字对照（→ KB build-tooling #61 / `errors` E3）；⑥ **1.21.6 写回替换的内容等价性无证据**（该版 pre 无对照载体）⇒ **不得表述为「已等价」**；⑦ **`writeChunkPerBlock` 无运行期证据**（1.20.1 回退路径；低成本闭合方式 = 跑 `-Dcoreswap.bulkwb=0` 臂与生产臂对比 `[WG-CONTENT-WB]`，未做）；⑧ **nether/end 零覆盖**（本波被改的 `writeChunk` 共享于三维度，而本门仅 overworld）；⑨ 1.20.1 class 条目仅 4 条做 `javap -c -p` 全等证明 + 一次方法级对拍（judge 抽样复核 5/5 成立），**未做全量逐条 javap 对拍**；⑩ 4 次 `nanoTime/chunk`（≈80ns）为算术估计未实测；⑪ **门数字纪律**：本波曾把异常行写成「0 / 0」（未测量即断言）被 judge C2 复算推翻（实测 4/12）⇒ 门数字 MUST 复算 + 附可复现命令 + 原始输出落盘 + 定义口径（`errors-260912-01.md` E4）；⑫ `testcontent` 环境噪声（dev 测试 mod、与 CoreSwap 无关）使两臂 mod 环境非逐字节同 ⇒ 严格单变量需摘除后重跑（未做）；⑬ **块号/日期**：本块 260912-01 与计划目录标签一致（`Get-Date` 实取 2026-09-12 14:0x）。
- ❌ **排除清单（一行，防重走弯路）**：① 「jar 字节变了 ⇒ 行为一定变了」对**纯移动波次**不成立——Wave 1 字节全等 ⇒ **V2/V3 被蕴含**（同一执行体 ⇒ 行为必然相同），该判据已写进 record §4.1 **防后波次偷懒或重复劳动**；② 计划原表述「V1 对全范围统一」**不成立**（plan §14.1 追加式修正：纯移动 vs 语义统一在产物字节上性质不同）；③ 计划 §1 目标 1 / §2 表 / §3 D-3.1 把「exec 模式 / P1 信号量 / maxinflight」列为 1.20.1 超前项并暗示进共享核 —— **错**（这些住在 `mixin/NoiseChunkGeneratorMixin`，属**范围 C**；`CppBridge` 全文对 `exec|Semaphore|maxinflight|Executor` **零命中**）；`Identifier.of`/`getOrThrow` 改名同样属范围 B/C、**不在范围 A**（plan §14.3）；④ 计划 §6.1 预期差异清单**漏项**已按 scout-judge C2 补齐（补 `CppBridge$1.class`；补声明 1.20.1 侧 `BulkWb`/`BulkWb$TL` 因 `ON` 改走 `WgCompat.flag` 而字节变）；⑤ **「异常行 0 / 0」= 「未测量即断言」**（脚本没打印异常计数 ⇒ 当作 0），judge 复算实测 **4 / 12** ⇒ 该写法已作废（`errors-260912-01.md` E4）；⑥ **「默认关 ⇒ 该路径不存在」不成立**——共享化把 1.21.6 上**原本不存在**的 bulk 路径变成**可达**（该版 pre 无 `BulkWb`、property 被忽略），强制臂一开即崩（130 chunk `EntryMissingException`）⇒ **默认关不豁免冒烟**；且被改的 `writeChunk` 共享于三维度 ⇒ 冒烟须覆盖 nether/end（`errors-260912-01.md` E5）。

---

### 1.4 应用时需核（**不属于粘贴正文**）

1. **提交号已纠正**（A-1/A-2 均已按 `record-260912-01.md` §4.5 写入）：A1a+A1b 同在 `b2b2f26`；`6b90998` = docs 提交（plan + record）；`e8decef` = judge C1-C12。旧稿 §5-1 的「两号零命中 ⇒ 存疑」**作废**，粘贴时勿保留该存疑句。
2. **插入顺序**：A-1 → A-2（同一 EOF，两次追加；若先插 A-2 再插 A-1，A-2 的「上文 A 线」引用会错序）。
3. **状态不得升格**：A-1 = candidate、A-2 = **draft**（judge 已做 = PASS-with-conditions / record §6 已回填 / 用户未 confirmed / 1.21.6 强制 bulk 缺陷根因未定位）。A-2 的标题与正文均须保留 `🔍 draft` 字样。
4. **A-1 的「实际 2026-09-11 19:2x–20:5x」** 为主会话给定（材料内仅见「18:02 运行 / 19:36 提交」）——如需精确区间，标「（材料未声明，应用时需补）」。
5. 粘贴后 `git add` 前扫一遍：主题篇（07）只放结论小节，**不得**把 §4.6 的逐条声明表 / §4.7 的 dll 事故过程搬进 07（过程留 10 + `.investigations/`）；**Wave 2 的 1.21.6 确认缺陷（E5）与门数字纠错（E4）只在 07/10 各留结论 + 指向 `errors-260912-01.md`，不复制五段式全文**。

---

## 2.【成稿 B】`versions\1.20.1\docs\07-block-pipeline.md` 追加小节（两个，按序追加在 EOF）

### 2.1 插入锚点（精确）

- **主锚点**：文件**末尾**（当前 total = 1289 行）；末行 = `1289: - **遗留 / 未覆盖**：R9-b 并发可见性专项未做；…共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）。`
- **追加顺序**：B-1（A 线 2026-09-11）→ B-2（2026-09-12 Wave 2）。**时序说明**：B-1 的实际时点（09-11 19:2x）早于现有末节 C 线（09-11 20:56–22:3x），按既有「追记」实践追加在 EOF，B-1 小节首行已声明与 C 线的**路径关系**（旧逐块路径经 `-Dcoreswap.bulkwb=0` 保留）⇒ 不得读作「已消失」。
- **备选锚点**（若主会话偏好严格时序）：B-1 插到 C 线小节之前，即 `1233: ## 2026-09-11 C 线：Java 侧 bulk section 写回（D1，Rust 零改动）— candidate（judge PASS-with-conditions；M1/M2/M3 已应用 / confirmed 留用户）` **之前**（锚点文本 = 行 1233 整行）；B-2 只能追加在 EOF。**二选一，勿两处都插。**

### 2.2 可直接粘贴正文 B-1（A 线 2026-09-11；从 `##` 行起整段复制）

## 2026-09-11 A 线：Java 侧写回优化池（A1a 跳空气 / A1b 诊断门控）+ Rust 共享层线程 clamp（A1d）+ nether/end 全维行为门（A2）— candidate（judge PASS-with-conditions；C1-C12 + I1-I4 已应用 / confirmed 留用户）

> 载体与依据：`.investigations/a1-opt-pool-260911-05/record-260911-05.md`（A1a/A1b/A1c/A1d + 行为门 4140/4140 + §9.7 降级声明）+ `a2-dim-gate-260911-05.md`（nether/end 各 4761/4761）+ `review-260911-05.md`（judge 原文 + §7 处置表）；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`）。提交 A1a+A1b `b2b2f26`（**同提交**）/ A1d `b53b23f` / A2 `8d8075a` / judge 修正 `e8decef` / docs `6b90998`。
> ⚠️ **与 C 线小节的路径关系**：C 线（本篇「2026-09-11 C 线」小节）把默认写回改走 bulk section 路径；**A1a/A1b 所在的旧逐块路径保留不删**（回退 `-Dcoreswap.bulkwb=0`）⇒ 本节结论对该回退路径**仍然有效**，不得读作「已消失 / 已被 C 线取代」。

### A1a 写回跳空气（`CppBridge.writeChunk`，回退 `-Dcoreswap.skipair=0`）
- **形态**：`writeChunk` 遇 raw id 0（= `minecraft:air`，`data/blocks.json` 0）不再逐格 `setBlockState`；开关静态 final ⇒ 常量折叠，生产路径实际新增每格一次 `id==0` 判断（judge I4）。
- **判据（预登记）**：跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；谓词取 **`!isOf(Blocks.AIR)`**（`isAir()` 会漏计 `cave_air` 730 / `void_air` 729，而跳过的写是「写 air 默认态」）。
- **vanilla 依据**：`ChunkSection.setBlockState` 只随 `isAir` 变化增减 `nonEmptyBlockCount`；`PalettedContainer.swap` 同值写回同一 palette index ⇒ 对空气格写 air 为语义 no-op（judge 补引；机制指纹 → algorithm-fingerprints #25）。
- **等价性（Full，噪声无关；§9.7 三要素）**：
  - **载体**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），`[WB-CHECK]` 全格计数。
  - **覆盖面**：三维全格计数，非抽样——overworld 276,171,456 / nether 194,630,604 / end 154,752,709 空气格被跳过，`stale_nonair` **三维均 0**；覆盖面分母（经接管 chunk 数）见 A2 段（overworld 4140 / nether·end 4761）。⚠️ 该项为**单 run 单维、无负对照、与计时同 run**（judge 核对表）。
  - **可比性**：a1 双臂同 dll `838e89794a54e19d`、同 seed/区域中心/工具修订、背靠背串行；三归档臂（exec-def/cap0-fresh/exec-16）为 **A1d 前构建** `dd3b645f2c79d2cb`，跨 dll 逐 chunk 指纹全等（4140/4140）⇒ 兼作 **A1d 输出中立性独立证据**。
  - **结果**：等价性成立（三维实证）；行为门跨臂 + 跨 dll + 跨 session 归档臂指纹 `hash_diff=0`。
- **收益：Degraded（不主张定量）**——cpuSec 493→476（**−3.4%**），单对跨 run 且**落在 ±10% 机器噪声带内 ⇒ 只作趋势**；两臂均开 `WBCHECK`（2.76 亿次 `getBlockState`）⇒ **含诊断成本的收益下界**，引用定量数字须关 WBCHECK 重跑。结论性质 = 「等价性成立 + 无回归」。

### A1b 诊断扫描门控（`CppBridge.fillChunk`）
- **形态**：`nzBuf` 全 buffer 扫描（98,304 读/chunk）+ nether/end 16 点读回**整块移入 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。
- **判据**：纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。
- **取舍声明**：改前 `buf-all-air`/`buf-sparse` 为**无条件 println**（原「只门控 println」理由句错误、类 javadoc 旧措辞作废）；`buf-sparse` 分级诊断随全量扫描移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。

### A1d `adaptive_threads` count=1 clamp（Rust 共享层，清理项）
- **形态**：`worldgen-core/src/api.rs` 的 `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`；批量路径（count>1）语义逐字未变。
- **机制自证（运行期）**：`[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll 两条日志行）。
- **与上文 2026-08-16 clamp 小节的关系（本条为该待办在 Rust 共享层的收口）**：8-16 发现的历史前提是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 **per-call `std::thread::scope`**（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ 前提消失、clamp 安全（judge 独立核对）。
- **归口理由**：本篇已承载 clamp 课题的发现与待办（上文「[B]/实机 M=1 结构性串行（threads clamp 发现，candidate）」），且 `03-density-functions.md` 无 threading/池宽章节（`线程池|adaptive_threads|池宽` 零命中）⇒ **不归 03 篇**。
- **无性能归因**（双臂都含 clamp，未做隔离 A/B）；**改前 nthreads=10 为公式推导非实测**（旧 dll 无自证行）。

### A2 nether/end 全维行为门（`CppBridge.fillChunkEnd` 补指纹载体）
- **开工缺口（judge 核实属实）**：end 路径原本**没有 `[WG-CONTENT]` 指纹行**——首轮 end 臂 `intercepted=4761` 而 `contentLines=0`；判据意义 = **「接管生效」与「行为门可判」是两件事**，门禁类课题 MUST 先核「该维是否有载体行」再谈门值。修复 = MIXLOG 块内补指纹行。
- **判据（三条可复用）**：① 该维有载体行（否则先补载体）② 跨形态 diff=0（证 exec/异步化不改内容）③ 跨 run diff=0（证该维 Rust 输出确定性）。
- **结果（Full，噪声无关；§9.7 三要素）**：
  - **载体 / 覆盖面**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），**经接管的全部 chunk 均进门、非抽样**——overworld 4140、nether 4761、end 4761；**分母 = 接管调用数 ≠ Chunky Processed**（门判据不依赖分母）。
  - **可比性**：9 条维度臂 dll 全为 `838e89794a54e19d`、同 seed/区域中心/工具修订；nether 臂与 end 臂**不同 Java 构建态**（end 在补指纹行后重跑）已声明——`git show 8d8075a --stat` = 2 files/57 insertions，Java 仅 +4 行且全在 `fillChunkEnd`，不影响 nether/overworld 路径。
  - **结果**：三维跨形态 + 跨 run 逐 chunk 指纹差 **0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；正对照 overworld vs nether `hash_diff=4140/4140`（门有检测力）；比对脚本已加**最小载体阈值断言**（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）。
  - **打印位置**：overworld 的 `[WG-CONTENT]` 在 `writeChunk` **之前**、nether/end 在其**之后**；三者都对**只读 `buf`** 计算 ⇒ hash 语义一致。
- **附带读数（非门判据，Degraded）**：nether async 25s/24s vs sync 69s（~2.8×，cpuSec 212/202 vs 149）；end async 11s/8s vs sync 20s（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致；跨 run 摆动 **±27%** ⇒ 不宣布定量收益。
- **❌ 排除清单（一行，防重走弯路）**：① 「nether run 级非确定来自 Rust 填充层」以外的下游归因（Java carver/feature/装饰层、写回路径、存档序列化）= **未测外推**，不得当结论引用（judge C8）；② 早前「85 个 Chunky 已处理但无接管行」算式**不成立**——期望集不是 Processed 集（judge C12）；③ A1c「6 张高度图全量重扫」= 前提被 vanilla 一手源证伪（`Heightmap.java:37-71` 本就单遍）⇒ 已评估·不实施（用户裁决）。

### 遗留 / 未覆盖（A 线）
- **写回路径未覆盖（降级）**：指纹取在 `writeChunk` 前/后但**只哈希只读 `buf`**，不覆盖写回结果；**写回后内容指纹（post-write hash）三维均未做**（260910-06 open ⑤ 延续；→ 260912-01 的 `[WG-CONTENT-WB]` 层部分回补该缺口）。
- overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」已废，降为 open 假设）；nether run 级非确定**成因域未测**（只排除 Rust 填充层）。
- A1d 改前 `nthreads=10` 为公式推导非实测；`dd3b645f` 是否确为 A1d 前构建按时间线推断（未反汇编）。
- 1.0.29 **不含 A1 改动**（尚未随任何 release 出货）；1.21.6 侧同构问题属 Phase B 范围。

### 2.3 可直接粘贴正文 B-2（Wave 2 2026-09-12；从 `##` 行起整段复制，接在 B-1 之后）

## 2026-09-12 D3 共享 Java 适配核（Wave 2 语义统一，commit `997d40f`）— draft（V1b PASS-with-declarations；1.20.1 V2+V3 双 PASS；**1.21.6 默认臂 PASS + 强制 bulk 臂发现确认缺陷（阻断 D-4(i)）**；judge = PASS-with-conditions（C1–C9 已响应）/ confirmed 留用户）

> 载体与依据：`.investigations/shared-java-core-260912-01/record-260912-01.md`（§2 pre 冻结 + 构建确定性 / §2.4 V0 接线预检 / §4.1 Wave 1 / §4.2 HOOK-2 / §4.6 V1b 判定 + **构建三态表** / §4.7.0-§4.7.8（V2+V3 / dll 血统事故 / 1.20.1 双 PASS / **1.21.6 回填 + 确认缺陷** / dll 归一化 / 证据落盘 / 一手锚补正 / judge 条件响应 / 未闭合项））+ `review-wave2-260912-01.md`（judge，verdict = PASS-with-conditions）+ `errors-260912-01.md`（E1–E5 + 速查表）+ `evidence/`（42 文件 + `MANIFEST.txt`，tracked）+ 已批准计划 `.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md`（§14 追加式补登）；提交 `ce5286b`（Wave 1 纯移动）+ `997d40f`（Wave 2 语义统一）。通用模式 → knowledge/discovered：workflow-patterns #138 / #14 补充案例（260912-01）、build-tooling #59（合并）/ #60 / #61、compiler-idioms #25。

### 形态（共享核抽取）
- `java-core/src/main/java` = **单一源**；两版 `build.gradle` 各加 `sourceSets { main { java { srcDir '../../../java-core/src/main/java' } } }`（各 +11 行），`git mv` 1.20.1 版入共享、`git rm` 1.21.6 副本（**一个类只有一个家**）。
- **分版差异收进分版缝类 `WgCompat` ×2**：`BULKWB_ON`/`SKIPAIR_ON` 默认常量（1.20.1 = true / 1.21.6 = **false**）；`WgCompat.flag(prop, def)` 语义 = 设了 property 则「非 0 即真」（`0` → false，其它非空 → true，未设 → 默认）⇒ 1.21.6 可用 `-Dcoreswap.bulkwb=1` 强制开启而**不改码**，翻转 = 改一个常量。
- 共享超集：`CppBridge`（并入 1.21.6 独有项 + `WgCompat` 引用）、`BulkWb`/`StallWatch`/`ChunkTiming`（超集）/`CoreSwapFixHelper`；1.21.6 补 `mixin/ChunkSectionAccessor` + `coreswap.mixins.json` 25 条 + refmap 重建（S-1）。
- 回退/门控形态不变：`-Dcoreswap.bulkwb=0`（旧逐块路径保留不删）、`-Dcoreswap.skipair=0`、`-Dcoreswap.chunktime`（默认关）、`-Dcoreswap.bulkwbtest` 等仍有效；1.21.6 `BULKWB_ON=false` ⇒ 该版**默认不调用** bulk 写回。

### 判据（分档，可复用；→ workflow-patterns #138）
- **V1-strict**（**纯移动子集**）：条目级 sha 全等（最好整文件 sha 不变）——Wave 1 实测两版 jar sha **逐字节不变**（1081/1081、1798/1798）。**反向蕴含**：字节全等 ⇒ 同一执行体 ⇒ 行为必然相同，**V2/V3 免跑**（防后波次偷懒/重复劳动）。
- **V1b**（**语义统一波**）：① **未变更侧**条目 MUST 逐字节不变（两侧都变者无字节锚，MUST 显式声明理由）；② 变更条目 MUST **逐条声明**（文件 + 变更性质 + 依据；`jar_manifest_diff.py --expect` 只做归类，**归类 ≠ 声明**）；③ **任何 class 字节发生变化的版本 MUST 补跑 V2 + V3**（「jar 不同」不再蕴含「行为相同」）；④ **判定基线 MUST 钉死构建态**（本波三态：post1 含头注释 → **post2 = V1b 判定基线** → post3 权威 dll；用错态会把「头注释行号位移」或「dll 归一」混进 Java 面差异 —— judge C6）；⑤ **运行期门 MUST 覆盖被测路径与维度**（本波 V3 只覆盖 bulk 写回路径 = 1.20.1 生产路径 + 仅 overworld ⇒ `writeChunkPerBlock` 与 nether/end **无运行期证据** —— judge C3/C7）；⑥ **「默认关」的共享路径 MUST 至少跑一次冒烟**——1.21.6 原本不存在 bulk 路径、共享化后变成可达，强制臂一开即崩（130 chunk `EntryMissingException`、根因未定位、阻断 D-4(i)）⇒ 「默认关」不等于「该路径不存在」（`errors-260912-01.md` E5）。
- **V2/V3 运行期门（两层指纹齐跑）**：`[WG-CONTENT]` = **Rust buf 层**（引擎输出，对 Java 侧改动不敏感）、`[WG-CONTENT-WB]` = **Java 写回读回层**（覆盖写回代码变更）；判据 = 两族指纹 **multiset + sorted-sequence 双判全等**（多重集证集合、序列另证顺序）。**只跑 buf 层 = 假安全感**（→ workflow-patterns #14 补充案例 260912-01）。

### 结论（1.20.1 出货线：行为不变）
- **V2 + V3 双 PASS**（**三臂**同配方、唯一变量 = Java 源码状态、同一 dll `dd3b645f`；pre 臂跑 **2 次**）：441 chunks；异常行实测 **post 4（全 WMI 良性）/ pre 12（8 WMI 良性 + 4 条 `testcontent` 无关噪声）**（原「0 / 0」写法已作废 —— 未测量即断言，E4）；两层指纹各 **607/607**（三臂两两比对）且 multiset + sorted-sequence 全等（`only-pre=0 / only-post=0`）⇒ ① 引擎层输出未变 ② **Java 写回结果逐 chunk 全等**。⚠️ **覆盖面**：三臂均未设 `-Dcoreswap.bulkwb` ⇒ 走 bulk（= **1.20.1 生产路径**）；**`writeChunkPerBlock`（逐格/回退路径）从未执行** ⇒ 其「未变」仅有指令级证据、无运行期证据；**nether/end 零覆盖**（被改的 `writeChunk` 共享于三维度）。**§9.7 口径**：仅 overworld / 仅 bulk 路径；与 C 线 region 层噪声（0.024%）**不同层、不可混用**，本判据为逐 chunk 精确等，且同实现跨 run 在**本层无噪声**（pre-r1 vs pre-r2 全等）。
- **class 条目变化 100% 逐条声明**：1.20.1 = 6 差异 + 1 新增（4 条经 `javap -c -p` 证明「指令完全相同、仅调试属性」`LineNumberTable`；`BulkWb.class` = 仅 `<clinit>` 常量求值序列改走 `WgCompat`，逐值等价；`ChunkTiming` = 已批准超集；`CppBridge` = 超集 + 分派/计时钩子；`StallWatch` = **字节全等**）；1.21.6 = 5 差异 + 5 新增（mixin json/refmap 各 +1 条、2 条仅调试属性、`CppBridge` 超集合并、5 条新增）。
- **1.21.6 侧**（默认 `BULKWB_ON=false`）：**默认臂 PASS** —— boot 42.1s；bridge init（seed / worldgenDir / `stageMask=3`）全对；`[WG-CONTENT]` **625** + `[WG-CONTENT-WB]` **625**（**该版首次带上读回门**）、`[WG-BULKWB]` 0（符合预期）；非 WMI 真实异常 **0**；jar 内 `coreswap.mixins.json` 25 条含 `ChunkSectionAccessor`、启动无 `Mixin apply failed` / `Cannot find target method`。**但**：① **写回替换的内容等价性仍无证据**（该版 pre 无 `BulkWb` / 无读回门 ⇒ 无同层 pre 对照）；② **强制 `-Dcoreswap.bulkwb=1` 臂崩解 = 确认缺陷**（130 chunk 全抛 `EntryMissingException: Missing Palette entry for index 2…8`，栈 `ArrayPalette.get` ← `PalettedContainer.get` ← `ChunkSection.getBlockState`；`[WG-CONTENT-WB]` **0**、流水线未完成；**根因未定位**）⇒ 不阻断 Wave 2（默认关）、**阻断 D-4(i) 翻转**（须先定位 palette 未填充注入点，再跑「默认 / `bulkwb=0` / `bulkwb=1`」三臂逐 chunk diff=0，且覆盖 nether/end）。
- **可复用过程判据（→ KB）**：① 跨臂/跨 run 对照前 **MUST 逐臂读「执行体自证行」**（`[CppBridge] dll= sha256=`）核对，**不得只看 target 文件 sha**（本轮 target 被「A/B 收尾 restore」与「实验暂存」两次改写 ⇒ 首轮对照执行了不同引擎、判定作废；build-tooling #59 + `errors-260912-01.md` E1）；② **fresh `git worktree` 不能当「条目级」基线**（检出文本被 `core.autocrlf` 物化为 CRLF ⇒ `worldgen-data/**` **1024 条**伪差异、**class 条目 0**；条目级基线 MUST 同工作树，跨树比对只可用于类文件/运行期判据；build-tooling #60 + `errors` E2 + 一手锚 record **§4.7.6**）；③ **需要字节锚的逐字复制文件不得加头注释**（`LineNumberTable` 随源码行号位移 ⇒ 条目 sha 变、指令不变；compiler-idioms #25）；④ **注释修补保行数即保字节**（`ChunkTiming.java` javadoc 逐行替换 ⇒ `post4` ≡ `post3`、差异 0；compiler-idioms #25 的正向对偶）；⑤ **门数字 MUST 复算 + 附可复现命令 + 原始输出落盘 + 定义口径**（本波异常行曾写 0/0、实测 4/12；`errors` E4）；⑥ **javap 差异计数 ≠ 语义差异**（三陷阱：lambda 序号 / zip 级联 / 常量池未归一；`errors` E3 + build-tooling #61）。

### 遗留 / 未覆盖（Wave 2）
- **1.21.6 强制 bulk 缺陷（确认缺陷）**：`EntryMissingException: Missing Palette entry for index 2…8` 根因未定位 ⇒ **阻断 D-4(i)**；默认臂干净 ⇒ **不阻断 Wave 2**（`errors-260912-01.md` E5 + record §4.7.8）。
- **内容等价性缺口**：① 1.21.6 写回替换无证据（该版 pre 无对照载体）；② `writeChunkPerBlock` 无运行期证据（1.20.1 回退路径；低成本闭合 = 跑 `-Dcoreswap.bulkwb=0` 臂与生产臂对比 `[WG-CONTENT-WB]`，未做）；③ **nether/end 零覆盖**（本波门仅 overworld，而被改的 `writeChunk` 共享三维度）。
- **judge 已做**（`review-wave2-260912-01.md` = PASS-with-conditions，C1–C9 已全部响应/修正，record §4.7.7）；**record §6 已回填/ 用户未 confirmed**；1.20.1 class 条目**未做全量 javap 逐条对拍**（4 条 `javap -c -p` 全等证明 + 一次方法级对拍；judge 抽样复核 5/5 成立）；`stateById` 归一后 6 行残差已用**源码逐字对照**排除（序号伪差）。
- 1.20.1 生产路径新增 **4 次 `System.nanoTime()`/chunk**（≈80ns/chunk 为算术估计、未实测；保留 1.21.6 逐字形态、不做微优化）；1.21.6 S-6/S-7 诊断面差异已按批准生效。
- **证据落盘（judge C5）**：判据已从 `.tmp/` 复制到 tracked **`evidence/`**（42 文件 + `MANIFEST.txt`：清单 / 差异输出 / 门控原样行 / 完整日志 / 复现工具；jar 本体与中间 dump 不入库、只入 sha）⇒ 判据可从仓库复现（此前「只在 `.tmp`」的状态已终结）。
- **再发布提醒**：`build/libs` 现产物与已发布 1.0.29 的 jar sha `b057fda2…` **不同**（发布 jar 已被本地构建就地覆盖，§2.6「构建产物目录不是存档目录」）⇒ 再发布 MUST 重跑全量回归 + 三元组重算。

### 2.4 A1d 归口判定（沿用旧稿，写明理由）+ 可选补丁

**判定：A1d 归 07（B-1 的 A1d 段）+ `07:601` 取代指针，不只进时间线。** 理由：① 07 已承载 clamp 课题的**发现 + 待办**（`07:571` 小节「2026-08-16 影响评估修正…clamp 发现」，其中 `07:601` = 修复待办行），A1d 是同一课题在 **Rust 共享层**的收口 ⇒ 检索入口在 07；② `03-density-functions.md` **无论证归口**（`线程池|adaptive_threads|池宽` 零命中，其 `clamp` 命中全是数学 clamp）；③ A1c 才是「只进时间线」项（07 只留一行 ❌）。

**可选补丁（A1d 取代指针，建议应用；`07:601` 该行**末尾追加**，原句不删不改——§15.4）**：

` ⬆️ **260911-05 A1d 收口（Rust 共享层同族 clamp）**：`worldgen-core/src/api.rs` 的 `adaptive_threads` 已改为一律 `threads.min(count).max(1)`，运行期自证 `[WG-THREADS] count=1 nthreads=1`；本条历史前提（常驻池）在现实现（per-call `std::thread::scope`，唯一调用点 `api.rs:143`）中已消失 ⇒ clamp 安全（清理项、无性能归因；详见 10 时间线 260911-05 块）。旧措辞按 §15.4 保留不改。`

（`10:1475` 同源待办行与 `10:1485` open 行同法追加与否**由主会话裁决**；不加不影响本块结论。）

---

## 3. §15.4 取代链建议（原结论正文不删不改，只加取代指针）

| # | 被取代的原陈述（锚点，**行号会漂移、以锚点文本为准**） | 取代内容 | 建议指针文本（就地追加） |
|---|---|---|---|
| S-1 | `10:3103`（260910-06 块 open 行）①「nether/end 全维行为门未做（指纹门只覆盖 overworld 4140/4225 = 98.0%）」 | A2 已建三维载体与门；且「4140/4225 = 98.0%」的**分母口径**被 judge C3 判为应为「经接管 chunk 数」 | `⬆️ 260911-05 A2 取代：nether/end 全维行为门已做（nether/end 各 4761/4761 逐 chunk diff=0）；原「4140/4225 = 98.0%」分母口径作废——分母应为**经接管 chunk 数**（≠ Chunky Processed），门判据不依赖分母。写回路径仍未覆盖（该缺口于 260912-01 由 `[WG-CONTENT-WB]` 层部分回补）。` |
| S-2 | `10:3103` ⑥「85 个『Chunky 已处理但无接管行』chunk 与 621 格空白成因未查」 | 「85」算式不成立（期望集不是 Processed 集）；实测 overworld 相对 nether/end **少 621 条载体**，成因降 open | `⬆️ 260911-05 A2 取代：「85 个已处理但无接管行」说法不成立（judge C12）；实测 overworld 相对 nether/end 少 **621 条载体**（4140 vs 4761，同 center/radius/Processed），成因未查、降 open 假设。` |
| S-3 | `07:601`「修复待办（clamp 改 `if (threads > count && count > 1)` 或实机改批量调用）」；`10:1475` 同源待办行 | Rust 共享层已实施 `threads.min(count).max(1)`（与待办建议写法不同）+ 前提（常驻池）消失 | 见 §2.4 可选补丁 |
| S-4（延续，非取代） | `10:3103` ⑤「b3『写回/并发路径非确定』未被排除」 | A2 仍**未排除**（写回后指纹三维均未做）⇒ 状态不变，仅延续登记 | 无需指针（A-1 的 open ① 已承接）；如需可加「⬆️ 260911-05：仍未排除（写回后指纹三维均未做；260912-01 已补 Java 写回读回层门）」。 |
| **S-5（新）** | `10:3154`（260911-06 块 open 行）⑥「共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）」 | **260912-01 已交付**：Wave 1 纯移动（`ce5286b`）+ Wave 2 语义统一（`997d40f`），`java-core/src/main/java` 成为两版单一源 | `⬆️ 260912-01 取代（承接）：共享 Java 适配核已抽取（Wave 1 纯移动 V1-strict PASS + Wave 2 语义统一 V1b PASS-with-declarations；1.20.1 V2+V3 双 PASS；**1.21.6 默认臂 PASS（625/625）、强制 bulk 臂发现确认缺陷（阻断 D-4(i)、不阻断 Wave 2）**）；「只落 1.20.1 一份」不再成立（两版共享源 + 分版缝类 `WgCompat`）。**judge = PASS-with-conditions（C1–C9 已响应）；用户未 confirmed**（见该块 🔍 draft 状态）。` |
| **S-6（新）** | `07:1289`（C 线「遗留 / 未覆盖」长行）句「共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）」 | 同上（260912-01 已交付）；另该行「编码非规范位宽…留待 1.21.6 移植随适配核修正」的**处置时机已到**（共享核已建） | `⬆️ 260912-01 承接：共享 Java 适配核已抽取（两版单一源 + `WgCompat`）；C 线遗留的「编码非规范位宽（`BulkWb.java:243` 写请求位宽）」修正时机已具备，**但本波未修**（Wave 2 范围只做语义统一、不改行为）⇒ 登记为后续项。` |

> ⚠️ **S-6 的依据边界**：record §4.7 未提该编码位宽项；本条指针只依据 07:1289 原文 + 本波「不改行为」的范围声明 ⇒ 措辞限定为「时机已具备、本波未修」，**不声称任何修复**。

---

## 4. §9.7 / 分层声明检查表（本稿每个百分比 / 耗时 / 计数）

> 判据：量化输出 MUST 同行声明 **载体 + 覆盖面 + 与既有口径可比性**；缺项一律标 **（材料未声明，应用时需补）**，不代填。

### 4.1 Wave 2（A-2 / B-2）

| # | 数字（出处） | 载体 | 覆盖面 | 可比性 | 处置 |
|---|---|---|---|---|---|
| W1 | **构建三态** jar sha + 条目数（pre 1081/57 → **post2 = V1b 判定基线** `0681ec03…` 1082/58；1.21.6 1798/54 → `772d7a6e…` 1803/59；post3 = 权威 dll `461baedc…`；post4 ≡ post3） | ✅ 主树同源构建（`--rerun-tasks`）+ 条目级 sha256 清单（`evidence/tool-jar_manifest.py`） | ✅ 全条目（1081/1798），非抽样 | ✅ 同树、同配方；构建确定性已实测（同源两次字节相同 / 跨 2 天同源相同） | 可直接引用（须带「三态 + 判定基线 = post2」） |
| W2 | 方法级 `same=28 / changed=4 / added=1`；`stateById` 归一后残差 6 行 | ✅ javap（偏移/常量序号/`ldc_w` 归一） | ✅ 32→33 方法全集 | ⚠️ 归一化掩蔽了常量池序号伪差（已用源码逐字对照排除）；**lambda 名序号域不可按名对拍** | 引用须带「归一化 + 源码对照」两句 |
| W3 | V2 **三臂** boot 55.8s / 54.2s、51.2s；441 chunks / chunky 自报 **5s**（脚本 10s = 轮询粒度）；异常行实测 **post 4 / pre 12**（WMI 良性 + 4 条 `testcontent` 无关噪声） | ✅ live dev-server chunky（overworld, r160, 441 chunks, seed `417950215108767439`） | ✅ 本次生成 chunk 集（441） | ✅ 三臂同 dll/seed/坐标/门控，唯一变量 = Java 源码状态（pre 侧 `testcontent` 环境噪声已声明） | boot/生成时长为**环境读数**（未做噪声带声明）⇒ 只作过程记录；**异常计数 MUST 带口径 + 可复现命令**（E4） |
| W4 | `[WG-CONTENT]` / `[WG-CONTENT-WB]` 各 **607/607/607**（三臂两两）+ multiset/sorted-sequence 全等；1.21.6 默认臂 **625/625** | ✅ 两层指纹（Rust buf 层 + Java 写回读回层） | ⚠️ **仅 overworld / 仅 bulk 写回路径**（`writeChunkPerBlock` 与 nether/end **零覆盖**）；607 chunk 逐 chunk、非抽样 | ✅ 同 dll/seed/坐标；**与 C 线 region 层 0.024% 噪声不同层**（逐 chunk 精确等） | 1.20.1 侧可直接引用（须带「两层 + 双判 + **覆盖面边界**」）；1.21.6 默认臂 625/625 仅**正向 sanity**，**不构成内容等价证据** |
| W5 | 1.20.1 +4 `nanoTime`/chunk ≈ 80ns/chunk | 算术估计（非测量） | — | **（材料未声明/未实测）** | 正文已标「算术估计、未实测」；不得作收益论断 |
| W6 | 1.21.6 dll `abd7d889…` 三处一致；1.0.29 票 dll `dd3b645f…` | ✅ 三元组（票 / jar 内 / target） | ✅ 两版各 1 | ✅ 事后核验（事故后） | 可直接引用（须带「事后」） |
| W7 | worktree CRLF 伪差异「`worldgen-data/**` **1024 条**、class 条目 **0**」 | ✅ record **§4.7.6** 一手锚 = `evidence/prewt-pseudodiff.txt`（prewt vs post3 清单） | ✅ 全条目（1081）逐条归类 | ✅ 反证 = post2/post3 **同树**比对仅 1 条 dll | 可直接引用（→ KB build-tooling #60 + `errors` E2） |
| W8 | 1.21.6 强制 bulk 臂 **130 chunk** `EntryMissingException` / 日志 132 次 / `[WG-CONTENT-WB]` **0** | ✅ `evidence/log-1.21.6-bulk.txt` + `fp-1.21.6-bulk.txt`（0 条） | ✅ 该臂全部写回尝试（130 chunk） | ✅ 与默认臂（干净）对照 ⇒ 缺陷归因于该路径 | 计数可引用；**根因不得引用任何候选**（三条方向均未验证，`errors` E5） |
| W9 | `post4` vs `post3` **差异 0**（C4 复算） | ✅ `evidence/post3-vs-post4-{1.20.1,1.21.6}.txt`（相同 1082、差异 0） | ✅ 全条目 | ✅ 同构建链、仅 javadoc 逐行替换（总行数不变） | 可直接引用（→ compiler-idioms #25 正向对偶） |

### 4.2 A 线（沿用旧稿结论，不再重列）

详见旧稿 `knowledge-draft-260911-05-docs.md` §3 的 12 行表（本稿不重复）：可直接引用的只有 A2 门（三要素齐备）；A1a 收益（−3.4%）标「Degraded / 只作趋势 / 含诊断成本下界」；A2 附带读数标 Degraded（±27% 摆动）。**新增一条交叉引用**：A 线遗留的「写回后内容指纹三维均未做」在 260912-01 由 `[WG-CONTENT-WB]` 层**部分回补**——1.20.1 为**逐 chunk 等价**（三臂 607/607，但仅 bulk 路径 × overworld）；1.21.6 默认臂**首次产出门值**（625/625）却**无 pre 对照 ⇒ 等价性无证据**，强制 bulk 臂更崩解 —— 引用时须写成「部分回补」而非「已闭」，且**不得跨版外推**。

---

## 5. 未核 / 存疑清单（**不替它们补论证**）

| # | 存疑项 | 性质 | 本稿处置 |
|---|---|---|---|
| D1 | **§4.7.3 1.21.6 臂** | **已回填**：默认臂 PASS（`[WG-CONTENT]`/`[WG-CONTENT-WB]` 625/625）+ 强制 bulk 臂**确认缺陷**（130 chunk，根因未定位） | A-2/B-2 已改写（不再写「待回填」）；**缺陷根因不得写任何候选为结论**；「1.21.6 内容等价」仍标**无证据** |
| D2 | **Wave 2 judge / confirmed** | judge **已做**（`review-wave2-260912-01.md` = PASS-with-conditions，C1–C9 已全部响应/修正，record §4.7.7）；**record §6 已回填、用户未 confirmed** | A-2/B-2 状态一律 **draft**（不升格；judge 意见不改 status） |
| D3 | Wave 2 数字（条目数 / 方法级计数 / 异常计数） | 本稿只读 record §4.6/§4.7 转录，**未复算 jar**；**异常计数已由 judge 复算纠正**（原 0/0 → 实测 4/12，E4） | 条目内均带 `record §4.6/§4.7` 锚；条目数已与 `evidence/v1b-*.txt` 头对齐（1.20.1 1082/58、1.21.6 1803/59） |
| D4 | 1.20.1 class 条目**未做全量 javap 逐条对拍** | record 只记 4 条 `javap -c -p` 全等 + 一次方法级对拍（judge 抽样复核 **5/5 成立**） | A-2/B-2 已如实登记为 open ⑨；不宣称全量证明 |
| D5 | 4 次 `nanoTime/chunk` ≈ 80ns | **算术估计、非实测** | 已标「算术估计、未实测」 |
| D6 | 「`gradle :build` 的 dll 同步 doFirst 有自愈作用」 | record §4.7.1 附带发现（单次观察） | 已带「不能依赖」限定；不单列 KB 条目（并入 build-tooling #59 判据⑤） |
| D7 | **`build/libs` 现产物 ≠ 已发布 1.0.29 jar sha** | record §2.6/§4.7.4 明写（发布 jar 被本地构建覆盖） | 已写入 A-2/B-2 的「再发布提醒」；引用工单权威 = 工单内 sha256 |
| D8 | 编码非规范位宽（`BulkWb.java:243`）本波是否随共享核修正 | **record §4.7 未提**；只知 07:1289 记「留待 1.21.6 移植随适配核修正」 | S-6 指针限定为「时机已具备、**本波未修**」，不声称修复 |
| D9 | A 线旧稿的 15 条存疑（提交号零命中、`dd3b645f` 归属推断、f2 成本拆分未复核等） | 部分已由 record §4.5/§4.7 解除 | **提交号存疑解除**（D9-a：A1a+A1b = `b2b2f26`、`6b90998` = docs）；其余（`dd3b645f` 归属、0.42ms 未复核、16 hex 前缀门禁强度）**沿用旧稿存疑**，见旧稿 §5 |
| **D10** | **worktree 换行物化伪差异**（`core.autocrlf` → CRLF） | **已解除**：record **§4.7.6** 补一手锚 = `evidence/prewt-pseudodiff.txt`（prewt 清单 vs post3 清单 = **1024 条差异 / class 条目 0**）+ `errors-260912-01.md` E2 | KB build-tooling **#60** 已按其改写（引用 = record §4.7.6） |
| **D11** | javap「按行 zip 对拍在指令数变化处级联误报」 | **已解除**：record **§4.7.6** 补一手锚（初版把 `stateById` 报成 30/30 + 6 行、`lambda$static$0` 报成 98→97 + 46 行；源码逐字对照证明相同；judge 改三重判定后确认 4/4 条「仅调试属性」）+ `errors` E3 | KB build-tooling **#61** 已按其改写 |
| D12 | worktree 作条目级基线的**流程次序** | **已澄清**（`errors` E2）：worktree 产物**曾**被拿作条目级基线 → 全量伪差异 → **放弃**，条目级基线改用**同一主工作树**的 `post2/post3`；worktree 仅保留运行期 **pre 臂**用途 | KB #60 正文已按此实况改写 |
| **D13** | 1.21.6 写回替换的**内容等价性** | judge 列为「未证伪项」：该版 pre 无 `BulkWb`/无读回门 ⇒ **无同层对照、目前无证据** | 已写入 A-2/B-2 open；**禁止**「已等价」表述 |
| **D14** | `writeChunkPerBlock` 的**运行期证据** | 本波三臂均走 bulk（缺省 `BULKWB_ON=true`）⇒ 该路径**从未执行** | 已入 open；闭合方式已给（跑 `-Dcoreswap.bulkwb=0` 臂对比 `[WG-CONTENT-WB]`），未做 |
| **D15** | **nether/end 覆盖** | 本波门仅 overworld（`[WG-CONTENT-NETHER/END]` 命中 0），而被改的 `writeChunk` 共享三维度 | 已入 open；**不得**由 overworld 结果外推三维 |
| **D16** | 1.21.6 强制 bulk 缺陷**根因** | **未定位**（三条方向候选均未验证） | 只登记「缺陷 + 阻断 D-4(i)」；候选仅留 `errors` E5 / record §4.7.8 |
| **D17** | 门数字「异常行 0 / 0」曾写入记录 | 已由 judge C2 复算推翻（实测 **4 / 12**，口径未定义） | 已改写为实测值 + 分类口径；判据入 `errors` E4 |

---

## 6. 应用前自检（逐条核对，应用后请照抄进签核证据）

1. **主题篇未堆时间线**：B-1/B-2 只含「形态 / 判据 / §9.7 三要素 / ❌ 排除清单 / 遗留」；V1b 逐条声明表、dll 事故过程、judge 处置明细**全部留在 A-2 与 `.investigations/`**。✅
2. **结论与过程分流**：结论（共享核形态、分档判据、1.20.1 行为不变）→ 07；过程（Wave 顺序、事故、CRLF/javap 坑、取代链）→ 10 时间线 + `.investigations/`。✅
3. **状态未升格**：A-1/B-1 = candidate；**A-2/B-2 = draft**（judge 已做 = PASS-with-conditions、C1–C9 已响应，但 record §6 已回填、用户未 confirmed），标题与正文均带 `draft` 字样；无任何 AI 授予 confirmed 的措辞。✅
4. **数字带锚、无编造**：Wave 2 全部数字出 `record-260912-01.md §4.6/§4.7`（含 judge 复算纠正后的异常计数 **4/12**、三态 jar sha、`post4` ≡ `post3`）；两项曾缺一手锚的实测（CRLF / javap zip）**已由 record §4.7.6 补锚**（`evidence/prewt-pseudodiff.txt` / `evidence/src-pre-1.20.1-CppBridge.java` 逐字对照），本稿已改引、不再标「主会话给定」。✅
5. **§9.7 三要素**：可直接引用的 = W1/W4/W6/W7/W8/W9；**W4 已带覆盖面边界**（仅 overworld / 仅 bulk 路径、`writeChunkPerBlock` 与 nether/end 零覆盖）；其余缺项已就地标注或标 Degraded。✅
6. **旧稿一致性**：旧稿的提交号错误、A1b 归属已按 record §4.5 全面纠正（§1.2/§2.2/§2.4）；旧稿锚点（10:3154 / 07:1289 / 07:601）**本轮实读复核仍有效**。✅
7. **取代指针不删原文**：S-1~S-6 全部为「追加指针」形态，未要求改写任何既有行。✅
8. **落盘纪律**：本稿只写 `.investigations/shared-java-core-260912-01/`；未改 docs/knowledge；未触碰 confirmed。✅

---

> **本文件自身状态**：draft（草稿，未应用；2026-09-12 按 judge `review-wave2-260912-01.md` 的 C1–C9 修正后原地修订）。应用后建议状态：A-1/B-1 = candidate（judge 建议保持，confirmed 留人类）；A-2/B-2 = **draft**（judge 已做 = PASS-with-conditions；record §6 已回填；1.21.6 强制 bulk 缺陷根因未定位；confirmed 留人类）。
