# 勘探报告：worldgen 多线程吞吐反降 — 候选机制地图（2026-08-15）

> 勘探角色：recode-scout（subagent 27898367）| 范围：SURFACE 模式 [A] 池并行批提交（threads=1/8/22 → 71.68/87.30/97.23 ms/chunk，+22-36% 单调变差）| 对照：C2ME-fabric CPU 同场景近线性。
> 主会话补充实测：WG_PROFILE 显示 density 1170ms 占 82%、spline 单次 92μs（单线程也慢，基线 1.7μs = 54 倍退化）——**spline 退化是独立于多线程反降的第二问题**。

## 0. 静态证据快照

- 每 chunk 工作结构（fillOneChunkCore L645-999）：densityBuf 98,304×double=786KB（L720）+ col 393KB（blocks.h L68）+ 输出 memcpy 393KB（L997）→ **每 chunk ~1.2MB 堆分配+释放 + 1.2MB 拷贝**
- 锁审计：beardifierMtx 1 次/chunk（µs 级）；regionCols/pendingCross 仅 FULL 模式（SURFACE bench 不触达）——**SURFACE 热路径无跨线程全局锁**
- thread_local：InterpolatedDF/Cache2DDF/FlatCacheDF 全 thread_local（density.h L577/711/777）——无跨线程缓存行共享
- 带宽粗算：全批 0.6-1.5GB，22t 下 ~0.5-1.4GB/s << DDR 带宽（除非每块树读 1.5KB 口径 → 38GB → 34GB/s 逼近上限）
- C2ME 对照差异：① DFC 编译直排方法（每点非树遍历）② Java DensityInterpolator 3-pass 批量 fill ③ Java TLAB 无全局堆锁

## 1. 候选机制地图（C1-C10）

### C1 — 每 chunk 大块堆分配/释放串行化（786KB+393KB malloc/free 抖动）★ 首要候选
- 机制：大 vector 超 MSVC LFH 上限 → 进程堆全局临界区；256 chunk × 4 次大块操作；线程数 ↑ → 全局堆锁等待 + HeapFree 触发 VirtualFree/TLB shootdown
- 与 C2ME 差异：Java TLAB bump 无锁；Java 不物化全 chunk densityBuf（逐列流式）
- 验证：per-worker TLS 复用缓冲 A/B；ETW HeapAlloc/Free 采样；mimalloc/tcmalloc 复测
- 预期：缓冲复用后 T=8/22 膨胀消失

### C2 — 全核睿频下降 + SMT 过订阅（12C/24T）★ 强解释单调性
- 机制：T=1 单核高睿频 → T=22 全核降频 + SMT 共享执行单元；每 chunk latency-bound
- 注意：C2ME 同机也吃同样惩罚 → 解释绝对膨胀，不解释对照差
- 验证：ETW 频率 trace 归一化；绑物理核实验
- 预期：频率归一化后残差归零

### C3 — LLC 容量饱和（22t 每线程 ~1.3MB 活动集挤 LLC）
- 机制：T=8 ~10MB 轻松容纳；T=22 ~26-29MB 顶到 LLC 边界 → DRAM 往返
- 验证：L3 miss 率随 T 跳变；小缓冲版复测；大页面复测
- 预期：L3 miss 在 T=16-22 跳变

### C4 — 任务调度顺序破坏 InterpolatedDF 边列复用 + 缓存亲和（3-5%）
- 机制：edgeCol 复用条件 = 同线程上一 chunk 为左邻；多线程 FIFO 跨距任务几乎不可能命中
- 验证：行主序 vs 随机洗牌对比；edgeCol 命中计数器；空间分片调度
- 预期：T=1 行主序 < 随机；edgeCol 命中率 T≥8 归零

### C5 — 热路径共享原子读取（每块 ~8 次 instanceCount seq_cst load）低量级
- 机制：每 chunk ~786k 次共享行 seq_cst load
- 验证：chunk 级缓存 instanceCount；padding
- 预期：<1%

### C6 — 共享只读大表（spline 表）跨核 L3 随机访问（叠加项）
- 机制：spline 扁平化表随机访问 + 22 核并发命中同一批表
- 验证：L2/L3 miss 分布；cacheline 对齐
- 预期：与 C3 强相关难剥离

### C7 — 内存带宽饱和（按数字反推：弱/排除为主因，保留 22t 叠加）
- 机制：粗算 0.6-1.5GB 不饱和；但每块树读 1.5KB 口径 → 38GB → 34GB/s 逼近上限
- 验证：perf stat 实测 DRAM 读字节；DFC 树扁平化后带宽下降
- 预期：分歧在「每块树读真实量」

### C8 — 互斥锁竞争（排除项，静态审计负结果）
- SURFACE 热路径无跨线程全局锁；负结果本身是知识
- FULL 模式另审（regionColsMtx 每 chunk 393KB + phase2 强制串行）

