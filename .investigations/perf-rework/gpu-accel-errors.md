# GPU 加速预研：错误与根因清单（重点记录，2026-08-13）

> 用户观点（本文件存在的理由）：**错误信息 + 探明「为什么错」的过程，比验证通过的结果更有价值。**
> 本文件把所有错误按「现象 → 根因 → 定位 → 修复」完整记录，作为后续排查的索引。
> 结论性验证数据见 gpu-accel-findings-summary.md；本文件只记「错在哪、为什么错」。

---

## A. 精度类错误（FP32/FP64 的坑）

### A1. 坐标整体 float 化 → 折叠后小数丢失（误差 2.2e-1，翻方块）
- **现象**：maintainPrecision 折叠后坐标（~2^24，如 16777216.5）整体 float 化，噪声误差 2.2e-1。
- **根因**：折叠后坐标 ~2^24 时 float 的 ulp=2，`16777216.5 → float` 直接丢掉小数部分 0.5。Perlin 采样的「小数部分」是 grad/fade 的核心输入，丢了小数 = 噪声完全错乱。
- **定位**：coord_precision_probe.py 对比「整体 float 化」vs「int32 整数 + float 小数拆分」。
- **修复**：坐标拆成 int32 整数（精确 hash）+ float 小数（~1e-7 精度），误差降到 1.6e-7。
- **教训**：**大数的「整体精度」和「小数精度」不能混为一谈**——float 的精度是相对精度（2^-23 × 数值），大整数会把小数精度挤掉。

### A2. InterpolatedNoiseSampler 的 `/o` 放大 → FP32 误差 1.03e-2（翻方块）
- **现象**：base_3d_noise（InterpolatedNoiseSampler）拆分 float 采样，误差 1.03e-2。
- **根因**：`for r=0..15: l += sample(coord×o)/o; o/=2` 里高 octave（o=2^-r）噪声被 `1/o=2^r` 放大。高 octave 要求 **~35 位坐标精度**（2^-35），float 只有 24 位（2^-23）。坐标拆分救不了——拆分把小数精度固定在 float 的 2^-23，超不出 float 硬上限。
- **定位**：diag_interpolated.py 逐 octave 诊断（r=15 单 octave 贡献误差 1.9e-4，40 octave 累积 1e-2）。
- **修复**：base_3d_noise 改用 GPU fp64（误差 3.2e-16 逐位一致）。
- **教训**：**精度需求要看「结构放大系数」**——vanilla 的 `/o` 结构把高 octave 精度需求放大 2^15 倍，这是 FP32 方案死掉的根因，不挖透会一直以为「FP32 够」。

### A3. lacunarity 公式反了（2^9=512 vs 2^-9=1/512）→ 坐标放大 512 倍，噪声符号都错
- **现象**：continents 端到端 maxDiff=0.92，gpu=0.83 vs cpu=-0.085，**符号都反了**。
- **根因**：noise.h 的 `lacunarity = 2^(-j), j=-firstOctave` = `2^(firstOctave)`；我写成 `2^(-firstOctave)`。firstOctave=-9 时正确值是 2^-9=1/512，我算成 2^9=512，**坐标被放大 512 倍** → maintainPrecision 折叠 → Perlin hash 全错。
- **定位**：纯 CPU 诊断（diag_continents_cpu.cpp）对比 noise.h sample(-0.0768) vs 手动复刻(-0.557)，锁定采样逻辑；再逐项核对 lacunarity 公式。
- **修复**：`2.0 ** firstOctave`。
- **教训**：**符号级错误（噪声符号都反）一定是「结构错」不是「精度错」**——先查公式/索引/坐标，别在精度上纠结。

### A4. maintainPrecision `floor` → `trunc`（负数差 2^25）
- **现象**：shader 用 `floor(v/2^25+0.5)`，noise.h 用 `(long)(v/2^25+0.5)`（向零截断），负数时差 1 → 折叠值差 2^25。
- **根因**：C++ `(long)` 是向零截断（trunc），GLSL `floor` 是向下取整；对负数 v（v<-2^24 时 t<0）两者差 1。
- **定位**：核对 noise.h 的 maintainPrecision 注释「(long)(...) 向零截断」。
- **修复**：shader 用 `trunc(...)`。
- **教训**：**「向下取整」和「向零截断」在负数上不同**——移植任何 floor/取整语义都要核对负数分支。

---

## B. 语义类错误（对象/结构搞混）

### B1. base_3d_noise = InterpolatedNoiseSampler（不是 DoublePerlinNoiseSampler）
- **现象**：octave.comp 实现的是 DoublePerlinNoiseSampler（first+second×1.018 = NormalNoise），验证的 1.4e-7 结论对 base_3d_noise 不适用。
- **根因**：没查证 vanilla 源码，凭「base_3d_noise 是 3D 噪声」想当然。
- **定位**：查 `DensityFunctionTypes.java` L46：`register(registry, "old_blended_noise", InterpolatedNoiseSampler.CODEC)`。
- **修复**：重写为 InterpolatedNoiseSampler（16+16+8 octave + smear + 插值）。
- **教训**：**「之前验证过的结论」只对「验证过的对象」成立**——DoublePerlinNoiseSampler 无 /o 放大所以 FP32 可行，InterpolatedNoiseSampler 有 /o 放大所以 FP32 死掉，两者精度特性完全不同。

### B2. flat_cache 是 biome 对齐（不是简单剥掉）
- **现象**：flat_cache 剥掉（= delegate 原始坐标）会差 0.01-0.1。
- **根因**：vanilla `FlatCache.sample` = `delegate((pos.x>>2)<<2, 0, (pos.z>>2)<<2)`（biome 坐标对齐），不是 `delegate(pos)`。而 cache_2d/cache_once/cache_all_in_cell 才是「= delegate 原始坐标」可剥掉。
- **定位**：读 vanilla `ChunkNoiseSampler.java` L836-865 的 FlatCache + L557-595 的 Cache2D。
- **修复**：flat_cache 用坐标变量参数化（gen_with_coords）+ biome 对齐；cache_2d/cache_once/cache_all_in_cell 剥掉。
- **教训**：**「缓存包装」不都是无副作用的**——flat_cache 改变了采样语义（biome 对齐），只有「纯缓存」类（Cache2D/CacheOnce）才能安全剥掉。

---

## C. 代码生成类错误（DFC 生成器）

### C1. spline 调用坐标硬编码 `(ix,iy,iz)`（flat_cache 对齐丢失）
- **现象**：factor 端到端 maxDiff=0.43，gpu=4.69 vs cpu=5.12。
- **根因**：`_gen_spline` 返回 `f"{fname}(ix, iy, iz)"` 硬编码原始坐标，而 factor 的 flat_cache 对齐后应该传 `((ix>>2)<<2), 0, ((iz>>2)<<2)`。spline 在 flat_cache 内，它的 coordinate（continents）应该用「对齐后的坐标」。
- **定位**：hand_calc_factor2.py 手算 factor=5.1188 与 CPU 一致 → 锁定 GPU 生成；读生成的 factor.comp 的 eval_density 发现 `spline_18(ix,iy,iz)` 没对齐。
- **修复**：`_gen_spline` 和 `_gen_registry_call` 都用 `self.cx/self.cy/self.cz`（当前坐标上下文）。
- **教训**：**「坐标变量」必须跟着上下文走**——flat_cache 改变了坐标，内层 spline 的 coordinate 也要用改后的坐标；硬编码全局坐标变量是这类 bug 的根源。

### C2. normal 噪声去重失效（offset 被 shift_x/shift_z 重复注册）
- **现象**：continents 的 noise_instances 里 offset 注册了两次（[1] 和 [2]）。
- **根因**：`_register_noise` 的去重 key 用自增 `f"n{len(...)}"`（每次都不同），没用 noise key。
- **修复**：去重 key 改用 noise key（`df.get("noise")` / `"minecraft:offset"`）。
- **教训**：**去重 key 必须是「业务唯一标识」**，自增 id 永远不重复 = 永远不去重。

### C3. registry 引用内联展开 → 表达式爆炸（168KB）
- **现象**：factor 表达式 168KB（continents 被多处引用，每处内联展开）。
- **根因**：registry 引用（`minecraft:overworld/continents`）在 spline 的 coordinate 里被多次引用，每次 `self.gen` 都内联展开完整 DF 树。
- **修复**：registry 引用函数化（`df_overworld_continents(ix,iy,iz)` 命名函数），引用处只调用 → 70KB。
- **教训**：**递归引用必须去重/函数化**，否则表达式随引用次数指数膨胀。

### C4. GLSL 函数顺序（normal_noise 定义在 registry 函数之后）
- **现象**：编译报 `normal_noise_1 no matching overloaded function`。
- **根因**：GLSL 要求「先声明后使用」，registry 函数调用 normal_noise，但 normal_noise 定义在后面。
- **修复**：噪声函数先于 registry 函数输出。
- **教训**：代码生成器要**显式管理函数依赖顺序**（噪声函数 → registry 函数 → spline 函数）。

### C5. registry 缓存命中分支漏改 `(x,y,z)` → `(ix,iy,iz)`
- **现象**：factor 编译报 `ix/iy/iz undeclared`。
- **根因**：`_gen_registry_call` 的「第一次注册」返回 `(ix,iy,iz)`，「缓存命中」分支还是旧的 `(x,y,z)`，两处不一致。
- **定位**：rg 检查发现两处返回值不一致。
- **教训**：**改了主路径别忘了缓存分支**——去重/缓存的「命中」和「未命中」两条路径要一起改。

### C6. registry 函数调用参数不一致（首次 5 参数 / 后续 3 参数）
- **现象**：factor.comp 第 219/237/250/263 行报 `df_overworld_ridges no matching overloaded function`。
- **根因**：`_gen_registry_call` 的「首次注册」返回 `fname(sIdx, ix, iy, iz)`（5 参数），「缓存命中」分支返回 `fname(ix, iy, iz)`（3 参数），而函数签名是 `fname(int sIdx, int ix, int iy, int iz)`（5 参数，因为 normal_noise 需要 sIdx 读拆分坐标）。spline coordinate 多次引用 ridges 时命中缓存分支 → 3 参数调用 5 参数函数。
- **定位**：rg 看 df_overworld_ridges 的定义（5 参数）vs 调用点（3 参数）→ 锁定缓存分支漏传 sIdx。
- **教训**：**「带上下文参数的函数化」——首次/缓存两条路径的参数列表必须一致**；函数签名加了一个参数（sIdx），所有调用点（含缓存分支）都要同步。

### C7. shift 表达式运算符优先级（`<<` 与 `*`）
- **现象**：生成的 cpu_backend.h 报 `C2297 '<<': 无效，右操作数 double`。
- **根因**：坐标 `ax = "(x >> 2) << 2"` 直接拼 `f"{ax} * 0.25"` → `(x >> 2) << 2 * 0.25`，C++ 里 `<<` 优先级低于 `*`，被解析为 `(x >> 2) << (2 * 0.25)`（右操作数 double）。
- **定位**：读生成的 cpu_backend.h 第 48 行看到 `<< 2 * 0.25`。
- **修复**：坐标表达式加括号 `({ax}) * 0.25`、`({ax}) * {xs}`。
- **教训**：**代码生成拼表达式，位移/位运算片段必须加括号**——`<<` 优先级低于 `*`/`+`，裸拼必炸。

### C8. DensityBuilder 在 namespace wg（C++ 引用无前缀）
- **现象**：dfc_factor_backend_e2e.cpp 报 `C2653 DensityBuilder 不是类或命名空间名称`。
- **根因**：density_builder.h 里 `class DensityBuilder` 在 `namespace wg { }` 内，引用时没加 `wg::` 前缀（noise.h/xoroshiro.h 同样在 wg）。
- **修复**：`wg::DensityBuilder` / `wg::DF` / `wg::NoisePos` / `wg::JsonParser` / `wg::JsonValue`。
- **教训**：**复用 worldgen 头文件先确认命名空间**——wg 命名空间是工程约定，外部文件引用必须带前缀。

### C9. DoublePerlinNoiseSampler 不可拷贝（unique_ptr 成员）
- **现象**：dfc_interp_e2e.cpp 报 `C2280 unique_ptr<PerlinNoiseSampler> 拷贝构造已删除`。
- **根因**：DoublePerlinNoiseSampler 内部 octaveSamplers 是 `vector<unique_ptr<PerlinNoiseSampler>>`，不可拷贝；用栈对象 + `make_shared<DoublePerlinNoiseSampler>(noodle)` 触发拷贝构造。
- **修复**：直接用 `auto noodle = std::make_shared<DoublePerlinNoiseSampler>(rd.split(...), {...})`（移动/直接构造进 shared_ptr），后续 `noodle->` 访问。
- **教训**：**含 unique_ptr 的对象不能拷贝**——用 shared_ptr 直接构造，别走「栈对象 → 拷贝进 shared_ptr」的路径。

### C10. old_blended 去重 key 自增 → gen 重复调用累积实例
- **现象**：base_3d_noise 的 splitTotal=560（应为 280）、normals=3（应为 1）——gen 被调多次后 noise_instances 累积。
- **根因**：old_blended 的去重 key 用 `f"ob{len(self.noise_instances)}"`（自增，永不重复），gen_shader 内部再次 gen(df) 时不去重。
- **修复**：去重 key 改用参数组合 `old_blended:xz_scale:y_scale:xz_factor:y_factor:smear`。
- **教训**：**去重 key 必须是「业务唯一标识」**（C2 同款坑的 old_blended 变体）——任何用 len()/自增 id 做 key 的地方都会永不重复。

### C11. 坐标变量硬编码（minecraft:y / minY）在 interp 函数内未定义
- **现象**：noodle shader 编译报 `y undeclared`、`minY undeclared`。
- **根因**：`minecraft:y` 分支硬编码返回 "y"，但 interpolated 包装函数（interp_N）里 y 未定义（只有 ix/iy/iz + fx/fy/fz）；interp 角点坐标用 minY 但 shader 模板没定义。
- **修复**：`minecraft:y` → `self.fy`（坐标变量跟着上下文走）；shader 模板加 `const int minY = -64`。
- **教训**：**「坐标变量」必须跟着上下文走**（C1 同款坑）——硬编码全局坐标变量 y/minY 在切换坐标上下文的函数（interp 角点）里必然未定义。

