# docs 修正草稿：07-block-pipeline.md（任务 A）+ 10-timewise-archive.md（任务 B）

> 产出：知识库 subagent（2026-08-16）| 状态：draft，待主会话应用 + 验证
> 用途：结论性 docs 落盘强制 subagent 产出草稿，主会话只应用。
> 约定：所有 old→new 为**精确原文**（锚点已核实唯一）；行号与主会话描述一致（07 篇 L74/L97/L109 已核实，无偏移）。
> 交叉引用：错误台账 `.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（MT1-MT7）；影响评估 `notify-bug-impact.md`；勘探 `scout-map.md`。

---

# 任务 A：`versions/1.20.1/docs/07-block-pipeline.md` 修正（3 处）

> 原则：**保留原表 + 加注释行，不删历史**（错误优先原则——历史数据标注作废而非删除）。
> 说明：L74/L109 的标注因表格内不能插行，用表格后紧邻 blockquote 实现（锚点 = 表内唯一行）。

## A1. L74「并行（8/22 线程）108-239ms/chunk 无加速反降」追加影响标注

**old**（L75 行，表内唯一锚点）：
```
| density 阶段 | 8.5-11.7ms/chunk | **670-1000ms/chunk** | ~100×；根因所在 |
```

**new**（保留原行，表后追加 blockquote）：
```
| density 阶段 | 8.5-11.7ms/chunk | **670-1000ms/chunk** | ~100×；根因所在 |

> ⚠️ **2026-08-16 影响标注（notify 丢失 bug，0a781e1 修复）**：本表「并行（8/22 线程）108-239ms/chunk 无加速反降」在 **notify 丢失 bug 活跃期（8/6-8/15）** 采集——[A] T>1 顺序跑下补建 worker 错过 notify 永久等待，**实际并行度=1（串行假象）**，「反降/无加速」幅度**不可信**（真实并行成本被 bug 伪影掩盖）。**H2 主因（FlatCache 单槽缓存 + buildGrid 角点越界 → rebuild 168×）为单线程精确统计（WG_SPLINEDEBUG），不受影响，保留成立**。影响面：`.investigations/worldgen-mt-scaling/notify-bug-impact.md`（§2 #3）+ `mt-scaling-errors.md`（MT1/MT2）；修复后重测数据见文末「2026-08-16 影响评估修正」。
```

## A2. L97 H3「thrashing ×16（mt 27,155ns vs t1 1,714ns）」追加标注

**old**（L97 编号项，原文）：
```
2. **放大器（H3 成立）**：thread_local 单槽缓存 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。调用量不变（4,703,488 ≈ 4,695,145），单次成本 ×16（多线程 27,155ns vs 单线程 1,714ns）；wall 多线程 8488ms > 单线程 6533ms（并行反而更慢）。
```

**new**（保留原编号项，其后追加 blockquote）：
```
2. **放大器（H3 成立）**：thread_local 单槽缓存 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。调用量不变（4,703,488 ≈ 4,695,145），单次成本 ×16（多线程 27,155ns vs 单线程 1,714ns）；wall 多线程 8488ms > 单线程 6533ms（并行反而更慢）。

> ⚠️ **2026-08-16 影响标注（H3 ×16 需重新定性）**：本条「mt 27,155ns（×16）」在 **notify 丢失 bug 活跃期**采集——实际并行度=1，「多线程环境」实为单 worker + 扩池开销，**×16 的「多线程侧」基数不可信**，H3 结论**需修复后重测重新定性**（待办：mt 侧 spline 单次成本重测；若 mt≈t1 则 H3 为伪结论/降级，详见 `mt-scaling-errors.md` MT2）。**H2 主因（rebuild 168×，单线程 WG_SPLINEDEBUG 精确统计）不受影响，保留成立**。
```

## A3. L109 表格「spline 单次 t1 1,714ns / mt 27,155ns」追加标注

**old**（L110 行，表内唯一锚点）：
```
| rebuild chunk 覆盖 | — | **112 chunk**（36 生成 + 76 邻居） | 递归蔓延实锤 |
```

**new**（保留原行，表后追加 blockquote）：
```
| rebuild chunk 覆盖 | — | **112 chunk**（36 生成 + 76 邻居） | 递归蔓延实锤 |

