# A 方案（SPLINE 表 SSBO 化）实施记录（2026-08-15）

> 架构：`../000-架构设计/架构计划-gpu-spline-fix.md`（001 修订版，用户拍板 A 先行）
> 状态：A1a-A4a 完成；A4b 性能未达标（350.6s > 2min 目标），减法二分诊断中

## 改动清单（未提交）

- `dfc_gen.py`：
  - `_spline_ssbo_glsl` 重写：6 张 spline 表（NODE_PACK/LOCS/DERS/VAL_F/VAL_KIND/VAL_NODE）从 `const float/int X[N] = ...` 改为 `layout(set=0, binding=6..11, std430) buffer` SSBO 声明；spline_eval 从「每 node 静态展开 56 函数 + switch」恢复为 b1a 设计的 while 栈显式栈后序求值（帧 {node,i,coord,stage,v0,v1}，stage 0/1/3，32 深）；spline_find_range 恢复 while 二分（原固定 5 步展开）；spline_hermite 保留
  - 新增 `self.spline_layout` 导出（nNodes/nodePack/locs/ders/valF/valKind/valNode）——gen_cpu 输出到 CpuBackend（D19 铁律：宿主零硬编码）
  - gen_cpu 输出 splineNodes/splineNodePack/splineLocs/splineDers/splineValF/splineValKind/splineValNode 成员
  - D21 诊断开关扩展：`DFC_DIAG=fixed_node`（spline_eval 固定 node=0）/`coord_const`（spline_coord 固定返回 0.0f）
- `dfc_final_backend_e2e.cpp`：descriptor layout 5→12 binding（HEAD 版 bindings[5]）；新增 6 个 spline SSBO buffer 创建/上传/绑定/释放（binding 6-11）
- `compile_bench.cpp`：descriptor layout 8→12 binding（匹配 shader）
- 新增 `gen_spline_diag.py`：spline 剩余成本二分变体生成器

> 数字来源澄清（judge 审查项）：71.4s = 修复后 compile_bench 单独测 vkCreateComputePipelines（pwsh-3）；67.4s = 同 spv 在 e2e 内的 pipeline 计时（pwsh-4）。两工具测量同一 spv，差异 ~4s 属测量上下文/噪声，均 <120s 达标。final 确认值见 cmd-output/compile_bench-A5-*.txt。

## 验证结果（A4a 正确性，seed 8576294172403134396，N=1024）

| 指标 | 基线（D17/D19 修复后，while 栈 + const 表） | A 方案（SSBO 化） |
|---|---|---|
| maxDiff | 3.128e-07 | **3.128e-07**（逐位一致） |
| avgDiff | 1.097e-08 | **1.097e-08**（逐位一致） |
| TOP 差异点 | i=1004 pos=(44,-49,0) 3.128e-07 | 同点位同值 |
| 分量 | factor=3.95/sloped=12.69/entrances=0.5691/when_out=0.0989 | 待 ref_probe 复核（e2e 全树已覆盖） |

结论：**SSBO 化语义零影响，正确性完全保持**（spline 数据收集逻辑未动，只改输出形态）。

## 性能验证（A4b，compile_bench / e2e pipeline 计时）

| 变体 | const 表版（D21） | SSBO 版（A 初版） | 修复后 |
|---|---|---|---|
| 完整 | 903.4s | 350.6s（-61%） | **67.4s**（-92.5%，**达标 <2min**） |
| no_old（去 fp64） | 591.8s | 278.8s | **58.9s**（fp64 交互 ~310→~72→**~8.5s**） |
| no_spline | 17.6s | 17.2s | 17.2s |
| no_old+no_spline | 7.3s | 8.1s | — |
| **spline 子系统** | **~885s** | **~333s**（-62%） | **~50s**（-94%） |

- **达标判定：✅ 67.4s < 2min 目标**（架构计划 §5 拍板 HOOK）
- **fp64 次因自动作废**：修复后 no_old 只省 ~8.5s（曾 310s→72s）——fp64 成本本质是「与 spline 展开的交互效应」，coord 查表修复后消失（NEXT_SESSION 待办 2 不再需要）

## A5：spline_coord 根因二分 + 修复（2026-08-15 追加）

**二分证据链（减法二分，非猜测）**：

| 变体 | 编译时间 | 结论 |
|---|---|---|
| full | 350.6s | 基线 |
| fixed_node（node 固定 0） | 361.0s | **动态 node 索引不是 SSBO 版主因**（D21 结论只在 const 表版成立） |
| coord_const（coord switch 全 0） | 37.2s | coord 表达式贡献 ~313s |
| coord_slot0（4 case 同 slot） | 302.3s | 与「不同实例数」无关 |
| coord_case0（仅 1 case 调 normal_noise） | 74.8s | **1 次调用 +37s** |
| no_spline | 17.2s | eval_df 里同函数调用不慢 |

- **根因**：spline_coord 的 `switch(coordType)` 让每个 case 内 `NOISE_SLOT_BASE[0]` 成为**编译期常量下标** → 驱动常量传播进 normal_noise → NORMAL_PACK 读取静态化 → 循环展开（每次调用 +37~75s）。eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]` 索引完全动态 → 驱动放弃展开（快）。
- **修复**：spline_coord 改「coordType 运行时查表」——`const int COORD_SLOT_TABLE[N] = int[](...)` + `int slot = COORD_SLOT_TABLE[coordType];` → normal_noise 实例索引运行时不可解析；fold 包装（coordType==2 的 abs 链）提取为 `if (coordType == 2) v = ...` 特例；非标准形态（无纯 normal_noise 调用）fallback 原 switch。
- **教训**：①「动态 node 索引」结论有版本域——const 表版成立、SSBO 版不成立（SSBO 已把动态索引变运行时读，固定 node 无收益）；② **编译期常量下标（NOISE_SLOT_BASE[0]）进数据驱动函数 = 常量传播展开陷阱**——switch/case 常量化与动态索引是编译时间分水岭；③ 减法二分（coord_case0 单次调用定位 +37s）比猜更快。

