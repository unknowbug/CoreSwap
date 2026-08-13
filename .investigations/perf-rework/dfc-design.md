# DFC shader 生成器设计（dfc_gen.py，2026-08-13）

> 把 CoreSwap 的 DF 树（density_function JSON）编译成 Vulkan compute shader（GLSL），含 fp64/fp32 分层标注。
> 这是 CoreSwap GPU 加速落地的一步：CPU 侧 DF 树 → GLSL 代码 → SPIR-V → GPU 算 density。

## 一、架构

```
density_function/*.json（DF 树，含 registry 引用）
  → DfcGen.gen()：递归生成 GLSL 表达式（精度分层）
  → DfcGen.gen_shader()：拼装完整 shader（辅助函数 + registry 函数 + 噪声函数 + main）
  → glslc → SPIR-V → Vulkan compute
```

## 二、精度分层（核心）

| DF 类型 | 精度 | 理由 |
|---|---|---|
| `minecraft:old_blended_noise`（InterpolatedNoiseSampler） | **fp64（double）** | /o 放大要求 35 位精度（见 interpolated-fp64-finding.md） |
| 其余（NormalNoise/spline/算术/插值） | **fp32（float）** | 无 /o 放大，~1e-7 误差方块零影响 |

- old_blended_noise 生成 `float(interp_noise_N(ix,iy,iz))`：double 采样函数返回 double，转 float 交给下游算术。
- 坐标用 int 块坐标（`ix,iy,iz`），old_blended_noise 内部 `double(px) × scale` 精确；下游 float。

## 三、关键设计点

1. **registry 引用去重（函数化）**：`minecraft:overworld/continents` 被多处引用（spline 的 coordinate），若内联展开表达式爆炸（168KB）。改为生成命名函数 `df_overworld_continents(x,y,z)`，引用处只调用 → 70KB。
2. **嵌套 spline 识别**：spline 的 value 里嵌套 spline 是 `{coordinate, points}`（无 type 字段），需单独识别。
3. **spline 三段式**：对齐 vanilla `Spline.apply` 的边界外推（`i<0`/`i==n-1`）+ 中间 Hermite（`lerp(kd,nv,ov) + kd(1-kd)lerp(kd,p,q)`）。
4. **缓存/插值包装剥离**：`flat_cache`/`cache_2d`/`cache_once`/`interpolated` 在 GPU 端剥掉包装（flat_cache 由 CPU 预填充，对齐 C2ME CacheElimination 思路）。

## 四、当前状态（Phase 2 进展）

- ✅ 表达式生成（全部 15 类 DF 类型）+ registry 引用解析 + 嵌套 spline。
- ✅ base_3d_noise 的 shader 生成 + glslc 编译通过。
- ✅ **NormalNoise（minecraft:noise）/ shifted_noise / shift 的 float 采样函数**（double 坐标拆分 + float 采样，见下）。
- ✅ **全部 10 个 overworld DF 编译通过**：base_3d_noise / continents / depth / erosion / factor / jaggedness / offset / ridges / ridges_folded / sloped_cheese。

### NormalNoise 的精度设计（关键）

NormalNoise（DoublePerlinNoiseSampler）无 /o 放大，但坐标缩放（pos×xz_scale）在远坐标会丢精度。方案：
- **double 做坐标缩放 + maintainPrecision + floor 拆分（精确）** → int32 整数 + float 小数；
- **float 做 grad/fade/lerp 采样（~1e-7）**。

这样 NormalNoise 不需要 CPU 侧拆坐标，GPU 内 double 拆分 + float 采样（与 F3 坐标拆分结论一致）。

### 踩坑（Phase 2 新增）

1. GLSL 函数顺序：噪声函数（normal_noise/interp_noise）必须先于 registry 函数定义（GLSL 先声明后使用）。
2. registry 函数缓存命中分支漏改 `(x,y,z)`→`(ix,iy,iz)`（第一次注册和缓存命中不一致）。
3. Python `.pyc` 缓存导致改代码后仍用旧逻辑（需删 `__pycache__`）。

