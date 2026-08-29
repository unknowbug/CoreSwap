# DFC GLSL → C++ 采样函数映射表（gen_cpu 扩展依据）

> 范围：`final_density.comp`（GPU 已验证，e2e maxDiff=3.128e-07）的采样函数全集 → CpuBackend（`cpu_backend.h`）之上的 C++ 移植。
> 目标：为 `dfc_gen.py gen_cpu`（L1649+）扩展生成 C++ 采样函数，复用已有数据表（splineNodePack/Locs/Ders/ValF/ValKind/ValNode + normal perm + split 行）。
> 可复用原型：`mvp_spline_eval.cpp`（C++ 显式栈 spline_eval 已验证）、`dbg_full_sim.py`（Python 全量复刻解释器，sim vs e2e-A5 maxDiff=5.7e-9）。
> 校验基线：e2e GPU vs CPU 参照 maxDiff=3.128e-07，avgDiff=1.097e-08 —— C++ 移植必须落到同量级（D19/D23 语义已确定）。

---

## 0. 精度 / 数据来源分层（先定全局约束）

| 域 | 精度 | 采样来源 |
|----|------|----------|
| NormalNoise / spline / 算术 / 插值 / y_clamped_gradient | **float32** | 读 `splitCoord`（CPU 预拆分 `[ix,iy,iz,gx,gy,gz]` int32 格点 + float 小数）+ `perm` 表 |
| old_blended（InterpolatedNoiseSampler） | **float32**（shader 用 fp32 采样；原始为 fp64，CPU 拆分已把精度压进 7 值） | 读 `splitCoord` 7 值 `[ix,iy,iz,gx,gy(=h-n),gz,fadeY(=h)]` |

- `splitCoord`/`perm` 由 CPU 侧 `split()`/`collectPerm()` 预生成（GPU 预计算 split 是 D19 铁律的核心：格点 int32 + 小数 float，浮点误差在拆分侧吸收）。
- **C++ 采样函数绝不重新调 `normals[i].sample()` 做 fp64 采样**——那会给 NormalNoise 引入双精度路径，与 GPU 的 fp32 采样不一致。C++ 与 GPU 必须共用同一 `splitCoord`/`perm` 数据。

---

## 1. DF 节点类型分派枚举（eval_df 核心，C++ 的 dispatch 依据）

`dfc_gen.py` L211-215 定义，与 GLSL `eval_df`/`eval_df_base_N` 的 `if (t == ...)` 分支一一对应。**C++ 的 eval_df 就是复刻这张表中的 23 种运算**。

| type 值 | 符号名 | GLSL 运算 | 输入来源 | 类别 |
|---------|--------|-----------|---------|------|
| 0 | DF_CONSTANT | `r = CF0_[ci]` | 常量 `f0` | 叶子（字面量） |
| 1 | DF_Y | `r = float(iy)` | 块坐标 y | 叶子（坐标） |
| 2 | DF_NOISE | `r = normal_noise(NOISE_SLOT_BASE[a1] + corner*STRIDE[a1], sIdx)` | 噪声 | 叶子（数据驱动） |
| 3 | DF_OLD_BLENDED | `r = interp_noise(NOISE_SLOT_BASE[a1] + corner*STRIDE[a1], sIdx)` | old_blended | 叶子（数据驱动） |
| 4 | DF_SPLINE | `r = spline_eval(a1, corner, sIdx, ix, iy, iz)`（a2==1 时坐标 `(ix>>2)<<2,0,(iz>>2)<<2`） | spline | 叶子（显式栈） |
| 5 | DF_INTERP | `r = interp_N(sIdx, ix, iy, iz)`（a1 选 N） | interpolated | 嵌套调用（仅顶层闭包出现） |
| 6 | DF_ADD | `val[a1] + val[a2]` | 子节点值 | 算术 |
| 7 | DF_MUL | `val[a1] * val[a2]` | 子节点值 | 算术 |
| 8 | DF_MIN | `min(val[a1], val[a2])` | 子节点值 | 算术 |
| 9 | DF_MAX | `max(val[a1], val[a2])` | 子节点值 | 算术 |
| 10 | DF_ABS | `abs(val[a1])` | 子节点值 | 算术 |
| 11 | DF_SQUARE | `v*v`（v=val[a1]） | 子节点值 | 算术 |
| 12 | DF_CUBE | `v*v*v` | 子节点值 | 算术 |
| 13 | DF_HALF_NEG | `(v>0?v:v*0.5)` | 子节点值 | 算术（负半轴折半） |
| 14 | DF_QUARTER_NEG | `(v>0?v:v*0.25)` | 子节点值 | 算术（负半轴折四） |
| 15 | DF_SQUEEZE | `c=clamp(v,-1,1); c/2 - c³/24` | 子节点值 | 算术 |
| 16 | DF_CLAMP | `clamp(val[a1], f0, f1)` | f0/f1 常量 + 子节点 | 算术 |
| 17 | DF_RANGE_CHOICE | `(inp>=f0 && inp<f1) ? val[a2] : val[a3]` | f0/f1 + 子节点 | 算术（条件选择） |
| 18 | DF_Y_CLAMPED | `y_clamped_gradient(iy, f0,f1,f2,f3)` | 常量 | 叶子（坐标梯度） |
| 19 | DF_SHIFTED_NOISE | 同 DF_NOISE（`r = normal_noise(...)`） | 噪声 | 叶子（数据驱动） |
| 20 | DF_BLEND_DENSITY | `r = val[a1]` | 子节点 | 直通 |
| 21 | DF_FLAT_CACHE | `r = val[a1]` | 子节点 | 直通（flat_cache 为对齐坐标采样，见 §7） |
| 22 | DF_WEIRD | `d = ws_scale(f0, val[a1]); r = d * abs(normal_noise(NOISE_SLOT_BASE[a2]+corner*STRIDE[a2], sIdx))` | f0=kind + a2=ws噪声slot | 叶子（数据驱动） |

