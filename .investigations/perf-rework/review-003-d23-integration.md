# review-003.md —— judge 审查意见：block_probe 集成立项（I1-I5）+ D23 修复（2026-08-15）

> 审查角色：core.judge subagent（45f0f1aa）。只出审查意见，不改 status；confirmed 只能由人类授予。
> 审查对象：I1 Vulkan runtime / I2 GpuDensityEngine + worldgen 接入 / I3 生成器产物纳入构建 / I4 零退化 / I5 吞吐 / D23 spline 边界修复（GPU+sim）。
> 三源交叉：产物/记录快照 + git HEAD/工作区 diff + 验证记录（cmd-output/）。

## 审查结论速览

**通过（证据完整可采信）**：D23 GPU 修复、I1（两副本 SHA256 一致、资源配对/错误检查达标）、I3（gpu-assets + CMake 条件编译）、PIMPL + density.h inline、crash handler DBG_PRINTEXCEPTION 修复。

**不推荐当前整体确认，需先处理 4 个 P1**（逐项跟踪见下）：

## P1 清单与处理状态

### P1-1 [已修 + 已验证] sim 修复声称不实——stage 6/7 完成路径仍覆盖父帧 stage
- **意见**：dbg_full_sim.py stage 6/7 完成路径（原 L289/302）仍保留 `stageStack[ps>>1]=2`——正是 D23 记录声称已修的「回填覆盖父帧 stage 跳 v1」。normal-range 父帧的 v0 子帧为边界嵌套帧时仍会算出错 Hermite；现有验证（顶层边界 + e2e 无边界域）均不覆盖。
- **处理**：已删除全部 5 处 `stageStack[ps>>1]=2`（grep 确认 0 处残留）；父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），回填只写值不覆盖。
- **验证**：`verify_p11_recursive.py`——显式栈 spline_eval_py vs 递归版 Spline.apply 参照（vanilla 语义直译），**全部 56 节点 × 8 坐标 × 2 sIdx = 1344 组合 0 mismatch**（覆盖边界触发域坐标 (784,160,-408)/(720,160,-432) 等）；sim eval_df(784,160,-408)=-0.458333333 ✓；sim vs e2e-A5 对拍 maxDiff=5.7e-9 ✓ 无回归。

### P1-2 [已修正表述 + 已补落盘] I4 范围与证据
- **意见**：块级生成恒走 CPU `finalDensity->sample`，GPU 从未参与块生成；「I4b GPU 路径 99.9994%」超出实际验证范围，且无 08-15 新 block_probe 输出落盘。
- **处理**：① i-integration-record.md I4b 修正为「GPU 引擎接入不破坏（块级恒走 CPU，GPU 仅构造 + wg_fill_density 批量接口）」，注明 GPU 自身逐位正确性由 e2e + domain probe 验证；② 补落盘：I4a CPU 路径 `cmd-output/blockprobe-I4a-cpu-20260815-191247.txt`（TOTAL 99.9994% 与记录一致）+ I4b GPU 构造 `cmd-output/blockprobe-I4b-gpu-*.txt`（WG_GPU_FILL=1 引擎构造 + 块级不破坏）。
- **遗留**：真实「GPU 参与块级生成」的逐位对照未做（当前架构块级恒 CPU，属设计而非缺陷）。✅ 已闭合

### P1-3 [已闭合] I5 证据缺失
- **意见**：throughput-I5 文件审查结束时仅 3 行，24-32x 与 1e-6~4e-6 无完整落盘可核。
- **处理**：已重跑 gpu_throughput_probe 1/4/16/64 chunks 落盘 cmd-output/throughput-I5-20260815-185504.txt（935 字节完整 4 档）。实测：**22.26x / 24.26x / 34.46x / 39.29x**，maxDiff **1.06e-6 / 2.86e-6 / 4.42e-6 / 8.26e-6**（D23 修复后正确性恢复；原 16/64 chunks 0.2-0.5 错值已消）。注：实测速度比原记录 24-32x 更高（64 chunks 39.3x）——原记录是 D23 修复前的数据。✅ 已闭合

