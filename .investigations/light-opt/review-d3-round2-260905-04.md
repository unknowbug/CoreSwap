# review · D3 光照优化 round2（260905-04）

- 审查角色：anchor.judge / core.judge（隔离 subagent，只出意见，不改 status，不授 confirmed）
- 审查对象：.investigations/light-opt/d3-opt-round2-260905-04.md 及其代码交付
- 三源核对基线：① golden/e2e 落盘产物（.tmp/d3-opt-260905-04/）② git diff（HEAD 4496586 + 工作区）③ round1/round2 记录 + e2e 原始日志
- 结论：**APPROVE-WITH-CONDITIONS**

## 三源核对结果

1. **git diff 实际改动 vs 记录声明**：工作区 diff = `worldgen-core/src/light/mod.rs`（~103 行）+ `versions/1.20.1/rust/src/jni_bridge.rs`（~92 行）+ 4 个 .investigations 记录更新——与 round2 声明的改动范围（内核四项 + JNI thread_local/i8 视图）一致，无越界改动。⚠️ Java 侧 `ServerLightingProviderMixin.java` 位于 gitignored `runtime/`（.gitignore:87），**无法经 git 三源核对该文件历史**，仅能以磁盘当前内容审查（内容与记录描述一致）——info。
2. **golden 链交叉核对**（独立复算，非采信记录）：golden_pre_round2.txt 与 round1 golden_post.txt 内容 diff = 0；golden_pre_round2 与 golden_post_round2 逐行 Compare-Object = 0，4 用例 case-hash 全等——C4 PASS 成立（两文件 SHA256 不同为编码差异，逻辑内容逐字节等价）。
3. **e2e 日志核对**：e2e-arm1-on.log 实测 `Done (17.119s)` + `lightInit ok`，与记录表一致；四臂日志文件齐备（11:35–11:41）。

## 逐项发现

### a. 内核三项等价性论证 —— 通过（judge 独立重推，非复述）

- **air 快路径 v==0 充分性：成立**。`v==0` ⇒ id=0 且 lum=0；`lookup(0)` 返回 `table[0]`，air_fast 前提 `table[0]==(0,0)` ⇒ op/em 均为 0，与通用路径逐位一致。id=0+luminance>0 因全字判断被正确排除（round2 记录的陷阱自捕是 golden 逐位门的有效实证）。air_fast=false 时自动走通用路径，无双态漂移。
- **sky_fall col_max+1 区间等价：成立**。fill 按 y 升序扫，col_max = 列内最高不透明 y；15-区间 = 严格高于最高不透明格，与原自上而下遇阻即停扫描的 15 集合恒等。配套不变量：不透明格 sky 恒 ≤14（BFS 15-传播特例要求 op==0），故「列底下方 sky<15」论证成立。
- **边界种子区间算术 vs round1 逐格扫描：集合恒等成立**（judge 逐邻域类重推）：①列底 y==lo（lo>0）下方为 col_max 不透明格 sky≤14 ⇒ 恒为 round1 种子，恒入队 ✓；②水平边界 ⇔ y<lo_nb，代码对每邻列推补集区间 [lo, min(lo_nb,384)-1]，多邻列区间并集 = 全部水平边界 y ✓；③lo==0 列底无下方邻，仅水平判定，代码以 bottom_covered 防重复且不漏 ✓；④lo_nb=384（邻列无 15）时 hi=383 全区间入队，与逐格扫描一致 ✓；⑤y+1 方向不可能越界（区间向上连续到顶）✓。
- info：同一 y 可经多邻列区间重复入队 + 列底与区间重叠（lo>0 时 bottom 与区间 [lo,hi] 重复推 y=lo）——BFS 对重复种子幂等（try_spread 要求严格递增），只多常数开销，不影响结果。不影响等价性，无需修。

### b. Java section 直采 vs WorldChunk#getBlockState —— 通过（附 info）

- 空节：`sec.isEmpty()` 短路 + `Arrays.fill(0)` 与一手源 L202-204 的 `!isEmpty() → section.getBlockState` / 否则 AIR 分支一致（同一 isEmpty 谓词）✓。
- sectionArray 长度：一手源守卫 `l>=0 && l<sectionArray.length`，mixin 守卫 `secs.length<24 → 回退`——mixin 更保守（长度≠24 直接回退 vanilla），等价或更安全 ✓。
- y 范围：s↔getSectionIndex(y) 对齐由前置守卫保证（bottomY=-64、span=24），s∈[0,24) 与世界 y 区间一一对应 ✓；sec==null 判空、FEATURES 状态门 + null 判均已守卫。
- **should-fix（记录措辞，非代码）**：round2 记录 §验证分层称「sec==null/nullary 差异方向均为回退（safe）」——实现实际是 **AIR 填充**而非回退（sec==null 时 vanilla WorldChunk 会 NPE→CrashException，mixin 返回 AIR）。与 vanilla 的 crash 行为不构成等价目标（实际不可达输入），方向安全，但**声明措辞与实现不符，应更正为「sec==null 按 AIR 处理（保守，实践不可达）」**，一行修正。若严格按「声称与实现矛盾」论处本可 blocking，鉴于不影响任何可达输入的等价性且方向安全，降为 should-fix。
- info：`isDebugWorld` 分支未复刻（WorldChunk L186-196）——debug 世界不在课题范围，标注即可。
- FEATURES 状态假设为 round1 C 条件已声明项（LIGHT 任务时邻 chunk ≥FEATURES），round2 未引入新的状态越界假设（空节跳过 = vanilla getBlockState 同语义，非 C1 型越界）。

