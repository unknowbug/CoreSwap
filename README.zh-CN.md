# CoreSwap

> **我们把 "Java" 从 Minecraft Java 版里掏了出来。同样的 MOD，同样的世界，不一样的帧数。**

[English / English](./README.md)

把 Minecraft Java 版的性能核心——**区块生成** 和 **实体 AI / 寻路**——用原生代码重写，**完整保留 Java MOD 生态**。
同一个种子、同一个世界、同一个 MOD——只是底下换成了原生实现。

**为什么：** Java 版的性能被诟病了二十年。现有的每个方案都有致命缺陷：

| 方案 | 缺陷 |
|---|---|
| Paper 等优化插件 | 还是 Java——治标不治本 |
| Cuberite（全 C++ 重写） | 性能好，但 MOD 生态全灭 |
| 换基岩版 | 生态全丢，还得吃它的版本漂移 |

CoreSwap 走的是没人走过的中间路线：**原生性能核心 + Java MOD 层（JNI 桥）**——MOD API 保持 Java 不变，API 之下一律原生。

## 项目调整（2026-08-30）：核心迁移到 Rust

worldgen 核心已**从 C++ 迁移到 Rust**。现在一个 `worldgen.dll` 打包全部——JNI 桥（`Java_wg_CppWorldgen_*`）与引擎（`wg_*` C ABI）同体，单个 Rust cdylib。C++ 线已归档（仅历史参考）；所有活跃开发在 [`worldgen-core/`](./worldgen-core) + [`versions/`](./versions) 下的版本薄壳。

**为什么换 Rust**：桥 + 引擎同一门语言（少一条工具链）、热多线程路径的内存安全、以及 build-time **密度函数 transpiler**（vanilla JSON → 专用原生代码）兼职正确性裁判——transpiled 管线与运行时解释器证明等价（浮点残差 <5e-7），能抓到生产采样域看不见的语义 bug。

## 当前状态（2026-09-11，v1.0.28）

- ✅ **主世界地形全原生**：密度 → 含水层 → 矿脉 → 表面规则——**雕刻器与装饰层保留 vanilla Java（为 MOD 兼容）**。端到端**存档口径块级对齐 ≈ 99.0%**（大 region sweep 三采样均值 99.01%；残差为密度零面附近的浮点擦边带，非地形结构差）；密度场对齐至浮点残差（<5e-7）。游戏内实测通过（服务端 + 客户端）
- ✅ **下界全原生**：端到端块级对齐 **99.9992%**（两个 4×4 region 共 16 块失配，全部归因已闭合的密度擦边机制）；极限坐标已验证（±30M 角点，98.85–99.85%）
- ✅ **末地全原生（1.0.26 新增）**：主岛 + 外岛——从零移植 vanilla 的 `EndIslands` 密度函数（SimplexNoiseSampler + worldSeed 直传噪声链）与基于位置的 `TheEndBiomeSource` 判定器——**与 vanilla 逐位一致：36/36 chunk、0 块失配**（同种子生产存档对比，Forge 专用服务端，含黑石柱/出口传送门/黑曜石平台等 vanilla feature 全部由 Java 层在原生地形之上正常放置）
- ⚡ **世界生成性能——快过 vanilla Java，且比我们自己的上一版快 5.6 倍**：全新世界、Chunky 预生成、按整片区域的 wall clock 计：

  | 样本 | vanilla Java | CoreSwap 1.0.28 | 加速 |
  |---|---|---|---|
  | 4,225 chunk | 47–50 s（≈86 ch/s） | **36–38 s（≈114 ch/s）** | **≈1.3×** |
  | 16,129 chunk | 190 s（84.9 ch/s） | **151 s（106.8 ch/s）** | **≈1.26×** |
  | 4,225 chunk（1.0.27，同 jar 的旧单车道路径） | — | 202 s → **36 s** | **5.6×**（1.0.28 vs 1.0.27） |

  原因：Minecraft 的 worldgen 车道是串行的——一次一个 chunk，**≈1.4 核忙**。1.0.28 把重的噪声填充工作从这条车道搬到工作池上，同一台机器因此能跑出 **≈11.6 核**的世界生成吞吐。全部收益无损——不靠近似。
  <sub>方法：专用 Fabric 1.20.1 开发服务端、无客户端渲染；全新世界，seed `417950215108767439`，区域中心 (-48,-11)；Ryzen 9 7845HX（12C/24T）、Windows 11；每配置 3–4 次取范围；「ch/s」= wall clock 每秒 chunk 数；CPU 数字为整个 JVM 的利用率。vanilla 全管线跑 Java；CoreSwap 用 Rust 跑密度/含水层/矿脉/表面规则，雕刻器与装饰层留给 vanilla Java。以上是**批量预生成**口径的数字——客户端同机渲染时，体感差异因机器而异。</sub>