> ⚠️ **2026-08-16 影响标注**：本表「spline 单次 **t1 1,714ns / mt 27,155ns**（×16 H3 放大器）」的 **mt 侧数值在 notify bug 活跃期采集**（实际并行度=1），×16 需修复后重测重新定性（`mt-scaling-errors.md` MT2）；**t1 1,714ns 为单线程精确统计，不受影响**。H2 行（rebuild 36,252 = 168×）为单线程数据，保留成立。
```

## A4. 文末追加小节「2026-08-16 影响评估修正」（文件末尾 L555 之后）

**old**（文件最后一行）：
```
- 状态：**保持 draft**——spline 扁平化单线程 -24% 是真实收益，但「多线程膨胀」课题未闭合，需重新定位 InterpolatedDF::buildGrid 树遍历的 cache miss 构成后再评估 DFC。
```

**new**（原行 + 追加小节）：
```
- 状态：**保持 draft**——spline 扁平化单线程 -24% 是真实收益，但「多线程膨胀」课题未闭合，需重新定位 InterpolatedDF::buildGrid 树遍历的 cache miss 构成后再评估 DFC。

---

## 2026-08-16 影响评估修正：notify 丢失 bug 污染面 + 修复后重测 + clamp 发现

> 状态：draft（结论性落盘，待主会话应用）| 来源：`.investigations/worldgen-mt-scaling/`
> 完整错误台账（五段式 + 判错经验 + 速查表）：`mt-scaling-errors.md`（MT1-MT7）；影响评估：`notify-bug-impact.md`；勘探：`scout-map.md`；本修正对应上文 L74/L97/L109 三处 ⚠️ 标注。

### notify 丢失 bug（0a781e1 修复）影响面摘要

- **bug**：CoreSwapPool ensure() 锁内建 worker + run() 入队后 notify_all() 竞争 → 补建 worker 错过通知永久等待（tasks 空 + stop false）→ 只有老 worker 干活 = **串行假象**（[A] T>1 顺序跑实际并行度=1）。引入 252d988（8/6 20:11），修复 0a781e1（8/15 23:50），**活跃约 9 天**。
- **影响**：8/11-8/15 所有 [A] T>1 顺序跑数据作废（含本文件 L74「108-239ms 反降」、L97/L109「×16」）；**单线程数据全部不受影响**（T=1 无补建）；**H2 主因（rebuild 168×）保留成立**（单线程精确统计）。
- **触发边界**：只影响 [A] 批量模式（count=N 线程数递增 → 补建 worker 空闲）；[B]/实机 count=1 不补建不触发（其「无并行」是 clamp 问题，见下）。

### 修复后重测（64-chunk 8×8 前台，bench-notifyfix-8x8-20260816.txt）

```
[A] threads=  1   98.02 ms/chunk
[A] threads=  8   89.88 ms/chunk   （-8.3%：不再反降，轻度加速）
[A] threads= 12   90.39 ms/chunk
[A] threads= 22   97.76 ms/chunk
[A] threads=  0   96.30 ms/chunk
[B] workers=  1   86.80 ms/chunk   （[B] 段 120s cap 截断，不影响 [A] 结论）
```

- **结论**：notify 修复后 [A] T=8 不再反降（比 T=1 快 8%），但**远未到 8× 加速**——「每 chunk 并发下慢」仍存在（第二阶段课题：fillOneChunkCore 并发下每 chunk 耗时随并发增长，WG_MTTRACE 证明 8 worker 真并行但批间 525ms ≈ 8×65ms；fprintf stderr 锁竞争污染待无 fprintf 计数器复测）。
- ⚠️ 与 scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」**矛盾**（中间状态 C1 版/计时污染混测，单线程基差 +37%）——待同机同状态对照（notify-bug-impact.md §5 #1）。

### [B]/实机 M=1 结构性串行（threads clamp 发现，candidate）

