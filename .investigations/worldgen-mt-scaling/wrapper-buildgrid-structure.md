# finalDensity buildGrid wrapper 链结构 — scout 静态勘探

> 角色：scout（只读勘探，不改码不编译）。产物 `.investigations/`，供主会话定位可数据驱动化的 buildGrid 虚调用点。
> 前置（采信，未重验）：production 并发 11× = wrapper 链（InterpolatedDF/buildGrid 链虚调用 + 寻址），非 spline；spline-only（WG_SPLINE_FILL）1.62×，wrapper 链占 density 91%（33.54ms vs 3.015ms）。
> **标注**：确证 = 静态读源码/JSON 可定；推断 = 需运行时测量 -> 以 `@anchor.idk` 标注。

---

## 0. TL;DR（5 问答案）

| # | 问题 | 答案 |
|---|---|---|
| 1 | finalDensity 树拓扑 | 5 层 **InterpolatedDF**（1 主 terrain + 4 noodle 嵌套），深度约 21 层（从 root 到 base_3d_noise 叶）。wrapper 层 = min/squeeze/mul/LinearOp/blend_density/add/range_choice/ycg/clamp/abs/max/cube/square... |
| 2 | buildGrid 虚调用链深度与点 | 每个 InterpolatedDF::buildGrid 建 **5×49×5 = 1225** 网格点（左邻 edge-reuse 时实采样 **980**）；每点调 `arg->sample`（L607）1 次。**每 chunk 总 buildGrid arg->sample 虚调用 = 5×1225 ≈ 6125**（reuse 后 5×980 ≈ 4900）。 |
| 3 | wrapper 额外虚调用 | 是。wrapper 链**不是**纯委托：min/squeeze/mul/add/range_choice 的 sample 内部**再调子项**（BinaryOperation 调 a->sample+b->sample；RangeChoice 调 input + branch；Unary 调 input）。每个 grid 点的 arg->sample 平均下探 **~15-20 层**虚分派才到叶（spline/noise/flatcache）。 |
| 4 | 可数据驱动化点 | **主争用 = InterpolatedDF#1 的 `arg->sample`（blend_density→add→mul→...→sloped_cheese→spline/noise 深链）逐网格点重复遍历**。纯委托可扁平层 = **blend_density / cache_once(WrappingDF) / LazyRef**（无内部计算，只转发，删一层虚调用零成本）；LinearOperation（纯 ±c 或 ×c）仅次于纯委托。
| 5 | 总虚调用估算 | **buildGrid 虚调用（每 chunk 一次性）= 每 InterpolatedDF 1 次 buildGrid × 每点 1 次 arg->sample**：interp#1 的 grid 点每个都下探深链（~18 虚分派 + spline 递归）→ **~4900×深链 = 主争用（91% 去向）**；grid 命中后的每点 trilinear 无虚调用（InterpolatedDF.sample 纯查表），但**顶层 min/squeeze/mul/noodle-RC 仍是逐点虚调用**（98304 点 × ~6-12 次）。**11× 争用 = buildGrid 深链遍历（interp#1 的 980 个网格点每点重走整棵 terrain 深链），不是 grid 命中后的 trilinear。** |

---

## 1. finalDensity 树拓扑（从 noise_router.final_density 出发）

### 1.1 顶层结构（overworld.json L30-168）

`noise_router.final_density` 顶层是 **`minecraft:min`**，双输入：

```
final_density = min(
    arg1 = squeeze(                          # UnaryOperation(SQUEEZE)
        mul(0.64,                            # BinaryOperation::create(MUL, Constant, X) → 折叠为 LinearOperation(c=0.64)
            interpolated(                    # ★ InterpolatedDF #1 —— 唯一主 terrain 插值
                blend_density(               # BlendDensityDF → 纯委托
                    add(0.1171875, ...)      # BinaryOperation::create(ADD, Constant, X) → LinearOperation
                )
            )
        )
    ),
    arg2 = "minecraft:overworld/caves/noodle"   # String ref → 解析成 noodle 树（含 4 个 InterpolatedDF）
)
```

**关键点**：`arg1`（squeeze 链）是**唯一的** InterpolatedDF#1，它的 `arg` = blend_density → 深 terrain 链。`arg2`（noodle）是**独立子树**，从 `caves/noodle.json` 单独解析，其内部有 4 个 InterpolatedDF（#2-#5）。

### 1.2 InterpolatedDF 实例计数（整棵 finalDensity 树）

确证：整棵 finalDensity 树含 **5 个 `minecraft:interpolated`**（grep JSON 全量）：

