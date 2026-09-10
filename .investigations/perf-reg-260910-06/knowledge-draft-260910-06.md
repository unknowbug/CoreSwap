# 知识库草稿 — 区块 260910-06（1.20.1 R3 异步化 + 同构建态 A/B + 内容指纹门）

> **本文件 = 草稿，只供主会话应用；未改动 `knowledge/` 与 `docs/` 正文。**
> **格式规范**：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（先用价值门筛，再选载体；无复用价值的结论不写知识库）。
> **产出者**：core.worker / core.knowledge subagent（260910-06 Phase 3.5）；**无 shell**，数字复核限于文件级/抽样级（见 §F.4）。
> **编号依据（取追加序尾号，已读目标文件末尾）**：
> - `knowledge/discovered/workflow-patterns.md` 现网**最大号 = #112**（`### 发现 #112 简记` 在 `:1853`，`^## 发现` 正则漏抓 `###` 形态，已用 `^#+` 复核）⇒ 新增 **#113 / #114 / #115**（与 260910-05 的 #110-#112 不冲突）。
> - `knowledge/discovered/build-tooling.md` 现网**最大号 = #50**（`:939`）⇒ 新增 **#51 / #52 / #53**。
> - `knowledge/discovered/compiler-idioms.md` 现网**最大号 = #23**（`:329`）⇒ 新增 **#24**。
> **原料**：`.investigations/perf-reg-260910-06/errors-260910-06.md`（E1-E3 + 速查表）、`.artifacts/perf-reg-260910-06/verdict-260910-06.md`（candidate）、`.investigations/000-架构设计/架构计划-260910-06.md`（§14 执行中变更）、一手 `cmd-output/`（10 臂 `logs/` + `results.txt` + `diffs/` + `region-*` + `java-snapshot/post-260910-06/`）。

---

## A. `knowledge/discovered/workflow-patterns.md` 拟写入正文（追加到文件末尾，`:1872` 之后）

### 发现 #113（最高价值·错误优先）: 跨实现搬运「锁语义」必须成对核对——外层段锁与写路径自带内层锁是配套契约，只搬一半即**同线程自锁死**（260910-06）

- **发现时间 / 发现者 / 置信度 / module**：260910-06（实际 2026-09-10 20:16–21:4x，Get-Date 锚定）；主会话（实测 + 自建线程栈看门狗 dump 一手定位）；**candidate**（数据层决定性证据 = 线程栈 dump 内含同一把锁的「持锁帧 + acquire 帧」；同步/异步两条臂对照排除竞争变量；修复后两臂跑通且有量级与内容门）——confirmed 留用户；workflow-patterns / 跨实现搬运纪律（build-tooling #8/#47「接线/映射错觉」家族的**锁语义维**）。
- **来源定位**：`runtime/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`（首版含 `wgLockSections`/`wgUnlock` + `whenCompleteAsync` 完成跳回；post 快照已为无外层锁形态）；`.../wg/bench/CppBridge.java:519`（`writeChunk` → `setBlockState(x,sy,z,st)` 4 参重载）；错误台账 `.investigations/perf-reg-260910-06/errors-260910-06.md` E1（含 dump 栈原文与 vanilla 对照 `NoiseChunkGenerator.java:337-346`）；架构计划 `.investigations/000-架构设计/架构计划-260910-06.md` §14.1；判决 `.artifacts/perf-reg-260910-06/verdict-260910-06.md` §2/§4.1/§7.5。
- **现象（错误链）**：Chunky 任务启动后**零进展**，且**同步臂与异步臂都卡**——`[Chunky] Task running for minecraft:overworld. Processed: 0 chunks (0.00%), ETA: 0:00:00, Rate: 0.0 cps, Current: 0, 0`（首轮 `oa-r1`/`os-r1` 停滞行，一手 `cmd-output/results.txt:1-6`）；同窗口服务器 CPU 累计 **11-13s / wallgen 204s ⇒ 0.05-0.06 核**（进程原地耗电水平；errors E1 另有 12s 窗口活采样 0.1s ⇒ ≈0.01 核，两者同量级不同窗口），`run\world\region` mca mtime 停在任务起始时刻，**等 40min 不复原**（不是慢，是停）。
- **根因（机制）**：把 vanilla 1.20.1 `NoiseChunkGenerator.populateNoise`(:337-346) 的**外层 sections 锁**照搬进自家接管段，但自家写回路径**自带逐次锁**：
  `CppBridge.writeChunk:519 sec.setBlockState(x,sy,z,st)` → **4 参重载（`lock=true`）** → `ChunkSection.setBlockState(x,y,z,state,true)` → **`PalettedContainer.swap(x,y,z,value)`（`this.lock(); try{…} finally{ this.unlock(); }`，`PalettedContainer.java:142`）** → `LockHelper.lock()`（`LockHelper.java:42`，**非可重入 Semaphore**，acquire 阻塞）
  ⇒ 同线程**已持有**该 section 的锁时，写回内部再次 `lock()` = **永久 park**（无异常、无日志）。vanilla 之所以能加外层锁，是因为它自己的 fill 在持锁后改用 `setBlockState(..., lock=false)`（`swapUnsafe`，**不加内层锁**）——**「外层段锁 + `lock=false` 写」是一个成对契约**；本工程写回用的是**加内层锁**的那条重载 ⇒ 两者不可混搭。
- **证据（怎么定位的，可复用）**：
  1. **双臂对照排除竞争变量**：同步臂**也**卡 ⇒「异步分派 / `whenCompleteAsync` 完成跳回」被排除，嫌疑立即收敛到**两臂共有**的新增项（= 外层锁）。此步不需要 dump 也成立，是最便宜的第一刀。
  2. **dump 直读阻塞链（决定性）**：沙箱 `jstack`/`jcmd` 全拒访（attach 被拦，build-tooling #46）⇒ 自建门控看门狗自打线程栈（#114），一次 dump 读出 `Worker-Main-*` `state=WAITING` 且**同一把锁出现两次**：`LockHelper.lock(:42)` ← `PalettedContainer.lock(:45)` ← `PalettedContainer.swap(:142)` ← `ChunkSection.setBlockState(:63)` ← `CppBridge.writeChunk(:519)` ← `fillChunk(:394)` ← …`wgPopulateNoise`（外层持锁帧在上方）。