- **发现**：`wg_fill_blocks_multi` L1189 `if (threads > count) threads = count;`（**66e05f5，8/5 引入**，池化 c792e9d 后语义失效）→ count=1 时 clamp 到 1 → ensure(1) → **池恒 1 worker**。
- **实机推论（代码链路铁证，待实机实跑对比）**：CppBridge.java L170-171（count=1 + THREADS）→ jni_bridge.cpp L93（透传）→ clamp → ensure(1)：**实机 mod 每 worker 调 count=1 时即使传 THREADS=12 也被 clamp 到 1 → 实机「多线程」可能从未真正并行**（结构性串行）。
- **与 notify bug 独立**：notify 只影响 [A] 批量；clamp 影响 [B]/实机 M=1。
- **状态/待办**：candidate（代码链路已闭环，唯一剩余验证 = 实机实跑对比）；修复待办（clamp 改 `if (threads > count && count > 1)` 或实机改批量调用）见 `mt-scaling-errors.md` MT3。
```

---

# 任务 B：`versions/1.20.1/docs/10-timewise-archive.md` 补记草稿

> 位置：文件末尾（当前最后条目 = 2026-08-15 深夜段 GPU 块级生成 I6-I8，其后追加）。
> 日期：提交落 8/15 23:50-23:59，排查/评估/台账在 8/16 → 条目日期用 2026-08-16，标题注明承接。
> 每条状态：✅ 修复闭环 / ↩️ 回滚 / ⚠️ 影响评估 / 🔍 待重测 / ⚡ candidate。

**old**（文件最后一行）：
```
- 正确方向（GPU 网格角点 + CPU 插值）未实施——需 fillOneChunkCore 密度阶段重构（「先 GPU 出网格 → CPU 插值」），工作量中等，待后续立项评估。
```

**new**（原行 + 追加条目）：
```
- 正确方向（GPU 网格角点 + CPU 插值）未实施——需 fillOneChunkCore 密度阶段重构（「先 GPU 出网格 → CPU 插值」），工作量中等，待后续立项评估。

---

## 2026-08-16：线程池 notify 丢失修复（0a781e1）+ C1 回滚（8966ba9）+ 影响评估 + clamp 发现 + MT 错误台账（✅ 修复闭环 / ↩️ 回滚 / 🔍 H3 待重测 / ⚡ clamp candidate 待实机验证）