### C9 — 阶段串行化（density→aquifer→surface，排除为 MT 机制；记录结构差异）
- 全部 per-chunk 局部状态 → 不构成 MT 反降
- 记录：Java DensityInterpolator 3-pass 批量 fill vs C++ 每块 8 角点三线性 + 完整树虚调用——C++ 每块树遍历更贵是「膨胀基数」放大器

### C10 — 每 chunk 诊断/防御开销（弱，<0.1ms）

## 2. 最可能候选排序（fan-out 优先）

1. **C1（每 chunk 1.2MB 堆分配/释放 → 全局堆锁 + TLB）**：静态可见 + 随线程数放大 + C2ME 结构性差异，三性俱全；改造可测
2. **C2（睿频 + SMT）**：干净解释单调形态；但不能解释 C2ME 对照差——频率归一化剥离
3. **C3（LLC，22t）**：与 C1 同源（都是 1.2MB 缓冲后果）——建议缓冲复用改造同时观测

优先级：C2 频率归一化（低成本）→ C1+C3 联合 A/B（缓冲复用）→ C4 分片调度 + C7 带宽实测 → C5/C6 末位

## 4. 主会话验证记录（2026-08-15 晚段）

### C1 验证 — ❌ 排除（缓冲复用无效）
- 改造：densityBuf/col 改 thread_local 复用（chunk 间不重新分配）
- 结果：T=1 77.93 / T=8 90.23 / T=22 95.90（改造前 71.68/87.30/97.23）——**反降依旧**（复用后无改善）
- 正确性：8576 99.9994% 零退化 ✓（col 复用无残留问题）
- 结论：每 chunk 1.2MB 堆分配/释放**不是**多线程反降主因

### 单线程基线复核（WG_STAGETIMER，block_probe -threads 1）
- density ~47-57ms + aquifer ~32-61ms + surface ~8-24ms = 总 ~90-120ms/chunk
- 与 8/11 基线（43-46/26-48/6-10）**基本一致**——单线程无退化
- **spline 92μs 是 WG_PROFILE 计时污染**（真实 spline 正常；WG_PROFILE 每采样 steady_clock 计时 + 原子计数污染热点）
- 结论：多线程反降是**纯 MT 问题**（非 spline 退化），C2/C3/C4/C7 待验

### C2 验证（2026-08-15 晚段，T=1/8/12/22）— ❌ 排除睿频/SMT 为主因
- 实测：[A] T=1 73.23 / T=8 87.51 (+19%) / T=12 89.92 (+23%) / T=22 94.35 (+29%) ms/chunk
- **T=8 就 +19% 反降**（8 ≤ 12 物理核，无 SMT 惩罚）→ **不是睿频/SMT 主因**
- T=12→22 继续涨（+6%）= SMT 过订阅有叠加，但主因在 8 线程已显现
- 待验：C4（调度亲和）/ C3（LLC）/ C7（带宽）/ scout 未覆盖机制

### C3/C7 验证（worker d01823a8，2026-08-15）— ❌ 双排除
- **C7 排除（算术级）**：每 chunk DRAM ≈ 2.3MB（densityBuf 写 768KB + aquifer 读回 + col 写 + memcpy）→ T=22 ≈ 540MB/s vs DDR ~21GB/s = **1-2%**。scout 的「34GB/s 逼近上限」是口径错误（把表驻留热读重访字节误算为 DRAM 字节）
- **C3 对 8T 排除**：8T 活动集 10.4MB < 12C/24T 最低 LLC（16.5MB）；曲线平滑单调无容量拐点。只在 LLC≤30MB 且 T=22 时可能贡献（28.6MB 贴边），待 ②a 实测
- **活动集修正**：interpolated 树采样限制在 1225 角点/chunk/实例（4 实例=4900 次），真正逐块采样在 aquifer（barrier/fluidType ~20 万次/chunk）但全表驻留
- **真凶方向**：C2 睿频半部（全核降频从未实测，可解释 ~8%）+ 共享内存延迟 QoS（并发依赖链 miss 延迟放大）
- **决定性验证**：频率归一化（WG_CLOCKTRACE 已加，rdtsc/QPC 标定）——正在跑

### 频率归一化结果（2026-08-15 晚段）— ❌ C2 睿频完全排除
- [A] T=1 78.55 / T=8 89.68 / T=12 91.57 / T=22 98.40 ms/chunk，**GHz 全恒 2.99**（无降频）→ 归一化后反降依旧（+14~25%）

