# 260910-04 知识库草稿包（core.worker 只读产出，未改任何现网文件）

> 产出者：知识库 subagent（core.worker 角色，只读 + 只产草稿）。
> 已读：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（项目级错误记录规范，优先级最高）→ `knowledge/INDEX.md`（总入口 + 写入规则/记录价值门）→ 五份来源（verdict / record §6 / b1 / b2 / pipeline-cost-map）→ 现网目标文件末尾（`discovered/workflow-patterns.md` 至 `发现 #106 简记`；`discovered/build-tooling.md` 至 `#27 家族补充案例（第三形态）`）。
> **本文件只是草稿**：不修改 `knowledge/`、`docs/`、`.artifacts/` 任何现网文件；应用与验证由主会话执行。
> 价值门结论：A（E1-E8）高价值详写；B #107/#108 高价值详写、#109 中价值简记、B4 家族补充案例高价值详写；C #46 高价值详写、#47/#48 中价值简记；D 索引追加。

---

## A 错误台账草稿（文件全文）

**应写入路径**：`.investigations/perf-regression-260910-04/perf-regression-errors.md`（**新文件**，当前目录下不存在；项目级错误台账载体 = `.investigations/<课题>/<课题>-errors.md`，见 `knowledge/INDEX.md`「错误台账载体」节）。

**编号说明（主会话应用前必读）**：任务书要求「record §6 的 E1-E5 扩写成五段式 + 新增 E6（`-P` 映射写错作用域）/ E7（并发 bench run 互抢）」。但经核 `record.md` §6，**「-P 作用域」= 原 E2、「并发抢占」= 原 E3**（record 已收编为错误条目，不是「零散提到」）。为避免同一错误双编号，本稿一节一错：E1-E5 = record §6 全五段式扩写（**E2/E3 即任务书点名的两条**，家族叙述已内嵌其「教训」段）；**E6/E7/E8 = 本块另三条「record 只零散提到、未进 §6」的可复用判错经验**（口径不对等 / 机制命名证伪 / 驱动口径自述≠生效）。若主会话坚持任务书编号，只需把 E2→E6、E3→E7 改名、E6/E7/E8 顺延为 E8/E9/E10，**正文内容零改动**。

