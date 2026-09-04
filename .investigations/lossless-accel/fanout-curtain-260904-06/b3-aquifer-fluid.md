# B3 候选：aquifer fluid 判定差（d≤0 误回 solid / 流体位语义差）（260904-06，core.worker fan-out b3 臂）

> status: candidate-pending（分析候选，未运行任何探针——纯静态三方对拍；不下裁决，confirmed 留人类）
> 前置（已裁决，勿重查）：writer-verdict-260904-06.md / convergence-260904-06.md（幕帘复合结构：stone run + ~16 间距 aquifer 水口袋 @ 197,212,228,244,261,277）；ore_vein y≤50 上限两臂本地成因已排除（ore_vein.h:33 / ore_vein.rs:46-48 亲核）。
> 验证分层：**Degraded（静态审查）**——本产物全部结论为代码级推演，无 trace/probe 数据层证据；§5 探针执行前不得升级。
> §9.7 口径：载体 = C++ aquifer.h 全文 / Rust aquifer.rs 全文 / Java AquiferSampler.java（yarn sources extract）逐分支静态对拍 + 消费环（worldgen_api.cpp / worldgen_handle.rs / terrain.rs）；覆盖面 = apply→calculateDensity→getFluidLevel→getFluidBlockY 全调用树 + granite/copper 写者盘点；与既有口径可比性 = 复用 convergence 的 est 零语义差裁决（m2，域 (200,200) 未复验）与 oreVein 排除裁决，未引入新数值口径。

---

## 1. 结论速览（B3 主体）

1. **「d≤0 误回 solid」在三方实现中均无代码路径（结构性排除）**。`apply` 在 d≤0 时所有 return 只能是：流体块（lava）、bs（water/air）、或 margin 三连（density+e/g/h>0）返回 -1（= 交还调用方写 default block = stone）。-1 不是「误回」——它是 Java 同构的 barrier 边际机制（AquiferSampler.java:222-242 三次 `density + e > 0.0 → null`）。**solid 只能由 margin 三连或 d>0 产生，没有第三条路。**
2. **aquifer 本体逐分支对拍未发现任何 C++/Rust 共有转录偏离**（§2 全表，17 项全 ✓）。两侧 aquifer 是同一份 Java 翻译（aquifer.rs:2 自述 port of aquifer.h），共享错误的可能性集中在更上游的输入（est/密度值），不在 aquifer 分支逻辑内。
3. **新发现一处真实 C++↔Rust 分叉（B3 域内，方向与 B3 预设相反）**：aquifer margin 返回 -1（solid 语义）时，**C++ 写 stone**（worldgen_api.cpp:1040-1043，block<0 → oreVein → 仍 <0 → stone），**Rust 写 air**（terrain.rs:232 `match apply { 1=>Water, 2=>Lava, _=>Air }` 把 -1 归入 `_=>BlockKind::Air`，worldgen_handle.rs:541-546 Air→air id）。Java 语义 = null → default block（stone）。**即 Rust 臂丢失了全部 barrier-margin stone**——这不是共有偏离，而是 Rust 单臂 vanilla 偏离；它直接限定：**两臂共有的幕帘 stone 不可能来自 barrier margin（Rust 侧根本产不出），共有 stone 必须 = d>0（Rock 路径）→ 指向 B1**。
4. **水口袋位置反演（B3 对世界图景的贡献）**：幕帘域水口袋 @197-277 要求 `fl2.y > blockY`（FluidLevel.getBlockState：y < liquid level 才回 water，AquiferSampler.java:61-63 / aquifer.h:23-25 / aquifer.rs:70）。幕帘高度 default fluid level 恒 63-water（C++:80-81 / Rust:298 / Java NoiseChunkGenerator.java:78-84），要出 y≥197 的水口袋，唯一路径 = getFluidBlockY 的 noise-based fluid level = `min(est, q)`（C++:355-366 / Rust:462-472，q = cellY/40*40+20+spread，上界 ~278）——而这要求 13 邻域 est 的 min 值 **≳ 200**（否则 getFluidLevel 早退 defaultFL：`blockY-12 > est+8` 直接返回 63-water，C++:306 / Rust:429 / Java:366-368）。**即：幕帘列的 est 在两臂被抬到了 ~200+，与 stone 本体（d>0）同指向「高海拔密度抬升」一个上游成因（B1/B2 域），aquifer 只是如实转换器。**
5. 同理，幕帘 stone run 的下界约束：对 y > est+20 的区段，aquifer 对一切 d≤0 只能回 AIR（margin 必为 0：fl2/fl3/fl4 全 63-water → calculateDensity j==0 → return 0，C++:246 / Rust:385）；est≈200+ 时 y≈192-220 区段才可能由 margin 贡献 stone——而 Rust 侧该贡献为 0（见结论 3）。**预测：Rust 幕帘列 stone 集合严格等于 {d>0 集合}；C++ 幕帘列 stone ⊇ {d>0} 且多余部分只可能来自 margin（若 est 足够高）或 surface。**

