# .b5 域批 chunk light 值 run 间抖动（d_self=2185）源 — 重开候选（260919-07，draft，静态分析）

- 角色: fan-out worker .b5（子候选 .b5a = Java 写回后 vanilla 覆写 / .b5b = 输入快照 run 间差）
- 方法: 纯静态（一手源码直读 + 前置产物继承）；沙箱无 shell，未跑任何命令。
- F = 一手源码/日志直读；I = 推断，分开标注。status: **draft**。
- 前置已读: scout-map.md / b1 §10-§11 / b2 §9-§10 / b4 §8-§9 / cmd-output 三份读数。
- 一手源码核对点：ServerLightingProviderMixin.java:530-576（提交）/ :582-596（写回）/ :598-744（任务体）/ :746-892（legacy 接管 + light() HEAD）/ :894-903（nibble 构造）；LightDomainBatch.java 全文（1-134）。

---

## 0. 辖区结论（先答）

**.b5a（写回后 vanilla 增量覆写）静态先验维持高位且细化成立：写回链 Mixin:594-595 `setLightOn(true)` + `releaseLightTicket` 把 Rust 已写 nibble 的 chunk 以 lit 态交还 vanilla 光照存储，该 chunk 从此免于全量重光（终态化 → 恒定 2038），但其 section 已入 vanilla LightStorage，后续邻居加载触发的 vanilla 增量传播会**就地改写**边界带 nibble（覆写集 = 距后加载邻居 <15 格的边界带 → 抖动 2185 且限域批 chunk）。成员资格逐 run 恒定由「内部 Rust 值逐 run 确定 + fullHit=3078/3078（内部本身即 ≠ vanilla）」保证，无需指派变动假设。**
**.b5b（输入快照 run 间差）静态出清：全部候选通道逐一排除或压至单条低先验残余（跨线程可见性，§2-R6），不构成主解释。**
**与 .b1 终态化假说裁决：同一机制（写回终态化通路，Mixin:582-596）的两面，合并表述成立（§3）。**

---

## 1. .b5a 静态细化（辖区 1）

### 1.1 通路一手源码链（F，file:line）

1. **写回**（Mixin:582-596 `wgLightDomainWriteBack`）：对每 center 逐 section（24 节 × BLOCK/SKY）`provider.enqueueSectionData(...)`（:590-591，nibble 由 out 缓冲拷出，:894-902 三态 flag）→ **随后** `chunk.setLightOn(true)`（:594）+ `wgReleaseLightTicket(chunkPos)`（:595）。
2. **语义复刻声明（F，注释 :593）**：这两行显式复刻 vanilla `light()` 尾部 POST 语义（yarn L179-183）。即写回后的 chunk 在 TACS 眼中与「vanilla 自己算完 light()」**不可区分**：lightOn=true。
3. **终态化效果（F 代码语义 + I 引擎语义）**：vanilla 管线对 `isLightOn==true` 的 chunk 不再调度 `light()` 全量重光（light() 只对未 lit chunk 调用；本工程 LegacyTakeover :835-837 同款复刻即为此语义的镜像）→ **Rust 写入的值（含错值）从此无全量修正路径**。
4. **覆写效果（I，引擎语义参照；yarn LightStorage/ServerLightingProvider 不在工作区，诚实声明同 .b2 §3.3）**：`enqueueSectionData` 把 nibble 放入 ServerLightingProvider/LightStorage 的 section 存储；vanilla 光照引擎的**增量更新**（邻居 chunk 加载完成时 TACS 对光照提供者的 `setLightEnabled`/`updateSectionStatus`/边邻居 skylight 传播调度，1.20.1 常规行为）作用于**同一份 section 存储**——它们不检查「这份数据是谁写的」，只按方块/邻居状态做 increase/decrease 传播，直接 set 已存 nibble 的值。→ Rust 值可被 vanilla 增量传播**就地部分改写**。
5. **触发条件（I，由 1-4 推出）**：① 该 chunk 已被域批写回（lit）；② 其**邻居 chunk 在写回之后**才到达光照相关调度点（加载/enable/传播触发）；③ 存在可传播的光梯度（邻块侧有光而本侧不同值）。加载次序由 worker 线程调度决定 → ② 逐 run 有抖动 → **覆写集逐 run 不同 → 值抖动**。