## 五、Phase 3 进展（端到端验证 ✅）

- ✅ **噪声参数布局**：shader 用 4 个 buffer（CoordBuf int + PermBuf uint + OriginBuf double + OutBuf float），perm/origin 运行时上传。
- ✅ **seed 生成**：复用 noise.h 的 `XoroshiroRandom.split("minecraft:terrain")` + `OctavePerlinNoiseSampler`（legacy 构造）生成 40 个 octave 的 perm/origin（对齐 vanilla）。
- ✅ **端到端验证**（dfc_e2e.cpp，真实 seed 8576294172403134396）：

```
[device] NVIDIA GeForce RTX 4060 Laptop GPU
[result] N=1024, DFC shader(真实seed) vs CPU double: maxDiff=1.422e-08 avgDiff=5.817e-09
```

- 误差 1.4e-8 = base_3d_noise 的 GPU fp64 采样 → `float()` 输出截断（下游算术用 float），方块判定零影响（远小于 1e-2 判定边际）。

## 六、Phase 4 进展（NormalNoise 完整参数 ✅）

- ✅ **NormalNoise 完整参数**：noise_dir 解析 `noise/*.json` 的 firstOctave/amplitudes → 内联 lacunarity(2^-fo)、persistence(2^(n-1)/(2^n-1))、amplitude(0.16667/createAmplitude)。
- ✅ **octBase 统一分配**：old_blended 40 octave、normal 2×n octave，连续分配在 PermBuf/OriginBuf。
- ✅ **NormalNoise float 采样验证**（normalnoise_probe.py，continentalness 参数）：

```
continentalness: firstOctave=-9 n=9 lacunarity=512 persistence=0.501 amplitude=1.5
NormalNoise double拆分+float vs 纯double（远坐标）: maxDiff=3.559e-07 avgDiff=7.569e-08
```

- NormalNoise 误差 ~3.6e-7（double 坐标拆分 + float 采样），方块零影响，与单 octave 拆分（1.6e-7）/多 octave（1.4e-7）一致。
- 踩坑：Python 诊断脚本 float `fade` 漏了 v（v² 写成 v³ 的少一个 v），shader 的 `perlinFadeF` 是对的。

## 七、Phase 5 进展（完整 DF 树端到端验证 ✅ continents）

```
[device] NVIDIA GeForce RTX 4060 Laptop GPU
[result] N=1024, continents DFC shader vs CPU double: maxDiff=2.699e-07 avgDiff=8.380e-08
```

- **continents 完整链路**（flat_cache + shifted_noise + shift_a/shift_b + NormalNoise×2）GPU vs CPU 误差 7.7e-8，方块零影响。
- **flat_cache 语义修正**：不能简单剥掉——vanilla FlatCache.sample 是 biome 对齐（`x>>2<<2, 0, z>>2<<2`），剥掉会差 ~0.01-0.1。改为坐标变量参数化（gen_with_coords）+ biome 对齐；cache_2d/cache_once/cache_all_in_cell 才是剥掉（= delegate 原始坐标）。
- 端到端数据流：seed → randomDeriver → split(noise key) → DoublePerlinNoiseSampler（modern 构造）→ 收集 perm/origin（octBase 布局）→ 上传 → GPU 采样 vs CPU。

### Phase 5 修的 3 个 bug

1. **lacunarity 公式反了**：`2^(-firstOctave)` 应 `2^(firstOctave)`（noise.h 是 `2^(-j), j=-firstOctave`）——错时坐标放大 512 倍，噪声符号都错（误差 0.92）。
2. **maintainPrecision floor→trunc**：noise.h 用 `(long)` 向零截断（trunc），shader 用了 floor（向下取整），负数差 2^25。
3. **normal 噪声去重失效**：去重 key 用自增 `n{len}`，改成 noise key（offset 被 shift_x/shift_z 引用应复用）。

