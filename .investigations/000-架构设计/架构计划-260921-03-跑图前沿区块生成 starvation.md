---
编号: 000
任务: 跑图前沿区块生成「慢到不可用」定位（LIGHT-DOMAIN 任务饥饿嫌疑）
任务类型: 定位 + 验证
模式档位: 轻量
状态: 已批准（用户 2026-09-21 19:39 口头确认「确认，开工」）
---

## 范围（含明确不做什么）
- 做：复现 + 定位「跑图生成新区块时区块完成极慢」（1.20.1 客户端实机报障，260921-03）。
- 嫌疑主线（日志硬证据）：LIGHT-DOMAIN 域批任务全部 timedOut=true（timeout 19→37 单调爬升、avgMs 仅 3-25ms）= 队列饥饿 → latesubmit=rebuild 重建风暴 → chunk 完成被光照收尾阻塞。
- 不做：FB-2 去留裁决（仍留用户）；CP-1 增量化立项；1.21.6 侧；光照值差（d_cross/2038 恒定面）。

## 任务拆解
- T1 三臂复现（dedicated server + 程序化跑图前沿）：A=lightRust+domainbatch（现役）/ B=仅 lightRust（legacy per-chunk）/ C=vanilla 光照。判据 = 前沿 chunk 完成速率（wall + chunk/s），§9.7 口径预登记。
- T2 饥饿机制定位：A 臂日志分解（域任务排队时延 vs 执行时延、池占用、rebuild 链）；读 Mixin/Rust 域批代码确认调度前提。
- T3 修复方向验证：`latesubmit=merge` 止血臂（一行开关）± 独立 lane 方向评估；修复验证臂（如实施）。
- T4 判读 record（subagent 草稿 + 主会话应用）+ judge 审查（MUST，收尾交付）。
- T5 知识库更新（subagent 产出：workflow-patterns / build-tooling 增补；时间线 260921-03 块）。

## 验证方式
- 三臂同 seed 同 world 前置（每臂删 world 或用未生成远区，#144/#146 归档纪律）；SELFCERT 自证行（lightInit / LIGHT-DOMAIN hook armed / 臂形态日志）缺失整臂 VOID。
- 量化数字声明 §9.7 三要素（载体/覆盖面/可比性）；E1 档（同构建态单变量）。

## judge 预置
- 收尾交付 MUST judge（三源核对）；T3 若出修复验证 candidate SHOULD judge。

## fan-out 预置
- T2 若机制候选 ≥2 互斥（池饥饿 vs grace 判定 bug vs rebuild 级联）且日志不能单轮分辨 → fan-out .bN；预期日志+代码可收敛，主会话直接做。

## 知识库更新
- 结论性 docs（07/12 篇 + 10 时间线）：subagent 产出草稿 + 主会话应用。

## 子角色介入点
- scout: 否（机制主线已有日志证据，非机制未明大排查）
- worker: T4 判读 record 草稿 + T5 知识库草稿（subagent）
- fan-out: T2 潜在分叉（见上，预期不触发）
- judge: T4 MUST
- knowledge: T5 subagent 产出

## 复测口径
- 驱动：dedicated server `:runServer` + forceload 前沿区（未生成远区，避开 spawn 覆盖 #144）；臂开关经 -P 通道；日志 .tmp/lane-260921-03/。
- 用户可玩止血验证臂（可选）：runClient -PlightRust=1 -Pdomainbatch=1 -Platesubmit=merge。