- **判据（MUST，可复用）**：
  1. **照抄参考实现的「锁语义」前，MUST 先核自家写路径的锁语义与可重入性**——查「我这条路最终调到的写接口带不带锁」；自带锁则**不要**在外层再加，或整段改用 `lock=false` 系列接口（二选一，不可混）。与 build-tooling #8/#47「参数映射/接线错觉」同族：**跨实现搬运必须成对核对，不能只搬可见的一半**。
  2. **判别签名（三件齐）= 两条臂都卡 + CPU ≈ 0（进程原地耗电）+ dump 里同一把锁出现「持锁帧 + acquire 帧」**。这是**单线程非可重入自锁死**，不是死锁（两把锁互等的 ABBA 形态），更不是性能问题——不要派性能诊断轮次。
  3. **「卡死 ≠ 慢」：先量 CPU 增量 + 进展探针，再谈性能**。停滞定性只需两个廉价量（进程 CPU 累计增量 + 任务进度回显）；驱动 MUST 带**早停探针**（本案 `Processed` 恒 0 超 180s 即落盘诊断并退出，替代 40min 盲等）。
  4. **同族附注（本块同源，API 形态维）**：搬运同一机制时**同名 API 的形态也可能不同**——1.20.1 `Util.getMainWorkerExecutor()` 返回 `ExecutorService`（`Util.java:229`，架构计划 §2 F4），1.21.6 才是 `NameableExecutor`/`.named(...)`（照抄 `.named()` 在 1.20.1 编译不过）。即**锁语义 / 签名 / 参数映射 / API 形态**是「跨实现搬运核对表」的四栏，缺任一一栏都可能静默或响亮失败。
- **家族索引**：build-tooling #8/#47（映射/接线错觉）、#107/#108（形态错即性能错——本条为「形态错即卡死」的更强形态）、#100/#102（归因须在真实通路上核对——本条是「卡点根本不是性能」的对照面）、#114（本条定位所依赖的诊断件）；compiler-idioms #24（本条所用锁语义事实的惯用法简条）。

### 发现 #114 简记（中价值）: 沙箱禁 JVM attach 时，「门控看门狗自打线程栈」是停滞定位最廉价、往往也是唯一的手段（260910-06）

- **发现时间 / 发现者 / 置信度 / module**：260910-06；主会话（一手实测：attach 全拒 + 看门狗 dump 一步读出阻塞链）；**确定**（本机沙箱复现 + 应用成功一次）；workflow-patterns / 停滞定位手法（build-tooling #46 的**方法侧**配对条）。
- **来源定位**：`runtime/1.20.1/java/src/main/java/wg/bench/StallWatch.java`（59 行；post 快照 `.investigations/perf-reg-260910-06/java-snapshot/post-260910-06/src__main__java__wg__bench__StallWatch.java` + `SHA256SUMS.txt` 已登记 sha256）；挂点 `CppBridge.java:88`（`init` 末尾 `StallWatch.start()`）；栈解析件 `cmd-output/parse_stallwatch.py`；应用实例 = errors E1（`[STALLWATCH]` 段）。
- **手法**：`-Dcoreswap.stallwatch=<秒>`（**默认完全关**：只在启动读一次 prop，未设即 return ⇒ 生产零成本）；打开后起 daemon 线程（名 `coreswap-stallwatch`），每 N 秒把 `Thread.getAllStackTraces()` **全量**打 stdout（前缀 `[STALLWATCH]`，逐线程含 `name/id/state/daemon` + 每帧 `at`），随 gradle 日志落盘即可 grep；幂等启动（`started` volatile + `synchronized` 守卫），dump 段整体 `catch (Throwable ignored)`——**看门狗自身绝不打断主流程**。
- **价值**：沙箱禁 attach（`jstack`/`jcmd` exit 1，build-tooling #46）时，「卡死」类故障本来**没有观测面**；自打栈把线程栈变成**日志内证据**——本案一次 dump 即读出「持锁帧 + acquire 帧」阻塞链，**成本比反复猜机制低一个量级**（对照：纯日志/`chunky progress` 只能判「停」，不能判「为什么停」）。
- **陷阱 / 纪律**：① **默认关 + 间隔可调**（诊断门控纪律，同 #11/#53 家族：门控不得改生产语义与成本）；② **间隔即成本**——全线程栈 dump 在重并发下不便宜，按「等待窗口」取值（本案 30s），不要 1s 级常开；③ dump 是**瞬时快照**，单次可能打偏 ⇒ 与 #106「双 dump 间隔观察」配合：两帧同点 = 钉死，帧在移动 = 进展崩塌（另一态）；④ 输出走 stdout 需独立前缀（`[STALLWATCH]`）与既有日志噪声区分。
- **家族索引**：build-tooling #46（沙箱 attach 拒访——本条为其**方法侧**配对）、workflow-patterns #106（双 dump 三态观察法——本条为其 dump **通道**在禁 attach 环境的落地）、#11/#53（诊断门控默认关）、#113（本条能力的首个应用实例）。

### 发现 #115 简记（中价值）: 噪声无关的内容指纹门——对「每 chunk 原生输出」取位置敏感指纹跨形态逐 chunk 比对，绕开 run 级非确定淹没（260910-06）

