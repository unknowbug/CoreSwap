# 候选 b1 设计文档 —— est/Aquifer 缓存生命周期对齐 + 适度延长（core-worker，只读 + 设计）

- **status: draft**
- 置信度：candidate（静态阅读 Rust 生产源 + 引用 P1 调研已实测数字；Degraded/静态审查分层，未运行任何验证——本任务明确不运行命令）
- 日期标签：260903（随父 session Q-AQ1 / est-opt 课题）
- 前置输入：`java-est-cache-semantics.md`（P1 调研）+ `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`

---

## 0. ⚠️ 先决修正：P1 的 G5 表述与生产代码不符（开工前廉价独立验证结果）

按交接结论验证纪律，动笔前对 G5 做了 grep/read 核对，**G5 原表述（"fill 与 carver 各自 Aquifer::new，同 chunk 两阶段缓存不共享"）在生产路径上不成立**：

| 事实 | 位置 | 证据 |
|---|---|---|
| `fill_chunk_blocks` 只 `Aquifer::new` **一次**（:446），装入 `VanillaAquifer.va` | `WorldgenRust/src/worldgen_handle.rs:446-453` | 局部变量 `va` |
| carver 复用**同一** Aquifer 实例：`self.apply_carvers(&mut col, cx, cz, &mut va.aq, …)` | `worldgen_handle.rs:520` → `apply_carvers` 签名收 `aquifer: &mut Aquifer`（:589-590）→ `CarveContext { aquifer, … }`（:611-613） | **fill/NOISE 阶段与 CARVERS 阶段已经共享 water_levels / block_positions / surface_cache** ——与 Java 语义（sampler 挂 chunk 跨阶段复用）**一致** |
| 第二处 `Aquifer::new`（:547）在 `diag_pre_surface_column`——**bin-diag 增量诊断 API**（:535-540 注释明示），不做 carver、不与 fill 同 chunk 并存 | `worldgen_handle.rs:540-547` | 诊断路径自建实例是正确行为，非差距 |

**结论**：G5 应从「Rust 真实差距」改判为「**已对齐，无差距**」（❌ 排除）。P1 调研引用的 `worldgen_handle.rs:547` 是诊断 API 而非 carver 路径——本修正即取代记录（P1 文档 §3 G5 行待其归属方按 §15.4 结论取代链标注，本文不代改）。

**因此 b1 的真实内容重排为**：
- **b1-a（对齐项）**：SURFACE 阶段 `est_at` 闭包与 Aquifer 的 est 缓存**不共享**，是 chunk 内仅存的重复扫描点（详见 §1）；
- **b1-b（纯优化，Java 语义外，可选第二级）**：est 缓存跨 chunk 持久化（LRU + 容量上限 + blend 旁路闸门）；
- 原「修 G5」任务**取消**（无东西可修）——这本身是 b1 最重要的产出：防止按错误差距点投入实现。

---

## 1. b1-a：SURFACE 阶段 est 查询收敛到共享列缓存

### 1.1 现状（事实）

- Aquifer 的 est 缓存：`surface_cache: Vec<i32>`（32×32，`(bx>>2, bz>>2)` 量化索引），`estimate_surface_height`（`WorldgenRust/src/aquifer.rs:282-299`）——惰性填充、chunk 生命周期、哨兵 `i32::MIN` 未算 / `i32::MAX` 空列（:287-297，空列 `i32::MAX` **会**写入缓存 :297，G4 哨兵冲突静态读未见）。
- SURFACE 阶段独立的冷扫描：`fill_chunk_blocks` 内 `est_at` 闭包（`worldgen_handle.rs:495-505`）——`self.init.sample` 从顶到底 `step_by(8)` 扫描，**4 个 chunk 角列（:502-505）**，完全不经过 `va.aq.surface_cache`。每 chunk 额外 ≤ 4×48 = 192 次 initial_density 全价采样。
- Java 语义：`estimateSurfaceHeight` 走 sampler 同一张 map（ChunkNoiseSampler.java:222-226），SURFACE 阶段复用 NOISE 阶段已填的列。

