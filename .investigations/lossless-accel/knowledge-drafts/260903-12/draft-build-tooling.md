# 草稿：knowledge/discovered/build-tooling.md 追加两条（subagent 产出，主会话应用）

> **应用位置**：`knowledge/discovered/build-tooling.md`——「发现 #12」之后（文件末尾）追加「## 发现 #13」「## 发现 #14」。追加不覆盖。写后同步 INDEX.md。
> 现有编号核对：build-tooling.md 当前最大编号 **#12**（另有一条 #5 草稿头与「环境坑补记」，均不占正式编号位），本两条为 **#13 / #14**。
> 来源 session：260903-12（实际 2026-09-03 晚）；错误台账落点：`.investigations/lossless-accel/lossless-accel-errors.md`（主会话补 LL 系列五段式）。

---

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
