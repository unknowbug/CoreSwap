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

## 四、当前状态（Phase 1）

- ✅ 表达式生成（全部 15 类 DF 类型）+ registry 引用解析 + 嵌套 spline。
- ✅ base_3d_noise 的 shader 生成 + glslc 编译通过。
- ⏳ 未完成（Phase 2）：
  - NormalNoise（`minecraft:noise`）/ shifted_noise / shift 的 float 采样函数（当前占位返回 0）。
  - 噪声参数（perm/origin/amplitudes）的 params buffer 布局 + 运行时从 seed 生成上传（当前 identity 内联）。
  - 完整 DF 树（factor/depth 等）的端到端验证（对比 CPU）。

## 五、踩坑

1. GLSL 保留字 `out` 不能作 buffer 变量名 → `outBuf`。
2. GLSL 的 C 风格类型转换 `(double)x` 在 fp64 下需 `GL_NV_explicit_typecast` → 用构造函数式 `double(x)`。
3. `minecraft:shift_a` 的 type 带前缀，字典 key 要对齐。
