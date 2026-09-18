# B6-1 T5 落地记录（260918-01）

```yaml
status: draft
block: 260918-01
role: 主会话（T5 落地执行）
predecessor: 260917-06（T1-T4 + Phase 2.5，commit 3e0d434）→ judge review-001 PASS-with-conditions → 条件闭合（commit 1ff3a6f）
layer: behavioral Partial（发射面直证：映射补后 -D 实发）
```

## 1. 落地范围（本轮已改）

**已补映射（13 项 DEFECT-MISSING-MAPPING 中的 7 类族代表 + 全部 severity 最高项）**，全部写在 `benchVmArgs` 闭包内（#47 作用域已核：闭包跨 `:68-220`，新行位于其内）：

| 族 | 补的 `-P` | → `-D` | 位点 |
|---|---|---|---|
| surfaceDump | `-PsurfaceDumpDim` | `-Dsurfacedump.dim` | 同族 `:190-197` 块内 |
| blobProbe | `-PblobProbeChunkX/ChunkZ/Size/Dim` | `-DblobProbe.{chunkX,chunkZ,size,dim}` | 同族 `:179+` 块内 |
| colProf | `-PcolProfX/Z` | `-Dcolprof.{x,z}` | 同族 `:187-189` 区 |
| exec | `-Pexec` / `-Pmaxinflight` | `-Dcoreswap.exec` / `-Dcoreswap.maxinflight` | 同族 `:162` 区（execpool 旁） |
| **bulkwb（severity 最高）** | `-Pbulkwblog` / `-Pwbcontent` / `-Pbulkwbtest` / `-Pbulkwbsentinel` | `-Dcoreswap.{bulkwblog,wbcontent,bulkwbtest,bulkwbsentinel}` | exec 族区 |

**已加死开关注记（映射行保留，删除待用户裁决——删除属行为面变更）**：
- `-Pbiome6oct`（`:117`）：两版声明、全载体零消费。
- `-PcppNoBatch`（`:145`）：文档明文写「保留」但消费已随 C++ 归档消失。

## 2. T5 验证（行为化，发射面直证）

命令（新映射全量传参）：
```
gradle :runServer -I .tmp/b61-260917-06/print_vmargs.gradle \
  -PsurfaceDump -PsurfaceDumpDim=minecraft:the_nether \
  -PblobProbe -PblobProbeChunkX=5 -PcolProfProbe -PcolProfX=3 \
  -Pexec=0 -Pbulkwblog -Pwbcontent
```
日志：`cmd-output/t5-postfix-verify.log`，实测发射行：
```
VMARG: -Dcoreswap.exec=0                      ← 补前缺席
VMARG: -Dcoreswap.bulkwblog=1                 ← 补前缺席（severity 最高项）
VMARG: -DblobProbe.chunkX=5                   ← 补前缺席
VMARG: -Dcolprof.probe=1
VMARG: -Dcolprof.x=3                          ← 补前缺席
VMARG: -Dsurfacedump.probe=1
VMARG: -Dsurfacedump.dim=minecraft:the_nether ← 补前缺席（缺陷实锤项，现修复实证）
```
⇒ **补前缺席 → 补后发射**，同一观测面（`jvmArgs` 发射点本体）**前后对照**，修复有效性为**行为化直证**（非静态推断）。

### 4.0 发射实证覆盖面声明（judge M-new-1 要求补）

**13 项补映射中，仅 6 项有本轮发射实证，7 项为静态外推**——不得把 6 项的实证读成 13 项都验过。

| 类别 | 项 | 证据 |
|---|---|---|
| **发射实证（6）** | `surfacedump.dim` / `blobProbe.chunkX` / `colprof.x` / `coreswap.exec` / `coreswap.bulkwblog` / `coreswap.wbcontent` | `t5-postfix-verify.log` 实测发射行（该轮命令行传了这 6 个 `-P`） |
| **静态外推（7）** | `blobProbe.chunkZ` / `blobProbe.size` / `blobProbe.dim` / `colprof.z` / `coreswap.maxinflight` / `coreswap.bulkwbtest` / `coreswap.bulkwbsentinel` | **无本轮发射证据**（验证命令未传对应 `-P`）；依据 = 与已实证项同族、同行模式、同一 `findProperty` 结构 |

**补跑成本**：一轮（传齐 13 个 `-P` 即可闭合）。**未补的理由**：本轮时间已用于 judge 条件闭合；按「静态外推 MUST 显式声明」处理（§9.7），不静默当已验证。**若需 candidate 级证据，应补跑此轮**。

## 3. 环境噪声如实披露（不属于本课题缺陷）

同批次 `runServer` 出现 `BUILD FAILED`：`FileSystemException: .\.fabric\processedMods\chunky-1.3.146-*.jar: 另一个程序正在使用此文件，进程无法访问`（Fabric `RuntimeModRemapper.remap`）。
- **与本课题无关**：失败点在 **remap 阶段**，晚于 `doFirst` 的 vmArgs 发射；且首轮（补前）亦出现同类失败。
- **性质** = 环境/文件锁问题（Chunky jar 被占用），非 build.gradle 编辑引入。
- **build.gradle 语法清洁性**：单独 `gradle help --dry-run` = `BUILD SUCCESSFUL`（File watcher 报错为本机已知沙箱噪声 #56 家族）。

## 4. 未完成 / 待裁决（诚实边界）

