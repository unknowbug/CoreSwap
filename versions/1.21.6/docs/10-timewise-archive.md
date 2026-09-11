# 1.21.6 时间线（格式同 versions/1.20.1/docs/10-timewise-archive.md）

## 260909-02（实际 2026-09-09 Get-Date 锚定：mc-1216 features 接管 Phase 2.5——双臂对拍 → batchD-E1 发现修复 → 噪声基线 → 共现相关 → fan-out → 序列探针根因定位 → mask 不翻转建议）draft（verdict judge 未做，confirmed 未授）

> 过程产物 `.artifacts/mc-1216-features-takeover/phase25-verdict-260909-02.md`（verdict 主文档）+ `.investigations/mc-1216-features-takeover/fanout-260909-02/{b1-rng-wiring,b2-feature-algo}.md`（fan-out 双候选）+ batchD-record-260909-01.md v2 节；数据 `.tmp/mc1216-closeout-260909-02/`（不入库）；mixin 探针 runtime/1.21.6/java（不入库，batchD-record 为追踪载体）；通用模式 → workflow-patterns #91/#92 + build-tooling #43（subagent 草稿 → 主会话应用）。

- ✅ **双臂对拍 v1 → batchD-E1 发现（P1）**：v1 HEAD cancel 连坐 generateFeatures 内结构方块放置段（ChunkGenerator.java:360-379 与 feature 迭代同方法）→ Rust 臂丢全部结构件（vault/trial_spawner/waxed copper/rails/chest/spawner/cobweb 大量差异）。v2 = HEAD 仅门控 + @Redirect placedFeature.generate（:406）精准让位，结构族差异消失。注入点第三查沉淀 → workflow-patterns #91。
- ✅ **噪声基线（#67/#51 家族量化）**：Java-vs-Java 同代码双 run terrain=101,529 / veg=35,967 → run 级非确定只解释 ~2% 残差，信噪比 ≈48×。
- ✅ **共现相关分析 → b3 候选消解**：stone 族 3.98M 中 89.5% 与 ore 指示共现、75.5% 与 disk/lake 共现、独立仅 3.4% → 级联非独立层分歧（方法论 → #92）。
- ✅ **fan-out 双候选（fanout-260909-02）**：b1 RNG 接线——公式/类型/迭代序三层静态同构（population seed 有 Java 数值锚），输入域层保留 registry_order 回退残差（部分排除，Degraded）；b2 特征算法——4 族静态 PROVEN（Disk break 过早 / Geode isAir 缺 cave_air / Ore 邻 chunk 暴露判定 / Lake isSolid 含树叶）+ emerald_ore catch-all（0 RNG 消费）。
- ✅ **P-b1 序列探针 → 根因定位（decisive）**：chunk(0,0) 双侧 (k,p,fid) 对拍，step 1-8 共 40 条完全一致；step 9 特征集相同（13 fid 一致）但 p 全体错位（trees_water J=50/R=28 等）→ decorator seed=f(l,p,k) 域错位 → 植被整体偏移+级联。根因 = Rust PlacedFeatureIndexer 植被段 lastIndex 指派与 Java 不一致（runtime 直接证据：probe-rustfeat-err-snapshot.log:13/65/78 biome_registry_order.json 缺失 → 字典序回退实录生效）。探针做法 → build-tooling #43。
- ⚠️ **次级实现缺陷 5 项（b2，静态 PROVEN，位置修复后逐一可验）**：Disk break（feature.rs:467-473）/ Geode isAir（feature.rs:2006）/ Ore isExposedToAir（feature.rs:358-369）/ Lake isSolid（tree.rs:969-981）/ emerald_ore catch-all——详见 fanout b2 §一（emerald「下游错位」论断已按 judge C2 勘误：无下游错位，仅直接损失）。
- ✅ **mask 翻转建议 = 不翻转（0b011 维持）**：step 9 p 错位未修复前翻转 = 全域植被换位 + 4 个已知 feature 族缺陷上线。翻转前置 = indexer p 域对齐 + b2 五缺陷修复 + 重对拍信噪比回噪声基线量级。
- 🔍 **open**：indexer p 域根因修复（生成 1.21.6 biome_registry_order.json——1.20.1 文件缺 pale_garden 等新 biome，覆盖面逐项核对 / feature list 构建序）；b2 五缺陷逐一修复+探针验证；b2 探针 P1-P6 清单；残差 9 项 §9.7 单列（dripstone_block=237 + pointed_dripstone=34 本 region 实测；iceberg/fossil/sculk/large_dripstone 未出现）；confirmed 留用户。
- 口径声明（§9.7）：双臂/基线 = Chunky region 存档口径（全量三分分类）；序列探针 = 行为化日志（#81，单 chunk 覆盖面声明：step 1-8 对全 region 有代表性，step 9 在 chunk(0,0) biome 邻域成立）；b1/b2 静态结论 = Degraded。

