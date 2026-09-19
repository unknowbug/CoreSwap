# .b1 域批算法差 — T8 light-hash diff 44.69% 归因候选（260919-07，draft，静态分析）

- 角色: fan-out worker .b1（域批 fill pass 与 per-chunk 传播的算法语义差）
- 方法: 纯静态（源码 + 两臂 log + 既存分布分析）；沙箱无 shell，未跑任何命令。
- 置信度状态: **draft**（全部结论未做运行时验证；验证模板见 §6）。

---

## 0. 辖区结论（先答判别点）

**倾向排除**：「域批分片/批处理语义本身」是 44.69% diff 的主机制（置信度：中高，静态依据充分，见 §2 判别 ①②③）。

**保留两个 .b1 残余子面**（量级核算后均不足以承载主 diff，见 §3）：
- R1: n∈{2,4,8} 的 timedOut 任务 blocks25 装配槽位错置缺陷（真实存在，上界 ~70 chunk）。
- R2: 数值巧合 簇A=616 ≈ ΣtimedOut centers=625（待运行时验证，若证实则 timedOut 空间集中假设升级）。

**重要框架修正（F，影响全 fan-out 读法）**：vanilla 臂**不是**「Rust per-chunk 传播」，而是**纯 Java vanilla 光照**——run_t8.py:3「臂 vanilla = 无 lightRust/domainbatch」、result.json vanilla 臂 lightInit_ok=0 / lighrust_lines=0 / domain_hook=0（scout-map §1.2 负自证一致）。因此本对拍 = **E2 跨引擎**（Rust light 引擎[域批形态] vs Java light 引擎），44.69% 的主 diff 质量更符合「**Rust vs Java 引擎语义差（地形相关）**」——该候选**当前 fan-out 无独立 .bN 承接**，上报 parent 裁决（让渡清单 §5）。

---

## 1. 证据（F = 一手直读；I = 推断）

### 1.1 域批空间分片规则（Q1 答复，file:line）

- **分片解析式（F）**：domain key = `(floorDiv(cx,3), floorDiv(cz,3))` — ServerLightingProviderMixin.java:512-516（`wgLightDomainKey`）。即 **3×3 chunk 网格对齐域**，分片边界在 cx ≡ 0 (mod 3) / cz ≡ 0 (mod 3)。
- 封板规则：9 中心到齐即封板，否则 GRACE_MS=10000 超时按已到齐中心封板 — LightDomainBatch.java:21,64-85,116-128。
- 任务体：中心按 packed pos 排序后 `kx=k%3, kz=k/3` 槽位装配进 5×5 blocks25 — ServerLightingProviderMixin.java:602,610-611,673-687；Rust 内核按槽位 (kx,kz) 提窗、按段序 k 写回 — mod.rs:398-399,474-527。

**判别点回答（簇边界 vs 分片边界）**：
| 簇 | 范围 | x 边界 mod 3 | z 边界 mod 3 | 与分片对齐？ |
|---|---|---|---|---|
| A | x 3..57, z -35..0 | 3≡0, 57≡0 | -35≡1, 0≡0 | x 对齐、**z 不对齐** |
| B | x 173..342, z 184..222 | **173≡2**（不对齐）, 342≡0 | 184≡1（不对齐）, 222≡0 | **均不对齐** |

（I）若 diff 由分片机制产生，形态应为「全箱弥散（域批覆盖全部 9450 chunk）」或「每 3 chunk 周期条纹（分片边界伪影）」或「3×3 块的并集」。实测（dist-analysis-260919-07.txt，F）= 2 个实心大矩形、singleton=0、中带 x 57-173 整带零 diff——**三种分片形态均不符**。簇边界不与 batch/task 分片边界吻合。

### 1.2 判别链（Q1/Q3 核心，三条独立静态判据）

