# Q-AQ1 fan-out 候选 b1：生产 Aquifer/构建差异致 apply ~6× 贵（260903-10）

status: draft（置信度：静态分析 + 探针已写未跑；验证分层 = Degraded→Partial，探针运行后升 Partial）
口径（§9.7）：载体 = 本文档静态对比 + bin-diag 探针（待主会话运行）；覆盖面 = fill_chunk 循环内 aquifer 路径；与 F1-F8 同 seed/region 口径族可比。

## 1. 静态对比：生产 vs diag 构建与调用路径

### 1.1 构建（worldgen_handle.rs create L180-246 vs qaq1_apply_breakdown.rs L14-32）

两者都从同一 DensityBuilder 逐 key `build_node` 出独立树（barrier/flooded/spread/lava/erosion/depth/init 各自 Arc）。逐项核对：

- **无共享节点差异**：`build_node` 每 key 独立构建，生产树与 diag 树结构应逐位同构（同 JSON、同 seed、同 loader）。未发现 NoiseSet/采样器共享差异（生产 macro_sampler 用 `macrolize_channels(tree)` 只抽 final_density 内部 channel，与 aquifer 的 barrier/flooded 等树无交叠）。
- **splitter 同源**：生产 `db.random_deriver()` Xoro clone（L316-320）= diag L27-30 同构。
- ❌ 排除子假设「NoiseSet/构建态差异使单次采样更贵」——静态层面无此差异。

### 1.2 Aquifer 实例生命周期 —— **发现的真差异 D1**

- 生产 `fill_chunk_blocks`（worldgen_handle.rs L446）**每 chunk 新建 Aquifer**：`block_positions`(315×i64)、`water_levels`(315×FluidLevel)、`surface_cache`(1024×i32) **全部冷启动**；`cache_cx/cache_cz` 对齐当前 chunk。
- diag（qaq1_apply_breakdown L32,46-52）：**一个 Aquifer 跑 3 rounds 同 chunk** → round1 后缓存全暖，计时是 3-round **暖缓存平均**。
- 影响：diag 的 apply 内部 ~90-120ns/apply 是暖态下界；生产冷态每 chunk 有 bp miss 281 + wl miss 174 + surface_cache 全冷。但按 F2 计数 miss 总量 ~455 次/chunk，每次 miss 最贵路径 = get_fluid_level（13 offset × estimate_surface_height + floodedness/erosion/depth 采样）≈ 数 µs/次 → 455 × ~2µs ≈ **~1ms/chunk 量级**，**不足以解释 29ms**。
- 结论：D1 是测量可比性缺陷（diag 低估生产单 apply），但量级估算说明它最多贡献 ~1ms，**不是 G1 主因**。

### 1.3 载体差异 —— **D2（已知，证据包 F4 已标注）**

diag apply 完整 13.51ms 内含逐点 `tree.sample`（64k × 89ns ≈ 5.7ms）；生产 d 来自 macro cell-grid 插值。diag 内部 apply-only ≈ (13.51−5.7)/64k ≈ 120ns 暖态。生产 515ns/apply 是**冷态含缓存 miss**，两口径不可直接相减——D1+D2 合并解释 ~1-2ms，仍远小于 29ms。

### 1.4 生产 classify 包装层（terrain.rs L222-233）

- d>0 早退 Rock（每点 1 比较）；skip_aquifer → Air；enabled=false → sea_level 分支。包装开销 O(1) 比较级，98304 点 ≪1ms。❌ 排除。

### 1.5 fill_chunk 泛型循环（terrain.rs L247-289）

- 单遍逐列自顶向下；aquifer ON/OFF 在循环内**唯一差异 = classify 分支**（L277 `aqua.classify`）。surface_height/biome 采集两配置同路径。**fill 循环本身无 aquifer 耦合分支**。
- 泛型 `D: DensitySource<S>` 静态分发，无虚调用。❌ 排除「泛型/虚调用 6×」。

### 1.6 carver 交叉（aquifer.rs 计数共享；F3/F6）

- F2 的 68k apply/chunk **含 carver 的 `aq.apply(x,y,z,0.0)`**（carver.rs L409，绕 skip 标志）。carver apply 在 aquifer ON/OFF 两配置**都执行真实 aquifer 逻辑**（skip 标志只拦 classify 不拦 carver）→ 差分（35ms）理论上**不含** carver apply 成本（两边同额）。⚠️ 但需验证 no-aquifer 配置下 carver 是否真的一致执行（若测量配置还关了 carver 或其行为耦合 aquifer 状态，则差分被污染 → 归 b2）。
- 注意生产 `d`（macro 插值）与 diag `d`（树采）分布不同：d≤0 点集略不同（64-68k vs 64,433），apply 内部 `density + e > 0` 等分支走向可能偏移，但难以支撑 6×。

