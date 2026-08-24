# 草稿 · mt-scaling-errors.md 追加 —— 11× 争用定位误判与新错误（本轮新增 MT12-MT17）

> **用途**：追加到 `.investigations/worldgen-mt-scaling/mt-scaling-errors.md`（错误台账本体）末尾（主会话应用）。
> **编号**：接续既有 MT1-MT11，本轮新增 **MT12-MT17**（对应任务标注 ①-⑥）。
> **格式**：五段式（现象/根因/定位/修复/教训），根因=机制层面「为什么错」，禁止只记「已修复」。
> 主过程日志：`.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`。

---

## MT12. 🔥 SERIAL 的 `static_cast<const DensityFunction&>(pool[i]).sample()` = 强制虚调用（SERIAL 从未去虚分派）——A/B 只证「存储非争用」，误以为证了「虚分派非争用」（🔍 已纠正）

### 现象
- SERIAL A/B 结果：BASE 10.03× / SERIAL 10.25×（放大比持平）。
- 初读该结果时（叙述层面）把「SERIAL 」当成「已验证 locFn 存储 + 虚分派综合非争用」——但后续 DEVIRT 单独去虚分派（10.05×）几乎无变化，才暴露 SERIAL 自身**根本没去虚分派**。

### 根因（机制层面）
- `sampleSerialLocFn`（density.h）的 kind-switch 分支写的是：
  ```cpp
  case FLAT_CACHE: return static_cast<const DensityFunction&>(flatCachePool[r.index]).sample(pos);
  ```
- `static_cast<const DensityFunction&>(obj)` 把**具体类型的实体临时转成基类引用**，再调 `.sample(pos)` —— `sample` 是**虚函数**，经基类引用调用**必然走 vtable 分派**。
- 所以 SERIAL 只去掉了两样东西：**shared_ptr deref**（不再间接访问）+ **存储连续化**（池实体连续）。**虚分派从未被去掉**（kind-switch 后一视同仁转回基类引用）。
- 结论链条误读：SERIAL 只隔离了「存储布局（A 类）」的贡献，**没有**隔离「虚分派」的贡献。A/B 只能证明「存储非争用」；「虚分派非争用」是让 DEVIRT（真正去掉基类引用 cast）单独证的。

### 定位
- 读 `density.h` `sampleSerialLocFn` 源码 → 见 `static_cast<const DensityFunction&>` 三处（FLAT_CACHE/CACHE_2D/BINOP）→ 确认转基类引用调用虚函数。
- 交叉印证：DEVIRT 改法（去掉 cast、具体类型直接 `.sample()`）放大比 10.05× ≈ BASE 10.32× → 说明虚分派本来就不是 11× 主导 → 反推 SERIAL 的「虚分派未被改动」成立。

### 修复
- 下一步 DEVIRT 修改：把 `sampleSerialLocFn` 三个 case 的 `static_cast<const DensityFunction&>(pool[i]).sample()` 改为**具体类型直接调 `.sample()`**（by-value 池实体，语义上可 devirtualize，O2）。env `WG_SERIAL_LOCFN=1`（DEVIRT）。
- 语义：去掉转基类引用 → 编译器能确定静态类型 → 去 vtable 跳转。

### 教训（判错经验）
1. **「kind-switch + 基类引用调用」≠ 去虚分派**——判断「是否已去虚分派」要盯**`.sample()` 是否经基类引用发起**（`static_cast<const DensityFunction&>` 即虚调用），而不是看有没有 kind-switch。
2. **A/B 隔离变量要精确到「机制维度」**——「存储布局」与「虚分派」是两类不同的代价；一次 A/B 只能隔离它真正改动的那一类。若 A/B 同时保留了另一个候选（虚调用），其结论不得外推为「那个候选也被测过」。
3. 复用判错：看到 `static_cast<const Base&>(derived_obj).virtualMethod()` 模式，默认它仍是虚调用。