## 2. 三方逐分支对拍表（全部 ✓ = 无偏离；引用 = 文件:行）

| # | 分支 | C++ | Rust | Java | 判定 |
|---|---|---|---|---|---|
| 1 | d>0 → -1/null | aquifer.h:74 | aquifer.rs:295 | AquiferSampler.java:149-151 | ✓ |
| 2 | default fluidLevelSampler（y<-54 lava@-54 / else water@63） | aquifer.h:76-82 | aquifer.rs:298-299 | NoiseChunkGenerator.java:78-84（y<min(-54,63)） | ✓（overworld 硬编码合法；非 overworld defaultFluid/seaLevel 数据驱动缺口——与本课题无关，仅登记） |
| 3 | getBlockState 语义（y≥level→air） | aquifer.h:23-25 | aquifer.rs:70 | AquiferSampler.java:61-63 | ✓ |
| 4 | 18 cell 邻域 + top-3（o≥ag 移位序） | aquifer.h:90-104 | aquifer.rs:306-316 | AquiferSampler.java:168-207 | ✓ |
| 5 | cell 随机偏移 split(x,y,z)→nextInt(10/9/10) + pack 26/12/26 位 | aquifer.h:220-230,198-217 | aquifer.rs:277-290,194-210 | AquiferSampler.java:180-183 + BlockPos | ✓ |
| 6 | d = maxDistance(o,p)，25.0 分母 | aquifer.h:107,232-235 | aquifer.rs:319,292 | AquiferSampler.java:210,258-261 | ✓ |
| 7 | d≤0 → 直接回 bs | aquifer.h:113 | aquifer.rs:321 | AquiferSampler.java:212-214 | ✓ |
| 8 | water-over-lava 特例（bs==water 且 (x,y-1,z) fluid==lava → 回 bs） | aquifer.h:114-117 | aquifer.rs:322 | AquiferSampler.java:215-217 | ✓ |
| 9 | margin 三连 e/g/h（d·calc、d·f·calc、d·g2·calc，f/g2>0 门） | aquifer.h:119-139 | aquifer.rs:324-340 | AquiferSampler.java:218-247 | ✓ |
| 10 | calculateDensity：lavaWater→2.0；j==0→0；q 分段 p/1.5、p/2.5、p/3、p/10；|q|≤2 采样 barrier；2·(r+q) | aquifer.h:238-276 | aquifer.rs:379-401 | AquiferSampler.java:263-321 | ✓（逐常量一致） |
| 11 | barrier 采样点 = 原始块坐标（非 cell 坐标） | aquifer.h:262-266 | aquifer.rs:395 | AquiferSampler.java:306-308 | ✓ |
| 12 | getFluidLevel：13 OFFSETS、bl2/bl3 逻辑、`getBlockState(o) != air` 门 | aquifer.h:290-319 | aquifer.rs:416-442 | AquiferSampler.java:353-389 | ✓ |
| 13 | method_43718 常量：erosion < -0.225F && depth > 0.9F（float 常量提升 double） | aquifer.h:335（-0.225f/0.9f） | aquifer.rs:447（-0.225f32/0.9f32） | VanillaBiomeParameters.java:1207（-0.225F/0.9F） | ✓（亲核 Java 源，浮点常量逐位同值） |
| 14 | getFluidBlockY：f=bl?clampedMap(i,0,64,1,0):0；g=clamp(floodedness)；h/k=map(f,1,0,-0.3,0.8)/(-0.8,0.4)；e=g-h、d=g-k | aquifer.h:329-353 | aquifer.rs:444-460 | AquiferSampler.java:391-419 | ✓ 语义同构；⚠️ 微项见 §4-d |
| 15 | getNoiseBasedFluidLevel：16/40、+20、spread·10、roundDown 3、min(est,q) | aquifer.h:355-366 | aquifer.rs:462-472 | AquiferSampler.java:421-433 | ✓ |
| 16 | getFluidBlockState：-32512 哨兵（field_35479）、≤-10 且 !lava 时 fluidType·|d|>0.3→lava，采样 cell(64/40/64) | aquifer.h:368-381 | aquifer.rs:474-485 | AquiferSampler.java:435-450 | ✓ |
| 17 | est 扫描（步长 8、域、阈值 0.390625、哨兵） | aquifer.h:145-164 | aquifer.rs:343-377（+EstL2 纯缓存） | ChunkNoiseSampler.estimateSurfaceHeight（m2 已裁决） | ✓（(200,200) 域复验仍开放 = P3，非 B3 范围） |

**表内零 ✗** —— B3 原命题（两侧共享的 aquifer 分支转录差）在静态层面不成立。

