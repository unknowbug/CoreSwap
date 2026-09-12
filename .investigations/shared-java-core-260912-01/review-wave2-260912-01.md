# Wave 2「语义统一」独立审查（260912-01）

> **审查角色**：core-judge（只出意见；**不改任何 status / 置信度，不改产物体正文**）
> **审查日期**：2026-09-12（日期锚 = git HEAD 提交时间 14:45 + 本会话 `pwsh` 取时）
> **审查对象**：D3 共享 Java 适配核 **Wave 2 = 语义统一**（commit `997d40f`）
> **被判据**：计划 §14.1 **V1b-1/2/3**（`架构计划-260912-01-共享Java适配核.md:226-230`）
> **verdict**：**PASS-with-conditions**（见 §7 verdict 展开；条件 C1–C9）

---

## 0. 审查范围与三源快照

### 0.1 三源（缺一即不算核对过）

| 源 | 具体载体（本次实读） |
|---|---|
| ① **产物快照** | `record-260912-01.md` §4.6（V1b 逐条声明）+ §4.7（V2/V3 记录）、`java-core/README.md`、`scout-map.md`、`review-scout-260912-01.md`、计划 §5/§12/§13/§14 |
| ② **git HEAD + 工作区 diff** | HEAD `997d40fd891123780ac6a3d7891ff0172fea6d03`（2026-09-12 14:45）；Wave 1 = `ce5286b`；`git status` 实取；**6 份 pre 源码副本与 `ce5286b` blob 逐字节对拍**；两版 `build.gradle` srcDir；`java-core/` 与 `versions/*/java` 源码树 |
| ③ **验证记录** | `.tmp/shared-java-core-260912-01/{pre,w2}/*.tsv`（4 份 manifest + 2 份冻结基线）、`build3-*.log`、`v1b*.txt`、`post2-vs-post3-*.txt`、3 个臂的运行日志 + `.err`、`fp_compare.py` / `jar_manifest_diff.py` / `javap_method_diff.py` / `class_home_check.py` **复跑** |

### 0.2 工作区状态（纪律事实，非结论）

```
M .investigations/shared-java-core-260912-01/record-260912-01.md   ← §4.6/§4.7 全段未提交（+84 行）
?? .investigations/shared-java-core-260912-01/knowledge-draft-260912-01-docs.md   ← 未跟踪
```

- **§4.6 / §4.7（本次审查主对象）只存在于工作区，未进任何 commit**。HEAD `997d40f` 的树里没有这两节。
- `record §6 结论 / 置信度 / 分层` = `（待回填；candidate 授予前 MUST judge）` ⇒ **交付物当前未声称任何 status**，无「AI 自授 confirmed」违规。
- 所有 Wave 2 证据（manifest / jar / 日志 / 指纹）**全部位于被 gitignore 的 `.tmp/`**（`.gitignore:86 .tmp/`）；`.investigations/shared-java-core-260912-01/` 下**不存在 `evidence/` 目录**，无 tracked 副本。（详见 C5）

### 0.3 本次审查未做的事（边界声明）

- **未**修改 record / README / 任何源码 / 任何 status（仅新建 `.tmp/w2-judge/*` 与本文）。
- **未**跑 gradle / runServer（遵守任务约束）；运行期数据全部来自既有日志的**复算**。
- `javap` 版本：`jdk-24.0.1`（`javap -version` = 24.0.1），两版两侧同工具同参数。
- **未**验证项目：Wave 1（V1-strict）结论；`ChunkTiming` 超集在 1.20.1 的实际分项数值（该臂未开 `-Dcoreswap.chunktime`）。
- 任务书里引用的计划路径 `…-共享Java适配核心.md` 有一字之差；**实际文件 = `架构计划-260912-01-共享Java适配核.md`**（record L4 / README L5 引用均正确，仅任务书笔误）。本文一律用实际名。

---

## 1. 逐条主张判定表（汇总）

| # | 主张 | 判定 | 一句话依据 |
|---|---|---|---|
| 1 | V1b 声明完整性（1.20.1「6 差异+1 新增」/ 1.21.6「5 差异+5 新增」逐条被声明、无漏项多项） | **成立**（含措辞修正） | 复算 = 1.20.1 **7 变+1 增**、1.21.6 **5 变+5 增**；第 7 条 = dll，§4.7.4 已声明且隔离事实复算成立 |
| 2 | 四条「指令完全相同、仅调试属性」（实为 5 条含 `CppBridge$1` 两版） | **成立** | 5/5 抽样复核：`javap -c -p -constants` 文本**逐字相同**；`-v -p` 差异**仅 LineNumberTable**（+Classfile 路径/SHA-256 自证行），`code=0 / pool=0 / LVT=0 / other=0` |
| 3 | 1.20.1「生产行为不变」（a–e 五小项） | **成立** | a 源码逐字同 + **字节码偏移序列逐字同**；b 仅 2 处声明变更；c 源码同 + 不在 javap 差异集；d **逐输入等价**（含 `""`/`"00"`）；e 缺省 true |
| 4 | V2/V3 运行证据（dll 自证同值 / seed 同 / 两族指纹全等） | **部分成立**（3 项复算 ✅ + 3 处记录失准） | dll `dd3b645f` 两臂同 ✅；seed 同 ✅；`only-pre=0/only-post=0` ✅；但「异常行 0/0」**被证伪**，「V3 覆盖逐格写回本体」**过度声称**，nether/end 未覆盖 |
| 5 | 1.21.6 残留风险是否阻断 Wave 2 | **不阻断 Wave 2**（但构成 D-4(i) 阻断 + 必记项） | 计划 §5/§12 对 1.21.6 的 V3 要求 = **sanity 命中**（625 命中 ✅）；D-4(i) 默认关已批准。**但** V1b-3 的 1.21.6 证据在 record 里**待回填**，且强制 bulk 臂**实测量崩**（130 异常）——record 完全未记 |
| 6 | 落盘 / 纪律（唯一归属 / mixin 包 / README 血统表 / 结论是否只在对话） | **部分成立** | V6 复跑 PASS、无同名校两份、java-core 无 mixin 类、srcDir 接线正确、README 血统表与源码事实一致；**但** 计划 §9 强制的 `errors-260912-01.md` **不存在**、证据仅在 `.tmp/`、record 未提交、`.artifacts/index.yaml` 无条目 |
| 7 | 诚实性（过度声称 / 该记未记） | **存在 4 处过度声称 + 3 处该记未记** | 见 §4.7 与 C2/C3/C4/C6 |

---

## 2. 主张 1 —— V1b 声明完整性（复算）

### 2.1 命令与结果（`jar_manifest_diff.py` 复跑）

```
1.20.1  pre\manifest-1.20.1-pre.tsv  vs  w2\manifest3-1.20.1.tsv
        相同=1074  差异=7  新增=1  删除=0
1.21.6  pre\manifest-1.21.6-pre.tsv  vs  w2\manifest3-1.21.6.tsv
        相同=1793  差异=5  新增=5  删除=0
隔离核  w2\manifest2-1.20.1.tsv vs w2\manifest3-1.20.1.tsv  ⇒ 差异=1（仅 native/worldgen.dll）
        w2\manifest2-1.21.6.tsv vs w2\manifest3-1.21.6.tsv  ⇒ 差异=0
```

### 2.2 逐条对齐

**1.20.1（7 变 + 1 增）**：`BulkWb$TL` / `BulkWb` / `ChunkTiming` / `CoreSwapFixHelper` / `CppBridge$1` / `CppBridge`（= §4.6 表的 6 行「差异」）＋ `+WgCompat`（= §4.6 的「新增」）＋ `native/worldgen.dll`（**§4.7.4 单列**，不在 §4.6 表内）。
⇒ **无漏项、无多项**。§4.6 的「6 差异」是 **Java 面条目数**，第 7 条 dll 差异由 §4.7.4 承担声明，两处合计覆盖全量差异集。
`StallWatch`（§4.6 表列为「无变化 / 字节全等」）**成立**：`javap -v -p` 对称差异仅 2 行（两条 `Classfile` 路径自证行），`LN=0/LVT=0/pool=0/code=0` ⇒ 字节全等。

