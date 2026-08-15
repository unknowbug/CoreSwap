# kb-draft-d23-timewise —— 草稿 1/3（目标文件：versions/1.20.1/docs/10-timewise-archive.md，追加到文件末尾）

> 本文件是知识库 subagent 草稿，供主会话应用 + 验证签核。插入位置：10-timewise-archive.md 末尾（现有「2026-08-15（下午段）：知识库流程改进」条目之后）追加以下条目。格式对齐同文件 G4 段（2026-08-15 上午段）风格：承接行 + ✅/❌/🔍 状态分节。

---

## 2026-08-15（晚段）：block_probe 集成立项 I1-I5——GPU 引擎接入 worldgen + D23 spline 边界 bug（✅ I1-I5 集成闭环 / ✅ D23 GPU+sim 双修 / 🔍 遗留 P2/P3 + confirmed 待用户）

> 承接 2026-08-15 上午段 D22 条目（A 方案 SSBO 化 + coord 查表达标）。架构：`.investigations/000-架构设计/架构计划-gpu-integration.md`（002，用户 2026-08-15 批准）。目标：DFC + CpuBackend + Vulkan 运行时接入 worldgen，8576/3200 零退化终验 + 吞吐对比。集成记录 `.investigations/perf-rework/i-integration-record.md`；judge 审查 `review-003-d23-integration.md`（4 个 P1，P1-1/P1-2 已闭环、P1-3 已重跑落盘、P1-4 由本知识库更新闭环）；D23 完整错误记录 gpu-accel-errors.md D23 段（含最终合并版 + 判错经验补充段）。

### ✅ I1：Vulkan 运行时封装（vulkan_runtime.h）

- header-only 组件（复制到 `worldgen/src/vulkan_runtime.h`）；接口 init / createPipeline(spv) / createBuffer / upload / makeDescriptorSet<N> / dispatch / readback / destroy / destroyBuffer
- 语义与 e2e 内联版逐位一致：**12 binding storage buffer 布局**（binding 2 已删 OriginBuf 但保留占位）、host-visible+coherent memory、单 command buffer + fence、256 work items/组
- 驱动一次性 pipeline 编译 ~70-100s（domain probe 标注）；e2e 改用组件后 maxDiff=3.128e-07 / avgDiff=1.097e-08 与内联版逐位一致，pipeline 90.9s 达标

### ✅ I2：GpuDensityEngine PIMPL + worldgen 接入

- `vulkan-proto/gpu_density_engine.h/.cpp`（PIMPL，复制到 `worldgen/src/`）；接口 GpuDensityEngine(seed, spvPath) / fill(coords, n, out) / sample / splitTotal / perSample / splineBindBase
- **PIMPL 原因（集成期新坑）**：cpu_backend.h → density.h 的 static 成员定义（InterpolatedDF::nextId 等 L937-942）**非 inline**，多 TU include 会 **LNK2005**（worldgen_core 恰好单 TU 持有定义未触发；引擎引入第二 TU 暴露）→ **修复**：density.h L937-942 static 定义加 `inline`（C++17 inline 变量，语义与单 TU 完全一致，零运行时影响）
- 引擎验证（gpu_fill_probe）：maxDiff=3.128e-07 / avgDiff=1.097e-08 与 DensityBuilder 参照逐位一致；splitTotal=8672 / perSample=352 / splineBindBase=6 对齐生成器（D19 合规：宿主零硬编码）
- worldgen 接入（worldgen_api.cpp）：WorldgenHandle 加 `gpu` 字段（`#ifdef CORESWAP_GPU_ENABLED` 条件）；wg_create 尾部 env `WG_GPU_FILL=1` 时构造引擎（spv 从 gpu-assets 读，缺文件 CPU fallback）；wg_fill_density GPU 分支（批量坐标 → fill → float 转 double 输出）/ CPU 分支（默认，零退化）

### ✅ I3：生成器产物纳入构建（gpu-assets）

