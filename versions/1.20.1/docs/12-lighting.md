# 12 · 光照引擎（Rust 重写）——P2 收口状态

> 状态：candidate（G1/G3/G3c PASS 判据已过，G2 转向 + D3 FAIL 待后续处理；judge APPROVE-WITH-CONDITIONS；confirmed 待用户拍板）
> 本篇只记结论与判据；过程/被推翻假说归 10 时间线。

## 结论链（猜测→验证→排除→发现）
1. **猜测**：G2 残差 = Rust fallback 收不到 propagateLight（光照传播调用缺失）。**验证/排除**：fan-out .b1 DENY——残差 0/448 全在 pregen 边界 + vanilla 传播是拉取语义（无主动 propagate 链）。
2. **发现（G2 真根因）**：残差 = **worldgen feature 放置分歧**（树叶/藤蔓/矿石/安山岩）；judge 全量 447 chunk palette 对比归因。光照内核在 blocks 一致域**无缺陷**。
3. **G1（内核正确性）**：blocks 一致域光照 exact 100%（口径：4×4@200 树冠一致区 16 chunks 光照 MCA 逐位 + 静态审查；**不为树冠差区域背书**）。
4. **G2 收口口径**：feature 放置分歧闭合后重测；当前残差归因已明，光照侧无待修项。
5. **G3（round-trip）二次收敛口径**：1.20.1 存档无 isLightOn 键 + 每次启动必 relight（vanilla 亦然）→「光照不变」严格判据对该引擎**不成立**；有效判据 = **二次重启收敛 0 + 相对基线**。首载漂移 vanilla 17/2025、rust 123/2025（**7×，待办**——相对基线差，二次重启均收敛 0；口径：2025 chunk pregen，首载 vs 二次重启）。
6. **G3c（nether/end sanity）**：PASS——64+64 chunks 生成、高度守卫回退 vanilla（bottomY=0 span=16 打点正确）、无新 crash。
7. **D3（性能）双 FAIL**：内核 6.88ms/chunk（合成数据）；e2e 回退 2.4×（29.7 vs 12.4s / 2025 chunks，gate ON vs OFF 全量 wall）。开销主体 = 内核 BFS。

## 已知语义问题
- light_data.json opacity:-1 经 parse_u8_field clamp 成 0（18 条目；与残差无关，语义有损，见 knowledge/discovered/compiler-idioms.md #14）。
- @anchor.idk（降级）：LightStorage.enqueueSectionData 再传播语义未核（round-trip 二次收敛已实证，风险降级；随 7× 漂移待办一并跟踪）。

## 对比判据（复用）
- 跨实现光照 MCA 对比 MUST 先核两侧缺键语义（vanilla 缺 SkyLight=隐式 15 / rust 缺键=flag1 全 0；统一填充 = 1.47M 假差异），见 workflow-patterns #46。
- MCA 解析坑（无 xPos / unpack_from 不推进）见 build-tooling #20；源码参照核版本见 f5-bugs #5。

## 被排除假说（一行排除清单）
- ❌ fallback 收不到 propagateLight——.b1 DENY（残差在 pregen 边界 + 拉取语义）
- ❌ 光照内核 BFS 缺陷——G1 exact 100%（blocks 一致域）
- ❌ isLightOn 缺失是 rust 侧 bug——vanilla/rust 双侧存档均无此键（.tmp/net 源树疑非 1.20.1）

## D3 性能优化 round1（种子收缩）——内核 5.80→3.72ms（1.56×），e2e 回退收窄至 1.74×（260905-04）

> 状态：candidate（C1-C4 judge 条件已落实，judge 收尾 APPROVE-WITH-CONDITIONS；「e2e 不回退」严格判据仍 FAIL，confirmed 待用户拍板）
> 口径声明（§9.7）：内核微基准载体 = `light_bench_real.rs` 链接 rlib 96a33b09；数据 = 真实 blocks9（vanilla WGB2 4×4@200 seed 8576294172403134396 抽 3×3）；256 chunks 批 wall，预热 8。与 260905-03 合成口径 6.88ms **不可比**（数据分布不同，仅量级对照）。

### 措施：sky BFS 边界 15 种子收缩
`worldgen-core/src/light/mod.rs`：原实现把全部 sky==15 cell（数十万）无差别进队；改为只让「边界 15」（6 邻域存在 sky<15 的 15 格）进队。

语义等价论证要点：
1. **内部 15 格零贡献**：内部 15 格的 6 邻域在扫描时刻已全 15，且 BFS 光照单调不降 → `try_spread` 的 `new_level > light[n]` 恒 false → 出队后零展开贡献；
2. **BFS 单调不动点与入队顺序无关**：光照传播是单调松弛，最终不动点唯一，入队集合中剔除零贡献种子不改变不动点 → 结果逐位不变。

