# core.judge 审查意见 v2 — rust-density-builder 对齐（升级证据后）

> 审查对象：`E:\PYTHON\CoreSwap\.artifacts\rust-density-builder\rust-density-builder-alignment.md`（candidate）
> 日期：2026-08-26 | 审查人：core.judge（subagent，只出意见，不改 status）| v2 = 依用户提供的升级证据更新，取代 v1（review-001.md）
> 本次升级：验证扩展到 **16 个 overworld 密度函数（10 顶层 + 6 caves/*）× 10 采样点 = 160 采样值 + 全部 min/max**；并抓修 3 个对齐 bug。

## 结论：通过（推荐维持 candidate）——原 P1 已获实质升级，仅剩 P2/P3 残余项

v1 的 P1-①（取样过稀）与 P1-②（仅 Rust-vs-C++）已在升级证据下显著缓解。本判定建立在我**亲自复核**以下事实之上，非仅采信结论：

### 复核一：数值差分 = 0（新证据确认）
- 用 `[System.IO.File]::ReadAllLines` 单次读现行文件，逐行/逐列正则 diff：**16 条函数行、192 个数值全部一致，MISMATCH LINES = 0**。
  - `rust_out.txt` = 17 行（16 函数 + 1 行 `sloped_cheese min/max` summary，非数值差），`cpp_out.txt` = 16 行。
  - rust 数值总数 = 194，cpp = 192，差值恰为末尾 summary 一行的 2 个值——与「仅 summary 多一行」一致。
  - 每函数行现为 **10 个采样点**（如 base_3d_noise：(0,0,0)(4,64,4)(8,128,8)(40,192,40)(100,-64,-40)(-64,64,-64)(128,288,128)(200,0,200)(16,-112,16)(72,320,72)）+ min/max。
- 上轮 v1 观测的 `caves/noodle min` 瞬态分叉（-0.0083 vs -0.1583）在本快照已归一双稳 -0.1583（含在 diff=0 内）。

### 复核二：3 个对齐 bug 修复与 C++ 参照逐行吻合（静态 + 实证）
| Bug | C++ 参照 | Rust 现态 | 判 |
|---|---|---|---|
| ① ABS/SQUARE mn | `density.h` L184-186：`if(op==ABS\|\|SQUARE){mn=max(0,imin); mx=max(applyUnary(imin),applyUnary(imax));}` | `density_builder.rs` L214-216：`if op==Abs\|\|Square { mn=imin.max(0.0); mx=apply_unary(op,imin).max(apply_unary(op,imax)); }` | ✓ |
| ② minecraft:clamp 读 "input" 非 "argument" | `density_builder.h` L92：`arg("input")` | `density_builder.rs` L245：`self.arg(v,"input")`（v1 读到过旧版 `"argument"`，现为 `"input"`） | ✓ |
| ③ InterpolatedDF min/max 委托 arg 非 -arg.max | `density.h` L560-561：`minValue(){return arg->minValue();} maxValue(){return arg->maxValue();}`（InterpolatedDF L482） | `density.rs` L447/Min `id.arg.min_value()`、L475/Max `id.arg.max_value()` | ✓ |
- 佐证：`ridges_folded min=-14.1429 max=1.0000`、`factor min=0.625 max=6.300` 等 min/max 两端在 16 行全对齐，证实 ①/③ 的 range 折叠路径生效。

### 复核三：factor=3.95000005 判断复核（仍认同为非 bug，但连带覆盖缺陷仍存）
- `factor.json` 外层 spline 首点 `location=-0.19 value=3.95`（derivative 0，左端平段）；10 点 continents 全在 [-0.738, -0.466]，**全部 < -0.19** → factor 恒取 3.95。此判断正确（非 bug）。
- **但**这暴露：本次 10 点**仍全部落在低大陆度区**，factor 的嵌套 erosion/ridges/ridges_folded spline（location -0.19..0.06）及其分支**未被采样**。见 P2-a。

---

## P1 —— 无（升级后原 P1 不再成立）

- 原 P1-①（取样过稀）：10 点 × 16 函数 + 32 个 min/max（含 SplineDF `nodeMin/nodeMax` L1073-1074 对全 spline 树的静态 range 走查）= 对 16 函数的结构/数值覆盖已达合理强度。「逐位对齐」措辞现可支撑为「在 16 函数的 10 采样点 + 全 min/max 上逐位一致」。**不再阻 confirmed**，仅建议补高大陆度点（P2-a）以闭合 factor 空窗。
- 原 P1-②（仅 Rust-vs-C++）：此为本项目目标本身（AGENTS.md「Rust 全量重写 CoreSwap worldgen」= 对齐 C++ 参照；C++ 的 vanilla 正确性另有 block_probe 链路验证）。故 Rust==C++ 是**正确的中间目标**，非缺陷。**降级为 P2-b 范围注明**（见下）。

---

## P2（建议处理，不阻 candidate）

### P2-a  factor / 高大陆度 spline 分支未被采样（残余覆盖空窗）
- **问题**：10 点 continents 全 < -0.19，factor（coordinate=continents）10 点恒在左端平段 3.95；侵蚀/ridges/ridges_folded 嵌套 spline（location -0.19..0.06）的采样行为未被任何点触发。
- **证据/理由**：10 点 continents = -0.6527/-0.6399/-0.6274/-0.4757/-0.4658/-0.7378/-0.5361/-0.5109/-0.6038/-0.4866，全部 < -0.19。
- **建议处置**：追加至少 1-2 个高大陆度采样点（如 continents > 0 的坐标，例如 (400,80,400) 或按 continents 值反推），使 factor 的嵌套 spline / ridges_folded 分支被采样，闭合空窗；或在此点意外分叉时再评估。长期：按 spline knot 边界做确定性扫点。

### P2-b  对齐范围注明（Rust-vs-C++ 非 Rust-vs-vanilla）
- **问题**：本验证对照物 = C++ `density_builder.h` 复现，不直接对照 vanilla；C++ 自身 vanilla 正确性依赖外部 block_probe 链路。
- **证据/理由**：产物「域/边界」已声明未做 vanilla 端到端/逐块对齐。
- **建议处置**：在结论显式注明「对齐基准 = C++ buildNode（C++ 已另由 block_probe 独立对 vanilla 验证）」，vanilla 端到端属后续阶段。此为范围注明而非缺陷。

### P2-c  WorldgenRust 全树 untracked（git 基线缺失）
- **问题**：`git ls-files WorldgenRust` 为空，`git status` 显示 `?? WorldgenRust/`——无 HEAD 基线，v1 所述「3 编译错误修复」「&self→&mut self」「分派不全→补全」均**无 diff 可查**，只能审整文件终态（我已直接审终态，与产物一致）。
- **证据/理由**：git 复核结果同 v1；`.artifacts/rust-density-builder` 与重建的 `.investigations/rust-density-builder` 亦 untracked。
- **建议处置**：提交 WorldgenRust 基线（或整树）入 git，使 diff 维度可查；符合提交纪律。否则声明「全新/untracked 树，本核对为整文件终态审」。

### P2-d  evidence 非冻结态（评审期间被再生成）
- **问题**：复核中 read 工具首见 10 行版本、后见 17 行版本；v1 观测 clamp 旧版（`"argument"`）→ 现新版（`"input"`）；两输出文件 LastWrite 不同（cpp 16:22:29.541 vs rust 16:22:59.674）。均为「文件被再生成」证据。
- **证据/理由**：证文件是运行产物，非不可变快照；「diff=0」只对当前字节串成立。
- **建议处置**：把 rust_out.txt / cpp_out.txt + 生成命令（`cargo run --bin overworld_probe` / `rust_ref_check.exe`）+ 探针源 + 二进制哈希复制固化到 `.investigations/rust-density-builder/cmd-output/` + `regression-record.md`；确认无后台再改动（我 job_list 无任务，但可能属发起方构建循环）。

### P2-e  noise_params 硬编码表 vs noise_params.json
- **问题**：Rust `build_noise_params()` 与 C++ `buildNoiseParams()` 均为硬编码全表，共享键逐值一致，但都不读 `minecraft:noise_params.json`.
- **建议处置**：将该硬编码表与真实 `noise_params.json` 逐键核对（或下一步改读文件）；在产物边界注明「当前用硬编码表，非 vanilla 权威」。

---

## P3（文档/细节）

- P3-a 产物只声称/描述 10 个 overworld，现行探针已覆盖 16（含 6 caves/*）且 diff=0——文档滞后于探针，建议补记 caves/* 已对齐。
- P3-b 产物未声明验证分层（Full/Partial/Degraded）与执行者/环境；Rust 侧 anchor 标注缺属预期（v0.17 Rust 协议外）。建议在「验证方式」注明分层。
- P3-c 审核期间文件不稳定导致我 v1 读到过 clamp 旧版 / 10 行快照——固证时须锁源，避免审到半成品。

---

## 审查意见（只意见，不改 status）

- **推荐状态**：维持 candidate；本次升级已使证据充分支持 candidate，可作为「Rust buildNode == C++ buildNode 对齐里程碑」交用户裁决是否 confirmed。
- **confirmed 前建议（不再阻断 candidate）**：① P2-a 补高大陆度点（闭合 factor 空窗，可选但更稳）；② P2-c WorldgenRust 入 git；③ P2-d 固化证文件快照+命令/哈希日志；④ P2-e 噪声参数表对文件。
- **我认同**：factor=3.95 为左端平段（非 bug）；mn/mx 对齐充分（`ridges_folded min=-14.1429 max=1.0000` 两端一致，证 ① 的 range 修复生效）；3 个 bug 修复均与 C++ 逐行吻合且被 diff=0 实证。
- **待用户拍板**：对齐基准为 C++ buildNode（项目目标）；vanilla 端到端属后续。

> 说明：本次审查未改动任何状态；`.artifacts/index.yaml` 的 `re-code:rust-density-builder:alignment` 仍为 candidate。v1（review-001.md）保留作证据链审计。