### 1.2 ⚠️ 收敛前必须先裁决的语义分歧（对拍点 0，最高优先）

Rust `est_at` 与 `Aquifer::estimate_surface_height` 存在**两处行为差异**，直接路由到共享缓存会改变输出，必须先逐位裁决：

| # | 差异 | est_at（worldgen_handle.rs:495-505） | Aquifer（aquifer.rs:282-296） | Java |
|---|---|---|---|---|
| D1 | 列坐标量化 | **不量化**：`cx*16+15` 直接作为采样 x | `(bx>>2)<<2` 量化（+15→+12） | `BiomeCoords.fromBlock` 量化（ChunkNoiseSampler.java:223-224） |
| D2 | 扫描步长 | `step_by(8)` | `l -= 8`（:295） | P1 文档记 4 格步长（§1.2/§2.1），与两处 Rust 的 8 步不一致——**P1 G3「一致 ✅」存疑，需 probe 裁决** |

- D1：若 Java SURFACE 阶段走的是量化版 `estimateSurfaceHeight`，则当前 Rust surface 阶段 4 角采样点是**既有潜在错位**（+15 vs +12 列），b1-a 修复它属「对齐 Java」而非改变已确认行为——但必须先用 block_probe 在 +15 列差异可见的地形（角列表面陡变区）确认现状是否已偏离，再决定 b1-a 是「纯收敛」还是「顺带修 bug」。
- D2：步长 8 vs 4 直接决定 est 值格点分辨率与每列采样数（384/8=48 vs 384/4=96）。P1 调研称 Java 为 4、Rust 为 8 且标「一致」，自相矛盾；**以 trace/probe 为准裁决**（对 Java 侧列做单列 est 值对比即可，成本一轮）。
- **裁决前 b1-a 不动默认行为**：以 env 门控 `WG_EST_SHARED=1` 提供 A/B，双路径并存，block_probe Full 对拍后才翻默认。

### 1.3 改动设计（结构归属 / 生命周期 / 借用选型）

**选型：不做结构拆分，est 缓存留在 `Aquifer` 内，surface 阶段直接查询 `va.aq`。**

- **结构归属**：`surface_cache` 仍在 `Aquifer`（aquifer.rs:95），不拆独立 `SurfaceEstCache` 结构体——理由：① fill/carver/surface 三阶段在同一函数调用栈内，`va` 局部变量天然覆盖全程（worldgen_handle.rs:453 创建 → :520 carver → surface 闭包 :495-516 在 :520 之前，均在 `va` 存活期内）；② 拆分需要 `&mut SurfaceEstCache` 穿透 `fill_chunk` 泛型（4 个 density 后端 × VanillaAquifer），纯借用管道成本，无语义收益。
- **借用方案**：`est_at` 闭包改为捕获 `&mut va.aq`，调用 `va.aq.estimate_surface_height(x, z)`。借用冲突核查（静态）：`est_at` 闭包存活区间 :495-516，其间捕获的其余闭包 `biome_at`/`initial_density_at` 只借 `&self`（不可变，与 `&mut va.aq` 不冲突——`va` 是局部变量，非 `self` 字段）；`apply_carvers(&mut va.aq)` 在闭包生命周期结束后的 :520，无重叠。**无需 RefCell / once_cell / 字段分割**——任务书预置的三个借用选项全部不需要，零成本通过。
- **生命周期**：与现状一致（per-chunk、随 `fill_chunk_blocks` 返回消亡）——对齐 Java「chunk 管线内持久」，不引入任何新生命周期。
- **诊断 API**：`diag_pre_surface_column`（:547）维持自建 Aquifer，不改（诊断语义独立，且 P1 引它当 carver 路径是误读）。

---

## 2. b1-b（可选第二级）：est 缓存跨 chunk 持久化（LRU + 容量上限）

