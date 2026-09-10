---
编号: 000
任务: B3a 快照 + CAP 2048 的 vivo 实机验证（bin-diag bench 载体 → 生产口径确认）
任务类型: 验证
模式档位: 轻量
日期: 260910-02（实际 2026-09-10 14:15，Get-Date 实锚）
---

## 范围（含明确不做什么）
- 做：一次实机 runServer（1.21.6 主线版本），确认 ① `[CppBridge] init ... enabled=true` 管线正常（#28 判别签名）② dll = e91c7f6 构建产物（执行体三元组核验，#96：加载文件 sha vs target 产物 sha）③ B3a 快照 / CAP 2048 在 vivo 生成若干 chunk 无崩溃、无 hash 异常日志。
- 不做：性能残差测量（IDK-b1 另立项）；多世界内存测试（IDK-b3，等多世界接入）；功能对拍（hash 逐位不变已在 bench 轮验证，本轮只确认生产管线健康）。

## 任务拆解（子任务 → 预期产物）
1. 前置核验：target/release/worldgen1216.dll sha + mtime（内容指纹哨兵，#23），必要时 rebuild → cmd-output 记录
2. 实机 runServer 启动 + post-Done forceload 触发若干新区块（#80：boot spawn 预生成不算 vivo 验证面）
3. 日志判读：[CppBridge] 行 + 生成 chunk 计数 + 无 panic/异常 → .investigations/vivo-verify-260910-02.md

## 验证方式
- 行为化日志判据（#37/#81）：[CppBridge] enabled 行命中 + chunk 生成行为发生 + 无崩溃捕获。验证分层 = Partial（运行时行为面，非逐位 Full）。

## judge 预置
- 收尾交付 MUST judge（三源核对：.investigations 记录 + git HEAD/diff + 运行日志）——本轮结论只到 candidate（vivo 管线健康确认），无需用户 confirmed 定论（除非发现异常升级课题）。

## fan-out 预置
- 无预置分叉。若实机出现崩溃/管线异常且机制不明 → 暂停回报用户，另立课题（届时按互斥候选 fan-out）。

## 知识库更新
- 预期无新发现 = 无写入（价值门：一次性确认不写）。若踩新坑 → subagent 产出草稿（先读 knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用。

## 子角色介入点
- scout: 否（范围明确，无机制未明）
- worker: 日志判读若出现异常且发散 → subagent；正常收敛判读主会话直接做
- fan-out: 无
- judge: 收尾 MUST（三源核对）
- knowledge: 无新发现则无；踩坑则 subagent 产出