### 1.7 b1 静态结论

**未找到能单独支撑 6× 的构建/实现差异。** 唯一真差异 D1（每 chunk 新建 + diag 暖缓存计时偏差）量级 ~1ms。静态证据**弱不支持 b1**，但静态无法排除「fill 循环内 real-classify 全路径」（cache 行为/内存布局/分支分布的组合效应）——由探针 T3-T2 决定性裁决。

## 2. 决定性探针

文件：`WorldgenRust/src/bin-diag/qaq1_b1_prodfill_probe.rs`（已写入，未编译未运行——subagent 无 shell）。

设计：复刻生产 fill 循环（terrain.rs L265-287 逐行镜像 + 生产同源 DensityBuilder/DensityMacroSampler/Aquifer::new），分段计时：

| 段 | 内容 |
|---|---|
| T0 | macro grid 构建（build_slices_for，每 chunk 一次） |
| T1 | 插值循环无 aquifer（sample_interp only） |
| T2 | 插值 + classify(skip_aquifer=true) —— 生产 no-aquifer 配置语义 |
| T3 | 插值 + classify(真实, 每 chunk 新建 Aquifer=生产冷缓存) |
| T4 | 同 T3 但 Aquifer 跨遍复用（暖缓存对照 → 隔离 D1） |
| T5 | d≤0 直调 aq.apply（隔离 classify 包装；含插值采样） |

附 bp/wl 计数器（对照 F2 口径自检）。

运行命令（主会话执行；在 `WorldgenRust/` 目录下）：

```powershell
Set-Location E:\PYTHON\CoreSwap\WorldgenRust; cargo build --release 2>&1 | Select-Object -Last 3; rustc --edition 2021 -O --extern WorldgenRust=target/release/libWorldgenRust.rlib -L target/release/deps src/bin-diag/qaq1_b1_prodfill_probe.rs -o target/release/qaq1_b1_prodfill_probe.exe; target/release/qaq1_b1_prodfill_probe.exe
```

## 3. 预期判读表

| 结果模式（per chunk） | 判读 | 归属 |
|---|---|---|
| A. T3−T2 ≈ 30-40ms（复现 35ms），且 T5−T1 ≈ 同量级 | 差异确实在 fill 循环内 apply 热路径；T4−T3 揭示 D1 量级；若 T4 仍贵 → Aquifer 内部冷/暖都贵 → 与 diag 分解矛盾 → 查 diag 分解覆盖面（diag 各段为固定上界模拟，非真实 apply 覆盖） | **b1 支持**（结构性成本在 fill 循环内） |
| B. T3−T2 ≈ 5-8ms（diag 可解释量级） | fill 循环内不贵；生产 35ms 来自循环外——carver apply / 测量级联 / 计数盲区 | **b1 否定，转 b2/b3** |
| C. T3−T2 ≈ 15-25ms（中间值） | fill 循环内贡献一部分 + 循环外仍有缺口；按 T4 分离 D1 后剩余归循环外 | **b1 部分支持**，残差转 b2/b3 |
| D. T2−T1 本身 >5ms | skip classify 也有大开销 → 差分被包装层污染（意外发现，重审 F1 差分口径） | 方法论修正，三候选重审 |
| E. T0 宏观网格构建 >5ms | aquifer ON/OFF 下 T0 相同 → 不进差分；仅说明 density 底座归因需修正 | 不影响 b1，修正 F1 底座口径 |
| F. counters 与 F2 明显不符（bp≠~815k、wl≠~110k 每 chunk） | 探针循环与生产循环行为不一致（复刻失败），本轮结果作废 | 探针修正后重跑 |

## 4. 探针结果解读（260903-10 补，原始输出 cmd-output/qaq1-b1-prodfill-260903-10.txt）

结果模式 A（修正版）：**T3−T2 = 32.20ms ≈ 生产 35.07ms —— 缺口在 fill 循环内复现**（探针无 carver/无生产额外段）。counters bp 784k/wl 106k 与 F2（815k/110k）同量级 ✓（模式 F 排除）。

### 4.1 关键数字

- T3−T4 = **26.65ms** = 冷态超额；warm 态 aquifer 段 T4−T2 = **5.55ms** ≈ diag 可解释量级 ✓。
- bp miss 251 + wl miss 158/chunk ≈ 409 次 → 若只算 Aquifer 自身 Vec 缓存 miss，26.65ms 意味 ~65µs/miss——Aquifer 自身缓存绝无此价。

