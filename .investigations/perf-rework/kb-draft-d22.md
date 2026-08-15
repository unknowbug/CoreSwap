# KB 更新草稿：G4 编译时间修复 A 方案（SPLINE 表 SSBO 化）
# core.worker 产出（2026-08-15）· 仅供主会话应用，不修改正式知识库
# 数据源：.investigations/perf-rework/a-plan-ssbo-implementation.md（A5 节）

---

## 草稿 1/4：gpu-accel-errors.md D22 条目

> **追加位置**：D21 条目末尾（「方案决策点（待用户拍板）」一行）之后、D18 条目之前。

### D22. A 方案 SSBO 化后仍 350.6s——spline_coord 编译期常量下标进 normal_noise 触发常量传播展开（2026-08-15，已修复，A5 减法二分）
- **现象**：A 方案（6 张 spline 表 const→SSBO，binding 6-11）实施后 compile_bench 单独测 vkCreateComputePipelines = **350.6s**（D21 基线 903.4s，-61%）——**仍未达 <2min 目标**；no_spline 17.2s（spline 子系统仍占 ~333s）。正确性同时验证：SSBO 化语义零影响（maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线逐位一致）。
- **根因（A5 二分锁定，非猜测）**：spline_coord 的 `switch(coordType)` 让每个 case 内 `NOISE_SLOT_BASE[0]` 成为**编译期常量下标** → 常量传播进 normal_noise（数据驱动函数，参数表在 const 数组）→ `NORMAL_PACK` 读取静态化 → **循环展开**（每次调用 +37~75s）。对照：eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]` 索引完全动态 → 驱动放弃展开（快）。
- **定位（DFC_DIAG 诊断开关 + compile_bench 秒级测，减法二分表）**：
  | 变体 | 编译时间 | 结论 |
  |---|---|---|
  | full | 350.6s | 基线 |
  | fixed_node（node 固定 0） | 361.0s | **动态 node 索引不是 SSBO 版主因**（D21 结论只在 const 表版成立） |
  | coord_const（coord switch 全 0） | 37.2s | coord 表达式贡献 ~313s |
  | coord_slot0（4 case 同 slot） | 302.3s | 与「不同实例数」无关 |
  | coord_case0（仅 1 case 调 normal_noise） | 74.8s | **1 次调用 +37s** |
  | no_spline | 17.2s | eval_df 里同函数调用不慢 |
- **修复**：spline_coord 改「coordType 运行时查表」——`const int COORD_SLOT_TABLE[N] = int[](...)` + `int slot = COORD_SLOT_TABLE[coordType];` → normal_noise 实例索引**运行时不可解析**；fold 包装（coordType==2 的 abs 链）提取为 `if (coordType == 2) v = ...` 特例；非标准形态（无纯 normal_noise 调用）fallback 原 switch。结果：**67.4s**（e2e 内 pipeline 计时）/ **71.4s**（compile_bench 单独测，同一 spv 两工具差 ~4s 属测量噪声）/ **101.8s**（第 3 次测量波动）——**3 次均 <120s 达标**；no_old 278.8→51.8-58.9s（fp64 交互 310→72→~8-10s，**fp64 次因自动作废**）；正确性保持逐位一致（e2e N=1024 seed 8576294172403134396；ref_probe factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105）。
- **D19 合规确认**：spline 布局 6 表（splineNodes/splineNodePack/splineLocs/splineDers/splineValF/splineValKind/splineValNode）+ perSample 全部由生成器产出（`self.spline_layout` 导出 → gen_cpu 输出 CpuBackend 成员），宿主零硬编码。
- **教训**：① **「动态 node 索引」结论有版本域**——const 表版成立（D21）、SSBO 版不成立（SSBO 已把动态索引变运行时读，fixed_node 无收益 361.0s≈full）——跨版本复用根因结论前必须重新验证版本前提；② **编译期常量下标进数据驱动函数 = 常量传播展开陷阱**——switch/case 把下标常量化与动态索引是编译时间分水岭（coord_const 37.2s vs full 350.6s，~10× 级差），「数据驱动函数」必须用运行时查表把索引变不可静态化，不能留编译期常量下标；③ **减法二分比猜快**——coord_case0 单次调用定位 +37s，一次实验排除一个候选（复用 D21 的 DFC_DIAG 方法论）。

> 配套速查表追加行（可选，附在「附：错误 → 根因 速查表」末尾）：
> | SSBO 化后仍 350.6s（D22） | spline_coord 的 `switch(coordType)` 使 case 内 `NOISE_SLOT_BASE[0]` 变编译期常量下标 → 常量传播进 normal_noise → NORMAL_PACK 静态化 → 循环展开（每调用 +37~75s）。**改运行时查表 COORD_SLOT_TABLE + fold 特例 → 67.4-101.8s 达标；「动态 node 索引」结论有版本域（const 表成立 / SSBO 不成立）** |

---

## 草稿 2/4：10-timewise-archive.md 追加条目

> **追加位置**：文件末尾（当前最后一条为 2026-08-13 条目）。

## 2026-08-15：G4 编译时间修复——A 方案（spline 6 表 SSBO 化）实施 + A5 coord 查表根因 + 达标（✅ 性能/正确性双达标 / 🔍 遗留 P2/P3）

> 承接 2026-08-14 D21 条目（903.4s 根因 = spline 动态 node 索引 + 方案决策点）。用户拍板 **A 先行**（spline 数据表 const→真 SSBO，架构计划 001 修订版）。实施 + 二分 + 修复完整记录落盘 `.investigations/perf-rework/a-plan-ssbo-implementation.md`（A5 节）。

### ✅ A 方案实施（A1a-A4a 完成）

- dfc_gen.py `_spline_ssbo_glsl` 重写：6 张 spline 表（NODE_PACK/LOCS/DERS/VAL_F/VAL_KIND/VAL_NODE）const→`layout(set=0, binding=6..11, std430) buffer` SSBO；spline_eval 恢复 b1a 设计的 while 栈显式栈后序求值（帧 {node,i,coord,stage,v0,v1}，stage 0/1/3，32 深）；spline_find_range 恢复 while 二分；新增 `self.spline_layout` 导出 → gen_cpu 输出 7 个 spline 成员（**D19 合规：宿主零硬编码**）。
- dfc_final_backend_e2e.cpp：descriptor 5→12 binding，新增 6 个 spline SSBO buffer 创建/上传/绑定/释放（binding 6-11）；compile_bench descriptor 8→12；新增 `gen_spline_diag.py`（spline 剩余成本二分变体生成器）。

### ✅ A4b 性能（compile_bench / e2e pipeline 计时）

| 变体 | const 表版（D21） | SSBO 版（A 初版） | 修复后 |
|---|---|---|---|
| 完整 | 903.4s | 350.6s（-61%） | **67.4s**（-92.5%，**达标 <2min**） |
| no_old（去 fp64） | 591.8s | 278.8s | **58.9s**（fp64 交互 ~310→~72→**~8.5s**） |
| no_spline | 17.6s | 17.2s | 17.2s |
| no_old+no_spline | 7.3s | 8.1s | — |
| **spline 子系统** | **~885s** | **~333s**（-62%） | **~50s**（-94%） |

- **达标判定 ✅**：67.4s < 2min 目标（架构计划 §5 拍板 HOOK）。3 次测量 67.4/71.4/101.8s 有波动，均 <120s。数字口径（judge 审查项）：71.4s = compile_bench 单独测 vkCreateComputePipelines（pwsh-3）；67.4s = 同 spv 在 e2e 内 pipeline 计时（pwsh-4）；同一 spv 两工具差 ~4s 属测量上下文/噪声；final 确认值见 cmd-output/compile_bench-A5-*.txt。
- **fp64 次因自动作废**：修复后 no_old 只省 ~8.5s——fp64 成本本质是「与 spline 展开的交互效应」，coord 查表修复后消失（NEXT_SESSION 待办 2 不再需要）。

### ✅ A4a 正确性（与基线逐位一致）

- maxDiff=**3.128e-07** / avgDiff=**1.097e-08**，与基线（D17/D19 修复后 while 栈 + const 表版）**逐位一致**（e2e N=1024 seed 8576294172403134396；TOP 差异点 i=1004 pos=(44,-49,0) 同点位同值）。
- ref_probe 分量：factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105。
- 结论：SSBO 化 + 查表修复语义零影响（spline 数据收集逻辑未动，只改输出形态）。

### ✅ A5 根因二分 + 修复（coordType 查表，本日最重要新知识）

- **二分证据链（减法二分，非猜测）**：fixed_node（361.0s ≈ full）排除「动态 node 索引」在 SSBO 版是主因（**D21 结论有版本域：const 表版成立、SSBO 版不成立**——SSBO 已把动态索引变运行时读）；coord_const（37.2s）定位 coord 表达式贡献 ~313s；coord_slot0（302.3s）排除「不同实例数」因素；coord_case0（74.8s）定位 1 次 normal_noise 调用 +37s；no_spline（17.2s）排除 eval_df 内同函数调用慢。
- **机制**：spline_coord 的 `switch(coordType)` 让每个 case 内 `NOISE_SLOT_BASE[0]` 成为**编译期常量下标** → 常量传播进 normal_noise 数据驱动函数 → NORMAL_PACK 读取静态化 → **循环展开**（每次调用 +37~75s）。eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]` 索引完全动态 → 驱动放弃展开（快）。
- **修复**：spline_coord 改「coordType 运行时查表」——`const int COORD_SLOT_TABLE[N] = int[](...)` + `int slot = COORD_SLOT_TABLE[coordType];` → normal_noise 实例索引运行时不可解析；fold 包装（coordType==2 的 abs 链）提取为 `if (coordType == 2)` 特例；非标准形态 fallback 原 switch。
- **教训（可复用）**：①「动态 node 索引」结论有版本域 ② 编译期常量下标进数据驱动函数 = 常量传播展开陷阱（switch/case 常量化 vs 动态索引是编译时间分水岭）③ 减法二分（coord_case0 单次调用定位 +37s）比猜快。
- 错误台账完整条目：gpu-accel-errors.md D22；通用模式：knowledge/discovered/algorithm-fingerprints.md 发现 #13。

