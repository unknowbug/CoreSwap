# .b2 settled 时序差 — T8 light-hash diff 44.69% 机制归因（260919-07，draft）

- 角色: fan-out worker .b2（辖区 = 采集/停服时点未 settled / 域批任务未终态 / 重载不收敛）
- 纪律: 静态分析 + 既有 log 只读解析（零采集）；F = 一手源码/日志直读，I = 推断，分开标注。
- 让渡: 空间重叠验证与 n=2 重跑执行权在主会话（见 §5 模板）。

---

## 0. 辖区结论（先答）

**倾向：timedOut=true ≠ 非终态——超时任务确实产出终态光照写回；但「grace 超时 → 部分域 + 按排序索引假装满网格装配」是一条代码级实证的静默错值通路，其触发条件纯由时序决定（.b2 辖区内），产量上限 625 chunk（不足以独力承载 4223）。**
「46s 尾窗内未 flush 任务」「重载不收敛」「采集时点未 settled」三个子机制均**无证据支持（排除倾向）**；但存在一个静态不可闭合的残余盲区（enqueueSectionData→落盘时序，§3.3）。
综合置信度：timedOut 终态语义 = 高（F，源码直读）；装配错位通路 = 高存在性/中「是否产错值」（F 结构 + I 数值影响）； settled/flush 排除 = 中-高（F 时序 + I 引擎语义）。

**总判（I，中置信）**：.b2 单独承载 44.69% **不可判且大概率不足**（625/4223 = 14.8% 硬上限）；.b2 的真实角色更可能是「时序选择哪些 chunk 走上错值通路」的选择器——即 .b1（内核/边界语义差）提供差值来源、时序（.b2）决定其在空间上的出现位置。两者不是互斥候选而是串联关系。此判断交 fan-out 收敛/judge。

---

## 1. Q1：timedOut=true 且 ok>0 的语义（.b2 核心待判事实）

### 1.1 F — 触发侧（LightDomainBatch.java）

- grace 定时器：域首提交时挂 10s 延迟 seal（`versions/1.20.1/java/src/main/java/wg/bench/LightDomainBatch.java:67-69`，GRACE_MS 定义 :21，默认 10_000 = log 头 `graceMs=10000` 一致）。
- 满 9 中心立即 seal(timedOut=false)（:81-83 / :109-112）。
- seal 幂等：`DOMAINS.remove(st.key, st)` 成功才执行一次，置 `st.timedOut`，计数 SEALED/TIMEDOUT，然后**无条件**执行 `taskFactory.run(st)`（:116-128）。**没有任何「超时则丢弃任务」的路径。**

### 1.2 F — 任务侧（ServerLightingProviderMixin.java）

- 任务体取 `st.futures.keySet()` 为 centers（`versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:602-603`）——部分域 = centers<9，任务照跑。
- 写回分支对 `st.timedOut` **完全不敏感**：`out != null` 即对每个 center `wgLightDomainWriteBack`（enqueueSectionData BLOCK+SKY 全 24 节）+ `setLightOn(true)` + `releaseLightTicket` + `f.complete(ch)` + `ok++`（:705-715；WriteBack 体 :582-596）。
- 降级路径（out==null）才走 legacy 重放，本次 run `degraded=0`（log 全量，域臂 457 行均 degraded=0）。

**语义结论（F，高置信）**：`timedOut=true 且 ok>0` = 域在 grace 内未凑满 9 中心、按已到齐中心子集封板并**正常完成终态光照写回**。ok 值 = 该批写回成功中心数；115 条 timedOut 行 ok 均>0 且 Σok = Σcenters = 625（log 实算，见 §2）——**无一个 chunk 因超时而未产出**。超时未到齐的「缺席中心」不在本批 futures 里，其后续提交会因 seal 已 remove 而新建 State 走新批（LightDomainBatch.java:65-66 compute 语义），同样最终产出。**不存在「timedOut 任务的光照是半成品」的读法。**

---

## 2. Q2：115 条 timedOut task 的空间分布可否判读

### 2.1 F — task 行本身：不可判

