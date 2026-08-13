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