## 八、Phase 6 进展（编译慢根因定位 + DontInline 的坑）

- ✅ **编译慢根因确认**：不是 fp64（fp64 只占 2% 指令），而是**驱动内联展开 34 个函数**（spline 嵌套 + NormalNoise）→ SPIR-V 17 倍膨胀（135KB → 2.34MB，`spirv-opt --inline-entry-points-exhaustive` 0.9s 复现）→ LLVM 寄存器分配超线性爆炸（>10min）。
- ✅ **spline 函数化 + 去重**：嵌套 spline 用函数调用，结构去重 98→19 函数，shader 514KB→135KB。
- ⚠️ **DontInline 的坑**：给非 entry 函数设 `FunctionControl DontInline`（bit 1，注意不是 OpDecorate decoration）编译从 >10min 降到 1.6s，**但引入 fp64 行为错误**——erosion 原始 spv 结果 3.77e-7 正确，DontInline 版 1.48 错误（fp64 的 `maintainPrecision` 等函数不被内联后行为异常）。
- **方向修正（采纳报告建议）**：报告「CPU 预拆分」是对的——把 NormalNoise 的 fp64 坐标拆分移到 CPU（int32 格点 + float 小数上传），GPU 纯 float 采样，同时解决编译慢（无 fp64 内联）和 DontInline 的 fp64 bug。

### Phase 6 修的 bug（端到端揪出）

1. spline 调用坐标硬编码 `(ix,iy,iz)` → 用 `self.cx/cy/cz`（flat_cache 对齐后）。
2. registry 调用坐标硬编码 `(ix,iy,iz)` → 用 `self.cx/cy/cz`。
3. lacunarity `2^(-fo)` → `2^(fo)`；maintainPrecision `floor` → `trunc`；normal 去重 key。

## 九、Phase 7 进展（CPU 预拆分 ✅ 验证通过）

- ✅ **CPU 预拆分方案落地**：NormalNoise 的 fp64 坐标拆分（maintainPrecision + floor → int32 格点 + float 小数）移到 CPU 侧预计算，GPU 只做纯 float 采样（hash 用 int32、grad/fade/lerp 用 float）。
- ✅ **同时解决两个问题**：编译慢（GPU 无 fp64 → 秒级，无需 DontInline）+ DontInline 的 fp64 bug（无 fp64 就没有）。
- ✅ **验证**（dfc_presplit_e2e.cpp，erosion，近坐标）：

```
[dbg] pipeline created   ← 编译秒级（无 fp64）
[result] CPU预拆分 erosion GPU float vs CPU double: maxDiff=3.475e-07 avgDiff=2.867e-07
```

- 精度 3.5e-7（float 采样 ~1e-7 量级，方块零影响），与之前「double 拆分 + float」验证一致。
- **架构变化**：normal_noise 函数签名改为 `normal_noise_N(int sIdx)`（采样点索引），从拆分坐标 buffer 读 int32 格点 + float 小数；坐标链（flat_cache biome 对齐 + shift_x/shift_z 的 offset 采样 + shifted_noise 坐标）移到 CPU 侧重放（端到端验证程序手动重放）。GPU 侧只剩 float 采样 + spline + 算术。
- 拆分坐标 buffer：每采样点 SPLIT_TOTAL 值 = Σ(6 × 2n octave)，每 octave [ix,iy,iz,gx,gy,gz]。

## 十、Phase 8 进展（多后端：坐标链描述 + 通用重放 ✅）