```markdown
# 260910-04 · 1.21.6 性能回归定责（5.15×）错误台账

> 课题：`.investigations/perf-regression-260910-04/`（1.21.6 性能回归定责；起点 = NEXT_SESSION 下轮开工点 1，用户拍板立项；架构计划 `.investigations/000-架构设计/架构计划-260910-04-1216-perf-regression.md` 已批准，含 R1 授权）
> 载体：CoreSwap 项目级指定错误台账（独立成篇）——`knowledge/INDEX.md`「错误台账载体」节 + `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md` §三
> 格式：五段式（现象 / 根因 / 定位 / 修复 / 教训）+ 末尾「错误→根因」速查表
> 来源：`.investigations/perf-regression-260910-04/record.md`（§6 原始台账 E1-E5，§0-§2 数据，§4 未闭合）+ `.artifacts/perf-regression-260910-04/verdict-260910-04.md` + `.artifacts/perf-regression-260910-04/candidates/b1-choreography.md`
> 置信度：E1-E5 = 本块一手实测（确定级事实记录）；E6/E8 = 本块一手证据；E7 含 b1 fan-out worker 的静态一手源码推导（机制侧 Degraded，实测签名支撑）
> 编号说明：任务书要求「E1-E5 扩写 + 新增 E6（-P 作用域）/E7（并发抢占）」。经核 record §6，**-P 作用域 = 原 E2、并发抢占 = 原 E3**（已收编），故本台账一节一错不重复编号：E2/E3 即任务书点名的两条；E6/E7/E8 = 另三条「record 只零散提到」的可复用判错经验（口径不对等 / 机制命名证伪 / 驱动口径自述≠生效）。

## E1: 沙箱内 `Get-NetTCPConnection` 阻塞导致跑批脚本静默挂死（WMI/CIM 后端不可用）

- **现象**：跑批脚本「启动了服务器但从不发 RCON 命令」，进程挂死；服务器日志里没有 RCON Client 行。
- **根因**：依赖 WMI/CIM 的 cmdlet 在本沙箱**没有后端**——`Get-NetTCPConnection` 不是抛异常而是**阻塞**；同族 `Get-CimInstance` 直接报「无法从客户端访问 CIM 资源」。即失败形态是「**静默挂起**」，不是「报错」——按「会报错」的常识预期去写编排脚本，就会把阻塞当成正常等待。
- **定位**：手动发 RCON 命令可用 + 服务器日志无 RCON Client 行 ⇒ 卡点不在服务器而在脚本「发现服务器/端口」这一步；再用 `Get-CimInstance` 直接复现「CIM 资源不可访问」，确认后端缺失。
- **修复**：编排脚本内**禁用一切 WMI/CIM cmdlet**；改用「WorkingSet 最大 / CPU 最高 的 java 进程」识别服务器进程。
- **教训**：① **沙箱可用性判据必须实测，不能凭常识**——被拦的形态可能是阻塞而非异常，编排脚本静默挂死先怀疑「某步 cmdlet 后端不可用」；② 编排脚本每一步都要可观测（transcript/落盘），否则挂死点只能手二分；③ 替代观测面（汇总见 build-tooling 发现 #46）：进程识别 → 进程属性枚举；线程状态 → .NET `Process.Threads`（ThreadState/WaitReason）。

## E2: gradle `-P` → `-D` 映射写错作用域——`Could not get unknown property 'run' for task ':runServer'`

- **现象**：构建日志 line 235 报 `Could not get unknown property 'run' for task ':runServer'`，runServer 起不来（**配置期**失败，不是运行期）。
- **根因**：把握手参数映射写进了 `tasks.matching { it.name == 'runServer' }` 闭包——`run` 只在 `loom.runs` 的 `benchVmArgs` 闭包作用域内可见；`tasks.matching{}` 的闭包委托对象是 Task，没有 `run` 属性 ⇒ 配置期求值即抛 unknown property。这是「参数接线」家族的**第三维（作用域）**失配：通道对（property）、名字对（映射行名）、**位置错（闭包）**→ 仍不通。
- **定位**：构建日志的配置期异常（与运行期行为日志区分）+ 对照 `benchVmArgs` 闭包内既有可用映射写法。
- **修复**：把 `-Pmixlog / -Pchunktime / -PmaxBgThreads / -PcaMin / -PestL2 / -PcaCap` 映射移回 `benchVmArgs` 闭包（`runtime/1.21.6/java/build.gradle`）。
- **教训**：① 「编译过/配置过 ≠ 接线生效」（`build-tooling` #8/#19/#25 家族）新增**作用域**形态：新增 -P→-D 映射 MUST 同时核「通道 / 名字 / 闭包位置」三查；② 本形态是**响亮失败**（配置期 unknown property），比家族的静默不生效形态好排查——配置期报错先查闭包作用域，别去查参数名；③ 生效判据仍是行为化证据（被测程序回显 / 一次性日志行），构建成功不构成接线证据（#81/#37）。

## E3: 并发启动两个 bench run 相互抢占 → 世界/端口冲突 + 双 FAIL

- **现象**：`results.txt` 出现重复 FAIL 行；两个 run 同时在跑，同 seed 的 `run\world` 被两侧同写、端口冲突，两臂读数双废。
- **根因**：编排错误——前一轮未结束就启动下一轮；launch 前置条件只看「进程已 spawn」，没有「上一轮结果已落盘终态」这一状态机门。
- **定位**：`results.txt` 出现重复 FAIL 行 + 同时存在两个服务器进程；回查脚本 launch 前置条件缺失。
- **修复**：改为单进程串行编排 + 启动前确认上一轮已出现 `DONE` 终态行。
- **教训**：① **性能 run 必须严格串行**（workflow-patterns #28 的**编排侧**形态）——串行判据 = 上一轮**终态落盘**（DONE/FAIL 行）才允许下一轮 launch，不能以进程退出码/固定等待间隔代替；② 双 run 抢世界是隐蔽污染：它不抛异常，只产生**双 FAIL 或静默劣化**，results 里出现重复/成对 FAIL 行是第一签名；③ 编排脚本应是「上一轮终态驱动」的状态机，不是线性 spawn 序列。

## E4: 沙箱内 JVM attach 不可用——`jcmd <pid> Thread.print` 返回 `IOException: 拒绝访问`

- **现象**：`jcmd <pid> Thread.print` 报 `IOException: 拒绝访问`；输出的 dump 文件只有 8 字节（空壳）。
- **根因**：沙箱禁止命名管道，JVM attach 机制走命名管道 ⇒ `jcmd`/`jstack`/`jmap` 一类 attach 工具在本沙箱**一律不可用**（不是命令写错、不是权限参数问题，换参数重试无意义）。
- **定位**：jcmd 的报错文本本身 + dump 文件大小 8 字节（「产出文件存在」但无内容 = 假证据）；同沙箱内改用非 attach 通道成功取到线程状态，交叉确认是 attach 面被禁。
- **修复**：线程状态观测改 .NET `Process.Threads`（`ThreadState`/`WaitReason`）——纯宿主 API，不经被观测进程的 attach 面。
- **教训**：① 沙箱内 jcmd/jstack/jmap 不可用是**环境事实**，不要反复换参数重试；② 「输出文件已生成」≠「有内容」——8 字节 dump 是假证据（核文件大小/内容；同族：build-tooling #42 瞬时拒访）；③ 需要替代观测面时先盘「哪些面不经过被禁通道」（进程属性 / 宿主 API / 落盘日志）。

## E5: 日志 tail 读取滞后——实时监控不可全信，结论以落盘终值为准

- **现象**：读日志时读到的是数分钟前内容，且文件 `LastWriteTime` 也不更新；同一文件晚 2 分钟再读出现新行。
- **根因**：gradle 重定向 stdout 的写缓冲 / 元数据刷新延迟（写端缓冲 + 文件系统元数据滞后），读端与写端不同步 ⇒ 「即时 tail + mtime」这对判进展的手段同时失效。
- **定位**：同一文件两分钟后重读出现新行，而当时 `LastWriteTime` 未变 ⇒ 失效点被钉死为「用即时 tail/mtime 判进展」。
- **修复**：监控改为「隔轮复核 + 以最终 results 终态行为准」。
- **教训**：① **实时 tail 不能作为「卡死 / 无进展」判决依据**（此类判决 MUST 隔轮复核，或直接等终态落盘）；② mtime 与内容新鲜度都可滞后（与 build-tooling #23「mtime 两面都不可信」同族）；③ 长跑采集的结论一律取落盘终值。

## E6: 两臂口径不对等——处理臂的 per-chunk 同步日志成本混进 headline（5.15× 里含 −13% 非机制成本）

- **现象**：历史基线（260910-03）与本轮处理臂多用 **8450 行 per-chunk 同步日志**（log4j），vanilla 臂 0 行；处理臂 268s，日志门控关掉后 **234s（−13%）**。
- **根因**：headline 5.15× 是「未做口径对等化」的差值——处理臂**独有**的诊断/日志成本被算进了「机制差」。per-chunk 每次生成同步写日志是两臂**不对称成本**，与「同步接管」机制无关。
- **定位**：Phase 0 交接结论廉价验证（Anchorlaw §16.3）时 diff 两臂驱动脚本 + 数日志行数（8450 行差）；再用 c1 臂（日志门控关）A/B 量化 −13%。
- **修复**：R1 = per-chunk mixin 日志门控化（默认关，`-Dcoreswap.mixlog=1` 开；1.21.6 + 1.20.1 双版本）；生产 jar 同样受益。
- **教训**：① **两臂对比前先 diff「驱动参数集 / 日志级别 / 探针开关」**，处理臂独有成本一律先归零再比（否则 headline 系统性偏大）；② headline 数字必须带口径标注（本例 5.15× 修正后仍 4.5× 级，结论不变但数字要诚实）；③ 与 #24/#28/#51 同族：口径不对等与并发污染一样，属「测量侧先查」必查项。

## E7: 机制命名错误——「948ms 在等依赖」被一手源码证伪，正确机制是「单车道 + 同步接管」

- **现象**：占周期 95% 的 gap=948ms/chunk 被父候选命名为「Worker 线程在**等 chunk 依赖 / 阻塞**」。
- **根因**：1.21.6 的 loader 遇到未满足依赖**从不阻塞**——`ChunkLoader.load` 用 `getNow(null)` 非阻塞探测，未完成则记账 + `thenRun` 回调重排（`ChunkLoader.java:134-173`；`ServerChunkLoadingManager.java:668-682`），强制链上**无** `join()`/`.get()`（`ServerChunkLoadingManager` 唯一的 `join()` 在 crash 调试快照 `:349`，且前置 `isDone()` 守卫）。真机制 = `ChunkTaskScheduler` + `SimpleConsecutiveExecutor("worldgen")` **单车道、同时只 1 个 entry 在飞**（`ServerChunkLoadingManager.java:192,196`；`ChunkTaskScheduler.java:44-84`），我们的 mixin 把 47ms 重活**同步**做在车道内 ⇒ 车道吞吐 = 1/55.4ms/chunk，其余线程是「**没人喂**」而不是「在等」。
- **定位**：① b1 fan-out worker 一手源码逐点核对（getNow/thenRun/无 join；`PrioritizedConsecutiveExecutor(4)` 的 4 是**优先级档数**不是车道数）；② 实测签名三连：mixin 并发 ≈1（全世界最多一个接管在跑）+ Σ(车道内工作)≈wall（93%）+ 进程 1.27-1.7 核（其余线程 parked）——三者与「等依赖」的预测（worker BLOCKED/WAITING 在 future 上、任务并发 >1）**相反**。
- **修复**：结论措辞改为「单车道 + 同步接管」，把「等依赖」列入排除清单；修复方向由「查依赖调度」改为 **R3 = 把接管段移出 worldgen 车道**（照 vanilla 的 `supplyAsync(..., Util.getMainWorkerExecutor().named("wgen_fill_noise"))` 异步形态）。
- **教训（判错经验，可复用）**：① 「大 gap + 低 CPU + 多线程 park」不要默认「等依赖/等锁」——先算 **Σ(被串行化执行点内的工作量) vs wall**，比值≈1 即并行度 1.000 的串行车道（workflow-patterns 新 #107 判据 ①）；② 断言「阻塞/等待」前 MUST 在强制链上 grep `join`/`get`/阻塞原语，**零命中即是「无阻塞」的结构证据**；③ **机制命名错误会直接误导修复方向**——「等依赖」指向调度器排查，「单车道」才指向「把重活移出车道」；候选机制命名在定责前 SHOULD 做一次廉价证伪（同 §16.3 交接结论验证纪律）。

## E8: 驱动脚本自述「已设置」≠ 参数生效——`chunky chunkradius 56` 在本版 Chunky 无效，实际一直用默认 radius 500（=4225 chunks）

- **现象**：驱动脚本历来自述并 echo「chunkradius=56」，历史文档/交接也按该口径转述；实际 **Chunky 1.4.40 无 `chunkradius` 子命令**（回显 `Incorrect argument`），区域一直是**默认 radius 500 方块 = 4225 chunks**。
- **根因**：口径声明来自**驱动脚本自述**而不是**被测程序回显**——脚本把「我发了这条命令」当成「参数生效」；Chunky 对未知子命令不中断脚本、只回一行错误，于是脚本自己的 echo 文案成了第二真相源，并被下游文档继续转抄（scout 成本图 `.investigations/perf-regression-260910-04/pipeline-cost-map.md` §0 亦照抄「chunkradius=56 → 4225 chunks」的失真措辞）。
- **定位**：核对 Chunky 回显（`Incorrect argument`）+ 逐臂核对实际 chunk 数（4225，全臂一致）⇒ 参数从未生效，但**各臂吃同一默认值**（因此基线仍可比，未造成数值错误，只造成文字口径失真）。
- **修复**：口径表述改为「Chunky 默认 radius 500 方块 = 4225 chunks」；驱动脚本不再 echo「已设置」，改为读取并回显**被测程序**的配置/规模输出（或核对完成 chunk 数）。
- **教训**：① **口径声明 MUST 来自被测程序回显/日志，不能来自驱动脚本自述**（#37/#81 家族：生效证据必须行为化）；② 「参数无效却不报致命错」会造成**跨文档转抄漂移**（#90 转抄家族）——失真措辞一路进交接/时间线/scout 地图，纠偏只能靠被测程序回显；③ 本例幸运在「各臂都吃同一默认值」，基线可比；**若只有部分臂受默认值影响，就会直接变成假差异**（正是 #20/#53「默认值当公理」家族的风险面）。

## 错误→根因 速查表

| # | 一句话现象 | 根因（机制） | 判据 / 家族 |
|---|---|---|---|
| E1 | 脚本启动服务器后挂死，从不发 RCON | 沙箱无 WMI/CIM 后端，`Get-NetTCPConnection` **阻塞**（非报错） | 沙箱可用性必须实测；禁 WMI/CIM → 进程属性识别（build-tooling #46） |
| E2 | `Could not get unknown property 'run'` | -P→-D 映射**位置**错：`run` 只在 `benchVmArgs` 闭包内可见 | 「编译过/配置过 ≠ 接线生效」家族**作用域维**；三查 = 通道/名字/闭包（#8/#19/#25/#47） |
| E3 | 双 FAIL + 世界/端口冲突 | 编排未等上一轮结束，两个 bench run 互抢 | 性能 run 严格串行；串行判据 = 上一轮终态落盘（#28 编排侧） |
| E4 | `jcmd` → `IOException: 拒绝访问`，dump 8 字节 | 沙箱禁命名管道，JVM attach 通道不可用 | attach 类工具禁用 → .NET `Process.Threads`；「文件存在 ≠ 有内容」（#46） |
| E5 | tail 读到旧内容、mtime 不更新 | gradle stdout 重定向缓冲/元数据刷新延迟 | 实时 tail 不作卡死判据；结论取落盘终值（#23 mtime 家族） |
| E6 | 处理臂多 8450 行 per-chunk 日志，headline 偏大 13% | 未做口径对等化，处理臂独有诊断成本混入机制差 | 两臂比较先 diff 参数集/日志级别/探针开关（#24/#28/#51） |
| E7 | 948ms gap 被称「在等依赖」 | 1.21.6 loader 非阻塞（getNow+thenRun，无 join）；真机制 = 单车道 + 同步接管 | Σ(车道内工作)≈wall ⇒ 并行度 1.000；断言阻塞前 grep join/get（新 #107） |
| E8 | 脚本 echo「chunkradius 已设置」 | Chunky 1.4.40 无该子命令，实际默认 radius 500 = 4225 chunks | 口径声明取被测程序回显；脚本自述不构成生效证据（#37/#81/#90） |
```

