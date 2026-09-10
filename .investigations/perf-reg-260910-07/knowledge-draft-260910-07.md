# 知识库草稿 — 区块 260910-07（Java mod 工程迁出 runtime/：源码入库 + 运行环境原地）

> **本文件 = 草稿，只供主会话应用；未改动 `knowledge/` 与 `docs/` 正文。**
> **格式规范**：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（先过价值门 → 再选载体；无复用价值的结论不写知识库）。
> **产出者**：core.worker / core.knowledge subagent（260910-07 Phase 3.5）。
> **一手复核声明**：本 session **有 pwsh**（与 260910-06 稿的「无 shell」不同）⇒ 本稿对关键数字做了**独立复算**（git 历史/提交号/日期、`git ls-files` 计数、`git check-ignore -v` 双向核、old/new jar + jar 内 dll + target dll 的 sha256、zip 条目数、`results.txt` 原文、`.gitignore`/`build.gradle` 原文），逐项见 §E.4。
>
> **编号依据（读目标文件末尾后取「追加序尾号」）**：
> - `knowledge/discovered/build-tooling.md`：**追加序尾号 = #53**（`## 发现 #53` 在 `:980`，另有其 **E5 补充段** 收在 `:993-998`，为文件最末）⇒ 新条 **#54 / #55 / #56**，追加位置 = `:998` 之后。
>   ⚠️ **该文件编号有历史异常（需主会话知悉）**：文件内存在 `### 发现 #96`（`:854`，260909-03「执行体三元组核验 dev-run 形态」）**插在 #44 与 #45 之间**——即**文件内最大号 = #96，但追加序尾号 = #53**。旁证：`workflow-patterns.md` 从 #95 直接跳到 #97（**无 #96**），且 `INDEX.md:119` 记「260909-03 … build-tooling 新增**发现 #96**」⇒ 260909-03 的草稿很可能**错用了跨文件计数器**。项目此后既有实践是**按追加序续号**（插入 #96 之后，260909-06 仍从 #45 续到 #53），故本稿续 **#54-#56**；**不建议**改跳到 #97（会在 #54-#95 留 42 个空洞，且破坏「追加序」惯例）。另有两处跨文件引用错位一并登记（§E.3-5）。
> - `knowledge/discovered/workflow-patterns.md`：**最大号 = #115**（`### 发现 #115 简记` 在 `:1900`，文件末条）⇒ 新条 **#116 / #117**；任务书第 6 条（可选）建议**不占新号**，作 **#28 补充案例** 就近挂靠（理由见 §E.1-6）。
> - `knowledge/INDEX.md`：当前末段 = `:135` 的 260910-06 追加块 ⇒ 本块追加行置于其后。
> - `versions/1.20.1/docs/10-timewise-archive.md`：当前末块 = `:3089-3103` 的 260910-06 块 ⇒ 本块时间线追加位置 = `:3103` 之后。
>
> **原料**：`.investigations/perf-reg-260910-07/migration-errors-260910-07.md`（E1-E4 五段式 + 速查表）、`.artifacts/perf-reg-260910-07/verdict-260910-07.md`（**candidate**，judge 未做 / 用户未 confirmed）、`.investigations/000-架构设计/架构计划-260910-07.md`（§2 事实表 / §3 迁移设计 / §4 验证判据 / §6 judge 预置）、一手证据 `.investigations/perf-reg-260910-07/cmd-output/{equivalence-260910-07.txt,ignore-checks-260910-07.txt}`、一手件 `.tmp/equiv-260910-07{,-1216}/{old,new}.jar` + `.tmp/perf-reg-260910-07/results.txt`、现文 `.gitignore` / `versions/1.20.1/java/build.gradle` / `knowledge/discovered/{build-tooling,workflow-patterns}.md`。

---

## A. `knowledge/discovered/build-tooling.md` 拟写入正文（追加到 `:998` 之后）

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

## B. `knowledge/discovered/workflow-patterns.md` 拟写入正文（追加到 `:1912` 之后）

### 发现 #116（最高价值）: 结构性迁移的等价性门 = **产物全量条目级 sha256（最好达到整文件逐字节相同）**——把「只搬位置、不改语义」变成二值强判据（260910-07）