- **判据①（中带零 diff，F+I）**：x 57..173 整带（跨全 z）零 diff。这些 chunk 同样走域批路径（全驱动箱均为域批处理），其 hash 与 Java vanilla **逐位一致**（16hex 总 hash 相等）。域批算法若系统性错误，不可能只在两个矩形内出错。→ 域批路径在光照拓扑简单区产出 vanilla 一致结果（中带解释候选归 Q2，见 §4）。
- **判据②（位等价门，F）**：域批内核与 Rust per-chunk 同内核同输入逐位等价，由 `light_compute_domain_bitwise_equivalence` 钉死（worldgen-core/src/light/mod.rs:866-900 测试；mod.rs:389-399,446-455 等价性论证：fill 逐格纯查表邻接无关、BFS 每中心从本帧快照独立重算、**无新不动点语义** mod.rs:451-453）。→ 「域批 vs per-chunk」这个轴上不存在算法语义差；存在语义差的轴是「**Rust 引擎 vs Java 引擎**」。
- **判据③（量级核算，F+I）**：timedOut=true 115 task 的 center 总数 = **625**（本次逐行累加，log 行号见 §3.1；可复算）。即使全部算错也 ≤625 ≪ 4223，更 ≪ 簇B=3607。→ timedOut 相关任何缺陷面都无法单独承载主 diff。

### 1.3 timedOut 语义判读（Q4 答复）

- （F）task 行**不含坐标**（grep 全量核实：行格式仅 centers/ok/degraded/timedOut/avgMs/selfProof，Mixin :733-743 打印体无 ChunkPos）→ **静态无法判读 115 task 是否落入簇A/簇B**。这是本辖区的硬盲区，不是「已判读」。
- （F）timedOut=true = 10s grace 到期按已到齐中心封板（LightDomainBatch.java:68,93,116-119）；ok 均>0 = 全部已到齐中心完成 JNI+写回，**无降级**（degraded=0 全量核实）→ 任务结果是「完整计算但中心不全」，非「残缺计算」。
- （F）timedOut 任务呈 **7 个时间波**（18:55:04-14 / 18:55:54-55 / 18:56:19 / 18:56:44-45 / 18:57:09-10 / 18:57:34-35 / 18:57:59-18:58:00），每波 14×centers=6 + 1×centers=4 为主（首波例外）——周期 ≈ 25s，结构高度规律（I：像驱动回路周期性路过某类地形/负载，空间位置未知）。
- （F）sizes 分布：1×1, 1×8, 2×2, 6×4, 76×6, 29×3（合计 115 task / 625 centers）。

---

## 2. 域批 fill pass 算法语义 vs per-chunk：系统性差异面清单（Q3 答复）

**域批内部（C-1a vs Rust per-chunk）——无语义差面（F，均有等价论证/测试）**：
1. fill：全域共享一趟，逐格纯查表（opacity/emission/col_max 列内），种子按 i 升序交付 = 旧内联序（mod.rs:446-468,531-582）。
2. BFS：每中心 3×3 窗（48×48）从本帧快照独立重算，传播单调 + 不动点唯一（mod.rs:451-453,584-598）。
3. sky 直落 round2 = col_max 区间算术；种子收缩 round2 区间枚举与 round1 集合恒等（mod.rs:605-659）。
4. 位等价测试 PASS 双 LCG 种子（mod.rs:866-900；docs/10-timewise-archive.md:3567）。

**域批路径 vs Java vanilla 引擎——真实语义差异面（此轴才是 diff 的候选承载面；部分让渡）**：
- S-a 窗口截断 vs 全局传播：Rust 任何中心只看 3×3 chunk 窗（理论界：光级 ≤15 ≤ 16-block halo，窗内自洽）；Java 增量传播跨已加载 chunk 边界动态扩散，邻居加载时序影响结果。（让渡：引擎差候选）
- S-b 快照时点：Rust 用提交时刻 blocks9 快照（Mixin :540-543,562-567）；Java 边算边读活世界。（.b2 时序差与引擎差的交叉面）
- S-c 传播算法形态：Java increase/decrease 双队列增量；Rust 全量重算不动点。不动点唯一时等价，**唯一性前提被破坏处（如有）即差异面**。（让渡：引擎差候选）
- S-d opacity/emission 查表语义（wgLightEncState luminance 高位，Mixin :420-426；mod.rs:544-570 lookup+lum 覆写）与 Java `getOpacity/getLuminance` 的逐 block 对拍——未验证。（让渡：引擎差候选）

---

## 3. .b1 残余子面（保留，附排除推理链）

### 猜测→验证→排除→发现 链（本辖区）

