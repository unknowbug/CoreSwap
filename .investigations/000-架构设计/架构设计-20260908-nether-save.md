---
编号: 000
任务: nether 存档链路收尾——6 项 candidate 拍板落地 + 双跑修复设计实施 + soul B2 深挖定案
任务类型: swe（双跑修复）+ re-code（soul B2 机制归因）混合
模式档位: 重量
状态: 待批准
---

## 1. 全局视图

- **目标**：把 nether 存档链路对齐从 94.42%（SKIP_FEATURES 消融预期）推进到修复后实测达标（≥94.42%，≥3 采样区间判据）；soul 三签名 B2 定案（V3→V4→V5）；6 项 candidate 获得用户 confirmed 与否的决定。
- **范围**：双跑修复 = Rust `wg_create` 级显式 flag（句柄/调用级，禁 env 全局翻转）+ mixin 侧接线；B2 = Rust-vs-JSON rule 结构对拍 + RouterProbe 同点 selector 对照 + biome 边界带对比。
- **明确不做**：A2 103 cave_air 簇、2330 块非确定性带宽机制（低优先，本轮不动）；多世界 Rust 参数化（独立课题）。
- **开工前置（交接验证纪律 §16.3）**：继承结论「双重 feature 应用机制（+5508）」「soul 缺口在 Rust 管线内」在动手前各做一次廉价独立验证（SKIP_FEATURES 复跑一次核消融数 / V1 stage dump 复跑核签名 A 表现），验证通过才续推。

## 2. 角色分配

| 角色 | 节点 | 执行方式 |
|---|---|---|
| scout | 无需独立勘探（上轮已产出管线地图与消融证据，B2 入口明确） | — |
| worker | B2 V3/V4/V5 数据解读、双跑修复方案评审 | subagent（core.worker），主会话只执行命令不解读 |
| fan-out | B2 若 V3/V4 出互斥候选（签名 B 根因 vs 签名 C 根因互斥分叉）→ .bN 并行 | 预置，触发才执行 |
| judge | ① 双跑修复方案设计 MUST ② 修复后回归结论 candidate 授予 SHOULD ③ B2 定案 candidate MUST ④ 收尾交付 MUST（三源核对） | subagent |
| knowledge | 09 篇/10 时间线/errors 台账更新 | subagent 产出草稿 + 主会话应用 |

## 3. 任务拆解 & 依赖图

```
P0 用户拍板 6 项 candidate（人工 HOOK，先于一切落地动作）
P1 交接验证（廉价复跑：消融数 + soul 签名 A 复现）→ 通过才继承
P2 双跑修复设计 → judge 审查 → 实施 → 回归（依赖 P0 对「双跑」项的拍板）
P3 B2 深挖 V3（Rust-vs-JSON 对拍，零成本）→ V4（RouterProbe 同点）→ V5（biome 边界带）
P4 overworld 双跑量化（seed B SKIP_FEATURES 消融 run；依赖 P2 flag 实现可复用）
P5 知识库/文档收尾 + judge 收尾审查
```

## 4. 并行执行计划

- 第一波：P0（用户）∥ P1（主会话复跑）∥ P3-V3（零成本，worker 解读）
- 第二波：P2 设计+judge ∥ P3-V4/V5
- 第三波：P2 实施+回归 ∥ P4 消融
- P5 收尾

## 5. 人工决策 HOOK 点

1. P0：6 项 candidate 拍板（confirmed / 维持 candidate / 驳回）。
2. P2 修复方案 judge 通过后实施前：方案批准（wg_create 加参 vs 新导出函数选型）。
3. 回归判据达标 → candidate 授予建议 → 用户 confirmed。
4. B2 定案 → 用户拍板。

## 6. 风险 & 回退

- env 全局门误翻 → 破坏 DensityProbe 纯净性：修复必须句柄/调用级，禁全局默认翻转（NEXT_SESSION 既定）。
- gradle 沙箱 AccessDenied（E8）：run 必带 JAVA_TOOL_OPTIONS tmpdir；残留 java 进程先 Stop-Process。
- seed 三查铁律：任何对比前核 seed/坐标语义/参照 header（2026-08-31 强化版）。
- bin-diag 编译法（E9）：soul_selector_probe 临时挪 src/bin/ 用完挪回。

## 7. judge 步骤预置

- 节点: 双跑修复设计方案 | MUST | 审查对象: 设计产物 + worldgen_handle.rs 现状
- 节点: 修复回归结论 | SHOULD | 审查对象: 回归数据 + 判据声明（§9.7 可比性三要素）
- 节点: B2 定案 | MUST | 审查对象: V3/V4/V5 产物 + .bN 候选（若触发）
- 节点: 收尾交付 | MUST | 三源核对（.artifacts 快照 + git diff + 验证记录）

## 8. fan-out 步骤预置

- 节点: B2 V4 后若签名 B 根因分叉 ≥2 互斥候选（selector 语义差 / 输入噪声差）| .bN 并行 worker | 禁止主会话自推

## 9. 知识库更新

- 结论性 docs（09 篇新节 / 10 时间线 / nether-save-full-errors.md）/ discovered（若有新通用模式）: knowledge worker subagent 产出草稿（prompt 必含 SUBAGENT-KNOWLEDGE-GUIDE.md 指引行）+ 主会话应用验证。

## 10. 子角色介入点（全部预置）

- scout: 无独立勘探（上轮管线地图可复用，P1 复跑即核验）
- worker: B2 V3/V4/V5 解读、消融数据解读、docs 草稿 — subagent
- fan-out: B2 分叉点（见 §8）— 触发才执行
- judge: §7 四节点 — subagent
- knowledge: P5 — subagent 产出 + 主会话应用
