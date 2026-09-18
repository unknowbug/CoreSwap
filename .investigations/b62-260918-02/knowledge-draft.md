# B6-2（260918-02）知识库落盘草稿（subagent 产出，主会话应用 + 验证）

> 续号核对（读文件确认）：`knowledge/discovered/workflow-patterns.md` 现有最大 **#173**（发现 #173，260918-01）；`knowledge/discovered/build-tooling.md` 现有最大 **#155**。本草稿新占：workflow-patterns **#174/#175**、build-tooling **#156**。
> 价值门预检（GUIDE §〇）：候选 1 = 可复用判据延伸（高价值，详写）；候选 2 = 对账工具覆盖面盲区/判错的判错家族新形态（中-高价值）；候选 3 = #90 家族预防形态（中价值，简记）；候选 4（镜像纪律）= **未过价值门，不独立立条**——「补映射 -P 命名与所在族/兄弟载体逐字对齐」已由 AGENTS.md §一 + workflow-patterns #19（点分/驼峰三犯）+ build-tooling #19 完整承载，本次 B6-2 仅是其执行实例（12 项命名逐字对齐），无新机制面，写入即稀释。

---

## 一、workflow-patterns.md 追加（文件末尾，「## 发现 #173」之后，追加不覆盖）

```markdown
## 发现 #174（高价值·判据延伸）: 跨载体共享消费点的族内对称性判据延伸——「族内先例」可取另一载体的映射先例，共享消费点在单载体缺映射 = 同源缺陷（260918-02）

- **观察**：B6-2 对 1.21.6 侧 21 项缺口逐项定性时，`coreswap.bulkwblog/wbcontent/bulkwbtest/bulkwbsentinel`（bulkwb 族 4 项）与 `coreswap.stallwatch` 在 **1.21.6 自己的 build.gradle 内零先例**——按族内对称性判据的字面读法（「族内零 `-P` 映射 ⇒ SCOPED」）应判合法旁路。但它们的消费点全部在**共享载体**（`java-core\src` 的 BulkWb.java/StallWatch.java/CppBridge.java——两版本用同一份代码），而 **1.20.1 已为同一批消费点建立 `-P` 先例**（B6-1 补映射并确证真缺陷）。
- **机制（为什么字面读法在这里失效）**：族内对称性判据的原理是「用户自然预期：同族参数能走同一通道」。当消费点在共享载体时，用户面对的「族」不是 per-carrier 的 build.gradle 族，而是**跨载体的同一消费面**——1.20.1 已证用户预期 `-P` 可用（且 B6-1 确证缺失是真缺陷而非设计），同一共享消费点在另一载体缺映射就不是「族设计即 `-D` 直传」，而是**同源缺陷在第二载体的复制态**。这是 #168（per-carrier 视角：union 口径掩盖单版本缺口）的判据面延伸——#168 说「缺口要 per-version 看」，本条补「定性时『族内先例』的证据域也要跨载体取」。
- **可复用判据（表述）**：族内对称性判据的「族内先例」取证域 = **消费点所属族的全部载体**，不只当前 build.gradle。消费点在共享载体（两版同一份代码）时：任一载体已为该消费点建立 `-P` 先例 ⇒ 另一载体缺映射 = **DEFECT**（同源缺陷）；仅当**全部载体**均零先例时才落「SCOPED」侧，且仍须过豁免子句合读（#171——探针专用路径 + 缺省权威两条合读，1.20.1 的定性结论证明 bulkwb 族不是豁免形态：bulkwblog 是 MUST 级判据的唯一仪器）。
- **证据/来源定位**：`.investigations/b62-260918-02/t4-adjudication-1216.md` §2.1（12 项 DEFECT 中 5 项靠此延伸定性，判据源 = `.investigations/b61-260917-06/t4-adjudication.md` §2/§3.4）；补映射后门禁 per-version 缺口 21→9（全 SCOPED），哨兵发射面 12/12 PASS（`.investigations/b62-260918-02/phase25-sentinel-and-gate.md`）。
- **验证分层**：定性 Degraded（静态审查）；发射面 Partial（init script 直证 jvmArgs）。
- **如何利用**：跨载体课题做开关映射对账时，单载体 build.gradle 的族清单是**不完整的先例证据域**；判 SCOPED 前先 grep 另一载体是否已为同一共享消费点建过映射。
```

