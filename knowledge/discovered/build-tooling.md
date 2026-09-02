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