## 260909-03（实际 2026-09-09 15:00-16:45）：mc-1216 features 接管 T0-T3——registry_order 根因修复 → b2 四缺陷修复+emerald 证伪 → 区域重对拍+归因 → 矿物/树/装饰层定位确认、mask 翻转撤销、CA_MIN 翻默认

> 过程产物 `.investigations/mc-1216-features-takeover/{rootfix,b2-fix,t3-rerun}-record-260909-03.md`；数据 `.tmp/mc1216-closeout-260909-03/`（不入库）；通用模式 → build-tooling #96 + workflow-patterns #97/#98/#99（subagent 草稿 → 主会话应用）。

- ✅ **T0 廉价独立验证通过**：probe-rustfeat-err-snapshot.log 三维度 registry_order 缺失实录 + biome.rs 回退/排序消费点在位 → 根因方向（字典序回退 → PlacedFeatureIndexer p 域错位）继承。
- ✅ **T1 根因修复（registry_order）**：复用 BiomeSourceLogMixin（`-PseedLog=1`）存档口径导出 **54 项 1.21.6 overworld 枚举序** → `versions/1.21.6/data/worldgen/biome_registry_order.json`（表外 11 = nether 5 + end 5 + the_void，与 1.20.1 同构；cherry_grove 等新 biome 已在序内）。Rust 零代码改动。短跑探针：missing=0，chunk(0,0) step9 13 项 (p,fid) 与 Java 逐项全同（修复前 p 全体错位 {0,28,53,...}→{0,50,57,...}）。
- ✅ **T2 四缺陷修复**（feature.rs/tree.rs，workspace 全量绿）：① Disk 列内 break 过早（对齐 half_height 层语义）② Geode isAir 扩全族（+cave_air/void_air）③ Ore isExposedToAir 走 block_at 全路由 ④ Lake isSolid 新 is_solid_lake（其他调用点不扩散）。⑤ emerald_ore catch-all **证伪**（§15.4 勘误，→ #99）。step9 回归哨兵：p 域未扰动。
- ✅ **T3 执行体三元组核验先行**（→ #96）：dev run 实际加载 build/resources/main/native/worldgen.dll sha16 02d897a3；build/libs jar 13:26 时间戳是红鲱鱼；脚本哨兵 grep v1 旧措辞 NONE-SEEN 假阴性（→ #97）。
- ⚠️ **T3 验收判据未达成 + 归因**：T1+T2 后 terrain 4.69M（噪声基线 0.10M）几乎不降；开 WG_CA_MIN=1 → 3.65M（−22%）。三层归因：① 序列全同 ≠ 区域收敛（→ #98）② WG_CA_MIN 生产 gap：邻 chunk 读修复整体 no-op ③ tree placer 缓装缺口（→ 台账 B13）：本 region spruce_leaves 69,256 等逐项 bit 级 deterministic。
- ❌→撤销 **mask 翻转（T4）**：**用户拍板（260909-03）确认项目定位——矿物/树/装饰层不接管，mod 兼容留 Java 侧、性能收益小**；0b011 维持，T1/T2 修复只服务于接管实验臂语义。D1（实装 tree placer）不立项。
- ✅ **WG_CA_MIN 翻默认开（用户拍板，语义优先）**：worldgen_handle.rs:971 默认 on（WG_CA_MIN=0 显式关）；+68% 成本另立性能题。
- 🔍 **open**：无（本块闭合）。接管实验臂若再启用：B13 tree placer 族 + kelp/seagrass 边界带为已知残差。
- 口径声明（§9.7）：Chunky #26 seed -8248318472910187742 region 0,0 r=16（1089 chunks/臂，3410 common）；同载体同 region 与 260909-02 基线直接可比，diff 脚本同源；序列探针=单 chunk 覆盖面；nether/end registry 序未对齐沿用 260905-08 近似声明。