task 行格式（Mixin:733-743）只含 `centers/ok/degraded/timedOut/avgMs[/fillMs…]/sealed/timeout/dup[/fail]`，**无任何坐标/域键**。fail= 才带 ChunkPos，本次 fail 行 = 0。→ 从 task 行直接判读与簇 A/B 重合度：**不可判**。

### 2.2 F — 但 log 里存在间接坐标面：FP-LIGHT burst 关联

- 域批写回循环内每 center 调 `FormProbe.lightCall(cx, cz, …)`（Mixin:716-719）→ log 产生 `[FP-LIGHT] ev=call … cx=… cz=… th=Worker-Main-N` 行；task 行在同线程、于循环结束后立即打印（:733）。
- log 尾实读（18:58:00，Worker-Main-10/15/19）可见模式：同线程 6 连发 FP-LIGHT（gapMs 0.0-0.2，cx=330-331, cz=189-200）紧跟/伴随 `centers=6 ok=6 timedOut=true` task 行——**6 连发 = 该 timedOut 任务的 6 个 center 坐标**。
- 同理，inline/vanilla 回退路径也发 FP-LIGHT（wgLightLegacyTakeover 内 FormProbe.lightCall），但不伴随 task 行 → 按「task 行消费同线程前序 FP-LIGHT」做确定性配对，**未被消费的散行 = inline 路径 chunk 集**。
- 本次已实算（F，log 只读解析）：timedOut 任务 **115 条，Σcenters=Σok=625**；满域任务 342 条 Σcenters=3078；timedOut centers 分布 = 1×1, 2×2, 3×14, 4×6, 6×91, 8×1。

### 2.3 判读

- **硬上限（F）**：timedOut 错值通路最多污染 625 chunk < 4223 diff → **timedOut 任何机制都不可能独力解释 44.69%**。
- 6 连发样例坐标 (330-331, 189-200) **落在簇 B（x173-342 / z184-222）内**（F，样例级；全量重合度待主会话跑 §5.1 模板）。这与「簇 B = move5 尾波（最后 FP-LIGHT gapMs≈10715/11216 说明 18:58:00 是 10s+ 停顿后的收尾加载）+ 尾波到达慢 → grace 频繁超时」自洽（I）。
- ⚠️ **反读声明（I，防 §9.7 无效声明）**：簇 A/簇 B 成簇 + 中带零 diff **不能**直接读作「机制只在簇内发生」——中带也可能是光照值不敏感地形（如均匀海面/平原：错邻居算出的值恰好相同），机制全表面活跃但只在敏感地形显影。空间分布是「机制 × 地形敏感度」的卷积，单凭成簇无法解卷。此声明对 .b1/.b2/.b3 全体候选生效。

---

## 3. Q3：不动点/收敛性 + 46s 尾窗

### 3.1 F — 域批无「逐 task 增量收敛」结构

- 每 chunk 光照**恰好写回一次**：域提交 `cir.setReturnValue(cf)` cancel 原 vanilla light()（Mixin:559/574）；同一 centerPos 重复提交走 DUP 返回既有 future（LightDomainBatch.java:73-80），不重算。seal 后 DOMAINS 条目已删，同域后续提交是新批新 State，仍只产一次。
- 无任何重载-再传播循环：不存在「域批 fill 一次到位 vs 增量逼近不动点」的分叉——两臂都是单次计算终写。#157 的「一次重载收敛 0.10% vs 9.88%」是**跨 boot 重载收敛质量**口径，与本次 run 内单次写回无冲突，不构成本课题的机制面。

### 3.2 F — 时序账（域臂 log 实读）

Done@18:55:14 → move5@18:57:44（尾波加载启动）→ 最后 [LIGHT-DOMAIN] task + 最后 FP-LIGHT@18:58:00 → settle 60s（无新 task 行 = 无新提交/seal）→ rcon stop@18:58:46 → Saving chunks 18:58:47-52 → `All chunks are saved`（overworld 18:58:52）→ snap。

- 最后一批 grace 定时器最晚 18:58:00+10s 触发 seal，其 task 行若产出必在 18:58:1x 前打印——log 中 18:58:00 后**零** task 行（F）→ **尾窗内无未 flush 的域批任务**（高置信）。
- 「settle 60s 期间世界空闲、无 chunk 卸载重载、方块不变」→ 已写回的 light 值在 stop 前无变化来源（I，基于 vanilla 1.20.1 无固定时间外天光变化的常识；如需运行时确认见 §5.3）。**「采集时点未 settled」排除倾向成立。**

