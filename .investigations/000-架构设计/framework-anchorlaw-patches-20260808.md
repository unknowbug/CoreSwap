# RE-Framework / Anchorlaw 并入补丁（2026-08-08，基于 CoreSwap 8576-24blocks 任务实证）

> 用法：按【位置】定位到目标文件对应章节，粘贴【新增文本】即可。所有建议均有本轮任务实证（见各条「实证」）。
> 对应文件：`E:\PYTHON\RE-Framework\AGENTS.md`、`E:\PYTHON\RE-Framework\.reasonix\skills\core.artifact\SKILL.md`、`E:\PYTHON\Anchorlaw\spec\protocol-v0.6.md`

---

# Part A — Anchorlaw 协议（spec/protocol-v0.6.md）

## Patch A1：【§5.5 Source Provenance 末尾（"---" 之前）追加】

**新增文本：**

```
**Rule (v0.6.1): Source 落盘证据（Source Artifact Requirement）**

`source` 不仅须是合法的 `probe:<probe>!<entry>#<id>` 格式，其所引用的验证记录
MUST 有可引用的落盘证据（artifact）：运行命令 + 输出摘要，存放于 `.investigations/`
或 `.artifacts/` 下（如 `regression-record.md` 条目）。scan 门禁对 source 引用的
记录做存在性校验（至少 WARN 级：source 引用的 probe 条目无对应落盘记录时告警，
不允许直接 PASS）。

Rationale（实证）：CoreSwap 新增 `@anchor.test(..., source="probe:block_probe!SURFBIOME#003")`
后，scan 只校验格式；judge 质疑「探针是否真的跑过」——实际跑过（-biomeDump 812 73 -337
= badlands）但协议无机制要求验证记录落盘，只能事后补 regression-record.md。source 的
价值在于可复现，而可复现的前提是验证记录本身可被找到。
```

## Patch A2：【§9.4 Retry Cap 末尾追加】

**新增文本：**

```
**Scope Clarification (v0.6.1): retry cap 只约束逆向假设的验证轮次**

§9.4 的 3 次硬上限针对「同一逆向假设的 Lift→Verify 验证循环」——在无新 A 层数据
（trace/探针）时反复调整 B1 假设属于过程熵增，cap 强制回 Scout/dynamic tracing。

工程修复（swe）迭代**不适用本 cap**：代码 bug 修复（编译失败、崩溃、运行期错误）
可无限迭代至正确，不消耗逆向假设的 retry 额度。两者区分标准：
- 逆向假设验证：修改的是「对机制的理解」（hypothesis），验证的是理解是否正确 → 计 cap
- 工程修复：修改的是「实现的缺陷」（bug），验证的是程序是否按已确认语义运行 → 不计 cap

Rationale（实证）：CoreSwap SearchTree 移植（MultiNoiseUtil.SearchTree C++ 版）连续
3 版迭代全崩（空指针 → C++ 异常 → MSVC long 32 位截断 INT64_MAX 为 -1），均为工程
缺陷修复，与逆向假设无关；若误计 cap 将迫使放弃本可修复的正确移植。宿主人类已拍板。
```

## Patch A3：【§9.5 之后新增 9.6】

**新增文本：**

```
### 9.6 验证执行者分离（Verification Executor Separation, v0.6.1）

分析（静态、无运行时）与验证（运行时、需工具）可能由不同执行者承担（subagent 沙箱
无 shell 时尤其如此）。规则：

1. **分层标注以实际执行为准**：Full/Partial/Degraded 标注的是「验证执行时的实际手段」，
   而非「分析者意图」。分析产物在无运行时验证证据前，不得仅凭静态对拍升 candidate
   （除非显式声明 Degraded + 诚实声明）。
2. **验证交接**：分析者产出「命令模板 + 预期判据」，执行者运行并落盘原始输出，解读
   必须回传分析者——执行者只执行不解读，分析者只解读不伪造执行。
3. **产物链完整**：交接的命令、原始输出、解读结论三者在 `.investigations/` /
   `.artifacts/` 中可互相引用，缺失即视为验证未完成。

Rationale（实证）：CoreSwap 分析 subagent 沙箱无法运行 block_probe/gradle，运行时
验证全部由主会话补跑。手动执行该流程后结论成立（worker 下模板 → 主会话执行不解读 →
worker 解读），但该流程靠自觉，需协议化。
```

## Patch A4：【§15.4 Consistency Contract 末尾追加】

**新增文本：**

```
**Judge 审查基线（v0.6.1）**——judge 审查 MUST 三源交叉核对，防止交付快照滞后：

1. `.artifacts` 交付快照（worker/subagent 产出）
2. git HEAD + 工作区 diff（代码实际应用版——subagent 交付后可能被宿主修改/合并）
3. 验证/回归记录（`.investigations/` 下 regression 类文档）

三者不一致时（如快照旧于工作区），以工作区实际状态为准并标注差异。

