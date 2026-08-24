# finalDensity 顶层 wrapper sample 逻辑 — scout 深挖（每点虚调用层数 / 纯委托 vs 有计算 / 可合并可剥层 / 最小去虚调用改法）

> 角色：scout（只读勘探，不改码不编译）。产物 `.investigations/worldgen-mt-scaling/`，供主会话定「顶层逐点 wrapper 包装虚调用」的最小去虚调用改法。
> 前置（采信，不重验）：production 并发 11× = **顶层逐点 wrapper 包装虚调用**（production 三层对照：spline-only 1.62×、warm 10.10×、cold 10.32×；buildGrid 深链无碍、spline 无碍）。final_density 树顶点 = `min(squeeze(mul(0.64, InterpolatedDF#1)), noodle)`。
> **标注**：确证 = 静态读 `density.h` / `density_builder.h` / `overworld.json` 及 overworld 各 JSON 可定；推断 = 需运行时测量 → 以 `@anchor.idk` 标注。
> 与既有 `wrapper-buildgrid-structure.md` 的关系：那篇聚焦 **buildGrid 冷路径**（arg->sample 深链）；本篇聚焦 **每点温暖路径的顶层 wrapper 虚调用链**（用户已钉死 11× 主因 = 顶层逐点，故本篇不做 buildGrid 深链，只做顶层 per-point）。

---

## 0. TL;DR（核心结论）

| # | 问题 | 结论 |
|---|---|---|
| 1 | finalDensity 顶层拓扑 | `min( squeeze( mul(0.64, InterpolatedDF#1) ), noodle )`。a 链 = min→squeeze→mul(0.64)→InterpolatedDF#1（唯一 terrain 插值）；b 链 = noodle（RangeChoice 顶，内含 InterpolatedDF#A/B/C/D）。 | 
| 2 | 顶层 wrapper 的 sample | **min/squeeze/mul/range_choice ≈ 有计算**（每层 1-2 次子虚调用 + 自身算子）；**blend_density/cache_once(WrappingDF)/LazyRef = 纯委托**（sample 一行 `return input->sample`，0 自身计算）。见 §2 逐类表。 |
| 3 | 每点虚调用层数（温暖路径） | a 链 = **4 层虚分派**（MIN、squeeze、mul、interp#1），其中 3 层为「有计算」wrapper，interp#1 为 grid 边界；noodle 链另计。**interp grid 命中后 = 0 虚调用**（纯 trilinear 查表）；**spline 内部 = 温暖路径根本不触碰**（只在 buildGrid 冷路径）。见 §3。 |
| 4 | 可剥（纯委托）层 | `BlendDensityDF`(density.h:639)、`WrappingDF`(:649)、`LazyRef`(density_builder.h:278)。**但它们全部位于 interp 网格之下 → 只走 buildGrid 冷路径，温暖 per-point 路径零纯委托层**。可数据驱动化层 = 温暖链的 **min/squeeze/mul/range_choice**（有计算，不能剥，只能 kind-switch）。见 §4。 |
| 5 | 最小改法 | **剥纯委托（blend_density/cache_once）只降 buildGrid 冷路径虚调用，对 11× 温暖 per-point 争用收益≈0**。针对 11× 的最小改法 = **数据驱动化温暖 a 链的 min/squeeze/mul（+ noodle 的 range_choice）**，把每点 a 链 **4 层 → 2 层**（1 kind-switch 内联 + 1 interp#1 grid 访问）；全 DFC 则 **4 层 → 0 层**（整树扁平，interp 也是 op）。见 §5。 |

---

## 1. finalDensity 顶层确切拓扑（从 overworld.json noise_router.final_density 出发）