## 260909-04（实际 2026-09-09 17:32 起）：WG_CA_MIN 性能优化——C1「不回填」被 E1 零改善证伪 → 真根因「主管线不读缓存」E1b 回收 ~72% → 残差 +35% 定性固有成本

> 过程产物 `.investigations/camin-perf/{scout-map,perf-record}-260909-04.md` + 架构计划 `.investigations/000-架构设计/架构计划-260909-04-camin-perf.md`；载体 `worldgen-core/src/bin-diag/camin_bench.rs`（新增，#13/#30 纪律 rustc 单编）；通用模式 → workflow-patterns #100/#101（subagent 草稿 → 主会话应用）。

- ✅ **基线复现**：camin_bench 16×16=256 chunks 串行（seed -8248318472910187742），on 289.1ms/chunk vs off 128.5（**+125%**；dump 口径为 +68%，区域/版本上下文不同——§9.7：bench 口径 ≠ 存档 dump 口径，不可比）。行为 hash 哨兵：on 6908dbfc… / off 115641b8…。
- ❌→证伪 **E1（scout C1 原模型：主管线不回填 terrain_cache）**：回填后 297.4ms **零改善**——C1-as-modeled 被证伪。行号证据真实但不是成本来源。
- ✅ **E1b 真根因实锤：主管线不读缓存（地形双算）**：管线改缓存优先命中 + 未命中回填（terrain_cache 值升级 CaTerrainEntry{col,heightmap} Arc 对）→ on 289→**177.9ms（+125%→+35%）**，off 无回归，hash 三轮逐位不变。回收 ~72%。
- ✅ **E2b 排除 C2 雪崩**：WG_CA_CAP 256 vs 2048 → 178.6 vs 176.5ms（差 <2%）。
- ✅ **残差归因（E4b/E4c，判别式 → workflow #101）**：WG_SKIP_FEATURES 分解 → 残差全在 features 段（58.2 vs 13.8ms，4.2×）；all_reads on **853k/chunk** vs off 7.5k（**114×**），≈**51ns/读**；per-feature seagrass ≈53% + ore×4/disk/monster_room——ca_min 语义正确的固有成本（真实邻值使 feature 不再 -1 早退）。memo 化下扫需写失效处理，语义漂移风险高，不建议。
- 📌 **改动清单**：worldgen_handle.rs（CaTerrainEntry / neighbor_terrain 返回 entry.col / fill_chunk_blocks 缓存优先命中+回填 / WG_CA_LOG all_reads+per-feature 扩展 / WG_CA_CAP 判别 env，ca_cap() 单一定义）；blocks.rs BlockColumn derive Clone。git 基线 2bea71c。
- ✅ **验证**：workspace 全量 build 绿（worldgen1216 薄壳含）；cargo test -p WorldgenRust --release 14/14；行为等价门 PASS（hash 三轮逐位）；judge review PASS-with-conditions 三条件已应用（CAP 统一 / .artifacts 登记 / 表述修正）。
- ⚠️ **降级声明（§9.7）**：本块读数 = 本地 bench 载体（串行 region、bench 口径），与 260909-03 的 223.7s/133.3s 存档 dump 口径不可比（载体/覆盖面/口径三要素均不同）；vs Java e2e 本轮未跑（Java 基线不变式由 260909-03 既有 evidence 覆盖；「Java 同付这部分工作」为推断非实测）。
- 🔍 **IDK**：IDK-p1 残差 +35% 若需再回收 → 写失效感知 memo（中高风险，需独立架构评审）；IDK-p2 bench +125% vs dump +68% 口径差未深究（bench 含 region 边缘外邻重算）。
- 状态：✅ 用户 confirmed（260909-09）。

## 260909-06（实际 2026-09-09 21:51 起）：ca_min B3 探针轮 + B3a 快照实现——预置三分支均不匹配实际 → (a) clear-all 纯增 (b) 尾缘语义 (c) per-read 同步三路混合 → CAP 默认翻 2048 + B3a 快照落地

> 上接 260909-04/05 块（05 块：memo-review 三证廉价验证 + 写侧判别实验「51ns 系摊销值」判定 + fan-out B1/B2/B3；judge 条件①E2b 冲突张力显式化 → 本块履行）。过程产物 `.investigations/camin-perf/{memo-review-260909-05,probe-260909-06}.md` + 架构计划 `架构计划-260909-06-camin-b3-probe-impl.md`（已批准）；通用模式 → workflow-patterns #102/#103/#104 + #101 §15.4 部分取代 + build-tooling #45（subagent 草稿 → 主会话应用）。

