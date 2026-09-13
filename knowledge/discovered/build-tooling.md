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

**#31 补充案例（260907-10）**：remapped jar 进 Forge+Connector 生产 mods + post-Done forceload 观察判据第三轮复用确认——主线 `coreswap-1.0.26.jar` 与 content-test `content-test-1.0.0.jar` 均以 remapJar 产物进 `runtime/forge-server/mods/`（SRG 载体，boot 日志 `server-...-srg.jar` 在位），forceload 在 `Done (13.928s)` 之后触发（#80）。来源：.artifacts/forge-prod-e2e-260907-10.md §1/§2。

## 发现 #32: 生产/开发口径参数缺失第二实例——`-Dcpp.blockRegister` 门控生产侧无携带，注册链下半断 → 惰性解析 miss 全 AIR（260907-10）

- **时间/置信度/module**：260907-10；candidate；build-tooling / 生产口径参数清单（**#28「冒烟口径 ≠ 存档口径」家族第二实例**；#8「-P→-D 映射遗漏」家族跨口径形态）。
- **来源定位**：.artifacts/forge-prod-e2e-260907-10.md §3；CppBridge.java:168（sysprop 门控默认关）。
- **现象**：三轮对照——轮1/轮2 env override 行为化日志三维命中（`[WGH] env override ...`），但 region 扫描 test_brick 恒 **0**；boot 日志 `[BLOCKS] unknown block 'testcontent:test_brick' -> AIR (register via wg_register_block)` ×3 维度，全日志**无任何 `[BLOCKS-REG]` 行**。轮3（user_jvm_args.txt 追加 `-Dcpp.blockRegister=1`）后 test_brick = 672 命中 / 144 sections，`[BLOCKS-REG] ... java_raw=1003 rust_id(三域 1003) writeback=...` 在场。
- **根因**：`CppBridge.registerModBlocks()` 被 `-Dcpp.blockRegister` sysprop 门控默认关。dev 口径 260907-09 经 gradle `-PblockRegister`→`-D` vmArg 映射自动带入；生产 run.bat 裸 java 口径无该映射环节 → mod 方块从未注册进 Rust registry → #79 惰性解析按名 miss → default_block 整片 AIR。机制层：**注册是链路独立的下半段**——env override（上半段，分支选择）生效 ≠ 方块名可解析（下半段，注册表内容）；两段由不同参数分别门控，只验上半段会误判「链路已通」。
- **定位**：两段行为化日志组合判读——① `[WGH] env override` 命中（分支活）；② `[BLOCKS-REG]` 缺席 + `[BLOCKS] unknown block 'X' -> AIR` ×维度数（惰性解析 miss 直证）。注意：PASS 轮 boot 早期（世界创建时）也会出现 unknown 行（注册在其后），unknown 行**在场不作判别**，`[BLOCKS-REG]` 有无才是判别面。
- **修复**：`runtime/forge-server/user_jvm_args.txt` 追加 `-Dcpp.blockRegister=1`（生产口径必带清单 +1）。注：值 `1` 同时解析为注册 limit=1，只注册首个 mod 块（test_lamp 恒 0 属预期）；多 mod 块验证时 limit 须调大。
- **教训**：① 跨口径参数清单**按口径分组维护**，新门控参数（sysprop/env/`-P` 均同）加入时 MUST 同步登记每个口径的携带方式——映射环节本身是口径差异点；② **「单段行为化日志命中 ≠ 全链通」**——每个参数门控的段都要有自己的在场/缺席证据（#37 家族延伸）。
- **证据**：`.tmp/forge-prod-v1-260907-10-boot.log`（FAIL 轮）vs `.tmp/forge-prod-v1b-260907-10-boot.log`（PASS 轮）；region 扫描 0 vs 672。

## 发现 #33: 离线启动载具假 UUID → 依赖玩家身份的 mod 静默半初始化——「管线没跑」伪装成「bug 不复现」（260908-04，#34 谓词耦合家族）

- **时间/置信度/module**：260908-04；candidate（R4/R5 修复前后对照实锤）；build-tooling / 离线启动链参数（**workflow #34 谓词耦合家族 / #28 跨口径参数家族跨载具形态**）。
- **来源定位**：`.investigations/mod-compat-260908-02/voxy-debug-260908-03.md` §260908-04 增补·事件4 + `cmd-output/` R4/R5 日志。
- **现象**：自建 Forge 离线客户端（launch_client.ps1）R4 轮 Voxy `async init failed`，LOD/Ingest 管线整体不起，Ingest 已知异常当轮「不复现」——差点得出「异常与环境相关」的假结论。
- **根因**：离线鉴权参数 `--uuid 0` 非法 UUID → Voxy `createStorage` NPE（`User.m_240411_()` 返回 null）→ 存储层初始化失败使 async init 整体退出，**Ingest worker 管线从未启动**。被测 bug 的宿主管线不在线，阴性观测（异常未出现）无判别力——不是 bug 消失，是触发 bug 的机器没开机。
- **定位**：`async init failed` + createStorage NPE 栈回溯到离线启动参数中的 uuid 字段。
- **修复**：`--uuid 12345678-abcd-3ef0-9cba-1234567890ab`（合法格式）后 Voxy 管线正常起，R5 轮 Ingest 异常照常复现（8 worker 全炸，与 R1 逐字同）。
- **教训/判据**：
  1. 离线/自动化启动载具的占位参数（uuid/playername/session）必须是**格式合法**值——依赖玩家身份的 mod（存储/存档/权限类）会静默半初始化，无显式报错面。
  2. **判别实验前先验证被测 bug 的宿主管线在场**（worker 调度日志 / init 成功日志 / 前轮复现行再次出现），否则「未复现」类阴性结论无效。
  3. 「本轮 bug 不复现」与「上轮复现」矛盾出现时，先查两轮环境参数 diff（含启动链参数），不先怀疑 bug 的环境相关性。
- **证据**：`runtime/forge-client-test/launch_client.ps1`（uuid=0 vs 合法 uuid）R4/R5 对照日志；Voxy `async init failed` 栈。

## 发现 #34 简记: Move-Item 目标父目录不存在时把源文件静默改名成目标路径（260908-04）

- 1.0.27 jar 一度「消失」成 `.tmp\coreswap-1027-client-test` 无扩展名文件：Move-Item 目标父目录不存在时把源改名为目标路径（**无 SilentlyContinue 也静默**）。AGENTS.md 八.5 坑的扩展形态（彼条记 SilentlyContinue 吞错，本条补「无开关也静默改名」）。判据：移动前 `New-Item -ItemType Directory` 确保目标父目录存在；「文件消失」先查目标路径名的同名无扩展文件。
- **置信度**：本轮已验证事实（现场文件实证）。

## 发现 #35: 外部 mod 配置枚举跨主版本改名盲区——2.x 字面值写入 3.x toml 解析失败/回落默认，且「模式语义」与「模式名」都要从目标版本 jar 一手核（260908-06）

- **时间/置信度/module**：260908-06；candidate（jar 常量池一手 + toml 落盘实锤双证）；build-tooling / 跨版本外部配置核对（**#37/#53「默认值当公理」家族的枚举值形态**）。
- **来源定位**：.investigations/mod-compat-260908-02/dh-scout-260908-06.md §1/§3 + .artifacts/dh-e2e-verdict-260908-06.md 证据链 1。
- **现象**：DH 2.x 的 `distantGeneratorMode` 枚举值 `FEATURE_GENERATOR`/`INTERNAL` 在 3.2.0 已不存在——jar 常量池一手核对（`EDhApiDistantGeneratorMode.class` values 顺序）实为 `PRE_EXISTING_ONLY`/`SURFACE`/`FEATURES`/`INTERNAL_SERVER`（2.x `FEATURE_GENERATOR` 拆分/改名 `FEATURES` + 新增 `SURFACE`，`INTERNAL` 改名 `INTERNAL_SERVER`）。按旧名字面写 toml 会解析失败/回落默认。且不只是名字：**语义也不同线**——`FEATURES` 走 DH 进程内 BatchGenerator（自持 worldgen 调用，不排队 server chunk、不写 region），`INTERNAL_SERVER` 才驱动 integrated server 的真实 chunk 管线；CoreSwap 只接管 server 管线时 FEATURES 模式根本不经过 Rust。
- **根因**：外部 mod 配置项的「可选值集合 + 语义」是目标版本 jar 的事实，不是上一版本经验的延续——跨主版本升级后枚举集合可增删改名（本例还带模式→生成步骤映射失败错误签名 `,no target step defined for generator mode: [`）。默认值当公理 = #37/#53 家族：本例默认值 `FEATURES` 由 toml 落盘实锤（首个客户端启动后 `config/DistantHorizons.toml` 出现 `distantGeneratorMode = "FEATURES"`，scout 的字节码级 candidate 推断获廉价确认）。
- **定位**：jar 常量池字符串提取（`runtime/forge-client-test/mods/` 本地已装 jar 反查）拿枚举 values 顺序 + web javadoc 佐证；运行时 toml 落盘核对默认值；日志零 `no target step defined for generator mode` 行确认模式值有效。
- **修复/判据**：① 外部 mod 配置项跨主版本升级后，枚举值 MUST 从**目标版本 jar 常量池/落盘 toml** 一手核对，禁止按上一版本记忆面写；② **「模式名」与「模式语义」分开核**——名字对上只保证解析通过，语义（走哪条生成管线）决定行为学结论是否成立（本例 FEATURES/INTERNAL_SERVER 的管线差直接决定「LOD 是否吃到 Rust 地形」）。
- **教训**：选模式前先问「哪条路径经过我接管的管线」——本例首选 INTERNAL_SERVER 才闭环「远景 LOD 继承 Rust 地形」验证（`InternalServerGenerator_forge` + populateNoise Mixin 拦截 ×7,462 实证）。
- **证据**：dh-scout-260908-06.md §1（常量池一手）+ verdict 260908-06 §证据链 1（toml 落盘 + 行为化日志双证）。

## 发现 #36: MC 1.16+ level.dat 的 world seed 在 `WorldGenSettings.seed`（TAG_Long），根 Data 无旧 `Seed` 键——NBT 手解析复合头多跳一字节静默错位（260908-06）

- **时间/置信度/module**：260908-06；candidate（本例 seed 三处逐字一致实锤：level.dat 解析值 = F3 屏显 = CppBridge init 行）；build-tooling / seed 三查·level.dat 环节（**seed 类错误三犯家族的键名盲区形态**）。
- **来源定位**：.artifacts/dh-e2e-verdict-260908-06.md 证据链 2（judge N-2 落盘补正：`.tmp/check_level_seed_260908-06e.py` 解析实得 -546755292641445454）+ dh-scout-260908-06.md。
- **现象**：在 level.dat 根 Data 下按旧键名 `Seed` 粗扫扑空（1.16+ 结构已改）；手写 NBT 解析器若把复合头按「tag(1B)+name」读而漏掉 namelen(2B)，偏移多跳一字节——**无任何报错**，后续键解析结果为空/垃圾。
- **根因**：MC 1.16 起 world seed 移入 `Data.WorldGenSettings.seed`（TAG_Long），旧 `Seed` 键不复存在，键名先验过期 = 扑空；NBT 手解析的每个字段头都是复合结构 tag(1B)+namelen(2B)+name，长度前缀漏读是结构性静默错位（NBT 无校验和，错位只表现为下游数据荒谬或空）。
- **定位**：seed 三查交叉——level.dat 解析值 vs F3 屏显 vs `[CppBridge] init seed=` 行逐字一致（三处同值即解析正确性直证）；字节级 sanity = 复合头肉眼核对 `04 00 04 's','e','e','d'`（tag=04 TAG_Long + namelen=0x0004 + "seed"）。
- **修复/判据**：① seed 三查的 level.dat 环节一律用 `WorldGenSettings.seed`；② NBT 手解析每读一个字段先做已知键字节级 sanity（复合头 4 字节对齐肉眼可见），解析值再与独立源（屏显/日志行）交叉核一遍——错位无报错面，交叉核对是唯一廉价防线。
- **教训**：「键名粗扫扑空」先疑版本结构变更（1.16 分界），不疑文件损坏；手写二进制格式解析器的错位全是静默的，sanity 锚点必须前置不是事后验结果。
- **证据**：verdict 260908-06 证据链 2（三元组一致）+ `.tmp/check_level_seed_260908-06e.py`（脚本自证）。

## 发现 #37 简记: Zstandard 容器 magic = `28 b5 2f fd`——无 zstd 通道时探测顺序 python zstandard → zstd CLI → 7-Zip，全无则降级声明不跳过（260908-06）

DH 3.x FullData blob（SQLite）为 Zstandard 压缩，magic `28 b5 2f fd`（很多新工具链默认压缩，遇到未知二进制 blob 先看头 4 字节）。本机无 zstd 通道时按序探测解压途径；三条全无则按 Anchorlaw 降级声明如实记「内容未验证」，不得静默跳过验证环节伪装成「无异常」。本例：python 库/CLI/7-Zip 全无，LOD 内容抽查降级挂账（用户拍板收口）。**置信度**：本轮已验证事实（magic 实读 + 三通道探测记录）。来源：.artifacts/dh-e2e-verdict-260908-06.md §证据链 4。

## 发现 #38: mavenCentral 经本地代理 9199 全 403 → build.gradle 换阿里云镜像；pwsh/curl 出网 SSL 全挂是常态、gradle JVM 网络栈另算——宿主网络分层不互相代表（260908-08）