- **发现时间 / 发现者 / 置信度 / module**：260910-07（实际 2026-09-10 22:40–23:1x，日期锚 Get-Date 22:40）；主会话（迁移 + 等价性采集）+ 本稿 subagent（**独立复算**：两版本 old/new jar sha、zip 条目数、jar 内 dll sha 与 target dll sha，逐项一致）；**candidate**（证据 = 全量条目比对 + 整文件 sha，无抽样；judge 未做、confirmed 留人类）；workflow-patterns / 迁移·重构等价门（#47 golden 逐位不变对照法的**搬运形态**、#115 内容指纹门的**不经过运行**形态、#52 确定性 dump 载体的载体域延伸）。
- **来源定位**：判决 `.artifacts/perf-reg-260910-07/verdict-260910-07.md` §3（V1/V3）；证据 `.investigations/perf-reg-260910-07/cmd-output/equivalence-260910-07.txt`；一手件 `.tmp/equiv-260910-07/{old,new}.jar`（1.20.1）与 `.tmp/equiv-260910-07-1216/{old,new}.jar`（1.21.6）；迁移前旧产物 `runtime/<ver>/java/build/libs/*.jar`（**原地保留未删**，判决 §7.2 明令不得误删）；迁移提交 `d5151a1`。
- **做法（四步，可直接搬）**：
  1. **迁移前先冻一份 pre 产物**（本案靠旧 `build/libs/*.jar` 原地保留；**无 pre 产物 ⇒ 此门不可做**——这也是 #54 的代价传导面：源码离开 VCS 时 pre 只能靠旧 jar 反推）；
  2. 迁移后**同配方构建 post 产物**（同 dll、同依赖、同 gradle 配置）；
  3. **三层比对**：**a. 整文件 sha256 逐字节相同**（最强——蕴含条目集/顺序/压缩参数全同）→ **b. 条目级 sha256 全等**（排除 `META-INF/**`；**描述性更强**，能定位到具体差异条目）→ **c. 条目数 + `only-old` / `only-new` / `content-diff` 三个差集全为 0**；
  4. **配套执行体三元组**（jar sha / jar 内 `native/worldgen.dll` sha / `target/release` 产物 sha 三者一致）确认 post 产物**与本次构建同源**（#96 家族、#33/#36 载体核查）。
- **本案例读数（本稿 subagent 一手复算，非转述）**：**1.20.1** old = new = `297680e796b64c50…`（zip 条目 **1147/1147**；非 META-INF **1077/1077**，`only-old`/`only-new`/`content-diff` 全 0）；**1.21.6** old = new = `16d5e5e7adf0780c…`（**1908/1908**；**1797/1797**，全 0）；jar 内 dll = target dll（1.20.1 `597e12ed…`、1.21.6 `abd7d889…`，两版 `TRIPLE MATCH = True`）。
- **判据强度（为什么值得写）**：
  - 「跑一遍看着没坏」= **非判据**（抽样、无锚、不可复算）；
  - 「日志看着正常」= 弱判据（受运行期非确定/缓存影响，本项目已多次被 #18/#19/#51 类现象骗过）；
  - **逐字节相同 = 二值强判据 + 现成复现锚**——任何人可用同配方复算并比对 hash；且**覆盖面可声明**（全量条目、无抽样，`§9.7` 三要素齐全）；
  - **等价性门与入口回归是两件事，都要做**：前者证「**语义不变**」；后者（迁移后 MUST 跑一次真实入口 `build + run`）证「**接线/路径仍通**」——本案入口回归恰好暴露了 `runDir` 解析基准（build-tooling #55）与驱动拼接式路径（#117），静态 grep 未覆盖；**等价性门很强，但它对「路径/配置面」零判别力**（jar 内容不含 runDir）。
- **边界（§9.7 可比性声明，照抄进条目）**：① 载体 = 本机 Gradle + fabric-loom 1.10.5 + jdk-17.0.12 + `GRADLE_USER_HOME=E:\PYTHON\CoreSwap\.gradle-home`；② 覆盖面 = jar **全部非 META-INF 条目**（1077 / 1797，无抽样），**未覆盖**：`content-test` 子工程产物、dev-run 参数面（loom runs/runDir 不进 jar）、1.21.6 的运行回归（V2 只跑 1.20.1 一臂）；③ 可比性 = 新旧 jar 出自**同一 source tree 内容**（仅位置不同）⇒ 逐条目比对有效；与 260910-06 的性能数字**不可混读**（本块 38 s 是路径回归读数，不是性能实验）。
- **家族索引**：#47（golden 逐位不变对照法——本条为其**搬迁**域形态：变量是「位置」而非「代码」）、#115（噪声无关内容指纹门——本条更廉价：**不经过运行**）、#52（确定性 dump 载体）、#33/#36（载体可比性 / 执行体三元组）、build-tooling #54（无 VCS ⇒ 无 pre 产物的代价）、#56（`--rerun-tasks` 得到的「注释级改动 jar 不变」是本门的一环）。

### 发现 #117 简记: 迁移的引用面 = **活配置 vs 历史归档**两分——活引用 MUST 改、历史证据 MUST NOT 改写（260910-07）

- **发现时间 / 置信度 / module**：260910-07；candidate（本块四类活引用逐项落地 + 历史归档逐项不改；**E2 实测**暴露「有界静态扫描」漏面）；workflow-patterns / 迁移纪律（**#77「回滚 ≠ 引用面清理」的分界形态**）。
- **来源定位**：架构计划 `架构计划-260910-07.md` §1 D5 + §3「引用更新」行；错误台账 **E2**；落地实例 = `AGENTS.md` §四路径声明、`versions/1.20.1/java/run_rust_client.ps1` 的 `$runJava`、`.tmp/perf-reg-260910-07/run_arms_1201.ps1`（拆 `$run`/`$rd`）、`.gitignore:97-104`。
- **两分清单（本案实际处置）**：
  - **活引用（MUST 改）**：`.gitignore` 规则、`AGENTS.md` 的「现役 Java 权威路径」、工程内脚本（`run_rust_client.ps1` 的 `$runJava`）、**`.tmp/` 一次性驱动脚本的路径推导**、发布契约 / 工单**模板**（新工单须用新路径）；
  - **历史证据（按历史读、MUST NOT 改写）**：`.investigations/**` 归档日志、已发工单（如 `.artifacts/releases/RELEASE-1.0.26/27.md`）、`versions/*/docs` 时间线——**改写 = 篡改证据链**；路径变更点只在**新块文档**里注明（本案判决/计划/本稿即承载）。
