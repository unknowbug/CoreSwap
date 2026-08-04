# MC C++ Worldgen POC

Minecraft Java 版（1.20.1）区块生成 C++ 化的 POC 项目。

## 状态：✅ POC 完成（2026-08-05）

- **密度场 100% 逐位一致**：C++ 与 vanilla 同 seed 同区域 finalDensity 完全一致（IEEE double 精确，maxErr=0）
- **性能 2.43×**：C++ density 求值 4.42ms/chunk vs Java 10.75ms/chunk（-O2 基线未优化）
- 详见 `bench/report.md`

## 目标

验证"Java 版保留 MOD 生态、性能热点下沉 C++"架构是否成立，第一刀 = **区块生成/加载**。

## 目录结构

- `cpp/` — C++ 核心（噪声 + 密度场 + JSON 装配），CMake 构建
- `java/` — Fabric Loom 1.20.1 dev env，vanilla 基准 harness + 探针
- `bench/` — 对比脚本与报告
- `data/` — 基准输出（vanilla 真值 + 探针数据，不入库）