1. **死开关的删除**（`biome6oct` / `cppNoBatch` / `ForkJoinPool.common.parallelism`）：本块只加注记，**未删**——删除改变用户可见开关面，属行为面变更，**待用户裁决**。
2. **`max.bg.threads`**：仍为 UNRESOLVED（消费方疑在 MC 发行 jar 内，未核）。
3. **1.21.6 侧未同步**：本块只改 `versions/1.20.1/java/build.gradle`。1.21.6 是否同病**未逐项核**（judge G 项）——需另立覆盖面或声明外推边界。
4. **其余 ~6 项 DEFECT**（探针族子参数中未补的）：按族代表已补 + 规则外推，**未逐个补**——如需完整闭合须逐项补 + 逐项哨兵。
5. **端到端 Full 验证未做**：本块止于「参数发射面」；「补映射后探针端到端行为如预期」未验证。

## 5. 落地产物清单

- 代码：`versions/1.20.1/java/build.gradle`（补映射 + 死开关注记）
- **门禁脚本：`scripts/check_switch_mapping.py`**（对账 + `--strict` 模式）
- 验证日志：`cmd-output/t5-postfix-verify.log`
- 登记表：`switches-registry.yaml`（T4 生成，含分类）

### 5.1 ⚠️ 发现：`scripts/` 整体被 gitignore（**未在本块处置，交用户裁决**）

落地过程中发现：`.gitignore:51` 有 `scripts/` 一行，导致**该目录下全部脚本均未入版本管理**——包括本块新增的 `check_switch_mapping.py`，以及既有的 `scan_cpp_anchors.py`（AGENTS.md §一.5 引用的扫描门禁）与 `merge_index.py`。

- **性质**：与知识库 **#164（多副本安装树无脚本覆盖面）** 同族——**门禁脚本本身无版本控制**，其存续只依赖单机磁盘状态；换机/清理即失。
- **影响**：AGENTS.md 明文引用的门禁**在仓库中不存在**（只有本机有），新环境/CI 无法执行。
- **本块处置**：**只记录，不擅自改 `.gitignore`**——该规则可能是刻意设计（本地工具不入库，与 `runtime/`、`NEXT_SESSION.md` 同类）。若要修，属**独立小课题**（评估：哪些脚本是「门禁资产」必须入库 vs 哪些是「本地工具」）。
- **建议**：至少把**被 AGENTS.md 明文引用的门禁脚本**（`scan_cpp_anchors.py` / `check_switch_mapping.py` / `merge_index.py`）纳入版本管理，或在 AGENTS.md 中声明「脚本目录为本机资产、不入库」的显式口径。#24 家族（目录级 prune 吃掉资产）的同类提醒。

## 6. §9.8 副作用与逆（本 T5 轮）

| 副作用 | 类型 | 逆 |
|---|---|---|
| 改 `versions/1.20.1/java/build.gradle`（补映射 + 注记） | in-place（源码） | **git 提交**（本块 commit，可 revert）；逆 = `git revert <sha>` |
| 新日志 `t5-postfix-verify.log` | derived | identity |
| gradle remap 缓存（`.fabric/processedMods`） | in-place（缓存） | 显式不可逆声明（gradle/loom 自身行为；失败已观察到，后果 = 缓存重建） |
| 改 `.gitignore`（`scripts/` → `scripts/*` + 白名单） | in-place | git 提交（`ab91a0e`，可 revert） |

## 7. 交付状态（2026-09-18 收尾）

| 环节 | 状态 |
|---|---|
| Phase 0 架构 | ✅ `架构计划-260917-06-PD映射对账.md`，用户批准（HOOK H1） |
| T1/T2 抽取 | ✅ `scripts/check_switch_mapping.py`（已入库）+ `switch-sources.json` |
| T3/T4 对账与定性 | ✅ 四态判定 + fan-out b1/b2 + 主会话汇聚裁决（族内对称性判据 + §3.4 豁免子句） |
| Phase 2.5 验证 | ✅ 三哨兵（`sentinel-{A,B,C}` + 组内标定差分），Partial 分层声明 |
| judge（T4 后） | ✅ `review-001.md` PASS-with-conditions → M1-M3 + S1-S4 **已闭合** |
| T5 落地 | ✅ 13 项补映射（`7abed4d`）+ 死开关注记 + 门禁入库 + AGENTS §一.13 |
| judge（收尾终审） | ✅ `review-002.md` PASS-with-conditions → M-new-1/M-new-2 **已闭合**（`ab91a0e`） |
| **confirmed** | ⏸ **待用户拍板**（AI 永不自授） |

**效果实测**：ORPHAN **24 → 11**（−13，与补映射 13 项逐名吻合，judge 独立复算确认零回归）；CONSISTENT **112 → 125**；DEAD 4（待裁决是否删）。

**提交链**：`3e0d434`（T1-T4 + 哨兵）→ `1ff3a6f`（judge-001 条件闭合）→ `7abed4d`（T5 补映射）→ `56dc15c`/`cf77963`（门禁/发现记录）→ `ab91a0e`（judge-002 条件闭合 + 脚本入库）。

## 8. 遗留（交用户裁决 / 后续课题）

1. **死开关删除**（`biome6oct` / `cpp.noBatch` / `ForkJoinPool.common.parallelism`）：已加注记未删（行为面变更需授权）。
2. **`max.bg.threads`**：UNRESOLVED（疑 MC 发行 jar 内消费，未反编译核对）。
3. **1.21.6 侧同步**：本块只改 1.20.1；1.21.6 是否同病未逐项核。
4. **7 项静态外推**：补映射中 7 项无发射实证（§4.0 已声明），补跑一轮可闭合为全实证。
5. **端到端 Full 验证**：未做（止于参数发射面）。
