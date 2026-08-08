# RE-Framework / Anchorlaw 改进建议报告（2026-08-08，CoreSwap 8576-24blocks 任务实证）

> 来源：CoreSwap 8576-24blocks 收尾任务（seed 8576、24 块 mismatch、SearchTree 移植 3 版迭代、judge 审查闭环）。
> 三条工程项已在 CoreSwap AGENTS.md 本地落地（工具链表 + 第九章职责边界）；以下为**框架侧（RE-Framework）与协议侧（Anchorlaw）**需更新的内容。

---

## 一、RE-Framework 侧（框架机制，3 项）

### 1.【最高优先】分析 subagent 的「工具执行」通道缺失

**实证**：anchor.worker / recode.scout 沙箱**无 shell / 只读命令白名单拦 exe**（block_probe、gradle、python 读取脚本均不可执行）。后果：
- 运行时验证（block_probe -blockDump/-biomeDump/WG_* 诊断、RouterProbe）全部被迫主会话执行 → 「主会话补跑」边界模糊 → 宿主人类三次纠正「你只做决策，干活审计全部 subagents」
- 验证分层（Full=block_probe / Partial=反射探针）在分析侧实际不可达，worker 只能静态对拍，结论置信度被动降级
- 本轮 SearchTree 移植 3 版全崩（空指针→异常→MSVC long 32 位截断），worker 无编译/运行闭环是主因

**建议**（二选一或并行）：
- **A. 沙箱放行只读 exe**：block_probe 的 `-blockDump / -biomeDump / WG_SURFDUMP / WG_BIOMEDUMP / WG_FINDTOP` 等全部只读（读参照+stdout/stderr），可归入「只读命令」分类放行给分析 subagent
- **B. 官方「命令委托」通道**：subagent 提交命令模板 → 宿主执行 → 原始输出落盘回传 subagent 解读（替代主会话手动「补跑」，消除边界模糊）。CoreSwap 本轮已用临时手动版（worker 下模板 → 主会话执行不解读 → worker 解读），效果可行，建议框架级支持

### 2. worker 的 index-entry.yaml 片段合并自动化

**实证**：本轮 5 个并行 worker（surface-plus1/aquifer-wateredge/biome-terracotta/followup/biome-fix 多轮）各自交付 `index-entry.yaml` 片段，根 `.artifacts/index.yaml` 合并靠主会话手动——biome-fix 的 5 个 patch/code/test 条目差点漏合并（judge 抓到）。

**建议**：core.artifact 提供 merge 工具/命令：扫描 `.artifacts/**/index-entry.yaml` → 合并根 index + 冲突检测（id 重复/状态不一致），替代手动。

### 3. complete_step 与宿主 todo 列表的匹配机制

**实证**：宿主从计划生成的 todo 列表（含 phase 级项 + 子步骤 + 「产物」项）与 complete_step 的 `step_index` 多次错位（本任务 4 次签核失败重试），列表不可查询、映射不可见。

**建议**：① complete_step 支持按「标题子串」匹配当前 in_progress 项（而非仅 index）；② 或提供 todo 列表查询（返回当前完整列表 + in_progress 项），减少盲猜。

---

## 二、Anchorlaw 协议侧（5 项）

### 1. @anchor.test source 的证据可追溯性强化

**实证**：`searchtree.h` 新增 `@anchor.test(..., source="probe:block_probe!SURFBIOME#003")`，scan 只校验格式合法；judge 质疑「探针是否真的跑过」——实际跑过（-biomeDump 812 73 -337 = badlands）但**协议无机制要求 source 对应验证记录落盘**，只能靠事后补 regression-record.md。

**建议**：§5.5 增加「source 指向的验证记录 MUST 有可引用的落盘证据」（如 `.investigations/*/regression-record.md` 条目 + 命令 + 输出摘要），scan 门禁可升级为「source 引用的记录存在性校验」（至少 WARN 级）。

### 2. 验证分层（Full/Partial/Degraded）与执行者分离

**实证**：worker 无 shell → Partial（反射探针）/Full（block_probe）在分析侧不可达，实际只能 Degraded（静态对拍）——但产物仍标「candidate」（后来靠主会话补运行时证据才成立）。

**建议**：§9 补充「验证执行者分离」条款：分析（静态，worker）与验证（运行时，执行者）解耦；分层标注**以实际执行为准**（谁跑的、什么环境），分析产物在无运行时证据前不得升 candidate（除非显式声明降级）。

### 3. judge 审查基线 = 交付快照 + git HEAD/工作区 diff

**实证**：judge 只读 `.artifacts` 交付快照（worker 旧版），与 src 应用版（主会话应用时改过 64 位）不一致 → 「Node::getSquaredDistance 32 位」误报。

**建议**：§15/§16 参考实现补充：judge 必须核对 ① 交付快照 ② git HEAD + 工作区 diff（代码应用版）③ regression/验证记录，三源交叉。

### 4. retry cap 与程序修复迭代明确区分

**实证**：用户明确「程序修复不算逆向的 3 次 retry cap」——SearchTree 移植 3 版迭代（崩溃×2 + 修复）是 swe 工程，不应触发「≤3 换方向」。

**建议**：§9 明文区分：**retry cap（≤3）只约束逆向假设的验证轮次**（同一假设验证失败换方向）；**工程修复/代码迭代不计数**（可无限迭代至正确，宿主人类已拍板）。

### 5. 【新增条款】order-dependent 语义等价验证

**实证**：biome 判定平局 tie-break——C++ 线性 find 严格 `<` 取 entries 首个（forest），vanilla SearchTree 树序遍历平局取 badlands（且 Java 有 previousResultNode ThreadLocal 缓存，**平局结果依赖查询序列**）。三语言等价（§13）覆盖「结果等价」但此类 **order-dependent 语义**（平局/缓存/遍历序）是盲区。

**建议**：P1 还原点涉及排序/缓存/平局/tie-break 时，@anchor 描述 MUST 标注 order-dependence + 验证「确定性 + 与 Java 查询序列对齐」；协议新增 order-dependent 等价验证条款（Java ThreadLocal 缓存语义、populateBiomes 查询序列 vs C++ 即时判定序列的差异是真实坑）。

---

## 附：CoreSwap 侧已本地落地（无需框架改动）

- AGENTS.md 工具链表：WG_BIOMEDUMP / WG_FINDTOP / WG_FINDDUMP / WG_SEARCHTREE_CACHE / st_bug_test 登记
- AGENTS.md 第九章「主会话/subagent 职责边界」：主会话可做（编译/回归/工具采集/崩溃调试/git/签核）vs 必须 subagent（分析/解读/审查/知识库/代码交付）；subagent 写码强制自检清单；judge 三源核对；「主会话只执行不解读」
- AGENTS.md「探针/参照数据采集核对铁律」：seed 三查 + 采样坐标语义（floor 对齐/8 邻域选点/原始直采三套）+ 参照文件完整性