- 目录约定 `worldgen/gpu-assets/`（cpu_backend.h + final_density.spv）；gen_final_density.py 同步 cpu_backend.h 到 gpu-assets（spv 由 glslc 编译后复制/脚本化）
- CMake：worldgen_core 加 gpu_density_engine.cpp / vulkan_runtime.h；`if(DEFINED ENV{VULKAN_SDK})` 条件加 Vulkan include/lib + CORESWAP_GPU_ENABLED 定义（无 SDK 时 CPU-only 构建）

### ✅ I4：零退化（8576 CPU 路径 + GPU 接入不破坏）

- **I4a**：8576 CPU 路径 99.9994% 与基线一致（block_probe CPU 路径实测；3200 零退化沿用 2026-08-12 回归口径 99.9997%）
- **I4b**：GPU 引擎接入不破坏——块级生成（fillOneChunkCore）**恒走 CPU finalDensity->sample**，GPU 引擎（WG_GPU_FILL=1）仅构造 + wg_fill_density 批量接口生效，块级路径不受影响（fallback 机制 + WG_GPU_FILL=1 下 block_probe 运行不崩溃）
- ⚠️ **范围修正（judge P1-2）**：I4b 不是「GPU 参与块生成的逐位验证」——块级正确性由 CPU 路径保证；GPU 引擎自身的逐位正确性由 e2e（3.128e-07）+ domain probe（9.9e-9）验证（i-integration-record 表述已修正）

### ✅ I5：吞吐对比——GPU 24-32x，吞吐探针带 diff 抽查 → 发现 D23

- gpu_throughput_probe（chunk 批量 1/4/16/64）实测：**GPU 24-32x**（1/4/16 chunks）
- **意外收获**：探针顺带做同点 diff 抽查 → **16/64 chunks maxDiff 飙到 2.02e-01 / 4.45e-01**（应 ~1e-7 量级），1/4 chunks 正常（1.04e-06 / 1.33e-06）→ 发现 GPU 引擎在 e2e 验证域外系统性错值 → 引出 D23（**吞吐探针若只测时间不测 diff 就漏了**）
- D23 修复后：I5 各 chunk diff **1e-6~4e-6**（正确性恢复），吞吐 24-32x 保持

### ✅ D23：spline 边界外推遇嵌套 value 直接返回 0（GPU+sim 双修，judge P1-1 追补闭环）

> 完整错误记录（含现象/根因/定位链/误判更正/修复/教训五段式 + 速查表）见 gpu-accel-errors.md D23 段；通用模式见 discovered/algorithm-fingerprints.md 发现 #14。此处时间线式记录推理过程（保留被排除候选与中间误判）。

**现象**：I5 吞吐探针 16/64 chunks 带 diff 抽查发现 GPU 引擎在 e2e 验证域外系统性错值——决定性单点 (784,160,-408) gpu=0.045303289 vs cpu=-0.458333333（diff 5.036e-01，量级级差异非浮点舍入）；而 e2e 域（x≤63, y∈[-64,-49], z≤4）maxDiff=3.128e-07 全过——**e2e 域是 D23 盲区**。

**根因（最终锁定）**：`spline_eval` 边界外推（coord < loc[0] / coord > loc[n-1]）写成 `(splineValKind[valB]==0 ? splineValF[valB] : 0.0f)`——**嵌套 value（kind==1）直接返回 0.0，未递归求值**。vanilla `Spline.apply` L259/261 边界外推是 `value[0]+der[0]*(x-loc[0])`，端点 value 为嵌套样条时**必须递归求值**。触发：(784,160,-408) 的 spline55（factor 的 spline，locs=[-0.19,-0.15,-0.1,0.03,0.06]）coord（continentalness@c0）=0.060231412 **恰好 > 最后 loc 0.06** → 右边界 → vn=嵌套(spline54) → 0.0（参照应递归得 factor=4.524）→ 上层 entrances 链错 → fd 错。**e2e 域为何对**：域内 spline coord 全在 locs 范围内 → 正常 Hermite → 对；大坐标域 coord 恰好跨出末 loc → 边界嵌套 → 0。**D17 修复后遗留**（D17 只修 node_idx/val_begin 陈旧索引，未处理边界嵌套 value 的递归）。