### 3.3 残余盲区（F 缺口，静态不可闭合）

- `enqueueSectionData`（Mixin:590-591）把 nibble 入 ServerLightingProvider 队列后，**本沙箱无 Java 参照源可核「stop 前队列必然 drain + 保存路径读到的是已应用值」**（yarn 源不在工作区，不猜）。若存在「入队未应用即保存」，会出现「最后时刻任务 chunk 存盘 stale light」——但两臂同为停服保存、vanilla 臂自身同样有 light 队列，且 46s 空闲 + save 阶段 5s，可能性低（I，低置信）。列入 §5.3 运行时核对模板，非本辖区可决。

---

## 4. 核心新发现：timedOut 部分域的装配错位通路（.b2 辖区内、代码级实证）

### 4.1 F — 结构

`wgLightDomainTaskRun` blocks25 装配（Mixin:670-690）：

- centers **排序后按 k=0..n-1 映射网格槽 `kx=k%3, kz=k/3`**（:673, :680-681），把 center k 的 3×3 邻域快照写到 blocks25 的 `(kx..kx+2, kz..kz+2)` 窗（:683-686）；
- 内核导出「只读各中心 3×3 子窗」，未覆盖边缘槽保持 0（:689-690 注释原文）。
- **代码中没有任何「到齐 centers = 对齐网格前缀」的校验。**

满域（centers=9）时 9 中心恰为对齐 3×3 全集，排序序 = 网格序，装配正确。**部分域（centers<9，即全部 115 条 timedOut 任务）时，到齐子集 ≠ 网格前缀** → 各快照被摆到与真实几何无关的网格槽上：窗口错位、互相重叠（同槽双写，后写者胜，序仍确定但非几何）、或留零隙。内核在这个**几何失真场**上算光并按槽 k 导出 → 写回给真实 chunk centers[k]。

### 4.2 判读（I，中-高置信）

- 每 center 的导出子窗中心确是它自己的 chunk（自身快照中块恒落窗中央），但**跨窗传播边界条件失真**（假邻接/假边界/零隙）→ 中心子窗边界带的光照值可偏离 vanilla per-chunk（vanilla 以真实 3×3 邻域为界）。
- 数值是否实际偏离取决于地形（边界带敏感度）——与 §2.3 反读声明一致：可能大量 chunk 偏差为 0。
- 本 run 走 blocks25 路：task 行无 `packed=` 字段（Mixin:735 仅 packedFrames≥0 打印）→ `LIGHT_PACKED` 关，`lightComputeDomainPacked` 未用（F）。
- **产出影响上限 625 chunk**（§2.2）；若全量 FP-LIGHT burst 提取（§5.1）显示 timedOut center 集与簇 A∪B 高重合，则「.b1 提供差值语义 × .b2 时序选择受害 chunk」的串联模型获得强证据。

---

## 5. 主会话执行命令模板

### 5.1 【零采集，最优先】timedOut/inline 受害 chunk 集提取 + 簇重合度（只读既有 r1 log）

