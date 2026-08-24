# interp/noodle 采样的内存访问结构 — 11× 争用来源勘探（只读 scout）

> 角色：scout（只读，纯静态读码，未编译未改码）| 日期：2026-08 后续
> 前提（已确证，直接采信）：production density 并发 11× 争用。三层对照已排除 spline-only(1.62×)/warm-des-buildGrid(10.10×)/WG_FLAT_TOP-des-virtual(10.55×)/production(10.32×) → **争用在 interp/noodle 采样内部（memory access pattern）**，非虚调用数、buildGrid、spline、min/squeeze/mul 虚分派。
> 本文只做「内存访问结构」量化 + 判断带宽 vs SMT + 可测量方法。**不重复已排除项**（虚调用/buildGrid/spline）。
>
> 标记约定：**确证** = 源码/JSON 直接读清（行号内联）；**推断** = 需运行验证；**@anchor.idk** = 静态无法定论。

---

## 0. 结论先行（TL;DR）

| 问题 | 判断 | 置信度 |
|---|---|---|
| interp grid 是否跨线程共享？ | **否**，`thread_local`（每线程独立 vector，按 cacheId 索引） | 确证（density.h:576-578） |
| interp#1 命中后每点读？ | **8 角点 double**（64B）+ 3 lerp，0 虚调用 | 确证（density.h:537-548） |
| grid 数组多大？ | **5×49×5 = 1225 cells × 8B = 9800B**/实例/线程 | 确证（density.h:511/590） |
| noodle 读什么？ | **4 个 InterpolatedDF**（RangeChoice 顶 + interp#A/B/C/D），可到 **32 角点读** | 确证（noodle.json + density.h:497） |
| noodle 内层是 InterpolatedNoiseDF 吗？ | **否**（@anchor.idk 更正任务标注）：noodle 内层是 `InterpolatedDF`（cell grid 插值）包 `range_choice` 包 `noise`(DoublePerlinNoiseSampler)，**非** InterpolatedNoiseDF(old_blended_noise) | 确证（noodle.json） |
| noise 表（perm）是否共享 const？ | **是**，全局 `noiseSamplers` map 的 `shared_ptr<DoublePerlinNoiseSampler>` | 确证（density_builder.h:300-309,348） |
| 机器形状 | **12 物理核 / 24 逻辑（SMT）** | 确证（worldgen_api.cpp:1137-1158 + scout-map hw_threads=24） |
| 8 worker 是否触碰 SMT 争用？ | **否**（T=8 ≤ 12 物理核，各占独立物理核，无 core 共享; C2 频率归一化后仍 +14-25%） | 确证（worldgen_api.cpp:1290-1303 + scout-map C4） |
| 11× 是内存带宽（共享读）还是 SMT 执行争用？ | **两者都不是主导**（见 §6）：带宽 C7=1-2% DDR；SMT 对 T=8 不触发。**最一致 = 长串行依赖链 + 内存子系统 latency QoS** | 推断（需实验钉死） |
| 决定性可测量方法 | **per-thread perm 副本（测共享读）/ pin 物理核（测 SMT）/ interp-only-grid-hit 隔离（测链长相干）** | 见 §7 |

---

## 1. interp#1 trilinear 的内存读（确证）

`InterpolatedDF::sample`（density.h:497-558），grid 命中后（§4 的懒建已完成）：

- **每次命中读 8 个 grid 角点**（L537-548）：`g(dx,dy,dz)` 对 `slot.grid[((cy+dy)*GZ + (cz+dz))*GX + (cx+dx)]`，8 次：d000/d100/d010/d110/d001/d101/d011/d111。
- **每值 double（8B）** → 每点 8×8B = **64B** grid 读 + 3 次 lerp（L542-548）+ 0 虚调用。
- **grid 数组大小**：`GX = 16/4+1 = 5`（CELL_X=4）、`GY = height/CELL_Y+1`。height=minY..minY+height。overworld height=384 → GY = 384/8+1 = **49**；`GZ = 16/4+1 = 5`。→ **5×49×5 = 1225 cells**。
- **每 chunk 每线程每实例 grid 内存** = 1225 × 8B = **9800B**（~9.6KB）。
- **grid 是否 thread_local**：**是**。`tlSlots()` 返回 `static thread_local std::vector<Slot>`（L576-578），每线程独立。`Slot& slot = slots[cacheId]`，cacheId = 构造期 `nextId` 分配的**每实例** id（L490,573）。→ **grid 数据不跨线程共享，无跨线程写/读争用**。