### 1.2 覆写集预期空间形态：边界带 vs 内部（I，可证伪形态预测）

- vanilla 跨 chunk 传播的单步尺度受 light 级 ≤15 约束：从邻居 chunk 侧传入的光最多影响本 chunk **距边界 <15 格**的块列/section → 预期覆写集中在**水平边界带**（局部坐标 x∈{0..14}/{16..31} 侧、z 同理，面向「后加载」侧），不达 chunk 内核（距边 ≥15 的 2×2 块心区域不受任何单侧邻居传播触及）。
- 垂直方向：sky 光柱列在 xz 边界带内整列可动 → 预期抖动 section = 与边界带相交的 section；全空/全 15 均质 section（flag 1/2 路径，:894-898）对小幅传播可能不敏感或整体翻转，形态判读以 per-section 重解析实测为准，此处只登记**方向预测：边界带集中**。
- 与既有观测自洽（F+I）：抖动 100% ⊆ center 集（未 lit chunk 无 Rust nibble 可覆写，unlit 侧零波动，d_self∩outside=0）、中带 0（中带非域批 chunk）、d_cross 集恒定（见 1.3）。

### 1.3 为何成员资格（d_cross key 集）仍逐 run 恒定（判别力核心，F+I）

- F：fullHit = 3078/3078（b2 §8.1）——**每个** center chunk 的 Rust 值都与 vanilla 臂不同。即 diff 成员资格由「内部 Rust 值 ≠ vanilla 值」决定，而内部值是同输入确定函数（b2 §10.2-2 内核确定性复核）。
- I：边界带覆写只改 chunk hash 的**输入内容**，不改变「该 chunk hash ≠ vanilla 臂 hash」这个布尔——内部差异已足以保证两者不等 → **无论覆写发生在哪次 run、覆写多少，chunk 都留在 diff 集内** → d_cross key 集逐 key 恒定（实测 r2/r3 同一）。
- 反证方向：若某 center 的 Rust 内部值恰好与 vanilla 相等（fullHit<100%），覆写抖动才能改变成员资格。实测 0 例 → 模型无张力。

### 1.4 可观测推论（≥2 条）

- **O1（形态门，需档②新采集）**：对两 run 的 center chunk 做 per-section hash 对比，jitter section 占比在**边界带 ≥80%、内部 ≤10%**（沿 b2 §10.3 预登记门）。若 jitter 弥散到内部 section → .b5a 削弱、.b5b 残余（R6）升位。
- **O2（成员恒等门，已满足）**：d_cross key 集 r2/r3 逐 key 同一（n2-mechanical 实测 True）+ d_self ⊆ center 集 → 与 1.3 联合为模型必要条件；任何未来 run 出现 d_cross 集漂移或 d_self 触及非 center chunk → .b5a 证伪。
- **O3（幅度/位置定性，需档②）**：抖动块的域臂值在两 run 中应都**更接近 vanilla 臂值**（覆写是朝 vanilla 方向的修正）而非随机漂移——可由 per-section 值差符号统计检验。
- **O4（日志旁证，零采集部分可行）**：抖动应偏集中于**加载尾波/簇边缘**（邻居后到概率最高处）——r1 中 timedOut 主体与簇B 尾波重合（b2 §2.2 样例）方向一致；定量核验需 center 时间戳×抖动集叠合（档②采集时顺带记录）。

---

## 2. .b5b 出清：输入快照 run 间差候选清单（辖区 2）