- **关键实战判据（本条的核心增量）**：**活引用面不能靠静态 grep 保证完整**。本块事前做了有界扫描（**排除 `.tmp/`、`build`、`.gradle`**）并覆盖了 `.gitignore`/`AGENTS.md`/工程内脚本，**仍漏掉 `.tmp/` 驱动**——它的路径推导**写在变量拼接里**（`$run = "…\runtime\1.20.1\java"` + 处处 `"$run\run\…"`），关键词 grep 也易漏；失败形态是运行时 `FAIL no Done` / `找不到 …\versions\1.20.1\java\run\server.properties`。⇒ **迁移后 MUST 跑一次真实入口（build + run）**，它是最快的兜底（本案一轮即暴露）。
- **修法示例（本案）**：驱动拆**两个路径变量**——`$run`（**工程目录**，交给 gradle 的 workdir）/ `$rd`（**运行环境目录**，给 world / `server.properties` / chunky tasks / region 采集）；「工程目录 + `\run`」的**拼接式推导**在「工程与环境分家」后必然失效（这正是 260910-06 驱动的写法）。
- **家族索引**：#77（回滚 ≠ 引用面清理——引用面扫描）、#90（转抄漂移——历史文档「按历史读」的意义）、AGENTS.md §八.10（目录迁移后全局更新相对引用）、build-tooling #54（同一块迁移的资产侧形态）。

### #28 补充案例（260910-07，任务书第 6 条·可选，**建议不占新号**）: 迁移/重构后的单点性能离群——先核等价性证据，等价则归因直接闭到环境

> **载体建议**：任务书给的是「compiler-idioms **或** build-tooling 简记」；本稿建议改为**workflow-patterns #28 就近挂靠**，理由：① 内容属**归因纪律**（不是语言/工具惯用法，也不是构建链机制），② #28 已立「并发 bench 污染 / **单点离群先复跑再归因**」，本条是它的**前置一步**，拆到别的文件会割裂判据。若主会话坚持按任务书载体落，同一段正文改挂 **build-tooling 简记（续 #57）**即可，文字可原封搬（见 §E.1-6）。

- **现象（260910-07）**：迁移后**首臂** `oa-r1` 跑出 **66 s**（wallgen 72.2 s、6.58 核、`inflightMax=23`），而 260910-06 同形态家族值为 37-38 s；**复跑同臂** `oa-r2` = **38 s**（wallgen 41.3 s、11.53 核），回到家族值（一手 `.tmp/perf-reg-260910-07/results.txt`）。
- **判据（可复用）**：#28 已立「单点离群**先复跑**再归因」；本条补**前置一步**——**先核等价性证据**：本块 V1 已证迁移前后 jar **逐字节相同**（同 dll、同 seed、同区域）⇒ **「搬位置」不可能改变生成耗时**，离群可直接归因**环境**（当时的并发文件操作 —— 同批在做等价性解包/搬运 —— 或冷 gradle daemon / JIT 预热），复跑只作**确认**而非**定性前提**。反之，**若无等价性硬证据**，离群必须停手先查「重构是否真无影响」，不得先归因环境（那是「结论先行」）。
- **边界**：66/38 s 是**路径回归读数，不是性能实验**——与 260910-06 的性能数字**不可混读**（§9.7；判决 §4.3 已声明）。
- **家族索引**：#28（并发 bench 污染 + 单点离群先复跑）、#51（噪声基线与信号同阶）、#116（本条的等价性证据来源）。

---

## C. `knowledge/INDEX.md` 拟追加行（追加到 `:135` 之后，沿用「`> YYMMDD-## 追加：…来源：…`」单段式）

