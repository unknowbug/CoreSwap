# MC C++ Worldgen POC

Minecraft Java 版（1.20.1）区块生成 C++ 化的 POC 项目。

## 目标

验证"Java 版保留 MOD 生态、性能热点下沉 C++"架构是否成立，第一刀 = **区块生成/加载**。

## 验收标准

- 同 seed 同区域，C++ 密度场与 vanilla 逐点一致（浮点容差）
- C++ 生成耗时有可量化优势
- JNI 数据通路（大块数据一次交换）跑通

## 目录结构

- `cpp/` — C++ 核心（噪声 + 密度场），CMake 构建
- `java/` — Fabric Loom 1.20.1 dev env，vanilla 基准 harness
- `bench/` — 对比脚本与报告
- `data/` — 基准输出（vanilla 真值 + C++ 输出）