同步改造：`light_compute` 委托 `light_compute_inner`（phases Option）+ `#[doc(hidden)] light_compute_phased` + PhaseTimings 探针（生产路径传 None 零开销）。

### golden 逐位不变对照方法（C4，可复用）
- 优化前：冻结 pre-opt rlib（96a33b09）跑 4 用例（synthetic + 3 真实 region）→ `golden_pre.txt`；优化后同 rlib 路径复跑 → `golden_post.txt`；FNV hash 逐用例等值 = PASS。
- 本轮 4 用例 hash 全等；region_a 与 blocks9_real 的 sky/flags hash 相同为合理非异常（相邻 flat plains，sky 直落同构、block 通道不同）。

### 数据
- 内核：blocks9_real **5.796 → 3.723 ms/chunk（1.56×）**；region_a 3.732 / region_b 3.645。
- e2e（C3，四臂交替 ON/OFF/ON/OFF，同机同 seed，删 world + RCON stop）：ON median **23.46s** vs OFF median **13.52s** = **1.74× 回退**（g3 基线 2.4×，收窄 29%）；绝对开销 17.3s→9.9s。可比性：四臂 worldgen 模式行一致、dll 同版（1955840）、ON 臂 lightInit ok、fallback=0。
- 未达原「内核 ≥2×」判据；剩余大头 = sky_seed_bfs / fill / sky_fall 三者均 O(N) 全域扫描（内存带宽型）。下一层候选（未实施）：边界扫描与 sky_fall/fill 融合、fill 直读 blocks9 布局、均质 section 跳过（需边界白名单）。Java 收集循环（884736 getBlockState/chunk ≈5.6s）成 e2e 下一大头。

### Phase 分解（C2，post-opt，64 chunks 均值）
| phase | blocks9_real | region_a |
|---|---|---|
| sky_seed_bfs（含边界扫描） | 1819.5 µs (48.3%) | 1846.8 µs (48.7%) |
| fill | 925.7 µs (24.6%) | 919.7 µs (24.3%) |
| sky_fall | 796.8 µs (21.2%) | 798.3 µs (21.1%) |
| export | 210.8 µs (5.6%) | 211.3 µs (5.6%) |
| block_bfs | 13.3 µs (0.4%) | 15.1 µs (0.4%) |

### 链条（猜测→验证→排除→发现）
1. **猜测**：内核 BFS 为 e2e 开销主体。**验证**：真实 blocks9 口径复核 = 5.796ms/chunk，量级成立 → 归因可继承。
2. **猜测**：内部 15 格种子是冗余。**验证**：单调性 + 邻域论证 → golden 4 用例逐位不变（C4）+ 内核 1.56×。
3. **排除**：❌ 合成口径 6.88ms 直接作优化基线——真实口径 5.80ms，不可比（calibration 复核取代，§15.4）。
4. **发现**：e2e 回退 1.74× 仍未达「不回退」严格判据——剩余为 O(N) 全域扫描 + Java 收集循环，算法级降维是下一层。

判据状态：「e2e 不回退」严格判据**仍 FAIL**，待用户拍板（接受收窄/继续 round2/其他）。

---

## D3 性能优化 round2（全扫融合 + 收集/JNI 直采）——内核 3.72→1.59ms（3.65× 累计），e2e 回退收窄至 1.25×（260905-04）

> 状态：**confirmed（2026-09-05 用户拍板）**；judge APPROVE-WITH-CONDITIONS（条件已全落实，见验证链）。
> 口径声明（§9.7）：内核微基准载体 = `light_golden_dump` / `light_bench_real` rustc 直编链 rlib；数据 = blocks9_real 4×4@200 抽 3×3 + region_a/b + synthetic；256 chunks 批 wall，预热 8。真实数据内核口径 **1.587 ms/chunk**，与 round1 同载体可比（3.723→1.587 = 本轮 2.34×；自 round1 基线 5.796 累计 3.65×）。

### 措施（三层）

**内核（worldgen-core/src/light/mod.rs）三项融合**：
1. **fill dom-major 重排 + col_max 同趟收集**：写侧连续（消 dom_index 乘法散写）；同趟记录每列最高不透明 y（`col_max`，Scratch 复用）。
2. **sky_fall 纯写趟**：15-区间 = [col_max+1, 383]，与原自上而下 blocked 扫描逐位等价（首不透明格 = 列最大 y），免 O(N) opacity 读（0.80→0.30ms）。
3. **边界种子区间算术**：15-区间每列连续 ⇒ 格为边界 ⇔ 列底（y==lo，下方不透明 sky=0；lo==0 无下方邻）或 ∃ 水平邻列 lo_nb>y——与 round1 O(N)×6 逐格扫描种子集合**严格恒等**，计算量 2304 列 × 4 邻区间（sky_seed_bfs 1.82→0.20ms）。