### C12. 端到端 maxDiff=0 假象（range_choice 常数分支吸收误差）
- **现象**：noodle 端到端 maxDiff=0.000e+00，gpu=cpu=64.000000000。
- **根因**：采样点恰在 range_choice 的 when_in_range 常数分支（interpolated 采样值 < 0 → 返回 64.0），误差被阈值判定吸收，不是真零误差。
- **定位**：诊断打印 gpu=cpu=64.0 → 发现是常数分支；改采样坐标让 interpolated 采样值跨 0 阈值后误差才体现（maxDiff=5.053e-07）。
- **教训**：**端到端验证必须让采样点覆盖阈值两侧**——range_choice/abs/max 等非线性节点的常数分支会掩盖底层误差；maxDiff=0 要先怀疑「是不是采样点没覆盖有效路径」，而不是「完全对齐」。

### C13. gen_shader/gen_cpu 顺序污染映射 → normals[131] 越界（0xC0000005）
- **现象**：final_density 端到端崩溃 0xC0000005，normals[131] 越界（normals 只有 0..130）。
- **根因**：gen_final_density.py 里 gen_shader 在 gen_cpu 前调用，gen_shader 的分配循环已经填充 normal_vec_index（0..130），gen_cpu 的收集循环再 `normal_vec_index[key] = len(self.normal_vec_index)` 从 131 开始 → vi=131 越界。
- **定位**：诊断打印 split + collectPerm 后崩溃 → 定位 split 方法 normals[131]；rg 看 normals.emplace_back=131 vs normals[131] 最大索引=131 差 1。
- **修复**：gen_cpu 在 gen_shader 前调用（先填映射再生成）。
- **教训**：**gen_cpu 和 gen_shader 都填充 noise 映射（normal_vec_index/split_base），两者顺序必须固定且 gen_cpu 先于 gen_shader**——否则第二次填充从 len() 继续，索引越界。

---

## D. 编译类错误（GPU 驱动）

### D1. 驱动内联展开 34 个函数 → SPIR-V 17 倍膨胀（编译 >10min）
- **现象**：factor shader（135KB SPIR-V）vkCreateComputePipelines 编译 >10min。
- **根因**：**不是 fp64**（fp64 只占 2% 指令，184/8633），而是驱动内联展开 34 个嵌套函数（spline + NormalNoise）。`spirv-opt --inline-entry-points-exhaustive` 0.9s 复现 SPIR-V 135KB→2.34MB（17 倍），LLVM 寄存器分配在巨型基本块上超线性爆炸。
- **定位**：spirv-dis 量化 Opcode（OpLoad 1716/OpVariable 1383/OpFunctionCall 291）→ 发现大量未标量化变量 + 嵌套调用；`--inline-entry-points-exhaustive` 复现膨胀。
- **修复**：spline 函数化 + 去重（98→19 函数）缓解源码体积；真正的编译慢要靠「GPU 纯 float（无 fp64）」根治。
- **教训**：**编译慢先量化，别猜**——spirv-dis 一查就知道是「内联」还是「fp64」，报告里「fp64 展开十几条」的机制描述就是没量化想当然的（fp64 实际只占 2%）。

### D2. DontInline 的坑：位置对，但引入 fp64 行为错误
- **现象**：给非 entry 函数设 DontInline 后编译从 >10min 降到 1.6s，但 erosion 结果从正确（3.77e-7）变错（1.48）。
- **根因**：DontInline 的**正确位置是 `FunctionControl` 位（bit 1 = 0x2），不是 `OpDecorate` decoration**（spirv-as 报 `Invalid decoration 'DontInline'`）。但即使位置对了，阻止内联后 fp64 的 `maintainPrecision` 等函数不被内联，行为异常（fp64 运算在 DontInline 下的驱动 bug）。
- **定位**：对比「原始 spv（无 DontInline）erosion 正确 3.77e-7」vs「DontInline 版错误 1.48」，锁定 DontInline 引入 bug。
- **修复**：方向改为「CPU 预拆分」（fp64 坐标拆分移到 CPU，GPU 纯 float），彻底没有 fp64 就没有这个坑。
- **教训**：**DontInline 是 FunctionControl 位不是 decoration**（查 spirv.hpp `FunctionControlDontInlineShift=1`）；且 **DontInline 对 fp64 有副作用**，不是「免费午餐」。

### D3. final_density shader 驱动编译 >2min（210 函数 76338 行，DontInline 无效）
- **现象**：final_density.spv（1.2MB，76338 行）vkCreateComputePipelines 编译 >2min。
- **根因**：final_density 是 factor 的 7 倍规模——OpFunctionCall 2073 次、OpVariable 11296、OpLoad 14119（factor 仅 291/1383/1716）。210 个函数（56 spline + 139 normal + 6 interp）嵌套调用，驱动内联展开 → LLVM 寄存器分配在巨型图上超线性爆炸。
- **尝试**：① FunctionControl DontInline（spirv-as 正确生成 210 个）仍 >2min——NVIDIA 驱动忽略或 call 消除后仍爆炸。② 纯 float（normal/old_blended 的 double 累加 → float，OpFConvert 从数百降到 3、double OpFAdd 清零）仍 >2min——**fp64 不是主因**。
- **待解决**：根因是「函数嵌套」（210 函数 × 2073 调用，spline 56 + normal 139 + interp 6），需**拆 shader**（final_density 拆成多个子 shader）或**深度扁平化**（spline 内联到调用点）。
- **教训**：**shader 规模（函数数 × 调用数）是驱动编译时间的主因**，不是单看 fp64 也不是 DontInline；210 函数 76338 行已经超出「单 shader 可编译」的合理规模——纯 float 只把 fp64 清零（1.2MB→1.2MB 几乎不变），驱动还是要编译 210 个函数的嵌套调用图。

### D4. GLSL 不支持递归（spline 数据驱动 spline_eval ↔ spline_val_at 相互递归被拒）
- **现象**：spline 数据驱动（spline_eval 递归查表替代 56 个 spline 函数）glslc 报 `Linking compute stage: Recursion detected: spline_eval(...) calling spline_val_at(...)`。
- **根因**：GLSL/SPIR-V 链接阶段**禁止递归**（含相互递归），spline 的嵌套 value（child 递归查表）天然是递归结构，不能用递归函数表达。
- **尝试**：前向声明（prototype）后仍报 Recursion detected。
- **待解决**：① 显式栈循环（栈数组 + while 模拟后序递归，1 个函数）；② Python 侧递归内联（嵌套 spline 的 Hermite 内联到 parent，非 GLSL 递归，但代码膨胀 56×15 if-else）。
- **教训**：**GLSL 无递归**——数据驱动树/图结构（spline 嵌套）必须用「显式栈」或「编译期展开」，不能照搬 CPU 的递归算法。

### D5. 节点函数化（每 DF 节点一函数）→ 函数体小了但函数数爆炸（300 个），编译仍 >10min（2026-08-14 D1）
- **现象**：把 interp 8 角点内联展开（68KB）改成「节点函数化」（每个 DF 节点注册 df_N 函数 + 子节点调用），interp 函数体从 69868 → 950 chars，最大 df_N body ≤ 307 chars，但 `vkCreateComputePipelines` 仍 >10min。
- **根因**：**驱动编译时间 = f(函数数, 嵌套调用深度) 双维度**——函数体大小优化了（150-300 chars），但函数数爆炸到 ~300（interp 角点 × 每节点一函数），300 个函数的嵌套调用图照样超线性爆炸。单方面优化函数体或函数数都不够，**两者都要小**（对照：noodle 44 函数 × 1.6KB → 2.4s ✅）。
- **定位**：`check_func_sizes.py` 量化函数族大小（interp 950 / df_N ≤307 全部小），pipe_bench 计时仍 >10min → 函数数是瓶颈。
- **修复**：未解决。方向 = **节点类型分派 + 数据 buffer**（C2ME 的 const_data 机制）：每算术类型一个函数（add/mul/min/... 各 1 个），节点数据（操作数引用）放 buffer，函数数 = 类型数（~15）而非实例数（300）。
- **教训**：**「每实例一函数」和「每节点一函数」都是展开式的变体**——正确形态是「每类型一函数 + 数据」（镜像 Java 解释器：一个类服务所有实例，数据在 JSON/buffer）。函数数目标 ≤ ~50（noodle 44 已验证秒级），函数体目标 ≤ ~1.6KB。

### D6. GLSL 函数依赖序：registry 函数引用 df_N，但 df_N 前向声明必须在 registry 之前（2026-08-14 D1）
- **现象**：glslc 报 `df_195 : no matching overloaded function found`——引用存在但定义在后。
- **根因**：node_funcs 注册顺序 = 父先子后（`_gen_node_body` 递归注册子节点），而输出顺序若按注册序，父函数 body 引用的子函数定义在后面 → GLSL 先声明后使用违反。且 registry 函数（df_overworld_xxx）也引用 df_N。
- **修复**：**所有 df_N 前向声明放在 gen_shader 最前**（interp 声明后、registry 之前）——前向声明是声明，不要求定义序，一次声明全部解决。
- **教训**：**GLSL 生成器输出顺序必须显式管理依赖**（C4 同款坑的 D1 变体）——「节点函数化」引入大量跨函数引用后，最稳的做法是**全部前向声明**，不依赖注册顺序。

### D7. 节点函数化坐标上下文：y_clamped_gradient / minecraft:y 必须用形参 iy（2026-08-14 D1）
- **现象**：glslc 报 `cy : undeclared identifier`——df_N 函数体里 `y_clamped_gradient((minY + (cy + 0) * 8), ...)`。
- **根因**：gen_node 的 `_gen_leaf_expr` 里 `minecraft:y` 和 `y_clamped_gradient` 返回 `self.fy`/`self.cy`（旧坐标变量），而 interp 角点 `gen_with_coords_call` 把 self.cy 设为角点 y 表达式 `(minY + (cy + 0) * 8)` → 内联进 df_N 函数体 → cy 未定义（df_N 形参是 ix/iy/iz）。
- **修复**：节点函数化后**所有坐标引用用形参 ix/iy/iz**（调用点传实际坐标：interp 角点调 `df_N(sIdx, ax, ay, az)`，flat_cache 调 `df_N(sIdx, (ix>>2)<<2, 0, (iz>>2)<<2)`）。
- **教训**：**「坐标变量跟着上下文走」的机制（self.cx/cy/cz + gen_with_coords）与节点函数化冲突**——函数化后坐标是形参，上下文（flat_cache 对齐/interp 角点）由调用点显式传参，不能再靠生成时切换变量。改节点函数化时**所有坐标引用必须同步改形参**（minecraft:y / y_clamped_gradient / spline_eval 调用 / registry 调用 5 处）。

### D8. gen_node 递归注册 idx 冲突：父节点 body 生成前子节点已占 idx（2026-08-14 D1）
- **现象**：shader 里 `df_0 function already has a body`——同 idx 多个不同 body。
- **根因**：`idx = len(self.node_funcs)` 在 `append` 前取，父节点 idx = N，但 `_gen_node_body` 递归子节点时子节点也取 `len(self.node_funcs)` = N（父未 append）→ 子节点与父节点 idx 冲突；父 append 后后续节点从 N+1 开始。
- **修复**：**先 append 占位（idx, None）再生成 body，最后回填**（`self.node_funcs[idx] = (idx, body)`）。
- **教训**：**递归注册节点函数必须「先占位后回填」**——idx 分配与 body 生成解耦，防止递归子节点抢占父节点索引。flat_cache 分支同样要占位（它也在 _gen_leaf_expr 内手动注册）。

### D9. interp 角点 delegate 去重与 gen_cpu 拆分冲突：jagged@c1 缺失（2026-08-14 D1 遗留）
- **现象**：gen_cpu 报 `KeyError: 'minecraft:jagged@c1'`——interp 角点 c=1 的 noise 未注册。
- **根因**：gen()/gen_node 的 interp 角点 delegate 被「结构去重」——相同 delegate 树（不同角点坐标）只注册一次（@c0），c1 复用缓存不重新注册。但 gen_cpu 的 `_gen_split_lines` 期望每角点独立 noise 实例（拆分坐标按角点布局）。
- **状态**：**遗留未解**（D1 验证阶段临时跳过 gen_cpu）。方向：gen_cpu 的收集与 GPU 节点函数化的去重语义需对齐——要么 gen_cpu 也按「角点显式传坐标」模型，要么 GPU 保留每角点实例。
- **教训**：**GPU 节点函数化（坐标无关函数 + 调用点传坐标）与 CPU 拆分（每角点独立实例）是两种模型**，两者对 interpolated 的处理不一致会破坏 splitCoord 布局对齐——重构时必须同时改两侧。