### 2.1 语义定性（沿用 P1 §4，本文确认）

est = `initial_density.sample(NoisePos{quantized x, y, z})` 的纯函数（世界种子 + 噪声参数决定，无 chunk 局部状态）——**除 blend 输入外**。这是 Java 不存在的纯优化（G6 双侧均无），**不是对齐项**；引入必须端到端逐位回归证明零退化，且性能口径按 AGENTS §四 端到端铁律 + §9.7 可比性声明。

### 2.2 blend chunk 旁路闸门（对拍点 3 的落地设计）

- P1 §4 对拍点 3：blend density 缓存是 per-chunk 预填（ChunkNoiseSampler.java:52-53、142-154），跨 chunk 缓存若混入 blend chunk 的 surface 估算即被污染。
- Rust 现状（静态）：`WorldgenHandle` 是无 ProtoChunk 邻居的独立 chunk 生成，**未见 blend 路径**（无 `cachedBlendAlphaDensityFunction` 对应物；grep 全 src 无 blend density 实现）。闸门设计因此分两层：
  1. **现状闸门**：`WorldgenHandle` 增加 `blending_active: bool` 字段（构造时常量 false + 注释锚定「Rust 未实现 old_generation blend」）；L2 查询入口 `if self.blending_active { bypass L2 }`——闸门成本 = 一次 bool 读，chunk 级判断一次，不违反热路径诊断禁令。
  2. **未来防御**：若后续实现 blend，`blending_active` 置真即自动旁路；并在 blend 实现的验收清单里加一条「开启 L2 下 blend chunk 输出与 L2 关闭逐位一致」。

### 2.3 数据结构与淘汰策略

- **key**：`u64 = (量化列坐标 pack) ^ (世界上下文代际 u64 的混入)`。世界上下文代际 = `WorldgenHandle` 构造时分配的 `cache_epoch: u64`（seed/min_y/height/density 源版本任一变 → 新 epoch，旧代际整表作废）——对应 P1 对拍点 3 的「key 必须含 (worldSeed, noiseRouter 参数)」，用代际号替代把 seed 烧进每条 key（省 8B/条）。
- **value**：`i32`（含 `i32::MAX` 空列哨兵——G4 语义原样入缓存）。
- **容器**：`HashMap<u64, (i32, LruPrevNext)>` 或直接 `lru`-语义手写双向链表 + HashMap（不引入外部 crate，项目现状无 lru 依赖）；或最简版：固定容量直接映射表 `Vec<(u64, i32)>` + 线性探测 + **时钟（clock/二次机会）淘汰**——est 列访问呈强空间局部性（13-offset 邻域），clock 与 LRU 命中率差距预期可忽略，实现成本低一个量级。**推荐 clock 版**。
- **容量/内存界**：容量 `2^17 = 131072` 条 × (8B key + 4B val + 4B 元数据) ≈ **2 MB 硬上限**（对齐 17×17 邻域 carver 的访问窗口：一次 carver 扫 289 chunk 的列坐标，(17×16/4)² ≈ 4624 量化列/窗口，131072 条足够覆盖工作集 + 数倍余量）。溢出走 clock 淘汰，无峰值膨胀。
- **归属与并发**：挂在 `WorldgenHandle` 字段。⚠️ `fill_chunk_blocks(&self, …)` 是 `&self`——共享可变缓存需要 `Mutex<EstL2>` 或 `RwLock`。**选 `Mutex`**：est 查询粒度 ~µs 级、锁内只做 hash 查找+链表挪动，争用可忽略；RwLock 的读锁升级（读 miss 后要写）反而要重试逻辑。若未来Mutex争用实测可见，再降级为 `DashMap`-式分片（不在本设计范围）。
- **降级开关**：`WG_EST_L2=off`（env，chunk 级读一次）+ `cache_epoch` 双保险；默认 **off**，Full 回归通过后才翻默认。

