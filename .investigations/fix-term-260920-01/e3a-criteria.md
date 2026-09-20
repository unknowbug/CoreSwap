# e3a-criteria（档③ E-3a 输入 hash 判别，260920-01 定稿）

---
status: candidate-预登记（判据先于采集，#112 mtime 序）
block: fix-term-260920-01
设计源: .investigations/fix-term-260919-08/b5-fa-branchB.md §1（已批准，用户 2026-09-20）
---

## 0. 口径定稿（相对 b5 §1.2 草案的落地适配，采集前写死）

1. **打点实参形状（b5 §4.2 待核项已核）**：域批臂输入 = `blocks25`（25×16×16×384 int，
   Mixin:713-737 由 9 邻 chunk blocks9 拼装）；**批级共享一个 blocks25**（3×3 中心一次 JNI）→
   `[E3A]` 行的 `blk25=` 为**批级 hash，同批 9 中心同值**——判读按 (cx,cz) 对齐时该字段含义 =
   「该中心所在批的输入窗口」。
2. **输出 hash 口径**：该中心输出段 `[k*98352, +98352)` 整段 sha256（block 24×2048 + sky 24×2048 +
   flags 48），单 `out=<sha8>` 字段——**非** b5 草案的「24 节 csv」。理由：分支判读只需等值比较；
   空间指纹（K3 探针）需 24 节分布时另立项，不在本档。
3. **输入 hash 口径**：blocks25 int[] 逐元素小端字节序 sha256，取前 4 字节 8 hex。仅用于同口径
   跨 run 等值比较（§9.7：载体 = 打点面 blocks25/输出段；**与 region .mca 口径不同粒度，经 P09 互验
   挂钩后方可引用**）。`out=` = 中心段 98352B 原始字节（含 flags）整体 sha256；
   `nib=` = flag 归一化后 48×2048 字节流 sha256（region 同构口径，P09 专用）。
4. **seed 身份**：行级不带 seed；由采集台 world 身份链承载（driver SEED 常量 + level.dat
   seed_be 校验 + region mtime > rmtree cutover，run 脚本 [WORLD-ID]）。
5. **判别域**：blocks25 域批臂（`LIGHT_PACKED` 关）。packed 路与内联路（Mixin:829-837 同构副本）
   不打点——覆盖面限定（与档②一致），非遗漏。
6. **FB-2 隔离性论证（260920-01 新判读，采集前提）**：`[E3A]` 行在**写回时点**打印（enqueue 前），
   FB-2 补传播与 vanilla 增量覆写（档②已裁决的 2185 抖动面）均发生在打点**之后** → FB-2 在
   （当前 HEAD 语义）不污染 E3A 打点数据；r5/r6 在 HEAD `95a6039`+E-3a 打点 commit 上采集，
   不回退 FB-2。
7. **dll sha 前提修正（交接勘误候选）**：b5 §1.6「打点致 dll sha 漂移」判读为**不成立**——打点面
   纯 Java mixin + gradle 映射行，不触 Rust 源。SELFCERT dll 项仍按 cc4e39fe 校验；若实测漂移
   （≠预期）则按 §15.1 申报 premise-expired 并停线。**实证 = 两臂采集 log 的 dll sha 读数。**

## 1. 前置集（preconditions，P01-P10；任一失效 → 全体判据 suspended，比较器 exit 1 [VOID]）

| # | key | expected | check（机械） |
|---|-----|----------|---------------|
| P01 | done+driver | ≥1 / ≥1 | run log `Done (` 计数、`[FP-DRV] ev=move seq=5 ` 计数 |
| P02 | lightInit | ≥1 | `lightInit ok` 子串 |
| P03 | hook armed | ≥1 | `[LIGHT-DOMAIN] hook armed` |
| P04 | dll sha | `cc4e39fe` | log `sha256=(\w{8})` 实测对拍（见 §0.7 修正预期） |
| P05 | world seed | 8576294172403134396 | level.dat seed_be + driver 常量一致 |
| P06 | world 身份链 | PASS | region 全部 mtime > rmtree epoch（#161） |
| P07 | E3A 行完备性 | 缺口率 ≤1% | **P07v2（260920-01 amend，先于 r6 采集与判读；v1 规格错误：误把 region 全部 lit chunks (9450) 当打点应覆盖面）**：打点面 = 域批写回面（`[LIGHT-WB] pre` 计数，r5 实测 3703；vanilla 直亮 chunk 不走域批无打点行）→ 机械 check = 两臂 `[E3A]` 行数 ≥ 0.99 × 各臂 wb_pre 计数；`degraded>0` 的 chunk 豁免并逐个登记 |
| P08 | E3A 开关生效（#81 行为化） | ≥1 | `[E3A]` 行存在即开关生效的行为化证据；vanilla 臂/无开关 run 不得出现 |
| P09 | 打点面↔region 互验 | ≥100 chunk 命中率 ≥70% | `nib=` 字段（flag 归一化 48×2048 字节流 sha256，Y 升序）与 region `.mca` 判读侧复刻 hash 对比；复刻含 #46 隐式键语义（BlockLight 缺键=全 0、SkyLight 缺键=全 15）。**阈值声明**：不命中源 = 写回后 vanilla 覆写/增量光照面（档②实测 ≈23% jitter），故 70% 预登记（非 100%）；不命中 chunk 逐个登记不静默 |
| P10 | 恒定面 key 集可复算 | 集合非空 | cmp_t8.py 重算 t8-domain-r2/r3 light json → d_cross 恒定 key 集（≈2038 面），与 verdict-260919-07 §1 读数对账 |

