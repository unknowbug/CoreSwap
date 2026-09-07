# 数据补齐纪要（260908-02 追加，draft）——多线程分母实测 + C2ME 源码级调研

> 对 convergence-main-260908-02.md §二/§三 的修订补丁。两块新证据后，B2 的 J6 核算**必须分场景**。

## A. aquifer 多线程分母四臂实测（estopt_mt_bench 复用，256 chunks 吞吐口径，seed 同，L2 状态见注）

| 臂 | T=1 | T=10 |
|---|---|---|
| WITH aquifer | 63.91 ms/chunk | 7.43 ms/chunk |
| NO aquifer | 54.21 | 6.41 |
| **aquifer 差分** | **9.70** | **1.02** |

- ⚠️ 口径注记 1：运行显示 `l2=true`——`$env:WG_EST_L2` 已清空仍为 true（**默认开**，workflow #53「env 默认值当公理」家族的新实例，待 api.rs 直读确认默认值方向）；且 256 chunk 连片 + est L2 命中 ~90% = **暖区口径**。
- ⚠️ 口径注记 2：qpd1 的 35-37ms 是**单 chunk 冷态延迟**口径；本表 T=10 差分 1.02ms 是**暖区吞吐摊销**口径。两者不矛盾（9.70/10 线程 ≈ 0.97 ≈ 1.02，aquifer 并行扩展良好，无严重串行瓶颈）。
- **J6 修订结论：分母分场景——暖区生产吞吐 ~1.0ms/chunk（GPU 门槛回到 D27 同款 ~1ms 预算级，难）；冷态/新区域单 chunk ~35-37ms（门槛 ~1×，易）。GPU 的真实机会窗口在冷路径（世界加载卡顿/离线预生成首遍），不在暖区稳态。**

## B. C2ME aquifer 形态源码级钉死（worker-c2me-aquifer-emitter-260908-02.md，vendored 源树 versions/1.20.1/data/C2ME-fabric，HEAD 615baf8）

1. **每点 GPU 上传 = 0 字节**（仅 global_id 坐标 + 共享小表 ~3KB/批）；est cache 由 GPU kernel 预填充后 device 内流转，aquifer 主 kernel 逐 block 在 GPU 现场跑（12 候选→4×uint64），结果不 readback——J1/J3/J4 全部结构域通过，且 **convergence §二「坐标派生量」待验假设升级为源码事实**。
2. dispatch ≈ 4-chunk 方批 1260 点（~315 点/chunk）；手写专项化直排 kernel（非解释器形态，D27 被否形态不适用）。
3. FLAT_CACHE_PREFILL 两表述矛盾已解（GPU 预填充→readback→再上传→主 kernel 只读 miss 即 trap）。
4. 无公开收益数字（降级声明）。
5. 注意：C2ME aquifer kernel 依赖其 df_noise_kernel（专用 density kernel）生态与 est GPU 预填充——**CoreSwap 复刻同样绕不开 initial_density 的专用 kernel**（与 worker-aquifer-input-face §C1 一致）。

## C. 修订后的 Phase 0 问题定义（交 todo 6）

- B2 不是「把 35ms 的阶段 GPU 化」，而是「**initial_density 专用 kernel** 的最小切片，服务 est 冷路径 + surface」；暖区稳态 GPU 无利可图（分母 1ms）。
- 与 **todo 5 CPU 路线的竞争面因此变大**：est 冷路径的 CPU 侧优化（L2 预取/预计算、old_blended 冷价、后台预热线程）可能以小得多的工程量吃掉大部分冷态痛点——Phase 0 必须双路线同页对比，且先钉「冷路径在真实使用中的暴露面」（玩家实际遇到多少冷 chunk）。
