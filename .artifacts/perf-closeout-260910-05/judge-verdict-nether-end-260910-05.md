# judge verdict — 260910-05 · P4：nether / end 分支 R3 同款异步化 + 各自行为门

> 审查角色：core-judge（subagent 隔离执行、**只读、无 shell**）。审查节点：**P4 结论的 confirmed 前审查**（被审产物自标 candidate）。
> 审查对象：`.artifacts/perf-closeout-260910-05/verdict-nether-end-260910-05.md`(candidate)；`runtime/1.21.6/java/.../NoiseChunkGeneratorMixin.java`、`CppBridge.java`（+`java-snapshot/pre|post` 4 文件与 `SHA256SUMS.txt`）；`.investigations/perf-closeout-260910-05/{record.md, patch-nether-end-async.md}`；原始证据 `.tmp/perf-reg-260910-05/{results.txt, logs/*}`、`cmd-output/{diff-N_*,diff-E_*, run_arms3.ps1, diff_dims_*, MANIFEST-sha256.txt}`、`region-*/`（27 组 ×16 mca）。
> **范围**：仅 P4（J3 代码同构性 / J4 `[WG-FILL]` 门控 / J4 行为门 / 性能读数口径 / 边界诚实性 / 状态纪律）。C7 收口已由另一 judge 审毕（`judge-verdict-c7-260910-05.md`，PASS-with-conditions），本文件只在交叠处引用（§七），**不重审 C7**。
> **本文件只出审查意见**：不改任何被审文件、不改 status、不动 `index.yaml`（该文件已有本件的 draft 条目）。confirmed 只能由人类授予。
> 方法：无 shell ⇒ 全部结论由**逐行回读一手文件**获得；计数类事实用「grep 命中总数」自算（下文标注「行计数」）；不能自算的一律进 §六 未核项。

## 判定：**PASS-with-conditions**

主干成立：**P4 的代码改动与 overworld R3 分支逐要素同构、无遗漏、无越界改动**（§一.1，高置信）；**行为门 6 对数字逐格可核到一手 diff 文件**（§一.3，高置信）；**`inflight 1→23` 是有效仪器读数的原位证据**（§一.5，高置信）；end 的「0 差」经我独立检验**不是假阴性**（§一.4，高置信）；状态纪律干净（candidate、无自授 confirmed、index 契约齐备）。
但存在 **10 项必改**，全部是**静态可修**的口径/证据范围问题，其中 4 项实质性：**①（C1）nether「自身噪声带」实为 async 同形态对作代理（该维 sync 无 run-to-run 对）；②（C2）end 的跨实现差 7 块高于同形态噪声 0，被与「零差」并列读成同性质，并外推出「该维不存在 run 级非确定」；③（C4）3.3-3.5×/2.7× 取的是 Chunky 进程内任务计时，wallgen 口径下只有 2.34×/2.00×，「端到端」措辞不成立；④（C5）CPU 边界与根因：唯一 CPU 读数（86 CPU·s/6s 任务）指向「更多总 CPU 换更低延迟」，而 §5.4 的「脚本已修 ⇒ E2 已解决」与现版脚本实况矛盾（详见 C5，含与 C7-C9② 的交叠）。**
⇒ **PASS-with-conditions**（C1–C10 应用后可授予 confirmed），不判 FAIL。

---

## 一、逐条回答（对应任务书 7 问）

### 1. J3 代码同构性（work supplier / 顺序 / 异常 / 计时配对 / 开关 / 池）+ 有无遗漏 + overworld 未被动 —— **逐要素同构，无遗漏，无越界**（置信度：**高**）

逐行比对 `pre/NoiseChunkGeneratorMixin.java` ↔ `post/` ↔ `runtime/.../NoiseChunkGeneratorMixin.java`（post 与 runtime 逐行一致：`MIXLOG:51`、nether `:154-189`、end `:191-224` 行号与内容均相同）：

| 要素 | overworld（pre=post 同文本 108-151） | nether（post 154-189） | end（post 191-224） |
|---|---|---|---|
| `final Chunk target` / `final StructureAccessor structures`（防捕获错误） | ✅ 110-111 | ✅ 158-159 | ✅ 194-195 |
| `Supplier<Chunk> work` | ✅ 119 | ✅ 160 | ✅ 196 |
| 计时入口 `tt0` + `enter(tt0)` + `inflightEnter()` | ✅ 120-122 | ✅ 161-163 | ✅ 197-199 |
| `feedBeardifier*` → `addBeard` → `fillChunk*` 顺序 | ✅ 126-129 | ✅ 165-168（`feedBeardifierNether`/`fillChunkNether`） | ✅ 201-204（`…End`） |
| print + `printStackTrace()` + `throw t`（不吞） | ✅ 131-136 | ✅ 170-174（标签 `(nether)`） | ✅ 206-210（标签 `(end)`） |
| `finally` 内 `if (ChunkTiming.ON){ exit(tx, tx-tt0); inflightExit(); }` | ✅ 137-143 | ✅ 175-181 | ✅ 211-217 |
| 共用 `SYNCFILL`（`-Dcoreswap.syncfill`，`:61`） | ✅ 145-149 | ✅ 183-187 | ✅ 219-223 |
| `supplyAsync(work, WG_FILL_POOL)`（`Util.getMainWorkerExecutor().named("coreswap_fill_noise")`，`:57-58`） | ✅ 148 | ✅ 186 | ✅ 222 |
| 分支门控 | `overworldShape && id==overworld` | `zeroShape && netherActive() && id==nether` | `endShape(…) && id==end` |

