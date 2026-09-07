# discovered/build-tooling — 构建/工具链坑（跨版本通用）

> 从 CoreSwap 构建与调试排查中提炼的可复用模式。写入格式见 knowledge/INDEX.md。

## 发现 #1: processResources 里 doFirst copy 不算 task input → UP-TO-DATE 跳过（dll 不重新同步）

**发现时间:** 2026-08-08 晚
**发现者:** worker（spawn 崩溃 DEBUG 顺带）
**来源定位:** build.gradle processResources（dll 同步）
**置信度:** confirmed
**module:** build

### 观察
在 task 的 `doFirst` 里 copy 文件（native dll 同步），gradle **不把它算作 task input**——dll 更新后 gradle 判定 UP-TO-DATE 直接跳过，进程仍加载旧 dll（sha 不匹配排查半天）。

### 证据
- processResources doFirst copy 旧 dll → 服务器加载旧 dll（sha256 校验才发现）
- 规避：手动 Copy resources 或 `--rerun-tasks`

### 如何利用
- 文件同步必须声明为 task input/output（gradle 可见），或在源文件变化时主动使 task 过期；兜底 `--rerun-tasks`
- dll 打包/运行前后做 sha256 校验（worldgen.dll 对齐铁律），UP-TO-DATE 跳过是常见坑源

## 发现 #2: gradle daemon env 缓存——fork 的 JVM 继承 daemon 启动时的 env，不是调用时的 env

**发现时间:** 2026-08-08 晚
**发现者:** worker
**来源定位:** gradle daemon + `$env:CORESWAP_THREADS`
**置信度:** confirmed
**module:** build

### 观察
给 `gradle runServer` 传环境变量（如 `$env:CORESWAP_THREADS`）不重启 daemon 不生效：fork 出的 JVM 继承的是 **daemon 启动时**的 env，不是本次调用时的 env。

### 证据
- 设 `$env:CORESWAP_THREADS=1` 后 runServer 仍用旧线程数 → `gradle --stop` 重启 daemon 后生效
- 用 `-P` 属性（vmArg 映射）传递不受 daemon 缓存影响

### 如何利用
- 给 gradle 运行的 JVM 传自定义值：优先 `-P<name>=<value>` + build.gradle 里映射为 vmArg；不要依赖 shell env（daemon 缓存）
- 改了 env 记得 `gradle --stop` 或重启 daemon 再验证

## 发现 #3: gradle 8.13 `-D` 参数解析——`gradle runServer -Dcpp.replace=1` 被拆成任务名

**发现时间:** 2026-08-08 晚
**发现者:** worker
**来源定位:** gradle 8.13 命令行解析
**置信度:** confirmed
**module:** build

### 观察
gradle 8.13 不把 `-Dcpp.replace=1` 当 Java 系统属性传给任务；整串被解析成任务，报 `.replace=1 not found`（任务不存在）。

### 证据
- `gradle runServer -Dcpp.replace=1` → `.replace=1 not found` 任务错误
- 改为 build.gradle 的 `-PcppReplace=1` → vmArg 映射 `-Dcpp.replace=...` 生效

### 如何利用
- gradle 命令行传自定义参数统一用 `-P<name>=<value>`（project 属性），在 build.gradle 里映射为 JVM 参数（vmArg）；不用 `-D`

## 发现 #4: gradle home 放工作区内（GRADLE_USER_HOME 指向项目 .gradle）→ native-platform.dll + 依赖缓存在沙箱区内，免提权

**发现时间:** 2026-08-29
**发现者:** knowledge subagent（运行环境迁移里程碑）
**来源定位:** b2b9bea（git mv versions/1.20.1/java → runtime；gradle home C:\Users\NDark\.gradle → CoreSwap\.gradle）
**置信度:** candidate
**module:** build / env

### 观察
gradle 默认 home（C:\Users\...\.gradle）位于项目外部，其 native-platform.dll（gradle 原生平台库）与依赖缓存不在沙箱/受限工作区内 → 每次 gradle 运行需提权。把 gradle home 整个迁到工作区内（robocopy/copy 现有 home + 设 GRADLE_USER_HOME=项目\.gradle）→ 原生库 + 依赖缓存在可见区内，gradle 免提权（classes 编译 / runServer / 探针全通过）。

### 证据
- 迁移前：gradle 运行需 danger-full-access（native-platform.dll 在 C:\Users\NDark\.gradle 外部）。
- 迁移后：GRADLE_USER_HOME=CoreSwap\.gradle（robocopy 秒级复制 2.6GB home）+ runtime 目录 → gradle 全免提权（b2b9bea + 50ba9a4）。
- 关键坑：junction 把 caches 链到外部会被沙箱拒写（journal-1.lock 拒绝访问）——**依赖缓存必须真正在工作区内**，不能 junction 外部；跨盘复制 2.6GB 秒级（robocopy/MT）。

### 如何利用
- 沙箱/受限环境跑 gradle：把 gradle home 迁进工作区（复制现有 home 避免重下依赖），GRADLE_USER_HOME 指向之 → 免提权。
- 配合把验证工程作为独立 runtime 目录（versions/ 回归纯数据/参考，runtime/ 承载运行/验证环境）——运行环境与数据/参考分离，职责清晰。

# 草稿：knowledge/discovered/build-tooling.md 追加「发现 #5」（subagent 产出，主会话应用）

> **应用位置**：`knowledge/discovered/build-tooling.md`——「## 发现 #4」之后（文件末尾）追加。追加不覆盖。写后同步 INDEX.md。
> 现有编号核对：build-tooling.md 当前至发现 #4，本条为 **#5**。

---

## 发现 #5: 自研/手写 JSON 解析的布尔字段走数值读取接口 → `unwrap_or` 默认值静默生效

**发现时间:** 2026-08-30 深夜
**发现者:** worker（多世界收尾 M6，Rust worldgen）
**来源定位:** WorldgenRust json.rs `as_f64()` + worldgen_handle.rs aquifers_enabled 读取（错误台账 M6：`.investigations/multiworld-port/multiworld-errors.md`）
**置信度:** candidate
**module:** build / config-parsing

### 观察
配置 JSON 写的是布尔（`"aquifers_enabled": false`），读取代码却走数值接口：`settings.get("aquifers_enabled").and_then(|v| v.as_f64()).map(|x| x != 0.0).unwrap_or(true)`。自研 parser 的 `as_f64()` 只匹配 Number——**Bool 恒返回 None** → `and_then` 链断 → **`unwrap_or` 的默认值静默生效**，且默认值方向与 JSON 真实值相反（false → true）。字段不是「缺失」而是「在但类型读不到」，却按缺失处理。后果：下界被错误启用真实含水层（6.7 万块水 vs vanilla air），match 卡 74.04%；同款坑还埋了 `legacy_random_source`（legacy 分流从未激活）和 `requires_block_below` 两个字段。

### 证据
- 修前：nether match 74.04% 卡住；y32..63 带仅 7.9% 纹丝不动；legacy_random_source 加了读取逻辑后零效果（多字段聚簇）。
- 判错路径：混淆对直方图（got→want Top 配对）暴露 id32=water 聚集 → skip 开关二分锁 stage 1（fill）→ 反查 classify 分支条件反推 enabled 状态错误 → 下钻 JSON 解析层发现 as_f64() 对 Bool 恒 None。
- 修后：json.rs 加 `as_bool()`（Bool 直读；Number 兼容 !=0），三处读取改 as_bool → nether **74.04% → 82.69%**，overworld 95.40% 零回归。

### 如何利用（通用判据 + 通用修法）
- **通用判据**：任何「optional 读取 + unwrap_or 默认值」链的默认行为必须**显式验证类型**——新 JSON/配置字段接入时验证「读到的是什么」（读取后打一行日志或 assert 类型），不是验证「默认值是什么」。字段类型不匹配被静默吞成默认行为，是该反模式的通用形态（任何 self-parsed JSON/配置——Rust/Java/C++/手写 parser——都会踩，不限 MC）。
- **通用修法**：parser 提供类型化读取接口（`as_bool`/`as_int`…，Bool 直读 + 数值兼容 !=0），读取处用匹配的类型接口；多配置字段同时「写了没反应」是解析层错的聚簇签名，先查共同解析层不逐字段查逻辑。

## 发现 #6: fs::copy 保留 mtime——复制链产物判新旧用内容指纹，不用时间戳（260902-01）

- **发现时间**：260902-01（E9，nether-save-full 课题）
- **置信度**：confirmed 级机制（语言/OS 层行为），案例 candidate
- **module**：build-tooling / rust
- **观察**：WorldgenRust.dll 经 `fs::copy` 部署，mtime 显示 9/1 实为最新构建——fs::copy 保留源文件时间戳，mtime ≠ 生成时刻。
- **证据**：二进制字符串探测（C1 特征串在「旧 mtime」文件中）证明内容为最新；cargo 全 fresh 与 mtime 矛盾。
- **如何利用**：①判产物版本 = 内容指纹（二进制字符串探测/哈希），mtime 只作线索；②「复制即部署」链路默认不信任产物时间戳；③需真实生成时间时复制后显式 `File::set_modified` 或内嵌构建戳。


## 发现 #7: gradle 全套状态（native 锁/daemon）都在 GRADLE_USER_HOME——沙箱下指到工作区即可绕开 home 目录权限（260902-02）

- **发现时间**：260902-02（E10，nether-save-full 课题）
- **置信度**：candidate
- **module**：build-tooling / env

### 观察

强杀 gradle daemon（java 进程）后，所有 gradle 调用报 `Failed to load native library 'native-platform.dll'`。--stacktrace 显示根因不是 dll 本身，而是 `C:\Users\NDark\.gradle\native\**\native-platform.dll.lock` **锁文件拒绝访问**——daemon 被杀时锁未释放，且锁文件位于工作区外的 home 目录，沙箱下删除被硬拒（升级亦被拒）。最终修复：`GRADLE_USER_HOME` 指向工作区 `E:\PYTHON\CoreSwap\.gradle-home`——gradle 全套可变状态（native 锁、daemon 目录、依赖缓存）都在 GRADLE_USER_HOME 下，指到工作区即整体绕开 home 目录权限问题。

