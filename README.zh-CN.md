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

## 当前状态（2026-08-08）

**最新：v1.0.17（pre-release 测试版）**——可安装的 Fabric mod（MC 1.20.1）。连续修复：heightmap 索引、并发崩溃、原生崩溃日志 handler（异常+调用栈+crash 文件）、完整调用栈 + dll sha256 诊断、内存损坏诊断。兼容 Forge（Sinytra Connector）。

- ✅ NOISE+SURFACE 阶段（密度函数 / 含水层 / 矿脉 / 表面规则）与 vanilla **逐位一致**——同种子同地形，逐块不差（3200 区域 100%；玩家 seed 8576 区域 99.9768%+，剩余 terracotta 带边缘排查中）
- ✅ **世界生成 10-20× 提速**：批量并行生成（~3ms/chunk vs vanilla ~60ms），自适应 `min(核数, 任务数)`
- ✅ 纯算法优化全部无损（FlatCache / Cache2D / spline 缓存）——不靠近似
- ✅ **与 Sodium/Iris 互补**：Sodium 管渲染（帧率）、CoreSwap 管生成（探索加载）——实测 RTX 4060 笔记本 + BSL 光影 + 最大渲染距离全程不卡
- 📦 下载：[CoreSwap 1.20.1 Releases](https://github.com/unknowbug/CoreSwap/releases)
- 🗺️ 版本计划：**全版本覆盖**（含 1.17 及更早）；优先 1.20.x 系列，其余按顺序推进
- 🔭 路线：光照（LIGHT）、实体 AI（Brain / Goal / 寻路）C++ 化

## 安装教程

### 前置要求

- **Minecraft 1.20.1**（Java 版）
- **Fabric Loader 0.15.x**——还没装 Fabric 的话，用 [Fabric 安装器](https://fabricmc.net/use/)（选 MC 1.20.1，点 Install）
- **Java 17**——Fabric Loader 0.15 要求 Java 17+

### 安装步骤

1. **下载**最新 `coreswap-1.20.1-*.jar`（[Releases](https://github.com/unknowbug/CoreSwap/releases)）
2. **安装 Fabric**（已装可跳过）：运行 Fabric 安装器，选 **1.20.1**，Install——启动器里会出现 "fabric-loader-…" 配置
3. **打开 mods 文件夹**：Fabric 配置里点 **打开 Mods 文件夹**，或手动找：
   - Windows：`%appdata%\.minecraft\mods`
   - macOS：`~/Library/Application Support/minecraft/mods`
   - Linux：`~/.minecraft/mods`
4. **把 CoreSwap 的 jar 丢进 `mods/`**——完成
5. **（推荐）加 Sodium + Iris**（[Modrinth](https://modrinth.com/) 下载 1.20.1 版，同样丢进 `mods/`）——**Sodium 管渲染（帧率）、CoreSwap 管生成（区块加载），互补不冲突**；想开光影就再装个光影包（如 BSL、Complementary），在 `选项 → 视频设置 → 光影` 里启用
6. **启动** Fabric 配置。在 `logs/latest.log` 里确认生效：
   ```
   [BenchMod] CoreSwap replace mode: C++ worldgen active
   ```

### 注意事项

- **服务端**：专用 Fabric 服务端同样可用——把 jar 放进服务端的 `mods/` 即可
- **Forge**：通过 [Sinytra Connector](https://modrinth.com/mod/connector) 兼容
- **FEATURES 阶段**（矿物/装饰）仍是 vanilla——**NOISE+SURFACE 已逐位一致**
- 看不到上面的日志：检查 jar 在不在 `mods/`、MC 是否 1.20.1、Fabric Loader 是否 0.15.x、Java 是否 17

## 版本组织

仓库按 **Minecraft Java 版本号**组织，每个版本一个目录：

```
CoreSwap/
├── README.md
└── versions/
    ├── 1.20.1/          # ← 当前
    │   ├── cpp/         # C++ 核心（噪声 + 密度场 + 表面规则）
    │   └── data/        # worldgen JSON + 参照方块数据（验证用）
    └── <未来版本>/
```

## 工作原理

C++ 核心完全复刻 vanilla 的密度场构建：

- **噪声原语**：Xoroshiro128PlusPlus 随机数、MD5 种子派生、Perlin / octave / double-perlin 采样器——与 Mojang 实现逐位一致
- **密度函数树**：运行时从 vanilla 的 `worldgen` JSON 装配（`noise_settings/overworld.json` + `density_function/overworld/*.json`），镜像 `NoiseConfig` 的 visitor 语义
- **InterpolatedNoiseSampler**（`old_blended_noise`）：地形骨架，精确复刻

不需要容差：C++ 密度场与 vanilla 精确到 IEEE double 完全一致。

## 路线图

1. ✅ **JNI 桥**：批量区块数据交换
2. ✅ **方块层**：密度 → 方块状态（表面规则 + 区块填充）
3. ✅ **集成**：可安装的 Fabric mod / 服务端插件
4. **内存优化**：紧凑数组 + 索引 + 缓存友好的布局（预计再提速 2-5×）
5. **实体 AI / 寻路**：第二个 C++ 化核心（社区先例：JNI 加速寻路）

## Credits

- **dustinmoon78** — Forge + Sinytra Connector 兼容：多级 mod jar 定位（`CoreSwapFixHelper`）+ 直接 `JarFile` 提取，400+ modpack 实测。见 [#3](https://github.com/unknowbug/CoreSwap/pull/3)。

## License

MIT