**注意**：GLSL 里 `SLOT_OF_*[CAx]` 是 val-stack 槽位映射（把全局 DF 节点索引 `a1/a2/a3` 重映射到当前闭包的线性槽位）。C++ 完全沿用它——`_compute_val_layout` 已算出 `top_slot`/逐 interp `slot`，gen_cpu 需把 `SLOT_OF_T/SLOT_OF_0..4` 及 `CTYPE/CA1/CA2/CA3/CF0..3` 一并导出。

**读数来源**（`_compute_val_layout` L422-424 `read_fields`）：type 6/7/8/9 读 a1+a2；10-16 读 a1；17 读 a1+a2+a3；20/21/22 读 a1。C++ eval_df 需按这些字段访问 `val[SLOT_OF_*[a1]]` 等。

---

## 2. 映射表（逐函数）

### 2.1 `eval_density` — 顶层入口

- **GLSL**：`float eval_density(int sIdx, int ix, int iy, int iz)`（L955）
- **逻辑**：包装器，直接 `return eval_df(20, 0, sIdx, ix, iy, iz);`。rootPos=20 是顶层闭包中根节点的槽位（`SLOT_OF_T[rootPos]` 处收结果），corner=0。
- **数据表**：无（仅转发）。
- **C++ 要点**：`float eval_density(int sIdx, int ix, int iy, int iz) { return eval_df(top_root, 0, sIdx, ix, iy, iz); }`。`top_root` = 顶层闭包根槽位（gen 从 `top_pos[n_nodes-1]` 得，此处样例=20，但**不得硬编码**，gen_cpu 必须导出 `top_root_pos`）。
- **复用**：新增（一层薄包装）。

### 2.2 `eval_df` — 顶层解释器（含 DF_INTERP 分支）

- **GLSL**：`float eval_df(int rootNode, int corner, int sIdx, int ix, int iy, int iz)`（L814）
- **逻辑**：**数据驱动**。后序节点数组（子节点先编号）顺序求值，每节点按 `CTYPE_T[ci]` 分派到 §1 的运算，结果写入 `val[PER_SAMPLE*sIdx + SLOT_OF_T[ci]]`。`DF_INTERP`(5) 分支调用 `interp_N`（a1=interp_idx）并把结果写槽后 `continue`。无递归、无虚调用、无显式栈（顺序直排）。顶层闭包 ~21 节点（单调用者 main，D16 防驱动强制内联）。
- **数据表**：`CTYPE_T/CA1_T/CA2_T/CA3_T/CF0_T..CF3_T/SLOT_OF_T/CLOSURE_T_LEN` + `NOISE_SLOT_BASE/STRIDE` + `PER_SAMPLE` + `valBuf`。
- **C++ 要点**：
  - `CTYPE_T[ci]` 等 → `const int CTYPE_T[K]` 或 `std::vector<int>`；per-闭包数组建议 `std::array`/`constexpr` 静态表（免堆分配，逐采样点复用）。
  - `valBuf` 语义：`PER_SAMPLE` 是每采样点槽数（CpuBackend 已有 `perSample=352`）。C++ 用 `std::vector<float>`；若批量采样，`sIdx` 偏移 `sIdx*perSample`；单点则 sIdx=0。
  - `abs/min/max/clamp` → `std::fabs/std::min/std::max/std::clamp`（注意 `std::clamp` 需 `<algorithm>`，且参数顺序 GLSL `clamp(x,a,b)` = C++ `std::clamp(x,a,b)` 一致）。
  - 算术分支直接读 `val[SLOT_OF_T[CAx]]`；`t==13/14` 的三元表达式必须保持浮点语义（`v>0.0f?v:v*0.5f`）。
