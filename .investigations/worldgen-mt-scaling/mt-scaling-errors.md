# 多线程/性能排查：错误与根因台账（worldgen-mt-scaling 课题，2026-08-16 草稿）

> 用户观点（错误优先原则，2026-08-13 明确）：**错误信息 + 探明「为什么错」的过程，比验证通过的结果更有价值。**
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式完整记录 2026-08-06 ~ 2026-08-16 多线程吞吐排查的全部错误，
> 作为后续排查的索引；结论性验证数据见 scout-map.md / notify-bug-impact.md，本文件只记「错在哪、为什么错」。
> 编号体系：MT1-MT7（课题内编号）。状态标签：✅已修复 / ↩️已回滚 / 🔍待定性 / ⚡待实机验证（candidate）。
> 素材来源：主会话实测（bench-C2-20260815 / bench-notifyfix-* / WG_TASKTIME / WG_MTTRACE / WG_STAGETIMER）+ git 历史（c792e9d→252d988→e388ab4→cc93c50→0a781e1→8966ba9）+ notify-bug-impact.md 影响评估。

---

## MT1. 🔥 notify 丢失 bug（核心）——线程池「补建 worker + notify」竞争 → 多线程 bench「反降」串行假象（✅已修复 0a781e1）

**状态**：✅ 已修复（0a781e1，2026-08-15 23:50）。**引入**：252d988（8/6 20:11，扩容支持，ensure 补建 worker 逻辑）；**活跃期**：8/6 20:11 → 8/15 23:50（**约 9 天**）。

### 现象
- bench-C2-20260815.txt（256 chunks 顺序跑 {1,8,12,22}，[A] 模式）：**T=1 73.23 / T=8 87.51 / T=12 89.92 / T=22 94.35 ms/chunk**——多线程「反降 +19~29%」，单调变差，与 C2ME 对照（近线性）严重不符。
- WG_TASKTIME 实证（0a781e1 加的诊断钩子）：
  - **{1,8,12,22} 顺序跑时**：T=8 补建 7 个 worker，但第一批 16 任务**全部由老 worker（tid 50952）单线程执行**，补建 worker 全空闲 → 批墙钟 ≈ 串行；
  - **T=8 单独跑**（进程内首个调用）：16 任务分散 8 线程，完美并行——**证明池本身能并行，问题只在「T=1 前置后补建」路径**。
- 8/11-8/15 期间「多线程无加速反降」被当作性能现实，反复误导排查（scout-map C1-C7 候选全排查完，全是表象）。

### 根因
`CoreSwapPool`（worldgen_api.cpp L1052-1165）的 worker 生命周期竞争：
- `ensure(n)`（L1057-1098）**在 mtx 锁内**补建 worker（`workers.emplace_back(...)` 循环）；
- 新 worker 启动后第一件事是**拿 mtx 进 `cvTask.wait`**（L1068-1074，谓词 `stop || !tasks.empty()`）；
- `run()`（L1105-1128）入队任务后 `notify_all()`（L1125）——**若 notify_all 发生在新 worker 进入 wait 之前**（worker 就绪与通知的竞争窗口），这批补建 worker 的通知全部丢失；
- 补建 worker 之后拿到 mtx 评估谓词时，队列可能已被老 worker 消费清空（tasks 空 + stop false）→ **永久等待**（本批后续每批任务都由老 worker 串行消费）→ **串行假象**。

这就是并发编程教科书的经典**丢失唤醒（lost wakeup）**：notify 必须发生在 waiter 注册之后（wait 内），否则通知打空。池在「无扩容」时正确（worker 已注册为 waiter），所以 T=8 单独跑完美并行——**bug 只在池扩容路径暴露**，这是它藏了 9 天的结构性原因。
**触发边界（重要澄清）**：本 bug 只影响 [A] 批量模式（count=N，线程数递增 → 补建 worker）；**[B]/实机 count=1 模式每次 ensure(1) 不补建 → 不触发本 bug**——其「无并行」另有根因 = MT3 的 threads clamp，两 bug 独立（详见 MT3）。

### 定位
1. WG_TASKTIME 对比「单独跑 vs 顺序跑」：单独跑 `done_by` 分散 8 线程；顺序跑 `done_by` 恒为老 worker → 锁定「补建 worker 没干活」；
2. 结合 bench 顺序跑 {1,8,12,22} 的结构（T=1 先 ensure(1) → T=8 ensure(8) 补建 7 个），推断补建 worker 错过 notify → 对照 run()/ensure() 代码确认竞争窗口（L1110-1118 修复注释即此定位的落盘）。
3. 此前 C1-C7（每 chunk 1.2MB 堆分配/睿频/SMT/LLC/调度亲和/带宽/锁）逐一 A/B 排除（scout-map.md L73-99），全部非主因——**排除链本身是资产**（为 notify 定位缩小了空间）。

### 修复
`readyCount` 原子（worker 进 wait 自增 / 拿任务自减，L1069/L1071）+ `run()` 入队前自旋等 `readyCount >= workers.size()`（L1110-1118，0a781e1）。**为什么这样能修**：把「所有 worker 已注册为 waiter」变成 run() 入队前的显式前置条件，notify_all 必然唤醒全部 worker，竞争窗口消除。

### 教训
1. **线程池「补建 worker + notify」是经典丢失唤醒**——多线程性能「反降」先查线程池实现正确性（worker 就绪与通知竞争），再查内存/调度/带宽（scout-map L107 已沉淀）。
2. **bench 顺序跑 {1,8,12,22} 必然触发**（T=1 后 T>1 补建全错过）——多线程 bench 的线程数递增必须与「池扩容」语义对齐，或每个 T 独立进程跑；「池无增长时正确」是假阴性信号。
3. **多线程性能结论必须先验证「任务确实被并行消费」**（TASKTIME/并行度证据），不能只看 wall time 就下「反降/无加速」结论——本次 9 天排查的教训源头。
4. **影响面**：8/11-8/15 所有 [A] T>1 顺序跑数据作废（串行假象），详见 MT2/MT6；单线程数据全部不受影响（T=1 无补建）。

---

## MT2. H3「thrashing ×16」结论被 notify bug 污染——需重新定性（🔍待定性）

**状态**：🔍 待定性（pending）——修复后重测未做，本台账标记待办。

### 现象
07-block-pipeline.md 记（08-11 实测）：
- spline 单次 **mt 27,155ns vs t1 1,714ns（×16）**（L97/L109）——成为 H3「多线程 thread_local thrashing 放大器 ×16」结论的依据；
- wall 多线程 **8488ms > 单线程 6533ms**（并行反而更慢）；
- 该结论曾过 judge 审查（review-rootcause.md）并经用户确认（07 篇 L67）。

### 根因
**mt 侧数据在 notify bug 活跃期（8/6-8/15）采集**——8/11 的「多线程环境」实测即顺序跑 bench 下补建 worker 空闲、**实际并行度=1**（MT1）：「多线程环境」实为单 worker + 扩池/空转开销。「×16 多线程 thrashing 放大器」并非干净环境下的证据——部分是 bug 伪影。t1 侧（1,714ns）为单线程数据不受影响，但 **×16 的「多线程侧」基数不可信**。

### 定位
本次影响评估（notify-bug-impact.md §2 #4）：逐条核对 07 篇数据采集时间与 bug 活跃期（8/6 20:11 → 8/15 23:50）的覆盖关系 → 8/11 所有 [A] T>1 顺序跑数据均触发 bug → mt 27,155ns 属此列。

### 修复
**未做（标记 pending）**：需在 notify 修复 + C1 回滚（8966ba9）后的最终状态重测 mt 侧 spline 单次成本，重新定性 H3：
- 若修复后 mt ≈ t1 → H3 是伪结论（bug 伪影），07 篇需降级/删除 ×16；
- 若仍显著 > t1 → H3 保留但必须用新数据支撑（且要排除 WG_MTTRACE 的 fprintf stderr 锁竞争污染，notify-bug-impact.md §5 #2）。
- 注意：H2 主因（FlatCache 单槽 thread_local 缓存 + buildGrid 角点越界 → 邻居 chunk key 污染 → rebuild 168×）为**单线程**精确统计（WG_SPLINEDEBUG，36,252 = 168×），**不受 notify bug 影响，保留成立**（notify-bug-impact.md §3）。

### 教训
1. **性能数据的「环境」描述要精确到「实际几 worker 在干活」**，不能只写「多线程」——并行度=1 的环境下测出的「多线程成本」没有意义。
2. **多线程数据采集必须带并行度证据**（TASKTIME/worker 活跃度/单批 done_by 分布），否则结论可能建立在伪环境上。
3. **已过 judge + 用户确认的结论也要在根因变更后回查数据来源环境**——judge 审查的是当时的证据链，环境（bug 活跃期）失效则结论需重新定性，不因「已确认」免疫。

---

## MT3. bench [B] 模拟实机结构性串行 = `threads>count` clamp（66e05f5 引入）——实机 mod「多线程」从未真正并行（🔍待定性 / ⚡代码链路已闭环，待实机实跑对比）

