# Worker：C2ME AQUIFER_PREFILL（及 FLAT_CACHE_PREFILL）kernel 实现细节调研（260908-02）

> 角色：core.worker（分析解读）。任务来源：scout-b 待深入点 1/2（scout-b-c2me-coverage.md L85-88）+ convergence-main §二「est 每点输入是坐标派生量」待验假设钉死。
> **证据级别：源码级（Full 静态阅读，非 doc/issue 级）**。证据基线 = 本仓库 vendored C2ME-fabric 完整源码树 `versions/1.20.1/data/C2ME-fabric/`（git HEAD `615baf860a2c0ebdc39e63b408fe1c3539e5af9d`，2026-08-02，`c2me-opts-accel-opencl` 模块），下文所有行号均指该 vendored 树。
> 外部网络（web_search 仅能命中 modrinth/发布页，无源码细节；本会话直连 GitHub 被沙箱拒绝 curl exit 35）——但 vendored 源码树即权威，无需降级到 doc 级。
> 降级声明：静态阅读，无运行时 trace；「推导值」段落为本 worker 按源码公式推算，非实测。

## 关键文件索引

| 文件 | 内容 |
|---|---|
| `c2me-opts-accel-opencl/src/main/resources/clsources/c2me_opencl_ext_math.cl` | aquifer kernel 本体 + aquifer 数据结构（.cl 手写部分） |
| `.../opencl/common/gen/CLServerBatchedBiomeNoiseContext.java` | dispatch 现场（网格大小、攒批、readback） |
| `.../opencl/common/gen/CLDataUtil.java` | CPU 侧 buffer 组装（aquifer 头/fluidLevelSampler/surface height cache） |
| `.../opencl/common/gen/cache/Stage1Cache.java` | FLAT_CACHE_PREFILL + ESTIMATE_SURFACE_HEIGHT 的 GPU 预填充与 readback |
| `.../opencl/common/compiler/OpenCLCGen.java` | flatcache/cache2d/interpolator prefill 函数生成器 |
| `.../opencl/common/util/OpenCLStructs.java` | Java MemoryLayout：aquifer_data_t 等 |

## a. AQUIFER_PREFILL 每工作项算什么 / 输入多少字节

**每工作项 = 1 个 aquifer 采样点（非 block、非 cell 角点）**。aquifer 点网格 = 16×12×16 block 一格（vanilla 同款）。

kernel 本体：`c2me_opencl_ext_math.cl` L1570-1658 `kernel void aquifer_data_prefill(const_data, rw_data)`：

- 每工作项坐标：`curX = data->startX + get_global_id(0)`，`curZ = startZ + get_global_id(1)`，`curY = startY + get_global_id(2)`（L1599-1601）。
- **每点工作内容**：
  1. 用 `random_state_split_coords + nextIntBounded(10/9/10)` 派生该点随机偏移 (r0,r1,r2)，打包成 **1 个 uint16** 写入 `packedBlockPositions[index]`（L1604-1613）。
  2. 对该点做 13 邻 chunk 偏移循环（`__aquifer_chunkPosOffset[13][2]`，L1494），每次查 **surface height cache**（`chunkNoiseSampler_estimateSurfaceHeight` → 读 `offset_estimateSurfaceHeight` 预填充 int32 缓存，L1226-1251）+ 查 fluidLevelSampler 表，早退或落到 `__aquifer_getFluidBlockY/getFluidBlockState`，最终写 **1 个 `aquifer_fluidlevel_t`（int32 y + int32 blockState = 8 字节）** 到 `waterLevels[index]`（L1653-1657）。
- **每点 GPU 输入上传量 = 0 字节**。kernel 只有两个 buffer 指针参数（const_data/rw_data，L1572）；每点的全部「输入」是 `get_global_id` 坐标 + 共享 buffer 里的结构头（`aquifer_data_t` 44 字节头 + randomDeriver 状态 + 两个共享表）。没有 per-point 数组上传。

