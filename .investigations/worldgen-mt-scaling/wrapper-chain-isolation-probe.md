# ② wrapper 链虚调用隔离探针 — 最小实验（worker 交付，不编译）

> 角色：验证 worker（只改 worldgen 源码最小实验，不编译——主会话 build.ps1 编译）。
> 课题：production 并发 11×（T=1 35.95ms → T=8 373.27ms，放大比 10.38×）。**剩余候选 = ② DF wrapper 链虚调用 + 寻址**。
> **状态：代码已交付（draft，未编译验证——主会话编译 + 运行后升级 candidate）。** 锚点扫描 invalid=0（本次改动未增删 @anchor）。

---

## 1. 修正方向（Deliverable 1 — 结论）

### 1.1 排除链（已确证，主会话直接采信）

| # | 候选 | 排除证据 | 放大比 | 判定 |
|---|---|---|---|---|
| 噪声计算 | shared perm 表读 | conc_sample_probe noise 模式 | 1.07-1.15× | **非争用**（共享 perm 广播几乎无争用） |
| ① locFn 存储连续化/去 deref | 散布堆指针追逐（scout A） | SERIAL（`WG_SERIAL_LOCFN`，按类型连续池 + kind 索引，去 shared_ptr deref） | 10.25× vs BASE 10.03× | **持平**（存储/去 deref 无效） |
| 递归 | sampleNode 递归 → 显式栈 | NOSPLIT（`WG_DFC_NOSPLIT`，去递归，保留虚调用） | 9.9× vs BASE 10.38× | **持平**（递归形状无效） |
| ① locFn 虚调用本身 | spline 每非叶节点 1 次 `->sample` | **未直接单独测**（见 1.2 修正） | — | 待定（但预期非主导，见 1.3） |
| **② wrapper 链** | min/squeeze/interpolated/blend_density/add/mul …… → spline（15-20 层） | **本探针隔离** | 待测 | **剩余候选** |

### 1.2 ⚠️ 重要修正（纠正任务前提的一个不精确点）

任务前提写道「SERIAL 已『去虚分派』仍 10.25× → ① locFn 虚调用无效」。**这不准确**：

- SERIAL 的 `sampleSerialLocFn`（density.h L971-984）是 `switch(kind){ case FLAT_CACHE: return static_cast<const DensityFunction&>(flatCachePool[i]).sample(pos); ... }`。
- 这里的 kind-switch **只决定「从哪个连续池取索引」**，随后**仍经基类引用调 `.sample(pos)` = 仍是虚调用**（`DensityFunction&` 引用 → vtable 分派）。
- **所以 SERIAL 只证明了「locFn 去 deref + 存储连续化」（scout 候选 A）无效，并未证明「去掉 locFn 的虚分派」无效。** ①（spline locFn 虚调用）在技术上**仍未被 SERIAL 单独测过**。

### 1.3 但①大概率非主导（独立于 SERIAL 的论证）

- spline locFn 虚调用 = **每非叶节点 1 次**（坐标函数求值），是节点的一次性常数开销；spline 中**占成本主导的是 locFn 内部噪声计算**（`continents` FlatCacheDF → shift_noise → 噪声），而非那个 `->sample` 虚分派。
- 噪声本身并发放大仅 1.07-1.15× → coord 的噪声成本无争用 → 包住它的那个单次 locFn 虚调用也不可能是争用主体。
- **结论（candidate）**：① 大概率非主导；真正剩余 = ② wrapper 链 + spline 自身依赖 load 链。

### 1.4 剩余候选 ② + 决定的判据

**② 是「finalDensity 顶部那段 wrapper 链」**：`min(squeeze(mul(0.64, interpolated(blend_density(add(0.117, mul(y_gradient, add(...))))))), noodle)` —— 从 `overworld.json` noise_router.final_density 顶到「真正的地形 spline」之间那 6-10 层纯委托 DF（min/squeeze/mul/interpolated/blend_density/add/y_clamped_gradient/range_choice）。每层一次 virtual `sample` 转发 + shared_ptr 跳转，串行依赖 load 链叠加。