### D10. D2 节点数组 879 节点仍编译慢——「数据展开」与「代码展开」同病（2026-08-14 D2，实测 60+min）
- **现象**：D2 把 DF 树序列化成节点数组（eval_df 解释器，函数数 300→38、函数体都 ≤3.8KB、shader 总行数 750），glslc 通过，但 `vkCreateComputePipelines` 实测 **>60min 仍未完成**（进程 CPU 3590s、内存 5.8GB，后降 2.4GB 疑似进入下一阶段）。
- **根因**：**df_nodes = 879**（jagged@c1 修复后 767→879）——interp 8 角点 delegate 树每角点独立序列化（noise 节点按 @c{c} suffix 独立，导致算术节点也跟着角点独立），5 interp × 8 角点 × ~93 节点 ≈ 879。这是「8 角点展开」从**代码展开变成数据展开**——`float val[879]` + 879 次求值循环 + 879×7 const 数组，驱动编译大数据量照样慢，**比 D1 的 300 函数更慢**（879 节点 > 300 函数 > 210 函数）。
- **定位**：dbg_node_analyze.py 显示 df_nodes=879、噪声实例 160（8 角点 × 20）、噪声节点 136（8×17）；interp 节点 5 个，每个的 8 角点 root 分别为 129/222/315/408/501/594/687/780（每角点 ~93 节点独立序列化）。bench 5min 未出 → 大数据量编译慢。
- **修复方向（未实施）**：**delegate 树只序列化一份（跨角点共享），8 角点 = 8 次 eval_df_base(共享节点, 角点坐标) 调用**——即 C2ME 的 interpolator 网格预填充。noise 的 splitBase 偏移改为「运行时查表」（节点数据），而非编译期常量。这样 df_nodes = 唯一树结构（~19 个）。
- **教训**：**「每实例数据」和「每实例代码」一样会爆炸**——数据驱动化只解决「函数数」，不解决「数据量」。真正根治 = **结构共享**（delegate 树一份 + 坐标/参数运行时传），即 Java 解释器的「一个类 + 实例数据」形态，也是 C2ME interp 预填充的本质。数据驱动化要「结构去重到唯一」，不能按采样点/角点复制结构。

### D11. interp 角点 eval_df 与顶层 eval_df 形成递归调用图（2026-08-14 D2）
- **现象**：glslc 报 `Recursion detected: eval_df calling interp_0`——interp 角点 delegate 用 eval_df，而 eval_df 的 DF_INTERP 分支调 interp_N。
- **根因**：GLSL 静态递归检测（D4）——即使运行时 delegate 不含嵌套 interp（已确认 5 个 interp delegate 均无嵌套），链接期看到 eval_df→interp→eval_df 符号环就报错。
- **修复**：eval_df 拆两个版本——`eval_df`（含 DF_INTERP，顶层）+ `eval_df_base`（不含 DF_INTERP，interp 角点 delegate 用，delegate 树保证无 interp）。
- **教训**：**GLSL 静态递归检测是符号级的**——两个函数互相调用（即使运行时不同路径）即报错。数据驱动解释器里，「含 X 节点」和「不含 X 节点」的求值函数要拆开，避免符号环。

### D12. interp 角点 delegate 共享 vs CPU 每角点拆分——splitBase 必须是运行时数据（2026-08-14 D2 遗留，D9 的完整版）
- **现象**：delegate 树跨角点共享（全局去重）时，noise 节点的 splitBase（splitCoord 偏移）无法确定——CPU 侧按「角点实例」分配 splitBase，GPU 侧共享树则 noise 节点缺「每角点的 splitBase」。
- **根因**：splitBase 是「编译期常量」（noise 实例在 gen 时分配），而共享 delegate 树要求「同一 noise 结构在不同角点用不同 splitBase」→ 必须运行时查表。
- **状态**：遗留。方向 = noise 节点增加「splitBase 运行时查表」（noise 实例数组 + 角点索引），CPU/GPU 两侧对齐。
- **教训**：**「结构共享」与「参数实例化」必须分离**——结构（树拓扑）全局共享一份，参数（noise 的 splitBase/octBase、坐标）按采样点运行时查表。这是 Java 解释器的核心（一个类 + 实例字段），也是 D9 的最终解。

### D13. D1 把 `gen_node` 泄漏进 gen() 的 range_choice → gen_cpu 收集破坏 jagged@c1（2026-08-14，已修复）
- **现象**：gen_cpu 报 `KeyError: 'minecraft:jagged@c1'`。dbg 对比 HEAD vs 当前版：HEAD 的 `normal_chain_index` 有 jagged@c0..c7（8 角点），当前版只有 jagged@c0。
- **根因（真正的，非一开始猜的 spline key）**：D1 节点函数化时，把 gen()（旧路径，gen_cpu 依赖）的 `range_choice` 分支从 `self.gen(...)` 改成了 `self.gen_node(...)`。`gen_node` 设 `node_mode=True` → `_gen_registry_call` 走 `_gen_registry_call_node`（**不展开 registry ref**，只生成一个 `df_sloped_cheese` 函数）。而 `minecraft:jagged` 噪声在 `sloped_cheese.json`（registry ref）里、且被 `range_choice` 的 input 引用。HEAD 版 range_choice 用 `self.gen`，在 `interp_depth>0` 时**每角点展开** sloped_cheese → jagged 每角点注册；当前版 gen_node 只在 c0 首次生成 df_sloped_cheese 时注册一次 jagged@c0，c1..c7 走缓存不再注册 → gen_cpu 的 `_gen_split_lines` 查 `normal_chain_index['minecraft:jagged@c1']` KeyError。
- **定位**：`dbg_jagged_cmp.py` 对比 HEAD/当前版的 `normal_chain_index`（jagged keys 8 vs 1）；`dbg_jagged_tree.py` 追到 jagged 在 `sloped_cheese.json`（非 spline）；grep `self.gen_node` 发现 gen() 里 range_choice 是唯一无条件泄漏点（spline_coord_type 里那处是 `if node_mode` 条件式，不泄漏）。
- **修复**：gen() 的 range_choice 改回 `self.gen(...)`（一行）。修复后 jagged@c0..c7 恢复，gen() 与 gen_df() 噪声实例序列完全一致（160=160）。
- **教训**：**两条生成路径（gen() 旧路径 / gen_df D2 路径）必须严格隔离**——D1/D2 的 `gen_node`/`gen_df` 只能在自己的路径里用，绝不能「顺手」替换 gen() 内部对 `self.gen` 的递归调用。gen() 是 gen_cpu 的回归基线（每角点展开语义），任何对 gen() 节点分支的改动都要先问「这会不会改变噪声实例注册集合」。**排查 KeyError 类错误时，先 grep 有没有把新路径的函数调用泄漏进旧路径**，而不是盯着 key 后缀猜。

### D14. spline coordinate 噪声不生成 split 行（HEAD 既有 bug）——spline 坐标噪声 GPU 恒 0（2026-08-14 方案1 排查发现）
- **现象**：gen_cpu 的 `split()` 里**没有 continentalness/erosion/ridge 的 splitDouble 行**（normals[0/1/2]，splitBase 0/108/168）——spline_coord 的 case 引用 `normal_noise(0/1/2, sIdx)` 读 splitCoord[0..] = 未初始化 0。
- **根因**：`_gen_split_lines` 递归遍历 DF 树时，**spline 节点（type=minecraft:spline / nested {coordinate,points}）不被遍历**——else 分支只遍历 `argument/argument1/argument2/input/when_in_range/when_out_of_range`，不含 `spline`/`coordinate`/`points` 字段。而 spline 的 coordinate 正是 continentalness/erosion/ridge（flat_cache(shifted_noise)）的**唯一引用点** → 它们的 split 值从未生成。HEAD 同样有此 bug（之前 e2e 只覆盖 noodle/continents/erosion 等**不含 spline coordinate 噪声**的子树，final_density 完整树从未验证过）。
- **定位**：grep cpu_backend.h 的 split() 内 `splitDouble(normals[0|1|2]` 无匹配；spline_coord case 语句确认引用固定实例索引；factor.json 顶层 = flat_cache(cache_2d(add(...spline...)))。
- **修复方向（方案1 一并做）**：① `_gen_split_lines` 遍历 spline 的 coordinate（+ nested spline 递归）→ 生成坐标噪声的 split 行；② spline coordinate expr 从「固定实例引用」改成 **slot 化**（`NOISE_SLOT_BASE[slot]+corner*NOISE_SLOT_STRIDE[slot]`），spline_coord/spline_eval 加 corner 参数；③ 由此 continentalness 从「HEAD 的 @c0 1 份」恢复为「每角点 @cN 8 份」（**HEAD 的 spline 结构去重把 coordinate 噪声错误共享成 @c0 是又一个潜在 bug**——flat_cache 内噪声 8 角点不同 4×4 列，density.h FlatCacheDF.k 判定，必须角点独立）。
- **教训**：**「遍历树生成 split」与「节点化生成 shader」必须覆盖同一棵树的同一批噪声**——spline 这种「被 SSBO 数据驱动吸收、不显式出现在树遍历路径」的节点最容易漏。改生成器时，用「cpu_backend.h 里每个实例的 splitBase 都有 split 行」做完整性断言（139 实例 → 139 组 split 行，缺一组就是漏遍历）。

### D15. eval_df 的 `val[N]` 数组 local memory 溢出 → TDR（VK_ERROR_DEVICE_LOST）（2026-08-14 方案1 已修复）
- **现象**：方案1 版 e2e（158 节点解释器）pipeline 编译 1185s 成功，但 dispatch 后 `VK_ERROR_DEVICE_LOST`；Windows 事件日志 nvlddmkm 153（GPU 超时恢复 TDR）。N=64 同样复现 → 与数据量无关。
- **根因**：eval_df_base/eval_df 的 `float val[158]` 数组被 glslang 复制成 **10 个 indexable 副本/函数**（循环内动态索引 `val[DF_A1[i]]` 的多个访问路径）→ 每 work item local memory ≈ 10×158×4B×2 函数 = **12.6KB** → 256 work items/组 = 3.2MB，远超寄存器 → spill 到全局内存 → kernel 极慢（每访问一次 L2/DRAM）→ TDR。
- **定位**：spirv-dis 反汇编 → `OpVariable %_ptr_Function__arr_float_uint_158` 出现 20 次（eval_df_base 10 + eval_df 10）；dbg_val158.py 统计归属；对比 noodle（函数式无 val 数组，2.4s 编译 + 无 TDR）确认解释器 val 数组是罪魁。
- **修复（方案1b）**：**val 槽位复用（活跃分析）**——后序求值中节点值「最后被引用后即弃」：`max_parent[i]` = 引用 i 的最大父节点索引，i 在 `[i, max_parent[i]]` 活跃；贪心分配槽位 → **val[158] → val[19]**（峰值同时活跃 19）。生成 `SLOT_OF[i]` 表，eval 时 `val[SLOT_OF[i]]`。结果：SPIR-V 里 val[19] 仅 2 个副本（12.6KB → 152B/work item），SPIR-V 219KB→144KB。
- **修复续（方案1c，TDR 仍在）**：val[19] 后 kernel（1 work item 空数据）仍 DEVICE_LOST（1.957s = TDR 阈值）——spirv-dis 证实局部数组副本只剩 2 个，但 **kernel 执行仍卡**。方向：val 栈改 **SSBO**（每采样点 9 区段 = 8 角点 + 1 顶层，`valBuf[(sIdx*9+corner)*VAL_SLOTS + SLOT_OF[...]]`），消除局部数组 → 无寄存器溢出/spill。**注意**：GLSL buffer 块成员名与实例名不能相同（`{float valBuf[];} valBuf;` → `valBuf[x]` 歧义报「not of type array」）——用无实例名块声明。
- **教训**：**GPU 局部大数组（解释器栈）是 TDR 隐形杀手**——glslang 为「循环内动态索引数组」生成多副本，local memory 指数膨胀。数据驱动解释器的 val 栈必须做**活跃分析/槽位复用**（像寄存器分配一样），而不是简单 `val[N]`。这也解释了为什么 879 节点版（val[879]×10≈35KB）即使编译完也会 TDR。
- **附带发现**：glslc -O（SPIR-V 优化）也超时 2min+——大数组 + 循环 + 动态索引的组合让 glslang 优化器也爆炸，佐证 shader 结构需「小而多函数」而非「大而单循环」。

### D16. `normal_noise`/`interp_noise` 参数表索引与实例索引错位 → GPU 越界读 → TDR（2026-08-14 方案1 二分定位，根因确认）
- **现象**：完整版（SSBO val + DontInline）kernel 1 work item 空数据仍 DEVICE_LOST（1.96s = TDR）；pipeline 编译 776s→35s（DontInline 生效）但 kernel 依旧卡。
- **根因（最终确认，二分定位）**：**`normal_noise` 的实例索引（NOISE_SLOT_BASE[slot] + corner）是 noise_instances 索引（含 old_blended），而 NORMAL_PACK 参数表按 normal 序号生成 → 末尾实例（noodle_ridge_b@c0..c7，slot base=152）索引越界（NORMAL_PACK[456..] 超 152×3）→ GPU 越界读 const 数组 → TDR**。中间曾被误判为「eval_df_base 多调用者→驱动内联」（DontInline 编译快但 kernel 仍卡，故内联不是执行卡的原因；minimal 系列又因生成脚本 replace 失败误判执行链）。
- **定位过程（minimal 系列二分）**：minimal3（main 直接调 eval_df_base 单次 + 节点 + 噪声 + spline）→ 0.001s ✅；test_base0（方案1f 的 eval_df_base_0 单次）→ 0.019s ✅；interp_0/1/2/3 单独调 → OK；**interp_4 单独调 → TDR**；interp_4 去 spline → 仍 TDR；**interp_4 去噪声（noodle_ridge_b）→ 0.007s ✅** → 锁定噪声；查 slot 19 base=152 + corner → 实例 152..159，而 NORMAL_PACK 仅 152 项 → 越界。
- **修复（方案1f + 对齐）**：① 全部闭包化（顶层闭包 21 + 每 interp 独立解释器副本 eval_df_base_N 循环各自闭包）；② **NORMAL_PACK/NORMAL_PACK_F/NORMAL_AMP_OFF/NORMAL_AMPS 按 noise_instances 全量索引生成（old_blended 位置占位 0）**。结果：kernel 0.043s（TDR 消失），SPIR-V 编译 314s。
- **教训**：① **GPU 越界读 const 数组也会 TDR**（不是只读 garbage）——凡「索引 = 实例/槽位」必须与参数表索引严格同源（noise_instances 含 old_blended 的混合索引 vs normal 序号，两类索引在 slot 化后被混用）。② **排查 TDR 用「功能二分」**（逐函数/逐分支替换为常量，测 kernel 时间）比猜内联/内存更快。③ minimal 系列生成脚本的 replace 脆弱（锚点不匹配静默失败 → 测试的是旧产物）——**二分测试必须基于生成器直接输出**（改生成器 + 重生成），不手改字符串。
- **遗留**：pipeline 编译 314s（4.6min）仍远高于目标（<2min）——编译时间优化待续（G 系列：数据表/解释器结构/拆 shader）。