| # | 位置 | arg（buildGrid 每点采样对象） |
|---|---|---|
| InterpolatedDF#1 | overworld.json L38（squeeze/mul 下） | `blend_density( add(...深 terrain...range_choice(sloped_cheese...)...) )` |
| InterpolatedDF#2 | noodle.json L4（noodle 的 range_choice.input） | `range_choice("minecraft:y", ..., when_in=noise(noodle), when_out=-1.0)` |
| InterpolatedDF#3 | noodle.json L25（when_out arg1） | `range_choice(y, ..., when_in=add(-0.075, mul(-0.025, noise(noodle_thickness))))` |
| InterpolatedDF#4 | noodle.json L56（when_out arg2→max→arg1） | `range_choice(y, ..., when_in=noise(noodle_ridge_a))` |
| InterpolatedDF#5 | noodle.json L75（when_out arg2→max→arg2） | `range_choice(y, ..., when_in=noise(noodle_ridge_b))` |

> 注：`vein_ridged`/`vein_toggle` 里的 interpolated（overworld.json L280/299/318）**不在** finalDensity 树内（它们是独立 router 字段，final_density 不引用），不计入。

### 1.3 InterpolatedDF#1 的 arg（深 terrain 链）—— 主争用对象

`interp#1.arg` = blend_density 到 sloped_cheese/terrain。下探路径（C++ 类型）：

```
blend_density (BlendDensityDF, 纯委托)
  add(0.1171875, ...)            → LinearOperation(ADD, c=0.1171875)
    mul(ycg(-64,-40,0→1), ...)   → BinaryOperation(MUL, a=ycg, b=...)
      add(-0.1171875, ...)       → LinearOperation
        add(-0.078125, ...)      → LinearOperation
          mul(ycg(240,256,1→0), ...) → BinaryOperation(MUL)
            add(0.078125, ...)   → LinearOperation
              range_choice(sloped_cheese, min=-1e6, max=1.5625)   → RangeChoice
                ├ when_in:  min(sloped_cheese, mul(5.0, caves/entrances))
                └ when_out: max( min( min(add(4*square(noise cave_layer), add(clamp(add(0.27,noise cave_cheese),-1,1), clamp(add(1.5,mul(-0.64,sloped_cheese)),0,0.5))), entrances),
                                       add(spaghetti_2d, spaghetti_roughness) ),
                                  range_choice(pillars) )
```

`sloped_cheese`（sloped_cheese.json）= `add(mul(4.0, quarter_negative(mul(add(depth, mul(jaggedness, half_negative(noise jagged))), factor))), base_3d_noise)`，其中：
- `depth` → add(ycg, `offset`)；`offset` → flat_cache(cache_2d(spline(continents→erosion→ridges_folded)))** = **SplineDF**。
- `factor` → flat_cache(cache_2d(spline(continents→erosion→ridges/ridges_folded))) = **SplineDF**。
- `jaggedness` → flat_cache(cache_2d(spline(continents→erosion→ridges_folded))) = **SplineDF**。
- `base_3d_noise` → **InterpolatedNoiseDF**（`minecraft:old_blended_noise`，注意：**不是** InterpolatedDF，无 buildGrid，直接 Perlin）。

**确证**：interp#1 的 grid 每个点都会下探到 **SplineDF（factor/offset/jaggedness）+ InterpolatedNoiseDF（base_3d_noise）+ NoiseDF（cave_layer/cave_cheese/jagged）+ 若干 cache_2d/flat_cache**。这就是「wrapper 链 = 深链虚调用+寻址」的主体。

### 1.4 整棵树 wrapper 层类型清单（确证，从 density_builder buildNode 映射）

| JSON type | C++ 类 | sample 内部虚调用 | 是否纯委托 |
|---|---|---|---|
| `min`/`max`/`mul`(非 const)/`add`(非 const) | BinaryOperation | a->sample + b->sample（MIN/MAX 有 min/maxValue 短路） | **否** |
| `add`/`mul` 带 Constant | LinearOperation | input->sample（+c 或 ×c 数学） | 近纯（1 次数学） |
| `squeeze`/`abs`/`square`/`cube`/`half_negative`/`quarter_negative` | UnaryOperation | input->sample + applyUnary | **否**（1 次数学） |
| `clamp` | Clamp | input->sample + clampD | **否**（1 次数学） |
| `range_choice` | RangeChoice | input->sample + in_range/out_range->sample | **否** |
| `blend_density` | BlendDensityDF | input->sample | ✅ **纯委托** |
| `cache_once`/`cache_all_in_cell` | WrappingDF | wrapped->sample | ✅ **纯委托** |
| `cache_2d` | Cache2DDF | miss 时 arg->sample（LRU 16 槽） | 缓存包装（有计算） |
| `flat_cache` | FlatCacheDF | buildGrid 无 chunk 命中时 arg->sample | 缓存包装 |
| `interpolated` | InterpolatedDF | buildGrid 时 arg->sample（每点） | 缓存包装（主争用） |
| `noise`/`shifted_noise`/`shift`/`shift_a`/`shift_b` | NoiseDF/ShiftedNoiseDF/ShiftDF | （噪声计算，无 DF 子虚调用除 shifted_noise 的 3 个 shift） | 否 |
| `spline` | SplineDF | sampleNode → locFn->sample（每非叶 1 次） | 否 |
| `y_clamped_gradient` | YClampedGradient | 无 DF 子虚调用 | 叶 |
| `old_blended_noise` | InterpolatedNoiseDF | 无 DF 子虚调用 | 叶（Perlin） |
| `weird_scaled_sampler` | WeirdScaledSampler | input->sample + noise | 否 |

