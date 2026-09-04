# 翻默认后 block_probe 存档口径 Full 回归裁决 — 260904-03

---
status: confirmed（260904-03 用户拍板；judge review-260904-03-p2full.md PASS-with-CONCERN 在前，MUST-1/SHOULD-2 已清偿）
verification: full（WGB2 存档口径逐位对比）
date: 260904-03（实际 2026-09-04 12:48-12:57，git/日志时间戳锚）
supersedes: 无（闭合 off-scan-cornerfix-verdict-260903-13.md:31-33 遗留，非推翻既有 confirmed 结论）
---

## 核心结论

1. **翻默认后生产 dll 存档口径 Full 回归无回归**：
   - 生产路径 = Rust cdylib（`resources native/worldgen.dll`，sha256 `EC4A9AED…C8FC3`，晚于全部生产改动），est 默认开（WG_EST_SHARED/WG_EST_L2 已翻默认，260903-13 confirmed）。
   - default 臂（Run2，cppReplace，est 默认开）vs vanilla 参照（Run1b，无 cppReplace）：**16/16 chunk 共 13328/1572864 cell 差，99.1526% 一致**（解析感知逐位对比，`.tmp/p2full/cmp_full2.py`，输出 `cmd-output/cmp_full2-260904-03.txt`）。
   - 该残差水平与 default==off（Run2 与 Run3 SHA256 完全一致 `2FF71249E9C5CF9745CAC95BC27DC011D53F91C5C454C042E2DF9C9B23F210BC`，`cmd-output/arm-compare-260904-03.txt`）以及 260903-13 四臂零语义差结论（`f2b1a3932c6e589e`）**自洽**：default 臂输出与优化语义无关，翻默认未引入任何新语义差。
2. **既有残差（99.15%，aquifer stone↔water 互换族）是独立新观察，不阻塞本回归**：
   - Run2 == Run3 逐位一致 ⇒ 该残差在 est off 臂同样存在 ⇒ 与 est 优化/翻默认**无关**（est 优化的语义无关性再次独立复证）。
   - 详见 07 篇「存档口径残差模式化（260904-03）」小节 + `.investigations/lossless-accel/knowledge-drafts/260904-03/draft-aquifer-residual.md`——登记为待查项，非本回归阻塞项。
3. **seed/世界三查核对通过**：seed 8576294172403134396 双侧日志核对一致；参照文件四要素（magic WGB2 / seed / size=4 / origin=(200,200) / min_y=-64 / height=384）双侧一致，trailing bytes=0。

## 验证可比性声明（Anchorlaw §9.7 三要素）

- **载体**：WGB2 FULL 存档口径 block_probe 导出（逐位 cell 对比），4×4 单区域（seed 8576 @ chunk(200,200)）。
- **覆盖面**：16 chunk × 98304 cell = 1572864 cell/chunk 区域，差 13328 cell（99.1526% 一致）。
- **历史口径可比性**：与 260903-13 est 角列口径（est dump 4 角 × 64 chunk）**不同载体，不可直接比数值**；与 260903-14 存档口径 3 采样（均值 99.0107%）同族载体（存档口径 FULL）但采样区域/次数不同，仅可作「同族残差带」参照，不可直接数值对齐。

## 已知局限（如实声明）

1. **Run3 的 A/B 判别力受限（死参数风险，workflow-patterns #20/#32 族）**：gradle daemon 复用了 run1/run2 环境，run3 客户端 env（WG_EST_SHARED=0/WG_EST_L2=0）可能未透传进 daemon ⇒ 「default==off 同 hash」**不能独立证明 env 反转生效**，只能作为与四臂 hash 结论自洽的**旁证**。env 门控生效的主要证据仍是 260903-13 `estopt-ab-defaultflip-260903-13.txt`（专用探针验证 default 臂 shared=true/l2=true）。本回归的核心结论（无回归）不依赖 Run3 生效——Run2 vs Run1b 的对比已闭合。
2. **Run1 产物覆盖流程瑕疵**：Run1 与 Run2 输出同名文件，Run1 首跑产物被 Run2 覆盖后重跑（run1b 重采 vanilla 参照）。无数据混入（run1b 为独立重跑 + 四要素核对通过），流程瑕疵已登记错误台账 LL12。
3. **运行日志既有噪声**：两 run 均见 `[AQF-J] failed NPE`（Cannot invoke Object.getClass() because o is null）每 chunk 重复，**两臂同现** ⇒ 非 cppReplace 引入（Java 探针侧既有噪声）；不构成差异来源（本轮判据为逐位文件对比，非 AQF-J 日志判定）。

## 证据链

- 运行日志：`cmd-output/run1b-vanilla-full-260904-03.log` / `run2-cppreplace-full-260904-03.log` / `run3-estoff-full-260904-03.log`
- 对比输出：`cmd-output/cmp_full2-260904-03.txt`（解析感知逐位）+ `cmd-output/arm-compare-260904-03.txt`（臂间 SHA256）+ `cmd-output/cmp_full-misaligned-260904-03.txt`（首版 24 字节 header 误读，留档）
- 对比脚本：`.tmp/p2full/cmp_full2.py`（WGB2 布局：每 chunk wx/wz int32 + 16×16×384 int16-BE block id + 256 UTF biome）
- 前置 confirmed：off-scan-cornerfix-verdict-260903-13.md（四臂 hash 统一 + 翻默认实施）