---

## MT13. 🔥 conc_sample_probe scattered 坐标失真——spline 探针 per-sample 0.44ms（比 production 慢 1000 倍），grid 重建主导（✅ 已修正）

### 现象
- conc_sample_probe spline 模式初版：per-sample = **440552ns（0.44ms）**——比 production 的 spline 单次（μs 级）慢约 **1000 倍**，完全失真。
- 修正后（固定同 chunk）：per-sample **4493.5ns**（快 98×），spline 并发放大 1.22×/1.21×。

### 根因（机制层面）
- spline 的 locFn（ContinentsDF 等 = FlatCacheDF）grid **按 chunk 懒建**（FlatCacheDF/Cache2DDF 的 grid/缓存 key 依赖 `g_curChunkX/Z`）。
- scattered 坐标 `x=3200+(i*17)%2048` → **跨越 128 个不同的 chunk** → 每个采样点落在不同 chunk → **每换一个 chunk 就触发一次完整 buildGrid（重建 25 点 grid）**。
- 结果是探针成本被 **grid 重建**主导（每采样一次重建），而非生产路径（同 chunk grid 命中 + 只读）。这不反映生产行为。

### 定位
- 对比初版 scattered per-sample 0.44ms 与修正后固定同 chunk per-sample 4493.5ns（快 98×）→ 差异来自「是否跨 chunk 重建 grid」。
- 对照 production `fillOneChunkCore`：是「同 chunk grid 命中」访问模式（fillOneChunkCore 处理单 chunk，所有采样在同一 chunk 坐标域，grid 命中）。→ 探针必须复刻这一访问模式。

### 修复
- 改 conc_sample_probe 固定 x,z 同 chunk（3200-3215 / 3224-3239）、y 扫 → grid 命中 → per-sample 4493.5ns（可靠）。

### 教训（判错经验）
1. **探针必须复刻 production 的访问模式**（同 chunk grid 命中），否则测的是「探针自己的失真路径」而非生产路径。
2. **探针初值要先用「合理性检查」**：per-sample 比 production 慢 1000 倍本身就说明有系统性失真（要么探针 bug，要么访问模式错），应先排查再下结论——不要直接拿失真数据做排除链依据。
3. 复用判错：凡探针里按坐标懒建缓存的组件（grid/FlatCache/Cache2D），scattered 坐标必触发重建，须固定同 chunk / 复刻生产 chunk 域。

---

## MT14. 🔥 conc_sample_probe(std::thread) ≠ conc_density_probe(wg_worker pool) 线程模型混淆——spline 1.2× 不能独立证明「spline 在 production 下无争用」（⚠️ 已纠正，spline 1.2× 降为辅证）

### 现象
- conc_sample_probe spline 模式（std::thread）测 spline 并发放大 **1.2×**（接近无争用）。
- production 全 tree 并发放大 **10.32×**。
- 两者悬殊 → 一度不严谨地倾向「spline 在 production 下也无争用」。

### 根因（机制层面）
- **线程模型不同**：conc_sample_probe 用 **std::thread**（每线程独立循环采样）；production 争用（10.32×）发生在 **wg_worker pool**（wg_fill_blocks_multi 填 chunk，CoreSwapPool 队列 + worker）。
- std::thread 各自独立循环 → 每线程跑自己的数据，**不存在 pool 的任务调度 + 共享队列 + 线程间交互** → std::thread 下多入口都低放大（noise 1.15× / spline 1.2×）。
- 所以 spline 的 1.2× **无法排除「std::thread 模型本身无争用」的伪影**——这可能是「std::thread 下测什么都低」，而不是「spline 生产无争用」。

### 定位
- 对比 conc_sample_probe（std::thread 实现）与 conc_density_probe（wg_fill_blocks_multi 填 chunk 实现）→ 确认两者线程模型不同。
- 交叉证据：全部「低放大」入口（noise 1.15×/spline 1.2×）都来自 std::thread 探针；全部「高放大」（10.32×）来自 production 池 → std::thread 探针自身不放大，问题在模型不一致。

