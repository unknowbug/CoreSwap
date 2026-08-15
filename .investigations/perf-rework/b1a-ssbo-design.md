# B1a 设计注记：spline SSBO 数据驱动化（dfc_gen.py 改造方案）
# 2026-08-14，基于 analyze_spline_hook.py 统计

## 数据结构（实测）
- 56 个 spline 节点；coordinate 仅 4 种 registry 引用（ridges_folded x32 / ridges x11 / erosion x10 / continents x3）
- value 仅 2 种：const 209 + nested_spline 120（**无 other_df** —— value 不是常量就是嵌套 spline，SSBO 可紧凑）
- locs 大小 2..11（avg 4.4）；嵌套链深度 ≤4（父→子→孙→曾孙）
- 当前每个 spline 生成 1 个 GLSL 函数（`spline_N(sIdx,ix,iy,iz)`），函数体内 if-else 链（n-1 分支）

## 目标
56 个 spline 函数 → **1 个 `spline_eval(nodeIdx, coord)` 单函数**（显式栈模拟后序求值），消除函数嵌套。

## SSBO 布局（新增 3 个只读 buffer，set=0 binding=5/6/7）
```
struct SplineNode { int coordType; int n; int locBegin; int derBegin; int valBegin; };  // coordType: 0..3
layout(set=0, binding=5, std430) buffer SplineNodeBuf { SplineNode splineNodes[]; };
layout(set=0, binding=6, std430) buffer SplineDataBuf { float splineData[]; };   // locs 然后 ders 连续拼
layout(set=0, binding=7, std430) buffer SplineValBuf {
  // value 记录：kind(0=const, 1=nested), value(常量值 or 嵌套节点索引)
  int splineValKind[];
  float splineValF[];
  int splineValNode[];
};
```

## 单函数实现（GLSL）
```
float spline_eval(int nodeIdx, float coord) {
    // 显式栈：帧 = {node, i(区间), stage, v0, v1, outSlot}
    // stage: 0=求 coord/二分; 1=等 v0; 2=等 v1 完成 Hermite
    // 简化：因为 Hermite 需要 v[i] 与 v[i+1] 两个值，且 value 可能嵌套，
    // 用「值引用栈」：先求值左 value，再求值右 value，然后 Hermite。
    // 实现：递归展开为迭代 —— 后序遍历所有需要的 value 节点。
}
```

## 关键设计决策
1. **coordinate 4 种** → `float spline_coord(int coordType, int sIdx, int ix, int iy, int iz)`：
   `switch(coordType) { case 0: return df_overworld_ridges_folded(...); ... }`
   （registry 函数已是独立函数，调用它们不增加嵌套深度——它们深度固定 1 层）
2. **value 2 种** → kind=0: `splineValF[i]`；kind=1: 递归求 `spline_eval(splineValNode[i], childCoord)`。
   **注意**：嵌套 spline 的 child coordinate 是**它自己的 coordinate**（如 erosion 的 coordinate=erosion 噪声），
   不是父的 coord —— 所以 spline_eval 内部对每个节点先算自己的 coordinate。
3. **后序求值**：Hermite 区间 [i,i+1] 需要 v[i] 和 v[i+1]。若 value 是嵌套 spline → 需先求子 spline。
   显式栈帧：
   ```
   struct Frame { int node; int i; int stage; float v0; float v1; };
   Frame stack[16]; int sp = 0;
   ```
   - stage 0：算 coord → 二分 i → 边界直接出值（外推）→ 否则压帧 stage=1（等 v0）
   - stage 1：求 v[i]（常量直接取；嵌套则压子帧，算完写 v0）→ stage=2
   - stage 2：求 v[i+1] → Hermite 出值
   - 子帧完成时把结果写回父帧的 v0/v1 槽 + 恢复父帧 stage
4. **二分**：`findRange` = vanilla `MathHelper.binarySearch` 精确复刻（`min=0,i=len; while(i>0){j=i/2,k=min+j; if(x<locs[k])i=j; else{min=k+1;i-=j+1;}} return min-1;`）
5. **边界外推**：i<0 → `vals[0]+ders[0]*(coord-locs[0])`；i>=n-1 → `vals[n-1]+ders[n-1]*(coord-locs[n-1])`
6. **Hermite**：`lerpF(kd,nv,ov)+kd*(1-kd)*lerpF(kd,p,q)`（复用现有 spline_seg 或内联）

## 与现实现的对齐点
- `_gen_spline` 收集逻辑保留（spline_cache 去重 / 嵌套识别 / coord 表达式生成），只改「输出形态」：
  - 不再 append 到 `spline_funcs`（生成 per-spline 函数）
  - 改为收集 `self.spline_ssbo = {nodes:[], locs:[], ders:[], vals_kind:[], vals_f:[], vals_node:[]}` +
    `self.spline_coords = [coordExpr per coordType]`（去重后 4 个）
- `_spline_body` 删除；gen_shader 输出 `spline_coord` + `spline_eval` + SSBO 声明
- **CPU 后端 gen_cpu 不受影响**（spline 求值在 CPU 参照 density_builder.h，生成器只出 GPU shader）
- 调用点 `spline_N(sIdx,ix,iy,iz)` → `spline_eval(nodeIdx, spline_coord(coordType,sIdx,ix,iy,iz))`

## 风险
- R1 显式栈语义（后序/帧恢复）——对照 if-else 链版逐点对拍（生成同样输入输出）
- R2 coordinate 的 flat_cache 对齐上下文（gen_with_coords）——coord 表达式在 gen 时已带坐标上下文，spline_coord 内保持
- R3 栈深 ≤4 层 × value 2 个 = ≤8 帧，定长 16 足够
- R4 精度：求值顺序（v0 先 v1 后）与 if-else 链一致（先算 nv 再算 ov）→ 舍入一致