### D17. e2e y>-64 语义差 maxDiff 0.099 → **两个独立根因**（2026-08-14，已修复，模拟侧 maxDiff 3.5e-10）
- **现象**：e2e N=1024：y=-64 底部正确（diff 3.755e-9），y>-64 GPU 值线性降（0.0375→0.0018，斜率 -0.0023/层）而 CPU 参照缓降+跳变（-0.00049/层，y=-54 起跳变）→ maxDiff 0.099。
- **踩坑 1（误判 OLD_PACK 是 y 依赖来源）**：以为 old_blended（base_3d_noise）索引错位（同 D16）会解释 y 差 → 修了 OLD_PACK 对齐 → **e2e 输出一字没变**（白费 1 次 5min e2e）。根因：y 线性降是「平滑的」而 3D 噪声随 y 会波动 → 从波形形状就该判断不是 base_3d_noise。教训：**修完一个嫌疑后若输出完全没变，先怀疑「嫌疑本身不相关」而不是继续深挖同一条线**（输出不变 = 该路径未生效）。
- **踩坑 2（模拟 interp_noise 用旧 l/5 公式 → 数量级错误）**：Python 复刻 GLSL 的 interp_noise 时凭记忆写成 `(l/5+1)/2` 旧公式（GLSL 实际是 `(l/512 + clamp(qq)*(m/512-l/512))/128`）→ 模拟输出 1125（真实 ~0.46）→ 一度误以为 GPU/模拟的 base_3d_noise 爆炸。用 b3d_probe（参照 density.h + WG_B3DDUMP）对拍才揭穿是模拟 bug。教训：**GLSL→Python 移植必须逐行对拍公式常量**（l/5 vs l/512 是数量级差异），且模拟结果异常时先用参照实现（b3d_probe）验证模拟本身。
- **根因 1（spline SSBO 陈旧索引 → factor=6.3 应为 3.95）**：`_gen_spline` 的 **`node_idx = len(spline_ssbo_nodes)` 在循环前捕获**，但嵌套子样条在循环内递归 append（占据 node_idx 起的槽位），父样条最后 append（实际索引 = node_idx + 新增子数）→ **父样条记录陈旧索引 = 第一个新子样条的槽位**；且 `val_begin` 同样在循环前捕获（父 val 与子 val 交错 append）→ 父 val 读成子样条的 val。后果：factor 的顶层 continents 样条（SSBO[55]，端点 3.95）被 node[33] 引用成 ridges 子样条（SSBO[37]，6.3/6.25）→ factor 6.30 而非 3.95 → sloped_cheese 27.14 而非 12.69（参照 factor=3.95 是 vanilla 经典值）。**定位**：ref_probe（DensityBuilder 参照）直接采样 factor=3.95/sloped=12.69 与模拟 6.3/27.14 对拍 → 数值量级差异锁定 spline；dbg_ssbo_idx 打印 SSBO 全表发现 node[33].a1=37 而 continents 在 55。**修复**：node_idx/val_begin 移到循环后（val 缓冲后统一 append）。
- **根因 2（weird_scaled_sampler 被 stub 成 0.0f → entrances Y 分支错值）**：`gen_df`/`gen()`/`_gen_split_lines` 三处 ws 全 stub（gen 返回 0.0f，split 不遍历）。后果：entrances = min(X, Y) 中 Y = spagrough + clamp(MAX(ws(spag3d_1), ws(spag3d_2)) + add(-0.0765, -0.0115×thickness)) 的 **MAX 项恒 0** → Y=-0.0654 错误绑定（正确 Y≥0.57，min 取 X=0.5691）；when_out 从 A=0.0989（正确）变 -0.0656 → 系数链 -0.1172+when_out 全错。**定位**：ref_probe 采样 entrances=0.5691/spag2d=1.0/spagrough=0.0207，与模拟逐节点对拍发现 node[66]=Y=-0.0654 错（node[57]=spagrough ✓ 0.0207、node[104]=spag2d+spagrough ✓ 1.0207、node[83]=A ✓ 0.0989）→ 唯一差异在 Y 的 clamp(MAX(...)) 项 → node[58]=MAX(node[9],node[9]) 双引用同一节点 → 查 JSON 发现是 2 个 ws 节点 → gen 里 ws stub。**修复**：完整实现 ws——ws 噪声按 normal 实例注册（xz/y_scale=1.0，split 在 (x/d,y/d,z/d)，d=ws_scale(rarity, 输入值) 由 CPU split() 先算）；rarity 输入（cache_once(noise)）走普通噪声槽；DF_WEIRD=22 新节点类型（a1=输入节点, a2=ws slot, f0=kind）；GLSL/C++/Python 三侧加 ws_scale；注意 **closure visit / map_a 按字段区分（a2 是 slot id 非节点索引，防递归错位）**。
- **验证**：dbg_full_sim（Python 复刻 GPU 解释器）全 1024 点 vs 参照 maxDiff=3.6e-8；**e2e GPU vs CPU 参照 maxDiff=3.128e-07 avgDiff=1.097e-08**（原 0.0998）——sloped=12.69、entrances=0.5691、when_out=0.0989 全部逐位对齐。e2e 中途还踩 D19（PER_SAMPLE 硬编码 320 → valBuf 越界 → 尾部输出 0），修复后全绿。
- **教训（复用判错经验）**：① **y 相关语义差优先怀疑「分支切换」（range_choice/spline 区间跨越），跳变点（y=-54）是强线索**——不是噪声本身（噪声平滑，分支是阶跃）。② **「生成器里留的 stub/TODO」（如 ws→0.0f）是语义差的头号嫌疑**——先扫代码里的 `return 0.0`/`pass`/`暂简化` 再深挖。③ **参照侧用 registry 分量探针（DensityBuilder.getRegistryEntry）逐分量采样对拍**——比只比最终值快得多（factor/sloped/entrances/spag2d 一跑全现形）。④ **「n=2 样条 6.3/6.25」是 factor.json 的内层 ridges 样条，不是顶层**——样条收集的「先子后父」顺序 + 缓存去重让索引错位极具隐蔽性，验证样条数据必须对照 JSON 的 coordinate 层级。

### D19. e2e 硬编码 `PER_SAMPLE=320` → valBuf 越界 → 尾部 work item 输出 0（2026-08-14，已修复）
- **现象**：D17 双根因修复后，e2e DBG 点（x=0 列 + y=-64 行）全部 ~1e-9 匹配，但 maxDiff 仍 0.0998、avg 5.7e-3——TOP 差异点 gpu=**0.000000000**（不是错值，是输出恰为 0）。
- **根因**：e2e 里 `const uint32_t PER_SAMPLE = 320;` **硬编码旧值**——ws 实现后 per_sample 从 320 涨到 352（+5 节点 +5 槽）。valBuf 按 320×N 分配，shader 按 352×sIdx 索引 → **sIdx ≥ (320×1024-351)/352 ≈ 930 越界 → 这些 work item GPU 写失败 → 输出 0**。零点模式（sIdx 931-1023 全 0 + sIdx 899-930 部分）与越界阈值精确吻合。
- **定位**：out_dump.txt 全量 dump → 零点按 (x,y) 分组：y=-49 全 64 点 + y=-50 上半 29 点 = sIdx 931-1023 连续段 → 联想到「缓冲区大小跟不上生成器」→ 查 e2e 缓冲分配发现 PER_SAMPLE 硬编码。
- **修复**：`_compute_val_layout()` 抽取（eval_df_glsl + gen_cpu 共用闭包/活跃分析/per_sample）→ gen_cpu 在 CpuBackend 输出 `int perSample = 352` → e2e 改用 `backend.perSample`。**宿主与生成器之间的任何布局常量（PER_SAMPLE/splitTotal/permSize）必须由生成器产出，禁止硬编码**。
- **教训**：① **GPU「输出恰为 0」= 该 work item 写失败（越界/UB），不是数值错**——先查缓冲区大小/索引与生成器是否同源。② **修改生成器后必须全文搜索宿主侧硬编码的对应常量**（PER_SAMPLE 这类「恰好没报错直到越界」的坑最隐蔽）。

### D21. 管线编译 903s 根因 = spline 动态 node 索引（2026-08-14 Phase 1 实验判别，方案决策点）
- **现象**：D17/D19 修复正确性后，compile_bench 单独测 vkCreateComputePipelines = **903.4s**（glslc 前端仅 0.2s，瓶颈全在驱动 SPIR-V→机器码）。
- **实验判别**（DFC_DIAG 诊断开关 + compile_bench 批量测编译时间，最小 shader 验证）：
  | 变体 | 编译时间 | 结论 |
  |---|---|---|
  | 完整 | 903.4s | 基线 |
  | no_old（去 fp64 计算链） | 591.8s | fp64 次因 ~310s |
  | **no_spline（去整个 spline）** | **17.6s** | **spline 主因 ~885s** |
  | no_old + no_spline | 7.3s | |
  | spline 固定 node=0（深度展开 d0..d3） | **31.0s** | **动态 node 索引是关键** |
  | 每 node 展开 56 函数 + switch | >12min | 56 函数×复杂 body 也爆 |
  | switch 56 case（简单 body 实验 shader） | 0.0s | switch 本身不慢 |
- **根因（分层）**：**spline 子系统（~885s）是绝对主因**；其内部是「eval_df 里 DF_SPLINE 的 rootNode（CA1[ci]）动态 → `SPLINE_NODE_PACK[node*5]` 动态索引 const 大表 → 驱动为 56 个可能 node 各自做数据流展开 → 组合爆炸」。固定 node 后 31s。fp64 是次因（~310s，与 spline 交互）。**「动态 node → 动态索引 const 大表」是驱动编译地狱**（G 系列结论的最终形态：不是 while 循环、不是 switch、不是函数数，是「数据驱动的动态分派」本身）。
- **教训**：① **编译慢的主因定位要用「子系统减法二分」（DFC_DIAG 开关 + compile_bench 秒级测），不是猜**——while 栈、二分循环、switch 都被逐一排除后才锁定「动态 node 索引」。② **最小实验 shader 必须复刻真实结构的「数据驱动动态分派」特征**，只复刻「循环+分支+const 数组」会得 0.0s 假阴性。③ spline 是 2D flat_cache，是 C2ME FLAT_CACHE_PREFILL 的天然对象——预计算比现场编译更符合「解释器小代码 + 数据」的正解。
- **方案决策点（待用户拍板）**：A. SPLINE 表 SSBO 化（数据从 const→运行时，驱动不展开）；C. spline 值 CPU 预计算到 buffer（C2ME 正统，改动大）；D. VK_KHR_pipeline_binary 预编译分发（用户侧秒级，开发侧仍 13min）。

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

### D23. GPU 引擎在大坐标 chunk 域系统性错值——e2e 验证盲区（2026-08-15 I5 发现，根因待定，可复现）
- **现象**：I5 吞吐对比（gpu_throughput_probe，chunk 批量 1/4/16/64）——16/64 chunks 时 maxDiff 飙到 2.02e-01 / 4.45e-01（应 ~1e-7 量级），而 1/4 chunks 正常（maxDiff 1.04e-06/1.33e-06）。**GPU 输出在特定坐标域系统性错**（gpu=0.045 vs cpu=-0.458，量级级差异，非浮点舍入）。
- **复现（MUST 先复现）**：
  ```
  # 探针：vulkan-proto/gpu_domain_probe.exe（编译见 i-integration-record.md）
  # 单点复现：(784,160,-408) gpu=0.045303289 cpu=-0.458333333 diff=5.036e-01 <== DIFF
  # 完整 z/y 扫描：probe 内置 z-scan（y=160 x=784 z=-432..-404）+ y-scan（x=784 z=-408 y=-64..312）
  # 吞吐复现：gpu_throughput_probe.exe 16 / 64（maxDiff 0.2-0.5）
  ```
- **定位过程（域扫描二分，非猜测）**：
  1. throughput probe 16 chunks → top diff @ (784,160,-408)：先定位到「大坐标 chunk 域」（x=784 > e2e 的 x≤63）
  2. domain probe 定点对比 → (784,-64,-408) 对（diff 3.7e-9）、(784,160,-416) 对、（720,160,-432) 对 → **错误依赖具体 (x,z,y) 组合，不是简单坐标域** 
  3. **z-scan**（y=160 x=784）：z=-432..-412 全对、**z=-408/-404 错**（cz=2/3 格错）
  4. **y-scan**（x=784 z=-408）：**y=-64 对、y∈[-56,248] 几乎全错、y≥256 对**（y≥256 = 无地形常数分支 -0.02499）
  - **关键模式**：错误域 = 「y 中间层 + cz≥2 的 z」组合；正确域 = y 底层/高层（常数分支）或 cz≤1
- **当前根因假设（待 fan-out 判别）**：
  - H1：split() 8 角点拆分（corner 位序 x+2y+4z，坐标 `_chunkX*16+(_cx+dx)*4` 等）与 GPU kernel interp 角点坐标推导/噪声 slot 读取在**大坐标 cell（cz≥2、cy≥1）**下不一致（错位/缺失）
  - H2：GPU kernel interp 的 cell 推导（cy/cz 整数除法 vs floorDiv）在负大坐标下与 split() 不一致
  - H3：split() 内部对某些噪声（ws_scale 分支等）在特定坐标的拆分数值本身错（与 CpuBackend 实例相关）
