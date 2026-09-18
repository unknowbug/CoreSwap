# B6-2 Phase 2.5/3 记录：T2 落地 + T3 哨兵 + T4 门禁闭环（260918-02）

```yaml
status: draft
block: 260918-02
role: 主会话
layer: Partial（发射面实测——init script doFirst 打印 t.jvmArgs；端到端行为验证不做，与 B6-1 同边界，MUST 声明）
```

## T2 落地（versions/1.21.6/java/build.gradle，+20 行）

12 项 DEFECT 补映射全部写入 `benchVmArgs` 闭包内（#47），命名与 1.20.1 B6-1 修复逐字对齐（#19）：

| 族 | 新增 -P | -D 目标 |
|---|---|---|
| surfaceDump | `surfaceDumpDim` | `surfacedump.dim` |
| blobProbe | `blobProbeChunkX/ChunkZ/Size/Dim` | `blobProbe.*` |
| colProf | `colProfX/colProfZ` | `colprof.x/z` |
| bulkwb | `bulkwblog/wbcontent/bulkwbtest/bulkwbsentinel` | `coreswap.*=1` |
| stallwatch | `stallwatch` | `coreswap.stallwatch=<秒>` |

还原纪律：git 为权威（改动仅此一文件，`git diff --stat` = 20 insertions ✓）；无注入式副作用。

## T3 哨兵（发射面直证，#170）

- **命令**：`gradle :runServer -I .tmp/b61-260917-06/print_vmargs.gradle` + 12 项新 `-P`（附父门 surfaceDump/blobProbe/colProfProbe），1.21.6 树。
- **环境纪律**：GRADLE_USER_HOME 指工作区（#7）+ tmpdir 固化 + `gradle --stop` 先行（#42 家族，成对）。
- **原始日志**：`cmd-output/sentinel-1216-all12.log`（BUILD SUCCESSFUL in 3m 9s，exit 0）。
- **判据**（预登记于 t4-adjudication-1216.md §3）：12 项新 `-D` 全部出现在 `t.jvmArgs` → **12/12 PASS**（grep 计数 12，逐行见日志）。
- **组内标定**（#170）：同轮三个探针族父门 + 子参数同时发射，全部在场——「发射成功」由同轮多行自证，无缺席项。

### 副作用登记（v0.22 §9.8）

| 副作用 | 类型 | 逆/声明 |
|---|---|---|
| `.tmp/b62-260918-02/`（gradle-home / jtmp / raw log） | derived | identity（.tmp 不入库，可整目录删除） |
| 1.21.6 `run\world` 存档生成（哨兵 runServer boot） | in-place | 覆盖性产物，非参照数据（无对比用途）；如需干净态可删 `runtime/1.21.6/java/run/world` 重建 |
| gradle/loom 对 `.fabric/processedMods` 的缓存操作 | in-place | **显式不可逆声明**：gradle/loom 自身行为，非本课题产生（同 B6-1 judge E 项形态） |

## T4 门禁闭环

- `python scripts\check_switch_mapping.py`：**1.21.6 声明 106→118（+12）**；per-version 缺口 **21→9**。
- 剩余 9 项 = T1 定性全部 SCOPED 豁免项（bench.threads/bench.worldgen/chunkRandom.seed/seed288/colDump.targets/coreswap.wbcheck/height.x/height.z/java.io.tmpdir）——**真缺陷归零**（H2 拍板口径），与 `t4-adjudication-1216.md` §2.2 逐名一致。
- **判据措辞修正（judge S1）**：预登记判据 `check` 字面为「per-version 缺口=0」，实际闭环 = 缺口 9（全部 SCOPED，经 H2 批准豁免）。实现的是判据意图（**DEFECT 缺口=0**）；字面未满足属预登记措辞过宽，本行即修正记录，不构成 §15.4 取代（判据意图与闭环一致，无结论被推翻）。
- **judge S3 登记**：1.21.6 的 blobProbe 四行子映射在 `if (blobProbe)` 块外（1.20.1 镜像在块内）——行为等价（各 findProperty 独立 null 守卫），下次触碰该文件时顺手对齐，非缺陷。
- **judge S4 登记**：哨兵退出码为驱动 pwsh job 的 exit 0（平台回执），日志文件本身未内嵌 `$LASTEXITCODE` 行——后续哨兵落盘时附一行退出码记录。
- **judge S2 澄清**：`surfacedump.dim` 消费点行号——`SurfaceDumpProbeMixin.java:46` 为 `System.getProperty` 消费行（定性文档引用正确），`:37` 为同文件 javadoc 提及行（judge 命中处），两者并存非漂移。
- 1.20.1 侧零变化（声明 128 / 缺口 11 全 SCOPED，未动）。

## 附带观察（诚实记录，非本块缺陷）

- 哨兵运行中 `[BLOB-PROBE] stats read failed: Mixin transformation of wg.bench.mixin.BlobProbeMixin failed`——1.21.6 树 BlobProbeMixin 的 mixin 变换失败（#40 家族形态：编译绿 ≠ apply 绿）。**与本块改动无因果**（本块仅加 vmArg 行，配置层；mixin 变换失败在类变换层）——但该探针在 1.21.6 生产路径**本来就不可用**，登记为独立待查项（不阻塞本块：哨兵判据是发射面，非探针功能面）。

## 判据前置集核对（v0.22 §15.1）

- init script 在位 ✓（`.tmp/b61-260917-06/print_vmargs.gradle`，B6-1 已验证工具复用）
- dll 在位 ✓（`target/release/worldgen1216.dll`，2026-09-15 产物，本块未改 Rust 侧）
- 门禁可运行 ✓（exit 0，报告级输出正常）
