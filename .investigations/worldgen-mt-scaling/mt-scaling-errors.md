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

## 判错经验（可复用，2026-08-16 提炼——优先级高于单条错误）

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