1. **猜测**：域批分片边界伪影 → **验证**：簇边界 mod 3 对齐检查 → **排除**：簇B x=173≡2、z 双簇不对齐；形态为实心矩形非条纹（§1.1）。
2. **猜测**：timedOut 不完整任务装配错置 → **验证**：静态复核装配代码 + sizes 分布 → **部分排除**：错置仅可能发生于 n∈{2,4,8}（n=3/n=6 排行满行时槽位正确：排序 = (z,x) 字典序，满行连续 → kx=k%3,kz=k/3 与真实相对位置一致；n=1 自洽）。且每中心内核窗内容 = 自己提交的 blocks9（窗列 [kx*16,kx*16+48) 恰为其自身拷贝区，Mixin :680-687 dst 公式），错置危害只能经「后 k 中心覆盖前 k 窗口污染」发生，受影响 ≤ n∈{2,4,8} 的 9 task ≈ 36 centers（2×2+6×4+1×8=36）±被污染邻窗 ~30 → **上界 ~70 chunk**（I，静态推演）→ 量级不足以进主机制。
3. **发现（数值巧合，待验证）**：ΣtimedOut centers = 625 vs 簇A = 616，Δ=9（恰一个满 task）。若运行时证实 115 task 的 center 全部（或近乎全部）落在簇A 矩形内，则「timedOut 空间集中 + 某种与中心不全相关的系统性差」重新升级为簇A 的主嫌；但簇B=3607 仍需别解。→ §6 模板 V2。

### R1 装配槽位错置（代码依据）
- Mixin :610-611 `minX/minZ = centers[0]±1`、:680-681 `kx=k%3,kz=k/3`、:683-685 覆盖式 arraycopy——n<9 且中心集非「满行前缀」时槽位 = 枚举序而非真实相对位置；后续中心拷贝会覆盖先前窗口槽（last-writer-wins），窗口读到错位邻域数据。（F 代码 + I 危害推演）

### R2 快照合并一致性
- 5×5 重叠槽由多个中心的 blocks9 依次覆盖，同一 chunk 在不同提交时刻的快照若不一致，合并结果任取其一——**本帧一致性前提**（mod.rs:451-453「本帧快照的确定函数」）在跨时刻快照混拼时不成立。危害面 = 满编 9 中心任务也可能错（其重叠槽来自他中心异时快照）。（F 代码 + I）→ 与 .b2 时序差边界交叉，**让渡 .b2 主判**，.b1 只登记结构面。

---

## 4. Q2：中带 x 57-173 零 diff 的解释候选（只标归属）

- E-a **地形简单区两引擎自然一致**（光照拓扑平凡：全空柱 skylight=15 / 无光源 → 任何正确引擎同结果）→ 归属：**引擎差候选**（地形相关性证据，方向上支持主 diff 质量走引擎差而非域批差；当前无独立 .bN，上报 parent）。
- E-b 中带 chunk 走了非域批路（WG_DOMAIN_INLINE 回退 / 预检失败走 vanilla 内联）→ 归属：**.b1**（但与判据①冲突度低：inline 路也是 Rust 内核，若引擎有差 inline 路同样该差 → E-b 解释力弱，标低置信）。
- E-c 中带 = 某执行批次（如某波 task）整体特殊 → 归属：**.b2**（时序/批次语义）。
- （I，读法）E-a 与「diff 集中在 2 个矩形」联合读 = diff 强地形相关；簇B z 184-222 / 簇A z -35-0 是否对应海洋/大型洞穴等地貌，静态不可判，需地图叠合（§6 V3）。

---

## 5. 让渡清单（边界声明）

| 面 | 让渡给 | 一句话 |
|---|---|---|
| timedOut 115 task 空间定位/瞬态语义 | .b2 + 运行时验证 V2 | task 行无坐标，静态不可判；数值巧合 625≈616 需探针裁决 |
| 跨中心快照异时合并一致性 | .b2（主）| R2 结构面在 .b1 登记，机制判读归时序 |
| snap section 缺键语义残余 | .b3 | 本辖区未触碰 |
| **Rust vs Java 引擎语义差（S-a/c/d + 地形相关主 diff 质量）** | **parent 裁决（无现成 .bN）** | 判据①②将主 diff 质量推出域批轴；建议补 .b4 或并入既有候选重述 |
| 44.69% 判据/verdict | 不变 | 遵守 scout-map §4.2，不回改判据读法 |

---

## 6. 主会话执行命令模板（运行时验证，本 worker 未执行）

