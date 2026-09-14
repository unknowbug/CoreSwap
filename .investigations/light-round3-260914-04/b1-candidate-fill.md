---
候选: A — 内核 fill 降维（fan-out .b1）
角色: core.worker（fan-out 并行，只交付候选设计，不实施）
状态: draft
日期: 2026-09-14 15:59（Get-Date 实取）
输入: 架构设计-260914-04 / scout-map.md（P1）/ 12-lighting.md（round1/2）/ worldgen-core/src/light/mod.rs 全文静态读出
retry 轮次: 1（静态推演，无运行数据；所有运行数字引用历史口径并标注来源）
---

# 候选 A：内核 fill 降维——成本上限推演 + 等价性 + 风险

## 0. 前提确认（scout 结论复核）

- fill×col_max 融合**已存在**（mod.rs:208-209 注释 + L241-242 同趟写 col_max）——该子项零剩余空间，本候选不含。
- 均质 section 跳过在 int[] ABI 下前提不成立（blocks9 不携带节边界/均质信息，mod.rs:142 注释的 ABI 契约）——**排除**，归候选 C 改形面。
- 剩余三子项：① lookup() 消边界检查/Option；② fill×sky_fall 融合；③ scratch 清零成本削减。以下逐项。

**一个关键时序事实（静态读出，scout 未显式指出）**：scratch 清零（mod.rs:195-203，2.65MB）发生在 `_t0 = Instant::now()`（mod.rs:192）**之后**、fill 主循环之前 ⇒ **清零成本已计入 fill 的 943µs**（12 篇 L99 口径）。子项③的上限账是 fill 943µs 的一个切片，不是独立 phase。

## 1. 成本上限推演（#124 先量后改，#22 禁自由参数）

### 基线数字（全部历史口径，来源标注）

| 量 | 值 | 来源 |
|---|---|---|
| fill | 943 µs/chunk (58%) | 12-lighting.md L99（round2 post-opt phase 分解，含 Instant 探针开销） |
| sky_fall | 300 µs/chunk | 同上 |
| export | 157 µs/chunk | 同上 |
| 内核合计 | 1.587 ms/chunk | 12 篇 L73（round2 口径） |
| e2e 回退 | ON 17.4s vs OFF 13.9s = 3.5s / 2025 chunks | 12 篇 L94（round2 四臂中位） |
| 表规模 | 1003 项 × 2B（Vec<(u8,u8)>，mod.rs:57/89-95） | scout §2（python 读出 light_data.json） |
| scratch 清零 | 3×884736B = 2.65MB/调用 | mod.rs:195-203 计算 |

### 子项① lookup() 消边界检查/Option（mod.rs:119-124）

**改动**：表加载后预扩容到 `max_id+1`（构造期一次性，L94-95 已是 resize 语义），热路径 `table[id as usize]` 直接索引（或 `unsafe get_unchecked` + 构造期不变式断言），消 `get().copied().unwrap_or()` 的边界检查 + Option 解包 + 冷回退分支（L123）。

**指令账（上界推演，每非 air 格）**：现 lookup = 比较 id<0（L120）+ 越界比较 + `get` 的 Option 构造 + `unwrap_or` 分支 + 2×u8 拷贝 ≈ **5-7 条额外指令** vs 直接索引的 1 条 load。air 格走快路径不查表（mod.rs:226-229）。

**量级推演**：fill 943µs 覆盖 884736 格 = **1.07 ns/格**（含 blocks9 读 3.5MB 流 + 清零 + 查表 + 三种条件写，已是内存带宽型而非法指令型）。非 air 格占比：真实地形 surface 区地下实心 + 地上空气，blocks9 全域非 air 格约 40-60%（开放问题 5 的运行实测才能定；scout §1.2 量级 3-6 万非空格/chunk 中心口径不适用于 9-chunk 域——**open**）。取保守上界：非 air ≤ 50 万格 × 6 指令 × (1/4GHz ÷ ~4IPC ≈ 0.06ns/指令 SIMD 混合下限) ≈ **≤40-80 µs/chunk**。**上限声明：即使全部非 air 格都省 7 条指令，绝对上界 ~100µs/chunk，即 fill 的 ≤10%**——因为 fill 是带宽型（1.07ns/格 ≈ 每格 ~4 周期总量），指令层只剩零头。

**变体（opacity-only 紧表）**：拆 `opacity: [u8; N]`（1003B，L1 常驻）+ emission 惰性查（emission>0 仅光源格，量 ~1e2/chunk）。省的是 (u8,u8) 16-bit stride 的缓存足迹（1003×2B=2KB → 1KB，均 L1 内，**缓存收益≈0**，scout §2 已核表全驻 L1/L2）⇒ 紧表变体收益不优于直接索引，**降级为可选形态**（实现简单度择一）。

### 子项② fill×sky_fall 融合（sky_fall L266-273 并入 fill 趟）

**现状**：sky_fall = 独立 2304 列循环，读 col_max 后逐格写 `sky_light[...]=15`（L266-273），300µs。写量 = 全部 15-区间格（列 [col_max+1, 383]，地表以上空气柱，典型 30-60 万格/chunk，**open** 需运行实测分布）。