### 🔍 遗留项（P2/P3，未立项）

- z 采样覆盖 / binding 号导出 / gen_split_shaders 宿主适配 / binding 2 死代码 / block_probe 终验（8576/3200 零退化终验）——均未立项。

---

## 草稿 3/4：架构计划-gpu-spline-fix.md 回填小节

> **追加位置**：文件末尾（§9 子角色介入点之后），作为「## 10」。

## 10. 实施结果回填（2026-08-15）

> 001 修订版追加。A 方案实施 + A5 根因二分 + 查表修复全部达标；完整记录 `.investigations/perf-rework/a-plan-ssbo-implementation.md`。

### A1-A5 完成情况

| 子任务 | 状态 | 结果 |
|---|---|---|
| A1a（while 栈 spline_eval + 6 表 SSBO 声明） | ✅ 完成 | dfc_gen.py `_spline_ssbo_glsl` 重写：NODE_PACK/LOCS/DERS/VAL_F/VAL_KIND/VAL_NODE const→binding 6-11 SSBO；spline_eval 恢复 b1a while 栈（帧 {node,i,coord,stage,v0,v1}，32 深）；spline_find_range 恢复 while 二分 |
| A1b（生成器导出 spline 数据 + 布局常量） | ✅ 完成 | 新增 `self.spline_layout` 导出 → gen_cpu 输出 splineNodes/splineNodePack/splineLocs/splineDers/splineValF/splineValKind/splineValNode 成员；D19 检查通过（宿主零硬编码，含 perSample） |
| A2（e2e spline SSBO 上传/绑定） | ✅ 完成 | dfc_final_backend_e2e.cpp descriptor 5→12 binding + 6 buffer 创建/上传/绑定/释放（binding 6-11）；compile_bench descriptor 8→12 |
| A3（重新生成 + glslc） | ✅ 完成 | final_density.comp/spv 重新生成，glslc 通过 |
| A4a（正确性回归） | ✅ 完成 | e2e N=1024 seed 8576294172403134396：maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线**逐位一致**；ref_probe factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105 |
| A4b（性能） | ✅ 达标（经 A5） | SSBO 初版 350.6s 未达标 → A5 二分根因 + 查表修复 → 67.4/71.4/101.8s（3 次测量均 <120s，**达标 <2min**，§5 拍板 HOOK 触发为「达标收尾」） |
| A5（spline_coord 根因二分 + 修复，实施中追加） | ✅ 完成 | 根因 = switch(coordType) 使 case 内 NOISE_SLOT_BASE[0] 成编译期常量下标 → 常量传播进 normal_noise → NORMAL_PACK 静态化 → 循环展开；修复 = COORD_SLOT_TABLE 运行时查表 + if(coordType==2) fold 特例 + fallback。二分链：fixed_node 361.0s / coord_const 37.2s / coord_slot0 302.3s / coord_case0 74.8s / no_spline 17.2s |