**1.21.6（5 变 + 5 增）**：`mixins.json` / `refmap` / `ChunkTiming` / `CppBridge$1` / `CppBridge`（5 变，与 §4.6 表**逐条同名**）＋ `BulkWb$TL` / `BulkWb` / `StallWatch` / `WgCompat` / `mixin/ChunkSectionAccessor`（5 增，与 §4.6 表**逐条同名**）。
⇒ **完全成立**，dll 侧 0 差异（`post2 == post3` jar sha 相同），无归一化污染。

### 2.3 dll 隔离声明（§4.7.4）判定 = **成立**

`manifest2 vs manifest3` 复算 1.20.1 **恰 1 条**（`native/worldgen.dll`：`838e8979…` → `dd3b645f…`）、1.21.6 **恰 0 条**。⇒ §4.7.4「dll 归一化对 class 条目零影响」**独立复算确认**。

### 2.4 措辞失准（需修，不影响结论）

- §4.6 首行把 post jar 写成 **1.20.1 `41f4a551…` / 1.21.6 `2be87e40…`** —— 这两个 sha 属于 **`post-1.20.1.jar` / `post-1.21.6.jar`（= 首次 post 构建，`manifest-*.tsv`）**，而 §4.7.4 自述「V1b 的 Java 面判定用 **post2**」（`0681ec03…` / `772d7a6e…`），§4.7.1 的三元组核验用 **post3**（`461baedc…` / `772d7a6e…`）。**§4.6 表头引用的 jar sha 与其声明的差异集不属同一构建**（post1 的差异集与 post2/post3 **不同**，见 §2.5）。⇒ 记 **C6**。
- §4.6 首行「**6 差异**」未注明「另有 1 条 dll 差异见 §4.7.4」，读者按 §4.6 单读会与 `jar_manifest_diff.py` 输出（7 变）对不上。⇒ 记 **C6**。

### 2.5 顺带查出的基线漂移（record 已含线索，但未落在 §4.6）

`v1b-1.20.1.txt`（post1）与 `v1b2-1.20.1.txt`（post2）的差异集**不同**：

| 条目 | post1 (`41f4a551`) | post2/post3 |
|---|---|---|
| `StallWatch.class` | **在差异集内**（`e3127175…`→`e78f09e7…`） | **不在**（字节全等） |
| `CoreSwapFixHelper.class` | `33033ab2…` | `21c92aba…` |

1.21.6 同理：post1 有 `CoreSwapFixHelper` 差异 + `StallWatch` post sha `fdbd30c8…`；post2/post3 无 `CoreSwapFixHelper` 差异 + `StallWatch` `68e305c2…`。
⇒ §4.6 的「StallWatch 删注释恢复字节锚」叙述**可被证据解释**（post1→post2 的注释回退），但 **§4.6 未记录这次基线切换**，而表头却引 post1 的 sha。记 **C6**。

---

## 3. 主张 2 —— 「指令完全相同、仅调试属性」抽样复核（5/5）

### 3.1 方法

自建脚本 `.tmp/w2-judge/judge_javap.ps1`：用 record 自己的 `extract_entry.py` 从 **pre jar / post3 jar** 抽 `.class` → `javap -c -p -constants` 与 `javap -v -p` → 三重判定：

- **[A]** `javap -c -p -constants` 文本（路径归一后）是否**逐字相同**；
- **[B]** record 自己的 `javap_method_diff.py`（偏移 + 常量池序号归一）方法级结果；
- **[C]** `javap -v -p` 对称差异按属性**严格分类**（期望：仅 `LineNumberTable`）。

### 3.2 结果（全部落在 `LineNumberTable`）

| 类（版本） | [A] 逐字相同 | [B] 方法级 | [C] 严格分类（total / LN / LVT / pool / code / other） |
|---|---|---|---|
| `BulkWb$TL`（1.20.1） | **True** | 1/1 same | 24 / **20** / 0 / 0 / 0 / **0** |
| `CoreSwapFixHelper`（1.20.1） | **True** | 15/15 same | 344 / **340** / 0 / 0 / 0 / **0** |
| `CppBridge$1`（1.20.1） | **True** | 3/3 same | 14 / **10** / 0 / 0 / 0 / **0** |
| `ChunkTiming`（1.21.6） | **True** | 16/16 same | 124 / **120** / 0 / 0 / 0 / **0** |
| `CppBridge$1`（1.21.6） | **True** | 3/3 same | 14 / **10** / 0 / 0 / 0 / **0** |

（`hdr` 余数 = 2×`Classfile <path>` 自证行 + 2×`SHA-256 checksum` 自证行；`LVT=0` 表示**局部变量表也完全相同**，不仅 LineNumberTable。）

⇒ **主张 2 成立**：这 5 条的**方法指令序列真正等价**，差异只出现在 LineNumberTable（源码注释行数变化导致的行号位移）与 class 文件自证行。

参照点（非主张项）：`StallWatch`(1.20.1) / `WgDiag`(1.20.1) total=2（仅路径行）⇒ **字节全等**，与 §4.6 / README 的「不变」一致。

---

## 4. 主张 3 —— 1.20.1「生产行为不变」源码级核对（a–e 全部成立）

**方法**：`.tmp/w2-judge/method_body_diff.py`（花括号配对整法提取 + 文本对拍），输入 = **已验证血统的 pre 源码**（见 §4.0）与 `java-core` 现源码。

### 4.0 pre 源码血统（先验，避免拿错基线）

6 份 `.tmp/.../w2/pre-*.java` / `orig-*.java` 与 `git cat-file blob ce5286b:<path>` **LF 归一后逐字节相同**（sha256 全等）：

```
CppBridge.java(1.20.1) 0cc60bc0dae0c0b8 ✅   CppBridge.java(1.21.6) ac6a206ff3664650 ✅
BulkWb.java(1.20.1)    bdf2a29cd41f2336 ✅   ChunkTiming.java(1.21.6) e60e757fc37c7358 ✅
CoreSwapFixHelper.java 2fdc83d600851218 ✅   StallWatch.java(1.20.1)  c62f16559119d9bf ✅
```

### 4.1 (a) `stateById` pre ≡ post —— **成立，且比 record 的论证更强**

- **源码**：`[IDENTICAL] static BlockState stateById  pre L629-638  post L725-734`（10 行 vs 10 行，逐字相同）。
- **字节码**：`javap_method_diff` 报 `stateById` CHANGED（同 30 指令，6 行归一后不同）。逐行展开后，**10 处原始差异全部是常量池序号**（`#987→#1010`、`#990→#1013`、`#853→#876` …），而
  - **偏移序列完全一致**：`0,3,4,7,10,11,12,15,18,19,24,27,28,29,32,35,38,39,42,43,44,47,50,51,54,55,56,59,60`（逐字相同 ⇒ 代码长度与跳转目标均相同）
  - **助记符/操作数/描述符完全一致**（`getstatic/iload_0/invokevirtual/checkcast/astore_1/ifnonnull/goto/areturn` …）
  ⇒ record 的「常量池序号伪差」**判定正确**；我进一步用「偏移序列逐字相同」把它从「归一化后仍差 6 行」升级为**直接可判**。

### 4.2 (b) `writeChunk` 唯一变化 = 常量 + 计时钩子 2 行 —— **成立**