**状态**：🔍 待定性（bench 侧已定性）+ ⚡ 实机推论**已升级为代码链路铁证**（唯一剩余验证 = 实机实跑对比，见根因 #6）。
**根因定论（主会话 2026-08-16 核实）**：真正根因 = L1189 threads clamp（**66e05f5，8/5 18:05「方块层多线程并行」引入**，`git log -S "threads > count"` 确认），**不是 notify bug**——两个 bug 独立作用于不同模式（见根因末）。

### 现象
- bench-C2-20260815.txt [B] 模式（T worker 各调 count=1）：**workers=1 73.40 / workers=8 88.21 ms/chunk**——count=1 模型下多 worker **无并行收益（甚至更慢）**；
- bench-notifyfix-8x8-20260816.txt [B] 段同样：workers=1 86.80 ms/chunk（更大 workers 数未跑完即被 120s cap 截断）。

### 根因
**单一根因 = `wg_fill_blocks_multi` L1189 `if (threads > count) threads = count;`**，机制链（逐层核对，Java→JNI→API→池）：
1. 实机 `CppBridge.fillChunk`（CppBridge.java L170-171）每 worker 调 `fillBlocks(h, {cx}, {cz}, {buf}, THREADS)`——**count=1，threads=THREADS（物理核数自适应）**；
2. **JNI 透传（无变换）**：`jni_bridge.cpp` L79-107 `Java_wg_CppWorldgen_fillBlocks` L93 `wg_fill_blocks_multi(handle, cxs, czs, bufs.data(), (int)count, (int)threads)`——Java 侧 THREADS **原样透传**进 C++ API，无 clamp/截断/改写；
3. API 层 `wg_fill_blocks_multi`：count=1 → `threads > count` → **threads 被 clamp 到 1** → L1193 `ensure(1)` → **池恒 1 worker**；
4. 8 个 Java worker 的 8 个 count=1 任务**全部排队到 1 个 C++ worker** → run(1) 结构性串行。
5. **bench [B] 殊途同归**：每个 worker 显式传 `threads=1`（bench_chunks.cpp L141/L160）——不经过 clamp 也被 ensure(1)；实机传 THREADS 但被 clamp 到 1——**两条路，同一个结果：池都只有 1 worker**；**[B] 模拟失真只在「传参方式」（显式 1 vs 被 clamp），串行结果与实机一致**。
6. **结论升级（代码链路铁证，2026-08-16 闭环）**：完整链路 = CppBridge L170-171（count=1, THREADS）→ jni_bridge L93（透传）→ L1189 clamp（count=1 → 1）→ L1193 ensure(1) → 池恒 1 worker → run(1) 串行。**实机 mod 每 worker 调 count=1 时，即使传 THREADS=12 也被 clamp 到 1 → 实机「多线程」从未真正并行（结构性串行）**。推论从 candidate **升级为代码链路铁证**——唯一剩余验证 = 实机实跑对比（可观察到：实机多线程生成时 C++ 侧只有一个 worker 在干活，或吞吐与单线程无差）。
7. **clamp 的引入语境**：66e05f5（8/5）时是**每调用创建/销毁线程**的模型（16 chunks 并行 109.8ms ≈ 9.6×），clamp 合理（每调用最多建 count 线程）；**8/6 池化（c792e9d）后语义失效**——池是进程级常驻资源，「单批任务数」错误地决定了「池线程数」，clamp 从合理优化变成结构性串行陷阱。

**与 notify bug 的独立性（重要澄清）**：
- **notify bug 只影响 [A] 批量模式**（count=N，线程数递增 {1,8,12,22} → 补建 worker 错过通知空闲）；
- **[B]/实机每次 ensure(1) 不补建 → notify bug 不触发**；
- 两个 bug **独立作用于不同模式**：[A] 的「反降」是 notify 串行假象；[B]/实机的「无并行」是 clamp 结构性串行——**不能混为一谈**（MT1 排查期间 [B] 的持平数据曾被误读为「B 模式正常」，实际是 B 模式被 clamp 锁死在 1 worker）。

### 定位
1. `git log -S "threads > count"` → **66e05f5（8/5 18:05）引入**（早于池 c792e9d 8/6，确认「先有 clamp 后池化」的语义失效链条）；
2. **三层逐级核对（Java→JNI→API）**：CppBridge.java L170-171（count=1 + THREADS 传参）→ jni_bridge.cpp L93（`(int)count, (int)threads` 原样透传，无变换）→ `wg_fill_blocks_multi` clamp（L1189）——完整链路确认 threads 参数一路无失真到达 clamp 点，clamp 是唯一（也是最后的）收缩点；
3. bench [B]（显式 threads=1，L141）与真实 JNI 调用链（THREADS 被 clamp）逐参数对齐 → 两者共享同一 ensure(1) 路径 → clamp 是共同瓶颈。

### 修复
**待定（记录为待办，不实施）**。clamp 语义需重新设计：
- **池线程数 ≠ 本批任务数**——池是进程级常驻资源（CoreSwapPool::instance() 单例），线程数应由全局配置（物理核数/环境变量）决定，与单批 count 无关；
- 候选方向：① **count=1 时应允许池线程数 = THREADS**——clamp 改为 `if (threads > count && count > 1)`（count=1 时保留 THREADS）或**去掉对池 size 的限制、只限制每批任务数**（少任务只少建批内任务，不缩小已建池）；② **[B]/实机改批量调用**（一次传多 chunk，count=N 走 [A] 路径，同时绕开 clamp 与 ensure 频繁调用）。

### 教训
1. **「count 小就缩线程数」的 clamp 在多批短任务场景是错误优化，且是结构性阉割**——单批并行度是「任务的并行度」，池规模是「进程级资源」，混用会锁死并行（每批 count=1 → 池恒 1 worker → 实机串行）；**机制有效性随架构演化失效**：per-call 线程时代 clamp 合理，池化后必须重审（66e05f5 引入 → c792e9d 池化 → 8/5-8/16 从未被发现）。
2. **模拟实机 bench 与真实 JNI 调用链必须逐参数对齐，且逐层核对（Java→JNI→API→池）**——bench [B] 显式 threads=1 与实机 THREADS 被 clamp 殊途同归，但只有对齐完整调用链才能发现共同瓶颈是 clamp 而非 bench 参数；**JNI 透传层（jni_bridge L93）是最易漏查的盲区**（看似无变换就不核，实际它是「参数是否失真」链条的一环——本链路确认透传无损，收缩点只在 clamp）。
3. **多个性能 bug 要按模式归因**——notify（[A] 批量）与 clamp（[B]/实机 M=1）独立作用：**[B] 的持平数据不是「B 模式正常」的证据，而是 B 模式被锁死在 1 worker**；修复一个 bug 后另一个模式的数据不能当作「已验证」。
4. 性能问题排查要回追「**实机是否真的并行过**」——clamp 在源头让实机从未并行，而 mod 侧看不到任何报错（正确性无碍，只是慢）。

---

## MT4. 计时污染——WG_PROFILE/WG_STAGETIMER 探针伪影（✅已修复 cc93c50 + 8/15 揭穿）

**状态**：✅ 已修复（探针分离 cc93c50，2026-08-11 18:17；伪影彻底揭穿 2026-08-15 晚段）。

### 现象
- WG_PROFILE 下 **density 显示 460ms（真实 45ms，慢 10 倍）**；**spline 单次 92μs 伪影**（真实 spline 正常，基线 ~1.7μs 曾判「54 倍退化」）；
- 8/11 `wgprofile_8576_mt.txt` 显示 density=746ms 等（伪影）；
- 后果：8/11 曾据此判定「spline 退化是独立于多线程反降的第二问题」（scout-map 引言），误导单线程基线复核方向。

### 根因
**探针自身开销计入阶段耗时**：WG_PROFILE/WG_STAGETIMER 对**每个采样点**执行 `steady_clock` 计时 + 原子计数（98,304 点/chunk 的 density 阶段被放大最多）；探针计数语义与阶段耗时耦合（spline 计数器进 density 阶段 = 双重污染）。典型量级就是「×10」——每采样点一个 steady_clock + 原子 RMW。

### 定位
8/15 晚段对比：**WG_STAGETIMER 开 → density 458-471ms；关 → 45ms**（scout-map L112）→ 同一代码同一数据探针开关差 10 倍 → 揭穿计时污染；同时 8/11 的「spline 92μs」用 WG_SPLINEDEBUG 单线程复核（正常）交叉印证。

### 修复
- cc93c50（8/11）：分离 WG_STAGETIMER（阶段计时，不带 spline 计数器污染）；
- 8/15 起：探针默认关闭，需要时以「探针开销标定（开/关对比）」为前提读数据；WG_MTTRACE 的 fprintf stderr 也被确认有同步锁竞争污染（notify-bug-impact.md §5 #2，待无 fprintf 计数器复测）。

### 教训
1. **性能探针自身会污染测量——先标定探针开销（探针开/关对比）再读数据**；「慢 10 倍」先查探针再查代码（探针伪影的典型量级就是 ×10）。
2. **计时与计数必须分离**——探针里「每采样点计时」与「每采样点计数」耦合时，计数语义会串入阶段耗时（spline 计数器进 density 阶段）。
3. 历史教训：8/11 的 92μs/460ms/746ms 全被判为真退化，直到 8/15 探针开关对比才揭穿——**带探针的历史性能数据在引用前必须确认探针口径**。

