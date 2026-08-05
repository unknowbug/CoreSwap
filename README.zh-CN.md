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

## 当前状态（2026-08-06）

**v1.0.0 已发布——可安装的 Fabric mod（MC 1.20.1）。**

- ✅ NOISE+SURFACE 阶段（密度函数 / 含水层 / 矿脉 / 表面规则）与 vanilla **100% 逐位一致**——同种子同地形，逐块不差
- ✅ **世界生成 10-20× 提速**：批量并行生成（~3ms/chunk vs vanilla ~60ms），自适应 `min(核数, 任务数)`
- ✅ 纯算法优化全部无损（FlatCache / Cache2D / spline 缓存）——不靠近似
- ✅ **与 Sodium/Iris 互补**：Sodium 管渲染（帧率）、CoreSwap 管生成（探索加载）——实测 RTX 4060 笔记本 + BSL 光影 + 最大渲染距离全程不卡
- 📦 下载：[CoreSwap 1.20.1 v1.0.0](https://github.com/unknowbug/CoreSwap/releases/tag/coreswap-1.20.1-1.0.0)
- 🗺️ 版本计划：优先 1.20.x 全系列 + 次新大版本；1.18/1.19 视需求；1.17- 不计划（世界生成架构差异过大）
- 🔭 路线：光照（LIGHT）、实体 AI（Brain / Goal / 寻路）C++ 化

📄 技术知识库：[`docs/`](docs/README.md)（8 篇：架构 / 随机 / 密度 / 含水层 / 矿脉 / 表面 / 流水线 / 版本迁移）

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

## 路线图

1. **JNI 桥**：`generateRegion` 大块数据一次交换
2. **方块层**：density → 方块状态（surface rules + 区块填充）
3. **集成**：可安装的 Fabric mod / 服务端插件（这是用户真正能用上的节点）
4. **内存优化**：紧凑数组 + 索引 + 缓存友好布局（预计再提 2-5×）
5. **实体 AI / 寻路**：第二个要 C++ 化的核心（社区已有 JNI 加速寻路先例）

## 工作原理

C++ 核心与 vanilla 完全一致地重建密度场：

- **噪声原语**：Xoroshiro128PlusPlus 随机源、MD5 种子派生、Perlin / 八度 / double-perlin 采样器——与 Mojang 实现逐位一致
- **密度函数树**：运行时从 vanilla 的 `worldgen` JSON 装配（`noise_settings/overworld.json` + `density_function/overworld/*.json`），对齐 `NoiseConfig` 的 visitor 语义
- **InterpolatedNoiseSampler**（`old_blended_noise`）：地形主干，精确复刻

无需任何容差：C++ 密度场与 vanilla 精确到 IEEE double 每一位。

## License

MIT