- **发现时间 / 发现者 / 置信度 / module**：260910-06；主会话（仪器 + A/B）+ 本稿 subagent（日志抽样独立复核，见下「附加证据」）；**candidate**（覆盖面 = 两条 mixlog 臂日志中的**全部** overworld 接管 chunk，无抽样；本稿抽查 5 chunk 逐字一致）；workflow-patterns / 等价门手法（#111 的**绕噪声**配对条、#52 确定性 dump 载体家族的内容域形态）。
- **来源定位**：仪器 = `CppBridge.wgBufHash`（**FNV-1a 64**，对 `fillBlocks` 写回 buffer **逐 int 位置敏感**混入）+ 打印 `[WG-CONTENT] chunk(x,z) hash=<hex> nz=<n>`（`-Pmixlog=1` 门控，`MIXLOG` 门控纪律同 #11）；判决 = `.artifacts/perf-reg-260910-06/verdict-260910-06.md` §4.1；一手日志 = `cmd-output/logs/osS-r2.log`（sync）/ `oaS-r2.log`（async）；region 门旁证 = `cmd-output/diffs/*.txt`（7 对）。
- **观察**：1.20.1 载体 **run 级非确定 0.0209-0.0467%**（同形态锚；跨形态 0.0576-0.0648%），与待测信号**同阶** ⇒ region 逐块门检验力低（#111/#51 的形态）。把门面从「存档 region 最终状态」上移到「本实现**每 chunk 的原生输出**」后：sync（`osS-r2`）× async（`oaS-r2`）**4140/4140 chunk 指纹全等、0 不一致**，且臂内无「同 chunk 多份互异指纹」（= 无重复/非确定重生成）。
- **判据（可复用）**：
  1. **当载体 run 级非确定淹没逐块门时，先在实现侧找可指纹化的「每 chunk 原生输出」并把门移到那里**——对写回 buffer 逐元素做位置敏感指纹（本案 FNV-1a 64 逐 int），跨形态/跨臂按 **chunk 键**比对；该门**不经过** Java carver/feature 调度序 ⇒ **噪声无关**，判据回到二值强判据（0 不一致），而不是「落在噪声带内」（#111 的读法困境整个消失）。
  2. **该门顺带回答「有无跨 chunk 状态」**：若输出依赖完成序/跨写，指纹会按臂分叉；全等 ⇒ 当前出货语义下该实现 = **per-chunk 纯函数**（#110 对「纯 memo + 纯函数」静态结论的**行为侧对偶**）。
  3. **覆盖面必须随判据声明**：本案指纹门只覆盖 overworld 接管 chunk（4140；nether/end 未开 mixlog = 递延，verdict §7.1）；`nz` 计数可作**非空气块数 sanity**（本案 ~22.8k-34.7k）——空/零输出会立刻在 `nz` 上暴露。
  4. 与 #111 的分工：**噪声锚解决「region 门能读多严」，指纹门解决「根本不需要噪声锚」**——能上移判据面时优先上移；不能上移（跨实现对照、最终存档正确性）才回退「同配置 run-to-run 锚」。
- **附加证据（本稿 subagent 一手 grep 抽样复核，非转述）**：`osS-r2` × `oaS-r2` 抽 5 个 chunk 逐字比对一致，例：`chunk(-29,-28) d689a432ad9da3b1 nz=31600`、`(-28,-28) 5c4743023c45ec1a nz=29851`、`(-30,-28) 74ac513c2f27c1e6 nz=32229`、`(-33,-33) c86346be9653c18a nz=32172`、`(-22,-22) d5ff3b43d0804e8f nz=25056`。⚠️ 精确**去重计数**（4140）取自 verdict，本稿无 shell 未重算（两臂 `[WG-CONTENT]` 行 grep 计数同量级 ≈4141，含少量非该行匹配）——引用时以 verdict 为准并随 §9.7 覆盖面声明。
- **家族索引**：#111（噪声锚分形态/维度——本条给「绕开噪声」的上移形态）、#52（确定性 dump 载体——本条为内容/指纹域形态）、#51（噪声与信号同阶）、#110（「纯 memo + 纯函数」静态结论——本条为其行为侧对偶）、#33/#36（可比性 / 执行体三元组）。

---

## B. `knowledge/discovered/build-tooling.md` 拟写入正文（追加到文件末尾，`:956` 之后）

## 发现 #51: 多项目 gradle 构建下**裸任务名会级联**到子工程——1.20.1 根项目 `runServer` 连带 `:content-test:runServer`（第二服务器共用同一 `run` 目录）（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手日志任务行 + 对照臂差异；错误文案一手）；build-tooling / gradle 任务接线（#8/#47「接线/映射错觉」家族**第六形态：任务名作用域**维）。
- **来源定位**：错误台账 `.investigations/perf-reg-260910-06/errors-260910-06.md` E2；驱动就地注释 = `.investigations/perf-reg-260910-06/cmd-output/run_arms_1201.ps1:51-53`（`# ⚠️ 必须用 :runServer …`）与修好后的调用 `:54`（`@(":runServer") + $extra`）；对照臂 = 1.21.6 侧只有 `> Task :runServer`。
- **现象**：1.20.1 臂日志出现 `> Task :content-test:runServer`（**两次**），随后 `Failed to start the minecraft server … testcontent.TestContentMod.<clinit> … IllegalStateException: This registry can't create intrusive holders`；该子工程服务器**共用同一个 `run` 目录 / 同一个 world**；1.21.6 臂的 `^> Task` 列表无此行。
- **根因（机制）**：在仓库根执行**裸任务名** `gradle runServer` 时，gradle 名称匹配命中**所有子工程**的同名任务；1.20.1 的 `content-test` 是独立 loom 工程，其 `runServer` 必然失败（测试内容 mod 不能独立启动），且与主服务器抢同一 world 目录。
- **定位（便宜的自证）**：`Select-String -Pattern '^> Task'` 列出 gradle **实际执行**的任务（级联任务是一行显式日志，**不看必漏**），与对照臂任务行集合比对即可判。
- **修复**：驱动改用**根项目限定名** `gradle :runServer`。
- **教训 / 判据**：① **多项目构建里任务名 MUST 限定到根项目**（`:<root-task>`）；② 「我以为我跑的就是那个任务」的接线错觉家族（#8/#9/#19/#47）在此多一维——**任务名作用域**，判据 = `^> Task` 列表与对照臂不一致；③ **主服务器照跑不构成「批次干净」**：与之并存的子工程任务失败/抢资源可以同时发生；④ ⚠️ **归因链留档（同形不同因）**：该级联一度被当作「`Processed` 恒 0」的嫌疑（驱动注释 `:51-53` 保留了当时的判断），最终由 E1 的线程栈 dump 定因为**自锁死**——级联是**真坑但不是本块卡死根因**。「现象同形」不等于「根因同一」，两者各自需要独立证据，否则会把 A 的修复记成 B 的根因。
- **家族索引**：#8/#9/#19/#25（`-P`→`-D` 接线链）、#47（映射**作用域**——本条为**任务名作用域**姊妹条）、#15/#28（run 口径参数集）、workflow-patterns #81/#37（生效证据行为化——本条证据面 = 任务行）。