> 260910-07 追加：**build-tooling 新增发现 #54（最高价值·错误优先）**（目录级「整树 untrack / ignore」连带吃掉住在里面的**源码资产**——`0fc44d3`（2026-08-30 20:29）为「runtime = 本地运行环境不入库」给 `.gitignore` 加 `/runtime/`，实测同时把 `runtime/1.20.1/java/` 下 **94 个已跟踪文件**（build.gradle/settings.gradle/gradle.properties + `src/main/java/wg/**` 源码 + content-test + 3 份 vanilla 散件）一起移出版本管理；14 天内同族另一犯 = 08-24 把 `versions/1.20.1/java/` gitignore「留在 MC 只读」致工程「找不回来」，`ae86155` 迁回；离库窗口 **08-30 20:29 → 09-10 22:56（`d5151a1`）≈ 11 天**（⚠️ 订正任务书「5 周」）；判据 MUST = `git ls-files <dir>`（已跟踪资产清单）× `git check-ignore -v`（规则命中链）**双向**核 +「用途标签不足以授权整树规则」+「承载体不混装」；修复 = **工程/环境物理分离**（`versions/<ver>/java` 入库 + `runtime/<ver>/java/run` 原地由 `runDir` 指回）+ 只忽略派生/缓存 + **嵌套 prune 白名单链**（全局 `data/`+`**/data/*` 会 prune `worldgen-data/data/**`，每版本 **1015** 个 JSON 靠 `.gitignore:109-110` 放行，#24 三段链第二落地）；#24 家族**方向相反**的第二形态）+ **发现 #55 简记**（loom `runDir` 是 **String 且按工程目录相对解析**——`javap -p`（loom 1.10.5）实证；绝对路径被拼成 `工程目录\E:\…` ⇒ `CreateProcess error=267`；定位 = **读错误里 `(in directory …)` 解析后的路径** + 从 gradle 缓存 `javap` 读真实签名；判据 = 凡指向工程外的路径配置都要核解析基准（loom 工程目录 / gradle `-P` 闭包作用域 #47 / gitignore 仓库根 #24）；修复 = `../../../runtime/<ver>/java/run`）+ **发现 #56 简记**（沙箱内 gradle **文件监视失效 ⇒ `UP-TO-DATE` 假绿**——日志 `File watcher server NativeException: Couldn't open current thread, error = 5`，源码刚改仍报 UP-TO-DATE；与 #1（doFirst copy 非 task input）为同一「UP-TO-DATE 假绿」家族的两个机制面；纪律 = **验证性构建一律 `--rerun-tasks`**，涉及「产物是否变了」的结论不得只信 up-to-date；260910-06 已实际依赖此判据、本块补记）；**workflow-patterns 新增发现 #116（最高价值）**（结构性迁移的**等价性门 = 产物全量条目级 sha256（最好整文件逐字节相同）**——本案 1.20.1 old=new=`297680e7…`（zip 1147/1147、非 META-INF **1077/1077**，only-old/only-new/content-diff 全 0）、1.21.6 old=new=`16d5e5e7…`（1908/1908、**1797/1797**、全 0）+ 三元组 MATCH，把「只搬位置不改语义」变成**二值强判据 + 现成复现锚**；四步 = 迁移前冻 pre 产物 → 同配方构建 post → 整文件 sha / 条目级 sha / 差集三层比对 → 执行体三元组；**等价性门证「语义不变」，入口回归（build+run）证「接线仍通」，两件事都要做**——后者才暴露 `runDir` 解析基准与驱动拼接式路径；边界 = 不覆盖 dev-run 参数面/子工程/1.21.6 运行回归）+ **发现 #117 简记**（迁移的引用面 = **活配置 vs 历史归档**两分——活引用（`.gitignore`/`AGENTS.md`/工程内脚本/`.tmp/` 驱动/发布模板）MUST 改，历史证据（`.investigations/**`、已发工单、`versions/*/docs`）MUST NOT 改写（改 = 篡改证据链）；**活引用面不能靠静态 grep 保证完整**——本块有界扫描（排除 `.tmp/`）漏掉 `.tmp/` 驱动，其路径推导写在变量拼接里 ⇒ 迁移后 **MUST 跑一次真实入口**兜底；驱动拆 `$run`（工程）/`$rd`（环境）两变量）+ **#28 补充案例（可选，不占新号）**（迁移/重构后的**单点性能离群先核等价性证据**——jar 逐字节相同 ⇒ 「搬位置」不可能改变耗时，归因直接闭到环境（并发负载/预热），复跑只作确认；无等价性证据则必须先查重构影响，不得结论先行）。来源：`.investigations/perf-reg-260910-07/{migration-errors-260910-07.md,knowledge-draft-260910-07.md,cmd-output/{equivalence-260910-07.txt,ignore-checks-260910-07.txt}}` + `.artifacts/perf-reg-260910-07/verdict-260910-07.md`（candidate，judge 未做 / 用户未 confirmed）+ `.investigations/000-架构设计/架构计划-260910-07.md`；一手件 `.tmp/equiv-260910-07{,-1216}/{old,new}.jar`、`runtime/<ver>/java/build/libs/*.jar`（**历史产物，不得误删**）。

---

## D. `versions/1.20.1/docs/10-timewise-archive.md` 拟追加的 260910-07 时间线块（追加到 `:3103` 之后）

> 格式对齐现网：`## YYMMDD-NN（实际 …锚定：…）状态` → 一段 `> 过程产物/通用模式` → `- ✅/❌/🔍` 过程条目；**只记过程与排除项，不搬结论数字堆**（数字在判决/主题篇）。

## 260910-07（实际 2026-09-10 22:40–23:1x，日期锚 Get-Date 22:40：Java mod 工程迁出 `runtime/`（源码入库 + 运行环境原地，两版本 ×2））🔍 candidate（judge 未做 / 用户未 confirmed）

