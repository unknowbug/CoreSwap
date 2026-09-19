# .b4 非域批 520 chunk 的路径归属与 diff 源定性 — T8 归因（260919-07，draft，静态分析）

- 角色: fan-out worker .b4（520 个非域批 chunk 的独立 diff 源，疑 per-chunk lightRust vs vanilla）
- 方法: 纯静态（一手源码直读 + 既有 log/前置产物数字）；沙箱无 shell，未跑任何命令。
- F = 一手源码/日志/前置产物数字直读；I = 推断，分开标注。status: **draft**。
- 前置已读: scout-map.md / b1-domain-algo.md（含 §8/§9 更新）/ b2-settled-timing.md（含 §8 更新）。

---

## 0. 辖区结论（先答）

**520 个非 center diff chunk 在 domain 臂从未被任何路径产光——它们在 domain 臂保存时是无光照数据（未到达 LIGHT 阶段）的 chunk；其 diff 源 = 跨臂「光照存在性」不对称（vanilla 臂 lit、domain 臂 unlit），不是「per-chunk lightRust vs vanilla」的值差。**（I，高置信，见 §2 强制链——每步仅依赖 F 数字 + F 代码 + 排除逻辑，不依赖 ChunkSerializer 细节）

**域臂光照生产路径全集（F，源码穷举）**：domain 臂开 `-PlightRust=1 -Pdomainbatch=1`（run_t8.py:30-32，LIGHT_RUST/LIGHT_DOMAIN 均开，LIGHT_PACKED 未开——task 行无 `packed=` 字段，.b2 §4.1 已核），`ServerLightingProvider.light(Chunk,boolean)` HEAD 接管（Mixin:856-858）后 chunk 光照只有三种归宿：
1. **域批 center 写回**：`wgLightDomainSubmit` 预检过 → 登记 → 封板任务逐 center `wgLightDomainWriteBack`（Mixin:535-576, 707-715）。写回循环**只遍历 centers**（Mixin:707），非 center chunk 不可能从这里得到光。
2. **per-chunk legacy 内联路**：仅当预检/收集失败（`WG_DOMAIN_INLINE++`，Mixin:547/564/880）或域任务降级重放（Mixin:722-724）。本 run **inline=0、fallback=0、degraded=0**（主会话实测，.b2 §8.1）→ 该路径产出 = **0 chunk**（F）。
3. **vanilla 自产**：仅 `handle==0`（init 失败）时不 cancel 走 vanilla（Mixin:866-871）。本 run `lightInit ok`（[LightRust]=1 行）→ handle≠0 → 该路径 = **0 chunk**（F）。
4. hook armed 前的 vanilla 自产：**不存在此窗口**——`wgLightDomainEnsureHook` 在第一次域提交内部调用（Mixin:568），即第一条 light() 调用就已进域批分流；`[LIGHT-DOMAIN] hook armed` 18:54:43 早于首 task 18:54:53（.b1 §1.3）；init 懒加载同样发生在首条 light() 调用内（Mixin:144-166）。域臂从第一条 light() 起**没有任何 vanilla 光照生产**（F 代码 + I 时序）。

由 1-4：**域臂 9450 chunk 中只有 3703 个 center 得到了光照（F，.b2 §8.1 交集复算）；其余 5747 个非 center chunk 的 region 光照数据只能为「未产光」态**（空/缺省 section，具体序列化形态属 §4 盲区，但不影响下方强制链）。

---

## 1. 520 chunk 路径归属（辖区 1）

**归属 = 「域批 task 的非 center 邻域受益者 / hook 前 vanilla 自产 / per-chunk legacy 路 / 覆盖漏网」四候选中的哪一个：都不是——是第五种：域臂从未产光。**

