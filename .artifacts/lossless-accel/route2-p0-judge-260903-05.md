# route2 P0 交接假设推翻结论 — core.judge 审查意见（260903-05）

审查对象：`.investigations/lossless-accel/route2-260903-05.md` P0 节（交接假设廉价独立验证 → 推翻结论）。
审查基线（三源核对）：① 上述验证记录 ② 三份被引源文件原文逐行核对（非转述）③ 本审查的独立源码推理。核对时点 260903-05，仅读源码与记录，未改任何 status。

## 审查点 1：证据链是否支持「fill 完全同步串行、无跨线程 GPU 流水重叠」——通过

逐行核对结论（独立验证，与记录转述一致）：

1. **lock_guard 作用域**：`gpu_density_engine.cpp:106-118`，`fill()` 首行 `std::lock_guard lk(im.fillMtx)`（:108），函数体全部语句（ensureBuffers、CPU split 循环 :111-113、upload×2、dispatch、readback）都在锁作用域内直至函数返回。✅ 全程持锁。
2. **fence 等待语义**：`vulkan_runtime.h:119-120`——`vkQueueSubmit(..., fence)` 后**同一调用内** `vkWaitForFences(m_device, 1, &fence, VK_TRUE, UINT64_MAX)`。`VK_TRUE` = waitAll、`UINT64_MAX` = 无限超时：host 线程阻塞直到 GPU 完成本次 dispatch，才能返回。✅ 同步阻塞，无异步返回路径。
3. **readback host 阻塞性**：`vulkan_runtime.h:124-126` = `vkMapMemory` + `memcpy` + unmap；buffer 全部为 host-visible + HOST_COHERENT（createBuffer :69-79 选择条件显式要求两标志）。coherent 内存 + fence 已等待 = GPU 写入对 host 立即可见，readback 是纯 memcpy，不引入额外同步但也不需要。✅。
4. **无隐藏第二提交点/第二队列**：VkRuntime 单 queue（:43-46，queueCount=1）、单 fence 单 command buffer per dispatch、无 timeline semaphore / event / 二次 submit。`dispatch()` 每次调用即「submit + 等完 + 销毁」，命令级也无法跨调用重叠。
5. **附加证据（记录未充分利用）**：`gpu_ffi.cpp:36` 调的是 `GpuDensityEngine::fill()`，**fill 内部本就有 fillMtx**——即「互斥策略在 Rust 侧（shim 不加锁）」的注释（gpu_ffi.cpp:7-8）只对并发正确性而言成立；即使 Rust 侧不持 Mutex，C++ fillMtx 也已把同一 handle 的所有 fill 串行化。这使推翻结论比记录所述**更硬**：任何通过该 handle 的路径都不存在并发 fill。
6. upload 用 host-visible coherent 内存写 + 下一次 submit（:81-83），无独立传输队列，不构成与 compute 的重叠通道。

**意见：证据链完整、指向唯一，推翻结论「fill 完全同步串行、无跨线程 GPU 流水重叠」在静态层面成立。建议升 candidate。**

## 审查点 2：0.61× 残留异常的两候选是否穷尽——不穷尽（CONCERN，中等）

记录给出 ①探针计时口径混杂 ②驱动 fence 等待行为与其他段重叠。二者合理但**未穷尽**，至少还有以下互斥候选应在 P4 复测时一并纳入：

- **③ 锁竞争/调度开销本身**：双线程在 fillMtx 上互斥，持锁方在锁内还做 CPU split（纯 CPU 段 :111-113），等待方空转唤醒、上下文切换、cache/TLB 污染——两线程 CPU 侧互相干扰即可产生 <1× 吞吐，与 GPU 无关。这与本仓库已有铁律同族（线程池唤醒竞争 MT 案例、SplineDF cache 污染）。
- **④ 计时把 GPU 等待排除在外 / 批粒度不一致**：并行路径每线程 fill 批量 n 可能小于串行基准的批量，dispatch 固定开销（每次 dispatch 新建 command pool + fence + alloc，`dispatch()` :106-121——**每次调用都重建命令资源**，固定开销不小）在小批量下占比放大。若 0.61× 的两侧对比批量/口径不同（§9.7 验证可比性），数值本身不可比。
- **⑤ 测量探针污染（项目已知系统性风险）**：AGENTS.md 测量铁律明确「WG_* 计时探针并发下污染测量，并行性能只能信无探针整批 wall + 调用次数计数」。若 0.61× 来自带探针的测量，数值不可信，无需机制解释。

