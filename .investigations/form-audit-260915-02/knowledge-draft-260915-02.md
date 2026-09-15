# knowledge-draft-260915-02 — 260915-02 工作块知识库落盘草稿（core.knowledge/core.worker）

> 产出者：knowledge subagent（草稿；主会话应用 + 验证后才生效，confirmed 留人类）。
> 依据：`SUBAGENT-KNOWLEDGE-GUIDE.md`（价值门先行）+ cp6-needssaving-verdict.md（candidate，Degraded 静态）+ review-001-cp3-cp6-judge.md（PASS-with-conditions）。
> 价值门预判：A = 高价值（可复用审查判据）→ discovered + 主题篇 + 时间线；B = 高价值结案 → 07 篇小节；C = 中价值过程 → 10 时间线；D = CP-3 低价值不立项 discovered（取舍论证见 D 节）。

---

## A. 可复用判据评估 + discovered 条目草稿（发现 #152）

**价值门判定：够格（高价值）**。理由：
1. 这是**审查方向级判据**（「核对哪条链」），不是一次性机制结论——任何未来对生成期写回/落盘完整性的疑点（形态审计 D 段类、bulk 写回类、未来版本升级后的标脏面核对）都会重走同样的判断：若按直觉查 setBlockState 链会得出「不落盘」假疑点（CP-6 疑点本身就是这么产生的），重查一遍 = 重花一轮一手源核对。
2. 有反直觉坑：「标脏」直觉上归属块写链，实际在 status 阶段推进链——符合「判错方法/可复用判据」高价值档。
3. judge 六点一手源抽查零漂移 + 推理链问询闭合，证据扎实。

**载体选择**：`knowledge/discovered/workflow-patterns.md`（非 algorithm-fingerprints——本质是「审查核哪条链」的工作流判据，机制行号只是判据的锚；且 #151 已有同域审查分工条目，家族连续）。当前最大发现编号 = **#151**（workflow-patterns.md:2596），续号 **#152**。

### 建议写入的精确目标与位置

- **目标文件**：`E:\PYTHON\CoreSwap\knowledge\discovered\workflow-patterns.md`
- **插入位置**：文件末尾（发现 #151 条目之后），追加。
- **同步动作**（主会话应用时）：`knowledge/INDEX.md` 若有条目清单需同步登记 #152（按 INDEX 现有格式）。

### 完整文本

```markdown
## 发现 #152: MC 生成期 chunk 持久化标脏 = status 阶段完成统一置位，不随块写发生——审查生成期写回落盘完整性核对阶段推进链而非 setBlockState 链（260915-02）

- **发现时间 / 发现者 / 置信度 / module**：260915-02；knowledge subagent 草稿（判据候选由 CP-6 verdict 提出、judge 六点一手源抽查零漂移后交评）；candidate（一手源 file:line 静态核对，Degraded 如实声明；未做运行时复现——非必需，机制链完整）；workflow-patterns / vanilla 机制审查判据（#151 同域）。
- **来源定位**：`.investigations/form-audit-260915-02/cp6-needssaving-verdict.md`（candidate）+ `review-001-cp3-cp6-judge.md`（PASS-with-conditions）；一手源 = `.tmp/scout-260905-08/mcsrc`（1.20.1 yarn，DataVersion 3465）。
- **观察（动机 = 一个真疑点被一手源否定）**：形态审计 260915-01 CP-6 疑点「bulk 原地替换不经 setBlockState/标脏链 → 早 unload 存盘场景生成方块可能不落盘」（推理级）——直觉把「标脏」归属块写链。一手源核对否定：① 门控真实存在（`ThreadedAnvilChunkStorage.save()` :797-802，`!needsSaving()` 即不序列化）；② **vanilla 自己的生成期块写也不标脏**（`ProtoChunk.setBlockState` :108-158 无任何 needsSaving 置位）——「标脏靠块写链」前提对 vanilla 本身不成立；③ 真正标脏点 = **每个生成阶段完成时统一置位**：`ChunkStatus.runGenerationTask` :357-363 的 `doWork(...).thenApply(... setStatus(this))`（:361-362，`!isAtLeast(this)` 时）→ `ProtoChunk.setStatus` :215-222 末行 `setNeedsSaving(true)`（:221）。CoreSwap bulk 拦截点在 NOISE 阶段内 ⇒ 阶段完成即标脏，与块写入路径无关 ⇒ 疑点不成立，不立项。
- **判据（可复用）**：
  1. **机制判据**：MC 生成期（ProtoChunk 管线内）chunk 持久化标脏 = status 阶段完成统一置位（锚点链：ChunkStatus:362 → ProtoChunk:221），**不随 setBlockState/块写发生**。补充标脏面（后续阶段兜底）：`Chunk.setLightOn` :389-391、Chunk 结构四方法 :214/224/235/247、`ChunkHolder.markForLightUpdate` :190、`ChunkSerializer` 载入 :212-214。
  2. **审查方向判据**：凡审查「生成期写回是否落盘完整」，核对对象是**阶段推进链**（该 chunk 是否走到下一阶段完成的 thenApply），**不是 setBlockState 链**——按后者审查会把 vanilla 同构行为误判为对齐缺口（制造假疑点）。
  3. **范围边界（引用本判据必须随附）**：① 本判据限**生成管线期**；全部 status 完成后的**非生成期** bulk 写不在覆盖面（该场景 vanilla 走 `WorldChunk.setBlockState` 自标脏域，另一条链，出现该形态需另核）；② `isAtLeast(this)` 已达标路径（重入/跳跃推进）不再置位——良性（首达已标脏）；③ 阶段完成前 abort/unload 不标脏 = vanilla 同构的管线取消语义，非偏差；④ `WrapperProtoChunk`（:181-186 转发 setNeedsSaving）是 worlds 边界视图，不进生成任务链（:361 `instanceof ProtoChunk` 只匹配中心 chunk），无独立缺口。
- **家族索引**：#151（审查分工/条件项闭合——本条是其 vanilla 机制侧输入实例）；#36 家族（静态机制断言须一手源核对——本条为「核对方向选对链」的正例）；#149 判据 4（参照系同位拆解——CP-6 疑点生成即参照系错置：把块写链当标脏参照）。
```