```python
# -*- coding: utf-8 -*-
# t8_b2_burst.py — 从 t8-domain-r1.log 提取域批任务受惠/受害 chunk 坐标（只读，零采集）
# 方法: 每线程维护 FP-LIGHT 行队列；遇该线程 task 行时弹出队尾 centers=N 行 = 该任务 center 集
#       （依据: lightCall 在写回循环内、task 行在同线程循环后立即打印, Mixin:716-733）。
#       未被 task 行消费的 FP-LIGHT 散行 = inline/legacy 回退路径 chunk。
# 输出: timedOut 受害 chunk 集 / 满域 chunk 集 / inline 集 的坐标文本 +
#       与簇A(x3-57,z-35-0) 簇B(x173-342,z184-222) 的重合计数（按 dist-analysis 矩形界）。
import re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
LOG = r"E:\PYTHON\CoreSwap\.tmp\legacy-sweep-260919-06\t8-domain-r1.log"
rx_task = re.compile(r"\[LIGHT-DOMAIN\] task centers=(\d+) ok=(\d+).*timedOut=(true|false)")
rx_fp   = re.compile(r"\[FP-LIGHT\] ev=call \S+ cx=(-?\d+) cz=(-?\d+) th=(\S+)")
q = {}  # thread -> list[(cx,cz)]
out = {"to": [], "full": [], "inline": []}
for ln in open(LOG, encoding="utf-8", errors="replace"):
    m = rx_fp.search(ln)
    if m:
        q.setdefault(m.group(3), []).append((int(m.group(1)), int(m.group(2))))
        continue
    m = rx_task.search(ln)
    if m:
        n = int(m.group(1)); th = ln.split(" th=")[-1].split(" ")[0] if " th=" in ln else None
        # task 行线程: 行内 "[Worker-Main-N/INFO]" —— 用它取键
        thm = re.search(r"\[(Worker-Main-\d+)/", ln)
        th = thm.group(1) if thm else None
        buf = q.get(th, [])
        take = buf[-n:] if n <= len(buf) else buf  # 取后 n 个（防御不足）
        q[th] = buf[:len(buf)-len(take)]
        out["to" if m.group(3) == "true" else "full"].extend(take)
for th, buf in q.items(): out["inline"].extend(buf)
A = lambda p: 3 <= p[0] <= 57 and -35 <= p[1] <= 0
B = lambda p: 173 <= p[0] <= 342 and 184 <= p[1] <= 222
for k, v in out.items():
    inA = sum(1 for p in v if A(p)); inB = sum(1 for p in v if B(p))
    print(f"{k}: n={len(v)} inClusterA={inA} inClusterB={inB} other={len(v)-inA-inB}")
    with open(rf"E:\PYTHON\CoreSwap\.investigations\t8-attrib-260919-07\cmd-output\b2-{k}-chunks.txt", "w") as f:
        f.write("\n".join(f"{x},{z}" for x, z in v))
# 读法（预登记）: to 集 in(A∪B) 占比 ≥80% => timedOut 选择器与 diff 簇强重合（.b2 强证据）;
#               inline 集大量落在簇内 => 差值来源偏向 legacy/vanilla 分歧面（另一轴, 让渡 .b1/.b3）。
```

（线程键解析已按 log 行 `[Worker-Main-N/INFO]` 实形态写；防御段：buf 不足 n 时全取并记差异——若出现，说明配对假设被破坏，结果作废重议。）

### 5.2 【n=2 重跑判别实验】domain 臂自身 ×2 + 三层判别式

执行（每臂 rmtree 重建 + world 身份链 #161 内建于 run_t8.py；SELFCERT/VOID 非零退出内建 :136-164；本轮为 domain 自身对，无 vanilla 负自证臂）：

```
python .tmp\legacy-sweep-260919-06\run_t8.py r2 domain
python .tmp\legacy-sweep-260919-06\run_t8.py r3 domain
```

（OUTDIR 固定 260919-06，日志名带 tag 不冲突 :34-37。判据文件 criteria-t8.md 为存在性判据，本次 n=2 是归因实验，不回改其读法 #154。）

预登记判据文本（先于执行，四方judge 可审）：

