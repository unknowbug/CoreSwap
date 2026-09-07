# Scout-B：C2ME OpenCL kernel 覆盖面 vs CoreSwap 已试 GPU 介入点对照（260908-02）

> 角色：recode.scout（只读勘探，只摆事实不下结论）。
> 任务：对照 C2ME GPU(OpenCL) 路线实际覆盖的 kernel 类型，为「CoreSwap 还没试过的 GPU 介入点」提供外部可行性参照。
> 证据基线：仓库内既有调研（.investigations/perf-rework/gpu-accel-errors.md G4 条目 L469-473、c2me-dfc-review.md、gpu-route-decision.md、D24/D25/D27）+ 外部旁证（web_search，标 URL）。

## 一、C2ME kernel 清单（仓库内已记录的机制事实）

来源：gpu-accel-errors.md G4（L469-473），引用 `OpenCLCGen.java` L442-556；总清单 7 项：
**ESTIMATE_SURFACE / AQUIFER_PREFILL / NOISE_KERNEL / FLAT_CACHE_PREFILL / INTERPOLATOR_PREFILL / BIOME_MULTINOISE / arena**。

分发机制（G4 记录）：
- 拆 7 个**独立内核程序**（每 kernel 单独编译，避免单巨型内核编译爆炸）。
- **预编译二进制随 mod 分发**：`config/c2me-shader-delivery/*.tar.zst`，`clCreateProgramWithBinary` 秒级加载，失败才现场编译。

精度/运行时（c2me-dfc-review.md 2026-08-13 记录）：
- 全 double（F64）+ `#pragma OPENCL FP_CONTRACT OFF`（关 FMA 保精度）。
- flat_cache miss 即 `__builtin_trap()`（`CacheLikeNodeOpenCLCEmitter` 的 FLAT_CACHE/CACHE2D/INTERPOLATED 分支）——**flat_cache 必须 CPU 端预填充，否则崩**。
- 大量 Intel/AMD/Mesa/NVIDIA 驱动 workaround blocklists（OpenCL 生态差）。
- DFC/OpenCL 只在 Fabric 版（C2ME-forge 停 1.16.5 无此模块）。

## 二、逐 kernel 对照

### 1. NOISE_KERNEL（finalDensity 主求值）
- **机制（仓库已记录）**：DF 树 → AST（McToAst）→ OpenCL C 代码生成，全 F64。spline 用「const 数组 + 二分查找 + switch-case + value 函数调用」全非递归形态（c2me-dfc-review.md §七）。
- **数据形态**：每工作项 = 1 个点的 delegate 树求值（函数体小，非展开式）。
- **CoreSwap 是否已试**：✅ **已试且已否**——解释器式 finalDensity kernel（dfc_gen GLSL 解释器）；I5 wg_fill_density 角点批量曾得 22-39x（corner 768 点/chunk），但 D27（260908-01）摊销曲线证伪：per-pt ~0.22ms 主导、扩批 32× 只摊 13%，GPU vs CPU 8t 慢 ~38×（门槛原义口径）/ ~170×（总预算口径），现引擎形态下无解（gpu-accel-errors.md D27 L691-727）。
- **证据**：gpu-accel-errors.md G4/D27；c2me-dfc-review.md。