> 过程产物 `.investigations/perf-reg-260910-07/`（`migration-errors-260910-07.md` E1-E4 五段式 + 速查表 / `cmd-output/{equivalence-260910-07.txt,ignore-checks-260910-07.txt}` / `knowledge-draft-260910-07.md`）；架构计划 `.investigations/000-架构设计/架构计划-260910-07.md`（**HOOK-1 用户批准三项**：目标路径 `versions/<ver>/java`、环境原地 `runtime/<ver>/java/run`、入库范围含 worldgen-data + 散件隔离）；判决 `.artifacts/perf-reg-260910-07/verdict-260910-07.md`（**candidate**）；提交 `d5151a1`（2876 files changed）；通用模式 → workflow-patterns #116/#117（+ #28 补充案例）、build-tooling #54/#55/#56（subagent 草稿 → 主会话应用）。

- ✅ **范围与形态（结构重构，非语义改动）**：两个 MC 版本（1.20.1 / 1.21.6）的 loom dev 工程（= **出货 mod 源码本体**：`src/main/java/wg/**` **49 文件 ×2** + 构建定义 + `content-test` + `worldgen-data` 1019/1739 文件）由 `runtime/<ver>/java/` 迁 `versions/<ver>/java/`；**运行环境原地不动**（`runtime/<ver>/java/run/` 世界/mods/`server.properties`，698MB/56MB 零搬迁），由 `build.gradle` 的 `loom { runs { server/client { runDir "../../../runtime/<ver>/java/run" } } }` 指回。
- ✅ **V1 等价性门（决定性，全量无抽样）**：迁移前后构建 jar **逐条目 sha256 全等且 jar 整体 sha 逐字节不变**——1.20.1 **1077/1077**（only-old/only-new/content-diff 全 0）、1.21.6 **1797/1797**；jar 整体 sha 1.20.1 `297680e7…`、1.21.6 `16d5e5e7…` 前后同值 ⇒ **迁移零语义影响**（本稿 subagent 已独立复算 old/new jar sha + zip 条目数 1147/1908 + jar 内 dll sha，逐项一致）。
- ✅ **V3 三元组**：新 jar sha / jar 内 `native/worldgen.dll` sha / `target/release/worldgen{,1216}.dll` sha 两版本均 MATCH（`597e12ed…` / `abd7d889…`，一手复算一致）。
- ✅ **V2 路径回归（1.20.1 一臂）**：新路径 `gradle :runServer`（`-PcppReplace=true` + `-PcppWorldgenDir=…versions/1.20.1/data/worldgen` + `-Pchunktime=1`）跑通——Chunky `Processed: 4225`、`inflight max=23`、无 STALL、`run/` 仍在 `runtime/` 下被读写（`oa-r2` 38 s / wallgen 41.3 / 11.53 核）。
- ✅ **V4 入库双向核（8 条全对）**：源码 / `worldgen-data` / 构建定义 = **未忽略**；`run/`、`build/`、`.gradle/`、`native/`、`runtime/` = **忽略**（一手复跑 `git check-ignore -v` 与 `git ls-files` 计数 0 双重确认；无 `??` 噪声）。
- ✅ **V5 回滚**：反向 `Move-Item` + 还原 `.gitignore`/`runDir` 步骤自足（**记录即算**，未实跑）；旧 `runtime/<ver>/java/build/libs/*.jar` 与旧 `run/` **均原地未动**（历史产物/1.0.27 pre 证据，不得误删）。
- ❌ **E1（响亮失败）loom `runDir` 绝对路径被拼坏**：`CreateProcess error=267 目录名称无效`，错误里工作目录 = `…\versions\1.20.1\java\E:\PYTHON\…` ⇒ 根因 = `runDir` 是 **String 且按工程目录相对解析**（`javap -p` 实证，loom 1.10.5）；修复 = 相对路径 `../../../runtime/<ver>/java/run`；判据 = **读错误里解析后的路径**（→ build-tooling #55）。
- ❌ **E2 驱动脚本「工程目录 + `\run`」推导式失效**：`FAIL no Done` / `找不到 …\versions\1.20.1\java\run\server.properties` ⇒ 迁移后工程与环境分家；**有界扫描（已排除 `.tmp/`）漏掉 `.tmp/` 驱动**（路径推导写在变量拼接里，grep 易漏）⇒ 修复 = 驱动拆 `$run`（工程）/`$rd`（环境）两变量；判据 = **迁移后跑一次真实入口**比纯静态扫描更快暴露（→ workflow-patterns #117）。
- ❌ **E3 沙箱内 gradle `:build UP-TO-DATE` 假绿**：改了 Java 注释仍报 UP-TO-DATE、jar 未变，日志有 `File watcher server … NativeException: Couldn't open current thread, error = 5` ⇒ 文件监视失效使 VFS 陈旧；修复 = 验证性构建 `--rerun-tasks`（本块据此得到「注释级改动后 jar 逐字节相同」这一等价性门的一环；→ build-tooling #56）。
- ❌ **E4 首臂 66 s 离群（家族 37-38 s）——已排除迁移效应**：V1 证 jar 逐字节相同 ⇒ 搬位置不可能改变生成耗时；复跑 `oa-r2` 得 38 s 回到家族值 ⇒ 归因环境（同批并发文件操作 / 冷 gradle daemon / 预热），**非迁移效应**；如实登记为「单点离群先复跑再归因」，并补「有等价性硬证据时归因可直接闭到环境」（→ #28 补充案例）。
- 🔍 **open（未核 / 降级 / 边界）**：① judge **未做**（计划 §6 预置的 candidate-SHOULD / 收尾 FIN-MUST 均未签）、confirmed 未授予；② **证据归档缺口**：判决 §1 声明的证据含 `results.txt` / `logs/`，但二者实际只在 `.tmp/perf-reg-260910-07/`（**临时区，不入库，迟早灭失**）——建议复制进 `.investigations/perf-reg-260910-07/cmd-output/`（证据优先级高于工作区整洁，260910-06 E1 教训）；且 `results.txt` 中 `oa-r1` 同时存在两条 `FAIL_no_done` 与一条完整 66 s 行（**同名臂跨「失败轮/测量轮」复用**），引用需注明轮次；③ **V4 归档证据缺嵌套 prune 那一格**：`ignore-checks-260910-07.txt` 8 条中没有 `worldgen-data/data/**`（而 `.gitignore:109-110` 白名单链正是为它加的，每版本 1015 个 JSON）——本稿已独立复算确认其在库且未被忽略，建议补一行进证据文件；④ 等价性门**只覆盖 jar 产物**——不覆盖 dev-run 参数面 / `content-test` 子工程产物 / 1.21.6 运行回归（V2 仅 1.20.1 一臂），且**不证明 dev-run 行为完全一致**；⑤ `content-test/run`（0.1 MB）随子工程迁移（不影响主流程，已由 `versions/*/java/*/run/` 忽略）；⑥ `runtime/<ver>/java/{build,.gradle,.gradle-home,scripts}` 旧残留**原地保留**、gradle home 仍有多份（`$root\.gradle`、`$root\.gradle-home`、`runtime/...`）= 文件管理债，本块未动（登记后续清理）；⑦ `.investigations/**` 历史归档的路径**按历史读、不改写**（改写 = 篡改证据链），新块文档注明变更点；⑧ 上游影响：**260910-05（1.21.6）旧读法数字复算**仍挂账（260910-06 E5 主题），与本块无关但同批未闭；⑨ 本块跨两版本，时间线**只落 1.20.1**（与 260910-06 同做法）——1.21.6 时间线无 260910-06/07 块，是否加一行指针由主会话定。