Rationale（实证）：CoreSwap judge 只读 `.artifacts` 快照审查，而 SearchTree 的
`Node::getSquaredDistance` 64 位修复由主会话应用时补做，judge 基于旧快照误报
「32 位未修」。
```

## Patch A5：【§13 末尾追加】

**新增文本：**

```
**Order-dependent 语义等价（v0.6.1）**

三语言等价默认验证「结果等价」（同一输入 → 同一输出）。对**order-dependent 语义**
（平局 tie-break、缓存命中序、遍历序、查询序列依赖），结果等价不足以证明语义等价。

涉及排序/缓存/平局/遍历序的还原点（P1 及以上）MUST：
1. @anchor 描述中显式标注 order-dependence（如「平局取树序遍历第一个」）
2. 验证「确定性」：同输入重复运行结果稳定
3. 验证「查询序列对齐」：若参考实现的结果依赖调用序列（如 Java ThreadLocal
   previousResultNode 缓存、populateBiomes 固定遍历序），C++ 实现 MUST 复刻该序列
   或证明结果与序列无关

Rationale（实证）：CoreSwap biome 判定平局 tie-break——C++ 线性 find 严格 `<` 取
entries 首个（forest），vanilla MultiNoiseUtil.SearchTree 树序遍历平局取 badlands
（且 Java previousResultNode 缓存使平局结果依赖查询序列）。静态「结果等价」对拍
（同点 6 维逐位一致）无法发现此差异，须显式验证平局语义与查询序列。
```

---

# Part B — RE-Framework 规范源

## Patch R1：【AGENTS.md 新增章节（建议插在「五、工作流速览」之后）】

**新增文本：**

```
## 六、subagent 工具执行边界（2026-08-08 实证补充）

分析 subagent（scout/worker/judge）默认沙箱无 shell / 只读命令白名单拦 exe——
block_probe、gradle、探针运行等**运行时验证工具不可在 subagent 内执行**。

**执行者分离（强制）**：
- 分析（静态对拍、数据解读、结论归纳）→ subagent
- 验证（运行 exe/探针、采集原始输出）→ 宿主（主会话）或指定执行者
- 交接：subagent 产出「命令模板 + 预期判据」→ 执行者运行并落盘原始输出
  （.investigations/）→ 原始输出回传 subagent 解读 → 执行者不解读、subagent 不伪造

**职责边界（宿主人类拍板，2026-08-08）**：
- 宿主可做：编译/构建、工具采集与回归、应用 subagent 交付的代码、git 提交、
  流程签核（complete_step/todo）、崩溃调试定位、AGENTS/工具链文档维护
- 必须 subagent：数据解读/根因分析/结论归纳、审查（judge）、知识库更新、
  代码/算法交付（patch 或新文件）

**subagent 写码交付强制自检**（2026-08-08 SearchTree 3 版全崩教训）：
- 交付代码 MUST 声明「未编译验证」状态 + 附静态自检清单：
  ① 类型宽度（MSVC long=32 位：距离/平方和/INT64_MAX 赋值 MUST long long）
  ② move 语义/悬垂指针 ③ throw 路径/空容器 ④ 与参考实现逐行对拍点清单
- 宿主编译失败/崩溃时退回 subagent 修（附崩溃现场），宿主不代写分析结论
```

## Patch R2：【core.artifact skill 补充】

**新增文本（core.artifact/SKILL.md 的 index.yaml 管理部分）：**

```
### index-entry.yaml 合并（v2.1，2026-08-08）

并行 worker 各自交付 `index-entry.yaml` 片段时，根 `.artifacts/index.yaml` 合并规则：
- 提供 merge 工具/命令：扫描 `.artifacts/**/index-entry.yaml` → 合并根 index
- 冲突检测：id 重复（不同 status）→ 报错待人工；id 重复（相同 status）→ 去重
- 合并 MUST 保留各片段内全部字段（path/kind/status），并标注合并依据
- 禁止手动粘贴式合并（CoreSwap 实证：5 个 worker 片段中 biome-fix 的 5 个
  patch/code/test 条目差点漏合并，judge 抓到）
```

## Patch R3：【宿主集成（Reasonix 平台侧）】

**新增文本（提交给 Reasonix 宿主维护方）：**

```
### complete_step 与 todo 列表匹配（2026-08-08 反馈）

- 宿主从计划文本生成的 todo 列表（phase 级项 + 子步骤 + 「产物」项）与
  complete_step 的 step_index 多次错位（CoreSwap 实证 4 次签核失败重试）
- 建议：① complete_step 支持按「标题子串」匹配当前 in_progress 项（而非仅 index）；
  ② 或提供 todo 列表查询（返回完整列表 + in_progress 项标识），减少盲猜
```

---

## 附：CoreSwap 侧已本地落地（无需并入）

- AGENTS.md「探针/参照数据采集核对铁律」（seed 三查 + 采样坐标语义 + 参照完整性）
- AGENTS.md 第九章职责边界（= Patch R1 的 CoreSwap 版）
- 工具链表登记 WG_BIOMEDUMP / WG_FINDTOP / WG_FINDDUMP / WG_SEARCHTREE_CACHE / st_bug_test