---

## B. docs 07 篇 D-④-b 结案小节草稿

**目标文件**：`E:\PYTHON\CoreSwap\versions\1.20.1\docs\07-block-pipeline.md`
**插入位置与方式（两处，均为追加/标注，不覆盖旧描述——§15.4 意识）**：

1. **就地标注（最小改动）**：候选池表格 CP-6 行（:1552）末尾「核对优先于立项」后追加一枚结案指针，改该行末尾为：
   `…核对优先于立项 **→ 已结案不立项（260915-02，见「D-④-b 结案」小节）**`
   （旧描述文字全部保留，只追加指针标注。）

2. **追加结案小节**：文件末尾（:1566 `> 通用模式 → …` 行之后），追加完整文本如下：

```markdown

### D-④-b needsSaving 结案（260915-02）：CP-6 疑点不成立，不立项（candidate）

> 状态：candidate（一手源 file:line 静态核对，Degraded 分层如实声明；judge PASS-with-conditions，review-001）。本小节为 260915-01 候选池 CP-6 行的**结案补充**——上表 CP-6 行的原始疑点描述不删不改（§15.4：结案以追加标注表达，原行已就地加「已结案」指针）。

- **核对结论**：`needsSaving` 门控真实存在（`ThreadedAnvilChunkStorage.save()` :797-802），但**标脏机制 = 每个生成阶段完成时统一置位**（`ChunkStatus.runGenerationTask` :357-363 `thenApply` → `ProtoChunk.setStatus` :215-222 末行 `setNeedsSaving(true)`），**不随块写发生**（vanilla `ProtoChunk.setBlockState` :108-158 自身即无标脏置位）。CoreSwap bulk 原地替换与 vanilla populateNoise 块写在同一标脏面上（均为块写不标脏、阶段完成标脏）⇒ 「早 unload 存盘不落盘」疑点不成立，**不立项**。
- **范围边界**：① `isAtLeast(this)` 已达标路径不再置位——良性（首达已标脏）；② 本结案限**生成管线期**，全部 status 完成后的非生成期 bulk 写不在覆盖面（vanilla 该场景走 WorldChunk.setBlockState 自标脏，另一条链）；③ WrapperProtoChunk 不进生成任务链（:361 instanceof ProtoChunk 只匹配中心 chunk），无独立缺口；④ 阶段完成前 abort/unload 不标脏 = vanilla 同构的取消语义。
- **判据沉淀**：→ workflow-patterns **#152**（审查生成期写回落盘完整性核对阶段推进链而非 setBlockState 链）。
- **产物**：`.investigations/form-audit-260915-02/cp6-needssaving-verdict.md` + `review-001-cp3-cp6-judge.md`（CP-6 建议 candidate，confirmed 留用户）。
```

---

## C. 10 时间线 260915-02 块草稿