### 修复
- 设计 WG_SPLINE_FILL（fillOneChunkCore 加 env，density 采样绕 wrapper 直接 `spl[which]->sample(fpos)`），用 **production 线程池**（wg_fill_blocks_multi + conc_density_probe）测 spline 绕 wrapper。
- 结果：spline-only[2] 1.62×（production 池）→ 确认 spline 在 production 下也几乎无争用（1.62× vs 全 tree 10.32×）。

### 教训（判错经验）
1. **并发放大对照必须同一线程模型**（生产线程池 vs std::thread 是不同的并发形态）；跨模型对比不可靠。
2. **不要用「std::thread 探针的低放大」去反推「production 池里的低放大」**——两种模型的争用结构不同（std::thread 独立循环无池调度/共享队列压力）。
3. 复用判错：任何「并发放大/无争用」结论，先确认它是在 production 同一线程模型（wg_fill_blocks_multi + CoreSwapPool）下测的，还是在独立的 std::thread 微基准下测的（后者仅作辅证）。

---

## MT15. scout 静态误判「buildGrid 深链=91% 主争用」——warm 实测推翻（虚调用数 ≠ 争用贡献）（✅ 已纠正）

### 现象
- scout（wrapper-buildgrid-structure.md / 83c9d1b0）断言：buildGrid 深链（interp#1 每点重走 18-20 层实虚分派 + spline 递归）= 91% 走 wrapper 链的时间，是 11× 主争用。
- 但 warm 实测：预建 grid（排除 buildGrid 深链）后仍 **10.10×**（vs cold 10.32×，差 0.22×）→ buildGrid 深链对 11× 争用贡献 **微乎其微（<2%）**。

### 根因（机制层面）
- scout 用的是**静态虚调用次数**推导争用占比：buildGrid 虚调用深（每点 18-20 层）→ 它「看起来」是大头（17.6K/chunk、含深链下探）。
- 但**虚调用次数 ≠ 争用贡献**——争用（latency QoS / 延迟排队）本质是**并发下访存排队的放大**，与「单次调用长不长」不直接对应。buildGrid 是**每 chunk 冷路径一次性**（每 chunk 触发 1 次/实例），8 线程各触发自己的 buildGrid 不互相排队摊薄（warm 去它后几乎没有变化）；而**顶层逐点包装**是**每 chunk 98304 点 × 每点（warm 后仍在）**，是真正的 per-point 并发放大面。
- 静态看「buildGrid 深」≠ 动态争用大；scout 忽略「冷路径一次性 vs 温暖 per-point 重复」的并发放大差异。

### 定位
- warm/cold 实测（production 模型 conc_density_probe）：cold 10.32× vs warm 10.10×（差 0.22×）→ buildGrid 对 11× 贡献 <2%。
- 对照 scout 静态断言（buildGrid=91%）→ 静态推断被运行时测量推翻。

### 修复
- 无代码修复（这是诊断判断修正）。
- 结论修正：11× 主争用 = **顶层逐点包装**（min/squeeze/mul/interp 每点 98304× 重复）+ 后续收窄到 interp/noodle 采样内部；buildGrid（冷路径）无碍。

### 教训（判错经验）
1. **静态「虚调用数/深链」推断不能代替运行时争用测量**——争用是并发访存排队现象，与「单次调用深不深」不是一回事；冷路径一次性 vs 温暖 per-point 重复的并发放大差异必须用实测区分。
2. **判别「争用在哪」要用「单一变量剔除」**（warm 排除 buildGrid / WG_FLAT_TOP 排除顶层包装），不能靠静态结构数推断占比。
3. 复用判错：凡「xxx 深/虚调用多 → 必是争用大头」的静态断言，都要用「剔除该项的 A/B」验证；剔除后放大比不变即该项非争用。

