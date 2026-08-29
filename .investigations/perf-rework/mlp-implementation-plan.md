# MLP 实现方案（production 完整 MLP，分阶段）

> 主会话 | 2026-08-24 后续 | 状态：plan（分阶段推进）
> 依据：MLP 假说已验证（软流 -36%，mlp-probe-result.md）；DFC 结构已读（cpu_backend.h：eval_df/eval_df_base/normal_noise/sample/split/splitTop）；11× = latency QoS。

## 目标
production 完整 MLP：**数据驱动 op 表 + 软流（≥8 路多点交错 op 段）**，降 11×（latency QoS），保留单点快（不预拆分，无 DFC 600× 慢）。

## 关键理解（DFC 现状，cpu_backend.h）
- `sample(x,y,z)`：`splitTop(x,y,z,splitCoord)`（@c0 拆分）→ `eval_density(0,x,y,z)`（数据驱动解释器）。
- `eval_df`（top 闭包）/`eval_df_base`（interp 闭包）：**单点**解释器——遍历 CLOSURE_* op 表（TYPE/A1/A2/.../SLOT），switch 算，存 val[SLOT]。**无虚调用/递归**。
- `normal_noise(idx,sIdx)`：读 `splitCoord`（split 预拆分的 [ix,iy,iz,gx,gy,gz]）→ `pn_sample3_f32`（perm 噪声）。
- `split()`/`splitTop()`：预拆分坐标（**慢 600× 来源**——split 每 cell/每点整树拆分）。

## 两个改造（耦合）
1. **去 split**：`normal_noise` 从读 splitCoord → **直接 `normals[vi].sample(...)`**（实时算，不预拆分）。⚠️ 需**复刻 split 坐标语义**（split 保存 ix,iy,iz 整数坐标 + gx,gy,gz 小数部分 = 精度变换/`maintainPrecision`）。→ 这是**正确性关键**（复刻错 = 值错）。
2. **软流 K 路**：`eval_df`/`eval_df_base` 改为 **K 路**（`val[K][slots]` + K 点同一 op 交错）——`sample_batch(K, x[],y[],z[])`。→ K 个独立点的 op 计算/load 交叠（MLP）。

## 分阶段
- **阶段 A（软流改造，加法）**：加 `eval_df_base_soft`/`eval_df_soft`（K 路）+ `sample_batch(K,...)`（保留原单路）。编译 + 测 **K 路 vs 单路吞吐**（用 dfc_cpp_conc 或新探针）。**先只做软流**（保留 split）——若软流 K 路吞吐提升 → 软流改造有效；若 splitTop 掩盖 → 需先去 split（阶段 B）见效果。
- **阶段 B（去 split）**：改 `normal_noise` 直接 `normals.sample`（复刻 split 坐标语义）→ 消除 split()/splitTop() 600× 慢。block_probe 对拍（maxdiff）确保值对。
- **阶段 C（完整 MLP，测并发放大）**：`sample_batch`（去 split + 软流）进 fillOneChunkCore（每 chunk K 路软流采样），`conc_density_probe` 测 **T8/T1 放大比**（vs 生产 10.32×）。
- **阶段 D（对拍）**：block_probe 逐位（软流 K 路 vs 单路，maxdiff=0）。

## 正确性风险
- 去 split 复刻 split 坐标语义（编译期难验，需 block_probe 对拍）。
- 软流 K 路（val[K][slots]）每 op 分支正确性。

## 预期 / 边界
- 软流 -36%（微基准，纯访存依赖链）；production 依赖链更复杂（noodle range_choice/虚调用/grid trilinear），实际幅度需阶段 C 实测。
- 若阶段 A（保留 split）软流吞吐已提升 → 强化软流有效；若非 → 需先去 split（阶段 B）再测。

## 引用
- mlp_probe.cpp / mlp-probe-result.md（软流 -36%）
- cpu_backend.h（DFC 结构）
- 11x-contention-investigation-log.md（11× = latency QoS）