---

## E. 自检清单结论（价值门逐条 + 不写项 + 与原料的矛盾/缺口）

### E.1 价值门逐条判定（任务书 6 条）

| # | 内容 | 价值门 | 载体（决定性理由） | 判定 |
|---|---|---|---|---|
| 1 | 目录级 untrack/gitignore 整树规则**连带吃掉源码** | **高价值（必记）**——错误链 + 判错判据（双向核）+ 反模式 + **15 天内同族两犯** | build-tooling **#54**（gitignore/VCS 规则载体就在此；#24 家族） | 写（详写，「现象/根因/定位/修复/教训」齐全） |
| 2 | 结构性迁移的**等价性门 = 条目级 sha256（整文件逐字节相同）** | **高价值（必记）**——把「只搬位置不改语义」变成二值强判据 + 现成复现锚 + 可搬四步 | workflow-patterns **#116**（等价门手法域：#47/#115/#52 家族） | 写（详写） |
| 3 | 迁移的引用面 = **活配置 vs 历史归档** | **高价值（必记）**——纪律 + 分界判据（改/不改）+ 「静态 grep 不可保证」的实测反例 | workflow-patterns **#117 简记**（#77 分界形态） | 写（简记） |
| 4 | loom `runDir` String 按工程目录相对解析 | **高价值（错误链）**——响亮失败 + 「路径配置解析基准」通用判据 | build-tooling **#55 简记**（五段式压缩，完整五段式在 `migration-errors-260910-07.md` E1） | 写（简记） |
| 5 | 沙箱 file-watcher 失效 ⇒ `UP-TO-DATE` 假绿 | **高价值（必记）**——环境坑 + 「构建绿 ≠ 改动进去」判据；且 260910-06 已实际依赖未落盘 | build-tooling **#56 简记**（#1/#23/#25/#40 家族） | 写（简记 + 补记说明） |
| 6 | 迁移后性能离群先查等价性证据（**可选**） | **中价值（简记）**——判据可复用，但与 #28「单点离群先复跑再归因」强重叠，增量只有「前置一步」 | **建议**：workflow-patterns **#28 补充案例**（不占新号；任务书给的 compiler-idioms/build-tooling 亦可，文字可原封搬） | 写（简记；**载体建议偏离任务书，理由见上**） |

### E.2 我认为**不该写**的项（及理由）