---

## MT5. C1 thread_local 复用改造引入退化——「每 chunk 1.2MB 堆分配」被排除但改造本身变慢（↩️已回滚 8966ba9）

**状态**：↩️ 已回滚（8966ba9，2026-08-15 23:59）。

### 现象
C1 候选验证（tl_col/tl_densityBuf thread_local 复用，消除每 chunk 1.2MB 堆分配/释放）：
- **单线程慢 9%**：71.68 → 77.93 ms/chunk；
- 多线程反降依旧：87.30 → 90.23 / 97.23 → 95.90 ms/chunk（T=8/22）；
- 正确性：8576 区域 99.9994% 零退化（col 复用无残留问题）。

### 根因
两层：
1. **C1 机制本身不是 MT 反降主因**（验证结论：每 chunk 1.2MB 堆分配/释放不是多线程反降主因——改造后反降依旧）；
2. **改造本身引入单线程退化**：thread_local 大缓冲复用 + 跨 chunk 缓存亲和性劣化（推测，**未完全定性即实施**）——大缓冲常驻 TLS 改变缓存行为（跨 chunk 数据驻留、TLS 段访问），单线程也可能变慢。

### 定位
C1 验证 A/B（scout-map L73-77）：改造前后同口径对比（T=1/8/22 各档）→ 单线程档位即变慢 9% → 定性为「改造有副作用且目标机制被排除」→ 回滚。

### 修复
回滚（8966ba9）：恢复每 chunk 栈/堆分配（`BlockColumn col(...)` + `std::vector<double> densityBuf(...)`），线程数维度回归原位。**C1 排除结论保留**（负面验证结果本身是资产）。

### 教训
1. **优化必须 A/B 验证（改造前后同口径对比）再定论，不能凭机制推测直接实施**——C1 的机制推演（堆锁+TLB）合理但实测无效，且改造还引入新退化。
2. **thread_local 大缓冲复用不是自动赢**——缓存亲和性是双向的：每 chunk 重新分配有分配成本，常驻复用有缓存驻留/亲和劣化成本，必须实测决定。
3. **负面验证结果要沉淀**：「每 chunk 1.2MB 堆分配不是 MT 反降主因」为后续 notify 定位（MT1）缩小了排查空间。

---

## MT6. 修复后验证缺失——bench-notifyfix 只 3 行断跑，修复效果一度无数据（✅已补充：本 session 64-chunk 前台重测）

**状态**：✅ 已补充（本 session 重跑 64-chunk 前台验证，bench-notifyfix-8x8-20260816.txt）。

### 现象
0a781e1（8/15 23:50）notify 修复后，256-chunk bench（`bench-notifyfix-20260815.txt`）**输出 3 行 [BENCH] 头就断**（dll 行/crash handler 行/BENCH header 行后无任何 [A] 数据）——修复后大规模验证数据缺失；
- 且 scout-map L110 记「修复后仍反降（T=1 71.40 / T=8 84.24）」——该中间状态结论与最终状态（C1 回滚 + notify 修复）**矛盾**（单线程基差 +37%），待同机同状态对照核实（notify-bug-impact.md §5 #1）。

### 根因
**验证规模与超时 cap 不匹配**：256 chunks（16×16）× 5 组线程（{1,8,12,22,0}）× warmup+reps 的总耗时 > 120s cap → 被杀/超时断跑；用户标准是 30s-1min 小规模前台。

### 定位
读 bench-notifyfix-20260815.txt 全文件（3 行）→ 确认断跑点（header 后无数据）→ 推断超时；对照用户标准（30s-1min 前台）重排规模。

### 修复
本 session 重跑 **64-chunk（8×8）前台**（`bench-notifyfix-8x8-20260816.txt`，reps=2，seed 8576）：
```
[A] threads=  1   98.02 ms/chunk
[A] threads=  8   89.88 ms/chunk   （-8.3%：不再反降，轻度加速）
[A] threads= 12   90.39 ms/chunk
[A] threads= 22   97.76 ms/chunk
[A] threads=  0   96.30 ms/chunk
[B] workers=  1   86.80 ms/chunk   （[B] 段 120s cap 截断，未跑完——不影响 [A] 结论）
```
- **结论**：notify 修复生效——[A] T=8 不再反降（比 T=1 快 8%），但远未到 8× 加速 →「每 chunk 并发下慢」仍存在（第二阶段课题：fillOneChunkCore 并发下每 chunk 耗时随并发增长，WG_MTTRACE 证明 8 worker 真并行但批间 525ms ≈ 8×65ms，fprintf 污染待排除，见 notify-bug-impact.md §5 #2）。

### 教训
1. **修复后必须立即跑小规模验证闭环（30s-1min 前台）**，不能只依赖大规模慢跑——大规模会超时断跑产生验证空窗，修复效果一度无数据支撑。
2. **验证规模要与超时 cap 匹配**：256 chunks × 5 线程组 × reps 超 120s；64 chunks × reps=2 ≈ 1min 内可完成。
3. **中间状态（C1 版 / 计时污染）测出的「修复后仍反降」不能当最终结论**——需在最终状态（C1 回滚 + notify 修复）重测；scout-map L110 与 8x8 数据矛盾已列入待核清单。

---

## MT7. e388ab4 runMtx（用户记忆的「排队」）——架构决策「加了又去掉」未留痕（✅已核对留痕：本台账）

**状态**：✅ 已核对留痕（本 session 完成演进链核对，完整链记于本条目 + 待补 10 时间线）。

### 现象
- 用户记得「**JNI 层搞过排队**」（实机 fillBlocks 被串行化/排队）；
- 但现代码 `CoreSwapPool::run()` **无 runMtx**（per-run RunState 隔离，批间真并行）——**用户记忆与代码现状对不上**，反复被当作「当前还在排队」的假设参与讨论。

### 根因
**「加了又去掉」只留了一半痕**：
- e388ab4（8/7 16:03）加 `static std::mutex runMtx`（32 视距崩溃修复：MC 多 Worker 并发 run 覆盖共享 fn → 读空 std::function → 读地址 0 崩溃），演进记录散落在 10-timewise-archive.md（L567 加 runMtx / L1112 runMtx 实证 / L1118 删 runMtx）；
- 6e2c7ea（8/11 15:43，FlatCache 修复提交）**用 per-run RunState 隔离取代 runMtx**（批间真并行）——**只更新了代码注释（worldgen_api.cpp L1101-1104），未在 09 篇/10 时间线显著标注旧方案作废**（10 时间线 L1118 虽有记录但夹在长文档中，用户检索不到）；
- 结果：用户记忆「还在排队」（源于 8/7-8/11 期间的「排队」叙述），代码早已是 per-run 并行 → **演进链未集中留痕 = 用户记忆与代码现状脱节**。

> 主会话核实（2026-08-16）：09-multi-dimension.md **无**「排队/runMtx」字样（grep 确认）；「排队」叙述实际在 10 时间线 L567（e388ab4 加）+ L1112（实证）+ L1118（6e2c7ea 删）。subagent 初稿「09 篇记录」说法有误，此处已修正——**留痕缺口 = 演进链分散在长文档 + 无「作废标注」，而非完全未记**。

### 定位
本 session 核对流程（可复用）：
1. `git log -S "runMtx"` → e388ab4（加）/ 6e2c7ea（删）；
2. `git show 6e2c7ea` 池 diff → per-run RunState 结构（原子 done + 独立 cv + 共享任务队列）确认 runMtx 已移除；
3. grep 09 篇无「排队」+ 10 时间线 L567/L1112/L1118 找到演进记录 → 偏差来源 = 演进链分散 + 无作废标注（非未记录）。

### 修复
本台账（MT7）记录完整演进链；后续动作：09 篇「排队」段标注为历史（✅ 已过时）+ 10 时间线补 6e2c7ea 的池重构条目。

**完整演进链（核对结论）**：
| 提交 | 时间 | 池机制 |
|---|---|---|
| c792e9d | 8/6 19:57 | 持久线程池（首次按物理核数创建，后续复用；run 用共享成员 fn/totalTasks/doneCount/...） |
| 252d988 | 8/6 20:11 | 扩容支持（不锁死首次 count）+ shutdown 退出机制（wg_destroy） |
| e388ab4 | 8/7 16:03 | **runMtx 全局互斥**（32 视距崩溃补丁——并发 run 覆盖共享 fn；= 用户记忆的「排队」） |
| 6e2c7ea | 8/11 15:43 | **per-run RunState 隔离取代 runMtx**（批间真并行；注释 L1101-1104 注明） |

### 教训
1. **架构决策「加了又去掉」必须两端留痕**：加的时候记（e388ab4 做到了），**去掉的时候必须标注旧记录作废 + 时间线补一条**（6e2c7ea 没做到）——只留一半痕 = 用户/后人按旧文档记忆。
2. **commit message 简短不能替代时间线归档**——`fix: CoreSwapPool 并发 run 竞争（32 视距崩溃）` 和 FlatCache 提交里顺带的池重构，都需要在 docs/10 时间线单独成条。
3. **「用户记忆的机制」与代码现状核对流程**：`git log -S <符号>` 找变更提交 → 对照当前代码注释 → 旧文档标历史——先核对再下结论，不猜。