### 4.2 冷态成本真正来源（已核实源码）：InterpolatedData/FlatCache **单槽 chunk-key 抖动**

> **[supersedes 260903-10 v2] 本节机制已被 GRID_ARG_SAMPLES 探针反驳**（cmd-output/qaq1-grid-thrash-260903-10.txt：冷/暖增量均 0 → build_grid 从未执行；且 initial_density 子树核实无 interpolated 节点，§5.1）。正文保留不改（§15.4 取代链）；修正归因见 §5。

- `density.rs L280-290`：每个 `Interpolated` 节点 thread_local **单槽**（`slot.key` = 单个 chunk key），key 不匹配 → `build_grid` **全量重建 5×49×5 = 1225 个角点**（每角点一次内层 arg 树采样，L261-279）。FlatCache 同为单槽（L465-470）。
- 触发链：`get_water_level_at` miss（158/chunk）→ `get_fluid_level`（aquifer.rs L338-364）→ **13 个 offset 列**（x 偏移 −3~+1 chunk，z ±1）各调一次 `estimate_surface_height` → 每列自顶向下 ~34 次 `initial_density.sample`（aquifer.rs L292-296）。
- initial_density 树内 Interpolated 节点单槽：13 列横跨最多 5 个 x-chunk 交替采样 → **每次换 chunk key 整网格重建**。量级核对：158 miss × ~4-5 次重建 × 1225 角点 × ~30ns ≈ 25-29ms —— 与 26.65ms 吻合。
- **生产同病**：生产 Arc 树跨 chunk 存活但单槽 key 仍按 chunk 翻转，且每 chunk 新建 Aquifer（surface_cache 冷）→ 首 wl miss 必全量走 13 列。T3 即生产语义，32.2ms ≈ 35.07ms（差 ~3ms = 生产段 carver 侧少量共担/测量噪声）。

### 4.3 为什么 diag 之前"漏掉"了它

qaq1_apply_breakdown 打印了 `get_fluid_level(全调用上界): t_fl` 行，但证据包 F4 摘要**未收录 t_fl**（只收 bp/wl/caldensity/apply 四项合计 4-6ms）——t_fl 正是含 13 列 est_surf + Interpolated 抖动的整条链，数值大概率 ~25-30ms 量级，被当"上界"丢弃造成 G1 假缺口。**修正：缺口不是生产比 diag 贵 6×，而是 F4 摘要漏了一项。**

### 4.4 b1 候选裁决

- b1 原表述（生产 Aquifer 实现差异）**否定**：Aquifer 构建/实现与 diag 同构，D1（每 chunk 新建）量级 ~1ms。
- "生产 fill 循环内结构性成本"半边**成立**，机制已定位：**aquifer 触发的跨 chunk 列采样 × InterpolatedData 单槽抖动**。归属修正 = b1'（结构性，机制如上）+ b3 变体（F4 摘要遗漏 t_fl）。b2（carver/测量级联）非主因（探针无 carver 已复现 32ms）。

### 4.5 后续验证建议（交主会话）

1. 决定性确认：density.rs 已有 `GRID_ARG_SAMPLES` 计数器（build_grid 每次 fetch_add，L274）——env 门控打印冷态每 chunk 增量，预期 ≈ 数十次 × 1225；warm pass ≈ 0。
2. 修复方向（lossless-accel 课题）：① InterpolatedData 槽扩 2-5 槽（覆盖 est_surf −3~+1 chunk 窗口）；② surface_cache 不随 Aquifer 每 chunk 重建（跨 chunk 持久化）。预期吃掉 ~26ms 大头。

## 5. v2 重归因（260903-10，GRID_ARG_SAMPLES=0 反驳后）

### 5.1 反驳与新证据（静态核实，非猜测）

- **InterpolatedData 抖动机制否定**：GRID_ARG_SAMPLES 冷/暖均 0。旁证：对 noise_settings/overworld.json 内 `initial_density_without_jaggedness` 子树做括号配平提取，实际组成 = add×5 / mul×4 / y_clamped_gradient×2 / clamp / quarter_negative + reference **overworld/depth**、**overworld/factor** —— 无任何 interpolated 节点。§4.2 的"158 miss × 重建 × 1225 角点"算术是凑数（两处自由参数），作废。
- **aquifer 各 DF 树组成**（同法提取）：barrier / fluid_level_floodedness / fluid_level_spread / lava = **单个 noise 节点**（~百 ns 级）→ get_fluid_block_y 的 floodedness/erosion/depth 采样不贵。
- **重量叶定位**：`depth.json`（add+y_clamped_gradient+reference）→ `sloped_cheese.json` → `base_3d_noise.json` = **old_blended_noise**。density.rs L177-223 `InterpolatedNoiseData::sample`：**无任何缓存**，每次 sample = 8(interpolation)+16(lower/upper) 次 octave `sample_ys`。`factor.json` = flat_cache(cache_2d(spline(...)))（flat 单 chunk 槽，25 角点重建**不计数**）。

