# Vulkan 最小原型（2026-08-13）

> 目的：验证「CPU FP64 折叠坐标 → storage buffer → GPU FP32 compute → 读回」的端到端数据流，为分层精度方案落地打地基。

## 环境

- Vulkan SDK 1.4.357.0（winget `KhronosGroup.VulkanSDK`，装于 `C:\VulkanSDK\1.4.357.0`）
- GPU：NVIDIA GeForce RTX 4060 Laptop（PCIe 4.0 x8）
- shader 编译：`glslc`（GLSL → SPIR-V）；宿主编译：MSVC（cl.exe）+ vulkan-1.lib

## 文件

- `compute.comp`：最小 compute shader（binding 0 输入坐标三元组、binding 1 输出 density），本阶段用 `x*x+y*y+z*z` 占位，后续替换为 Perlin 噪声。
- `vulkan_min.cpp`：最小 Vulkan 宿主（instance → device → compute pipeline → storage buffer → descriptor set → dispatch → fence 等待 → 读回 + 与 CPU double 对比）。

## 构建 + 运行

```powershell
$sdk = "C:\VulkanSDK\1.4.357.0"
& "$sdk\Bin\glslc.exe" compute.comp -o compute.spv
cmd /c "call `"D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat`" && cl vulkan_min.cpp /I `"$sdk\Include`" /EHsc /utf-8 `"$sdk\Lib\vulkan-1.lib`" /Fe:vulkan_min.exe"
.\vulkan_min.exe
```

## 结果（首次跑通）

```
[device] NVIDIA GeForce RTX 4060 Laptop GPU
[result] N=1024, GPU float vs CPU double maxDiff = 0.000e+00
[result] 样例: coord(0)=-2.0,-2.0,0.5 -> gpu=8.25 cpu=8.25
[done]
```

- compute 全链路（instance→device→pipeline→storage buffer→dispatch→读回）打通，GPU 结果与 CPU 一致。
- 踩坑：`(i % 32) - 16` 中 `i` 为 `uint32_t` 导致减 16 下溢（坐标变 2^29）——已改为 `(int)(i % 32) - 16`。
- maxDiff=0 是因为坐标取值（`k*0.125`）恰好都能被 float 精确表示；要测真实 float/double 差异需用非精确表示的值（如 0.1）。

## 结果（Perlin 噪声 FP32 vs CPU double）

```
[device] NVIDIA GeForce RTX 4060 Laptop GPU
[result] N=4096, GPU float vs CPU double: maxDiff=4.904e-07 avgDiff=3.822e-08
[result] maxDiff @ i=2453: coord=(0.500,0.900,0.900) gpu=0.581388295 cpu=0.581387804
```

- **高频 3D 噪声 FP32 相对 CPU double 差 ~5e-7（噪声值）**，与 fp32-experiment.md 的 ~1e-7 量级一致 → 对方块判定零影响（此前 block_probe 近坐标实测零新增 mismatch）。
- 纠正：vanilla 1.20.1 `PerlinNoiseSampler` **全是 double**（`double grad`/`perlinFade`/`MathHelper.lerp3`），不是 float——故「高频噪声 FP32」是相对 double 降精度 ~5e-7，而非「对齐 vanilla float」。此前 c2me-dfc-review.md 里「C2ME 噪声 double vs vanilla float」的表述需一并修正。

## 下一步

1. 把 OctavePerlinNoiseSampler 多 octave 叠加 + `maintainPrecision` 折叠加进 GLSL，验证多 octave 的 FP32 差异。
2. 加 `DoublePerlinNoiseSampler`（first + second×1.018）验证 base_3d_noise 完整链路。
3. 批量 dispatch 测量（多 chunk 摊薄固定开销）。