**air 快路径**（⚠️ luminance 陷阱）：`v==0` 且 `table[0]==(0,0)` ⇒ 免查表 + 免 col_max 写；陷阱 = `id=0 + luminance>0` 是合法光源（`15<<24`），必须判**全字 v==0** 而非只判 id 位——首轮实现漏 luminance 位，被合成 golden 立即抓出（golden 逐位门的价值实证）。

**Java 收集（ServerLightingProviderMixin.wgLightCollectBlocks）**：
- **section 直采**：`nc.getSectionArray()` → 逐节 `isEmpty()` 短路（Arrays.fill(0)=AIR，与 WorldChunk#getBlockState L199-207 空节语义等价）/ `sec.getBlockState(x,ly,z)` 局部坐标——免 884736 次/chunk 的 BlockPos+世界坐标→section 换算。
- **ThreadLocal 缓冲复用**：blocks9(3.5MB)+out(100KB) 每线程一份（原每 chunk 新分配 ~3.6MB，2025 chunks ≈ 7GB GC 压力）。

**JNI（jni_bridge.rs lightCompute）**：thread_local 缓冲复用（b9/ob/os/of；脏数据安全性：b9 全量覆写、out 由 export_center 全量写出、失败路径不写 out 且 Java 侧回退不引用）+ u8→i8 视图转换（`from_raw_parts` 同宽免逐字节转换拷贝）。

### 验证链（分层：内核 Full / Java+JNI Full / e2e）

- **golden 逐位（C4）**：4 用例逐位一致 PASS（golden_pre 现场重冻自 HEAD 92d9b7b，与 round1 golden_post hash 交叉一致）。
- **双采集对拍（judge 条件，已执行）**：新路 section 直采 vs 旧路 getBlockState，实机 gate ON 前 4 chunk 逐元素对比 = **4× MATCH（0/98304 diff）**；对拍后诊断已移除并复编通过。
- **e2e 四臂（C3）**：交替 ON/OFF/ON/OFF，同机同 seed 8576294172403134396，删 world——ON 中位 17.4s vs OFF 中位 13.9s = **1.25× 回退**（round1 1.74×、g3 基线 2.4×）；绝对开销 ≈3.5s ≈ 内核份额预测（1.59ms×2025≈3.2s，吻合）→ **Java 收集/JNI 侧开销已基本消除**。〔260914-04 round3 supersedes 本行后半「→ Java 收集/JNI 侧开销已基本消除」及前半「≈内核份额预测吻合」归因：round2 减法口径把 Java 段记 ≈0.15ms/chunk，被 round3 分段探针实测证伪——collect 串行 4.5ms/chunk（30×）；见本篇 round3 小节 + .investigations/light-round3-260914-04/probe-verdict-260914-04.md §1〕
- **judge**（review-d3-round2-260905-04.md）：APPROVE-WITH-CONDITIONS；must 条件双采集对拍已执行 ✓；should-fix：sec==null 措辞 ✓ / e2e n=2 噪声明示 ✓ / light 课题 .artifacts/index.yaml 登记（收口归档时补，standing）。

### 数据与遗留

- 内核 Phase 分解（post，含 Instant 开销）：fill 943µs(58%) / block_bfs 8.8 / sky_fall 300 / sky_seed_bfs 206 / export 157。
- fill 仍为最大头（~0.94ms，lookup 查表 + col_max 写）；下一层候选（未实施）：palette 级批量展开（Java 侧）、opacity u8 表内联。
- 降级声明：Java/JNI 侧收益未单独微基准（运行时验证须主会话/实机），以 e2e 四臂差值为证据；OFF 基线漂移 ±1.5s 属同量级噪声。〔260914-04 补强：Java/JNI 侧收益已补单独微基准（round3 collect 4.527→0.288 / native 2.770→2.651ms，见 round3 小节）——本行降级声明由 round3 撤销〕
- 过程/偏差 → 10-timewise-archive 260905-04 round2 条。

---

## D3 性能优化 round3（palette 展开收集 + packed 直传）——ON 路径 7.3→2.94ms/chunk，e2e 判据 FAIL 1.07 如实收尾（260914-04）