### 证据

- --stacktrace 定位到 `.lock` 文件级拒绝（非 dll 损坏）；
- 删锁：沙箱拒绝工作区外写（danger-full-access 升级亦被拒）；
- GRADLE_USER_HOME=E:\PYTHON\CoreSwap\.gradle-home 后 gradle 调用恢复；
- 与发现 #4 同族互证：#4 迁 home 免提权（依赖缓存须真在工作区内，不能 junction 外部）；本条补齐「锁/daemon 状态」同样受制于 home 位置——**同一机制（GRADLE_USER_HOME 决定全部可变状态位置）的两个表现面**。

### 如何利用

- **沙箱下杀 java daemon 前先想锁文件**：daemon 非正常退出会留下 native-platform.dll.lock，锁在工作区外则无法清理——预防优于修复。
- **gradle 全套状态（native 锁/daemon/依赖缓存）都在 GRADLE_USER_HOME**：沙箱/受限环境第一步就把 GRADLE_USER_HOME 指到工作区，一次性规避 #4（提权）与本条（锁权限）两类坑。
- 配套教训（同课题 E10 过程事实）：**nether 回归完整命令的参数须与参照文件名四要素一致**（cppReplace + readWorldProbe + blockProbeDimension=nether + bench 参数）——本次 run2/run3 两次因参照不匹配空跑；完整命令模板以 `.investigations/nether-save-full/cmd-output/flags-regression-run4.log` 对应调用为准，与 AGENTS.md「参照文件名内嵌 seed」纪律（操作环境纪律 #9）同族：**跑对比前先核对命令参数 ↔ 参照文件名逐项一致，防止空跑烧轮次**。
- 配套简记（260902-04，v5-residual 轮）：残留 java 进程会占 session.lock 导致重跑**静默失败**（无明确报错指向锁）——gradle runServer 类调用失败先 `Stop-Process -Name java` 清残留再重跑（AGENTS.md「残留 java 进程」铁律的 session.lock 表现面）。

## 发现 #8: gradle -P 属性手工映射 → -D vmArg——新系统属性必须同步加映射行，否则静默不生效（260902-04）

- **发现时间**：260902-04（V5 残差排查）；**置信度**：candidate（三犯实锤）；**module**：build-tooling / gradle。

### 现象

Java 探针工程新增系统属性开关（本轮 `biome6.points` / `biome6.cellDump` / `biome6.colDump`）后，命令行 `-Pbiome6.points=...` 传入，探针侧读不到——前两次（points / cellDump）均静默无效、空跑烧轮次，第三次（colDump）才提前防住。

### 根因

`build.gradle` 对 `-P` 项目属性到 `-D` JVM 系统属性的传递是**手工枚举映射**（逐行 `if (findProperty) run.vmArg "-D..."`）——新增系统属性若忘了在映射清单加一行，属性停在 gradle 侧进不了 JVM，**无任何报错**（findProperty 侧缺省静默为 null）。

### 定位

探针输出缺对应 dump/无属性生效迹象 → 反查 build.gradle 的 -P→-D 映射清单，发现新属性名不在清单内。

### 修复

build.gradle 映射清单补对应行（每新增一个系统属性同步加一行）。

### 教训

- **判据**：「-P 传了但程序里读不到/没效果」且无报错 → 第一反应查 build.gradle 的 -P→-D 手工映射清单，不查代码逻辑。
- **结构修法建议**：映射清单改为遍历一批约定前缀（如 `project.properties.findAll { it.key.startsWith("biome6.") }` 批量 vmArg）消除逐行枚举的遗漏面——未落地，暂以纪律约束（新增属性即同步加映射）。
- 同族：AGENTS.md 操作环境纪律「参数 ↔ 参照逐项一致防空跑」——配置传递链上的静默丢弃（无报错 + 无效果）都要靠清单核对防，不靠运行时暴露。

## 发现 #9: gradle runServer 传 CLI --nogui 必失败 + -P 属性缺映射行静默不生效（260902-09）

- **现象**：`gradle runServer --nogui` 报「Unknown command-line option」失败；`-PrustStages=7` 传了但 JVM 侧读不到（coreswap.rust.stages 为空走默认 0b011）。
- **根因**：--nogui 是 build.gradle `programArgs` 注入的 server 参数而非 gradle CLI 选项；-P→-D 靠 build.gradle 手工映射行，rustStages 行曾缺失即静默丢弃。
- **定位**：看 gradle 失败原文（选项级报错即刻暴露）；属性类查 build.gradle 映射清单逐项比对（dry-run 看不到 vmArg，权威核验点 = JVM 侧日志打印属性值）。
- **修复**：--nogui 从 CLI 去掉（programArgs 已有）；build.gradle 补 `rustStages` 映射行；补后 `gradle --stop` 防 daemon 缓存（#8 三犯）。
- **教训/如何利用**：gradle run* 任务自定义参数先看 build.gradle 三层接线（CLI 选项/programArgs/-P→-D 映射）再传，勿按直觉传。与发现 #8 同族，本条补 runServer + programArgs 场景。


## 发现 #10: 参照文件核对四要素不够——文件名不含 stage，SURFACE 参照被当 FULL 用贯穿多轮；判据升级五要素 + 内容指纹（260902-10）

- **发现时间**：260902-10（amplification 课题）；**置信度**：confirmed（260902-10 用户拍板；judge 0 BLOCKER 曾建议 candidate）；**module**：re-code。

### 现象

历史「~3.4% 真实存档残差」（run3-6，96.6215% 口径）参照 = `versions/1.20.1/data/vanilla_8576294172403134396_4_3200_3208_nether.blocks`（sha256 02b94092f917cb5d）——文件名四要素（seed/size/origin/dim）核对全过，但该文件实为 **SURFACE 阶段参照**。FULL 存档 vs SURFACE 参照 → feature/carver 产物（矿石 417/607/45、cave_air 730、basalt blob）全被计为失配 → 伪残差 3.4%，并引出「feature/carver 放大假设」整条错误方向（贯穿 M16→V5 多轮：96.62% / 13.7% / 22.5% / 3.4% 同一污染链）。同轮还踩 benchOriginX/Z 是块坐标非 chunk 坐标（chunk 3200 区要传 51200/51328，wx=origin/16+cx）。

### 根因

参照文件核对判据缺**阶段（stage）**维度——文件名只含 seed/size/origin/dim 四要素，SURFACE 参照与 FULL 参照在文件名上不可区分；而两者内容差异巨大（feature/carver 产物只在 FULL 产物出现）。四要素核对通过 ≠ 参照口径正确；跨阶段对比的差异量天然等于两阶段产物差，任何归因结论都建立在伪残差上。

### 定位

三方判别法：同一区域 fresh vanilla FULL vs old ref = 20.4538% 失配（old ref 缺 feature 产物），fresh vanilla vs cppReplace = 0.0000% → 异常收敛到 old ref 一侧；再对 old ref 做内容指纹（阶段特征 id 有无）定性为 SURFACE 参照。证据：.tmp/amp_step3_region200.out.txt（20.45%）、amp_step4_crosscheck.out.txt（0.0000% + top pairs 独立佐证）、amp_step2_join.out.txt（同域重测 16/1048576，放大系数 0.62 < 1 不存在）。

### 修复

参照文件核对判据升级**五要素**：seed / size / origin / dim / **stage**。文件名不含 stage 时用**内容指纹**判定：阶段特征 id 有无——nether 矿石（417/607/45）、cave_air（730）、basalt blob 族只在 FULL 产物出现，全无即 SURFACE 参照。对比前先定性两侧阶段同源，再谈失配率归因。

### 教训/如何利用

- **判据**：拿到任何参照 .blocks 文件，第一动作不是跑对比，是按五要素核对——四要素对上后必须补一步内容指纹验 stage（grep 阶段特征 id 计数）。
- **伪残差签名**：失配率量级与「某生成阶段的产物量」同阶（如 ~3.4% vs feature 覆盖率），且失配块集中在该阶段产物 id 上 → 先怀疑跨阶段参照，不怀疑实现差。
- 附记（同课题坐标坑）：benchOriginX/Z = 块坐标（wx=origin/16+cx），chunk 3200 区传 51200/51328——采集命令与参照 origin 核对用同一单位。
- 同族：workflow-patterns #4（参照状态三查，阶段同源意识）/#14（探针阶段同源性）；本条补齐「文件级参照的阶段指纹核对」操作判据。上游结论见 `.artifacts/b1-candidates/amplification-verdict-260902-10.md`。

## 发现 #11: header/文件名本身也可能是错的——参照核对以内容实测坐标为准 + 探针带恒等式自检（260903-02）

- **发现时间**：260903-02（lossless-accel 课题 P0-① 探针踩坑，五段式见 `.investigations/lossless-accel/lossless-accel-errors.md` LL2）；**置信度**：draft；**module**：re-code/swe 通用。

### 现象

参照文件 `vanilla_..._4_-288_-256_FULL.bak.blocks`（E:\python\MC\data\）文件名与 header origin 均为 (-288,-256)，Python 直读二进制实测内容 chunk 坐标为 (-18..-15, -16..-13)——header origin 字段与内容不符，文件名同被误导。按 header 配对的探针报告「match 差 12321 块」与同运行内「分解计数差 0」自相矛盾。

### 根因

header origin 是导出工具写入的**声明**，不是数据的**实测**——写 header 的代码与写 chunk 的代码可能不同源/不同步。五要素核对（#10）核对的是声明字段，声明本身可漂移；跨工具 chunk 配对以声明为键即产生假配对，差异被归因到错误一侧。

### 定位

python 直读二进制逐 chunk 打印坐标：header 32 字节（magic u32 + seed i64 + size/ox/oz/minY/height 5×i32）；每 chunk = 8B 坐标 + bpc*2 blocks + 256 个 u16 前缀计数的变长 biome 段。实测内容坐标 vs header 声明即暴露不符；同运行内恒等式自检（match 差 ≠ 分解计数差）一次即确认假配对。

### 修复

跨工具 chunk 配对/对比一律以**文件内容实测坐标**为键，header/文件名/注释仅作线索不作判据；探针必须内置恒等式自检（配对 match 差 ≡ 分解计数差，违反即报假配对拒绝出数）。