**确证**：finalDensity 树全部 21 层里，**纯委托 DF = blend_density + cache_once(WrappingDF) + LazyRef**（循环引用自引用时）。这些是「删除一层虚调用零计算损失」的层。

---

## 2. buildGrid 虚调用链深度与点数

### 2.1 网格维度（density.h L485/L590，确证）

```
CELL_X=4, CELL_Y=8, CELL_Z=4
GX = 16/CELL_X + 1 = 5
GY = height/CELL_Y + 1 = 384/8 + 1 = 49     (height = 384, minY = -64)
GZ = 16/CELL_Z + 1 = 5
网格点 = GX*GY*GZ = 5*49*5 = 1225/InterpolatedDF·chunk
```

### 2.2 buildGrid 循环结构（density.h L589-619，确证）

```cpp
for (gy in 0..GY)          // 49
  for (gz in 0..GZ)        // 5
    for (gx in 0..GX)      // 5
        if (gx==0 && reuseLeft) { 复用 edgeCol; continue; }   // L600-603：左邻 chunk 复用 gx=0 列
        p = (chunkX*16 + gx*4, minY + gy*8, chunkZ*16 + gz*4)
        grid[...] = arg->sample(p);         // ★ L607 —— 主争用虚调用点
```

- 每网格点 **1 次 `arg->sample(p)` 虚调用**（L607）。
- `reuseLeft`（上轮 chunk 是左邻 chunkX-1）时，`gx==0` 全列（49×5=245 点）不采样 → **实采样 1225-245=980 点/InterpolatedDF·chunk**（若无双邻复用则 1225）。

### 2.3 每 chunk 总 buildGrid 虚调用（确证点 + 推断点）

| 层次 | 网格点 | 每点 arg->sample 下探深度 | 每 chunk buildGrid 虚调用 |
|---|---|---|---|
| InterpolatedDF#1（main terrain） | 1225（reuse 980） | **~18-20 虚分派**（blend→add→mul→...→range_choice→sloped_cheese→factor/offset/jaggedness spline + base_3d_noise） | ~980×18 ≈ **17.6K** 虚调用（含 spline 递归） |
| InterpolatedDF#2（noodle input） | 980 | ~6（range_choice→noise） | ~5.9K |
| InterpolatedDF#3/#4/#5（noodle when_out） | 3×980 | ~6（range_choice→noise/add/mul） | ~17.6K |
| **合计** | 5×1225（reuse 5×980） | — | **≈ 41K 虚调用/chunk（nominal 5×1225=6125 次 arg->sample，含深链内层虚分派）** |

> **推断 @anchor.idk**：`#3/#4/#5` 是否每 chunk 都触发 buildGrid，取决于 noodle range_choice 的 sampled 值是否落入 when_out 分支（`[-1e6, 0)` 之外）。多数 y 为 when_in（返回 64.0）→ #3/#4/#5 可能**较少**构建。此计数需运行时 WG_PROFILE 数 buildGrid 次数确认（静态无法定）。

### 2.4 相邻 cache 包装（非 InterpolatedDF，简列）

- `FlatCacheDF::buildGrid`（density.h L787-803）= **5×5=25** 点/chunk/实例（spline locFn：continents/erosion/ridges/factor/offset/jaggedness）。每点 arg->sample 一次。
- `Cache2DDF`（多槽 LRU 16）miss 时 arg->sample（无固定网格，逐点 LRU）。

---

## 3. wrapper 链的额外虚调用（确证）

**关键：wrapper 链不是单层转发，每层非纯委托的 sample 内部会再调子项虚调用。**