```markdown
## 发现 #175 简记: 计划转述 vs 机械产出清单——定性以机械证据为准，「exec 族疑 SCOPED」被门禁 gap 清单机械更正（260918-02，#90 转抄漂移预防形态）

- **观察**：B6-2 架构计划 §0 转述「exec 族（exec/maxinflight）疑 SCOPED」为待定性项。一手门禁 gap 清单实测：这两项**根本不在 1.21.6 的 21 项缺口内**——其唯一消费点在 `versions/1.20.1/.../NoiseChunkGeneratorMixin.java:106/:138`（1.20.1 树独有），1.21.6 树与共享载体均无消费点 ⇒ 无缺口、无需定性。
- **教训**：计划/交接里的转述性列举（哪怕来自上一块的结论）不构成工作清单；**定性对象的集合以机械产出（门禁 gap 清单/扫描差集）为准**，计划转述逐项与机械清单对账，对不上的以机械为准并在定性文档里留一行更正记录（本例：t4-adjudication-1216.md §0）。#90（转抄漂移）的预防形态：不是「发现漂移后溯源」，而是开工第一步就让机械清单否决转述清单。
- **来源定位**：`.investigations/b62-260918-02/t4-adjudication-1216.md` §0。
```

---

## 二、build-tooling.md 追加（文件末尾，「### 发现 #155」之后，追加不覆盖）

```markdown
## 发现 #156 简记: 门禁消费面抽取的 `WgCompat.flag` 盲区——sysprop 名以变量实参传入绕过「`System.getProperty("字面量")`」正则（260918-02，#169 家族新形态）

- **观察**：`check_switch_mapping.py` 的消费面抽取靠 `RE_GETPROP`（匹配 `System.getProperty("字面量")`）。但 `WgCompat.flag("coreswap.bulkwb", …)` / `WgCompat.flag("coreswap.skipair", …)` 这类形态里，sysprop 名是 `WgCompat.flag` 的**变量实参**，字面量在 `WgCompat` 内部拼接/直读——正则抓不到 ⇒ 这两个开关**不在门禁的消费面集合**。
- **影响面（本例诚实登记）**：本次**未造成误判**——两版 build.gradle 均未声明这两个开关，缺口集合不受影响（不产生假 DEAD/ORPHAN）。但这是 #169（「判据的判据」：对账工具覆盖面自身必须可核）的**家族新形态**：#169 说差集数字不可脱离 coverage 引用，本条给出一个具体的 coverage 缺口实例——抽取器正则只认「字面量直读」形态，任何经包装函数/变量中转的 sysprop 读取都是盲区。
- **处置**：本块不改门禁（范围外），登记为门禁后续强化点——强化方向 = 抽取器增补「包装函数形态」规则（识别 `WgCompat.flag(...)` 等已知包装器 + 其内部 getProperty 读取点的展开），或最小化：消费面报告显式声明「只覆盖字面量直读形态」的 coverage 注记（#169 合规的最小动作）。
- **来源定位**：`.investigations/b62-260918-02/t4-adjudication-1216.md` §4 未处置 1 + §2.1 注。
```

---

## 三、knowledge/INDEX.md 需追加的行（文件末尾，现有「> 260918-01 追加…」行之后）

```markdown

> 260918-02 追加：workflow-patterns 新增**发现 #174（高价值·判据延伸）**（跨载体共享消费点的族内对称性判据延伸——消费点在共享载体（java-core 两版同一份代码）时「族内先例」可取另一载体映射先例：1.20.1 已为同一消费点建 -P 先例 ⇒ 另一载体缺映射 = 同源缺陷；#168 per-carrier 视角的判据面延伸，B6-2 12 项 DEFECT 中 5 项（bulkwb 4 + stallwatch）靠此定性）+ **发现 #175 简记**（计划转述 vs 机械产出清单——「exec 族疑 SCOPED」被门禁 gap 清单机械更正：两项根本不在 21 项缺口内（1.21.6 无消费点）；#90 转抄漂移预防形态 = 开工先让机械清单否决转述清单）；build-tooling 新增**发现 #156 简记**（门禁消费面抽取的 `WgCompat.flag` 盲区——sysprop 名以变量实参传入绕过 `System.getProperty("字面量")` 正则，#169 家族新形态；本次未误判，登记为门禁 coverage 强化点）。来源：.investigations/b62-260918-02/（t4-adjudication-1216.md + phase25-sentinel-and-gate.md；judge PASS 无 M 级）。
```

---

## 四、versions/1.21.6/docs/10-timewise-archive.md 需追加的 260918-02 时间线块（文件末尾，260914-02 块之后）