---

## MT8. 🔥🔥 WG_DENSITYTICK bug 误导 + density 11× 真实定位——「每 chunk 并发下慢 9×」确认（2026-08-16 反转修正）

**状态**：↩️ **初稿「并发正常」结论被推翻（2026-08-16 反转）**——density 11× / 每 chunk 并发下慢 9× **真实存在**（WG_PHASETICK 干净确认）。本条目记录完整反转链。

### 现象（初稿被推翻）
- 初稿基于 WG_DENSITYTICK 测「density 6.95ms 不变」→ 错误结论「并发正常（T=8 +8%）」。
- **WG_PHASETICK（QPC 单次 + 无 profiling 污染 + 单循环）重测**：
  | 阶段 | T=1 | T=8 |
  |---|---|---|
  | density | 34-42ms | **400-412ms（11×）** |
  | aquifer+ore | 8ms | 25-28ms |
  | surface | 7ms | 25-38ms |
  | total | 50ms | **462ms** |
- **自洽验证**：462ms/chunk × 8 并行（64 chunks = 8 批）≈ 3696 + 批间 = 4618ms = wall。**每 chunk 真实 462ms（T=8）vs 50ms（T=1）= 并发下慢 9× 真实。**

### 根因（初稿错误的机制 + 真正污染）
1. **初稿错误来自 WG_DENSITYTICK 的 bug**：重复循环（density 循环算两次）+ QPC 计时环绕位置错 → 测出 6.95ms 假象 → 我据此错误推翻 density 11×、得出「并发正常」。
2. **真正污染（非初稿所说）**：WG_DENSITYTICK 的测量崩溃（重复循环），**不是「探针普遍污染」**。WG_PHASETICK（干净）证明 density 11× 是真的——**WG_PROFILE/ WG_STAGETIMER 的 density 34→400ms 是对的**（不是探针污染）。
3. **概念混淆（关键）**：bench 的 `med/N`（wall/N）= 吞吐均值（72ms/chunk）；**但每 chunk 真实耗时 = 462ms**（8 worker 并行，wall 4618ms 处理 64 chunks，吞吐 14/s）。**我把吞吐均值误当「每 chunk 耗时」，导致误判「只慢 8%」**。吞吐（mean 72ms）和每 chunk 耗时（462ms）是两回事——wall/N 是平均吞吐，不是每 chunk 延迟。

### 定位（诊断方法）
- **WG_PHASETICK（QPC 单次、无 profiling、单循环）是可靠阶段计时**——它和 WG_PROFILE/WG_STAGETIMER 的 density 34→409ms 一致 → density 11× 真实。
- **自洽检查**：每 chunk 462ms × 8 并行 = wall 4618ms（8 批）→ 数据闭环正确。
- **WG_DENSITYTICK 反例**：它有重复循环 bug → 6.95ms 假象（**不是 QPC 污染，是代码 bug**）。

### 修复
- 回退 WG_DENSITYTICK（重复循环 bug，不可信）；WG_PHASETICK 保留（干净）。
- 修正结论：「并发正常」**错**——density 11× / 每 chunk 慢 9× **真实**。
- **进一步定位（已达成，见 MT10）**：density 11× 根源 = **SplineDF 树遍历虚调用**（6 实例、17KB、单次 15.8→190μs、195 locationFunction）——非 squeeze/InterpolatedDF/共享表（这些被排除）。优化方向 = C2ME 式 DFC 编译直排。

### 教训（本项目最重要，第 6 个探针/测量污染案例）
1. **区分「吞吐均值（wall/N）」与「每 chunk 真实耗时」**：wall/64=72ms（吞吐）≠ 每 chunk 462ms（延迟）。多线程下吞吐均值掩盖单 chunk 延迟。
2. **测量工具 bug 会给出「看似合理但错误」的数据**：WG_DENSITYTICK 的 6.95ms 看似干净（QPC 单次），实则重复循环 bug → 误导整个结论。**测量工具本身必须验证正确性（数据自洽：阶段 462ms vs wall 4618ms）**。
3. **「每 chunk 耗时 × 并行度 ≈ wall」自洽检查**：若阶段耗时 × 并行批次 ≠ wall，测量有 bug。本案例 462×8≈3696+批间=4618 自洽（对）；WG_DENSITYTICK 6.95×8≈55 ≪ 4618（明显不自洽 → 测量 bug）。
4. **不要轻易用「探针污染」解释数据**——先验证测量工具自身正确性（自洽性），再怀疑真实计算慢。本项目「所有探针都污染」的初稿结论是**过度泛化**。
5. **判断链多错叠加**：WG_DENSITYTICK 工具 bug（MT9）→ 误判「并发正常」→ 527cade 错误 commit → 又被 WG_PHASETICK 纠正。**工具错误 → 结论错误 → 提交错误**的级联——每一步都可能出错，需回到最底层工具自洽性验证。

---

## MT9. 🔥 WG_DENSITYTICK 重复循环 bug——我写代码引入的测量 bug，误导「并发正常」（↩️已回退）

**状态**：↩️ 已回退（git checkout）。**这是「工具 bug」层错误，独立于 MT8 的「判断错误」层**——先有工具 bug，才导致判断错误。

### 现象
- 我加 WG_DENSITYTICK（QPC 单次 density 计时）时，edit 引入**重复循环 bug**——density 循环（L779-796）被算了两遍（原循环 + WG_DENSITYTICK 带计时的循环）。
- 测得 density 6.95ms（T=1）——**看似合理**（QPC 单次 + 无 profled 采样计时），实则是**循环 bug 下的错误值**。
- 据此我错误得出「density 并发下不变 → **并发正常**」（MT8 初稿 + 527cade commit「排除并发慢」——**均错误**）。

### 根因（机制层面）
- **edit 时把新循环插入到原 density 循环之后，但没删原循环** → **density 循环执行两次**（每次算完 densityBuf 又覆盖）。
- 计时环绕方式是「ph0 在循环前、phA 在循环间」——但重复循环导致 QPC 差分读到的是**两个循环的总时间被错误切分** → 6.95ms 假象。
- **关键**：这个 bug 深藏（6.95ms「看似干净」QPC 单次）——**如果不用自洽检查，无法发现**。

### 定位（诊断方法）
- **自洽检查**：阶段耗时 × 并行批次 ≈ wall。WG_DENSITYTICK 6.95ms × 8 ≈ 55ms ≪ wall 4618ms（**明显不自洽**）→ 测量 bug。
- 重跑 WG_PHASETICK（单循环、无 profled）得 density 34→409ms（T=1→T=8）→ 与 WG_PROFILE 一致 → **density 11× 真实**。

### 修复
- 回退 WG_DENSITYTICK（`git checkout -- worldgen_api.cpp`）——不可信测量工具。
- **保留** WG_PHASETICK（单循环 + QPC 单次 + 无 profled，自洽验证过）。

### 教训
1. **写测量/诊断代码时，edit 会引入重复循环**（原结构 + 插入结构并存）——**必须 review 确保只执行一次**。
2. **测量工具正确性 = 数据自洽**（阶段耗时 × 并行批次 ≈ wall）——不自洽即工具 bug，不是被测对象慢。
3. **「看似干净（QPC 单次）」≠「正确」**——QPC 单次规避了探针污染，但**代码 bug（重复循环）照样给出错误值**。

---

## MT10. typeid 遍历漏 BlendDensityDF/WrappingDF——误判「spline 不存在」（🔍定位工具缺陷）

**状态**：✅ 已纠正（补全遍历后确认 6 个 SplineDF）。**这是「定位工具不完整」导致的独立误判**。

### 现象
- 用 typeid 递归遍历 finalDensity 树（WG_DENSITYSTATS）打印 53 个节点，**无 `wg::SplineDF`** → 误判「spline 不在 finalDensity 树」「density 11× 与 spline 无关」。
- **但 WG_PROFILE spline.sample=34K 次（非零）+ WG_SPLINEDEBUG 刷屏超时**（spline 被大量调用）——与「无 spline」矛盾。
- 补全遍历（WG_SPLINESTATS，加 BlendDensityDF/WrappingDF 分支）后：**splineInst=6、537 节点、17KB、195 locationFunction**——spline 真实存在！

### 根因（机制层面）
- **typeid 遍历的 dynamic_cast 分支不完整**——漏了 `BlendDensityDF`（blend_density 的 input）+ `WrappingDF`（wrapped）。
- spline 藏在这些**包装类**里：final_density = `min(squeeze(mul(interpolated(blend_density(add(...))), ...)))`——`blend_density`/`WrappingDF` 的 arg 引用 continents/erosion/depth 分量（各含 SplineDF）。
- **遍历若不含这些包装类的 arg 递归，就漏掉它们内的 spline** → 误判「无 spline」。

