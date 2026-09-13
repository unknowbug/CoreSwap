# knowledge-draft-260913-06 —— 工作块 260913-06 知识库更新草稿（subagent 产出，主会话应用）

> 状态：draft（供主会话应用 + 验证）。价值门已过：候选 1/2/3 = 高价值必记；候选 4（P3 四臂 PASS + fp 逐字节同一快照）= 一次性验证结论，**不进 knowledge/**，只进 10 时间线。
> 编号核对：INDEX 最新 #144（260913-05）；workflow-patterns 与 build-tooling 共享全局序号，新条目 = **#145/#146（workflow-patterns）+ #147（build-tooling）**。

---

## 草稿一：knowledge/discovered/ 追加条目

### 追加到 `knowledge/discovered/workflow-patterns.md`（文件末尾）

````markdown

---

## 发现 #145: 判别实验的「自变量前置核算」新增硬约束实例——vanilla forceload 单次加载上限 256 chunks，region 17×17=289 被服务器拒绝 `Too many chunks`，「零输出零 armed」是驱动未生效签名而非行为判据（260913-06；#20 死参数家族新实例）

- **发现时间 / 发现者 / 置信度 / module**：260913-06；主会话（运行台实测 + 缩区重跑对照）；**candidate**（现象复现 + 修复后四臂全过，confirmed 留人类）；workflow-patterns / 判别实验驱动纪律（**#20「死参数制造假判别——判别实验必须验证自变量真被改变」家族：驱动参数合法性前置核算面**）。
- **来源定位**：`.investigations/sentinel-260913-06/record-260913-06.md` §过程错误 1；运行台 `.tmp/sentinel-260913-06/run_sentinel_{1201,1216}_260913-06.ps1`；失败首跑原始日志灭失（#146 另立），运行时 [result] 行仅存 `cmd-output/arm-summary-260913-06.txt` 转录。
- **五段式（错误优先）**：
  - **现象**：sentinel 跨 region 臂 r2 用 region `[-3072,-2848]²` 对应 17×17 chunk 网格 = **289 chunks**，`forceload add` 后服务器拒绝（`Too many chunks`），该臂 **零输出零 armed**——若按判据表面读，会得出「sentinel 未生效/接管未跑」的错误行为结论。
  - **根因（机制）**：vanilla `forceload` 命令对单次加载量有 **256 chunks 硬上限**，超限请求被整体拒绝——驱动命令未生效 = 自变量从未被改变；此时「零输出」反映的是**驱动失败**，不是被测系统的行为。与 #20 同机制：判别实验的观测面（armed/wb）建立在「自变量真被改变」之上，驱动层静默失败使全部判据失去前提。
  - **定位**：「零输出 + Done 达成 + bridge init 正常」组合先过 #13「探针零输出先查过滤/驱动条件」→ 核对 region 尺寸与 forceload 上限核算（17×17=289 > 256）一步定因；非行为面缺陷。
  - **修复**：region 缩至 **15×15 = 225 chunks** 重跑 → 两执行体 r2 臂 armed=1 / wb=529 / crash=0 全过。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **驱动类命令（forceload/Chunky/探针注入等）使用前 MUST 前置核算其量级上限**——vanilla `forceload` 单次 ≤ **256 chunks**（本项目区尺寸按 15×15=225 或更小取值）；超限拒绝 = 驱动未生效。
  2. **「零输出零 armed」MUST 先过「驱动是否生效」签名判别再立行为结论**——判据优先级：驱动命令回显/拒绝信息 > 行为观测面（#20/#37/#81 家族：生效证据先于行为判据）。
  3. **失败首跑的原始日志也是证据件**——本案失败日志被复跑同标签覆盖而灭失，见 #146。
- **家族索引**：#20（死参数制造假判别——本条为「参数超限被拒」形态）、#13（探针零输出先查过滤/驱动条件）、#37/#81（生效证据必须行为化）、#144 判据 2（forceload 区 ⊆ spawn 预生成的 seed 相关形态——同属 forceload 区选区前置核算域）、#146（失败臂日志覆盖——本条的证据保全面）。

---

## 发现 #146: 「驱动失败臂」的日志覆盖变体——失败/作废臂复跑同标签同样覆盖首轮日志，判据 1「复跑前归档」的适用面是**所有臂**而非仅成功臂（260913-06；#144 判据 1 延伸）

- **发现时间 / 发现者 / 置信度 / module**：260913-06；主会话（运行时归档核对）+ judge subagent（J5 小缺口 + C3 SHOULD 抓出）；**candidate**；workflow-patterns / 证据链完整性（**#144 判据 1 的「臂类型」扩展**）。
- **来源定位**：`.investigations/sentinel-260913-06/record-260913-06.md` §过程错误 1 证据面（C3 声明）；judge `review-260913-06-judge.md` J5 + C3；运行台 `.tmp/sentinel-260913-06/run_sentinel_*.ps1`（RTag 入标签）。
- **五段式（错误优先）**：
  - **现象**：R2 失败首跑（289>256，`Too many chunks`）的原始日志已不存在——复跑（缩 225 后）使用**同一臂标签（RTag 同值）** ⇒ 日志路径同文件，失败首轮被成功复跑**静默覆盖**；运行时仅存 `cmd-output/arm-summary-260913-06.txt` 的 [result] 行转录，原始日志灭失。
  - **根因（机制）**：#144 判据 1 建立时的心智模型是「成功臂复跑覆盖成功首轮」；但运行台日志路径按臂标签固定的机制**不区分臂成败**——**失败臂同样是「某一轮的日志」**，复跑（无论是修复后重跑还是换参重跑）同样触发覆盖。失败轮日志恰恰记录「驱动为什么没生效」，是判错链条的一手证据，灭失代价更高。
  - **定位**：judge C3 独立核对「失败首跑是否有证据件」→ 无；record 已按 §16.3 姿势补 C3 声明（运行时转录为唯一存留面）。
  - **修复**：本块 = record 补证据面声明（叙述记录、无证据件，judge 判 SHOULD 不阻断）；机制面 = 运行台标签设计改进登记——**失败臂也应换标签或先归档**。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **#144 判据 1 的适用面扩展为：凡将被复跑/覆盖的轮次日志——成功臂、失败臂、作废臂一视同仁——MUST 在任何复跑之前归档（或换标签）**。失败轮日志记「为什么没生效」，判错价值高于成功轮，灭失代价更高。
  2. **运行台标签设计 SHOULD 天然抗覆盖**：臂标签带轮次序号（`r2-attempt1/2`）或时间戳，使覆盖在机制上不可能发生，而非靠纪律记忆。
  3. **失败臂的最低保全面 = 驱动命令回显/拒绝信息 + [result] 行转录**（本案 arm-summary 转录即此档）；叙述性记录 + 无证据件时，record MUST 按 #144 MUST-1 姿势补证据链声明。
- **家族索引**：#144 判据 1（复跑前归档——本条为其**臂类型扩展**：失败/作废臂）、#88（「复跑采集必删 world」对偶面）、#118（判据输入集合可审计——失败轮日志属判错链输入）、#145（本案失败轮的成因条）。

---

### 追加到 `knowledge/discovered/build-tooling.md`（文件末尾）

````markdown

---

## 发现 #147: 程序化修改 PowerShell 脚本后 MUST 用 `Language.Parser::ParseFile` 核语法——正则/字符串替换引入杂散反引号 + 末参数插参漏逗号两连犯；`[scriptblock]::Create` 式自查会吞错假 OK（260913-06）

- **发现时间 / 发现者 / 置信度 / module**：260913-06；主会话（脚本副本生成两连翻车 + Parser 实核）；**candidate**（修复后 PARSE-OK + 四臂全跑通，confirmed 留人类）；build-tooling / PowerShell 脚本生成坑（#45/#49/#41 同文件家族的**脚本生成面**）。
- **来源定位**：`.investigations/sentinel-260913-06/record-260913-06.md` §过程错误 2；运行台 `.tmp/sentinel-260913-06/run_*.ps1`（母本副本 + 程序化变更，变更登记见 judge J6——diff 恰为登记项，机制面见本条）。
- **五段式（错误优先）**：
  - **现象**：从母本脚本程序化生成运行台副本（正则替换 out 目录 / 在命令末参数插入新参数）连续两次产生**语法级损坏**：① 正则替换引入**杂散反引号**（行尾续行符残留/误置 ⇒ 后续语句被续行吞并）；② 末参数后插入新参数**漏逗号**。且自查用 `[scriptblock]::Create` 包装脚本文本——**报「成功」但错误仍在**（假 OK）。
  - **根因（机制）**：① 文本层正则/字符串替换对 PowerShell 语法**零感知**——反引号是合法续行/转义字符，替换边界落在反引号附近即产生语法损坏，且损坏点常在替换点之外（静默扩散）；② 末参数插参靠字符串拼接，逗号是纯文本，漏掉不报任何警告；③ `[scriptblock]::Create` 将脚本编译为 ScriptBlock 时**解析错误以异常抛出与否取决于调用方式/PS 版本语义**，作为「自查」它不是语法门——**吞错或错位报错，给人已校验的错觉**。
  - **定位**：运行报 ParserError 后逐行核对生成脚本 diff；改用官方解析器实核后一次定位两处损坏。
  - **修复**：以 `[System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$null, [ref]$errors)` 实核，`$errors` 非空即拒收——修复后 PARSE-OK 才入运行台。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **程序化生成/修改 .ps1 后，MUST 跑 `Language.Parser::ParseFile` 并断言解析错误数为 0**（一行成本），再交付执行；「生成成功 + 文件存在」零证据力。
  2. **`[scriptblock]::Create` 不是语法自查门**——会吞错/假 OK；语法门只用 Parser API（`ParseFile`/`ParseInput`，取 `[ref]` errors）。
  3. **文本层替换改脚本时，替换点之外的语法损坏是常态风险面**（反引号续行/引号配对/逗号）——凡对**要执行的脚本**做程序化变更，Parser 门是唯一可靠防线；judge 类逐行 diff（J6）核对的是「改了什么」，Parser 核的是「改完是否还是合法 PowerShell」，两者互补、都不可省。
- **家族索引**：#45（`-like` 字符类坑）、#49（`-File` 多值参数）、#41（管道早退截断日志）、#34（Move-Item 静默改名）——同文件 PowerShell 坑家族；#62（「生成/继承的成功 ≠ 合法/生效」同构：编译过不构成接线证据——本条为「文件生成了不构成语法合法」）。
````

**归属理由（一行）**：
- #145 → workflow-patterns：核心价值是「判别实验自变量前置核算 + 零输出先查驱动生效」的可复用判据（#20 家族），forceload 256 只是实例载体。
- #146 → workflow-patterns：核心是证据链完整性纪律（#144 判据 1 的适用面扩展），非工具实现坑。
- #147 → build-tooling：核心是 PowerShell 脚本生成/校验的工具链坑与机械防线（Parser API），归属工具坑文件。均标注**详写**（高价值·错误优先五段式）。

---

## 草稿二：knowledge/INDEX.md 追加行（追加到文件末尾，风格对齐现有「> YYMMDD-N 追加」块）

> 260913-06 追加（sentinel 跨 region 四臂 + BULKWB 翻转后 nether/end 默认路径行为门）：workflow-patterns 新增**发现 #145（错误优先）**（判别实验驱动参数**前置核算**硬约束实例——vanilla forceload 单次上限 **256 chunks**，region 17×17=289 被拒 `Too many chunks`，「零输出零 armed」= 驱动未生效签名而非行为判据，#20 死参数家族新实例）+ **发现 #146（错误优先）**（**失败/作废臂的日志覆盖变体**——复跑同标签同样覆盖失败首轮日志，#144 判据 1 适用面扩展为「凡将被覆盖的轮次日志，成败一视同仁 MUST 先归档或换标签」）；build-tooling 新增**发现 #147（错误优先）**（程序化改 .ps1 后 MUST `Language.Parser::ParseFile` 核语法——正则替换引入杂散反引号 + 末参数插参漏逗号两连犯，`[scriptblock]::Create` 式自查吞错假 OK）。来源：`.investigations/sentinel-260913-06/`（record + judge PASS-with-conditions）。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260913-06 块。

（另：P3 四臂 PASS + 翻转后 nether/end fp 逐字节同一为一次性验证结论快照，按记录价值门不进 knowledge/，只进时间线。）

---

## 草稿三：versions/1.21.6/docs/10-timewise-archive.md 260913-06 时间线块（追加到文件末尾）

## 260913-06（实际 2026-09-13 晚，Get-Date 锚 21:58）：BULKWB 翻转后 nether/end 默认路径行为门（#141 范式）+ sentinel 跨 region 泛化（四臂）—— **candidate**（judge PASS-with-conditions 条件已应用；confirmed 待用户授予）

> 过程产物 `.investigations/sentinel-260913-06/`（`record-260913-06.md` 主记录 + `review-260913-06-judge.md` + `cmd-output/` 归档：6 份日志(.log/.err) + fp 六件 + fp-sha + arm-summary 转录 + 3 运行台脚本副本 + MANIFEST 含脚本 sha）+ `.tmp/sentinel-260913-06/`（运行台与原件，不入库在盘可核）。上游：260913-01 HOOK-C 翻转（验证臂仅 overworld）+ 260913-05 sentinel 跨 seed（单 region）。通用模式 → workflow-patterns **#145/#146** + build-tooling **#147**。

- ✅ **P2 翻转后 nt/en 默认行为门（两臂全 PASS）**：`postflip-{nt,en}-default`（不带 property，仅 `-Dcoreswap.bulkwblog=1`）：路径已切换（`[WG-BULKWB] calls=625` / `[WG-PERBLOCK] calls=0` 正/负成对）+ 产物不变（fp 全 64 位 sha + 尺寸与 d4i-260913-01 基线**逐字节同一**；content/wb 层亦同一）。end 维确定性指纹（#21，end=0）下零差异。**#141 两断言齐备** ⇒ 翻转后三维默认路径 = bulk 且产物不变（ow 由 260913-01 承载）。
- ✅ **P3 sentinel 跨 region 四臂全 PASS**：判据 armed≥1 / wb>0 / crash=0；r1 [2048,2303]²（256 chunks，wb=576）/ r2 [-3072,-2848]²（225 chunks，wb=529），两执行体 × 两 seed 行为化自证一致。
- ❌→修正 **过程错误 1：R2 首跑 region 17×17=289 > forceload 256 上限**——`Too many chunks`，零输出零 armed = 驱动未生效签名（#20 家族）；缩 15×15=225 重跑全过。失败首轮日志被同标签复跑覆盖灭失（#146；C3 声明：仅存 arm-summary 转录）。
- ❌→修正 **过程错误 2：脚本副本生成两次语法翻车**（杂散反引号 + 插参漏逗号；`[scriptblock]::Create` 自查吞错假 OK）——修复后 `Language.Parser::ParseFile` 实核 PARSE-OK（#147）。
- ✅ **judge（隔离 subagent，三源核对）= PASS-with-conditions，条件已应用**：C1（MUST）= P2/P3 六臂 [result] 汇总转录补归档 arm-summary（已做）；C2（MUST）= suspExc 运行时计数 3/4 无法从归档 log 复现 → record 已修正为「可复现异常面 = 各 .log.err 各 1 行 rubygrapefruit error=5（gradle watcher 良性）」，运行时计数不作判据引用；C3（SHOULD）= 失败首跑留档 + MANIFEST 补脚本 sha（后者已做，前者以声明替代，机制面立 #146）。J1-J6 全 PASS；suspExc 归档不可复现面与「盘上 hash 实核」列入 judge 无法核查面显式声明。
- ⚠️ **§9.7**：P2 仅 1.21.6 执行体（翻转只在其臂）；ow 未复跑（260913-01 承载）；P3 每 (执行体, seed) 组合 2 region，不声称全 region 泛化；fp 口径 = 排序域多重集判据（#142 窄化声明）。
- 状态：**candidate**（judge 推荐），confirmed 待用户授予。提交号：（占位——待主会话提交后回填 commit hash）。