- **overworld 分支未被改动**：pre `:108-151` 与 post `:108-151` 逐字相同（含 `MIXLOG` 行、注释、`SYNCFILL` 分支）——**核到**。
- **句柄/缓冲未串味**：`feedBeardifierNether/End` 分别传 `netherHandle`/`endHandle`（post `CppBridge.java:259-261 / 273-275`）；`fillChunkNether` 用 `BUF_NETHER`+`writeChunk(…,256)`（`:416-417 / 437`）、`fillChunkEnd` 用 `BUF_END`+`writeChunk(…,128)`（`:460-461 / 481`）；两缓冲均为 `ThreadLocal`（23 并发下必需的形态）——**核到**。
- **无遗漏**：nether/end 未出现「未接 inflight 计数 / 异常被吞 / 开关不一致」——逐要素表已覆盖。仅存的吞异常点（`CppBridge.fillChunk*` 内 `catch → print + return`）是 **pre-existing**，且 `patch-nether-end-async.md:69` 已显式登记为「本块不扩散」——**核到**。
- **只改了应改的地方**：本块 Java 差异 = mixin 的 nether/end 两分支 + `CppBridge` 的 `MIXLOG`（`post:31-35`）；`ChunkTiming.java`/`build.gradle` pre/post **同 sha**（`SHA256SUMS.txt` 两文件 sha 逐字相同）——**核到**。
- 未核（无 shell）：mixin 69 行 / CppBridge 10 行的精确 diff 行数我未复算（不依赖该数字）。

### 2. `[WG-FILL]` 日志门控：理由是否同 R1、A/B 是否对称、sanity 单独开是否有据、是否毁掉证据链 —— **理由同族、A/B 对称、sanity 有据；但门控只到日志层，且 end 侧没有对应 sanity 臂（C3/C8）**（置信度：**高**）

- **理由与 R1 一致**：新增 `MIXLOG`（post `CppBridge.java:31-35`）与 mixin R1 用**同一属性名** `coreswap.mixlog`；`-Pmixlog=1 → -Dcoreswap.mixlog=1`（`build.gradle:130`）⇒ 单一开关同时开闭 mixin 与 bridge 两处 —— **核到**。overworld `fillChunk` 确无此行（post `:379-413` 无 `[WG-FILL]`）——**核到**（「拉平口径」的说法成立）。
- **A/B 对称**：`results.txt` 全 13 行的 `args=` 里，只有 `naS-r1` 含 `-Pmixlog=1`；**就地验证**——`na-r1.log` / `ea-r1.log` 用 grep 查 `WG-FILL|intercepted|buildSurface` = **0 命中**（门控确为关）——**核到**。
- **sanity 单独开有据**：`naS-r1.log` 行计数 **`[WG-FILL] chunk` = 4761**、**`populateNoise(nether) intercepted` = 4761**、**`buildSurface skipped` = 4761**（三者相同 ⇒ 每 chunk 一次接管 + 一次写回自证 + 一次 SURFACE cancel 都在位）；同臂 `results.txt:5` `interceptDim=4761 mixinLines=4761`（后者实为 buildSurface 计数，因 nether 行带 `(nether)` 不匹配 `[Mixin] populateNoise intercepted` 字面）——**核到**。
- **但门控只 wrapping 了 println，不是工作层**（post `CppBridge.java:441-453`（nether）/ `485-499`（end）：`nzBuf` 全缓冲扫描（65536/32768 int）+ 16 点 `getBlockState` 仍**每 chunk 无条件执行**，只有 `println` 在 `if (MIXLOG)` 内）⇒ 「R1 同族拉平」止于日志；overworld `fillChunk` 是**连这块计算都没有**。verdict §1 第 2 行字面（「读回自证**行**改为门控」）没错，但读者易读成「该诊断已消除」——→ **C8(i)**（量级未测，估计 <1% 单块时长，我不主张改代码，只要求写清）。
- **是否毁掉证据链**：A/B 臂现在**没有任何 per-chunk 接管/写回直证**（`interceptDim=0`，record §5.3 已解释）。nether 由 `naS` 覆盖 ✅；**end 侧没有**——驱动脚本 `run_arms3.ps1:32` **定义了 `eaS`（end async + `-Pmixlog=1`）但全块不存在 eaS 的 log/results 记录**（`logs/` 目录与 `results.txt` 均无）⇒ 全块没有任何一行 `populateNoise(end) intercepted` 或 end 的 `[WG-FILL]`——→ **C3**。
- 附带（C8(ii)）：4761 是**调用次数**，不是 chunk 数（Chunky 报 `Processed: 4225 chunks`；`na-r1.log:144`）⇒ 约 +13% 重复接管调用，与 `[CHUNKTIME] n=4608` 超计同族，不得读作 chunk 数。