### 🎯 真凶锁定：线程池 notify 丢失 bug（2026-08-15 晚段，决定性）
- **WG_TASKTIME 实证**：bench {1,8,12,22} 顺序跑时，**T=1 先建 1 worker 并跑完 → T=8 补建 7 个 worker，但第一批 16 任务全部由老 worker（50952）单线程执行**（补建 worker 没干活）→ 批墙钟 ≈ 串行 → 多线程「反降」
- **T=8 单独跑时完美并行**（16 任务分散 8 线程）——证明池本身能并行，问题在「T=1 前置后补建」
- **根因**：`ensure` 在 mtx 锁内创建新 worker（L1053-1075）→ 新 worker 启动后要拿 mtx 进 `cvTask.wait`，但 ensure 持锁；`run()` 入队后 `notify_all()`（L1105）时**新 worker 可能还没进 wait** → notify 丢失 → 新 worker 永久等待（tasks 空 + stop false）→ 只有老 worker 干活
- **修复**：`readyCount` 原子（worker 进 wait 自增/拿任务自减）+ `run()` 入队前等 `readyCount >= workers.size()`
- **影响**：修复后多线程应从「反降」变「加速」（T=8 批墙钟 ≈ 串行/8）——验证中（bench-notifyfix 后台跑）
- **教训**：C1-C7 全排查（堆/睿频/LLC/亲和/带宽）都是表象——真凶是池实现的经典「notify 丢失」（worker 就绪与通知竞争）。**多线程性能问题先查线程池实现正确性，再查内存/调度**

### notify 修复验证（2026-08-15 晚段）— ⚠️ 修复未解决 [A] 反降，但揭穿计时污染
- 修复后 [A]：T=1 71.40 / T=8 84.24 / T=12 86.55 / T=22 88.44——**仍反降**（notify 修复不充分或非主因）
- **[B] 模式持平**：T=1 84.34 / 8 83.02 / 12 80.00 / 22 85.38——**B 模式基本无反降**（各 worker count=1 独立）
- **计时污染揭穿**：WG_STAGETIMER 下 density 458-471ms（真实 45ms）——**WG_STAGETIMER/WG_PROFILE 的每采样 steady_clock + 原子计数让 density 慢 10 倍**！之前所有「spline 92μs / density 460ms」全是计时开销，真实正常

### 🎯 第二阶段：fillOneChunkCore 内部串行化线索（2026-08-15 晚段，进行中）
- **WG_TASKTIME 时间戳**：T=8 64 任务，**每批 8 任务同时完成但批次间隔 525ms = 8×65ms（串行）**——8 worker 分散执行（不同 done_by），**但每批 8 个 chunk 是串行完成的（525ms ≈ 8×65ms 而非并行 65ms）**
- **含义**：池并行正常（worker 各执行），但 **fillOneChunkCore 内部有全局串行点**（8 worker 并行进入但实际一个接一个完成）
- **待定位**：fillOneChunkCore 入口/出口加时间戳，看 8 并行 chunk 进出时间是否重叠（重叠=真并行，不重叠=被串行化）——方向 1（进行中）
- **方向 2 备选**：C2ME DFC 编译直排（消除每块虚调用链）可能同时解决单线程慢 + 多线程串行

### 方向 1 结果（2026-08-15 晚段，WG_MTTRACE）— ✅ fillOneChunkCore 真并行，无串行化
- **MT trace 实证**：T=8 64 任务，每批 8 chunk **enter 相同（620061）+ exit 相近（620591-595）**——**8 worker 真并行**，每批 530ms = 1 个 chunk 耗时（8 并行共享），非串行
- **排除「fillOneChunkCore 内部串行化」**：每批 8 chunk 同时进出，无全局锁等待
- **新线索**：并发下每 chunk dur 530ms vs 单线程 71ms（7.5 倍）——**但 WG_MTTRACE 的 fprintf 到 stderr 是同步的，8 线程同时 fprintf 有 stderr 锁竞争**，可能污染 dur
- **需无 fprintf 测量**（计数器而非打印）确认「并发下每 chunk 真实耗时」——若真 7.5 倍，指向共享资源竞争（但 C7 已排除带宽 1-2%）；若 fprintf 干扰，则多线程实际正常

## 3. 额外线索

1. C2ME DFC = 发现 #11 预告的「整个 DF 树扁平化」未做项——C++ 每块树遍历是膨胀基数
2. Java TLAB vs C++ 全局堆 = C1 对照证据
3. C2ME 调度器有线程亲和设计（ThreadLocalWorldGenSchedulingState）；CoreSwap 无亲和 FIFO
4. **bench 口径待确认**：71.68/87.30/97.23 若为 [A] med/N（批墙钟÷256）则 8t 批墙钟 22.3s > 1t 的 18.35s = **池并行绝对反降**；需读输出原件确认
5. FULL 模式 phase2 强制 threads=1（L1180）——扩展性天花板
6. 机器形状：bench 打印 hw_threads；physicalCoreCount 存在（L1015-1036）暗示 SMT
