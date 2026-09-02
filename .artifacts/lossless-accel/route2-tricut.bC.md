# .bC 候选 · 历史「GPU e2e 逐位一致 maxDiff 3.1e-07」验证域考古（260903-04）

- **status: draft**（置信度 draft；验证分层 Degraded——纯文件考古，未运行任何命令/git）
- worker：.bC（fan-out 三候选之一，见 `.investigations/lossless-accel/route2-ffi-260903-04.md`）

## 结论：三选一 → **(b) 新证据落历史域外，历史结论保留但必须补域声明**

新证据（tri-cut-260903-04.txt：16 点 5 点 major diff 最大 0.502）**不落在**历史 e2e 结论的验证域内——seed、坐标域、y 覆盖、对比参照四要素全部不同。历史结论在其域内仍然成立（D23 修复后 e2e 回归 + 域扫描复测均通过），但它从未覆盖负坐标远端 chunk / 全 y 柱 / seed -8248…，必须按 §9.7 补显式域声明。

## 证据链

### ① 历史 e2e 的精确覆盖域（dfc_final_backend_e2e.cpp 实读）

源文件：`.investigations/perf-rework/vulkan-proto/dfc_final_backend_e2e.cpp`（仓库内仅存于 .investigations，无 versions 下副本）
- **L57**：`worldSeed = 8576294172403134396ULL` —— **唯一 seed**
- **L81-91**：N=1024；`x = 0 + (i%64)` → **x∈[0,63]**；`y = -64 + (i/64%16)` → **y∈[-64,-49]**（仅底部 16 层）；`z = 0`（zCover 时 -2..0）→ **z≈0**
- **L62-78**：CPU 参照 = DensityBuilder 从 final_density JSON 建树（f64）；GPU 侧 = CpuBackend + 同 spv；三方同程序对比
- 时间线佐证：`versions/1.20.1/docs/10-timewise-archive.md` L1264-1266（A4a：CpuBackend vs DensityBuilder 基线 maxDiff=3.128e-07，TOP i=1004 pos=(44,-49,0)）、L1334（gpu_fill_probe maxDiff=3.128e-07）、L1358（**e2e 域显式声明**：x≤63, y∈[-64,-49], z≤4）、L1385-1391（D23 修复验证 seed 亦为 8576…）

→ 历史域 = **seed 8576294172403134396 × 原点附近小盒（x≤63, y∈[-64,-49], z≤4）× GPU vs DensityBuilder/CpuBackend 参照**。全部历史 GPU 验证（e2e、domain probe、y/z-scan、I5 复测）**只用过这一个 seed**，且大坐标复测全在**正 x（784/720）**侧（L1386-1388）。

### ② 新证据的域（tri-cut-260903-04.txt + .tmp/tmp_diag_tri.cpp 实读）

- **seed = -8248318472910187742**（tmp_diag_tri.cpp L11；与 gpu_corner_probe.rs L13 一致）→ **与历史 e2e 不同 seed**
- 坐标：原点 chunk(0..12) 全过（diff 0~7.45e-8）；chunk(-288,-256)（x≈-4608, z≈-4092..-4096）内 y=-64、y=320 过，**y=-56/0/64/128/200 错（0.039~0.502）** → **负坐标远端 × 全 y 柱**，历史从未采样
- 对比参照 = **CpuBackend.sample 单点**（tmp_diag_tri.cpp 同程序双路径）；历史 = DensityBuilder 树 + e2e 批量。载体同族（都是 CPU 侧参照 vs GPU），但 CpuBackend 单点路径 vs DensityBuilder 的关系历史只在该 seed 的小盒内验证过

### ③ shader/资产生成与 D23 修复记录（重生成纪年）

- `versions/1.20.1/cpp/worldgen/CMakeLists.txt` L25：gpu-assets = cpu_backend.h + final_density.spv（生成器产物）
- 10 时间线 L1339：`gen_final_density.py` 同步 cpu_backend.h 到 gpu-assets（spv 由 glslc 编译复制）
- **L1374（教训⑧）**：「当前重新生成后 NORMAL_PACK[168]=8288 三方一致」——D23 排查期间 spv/cpu_backend.h **已全量重生成**
- L1378：D23 修复改 `dfc_gen.py _spline_ssbo_glsl`（shader 生成器），修复后 L1389 e2e 回归零回归 → **当前 final_density.spv = D23 修复后产物**
- gpu_density_engine.h L3 / vulkan_runtime.h L2：现生产引擎注释自称「与 dfc_final_backend_e2e.cpp 逐位一致（同一 shader + 同一 CpuBackend 数据）」——该注释**无域限定**，即域声明缺失的源头
- ⚠️ 未能确证 cpu_backend.h 与当前 spv 严格同批（禁 git）；但 tri-cut 中 CpuBackend.sample 与原点 chunk GPU 值逐位一致、与 DFC oracle（gpu-corner-probe-260903-04.txt）关系待 .bB 判读，不阻塞本结论

