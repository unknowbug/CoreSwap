# panic-505 修复 verdict（260903-14，candidate）

> 承接 NEXT_SESSION 260903-13 未闭合课题 #1：surface_rules.rs:505 大 region panic `missing noise sampler`。
> 架构计划：`.investigations/000-架构设计/架构计划-260903-14-surface505-panic立项.md`（用户已批准）。

## 结论（candidate）

- **根因**：overworld surface 规则预加载 noise key 清单（worldgen_handle.rs L272-274 静态 8 key）缺 `minecraft:badlands_pillar_roof`——`place_badlands_pillar`（surface_rules.rs:1372，Java L214 对拍点）运行时 `get_noise` → :505 `expect` panic。仅 eroded_badlands biome 列且 e>0 时触发 → sweep 至 ~2304-2560 chunk 进入 badlands 区域才崩（延迟触发自洽）。
- **修复**：清单补 `"minecraft:badlands_pillar_roof"`（一行）。
- **候选 b/c 排除**：NoiseThreshold key 缺失走 :131 warn 回退 0.0 不 panic（scout 穷举 get_noise 调用方确认 pillar_roof 是唯一漏项）。

## 验证（Full 层）

| 项 | 结果 | 载体 |
|---|---|---|
| P0 复现 | panic 稳定复现 block 8→9 边界，backtrace = fill_chunk_blocks→build_surface→:505 | sweep-repro-260903-14.txt（RUST_BACKTRACE=full） |
| 修复后 4096 chunk 全程 sweep | **4096/4096 完成，无 panic**，total wall=121534ms；各 block hits/misses 与修复前逐项相同。⚠️ 载体定位：崩溃回归载体（计数器口径），非逐位正确性载体（逐位由 64-chunk hash 承担） | sweep-fixed-4096-260903-14.txt |
| 零语义回归（四臂 hash） | off/shared × L2 开关四臂 agg hash 全部 = `f2b1a3932c6e589e`（与 260903-13 confirmed 值一致；四臂完整输出落盘）；L2 stats 开关行为正常 | estopt-ab-4arms-260903-14.txt |
| 存档口径 Full 回归（overworld 4×4@3200,3208，seed B，FULL 参照，ReadWorldProbe） | **3 采样 {98.9969%, 99.0284%, 99.0067%}，均值 99.0107%**，与修复前历史 98.9520%（C4 基线，早于 est 优化）区间不重叠且方向向上；run 间散布 495 块在 #10 非确定带宽（~2330 块）内——改善幅度在散布带内，仅作无回归佐证，不具单修复归因力 | save-full-regress-run{1,2,3}-260903-14.log |

## §9.7 可比性声明

- sweep 命中率/est L2 stats：estopt_mt_bench sweep 口径，修复前后同 seed 同 region 同顺序，逐项可比。
- 存档口径 98.99-99.03%：MCA/内存读（ReadWorldProbe）vs vanilla FULL 参照；与 SURFACE 口径、纯 Rust 口径不可比分列；与 C4 基线 98.9520% 同口径可比（但基线早于 est 优化，向上偏移含 est 收口贡献，非本修复单独效应）。
- 四臂 hash：64-chunk 聚合 FNV，非逐位全量 diff 口径（与 260903-11 C2 声明一致）。

## 遗留 / 通用升级点（不阻塞本 verdict）

1. 非 overworld JSON 路径 collect_noise_keys 只收 `noise_threshold.noise` 字段——当前无 panic 风险（NoiseThreshold warn 回退），但 get_noise 调用方扩展时同类风险仍在。**通用判据：新增 get_noise 调用点必须同步预加载来源**（已交知识库）。
2. jar 内 worldgen-data 布局（minecraft/ 直下）与 CoreSwapFixHelper marker 期望（data/minecraft/…）不一致——显式 `-PcppWorldgenDir` 绕过即正常；资源解压路径为死路（见错误台账 E1）。
3. 沿用 idk：Java 55→33 漂移前半段；pc_e2e 256 vs stage 64 口径 ~13% 差；P2.4 剩余 ~1.5×（Partial 已声明）。

## 证据索引

- 勘探：`.investigations/panic-505/scout-map.md`（recode-scout 产物）
- 复现/修复验证：`.investigations/panic-505/cmd-output/`（sweep-repro / sweep-fixed-4096 / estopt-ab-regress / save-full-regress-run1-3）
- 修复 diff：`WorldgenRust/src/worldgen_handle.rs` L272-275（diff 见 git 工作区，随本 verdict 一并提交）
- judge 审查：`.investigations/panic-505/review-panic-fix-260903-14.md`（PASS，2 should-fix 已清偿：四臂落盘补齐 + 315→495 块修正/均值补入/措辞限定）
- 错误台账：`.investigations/panic-505/panic-errors.md`