| wrapper | sample 内部虚调用数 | 说明（density.h 行号） |
|---|---|---|
| BinaryOperation（min/max/add/mul） | **2**（a->sample + b->sample） | L123-150；MIN/MAX 用 min/maxValue 短路可降到 1 |
| LinearOperation | **1**（input->sample）+ 1 数学 | L69-72 |
| UnaryOperation（squeeze/abs/...） | **1**（input->sample）+ applyUnary | L191-197 |
| Clamp | **1**（input->sample）+ clampD | L208-214 |
| RangeChoice | **2**（input->sample + in/out_range->sample） | L293-307 |
| ShiftedNoiseDF | **3**（shiftX/Y/Z->sample）+ noise | L270-280 |
| SplineDF | 每非叶 **1**（locFn->sample）+ 递归 | L920 |
| WeirdScaledSampler | **1**（input->sample）+ noise | L364-374 |

**所以**：一个 grid 点（interp#1）的 `arg->sample` 不是一个虚调用，是从 blend_density 一路到叶（spline/noise/flatcache）的**整条链**：每次经过 min/squeeze/add/mul/range_choice 都再触发其内部子项虚调用。**总虚分派 ≈ 层的倍数，不是层数。**

---

## 4. 可数据驱动化点（核心交付）

### 4.1 主争用虚调用点（target）

**`InterpolatedDF::buildGrid` 的 `arg->sample(p)`（L607）** + 其下深链。其中 **InterpolatedDF#1**（arg = blend_density→深 terrain 链→spline/noise）是最重者：**~980 个网格点 × 每个点重走 18-20 层实虚分派 + spline 递归 + base_3d_noise Perlin**。这就是 91% 走 wrapper 链的时间。

> 对比：grid 命中后的 `InterpolatedDF::sample`（L497-558）= 纯查表 trilinear（下采样读 grid[ ]，**无虚调用**）——不是争用。

### 4.2 纯委托层 —— 最容易扁平化（确证清单）

这些层 `sample` **只 `return input->sample(pos)`，无任何内部计算**，是「删一层虚调用零成本」：

| 类 | 位置 | JSON type | 在树中位置 |
|---|---|---|---|
| **BlendDensityDF** | density.h L635-642 | `minecraft:blend_density` | **interp#1 直接 arg**（最重要的一个） |
| **WrappingDF** | L645-652 | `cache_once`/`cache_all_in_cell`；`shift_x`/`shift_z` 包装 | caves/entrances/pillars/spaghetti_roughness；bias 链 |
| **LazyRef** | density_builder.h L275-282 | range_choice 自引用 | 自引用回退 |

**扁平化价值**：
- **interp#1 直接 arg = BlendDensityDF** → 把 interp#1 的 arg 直接接到内层 `add(0.1171875,...)`，**删掉 blend_density 这 1 层虚分派**（每网格点省 1 次，980 点 → 省 ~980 次/chunk 纯虚调用）。成本 0，语义无损（NoBlending 恒等）。
- **cache_once（WrappingDF）**：在 buildGrid 下探路径（entrances/pillars/spaghetti_roughness）里是纯转发 → 可类似剥掉。

### 4.3 LinearOperation（近纯委托）

`LinearOperation`（add/mul 带 Constant 折叠结果）：`input->sample` + 一次 ±c/×c。删它只能省 1 次虚调用但**保留 1 次数学**（即省不了计算层），扁平化收益小于纯委托但仍是高覆盖层（顶部 add(0.117)/mul(0.64)/各 LinearOp）。

### 4.4 数据驱动化方向（推断 @anchor.idk，需主会话验证）

把 buildGrid 的 `arg->sample` 改为**数据驱动/类型分派**（类似 SplineDF 已实现的 `sampleSerialLocFn` 的 kind-switch + 连续池，density.h L975-988；或 C2ME DFC 式 op 表直排）：
- 对 interp#1 的 arg（深 terrain 链）按其**静态类型序列**编译成 **op 表**（min/squeeze/mul/range_choice/spline/noise...），buildGrid 每网格点按**整数索引连续遍历**执行，**去掉虚分派 vtable 跳转 + shared_ptr 解引用寻址**。
- 收益：**每网格点省 ~18 次虚分派 → interp#1 的 980 点 → ~17K 虚调用/chunk 变直接函数调用**。这是 91% 争用的直接打击面。

---

## 5. 单次 chunk 总虚调用估算（确证 + 推断分界）

fillOneChunkCore 每 chunk 采 **98304 点**（16×16×384，worldgen_api.cpp L794）。分两块：

### 5.1 buildGrid 虚调用（每 chunk 一次性，主争用，推断含深链内层）