### 2.4 逐位安全性论证链（供 judge 审查）

1. est 纯函数性：`initial_density`（`Arc<DensityFunction>`，aquifer.rs:88）跨 chunk 共享同一实例（`self.init.clone()` :448），采样输入只有坐标——同 epoch 下 (x,z) 相同 ⇒ 值相同。
2. blend 旁路：§2.2 闸门。
3. 缓存命中/未命中输出一致：L2 miss 走的正是现 `estimate_surface_height` 路径（L2 填充点 = aquifer.rs:297 写 `surface_cache` 的同时回填 L2），读路径优先级 L2 → surface_cache → 全价扫描。
4. 唯一非确定性来源 = D1/D2 语义分歧（§1.2）——**b1-b 依赖 b1-a 的裁决结论先行**（est 的「正确值」定义统一后 L2 才有唯一定义）。

---

## 3. 预期收益量级（口径 + 依据，无自由参数）

已实测数字（来源：confirmed 记录 / P1 调研）：est 冷扫描 ~15.5 ms/chunk（7342 initial_density 采样/chunk × ~2117ns）；13 offset 邻列；214 calls/chunk（去重比口径，P1 引用）。

| 项 | 收益量级 | 推算（全部来自实测口径） |
|---|---|---|
| ~~修 G5~~ | **0** | G5 已排除（§0）——fill/carver 已共享，无重复可省。此项收益为零是修正后的诚实结论 |
| b1-a（surface est 收敛） | ≤ ~0.4 ms/chunk（≈ est 冷扫描的 2.5%） | 4 角列 × ≤48 采样（step 8）× 2117ns ≤ 192×2117ns ≈ 0.41ms；且仅当 D1 裁决为「现状已错、修复不改值」时才有纯收益，否则是正确性收益非性能收益。**量级：小** |
| b1-b（L2，warm 稳态） | 上界 ~15.5 ms/chunk（est 冷扫描全额）；实际 = 15.5 × (1 − miss率) | miss 率**未实测**——13-offset 邻域跨 chunk 重叠率高（相邻 chunk 的 aquifer cell 邻域互相覆盖量化列），但具体命中比无数据，**禁止编造**。实现后第一动作 = `aquifer_surf_watch` 计数器扩 L2 命中/未命中两计数，实测 miss 率后再定收益表述。保守表述：**est 冷扫描 15.5ms/chunk 是可消除上界，实测前不承诺比例** |

声明（§9.7 可比性）：上述均为「单 chunk 阶段分解」口径（qaq1/qpd1 系探针），与端到端整批 wall 口径不可直接比；验收以端到端大样本（≥256 chunks，AGENTS §四）为准。

---

## 4. 逐位对齐风险清单 + 回归验证方案

### 4.1 风险清单

| # | 风险 | 等级 | 缓解 |
|---|---|---|---|
| R1 | D1 量化分歧：收敛 est_at 到共享缓存若改变 +15 列采样坐标 → surface 输出变 | 高（行为） | 先 block_probe 裁决 Java SURFACE 输入是否量化；A/B 门控 |
| R2 | D2 步长分歧（8 vs P1 声称的 4）：est 值本身可能已与 Java 错位（格点分辨率差 4 格） | 高（行为，且是**既有**风险非 b1 引入） | 单列 Java↔Rust est 值对拍（P1 G3 存疑的裁决） |
| R3 | G4 空列哨兵：`i32::MAX` 空列入 L2 后被 clock 淘汰/重算，需与 Java「MAX 也入缓存、永不重算」语义核对（L2 淘汰引入了 Java 没有的重算——重算结果相同故逐位安全，但需声明） | 低 | 纯函数性保证重算同值；对拍点 1（海洋深列）验证 |
| R4 | L2 跨 chunk：epoch 混入错误（同 seed 多 WorldgenHandle 并存时互串） | 中 | epoch = handle 构造序号 + seed 哈希；单测两 handle 不同 epoch 不互命中 |
| R5 | Mutex 引入的并发回归（fill 并行路径） | 低 | 锁粒度=单条缓存操作；mt_fill 类基准回归确认无反降 |
| R6 | 诊断 API（:547）若误接 L2 会在诊断计数里混入 L2 命中，污染 qaq1 系探针口径 | 低 | 诊断 API 保持自建 Aquifer + L2 旁路 |