**aquifer_data_t 结构**（.cl L639-652 / OpenCLStructs.java L331-354）：startX/Y/Z、sizeX/Y/Z、samplingYLowPassCutoff、randomDeriver 偏移、posIdx_len、waterLevels 偏移、packedBlockPositions 偏移 —— 共享一次，随 rw_data buffer 上传（`clEnqueueWriteBuffer(rwBuffer, rwData)`，Stage1Cache.java L127）。

**fluidLevelSampler 表 = CPU 预计算**：CLDataUtil.java L253-268 `MARKER_fluidLevelSampler` → `fluidLevelSamplerCreate(minimumY, height, fluidLevelSampler, mappings)`，即按每 block-Y 一条 `aquifer_fluidlevel_t`（8B）在 CPU 算好随 rw_data 一次性上传；overworld 384 层 ≈ 3KB。**不是坐标参数化在线计算，是「CPU 预计算小表 + GPU 查表」**。

**主内核消费形态**（证明 prefill 服务于逐 block aquifer）：`df_noise_kernel` 内逐 block 调 `aquifer_sample`（.cl L1968），它读 prefill 好的 packedBlockPositions（uint16/点），现场跑 `math_aquifer_refreshDistPosIdx_global`（12 候选点距离排序，输出 4×uint64 packed {dist|idx|posIdx}，.cl L698-762）再 `__aquifer_applyPost`。即 **GPU 逐 block aquifer 的每 block 额外输入 = 16 字节 packedBlockPositions（12 格共享）+ 0 上传（全在 device buffer）**。

## b. 输入如何喂 GPU（对照 D24 判据「每点输入必须小」）

C2ME aquifer 的喂法 = **共享小表 + 坐标参数化，无任何 per-point/per-block 数组上传**：

1. **rw_data 单 buffer 一次性上传**（Stage1Cache.java L127）：内含 worldgen_params_t 头、aquifer_data_t 头（44B）、randomDeriver 随机状态、fluidLevelSampler 预计算表（CPU 算，~3KB/批）、estimateSurfaceHeight 缓存（int32/生物群系列，来自 Stage1 GPU readback，CLDataUtil L200-210）。
2. **每点/每 block 数据全部坐标派生**：packedBlockPositions 由 prefill kernel 现场派生（随机数 + 打包 uint16）；距离/索引现场算。**「est 每点输入是坐标派生量、非 8672 floats split」→ 由源码证实为事实**（convergence-main §二待验假设可以钉死：C2ME aquifer 路径根本不存在 split 全量上传概念，输入契约 = 坐标 + 两个预计算小表）。
3. D24 判据核对：aquifer 采样点每点输入 = 3 个 global_id int（12B，甚至可只传 chunk 原点）+ 共享表摊销 ≈ 0；满足「每点输入必须小」**且**满足 scout-c J3「专用 kernel 形态」（无解释器分派，直接手写 vanilla 移植直排代码）。

## c. dispatch 形态与 readback 形态

dispatch 现场：CLServerBatchedBiomeNoiseContext.java L264-291：

- **攒批单位 = BATCH_SIZE×BATCH_SIZE chunk 批**（L80：`BATCH_SIZE = Config.useSmallerBatches ? 2 : 4`，注释 "must be power of 2, determines axis size"）。
- **网格 = 整个批次的 aquifer 点三维网格一次 dispatch**：`globalWorkSize = (endX-startX+1, endZ-startZ+1, endY-startY+1)`（L277-280，注意 dim1=z, dim2=y），**localWorkSize = null**（无显式 workgroup，L287）。
- 网格范围含 vanilla ±5 block 边距（L268-274，`getStartX()-5` / `getEndX()+5-1`）。
- **推导值（overworld 384 高、BATCH=4）**：X/Z ≈ 6（64 block + 5+5 边距 → 5 格 + 端点），Y ≈ 35 → **约 6×6×35 ≈ 1260 工作项/批 ≈ 315 点/chunk**；每点产出 uint16 + 8B ≈ 10B → 产出 ~12KB/批。此为公式推算，非实测。
- **readback：AQUIFER_PREFILL 无 readback**——结果写 device 端 rwBuffer，随后 `df_noise_kernel`（同 commandQueue 内 event 链依赖，L282/287 prevEvent）直接在 GPU 上消费；真正回 CPU 的只有 blockOutBuffer（`clEnqueueReadBuffer` block 结果，L355，每 block 1 byte：blockState 7bit + needsFluidTick 1bit，.cl L1977）。
- kernel 内越界/未配置直接 `__builtin_trap()`（.cl L665, L1577, L1588）。