### 定位（诊断方法）
- **交叉验证数据矛盾**：WG_PROFILE spline=34K（非零）× typeid 遍历无 SplineDF → 矛盾 → 遍历必然有缺陷。
- **补全遍历**：加 BlendDensityDF::input / WrappingDF::wrapped / BlendDensityDF 等包装类的递归 → splineInst=6 确认。
- Python 直接 walk JSON 也证明 final_density 子树**无 `minecraft:spline`**（C++ 构建了 SplineDF 但 JSON 无 spline type——注意：C++ SplineDF 来自**别的 type 映射**，JSON 无 `minecraft:spline` 不代表 C++ 无 SplineDF）。

### 修复
- 补全 WG_SPLINESTATS 遍历（覆盖所有 DF 包装类）→ 确认 6 SplineDF。
- 修正「无 spline」误判：spline 真实存在，是 density 11× 根源。

### 教训
1. **遍历/诊断代码必须覆盖所有可能的容器类型**——漏一个包装类（BlendDensityDF/WrappingDF）就漏掉其子树，导致「无 X」误判。
2. **用「多个独立证据交叉验证」避免单工具误判**：typeid 遍历（无 spline）× WG_PROFILE 计数（34K spline）矛盾 → 遍历工具不可信，需补全。
3. **「JSON 无 X type」≠「C++ 无 X」**——C++ 可从 JSON 的其他 type 映射构建 SplineDF（JSON 用 `minecraft:spline` 只在直接声明时；非直接时经包装构建）。

---

> 按「现象→定位→教训」浓缩为可复用判错条目（五段式已在 MT1-MT7 主体完整记录，此处只沉淀「下次怎么判」）。

> 按「现象→定位→教训」浓缩为可复用判错条目（五段式已在 MT1-MT7 主体完整记录，此处只沉淀「下次怎么判」）。

1. **「多线程性能反降/无加速」先查线程池实现正确性，再查内存/调度/带宽**：本次 C1-C7（堆分配/睿频/LLC/亲和/带宽）全排查完才发现是 notify 丢失——线程池「补建 worker + notify」竞争是经典丢失唤醒，且「池无增长时正确」是假阴性信号（bug 只在扩容路径暴露）。
2. **多线程性能结论必须以「任务确实被并行消费」为前置证据**（TASKTIME/worker 活跃度/单批 done_by 分布），不能只看 wall time——「反降」可能是串行假象，不是真实并行成本。
3. **性能探针自身会污染测量**：「慢 10 倍」先做探针开/关对比标定再查代码；计时与计数必须分离（每采样点 steady_clock+原子计数 = ×10 伪影量级）。
4. **性能数据的「环境」描述必须精确到实际并行度**（几 worker 在干活），「多线程」三个字不够——并行度=1 的环境下测出的「多线程成本」没有意义。
5. **bench 与真实调用链必须逐参数对齐**（count/threads/池生命周期）——「count 小就缩线程数」的 clamp（66e05f5 引入）在多批短任务场景是结构性阉割并行；模拟 bench 传错参数会让「实机是否并行过」从未被测到；**机制有效性随架构演化失效**（per-call 线程时代 clamp 合理，池化后成为陷阱）——架构变更后必须重审旧优化。
6. **多个性能 bug 要按模式归因**——notify 丢失（[A] 批量，线程递增触发）与 threads clamp（[B]/实机 M=1，count=1 触发）独立作用：一个模式的「持平/正常」数据不能当作另一个模式健康的证据（[B] 持平 = 被 clamp 锁死 1 worker，不是「B 模式正常」）。
6. **优化必须 A/B 验证再定论**：机制推演合理（C1 堆锁+TLB）≠ 实测有效；thread_local 大缓冲复用不是自动赢（缓存亲和是双向的）。
7. **修复后必须立即小规模验证闭环**（30s-1min 前台），大规模慢跑会超时断跑产生验证空窗；中间状态测出的「修复后仍反降」不能当最终结论（需最终状态重测）。
8. **架构决策「加了又去掉」必须两端留痕**：去掉时标注旧文档作废 + 时间线补条；用户记忆与代码现状核对用 `git log -S <符号>` 找变更提交，不猜。
9. **已过 judge + 用户确认的结论也要在根因变更后回查数据来源环境**（MT2：H3 ×16 的 mt 侧数据在 bug 活跃期采集）——环境失效则结论需重新定性，不因「已确认」免疫。

**MT8-10 补充判错经验**（多轮反转链，2026-08-16）：
10. **区分「吞吐均值（wall/N）」与「每 chunk 真实耗时」**：多线程下 `wall/N` 是平均吞吐，不是每 chunk 延迟——`wall/64=72ms`（吞吐）≠ 462ms（每 chunk 真实）。**并行性能要看「每 chunk 耗时 × 并行度 ≈ wall」是否自洽，不能只看 wall/N 均值**。
11. **用「数据自洽性」验证测量工具，而非「探针污染」猜**：`阶段耗时 × 并行批次 ≈ wall` 自洽 → 工具对；不自洽（如 6.95×8 ≈ 55 ≪ 4618）→ 工具 bug。**先验证工具，再怀疑真实计算慢**——初稿「所有探针都污染」是过度泛化。
12. **判断链多错叠加**：工具 bug（WG_DENSITYTICK 重复循环）→ 误判「并发正常」→ 错误 commit（527cade）→ 又被正确工具（WG_PHASETICK）纠正。**每一层都可能错，需回到最底层工具自洽性验证，不能层层叠加推理**。
13. **遍历/诊断代码覆盖完整性**：typeid/DSTATS 遍历漏 BlendDensityDF/WrappingDF → 误判「无 spline」。**多证据交叉验证**（typeid 遍历 vs WG_PROFILE 计数的矛盾）暴露工具缺陷。
14. **不要承诺用「探针污染」解释并发数据，先做探针开/关标定**：本项目 WG_* 探针并发下有污染，但**不能默认所有探针都污染**——WG_PHASETICK（QPC 单次）证明 density 11× 真实，是**真计算慢**非污染。

---

## MT11. 🔥 MVP 决定性对照未复现 11×——微基准与生产负载特征不同，不能直接外推（2026-08-23，兼推翻「放大 MVP 验证 DFC」路径）

**状态**：🔍 已定性（对照实验结论；非 confirmed，待主会话/用户拍板归档）。本条属「错误路径/错误外推」类**方法论错误**，非代码 bug。

### 现象（数据）

MVP 并发线程扫描（`.investigations/perf-rework/vulkan-proto/mvp_spline_eval.cpp`，加线程扫描 + 每样本成本，T=1/2/4/8，100000 点/线程，5 轮取 min）：

```
per-sample ns:  constRec          explicit(DFC)     virtual
    T=1            11.7              13.4             13.4
    T=8             2.1               2.2              2.4
 concurrency amplification (T=8 vs T=1): constRec=0.2x explicit=0.2x virtual=0.2x
```

- **MVP spline 采样随线程数完美扩展**（0.2× = 8 线程快 5×），**未复现任何并发退化**；
- 对比 production 单样本 T=1 15.8μs → T=8 190μs（12×，density 11× 的直接来源）——MVP **完全看不到这个放大**；
- 这是今日唯一「未复现」证据，且是**决定性**（非「没测到」，而是干净对照后确认 MVP 层面不存在该机制）。

### 根因（为什么 MVP 复现不了）

**微基准与生产负载的结构性差异**（scout 勘探 + 对照实验共同给出）：
1. **表太小**：MVP 245 元素全驻留 L1/L2 无争用；production 是 537 节点/17KB + 195 散布堆 locFn（指针追逐的基础）。
2. **locFn 轻量子类**：无真实 shared_ptr 散布堆对象指针追逐（MVP 用 const/连续数组，编译器优化掉虚调用 + 无指针追逐）。
3. **无「并发读同一批共享对象」压力**：MVP 每线程读独立/驻留数据，不构成「8 线程同读同一批 cache-line 打满 load-store/MSHR」的延迟放大器。

而 production 的 11× 根（**共享内存延迟/指针追逐**：`sampleNode` 长串行依赖链 + 虚调用 + shared_ptr 间接 + 随机读 nodes/locs/ders/subIdx，density.h:876-925）**需要**：真实 195 个散布堆 locFn + 长依赖链 + 8 线程同时读——MVP 三缺一，天然复现不了。

**关键辨析（本条目的核心错误）**：MVP「没复现 11×」**不是**「11× 不真实」的证据——production 的 11× 已由 WG_PHASETICK（干净）独立确证。MVP「没复现」只说明**微基准与生产负载特征不同**，不能把「微基准无放大」外推为「生产无放大」或「机制不存在」。

### 定位（怎么测的 / 怎么诊断出「MVP 路径不通」）

