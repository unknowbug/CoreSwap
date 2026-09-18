# B6-3 S4 能力演示 judge 审查意见（260918-07）

- 审查角色：core.judge（subagent，只出意见，不改任何 status；confirmed 留人类）
- 审查对象：`record-260918-07.md` + `.tmp/260918-07/b63_comparator_v2.py`（对 v1 备份基线）+ 架构计划判据 + 声明表 declaration-light-260918-06 §五门控总表
- 三源基线：① 交付文件（v2 比对器 + 三夹具 + record，逐文件直读）② v1 基线 `backup/b63_comparator.orig.py`（sha256 实测复核 = `7811A5B4…DC5FD`，与 record 记载一致）③ 验证记录——judge 亲手复跑四测 + v1 基线正测（本轮 2026-09-18，python 实跑，输出全文在本轮工作上下文，实测数值已逐项对照 record 表）

---

## A. 判据预登记一致性 —— ✅ 通过

- 时间戳链：架构计划 20:07:24（「期望值先于执行登记」载体）→ v2 比对器 20:10:20 → 三夹具 20:10:32 → record 20:11:37。判据登记先于测试执行，无事后改判据迹象。
- 四测与计划 §验证方式对应：正测（pbeta05c rc=0 不回归）= 计划正测；NEG-A（前提注入假值 → rc=1 + suspended 域指出）= 计划负测①；NEG-B（枚举外键 → rc=1）= 计划 §#182 行；NEG-C 为计划「负测 ≥2 例」之外的**加测**（方向与 S4 能力主张一致，非改判据）。
- judge 复跑逐项对照：POS rc=0 / actual 8 / touched 8 / suspended 3 ✅；NEG-A rc=1 / suspended 16 / SUSPENDED-TOUCHED 恰 7 键 ✅；NEG-B rc=1 / UNKNOWN-FACT-KEY + 封闭集全文 ✅；NEG-C rc=0 / suspended 16（唯 sysprop 域可达）/ touched 0 ✅。实测与 record 表格**逐格一致**。
- 小瑕疵（不设条件）：record NEG-A 写「逐键列出」但 record 本体只举 3 例概括，7 键全列只在比对器输出中——表述与落盘载体轻微错位，实测已由 judge 补足复核。

## B. suspended 语义正确性（Anchorlaw v0.22 §15.1 / #160）—— ✅ 通过

- 前提失效 → 域判 `[SUSPENDED]` + 显式 failed 三元组（want/got 逐条），**suspended 本身不置 rc**（POS 中 3 个 suspended 域照常 rc=0）——「suspended ≠ FAIL、状态不自动变更」语义成立。
- SUSPENDED-TOUCHED FAIL 语义成立：前提失效域被证据触达 = 声明与事实矛盾，属 S4 要求的「不可达前提机械检出」，NEG-A 实测命中。此处 FAIL 判的是**矛盾**而非 suspended 本身，与 §15.1 不冲突。
- 退出码机械断链：rc=1 仅由 UNKNOWN-FACT-KEY / UNDECLARED / SUSPENDED-TOUCHED 三 FAIL 通道置位，逐通道独立累积，无静默吞并（#160 机械信号形态）。
- 前置失效传播（premise-expired）：本工具为单件静态核对器、无下游判据依赖者，域内不适用——如实，不算缺口。

## C. 值域封闭性（#182）—— ✅ 通过，一处潜在旁路见 N2

- 未知事实键：注入侧 `env_facts` 键 ∉ FACT_KEYS（6 键封闭集）→ UNKNOWN-FACT-KEY + rc=1，NEG-B 实测验证。
- 无 silently-green 恒绿通道：NEG-A（矛盾检出）与 NEG-C（全 suspended 不误报 PASS）分别堵死「恒 FAIL 不可能」与「恒 PASS 不可能」两个方向——负向测试能失败，判据有消解力。
- N2（潜在旁路，条件）：`check_preconds` 的 `<missing>` 防御分支（:120-121）只产生 SUSPENDED 而非 FAIL——若未来 PRECONDS 引入了封闭集外的键，该域会**静默 suspended**而非触发封闭性 FAIL。当前 5 组 PRECONDS 的键全部 ∈ FACT_KEYS（judge 逐一核对），故现版无实际漏洞，但这是依赖「两处手写集合保持同步」的隐性约束，建议加一行断言（PRECONDS 键 ⊆ FACT_KEYS）把隐性约束变机械约束。

## D. 声明表对齐 —— ✅ 基本对齐，一处不完整见 N1

- `sysprop.light.*` 域显式空前提（类加载即消费，任何 mask 可达）✅ = 声明表 §一 :77-133 行；NEG-C 实测唯它保留可达。
- betadump 双探针门 `[betaprobe.armed ∧ betadump.armed]` ✅ = §五 :698 一手判据（NEG-A/C 输出逐键双前提）。
- LIGHT_RUST 缺省关 → DEFAULT_PRECOND = `light.rust.armed=true` ✅ = §五总门行；未见 armed=false 时误判可达。
- N1（对齐缺口，条件）：`palette-private-record`（paldump）域前提只挂 `paldump.armed`，**漏 `light.rust.armed`**——声明表 §一 LightPalDump 行明写「paldump>0 **且 LIGHT_RUST 开**」（消费点 mixin:279/:757 均在 LIGHT_RUST 臂内）。方向上这是**检出力偏弱**（欠 suspend → 漏报 SUSPENDED-TOUCHED 的可能），不会误报；且现役证据源 pbeta05c 无 paldump 行，本块四测结果不受影响。但既然 v2 的卖点就是「前提与声明表逐域对齐」，此处应补齐第二前提后重跑四测（预期四测结果不变，可快速回归）。

