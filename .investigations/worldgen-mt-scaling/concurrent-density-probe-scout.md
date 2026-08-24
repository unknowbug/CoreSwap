# 并发 density 阶段单 chunk 延迟暴涨 ~10× 机制勘探（只读）

> 勘探角色：本 subagent（只读，未编译未改码）| 日期：2026-08 后续
> 目标：读并发执行路径，核对 a)-e) 并发延迟候选机制，给出「证据 / 排除」+ 最可能机制排序 + 下一步验证动作。
> 数据基线（直接采信，来源 density-latency-rootcause.md + per-chunk-concurrent-slow-mtrace.md）：
> - WG_PHASETICK（干净，QPC 单次无 profiling）：density T=1 34-42ms → T=8 **400-412ms（11×）**；total 50ms→462ms（9×）；每 chunk 真实 462ms（T=8）vs 50ms（T=1）。
> - 自洽验证：462ms × 8 并行（64 chunks = 8 批）≈ 3696ms + 批间 ≈ wall 4618ms ✅。
> - bench wall/N：T=1 69ms/chunk vs T=8 73ms/chunk（**平均吞吐近似，非每 chunk 延迟**——多线程下掩盖单 chunk 延迟）。

---

## 一、已确证事实（压缩，带证据出处）

1. **density 阶段本身 = 每 chunk 98304 点**（fillOneChunkCore L784-801：by<noiseHeight × bz<16 × bx<16，每点 `fd = h->finalDensity->sample(fpos)` L795）。— `worldgen_api.cpp:784-801`
2. **finalDensity 顶层是 InterpolatedDF**（grid 网格 4×4×8 三线性插值，懒构建 + thread_local 缓存）。稠密 98304 点循环只做「grid 角点读 + 三线性」，真正贵的是 **buildGrid 时对 `arg->sample(p)` 的 1225 个角点采样**（5×49×5），arg = squeeze/spline 链。— `density.h:482-620`（InterpolatedDF::sample/buildGrid，edgeCol 复用 L596-618）
3. **spline 树真实存在且是每 chunk 重建网格的主成本**：6 个 SplineDF、537 节点、**17KB 表（17112B，驻留 L2，排除 L3 容量问题）**、195 locationFunction。— `density-latency-rootcause.md:44-50`；`density.h:811-941`（SplineDF）
4. **spline 单次采样成本高且并发放大**：T=1 15.8μs → T=8 190μs（12×）——数据反推自 WG_PHASETICK + spline 计数；背景另给 WG_PROFILE 实测 spline 45.7μs / noise 1.12μs。— `density-latency-rootcause.md:57-69`
5. **所有采样缓存/上下文皆是 thread_local，无跨线程可变共享**：InterpolatedDF::tlSlots（density.h:576-578）、Cache2DDF::tlSlots（:710-712）、FlatCacheDF::tlSlots（:776-778）均为 `static thread_local std::vector<Slot>`；`g_curChunkX/g_curChunkZ` 为 `thread_local`（:40-41）。**排除共享可变缓存竞争**。
6. **noise sampler 全部 const 只读**：PerlinNoiseSampler::sample const（noise.h:50/54）、OctavePerlinNoiseSampler::sample const（:214）、DoublePerlinNoiseSampler::sample const（:282）——只读 perm 表，无写共享。**排除 sampler 共享状态**。
7. **density 热路径无锁**：哨兵 L795 只调 `finalDensity->sample`（虚调用一路到 SplineDF，纯读）；L797 `if (beard) fd += beard->sample(...)` 是**门控**（无结构时 beard=nullptr，不执行）；beardifier 采样用 **beardLocal 拷贝**（L739-745 锁内 copy 后 release），不在热循环里。density 内循环无 any mutex。— `worldgen_api.cpp:736-801`
8. **Beardifier 线程安全，无 per-point 锁**：weightTable() 是函数局部 static（C++11 magic static，线程安全惰性初始化，beardifier.h:82-96）；sample() const（:131）只读 pieces/junctions（操作拷贝）；无锁。**但**：L797 每点调用点在**有结构 chunk** 才触发，bench（无 wg_set_beardifier，beardifiers 空）下 beard=nullptr，**与观测到的 10× 无关**。— `beardifier.h:82-165`
9. **线程池补建 worker notify 竞争已修复**（0a781e1：readyCount 原子 + run() 入队前等 readyCount==workers.size()），8 worker 真并行（MTTRACE enter 相同 exit 相近，non-serialized）。— `worldgen_api.cpp:1086/1126-1134`
10. **CORESWAP_THREADS 默认 = physicalCoreCount()**（GetLogicalProcessorInformationEx(RelationProcessorCore) 数物理核，SMT 不重复计）。bench 打印 hw_threads=24，物理核应为 12（12C/24T）。— `worldgen_api.cpp:1043-1064, 1196-1204`