- 邻域受益者 ❌：写回只对 centers（Mixin:707-715），邻居 chunk 的 blocks 只被**读**（收集）不被**写**（F）。
- hook 前 vanilla 自产 ❌：§0-4，无时间窗（F）。
- per-chunk legacy 路 ❌：inline=0 + fallback=0 + degraded=0 + burst 提取 FP-LIGHT 总量恰 = 3703 = Σcenters（legacy 路成功也发 `FormProbe.lightCall`，Mixin:839-841；未出现多余行）（F，.b2 §8.1）。
- 覆盖漏网（预检失败走 inline）❌：同上 inline=0（F）。
- **域臂未产光 ✅**：排除法唯一剩余（I，高置信）。

旁证（F→I，算术闭合）：diff∩center=3703、diff 外余 520；簇A=616、簇B=3607（dist-analysis-260919-07.txt，scout §5 引用）；center 落簇 to: 97+528、full: 432+2646（.b1 §8.1）→ 簇A 外 diff = 616−529 = **87**，簇B 外 diff = 3607−3174 = **433**，87+433 = **520 = 全部 outside**（I，精确闭合，零自由度）→ **520 个非 center diff chunk 全部落在簇 A∪B 内部**（贴着已 lit 区），中带/outside-cluster 一个都没有。这与「域臂在已加载区边缘/尾波处被截断的 chunk」空间形态自洽。

---

## 2. diff 源强制链与互斥子候选（辖区 3）

### 2.1 强制链（每步只依赖 F + 排除）

1. F：520 ∉ center 集（.b2 §8.1）＋ §0 路径穷举 ⇒ **域臂这 520 chunk 无 Rust 光、无 vanilla 光**（域臂 unlit 态）。
2. F：它们在两臂 snap 中 hash 不同（diff 集成员）⇒ **vanilla 臂同 chunk 内容 ≠ 域臂 unlit 内容** ⇒ vanilla 臂这 520 chunk 是 lit 态（真实光照数据）。若 vanilla 臂也 unlit，两臂同为 unlit 序列化形态、hash 必同——被 diff 成员资格排除。
3. ∴ **520 = vanilla 臂 lit × domain 臂 unlit 的「存在性差」chunk**（I，高置信）。

> 本链不依赖「unlit chunk 序列化为什么」（空节点/缺 section/全 0），只依赖「unlit 态在两臂相同输入下确定性相同」——unlit 序列化是纯 chunk 数据的确定函数，无引擎差输入面（F+I）。

### 2.2 互斥子候选（列出不收敛；方向已被 center 集排除一半）

- **C1a（唯一存活方向）**：vanilla 臂 lit、域臂 unlit。✅ 由 §2.1 强制。
- C1b（域臂 lit、vanilla 臂 unlit）❌：域臂 lit ⇒ 必是 center ⇒ ∈ 3703 集，与 520 ∉ center 矛盾（F 排除）。
- C1c（两臂均 lit、值差）❌：域臂 lit ⇒ center，同上矛盾（F 排除）。

### 2.3 域臂为何少了这 520（时序驱动子候选，互斥列出不收敛——让渡 .b2/运行时）

域臂光照生产管线在 light 阶段比 vanilla 臂多一层**结构性延迟**：light() 提交后并不立即计算，而要等域封板（满 9 centers 或 GRACE_MS=10000 超时，LightDomainBatch.java:21,67-68,81-83）+ 任务线程执行。vanilla 臂 light() 调度即算。因此「chunk 到达 light 阶段 → 完成 lighting」窗口内，域臂的暴露时长系统性更长：
- **T-a 域批 grace 拉宽 unload 竞态**（主嫌，I，中置信）：ticket 过期/卸载发生在 light 阶段等待期内 → chunk 以未 lit 状态保存。同批次 tail 波（18:58:00，cx≈330/cz≈189-200 落簇B）最受伤——433/520 在簇B（§1 旁证算术），尾波时间密度与「最后一批 + settle 60s 内不再产 task」一致。
- **T-b 停服截断差**：域臂 stop 前仍有排队未到 light 阶段的 chunk——但 settle 60s 零新 task 行提示管线静止（.b2 §3.2），排队未 lit 的 chunk 若仍在活跃 ticket 内必然继续产 task → 与观测矛盾 → T-b 降权（I）。
- **T-c vanilla 臂二次 lit**（重载恢复路差）：ticket 过期的 proto chunk 在驱动回程被重新加载、续生成到 light 阶段——两臂机制相同，但域臂重新提交后再次承受 grace 延迟、短命 ticket 下可能反复截断（T-a 的重载变体，I，与 T-a 不完全互斥，标注并存）。

