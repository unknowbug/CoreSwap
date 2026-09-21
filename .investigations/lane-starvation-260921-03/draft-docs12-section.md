# 草稿：docs/12-lighting.md 追加小节（260921-03）

> **草稿性质**：subagent 产出，**未写入** versions/1.20.1/docs/12-lighting.md——主会话应用（追加到文末，`### 复测口径指针` 小节之后，对齐既有「## 标题（日期，状态）」格式）+ 验证。

---

## 跑图前沿区块完成 starvation——grace 等待窗口在零散触发形态暴露（260921-03，candidate）

> 判读 record：`.investigations/lane-starvation-260921-03/record-260921-03.md`；judge PASS-with-conditions（M1/M2/N1-N3 已应用）；知识沉淀 → workflow-patterns #216。

### 结论（机制）

光照域批（LIGHT-DOMAIN）**grace 窗缺省 10000ms**：窗口期内域批任务不 seal，chunk 光照 future 不完成 → chunk 无法 FULL → **跑图前沿区块完成被拖 ≥10s/批**；阻塞式消费者（forceload 命令 / 玩家走图区块发送）全部被拖死，latesubmit=rebuild 放大。帧率正常但「跑图生成新区块极慢」。**grace 250ms 恢复与 legacy/vanilla 持平（域批机制保留）**。boot 洪峰形态（任务一次灌满、grace 一次性摊销）下该窗口结构性不可见——260914-04 e2e 判据因此测不出它（触发形态维度验证域缺口，→ #216）。

### 六臂摘要（E1：dll sha cc4e39fe 全臂一致；载体 = dedicated server + fresh world seed 8576294172403134396 + RCON forceload 前沿 64 chunk；采样 = mca header 非零 entry 近似口径，三臂同口径可比，不外推）

| 臂 | 形态 | forceload 命令阻塞 | 144 完成时刻 |
|---|---|---|---|
| A | lightRust+domainbatch（现役） | 10-41s/条 | ~t+98s（keepup `Can't keep up 27315ms`） |
| B | 仅 lightRust（legacy per-chunk） | 0.3-1.7s | ≤ t+19.3s |
| C | vanilla 光照 | 0.2-1.8s | ≤ t+19.6s |
| E | A + stallwatch15 | 冻结复现（同 A） | — |
| F | A + domainGrace=250 | 0.5-2.4s | ≤ t+20.5s（域批 264 任务行在跑） |
| A-r3 | A 复现跑（judge M1 闭合） | 同 A 形态 | — |

域任务 avgMs 仅 3-25ms（执行不慢）但 A 臂全部 timedOut=true、timeout 19→65 单调爬升 = 滞留在**等待窗口**而非执行慢。

### stallwatch 栈证据（E 臂）

20:02:58-20:04:14 连续 6 次 dump 同栈：主线程 park 于 `ForceLoadCommand.executeChange → ServerWorld.setChunkForced → World.getChunk → ServerChunkManager.getChunk → ThreadExecutor.runTasks`（等待 chunk 完成）= 非瞬态阻塞，一轮 dump 直指阻塞点。

### 修复候选（用户决策项，未实施）

1. **翻缺省 grace**（10000 → 数百 ms 级）：F 臂已实测方向有效；代价 = 合批率下降（吞吐影响未实测）。
2. **自适应 grace**（零散触发短、洪峰长）：保两端形态，实现成本高。
3. 兜底（可叠加）：`latesubmit=merge`（止血开关）。

### §15.4 无取代声明

与 260914-04 e2e FAIL（round3 判据）**域边界互补、不取代**：彼验证域 = boot 洪峰预生成 wall（该域内 FAIL 成立）；本块验证域 = 跑图前沿持续零散触发形态（A 臂 FAIL、F 臂恢复）。触发形态 × 时间粒度不同，无双指针取代关系。

### 遗留与诚实注记

- timedOut 旗标精确语义待究（当前语义 = 任务超 seal deadline 执行，不阻塞 chunk 完成；F 臂恢复后计数仍 263/264，dup=0 resealed=0）——引用其做健康度判据前必须先钉死。
- 玩家实机走图形态未实测（机制链推断覆盖，采样仅 RCON forceload 载体）。
- 状态 = **candidate**（confirmed 留用户）。