### 教训/如何利用

- **判据再升级**：五要素核对声明字段 + 内容指纹验 stage 之外，**声明字段本身也要与内容实测交叉验证**——「字段说 X」≠「数据是 X」。
- 恒等式自检是识别假配对的最廉价手段：同一数据两种独立口径必须相等，不等即配对/坐标系出错。
- 历史对比未污染的原因：handle_probe 用文件内坐标生成对比侧（自洽）——侧证「内容实测键」天然免疫 header 谎报。
- 同族：workflow-patterns #13/#16；上游：build-tooling #10（本条为其第二次升级）。

## 发现 #12: 二进制产物（.spv 等）无法从内容判断新旧——生成器多产物重生成必须整体原子更新，「逐位一致」哨兵结论须配已知值哨兵点（260903-04）

- **发现时间**：260903-04（lossless-accel 路线② FFI 工作包）；**置信度**：candidate（根因经双 seed 重编复现闭环，judge 待过）；**module**：swe/build 通用。
- **来源定位**：GPU final_density pipeline 差异排查；证据 = tri-cut2/3 切分输出（.investigations/lossless-accel/cmd-output/）+ git 提交时间戳（cc58e05 08-15 19:21 / 9de661e 19:22）+ spv mtime 08-15 14:17。

### 现象

GPU 密度引擎 vs DFC-CPU oracle 6144 点 f32_exact 仅 43.26%、max_diff 0.5533——系统性 diff 非纯精度；tri-cut 证明 FFI/Rust 侧无罪、C++ CPU 与 GPU 自身 major diff（最大 0.502）。已知值哨兵点 (784,160,-408)（历史验证 seed）GPU 输出 0.0453032888——正是时间线 L1386 记录的 D23 修复**前**错误值（正确 -0.458333343）。而最终 density 源码、cpu_backend.h 均为 D23 修复后版本。

### 根因

`final_density.spv` 是 D23 修复**前**编译的陈旧产物：mtime 08-15 14:17 早于修复提交 cc58e05（08-15 19:21）5 小时，commit 9de661e（19:22）提交的 spv 是修复前编译的；08-23 `final_density.comp` 与 cpu_backend.h 同批重生成，但 **spv 不随之自动重编**（glslc 编译步骤脱节）——生成器多产物（comp / cpu_backend.h / spv）部分更新造成跨产物语义失配。机制层面：① **二进制产物无法从内容判断新旧**；② **mtime 与提交时间新鲜度均具误导性**——mtime 与提交时间各看都对，合起来才是「产物早于修复」；③ 教训⑧（对账必须基于当前生成产物）针对 dump 对账域，本案升级为**部署产物本身陈旧**。

### 定位

决定性手段 = **已知值哨兵点**：(784,160,-408) 在历史验证 seed 下应输出 -0.458333343（DF_SQUEEZE clamp -1 饱和值），实测 0.0453032888 与时间线历史错值逐位吻合 → 直接锁定「旧语义产物」而非引擎 bug。辅以 tri-cut 同程序同坐标双路切分（排除 FFI/Rust/坐标/seed 错位）+ git 时间戳与 mtime 交叉（5 小时窗）。重编（gen_final_density.py → glslc → 部署，旧 spv 备份 .bak-pre-d23）后双 seed 23 点 major_diff=0、6144 点 max_diff=9.18e-6——闭环。

### 修复

① 重编 spv 并部署（旧产物备份）；② 判据固化：**生成器多产物（源模板/生成头/spv 二进制）重生成时必须整体原子更新**——改了任何一个生成输入，所有下游产物同批重编，构建脚本应把 spv 编译纳入与 comp/backend.h 同一入口；③ **任何「逐位一致 maxDiff ~e-07」类哨兵结论必须配一个已知值哨兵点做产物健康检查**——哨兵点的值域应含饱和/边界值（如 clamp -1 的 -0.458333343），饱和值丢失 = 产物语义级陈旧的即时签名。

### 教训/如何利用

- **判据**：拿到任何二进制生成产物，先问「它编译于哪次源状态」——mtime/提交时间/内容都答不了；直接跑已知值哨兵点，一测便知。
- **哨兵结论的反模式**：「同引擎 chunk(0,0) 全对 ≤7e-8」这类逐位一致只证明「该域内新旧产物恰好语义相同」，**一致域外产物可能陈旧**——哨兵点必须覆盖历史修过的错误签名域（负 chunk/饱和值）。
- **家族谱系**：教训⑧（dump 对账须基于当前产物）→ 本条 #12（部署产物本身陈旧 + 提交新鲜度误导）；同文件 #6（mtime 不可靠→内容指纹）、#10（声明字段核对）、#11（声明 vs 内容实测）——共同上位原则：**「看起来对」的元数据一律不作产物健康判据，用可复现实测值验**。
- 上游主记录：`.investigations/lossless-accel/route2-ffi-260903-04.md`（根因闭合节）。

### 环境坑补记（260903-08，runtime 路径迁移事实）

- **事实**：Java runtime 现位于 `E:\PYTHON\CoreSwap\runtime\1.20.1\java\`（gradle runServer + `GRADLE_USER_HOME=E:\PYTHON\CoreSwap\.gradle`）；`E:\PYTHON\MC\versions\1.20.1\java` 是**迁移前废弃目录**——260903-08 session 误访问一次（env-check 在案，无损害）。
- **判据（避免再犯）**：① 交接文档写「未动/位置在 X」必须带绝对路径，目录迁移后须在旧位置留转发注记或删除废弃目录——「路径惯性」（按记忆路径访问）是迁移后首犯高发位；② 任何 runtime/工具链路径使用前先 `Test-Path` + 核对版本标记，不靠路径记忆。


## 发现 #13: gradle「Failed to load native-platform.dll」#7 同族复现——GRADLE_USER_HOME 未指工作区即复发，修复 = 显式指向工作区 .tmp 下（260903-12）

- **发现时间**：260903-12（实际 2026-09-03 晚，锚 git 260903-12 提交簇）；**置信度**：candidate（同族复现实锤，本 session 修复即闭环）；**module**：build-tooling / env。

### 现象

本 session gradle runServer（Java est dump 探针侧）报 `Failed to load native-platform.dll`——与发现 #7（260902-02）同族。触发条件 = 新 shell/新 session 未继承 GRADLE_USER_HOME（默认落到 home 目录），或 daemon 强杀后锁文件残留。

### 根因

gradle 全套可变状态（native-platform.dll 及其 .lock、daemon 目录、依赖缓存）都在 **GRADLE_USER_HOME** 决定的目录下；该变量是 per-shell env，新 shell 不设即回退 home 目录 → 沙箱下 home 不可写/锁不可删 → 崩在「加载原生库」这个最外层症状上（#7 已定机制：根因是 .lock 拒绝访问，非 dll 本身）。

### 定位

报错原文先查三处：① `$env:GRADLE_USER_HOME` 是否为空/指向外部；② `.lock` 文件位置（--stacktrace 显示路径）；③ 残留 java 进程占用（`Stop-Process -Name java` 先清）。

### 修复

`$env:GRADLE_USER_HOME` 显式指到**仓库工作区内**（本 session 约定：`.tmp` 下，如 `E:\PYTHON\CoreSwap\.tmp\gradle-home`；#4/#7 用的是 `.gradle`/`.gradle-home`——位置不关键，**在工作区内**才关键），新 shell 每次都要设。

### 教训

- **判据（第三次复现后固化）**：「Failed to load native-platform.dll」第一反应不是查 dll，是查 GRADLE_USER_HOME——它是 per-shell env，不设必复发；**预防性设置应写进每轮 Java 侧采集的标准前置命令**（与 #4 免提权、#7 锁权限同一机制的第三表现面）。
- 同族：#2（daemon env 缓存）、#7（锁文件权限）——gradle env/状态类坑合订：**gradle 的全部可变状态位置由 GRADLE_USER_HOME 决定，全部状态都该进工作区**。

---

## 发现 #14: runServer 主线程预生成大 region（64 chunk）触发 watchdog 60s 强杀；探针 dump 文件必须内嵌 seed 头（260903-12）

- **发现时间**：260903-12（实际 2026-09-03 晚）；**置信度**：candidate（crash-report + 修复复跑闭环）；**module**：build-tooling / 探针工程。

### 现象

runServer 预生成 64×64 大 region 时 watchdog（`max-tick-time` 默认 60000ms）60s 强杀主线程，服务器崩溃退出。crash-report：`run\crash-reports\crash-2026-09-03_21.10.10-server.txt`。同一 session 附带发现：本批探针 dump 文件（estopt-ab-arms-p0 / 三份 CSV）头内**无 seed 字段**——seed 一致性靠 session 流程与旁证互推，未内嵌（judge CONCERN-C1）。

### 根因

① 主线程单次 tick 内连续生成数百 chunk，单 tick 耗时 >> 60s → watchdog 判死锁强杀——这是**大 region 预生成的结构性行为**，不是死锁；探针运行时不需要 watchdog 保护。② dump 工具写文件时不回显输入参数（seed），违反「seed 三处核对」铁律的落盘化要求——原始文件自身不可自证 seed，事后核对只能靠旁证。

### 定位

① crash-report 直接指名 watchdog（`Considering it to be crashed, server will forcibly shutdown`）；② judge 审查 P0 原始输出时逐文件查 seed 字段发现缺失（CONCERN-C1）。

### 修复

① `server.properties` 设 `max-tick-time=-1`（禁用 watchdog；**仅限探针/预生成运行时**，常规实机运行不改）；② 探针 dump 工具在文件头/行头内嵌 seed（`# seed=...`）——每份原始输出自证 seed，核对铁律从「流程保证」升级为「文件自保证」。

### 教训

- **大 region 预生成三件套前置**：删 `run\world`（#19）+ 清残留 java 进程 + `max-tick-time=-1`——缺一即烧轮次。
- **探针 dump 文件头自证原则**：dump 文件 MUST 内嵌 seed/origin/口径头，使「seed 三查」可以在文件本身上完成（与 #11「header 也可信不过」互补：#11 管声明字段要实测交叉验证，本条管**声明字段必须先存在**——两道关卡都过，声明才可用作线索）。
- 同族：judge CONCERN-C1（est-shared-verdict 审查）；AGENTS.md seed 三查铁律的落盘化延伸。