## 3. 疑点①：幕帘 granite/copper_ore 写者（互斥候选 ≥2 + 判别法）

硬事实（亲核）：
- oreVein 两臂 y≤50 硬上限：ore_vein.h:33、ore_vein.rs:46-48 → NOISE 阶段两臂均不可能在 192-318 产 granite/copper。
- vanilla feature 高度上界：ore_granite_upper = uniform 64..128 absolute（placed_feature/ore_granite_upper.json:13-22）；ore_copper = trapezoid -16..112（ore_copper.json:12-22）→ **vanilla 侧任何 granite/copper 写者都到不了 192-318**（ore_copper_large 未核范围，idk 登记见下）。
- surface 规则树（C++ surface.h / Rust surface_rules.rs）无 granite/copper 写点（grep 两 src 树：granite/copper 只出现在 ore_vein、carver replaceable 集、feature ore 目标表——carver 只写 air/lava，replaceable 集只是「可被雕刻」白名单，不是写者）。
- C++ 臂 features 整体跳过（worldgen_api.cpp:1585、feature_loader.h:5）；mod 臂 features 可被 bit1/WG_FEATURE_SKIP 跳过（worldgen_handle.rs:620）。
- block id：stone=1、granite=2（blocks.h:32 blocks.json 契约 + bin/vein_probe.rs:11-15 常量注释）。

候选（互斥）：

- **G-A：feature 阶段实际参与采集 + 高度域转录偏离**。若 E4-lite/mod 采集时 features 未被跳过，ore_granite/ore_copper 被放置，但其 height_range（uniform 64..128 / trapezoid -16..112）在 placement 解释层出错（absolute 语义未按「绝对 y」处理、或 min_y=-64 偏移重复/缺失 ±64 级错位）→ 落点整体上移到 192-318。落点形态应为离散团块（size 64 的 blob）而非「交替 run」——与幕帘 run 形态的相容性本身是判别点。
  判别法：① 同 chunk `WG_FEATURELOG=1` 开/关（Rust flags 或 WG_FEATURE_SKIP A/B）重导，对比幕帘区间 granite/copper 是否随 features 消失——消失即 G-A；② 开 `WG_FEATURELOG` 后查 `[FEATURE]` 行是否含 ore_granite/ore_copper 且落点 y∈192-318；③ 若确认，再审 placement.rs/placement.h height_range absolute→y 变换（已知风险注释 placement.rs:288 / placement.h:326：modifier 顺序错即 height 全错）。
- **G-B：导出/分类层 block id 错位**。NOISE 写的 stone(id=1) 在导出映射/registry 中被错解为 granite(2)/copper_ore(923)（blocks.json 双向表错位、或 mod 侧 cppReplace 链 id 空间不一致）。预测「交替 run」其实是 stone/id 常量在导出文本层的漂移。判别法：① 现有导出直接取幕帘区间整数 id 直方图，核对 1/2/923 分布与 block registry 表逐一对名；② C++ 参照臂同列（C++ 无 feature 写者）若也出现 granite/copper id → 强烈指向 id 层或共享上游而非 mod feature（C++ 侧写者仅 aquifer(-1)/oreVein，均已排除）→ 只剩 id 错位或消费环 `stone` 常量错（worldgen_api.cpp:775 是 `id("minecraft:stone")` 动态查表，表错则全列错，无法解释「交替」——除非表本身重复 id）。
- **G-C（低先验，登记不主推）：ore_copper_large / granite_lower 范围未核 + 被排除项回潮**。ore_copper_large placed_feature 范围本臂未读（idk）；若其 uniform 上界含相对语义被错解可更高。判别法同 G-A ①。另外「部署 dll 早于 oreVein 上限修复」类工具链回潮（B6 类）用 dll sha256 + git 时间戳一行排除（writer-verdict 轮已有现役 dll=ec4a9aed 记录可复用）。

诚实声明：**本臂未运行任何探针，G-A/G-B 无法在静态层面二选一**；但 G-B 的「交替 run」形态在 id 错位模型下缺乏周期性机制（为什么 1/2/923 交替而非全列错），形态证据其实偏 G-A 或「幕帘 stone 本体 + 某种按位置的选择表」——后者在两 src 树 grep 无落点，**写者整体仍 idk**（与 scout map P1 一致）。

## 4. idk 与微项登记（不影响主结论）

