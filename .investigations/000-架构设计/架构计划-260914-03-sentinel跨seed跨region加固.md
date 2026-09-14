---
编号: 000-260914-03
任务: sentinel 判据（armed≥1 / wb>0 / crash=0）跨 seed × 跨 region 交叉矩阵加固（泛化扩展）
任务类型: 验证（测量/回归加固）
模式档位: 轻量
状态: 待批准
---

## 范围（含明确不做什么）

- **做**：sentinel 噪声/崩溃哨兵泛化——在已验证 2 seed × 2 region（260913-06 四臂全 PASS）基础上，新增 **2 个新 seed × 2 个新 region** 的交叉矩阵，两执行体（1.20.1 `DD3B645F…` / 1.21.6 `ABD7D889…`）各跑全组合 = **8 个新臂**。
- **不做什么**：不改引擎/dll/门控（纯测量，零重编）；不复跑 260913-06 旧四臂（已 confirmed，留作基线引用）；不做 Scope B/C、B3、R4 转交。

## 任务拆解（子任务 → 预期产物）

1. **P1 运行台准备**：从 `.tmp/sentinel-260913-06/run_sentinel_{1201,1216}_260913-06.ps1` 复制副本到新目录 `.tmp/sentinel-260914-03/`（变更登记文件头：out 新目录 / RTag 入标签防覆盖——#144/#146）。新 seed × 新 region 参数化（region 尺寸前置核算 **≤256 chunks**——#145）。脚本改完 **Parser 门实核**（#147）。
2. **P2 采集**：8 臂运行（2 执行体 × 2 seed × 2 region），判据 armed≥1 / wb>0 / crash=0；每臂路径自证行（`[CppBridge] init seed=`）+ .log/.err + fp 产物落 `.tmp`，**跑前先归档任何将被覆盖的旧轮日志**。
3. **P3 汇总 record**：`.investigations/sentinel-260914-03/record-260914-03.md`（draft，含 §9.7 覆盖面声明：不声称全 region/全 seed 泛化）。

## 验证方式

- 判据（沿用，不新立）：每臂 armed≥1 / wb>0 / crash=0；seed 行为化自证 `[CppBridge] init seed=` 与目标 seed 逐字一致（seed 三查）；forceload 区前置核算通过（驱动生效自证——#145）。
- 与 260913-06 基线四臂结果并列对照（同判据，不跨载具引用数值）。

## judge 预置

- 收尾交付 **MUST judge**（三源核对：record 快照 + 日志/归档 + 运行台脚本 sha）。
- 无 candidate 授予节点（单判据验证块，candidate=confirmed 同节点交用户）。

## fan-out 预置

- 无预置分叉：单判据线性验证；若某臂 FAIL 且出现 ≥2 互斥机制候选（如哨兵未触发 vs 驱动未生效），**临时 fan-out 点** = 该臂归因（.bN 并行 worker，禁止主会话自推）。

## 知识库更新

- 结论性 docs/时间线/discovered：**subagent 产出草稿**（core.worker + 先读 SUBAGENT-KNOWLEDGE-GUIDE.md）→ 主会话应用 + 验证；无新判据则按价值门不写。
- 时间线 → `versions/1.21.6/docs/10-timewise-archive.md`（1.20.1 侧对应篇按实际涉及面）。

## 子角色介入点

- scout: 否（机制已明，无勘探需求）
- worker: P3 record 汇总解读 + 知识库草稿（subagent）
- fan-out: 无预置（臂 FAIL 分叉时触发，见上）
- judge: 收尾 MUST
- knowledge: 结论落盘 MUST subagent 产出