**建议**：P4 廉价复测时按 fan-out 纪律把 ≥3 个互斥候选（③④⑤ 至少其一 + 原①②）并列验证，优先用「无探针整批 wall + fill 调用次数计数」重测，先确认 0.61× 是否真实存在再谈机制。记录中「驱动 fence 行为与其他段重叠」表述含糊（fence 等待是 host 阻塞，何来「重叠」段？建议改写为明确机制表述，否则该候选不可检验）。

## 审查点 3：接线设计决策「GPU 提交单线程化」——稳妥，支持，附两点边界

- 决策与推翻结论逻辑自洽：fill 全串行 ⇒ 双线程共享 handle 无 GPU 并行收益 ⇒ 不投共享-handle 并发优化。且该决策对 0.61× 异常的最终解释**鲁棒**：无论异常归因于哪种候选，「共享 handle 无并行收益」都成立。✅ 支持。
- **边界 1**：「每线程独立 handle 后置评估」**不应被过早丢弃**——这是唯一可能拿到真 GPU 并行的路线（各 handle 独立 buffer/queue，绕开 fillMtx 与驱动单队列），代价是 create ~75s × N（handle 缓存可摊薄）+ 显存翻倍。建议在后置评估中保留显式触发条件（如单线程 GPU 吞吐仍是瓶颈时）。
- **边界 2**：`dispatch()` 每次调用重建 command pool/fence/alloc（vulkan_runtime.h:106-121）是比线程模型更大的低垂果实；单线程化后若吞吐不达标，应先做 dispatch 资源复用/批量合并（一次 submit 多 chunk），而非线程优化。建议接线设计把这一项列入待办。

## 审查点 4：验证分层声明诚实性——基本诚实，缺一行显式降级声明（CONCERN，低）

- 记录明确写「验证方式：静态源码核对」，且结论标注 draft / candidate 建议、0.61× 标「待 P4 廉价复测」——未冒充数据层证据，诚实。✅
- 但按 Anchorlaw §9 / 项目验证分层，本结论属 **Degraded（静态审查）级**，记录未出现「降级声明」字样，也未声明后续补数据层（trace/probe）验证的计划。**建议补一行显式降级声明**：「本结论 = Degraded 静态审查级，升 confirmed 前须有数据层验证（fence 等待实测计时 / 双线程 fill 时间线 trace）」。这与候选授予（candidate）不冲突——candidate 可先授。

## CONCERN 清单

| # | 内容 | 严重度 |
|---|------|--------|
| C1 | 0.61× 残留异常两候选不穷尽，缺锁竞争/批粒度口径/探针污染类互斥候选；P4 复测应 fan-out ≥3 候选 | 中 |
| C2 | 「驱动 fence 行为与其他段重叠」表述机制不明确、不可检验，建议改写或删除 | 低 |
| C3 | 缺显式 Degraded 降级声明与「confirmed 前补数据层验证」承诺 | 低 |
| C4 | 「每线程独立 handle」路线的保留条件未显式化，有被沉默丢弃的风险 | 低 |

## 结论建议

- 对**推翻结论**（fill 完全同步串行）：**支持升 candidate**（静态证据链完整且互证；confirmed 留待数据层验证 + 人类拍板）。
- 对**接线设计决策**（GPU 提交单线程化）：**支持**，附 C4 边界 + dispatch 资源复用建议。
- **0.61× 后续动作**：P4 用无探针整批 wall + 调用计数重测（先验真伪）→ 若仍 <1× 再 fan-out ≥3 互斥候选并行定位。
- 本审查为意见，不改任何 status；未发现噪声卡/retry cap 相关冲突点（本次为静态核对，不消耗数据层 retry 计数）。
