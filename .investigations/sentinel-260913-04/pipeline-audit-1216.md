# 1.21.6 chunk 管线逐行核对记录（260913-04）

## 0. 载体与前提

- **一手源**：1.21.6 yarn（`net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2`）merged jar（`versions/1.21.6/java/.gradle/loom-cache/minecraftMaven/.../minecraft-merged-b07cf08c30-1.21.6-...-v2.jar`），用 vineflower 1.11.1（loom genSources 同款反编译器，缓存于 `.gradle-home/caches/modules-2/`）定向反编译所需类 → `.tmp/sentinel-260913-04/mcsrc1216/`。
- **被继承的结论**：`.investigations/r9b-260913-02/record-260913-02.md` §2 A1-A6（1.20.1 confirmed，2026-09-13）。本次 = §16.3 交接验证 + 覆盖面扩展，A1-A6 逐点对拍 1.21.6 对应物。
- **验证分层**：Degraded（纯静态源码论证）+ 行为级冒烟（见 §2，Full 运行时证据，待跑）。

## 1. 类名/结构变化（1.20.1 → 1.21.6）

| 1.20.1 | 1.21.6 | 变化 |
|---|---|---|
| `ChunkHolder.futuresByStatus`（ChunkHolder 内） | `AbstractChunkHolder.chunkFuturesByStatus`（新增父类，`world/chunk/AbstractChunkHolder.java:33`） | 结构上移 + 改名，仍为 `AtomicReferenceArray<CompletableFuture<OptionalChunk<Chunk>>>` |
| `ChunkHolder.getChunkAt` 缓存查询 | `AbstractChunkHolder.getOrCreateFuture`（:130-152，CAS `compareAndExchange` 去重） | 同 (chunk,status) 二次请求复用同一 future，机制不变 |
| `ThreadedAnvilChunkStorage` | `ServerChunkLoadingManager` | 改名 |
| `ChunkStatus` 任务 = `register(..., prev, ...)` + `populateNoise().thenApply()` | `ChunkGenerationStep`（record：targetStatus/directDependencies/accumulatedDependencies/task）+ `ChunkGenerationSteps` 注册链 | 任务定义从 ChunkStatus 内联 lam 分离为 step 表；`Builder(previousStep)` 链式（ChunkGenerationStep.java:63-71，乱序抛异常） |
| `upgradeChunk` 内 `getRegion` 等邻居前置状态 | `ChunkLoader.loadAll/load`（`world/chunk/ChunkLoader.java:113-179`）按 `accumulatedDependencies` 把区域邻居推到前置状态 + `ServerChunkLoadingManager.generate:639` 强制 `getUncheckedOrNull(targetStatus.getPrevious())` 非 null（"Parent chunk missing"） | 前置状态语义保留且更结构化（J1 措辞依然成立：等待的是生成用前置状态） |
| —（无） | `progressStatus`（AbstractChunkHolder.java:220-230，`currentStatus` CAS） | **新增**：每状态任务启动一次性 CAS 门，比 1.20.1 更强的单写者保证 |

## 2. A1-A6 逐点对拍

| # | 断言（1.20.1 confirmed） | 1.21.6 判定 | 证据（mcsrc1216 行号） |
|---|---|---|---|
| A1 | 同 chunk 同状态任务只执行一次 | **✅ 成立（更强）** | `chunkFuturesByStatus` CAS 去重（AbstractChunkHolder.java:137-148）；`progressStatus` CAS（:220-230）保证每状态至多启动一次生成，二次 `generate` 走 `getOrCreateFuture` 复用（:78） |
| A2 | 状态任务严格有序（NOISE→后续） | **✅ 成立** | `ChunkStatus.register` previous 链（ChunkStatus.java:21-32，NOISE.prev=BIOMES…FULL.prev=SPAWN）；`ChunkGenerationStep.Builder(previousStep)` 乱序抛异常（ChunkGenerationStep.java:63-71）；`ChunkLoader.loadNextStatus` 按索引 +1 逐状态推进（ChunkLoader.java:57-70） |
| A3 | 邻居读不会在 fill 期间读本 chunk section（等待前置状态） | **✅ 成立** | `ChunkLoader.loadAll` 区域遍历把邻居推到 `accumulatedDependencies` 前置（ChunkLoader.java:113-151）；`generate` 强制父 chunk（前一状态）已完成，否则 "Parent chunk missing"（ServerChunkLoadingManager.java:638-641）；`ChunkGenerationSteps` 依赖表 `dependsOn(status, level)` 结构化表达（ChunkGenerationStep.java:73-90） |
| A4 | 写入对下游可见（happens-before 经同一 future 链） | **✅ 成立** | 全部消费（`getUncheckedOrNull`/`getOrNull`/`getLatest`/`replaceWith`/`completeChunkFuture`）均读写 `chunkFuturesByStatus` 同一 future（AbstractChunkHolder.java:172-194, 259-278）；`CompletableFuture` 完成语义同 1.20.1 |
| A5 | NOISE→LIGHT 间无绕过 future 的 section 消费者 | **✅ 成立（静态排查，同残留边界）** | proto chunk FULL 前不进 world；light 为 `INITIALIZE_LIGHT`/`LIGHT` 状态任务（ChunkStatus.java:30-31，注意 1.21.6 拆出 INITIALIZE_LIGHT 新状态）；本次静态排查未发现跨 future 边缓存 section 的路径。⚠️ 残留边界随转：对未来新增消费者不承担证明义务 |
| A6 | 主线程不并发触碰生成中 chunk | **✅ 成立** | `ServerChunkManager.getChunk:160/178` 主线程 `join()` future 等待；`getChunkFutureSyncOnMainThread:227-237` `thenCompose` future 链，不直读 proto section |

