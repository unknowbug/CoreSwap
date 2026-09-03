# X2 重大转向 + 技术方案审查意见（core.judge，260903-05）

- 审查对象：路线② 逐块 GPU 接线后转向 X2（shader 暴露 5 channels @ cell corners → Rust trilerp + combine）的重大转向决策（MUST 节点）+ X2 技术方案。
- 审查材料：route2-260903-05.md 全程记录；gpu-accel-errors.md D24/D25（含 L598-656）；gpu_density_engine.cpp/.h；dfc_gen.py（gen_shader/interp_funcs/perSample 布局）；terrain.rs（DensityMacroSampler/TranspilerDensity）；vanilla_density_functions.rs:162-172。
- 审查性质：只读审查，只出意见，不改 status。status 判定建议见文末（最终由用户拍板）。

## A. 转向决策证据链 —— **通过**

1. **历史一致性成立**：实测 25.4s/chunk（258µs/pt）与 D24 C++ I6 实测（24 chunks 11 分钟未完成 ≈ 27s/chunk，CPU 基线 2.5 分钟）同量级吻合，非新异常。D24 根因定性（split 全量上传带宽死局：98304 点 × 8672 floats × 4B ≈ 3.4GB/chunk）机制清楚，与本次「逐块形态慢」互证。判死结论由两条独立实现（C++ I6 / Rust FFI 逐块）到达同一数字——这是强证据。
2. **语义 PASS 可信**：98304 点 major_diff(>1e-4)=0、max_diff=5.729e-6（f32 ULP 级），覆盖整 chunk 全点非抽样；P2 通路本身（handle 缓存/fallback/门控）已工作。「逐块不可行、通道无损」两个事实都成立，转向前提扎实。
3. **遗留不符**：P0 judge 后仍留 0.61× 并行异常未决（候选已扩为 ≥4 类，P4 待复测）——不阻塞转向（逐块判死不依赖它），但 X2 单线程化 GPU 提交设计正确依赖了 P0 的 fillMtx+fence 全串行结论，该结论目前是 Degraded 静态级，confirmed 前须补数据层验证（已在 route2 记录中声明）。

## B. X2 关键语义声明 —— **成立，但「必然一一对应」措辞过强，探针封堵是必要条件而非可选**

1. **语义结构本身成立**：
   - shader interp 结构：每个 Interpolated 节点 → interp_N（8 角点 delegate + shader 内三线性），顶层 eval_df 分发 interp_0..interp_4（dfc_gen.py L634-638），与 vanilla final_density = combine(interp_0..4) 结构吻合。
   - 关键等价性已验证过：**网格节点是其 cell 的 (0,0,0) 角点 → 在角点坐标处 interp 退化为 corner-0 delegate 取值**（dfc_gen.py L2364-2365 注释 + verif_grid_cache_correctness.md 单实例 corner=0 等价性证明）。因此「GPU 在 1225 角点出 channels + Rust trilerp + combine」与 Java NoiseChunk 语义（cell 角点采样 + 块级三线性）同构——这正是已 diff0 验证的 Rust transpiler 路径（vanilla_density_functions.rs fill_cell_corner_densities_final_density / compute_final_density）的 GPU 版。
   - D25 否决的「方案 C」是 interp **内容树**在角点求值（8 份冗余实例绑定固定角点坐标、无法算任意点）——X2 求值的是顶层 **Interpolated 节点值**（interp_N 本身就支持任意点坐标进 cell），结构不同，D25 判死不被 X2 踏线。YClampedGradient 等 y-依赖项在 vanilla 里位于 Interpolated argument **内部**（随 8 角点坐标求值），不构成「channel 里混入须逐块求值的非线性项」问题。