**等价性推演（开放问题 4 的回答）**——正序 y 单趟**确实不可得**（15-区间依赖最终 col_max，y 升序时列的最终 col_max 要到最后一格才知）。两个可行变体：

- **变体 ii-a（y 降序单趟，推荐推演对象）**：fill 主循环改 y 从 383 降到 0。每列自顶向下：air/opacity==0 格 ⇒ `sky_light[i]=15`（与 fill 同趟写）；**首次遇 op>0 格** ⇒ 记 col_max=y 并置列 done 标志，此后该列不再写 sky。等价性：
  1. 降序下「首个 op>0」= 升序下「最后写者」= max(y | op>0)（mod.rs:242 语义恒等——升序是条件覆盖取 max，降序取 first，同一集合同一 max）；
  2. 写集 = {(x,z,y) : y > col_max(x,z)} = 升序 sky_fall 的区间 [col_max+1, 383]（L268-272）**逐位恒等**；
  3. BFS 阶段不变（种子枚举 L285-316 仍读 col_max，语义不变）；
  4. block_light 种子 queue 的**入队顺序随 y 反转**——需引用 round1 已确立论证「单调松弛 + 不动点唯一，与入队顺序无关」（12 篇 L38-39 / mod.rs:284 同款论证，先例已被 golden 逐位验证背书）。此为本变体唯一新增论证义务。
  5. 降序遍历的 blocks9 读流仍是顺序流（y 步长 256，方向反转不影响预取）——**open**（编译器/硬件预取对反向流的实测差异）。
  - 代价：每列加 done 判断（可用 `col_max>=0 && y<col_max` 的单比较替代标志位，因降序首次写 col_max 后比较即短路）；air 快路径合并 sky 写（air 格从「continue 纯跳过」变为「写 sky 15 再跳过」——**air 快路径变慢**：多一次 1B store）。
- **变体 ii-b（两趟保守）**：fill 升序不动，sky_fall 保持独立但与 fill 共享 y-tiling 改善局部性——收益微小，列为 ii-a 不可行（如预取劣化）时的回退形态。

**上限账**：sky_fall 300µs 的构成 = 15-区间 store 流（30-60 万 B，不可省——store 本身只是搬家进 fill）+ 独立循环的列遍历/col_max 二次读/分支开销。融合省的是**第二趟遍历开销**，store 带宽全额保留 ⇒ 上限 ≈ 300µs − store 下限。60 万 store @ ~1B/cycle-SIMD ≈ 保守 60-100µs ⇒ **融合上限 ≈ 100-200µs/chunk，且部分被 air 快路径新增 store 抵消**。

### 子项③ scratch 清零削减（mod.rs:195-203，2.65MB 计入 fill 943µs）

**语义边界（谁依赖全 0）**：
- `opacity` 全 0 ⇒ air 快路径合法（mod.rs:226-229 靠「清零后不写」表达 op=0）；
- `block_light` 全 0 ⇒ BFS 初值（L254 传播基线）；
- `sky_light` 全 0 ⇒ 15-区间以下格的隐式 0（L270 只写区间内）+ BFS 初值（L317）。

**削减选项与账**：
- (a) sky_light 不清零、sky_fall 改「全列写」（15 区间写 15，[0,col_max] 显式写 0）：把 0.88MB memset 换成等量 in-loop store——memset（无 load、SIMD、纯顺序）单位成本低于带分支散 store，**预计负收益或持平**。
- (b) opacity 不清零、air 格显式写 0（消清零依赖）：同上，把 memset 转移进热循环，**负收益**。
- (c) generation/epoch 标记懒失效：复杂度 + 每格额外比较，与 (a)/(b) 同病。
- **结论：2.65MB memset 本身 ≈ 2.65MB @ memset 带宽（数十 GB/s）≈ 30-90µs（open：实测）**，是 fill 943µs 的 3-10%；三个削减方向都把「更便宜的批量清」换成「更贵的散写」，**无正收益路径，建议维持现状**——除非 ii-a 落地后 sky 全列写自然覆盖 (a)（ii-a 降序写 15 + done 后格保持 0 依赖清零…… 依赖仍在，(a) 与 ii-a 组合需重新核算，标 open）。

### e2e 绝对值预期（对照回退 3.5s / 2025 chunks）

| 子项 | 内核上限节省 | e2e 上限（×2025） |
|---|---|---|
| ① lookup 内联 | ≤100 µs（保守 40-80） | ≤0.20 s（保守 0.08-0.16） |
| ② fill×sky_fall 融合 | 100-200 µs | 0.20-0.40 s |
| ③ 清零削减 | ≈0（无正收益路径） | ≈0 |
| **A 合计** | **140-300 µs（占内核 9-19%）** | **0.28-0.60 s / 3.5s = 8-17%** |

**结论性判断（draft）**：A 全落地后内核 1.59→~1.3ms，e2e 回退 1.25×→约 1.21×——**A 单独不可能达到 ON≈OFF（<1.05×）**；A 是低风险快赢层，不是主力。绝对上限（fill+sky_fall 全消，物理不可达）也只封顶 2.5s。排序建议：若 B/C 的上限推演（各自 .b2/.b3）显著高于 0.6s，A 应作为「随行小改」或被 C 吸收（见 §4）。