- ✅ **Step 0 交接廉价验证**：fc9c4fe MEMODIAG 代码在位；本块双臂行为 hash 与上轮哨兵逐位一致（on 6908dbfc / off 115641b8）→ 260909-05 结论继承合法。
- ✅ **探针扩展（本块新码，WG_CA_MEMODIAG 门控）**：① [CA-MEMO-OFF] 越界读 chunk 偏移直方图；② [CA-NT] neighbor_terrain calls/misses/fill 计时；③ WG_CA_CAP 臂复用（无新码）。诊断开关行为 hash 恒等验证 ✅。
- ❌→证伪 **预置三分支判据均不匹配实际**（判据核对节）：① 偏移直方图全落 3×3 内（B3a 覆盖率假设成立）✅；② miss 实测 0-8/chunk（全 region 358/322），**miss<<288** → 「P2 = 288 唯一列 clear-all 雪崩」形态证伪——真机制是 clear-all 后对**已生成过**邻列的纯增重生成（work 移位后净增量），且打在主管线缓存读路径；③ P1 单独主导亦不成立 → 实际 = (a) clear-all ~16ms + (b) region 尾缘 ~17.6ms（小 region 虚高）+ (c) per-read 同步 ≤13ms/chunk 三路混合。
- ❌→修正 **E2b 结论被取代**（§15.4，260909-04 块第 4 行原读数保留不改）：CAP 256 vs 2048 三轮配对交错 ~10-11% wall（165.9/172.4/162.5 vs 153.7/159.2/147.1 等，方向一致）——260909-04 的 <2% 系单轮非配对口径 + 机器噪声带 ±10% 淹没信号，「C2 排除」撤回。
- ✅ **fill 恒等式自检**：fill 单价 112ms ≈ off 臂整 chunk 118.9ms 自洽；cap256−cap2048=3.5s ≈ 34 interior miss 差 × 112ms≈3.8s 自洽——clear-all 差异落在 miss 数差上，分解闭合。
- ❌→意外结果 **B3a（features 3×3 Rc 快照）单独仅 ~2%**（pre vs B3a@256 交错 ×3：180.5/164.1/162.6 vs 178.9/161.5/157.3，三轮全赢但小）——「消 per-read 同步 = 大头」预期落空。机制修正：clear-all 惩罚打在**主管线**缓存优先读路径（#100 读路径被 clear 波及），features 侧快照不护主管线 → B3a 与 CAP 是互补杠杆非二选一；B3a@256 vs B3a@2048 交错 ×3 仍差 ~11% 实锤（workflow #102）。
- ✅ **32×32 摊薄验证**：B3a@2048 on=178.5 vs off=163.3 → 残差 15.2ms/chunk（9.3%）vs 16×16 的 ~30ms（26%）→ 尾缘项随 region 规模摊薄成立（workflow #104；§9.7：本载体 16/32 边长，连续生成按边际继续摊薄）。
- 🔍 **机器噪声带发现过程（workflow #103）**：off 臂同执行体跨批次 118.9→145→127.6ms（hash 同 115641b8，排除行为面）——漂移 ±10% 与被测效应同阶；判据沉淀 = 新旧 binary 同批配对交错唯一有效 A/B 口径，跨批绝对值结论禁引（CAP65536 round2 178.3ms 单点离群未复跑，#28 标注）。
- ❌→作废重算 **PowerShell `-like '*[CA-NT]*'` 字符类坑（build-tooling #45）**：`[...]` 被当字符集（匹配 C/A/-/N/T 任一字符），诊断日志混入 [CA-MEMO-TOP] 行污染求和，首版汇总作废；修复 = `.Contains('[CA-NT]')`。判据：方括号标签日志过滤后必须打印样本行核纯度。
- ✅ **B3a 实现与硬门**：ca_snapshot Rc<HashMap> 3×3 预取（apply_features 开头，ca_min 门控，未命中兜底 neighbor_terrain，risk-3 不变量注释钉入）；hash 硬门 ✅（on=6908dbfc / off=115641b8 逐位不变）；全量绿 + 14 tests ✅。
- 📌 **最终配置定案（用户拍板 260909-06）**：CAP 默认 256→2048（800MB 不构成约束；65536 臂证更大无增益，1024 中途选项被覆盖）。最终 sanity：16×16 on=159.3/162.2/157.4（hash ✅）/ off=127.6（✅）；32×32 on=179.9（hash 93dc1dce，与先前 178.5 复现一致）。
- ⚠️ **降级声明（§9.7）**：本块全部读数 = bin-diag camin_bench 载体（串行 region 16/32 边长、bench 口径），与存档 dump 口径不可比；CAP 内存账：BlockColumn ≈384KB/条 → 2048 ≈800MB。
- ⚠️ **±5% 目标诚实声明**：B3a+CAP2048 组合后残差仍 ~15ms/chunk（32×32 口径），「on≈off±5%」未达成——剩余为尾缘语义固有 + 主管线语义成本，bench 内不可再消（in vivo 连续生成按边际摊薄）。
- 🔍 **IDK**：IDK-b1 (c) per-read 同步成本 ~13ms/chunk 为 1.68M reads × ~2μs 粗口径上界，未独立实测；IDK-b2 pending_writes=0 为单 region 单样本，外推其他区域/维度需复核；IDK-b3 CAP 2048 在多世界/大 region 场景的内存上限行为未测。
- 状态：✅ 实现落盘 + hash/wall 硬门已验；知识库批次已应用（commit f66ade3）；✅ 用户 confirmed（260910-01，实际 2026-09-10 12:58）。git 基线 e91c7f6（代码）/ f66ade3（docs）。