### 2.4 对 44.69% 总账的分解（辖区 3 交付）

**diff 4223 = 3703（两臂均 lit 的值差：Rust 域批 vs Java vanilla，.b1 辖区）+ 520（vanilla lit vs 域臂 unlit 的存在性差，本辖区，时序/截断性质）**。（I；第二项使 44.69% 的「值差」解读被稀释为 39.2% 值差 + 5.5% 存在性差——归因汇总时须分账，不得混读）

---

## 3. 中带零 diff 解释（辖区 2）

**最简机制（I，高置信，与 §1/§2 同一条链）**：中带 x57-173 的 chunk 在**两臂都从未 lit**（域臂非 center ⇒ 域臂 unlit；与 vanilla 臂 hash 相等 ⇒ vanilla 臂同为 unlit 态）→ 两臂保存的 unlit 序列化逐位相同 → 零 diff。**中带不是「地形简单区两引擎自然一致」（.b1 E-a）的正面证据**——它根本不是值对值的比较面；「center 集 ⊆ diff 簇 + 中带零 diff」由「是否 lit」这一个变量完整解释，无需地形相关性假设。β 读法（.b1 §8 读法 β）据此被压缩：地形敏感度卷积不再必要。
- 反向自检（F）：中带 chunk 若在 vanilla 臂 lit，必 diff（域臂 unlit）→ 中带零 diff ⇒ vanilla 臂中带全 unlit ⇒ 域臂/ vanilla 臂的 lit 集都完全落在簇 A∪B（I）。
- 此结论同时改写全局读法：**两臂 9450 chunk 中只有 3703（39.2%）真正 lit**，region 其余为未到 light 阶段的 proto/截断 chunk halo——驱动方式（FP-DRV 移动+卸载）与 ticket 行为的产物。该 halo 解释不依赖任何引擎差（I）。

---

## 4. 诚实声明 / 盲区

1. **unlit 序列化形态未核**（yarn ChunkSerializer 不在工作区，同 .b2 §3.3 盲区）：「未 lit chunk 无 light section/序列化为缺省」未一手核对。不影响 §2.1 强制链（见该节声明），但影响 §5 验证模板的判读细节。
2. n=1、E2 档：本结论仅对本次 r1 对拍成立，不外推其他 seed/驱动/维度（scout §4.4 继承）。
3. 520 的逐 chunk「哪臂 lit」判定需要 section 存在性/通道分离数据——既有 snap json 只有总 hash，信息论盲区（scout §1.4）；§2 链是排除法推理，非直接观测。
4. 簇A/B=616/3607 与 center 落簇数字均为前置产物转引（dist-analysis / .b1 §8.1），未独立复算（标注转引，主会话可一键复算）。

---

## 5. 让渡清单（辖区 4）

| 面 | 让渡给 | 一句话 |
|---|---|---|
| 3703 center 值差的机制（域批执行面 vs 引擎差） | .b1（主嫌格局不变） | 本辖区不动该判决；但「中带=地形一致」的 E-a 证据被 §3 剥离，.b1 §4 E-a 应降权 |
| 520 的时序驱动判别（T-a/T-b/T-c） | .b2 + 运行时验证 | grace 拉宽 unload 竞态是 .b2 辖区延伸；判别实验见 §6 V2 |
| 「两臂均只有 3703 lit」的全局读法 | parent 收敛 + judge | 改写 44.69% 分解（§2.4）；判据 verdict 本身不回改（scout §4.2） |
| unlit 序列化形态一手核对 | 运行时（主会话有 shell） | §6 V1，yarn 源或 mca 重解析 |
| snap section 缺键语义残余 | .b3（不变） | 本辖区未触碰其残余子面 |

