# B6-1 T5 收尾终审（review-002，core.judge）

```yaml
status: opinion-only        # 只出意见；不修改任何产物 status
role: core.judge（subagent，隔离执行）
block: 260918-01（T5 收尾终审）
subject: B6-1「CoreSwap -P→-D 开关生效机械对账」T5 收尾交付
baseline:
  artifacts_snapshot: .investigations/b61-260917-06/
  git: HEAD=cf77963（工作区 clean；T5 代码入库 7abed4d，门禁/AGENTS 条款 56dc15c，scripts 发现记录 cf77963）
  verification: cmd-output/{sentinel-A,B,C}*.log + t5-postfix-verify.log + 首轮 sentinel-surfacedump.log / vmargs-sentinel.log
method: 三源交叉核对；静态读脚本 + 集合运算复算（见 §0.2 能力边界）
```

## 0. 方法与能力边界

### 0.1 三源核对实际动作
| 源 | 核对动作 | 结果 |
|---|---|---|
| ① 产物快照 | 逐份读 t4-adjudication / classification.yaml / switches-registry.yaml / phase25-verification / t5-landing / review-001 | 已读 |
| ② git HEAD + diff | `git log` / `git status` / `git show 7abed4d -- build.gradle` / `git ls-files scripts/` / `git check-ignore -v` | 已核 |
| ③ 验证记录 | 逐份读 6 个 cmd-output 日志原文 + 独立复算门禁数字 | 已核 |

### 0.2 能力边界（MUST 声明）
**本次审查无 shell/无法执行脚本**。因此：
- `check_switch_mapping.py` 的**行为**（含 `--strict` 退出码）为**静态读码推断**，非实跑观测；
- 其**数字输出**由我用 PowerShell 独立**复算**同一集合运算得到（§C），属**独立复现**而非「照抄 json」——复算口径声明：我按脚本同款正则（`run.vmArg\s+"-D([A-Za-z0-9_.]+)=` / `System.getProperty|Integer.getInteger|Boolean.getBoolean`）在脚本同款载体面（2 gradle + 4 src 树）上取集合交并；**我未运行脚本本体**，故「脚本输出 == 我复算结果」这一点是**等价复现**，不等于「脚本被实跑过」。

---

## 1. A. M1/M2/M3 闭合性

### M1 — 豁免子句 ✅ **CLOSED**
- `t4-adjudication.md:67-82` §3.4 给出**判据的完整表述**：豁免子句两条要件（① 探针专用路径 **且** ② 缺省值即权威）+ **三条豁免项逐条 reason**表（bench.threads / bench.worldgen / colDump.targets）；
- `:82` 注**明确写了合读关系**：「原 §2 规则需与本节合读」「这三条的豁免是判据的一部分，不是事后找补」——**M1 要求的「是否写明需合读」已满足**；
- 三源一致：`classification.yaml:29/30/33` 三条 reason 均回指 §3.4 豁免。

### M2 — 哨兵证据 ✅ **CLOSED**（且证据质量高于要求）
- **三份日志真实存在且内容自洽**。逐份核对：
  - `sentinel-B-calibration.log:28-42` = 同轮 `chunkX=1`/`chunkZ=2` **发射**、`dim` **缺席** → **组内差分成立**（同一次运行、同族三 `-P` 传参，两发一不发）；且该轮 boot 全程走完（`:121` Done (3.053s)）——**仪器有效性由同轮「发射成功」两行自证**，非循环论证；
  - `sentinel-A-surfacedump.log:28-40` = `surfacedump.probe=1` 在、`surfacedump.dim` 不在；
  - `sentinel-C-height.log:28-40` = 仅 `height.probe=true`（父门），零子参数 → 与 SCOPED 规则一致。
- `phase25-verification.md:21-26` §2 **如实披露首轮三缺陷**，含 ① 哨兵 B 原始输出不存在（`vmargs-sentinel.log` 内 height 命中 0——**我复核属实**：该日志 `:27-39` 确无 height 行）② **首轮 `BUILD FAILED` 未披露**（我复核：`vmargs-sentinel.log:127/:141` 确为 `runServer FAILED` / `BUILD FAILED in 13s`）③ 缺正向标定；
  - `:89` §9.8 明确登记**首轮日志「保留不删」的逆**（如实披露优先于整洁）——符合 §9.8 且**未自动回退**。

