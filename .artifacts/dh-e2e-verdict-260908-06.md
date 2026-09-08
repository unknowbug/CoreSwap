# DH e2e 接入验证（260908-06，R1 轮）— verdict 草稿

## 任务
官方 Distant Horizons 3.2.0-b-1.20.1（Forge）接入自建客户端载具，验证远景 LOD 是否继承 Rust 接管地形（原 NEXT_SESSION 待办④）。同块附带：Voxy 线挂起决议、grove 细雪现场判定、seed/坐标三查。

## 结论（candidate，待 judge + 用户拍板）

**DH 3.2.0 INTERNAL_SERVER 模式下远景 LOD 继承 Rust 地形 —— 生成源层面闭环（behavior 级），LOD 编码内容未逐位核对（idk 降级声明）。**

### 证据链
1. **模式生效**：`config/DistantHorizons.toml` `distantGeneratorMode = "INTERNAL_SERVER"`（默认值 FEATURES 已由 toml 落盘实锤后改写）；日志零 `no target step defined for generator mode` 错误行；`InternalServerGenerator_forge` 活动路径命中（行为化，非仅配置核对）。
2. **生成源 = Rust 接管管线**（judge N-5 措辞修正：属间接行为化确认，由 Mixin 拦截计数 + region 落盘推断，日志无 InternalServerGenerator 类名字面行）：`[Mixin] populateNoise intercepted` × 7,462 chunks + `buildSurface skipped` × 7,461，覆盖远处区块（至 (49,-28)）；`[CppBridge] init seed=-546755292641445454 enabled=true stageMask=3`（log L4924/4939/4941）与 F3 屏显 seed、level.dat WorldGenSettings.seed 三处逐字一致（执行体三元组 + seed 三查全绿；judge N-2 落盘补正：level.dat 数值由 `.tmp/check_level_seed_260908-06e.py` 解析实得 -546755292641445454，F3 seed 来自用户截图 891569b9/9d8112aa 屏显行 `Seed: [-546755292641445454]`，过程输出未留档，数值以本行与截图为凭）。
3. **LOD 数据落盘**：`saves/New World (2)/data/DistantHorizons.sqlite`（overworld 37.9MB；DIM-1/DIM1 各空库）——scout idk（父目录）消解。
4. **LOD 内容抽查未做（idk）**：FullData blob 为 Zstandard 压缩（magic `28 b5 2f fd`），本机无 zstd 通道（python 库/CLI/7-Zip 全无）；用户拍板选项 A：内容抽查挂账，以降级声明收口。DH 的导入/编码正确性属 DH 自身正确性，对 CoreSwap 价值有限。

### 非阻塞观察
- **生成速率崩塌**：DH 队列 331k chunks，1,460 → 23 chunks/s（ETA 4h）——INTERNAL_SERVER 自述代价（每 chunk 全管线 + 写 region），兼容正确性 vs 吞吐取舍，非缺陷。
- **34 条 UNLOADED RuntimeException 全部在关服瞬间（12:11:10，恰好紧随 log L21336-21337 `12:11:08 Stopping server`/`Stopping singleplayer server` 之后）**——关服时未完成 chunk 请求被取消的预期形态（时序排除，Partial 级；「请求被取消」为推断而非日志直证，judge N-4 注记），非 bug。

## 附带判定
- **grove 细雪 = 机制非 bug（judge N-3 降为 draft）**：F3 Targeted Block = powder_snow；复刻侧 surface.h:525-526 噪声阈值细雪规则（Java 参照逐位复刻产物）实锤存在；vanilla grove 细雪陷阱机制先验。⚠️ 无同坐标 Java 对照，不能排除「错误位置放粉雪」——待细雪定向对拍完成后方可升 candidate。
- **Voxy 线挂起**（用户决议）：官方 Voxy 最低 1.20.4，0.2.18-beta 必为移植版，Ingest AIOOBE/shutdown hang 归因价值消失。

## 载具/流程记录
- 载具：`runtime/forge-client-test/`，运行集 = Connector + coreswap-1.0.27 + fabric-api + DH 3.2.0-b（sha1 `5667440f...` 已验）；Voxy 摘至 `mods-disabled/`。
- DH 下载经网络 escalation（沙箱 restricted token 阻断 schannel）。
- 产物：本文件 + `.investigations/mod-compat-260908-02/dh-scout-260908-06.md` + `cmd-output/`（client_run.log 1.79MB 于 `runtime/forge-client-test/`，关键行摘录见本文件）。
