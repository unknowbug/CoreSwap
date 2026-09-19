# B3 余额重算（B2 清单逐条三态核对）

- status: draft
- 日期标签: 260919-06（角色 core.worker，只读核对 + 本产物）
- 首证: `.investigations/000-架构设计/架构计划-260911-05.md` §3 B2 清单 `:68-80`；方向变更 `:152-164`（A1a/A1b 作废 `:159`；exec/P1/指纹门/StallWatch 正交重排 `:160`）；B3 定义 `:84-86`。
- 方法: 逐条 grep/直读 `versions/1.21.6/java` + 共享核 `java-core/` + `worldgen-core/src`。所有 file:line 为本 session 实读。

## 余额表（B2 清单 6+3 项）

| # | 项（B2 原文行） | 三态 | 证据（file:line） |
|---|---|---|---|
| 1 | exec 模式（自有有界执行器，缺省开）★用户点名（`:71`） | **仍缺** | 1.21.6 `NoiseChunkGeneratorMixin.java:61/145/148/183/186/219/222` 仅 SYNCFILL + `supplyAsync(work, WG_FILL_POOL)`（R3 共享池）；`CoreSwapExec`/`coreswap.exec` 全树仅 1.20.1 mixin 命中（`1.20.1/.../NoiseChunkGeneratorMixin.java:125-159`） |
| 2 | P1 信号量 + `-Dcoreswap.maxinflight`（`:72`） | **仍缺** | `maxinflight`/`WG-INFLIGHT` 全仓 Java 仅 1.20.1 mixin 命中（`:83-111`）；1.21.6 树与 java-core 均零命中 |
| 3 | `[WG-CONTENT]` 指纹门 + `wgBufHash`（`:73`） | **已顺带完成**（共享核） | `java-core/src/main/java/wg/bench/CppBridge.java:432`（wgBufHash）/`:491-492`（overworld）/`:536-537`（nether）/`:590-591`（end，260911-05 A2 补齐三维）；`:419` nz 扫描+16 点读回+指纹统一门控 |
| 4 | StallWatch 停滞看门狗（`:74`） | **已顺带完成**（共享核） | `java-core/src/main/java/wg/bench/StallWatch.java` 存在；1.21.6 `build.gradle:147-156` 已接线 `-Pstallwatch`（注释明言「两版同一份代码」，消费点 `java-core/BulkWb.java:91/93/95/97 + CppBridge.java:616/628 + StallWatch.java:21`） |
| 5 | A1a writeChunk 逐块含空气（`:76`） | **已作废**（且被 C 覆盖） | 作废记录 `架构计划-260911-05.md:159`；bulk 写回已落地并默认开：1.21.6 `WgCompat.java:7-10`（bulkwb 默认 true，260913-01 三臂×三维等价验证翻转） |
| 6 | A1b nz 扫描门控（`:76`） | **已作废**（共享核门控顺带实现） | 作废记录 `:159`；实现面由 `CppBridge.java:419` 门控整块控制（MIXLOG 门）取代原恒执行扫描 |
| 7 | A1c 6 高度图单遍（`:76`） | **已作废**（前提证伪） | `架构计划-260911-05.md:41`「vanilla 本就单遍 → 用户裁决放弃（HOOK-2）」——非 1.21.6 特有缺项 |
| 8 | ca_min gate 微修（`:78`，Rust 共享） | **已顺带完成** | `worldgen-core/src` 共享；1.21.6 docs `10-timewise-archive.md:46/:59`（ca_min B3a 快照落地 + hash 硬门 ✅）；1.21.6 `build.gradle:261` 接线注释 |
| 9 | adaptive_threads clamp（`:78`，Rust 共享） | **已顺带完成** | `worldgen-core/src/api.rs:24-43`（clamp `min(count).max(1)` 在位）；Rust 共享 ⇒ 一次覆盖两版 |
| — | perfprofile 残留（`:80`） | 无残留（B2 原文已核） | 原文「全树零命中」 |

## 余额结论

- **6+3 项中：仍缺 = 2（exec 模式、P1 maxinflight）；顺带完成 = 4（指纹门、StallWatch、ca_min、adaptive_threads）；作废 = 3（A1a/A1b/A1c）。**
- 仍缺两项均为 1.21.6 分版 mixin/调度面（Scope A 边界内 mixin 留分版，共享核不覆盖），且方向变更记录 `:160` 已定性「与 C 正交…按需重排」——无验证前置压力（指纹门原为 C 验证前置，C 已于 260913-01 用三臂×三维等价收口并默认开）。
- **一句建议**：不满足「正式关账」条件（2 项真缺口仍在、未作废），但可**继续挂起**——两项纯为 1.20.1 侧优化/诊断能力（有界执行器 + inflight 限流）的 1.21.6 对齐，触发条件 = 1.21.6 实际上线/并发压测需求出现时恢复；届时工作量 ≈ 移植 1.20.1 mixin 的 exec/P1 段（约百余行、同形移植 + 1.21.6 A/B 一轮），无 Rust 侧改动。若用户判断 1.21.6 永不对齐运行形态，则可升格为 wontfix 关账。

## 边界与诚实声明

- 本产物 draft；不改变任何 status；「已顺带完成」判据 = 现源码实读命中（非行为级 A/B 复验——共享核文件两版同源引用见 build.gradle:148 注释，行为级证据在各自原块）。
- exec 模式在 1.21.6 的「形式不同但等价」可能性：已排查——1.21.6 mixin 仅 SYNCFILL/共享池两条路径，无自有有界执行器形态，判定「真缺」（B2 `:82` scout 问题一并回答）。
- 反向回灌（1.21.6→1.20.1）按 `:161` 只登记不实施，不在本余额表范围。
