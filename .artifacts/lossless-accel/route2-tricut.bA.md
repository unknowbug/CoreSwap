# 候选 .bA：GPU fill 执行路径审查——哪些输入类会导致 GPU 输出错误/陈旧

- status: draft（静态审查，Degraded——本 worker 未运行任何命令，仅代码+数据阅读）
- 审查对象：`versions/1.20.1/cpp/worldgen/src/gpu_density_engine.cpp`（129 行全文）、`gpu-assets/cpu_backend.h`（生成产物，splitTop/buildInterpGrid/sampleInterpGrid/eval_df 区段）、`.investigations/perf-rework/dfc_gen.py`（GLSL 生成器，shader 模板与 interp 生成区段）
- 数据：`.investigations/lossless-accel/cmd-output/tri-cut-260903-04.txt`、`gpu-corner-probe-260903-04.txt`

## 结论（四问逐答）

### ① fill(n) 跨多 chunk 批量：GPU 侧无共享 interp grid，语义安全（排除）

关键事实：**GLSL shader 没有路径 C grid 缓存**。shader 的 interp_N 是无状态逐点 8 角点重算：

- dfc_gen.py:1471-1487（GLSL interp 生成）：每 interp 函数内部 `chunkX = floorDivP(ix, 16)` **逐采样点**推导 chunk，8 角点 `chunkX*16+(cx+dx)*4` 局部计算，三线性插值后返回——函数纯输入驱动，无跨调用状态。
- 「grid 缓存（path C：每 interp 每 chunk 5×49×5 + 三线性）」只存在于 **CPU 侧** cpu_backend.h:937-1036（`GridSlot` thread_local，key 按 `(chunkX<<32)|chunkZ` 逐点采样时推导，cpu_backend.h:1016-1020）与 dfc_gen.py:2347-2446（gen_cpu 模板）。
- gpu_density_engine.cpp:106-118 `fill()`：per-point `backend.split()` → upload coordBuf/splitBuf → `dispatch(n)` → readback outBuf，均以 sIdx=采样点索引区段化（shader 主函数 dfc_gen.py:2843-2847：`idx >= outBuf.density.length()` 守卫 + 逐 idx 读 coordBuf）。

→ **「一次批量跨多 chunk 时 grid 按哪个 chunk 建」这个问题在 GPU 侧不存在**：无 grid、无 per-batch chunk 状态，每 invocation 自含。混合 chunk 批量不是致错输入类。（CPU 侧 CpuBackend::sample 逐点采样时 grid key 也按点坐标推导，混 batch 同样安全。）

### ② buffer 未写槽读陈旧值：未发现可致错的未写槽（基本排除，留一个无害项）

- valBuf（binding 5，n×perSample=352，dfc_gen.py:2760-2761）= 解释器值栈，按 `PER_SAMPLE*sIdx + B + corner*VAL_SLOTS + SLOT_OF[ci]` 区段（dfc_gen.py:564-588）。liveness 槽布局保证先写后读；未写槽（其他 sIdx/其他 corner 区段、以及本点未触及的槽）从不被读。
- outBuf 每 dispatch 每 idx 必写（dfc_gen.py:2847）；readback 只读前 n 个 float（gpu_density_engine.cpp:117）。
- ensureBuffers 容量复用（gpu_density_engine.cpp:31 `n <= curCap` 直接返回）会留下**尾部陈旧数据**，但 upload/dispatch/readback 都只覆盖前 n 点，陈旧尾部不可达 → 无害。
- 唯一理论竞态 = upload→dispatch 无显式 barrier（若 VkRuntime 内部未做），但该竞态是非确定性的，与 tri-cut 中**逐点确定性的失配模式**不符（同 chunk 同 x,z 列上 y=-64/320 精确一致、中间 y 稳定失配，两次以上运行可复现），不作为主候选。

### ③ push constant / specialization per-batch 状态：未发现（排除）

shader 模板主函数（dfc_gen.py:2843-2847）无 push constant；spec constant（PER_SAMPLE/minY 等，dfc_gen.py:1761、2757 附近）均为 seed 级常量。fill() 每次只变 upload 内容与 dispatch count，无 per-batch 状态需要刷新。

### ④ 数据模式解释：GPU 侧小偏差在分支阈值处被放大（主候选）

**数值指纹**：CPU 侧 -0.458333343 精确等于 **squeeze(clamp(v,-1,1)) 在 v≤-1 时的常数**：`(-1)/2 - (-1)³/24 = -0.458333`（公式见 cpu_backend.h:1065 / dfc_gen.py:569）。即 CPU 在 y=128/200 处 `mul/squeeze` 链的输入 ≤ -1 被 clamp 成 -1；GPU 输出 0.019/0.043（小正值）= **同一 min/squeeze 分支结构下 interp 值未越阈值时的正常输出**。GPU 失配值彼此接近（0.0075~0.049）正是「分支翻转后的同一连续函数面」特征，而非随机陈旧值。

