# 11× 并发放大 — 完整排查过程日志（详细，含每步为什么 + 数据 + 教训）

> 目的：**详细记录整个过程**（不是只记结果），避免下次遇到类似问题因只记结果而重跑确认「为什么是这个结果」。
> 覆盖：本 session 主线「production density 并发 11× 争用定位」从 DFC 失败定论 → 逐项排除 → 锁定 interp/noodle 访存。
> 方法遵循：AGENTS.md「测量/探针污染铁律」（并发下禁 WG_PROFILE/WG_STAGETIMER，只用 WG_PHASETICK QPC）+「吞吐 vs 每 chunk 延迟分开」+「错误优先原则（五段式）」。
> 状态：**过程记录（draft）**；排除链为本轮 confirmed 级证据；访存机制勘探进行中。

---

## 0. 起点与背景（为什么进入这条线）

- **Dfc 失败定论**（前 session）：DFC C# 移植（CpuBackend split 预拆分）失败 = 600× 慢，且是「实现（split=GPU 设计）错」不是「直排方向（去递归/去虚调用/去寻址）错」。scout 明确「DFC 直排仍对，只是 split 实现错」。
- 用户拍板：**研究 (b) production 并发争用的无损修复（非 DFC）**。
- 关键既有事实（scout/测量）：production density 单 chunk T=1 39ms → T=8 331ms = 8.4×；纯 noise 1.07×（无争用）。「11×」= 放大比 median(T8)/median(T1)。

---

## 1. locFn 连续化 A/B（SERIAL）— 试验 1：存储布局

**为什么做**：scout 候选 A = 散布堆 locFn 指针追逐（每个 spline 节点 shared_ptr deref + 虚调用）。想验证「locFn 存储连续化能否解 11×」。

**怎么做**：density.h SplineDF 加 SERIAL 路径（`WG_SERIAL_LOCFN` env）：`locationFunctions` 从 `vector<DF>`（shared_ptr 散布堆）→ 按类型连续池（flatCachePool/cache2dPool/binopPool 实体）+ `LocFnRef{kind,index}`。sampleNode 经 `sampleSerialLocFn`（kind-switch 选池）+ 用 `static_cast<const DensityFunction&>(pool[i]).sample()`。

**测量**（conc_density_probe 12 固定 chunk + WG_PHASETICK，median density ms）：

| | T1 | T8 | 放大比 |
|---|---|---|---|
| BASE | — | — | 10.03× |
| SERIAL | — | — | 10.25× |

**结论**：SERIAL ≈ BASE（10.25× vs 10.03×，持平）→ **locFn 存储布局非争用**（scout 候选 A 排除）。

**⚠️ 教训①（重要，后续修正）**：SERIAL 的 `sampleSerialLocFn` kind-switch 后 `static_cast<const DensityFunction&>(pool[i]).sample()` 转回基类引用 → **仍是虚调用**！所以 SERIAL 只去掉「shared_ptr deref + 存储连续化」，**从未去虚分派**。A/B 只能证明「存储非争用」，**不能**证明「虚分派非争用」。**`static_cast<const DensityFunction&>(obj).sample()` = 强制虚调用**，不是去虚调用。

---

## 2. measurement 修正：production SplineDF 不是深链

scout（d5bb8c50「测量生产 SplineDF 树结构」+ 现有 evidence）测量：production SplineDF 是**浅而宽**（递归深度仅 3 边/4 级，但节点多：factor 135/node、offset 254/node，共 ~433 节点，表 13.8KB）。**非深链** → 早期「深链递归」推测修正。

**机制（scout 测量确认）**：无跨实例 SplineDF 嵌套（coordinate 全解析为噪声/二进制）；真实争用 = 每采样点长 DF wrapper 虚调用链（InterpolatedDF.grid→blend_density→...→spline，15-20 层）+ spline 宽递归 → 8 线程灌同一缓存层级 → 每级延迟膨胀（15.8→190μs）。