- ✅ **jar 自包含**：mod jar 内置完整 worldgen 数据集（849 文件）+ 原生 dll——丢进 `mods/` 即用，零配置、无需外部数据目录；解压带版本哈希自更新
- ✅ **启动期安全网**：surface 引擎可查询的每个噪声采样器都在启动期对照预加载表机械校验——缺 key 在启动即 fail-fast 并给出精确诊断，而不是在稀有 biome 游玩中途崩溃
- ✅ **双加载器支持——Fabric + Forge**：一个 jar 两边通用。Fabric 原生；Forge 经 [Sinytra Connector](https://modrinth.com/mod/connector)（该环境 400+ mod 包实测）。Forge 已是**一等公民、生产级实测**：Forge 47.4.5 专用服务端、生产 SRG 重映射运行时，三个原版维度全部接管并与 vanilla 存档同种子逐位验证
- ✅ **与 Sodium/Iris 互补**：Sodium 管渲染（帧率）、CoreSwap 管生成（探索加载）——互不冲突
- 📦 下载：[Releases](https://github.com/unknowbug/CoreSwap/releases)——`1.0.28`
- 🔭 路线：光照（LIGHT）、实体 AI（Brain / Goal / 寻路）Rust 化

## 安装教程

### 前置要求

- **Minecraft 1.20.1**（Java 版）
- **Fabric Loader 0.15.x**——还没装 Fabric 的话，用 [Fabric 安装器](https://fabricmc.net/use/)（选 MC 1.20.1，点 Install）
- **Java 17**——Fabric Loader 0.15 要求 Java 17+

### 安装步骤

1. **下载**最新的 `coreswap-1.20.1-*.jar`：[Releases](https://github.com/unknowbug/CoreSwap/releases)
2. **装 Fabric**（已装跳过）：Fabric 安装器选 Minecraft **1.20.1**，Install
3. **打开 mods 文件夹**：启动器配置里点 **Open Mods Folder**，或手动到：
   - Windows：`%appdata%\.minecraft\mods`
   - macOS：`~/Library/Application Support/minecraft/mods`
   - Linux：`~/.minecraft/mods`
4. **CoreSwap jar 丢进 `mods/`**——完成
5. **（推荐）加 Sodium + Iris**（[Modrinth](https://modrinth.com/)）——**Sodium 管渲染（帧率），CoreSwap 管生成（探索加载）——互补不冲突**
6. **启动** Fabric 配置。`logs/latest.log` 里验证已生效：
   ```
   [BenchMod] CoreSwap replace mode: C++ worldgen active
   [CppBridge] init seed=... enabled=true
   [CppBridge] initNether seed=... enabled=true
   [CppBridge] initEnd seed=... enabled=true
   ```

### 说明

- **服务端**：Fabric 专用服务端同样可用——同一个 jar 放服务端 `mods/`
- **Forge**：经 [Sinytra Connector](https://modrinth.com/mod/connector) 完整支持——装 Forge 47.4.5 + Connector，同一个 jar 丢 `mods/` 即可。已在生产（SRG 重映射）专用服务端、三个原版维度全量实测
- 日志里那句 "C++ worldgen" 是历史原因——1.0.19 起原生核心已是 **Rust**
- 主世界、下界、末地均由引擎生成；mod 维度回落 vanilla

## 版本组织

仓库按 **Minecraft Java 版本号**组织，每个版本独立目录：

```
CoreSwap/
├── README.md
├── worldgen-core/             # ← 跨版本 Rust worldgen 引擎（活跃开发）
│   └── src/                   # 引擎：density / aquifer / surface / carver / features / noise / biomes
└── versions/
    ├── 1.20.1/              # ← 当前
    │   ├── java/            # Fabric mod 工程（源码；运行环境在 runtime/）
    │   ├── rust/            # 版本薄壳 → 编出 worldgen.dll（cdylib）
    │   ├── cpp/             # 已归档 C++ 核心（历史参考）
    │   ├── data/            # worldgen JSON + 参照方块数据（验证用）
    │   └── docs/            # 工程知识库（01-11 主题篇）
    └── <future versions>/
```

Fabric mod 工程在 [`versions/1.20.1/java`](./versions/1.20.1/java)（fabric-loom）；**运行环境**（世界/mods/配置）单独在 [`runtime/1.20.1/java/run`](./runtime/1.20.1/java/run)——工程是源码、`runtime/` 是环境（2026-09-10 拆分）。构建时自动把新编译的 Rust dll 同步进 mod jar。

## 从源码构建

**工具链（Windows x64）：**

- **Rust**（stable，`cargo`）——编原生核心
- **JDK 17**——JNI 头文件 + Fabric/loom 构建
- **Gradle 8.x**——mod 打包（fabric-loom 1.10）

```bat
:: 1. 编 Rust 核心 + 版本薄壳（产出 worldgen.dll；build.rs 同时从 vanilla JSON
::    重生成 transpiled density 代码）
cargo build --release -p worldgen

:: 2. 编 mod（自动把 dll 同步进 jar）
cd versions\1.20.1\java
gradle build
:: jar 在 build\libs\coreswap-1.20.1-*.jar
```

`build.rs` 里的 transpiler 构建期读取 `versions/1.20.1/data/worldgen`（vanilla worldgen JSON 树）。验证探针（`worldgen-core/src/bin-diag/*`）另需 `blocks.json` + 参照 `.blocks` dump——从 vanilla 1.20.1 服务端导出，有意不入库。

## 工作原理

Rust 核心与 vanilla 完全同构地重建密度场：

- **噪声原语**：Xoroshiro128PlusPlus 随机数、MD5 种子派生、Perlin / octave / double-perlin 采样器——对齐 Mojang 实现
- **密度函数树**：运行时从 vanilla `worldgen` JSON 加载（`noise_settings/<dim>.json` + `density_function/<dim>/*.json`），镜像 `NoiseConfig` 的 visitor 语义——**数据驱动，无维度专属代码**（插值 cell 尺寸、高度、海平面、表面规则全部来自 JSON；多世界原生）
- **Build-time transpiler**（`build.rs`）：把同一份 JSON 编译成专用原生函数（spline 内联、缓存解算、CSE）——独立的第二评估路径，用作正确性裁判并经 env 门控接入生产
- **块级管线**：density → aquifer → ore veins → surface rules → carvers → features，镜像 vanilla 阶段语义（含下界的噪声/世界双高度、末地的 simplex 岛屿密度函数 + 位置式 biome 判定）。**出厂的 1.20.1 mod 里，Rust 引擎负责 density → aquifer → ore veins → surface rules；雕刻器与装饰层保留 vanilla Java（为 MOD 兼容），Rust 侧这两阶段的实现以门控关闭状态出厂。**

## 路线图

1. ✅ **JNI 桥**：区块数据批量交换（已 Rust 化）
2. ✅ **方块层**：density → block states（表面规则 + 区块填充）
3. ✅ **集成**：可安装 Fabric mod / Forge（经 Connector）/ 服务端
4. ✅ **多世界**：主世界 + 下界 + 末地引擎 + 游戏内维度分派
5. ✅ **下界打磨**：转换面漂移、玄武岩/黑石转换带与熔岩海界面对比闭合（端到端 99.9992%）
6. ✅ **末地引擎**：EndIslands（SimplexNoise）密度函数 + 位置式 biome 判定——与 vanilla 逐位一致（36/36 chunk、0 差异）
7. **实体 AI / 寻路**：下一个原生化的核心

## 致谢

- **dustinmoon78**——Forge + Sinytra Connector 兼容：多级 mod jar 定位（`CoreSwapFixHelper`）+ 直接 `JarFile` 解压，400+ mod 包实测。见 [#3](https://github.com/unknowbug/CoreSwap/pull/3)。

## 许可

MIT