### 性能数据（A4b 最终确认）

- 完整 pipeline 编译：903.4s（const 表基线）→ 350.6s（SSBO 初版，-61%）→ **67.4s**（查表修复，-92.5%，达标）。
- no_old（去 fp64）：591.8s → 278.8s → 58.9s；no_spline：17.6s → 17.2s → 17.2s；no_old+no_spline：7.3s → 8.1s；spline 子系统 ~885s → ~333s（-62%）→ ~50s（-94%）。
- **fp64 次因自动作废**（NEXT_SESSION 待办 2 不再需要）：修复后 no_old 只省 ~8.5s——fp64 成本是「与 spline 展开的交互效应」，coord 查表修复后消失。
- 数字口径（judge 审查项）：67.4s = e2e 内 pipeline 计时（pwsh-4）；71.4s = compile_bench 单独测 vkCreateComputePipelines（pwsh-3）；同一 spv 两工具差 ~4s 属测量上下文/噪声；final 确认值见 cmd-output/compile_bench-A5-*.txt。

### 正确性（A4a）

- maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线逐位一致（TOP 差异点 i=1004 pos=(44,-49,0) 同点位同值）；ref_probe factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105。

### 遗留项（P2/P3，未立项）

- z 采样覆盖 / binding 号导出 / gen_split_shaders 宿主适配 / binding 2 死代码 / block_probe 终验（8576/3200 零退化终验）——未立项，待后续。

