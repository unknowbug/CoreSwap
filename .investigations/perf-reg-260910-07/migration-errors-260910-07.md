# 错误台账 — perf-reg-260910-07（Java 工程迁出 runtime/）

> 载体规范：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（五段式：现象 → 根因 → 定位 → 修复 → 教训）。

## E1 — loom `runDir` 是「按工程目录相对解析」的 String，绝对路径被拼坏 ⇒ CreateProcess error=267

**现象**：迁移后 `gradle :runServer` 启动即失败：
`Execution failed for task ':runServer' > A problem occurred starting process 'command 'D:\Program Files\Java\jdk-17.0.12\bin\java.exe''`，
`Caused by: java.io.IOException: Cannot run program "…\java.exe" (in directory "E:\PYTHON\CoreSwap\versions\1.20.1\java\E:\PY…")`
→ `CreateProcess error=267, 目录名称无效。`

**根因**：`loom { runs { server { runDir "E:/PYTHON/CoreSwap/runtime/1.20.1/java/run" } } }` —— loom 的 `RunConfigSettings.runDir`
是 **String**（`javap` 实证：`private java.lang.String runDir; public void setRunDir(String)`），消费侧**按工程目录相对解析**，
于是绝对路径被拼成 `工程目录\E:\PYTHON\…`（无效目录）。

**定位**：① 看错误里的 `(in directory …)` —— 它直接把解析后的工作目录打出来（一眼可见被拼接）；
② 不猜 API：从 gradle 缓存里 `jar xf fabric-loom-1.10.5.jar` + `javap -p …RunConfigSettings` 读真实签名（比试错快且不污染源码）。

**修复**：改相对路径 `runDir "../../../runtime/1.20.1/java/run"`（工程目录 = `versions/1.20.1/java` ⇒ 上三级回到仓库根）。

**教训**：**迁移工程时，凡「指向工程外」的路径配置都要核「解析基准」**——loom `runDir` 按工程目录解析、gradle `-P` 映射按闭包作用域（#47）、gitignore 按仓库根；
「绝对路径一定安全」是错觉。判据 = 直接读**错误信息里解析后的路径**，而不是读你在配置里写的那份。

---

## E2 — 驱动脚本「工程目录 + `\run`」推导环境路径，迁移后两者分家 ⇒ 找不到 server.properties

**现象**：迁移后驱动第 1 次运行 `FAIL no Done`，日志：`找不到路径"…\versions\1.20.1\java\run\server.properties"`（并连带 `$spf.bak` 复制失败）。

**根因**：260910-06 的驱动把**工程目录**与**运行环境目录**当成同一个（`$run = "…\runtime\1.20.1\java"`，随后处处用 `"$run\run\…"`）——
迁移后工程在 `versions/<ver>/java`，而运行环境仍在 `runtime/<ver>/java/run`，该推导式失效。

**定位**：报错路径本身即判据（`versions\…\java\run\…` 不存在）；`Select-String '$run\\run'` 一次列全 4 处。

**修复**：驱动拆两个变量——`$run`（工程，交给 `gradle` 的 workdir）/ `$rd`（运行环境，给 world/server.properties/chunky tasks/region 采集）；
本块的驱动副本已改（`.tmp/perf-reg-260910-07/run_arms_1201.ps1`）。

**教训**：**结构性迁移的引用面必须包含「工作区根的脚本 + `.tmp/` 一次性驱动 + preset/preset 外部脚本」**——
本块事前有界扫描只覆盖了 `.gitignore`/`AGENTS.md`/工程内脚本，**漏了 `.tmp/` 驱动**（它们的路径推导写在变量拼接里，grep 关键词也易漏）。
判据：迁移后**跑一次真实入口**（build + run），比纯静态扫描更快暴露拼接式路径依赖。

---

## E3 — 沙箱内 gradle 报 `:build UP-TO-DATE` 但源码刚改（VFS 陈旧假绿）

**现象**：改了 Java 注释后 `gradle :build` 输出 `> Task :build UP-TO-DATE`，jar 未变；日志有
`Exception in thread "File watcher server" net.rubygrapefruit.platform.NativeException: Couldn't open current thread, error = 5`。

**根因**：沙箱限制导致 gradle 文件监视不可用，增量构建的 VFS 未察觉磁盘改动 ⇒ 报 UP-TO-DATE。
**定位**：`UP-TO-DATE` 与「文件确实改了」冲突时，先看是否有 file-watcher 异常；用 `--rerun-tasks` 强制重编做对照。
**修复**：验证性构建一律 `--rerun-tasks`（本块据此得到「注释级改动后 jar 逐字节相同」的结论）。
**教训**：**「构建绿」不等于「改动被编译进去」**（#23/#25/#40 家族）——涉及「产物是否变了」的结论必须用强制重编或内容指纹验证，不能只信 gradle 的 up-to-date 判定。

---

## E4 — 迁移后首臂 66 s（家族 37-38 s）：离群，不是迁移效应

**现象**：迁移后第一次 `oa-r1` 跑出 66 s（wallgen 72.2 s、6.58 核），而 260910-06 同形态为 37/38 s。
**根因**：**离群**——V1 证明迁移前后 jar **逐字节相同**（同 dll 同 seed 同区域），迁移不可能改变生成耗时；
疑与当时的并发文件操作（同批做等价性解包/搬运）或冷 gradle daemon/JIT 预热有关。
**定位**：复跑同一臂（`oa-r2`）得 **38 s / 41.3 s wallgen / 11.53 核**，回到家族值。
**修复**：无需修复；**记录为「单点离群先复跑再归因」**（workflow-patterns #28 家族）。
**教训**：**「迁移/重构后出现性能差异」先查等价性证据**——若产物逐字节相同，性能差异必然来自环境（并发负载/预热），不要在重构上找原因。

---

## 速查表（错误/现象 → 根因）

| 错误/现象 | 根因 | 判据 |
|---|---|---|
| `CreateProcess error=267`，工作目录 = `工程目录\E:\…` | loom `runDir` 按工程目录相对解析（String） | 读错误里的 `(in directory …)` |
| 驱动找不到 `run\server.properties` | 「工程目录 + \run」推导式在迁移后失效 | 报错路径；`Select-String '\$run\\run'` |
| `:build UP-TO-DATE` 但源码刚改 | 沙箱 file-watcher 失效 ⇒ VFS 陈旧 | 日志 `File watcher server` 异常；`--rerun-tasks` 对照 |
| 迁移后单臂耗时离群 | 并发负载/预热，而非迁移（jar 逐字节相同） | 复跑回到家族值 |
