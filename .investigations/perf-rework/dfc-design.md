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

- **continents 完整链路**（flat_cache + shifted_noise + shift_a/shift_b + NormalNoise×2）GPU vs CPU 误差 2.7e-7，方块零影响。
- 端到端数据流：seed → randomDeriver → split(noise key) → DoublePerlinNoiseSampler（modern 构造）→ 收集 perm/origin（octBase 布局）→ 上传 → GPU 采样 vs CPU。

### Phase 5 修的 3 个 bug

1. **lacunarity 公式反了**：`2^(-firstOctave)` 应 `2^(firstOctave)`（noise.h 是 `2^(-j), j=-firstOctave`）——错时坐标放大 512 倍，噪声符号都错（误差 0.92）。
2. **maintainPrecision floor→trunc**：noise.h 用 `(long)` 向零截断（trunc），shader 用了 floor（向下取整），负数差 2^25。
3. **normal 噪声去重失效**：去重 key 用自增 `n{len}`，改成 noise key（offset 被 shift_x/shift_z 引用应复用）。

## 八、未完成（Phase 6）

- factor/depth（含 old_blended + spline + 多噪声）的端到端验证。
- shader 尺寸优化。

## 九、踩坑（Phase 1-5 保留）

1. GLSL 保留字 `out` 不能作 buffer 变量名 → `outBuf`。
2. GLSL 的 C 风格类型转换 `(double)x` 在 fp64 下需 `GL_NV_explicit_typecast` → 用构造函数式 `double(x)`。
3. `minecraft:shift_a` 的 type 带前缀，字典 key 要对齐。
