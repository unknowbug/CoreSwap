# 草稿：docs/10-timewise-archive.md 时间线 260921-03 块

> **草稿性质**：subagent 产出，**未写入** versions/1.20.1/docs/10-timewise-archive.md——主会话应用（追加到文末，对齐既有「## YYMMDD-NN（实际 …）标题」+ `### 过程链` / `### 结论与状态` / `### 产物` 格式）+ 验证。

---

## 260921-03（实际 2026-09-21，Get-Date 已核）跑图前沿区块生成 starvation 定位——grace 等待窗口缺省 10000ms 拖死前沿 chunk 完成（域批保留，grace 250 恢复持平）——record candidate（judge PASS-with-conditions M1/M2/N1-N3）；#216 落库

> 承用户实机报障「帧率正常但跑图生成新区块极慢」（1.20.1 客户端）；架构 `.investigations/000-架构设计/架构计划-260921-03-跑图前沿区块生成 starvation.md`（轻量，用户 19:39 口头批准）；record = `.investigations/lane-starvation-260921-03/record-260921-03.md`；采集 `.tmp/lane-260921-03/`（run_lane*.py + arm-*.log）。

### 过程链

- ✅ **立项**：实机报障 + 日志硬证据（[LIGHT-DOMAIN] 全部 timedOut=true、timeout 19→65 爬升而 avgMs 仅 3-25ms = 队列饥饿嫌疑）→ 三臂复现架构批准（A=现役 lightRust+domainbatch / B=仅 lightRust / C=vanilla；scout 不触发——机制主线已有日志证据）。
- ✅ **三臂采集**（dedicated server + fresh world seed 8576294172403134396 + RCON forceload 前沿 64 chunk，dll sha cc4e39fe 全臂一致 = E1）：A 命令阻塞 10-41s/条、144 chunk 至 ~t+98s（keepup 27315ms）；B ≤ t+19.3s；C ≤ t+19.6s—— starving 只在 domainbatch 臂。
- ✅ **stallwatch 栈定位（E 臂 = A + stallwatch15）**：连续 6 次 dump（20:02:58-20:04:14）同栈——主线程 park 于 ForceLoadCommand.executeChange → ServerWorld.setChunkForced → World.getChunk → ServerChunkManager.getChunk → ThreadExecutor.runTasks（等待 chunk 完成），一轮 dump 直指阻塞点。
- ✅ **恢复臂（F = A + domainGrace=250）**：命令 0.5-2.4s、144 ≤ t+20.5s（与 B/C 持平），域批 264 任务行仍在跑 = 单变量归因闭合到 grace 窗（缺省 10000ms：窗口内任务不 seal → chunk 无法 FULL → 前沿完成被拖 ≥10s/批，latesubmit=rebuild 放大）。
- ✅ **judge PASS-with-conditions（M1/M2/N1-N3）**：M1 = A 臂复现跑（A-r3）闭合；M2 修正 = boot 洪峰批正常 seal（sealed=48 冻结）+ 跑图零散阶段全部 timedOut（佐证「洪峰形态测不出窗口」）；条件全部应用。
- ✅ **判读 record candidate + 知识落库**：record-260921-03.md（§9.7 口径：E1 / 单 seed 单载具 / mca 近似口径不外推；臂 sanity 负自证行 = armed graceMs + lightInit + dll sha）；workflow-patterns **#216**（等待窗口参数在 boot 洪峰载体不可见，验证域覆盖 = 触发形态 × 时间粒度；stallwatch 一轮定位形态；grace 类参数复测口径须含每 chunk 完成延迟）——均 subagent 草稿 → 主会话应用。
- 🔍 **遗留**：grace 翻默认/自适应属用户决策项（未实施）；timedOut 旗标精确语义待究（F 臂恢复后计数仍 263/264；当前语义 = 超 seal deadline 执行，不阻塞 chunk 完成）；玩家实机走图形态未实测（采样仅 RCON forceload 载体）。

### 结论与状态

- **机制结论（candidate）**：域批 grace 窗（缺省 10000ms）期间 chunk 光照 future 不完成 → chunk 无法 FULL → 跑图前沿区块完成被拖 ≥10s/批；grace 250ms 恢复与 legacy/vanilla 持平、域批机制保留。与 260914-04 e2e FAIL **域边界互补不取代**（彼 = boot 洪峰 wall 域，本块 = 跑图前沿零散触发域）。
- confirmed 留用户。

### 产物

`.investigations/lane-starvation-260921-03/{record-260921-03.md,draft-*}.md` + `.tmp/lane-260921-03/{run_lane.py,run_lane_diag.py,run_lane_grace.py,arm-*.log,driver-A-r3.log}` + `.investigations/000-架构设计/架构计划-260921-03-跑图前沿区块生成 starvation.md`。
