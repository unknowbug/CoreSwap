---
编号: 000
任务: R9-b 并发义务缺口——light reader × bulk writer 的 SENTINEL 覆盖判定
任务类型: 并发安全性判定 + 可选加固
模式档位: 轻量
状态: 已批准（用户 2026-09-18 21:2x，260918-10）
---
## 范围（含明确不做什么）
- 判定：光照读路径（light reader）与 bulk 写回（`BulkWb` 原地换容器）并发时，读者是否被 R9-b「单写者不变量」的可见性链（per-status future happens-before）覆盖。
- 若存在真缺口 → sentinel（`-Dcoreswap.bulkwbsentinel`）覆盖扩展最小方案 + 两臂验证。
- 不做：其他遗留项（R4/legacy run3/A1/Scope B/C/B3 等）、性能课题、1.21.6 专项移植。

## 任务拆解
- T1 scout 勘探（subagent 只读）：生成期 sections 读者清单（Java light status 任务 / Rust 光照 JNI 读 / 主线程 / 诊断读回）× 线程域 × 获取途径（future 链上/链外）。产物 `.investigations/r9b-light-260918-10/scout-map.md`。
- T2 收敛判定（主会话）：对照 #143 四件套 + sentinel 检测域（写者×写者 chunk 级）判定覆盖状态。fan-out 预置：≥2 互斥候选（链上已覆盖 / 链外真缺口 / 链上但可见性边不足）→ 强制 .bN 并行。
- T3 验证 + 收尾：缺口实存 → sentinel 扩展最小实现（主会话写码，声明未编译验证 + 静态自检清单）+ 门开/门关行为化两臂；judge MUST（三源核对）；知识库更新 subagent 产草稿 + 主会话应用。

## 验证方式
- 行为化自证（#81）：armed 正/负成对 + 门关零介入；读者义务判定 = 一手源 file:line 对拍（#143 模板四件套）。

## judge 预置
- candidate 授予 SHOULD judge；收尾交付 MUST judge（三源：artifacts 快照 + git diff + 验证记录）。

## fan-out 预置
- T2 判定树分叉 ≥2 互斥候选 MUST fan-out（禁止主会话自推）。

## 知识库更新
- 结论性 docs/discovered 写入：subagent 产出草稿（core.worker）+ 主会话应用验证（最后一项恒置）。

## 子角色介入点
- scout: T1 机制未明勘探（生成期读者全清单）| 是
- worker: T2/T3 分析解读与代码交付
- fan-out: T2 分叉点（覆盖状态三候选）
- judge: 收尾 MUST + candidate SHOULD
- knowledge: 结论性落盘 subagent 产出