### 2. INTERPOLATOR_PREFILL（interp 网格角点预填充）
- **机制（仓库已记录，最细）**：每个 interpolator 生成 `df_interpolator_buffer_prefill_<name>` 函数，**按 cell 网格 dispatch**（`get_global_id = cellX/cellY/cellZ`），每工作项算 **1 个角点** 的 delegate 采样 → 写入 interpolator buffer；主内核只读 buffer 做三线性。角点采样表达式 = 1 次 delegate 调用（几十字节），无 8 角点内联巨型展开（gpu-accel-errors.md G4 L471）。
- **数据形态**：每工作项 1 角点 × 1 次 delegate 采样；批量单位 = cell 网格（C2ME 侧 cell 数/chunk 未在仓库笔记记录，待源码核对）。
- **CoreSwap 是否已试**：✅ **已试**（部分形态）——I5 wg_fill_density = GPU 算 768 角点/chunk（曾 22-39x，后被 D27 摊销证伪现行态）；方案 C（GPU 算 5 个 interp 内容树 1225 角点 + CPU 三线性）被 D25 否——根因是 CoreSwap shader 的角点分组噪声结构（8 份冗余实例绑定固定角点坐标）与共享网格求值不兼容，**不是 C2ME 形态本身的否定**。C2ME 形态（每角点一次 delegate 调用、参数化坐标）正是 D25 修复方向「共享实例 + 每点坐标参数」的外部已验证等价物。
- **证据**：gpu-accel-errors.md G4/D25/D27。

### 3. FLAT_CACHE_PREFILL（flat_cache 预填充）
- **机制（仓库已记录）**：C2ME 把 flat_cache（CacheLikeNode FLAT_CACHE/CACHE2D）作为独立 kernel 预填充进 buffer；主 kernel 读 buffer，miss 即 `__builtin_trap()`（c2me-dfc-review.md §四.2）。注意：c2me-dfc-review.md §五又写「flat_cache 宏观噪声 CPU 预填充 + GPU 只读」——**两个表述（GPU kernel 预填充 vs CPU 预填充）并存于仓库笔记，需源码核对哪条为准**（可能是「CPU 兜底 + kernel 形态皆有」，未核实）。
- **数据形态**：每工作项 1 个 cache 条目（biome 对齐粒度，CoreSwap 侧 flat_cache 差 0.01-0.1 教训记录于速查表）。条目数/输入量未在仓库笔记记录。
- **CoreSwap 是否已试**：⚠️ **部分接触、未做过独立 GPU 评估**——CoreSwap 的 flat_cache 只在 CPU/GPU shader 内作为查询节点出现（速查表「flat_cache 是 biome 对齐，不是剥掉」），从未把「flat_cache 预填充」当独立 GPU 批量任务评估。
- **证据**：c2me-dfc-review.md §四/§五；gpu-accel-errors.md 速查表。

### 4. AQUIFER_PREFILL（含水层预填充）
- **机制**：仓库笔记**只有 kernel 名**，无机制细节记录（G4 清单一行）。
- **数据形态**：未记录（vanilla 侧含水层 = 每 chunk 局部流体屏障采样，CoreSwap 台账有「含水层 = carvers 阶段产物」历史认知，见 000-架构设计复盘引用；输入量/每点数据未记录）。
- **CoreSwap 是否已试**：❌ **从未做过 GPU 评估**——CoreSwap GPU 线路只碰过 finalDensity/块级 fill/interp 角点（D24/D25/D27）。
- **证据**：gpu-accel-errors.md G4 L472（仅名字）。

### 5. BIOME_MULTINOISE（生物群系多噪声判定）
- **机制**：仓库笔记只有 kernel 名，无细节。
- **数据形态**：未记录（CoreSwap 侧参照：biome 判定输入 6 维参数 + SearchTree，WG_BIOMEDUMP 探针存在；C2ME 侧每点数据量/批量形态未记录）。
- **CoreSwap 是否已试**：❌ **从未做过 GPU 评估**——CoreSwap biome 全 CPU（SearchTree 移植 + st_bug_test 单元测试，AGENTS.md §五）。
- **证据**：gpu-accel-errors.md G4 L472（仅名字）。

### 6. ESTIMATE_SURFACE（地表高度估计）
- **机制**：仓库笔记只有 kernel 名，无细节。
- **数据形态**：未记录（vanilla 侧 = 每列高度估计用于 surface/初始 y 定位；输入量未记录）。
- **CoreSwap 是否已试**：❌ **从未做过 GPU 评估**。
- **证据**：gpu-accel-errors.md G4 L472（仅名字）。