```powershell
# V1 复算 ΣtimedOut centers（验证本文件 §1.3 的 625；纯读 log）
python -c "import re;ls=open(r'.tmp\legacy-sweep-260919-06\t8-domain-r1.log',encoding='utf-8',errors='replace').read().splitlines();ms=[int(m.group(1)) for m in re.finditer(r'centers=(\d+) ok=\d+ degraded=\d+ timedOut=true','\n'.join(ls))];print(len(ms),sum(ms))"
```

```python
# V2 timedOut 空间定位探针（需主会话加一次性诊断：在 Mixin :733 task 行打印各 center ChunkPos
#    或 FormProbe 开启后复跑 domain 臂 n=1；无既有数据可静态回答。产出后与 diff 簇叠合：
#    判据 = timedOut centers 是否 ≥90% 落入簇A 矩形 [3,57]x[-35,0]。
#    若是 → R2 巧合成立，簇A 归 .b1 残余面重审；若否 → .b1 残余面降权。）
# V3 地貌叠合（零采集）：对 diff 簇A/B 与中带做 surface/heightmap 可视化（复用 .tmp 两臂
#    light json + 任一臂 region 高度图只读重解析），判 E-a 地形相关性。
```

## 7. 结论一句话（draft，已被 §9 取代，保留原文为取代链）

**域批分片/批语义与 44.69% diff 的空间分布（实心双矩形 + 分片不对齐 + 中带零 diff + 位等价门 + timedOut 量级上界 625≪4223）五点不符，倾向排除其为主机制（置信度中高）；主 diff 质量指向 Rust vs Java 引擎语义差（地形相关，当前无 .bN 承接，上报裁决）；.b1 保留残余子面 R1（装配错置 ≤~70 chunk）与 R2（625≈616 巧合，待运行时验证）。**

---

## 8. 更新（260919-07 追加：主会话 b2-burst-extract 新事实后的判读修订）

**新事实（F，源 = cmd-output/b2-burst-extract-260919-07.txt，零采集）**：
- timedOut 受害集 n=625：inA=97 / inB=528 / other=0（与 §1.3 Σcenters=625 互证 ✓）。
- full 集 n=3078（=342 满编 task ×9）：inA=432 / inB=2646 / other=0。
- inline=0。
- **并集：全部 3703 个 task center 100% 落在簇A∪B 内**；中带 x 57-173 从未成为 center。diff=4223 > 3703 → 另有 520 个 diff chunk 从未是 center（未解释，开放）。

**R2 判读收口（猜测→验证→排除）**：「簇A=timedOut 集中」假设 → 实测 inA 仅 97/625（15.5%），timedOut 主体在簇B（528）且 528≪3607 → **R2 排除**：timedOut 集无论按总量还是按落点都无法承载任一簇。R1（装配错置 ≤~70 chunk）维持原判。

**判读反转风险（重要，I，限定 §0 倾向）**：新事实给出比「分片边界」强得多的相关——**center 集 ⊆ diff 集**（3703/4223=88%，且中带零 diff 恰与「从未是 center」重合）。两个互斥读法：
- 读法 α：center 覆盖面 ≈ 后期生成区，中带 = hook armed（18:54:43，首 task 18:54:53）**之前**或预检失败走 Rust per-chunk legacy 路的 chunk；中带与 vanilla hash 一致 ⇒ Rust per-chunk ≈ vanilla ⇒ 引擎差被剥离，**主机制回流 .b1 的域批执行面**（快照合并 / 写回路径 wgLightDomainWriteBack / blocks9 收集），而非内核语义或分片边界。
- 读法 β（§0 原判）：中带 = 地形简单区，center 集 ⊆ diff 只是地形相关巧合。
- **α/β 可判别（交主会话）**：① 验证中带 chunk 是否确实未经域批（域 arm 中 Rust per-chunk legacy 路的调用计数 / task 行外 light 路日志）；② 复核提取脚本覆盖面（若中带其实有 center 则回 β）。α 证实前，§0 结论限定为：「分片边界/内核语义子面排除维持；『域批执行面』子面随 center⊆diff 升级为主嫌」。

## 9. 更新后结论一句话（draft，取代 §7）