---

## 3. 对照确认差异在 spline/wrapper 链：noise vs density

**为什么做**：确认「11×」差异来自 spline/wrapper 链（不是 noise 或公共基础设施）。

**怎么做**：conc_sample_probe（density/noise 单点对照）。

**数据**：noise 1.07× vs density 8.4× → **差异在 spline/wrapper 链的 load 争用**（noise 无争用）。

---

## 4. NOSPLIT（去递归）— 试验 2：递归

**为什么做**：scout 候选 B = spline 递归串行依赖链（latency-bound）。想验证「去递归能否解 11×」。

**怎么做**：SplineDF 加 `sampleNodeStack`（递归→显式栈 128 帧）+ `WG_DFC_NOSPLIT` env。保 production 表（nodes/locations/derivatives/subIdx）+ locFn 虚调用。

**数据**：NOSPLIT T1 34.91 → T8 345.78 = **9.9×** vs BASE **10.38×**（持平）。

**结论**：**去递归无效**（递归非争用）。NOSPLIT 保留了 locFn 虚调用 + wrapper 链虚调用（未动）。

**⚠️ 教训②**：NOSPLIT/SERIAL 都**没去「虚调用本身」**——递归和存储都改了，但虚调用还在。虚调用是剩余候选。

---

## 5. DEVIRT（去 spline.locFn 虚分派）— 试验 3：虚分派

**为什么做**：剩余候选 = 虚调用。先隔离 spline.locFn 虚调用（次要那份）。

**怎么做**：先改 `sampleSerialLocFn` 去掉 `static_cast<const DensityFunction&>`（3 case），具体类型直接调 `.sample()`（by-value 池，语义保证 devirtualize，O2）。env `WG_SERIAL_LOCFN=1`（DEVIRT）。

**测量**（conc_density_probe，12 chunk）：

| | T1 | T8 | 放大比 |
|---|---|---|---|
| BASE | 33.54 | 346.26 | **10.32×** |
| DEVIRT（+WG_SERIAL_LOCFN） | 34.03 | 342.06 | **10.05×** |

**结论**：DEVIRT ≈ BASE（10.05× vs 10.32×，降 2.6% 噪音内）→ **spline.locFn 虚分派非争用**。① 排除。

---

## 6. wrapper 链隔离 — 决定性转向主靶

**为什么做**：做了①（spline.locFn 虚分派）无效，但怀疑主靶是 wrapper 链。要隔离 wrapper 链。

### 6.1 探针实现（worker 交付，先 scattered 失真）
- density_builder.h `getSplines()/splineCount()` + SplineDF 捕获；worldgen_api `wg_sample_spline`（直接采样单 SplineDF，绕 wrapper）；conc_sample_probe spline 模式；build.ps1 加 conc_sample_probe。

**⚠️ 教训③（探针失真）**：conc_sample_probe spline 模式初用 scattered 坐标（`x=3200+(i*17)%2048`，跨 128 chunk）。spline 的 locFn（FlatCacheDF）grid 按 chunk 懒建，scattered 坐标 → **每换 chunk 重建 grid** → per-sample = 440552ns（0.44ms），**比 production 慢 1000 倍，完全失真**（grid 重建主导，非生产路径）。

### 6.2 修正：固定同 chunk（grid 命中）
**为什么**：生产 fillOneChunkCore 是「同 chunk grid 命中」访问模式。改 conc_sample_probe 固定 x,z 同 chunk（3200-3215/3224-3239）、y 扫 → grid 命中 → per-sample 4493.5ns（快 98×）→ 可靠。

**spline 并发放大**（conc_sample_probe，std::thread，固定同 chunk）：
| spline | 节点 | T1 | T8 | 放大 |
|---|---|---|---|---|
| [0] | 135 | 4139.9ns | 5039.0ns | 1.22× |
| [2] | 254 | 4493.5ns | 5439.3ns | 1.21× |