### 3. 行为门数字可核对性（逐行核到原始文件） —— **6 行全部逐格核到，无矛盾；仅 1 处措辞级数字错误（C6）**（置信度：**高**）

| verdict 行 | 一手出处 | `common=/blocks=/diff=(%)/sections` | 判定 |
|---|---|---|---|
| §3.1 `ns × nv` | `diff-N_sync_vs_vanilla.txt:74` | `common=7542` / `209,965,056` / `214,240` (0.1020%) / 50555-706 | ✅ 逐格一致 |
| §3.1 `na-r1 × ns` | `diff-N_async_vs_sync.txt:74` | `common=7542`（:1-2 两侧 7543/7542 ⇒ 1 chunk 单侧，见 N2）/ `209,981,440` / `255,942` (0.1219%) / 50508-757 | ✅ |
| §3.1 `na-r1 × na-r2` | `diff-N_async_vs_async2.txt:74` | `common=7543` / `209,981,440` / `295,191` (0.1406%) / 50436-829 | ✅ |
| §3.2 `es × ev` | `diff-E_sync_vs_vanilla.txt:74` | `common=7567` / `494,010,368` / `7` (0.0000%) / 120601-7 | ✅ |
| §3.2 `ea-r1 × es` | `diff-E_async_vs_sync.txt:74` | `common=7567` / `494,055,424` / `0` / 120619-0 | ✅ |
| §3.2 `ea-r1 × ea-r2` | `diff-E_async_vs_async2.txt:74` | `common=7567` / `494,100,480` / `0` / 120630-0 | ✅ |
| §4 表 8 行（sec/wallgen/inflightMax/world） | `results.txt:1-13` | nv 56/60/–、ns 69/70.1/1、na-r1 21/30.0/23、na-r2 20/30.0/23、ev 5/10/–、es 18/20.0/1、ea-r1 6/10.0/23、ea-r2 6/10.0/23 | ✅ 全对 |
| §2 sanity（naS） | `results.txt:5` + `naS-r1.log` 行计数 | `Total time: 0:00:21`、`Processed: 4225`、intercepted=WG-FILL=**4761** | ✅（4761 语义见 C8(ii)） |
| §1「全部 9 臂 `dllsha=abd7d8893d22e030`」 | `results.txt:6/10` | nv-r1/ev-r1 = **vanilla 臂，`dllsha=-`** | ❌ **不成立** → **C6** |
| §2 载体/维度/region 路径 | `run_arms3.ps1:24-32` + MANIFEST `region-naS-r1/*` 16 mca | nether `DIM-1\region`、end `DIM1\region`，每臂 16 mca | ✅ |

- **核不到/未核**：`Chunky radius 500`（脚本内无 `chunky radius` 命令，`:75` 只有 quiet/world/center/start；4225=65²⇔半径 32 chunk）——数值口径来源未核（N1）；`[WG-FILL]`=4761 在 `results.txt` 中**没有对应列**，我用日志**行计数**独立核到，属「可核但非脚本产出字段」。

### 4. 判据是否预登记、事后是否偷换；end「0 差」的假阴性风险 —— **判据已预登记且未被偷换；end 的 0 是完成比对后的零，非假阴性；但 7 块与 0 的性质差异被抹平（C2），且 end 侧无 sanity 直证（C3）**（置信度：**高**）