2. **疑点（对应关系非构造保证）**：shader interp_0..4 的实例编号来自 **Python dfc_gen 对 JSON 树的遍历序**（后序，_df_interp_node 追加序），Rust macrolize_channels 的 channel 序是 **Rust 对同一树的独立遍历序**。两套实现、两个遍历——「数量=5、顺序一一对应」是高度可能但**非必然**（缓存节点剥离、遍历序差异、未来 JSON 变化都可能破坏）。另外 cache_2d/cache_once/cache_all_in_cell 在生成器侧先剥离再判 Interpolated（L363-366）——语义正确但属隐含前提。
3. **探针封堵判定**：方案任务④「通道对拍」**足以封堵**，前提是：
   - 对拍必须是**逐通道**（5 通道各自 GPU vs CPU 参照），不能只对拍 combine 后最终值（combine 后 min 会掩盖单通道错位/交换——min(0.64 squeeze, noodle) 对通道互换部分钝感）；
   - 含**计数断言**（n_interp==5 == channels.len()==5，不等即 fail-fast）；
   - 覆盖域按 C/D 节跨域要求（D23 盲区教训）。
   建议把计数断言做成 GpuChannelDensity::new 的**常驻检查**而非仅探针一次性。

## C. 改动面评估 —— **总体可控，方案遗漏 5 个工程点（须补入计划）**

方案描述的三大块（dfc_gen 新增 channels 输出 shader 生成、不动 final_density.spv、engine/FFI 多通道出口 + Rust GpuChannelDensity）与代码实况核对一致：interp_funcs/eval_df_base 结构可直接复用（channels.spv 顶层只需调 interp_0..4 写 5 floats/pt，剥离 combine 子树，比 final_density.spv 更简单）；engine .h 的 perSample()/splitTotal() 已暴露布局参数；Rust 侧 DensityMacroSampler::build_slices/sample_interp_impl 结构可直接换成 GPU 通道源。遗漏点：

1. **[C1·中] 上传带宽须实测，不能算术外推**：1225 角点 × splitTotal(8672) × 4B ≈ **42.5MB/chunk** split 上传 + 5×readback。「corner-probe 5µs/pt ≈ 6ms/chunk」是基于 768 点 wg_fill_density（I5，27MB/chunk）的外推；D24 教训 #2 明确「吞吐证据不能外推采样密度」。42.5MB @ PCIe ~16GB/s 理论 ~2.7ms + dispatch/同步开销，6ms 量级可信但必须以实测为准——这恰是 X2 立项的生死判据。
2. **[C2·中] 双 spv 双引擎成本**：GpuDensityEngine ctor 绑定单 spvPath。channels.spv 与 final_density.spv 并存 → 若两个 engine 实例 = create ~75s × 2（seed 变更再付）。建议：同 engine 双 pipeline，或明确接受 150s 一次性成本并写入声明；同时 X2 通路与 P2 逐块通路（同 seed 共存）的 handle 缓存键要含 spv 身份。
3. **[C3·低] FFI ABI 与 out 布局**：新增 fill_channels 出口（n×5 floats）不动旧 fill/旧 spv（方案已声明，正确）；out 布局（point-interleaved [p0ch0..p0ch4, p1ch0..] vs channel-planar）必须在 ABI 文档写死并与 Rust 读取序一致——这是本项目「索引/布局错位」三犯区（D14/D23 教训 #5：布局结论必须基于当前生成产物 dump，不靠推断）。
4. **[C4·低] 角点坐标与边界语义**：corner = (chunkX*16+ix*4, minY+iy*8, chunkZ*16+iz*4)，gx=gy=5×49×5=1225；顶角 y=320=minY+height（含上边界，noodle 通道 guard v<321 恰覆盖）。shader `minY=-64` 硬编码 = overworld 假设——与数据驱动铁律一致性须标注为已知升级点（对齐 C++ wg_create 参数化方向）。
5. **[C5·低] 门控命名与回退链**：X2 须独立 env（如 WG_GPU_CHANNEL），与 P2 的 WG_GPU_DENSITY（逐块，已判死形态）分离，默认关零退化；失败回退链（dll/spv 缺失→None→transpiler/macro）对齐 P2 既有设计。另 P3 的 block_probe 启动期异常（0xE06D7363，已证与钳制修复无关但未查明）是开放项，不阻塞 X2 但不得遗忘。

## D. 验证判据与 §9.7 —— **判据方向完备，两处须预先约定**