**域批的「分片边界伪影 / 内核语义差」子面维持排除（中带零 diff + 位等价门 + timedOut 落点实测 97/625 否证 R2；R1 ≤~70 chunk 维持），但新事实「全部 3703 个 task center 100% 落于 diff 簇、中带从未是 center」使「域批执行面（快照合并/写回/收集）」在读法 α（中带=per-chunk legacy 路且与 vanilla 一致 ⇒ 引擎差剥离）下升级为最大主嫌——主机制可能回到域批批处理执行本身（.b1）；α/β 判别动作见 §8。**

---

## 10. 更新二（260919-07：n=2 判别实验 d_self/d_cross 分账，取代 §9）

**新事实（F，源 = cmd-output/n2-mechanical-260919-07.txt，domain 臂双跑 r2/r3 机械复算）**：
- d_self = 2185（r2 vs r3 互异 chunk，23.12%）；d_self 100% ⊆ 3703 center 集、100% 落簇A∪B、中带 0。
- d_cross_r2 = d_cross_r3 = 4223，**key 集逐 key 同一**（两跑对 vanilla 的 diff 集合完全可复现）。
- H-b1-纯型预证伪观察点（中带出 diff）未触发。

**分账（I，基于 F 的机制含义）**：
- **恒定部分 = 2038 chunk**（4223−2185）：两跑对 vanilla 的错法逐位相同 → **确定性系统错**：同输入必同错。内核语义差（确定性）无法再被笼统排除，但结合 §8 center⊆diff 与中带=非 center 的读法 α，更可能是域批执行面的**结构性效应**（如写回终态化过早：wgLightDomainWriteBack 的 setLightOn+releaseLightTicket（Mixin :582-596）使 chunk 此后不再被重新光照，而 vanilla 会在邻居加载/更新后继续传播修正——错误集合由「提交时快照 vs 终态世界差」决定 → 地形成簇且跨跑稳定）。
- **波动部分 = 2185 chunk**（占 diff 集 51.7%）：两跑均错（仍在 d_cross 集内）但**错值逐跑不同** → 存在**时序依赖的输入不定性**。Rust 内核给定输入是确定性的（位等价门同源确定性）→ 不定性只能来自批次执行：grace 10s 封板竞速（timedOut 任务组成逐跑不同）、task 交错、跨中心快照 last-writer-wins 合并的落点（§3-R2 面）。这是 **.b1 执行面 × .b2 时序差的交叉签名**，单独任一候选都不完整。
- **d_cross 集恒定 vs d_self 非零的联合约束（判别力核心）**：机制必须同时满足「错误集合逐跑不变（规则性/结构性的错）」+「约半数错值逐跑漂移（时序敏感的错法）」。候选匹配度：① 引擎内核纯语义差 → 预测 d_self=0（❌）；② 随机损坏/内存错 → 预测 d_cross 集漂移（❌）；③ **域批「快照时点终态化」结构效应 + 封板/合并时序不定 → 两者兼备（✓ 唯一存活形态）**。
- vanilla 臂 + snap 管线两侧跨跑稳定（d_cross 集恒定旁证）→ 波动源被干净隔离在 domain 臂内（E1 档旁证，非 E2 上界声明）。

**对 §8 读法的影响**：n=2 结果**加强读法 α**（中带 chunk 两跑均与 vanilla 逐位一致且互跑一致 → 该子集是「另一条确定性且正确的路」的产出，与「地形恰好简单」相比解释力更强）；§0 的「排除域批主机制」至此**反转收窄**为：排除的是「分片边界伪影 / 内核逐位语义差」两个子面，**「域批执行面（终态化过早 + 快照时点 + 封板竞速）」成为唯一同时满足全部空间判据与 n=2 判据的候选机制**，.b1 与 .b2 需在其上分工（错集归 .b1 结构面、错值漂移归 .b2 时序面）。

## 11. 更新二结论一句话（draft，取代 §9）

**n=2 分账（恒定 2038 + 波动 2185、d_cross 集逐 key 恒定、d_self ⊆ center 集且中带 0）唯一同时满足「错集规则稳定 + 半数错值时序漂移」的候选是域批执行面「快照时点终态化结构效应 × 封板/合并时序不定」——「分片边界伪影/内核逐位语义差」两子面维持排除，主机制判读反转为 .b1 执行面（错集）+ .b2 时序面（错值漂移）联合承载，读法 α（中带=确定性正确的非域批路）被加强；待判残余：终态化过早假说需单 chunk 追踪验证（错值是否随邻居加载时点收敛）。**
