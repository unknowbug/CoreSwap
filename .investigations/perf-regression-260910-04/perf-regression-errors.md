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