**边界列复用缓存**（buildGrid L614-618）：`edgeCol` 长度 GY×GZ = 49×5 = 245 doubles = **1960B**/实例/线程，thread_local，供左邻 chunk 复用 gx=0 列。

### 1.1 interp#1 的 arg 是什么（buildGrid 的采样成本，非争用成本）

interp#1（finalDensity 顶 a 链，overworld.json:38-166）的 arg = 整棵 **sloped_cheese + entrances + spaghetti_2d + spaghetti_roughness + pillars** 怪物树（含 range_choice/add/mul/square/clamp/min/max + 多个 noise + 6 个 SplineDF）。**buildGrid 这棵树 1225 次**（每 grid 点 arg->sample 一次，L607）。这是**绝对耗时大头**，但 warm 实验（去 buildGrid）仍 10.10× → **buildGrid 不是争用来源**（本课题不重复）。

---

## 2. noodle 分支的内存读（确证）

noodle = `minecraft:overworld/caves/noodle`（finalDensity 的 min 之 b，overworld.json:167）。结构（noodle.json）：

```
range_choice(              // 顶
  input = interpolated(    // interp#A = wrapper 选通
            range_choice(input= y, min=-60, max=321,
                        in= noise(noodle), xz/y_scale=1,   // DoublePerlinNoiseSampler
                        out=-1))
  min=-1e6, max=0,
  in_range=64.0,
  out_range= add(
    interpolated(range_choice(y,-60,321, add(-0.075, mul(-0.025, noise(noodle_thickness))), out=0)),   // interp#B
    mul(1.5, max(abs(interpolated(range_choice(... noise(noodle_ridge_a) ...))),   // interp#C
                 abs(interpolated(range_choice(... noise(noodle_ridge_b) ...))))))  // interp#D
)
```

- **RangeChoice 顶 + 4 个 InterpolatedDF**（A=选通、B=厚度、C=ridge_a、D=ridge_b）——与任务描述一致。
- **每点 noodle 读**：interpA 命中（8 角点）；若 d 落 out_range（通常多数点），再 evalinguard interpB（8）+ interpC（8）+ interpD（8）= **最多 32 角点 / 256B** grid 读，来自 **thread_local** grid（各实例独立 Slot）。
- **@anchor.idk（更正任务标注）**：noodle 内层**不是** `InterpolatedNoiseDF`（density.h:383，old_blended_noise，8 interp octaves + 16 lower/upper octaves 的 OctavePerlinNoiseSampler 采样，极重）。noodle 用的是 `InterpolatedDF`（cell-grid 4×4×8 插值）包 `range_choice` 包 `noise`(DoublePerlinNoiseSampler)。InterpolatedNoiseDF 不在 finalDensity 路径。→ 记忆此更正：noodle 的「重读」来自 4 个 InterpolatedDF 的 grid + 每 grid 一次 buildGrid（每 chunk），**不是** old_blended_noise 的多 octave 采样。

### 2.2 noise 表（perm）读 — 出现在 buildGrid，不是逐点

