# 知识库草稿 —— 260919-04 工作块（subagent 产出，主会话审阅后应用）

> 依据：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（价值门 + 载体映射）。素材仅限任务给定的 260919-04 事实清单，无编造。
> 载体：① `versions/1.20.1/docs/10-timewise-archive.md` 追加 260919-04 块（过程记录，中价值简记，正常写）；
> ② `knowledge/discovered/workflow-patterns.md` 发现 #193（可复用判据，**候选草稿**——是否够格由主会话按价值门终审）。

---

## 一、10-timewise-archive.md 追加块草稿（追加到文件末尾，接 260919-03 块之后）

```markdown
## 260919-04（2026-09-19 16:48 起）交接陈旧漂移修复（NEXT_SESSION 停在 260919-02 vs git HEAD 已到 260919-03）+ 260919-02/-03 双 verdict confirmed 回写 + C-1 前置 G3 收敛性预研（scout 机制地图 + 主候选 C-1a）🔍 过程块（C-1 立项待后续正式 Phase 0）

> 承接 260919-03 收尾后的独立工作块。接手时发现 NEXT_SESSION.md 停在 260919-02，而 git HEAD 显示
> 260919-03 已全量执行完（C-3+C-4 实现 → 裁断 → 知识库 96a8b9a）——「下轮第一 hook」指示未对
> git HEAD 验证导致交接陈旧漂移；本块先修复交接（重写 NEXT_SESSION），再做 C-1 前置预研。

- ✅ **交接漂移修复（本块触发事件，五段式要点）**：
  - **现象**：NEXT_SESSION.md「下轮第一 hook」指向 260919-02 的下游，而 git HEAD 已含 260919-03
    全量产物（96a8b9a：C-3+C-4 实现 → 裁断 → 知识库）——交接文档落后实际进度一个工作块。
  - **根因（机制）**：上块（260919-03）收尾时未同步重写 NEXT_SESSION；「下轮第一 hook」是交接文档
    内容，未经 git HEAD 交叉核对即被当作当前状态继承——交接 claim（qualitative 类）未按 §1.3
    重采样原始事实源（git HEAD）核对，属交接陈旧漂移（AGENTS STEP 1「交接结论验证纪律」的交接侧形态）。
  - **定位**：接手例行交叉核对——NEXT_SESSION 指示 vs `git log` HEAD 不一致，以 git HEAD 为权威锚。
  - **修复**：按当前最新状态重写 NEXT_SESSION.md（交接文档唯一权威，只在换 session 前更新——
    本块即换界点）；判据沉淀 → workflow-patterns **#193**（接手时 git HEAD vs NEXT_SESSION 交叉核对）。
  - **教训**：交接文档是快照不是实时事实源；接手第一动作 = git HEAD 对账，不一致先修交接再开工。
- ✅ **用户拍板（confirmed 授予 ×2）**：verdict-260919-02 与 verdict-260919-03 均 confirmed
  （ask_user_question）；C-3 缺省关已核实——build.gradle:111-112 → LIGHT_PACKED=false，
  ServerLightingProviderMixin.java:81 消费点确认，**无需改码**（260919-03 建议 ① 落地完成）。
- ✅ **架构计划落盘**：`.investigations/000-架构设计/架构计划-260919-04-C1前置G3收敛性预研.md`
  （轻量档，已批准）——C-1 立项前先做 G3（光照跨任务状态）收敛性约束的机制预研。
- ✅ **C-1 前置 scout 机制地图**：`.investigations/c1-pre-260919-04/scout-map.md`——关键结论：
  G3 性质 = 输出为本帧快照确定函数；fill chunk 局部纯函数可共享；**BFS 跨任务缓存触 G3**；
  改动面 DOM = 48 常量；open 4 项登记（预研阶段，未闭合）。
- ✅ **C-1 设计文档（预研产出）**：`.investigations/c1-pre-260919-04/design-c1-260919-04.md`——
  **主候选 C-1a** = 全域 fill 一趟 + 9 中心 BFS（不跨任务缓存，绕开 G3 约束）；
  **C-1b（跨任务缓存）不立项**（触 G3，收敛性代价高于收益，❌ 排除留档）。
- ✅ **提交**：aebf0f6——confirmed 落盘（双 verdict 回写）+ 架构计划 + 260919-02 调查文件归档。
- 📌 **知识库**：workflow-patterns **#193**（subagent 草稿 `.investigations/c1-pre-260919-04/
  knowledge-draft-260919-04.md` + 主会话应用）；10 篇本条目。
- 📌 过程产物：`.investigations/c1-pre-260919-04/`（scout-map + design-c1 + 本知识草稿）+
  `.investigations/000-架构设计/架构计划-260919-04-C1前置G3收敛性预研.md`。
- 🔍 **未闭合/下一步**：C-1a 立项属独立决策（预研 = 前置 G3 收敛性，非正式 Phase 2 分析）；
  scout open 4 项随正式立项带入；C-2 受 round4 纯算力冻结排序约束（承接 260919-03）。
```