- **预登记核到**：`record.md:126-129`（§5.4，标注「维度对拍（运行中）」，早于 6 份 diff 产出）登记了两条规则——**接管基线**（sync×vanilla，差异大 ⇒ 归既存接管分歧，不记异步账面）与**形态等价门**（async×sync，判据 = ≤ 同维度同形态噪声 async1×async2）。verdict §3.1/§3.2 用的**正是**这两条，**未偷换**（无事后引入的替代判据）——**核到**。
- **end「0 差」不是假阴性（我独立检验，反而应作为支持性证据写入）**：
  ① **同一工具/同一口径检出过非零**：`es×ev` 在 494,010,368 块的比对里检出 **7 块**（`diff-E_sync_vs_vanilla.txt:74-77`：air→end_stone 5、end_stone→air 2）⇒ 该比对链对 ~1.4e-8 级差异敏感；
  ② **比对真跑完且无单侧排除**：`common=7567`，两侧 `vanilla chunks=7567 / coreswap chunks=7567`（`:1-2`）；`blocks` 494,055,424 / 494,100,480（均 >0）；`sections same/diff` 120619/0 与 120630/0；
  ③ **分母非空**：`diff_arms.py:140-162` 的 `blocks = 4096 × 两侧 section 并集`（≥120,619 个 section 对），且比较循环无 try/except ⇒ 解码失败会崩而非静默 0；
  ④ **输入确为不同数据**：`MANIFEST-sha256.txt` 中 `region-es-r1 / region-ea-r1 / region-ea-r2` 的**同名 mca sha 互异**（如 `r.0.0.mca`：94081c21… / 801b4325… / cce3a994…；值我未复算，仅读清单）⇒ 排除「同一目录传两次」。
  ⇒ 结论：end 的 0 是**真零**。verdict §3.2 目前只用「位级一致」断言，未给这套「灵敏度证明」——→ **C9**。
- **但 7 与 0 的性质被并列**：`es×ev` 的 7 块 **高于** 同形态噪声（0）⇒ 是可检出的跨实现差（极小但非零），verdict 却把它与「跨形态 0 ≤ 同形态 0」并列为同一类「零差/一致」，并进一步写「端到端确定性（同配置两 run 零差）⇒ **该维不存在 run 级非确定**」。后一句的 0 对全部是 **coreswap×coreswap**（ea-r1×ea-r2、ea×es），而 `ev`（vanilla end）**只有 1 个 run** ⇒ 无法把 7 块差归因于「实现差」还是「vanilla 侧 run 噪声」——→ **C2**。

### 5. 性能读数口径：`inflight 1→23` 是否有效证据；3.0-3.5× / 2.7× / 持平是否有据；CPU 缺失是否如实声明 —— **instrument 有效且原位可核；比值有据但口径必须点明（C4）；CPU 声明存在但解读缺位、且根因叙述有误（C5）**（置信度：**高**）

- **`inflight max` 是有效仪器**（非「任意并发数」）：`ChunkTiming.java:31-40` 的 `inflightEnter/Exit` **都** `if (!ON) return;`，`MAX_INFLIGHT` 只在 mixin 的 `work` 区间内被改；全部 coreswap 臂 `args` 均含 `-Pchunktime=1`（`results.txt`），vanilla 臂 `inflightMax=-`（未开）⇒ 读数有效。**原位核到**：`ns-r1.log:107` `inflight max=1`；`na-r2.log:106` `inflight max=23`；`es-r1.log:107` `=1`；`ea-r1.log:107` / `ea-r2.log:107` `=23`（与 `results.txt` 列一致）。
- **比值有据**：ns 69 → na 21/20 = **3.29/3.45×**；es 18 → ea 6/6 = **3.0×**；nv 56 → na 21/20 = **2.67/2.80×**；ev 5 vs ea 6 = 同量级 ✅（`results.txt` + 臂内 `Task finished … Total time`：`ns-r1.log:148`=0:01:09、`na-r2.log:142`=0:00:20、`es-r1.log:143`=0:00:18、`ea-r1.log:142`/`ea-r2.log:142`=0:00:06）。
  **但**：3.3-3.5×/2.7× 用的 `sec` 是 **Chunky 进程内任务计时**；`wallgen`（进程级）为 ns 70.1→na 30.0 = **2.34×**、nv 60.0→na 30.0 = **2.00×**，且 `wallgen−sec` 尾巴在 **+1.1s(ns) 到 +10s(na-r2)** 间不一致 ⇒ 两种口径给出**明显不同**的「相对 vanilla」倍数，verdict §4 只报前者且未标口径——→ **C4**。
- **CPU**：`results.txt:13`（`ea-r2`）`serverCpu=86 cores=8.6`（cores=`cpuSec/wallgen`，`run_arms3.ps1:113`）是全块唯一 CPU 读数；verdict §5.4 如实写了「仅 ea-r2 采到」✅。**但缺解读**：6s 任务消耗 86 CPU·s ⇒ async 形态是「**更多总 CPU 换更低延迟**」，故「反超 vanilla 2.7× / 持平」是**延迟**结论而非效率结论；nether 侧同口径缺失。→ **C5**。
- **额外客观读数（verdict 未用，建议登记 N1）**：单 chunk mixin 时长在 async 下显著膨胀（nether `ns-r1.log:107/147` ≈10.1→11.4ms vs `na-r2.log:106/141` ≈36.8→25.0ms；end `es-r1.log:107/142` ≈3.5ms vs `ea-r2.log:107/141` ≈28.7→20.1ms），而 `sumMixin/wall` 平均并发度仅 ≈**5.75**（nether 115.0s/20s）/≈**15.4**（end 92.6s/6s），远低于峰值 23 ⇒ 「3.0-3.5×」不是「23× 线性扩展」的证据。