### 5.2 新归因假设 H*

**冷态超额 = estimate_surface_height 全量扫描 × initial_density 全价采样（old_blended_noise 主导）**：

- warm 态（T4/生产第二遍同 chunk）：surface_cache 命中 → est 一次树采样都不做 → aquifer 段只剩 ~5.5ms（T4−T2）。
- cold 态（T3/生产每 chunk 新建 Aquifer → surface_cache 冷）：每 est 列自顶向下 ~34 次全价 init 采样。
- 算术：~350-400 个不同 est 列/chunk × 34 iters × **~2µs/sample（old_blended 24 octave）** ≈ 24-27ms ✓ 吻合 26.65ms。全部落在已计数路径（SURF 计数器 214 calls × 34.35 iters = 7342，F5 自己就数出来了）。
- **F5 的 0.089µs/sample 判定为错误基线**（若真，est 全链只有 0.65ms，与探针 26.65ms 直接矛盾；0.089µs 连单个 old_blended_noise 的 24 次 sample_ys 都盖不住）。可能测的是别的树或经缓存命中路径。
- 26.65ms/600 次"miss"≈40µs 的矛盾消解：**贵路径的真实次数不是 600 次 cache-miss，而是 ~7342 次全价树采样**（SURF iters 计数）。

### 5.3 决定性探针 v2（已写：bin-diag/qaq1_b1_coldpath_probe.rs，未编译运行）

分段：A = init 树扫描式采样微测（est 形态：固定列 y 步 −8）→ ns/sample；B/B1/B2/B3 = depth / factor / sloped_cheese / base_3d_noise 单独同测（归因占比）；C = Fresh Aquifer `diag_fluidlevel_cost` pass1(cold) vs pass2(warm) + SURF 计数器 → 隐含 ns/iter。build_slices_for 预热问题对本探针无关（测的是 init 树直采，不依赖 INTERP 槽；probe 里 init.sample 预热仅触发 TLS 槽结构分配）。

运行命令（主会话，WorldgenRust 目录）：

```powershell
rustc --edition 2021 -O --extern WorldgenRust=target/release/libWorldgenRust.rlib -L target/release/deps src/bin-diag/qaq1_b1_coldpath_probe.rs -o target/release/qaq1_b1_coldpath_probe.exe; target/release/qaq1_b1_coldpath_probe.exe
```

判读：A 的 ns/sample ≥ ~1µs 且 B3(base3d) 占 A 的大头、C cold−warm ≈ 20-30ms → **H* 成立，G1 闭合**（缺口 = est 扫描 × init 全价采样 + warm apply ~5.5ms + F4 漏 t_fl）。若 A 反而 ~0.1µs → F5 对、H* 错 → 剩余归因转 factor flat_cache 重建路径（B1 应显示异常）或升级人类。

### 5.4 修复方向（若 H* 证实，替代 §4.5-2）

1. **est 扫描成本**：estimate_surface_height 步长 8 从 y=320 起扫是 Java 语义，但采样载体可换——init 树在 est 场景可用宏观粗化（与 fill_chunk 同款 cell-grid 插值复用 slices）或对 init 也建 per-chunk slices 缓存（每 chunk 一次构建，est 查表）。
2. surface_cache 跨 chunk 持久化（保留下）仍是次级收益。

## 6. 自检清单

- 类型/签名：全部 API 经 grep/read 确认（Aquifer::new 7 DF + splitter + 4 坐标参数；DensityMacroSampler::new(&tree,min_y,height)；build_slices_for pub；ChunkDensitySampler::sample_interp pub trait；aquifer_bp/wl_count_reset 均 [usize;2]——中途修过一次 bp 返回类型误判）。
- 未编译验证：**探针未编译未运行**（subagent 无 shell，主会话执行 §2 命令）。
- 已知风险：① `WorldgenRust::xoroshiro::XoroshiroSplitter` 路径按 lib.rs L22 推定（xoroshiro mod pub），若实际在别处编译报错改 import 即可；② black_box 防优化删除；③ T4 灌缓存遍复用同 va2（同 chunk 同 slices）语义自洽。
