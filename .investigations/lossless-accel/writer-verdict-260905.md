# 出界 dirt 写者裁决（260905 · STEP 2 收敛 · 状态 candidate · 已过 judge 260905：可保留，待运行时确证不得升级）

> 验证分层声明：**探针实测只排除 C++ block_probe 的 NOISE+SURFACE**；对 mod 载具实际执行体（Rust）的 NOISE/SURFACE 排除 = b1 静态闭环（Degraded）。judge 260905 意见：①「双跑」为最佳解释而非数据集定点证实（升级前需 dumpbin 核 dll 导出表 + stage-skip 重导消融）；②「Rust feature origin 分叉」降格为 working hypothesis（未做 Rust↔Java origin 对拍）。

## 裁决（working conclusion，非定论）
**mod 路径（cppReplace 存档口径）出界 dirt 的最佳解释写者 = Rust FEATURES 阶段产物（叠加于 Java vanilla features 之上的「双跑」，最佳解释待运行时确证）**，而非 C++ 实现、非 Rust NOISE/SURFACE。

## 证据链（三源交叉）
1. **探针实测（数据层）**：C++ block_probe 默认模式（NOISE+SURFACE）导出在 12/12 出界样本 cell 全为 stone；(195,199) 全列 mod∧C++ 同 dirt = 0 → C++ NOISE/SURFACE 排除（probe-round-260905.md）。
2. **b1 静态闭环（Degraded）**：Rust NOISE 每 cell 只写 air/stone/water/lava，ore_vein 块表无 dirt/sand/gravel 且 y 硬门 [-60,50]；Rust SURFACE dirt 规则只能产表面薄层，机制上无法产「同列 30 散布 dirt y=-41..263」；全库无按列写 dirt 模式（b1-rust-vein-dirt.md）→ Rust NOISE/SURFACE 排除。
3. **b2 链路考古（Degraded）**：生产 dll = WorldgenRust.dll 改名 worldgen.dll（build.gradle L27-47 硬编码，2026-08-30 C++→Rust 转向）；现役 dll（EC4A9AED，260903-03）早于 stage-skip 修复 → mod 世界导出时 **Rust features（ore_dirt/ore_gravel/disk_ 等）全量运行 + Java vanilla features 同时运行**（CppBridge L63-79 的默认 0b011 跳 carver+features 是后加修复）（b2-dll-probe-parity.md）。

## 机制解释（working hypothesis，judge 260905 降格标注）
Rust feature origin 计算（OCEAN_FLOOR top-Y / height provider）**可能**与 Java 分叉 → Rust ore/blob 写在 vanilla 不会写的位置（含出界 y），叠加 Java vanilla 正常 blobs → 双向置换残差。**此假设零验证**——需 Rust↔Java feature origin 对拍才可升级。

## 推论（改变课题性质）
- mod 路径残差主体 = **双跑伪影 + Rust feature 实现偏差**，不是「C++ 复刻错误」——原 aquifer 残差课题在 mod 载具上**结案重定向**。
- 残差正确性验证应回到 **block_probe C++ 全程载具**（生产语义，STEP 1 裁决）——其残差族为 ore 矿石族 desync（98.56%），是真正待修的实现缺口。
- 上轮「ore_dirt origin y∈[0,158] 零出界」审计只约束 C++ feature 实现，对 Rust features 无约束力（载具可比性 #33 的又一实例）。

## 遗留（下一工作节点）
- 运行时确证（可升 candidate→强 candidate）：用带 stage-skip 的现版 dll 重导 mod 世界 → stone→dirt/sand 族应消失；或 Rust 单侧 FULL 导出定位具体 Rust feature 写者。
- Rust feature 与 Java 的 origin 对拍（OCEAN_FLOOR/height provider）为 Rust 侧独立待查项。

## §9.7 口径
seed 8576294172403134396；WGB2 存档口径 4×4 @ chunk(200,200)，16×98304 cell；ns 探针载体 = C++ block_probe 默认模式。