- **判别进展（2026-08-15 追加，H3 排除 + 布局自洽 + 新嫌疑）**：
  - **H3 排除**（gpu_split_probe，纯 CPU）：split 数据无 NaN/无越界模式；3D 噪声区段（normals[40] base=5984）在 cy=28 vs cy=0 时 iy=131 vs 130（正确区分角点 y）；cz 2 vs 1 时 gz 小数不同（正确区分 z）——**拆分数据自洽，非 split 数值错**。
  - **布局三方自洽**（dump_noise_layout.py）：noise_instances = 200 = 每噪声名（slot）× 8 角点展开（continentalness@c0..c7=实例0..7, erosion=8..15, ridge=16..23）；NOISE_SLOT_BASE[slot]=slot*8、STRIDE=1 → GPU 合成索引 = base+corner 与 split 的 normals[k] 一一对应；NORMAL_PACK[600]=200×3 无越界（D16 已修）。**e2e 域 3.128e-07 验证三方一致**。
  - **corner probe 初步**（gpu_corner_probe，valBuf 角点区段）：interp_4 base=304 peak=6 SLOT_OF_4[17]=3 → 角点值 vb[304+cc*6+3]。但 GPU 角点值连 OK 点也 ≠ finalDensity(角点坐标)——**对比方法存疑**（finalDensity(角点坐标) 含 interp 包装，非角点 delegate 值），需用「split 数据重算角点」或 Python 模拟对拍。
  - **新嫌疑（y=72 反例）**：y-scan 显示 y=72 (cy=17) cz=2 对、y=160 (cy=28) cz=2 错——同 cz 同 cx 仅 cy 不同，若拆分/读取全对不应差异。**未收敛**。
- **判别进展 2（2026-08-15 追加，H5 证实 = 生成器/解释器共同 bug + 分量参照定位 sloped 链）**：
  - **H5 证实（sim_single_point.py，决定性）**：Python 复刻 GPU 解释器（dbg_full_sim）对 (784,160,-408) = **0.045303285**，与 GPU 0.045303289 **完全一致**；而 CPU 参照（DensityBuilder）= -0.458333。**生成器产物（sim 与 GPU shader 同源）对这类坐标求值错——不是 GPU kernel 特有 bug**。
  - **分量参照（gpu_domain_probe + DensityBuilder 显式加载）**：
    | 点 | 参照 sloped | 参照 factor | 参照 fd |
    |---|---|---|---|
    | (784,160,-408) 错点 | **-2.664** | 4.524 | -0.458（range_choice 常数分支） |
    | (784,160,-416) 对点 | -2.820 | 4.656 | -0.458 |
  - **根因链收敛**：GPU/sim 的 sloped_cheese 角点值 = **-0.0165** vs 参照 -2.664（**差 160 倍，结构性错非精度**）→ sloped 错 → final_density 的 range_choice 走了 when_in_range 分支（fd=0.045）而非参照的 when_out_of_range 常数（-0.458）。**嫌疑 = sloped_cheese 链（y_clamped_gradient / factor×sloped / y 传递）在 y 高空角点算错**。
- **根因锁定（2026-08-15 追加，最终）**：
  - **split() 缺 noodle_ridge_b 拆分行**（grep cpu_backend.h 实证）：init() 里 normals[192..199]（noodle_ridge_b@c0..c7，L223-230，splitBase 8576-8660）存在，但 **split() 函数体在 normals[191]（L536，base 8660）结束——noodle_ridge_b 的拆分从未生成**；splitTotal=8672 分配了空间但末段（8576+）是未初始化垃圾/错位数据。
  - **机制**：GPU/sim 的 node[154] = normal_noise(192)（noodle_ridge_b@c0）读 splitCoord[8576..] = 垃圾/别的实例数据 → noodle_ridge_b 采样错 → sloped_cheese 链错（-0.0165 vs 参照 -2.664）→ final_density 的 range_choice 分支选错（when_in_range 0.045 vs when_out_of_range -0.458）。
  - **为什么 e2e 域对**：e2e 域（y∈[-64,-49]）该 range_choice 分支可能未触发 noodle_ridge_b 或垃圾值恰好在无害区间——**e2e 域未覆盖到 noodle_ridge_b 的有效路径**（C12 同类：分支覆盖不足）。
  - **D14 同类 bug 完整版**：`_gen_split_lines` 遍历 DF 树漏了 noodle_ridge_b（藏在 caves/entrances 链某处）——**「遍历树生成 split 与节点化生成 shader 覆盖同一批噪声」铁律第三次被违**（D14 修 spline coordinate，D23 是 noodle_ridge_b）。验证方法：**「cpu_backend.h 里每个实例的 splitBase 都有 split 行」完整性断言**（D14 教训，139→200 实例后未重新断言）。
- **教训（暂）**：① **e2e 验证域（x≤63, y∈[-64,-49], z≤4 = chunk 0 的 cy=0 cell）严重不足**——GPU 引擎在 chunk 0 之外、cy≥1 的坐标域从未验证过，「3.128e-07 逐位一致」只证明了一小块区域；集成到 worldgen（wg_fill_density 批量）才暴露。**验证覆盖必须跨 chunk/cell 域，不能只信单一小域**。② 吞吐对比探针（gpu_throughput_probe）**顺带做正确性抽查**（同点 diff）是意外收获——如果只测时间不测 diff 就漏了。③ 「y 高层对（常数分支）+ y 底层对（cy=0）」容易误判为「大体正确」——**常数分支吸收差异是 C12 同款陷阱**（采样点落在常数分支 = 假正确），必须让采样覆盖有效路径。④ **「角点值 ≠ finalDensity(角点坐标)」**——interp 角点是 delegate 树值（无 interp 包装），对比必须用「split 数据重算」或 Python 模拟，不能直接 sample 角点坐标。⑤ **sim（Python 复刻解释器）= GPU 同源**——sim 与 GPU 一致只能证明「生成器产物内部一致」，**必须与参照（DensityBuilder）对拍才能发现生成器级错误**（本次 sim=GPU=0.045 正是「同源错误」的体现）。⑥ **split 完整性断言必须随实例数变化重新验证**——D14 的「每个实例 splitBase 都有 split 行」断言在 139→200 实例后未重跑，noodle_ridge_b（192-199）漏生成未被发现。
- **根因最终定性（2026-08-15 追加，替代上述「缺 noodle_ridge_b 拆分行」初判——实际是双索引错位）**：
  - **真相**：split() 实际生成了 192 个 normal 的拆分（normals[0..191]，check_split_base.py 实证 normals[160]=noodle@c0 base=8288 等）——**不是「缺行」，是「索引错位」**。初判的「缺 192-199」是误报（用全量序号对比纯 normal 的 normals[]）。
  - **根因 = D16 双索引错位的 gen_cpu 侧残留**：**gen_cpu 的 normal_vec_index/normal_split_base 用「纯 normal 序号」（0..191，跳 old_blended 8 个占位），gen_shader 的 NORMAL_PACK 用「noise_instances 全量序号」（0..199，含 old_blended 占位）**。split() 写 noodle@c0 到 normals[160] base=8288（纯 normal vi=160），但 GPU 实例 168（noodle@c0 全量）读 NORMAL_PACK[168].splitBase=**8384** → **读错位 8 个实例的拆分**（读到 noodle_thickness@c0 的区段）。
  - **证据链**（check_split_base.py + check_splitbase_val.py）：
    | 噪声 | split 写 normals[vi]（纯 normal） | GPU 实例（全量）读 splitBase |
    |---|---|---|
    | noodle@c0 | vi=160 base=8288 | 实例 168 splitBase=**8384** ❌ 错位 8 |
    | noodle_thickness@c0 | vi=168 base=8384 | 实例 176 splitBase=**8480** ❌ |
    | noodle_ridge_b@c0 | vi=184 base=8576 | 实例 192 splitBase=8576（巧合同值） |
  - **e2e 域为何对（验证盲区再背锅）**：e2e 的噪声实例（continentalness/erosion/ridge，实例 <64）在 old_blended 占位**之前** → 纯 normal 序号 = 全量序号 → 无错位；pillar_thickness（152）+ noodle 家族（160+）在 old_blended 占位**之后** → 错位 8 → 错。**D16 修 shader 侧（NORMAL_PACK 全量）但 gen_cpu 侧（split 生成）未对齐——同源双索引一侧改一侧没改**。
  - **教训（追加）**：⑦ **双索引（纯 normal vs 全量含占位）的错位是最隐蔽的——D16 修了一侧，另一侧（gen_cpu split 生成）未同步**——**任何「一侧按全量、另一侧按纯序号」的分配 MUST 两侧同源并加断言**（D16 的「参数表与实例索引严格同源」应扩展到生成器的两侧输出）。
- **重要更正（2026-08-15 追加，上述「双索引错位」被证伪）**：
  - **误判来源**：前述「NORMAL_PACK[168]=8384」数据来自**旧版 final_density.comp**（P2 修改前产物）——当前重新生成后 NORMAL_PACK[168]=**8288**，与 split 写、normal_meta 三方一致。
  - **证伪证据**（三探针对账全部一致）：
    - check_two_alloc.py：gen_shader 与 gen_cpu 的 normal_split_base/normal_vec_index **0 处不一致**；
    - check_meta_vs_splitbase.py：normal_meta[idx].splitBase == normal_split_base[key] 全部 YES（含实例 160/168/176/184/192）；
    - check_split_base.py：split() 写 normals[160]=noodle@c0 base=8288 == NORMAL_PACK[168].splitBase=8288（当前 comp）✓；noodle_ridge_b@c0 base=8576 ✓。
    - gpu_split_probe：base=8576（noodle_ridge_b@c0）拆分 [92,243,61, 0.1754,0.7911,0.1285] 合理，cz 变化有小数差异（数据正常）。
  - **错误收敛到「求值语义层」**：sim/GPU 与参照（DensityBuilder）用**同一拆分数据**在同一坐标 (784,160,-408) 算出不同值（GPU/sim sloped 链 -0.0165 vs 参照 -2.664）——**求值逻辑某节点分叉**（候选：RANGE_CHOICE 边界 / y_clamped / 算术舍入 / interp 角点坐标），**需逐节点对拍定位**。
  - **教训（追加）**：⑧ **对账必须基于当前生成产物，不能依赖旧 comp/spv 的 dump**——「NORMAL_PACK[168]=8384」是旧版误读导致整个「双索引错位」误判（多花数轮）；任何「索引/布局不一致」结论 MUST 先重新生成 + 重新 dump 确认。
- **求值分叉定位（2026-08-15 追加）**：
  - **interp_4 的 delegate = caves/entrances 链**（非 sloped_cheese）——node[155] RANGE_CHOICE(in: y=160 in [-60,321)) 取 a2=node[154]。
  - **node[54]（roughness@c0，slot 7）本身完全正确**：拆分采样 -0.113109157 == CpuBackend.normals[48].sample(784,160,-408) 直接采样 **-0.113109157 逐位一致**（noise_direct_probe）。
  - **分叉在 node[54] 之后**：**node[22]/[33] SPLINE 算出 0**（corner0 (784,160,-408)）——entrances 链的 spline 部分应贡献参照 entrances=0.281 的组成——**spline 大坐标域求值错嫌疑**（spline_eval 在 (784,160,-408) 返回 0）。
  - **状态**：嫌疑收敛到「entrances 链的 spline（node[22]/[33]）在大坐标域算出 0」；判别 = sim 单独调 spline_eval(36/55, corner0, 784,160,-408) vs 参照（待做）。
- **根因最终锁定（2026-08-15 追加）**：
  - **spline_eval 边界外推遇嵌套 value 直接返回 0.0**——GPU while 栈（final_density.comp L441/L445：`(splineValKind[valB] == 0) ? splineValF[valB] : 0.0f`）与 sim 显式栈（dbg_full_sim L205/L214：`svf[valBegin] if vk == 0 else 0.0`）**相同错误**：边界（i<0 或 i>=n-1）外推时若端点 value 是嵌套 spline，直接当 0.0，**未递归求值**。
  - **触发**：spline 55（factor 的 spline，locs=[-0.19,-0.15,-0.1,0.03,0.06]）在 (784,160,-408) 的 coord（continentalness@c0）= 0.060231412 **> 最后 loc 0.06** → 右边界 → vn=嵌套(spline 54) → **0.0**；参照应递归 spline 54（factor 该点=4.524）→ **spline 55=0 → 上层 entrances 链错 → fd 错**。
  - **证据链**：sim normal_noise(0)=0.0602 == CpuBackend 直接采样（flat_cache chain）0.060231412 逐位一致（coord 正确）；spline 55 数据（locs/嵌套）完整；**coord 恰好 > 最后 loc 触发边界**。
  - **e2e 域为何对**：e2e 域的 spline coord 在 locs 范围内 → 正常 Hermite → 对；大坐标域 continentalness=0.0602 恰好 > 0.06 → 边界嵌套 → 0 → 错。**这是 D17 修复后遗留**（D17 只修 node_idx/val_begin 陈旧索引，未处理边界嵌套 value 的递归）。
  - **教训（追加）**：⑨ **spline 边界外推（i<0 / i>=n-1）的端点 value 若为嵌套 spline，必须递归求值——直接 0.0 是错误**（vanilla Spline 的 boundary 是 `getValue(0)` 递归）。⑩ **「coord 恰好跨过最后一个 loc」的边界场景是验证盲区**——e2e 域 coord 全在 locs 内，从未触发边界嵌套。
- **状态**：根因已最终锁定（spline_eval 边界嵌套未递归，GPU+sim 同 bug）；修复 = 边界外推遇嵌套 value 递归求值（GPU while 栈压子帧 / sim 递归）；修复后需重跑 I5 + 全域验证 + e2e 回归。
- **修复完成（2026-08-15 追加，GPU 已验证）**：
  - **GPU 修复（dfc_gen.py `_spline_ssbo_glsl`）**：while 栈边界分支（i<0 / i>=n-1）遇嵌套 value 时不再直接 0.0，改压子帧递归求值（新增 stage 4=等边界 v0 / stage 5=等边界 vn，回填后用子帧值做外推）。
  - **验证（gpu_domain_probe，seed 8576294172403134396）**：
    - (784,160,-408)：**0.045303289（错）→ -0.458333343（对，diff 9.9e-9）**
    - z-scan（y=160 x=784, z=-432..-404）：全部 diff 9.9e-9（原 z=-408/-404 错 0.5）
    - y-scan（x=784 z=-408, y=-64..312）：y=80-120 diff 5e-7~3e-6（float 精度，原 0.03-0.5），y≥128 全 9.9e-9
    - e2e 回归：maxDiff=3.128e-07 / avgDiff=1.097e-08 **与基线逐位一致（零回归）**；pipeline 94.4s 达标
  - **sim 同步修复（dbg_full_sim.py）**：显式栈边界嵌套递归（stage 6/7 + outSlot 修正 (sp-1)*2）——**诊断脚本仍输出 -0.0075（outSlot 索引在深层递归仍有问题），GPU while 栈无此问题（stage 4/5 直接 outVal 回填）**——sim 待继续修（诊断工具，不影响 GPU 交付）。
  - **待办**：I5 吞吐复测（进行中）；8576 终验；sim 深层递归修复；知识库落盘。