### P1-4 [知识库 subagent 处理中] 时间线契约
- **意见**：10-timewise-archive.md 无 I1-I5/D23 条目；i-integration-record L40 声称「见 timewise 2026-08-15 段」与实际不符；D23 通用教训未进 discovered/。
- **处理**：i-integration-record 引用已标注待办；timewise 2026-08-15 段 + discovered/algorithm-fingerprints.md 发现 #14 由知识库 subagent 产出草稿后应用。

## P2 清单（不阻塞，记录）

- P2-1：sim 验证无落盘且判别力不足——已通过 verify_p11_recursive.py + check_sim_vs_e2e.py 补齐落盘（脚本在 .investigations/perf-rework/，结果落盘 cmd-output/sim-recursive-check-20260815.txt：1344 组合 0 mismatch + e2e 对拍 5.7e-9）。
- P2-2：shaderFloat64 未启用 + Vulkan 初始化失败 `exit(1)` 无 CPU fallback——GpuDensityEngine 构造失败路径（wg_create 已包 try/catch 返回 nullptr 走 CPU，但引擎内部 exit 需复核；shader 无 fp64 需求因 CPU 预拆分）。**遗留 NEXT_SESSION 待办 2**。
- P2-3：I2/I1 若干验证数字无独立落盘——I5 已重跑落盘；gpu_fill_probe 引擎验证输出未单独落盘（e2e-A5 同口径覆盖）；「90.9s/94.4s」为 pipeline 编译波动（67-102s 口径，a-plan L17 有说明）。
- P2-4：fill() 共享 buffer 无互斥——wg_fill_density 为 JNI 批量接口（单次调用），池 worker 并发路径（fillOneChunkCore）走 CPU finalDensity 不触 GPU；多线程同时调 wg_fill_density 的场景未验证（当前 mod 调用方单线程，**遗留记录**）。
- P2-5：worldgen 版 gpu_density_engine.h 为「去诊断方法（dumpValBuf）的精简版」，与 proto 版差异有意且合理——记录已注明「复制到 worldgen/src」为精简版。
- 其余 3 项 P2 / 3 项 P3 为代码风格/文档细节，不逐一记录。

## P3 清单（记录）

- P3-1：无 GPU/compute family/合适内存类型守卫（VKRT_CHECK 兜底 exit）——低危（目标机器恒有 GPU）。
- P3-2：dispatch() 每次新建 command pool + fence——正确但可复用优化，非必须。
- P3-3：anchor 门禁——**已跑 scan_cpp_anchors.py：✅ 所有 anchor 有效**（含 gpu_density_engine.cpp/.h 新文件）。

## 结论

candidate 状态可维持；**4 个 P1 全部闭合**：
- **P1-1** ✅：sim stage 6/7 完成路径 `stageStack[ps>>1]=2` 已删除（grep 0 残留）→ 递归版 Spline.apply 参照对拍 1344 组合 0 mismatch（verify_p11_recursive.py，落盘 cmd-output/sim-recursive-check-20260815.txt）
- **P1-2** ✅：I4b 表述修正（块级恒 CPU）+ I4a 落盘（blockprobe-I4a-cpu-20260815-191247.txt：99.9994%）+ I4b GPU 构造落盘（blockprobe-I4b-gpu-20260815-191310.txt：引擎构造成功 + 99.9994% 不破坏）
- **P1-3** ✅：I5 重跑落盘（throughput-I5-20260815-185504.txt：22.26x/24.26x/34.46x/39.29x，maxDiff 1e-6~8e-6）
- **P1-4** ✅：timewise 2026-08-15 晚段 + discovered #14 + gpu-accel-errors.md 判错经验 6 条 + 速查表补充行（subagent 草稿 + 主会话应用，三载体均确认在位）

P2/P3 记录在案（P2-2 shaderFloat64/exit(1) 与 P2-4 fill() 并发为遗留项，进 NEXT_SESSION 待办 2）；P3-3 anchor 门禁 ✅ 所有 anchor 有效。

**✅ confirmed（2026-08-15 用户拍板）**：I1-I5 集成 + D23 修复（GPU+sim）+ 知识库闭环全部确认；.artifacts 9 条升 confirmed。遗留 P2-2/P2-4 进 NEXT_SESSION 待办 2 跟踪。