---

## B workflow-patterns 插入段

**目标文件**：`knowledge/discovered/workflow-patterns.md`（现网末尾 = `### 发现 #106 简记`，共 1726 行）。
**插入位置**：**全部追加到文件末尾**（即 `发现 #106 简记` 之后），按下列顺序：B1(#107) → B2(#108) → B3(#109) → B4(#83/#51 家族补充案例)。

### B1 — 追加到文件末尾（#106 之后）

```markdown
## 发现 #107（最高价值·错误优先）: 「同步接管阻塞单车道调度器」——把重活同步做在被调度器串行化的执行点上，整条管线的并行度塌成 1（260910-04）

- **发现时间 / 发现者 / 置信度 / module**：260910-04（实际 2026-09-10）；主会话（实测）+ b1 fan-out worker（一手源码静态推导）；**candidate**（数据层实测成立：chunk 级计时双跑一致 + Σ/wall 93% + 池缩放反向旁证；「单车道 busy/inFlight 直接计数」未做 ⇒ 机制命名部分 = **Degraded（静态一手源码，未跑车道占用探针）**，故机制命名建议 candidate 并保留 b1 @idk）；workflow-patterns / 性能结构归因 + 接管形态。
- **来源定位**：`.artifacts/perf-regression-260910-04/verdict-260910-04.md`（candidate，§1 定量闭环 / §2 证据链）；`.investigations/perf-regression-260910-04/record.md` §2.4-§2.5；`.artifacts/perf-regression-260910-04/candidates/b1-choreography.md`（一手源码 file:line 全表）；b2（写回副作用候选）= REFUTED（≤3.7ms/chunk = 0.37% 周期）。
- **现象（错误链）**：1.21.6 生产 5.15× 回归（4225 chunks：vanilla 52s vs coreswap 268s，5.15×；处理臂含历史 per-chunk 日志成本，见本课题错误台账 `perf-regression-errors.md` 的 E6 口径修正）。chunk 级计时：`mixin=51.0ms [jni=47.2 write=3.5 hmap=0.22 scan=0.08 beard=0.04]`、`carve=1.3ms`、`feat=3.1ms`（≈55.4ms/chunk），而同一 Worker-Main 线程「上次接管返回 → 本次接管开始」的 `gap=948ms/chunk`（占周期 95%）；进程 CPU 仅 **1.27-1.7 核 / 24**（vanilla 同仪器 **4.93 核**），线程绝大多数 parked。
- **根因（机制）**：MC 1.21.6 的 worldgen 推进只有**一条车道**——`SimpleConsecutiveExecutor("worldgen")`（`ServerChunkLoadingManager.java:192,196`）+ `ChunkTaskScheduler` **同时只允许 1 个 entry 在飞**（下一个 entry 必须等当前 entry 的 task 全部跑完才 `poll`；`ChunkTaskScheduler.java:44-84`，`pollOnUpdate` 门只在单车道 dispatcher 内读写）。vanilla 在 `NoiseChunkGenerator.populateNoise` 里 `supplyAsync(..., Util.getMainWorkerExecutor().named("wgen_fill_noise"))`（`NoiseChunkGenerator.java:326-353`）把同量级重活甩给 23 线程池，**车道任务只做 µs 级调度**（pending future 就是「把 worker 还回池」的 yield 点）。我们的 mixin 在 `populateNoise` **HEAD** 同步做完 `feedBeardifier + fillChunk + writeChunk` 并返回**已完成** future（`runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java:76-113`）⇒ `ChunkLoader.load` 的 `getNow(null)` 永远非 null ⇒ 同一次车道任务里把整条链（含区域内邻居）一路同步跑完 ⇒ **任何时刻全世界最多 1 个接管在执行**，与线程池大小无关。
- **证据（怎么定位的）**：① **Σ ≈ wall**：Σ(接管 mixin 251.1 + carve 5.7 + features 12.3) = **269.1s** vs 实测 wall 290.3s（Chunky Total 283s）→ 92.7%（vanilla 侧 Σ 17.0s vs 60.1s = 28%，重活在池里）；若机制错，coreswap 的 Σ 应 ≪ wall（如 <60%）——b1 的证伪测试① PASS。② **池大小反向旁证**：coreswap `-PcoreswapThreads=1` = 279s（比 234s **更慢**）→ 线程 spawn/池容量不是限流点；vanilla 池 23→4 时 52s→约 100s（**池敏感**），两臂方向相反。③ 算术闭合：4225 × 55.4ms = 234.06s ≈ c1 实测 234s（差 0.04%）——**实测 wall 就是结构下限本身**。④ b2（写回语义副作用）REFUTED：可归因项 ≤3.7ms/chunk（0.37%），后续 stage 逐条核对 ≈0（光照不读高度图、4 张高度图必被 FEATURES 重建、post-processing 方向为负、序列化只由内容决定）。
- **判据（MUST，可复用）**：
  1. **性能回归先算「被串行化执行点内的工作量之和」vs wall**——比值 ≈1 即并行度塌陷（本案 93%）；这比逐段优化/逐项排除有效得多：它一次给出「瓶颈在结构不在算力」的定量闭环。
  2. **「线程池大小调参无效 + CPU 占用极低 + 线程大量 park」= 单车道症状三联**（本案：`-PcoreswapThreads=1` 反而更慢 / 1.27-1.7 核 / gap 948ms）。三联齐出时不要再去查 CPU 侧（引擎算力、全局锁、JNI 拷贝）——它们在这条链上**不可观测**（并发恒为 1，锁不会被争用）。
  3. **接管类优化必须对齐原实现的「同步/异步形态」，不能只对齐语义正确性**——我们语义上完全正确（hash 哨兵逐位同），但把本该异步卸到池里的重活同步留在了唯一的串行车道上，形态错即性能错。接管评审 MUST 附一栏「原实现在哪条车道/哪个池上执行、我们的接管点是否保持同样的 yield 形态」。
  4. 反向护栏：Σ≈wall 只证「车道饱和」，不排除上游限流（`ThrottledChunkTaskScheduler maxConcurrentChunks=4`，`ChunkLevelManager.java:49-56`，b1 @idk 未验证）——宣布结构定论前 SHOULD 补车道 busy/inFlight 直接计数（b1 建议测量 2；若 R3 后 wall 仍远高于 vanilla，优先查此）。
- **家族索引**：#83（性能分母/结构场景——本条为「瓶颈在并行结构不在算力」形态）、#100/#102（性能课题归因必须在真实通路上核对多面——本条补「执行位置：哪条车道/哪个池」为性能结构面）、#28（性能 run 串行测量纪律）、#36/#98（执行体与生效口径——本条为「执行位置」维）、新 #109（臂无关尺子）。
```

### B2 — 紧随 #107 之后追加

```markdown
## 发现 #108: 「Σ(被串行化执行点内工作) ≈ wall ⇒ 并行度 1.000」判据的使用要点与陷阱（260910-04）

- **发现时间 / 发现者 / 置信度 / module**：260910-04；主会话（实测）+ b1 worker（判据组织）；**candidate**（同批实测；跨 run 摆动实测）；workflow-patterns / 性能判据口径。
- **来源定位**：`.artifacts/perf-regression-260910-04/verdict-260910-04.md` §1/§4（§9.7 声明）；`candidates/b1-choreography.md` 反证条件 1/2。
- **观察（判据怎么用）**：coreswap Σ=269.1s vs wall 283-290s（93%）；vanilla Σ=17.0s vs wall 53-60.1s（28%）——**同一个比值口径**在两臂上把「车道饱和」与「重活在池里」分开。本轮同配置两臂 wall 摆动可见（ct1 同配置双跑 **234s vs 283s，±20%**）。
- **要点与陷阱（MUST）**：
  1. **同批同臂才可比**：Σ 与 wall 必须来自**同一次 run**（同一批采样行）；跨 run/跨批次拼接的 Σ 与 wall 不构成比值证据。
  2. **比值抗摆动，绝对值不可引**：跨 run ±20% 摆动时（234 vs 283）比值判据仍成立——因为 Σ 与 wall **同向变化**（机器整体快/慢同时缩放两者）；但**绝对值禁止跨 run 引用**（#18 跨 session 数字 / #51 噪声基线 / #103 机器噪声带家族）。
  3. **必须与「进程 CPU / 核数」联用**：比值 1.000 + 进程 CPU 低核数（1.27 worldgen 重活推算 / 1.7 进程实测 vs vanilla 4.93 核）= 串行车道双证据；单独 Σ≈wall 可能是「单线程任务但 CPU 满」（那是算力受限，不是车道）。
  4. **Σ 的分母界定必须与被串行化执行点一致**：先确认哪些阶段真的在同一条 lane 内（本案 carve/features 在 vanilla 里本来也同步在 lane 内，故计入 Σ；SURFACE 亦然），漏计/多计都会把比值推离 1——比值异常先回查阶段归属（b1 §证据 A/C）。
  5. **比值≈1 只证「车道饱和」，不排除上游限流**：结合队列深度/loader 积压（b1 建议测量 3）才能给「等号」；否则只能给不等式。
- **家族索引**：#107（主条）、#83（分母/场景）、#18/#51/#103（跨 run 绝对值不可引）、#24（同批交错）。
```

### B3 — 紧随 #108 之后追加

```markdown
### 发现 #109 简记: 臂无关「每 chunk 线程时间」尺子——在两侧都会经过的方法上打同线程间隔点，可给两臂同一把尺子（260910-04）

- **发现时间 / 发现者 / 置信度 / module**：260910-04；主会话（实测，`-Dcoreswap.chunktime=1` 的 featInterval）；**candidate**（单 region 单会话口径）；workflow-patterns / 测量手法。
- **手法**：在**两侧都会经过**的方法（本案 `ChunkGenerator.generateFeatures` HEAD）打同线程间隔点，测「同一线程两次到达该点的间隔」= 该线程的**周期**。本案 coreswap **1422ms** vs vanilla **299ms**（比值 **4.76×**），与 wall 比 283/53 = **5.34×** 同阶自洽。
- **价值**：这是**臂无关**的尺子——不依赖阶段计时探针（避开多线程探针污染铁律）、不依赖两臂载体/阶段口径一致；它天然包含「排队 + 执行」，测的是线程周期而非 CPU 成本，正好与「并行度」这一待测量同构。
- **陷阱**：① 它测的是周期不是成本——两臂同期比同阶**不能**反推「谁算得多」（要算量另配计数器）；② 间隔点必须选两侧共有的方法（选错成只在一侧经过的点，比值即无意义）；③ 与 wall 比值对照时须声明 wall 口径（Chunky Total vs 生成段）。
- **家族索引**：#107/#108、#83（分母）、#103（噪声）。
```

### B4 — 紧随 #109 之后追加（口径修正）

```markdown
### 发现 #83/#51 家族补充案例（260910-04）: 驱动参数「看起来设置了但无效」——Chunky 1.4.40 无 `chunkradius` 子命令，实际一直用默认 radius 500 方块 = 4225 chunks；脚本自述的「已设置」不构成生效证据

- **发现时间 / 发现者 / 置信度 / module**：260910-04；主会话（回显核对 + 逐臂 chunk 数核对）；**candidate**（实测回显 + 4225 逐臂核对一致）；workflow-patterns / 口径纪律（#83 分母口径 + #51 跨 run 可比性）。
- **现象**：驱动脚本历来自述并 echo「chunkradius=56」，历史文档/交接一并转述「chunkradius=56 → 4225 chunks」；实际 Chunky 1.4.40 **无 `chunkradius` 子命令**（回显 `Incorrect argument`），区域一直是**默认 radius 500 方块 = 4225 chunks**（逐臂核对一致）。
- **根因（机制）**：口径声明来自**驱动脚本自述**而非**被测程序回显**——脚本把「我发了这条命令」当「参数生效」；Chunky 对未知子命令不中断脚本、只回一行 `Incorrect argument`，脚本自己的 echo 文案遂成第二真相源，并被下游继续转抄（scout 成本图 `pipeline-cost-map.md` §0 亦照抄失真措辞）。
- **判据（MUST）**：① **口径声明（region 规模/采样范围/参数集）MUST 来自被测程序回显或日志，不能来自驱动脚本自述**；脚本自己 echo 的「已设置」文案零证据力（与 #37/#81「生效证据必须行为化」同族）。② 转述历史口径前先核被测程序回显（#90 转抄漂移家族——失真措辞会跨文档存活）。③ 本例各臂都吃同一默认值故**基线仍可比**（4225 逐臂一致）；**若只有部分臂受影响，就会直接变成假差异**（#20/#53「默认值当公理」的风险面）——这正是此类口径失真必须记录而非「无害」的原因。
- **家族索引**：#83（性能分母分场景——本条为「分母口径被脚本自述污染」形态）、#51（跨 run 可比性——同默认口径才可比）、#37/#81（生效证据行为化）、#20/#53（默认值当公理）、#90（转抄漂移）。
```

---

## C build-tooling 插入段

**目标文件**：`knowledge/discovered/build-tooling.md`（现网末尾 = `### #27 家族补充案例（第三形态）`，共 887 行）。
**插入位置**：**全部追加到文件末尾**（即 `#27 家族补充案例（第三形态）` 之后），顺序：C1(#46) → C2(#47 简记) → C3(#48 简记)。

### C1 — 追加到文件末尾

```markdown
## 发现 #46: 沙箱内 WMI/CIM cmdlet 会阻塞、JVM attach 工具一律拒访——编排脚本配方：进程属性识别 + .NET `Process.Threads`（260910-04）

- **发现时间 / 置信度 / module**：260910-04；确定（本块一手实测：cmdlet 阻塞/报错 + `jcmd` IOException 双证）；build-tooling / 沙箱环境坑（错误优先）。
- **来源定位**：`.investigations/perf-regression-260910-04/record.md` §6 E1/E4；驱动脚本 `.tmp/perf-reg-260910-04/*.ps1`。
- **现象（五段式·现象）**：① 跑批脚本「启动了服务器但从不发 RCON 命令」，进程挂死，服务器日志无 RCON Client 行；② `Get-CimInstance` 报「无法从客户端访问 CIM 资源」；③ `jcmd <pid> Thread.print` 返回 `IOException: 拒绝访问`，dump 文件 8 字节（空壳）。
- **根因（机制）**：① 本沙箱**没有 WMI/CIM 后端**——依赖它的 cmdlet（`Get-NetTCPConnection` 等）不是抛异常而是**阻塞**（失败形态 = 静默挂起）；② 沙箱**禁命名管道**，JVM attach 机制走命名管道 ⇒ `jcmd`/`jstack`/`jmap` 一类 attach 工具**一律不可用**（与命令参数/权限无关）。
- **定位**：手动 RCON 可用 + 日志无 RCON Client 行 ⇒ 卡点在脚本「发现服务器/端口」步；`Get-CimInstance` 直接复现 CIM 后端缺失；jcmd 报错文本 + dump 8 字节确认 attach 失败。
- **修复**：编排脚本**禁用一切 WMI/CIM cmdlet**，改用「WorkingSet 最大 / CPU 最高 的 java 进程」识别服务器；线程状态观测改 .NET `Process.Threads`（`ThreadState`/`WaitReason`）。
- **教训/判据（可复用）**：
  1. **沙箱可用性判据必须实测，不能凭常识**——被拦的形态可能是**阻塞**而非异常；脚本静默挂死先怀疑「某步 cmdlet 后端不可用」，而不是先查被测程序。
  2. 需要被禁通道（命名管道 / CIM / attach）的能力，开工前先做**最小可用性试跑**，再决定观测面（本块为此多绕一轮）。
  3. 「产出文件已生成」≠「有内容」——8 字节 dump 是假证据（核大小/内容；同族 #42 瞬时拒访，另有本课题错误台账 E5「日志 tail 滞后 → 结论取落盘终值」）。
  4. 替代面清单：**进程识别** → 进程属性枚举（WorkingSet/CPU 排序）；**线程状态** → .NET `Process.Threads`；**服务端口** → 被测程序自身日志/RCON 可达性探测（不要用 WMI 网络 cmdlet）。
- **家族索引**：#42（沙箱拒访/瞬时环境错误）、#21（编排脚本外部状态依赖）、workflow-patterns #106（冻结类故障观察法——本条为其 dump 通道的沙箱替代面）。
```

### C2 — 紧随 #46 之后追加

```markdown
## 发现 #47 简记: gradle `-P`→`-D` 映射的「作用域」坑——`run` 只在 `loom.runs` 闭包内可见，写进 `tasks.matching{}` 会配置期失败（260910-04）

- **发现时间 / 置信度 / module**：260910-04；确定（一手构建日志 + 移回后映射生效）；build-tooling / 构建配置（#8/#19 家族新形态——**现网该家族已计至「第四形态」（260908-10 补充案例），故本条按序计为第五形态**；任务书简述的「第三形态」与现网计数不符，以现网为准）。
- **来源定位**：`.investigations/perf-regression-260910-04/record.md` §6 E2 + §5；`runtime/1.21.6/java/build.gradle`。
- **现象**：构建日志 line 235 报 `Could not get unknown property 'run' for task ':runServer'`（**配置期**失败，runServer 起不来）。
- **根因（机制）**：映射被写进 `tasks.matching { it.name == 'runServer' }` 闭包——`run` 只在 `loom.runs` 的 `benchVmArgs` 闭包作用域内可见；`tasks.matching{}` 的闭包委托对象是 Task，无 `run` 属性。这是「参数接线」家族的**第三维（作用域）**：通道对（property 通道）、名字对（映射行名）、**位置错（闭包）**→ 仍不通。
- **修复**：映射移回 `benchVmArgs` 闭包（`-Pmixlog / -Pchunktime / -PmaxBgThreads / -PcaMin / -PestL2 / -PcaCap`）。
- **教训/判据**：
  1. 新增 `-P`→`-D` 映射 MUST 核**三查：通道 / 名字 / 闭包位置**（#8/#19/#25/#32 家族的第三维）。
  2. 本形态是**响亮失败**（配置期 unknown property），优于家族的静默不生效形态——**配置期报错先查闭包作用域，别去查参数名**。
  3. 「编译过/配置过 ≠ 接线生效」通用判据不变：生效证据 = 被测程序回显/行为化日志（#81/#37），构建成功不构成接线证据。
- **家族索引**：#8、#19、#25、#32、#81/#37。
```

### C3 — 紧随 #47 之后追加

```markdown
## 发现 #48 简记: `merge_index.py` 对「顶层 list 形态」的 index-entry.yaml 片段崩溃——片段 schema 契约不统一（260910-04）

- **发现时间 / 置信度 / module**：260910-04；**机制部分 Degraded**（本 subagent 不能跑命令，未亲自复现）；build-tooling / 工具坑（index 合并链）。
- **来源定位**：`.artifacts/8576-24blocks/biome-fix/index-entry.yaml`（本次直读确认其顶层是 `- id: ...` **YAML 序列**，供粘贴到 `entries` 之下）；崩溃记录 = 主会话按 `ref_merge_index` 合并该片段时实测 `AttributeError: 'list' object has no attribute 'get'`（一手运行记录，本稿未复现）。
- **现象**：`merge_index.py` 合并上述片段时抛 `AttributeError: 'list' object has no attribute 'get'`。
- **根因（机制）**：**片段 schema 契约不统一**——worker 交付的是「entries 追加片段」（顶层为序列），而合并脚本假定顶层是**单个 entry 映射**（对解析结果直接调 `.get(...)`）；序列对象没有 `.get` ⇒ AttributeError。即「脚本假定 vs 交付形态」错配，属 #8 家族的**契约维**形态。
- **修复（建议）**：统一 fragment 契约为「顶层 = 单个 entry 映射」（或让脚本显式接受 list 并逐条合并）；应用前把 list 形态片段改写为逐条映射，或主会话手工合并；脚本对输入形态 SHOULD 显式校验并给可读错误。
- **教训/判据**：① 交付 + 合并的组合 MUST 钉死**片段 schema**（顶层类型/字段名/嵌套层级），写进 `core.artifact` §5.1 的片段约定并由脚本强制；② `AttributeError: 'list' object has no attribute '<mapping method>'` 是「工具假定映射、实际收到序列」的签名，遇到直接查片段顶层类型即可，不必调试合并逻辑；③ 交付方声明的形态（文件头注释「追加到 entries 末尾」）不等于脚本接受的形态。
- **家族索引**：#8/#19/#25（契约/接线维）、#33 家族（交付形态与消费方不一致）。
```

---

## D INDEX 追加段

**目标文件**：`knowledge/INDEX.md`。
**插入位置**：**追加到文件末尾**（现网末尾 = `> 260910-03 追加：…` 段之后），保持 `> 260910-04 追加：…` 的单段风格。

```markdown
> 260910-04 追加：workflow-patterns 新增**发现 #107（最高价值·错误优先）**（「同步接管阻塞单车道调度器」——把重活同步做在被调度器串行化的执行点上，管线并行度塌成 1：MC 1.21.6 worldgen 只有一条 `SimpleConsecutiveExecutor("worldgen")` 且 `ChunkTaskScheduler` 同时只允许 1 个 entry 在飞，47ms native 生成同步做在 `populateNoise` HEAD → 全世界并行度 1.000，Σ(车道内工作)=269.1s ≈ wall 283-290s（93%），而 vanilla 用 `supplyAsync(...,"wgen_fill_noise")` 把同量级重活甩给 23 线程池、车道只做 µs 级调度；判据 = ① 性能回归先算「被串行化执行点内工作量之和」vs wall，比值≈1 即并行度塌陷 ② 「池调参无效 + CPU 极低 + 线程大量 park」= 单车道症状三联 ③ 接管类优化必须对齐原实现的**同步/异步形态**，不能只对齐语义正确性）+ **发现 #108**（Σ/wall 判据使用要点——同批同臂才可比；跨 run ±20% 摆动时比值仍成立而**绝对值不可引用**；须与进程 CPU/核数联用：1.27-1.7 核 vs vanilla 4.93 核）+ **发现 #109 简记**（臂无关「每 chunk 线程时间」尺子——`ChunkGenerator.generateFeatures` HEAD 同线程间隔点两侧同尺，coreswap 1422ms vs vanilla 299ms = 4.76× 与 wall 比 5.34× 同阶）+ **#83/#51 家族补充案例**（驱动口径自述 ≠ 生效——Chunky 1.4.40 无 `chunkradius` 子命令（回显 `Incorrect argument`），实际默认 radius 500 = 4225 chunks；脚本自己 echo 的「已设置」文案不构成生效证据，#37/#81/#90 同族）；错误台账 `.investigations/perf-regression-260910-04/perf-regression-errors.md` 新增 **E1-E8**（沙箱 WMI/CIM cmdlet 阻塞 → 脚本静默挂死 · gradle `-P` 映射**作用域**错位（`run` 未定义）· 并发两个 bench run 互抢世界/端口双 FAIL · 沙箱 JVM attach 拒访 → 改 .NET `Process.Threads` · 日志 tail 读取滞后 → 结论以落盘终值为准 · 两臂口径不对等（处理臂 8450 行 per-chunk 日志，headline −13%）· 机制命名「在等依赖」被一手源码证伪（getNow+thenRun 无 join → 单车道）· 驱动口径自述≠生效）；build-tooling 新增**发现 #46**（沙箱 WMI/CIM cmdlet 阻塞 + JVM attach 全拒访——编排配方：进程属性识别 + .NET `Process.Threads`）+ **发现 #47 简记**（gradle `-P`→`-D` 映射的**作用域**坑——`run` 只在 `loom.runs` 闭包内可见，写进 `tasks.matching{}` 配置期失败；#8/#19 家族**第五形态**）+ **发现 #48 简记**（`merge_index.py` 对顶层 list 形态 index-entry.yaml 片段崩溃 `AttributeError: 'list' object has no attribute 'get'`——片段 schema 契约不统一，机制侧 Degraded、建议改写为单映射片段复核）。来源：`.artifacts/perf-regression-260910-04/verdict-260910-04.md`（candidate，待 judge + 用户 confirmed）+ `.investigations/perf-regression-260910-04/{record.md,pipeline-cost-map.md}` + `candidates/{b1-choreography.md,b2-writeback-sideeffects.md}`（b2 = **REFUTED**：写回副作用 ≤3.7ms/chunk = 0.37%，非 948ms gap 主体）。
```

---

## 自检清单（按 SUBAGENT-KNOWLEDGE-GUIDE §四 逐条打勾）

- [x] **先过价值门**：A 错误台账 E1-E8 = 高价值（错误链/判错经验）→ 详写五段式；B1 #107 / B2 #108 = 高价值（可复用判据）→ 详写；B3 #109 = 中价值（测量手法）→ 简记；B4 = 高价值（口径判据 + 家族修正）→ 详写；C1 #46 = 高价值（沙箱坑，错误优先）→ 详写；C2 #47 / C3 #48 = 中价值（工具/构建坑）→ 简记；D = 索引。**未收录**低价值项见文末说明。
- [x] **每个错误都有五段式**：A 的 E1-E8 均含 现象 / 根因 / 定位 / 修复 / 教训，无「只记已修复」条目（模板见 E2/E3，家族叙述按 build-tooling #8/#19/#25 写法内嵌「教训」段）。
- [x] **根因是机制层面**：E1 = 沙箱无 CIM 后端致 cmdlet 阻塞（非报错）；E2 = 闭包作用域（委托对象不同）；E3 = 编排状态机缺终态门；E4 = 命名管道禁用致 attach 通道不可用；E5 = 重定向缓冲 + 元数据刷新；E6 = 口径不对等；E7 = loader 非阻塞 + 单车道串行化；E8 = 声明源错位（脚本自述 vs 程序回显）。
- [x] **定位含诊断方法/工具**：E1（手动 RCON 可用 + 无 RCON Client 行 + Get-CimInstance 复现）、E2（构建日志配置期异常 + 对照 benchVmArgs 写法）、E3（重复 FAIL 行 + 双进程）、E4（报错文本 + dump 8 字节）、E5（同一文件两分钟后重读 + mtime 未变）、E6（diff 驱动脚本 + 数 8450 行日志 + c1 A/B）、E7（一手源码逐点 + 实测三连签名）、E8（被测程序回显 + 逐臂 chunk 数核对）。
- [x] **判错经验已沉淀**：E7 教训三条（先算 Σ/wall、断言阻塞前 grep join/get、机制命名错误误导修复方向）、E5（实时 tail 不作卡死判据）、E6（口径不对等先归零）、E8（口径声明取被测程序回显）、E1（沙箱可用性实测）均为可复用判据。
- [x] **被排除假说有标注（❌/⚠️）**：b2 写回副作用候选 **REFUTED** 已在 B1 证据段与 D 段显式标注；E7 的「等依赖」列入排除清单；上流限流（`ThrottledChunkTaskScheduler`）标 ⚠️ 未验证并保留 @idk；#109 未做车道 busy 直接计数 → 机制侧 Degraded 声明。
- [x] **写入载体正确**：错误 → `.investigations/perf-regression-260910-04/perf-regression-errors.md`（独立成篇，新文件）；通用可复用判据 → `knowledge/discovered/workflow-patterns.md`（#107/#108/#109 + #83/#51 家族补充）；工具/构建坑 → `knowledge/discovered/build-tooling.md`（#46/#47/#48）；索引 → `knowledge/INDEX.md` 追加段。**未**动 `docs/`（本块为性能归因，无主题篇结论落盘）。
- [x] **无复用价值的结论未写 docs**：见文末「判定低价值未收录」清单（一次性数值/单次读数/已被取代的候选细节）。
- [x] **错误台账末尾速查表已同步**：A 稿末尾「错误→根因 速查表」含 E1-E8 共 8 行，一行一错，与正文条目一一对应。
- [x] **数字来自一手记录，无编造无占位符**：全部数字取自 `verdict-260910-04.md` / `record.md` §2 / `b1` §D / `b2` §1；跨文档不一致处（任务书「第三形态」vs 现网「第四形态」）已显式标注以现网为准，未替主会话拍板。
- [x] **格式与目标文件末尾现状对齐**：已先读 `workflow-patterns.md` 末条（#106 简记）与 `build-tooling.md` 末条（#27 家族补充案例 第三形态）；条目字段沿用现网近期风格（时间/置信度/module + 来源定位 + 观察/证据/判据 + 家族索引），详写条用 `## 发现 #N:`、简记条用 `### 发现 #N 简记:` / `## 发现 #N 简记:`。
- [x] **只读纪律**：本次仅新建本草稿文件；未修改 `knowledge/`、`docs/`、`.artifacts/` 任何现网文件；未运行任何命令。

### 判定「低价值」而未收录的项（按记录价值门 §〇）

1. **本块的具体读数快照**（c1 234s / c2 279s / c3 未跑完 / A 组 156.2/135.2/166.0/58.3/47.7ms / A2 组 39.86/39.43/38.55/42.66ms）——只在 verdict/record 中作证据，不单独进 discovered（属「某次跑出的数值」，且 A/A2 组数字已在 B1 证据段按需引一次）。
2. **sha/hash 哨兵值**（dll `abd7d8893d22e030…`、`6908dbfc…`/`115641b8…`）——属现成复现锚，留 verdict/record，不进知识库（下一次重编即失效，#10 补充案例已确立「sha 基线必须带构建上下文」）。
3. **b1 建议测量 1-8 的逐条预测值**（如「vanilla N=1 ⇒ ~200s+」「queue>0 预测」）——属未验证预测，留候选文件；只有「Σ/wall 判据 + 池缩放方向」这类已实测结论进 #107/#108。
4. **b2 的 @idk-1~11 明细**（air 写占比、GC 停顿、光照计时等）——未测清单，属课题内部待办，不进通用知识库（其结论「写回 ≤3.7ms/chunk、后续阶段 ≈0」已在 B1 证据段引用）。
5. **Chunky 默认 radius 的具体 chunk 数换算细节与 region 中心坐标**——一次性区域定位，不进知识库（判据形态已收进 B4）。
6. **`merge_index.py` 崩溃的完整 traceback**——工具坑的判据已收进 C3，堆栈本身无复用价值。