1. **一次性数字 / 状态快照**：jar 全串 sha、zip 条目 1147/1908、66 s / 38 s / wallgen 72.2 / 41.3 / 6.58 核 / 11.53 核、`inflight max=23`、dll sha 全串——**低价值（不记）**：判决表与时间线已承载；本稿仅在**作为判据锚**处保留 hash 前缀（#116 需要可复现锚）。写进 discovered 只稀释高价值权重。
2. **散件隔离处置**（`MaterialRules.java` / `NoiseChunkGenerator.java` / `VanillaSurfaceRules.java` / `x`(4.2MB) / 58 个 stray `.log` → `.tmp/legacy-runtime-260910-07/`）——一次性工程处置，无跨块复用价值（判决 §2 已记，可回滚）。
3. **`content-test/run` 随子工程迁移、`runtime/` 侧残留（build/.gradle/.gradle-home/scripts）、gradle home 多份**——一次性状态 / 清理债（判决 §7 已记 + 本稿时间线 open），**不写知识库**。
4. **`AGENTS.md` 路径声明更新、`.gitignore` 具体规则行**——属**主会话直接改的现役配置**（不在知识库载体），且判决 §2 已清单化；本稿不重复。
5. **`.artifacts/index.yaml` 追加条目**——主会话手工追加（**#50 警告：带注释的根索引禁跑 `merge_index.py`**，会静默丢注释与四字段外内容）；非本次知识库草稿的产出面。
6. **loom `RunConfigSettings` 的 javap 方法面单列 `compiler-idioms`**——不单列：事实（`private String runDir` / `setRunDir(String)`）已含在 #55 内，单列会与 #55 重复；若主会话希望 API 惯用法与判据分离，可拆一条 compiler-idioms 简条（本稿不占号）。
7. **`equivalence-260910-07.txt` / `ignore-checks-260910-07.txt` 的逐行数字**——证据件本身即载体，不搬进 discovered。
8. **1.21.6 侧重复流程**——两版本同配方，条目按机制写一次即可（数字在边界/覆盖面里声明「两版本各自独立验证」）。

### E.3 与原料的**矛盾 / 缺口**（建议主会话优先处置）

1. **⛔「5 周 Java 侧改动无 git 史」与实测不符（最高优先）**：任务书称 untrack 后 **5 周**无 git 史；实测 `0fc44d3` = **2026-08-30 20:29**、恢复提交 `d5151a1` = **2026-09-10 22:56** ⇒ **≈ 11 天**（即使从 `b2b9bea` 08-29 19:51 搬进 runtime 算也只有 12 天）。**建议条目标「≈11 天」**，并把同族更早一犯（08-24 前后 gitignore `versions/1.20.1/java/` →「找不回来」→ `ae86155` 08-26 迁回，约 2 天）作为**第二个实例**——「15 天内同族两犯」比「一次 5 周」在论证「判据缺失」上更强，也更真实。
2. **判决 §1 的证据路径与实际不符（缺证据的断言面）**：`verdict-260910-07.md` 第 6 行称证据含 `.investigations/perf-reg-260910-07/cmd-output/{…,results.txt,logs/}`；实测该目录**只有** `equivalence-260910-07.txt` 与 `ignore-checks-260910-07.txt`，`results.txt` / `logs/` / `run_arms_1201.ps1` / `rcon_one.py` 全在 **`.tmp/perf-reg-260910-07/`**（临时区，#18/#31 家族「产物在盘 ≠ 已归档」）。⇒ 建议把 `results.txt` + `logs/oa-r1.log(.err)` + `logs/oa-r2.log(.err)` + 驱动脚本复制进 `.investigations/perf-reg-260910-07/cmd-output/`（V2「路径回归跑通」目前**无归档证据**支撑）。
3. **`oa-r1` 同名臂跨「失败轮 / 测量轮」复用**：`results.txt` 里 `oa-r1` 先有两条 `FAIL_no_done`（对应 E2 的失败轮），随后又有一条完整 66 s 行；判决/E4 只写「首臂 66 s」，未提失败轮同名。⇒ 引用时须注明轮次；建议驱动对失败轮与测量轮分名（如 `oa-r1a` / `oa-r1b`）。
4. **V4 归档证据缺「嵌套 prune 白名单」那一格**：`ignore-checks-260910-07.txt` 8 条里没有 `versions/<ver>/java/src/main/resources/worldgen-data/data/**`——而 `.gitignore:106-110` 的白名单链**正是为它加的**（全局 `data/` + `**/data/*` 会 prune 掉每版本 **1015** 个 JSON）。本稿已独立复跑 `git check-ignore -v`（未命中忽略）与 `git ls-files`（1015 个在库）确认修复有效，**建议补一行进证据文件**，使「V4 双向核」覆盖到它存在的理由。
5. **build-tooling 编号异常（影响取号，必须说明）**：文件内 `### 发现 #96`（`:854`，260909-03）插在 **#44 与 #45 之间**；`workflow-patterns.md` 从 #95 跳到 #97（**无 #96**）；`INDEX.md:119` 亦记「build-tooling 新增发现 #96」⇒ 260909-03 草稿很可能**错用跨文件计数器**（workflow-patterns 的号段在 260909-02 已到 #95）。另有**跨文件引用错位**两处：① `build-tooling #49` 家族索引写「#20/**#53**（参数静默不生效）」——build-tooling #53 实为 region 对拍工具，语义对不上（**应指 workflow-patterns #53**）；② `build-tooling :553` 标题「发现 #25 补充案例（**#61**)」——#61 不在本文件号段。⇒ 本稿按**追加序尾号 #53** 续 **#54-#56**（与项目既有实践一致：插入 #96 后 260909-06 仍从 #45 续到 #53）；**不建议**跳到 #97。
6. **判定状态**：判决 = **candidate** 且 **judge 未做**（计划 §6 预置 candidate-SHOULD / 收尾 FIN-MUST 均未签）⇒ 本稿全部条目维持 candidate / 简记级，**无一条可 confirmed**（confirmed 仅由用户授予）；若 judge 挑战 #116/#54，本稿对应条目须一并回改。
7. **时间线归属**：本块跨两版本（各 49 源文件），任务指定落 **1.20.1** 时间线（与 260910-06 同做法）；`versions/1.21.6/docs/10-timewise-archive.md` **无 260910-06/07 块** ⇒ 建议在其 260910-05 块后加**一行指针**（可选，主会话定）。
8. **理论边界（须随条目声明）**：等价性门只覆盖 **jar 产物内容**——不覆盖 dev-run 参数面（`runDir`/loom runs 不进 jar）、`content-test` 子工程产物、1.21.6 运行回归；且「jar 不变」**不证明 dev-run 行为完全一致**（判决 §7.5 自述）。