### c. JNI thread_local 脏数据 —— 通过

- b9：每次调用 `get_int_array_region` 全量覆写（长度先校验=884736）✓。
- ob/os/of：仅 `light_compute` 成功路径全量写出（export_center 每 section fill 后全写、flags 48 字节全写）；`rc!=0`/抛异常路径 Java 侧一律回退且**不读 out**（wgLightRustTakeover L210-213 / L203-209），panic 路径残留脏缓冲不可达 ✓。
- panic 后状态：scratch 所有缓冲 clear+resize 自清理，RefCell 借用随 unwind 释放，catch_unwind 防 JVM 崩 ✓；无重入（light_compute 内不回调 JNI）。
- i8 视图：u8/i8 同宽 reinterpret，`from_raw_parts` 生命周期限于借用内 ✓。

### d. e2e 结论有效性 —— 有效，附 should-fix

- 四臂交错 ON/OFF/ON/OFF、同 seed、每臂删 world、dll 时间戳三元组核对——方法学合规；ON 臂复现 ±0.3s；OFF 漂移 ±1.5s 已如实记录；3.5s 差值与内核份额预测 3.2s（1.59ms×2025）自洽 ✓。
- RCON 失败强杀发生在 Done 行之后，Done 为计时终点，数值有效——接受；enable-rcon 已恢复，无遗留。
- **should-fix**：每条件 n=2、OFF 漂移带 ±1.5s（≈11%），1.25× 结论方向可信但精度薄——课题收口前建议补 2 臂（或明示接受该噪声带）；不阻塞本轮 candidate。

### e. 降级声明（Java/JNI Partial/Degraded）—— 可接受，附条件

- 声明诚实、分层正确：内核 Full（golden 逐位 + phase + 真实数据 bench）、Java/JNI 依赖 yarn 一手源语义复刻 + 编译期签名核实 + 运行时 smoke（fallback=0 × 四臂、无新 ERROR），无行为级 A/B 对拍——**对 candidate 可接受**。
- **条件（confirmed/课题收口前 MUST）**：落实记录中自提的「前 N chunk 双采集 hash 对拍」（旧逐格收集 vs 新 section 直采，同输入 hash 一致一行日志）转 Full；`runtime/` 侧 Java 文件不入 git，建议在记录中附该文件 sha256 固定审查基线。

### f. golden 口径可比性 + 产物契约

- §9.7 三要素（载体=light_golden_dump/light_bench_real rustc 直编 rlib；数据=blocks9_real 3×3+region_a/b+synthetic；256 chunks wall 预热 8）在 round2 §内核验证同行声明 ✓；与 round1 同载体同数据（blocks9_real.bin 09:43 未变）、同计法，跨轮内核 3.723→1.587 可比 ✓；golden pre 现场重冻自 HEAD 92d9b7b 并与 round1 golden_post 交叉一致（judge 复算 diff=0）✓。
- **should-fix（standing，round1 起未解）**：light 课题结论无任何 `.artifacts/` + index.yaml 登记（全仓 .artifacts 无 light 条目），结果仅存 .investigations + 待落 docs——core-artifact 契约（结果进 .artifacts，.investigations 只放思维链）未履行，应在收口归档时补登记。

## 裁定

- **APPROVE-WITH-CONDITIONS**。无 blocking 项（测试/编译均绿；等价性论证经独立重推成立，golden 4 用例逐位一致经复算确认）。
- **candidate 可授**：内核结论（Full 层）可直接授 candidate；round2 整体结论（含 Java/JNI 与 e2e 1.25×）授 candidate 附以下条件：
  1. confirmed/收口前 MUST：双采集 hash 对拍转 Full（e 项条件）；
  2. should-fix：修正 sec==null「回退」措辞（b 项）；
  3. should-fix：e2e 补臂或声明接受噪声带（d 项）；
  4. should-fix：.artifacts 登记 + mixin sha256 基线（f/e 项）。
- confirmed 仍留人类拍板；本意见不改变任何产物 status。
