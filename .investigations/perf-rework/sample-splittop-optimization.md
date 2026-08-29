# sample() splitTop 优化（dfc_gen.py gen_cpu_sampling）——避免 grid 命中时每点整树 split()

> worker：perf-rework DFC 性能优化 worker（只改 dfc_gen.py 生成器，不编译）。
> 状态：**candidate**（静态自检通过，未编译/未运行 benchmark——由主会话 gen_final_density.py + 编译验证）。
> 目标：`CpuBackend::sample(x,y,z)` 在 grid 缓存命中（同 chunk）时不再每点整树 `split()`，
>       只喂非 interp 所需的最小拆分（interp 走 grid），显著降低每点 split 开销，使整 chunk 生成不超时。

---

## 1. 根因（源码级确认）

- `sample(x,y,z)`（生成 L1047-1051）每点 `split(x,y,z, splitCoord.data())` —— **整树 8 角点展开（8672 floats）**。
- `split()`（gen_cpu L1837-1839）生成体 = **仅 5 个 interpolated 块，每块展开 8 角点**（实测 200 条 split-call 行，
  全部引用 cell 角点坐标 `_chunkX*16 + (_cx+Δ)*4`，**无任何裸坐标顶层噪声行**）。
- `eval_density(sIdx=0)` → `eval_df(TOP_ROOT, 0,...)` 的非 interp 路径只**读 @c0**（`NOISE_SLOT_BASE[a1]`，
  无 `+corner*stride`）；interp 由 `interp_N(sIdx==0)` → `sampleInterpGrid` 走 **grid 缓存**（不读 splitCoord）。
- **结论**：grid 命中时 eval_df 消费的 split = 各 interp delegate 的 **corner=0（@c0）实例**（顶层 spline 坐标 +
  delegate @c0 孤儿节点）。interp 的 8 角点三线性已由 grid 覆盖，无需在当前点重算。

## 2. 改动（dfc_gen.py）

### 2.1 `_gen_split_lines(..., corner0_only=False)`
interpolated 分支 `for c in range(8)` → `for c in (range(1) if corner0_only else range(8))`，
并把 `corner0_only` 透传到**全部**递归调用（str resolve_ref / coordinate / nested spline / ws input / else 分支）。
> 关键：初次实现漏了非 interp 分支的透传，导致 splitTop=200 行（未收敛）；补齐后 = 25 行（= split 的 1/8）。

### 2.2 `gen_cpu`
```python
self.split_visited.clear()
split_lines = self._gen_split_lines(root_df, "x", "y", "z")
self.split_visited.clear()
split_top_lines = self._gen_split_lines(root_df, "x", "y", "z", corner0_only=True)
```
模板中在 `split()` 之后生成 `splitTop(x,y,z,out)`（用 `split_top_lines`）。

### 2.3 `gen_cpu_sampling` sample()
```cpp
float sample(int x, int y, int z) {
    if ((size_t)splitCoord.size() < (size_t)splitTotal) splitCoord.assign((size_t)splitTotal, 0.0f);
    splitTop(x, y, z, splitCoord.data());   // 由 split() 改为 splitTop()：grid 命中只算 @c0（1/8）
    return eval_density(0, x, y, z);
}
```
`prepare()` / `buildInterpGrid()` / `split()` 全部保持全量，不破坏。

## 3. 生成的关键 C++
```
void splitTop(int x, int y, int z, float* out) {
    { int _chunkX=floorDiv(x,16); ... int _cx=_gx/4; ...
      { splitDouble(normals[0], ...(cx+0)... out, 0, 9); }   // interp0 @c0（21 行）
      ... oldBlendeds[0] @c0 ... ws @c0 ...
    }
    { ... splitDouble(normals[160], ... out, 8288, 1); }     // interp1 @c0
    { ... splitDouble(normals[168], ... out, 8384, 1); }     // interp2 @c0
    { ... splitDouble(normals[176], ... out, 8480, 1); }     // interp3 @c0
    { ... splitDouble(normals[184], ... out, 8576, 1); }     // interp4 @c0
}
```
（25 条 split-call 行 = 整树 `split()` 200 行的 1/8，覆盖全部 5 个 interp 的 corner=0。）

## 4. 静态自检（已跑，全部通过）

工具 `verify_splittop.py`（保留可重跑）：

