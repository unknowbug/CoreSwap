# D1 设计：DF 节点函数化重构（镜像 C2ME newMethod/callDelegate，根治 68KB 展开）

> 2026-08-14。前置：G1-G6 错误记录（gpu-accel-errors.md）+ coreswap-vs-c2me.md 六.1（形态趋同说明）。
> 目标：interp 8 角点内联展开（68KB）→ 节点函数化（每个 DF 节点 1 个函数 + 子节点调用），
>       驱动编译时间 >10min → 秒级；精度分层（宏观 F64 + 高频 F32）不变。

## 一、问题本质（G1-G6 教训）

- 驱动编译时间主因 = **单函数体的表达式复杂度**（巨型展开树），不是函数数。
- interp_0/5 函数体 68KB = 8 角点 × init 树全展开（152 y_clamped_gradient + 288 spline_eval + 592 normal_noise 调用）。
- 展开根因 = 生成器 `gen()` 递归把 DF 树展开成表达式字符串（每层复制父表达式）。
- spline/normal 已数据驱动化（56→0、139→1），但 **interp 角点的 delegate（init 树）仍整棵展开**。

## 二、C2ME 的机制（参考，不照抄精度）

`OpenCLCGenContext.newMethod(node)`：每个 DF 节点生成独立 OpenCL 函数（节点类型分派），
子节点用 `callDelegate(target)` 调用（不展开到调用点）。interpolated 用网格预填充两阶段：
- `df_interpolator_buffer_prefill_<name>`：按 cell 网格 dispatch，每工作项算 1 个角点的 delegate 采样 → buffer
- 主内核读 buffer 三线性插值

## 三、我们的 D1 方案（节点函数化，保持精度分层）

### 3.1 核心改动：gen() 从「返回表达式」→「注册节点函数 + 返回调用」

改造前：`gen(df)` 递归返回 GLSL 表达式字符串（整树展开）。
改造后：`gen(df)` 递归注册「节点函数」+ 返回 `df_N(sIdx, ix, iy, iz)` 调用。

- **节点函数注册表** `self.df_funcs = {}`：`df 结构 json -> df_N`
- **每个 DF 节点**（含 registry 引用、算术、spline、noise 调用、interpolated 调用）生成一个函数：
  ```glsl
  float df_N(int sIdx, int ix, int iy, int iz) {
      float x = float(ix), y = float(iy), z = float(iz);
      return <该节点的表达式，子节点用 df_M(...) 调用>;
  }
  ```
- **共享去重**：相同结构节点（json key）复用同一函数（registry 多处引用 → 1 个函数）
- **interpolated 角点**：不再展开 delegate 树，只调用 delegate 根函数：
  ```glsl
  // 角点采样（仍内联在 interp 函数里，但 delegate 是 1 次函数调用）
  float d000 = df_delegate(sIdx, chunkX*16+(cx+0)*4, minY+(cy+0)*8, chunkZ*16+(cz+0)*4);
  ```

**关键效果**：interp 函数体 = 8 个 `df_delegate(...)` 调用 + 三线性插值（~200 字符），
delegate 树本身变成 df_N 函数链（每节点小函数），**无 68KB 巨型展开**。

### 3.2 节点函数的大小控制

- **算术节点**（add/mul/min/max/clamp/range_choice/abs/square/squeeze/blend_*）：函数体 = 子节点调用 + 运算符 → 小
- **y_clamped_gradient**：函数体 = 1 行 → 小
- **spline**：已数据驱动（spline_eval 单函数 + const 数组）→ 节点函数只调 spline_eval → 小
- **normal_noise**：已数据驱动（normal_noise(idx, sIdx)）→ 节点函数只调它 → 小
- **old_blended**：interp_noise_N 已函数化 → 节点函数只调它 → 小
- **registry 引用**：df_overworld_xxx 已是函数 → 节点函数调它 → 小
- **interpolated**：节点函数 = 8 角点 delegate 调用 + 插值 → 小（delegate 是函数调用）

**预期**：所有节点函数体 ≤ 几百字符，整个 shader ~30-50 个节点函数 + 数据表，编译秒级。

### 3.3 精度分层不变