- ✅ **DFC 输出噪声清单**（gen_noise_manifest）：normal 实例的坐标链（type/scale/shift/flat_cache）+ octBase/splitBase + shift 噪声参数（offset 的 firstOctave/amplitudes）。
- ✅ **坐标链 CPU 侧重放**：gen 时记录 coord_chain（noise/shifted_noise/shift_a/shift_b/shift + flat_cache 对齐），shift 噪声（offset）记录到 shift_noises（CPU double 采样，不生成 GPU 函数）。
- ✅ **通用重放函数验证**（dfc_manifest_e2e.cpp 的 splitChain + evalShift）：处理 flat_cache 对齐 + shifted_noise + shift 递归，结果与手动重放一致：

```
[dbg] pipeline created
[result] 通用坐标链重放 CPU预拆分: maxDiff=3.475e-07 avgDiff=2.867e-07
```

- **架构定型**：GPU 后端（GLSL 纯 float 采样 + spline + 算术）+ CPU 后端（坐标链重放 + 拆分）。shift 噪声（offset）是「坐标链的一部分」，CPU double 采样；主噪声（erosion/continentalness）是「density 输出」，CPU 拆分 + GPU float。

## 十一、Phase 9 进展（完整 DF 树：CPU 后端代码生成 + factor 端到端 ✅）

- ✅ **DFC 生成 CPU 后端代码**（gen_cpu）：输出完整 C++ 头文件（CpuBackend），含噪声生成（shift + 主噪声的 DoublePerlinNoiseSampler）+ 坐标链重放（split）+ 拆分（splitDouble/splitOctave）+ perm 收集（collectPerm）。
- ✅ **factor 完整 DF 树端到端验证**（dfc_factor_backend_e2e.cpp，DensityBuilder 做 CPU 参照）：

```
factor DF built
[dbg] pipeline created   ← 编译快（无 fp64）
[result] factor 完整 DF: GPU float vs CPU double: maxDiff=3.879e-06 avgDiff=2.898e-06
```

- factor = spline(coordinate=continents，嵌套 spline value=erosion/ridge) + 3 个 NormalNoise（continentalness n=9 / erosion n=5 / ridge n=6）+ flat_cache + shifted_noise + shift（offset）。全部走 CPU 拆分 + GPU float 采样 + GPU spline，精度 3.879e-06（spline 插值放大 continents 的 float 误差，仍方块零影响）。
- **修复的 bug**：① registry 函数调用参数不一致（首次 5 参数 / 后续 3 参数 → 统一 sIdx+坐标）；② shift 表达式运算符优先级（`(x>>2)<<2*0.25` 被解析为 `<<(2*0.25)` → 坐标加括号）；③ DensityBuilder 在 namespace wg（需 wg:: 前缀）。

## 十二、Phase 10 进展（interpolated cell 三线性插值 GPU 实现 ✅）

- ✅ **interpolated 语义**：cell 三线性插值（CELL 4×8×4），delegate 在 cell 角点（chunkX*16+gx*4, minY+gy*8, chunkZ*16+gz*4）采样，采样点三线性插值 8 角点。GX=5, GY=49, GZ=5（per chunk 1225 角点）。
- ✅ **GPU 实现（单 pass 8 角点）**：每个采样点重算所在 cell 的 8 角点 delegate 采样（normal_noise 读角点拆分坐标）+ 三线性插值。验证（dfc_interp_e2e.cpp，delegate=NoiseDF(noodle)，InterpolatedDF 做 CPU 参照）：

```
[dbg] pipeline created   ← 编译快（无 fp64）
[result] interpolated 8角点单pass GPU float vs CPU double: maxDiff=1.489e-07 avgDiff=7.531e-08
```

- 精度 1.489e-07（float 采样 ~1e-7 量级，方块零影响）。
- **方案取舍**：单 pass 每采样点重算 8 角点（8 倍 delegate 采样），实现简单；块级两 pass（角点网格算一次 + 插值）更高效但需角点网格 buffer，留作优化。noodle 洞穴使用稀疏，8 倍开销可接受。

## 十三、Phase 11 进展（old_blended_noise 5 参数 sample 7 值拆分 ✅）

