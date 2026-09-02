# 草稿：knowledge/discovered/build-tooling.md 追加「发现 #7」（subagent 产出，主会话应用）

> **应用位置**：`knowledge/discovered/build-tooling.md`——「## 发现 #6」之后（文件末尾）追加。追加不覆盖。写后同步 INDEX.md。
> **编号核对**：build-tooling.md 当前至发现 #6，本条为 **#7**。
> 来源：错误 E10（`.investigations/nether-save-full/nether-save-errors.md`，五段式详录在错误台账，本条提炼跨版本可复用模式）。

---

## 发现 #7: gradle 全套状态（native 锁/daemon）都在 GRADLE_USER_HOME——沙箱下指到工作区即可绕开 home 目录权限（2026-09-08）

- **发现时间**：2026-09-08（E10，nether-save-full 课题）
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