## 2. 分支读法（写死，#112——采集前定稿，禁止事后挑读法）

比较器按 (cx,cz) 对齐两臂 `[E3A]` 行，S = 双臂均存在的 chunk ∩ P10 恒定面 key 集：

| 分支 | 机械条件 | 读法（写死） |
|---|---|---|
| **乙直证** | ∃ chunk ∈ S：blk25 同 且 out 异 | 输入同而输出异 → **乙读法（确定性内核/边界语义差）获数据层直证**；甲读法对该 chunk 的「终态化固化」解释被证伪 |
| **甲/上游差支持** | ∃ chunk ∈ S：blk25 异 | 输入异 → 差异源在上游（快照时机/域批几何/邻域装配）；乙对恒定面被削弱，甲与上游差家族获支持空间 |
| **混合** | 两分支成员集均非空 | 分账式登记（逐 chunk 归属计数落盘），交 judge 强制复核 |
| **不可判** | S 与恒定面交集 <80%，或空载体 | exit 5 `[NO-JITTER]` 同构：不是 PASS，登记「判据不可判（数据有效）」，决策矩阵走 b5 §3 保守默认行 |

- 等价档位（§9.7）：**E1**（同构建态双 run，单 seed 单维度 overworld，1.20.1 域批臂专属）；
  不外推 E2/E3、不外推 1.21.6。
- 无效声明对照（#162）：本档不做定性未抽样结论；out= 与 region 口径经 P09 挂钩前不互称；
  blk25 批级共享口径已显式声明（同批同值 ≠ 同 chunk 独立输入）。

## 3. 采集协议

- 臂：r5、r6 = 同 seed、同 dll（cc4e39fe 预期不变）、同 FP-DRV、`-Pe3aInputHash=1` + fix-domain
  臂全开关（lightRust/domainbatch/postFinalize/latesubmit=merge/wbprobe），keep-world 协议
  （r6 前 rmtree 重建 world；失败轮换新 tag 留档，#144/#146 禁自动回退）。
- 驱动：改造 run_b1.py → `run_e3a.py`（新目录 .tmp/e3a-260920-01/，日志/归档不复用旧台）。
- 四件套：本判据（mtime 先于采集）+ `e3a_cmp.py`（只读输入、只写 --out json，VOID 即 exit 1）
  + 采集 raw log 落 .tmp/e3a-260920-01/ + 结果 json。执行 = 主会话（subagent 无 shell），
  原始输出落盘交解读。

## 4. §9.8 副作用与逆

| 副作用 | 逆 |
|---|---|
| r6 rmtree world | 逆 = 两臂 region 归档 regions/r5*/r6* + 采集 log；无自动回退 |
| server.properties level-seed | 逆 = .bak-formprobe（既有） |
| Mixin 打点块 + build.gradle 映射行 | 逆 = 单 commit revert（`feat(diag): e3a input hash probe`） |
| 判据/比较器/采集产物 | derived，天然合规 |

## 5. amend 记录（判读后回写，原文不改 + 加注；judge N2 要求）

- **P09v2（260920-01 amend，judge review-260920-01 N1/N5 处置）**：P09 v1（region 互验 ≥100 chunk 全等 → 后改 ≥70% 命中）判据前提不可实现——复刻口径经两轮缺陷修复（.mca blob 8192B header 偏移、NBT Y 无符号字节）后仍 0/300 命中；r5/r6 region 终态自差 2581/9450=27.3%，证伪「稳定 chunk 的 region 值 = 写回输出值」前提（vanilla 增量覆写 + 收敛重算全域发生）。**处置 = 通道降级关闭，nib= 保留原始数据不进判据**；复刻代码残余 bug 可能性不排除（idk-E3a2，两可能同处置）。⚠️ judge N1：该 amend 实质改写「前置失效 → 数据不进判读」语义，高风险——缓解 = 总体结论（不可判）未因此改变、1518 子集信号仅登记为数据层证据不构成判读依据。
- **P11（260920-01 新增，判读后登记）**：转录/解析稳健性核对——① nib/out 转录一致性（nib 同 ⇔ out 同，逐 chunk 两臂）：0 违例 / 3703；② 批结构 blk25 共享性（同批行 blk25 同值）：近似 PASS（非均匀组 36/459 = 并发任务行交错分组假象，非打点缺陷）。
- **稳健性声明（judge N6）**：coverage 判定（74.48% < 80% → 不可判）对 P07 v1/v2 口径均不敏感——v1 口径下 P07 本身即失败（rows 3703 < 99%×9450），不存在「换口径才得出不可判」的路径。
