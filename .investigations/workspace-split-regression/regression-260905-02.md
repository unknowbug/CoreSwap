# workspace 拆分行为回归记录（260905-02）

任务：验证拆分（dadc4f4）后新 dll 行为等价，MUST 门（NEXT_SESSION 0.5 ⛔ 项）。
口径声明（§9.7 三要素）：载体 = blockProbe FULL 存档写入口径 blocks 逐字节；覆盖面 = 单 seed 8576294172403134396、4×4 chunk @ (200,200)、overworld、含 carver 邻域预生成；可比性 = 与 NEXT_SESSION 既有「100.000%」口径同构（但见下「历史口径差异」）。

## 实验矩阵（每次运行前 Stop-Process java + 删 run\world + seed 核对）

| # | 执行体 | 对比对象 | 结果 |
|---|--------|----------|------|
| 1 | vanilla（无 cppReplace） | — | 参照导出（4×4@200 FULL） |
| 2 | 新 dll（target/release/worldgen.dll，sha256 2A8915…1423，00:19 构建） | #1 | 3 diff bytes / 3243362（99.99991%） |
| 3 | 新 dll 重跑 | #2 | **0 diff（100% 确定性）** |
| 4 | vanilla 重跑 | #1 | **0 diff（100% 确定性）** |
| 5 | 旧 dll（worktree @ 291bda6 拆分前源码，cargo --offline 构建 WorldgenRust.dll，-PcppLib 显式路径） | #1 | **3 diff bytes，偏移与 #2 完全相同**（42229 / 2079781 / 2080293） |
| 6 | 新 dll | #5（judge 补做直接对比） | **0 diff——新/旧 dll 逐位一致，非「差集相同」推断** |

基线锚声明：旧 dll 锚 = commit 291bda6 干净 worktree 构建（git 层面该时点无未提交源码混入，judge 核对）。

## 差异块解码（WGB2 格式逐块解析，biome UTF 段解析至 EOF 无失步）

| 世界坐标 | vanilla | rust/旧 dll |
|---|---|---|
| (198,18,198) | water | dirt |
| (237,41,224) | granite | gravel |
| (237,42,224) | gravel | water |

签名（单块水/矿物互换）与永久挂起域（aquifer 域 2 / ore 族）一致 → 既有归因 candidate，按 260904-15 拍板不重新拉起。

## 结论（candidate）

- **拆分行为等价成立**：新 dll 与拆分前源码直接构建的 dll 在同口径下逐位一致（#2 与 #5 差异集合相同 = 拆分引入差异 = 0）。构建级验证（上轮 cargo check/build 绿）+ 行为级验证（本轮）双层齐备 → 拆分升 candidate。
- **vanilla-vs-Rust 全量 100% 未达成**：3 块残差为拆分前既有，非本轮回归。NEXT_SESSION「预期 100% match」指拆分前后行为一致（已证），非 Rust-vs-vanilla 全量（该 100.000% 历史口径与本轮可比性存疑——发现 #36 补充案例同族：历史数字缺口径标注不可直接续推）。

## 执行环境

- gradle runServer（workdir=runtime/1.20.1/java，GRADLE_USER_HOME=.gradle-home，JAVA_TOOL_OPTIONS tmpdir）
- 参照/接管导出参数：-PblockProbe=1 -PblockProbeFull=true -PbenchSeed=8576294172403134396 -PbenchSize=4 -PbenchOriginX=200 -PbenchOriginZ=200 -PcppWorldgenDir=E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen
- 坑记录：workspace 根已无 settings.gradle，gradle 必须在 java 目录执行（根 Cargo workspace 后新坑）；-PcppLib 空值会 UnsatisfiedLinkError（System.load 需绝对路径），不传 = 走 jar 内嵌。

产物：.tmp/regress-260905-02/（5 份 blocks + cmp/decode 脚本）；worktree .tmp/ab-old/wt（旧 dll 构建树）。
