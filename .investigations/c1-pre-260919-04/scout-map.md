# scout-map：C-1（域共享 fill/传播增量化）立项前置 G3 收敛性约束勘探

- 勘探角色：recode.scout（只读）
- 日期标签：260919-04（沿用任务指派标签）
- 一手源码：worldgen-core/src/light/mod.rs、versions/1.20.1/rust/src/jni_bridge.rs、versions/1.20.1/docs/12-lighting.md
- 置信度：draft（全部断言带 file:line；机制不明处标 open）

---

## 1. G3 性质「整域快照一次重载收敛」的实现位置与依赖

### 1.1 现域批路径的三段结构

**输入快照点（JNI copyin，一次性）**
- 域批入口 `lightComputeDomain`：Java 传入 `blocks25`（5×5 chunk = 25 chunk raw id，总长 2457600）+ `out`（885168 B）——jni_bridge.rs:439-441，常量定义 jni_bridge.rs:284-285。
- 快照动作 = `env.get_int_array_region(&blocks25, 0, &mut b25)` 一次整块拷入 thread_local 缓冲——jni_bridge.rs:461。packed 路径（`lightComputeDomainPacked`）的快照点等价：n 帧一次解码进同一 b25（jni_bridge.rs:577，`light_decode_packed_domain`），后帧覆盖重叠 = Java 侧 blocks25 arraycopy 顺序语义（mod.rs:317-319）。
- 帧契约明确：5×5 块覆盖 chunk [X..X+4]×[Z..Z+4]，9 中心 = [X+1..X+3]×[Z+1..Z+3]，边缘 chunk 只读不产出（mod.rs:391-395）。

**迭代/传播到不动点的循环结构**
- 外层：`light_compute_domain_impl` 9 中心循环（mod.rs:441-468）——每中心把对应 3×3 子窗从 b25 拷进 b9（mod.rs:446-452），再调同一内核 `light_compute_inner`（mod.rs:459-467）。
- 内核四相（mod.rs:472-630）：① fill（blocks9→opacity/emission，同趟收 block 光种子与 col_max，mod.rs:508-545）；② block light BFS 到不动点（`bfs_propagate`，mod.rs:552；BFS 本体 mod.rs:635-676，传播规则 `try_spread`：new_level > light[n] 才写入并入队 = 单调上升 ⇒ 不动点唯一，mod.rs:694-697）；③ sky 直落（col_max 区间直取，mod.rs:564-571）+ 边界种子收缩（mod.rs:583-614）+ sky BFS（含 15 直落特例，mod.rs:615、689-693）；④ 导出中心 chunk（域坐标 16..32）+ 均质性 flags（`export_center`，mod.rs:623、701-737）。
- 关键：**这里没有任何跨调用/跨中心的光照状态**——每中心从零快照重算整个 48×48×384 域，Scratch 每相清零复用（mod.rs:492-501），「不动点」是单次 BFS 内部的收敛，不是跨轮收敛。

**输出写回语义**
- 9 段连续输出，段序 k = kz*3+kx，每段 outBlock(49152)++outSky(49152)++outFlags(48)（mod.rs:394-395）；JNI 一次整块 copyout `set_byte_array_region`（jni_bridge.rs:495、609）。
- 等价门：域批 9 中心输出与 per-chunk 路径逐位相等（测试 `light_compute_domain_bitwise_equivalence`，mod.rs:782-826）。

### 1.2 G3「一次重载收敛不动点」性质的机制本质（12 篇 :158-181 论证链）

12-lighting.md:158-181（confirmed，verdict-260917-01）：
- G3 drift 基底 = **legacy per-chunk 路径的轮次级不收敛**：run3 legacy 仍 200 changed（9.88%）且漂移集换血（基底交集仅 48.3%），5.93→6.27→9.88% 逐轮上升，**无不动点**（:167）。
- **域批路径一次重载收敛至不动点**（run3 changed 2，0.10%≈噪声地板）（:168）；settle 持久性 rL=99.2% 排除快照窗/停服时机效应（:166）。
- 机制解释（12 篇自己标注为**静态推演、无直接探针**，不属 confirmed 结论内容，仅存候选草稿 .b3 §1-§2）：「域批 5×5 全帧重算覆盖 legacy 陈旧历史」（:173）。

**本勘探的机制定位（基于一手源码，标 candidate 级观察）**：域批收敛性的结构性来源是**无状态纯函数**——内核输出是 blocks25 快照的确定函数（无持久光照层、无跨任务状态、BFS 单调不动点，mod.rs:64-79 注释「无状态纯函数 + 清零复用缓冲」；等价性由编译期借用分离复证，mod.rs:77-78）。重载时同输入 ⇒ 同输出 ⇒ 与上次定值逐位一致 ⇒ changed=噪声地板。**任何引入跨任务缓存/增量的设计（C-1 方向）都会把「输出 = 本帧快照的函数」破坏为「输出 = 本帧快照 + 历史状态的函数」，这正是 legacy per-chunk 轮次级 drift 的机制同构**（12 篇 :164-168 的被证伪形态）。→ C-1 设计 MUST 回答：缓存失效条件能否保证「同输入必同输出」（例如缓存 key 恰为该 chunk 全部光相关输入）。