## 发现 #52: Chunky 任务状态**跨 run 持久化**在 `config/chunky/tasks/`，删 `run/world` 不清它 ⇒ 残留任务让 `chunky start` 静默不开始，伪装成同形不同因的「卡死」（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手回显字面量 + 清目录后同臂跑通）；build-tooling / 验证载体（#26 Chunky 载体系列，#18 残留态家族第三形态）。
- **来源定位**：错误台账 `.investigations/perf-reg-260910-06/errors-260910-06.md` E3；驱动修正 = `.investigations/perf-reg-260910-06/cmd-output/run_arms_1201.ps1:44-46`（每臂 `Remove-Item run\run\config\chunky\tasks -Recurse -Force`，紧邻既有的删 world `:43`）；Chunky **1.3.146**。
- **现象**：`cm-r2` 臂 `chunky start` 回 **`[Chunky] A task was already started for this world. To continue running it, type '/chunky continue'. To start a new task, type '/chunky confirm'.`**——该臂因此**根本没有新任务**，最终表现为「`Processed` 恒 0」，与 E1 的卡死**现象同形、机制不同**。
- **根因（机制）**：Chunky 把任务状态落到 `run\config\chunky\tasks\<namespace>\<dim>.properties`（实测 `chunks=0, cancelled=false`），**删 `run\world` 不会清它**；驱动原来只删 world ⇒ 上一臂的残留任务状态污染本臂。
- **定位**：回显字面量本身即判据；再核 `config\chunky\tasks\**` 的 `cancelled/chunks` 字段。
- **修复**：驱动每臂开跑前清 `run\config\chunky\tasks`。
- **教训 / 判据**：① **「清环境」清单 MUST 覆盖工具自己的状态目录**，不只世界/存档目录——否则「上一臂的残留」会伪装成「本臂的失败」，制造**同形不同因的假因果**（#18「导出产物在盘 ≠ 本次生成」的第三形态）；② 遇到「工具明明该开始却什么都没做」，**先读工具自己的回显/状态文件**，再怀疑被测管线（#37/#81 生效证据行为化家族）；③ 同形现象（`Processed` 恒 0）在本块集齐三种因（E1 自锁死 / E3 残留任务 / 级联干扰的初判），**判据必须能区分因，不能只看现象**。
- **家族索引**：#18（残留世界缓存制造假象——本条为其**工具状态目录**形态）、#26（Chunky 区域级载体——本条为其前置清理条件）、#46（沙箱/工具环境坑）、workflow-patterns #113（同形不同因判据）。

## 发现 #53: region 对拍工具在**无 `xPos` 键的载体**上必须按「region 文件名 + 槽位索引」推导坐标，并**与其已验证版本逐行对齐 NBT reader**——手写 reader 把 TAG_Byte 读 8 字节 / TAG_Byte_Array 读 i4 长度会**静默少解析**（不报错）（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手：首版结果为 760 chunk 且不报错，对齐已验证版 reader 后自比 0 差 + 正对照非零 + common 非空）；build-tooling / MCA·NBT 工具链（#20 的**第二形态**：本题是「payload 长度读错 ⇒ 指针走错」，#20 是「指针根本不推进」）。
- **来源定位**：工具 = `.investigations/perf-reg-260910-06/cmd-output/diff_1201.py`（`payload()` `:35-60` 已注明「与 260910-05 已验证版 `diff_arms.py:15-36` **逐行一致**」，`:36-37` 留档首版 bug）；坐标推导 `:91-105`（`cx = rx*32 + (slot & 31)`、`cz = rz*32 + (slot >> 5)`）；自检 = `cmd-output/diffs/diff-self_os.txt`（`common=7703` / `diff=0 (0.0000%)`）；正对照 = `diffs/diff-ctrl_vanilla_x_async.txt`（122,375 块差）；错误台账 E4（**正文待补**，见 §F.1）。
- **现象**：本块首次手写的 NBT reader 使全域 chunk **只解析出 760 个**（判决记「7703 → 760」，工具注释记「7542 → 760」，**分母口径不一**见 §F.2）——**且不抛任何异常**；下游据此得到的「可比块数/差异率」全部失真（若不做自检，最坏形态是 common=0 的「全量一致」或大比例假差异）。
- **根因（机制）**：NBT `payload(r, t)` 的 tag→payload 映射写错两处：**TAG_Byte 当 8 字节读**、**TAG_Byte_Array 当 i4 长度读** ⇒ 位置指针**失步**（desync），解析从中途开始读 tag header，结果被静默截断/丢弃而非报错（同族：手写二进制 reader 无校验，错位不产生异常）。
- **定位（三步自检，MUST）**：① **自比（同臂 × 同臂）**必须 `diff=0` **且 common 非空**（本案 7703）——本次首版即在此穿帮（解析数远小于生成器自报 chunk 数）；② **灵敏度正对照**：拿**已知不同**的两臂比，必须报非零（本案 vanilla × async = 122,375 块差）——零比对数 = 假阴性嫌疑先查工具（#12/#13 家族）；③ **解析 chunk 数 ≈ 生成器自报数**（Chunky `Processed: 4225` + 邻区）。
- **修复**：与 **260910-05 已验证版** reader 逐行对齐（现网 `diff_1201.py:36-37` 就地注明来源与 bug 留档）。
- **教训 / 判据**：① **无 `xPos` 载体的坐标唯一来源 = region 文件名 + 槽位索引**（1.20.1；#20 已有同判据）——照抄「读 `xPos` 取键」会把**所有** chunk 丢掉 ⇒ common=0 的**全量假阴性**（工具 bug 伪装成「100% 一致」）；② **手写 NBT reader MUST 与已验证版本逐行对齐**，不做「凭记忆重写」——reader 是本项目对拍链的公共地基，错一处全域静默失真；③ 对拍工具首用 MUST 跑完三步自检再把数字当结论（**先查工具不查结论**）；④ 复用既有工具时**注释里钉死「与哪个版本的哪一段逐行一致」**（本工具已做），使下次改动可回溯。
- **家族索引**：#20（1.20.1 chunk NBT 解析两坑——本条为**第二形态**）、#10/#11（参照/解析产物核对以内容实测为准）、#27（静默退化家族）、workflow-patterns #12/#13（工具 bug 伪装成结论 / 空集先疑工具）、#111（分母语义随判据声明）。

---

## C. `knowledge/discovered/compiler-idioms.md` 拟写入正文（追加到文件末尾，`:340` 之后）

> **择一说明（任务书第 6 条）**：该事实的**载体选 compiler-idioms 而非 workflow-patterns**——理由是它与 compiler-idioms #12（mixin 包禁止非 mixin 类）、#17（Fabric 反射字符串不重映射）、#23（双 HORIZONTAL 数组）同型：**平台/API 的语义约束与惯用法（「是什么」）**，而 workflow-patterns #113 承载的是**搬运纪律与判据（「怎么办」）**。两条互引、不重复：本文只记「`swap()` 自带锁 / `LockHelper` 非可重入 / `lock=false` 成对」这一机制事实，方法面判据留在 #113。