### 7. arena（内存分配竞技场）
- **机制**：kernel 程序之一，形态为共享分配 arena（非算法 kernel）。仓库笔记无更多细节。
- **CoreSwap 是否已试**：❌ 无对应物（CoreSwap shader 用 SSBO/const 数组静态布局，无 arena 机制）。
- **证据**：gpu-accel-errors.md G4 L472（仅名字）。

## 三、C2ME 光照 / feature / carver 是否有 GPU 路线

- **仓库内旁证（弱，未核实）**：gpu-accel-errors.md L737「C2ME 的 GPU 路线只做 noise/density，未做光照」——原笔记自标「旁证（弱，未核实）」。
- **kernel 清单侧证**：7 kernel 全部属于 noise/density/biome/aquifer/surface 的 worldgen 噪声计算域，**无光照、无 feature placement、无 carver kernel**。
- **外部旁证（web_search，2026-09-08）**：C2ME OpenCL 加速以独立实验模块形态存在（「C2ME OpenCL Acceleration Module」，modrinth 独立条目，alpha/beta 早期预览版），主 mod 页面未宣传光照/feature/carver GPU 化。来源：[Modrinth C2ME-ocl](https://modrinth.com:8443/mod/c2me-ocl/version/nbDoBmhu)、[GitHub RelativityMC/C2ME-fabric](https://github.com/RelativityMC/C2ME-fabric)、[C2ME-neoforge](https://github.com/relativitymc/c2me-neoforge)。未检索到任何 C2ME 光照/feature/carver GPU 实现的说明。
- **事实陈述**：现有证据（仓库笔记 + kernel 清单 + 外部检索）一致指向「C2ME GPU 路线不覆盖光照/feature/carver」；未做 C2ME 源码级穷举核对，保留为「无反证」而非「已证实不存在」。

## 四、对照表摘要

| C2ME kernel | 机制/数据形态记录深度 | CoreSwap GPU 评估状态 |
|---|---|---|
| NOISE_KERNEL | 深（DFC→OpenCL C，F64，每点 1 次 delegate） | ✅ 已试已否（解释器式 kernel，D27 摊销不可达） |
| INTERPOLATOR_PREFILL | 深（cell 网格 dispatch，每工作项 1 角点 1 次 delegate） | ✅ 已试（I5 角点批量曾 22-39x；方案 C 被 D25 结构不兼容否） |
| FLAT_CACHE_PREFILL | 中（独立 kernel 预填充 + miss 即 trap；CPU/GPU 预填充两表述并存待核） | ⚠️ 只作 CPU 侧查询节点，未做独立 GPU 评估 |
| AQUIFER_PREFILL | 浅（仅名字） | ❌ 从未评估 |
| BIOME_MULTINOISE | 浅（仅名字） | ❌ 从未评估 |
| ESTIMATE_SURFACE | 浅（仅名字） | ❌ 从未评估 |
| arena | 浅（分配机制，非算法 kernel） | ❌ 无对应物 |
| 光照/feature/carver | —（清单无此 kernel，外部无说明） | —（CoreSwap 光照 GPU 不立项已有独立三条理由，gpu-accel-errors.md L729-739） |

## 五、待深入点（交 worker/主会话）

1. AQUIFER_PREFILL / BIOME_MULTINOISE / ESTIMATE_SURFACE 的输入数据量/每点数据/批量形态——仓库笔记均只有名字，需读 C2ME-fabric 源码 `OpenCLCGen.java` 相应 emitter 才能回答「CoreSwap 从未试过的介入点」的量级问题。
2. FLAT_CACHE_PREFILL 的「CPU 预填充 vs kernel 预填充」两表述矛盾核对（c2me-dfc-review.md §四.2 vs §五）。
3. C2ME 预编译二进制分发（shader-delivery tar.zst）与 Vulkan VK_KHR_pipeline_binary 的对应可行性（G4 已提示，gpu-reentry 主线课题）。