提交侧事实（F）：blocks9 收集在 light() 调用线程（:562-567，`wgLightCollectBlocks` + 立即 `Arrays.copyOf`）；预检要求 3×3 邻居全部到 FEATURES（:520-528）；LIGHT_PACKED 关（task 行无 packed=，b2 §4.1）→ blocks25 装配路（:669-688），快照内容决定 blocks25，装配序确定（centers 排序 :602）。

| # | 候选源 | 裁决 | 依据/可观测推论 |
|---|---|---|---|
| R1 | 邻居容器未冻结（FEATURES 后仍变） | **排除**（F 语义） | worldgen 中 chunk 到 FEATURES 后方块态已定（后续阶段不写 blocks），内容 = seed 确定函数 → 跨 run 同值。推论：档③输入 hash 跨 run 应相同。 |
| R2 | ThreadLocal 缓冲竞态 | **排除**（F） | :567 提交前即拷贝快照；任务线程只读 st.blocks9s（LightDomainBatch:45,77）。 |
| R3 | 装配/合并序不定 | **排除**（F，b2 §10.1 已实证） | centers 排序确定 + to/full 集三 run 逐 key 同一 → 装配无 run 间变动。 |
| R4 | dup 重提交改变输入 | **排除**（F 代码） | DUP 只返回既有 future（LightDomainBatch:73-80），不改快照、不重算。 |
| R5 | packed/palette 编码 run 间差 | **排除**（F） | LIGHT_PACKED 未开；blocks9 直接读 PalettedContainer 值域。 |
| R6 | 跨线程可见性（采集线程读生成线程写入，无显式 happens-before 边） | **未清零（唯一残余，低先验）** | 静态无法证明 JVM 可见性时序；但 JMM 下无同步边的读即使可见也只会看到**更旧**的邻居态（更早世代），而邻居在 light 提交时已过 FEATURES 状态门（状态门本身经 AtomicInteger/同步，构成间接同步边——I，倾向已覆盖）。可观测推论：若真实存在，抖动不应呈边界带形态（整窗 98304 字节任一处可异），且档③输入 hash 会逐 run 不同 → 与 O1 互斥可判。 |

**出清结论（I）**：.b5b 无静态存活通道承载 2185 量级抖动；唯一残余 R6 与 .b5a 的形态预测互斥，档②/档③一次采集即可同时裁决。

---

## 3. 与 .b1「快照时点终态化」假说的关系裁决（辖区 3）

**裁决：同一机制的两面，合并表述（非互斥）。**（I，中-高置信）

- 共同源头是**同一行代码对**：Mixin:594-595。终态化面 = `setLightOn(true)` 关闭全量重光 → 「提交时快照 vs 终态世界」的差**永久固化** → 恒定 2038（错集规则稳定）；覆写面 = 写回值已入 vanilla LightStorage → 增量传播**部分改写**边界带 → 抖动 2185（错值时序漂移）。两面由同一写回动作同时开启，联合恰是 b1 §10 要求的「错集恒定 + 半数错值漂移」唯一存活形态（③）。
- 若硬按互斥假说判别（判别点备查）：① 「纯终态化、无覆写」预测 d_self=0（已被观测证伪）；② 「纯覆写、无终态化」预测 vanilla 后续修正最终收敛到 vanilla 值 → d_cross 集应随 run 推移缩小/漂移（未观测，集恒定）；③ 「两面合并」预测 = 现观测全集。→ 合并表述是唯一与全部观测无张力的形态。
- 术语归口（建议随收敛采纳）：**「写回终态化通路」= 终态化子面（值固化，.b1 承接恒定项）⊕ 覆写子面（增量改写，.b5a 承接抖动项）**。

---

## 4. 判别实验落地性审查（单 world 约束，辖区 4）

**环境事实（F，parent 给定约束）**：run_t8.py 每臂 rmtree 重建 world → r2 / vanilla-r1 的 region 已被删除，**盘上仅存 r3 的 region**（`runtime\1.20.1\java\run\world\region`，归因实验序列最后一跑）。所有需两 world 对比的 per-section 重解析（b2 §10.3 实验1原形态）**不可直接跑**。