## 260910-05（实际 2026-09-10 18:4x-19:2x，日期锚 Get-Date 18:45）：C7 收口（跨 chunk 写序竞争假说 = 空集）+ 同配置 run-to-run 基线 + nether/end 异步化推广

> 过程产物 `.investigations/perf-closeout-260910-05/{record.md, scout-vanilla-crosswrite.md, patch-nether-end-async.md}` + `.artifacts/perf-closeout-260910-05/{verdict-c7, verdict-nether-end, judge-verdict-c7}`；数据 `.tmp/perf-reg-260910-05/`（不入库）+ `cmd-output/`（region 5+4 组、diff 12 份、MANIFEST-sha256 251 条）；Java 快照 `java-snapshot/pre|post`；通用模式 → workflow-patterns #110/#111/#112 + #107 家族补充案例 + algorithm-fingerprints #21 + build-tooling #49/#50 + #26 家族补充案例 + 错误台账 `perf-closeout-errors.md` E1-E4（subagent 草稿 → 主会话应用）。
> 上游：260910-04 confirmed（机制 = 单车道同步接管；R3 = 异步化 42s vs 279s）；judge C7（`perf-regression-260910-04/judge-verdict-r3-260910-04.md:117-118`）为本块任务来源。

- ✅ **开工前核验（交接结论廉价独立验证）**：R3 落地直证（`NoiseChunkGeneratorMixin.java:61/119-149`、`ChunkTiming.java:28-39 inflight`）；驱动脚本含 r3/r3sync 臂；**执行体三元组** sha 前 16 位 `abd7d8893d22e030` 与归档日志一致；nether/end 句柄在 1.21.6 可用（`initNether/initEnd enabled=true stageMask=3`）。
- 🔴→✅ **前提核验改变做法（本块最关键的一次转向）**：judge 要求的「跨 chunk 写序会计」其**前提为假**——静态链证明默认 `resolveStageMask() = 0b011` ⇒ `skip_features=true` ⇒ Rust `apply_features` 不执行 ⇒ `pending_cross_writes` **结构不可达**（生产 `:1088`/消费 `:1053` 同在 `:1036-1375`，唯一调用点 `:671`；`neighbor_terrain` 调用点 `:1124/:1242` 同族不可达）⇒ 会计降级为条件项，改成**行为化核验**（避免造不可执行路径）。
- ✅ **行为化核验（正负对照 + 通道证明齐备）**：负对照 `r3log-r2`（mask=3 + `WG_CA_LOG=1`）`[CA]` 行 = **0** 且跑满 4225 chunks（39s）；正对照 `r3feat-r2`（mask=5 + `WG_CA_LOG=1`）`[CA]` 行终态 = **1711**、`[WG-CONF] flags=5 skip_features=false`；双通道直证 = JVM banner `Picked up JAVA_TOOL_OPTIONS: … -Dcoreswap.rust.stages=5`（送达）+ `stageMask=5` 系 `CppWorldgen.getFlags(handle)` 的 **JNI 回读**。⇒ C7-①「跨 chunk 写序 ⇒ 写丢失」竞争假说 = **空集**，无复活出口（judge 独立复核：`beardifiers` 全局清空出口 `clearBeardifier` 在 Java 全树 grep **0 命中**；`terrain_cache`/`est_l2` 纯 memo 无别名写 ⇒ 出货语义下 Rust 侧 = per-chunk 纯函数 + 纯 memo，**比原结论更强**）。
- ⚠️ **正对照臂为主动终止（判据不受影响）**：末次 Chunky `Processed: 865 chunks (20.47%)` @18:57:08、末条 `[CHUNKTIME] n=1536` @18:57:12、`CALines` 终值 1711 ⇒ 完成度 ~20%（口径：CHUNKTIME `n` 有 ~+9% 超计、`[CA]` 行与 chunk 非严格 1:1）。门是 chunk 无关的常量 flags 判断，故终止不削弱判据（负对照臂跑满得零行）。
- ✅ **C7-② 同配置 run-to-run 基线（本轮新采）**：sync `0.0086%`（375,533,568 / 32,169）、async `0.0092%`（375,541,760 / 34,459）；跨形态 `0.0122%`/`0.0130%`（高于同形态基线约 0.004pp ⇒ **未归因**形态相关分量，假说：填充序影响 vanilla 结构/装饰放置）；跨实现 `0.0111-0.0148%`；历史代理基线 `0.0150%`。⇒ 等价门的锚由「跨实现代理基线」升级为「同配置实测基线」，门性质仍为**同量级筛选门**。
- ❌→更正（judge C3）**草稿方向读数写反**：把 async（0.0092%）> sync（0.0086%）写成「async 未比 sync 更大」⇒ 改为「async 略高 **+7%**，方向与 #67 预期一致；每形态 1 对读数 ⇒ 不足以判定显著（既未证实也未否证）」。教训：「不显著」≠「方向相反」（→ 台账 E3）。
- ❌→更正（judge C1/C2）**汇总表两处 `blocks` 与一手 diff 矛盾**（③行 `375,562,240` 全树 0 命中；⑦行与⑤行撞值）+ 终止点数字「768/4225」核不到 ⇒ 改为一手值（`375,545,856` / `375,549,952`）与可核锚点区间。**与 260910-04 C1 同族二犯**（→ 台账 E4）。
- ✅ **C7-③ 幅度范围**：async `42s/41s`（诊断口径另有 39s）vs sync `279s/243s` ⇒ **5.8×-7.2×**（非诊断臂口径 5.8×-6.8×）；相对 vanilla 快 **1.24×-1.33×**；`inflight max` **23（async ×3）vs 1（sync ×2）**；CPU 本轮缺失（根因按**未核**；仅 `ea-r2`/`eaS-r1` 两臂采到 ⇒ 引 260910-04 归档 543/375 CPU-s 作证据）。
- ✅ **§15.4 取代记录（不删改 confirmed 正文）**：supersedes `260910-04/verdict-260910-04.md:106`「代理基线级筛选结论：R3 未引入超出既有 run 级非确定的内容差」——新锚点下该**因果指认**不成立（跨形态 0.0111-0.0130% 高于同配置基线 0.0086-0.0092% 约 0.004pp），替代陈述 = 「差异高于同配置基线但 ≤ 历史代理基线」；**预登记触发器读法已显式声明**（数量级读法 ⇒ 触发器成立 ⇒ 记取代；严格数值读法 ⇒ 不成立但取代理由仍由数值直接支撑）——两种读法结论差 = 「是否显式记取代」（→ workflow-patterns #112）。**R3 性能/机制主结论不需重审**。
- ✅ **P4 代码改动（Java 侧纯调度形态，Rust 零改动）**：nether/end 两分支改为与 overworld R3 同构（`Supplier<Chunk> work` + print+rethrow + `ChunkTiming` 包裹 + 共用 `SYNCFILL` + `supplyAsync(work, WG_FILL_POOL)`）；`CppBridge` 新增 `MIXLOG` 门控 nether/end 每 chunk `[WG-FILL]` 行（R1 同族拉平）；`gradle compileJava` **BUILD SUCCESSFUL**（23s）；`runtime/` 无 VCS ⇒ 以 `java-snapshot/pre|post`（mixin 69 行 + CppBridge 10 行差异，`ChunkTiming`/`build.gradle` UNCHANGED）承载 diff。
- ✅ **P4a 载具首用 sanity（新载体：Chunky 维度 + DIM region）**：`chunky world minecraft:the_nether` 被接受（`Processed: 4225 chunks (100.00%), Total time: 0:00:21`）；`[Mixin] populateNoise(nether) intercepted` = **4761** + `[WG-FILL]` = **4761**（依赖 `-Pmixlog=1`）；region 采集 `run\world\DIM-1\region`（end 为 `DIM1\region`）⇒ 维度名/路径/接管生效三风险排除后才开 A/B。
- ✅ **P4b A/B 矩阵（9 臂）**：nether vanilla 56s / sync 69s（inflight 1）/ async 21s、20s（inflight 23）；end vanilla 5s / sync 18s（1）/ async 6s、6s（23）；`ea-r2` 是本批两臂之一采到 CPU（**86 CPU·s / 6s**；补跑 `eaS-r1` = 99 CPU·s ⇒ 更多总 CPU 换更低延迟，属**延迟结论非效率结论**；nether 总 CPU 未测；CPU 缺失根因按**未核**声明）。⇒ **形态效应（口径 = Chunky 进程内任务计时）** nether **3.3-3.5×**、end **3.0×**；同口径下 wallgen 仅 **2.34×（70.1→30.0）/ 2.00×（20.0→10.0）**；相对 vanilla nether **反超 2.7-2.8×**、end async 6s ≈ 5s，而 **sync 形态 wallgen 净亏损**（70.1>60.0、20.0>10.0）⇒ 单车道形态在三维都是净亏损，异步化后转为不亏/反超。**禁用「端到端」措辞**（vs Java 生产口径未测）。
- ✅ **P4c 维度行为门**：nether 跨形态 `0.1219%` ≤ 同形态噪声 `0.1406%` ⇒ PASS（⚠️ 该噪声是 **async 代理**——nether 只有 1 个 sync run，无 sync run-to-run 对）；end 跨形态 **0 块差**（同形态噪声 0）⇒ PASS。**限定**：end「0 差」仅适用 **coreswap 臂内 + 该 region 集**（灵敏度论证：同链在 `es×ev` 检出 **7 块 / 494M** ⇒ 跨实现差在该维**可检出**，**不得**与零差并列；`ev` 仅 1 run ⇒ 7 块差无法归因「实现差 vs vanilla run 噪声」；**不得**外推「该维不存在 run 级非确定」）。附带读数：nether 接管基线 `0.1020%` **低于**自身（async 代理）噪声 ⇒ 差异落在噪声带内，**≠「接管已验证」**（1.21.6 nether/end 首次验证）。单对点估计、检验力低。
- 🔍 **open（登记后续项）**：① nether run 级非确定显著高于 overworld（0.1406% vs 0.0092%，~15×）**成因未查**；② 跨形态 vs 同形态基线的高出分量（~0.004pp）**未归因**（需冻结顺序生成载体，属新测量设计，后置，不阻塞 C7）；③ 未来重启 Rust features 接管（bit1 清零）MUST 先加「迟到写/已生效/终态常驻」三数且**按 chunk/线程分桶**（现成 `CA_PENDING_WRITES` 是全局 atomic、23 线程并发下每 chunk 重置 ⇒ 只作量级：实测区间 0-37,893、中位 1,575）+ 逐项裁定 scout §4 的 A1-A7/C1/C2（含越界高度图哨兵、Rust features + Java structures 混合语义）；④ ~~nether/end 的 J3/J4 judge 未做~~ → **已完成**：`judge-verdict-nether-end-260910-05.md` = PASS-with-conditions（C1-C10 已应用；含 eaS 补跑闭合 end sanity 缺口、CPU 根因按未核、比值口径点明）。
- ❌ **工具坑（→ 台账 E1/E2）**：`pwsh -File … -Arms a,b,c` 逗号串当单字符串（`unknown arm a,b,c`）；CPU 采样点疑似**重复覆盖** ⇒ 13 行归档表 **12 行** `serverCpu=-1` 恒成立（初诊「旧解析版本」被证伪）；**但真根因按未核声明**（FIN-C1）——两轮 judge 各读到一版脚本、归档副本均为修后版 ⇒ 文件级不可复核，行为侧旁证 = 修后采到 CPU 的**两臂**（`ea-r2` 86 / `eaS-r1` 99 CPU·s）。
- ⚠️ **工具契约（→ build-tooling #50）**：`scripts/merge_index.py` 写回根 index 是「解析 + 重建」（四字段白名单 + YAML 丢注释）⇒ **带注释的根索引禁跑该工具**，本块 P5 改手工追加 entry（现网 `.artifacts/index.yaml:1189` 已就地写下警告）。
- 口径声明（§9.7）：**载体** = 1.21.6 + Chunky radius 500 = 4225 chunks/臂；**覆盖面** = region 全域逐块普查（overworld `common=7959`、nether `7542/7543`、end `7567`，非抽样）；**可比性** = 同载具/同 seed（`417950215108767439`）/同 region 中心/同工具修订；分母 = `4096 × 两侧 section 并集`；chunk 集缺口 = 比对集 7959 vs 本臂生成 4225（含非本臂 chunk）；**维度间不可互引噪声基线**（nether 与 overworld 差 ~15×）。
- 状态：C7-①/②/③ + §15.4 处置 = **candidate**（judge PASS-with-conditions，C1-C9 已应用；§5 产物契约 C7 已补 = `index-entry.yaml` + 根索引手工登记）；nether/end = **candidate**（judge J3/J4 = PASS-with-conditions，C1-C10 已应用）；两者 confirmed 均留用户。git 基线 `42d46e7`（继承任务书）。