## 发现 #24: MC 1.20.1 `PalettedContainer.swap()` 自带 `lock()/unlock()`（`LockHelper` **非可重入**）——`ChunkSection.lock()` 只与 `setBlockState(..., lock=false)`/`swapUnsafe` 配套（260910-06）

- **发现时间 / 发现者 / 置信度 / module**：260910-06；主会话（实测线程栈 dump + 一手调用链）；**candidate**（dump 决定性证据 + 调用链 file:line；vanilla 对照为源码直读）；compiler-idioms / Java（MC）API 惯用法与锁语义。⚠️ **module 侧来源声明**：`runtime/` 被 `.gitignore` 忽略（verdict §2），本条的类/行号来自 errors E1 与 verdict §14.1 内联的源码引证（一手读码记录），非 git 版正文。
- **来源定位（调用链）**：
  - `CppBridge.java:519` `writeChunk` → `ChunkSection.setBlockState(x, sy, z, st)`（**4 参重载 ⇒ `lock=true`**）
  - → `ChunkSection.setBlockState(x,y,z,state,true)` → **`PalettedContainer.swap(x,y,z,value)`**（`PalettedContainer.java:142`：`this.lock(); try{…} finally{ this.unlock(); }`）
  - → **`LockHelper.lock()`**（`LockHelper.java:42`，**非可重入 Semaphore**，`acquire` 阻塞）／`PalettedContainer.lock(:45)`
  - vanilla 对照：`NoiseChunkGenerator.java:337-346`（外层 `sections` 锁 + 持锁期 `setBlockState(..., lock=false)` 即 `swapUnsafe`，**不加内层锁**）。
- **语义（是什么）**：
  1. `ChunkSection.setBlockState(x,y,z,state)` 的 **4 参重载 = `lock=true`**，其内部会走 `PalettedContainer.swap()`，而 `swap()` **自己负责** `lock()/unlock()`（**写路径自带锁**）。
  2. 该锁实现（`LockHelper`）**不可重入**：同线程二次 `lock()` 不是计数 +1，而是**永久阻塞**（无异常、无日志、CPU≈0）。
  3. 因此 `ChunkSection.lock()`（外层持锁）**只在写路径改用不带内层锁的接口**（`setBlockState(..., lock=false)` / `swapUnsafe`）时才成立——**「外层段锁 + `lock=false` 写」是一个成对契约**：只加外层锁 = **自锁死**；只用 `lock=false` = **失去互斥**。
- **判据（可复用）**：① 复刻/接管类改动凡要**自行加段级锁**，MUST 先核该段内所有写接口是否**自带锁**（在被调方找 `lock()` / `try{…} finally{ unlock(); }`）；自带则不要在外层再加，或整段改用 `lock=false` 系列（二选一，不可混）；② 排查签名 = 线程栈里**同一把锁出现两次（持锁帧 + acquire 帧）** + 进程 CPU≈0（方法面判据见 workflow-patterns #113）；③ 跨版本/跨实现的搬运核对表把**锁语义**单列一栏（与 API 形态 / 签名 / 参数映射并列）。
- **家族索引**：workflow-patterns #113（主判据与错误链）、compiler-idioms #11（诊断门控在初始化器内唯一置位——同为「初始化/持锁期语义」类简条）、#12（mixin 包约束——同为「平台语义约束」简条）。

---

## D. `knowledge/INDEX.md` 拟追加行（追加到文件末尾 `:133` 之后，保持「`> YYMMDD-## 追加：…来源：…`」单段式）

> 260910-06 追加：workflow-patterns 新增**发现 #113（最高价值·错误优先）**（跨实现搬运**锁语义**必须成对核对——照抄 vanilla 1.20.1 的 `populateNoise` 外层 sections 锁（`ChunkSection.lock()`）撞上自家写回路径**自带**的 `PalettedContainer.swap()`(`:142`) → `LockHelper.lock()`(`:42`，**非可重入 Semaphore**) ⇒ **同线程自锁死**，两臂（sync/async）**都**卡：`Processed: 0`/`Rate 0.0 cps`、CPU 累计 11-13s / wall 204s ≈ 0.05-0.06 核、等 40min 不复原；vanilla 能加外层锁是因为它的 fill 持锁后改用 `setBlockState(..., lock=false)`（`swapUnsafe`）——**「外层段锁 + lock=false 写」是成对契约**；判别签名三件 = **两臂都卡 + CPU≈0 + dump 里同一把锁出现「持锁帧 + acquire 帧」**（单线程非可重入，**不是** ABBA 死锁、**不是**性能问题）；判据 = 搬锁前先核自家写路径锁语义与可重入性 + 「卡死 ≠ 慢」（先量 CPU 增量与进展探针、驱动带早停探针）+ 同族附注 API 形态维（1.20.1 `Util.getMainWorkerExecutor()` 返 `ExecutorService`，`.named()` 只有 1.21.6 有）+ **发现 #114 简记**（沙箱禁 JVM attach 时「门控看门狗自打线程栈」是停滞定位最廉价/往往唯一的手段——`-Dcoreswap.stallwatch=<秒>` 默认关、daemon、全量 `Thread.getAllStackTraces()` 带 `[STALLWATCH]` 前缀，一次 dump 读出阻塞链，成本比猜机制低一个量级）+ **发现 #115 简记**（噪声无关的**内容指纹门**——载体 run 级非确定 0.0209-0.0467% 淹没 region 逐块门时，对每 chunk 原生输出取 FNV-1a 64 **位置敏感指纹**跨形态逐 chunk 比对：4140/4140 全等 ⇒ 二值强判据且顺带证「per-chunk 纯函数」= #110 静态结论的行为侧对偶；判据面能上移就上移，不能上移才用 #111 的噪声锚）；build-tooling 新增**发现 #51**（多项目 gradle 下**裸任务名级联**——1.20.1 根项目 `runServer` 连带 `:content-test:runServer`（第二次、共用同一 `run`/world），判据 = `^> Task` 任务行与对照臂不一致，修复 = `:runServer`；⚠️ 该级联一度被当作 `Processed` 恒 0 的嫌疑，最终由 E1 dump 定因而降为「真坑非根因」——**同形不同因须各自独立证据**；#8/#47 家族第六形态：任务名作用域）+ **发现 #52**（Chunky 1.3.146 任务状态**跨 run 持久化**在 `run\config\chunky\tasks\<ns>\<dim>.properties`，删 `run\world` 不清它 ⇒ 残留任务让 `chunky start` 回「A task was already started…」而**静默不开始**，伪装成同形不同因的卡死；「清环境」清单必须覆盖**工具自己的状态目录**，#18 残留态家族第三形态）+ **发现 #53**（region 对拍工具在**无 `xPos` 键载体**（1.20.1）MUST 按「region 文件名 + 槽位索引」推导坐标（`cx = rx*32 + (i&31)`），且手写 NBT reader MUST 与**已验证版本逐行对齐**——首版把 TAG_Byte 读 8 字节 / TAG_Byte_Array 读 i4 长度 ⇒ **静默失步少解析**（7703→760，**不报错**），判据 = 三步自检：自比 0 差且 common 非空 + 已知不同两臂正对照非零 + 解析数 ≈ 生成器自报数；#20 第二形态）；compiler-idioms 新增**发现 #24**（1.20.1 `PalettedContainer.swap()` 自带 `lock()/unlock()`、`LockHelper` **非可重入** ⇒ `ChunkSection.lock()` 只与 `setBlockState(..., lock=false)`/`swapUnsafe` 配套，「外层锁 + lock=false 写」成对，单独任一半即坏）。来源：`.artifacts/perf-reg-260910-06/verdict-260910-06.md`（candidate，judge 未做 / 用户未 confirmed）+ `.investigations/perf-reg-260910-06/{errors-260910-06.md,cmd-output/{results.txt,logs/,diffs/},java-snapshot/post-260910-06/}` + `.investigations/000-架构设计/架构计划-260910-06.md` §14；时间线 → `versions/1.20.1/docs/10-timewise-archive.md` 260910-06 块（新建块，追加于 260907-01 之后）。

