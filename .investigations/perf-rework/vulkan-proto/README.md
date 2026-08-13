# Vulkan 最小原型（GPU 加速预研验证，2026-08-13）

> 目的：验证「CPU FP64 折叠坐标 + 拆分 → storage buffer → GPU FP32 compute → 读回」的端到端数据流，为「分层精度方案」（宏观 F64 CPU + 高频 F32 GPU）落地打地基。

## 一、环境

| 项 | 值 |
|---|---|
| Vulkan SDK | 1.4.357.0（winget `KhronosGroup.VulkanSDK`，装于 `C:\VulkanSDK\1.4.357.0`） |
| GPU | NVIDIA GeForce RTX 4060 Laptop GPU（8GB，PCIe 4.0 x8，FP32 ~15 TFLOPS） |
| shader 编译 | `glslc`（GLSL → SPIR-V） |
| 宿主编译 | MSVC `cl.exe` + `vulkan-1.lib`（vcvars64.bat 环境） |

## 二、文件清单

| 文件 | 作用 |
|---|---|
| `compute.comp` / `vulkan_min.cpp` | 最小数据流验证（`x*x+y*y+z*z` 占位） |
| `perlin.comp` / `perlin_probe.cpp` | 单 octave Perlin 噪声 FP32 vs CPU double（近坐标，原始坐标直接输入） |
| `perlin_split.comp` / `perlin_split_probe.cpp` | 单 octave 坐标拆分（远坐标，CPU 折叠+拆分 int32/float） |
| `octave.comp` / `octave_probe.cpp` | 多 octave + DoublePerlinNoiseSampler 完整链路（base_3d_noise 结构） |
| `batch_probe.cpp` | 批量 dispatch 吞吐测量（复用 octave.comp） |

## 三、构建命令（每个探针通用）

```powershell
$sdk = "C:\VulkanSDK\1.4.357.0"
# 1) shader → SPIR-V
& "$sdk\Bin\glslc.exe" <name>.comp -o <name>.spv
# 2) 宿主 → exe
cmd /c "call `"D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat`" && cl <name>.cpp /I `"$sdk\Include`" /EHsc /utf-8 `"$sdk\Lib\vulkan-1.lib`" /Fe:<name>.exe"
# 3) 运行
.\<name>.exe
```

编译产物（`*.spv`/`*.exe`/`*.obj`）不提交（`.gitignore`）。

## 四、验证结果（按时间序，含完整参数）

### 4.1 最小数据流（compute.comp）

- 参数：N=1024 点，坐标 `(int)(i%32)-16) * 0.125`（0.125 步长网格），`x*x+y*y+z*z`。
- 结果：GPU float vs CPU double maxDiff=0（坐标是 2 的幂，float 精确表示）。
- 踩坑：`(i%32)-16` 中 `i` 是 `uint32_t`，减 16 下溢 → 坐标变 2^29；改为 `(int)(i%32)-16`。

### 4.2 单 octave Perlin（perlin.comp，近坐标）

- 参数：identity perm（`perm[i]=i`）、origin=(0,0,0)、N=4096 点、坐标 `(int)(i%16)*0.1`（0.1 步长，非精确表示）。
- 数据流：原始坐标直接进 shader，`floor`+小数在 shader 内 float 做（**非拆分，坐标是近坐标小值**）。
- 结果：`maxDiff=4.904e-07 avgDiff=3.822e-08`。
- 踩坑：GLSL 保留字 `input` 不能作参数名 → 改 `v`。

### 4.3 单 octave 坐标拆分（perlin_split.comp，远坐标）

- 参数：identity perm、origin=(0,0,0)、N=4096、坐标 `(30000000 + (n%64)*0.13) * 171.103`（远坐标 3000 万块 × scale 171.103，带小数）。
- 数据流：CPU double 做 `maintainPrecision` 折叠 → `floor` 拆 int32 整数 + float 小数 → 上传 6 float/点 `[i,j,k,g,h,l]` → shader 用 i(int)/g(float) 采样。
- 结果：`maxDiff=1.581e-07 avgDiff=2.736e-08`（对比整体 float 化的 2.2e-1，压低 6 个数量级）。

### 4.4 多 octave + Double 完整链路（octave.comp，远坐标）

- 参数：4 个 identity perm（first 2 octave + second 2 octave）、origin 全 0、`amplitudes=[1,1]`、`firstOctave=0`（lacunarity=1、persistence=2/3）、`DOMAIN_SCALE=1.0181268882175227`、`amplitude=1.0`、N=4096、远坐标同上。
- 数据流：CPU 对每 octave（first 2 + second 2×1.018）独立做 `maintainPrecision(coord×lacunarity×2^oct)` 折叠 + 拆分，上传 `4 octave × 6` = 24 float/点，shader 逐 octave 采样叠加 + first/second 相加 × amplitude。
- 结果：`maxDiff=1.410e-07 avgDiff=3.245e-08`（多 octave 不放大误差）。

### 4.5 批量 dispatch 吞吐（batch_probe.cpp）

- 参数：每 chunk 7350 角点（InterpolatedDF buildGrid 1225×6 实例），2+2 octave，M={1,4,16,64,256} chunk，每档测 20 次取平均。
- 结果：

| chunks | 点数 | kernel 时间 | 吞吐(chunk/s) |
|---|---|---|---|
| 1 | 7350 | 183.9 µs | 5436 |
| 4 | 29400 | 499.9 µs | 8002 |
| 16 | 117600 | 1693.3 µs | 9449 |
| 64 | 470400 | 6637.4 µs | 9642 |
| 256 | 1881600 | 25706.0 µs | **9959** |

- 结论：批量摊薄固定开销（单 chunk 183.9µs → 批量每 chunk ~100µs），吞吐 ~10000 chunk/s；CPU 单线程 ~14 chunk/s → **快 ~700×**（2+2 octave 简化；真实 16 octave + 完整 density 按比例缩）。

## 五、验证矩阵汇总（GPU 加速预研全链）

| 验证项 | 误差/结果 | 结论 |
|---|---|---|
| 数据流开销（CUDA 探针） | 144µs vs CPU 47ms | 326× 快，非瓶颈 |
| FP32 计算内部 | ~1e-7 | 方块零影响 |
| 坐标整体 float 化 | 2.2e-1（翻方块） | ❌ 不可接受 |
| 坐标拆分（int32+float） | 1.6e-7 | ✅ 远坐标损失消除 |
| 单 octave Perlin FP32 | 4.9e-7 | ✅ |
| 多 octave + Double 完整链路 | 1.4e-7 | ✅ 多 octave 不放大 |
| 批量吞吐 | ~10000 chunk/s | ✅ 批量是甜点 |

## 六、关键结论

1. **高频 3D 噪声 FP32 全程 ~1e-7 误差，方块零影响**——分层方案精度侧完全验证。
2. **坐标必须拆 int32 整数 + float 小数**（整体 float 化会丢小数 → 翻方块）——这是分层方案能否落地的命门。
3. **批量预生成是 GPU 甜点**（~10000 chunk/s vs CPU 14 chunk/s）。
