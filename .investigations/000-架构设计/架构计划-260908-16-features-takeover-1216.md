---
编号: 000
任务: 1.21.6 Rust features 阶段接管实装（生产路径从「Java vanilla 装饰器在 Rust 地形上跑」切到 Rust features 执行）
任务类型: swe（Rust 实装 + 验证）+ 前置 re-code 核对
模式档位: 重量
状态: 待批准
日期锚: 260908-16（Get-Date 2026-09-08 23:37；git 锚 288479d）
---

## 1. 全局视图（目标/范围/排除项）

**目标**：让 1.21.6 生产路径的 features 阶段由 Rust worldgen 执行（关闭/改写 jni_bridge 的 SKIP_FEATURES 通路），支持面 = 1.21.6 数据包全部 configured feature 类型的 Rust 侧支持，行为对齐判据沿发现 #67「冻结顺序生成」。

**背景（继承自 NEXT_SESSION 260908-15，⚠️ 均为未验证交接结论，Phase 0.5 必须廉价验证后才可续推）**：
- 1.21.6 生产路径 features 阶段 = Java vanilla 在 Rust 地形上执行（jni_bridge flag SKIP_FEATURES）；
- B5 (fallen_tree) / B6 (place_on_ground + attached_to_logs) Rust 侧已支持（confirmed 260908-15）；
- 1.21.6 移植主线 P1-P7 已完成（对齐率 99.9993%，260908-10 verdict）；
- features 无单一 Java 基线（发现 #67，run 级非确定，对齐判据 = 冻结顺序生成）。

**范围**：
- 1.21.6 版本薄壳 + worldgen-core 的 features 执行通路接线（stageMask/flag 语义核对与改写）；
- 1.21.6 feature 类型支持缺口盘点与补齐（含 #87 家族：枚举新常量 catch-all 排查）；
- 验证载体搭建：Java-vs-Rust feature 层冻结序对拍（confirmed 前置，260908-15 verdict §4 声明）。

**明确不做**：
- 1.20.1 任何改动（其 features 接管已稳定，仅作参照实现只读）；
- carver 阶段（独立课题，不在本计划）；
- structures 阶段变更（保持现状）；
- Chunky 双臂全量对齐率验收（260908-15 已论证 features 层无单一 Java 基线，全量对齐率不是本阶段验收判据；Chunky 仅作可选的宏观 sanity 手段）。

## 2. 角色分配（scout/worker/judge × 子任务）

| Phase | 角色 | 内容 |
|---|---|---|
| 0.5 | 主会话 | 交接结论廉价独立验证（见 §3） |
| 1 | **scout**（subagent，recode-scout 手册） | ① 1.20.1 features 接管参照实现地图（feature_loader/生成入口/stageMask 接线/数据加载面）；② 1.21.6 数据包 feature 类型/装饰器集合 diff 清单；③ jni_bridge SKIP_FEATURES 通路现状 |
| 2 | **worker**（subagent） | 支持缺口补齐的代码交付（patch 落盘 → 主会话应用+编译）|
| 2 | 主会话 | 收敛型接线改动 + 冻结序对拍载体的执行（subagent 无 shell，运行时验证主会话执行，原始输出回传解读）|
| 2.5 | 主会话 | 验证执行（冻结序对拍 + 冒烟）|
| 3 | **judge**（subagent） | MUST：candidate 授予前 + 收尾交付（三源核对）|
| 末 | **knowledge**（subagent） | 结论性落盘草稿 → 主会话应用 |

## 3. 任务拆解 & 依赖图

**Phase 0.5 — 交接结论廉价验证（每项成本 ≤ 一轮，全部通过才进 Phase 1）**
1. 核 SKIP_FEATURES 通路现状：读 jni_bridge flag/stageMask 源码，确认「1.21.6 生产 features 归 Java」这一结论在当前 HEAD 仍成立；
2. 核 B5/B6 支持在位：`cargo build --offline -p worldgen --release`（1.21.6 薄壳）绿 + feature_loader 分支冒烟（bin-diag b5b6_smoke 复跑一轮，先核产物时间戳——发现 #16）；
3. 核 1.21.6 薄壳 features 数据面在位：data 目录 configured feature JSON 可解析（复用既有加载路径）。

**Phase 1 — scout 勘探（subagent，只读，产物 .investigations/）**
- scout-a：1.20.1 features 接管参照实现地图（入口、feature 类型分发、装饰器、RNG 消费序、数据加载）；
- scout-b：1.21.6 数据包 feature 类型 × 装饰器集合 vs worldgen-core 已支持集合的 diff 清单（#87 判据：Java 枚举类逐版本 diff 常量集合；catch-all 分支全列）；
- scout-c：jni_bridge 通路 + stageMask 语义图（接管切换点在哪、有哪些副作用）。