1. **mvp_spline_eval.cpp 决定性对照**：加线程扫描（T=1/2/4/8）+ 每样本成本（100000 点/线程，5 轮取 min），三种形态（constRec / explicit(DFC) / virtual）同测——把「并发放大」从每 chunk 表象剥到「每 spline 采样」的决定性工具（scout 报告 §四 #1 变体 b）。
2. **scout 静态勘探**（concurrent-density-probe-scout.md，只读）：Tier-1 = 共享内存延迟放大（指针追逐+长链）、Tier-2 = I-cache 争用；明确排除 a-1/b/d/e/c-3（run yield 每批一次 / T=8≤12 物理核无 SMT / 密度循环无锁 / beard null 门控 / 17KB 表广播友好）。**静态本身是资产，但只到「收窄方向」，定论靠对照**。
3. **conc_density_probe + WG_PHASETICK**：确证 production 的 density 11×（8.4×/9.2×）真实，排除探针污染（「测量/探针污染铁律」）。
4. **权威 JSON**（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/*.json`）：证伪「locationFunction 嵌套 SplineDF」，所有 spline coordinate 为纯噪声，嵌套仅存于 `points[].value` 数据表。

### 修复（暂无代码修复；指出 DFC 非良药 + 路径闭合）

- **暂无代码修复**。
- **DFC 非良药**：DFC（显式栈）只消除虚调用/递归（~5% 量级），**不针对 11× 根（共享内存延迟/指针追逐）**，故不是 11× 的修复。上一节「DFC 收益天花板 ~5%」在这条 MVP 路径上**既不能证实也不能证伪**——「非良药」方向结论仍成立（机制不匹配，而非仅凭 ~5% 数字）。
- **「放大 MVP 验证 DFC」路径闭合（❌ 推翻）**：MVP 天然复现不了真实共享内存延迟，任何放大（扩表 + locFn 真递归）都复现不了 → 这条验证路径不可行。若需验证 DFC / 11× 机制，必须回到**真实 production 树**（195 散布堆 locFn + 长链 + 并发读）上做，而非微基准外推。

### 教训

1. **吞吐 vs 延迟分开**：并行性能看「每 chunk 延迟（阶段耗时）」，不是 wall/N 吞吐均值——吞吐正常 ≠ 并发无问题（本课题反复踩，AGENTS.md 已警告）。
2. **静态排除不够，要干净实验**：scout 静态收窄方向（Tier-1/Tier-2），但「locationFunction 嵌套 SplineDF」是静态 JSON/typeid 误读——定论必须用权威 JSON + 干净探针对照。
3. **微基准与生产负载特征不同，不能直接外推**：MVP（表小、无真实指针追逐、无共享对象并发压力）复现不了生产共享内存延迟——**MVP「没复现」≠「生产无放大」≠「机制不存在」**。微基准只能验证**算法正确性**（显式栈=递归逐位一致 ✅），不能作为**性能/机制**结论依据。
4. **先钉主导成本再立项**：DFC 立项基于「消除嵌套密度树递归」（误读），实际主导成本是 shift_noise 噪声计算 + 指针追逐（11× 根）——用权威 JSON 树结构提前证伪「坐标递归」，用对照实验确认 MVP 路径不可行，避免在错误方向投入大工程。
5. **「未复现」是双刃证据**：验证某假设时，「未复现」可能是（a）假设假 /（b）微基准不具备复现条件。**必须区分两者**——本条是（b）微基准结构差异，不能当（a）用。判定法：先确认微基准包含生产的关键特征（散布堆对象/长链/并发共享读），不含则「未复现」不构成对生产机制的否定。

---

## 附：错误 → 根因 速查表（一页索引）

| 错误 | 一句话根因 | 状态 |
|---|---|---|
| [A] 多线程反降 +19~29%（T=1 73.23 → T=8 87.51 → T=22 94.35 ms/chunk） | ensure 锁内建 worker + run notify_all 竞争 → 补建 worker 错过通知永久等待（tasks 空+stop false）→ 只有老 worker 干活 = 串行假象（丢失唤醒）；T=8 单独跑不触发（首次 ensure 时序天然就绪） | ✅ 已修复 0a781e1 |
| WG_TASKTIME：顺序跑 T=8 补建 worker 全空闲 / 单独跑完美并行 | 池本身能并行，bug 只在「T=1 前置后补建」的扩容路径暴露 | ✅（同 0a781e1） |
| H3「thrashing ×16」（mt 27,155ns vs t1 1,714ns / wall 8488>6533） | mt 侧数据在 notify bug 活跃期采集（实际并行度=1）→ ×16 是伪环境结论；t1/H2（rebuild 168×）单线程数据不受影响 | 🔍 待重测定性 |
| [B] 模式多 worker 无并行收益（workers=1 73.40 / =8 88.21） | **L1189 `threads>count` clamp（66e05f5 引入）结构性串行**：count=1 → threads clamp 到 1 → ensure(1) → 池恒 1 worker；bench [B] 显式 threads=1（L141）与实机 THREADS 被 clamp **殊途同归**。非 notify bug（[B] 不补建不触发）；池化（c792e9d）后 clamp 语义失效 | 🔍 待定性 |
| 实机 mod 多线程并行从未真正生效（**代码链路铁证**） | CppBridge L170-171（count=1 + THREADS）→ jni_bridge L93 透传 → L1189 clamp 到 1 → ensure(1) 池恒 1 worker → run(1) 串行——即使 THREADS=12 也被 clamp；唯一剩余验证 = 实机实跑对比 | ⚡ 代码链路已闭环，待实机实跑对比 |
| density 显示 460ms（真实 45ms）/ spline 92μs 伪影 / 8/11 wgprofile density=746ms | 探针每采样点 steady_clock + 原子计数 → 探针自身开销计入阶段耗时（×10 量级伪影） | ✅ 已修复 cc93c50 + 8/15 揭穿 |
| C1 thread_local 复用改造：单线程慢 9%（71.68→77.93） | thread_local 大缓冲复用 + 跨 chunk 缓存亲和劣化（未定性即实施）；且 C1 机制本身被排除（非 MT 主因） | ↩️ 已回滚 8966ba9 |
| 修复后验证缺失（bench-notifyfix 只 3 行断跑） | 256 chunks × 5 线程组 × warmup+reps > 120s cap 被杀；需 30s-1min 小规模前台 | ✅ 已补充 64-chunk 重测 |
| 用户记忆「JNI 层搞过排队」vs 现代码无 runMtx | e388ab4（8/7）加 runMtx 记录在 10 时间线 L567/L1112；6e2c7ea（8/11）per-run 隔离取代 runMtx 只改代码注释、演进链分散长文档无作废标注 → 「加了又去掉」留痕不全（09 篇无此内容，初稿说法已修正） | ✅ 已核对留痕 |
| scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」vs 8x8 数据（T=1 98.02 / T=8 89.88） | 中间状态（C1 版/计时污染）与最终状态（C1 回滚+notify 修复）混测；单线程基差 +37% 待同机对照 | 🔍 待核（notify-bug-impact §5 #1） |
| 并发下每 chunk 耗时 7.5 倍增长（WG_MTTRACE 批间 525ms ≈ 8×65ms） | 8 worker 真并行（enter/exit 同批）但每 chunk 并发下慢——fprintf stderr 锁竞争污染待排除；第二阶段课题 | 🔍 待无 fprintf 计数器复测 |
| **整 chunk wall T=8 比 T=1 慢 8~12%（64→69.6ms）** | ~~探针污染~~（527cade 错误结论，**已纠**）；**真相 = 吞吐均值（wall/64=72ms）≠ 每 chunk 真实耗时（462ms）**——多线程下吞吐均值掩盖单 chunk 延迟 | 🩹 已纠（fcbdad1：density 11× 真实） |
| WG_DENSITYTICK density 测量反复（5ms/370ms/33ms） | **重复循环 bug + QPC 调度污染**——阶段计时探针多线程下全部不可信 | ↩️ 已回退（MT9） |
| **density 11×（34→409ms，WG_PHASETICK 干净）** | **真实**（非探针污染）——spline 树遍历虚调用 + 并发争用；spline 6 实例、17KB、单次 15.8→190μs（12×） | ✅ 真实（MT8/MT10） |
| spline 单次 15.8μs（T=1）→ 190μs（T=8） | **共享内存延迟/指针追逐**（SplineDF 长串行依赖链 + 195 散布堆 locFn 指针追逐 + 虚调用，非虚调用本身）；DFC 只消虚调用（~5%）不消 11× 根 | 🔍 DFC 非良药（MT11，机制不匹配） |
| typeid 遍历「无 spline」→ 误判「spline 不在 density」 | typeid 遍历漏 BlendDensityDF/WrappingDF（包装类 arg 未递归）→ spline 经 blend_density 引用分量，6 实例被漏 | 🩹 已纠（MT10，补全遍历 = 6 SplineDF） |
| MVP 并发线程扫描「未复现」density 11×（每样本 0.2× 完美扩展） | 微基准与生产负载特征不同：MVP 表小（245 元素全驻留 L1/L2 无争用）+ locFn 轻量子类（无 shared_ptr 散布堆指针追逐）+ 无「8 线程同读同一批共享对象」压力 → 复现不了 production 的 11×（共享内存延迟/指针追逐）；MVP「没复现」≠「11× 不真实」（production 11× 已由 WG_PHASETICK 另节确证） | 🔍 已定性（对照实验） |

---

## MT12. 🔥 SERIAL 的 `static_cast<const DensityFunction&>(pool[i]).sample()` = 强制虚调用（SERIAL 从未去虚分派）——A/B 只证「存储非争用」，误以为证了「虚分派非争用」（🔍 已纠正）

### 现象
- SERIAL A/B 结果：BASE 10.03× / SERIAL 10.25×（放大比持平）。
- 初读该结果时（叙述层面）把「SERIAL 」当成「已验证 locFn 存储 + 虚分派综合非争用」——但后续 DEVIRT 单独去虚分派（10.05×）几乎无变化，才暴露 SERIAL 自身**根本没去虚分派**。

### 根因（机制层面）
- `sampleSerialLocFn`（density.h）的 kind-switch 分支写的是：
  ```cpp
  case FLAT_CACHE: return static_cast<const DensityFunction&>(flatCachePool[r.index]).sample(pos);
  ```
- `static_cast<const DensityFunction&>(obj)` 把**具体类型的实体临时转成基类引用**，再调 `.sample(pos)` —— `sample` 是**虚函数**，经基类引用调用**必然走 vtable 分派**。
- 所以 SERIAL 只去掉了两样东西：**shared_ptr deref**（不再间接访问）+ **存储连续化**（池实体连续）。**虚分派从未被去掉**（kind-switch 后一视同仁转回基类引用）。
- 结论链条误读：SERIAL 只隔离了「存储布局（A 类）」的贡献，**没有**隔离「虚分派」的贡献。A/B 只能证明「存储非争用」；「虚分派非争用」是让 DEVIRT（真正去掉基类引用 cast）单独证的。

### 定位
- 读 `density.h` `sampleSerialLocFn` 源码 → 见 `static_cast<const DensityFunction&>` 三处（FLAT_CACHE/CACHE_2D/BINOP）→ 确认转基类引用调用虚函数。
- 交叉印证：DEVIRT 改法（去掉 cast、具体类型直接 `.sample()`）放大比 10.05× ≈ BASE 10.32× → 说明虚分派本来就不是 11× 主导 → 反推 SERIAL 的「虚分派未被改动」成立。

### 修复
- 下一步 DEVIRT 修改：把 `sampleSerialLocFn` 三个 case 的 `static_cast<const DensityFunction&>(pool[i]).sample()` 改为**具体类型直接调 `.sample()`**（by-value 池实体，语义上可 devirtualize，O2）。env `WG_SERIAL_LOCFN=1`（DEVIRT）。
- 语义：去掉转基类引用 → 编译器能确定静态类型 → 去 vtable 跳转。

### 教训（判错经验）
1. **「kind-switch + 基类引用调用」≠ 去虚分派**——判断「是否已去虚分派」要盯**`.sample()` 是否经基类引用发起**（`static_cast<const DensityFunction&>` 即虚调用），而不是看有没有 kind-switch。
2. **A/B 隔离变量要精确到「机制维度」**——「存储布局」与「虚分派」是两类不同的代价；一次 A/B 只能隔离它真正改动的那一类。若 A/B 同时保留了另一个候选（虚调用），其结论不得外推为「那个候选也被测过」。
3. 复用判错：看到 `static_cast<const Base&>(derived_obj).virtualMethod()` 模式，默认它仍是虚调用。

---

## MT13. 🔥 conc_sample_probe scattered 坐标失真——spline 探针 per-sample 0.44ms（比 production 慢 1000 倍），grid 重建主导（✅ 已修正）

### 现象
- conc_sample_probe spline 模式初版：per-sample = **440552ns（0.44ms）**——比 production 的 spline 单次（μs 级）慢约 **1000 倍**，完全失真。
- 修正后（固定同 chunk）：per-sample **4493.5ns**（快 98×），spline 并发放大 1.22×/1.21×。

### 根因（机制层面）
- spline 的 locFn（ContinentsDF 等 = FlatCacheDF）grid **按 chunk 懒建**（FlatCacheDF/Cache2DDF 的 grid/缓存 key 依赖 `g_curChunkX/Z`）。
- scattered 坐标 `x=3200+(i*17)%2048` → **跨越 128 个不同的 chunk** → 每个采样点落在不同 chunk → **每换一个 chunk 就触发一次完整 buildGrid（重建 25 点 grid）**。
- 结果是探针成本被 **grid 重建**主导（每采样一次重建），而非生产路径（同 chunk grid 命中 + 只读）。这不反映生产行为。

### 定位
- 对比初版 scattered per-sample 0.44ms 与修正后固定同 chunk per-sample 4493.5ns（快 98×）→ 差异来自「是否跨 chunk 重建 grid」。
- 对照 production `fillOneChunkCore`：是「同 chunk grid 命中」访问模式（fillOneChunkCore 处理单 chunk，所有采样在同一 chunk 坐标域，grid 命中）。→ 探针必须复刻这一访问模式。

### 修复
- 改 conc_sample_probe 固定 x,z 同 chunk（3200-3215 / 3224-3239）、y 扫 → grid 命中 → per-sample 4493.5ns（可靠）。

### 教训（判错经验）
1. **探针必须复刻 production 的访问模式**（同 chunk grid 命中），否则测的是「探针自己的失真路径」而非生产路径。
2. **探针初值要先用「合理性检查」**：per-sample 比 production 慢 1000 倍本身就说明有系统性失真（要么探针 bug，要么访问模式错），应先排查再下结论——不要直接拿失真数据做排除链依据。
3. 复用判错：凡探针里按坐标懒建缓存的组件（grid/FlatCache/Cache2D），scattered 坐标必触发重建，须固定同 chunk / 复刻生产 chunk 域。

---

## MT14. 🔥 conc_sample_probe(std::thread) ≠ conc_density_probe(wg_worker pool) 线程模型混淆——spline 1.2× 不能独立证明「spline 在 production 下无争用」（⚠️ 已纠正，spline 1.2× 降为辅证）

### 现象
- conc_sample_probe spline 模式（std::thread）测 spline 并发放大 **1.2×**（接近无争用）。
- production 全 tree 并发放大 **10.32×**。
- 两者悬殊 → 一度不严谨地倾向「spline 在 production 下也无争用」。

### 根因（机制层面）
- **线程模型不同**：conc_sample_probe 用 **std::thread**（每线程独立循环采样）；production 争用（10.32×）发生在 **wg_worker pool**（wg_fill_blocks_multi 填 chunk，CoreSwapPool 队列 + worker）。
- std::thread 各自独立循环 → 每线程跑自己的数据，**不存在 pool 的任务调度 + 共享队列 + 线程间交互** → std::thread 下多入口都低放大（noise 1.15× / spline 1.2×）。
- 所以 spline 的 1.2× **无法排除「std::thread 模型本身无争用」的伪影**——这可能是「std::thread 下测什么都低」，而不是「spline 生产无争用」。

### 定位
- 对比 conc_sample_probe（std::thread 实现）与 conc_density_probe（wg_fill_blocks_multi 填 chunk 实现）→ 确认两者线程模型不同。
- 交叉证据：全部「低放大」入口（noise 1.15×/spline 1.2×）都来自 std::thread 探针；全部「高放大」（10.32×）来自 production 池 → std::thread 探针自身不放大，问题在模型不一致。

### 修复
- 设计 WG_SPLINE_FILL（fillOneChunkCore 加 env，density 采样绕 wrapper 直接 `spl[which]->sample(fpos)`），用 **production 线程池**（wg_fill_blocks_multi + conc_density_probe）测 spline 绕 wrapper。
- 结果：spline-only[2] 1.62×（production 池）→ 确认 spline 在 production 下也几乎无争用（1.62× vs 全 tree 10.32×）。

### 教训（判错经验）
1. **并发放大对照必须同一线程模型**（生产线程池 vs std::thread 是不同的并发形态）；跨模型对比不可靠。
2. **不要用「std::thread 探针的低放大」去反推「production 池里的低放大」**——两种模型的争用结构不同（std::thread 独立循环无池调度/共享队列压力）。
3. 复用判错：任何「并发放大/无争用」结论，先确认它是在 production 同一线程模型（wg_fill_blocks_multi + CoreSwapPool）下测的，还是在独立的 std::thread 微基准下测的（后者仅作辅证）。

---

## MT15. scout 静态误判「buildGrid 深链=91% 主争用」——warm 实测推翻（虚调用数 ≠ 争用贡献）（✅ 已纠正）

### 现象
- scout（wrapper-buildgrid-structure.md / 83c9d1b0）断言：buildGrid 深链（interp#1 每点重走 18-20 层实虚分派 + spline 递归）= 91% 走 wrapper 链的时间，是 11× 主争用。
- 但 warm 实测：预建 grid（排除 buildGrid 深链）后仍 **10.10×**（vs cold 10.32×，差 0.22×）→ buildGrid 深链对 11× 争用贡献 **微乎其微（<2%）**。

### 根因（机制层面）
- scout 用的是**静态虚调用次数**推导争用占比：buildGrid 虚调用深（每点 18-20 层）→ 它「看起来」是大头（17.6K/chunk、含深链下探）。
- 但**虚调用次数 ≠ 争用贡献**——争用（latency QoS / 延迟排队）本质是**并发下访存排队的放大**，与「单次调用长不长」不直接对应。buildGrid 是**每 chunk 冷路径一次性**（每 chunk 触发 1 次/实例），8 线程各触发自己的 buildGrid 不互相排队摊薄（warm 去它后几乎没有变化）；而**顶层逐点包装**是**每 chunk 98304 点 × 每点（warm 后仍在）**，是真正的 per-point 并发放大面。
- 静态看「buildGrid 深」≠ 动态争用大；scout 忽略「冷路径一次性 vs 温暖 per-point 重复」的并发放大差异。

### 定位
- warm/cold 实测（production 模型 conc_density_probe）：cold 10.32× vs warm 10.10×（差 0.22×）→ buildGrid 对 11× 贡献 <2%。
- 对照 scout 静态断言（buildGrid=91%）→ 静态推断被运行时测量推翻。

### 修复
- 无代码修复（这是诊断判断修正）。
- 结论修正：11× 主争用 = **顶层逐点包装**（min/squeeze/mul/interp 每点 98304× 重复）+ 后续收窄到 interp/noodle 采样内部；buildGrid（冷路径）无碍。

### 教训（判错经验）
1. **静态「虚调用数/深链」推断不能代替运行时争用测量**——争用是并发访存排队现象，与「单次调用深不深」不是一回事；冷路径一次性 vs 温暖 per-point 重复的并发放大差异必须用实测区分。
2. **判别「争用在哪」要用「单一变量剔除」**（warm 排除 buildGrid / WG_FLAT_TOP 排除顶层包装），不能靠静态结构数推断占比。
3. 复用判错：凡「xxx 深/虚调用多 → 必是争用大头」的静态断言，都要用「剔除该项的 A/B」验证；剔除后放大比不变即该项非争用。

---

## MT16. wg_sample_density 单点无 grid 缓存——std::thread 20000 点超时（每点 buildGrid 6ms）（🔍 已记录，探针入口需 grid 缓存）

### 现象
- wg_sample_density（whole tree 单点）用 std::thread 循环 20000 点 → 120s 超时。
- 原因：每点触发一次整树 buildGrid ≈ 6ms → 20000 点 ≈ 120s。

### 根因（机制层面）
- `wg_sample_density` 单点采样走整棵 finalDensity 树，每点 `finalDensity->sample(pos)`；InterpolatedDF 首次采样触发 `buildGrid`（懒建 5×49×5=1225 点 grid，每点 arg 下探深链，怪物树 ≈ 27.9ms/次建）。
- **窗口/入口无 grid 缓存**：每次调用都是「新 chunk、首访」→ 每点重建 grid（无 thread_local grid 命中复用）。
- 与 production `fillOneChunkCore` 不同：后者对单 chunk 处理所有 98304 点，grid 只建一次（首点），后续点命中复用 → 摊薄到 0.4μs/点。单点入口无此摊薄。

### 定位
- 超时（120s cap）+ 反推单点 6ms（20000 点 × 6ms ≈ 120s）。
- 对照 production fillOneChunkCore（同 chunk grid 命中复用）→ 确认走「whole tree 单点无 grid 缓存」路径。

### 修复
- 改用 WG_SPLINE_FILL（production 模型，绕 wrapper 只测 spline）做严格对照；避免「std::thread 循环 whole-tree 单点」。
- 探针入口设计：采样 whole tree 必须**先预建 grid / 固定 chunk / 同 chunk grid 命中**，否则失真或超时。

### 教训（判错经验）
1. **whole-tree 单点采样必须 grid 缓存**（固定 chunk + 预建 grid），否则每点重建 grid → 失真/超时。
2. **探针入口要复刻 production 的 chunk 内 grid 命中复用**，不能用「每点独立 whole-tree 单点」——那会触发 buildGrid 深链（冷路径），测的是 buildGrid 不是 warm per-point 争用。
3. 复用判错：凡含 InterpolatedDF/FlatCacheDF/Cache2DDF 懒建缓存的探针，单点采样必建 grid；multi-point 同 chunk 才命中。

---

## MT17. 改生产路径（WG_FLAT_TOP）后没先 block_probe 对拍就下性能结论——须逐位一致才可信（✅ 本项已执行对拍，此为纪律沉淀）

### 现象
- WG_FLAT_TOP 性能结论（10.55× ≈ 生产 10.32×，「减少虚分派不降 11×」）若**不先逐位对拍**，会建立在「同算术理论上一致」的推断上，无法排除 WG_FLAT_TOP 因改错算术而失效的假象。
- 本项正确流程已执行：block_probe `-save`（WG_FLAT_TOP=0/1 同参照 `vanilla_8576294172403134396_6_720_-432.blocks`），`out_prod.bin` vs `out_flat.bin` **SHA256 完全一致（identical: True）** → WG_FLAT_TOP 逐位一致。

### 根因（机制层面）
- WG_FLAT_TOP 是**改写生产采样路径**（把 min/squeeze/mul 扁平化为内联算术），若算术/边界/顺序有一处不同 → 采样值错误 → 性能对比建立在**错误代码**上，结论不可信。
- 理论上「同算术」可由源码推演，但**浮点顺序/rounding/clamp 边界**（squeeze 的 clampD、min 分支 `da<bmin`）无法仅凭静态推演保证逐位一致——必须实证对拍（Full = block_probe 逐位）。
- 性能结论若建立在未对拍的改动上，可能得出「改动 A 无效」的实际是「改动 A 改错了」。

### 定位
- 性能对比前先做正确性对拍：block_probe -save 导出对照 → SHA256 逐位比较。
- 本项：`out_prod.bin` vs `out_flat.bin` SHA256 identical → 确认一致性后才记录 「WG_FLAT_TOP 逐位一致 + 10.55× ≈ 生产」结论。

### 修复
- 对 WG_FLAT_TOP 执行 block_probe 对拍（SHA256 identical）→ 确认逐位一致 → 性能结论可信。
- 教训沉淀为「改生产路径后必须 block_probe 对拍（Full）再下性能结论」的固定纪律。

### 教训（判错经验）
1. **改生产路径后的性能结论必须先过正确性门（Full = block_probe 逐位对拍）**——性能对比不能建立在未验证正确性的代码上（否则「改动无效」可能是「改动改错了」）。
2. **理论推演（同算术）≠ 逐位一致**——浮点顺序/rounding/clamp 边界需实证；用 block_probe -save + SHA256 identical 做硬验证。
3. 复用判错：凡性能 A/B 改动了采样/求值路径，先对拍正确性（SHA256 逐位）再读性能；不改采样路径的纯调度/存储实验（M1 pin 核/serial 池）可不全对拍，但改动求值语义的必须对拍。

---

## 附：本轮新增错误 → 根因 速查表（追加到既有速查表尾部）

| 错误 | 一句话根因 | 状态 |
|---|---|---|
| SERIAL 与 BASE 放大比持平（10.25× vs 10.03×），误以为「虚分派已测」 | `static_cast<const DensityFunction&>(pool[i]).sample()` = 转基类引用调虚函数（**强制虚调用**）；SERIAL 只去 shared_ptr deref + 存储连续化，**从未去虚分派** → A/B 只证「存储非争用」 | 🔍 已纠正（DEVIRT 单证虚分派非争用，MT12） |
| conc_sample_probe spline per-sample 0.44ms（比 production 慢 1000 倍） | spline locFn grid 按 chunk 懒建，scattered 坐标跨 128 chunk → 每换 chunk 重建 grid → grid 重建主导（非生产同 chunk 命中路径） | ✅ 已修正（固定同 chunk，MT13） |
| spline 1.2×（std::thread）vs production 10.32× 悬殊，一度误断「spline 无争用」 | conc_sample_probe 用 std::thread（独立循环）、conc_density_probe 用 wg_worker pool（填 chunk）——**线程模型不同**；std::thread 下多入口都低放大（noise 1.15×/spline 1.2×） | ⚠️ 已纠正（spline 1.2× 降为辅证；WG_SPLINE_FILL 用 production 池测，MT14） |
| scout 静态断言「buildGrid 深链=91% 主争用」 | 静态虚调用数（buildGrid 深 18-20 层）推导占比，但**虚调用数 ≠ 争用贡献**；buildGrid 是冷路径每 chunk 一次性（warm 剔除后近乎无变化），顶层逐点（98304 点/次）才是 per-point 并发放大面 | ✅ 已纠正（warm 10.10× ≈ cold 10.32×，MT15） |
| wg_sample_density std::thread 20000 点 120s 超时 | 单点 whole-tree 采样无 grid 缓存（每点触发 buildGrid ≈6ms）→ 20000 点 ≈120s；与 production 同 chunk grid 命中复用（0.4μs/点）不同 | 🔍 已记录（探针入口需 grid 缓存，MT16） |
| WG_FLAT_TOP 性能结论可信度 | 改生产采样路径（min/squeeze/mul 扁平化），未对拍前基于「同算术理论一致」不可信；须 block_probe 逐位（SHA256 identical）实证 | ✅ 已对拍（out_prod vs out_flat SHA256 identical，MT17） |

> **主会话应用注意**：①-⑥ 分别编号 MT12-MT17 追加到 `mt-scaling-errors.md` 末尾 + 全局速查表各加一行（见上）。编号不覆盖既有 MT1-MT11。