### M3 — b2 误记纠错 ✅ **CLOSED**
- `t4-adjudication.md:65` 纠错行已把原措辞「b2 该 2 项误判」改为「**对 b2 让渡项的确认，非推翻**」，并附 b2 原文位置（`.b2-defect-classification.md:44/:252`）；
- `classification.yaml:38` 同口径：「确认 b2 让渡项（b2 未判缺陷）」——**交叉一致**。

> A 项结论：**M1/M2/M3 三项全部真闭合**，且均有一手证据支撑（非声明式闭合）。

---

## 2. B. T5 落地的真实性与正确性（本次重点）

### B1. 作用域 ✅ **PASS**（#47 已核，且比文档更精确保守）
- `benchVmArgs` 闭包 = `build.gradle:68` 起（`:253/:259` 两处调用）；
- 补的 **13 行全部落在闭包内**（行号：172/173/177/178/179/180/200/201/202/203/213/214/223），**无一越界**；
- 注记行同样在闭包内（117-119 / 147-150）。
- ⚠️ 但见 **S-new-1**（缩进异常，虽不破坏 Groovy 语义）。

### B2. `-P` 命名与族对齐 ✅ **PASS**
逐行核对 13 项，全部**与所在族既有命名风格一致，且映射到正确的 `-D` 名**：

| 补的 `-P` | 族 | 族内既有命名风格 | 判定 |
|---|---|---|---|
| `surfaceDumpDim` | surfaceDump | `surfaceDumpChunkX/ChunkZ/Size/Out/OutPost` | ✅ 驼峰对齐 |
| `blobProbeChunkX/ChunkZ/Size/Dim` | blobProbe | `blobProbeOut` | ✅ 驼峰对齐 |
| `colProfX/Z` | colProf | `colProfProbe/Mode/R` | ✅ 驼峰对齐 |
| `exec` / `maxinflight` | exec | `execpool` | ✅ 与 §3 裁决定的命名一致（t5-landing §1 表格写的就是 `-Pexec`/`-Pmaxinflight`） |
| `bulkwblog/wbcontent/bulkwbtest/bulkwbsentinel` | bulkwb | 族内零先例（新建） | ✅ 全小写驼峰，无点分/驼峰二义 |

- **#19 家族风险 = 零**：新建的 `-P` 名无一处混用点分风格。

### B3. 语法/重复 ✅ **PASS**（有一处风格瑕疵，见 S-new-1）
- 无重复映射行：全量 `-D` 名去重后**仅 3 个重复**（`cpp.replace` / `cpp.vanilla` / `cpp.worldgen.dir`），**均为补前既有**（1.20.1/1.21.6 双载体或历史既有，非本次引入）；
- 13 行语法均为标准 `if (project.findProperty('x') != null) run.vmArg "-Dy=${project.findProperty('x')}"`，与邻居逐字同构；
- 闭括号配平（196-204 / 217-224 块结构完整）。

### B4. 发射实证 — ⚠️ **部分实锤，非 13/13**（本次审查核心发现 → **M-new-1**）
`t5-postfix-verify.log:31-51` 实测发射 **9 行**（原文逐字，非 Select-String 误判，已用 read 工具直读行区间确认）：

```
:40 VMARG: -Dcoreswap.exec=0              ← 补后发射 ✅
:41 VMARG: -Dcoreswap.bulkwblog=1         ← 补后发射 ✅
:42 VMARG: -Dcoreswap.wbcontent=1         ← 补后发射 ✅
:44 VMARG: -DblobProbe.chunkX=5           ← 补后发射 ✅
:46 VMARG: -Dcolprof.x=3                  ← 补后发射 ✅
:48 VMARG: -Dsurfacedump.dim=minecraft:the_nether ← 补后发射 ✅
:43 VMARG: -Dblob.probe=1                 （族父门，非新补）
:45 VMARG: -Dcolprof.probe=1              （族父门，非新补）
:47 VMARG: -Dsurfacedump.probe=1          （族父门，非新补）
```

**发射面覆盖核**：13 项新映射中 **6 项有直接发射证据**（exec / bulkwblog / wbcontent / blobProbe.chunkX / colprof.x / surfacedump.dim），**7 项无发射证据**（blobProbe.chunkZ / blobProbe.size / blobProbe.dim / colprof.z / coreswap.maxinflight / bulkwbtest / bulkwbsentinel）。

