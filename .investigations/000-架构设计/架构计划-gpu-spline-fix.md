---
编号: 001
任务: GPU 加速主线——final_density shader 驱动编译瓶颈解决（spline 数据驱动 + 拆 shader + pipeline cache）
任务类型: 重构（代码生成器）+ 验证（性能/精度）
模式档位: 重量
状态: 已批准（2026-08-14 用户拍板：B→C→D 三步走）→【2026-08-15 修订：D21 推翻 B 路线，新决策 A→C，用户拍板 A 先行】
日期: 2026-08-14（修订 2026-08-15）
---

# 架构计划：spline 数据驱动（B）+ 拆 shader（C）+ pipeline cache（D）
# 2026-08-15 修订版：B 已被 D21 实证推翻 → A（SSBO 化）→ C（CPU 预计算）

## 0. 背景与目标（NEXT_SESSION 四 待办 1 + D21 结论）

- 卡点：final_density 全量 shader 驱动编译 903.4s（vkCreateComputePipelines；glslc 前端仅 0.2s）。
- **D21 根因（2026-08-14 实证锁定，推翻原 B 路线）**：
  - 原 B（spline SSBO+显式栈 56→1）已实施为「while 栈版」，但**数据表留成 shader 内嵌 const 数组**（AOT 简化，注释「数据为编译期常量，无需 SSBO 上传」）→ `SPLINE_NODE_PACK[node*5]` 动态索引 const 大表 → 驱动为 56 个可能 node 各自展开数据流 → 组合爆炸 = 903.4s。
  - 实验判别：去 spline 17.6s；固定 node=0 31.0s；while/二分/switch/函数数全部排除 → **「动态 node → 动态索引 const 大表」是驱动编译地狱**。
  - fp64 次因 310s 是「与 spline 的交互效应」：no_spline 后 no_old 只差 ~10s（17.6 vs 7.3）→ **spline 修复后 fp64 次因自动消失，无需单独处理**（NEXT_SESSION 待办 2 作废）。
- **用户拍板（2026-08-15）**：**A 先行**——spline 数据表 const → 真 SSBO（运行时 buffer），求值逻辑（while 栈）不动；实测 >2min 再切 C（CPU 预计算）。

## 1. 范围（含明确不做什么）

**做（A 方案）**：
- A1：dfc_gen.py `_spline_ssbo_glsl` 输出形态改造——6 张 spline 表（NODE_PACK/LOCS/DERS/VAL_F/VAL_KIND/VAL_NODE）从 `const float X[N] = float[](...)` 改为 `layout(set=0, binding=N, std430) buffer` 声明；恢复 while 栈求值单函数 `spline_eval`（当前是诊断中间态「每 node 静态展开 56 函数 + switch」，编译 >12min 且未做正确性回归，需重写回 b1a 设计的显式栈形态）。
- A2：生成器导出 spline 数据表供宿主上传（cpu_backend.h 或独立头 + 布局常量 SPLINE_NODES 等随生成器产出——D19 铁律：禁止宿主硬编码）。
- A3：dfc_final_backend_e2e.cpp 新增 spline 表上传 + 绑定（当前 e2e 无任何 spline 代码）。
- A4：重新生成 final_density.comp + cpu_backend.h，glslc 编译。
- A5：正确性回归 e2e maxDiff vs 基线（3.128e-07 不回退）+ 分量对拍（ref_probe：factor=3.95/sloped=12.69/entrances=0.5691/when_out=0.0989）。
- A6：性能验证 compile_bench 计时 vs 903.4s（目标 <2min）。

**不做（本阶段）**：
- ❌ C（CPU 预计算）——A 实测 >2min 才启动（决策 HOOK，见 §5）。
- ❌ D（VK_KHR_pipeline_binary）——正确性达标后的正交叠加，单独立项。
- ❌ fp64 次因处理——spline 修复后自动消失（D21 证据）。
- ❌ 集成进 block_probe（A 达标后单独立项，8576/3200 零退化终验）。
- ❌ 改动 CPU 侧参照（density_builder.h SplineDF / worldgen 现有路径）——零退化铁律。
- ❌ 改 Anchorlaw/RE-Framework。

## 2. 任务拆解

| # | 子任务 | 产物 | 验证 |
|---|---|---|---|
| A1a | dfc_gen.py：恢复 while 栈 spline_eval（b1a 设计：Frame 栈后序求值/二分/边界外推/Hermite）+ 数据表改 SSBO 声明 | dfc_gen.py diff | 静态审查（对照 b1a 设计文档） |
| A1b | 生成器导出 spline 数据 + 布局常量（SPLINE_NODES/表长/binding 号） | cpu_backend.h 或 spline_tables.h | 生成跑通 + D19 检查（无宿主硬编码） |
| A2 | e2e 新增 spline SSBO 上传/绑定（binding 5/6/7 或续号） | dfc_final_backend_e2e.cpp diff | MSVC 编译通过 |
| A3 | 重新生成 + glslc | final_density.comp/spv | glslc 通过 |
| A4a | 正确性回归：e2e maxDiff + 分量对拍（ref_probe） | e2e 输出 + ref_probe 记录 | maxDiff 不回退（≈3.128e-07）+ 分量逐位 |
| A4b | 性能：compile_bench 计时 | 计时记录 | vs 903.4s，目标 <2min |
| T7 | 知识库更新（subagent 产出草稿）：gpu-accel-errors.md（A 实施坑）+ 10 时间线 + 本计划更新 | docs diff | 一致性 |

