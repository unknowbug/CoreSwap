# Phase 0 架构：aquifer est 冷路径优化（GPU vs CPU 双路线）（260908-02，draft 待 judge + 用户拍板）

> 输入链：scout-a/b/c + convergence + addendum-denominator + worker-aquifer-input-face + worker-c2me-aquifer-emitter（.investigations/gpu-reentry-260908-02/）
> 范围声明：verdict-260908-01 confirmed（noise/density 现引擎形态负面）不受本架构触碰；本课题 = est 冷路径子管线（其留白③内）。

## 一、问题定义（数据钉死后的形态）

- aquifer 成本分场景：**暖区吞吐 ~1.0ms/chunk（T=10 差分，GPU 无利可图）；冷态单 chunk ~35-37ms（est 冷扫描 ~15.4 为主，每列 34 次全价 initial_density 采样，重叶 = old_blended 无缓存）**。
- 真实痛点 = **冷路径**：新区域首遍生成/世界加载卡顿。优化目标：压冷态 est，不动暖区稳态（已近零）。
- 共享结构：`initial_density_without_jaggedness` 同时服务 aquifer est + surface rules；old_blended（fp64 需求，A2 台账）是共同重叶。

## 二、路线 A：GPU initial_density 专用 kernel（C2ME 形态）

- **形态**：手写专项化直排 kernel（非解释器，绕开 D27 死因）；est 预填充 = 固定列网格 dispatch（~256 列/chunk，坐标-only 输入）；C2ME 源码级先例证明 0 上传字节/点 + device 内流转可行（worker-c2me 源码钉死）。
- **范围（最小切片）**：只做 `initial_density_without_jaggedness` 一棵树的 kernel（非整 final_density 树）；服务面 est（15.4ms 冷）+ surface（5.5-6.7）≈ 21ms/chunk 冷态。
- **前置 micro-bench（gate，非承诺）**：old_blended fp64 单 kernel 微测——A2 台账：/o 放大要求 35 位精度 → 必须 fp64；GeForce fp64 = 1/32 吞吐（C2ME 接受此代价但无公开收益数字）。**gate 对比式（judge J3 修订）：kernel 采样单价 + dispatch/readback + 与 CPU 线程并存的 mutex 互斥成本（D24 P2-4 实锤项，不可漏）三者端到端 × 34×256 列 vs CPU 冷价 60µs/列**（预计 1-2 天含 SPIR-V 编译调试，G 系编译时间风险按 noodle 基准 44 函数×1.6KB 控制）。**且 B 落地后 A 的 gate 按新分母重算**（D27 教训 2：CPU 基线变快则 GPU 门槛同步抬高，judge J4）。
- **工程量**：中-大（新 kernel 模块 + 验证链复用既有 gpu_ffi 探针基建）。
- **明确不做**：apply 链 GPU 化（暖区已 1ms，无可图）；整树 final_density（= B1 新立项，另案）。

## 三、路线 B：CPU 冷路径优化（并列评估）

- **B-1 old_blended 结果记忆化**：pure function，无宿主缓存（冷态重复采样同列/邻列重复坐标）；bounded LRU（跨 chunk Arc 共享，est L2 同款机制先例）→ est + density + surface 三处共同受益。工程量小，风险低（值不变，只加速）。**命中率未知，需计数探针定上界；探针 MUST 覆盖连片预生成的 L2 命中率爬升带（冷/暖混合中间态，judge J1）**。
- **B-2 est L2 扩容/持久化**：现 FIFO 131072 ≈ 4370 chunk 上限（evictions=0 在 256 chunk 内）；大 region 预生成/重复访问场景扩容或落盘持久化。工程量小。
- **B-3 预热线程**：后台预计算 est 列（idle 时），把冷价移出关键路径。工程量中（线程/生命周期管理），语义零风险。
- **收益上界**：est 冷 15.4 + 冷 miss 6-8 ≈ 21-23ms/chunk 冷态；B-1 命中率取决于坐标重复度（**需一轮计数探针定命中上界**，便宜）。
- **纪律**：est 扫描循环本身不可改（逐位对齐铁律）——只允许缓存/记忆化/预取类保值优化。

## 四、建议（待用户拍板）

1. **先走 B-1 前置计数探针**（最便宜，1 轮）+ **A 前置 fp64 微测**（gate）——两个小探针出数后双路线同页对比再定投入。
2. 若 fp64 微测显示 GPU 无优势（大概率风险项）→ 纯 CPU 路线收口。
3. 两路线不互斥：B 系列即使 A 立项也值得做（工程量小）。

## 五、judge 预置与遗留待验项

- 本架构文档 = 重大方向提案 → judge MUST 审查后才交用户 HOOK（已审：review-phase0-260908-02.md PASS-with-conditions，J1-J5 已应用）。
- 后续各探针结论 candidate 级 SHOULD judge；confirmed 留用户。
- 随行待验项（judge J2/J5 继承）：① 冷路径暴露面量化——真实使用中冷 chunk 出现频率（首遍预生成/玩家探索边界比例）未测，影响双路线收益权重；② est 分解残差 27-29 ≠ 35-37 两行口径不同源未合并；③ 本 session 实测显示 `WG_EST_L2` 清空 env 后仍 l2=true（疑似默认开），须 api.rs 直读确认默认值方向（workflow #53 家族）。
