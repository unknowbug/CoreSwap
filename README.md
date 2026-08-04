# CoreSwap

> **We took the 'Java' out of Minecraft Java Edition. Same mods. Same worlds. Different FPS.**

把 Minecraft Java 版的性能核心（区块生成、实体 AI/寻路）用 C++ 重写，**保留完整的 Java MOD 生态**。
同一个 seed、同一个世界、同一个 MOD —— 底下换成了 C++。

**Why:** Java 版性能被诟病二十年。现有方案都有致命缺陷：Paper 等优化插件还是 Java（治标不治本）；
全 C++ 重写（Cuberite）性能好但 MOD 生态全灭；换基岩版生态全丢。CoreSwap 走的是没人走过的中间路线：
**C++ 性能核心 + Java MOD 层（JNI 桥）**——API 层保持 Java 不变，C++ 化全在 API 下面自由进行。

## 版本管理

仓库按 **Minecraft Java 版版本号**组织目录，每个版本独立维护：

```
CoreSwap/
├── README.md
└── versions/
    ├── 1.20.1/          # ← 当前版本（冻结的现代版，官方 Vulkan 迁移不涉及）
    │   ├── cpp/         # C++ 核心（噪声 + 密度场 + JSON 装配）
    │   ├── java/        # Fabric Loom dev env（vanilla 基准 + 探针）
    │   ├── bench/       # 对比脚本 + POC 报告
    │   └── build.ps1    # 构建脚本
    └── <未来版本>/       # 加新版本 = 加目录
```

版本选择依据：渲染架构稳定（1.18 重构后）、MOD 生态最活跃的冻结期、官方不动的目标。

## 当前状态（1.20.1）

- ✅ **密度场与 vanilla 100% 逐位一致**（12288/12288 点，maxErr=0，IEEE double 精确）
- ✅ **性能 2.43×**（C++ 4.42ms/chunk vs Java 10.75ms/chunk，-O2 基线未优化）
- 📄 详见 [`versions/1.20.1/bench/report.md`](versions/1.20.1/bench/report.md)

## 路线图

1. **代码优化**：紧凑数组 + 索引 + 缓存友好布局（预计 2-5× 提升）
2. **JNI 桥**：`generateRegion` 大块数据一次交换
3. **方块层**：density → 方块状态（surface rules + 区块填充）
4. **实体 AI / 寻路**：第二刀，C++ 化（社区已有 JNI 加速寻路先例）

## 构建

```powershell
# 工具链（MinGW + JDK17，免安装包，不入库）放到仓库根 tools/ 下
powershell -File versions\1.20.1\build.ps1
```

## License

MIT