- ✅ **old_blended_noise 语义**：3 个 legacy OctavePerlinNoiseSampler（lower/upper 16 octave + interpolation 8 octave），5 参数 sample（x,y,z,yScale,yMax，y 轴 smear）。random = randomDeriver.split("minecraft:terrain")。参数 xz_scale=0.25 y_scale=0.125 xz_factor=80 y_factor=160 smear=8。
- ✅ **CPU 预拆分（5 参数 sample 7 值/octave）**：每 octave 拆 [ix,iy,iz,gx,gy(=h-n),gz,fadeY(=h)]，比 normal_noise 的 6 值多 fadeY（y 轴 smear 后 h 仍是 fade 输入，插值位置用 h-n）。40 octave × 7 = 280 值/采样点。
- ✅ **GPU**：pn_section_f32（float grad/fade/lerp 采样，读 7 值）+ double 累加（16 octave 累加精度）。验证（dfc_oldblended_e2e.cpp，InterpolatedNoiseDF 做 CPU 参照）：

```
[dbg] pipeline created   ← 编译快（采样 float，仅累加 double）
[result] old_blended_noise 5参数7值拆分 GPU float vs CPU double: maxDiff=1.247e-07 avgDiff=3.341e-08
```

- 精度 1.247e-07（float 采样 ~1e-7 量级，方块零影响）。

## 十四、CPU 预拆分方案全节点验证收官

| 节点 | 精度 | 状态 |
|---|---|---|
| normal_noise（单噪声 + shift） | 3.5e-7 | ✅ |
| factor 完整 DF 树（spline + 3 噪声） | 3.9e-6 | ✅ |
| interpolated（cell 插值） | 1.5e-7 | ✅ |
| old_blended_noise（base_3d_noise 5 参数 sample） | 1.25e-7 | ✅ |

全部噪声类型验证通过 → 可进入「集成进 block_probe」。

## 十五、Phase 12 进展（old_blended_noise 收编进 DFC ✅）

- ✅ **old_blended 收编进 DFC**：`_old_blended_func` 改 float 版（pn_section_f32 读 7 值拆分坐标 + double 累加）；`gen_cpu` 生成 oldBlendeds 成员 + split7/splitOldBlended 拆分 + 40 octave perm 收集；shader 模板加 pn_section_f32。splitBase 分配 old_blended += 7×40=280。
- ✅ **验证**（dfc_base3d_backend_e2e.cpp，DensityBuilder 做 CPU 参照）：base_3d_noise GPU float vs CPU double maxDiff=1.737e-07，编译快。
- ✅ **修复 old_blended 去重 bug**（C10）：去重 key 原用 `f"ob{len(...)}"`（自增，永不重复）→ 改用参数组合 key（xz_scale/y_scale/xz_factor/y_factor/smear），避免 gen 重复调用累积实例。
- DFC 生成器当前支持：normal_noise ✅ + old_blended_noise ✅ + spline ✅ + 算术 ✅；interpolated 仍手写 shader（待收编）。

## 十六、Phase 13 进展（interpolated 收编 gen 侧 ✅，gen_cpu 侧待做）

- ✅ **gen 侧收编**：interpolated 分支生成「8 角点 delegate 采样 + 三线性插值」；引入 `self.interp_funcs`；shader 模板加 floorDivP + minY 常量。
- ✅ **8 角点去重 key 方案**：`self.noise_key_suffix = "@c{c}"`（8 个独立角点实例，去重 key 含角点），避免 stride 上下文复杂度。noodle 生成 noise_instances=32（4 噪声 × 8 角点）、splitTotal=384，shader 编译通过。
- ✅ **修复**：① `minecraft:y` 硬编码 "y" → `self.fy`；② minY undeclared → shader 模板加 `const int minY = -64`。
- ⏳ **gen_cpu 侧待做（递归拆分生成）**：interpolated 的 8 角点 delegate 拆分，需要 gen_cpu 的 split 递归遍历 DF 树（`_gen_split_lines`），对 noise 节点在「角点坐标」重放 coord_chain + 拆分，对 interpolated 节点生成 8 角点 delegate 拆分（块级 floorDivP + cell 索引）。这是 interpolated 端到端验证的前置。