**空间模式**：所有 16 点都在 y 网格节点上（-64→gy=0、-56→gy=8、0→64、64→128、128→192、200→264、320→384），interp 在节点上直接取网格/角点值（fy=0）。失配 = y 网格行 8~264 区间；精确一致 = 行 0 与 384。即 **GPU 与 CPU 的 interp delegate 在该 chunk 的中海拔角点值存在小符号性偏差，行 0/384 处为零或同号**。

**放大机制**：finalDensity = min(squeeze(mul(c, interp)), noodle) 顶层的 clamp(±1)/min 分支使 ~1e-7 级偏差在阈值附近放大为 O(0.5)。证据：匹配区 16 点中仍有 3.73e-08 / 7.45e-08 的 ULP 级残差（tri-cut 行 8-9）——GPU 与 CPU **处处已有 f32 级微差**，chunk(0,0) 各点恰好不在阈值敏感区所以不被放大；chunk(-288,-256) 中海拔值恰在 clamp(±1) 边界附近 → 5 点大失配。「同 chunk 部分 y 一致部分不一致」= 该列上只有中海拔行的中间值落在阈值敏感带。

**偏差源头候选（待 .bB/.bC 收敛，按可能性排序）**：
1. **GLSL spline_eval vs C++ spline_eval 数值差**（interp delegate 内 DF_SPLINE，y 分段 spline 常数 F0/F1 含 -60/64/320/321 等 y 带边界，恰好覆盖行 8~264 失配带）：10-timewise-archive.md:1374-1378 记录 GLSL while 栈边界分支（嵌套 value 外推 stage 4/5）曾有「直接返 0.0」bug 并已修——**需核实 final_density.spv 是否为该修复后重新编译**（spv 是构建产物，CPU backend .h 与 spv 可能不同代）。
2. normal_noise GLSL 重建的负坐标 floorDiv / maintainPrecision 残差（dfc_gen.py:1891 已知易错点清单）。
3. interp_noise（old_blended）y-fade 双早停的 f32 汇合差。

## 致错输入类判定（本候选的直接回答）

会导致 GPU 输出与 f32 CPU 参照 major diff 的输入类 = **中间值落在 clamp(±1)/range-choice/min 分支阈值敏感带的中海拔（y 网格行 ≈8~264）采样点，且所在 chunk 局部值非平坦**；批量混 chunk、buffer 陈旧、push constant 均排除为成因。ULP 级微差全点存在，分支放大是 major diff 的直接机制。

## 证据引用索引

- gpu_density_engine.cpp:30-31（ensureBuffers 容量复用）、106-118（fill 上传/dispatch/readback 全以 n 区段化）、120-125（sample→fill(1)）
- cpu_backend.h:542-598（splitTop @c0 注释与实现）、937-1036（path C grid 缓存——**仅 CPU**）、1079-1097（interp_N：sIdx=0 走 grid，否则 8 角点）、1142-1151（sample()：splitTop + grid 命中语义，splitTop 只覆盖非 interp 路径 @c0）
- dfc_gen.py:1471-1487（GLSL interp 8 角点无状态生成）、1474（floorDivP 逐点推 chunk）、2757-2761+2843-2847（shader 主函数/outBuf/valBuf）、564-588（valBuf 槽布局）、1891（已知负坐标易错点）、2347-2446（CPU 模板 grid——对照证明 shader 无此路径）
- tri-cut-260903-04.txt 行 8-9（匹配点 ULP 残差 3.73e-08/7.45e-08）、行 15-19（5 点失配）、行 18-19（CPU -0.458333343 = squeeze(-1) 常数）
- 10-timewise-archive.md:1374-1378（GLSL spline 边界外推 bug 修复史——spv 代际核查点）

## 后续验证建议（主会话执行）

1. 核对 final_density.spv 编译时间 vs dfc_gen.py 最后一次 GLSL 修复（spline stage4/5）的提交先后——不同代即重编 spv 复测 tri-cut。
2. 用 gpu-corner-probe 扩展：dump chunk(-288,-256) y 网格行 0/8/64/264/384 的 interp delegate 单值（CPU grid vs GPU 逐角点），定位首个分歧行/分歧节点 → 区分候选 1/2/3。
3. 若重编 spv 后 tri-cut 全绿 → 根因 = spv 代际陈旧（候选 1 实锤）。

---
**supersedes 回指针（§15.4）**：本文「分支阈值放大」候选的「spv 旧代」半边被 260903-04 根因闭合证实（final_density.spv 陈旧产物，见 .investigations/lossless-accel/route2-ffi-260903-04.md 根因闭合节）；「阈值放大连续函数面」机制解释被证伪（失配值与正确值非连续邻域，而是历史错值签名）。