---

## 二、并发延迟候选机制表

> 结论先行：在 **[A] 批模式（count=N=64，T=8）** 下测得的每 chunk density 11×，机制不是「调度 / 锁 / 超线程」，而是 **spline 树递归虚调用链在 8 线程并发下的共享内存延迟放大（cache-line / code / prefetch）**。a) b) d) 对该 11× **排除为主因**；c) 是主因方向；e) 与观测无关。

| # | 机制 | 证据（文件:行号） | 是否可能解释 10× 延迟 | 下一步验证方法 |
|---|---|---|---|---|
| **a-1** | **CoreSwapPool run() yield 空转**（L1126-1134 `std::this_thread::yield()` 忙等 readyCount==workers.size()） | `worldgen_api.cpp:1121-1144` | **排除（对 [A] 批模式）**：run() 每批只调一次（count=N 单次入队），yield 旋转只在批量派发前发生，不在 fillOneChunkCore 内、不在 WG_PHASETICK 测的 density 区间（ph0→phA 在 fillOneChunkCore 内部 L730/L803）；稳定池后 readyCount 即刻满 → 无旋转。**不能**解释每 chunk density 11×。 | 计数器：记录每批 run() 自旋次数/时长（性能计数器而非打印）；对比 [A] T=1 vs T=8 的 batch-dispatch 开销（应 <1ms）。 |
| **a-2** | **count=1 模式（[B]/实机 JNI）per-chunk 派发开销**：每个 worker 每 chunk 调 wg_fill_blocks_multi(count=1) → run(1) 各付一次 readyCount 自旋 + notify_all + cvDone.wait | `worldgen_api.cpp:1186-1222`；bench `bench_chunks.cpp:126-171` | **部分相关，但非密度阶段放大器**：这是**派发/调度开销**（在 fillOneChunkCore 之外），影响 [B] 每 chunk wall，**不在 WG_PHASETICK 的 density 区间内**。与「单 chunk density 400ms」无直接因果。 | 测 [B]/实机 count=1 的 run() 往返时间（空任务基准）；对比 [A] batch 派发。 |
| **b** | **8 worker 抢超线程/物理核**（8 线程是否落在同一物理核的 2 逻辑线程上） | `worldgen_api.cpp:1043-1064`（pool ≥ physicalCoreCount=12）；`density-latency-rootcause.md`（T=8 ≤ 12 物理核） | **排除为主因**（对 T=8）：pool 线程数=物理核数=12，T=8 工作线程 ≤ 12 物理核 → 不触发 SMT 共享执行单元；且 scout-map C2 实测 GHz 恒 2.99（无降频），归一化后反降依旧 → 睿频/SMT **已排除为主因**（`scout-map.md:85-99`）。T>12（如 22）才有 SMT 叠加但主因在 8 已显现。 | 对照实验：将 8 worker 分别绑到 8 个不同物理核（SetThreadAffinity）与全部绑到 1 核，对比 spline 单次成本——若绑不同核仍放大 → 非调度/SMT。 |
| **c-1** | **I-cache / code-fetch 争用**（8 线程执行同一段样条递归代码，失同步互相驱逐 L1I） | `density.h:876-925`（sampleNode 递归 + virtual call `locationFunctions[nd.locFn]->sample` L879）；8 线程同跑同一冷路径 | **强候选**：代码工作集（sampleNode + FlatCacheDF/NoiseDF/ShiftedNoiseDF/Octave/Perlin/InterpolatedNoiseDF 各 sampler）虽小，但 8 线程**失同步**时互趋 L1I → L1I miss → 从 L2 取指。每层递归 + 每 virtual call 触发更多取指。可放大长依赖链。 | ① I-cache miss 计数器（perf stat `iTLB-load-misses` / `icache`）；② 用 `__noinline`/`likely` 隔离热点函数；③ 对比「8 线程同时滑同 chunk 区域」（同步执行，共享 I-cache）vs「8 线程分散不同位置」（更失同步）——若后者更慢 → I-cache 争用。 |
| **c-2** | **spline 树数据读取的共享内存延迟放大**（nodes/locations/derivatives/subIdx + 195 个 locFn 对象共享只读，8 线程并发遍历读同一批 cache-line，prefetch 被扰乱、MSHR/L2 压力 ↑，长依赖链每级 load 延迟叠加放大） | `density.h:879-925`（locationFunctions 为 `std::vector<DF>`=shared_ptr<DensityFunction>，locFn 对象散布堆 → 指针追逐）+ 表 17KB 共享读 | **最可能主因（与既有定位一致）**：spline `sampleNode` 是**长串行依赖链**（递归子节点→取子值→lerp），每级做随机读（nodes/locs/ders/subs）+ virtual call + shared_ptr 间接。这是**内存延迟受限**负载。8 线程把这些依赖链的 load 流交错灌入 L1/L2/load-store 单元 → 每线程有效 load 延迟↑ → 长链乘法叠加 = 10-12×/sample。**无锁也能发生**（不互斥，只是争共享 cache 资源）。 | ① 微观基准：直接对同一棵 SplineDF 树（不做 InterpolatedDF grid，去掉外层）在 1/2/4/8 线程下各跑固定次数，测每样本成本 vs 线程数——若随线程数↑即坐实；② 读 L1/L2/L3 miss 率 + load 延迟（perf stat / ETW）在 T=1 vs T=8；③ **决定性**：跑 DFC 显式栈（MVP path B，连续表 + 无递归无虚调用）同并发设置对比——若不随线程数放大 → 证实「递归+虚调用指针追逐」是放大器，且直接检验 C2ME DFC 修复方向。 |
| **c-3** | **17KB 表 + locFn 对象的 L2 broadcas / set-conflict**（8 线程共享读同一小表是否广播友好） | `density.h:822-833`（表 17KB）；`density-latency-rootcause.md:44-50` | **排除为主因（单项）**：17KB 只读小表，**共享读在 cache 中广播**，不产生 ping-pong（无写）。单纯「表共享读」不足以解释 10×——真正开销在 **locFn 对象指针追逐**（shared_ptr 指向散布堆对象 + vtable），而非表本身。 | 把 locationFunctions 从 `std::vector<DF>`（shared_ptr 指向堆对象）改为**连续对象数组**（指针追逐消除）对比——若单线程即显著变快 + 并发放大减小 → 指针追逐是关键。 |
| **d** | **硬锁**（beardifierMtx / regionColsMtx / pendingCrossMtx / carverCacheMtx） | `worldgen_api.cpp:208/739`（beardifier 锁内拷贝 1×/chunk）；`:222/1012`（regionColsMtx 仅 FULL phase1 storeRegion）；`:226/1247`（pendingCrossMtx 仅 phase2）；`:1319-1327`（carverCacheMtx 仅 FULL carvers） | **排除为主因**：density 内循环（L784-801）**无任何锁**；beardifierMtx 1×/chunk 在密度循环**之前**（L739，µs 级无争用），且锁内只做拷贝后即 release。regionCols/pendingCross/carverCache 均只在 FULL 模式（SURFACE bench 不触达）。**「必须 mutex 保护」注释（L207）指的是 beardifier 数据结构的读写并发，实际用锁内拷贝避免,不进入热循环**。 | 静态已闭环（负结果本身是资产）；如需实证：在 [A] bench 下用 `QPC` 累计各 mutex 的 lock 等待时间（仅 FULL 模式需测）；SURFACE bench 应全 0。 |
| **e** | **Beardifier / 其它阶段每点调用**（L797 `if (beard) fd += beard->sample(...)`；L916-930 aquifer+oreVein；L981+ surface） | `worldgen_api.cpp:797/916-930/981+` | **排除（对观测到的 10×）**：L797 是 `if (beard)` 门控——bench（无 wg_set_beardifier，`beardifiers` 空）→ beard=nullptr → **L797 不执行**。Beardifier 本身线程安全（weightTable magic-static :83；sample() const :131 无锁；操作 beardLocal 拷贝）。aquifer/oreVein/surface 是 per-chunk 局部对象 + buildSurface 读 router（纯读），非密度阶段主因（那 3 阶段并发仅放 3-4×，非 11×，见 rootcause 表）。 | 对照：在**带结构** chunck（wg_set_beardifier 后）测 T=1 vs T=8 的 density 差异，确认是否 Beardifier 引入额外并发放大（预期非主因）；但**当前 bench 无结构**，此候选不适用。 |
| **（附加）f** | **InterpolatedDF edgeCol 边界列复用失效**（并发 FIFO 分散 chunk 几乎不命中 `reuseLeft`，每 chunk 多建一列网格） | `density.h:570-571/596-618` | **贡献项（少量）**：T=1 行主序相邻 chunk 命中 edgeCol 复用（省一列 49×5 采样），T=8 分散 chunk 不命中 → 每 chunk 多做少量采样。**固定小幅惩罚，非 10×**（scout-map C4 估 3-5%）。 | 行主序 vs 随机洗牌对比（T=1 下）；edgeCol 命中计数器（thread_local 计数非打印）。 |
| **（附加）g** | **每 chunk 大块堆分配 DMA/TLB**（densityBuf 786KB + col 393KB） | `worldgen_api.cpp:721/731` | **已排除（MT5/C1 回滚）**：thread_local 复用改造实测单线程反慢 9%、多线程反降依旧 → 每 chunk 1.2MB 堆分配/释放**不是 MT 主因**（`mt-scaling-errors.md:MT5`；`scout-map.md:73-77`）。已回滚（8966ba9）。 | 不重复验证（已闭环）。 |