**⚠️ 教训④（线程模型混淆；关键）**：conc_sample_probe 用 **std::thread**（各线程独立循环），production 争用（10.32×）用 **wg_worker pool**（wg_fill_blocks_multi 填 chunk）。**线程模型不同** → spline 1.2× **不能**独立证明「spline 在 production 下无争用」（std::thread 下多入口都低放大：noise 1.15×/spline 1.2×，可能 std::thread 本身无争用）。**spline 1.2× 仅作辅证**。

### 6.3 决定性：WG_SPLINE_FILL（production 模型严格对照）
**为什么**：消除线程模型混淆，用 production 线程池（wg_fill_blocks_multi）测 spline 绕 wrapper。
**怎么做**：worldgen_api.cpp fillOneChunkCore 加 `WG_SPLINE_FILL=which` → density 采样绕 wrapper，直接 `spl[which]->sample(fpos)`（production 线程池）。

**数据**（conc_density_probe，同一探针/线程池，只差 wrapper）：

| | T1 | T8 | 放大比 | 占时间 |
|---|---|---|---|---|
| 全 tree（含 wrapper） | 33.54 | 346.26 | **10.32×** | 100% |
| spline-only[2]（绕 wrapper） | 3.015 | 4.895 | **1.62×** | 9% |

**结论（决定性）**：wrapper 链把 1.62× 拉到 10.32×（6.4× 放大贡献）+ 占 91% 时间 → **wrapper 链是主争用**，spline 自身几乎无争用。

**⚠️ 还要**：wg_sample_density（whole tree 单点）**无 grid 缓存（每点 buildGrid 6ms）→ std::thread 20000 点 120s 超时**（探针入口需 grid 缓存）。

---

## 7. warm vs cold（区分 buildGrid vs 顶层逐点）

**为什么做**：wrapper 链分「buildGrid 深链（每 chunk 首点一次性）」vs「顶层逐点包装（98304 点 × 虚调用）」。要区分谁主争用。

**怎么做**：fillOneChunkCore 加 `WG_WARM_GRID=1` 预建 grid（对 chunk 中心点调 finalDensity->sample 触发懒建），排除 buildGrid 深链，只剩顶层逐点包装。

**数据**：

| | T1 | T8 | 放大比 |
|---|---|---|---|
| cold（含 buildGrid） | 33.54 | 346.26 | **10.32×** |
| warm（排除 buildGrid） | 34.28 | 346.12 | **10.10×** |

**结论**：warm ≈ cold（差 0.22×）→ **buildGrid 深链无碍**。

**⚠️ 教训⑤（修正 scout）**：scout（83c9d1b0「勘探 buildGrid 链虚调用结构」）断言「buildGrid 深链=91% 主争用，顶层逐点每层浅、次要」**有误**。warm 证明 buildGrid 无碍；顶层逐点包装才是主争用。

---

## 8. scout 顶层 wrapper sample 逻辑（7e49cc07）

深挖 finalDensity 顶层：`min(squeeze(mul(0.64, InterpolatedDF#1)), noodle)`。
- a 链（terrain）= BinaryOperation(MIN) → UnaryOperation(SQUEEZE) → LinearOperation(MUL,0.64) → InterpolatedDF#1（唯一 terrain 插值）→ 其下 arg=blend_density(add(...))。
- 每点虚分派：a 链 **4 虚分派/点**（MIN、squeeze、mul、interp#1），3 层有计算。98304 点 × ≈80万-150万次。
- **纯委托层**（BlendDensityDF/WrappingDF/LazyRef）**全部在 InterpolatedDF 网格之下（buildGrid 冷路径）**→ **温暖 per-point 链零纯委托层** → 只剥纯委托对 11× 收益≈0。
- **最小改法（scout candidate）**：数据驱动化温暖 a 链 min/squeeze/mul → a 链每点 4→2 虚分派。**量级 = candidate（需实测）**。