- **复用**：`dbg_full_sim.py` `eval_df`（L374）是逐行忠实复刻，可对照；C++ 需新写。

### 2.3 `eval_df_base_{0..4}` — interp 角点 delegate 解释器（不含 DF_INTERP）

- **GLSL**：`float eval_df_base_N(int rootPos, int corner, int sIdx, int ix, int iy, int iz)`（0: L554, 1: L606, 2: L658, 3: L710, 4: L762）
- **逻辑**：**数据驱动**。与 `eval_df` 同构（顺序后序求值 + `CTYPE_N/CAx_N/CFx_N/SLOT_OF_N` 分派），但**无 DF_INTERP 分支**（角点 delegate 树不含 interpolated）。`corner`（0..7）作为运行时参数传给 `normal_noise/interp_noise/spline_eval` 用于查噪声实例索引。val 区段偏移 = `perSample*sIdx + BASE_N + corner*VAL_SLOTS_N`（BASE_N 来自 `_compute_val_layout` 的 `bases[]`）。
- **数据表**：`CTYPE_N/CA1_N/CA2_N/CA3_N/CF0_N..CF3_N/SLOT_OF_N/CLOSURE_N_LEN/VAL_SLOTS_N` + `NOISE_SLOT_BASE/STRIDE` + `PER_SAMPLE` + `valBuf`。BASE 偏移：N=0→0(即 8), N=1→152, N=2→208, N=3→256, N=4→304。
- **C++ 要点**：
  - 每 interp 的索引基址 `B + corner*VAL_SLOTS_N` 必须保留（`corner` 区分 8 角点的独立噪声/值，这是角点间唯一的运行时差异来源）。
  - `DF_SPLINE` 分支：`CAx==1` → `spline_eval(a1, corner, sIdx, (ix>>2)<<2, 0, (iz>>2)<<2)`（flat_cache 对齐坐标）；否则 `spline_eval(a1, corner, sIdx, ix, iy, iz)`。
  - 节点类型 t 从 §1 表按 `CTYPE_N[ci]` 查。5 个函数体结构相同，仅数组名/偏移/长度不同——**建议 gen_cpu 用模板或宏批量生成**（C++ 无函数名冲突问题，可直接复制 GLSL 结构）。
- **复用**：`dbg_full_sim.py` `eval_df_base`（L303）逐行复刻。C++ 需新写 5 个。

### 2.4 `spline_eval` — 显式栈后序求值（隐式递归的人工展开）

- **GLSL**：`float spline_eval(int rootNode, int corner, int sIdx, int ix, int iy, int iz)`（L421）
- **逻辑**：**显式栈**（GLSL 禁递归 D4）。帧 = `{node, i, coord, stage, v0, v1}`。stage 机：0=init(coord+二分+边界)，1=等 v0 子帧回填，2=瞬态（v0/v1 都齐后 Hermite），3=等 v1 子帧回填，4=左边界 v0 子帧回填补，5=右边界 vn 子帧回填补。复用 B1a 数据驱动 splineNodePack 表，**动态 node 索引运行时读表**（D21 根因修复：SSBO 表 → 驱动不展开）。
- **数据表**：`splineNodePack`(5字段/节点=coordType,n,locBegin,derBegin,valBegin)、`splineLocs`、`splineDers`、`splineValF`、`splineValKind`(0=const,1=nested)、`splineValNode`(nested 节点 idx)。均在 CpuBackend 已存在。
- **C++ 要点**：
  - 显式栈数组可用 `std::array` 或栈上定长 C 数组（GLSL 用 32；mvp 用 64）。**注意边界递归（D23）**：`i<0` 且 `valKind==1` 时压子帧（stage 4→`splineValNode[valB]`）；`i>=n-1` 且 nested 时 stage 5；正常区间 nested 压 stage 1/3。**外推端点 nested 必须递归求值而非 0.0**（D23 修复）。
  - `spline_hermite`（L412）= C++ 直接复刻。
  - `spline_coord`（§2.5）是每次 stage-0 帧的关键输入。