> **⛔ 260911-05 取代指针（§15.4，本块数值已被 E5 规范读法复算取代；原文不删不改）**：本块 `:75`（C7-② 同配置基线）、`:76`（judge C3 方向读数）、`:77`（judge C1/C2 汇总表 blocks）、`:79`（本块 §15.4 取代记录）、`:83`（P4c 维度行为门）、`:84`（open ①）、`:87`（§9.7 口径声明）的**全部百分比 / 分母 / chunk 数**系**旧读法**（对拍工具 `payload()` 的 `tag 7` 长度按 1 字节读，规范 = **TAG_Int 4B**）产物。同载体、同 region 目录、唯一改动 = 该字段 `u1→i4` 的**离线复算**（`.investigations/e5-recompute-260911-05/`，15 对含历史代理基线）结果（旧 → 新）：sync `0.0086%→0.0133%`、async `0.0092%→0.0169%`、跨形态 `0.0122%/0.0130%→0.0173%/0.0177%`、跨实现 `0.0111-0.0148%→0.0192-0.0198%`、历史代理基线 `0.0150%→0.0180%`；nether 同形态 `0.1406%→0.3759%`、跨形态 `0.1219%→0.2978%`、接管基线 `0.1020%→0.2703%`；**end 7/0/0 不变**（分母 `494,010,368→496,041,984`、`common 7567→7569`）；`common`（overworld）`7959→8004`、分母 `375,5xx,xxx→786,825,216`；**sections/chunk `11.52→24.0000`（overworld）/ `6.80→16.0000`（nether·end）**；维度倍数 **~15×→~22×**。
> ⇒ **`:79` 的替代陈述「差异高于同配置基线但 ≤ 历史代理基线 0.0150%」随之失效**（基线复算 = 0.0180%，R3 跨实现读数 0.0192-0.0198% **反超 0.0012-0.0018pp**）；`:83` **nether 门方向不变、仍 PASS**（0.2978% ≤ 0.3759%）、end 0 差不变；`:84` open ① 成因**域收窄**（不在 Rust 填充层；下游未测）；`:87` 的 §9.7 三要素**框架不变**，载体/覆盖面/可比性表述有效，仅其中数值随本指针换代。取代链：`record-260911-05.md` §3/§4/§5（S1-S5）。**性能与机制主结论（R3 异步化 42s vs 279s、inflight 23 vs 1、nether/end 推广）不受 E5 影响。**


> 260910-06 / 260910-07 两个工作块的**记录归口** = `versions/1.20.1/docs/10-timewise-archive.md`（260910-06 = 1.20.1 三分支异步化；260910-07 = Java 工程迁出 runtime/）。
> ⚠️ 但 260910-07 **同时改了 1.21.6 侧**：`runtime/1.21.6/java/` 的 loom dev 工程（49 源文件 + 构建定义）迁至 **`versions/1.21.6/java/`** 并入库（运行环境原地 `runtime/1.21.6/java/run`，由 `runDir` 指回），等价性门 = 迁移前后 jar **1797/1797 条目全等、整包 sha `16d5e5e7…` 不变**、三元组 MATCH。详见 `.artifacts/perf-reg-260910-07/verdict-260910-07.md`。
