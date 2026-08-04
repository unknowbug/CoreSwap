# CoreSwap

> **我们把 "Java" 从 Minecraft Java 版里掏了出来。同样的 MOD，同样的世界，不一样的帧数。**

[English / English](./README.md)

把 Minecraft Java 版的性能核心——**区块生成** 和 **实体 AI / 寻路**——用 C++ 重写，**完整保留 Java MOD 生态**。
同一个种子、同一个世界、同一个 MOD——只是底下换成了 C++。

**为什么：** Java 版的性能被诟病了二十年。现有的每个方案都有致命缺陷：

| 方案 | 缺陷 |
|---|---|
| Paper 等优化插件 | 还是 Java——治标不治本 |
| Cuberite（全 C++ 重写） | 性能好，但 MOD 生态全灭 |
| 换基岩版 | 生态全丢，还得吃它的版本漂移 |

CoreSwap 走的是没人走过的中间路线：**C++ 性能核心 + Java MOD 层（JNI 桥）**——MOD API 保持 Java 不变，API 之下一律 C++。

## 版本管理

仓库按 **Minecraft Java 版本号** 组织，每个版本独立目录：

```
CoreSwap/
├── README.md
└── versions/
    ├── 1.20.1/          # ← 当前版本（冻结的现代版，官方 Vulkan 迁移不涉及）
    │   ├── cpp/         # C++ 核心（噪声 + 密度场 + JSON 装配）
    │   ├── java/        # Fabric Loom dev env（vanilla 基准 + 探针）
    │   ├── bench/       # 对比脚本 + POC 报告
    │   └── build.ps1    # 构建脚本
    └── <未来版本>/
```

加新 MC 版本 = 加一个目录。

## 当前状态（1.20.1）

- ✅ **密度场与 vanilla 100% 逐位一致**（12288/12288 采样点，maxErr=0，IEEE double 精确，零容差）
- ✅ **性能 2.43×**（C++ 4.42ms/chunk vs Java 10.75ms/chunk，-O2 基线，未做任何内存优化）
- 📄 详情见 [`versions/1.20.1/bench/report.md`](versions/1.20.1/bench/report.md)

## 路线图

1. **内存优化**：紧凑数组 + 索引 + 缓存友好布局（预计再提 2-5×）
2. **JNI 桥**：`generateRegion` 大块数据一次交换
3. **方块层**：density → 方块状态（surface rules + 区块填充）
4. **实体 AI / 寻路**：第二个要 C++ 化的核心（社区已有 JNI 加速寻路先例）

## 预编译版本

不用装编译器 / JDK，Windows x64 下载即用：

[**下载 CoreSwap-1.20.1-poc.zip**](https://github.com/unknowbug/CoreSwap/releases)（1.6 MB）

包含 `density_probe.exe` / `noise_probe.exe` / `router_probe.exe` / `worldgen.dll` +
vanilla 参照密度数据 + worldgen JSON 数据。快速验证：

```
density_probe.exe -8248318472910187742 vanilla_reference.density worldgen-data
# 期望输出：match=12288/12288 (100.0000%) maxErr=0
```

静态链接 MinGW 运行时——exe 完全自包含，无需额外安装。

## 使用说明

### 前置条件

- **Windows**（当前目标平台）
- **CMake**
- 工具链（免安装 zip，不入库）放到仓库根 `tools/`：

```
tools/
├── mingw/mingw64/bin/        # MinGW-w64 (gcc 16.x) —— 编译 C++ 核心 + JNI DLL
└── jdk17/jdk-17.0.20+8/      # Temurin JDK 17 —— Loom 1.20.1 工具链（JDK 24 太新）
```

### 构建

```powershell
powershell -File versions\1.20.1\build.ps1
```

编译 C++ 核心 + `worldgen.dll`（JNI）、编译 Java JNI 测试并运行（看到 `seed=... => <hash>` 即成功）。

### 用 C++ 核心对比 vanilla（验证一致性）

1. **提取 vanilla worldgen 数据**（`density_probe` 需要）：

```powershell
# 从 1.20.1 的 minecraft jar（客户端或服务端）
jar xf minecraft-1.20.1.jar data/minecraft/worldgen
# 把得到的 data/ 放到如 versions\1.20.1\data\worldgen
```

2. **生成 vanilla 密度参照**（通过 Loom 起 dedicated server，生成区块并导出密度采样）：

```powershell
cd versions\1.20.1\java
# JAVA_HOME 需指向 JDK 17
gradle runServer -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=200 -PbenchOriginZ=200
# → 写出 data/vanilla_<seed>_<size>.density + .json（大端 double，格式见 bench/report.md）
```

3. **C++ 对比**：

```powershell
cd versions\1.20.1
cpp\build\density_probe.exe -8248318472910187742 data\vanilla_-8248318472910187742_4.density data\worldgen
# 期望输出：match=12288/12288 (100.0000%) maxErr=0
```

4. **噪声原语探针**（C++ 侧，54 个 noise key × N 个采样点）：

```powershell
cpp\build\noise_probe.exe <seed> <count>
# 对比参照：gradle runServer -PprobeCount=<count>（输出 Java 侧参照值）
```

### Java 侧探针（经 Loom）

| 模式 | 命令 | 输出 |
|---|---|---|
| 噪声参照 | `gradle runServer -PprobeCount=64` | 54 个 noise key × 64 点 |
| Router 分量 | `gradle runServer -ProuterProbe=true` | 全部分量 + density 计时 |
| 区块基准 | `gradle runServer -PbenchSeed=<s> -PbenchSize=<n> -PbenchOriginX=<x> -PbenchOriginZ=<z>` | 区块生成计时 + 密度文件 |

Java 探针需要 `versions/1.20.1/java/run/eula.txt`（内容 `eula=true`）。

## 工作原理

C++ 核心与 vanilla 完全一致地重建密度场：

- **噪声原语**：Xoroshiro128PlusPlus 随机源、MD5 种子派生、Perlin / 八度 / double-perlin 采样器——与 Mojang 实现逐位一致
- **密度函数树**：运行时从 vanilla 的 `worldgen` JSON 装配（`noise_settings/overworld.json` + `density_function/overworld/*.json`），对齐 `NoiseConfig` 的 visitor 语义
- **InterpolatedNoiseSampler**（`old_blended_noise`）：地形主干，精确复刻

无需任何容差：C++ 密度场与 vanilla 精确到 IEEE double 每一位。

## License

MIT