- **前置集（preconditions，任一失效 → 该臂 VOID，run_t8.py 已机械非零退出）**：done≥1 / drv_move5≥1 / lightInit_ok≥1 / domain_hook≥1 / dll_sha8=cc4e39fe / seed_in_leveldat=true / region_min_mtime_gt_rmtree=true。domain 臂无负自证门（vanilla 负自证 #118 仅对 vanilla 臂有意义）；两臂各 457±量级 task 行为正对照参考（不作硬门）。
- **d_self = diff(r2,r3)（同执行体同环境，E1 档）；d_cross = diff(r2,vanilla-r1), diff(r3,vanilla-r1)（E2 档）。**
- **H-b2-支持**：d_self > 0 且 d_self chunk 的 ≥80% 落在簇 A∪B 内 且 |d_self| ≪ |d_cross| → 时序选择器面成立，.b2 升 candidate 共享者（与 .b1 串联）。
- **H-b2-排除**：d_self = 0（逐位）且 d_cross ≈ 4223 复现 → run 间非确定带不存在，瞬态/时序面剥离，证据流 .b1/.b3。
- **H-b1-纯型预证伪**（借本次顺带）：若 d_self = 0 但 d_cross 的 diff 矩形界扩到中带（x57-173 出现 diff）→ 与既有中带零 diff 矛盾，说明 r1 的空间形态本身有随机成分，全部既有空间判读降级重议。
- **机械判定**：diff 计数用 .tmp/legacy-sweep-260919-06/cmp_t8.py 形态复算 + judge-recompute 双算一致才采数（sample 行缺陷已知，不用其坐标）。
- **声明**：n=2 为 E1 档自身对拍，不外推其他 seed/维度/驱动；snap json 为稳定证据体，若任何动作重开 region 文件须重走 #161。

### 5.3 【可选，低位】enqueueSectionData 落盘时序核对（§3.3 盲区）

在 vanilla/yarn 参照源（gradle cache 的 ServerLightingProvider 反编译或 sources jar）核 `enqueueSectionData(LightType, ChunkSectionPos, ChunkNibbleArray)` 是否同步落 section 存储 + stop 保存路径是否等待 light executor。仅当 5.1/5.2 结果与「时序选择器」模型冲突时才值得做；预期结论为「已 drain」，作否证备案。

---

## 6. 让渡清单

1. → 主会话：跑 §5.1（零采集，直接产出 timedOut 受害集 × 簇重合度）、§5.2（n=2 重跑）；原始输出落 cmd-output/ 回传解读。
2. → fan-out 收敛/judge：.b1 与 .b2 非互斥而是「差值来源 × 时序选择器」串联模型（§0 总判）需按多假设竞争程序裁决；.b3 残余子面（section 缺键语义）不受本产物影响。
3. → .b1 worker：满域（342 条，3078 chunk）输出与 vanilla per-chunk 在中带零 diff 是对「内核算法差」的最强约束——任何 .b1 机制必须解释「为何中带满域不差、簇内才差」（地形敏感度卷积或真实空间选择性，二选一须给出判据）。
4. → 知识库（结论性沉淀时）：「grace 超时部分域 + 排序索引网格装配无子集校验」若经 5.1/5.2 证实产错值，按错误五段式（现象/根因/定位/修复/教训）立卡——修复方向预告：部分域装配按 centers 真实相对几何布位（或部分域直接整批走 legacy 重放）。
5. 边界让渡：625 受害 chunk 集坐标明细待 §5.1 产物；§3.3 引擎 drain 盲区不在静态辖区可决。

## 7. 置信度汇总

| 结论 | 级别 | 置信 |
|---|---|---|
| timedOut=true 且 ok>0 = 终态写回（非半成品） | F | 高 |
| 115 timedOut 任务 Σcenters=Σok=625（上限硬数） | F | 高 |
| 部分域装配无子集校验、按排序索引假装网格 | F | 高 |
| 该装配错位产生错误 light 值 | I | 中（数值面待 5.1/5.2） |
| 尾窗无未 flush 任务 / 采集时点已 settled | F+I | 中-高 |
| .b2 独力承载 44.69% | — | 排除倾向（625<4223） |
| .b1×.b2 串联模型 | I | 中（待判别实验） |

status: **draft**（无验证运行；待主会话执行 §5 后升级候选/证伪）。

---

## 8. §5.1 执行后更新（260919-07，主会话回传 + 本 worker 交集复算；仍 draft）

### 8.1 新事实（F）

原始输出：`cmd-output/b2-burst-extract-260919-07.txt`（无 WARN 配对破坏行）+ `b2-to/full/inline-chunks.txt`。本 worker 只读交集复算（t8-*-r1-light.json × 明细文件，零采集）：

