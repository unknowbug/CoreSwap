# MLP 立项 + Rust 语言评估（决策记录，2026-08-24 后续）

> 主会话决策记录 | 状态：draft（用户已定：11× 用 C++ MLP 先行；Rust 作为未来方向评估）

## 一、MLP 立项（production 完整 MLP）

**背景**：11× 争用 = 长串行依赖链 + latency QoS。MLP 假说已验证（软流 K=16 -36% 吞吐，mlp-probe-result.md）。

**立项内容**：**production 完整 MLP** = 数据驱动 op 表（无 split）+ 软流（≥8 路多点交错 op 段）。
- 关键：finalDensity->sample 是黑盒（一次一点，无法手工交错）→ 需**拆成 op 记录**（数据驱动）才能软流多点。
- 本质 =「无 split 直排 + 软件流水」：无 split 直排保留依赖链（预期仍 ~10×，虚调用/递归已证无效）；**软流打断依赖链**（点间交错）→ 对症。
- 实现：复用 DFC 的 CpuBackend（已有数据驱动 op 表 eval_df_base）+ **去 split 预拆分**（改 eval_df_base noise 从 splitCoord → 直接 normals.sample）+ **软流 ≥8 路**。
- 预期：降 11×（latency QoS），但生产依赖链更复杂（noodle range_choice/虚调用/grid trilinear），实际幅度需实测。
- 大工程（改采样路径 + 软流循环），另立项，本 session 收束后启动。

## 二、Rust 语言评估（针对 CoreSwap worldgen）

### Rust 优势
1. **编译期并发安全**（ownership + Send/Sync）：防数据竞争/悬垂——正是本 session 踩过的线程池 notify bug（0a781e1）/thread_local 缓存污染/共享只读表跨线程类问题。Rust 编译期拒绝。
2. **零成本抽象 + 无 GC**：性能 ≈ C++。
3. **数据驱动表达更自然**：`enum`+`match`（op 表）、iterator/`core::simd`（软流/MLP）。

### 关键澄清（重要）
- **11× 是访存延迟 QoS（性能/延迟问题），Rust 不解决**——Rust 保证内存/并发安全，**不保证性能 QoS**。MLP 软流打断依赖链，**C++ 已验证 -36%**（Rust 并不比 C++ 强）。
- **转 Rust（CoreSwap 全量）= 推倒重来 + 重新验证**：大量成熟 C++（density/worldgen_api/spline/terrain）+ 逆向对齐 Java（block_probe 逐位验证）。重写全部 + 重做对齐验证，**成本远超 MLP 收益**。

### 结论
| 决策 | 依据 |
|---|---|
| **11× 用 C++ MLP 先行** | 性能问题（Rust 不解）+ C++ 已做软流验证 + 已有 DFC 基础 |
| **CoreSwap 整体不转 Rust** | 全量重写 + 重新验证成本 >> 收益；尤其逆向对齐（block_probe） |
| **Rust 作为未来方向** | 若将来重写某**纯新高并发模块**（Rust 编译期兜底层并发/内存 bug）；或 C++ 并发/内存 bug 频发到难以维护 |

**一句话**：Rust 对「新写高并发/内存敏感 + 无既有大代码」的场景很好；对 CoreSwap（成熟 C++ + 逆向对齐 + 11× 是性能问题）**转 Rust 不划算**，**11× 用 C++ MLP 解决**。

## 引用
- mlp-probe-result.md（MLP 假说验证 -36%）
- 11x-contention-investigation-log.md（11× = latency QoS）