- **复用**：**`mvp_spline_eval.cpp` L177-248 `spline_eval` 已验证（vs 递归 maxDiff≈0）**，几乎逐行可搬（把 `NP`→`splineNodePack`、`SPLINE_*`→CpuBackend 成员、`spline_coord` 换成真实现（§2.5）即可）。**这是最高优先级复用**。`dbg_full_sim.py` `spline_eval_py`（L185）用 outSlot 返回地址模型，与 GLSL 的 stage 模型略有差异（更隐晦），**以 mvp 的 stage 模型为准**（它是 GLSL 的直译）。

### 2.5 `spline_coord` — coordType → 噪声实例 分派（核心复杂点）

- **GLSL**：`float spline_coord(int coordType, int corner, int sIdx, int ix, int iy, int iz)`（L389）
- **逻辑**：**数据驱动 + coordType 查表**。分两步：
  1. `slot = COORD_SLOT_TABLE[coordType]`；`v = normal_noise(NOISE_SLOT_BASE[slot] + corner*NOISE_SLOT_STRIDE[slot], sIdx)`。
  2. 按 coordType 施加 fold 包装（if(coordType==k) 特例）：样例 `COORD_SLOT_TABLE=[0,1,2,2]`，coordType0/1/3 = 恒等（`v=(v)`），coordType2 = ridge 折叠 `-3*(-1/3 + abs(-2/3 + abs(v)))`。
  - GLSL 用「运行时查 slot 表 + if(coordType) fold」而非 switch（D21 后 A5 修复：switch 让 slot 下标变编译期常量 → 驱动常量传播进 normal_noise → NORMAL_PACK 静态化 → 循环展开 → TDR。因此 **C++ 也应保持「运行时查表」形态，勿转成 switch 常量**）。
- **coordType → 噪声实例映射**（核心）：
  - `NOISE_SLOT_BASE[slot]` + `corner * NOISE_SLOT_STRIDE[slot]` → **噪声实例索引**（noise_instances 数组索引）。
  - `NOISE_SLOT_STRIDE`：角点独立（`is_corner=True`）slot 为 **1**（同一 slot 内 8 份连续实例，`base+c` 索引）；共享（`is_corner=False`，flat_cache 内或 interp 外）为 **0**。`_noise_slot`（L180-207）确定 base/stride。
  - **gen_cpu 必须导出 `NOISE_SLOT_BASE`/`NOISE_SLOT_STRIDE`/`COORD_SLOT_TABLE` 与每个 spline 节点的 `coordType`**。coordType 由 `_spline_coord_type`（L1119）为每个去重 coordinate 表达式分配（0..N-1），并将该节点的 coordType 写入 `splineNodePack[..+0]`。
  - 实际噪声实例 = `normal_noise` 的 `noiseIdx`，`NORMAL_PACK[NORMAL_INSTANCES]` 按实例索引全量填充（含 old 占位 0），`NORMAL_AMP_OFF` 每实例记录 amps 偏移。**不是 slot 直接对实例**：slot → base 再 +corner*stride。
- **C++ 要点**：
  - `COORD_SLOT_TABLE` + `NOISE_SLOT_BASE/STRIDE` + fold 特例（`if (coordType==k) v = ...`）。保持运行时查表，**不要展开 switch**。
  - fold 表达式（`_coord_folds`）由 gen 从 `spline_coords` 提取（用 `v` 替换 `normal_noise(...)` 子串）。gen_cpu 需导出每个 coordType 的 fold 表达式对应的 C++ 代码（或直接生成 fold 分支）。
- **复用**：`mvp_spline_eval.cpp` 的 `spline_coord`（L118）是**简化占位**（`0.1f*(ct+1)+0.001f*(iy+iz)`），**不可用于正确性**——必须替换为真实 `normal_noise` 版。`dbg_full_sim.py` `spline_coord_py`（L157）是正确参考（从 `g.spline_coords` 提取 slot + 按 `abs(` 判定 ridge fold），C++ 照此生成。

### 2.6 `spline_find_range` — 精确二分（vanilla MathHelper.binarySearch 复刻）

- **GLSL**：`int spline_find_range(float x, int locBegin, int n)`（L400）
- **逻辑**：**二分**（非递归、非数据驱动）
- **数据表**：`splineLocs`。
- **C++ 要点**：原样复刻，注意 `min/k/i` 为 int，比较 `x < splineLocs[locBegin+k]` 用 float。
- **复用**：`mvp_spline_eval.cpp` L125-135 已有（`const float* locs` 形参版），改传 CpuBackend `splineLocs.data()` 即可。