- to 集 n=625：inA=97 inB=528 other=0 → **in(A∪B)=100%**；inline 集 n=0；full 集 n=3078：inA=432 inB=2646 other=0 → **同样 100% 落簇内**。
- 交集复算：**diff=4223 = diff∩to(625) + diff∩full(3078) + outside(520)；toHit=625/625、fullHit=3078/3078** —— 即本 run **全部 3703 个域批路径 chunk 100% 与 vanilla 臂不同**，另有 520 个非域批 chunk 也 diff；簇外（中带）零 diff chunk 全部为非域批 chunk。
- 旁证自洽（F）：域臂 [FP-SUM] `lightN=3703` == burst 提取总量 == Σcenters（625+3078）→ legacy/vanilla 回退路径不产生 FP-LIGHT ev=call 行；非域批 chunk（5747 = 9450−3703）在本 log 无坐标面。

### 8.2 判读修订（I）

1. **预登记判据字面满足、判别力失效**：to in(A∪B)=100% ≥80% 达「.b2 强证据」门槛，但 full 对照集同为 100% —— 空间成簇的选择变量是「是否走域批路径」，不是「是否 timedOut」。预登记未设 full 对照，判据只有必要性无特异性，按诚实声明降级：**「时序选择器」模型不支持为 diff 成员资格的决定变量**。
2. **更强的 .b1 信号浮出**：fullHit=3078/3078 —— 满网格正确装配的域批输出**每个**都与 vanilla per-chunk 不同，是系统性内核/边界语义差（.b1 辖区），非 625 受害 chunk 的孤立事故；「地形敏感度卷积」解释被压缩（full 集 100% 命中无法用敏感地形显影解释，除非域批 chunk 恰好只分布敏感地形——该分布本身待 .b1 解释）。
3. **覆盖缺口闭合、揭示第二贡献源**：outside=520 = 非域批路径（legacy 接管 per-chunk Rust light 或 vanilla 回退）在域臂 vs vanilla 臂的 diff → 若为 legacy Rust per-chunk，则存在**与域批无关的独立 diff 源**；44.69% = 域批差 + per-chunk Rust 差之和。§4 的 625 上限论证仍成立但已非主要面。
4. **.b2 辖区最终定位（降级）**：时序（grace/部分域/装配错位）只影响 625 chunk 内错值形态，不决定 diff 成员资格（由执行路径决定）；「未 settled/未 flush/不收敛」维持排除倾向。.b2 作为 44.69% 主嫌**降级为配角**；主嫌转移 .b1 + 新开「per-chunk lightRust diff」子候选（独立轴，让渡 fan-out 收敛裁决）。
5. **下一步（让渡主会话，零采集优先）**：① 判定 520 outside-diff chunk 的路径归属（legacy 接管 vs vanilla 回退，可从 log [LightRust]/fallback/WG_DOMAIN_INLINE 计数面判读）；② n=2 重跑（§5.2）判据修订：d_self>0 应集中表现为 to 集（625）与 outside 集的 run 间摆动；若 full 集 3078 d_self=0，坐实域批差为确定性 .b1 差。

---

## 9. §5.2 n=2 执行后更新（260919-07，主会话回传 + 本 worker 分解复算；仍 draft，灰区声明）

### 9.1 新事实（F）

- 主会话读数（`cmd-output/n2-mechanical-260919-07.txt`）：d_self = diff(r2,r3) = **2185（23.12%）**，100% 落 center 集、100% 落簇 A∪B、中带 0；d_cross_r2 = d_cross_r3 = **4223 且 key 集完全同一**；两 run SELFCERT 全绿 exit 0。
- 本 worker 分解复算（r1/r2/r3 light json × r1 to/full 明细，零采集）：**self=2185 = self∩to(r1)=300 + self∩fullOnly(r1)=1885 + self∩outside=0；self ∈ diffUniverse = 2185/2185**；cross_r2=4223 独立复现。

### 9.2 判读（I，中-高置信）