## 发现 #15: gradle run 存档口径照抄历史 run 完整参数清单——裁剪属性列表会裁掉历史踩坑后的必带项（-PcppWorldgenDir）（260903-14）

- **现象**：不带 `-PcppWorldgenDir` 跑 `-PcppReplace=true -PreadWorldProbe=true`，server started 即抛 `IllegalStateException: worldgen-data not found in mod resources`（CoreSwapFixHelper.extractWorldgenDir:48），服务器立即停止。
- **根因**：jar 内资源布局 `worldgen-data/{minecraft, blocks.json, …}`（minecraft 直下），而 marker 检查路径是 `wgDir/data/minecraft/worldgen/noise_settings/overworld.json`（多一层 `data/`）——资源解压路径与 marker 路径两条布局约定不同步，解压分支的 marker 永远不存在 → 必然二次抛异常。**解压路径本身是死路**，历史 run 全部靠显式 `-PcppWorldgenDir=<工作区 data/worldgen>` 绕过解压。
- **定位**：读 CoreSwapFixHelper.java marker 路径 + `Get-ChildItem src/main/resources/worldgen-data` 对照布局；再查历史 run 日志确认全部显式传参绕过——「为什么历史没炸」的答案是历史从来没走过解压分支。
- **修复**：run 命令补 `-PcppWorldgenDir=...`（workaround）；资源布局与 marker 不一致未改，列为升级点。
- **教训**：**跑存档口径 run 照抄历史 run 的完整参数清单，不要凭 build.gradle 属性列表自行裁剪**——属性列表只声明「存在」，不声明「必带」；裁掉的可能是历史踩坑后的必带项。同族：#8（gradle -P→-D 映射遗漏静默不生效）、#9（缺映射行静默不生效）——本条补「不能反向从属性列表推断可省略项」维度。根治方向（升级点）：marker 路径与资源布局对齐，或解压失败 fail-fast 时提示带 `-PcppWorldgenDir`。

## 发现 #16: 「绕过项永远在用的分支 = 死分支」信号——#15 根治复盘：解压死路主因是资源集不完整而非布局；bin-diag 旧 exe 假阴性——探针用前必须核产物时间戳（260903-15）

- **发现时间**：260903-15；**置信度**：candidate（judge PASS with should-fix 已清偿，Full 层验证闭环）；**module**：build-tooling / 资源打包 + 二进制产物新鲜度。

### 现象

① 发现 #15 记载的 `-PcppWorldgenDir` 死路（解压 marker 永远不存在）按当时根治方向修复：routeRel 布局双兼容（`data` 开头 → wgDir 原版布局；`minecraft` 开头 → `wgDir/data/` 拼接旧布局）——修复后解压成功，但 `noise_settings/overworld.json` **仍不存在**，解压产物 `worldgen/` 下只有 `biome/`（68 文件）。② 同 session 负向测试（删 noise key 验证启动断言生效）连续两次"未触发 panic"——改了源码重编后探针行为完全不变。

### 根因（为什么错）

① **#15 的根因记载不完整**：mod 资源 `worldgen-data/` 里**根本没有 noise_settings/density_function 等完整数据集**（完整权威集 = `versions/1.20.1/data/worldgen` 845 文件）——marker 指向的文件在 jar 里结构性不存在，布局只是次因。路由修好等于把路修通到一片空地。② `cargo build --release` **只编译 `src/bin/`，不编译 `src/bin-diag/`**（后者按临时区纪律特意隔离出默认构建，AGENTS §八.13）——estopt_ab.exe 是前一天的旧产物，静态链接旧 lib 代码，源码改动对它完全无效。两次"未触发"全是假阴性。

### 定位（怎么发现的）

① 解压产物逐层列目录 + 对照权威目录 `versions/1.20.1/data/worldgen`（845 vs 68 文件清点）。② `Get-Item exe | LastWriteTime`——时间戳早于本次改动即穿帮（发现 #6 内容指纹判据的时间戳变体）。

### 修复

① 资源整体重排：`src/main/resources/worldgen-data` = 权威 `versions/1.20.1/data/worldgen`（自带 data/ 层，845 文件）+ 顶层 4 json；routeRel 保留双兼容路由（旧布局目录用户可指 data/ 层目录）；fail-fast 报错 2 处补 `-PcppWorldgenDir` 绕过提示。② 按 bin-diag 单编纪律 `rustc --edition 2024 ... --extern WorldgenRust=target\release\libWorldgenRust.rlib -o target\release\estopt_ab.exe`。

### 教训

- **「绕过项永远在用的分支 = 死分支信号」**：历史全靠 `-PcppWorldgenDir` 绕过的解压分支，本身就提示该分支从未工作过——修 root cause 前先确认**分支的输入数据是否存在**，再修路由/布局/逻辑（次因）。
- **文档记载的根因要验证到"能闭合"为止，不能到"能解释"为止**——#15 的解释（布局不一致）能自洽但修完不闭合（E3）；修复闭合才是根因完整的唯一证明。
- **bin-diag 探针每次用前必须单编或核产物时间戳**：`cargo build --release` 不触达 bin-diag；「我编译过了」不是产物新鲜度证据（时间戳/哈希才是）。错误签名「改了源码但探针行为不变」先查这个。
- 同族：#6（fs::copy 保留 mtime——产物判新旧用内容指纹）、#8/#9（映射遗漏静默不生效）；AGENTS §八.13 bin-diag 隔离纪律的配套判据。

---

## 发现 #17: block_probe 调用契约三坑——argv[3] 必须 vanilla 参照、wgDir 必须含 data/minecraft/、-save 导出为无 biome 段格式（260904-04）

- **发现时间**：260904-04；**置信度**：candidate（多次踩坑复现实证）；**module**：build-tooling / block_probe 调用契约。

### 现象

① argv[3]（参照 blocks 文件）传了非 vanilla 参照或路径错 → 对比结果整段错位/全红；② wgDir 传了不含 `data/minecraft/` 层的目录 → **未捕获异常 0xE06D7363 且 crash 栈无任何消息**（裸终止，无从下手）；③ `-save` 导出文件 **3145888B vs Java 参照 3243362B**——导出为无 biome 段格式，若下游 cmp 脚本按 `len≥3243362` 判「格式正确」会把缺段格式误判为合法，进而整段错位而不报错。

### 根因（为什么错）

① 参照语义要求是 vanilla 全程产物，混入其它载具/阶段产物即坐标语义不齐；② wgDir 布局契约 = `data/minecraft/worldgen/...`，探针直接按该层拼路径，缺层即抛未包裹异常（MSVC EH 异常码 0xE06D7363）；③ 导出器与 Java 导出器**格式不互为超集**，文件大小差异是格式差不是损坏——拿 Java 侧大小当格式校验阈值是错误不变量。

### 定位（怎么发现的）

② 由崩溃码 0xE06D7363（MSVC throw）+ 栈无消息 → 直接查入口参数校验路径试出布局要求；③ 由导出/参照字节数对照发现恒差一段，追导出器写出的 section 列表确认无 biome 段。

### 教训

1. block_probe 调用前置三查：argv[3]=vanilla 参照、wgDir 含 `data/minecraft/`、下游校验用**格式签名（段列表）不用字节数阈值**。
2. 「未捕获异常 + 无消息栈」先查输入契约再查逻辑——探针入口参数应有显式校验与报错（升级点）。
3. 同族：本文件 #10/#11（参照核对五要素/内容指纹）、workflow-patterns #17（打印坐标≠采样坐标）——本条补「调用参数契约」维度。


## 发现 #18: 导出产物在盘 ≠ 本次执行体生成——重导命中旧 world 缓存（chunk "FULL in 0-1ms" vs pregen ~24s），判别签名 = 每chunk耗时 + pregen 时长（260904-08）

- **发现时间**：260904-08；**置信度**：candidate（实机 A/B：首次全新生成 vs 重导双载体对照）；**module**：build-tooling / 探针导出 + 产物新鲜度。

### 现象

BlockProbe 重导（未删 run\world）导出顺利完成、产物落盘，但对比侧表现异常。每 chunk 耗时对照：重导时 chunk 打印 "FULL in 0-1ms"，而首次全新生成（删 world 后）有 pregen 阶段 ~24s + 每 chunk 实际生成耗时——重导命中旧 world 缓存，导出的是旧执行体当时生成的产物，不是当前新执行体的产物。「导出成功 + 文件在盘」完全掩盖了「执行体未参与本次生成」。

### 根因（为什么错）

导出链路 = 世界生成（受 run\world 存档缓存支配）→ 读取落盘（成功）。两段解耦：只要存档里有现成 chunk，生成阶段整个被缓存跳过，新编译的执行体代码根本没跑。「文件时间戳新」只证明「这次写了文件」，不证明「这次生成了数据」——与 #19（重导前删 run\world）是同一坑的操作面与判别面：#19 立操作纪律，本条补「纪律漏执行时如何从输出识别」的判别签名。

### 定位（怎么发现的）

判别签名 = 每 chunk 耗时 + pregen 时长：全新生成必有 pregen（秒级，本例 ~24s）且每 chunk 耗时非零；缓存命中则 pregen 缺失/极短、chunk "FULL in 0-1ms"。发现路径：对比结果异常 → 回看导出日志逐 chunk 耗时列 → 量级差两个数量级以上即穿帮。

### 教训/如何利用

- 判据：任何「重导」类采集，导出完成后核对日志：① pregen 时长存在且为秒级；② 每 chunk 耗时非毫秒级近零——任一缺失即缓存命中，本次产物作废重导（先删 run\world，#19 前置）。
- 上位原则（同族合流）：产物「在盘」≠「本次执行体生成」≠「当前语义」——#16（旧 exe 假阴性）、#12（二进制产物哨兵点验）、#6（mtime 不可靠用内容指纹）、#19（删 world）与本条同族：每一环「产物是否由当前代码/当前运行产生」都必须有独立证据，不作默认假设。
- 与 AGENTS「seed 三查」同层级：seed 核对管「数据语义对不对」，本条耗时签名管「生成过程真没真发生」——两道关卡独立，都要过。