### 6. 边界诚实性（§5 的 7 条是否充分） —— **方向诚实、7 条基本到位；但有 3 处需收紧（C1/C2/C4/C5），1 条「首次验证」属实**（置信度：**中高**）

- **§5.2 的措辞恰当**：`nether 0.1020% 落在自身噪声带内 ⇒ 不能读成「接管已对齐 vanilla」（只能说「差异未超出 run 级噪声」）`——**表述正确且是必要的防误读**；比「接管分歧」更保守，无更危险的误读空间。仅「**自身**噪声带」的定语不准（见 C1）。
- **「1.21.6 nether/end 接管属首次验证」——属实（抽样核到）**：`nether-save-full/judge-review.md:4` 引 `runtime/1.20.1/java/.../ReadWorldProbe.java`、`candidate.b1-timing.md:8` 述 1.20.1 chunk status 管线；`end-takeover/01-rust-status.md:22` 以 1.20.1 的 `DensityFunctions.EndIslandDensityFunction` 为参照 ⇒ 两处载体均为 **1.20.1**（我未逐份读完两目录全部 20 个文件，属抽样）。
- **应补而缺的边界**：① wallgen/端到端口径（C4）；② 总 CPU（C5）；③ end 无 sanity 臂（C3）；④ end 的 7 块 vs 0 差性质（C2）；⑤ 单对点估计/检验力低的内联限定（C7）；⑥ `[WG-FILL]` 门控只到日志层（C8）。
- **§5.7（`pending_cross_writes` 不可达对三维一并适用）**：与 C7 verdict 的空集判定一致，`flags=3` 在 13 行 `results.txt` 的 `conf=` 字段逐臂可核（含 nv/ev 为 `conf=-`）——**核到**，无异议。

### 7. 状态纪律与措辞 —— **合规；无自授 confirmed、无禁用措辞；「位级零差/位级一致」需内联限定（并入 C2）**（置信度：**高**）

- 自授 confirmed：**无**（verdict §3 header + §6 均写 candidate、confirmed 留 HOOK-3；index 条目 status=candidate、`index-entry.yaml:13-16` 同）——**核到**。
- 禁用措辞：全文无「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响」的**断言式**用法；`inflight max` 只报 1/23 整数读数——**核到**。
- 「end 位级零差」= 实测值（494,100,480 块比对得 0 差），**不属禁用措辞**；但 §6 收尾句「end 位级零差」有被读成「全域全维度绝对无内容影响」的空间 ⇒ 要求内联限定为「在比对集（common=7567 chunks、两侧 section 并集 494,100,480 块）上」并去掉 §3.2 的跨臂外推（并入 **C2(iii)**）。

---

## 二、三源核对（spec §4）

| 源 | 可核性 | 结论 |
|---|---|---|
| ① 交付快照 `.artifacts/perf-closeout-260910-05/` | 可核 | `verdict-nether-end`(candidate) + `index-entry.yaml`(4 条) + 根 `index.yaml:1210-1220` 条目**齐备**（C7-C7 契约缺口已被 C7 侧闭合，本件无缺口）。但根索引**摘要注释**（`:1214-1218`）复述了同一批需修正的表述（3.3-3.5×/2.7×/「端到端确定性」/「自身 run 级噪声」）⇒ 与 verdict 同批修正（**C10**）。 |
| ② git HEAD + 工作区 diff | **不可自核（无 shell）** | 继承任务书 `HEAD=42d46e7`、`runtime/`/`.tmp/`/`target/` gitignore。替代载体 `java-snapshot/pre|post/SHA256SUMS.txt`：`build.gradle`(09c8c8a1…) 与 `ChunkTiming.java`(e60e757f…) pre/post **同 sha**；`CppBridge.java`(4cda71a9…→2cd61e0f…)、`NoiseChunkGeneratorMixin.java`(5d22dc66…→aa5522c2…) 变 ⇒ 在途改动面与 verdict §1 声明一致（sha 值我未复算，仅作「哪些文件变了」的证据）。**runtime 现版 = post 快照**（我逐行比对 `MIXLOG`/nether/end 行号一致）。 |
| ③ 验证记录 | 可核，含 1 处矛盾 | 6 份 diff 逐格核到（§一.3）；`results.txt` 13 行核到；臂内日志原位核到 `initNether/initEnd`、`inflight max`、`Task finished`、计数；**矛盾 1 处** = §1「全部 9 臂 dllsha」 vs `results.txt:6/10`（C6）。**脚本实况与 record §2.2 的 E2 叙述矛盾**（C5）。 |

