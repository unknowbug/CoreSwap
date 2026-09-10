# 260910-03 用户实机冻结排查（1.21.6 vivo 冻结/卡死）——已结案：根因 = 数据缓存陈旧 id 域错位，用户 confirmed 冻结解决

> 起点：用户自测新构建 jar（1.21.6-0.1.0，dll sha 5e30187a…= 260910-02 vivo confirmed 同一产物）。
> 本文件 = 过程记录（主会话写）。结论性落盘待 judge 后由 subagent 产出。

## 根因（终局，用户 confirmed 冻结解决 260910-03）

**`CoreSwapFixHelper.extractWorldgenDir()` 数据缓存陈旧 → id 域错位**：
- `coreswap-data` = 跨版本共享固定 Temp 路径；缓存判据只有两个 existence marker（overworld.json + tags marker）
- 用户机器上 1.20.1 生产 jar 留下的旧缓存 marker 全在 → 1.21.6 jar 跳过解压
- **1.21.6 dll 读 1.20.1 blocks.json：982/1003 同名块 id 错位**（1003 vs 1105 块表）
- → 每个 chunk 大规模错块（灰草/沙砾/假基岩）+ 无支撑重力块 → 级联坍塌 + 邻居更新链
- → 103 万 item 实体 → O(n²) 实体碰撞（Server thread 166s CPU 烧在 FallingBlockEntity tick）+ 16GB 堆 OOM → 冻结
- 修复 = extractWorldgenDir 加内容指纹（jar blocks.json vs 缓存逐字节比对，不一致整体重解压，对齐 extractNativeDll 已有同款）；修复 jar sha 9eae48cd…（16:44）
- 铁证：Temp 缓存 blocks.json sha ee01b749（=1.20.1 内容）vs jar 内 617c3dae；实体普查 minecraft:item ×1,031,765；读图卡死 dump = FallingBlockEntity tick 实体碰撞扫描

## 为什么排查绕了远路（方法论复盘）

1. dev 服/Chunky/forceload 全干净——因为 `-PcppWorldgenDir` 直读权威数据不走 Temp 解压路径（双臂 diff 0.015% 是权威数据的差，与客户端故障面完全无关）
2. 客户端 2×2 对照（vanilla 不冻 / coreswap+sim10 仍冻）才把责任钉在 CoreSwap；内容普查又排除内容面 → 转向行为面
3. 三份线程 dump 逐步收敛：FallingBlockEntity→getChunk park → 流体 tick park → 读图卡死 = FallingBlockEntity tick O(n²) 实体碰撞（RUNNABLE 166s CPU）→ 实体普查百万 item 一锤定音
4. 用户视觉观察（灰草/沙砾/假基岩）是错块的关键人证

## 遗留：1.21.6 性能回归（下轮立项，用户拍板）

- 量化基线（本日双臂同会话）：vanilla 52s vs coreswap 268s / 4225 chunks = **5.15× 慢**
- 1.20.1 时代曾快 ~1.2×，1.21.6 反转；嫌疑面：多 worker 并发下 fillBlocks 线程超额订阅 / WG_CA_MIN 默认开的邻 chunk 重算（camin-perf 语义优先决策）/ est_l2 / per-chunk JNI 单批调用
- 注意：冻结修复后回归才显性化

## 事件链（过程留档）

1. **崩溃 #1（15:09）**：`NoSuchMethodError ShaderParser.parseShader`——Voxy 0.2.4-alpha ↔ Sodium 0.6.13 内部 API 失配，与 CoreSwap 无关。
2. **崩溃 #2（15:14，带 Epic Terrain 等 mod）**：`IllegalStateException: Requested chunk unavailable during world generation`，FEATURES 阶段 feature 读未生成邻居 chunk（`wgSkipPlacedFeature` redirect 核对为纯委托，mask=3 下 features 走 Java）。根因背景下重新判读：错块地形上 feature 行为异常的连带症状，非独立 bug。
3. **冻结 #1/#2/B/C（多次现场捕获）**：见上根因。冻结有两种表现（日志停止/级联卡顿+自行恢复），同一根因。

## 已排除清单（证据）

- Rust 死锁：冻结#2 中 Rust 调用持续返回（2375 unique chunks 推进）；fillBlocks 卡帧 60s 后消失（同一 Worker 转空闲）
- 地形内容差（权威数据口径）：双臂 0.015%（含 features run 噪声），15 个级联点零差
- 光照接管：默认关（需 -Dcoreswap.light.rust）
- 专用服复现：Chunky 4225 + forceload 1600 覆盖级联区零异常（原因见方法论 1）
- mixin 配置全服务端，无 client mixin

## 产物清单