## 发现 #19: -PblockProbe.full=true 静默不生效——build.gradle 映射名实为 blockProbeFull，#8 家族三犯形态（260904-13）
- 现象：decisive ref vs off-fix13 报 78107 mism（12→78107 假回归）；日志缺 "pre-generated FULL region" 行，chunk 逐条 "FULL in 0-1ms"（旧 world 缓存形态，#18 家族签名）。命令带的是 -PblockProbe.full=true。
- 根因：build.gradle 映射行实为 blockProbeFull（camelCase），blockProbe.full（点分）映射不到任何属性 → FULL 口径静默缺失、carver 邻域预生成未执行；gradle 对未消费 -P 零告警。
- 定位：① 核 build.gradle 映射行；② 日志行为化证据（pregen 行不在场，#37/#18 判据）；③ A/B 隔离定责法——旧 dll 同命令复跑得 78116≠12 ⇒ 环境口径问题非代码回归；78116−78107=9 恰等于修复点数，形成意外旁证（口径修好后修复收益=9，与 verdict 一致；旁证不可单独定责）。
- 修复：改 -PblockProbeFull=true 后 mism=3，12→3。
- 教训：新 -P 参数首次使用必须核对映射行 + 日志行为化证据（#8 家族三实锤：手工清单遗漏 / rustStages 缺映射 / 点分驼峰不匹配——同根因三形态）。判据：① -P 用前 grep build.gradle 精确映射名；② 口径开关须有一次性日志行在场（#37）；③ 大面积异常残差先旧执行体同命令复跑做 A/B 隔离。
- 证据：.artifacts/lossless-accel/residual9-verdict-260904-13.md §3/§4.2 + .tmp/p2full/off-fix13-260904-13/。

## 发现 #20: 1.20.1 chunk NBT 解析两坑——①chunk NBT 无 xPos 键（region 坐标+槽位推导）②python struct.unpack_from 直读 int64 不推进位置指针（260905-03）

- **发现时间**：260905-03；**置信度**：candidate（2025 chunk 全量解析实锤，judge 待走）；**module**：build-tooling / MCA·NBT 工具链。

### 现象
① 按旧版经验在 chunk NBT 根 compound 找 `xPos` 键 → 键不存在，解析流程断言失败/坐标全错；② python 自写 NBT reader 用 `struct.unpack_from` 直读 TAG_Long / 长数组 → 解析若干 tag 后报 `bad tag`（非法 tag type），且报错位置随数据内容漂移。

### 根因
① 1.18+ chunk 格式移除了 `xPos`/`zPos` 键——chunk 坐标由 region 文件名坐标 + 槽位索引推导：`cx = rx*32 + (i & 31)`、`cz = rz*32 + (i >> 5)`（i = region 内 chunk 槽位序号）。用 1.17- 的格式心智找键必然落空。② `struct.unpack_from(buf, pos)` 从 pos 读值但**不返回也不修改 pos**——把 `unpack_from` 当「读且推进」用，位置指针停在原地，后续 tag header 从数据中间读起，tag type 命中非法值即 bad tag。这是解析失步（指针不前进），不是数据损坏。

### 定位
① 键缺失 → 查 MC wiki/反编译该版本 SerializedChunk 写入路径，确认键移除版本；② bad tag 位置漂移 + 报错点前必有一个 int64/长数组 tag → 打印每 tag 的 pos 前后值，发现 unpack 后 pos 不变即穿帮。

### 教训/如何利用
1. 跨版本解析 MCA 前先核该版本的 chunk 键集合（1.18+: 无 xPos；高度 span、section 索引基也随版本变）。
2. python 手写二进制 reader 铁律：用**推进式**读取（`pos += size` 显式推进或 `BytesIO.read(n)` 自推进），`unpack_from` 只用于「偷看不消费」场景。
3. 家族索引：#12（spv 多产物哨兵）、#11（配对以内容实测为准）；本条补「解析器自身推进状态」维度。

## 发现 #21: e2e_run 复用脚本的外部状态依赖——RCON 配置被上轮收口重置 off，stop 静默失败强杀世界（260905-04）

- **现象**：本轮 e2e 四臂运行中 `e2e_run` 脚本 stop 阶段静默失败，server 进程被强杀（world 未优雅保存）；Done 计时在 stop 前完成、数值本身有效，但依赖世界快照/善后状态的用途已坏。回查 `run/server.properties`：`enable-rcon` 已被上一课题收口时的复原动作重置为 off。
- **根因**：复用脚本对**外部可变配置**（server.properties 的 rcon 开关）有隐式依赖，而该配置会被其他课题「借出-复原」流程改动——脚本不自检依赖项在位，配置缺失时 stop 走不到优雅路径且无醒目报错（静默降级为强杀）。
- **定位**：stop 失败 + 世界被杀 → 核对 server.properties 发现 enable-rcon=off；对照备份 run/server.properties.bak-g3 确认被上轮复原动作重置。
- **修复**：恢复 `enable-rcon=true` + `rcon.password=coreswap`（备份保留，光照课题收口时再复原）。
- **教训/判据**：① 复用脚本开工前核其外部依赖项在位（如 enable-rcon），禁止假设「上次的配置还在」。② 「stop 失败但主计时正常」≠ 无损——区分计时类用途与快照类用途对善后的不同要求。③ 借出-复原流程在多课题并行/交接时天然制造静默重置，复原动作应记录到交接（NEXT_SESSION）显眼处。

---

## 发现 #22: Java 侧采集三坑合集——①gradle daemon 吞客户端 env（哨兵法验传播）②Loom RunConfigSettings 无 environment() ③rcon 单命令同步执行可超共享连接 10s 默认超时（260905-05）
- 时间/置信度/module：260905-05，candidate（WG_FEATURELOG 两次未达 native + 脚本侧修复实跑验证），build-tooling / gradle·loom·rcon 采集链。
- **现象**：① PowerShell 设 $env:WG_FEATURELOG 后 gradle runServer，native 侧两次未见 featurelog 输出；② loom RunConfigSettings 无 environment() 方法；③ rcon 发 forceload add 单条命令，共享连接在默认 10s 超时被掐断，命令实际还在服务器同步执行。
- **根因**：① gradle daemon 复用——daemon 常驻进程 env 在启动时定型，客户端 shell 后设的 env 不传导到 runServer 派生 JVM；② loom RunConfigSettings DSL 未暴露 environment 注入，只有 run task 本体（JavaExec 型）的 environment() 可用；③ rcon 客户端读回包，但 forceload 类命令服务器同步执行完才回包，8×8 tile 循环单条可远超 10s；共享连接超时即断链并污染后续命令。
- **定位**：① 哨兵法——JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=...，JVM 启动必打「Picked up JAVA_TOOL_OPTIONS: ...」（各 run 日志 .err 首行实锤），哨兵在位而 WG_FEATURELOG 不在位 → 锁定通道差；② grep loom API 确认 RunConfigSettings 方法面；③ 超时时间点与命令实际完成时间对表。
- **修复**：① env 传递走 JAVA_TOOL_OPTIONS（-D 属性通道，轮次间清理防污染）或 build.gradle -P→task environment() 映射（-PfeatureLog=1 已落地）；② forceload 预生成改 per-command 独立 rcon 连接 + 180s 超时 + 失败重试（pregen_forceload.py 的 rcon_one()）；③ 需 task 级 env 时在 run task（JavaExec）上用 environment()。
- **教训/判据**：① gradle 链路传 env 第一动作 = 放哨兵验证传播——「客户端设了」≠「JVM 收到」≠「native 读到」，三段各自要证据；② loom 注入优先 -P 映射（#8 家族，核对映射名），RunConfigSettings.environment() 此路不通；③ rcon 长命令判据：服务器同步执行类命令一律独立连接 + 超时 ≥ 最坏耗时 + 重试。同族：build-tooling #19——「传递通道名实不符」家族，本条补进程 env 通道 + rcon 连接通道两维度。

---

## 发现 #23: `cargo build -p <薄壳>` 依赖 rlib 陈旧假绿——「Finished」不等于依赖包重编（260905-06）

- **时间/置信度/module**：260905-06；candidate（exe/rlib 字符串核验实锤）；build-tooling / cargo·workspace 构建链（#16/#12「产物在盘 ≠ 当前代码」家族的 cargo 形态）。

### 现象
260905-01 workspace 拆分后的薄壳结构（`worldgen-core`（包名 WorldgenRust，rlib）+ `versions/1.20.1/rust`（cdylib 薄壳））下，改完 `worldgen-core` 源码后执行 `cargo build --offline -p worldgen --release`，多次输出 `Finished`（零告警零重编迹象），产物 worldgen.dll 正常产出、注入运行「成功」。但 `target/release/libWorldgenRust.rlib` 的 LastWriteTime 停留在数小时前——依赖包根本没重编，新旧源码链接的是**同一份陈旧 rlib**。本轮后果：带着真实 bug（BlockPredicate 字段名错读，见 feature-parity 错误台账 ①）的旧代码与「修复后」代码产出**逐字节一致的 dump**，形成「修复无效/行为无差异」假象，浪费一整轮对拍。

### 根因
`-p worldgen` 只把薄壳 cdylib 作为构建根——cargo 的增量/时间戳判断在某种状态下漏判了依赖 rlib 的过期（源码 mtime 与 rlib mtime 的判定被扰动，如 git 操作/批量 touch 后 mtime 关系反转），于是直接复用旧 rlib 链接。**「构建成功」= 图上被选中的目标编译链接成功，不保证依赖目标被重编**；薄壳 + workspace 的间接层把这层不透明化（用户盯着的是 dll，rlib 在背后）。

### 定位（怎么发现的）
排除法：dump 逐字节一致 → 先怀疑修复未生效 → 核产物链 mtime——`target/release/libWorldgenRust.rlib` 的 LastWriteTime 早于源码修改时间数小时 → 实锤；旁证 = 对 dll/rlib 做字符串核验，新日志串缺失（新代码里的诊断字符串不在二进制内）。「构建日志绿 + 产物在盘 + 运行成功」三绿俱全仍是假象，唯一可信的是**产物 mtime vs 源码 mtime + 内容指纹**。