- **根因**：不是缺陷，是**验证覆盖缺口**——`t5-landing.md:31-35` 的验证命令根本没传那 7 个 `-P`（未传的开关自然不会发射）。日志与命令**自洽**。
- **但** `t5-landing.md` §2 标题称「**补前缺席 → 补后发射**」并列 7 行示例，**未声明「这是 13 项中的 6 项抽样」**，读者易误读为 13/13 全量实证。→ 需补覆盖声明（M-new-1）。
- 未覆盖 7 项的**静态充分性**：均为同构单行 `if(...)!=null` 模式、且**经门禁复算证明 ORPHAN 已关闭**（§D），故风险低——但**属外推非实证**，MUST 按 §9.7 声明。

### B5. `BUILD FAILED` 归因 ✅ **PASS（归因正确，证据充分）**
`t5-postfix-verify.log` 的 `BUILD FAILED` 归因为环境问题（Chunky jar 文件锁），**证据链完整，我独立复核通过**：
1. **失败点 = remap 阶段**：`:104-118` 栈顶为 `RuntimeModRemapper.remap(:184)`，`:119-127` `Failed to remap mods!`，`:138` `> Task :runServer FAILED`，`:146` `java.exe finished with non-zero exit value 1`；
2. **晚于 vmArgs 发射**：VMARGS 块在 `:31-51`，失败在 `:104+` —— **发射在前、失败在后，发射证据不受污染** ✅；
3. **首轮同现象**：`vmargs-sentinel.log:127/:141` 首轮同样 `runServer FAILED` / `BUILD FAILED in 13s` —— **补前补后同现象 ⇒ 非本课题引入** ✅；
4. **性质 = 文件锁**：`:105/:120/:128` 均为 `FileSystemException: ...chunky-1.3.146-*.jar: 另一个程序正在使用此文件`，属沙箱/环境锁，非 gradle 编辑引入。
→ 归因**成立**，且 `t5-landing.md:48-53` §3 的三点论证与日志逐条对得上。

---

## 3. C. 门禁脚本效力

### C1. 覆盖面表 ✅ **与 switch-sources.json 一致**
`scripts/check_switch_mapping.py:34-43` 的 `GRADLE_FILES`（2 项）+ `SRC_DIRS`（4 项）与 `switch-sources-postfix.json:3-12` coverage 段**逐项一致**（1.20.1/1.21.6 java build.gradle；1.20.1/1.21.6 java src、java-core、worldgen-core）。

- ⚠️ **但脚本丢失了 `not_scanned` 声明**：`switch-sources.json:13-17` 原有「未覆盖面」三行（native/C++ 侧读 env、外部工具直接 -D 传参、1.21.6 python/JS 辅助脚本），**脚本内无对应字段**（仅 `:21` docstring 一行「未扫描的载体一律不判 DEAD」）。→ **S-new-2**（§9.7 覆盖面声明退化）。

### C2. `--strict` 语义 ✅ **合理**
`:115-118`：`if "--strict" in sys.argv and dead: return 1` —— **仅 DEAD 非零退出，ORPHAN 不阻断**。
- 合理：ORPHAN 多为合法旁路（11 项中 8 项是 SCOPED-豁免项 + `java.io.tmpdir` 标准 JVM 属性 + `coreswap.wbcheck`），若阻断则门禁**永久红灯**，丧失信号价值；
- **与 AGENTS.md §一.13 措辞一致**（AGENTS:115「`--strict` = 存在 DEAD 即非零退出」↔ 脚本 `:14/:115-118`）✅。

### C3. 能否复现 125/4/11 ✅ **可复现（我独立复算得到逐位相同结果）**
按脚本同款正则 + 同款载体面**独立复算**：

| 量 | 脚本/`switch-sources-postfix.json` | 我的独立复算 | 一致 |
|---|---|---|---|
| M1（声明面） | 129 | **129** | ✅ |
| M2（消费面） | 136 | **136** | ✅ |
| CONSISTENT | 125 | **125** | ✅ |
| DEAD | 4 | **4** | ✅ |
| ORPHAN | 11 | **11** | ✅ |