---

## 9. WG_FLAT_TOP（数据驱动化 4→2 虚分派）— 试验 4：虚分派数（最终排除）

**为什么做**：验证 scout candidate「数据驱动化 min/squeeze/mul 降 11×」。

**怎么做**：worldgen_api.cpp 3 处 edit：
1. WorldgenHandle 加 `FlatTop` 成员（enabled/mul_c/interp/b/bmin）。
2. wg_create dynamic_cast 识别 `finalDensity == BinaryOperation(MIN,[UnaryOperation(SQUEEZE,[LinearOperation(MUL,c, interp)])], b)` → 存 flatTop（mul_c=0.64、interp、b、bmin）。
3. fillOneChunkCore 加 `WG_FLAT_TOP` 分支：`double da = applyUnary(SQUEEZE, mul_c * interp->sample(fpos)); fd = da < bmin ? da : std::min(da, b->sample(fpos));`

**逐位一致依据**（与生产 sample 同算术）：mul=`x*c`（LinearOperation L71）、squeeze=`applyUnary(SQUEEZE)`（L165 clampD(x,-1,1)/2 - clampD^3/24）、min=`da<bmin?da:min(da,b->sample)`（BinaryOperation L129）。worldgen_api.cpp include density.h（applyUnary 可见）。

**测量**：

| | T1 | T8 | 放大比 |
|---|---|---|---|
| 生产 | 33.54 | 346.26 | 10.32× |
| WG_FLAT_TOP | 34.34 | 362.41 | **10.55×** |

**✅ 对拍通过**：用 block_probe `-save`（WG_FLAT_TOP=0/1 同参照 `vanilla_8576294172403134396_6_720_-432.blocks`），`out_prod.bin` vs `out_flat.bin` **SHA256 完全一致（identical: True）** → WG_FLAT_TOP **逐位一致**（保正确）。

**结论（关键负面）**：WG_FLAT_TOP ≈ 生产（10.55× vs 10.32×，持平甚至略高）→ **减少虚分派层数（4→2）不降 11×**。scout 的「数据驱动化 min/squeeze/mul 降 11×」candidate **被证伪**。**11× 争用不是虚分派层数多导致**。

---

## 10. 排除链汇总（全部 production 模型同探针 conc_density_probe）

| 试验 | 改动位置 | 改动 | 放大比 | 结论 |
|---|---|---|---|---|
| BASE | — | — | 10.32× | 基线 |
| SERIAL | spline.locFn 存储 | locFn 存储连续化 | 10.25× | ❌ 存储非争用 |
| NOSPLIT | spline | 递归→显式栈 | 9.9× | ❌ 递归非争用 |
| DEVIRT | spline.locFn | 虚分派 devirtualize | 10.05× | ❌ locFn 虚分派非争用 |
| spline-only | 绕 wrapper | 直采 spline（WG_SPLINE_FILL） | 1.62× | spline 无碍 |
| warm | wrapper buildGrid | 预建 grid 排除 buildGrid | 10.10× | ❌ buildGrid 无碍 |
| **WG_FLAT_TOP** | 顶层 wrapper | 去 min/squeeze/mul 虚分派（4→2，逐位一致）| 10.55× | ❌ **虚分派数无碍** |

⇒ **11× 争用 = interp/noodle 采样内部**（内存访问模式），**非** 虚调用数、buildGrid、spline、min/squeeze/mul 虚分派、存储、递归。

### 剩余候选
- **内存带宽**（interp grid 数组读 + 多层缓存 + 共享噪声表读）。
- **SMT 执行争用**（8 worker 在 4 物理核/8 SMT 同核争用）。
- 待 scout（dcf85758「勘探 interp/noodle 访存机制」）定。

---

## 11. 教训汇总（五段式，防重踩）