---

## 2. 域重叠结构（9 中心 × 3×3 → 5×5）

### 2.1 精确 chunk 布局
- 5×5 = 25 chunk；中心 c25 = (kz+dz9)*5 + (kx+dx9)，dz9/dx9∈0..3（mod.rs:448-450）；域内扁平索引 dom_index(x,y,z) = (y*48+z)*48+x（mod.rs:51-53），域 x/z 0..48、y 0..384（mod.rs:18-24）。
- 每 3×3 子窗映射：b9 chunk c9(dz9*3+dx9) ← b25 chunk (kz+dz9)*5+(kx+dx9)（mod.rs:444-452）。

### 2.2 冗余度（输入是否完全相同）
- **fill 相（①）是逐格纯查表**：opacity[i]/em 由 blocks9 单格决定 + col_max 逐列 max（mod.rs:508-545）⇒ fill 结果 per chunk 只依赖该 chunk 自身方块（邻接无关；col_max 也是列内运算）。
- **block/sky BFS（②③）依赖 3×3 邻域**，但 3×3 窗对中心 chunk 的**边距恰 = 16 格**（中心 chunk 在域坐标 16..32，mod.rs:708；域宽 48），> 光最大传播距离 15 ⇒ **中心 chunk 的光照值与 3×3 窗外的方块严格无关**。因此：同一 chunk 的方块数据在 b25 中只有一份（快照一致性由构造保证），各中心对它的计算输入（含邻接 16 格边距内）在重叠中心间**完全相同** ⇒ 重叠 chunk 的 fill/传播计算是完全冗余重复，不是近似重复。
- 重复计数：5×5 中 25 chunk 作为「被全量重算的域成员」参与次数——9 个 3×3 窗并集下，角 4 chunk 各 1 次、边 12 chunk 各 2 次、内 9 chunk 各 4 次（按 3×3 滑窗计数；另有「作为邻域参与其他中心域」的 9× 计法见 12 篇 :141 L5 行：邻 chunk 被中心重算 1 次 + 作为邻居参与 8 次，总结构量 ≥9×）。
- open：以上滑窗重叠计数为静态推演（未跑探针复核 12 篇 :141 的 9× 口径实测值）；两者口径不同（域成员 vs 邻域参与），C-1 收益建模时须先定口径。

### 2.3 重叠 chunk 的传播边界相互影响
- 现实现中**互不影响**：每中心独立建域、独立 BFS、互不见对方结果（Scratch 为 per-batch 局部，9 中心循环共享但每轮清零，mod.rs:439、488-501）；输出只取各域中心 16×16×384（export_center，mod.rs:708-731），重叠区域的传播结果只用于给本中心当边距，不导出。
- 等价性已由逐位等价门钉死（mod.rs:782-826）。

---

## 3. 可共享 / 不可共享状态清单

### 3.1 跨任务可缓存候选（机制事实列举，不做决策）
| 候选 | 依据 | 纯度 |
|---|---|---|
| fill 结果 per chunk（opacity 数组 + col_max 列表） | fill 逐格纯查表，仅依赖本 chunk 方块（mod.rs:508-545） | chunk 局部，无邻接依赖 |
| emission 种子队列 per chunk | 同趟收集（mod.rs:535-538） | chunk 局部 |
| packed 解码产物（b25 chunk 段） | light_decode_packed_into 每 chunk 独立解码（mod.rs:229-314） | chunk 局部 |

### 3.2 触 drift 风险面
- **传播（BFS）结果 per chunk 不可直接缓存为跨任务状态**：BFS 值依赖 16 格边距内邻域方块；缓存即引入「历史快照时序」依赖——与 12 篇 :164-168 被证伪的 legacy 轮次级不收敛机制同构。缓存 key 若不能覆盖「该 chunk 全部 3×3 邻域输入的当前快照值」，则同输入不再保证同输出，G3 性质破坏。
- **快照时序依赖点**：blocks25 是 Java 侧在任务提交时刻的 writePacket/arraycopy 帧（jni_bridge.rs:432-436 契约；重叠帧后写覆盖语义 mod.rs:317-319）——若 C-1 缓存跨任务复用旧帧的某 chunk，则该 chunk 不再反映本帧值。
- **LightStorage 式持久层不存在**：我方无 vanilla LightStorage 增量引擎（12 篇 :141 L5+L6 行），G3 收敛性正是「无持久层 + 全量重算」的副产品——引入持久层 = 直接触碰该性质的载体。

---

## 4. Scratch / b9 / 缓冲现状