且 **DEAD/ORPHAN 名单逐项相同**：
- DEAD = `biome6oct` / `cpp.noBatch` / `java.util.concurrent.ForkJoinPool.common.parallelism` / `max.bg.threads`；
- ORPHAN = `bench.threads` / `bench.worldgen` / `chunkRandom.seed` / `chunkRandom.seed288` / `colDump.targets` / `coreswap.light.blockabi` / `coreswap.light.oldcollect` / `coreswap.wbcheck` / `height.x` / `height.z` / `java.io.tmpdir`。
- 注：`t4-adjudication.md:86` §3.2 的 SCOPED 列表（11 项）与 json ORPHAN（11 项）**集合相同**（仅列序不同）；`classification.yaml` 的 SCOPED 段亦为同 11 项 —— 三处口径一致，无差异。
- **能力边界重申**：我**没有实跑脚本**；这是「脚本逻辑 == 复算逻辑」的等价复现。若需「脚本实跑通过」的硬证据，须由有 shell 的执行者补跑一次并落盘。

### C4. 脚本可执行性风险 ⚠️ **→ M-new-2**
`scripts/` 被 gitignore（§E 已核实），脚本**不在版本控制内**。门禁的价值 = 可被任何环境执行；当前形态下「门禁在仓库中不存在」。且 `switch-sources-postfix.json` 内的 coverage 是**数据副本**，与脚本常量**双写**——一旦分叉无人对账（建议脚本读 json 或加断言）。

---

## 4. D. 修复效果量化

### D1. ORPHAN 24 → 11 = −13 ✅ **与补映射数 13 逐项吻合，零差异**
用集合运算对 `switch-sources.json` orphan（24 项）与 `switch-sources-postfix.json` orphan（11 项）求差：

**关闭的 13 项（pre − post）**：
```
surfacedump.dim       blobProbe.chunkX    blobProbe.chunkZ    blobProbe.size
blobProbe.dim         colprof.x           colprof.z           coreswap.exec
coreswap.maxinflight  coreswap.bulkwblog  coreswap.wbcontent  coreswap.bulkwbtest
coreswap.bulkwbsentinel
```
**新出现的项（post − pre）**：`（空）` —— **无回归、无新 ORPHAN** ✅

**逐项对 T5 声称的 13 项**：surfaceDump.dim(1) + blobProbe(4) + colProf(2) + exec/maxinflight(2) + bulkwb(4) = **13** ↔ **集合差 13 项，逐名相同** ✅
→ **不等项 = 0**。`t5-landing.md` 未在此处声称数字，但 §5「落地产物清单」口径与实测一致。**D 项无缺陷**。

> 附注：`switch-sources-postfix.json` counts 的 `M1=129`（原 `M1_declared=116`）与 `pprop_names=121` 的口径差异未在文件内说明 —— 属**次要口径瑕疵**（见 S-new-3）。

---

## 5. E. 诚实边界

### E1. `t5-landing.md` §4 未完成清单 ✅ **基本完整，缺 1 项**
列了 5 项：① 死开关未删 ② `max.bg.threads` UNRESOLVED ③ 1.21.6 未同步 ④ 其余 ~6 项未逐个补 ⑤ 端到端 Full 未做。
- ✅ 与 T4 裁决口径一致（DEFECT 13 − 已补 7 类族代表… ④ 的「~6 项」需与 T5 声称的 13 项对齐核算）；
- ⚠️ **缺项**：**「13 项中仅 6 项有发射实证、7 项无发射证据」未列入**（→ M-new-1）。这是**诚实边界里最实质的一处遗漏**。
- ⚠️ ④「其余 ~6 项 DEFECT 未逐个补」与 §1 声称「13 项中的 7 类族代表」**口径未对账**（13 − 补的 7 类 = 6 项未补，数字自洽；但 §1 用「7 类族」、§4 用「~6 项」，读者需自行换算）→ S-new-4。

### E2. `scripts/` gitignore 发现 ✅ **准确（我独立核验属实）**
- `git ls-files scripts/` → **空**（零文件入库）✅
- `git check-ignore -v scripts/check_switch_mapping.py` → `.gitignore:51:scripts/` ✅
- `git show --stat 56dc15c` → **只提交了 `switch-sources-postfix.json` 一个文件**，脚本未入库 ✅
- → `t5-landing.md:70-77` §5.1 的说法（「该目录下全部脚本均未入版本管理」「包括 check_switch_mapping.py + scan_cpp_anchors.py + merge_index.py」「只记录，不擅自改 .gitignore」）**逐条属实**，且**处置姿态正确**（不擅自改）。
- ⚠️ **但**：commit `56dc15c` 的 message 写「land switch reconciliation gate **as repo script** + AGENTS discipline clause 13」——**「as repo script」与实际只提交 json 不符**（脚本未入库）。这是**commit message 与事实不一致**（→ S-new-5，须以取代记录更正，不得改历史）。