| # | 教训 | 五段式 |
|---|---|---|
| ① | `static_cast<const DensityFunction&>(pool[i]).sample()` **= 强制虚调用** | 现象：SERIAL 10.25×≈BASE，误判「虚分派已测」；根因：转基类引用 `.sample()` 走 vtable；定位：读 sampleSerialLocFn 源码见 `static_cast<const DensityFunction&>`；修复：去掉 cast 具体类型直接调；教训：**「kind-switch + 基类引用调用」≠ 去虚分派**，要看 `.sample()` 是否经基类引用。 |
| ② | 探针 scattered 坐标失真 | 现象：spline per-sample 0.44ms（比 production 慢 1000 倍）；根因：spline locFn grid 按 chunk 懒建，scattered 跨 chunk 每采样重建；定位：conc_sample_probe 初始 scattered；修复：固定同 chunk（grid 命中）；教训：**探针必须复刻 production 访问模式（同 chunk grid 命中）**。 |
| ③ | std::thread（conc_sample_probe）≠ wg_worker pool（conc_density_probe）线程模型 | 现象：spline 1.2× vs production 10.32× 悬殊；根因：线程模型不同（std::thread 各自循环 vs production 池填 chunk）；定位：对比两探针实现；修复：WG_SPLINE_FILL 用 production 线程池；教训：**并发放大对照必须同一线程模型**，否则结论不可靠（曾误判 spline 无争用）。 |
| ④ | scout 误判「buildGrid 深链主导」（warm 证明无碍） | 现象：scout 断言 buildGrid=91% 主争用；根因：scout 只看虚调用数（buildGrid 17.6K vs 顶层 60万）但误判；定位：warm/cold 实测；修复：warm≈cold → buildGrid 无碍；教训：**虚调用数 ≠ 争用贡献**，需运行时测量（scout 静态推断可能误导）。 |
| ⑤ | wg_sample_density 单点无 grid 缓存 | 现象：std::thread 20000 点超时（每点 6ms）；根因：wg_sample_density 单点每点 buildGrid（无缓存）；定位：超时 + 查实现；修复：改用 WG_SPLINE_FILL（production 模型）/避免 std::thread 循环 whole-tree 单点；教训：**whole-tree 单点采样必须 grid 缓存，否则失真/超时**。 |
| ⑥ | WG_FLAT_TOP 必须 block_probe 对拍才可信 | 现象：WG_FLAT_TOP 10.55×≈生产，但未对拍前不可信；根因：同算术理论一致但需实证；定位：block_probe 对拍；修复：block_probe -save 对比 SHA256 identical；教训：**改生产路径后必须逐位对拍（Full）再下性能结论**。 |

---

## 12. 关键产物/文档
- `.investigations/worldgen-mt-scaling/density-latency-rootcause.md`（历史 11× 机制）
- `production-contention-scout.md`（locFn 非主导勘探）
- `locfn-serialization-ab.md`（SERIAL A/B）
- `wrapper-buildgrid-structure.md`（scout：buildGrid 结构）
- `topwrapper-sample-logic.md`（scout：顶层 wrapper sample 逻辑）
- `wrapper-chain-measurement.md`（**本次主记录**：§6 spline-only / §7 warm-cold / §8 WG_FLAT_TOP + 对拍，排除链）
- `interp-memory-access.md`（scout dcf85758 进行中：访存机制）

---

## 13. 下一步（scout dcf85758 深挖访存返回后继续）
- 基于 scout 访存结构分析：确认「内存带宽（共享读/大 grid）」vs「SMT 执行争用」谁主导 11×。
- 设计测量（如：worker 限物理核/关 SMT → 若争用降则 SMT；减小共享表读 → 若降则带宽），进一步钉死。
- 落盘结论后更新 docs 10-timewise + artifacts。

---

## 14. scout 访存机制结论（dcf85758，interp-memory-access.md 已落盘）