## 十七、Phase 14 进展（interpolated 收编完成 ✅）

- ✅ **gen_cpu 侧递归拆分生成**（`_gen_split_lines`）：gen_cpu 的 split 从 flat 列表改成递归遍历 DF 树，对 noise 节点在「坐标上下文」重放 coord_chain + 拆分，对 interpolated 节点生成 8 角点 delegate 拆分（块级 floorDiv + cell 索引）。加了 normal_chain_index/normal_vec_index/normal_split_base/old_split_base/old_vec_index 映射（key = 去重 key 含角点 suffix）。
- ✅ **noodle 完整 DF 端到端验证通过**（dfc_noodle_backend_e2e.cpp，DensityBuilder 做 CPU 参照）：

```
[DBG] i=0 pos=(0,-32,0) gpu=0.647133410 cpu=0.647133360 diff=5.022e-08
[result] noodle 完整 DF: GPU float vs CPU double: maxDiff=5.053e-07 avgDiff=1.165e-07
```

- 精度 5.053e-07（float 采样 ~1e-7，方块零影响），编译快。
- ⚠️ **教训（C12）**：首次验证 maxDiff=0 是「采样点恰在 range_choice 的 when_in_range 常数分支（返回 64），误差被阈值判定吸收」——不是真零误差。改采样坐标让 interpolated 采样值跨阈值后误差才体现。**端到端验证必须让采样点覆盖阈值两侧，否则常数分支会掩盖误差**。

## 十八、DFC 生成器全节点支持收官

| 节点 | DFC 支持 |
|---|---|
| normal_noise | ✅ |
| old_blended_noise | ✅ |
| interpolated（cell 插值） | ✅（本轮收编完成） |
| spline | ✅ |
| 算术/range_choice/y_clamped_gradient | ✅ |

DFC 能生成 vanilla 完整 final_density 树的 shader + CpuBackend → 集成 block_probe 的前置全部就绪。

## 十九、Phase 15 进展（final_density 完整树生成 ✅，驱动编译慢 ⚠️）

- ✅ **final_density 完整树生成成功**：noise_instances=139、spline=56、interp=6、split_total=6512。glslc 编译通过（final_density.spv 1.2MB）。
- ✅ **修复 C13**（normals[131] 越界）：gen_final_density.py 的 gen_shader/gen_cpu 顺序颠倒污染 normal_vec_index，改 gen_cpu 先于 gen_shader。
- ⚠️ **驱动编译慢（D3）**：final_density.spv（76338 行，210 函数）vkCreateComputePipelines >2min。OpFunctionCall 2073 次 = factor 的 7 倍。FunctionControl DontInline（210 个）无效——NVIDIA 驱动忽略或 call 消除后仍爆炸。
- **待解决方向**：纯 float（double 累加 → float）减 fp64 寄存器压力 + 减少函数嵌套（spline 深度扁平化 / normal 内联 / 拆 shader）。这是集成 block_probe 的最后一个障碍。

## 二十、未完成

- final_density shader 驱动编译慢的解决（纯 float / 减少嵌套 / 拆 shader）。
- 真正集成进 block_probe。

## 二十一、踩坑（Phase 1-15 保留）

1. GLSL 保留字 `out` 不能作 buffer 变量名 → `outBuf`。
2. GLSL 的 C 风格类型转换 `(double)x` 在 fp64 下需 `GL_NV_explicit_typecast` → 用构造函数式 `double(x)`。
3. `minecraft:shift_a` 的 type 带前缀，字典 key 要对齐。