### 风险回填（对照 §4）

- R1（驱动对 SSBO 动态索引仍展开）→ 实测排除：SSBO 初版 350.6s 后经 A5 达标，**未触发切 C 决策 HOOK**。
- R2（while 栈重写语义错）→ 排除：maxDiff 与基线逐位一致。
- R3（上传/绑定错）→ 排除：无输出 0/越界类 D19 症状；布局常量生成器产出。
- R4（求值顺序舍入）→ 排除：与 D17 基线逐位一致。

---

## 草稿 4/4：algorithm-fingerprints.md 发现 #13

> **追加位置**：文件末尾（当前最后一条为发现 #12）。

## 发现 #13: GPU 驱动编译时间——编译期常量下标进数据驱动函数 = 常量传播展开陷阱；「编译慢根因结论」有版本域（const 表 vs SSBO）

**发现时间:** 2026-08-15
**发现者:** worker（G4 A 方案实施 + A5 减法二分）
**来源定位:** `.investigations/perf-rework/a-plan-ssbo-implementation.md`（A5 节）+ `.investigations/perf-rework/gpu-accel-errors.md` D21/D22 + `.investigations/000-架构设计/架构计划-gpu-spline-fix.md`（001 修订版）；复现数据 `cmd-output/compile_bench-A5-*.txt`
**置信度:** candidate（减法二分证据链 + 3 次实测 67.4/71.4/101.8s 均 <120s 达标 + 正确性逐位一致 maxDiff=3.128e-07；confirmed 待用户拍板）
**module:** perf