### ④ 知识库交叉印证

- `knowledge/discovered/algorithm-fingerprints.md` L369（发现 #14/D23）：「在 **e2e 验证域（x≤63, y∈[-64,-49], z≤4）** 逐位一致，但在域外大坐标 chunk 域系统性错值」——知识库早已自带域声明；D23 修复只对**已探明的正坐标触发域**做了复测
- `knowledge/discovered/workflow-patterns.md` 发现 #13「多域抽查」+ I5 教训（vulkan-gpu-programming.md L556）：性能探针带 diff 抽查暴露域外错值——本轮 tri-cut 正是该模式的再次触发（**新 seed + 负坐标域 = 又一个未抽查域**）
- 10 时间线 L1394 教训 1：「e2e 单域验证是盲区制造机」——直接适用

## 判定理由（为何不是 (a)/(c)）

- **非 (a) 推翻**：新证据点（seed -8248…、x≈-4608、全 y 柱）没有任何一点落在历史验证域内；域内结论（seed 8576 小盒 maxDiff=3.128e-07，D23 修复后零回归）没有被触碰。「推翻」要求同域反例，不存在。
- **非 (c) 口径不可比**：两轮都是「同点 GPU float vs CPU 侧参照」的 absdiff 对比，载体可比；差异在**覆盖面与 seed**（§9.7 的覆盖面/历史口径要素），不是载体口径冲突。可 comparable，只是域不相交。
- **归 (b)**：历史结论保留 + 域声明 MUST 补：「逐位一致（3.128e-07）仅在 seed 8576294172403134396 × x≤63/y∈[-64,-49]/z≤4 小盒内成立；D23 后追加正坐标 (784/720) 扫描域；**负坐标远端 chunk 与其他 seed 从未被任何 GPU 验证覆盖**」。gpu_density_engine.h L3 的无限定注释应同步加域限定。

## 对 .bA/.bB 的边界输入（非本候选职责）

新错值签名与 D23 高度同族：仅 y 中间层错（y=-64/y=320 常数分支层过）、GPU 值坍缩到小值带（0.0075~0.049）而 CPU=-0.4583 恒定、且只在一个 chunk 出现——但 D23 修复点（边界外推递归）已修并在正坐标验证过，故新证据**不能**直接归因 D23 回归；嫌疑应指向未验证过的组合轴（负坐标 grid 索引 / seed 相关 spline coord 跨界 / 跨 chunk 批量槽位），由 .bA（GPU fill 路径）/.bB（cpu_backend 单点路径）裁决。

## 引用清单

| 证据 | 位置 |
|---|---|
| e2e 域源码 seed/坐标 | .investigations/perf-rework/vulkan-proto/dfc_final_backend_e2e.cpp L57, L81-91, L62-78 |
| e2e 域声明 + D23 全记录 | versions/1.20.1/docs/10-timewise-archive.md L1264-1266, L1327-1334, L1358-1404 |
| 新证据原始输出 | .investigations/lossless-accel/cmd-output/tri-cut-260903-04.txt |
| tri-cut 探针源（seed/双路径） | .tmp/tmp_diag_tri.cpp L9-11 |
| corner probe（seed/域设计） | WorldgenRust/src/bin-diag/gpu_corner_probe.rs L13-16, L113-114 |
| 知识库域声明 | knowledge/discovered/algorithm-fingerprints.md L369（#14）；vulkan-gpu-programming.md L535-556 |
| 引擎无限定注释（域声明缺失源） | versions/1.20.1/cpp/worldgen/src/gpu_density_engine.h L3；vulkan_runtime.h L2 |
| 资产生成纪年 | CMakeLists.txt L25；10 时间线 L1339, L1374, L1378, L1389 |