**定位链（域扫描二分，非猜测）**：
1. throughput probe 16 chunks → top diff @ (784,160,-408)：先定位到「大坐标 chunk 域」（x=784 > e2e 的 x≤63）
2. domain probe 定点对比 → (784,-64,-408) 对、(784,160,-416) 对、(720,160,-432) 对 → **错误依赖具体 (x,z,y) 组合，不是简单坐标域**
3. z-scan（y=160 x=784）：z=-432..-412 全对、**z=-408/-404 错**（cz=2/3 格错）
4. y-scan（x=784 z=-408）：y=-64 对、y∈[-56,248] 几乎全错、y≥256 对（= 无地形常数分支 -0.02499）——**错误域 = 「y 中间层 + cz≥2」组合；正确域 = 常数分支层或 cz≤1**
5. 🔍 **y=72 反例（新嫌疑，后被根因解释）**：y=72 (cy=17) cz=2 对、y=160 (cy=28) cz=2 错——同 cz 同 cx 仅 cy 不同，若拆分/读取全对不应差异（未收敛于拆分/索引层）

**候选 fan-out 排除（❌，各一行）**：
- ❌ **H1 角点序**：interp 角点 delegate 顺序 GPU=sim 一致，排除
- ❌ **H2 cell 推导**：cx/cy/cz（整数除法 vs floorDiv）逐位核对无差，排除
- ❌ **H3 split 数值**：gpu_split_probe（纯 CPU）拆分数据无 NaN/无越界/cz 变化小数正确区分，排除
- ❌ **初判「缺 noodle_ridge_b 拆分行」**（grep 实证 split() 在 normals[191] 结束）——**证伪**：check_split_base.py 实证 192 个 normal 拆分实际生成（normals[160]=noodle@c0 base=8288）——误报来源 = 用全量序号对比纯 normal 的 normals[]
- ❌ **「双索引错位」**（gen_cpu 纯 normal 序号 0..191 vs gen_shader 全量 0..199，splitBase 错位 8）——**证伪**：数据来自**旧版 final_density.comp** dump（P2 修改前产物）；当前重新生成后 NORMAL_PACK[168]=8288 与 split 写、normal_meta 三方一致（check_two_alloc.py 0 处不一致 / check_meta_vs_splitbase.py 全 YES）——教训 ⑧：**对账必须基于当前生成产物，不能依赖旧 comp/spv 的 dump**

**求值分叉定位（决定性）**：sim（dbg_full_sim.py 复刻解释器）对 (784,160,-408) = 0.045303285 **与 GPU 完全一致** → **生成器产物 + 解释器共同逻辑 bug（不是 GPU kernel 特有）**；分量参照（DensityBuilder）：错点参照 sloped=-2.664 / factor=4.524，GPU/sim sloped 角点值 -0.0165（差 160 倍，结构性错）→ 嫌疑收敛 spline 链；node[54]（roughness@c0）拆分采样 -0.113109157 == CpuBackend 直接采样**逐位一致**（coord 正确）→ 分叉在 node[54] 之后：**node[22]/[33] SPLINE 大坐标域算出 0** → 对照 vanilla Spline.apply 逐行 → 边界外推分支的嵌套 value 用 0.0f 占位 → 最终锁定。

**修复（GPU 侧，dfc_gen.py `_spline_ssbo_glsl`）**：while 栈边界分支（i<0 / i>=n-1）遇嵌套 value 不再直接 0.0，改压子帧递归求值（新增 **stage 4=等边界 v0 / stage 5=等边界 vn**，回填后用子帧值做外推；与普通 Hermite 路径共用同一栈帧回填机制，无新增数组）。