## E. v1 回归 —— ✅ 通过

- judge 实跑 v1 备份基线：actual 8 / declared 17 / touched 8 / rc=0，事件计数 `enter ×1587 / abi=packed ×1587 / path=legacy-rust ×1587 / chunk( ×1587`，与 260918-06 judge 复跑口径及本块 record 完全一致；v2 POS 同 log 同数值 → POS 测触达 8 ⊆ 声明 17 与 v1 行为等价（E1 同构建态同证据源单变量）成立。
- 附注（不设条件，v1 继承项）：ALIASES `path=domain` 映射的 3 个 `wgLightDomainWriteBack` 键不在 DECLARED 17 键集内——若未来证据源为 domain 臂，将 UNDECLARED FAIL。这是 260918-06 S2 对齐后的已知形态（域臂证据尚未出现），record 未提及；不属本块引入，登记为下块边界备忘。

## F. 诚实声明充分性 —— ✅ 通过

- 边界三条准确：① mixin:644 / bin-diag 不在求值器域内——正确（前者是状态机不可达非门控形态，后者是构建产物结构不可达，均非「门控关闭」类，不外推属实）；② vanilla-min.log 合成载体、NEG-C 只证能力不证生产事实——正确（83 字节两行合成日志，judge 已直读）；③ 封闭集 6 键为 light 面本轮封闭、扩面须先扩集——正确。
- 无夸大：record 未声称「三条人工标注全部机械化」（实际只覆盖③类 + 整面总门形态），与声明表 §③ 第 1/2 条保留人工静态的边界一致；S4 原文要求的正是「门控关闭不可达」机械检出，覆盖面主张与悬置条件**精确对位**。
- 验证分层 Partial + #172 注入式范式声明如实（注入 → 命中 + rc=1 → 夹具为新建 derived 件）；§9.8 副作用声明属实——judge 核对全部产物为 20:10 新建文件，pbeta05c.log 只读（mtime 未动），零 in-place 副作用。

## G. 静态自检 —— ✅ 通过（工具级）

- 异常路径：日志/夹具文件缺失、JSON 损坏 → Python 未捕获异常 traceback + 非零退出——对一次性核对工具属可接受机械失败（无吞异常、无假绿）；无空容器/索引越界路径（`facts.get` 语义由显式 `in` 判断替代）。
- `bool(facts[k]) != expected` 对布尔事实键正确；`env override` 后输出逐键标注覆盖关系（含 log 侧原值）——审计可追溯性好。
- 编码：`sys.stdout.reconfigure(utf-8, errors=replace)` + io.open 显式 utf-8，GBK 控制台坑已防。

---

## 其他 judge 清单快核

- status 合法：record 自标 draft/candidate，无 confirmed 越权 ✅。
- 判据 preconditions 字段（v0.22 待办）：v2 是其**工具侧**落地形态；record/架构计划文档侧未加 `preconditions:` 字段——判据 artifact 规范化仍属全局待办，不在本块条件内。
- 模块边界：无跨模块 skill 正文引用 ✅。retry cap：零逆向假设轮次，不适用 ✅。
- 备份纪律：#144 覆盖前归档完成且 sha256 实测吻合 ✅（备份 mtime 19:54 早于架构计划批准 20:07——归档先于批准纯为动作顺序，无内容影响，仅记录）。

---

## 总判定：**PASS-with-conditions**（2 项，均不推翻四测结论与 S4 能力主张）

- **N1（D，建议收口前补）**：paldump 域 PRECONDS 补 `("light.rust.armed", True)` 第二前提（对齐声明表 §一「paldump>0 且 LIGHT_RUST 开」），补后重跑四测确认结果不变（预期 rc 与计数均不变）。当前为检出力偏弱方向的缺口，不产生误报。
- **N2（C，防御性）**：比对器加一行机械断言 `all(k in FACT_KEYS for pre in PRECONDS.values() for k, _ in pre)`，把「PRECONDS 键 ⊆ FACT_KEYS」从手写同步约束变为封闭性 FAIL/断言（消除 `<missing>` 静默 suspended 旁路）。

备忘（不设条件）：① record NEG-A「逐键列出」措辞与落盘粒度对齐；② domain 臂证据源触达时 DECLARED 缺 WriteBack 键的 v1 继承盲区，建议随第二接管面扩面一并处理。

推荐状态：**建议 candidate**——S4 悬置条件（「机械检出不可达前提」能力演示）已由 NEG-C（不可达面机械复现）+ NEG-A（矛盾检出判别力）双向成立，且 POS 无回归、边界声明诚实；N1/N2 为工具加固项，完成属机械动作。confirmed 留用户拍板。
