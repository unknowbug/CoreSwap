# judge 审查意见 — legacy run3 收敛性补证（260918-11）

- 审查角色：core.judge（subagent，隔离）——只出意见，不改任何 status；confirmed 留人类。
- 审查对象：verdict-260918-11.md（draft）/ criteria-260918-11.md（pre-registered）/ cmd-output/*（lr3b + lr3a 失败轮留档）/ errors-260918-11.md（E1）/ 被补证对象 verdict-260917-04.md §2。
- 审查日期锚：文件系统证据 + 判据/verdict 交叉核对；git 面核对列为主会话执行项（本 judge 无 shell）。

## 结论：PASS-with-conditions

verdict 草稿结论边界如实、判据对照与落盘证据自洽，建议升 n=2 candidate 的措辞未越界（未触 confirmed）。但产物契约有一处 MUST 缺口（index.yaml），另有若干 SHOULD/INFO。

## 逐项审查（按任务书清单）

### 1. 判据预登记完整性（#112）— 通过（附主会话核对项）

- criteria front-matter 带 `status_gate: pre-registered`，时序锚声明 commit 0e8083c @2026-09-18 22:02:35。
- 时序自洽性（judge 侧可核部分）：lr3b.log 首 boot lightInit 22:09:27 / Done 22:09:38，均晚于声明 commit 时刻 22:02:35；lr3a region_min_mtime 1789740485 < lr3b 1789740845，且两者均 > rmtree epoch 1789740532（≈22:08:52），与「lr3a 先跑 VOID → 修脚本 → lr3b 重采」叙事一致。
- **主会话执行项（git 面）**：① `git log -3 --oneline` 确认 0e8083c 存在且提交时间 = 2026-09-18 22:02:35（本 judge 无 shell，未核）；② `git status`/diff 确认本块 src 零改动（dll_sha 6f7fa3ae 与 260917-04 同一执行体的声明依赖此项）。
- 判据前置集形态：§0 以 key/expected/check 列出 SELFCERT 前置集（§15.1 精神达标）。`preconditions:` 独立 front-matter 字段未加——该项在 AGENTS.md 仍标「待办」，本块不判违规，记 INFO-2。

### 2. lr3a VOID 处置合规性（#118 / #144-146）— 通过

- 失败轮完整留档：cmd-output/ 内 lr3a.log + lr3a_result.json + lr3a_light_run{1,2,3}.json 均在（#144 留档成立）。
- 不挑臂：verdict §3 明示「lr3a 的 changed=0/warmup=158 不计入本块结论」，换标签 lr3b 重采（#146 换标签形态，无自动回退）；lr3a_result.json seed_in_leveldat=false 与 VOID 理由一致。
- E1 五段式（errors-260918-11.md）：现象（测量面全绿但门 miss，rc=2）/ 根因（gzip(NBT) 载体直读）/ 定位（解压后 offset 305 手工命中）/ 修复（gzip.open，脚本 ：130 注释落点）/ 教训（判据语义≠载体读取层两层）五段齐全 + 速查表——符合 SUBAGENT-KNOWLEDGE-GUIDE §二。
- 脚本修复先于 lr3b 采集：run_legacy_run3.py:129-131 已是 gzip.open 且注释引 lr3a 实证，时序自洽。

### 3. world 身份项（#161）三项证据链 — 通过（附 INFO-1）

- ① seed BE 命中：lr3b_result.json `seed_in_leveldat=true`；② region mtime 1789740845.58 > rmtree epoch 1789740532（差 +313s，落在首 boot Done 22:09:38 之前，物理自洽）；③ common=2025 ≥1000（主会话已独立复核 2025×3）。
- **INFO-1（落点 errors-260918-11.md 或 verdict §2 一行）**：门实现为 `seed_be in f.read()` **包含性子串匹配**而非结构化 NBT 定位（脚本 ：131）；「offset 305 精确命中」是 E1 定位时的一次性手工核对（check_seed*.py），门本身不做位置校验。8 字节 BE 模式误撞概率可忽略，判定维持，但 gates 语义与判据表述（「level.dat 含 seed BE int64」）存在精度差，建议在 E1 教训段补一句登记，防后续照抄该门形态时高估其严格度。
- **SHOULD-2（落点 lr3b_result.json 或 criteria §0）**：rmtree epoch 值（1789740532）只存在于脚本运行时输出与判据叙述，未进任何落盘 result/criteria 字段——`region_min_mtime_gt_rmtree` 判定不可第三方复算。建议 result.json 增 `rmtree_epoch` 字段或 criteria 补录实测值（lr3a/lr3b 各一）。

### 4. §9.7 口径声明三要素 + E1 等价档位 — 通过

verdict §5：载体（snap_light region 快照、与 260917-04 同工具谱系）/ 覆盖面（单 seed 单 spawn 区域 × Done+60s × 3 boot）/ 不可比声明（存档写入口径、域批形态、vanilla）三要素齐；等价档位 = E1（同构建态、同载体、单变量 = seed）+ 共享观测 key 集 S（两轮 common chunk 快照面，各 2025）已声明；#162「计数恒等≠集合恒等」自检（changed_set=[] 空集证据）在案。与 260917-04 §2 的 E1 口径同层，可比性声明成立。

### 5. verdict 结论边界如实性 — 通过

- 措辞核：verdict 全文未出现「全条件收敛/整体收敛」断言；§4.3 与 §5 双重边界（单区域 × Done+60s × 3 boot + 「不得外推至 legacy 光照整体收敛或域臂」）。「收敛（=0）」措辞沿 verdict-260917-04 §2 原结论既定表述，且被 §5 栅栏围住——未越界。
- 升级建议边界：§4.2 仅建议「n=1 candidate → n=2 双 seed candidate」，明确 confirmed 留人类——符合本 judge 角色契约与状态机。
- 一处措辞精确化（INFO-3，落点 verdict §4.1）：「判据 §4 分叉候选②被旁证削弱」表述准确（warmup=153 只削弱「假 0/检测窗」解释，不排除），维持现文可接受，无需改动。

### 6. worker 建议「#151 追加实例引用行，不新增条目」— 符合记录价值门

E1 的可复用资产是判错方法层：「自证门的载体读取方式与判据语义是两层 + 部分成功掩盖全流错位」——该签名已由 knowledge/discovered/build-tooling.md 发现 #151（260914-04）承载，且同文件后续 NBT 解析条目（:1338，#36 家族 + #151 引用）已在同族延伸。E1 是既有家族在「自证门/压缩载体」维度的**实例**，非新判错签名——按价值门（§〇：一次性实例不新增条目）处置为**向 #151（或其 NBT 同族条目）的家族索引追加一行实例引用**正确；不新增 discovered 条目、不写主题篇。附加约束：该追加行属结论性 discovered 写入，MUST 走「subagent 产草稿 → 主会话应用」链（AGENTS 九），落点随 verdict 定稿一并执行。

### 7. 「worker 未跑 shell」缺口的覆盖 — 足以覆盖（本 judge 侧补一轮独立 grep）

- 数值层（changed=0 / common=2025 / 2025×3 key 重数）：主会话已从快照独立复算——覆盖。
- SELFCERT 层：judge 本次对 lr3b.log 独立 grep 复核：`Done (`×3（22:09:38 / 22:11:19 / 22:13:00）、`lightInit ok`×3、`sha256=6f7fa3ae0169fb5c`×3（size 2474496）、`[LIGHT-DOMAIN]` 0 行、`fallback` 0 行——与 lr3b_result.json per_boot 逐项一致，verdict §2 的「独立 grep 复核」声明属实。
- 脚本越权出码路径：run_legacy_run3.py VOID 出码点（:170-173）仅 world 身份两项，无「测量面绿即放行」旁路——静态核对成立。
- 判定：worker 无 shell 的缺口已被「主会话数值复算 + judge SELFCERT grep + 脚本静态核对」三面覆盖，不构成保留条件。

## 产物契约与状态机核对

- status 合法：verdict draft / criteria draft，无 confirmed 自授——合规。
- 落盘：.investigations/ 链（criteria + cmd-output + errors）齐。
- **MUST-1：`.artifacts/legacy-run3-260918-11/` 缺 index.yaml**——core.artifact 契约要求结论产物登记主索引（当前该目录仅 verdict 一件）。verdict 定稿（或升 candidate）前 MUST 补登记。
- 噪声卡：未检索到本载体（legacy snap_light 链）未解决噪声卡关联；dll_sha 6f7fa3ae 与 260917-04 同执行体跨块复用，criteria §0 已按 #77 声明 mtime 复核——合规。
- retry cap：lr3a→lr3b 一轮修复即成，无饱和问题；E1 修复属工程修复不计数——合规。
- 副作用与逆（§9.8）：每臂 rmtree 重建 world 属采集前寄存副作用，逆 = 换标签重采 + region freshness 门（#144/#146 形态），verdict §3 已声明——合规。
- 模块边界：verdict 未引用跨模块 skill 正文——合规。

## 条件清单

**MUST**
1. 补 `.artifacts/legacy-run3-260918-11/index.yaml` 登记 verdict（core.artifact 契约；升 candidate 前完成）。
2. 主会话执行 git 面核对：① 0e8083c 存在且 commit 时间 = 2026-09-18 22:02:35（先于 lr3b 采集 22:08:52+）；② 本块 src 零改动 diff。任一不成立 → 本意见作废重审。

**SHOULD**
1. lr3b_result.json（或 criteria §0）补录 rmtree epoch 实测值（lr3a/lr3b 各一），使 world 身份判定可第三方复算。
2. verdict 定稿时把 verdict §4.2 的 n=2 升级建议同步登记为对 verdict-260917-04 §2 的**追加补证指针**（原正文不删不改，§15.4 精神；当前 §2 状态行仍写 n=1，建议由主会话以一行追加方式补 n=2 指针）。

**INFO**
1. E1 教训段补一句：seed 门实现为包含性子串匹配，offset 305 为一次性手工核对，门严格度低于判据表述。
2. 判据 artifact 增 `preconditions:` front-matter 字段属全项目待办（AGENTS v0.22 宿主侧待办③），本块不强制，后续块统一落。
3. verdict §4.1「分叉候选②被旁证削弱」措辞可保留（准确），无需改。

## 推荐状态

- verdict-260918-11：MUST-1 完成 + git 面核对通过后，**建议 candidate**（n=2 双 seed 扩 n 结论成立，边界声明如实）。
- verdict-260917-04 §2「legacy 干净链 run2→run3 收敛（=0）」：**建议升 n=2 candidate**（SHOULD-2 落地后）；confirmed 留人类拍板。

—— judge（core.judge subagent，260918-11；本意见不构成任何 status 变更）