- **时间/置信度/module**：260908-08；candidate（scout-1a 代理配置一手 + 本轮构建记录；镜像切换的具体 403 复现日志如另存应回填链接）；build-tooling / 网络通道。
- **来源定位**：`.investigations/mc-1216-port-260908-08/scout-1a-reference-source.md`（gradle.properties 本地代理 127.0.0.1:9199，loom 官方 jar/yarn 依赖 maven.fabricmc.net/mojang 通道）+ 本工作块 1.21.6 参照链搭建记录。
- **现象**：1.21.6 参照工程首建 `gradlew build` 需联网拉取 jar/yarn/loom 依赖；mavenCentral(repo1) 走本地代理 9199 时全部请求 403（代理对该上游拒绝）；同时 pwsh/curl 直接出网 SSL 握手全挂（本沙箱环境常态）。
- **根因（机制）**：① 代理对 repo1.maven.org 上游的拒绝是代理策略问题，不是本机网络不可用；② pwsh/curl 的 SSL 失败与 gradle 失败分属**不同进程的网络栈**（curl 用 schannel/OpenSSL，gradle 用 JVM truststore + 自己的代理设置）——一个通道挂不推出另一个通道挂，「工具级出网失败」极易被误读成「本机无法联网」从而放弃构建。
- **定位**：分层排除——先看 gradle 报错里具体 upstream URL 与状态码（403 = 代理侧拒绝而非 DNS/SSL 失败），再对比 curl 同 URL 表现，确认两层独立。
- **修复**：build.gradle/settings.gradle 仓库列表把 mavenCentral 置于阿里云镜像（`https://maven.aliyun.com/repository/public` 等）之后/替换，绕开代理对 repo1 的 403；gradle 自身走 JVM 网络栈（gradle.properties 代理配置）正常通。
- **教训/判据**：① gradle 联网构建失败先按「仓库源 × 代理策略」矩阵排查，镜像换源是 mavenCentral 403 的首选低成本修复；② **「pwsh/curl 出网 SSL 全挂」在本环境是常态基线，不作「网络不可用」判据**——判断某构建通道可用性必须用该通道自己的进程（gradle = JVM 栈）实测，宿主网络分层不互相代表；③ 新版本探针工程首建前预留联网需求声明（scout-1a G-62 已预置），依赖落 `.gradle-home/caches/fabric-loom/` 后即可离线复建。
- **证据**：scout-1a-reference-source.md §gradle.properties/要点/G-62。

## 发现 #39 简记: 多版本 dll 同名冲突裁决——薄壳 cdylib 包名带版本（worldgen1216→worldgen1216.dll）+ processResources rename 回 worldgen.dll，两版本输入互不覆盖（260908-08）

workspace 多版本薄壳并存时 cdylib 产物同名（都叫 worldgen.dll）会互相覆盖 target 产物；裁决 = 每版本薄壳包名内嵌版本号（`worldgen1216` → 产物 `worldgen1216.dll`），version-specific 的 build.gradle `processResources` 再 rename 回运行时固定名 `worldgen.dll`——版本隔离在构建产物层，运行时契约名不变。下个版本 = 新薄壳包名 + 对应 rename，模式照抄。（260905-01 workspace 拆分 §13a 的多版本延伸。）来源：`.investigations/mc-1216-port-260908-08/port-list-260908-08.md` P1。**置信度**：本轮已定稿裁决（清单 judge PASS-with-conditions 已过）。

## 发现 #40: mixin 配置文件与 mixin 类文件失同步——`required=true` 下编译期零提示、运行时装载失败（「编译绿 ≠ apply 绿」第二形态，260908-10）

- **时间/置信度/module**：260908-10；candidate（脚本集合对拍 + 1.20.1 侧 23/23 全等反证；修后 22/22 实测）；build-tooling / mixin 工程（**#25/#55 mixin 家族，「编译绿 ≠ apply 绿」第二形态**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-1；`runtime/1.21.6/java/src/main/resources/coreswap.mixins.json`（修前 23 项 vs 目录 22 个类）；`runtime/1.20.1/java/src/main/resources/coreswap.mixins.json` 23 项 vs 目录 23 个类（反证）。
- **现象**：1.21.6 侧 `coreswap.mixins.json` 的 `mixins` 数组列 23 项，实际 mixin 类文件 22 个——P4 迁移随 light ticket 删除了 `ThreadedAnvilChunkStorageAccessor`，json 未同步。`"required": true` 下缺类 = mixin 装载失败 = 服务器起不来；而 `gradle compileJava` 与单测**全绿**（json 不是 javac 的编译输入，无任何提示）。
- **根因**：mixin 的配置清单（json 数组）与类文件目录是**两份独立维护的事实**，构建链没有任何一致性校验。删/改/重命名 mixin 类时只改代码不改 json（或反之）都不报错——缺类只在**运行时 mixin 装载阶段**暴露，`required=true` 把「清单指向不存在的类」升级为硬失败。
- **定位**：4 行脚本做**集合对拍**——json 数组项 vs `mixin/` 目录 `*.java` 文件名，双向差集为空才算对齐；再用 1.20.1 侧 23/23 全等作反证，排除「脚本误报」。
- **修复/判据**：修复 = json 移除 stale 条目（修后 22 项 vs 22 类对齐）。判据（MUST）：**mixin 类增删后 MUST 脚本对拍 `mixins.json` 列表 vs `mixin/` 目录文件名集合**，并把它做成构建前置门禁；「编译绿」对 mixin 装载零判别力。
- **教训**：「编译绿 ≠ apply 绿」现有两形态：① 描述符/目标签名失配（#25/#55 家族，AP 警告或生产 APPLY FAILED）；② **清单与类文件失同步**（本条，`required=true` 运行时装载失败）。共同点 = **mixin 的有效性不在 javac 的检查域内**，必须用 mixin 自己的三重门禁兜底：json↔目录对拍 + AP `Cannot find target method` grep + 生产 apply 日志。
- **证据**：progress §T0-1；两个 json 与两个目录的集合对拍记录（本轮实读：1.21.6 json 22 项 / 目录 22 文件；1.20.1 json 23 项 / 目录 23 文件）。


## 发现 #25 补充案例（260908-10）：mixin AP 的 `Cannot find target method` 警告 = 运行时 APPLY FAILED 的可预测前兆——`defaultRequire=1` 下该串必须当错误

- **时间/置信度/module**：260908-10；candidate（AP 警告计数修前 2 / 修后 0 + 重编译 BUILD SUCCESSFUL）；build-tooling（#25 mixin 家族；**#55 判据 4「签名/refmap 错误只在生产 APPLY FAILED」的编译期可见形态**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-3；`.tmp/p2b-compile-260908-10.log:390/393`（修前 2 条）；`.tmp/p2b-compile-r3-260908-10.log:25`（修后 `BUILD SUCCESSFUL in 8s`，0 条）；一手签名权威 `versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:326`。
- **现象**：`gradle compileJava` 通过，但 mixin 注解处理器报 2 条 `警告: Cannot find target method "populateNoise(Ljava/util/concurrent/Executor;...)..."`（`NoiseChunkGeneratorMixin.java:63` / `NoiseDumpProbeMixin.java:46`）——1.21.6 去掉了 `populateNoise` 的首参 `Executor`（1.20.1 五参 → 1.21.6 四参）。`injectors.defaultRequire=1` 下该 mixin 运行时必 APPLY FAILED。修描述符 + handler 形参后重编译，警告计数 **2 → 0**。
- **根因**：mixin AP 在编译期解析 `@Inject.method` 描述符与目标类，找不到只发 **warning**（javac 层面构建成功），而 mixin 的 `defaultRequire=1` 把「目标方法缺失」升级为**运行时硬失败**——**编译器的告警等级与 mixin 的运行时严格等级不一致**，中间没有门禁把它们对齐。
- **定位**：编译日志 grep `Cannot find target method`（修前 2 / 修后 0）；目标方法签名以一手 `NoiseChunkGenerator.java` 为准逐参核对，不靠记忆/上一版本。
- **修复/判据**：修复 = 两处 `@Inject` 描述符 + handler 形参同步去 `Executor`。判据（MUST）：**mixin 项目构建日志中的 `Cannot find target method` 必须当错误处理**——构建脚本/CI 应 grep 该串做门禁，出现即 fail；跨版本升级后 mixin 目标方法签名 MUST 以一手源码逐方法核。
- **教训**：「compileJava 成功」在 mixin 项目里只证明 Java 语法/类型，不证明注入有效；AP 警告是**免费的前置信号**，漏读即把编译期可发现的问题推到运行时（#25 家族共同结论：门控/注入配置层静默不生效，只有显式门禁兜底）。
- **证据**：上述两个编译日志 + 一手源码行。


## 发现 #8 补充案例（260908-10）：跨版本新探针工程首建漏掉**整块** `-P`→`-D` 映射——参数静默不生效，「编译过」不构成接线证据（#8/#19 家族第四形态）

- **时间/置信度/module**：260908-10；candidate（两侧 build.gradle 逐项对照 + 修复后 P2b 三跑参数行为化生效）；build-tooling（#8 家族：从「漏一行」升级为「漏整块」）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-2 + §P2b 三跑表；`runtime/1.21.6/java/build.gradle`（修前只有 `processResources` + `loom.runs.server`，零映射）vs `runtime/1.20.1/java/build.gradle:56-181`（`benchVmArgs` 映射块，109 处 `findProperty`，约 126 行）。
- **现象**：1.21.6 探针工程首建时 `build.gradle` 未移植 1.20.1 的 `benchVmArgs` 映射块 → `-PbiomeProbe=true` / `-PblockProbe=true` 等参数**静默不生效**（探针不跑、无报错）。修复 = 整块移植（bench 参数 / 各探针 / `cpp.replace|cpp.lib|cpp.worldgen.dir` / `rustStages` / `cpp.blockRegister` / `featureLog`+`defaultBlock` env 通道），并把 `benchOut` 默认值改指 1.21.6 数据目录；修后 P2b 三跑（biome `biomes=7593` / block `DONE` / nether `biomes=5`）参数全部行为化生效。
- **根因**：gradle `-P`（项目属性）与 JVM `-D`（系统属性）是**两个命名域**，桥接靠 `build.gradle` 手工 `findProperty → vmArg` 逐行映射（#8 原始机制）。新版本工程从零起 `build.gradle` 时，映射块**不在编译依赖里、也不在任何模板里**——漏掉整块没有任何编译/运行报错，只是参数进不了 JVM。
- **定位**：两侧 `build.gradle` 逐项对照（`findProperty` 计数 + 属性名集合），而不是「跑一下看有没有输出」（空跑也会走默认行为，看起来正常）。
- **修复/判据**：判据（MUST）：**新版本工程首建时，`-P` 参数清单必须逐项对照上一版本复制，并以行为化日志（探针 banner / 属性回显）核验生效**——「编译过 / 构建成功」不构成接线证据；映射块建议做成可复用片段或前缀批量映射（#8 结构性修法，仍未落地）。
- **教训**：参数传递链上的静默丢弃只有**清单核对 + 行为化证据**能兜底（#8/#19/#25/#32 家族共同结论）。
- **证据**：两侧 `build.gradle`；progress §T0-2 与 §P2b 三跑结果。


## 发现 #14 补充案例（260908-10）：探针 dump 副产品文件名内嵌的是「探针参数 seed」不是「世界 seed」——文件名自证 ≠ seed 自证，误用即伪参照

- **时间/置信度/module**：260908-10；candidate（同一日志内两 seed 并存实读 + P7 修复后三处一致实锤）；build-tooling（#14 dump 自证 seed 的**字段语义陷阱**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §P2b「seed 三查留痕」+ §P7「seed 三查」；`.tmp/p2b-block-260908-10.log:216-217`（`[BlockProbe] seed=-8248318472910187742` vs `[BlockProbe] worldSeed=-5307016484385870680`）+ L1186（产物 `vanilla_-8248318472910187742_8_200_200.blocks`）。
- **现象**：`BlockProbe` 产物文件名内嵌 `bench.seed`，但实际地形由 `worldSeed`（`server.properties` 的 `level-seed`）决定——两者不同时（本例 bench seed `-8248318472910187742` vs 未设 level-seed 随机得到 `-5307016484385870680`），`vanilla_<benchSeed>_..._<origin>.blocks` 是**伪参照**：文件名声称的 seed 与文件内容的地形 seed 不一致，拿它做对拍即 seed 三查违例。
- **根因**：`#14` 要求 dump 文件自证 seed，落地时嵌入的是**工具输入参数**（`bench.seed`）而非**世界生成 seed**（`worldSeed`）——两个 seed 在探针里都存在且都有名字，但只有后者决定地形；文件名模板选了前者，自证字段就变成误导字段（**自证 ≠ 正确自证**）。
- **定位**：同一日志内 `seed=` 与 `worldSeed=` 两行并列对照；修复路径 = 先设 `level-seed` + 删 `run/world` 重导，之后 `worldSeed` = level-seed = bench seed，两臂导出 header 逐字段相同（P7 实锤）。
- **修复/判据**：判据（MUST）：① 导出参照前先设 `level-seed` + 删 `run/world`（否则 worldSeed 随机）；② 对比前核对三处一致——产物 header/文件名内嵌 seed == 命令行/bench seed == 日志 `worldSeed`；③ **dump 文件头应同时落 bench seed 与 worldSeed 两个字段并标注哪个是地形 seed**（#14 字段清单升级），只落一个时按「伪参照」处理。
- **教训**：「文件自证」只有在**自证字段与结论所依赖的语义同一**时才成立——自证字段选错比没有字段更危险（看起来有据可查）。
- **证据**：`p2b-block-260908-10.log` 两行 + 产物文件名行；progress §P7 三处一致记录。


## 发现 #41: PowerShell `Select-Object -First N` 提前关闭管道会杀掉上游进程——长时构建日志被截断成「构建中断」假象（260908-10）

- **时间/置信度/module**：260908-10；candidate（同一命令两次运行对照：截断日志 vs 全量落盘日志）；build-tooling / PowerShell 环境坑。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-3 ⚠️过程坑；`.tmp/p2b-compile-r2-260908-10.log`（截断：104 行、无 `BUILD` 行、末尾停在堆栈中间帧，夹 `java.io.FileNotFoundException: ...mixin-targetdb-*.tmp`）vs `.tmp/p2b-compile-r3-260908-10.log`（全量落盘，含 `BUILD SUCCESSFUL in 8s`）。
- **现象**：`gradle ... | Select-String ... | Select-Object -First 30` 运行时 gradle 被中途终止——日志只剩半截堆栈（末尾停在 `DefaultBuildOperationRunner.execute` 中间帧），被误读为「构建中断/构建失败」。
- **根因**：PowerShell 管道中 `Select-Object -First N` 满足数量后**停止消费并关闭下游管道**，上游进程（gradle/java）收到管道关闭后终止——early-exit 的正常语义，但对「长时间运行、日志即证据」的构建命令等于**中途杀进程**；截断的堆栈与真实失败的堆栈在观测上不可区分。
- **定位**：对照两次运行——截断版 104 行、无 `BUILD` 行、末尾非自然结束；全量版有 `BUILD SUCCESSFUL`；确认是管道早退而非构建错误。
- **修复/判据**：修复 = 长时构建/采集命令一律 `... *> <logfile>` 全量落盘，**再**对落盘文件单独过滤（`Select-String <logfile>` / `Get-Content -Tail`）。判据 = 「日志末尾停在堆栈中间帧 + 无 BUILD 行」是**管道截断签名**，先复跑全量落盘再判构建失败。
- **教训**：日志采集与日志过滤必须**两阶段分离**；任何在管道里做 early-exit 截断的写法都会把「采集侧副作用」伪装成「被测系统故障」（与 #29 RCON 帧解析伪装「服务器挂死」同构：工具层假象优先排除）。
- **证据**：两个日志文件对照（r2 截断 / r3 完整）。