**执行体一致性旁证**：11 个 coreswap 臂（含 `r3feat-r2`）`dllsha=abd7d8893d22e030` 与 260910-04 一致；`patch-nether-end-async.md:68` 明记「Rust 侧零改动」⇒ P4 与 04 同 Rust 执行体（Java 侧为 P4 后版本）✓。

---

## 三、必改条件（C1..C10，逐条可执行，均为静态修正）

- **C1（MUST）nether「自身噪声带」的来源必须写准。** 该维仅有 1 个 sync run（`ns-r1`），**不存在 sync 的 run-to-run 对**；0.1406% 出自 **async 同形态对**（`na-r1×na-r2`，`diff-N_async_vs_async2.txt:74`）。§3.1 附带读数与 §5.2 须改为：「噪声带 = async 同形态对（0.1406%）代理；nether sync 的 run 级噪声本块未测（若要沿用代理，须引 overworld 同配置读数 0.0086%/0.0092% 作依据，且同样只有 1 对/形态）」。
- **C2（MUST）end 的 7 块与 0 差不得并列为同性质；确定性声明必须限定范围。** ① §3.2 明记 `es×ev = 7 块` **高于**同形态噪声（0）= 可检出的跨实现差（占比 1.4e-8，极小但非零）；② 「端到端确定性」限定为 **coreswap 臂内**（`ea-r1×ea-r2 = 0`，494,100,480 块）；③ 删除/限定「**该维不存在 run 级非确定**」这一跨臂外推，并补一句「`ev` 仅 1 run ⇒ `es×ev` 的 7 块差无法在『实现差』与『vanilla 侧 run 噪声』之间归因」；④ §6 的「end 位级零差」内联限定为「在比对集（common=7567 chunks、两侧 section 并集 494,100,480 块）上」。
- **C3（MUST）end 侧 sanity 缺口的处置。** 事实：`run_arms3.ps1:32` 定义了 `eaS`，但 `logs/`、`results.txt`、MANIFEST 中**均无 eaS 记录**；A/B 双臂 mixlog 关 ⇒ 全块无 `populateNoise(end) intercepted` / end `[WG-FILL]`。择一：**①（首选，静态）**在 §2/§5 显式声明并把 §2 的「接管生效已排除」限定为 nether（`naS`），end 改用**就地间接证据**补锚 —— `[CppBridge] initEnd enabled=true`（`es-r1.log:96`、`ea-r1.log:96`、`ea-r2.log:96`）+ 仪器读数 `inflight max=1`（`es-r1.log:107/142`）/`=23`（`ea-r1.log:107`、`ea-r2.log:107`）（只有 end 分支的 work 区间才能产生 inflight 计数）+ `es×ev` 非零差；**②（需新运行）**补跑 `eaS-r1`（驱动已支持），拿到 per-chunk 直证后再升级措辞。
- **C4（MUST）性能比值口径必须点明，且不得用「端到端」措辞。** §4 须写：「比值取自 Chunky 进程内任务计时（`sec`/`Total time`）」并同列 wallgen 口径：nether sync→async **2.34×**（70.1→30.0）、nether vanilla→async **2.00×**（60.0→30.0）、end sync→async 2.0×（20→10）；注明 `wallgen−sec` 尾巴 1.1–10s 不稳定。项目「端到端性能对比铁律」下，本块结论应自我限定为「生成任务计时口径」，另立端到端（含存档/停服）为未测项。
- **C5（MUST）CPU 边界补解读 + §5.4 根因更正。** ① 把 `ea-r2` 的 `serverCpu=86 / cores=8.6`（`results.txt:13`）读出来：6s 任务耗 86 CPU·s ⇒ 该形态是**更多总 CPU 换更低延迟**，「反超/持平」是延迟结论；nether 侧总 CPU 未测。② 更正根因：现版 `run_arms3.ps1`（`.tmp` 与 `cmd-output` **两份我都读**）在 `:90-91` 停服前采样、`:112` 是「**不得**重复采样」注释、`:113` 计算 `cores`——**不存在重复采样覆盖块**；而本批 8/9 臂仍 `-1` ⇒ 「E2 = 重复采样覆盖」既解释不了本批（record §2.2），也与 C7 judge C9② 的「:112-113 仍重复采样」描述不符 ⇒ **CPU 缺失的实际根因 = 未核**，须按未核项声明，禁止写成「脚本已修 ⇒ 已解决」。
- **C6（MUST）§1 执行体句修正。** 「全部 9 臂 `dllsha=abd7d8893d22e030`」→「nether/end 9 臂中 **7 个 coreswap 臂**为 `abd7d8893d22e030`；`nv-r1`/`ev-r1` 为 vanilla（`dllsha=-`，`results.txt:6/10`）」；「与 `target/release/worldgen1216.dll` 一致」本块**未复算**（sha 来自各臂 `[CppBridge] dll=` 行），须注明为继承 260910-04 的核验或降为未核。
- **C7（MUST）单对点估计的强度须内联。** §3.1「0.1219% ≤ 0.1406% ✅ PASS」与 §6「行为门（…）三面齐备」须内联「每形态仅 1 对、无置信区间、检验力低（仅表示未检出超噪声的形态分量，不等于已证等价）」。
- **C8（MUST）`[WG-FILL]` 门控与计数语义写清。** ① 门控仅 wrapping `println`：`fillChunkNether/End` 的 `nzBuf` 全缓冲扫描 + 16 点 section 采样仍每 chunk 无条件执行（post `CppBridge.java:441-453 / 485-499`；overworld `fillChunk` 无此块）⇒ 「R1 同族拉平」是**日志层**而非工作层（残余量级未测）。② `4761` = **调用次数**（`[WG-FILL]` / `intercepted` / `buildSurface skipped` 三者行计数均为 4761），而 Chunky 报 `Processed: 4225 chunks` ⇒ 约 +13% 重复接管调用，不得读作 chunk 数。
- **C9（MUST）把 end「0 差」的可信性从断言升级为可核论证。** 在 §3.2 或 §5 增三条：① 同工具同口径在 `es×ev` 检出 7 块（`diff-E_sync_vs_vanilla.txt:74-77`）⇒ 比对链敏感；② `common=7567`、两侧 chunk 数均为 7567（`:1-2`）⇒ 无单侧排除；③ `blocks` 494,055,424/494,100,480 且 `sections` 120,619/120,630 >0 ⇒ 完成比对后的零（附 `MANIFEST` 三方 end region 同名 mca sha 互异，值未复算）。
- **C10（MUST）根索引摘要同步。** `index.yaml:1214-1218` 的注释复述了 C1/C2/C4/C5 所修正的表述（「3.3-3.5×」「快 2.7-2.8×」无口径、「端到端确定性」、「自身 run 级噪声」）⇒ 与 verdict 同批改正，保持检索面一致。