### D18. 工具链/流程小坑合集（2026-08-14 本轮，全部已踩）- **① glslc -O（SPIR-V 优化）超时 2min+**——大 const 数组 + 循环 + 动态索引的组合让 glslang 优化器也爆炸（不只驱动）。定位：`glslc -O final_density.comp` 直接超时被杀。教训：**shader 结构必须「小而多函数/小数据」，任何优化器（前端/后端）都扛不住大单循环+大表**。
- **② `#pragma DontInline` 无效**——glslang 不识别该 pragma（生成的 SPIR-V 里无 DontInline，grep 证实）。改用 spirv-dis→手改 `OpFunction` 的 FunctionControl（None→DontInline）→spirv-as 重汇编（patch_noinline.py）。教训：**glslang 的 GLSL pragma 不产生 SPIR-V FunctionControl，要改 SPIR-V 必须 dis+as 层操作**。
- **③ DontInline 编译快但 kernel 仍 TDR**——排除「内联展开导致 kernel 慢」的假设（关键否定实验：DontInline 后 776s→35s 编译，但 kernel 同样 TDR → 内联不是执行卡的原因，真正根因是 D16 的越界读）。
- **④ minimal 系列生成脚本 replace 脆弱**——`minimal4` 的 `main` 替换锚点不匹配静默失败 → 实际测试的是「main 调 eval_df_base 单次」的旧产物，导致「执行链 OK」的**错误结论**（多误导了 4-5 轮）。定位：diff minimal4 vs minimal6 发现 eval_df 定义根本不存在。教训：**二分测试必须改生成器+重生成（或脚本 assert 替换成功），绝不手改字符串静默失败**。
- **⑤ Copy-Item 目标路径重复**——`Copy-Item a.spv .\vulkan-proto\a.spv`（工作目录已是 vulkan-proto）→ 目标变成 `vulkan-proto\vulkan-proto` 失败 → **kernel_exec_test 用旧 spv 跑出无效结果**（多费 1 轮）。教训：**复制前确认目标绝对路径，spv/exe 版本用 `(Get-Item).Length` 或 sha256 核对**。
- **⑥ XoroshiroRandom::split(String) 是 Splitter 方法**——`rng.split("...")` 编译报「不接受 1 参数」，正确是 `rng.nextSplitter().split("...")`（b3d_probe 踩）。
- **⑦ split_probe 链接缺 md5**——OctavePerlinNoiseSampler 构造引用 `wg::md5`，需带 `md5.cpp` 一起编译（LNK2019）。
- **⑧ 每轮 e2e 成本 5-10min**（pipeline 编译 300-660s）——定位期间不宜频繁跑 e2e；先用 kernel_exec_test（1 work item 秒级）判别执行/时间问题，语义问题用 CPU 模拟（dbg_full_sim.py 读 e2e dump）先行，e2e 只做最终确认。

---

## E. GLSL/SPIR-V 语法错误

### E1. GLSL 保留字 `input`/`out`
- `input` 不能作参数名（GLSL 保留字）；`out` 不能作 buffer 变量名。改 `v` / `outBuf`。

### E2. C 风格类型转换 `(double)x`
- GLSL 里 `(double)x` 报 `explicit typecast: required extension GL_NV_explicit_typecast`；用构造函数式 `double(x)`。

### E3. float 字面量 `0f`
- Python `:.17g` 格式化 0.0 输出 `0`，拼成 `0f` 非法；需保证小数点（`0.0f`）。

### E4. fp64 是 core feature 不是扩展
- `VK_KHR_SHADER_FLOAT64_EXTENSION_NAME` 宏不存在；fp64 是 Vulkan 1.1 core feature（`VkPhysicalDeviceFeatures.shaderFloat64`）。

### E5. smear 常量 `1.0e-7f` 是 float 字面量
- vanilla `1.0E-7F` 是 float，shader 里需 `double(1.0e-7f)` 对齐（不是 `1.0e-7` double）。

---

## F. 工程类错误

### F1. `uint32_t` 下溢
- `(i % 32) - 16` 里 `i` 是 uint32_t，减 16 下溢成 2^29；改 `(int)(i % 32) - 16`。

### F2. Python `.pyc` 缓存 stale
- 改 dfc_gen.py 后仍用旧逻辑（`.pyc` 缓存）；删 `__pycache__` 或注意缓存失效。

### F3. PowerShell `Remove-Item *.comp` 误删提交的 shader 源码
- 清理临时文件时 `*.comp` 通配符误删了已提交的 compute.comp/perlin.comp 等；`git checkout` 恢复。教训：清理前先确认哪些是「产物」哪些是「源码」。