> 260908-11 追加：build-tooling 新增**发现 #42 简记**（JNA tmpdir「拒绝访问」瞬时错误——瞬时环境错误重跑优先于立案 + JAVA_TOOL_OPTIONS tmpdir 固化进驱动脚本）+ **#14/#18 补充案例·高价值**（伪参照跨块存活在权威 data 目录——路径权威≠内容权威；三步交叉定责法：接管证据→dll/工作区 diff→旧 R 臂交叉 diff 100.0000%；已 confirmed 数字 99.9993% 是现成复现锚；来源：.investigations/mc-1216-port-260908-11/errors.md E1/E7）。

---

### 发现 #42 简记: JNA 临时文件「拒绝访问」瞬时错误——瞬时环境错误重跑优先于立案 + 驱动脚本固化 `JAVA_TOOL_OPTIONS` tmpdir 规避（260908-11）

- **时间/置信度/module**：260908-11；candidate（一次复现一次自愈 + tmpdir 固化后未复发）；build-tooling / JVM 环境坑（简记；#38 平台分层家族环境面）。
- **来源定位**：`.investigations/mc-1216-port-260908-11/errors.md` E7；progress-260908-11.md 载体备注。
- **现象**：runServer 启动一次报 `UnsatisfiedLinkError: Failed to create temporary file for jnidispatch.dll: 拒绝访问`，重跑即恢复。
- **根因**：沙箱下 java.io.tmpdir 提取 JNA 临时 dll 偶发被拒（环境面瞬时，非代码缺陷）。
- **修复/判据**：修复 = Chunky/探针驱动脚本统一 `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=.tmp\java-tmp`（1.20.1 驱动已有先例，本次固化到 1.21.6 驱动）。判据 = **一次性、重跑即消的环境错误先重跑复现次数再决定是否立案**；复发模式的环境坑才升级为脚本级固化规避。
- **教训**：驱动脚本固化规避要进脚本本身（不靠记忆），跨版本复制驱动时随命令清单一并带走。

---

## 三、补充案例：#14/#18 家族（build-tooling）——E1 伪参照跨块存活在权威数据目录

### #14/#18 补充案例（260908-11）：伪参照跨块存活在「权威数据目录」——路径权威 ≠ 内容权威；已 confirmed 数字是现成复现锚

- **时间/置信度/module**：260908-11；candidate（旧 R 臂交叉 diff 定责 100.0000% + 真参照复现 99.9993%）；build-tooling（#14「文件名内嵌 bench.seed ≠ worldSeed → 伪参照」260908-10 案例的**跨块存活**新形态 + #18「在盘 ≠ 本次执行体生成」目录维度）。
- **来源定位**：`.investigations/mc-1216-port-260908-11/errors.md` E1；`.tmp/p7-260908-10/vanilla/`（真参照）。
- **现象**：1.21.6 P7 R 臂复跑与 `versions/1.21.6/data/vanilla_-8248318472910187742_8_200_200.blocks` 对拍 = 89.08%、biome 14160/16384 差（ocean↔deep_ocean）→ 疑似大回归。该文件是 NEXT_SESSION 260908-10 已警告的伪参照（17:44 导出时 level-seed 未设 → worldSeed=-5307 污染副产品），**却放在正式 data 参照目录里且文件名内嵌正确 seed**，跨块存活一天后再次被当真参照消费。
- **根因**：测量侧错误。伪参照未清理出权威目录——「位于 data 目录 + 文件名内嵌 seed」两信号都是**位置/命名权威**，不构成**内容权威**；对拍异常时先怀疑实现回归，把测量侧排最后，成本倒挂。
- **定位（怎么发现的，可复用）**：三步交叉定责：① 核接管证据（807 intercepted ✓，排除未接管）→ ② 核 dll/工作区 diff（仅 F1 一分支，量级远小于 89%）→ ③ **旧 R 臂交叉 diff**：新 R vs 旧 R = 100.0000%，旧 R vs 伪参照 = 同一 89.08% → 锁定测量侧。改用 `.tmp/p7-260908-10/vanilla/` 真参照 → **99.9993%/41/biome 0**，与已 confirmed 结论逐位复现。
- **判据/如何利用**：
  1. **参照谱系核对是任何异常对拍的第一动作**——先核参照文件（来源 run、seed 头、导出时间）再怀疑实现；「已警告过的伪参照」会换目录/换块存活，警告不清除文件本身则必再犯。
  2. **已 confirmed 的结论数字是现成复现锚**——复跑结果对不上时，先对「旧产物 vs 旧产物」做交叉 diff 分离测量侧/实现侧（两侧都过 confirmed 锚 = 实现没变）。
  3. data 目录里的参照文件建议带谱系注记或在收录时核对 seed 头（内容指纹），防位置权威冒充内容权威。
- **证据**：errors.md E1 全段；99.9993% 复现数见 verdict-f1f2-chunky-260908-11.md §F1。

---



### 发现 #43: (k,p,fid) 单 chunk 序列探针——decorator seed 域错位的决定性判据；MixinExtras 不可用时的纯 Mixin 替代（260909-02）

- **时间/置信度/module**：260909-02；candidate（step 1-8 共 40 条完全一致 + step 9 p 错位实锤定位根因，behavior 级探针证据）；build-tooling / 探针工具（#81 行为化日志家族的序列化形态 + #62 seed 判据的调用点级落地）。**价值门：高（判据 + 工具做法直接复用）**。
- **来源定位**：`.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md` §一.4 + `.investigations/mc-1216-features-takeover/fanout-260909-02/b1-rng-wiring.md` §4（探针设计）。
- **做法**：单 chunk 双侧对拍 `(population_seed l, step k, index p, fid)` 全序列：Java 侧 mixin 在 ChunkGenerator.generateFeatures :402 `setDecoratorSeed(l, p, k)` 处打印四元组 + registryKey（本块实现 = **@Redirect placedFeature.generate 调用点**顺带打印——MixinExtras 编译期不可用时的纯 Mixin 替代；另 `setCurrentlyGeneratingStructureName` 供应商重定向可拿结构 fid，本块结构段走种子隔离不需）；Rust 侧 `WG_FEATURELOG=1` 现成钩子（worldgen_handle.rs:1080）。
- **判据（三层直接命中）**：`l` 不一致 → seed 接线差（worldSeed/公式）；`l` 一致、`(p, fid)` 序列有差 → **p 域输入差**（registry 序文件/biome feature list 构建序）；序列完全一致 → RNG 接线整体排除，归因转特征实现层。本块实测：step 1-8（40 条）完全一致，step 9 特征集相同（13 fid 一致）但 p 错位（trees_water J=50/R=28、flower_default J=57/R=53 等）→ 根因钉死为 **Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致**。
- **教训**：① 「全局位置整体错开」类症状，序列探针一步区分「种子域错位」vs「算法实现差」，先于任何逐族算法对拍；② p 索引差（数值不同但 fid 集合相同）是**种子域错位的决定性签名**——特征集合一致恰恰排除了「缺 feature」候选；③ 静态源码对读（b1 候选 Degraded）已把公式/类型/迭代序三层核到同构后，剩余疑点收敛到「输入数据域」，探针设计应直接对准该域。
- **证据**：probe-{javafeat-log,rustfeat-err}-snapshot.log（.tmp/mc1216-closeout-260909-02/）；mixin 代码 runtime/1.21.6/java ChunkGeneratorFeaturesMixin（不入库，batchD-record 为追踪载体）。

### 发现 #44（简记）: 接管开关单 flag 双向切换语义——mask=0 必须显式排除出「接管生效」判定（260909-02 收编 batchD）