## 四、建议项（N1..N7，不阻塞）

- **N1**：登记「延迟换 CPU」的量化读数：单 chunk mixin 时长 async 下膨胀（nether ≈10-11→25-37ms；end ≈3.5→20-22ms），`sumMixin/wall` 平均并发度仅 ≈5.75（nether）/≈15.4（end）vs 峰值 23 ⇒ 后置课题（`writeChunk`/JNI 争用）；同时说明 `Chunky radius 500` 的来源（脚本无 `chunky radius` 命令；4225=65² ⇔ 半径 32 chunk）。
- **N2**：§3.1 补一句：`na-r1×ns` 为 `common=7542` 而 `na-r1` 侧含 7543 chunks（`diff-N_async_vs_sync.txt:1-2`）⇒ 单侧 1 chunk 被排除（覆盖率影响可忽略，但应披露）。
- **N3**：把「region 全域普查」的口径写全：region 目录含比 Chunky 任务更多的 chunk（nether `common=7542` vs `Processed 4225`；overworld C7 的 7959 同族）⇒ 覆盖面是**区域目录口径**而非任务口径（同时可作为对 C7-C6「7959 无出处」的回应线索；本块未证成因，仅登记口径）。
- **N4**：`eaS` 臂「已定义未跑」应在 `record.md` 待办与 `run_arms3.ps1` 注释中显式登记（放弃或补跑），防后续读者误以为已覆盖。
- **N5**：`patch-nether-end-async.md:3` 仍写「状态：**未应用**」，而改动已应用并跑完 A/B ⇒ P5 收尾时回填状态（避免后续读者误判为未落地）。
- **N6**：登记一项**同族既有风险**（不阻塞）：重复接管确实存在（4761>4225），`feedBeardifier`（setBeardifier）与 `fillChunk` 是两次 JNI 调用，同 chunk 的两次接管若交错，理论上存在「A 的 fill 读到 B 写入的 beardifier」窗口；因同 chunk 内容同源 + overworld 已 confirmed 同形态，本块不新增分析，建议作为后续专项登记。
- **N7**：建议在 §5 增一条边界：本块未做 nether/end 的 `-Dsyncfill` × `-Pmixlog` 交叉/重复接管安全性专项（与 C7-C6 的覆盖口径问题同族）。

## 五、对 confirmed 授予的明确意见

**条件支持**（C1–C10 全部应用后授予；C1/C2/C4/C5 为实质项，其余为口径/证据范围/契约一致项）。范围：