> 状态：**confirmed（2026-09-14 晚用户拍板「授权确认」——语义 = 接受现状归档；judge 收尾 PASS：三源核对 8/8、行为门全绿、e2e 判据 FAIL 1.072 如实保留，非判据通过）**。
> 口径声明（§9.7）：探针计时 = light 线程每 chunk 串行耗时；e2e = runServer boot pregen wall（~400-600 chunk 接管，gate `coreswap.light.rust`）；二者不可直接换算（#128）。跨批 e2e 绝对值不可比（整批漂移 ~2s 实测）。

### 探针定线（三源）与 round2 归因取代
- 源 K（内核 1482.6µs 同数据 sanity）/ 源 T（Java 段计时 collect 4.5ms、native 2.8ms）/ 源 P（palette 直方图：bits 4:95%、5:5%、6:0.4%，singular 与 ID_LIST(bits≥15) 均 0——1.20.1 实测样本内）。
- 对价模型：ON 串行 7.3 = collect 4.5 + native 2.8（内核 1.48 + JNI ~1.3）vs vanilla ~5.6；gap 1.73ms × 2025 ≈ 3.5s 双锚自洽。round2「Java 段≈0」减法口径被证伪（见上文 L94 取代标记；→ workflow-patterns #147）。

### 措施
- **B（Java palette 级展开收集）**：非空节走 `PalettedContainer.writePacket` 公有序列化帧（PC.java:383-387）→ Java 位流解码填 blocks9；mixin @Accessor 撞私有内部 record 不可行（→ compiler-idioms #27）。collect 4.527→1.257→0.288ms。
- **C（JNI packed 直传）**：Java 拆帧 → sectionMeta/paletteData/storage 直传 → Rust `light_decode_packed` 解码 → 同一 light_compute 内核（输出等价 = 同内核同输入，结构性承载）；游标式偏移免除法。native 2.770→2.651ms。
- 1.21.6 面：仅 Rust 增量导出（Java 零改动）；packed 收集未做（#26 帧格式差：定长无前缀），行为与 round2 一致（声明式，构建绿 + 语义零变化）。

### 验证链（行为门全绿）
golden 4/4 逐位（与 round2 冻结件一致）→ DUAL ALL-MATCH（B 输入层）→ DUALP ALL-MATCH（C 输出层）→ fallback=0（paldump 采样 0 节间接直证；无显式回退计数器——1.21.6 移植时建议补一行 counter）→ 诊断移除后复编 BUILD SUCCESSFUL。对拍抽样 = 4 chunk/轮（须含位流节，W1 教训 → build-tooling #151）；大样本正确性依赖 fallback=0 + 结构性论证，非全量逐位。

### e2e 判据：FAIL 1.072（如实）
判据线 ON/OFF 中位比 <1.05×（HOOK-2b，探针真值解除 judge「物理冲突」保留意见后维持）。B 期 1.070-1.083 FAIL / 四臂 n=2 不可判（OFF 极差 2.14s 与缺口同阶）/ 六对交错 n=6 中位比 **1.072 FAIL**（配对差均值 1.32s，单对噪声 ±1.5s）。**e2e 判据累计 3 轮未满足 → C-gate 触发，用户裁决「接受现状收尾」——判据未达标，非通过**。载体噪声下限（OFF 极差 2.6s/15% + 跨批漂移 2s）使 1s 级缺口在该载体不可判（→ workflow-patterns #148）。

### 净收与剩余
- ON 路径串行 7.3 → **2.94ms/chunk**（collect 0.29 + native 2.65）；e2e 回退 1.25× → **~1.07×**（收窄 ~2/3）。
- 剩余优化面（未实施）：解码-查表单趟融合（~0.6-0.8ms）+ A-② sky_fall 融合（~0.1-0.2ms）+ **e2e 载体更换**（Chunky region / 光照阶段计时）。
- 遗留风险：global palette（ID_LIST bits≥15）整 chunk 回退路径——1.20.1 实测 0 例但未证不可能；静默正确性由 rc=-2 闩回退结构保证，若发生为整 chunk 性能悬崖且生产无计数可见（建议下轮补回退计数日志）。双开关矩阵：`coreswap.light.oldcollect`（B 层）× `coreswap.light.blockabi`（C 层）正交——双开 = 全旧路径，任一关 = 该层走新路径。
- 过程错误 W1-W4 五段式 → record §5（W1 → build-tooling #151；W2 初值-谓词成对核对 #81 家族；W3 PowerShell 输出流即返回值；W4 回退计数=0 前提下读均值）。

## 形态审计 C 段（260915-01）：光照执行形态错配 6 行 open + 候选池归属（candidate）