**结论（candidate）**：260913-02 A1-A6 单写者结构性论证在 1.21.6 管线**全部成立**；且 1.21.6 新增 `progressStatus` CAS 门使 A1 保证比 1.20.1 更强。1.20.1 confirmed 结论的覆盖面可扩展到 1.21.6（管线侧）。

## 3. §9.7 可比性声明

- 载体：静态源码论证（1.21.6 yarn merged jar 反编译，反编译器 = loom genSources 同款 vineflower 1.11.1）。
- 覆盖面：chunk 生成管线状态机（ChunkStatus/ChunkHolder/ChunkLoader/ServerChunkLoadingManager/ServerChunkManager）；BulkWb 工程侧共享核代码 1.21.6 未在本块重读（260913-03 judge 已核共享核接线）。
- 与既有口径可比性：与 260913-02 record §2 同构对拍表，直接可比。

## 4. 冒烟结果（Phase 2，已执行 2026-09-13 17:40-17:43）

- 运行臂：`.tmp/sentinel-260913-04/run_sentinel_1216.ps1`（承 run_sentinel_1201.ps1 改造：1.21.6 工程/运行时目录、`-PcppLib=target\release\worldgen1216.dll`、裸 `runServer`（#62：双命中为 1.20.1 独有）、不杀 java 进程）。
- 前提已核：`versions/1.21.6/java/.../WgCompat.java:11 BULKWB_ON = true`（bulk 路径在位）；seed 417950215108767439；post-Done forceload -128..127（#80）；run 目录 world 已删（#19/#88）。
- 两臂结果（串行）：
  | 臂 | armedLines | wbLines | sentinelCrash | suspExc |
  |---|---|---|---|---|
  | off | 0 | 576 | 0 | 3 |
  | on | 1 | 576 | 0 | 3 |
- suspExc=3 两臂同值，实读 = OSHI WmiQueryHandler COM 警告（Win32_Processor/PhysicalMemory），与 260913-03 #139② 良性噪声同签名。
- 判读：门开 armed 行为化自证 ✓；576 chunk 零误报零 crash；门关 armed=0。**sentinel 在 1.21.6 行为符合预期（Full 运行时证据）**。

## 5. 本块结论（**confirmed**，用户授予 2026-09-13；judge PASS-with-conditions 条件已应用；原 candidate 记录随 confirmed 回执升级）

1. A1-A6 单写者结构性论证在 1.21.6 管线全部成立（见 §2），且 `progressStatus` CAS 使 A1 更强 → 260913-02 confirmed 结论覆盖面扩展到 1.21.6（管线侧）。
2. sentinel（默认关）在 1.21.6 共享核行为正常（两臂证据，§4）；1.21.6 行为零变化（门关臂与既有冒烟形态一致）。

## 6. judge（隔离 subagent，2026-09-13）= PASS-with-conditions，条件已应用

- 三源核对全部独立实读通过；唯一偏移 = ServerChunkLoadingManager.generate 实际 :635-664 / "Parent chunk missing" :644（audit 写 :632-661/:639-641，偏移 ~3 行，机制措辞逐字一致）；INITIALIZE_LIGHT 确为 1.21.6 新状态（:29-30 实证）。
- SHOULD（已应用）：两臂原始日志补落盘 → `cmd-output/sentinel1216-{off,on}.log(.err)`（on 臂 sha256 501E8579...6D17）。
- INFO（已注明）：① suspExc=3 计数口径 = 主 log OSHI WMI COM 警告；`.log.err` 另有 2 条 gradle launcher rubygrapefruit thread error=5（良性，不在该计数内）。② 行号以实测为准，后续记录写实际行号。
- 审查意见存档：judge subagent 交付 review-260913-04.md。