### 4.2 回归验证方案（block_probe Full 分层）

1. **基线冻结**：改动前 block_probe Full 导出基线（seed + 区域与既有 confirmed 口径一致，三查纪律）。
2. **分层用例**：
   - L0 全量：默认配置逐位 diff = 0（b1-a 默认路径不变时必须 0；翻默认后同样 0）。
   - L1 aquifer 区：水域/含水层 pocket 密集区（est 冷扫描触发区）+ **海洋深空列**（G4/R3：整列 ≤0.390625 → i32::MAX 哨兵路径）。
   - L2 chunk 角列：+15 角列表面陡变地形（R1/D1 差异可见性用例）。
   - L3 跨 chunk：32×32 region 端到端（L2 缓存跨 chunk 命中的实际作用域；含相邻 chunk 13-offset 邻域跨界列）。
   - L4 A/B 门控矩阵：`WG_EST_SHARED` × `WG_EST_L2` 四臂逐位互 diff 全 0（语义安全证明）+ 四臂 wall 计时（收益证明，端到端口径 ≥256 chunks）。
3. **计数器**：`aquifer_surf_watch` 扩展 `[calls, init_scans, l2_hit, l2_miss]`，miss 率实测落盘后才允许在文档写收益比例。
4. 通过判据：L0-L3 逐位 0 + L4 语义 diff 0 + 扫描门禁 `scan_cpp_anchors.py`（Rust 侧对应 anchor 标注校验）invalid=0 → b1 整体 candidate；confirmed 由用户拍板。

---

## 5. 实现工作量估计（文件/函数级）

| 步骤 | 文件 / 函数 | 内容 | 估量 |
|---|---|---|---|
| 0（裁决，先行） | diag bin ×2（`bin-diag/`，如 `qaq1_est_ab.rs` 新建） | D1/D2 裁决探针：Rust est_at vs estimate_surface_height vs Java 单列 est 值 | 0.5 天（含主会话执行） |
| 1（b1-a） | `worldgen_handle.rs:495-505`（est_at）、:513-516（调用点） | est_at 改路由 `va.aq.estimate_surface_height`，`WG_EST_SHARED` 门控 | 0.5 天 |
| 2（b1-b 结构） | `aquifer.rs`（estimate_surface_height :282-299 写点回填）、新文件 `est_l2.rs`（clock 缓存 ~150 行）、`worldgen_handle.rs`（字段 + `cache_epoch` + Mutex） | L2 + 闸门 + epoch | 1-1.5 天 |
| 3（计数器） | `aquifer.rs` 计数器段（:58-62 附近模式照抄） | surf 计数扩 [l2_hit, l2_miss] | 0.5 天 |
| 4（回归） | block_probe 用例 + 四臂矩阵 + 端到端基准 | §4.2 全套 | 1-1.5 天（主会话执行） |
| 合计 | 3 文件改 + 2 新文件 | — | **~4-5 天**（b1-a 单独出货 ~1 天；b1-b 依赖步骤 0 裁决结论） |

---

## 6. 交付检查清单自检（GUIDE §四）

- [x] 数字全部来自 confirmed/P1 实测（15.5ms、7342、2117ns、13 offset、214 calls），未编造；miss 率明标「未实测」
- [x] G5 修正按取代记录形态呈现（§0），原 P1 结论未代改
- [x] 每个设计点含机制理由（借用选型、闸门、淘汰策略）
- [x] 风险含既有风险（D1/D2）与新增风险（R3-R6）分离
- [x] status: draft，未自授 candidate 以上