### 修复
显式构建依赖包：`cargo build --offline -p WorldgenRust --release` 后再 build 薄壳（或改源码后必核 rlib mtime）；根治性核验 = `Get-Item target\release\libWorldgenRust.rlib` 的 LastWriteTime 晚于全部 worldgen-core 源码 mtime。

### 教训/判据
1. **判据（MUST）**：改源码后 build 前后必核最终链上每个中间产物的 LastWriteTime 晚于源码 mtime——薄壳/包装层结构下尤其如此（盯的产物与改的代码隔了一层）。
2. 内容级旁证：新诊断字符串/新日志行必须在二进制里 grep 得到，缺失即陈旧产物（#12 哨兵点验思想的 cargo 形态）。
3. 家族索引：#6（mtime 不可靠用内容指纹——互补：这里 mtime 是**唯一**穿帮线索，因为「一致」本身被当结论）、#16（旧 exe 假阴性）、#18（缓存命中耗时签名）、#19（gradle -P 静默不生效——同属「gitignore/build 工具链静默假绿」家族）；与 AGENTS 构建铁律（ninja 卡死/build.ps1 直链）同级：「构建系统信任必须落到产物证据」。
4. 证据：`.investigations/feature-parity/260905-06-errors.md` 条目 ⑤ + 本轮 exe/rlib 字符串核验记录。

---

## 发现 #24: gitignore 目录级规则 prune 使 `!` 白名单失效——`data/` + 子文件 `!` 永不生效（260905-06）

- **时间/置信度/module**：260905-06；candidate（`git check-ignore -v` 双向核验实锤）；build-tooling / gitignore 规则语义（#8/#19「配置静默不生效」家族的 gitignore 形态）。

### 现象
`.gitignore` 写了 `data/`（目录级忽略）后又试图用 `!data/light_data.json` 类白名单重包含——白名单**静默不生效**，文件仍被忽略（judge S3 阻塞项的根因面：light_data.json 不入库）。且把规则改成 `data/*` 试图「只忽略内容」时，含斜杠模式被**锚定到仓库根**，丢失了「任意层级 data 目录」的原语义。

### 根因（git 两层机制）
1. **目录 prune**：git 对目录级规则（`data/`）直接**不进入该目录**——目录被剪枝后，里面的文件的 `!` 重包含规则根本没有被评估的机会（文档明确：「It is not possible to re-include a file if a parent directory of that file is excluded」）。白名单要生效，前提是**父目录链全部未被忽略**。
2. **锚定规则**：模式含非尾随斜杠（`data/*`）即相对 .gitignore 所在目录锚定，不带 `**` 就没有「任意层级」语义——`data/` 与 `data/*` 语义并不等价（前者任意层级目录、后者仅根下）。

### 定位
改完规则跑双向核验：`git check-ignore -v <白名单文件> <邻位应忽略文件>`——前者应无输出（未忽略）、后者应命中忽略规则行；再 `git status` 确认无 `??` 噪声。白名单文件仍报命中忽略规则即穿帮。

### 修复（正确形态模板）
```
data/              # 保留：任意层级 data 目录整体忽略（现状语义）
**/data/*          # 通用覆盖：任意层级 data 的内容（与上一行互补，防锚定误解）
!versions/1.20.1/data    # 链式放行目标目录本身（父目录先不被忽略）
versions/1.20.1/data/*   # 再重排除其内容
!versions/1.20.1/data/light_data.json   # 精确白名单目标文件
```
要点：**目录放行 → 内容重排除 → 文件白名单**三段链式，缺一段即静默失效。

### 教训/判据
1. **判据（MUST）**：任何 gitignore 白名单改动，`git check-ignore -v` 双向核验（白名单文件 + 邻位文件）+ `git status` 无 `??` 噪声，缺一不算完成——gitignore 不生效**永远无告警**。
2. 记忆锚：「父目录被忽略 = 子文件白名单死刑」；「模式含斜杠 = 锚定，`**` 才跨层级」。
3. 家族索引：#8（rustStages 缺映射）、#19（-P 点分驼峰不映射）、#22（gradle daemon 吞 env）——同族第四形态：**配置/规则层静默不生效，全部靠行为化核验兜底，无一例有报错**。证据：`.investigations/feature-parity/260905-06-errors.md`。

## 发现 #25: runServer 采集三坑扩展——①mixin 门控 sysprop 映射（JAVA_TOOL_OPTIONS 的 -D 是 sysprop 非 env）②非 cancellable 方法 @Inject 禁 setReturnValue ③SEEDLOG 全量噪声 + spawn 区预生成拖慢采集（260905-10）

- **时间/置信度/module**：260905-10（实际 2026-09-05）；candidate（三坑均本轮实测复现 + 修复验证）；build-tooling（#8/#19/#22/#32 家族扩展，Java 侧采集三坑第二辑）。

### ① mixin 门控 sysprop 映射——JAVA_TOOL_OPTIONS 的 `-D` 是 sysprop 不是 env
- **现象**：mixin 里 `System.getenv("wg.treediag")` 恒 null，WG_TREEDIAG 首跑 0 输出。
- **根因**：env 壳（JAVA_TOOL_OPTIONS）注入的 `-D` 进的是 JVM **系统属性**域，`getenv` 读的是进程环境域——两个域名字再像也不互通；mixin 代码在目标 JVM 里跑，读不到宿主 shell 的 env 变量（#32 daemon 吞 env 的姊妹面：域不同）。
- **定位**：哨兵法验证传播链（#22），逐环断在 env 壳→getenv 这一段。
- **修复**：走 build.gradle `findProperty` → vmArg 映射：`-Ptreediag=1` → runServer vmArg `-Dwg.treediag=true`，mixin 侧改读 sysprop；**每新增一个 -P 门控必须同步加映射行**（#8/#19 家族铁律，缺行静默不生效）。

### ② 非 cancellable 方法 @Inject 禁止 cir.setReturnValue——CancellationException 崩 feature 放置
- **现象**：Square 的 RETURN 注入用 `cir.setReturnValue` → `CancellationException`，feature 放置直接崩。
- **根因**：`getPositions` 返回 `Stream` 且**非 cancellable**——inject 回签不支持取消返回，`setReturnValue` 只对 cancellable callback 合法。
- **修复**：改 `@Redirect`（本轮双 ordinal 打 [SQX]/[SQZ]）或 peek 包装旁路观测（防 stream 消费副作用）。
- **判据**：给非 cancellable 方法做观测注入，只能选 @Redirect / peek 包装，**永远不用 setReturnValue**。

### ③ runServer 采集慢两根因：全量 SEEDLOG 噪声 + spawn 区预生成
- **现象**：runServer + SEEDLOG 全量打点 ~17min 才 Done（deadline 需 ≥25min）；spawn 区预生成 20min+。
- **根因**：① SEEDLOG 对**所有** population 行打点，噪声 -99% 级冗余；② spawn point 固定导致预生成无法绕开。
- **整改方向**（已识别未实施）：mixin chunk 过滤——population 行自带 x/z，static curChunk 比对即过滤（预期噪声 -99%）；spawn point 预置目标 chunk；结构性方案 = 单 chunk 直驱 harness 免 runServer。

### 教训/判据
家族索引：#8（-P→-D 映射遗漏）、#19（映射名点分驼峰不匹配）、#22（daemon 吞 env + 采集三坑第一辑）、#32（daemon 死同值）——同族第五形态延续：**门控/注入配置层静默不生效，全部无报错，只有行为化核验能兜底**。

### 证据
`.investigations/feature-parity/260905-10-interim.md`（WG_TREEDIAG Java 通道三坑 + 首跑 0 输出根因）；`.tmp/feature-parity-260905-10/treediag_java_run.ps1`（-Ptreediag→vmArg 映射实现）；`.artifacts/feature-parity/candidate-beehive-260905-10.md` §4（采集效率整改）。

---


## 发现 #25 补充案例（260905-13）：sysprop 名 ≠ env 名是两个命名域——`-D` 大小写/点号逐字对上才算数（jungle-l E1）

- **时间/置信度/module**：260905-13；candidate（三核验全败 → 读消费端 static 块逐行核对实锤）；build-tooling（#25① sysprop/env 域差的第二犯，补「名字逐字」维度）。

- **现象**：`JAVA_TOOL_OPTIONS="-DWG_DIAGCHUNK=29,-16"` 采集，banner 0 行、24109 行全量未过滤——本次 -D 通道选对了（读侧就是 sysprop），但**名字没对上**：`-DWG_DIAGCHUNK` 建的是 sysprop `WG_DIAGCHUNK`，消费端读 sysprop `wg.diagchunk` 或 env `WG_DIAGCHUNK`；JAVA_TOOL_OPTIONS 的 -D 不创建环境变量 → 两条都不命中 → 按「未设目标=全量」走全量，banner（防死参数哨兵）按设计不打。
- **根因**：sysprop 域与 env 域是**两个独立命名域**，`-DFOO=1` 只建 sysprop `FOO`；域选对后名字仍须**逐字**对上（大小写、点号/下划线），任一错位即死参数——无报错无告警，哨兵不设计就完全静默。
- **定位**：三核验（banner/过滤/目标行）全败 → 停止调 mixin，读 WgDiag.java static 块读取表达式 + build.gradle 映射表逐行核对。
- **修复**：v2 改 `-Dwg.diagchunk=29,-16`；补 build.gradle `-Pdiagchunk → -Dwg.diagchunk` 映射行（原本缺失，`-Pdiagchunk` 首跑中同样是死参数——#8 家族缺映射行再证）。
- **判据**：门控参数接线四查 = ①通道域（sysprop vs env）②名字逐字（大小写/点号）③-P→-D 映射行在位（#8/#19）④行为化哨兵（banner/首行标记，#37）——四查全过才承认「自变量真被改变」。
- **证据**：`.investigations/jungle-l/260905-13-errors.md` E1。