### 4.1 修正后实验清单

**档①（现可跑，只读 r3 单 world）**：
- **E-1a r3 世界存在性审计 + snap 互验**：用 snap_light.py 同构解析重算 r3 每 chunk 16hex hash，与 `.tmp\legacy-sweep-260919-06\t8-domain-r3-light.json` 逐 key 对比必须 100% 相等 → 互验解析器与证据体（E1 同构建态再推导）。前置失效 → 本档全部判据 VOID。
- **E-1b 520 unlit 直证（b4 P1 单臂版，V1）**：520 集在 r3 region 中无 light 节/全缺省 → C1a 直证；任一反向个体 → b4 §2 强制链证伪、连带本产物 §1.3 引用重议。
- **E-1c lit 计数审计（b4 P3 单臂版）**：r3 有 light 节 chunk 数 ≈ 3703（center 集）。
- **E-1d 终态化旁证（单臂弱检验）**：center chunk 与 unlit halo 相邻的边界 section 光值应呈「在 halo 侧被截断的暗边界」形态（若 vanilla 修正曾在域臂发生过，halo 侧会有光渗入记录）——只作旁证不作门。
- 档①**不能**做：r2 vs r3 per-section 抖动带分析（r2 已删）、跨臂中带核对（vanilla-r1 已删）。

**档②（需新采集：改造 run_t8.py 为 keep-world 模式或每 run 后拷走 region）**：
- **E-2a**：domain 臂 n=2 双跑、两 world 均留存 → per-section hash 对比跑 O1 形态门（边界带 ≥80% 且内部 ≤10% → .b5a candidate 门）+ O3 符号统计。
- **E-2b**：vanilla 臂 n=2 与 lightRust-only（-PlightRust=1 无 domainbatch）臂 n=2 各 self-diff——两者 ≈0 → 抖动为域批路径特有（.b5a 触发条件①要求域批写回，自洽）；lightRust-only 也抖 → 覆写通路外溢到 per-chunk 接管路（同 :836-837 尾语义），重定边界。
- 采集时按 #161 重走 world 身份链 + §9.8：keep-world 是**新增副作用**，须登记逆（归档路径清单）。

**档③（需改码，批准后）**：
- **E-3a**：task 行增 blocks25 输入 hash（复用 LIGHT_BETA wgBetaHash 通道，Mixin:777-783 同款）→ 输入 hash 跨 run 同 + 值异 = .b5a 直证；输入 hash 异 = .b5b R6 复活。

### 4.2 预登记判据文本（档①，先于执行）

- **前置集（preconditions；任一失效 → 该档判据全体 suspended→VOID，机械非零退出）**：
  - `key=r3_region_exists, expected=true, check=Test-Path runtime\1.20.1\java\run\world\region`
  - `key=r3_snap_json, expected=exists, check=.tmp\legacy-sweep-260919-06\t8-domain-r3-light.json 可读且 chunk 数=9450`
  - `key=snap_mutual, expected=0 mismatch, check=E-1a 逐 key 重算 hash 与 json 全等`
  - `key=r3_seed, expected=8576294172403134396, check=level.dat Data.WorldGenSettings.seed 实读`（如解析器支持；不支持则以 run log SELFCERT seed_in_leveldat 代替并声明降级）
  - `key=r3_selfcert, expected=exit 0, check=n2 记录中 r3 SELFCERT 全绿（前置产物转引）`