```
@@ pre L561-579 → post L651-671 @@
-        if (BulkWb.ON) {          +        if (BULKWB) {
+        long th0 = ChunkTiming.ON ? System.nanoTime() : 0L;   // B2 并入（1.21.6 :540）
         Heightmap.populateHeightmaps(...);
+        ChunkTiming.addHmap(System.nanoTime() - th0);   // B2 并入（1.21.6 :548）
```
**恰好** ① `BulkWb.ON`→`BULKWB` ② `populateHeightmaps` 前后 2 行计时钩子。无第三处。⇒ 与 §4.6 逐字吻合。

### 4.3 (c) `writeChunkPerBlock` 未变 —— **成立（双证）**

- 源码：`[IDENTICAL] writeChunkPerBlock  pre L582-618  post L674-710`（37 行 vs 37 行）。
- 字节码：`writeChunkPerBlock` **不在** 1.20.1 `CppBridge` 的方法级差异集内（见 §4.5）。
⇒ A1a 跳空气 / WBCHECK 自检原样。

### 4.4 (d) `BULKWB` 逐输入等价 —— **成立（含边界串）**

`pre`：`!"0".equals(System.getProperty("coreswap.bulkwb"))`（`pre-1.20.1-CppBridge.java:643`）
`post`：`WgCompat.flag("coreswap.bulkwb", WgCompat.BULKWB_ON)`，`WgCompat.flag = v == null ? fallback : !"0".equals(v)`，1.20.1 `BULKWB_ON = true`。

| `v` | pre | post(1.20.1) | 等价 |
|---|---|---|---|
| `null`（未设） | `true` | `fallback=true` | ✅ |
| `"0"` | `false` | `false` | ✅ |
| `"1"` | `true` | `true` | ✅ |
| `""` | `true` | `true` | ✅ |
| `"00"` / 任意其他串 | `true` | `true` | ✅ |

⇒ **逐输入等价**（含 record 未列的 `""` 与 `"00"`）。1.21.6 侧 `BULKWB_ON=false` ⇒ `null` 时 `false`（record §4.3 S-2/S-3 已显式声明，非漏项）。

### 4.5 (e) `SKIPAIR` 1.20.1 缺省仍 true —— **成立**

`pre:643` `SKIPAIR = !"0".equals(getProperty("coreswap.skipair"))`；`post:621` = `WgCompat.flag(..., SKIPAIR_ON)`，**1.20.1 `SKIPAIR_ON = true`** ⇒ `null` 时同为 `true`。输入等价表同 §4.4。

### 4.6 独立复算 `CppBridge` 方法级（record 表头数字被证实）

```
1.20.1  pre methods=32  post=33   same=28 changed=4 added=1
        CHANGED: fillChunk(138→168) / writeChunk(30→40) / stateById(30→30) / lambda$static$0(98→97)
        ADDED  : rustFeaturesTakeover()
1.21.6  pre methods=30  post=33   same=18 changed=12 added=3
        CHANGED: resolveThreads / init(84→85) / initEnd / wgMethod / fillChunk / fillChunkNether /
                 fillChunkEnd / writeChunk(133→40) / destroy(14→43) / didCompProbe / compProbe / lambda$static$0
        ADDED  : wgBufHash / writeChunkPerBlock(128 指令) / stateById
```
⇒ §4.6 的 `same=28/changed=4/added=1`、`writeChunk 133→40`、`writeChunkPerBlock 新增 128 指令`、`destroy 14→43`、`init +1` **逐项成立**。
⇒ record 对 `lambda$static$0` 的**诚实声明**（「lambda 名按序号命名 ⇒ 本条不构成证据」）**判定正确**：我复核 pre 98 指令 vs post 97 指令，方法名相同但内容不可比 —— record 主动作废自己的证据，属诚实行为。

### 4.7 未声明的影响面核查（发现 1 处记录内自相矛盾）

**`nanoTime()` 门控核查 —— record 的表述准确**：post 中 8 处 `System.nanoTime()`：
- 4 处**条件式**（`ChunkTiming.ON ? System.nanoTime() : 0L`，关时**不求值**）：`:457 tj0 / :480 ts0 / :494 tw0 / :659 th0`；
- 4 处**无条件**（`ChunkTiming.addXxx(System.nanoTime() - t0)` 的实参，Java 实参**急切求值**）：`:465 / :486 / :500 / :667`。

⇒ §4.6 残留风险 #1「新增 **4 次** `System.nanoTime()` 求值/chunk（`addJni/addScan/addWrite/addHmap` 的实参）… `ChunkTiming.addXxx` 内部 `if (ON)` 已门控 adder 写入」**精确成立**（`ChunkTiming.java:80-86` 逐行确认 `if (ON) X.add(ns)`）。**未把 4 次调用说成「被 ON 门控」** —— 诚实。

**但有 1 处记录内自相矛盾 + 1 处源码注释与源码事实矛盾（记 C4）**：

1. `java-core/.../ChunkTiming.java:15-19`（**同 commit 出货的共享源**）声明：
   > **1.20.1**：仅 `mixin/NoiseChunkGeneratorMixin` 调 `enter/exit/inflightEnter/inflightExit`；**其 `CppBridge` 无分项钩子** ⇒ `jni/write/hmap/scan/beard/carve/feat` 与 `featInterval` 在该版**恒 0**。

   **实测反例**：`pre-1.20.1-CppBridge.java` 中 `ChunkTiming.` 调用点 = **0 个**，而共享 `CppBridge.java` = **4 个**（`:465/:486/:500/:667`），且该共享文件**正是 1.20.1 的构建输入**（`javap` 实证：1.20.1 `fillChunk` 138→168 指令，钩子已编译进 1.20.1 jar）。⇒ 1.20.1 的 `jni/write/hmap/scan` **不是恒 0**；恒 0 的只有 `beard/carve/feat/featInterval`（其 mixin 为 1.21.6 独有）。
2. **§4.6 残留风险 #1 与 #3 互相矛盾**：#1 说 1.20.1 确实调用这 4 个钩子；#3（承 §4.3 S-7）说「1.20.1 `[CHUNKTIME]` 多打印**恒 0 列**」。二者不可能同时成立。

**影响面**：诊断面（默认 `chunktime` 关）、**零生产行为影响、零字节影响** ⇒ 不构成行为等价性缺陷，但属**过度声称/文档失真**，且违反本项目「注释断言须现场核对」纪律（计划 §6 R6 / §14.2 KB #25 家族）。

---

## 5. 主张 4 —— V2/V3 运行证据（复算）

### 5.1 dll 自证行与 seed（**成立**）

逐臂读**执行体自证行**（非 target 文件 sha，遵 §4.7.1 升级判据）：

| 臂 | 日志 | `[CppBridge] dll= sha256=` | seed |
|---|---|---|---|
| 1.20.1 post（主树 HEAD） | `w2c-post-1.20.1.log:117` | `dd3b645f2c79d2cb…` | `417950215108767439` |
| 1.20.1 pre（`ce5286b` worktree） | `1201-w2c-pre-1.20.1.log:170` | `dd3b645f2c79d2cb…` | `417950215108767439` |
| 1.21.6 post | `1216-w2-post-1.21.6.log:94` | `abd7d8893d22e030…` | `417950215108767439` |
| 1.21.6 forced-bulk | `1216-w2-bulk-1.21.6.log:94` | `abd7d8893d22e030…` | `417950215108767439` |

⇒ 1.20.1 两臂 dll **同值** ✅（`dd3b645f…`，与 record §4.7.2 一致）；seed 同 ✅；`initNether/initEnd` 均 `enabled=true stageMask=3`。
mod 集合同为 **48 mods**（含 `coreswap 1.20.1-1.0.29` + `testcontent 1.0.0` + `chunky 1.3.146`）⇒ Java 侧唯一变量成立 ✅。