- 节点函数内的表达式按原有精度语义生成（old_blended → double 采样转 float；其余 float）。
- 坐标上下文（flat_cache 对齐 `(ix>>2)<<2`）在节点函数调用参数里体现（调用点传对齐坐标）。

### 3.4 坐标上下文处理（关键难点）

原 `gen()` 用 `self.cx/cy/cz`（flat_cache 深度切换）控制坐标变量。节点函数化后：
- 节点函数签名 `(sIdx, ix, iy, iz)`，内部 `x=float(ix)` 等。
- **flat_cache 对齐**：调用点传对齐坐标（`df_N(sIdx, (ix>>2)<<2, 0, (iz>>2)<<2)`），不改变函数内部。
- **interpolated 角点**：调用点传角点 block 坐标（`chunkX*16+(cx+dx)*4` 等）。
- 原 `gen_with_coords` 的「坐标上下文切换」改为「调用点显式传坐标参数」。

### 3.5 与 gen_cpu 的关系

- gen_cpu（CpuBackend 拆分逻辑）**不动**——它生成 CPU 侧拆分/perm，与 GPU 节点函数化无关。
- gen_shader 输出形态变（节点函数链），但输入（DF JSON）和输出语义（density 值）不变。

## 四、实施步骤

1. `gen()` 重构：注册节点函数 + 返回调用（保留现有分支逻辑，改输出形态）
2. interp 分支：8 角点调 delegate 根函数（不再展开）
3. gen_shader：输出节点函数链 + 工具函数 + 数据表（spline/normal 已数据驱动）
4. 验证：glslc + 函数体大小（interp ≤ 几百字符）+ pipeline 计时（目标秒级）
5. 精度回归：dfc_final_backend_e2e maxDiff vs 基线（~1e-7）

## 五、风险

- R1 节点函数数失控（每算术节点 1 函数 → 数百函数）——函数体小（≤几百字符）所以编译 OK（noodle 44 函数 1.6KB 体 2.4s 证明小函数体不慢）。可后续合并纯算术链。
- R2 坐标上下文传递错误（flat_cache/interp 角点）——调用点显式传坐标，单测对齐。
- R3 gen_cpu 依赖 gen() 的表达式输出——检查 gen_cpu 是否复用 gen()（若是，需独立处理）。

## 六、验证标准

- interp 函数体从 68KB → ≤ 500 字符
- 全 shader 函数体均 ≤ 500 字符（或可接受）
- pipeline 编译时间从 >10min → ≤ 5s（目标）
- e2e maxDiff 不回退（~1e-7 量级）

## 七、实施结果（2026-08-14，未达标）

### 已达成的
- **interp 函数体 69868 → 950 chars**（↓98.6%）✅
- **最大 df_N body ≤ 307 chars**，spline_eval 3835（数据驱动解释器，固定）✅
- glslc 编译通过 ✅

### 未达成的（核心教训）
- **函数数爆炸到 ~300**（interp 角点 × 每节点一函数），`vkCreateComputePipelines` 仍 >10min ❌
- **结论：驱动编译时间 = f(函数数, 嵌套调用深度) 双维度**——函数体大小优化了但函数数爆炸，两者都要小（对照 noodle 44 函数 × 1.6KB → 2.4s）
- 「每节点一函数」仍是展开式的变体——**正确形态 = 每类型一函数 + 数据 buffer**（C2ME const_data 机制：add/mul/min 各 1 个函数，节点数据在 buffer，函数数 = 类型数 ~15）

### 实施中踩的坑（详细记录见 gpu-accel-errors.md D5-D9）
- D5 函数数爆炸（300 个仍慢）
- D6 GLSL 依赖序（registry 引用 df_N → 全部前向声明）
- D7 坐标上下文（节点函数化后坐标必须用形参，不能靠 self.cx/cy/cz 切换）
- D8 递归注册 idx 冲突（先占位后回填）
- D9 GPU 角点去重与 CPU 每角点拆分模型冲突（jagged@c1 缺失，遗留）

### 下一步方向（未实施）
- **节点类型分派 + 数据 buffer**（DF 树解释器）：每算术类型一个函数，节点数据放 buffer → 函数数 ~20
- 或 **接受 300 函数 + 拆 shader**（每 shader <80 函数）+ VK_KHR_pipeline_binary 分发