**Phase 2 — 实装（依赖 Phase 1 产物）**
1. 差距补齐（worker 交付 patch → 主会话应用编译；逐类型小步提交）；
2. 接管切换改动（stageMask/flag 语义修改，收敛型，主会话直接做）；
3. debug profile 全量绿（发现 #18：wrapping 审计器先跑）+ workspace 全量 build（#27 判据：-p 单包绿 ≠ 全量绿）。

**Phase 2.5 — 验证**
- 主载体：**Java-vs-Rust feature 层冻结序对拍**（发现 #67 判据：冻结顺序生成；固定 seed + 固定 chunk 集 + 冻结 RNG 序，dump 对拍）；
- 宏观 sanity（可选）：Chunky 双臂单 region 剖面抽查（#26 载体，剔除植被对比注意 B5/B6 现已 Rust 侧支持）；
- 生产观测核验（执行域先核再验，260908-15 纪律）：确认接管后生产路径确实走 Rust features（行为化日志 + hash 哨兵，#81 判据）。

**Phase 3 — judge + 收尾**
- judge MUST（candidate 授予前）：审查产物快照 + git diff + 验证记录三源；
- 用户 confirmed 拍板 → index 转正 → 知识库落盘。

## 4. 并行执行计划

- 第一波：Phase 0.5 三项验证（主会话串行，廉价）；
- 第二波：scout-a/b/c **三个 subagent 并行**（互不依赖）；
- 第三波：实装按 scout diff 清单分批（每批 worker patch + 主会话编译验证）；
- 第四波：冻结序对拍（依赖实装完成）→ judge。

## 5. 人工决策 HOOK 点

1. 架构批准（本文件，当前）；
2. Phase 1 勘探产物汇报后：实装批次划分与优先级拍板；
3. 接管切换方案确认（stageMask 改法若与 1.20.1 语义有分歧 → 回 Phase 0）；
4. judge 通过后：confirmed 授予（用户专属）。

## 6. 风险 & 回退

- **风险 1**：features 接管 = 高风险生产语义变更（Java 装饰器在 Rust 地形上跑的既有行为被替换）→ 回退 = stageMask 恢复 SKIP_FEATURES（单 flag 翻转，保持回滚通路）；引用面清理 grep（#77 判据）。
- **风险 2**：RNG 消费序跨版本漂移（1.20.1→1.21.6 feature 类型序/装饰器序变化）→ 冻结序对拍前置逐类型核对，#87 家族排查（新枚举常量 catch-all）。
- **风险 3**：features 无单一 Java 基线（#67）→ 验收判据用冻结序对拍而非存档对齐率，§9.7 声明口径。
- **风险 4**：交接结论失真（M14 家族）→ Phase 0.5 全部廉价验证前置，任一失败即回 Phase 0 修订本计划。

## 7. judge 步骤预置

- 节点: Phase 2 实装完成、candidate 授予前 | 级别: **MUST** | 审查对象: .artifacts 快照 + git diff + 冻结序对拍记录；
- 节点: 收尾交付 | 级别: **MUST** | 三源核对（快照/git HEAD+worktree/验证记录）；
- 节点: scout diff 清单定稿 | 级别: SHOULD | 审查对象: 清单 vs 数据包一手核对。

## 8. fan-out 步骤预置

- 触发条件预设：冻结序对拍出现成簇偏差 → 若 ≥2 互斥机制候选（RNG 序漂移 / 类型实现缺陷 / 数据解析缺键）→ **MUST fan-out**（.bN 并行 worker），禁止主会话自推（#68 两通道未分解禁止单通道归因）。

## 9. 知识库更新（每 Phase 末尾）

- 结论性 docs/discovered 写入：**subagent 产出草稿**（core.worker；prompt 含「先读 E:\PYTHON\CoreSwap\knowledge\SUBAGENT-KNOWLEDGE-GUIDE.md」）+ 主会话应用 + INDEX 同步；
- 预置产出：接管切换语义（若出新模式）、冻结序对拍载体实践（#67 落地经验）、10 时间线追加。

## 10. 子角色介入点（全部预置，执行不临时起意）

- **scout**: Phase 1 三路并行（scout-a 参照地图 / scout-b diff 清单 / scout-c jni 通路）| 触发: 接管实装=机制摸底 MUST 勘探 | 产物: .investigations/mc-1216-features-takeover/ 管线地图；
- **worker**: Phase 2 逐批次代码交付（patch + 未编译声明 + 静态自检清单四条）| 产物: .artifacts draft + patch 文件；
- **fan-out**: 节点 = 冻结序对拍成簇偏差 ≥2 互斥候选 | .bN 并行 | 禁止主会话自推；
- **judge**: 节点 = candidate 前 MUST + 收尾 MUST（三源）+ scout 清单 SHOULD；
- **knowledge**: 每 Phase 末尾结论性落盘 | subagent 产出草稿 + 主会话应用验证。