### 2.7 `normal_noise` — NormalNoise 数据驱动

- **GLSL**：`float normal_noise(int noiseIdx, int sIdx)`（L313）
- **逻辑**：**纯数据驱动**（139 函数 → 1）。读 `NORMAL_PACK[noiseIdx*3 + {n,octBase,splitBase}]`、`NORMAL_PACK_F[noiseIdx*2 + {persistence,amplitude}]`、`NORMAL_AMP_OFF[noiseIdx]`。两个 sampler（first/second，各 n octave）各累加 `Σ amps[ampOff+i]*pn_sample3_f32(octBase+i, ...)*persistence`（persistence 每 octave /2），返回 `(d + d2)*amplitude`。
- **数据表**：`NORMAL_PACK`(3int/实例)、`NORMAL_PACK_F`(2float/实例)、`NORMAL_AMPS`(全部 amps 连续)、`NORMAL_AMP_OFF`(每实例偏移) + `splitCoord` + `perm` + `SPLIT_TOTAL`。**注意**：`pn_sample3_f32` 的 octBase 是全局 octave 偏移（`octBase + i` 进 perm），splitBase 是拆分坐标偏移。
- **C++ 要点**：
  - 拆分坐标读取：`int b = sIdx*SPLIT_TOTAL + splitBase + i*6;`，从 splitCoord 读 `[b+0..b+5]` = `[ix,iy,iz,gx,gy,gz]`（float，int 格点先用 `int(...)` 截断）。**sIdx=0 单点时 offset 即 splitBase**。
  - perm：`mapPermD(octBase, v) = perm[(octBase*256 + (v&255))]`，perm 为 CpuBackend `collectPerm` 输出（`std::vector<uint32_t>`）。
  - `persistence/amplitude/amps` 都是 float；`f /= 2.0f`。
  - CpuBackend **没有** splitCoord/perm 成员——必须由 gen_cpu 补（存 member 或参数传入，见 §4）。
- **复用**：`dbg_full_sim.py` `normal_noise`（L85）+ `pn_sample3_f32`（L68）+ `gradDotF`（L61）+ `mapPermD`（L57）逐行复刻。C++ 需新写。

### 2.8 `old_blended` → `interp_noise` — 数据驱动

- **GLSL**：`float interp_noise(int idx, int sIdx)`（L351）
- **逻辑**：**纯数据驱动**（8 函数 → 1）。读 `OLD_PACK[idx*2 + {octBase,splitBase}]`。三段：interpolation 8 octave 累加（`octBase+32..39`），`qq=(n/10+1)/2`，`bl=qq>=1`、`bl2=qq<=0`，然后 lower 16 octave（`octBase+0..15`）与 upper 16（`octBase+16..31`）带早停的累加，混权 `w=clamp(qq,0,1)`，返回 `(l/512 + w*(m/512 - l/512))/128`。
- **数据表**：`OLD_PACK`(2int/实例) + `splitCoord`(7值/octave) + `perm`。
- **C++ 要点**：
  - `pn_section_f32(octBase_oct, sIdx, splitBase + oct*7)` 读 7 值 `[ix,iy,iz,gx,gy,gz,fadeY]`，其中 y 用 `perlinFadeF(fadeY)`（L89）。**读出并保留 fadeY 字段**。
  - 早停：`if (!bl) l += ...; if (!bl2) m += ...`（注意 `!bl` 与 `!bl2` 是两个独立早停，`o/=2` 每圈都执行）。
- **复用**：`dbg_full_sim.py` `interp_noise`（L130）+ `pn_section_f32`（L109）。C++ 需新写。

### 2.9 `interp_{0..4}` — 8 角点三线性插值

- **GLSL**：`float interp_N(int sIdx, int ix, int iy, int iz)`（0: L864, 1: L882, 2: L900, 3: L918, 4: L936）
- **逻辑**：**三线性**。cell 网格：`chunkX=floorDiv(ix,16)`，`chunkZ=floorDiv(iz,16)`；`gx=ix-chunkX*16, gy=iy-minY, gz=iz-chunkZ*16`；`cx=gx/4, cy=gy/8, cz=gz/4`；frac `fx=gx%4/4, fy=gy%8/8, fz=gz%4/4`。8 角点各调 `eval_df_base_N(rootPos, c, sIdx, ax, ay, az)`（ax/ay/az = chunkX*16+(cx+dx)*4, minY+(cy+dy)*8, chunkZ*16+(cz+dz)*4），再沿 x/y/z 三次 lerp。
- **数据表**：经 `eval_df_base_N` 间接用全部 DFC 表 + `minY`。
- **C++ 要点**：
  - `floorDivP`（CSS 语义 floor，负坐标）C++ 已有 `CpuBackend::floorDiv`（static，L31）。`minY=-64`。
  - 8 角点根节点 rootPos：N=0→133, 1→20, 2→19, 3→16, 4→17（**由 gen 导出，勿硬编码**——`interp_root_pos`）。
  - 插值链：`d00 = d000+(d100-d000)*fx` 等，顺 x→y→z。
