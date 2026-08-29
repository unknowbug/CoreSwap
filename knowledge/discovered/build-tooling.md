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
