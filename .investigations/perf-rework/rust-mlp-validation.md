# Rust 软流验证（mlp_probe.rs）— Rust vs C++ 软流增益对比

> 主会话 + 用户环境 Rust 1.98.0 | 2026-08-24 后续 | 状态：draft
> 目的：验证「Rust 写软流/数据驱动是否比 C++ 更有效」——用户判断「换 Rust 缓解不少」。
> 同基准 mlp_probe（访存依赖链 L=15 + 软流 K 路 + 随机索引），Rust 版用 read_volatile 防 LLVM 优化消除访存。

## 关键发现（C++ 微基准 per 计算 bug）
C++ `mlp_probe.cpp` 的 `per = 1e6 * wall_ms / N`（nowMs 返回毫秒却乘 1e6 当秒）→ **误读 C++ 为 44μs/点**；实际 C++ 也 ~50ns/点（同 Rust）。**修正后**：用 **wall 毫秒**对比（非 per 微秒）。

## Rust vs C++（wall 毫秒）

### 4MB 数组（a/b/idx 共 12MB < L3 16.5MB，L3 命中；N=400000）
| | seq | soft8 | 软流增益 |
|---|---|---|---|
| C++ | 19.6ms | 13.1ms | -33% |
| Rust | 21.0ms | 11.2ms | **-47%** |

### 32MB 数组（>L3 16.5MB，真实 DRAM miss 模拟 production 访存；N=200000）
| | seq | soft8 | 软流增益 |
|---|---|---|---|
| C++ | 17.4ms | 8.0ms | -54% |
| Rust | 25.3ms | **7.7ms** | **-70%** |

## 结论
- **真实访存（32MB，production 类似）下，Rust 软流更有效**：-70% > C++ -54%；Rust soft8 绝对更快（7.7 vs 8.0ms）。
- Rust seq 较慢（顺序依赖链 LLVM 优化弱，25.3 vs 17.4ms），但**软流交错时 LLVM 优化强**（-70%）。
- **支持「Rust 换能缓解」**（对软流/数据驱动这类负载，Rust > C++）。

## 边界 / 权衡
- 微基准 = 简化访存依赖链；**production 完整负载**（虚调用/grid/noodle/spline 多 octave）不同，需 production 验证。
- **11×（性能问题）**：C++ 软流（改现有，-54% 已证）**更划算**（不重写）；Rust 全量重写（worldgen + 逆向对齐重新验证）巨成本。
- **Rust 真优势**：并发安全/内存安全（编译期防数据竞争/悬垂——正是本 session 线程池 notify/thread_local 类 bug）——**若那是痛点**，Rust 有价值（但需全量重写）。

## 引用
- mlp_probe.rs（read_volatile 版，rustc 1.98.0 编译）· mlp_probe.cpp（32MB 版）
- rust-install-guide.md（Rust 安装，用户挂代理成功）
- mlp-probe-result.md（C++ -36%/44μs——注意该文档 per 用 44μs 是 C++ 误读，实际 ~50ns；-36% 增益对（wall），数值以本文件为准）
