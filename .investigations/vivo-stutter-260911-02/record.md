# vivo-stutter-260911-02 — 实机 VD32 卡顿（1.0.28 jar 00236547 / dll dd3b645f）

> 实际 2026-09-11 14:33 起。触发：用户 confirmed ca_min 门控微修 + 撤回 RELEASE-1.0.28 工单（实机测试「还是有问题」——性能/卡顿）。

## 用户日志判读（latest.log，144 行，附件副本 sha256 f2bc8500…）

- 载体：**集成服（单机客户端）**，Fabric 1.20.1，59 mods（coreswap 1.0.28 + fabric-api），**Java 24**，机器路径 `D:\Games\Minecraft\coreswap`，用户 Valkyrozen。
- 执行体自证：`[CppBridge] dll=…\Temp\coreswap-native\worldgen.dll sha256=dd3b645f…`（= 本块修复版，测的确实是最新产物）+ `stageMask=3` 三维度 enabled。
- **seed `7349435828306001495`** = 260910-08 §6 朋友报告日志同一 seed ⇒ 可能同一台机器/同人，两次报告同源（待确认）。
- 视距 10→**32**、模拟距离 32；13:42:40 起旁观模式飞行 ~2 分钟；13:44:15 `Received passengers for unknown entity`（孤立 WARN，暂无因果）；13:44:32 退出。
- `[CppBridge] init` 在 spawn region（8.8s）Done 之后 ⇒ 出生区恒 vanilla 生成（已知边界）。
- 卡顿窗口：推测 = 13:42:40-13:44:15 飞行探索段（VD32 高负载）。

## 候选假设（互斥）

- **H1 客户端共享工作池争用**：async worldgen 与客户端区块网格构建共用 `Util.getMainWorkerExecutor()`，集成服同进程争用——专用 dev server 载体结构性测不到（260910-08 §6 候选 2）。成立前提：CoreSwap 的池占用形态与 vanilla 可辨差（vanilla 用同一池）。
- **H2 对照缺失假象**：VD32 + 高速飞行本身极端负载，同机 vanilla VD32 可能同样卡（#64/#65：无对照实机观察不可归因）。
- **H3 接管尾延迟形态差**：突发区块请求下 async 提交/回调链尾延迟 ≠ vanilla（#83 延迟 vs 吞吐分母）。

## 待用户补充（判别信息）

1. 同机 vanilla + VD32 + 同样飞行：卡不卡（H2 一票判据）
2. 卡顿形态：持续低帧 vs 周期顿挫；静止是否卡
3. 机器规格（CPU/GPU/内存）
4. （可选）spark profiler 30s 链接

## 纪律注记

- 本块 A/B（dev server + Chunky）「无回归」结论**禁止外推**到集成服 VD32 场景（§9.7 载体差；verdict-260911-01 §3 已加⚠️）。
- 机制未明 ⇒ 开工先 scout 勘探（管线/线程面地图）；分叉 ≥2 互斥 ⇒ fan-out。