- a) ore_copper_large / ore_granite_lower 的 placed 高度范围未读（本臂只读了 ore_granite_upper + ore_copper）。
- b) 幕帘列真实 est 值未知（convergence P3：(200,200) 域 est 未复验）——结论 4 的「est≳200」是水口袋存在性的**必要条件推演**，非实测。
- c) C++ `estimateSurfaceHeight` 哨兵 INT32_MAX 进入 `surfaceHeightEstimate + 8 - blockY`（aquifer.h:339）是 signed overflow UB；Rust 同式 wrapping（aquifer.rs:450）。实机 x86 两者补码回卷同值，无行为差，但 C++ 侧为 UB 登记（触发前提 = 13 邻域 est 全哨兵，深海洋列先验极低）。
- d) `MathHelper.clampedMap/map` 与 C++/Rust `lerpClamp2/map2`（aquifer.h:384-391 / aquifer.rs:489-492）代数等价但 FP 求值形式不同（`c+t*(d-c)` vs `c+(d-c)*num/den`）——两 port 同形式所以**不构成 C++↔Rust 分叉**；仅当 e/d 恰好落在 0 的 FP 边界才可能三方分歧，先验极低，idk 不展开。
- e) 幕帘 stone 在 C++ 臂是否含 margin 贡献（est 足高时）未知——若实测 C++ stone ⊋ {d>0} 而 Rust stone == {d>0}，即结论 3 分叉的实锤形态。

## 5. 可证伪预测 + 主会话判别探针模板（不执行）

预测（每条可独立证伪 B3 残余框架）：
- **P-1（est 高位）**：seed 8576294172403134396 / chunk(200,200) 域幕帘列（如 195,199）的 est ≳ 200。若实测 est ≈ 30-60（正常海洋），则水口袋不可能由 aquifer 产生 → 复合结构模型崩塌，B3/B2 全线升级重查。
- **P-2（Rust stone=d>0 恒等式）**：Rust 幕帘列 stone 集合 == {y : d>0}（terrain.rs:232 路径推论）。出现 stone 但 d≤0 的点 → classify/消费环另有 bug。
- **P-3（C++ margin 贡献上限）**：C++ 幕帘列 stone ⊇ {d>0}，超出部分仅存在于 y ≤ est+20 区段且伴随 [AQF-e] SOLID 轨迹；y > est+20 区段 C++ 与 Rust stone 集合相同。
- **P-4（vanilla 空)）**：vanilla 同列 initialDensity 在 192-318 全 ≤ 0.390625 且无 stone/口袋（convergence 已有世界图景，此为独立自证 P4）。

探针模板（全部主会话执行， Degraded→Partial/Full 升级凭此）：
1. **est 实测（P-1）**：C++ `WG_ESTDUMP=1 WG_ESTDUMP_X=195 WG_ESTDUMP_Z=199`（worldgen_api.cpp:1090-1097）；Rust `WG_EST_DUMP=<path>` 四角（worldgen_handle.rs:587-601）；vanilla RouterProbe ESH 同列。三查 seed/坐标语义先行（scout map B6）。
2. **aquifer 逐块轨迹（P-2/P-3）**：C++ `WG_AQFDUMP=1 WG_SURFTRACE_X=195 WG_SURFTRACE_Z=199 WG_AQF_YMIN=180 WG_AQF_YMAX=320`（aquifer.h:71-124）→ [AQF]/[AQF-e] 行按 y 与 WG_DBDEBUG 同列 d 值（worldgen_api.cpp:924-938）做集合运算：stone_y == {d>0} ∪ {density+e/g/h>0}？Rust 用 bin-diag/aquifer_apply_breakdown.rs + b1_density_probe（WG_B1_COLS=195,199 WG_B1_YMAX=320）同式。
3. **granite/copper 写者（§3）**：① mod 臂 `WG_FEATURELOG=1` 与 feature-skip A/B 同 chunk 重导（worldgen_handle.rs:620-625），幕帘区间 granite/copper 随 skip 消失 → G-A；② 不消失 → 取整数 id 直方图对 registry（G-B）；③ C++ 参照臂同列（无 feature 写者）出现 granite/copper → 直接 G-B。
4. **口袋表（P-1 复核）**：C++ WG_AQFDUMP 的 bs=WATER y 序列 vs convergence 口袋表 {197,212,228,244,261,277}；vanilla BlockProbe 逐块 water-y 表（scout 缺口 G2）。

## 6. 对收敛的移交建议

- B3 原命题（共享「d≤0 误回 solid」）静态排除，建议收敛时记「❌ 无代码路径」一行；**保留 B3 的活体残余 = 结论 3 的 C++/Rust margin→stone/air 分叉**（Rust 单臂 vanilla 偏离，需单独立项或并入 B4 类「两臂作画边界微差」账目——它影响的是幕帘内部构成而非共有偏离）。
- 共有幕帘成因的证据权重经本臂分析后集中于 **B1（final+initial 密度高海拔抬升，一个上游成因同时解释 stone run、est 高位、水口袋）**；B2（est 路径差）降为 B1 的下游可测投影（P-1 探针同时裁决两者）。
- granite/copper 写者保持 idk（G-A/G-B 待探针），不随本臂收敛闭合。