## d. FLAT_CACHE_PREFILL 矛盾核对（scout-b 遗留：GPU kernel vs CPU 预填充）

**两条表述都对，是同一管线的两段——不矛盾**：

1. **GPU kernel 预填充（Stage1Cache，每 cache-chunk 区一次）**：OpenCLCGen.java L390-410 为每个 flat_cache 生成 `df_flatcache_prefill_<node>` 函数 + L490 包装 kernel `df_flatcache_prefill_kernel_<i>`（每工作项 = 1 个 biome 对齐列：`offsetX/offsetZ = get_global_id`，1 次 delegate 调用，结果 double 写 buffer + extra_out）；dispatch 在 Stage1Cache.java L131-164（workSize = CACHE_WIDTH²，local 16×16，**2D**），随后 **readback 到 CPU**（L208，`flatCacheOutBufferData = prefills × CACHE_WIDTH² × 8B`）。
2. **回传后再进 rw_data（主内核 GPU 只读）**：CLDataUtil.java L288-300 `MARKER_cacheLike_flatCache` 把 Stage1Cache 读回的 `flatCaches[]` 再写入 rw_data buffer；主 noise kernel 只查 buffer，**miss 即 `__builtin_trap()`**（c2me-dfc-review.md §四.2 的表述，与源码一致——主 kernel 内 flatcache 路径无 miss 兜底计算）。
3. 结论：**「GPU kernel 预填充」= Stage1 计算/readback 层；「CPU 预填充」表述指的是「主内核见到的 buffer 由 host 端组装（数据源自 Stage1 GPU 输出）」**。管线为 GPU算→CPU收→随rw_data再上传→GPU只读。c2me-dfc-review.md §五的说法应修订为此形态。
4. 附带：CACHE2D_PREFILL 是独立 3D-dispatch 2D 网格 kernel（CLServerBatchedBiomeNoiseContext L293-314，workSize = horizontalSize×horizontalSize×cache2dPrefills），与 FLAT_CACHE 分列两个 ProgramType（OpenCLCGen.java L165-170）。

## e. C2ME aquifer kernel 实测收益数字

**未找到**。vendored 仓库内无 benchmark 数据（全库 grep benchmark/speedup 无 aquifer 相关命中）；web_search 只命中 [Modrinth c2me-ocl 发布页](https://modrinth.com/mod/c2me-ocl)、[C2ME-fabric GitHub](https://github.com/RelativityMC/C2ME-fabric)、[mcmod.cn 页面](https://www.mcmod.cn/class/28450.html)，无 aquifer kernel 级收益数字。**降级声明：此项仅「无公开记录」级证据，非「收益为零」**。

## 对立项输入的证据小结（只摆事实）

- aquifer GPU 化在 C2ME 的形态 = 「~315 点/chunk、每点 0 上传字节（坐标派生 + 共享小表）、专用直排 kernel、结果 device 内流转不 readback」。
- 与 D24 死局（8672 floats/点）完全不同构；与 D27 被否的「解释器式 kernel」也不同构（aquifer kernel 是手写专项化，无逐节点分派）。
- est（surface height）在 C2ME 里本身也是 GPU kernel 预填充 + cache（Stage1Cache.java L183-194 `chunkNoiseSampler_estimateSurfaceHeight_prefill_indep`，CACHE_WIDTH² 网格），aquifer kernel 通过共享 cache 表消费它——「aquifer 依赖 est」在 C2ME 是缓存依赖不是每点重算。