**决定性对照（本探针）**：
- `wg_sample_density`（whole finalDensity，含 wrapper 链 + spline）：已知 8.4×。
- `wg_sample_spline[which]`（**直接采样单个 SplineDF，绕过顶部 wrapper 链**，保 spline + 其 locFn + 递归）：**待测**。

**判据**：
- 若 spline 单独放大 ≈ 8×（与 whole 持平）→ **wrapper 链不是争用贡献**，争用全在 spline 自身依赖 load 链 → **剥离 wrapper 无益**，不做 wrapper 扁平化。
- 若 spline 单独放大 **<< 8×**（如 2-3×）→ **wrapper 链是争用大头** → 值得做「wrapper 扁平/连续化」（仅去顶部委托层，不动 spline 求值）——这比 DFC（重写算法 + 重算 split，600× 慢）轻得多。

> **注意（不要重复 DFC 教训）**：先跑完本探针钉死「② 是否主导」，**再**决定是否投入 wrapper 扁平化。未测前不立项。

---

## 2. 探针设计（Deliverable 2 — 实现）

**目标**：直接采样单个 SplineDF（绕过 top wrapper 链），测 T=1/T=8 并发放大比，与 whole finalDensity 对照。

**改动（全部加法，production 行为零影响；env 无关，默认路径不变）**：

| 文件 | 改动 | 为什么 |
|---|---|---|
| `density_builder.h` L189 | `buildSpline` 里 `splines.push_back(spline)`；新增 `splines` 成员 + `getSplines()`/`splineCount()` | 捕获全部 SplineDF 实例（含 registry 文件 + finalDensity），供探针直接采样 |
| `worldgen_api.h` | 新增 `wg_spline_count` / `wg_spline_nodes` / `wg_sample_spline` | 暴露粗粒度 C API（探针无需直接依赖密度类） |
| `worldgen_api.cpp` | 实现上述 3 个函数（在 `wg_sample_density` 之后） | 采样第 `which` 个 SplineDF |
| `conc_sample_probe.cpp` | 新增 `spline` 模式（`mode=spline [N] [which]`），并打印 SplineDF 清单 | 复用现有并发放大测量框架 |
| `build.ps1` L45 | `$exes` 加 `"conc_sample_probe"` | 让主会话能编译该探针（此前不在 exes 列表） |

**为什么用 `finalDensity` 树内的 SplineDF，而非独立命名函数**：overworld 的地形 spline 被埋在各 registry 函数/嵌套 JSON 里（factor/depth/... 都是 `flat_cache`/`add` 包着 spline），**没有独立可采样的「纯 spline」命名入口**。在 `buildSpline` 处捕获所有 SplineDF 实例是最干净、唯一完整的获取方式。

**采样语义**：`wg_sample_spline[which]` 采样一个 SplineDF 树（含其自身 locFn 噪声 + Hermite 递归），**不含** finalDensity 顶部的 min/interpolated/blend/mul 委托层。FlatCacheDF locFn 在无 chunk ctx 时回退 pos 推导 key（同 `wg_sample_density` 路径，一致、正确）。

---

## 3. 静态自检

| 检查点 | 结果 |
|---|---|
| `SplineDF` 完整类型可见 | `density_builder.h` include `density.h`；`shared_ptr<SplineDF>` + `nodesSize()` 全 public | ✅ |
| `builder` 成员可达 | `WorldgenHandle.builder` 是 public `unique_ptr<DensityBuilder>`（worldgen_api.cpp L182） | ✅ |
| 新增 API 声明/定义 配对 | `worldgen_api.h` 3 个函数声明 ↔ `worldgen_api.cpp` 3 个定义，签名一致 | ✅ |
| 成员函数体引用后声明成员 | `getSplines()/splineCount()` 引用 private `splines`（L349）；C++ 类成员函数体可引用后声明成员（完整类作用域） | ✅ |
| 探针依赖 | `conc_sample_probe.cpp` 仅用 `wg_*` 粗粒度 API，不依赖密度类内部 | ✅ |
| production 行为 | 全部加法；未改任何现有采样/构建路径；env 无关 | ✅ |
| anchor | 未增删 `@anchor`；`scan_cpp_anchors.py` invalid=0（已实测） | ✅ |
| 并发/生命周期 | `splines` 持 `shared_ptr`，防悬垂；探针单线程构建后只读采样 | ✅ |
| 类型宽度 | `which`/`nodes` 用 int；无 long 位宽风险 | ✅ |

