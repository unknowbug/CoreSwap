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
- **e2e 四臂（C3）**：交替 ON/OFF/ON/OFF，同机同 seed 8576294172403134396，删 world——ON 中位 17.4s vs OFF 中位 13.9s = **1.25× 回退**（round1 1.74×、g3 基线 2.4×）；绝对开销 ≈3.5s ≈ 内核份额预测（1.59ms×2025≈3.2s，吻合）→ **Java 收集/JNI 侧开销已基本消除**。
- **judge**（review-d3-round2-260905-04.md）：APPROVE-WITH-CONDITIONS；must 条件双采集对拍已执行 ✓；should-fix：sec==null 措辞 ✓ / e2e n=2 噪声明示 ✓ / light 课题 .artifacts/index.yaml 登记（收口归档时补，standing）。

### 数据与遗留

- 内核 Phase 分解（post，含 Instant 开销）：fill 943µs(58%) / block_bfs 8.8 / sky_fall 300 / sky_seed_bfs 206 / export 157。
- fill 仍为最大头（~0.94ms，lookup 查表 + col_max 写）；下一层候选（未实施）：palette 级批量展开（Java 侧）、opacity u8 表内联。
- 降级声明：Java/JNI 侧收益未单独微基准（运行时验证须主会话/实机），以 e2e 四臂差值为证据；OFF 基线漂移 ±1.5s 属同量级噪声。
- 过程/偏差 → 10-timewise-archive 260905-04 round2 条。
