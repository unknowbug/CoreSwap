# 档②判别实验预登记判据（C1 备选分解判别：终态化固化 vs 确定性语义差）— c1-criteria

---
status: draft
block: fix-term-260919-08
date_label: 260919-08（判据定稿先于 r4 采集，mtime 序可核 #112）
discriminates: C1 备选分解（.artifacts/t8-attrib-260919-07/verdict-260919-07.md §1.1 ⚠️）
arms: 臂1 = r3（已归档 region）× 臂2 = r4（待采 keep-world）
comparator: .tmp/t8-keepworld-260919-08/o1_o3_cmp.py（与本判据同批定稿，#112）
---

## 0. 判别目标（一段话）

verdict-260919-07 §1.1 confirmed 主体附带 **C1 备选分解**：当前读法「恒定 2038 = 写回终态化固化（Mixin:594-595）」与备选读法「恒定 2038 = 确定性 Rust-vs-Java 内核/边界语义差（+2185 = 覆写）」满足全部现有观测、未判别分离。档②用 keep-world 双跑（r3/r4）做 **per-section 光数据对比**，以 O1（覆写面形态门）+ O3（朝 vanilla 方向的符号统计）对两读法给出支持/削弱裁决输入。本判据为**预登记文本**，读法在采集前写死（#112）；执行后禁止事后挑读法（#159 三条件之外不展开）。

## 1. preconditions 表（v0.22 §15.1；任一失效 → 本档全体判据 suspended → 比较器 VOID 非零退出，#160）

