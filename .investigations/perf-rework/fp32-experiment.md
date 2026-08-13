# FP32 有损程度探针实验（2026-08-13）

> 目的：回答「降成 FP32 把民用 GPU 性能吃满，方块级有损到什么程度」。
> 方法：C++ 侧在 density 计算路径加 `(float)` 强制舍入（模拟 GPU FP32），用 block_probe / got_export 对比 double 逐位基线。

## 一、实验设计（两档 float 化）

1. **下界实验（节点边界 float）**：噪声/算术/插值/spline 各节点输出 `return (float)x`（节点间 float、节点内仍 double）——FP32 误差下界。
2. **上界实验（噪声内部每步 float）**：`PerlinNoiseSampler::sampleSection` 内部 grad/perlinFade/lerp 每步 `(float)` 截断——更接近真实 GPU FP32。
3. **坐标 float 化（远坐标关键）**：`InterpolatedNoiseDF::sampleImpl` 的 `d/e/f/g/h/i/j/k = pos.x * scaledXzScale ...` 改为 `(float)((float)pos.x * (float)scaledXzScale)`——模拟 GPU 下 `int × float scale` 的坐标舍入。

## 二、结果

### 近坐标（chunk 45,-25 = 块 720,-400，参照 8576/3200）

| 实验 | base_3d_noise 值差异（%a） | 方块对齐 |
|---|---|---|
| 下界（节点边界 float） | — | **99.9994% / 99.9997% 完全不变** |
| 上界（噪声内部 float） | float 尾数 6 位 vs double 13 位（差 ~1e-7） | **99.9994% / 99.9997% 完全不变** |

### 远坐标（chunk 1875000 = 块 30000000，接近世界边界 ±3000 万）

| 项 | 近坐标 | 远坐标 |
|---|---|---|
| base_3d_noise 差异（坐标 float 化） | ~1e-7 | **~0.004（块级）** |
| finalDensity 差异 | 0 | **~1e-3，18/94 行有差异** |

## 三、结论（关键）

1. **FP32 的「损」主要来自坐标精度，不是计算内部精度。**
   - 计算内部 float（噪声/算术/插值舍入）：误差 ~1e-7，对方块判定零影响（近坐标 block_probe 零新增 mismatch）。
   - 坐标 float（`pos.x × scale` 在 float 下舍入）：远坐标（3000 万）下 float ulp ~2~512 → 坐标舍入 ~256 → 噪声采样错位 → finalDensity 差异 ~1e-3，**可能翻转边界方块**。
2. **近坐标几乎无损，远坐标有损**——用户洞察（「取值坐标离初始坐标太近」）完全正确，之前的近坐标验证有盲区。
3. **根因对应 MC 的 `maintainPrecision`**：它把大坐标折叠到 [-2^24, 2^24] 以缓解 double 下的精度损失；但 FP32 下折叠前的坐标 `pos.x × scale` 本身已舍入（ulp ~512），折叠不精确 → 远坐标精度损失无法靠折叠救回。

## 四、分层精度方案（对应用户「宏观噪声 FP64」洞察）

远坐标损失来自 **3D 高频噪声（base_3d_noise）的坐标 float 化**（采样频繁：interpolated buildGrid 1225 角点 × 6 实例；坐标缩放系数 171，放大精度损失）。

建议分层（无损性能 + 保远坐标精度）：
- **坐标折叠 maintainPrecision 用 FP64**（计算量极小：每采样 1 次除法+取整，远坐标精度的关键）。
- **2D 宏观噪声（continentalness/erosion/ridges，flat_cache 缓存）用 FP64**（每 chunk 每实例 build 一次 25 角点 = 低频，算一两次不亏）。
- **3D 高频噪声（base_3d_noise）+ 算术 + 插值用 FP32**（高频、性能关键；且其误差对方块判定鲁棒）。