---

## MT16. wg_sample_density 单点无 grid 缓存——std::thread 20000 点超时（每点 buildGrid 6ms）（🔍 已记录，探针入口需 grid 缓存）

### 现象
- wg_sample_density（whole tree 单点）用 std::thread 循环 20000 点 → 120s 超时。
- 原因：每点触发一次整树 buildGrid ≈ 6ms → 20000 点 ≈ 120s。

### 根因（机制层面）
- `wg_sample_density` 单点采样走整棵 finalDensity 树，每点 `finalDensity->sample(pos)`；InterpolatedDF 首次采样触发 `buildGrid`（懒建 5×49×5=1225 点 grid，每点 arg 下探深链，怪物树 ≈ 27.9ms/次建）。
- **窗口/入口无 grid 缓存**：每次调用都是「新 chunk、首访」→ 每点重建 grid（无 thread_local grid 命中复用）。
- 与 production `fillOneChunkCore` 不同：后者对单 chunk 处理所有 98304 点，grid 只建一次（首点），后续点命中复用 → 摊薄到 0.4μs/点。单点入口无此摊薄。

### 定位
- 超时（120s cap）+ 反推单点 6ms（20000 点 × 6ms ≈ 120s）。
- 对照 production fillOneChunkCore（同 chunk grid 命中复用）→ 确认走「whole tree 单点无 grid 缓存」路径。

### 修复
- 改用 WG_SPLINE_FILL（production 模型，绕 wrapper 只测 spline）做严格对照；避免「std::thread 循环 whole-tree 单点」。
- 探针入口设计：采样 whole tree 必须**先预建 grid / 固定 chunk / 同 chunk grid 命中**，否则失真或超时。

### 教训（判错经验）
1. **whole-tree 单点采样必须 grid 缓存**（固定 chunk + 预建 grid），否则每点重建 grid → 失真/超时。
2. **探针入口要复刻 production 的 chunk 内 grid 命中复用**，不能用「每点独立 whole-tree 单点」——那会触发 buildGrid 深链（冷路径），测的是 buildGrid 不是 warm per-point 争用。
3. 复用判错：凡含 InterpolatedDF/FlatCacheDF/Cache2DDF 懒建缓存的探针，单点采样必建 grid；multi-point 同 chunk 才命中。

---

## MT17. 改生产路径（WG_FLAT_TOP）后没先 block_probe 对拍就下性能结论——须逐位一致才可信（✅ 本项已执行对拍，此为纪律沉淀）

### 现象
- WG_FLAT_TOP 性能结论（10.55× ≈ 生产 10.32×，「减少虚分派不降 11×」）若**不先逐位对拍**，会建立在「同算术理论上一致」的推断上，无法排除 WG_FLAT_TOP 因改错算术而失效的假象。
- 本项正确流程已执行：block_probe `-save`（WG_FLAT_TOP=0/1 同参照 `vanilla_8576294172403134396_6_720_-432.blocks`），`out_prod.bin` vs `out_flat.bin` **SHA256 完全一致（identical: True）** → WG_FLAT_TOP 逐位一致。

### 根因（机制层面）
- WG_FLAT_TOP 是**改写生产采样路径**（把 min/squeeze/mul 扁平化为内联算术），若算术/边界/顺序有一处不同 → 采样值错误 → 性能对比建立在**错误代码**上，结论不可信。
- 理论上「同算术」可由源码推演，但**浮点顺序/rounding/clamp 边界**（squeeze 的 clampD、min 分支 `da<bmin`）无法仅凭静态推演保证逐位一致——必须实证对拍（Full = block_probe 逐位）。
- 性能结论若建立在未对拍的改动上，可能得出「改动 A 无效」的实际是「改动 A 改错了」。