| # | key | expected | check（机械） |
|---|-----|----------|--------------|
| P01 | r3_region_exists | 18 个 .mca | `regions\r3\` 内 `.mca` 文件数 ≥1（实测基线 = 18，采集日志 `.tmp/legacy-sweep-260919-06/t8-domain-r3.log`，当时 SELFCERT 全绿 exit 0） |
| P02 | r3_selfcert_lightinit | ≥1 命中 | r3 log 含 `lightInit ok` |
| P03 | r3_selfcert_hook | ≥1 命中 | r3 log 含 `[LIGHT-DOMAIN] hook armed` |
| P04 | r3_selfcert_dll | sha8=cc4e39fe | r3 log 含 `sha256=cc4e39fe` |
| P05 | r3_selfcert_seed | 8576294172403134396 | r3 log 含该 seed 串 **且** `regions\r3\..\level.dat`（若在归档内同级存在）NBT 实读 Data.WorldGenSettings.seed 相等；level.dat 不在归档 → 以 log SELFCERT 项代替并**声明降级**（b5 §4.2 预登记形态） |
| P06 | r3_selfcert_fpdrv | move5 | r3 log 含 `[FP-DRV] ev=move seq=5` |
| P07 | r4_region_exists | .mca 数 ≥1 | r4 跑后归档 `regions\r4\`，`.mca` ≥1 |
| P08-P12 | r4_selfcert_*（lightinit/hook/dll/seed/fpdrv） | 同 P02-P06 | r4 log（`.tmp/legacy-sweep-260919-06/t8-domain-r4.log`）同款子串命中 |
| P13 | world_identity_r4 | r4 全部 .mca mtime > r4 rmtree 时刻 | 比较器 `--r4-cutover <ISO>` 传入 rmtree 时刻；全部 r4 region mtime 晚于它 → pass。**读法写死：未提供 `--r4-cutover` = 身份项不可机械核对 = VOID**（#161：world 身份链是前置，不是可选项） |
| P14 | snap_crosscheck | 缺口率 ≤1% | 两臂 region 解析 chunk 数 vs `t8-domain-r3-light.json` / `t8-domain-r4-light.json` chunk 数交叉核对；且逐 chunk 重算 total-hash 与 json 全等比对，mismatch 率 >1% → VOID（#53 四步自检 + snap 互验同构 b5 E-1a） |

前置失效的后果（写死）：比较器 exit 1（VOID），输出 `[VOID]` 行 + 失效 key 清单；**数据不进任何判读**，判据 suspended、依赖本判据的结论标 premise-expired，status 永不自动变更（§15.4）。

## 2. 主判据 O1 — 覆写面形态门（per-section hash 集合级）

### 2.1 分析单元与 jitter section 定义（写死）

- 分析单元 = **light section**（1.20.1 = 16×16×16 nibble 数组，2048 字节；以解析器实得结构为准，若实得为 8×8×8 粒度则坐标换算按实得登记，不重写门逻辑）。坐标三元组 (cx, cz, Y, channel∈{sky,block})；chunk 定位 = region 文件名 (rx,rz) + 槽位 i → `cx=rx*32+(i&31), cz=rz*32+(i>>5)`。
- **候选节** = 该 (cx,cz,Y,channel) 在 r3 与 r4 **均有非空 light 数组**。
- **jitter section** = 候选节中两臂数组 hash（sha256 全节字节）不同者。
- **stable section** = 候选节中 hash 相同者。
- **单臂节**（仅一臂有数据的节）**不计入** jitter 也不计入 stable；逐个清点落盘（其数量改变不了门，但改变读数域声明）。**jitter 集 = 空 → 门不可判**：不是 PASS（空载体不报通过，#130 家族），输出 exit 5 + `[NO-JITTER]`，登记为「判据不可判（数据有效）」。

### 2.2 边界带 / 内部定义（写死，逐字节级）

对每个 jitter section，比较器直接对比两臂 nibble 数组**逐字节**（每字节含高低 nibble 两个光值），对每个**值不同的 nibble** 取其节内局部坐标 `x = idx&15, z = (idx>>4)&15`（nibble 线性索引 `idx = (y*16+z)*16+x`，与 1.20.1 ChunkNibbleArray 布局一致；不一致时以实得布局换算并登记）：

- **边界带 nibble**：`x ∈ {0..14} ∪ {16..31} 的轴带命中` —— 即 `x ≤14 或 x ≥16` **或** `z ≤14 或 z ≥16`（b5 §1.2 预登记的 {0..14}/{16..31} 两侧带）。
- **内部 nibble**：`x == 15 且 z == 15`（两轴带集的补的交）。
- **垂直方向（b5 §1.2 登记）**：不作门轴。sky 光柱整列可动 → 垂直无独立内部带；仅输出 jitter 的 Y 分布直方图作方向旁证（预测：与水平边界带相交的 Y 层集中），无阈值、不参与判定。

⚠️ **诚实声明（预登记即声明，不事后补）**：按上述写死的轴带集合，`band ∪ interior` 覆盖全部 nibble 且 `interior` 仅 (15,15) 一列——band≥80% 子句在此读法下近乎恒真，**实质约束是 interior ≤10%**。保留 band 子句是为忠实登记 b5 §1.2 原形态（#112 不改写预登记数字）；判读者不得据 band% 单独宣布通过。

### 2.3 门与两分支读法（#112：两分支都预登记）

设 `B` = 全部 jitter section 的异值 nibble 总数，`b` = 其中边界带命中数，`n` = 内部命中数。`band% = b/B`，`int% = n/B`。

| 分支 | 机械条件 | 读法 |
|---|---|---|
| **.b5a/O1 门通过** | `B>0 且 band% ≥ 80% 且 int% ≤ 10%` | 抖动集中边界带 → .b5a（写回后 vanilla 增量覆写）candidate 门成立；R6（跨线程可见性）维持低先验 |
| **.b5a 削弱 / R6 升位** | `B>0 且（band% < 80% 或 int% > 10%）` | 抖动弥散到内部 → .b5a 削弱；.b5b 唯一残余 R6 升位（与 O1 互斥可判，b5 §2-R6） |

分支覆盖：`B>0` 时两分支对实数轴完备划分（band%/int% 必落其一），**无灰区**（#154 全数轴覆盖声明）；`B=0` 已由 §2.1 NO-JITTER 分支显式归属。退出码：通过 = 0；削弱 = 2（数据有效、判据 FAIL 级非 VOID）。

## 3. O3 — 朝 vanilla 方向的符号统计（需 vanilla per-section 参照）

### 3.1 机械定义（写死）

前置：`--vanilla <region目录>` 提供 vanilla 臂 per-section 数据（同一解析器解析）。

- 对每个 jitter section j（两臂均有数据的候选节），取 vanilla 同坐标节数组 v：`d3(j) = Σ|nibble(r3,j) − nibble(v,j)|`（L1 距离），`d4(j)` 同理。
- 参照基线 `D_stable` = 全部 **stable lit section**（候选节中 hash 相同者）的臂-venilla L1 距离集合（两臂各算，取逐节均值入集合）；取 `med = median(D_stable)`（「典型 Rust-值 vs vanilla 距离」的代理；Rust 原始值无独立观测，此为预登记代理，声明代理性）。
- 统计量：`f = #{j : min(d3(j), d4(j)) < med} / |jitter|`。读法依据（b5 §1.4-O3）：覆写是朝 vanilla 方向的修正 → 被覆写 run 的值应比典型 Rust 值更接近 vanilla；两 run 至少一者被覆写。

| 分支 | 机械条件 | 读法 |
|---|---|---|
| O3 支持 .b5a | `f ≥ 60%` | 抖动节至少一臂显著比典型 Rust 值更近 vanilla → 覆写方向性成立，与 O1 通过合读为 .b5a 双门齐过 |
| O3 反对 .b5a | `f ≤ 40%` | 抖动节并不更接近 vanilla → 「朝 vanilla 修正」方向性不成立 → .b5a 再削弱、确定性语义差读法相对升位 |
| **灰区（显式归属，#154）** | `40% < f < 60%` | 判「O3 不可判」，不作任何方向的证据；仅登记数值 |

### 3.2 vanilla 参照缺失分支（预登记降级读法）