## 发现 #23 补充案例（260905-13）：mtime 两面都不可信，内容指纹哨兵是唯一可靠手段；diagnose 顺序 = 先哨兵后重编（jungle-l E3）

- **时间/置信度/module**：260905-13；candidate（exe 字符串哨兵 False→重编→True 实锤）；build-tooling（#23 第三犯，判据收敛升级：#6 说「mtime 不可信用内容指纹」、#23 说「mtime 是唯一穿帮线索」——本例证明两面都不可靠，内容指纹才是终审）。

- **现象**：J1 修复 + TREESET/CORN 打点后 `cargo build --offline -p worldgen --release` 显示 Finished，rlib mtime 仍 21:59（按 #23 判据核 mtime 会误判「未重编」）；单编 exe 字符串哨兵 `TREESET` = False（确实陈旧）。
- **根因**：`-p worldgen` 只保证薄壳包新，core 包（WorldgenRust）在 cargo 看来 fresh（改动时间戳未被识别，与 #6 mtime 红旗同族）→ rlib 陈旧。mtime 判断两面失效：#6 案例中 mtime 骗人「产物是新的」、本例 cargo 时间戳判定漏判使 rlib 实际是旧的——mtime 依赖的工具链状态本身不可靠。
- **定位/修复**：显式 `cargo build --offline -p WorldgenRust --release` 后重单编 exe，哨兵 True；重跑采集 TREESET 15 行正常。
- **判据（升级，MUST）**：产物新鲜度**只认内容指纹**（exe/rlib/dll 内 grep 本轮新增诊断字符串）；mtime 只作旁证不作裁决。诊断顺序固化：**先哨兵后重编**。
- **证据**：`.investigations/jungle-l/260905-13-errors.md` E3。



## 发现 #25 补充案例（#61): 诊断工具归位声明——concern2_nether_dump 收编 bin-diag（260906-05）；candidate

- **发现时间/发现者**：260906-05，core.worker subagent 草稿 + 主会话应用。
- **现象**：CONCERN-2 新增 nether 确定性 dump 工具留在 .tmp/concern2-260906/，无主状态。
- **判据（MUST）**：被知识库判据引用为「标准载体」的 .tmp 工具 MUST 声明归位（AGENTS.md §八.13），不留无主状态；留 .tmp 迟早灭失。
- **修复**：收编 worldgen-core/src/bin-diag/nether_region_dump.rs（头注 anchor.source 指 verdict 证据链 + wg_dir 参数化 + rustc 单编验证 OK）；ow 侧 idk7_region_dump.rs 已在 bin-diag，同族增殖（第二份维度变体）时考虑维度参数化合并。
- **证据**：.tmp/concern2-260906/concern2_nether_dump.rs（原件）→ worldgen-core/src/bin-diag/nether_region_dump.rs（收编版）。

## 发现 #25 补充案例（#8/#25 家族，260906-06）：env+sysprop 双通道同时失效——daemon 吞 env × 照抄历史脚本漏 -P 映射行

- **现象**：j5_java_baseline_260906-06.ps1 首跑退出 0、forceload 正常，但 [SEEDLOG]/[MJT0] 全零。
- **根因**：只设 env（WG_SEEDLOG/WG_TREEDIAG）而 gradle daemon 复用吞掉新 env（#32）+ 照抄 wgdiag_firstrun.ps1 时漏 `-PseedLog=1 -Ptreediag=1`（build.gradle findProperty→vmArg 映射，#8/#25）——双通道同时失效。
- **判别签名**：日志**零 [SEEDLOG] 行**（门未开）而非打点异常（门开无数据）；「诊断开关生效验证 = 输出行为化（#37）」再证：先验 1 行样本再等全量。
- **修复**：gradle --stop 杀 daemon + 补 -P 通道重采（1154 population 行）。
- **证据**：.investigations/jungle-l/260906-06-errors.md E12。

## 发现 #26: Chunky 双臂区域级地形验证载体——gradle runServer 同实例对称双臂 + region 程序化 diff，替代 forceload 人工观察（260906-09 深夜）；confirmed

- **时间/置信度/module**：260906-09 深夜；confirmed（用户拍板「授权确认」转正，区域级地形验证标准载体）；build-tooling（验证载体升级，非错误条目）。
- **是什么**：区域级地形 A/B 验证新载体——`gradle runServer` **同实例对称双臂**（vanilla 臂 `-PcppVanilla=1` / coreswap 臂默认 stageMask=3，切参数即换臂，排除跨实例变量）+ `run\mods` 放 Chunky 1.3.146（Fabric）jar + 控制台 `chunky world/center/radius/start` 命令（`Task finished ... 100.00%` = 确定性完成行，替代 sleep 猜时长）+ 生成落盘 region mca → python 程序化逐块 diff（每 section 每 palette 名计数差，chunk 用 region+slot 定位）。
- **效率实测（confirmed）**：Chunky ~4min/1089 chunks（33×33）vs 旧 forceload 法 ~27min/25 chunks（5×5），覆盖面/时间比 >40×；零人力（旧法 tp+F3 逐点截图），region 全量落盘可复用——任意柱/块事后复查零成本。固定启动成本两法相同；**单点疑问仍以 tp+F3 法最快**（本载体定位 = 区域级，不替代单点 sanity）。
- **前置三查（MUST，对比前逐项过）**：
  1. **CppBridge init 行核 stageMask / dll 大小 / seed**——确认 Chunky 生成真走 mod 管线（本轮 stageMask=3、dll 2160640 = 1.0.26 出货，#36）；
  2. **chunk 定位用 region 坐标 + 槽位推导，不信 NBT 内坐标**（#20：chunk NBT 无 xPos 键）；
  3. **对比必须剔植被**——特征流非确定（#67，两臂 Chunky 生成均多线程），不剔会把树/植被差当地形差（本轮 oak/jungle leaves+log、vine ≈86k 差值全属此类）。
- **证据**：`.investigations/jungle-l/chunky-trial-260906-09.md`（主文档）；`.tmp/jungle-l-260906/chunky/`（chunky-coreswap.log / chunky-vanilla.log / region-{coreswap,vanilla}/ 各 6 mca + chunky_diff_260906-09.py）。

## 发现 #27: 缓存新鲜度 marker 单判 → 增量数据集静默退化——marker 语义是「新鲜度」不是「完整性」，新增数据子集必须配新增 marker（260907-04）

- **时间/置信度/module**：260907-04；candidate（机制静态审查定论 + golden 5/5 + judge PASS-with-conditions，Java 侧行为未跑生产复验——降级声明在案）；build-tooling / 资源解压缓存（#18「产物在盘 ≠ 本次生成」家族第四形态：**缓存命中 ≠ 数据集完整**）。

### 现象

CoreSwapFixHelper 用单一 marker（`noise_settings/overworld.json` 存在）判定 tmp 解压缓存新鲜度。260907-04 tag 数据驱动化给数据集**增量**新增子目录 `tags/blocks/`（170 文件）后：升级侧旧 tmp 缓存仍在 → marker 命中 → 跳过重解压 → **新数据永远缺失且无任何报错**——Rust 侧 `expand_tag` 返回 false 走硬编码 fallback（fallback 协议掩盖了数据层失效），数据驱动静默退化为硬编码。生成不炸、从外部表现完全无法区分「正常 fallback」与「缓存导致的数据缺失」。

### 根因

marker 的真实语义是「**缓存相对于上次解压是否新鲜**」，却被当作「**缓存数据集与当前资源集等价**」使用——单判据语义超载。缓存命中只证明「旧数据完整」，不证明「旧数据 ⊇ 新数据集」；数据集**增量**演进（新增子目录/文件）时 marker 集合本身过时。与 #18 对比：#18 是「生成过程没发生但产物在盘」，本条是「解压没发生但 marker 在盘」——同一上位原则（「看起来对」的元数据不作产物健康判据）在资源缓存域的形态。

### 定位

改动评审阶段从架构推演发现（本例未烧轮次）：新增数据子目录 → 追问「旧缓存里有没有它」→ marker 只查旧路径 → 命中即跳过解压。judge 意见书补充定位到残余缺口：解压后只复检旧 `marker` 不复检 `tagMarker`——若 jar 打包事故缺 tags，同样静默走 fallback 无报错。

### 修复

单 marker → 双 marker：每个**新增数据子集**配对应 marker（`tags/blocks/overworld_carver_replaceables.json`），条件合取（任一缺失 → deleteRecursively + 重解压）。judge 条件 2（SHOULD，已应用）：解压后**逐 marker 复检**（缺则 throw），封堵打包侧缺数据的静默面。

### 教训/判据

1. **跨版本升级凡动资源目录结构（增/删子目录），第一动作 = 审计缓存新鲜度判据的 marker 集合是否覆盖新结构**——「marker 在」≠「数据全」。
2. **marker 判据必须与数据子集一一对应**（数据集增量演进时 marker 同步增量）；单 marker 守多子目录 = 语义超载反模式。
3. **fallback 协议是静默退化的温床**：fallback 设计初衷是「跨版本数据未跟上不炸」，但它同时吞掉「本版本数据该在而缺失」——有 fallback 的数据路径必须配**一次性显著日志** + 解压后 marker 复检，缺一即无法从行为面区分正常 fallback 与缓存事故。
4. 家族索引：#18（缓存命中耗时签名）、#16（旧 exe 假阴性）、#12（二进制产物哨兵点验）、#24（gitignore 静默不生效）——共同上位原则：**每一环「数据/产物是否与当前代码/资源集等价」都必须有独立证据**；本条新增「资源解压缓存」这一环。
5. 证据：`.artifacts/tag-datadriven-260907-04.md` 改动 #7 + `.artifacts/judge-tag-datadriven-260907-04.md` 条 D/条件 2。

## 发现 #23 补充案例（260907-04，机制面二）：`-p` 构建图范围收窄——库 enum 加 variant 后 6 个诊断 bin 编不过近一月未暴露，「单包绿」不构成任何全量结论

- **时间/置信度/module**：260907-04；candidate（6 bin 破损 + 修复全量绿实锤）；build-tooling / cargo workspace 构建链（#23 家族，**机制面二**：#23 = 依赖 rlib 陈旧漏判，本条 = `-p` 根本不把其它目标的 bins 纳入构建图——不是漏编，是没编）。