| 检查 | 结果 |
|---|---|
| splitTop 行数 | 25 == split 200 的 1/8（ratio 8.00）✅ |
| splitTop 每行与 split() corner-0 逐行匹配（同实例 index + 同 splitBase） | mismatch=0（25/25 命中）✅ |
| sample() 调 splitTop（不调 split） | ✅ |
| prepare() 仍用全量 split() | ✅ |
| buildInterpGrid 仍用全量 split() | ✅ |
| diff(旧 cpu_backend.h, 新生成) | 仅新增 splitTop 函数 + sample() 5 行注释/1 行改动，其余字节级一致 ✅ |

**正确性论证（逻辑）**：
1. **corner 恒 0**：grid 节点是 cell 的 (0,0,0) 角点；splitTop 与 split() 用同 cell 角点 0 坐标
   （`_chunkX*16+(_cx+0)*4` 等），生成值逐位一致。
2. **eval_df 非 interp 全走 @c0**：top 层读 `NOISE_SLOT_BASE[a1]`（无 `+corner`），即 @c0 实例；splitTop 提供全部 @c0。
3. **buildInterpGrid 的 `splitCoord.swap(saved)`** 会把 splitTop 填入的 @c0 还原，非 interp 读值不变；
   网格本身由 buildInterpGrid 内部全量 `split()`（@c0..@c7）逐 cell 求值，独立于 splitTop 状态。
4. **sIdx=0 恒走 grid**：`interp_N` 因 `sIdx==0` 走 `sampleInterpGrid`；`sample()` 恒 `sIdx=0`，故无
   sIdx!=0 的 8 角点回退路径读分裂的 splitCoord（该路径只与 prepare()（全量 split）配对）。
5. **grid miss 亦等价**：首个点 buildInterpGrid 后 splitCoord 还原为 splitTop @c0，@c0 读值与全量 split 相同。

**不破坏项**：eval_df / eval_df_base / interp_N / sampleInterpGrid / grid 缓存（buildInterpGrid+GridSlot）/
split() / prepare() 全部保持原样。

## 5. 预估提速

- `split()` 200 条 split-call → splitTop 25 条 = **split 成本 ×1/8**。
- 设计文档 §1.4：split 占每点成本 ~80-90%。则每点时间 T →
  T×(0.15 + 0.85/8) ≈ **T×0.26**（split=85%）或 T×(0.2+0.8/8) ≈ **T×0.30** —— **~3~4× 每点提速**。
- 882μs/点 → 估 ~230-260μs/点；整 chunk 87s → 估 ~20-30s（若节点计数 ~98k、网格构建保持）。
  **「先让整 chunk 生成不超时」即达成**；离 production 39ms 仍差，瓶颈转移到 grid 构建（每 interp 768 cell×
  全量 split）+ eval_df 孤儿 delegate 死计算。

## 6. 后续优化建议（本次未做，风险可控）
1. **per-cell splitTop 缓存**：同一 chunk 内同 cell 的 128 个 block 共享同一 cell 角点 0 → @c0 值恒定。
   可在 sample() 缓存 `(chunkKey, cellKey) → @c0 split`，grid 命中时复用（再省 ~25 条/点，达到接近 0 split/点）。
   需正确处理 buildInterpGrid 的 splitCoord.swap 与缓存命中时机。
2. **消除 eval_df 孤儿 delegate 死计算**：eval_df 遍历全部 163 节点，含 interp delegate 的 ~130 个孤儿节点
   @c0 计算（结果未被 TOP_ROOT 消费）。把 eval_df 改成像 GLSL eval_top 那样只遍历 top closure 可再省
   （≈ 9.57e-07 对齐下若闭包化，需逐位重验——GLSL 已是 top closure 语义，CPU 应同构）。
3. 参考 dfc_split_optimize_design.md §5 步骤 3/4：per-cell split 去重 + 生成器 split 布局翻转（高风险，需 Python 模拟层前置）。

## 7. 验证要求（主会话步骤 5）
重跑三张表：`dfc_cpp_vs_prod`（maxdiff 保持 ≤~1e-6，基线 9.57e-07）、`dfc_cpp_verif`、`verif_grid_cache_correctness`；
再跑无探针整批 wall + split 调用计数（AGENTS.md 测量污染铁律）。**不编译**——本 worker 只交付生成器改动。