- **复用**：`dbg_full_sim.py` `interp_N`/`interp_0`（L349/L371）参考。C++ 需新写 5 个（结构相同，仅 rootPos 不同）。

### 2.10 辅助采样函数（被上述引用的底层原语）

| GLSL | 逻辑 | 数据表 | C++ 要点 / 复用 |
|------|------|--------|------------------|
| `mapPermD(int octBase,int v)=imm.perm[octBase*256+(v&255)]` | perm 表索引 | perm | `perm[(octBase*256 + (v&255))]`，v&255 用 `v & 255`（int）。`dbg_full_sim` L57 |
| `gradDotF(int hash,float x,y,z)` | 梯度点积（16 方向 GRADIENTS[hash&15]） | GRADIENTS | `const double GRADIENTS[16][3]` 保持（shader 从 double 转 float）；`g.x*x+...`。`dbg_full_sim` L61 |
| `perlinFadeF(v)=v³(v(v6-15)+10)` | fade | — | 原样。`dbg_full_sim` L64 |
| `lerpF(d,s,e)=s+d*(e-s)` | lerp | — | 原样。`dbg_full_sim` L66 |
| `pn_sample3_f32(octBase,sx,sy,sz,lx,ly,lz)` | 单 octave float perlin | perm, GRADIENTS | 8 角点 grad + fade + 2 层 lerp；`int i=mapPermD(octBase,sx)` 链。`dbg_full_sim` L68 |
| `pn_section_f32(octBase,sIdx,splitOffset)` | old_blended 用 5 参数采样 | splitCoord(7值) | fade 用 `perlinFadeF(fadeY)` 于 y。`dbg_full_sim` L109 |
| `y_clamped_gradient(int y,fromY,toY,fromV,toV)` | 线性梯度 clamp | — | `t=clamp((y-fromY)/(toY-fromY),0,1)`. 原样 |
| `ws_scale(int kind,float v)` | scaleValue 分段 | — | CpuBackend 已有 **double** 版 `ws_scale`(L277)；shader 用 **float** 版。gen_cpu 需补 float 版（或模板），确保 eval 用 float、split 用 double 各就各位 |
| `floorDivP(int a,int b)` | floor 除法 | — | 复用 `CpuBackend::floorDiv`（L31） |
| `spline_hermite(...)` | Hermite 插值 | — | `mvp_spline_eval.cpp` L137-142 已有 |

---

## 3. 递归 / 数据驱动 / 显式栈 分类汇总（供性能与实现选型）

| 函数 | 类别 | 备注 |
|------|------|------|
| `eval_density` | 包装 | 无 |
| `eval_df` | 数据驱动（顺序直排） | 无递归、无虚调用、无显式栈 |
| `eval_df_base_N` | 数据驱动（顺序直排） | 同上，含 corner 参数 |
| `spline_eval` | **显式栈**（人工展开递归） | 需 stage + 栈数组；D23 边界递归 |
| `spline_coord` | 数据驱动（查表 + fold） | 核心：coordType→槽→噪声实例 |
| `spline_find_range` | 二分 | 非递归 |
| `normal_noise` | 数据驱动（循环累加） | n octave × 2 sampler |
| `interp_noise` | 数据驱动（三段循环） | 带早停 |
| `interp_N` | 顺序（8 角点插值） | 调 eval_df_base_N |

> DFC 目标（无递归/无虚调用）判定：**spline_eval 是唯一在 GPU 上是显式栈的**（原 vanilla SplineDF 递归/虚调用是 11× 延迟根因）；其余全部数据驱动顺序求值。C++ 移植只要 `spline_eval` 保持显式栈，其余保持顺序直排，即达成 DFC 目标。

---

## 4. CpuBackend 数据表现状 + gen_cpu 需补生成的最小集合

### 4.1 已存在（gen_cpu L1649-1854 已生成，勿重复）