**已知未验证（交主会话编译确认）**：
- `SplineDF` 在 `wg_sample_spline` 处是否 `sample` 可访问（public override，预期 ✅）。
- `getSplines()` 返回 const 引用在 C++17 下编译（预期 ✅）。

---

## 4. 测量方法（主会话）

### 4.1 编译

```pwsh
pwsh versions/1.20.1/cpp/build.ps1 -Target conc_sample_probe
```

（`conc_sample_probe` 已加入 build.ps1 `$exes`；`worldgen_core.lib` 会因 worldgen_api.cpp/density_builder.h 改动重建。）

### 4.2 先看 SplineDF 清单（选 which）

```pwsh
& versions/1.20.1/cpp/build-msvc/bin/conc_sample_probe.exe 8576294172403134396 versions/1.20.1/data/worldgen 1 spline 1 0
```
- 输出 `[SPLINES] [i] nodes=N` 列表。**选 nodes 最大的那个**（预期 = factor 的地形 spline，coordinate=continents，含 erosion/ridges/ridges_folded 嵌套节点，~135+ 节点树）作为 `which`。

### 4.3 跑测量（T=1 vs T=8，每模式 4 组；全程禁 WG_PROFILE/WG_STAGETIMER）

```pwsh
# whole finalDensity（含 wrapper 链）
$env:WG_PHASETICK=1; conc_sample_probe.exe <seed> <dir> 1 density 20000
$env:WG_PHASETICK=1; conc_sample_probe.exe <seed> <dir> 8 density 20000
# 纯 spline（绕过 wrapper 链）
$env:WG_PHASETICK=1; conc_sample_probe.exe <seed> <dir> 1 spline 20000 <which>
$env:WG_PHASETICK=1; conc_sample_probe.exe <seed> <dir> 8 spline 20000 <which>
```

### 4.4 计算

- 每 run 的 `per-sample=...ns`（壁钟/N）为吞吐均值（每样本成本）。
- `并发放大比 = per-sample(T=8) / per-sample(T=1)`。
- 对照：density vs spline 两个放大比。

### 4.5 判据

见 §1.4。核心：**spline 放大比 vs density 放大比**。
- 持平（±<20%）→ ② 非主导，剥 wrapper 无益 → 转向 spline 自身依赖链优化（或结案：11× 是 DF 树多层依赖 load 的固有代价，唯有 DFC 式全扁平可解但 CPU 不可行）。
- spline 显著低 → ② 主导 → 立项 wrapper 扁平/连续化（轻量，非 DFC）。

---

## 5. 风险 / 边界

1. **spline 选择偏差**：不同 SplineDF 大小/成本差异大。选错 which 会误导。**必先看 `[SPLINES] nodes` 清单选最大者**；若不确定，多试几个 which 看放大比是否稳健。
2. **`finalDensity` 会到多个 spline + cave 噪声**，spline 模式只到一个 spline —— 不是严格同集合。但 8 线程下「纯 spline 是否仍 ~8×」这一信号足以判定「wrapper 链是否必要」。
3. **未编译**：全部改动未编译，主会话 build.ps1 编译 + 一轮 `WG_PHASETICK` 冒烟（density 模式应与改动前 ~一致，验证零退化）后再进测量。
4. **此探针只测争用（并发放大），不测正确性**。`spline` 模式值是「纯 spline 采样值」，与 block_probe 无对拍义务（诊断专用，非生产路径）。

---

## 6. 一句话总结

**方向修正**：① locFn 虚调用大概率非主导（SERIAL 只证了「去 deref/连续化」无效，未真测「去虚分派」；且 locFn 虚调用只是一次性常数，噪声本身无争用 1.07×）。剩余候选 = **② wrapper 链 + spline 依赖 load 链**。已交付 `wg_sample_spline` 探针，直接采样单个 SplineDF（绕过 wrapper 链）→ T=1/T=8 对照，**先钉死 ② 是否主导再决定是否立项 wrapper 扁平化**（避免 DFC 重复教训）。

> **confidence: draft**（代码未编译）。主会话编译 + 跑 §4 测量后升级 candidate；`confirmed` 由用户拍板。
