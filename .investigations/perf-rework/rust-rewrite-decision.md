# 决策：CoreSwap worldgen 转向 Rust 全量重写

> 主会话记录 | 2026-08-24 后续 | 用户拍板 | 状态：confirmed（用户决定）

## 决策
**CoreSwap worldgen（density/spline/terrain）转向 Rust 全量重写**（替代 C++）。

## 动机（用户判断 + 已验证）
1. **Rust 软流/数据驱动更有效**（已实测）：32MB 真实访存下 Rust 软流 **-70%** > C++ **-54%**（rust-mlp-validation.md）。LLVM 优化交错更强。
2. **Rust 并发/内存安全**：编译期 ownership+Send/Sync 防数据竞争/悬垂——正是本 session 踩过的类问题（线程池 notify bug 0a781e1 / thread_local 缓存污染 / 共享只读表跨线程）。C++ 靠纪律，Rust 靠编译器。
3. 用户判断「换 Rust 缓解不少」——已用软流验证**部分成立**。

## 依据（本 session 已确证）
- 11× 争用 = 长串行依赖链 + latency QoS（production 模型排除链完整：存储/递归/虚分派/buildGrid/顶层虚分派数/spline/带宽/SMT 全排除）。
- 修复方向 = 提升 MLP（软流打断依赖链）——Rust 软流更有效（-70% vs C++ -54%）。
- C++ 现状的并发/内存 bug 类问题（thread_local 污染/线程池竞争）——Rust 编译期兜底。

## 已知成本 / 风险
- **全量重写**：worldgen（density/spline/noise/terrain）+ worldgen_api + 逆向对齐（Java 参照）。
- **重新验证**：Rust 输出 vs Java 参照（block_probe 等价）逐位对齐——**逆向对齐是大头**。
- 工程量：数周-月（骨架+移植+对齐+验证）。

## 里程碑（规划见 rust-rewrite-plan.md）
- Phase 1: cargo 项目 + 架构骨架 + 验证框架（ref 数据）。
- Phase 2: noise（DoublePerlinNoiseSampler 等）移植 + 对齐。
- Phase 3: density DF 树（InterpolatedDF/SplineDF/FlatCacheDF/...）移植 + 对齐。
- Phase 4: spline + terrain（finalDensity 构建）+ 对齐。
- Phase 5: api（wg_create/fill 等价）+ 集成。
- Phase 6: 双跑（Rust vs C++/Java 参照）回归 + 性能（MLP 软流）。

## 引用
- rust-mlp-validation.md（Rust 软流 -70% vs C++ -54%）
- rust-install-guide.md（Rust 1.98.0 安装，挂代理成功）
- 11x-contention-investigation-log.md（11× = latency QoS）
