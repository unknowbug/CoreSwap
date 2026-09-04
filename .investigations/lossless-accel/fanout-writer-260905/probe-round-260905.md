# 出界 dirt 写者定位 · decisive 探针轮（260905 主会话）

## 样本提取（extract_oob_samples_260905.py，.tmp/p2full/）
- 残差中 vanilla∈{stone,granite,diorite,andesite,gravel} 且 mod∈{dirt,sand}：
  **y>200 = 3351 cell；y<-32 = 810 cell**
- **竖列签名**：(195,199) 单列 30 个 dirt cell 散布 y=-41..263（(196,192) 列同形态）——柱状非 blob。

## decisive 探针（本轮核心证据）
- 实验：block_probe（versions\1.20.1\cpp\build-msvc\bin\block_probe.exe，**默认模式 = C++ NOISE+SURFACE，无 feature**）导出 `ns/cpp-noise-surface-260905.blocks`。
- 结果（probe_ns_cells_260905.py）：
  - 12/12 出界样本 cell：vanilla=stone、mod=dirt、**C++ns=stone**（零重叠）。
  - (195,199) 全列：mod dirt=30，C++ns dirt=7（其 oob 仅 5），**mod∧C++ns 同为 dirt 且 vanilla 非 dirt = 0**。
- **判定**：block_probe 的 C++ NOISE+SURFACE **不是** mod 世界出界 dirt 的写者。
- 佐证排除：上轮 cmp_full2 biome 段 diff = 0 → biome 特征表分叉排除（H3 削弱，但 heightmap 依赖型 feature 分叉仍未排除）。

## 当前分叉（fan-out 进行中）
- H1：生产 dll = Rust（WorldgenRust）实现，其 NOISE/SURFACE 有写 dirt 行为（y 全均匀吻合竖列待解释）——worker 3d4c1840 静态排查。
- H2：dll 与 block_probe NOISE+SURFACE 不同源/不同模式（dll≠探针输出）——worker 2bc9420b 排查 dll 来源与管线对拍。
- H3（后备）：Java feature heightmap 依赖分叉——只解释高 y 段，深部 y<-32 待解。
- §9.7 口径：seed 8576294172403134396，4×4 chunk @ (200,200)，WGB2 存档口径 16×98304 cell；ns 探针载体 = C++ block_probe 默认模式（与 mod 载具 dll 载体的可比性正是 H2 待裁决项）。
