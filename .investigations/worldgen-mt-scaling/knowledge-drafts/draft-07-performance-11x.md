# 草稿 · 主题篇 07 追加小节 —— production density 并发 11× 争用归因（latency QoS）

> **用途**：追加到 `versions/1.20.1/docs/07-block-pipeline.md`（块级流水线与性能，= 性能/多线程主题篇）末尾的独立小节（主会话应用）。
> **状态**：排除链 = **production 模型确证级**（同探针 conc_density_probe）；latency QoS 归因 = **candidate/推断**（@anchor.idk，需 M3 探针干净验证；M3 探针因自身 bug 未完成直接验证）。
> 主过程日志：`.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`。

---

## （追加小节）production density 并发 11× 争用归因 = 长串行依赖链 + 内存子系统 latency QoS

> 承接本主题既有多线程性能记录：2026-08-12 根因（H2 主因 FlatCache 单槽缓存 + buildGrid 角点越界 → 嵌套递归蔓延）/ H3 放大器（thread_local thrashing）为**旧课题**；本节是 **density 11×（SplineDF/InterpolatedDF 采样在并发下的每 chunk 延迟膨胀）** 的归因。DFC 已在 DFC CPU 移植失败定论中作废（600× 慢，净作用为负）；locFn 连续化（Plan A）在 A/B 中证伪（放大比持平，非主导）。

### 一、现象（核心数据）

production density 单 chunk 延迟随线程数线性暴涨（conc_density_probe + WG_PHASETICK，12 固定 chunk，median density）：

| T | density 耗时 | 相对 T=1 |
|---|---|---|
| 1 | 37.83~39.31ms | 1× |
| 2 | 74.01ms | 2× |
| 4 | 174.33ms | 4.6× |
| 8 | 331.04~346.26ms | **8.4×~9×（单 chunk 9.2× / density 11×）** |

**关键区分（AGENTS.md 早已警告，本课题反复犯）**：
- **每 chunk 延迟** = density 阶段耗时 = 42.69 → 391.41ms（**9.2×**）——真实暴涨；
- **整批吞吐** = wall/chunk = 69 → 73ms/chunk——**几乎不变**；
- 多线程下 **wall/N 是吞吐均值，不是每 chunk 耗时**——吞吐正常 ≠ 并发无问题。
- **单点 0.4μs·快**（thread_local grid 懒建 + 每点纯 trilinear），并发才是问题。

### 二、排除链（production 模型确证级，同一探针 conc_density_probe / 同一 wg_worker pool / 只差一项改动）

| 试验 | 改动 | 放大比 | 结论 |
|---|---|---|---|
| BASE | — | 10.32× | 基线 |
| SERIAL | spline.locFn 存储连续化 | 10.25× | ❌ 存储非争用 |
| NOSPLIT | spline 递归→显式栈 | 9.9× | ❌ 递归非争用 |
| DEVIRT | spline.locFn 虚分派 devirtualize | 10.05× | ❌ locFn 虚分派非争用 |
| spline-only | 绕 wrapper 直采 spline（WG_SPLINE_FILL） | 1.62× | spline 无碍（占时间仅 9%） |
| warm | 预建 grid 排除 buildGrid | 10.10× | ❌ buildGrid 无碍 |
| **WG_FLAT_TOP** | 去 min/squeeze/mul 虚分派（4→2，**block_probe SHA256 逐位一致**） | 10.55× | ❌ **虚分派数无碍** |

**排除清单（一行式）**：11× 争用 **不是** 存储（SERIAL）、**不是** 递归（NOSPLIT）、**不是** locFn 虚分派（DEVIRT）、**不是** buildGrid 深链（warm）、**不是** 顶层 min/squeeze/mul 虚分派（WG_FLAT_TOP），**不是** spline 本身（spline-only 1.62×），**不是** 内存带宽（C7 DDR 1-2% 未饱和）、**不是** SMT（T=8 ≤ 12 物理核，各占独立核）、**不是** 写乒乓（共享全 const 只读）。

⇒ **11× 争用 = interp/noodle 采样内部**（内存访问模式）。

### 三、latency QoS 机制（candidate/推断）

scout 访存分析（interp-memory-access.md，dcf85758）确证：interp grid 全 **thread_local**（density.h:576-578，跨线程独立不共享）；跨线程共享全为**只读 const**（noiseSamplers/SplineDF 表 17KB/GRADIENTS 192B/finalDensity 节点字段），**无写共享/ping-pong**；C7 带宽 DDR 1-2% 未饱和；C4/C2 SMT 对 T=8 不触发（12 物理核无 core 共享）；noise 1.15×/spline 1.62× 证明共享读不是 10× 放大器。

⇒ **最一致机制 = 长串行依赖链 + 内存子系统 latency QoS**：
- 每点串行链：interp#1 grid（8 读）→ noodle 顶 range_choice 判定 → interpA（8）→ [out_range] interpB/C/D（24）→ 各级 range_choice/add/mul/abs/max 数学。**每级 load 结果喂下一级**（数据依赖）。
- 8 线程同时灌入这些长链 → 共享内存子系统（L2 到 L3 / OOO 窗口 / load-store 队列）的**延迟 QoS** → 每级 load 延迟被**非线性放大** → 串行链延迟膨胀（~10×）。这发生在**无锁 + 读共享 const + 真并行**三条件下。
- **是延迟（latency）而非吞吐（throughput）**被共享资源排队放大——不是共享读带宽饱和（C7 已否），不是写 ping-pong（表全只读）。

### 四、修复方向（latency QoS 下）

**提升 MLP**（打破长依赖链形态：并行多独立点 / 软件流水 / 分块减少每级数据依赖），**不是**减虚调用/存储/递归（已排除）。

**⚠️ 边界（关键）**：DFC 式全扁平直排能在 CPU 上消除并发放大（11×→1.3×），但**每点绝对成本 238μs → 整 chunk 600× 慢 → 净作用为负，已作废**。故「提升 MLP」必须在 **production 自身形态**上做（保留单点 0.4μs 快），**不是算法重写**（DFC 教训）。

### 五、待验证（M3）

执行 M3（interp-only grid-hit 隔离）验证「长链 latency QoS」。**M3 探针（wg_sample_interp）目前因自身 bug 未完成干净验证**（hit 慢 850× vs production 0.34μs/点，探针自身需 perf 定位：thread_local slots resize/坐标跨 cell/每次 buildGrid）。latency QoS 归因基于排除链 + 结构自洽（**间接**），**待 M3 或等价干净测量直接证实**。

- 若 M3 低（争用不在 grid 读）→ 指向长链依赖 → MLP 方向。
- 若 M3 高（争用在 InterpolatedDF 机制）→ 另查 thread_local vector / cacheId 索引 / allocator。

### 六、引用文件

- `.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`（主过程日志）
- `.investigations/worldgen-mt-scaling/wrapper-chain-measurement.md`（§6-8：spline-only / warm / WG_FLAT_TOP + 对拍）
- `.investigations/worldgen-mt-scaling/interp-memory-access.md`（scout 访存分析）
- `.investigations/worldgen-mt-scaling/wrapper-buildgrid-structure.md` / `topwrapper-sample-logic.md`（scout 结构）
- `.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（历史 11× 机制 + DFC 作废）
- `.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（错误台账本体；新增 ①-⑥ 见 `knowledge-drafts/draft-mt-errors-11x.md`）