**目标文件**：`E:\PYTHON\CoreSwap\versions\1.20.1\docs\10-timewise-archive.md`
**插入位置**：文件末尾（260915-01 块之后），追加。格式对齐现有块（`## YYMMDD-##` 标题 + 状态标注 bullets；日期锚以主会话实际 Get-Date 为准——草稿中的「实际 HH:MM」占位由主会话落盘时回填真实时间，禁止沿用占位符）。

### 完整文本

```markdown

## 260915-02（实际 2026-09-15 HH:MM 起，Get-Date 锚：形态审计执行序第一批——CP-3 防御性修复 + CP-6 源码核对结案 + judge）🔍 candidate 建议（judge PASS-with-conditions；confirmed 待拍板）

> 过程产物 `.investigations/form-audit-260915-02/`（cp6-needssaving-verdict.md + review-001-cp3-cp6-judge.md）+ 架构计划 `.investigations/000-架构设计/`（260915-02，轻量档）。

- ✅ **CP-3（光照 scratch RefCell→Mutex，防御性修复）**：light/mod.rs 13 行（9+/4-）四改动点（use Mutex / 字段+注释 / 构造 / `lock().unwrap_or_else(|e| e.into_inner())`）；poison→into_inner 合理（scratch 每调用开头 clear+resize）；单线程语义等价（无重入路径）。**#150 纪律合规**：防御性修复先行、CP-2 解粘未动；mod.rs:64-67 注释声明「UB 可达性未实证，防御性修复」（260915-01 judge C-4 落地）。cargo build/test 14/14 绿（主会话声明；judge C1 未独立重跑，dll 双产物时间戳旁证自洽）。
- ✅ **CP-6 结案：不立项**——needsSaving 门控在（TACR:797-802）但标脏 = status 阶段完成统一置位（ChunkStatus:362→ProtoChunk:221），vanilla 生成期块写自身也不标脏 ⇒ bulk 原地替换与 vanilla 同标脏面，疑点不成立。judge 六点一手源抽查零漂移。范围边界：限生成管线期（非生成期 bulk 写另一条链，另核）。判据 → workflow-patterns **#152**。
- ✅ **judge review-001（CP-3+CP-6）PASS-with-conditions**：CP-3 条件 C1（cargo 输出未落 cmd-output）/ C2（「近零成本」未 benchmark，措辞保留「预期近零」）/ C3（index.yaml 补登记）；CP-6 条件 C4（verdict 补范围边界两句，可选）——均不阻塞 candidate 推荐。
- 📌 落盘：07 篇 D-④-b 结案小节（追加，原 CP-6 行加结案指针）+ discovered #152；CP-3 为一次性工程修复不立项 discovered（#150 已覆盖其判据面）。judge 条件 C1-C4 由主会话收尾应用。
```

---

## D. 低价值取舍判断：CP-3 不进 discovered

**判断：不立项 discovered 条目**。理由：
1. **判据面已被 #150 覆盖**：CP-3 的可复用部分（「防御性修复可先行、解粘不得先行」「隐式安全保证须有显式承接物」）就是 260915-01 的发现 #150 判据 1/2——本次是**该判据的首次正向执行实例**（合规案例），不是新判据。执行实例的中价值记录已由 C 节时间线条目承载（时间线一行 + judge 合规核对），不重复立项。
2. **修复本体是一次性工程**：RefCell→Mutex 的具体 diff、poison 恢复策略、单线程等价论证都是本仓本文件的特定事实，无跨项目复用价值（价值门「低价值：一次性结论/自推可得」档）。
3. **若未来要补**：唯一可能的增量角度是「Mutex poison 的 into_inner 恢复策略适用前提（缓冲区每调用自清）」——但这属于 compiler-idioms/Rust 惯用法层且过于局部，当前不满足「再遇到不想重新想」的门槛；留给未来真实第二次遇到时再评估。
4. 反向自检（防漏记高价值）：本次无新错误/踩坑（verdict 错误记录节为空，转抄核对通过 = #90 家族通过例），无需错误台账条目。

---

## 产出自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 先过价值门：A/B 高价值、C 中价值简记、D 判定不写并给理由
- [x] 本块无错误条目（verdict 错误记录 = 无），无五段式欠账
- [x] 根因/判据为机制层，定位含一手源 file:line（可复用）
- [x] 被排除疑点（CP-6）保留一行排除式记录（B 节结案小节即载体），未删除
- [x] 载体正确：判据→discovered workflow-patterns #152；结案→07 主题篇追加小节；过程→10 时间线；一次性修复→不进 discovered
- [x] 数字/行号全部来自 verdict/judge 原文，无编造；格式与各目标文件末尾现状对齐（均已先读）
- [x] §15.4 意识：07 篇原 CP-6 行不删不改只加指针；时间戳占位符已标注须主会话 Get-Date 回填
