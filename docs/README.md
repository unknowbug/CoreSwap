# CoreSwap Worldgen 技术知识库

> 目的：沉淀 1.20.1 主世界区块生成 C++ 复刻的全部逆向知识，使后续版本迭代（如 1.17.x）
> 只需「对比两个 Java 版本的核心类 diff → 按本库的『版本敏感点』清单定位 C++ 改动点」。
>
> 文档每篇固定结构：**功能目的 → 1.20.1 工作机制（含代码位置）→ 版本敏感点 → 已验证的坑**。
> 迭代时优先读第 7 篇《版本迁移方法论》。

## 目录

| 篇 | 主题 | Java 核心类 |
|---|---|---|
| 1 | [架构与文件映射](01-architecture.md) | 全局 |
| 2 | [随机数派生](02-random.md) | XoroshiroRandom / RandomSplitter |
| 3 | [密度函数系统](03-density-functions.md) | DensityFunctionTypes / NoiseConfig / DensityFunctions |
| 4 | [含水层](04-aquifer.md) | AquiferSampler / ChunkNoiseSampler |
| 5 | [矿脉](05-ore-vein.md) | OreVeinSampler |
| 6 | [表面规则](06-surface-rules.md) | VanillaSurfaceRules / MaterialRules / SurfaceBuilder |
| 7 | [块级流水线与性能](07-block-pipeline.md) | NoiseChunkGenerator / ChunkNoiseSampler |
| 8 | [版本迁移方法论](08-version-migration.md) | diff 流程 + 工具链 + 已知坑 |
| 9 | [多维度定位（通用引擎）](09-multi-dimension.md) | 参数化密度引擎 / 逐位对齐排查 / 负坐标与结构假 diff |

## 状态（2026-08-06）

- **方块层 100% 逐位对齐**（TOTAL 100.0000% / nonAir 100.0000%，seed -8248318472910187742，4×4 chunks 3200..3263）
- **客户端实机可玩**：地形全空气 bug 已修复（stateById 预填 AIR + 直写 container.set 不更新 nonEmptyBlockCount → 全读空气；详见 [07 篇「Java 侧写入路径的坑」](07-block-pipeline.md)）；实测 view-distance 64 无崩溃，视距加大只影响预生成性能
- 性能：16 chunks 并行 110ms（自适应线程数），串行 ~1056ms
- GitHub: unknowbug/CoreSwap；提交纪律：author=unknowbug，中文提交信息
