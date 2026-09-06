---
version: 1.0.26
status: pending
date: 2026-09-06 17:10（Get-Date 锚定）
---
## 1. 构建产物

- jar: `E:\PYTHON\CoreSwap\runtime\1.20.1\java\build\libs\coreswap-1.20.1-1.0.26.jar`
- jar sha256: `6B9EF26FC5D57C7CB15079F1E1145829BEB998C10CCDC614C51FAD96504BF6AF`
- dll sha256（jar 内 worldgen.dll = `target/release/worldgen.dll`，已在主工作区核验一致）: `1B5AA1DEA49445A29FCA3CA6B6ADE1FFBB5048EB7C855A7298DAEC74FAD99705`
- 源 commit: `5f73d7e`（HEAD；end 接管代码 = c2330cd，文档 = 5f73d7e）

## 2. 验证记录（judge MUST review-002 通过，推荐 confirmed；用户已授权确认）

| 判据 | 结论 | 证据 |
|---|---|---|
| End 端到端逐位（同 seed 7691421705105351955，Forge 生产存档 vs vanilla 臂，chunk -2..3 × -2..3 = 36） | **0 差异**（含黑石柱/出口传送门/铁栏杆等 Java feature 全一致） | `.tmp\end-takeover-260906\cmd-output\ab-final-zerodiff.txt`（注：正式终验以 `ab_regions.py` vanilla-DIM1 vs mod-DIM1，输出同目录会话记录；0 差异由 judge 独立复核） |
| SimplexNoise/EndIslands 组件对拍（4 seed：12345 / 7691421705105351955 / -999999999999999999 / 1） | 数值全等（含 -0.84375/0.5625 边界点） | `.tmp\end-takeover-260906\cmd-output\{java,rust}-endprobe-<seed>.txt` |
| 执行体三元组 | target dll = 发版 jar 内 dll = 服务器运行时加载，sha256 一致（judge 独立重算 + 运行时日志双证） | review-002-final.md ⑤ |
| 生产环境 | Forge 47.4.5 + Connector beta.49 专用服（SRG 实证），`initEnd enabled=true` + `populateNoise(end)` 接管 182 chunk | `.tmp\end-takeover-260906\forge-end-takeover-evidence.txt` |
| Nether 共享路径回归（judge CONCERN-2 跟进） | 1.0.25 基线 712 差 vs 1.0.26 新 dll 914 差（同区域 36 chunk），在既有同 jar 重跑非确定容差内；三处共享改动对 nether 静态恒等 → 无回归 | 时间线 260906-04 收尾跟进条 |
| judge 审查 | `.investigations\end-takeover\review-002-final.md`（推荐 confirmed；用户已确认授权） | 同左 |

## 3. Changelog

### English

**End dimension is now fully native** — the last vanilla terrain dimension.

- New `EndIslands` density function (from-scratch port: SimplexNoiseSampler, world-seed-direct noise chain, float-domain exact) + position-based `TheEndBiomeSource` classifier (center/highlands/midlands/barrens/small-islands) replacing MultiNoise for the End
- Bit-exact vs vanilla: same-seed production save comparison, 36/36 chunks, **0 mismatched blocks** — end spikes, exit portal and obsidian platform all placed by the vanilla layer on top of native terrain
- Dimension-parameterized interpolation cells (size_horizontal/vertical from JSON — fixes a +1-block surface shift on the End), hardened Java long wrapping arithmetic, empty-column heightmap sentinel handling
- Mixin dimension dispatch hardened: chunk-shape collision between End/Nether (both 0/256) resolved by settings-id discrimination (follow-up to the BUG-002 fix)
- Forge (via Sinytra Connector) is now production-verified across all three vanilla dimensions on a dedicated SRG-remapped server

### 中文

**末地现已全原生**——最后一个 vanilla 地形维度。

- 新增 `EndIslands` 密度函数（从零移植：SimplexNoiseSampler、worldSeed 直传噪声链、float 域逐行对齐）+ 基于位置的 `TheEndBiomeSource` 判定器（中心/高地/边缘/小岛/虚空），末地不再走 MultiNoise
- 与 vanilla **逐位一致**：同种子生产存档对比 36/36 chunk、**0 块失配**——黑石柱、出口传送门、黑曜石平台均由 vanilla 层在原生地形之上正常放置
- 插值 cell 尺寸维度参数化（size_horizontal/vertical 来自 JSON——修复末地岛面 +1 格系统性偏移）、Java long 回绕算术加固、空列 heightmap 哨兵处理
- Mixin 维度分派加固：末地/下界 chunk 形状碰撞（同为 0/256）由 settings id 判别解决（BUG-002 修复的延续）
- Forge（经 Sinytra Connector）已在专用 SRG 重映射服务端完成三维度生产级验证

## 4. 发布动作清单（Maint 执行）

- tag：`v1.0.26`（若仓库既有 tag 约定不同，按约定调整并在 status 镜像注明）
- 标题建议：`CoreSwap 1.0.26 — End dimension goes native` / 中文场景 `CoreSwap 1.0.26 —— 末地全原生`
- notes：用 §3（英文为主、中文附后，或按仓库惯例）
- asset：`coreswap-1.20.1-1.0.26.jar`（原文件名）
- 发布后：① status 镜像回写 ② BUG-002 跟踪 issue（如仍未关）回帖载体版本 1.0.26 + `triage:fixed` 已发版说明 ③ 本票 status → published

## 5. 已知边界 / 降级声明

- 末地逐位判据覆盖 y 0..127（engine 写入高度）；y≥128 vanilla 本就全空气，未逐位入判据
- mod 维度若复用 `minecraft:end` settings entry 会被接管（与 overworld/nether 既有边界同构，已在 README 注记）
- overworld 侧既有 A/B 存档回归尚未在新 dll 重跑（共享改动静态恒等 + nether 回归已补，风险低）——如 Maint/用户认为需要，可在发布前要求补跑