- **P1（E-1b）**：520 集（diff ∧ ∉ center，坐标 = r1 明细交 ⚠️ 以 parent 提供的 520 集清单为准）逐 chunk 在 r3 region 无 SkyLight/BlockLight 节（或全缺省）→ count(符合)=520 判「C1a 直证」；任何反向个体 → 判据 FAIL，b4 §2 降级重议。
- **P2（E-1c）**：lit chunk 总数 ∈ [3700, 3706]（容忍 ±3 解析边界效应）→ 「两臂仅 center lit」读法强化；出界 → 重议。
- **P3（E-1d，旁证不设门）**：输出边界 section 光值剖面文本供 worker 判读。
- **声明**：档①全为 E1 档单臂再推导；不得据此对 E2 差异下任何新结论（#162）；520 集成员资格系 r1 转引，用于 r3 chunk 的逐 chunk 判读时隐含「520 集跨 run 恒定」前提（d_cross 集恒定 + b4 §8.1 已支持，标注依赖）。

### 4.3 档①命令模板（主会话执行，本 worker 未跑）

```python
# -*- coding: utf-8 -*-
# t8_b5_r3_audit.py — 档①：r3 单 world 存在性审计 + snap 互验（只读，零采集副作用）
# 依赖：解析骨架同 .tmp/g3-260905-03/snap_light.py（chunk 坐标推导 rx*32+(i&31), rz*32+(i>>5)），
#       补两字段：每 section 的 SkyLight/BlockLight 节存在性（存在/缺失/flag 形态）。
# 步骤：
#   1) 解析 r3 region → per-chunk: {sections: [(y, has_sky, has_block, hash_sky12, hash_block12)], total_hash}
#   2) 互验：重算 total_hash 与 t8-domain-r3-light.json 逐 key 对比 → 全等才继续（否则 VOID, exit 1）
#   3) P1: 对 520 集逐 chunk 判 has_sky==False and has_block==False（或全缺省）计数
#   4) P2: 统计 has any light 节的 chunk 数，对照 3703±3
#   5) P3: 抽样 center chunk 输出与 halo 相邻侧边界 section 值剖面（文本）
# 输出落：.investigations/t8-attrib-260919-07/cmd-output/b5-r3-audit-<date>.txt
# 读法：P1 全中 → b4 C1a 直证；P1 有反例 → exit 2（判据 FAIL 非 VOID，数据有效需重议）；
#       P2 出界 → exit 3。退出码区分 VOID(1)/FAIL(2/3)/OK(0)。
```

（520 集清单与 r3 light json 的确切路径/文件名由主会话按盘面实际核对后填入；本模板不猜测文件名。）

---

## 5. 让渡边界

| 面 | 让渡给 | 一句话 |
|---|---|---|
| 恒定 2038 + 成员资格 4223 | .b1（维持主嫌格局） | 本产物只解释抖动 2185 与成员恒定性，不动 .b1 判决；建议采纳 §3 合并术语「写回终态化通路」 |
| 档①/②/③ 执行 | 主会话（shell 持有者） | 模板 §4.2-§4.3；改码/keep-world 需用户批准 |
| vanilla 引擎增量传播语义一手核对（LightStorage/ServerLightingProvider） | 主会话（yarn sources jar 不在工作区，同 .b2 §3.3 盲区） | §1.1-4 该步为 I 级参照，非 F；档② O1 结果即其运行时对账 |
| .b3 snap section 缺键残余 / 520 时序驱动 T-a/T-c | .b3 / .b4（不变） | 本辖区未触碰；520 unlit 直证归档① E-1b |
| R6 可见性残余 | 档② O1 × 档③ E-3a 联合裁决 | 与 .b5a 形态预测互斥，一次采集双判 |

---

## 6. 结论一句话（draft）

**d_self=2185 抖动源首选 .b5a「写回终态化通路的覆写子面」（Mixin:594-595 一行代码两面：免重光固化恒定 2038 + vanilla 增量传播改写边界带产生抖动，成员资格恒定由「内部 Rust 值逐 run 确定 ≠ vanilla」保证）；.b5b 输入快照差全通道出清、仅剩 R6 可见性低先验残余；与 .b1 终态化假说合并为同一机制两面；单 world 约束下档①仅可跑 r3 存在性审计+snap 互验+520 直证（模板 §4.3），抖动带形态门需档②新采集。**