## 6. 主会话执行命令模板（本 worker 未执行）

```python
# -*- coding: utf-8 -*-
# t8_b4_lit_audit.py — 两臂 lit/unlit 存在性审计（#161 前置：仅当两臂 world/region 未被
# 后续 run 覆写时可直接解析；否则须以同 seed 重跑采样，本模板不包含重跑）。
# 判据（预登记）：
#   P1: 520 集（diff∧∉center）在 domain 臂 region 中 light 节缺失/全缺省，vanilla 臂有实值
#       → C1a 证实；任何反向个体 → §2 强制链证伪、本产物降级重议。
#   P2: 中带（x57-173）chunk 在两臂均 light 缺失 → §3 证实。
#   P3: lit chunk 总数（有 light 节的 chunk 数）两臂各 ≈3703 → halo 读法证实。
import struct, sys, zlib, os, json
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
# 依赖: 现成 mca 解析路径见 .tmp/g3-260905-03/snap_light.py:23-88（chunk 坐标推导同源）；
# 本模板只补「section light 节存在性」字段（SNAP 不落盘该信息，scout §1.4）。
# 实现提示: NBT 中 sections[i].SkyLight/BlockLight 缺失或 Level 无 LightOn 标记 = unlit 态。
REGION = {arm: rf"E:\PYTHON\CoreSwap\runtime\1.20.1\java\run\world\region" for arm in ()}  # 填两臂各自 world 路径
CENTER = set(map(tuple, (l.split(",") for l in open(
    r"E:\PYTHON\CoreSwap\.investigations\t8-attrib-260919-07\cmd-output\b2-to-chunks.txt").read().splitlines() if l.strip()))) | \
    set(map(tuple, (l.split(",") for l in open(
    r"E:\PYTHON\CoreSwap\.investigations\t8-attrib-260919-07\cmd-output\b2-full-chunks.txt").read().splitlines() if l.strip())))
DIFF = json.load(open(r"E:\PYTHON\CoreSwap\.tmp\legacy-sweep-260919-06\t8_diff_dist_input.json"))  # 若无: 按 scout §5 重算 diff 集
# （解析主体由主会话按 snap_light.py 解析骨架补全；本 worker 沙箱无 shell，不实现完整解析。）
```

```powershell
# V2（零采集，先跑）: 簇外差集独立复算 + DUP 自证行核对（验证本文件 §1 旁证算术与 .b2 §8.1）
python .tmp\legacy-sweep-260919-06\cmp_t8.py   # diff=4223 复现
# 簇外差集 = diff − (b2-to ∪ b2-full) chunk 集；期望 520 且全部 ∈ 簇A∪B（87/433 分账）
findstr /c:"dup=" .tmp\legacy-sweep-260919-06\t8-domain-r1.log   # 最后 task 行 selfProof dup 计数，>0 说明存在重提交（T-c 相关面）
```

---

## 7. 结论一句话（draft，已被 §9 取代，保留原文为取代链）

**520 个非 center diff chunk 在域臂从未被产光（四候选路径全被计数/代码排除，域臂光照生产者只有 3703 个 center），其 diff 源 = 跨臂光照存在性不对称（vanilla 臂 lit / 域臂 unlit，方向由 center 集唯一强制），时序驱动主嫌 = 域批 grace 批处理拉宽 light 阶段 unload 竞态（T-a，让渡 .b2 判别）；中带零 diff = 两臂均 unlit 的同态匹配，「地形简单区两引擎一致」证据被剥离；44.69% 应分账为 3703 值差（.b1）+ 520 存在性差（本辖区）。**

