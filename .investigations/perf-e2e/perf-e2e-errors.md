# Rust worldgen 端到端性能定位（vs Java）：错误与根因清单

> 载体：`.investigations/perf-e2e/perf-e2e-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「Rust worldgen 性能定位（端到端 vs Java）」里程碑中的错误。结论性对齐/性能数据见 07 篇与 10 时间线；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。
> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链路闭合后，进入端到端性能优化。三个关键错误链条：①「Interpolated 慢 100× → 放弃」被 judge 推翻（双层污染）；② 诊断代码热路径每点执行污染；③「Java 60ms」错误基准导致「积累性差异」误判。
> 编号：本课题用 **P 系列**（perf-e2e），与 rust-mod-load（M 系列）/ functional-errors（F 系列）/ carver-port（C 系列）区分，避免跨课题编号混淆。
> 数据源：`cmd-output/e2e_java_vs_rust.txt`、`cmd-output/diag_pollution_cleanup.txt`、`cmd-output/base_breakdown.txt`、`cmd-output/aquifer_internal_profile.txt`、`.investigations/rust-mod-load/review-density-candidate.md`（judge 意见全文）。

---

## P1. 「Interpolated 慢 100× → 放弃」被 judge 推翻——双层污染假象

### 现象
- `density_interp_bench` 测量（`cmd-output/density_interp_correction.txt`）：裸 finalDensity 采样 6.4ms/chunk（0.065μs/pt），在最终树外层包 `Interpolated(4×4×8 网格)` 后 **632ms/chunk（约 100× 慢）**，且采样出现 44% 差异（三线性插值近似破坏对齐）。
- 据此得出结论：「Interpolated 优化方向错误（慢 + 破坏正确性），放弃」。
- judge 复核 **推翻** 该放弃结论：单层 Interpolated 包装纯 SplineDF（sloped_cheese，无缓存污染）= **加速 70×（83.74ms → 1.19ms）**；「慢 100×」是**双层包装污染假象**，Interpolated 不应放弃，反是密度优化正解。

### 根因（机制）
- **双层 Interpolated**：`overworld.json` 的 `minecraft:final_density` 树**内部已含 `minecraft:interpolated` 节点**。探针却在这棵 raw 树**外层再包一层** `Interpolated(new(raw,...))` → 形成**双层插值**。
- 外层 Interpolated 的 build_grid 网格点**跨多个 chunk**：外层网格点（如 x=16k、z=16m 边界）会反复清空**内层 chunk 网格缓存** → 内层 mesh 跨 chunk **雪崩重建**。
- 诊断探针（`density_interp_diag`，judge 新增）量化：内层 Interpolated 裸树每 chunk grid 采样 6029 次；外层再包后膨胀到**每 chunk 175 万次（112M/64，291×）**——这就是「632ms / 100×」的真相。
- 「44% 采样差异」同理是双层二次插值叠加 + 对插值语义的误解：MC 密度本就该插值（final_density 内部即 interpolated），单层插值误差是固有精度产物，非「不可用」。

### 定位（诊断方法）
- judge 审查（`.investigations/rust-mod-load/review-density-candidate.md`）用**新增单层对照诊断探针（density_interp_single + density_interp_diag）**复现：
  - 单层 Interpolated 包纯 SplineDF 子树 → 加速 70×——证明「100×」来自包装方式，非方向本身。
  - 网格采样计数对拍 → 6029 → 291× = 175 万次——证明内层网格跨 chunk 雪崩重建。
- **测量基准审查**（judge 三源核对：data 记录 ↔ 探针源码 ↔ 实际复现）是发现污染的入口——数值可复现（6.7/628ms 与记录 6.4/632 一致）**不代表基准正确**。

### 修复
- **撤销「Interpolated 放弃」结论**。Interpolated（或 DFC 直排）反是密度优化正解，方向与 AGENTS.md 铁律（SplineDF 树遍历 = 慢根源 → C2ME/DFC 直排 + 网格缓存）一致。
- 评估 Interpolated 性能时**用单层包装**（包纯 SplineDF 子树），不要包已含 interpolated 的 final_density 外层。
- 真正该放弃的是「在 final_density 外层再包一层 Interpolated」这一**本就冗余的双层插值写法**。

### 教训（可复用判错经验）
- **测量基准必须先确认无污染，再做优化方向结论**：一个「看起来可复现」的慢点，可能是多层包装 / 缓存雪崩的假象，不代表算法方向本身慢。
- **多层数组/网格 / 树包装需查「缓存雪崩」**：外层网格点跨 chunk 反复清空内层网格 → 灾难性膨胀（291×）。看到「某方向慢 N×」先问：是不是重复分层包装把缓存击穿了？
- **数值可复现 ≠ 基准正确**：复现同一数字只能证明测量稳定，不能证明测量衡量了正确的东西。judge 审查的价值正在于重新审视基准方法论，而非只复核数字。

---

## P2. 诊断代码热路径每点执行污染（27% 退化）

### 现象
- 用户提醒「断点污染」坑后复查：诊断代码放在热路径**每点执行**，导致性能退化 **61.5ms → 44.9ms（27% 退化）**（`cmd-output/diag_pollution_cleanup.txt`，region 200,200 单线程 median）。
- 污染点（每 chunk 98,304 点执行）：

| 污染点 | 位置 | 每点执行 | 清理后 |
|---|---|---|---|
| `AQPROF_ENABLED.load(Relaxed)` | aquifer.apply | 98304 次/chunk atomic load | 移除 |
| `Instant::now()` ×3 | terrain.fill_chunk | 98304 次/chunk | 移除 |
| `std::env::var("WG_SKIP_AQUIFER")` | terrain.classify | 98304 次/chunk env 查询 | 移到 chunk 级 |
| `std::env::var("WG_SKIP_OREVEIN")` | worldgen_handle 矿脉循环 | 每 rock 点 env 查询 | 移到 chunk 级 |

### 根因（机制）
- 诊断代码（env 查询 / atomic load / `Instant::now`）即使**门控默认关闭**，门控判断本身就有开销：`std::env::var` 每次查环境变量表、`AtomicBool::load` 每次原子读取、`Instant::now` 每次时钟采样。
- 这些操作在 **98304 点/ chunk × 16 chunk 批量**下被放大到 `~157 万次/批量`，单点微秒级开销累积成显式 27% 退化。
- 本质 = **「测量/探针污染铁律」同族的另一种形态**：先前铁律关注**计时探针**（WG_PROFILE 每采样 steady_clock + 原子 → 原子竞争）；本次是**诊断/门控代码本身**（env/atomic/now 每点执行）污染数据面，非计时探针竞争。

### 定位（诊断方法）
- **用户主动提醒「断点污染」坑**——经验触发（不是工具发现），指出「每点执行诊断」这个模式要警惕。
- 复查代码路径确认 `fill_chunk`/`classify`/`aquifer.apply`/矿脉循环里的 env/atomic/now 调用点都在内层点循环里。

### 修复
- 迁移到 **chunk 级门控**：env 查询（`WG_SKIP_AQUIFER`/`WG_SKIP_OREVEIN`）在 **chunk 开头判断一次**，读入局部变量传进点循环；`Instant::now` ×3 移出点循环或仅 chunk 级采样；`AQPROF_ENABLED.load` 移除（或编译期 feature gate）。
- 修复后性能从 61.5ms 恢复到 **44.9ms**。

### 教训（可复用判错经验）
- **诊断代码绝不能放热路径每点执行**——即使门控默认关闭，env 查询 / atomic load / `Instant::now` 自身就有开销；点级（98304×/chunk）放大成千上万次。
- **诊断门控必须 chunk 级判断一次，或编译期 feature gate**——把「是否跑诊断」的决定从「每点查」上移到「每 chunk 查一次」。
- 与「测量/探针污染铁律」同族：**性能数据可信度 = 数据面有无被诊断/探针代码污染**。点级诊断既污染计时（探针竞争）又污染真实负载（env/atomic/now 每点执行）。

---

## P3. 「Java 60ms」错误基准导致「积累性差异」误判

### 现象
- 早期 `fair_java_vs_rust.txt`（`cmd-output/`）测出「Java 60ms/chunk」，据此判定 Rust 与 Java 接近 / 已达标。
- 本次端到端精确对比（`cmd-output/e2e_java_vs_rust.txt`，region 200,200，单线程，2026-08-29）修正：**Java 原版（WorldGenBench FULL 含树花植被，充分预热 JIT）稳定后 chunk 5-24ms，中位数 ~8-9ms/chunk**（排除第 1 个 298ms 冷启动）。
- Rust（fill_chunk_blocks 无树花，清理诊断污染后）：median **44.9ms/chunk** → Rust 比 Java 慢 **~5 倍**。

### 根因（机制）
- **JVM JIT 预热不足**：Java 基准若未充分预热，首个 chunk 的冷启动开销（JIT 编译、类加载、解释执行）会把中位数拉高。早期「Java 60ms」测在 **JIT 未热**状态下，是**错误基准**。
- **「积累性差异」被错误基准掩盖**：逐段优化（`fill_chunk` 内一段 vs 上一段）用「Java 60ms」当锚点，会让 Rust 看似「每段都在进步 / 已达标」——但真实 Java 只要 8-9ms，Rust 远未达标。逐段 vs 上一段的相对优化**不暴露与最终标准的绝对距离**。

### 定位（诊断方法）
- **用户提出「积累性差异」担忧**（AGENTS.md 端到端性能对比铁律的催生者）：点出「逐段优化用错误基准会掩盖 Rust 未达标的真实差距」。
- 端到端公平对比：`e2e_java_vs_rust.txt` 用**同一 region（200,200）、单线程、同测量口径**，且 Java 侧 `WorldGenBench FULL` **充分预热后取稳定中位数**、排除首个冷启动 chunk——修正后暴露真实 5× 差距。

### 修复
- **性能优化必须端到端对比充分预热的 Java 原版**（最终标准 = 比 Java 强），禁止只做「逐段 vs 上一段」。
- Java 基准必须**充分预热**（worldgen bench warmup 后 JIT 热，取稳定中位数，排除首个冷启动 chunk）。
- AGENTS.md 新增「端到端性能对比铁律」（用户拍板）。

### 教训（可复用判错经验）
- **端到端对比必须用充分预热的 Java 基准，禁止逐段 vs 上一段**——逐段相对优化掩盖「积累性差异」，让 Rust 在错误锚点（未热 Java）上误判达标。
- **Java（JVM）基准必须先让 JIT 热起来**：冷启动 chunk（JIT 编译/类加载/解释）会把中位数拉高成假慢/假公平；取稳定中位数 + 排除首个冷启动 chunk。
- **「相对 vs 绝对」陷阱**：优化对比的锚点必须是最终标准（充分预热的 Java），不是「上一段 Rust」。相对进步 ≠ 达标。

---

## P4. 「barrier 是 aquifer 大头 → 加 Cache2D 缓存」方向被实测推翻（barrier.sample 仅 0.1%）

### 现象
- 依据早前 aquifer 内部 profile（污染态）得出「calculate_density 52% = barrier.sample 无 Cache2D 缓存」的结论，并据此定优化方向「aquifer 的 barrier.sample 跨点加 Cache2D 缓存」。
- 精确无污染计数（aquifer_barrier_probe）：barrier.sample 调用 346 次 / 4 chunks / 393216 点 = 0.1%（每 chunk 仅 ~86 次）——barrier 采样几乎不发生。
- 无污染精确构成：get_fluid_level 3.84ms（22%）、get_block_pos 2.57ms（14%）、calculate_density fluid 逻辑 ~0ms、get_water_level_at 小——合计可解释 ~6.4ms，aquifer 总 17.5ms 剩余 ~11ms 未解释 = apply 每点 98304 次调用的固定开销。

### 根因（机制）
- 早前「calculate_density 52%」是污染态读数：把整个 calculate_density 记作「barrier 慢」，未区分 barrier.sample 与 fluid/提前返回逻辑。
- calculate_density 大多走提前返回（lava_water / j==0 → 0.0），barrier 采样（3D Noise 树遍历）几乎不触发——0.1%。给根本采不到几次的 barrier 加缓存，是优化了错误的目标。
- aquifer 真实大头是每点 98304 次的 apply 固定调用开销（函数调用 + 3×3 距离计算 + 分支 + 数组访问），而非 barrier 采样。

### 定位（诊断方法）
- 计数类硬证据：aquifer_barrier_probe 精确统计 barrier.sample 调用次数（346/393216 = 0.1%），用「调用次数」而非「某函数耗时占比」判断热点——走提前返回的路径耗时占比被稀释/误导，次数才是直接证据。
- 无污染 diag 定位（非热路径 instrument）：规避 P2「诊断代码热路径每点执行」污染，得到可解释 ~6.4ms + 剩余 ~11ms 的精确构成。

### 修复
- 撤销「barrier 加 Cache2D 缓存」方向（barrier 采样本就 0.1%，缓存无意义）。
- aquifer 优化方向修正为：fill_chunk 宏观采样对齐 Java Interpolated 网格架构（降采样次数，~1225 网格交点 + 三线性插值 vs Rust 逐点 98304）——根本解决 apply 每点 98304 次的固定开销，而非在 barrier 上加缓存。需正确实现避免跨 chunk 雪崩重建（P1 教训）。
- 早前污染态 profile（calculate_density 52%）在 07 篇作为历史读数保留标注，被本次精确数据取代。

### 教训（可复用判错经验）
- 定位瓶颈要用「计数类硬证据」（采样次数 / 调用次数），不要凭「barrier 是 density 树」推断它是热点——走提前返回的路径耗时占比高但实际调用次数极少，加缓存优化的是错误目标。
- 优化方向先量化再动手：动手加缓存前先数清目标函数实际被调用几次（346 次/39 万点 = 0.1%），量化能直接否定「加缓存」这类方向。
- 注意污染态读数：精确拆分要看内部计数，不能把父函数耗时全部归因于子调用。

---

## P5. 「Java FULL 8-9ms」小样本缓存假象 → 误判「Rust 慢 5 倍」（重大反转：Rust 反快 ~1.2 倍）

### 现象
- 早期端到端公平对比（P3 时代）用 `WorldGenBench FULL` 充分预热后测出「Java 原版稳定 8-9ms/chunk」（排除冷启动），据此定判 **Rust 44.9ms 慢 Java ~5 倍**（写入 07 篇 + 10 时间线 + 铁律）。
- **大样本修正**（2026-08-29，region 200,200）推翻该数字：
  - **Java FULL（256 chunks）≈ 55ms/chunk**（稳定 54-57ms，avg 51.7 含冷启动，min=2ms=缓存，max=1006ms=冷启动）。
  - **Java 宏观 NOISE（256 chunks）≈ 23-25ms/chunk**（avg 25.4，稳定 20-27ms，min=0=缓存）。
  - **Rust 宏观（400 chunks）34.66ms/chunk**（density+aquifer，aquifer 增量 ~21.5ms）。
  - **Rust 全管线（400 chunks 无树花）45.48ms/chunk**。
- **修正结论**：**Rust 全管线 45.48ms < Java FULL 55ms → Rust 反而快 ~1.2 倍**（「慢 5 倍」不成立）；但**宏观专项 Rust 34.66 > Java 23-25 → aquifer 慢 ~1.4-1.5 倍**（真实差距需优化）。

### 根因（机制）
- **「8-9ms」是小样本 + 相邻 chunk 缓存假象**：先前用 16 chunks 小样本、**顺序生成相邻 chunk**——MC chunk 缓存 / blending 让后续相邻 chunk 复用高度/结构缓存，后续 chunk **假快**；`getChunk(FULL)` 的缓存共享使小样本中位数被严重低估。
- **样本量决定「测到缓存命中还是真实生成成本」**：小样本顺序相邻 chunk 命中缓存 → 8-9ms（仅缓存残差）；大样本 256 chunks region 200,200 → 相邻 chunk 缓存假象被稀释，暴露真实 55ms。
- 与 **P3（JIT 未热）同族**：P3 是「JVM 冷启动未排除」，P5 是「样本过小 + 相邻缓存未排除」——**两次都是基准方法论错误**，且 P5 的误差（8-9 vs 55ms，**~6 倍低估**）比 P3（60 vs 8-9ms 高估）方向相反、量级相当。

### 定位（诊断方法）
- **大样本对照**：把 region 从 16 chunks 拉到 **256 chunks（Java）/ 400 chunks（Rust）**，同一 region 200,200、单线程、同测量口径。
- **`java_full_correction.txt` / `macro_java_vs_rust.txt` / `fair_comparison_corrected.txt`**（cmd-output）三份数据交叉确认：Java FULL 稳定 54-57ms、avg 51.7；宏观 NOISE avg 25.4；Rust 宏观 34.66、全管线 45.48——数字自洽（全管线 > 宏观，Java FULL > 宏观）。
- 怀疑触发点：`macro_java_vs_rust.txt` 明确点出「**矛盾：Java 宏观(NOISE) ~23-25ms > Java FULL(之前测 8-9ms)？**」——宏观单测竟然比完整路径假快，本身就是缓存假象的信号，逼出对 FULL 8-9ms 的复核。

### 修复
- **撤销「Rust 慢 5 倍」结论** —— 错误基准。**正确结论**：Rust 全管线 45.48ms < Java FULL 55ms（**Rust 反而快 ~1.2 倍**，尽管 Rust 无树花做更少工作）；宏观专项 Rust 34.66 > Java 23-25（**aquifer 慢 ~1.4-1.5 倍，真实差距需优化**）。
- 07 篇端到端小节、10 时间线「端到端基准修正」条目需**按本修正更新**（由主会话应用修正稿）。
- **基准方法论写入铁律**：基准必须 **大样本 + 排除缓存/冷启动** chunk。

### 教训（可复用判错经验）
- **基准必须大样本 + 排除缓存/冷启动**——小样本 + 顺序相邻 chunk 会严重低估 Java 真实生成成本（本案例 **~6 倍误差**：8-9 vs 55ms）。看「某侧快 N 倍」先确认样本是否小到被相邻 chunk 缓存污染。
- **小样本顺序生成 = 测量缓存，不是测量生成成本**：相邻 chunk 复用高度/结构缓存让后续块假快。拉大样本（≥ 一个 region 的量级）才能把缓存假象稀释到底。
- **基准不可靠连续两次**（P3 JIT 未热 + P5 小样本缓存）→ **基准方法论必须先锚定**，否则「某侧慢/快 N 倍」这类结论随时可能被大样本反转。
- **「宏观 > 完整路径？」本身就是基准可疑信号**：完整路径（FULL）应 ≥ 子集（宏观）；若子集反而更快，立即怀疑基准（缓存/冷启动），不是急着解释算法。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| 「Interpolated 慢 100× → 放弃」被 judge 推翻（P1） | **双层 Interpolated 污染**：final_density 树内部已含 interpolated，外层再包一层 → build_grid 网格点跨 chunk 反复清空内层网格 → **291× 雪崩重建**（6029 → 175 万次/chunk）；单层 Interpolated 包纯 SplineDF 实际**加速 70×** | **测量基准先确认无污染再下优化方向结论**；数值可复现 ≠ 基准正确；**多层网格/树包装先查缓存雪崩**（外层点跨 chunk 击穿内层网格） |
| 诊断代码热路径每点执行 → 27% 退化（61.5→44.9ms）（P2） | env 查询/`AtomicBool::load`/`Instant::now` 即使门控关闭，自身每点执行（98304×/chunk）仍放大到 ~157 万次/批量 | **诊断门控必须 chunk 级判断一次或编译期 feature gate**；诊断代码绝不放热路径每点执行；与「测量/探针污染铁律」同族 |
| 「Java 60ms」错误基准 → 误判 Rust 达标（P3） | Java bench **JIT 未预热**，冷启动开销拉高中位数；「积累性差异」被逐段 vs 上一段的相对优化掩盖 | **端到端对比必须用充分预热的 Java 基准**（稳定中位数 + 排除首个冷启动 chunk）；**禁止逐段 vs 上一段**，锚点必须是最终标准 |
| 「barrier 加 Cache2D」被实测推翻（P4） | 早前 calculate_density 52% 是污染态读数（把 barrier.sample + fluid/提前返回混记）；实际 barrier.sample 仅 346/393216 = 0.1%（走提前返回几乎不触发）；aquifer 真实大头 = apply 每点 98304 次固定调用开销 ~11ms + get_fluid_level/get_block_pos ~36% | 定位用计数类硬证据（采样/调用次数），别凭「barrier 是 density 树」推断热点；先量化再动手优化（0.1% 采样率直接否定「加缓存」方向）；注意污染态 profile 把父函数耗时误归因于子调用 |
| 「Java FULL 8-9ms → Rust 慢 5 倍」大样本反转（P5） | 8-9ms 是**小样本（16 chunks）+ 相邻 chunk 缓存假象**（顺序生成相邻 chunk 复用高度/结构缓存 → 后续块假快，getChunk(FULL) 缓存共享）；大样本 Java FULL 256 chunks ≈ **55ms/chunk**（真实 8-9 vs 55ms ≈ **6 倍低估**）；与 P3（JIT 未热）同族——基准不可靠连续两次 | **基准必须大样本 + 排除缓存/冷启动**；小样本顺序生成 = 测量缓存而非生成成本；**「宏观子集 > 完整路径」本身就是基准可疑信号**（FULL 应 ≥ 子集）；修正后 Rust 全管线 45.48 < Java FULL 55（Rust 反快 ~1.2×），宏观 aquifer 34.66 > Java 23-25（慢 ~1.4× 真差距） |
