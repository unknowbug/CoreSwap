# 候选 .bB — tri-cut major diff 单点采样语义审查（cpu_backend.h 侧）

**status: draft**（置信度：机制方向 candidate 级证据，最终归属待 .bA/GPU 侧对照 + judge）
**验证分层：Degraded（纯静态代码阅读，未运行任何探针）**
日期：260903（依数据文件名 tri-cut-260903-04）

## 逐点核对数据（tri-cut-260903-04.txt）

- 行 5-12：chunk(0,0) 全部一致（absdiff ≤ 7.45e-8）→ GPU 引擎在 chunk(0,0) 语义正确。
- 行 13/14（y=-64，两 chunk）与行 12/20（y=320，两 chunk）**四个值跨 chunk 完全相同**（0.0374824181 / -0.0249947906）→ y=-64/320 处密度为位置无关的 clamp 平台（vanilla 端 slide：`YClampedGradient::clamped_map(y,-64,-40,…)` 与 `clamped_map(y,240,256,…)`，见 vanilla_density_functions.rs:163），这两行**无鉴别力**。
- 行 15-19：仅 chunk(-288,-256) 的 y∈{-56,0,64,128,200} major diff。

## 结论

### ① sample() 单点采样语义（cpu_backend.h:1142-1151）——语义自洽，CPU 路径无发现错误

- `sampleInterpGrid` 的 chunkX/chunkZ 由采样点自身推导：`floorDiv(ix,16)`（行 1017），任意（含负）坐标下 grid 建在正确的 chunk 上；GridSlot 以 `key=(chunkX<<32)|chunkZ`（行 1011/1018）精确匹配，单点乱序采样只会触发重建（慢），不会读错 chunk。
- grid miss 时 `buildInterpGrid`（行 962-1012）保存/还原 splitCoord（行 964/1004），网格节点 = `split(nodePos)` 后 `eval_df_base(interpIdx,0,…)` 的 @c0 值；`normal_noise` 全部从 splitCoord 缓冲读（行 766-792），非实参坐标 → 「节点值 = delegate(节点坐标)」等价关系成立（verif_grid_cache_correctness.md）。网格节点值与 vanilla InterpolatedDensityFunction 的 8 角点语义等价。
- edgeCol 复用（行 969-973/1006-1010）条件严格（左邻 chunkX-1 同 chunkZ），x 对齐 `chunkX*16` ✓；不满足即退回全建（行 968 注释），无错读路径。
- **边界缺陷（真实但不解释本 diff）**：`sampleInterpGrid` 行 1022/1024 当 `gy=384`（y=320）时 `cy=48, cy+1=49` → `grid[49]` 越界读；`iy<minY` 时 cy 为负同样越界。数据中 y=320 行恰好一致，说明该 y 下 interp 未被求值（clamp 分支主导）或越界读未踩雷——**应修但不属于本次 major diff 根因**。

### ② interp_N / 负坐标索引（行 1017-1023、1081-1084）

`floorDiv` 保证 chunkX/chunkZ 负坐标正确；gx∈[0,15]、gy=iy-minY≥0 时 cx/cy/cz、fx/fy/fz 推导与 vanilla `InterpolatedDensityFunction` 的 cell 索引一致。y 不在 grid 平面（gy%8≠0）由 fy 三线性覆盖，正确。唯一的索引缺陷即上述 ① 的 cy+1=49 越界。

### ③ y=128 与 y=200 的 CPU 值 -0.458333343 —— 不是 CPU bug，是合法饱和平台

- **-0.458333343 = -11/24（f32 舍入）= `DF_SQUEEZE` 在输入 clamp 到 -1 处的饱和值**：squeeze 输出 `c/2 - c³/24`（行 1065/1120），c=-1 → -1/2 + 1/24 = **-0.458333…**。两坐标（x,z 不同）取值相同正是饱和平台特征：输入一旦 ≤ -1，输出与位置无关。
- chunk(0,0) 的 y=128/200 两点 **GPU 与 CPU 完全一致地给出同一饱和值**（行 10/11，absdiff=0）→ 该饱和在正确实现下确实发生，不是 CPU 常数分支伪影。
- vanilla 语义佐证：final_density 顶层的 `y_clamped_map(y,-64,320,1.5,-1.5)` 垂直梯度（vanilla_density_functions.rs:163）在高 y 段给出强负输入，squeeze 饱和到 -11/24 是 vanilla 高空预期行为。**chunk(-288,-256) CPU 在 y=128/200 重复 -0.4583 是正确侧。**

### ④ GPU 错误模式签名

不一致点 GPU 值全部落在窄小值带（0.0075~0.049），且**完全丢失 y 结构**（y=-56 与 y=128/200 同量级）与**丢失 squeeze 饱和**。而同一 GPU 引擎在 chunk(0,0) 全对。签名指向：**GPU 在非零/负 chunk 的 interp 输入失效**——候选（按可能性）：
1. GPU 侧 per-chunk grid/split 缓存的 key 或负 chunk 基址偏移计算错误（负坐标 floorDiv/位打包，如 `chunkX<<32|chunkZ` 用有符号移位或 u32 截断），读了 chunk(0,0) 或零填充区域 → interp 贡献 ≈ 0 → 输出坍缩为非 interp 残余项的小值带；
2. GPU split/noise 槽位 fetch 在负 chunk 下偏移错误（噪声读错区 → 近零均值）。
「全部读 @c0 非 interp 值」签名不符：那会保留 8 角点级变化与 y 结构，与观测不符；「零填充 grid」与「错误 chunk key」均与观测吻合（输出≠0 因非 interp 项仍变动）。

### 总判定

**「哪个路径是对的」：CPU（cpu_backend.h grid+trilinear）在不一致点上是对的**（饱和平台 + vanilla 高空语义 + Rust f64 参照同构）；GPU 在 chunk(-288,-256) 的 interp/负 chunk 处理有 bug。**注意：本结论未在 GPU 侧源码复核（.bA 范围），judge 应要求与 GPU 侧候选交叉验证后才能升 candidate。**

## 证据索引

- `versions/1.20.1/cpp/worldgen/gpu-assets/cpu_backend.h`：行 962-1012（buildInterpGrid）、1016-1036（sampleInterpGrid，1022 越界缺陷）、1041-1078（eval_df_base）、1065/1120（squeeze 公式）、1142-1151（sample/splitTop）
- `.investigations/lossless-accel/cmd-output/tri-cut-260903-04.txt` 行 5-21（逐点数据）
- `WorldgenRust/src/generated/vanilla_density_functions.rs:163`（y_clamped_map 端 slide / 垂直梯度 1.5→-1.5）
- `WorldgenRust/src/density_builder.rs:312`（squeeze 节点存在，Rust 整树直采 0 mismatch = 已验证参照）

---
**supersedes 回指针（§15.4）**：本文「GPU 侧负 chunk 基址/缓存 key 输入失效」候选被 260903-04 tri-cut3 证伪（重编 spv 后负坐标远端 + 全 y 柱全对，见 .investigations/lossless-accel/route2-ffi-260903-04.md 根因闭合节）。CPU 参照侧正确、饱和值论证、grid[49] 独立越界缺陷等结论保留有效。