---

## 8. n=2 判别实验后的更新（260919-07 追加；输入 = cmd-output/n2-mechanical-260919-07.txt 转引，未独立复算）

### 8.1 新事实（F，parent 转告）

- `d_cross_r2` 与 `d_cross_r3` 的 diff key 集**完全同一**，各 4223 —— 对 vanilla 的 diff 集合跨 run 稳定。
- `d_self`（domain r2 vs r3）= 2185，且**全部 ⊆ 3703 center 集**；520 集未被 d_self 触及。

### 8.2 判读：恒定集对 T-a 的支持/削弱（辖区问题直答）

- **存在性差模型的内部自洽（F+I）**：520 在域臂 unlit 态是「无数据」的确定形态，r2/r3 域臂同 unlit → d_self 必然不触及 520 ✅（观测吻合）；它们出现在恒定的 d_cross 集 → 「vanilla lit / 域臂 unlit」形态在两次独立 run 逐 chunk 复现 ✅。存在性差模型对 n=2 无张力。
- **对 T-a 的净判读：削弱「细粒度时序抖动」变体、支持「饱和竞态（确定性极限）」变体**（I，中置信）：
  - 削弱面：纯 unload 竞态的朴素读法（线程调度抖动决定哪些 chunk 被截断）预期 cross-arm 存在性差集逐 run 摆动；实测逐 chunk 恒定，排除了「随机抖动型截断」为主要形态。
  - 支持面：T-a 的结构性参数是确定性的——GRACE_MS=10000（LightDomainBatch.java:21）对 chunk 到达 light 阶段后的可用 ticket 窗（由驱动轨迹/ticket 几何决定，同 seed 同驱动两 run 逐 chunk 相同）是**结构性 ≫ 关系**：竞态被 grace 宽度饱和、结局由确定性几何而非调度抖动决定 → 同一批 chunk 每次都以 unlit 收场。这与恒定集相容，且比「vanilla/域臂管线速度差」类机制更被证据偏好（后者难以给出逐 chunk 恒定）。
- **对比信号（F，强化本模型）**：d_self=2185 全部落在 center 集 = **域臂 lit chunk 的输出 run 间波动大，而 520 unlit chunk 零波动**——「lit=有计算、有计算面才有 run 间方差；unlit=无数据、恒定」的形态正是存在性差模型预言的二分（旁证；d_self 波动本身的机制归 .b1/.b2，本辖区不判）。
- 保留声明（§4 继承）：520 ⊆ d_cross_r2 恒定集的成员资格是 parent 转告的集合同一性推论，未做 520×d_cross_r2 的逐 key 交集复算（I，建议主会话补一次零采集交集核验）。

### 8.3 更新后 T 子候选状态

- T-a（grace 拉宽 unload 竞态）维持主嫌，但收窄为「饱和/确定性极限」形态（§8.2）；T-b 停服截断差维持降权；T-c 重载变体不受本实验约束（保留）。
- 新增可证伪点：若 V1 存在性审计（§6）显示 520 在域臂其实有 light 数据（即 C1a 证伪），则恒定集须由「值差」另解——该情形与 d_self ⊆ center 不冲突，但与 §2.1 强制链矛盾，须重议。

---

## 9. 更新后结论一句话（draft，取代 §7）

**520 非 center diff chunk 维持「域臂 unlit / vanilla 臂 lit」的存在性差定性（n=2 下 d_cross 恒定集与 d_self=2185⊆center 的二分形态均与模型无张力且互证），520 集 ⊆ d_cross 恒定集将 T-a 从「调度抖动型竞态」收窄为「grace(10s) 结构性 ≫ ticket 窗的饱和竞态（几何决定、逐 run 复现）」，T-b/T-c 维持降权/保留；44.69% 分账（3703 值差 .b1 + 520 存在性差本辖区）不变。**