### 5.2 `[WG-CONTENT]` / `[WG-CONTENT-WB]` 指纹（**成立**）

```
fp_compare(pre, post, "\[WG-CONTENT-WB\] chunk") → pre=607 post=607 common=607 only-pre=0 only-post=0
                                                    multiset=YES  sorted-seq=YES
fp_compare(pre, post, "\[WG-CONTENT\] chunk")    → 同上（607/607/0/0，multiset=YES）
```
另独立复算**去重坐标集**：`WB distinct coords pre=607 post=607 only-pre=0 only-post=0`，坐标包围盒两臂同为 `x[-15..9] × z[-13..11]`，**无重复行**。⇒ **逐 chunk 精确等**成立，且覆盖 607 个不同 chunk。

### 5.3 覆盖面与「门是否改变被测路径」

- **607 vs record 的「441 chunks」可解释**：`[Chunky] Task finished … Processed: 441 chunks (100.00%), Total time: 0:00:05`（chunky 目标区 = radius 160 blk = 10 chunk ⇒ 21×21 = **441**）；而 `fillChunk` 还会为 carver/feature 生成**邻接 chunk** ⇒ 门覆盖 **607 > 441**（包围盒 25×25 减去 18 个未覆盖位）。⇒ 覆盖**广于**目标区，非盲区。
- **门是否改变被测路径 = 否**：`BulkWb.contentLine(chunk,cx,cz)`（`:670`）位于 `writeChunk` **写回之后**，是只读读回指纹；`MIXLOG` 控制 `[WG-CONTENT]`（`:491/:536/:590`）也是只读 FNV 计算。两臂**同门控**（`.err` 实证同为 `-Dcoreswap.mixlog=1 -Dcoreswap.wbcontent=1`）⇒ 即使有扰动也等量施加。
- **真实覆盖盲区 2 处（记录已声明 1 处、未声明 1 处）**：
  1. **仅 overworld**：`[WG-CONTENT-NETHER]=0`、`[WG-CONTENT-END]=0`。而共享 `CppBridge` 的 `writeChunk` 被 **`fillChunk`(384) / `fillChunkNether`(256) / `fillChunkEnd`(128) 三者共用**（`:496/:525/:576`）⇒ **nether/end 走同一处被改的分派代码，但零运行期覆盖**。record 口径声明写了 `overworld`（**诚实**），但**未点明 nether/end 共享该路径** ⇒ 材料性盲区未被显式标注。记 **C7**。
  2. **`writeChunkPerBlock` 零运行期覆盖**（见 §5.4，记 **C3**）。

### 5.4 ⚠ 关键：V3 实际只覆盖 **bulk 路径**，未覆盖「逐格写回本体」（过度声称）

- 两臂 JVM flags（`.err` 实证）**均未设 `-Dcoreswap.bulkwb`** ⇒ 1.20.1 `BULKWB_ON=true` ⇒ `BULKWB=true` ⇒ `writeChunk` 走 `BulkWb.writeSections(...)`（`:653`）；`writeChunkPerBlock`（`:655`）**从未执行**。
- 而 §4.7.2 写：
  > 这是覆盖「`writeChunk` 分派常量替换 + **逐格写回本体未变**」的**运行期证据**（与 §4.6 的指令级证据互补）。

  ——「`writeChunk` 分派常量替换」✅ 被覆盖（两臂都求值该分派并同样选中 bulk）；「**逐格写回本体未变**」❌ **未被 V3 覆盖**（该分支两臂皆未进入），只有 §4.3 的 javap/源码证据覆盖它。
⇒ **过度声称**，记 **C3**。（结论本身仍安全：`writeChunkPerBlock` 源码逐字同 + 指令集不变，等价性由构造保证。）

### 5.5 ⚠ 「异常行 0 / 0」被**证伪**（record 门数字与自身证据不符）

以 record 自己脚本的判据（`run_pre_1201.ps1:66`：`"Exception|Error:|Cannot find target method|NoClassDefFoundError"`）复算：

| 臂 | `Exception\|Error:` 实际计数 | 内容 |
|---|---|---|
| post（1.20.1） | **4** | 全为 `[main/WARN] (WmiQueryHandler) COM exception querying Win32_*`（OSHI/OS 级告警，非项目代码） |
| pre（1.20.1） | **12** | 4 条上述 WMI 告警 ×2 段 ＋ **8 条真实异常链**（见下） |

pre 日志 `L38-67` 是**一次真实启动失败**：
```
[14:58:02] [main/ERROR] (Minecraft) Failed to start the minecraft server
java.lang.RuntimeException: Could not execute entrypoint stage 'main' due to errors, provided by 'testcontent'!
Caused by: java.lang.ExceptionInInitializerError
Caused by: java.lang.IllegalStateException: This registry can't create intrusive holders
    at testcontent.TestContentMod.<clinit>(TestContentMod.java:18)
```
**归因（我核过，非猜测）**：pre 臂的 gradle 调用**同时触发了 `:content-test:runServer`（L24）**（只加载 5 个 mod 的独立 run，`testcontent` 入口崩），随后 **`> Task :runServer`（L69）** 才是真正的对照臂（48 mods，`L71`），**成功**（`L134` 起服，`L2614` 441 chunks 完成，`L2632 BUILD SUCCESSFUL`）。post 臂（`run_ab.ps1`）**只有 `> Task :runServer`**（L17）。
⇒ **该失败的 `content-test` run 不影响对照结论**（真臂 mod 集合 = 48，两臂一致），**但 record §4.7.2 的「异常行 0」是错的**：它既没算 4 条 WMI 告警，更漏掉了 pre 臂这 8 条真实异常链。这属**门数字未按自身证据复算**，是本波最直接的诚实性问题。记 **C2**。

### 5.6 其他数值失准（次要）

- §4.7.2「生成 pre = 441 chunks / **10 s**」：日志 `[Chunky] Task finished … Total time: **0:00:05**`（两臂**同为 5 s**）。「10 s」疑为 `run_pre_1201.ps1:55-60` 的**轮询粒度**（`Start-Sleep 10`）产出的 wall 量，与 post 的「5 s」**非同一口径**却并排放进同一列 ⇒ §9.7「三要素同行声明」未落实。记 **C8**。
- §4.6「mixin AP 接受 1.21.6 的 `@Mutable` accessor」：`build3-*.log` 中 `Mutable` 命中 **0** ⇒ **引用日志不含该证据**。结论本身**可由产物证实**（`javap -v -p` 显示 `ChunkSectionAccessor` 带 `org.spongepowered.asm.mixin.gen.Accessor` ×7 + `org.spongepowered.asm.mixin.Mutable`，且构建成功 ⇒ AP 已接受）。⇒ 证据指针不准，非结论错。
- `build3-1.20.1.log` 的 `synced Rust dll` 命中 **0**（1.21.6 命中 1）⇒ 计划 §5 V2 要求的该指示行在 1.20.1 侧缺（UP-TO-DATE 跳过，正属 record 自述 KB #56/#96 家族）。但执行体自证行（`[CppBridge] dll=…`）是更强的载体 ⇒ V2 目的已达成，仅指示行缺位。

---

## 6. 主张 5 —— 1.21.6 残留风险：是否阻断 Wave 2？

### 6.1 判据（先查已批准计划，不凭感觉）

- **计划 §5 V3 行**：`1.21.6 **首次获得该门** ⇒ 先跑 **sanity**（行为化命中证据，#81）再谈等价`。
- **计划 §12 DoD**：`V3 1.20.1 三维指纹门 diff=0；**1.21.6 指纹门 sanity 命中**`。
- **计划 §13 HOOK-1**：`D-4 采纳方式 = (i) 共享代码 + 1.21.6 默认关，验证后单独翻转`；`HOOK-3 = 每个 1.21.6 语义采纳项放行（是否翻转默认）`。