### E.4 数字来源与复核边界（诚实声明）

- **本稿 subagent 本 session 有 `pwsh`**（与 260910-06 稿不同）⇒ 下表为**一手复算**，非转述：
  - **git 历史一手**：`0fc44d3`（`chore: untrack runtime/ (local-only mod runtime harness)`，2026-08-30 20:29；`--stat` = **94 D** + `.gitignore` +3；提交信息原文核对）；`ae86155`（2026-08-26 22:18，提交信息含「prior migration gitignored versions/1.20.1/java/ ('leave probe at MC read-only')」；`--name-status` 计 **90 A**）；`b2b9bea`（2026-08-29 19:51）、`03732db`（2026-08-05 19:28）、`d5151a1`（2026-09-10 22:56，**2876 files changed**）。
  - **文件计数一手**：`git ls-files versions/{1.20.1,1.21.6}/java/src/main/java` = **49 / 49**；`.../worldgen-data/**` = **1019 / 1739**；其中 `.../worldgen-data/data/**` = **1015**（1.20.1）；`build`/`run`/`.gradle`/`native` 下已跟踪 = **0**；`git status --porcelain` 两个 java 树 = **0 条**。
  - **ignore 双向核一手**：源码 & `biome_params.json` & `build.gradle` = 未忽略（无输出）；`run/server.properties` → `.gitignore:102`；`build/libs/x.jar` → `:100`；`native/worldgen.dll` → `:104`；`.gradle/x` → `:101`；`content-test/run/x` → `:103`；`runtime/1.20.1/java/run/server.properties` → `:95`（与归档证据 `ignore-checks-260910-07.txt` 一致）。
  - **等价性 / 三元组一手复算**：`Get-FileHash SHA256`——`.tmp/equiv-260910-07/{old,new}.jar` 均 `297680E7…E35CB`；`-1216/{old,new}.jar` 均 `16D5E5E7…E77BFC8E`；zip 条目 **1147 / 1908**（old=new）；jar 内 `native/worldgen.dll` = `597e12ed…` / `abd7d889…`；`target/release/worldgen.dll` = `597E12ED…`、`worldgen1216.dll` = `ABD7D889…`；旧路径 `runtime/{1.20.1,1.21.6}/java/build/libs/*.jar` 与新路径 `versions/{…}/java/build/libs/*.jar` **同 sha**。
  - **原文一手**：`.gitignore`（含 `:53-55`/`:95`/`:100-110` 注释与白名单链）、`versions/1.20.1/java/build.gradle:199/:205`、`.tmp/perf-reg-260910-07/results.txt`（4 行）、`.investigations/perf-reg-260910-07/cmd-output/*.txt`、`migration-errors-260910-07.md`、`verdict-260910-07.md`、`架构计划-260910-07.md`、`build-tooling.md` / `workflow-patterns.md` 末尾与编号普查、`INDEX.md:119/:135`、`10-timewise-archive.md:3089-3103`。
- **未复核**：逐条目 sha256 的全量重算（**未做**——但**整文件 sha 相同已蕴含条目全同**，故本条不妨碍结论；归档证据 `equivalence-260910-07.txt` 载有条目级读数）；V2 的 66 s/38 s 行之外的过程（未重跑 gradle/Chunky）；1.21.6 的运行回归（本块未做，判决亦声明未做）；`content-test` 产物比对（未做）。
- **§9.7 可比性**：本稿数字均属 260910-07 载体（MC 1.20.1/1.21.6 + Gradle + fabric-loom 1.10.5 + jdk-17.0.12 + `GRADLE_USER_HOME=E:\PYTHON\CoreSwap\.gradle-home`）；**禁与 260910-04/05/06 的性能数字互引**（判决 §4.3 已声明本块 38 s 是路径回归读数，不是性能实验）。
