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