1. **预登记灰区声明（#154）**：H-b2-支持（|d_self|≪|d_cross|，实得 52%）与 H-b2-排除（d_self=0）均不机械成立。灰区根因是判据设计缺陷而非数据歧义：比率检验隐含「.b1 确定性成员差与 .b2 时序抖动可加分离」假设；实际结构是**乘性两层**——成员资格（哪些 chunk ≠ vanilla，4223，跨 run 完全稳定）× 值抖动（其中 2185 的具体值随 run 变）。比率检验对乘性结构无判别力，判据作废重订（§9.4）。
2. **成员资格层 = 路径决定、run 稳定（.b1 面）**：d_cross key 集逐 run 同一 + self 全部落在 4223 宇宙内 + 中带恒 0 → 「哪些 chunk 与 vanilla 不同」由执行路径决定且 run 间不变。44.69% 成员资格主因维持 .b1（域批路径全体 ≠ vanilla）+ 520 非域批确定性差。
3. **值抖动层 = 时序真实贡献（.b2 面升级）**：d_self=2185 ≠ 0 且 self∩outside=0 —— 抖动严格限于域批路径内部。两个关键读数：① **self∩to(r1)=300/625**：r1 部分域 chunk 仅半数跨 run 值稳定；② **self∩fullOnly=1885**：抖动深入「r1 满网格」集——满域装配对相同方块输入是确定性的（排序 centers、单次 JNI、每任务独立 out 缓冲，Mixin:602-704 无共享可变态），故 full 集 jitter 的唯一自洽解释是**run 间路径/装配指派变动**（同 chunk 在 r2 为满域成员、在 r3 为部分域/他批成员 → 值变）。即：时序决定的不是「是否有 diff」而是「每个 chunk 被哪个引擎/哪种装配计算」——.b2 以「指派不稳定」形态成为二阶贡献者，真实且非平凡（23.12%）。
4. **520 outside 面（新确认）**：self∩outside=0 + d_cross key 恒定 → 非域批路径 diff 为**确定性、时序无关**，归 per-chunk lightRust/vanilla 轴（§8.2-3），.b2 出清。
5. **.b2 辖区结论（修订终版）**：
   - 排除维持：「采集时点未 settled」「尾窗未 flush」「重载不收敛」（§3 不变）。
   - 升级确认：**时序 → 路径/装配指派 → 值抖动**链路真实存在（d_self=2185，23.12%），但抖动面（2185）远超单 run to 集（625）→ §4「部分域装配错位」不足以独占抖动解释，「满域/部分域/非域批指派变动」整体是更大抖动源。
   - 定位：44.69% = 「成员资格 4223（.b1/路径 + 520 per-chunk Rust，run 稳定）」⊕「值抖动 2185（.b2 时序指派，run 变）」。.b2 = 二阶贡献者（非主嫌、非可忽略）。

### 9.3 判别实验让渡（零采集，决定性）

对 r2/r3 log 各跑一次 §5.1 burst 提取得 toSet(r2)/toSet(r3)，检验：**self∩fullOnly(r1) 的 1885 是否 ⊆ toSet(r2)∪toSet(r3) ∪ centerSet 变动覆盖**。若成立 → 「指派变动」解释坐实（.b2 值抖动机制闭合为 candidate 共享者）；若存在大块 self∩fullOnly chunk 在两 run 均为满域成员仍 jitter → 同输入同路径不同值 = 内核/并发非确定源（loud，升级新候选；当前静态分析未见此通路）。

### 9.4 重订判据（预登记，替代 §5.2 灰区判据）

- **指派不稳定判据（.b2 candidate 门）**：selfSet ⊆ centerSet(r2)∪centerSet(r3) 且 self∩fullOnly 主体（≥80%）可由 toSet(r2)∪toSet(r3) ∪ centerSet 变动覆盖 → .b2「时序指派」机制 candidate。
- **内核非确定判据（新候选门，与上互斥）**：存在 ≥100 个 chunk 在 r2/r3 均为满域成员且自值不同 → 域批内核并发非确定源成立，fan-out 重开。
- **成员资格稳定判据（.b1 坐实门）**：diff(r2,vanilla) = diff(r3,vanilla) 逐 key（已满足）→ 成员资格路径决定，.b1 主嫌地位维持。

---

## 10. §9.3 检验结果与 fan-out 重开建议（260919-07，仍 draft；触发 §9.4 互斥门 B，报用户裁决）

### 10.1 新事实（F，主会话 `cmd-output/b2-assignment-test-260919-07.txt`，无配对 WARN）