- **时间/置信度/module**：260909-02 收编（batchD 改动 2）；candidate；build-tooling / 接管开关设计。**价值门：中（简记——跨版本复制接管开关时的语义陷阱）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/batchD-record-260909-01.md` 改动清单 2。
- **要点**：`rustFeaturesTakeover() = enabled && mask!=0 && (mask&0b010)==0`——**mask=0（全 Java = 双跑对照意图）必须显式排除**，否则「无 mask 参数」与「mask=0」两种语义被合并，对照臂误走接管路径；单 flag 双向切换（默认 0b011 现状不变，回退 = 删 -D 参数）。翻转前置（verdict §二）：p 域对齐 + 已知 feature 族缺陷修复 + 重对拍信噪比回噪声量级，**残差未收敛前不翻转**。另：接管开关生效判定要配行为化哨兵计数（首拦 + 周期打点），纯布尔回读不构成生效证据（#81 家族）。
- **证据**：batchD-record 改动清单 + verdict §二（mask 翻转不建议，前置清单）。

---

### 发现 #96: 执行体三元组核验的 dev-run 形态——gradle runServer 实际加载 build/resources/main 下的 dll，build/libs jar 时间戳是红鲱鱼（260909-03）

- **时间/置信度/module**：260909-03；candidate；build-tooling / 构建产物核验（#23 家族新形态 + #18「导出产物在盘≠本次执行体生成」的 dev-run 面延伸）。**价值门：高（判据 + 直接证据行复用）**。
- **来源定位**：`.investigations/mc-1216-features-takeover/t3-rerun-record-260909-03.md` §执行体三元组核验。
- **现象**：T3 重对拍前核验执行体：build/libs 下 jar 时间戳停在 13:26（早于 16:07 的 dll 构建），按 jar 时间戳判断会误判「构建链陈旧、跑的是旧 dll」。
- **根因**：`gradle runServer`（loom dev run）不走 build/libs jar——dev classpath 直接加载 `build/resources/main/native/worldgen.dll`（processResources 从 `target/release/worldgen1216.dll` 拷入并 rename）。jar 的时间戳/内容与本次运行无关，是红鲱鱼。
- **定位/判据（执行体三元组核验，本形态版）**：加载文件 sha（build/resources/main/native/worldgen.dll）vs target 产物 sha 逐字节比对——本轮 sha16=02d897a3（2,411,008 B）一致；**processResources 日志行「synced Rust dll: N bytes」是同步发生的直接证据**（字节数与 target 产物大小一致）。三元组 = 加载文件 sha / target 产物 sha / 同步日志行，三者齐才认「跑的是本次构建」。
- **教训**：执行体核验判据必须跟着「实际加载路径」走，不能沿用上一次运行口径（jar 口径 vs dev classpath 口径）；核验前先确认本次 run 的 classpath 形态，再选三元组的「加载文件」一端。另：processResources 的 inputs.files 含 dllFile 存在性在**配置期**判定——dll 不存在时启动的门（up-to-date/跳过判定）不随 dll 后续出现而自动更新，dll 后补时强制重跑 processResources。
- **证据**：t3-rerun-record §执行体三元组核验（sha16 02d897a3 / 2,411,008 B / synced 日志行 / jar 13:26 vs dll 16:07 对照）。

## 发现 #45: PowerShell `-like '*[CA-NT]*'` 通配符字符类坑——`[...]` 是字符集不是字面量，日志过滤混行污染求和（260909-06）

- **时间/置信度/module**：260909-06；确定（首求和作废重算实测）；build-tooling / PowerShell 坑。
- **来源定位**：`.investigations/camin-perf/probe-260909-06.md` 恒等式自检节末；载体 = 诊断日志求和脚本（.tmp）。
- **现象**：用 `-like '*[CA-NT]*'` 风格过滤含 `[CA-NT]` 标签的诊断日志行，结果混入 `[CA-MEMO-TOP]` 等其他标签行，计数求和虚高，首版汇总作废。
- **根因（机制）**：PowerShell 通配符里 `[...]` 是**字符类**（匹配括内任一字符），`*[CA-NT]*` = 「含 C 或 A 或 - 或 N 或 T 的任意行」——不是字面子串 `[CA-NT]`。诊断日志标签恰好全用大写字母，过滤器退化为「几乎匹配所有行」。
- **定位**：求和与手数行数不符 → 打印被选中行样本发现 MEMO-TOP 行混入 → 逐字符核对通配符语义。
- **修复**：字面过滤改 `.Contains('[CA-NT]')`（或转义 / `-SimpleMatch`）。
- **教训/判据**：PowerShell 通配符里出现 `[` 一律先想字符类；对**含方括号标签的日志**做计数/求和类汇总，过滤后必须打印样本行核纯度（混行即作废，#13 sanity 家族同型）。
- **家族索引**：#41、#13。

### #27 家族补充案例（第三形态）: existence-only marker 跨版本陈旧缓存 → blocks.json id 域错位 982/1003 → 大规模错块级联 → 百万实体 OOM 冻结——「同 dll+权威数据干净而生产爆」先查数据缓存指纹（260910-03）

- **时间/置信度**：260910-03；candidate（机制闭环：Temp 缓存 sha 铁证 + 修复后用户 confirmed 冻结解决；**错误优先·高价值**）。
- **来源定位**：`.investigations/vivo-freeze-260910-03/record.md`；产物 `.tmp/hang-repro-260910/`。
- **现象（五段式·现象）**：1.21.6-0.1.0 vivo 生产客户端游玩 ~40s 后世界冻结（两种表现：日志戛然而止 / 级联卡顿+`Too many chained neighbor updates`×15 + `Can't keep up! 101531ms` + `OutOfMemoryError: Java heap space`@-Xmx16384m）；地形大规模错块（用户视觉人证：「灰色的草」「沙砾」「假基岩」）；实体普查 region/entities NBT = `minecraft:item ×1,031,765` + falling_block×79；Server thread 线程 dump RUNNABLE 烧 CPU 166-2340s 在 FallingBlockEntity tick 实体碰撞扫描。同 dll + 同 seed 的 dev 服/Chunky 4225/forceload 1600 全绿零异常。
- **根因（机制）**：`CoreSwapFixHelper.extractWorldgenDir()` 数据缓存 = 跨版本共享**固定 Temp 路径**（`coreswap-data`），新鲜度判据只有两个 **existence-only marker**（overworld.json + tags marker）。用户机器上 1.20.1 生产 jar 留下的旧缓存 marker 全在 → 1.21.6 jar 跳过解压 → **1.21.6 dll 读 1.20.1 blocks.json**：两版块表 1003 vs 1105 项，982 个同名块 id 错位 → 每个 chunk 大规模错块 + 无支撑重力块 → 级联坍塌 + 流体/邻居更新链 → 百万 item 实体 → O(n²) 实体碰撞 + 16GB 堆 OOM → 冻结。这是 #27「marker 语义超载」的跨版本整集复用形态：existence marker 只证「有数据」不证「是**本版本**数据」——#27 是增量缺失（静默 fallback），本案是整集陈旧（灾难性错块），同一根因的两种表现面。
- **定位（怎么发现的）**：多轮绕路后一锤定音的动作 = **比对客户端 Temp 缓存 blocks.json sha vs jar 内 blocks.json sha**（ee01b749 = 1.20.1 内容 vs 617c3dae = 1.21.6 jar 内）——一次指纹核对即破案，此前 Chunky/forceload/dump 多轮载体验证全部无效（见 workflow-patterns 载体偏差条目）。
- **修复**：`extractWorldgenDir` 加**内容指纹**（jar blocks.json vs 缓存逐字节比对，不一致整体重解压；对齐 `extractNativeDll` 已有同款逻辑）。修复 jar sha 9eae48cd…（16:44），用户实机 confirmed 冻结解决。
- **教训/判据（可复用）**：
  1. **「同一 dll + 权威数据直读全干净，生产客户端爆」= 数据缓存指纹第一嫌疑**——dev/验证载体走 `-PcppWorldgenDir` 直读权威数据，生产走 Temp 解压缓存，两者数据通路不同；dll 相同不能证明数据相同。
  2. existence-only marker 的语义边界：只证「缓存非空/上次解压过」，不证「缓存 = 当前版本资源集」——**跨版本共享缓存路径必须配内容指纹**（逐字节或 hash 比对锚点文件），extractNativeDll 早已有同款而 extractWorldgenDir 没有 = 同类判据接线不齐的欠账。
  3. 家族索引：#27（marker 单判·增量缺失形态）、#18（产物在盘≠本次生成）、#96（加载路径跟实际 run 形态走）——共同上位原则：**「数据/产物与当前版本资源集等价」必须有独立证据，existence 是最弱的一档**。

## 发现 #46: 沙箱内 WMI/CIM cmdlet 会阻塞、JVM attach 工具一律拒访——编排脚本配方：进程属性识别 + .NET `Process.Threads`（260910-04）

- **发现时间 / 置信度 / module**：260910-04；确定（本块一手实测：cmdlet 阻塞/报错 + `jcmd` IOException 双证）；build-tooling / 沙箱环境坑（错误优先）。
- **来源定位**：`.investigations/perf-regression-260910-04/record.md` §6 E1/E4；驱动脚本 `.tmp/perf-reg-260910-04/*.ps1`。
- **现象（五段式·现象）**：① 跑批脚本「启动了服务器但从不发 RCON 命令」，进程挂死，服务器日志无 RCON Client 行；② `Get-CimInstance` 报「无法从客户端访问 CIM 资源」；③ `jcmd <pid> Thread.print` 返回 `IOException: 拒绝访问`，dump 文件 8 字节（空壳）。
- **根因（机制）**：① 本沙箱**没有 WMI/CIM 后端**——依赖它的 cmdlet（`Get-NetTCPConnection` 等）不是抛异常而是**阻塞**（失败形态 = 静默挂起）；② 沙箱**禁命名管道**，JVM attach 机制走命名管道 ⇒ `jcmd`/`jstack`/`jmap` 一类 attach 工具**一律不可用**（与命令参数/权限无关）。
- **定位**：手动 RCON 可用 + 日志无 RCON Client 行 ⇒ 卡点在脚本「发现服务器/端口」步；`Get-CimInstance` 直接复现 CIM 后端缺失；jcmd 报错文本 + dump 8 字节确认 attach 失败。
- **修复**：编排脚本**禁用一切 WMI/CIM cmdlet**，改用「WorkingSet 最大 / CPU 最高 的 java 进程」识别服务器；线程状态观测改 .NET `Process.Threads`（`ThreadState`/`WaitReason`）。
- **教训/判据（可复用）**：
  1. **沙箱可用性判据必须实测，不能凭常识**——被拦的形态可能是**阻塞**而非异常；脚本静默挂死先怀疑「某步 cmdlet 后端不可用」，而不是先查被测程序。
  2. 需要被禁通道（命名管道 / CIM / attach）的能力，开工前先做**最小可用性试跑**，再决定观测面（本块为此多绕一轮）。
  3. 「产出文件已生成」≠「有内容」——8 字节 dump 是假证据（核大小/内容；同族 #42 瞬时拒访，另有本课题错误台账 E5「日志 tail 滞后 → 结论取落盘终值」）。
  4. 替代面清单：**进程识别** → 进程属性枚举（WorkingSet/CPU 排序）；**线程状态** → .NET `Process.Threads`；**服务端口** → 被测程序自身日志/RCON 可达性探测（不要用 WMI 网络 cmdlet）。
- **家族索引**：#42（沙箱拒访/瞬时环境错误）、#21（编排脚本外部状态依赖）、workflow-patterns #106（冻结类故障观察法——本条为其 dump 通道的沙箱替代面）。


## 发现 #47 简记: gradle `-P`→`-D` 映射的「作用域」坑——`run` 只在 `loom.runs` 闭包内可见，写进 `tasks.matching{}` 会配置期失败（260910-04）

- **发现时间 / 置信度 / module**：260910-04；确定（一手构建日志 + 移回后映射生效）；build-tooling / 构建配置（#8/#19 家族新形态——**现网该家族已计至「第四形态」（260908-10 补充案例），故本条按序计为第五形态**；任务书简述的「第三形态」与现网计数不符，以现网为准）。
- **来源定位**：`.investigations/perf-regression-260910-04/record.md` §6 E2 + §5；`runtime/1.21.6/java/build.gradle`。
- **现象**：构建日志 line 235 报 `Could not get unknown property 'run' for task ':runServer'`（**配置期**失败，runServer 起不来）。
- **根因（机制）**：映射被写进 `tasks.matching { it.name == 'runServer' }` 闭包——`run` 只在 `loom.runs` 的 `benchVmArgs` 闭包作用域内可见；`tasks.matching{}` 的闭包委托对象是 Task，无 `run` 属性。这是「参数接线」家族的**第三维（作用域）**：通道对（property 通道）、名字对（映射行名）、**位置错（闭包）**→ 仍不通。
- **修复**：映射移回 `benchVmArgs` 闭包（`-Pmixlog / -Pchunktime / -PmaxBgThreads / -PcaMin / -PestL2 / -PcaCap`）。
- **教训/判据**：
  1. 新增 `-P`→`-D` 映射 MUST 核**三查：通道 / 名字 / 闭包位置**（#8/#19/#25/#32 家族的第三维）。
  2. 本形态是**响亮失败**（配置期 unknown property），优于家族的静默不生效形态——**配置期报错先查闭包作用域，别去查参数名**。
  3. 「编译过/配置过 ≠ 接线生效」通用判据不变：生效证据 = 被测程序回显/行为化日志（#81/#37），构建成功不构成接线证据。
- **家族索引**：#8、#19、#25、#32、#81/#37。


## 发现 #48 简记: `merge_index.py` 对「顶层 list 形态」的 index-entry.yaml 片段崩溃——片段 schema 契约不统一（260910-04）

- **发现时间 / 置信度 / module**：260910-04；**机制部分 Degraded**（本 subagent 不能跑命令，未亲自复现）；build-tooling / 工具坑（index 合并链）。
- **来源定位**：`.artifacts/8576-24blocks/biome-fix/index-entry.yaml`（本次直读确认其顶层是 `- id: ...` **YAML 序列**，供粘贴到 `entries` 之下）；崩溃记录 = 主会话按 `ref_merge_index` 合并该片段时实测 `AttributeError: 'list' object has no attribute 'get'`（一手运行记录，本稿未复现）。
- **现象**：`merge_index.py` 合并上述片段时抛 `AttributeError: 'list' object has no attribute 'get'`。
- **根因（机制）**：**片段 schema 契约不统一**——worker 交付的是「entries 追加片段」（顶层为序列），而合并脚本假定顶层是**单个 entry 映射**（对解析结果直接调 `.get(...)`）；序列对象没有 `.get` ⇒ AttributeError。即「脚本假定 vs 交付形态」错配，属 #8 家族的**契约维**形态。
- **修复（建议）**：统一 fragment 契约为「顶层 = 单个 entry 映射」（或让脚本显式接受 list 并逐条合并）；应用前把 list 形态片段改写为逐条映射，或主会话手工合并；脚本对输入形态 SHOULD 显式校验并给可读错误。
- **教训/判据**：① 交付 + 合并的组合 MUST 钉死**片段 schema**（顶层类型/字段名/嵌套层级），写进 `core.artifact` §5.1 的片段约定并由脚本强制；② `AttributeError: 'list' object has no attribute '<mapping method>'` 是「工具假定映射、实际收到序列」的签名，遇到直接查片段顶层类型即可，不必调试合并逻辑；③ 交付方声明的形态（文件头注释「追加到 entries 末尾」）不等于脚本接受的形态。
- **家族索引**：#8/#19/#25（契约/接线维）、#33 家族（交付形态与消费方不一致）。

## 发现 #49 简记: PowerShell `pwsh -File script.ps1 -Arms a,b,c` 把逗号串当**单个字符串**——多值参数必须用数组 `@("a","b")`（260910-05）

- **发现时间 / 置信度 / module**：260910-05；**签名侧确定**（一手报错原文 + 修复后跑通）；build-tooling / 驱动脚本参数传递。
- **来源定位**：`.investigations/perf-closeout-260910-05/record.md` §2.1「踩坑 E1」；脚本 `.investigations/perf-closeout-260910-05/cmd-output/run_arms3.ps1`（臂定义处）。
- **现象**：`pwsh -File run_arms3.ps1 -Arms r3,r3log,r3feat,r3sync` ⇒ 脚本报 **`unknown arm r3,r3log,r3feat,r3sync`**——被拒的名字是**带逗号的整串**，即逗号串作为一个臂名进入参数（零臂可跑）。
- **根因（机制）**：逗号数组语法在 **`-File` 的实参串**上未生效，整串被当成单个字符串绑定到数组形参（得到长度为 1 的元素）。⚠️ 本块未做 PowerShell 参数绑定的最小复现 ⇒ **机制侧 Degraded（表述以实测签名为准）**，签名侧确定。
- **定位**：看**报错里的名字形态**即可判别——出现「带分隔符的整串」= 分隔符未解释；若只取到第一个名字则是另一类签名（绑定截断）。本案属前者。
- **修复**：调用侧显式构造数组 `& <script> -Arms @("r3","r3log",...)`（或在脚本内对字符串参数做 `-split ','` 兜底）。
- **教训/判据**：① `-File` 传参一律按**字符串语义**预期，多值 MUST 用 `@(...)` 显式数组；② 驱动脚本启动时**回显实际收到的参数**（本案 results 行含 `args=`/`stages=`/`calog=`/`world=`，使每臂口径事后可回查）——#37/#81「生效证据必须行为化」的**编排侧**形态；③ 批量跑批脚本的臂名解析失败应**立即非零退出**（本案是响亮失败，优于 #20/#53 家族的静默不生效）。
- **家族索引**：#37/#81（生效证据行为化）、#20/#53（参数静默不生效——本条为「响亮失败」对偶）、#28（冒烟/存档口径参数集）。

## 发现 #50: `merge_index.py` 写回根 index 是「解析 + **重建**」——注释与四字段以外的一切被静默丢弃（本仓库根索引 1200+ 行、其中 300+ 行注释承载结论摘要）⇒ 带注释的根索引禁跑该工具（260910-05）

- **发现时间 / 置信度 / module**：260910-05；**确定**（一手读工具源码 + 现网根索引实测注释行数；本块已按规避方案执行）；build-tooling / index 合并链（#48 同脚本的**第二形态**：一个是崩溃，一个是静默损毁）。
- **来源定位**：`scripts/merge_index.py`：`:37` `ENTRY_KEYS = ('id','path','kind','status')` + `:49-56` `norm_entries`（每条 entry 白名单化为四字段，`:55` = `{k: e.get(k,'') for k in ENTRY_KEYS}`）+ `:129-139`（写回时**新建整个文档** `{schema_version, project, module, entries}` 后 `yaml.safe_dump` **覆写**根文件）；现网 `.artifacts/index.yaml`（共 1229 行，以 `#` 开头者 300+ 行；本批写入后实测）；规避实例 = `.artifacts/index.yaml:1188-1189`（手工追加 + 就地写下「勿跑」警告）+ `record.md` §1.3。
- **现象/风险**：对该根索引跑合并后，**全部注释**与**非四字段信息**消失（本仓库根索引正用注释承载各块结论摘要，如 `:1194-1204` 的 260910-05 块）；无警告、无备份、**退出码 0**。
- **根因（机制）**：工具是 **parse → 重建 → dump**，而不是「原地编辑」。三处叠加：① 条目被 `norm_entries` 白名单化成四字段（未知/扩展字段丢）；② 写回 dict 只含四个顶层键（顶层其它键丢）；③ YAML 加载器**不保留注释**（注释丢）。⇒ 只要写回路径经过 parse/dump，**注释必然丢**（语言层面事实，无需复现）。
- **定位**：读脚本「写回」段（看它是 edit 还是 rebuild）+ 数现网目标文件的注释行数（307）+ 对照本仓库根索引的实际用法（注释型承载）⇒ 直接判定不兼容。
- **修复/规避**：**带注释的根索引 MUST NOT 跑 `merge_index.py`**——改**手工追加** entry（本案做法；或先备份注释、跑完恢复）。根治方向：工具改 ruamel.yaml round-trip，或把结论摘要从注释迁为条目字段。现网已在 `.artifacts/index.yaml:1189` 就地写下警告。
- **教训/判据**：① 对「**承载结论/说明的索引文件**」跑任何自动合并工具前，MUST 先核「写回是**原地编辑**还是**重建**」——重建式工具 = 注释与未知字段一律丢，且通常静默；② 本工具的契约是「**四字段 entries 集合**」，**不是**本仓库根索引的维护器：#48（片段形态崩溃）+ 本条（写回语义）合起来 = 两侧都不适配，本仓库应固定用「手工追加 + 注释承载」；③ **静默数据销毁签名** = 退出码 0 + 无警告 + 目标文件结构被替换（比报错危险得多，同族 #27 existence-only marker 静默 fallback、#16「死分支」）。
- **家族索引**：#48（同脚本 · 片段形态面）、#27 家族（静默退化/静默销毁）、#18（产物在盘 ≠ 本次生成——本条为「工具成功 ≠ 内容保留」）、#10（产物判新旧用内容指纹）。

### #26 家族补充案例（260910-05）: Chunky 载具的**维度扩展**——`chunky world minecraft:the_nether|the_end` 可用 + region 路径 `DIM-1`/`DIM1` + 首用 sanity 判据

- **发现时间 / 置信度 / module**：260910-05；确定（一手九臂 `results.txt` + 日志）；build-tooling / 验证载体（#26 的维度面）。
- **来源定位**：`.investigations/perf-closeout-260910-05/record.md` §5.2/§5.3；`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md` §2；一手 `.tmp/perf-reg-260910-05/results.txt`（`naS-r1` 行：`world=minecraft:the_nether … mixinLines=4761 interceptDim=4761`）。
- **内容（三点）**：① 命令 `chunky world minecraft:the_nether` / `minecraft:the_end` **被接受**（`Task finished for … Processed: 4225 chunks (100.00%)`）；② 存档 region 路径按维度分目录——nether = `run\world\DIM-1\region`、end = `run\world\DIM1\region`（驱动脚本用 `reg` 字段显式指定，别照抄 overworld 的 `region`）；③ 首用 **sanity 判据** = `[Mixin] populateNoise(nether|end) intercepted` 行数 **+** `[WG-FILL]` 行数（本案两维各 4761 = 4761）；两者均依赖 `-Pmixlog=1`，故 **A/B 臂未开日志时 `interceptDim=0` 属预期**，不能读成「接管没生效」（形态证据改用 `inflight max`）。
- **判据**：新维度载具首用 MUST 先过**三查**再开 A/B——维度名被接受 / 接管生效有行数（含写回行）/ region 路径存在；「接管计数为 0」先核**日志门控是否开**（#25/#8 家族门控），再怀疑管线。
- **家族索引**：#26（Chunky 区域级载体——本条为其维度扩展）、#25（mixin 门控 sysprop/env）、#80（时序/触发条件）、#107 家族补充案例（本批 A4——本条为其**载体侧**判据）。

## 发现 #51: 多项目 gradle 构建下**裸任务名会级联**到子工程——1.20.1 根项目 `runServer` 连带 `:content-test:runServer`（第二服务器共用同一 `run` 目录）（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手日志任务行 + 对照臂差异；错误文案一手）；build-tooling / gradle 任务接线（#8/#47「接线/映射错觉」家族**第六形态：任务名作用域**）。
- **来源定位**：错误台账 `.investigations/perf-reg-260910-06/errors-260910-06.md` **E2**；驱动就地注释与修好后的调用 = `.investigations/perf-reg-260910-06/cmd-output/run_arms_1201.ps1`（`@(":runServer") + $extra`）；对照臂 = 1.21.6 侧只有 `> Task :runServer`。
- **现象**：1.20.1 臂日志出现 `> Task :content-test:runServer`（**两次**），随后 `Failed to start the minecraft server … testcontent.TestContentMod.<clinit> … IllegalStateException: This registry can't create intrusive holders`；该子工程服务器**共用同一个 `run` 目录 / 同一个 world**；1.21.6 臂的 `^> Task` 列表无此行。
- **根因（机制）**：在仓库根执行**裸任务名** `gradle runServer` 时，gradle 名称匹配命中**所有子工程**的同名任务；1.20.1 的 `content-test` 是独立 loom 工程，其 `runServer` 必然失败（测试内容 mod 不能独立启动），且与主服务器抢同一 world 目录。
- **定位（便宜的自证）**：`Select-String -Pattern '^> Task'` 列出 gradle **实际执行**的任务（级联任务是一行显式日志，**不看必漏**），与对照臂任务行集合比对即可判。
- **修复**：驱动改用**根项目限定名** `gradle :runServer`。
- **教训 / 判据**：① **多项目构建里任务名 MUST 限定到根项目**（`:<root-task>`）；② 「我以为我跑的就是那个任务」的接线错觉家族（#8/#9/#19/#47）在此多一维——**任务名作用域**，判据 = `^> Task` 列表与对照臂不一致；③ **主服务器照跑不构成「批次干净」**：与之并存的子工程任务失败/抢资源可同时发生；④ ⚠️ **归因链留档（同形不同因）**：该级联一度被当作「`Processed` 恒 0」的嫌疑，最终由 E1 的线程栈 dump 定因为**自锁死**——级联是**真坑但不是本块卡死根因**。「现象同形」不等于「根因同一」，两者各自需要独立证据。
- **家族索引**：#8/#9/#19/#25（`-P`→`-D` 接线链）、#47（映射**作用域**——本条为**任务名作用域**姊妹条）、#15/#28（run 口径参数集）、workflow-patterns #81/#37（生效证据行为化——本条证据面 = 任务行）。

## 发现 #52: Chunky 任务状态**跨 run 持久化**在 `config/chunky/tasks/`，删 `run/world` 不清它 ⇒ 残留任务让 `chunky start` 静默不开始，伪装成同形不同因的「卡死」（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手回显字面量 + 清目录后同臂跑通）；build-tooling / 验证载体（#26 Chunky 载体系列；#18 残留态家族第三形态）。
- **来源定位**：错误台账 **E3**；驱动修正 = `cmd-output/run_arms_1201.ps1`（每臂 `Remove-Item run\run\config\chunky\tasks -Recurse -Force`，紧邻既有的删 world）；Chunky **1.3.146**；任务状态文件已补档 `cmd-output/chunky-task-state/260910-06-overworld.properties`。
- **现象**：某臂 `chunky start` 回 **`[Chunky] A task was already started for this world. … type '/chunky confirm'.`**——该臂因此**根本没有新任务**，最终表现为「`Processed` 恒 0」，与 E1 的卡死**现象同形、机制不同**。
- **根因（机制）**：Chunky 把任务状态落到 `run\config\chunky\tasks\<namespace>\<dim>.properties`（实测 `chunks=…, cancelled=false`），**删 `run\world` 不会清它**；驱动原来只删 world ⇒ 上一臂残留任务状态污染本臂。
- **定位**：回显字面量本身即判据；再核 `config\chunky\tasks\**` 的 `cancelled/chunks` 字段。
- **修复**：驱动每臂开跑前清 `run\config\chunky\tasks`。
- **教训 / 判据**：① **「清环境」清单 MUST 覆盖工具自己的状态目录**，不只世界/存档目录——否则「上一臂的残留」会伪装成「本臂的失败」，制造**同形不同因的假因果**（#18 的第三形态）；② 遇「工具明明该开始却什么都没做」，**先读工具自己的回显/状态文件**，再怀疑被测管线；③ 同形现象（`Processed` 恒 0）在本块集齐三种因（E1 自锁死 / E3 残留任务 / 级联干扰的初判），**判据必须能区分因，不能只看现象**。
- **家族索引**：#18（残留世界缓存制造假象——本条为其**工具状态目录**形态）、#26（Chunky 区域级载体——本条为其前置清理条件）、#46（沙箱/工具环境坑）、workflow-patterns #113（同形不同因判据）。

## 发现 #53: region 对拍工具在**无 `xPos` 键的载体**上必须按「region 文件名 + 槽位索引」推导坐标；且**「与已验证版逐行对齐」是必要非充分——还须过 NBT 规范**（`tag7` 长度 = TAG_Int(4B)）（260910-06）

- **发现时间 / 置信度 / module**：260910-06；**candidate**（一手：首版 760 chunk 且不报错；对齐后自比 0 差 + 正对照非零；**再由规范核对抓出继承缺陷 46/7749**）；build-tooling / MCA·NBT 工具链（#20 的**第二/第三形态**：本条是「payload 长度读错 ⇒ 指针走错」，#20 是「指针根本不推进」）。
- **来源定位**：工具 = `.investigations/perf-reg-260910-06/cmd-output/diff_1201.py`（`payload()` 已注明与 260910-05 版 `diff_arms.py` 对齐 + 本轮 `tag7` 规范修正就地注释；坐标推导 `cx = rx*32 + (slot & 31)`、`cz = rz*32 + (slot >> 5)`）；自检 = `diffs-fixed/diff-self_os.txt`（`common=7749` / `diff=0`）；正对照 = `diffs-fixed/diff-ctrl_vanilla_x_async.txt`（247,636 块差）；tag 普查件 = `cmd-output/tag7_scan.py`、`cmd-output/tag7_impact.py`；错误台账 **E4 / E5**。
- **现象（两次，形态不同）**：
  1. 本块首次**手写** NBT reader ⇒ 全域 chunk 只解析出 **760 个**（**且不抛任何异常**）——下游「可比块数/差异率」全部失真。
  2. 修正为「与 260910-05 已验证版逐行对齐」后看似正常（common=7703、自比 0 差、正对照非零），但**读码复核**发现两版共有的 `t == 7`（TAG_Byte_Array）长度按 **1 字节（`u1`）**读，而 NBT 规范是 **TAG_Int（4 字节）** ⇒ 实测同目录两读法：旧 **7703 ok / 46 bad**、新（规范）**7749 ok / 0 bad** = **静默丢 46/7749 = 0.6% chunk**。
- **根因（机制）**：手写二进制 reader **无校验**，tag→payload 长度映射写错即**指针失步**（desync），后续被 `try/except: continue` 静默截断/丢弃而非报错。**失步签名 = tag 类型普查里出现非 NBT 合法 tag 号**（15/16/18/32/64/…/255）。
- **定位（四步自检，MUST）**：① **自比**（同臂 × 同臂）必须 `diff=0` **且 common 非空**；② **灵敏度正对照**（已知不同两臂）必须报非零；③ **解析 chunk 数 ≈ 生成器自报数 / region 槽位数**（本案首版即在此穿帮）；④ **规范核对**（关键 tag 的载荷长度/结构逐条过规范；案例证明第 ④ 步不可省——第 ①-③ 步全绿仍有两版共有的规范错）。
- **修复**：坐标按 region+槽位推导；reader 与已验证版对齐**并**按规范修正 `tag7`；重跑全部对拍（权威 = 判决 §4.2 的规范读法表）。
- **教训 / 判据**：① **无 `xPos` 载体的坐标唯一来源 = region 文件名 + 槽位索引**（1.20.1；#20 已有同判据）——照抄「读 `xPos` 取键」会丢掉**所有** chunk ⇒ common=0 的**全量假阴性**；② **手写 reader MUST 与已验证版本逐行对齐**（reader 是对拍链公共地基）；③ **但「与已验证版对齐」不等于「解析正确」**——被继承的工具缺陷会随「对齐」动作一起继承；工具复核 MUST 加**规范/冗余量核对**；④ 对拍工具首用 MUST 跑完自检再把数字当结论（**先查工具不查结论**）；⑤ 复用既有工具时**注释里钉死「与哪个版本的哪一段一致」**，使下次改动可回溯；⑥ 工具缺陷修正后**必须重跑受影响结论**（本案跨形态读数由 0.058-0.065% 修正到 0.030% 量级，判据解读随之改变）。
- **家族索引**：#20（1.20.1 chunk NBT 解析两坑——本条为第二/第三形态）、#10/#11（参照/解析产物核对以内容实测为准）、#27（静默退化家族）、workflow-patterns #12/#13（工具 bug 伪装成结论 / 空集先疑工具）、#115（判据面能上移就上移）。

> **E5 补充（260910-06 复审 C1 追记）**：本条（#53）的「丢 46/7749 = 0.6% chunk」只是**可见的一小半**——同一对照下
> **section 并集 85,047 → 185,976（×2.187）**、**块分母 348,352,512 → 761,757,696（×2.187）**、**sections/chunk 11.04 → 24.00**
> （= 1.20.1 overworld 合法结构值，旧读法只摸到真实空间的 45.7%）；且新旧映射**非单调**（正对照 0.0351%→0.0325% 略降）⇒
> **禁按固定比例折算**，**凡引用旧读法数字的已 confirmed 结论（260910-05/1.21.6）必须复算**。
> ⇒ 判据再升一级：**工具首用除四步自检外，加「结构值断言」**（本案 `sections/chunk` 应 = 24/16；旧读法 11.04 会被立刻判死）——
> 该断言已落地在 `diff_1201.py` 的 `[SELFCHECK]` 输出（见 `.investigations/perf-reg-260910-06/cmd-output/selfcheck-demo-260910-06.txt`）。

> **E5 复算完成（260911-05，1.21.6 载体）**：1.20.1 侧的 **×2.187** 已在 1.21.6 侧同口径复算完成——**分母倍 overworld 2.095**（`375,5xx,xxx → 786,825,216`）、**nether·end 2.362**（`209,9xx,xxx → 496,041,984`；end `494,010,368 → 496,041,984` = 1.004），而 **% 倍 1.20-2.67 离散非单调**（历史代理基线 ×1.20 vs nether 同形态噪声 ×2.67）⇒ **禁按固定比例折算旧读数**；且**跨载体倍数不得互引**（1.20.1 ×2.187 vs 1.21.6 ×2.095/×2.362，§9.7 载体要素）。
> **结构核对法（判据升到结构级，可直接复用）**：`sections/chunk` MUST 精确等于**该维度高度 / 16**——overworld `384/16 = 24.0000`、nether·end `256/16 = 16.0000`；旧读法给出 **11.52 / 6.80 / 15.94**（**结构级失真**，不是「略偏低」）。⇒ 工具首用的「结构值断言」应写成**对维度高度的等式**（而非硬编码单一常数），使跨维度/跨版本载体自带校验（本案 judge 独立复算 17/17 精确成立）。
> **旧读法失真的「假一致」面（judge 另证）**：旧读法下 `diff-result.txt` 的**定点段**非空气样本仅 **10/55**，规范读法同载体 = **50/55** ⇒ 旧口径「逐点一致」中约 **40/55 是假一致**（`V=air C=air` 实为**水**，两侧同为误读）。⚠️ **出处 = 本块 judge 意见，尚未落盘 `.artifacts/`**（主会话应用前 SHOULD 补一手锚点；本草稿未独立复核该 55 点计数）。**判据**：定点段/小样本段的「一致」在解析失步下会**成对误读成同一值**（假一致），不能当保真证据——这是 #53 判据③「解析数 ≈ 生成器自报」之外必须补的**假阳性面**。
> **✅ 一手锚点已补（260911-05 主会话独立复核，原 provenance 警告就此结清）**：该 55 点计数**复算成立**——旧读法件 `.investigations/perf-regression-260910-04/cmd-output/diff-result-historical-260910-03.txt` 定点段 55 行、**非空气 10/55**（45/55 双侧读成 `air`，`V≠C` = 0/55）；规范读法同载体件 `.investigations/e5-recompute-260911-05/recompute/diff-hist-baseline.txt` 55 行、**非空气 50/55**（5/55 双侧 `air`，`V≠C` = 0/55）⇒ **约 40/55 的「一致」是假一致**：旧读法把 y=40-44 / 46-50 / 61-62 的**水**列在两侧**同读成 `air`**（失步后成对误读，故 `V≠C` 仍为 0，门不会报警）。复算脚本 `.tmp/c-260911-05/fixedpoint_count.py`（读法：`y=<n>: V=<b> C=<b>` 逐行取非 `air` 计数）。
> **四步自检升级（保真三件）**：#53 `:988` 的「自比 0 差」是**恒真对照**——同目录两次 `load_world` 得同一 dict，**只证解析确定性/幂等，不证保真**（新旧两版工具都能过自比 0 差）。保真证据三件齐才可把数字当结论：① **结构断言**（`sections/chunk` = 维度高度/16 等式）② **规范依据**（关键 tag 长度/结构逐条过规范）③ **旧读法失真对照**（同载体跑旧读法，报失真方向与量级：本案 11.52→24.00）；**可选第 ④ 件 = 独立第三方 NBT 库抽查**（本块**未做**，登记 open）。一手：`.investigations/e5-recompute-260911-05/recompute/diff-*.txt`（15 对 + 2 自比 + 1 历史基线复算）+ `record-260911-05.md` §2/§3。


### 发现 #54（最高价值·错误优先）: 目录级「整树 untrack / ignore」会连带吃掉**住在里面的源码资产**——环境目录规则的作用域 MUST 按「里面到底住了什么」核一遍（15 天内同族两犯，260910-07）

- **发现时间 / 置信度 / module**：260910-07；candidate（机制与链条**git 一手复核**：`0fc44d3` `--stat` 实测删除 **94 个已跟踪文件**；`ae86155` 提交信息自证前次同族犯；`d5151a1` 恢复 2876 文件）；build-tooling / VCS·gitignore 规则作用域（**#24 家族第二形态，方向相反**：#24 是「被目录 prune 挡在外面、`!` 白名单进不来」，本条是「**已在库的资产被整树规则/untrack 一起移出去**，全程无告警」）。
- **来源定位（一条完整历史链，均可 `git log --all -- <path>` 复算）**：
  - `03732db`（2026-08-05）工程首次入库于 `versions/1.20.1/java/`；
  - 2026-08-24 前后某次迁移把 `versions/1.20.1/java/` **gitignore**（承诺「留在 MC 只读」）⇒ 探针工程**「找不回来」**；`.gitignore:53-55` 至今留档此注释；
  - `ae86155`（2026-08-26 22:18）`feat(probe): migrate Java probe project back into CoreSwap (was wrongly gitignored at MC)`——提交信息一手原文：*“Root cause of 'probe not findable': prior migration gitignored versions/1.20.1/java/ ('leave probe at MC read-only') -- now re-enable (remove the rule) + commit source”*（当次 +90 文件）；
  - `b2b9bea`（2026-08-29 19:51）`git mv versions/1.20.1/java → runtime/1.20.1/java`（此时**仍被跟踪**，`.gitignore` 只忽略其 `build/`、`run/`、`data/`、`.gradle/`）；
  - **`0fc44d3`（2026-08-30 20:29）`chore: untrack runtime/ (local-only mod runtime harness)`**——`.gitignore` 追加 `/runtime/`（+3 行），`--stat` **删除 94 个已跟踪文件**（`build.gradle` / `settings.gradle` / `gradle.properties` / `src/main/java/wg/**` 源码 / `content-test` / 3 份 vanilla 源码散件）；提交信息原文：*“runtime/1.20.1/java (Fabric mod project + probes + run dir) is a local working harness, **not distributed source** — keep it out of the repo”*；
  - `d5151a1`（2026-09-10 22:56）`chore(repo): move Java mod projects out of runtime/ to versions/<ver>/java (source now versioned)`（**2876 files changed**）= 本块修复。
- **现象**：`0fc44d3` 的动机本身**合法**（runtime = 本地运行环境，不入库），但 `runtime/<ver>/java/` 里**同时住着出货 mod 的源码本体**——49 个 `src/main/java` 文件 + 构建定义，已发布的 1.0.17→1.0.28 **全部由此构建**（判决 §F2：1.0.28 jar 内 54 class = 这些源文件编译产物）。整树 untrack 把源码一起移出版本管理：**2026-08-30 20:29 → 2026-09-10 22:56 ≈ 11 天**（⚠️ **不是任务书写的「5 周」**，见 §E.3-1）Java 侧改动无 git 史；且**同族事故 15 天内两犯**（08-24 那次的后果是「工程找不回来」，代价更直观）。
- **根因（机制）**：「运行环境 / 缓存 / 本地私有」是**用途标签**，不是**目录内容清单**；而 gitignore 与 untrack 的作用域是**目录树**。写 `/runtime/` 或 `git rm -r --cached runtime/` 时，git 的判据是「这个目录叫什么/被声明成什么用途」，**从不检查「这棵树下已跟踪的是什么」**——于是 94 个资产被一次性移出，**零告警、退出码 0**（git 不会提示「你移出了一个 49 文件的源码工程」）。机制上这甚至比「删除」更隐蔽：**历史还在**（旧 commit 可查），但「当前状态」失去 VCS ⇒ 无 pre 快照、无 diff、无回滚点（260910-06 更因此只能用 1.0.27 jar 反推 class 级 pre）。⇒ **「git 历史还在」≠「当前受版本管理」**，后者才是迁移/发布的 pre/post 边界所依赖的东西。
- **定位（怎么发现的，可复用）**：① `git show --stat 0fc44d3`——一眼可见被删的 94 个文件里有源码（这是最便宜的第一刀）；② `git log --all --format='%h %ad %s' -- versions/1.20.1/java`——拿完整链条（只查当前分支会漏）；③ **判据核（MUST）= 双向**：`git ls-files <dir>`（这棵树下**实际被跟踪**什么，数量+清单）对照 `git check-ignore -v <path>`（规则命中链），再加 `git status --porcelain`（无新增 `??` 噪声）。
- **修复（两段，缺一不可）**：
  1. **结构性：工程/环境物理分离**——源码 + 构建定义迁 `versions/<ver>/java/`（与既有 `versions/<ver>/{rust,data,cpp}` 同构，也是历史原位），运行环境**原地** `runtime/<ver>/java/run/`（世界/mods/`server.properties` 零搬迁，loom `runDir "../../../runtime/<ver>/java/run"` 指回）。此时 `/runtime/` 规则**保留**——它终于名副其实（纯环境）。**注意：本案不是靠「规则精修」解决，而是靠把两种资产分开**（详见教训 ③）。
  2. **规则面**：`.gitignore` 只新增**派生/缓存**忽略（`versions/*/java/build/`、`versions/*/java/.gradle/`、`versions/*/java/run/`、`versions/*/java/*/run/`、`versions/*/java/src/main/resources/native/`），**源码 / 构建定义 / worldgen-data 零忽略**；并补**嵌套 prune 白名单链**：全局 `data/` + `**/data/*`（`.gitignore:19-20`）会 **prune** `src/main/resources/worldgen-data/data/**`，本块靠 `:109-110` 的两行 `!`（`!/versions/*/java/src/main/resources/worldgen-data/data/` + `.../data/**`）放行——**每版本 1015 个 JSON**（一手复核：1.20.1 `worldgen-data` 共 1019 个在库文件，其中 `data/**` 1015 个）。这正是 **#24 的三段链**（目录放行 → 内容重排除 → 文件白名单）在本块的第二次落地。
- **教训 / 判据（MUST，可复用）**：
  1. **任何目录级 ignore / 整树 untrack 落盘前，MUST 按「里面到底住了什么」清点一遍**：`git ls-files <dir>`（已跟踪资产清单）+ `git check-ignore -v`（规则命中链）**双向**。只核「新文件进不进得来」不够——**还得核「已在库的东西会不会被一起移出去」**。
  2. **「用途标签」不足以授权整树规则**：在写 `/runtime/`、`/cache/`、`/local/` 之前，先枚举该树下**已跟踪**资产（本案 94 个），确认无一有源价值物；有 ⇒ 改物理分离，不改规则。
  3. **承载体尽量不混装**（本条最上位判据）：环境与工程同目录时，规则**无法只作用于环境**——「工程/环境分离」是低成本一次投资，长期免于每次精修规则。
  4. **迁移/untrack 后核链条用 `git log --all -- <path>`**，不要只查当前分支；核「是否真离库」用 `git ls-files <path> | Measure-Object`（计数 0 = 已离库）。
  5. **「git 历史还在」不作兜底**：它不提供 pre/diff/回滚点；凡需要 pre 快照的工作流（等价性门、发布对照），MUST 确认**当前**受版本管理——否则 pre 只能靠旧产物反推（本案 260910-06 的困境）。
- **家族索引**：#24（目录级 prune 使 `!` 白名单失效——本条为其**方向相反**的第二形态：把已跟踪资产**移出去**；两者共用「目录级规则的语义边界」这一根因）、workflow-patterns #77（回滚 ≠ 引用面清理——同属「结构动作的完整性检查」家族）、#50（写入型工具的静默销毁：退出码 0 + 无告警——本条是 **git 侧的静默移出**）、#18/#27（「在盘/在历史」≠「当前有效」）。
- **证据**：`0fc44d3`（`git show --stat`：94 D + `.gitignore` +3）、`ae86155`（提交信息 + 90 A）、`b2b9bea`/`d5151a1`；`.gitignore:53-55/:95/:100-104/:106-110`；一手复核见 §E.4。

### 发现 #55 简记: loom `runDir` 是 **String 且按「工程目录」相对解析**——绝对路径被拼成 `工程目录\E:\…` ⇒ `CreateProcess error=267`（260910-07）

- **发现时间 / 置信度 / module**：260910-07；**确定**（一手错误原文 + 产物的 `javap` 读真实签名 + 改相对路径后跑通）；build-tooling / loom 配置解析基准（**#8/#47「接线/映射错觉」家族的「解析基准」维**）。
- **来源定位**：错误台账 `.investigations/perf-reg-260910-07/migration-errors-260910-07.md` **E1**；修复落点 = `versions/1.20.1/java/build.gradle:199`（server）与 `:205`（client）各一处 `runDir "../../../runtime/1.20.1/java/run"` + 就地注释（一手实读）。
- **现象**：迁移后 `gradle :runServer` 启动即失败——`Execution failed for task ':runServer' > A problem occurred starting process 'command 'D:\Program Files\Java\jdk-17.0.12\bin\java.exe''`，`Caused by: java.io.IOException: Cannot run program "…java.exe" (in directory "E:\PYTHON\CoreSwap\versions\1.20.1\java\E:\PY…")` → `CreateProcess error=267, 目录名称无效。`
- **根因（机制）**：`loom { runs { server { runDir "E:/PYTHON/CoreSwap/runtime/1.20.1/java/run" } } }`——`RunConfigSettings.runDir` 是 **String**（`javap -p` 实证 `private java.lang.String runDir;` / `public void setRunDir(String)`，fabric-loom **1.10.5**），消费侧**按工程目录相对解析** ⇒ 绝对路径被拼成 `工程目录\E:\PYTHON\…`（无效目录）。**「绝对路径一定安全」是错觉**——它的安全性取决于消费侧的解析基准。
- **定位（两条都便宜，可复用）**：① **读错误里的 `(in directory …)`**——它把**解析后**的工作目录直接打出来，拼接关系一眼可见；② **不猜 API**：从 gradle 缓存 `jar xf fabric-loom-1.10.5.jar` + `javap -p …RunConfigSettings` 读真实签名（比试错快，且不污染源码）。
- **修复**：改相对路径 `runDir "../../../runtime/<ver>/java/run"`（工程目录 = `versions/<ver>/java` ⇒ 上三级回仓库根）。
- **教训 / 判据**：① 迁移工程时凡「指向工程外」的路径配置**都要核解析基准**——loom `runDir` 按**工程目录**、gradle `-P` 按**闭包作用域**（#47）、gitignore 按**仓库根/锚定**（#24）；② 判据 = **读错误信息里「解析后」的路径，别读你自己写的那份**；③ 配置项的类型/签名以**产物 `javap` 一手**为准，不靠文档记忆或上一版本经验。
- **家族索引**：#8/#9/#19/#25/#47（接线/映射错觉家族——本条补**解析基准**维）、#24（gitignore 锚定语义——同属「模式的作用域由消费方定义」）、workflow-patterns #81/#37（生效证据必须行为化——本条是「不生效即响亮失败」的良性形态）。

### 发现 #56 简记: 沙箱内 gradle 文件监视失效 ⇒ `UP-TO-DATE` **假绿**——「构建绿」不等于「改动被编译进去」（260910-07）

- **发现时间 / 置信度 / module**：260910-07；**确定**（一手：日志 file-watcher 异常 + 源码刚改仍报 UP-TO-DATE + `--rerun-tasks` 对照）；build-tooling / gradle 增量构建（**#1/#23/#25/#40「UP-TO-DATE / 构建绿 ≠ 生效」家族**）。**补记说明**：本事实 260910-06 已**实际依赖过**（该块时间线 `:3102` 记「强制重编 `--rerun-tasks`，规避沙箱内 gradle VFS/文件监视失效的 `UP-TO-DATE` 假绿」）但**未落 discovered**；本条为**判据化补记**（本块把它当成等价性门的一环使用）。
- **来源定位**：错误台账 **E3**；本块应用 = `.investigations/perf-reg-260910-07/`（注释级改动后 `:build` 报 UP-TO-DATE → `--rerun-tasks` 强制重编 → 得到「注释级改动后 jar 逐字节相同」的结论，即 #116 等价性门的一环）。
- **现象**：改了 Java 注释后 `gradle :build` 输出 `> Task :build UP-TO-DATE`，jar 未变；日志有 `Exception in thread "File watcher server" net.rubygrapefruit.platform.NativeException: Couldn't open current thread, error = 5`。
- **根因（机制）**：沙箱限制导致 gradle 的**文件监视通道不可用** ⇒ 增量构建的 **VFS 未察觉磁盘改动** ⇒ 直接按旧状态判 UP-TO-DATE。**与 #1 是两个机制面**：#1 = `doFirst` 里的 copy **不是 task input**（声明缺失）；本条 = **监视通道本身失效**（VFS 陈旧）——两者都表现为「构建绿 + 产物没变」，但修复动作不同（#1 改声明/手动 copy，本条只能强制重跑）。
- **定位 / 判据**：**`UP-TO-DATE` 与「文件确实改了」冲突时，先看有没有 file-watcher 异常**（本案例日志里有），再用 `--rerun-tasks` 做对照——对照后产物变 = 确认假绿。
- **修复 / 纪律**：**验证性构建一律 `--rerun-tasks`**；凡结论涉及「产物是否变了」（等价性门、发布前核验、dll 同步），**不能只信 gradle 的 up-to-date 判定**，必须强制重编或内容指纹（#6/#10/#16/#18/#23）。
- **家族索引**：#1（task input 声明缺失——同一「UP-TO-DATE 假绿」家族的另一机制面）、#23（cargo `-p` 陈旧 rlib 假绿）、#25/#40（编译绿 ≠ apply 绿）、#16/#18（产物在盘 ≠ 本次生成）、workflow-patterns #116（本块等价性门依赖 `--rerun-tasks` 得到的「注释级无效改动」结论）。

---

---

### 发现 #57 简记: 池宽语义「物理核 − 2」≈ 实现 `logical/2 − 2` 在 SMT2 下同源等价——用户语义与实现公式换算要声明条件域（260911-04）

- **时间/置信度/module**：260911-04；candidate（用户拍板 + judge 池宽语义核对一致）；build-tooling / 运行环境配置语义。
- **观察**：池宽决策以用户语义「物理核 − 2」拍板，实现缺省 = `logical/2 − 2`——SMT2（Intel HT / 常规双线程核）下 `logical/2` = 物理核数，二者同源等价（引擎 `adaptive_threads` 同口径），故「维持用户设定」= 零代码改动。
- **判据（简记）**：① 池宽/并发度类配置，用户口中的「物理核」与实现公式的「logical」不得直接混写——引用等价性 MUST 带条件域（SMT2）；② 非 SMT2 或异构大小核机器上等价不成立，`logical/2 − 2` 会偏离「物理核 − 2」语义，跨机器部署前先核拓扑；③ 交接/工单写池宽时两种表述并存注明换算关系，防下游按字面误配。

---

### 发现 #58: 手写 NBT reader 的长度字段必须按「tag 类型 → 载荷长度类型」对表核规范——单点笔误造成**结构级失真**而非小幅偏差（260911-05）

- **发现时间 / 置信度 / module**：260911-05；**candidate**（同载体单变量复算：唯一改动 = `tag 7` 长度字段 `u1→i4`；结构核对 17/17 精确成立；旧读法失真对照在盘）；build-tooling / MCA·NBT 工具链（**#53 的规范表形态**，与 #53 的「坐标推导」形态并列）。
- **来源定位**：`.investigations/e5-recompute-260911-05/diff_arms_fixed.py`（相对 260910-05 版 `diff_arms.py` **仅一处**改动）+ `record-260911-05.md` §1/§2/§3。
- **观察**：`payload()` 的 `tag 7`（TAG_Byte_Array）长度字段按 **1 字节**读，规范 = **TAG_Int（4 字节）**——与 `tag 11/12`（TAG_Int_Array / TAG_Long_Array，本就按 `i4` 读）**同族**，属**单点笔误**（不是版本差异）。后果不是「数值略偏低」而是**结构级失真**：`sections/chunk` 旧 11.52 → 规范 **24.0000**（overworld，= 384/16）、旧 6.80 → **16.0000**（nether·end，= 256/16）；分母 `375,5xx,xxx → 786,825,216`（overworld）、`209,9xx,xxx → 496,041,984`（nether·end）。
- **根因（机制）**：手写二进制 reader **无校验**，`tag → 载荷长度类型` 映射写错即**指针失步**（desync），后续被 `try/except: continue` **静默**丢弃（不报错）。失步签名 = tag 类型普查出现**非 NBT 合法 tag 号**（15/16/18/32/64/…/255）。
- **判据（MUST）**：① 手写 NBT reader 的 `payload()` **MUST 按类型对表**逐条核规范——`tag1/2/3/4/5/6` 定长、`tag7/11 = TAG_Int(4B)`、`tag8 = TAG_String(u2 长度)`、`tag9/10 = 列表/复合`、`tag12 = TAG_Long(8B)`——**不得用「与已验证版逐行对齐」代替**（共有缺陷会随对齐一起继承，#53 判据③已证）；② **单点长度笔误的判别签名 = 结构级失真**（`sections/chunk` 偏离维度高度/16、分母倍远离 1），**不是**小幅百分比偏差——见到「分母/结构值系统性偏低」先查长度字段，别在精度上纠结。
- **家族索引**：#53（region 对拍工具——本条为其**规范表**升级）、#20（NBT 解析坑——本条与「指针不推进」并列）、workflow-patterns #12/#13（工具 bug 伪装成结论）、#115（判据面能上移就上移）。

---

### 发现 #59（最高价值·错误优先）: A/B 驱动的收尾 `restore` + `target/` 里残留的实验暂存 dll ⇒ 跨臂对照可能**执行了不同引擎**——执行体血统 MUST 逐臂读「执行体自证行」核对，不得只看 target 文件 sha（首例 260911-05 / 第二实例 260912-01）

- **发现时间 / 发现者 / 置信度 / module**：① 首例（**预防性、未发生**）= 260911-05，judge I3 指出（`review-260911-05.md`；当时收尾实测 dll 值正确、无遗留）；② 第二实例（**实际发生、判定作废一次**）= 260912-01（共享 Java 适配核 Wave 2 的 V2/V3 对照）；**首例 = candidate**（A 线 judge 已审）；**第二实例部分 = confirmed**（用户授予 2026-09-12 15:52；本波 judge 已做 = PASS-with-conditions、C1–C9 已响应（record §4.7.7、`review-wave2-260912-01.md` §5.1 复算自证行一致），record §6 已回填）；build-tooling / 构建链一致性与执行体血统（**#23 家族第三/第四形态**：驱动脚本自带回滚动作 → 污染对象从「交付产物」升级为「**对照实验本身**」）。
- **来源定位**：首例 = `record-260911-05.md` §1/§5（`:11-12`/`:89`）+ judge `review-260911-05.md` I2（`:55`，门禁强度）/ I3（`:56`）；第二实例 = `.investigations/shared-java-core-260912-01/record-260912-01.md` §4.7.0（配方 `:213-217`）/ **§4.7.1（事故根因链与处置 `:218-225`）**/ §4.7.2（三臂自证行 `:231`）/ §4.7.4（dll 归一化隔离 `:268-271`）+ §2.6（产物目录不是存档目录 `:83-87`）；五段式台账 = `errors-260912-01.md` **E1**；判据可从仓库复现（`evidence/MANIFEST.txt`、`evidence/arm-summary.txt`、`evidence/fp-*.txt` 的逐臂自证行）。
- **现象①（首例，交付侧）**：a1 A/B 驱动的 `results.txt` 记 `restore=dd3b645f`（= **A1d 前**构建，size 2457088）⇒ 驱动结束会把 `target/release/worldgen.dll` **还原为其 bak**（= pre 构建）。当时收尾实测当前值 = `838e8979`（正确、无遗留），但**交付/发版前仍值得复核构建链**。另：dll 硬门禁（`run_ab.ps1:71-78`）只比较日志里的 **16 hex 前缀**（`sha256=838e8979…`），**非全 sha256**。
- **现象②（第二实例，对照实验侧）**：260912-01 Wave 2 的 V2/V3 首轮，两臂**实际执行的 dll 不同**——post 臂 `838e8979…` vs pre 臂 `dd3b645f…` ⇒ **对照被引擎差异污染、V3 判定作废**（判 VOID、两臂重跑）。**发现方式 = 逐臂读取 `<CppBridge> dll= sha256=`「执行体自证行」**，非事后猜测。
- **根因（机制，第二实例的完整链）**：① `target/release/worldgen.dll` 当时是**早前 A/B 实验留在 target 的非权威产物**（`838e8979…`，A1d 后构建）；② A/B 驱动 `run_ab.ps1` 收尾**无条件** `Copy-Item $bak $targetDll`，把 `.tmp/.../worldgen-target.bak-597e12ed`（= **1.0.28 引擎 `597e12ed`**）写回 target；③ 在 `git worktree` 内 `gradle :build` 触发 dll 同步链，target 又被恢复为**权威 `dd3b645f`**（= 1.0.29 票记录值）⇒ **两臂各读一个 dll**。机制本质：`target/` **既是构建产物目录、又被实验脚本当暂存/回滚区**，而「执行体身份」没有任何硬门禁约束，只有一份可被覆盖的可变文件。
- **定位（可复用诊断方法）**：① **逐臂读执行体自证行**（`[CppBridge] dll= sha256=`）→ 立刻暴露两臂不同源；② **三元组核验**（事后）= 1.0.29 票 dll `dd3b645f…` ≡ 当前 `target/release/worldgen.dll` ≡ 重编 jar 内 `native/worldgen.dll`；1.21.6 = `abd7d889…` 三处一致；③ **归一化隔离验证**：`post3`（权威 dll）vs `post2`（旧 dll）= 1.20.1 仅 1 条目差异（`native/worldgen.dll`）、1.21.6 **0 条目差异** ⇒ 证明 dll 归一化对 class 条目零影响（避免「Java 面结论被 dll 污染」的二次误判）。
- **修复（处置）**：权威 dll 另存 `.tmp/.../w2/dll-canonical-1.20.1.dll`；**覆盖 harness 备份**（原备份另存 `…bak-597e12ed.historical-597e12ed`）使收尾 restore **退化为 no-op**；两臂重跑（重跑后 V2+V3 双 PASS、两层指纹各 607/607 全等）。**证据** = `evidence/log-1.20.1-{post,pre-r1,pre-r2}.txt` 的逐臂自证行 + `evidence/arm-summary.txt`；judge 独立复算一致（`review-wave2-260912-01.md` §5.1）。
- **判据（MUST）**：
  1. **跨臂 / 跨 run 对照前 MUST 逐臂读「执行体自证行」核对**；**不得只看 `target/` 的文件 sha**——target 会被「A/B 收尾 restore」与「实验暂存」改写。自证行 mismatch ⇒ 该臂判 **VOID**（不许人工挑臂，与 workflow-patterns #118 同族）。
  2. 凡驱动/编排脚本带 `restore=` / 回滚 / 备份还原动作，收尾**与发版前** MUST 重核 target dll 的**实际 sha**；restore 目标 MUST 钉为权威产物或**退化为 no-op**。
  3. **门禁强度粒度 MUST 声明**（16 hex 前缀 vs 全 sha256）——前置声明可防下游把「前缀命中」读成「全量一致」。
  4. **构建链对 dll 有自愈但不许依赖**：`gradle :build` 的 dll 同步 `doFirst` 会把 target 拉回权威产物（本次把非权威 `838e8979` 纠正为 `dd3b645f`），但若权威 dll 不存在或同步被 `UP-TO-DATE` 跳过则**不自愈**（#56 / #96 家族）。
  5. **「构建产物目录不是存档目录」**：`gradle :build` 会按 `version`/`archivesName` **就地重写** `build/libs/*.jar`（260912-01 覆盖了已发布 1.0.29 的出货 jar）⇒ 出货工单的权威完整性判据 = **工单内 sha256**（+ 发布侧 status 镜像），不是本地 `build/libs` 文件；冻结基线前 MUST 先 `Copy-Item` 另存。
- **家族索引**：#23（`cargo -p` 依赖 rlib 陈旧假绿——同属「绿/Finished ≠ 产物已更新」）、#16（探针用前核产物时间戳）、#18/#27（产物在盘 ≠ 本次执行体生成 / 缓存新鲜度）、#30（rlib 根产物陈旧）、#56（UP-TO-DATE 假绿——自愈不可依赖的机制面）、#96（dev-run 形态的执行体三元组核验）、workflow-patterns #118（自证行必须做成硬门禁）、#66/#36（执行体三元组与执行语义）、#105（载体偏差）。

---

### 发现 #60 简记: fresh `git worktree` 不能当「条目级」基线——检出文本被 `core.autocrlf` 物化为 CRLF，与主工作树的 LF 不同 ⇒ 逐条目比对出现全量伪差异（260912-01）

- **发现时间 / 置信度 / module**：260912-01；**confirmed**（用户授予 2026-09-12 15:52；judge 已做 = PASS-with-conditions、C1–C9 已响应；record §6 已回填；一手锚 = record §4.7.6）；build-tooling / 等价性基线与换行策略（**#116 条目级 sha 门的前提面**）。
- **来源定位**：`.investigations/shared-java-core-260912-01/record-260912-01.md` **§4.7.6**（一手锚 / 补记）+ §4.7.0（`git worktree` 臂设置 `:213-217`）；证据 = `evidence/manifest-prewt-1.20.1.tsv`、`evidence/manifest-post3-1.20.1.tsv`、`evidence/prewt-pseudodiff.txt`；台账 = `errors-260912-01.md` **E2**。
- **现象**：`git worktree add` 检出的**文本资源**被 `core.autocrlf` 物化为 **CRLF**，与主工作树的 **LF** 不同 ⇒ 用 worktree 构建出的 jar 与主树 jar 逐条目比对时，**所有 JSON/文本条目全部显示差异**（实测：prewt vs post3 = **相同 57 / 差异 1024 / 新增 1 / 删除 0**，差异**全落在 `worldgen-data/**` 文本资源**、**class 条目零差异**）。
- **根因（机制）**：worktree 检出走同一 `.gitattributes` / `core.autocrlf` 机制，**工作区文本换行**随策略变化；而 class 条目由 javac 产物决定、**与工作区换行无关** ⇒ 伪差异**只在文本资源路径前缀上成片出现**，class 差集为空。若基线取在别的树，则「差异集」里混入**换行噪声**，真差异被淹没。
- **定位 / 判据**：① 看差异集的**分布形态**——差异全部集中在文本资源（`data/**`、`*.json`）且 class 条目差集为空 ⇒ 先疑换行/编码策略，而非代码；② **条目级 sha 基线 MUST 用「同一工作树」构建的产物**（本案 V1b 基线最终取主树 `post2`/`post3`，`record §2.1`）；③ **跨树比对只可用于类文件 / 运行期判据**（class 字节与 CRLF 无关；本轮的 `git worktree @ ce5286b` 正是只作 **pre 臂** 的运行期对照）；④ 若必须用 worktree 作条目级基线，开工前 MUST 核 `core.autocrlf` 与 `.gitattributes` 并显式声明或统一化；⑤ **流程实况（`errors` E2）**：worktree 产物**曾**被拿作条目级基线 → 全量伪差异 → **放弃**；条目级基线改用**同一主工作树**的 `post2/post3`（V1b 判定基线 = `post2`），worktree 只保留**运行期 pre 臂**用途。
- **家族索引**：#116（条目级 sha 等价门——本条为其**基线来源前提**）、#53（region 对拍工具的结构断言）、#6/#10（内容指纹判新旧——同一「别信表象、要信内容」族）、workflow-patterns #138（等价门分档：纯移动 vs 语义统一）。

---

### 发现 #61 简记: javap 方法级对拍的三个陷阱——lambda 名按序号命名不可按名对拍 / 按行 zip 对拍在指令数变化处级联误报 / 常量池序号与 `ldc`↔`ldc_w` 宽度必须归一化（260912-01）

- **发现时间 / 置信度 / module**：260912-01；**confirmed**（用户授予 2026-09-12 15:52；judge 已做 = PASS-with-conditions、C1–C9 已响应；record §6 已回填；一手锚 = record §4.7.6）；build-tooling / 字节码对拍方法（**f5-bugs「javap 不可信点」的姊妹条**）。
- **来源定位**：`record-260912-01.md` **§4.7.6**（一手锚：初版伪差异计数 + 源码逐字对照 + judge 三重判定）+ §4.6（`:171-193` 各条 `javap -c -p`「输出完全相同」判定、`:186-192` 方法级对拍与 `lambda$static$0` 诚实声明）；台账 = `errors-260912-01.md` **E3**；证据 = `evidence/src-pre-1.20.1-CppBridge.java`、`evidence/tool-javap_method_diff.py`。
- **现象（三陷阱）**：① **`lambda$static$N` 按序号命名** ⇒ pre/post 的同名 lambda **可能不是同一个物**；本轮 `lambda$static$0` 因此被明确判为「**不构成证据**」（改名比对无意义）。
  ② **按行 zip 对拍**：两版 javap 输出按行号 zip 比对时，**任一处指令数变化**会把其后所有行的对齐整体错位 ⇒ 「differing lines」**级联放大**、把一处真差异报成成百上千行（record §4.7.6 已补一手锚：初版 `stateById` 6 行 / `lambda$static$0` 46 行均为伪差异）。
  ③ **常量池序号 / `ldc`↔`ldc_w` 宽度 / 字节偏移**在两侧天然不同 ⇒ 不归一化时**序号伪差淹没真差异**；归一化后 `stateById` 仍报 6 行残差，用**源码逐字对照**（pre `:629-638` ≡ post `:725-734`）确认为**序号伪差**。
- **根因（机制）**：javap 输出是**编译期产物的文本投影**，其中含三类**非语义自由度**——名称分配（lambda 序号）、常量池布局（序号、`ldc` 宽度）、偏移（指令地址）；对拍工具若把这些当成内容，就测的是「编译细节」不是「语义」。
- **判据（可复用）**：① **按方法名集合先对齐**（`same/changed/added` 三分类），再在单方法内比对——**不要按行 zip 全体输出**；② 单方法内比对 MUST 先**归一化偏移 / 常量池序号 / `ldc` 宽度**，归一后残差 MUST 用**源码逐字对照**定性（是序号伪差还是真差异）；③ **同名 lambda 不可按名对拍**——要定位身份须查源码或调用点；④ 报告 MUST 区分「指令级差异」与「调试属性差异」：`javap -c -p`（不含 `LineNumberTable`）输出**完全相同** ⇒ 仅调试属性（本案 4 条即据此定性）；⑤ 报告差异行数时 MUST 声明是否归一化、是否为 zip 对拍——否则「697 行差异」类数字不可比；⑥ **关键结论 MUST 走多重交叉判定**（`javap -c -p -constants` 文本逐字 + `javap -v -p` 属性分类 + **回源码逐字对照**）——比任何单一计数都可靠（judge 复算即此法，确认 4/4 条「仅调试属性」）。
- **家族索引**：f5-bugs（javap/反编译不可信点——本条为**对拍方法**形态）、workflow-patterns #138（V1b「仅调试属性」声明粒度）、#137（引用外部锚须自核）、compiler-idioms #25（`LineNumberTable` 使注释也改字节）、build-tooling #58（手写 reader 单点笔误造成结构级失真——同属「工具读法决定结论」）。

---

### 发现 #62: 裸任务名级联的**范围修正** + 判别手段升级——`runServer` 双命中是 **1.20.1 独有**（1.21.6 不触发），且判别 MUST 用 `--dry-run` 任务图而非目录存在性（260913-01；#51 的范围修正）

- **发现时间 / 发现者 / 置信度 / module**：260913-01；主会话（两版任务图对照直证）+ judge subagent（独立实跑两版 `--dry-run` 复核，并抓出叙述层事实错误）；**candidate**（源记录 = **confirmed**，用户授予 2026-09-13；范围明确**不含** 1.20.1 侧行为结论）；build-tooling / gradle 任务图（**#51 的范围修正 + 判据落地**）——#51 是前身（发现该级联），本条给出其**适用边界**与**低成本直证法**（#8/#47「接线/映射错觉」家族**第七形态：适用边界**）。
- **来源定位**：`evidence/arm-commands-260913-01.txt` §1（四行两版对照表）；`verify-260913-01.md:76-88`（§4.1 + 判错经验块）；judge 独立复核 = `judge-260913-01.md`（核对表「§4.1 结论」行 + 机制行 ❌ + 发现 A）；原机制首发 = `judge-260912-02.md:299-301`（C-17）；一手源 = `versions/1.21.6/java/settings.gradle:13`（`include 'content-test'`，本次实读）、`versions/1.20.1/java/content-test/`（目录在 + `git ls-files` = 3 文件）。
- **观察（现象）**：260912-02 记录机制「**无冒号** `gradle runServer` 会双命中 `:runServer` + `:content-test:runServer` ⇒ 两次 JVM 运行写同一日志、计数对整份日志做（12 = 4/4 + 4/0）」（= `build-tooling #51`）。本块进入 1.21.6 载体时**默认继承**该机制 ⇒ 会认为 1.21.6 也有「单 run 口径不可信」的载体缺陷。**两版任务图对照实测**：

  | 项目 | `gradle --dry-run runServer` 任务图 | 结论 |
  |---|---|---|
  | 1.20.1 裸 `runServer` | `:runServer` **+** `:content-test:runServer` | ⚠️ 双命中（#51 的真实机制） |
  | 1.20.1 `:runServer` | 仅 `:runServer` | ✅ 单 run |
  | **1.21.6 裸 `runServer`** | **仅 `:runServer`** | ✅ 单 run —— **本版无该缺陷** |
  | 1.21.6 `:runServer` | 仅 `:runServer` | ✅ 单 run |

- **根因（机制）**：该级联**依赖「子工程存在且应用了 fabric-loom」这一前提**——1.21.6 的 `versions/1.21.6/java/content-test/` **目录不存在**（`Test-Path` = False、`git ls-files` = 0），但 `settings.gradle:13` 的 `include 'content-test'` 仍让 Gradle **自动建一个空子工程**（`gradle :content-test:tasks` → `BUILD SUCCESSFUL`），该子工程**没有 loom 插件** ⇒ **`runServer` 任务不存在**（`gradle :content-test:tasks --all | grep runServer` = **零命中**）⇒ 名字匹配只命中根工程一个任务。⇒ **「上一块的根因机制」不是普遍规律，是带前提的机制**；旧机制被当成普遍规律而**未按载体核实前提**。
- **定位（零机时判别手段）**：`gradle --dry-run <task>` 打印**任务图**（不真正执行），两版对照即判；配合 `gradle :<proj>:tasks --all | grep <task>` 定因（子工程存在 ≠ 有该任务）。**注意**：不要用目录存在性 / 目录内容推断——见 workflow-patterns **#140**（`include` 与目录存在性解耦）。
- **修复 / 处置**：无需改代码；**登记该不对称性**（verify §4.1 + 命令表 §1），运行台仍**显式使用带冒号 `:runServer`**（防御性：将来若补 1.21.6 的 `content-test`，写法不退化）。
- **教训 / 判据**：
  1. **机制继承 MUST 同时重核其前提条件在本载体是否成立**（此处前提 = 子工程有 loom 的 `runServer`）——同 #139 ② 的「同一判据的旧诊断必须继承」互补：**继承机制的同时必须重核前提**。
  2. **反向误记比漏记更贵**：把 1.20.1 的载体缺陷记到 1.21.6 头上，会给后续**所有** 1.21.6 计数下「载体不可信」的错误前提（导致无谓的防御性设计或错误的排除结论）。
  3. **任务图直证（`--dry-run`）是零机时的判别手段**——凡「某机制是否**在本载体**成立」类问题，一律优先用它，而非推理或类比。
  4. **门禁强度粒度 MUST 声明**（承 #59 判据 3）：本条只证「任务存在性」；「该任务是否真起第二个 JVM」属**行为**，需要另跑（本条未做，属 260912-02 已 confirmed 范围）。
- **家族索引**：**#51（裸任务名级联——本条为其范围修正）**、#8/#9/#19/#25/#47（接线/映射错觉家族——本条补**适用边界**维）、#59（门禁粒度声明）、workflow-patterns **#140**（判别构建图 MUST 用任务图）、#139 ②（计数载体切分——本条的**直接消费方**）。
---

## 发现 #147: 程序化修改 PowerShell 脚本后 MUST 用 `Language.Parser::ParseFile` 核语法——正则/字符串替换引入杂散反引号 + 末参数插参漏逗号两连犯；`[scriptblock]::Create` 式自查会吞错假 OK（260913-06）

- **发现时间 / 发现者 / 置信度 / module**：260913-06；主会话（脚本副本生成两连翻车 + Parser 实核）；**candidate**（修复后 PARSE-OK + 四臂全跑通，confirmed 留人类）；build-tooling / PowerShell 脚本生成坑（#45/#49/#41 同文件家族的**脚本生成面**）。
- **来源定位**：`.investigations/sentinel-260913-06/record-260913-06.md` §过程错误 2；运行台 `.tmp/sentinel-260913-06/run_*.ps1`（母本副本 + 程序化变更，变更登记见 judge J6——diff 恰为登记项，机制面见本条）。
- **五段式（错误优先）**：
  - **现象**：从母本脚本程序化生成运行台副本（正则替换 out 目录 / 在命令末参数插入新参数）连续两次产生**语法级损坏**：① 正则替换引入**杂散反引号**（行尾续行符残留/误置 ⇒ 后续语句被续行吞并）；② 末参数后插入新参数**漏逗号**。且自查用 `[scriptblock]::Create` 包装脚本文本——**报「成功」但错误仍在**（假 OK）。
  - **根因（机制）**：① 文本层正则/字符串替换对 PowerShell 语法**零感知**——反引号是合法续行/转义字符，替换边界落在反引号附近即产生语法损坏，且损坏点常在替换点之外（静默扩散）；② 末参数插参靠字符串拼接，逗号是纯文本，漏掉不报任何警告；③ `[scriptblock]::Create` 将脚本编译为 ScriptBlock 时**解析错误以异常抛出与否取决于调用方式/PS 版本语义**，作为「自查」它不是语法门——**吞错或错位报错，给人已校验的错觉**。
  - **定位**：运行报 ParserError 后逐行核对生成脚本 diff；改用官方解析器实核后一次定位两处损坏。
  - **修复**：以 `[System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$null, [ref]$errors)` 实核，`$errors` 非空即拒收——修复后 PARSE-OK 才入运行台。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **程序化生成/修改 .ps1 后，MUST 跑 `Language.Parser::ParseFile` 并断言解析错误数为 0**（一行成本），再交付执行；「生成成功 + 文件存在」零证据力。
  2. **`[scriptblock]::Create` 不是语法自查门**——会吞错/假 OK；语法门只用 Parser API（`ParseFile`/`ParseInput`，取 `[ref]` errors）。
  3. **文本层替换改脚本时，替换点之外的语法损坏是常态风险面**（反引号续行/引号配对/逗号）——凡对**要执行的脚本**做程序化变更，Parser 门是唯一可靠防线；judge 类逐行 diff（J6）核对的是「改了什么」，Parser 核的是「改完是否还是合法 PowerShell」，两者互补、都不可省。
- **家族索引**：#45（`-like` 字符类坑）、#49（`-File` 多值参数）、#41（管道早退截断日志）、#34（Move-Item 静默改名）——同文件 PowerShell 坑家族；#62（同构：「编译过不构成接线证据」——本条为「文件生成了不构成语法合法」）。
