# D2 设计：DF 树解释器（节点类型分派 + 数据 buffer）

> 2026-08-14。前置：D5 教训（编译时间 = f(函数数, 函数体) 双维度，两者都要小）。
> D1 节点函数化失败原因：函数数爆炸 ~300（每实例一函数）。D2 = 每类型一函数 + 节点数据 buffer。

## 一、目标

- 函数数：~300 → **~40**（节点类型 ~20 + 噪声采样/spline_eval/interp + 工具）
- 函数体：全部 ≤ ~2KB
- 编译时间：>10min → **秒级**
- 精度分层（宏观 F64 + 高频 F32）不变；spline/normal 数据驱动复用

## 二、核心：DF 树 → 节点数组 + eval_df 显式栈解释器

### 2.1 节点序列化（Python 侧 gen_df）

DF 树递归遍历 → 节点数组（每个节点固定槽位），子节点索引引用：

```
节点类型枚举（int）：
  CONSTANT=0, Y=1, NOISE=2, OLD_BLENDED=3, SPLINE=4, INTERPOLATED=5,
  ADD=6, MUL=7, MIN=8, MAX=9, ABS=10, SQUARE=11, CUBE=12,
  HALF_NEG=13, QUARTER_NEG=14, SQUEEZE=15, CLAMP=16,
  RANGE_CHOICE=17, Y_CLAMPED=18, SHIFTED_NOISE=19, BLEND_DENSITY=20

节点数据（const 数组）：
  nodeType[idx]           # 类型
  nodeArg1/2/3[idx]       # 子节点索引（-1=无）；叶子语义不同（见下）
  nodeF0..3[idx]          # float 参数（constant 值 / clamp min max / yclamp 4 参 / range 阈值）
```

叶子节点 arg 语义：
- CONSTANT: f0=值
- Y: 无（返回 iy 形参）
- NOISE: arg1=noise_idx（normal_noise(noise_idx, sIdx)）
- OLD_BLENDED: arg1=noise_idx（interp_noise_N）
- SPLINE: arg1=spline_node_idx（spline_eval(node, sIdx, ix, iy, iz)）
- INTERPOLATED: arg1=interp_idx（interp_N）
- Y_CLAMPED: f0..3 = from_y/to_y/from_v/to_v，输入 iy

算术节点 arg 语义：
- ADD/MUL/MIN/MAX: arg1, arg2 = 子节点
- ABS/SQUARE/CUBE/HALF_NEG/QUARTER_NEG/SQUEEZE/BLEND_DENSITY: arg1 = 子节点
- CLAMP: arg1=子节点, f0=min, f1=max
- RANGE_CHOICE: arg1=input, arg2=when_in, arg3=when_out, f0=min_inclusive, f1=max_exclusive
- SHIFTED_NOISE: 同 NOISE（shift 在 CPU 侧）
- FLAT_CACHE: arg1=delegate（**对齐坐标特殊处理**，见 2.3）
- cache_2d/cache_once/cache_all_in_cell: 剥掉（arg1=delegate 直接引用）

### 2.2 eval_df 显式栈求值（GLSL 侧）

DF 树是固定 arity 的树（无递归），显式栈后序求值：

```glsl
float eval_df(int rootNode, int sIdx, int ix, int iy, int iz) {
    // 栈：nodeIdx + coordX/Y/Z（flat_cache 会切换坐标）+ stage
    int nodeStack[64]; int stageStack[64];
    int cxStack[64], cyStack[64], czStack[64];
    float valStack[64]; int valTop = 0;
    int sp = 0;
    push(rootNode, ix, iy, iz, 0);
    while (sp >= 0) {
        node = nodeStack[sp];
        t = nodeType[node];
        switch (t) {
        case CONSTANT: result = nodeF0[node]; pop; push_val(result); break;
        case Y: result = cyStack[sp]; pop; push_val(result); break;
        case NOISE: result = normal_noise(nodeArg1[node], sIdx); pop; push_val(result); break;
        case ADD: /* 子节点先求值，后序 —— 见 stage 机制 */
        ...
        }
    }
    return valStack[0];
}
```

**显式栈后序**：算术节点需要「子节点先求值」。帧 = {node, stage}：
- stage 0：若是叶子直接出值；若是算术，压子节点（先压 arg2 再压 arg1，后序）
- 子节点出值后，父节点 stage 递增，等所有子节点值齐 → 运算

**FLAT_CACHE 坐标切换**：flat_cache 节点的帧记录「对齐坐标 (ix>>2)<<2, 0, (iz>>2)<<2」，压 delegate 子节点时传对齐坐标。栈帧携带坐标（cxStack/cyStack/czStack）。

### 2.3 关键难点与对策

1. **GLSL 无递归**（D4）→ 显式栈（与 spline_eval 同法，已验证可行）
2. **坐标上下文**（D7）→ 栈帧携带坐标，flat_cache 切换坐标在栈帧内完成（不靠全局变量）
3. **CPU/GPU 对齐**（D9）→ gen_cpu 独立维护（旧 gen() 收集），GPU eval_df 独立序列化；splitCoord 布局由「噪声节点在树中的遍历序」决定，两侧遍历序一致即可
4. **函数数控制**（D5）→ 每类型 1 函数，节点数据在 const 数组，函数数 = 类型数

## 三、实施步骤

1. `gen_df()`：DF 树 → 节点数组（后序，子节点先编号）+ 去重（相同子树结构共享节点）
2. `eval_df_glsl()`：节点 const 数组 + eval_df 显式栈解释器 + 类型分派
3. gen_shader 改用 gen_df + eval_df（替代 gen_node + 300 df_N）
4. 验证：函数数 ~40 + 函数体 + 编译时间
5. 精度回归 + gen_cpu 对齐（D9 遗留）

## 四、验证标准

- 函数数 ≤ ~50
- 函数体 ≤ ~2KB
- 编译时间 ≤ 5s
- e2e maxDiff ~1e-7 不回退