---

## E. `versions/1.20.1/docs/10-timewise-archive.md` 拟追加的 260910-06 时间线块（追加到文件末尾 `:3087` 之后）

> 格式对齐现网：`## YYMMDD-NN（实际 …锚定：…）状态` → 一段 `> 过程产物/通用模式` → `- ✅/❌/🔍` 过程条目；**只记过程与排除项，不搬结论数字堆**（数字在 verdict/主题篇）。

## 260910-06（实际 2026-09-10 20:16 起 Get-Date 锚定：1.20.1 R3 异步化移植（重活移出 worldgen 单车道）+ 同构建态 A/B + 内容指纹门 + 出测试 jar）🔍 candidate（judge 未做 / 用户未 confirmed）

> 过程产物 `.investigations/perf-reg-260910-06/`（`errors-260910-06.md` 五段式台账 / `cmd-output/` 10 臂日志 + `results.txt` + `diffs/` 7 对 + `region-*` 快照 + `run_arms_1201.ps1` + `diff_1201.py` / `java-snapshot/post-260910-06/` 5 文件 + `SHA256SUMS.txt`）；架构计划 `.investigations/000-架构设计/架构计划-260910-06.md`（含 **§14 执行中变更** D-b 实测证伪）；判决 `.artifacts/perf-reg-260910-06/verdict-260910-06.md`（**candidate**）；通用模式 → workflow-patterns #113/#114/#115、build-tooling #51/#52/#53、compiler-idioms #24（subagent 草稿 → 主会话应用）。

- ✅ **范围与形态移植**：三分支（overworld/nether/end）`populateNoise` 接管段由「传入车道上同步执行」改为 `wgDispatch`——默认 `supplyAsync(work, WG_FILL_POOL)`（= `Util.getMainWorkerExecutor()`，1.20.1 无 `NameableExecutor` 故不加 `.named()`），`-Psyncfill=1` 回退为内联 `completedFuture(work.get())`；Rust/C++ 零改动（执行体与 260910-05 同源 dll，逐臂 `[CppBridge] dll=` 核对）。
- ❌ **首版镜像 vanilla 的「sections 外层锁 + `whenCompleteAsync` 解锁」被实测证伪**：两臂都卡死（`Processed: 0`、CPU 累计 11-13s/wall 204s、等 40min 不复原）——同线程**非可重入 `LockHelper` 自锁死**（errors **E1**）；**排除对照 = 同步臂也卡** ⇒ 「异步分派 / 完成跳回」变量被排除，嫌疑收敛到两臂共有的新增项（外层锁）；修复 = 删 `wgLockSections`/`wgUnlock` 与完成跳回，退化为 1.21.6 R3 已验证的无外层锁形态（锁语义交写回路径自带逐次 `swap()`）。
- ✅ **诊断件（门控默认关，生产零成本）**：`StallWatch.java`（`-Dcoreswap.stallwatch=<秒>`，daemon 自打全线程栈，`[STALLWATCH]` 前缀）——沙箱 `jstack`/`jcmd` 全拒访（attach 被拦）下**唯一**的停滞观测面，本块靠它一次 dump 读出阻塞链；`ChunkTiming.java`（1.20.1 精简版，只打非恒 0 分项）；`CppBridge` 新增 `wgBufHash` + `[WG-CONTENT]` 逐 chunk 指纹（MIXLOG 门控）＝本块行为门仪器。
- ✅ **驱动三修正**（落 `cmd-output/run_arms_1201.ps1`）：① `gradle runServer` → **`gradle :runServer`**（裸任务名级联 `:content-test:runServer`，**E2**）；② 每臂清 `run\config\chunky\tasks`（Chunky 任务状态跨 run 持久化，**E3**）；③ 新增**早停探针**（`chunky progress` 轮询，`Processed` 恒 0 超 180s ⇒ 落盘诊断后退出，替代 40min 盲等）。
- ✅ **形态直证 + 量级（同构建态单变量 A/B，同 dll sha / 同 seed / 同 region 背靠背）**：`inflight max` **1 → 23**（sync 恒 1 / async 达池并发）；异步与同步都取双 run（sync ×2 / async ×2 + mixlog 臂 + vanilla 对照 + nether/end sanity，共 10 臂）；vanilla 对照臂同批在位 ⇒ 得「同步形态慢于 vanilla、异步形态快于 vanilla」的**同批**读数（绝对值一律以判决表为准）。
- ✅ **行为门三层**：**第一层（决定性、噪声无关）** = 每 chunk 原生输出 FNV-1a 64 指纹跨形态逐 chunk 比对，全等零差、臂内无重复指纹；**第二层（旁证，检验力低已声明）** = region 逐块对拍 7 对（自比 0 差 / 正对照非零 / 同形态锚 vs 跨形态读数），**读法与判定预登记**；**第三层** = 工具自检（自比 0 差 + common 非空 + 正对照灵敏度）。
- ❌ **对拍工具首版手写 NBT reader 静默失步**：TAG_Byte 读 8 字节 / TAG_Byte_Array 读 i4 长度 ⇒ 全域只解析出极少数 chunk **且不报错**（**E4**，判决 §4.3 引；**台账正文待补**，见下 🔍）；修复 = 与 260910-05 已验证版 reader 逐行对齐。
- ✅ **nether/end 只做「接管生效 sanity」**（本块不设全维行为门，递延）：`intercepted` 与 `[WG-FILL]` 行数同阶（各 4761/4761）、`inflight max=23`、无异常；**判定域限定**写进判决 §7.1。
- ✅ **交付（Phase 4）**：构建 1.20.1 jar（版本自 1.0.27 递增）+ **执行体三元组核验 MATCH**（jar sha / jar 内 `native/worldgen.dll` sha / `target/release/worldgen.dll` sha 三者一致，且与全部 10 臂测量同源）；实机测试点交用户（**观察级**，含 `-Dcoreswap.chunktime=1` 与 `-Dcoreswap.syncfill=1` 回退开关）。
- 🔍 **open（未核/降级/边界）**：① nether/end 全维行为门未做（指纹门只覆盖 overworld）；② region 门检验力低（每形态 1-2 对、无置信区间，跨形态高于同形态锚的分量**未归因**）；③ **1.20.1 载体 run 级非确定量级本身未立项**（0.0209-0.0467%，约为 1.21.6 overworld 的 2-5×，只登记未查成因）；④ **pre 源码快照缺失**（`runtime/` 被 gitignore，pre 状态以 1.0.27 jar 的 class 级证据承载）；⑤ **未做锁相关并发压力测试**（写回路径逐次 `swap()` 锁语义沿用改造前行为）；⑥ 驱动/对拍工具为一次性件（`.tmp` 不入库，关键机制与判据已自足记录）；⑦ **流程**：judge 未做（candidate 待 SHOULD judge）、confirmed 待用户；⑧ **E4 台账正文缺失** + 速查表第 4 行（沙箱 attach 拒访）与判决 §8 的 E4 定义不一致（见 §F.1）。