- 每个 noodle noise（noodle/thickness/ridge_a/ridge_b）是 `DoublePerlinNoiseSampler`（amplitude=[1.0], firstOctave=-8，noise/*.json）。`sample`（noise.h:282-287）= firstSampler.sample + secondSampler.sample，各 1 octave（amplitudes.size()=1），每 octave = 1 `PerlinNoiseSampler::sample`。
- `PerlinNoiseSampler::sample`（noise.h:54-75）→ `sampleSection`（L82-110）：**8 次 `map()`（perm 读）+ 8 次 `grad()`**（读静态 GRADIENTS）。每 octave = 8 perm 读 + 8 grad 读。→ **每 DoublePerlinNoiseSampler.sample = 16 perm 读 + 16 grad 读**。
- 这些**只在 buildGrid 期间**发生（每 chunk 每 InterpolatedDF 1 次，1225 grid 点 × arg->sample）。**warm 去 buildGrid 后逐点不读 noise perm**（grid 已建）→ noise perm 共享读**不是逐点争用**。
- **perm 表**：`PerlinNoiseSampler::permutation` = `std::vector<uint8_t>(256)`（noise.h:32），**实例私有**（每个 sampler 各 1 份，256B）。`DoublePerlinNoiseSampler` 持 firstSampler+secondSampler = 2 个 OctavePerlinNoiseSampler = 2 个 perlin sampler。→ **共享对象**（全局 map 的 shared_ptr），所有线程读同一对象/同一 perm。
- **GRADIENTS** = `static constexpr int32_t[16][3]` = **192B**，全局共享 const（noise.h:17-22）。

---

## 3. 共享 vs 每线程表清单（确证）

| 数据 | 作用域 | 大小 | 每点读次（steady-state，warm/FLAT_TOP） | 是否共享争用候选 |
|---|---|---|---|---|
| **InterpolatedDF grid ×5**（interp#1 + noodle A/B/C/D） | **thread_local**（density.h:576-578） | 5×9800B = **49KB/线程** | 命中后 8 角点/实例 = 40 角点/点 | **否**（per-thread，无跨线程共享） |
| **edgeCol ×5**（interp#1 + noodle A-D） | thread_local | 5×1960B = 9.8KB/线程 | build 时写，命中后不读 | 否 |
| **noiseSamplers**（共享 const，noodle×4 + cave_layer/cave_cheese/spaghetti/pillars/etc ≈ 数十个） | **全局共享** | 每实例 ~256B perm×2 + 元数据 ≈ **~1KB**；合计 ~几十 KB | **仅 buildGrid**（每 chunk 每 InterpolatedDF 1 次）；warm 后逐点为 0 | **是**（共享 const，buildGrid 期） |
| **GRADIENTS**（192B） | 全局 constexpr | 192B | noise sample 内 16 次 | 是（共享 const，buildGrid 期） |
| **SplineDF 表**（nodes/locations/derivatives/subIdx，6 实例） | **全局共享**（finalDensity 树） | ~17KB/6 实例 | 依赖 spline 位置（a 链怪物树 buildGrid 期读，interp 命中后不读） | 是（共享 const） |
| **finalDensity DF 对象字段**（RangeChoice min/max/in/out、BinaryOperation a/b 等） | 全局共享 | ~几百 B | noodle 树逐点读（range_choice 判定 + 分支） | 是（共享 const，逐点小读） |
| **Cache2DDF/FlatCacheDF slots ×多个** | thread_local | Cache2D：16×(8+8+8)=384B；FlatCache：25×8=200B | 依依赖位置 | 否 |
| **g_curChunkX/Z** | thread_local | 2 int | 每次 FlatCache sample | 否 |

**要点**：
- 凡跨线程共享的都是**只读 const**（广播，无写 ping-pong / 无 invalidate / 无 coherence 流量——prior scout c-3 已确证）。**无任何跨线程写共享**。
- **真正逐点读的共享数据只有**：finalDensity 树的节点字段（noodle 的 range_choice 判定 + 分支虚调用）+ interp 对象自身字段（cacheId/minY/height，每点读一次）。**量极小**。
- **逐点读的"大"数据（grid 40 角点）全 thread_local**，不跨线程。

---

## 4. grid / 缓存分配（确证）

- **InterpolatedDF::buildGrid**（density.h:589-620）：`grid.assign((size_t)GX*GY*GZ, 0.0)`（L591）= 1225 × 8B = **9800B 分配 + 置零**，per chunk per instance。分配走 `std::vector<double>::assign`（malloc/free）。**thread_local Slot 复用同一 vector**（resize 语义：首访 resize，后续 assign 复用容量）→ 每 chunk 只 assign（不重复 malloc 大块，仅复用 buffer；除非 1225 后无增长）。
- **tlSlots 一次性扩容**：`if (slots.size() < instanceCount.load()) slots.resize(instanceCount.load())`（L504，原子读）—— per-sample 检查，只有实例数增大才 resize。thread_local。
- **edgeCol.resize(GY*GZ)**（L615）= 245×8B = 1960B，per chunk。
- **Cache2DDF**（L694-701）：`Slot{keys[16], values[16], stamps[16]}` = 16×(8+8+8) = 384B，thread_local，per chunk 复用（LRU 16 槽）。
- **FlatCacheDF**（L766-771）：`std::array<double,25> grid` = 200B，per chunk assign（`key` 切换即重建 25 点，L754）。
- **buildGrid 分配流量**：5 个 InterpolatedDF（interp#1 + noodle A-D）每 chunk 各 assign 9800B → **~49KB/chunk 分配 + 置零**。加上 each assign 也 `arg->sample()` 1225 次。这是 buildGrid 期内存流量，**warm 后为 0**（本课题不重复）。

---

## 5. SMT 线索（确证）

- **机器形状**：`physicalCoreCount()`（worldgen_api.cpp:1137-1158）用 `GetLogicalProcessorInformationEx(RelationProcessorCore)` 数**物理核**（SMT 不重复计）。bench 打印 **hw_threads=24**（scout-map L23），→ **12 物理核 / 24 逻辑（SMT）**。
- **线程默认数**：`wg_fill_blocks_multi` threads<=0 时 `threads = physicalCoreCount()`（L1294-1295）= **物理核数（12）**；`CORESWAP_THREADS` 显式覆盖优先（L1291-1292）。
- **亲和性/pinning**：**无**。全仓库无 `SetThreadAffinityMask`。pool 线程（CoreSwapPool, L1174-1206）用纯 `std::thread` 创建，**不设亲和**，OS 调度器自行放置。
- **8 worker 是否同核 SMT 争用**：**否**。T=8 ≤ 物理核数 12 → 8 工作线程落在 8 个不同物理核上（OS load-balancer 在 12 空闲核+8 线程下会摊开），**不触发 SMT 共享执行单元**。
- **C2 已排除 SMT/降频**：GHz 恒 2.99（无降频），频率归一化后仍 +14-25%（scout-map L99 + concurrent-density-probe-scout c-3/b）。
- **对照实验已设计**（concurrent-density-probe-scout c-3/b）：将 8 worker 分别绑到 8 不同物理核（SetThreadAffinity）vs 全部绑到 1 核，对比 spline 单次成本——若绑不同核仍放大 → 非调度/SMT。

---

## 6. 判断：带宽 vs SMT（@anchor.idk，需实验钉死）

**已排除（干净确认）**：
- **C7 内存带宽**：并发下 540MB/s = DDR 1-2%（per-chunk-concurrent-slow-mtrace C7）→ **DRAM 带宽远未饱和**。
- **C3 LLC 容量**：8T 活动集 10.4MB < 16.5MB LLC（无容量拐点）→ 不是 LLC 容量瓶颈。
- **C4 SMT/调度**：T=8 ≤ 12 物理核，各占独立物理核 → **SMT 执行争用不触发**；且 10× >> SMT 理论上限(~1.5×)。
- **共享读便宜**：noise 单点并发放大 **1.15×**（共享 perm 读）、spline-only **1.62×**（共享 spline 表读）——两者都低。**证明共享 const 读本身不是 10× 放大器**（若共享读是带宽瓶颈，noise/spline 早该 >> 2×）。

**所以**：
- **H1（内存带宽 / 共享读）**：**弱**。C7 1-2% DDR + noise 1.15× + spline 1.62× + grid 全 thread_local → 共享读带宽无法解释 10×。
- **H2（SMT 执行争用）**：**对 T=8 已排除**（12 物理核，无 core 共享）。10× 远超 SMT 能力。
- **最一致机制 = 长串行依赖链 + 内存子系统 latency QoS**（prior scout 残留方向「共享内存延迟 QoS（并发依赖链 miss 延迟放大）」）：
  - 每点串行链：interp#1 grid(8 读) → noodle 顶 range_choice 判定 → interpA(8) → [out_range] interpB/C/D(24) → 各级 range_choice/add/mul/abs/max 数学。**每级 load 结果喂下一级**（数据依赖）。
  - 8 线程同时灌入这些长链 → 共享内存子系统（L2 到 L3 / OOO 窗口 / load-store 队列）的**延迟 QoS** → 每级 load 延迟被**非线性放大** → 串行链延迟膨胀（~10×）。这发生在**无锁 + 读共享 const + 真并行**三条件下。
  - **注意归一化**：这**不是**「共享读带宽饱和」（C7 已否），也不是「写 ping-pong」（表全只读），而是**延迟（latency）而非吞吐（throughput）**被共享资源排队放大——8 线程共享 L3/互连的访存延迟带宽。

**一句话**：11× 既**不是**内存带宽（共享读）**也不**是 SMT 执行争用，而是**长串行依赖链的每级 load 延迟在并发访存排队下的非线性膨胀**（latency QoS）。grid 全 thread_local + 共享读全 const + 只读无写 —— 此三者与「无锁 + 读共享 + 真并行 + 单 chunk 膨胀 10×」完全自洽，排除了带宽/SMT/写乒乓，剩 latency QoS。

---

## 7. 可测量方法（区分/钉死机制）

> 测量纪律（AGENTS.md）：禁用 WG_PROFILE/WG_STAGETIMER（并发污染）；用 WG_PHASETICK（QPC 单次 + 无 profiling 污染）；分开「每 chunk 延迟（阶段耗时）」与「吞吐均值（wall/N）」。线程池正确性先核（notify 修复 0a781e1）。

### M1 — SMT 判别（pin 物理核）
- **操作**：给 pool 8 worker 设 `SetThreadAffinityMask`，各 pin 到**不同物理核**（12C/24T 上选 8 个偶数逻辑处理器 = 8 个物理核）；或者对比「全绑 1 核」vs「绑 8 不同核」。也可设 `CORESWAP_THREADS=8` 但用 Windows 任务管理器/`start /affinity` 确认 OS 摊开。
- **判据**：若绑 8 不同物理核后争用**大幅下降**（T8/T1 向 1.x）→ SMT/调度主导。若**基本不变**（仍 ~10×）→ **非 SMT**（与 C4/C2 预测一致，M1 大概率确认否定）。
- **成本**：低（改 pool 线程创建）。**无损**（只影响调度，不碰采样）。

### M2 — 共享读带宽判别（per-thread perm 副本）
- **操作**：把全局共享 `noiseSamplers` 的 perm 表做成**每线程副本**（pool worker 首访时 thread_local clone 一份 DoublePerlinNoiseSampler/perm；或最低成本——只 clone 4 个 noodle noise + cave 系 noise 的表）。采样线程读自己那份，消除跨核共享 const 读。
- **判据**：若争用**大幅下降** → 共享读带宽/latency QoS 主导（H1）。若**基本不变** → 共享读不是主因（与 C7 1-2% + noise 1.15× 预测一致，M2 大概率确认否定）。
- **成本**：中（改 build 期 + 采样期来源）。注意勿破坏逐位（副本 perm 完全相同）。

### M3 — 链长相干判别（interp-only grid-hit 隔离）【决定性】
- **操作**：在现有 `conc_sample_probe`（已有 density/spline/noise 模式，conc_sample_probe.cpp）加 **interp-only** 模式：对**已建好 grid** 的一个 InterpolatedDF 实例，只测 **grid 命中采样**（8 角点读 + 3 lerp，无 noodle/range_choice/怪物树），T=1 vs T=8 同批对照。
- **判据**：
  - 若 interp-only grid-hit 并发放大**低**（~1.5×，同 noise/spline）→ **争用不在 grid 读**，而在**长链 + noodle range_choice + interp 间算术 + 依赖深度** → latency-bound 依赖链（H3）。
  - 若 interp-only grid-hit 并发放大**高**（~10×）→ 争用在 **InterpolatedDF 机制本身**（thread_local vector 扩容检查 / cacheId 索引 / edgeCol 边界复用 / allocator），需另查。
- **成本**：低（复用 conc_sample_probe 加一模式 + 手动预建 grid）。**无损**。
- 这是把「争用来源」从「interp/noodle 整体」细化到「grid 读 vs 链长」的关键一刀；现有 warm(10.10)/FLAT_TOP(10.55) 只能排除 buildGrid/虚分派，**不能**区分「grid 读」vs「链长依赖」。

### M4 — MLP（并行内存）判别（补 M3 失败时）
- 若 M3 显示 grid 读低、链长主导 → 验证是否 **MLP 不足**：把 98304 点循环改成**并行处理多独立点**的链段（软件流水/批次重叠不同点的 load），看是否消除 10×。若消除 → latency-bound 依赖链（提升 MLP 有效）；若不消除 → 转向 I-cache / 其他。

**推荐执行序**：M3（最便宜、最判别）→ 依结果定 M1/M2。**M1/M2 大概率确认「非 SMT / 非共享带宽」**（与 C2/C4/C7 一致），M3 才指向真凶。

---

## 8. 边界与风险

1. **不破坏无损约束**：M1 只改调度，M2 副本 perm 相同，M3/M4 探针只读采样——皆不改变采样值（BK-001 零退化）。
2. **thread_local 网格结构不可破坏**（prior scout 强调）：InterpolatedDF/FlatCacheDF/Cache2DDF cacheId 别名 + thread_local Slot 是单点 0.4μs 快的结构来源，任何优化不得破坏。
3. **测量污染**：并发下禁用阶段计时探针；只信无探针整批 wall + 计数。
4. **本文件状态**：candidate（静态结构事实 = 确证；「11× 是 latency QoS 非带宽/SMT」= 推断，需 M3 实验钉死）。产出依据：density.h:219-238,383-619 / noise.h:17-22,54-110,114-238,244-288 / noodle.json / overworld.json:30-168 / density_builder.h:300-309,348 / worldgen_api.cpp:1130-1303,713-895 / scout-map C2-C7 / per-chunk-concurrent-slow-mtrace.md。
