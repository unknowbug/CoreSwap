# 10 时间线追加草稿：2026-08-15（下午段）知识库流程改进——错误记录强化方案 C
# core.worker 产出（2026-08-15）· 仅供主会话应用，不修改正式知识库
# 数据源：.investigations/000-架构设计/framework-sync-request-error-recording.md + knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md + AGENTS.md §九
# 追加位置：versions/1.20.1/docs/10-timewise-archive.md 末尾（2026-08-15 上午段「G4 编译时间修复」条目之后）

---

## 2026-08-15（下午段）：知识库流程改进——错误记录强化方案 C 落地 + RE-Framework 同步申请（✅ 项目侧已落地 / 🔍 框架侧待评估）

> 承接 2026-08-15 上午段 D22 条目（kb-draft-d22.md 产出——实证「草稿质量靠 prompt 显式要求兜底，非 skill 自动保证」）。用户提问「知识库 subagents 的 skills 有没有写明错误记录要求」→ 主会话核对 core-knowledge skill → 拍板方案 C → 三文件落地；转交材料落盘 `.investigations/000-架构设计/framework-sync-request-error-recording.md`，供 RE-Framework 维护侧评估框架层同步。流程改进类条目（非错误结论），记录 触发→诊断→决策→落地 全链。

### 触发（用户提问）

- 用户问：「知识库 subagents 的 skills 有没有写明错误记录要求」→ 主会话核对 core-knowledge skill（框架通用层，项目副本 `E:\PYTHON\CoreSwap\.dsh\skills\core-knowledge`）。

### 诊断（skill 内容核对——通用层与项目级要求的缝隙）

- **skill 通用层已有基线**：「错误 > 正确」原则（错误链条先写、已排除不删、INDEX 置顶）+ 错误账本条目格式（`knowledge/errors/error-NNN-*.md`，四段式：错误现象 / 诊断过程（含结论根因）/ 排除后的正确认识 / 诊断方法论沉淀）。
- **缺项目级强化三处**（缝隙 = 详实度 / 载体 / 判错经验未达项目要求）：
  1. **五段式 vs 四段式**：项目要求「现象→根因→定位→修复→教训」五段完整（AGENTS.md 三-2、2026-08-13 用户明确）；skill 四段式**无独立「修复（改了什么）」段**，且未写「不得只记『已修复』而不记『为什么错』」。
  2. **判错经验沉淀**：项目要求「符号级错误一定是结构错不是精度错，先查公式/索引/坐标，别在精度上纠结」类**可复用判错方法必须沉淀**（比单条错误更有价值）；skill 仅有通用「诊断方法论沉淀」段（下次遇到类似症状 → 第一步做什么），**未强化到项目级 MUST 强调度**。
  3. **载体写死**：skill 固定 `knowledge/errors/error-NNN-*.md` 独立文件（每条一个文件）；项目实际载体 = `.investigations/perf-rework/gpu-accel-errors.md` 等**独立成篇 + 末尾「错误→根因」速查表**（A-G/D 系列一个文件）——skill 未说明「项目可自定义错误台账载体」。
- **为什么这是问题（实证）**：错误优先原则项目早有、skill 有通用版，但 2026-08-15 上午 D22 草稿质量达标是靠派知识库 subagent 的 prompt **显式要求**「按现象→根因→定位→修复→教训格式（参照 D21）」兜底（kb-draft-d22.md 即产物）——**不是 skill 自动保证**；每次派 subagent 都需人肉强调，漏一次即退化。

### 决策（方案 C 拍板）

- 主会话给出三个候选方案，用户拍板 **方案 C**：新建项目级规范文件承载强化（方案 C 内容见转交材料 §二；被否方向的核心顾虑：仅靠 prompt 兜底不可靠——本次实证；直接改只读框架 skill 越界）。
- 方案 C 三件套：① 项目级规范文件 `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`；② AGENTS.md §九新增「错误记录强化」强制行；③ 转交材料供 RE-Framework 维护侧评估框架层同步。

### 落地（✅ 三文件）

- **`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（新建，项目级知识库产出须知，68 行）**：
  - 一、错误优先原则（错误 > 正确、被排除假说保留、判错经验尤其要记）；
  - 二、五段式格式表（现象/根因/定位/修复/教训 + 反模式三条：只写已修复 / 现象=根因 / 无定位过程）；
  - 三、知识库载体映射（错误台账 → gpu-accel-errors.md 等独立成篇 + 速查表；结论 → 01-09 主题篇；过程 → 10 时间线；通用 → discovered/）+ 载体纪律；
  - 四、产出检查清单 10 项（subagent 交付前自检）；
  - 五、与 core-knowledge skill 关系（冲突时项目级文件优先，同 AGENTS.md 优先级规则）。
- **AGENTS.md §九「知识库更新强制触发点」新增「错误记录强化」行**：派知识库 subagent 的 prompt MUST 包含一行 `先读 E:\PYTHON\CoreSwap\knowledge\SUBAGENT-KNOWLEDGE-GUIDE.md，按其中格式与载体要求产出草稿`，并写明理由（skill 通用层无「不得只记已修复 / 判错经验沉淀 / 项目自定义错误台账载体」三处强化，靠 prompt 兜底不可靠——2026-08-15 实证 D22）。
- **`framework-sync-request-error-recording.md`（转交材料，.investigations/000-架构设计/，44 行）**：背景（三处缺口 + D22 实证）→ CoreSwap 侧落地（方案 C）→ 建议框架层同步（core-knowledge skill 增「项目级错误记录强化（可选适配）」节：五段式、不得只记已修复、判错经验、载体灵活、被排除保留）→ 同步边界建议（框架保持通用基线；项目级强化归项目侧文件，框架提供「可被项目覆盖」说明；若框架内置五段式，建议把 skill「诊断过程」段改/补为「定位（诊断方法/工具）」+ 加「教训（可复用判错经验）」段对齐）。

### 🔍 框架侧待评估（RE-Framework 维护侧）

- 转交材料已就位，待 RE-Framework 维护侧评估是否在 core-knowledge skill / 模板层同步增强（五段式、判错经验、载体灵活、同步边界四条建议）。
- 项目侧已闭环 ✅：规范文件 + AGENTS.md 强制行 + 转交材料三件套完成；后续派知识库 subagent 的 prompt 一律带「先读 SUBAGENT-KNOWLEDGE-GUIDE.md」行（AGENTS.md 九强制，随 todo 预置纪律同款）。