| 成员 / 函数 | 用途 |
|------------|------|
| `splineNodePack/Locs/Ders/ValF/ValKind/ValNode` + `splineNodes` | spline_eval 数据驱动表（56 节点，pack 5字段/节点） |
| `normals`(DoublePerlinNoiseSampler), `n/octBase/splitBase` vectors | 生成 split 的采样器 + 每实例元数据 |
| `oldBlendeds`, `oldBase/oldSplitBase` | old_blended 采样器 + offset |
| `shiftNoises` map | shift 噪声 |
| `splitTotal=8672`, `permSize=356352`, `perSample=352` | 缓冲尺寸 |
| `split(x,y,z,out)` | 写单点 splitCoord（8672 float）到 `out` |
| `collectPerm(perm)` | 填 perm 表（permSize 个 uint32） |
| `floorDiv/minY/maintainPrecision/splitDouble/split7/splitOldBlended/ws_scale(double)` | 静态工具 |

### 4.2 需 gen_cpu **补生成**（C++ 采样目前缺失）

**A. 数据表（const 数组，镜像 GLSL const）——gen_cpu 必须新增导出：**
1. DF 节点数组：`DF_TYPE/DF_A1/DF_A2/DF_A3/DF_F0/DF_F1/DF_F2/DF_F3`（长度 DF_NODES）+ `top_root_pos`（顶层闭包根槽位）+ `PER_SAMPLE`。
2. 每 interp 闭包表：`CLOSURE_N_LEN/VAL_SLOTS_N/CTYPE_N/CA1_N/CA2_N/CA3_N/CF0_N..CF3_N/SLOT_OF_N`（N∈{0..4}）+ 顶层 `CLOSURE_T_LEN/CTYPE_T/CA1_T/CA2_T/CA3_T/CF0_T..CF3_T/SLOT_OF_T` + `interp_root_pos[N]`（8 角点根）。
3. 噪声 slot 表：`NOISE_SLOT_BASE/NOISE_SLOT_STRIDE/NOISE_SLOT_COUNT`（`_noise_slot_table_glsl` L1490）。
4. `COORD_SLOT_TABLE[#coordType]` + 每 coordType 的 fold 表达式（`_spline_coords`/`_coord_folds`，L1225-1253）。
5. `NORMAL_PACK/NORMAL_PACK_F/NORMAL_AMPS/NORMAL_AMP_OFF/NORMAL_INSTANCES`（`_normal_noise_glsl` L1925）。
6. `OLD_PACK/OLD_INSTANCES`（`_old_blended_glsl` L1863）。
7. `GRADIENTS[16][3]`（double）、`SPLINE_NODES`、`SPLIT_TOTAL`。

**B. 采样函数（gen_cpu 本轮真正要新增的主体）：**
- 原语：`floorDivP`(=floorDiv)、`perlinFadeF`、`lerpF`、`gradDotF`、`pn_sample3_f32`、`pn_section_f32`、`y_clamped_gradient`、`ws_scale(float)`、`mapPermD`。
- 数据驱动：`normal_noise`、`interp_noise`。
- spline：`spline_coord`、`spline_find_range`、`spline_hermite`、`spline_eval`。
- 解释器：`eval_df_base_{0..4}`、`eval_df`、`interp_{0..4}`、`eval_density`。

**C. 运行时缓冲（关键缺口）：**
- CpuBackend 现在 `split()` 把单点写进外部 `out`（float*），`collectPerm` 填外部 `perm`。**C++ 采样函数需要这两个缓冲**。gen_cpu 建议：
  - 添加成员 `std::vector<float> splitCoord; std::vector<uint32_t> perm;`
  - 提供 `void prepare(int x,int y,int z)` 或让 `split()`/`collectPerm()` 直接填 member，并提供 `eval(x,y,z)` 入口对单点走 sIdx=0。
  - 或保持「外部缓冲 + 传入」接口，`eval_density(sIdx,ix,iy,iz)` 读 `splitCoord + sIdx*splitTotal`。两者选一，推荐前者（封装 + 单点最常见）。

---

## 5. C++ 复用优先级（哪些可直接搬 / 哪些必须新写）