### 定位
- 性能对比前先做正确性对拍：block_probe -save 导出对照 → SHA256 逐位比较。
- 本项：`out_prod.bin` vs `out_flat.bin` SHA256 identical → 确认一致性后才记录 「WG_FLAT_TOP 逐位一致 + 10.55× ≈ 生产」结论。

### 修复
- 对 WG_FLAT_TOP 执行 block_probe 对拍（SHA256 identical）→ 确认逐位一致 → 性能结论可信。
- 教训沉淀为「改生产路径后必须 block_probe 对拍（Full）再下性能结论」的固定纪律。

### 教训（判错经验）
1. **改生产路径后的性能结论必须先过正确性门（Full = block_probe 逐位对拍）**——性能对比不能建立在未验证正确性的代码上（否则「改动无效」可能是「改动改错了」）。
2. **理论推演（同算术）≠ 逐位一致**——浮点顺序/rounding/clamp 边界需实证；用 block_probe -save + SHA256 identical 做硬验证。
3. 复用判错：凡性能 A/B 改动了采样/求值路径，先对拍正确性（SHA256 逐位）再读性能；不改采样路径的纯调度/存储实验（M1 pin 核/serial 池）可不全对拍，但改动求值语义的必须对拍。

---

## 附：本轮新增错误 → 根因 速查表（追加到既有速查表尾部）

| 错误 | 一句话根因 | 状态 |
|---|---|---|
| SERIAL 与 BASE 放大比持平（10.25× vs 10.03×），误以为「虚分派已测」 | `static_cast<const DensityFunction&>(pool[i]).sample()` = 转基类引用调虚函数（**强制虚调用**）；SERIAL 只去 shared_ptr deref + 存储连续化，**从未去虚分派** → A/B 只证「存储非争用」 | 🔍 已纠正（DEVIRT 单证虚分派非争用，MT12） |
| conc_sample_probe spline per-sample 0.44ms（比 production 慢 1000 倍） | spline locFn grid 按 chunk 懒建，scattered 坐标跨 128 chunk → 每换 chunk 重建 grid → grid 重建主导（非生产同 chunk 命中路径） | ✅ 已修正（固定同 chunk，MT13） |
| spline 1.2×（std::thread）vs production 10.32× 悬殊，一度误断「spline 无争用」 | conc_sample_probe 用 std::thread（独立循环）、conc_density_probe 用 wg_worker pool（填 chunk）——**线程模型不同**；std::thread 下多入口都低放大（noise 1.15×/spline 1.2×） | ⚠️ 已纠正（spline 1.2× 降为辅证；WG_SPLINE_FILL 用 production 池测，MT14） |
| scout 静态断言「buildGrid 深链=91% 主争用」 | 静态虚调用数（buildGrid 深 18-20 层）推导占比，但**虚调用数 ≠ 争用贡献**；buildGrid 是冷路径每 chunk 一次性（warm 剔除后近乎无变化），顶层逐点（98304 点/次）才是 per-point 并发放大面 | ✅ 已纠正（warm 10.10× ≈ cold 10.32×，MT15） |
| wg_sample_density std::thread 20000 点 120s 超时 | 单点 whole-tree 采样无 grid 缓存（每点触发 buildGrid ≈6ms）→ 20000 点 ≈120s；与 production 同 chunk grid 命中复用（0.4μs/点）不同 | 🔍 已记录（探针入口需 grid 缓存，MT16） |
| WG_FLAT_TOP 性能结论可信度 | 改生产采样路径（min/squeeze/mul 扁平化），未对拍前基于「同算术理论一致」不可信；须 block_probe 逐位（SHA256 identical）实证 | ✅ 已对拍（out_prod vs out_flat SHA256 identical，MT17） |

> **主会话应用注意**：①-⑥ 分别编号 MT12-MT17 追加到 `mt-scaling-errors.md` 末尾 + 全局速查表各加一行（见上）。编号不覆盖既有 MT1-MT11。