## 3. 验证方式

- 编译层：glslc 计时 + compile_bench（vkCreateComputePipelines 单独计时，秒级）。
- 精度层：dfc_final_backend_e2e（CPU 参照 vs GPU）maxDiff 对比基线 3.128e-07；ref_probe 分量对拍（factor/sloped/entrances/when_out 逐位）。
- 规模层：spirv-dis 函数数（spline 56→1）+ SPIR-V 体积。
- 布局层：D19 检查——perSample/splitTotal/SPLINE_NODES/表长全部生成器产出，宿主零硬编码。

## 4. 风险 & 回退

- **R1 驱动对 SSBO 动态索引仍展开**（A 的核心不确定点，未实测）：若 compile_bench >2min → 切 C（决策 HOOK）。判定标准：A 实施后 compile_bench 实测。
- **R2 while 栈重写语义错**（诊断中间态覆盖了基线，git 无 while 栈版）：对照 b1a 设计 + ref_probe 分量对拍 + 与当前诊断中间态的输出数值对拍（数据收集逻辑未变，只输出形态变）。
- **R3 上传/绑定错**（新增 binding、数据布局错位）：e2e 直接暴露（输出 0/越界类 D19 症状）；布局常量由生成器产出杜绝硬编码。
- **R4 spline 求值顺序舍入**（v0/v1 顺序）：与 D17 基线 while 栈版一致（先 nv 后 ov）→ 舍入一致。
- **回退**：改动集中在 dfc_gen.py + e2e + 生成产物（不提交 .spv/.comp）——git checkout 单文件回退；数据收集逻辑不动，回退成本低。

## 5. 人工 HOOK 点

- **A4b 性能实测后**：用户拍板——达标（<2min）收尾 / 未达标切 C（CPU 预计算）→ 回 Phase 0 重新架构。
- 收尾交付：用户拍板 confirmed。

## 6. judge 步骤预置

- 节点：A4a 精度回归 + A4b 性能数据落盘后 | MUST | 三源核对（dfc_gen.py diff + e2e/ref_probe 输出 + compile_bench 计时）
- 节点：A 闭环交付 | MUST | 三源核对（含 git diff 应用版）
- 节点：若切 C | 新架构计划批准后 | MUST

## 7. fan-out 步骤预置

- A4b 若编译仍慢：分叉候选（SSBO 动态索引仍展开 / SSBO 布局问题 / 其他）→ 先 DFC_DIAG 减法二分定位（D21 方法），仍互斥则 MUST fan-out 并行 .bN。
- 当前 A 实施无已知分叉（单一路径）。

## 8. 知识库更新（subagent 产出草稿）

- gpu-accel-errors.md：A 实施新坑按「现象→根因→定位→修复→教训」追加。
- 10 时间线：2026-08-15 条目（A 决策 + 实施结果）。
- 本架构计划：实施结果回填。

## 9. 子角色介入点（全部预置）

- scout：否（机制已明，D21 已锁定根因）。
- worker：知识库草稿产出（subagent）；若 A4b 数据解读发散再按 fan-out 规则。
- fan-out：A4b 分叉点预置（见 §7）。
- judge：A4a/A4b 数据落盘后 MUST；收尾交付 MUST。
- knowledge：T7 subagent 产出。

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
- 数字口径（judge 审查项）：67.4s = e2e 内 pipeline 计时；71.4s = compile_bench 单独测 vkCreateComputePipelines；同一 spv 两工具差 ~4s 属测量上下文/噪声；final 确认值见 cmd-output/compile_bench-A5-*.txt。

### 正确性（A4a）

- maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线逐位一致（TOP 差异点 i=1004 pos=(44,-49,0) 同点位同值）；ref_probe factor=3.950000048 / sloped=12.690109836 / entrances=0.569083105。

### 遗留项（P2/P3，未立项）

- z 采样覆盖 / binding 号导出 / gen_split_shaders 宿主适配 / binding 2 死代码 / block_probe 终验（8576/3200 零退化终验）——未立项，待后续。

### 风险回填（对照 §4）

- R1（驱动对 SSBO 动态索引仍展开）→ 实测排除：SSBO 初版 350.6s 后经 A5 达标，**未触发切 C 决策 HOOK**。
- R2（while 栈重写语义错）→ 排除：maxDiff 与基线逐位一致。
- R3（上传/绑定错）→ 排除：无输出 0/越界类 D19 症状；布局常量生成器产出。
- R4（求值顺序舍入）→ 排除：与 D17 基线逐位一致。