- **light_compute_domain_impl 头部分配**（mod.rs:439-440）：
  - `Scratch::new()`（mod.rs:93-101）：五 Vec——opacity/block_light/sky_light 各 BLOCKS9_LEN=884736 B（清零后 ~2.53 MB）、queue Vec<u32>、col_max 48×48 i32（mod.rs:81-88）≈ 共 ~2.7 MB（与任务背景数字吻合）。
  - `b9 = vec![0i32; BLOCKS9_LEN]` = 884736×4 B ≈ 3.5 MB，域批内 9 中心复用（mod.rs:440 注释）。
  - Scratch 在 9 中心循环间共享一个（mod.rs:388、439），跨调用不复用（CP-1 注释自认「跨调用复用收益未量化 @anchor.idk 级」，mod.rs:91-92）。
- **C-4 已并线的 thread_local**（jni_bridge.rs:296-326）：
  - `LIGHT_B25`（2457600 i32 = 9.4 MB）+ `LIGHT_OUTD`（885168 B ≈ 0.9 MB），260919-03 .b2 C-4 引入，每使用全量覆写（b25 copyin/解码零填、outv 内核全写出），jni_bridge.rs:311-316、455-463。
  - packed 帧缓冲 LIGHT_DM/DP/DS/DL 按需 resize（jni_bridge.rs:317-325）。
  - per-chunk 路另有 LIGHT_B9/OB/OS/OF/PM/PP/PS（jni_bridge.rs:296-310）。
- **C-1 若做域共享，这些结构的变化面（机制事实）**：Scratch 五 Vec 与 b9 均为「3×3 域」形态（48×48×384）；若共享粒度上移到 5×5 chunk 级（80×80×384），opacity/block_light/sky_light 容量 ×(80/48)²≈2.78，dom_index/DOM 常量、BFS 邻域边界判断（mod.rs:657-674 的 `x>0`/`x+1<DOM` 等）全部依赖 `DOM=48` 常量（mod.rs:19），改动面 = 常量参数化或编译期泛化；queue 的 u32 域索引打包上限 884735 < u32::MAX（mod.rs:49）需重验 80×80×384=2457600 仍安全（安全，但断言位置在注释非代码）。

---

## 5. 候选方案原始素材（只列机制事实，不做决策）

### 5.1 「5×5 chunk 域内 chunk 级 fill 共享一次」的最小改动切入点
- **fill 共享**：fill 是逐格纯函数且 col_max 列内（mod.rs:508-545），可在 5×5 全域（或 per chunk）算一次，9 中心复用——最小切入点为把 `light_compute_domain_impl` 的 9 中心循环（mod.rs:441-468）改为「先全域 fill 一趟，再 9 中心各自 BFS+export」；数据面天然就绪：b25 本来就是 5×5 一份（无重复存储），冗余只在计算侧。
- **解码共享已部分存在**：packed 路径 `light_decode_packed_into` 已支持任意 chunk_bases 目标布局（mod.rs:229-314，260919-03 泛化即为此类改动预留的形状），域批解码 `light_decode_packed_domain` 后帧覆盖重叠（mod.rs:339-364）——若共享解码，重叠帧的重复解码也可消除。
- **subcopy 消除**：9 次子窗拷贝（mod.rs:446-452，诊断面 subcopy 计时 mod.rs:410-412、453-455）在 5×5 单域形态下整体消失。

### 5.2 「一次重载收敛」性质在 chunk 级共享下的保持/破坏面
- **保持面**：fill/解码共享不触 G3——它们是本帧快照的纯函数，9 中心共享同一份本帧 fill 结果，输出仍是「本帧 blocks25 的确定函数」，逐位等价于现路径可由等价门（mod.rs:782-826）直接复核。
- **破坏面（机制边界，非方案建议）**：若共享升格为**跨任务缓存传播结果**（BFS 值复用），则输出依赖历史快照（§3.2）——G3 confirmed 性质（12 篇 :168「一次重载收敛至不动点」）的可证性即被打破，除非缓存 key 严格等于「该 chunk 及其 16 格边距邻域的全部当前帧输入」且失效即重算（此时缓存退化为纯函数 memoization，收敛性保持但收益取决于失效频率——open：失效频率无实测数据）。
- **同一任务内的传播共享与跨任务不同**：9 中心在同一 b25 快照上各自 BFS，若改为 5×5 单域一次 BFS + 9 次 export，则 BFS 不动点在更大域上计算——机制上中心 chunk 的 16 格边距保证（§2.2）使结果仍逐位一致（传播影响半径 ≤15 < 16），但该断言目前是静态推演，open：需等价门扩展到 5×5 单域形态验证。

---

## Open 清单汇总
1. §2.2：滑窗重叠计数（1/2/4 次）vs 12 篇 :141 的 9× 邻域参与口径——两口径未对齐，C-1 收益建模前置。
2. §4：Scratch 跨调用复用收益「未量化 @anchor.idk 级」（mod.rs:91-92 自认）。
3. §5.2：5×5 单域一次 BFS 的逐位等价——静态推演，需等价门扩展验证。
4. §5.2：跨任务 memoization 的失效频率——无实测数据。

## 产物
- 本文件：.investigations/c1-pre-260919-04/scout-map.md
