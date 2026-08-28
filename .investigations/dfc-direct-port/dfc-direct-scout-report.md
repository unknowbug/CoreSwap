# DFC 直排移植到 Rust — 摸底报告（recode.scout，2026-08-28）

> 载体：`.investigations/dfc-direct-port/dfc-direct-scout-report.md`
> 来源：recode.scout subagent（只读勘探，未改代码）
> 任务：为「DFC 直排」性能优化做移植范围摸底（Rust CPU 直排，消除运行时递归）。

## 结论

**最小可行范围 = 方案 B（只扁平化 spline），且已有现成蓝本（C++ cpu_backend.h）。**

## 核心发现

1. **C++ CPU 直排已存在且已集成**：`dfc_gen.py` 的 `gen_cpu_sampling()`（L1885）生成完整 CPU 扁平表求值路径，产物 `cpu_backend.h`（`DF_TYPE[163]`/`CLOSURE_TYPE[210]`/`TOP_TYPE[21]`/`spline_eval` 显式栈），已通过 `worldgen_api.cpp` `WG_DFC_CPU=1` 门控集成。**这是 Rust 移植的直接蓝本，不是从零设计。**
2. **gpu_density_engine 本身是纯 GPU**：只做 CPU 预拆分 + 数据上传，求值在 Vulkan shader。CPU 直排求值在 `cpu_backend.h`（生成器产物）。
3. **Rust 热点确认存在**：`SplineData::sample_node`（density.rs L89-125）递归调用自身 + `loc_fns`（`Arc<DensityFunction>`，L83）虚调用，finalDensity 含 6 SplineDF 深层嵌套 → 递归指数级膨胀。与 C++ 侧 11× 热点同源。
4. **Rust 现状**：`density.rs` 的 `DensityFunction` enum（23 variant）+ 递归 `sample()`；`spline.rs` 是 1 行空壳；`density_builder.rs` 的 `SplineBuilder` 已产出与 dfc_gen.py 同构的 SplineData 扁平表。

## 建议移植方案

- **方案 B 起步**（只扁平化 spline：`sample_node` 递归 → 显式栈，`loc_fns` 虚调用 → 数据表查表）——改动小、风险低、直接命中热点。
- **验证后扩展为方案 A**（全树扁平化：enum 树 → 后序扁平节点数组 + 显式栈循环，镜像 `cpu_backend.h` 的 `eval_df_base`/`eval_df`/`spline_eval`/`sample`）。
- **方案 C**（SteelMC 式编译期 transpiler）为长期目标，非最小可行范围。

## 关键行号索引

- `dfc_gen.py`：`gen_cpu` L1651 / `gen_cpu_sampling` L1885 / `spline_eval` 生成 L2249 / `_compute_val_layout` L414
- `cpu_backend.h`：`DF_TYPE[163]` L637 / `spline_eval` L839 / `eval_df_base` L1041 / `eval_df` L1098 / `sample` L1142
- `worldgen_api.cpp`：`WG_DFC_CPU=1` L453-457
- `density.rs`：enum L412-435 / `sample` L438-499 / `SplineData` L78-87 / `sample_node` L89-125
- `spline.rs`：空壳（1 行）

## 建议落盘

`.investigations/dfc-direct-port/` 下：`dfc-direct-scout-report.md`（本报告）+ `cpu_backend_cpu_path.md`（蓝本摘录）+ `spline-recursion-hotspot.md`（热点确认）。

## 状态

draft 置信度，需主会话 + judge 审查后由用户拍板。