1. 判据三件套（通道对拍 major_diff=0 / 端到端 ≥256 chunks vs Java / 门控关零退化）方向正确，且端到端大样本符合 2026-08-29 端到端铁律（≥256 chunks、排除冷启动、取中位数）。
2. **[D1·中] 验证域覆盖必须跨域**：D23 教训（e2e 单域 = 盲区制造机，域内 3.128e-07 全过、域外错 0.5）——≥256 chunks 的采样域 MUST 含远坐标 chunk、多 y 层（含高层边界外推分支、y=320 顶角）、负坐标；性能探针默认带逐点 diff（教训 #2）。
3. **[D2·中] §9.7 口径预先声明**（三要素，开工前写死，防 M16 类不可比误读）：
   - 载体：通道对拍 = GPU f32 channels vs Rust f64 macrolize/generated 参照（期望 max_diff ~f32 ULP，非 0；major_diff 门 1e-4）——**与 final 级 3.128e-07 口径不可比，不得混用**；端到端 = block_probe/存档写入口径。
   - 覆盖面：声明采样域（坐标范围/y 层/chunk 数）。
   - 可比性：与 D24（27s/chunk）、I5（22-39x@768 点）历史数字对比时注明采样密度差异。
4. 建议补第四判据：**X2 生死判据前置**——实测 ms/chunk（含上传）未达「显著快于 Rust CPU 路线 ~10ms 级」即止（可切换架构下零退化铁律兜底，但避免重演「语义 PASS 但性能死」的第二轮）。

## CONCERN 清单（按严重度）

| # | 严重度 | 内容 | 处置建议 |
|---|--------|------|----------|
| B1 | 中 | 通道↔interp 一一对应非构造保证（两套独立遍历），combine 后对拍会掩盖单通道错位 | 逐通道对拍 + 计数断言常驻（GpuChannelDensity::new fail-fast），探针覆盖跨域 |
| C1 | 中 | 42.5MB/chunk split 上传未实测，「6ms」是外推；X2 立项判据恰在此 | 首个里程碑 = 最小通道 shader + 实测 ms/chunk，未达标即止 |
| C2 | 中 | 双 spv 双引擎 create 75s×2 | 同 engine 双 pipeline 或明示接受成本；handle 缓存键含 spv 身份 |
| D1 | 中 | 验证域若沿用 e2e 单域 = D23 盲区重演 | ≥256 chunks 跨域采样（远坐标/多 y 层/负坐标），性能探针带 diff |
| D2 | 中 | §9.7 口径未预约定：通道级 f32 口径 vs final 级 3.128e-07 口径不可比 | 开工前落三要素声明（载体/覆盖面/可比性） |
| C3 | 低 | FFI out 布局未定义（interleaved/planar） | ABI 文档写死 + 当前产物 dump 核对（D14/D23 教训） |
| C4 | 低 | shader minY=-64 硬编码（overworld 假设） | 标注数据驱动升级点 |
| C5 | 低 | X2 与逐块门控混同风险；P3 block_probe 启动异常未决 | 独立 env 门控；P3 异常列开放项跟踪 |
| A1 | 信息 | P0 0.61× 并行异常未决（≥4 候选），其「全串行」结论为 Degraded 静态级 | X2 单线程化设计可先行；confirmed 前补 P4 数据层复测 |
| A2 | 信息 | §15.4 取代链缺失：P1/P2 语义钉子「否决角点+Rust 插值」被 X2 部分取代（final 插值非法 ↔ channel 插值合法，范围不同） | route2 记录加 supersedes 双指针一行，原文不改 |

## 结论与建议

- **审查点 A（转向决策）：通过**——证据链充分（历史一致 + 语义 PASS + 机制定性吻合），重大转向判定成立。
- **审查点 B（语义声明）：成立（有条件）**——结构等价性有既有验证支撑（corner=0 等价 + Rust transpiler diff0 同构）；「必然一一对应」应降格为「经逐通道对拍+计数断言封堵后成立」。
- **审查点 C（改动面）：可控，补 5 工程点后可开工**——C1（实测带宽）为立项生死判据，建议作为 X2 第一个里程碑前置。
- **审查点 D（判据）：方向完备**，补跨域覆盖（D1）与 §9.7 预声明（D2）后判据闭合。
- **建议 status 处置**（意见，用户拍板）：转向决策建议批准；X2 技术方案建议以「candidate + 首里程碑=C1 实测门」进入实施；本审查意见不授予任何 confirmed。