盘面核对（260919-08）：vanilla 参照臂 region **未在盘归档**（`regions\` 仅 r3；vanilla-r1 world 已被 rmtree）。→ 缺 `--vanilla` 时比较器进入 **O1-only 模式**：O1 照常出门（O1 只需两臂）；O3 输出 `[O3-DEGRADED] vanilla per-section 参照缺失，O3 不可判`，**读法写死：O3 缺失 ≠ O3 失败**，不得被任何下游读作「O3 反对 .b5a」。若后续主会话归档 vanilla region（新采集 = 重走 #161 + 本判据前置集，不得复用旧 world），补跑 `--vanilla` 升级为双门模式。

## 4. 判别结论映射表（O1 × O3 → C1 两读法）

C1 两读法：**读法甲「终态化固化」**（恒定 2038 = 提交时快照被 setLightOn(true) 固化）；**读法乙「确定性语义差」**（恒定 2038 = Rust-vs-Java 确定性内核/边界语义差 + 2185 = 覆写）。

| O1 | O3 | 对读法甲（终态化固化） | 对读法乙（确定性语义差） | 备注 |
|---|---|---|---|---|
| 通过（边界带集中） | 支持（f≥60%） | **支持**（覆写面形态+方向双证，甲的覆写子面 candidate 门成立） | **削弱**（乙需把 2185 归确定性源，边界带集中+朝 vanilla 方向与其不符） | 双门齐过 = .b5a 最强形态 |
| 通过 | 反对（f≤40%） | 部分支持（形态证覆写存在；方向异常登记 open——覆写不必单调朝 vanilla，登记后 judge 重议） | 部分支持（方向面未按预测） | 形态/方向分裂 = 预登记外的形态分裂，按 #159 三条件展开，不越界读 |
| 削弱（弥散） | 反对/灰/缺 | **削弱**（2185 难归覆写子面；甲的恒定 2038 面仍由 d_cross 集恒定独立支撑，不被本档推翻） | **相对升位**（R6/确定性源承载抖动的解释空间扩大；但乙**不被证实**——本档无乙的正面证据通道） | 需档③ E-3a 输入 hash 直证接手 |
| 削弱 | 支持 | 矛盾形态：按 #159 逐臂展开 + judge 强制复核，不机械归边 | 同左 | 两门方向冲突本身是登记项 |
| NO-JITTER（B=0） | — | 两读法**均不裁决**（2185 未复现即 r4 与 r3 抖动集不同，先查 P13 身份链与采集同构性，#161） | 同左 | exit 5，非通过 |

**「两者并存」分支（写死声明）**：O1 通过只裁决 **2185 抖动面的机制归属（覆写 vs 非覆写）**，对 **2038 恒定面**（甲的终态化固化 vs 乙的确定性语义差）**不直接裁决**——恒定面的判别通道是档③ E-3a 输入 hash 直证（需改码批准）。故任何 O1/O3 组合都**不构成**把 verdict §1.1 主读法升 confirmed 的充分条件；O1 通过 + O3 支持 = 覆写子面 candidate 门成立（对乙的「2185=覆写」侧亦无削弱——乙本身含覆写项，被削弱的是乙对 2038 面的解释必要性，该步归档③）。

## 5. §9.8 副作用与逆登记（v0.22）

| 副作用 | 性质 | 逆（独立字段，不并入 source） |
|---|---|---|
| r4 采集 run_t8.py rmtree 重建 world | 破坏性 in-place | 逆1 = r3 region 已归档 `regions\r3\`（18 文件，判据输入可回溯）；逆2 = r4 跑后归档 `regions\r4\`；**无自动回退**（#146：失败轮证据价值更高，禁止 rmtree 后回滚尝试） |
| server.properties level-seed 改动（采集链固有） | in-place | 逆 = `.bak-formprobe` 备份件（既有） |
| 本判据/比较器落盘 | derived（新命名产物） | 天然合规 |
| 无逆可登记面 | — | 无 |

## 6. 覆盖面 / 外推边界（#195 第四件）

- **E1 档内判别**：r3/r4 为同构建态双 run（#162/E1），本档一切结论**不外推** E2（跨执行体 Java↔Rust 对拍）与 E3。
- **单 seed**：8576294172403134396；**单维度**（overworld 域批）；n=2（r3/r4）——不外推其他 seed/维度/更多 run。
- **1.20.1 域批臂限定**：结论只覆盖域批写回通路（Mixin:582-596）；1.21.6 无域批（scout-map §5），无对应物；内联接管路（:829-837）不在本档判别域。
- per-section hash 载体声明（#162 口径三要素）：载体 = region anvil 内 light nibble 数组（sky/block 各 16³×4bit）；覆盖面 = 两臂 region 实存 chunk 的候选节；与既有 per-chunk light json 口径**不同粒度不可直比**（per-chunk 16hex 只作 §1-P14 交叉核对用，不作差异统计用）。