---

## 三、最可能机制 top-3 排序 + 理由

> 判定口径：**能否解释「无锁 + 真并行 + 但单 chunk density 10-12× 工作膨胀」**。a/b/d/e 均无法解释（要么每批一次、要么在密度区间外、要么无物理基础、要么观测路径不触发）；c 类机制（共享内存延迟 + code-fetch）是唯一能自洽的。

1. **Tier-1（主因）：SplineDF 树递归虚调用链的并发共享内存延迟放大**（c-2）。
   - 理由：spline `sampleNode`（density.h:876-925）是**长串行依赖链 + 指针追逐**：递归至 ~90 节点/实例，每级 `locationFunctions[nd.locFn]->sample(pos)`（virtual call + shared_ptr 间接，locFn 对象散布堆）+ 随机读 nodes/locations/derivatives/subIdx。这是**内存延迟受限**负载。8 线程并发把各依赖链的 load 流灌入同一 L1/L2/load-store/MSHR → 每线程有效 load 延迟↑ → 长链每题延迟乘法叠加 → **无锁也单样本 10-12×**。与「无锁、真并行(同时进出)、但单 chunk 膨胀」三者皆自洽；也与既有定论（spline 单次 15.8→190μs，12×）吻合。这正是 scout-map C7「共享内存延迟 QoS」方向（`scout-map.md:95`）。
   - 备注：17KB 表本身是只读共享读（广播友好，不 ping-pong）——**瓶颈不在表，在 locFn 指针追逐 + 长链**。