1. **「nether/end 两分支与 overworld R3 逐要素同构、overworld 未被动过」→ 支持 confirmed**（逐要素表 + pre/post 直读，无遗漏、无越界）。
2. **「`[WG-FILL]` 门控与 R1 同族、A/B 对称、naS 单独开有据」→ 支持**（C8 的措辞澄清必须随附）。
3. **「inflight max 1→23 两维复现」→ 支持 confirmed**（仪器语义 + 四个臂内原位行）。
4. **「形态效应 3.0-3.5×」→ 支持**，但限定为「Chunky 进程内任务计时口径」（C4）；**「nether 反超 vanilla 2.7×」同口径限定**（wallgen 口径为 2.0×，两值须并列）。
5. **「nether 跨形态 ≤ 同形态噪声」→ 支持为「单对点估计下未超阈（检验力低）」，不得 confirmed 为「等价已证」**（C7）。
6. **「end 跨形态位级零差」→ 支持**（假阴性已排除，C9 须随附），**但「该维不存在 run 级非确定」与把 7 块与 0 并列为同性质 → 不支持**（C2）。
7. **「nether 接管 vs vanilla 差异落在噪声带内、不等于接管已验证」→ 表述支持**，惟须写准噪声带来源（C1）。
8. **明确不同意**（延续 04/C7 judge 立场）：不得把「内容零差 / 并行度精确 1.000 / R3 绝对无内容影响」写入任何 confirmed 文本；也不得把「接管 vs vanilla 已对齐」「端到端持平/反超」写成无条件结论。
9. **范围外**：C7 收口结论与 260910-04 R3 主体本件不涉（C7 已另有 judge 判定）；本件不支持借 P4 verdict 一并 confirmed C7 或 04 的派生解释。

## 六、未核项与诚实边界

1. **无 shell**：未跑任何命令（未 git status/diff、未复算 sha256、未跑 diff 工具）。`HEAD=42d46e7`、三处 gitignore 声明、`java-snapshot` 的 sha 值、`MANIFEST` 的 sha 值、`target/release/worldgen1216.dll` 相等性，均为**继承/未复算**。
2. **未核**：`region-*/**.mca` 内容（只核 MANIFEST 中每臂 16 mca 与同名文件 sha 互异）；`diff_arms.py` 的解码/BitPack 正确性（只核其分母语义 = 两侧 section 并集，并用「es×ev 检出 7 块」证明其可检出差异）；`common=7542/7567` 与 `Processed 4225` 的差额成因。
3. **未核**：`sec` 与 `wallgen` 的采样点语义（`run_arms3.ps1:80-88` 读法只到「`sec` 来自 Chunky `Total time`、`wallgen` 来自脚本计时」这一层）；`wallgen−sec` 尾巴不一致的成因。
4. **未核**：CPU 缺失的**实际根因**（现版脚本无重复采样块，8/9 臂仍 -1；我未跑脚本，无法定位）。
5. **未核**：`end-takeover/`（10 文件）与 `nether-save-full/`（10 文件）我只抽样 4 份（引 1.20.1），「均 1.20.1」为抽样结论；`Chunky radius 500` 的配置来源。
6. 我**未修改任何被审文件**（verdict、record、patch、mixin、CppBridge、java-snapshot、diff 输出、results/logs、MANIFEST、region 目录、`index-entry.yaml`、根 `index.yaml` 均未动）；**本文件是本次审查的唯一写入**。

## 七、与 C7 judge verdict 的交叠（仅限交叠处，不重审 C7）

1. **C7-C9②（脚本仍有重复采样）↔ 本件 C5②**：我认为 **C7-C9② 的事实描述与现版脚本不符**——`run_arms3.ps1:112` 是「不得重复采样」注释、`:113` 计算 `cores`，两份副本（`.tmp` 与 `cmd-output`）一致。两种可能（我无法用 shell 分辨真伪）：(a) C7 judge 行号/内容误读；(b) 归档副本在 C7 审查后被改过（若如此，属 C7-N6 警示的「同名归档物 ≠ 产出证据的修订」，且与 `MANIFEST` 的 `run_arms3.ps1` sha 有关）。**无论哪种**，P4 §5.4 都不能再以「脚本已修」解释本批 CPU 缺失 —— 按 C5② 改为未核项即可。
2. **C7-C6（`7959 vs 4225` 无出处）↔ 本件 N3**：P4 侧同一现象（`common=7542/7567` vs `Processed 4225`）已由 P4 verdict §3 的「覆盖面 = region 全域普查」+ 明列 common 值部分覆盖；建议按 N3 补「区域目录口径 ≠ 任务口径」一句，C7 侧若要闭合可复用该口径说明（成因仍未证）。
3. **C7-C7（index 契约）**：已闭合——`index-entry.yaml`（4 条）与 `index.yaml:1210-1220` 均在位；本件只要求摘要注释随 C1/C2/C4/C5 同步（C10），**不改 status、不改 id/path**。
4. **一致性立场**：本件与 C7 均不支持「内容零差 / 绝对无内容影响 / 并行度精确 1.000」类措辞进入 confirmed 文本；本件追加的形态相关读数为「单块膨胀与平均并发度」而非内容差，不推翻 C7 的任何结论。