```markdown

## 260918-02（实际 2026-09-18，日期锚 = 门禁实测记录 12:43）：B6-2——1.21.6 侧开关映射补齐（21 项缺口逐项定性 → 12 项 DEFECT 补映射 → 哨兵 12/12 → 门禁闭环）—— ✅ judge PASS（无 M 级）；confirmed 待用户授予

> 过程产物 `.investigations/b62-260918-02/`（`t4-adjudication-1216.md` = T1 逐项定性 + `phase25-sentinel-and-gate.md` = T2-T4/judge S1-S4 应用记录 + `knowledge-draft.md`）。上游：B6-1（260918-01，判据源 = `.investigations/b61-260917-06/t4-adjudication.md` §2/§3.4 + extension-per-version-gap.md 的 21 项缺口清单）。通用模式 → workflow-patterns **#174/#175** + build-tooling **#156**（subagent 草稿 → 主会话应用）。

- ✅ **T1 定性（Degraded，静态审查）**：1.21.6 缺口 21 项 = DEFECT-MISSING-MAPPING **12**（surfaceDumpDim/blobProbe 4/colProf 2/bulkwb 4/stallwatch——跨载体 5 项按 #174 延伸判据定性）+ SCOPED-DIRECT-D **9**（豁免子句①②合读，#171）。计划 §0「exec 族」被门禁清单机械否决（1.21.6 无消费点，→ #175）。
- ✅ **T2 落地**：`versions/1.21.6/java/build.gradle` +20 行（benchVmArgs 闭包内，#47 作用域；命名与 1.20.1 B6-1 修复逐字对齐，#19）；git diff --stat = 20 insertions，1.20.1 侧零变化。
- ✅ **T3 哨兵（Partial，发射面直证 #170）**：init script 打印 `t.jvmArgs`，12 项新 `-D` 全部在场 = **12/12 PASS**；原始日志 `cmd-output/sentinel-1216-all12.log`（BUILD SUCCESSFUL 3m9s）；副作用登记三行（.tmp derived / run\world in-place 可重建 / loom 缓存显式不可逆声明，§9.8）。
- ✅ **T4 门禁闭环**：`check_switch_mapping.py` 1.21.6 声明 106→118，per-version 缺口 21→9（剩余 9 = 全部 SCOPED 豁免项，真缺陷归零）；判据字面「缺口=0」过宽经 judge S1 修正为「DEFECT 缺口=0」（判据意图与闭环一致，无取代）。
- ✅ **judge = PASS（无 M 级）**：S1 判据措辞修正 / S2 行号双源澄清（:46 消费行 vs :37 javadoc 行）/ S3 blobProbe 四行在 `if (blobProbe)` 块外（行为等价，下次触碰顺手对齐）/ S4 哨兵落盘附退出码行（后续改进）。
- ⚠️ **附带观察（独立待查，不阻塞）**：哨兵日志 `[BLOB-PROBE] stats read failed: Mixin transformation ... failed`——1.21.6 BlobProbeMixin mixin 变换失败（「编译绿 ≠ apply 绿」#40 家族），与本块改动无因果（配置层 vs 类变换层）。
- ⚠️ **诚实边界**：门禁 `WgCompat.flag` 消费面盲区未修（→ build-tooling #156 登记强化点）；`coreswap.bulkwb` 本体不在缺口清单（经 WgCompat.flag 读，消费面未计）；`max.bg.threads` DEAD 待核（B6-1 遗留，非本块范围）。
- 状态：✅ judge PASS（无 M 级）；**confirmed 待用户授予**。
```

---

## 自检清单（GUIDE §四）

- [x] 价值门：#174 高（判据延伸，详写）/ #175 中（简记）/ #156 中-高（简记）；候选 4 未过门（重复既有 #19/AGENTS 判据），**不立条**，理由已述。
- [x] 续号：workflow-patterns #174/#175（现最大 #173）、build-tooling #156（现最大 #155），无冲突。
- [x] 数字均来自主会话提供的一手记录（t4-adjudication-1216.md / phase25-sentinel-and-gate.md），无编造、无占位符。
- [x] 载体：通用判据 → discovered/；过程 → 1.21.6 时间线（本块为 1.21.6 侧工作，且该文件现无 260918 块）；INDEX 追加行与既有「> YYMMDD-XX 追加」格式对齐。
- [x] 追加不覆盖；所有块标注「追加位置」。