---

## F. 自检清单结论（价值门逐条 + 不写项 + 矛盾/缺口）

### F.1 价值门逐条判定（任务书 7 条）

| # | 内容 | 价值门 | 载体（决定性理由） | 判定 |
|---|---|---|---|---|
| 1 | 跨实现搬运锁语义必须成对核对（自锁死） | **高价值（必记）**——错误链 + 判错签名 + 反模式，正是「错误优先」资产 | workflow-patterns **#113**（方法面判据）+ compiler-idioms #24（机制事实） | 写（详写，五段式齐全） |
| 2 | 禁 attach 时门控看门狗自打线程栈 | **高价值（必记）**——环境坑 + 唯一可用手段 + 门控纪律 | workflow-patterns **#114 简记**（手法），与 build-tooling #46 配对互引 | 写（简记） |
| 3 | 裸任务名级联子工程 | **高价值（必记）**——真实构建接线坑 + 判据 + 便宜自证 + 归因链留档 | build-tooling **#51**（项目载体就在此） | 写 |
| 4 | Chunky 任务状态跨 run 持久化 | **高价值（必记）**——环境坑 + 「清环境清单」可复用判据 + 同形不同因 | build-tooling **#52**（Chunky 载体系列 #26 家族） | 写 |
| 5 | 无 `xPos` 载体坐标推导 + reader 逐行对齐 | **高价值（必记）**——工具 bug 静默毁结论 + 三步自检判据 | build-tooling **#53**（工具链载体；#20 第二形态） | 写 |
| 6 | 1.20.1 `swap()` 自带锁 / `LockHelper` 非可重入 | **中价值（简记）**——API 惯用法（「是什么」），方法面已由 #113 承载 | compiler-idioms **#24**（#12/#17/#23 同型：平台语义约束） | 写（简记，**不与 #113 重复**：只记机制事实，判据留 #113） |
| 7 | 噪声无关的内容指纹门 | **高价值（可复用判据）**——把「落噪声带内」升级为二值强判据，可直接复用 | workflow-patterns **#115 简记**（与 #111 分工互引） | 写（简记） |

### F.2 我认为**不该写**的项（及理由）

1. **一次性数字与状态快照**：10 臂的秒数/比值（222/37、6.00×、1.32-1.35×）、`inflight max=23`、dll/jar sha、region 差异百分比（0.0351%/0.0648% 等）、`serverCpu`/核数、`nz` 区间——**低价值（不记）**：判决表与时间线已承载，写进 discovered 只稀释高价值权重（价值门低价值层：一次性结论、当前对齐状态快照）。
2. **`-P`→`-D` 映射行补齐（`-Pmixlog`/`-Pchunktime`/`-Psyncfill`/`-Pstallwatch`）**——架构计划 §10 原拟写一条，**本稿判为不写**：build-tooling #8/#9/#19/#25/#47 已把该家族计到「第四/第五形态 + 作用域维」，本块只是既有判据的常规应用（且属 `runtime/` 无 VCS 的一次性接线），重复记录无新判据；若主会话坚持登记，建议只作 **#8 补充案例一行**而非新编号。
3. **Chunky 1.3.146 vs 1.4.40 命令面差异**——架构计划 §10 候选，**不写**：本块 F8 只核到「1.20.1 dev 环境有 1.3.146」，**未实测命令面差异**（无 `Incorrect argument` 类一手回显），属未验证推断；且 #26 家族补充案例（260910-05）已记 Chunky 维度/region 面。**待有实测差异再写**。
4. **`ChunkTiming.java` 1.20.1 版移植细节**（去掉 1.21.6 的恒 0 分项）——**不写**：工程实现细节，自推可得；「不要打恒 0 假分项」一句话若要有价值，宜作 #114/#115 的备注而不单列条目（本稿未单列）。
5. **「首轮 40min 盲等」教训单独成条**——**不写**：已并入 #113 判据 3（早停探针），单列会与 #113 重复。
6. **`runtime/` 无 VCS ⇒ pre 快照以 .27 jar class 级证据承载**——**本块不写**（判定中价值偏下）：与「核验证据落盘」（#31）「产物在盘 ≠ 本次生成」（build-tooling #18）家族相邻但形态不同；本块由 verdict §2 自足承载。**若再遇同情形（无 VCS 目录的 pre/post 边界）建议立项**，本稿不占号。
7. **nether/end「接管生效 sanity 通过」**——**不写**：一次性状态，非判据（其判据面已被 #107 家族补充案例 260910-05 记录）。
8. **实机 vivo 未测、jar 交付细节**——**不写**：交付记录进 10 时间线/判决即可，无跨块复用价值。

