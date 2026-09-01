# 拍板材料 — 六项 **confirmed**（260902-01 用户全部拍板批准）

> 汇总自 judge 审查 + 本 session 回归。每项含：结论 / 证据 / 载体口径 / judge 意见 / 状态。

## 1. selector 修复（.b2，260901-04）
- 结论：nether surface rule 6 噪声未预加载（fallback 0.0 → nether_state_selector 恒 true）为真实 bug；修复后存档口径 93.5508% → 93.68~93.90% 区间，改善下界 +0.126pp。
- 证据：judge 四项 PASS（review-b2-fix-20260906.md）；E1/E3 判据（enabled=true + seed 一致）；SURFACE-WARN 0 触发。
- 口径：存档 MCA 直解 + ReadWorldProbe 内存读 vs FULL 参照，4×4 全高度，与探针口径 96% 不可比（§9.7）。
- 状态：candidate（judge 建议授予）。

## 2. SURFACE 口径残差量化（260901-04）
- 结论：SURFACE 参照（无 carvers/features）vs 纯 Rust = 77.4857% → surface 层自身残差 22.5%（basalt→netherrack 157,658 主导）；SURFACE vs FULL 参照差 ~2%（4×4 局部）；judge WARN-4「双 feature 并存重复放置」按架构事实排除。
- 状态：candidate（judge PASS）。

## 3. 容差口径修正（260901-04，supersedes ±369）
- 结论：同 dll 存档口径散布实测 ~2330 块（0.22pp）；新判据 = 区间不重叠 + ≥3 采样，禁单次差值 vs 旧容差；修复改善下界 +0.126pp，点估计 +0.126~+0.348pp。
- 状态：candidate。
- 本 session 新证据：C2 后 3 连跑逐位同值（984600）——同 dll 非确定性并非每次显现（前轮波动或与 Java feature 调度窗口有关）；不推翻「2330 带宽」实测，补充观测点。

## 4. C2 预加载表数据驱动化（260902-01，本 session）
- 结论：step4 硬编码 nether 清单 → surface_rule JSON 构建期动态收集（collect_noise_keys）；overworld 保留静态清单（代码规则无 JSON 源）。
- 证据：静态覆盖完备（JSON 引用 = 旧 6 key）；3 连跑 93.8988% 逐位同值，无回归；无 SURFACE-WARN；cargo 绿。
- 产物：`WorldgenRust/src/surface_rules.rs` / `worldgen_handle.rs`；验证记录 `c2-data-driven-verification.md`；commit 709b006。
- 状态：candidate。

## 附：本 session P2/P3 归因结论（260902-01 晚，已 judge）
- **P2 矿石归因（judge PASS，建议 candidate）**：双重 feature 应用——wg_fill_blocks_multi 内含 carver+feature 阶段，存档链路 Java CARVER/FEATURES 照跑 → 双跑。消融：SKIP_FEATURES → 94.4241%（+5508），矿石全部落回 ref 邻域；SKIP_CARVER 仅 +370。遗留 idk：overworld 同路径矛盾（X1 裁决进行中）。修复方向 judge CONCERN：env 门进程全局，需句柄/调用级显式 flag，勿全局默认翻转。
- **P3 soul 家族（worker 定稿，candidate 级证据）**：上轮「soul_soil 大头在 Java feature」假设证伪（supersedes）；缺口 4140 在 Rust 管线内。V2 三签名：A biome 足迹偏移（x≥3410 边界带）/ B soul_soil 子分支失效 / C floor 侧 soul_sand_layer 分支疑似缺失；.b1a 结构差主导，.b1b idk。下一步 V3 结构对拍（零成本）→ V4 RouterProbe → V5 边界带。
- **judge 审查**：`.artifacts/.c2-p2-ore-attribution/review-judge-20260907.md`（1/2 PASS 建议授 candidate；overworld 矛盾 CONCERN 不阻塞；P3 措辞已修；消融输出已落盘 cmd-output/）。

## 拍板结果（260902-01）
**六项全部 confirmed（用户批准）**。修复类后续（双跑修复设计 / V3-V5 深挖）不在本拍板范围，见 NEXT_SESSION 下轮清单。