2. **Tier-2（叠加）：I-cache / code-fetch 争用**（c-1）。
   - 理由：8 线程执行同一段冷递归代码，虽共享代码行，但**失同步**时互相驱逐 L1I → L1I miss → 取指变慢；每层递归 + 每 virtual call 触发额外取指。为 Tier-1 的长链再叠加取指延迟。单独难构成 10×，但与 Tier-1 叠加后放大显著。

3. **Tier-3（贡献/基线污染，需量化）：InterpolatedDF edgeCol 复用失效**（f）。
   - 理由：T=1 行主序命中 edgeCol 复用（省一列采样），T=8 分散 chunk 不命中 → 每 chunk 固定多一小块采样。是「T=1 基线便宜、T=8 更贵」的**基线差异**，非并发本质；估 3-5%（scout-map C4）。需在排序后量化，不能凭直觉归入主因。

**明确排除（对观测 10×）**：a-1（每批一次）、b（T=8 ≤ 12 物理核，无 SMT，且 GHz 恒 2.99 已排除）、d（density 内循环无锁）、e（beard 门控 null，L797 不执行）。a-2 只在 [B]/实机 count=1 的**派发**有影响（密度区间之外）。

---

## 四、推荐的下一步验证动作（排序，测量全程用 WG_PHASETICK 干净工具，禁 WG_PROFILE/WG_STAGETIMER）