### E3. status 纪律 ✅ **PASS**
- 全部产物 `status: draft`：`t4-adjudication.md:4` / `t5-landing.md:4` / `phase25-verification.md:4` 均 draft；`review-001.md:4` = `opinion-only`（合规）；
- **无越权 candidate/confirmed**：b61 目录内 `confirmed` 一词仅出现在 review-001（judge 意见）与 t4-adjudication §6「candidate/confirmed 由后续 judge + 用户定」的**声明句**中，**无任何产物自称 confirmed** ✅；
- 我作为 judge **未修改任何 status**（本文件亦为 opinion-only）。

### E4. AGENTS.md 条款 13 措辞一致性 ⚠️ **3 处需核**（→ S-new-6）
| 项 | AGENTS:115-118 | 实现 | 判定 |
|---|---|---|---|
| 门禁命令名 | `python scripts\check_switch_mapping.py` | `scripts/check_switch_mapping.py` 存在 | ✅ 一致 |
| `--strict` 语义 | 「存在 DEAD 即非零退出」 | `:115-118` 仅 DEAD 阻断 | ✅ 一致 |
| 族内对称性判据 | 「族内 ≥1 `-P` 先例 ⇒ DEFECT；零先例 ⇒ SCOPED」 | `:16-19` docstring 同 | ✅ 一致 |
| **豁免子句** | 「探针专用路径 + 缺省权威**须逐条登记 reason**」——**但未写「与 §2 规则合读」** | `t4-adjudication.md:82` 写明需合读，**脚本 `:19` 仅一行带过** | ⚠️ 条款未传达「合读」要求（M1 的核心内容被压缩） |
| **覆盖面维护** | 「新增载体须同步更新 `GRADLE_FILES`/`SRC_DIRS`」 | 一致 | ✅ 但**脚本内未登记 `not_scanned`**（C1） |
| **可执行性** | 条款把命令写成可跑门禁 | **脚本未入库**（§E2） | ❌ **条款引用了一个「仓库中不存在」的门禁**——与 t5-landing §5.1 自陈的「AGENTS.md 明文引用的门禁在仓库中不存在」**同一致命点，但 AGENTS 条款未加任何注记** |

---

## 6. 审查意见汇总

### MUST（阻断 confirmed）

| 编号 | 问题 | 一句可执行修正 |
|---|---|---|
| **M-new-1** | T5 发射实证只覆盖 13 项中的 6 项，`t5-landing.md` §2 未声明此为抽样，§4 未完成清单亦漏登「7 项无发射证据」 | 在 `t5-landing.md` §2 与 §4 各补一行：「13 项中 6 项有发射实证（exec/bulkwblog/wbcontent/blobProbe.chunkX/colprof.x/surfacedump.dim），7 项为静态外推未实证（blobProbe.chunkZ/size/dim、colprof.z、maxinflight、bulkwbtest、bulkwbsentinel）」，或补跑一轮传齐 13 个 `-P` 的验证并落盘。 |
| **M-new-2** | 门禁脚本未入库（`git ls-files scripts/` 空），AGENTS §一.13 与 t5-landing §5 却把它当可执行门禁引用，且「as repo script」的 commit message 与事实不符 | 二选一并落盘：① 将 AGENTS 明引的 3 个门禁脚本（`check_switch_mapping.py`/`scan_cpp_anchors.py`/`merge_index.py`）`git add -f` 入库；**或** ② 在 AGENTS §一.13 与 t5-landing §5.1 加显式口径「脚本目录为本机资产、不入库」，并以取代记录更正 56dc15c 的 message（不改历史）。 |

### SHOULD