> 形态审计 260915-01 的光照辖区结论；候选池总表与预验证探针见 07 篇「形态审计」小节。验证分层 Degraded（静态源码对照 + round3 既有探针引用，无新采集）。

### 五维度 open 错配（W2 行集摘要）
1. **L1 同步性**：mixin HEAD 完全同步内联（收集→JNI→写回→setLightOn→releaseLightTicket→completedFuture，Mixin:423-501），每 chunk 全部光照成本（2.94ms/chunk）占住一条 worldgen 车道；vanilla `light()` 异步两段（ServerLightingProvider:171-184），调用线程纳秒级返回。
2. **L2 线程放置**：vanilla = 同物理池上两个逻辑优先级队列（light TaskExecutor TACR:188）；我方无自有池，落点 = 调用线程（ChunkStatus.LIGHT 车道）。同池共享池宽，我方光照不可被高优先级 chunk 任务插队。
3. **L3 RefCell UB 面**（⚠️ 静态推理未运行时验证）：vanilla 串行保证在 light 队列 drain 侧、**不在 light() 调用侧**；我方调用侧 LIGHT 任务经 FJP 可并行 → 全局单例 lightHandle 的 `RefCell<Scratch> borrow_mut`（light/mod.rs:64）无同步 = 数据竞争 UB 面。修复：Mutex（锁开销可忽略，串行语义等价 vanilla）或 per-thread handle。
4. **L4 批粒度**：vanilla ≤1000 任务/chunk 间混批一次 BFS drain；我方单 chunk / 3×3 域，批化是 L5 增量化前的廉价中间形态。
5. **L5+L6 全量重算 ×9 + 无跨 chunk 缓存（最大嫌疑）**：N chunk 全亮我方总计算 = N×(48×48×384) = **9N chunk 域**，vanilla ≈ N 域且种子化 BFS 触达 ≪ 全域 → 结构总量比 ≥9×、有效工作量比 ~9×–数十×；机制面 = 无 LightStorage 式持久层，邻 chunk 被中心重算 1 次 + 作为邻居参与 8 次。round3 拼合：串行口径单核算力优势已消化 9× 冗余仍反超 1.9× → e2e 7% 回退更可能来自调度形态而非剩余算力差。

### 与 G3 首载漂移 7× 的同源性（一箭双雕权重项）
- (i) **邻域时机形态**（L5 同族）：我方以「3×3 邻 FEATURES 时刻快照」全量定值，vanilla 经共享 LightStorage 增量收敛；首载邻域成熟序不同 ⇒ 边界带系统性偏移，重载读旧值 ⇒ 0 漂移——与「一次性 + 收敛」签名相容。
- (ii) **L3 UB 面**：首载并行窗口偶发竞争写 → 错值定值 → 同样一次性漂移，亦相容。两候选当前不可分，探针 = 单线程强制复跑 gate ON 首载（漂移消失 ⇒ UB 主导；不变 ⇒ 时机主导）。
- (iii) feature 写入序候选：**排除同源**（方块输入差，非执行形态差，归 feature 课题 D12 交叉面）。

### 候选池归属
- CP-1 = L5+L6+L4（增量化，大工程）；CP-2 = L1+L2（解粘，中工程，**与 CP-3 同批**）；CP-3 = L3（Mutex，近零成本先行）。排序与执行序见 07 篇。
- 辖区外保留：enqueueSectionData 再传播语义 / G3 判据 = 本篇遗留 @anchor.idk（正确性课题，非形态）；INITIALIZE_LIGHT 无 mixin 无差异不立行。

### 等价行（论证成立，judge 抽查通过）
- 2c 写读隔离：LightStorage 读写隔离结构未改，enqueue 线程安全入队，等价；
- 4a 运行时增量化：gate 只拦 light() HEAD，checkBlock/setSectionStatus 未触碰，vanilla 增量引擎保留；
- 5c Scratch/ThreadLocal 缓冲：均为单次计算内工作缓冲，等价。
- **D-hm（judge 独立抽查已闭合）**：writeChunk 一次性补 6 型 heightmap 的等价条件成立——ProtoChunk.setBlockState 按 `getHeightmapTypes()` 逐型增量更新（ProtoChunk.java:108-155），CARVERS/FEATURES 携带 POST_CARVER_HEIGHTMAPS 四正式型（ChunkStatus.java:34-35）+ FEATURES 步前全量 populateHeightmaps 兜底（:150-152）；2 个 WG 型 carve 后不被增量维护但 vanilla 自身同样如此（PRE_CARVER 只挂 NOISE/SURFACE，:33/:115）——两侧形态一致，非 CoreSwap 引入差异（judge-review-260915-01 §2）。状态提升留人类。
