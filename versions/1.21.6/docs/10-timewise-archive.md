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
- 状态：结论 candidate 待用户 confirmed。