| 优先级 | 函数 | 来源 | 说明 |
|--------|------|------|------|
| **P0 直接搬** | `spline_eval`, `spline_find_range`, `spline_hermite` | `mvp_spline_eval.cpp` | 显式栈算法已验证（maxDiff≈0 vs 递归）；改表引用 + 真 spline_coord 即可 |
| **P0 直接搬** | `spline_coord`, `normal_noise`, `interp_noise`, `pn_sample3_f32`, `pn_section_f32`, `gradDotF`, `mapPermD`, `eval_df`, `eval_df_base_N`, `interp_N`, `eval_density`, `ws_scale` | `dbg_full_sim.py` | Python 全量复刻与 GPU 同源、逐行对准（maxDiff=5.7e-9），是精确的「机械翻译」蓝本 |
| **P1 结构复用 + 参数化** | `eval_df_base_{0..4}`, `interp_{0..4}` | gen_cpu 批量生成 | 5 个函数体同构，仅数组名/偏移/root 不同；gen_cpu 循环生成（相似于 GLSL gen_shader 的循环） |
| **P1 必须重写** | `spline_coord`（真实现） | mvp 的版本是简化占位 | 替换为 `COORD_SLOT_TABLE + NOISE_SLOT_BASE/STRIDE + fold` 版 |
| **P1 需补** | `ws_scale(float)` | CpuBackend 现只有 double | 补 float 版（或模板化），与 split 的 double 版区分 |
| **P2 需 gen_cpu 补导出** | 全部 const 数据表 + perSample/splitTotal/perm/splitCoord 缓冲 | gen_cpu | 见 §4.2 |

### 参考原型文件
- `mvp_spline_eval.cpp`（C++ 显式栈 spline_eval + 递归 + 虚调用递归，已验证结果一致 + 性能扫描）：spline_eval/spline_find_range/spline_hermite/spline_recursive 可用。**注**：其 `spline_coord` 为简化占位，**不能用于正确性**（注释明示「先验算法」，真实 coord 第 2 步接）。
- `dbg_full_sim.py`（Python 全量复刻 DFC 解释器）：`eval_df/eval_df_base/spline_eval_py/spline_coord_py/normal_noise/interp_noise/interp_N/pn_sample3_f32/pn_section_f32/gradDotF/mapPermD/ws_scale_py` —— **最贴近 GLSL 的逐行蓝本**。其 `spline_eval_py` 用 outSlot 返回地址模型（与 GLSL stage 模型略异），以 mvp 的 stage 模型为准。

---

## 6. gen_cpu 扩展的实现建议

1. 新增 `gen_cpu_sampling()`（或并入 `gen_cpu`）：在 C++ struct 内生成 A 表 + B 函数 + C 缓冲，作为 `CpuBackend::eval(int x,int y,int z)` 的采样内核。
2. 常数表用 `constexpr`/`static const`（或 `std::array`），逐采样点复用，避免堆分配。
3. `eval_density` 对外 API：`float sample(int sIdx, int ix, int iy, int iz)`。单点常用路径 `sIdx==0`；批量（GPU e2e 等价）`sIdx=点序`。
4. `spline_coord` 的 fold：gen 已把 `_coord_folds`（`coordType==k → v = <expr>`）准备好，gen_cpu 按它生成 `if(coordType==k) v = ...;` 行（**保持运行时查表，勿转 switch**——D21 A5 教训）。
5. 验证：移植后与 `dbg_full_sim.py` 全量对拍（目标 maxDiff ≤ 1e-6，与 e2e GPU 同量级），再上 block_probe Full 对拍（maxDiff=3.128e-07 基线）。

---

## 7. 关键易错点（逐位对齐红线，移植时必须保留）

- **`spline_eval` 边界嵌套递归（D23）**：`i<0` 或 `i>=n-1` 且端点 valKind==1 → 必须求嵌套值，**不是 0.0**。
- **`normal_noise` 的 `persistence` 每 octave `/2`**：`f = persistence; ... f /= 2.0f;`，**不要**把它当 `persistence/octave` 固定。
- **`interp_noise` 早停**：`if(!bl) l+=...; if(!bl2) m+=...; o/=2;` —— 两个独立早停，除法每圈执行。
- **`pn_section_f32` y-fade 用 `fadeY`（第 7 值）而非 `gy`**：`perlinFadeF(fadeY)`。
- **`COORD_SLOT_TABLE` 查表而非 switch**：防驱动常量传播 TDR（D21 A5）。
- **floor 除法**：`floorDivP`/`CpuBackend::floorDiv`（负坐标），**不要用 C++ `a/b` 截断**。
- **`mapPermD` 的 `v & 255`**：int 保留低位。
- **`GRADIENTS` 为 double，`gradDotF` 转 float**：与 GPU 一致（浮点精度来源）。
- **`perSample`（352）与 val 区段偏移（B + corner*VAL_SLOTS_N）**：C++ 必须与 `_compute_val_layout` 输出一致，防越界（D19 教训）。
- **`minY=-64`**：interp cell 网格 y 基准。