- **顶层 arg->sample 虚调用**（L607）= 5 InterpolatedDF × 1225 ≈ **6125 次/chunk**（reuse 后 5×980 ≈ **4900**）。
- 每次 arg->sample **不是 1 次虚调用**：interp#1 的点下探 ~18-20 层（每层内部分派），noodle interp#2-#5 下探 ~6 层。→ **含内层的 buildGrid 总虚分派 ≈ 41K/chunk**（§2.3），且每个还叠 spline 递归 + Perlin/噪声计算。
- **这是 11× 并发争用所在**：8 线程各触发自身 buildGrid，深链虚调用 + 寻址（shared_ptr 跳转、spline 表、噪声 perm 表）被缓存层级/内存带宽放大约 10×。

### 5.2 每点采样虚调用（grid 命中后，非主争用）

- interp#1/#2/#3/#4/#5 grid 命中后 `InterpolatedDF::sample` = **纯 trilinear 查表，0 虚调用**。
- 但**顶层 wrapper 仍逐点虚调用**：98304 点 ×（min 的 a/b.sample + squeeze + mul(0.64) + noodle-RC input/branch + when_out 链）≈ **98304 × 6-12 ≈ 60万-120万次**虚调用/chunk。**数量多但每次浅（1 次 vtable 跳转），不深探** → 贡献低于 buildGrid 深链。

### 5.3 结论（哪些是 91%，哪些不是）

| 部分 | 虚调用次数/chunk | 每次代价 | 对 11× 争用 |
|---|---|---|---|
| interp#1 buildGrid（深链下探） | ~980 点 × ~18-20 层 ≈ **17.6K** | **高**（spline 递归 + Perlin + 寻址） | **主争用（91% 去向）** |
| noodle interp#2-#5 buildGrid | ~3920 点 × ~6 层 ≈ **23K** | 中（noise） | 次要 |
| grid 命中后 trilinear | 0 虚调用 | 低 | 无争用 |
| 顶层 wrapper 逐点虚调用 | 98304 × 6-12 ≈ **60万-120万** | 低（每层浅） | 次（但量大） |

> **推断 @anchor.idk**：究竟 buildGrid（深链）还是顶层逐点虚调用占比大，需运行时 WG_PROFILE/WG_PHASETICK 实测区分。**静态证据强烈倾向 buildGrid 深链主争用**（91% = whole-tree 33.54ms vs spline-only 3.015ms 的差，主要来自 interp#1 逐网格点重走深链）；grid 命中后 trilinear 无虚调用，排除。

---

## 6. 结论（强化 wrapper-buildgrid 数据驱动化方向）

1. **5 个 InterpolatedDF** 构建各自 grid；主争用 = **InterpolatedDF#1**（arg = blend_density→深 terrain 链→factor/offset/jaggedness SplineDF + base_3d_noise InterpolatedNoiseDF），其 buildGrid 每 chunk 的 **~980 个网格点** 每点重走 **18-20 层实虚分派**。
2. **buildGrid 虚调用点 = `arg->sample`（density.h L607）**，这是数据驱动化的直接 target（替换为 op 表直排 / 连续池 + kind-switch）。
3. **最易扁平**：blend_density（interp#1 的直接 arg）、cache_once(WrappingDF)、LazyRef = 纯委托，删 1 层虚分派零计算损失。
4. **wrapper 非纯委托**（min/squeeze/add/mul/range_choice/clamp/abs...）sample 内部有子项虚调用，是 18-20 层深链的构成，**不能**逐层剥，只能整体数据驱动化。
5. **11× 争用 = buildGrid 深链遍历**（interp#1 的 grid 点每点重走整棵深链 + 寻址），**不是** grid 命中后的 trilinear（无虚调用）。

## 关键源码索引（scout 定位）
- `versions/1.20.1/cpp/worldgen/src/density.h`：InterpolatedDF::sample L497-558 / buildGrid L589-619（L607=arg->sample）；BlendDensityDF L635-642；WrappingDF L645-652；BinaryOperation L90-153；UnaryOperation L173-200；Cache2DDF L661-719；FlatCacheDF L734-804；SplineDF L811-1084。
- `versions/1.20.1/cpp/worldgen/src/density_builder.h`：buildNode L31-181；buildSpline L184-218；resolveRef L221-263；LazyRef L275-282。
- `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/overworld.json`：final_density L30-168（interp#1 L38）。
- `versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/caves/noodle.json`：interp#2 L4 / #3 L25 / #4 L56 / #5 L75。
- `versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/{sloped_cheese,depth,factor,jaggedness,offset,base_3d_noise,caves/{entrances,spaghetti_2d,spaghetti_roughness_function,pillars}}.json`：terrain 深链。