⇒ 按**已批准判据**：1.21.6 的 V3 义务 = **sanity 命中**（不是等价）；翻转另走 HOOK-3。

### 6.2 结论：**不阻断 Wave 2**

1. 1.21.6 **默认关**已批准（`WgCompat1216.BULKWB_ON=false` / `SKIPAIR_ON=false`），且 1.21.6 **未出货**（record D-5；`coreswap1216-1.21.6-0.1.0`）。⇒ 无已出货行为依赖该路径。
2. 1.21.6 sanity **已命中且可复算自既有日志**：`1216-w2-post-1.21.6.log` = `intercept=625`、`[WG-CONTENT]`+`[WG-CONTENT-WB]` 各 625、`FAILED=0`、`Cannot find target method=0`、`BUILD SUCCESSFUL`、0 真实异常 ⇒ **V2 + V3-sanity 在证据上已满足**，只是 §4.7.3 仍写「**待回填**」。
3. 1.21.6 `writeChunk` 133→40 + `writeChunkPerBlock` 新增 —— 结构替换，符合 §14.1「两侧都变须显式声明」的要求（§4.6 残留风险 #2 已声明）。

### 6.3 但：**V1b-3 的落盘缺口 + 一个真实缺陷（必记 + 阻断 D-4(i)）**

- **缺口**：§14.1 **V1b-3** = 「任何 class 字节发生变化的版本 MUST 补跑 V2 + V3」。1.21.6 有 5 个条目变化 ⇒ 触发。而 §4.7.3 = `（待回填）` ⇒ **该 MUST 在记录中尚无证据**。（证据存在，记录未落。）记 **C1**。
- **真实缺陷（本次审查新发现，record 完全未记）**：强制开启臂 `1216-w2-bulk-1.21.6.log`：

  | 指标 | 强制 `-Dcoreswap.bulkwb=1` | 默认关（对照） |
  |---|---|---|
  | `[WG-CONTENT-WB]` | **0** | 625 |
  | `[CppBridge] DIAG write threw` | **130** | 0 |
  | `EntryMissingException` | **130**（`Missing Palette entry for index 2` … `index 8`） | 0 |
  | 收尾 | **未完成，栈回溯崩解**（末行 `ConfiguredCarver.carve` → `NoiseChunkGenerator.carve` → … `ForkJoinPool`） | 正常 `BUILD SUCCESSFUL` |
  | `.err` JVM flags | `-Dcoreswap.mixlog=1 **-Dcoreswap.bulkwb=1** -Dcoreswap.wbcontent=1` | 无 `bulkwb` |

  ⇒ **1.21.6 的 bulk 写回路径（= 由 1.20.1 形态替换进来的 `writeChunkPerBlock`/`writeSections` 组合）在运行期是坏的**：逐 chunk 抛 `EntryMissingException`（palette 索引缺失），写回后 chunk 状态被留成不一致，最终由 `carve` 阶段抛出并使生成流水线崩解。
  这正是 §4.6 残留风险 #2 所指向的项 —— 但现在它**不是「无法用指令级证明」的怀疑，而是有实测反例的已确认缺陷**。
  影响：① 不影响 1.20.1 出货线（不同版、默认关）；② **直接阻断 D-4(i) 的「1.21.6 自身 A/B」**（A/B 的开启臂跑不起来）⇒ 必须在 HOOK-3 之前修。
  违反本项目 **错误优先原则**（AGENTS.md §三.2：「不得只记『已修复』而不记『为什么错』」）：一个 130 次异常 + 完整栈回溯 + 流水线崩解的错误链，**record 里一个字都没有**。记 **C2**。

### 6.4 最小可行补证方案（供主会话采纳，成本 ≤ 1 轮）

1. **先修再证**（因为当前臂必崩）：定位 `EntryMissingException: Missing Palette entry for index N` 的机制面 —— 疑点在 `wgSetBlockStateContainer`/palette 语义（1.21.6 的 `ChunkSectionAccessor` 带 `@Mutable`，与 1.20.1 同形但 `PalettedContainer` 在 1.21.6 的序列化/palette 位宽语义可能不同；另 `BulkWb.buildContainer` 的 palette 构建路径需要对照 1.21.6 的 `PalettedContainer` 契约）。**派 fan-out/re-code worker 做机制定位**（判定树 ≥2 候选：palette 位宽 / `@Mutable` 字段语义 / `setBlockStateContainer` 后计数不一致）——不属本轮审查范围，仅给方向。
2. **修复后做逐 chunk 对照**（替代/补强 V3 等价）：
   - 臂 A：1.21.6 **默认**（`BULKWB=false`，走 1.21.6 原生 `writeChunk` 内联形态）；
   - 臂 B：1.21.6 `-Dcoreswap.bulkwb=0` 强制走 `writeChunkPerBlock`（A1a/A1b 同门控）；
   - 臂 C：1.21.6 `-Dcoreswap.bulkwb=1` 强制 bulk；
   - 同 seed / 同 chunky（overworld `center -48 -11` `radius 160`）/ 同 dll / 同 `-Dcoreswap.wbcontent=1`；
   - 判据 = 臂 B/C 的 `[WG-CONTENT-WB]` 多重集与序列 **diff=0**（`fp_compare.py`），并核 `[WG-FILL]`/异常行为 0。
   - 注意：该对照**必须**同时覆盖 **nether/end**（`chunky world the_nether|the_end`），否则重演 §5.3 盲区（这是 A2「三维同载体」的既有要求）。
3. **落盘**：把结果写入 `record §4.7.3` + 新建 `errors-260912-01.md`（五段式：现象/根因/定位/修复/教训）+ tracked 证据副本（C5）。

---

## 7. 主张 6 / 7 —— 落盘纪律与诚实性

### 7.1 成立项（复核通过）

- **V6 唯一归属**（`class_home_check.py` 复跑）：`交叠类 = 空`（1.20.1 / 1.21.6 双向）×，`共享源中位于 mixin 包的类 = 空`，共享类恰 **8** 个。另用目录扫描独立确认：两版工程树中**不存在** 8 个共享类的同名副本。⇒ 无「同名校两份」。
- **`java-core` 不含 mixin 包类** ✅（8 个全在 `wg/` 与 `wg/bench/`）。
- **`srcDir` 接线（机制 M1）** ✅：两版 `build.gradle` 均有 `srcDir '../../../java-core/src/main/java'` + 「一个类只有一个家」注释（1.20.1 `:28-31`、1.21.6 `:30-33`）。
- **README 血统表与源码事实一致** ✅：`StallWatch`(1.20.1 字节全等/1.21.6 新增)、`CoreSwapFixHelper`(1.20.1 仅 LineNumberTable/1.21.6 不变)、`ChunkTiming`、`BulkWb`、`CppBridge`、`WgCompat` 分版两份、`ChunkSectionAccessor` 1.21.6 为「1.20.1 同形副本」——`git diff --no-index` 实证：1.21.6 版 = 1.20.1 版 **+5 行头注释**，代码/注解/成员逐字同 ⇒「同形」准确。
- **README 头注释纪律与 `BulkWb.java` 的关系**（见 §7.3，属**建议**不属违规）。
- 无「结论只留在对话里」的**结论性**内容：§4.6/§4.7 已写入 record（虽未提交，见 C5）。

### 7.2 落盘缺陷