- **现象**：end 接管（260906-04）给 `DensityFunction` 加 `EndIslands` variant 后，6 个 `worldgen-core/src/bin/` 非穷尽 match 编不过（density_tree_profile / transpiler_ch0_census / channel_probe×2 处 / ch0_tree_analysis / macrolize_probe / tree_vs_noise_breakdown）。日常构建命令 `cargo build --offline -p worldgen --release` 只建薄壳 cdylib，core 的诊断 bins 不在 `-p worldgen` 构建图内——破损状态存活近一月，纪律 13a「全量绿」被无意识绕过，直到 260907-04 顺手修复（各 bin 仅补 `EndIslands(_)` arm）后才以 workspace 全量 build 恢复绿。

- **根因**：`-p <pkg>` 把构建根收窄到单包——**未选中的目标根本不参与编译**，「Finished」只证明被选中目标绿。库层公共 enum 加 variant 是「编译半径爆炸」型改动（全部 match 点都要动），但日常 `-p` 构建命令的半径恰好覆盖不到诊断 bins，破损被结构性隐藏。

- **定位**：本例非运行期发现，是 260907-04 全量 build（发布前检查）暴露 6 个 bin 编译错误回溯归因到 260906-04 的 variant 提交。判据（及早发现）：库 enum 加 variant 的 commit 当轮就跑一次 workspace 全量 build，看是否冒出非预期目标的 E0004。

- **修复/判据**：
  1. **库公共 enum（跨模块被 match 的 variant 承载体）加 variant 的 commit，MUST 以 `cargo build --offline --release`（workspace 全量，不带 -p）验证**——`-p` 单包绿 ≠ 全量绿，两者结论域不同。
  2. 破损暴露窗口 = 全量 build 间隔：纪律 13a 的「全量绿」检查必须绑定到**库层 API 变更类 commit**（enum/trait/公共签名），不能只在发版前兜底。
  3. 家族索引：#23（陈旧 rlib 假绿，机制面一）、#16（bin-diag 不参与默认构建的姊妹面：`src/bin/` 参与 `cargo build` 但不参与 `-p worldgen`）——合流判据：**「构建绿」结论必须声明构建图范围，范围外目标不作任何假设**。
  4. 证据：`.artifacts/tag-datadriven-260907-04.md` 改动 #9 + judge 意见书条 F（diff 逐行核对，全部仅添加 EndIslands arm）。

## 发现 #28: 冒烟口径 ≠ 存档口径——`-PcppWorldgenDir` 必带项（#15）在注册冒烟下反转，带错参数集 = handle=0 全 -1（260907-08）

- **时间/置信度/module**：260907-08；candidate（round3 失败 → 去参重跑实锤）；build-tooling / gradle run 参数口径（#15 家族延伸，「裁剪历史参数清单」的对偶面：不分场景照抄历史清单）。
- **现象**：round3 冒烟按 #15 口径带 `-PcppWorldgenDir versions\1.20.1\data` → 该顶层布局无 wg_create 所需根级 settings 文件 → 三句柄 handle=0，注册全 -1（`[BLOCKS-REG] ... rust_id=-1` ×全部探针 + `[BLOCKS-REG] done count=0`）；判别签名 = `[CppBridge] init ... enabled=false`。去掉该参数走 jar 内 worldgen-data 解压路径后全绿（与 260907-07 冒烟口径一致）。
- **根因**：#15 的「-PcppWorldgenDir 必带」是**存档口径**项（block_probe 等从 data 目录读参照的历史条件），不是普适必带项；注册冒烟的数据路径走 jar 解压，带存档口径参数反而把 worldgenDir 指向一个布局不兼容的目录。「历史参数清单必带项」被当成了跨场景公理——与 #15 的教训（裁剪清单裁掉必带项）恰成对偶：#15 是「少带」，本轮是「多带」。
- **定位**：`[CppBridge] init ... enabled=false` 一行即判别（handle=0 的直接签名），回看 #15 出处核对参数归属口径，一轮定位。
- **修复/判据**：
  1. gradle run 参数清单按**口径分组维护**（存档口径 / 冒烟口径 / 探针口径各自成列），引用时先声明自己处于哪个口径，禁止跨口径整单照抄。
  2. 「XX 参数必带」类历史判据引用前核对**其原始出处场景**，必带性是场景属性不是参数属性。
  3. handle=0 / enabled=false 是 CppBridge 数据路径失败的判别签名，注册类冒烟见到全 -1 先查它，不先查注册实现。
- **家族索引**：#15（裁剪参数清单裁掉必带项——本条为其对偶形态）；#22（缓存 marker 单判）同属「历史判据跨场景复用前核归属」上位原则。
- **证据**：.tmp/blockreg-smoke3-260907-08.log（enabled=false 全 -1）vs .tmp/blockreg-smoke4-260907-08.log（去参后 enabled=true stageMask=3 + 显式 id 1003 三句柄对齐）；.artifacts/jni-blockid-fix-260907-08.md 失败轮记录。

## 发现 #29: RCON 协议 type 字段为 4 字节 int（length = len(payload)+10）——1 字节 type 帧被静默误解析，auth 偶然通过 + 命令超时假象（260907-09）

- **时间/置信度/module**：260907-09；candidate（帧格式修正后 forceload/stop 全部立通）；build-tooling / RCON 客户端实现。
- **现象**：自研 RCON 客户端发 forceload 后无响应，现象酷似「服务器主线程挂死」（smoke8-10 三轮）；但 auth 又时而「成功」，服务器端无报错，watchdog 无触发。
- **根因**：RCON 帧格式 = `[length:int32][requestId:int32][type:int32][payload][0x00 0x00]`，type 与 requestId 都是 **4 字节小端 int**；客户端把 type 写成 1 字节 → 整帧长度与字节布局全错。服务器按 length 读取时帧边界错位，auth 包恰因短 payload + 错位边界「偶尔」被容忍，命令包则进入无响应/Unknown request——故障面在客户端编码，服务器完全无辜。
- **定位**：不疑服务器（watchdog/日志干净）→ 最小字节帧核对协议规范，一眼看出 type 域 1B vs 4B。判别签名：「auth 偶然通过 + 命令一律超时 + 服务器侧零异常」三联 = 客户端帧格式错误，不是服务端问题。
- **修复/判据**：
  1. RCON 帧统一 int32 LE 编码 type/requestId，length = len(payload) + 10（4+4+4+2 尾零）。
  2. 自研二进制协议客户端首通后，先发一条已知回显类命令（如 `list`）验证往返，再用于判别实验——帧格式错误会伪装成任何远端故障。
  3. 「远端疑似挂死」先排除自研客户端编码：服务器日志/watchdog 干净时优先疑测量侧。
- **家族索引**：与 #22（Java 侧 rcon 10s 共享连接超时）互补——#22 是超时口径坑，本条是帧编码坑，两者都伪装成「命令无响应」。
- **证据**：.investigations/realmod-e2e/realmod-e2e-260907-09.md smoke8-10 轮次；.artifacts/realmod-e2e-260907-09.md §6。

## 发现 #30（#23 家族新形态）: rustc 单编 bin-diag 链根路径 `target/release/libWorldgenRust.rlib` = 陈旧缓存——`cargo build -p worldgen` 不刷新依赖包根产物，必须链 deps 最新 hash rlib（260907-09）

- **时间/置信度/module**：260907-09；candidate（mtime 对比 + 修正后诊断输出含新行为化日志实锤）；build-tooling / cargo 产物布局（#23 家族第四形态）。
- **现象**：rustc 单编 bin-diag（如 diag_modns.exe）用 `--extern WorldgenRust=target/release/libWorldgenRust.rlib`，链接成功但诊断 exe 长时间反映旧代码行为——修复已进源码、cargo 构建显示 Finished，诊断结果却不变。
- **根因**：`cargo build -p worldgen`（薄壳 cdylib）**不刷新依赖包 worldgen-core 的根路径产物** `target/release/libWorldgenRust.rlib`（历史遗留快照）；cargo 正常增量走 `target/release/deps/libWorldgenRust-<hash>.rlib`。rustc 手工单编引用根路径 rlib = 引用无构建图维护的陈旧文件。
- **定位/判据**：**根 rlib mtime vs deps 目录最新 rlib mtime 对比**——根产物 mtime 早于源码最近修改即必陈旧；辅证 = 诊断 exe 输出缺少本轮新增的行为化日志行。
- **修复/判据**：
  1. rustc 单编 bin-diag 的 --extern 一律指向 deps 目录最新 hash rlib（`Get-ChildItem target\release\deps\libWorldgenRust-*.rlib | sort LastWriteTime | select -Last 1`），禁用根路径产物。
  2. 「cargo 构建绿 + 诊断行为旧」组合 = 先查链接产物新鲜度，不先疑诊断代码。
- **家族索引**：#23（cargo -p 依赖 rlib 陈旧假绿）第三形态；#16（bin-diag 旧 exe 假阴性）同族上位原则；#27（marker 新鲜度）。
- **证据**：.artifacts/realmod-e2e-260907-09.md §5；.investigations/realmod-e2e/realmod-e2e-260907-09.md 执行体三元组段。

## 发现 #31 简记: 测试载体判据——dev loom 子工程 jar 用 devlibs 未 remap 的 `-dev.jar` 进 dev run/mods（named 映射）；remapped jar 只进生产/Connector 场景（260907-09）

- **时间/置信度/module**：260907-09；candidate（content-test 实测双向验证）；build-tooling / loom jar 载体选择。
- **观察**：loom 子工程产物两套：devlibs 未 remap `<name>-dev.jar`（named）与 remapped 生产 jar。dev runServer 是 named 载体，run/mods 必须放 **-dev.jar**；remapped jar 放进去类名对不上。
- **如何利用**：dev 环境内容 mod 验证 → `build/devlibs/*-dev.jar` 入 `run/mods/`；forge-server/Connector 才用 remapped jar。放错载体判别签名 = mod onInitialize 自证打印不出现。
- **证据**：.artifacts/realmod-e2e-260907-09.md §1/§7（生产环境未测，idk 已声明）。