- toSet(r1)=toSet(r2)=toSet(r3) **逐 key 完全同一**（625=625=625，to1Δto2=0、to1Δto3=0）—— 部分域成员跨 run 零变动。
- 检验A（§9.3 覆盖检验）= **0/1885**：fullOnly-jitter 完全不被 toSet(r2)∪toSet(r3) 覆盖 → §9.2-3「指派变动」解释**被证伪**。
- 检验A' = 300/2185；检验B = **1885/1885** 命中 §9.4 内核非确定门观察点（满域成员跨 run 同位仍 jitter，远超 ≥100 门限）。

### 10.2 判读（I）

1. **§9.4 门 B 正式触发，§9.2-3 的 .b2「指派不稳定」模型证伪**：to 集零变动 + jitter 主体为两 run 满域成员 → 时序既不改部分域成员、也无可检测的路径指派变动，但同 chunk 同路径不同值。.b2 原三机制（未 settled/未 flush/装配错位选择器）均不足以解释。
2. **Rust 内核静态确定性复核（本 worker 新证，F）**：`light_compute_domain_impl` 全程 per-call 局部（opacity25/col_max25/seeds25/Scratch，worldgen-core/src/light/mod.rs:456-473），签名 `&LightEngine`（:401）+ `&mut [u8]`，无 static/Atomic/lazy 可变态；LightEngine 仅 `table: Vec` + `air_fast`（:68-83，scratch 已移除）→ **同 blocks25 ⇒ 同 out，内核层确定性成立**（逐位等价单测 :866-900 背书）→ 「同输入不同值」在内核层不成立，jitter 必来自**输入不同**或**写回后值被改**。故门 B 的正确归解不是「内核并发非确定」，而是 §10.2-3 的二选一。
3. **重开候选（.b4「light 值 run 间抖动源」，两子候选）**：
   - **.b4a（Java 写回后覆写，静态先验较高）**：域批写回 enqueueSectionData + `setLightOn(true)` + releaseLightTicket（Mixin:590-595）使 chunk 重新对 vanilla 光照引擎武装；后续邻块加载触发的 vanilla 跨块 light update 会以 vanilla 引擎覆写 Rust 已写 nibble（尤其边界带），覆写集随 run 间加载次序变 → 值抖动且限于域批 chunk。自洽解释：抖动 100% 限于 center 集、中带为 0、cross 成员资格稳定。
   - **.b4b（输入快照 run 间不同）**：blocks9/packed 快照在提交线程采集（Mixin:540-567），若采集时点相对邻块状态存在 run 间差 → blocks25 不同 → 出值不同。静态未见通道（预检要求 3×3 到 FEATURES），低先验未清零。
4. **.b2 辖区终局**：时序/瞬态/settled 轴对 44.69% 直接贡献**排除**；原 .b2 候选关闭，让位 .b4。

### 10.3 给 parent 的 fan-out 重开建议（报用户裁决）

- **建议重开**：新候选 **.b4「light 值 run 间抖动源」**（.b4a Java 写回后 vanilla 覆写 / .b4b 输入快照 run 间差），与 .b1（成员资格主因）、.b3 残余并行归因；520 per-chunk lightRust 差维持独立轴。
- **判别实验（低成本优先序）**：
  1. **零采集（最优先）**：.b4a 预测抖动集中于 chunk 边界带 section、内部 section 稳定——重解析两 run region mca 做 per-section hash 即判（主会话按 #161 重走 world 身份链；顺带出清 #188 通道盲区）。
  2. **n=2 基线臂**：vanilla 臂 n=2 与 lightRust-only（-PlightRust=1 无 domainbatch）臂 n=2 各 self-diff 一次——两者≈0 → 抖动为域批路径特有；lightRust-only 也抖 → 覆写/输入问题外溢到 per-chunk 接管路径，重定边界。
  3. **代码级探针（需改码，批准后）**：task 行增 blocks25 输入 hash（LIGHT_BETA wgBetaHash 同款通道）→ 直接分辨 .b4b（输入 hash 不同）vs .b4a（输入同值不同）。
- **判据前置**：沿用 #118 正负成对自证 + #161 world 身份链 + VOID 非零退出；.b4a 建议预登记「边界带 section 抖动占比 ≥80% 且内部 section ≤10%」形态门。