## 2. 等价性论证要点（golden 逐位门下）

| 子项 | 须论证 | 论证状态 |
|---|---|---|
| ① 直接索引 | 表构造期扩容后 `id ∈ [0, table.len())` 不变式成立（负 id 仍走 L120-122 分支）；缺 id 语义 = resize 时已填 default（L95），与 unwrap_or(table[0]) **语义等价仅当 default == table[0]**——构造上 table[0] 即 default 填充（L89/95），恒等 | 静态可闭合 |
| ② ii-a | 三点：col_max 恒等（§1 推演 1）+ sky 写集逐位恒等（推演 2）+ BFS 入队顺序无关（引用 round1 不动点论证，12 篇 L38）。golden 4 用例 FNV 覆盖顺序敏感回归 | 论证已列，待 golden 验证 |
| ③ 维持现状 | 无改动无论证义务；(a)/(b) 若实施需证「显式写 0 集 ≡ 清零集」（air 格 = op0 格 ∪ 显式 0 格） | N/A |

## 3. 实施风险与改动面

- **改动文件唯一**：`worldgen-core/src/light/mod.rs`。①= lookup（L119-124）+ from_json_str 构造期断言（L89-102）；②= fill 主循环反转 + sky 写融合（L210-247）+ sky_fall 删除（L266-273）+ phase 计时段重排（L192/259/278 计时锚随之变——**phase 分解口径变化须在 P4 报告声明 §9.7**）；③=无。
- **共享核波及 1.21.6**：light/mod.rs 是共享核（260905-01 拆分），1.21.6 侧须 golden 双版本 + gate ON 冒烟（架构 §6 已预置）；②的 y 反转不涉及版本数据差异（light_data.json 版本侧差异只在表内容，不影响遍历序）。
- **风险点**：① `unsafe get_unchecked` 的不变式靠构造期 debug_assert 把关（release 零开销）；② air 快路径新增 store 可能使 fill 净劣化（若 air 占比高）——**实施时须留 phase 分解对照，劣化即回退该子项**；queue 顺序反转对 block_bfs 8.8µs 影响可忽略。
- **回退方式**：逐子项独立 commit（①/② 各一笔），golden FAIL 或 phase 劣化即 revert 单 commit，ABI 零变化（签名/输出契约不动），门控默认关出厂无风险（架构 §6）。

## 4. 与 B/C 的组合关系

- **A-① 与 C**：C 落地后 Rust 侧变「palette 解码 + 查表 + col_max 单趟」，lookup 仍存在（表不变）——**A-① 的直接索引改造被 C 完全吸收**（同一行代码的两种上下文），若 C 排先则 A-① 不必单独做。但 A-① 成本极低（~10 行），作为 C 前的独立快赢不冲突。
- **A-② 与 C**：**正交且叠加**——sky_fall 融合与输入形态无关（col_max 语义两侧一致），C 落地后 ii-a 仍有效。
- **A-③ 与 C**：C 不改 scratch 语义，无交互。
- **A 与 B**：B 是 Java 收集侧，与内核零交集，正交。
- **排序含义**：若 C 排先，A 缩减为「仅 ②」；若 A 排先，C 实施时吸收 ①。**A 不排斥 B/C，但 A-① 与 C 有重复劳动面**——HOOK-2 排序时应避免两者相邻实施。

## 5. 判据预登记建议（A 单独验收）

1. **载体**：light_bench_real 同载体复测（rlib 内容指纹哨兵 #30，预热 8 + 256 chunks 批 wall，§9.7 声明与 round2 1.587ms 可比性）。
2. **主判据（内核微基准）**：`light_compute_phased` fill+sky_fall 合并口径 ms/chunk 前后对照——基线 fill 943µs + sky_fall 300µs = 1243µs，**达标线 = 合并后 ≤ 1000µs（省 ≥20%）**；进取线 ≤ 900µs。①单独 ≤ −40µs、②单独 ≤ −100µs 为子项达标（低于此即声明该子项无效并回退）。
3. **行为门**：golden 4 用例 FNV 逐位等值（C4 惯例，含 queue 顺序反转覆盖）；1.21.6 golden 同跑。
4. **e2e 附带观测（非验收线）**：四臂 ON 中位相对 round2 的 17.4s 下降量与内核节省 × 2025 自洽（±30% 内）。
5. **劣化哨兵**：任一 phase 相对基线劣化 >10% 即 revert 该子项 commit。

## 6. 开放问题（静态无法定，移交 P4）

1. 非 air 格占比 / 15-区间格数分布（决定 ①② 的真实收益，微基准加计数器一跑即得）。
2. 反向 y 遍历的预取/带宽实测差异（ii-a 风险面）。
3. memset 实际耗时（2.65MB @ 本机 memset 带宽，微基准单独计时即得）。
4. phase 计时口径变化（②删 sky_fall 段）后的分解表呈现方式。
