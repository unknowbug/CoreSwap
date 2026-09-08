# 架构计划-260908-14-p6-spawn-barrier

---
编号: 260908-14
任务: P6 SPAWN barrier 差异探针——量化/归因 1.21.6 移植 spawn 区 barrier 放置差异（C1 豁免清单 B3）
任务类型: 验证（运行时载体 + 静态源码核对）
模式档位: 轻量
状态: 已批准（用户拍板 260908-14，实际 2026-09-08 21:37 Get-Date 锚）
---

## 范围（含明确不做什么）

**做**：
1. 廉价独立验证前置：B3「spawn 区 barrier 差异」为 260908-09 挂账推断——先溯源首次取证描述（现象/坐标/载体），确认「差异是什么」有据。
2. 机制摸底（静态）：Java 1.21.6 spawn 区 barrier 放置语义 vs Rust 接管域边界（谁放哪些 barrier）。
3. 运行时探针（主体）：runServer + **post-Done forceload**（#80：spawn 区在 SERVER_STARTED 前已 vanilla 生成）双臂对拍（Rust 接管 vs 1.21.6 vanilla），spawn 区 barrier 块差量化（Chunky 双臂载体 #26 或 tp+F3 单点 sanity）。
4. 裁决：差异实存 → 归因（≥2 互斥候选 → fan-out）；差异消失 → B3 以取代记录关账。

**不做**：F2 spread-replay、I1 下钻、任何引擎/数据改动（修复另立项）、P5 重开。

## 任务拆解

| # | 子任务 | 执行者 | 产物 |
|---|--------|--------|------|
| W1 | B3 首次取证溯源 + Java/Rust barrier 机制摸底 | scout subagent（只读） | .investigations/mc-1216-port-260908-14/ 摸底文档 |
| W2 | 运行时双臂采集 | 主会话（命令执行） | cmd-output 原始输出落盘 |
| W3 | 数据解读 + 归因裁决 | 收敛主会话 / ≥2 互斥 fan-out | .artifacts/mc-1216-port/p6-verdict-260908-14.md（draft） |
| W4 | C1 清单 B3 条目更新 | subagent 草稿 + 主会话应用 | c1-exemption-list.md 追加 |

## 验证方式

- seed 三查（server.properties 备份 / 删 run\world / 输出 seed 核对）+ 参数口径三查（#28）+ 执行体三元组核验（#36）。
- §9.7 载体声明：Chunky 区域级 vs 单点探针口径分开标注。

## 子角色介入点（预置，执行只核对不补排）

- scout: W1 机制未明摸底 MUST（subagent 只读勘探，禁止主会话直接跳单点定位）
- worker: W3 发散解读 subagent；W1 解读按 core.worker
- fan-out: 差异归因 ≥2 互斥候选 MUST 并行 .bN（候选方向预置：①Rust surface rule/地形未覆盖 spawn 专用 barrier ②Java 调度层 spawn 路径差异 ③ vanilla 生成时序污染——以 W1 摸底为准修正）
- judge: candidate 授予前 MUST（三源核对：artifacts 快照 + git diff + 验证记录）
- knowledge: W4 结论性清单更新 MUST subagent 草稿；价值门评估先做

## 风险与回退

- spawn chunk 在接管生效前已生成（#50/#80）→ 判别必须 post-Done forceload 新区块 + 先核接管生效 chunk 范围。
- B3 首次取证若找不到具体描述 → 差异定义先重建，工作量上升，回用户拍板是否继续。
- 运行时载体失败 → 残留 java 进程纪律清理后重试；3 轮无新数据层证据即停手上报（evidence saturation）。