### F.3 建议主会话优先处置的台账缺口（**与原料矛盾之处**）

1. **`errors-260910-06.md` 缺 E4 正文，且速查表第 4 行与判决 §8 的 E4 定义不一致**（**最高优先**）：判决 §4.3/§8 把 **E4 = 对拍工具手写 NBT reader 静默少解析**（7703→760），但台账正文只有 E1-E3，**速查表第 4 行写的是「沙箱 `jstack`/`jcmd` exit 1 → JVM attach 拒访 → 改用 StallWatch」**（该内容实为 E1 的**定位方法**，非独立错误）。⇒ 建议：补 E4 五段式正文（本稿 #53 已备齐内容，可直接搬）+ 把速查表第 4 行改为 E4 的正确指向（attach 拒访若要保留，宜并入 E1 定位方法或显式单列为 E5 —— 本稿 #114 已按「环境事实 + 方法」承载）。
2. **NBT reader 失步的**分母口径不一**：判决 §4.3 写「**7703** chunk 只解析出 760 个」，工具注释 `diff_1201.py:37` 写「全域 **7542** chunk 只解析出 760 个」（760 一致）。⇒ 引用时须声明口径；本稿 #53 已把两个分母并列并注明「以最终自比 `common=7703`（`diff-self_os.txt:1`）更稳」。
3. **errors E1 的家族引用号对不上现网**：E1 写「与『签名迁移』(**#25/#55**) 同族」——现网 workflow-patterns #25 = 静态调研结论失真、#55 = mixin 反射字符串不重映射；build-tooling 无 #55。⇒ 建议改写为「符号/命名迁移面」（workflow #55）或直接引 build-tooling #8/#47 家族。
4. **errors E3 的家族引用号对不上现网**：E3 写「#18/#49 家族」——#18（旧 world 缓存）对得上，**#49（build-tooling）是 PowerShell 逗号串坑**，语义不符；若指 workflow-patterns #49（完成度伪差）则成立但需标明文件域。⇒ 建议标明「build-tooling #18 / workflow-patterns #49」。
5. **CPU 增量表述两源不一**：errors E1 写「0.1s/12s（≈0.01 核）」为活采样窗口，`results.txt:3,6` 的 STALL 行为累计「serverCpu=11/13 ÷ wallgen 204 ⇒ 0.05-0.06 核」。⇒ 同量级，但本稿条目已**分别标注来源**（#113 现象段），请勿合并成一个数字。
6. **`10-timewise-archive.md`（1.20.1）尾部为 `260907-01`**：本块将**直接追加 260910-06**（09-08～09-10 的 1.20.1 相关块未见于该文件——若确有（如 vivo-freeze/1.20.1 相关块），需先确认归口是 1.20.1 还是 1.21.6 时间线，避免时间线断档或双写）。
7. **本稿 subagent 读码发现的**待核线索**（**未验证 / Degraded，故不写进条目正文**）：现网 `diff_1201.py` 的 `t == 7`（TAG_Byte_Array）分支读的是 **1 字节长度**（`r.n(r.u1())`，`:44`），而 NBT 规范该 tag 的载荷长度是 **TAG_Int(4B)**——若 1.20.1 chunk NBT 真出现过 TAG_Byte_Array 且其长度高字节非零，会**再次静默失步**（同 #53 主题的第三形态）。本稿无 shell 不能统计其出现次数；建议主会话做一次廉价统计：出现次数 **0** = 潜在坑（值得在 #53 加一行"已核不可达"）；**>0 且长度 >255** = 现网工具 bug（#53 判据升级 + 重跑对拍确认无影响）。
8. **判据/结论状态**：本稿全部条目均为 **candidate/简记**（无一条可 confirmed——confirmed 只能由用户授予）；判决本身 judge 未做（架构计划 §8 预置的 candidate-SHOULD / FIN-MUST 均未签），故 #113/#115 若在 judge 中被挑战，本稿对应条目须一并回改。

### F.4 数字来源与复核边界（诚实声明）

- **本稿 subagent 无 shell**：所有数字**未跑命令重算**，来源逐条标注为「判决 / errors / 一手 `results.txt` / 一手 `logs/`」。
- **本稿实际一手核对过的文件**（非转述）：`results.txt`（16 行，10 臂 + 2 条 STALL 行）、`logs/os-r1.log` 与 `logs/oa-r1.log` 的 `[CHUNKTIME]` 全行（n=256…4096，`inflight max=1` vs `23`）、`logs/osS-r2.log` 与 `logs/oaS-r2.log` 的 `[WG-CONTENT]` **抽样 5 chunk 逐字比对一致**、`diffs/diff-self_os.txt`（`common=7703 / diff=0`）、`diffs/diff-cross_os1_x_oa1.txt`（`common=7703 / blocks=348504064 / diff=225703 = 0.0648%`）、`diff_1201.py`（reader 与坐标推导段）、`StallWatch.java`（59 行）、`run_arms_1201.ps1:36-65`、`errors-260910-06.md` 全文、架构计划 §14、判决全文。
- **未复核**：4140 的精确去重计数（仅抽 5 chunk + 行数量级）、region 7 对 diff 的其余文件、jar/dll 三元组 sha（需重算/读二进制）、`[STALLWATCH]` dump 原文（仅在 errors/verdict 内引用）。
- **§9.7 可比性**：本稿所有数字均属 1.20.1 载体（MC 1.20.1 + Fabric loom `:runServer` + Chunky 1.3.146 + seed 417950215108767439）；**禁与 260910-04/05 的 1.21.6 数字互引**（判决 §5.3）。
