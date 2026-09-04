---
编号: 000
任务: Vulkan GPU 算子重立项——GPU density 管线从 per-chunk 小批量同步 dispatch（~369ms/chunk，慢 CPU 4.8×）改造为算子级/批量化形态，目标 e2e 显著优于 CPU 基线 27.69ms/chunk
任务类型: 性能重构（swe 域，C++ + Rust 通道）
模式档位: 重量
状态: 用户拍板方向变更（260903-16 Phase 0.5 数据判决）——GPU 批量化前提不成立，(a)/(c) 出局、(b) 唯一活口待重估；主攻转「shared 臂开销」课题（架构计划-260903-16-shared-arm-cost.md）。本文件保留为 GPU 课题 Phase 0.5 判决记录。
session: 260903-16
---

## 1. 全局视图

- **目标**：GPU density 管线重新具备对 CPU 的竞争力。验收判据（先行）：
  - **性能**：e2e 256-chunk 大样本 median 显著 < 27.69ms/chunk（CPU est-L2 基线；沿用端到端性能对比铁律——Java 预热 + 大样本 + 删 run\world + 中位数口径 + §9.7 三要素声明）。
  - **正确性**：GPU 输出与 CPU 管线 hash/逐位在既有 GPU 残差口径内一致（四臂 hash 哨兵沿用，口径同行声明）。
  - **零退化**：CPU 管线路径不动摇（GPU 为 opt-in 通道），四臂 hash 零回归。
- **范围**：C++ GpuDensityEngine（dispatch 粒度/合批/异步 readback/算子切分重铺）+ Rust gpu_ffi 通道 + spv 多产物原子更新。
- **明确不做**：不改 CPU density 语义；不动 carver/feature/surface 阶段；非 density 阶段 GPU 化不在本轮；多世界维度参数化不在本轮。

## 2. 交接结论继承前廉价验证（纪律强制，开工第一步）

NEXT_SESSION 的关键数字属「机制方向类」结论，不得当公理直接续推：
- V1：复跑 GPU per-chunk bench（369ms/chunk 是否仍成立）+ CPU est-L2 基线（27.7ms 是否稳定）——一轮 bench 即可。
- V2：核对 260903-12 gpu-batch-merge 调查产物在 .investigations/ 的实际内容与结论边界。

## 3. 角色分配 & 任务拆解

| Phase | 任务 | 角色 | 产物 |
|---|---|---|---|
| 0 | 本架构计划 | 主会话 | 本文件，用户批准 |
| 0.5 | V1/V2 廉价验证 | 主会话（命令执行+数据落盘），解读交 worker | .investigations/vulkan-gpu-operator/cmd-output/ + baseline 数字 |
| 1 | scout 勘探：现状盘点——gpu_ffi.rs 通道结构、GpuDensityEngine dispatch 粒度、369ms 分解（dispatch/readback/同步各占比）、spv 管线、批合并调查产物回收 | scout subagent（只读，divergent） | .investigations/vulkan-gpu-operator/scout-管线地图.md |
| 2 | 方案分叉：合批多 chunk dispatch+异步 readback vs density 全树单 dispatch 固定网格 vs 混合——≥2 互斥候选 MUST fan-out | fan-out worker subagents ×N，各产 .bN 候选 | .artifacts/vulkan-gpu-operator/*.bN + index.yaml |
| 3 | 收敛实现：胜出方案 C++/Rust 改码 → build.ps1 构建 → bench + hash 对比 | 主会话（swe 收敛闭环） | 代码 + cmd-output 验证数据 |
| 3.5 | 验证：Full 层 hash 一致 + 大样本 e2e bench + 零退化四臂哨兵 | 主会话 | verification 记录 |
| 4 | judge 审查 → 用户拍板 confirmed | judge subagent | review-*.md |
| 5 | 知识库更新（错误台账五段式 + discovered 条目） | knowledge subagent 草稿 → 主会话应用 | errors 台账 + workflow/build-tooling 条目 |

## 4. 并行执行计划

- 第一波：Phase 0.5（bench 复跑）与 Phase 1 scout 可并行（scout 只读不依赖 bench）。
- 第二波：Phase 2 fan-out（依赖 scout 地图 + 基线数字）。
- 第三波：收敛实现串行（主会话）。

## 5. 人工决策 HOOK 点

1. 本架构计划批准（现在）。
2. fan-out 候选对比后方案拍板（Phase 2→3 之间）。
3. 收敛实现后如 e2e 不达标 → 重大方向变更回 Phase 0（暂停）。
4. confirmed 授予（最后）。

## 6. 风险 & 回退

- **R1 spv 多产物部分更新**（build-tooling #12）：生成器多产物必须整体原子更新 + 已知值哨兵点验。
- **R2 create ~75s pipeline 编译**：缓存语义已有，批量化改动勿破坏缓存 key。
- **R3 探针/bin-diag 陈旧产物假阴性**（#16）：探针用前 rustc 单编/核 LastWriteTime。
- **R4 测量污染**：GPU 计时探针并发污染前科——只信无探针整批 wall + 调用计数；bench 交错（#24）。
- **回退**：GPU 通道 opt-in，任何失败不影响 CPU 管线 shipped 状态（1.0.23 已发布）；每步 bench 前四臂 hash 哨兵。

## 7. judge 步骤预置

- 节点: fan-out 候选对比 | SHOULD | 审查对象: .bN 产物 + scout 地图
- 节点: 收敛实现验证通过后（candidate 授予前） | MUST | 审查对象: .artifacts 快照 + git diff + verification 记录（三源）
- 节点: 收尾交付 | MUST | 三源核对

## 8. fan-out 步骤预置

- 节点: Phase 2 方案选型 | 候选: (a) 多 chunk 合批 dispatch + 异步 readback；(b) density 全树单 dispatch 固定网格；(c) (a)(b) 混合/其他 scout 发现的新形态 | .bN 并行 worker | 禁止主会话自推

## 9. 知识库更新

- 错误台账：.investigations/vulkan-gpu-operator/vulkan-gpu-errors.md（五段式，遇到即记）
- 结论性 docs/discovered 条目：subagent 产出草稿（prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md 要求）→ 主会话应用 + INDEX 同步
- 时间线过程记录 → docs 10 篇（subagent 草稿）

## 10. 子角色介入点（全部预置）

- scout: Phase 1 | 触发: GPU 管线现状机制盘点（强制前置） | 产物: scout-管线地图
- worker: Phase 0.5 数据解读 + Phase 2 .bN 候选 | 产物: .artifacts draft
- fan-out: Phase 2 方案分叉（见 §8） | MUST
- judge: §7 三处 | MUST/SHOULD 如标注
- knowledge: Phase 5 | subagent 产出 + 主会话应用验证