| 编号 | 问题 | 一句可执行修正 |
|---|---|---|
| **S-new-1** | `build.gradle:118` 与 `:147-149` 的注记行**丢失缩进**（ind=0，所在块 ind=12）——Groovy 语义不受影响，但与周围代码视觉错位，且 `:150` 的 `cppNoBatch` 映射行也退到 ind=0 | 把这 4+1 行缩进补齐为 12 空格（与 `:119` 邻居对齐）；`gradle help` 已证明能跑，补后重跑一次确认。 |
| **S-new-2** | 脚本覆盖面表丢了 `switch-sources.json:13-17` 的三条 `not_scanned` 声明，§9.7 覆盖面声明退化 | 在 `check_switch_mapping.py` 的 coverage 输出（`:103`）中增补 `not_scanned` 字段，或把三条并回 `:21` docstring。 |
| **S-new-3** | `switch-sources-postfix.json` counts 的 `M1=129` 与原 `M1_declared=116`/`pprop_names=121` 三种口径并存，文件内无换算说明 | 在 postfix json 的 counts 段加一行口径注（`M1` = vmArg 去重 `-D` 名；`pprop_names` = `findProperty` 名，两者不等价）。 |
| **S-new-4** | `t5-landing.md` §1 用「13 项中的 7 类族代表」、§4 用「其余 ~6 项」，两处口径需读者自行换算 | 统一为「13 项中已补 13 项映射行（归为 5 个族），发射实证 6 项」的单一表述。 |
| **S-new-5** | `56dc15c` message「land ... gate **as repo script**」与「只提交了 json、脚本未入库」不符 | 以取代记录（§15.4 supersedes）在 t5-landing.md 或 10 时间线登记一行更正，**不改历史 commit**。 |
| **S-new-6** | AGENTS §一.13 的豁免子句表述未传达「§2 规则须与 §3.4 合读」，且未注记脚本未入库 | 在条款 13 的「豁免子句」句后加「（判据全文见 t4-adjudication §2+§3.4，**须合读**）」，并按 M-new-2 的结论加脚本入库/本机资产的显式口径。 |
| **S-new-7** | `switches-registry.yaml:5-11` 仍是**补前快照**（`CONSISTENT: 112` / `DEFECT-MISSING-MAPPING: 13`），而实测补后为 CONSISTENT 125 / DEFECT 13 中 13 项已补映射（ORPHAN 关 13） | 重生成或加一行「本表为 T4 补前基线快照，补后状态见 `switch-sources-postfix.json`」，避免读者误当现状。 |

### 通过项（无意见）
- ✅ **A. M1/M2/M3 全部真闭合**（M1 合读已写明 / M2 组内差分 + 首轮 FAILED 如实披露 / M3 纠错到位）
- ✅ **B1 作用域**：13 行全在 `benchVmArgs` 闭包内（:68-253）
- ✅ **B2 命名对齐**：13 项与族内风格一致，#19 家族风险为零
- ✅ **B3 语法/重复**：无重复引入，闭括号配平
- ✅ **B5 FAILED 归因**：remap 阶段、晚于发射、首轮同现象 → 环境问题，归因正确
- ✅ **C2 `--strict` 语义**：仅 DEAD 阻断，与 AGENTS 措辞一致
- ✅ **C3 数字可复现**：129/136/125/4/11 独立复算**逐位相同**，DEAD/ORPHAN 名单逐项相同
- ✅ **D. ORPHAN 24→11 = −13**：与 13 项补映射**逐名对上，零差异、零回归**
- ✅ **E2 scripts gitignore 发现属实**（`git ls-files` 空 + `.gitignore:51` + 56dc15c 仅 json）
- ✅ **E3 status 纪律**：全 draft，无越权 candidate/confirmed

### 关于 status 与 confirmed
- 本审查**只出意见，未修改任何产物 status**；
- 建议：A/B/C/D 项事实层面已实锤，但 **M-new-1（发射覆盖声明）与 M-new-2（门禁脚本入库）未闭合前，不宜建议 candidate**；二者任一闭合后可讨论 candidate 授予，**confirmed 仍须人类拍板**（Anchorlaw §16.1 confirm hook）。
- 本次**未触发** §15.4 PI-1 halt（无 C-gate 三次未满足），**未触发** §9.7 E1/E2/E3 争议（T5 的发射面比对是同构建态单变量，属 **E1**，`S` = `t.jvmArgs` 发射点本体；口径声明已在 `t5-landing.md:8` 给出）——**等价档位建议在 M-new-1 的补充声明中显式写出 `E1` 与 `S`**。

---

## 7. 最终结论

**PASS-with-conditions** —— 三项强制条件（M1/M2/M3）全部真闭合、13 项补映射作用域/命名/语法全对、ORPHAN −13 与补映射数逐项吻合、门禁数字（129/136/125/4/11）经独立复算逐位复现、FAILED 归因证据链完整、status 纪律无越权；**但 T5「补后发射」实证口径只覆盖 13 项中的 6 项却未声明为抽样（M-new-1），且被 AGENTS 条款明引的门禁脚本实际未入库、与 commit message 自述矛盾（M-new-2）——两条 MUST 闭合前不建议授予 candidate，confirmed 留人类拍板。**