| 缺口 | 事实 | 影响 |
|---|---|---|
| `errors-260912-01.md` | 计划 §9 明文要求「错误台账：`.investigations/shared-java-core-260912-01/errors-260912-01.md`（五段式 + 速查表）」；**文件不存在** | dll 血统事故（§4.7.1）+ 1.21.6 bulk 崩解 两个错误链均无归口载体。违 AGENTS.md 错误优先原则。记 **C2** |
| 证据仅在 `.tmp/` | `.gitignore:86 .tmp/`；`.tmp/` 下全部 manifest / jar / 日志 / 指纹**未跟踪**；`.investigations/shared-java-core-260912-01/` 下无 `evidence/` | V1b 判据、V3 指纹、V2 日志**无法从仓库复现**；项目已有先例（`.gitignore` 白名单 `!.investigations/**/evidence/*.log`，260910-07 judge C13 / 260911-05）却未沿用。记 **C5** |
| record §4.6/§4.7 未提交 | 工作区 `M`，仅 +84 行未 commit | 判据与声明未固化，不满足「每结果必须落盘 + 版本化」。记 **C9** |
| `knowledge-draft-260912-01-docs.md` 未跟踪 | `??` 状态 | 草稿状态合法（subagent 产出待应用），但需跟踪或明确标注。随 C9 一并处理 |
| `.artifacts/index.yaml` 无本课题条目 | 检索 `shared-java|260912-01|java-core` 命中 **0** | 按 core.artifact 契约，结论产物应登记。**但** record §6 尚未授予 candidate（`待回填；candidate 授予前 MUST judge`）⇒ 现在登记为时过早，**属可接受的暂缓**，仅提示在 candidate 授予时同步登记 |

### 7.3 过度声称 / 该记未记 清单（主张 7）

**过度声称 4 处**：

1. §4.7.2「V3 是覆盖『**逐格写回本体未变**』的运行期证据」—— 两臂 `BULKWB=true`，`writeChunkPerBlock` 从未执行。（**C3**）
2. §4.7.2「异常行（`Exception|Error:`）**0 / 0**」—— 实测 **4 / 12**（pre 含 8 条真实异常链）。（**C2**）
3. §4.3 S-7 / §4.6 残留风险 #3「1.20.1 `[CHUNKTIME]` 多打印**恒 0 列**」—— 与 §4.6 残留风险 #1 自相矛盾；且被 `ChunkTiming.java:15-19` + 共享 `CppBridge` 4 个钩子点 + javap（`fillChunk` 138→168）证伪（恒 0 的只有 `beard/carve/feat/featInterval`）。（**C4**）
4. §4.6 表头 jar sha 与其声明的差异集**非同一次构建**（post1 `41f4a551…`/`2be87e40…` vs 判定所用 post2/post3）。（**C6**）

**该记未记 3 处**：

5. **1.21.6 强制 bulk 臂的运行期崩解**（130 `EntryMissingException` + 栈回溯 + 流水线未完成）—— record 零字。（**C2**，最高优先级）
6. **§4.7.3「待回填」**：1.21.6 的 V2 + V3-sanity 证据已存在于日志（625 命中 / 0 FAILED / BUILD SUCCESSFUL），未回填 ⇒ §14.1 V1b-3 的 MUST 无证据。（**C1**）
7. **pre 臂跑出 `:content-test:runServer` 的偶发失败段**（L24-67）未记录；虽不影响结论（真臂 48 mods 一致），但「异常行 0」正是因此失准。（**C2**）
8. （次要）nether/end 走同一被改的 `writeChunk` 但零覆盖 —— record 只说 `overworld`，未指出该盲区的**共享性**。（**C7**）

**诚实性正面认定**（应记而记了的）：

- §4.6 主动作废 `lambda$static$0` 证据（「lambda 名按序号命名 ⇒ 本条不构成证据」）—— 核实**判定正确**。
- §4.6 残留风险 #1 把 4 次无条件 `nanoTime()` 的代价**明写出来**，未粉饰为「已门控」—— 核实**表述精确**。
- §4.7.1 把 dll 血统事故（首次对照被引擎差异污染、判定作废一次）完整记入，并升格为可复用判据（「跨臂对照前 MUST 逐臂读执行体自证行」）—— 属**高质量错误记录**，与 1.21.6 那条形成鲜明对比。
- §4.7.4 主动披露「`build/libs` 现产物 ≠ 已发布 1.0.29 jar sha」并要求再发布重跑全量回归 —— 核实：`RELEASE-1.0.29.md` 记 jar `b057fda216012647…312239`、`judge-verdict-260911-04.md` 记三元组 MATCH（`b057fda2…` + `dd3b645f…`）⇒ **披露属实**。
- §4.7.2 §9.7 口径声明（载体/覆盖面/可比性）**在形式上齐备**（虽 §5.6 指出「生成」列口径混用）。

---

## 8. 攻击路线与结果（全部记录，含未奏效的）

| # | 攻击路线 | 结果 |
|---|---|---|
| A1 | 复算两版 manifest 差异集，找漏声明/多声明 | 未奏效（无漏/无多）；但发现「6 差异」措辞遗漏 dll 第 7 条 + 表头 sha 属 post1 |
| A2 | 复算 `manifest2 vs manifest3` 隔离声明 | 未奏效 —— 隔离声明**成立**（1.20.1=1 条、1.21.6=0 条） |
| A3 | 不信 record 的 javap 结论，自建三重判定（raw/-c/-v 严格分类）复核 5 条「仅调试属性」 | 未奏效 —— 5/5 **全部成立**，LVT/pool/code 差异均为 0 |
| A4 | 攻 `stateById`「常量池伪差」是否掩盖真实指令差异 | 未奏效 —— 反被强化：**偏移序列逐字相同**，指令/操作数/描述符全同 |
| A5 | 攻 `writeChunkPerBlock`「不变」 | 未奏效 —— 源码逐字同 + 不在 javap 差异集（双证） |
| A6 | 攻 `WgCompat.flag` 是否与旧表达式逐输入等价（含 `""`/`"00"` 边界） | 未奏效 —— **逐输入等价** |
| A7 | 攻 `SKIPAIR` 1.20.1 缺省 | 未奏效 —— `SKIPAIR_ON=true` |
| A8 | 查 `nanoTime()` 是否真被门控（record 是否粉饰） | 未奏效 —— record 主动披露 4 次无条件调用，**表述精确** |
| A9 | 查 pre 源码副本是否真出自 `ce5286b`（防拿错基线） | 未奏效 —— 6/6 blob **LF 归一后逐字节相同** |
| A10 | 逐臂读执行体自证行核 dll（遵 §4.7.1 升级判据） | 未奏效 —— 1.20.1 两臂同为 `dd3b645f…` |
| A11 | 复算 `[WG-CONTENT]`/`[WG-CONTENT-WB]` 多重集 + 去重坐标集 + 重复行 | 未奏效 —— 607/607，`only-pre=0/only-post=0`，无重复 |
| A12 | **查 V3 实际执行的是哪条写回分支**（是否真覆盖 `writeChunkPerBlock`） | **命中** —— 两臂 `bulkwb` 未设 ⇒ bulk 分支，逐格本体**零运行期覆盖**（记录过度声称） |
| A13 | **用 record 自己的异常判据复算 pre/post 日志** | **命中** —— 实测 4/12 ≠ record 的 0/0；pre 含真实启动异常链 |
| A14 | **查全部既有臂日志是否还有未记录的运行期失败** | **命中** —— `1216-w2-bulk-1.21.6.log` 130 异常 + 崩解，record 零字 |
| A15 | 攻 `ChunkTiming` 共享注释与共享 `CppBridge` 的一致性 | **命中** —— 注释称 1.20.1「无分项钩子/恒 0」，实测 4 个钩子点 + javap 证实编译进 1.20.1 |
| A16 | 攻 nether/end 是否被 V3 覆盖 | **部分命中** —— 零覆盖，且共享同一被改路径（record 未点明） |
| A17 | 核计划 §9 强制的 `errors-260912-01.md` 是否存在 | **命中** —— 不存在 |
| A18 | 核证据是否 tracked（`.gitignore` / `git ls-files`） | **命中** —— 全在 `.tmp/`，无 tracked 副本 |
| A19 | 核 `b057fda2…`（§4.7.4 披露）是否有外部依据 | 未奏效 —— `RELEASE-1.0.29.md` + `judge-verdict-260911-04.md` **确证** |
| A20 | 核「任务书引用的计划路径」 | 命中（轻微）—— 实际为 `…共享Java适配核.md`，任务书作「…核心.md」；record/README 引用**正确** |