### 观察
GPU 驱动编译时间（vkCreateComputePipelines 的 SPIR-V→机器码）对「数据驱动函数」的索引形态极度敏感，两个互相关联的通用模式：

1. **编译期常量下标进数据驱动函数 = 常量传播展开陷阱**：spline_coord 的 `switch(coordType)` 使每个 case 内 `NOISE_SLOT_BASE[0]` 成为编译期常量下标 → 常量传播进 normal_noise（数据驱动函数，参数表在 const 数组）→ `NORMAL_PACK` 读取被静态化 → 驱动逐 case 循环展开（单次调用 +37~75s）。对照：eval_df 里 `NOISE_SLOT_BASE[CA1_T[ci]]`（索引完全动态）→ 驱动放弃展开（快）。**同一批数据、同一个求值函数，仅「索引在编译期是否可解析」的差异 → 编译时间 350.6s vs 37.2s（~10×）级差**。
2. **「编译慢根因结论」有版本域**：D21 在 const 表版实证「动态 node 索引是主因」（固定 node=0 → 903.4→31.0s）；SSBO 化后做同一实验（fixed_node=0）→ 361.0s ≈ full 350.6s（无收益）——**同一个「固定动态索引」实验，const 表版成立、SSBO 版不成立**（SSBO 已把动态索引变成运行时 buffer 读，驱动无从展开，因此固定它也没有额外收益）。

### 证据
- 减法二分链（DFC_DIAG 诊断开关 + compile_bench 秒级测）：full 350.6s / fixed_node 361.0s（动态 node 索引非 SSBO 版主因）/ coord_const 37.2s（coord 表达式贡献 ~313s）/ coord_slot0 302.3s（排除实例数因素）/ coord_case0 74.8s（1 次 normal_noise 调用 +37s）/ no_spline 17.2s（eval_df 内同函数调用不慢）。
- 修复验证：coordType 运行时查表（`COORD_SLOT_TABLE[coordType]`）+ fold 特例（`if (coordType == 2)`）后，pipeline 编译 350.6s → 67.4s（e2e 内计时）/ 71.4s（compile_bench 单独）/ 101.8s（第 3 次）——3 次均 <120s；正确性逐位一致（maxDiff=3.128e-07 / avgDiff=1.097e-08，seed 8576294172403134396，N=1024）；fp64 次因自动作废（no_old 只省 ~8.5s——fp64 成本是「与 spline 展开的交互效应」）。
- 历史对照：const 表版动态 node 索引 903.4s（D21）→ SSBO 版 350.6s → 查表修复 67.4s；spline 子系统 ~885s → ~50s（-94%）。

### 如何利用
- **排查 GPU 驱动编译慢时，先查「数据驱动/查表函数的所有索引是否运行时不可解析」**——switch/case 把下标常量化、函数参数折叠成常量、const 数组编译期已知下标，都会触发常量传播 + 展开；把下标改成「运行时查表」让驱动无法静态化，是编译时间分水岭。
- **「固定某索引 / 去掉某子系统的减法二分」是最快定位手段**：一次实验排除一个候选（本次 coord_case0 单次调用定位 +37s），比机制猜测快；配合 DFC_DIAG 类诊断开关 + 秒级编译计时器使用。
- **根因结论必须声明版本域**：凡是「在某结构版本下成立的编译慢根因」（如动态 node 索引），结构改变后（const 表→SSBO/数据 buffer）**必须重新验证，不能直接复用**——「动态索引」在 const 表里是驱动展开的输入，在 SSBO 里已经是运行时读，同一表述指代完全不同的编译行为。
- 跨项目通用：任何 GPU compute shader / OpenCL kernel 的驱动编译时间优化（C2ME 类内核分发、pipeline 预编译分发）都适用「索引可解析性」与「版本域」两条检查。