**确证（源码行号）**：
- interp grid **thread_local**（density.h:576-578），跨线程独立，**不共享**。
- interp#1 命中后每点读 **8 角点 double（64B）+ 3 lerp，0 虚调用**（L537-548）。grid = 5×49×5=1225 ×8B=**9800B**/实例/线程。
- noodle 内层 = **InterpolatedDF 包 range_choice 包 noise**（非 InterpolatedNoiseDF old_blended_noise——**更正任务标注 @anchor.idk**）。每点最多 **32 角点/256B** grid 读（thread_local），RangeChoice + interp#A/B/C/D。
- 跨线程共享**全为只读 const**（noiseSamplers/SplineDF 表 17KB/GRADIENTS 192B/finalDensity 节点字段），**无写共享/ping-pong**。
- 机器 **12 物理核/24 逻辑**；pool 默认 = `physicalCoreCount()`=12 物理核；**无 SetThreadAffinityMask/pinning**。**T=8 ≤ 12 物理核 → 各占独立物理核，不触发 SMT**。

**判断（推断，@anchor.idk，需 M3 钉死）——排除 带宽/SMT**：
- **C7 内存带宽**：并发 540MB/s = DDR **1-2%** → 带宽远未饱和。
- **C4/C2 SMT**：T=8 ≤ 12 物理核无 core 共享；频率归一化后 10× 远超 SMT 理论上限(~1.5×)。
- **共享读便宜**：noise 1.15×、spline-only 1.62×（都读共享 const）→ 共享读本身不是 10× 放大器。
- **最一致机制 = 长串行依赖链 + 内存子系统 latency QoS**：每点链（interp#1 grid 8 读 → noodle range_choice → interpA(8) → out_range interpB/C/D(24) → 各级数学）**每级 load 结果喂下一级**（数据依赖）；8 线程灌入长链 → 共享内存子系统排队 → **每级 load 延迟非线性膨胀** → 链延迟 ~10×。与「无锁 + 读共享 const + 真并行 + 单 chunk 膨胀 10×」自洽。**是延迟（latency）非吞吐（throughput）被共享资源排队放大**。

**⚠️ 关键区分**：这是**延迟 QoS**（latency，每级 load 排队放大），**不是**吞吐带宽饱和（C7 已否）、**不是**写乒乓（全只读）、**不是**虚调用、**不是** buildGrid/spline/存储/递归（已排除）。grid 全 thread_local + 共享读全 const + 只读无写 → 三者与「无锁+读共享+真并行+膨胀10×」自洽。

**可测量方法（scout 推荐执行序）**：
- **M3【决定性】interp-only grid-hit 隔离**：conc_sample_probe 加 interp-only 模式（预建 grid，只测 8 角点读 + 3 lerp），T=1 vs T=8。**低 → 争用不在 grid 读，在长链依赖（latency QoS H3）；高 → 在 InterpolatedDF 机制本身**。最便宜最判别。
- M1（pin 物理核）/M2（per-thread perm 副本）——大概率确认否定（与 C2/C4/C7 一致）。
- M4（MLP 提升，并行多独立点链段）——M3 显示长链主导时验证。

### 更新剩余候选（§11 修正）
非带宽、非 SMT、非共享读、非虚调用/buildGrid/spline/存储/递归 → **最一致 = 长串行依赖链 + latency QoS（H3）**。修复方向 = **提升 MLP**（打破长依赖链：并行多独立点/DFC 式全扁平直排/软件流水），不是减虚调用/存储/递归。

### 待验证（M3）
- 执行 M3（interp-only grid-hit）验证「长链 latency QoS」假说。
- 若 M3 低（争用不在 grid 读）→ 指向长链依赖 → MLP 方向（M4）。
- 若 M3 高（争用在 InterpolatedDF 机制）→ 另查 thread_local vector/cacheId 索引/allocator。

## 15. M3 interp-only 探针 — 执行遇阻（wg_sample_interp 采样慢，未干净隔离 trilinear）

