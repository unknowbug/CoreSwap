---
编号: 000
任务: 三 mod 兼容调试续（待办①接管激活证据 → ②Voxy Ingest 分支关闭 → ③性能归因 → ④DH e2e）
任务类型: 定位/验证（实机调试）
模式档位: 轻量（多子任务、依赖串行、含实机运行）
状态: 用户已批准（260908-04，Get-Date 2026-09-08 10:32）
---
## 范围
- 做：voxy-debug-260908-03.md 四待办；优化线全部挂账不动；Epic Terrain 不进本轮。
## 任务拆解
1. 待办①：`-Dcpp.blockRegister=1` 客户端重跑 + [CppBridge]/[BLOCKS-REG]/stageMask 行为化证据 + 生成速率 vs vanilla 单人（#36 integrated server 形态首验；R3 mixin 注入为已验证事实可继承，「接管生效」是待验假设）。
2. 待办②：R3 日志核 Rust 地形异常模式是否同为 idx 126/len 64 → 关闭「CoreSwap 引发」分支。
3. 待办③：仅当②闭合后才测飞行 CPU 占用；否则声明暂缓（#34 谓词耦合）。
4. 待办④：DH e2e（FEATURE_GENERATOR）视进度。
## 验证方式
实机主会话执行；日志落 cmd-output/；判据行为化（#37/#81）。
## 子角色介入点
- scout: 否（勘探已完成，定向调试）
- worker: ②日志异常模式解读 + 结论落盘（subagent）
- fan-out: 预置——①若参数带而无输出 → (a 参数映射未生效 / b 客户端不走 CppBridge init / c Connector remap 吞门控)，≥2 互斥即 fan-out
- judge: MUST 本轮结论闭合（三源核对）；SHOULD ②分支关闭 candidate
- knowledge: NEXT_SESSION 待办 2 两候选，subagent 草稿 + 主会话应用
## 风险 & 回退
客户端链 R3 已通但骨架薄；回退 `runtime/forge-client-test/` 备用载具。