**修复（sim 侧，dbg_full_sim.py 回归工具）**：显式栈移植同样的边界递归（stage 6/7 对应 GPU stage 4/5），但踩了两个**显式栈回填机制**的坑（GPU while 栈直接 outVal 回填无此问题）：
1. **outSlot 返回地址被覆盖**：压子帧时 `outSlot[sp]=-1` 清掉本帧自己的返回地址 → 深层嵌套完成时结果不回填祖父帧。修复：只改 stage 不覆盖 outSlot
2. **父帧 stage 被回填覆盖**：子帧完成回填 `stageStack[ps>>1]=2` 无条件覆盖 → 压 v0 子帧时父帧 stage 已设 1（等 v1），回填后被改成 2 → **跳过 stage 1（v1 求值）→ v1Stack 恒 0 → Hermite 用错值**。修复：父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），回填只写值不覆盖 stage
- **judge P1-1 追补**：审查发现 stage 6/7 完成路径仍保留原 L289/302 的 `stageStack[ps>>1]=2`（正是声称已修的同类 bug，normal-range 父帧的 v0 子帧为边界嵌套帧时仍会算错）→ 删除全部 5 处 `stageStack[ps>>1]=2`（grep 确认 0 残留）→ **verify_p11_recursive.py 显式栈 vs 递归版 Spline.apply 参照（vanilla 语义直译）1344 组合 0 mismatch**（覆盖边界触发域坐标 (784,160,-408)/(720,160,-432) 等）

**验证（seed 8576294172403134396，gpu_domain_probe / e2e）**：
- (784,160,-408)：0.045303289（错）→ **-0.458333343（对，diff 9.9e-9）**
- z-scan（y=160 x=784, z=-432..-404）：全部 diff 9.9e-9（原 z=-408/-404 错 0.5）
- y-scan（x=784 z=-408, y=-64..312）：y=80-120 diff 5e-7~3e-6（float 精度，原 0.03-0.5）；y≥128 全 9.9e-9；y≥256 常数分支 1.1e-9
- e2e 回归：maxDiff=3.128e-07 / avgDiff=1.097e-08 **与基线逐位一致（零回归）**（e2e-A5 落盘：pipeline 80.1s、TOP00 i=1004 pos=(44,-49,0) diff=3.128e-07；D23 修复验证记录 pipeline 94.4s，均达标）
- sim：eval_df(784,160,-408)=-0.458333333 ✓；sim vs e2e-A5 全量对拍 maxDiff=5.7e-9 ✓ 无回归；dbg_full_sim 四点全对齐
- I5 复测：各 chunk diff 1e-6~4e-6（正确性恢复），吞吐 24-32x 保持

**教训（D23 综合，完整版见 gpu-accel-errors.md D23 段 + discovered #14）**：
1. **e2e 单域验证是盲区制造机**：域内全过 ≠ 域外正确；吞吐/性能探针必须顺带做 diff 抽查（多 chunk / 多 cell / 多 y 层）
2. **边界分支是「执行不到」类 bug 的温床**：e2e 域触发不到的分支（边界外推、嵌套边界）必须用跨域采样覆盖
3. **模拟器复现 0.045 = 生成器+解释器共同逻辑 bug**（不是 GPU 特有）——「GPU 特有 vs 共同逻辑」二分法先做
4. 与 vanilla 逐行对照是最后手段也是最终手段：**Spline.apply 的边界外推是递归求值，不是取 0**
5. **显式栈移植纪律**：「返回地址（outSlot）」与「父帧恢复点（stage）」是两套状态——压帧时各设一次，回填时只写数据槽，任何「回填时顺带改父帧 stage」的优化破坏等待语义

### 🔍 遗留项（未立项 / 待复核）

- **judge P1-3 复核**：I5 吞吐已重跑落盘 cmd-output/throughput-I5-*.txt（1/4/16/64 chunks，64 chunks 档位 ~10min+），复核数字后闭合
- **judge P2-2（低危）**：shaderFloat64 未启用 + GpuDensityEngine 构造失败 `exit(1)` 无 CPU fallback（wg_create 已 try/catch 返回 nullptr 走 CPU；引擎内部 exit 需复核；shader 无 fp64 需求因 CPU 预拆分）
- **confirmed 待用户拍板**：judge 结论「P1 全关后可向用户推荐 confirmed」；知识库闭环（P1-4）= 本条目 + discovered #14 + gpu-accel-errors.md D23 判错经验段