- 用户日志：.tmp/hang-repro-260910/client-{frozen-live,B-final,C-live,C-live2}.log
- 线程 dump：.tmp/hang-repro-260910/client-threaddump-{elev,elev2,c1,c2,load}.txt
- 实体普查：.tmp/hang-repro-260910/count_entities.py（结果：item×1,031,765 / falling_block×79）
- 双臂数据：.tmp/hang-repro-260910/arms/（diff-result.txt = 0.015% 普查 + 定点对拍）
- 复现/看护驱动：.tmp/hang-repro-260910/*.ps1（Chunky radius 单位坑已记）
- 修复：CoreSwapFixHelper.java isDataCacheStale（内容指纹）

## 事件链

1. **崩溃 #1（15:09）**：`NoSuchMethodError ShaderParser.parseShader`——Voxy 0.2.4-alpha ↔ Sodium 0.6.13 内部 API 失配，与 CoreSwap 无关（栈内无我们帧）。用户自行配平。
2. **崩溃 #2（15:14，带 Epic Terrain 等 mod）**：`IllegalStateException: Requested chunk unavailable during world generation`，FEATURES 阶段 feature 读未生成邻居 chunk，经我们 `wgSkipPlacedFeature` redirect 帧（核对 ChunkGeneratorFeaturesMixin.java:126-138 = 纯委托，mask=3 下 features 走 Java，语义与 vanilla 逐字同）。
3. **冻结 #1（15:26，仅 CoreSwap 无其他 mod）**：游玩 ~40s 后世界冻结、角色可动、需强杀。日志戛然而止于 `populateNoise intercepted chunk(-32,0)`。
4. **冻结 #2（现场捕获，15:42-15:51）**：用户配合复现 + 本会话实时看护。

## 现场证据（冻结 #2；judge 修正：读图卡死 dump 的双线程栈为 **ItemEntity（class_1542）tick 块碰撞**，dump #1 的 FallingBlockEntity 帧为另一时刻样本——两者并存）

- 日志**持续活跃**：2375 unique chunks 无重复推进（~21 chunks/s），地形生成本身健康。
- **`Too many chained neighbor updates` 反复出现**（15 处，x -1531~-71，z -588~316，y 42-63，地表带）。
- **`Can't keep up! Running 101531ms or 2030 ticks behind`**（15:44:49）。
- **`OutOfMemoryError: Java heap space`**（15:51:22 关服时 JNA Cleaner 线程暴露；-Xmx16384m）。
- dump #1（15:44）：Server thread = `FallingBlockEntity.tick → getBlockState → World.getChunk → TACS runTasks 同步 park 等待 chunk 生成`。
- dump #2（15:49）：Server thread = tick 异常处理器打 error 日志循环（method_29741:732）。CPU 累计 2340s。
- 读图卡死 dump（16:4x）：Server thread RUNNABLE cpu=166s + Render thread cpu=125s，双线程烧在 **ItemEntity tick 实体块碰撞**（百万实体 O(n²) 直证）。
- 冻结 #2 存档 region/entities 全部 0 字节（服务器卡死 → 自动保存从未执行）→ 该轮实体证据路径断（后续在读图中途保存的存档上取到）。
- view distance 32 / simulation distance 32（用户设置；B 实验证实 sim 10 仍冻，非负载因素）。

## 已排除 / 已核对

- **dedicated server 载体不复现**：同 dll + 同 seed Chunky 4225 chunks 零异常 + forceload 1600 chunks 覆盖级联区零异常——事后归因：dev 服 `-PcppWorldgenDir` 直读权威数据，不走 Temp 解压缓存（根因即在该通路差，见根因节）。
- Rust 死锁排除（行为证据非静态推断）：冻结#2 中 Rust 调用持续返回（2375 unique chunks 推进）；fillBlocks 卡帧 60s 后同一 Worker 转空闲。
- 光照接管：默认关（需 -Dcoreswap.light.rust）。
- mixin 配置全服务端，无 client mixin。
- Chunky 载体坑：`radius` 单位是方块（81=9² chunks），须用 `chunkradius`。

## ~~当前候选机制（水位/沙类残差 → 活物理级联）~~ 【§15.4 取代：被「数据缓存陈旧 id 域错位」根因取代（supersedes）】

> 原候选（保留不改）：#26 补充案例量化的水差在实机激活后变为活物理级联。**取代理由**：双臂普查显示总差仅 0.015% 且 15 个级联点定点零差——权威数据口径下该机制量级不足；真实根因是客户端 Temp 缓存陈旧导致的整集 id 错位（见文首根因节）。原候选对「静态验证测不到动态物理」的方法论观察仍部分有效，已沉淀进 workflow-patterns #105/#106。

## 交付验证记录（judge C3：runtime/ gitignore 无 diff 源，jar 证据归档于此）

- 修复源码：runtime/1.21.6/java/src/main/java/wg/bench/CoreSwapFixHelper.java（isDataCacheStale 内容指纹）；1.20.1 同款同步修复（runtime/1.20.1/java/.../CoreSwapFixHelper.java，两文件逐字同逻辑）。
- 修复 jar（1.21.6）：build/libs/coreswap1216-1.21.6-0.1.0.jar，2026-09-10 16:44 构建，sha256 `9eae48cdfaf259457c2de1a1daa7f32f68333373ebbe582303b3ad16b9e03b08`；jar 内 worldgen-data/blocks.json sha `617c3dae…`（= 权威源逐字节一致）；jar 内 native/worldgen.dll sha `5e30187a…`（= 260910-02 vivo confirmed 执行体）。
- 1.20.1 jar 同步重构建通过（BUILD SUCCESSFUL）。
- 行为面验证：用户实机 confirmed 冻结解决（260910-03）；未做双臂回归量化（诚实声明：修复面为 Java 缓存逻辑，不动生成管线，回归风险低）。

## 产物清单（完整）

- 用户日志：.tmp/hang-repro-260910/client-{frozen-live,B-final,C-live,C-live2}.log
- 线程 dump：.tmp/hang-repro-260910/client-threaddump-{elev,elev2,c1,c2,load}.txt
- 实体普查：.tmp/hang-repro-260910/count_entities.py（item×1,031,765）
- 双臂数据：.tmp/hang-repro-260910/arms/ + diff-result.txt
- 复现/看护/双臂/forceload 驱动：.tmp/hang-repro-260910/*.ps1
- server 复现日志：.tmp/hang-repro-260910/{server,fl,arms}/*.log
- judge：.artifacts/vivo-freeze-260910-03-judge-verdict.md（PASS-with-conditions，C2-C4 已消化，C1 并入性能立项）
- 知识库草稿：.investigations/vivo-freeze-260910-03/knowledge-drafts.md