---

## 9. 未能证伪项（诚实边界）

以下**我未能证伪，也未独立证实到 Full 层级**，不计入 PASS 依据：

1. **1.21.6 `ChunkTiming` 超集对 1.21.6 诊断面的实际影响** —— 该臂未开 `-Dcoreswap.chunktime`，无 `[CHUNKTIME]` 行可比（`CHUNKTIME=0`）；§4.3 S-7 的「多打印恒 0 列」只能从源码推断，**未运行期验证**。
2. **1.20.1 bulk 路径与 pre 的 bulk 实现是否逐指令相同** —— `BulkWb.java` 相对 pre 仅 `ON` 行 + 1 行头注释（`git diff` 已证），故我按构造认定等价 + V3 指纹等支持；但**未**对 `BulkWb.class` 做逐方法 javap 归一化对拍（其 `-v` 差异含 pool/code/LVT，非仅调试属性 ⇒ 不能套主张 2 的判据）。record 亦未主张该点。
3. **nether/end 在 1.20.1 的行为是否不变** —— 零运行期覆盖（A16）。仅 `writeChunk` 是**同一处**被改代码（分派常量替换，逐输入等价已证）+ `writeChunkPerBlock`/`BulkWb` 实现同 ⇒ 推断不变，但**无该维运行期证据**。
4. **1.21.6 默认关路径的内容正确性** —— 只有 sanity（625 命中），无 pre 对照（1.21.6 pre 无 `[WG-CONTENT-WB]` 载体，record 已诚实声明）。按计划这不要求（§5 V3），但「1.21.6 内容等价」这一命题**目前无证据**，不应被表述为已成立。
5. **`EntryMissingException` 的根因** —— 我只证到「1.21.6 bulk 臂实测崩解」与「异常类型/位置/palette 索引缺失」；**机制根因未定位**（未做 palette/`@Mutable`/`setBlockStateContainer` 的对照分析）⇒ 属下游修复任务，本文只给方向。
6. **`gradle :build --rerun-tasks` 是否真的 `--rerun-tasks`** —— `build3-*.log` 的 `5 actionable tasks: 5 executed` 支持，但命令行未见 `--offline --rerun-tasks` 字样（日志不回显完整命令行）⇒ 未独立证实。
7. **1.20.1 两臂的「生成」耗时口径** —— 只能指出「10 s vs 5 s」与 chunky 内建计时不符；无法判定记录采的是哪个量（脚本 stdout 未落盘）。

---

## 10. 条件清单（Cx）与 verdict

### 10.1 verdict = **PASS-with-conditions**

**理由**：**出货线（1.20.1）的核心命题经独立复算完全站得住**——① V1b 声明完整无漏项；② 5/5「仅调试属性」经三重 javap 判定成立（且 `LVT=0/code=0/pool=0`）；③ `stateById`/`writeChunk`/`writeChunkPerBlock`/`BULKWB`/`SKIPAIR` 五小项全部成立（含偏移序列逐字同这一比 record 更强的证据）；④ dll 自证同值、seed 同、两族指纹 607/607 逐 chunk 全等。1.21.6 按已批准 D-4(i)/§5/§12 **不构成 Wave 2 阻断**。
**之所以不是 PASS**：存在 **1 处门数字被自身证据证伪**、**1 处实质过度声称**、**1 处 record 内自相矛盾 + 出货源注释与源码事实矛盾**、**1 个未记录的真实运行期缺陷（含 D-4(i) 阻断）**、**1 个计划强制的错误台账缺失**、**证据全部不可复现于仓库**。
**之所以不是 FAIL**：没有任何证据表明**已交付的 1.20.1 产物**（jar 内的字节与行为）错误；上述问题集中在**验证记录/声明/注释/落盘纪律**层面；1.21.6 默认关且未出货。

**推荐 status（仅供人类参考，我不改任何 status）**：`record §6` 目前为 `待回填`；在 **C1–C4 修完、C5 证据落盘** 之前，**建议维持 draft（不授予 candidate）**；C6–C9 可在 candidate 授予的同批修正中一并处理。**confirmed 只能由人类授予。**

### 10.2 条件清单

| Cx | 级别 | 条件（对象 → 要求） | 依据 |
|---|---|---|---|
| **C1** | **MUST（阻断 candidate）** | `record §4.7.3` 回填 1.21.6 的 V2 + V3-sanity 结果（来源：`1216-w2-post-1.21.6.log`：`intercept=625`、`[WG-CONTENT]`/`[WG-CONTENT-WB]` 各 625、`FAILED=0`、`Cannot find target method=0`、`BUILD SUCCESSFUL`、0 真实异常），并写明「1.21.6 pre 无 `[WG-CONTENT-WB]` 载体 ⇒ V3 只做 sanity（计划 §5/§12），等价另行补证」 | 计划 §14.1 **V1b-3**「任何 class 字节变化的版本 MUST 补跑 V2+V3」在 1.21.6 侧无证据 |
| **C2** | **MUST（阻断 candidate）** | ① 更正 §4.7.2「异常行 0 / 0」→ 实测 **4 / 12**，并说明 pre 臂 12 条的构成（4×2 条 WMI 告警 + 8 条 `:content-test:runServer` 启动异常链；真臂 mod 集合 48 两臂一致故不影响结论）；② **新建 `errors-260912-01.md`**（计划 §9），把两条错误链按五段式记录：(a) dll 血统事故（§4.7.1 已有内容，归口）；(b) **1.21.6 强制 bulk 臂崩解**（130 `EntryMissingException: Missing Palette entry for index N` + `ConfiguredCarver.carve` 栈回溯 + 流水线未完成 + `[WG-CONTENT-WB]=0`）；③ 把 (b) 升格进 §4.6 残留风险，明确「已确认缺陷（非仅难证）」，并注明**阻断 D-4(i) 翻转** | ①record 门数字与其自身日志不符；②AGENTS.md 错误优先原则；③§4.6 #2 现被实测反例取代 |
| **C3** | **SHOULD（强）** | 更正 §4.7.2：V3 的运行期覆盖 = **bulk 路径**（两臂 `BULKWB=true`，`writeChunkPerBlock` 从未执行）；「逐格写回本体未变」只由 §4.3 的源码逐字同 + 指令集不变（`same=28`）支持，**不是** V3 的运行期证据 | A12：两臂 JVM flags 无 `-Dcoreswap.bulkwb` ⇒ `BULKWB_ON=true` ⇒ `:653` 分支 |
| **C4** | **SHOULD（强）** | ① 修正 `java-core/.../ChunkTiming.java:15-19` 的合并说明：1.20.1 **确有** `addJni/addScan/addWrite/addHmap` 钩子（`:465/:486/:500/:667`，javap 实证 `fillChunk` 138→168），恒 0 的只有 `beard/carve/feat/featInterval`；② 同步更正 §4.3 S-7 与 §4.6 残留风险 #3 的「多打印恒 0 列」表述，消除与 #1 的自相矛盾 | A15：注释与同 commit 的共享 `CppBridge` 事实矛盾；属 KB #25 家族（静态断言未现场核对） |
| **C5** | **MUST（阻断 candidate）** | 为 Wave 2 证据建立 **tracked 副本**：至少 pre/post manifest（4 份）、`v1b*.txt`、`post2-vs-post3-*.txt`、`build3-*.log`、两臂运行日志（`w2c-post-1.20.1.log` / `1201-w2c-pre-1.20.1.log` / `1216-w2-post-1.21.6.log` / `1216-w2-bulk-1.21.6.log`）、`fp` 复算输出，落到 `.investigations/shared-java-core-260912-01/evidence/*.log`（沿用 `.gitignore` 既有白名单，260910-07 judge C13 / 260911-05 先例）；并在 record 中登记证据路径 + 一条可复跑的 `jar_manifest_diff.py` / `fp_compare.py` 命令行 | A18：`.tmp/` 被 ignore，当前 V1b/V3 判据**不可从仓库复现** |
| **C6** | **SHOULD** | 更正 §4.6 表头：post jar sha 应取**判定所用基线**（Java 面 = `post2` `0681ec03…`/`772d7a6e…`；含权威 dll 交付态 = `post3` `461baedc…`/`772d7a6e…`），或并列标明 post1/post2/post3 的关系；并把「6 差异」写成「**6 条 Java 面条目差异 + 1 条 `native/worldgen.dll`（见 §4.7.4）**」；同时补记 post1→post2 的 `StallWatch` 注释回退导致基线切换（`v1b-*.txt` vs `v1b2-*.txt` 差异集不同） | A1/A2：表头 sha 与差异集非同一次构建；措辞与工具输出对不上 |
| **C7** | **SHOULD** | 在 §4.7.2 口径声明中显式标注：nether/end `writeChunk` 共用同一处被改代码（`:496/:525/:576`）而 V3 **零覆盖**（`[WG-CONTENT-NETHER]=[WG-CONTENT-END]=0`），并把三维覆盖列入 D-4(i)/后续补证要求（与 A2「三维同载体」一致） | A16：`fillChunk*` 三路共用 `writeChunk` |
| **C8** | **SUGGEST** | §4.7.2「生成」列统一口径并同行声明三要素：两臂 chunky 内建 `Total time` **同为 0:00:05**；若用脚本 wall（`run_pre_1201.ps1:55-60` 轮询粒度 10 s）须显式标注为不同度量 | §5.6：口径混用，违计划 §5 §9.7 三要素 |
| **C9** | **SHOULD** | 把 `record-260912-01.md`（含 §4.6/§4.7）**提交**；确认 `knowledge-draft-260912-01-docs.md` 的跟踪状态（subagent 草稿待主会话应用，应用后一并提交）；`candidate` 授予时在 `.artifacts/index.yaml` 登记本课题条目 | §0.2：主对象未提交；无 index 条目 |