### F4. gen_final_density.py 相对路径写文件 → 产物写到错误 cwd（2026-08-14）
- **现象**：改完生成器重跑 gen_final_density.py，检查 `final_density.comp` 没变化（还是旧内容），glslc 编译的 SPV 函数数不变。
- **根因**：`open('final_density.comp', 'w')` 是**相对路径**，写到 pwsh 当前 cwd（CoreSwap 根 `E:\PYTHON\CoreSwap\final_density.comp`），不是预期的 `.investigations\perf-rework\`。我一直在检查/编译 `.investigations\perf-rework\final_density.comp`（旧文件）→ 看到的一切都是旧产物 → 误判「改造没生效」。
- **定位**：`python -c "import dfc_gen; print(dfc_gen.__file__)"` 确认模块加载正确 → 怀疑产物路径 → `Get-ChildItem *.spv -Recurse` 发现两份 final_density.spv（根目录 vs perf-rework）→ 实锤。
- **修复**：`Set-Location .investigations\perf-rework` 后再跑 gen_final_density.py（或全部用绝对路径）。
- **教训**：**生成脚本的相对路径输出 = 隐形炸弹**——检查产物前先确认 cwd；`open(..., 'w')` 写相对路径时，脚本在哪跑产物就在哪。PowerShell 的 `Set-Location` 不跨 pwsh 调用持久，每条命令要么用绝对路径要么在同一命令内 Push-Location/Pop-Location。

### F5. PowerShell `$lines[$i]` 索引数组返回 Object[] → 方法调用失败（2026-08-14）
- **现象**：`$lines[$k].Substring(...)` 报 `MethodInvocationException: [System.Object[]] does not contain a method named 'Substring'`。
- **根因**：`Get-Content` 返回的行数组，`$lines[$i..$j]` **切片返回 Object[]**（不是单行），对数组调 `.Substring()` 失败；且 `[Math]::Min($a, $b)` 在 PowerShell 里参数顺序/类型也易错。
- **修复**：改用 `for` 循环逐行取 + 显式长度判断；或 `Select-String` 精确定位后 `$lines[$i]`（单索引返回标量）。
- **教训**：PowerShell 索引数组：**单索引是标量，范围索引是数组**；对「取某函数体」类操作，用 Select-String 定位行号 + 循环累加长度，别用切片+方法链。

### F6. MSVC 编译 e2e 漏 `/std:c++17` → density.h 内联变量 error C7525（2026-08-14）
- **现象**：`cl dfc_final_backend_e2e.cpp ...` 报 `error C7525: 内联变量至少需要 "/std:c++17"`（density.h 16-25 行），并级联出 density_builder.h:294 的 `==` 类型推导错误（`k == key && lr->target == nullptr` 一堆「无法推导」）。
- **根因**：`density.h` 用了 C++17 内联变量（`inline const ...`），而 cl 默认 C++14。级联的第二处报错是**误导读**——第一处 C7525 使类型系统崩坏后，`==` 才推不出；真正要修的是第一处（加 `/std:c++17`），不是去改 `lr->target == nullptr`。
- **定位**：`Select-String "error C"` 只看 error 行（不看 note 长串），第一行就是 C7525。
- **修复**：加 `/std:c++17`（e2e 编译命令完整形态：`cl ... /EHsc /utf-8 /DNOMINMAX /std:c++17 /O2 ... /link vulkan-1.lib md5.obj`）。
- **教训**：**编译报错先看第一个 error**，后续级联的 note/error 是前一个错误把类型系统打崩后的假象——不要被「== 无法推导」这类 note 带偏去改无关代码。README 的模板命令漏了 `/std:c++17`，已补。

---

## G. 编译慢根因修正系列（2026-08-14，本 session 核心认知）

### G1. 拆 2 链（factor/noodle）不够——factor 122 函数仍 >10min
- **现象**：final_density 拆成 factor.comp（122 函数）/noodle.comp（44 函数）/merge.comp（1 函数）后，noodle pipeline 1.88s，但 factor 仍 >10min。
- **根因**：函数数不是驱动编译时间的决定因素。factor 122 函数里含 **interp_5 的 8 角点巨型表达式**（68KB），驱动在巨型基本块上做寄存器分配 → 超线性爆炸。noodle 44 函数但 interp 体只有 1.6KB → 快。
- **定位**：pipe_bench 逐个 shader 计时 → noodle 1.88s vs factor >10min；再量函数体大小 → interp_5 69868 字符。
- **修复**：无（此路不通）→ 转向 G2。
- **教训**：**「拆 shader 让每 shader 函数数 < 阈值」是伪解**——函数数达标但单个巨型函数体还在，编译照样爆炸。编译时间的真实维度 = **单函数体的表达式复杂度**。

### G2. 角点级拆 11 shader 仍 70s/corner——函数体大小才是主因
- **现象**：按「8 角点不相交（8×13 噪声）」拆成 8 个 corner shader（各 29 函数）+ interp + noodle + merge，corner pipeline ~70s，总计 ~580s（≈ 原 10min）。
- **根因**：corner 函数体 = 角点表达式 **8567 字符**（init 树全展开：0.1171875 + y_clamped_gradient × spline × 13 噪声）。**函数体 5 倍差 → 编译时间 30 倍差**（corner 8.5KB→70s vs noodle 1.6KB→2.4s）。
- **定位**：pipe_bench11 计时（corner 70s vs noodle 2.4s）+ 量函数体大小（corner_0 8567 chars vs noodle interp ~1630 chars）。
- **修复**：无（拆到 29 函数也没用）→ 转向 G3。
- **教训**：**驱动编译时间主因 = 单个基本块的表达式复杂度（巨型展开树），不是函数数**。数据驱动化（spline/normal）降低了函数数和行数（210→29 函数、76338→746 行），但 interp 的 8 角点内联表达式还在 → 编译时间不动。

### G3. normal_noise 数据驱动（139→1 函数）后仍 >10min——interp 68KB 是最后堡垒
- **现象**：normal_noise 合并成 1 个数据驱动函数（NORMAL_PACK 表）后，shader 29 函数/746 行/430KB，但 pipeline 仍 >10min。
- **根因**：interp_0/5 函数体 **69868 字符（68KB）** = 8 角点 × init 树全展开（152 y_clamped_gradient + 288 spline_eval + 592 normal_noise 调用）。normal_noise 调用变小了（1 次查表），但算术树 + spline 调用的展开结构还在。
- **定位**：`Select-String '^float interp_\d+'` 定位定义 → 逐函数累加 body 长度 → interp_0/5 = 69868 chars；数重复模式（y_clamped_gradient 152 / spline_eval 288 / normal_noise 592）。
- **修复**：无（数据驱动化已到极限，interp 是最后一块展开地）→ 需 G4 的两阶段网格预填充。
- **教训**：**interpolated 的 8 角点内联是「展开式生成器」的最后一块**——它把整棵 delegate 树在 8 个角点各自完整展开（8 倍 + 树深幂次放大）。不拆它，编译时间无解。

### G4. C2ME 的答案：interpolated 网格预填充 + 二进制缓存分发（正解）
- **现象/调研**：C2ME（OpenCL）同样面临内核编译问题，但它**从一开始就不展开 interpolated**。
- **机制**（`OpenCLCGen.java` L442-556）：每个 interpolator 生成 `df_interpolator_buffer_prefill_<name>` 函数，**按 cell 网格 dispatch**（`get_global_id = cellX/cellY/cellZ`），每工作项算 **1 个角点** 的 delegate 采样 → 写入 interpolator buffer；主内核只读 buffer 做三线性。**角点采样表达式 = 1 次 delegate 调用（几十字节），无 68KB 巨型展开**。
- **编译时间**：拆 7 个独立内核程序（ESTIMATE_SURFACE/AQUIFER_PREFILL/NOISE_KERNEL/FLAT_CACHE_PREFILL/INTERPOLATOR_PREFILL/BIOME_MULTINOISE/arena）+ **预编译二进制随 mod 分发**（`config/c2me-shader-delivery/*.tar.zst`，`clCreateProgramWithBinary` 秒级加载，失败才现场编译）。
- **教训**：① **interpolated 必须两阶段（角点预填充 buffer + 插值），单 pass 8 角点内联是展开爆炸的根源**（dfc-design Phase 10 当时记了「两 pass 留作优化」，现在必须兑现）② **Vulkan 对应 VK_KHR_pipeline_binary 可按设备型号分发预编译二进制**（之前判断「不能跨机器」是错的——C2ME 证明可以，随 mod 分发 + 兜底现场编译）③ 用户侧 70s 现场编译只有在「没带缓存的新 GPU」才发生，可接受。

### G5. Java 原版代码量 612 行 vs 我们展开 76338 行——「展开 100 倍」的本质
- **现象/对比**：Java 原版 Spline.java 314 行 / DoublePerlinNoiseSampler.java 116 行 / InterpolatedNoiseSampler.java 182 行（合计 ~612 行）；我们的 final_density.comp 原始 76338 行。
- **根因**：Java 是**解释器**（一个类 + 实例数据，树的规模在 JSON 数据不在代码）；我们的生成器是**表达式内联展开**（每个实例展开成独立函数 + 每层复制父表达式 → 嵌套越深膨胀越大）。
- **教训**：**代码生成器的正确形态 = 镜像 Java 解释器（小代码 + 数据表）**，不是展开。spline 数据驱动（56→1）、normal 数据驱动（139→1）已验证这条路；interpolated 网格预填充是最后一块。

### G6. 用户侧 70s 编译 = 每用户每 GPU 首次发生（mod 工程约束）
- **现象**：实测 corner 70s / 单 shader 580s，用户质疑「这是每个用户机器上都会发生的」。
- **根因**：SPIR-V → 驱动机器码（vkCreateComputePipelines）**发生在最终用户机器上，且 pipeline cache 是机器/驱动绑定的**（同一台机器第二次才缓存命中，换机器/换 GPU 首次必现）。
- **教训**：**mod 工程的 GPU 编译时间约束 = 每个用户首次启动**——70s 不可接受（进度条也撑不起），必须根治到秒级（G4 的两阶段 + 数据驱动）或按设备分发预编译二进制（VK_KHR_pipeline_binary）。

---

## 附：错误 → 根因 速查表（一页索引）

| 错误 | 一句话根因 |
|---|---|
| 坐标 float 化 2.2e-1 | 折叠后坐标 ~2^24，float ulp=2 丢小数 |
| /o 放大 1e-2 | 高 octave 被 2^r 放大，要 35 位精度，float 24 位不够 |
| lacunarity 512 vs 1/512 | `2^(-j),j=-fo` = `2^fo`，写反了 |
| maintainPrecision 差 2^25 | `(long)` 向零截断 ≠ `floor` 向下取整（负数） |
| base_3d_noise 对象错 | old_blended_noise 注册的是 InterpolatedNoiseSampler |
| flat_cache 差 0.01-0.1 | flat_cache 是 biome 对齐，不是剥掉 |
| spline 坐标硬编码 | `_gen_spline` 返回硬编码 `(ix,iy,iz)` 丢 flat_cache 对齐 |
| normal 去重失效 | 去重 key 用自增 id，不是 noise key |
| 表达式爆炸 168KB | registry 引用每处内联展开 |
| 函数顺序报错 | GLSL 先声明后使用 |
| 编译 >10min | 驱动内联 34 函数 → SPIR-V 17 倍 → 寄存器分配爆炸 |
| DontInline 行为错 | FunctionControl 位（非 decoration），且对 fp64 有副作用 |
| df_overworld_ridges 无匹配重载 | registry 函数签名 5 参数（sIdx+坐标），缓存分支漏传 sIdx 只传 3 参数 |
| `<<` 右操作数 double | 位移片段裸拼 `* 0.25`，`<<` 优先级低于 `*` → 坐标加括号 |
| DensityBuilder 非类名 | 在 namespace wg，引用需 `wg::` 前缀 |
| C2280 unique_ptr 拷贝已删除 | DoublePerlinNoiseSampler 含 unique_ptr 不可拷贝，用 shared_ptr 直接构造 |
| splitTotal 翻倍/实例累积 | old_blended 去重 key 用 len() 自增，改用参数组合 key |
| y/minY undeclared | 坐标变量硬编码，改 self.fy + shader 模板加 minY 常量 |
| maxDiff=0 假象 | range_choice 常数分支吸收误差，采样点要覆盖阈值两侧 |
| normals[131] 越界 0xC0000005 | gen_shader/gen_cpu 顺序污染 normal_vec_index，gen_cpu 先于 gen_shader |
| final_density 编译 >2min | 210 函数 76338 行超单 shader 规模，纯 float/DontInline 均无效，需拆 shader |
| 拆 2 链/角点级拆仍 70s+ | **编译时间主因 = 单函数体表达式复杂度（巨型展开树），不是函数数**（corner 8.5KB→70s vs noodle 1.6KB→2.4s） |
| normal 数据驱动后仍 >10min | interp_0/5 68KB = 8 角点 × init 树全展开，最后一块展开地 |
| C2ME 怎么解决 | interpolated **网格预填充两阶段**（角点独立 dispatch，不展开）+ **预编译二进制随 mod 分发** |
| Java 原版 612 行 vs 我们 76338 | 解释器（小代码+数据） vs 表达式展开（每层复制父表达式） |
| 用户侧 70s | SPIR-V→机器码在每个用户机器首次发生，pipeline cache 机器绑定，必须根治到秒级 |
| 节点函数化 300 函数仍 >10min | **编译时间 = f(函数数, 函数体) 双维度**——函数体小了但函数数爆炸；两者都要小（noodle 44×1.6KB=2.4s） |
| df_195 no matching | 节点函数注册序 = 父先子后，输出需**全部前向声明**（D6） |
| cy undeclared | 节点函数化后坐标必须用形参 iy，不能再靠 self.cy 切换（D7） |
| df_0 already has a body | 递归注册 idx 冲突，需**先占位后回填**（D8） |
| jagged@c1 KeyError | GPU 角点去重与 CPU 每角点拆分模型冲突，两侧需对齐（D9） |
| D2 节点数组 767 仍慢 | **「每实例数据」=「每实例代码」同样爆炸**——数据驱动化不解决数据量，要结构共享到唯一（D10） |
| eval_df↔interp 递归报错 | GLSL 静态递归检测是符号级，含/不含 interp 的求值函数拆开（D11） |
| splitBase 每角点查表 | 结构共享与参数实例化分离，noise 参数运行时查表（D12） |
| MSVC error C7525 | 内联变量需 `/std:c++17`——漏加 → 级联出「== 无法推导」误导，先修第一个 error（F6） |
| jagged@c1 KeyError（D13） | D1 的 `gen_node` 泄漏进 gen() 的 range_choice → registry ref 不展开每角点 → 噪声只注册 c0。**排查 KeyError 先 grep 新路径函数是否泄漏进旧路径** |
| spline 坐标噪声 GPU 恒 0（D14） | `_gen_split_lines` 不遍历 spline（else 只遍历 argument 等）→ continentalness/erosion/ridge 无 split 行。**「遍历树生成 split」与「节点化生成 shader」必须覆盖同一批噪声** |
| TDR（D15） | `val[158]` 数组被 glslang 复制 10 份/函数 → local memory 12.6KB/work item → spill → kernel 慢 TDR。**解释器 val 栈要做活跃分析/槽位复用**（158→19） |
| TDR 仍存在（D16） | `normal_noise` 实例索引（noise_instances 含 old_blended）与 NORMAL_PACK 参数表（按 normal 序号）**错位越界** → GPU 越界读 const 数组 → TDR。OLD_PACK 同理。**参数表必须与实例索引严格同源** |
| TDR 排查法 | kernel_exec_test（1 work item 空数据 + 60s 超时）判别死循环/极慢；minimal 系列逐函数替换常量二分；**minimal 生成脚本 replace 锚点脆弱（静默失败测旧产物）——必须基于生成器直接输出** |
| e2e y>-64 语义差（D17） | **两个根因**：① `_gen_spline` 的 node_idx/val_begin 在子样条递归前捕获 → 父样条引用第一个子样条（factor 顶层 continents 样条 3.95 被引用成 ridges 6.3 → sloped 27 应为 12.69）；② `weird_scaled_sampler` 被 stub 成 0.0f → entrances Y 分支 clamp(MAX(ws)) 恒 0 → when_out -0.0656 应为 0.0989。**索引必须在递归收集后捕获；生成器里 stub/TODO 是语义差头号嫌疑** |
| GPU 输出恰为 0（D19） | e2e 硬编码 `PER_SAMPLE=320`（ws 后应 352）→ valBuf 越界 → sIdx≥931 work item 输出 0。**宿主与生成器之间的布局常量必须由生成器产出，禁止硬编码；改生成器后全文搜宿主侧对应常量** |
| 模拟=GPU 但≠参照 | 模拟复刻的是「错误生成器产物」——**先用参照 registry 分量探针（getRegistryEntry 采样 factor/sloped/entrances）逐分量对拍**，再信模拟 |
| GLSL 移植模拟 bug | Python 复刻 interp_noise 用旧 l/5 公式（GLSL 是 l/512 /128）→ 数量级错误（1125 vs 0.46）。**GLSL→Python 移植必须逐行对拍公式** |
| glslc -O 超时 2min+ | 大数组+循环+动态索引让 glslang 优化器也爆炸——shader 需「小而多函数」 |
| `#pragma DontInline` 无效 | glslang 不识别该 pragma（SPIR-V 无 DontInline）；手工 patch FunctionControl 编译快但 kernel 仍 TDR（排除内联是执行卡的原因） |
| XoroshiroRandom::split(String) | 是 `Splitter` 的方法（`rng.nextSplitter().split("...")`），不是 XoroshiroRandom 的方法 |
| split_probe 链接缺 md5 | OctavePerlinNoiseSampler 构造引用 `wg::md5`，需带 `md5.cpp` 编译 |
| SSBO 化后仍 350.6s（D22） | spline_coord 的 `switch(coordType)` 使 case 内 `NOISE_SLOT_BASE[0]` 变编译期常量下标 → 常量传播进 normal_noise → NORMAL_PACK 静态化 → 循环展开（每调用 +37~75s）。**改运行时查表 COORD_SLOT_TABLE + fold 特例 → 67.4-101.8s 达标；「动态 node 索引」结论有版本域（const 表成立 / SSBO 不成立）** |
| GPU 大坐标域错值（D23） | I5 吞吐探针发现：GPU 引擎在 e2e 验证域外（chunk 0 外 / cy≥1 / cz≥2）系统性错值（(784,160,-408) gpu=0.045 vs cpu=-0.458）。**e2e 验证域不足（x≤63,y∈[-64,-49],z≤4）——「3.128e-07 逐位一致」只证明小块区域；吞吐探针必须顺带做 diff 抽查；y 高层常数分支吸收差异 = C12 同款假正确**。根因：spline_eval 边界外推 `(kind==0 ? valF : 0.0f)` 对嵌套 value 直接返回 0——vanilla Spline.apply L259/261 是递归求值。修复：_spline_ssbo_glsl while 栈 stage 4/5 边界嵌套压子帧递归（GPU + sim 双修）。（H1/H2/H3 候选排除：角点序/cell 推导/split 数值均验证无差） |

## D23. 大坐标域错值 = spline_eval 边界外推对嵌套 value 返回 0（2026-08-15，GPU+sim 双修）

### 现象
- I5 吞吐探针（gpu_throughput_probe，16/64 chunks 带 diff 抽查）发现：GPU 引擎在 e2e 验证域外系统性错值。
- 决定性单点：(784,160,-408) gpu=0.045303289 vs cpu(finalDensity)=**-0.458333**。e2e 域（x≤63,y∈[-64,-49],z≤4）maxDiff=3.128e-07 全过，域外错值——**e2e 域是 D23 盲区**。

### 根因
- `spline_eval` 边界外推（coord < loc[0] 或 coord > loc[n-1]）原来写成：`(splineValKind[val]==0 ? valF : 0.0f)`——**嵌套 value（kind==1）直接返回 0.0**。
- vanilla `Spline.apply` L259/L261：边界外推是 `this.value[0] + der[0]*(x-loc[0])`——**value[0] 是嵌套样条时要递归求值**，不是 0。
- (784,160,-408) 的 y=160 → cy=28 → 某样条（spline55）coord 落在 locs 末点之外 → 右边界外推 → 嵌套 value 返回 0 → Hermite 用错 v1 → 整链错值。
- **e2e 域不触发**：域内所有样条 coord 都在 locs 内（y∈[-64,-49] 低层，x/z 小 chunk），边界外推分支从不执行——「3.128e-07 逐位一致」只证明域内。

### 定位链
1. gpu_throughput_probe 16/64 chunks 跑 diff 抽查 → 发现 chunk 0 外错值（I5 附带 diff 检查立功——**吞吐探针必须带 diff**，否则只有速度没有正确性）。
2. 单点隔离：(784,160,-408) gpu=0.045 vs cpu=-0.458 → 排除"y 高层常数分支吸收"（C12 类假正确不成立，这里是真的错）。
3. 候选 fan-out 排除：H1 角点序（interp 角点 delegate 顺序 GPU=sim 一致）、H2 cell 推导（cx/cy/cz 公式逐位核对无差）、H3 split 数值（split 坐标 dump 与 CPU 完全一致）——三个候选均无差。
4. sim（dbg_full_sim.py 复刻解释器）复现 0.045 → **生成器产物 + 解释器共同逻辑错误**（不是 GPU kernel 特有）→ 锁定 spline_eval 函数本体。
5. 对照 vanilla Spline.apply 逐行 → 发现边界外推分支的嵌套 value 用 0.0f 占位。
6. 修复后 (784,160,-408) gpu=-0.458333343 diff=9.9e-9；z-scan/y-scan 全 clean；e2e maxDiff=3.128e-07 零回归；I5 各 chunk diff 1e-6~4e-6。

### 修复（GPU 侧，_spline_ssbo_glsl）
- while 栈新增边界嵌套 stage：`i<0` 且 `splineValKind[valBegin]==1` → 父帧 stage=4（左边界），压子帧递归；`i>=n-1` 且 `splineValKind[valBegin+n-1]==1` → 父帧 stage=5（右边界），压子帧递归。
- 子帧完成回填 v0/v1 槽后，父帧 stage 4/5 用 `v[0]+der[0]*(coord-loc[0])` / `v[n-1]+der[n-1]*(coord-loc[n-1])` 完成外推。
- 与普通 Hermite 路径共用同一栈帧回填机制，无新增数组。

### 修复（sim 侧，dbg_full_sim.py 回归工具）
- sim 显式栈移植同样的边界递归（stage 6/7 对应 GPU stage 4/5），但踩了两个**显式栈回填机制**的坑（GPU while 栈直接 outVal 无此问题）：
  1. **outSlot 返回地址被覆盖**：边界嵌套压子帧时 `outSlot[sp] = -1` 把本帧自己的返回地址清掉 → 若本帧本身是子帧（深层嵌套），完成时结果不回填给祖父帧。修复：只改 stage 不覆盖 outSlot。
  2. **父帧 stage 被回填覆盖**：子帧完成回填时 `stageStack[ps>>1] = 2` 无条件覆盖——压 v0 子帧时父帧 stage 已设 1（等 v1），回填后被改成 2 → **跳过 stage 1（v1 求值）→ v1Stack 恒 0** → Hermite 用错值。修复：父帧 stage 压帧时已设恢复点（1=等v1 / 2=Hermite / 6,7=边界），回填只写值不覆盖 stage。
- 验证：sim eval_df(784,160,-408)=-0.458333333 ✓；sim vs e2e-A5 全量对拍 maxDiff=5.7e-9；dbg_full_sim 四点全对齐。
- **教训（显式栈移植）**：显式栈的「返回地址」与「父帧恢复点」是两套状态，压帧时各设一次、回填时**只写数据槽**；任何「回填时顺带改父帧 stage」的优化都会破坏等待语义。

### 教训（D23 综合）
1. **e2e 单域验证是盲区制造机**：域内全过 ≠ 域外正确。吞吐/性能探针必须顺带做 diff 抽查（多 chunk / 多 cell / 多 y 层）。
2. **边界分支是「执行不到」类 bug 的温床**：e2e 域触发不到的分支（边界外推、嵌套边界）必须用**跨域采样**覆盖，不能只靠单域逐位一致。
3. **模拟器复现 0.045 = 生成器+解释器共同逻辑 bug**（不是 GPU 特有）——模拟器与 GPU 同源产物同错，定位时「GPU 特有 vs 共同逻辑」二分法先做。
4. 与 vanilla 逐行对照是最后手段也是最终手段：**Spline.apply 的边界外推是递归求值，不是取 0**。

### 判错经验（D23 提炼，2026-08-15 追加——可复用判错方法，优先级高于单条错误）

> 按「现象→定位→教训」浓缩为可复用判错条目（五段式已在 D23 主体 + 合并段完整记录，此处只沉淀「下次怎么判」）。

1. **「单域逐位一致 ≠ 全域正确」——验证域盲区是「执行不到」类 bug 的天然屏障**：e2e 域（x≤63, y∈[-64,-49], z≤4）maxDiff=3.128e-07 全过，域外 (784,160,-408) 错 0.5——被覆盖的域证明不了未覆盖的域。判错时先问「我的验证域覆盖了哪些坐标域/哪些分支」，再信「逐位一致」；边界外推这类只在特定坐标域触发的分支，必须用跨域采样（多 chunk / 多 cell / 多 y 层）覆盖。

2. **「性能探针必须带 diff」**：吞吐对比若不带正确性抽查，只能发现慢不能发现错——I5 的 gpu_throughput_probe 16/64 chunks 顺带做同点 diff 抽查才暴露 D23（若只测时间，GPU 24-32x 就「达标」了，错值无人知晓）。凡新引擎/新路径的性能对比，探针默认带逐点 diff 输出，diff 是正确性的免费抽查。

3. **「模拟器复现 = 共同逻辑 bug」**：sim（Python 复刻解释器）与 GPU 同源产物同错（sim=GPU=0.045303285）＝生成器+解释器共同逻辑 bug，非 GPU 特有——定位先做「GPU 特有 vs 共同逻辑」二分（sim 能复现 → 直接排除 GPU kernel/驱动层，省掉 kernel 级排查）。但 sim 只能证明「生成器产物内部一致」，**必须与第三方参照（DensityBuilder / registry 分量探针）对拍**才能发现生成器级错误（本次 sim=GPU=0.045 正是「同源错误」的体现，分量参照 sloped=-2.664 vs GPU/sim -0.0165 差 160 倍立即定位链）。

4. **「显式栈回填不覆盖恢复点」**：显式栈的「返回地址（outSlot）」与「父帧恢复点（stage）」是两套状态——压帧时各设一次（stage 设恢复点：1=等v1 / 2=Hermite / 6,7=边界），回填时**只写数据槽**；任何「回填时顺带改父帧 stage」的优化破坏等待语义（跳 stage 1 的 v1 求值 → v1Stack 恒 0 → Hermite 用错值）。栈式移植通用纪律（GPU while 栈 stage 4/5 直接 outVal 回填无此问题，sim 两坑 + judge P1-1 追补印证）。

5. **「对账必须基于当前生成产物」**：「NORMAL_PACK[168]=8384」来自**旧版 final_density.comp** 的 dump → 整个「双索引错位」误判多花数轮；「缺 noodle_ridge_b 拆分行」初判同理（全量序号 vs 纯 normal 序号对比错位）——任何「索引/布局不一致」结论 MUST 先重新生成 + 重新 dump 确认，不依赖旧 comp/spv 的 dump。

6. **「完整性断言随实例数变化重新验证」**（D14 教训第 6 条落地版）：「每个实例 splitBase 都有 split 行」断言在 139→200 实例后未重跑，noodle_ridge_b（192-199）漏检查未被发现——结构规模变化（实例/节点数）后 MUST 重跑完整性断言。

**速查表补充行（追加到现有 D23 行之后；现有 D23 行保留）**：
| sim 显式栈两坑（D23 补充） | 返回地址（outSlot）被 -1 覆盖 → 深层嵌套完成不回填祖父帧；回填 `stageStack[ps>>1]=2` 覆盖父帧恢复点 → 跳过 v1 求值 → Hermite 用 0。**显式栈「返回地址」与「父帧恢复点」两套状态：压帧各设一次、回填只写数据槽**（GPU while 栈 stage 4/5 直接 outVal 回填无此问题） |
| GPU 块级生成极慢（D24） | **逐 block GPU 化不可行**：每 block 走完整 finalDensity 树需全量 split 坐标（8672 floats/点），98304 点/chunk → 3.4GB 上传/chunk（分块 4096 = 24 次 × 142MB）→ 24 chunks 11 分钟未完成 vs CPU 2.5 分钟。**GPU 优势在「算得快」，瓶颈在「喂数据」——split 全量上传带宽死局**。正确方向 = GPU 只算网格角点（768/chunk，wg_fill_density 已实现 22-39x）+ CPU 三线性插值到逐 block，非逐 block 完整树。另：多线程并发 fill 驱动层崩溃（0xC0000005 @ nvtfi）→ fill 加 mutex 串行化（P2-4 闭环，正确性解决但性能更劣化） |

## D24. GPU 块级生成（fillOneChunkCore 密度阶段 GPU 化）性能不可行——split 全量上传带宽死局（2026-08-15）

### 现象
- 立项 003（I6-I8）：让 fillOneChunkCore 密度阶段（16×384×16=98304 点/chunk）走 GPU。
- **正确性侧**：I6 实现（分块 4096 点 batch fill）→ mutex 修复并发崩溃后**无崩溃**，但 24 chunks（8576 区域）运行 **11 分钟未完成**，被主动终止；CPU 基线同区域 **2.5 分钟**。
- **吞吐结论**：GPU 块级路径比 CPU **慢 4 倍+**（且未跑完）。

### 根因（带宽死局，非计算慢）
- GPU shader 求 finalDensity 完整树需要**每个点的全部分解坐标**：`splitTotal=8672` floats/点（CPU 预拆分，double→int32 格点+float 小数）。
- 逐 block 方案：98304 点/chunk × 8672 × 4B = **3.4GB split 数据/chunk** 需上传 GPU。
- 分块 4096（显存限制）→ **24 次 dispatch/chunk**，每次 upload **142MB** + readback → 24 chunks × 24 次 = 576 次大上传。
- **GPU 快在「算」（compute throughput），但这里被「喂数据」（host→device 带宽，PCIe ~16GB/s）完全主导**：142MB/次 × 576 = 82GB 数据搬运 → 分钟级。
- 对比 wg_fill_density（I5）：768 点/chunk × 8672 × 4 = **27MB/chunk**——GPU 只在「网格角点级」批量才有意义（22-39x）。

### 定位链
1. I7 首次运行（无 mutex）：`context=wg_fill_blocks_multi/fillOneChunk`，`code=0xC0000005`，栈在 **nvtfi（NVIDIA 驱动）** ——多线程（block_probe 默认 -threads 自适应）并发调 `h->gpu->fill()` → 共享 buffer 上传/dispatch 竞争 → 驱动层崩溃。**P2-4 预言实锤**。
2. fill() 加 `std::mutex fillMtx` 串行化 → 无崩溃（跑 11 分钟不崩）。
3. 但性能灾难暴露：CPU 基线 2.5 分钟 vs GPU 11 分钟未完成 → 带宽分析定位「split 全量上传」为瓶颈。

### 修复/方向
- **D24 不是代码 bug，是方案不可行**：逐 block 完整树 GPU 化在「split 全量上传」架构下无解。
- **正确方向**（未来若继续）：GPU 只算 InterpolatedDF 网格角点（768 或 1225 点/chunk，wg_fill_density 已验证 22-39x）→ CPU 三线性插值到 98304 逐 block。即「GPU 算网格 + CPU 插值」，不是「GPU 逐 block 完整树」。这需要把 fillOneChunkCore 的密度阶段改为「先 GPU 出网格 → CPU 插值」，工作量中等。
- **当前状态**：I6 代码保留（WG_GPU_FILL=1 时走 GPU 分支，但默认关闭 = CPU 零退化）；**结论 = GPU 块级加速在逐 block 方案下不可行**，回退 CPU 路径为默认（铁律零退化不受影响）。

### 教训
1. **GPU 加速要先算「每点喂多少数据」，不是先算「每点算多少」**：split 全量（8672 floats/点）让「每点数据量」成为带宽死局——GPU 批量加速的前提是「单点数据量小 + 点量大」（网格角点 768 点 × 27MB 可行；逐 block 98304 点 × 3.4GB 不可行）。
2. **吞吐探针（I5 22-39x）证明的是「网格角点批量」**，不能外推到「逐 block」——同引擎、同 shader，采样密度决定可行性（数据量 ∝ 点数）。
3. **多线程并发 GPU 调用必须加锁**（P2-4）：共享 buffer 上传/dispatch 无互斥 → 驱动层崩溃（不是返回错误，是 0xC0000005）——GPU 资源并发是硬约束，不是「可能有问题」。
4. **负面结论也是结论**：I6 的「接线」本身正确（无崩溃、逻辑对），但吞吐不可行——记录「为什么不可行」比假装成功有价值（错误优先原则）。

## D25. GPU 块级生成的方案 C（interp 内容树角点 + CPU 插值）不可行——shader 角点分组噪声结构限制（2026-08-15 深夜段）

### 现象
- 方案 C 目标：GPU 算 5 个 interp 内容树在网格角点（1225 点/chunk）的值 → CPU 逐 block 三线性插值 + 外层非线性 → 最终密度（语义正确 + 带宽优化 ~10MB/chunk）。
- sim 端到端验证：方案 C 重建的最终密度 vs 正确值 **maxDiff=8.7e-2**（与方案 B 同量级）——方案 C 不成立。

### 根因（角点分组噪声结构限制）
- GPU 的 interp 内容树噪声是「**8 份冗余实例**」结构（`continentalness@c0..c7`、`erosion@c0..c7`……参数完全相同，仅 `_key` 带角点后缀）。
- **每实例的 split 行 = 固定角点坐标的拆分**（sim 实证：不同 sIdx 同坐标值相同；corner 0-7 同坐标 range 1.6e-1~3.1e-1）。
- **内容树无法用「该坐标的拆分」算任意点**——实例绑定固定角点坐标。1225 网格角点（共享 cell 角）无法独立求值。
- CPU 的 `InterpolatedDF.arg->sample(p)` 是「共享实例 + 每点坐标参数」——与 GPU 角点分组**结构不兼容**。

### 定位链
1. 方案 B（完整树网格插值）排除：sim 网格加密到 step=(1,2,1) 误差仍 5e-2（min/squeeze/noodle 非线性，插值不成立）。
2. 方案 C 语义前提验证：interp_N（8 角点 delegate + 插值）== 完整树 **maxDiff=0**——interp_N 是唯一正确形式。
3. 角点等价性验证：corner=0 角点 vs 8 角点 delegate 差 2.7e-1（interp[0]）→ 角点分组是硬结构。
4. 实例分配查证：corner 偏移 = 同一噪声 8 份副本（参数相同，split 行按角点坐标拆分）→ 内容树无法算任意坐标。

### 结论（D24 深化）
- GPU 块级生成当前 shader 结构下**无可行路径**：带宽死局（逐 block，D24）+ 结构不兼容（角点分组 vs 共享网格，D25）。
- 要突破需重构生成器的 interp 噪声为「共享实例 + 坐标参数」（CPU 语义）——工程量大且收益存疑（外层非线性仍 CPU）。
- **wg_fill_density 批量 API（22-39x）是 GPU 的实际可用成果**；块级生成保持 CPU（零退化）。

### 教训
1. **shader 的数据布局（角点分组/实例化）决定算法可行性**——带宽可优化，但「实例绑定固定坐标」的结构无法用「每点坐标」求值。设计 GPU 内核前先查数据布局的「可变性」。
2. **同参数多实例是冗余信号**：8 份参数相同的噪声实例 = 「坐标参数化」的退路——若重构为共享实例 + 每点拆分，方案 C 复活（工程量大）。
3. **负面结论深化比重复尝试有价值**：方案 B/C 均被 sim 决定性排除（5e-2/8.7e-2），避免在不可行方向上继续投入。

---