源：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/overworld.json` L30-168（`noise_router.final_density`）。`density_builder.h` `buildNode`(L31-181) 映射 JSON type → C++ 类。

### 1.1 顶层结构（确证）

```
final_density                                        # L31  minecraft:min  → BinaryOperation(MIN), density.h:90-153
├── arg1 ("a") = squeeze                              # L33  minecraft:squeeze → UnaryOperation(SQUEEZE), density.h:173-200
│   └── argument = mul(0.64, ...)                     # L35  minecraft:mul 0.64 → BinaryOperation::create → 折叠为 LinearOperation(MUL,c=0.64), density.h:62-75
│       └── argument2 = interpolated                  # L38  minecraft:interpolated → InterpolatedDF#1（唯一主 terrain 插值）, density.h:482-620
│           └── argument = blend_density              # L40  minecraft:blend_density → BlendDensityDF（纯委托）, density.h:635-642
│               └── argument = add(0.1171875, ...)    # L42  → WhiteOperation::create(ADD, Constant, X) → LinearOperation(ADD,c=0.1171875)
│                   └── input = [深 terrain 链：ycg*mul + ... + range_choice(sloped_cheese) ...]   # L45-161
└── arg2 ("b") = "minecraft:overworld/caves/noodle"   # L167  字符串 ref → resolveRef → noodle.json 树（**非** 纯委托，RangeChoice 顶）
```

- **`arg1`（squeeze 链）是唯一的 InterpolatedDF#1**，arg = blend_density → 深 terrain 链（range_choice(sloped_cheese) → factor/offset/jaggedness SplineDF + base_3d_noise + entrances/pillars/spaghetti_*）。
- **`arg2`（noodle）是独立子树**（noodle.json），RangeChoice 顶，内含 4 个 InterpolatedDF（#A/#B/#C/#D）。
- 关键：add(0.1171875) 的 LinearOperation 位于 **interp#1 之下**（cold 路径），**不在**温暖 per-point a 链。

### 1.2 InterpolatedDF 实例（整树 5 个，确证）

| # | 位置 | arg（buildGrid 每点采样对象） |
|---|---|---|
| #1 | overworld.json L38（squeeze/mul 下） | `blend_density(add(...深 terrain...range_choice(sloped_cheese)...))` |
| #A | noodle.json L4（noodle 的 range_choice.input） | `range_choice("minecraft:y",...,when_in=noise(noodle),when_out=-1.0)` |
| #B | noodle.json L25（when_out.arg1） | `range_choice(y,...,when_in=add(-0.075,mul(-0.025,noise(noodle_thickness))))` |
| #C | noodle.json L56（when_out.arg2→max.arg1） | `range_choice(y,...,when_in=noise(noodle_ridge_a))` |
| #D | noodle.json L75（when_out.arg2→max.arg2） | `range_choice(y,...,when_in=noise(noodle_ridge_b))` |

> `vein_ridged`/`vein_toggle` 里的 interpolated（overworld.json L280/299/318）**不在** finalDensity 树（独立 router 字段），不计入。

### 1.3 顶层链每层 C++ 类 + 是否纯委托（确证，逐层列表）

| 层 | JSON type | C++ 类 | density.h | 是否纯委托 |
|---|---|---|---|---|
| 1（根） | min | BinaryOperation(MIN) | L90-153 | 否（有计算） |
| 2 | squeeze | UnaryOperation(SQUEEZE) | L173-200 | 否（有计算） |
| 3 | mul 0.64 | LinearOperation(MUL) | L62-75 | 否（有计算，×c） |
| 4 | interpolated | InterpolatedDF#1 | L482-620 | 否（grid/trilinear） |
| 5（#1 之下） | blend_density | **BlendDensityDF** | **L635-642** | **✅ 纯委托** |
| 6（#1 之下） | add 0.1171875 | LinearOperation(ADD) | L62-75 | 否（+c） |
| （b 链根） | noodle | RangeChoice | L286-314 | 否（有计算） |
| （b 链下） | interpolated | InterpolatedDF#A/B/C/D | L482-620 | 否（grid） |

---

## 2. 每类顶层 wrapper 的 sample 实现（逐类读 density.h，逐层判断）

### 2.1 逐类 sample 逻辑 + 虚调用子调用数（确证）

「虚调用子调用数」= 该节点 `sample()` 内部发起的**子 `sample()` 虚分派次数**（不含进入本节点的分派）。
「own compute?」= 自身有无算术算子（非纯转发）。

| C++ 类 | JSON type | density.h | sample 主体 | 子 sample 虚调用数 | own compute? | 纯委托? |
|---|---|---|---|---|---|---|
| BinaryOperation(MIN) | min | L123-129 | `da=a->sample; r = da < b->minValue() ? da : min(da, b->sample)` | **1**(a) + **1**(b, 条件) + 1 次 minValue() 虚分派（非采样） | 是（min + 分支测试） | 否 |
| BinaryOperation(MAX) | max | L130-138 | `da=a->sample; bmax=b->maxValue; bv=b->sample; r=da>bmax?da:max(da,bv)` | **1**(a) + **1**(b) + 1 次 maxValue() | 是 | 否 |
| BinaryOperation(ADD,MUL 无 const) | add/mul | L123-128 | ADD:`da=a->sample; r=da+b->sample` | **2**(a+b) | 是 | 否 |
| | | | MUL:`da=a->sample; r=da==0?0:da*b->sample` | **1-2** | 是 | 否 |
| LinearOperation | add/mul 带 const | L69-72 | `x=input->sample; return op==MUL? x*c : x+c` | **1**(input) | 是（×c / +c） | 否（近纯：1 次数学） |
| UnaryOperation | squeeze/abs/square/cube | L191-197 | `r=applyUnary(op, input->sample)` | **1**(input) | 是（applyUnary） | 否 |
| Clamp | clamp | L208-214 | `r=clampD(input->sample, mn, mx)` | **1**(input) | 是（clampD） | 否 |
| RangeChoice | range_choice | L293-307 | `d=input->sample; r=(min<=d<max)? inRange->sample : outOfRange->sample` | **1**(input) + **1**(branch) | 是（区间测试） | 否 |
| ShiftedNoiseDF | shifted_noise | L270-280 | `d=x*xz+sx->sample; ...` | **3**(shiftX/Y/Z) | 是 | 否 |
| WeirdScaledSampler | weird_scaled_sampler | L364-374 | `d=scale(rarity, input->sample); ...` | **1**(input) | 是 | 否 |
| YClampedGradient | y_clamped_gradient | L331-337 | `clampedMap(pos.y,...)` | **0** | 是 | 否（叶） |
| NoiseDF | noise | L226-235 | `noise->sample(...)` | **0** | 是 | 否（叶） |
| InterpolatedNoiseDF | old_blended_noise | L399-473 | Perlin octaves | **0** | 是 | 否（叶） |
| **BlendDensityDF** | blend_density | **L639** | `return input->sample(pos);` | **1**（仅转发） | **0** | **✅ 纯委托** |
| **WrappingDF** | cache_once/cache_all_in_cell | **L649** | `return wrapped->sample(pos);` | **1**（仅转发） | **0** | **✅ 纯委托** |
| **LazyRef** | （range_choice 自引用） | density_builder.h **L278** | `return target->sample(pos);` | **1**（仅转发） | **0** | **✅ 纯委托** |
| InterpolatedDF | interpolated | L497-558 | grid 命中 → 8 角点读 + 三线性；grid 未命中 → buildGrid(arg->sample ×1225) | 命中 **0**；未命中 buildGrid 内 **1225** | 是 | 否 |
| Cache2DDF | cache_2d | L668-689 | LRU 16 槽查；miss → arg->sample | 命中 **0**；miss **1** | 是 | 否 |
| FlatCacheDF | flat_cache | L742-761 | 5×5 网格查；界外 → arg->sample | 命中 **0**；miss **1** | 是 | 否 |
| SplineDF | spline | L898-909 | sampleNode → locFn->sample（每非叶 1）+ 递归 + Hermite | 每非叶 **1**(locFn)+递归 | 是 | 否 |

### 2.2 纯委托层三兄弟（确证，最轻量可剥）

- **BlendDensityDF** `density.h:639` —— `sample` 仅 `return input->sample(pos);`，0 自身计算（NoBlending 恒等）。
- **WrappingDF** `density.h:649` —— `sample` 仅 `return wrapped->sample(pos);`，0 自身计算。用于 `cache_once`/`cache_all_in_cell`（density_builder.h:158）+ `shift_x`/`shift_z` 包装（density_builder.h:227/233）。
- **LazyRef** `density_builder.h:275-282` —— `sample` 仅 `return target->sample(pos);`，0 自身计算。用于 range_choice 等自引用（`ref==selfKey`，density_builder.h:37-39）。

这三类样本全部是「一行 `return X->sample(pos)`」= **零逻辑损失可剥层**：可直接从 finalDensity 树剥离（把父层直接接到 X），每剥 1 层省 1 次虚调用 + 1 次 shared_ptr 寻址。

---

## 3. 每点虚调用层数（核心量化）

**约定**：`finalDensity->sample(pos)` 在**温暖稳定态**（本 chunk 全部 InterpolatedDF grid 已建好）下，每点触发的**虚分派 `sample()` 次数**（含进入各节点的分派）。分三段：【top wrapper 层】（主争用）/【interp grid 命中后】/【spline 内部】。

### 3.1【top wrapper 层】（主争用 = 每点都发生，98304 点/chunk）

a 链（terrain 主分支，每点必走）：

| 次序 | 节点 | 类 | 虚分派 | 备注 |
|---|---|---|---|---|
| 0 | final_density | BinaryOperation(MIN) | 1 | 进入 MIN::sample |
| 1 | arg1 | UnaryOperation(SQUEEZE) | 1 | MIN 内 a->sample |
| 2 | argument | LinearOperation(MUL 0.64) | 1 | squeeze 内 input->sample |
| 3 | argument2 | InterpolatedDF#1 | 1 | mul 内 input->sample → grid 边界 |
| — | **小计（a 链）** | — | **4 虚分派/点** | 其中 3 层「有计算」wrapper + 1 层 grid |

b 链（noodle 分支，MIN 内 `da >= noodle.minValue()` 才走）：

| 次序 | 节点 | 类 | 虚分派 | 备注 |
|---|---|---|---|---|
| 3a | noodle | RangeChoice | 1 | MIN 内 b->sample（条件） |
| 3b | input | InterpolatedDF#A | 1 | RC 内 input->sample → grid |
| 3c | branch | Constant(64.0) **或** add 链 | 1 | RC 内 in/out_range->sample |
| — | **小计（noodle 头）** | — | **≈3 虚分派/点** | 若 out-of-range 再 + add/mul/max/abs×2/interp#B/C/D ≈ **+7** |

> MIN 内还有 1 次 **`b->minValue()` 虚分派**（非采样，RangeChoice.minValue → min(inRange.minValue, outRange.minValue) 递归下探，density.h:308-313）。这也是 per-point 虚分派但属范围查询。
> `@anchor.idk`：noodle 分支 in-range（d<0，返回常量 64.0）vs out-of-range（d≥0，走 add 深链）的占比取决于 noodle 噪声值，需运行时确认；in-range 时每点 noodle 仅 +3 分派，out-of-range 时 +10 分派。

**典型温暖 per-point 总虚分派** ≈ a 链 4 + b->minValue 1 + noodle 头 3（in-range）≈ **8 层/点**；out-of-range 时 ≈ **15 层/点**。98304 点 × ~8-15 ≈ **80 万-150 万次/点虚分派/chunk**。

### 3.2【interp grid 命中后】= 0 虚调用（确证）

`InterpolatedDF::sample`（density.h:497-558）grid 命中后**纯查表 trilinear**：L534-548 `fx/fy/fz` + `d000..d111` 8 次 grid[ ] 读 + 3 次 lerp，**0 次子 sample 虚调用**。grid 未命中才进入 buildGrid（L589-619）→ `arg->sample(p)` ×1225（这是**冷路径**，任务已钉死「buildGrid 深链无碍」）。

### 3.3【spline 内部】= 温暖路径根本不触碰（确证）

全程 spline（factor/offset/jaggedness/continents/erosion/ridges）与 base_3d_noise、entrances/pillars/spaghetti_* 全部位于 **InterpolatedDF#1 的 arg 之内 → 只在 buildGrid（冷路径）逐网格点采样**。温暖 per-point 路径从 interp#1 grid 命中即返回，**永不下探 spline**。故「spline 内部虚调用」仅占 buildGrid 冷路径，不占温暖每点。这与「spline-only 1.62×（非主因）」自洽。

### 3.4 每点虚调用层数速查表

| 段 | 每点虚分派 | 是否主争用 | 说明 |
|---|---|---|---|
| top wrapper 层（a 链 min/squeeze/mul） | **3**（wrapper）+1（interp#1）= 4 | ✅ **主争用** | 每点必走，3 层有计算 wrapper |
| noodle 头（RangeChoice + interp#A + branch） | ≈3（in-range）/ ≈10（out-range） | 次（仍 per-point） | noodle 是独立 per-point 链 |
| interp grid 命中后 trilinear | **0** | 否 | 8 次数组读 + 3 lerp |
| spline 内部 | **0**（温暖）/ 高（冷） | 否（buildGrid 冷路径） | 只在 interp 建 grid 时 |
| MIN 的 minValue() 范围查询 | 1（虚分派） | 副 | 非采样，递归下探 |

---

## 4. 可合并/可剥层（核心）+ 可数据驱动化层

### 4.1 纯委托层（可剥，零逻辑损失）——确证清单

| 类 | 位置 | JSON type | sample 主体 | **剥 1 层省** |
|---|---|---|---|---|
| **BlendDensityDF** | density.h:635-642 | minecraft:blend_density | `return input->sample(pos);` | 1 虚调用 + 1 寻址 |
| **WrappingDF** | density.h:645-652 | cache_once/cache_all_in_cell；shift_x/z 包装 | `return wrapped->sample(pos);` | 1 虚调用 + 1 寻址 |
| **LazyRef** | density_builder.h:275-282 | range_choice 自引用 | `return target->sample(pos);` | 1 虚调用 + 1 寻址 |

**关键定位**：这三类**全部位于 InterpolatedDF 网格之下** → 只在 buildGrid 冷路径被采样。温暖 per-point 路径（top wrapper 链）**零纯委托层**（min/squeeze/mul 全是「有计算」，不能剥）。

- `blend_density`（BlendDensityDF）= interp#1 的直接 arg（overworld.json L40）→ 只在 interp#1 buildGrid 冷路径被采样。
- `cache_once`（WrappingDF）= entrances（caves/entrances.json L2，warm 链用 2 次：when_in 与 when_out）、pillars（L2）、spaghetti_roughness_function（caves/spaghetti_roughness_function.json L2）、spaghetti_2d_thickness_modulator（L2）→ 全部在 interp#1 的 arg 深链（cold 路径）。
- `LazyRef` = finalDensity 树无自引用（noodle/sloped_cheese/entrances 等 ref 均非 self）→ **温暖路径无 LazyRef**。

> **结论**：剥纯委托层（blend_density/cache_once）**降低的是 buildGrid 冷路径虚调用**（每 interp#1 buildGrid 省 ~980 次 blend_density 虚调用 + 各 cache_once 虚调用），**对 11× 温暖 per-point 争用收益≈0**——因为温暖 per-point 链中没有纯委托层。

### 4.2 有计算层（不能剥，可数据驱动化）——确证清单

**温暖 per-point a 链**：`BinaryOperation(MIN)`、`UnaryOperation(SQUEEZE)`、`LinearOperation(MUL 0.64)`（→ 若 noodle out-range 还有 `RangeChoice`、`BinaryOperation(ADD/MAX)`、`UnaryOperation(ABS)`、`Constant`）。
**noodle + buildGrid 深链**：`RangeChoice`、`BinaryOperation`、`UnaryOperation`、`Clamp`、`SplineDF`、`NoiseDF`、`Cache2DDF`、`FlatCacheDF`。

这些层 each sample 内部带自身算子（min/squeeze/×c/clamp/区间测试…），**不能直接从树剥离**（剥了会丢语义），但可 **kind-switch 数据驱动化去虚调用**：把多态虚分派改为「单一节点类型标签 + switch 直派 + 内联算子」，每层从 1 次 vtable 间接跳转 → 1 次直接函数调用/内联。

---

## 5. 最小去虚调用改法建议（N 层 → M 层）

### 5.1 先钉死目标：11× 主争用 = 温暖 per-point 顶层 wrapper 虚调用（非 buildGrid、非 spline）

由前置（warm 10.10× / cold 10.32× / spline-only 1.62×）：
- **spline-only 1.62×** → spline 递归虚调用不是 11× 主因。
- **buildGrid 深链无碍** → interp 建 grid 的冷路径不是主因。
- **top wrapper 每点虚调用** → 才是主因。即 §3.1 的温暖 a 链（min/squeeze/mul + interp#1 grid 访问）+ noodle 头。

### 5.2 各改法的每点虚调用降幅（a 链口径，N→M）

| 改法 | 每点 a 链虚分派 | 逻辑损失 | 打击面 | 说明 |
|---|---|---|---|---|
| 现状 | **4**（MIN+squeeze+mul+interp#1 grid） | 0 | — | warm 10.10× |
| **只剥纯委托**（blend_density/cache_once） | **4（不变）** | 0 | 仅 buildGrid 冷路径 | 温暖 a 链无纯委托层，**不降 per-point** ⚠️ |
| **数据驱动化 min/squeeze/mul**（kind-switch） | **2**（1 次扁平 wrapper 内联 + interp#1 grid） | 0 | ✅ 温暖 a 链 | 3 层有计算 wrapper 消 2 层虚分派 |
| **数据驱动化 min/squeeze/mul/noodle-RC** | a 链 2；noodle 头也消 | 0 | 温暖 a+b 链 | 更完整 |
| **全 DFC（整树扁平，interp 也是 op）** | **0**（全内联，interp 为 op 表项） | 0 | 整树 | C2ME 式，最大收益但改动最大 |

**估计**：只剥纯委托（blend_density/LazyRef/cache_once）→ 每点 a 链**仍 4 层**，对 11× 无益（它们不在温暖链）。**数据驱动化 min/squeeze/mul（+noodle RC）** → a 链 **4 → 2 层**（-50%），noodle 头同步受益，***直接打击 11× 温暖链***。**全 DFC** → **4 → 0 层**。

### 5.3 最小有效改法（推荐基线）

**数据驱动化「温暖 a 链」的 min/squeeze/mul（+ noodle 的 RangeChoice）**，而非剥纯委托：

1. **构造侧（density_builder.h buildNode）**：对 `minecraft:min`/`squeeze`/`mul(0.64, ...)`/`range_choice` 生成**带类型标签的扁平节点**（类似 SplineDF 已有的 `sampleSerialLocFn` kind-switch + 连续池，density.h:975-988），而非散 `shared_ptr<DensityFunction>` 对象树。
2. **求值侧**：温暖 per-point 链由 1 次 `sample` 进入一个 kind-switch，switch 内**内联** squeeze/mul/min 的算术，再单次分派到 interp#1 grid。min 的 `minValue()` 分支测试改为**预计算 minValue 常量**（hoist 出 per-point 循环），消除每点 1 次范围查询虚分派。
3. **结果**：a 链每点 **4 → 2 虚分派**（+ noodle 头受益），11× 主争用被直接打击（reduce 第 2、3 层虚分派 + minValue 分派）。

**顺带做**（低成本、收益在冷路径）：把 interp#1 的 arg 直接接到 blend_density 内层 add，剥掉 blend_density 这 1 层（buildGrid 冷路径省 ~980 次虚调用/点·chunk）；cache_once(WrappingDF) 同理剥。**但注意这不解决 11×（温暖链）**，只是冷路径锦上添花。

### 5.4 风险 / 边界

- **无损约束**：数据驱动化必须保证 `squeeze(mul(0.64, x))` 的浮点算式逐位不变（`d/2 - d³/24` + `×0.64`），min 分支语义（`da < b->minValue()`）不变。这与 SplineDF 扁平化（BK-001 零退化）同策略。
- **minValue() 优化风险**：noodle 的 `minValue()` 是递归范围查询（density.h:308-313），hoist 前需确认它是**纯函数/稳定**（minValue 不随 pos 变）——它是，故可缓存/hoist 出 per-point 循环。
- **noodle out-range 占比**：§3.1 的 noodle 头分支深链（in-range → 常量 64.0 vs out-range → add/mul/max/abs×2）占比影响每点总虚分派，`@anchor.idk` 需运行时确认。但无论如何，**a 链的 min/squeeze/mul 每点必走**，数据驱动化它们收益确定。
- **范围声明**：本文只做静态每点虚调用层数量化（§3）与可剥/可数据驱动化层判定（§4）。**实际 11× 的准确收益需主会话 A/B 实测**（WG_PHASETICK，禁 WG_PROFILE/WG_STAGETIMER，见 AGENTS.md 八）。静态证据：数据驱动化 min/squeeze/mul **必须**降低温暖 per-point 虚分派（4→2），故可预言对 warm 10.10× 有收益；但收益量级 @anchor.idk。

---

## 关键源码索引

- `versions/1.20.1/cpp/worldgen/src/density.h`：DensityFunction L49-55 / LinearOperation L62-75 / BinaryOperation L90-153 / UnaryOperation L173-200 + applyUnary L158-171 / Clamp L203-217 / NoiseDF L220-238 / RangeChoice L286-314 / BlendDensityDF L635-642 / WrappingDF L645-652 / Cache2DDF L661-719 / FlatCacheDF L734-804 / InterpolatedDF L482-620（buildGrid L589-619，L607=arg->sample）/ SplineDF L811-1084（locFn kind-switch L975-988）。
- `versions/1.20.1/cpp/worldgen/src/density_builder.h`：buildNode L31-181 / buildSplineNode L193-217 / resolveRef L221-263 / LazyRef L275-282 / cache_once→WrappingDF L155-159。
- `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/overworld.json`：final_density L30-168（min L31 / squeeze L33 / mul L35 / interp#1 L38 / blend_density L40 / add L42 / noodle L167）。
- `versions/1.20.1/data/worldgen/data/minecraft/worldgen/density_function/overworld/caves/noodle.json`：RangeChoice 顶 L2 / interp#A L4 / #B L25 / #C L56 / #D L75。
- overworld JSON：sloped_cheese / depth / factor / jaggedness / offset / base_3d_noise + caves/{entrances(noodle 用),spaghetti_2d,spaghetti_roughness_function,pillars} → terrain 深链（cold）。

## 状态
- 结构性事实（表/类/行号/每点虚分派链）**确证**（静态读源码+JSON）。
- 「数据驱动化 min/squeeze/mul 能降 11×」**candidate/推断**（@anchor.idk 量级需运行时 A/B）；纯委托层全在 cold 路径（温暖链无纯委托层）**确证**。