> **修完 C1–C5 → 建议可授 `candidate`；C6–C9 建议同批或紧随。confirmed 仍须人类拍板。**

---

## 11. 新增 / 未证伪风险（汇总，供人类裁决排序）

| # | 风险 | 级别 | 证据 | 现有缓解 |
|---|---|---|---|---|
| N1 | **1.21.6 bulk 写回路径运行期崩解**（`EntryMissingException` ×130 → carve 阶段崩解）；D-4(i) 的 1.21.6 自身 A/B 目前**跑不起来** | **高（阻断 D-4(i)，不阻断 Wave 2）** | `1216-w2-bulk-1.21.6.log`（130 异常 + 栈回溯 + `WB=0`）；`.err` 确认 `-Dcoreswap.bulkwb=1` | 1.21.6 默认关（`BULKWB_ON=false`）⇒ 不影响出货；须先修再证（§6.4） |
| N2 | **V1b-3 的 1.21.6 证据未落盘**（§4.7.3 待回填），使 1.21.6 侧的 MUST 判据「程序上未满足」 | **中（形式合规）** | record §4.7.3 原文；日志证据齐备 | 证据已存在，回填即可（C1） |
| N3 | **V3 只覆盖 bulk 路径 + 只覆盖 overworld**；`writeChunkPerBlock` 与 nether/end 无运行期覆盖 | **中** | A12/A16 | 逐格本体有源码+指令级双证；nether/end 有「同一被改代码 + 逐输入等价」推断；建议 C3/C7 |
| N4 | **出货源注释与源码事实矛盾**（`ChunkTiming.java` 1.20.1「无分项钩子/恒 0」）—— 同族失真的先例价值高（KB #25 家族第三形态） | **低（诊断面，零生产影响）** | A15（4 钩子点 + javap 138→168） | 默认闭门控；C4 |
| N5 | **验证记录不可复现**（证据全在 gitignored `.tmp/`）—— 若 `.tmp/` 被清理，V1b/V3 判据与「dll 血统事故」教训同时丢失 | **中** | A18；`.gitignore:86` | C5 |
| N6 | 「异常行」类门数字**未经复算即写入 record** —— 同类失误可能存在于其他波次的「门数字」中 | **中（方法学）** | A13（4/12 ≠ 0/0） | 建议：门数字一律附复算命令 + 复算输出落盘（C5 命令行要求覆盖此点） |
| N7 | `errors-260912-01.md` 缺失（计划 §9 强制）⇒ 两个错误链无归口 | **中** | A17 | C2 |

---

## 12. 附：本次审查的可复现命令（证据落于 `.tmp/w2-judge/`）

```powershell
cd E:\PYTHON\CoreSwap
# 主张1 差异集复算
python .tmp\shared-java-core-260912-01\jar_manifest_diff.py `
  .tmp\shared-java-core-260912-01\pre\manifest-1.20.1-pre.tsv `
  .tmp\shared-java-core-260912-01\w2\manifest3-1.20.1.tsv          # → 7 变 + 1 增
# 隔离核
python .tmp\shared-java-core-260912-01\jar_manifest_diff.py `
  .tmp\shared-java-core-260912-01\w2\manifest2-1.20.1.tsv `
  .tmp\shared-java-core-260912-01\w2\manifest3-1.20.1.tsv          # → 仅 native/worldgen.dll
# 主张2/4.5 方法级 javap（自建脚本：三重判定 + CppBridge 逐方法）
pwsh -NoProfile -File .tmp\w2-judge\judge_javap.ps1
python .tmp\shared-java-core-260912-01\javap_method_diff.py `
  .tmp\w2-judge\CppBridge-1201-pre.javap .tmp\w2-judge\CppBridge-1201-post.javap
# 主张3 源码级方法体对拍 + pre 源码血统
python .tmp\w2-judge\method_body_diff.py `
  .tmp\shared-java-core-260912-01\w2\pre-1.20.1-CppBridge.java `
  java-core\src\main\java\wg\bench\CppBridge.java `
  "static BlockState stateById" "writeChunk(Chunk chunk" "writeChunkPerBlock(Chunk chunk"
python .tmp\w2-judge\prov.py            # 6 份 pre 副本 vs ce5286b blob
# 主张4 V3 指纹 + 覆盖
python .tmp\shared-java-core-260912-01\fp_compare.py `
  ".tmp\shared-java-core-260912-01\w2\1201-w2c-pre-1.20.1.log" `
  ".tmp\vivo-stutter-260911-02\ab-threads\logs\w2c-post-1.20.1.log" "\[WG-CONTENT-WB\] chunk"
python .tmp\w2-judge\cov.py             # 去重坐标集 / 包围盒 / 重复行
# 主张6 V6
python .tmp\shared-java-core-260912-01\class_home_check.py
# 主张4/7 异常与 dll 自证
Select-String -Path <arm.log> -Pattern 'Exception|Error:'
Select-String -Path <arm.log> -Pattern '\[CppBridge\] dll='
```

**本次审查新增文件**：`.tmp/w2-judge/judge_javap.ps1`、`method_body_diff.py`、`prov.py`、`cov.py`、`dump_method.py`、`CppBridge-{1201,1216}-{pre,post}.javap`、本报告。

---

**（审查意见到此。未修改任何 status / 置信度 / 产物体正文；`confirmed` 只能由人类授予。）**