> 承接 2026-08-15 深夜段（I6-I8）之后；提交时间 8/15 23:50-23:59，排查/评估/台账 8/16。完整五段式错误记录：`.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（MT1-MT7 + 判错经验 + 速查表）；影响评估：`notify-bug-impact.md`；勘探：`scout-map.md`；docs 影响标注：07-block-pipeline.md「2026-08-16 影响评估修正」。

### ✅ notify 丢失 bug 修复（0a781e1，8/15 23:50）

- **bug**：CoreSwapPool ensure()（L1057-1098）锁内建 worker + run() 入队后 notify_all()（L1125）竞争 → 补建 worker 错过通知永久等待（tasks 空 + stop false）→ 只有老 worker 干活 = **串行假象**（经典丢失唤醒）。引入 252d988（8/6 20:11 扩容支持），**活跃约 9 天**。
- **现象**：bench [A] T>1 顺序跑「反降 +19-29%」（T=1 73.23 / T=8 87.51 / T=12 89.92 / T=22 94.35 ms/chunk，bench-C2-20260815.txt）；WG_TASKTIME 实证补建 worker 全空闲（顺序跑 done_by 恒老 worker；**单独跑完美并行 = 池无增长时正确，bug 只在扩容路径暴露**）。
- **修复**：readyCount 原子（worker 进 wait 自增 / 拿任务自减）+ run() 入队前等 `readyCount >= workers.size()`（L1110-1118）。
- **影响**：8/11-8/15 所有 [A] T>1 顺序跑数据作废（串行假象）；**单线程数据、H2 主因（rebuild 168×）不受影响**（单线程精确统计）。
- **修复后验证**：64-chunk 8×8 前台重测（bench-notifyfix-8x8-20260816.txt）：[A] T=1 98.02 / T=8 89.88（**-8.3% 不再反降**）/ T=12 90.39 / T=22 97.76——收益仍被「每 chunk 并发下慢」吞掉（第二阶段课题）。

### ↩️ C1 thread_local 复用回滚（8966ba9，8/15 23:59）

- C1 候选验证（tl_col/tl_densityBuf 复用，消除每 chunk 1.2MB 堆分配/释放）→ **单线程慢 9%（71.68→77.93）+ MT 反降依旧** → 回滚；**C1 排除结论保留**（堆分配非 MT 反降主因，负面验证结果本身是资产）。

### ⚠️ 影响评估（8/16，notify-bug-impact.md）

- **H3「thrashing ×16」（mt 27,155ns vs t1 1,714ns）**：mt 侧数据在 bug 活跃期采集（实际并行度=1）→ **×16 需重新定性（🔍 待修复后重测）**；H2（rebuild 168×）保留。
- **WG_PROFILE/WG_STAGETIMER 计时污染揭穿**：density 460ms 伪影（真实 45ms）——独立污染源（探针自身开销），非 notify bug；探针已分离修复（cc93c50）。

### ⚡ threads clamp 发现（[B]/实机 M=1 结构性串行，candidate 待实机验证）

- `wg_fill_blocks_multi` L1189 `if (threads > count) threads = count;`（**66e05f5，8/5「方块层多线程并行」引入**；池化 c792e9d 后语义失效）→ count=1 时 clamp 到 1 → ensure(1) → **池恒 1 worker**。
- **实机链路铁证**：CppBridge.java L170-171（count=1 + THREADS）→ jni_bridge.cpp L93（`(int)count, (int)threads` 原样透传）→ L1189 clamp → L1193 ensure(1) → **实机 mod「多线程」可能从未真正并行**（结构性串行；与 notify bug 独立——notify 只影响 [A] 批量，clamp 影响 [B]/实机 M=1）。
- **修复待办**：clamp 改 `if (threads > count && count > 1)`（count=1 保留 THREADS）或实机改批量调用（未实施，记录待办）。

### ✅ MT 错误台账建立（mt-scaling-errors.md）

- **MT1** notify 丢失（✅ 已修复 0a781e1）| **MT2** H3 ×16 污染（🔍 待重测）| **MT3** clamp 结构性串行（🔍 待定性 + ⚡ candidate）| **MT4** 计时污染（✅ 已修复 cc93c50）| **MT5** C1 thread_local 退化（↩️ 已回滚 8966ba9）| **MT6** 修复后验证缺失（✅ 已补充 64-chunk 重测）| **MT7** runMtx「排队」未留痕（✅ 已核对留痕）+ 判错经验 9 条 + 速查表 11 行。
- **MT7 演进链核对（git log -S "runMtx" 实证）**：c792e9d 持久池（8/6）→ 252d988 扩容+shutdown（8/6）→ **e388ab4 runMtx 全局互斥（8/7，32 视距崩溃补丁 = 用户记忆的「排队」）** → **6e2c7ea per-run RunState 隔离取代 runMtx（8/11，批间真并行）**——「加了又去掉」只留一半痕（e388ab4 记了 09 篇，6e2c7ea 只改代码注释未更新文档），本台账已留痕；09 篇「排队」段待标历史。

### 🔍 遗留项（未立项 / 待复核）

- 🔍 H3 ×16 修复后重测（mt 侧 spline 单次成本；若 mt≈t1 则 H3 降级/删除）
- ⚡ 实机实跑对比（clamp 推论最后验证——实机多线程生成时 C++ 侧 worker 数 / 吞吐与单线程无差）
- 🔍 scout-map L110「修复后仍反降（T=1 71.40 / T=8 84.24）」vs 8x8 数据（T=1 98.02）矛盾（中间状态混测，单线程基差 +37% 待同机同状态对照）
- 🔍 「每 chunk 并发下慢 7.5 倍」真实性（WG_MTTRACE fprintf stderr 锁竞争污染）——需无 fprintf 计数器测量
- 07 篇 L74/L97/L109 影响标注 + 文末「2026-08-16 影响评估修正」小节（本批次落盘）
```

---

# 附：应用提示（供主会话）

1. **A1/A2/A3 用 edit 工具**：old_string 用表内/编号项原文（已核实唯一），new_string 为「原行 + 紧邻 blockquote」；A4/B 用 edit 在文件末尾行后追加（old_string = 文件最后一行，new_string = 原行 + 新内容）。
2. **锚点唯一性已核实**：A1 锚点（density 670-1000 行）count=1；A3 锚点（rebuild 112 chunk 行）count=1。
3. **行号无偏移**：L74/L97/L109 与主会话描述一致（已逐行核对原文）。
4. **可选附加标注（任务未要求，供主会话定夺）**：07 篇 **L67**（根因定论 blockquote 中「放大器（H3）= 多线程 thread_local thrashing（单次 ×16）」）与 **L81**（WG_PROFILE 表「spline 单次 20,598ns ~21×（08-11 多线程 thrashing 环境）」）也承载 H3 ×16 叙述——如需彻底一致，可各加一行 ⚠️ 指向本批次 A2 标注；不追加也不矛盾（A2 标注已声明 H3 待重测）。
5. **载体纪律**：本批次均为结论性 docs 修正 → 主会话应用后应跑一致性检查（grep「notify 丢失」「2026-08-16 影响评估修正」确认落位），再按 git 提交纪律提交。
