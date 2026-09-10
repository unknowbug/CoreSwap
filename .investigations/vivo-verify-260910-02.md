# vivo-verify-260910-02 — B3a 快照 + CAP 2048 实机验证判读（实际 2026-09-10 14:15-14:32）

> 任务：架构计划-260910-02-vivo-b3a-verify.md（用户批准）。上块（260910-01）B3a + CAP2048 全部结论来自 bin-diag bench 载体，本轮确认生产口径健康。
> 原始日志：cmd-output/vivo-run-260910-02.log（全量落盘，#41）+ cmd-output/vivo-verify-260910-02.md（执行记录）。

## 判据矩阵

| 判据 | 预期 | 实测 | 判定 |
|---|---|---|---|
| dll 执行体三元组 | = e91c7f6 构建产物 | 加载 dll sha256=5e30187a…（size 2455552）= 本次 14:22 rebuild 的 target/release/worldgen1216.dll | ✅ |
| dll 内容哨兵 | 含 B3a 探针标签 | CA-MEMO-OFF / CA-NT 双命中（bin 字符串扫描） | ✅ |
| 陈旧警报 | — | 原 staged dll = 09-09 16:11（B3a 前旧产物）→ 强制 rebuild + processResources 重拷（#23 家族再现，见下「过程记录」） | ⚠️ 已处置 |
| [CppBridge] 管线 | enabled=true（#28 签名） | init/initNether/initEnd 三句柄 enabled=true stageMask=3 | ✅ |
| vivo 接管口径 | stageMask=3 出货默认（#66） | stageMask=3；populateNoise intercepted=242 / buildSurface skipped=242（Rust 地形接管活跃）；placedFeature skipped=0（features 由 Java vanilla 画，mask 口径自洽） | ✅ |
| 生成触发面 | post-Done forceload（#80） | RCON forceload 25,25 与 -25,-25 两区各 9 chunks，实际生成 242 chunks（含邻接装载） | ✅ |
| 崩溃/异常面 | 无 Rust panic | worldgen panic/error = 0；命中 Exception 均为 gradle 环境噪声（File watcher WmiQueryHandler，沙箱面非 worldgen） | ✅ |
| 干净关服 | RCON stop | Stopping server=1，BUILD SUCCESSFUL，exit 0 | ✅ |

## 结论（confirmed，260910-02 · 实际 2026-09-10 14:40 用户授权；judge PASS-with-conditions 条件已应用）

**B3a 3×3 Arc 快照 + CAP 2048 在 vivo 生产口径（gradle runServer -PcppReplace，stageMask=3，CA_MIN 默认开）管线健康**：加载正确执行体、242 chunks 经 Rust 地形接管生成、零崩溃、干净关服。

验证分层声明（§9.7）：**Partial**（运行时行为面确认——接管生效行为化哨兵 + 执行体指纹 + 无异常；未做生成内容逐位对拍，hash 逐位不变已由上块 bench 轮 Full 闭环，本块轮换执行体到 vivo 载体）。IDK-b3（多世界/大 region 内存上限）仍开放，待多世界接入复核。

## 过程记录

- 启动前发现 target/release 两 dll（09-09 22:39）早于 B3a 源码提交 e91c7f6（23:32）→ rebuild 双薄壳（-p worldgen / -p worldgen1216）+ bin 字符串哨兵验证。staged native/worldgen.dll（16:11 旧产物）由 runServer processResources 自动重拷（#96 流程正常走通，`synced` 体现为加载 sha 与新产物一致）。
- 判读源：日志行为化行（#37/#81），非静态推断。spawn 预生成不作为验证面（#80），验证 chunk 全部来自 post-Done forceload。

## 知识库评估

- 陈旧 dll 警报 = #23 家族既有知识（mtime/Finished 不可信、内容指纹哨兵）的又一次例行命中，**无新机制**，按价值门不写 discovered；时间线也不追加（一次性确认，低价值）。
