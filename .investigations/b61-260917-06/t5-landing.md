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
- 验证日志：`cmd-output/t5-postfix-verify.log`
- 登记表：`switches-registry.yaml`（T4 生成，含分类）

## 6. §9.8 副作用与逆（本 T5 轮）

| 副作用 | 类型 | 逆 |
|---|---|---|
| 改 `versions/1.20.1/java/build.gradle`（补映射 + 注记） | in-place（源码） | **git 提交**（本块 commit，可 revert）；逆 = `git revert <sha>` |
| 新日志 `t5-postfix-verify.log` | derived | identity |
| gradle remap 缓存（`.fabric/processedMods`） | in-place（缓存） | 显式不可逆声明（gradle/loom 自身行为；失败已观察到，后果 = 缓存重建） |