**实现**（5 处编辑）：WorldgenHandle 加 `interpTop`（Dynamic_cast 捕获 a 链 InterpolatedDF#1）+ `wg_sample_interp(handle,x,y,z)`（worldgen_api.h/.cpp）+ conc_sample_probe `interp` 模式（固定同 chunk 坐标，wg_sample_interp 采样）。

**探针故障链路（详细）**：
1. **初版 interp 模式 N=20000 超时（120s）**。
2. **诊断 N=5**：每采样 **1.1s**（wall 5.5s）——interp#1->sample 极慢。
3. **根因假设**：wg_sample_interp **未设 g_curChunkX/Z**（InterpolatedDF 懒建 grid 的 buildGrid 怪物树里，FlatCacheDF/Cache2DDF 的 grid/缓存 key 依赖 g_curChunkX/Z；fillOneChunkCore 的 CurChunkGuard 会设，wg_sample_interp 不设则它们回退 pos>>4 推导，逐点/跨 y 反复重建 → 慢）。
4. **修复：wg_sample_interp 设 g_curChunkX = x>>4, g_curChunkZ = z>>4**（仿 CurChunkGuard，RAII 恢复）→ N=5 per-sample **5.9ms（快 187 倍）**，含 interp#1 buildGrid（怪物树建 grid ≈ 25ms，production density 的大头）。
5. **N=20000**：per-sample **292μs**（wall 5847ms）——**仍比 production 0.34μs/点慢 850×**。

**结论 / 遇阻**：wg_sample_interp 未干净隔离到「grid 命中 trilinear」——per-sample 292μs 远高于预期的 trilinear（<1μs），可能是每次采样重建 grid 或 buildGrid 摊薄不足。**M3 探针未能干净测「interp#1 grid 命中」的并发放大**，latency QoS 假说**未直接验证**（需修探针或另法）。

**已有数据（间接指向 latency QoS）**：
- warm（production 预建 grid，去 buildGrid）10.10× → buildGrid 无碍
- spline-only 1.62×（绕 wrapper+interp+spline）→ 绕全部后低
- WG_FLAT_TOP（去 min/squeeze/mul）10.55× → 虚分派数无碍
- → 争用集中在 **interp#1 trilinear + noodle 长链**（非 buildGrid/spline/虚分派），与 scout 的「长串行依赖链 + latency QoS」一致（但未经 M3 直接证实）。

**待定**：修 wg_sample_interp（诊断为何 292μs/每次重建）or 接受 warm+排除链推断（latency QoS）。

### M3 探针诊断进展（更新）
- **N=1**：wall 27.9ms → interp#1->sample 单次 = **buildGrid 怪物树 ≈27.9ms**（production density 大头）。
- **N=20000**：per-sample 292μs = (27.9ms + 19999×hit)/20000 → **hit ≈ 291μs/采样**。
- **矛盾（探针 bug 铁证）**：production 33ms/chunk 含 98304 点（interp#1 hit + noodle + min/squeeze/mul）→ 每点仅 **0.34μs**；wg_sample_interp 的 hit（291μs）**比 production 慢 850×**。同 chunk 的 interp#1 trilinear（8 角点 grid 读 + 3 lerp）不可能 291μs。
- **结论**：wg_sample_interp 命中慢 850× 是**探针自身 bug**（非 11× 机制）。候选根因：① thread_local slots 每采样 resize/allocator 行为；② 坐标覆盖 256 个不同 (x,z) cell 的 cache 局部性；③ g_curChunk 设置引入的额外路径。**需 perf 分析钉死**（探针调试，非 11× 机制）。

> **教训**：interp#1->sample 单点即触发 buildGrid（怪物树 27.9ms）——探针测「hit」必须先预建 grid；且 wg_sample_interp 的 hit 慢 850× vs production，探针自身需 perf 调试（thread_local slots/坐标/allocator）。
