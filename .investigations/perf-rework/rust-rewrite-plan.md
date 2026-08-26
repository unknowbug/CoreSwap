# CoreSwap worldgen Rust 重写规划（架构/迁移/验证）

> 主会话 | 2026-08-24 后续 | 用户拍板全量重写 | 状态：plan（Phase 1 准备）

## 1. crate 结构
- `WorldgenRust/`（workspace 或单 crate，LIB）+ `src/bin/verif.rs`（验证工具）。
- **LIB**（无外部 crate 依赖，纯核心）：noise / density / spline / terrain / api / verif.

```text
WorldgenRust/
  Cargo.toml
  src/
    lib.rs
    noise.rs      // DoublePerlinNoiseSampler / OctavePerlin / Perlin + perm 表
    density.rs    // DF 树（InterpolatedDF/SplineDF/FlatCacheDF/Cache2DDF/NoiseDF/...）
    density_builder.rs  // worldgen JSON -> DF 树（buildNode/spline）
    spline.rs     // SplineDF（data-driven 表 + 采样，可软流）
    terrain.rs    // finalDensity 构建 + fill 逻辑
    api.rs        // wg_create/fill/sample 等价（C ABI 导出）
    json.rs       // worldgen JSON 解析（serde 或手写）
    verif.rs      // 验证：ref 数据（Java 参照）对比，逐位
```

## 2. 迁移顺序（从 C++，逐模块 + 逐位对齐）
| 顺序 | 模块 | C++ 源 | 对齐 |
|---|---|---|---|
| 1 | noise | noise.h/.cpp（DoublePerlinNoiseSampler/perm/split）| ref 噪声数据 |
| 2 | density DF 树 | density.h（InterpolatedDF/SplineDF/FlatCacheDF/NoiseDF/Unary/Binary/...）| ref 密度（Java cns）|
| 3 | spline | density.h SplineDF + density_builder.h buildSplineNode | ref spline 表 |
| 4 | terrain | worldgen_api.cpp finalDensity 构建 + fillOneChunkCore | ref blocks |
| 5 | api | worldgen_api.c/.cpp（wg_create/fill/sample）| C ABI + 双跑 |
| 6 | 验证/性能 | block_probe 等价 + conc_density_probe | ref + 软流 MLP |

## 3. 验证策略（逆向对齐——这是大头）
- **参照数据**：现有 `versions/1.20.1/data/`（vanilla blocks / density cns / 密度 dfreg）作为 ref。
- **逐位对齐**：Rust 采样输出 vs Java 参照（block 逐位 / density 逐位）。用现有 ref 文件（`vanilla_density_overworld_*` / `vanilla_*_blocks`）做测试。
- **测试分档**：unit（noise 单值）+ integration（chunk 密度 vs ref）+ 回归（Rust vs C++ block_probe 输出）。
- **性能**：重建 conc_density_probe 等价（Rust），测 11× 是否被软流 MLP 消除（目标 ~1.x）。

## 4. 关键设计（Rust 优势 + 软流）
- **DF 树**：`enum DensityFunction`（数据驱动，非虚调用）——Rust 天然 enum+match，比 C++ 虚调用树更顺。
- **Spline**：显式栈 + 数据驱动表（无递归）。
- **软流 MLP**：`sample_batch(K，...)`（K 点交错 op——Rust 已验证 -70%），用于 fill（打破 11× latency QoS）。
- **并发**：Rust Send/Sync（共享只读表）/ thread_local 独立（模拟 production），Rust 编译期保证。

## 5. Phase 里程碑
- P1: cargo 项目 + 骨架 + 验证框架（ref 数据加载 + 逐位对比 helper）。
- P2: noise 移植 + 对齐（ref 噪声）。
- P3: density DF 树移植 + 对齐（ref 密度）。
- P4: spline + terrain 移植 + 对齐（ref blocks）。
- P5: api + 双跑（Rust vs Java/C++ 参照）+ 回归。
- P6: 性能（软流 MLP + 并发）——目标 11×→1.x。

## 6. 现状（可复用/参考）
- `versions/1.20.1/data/`：ref 数据（Java 参照，逐位对齐用）。
- `versions/1.20.1/cpp/worldgen/src/`：C++ 参照实现（迁移源）。
- `.investigations/perf-rework/mlp_probe.rs`：软流原型（Rust 已有）。
- Rust 1.98.0（msvc）已装；C++ 项目（build.ps1 cl+lib）仍在（对照/验证）。

## 引用
- rust-rewrite-decision.md（决策）
- rust-mlp-validation.md（Rust 软流 -70%）