1. **（决定性）隔离 spline 单样本并发成本**：写独立微基准（或临时探针），对同一棵 SplineDF 树（**去掉 InterpolatedDF 外层**，直接 `spline->sample(pos)` 循环）在 1/2/4/8 线程（各线程 pin 到不同物理核）下各测 N 次，输出**每样本成本 vs 线程数**。若随线程数明显↑（如 15→190μs 量级）→ 坐实「共享内存延迟放大，非调度/锁」。这是把 10× 从「每 chunk 表象」剥到「每 spline 采样」的核心。
   - 变体 a：线程全部 pin 到**同一个**物理核 vs 分散到不同核——若同核才放大 → 调度/SMT；若分散也放大 → 共享内存服务延迟。
   - 变体 b（一石二鸟，兼验证 C2ME DFC 修复）：用 DFC 显式栈版本（`.investigations/perf-rework/vulkan-proto/mvp_spline_eval.cpp` 同构，连续表无递归无虚调用）跑同一并发设置——**若不随线程数放大 → 证实「递归 + 虚调用指针追逐」是放大器，且直接预期 DFC 直排可消除 11×**。

2. **量测 cache/miss 与 load 延迟**（需运行工具，本沙箱无 perf）：perf stat 读 L1/L2/L3 miss rate + `dTLB-load-misses`，在 T=1 vs T=8（同 bench）对比。若 miss 率/延迟显著↑ → 支持 Tier-1/Tier-2。

3. **剥离 baseline 偏差（edgeCol / 调度顺序）**：[A] bench 下用**行主序 vs 随机洗牌**chunk 序对比 T=1；并统计 thread_local edgeCol 命中数。量出 Tier-3 的固定贡献，排除「10× 部分来自 T=1 基线便宜」的混淆。

4. **（旁证）确认 locationFunctions 是否是连续对象**：把 `std::vector<DF>`（shared_ptr 指向散布对象）改为连续对象数组的 A/B（单线程 + 8 线程）——若单线程即显著变快、并发放大减小 → 证明「shared_ptr 指针追逐」是非递归部分的关键，同时佐证 DFC 直排（连表）的收益来源。

5. **（排除项）锁审计实证**：[A] SURFACE bench 下累计各 mutex lock 等待（QPC 单次计数值），预期 beardifierMtx≈0、regionCols/pendingCross/carverCache 全 0，静态排除 d) 收尾。

---

> 结论：**最可能 = SplineDF 树递归虚调用链的共享内存延迟放大（Tier-1）+ I-cache/code-fetch 争用（Tier-2）**，两者协同把每 chunk density 从 34ms 拉到 400ms 且**无需任何锁**；a/b/d/e 均排除为观测 11× 的主因。下一步 = **拆分并发下 spline 单样本成本**（变体 b 一并验证 DFC 显式栈修复方向）。
