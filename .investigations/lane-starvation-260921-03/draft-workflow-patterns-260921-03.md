# 知识库草稿：workflow-patterns 新发现一条（260921-03）

> **草稿性质**：subagent 产出草稿，**未写入 knowledge/discovered/workflow-patterns.md**——主会话应用 + 验证。
> **编号**：（待主会话分配）——草稿内暂用「发现 #N」，应用时替换为实际编号。
> **应用位置**：`knowledge/discovered/workflow-patterns.md` 末尾（对齐现有「## 发现 #N」格式），并同步 INDEX.md。

---

## 发现 #N（编号待主会话分配）: 调度型优化的「等待窗口」参数在 boot 洪峰载体上不可见，只在持续零散触发形态暴露——验证域覆盖 = 触发形态 × 时间粒度（260921-03）

- **时间/置信度/module**：260921-03；candidate（E1 三臂 + stallwatch 栈级证据 + 恢复臂，judge 待审）；workflow-patterns / 验证域覆盖面（#82 家族）。

**观察**：光照域批（LIGHT-DOMAIN）的 grace 窗缺省 10000ms——该参数下域批任务在窗口期内不 seal，chunk 光照 future 不完成、chunk 无法 FULL。260914-04 e2e 判据（boot 预生成 wall）**测不出它**：boot 时任务洪峰一次灌满，grace 一次性摊销，wall 差异被淹没。而实机跑图前沿形态（玩家走图持续零散触发）下，每个零散批各吃一个完整 grace 窗 → 前沿区块完成被拖 ≥10s/批，阻塞式消费者（forceload 命令 / 玩家区块发送）全被拖死，帧率正常但「跑图慢到不可用」。三臂实测（同 dll sha cc4e39fe，E1）：A=现役 lightRust+domainbatch 命令阻塞 10-41s/条、144 chunk 至 t+98s；B=仅 lightRust ≤ t+19.3s；C=vanilla 光照 ≤ t+19.6s；F=仅改 grace 250 → 命令 0.5-2.4s、144 ≤ t+20.5s（与 B/C 持平，域批机制保留）。

**证据**：① stallwatch dump 连续 6 次（20:02:58-20:04:14）同栈：主线程 park 于 `ForceLoadCommand.executeChange → ServerWorld.setChunkForced → World.getChunk → ServerChunkManager.getChunk → ThreadExecutor.runTasks`；② A 臂域任务 avgMs 仅 3-25ms（执行不慢）但全部 timedOut、timeout 19→65 单调爬升 = 滞留在等待窗口而非执行慢；③ F 臂单变量（grace 10000→250）恢复持平 = 归因闭合。来源：`.investigations/lane-starvation-260921-03/record-260921-03.md`；日志 `.tmp/lane-260921-03/arm-*.log`。

**如何利用（判据）**：
1. **调度型优化（合批/grace/延迟窗口/防抖类）引入的「等待窗口」参数，其验证载体 MUST 覆盖「持续零散触发」形态**——boot 洪峰载体上窗口被一次性摊销、结构性不可见；「验证域覆盖 = 触发形态 × 时间粒度」（#82 覆盖面判据的触发形态维度）。新调度参数上线前问一句：「用户慢触发形态下，每个触发各吃多长窗口？」
2. **stallwatch dump 一轮定位主线程阻塞点的实战形态**：给阻塞臂加周期 dump（本块 15s 间隔），连续 ≥2 次 dump 同栈即非瞬态阻塞；栈直接给出「谁在 park 等谁」，配合一臂单变量开关即可闭合归因——成本远低于日志计数器推理。
3. **grace 类延迟参数的复测口径必须含「每 chunk 完成延迟」**（前沿触发到 chunk FULL 的延迟），不能只看 boot wall——wall 是吞吐摊销口径，会掩盖逐批等待窗口（同 #83 分母分场景、AGENTS「吞吐均值 vs 每 chunk 耗时混淆」同族）。

**家族索引**：#82（验证域覆盖面——本条为其「触发形态」维度实例）；#83（分母分场景）；#80（验证载体时序判据）。