（时间线条目共 9 条主 bullet + 引言行——按既有块密度归并；如主会话要求凑整 10 条，可把
「提交 aebf0f6」与「架构计划落盘」拆为两条独立 bullet。）

---

## 二、workflow-patterns.md 发现条目草稿（追加到 #192 之后）

```markdown
## 发现 #193（高价值·错误优先）: 接手第一动作 = git HEAD vs NEXT_SESSION 交叉核对——交接快照漂移在继承前必须对账（260919-04）

- **发现时间**：260919-04（2026-09-19）
- **module**：workflow / 会话交接与事实源对账
- **现象**：NEXT_SESSION.md「下轮第一 hook」停在 260919-02，而 git HEAD 显示 260919-03 已全量
  执行完（C-3+C-4 实现 → 裁断 → 知识库，96a8b9a）——若按交接指示直接开工，会对已完成的
  260919-03 目标重开（重复劳动/方向冲突）。
- **根因（机制）**：NEXT_SESSION 是**换界时点的快照**而非实时事实源；上工作块收尾未同步重写交接
  文档，快照落后实际进度一个块。接手方把交接里的「下轮第一 hook」当公理直接续推，未做 STEP 1
  交接结论验证纪律规定的廉价独立验证（对 git HEAD 核对，成本一轮以内）——qualitative 类交接
  claim 未重采样原始事实源（§1.3）。
- **定位（怎么发现）**：接手例行动作——NEXT_SESSION 指示与 `git log` HEAD 并读，时间线断裂处
  即漂移信号；以 git HEAD 为权威锚（不可变、带时序戳）。
- **修复**：按当前最新状态重写 NEXT_SESSION（交接文档唯一权威），并以 git HEAD 为准重建开工点；
  双 verdict confirmed 由用户 ask_user_question 拍板后回写。
- **教训（可复用判据）**：
  1. **接手第一动作 = git HEAD vs NEXT_SESSION 交叉核对**（发现/readme 类指示一律先对账再执行）；
     不一致时 git HEAD 为权威，先修交接再开工——交接修复优先级高于新任务。
  2. 「下轮第一 hook」类指示性内容是**交接 claim 不是事实**：指针类（进行到哪/下一步做什么）
     MUST 对账（git log / 产物存在性），方向类才走 ≤1 轮廉价独立验证（STEP 1 纪律）。
  3. 交接漂移的检测签名 = 指示指向的产出物在 git HEAD 中**已存在或已被超越**——见此签名即停，
     禁止按指示续推（同 M14/M11 家族：假设当公理 = 绕圈根因，本条是其交接侧形态）。
- **家族索引**：AGENTS STEP 1 交接结论验证纪律（M14/M11 三犯复盘——本条补「进度指针漂移」维）、
  §1.3 交接 claim 证据分级（qualitative 重采样）、#137（anchor 类属主工具自核——同族「原始事实源
  优先」）、AGENTS 知识库铁律 4（NEXT_SESSION 唯一权威 + 只在换 session 前更新——漂移根因即
  违反此条）。
```

---

## 三、产出自检（GUIDE §四）

- [x] 价值门：时间线 = 过程记录（中价值简记）正常写；#193 = 可复用判据（接手对账签名 + 三条教训），
      判定**够格**，标高价值·错误优先（置信度：candidate——「confirmed」留主会话/用户终审）。
- [x] 数字/路径全部来自任务素材（16:48 / 96a8b9a / build.gradle:111-112 / Mixin:81 / DOM=48 / aebf0f6），无编造。
- [x] 格式与目标文件末尾现状对齐（260919-03 块 / #192 条目均为逐字参照）。
- [x] 未直接改 knowledge/ 或 docs/——草稿落本文件，主会话应用 + 验证。
- [x] open 4 项 / C-1a 未闭合如实标注 🔍，未提前结论化。
