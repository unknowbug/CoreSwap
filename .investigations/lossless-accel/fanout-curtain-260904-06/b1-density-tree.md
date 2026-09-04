# B1：density 树内容/采样差 —— 幕帘（y≈192-318 海洋域 0<d≤0.39 stone run）共有偏离候选

> **status**: candidate-pending（fan-out b1 臂 worker 产物；不下最终裁决，收敛归主会话+judge）
> **置信度**: B1 候选域可行性 = 中（数值层面高度可行，见 §2）；三个子候选（§4）均**未验证**，各自验前置信度不分配。
> **§9.7 验证可比性口径三要素**: ① 载体 = 纯静态审查（代码 + JSON 逐行审读），**无任何运行时/数据层验证**；② 覆盖面 = 幕帘带 y 192-318 深海（低 continentalness）域的理论符号推导 + density 树全常量抽查 + 两侧生成路径同源性核查；③ 与既有口径可比性 = 与 curtain-scout-260904-06/map.md §1 探针清单同域，但**与既有 96.06% 逐位对齐口径（block_probe Full）不可比**——那是陆地/浅域，本课题域未被其覆盖。
> 范围遵守：只读分析，未跑命令、未改任何源码/数据。

---

## 1. 域定义与读取清单

幕帘 = NOISE 阶段 y≈192-318、深海（低 continentalness）域的 stone/granite/copper_ore 交替 run，0<d≤0.39，间隔 ~16 含 aquifer 水口袋（convergence-260904-06 新世界图景）。stone 判定量 = **finalDensity**（+beardifier），不是 initial：

- C++：`worldgen_api.cpp:912`（finalDensity→densityBuf）→ `:1040` aquifer.apply → `aquifer.h:74` `density>0 → -1`（stone）。
- Rust：`terrain.rs:273`（d0=final 采样）→ `:277` classify → `terrain.rs:223` `d>0 → Rock`（stone）。
- initial_density_without_jaggedness 只进 est 扫描（`aquifer.h:159` / `worldgen_handle.rs:571-577`），**不直接判 stone**。

## 2. 理论数值行为（vanilla 语义，静态推导）

### 2.1 initial_density_without_jaggedness 在幕帘域的符号

树结构（`versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json:181-240`，已抽查与 vanilla 1.20.1 已知结构一致）：

```
0.1171875 + grad1 * ( -0.1171875 + ( -0.078125 + grad2 * ( 0.078125 + clamp(-0.703125 + 4*quarter_negative(depth*cache_2d(factor)), -64, 64) ) ) )
grad1 = y_clamped_gradient(-64→-40, 0→1)；grad2 = y_clamped_gradient(240→256, 1→0)
depth = y_clamped_gradient(-64→320, 1.5→-1.5) + offset   （density_function/overworld/depth.json:4-10）
```

深海域数值（offset 取 generated 文件所示深海段 ≈ -0.2222，`generated/vanilla_density_functions.rs:163`；factor 深海 continents<-0.19 段 ≈ 3.95，`factor.json:23-25`）：

| y 段 | grad1 | grad2 | depth@y=200 ≈ | initial ≈ |
|---|---|---|---|---|
| 192-240 | 1.0 | 1.0 | 1.5-2.0625-0.2222 ≈ **-0.785** | 0.117-0.078+0.078+clamp(-0.703+4·(-0.785·3.95)) ≈ 0.117-0.703-12.4 ≈ **-13.0 ≪ 0** |
| 240-256 | 1.0 | 1→0 线性 | — | 线性过渡 |
| **256-318** | 1.0 | **0.0（钳位）** | — | **+0.0390625（精确常数）** = 0.1171875-0.078125 |

**关键结论（本臂最重要单点发现）**：vanilla 语义下 initial 在 y>256 就是 **+0.0390625 > 0**（grad2 钳 0，深度项整个消失）——但因 <0.390625，est 扫描不触发（`aquifer.h:159`），也不判 stone。即「initial 在幕帘带恒 ≤0」的直觉是**错的**；正确表述是：**initial 在 256 以上小正值常数、192-240 大负值**。幕帘实测上界 0.39 与 0.390625/0.039 的数量级耦合 idk（可能只是测量巧合）。

### 2.2 final_density（判 stone 的量）在幕帘域的符号

final = min(squeeze(0.64·interpolated(blend_density(inner))), noodle)（`overworld.json:30-167`）。inner 的 range_choice 入口在 sloped_cheese<1.5625 时取 min(sloped_cheese, 5·entrances)；sloped_cheese ≈ 4·qn((depth+jag·hn)·factor)+base_3d ≈ -12.4±0.3（`sloped_cheese.json`，深海 jaggedness≈0）：

- y 192-240：inner ≈ -12.4 → ×0.64 = -7.9 → squeeze 钳 -1 → ≈ **-0.458**
- y 256-318：grad2=0 → inner = 常数 **-0.078125** → ×0.64 = -0.05 → squeeze ≈ **-0.025**（插值后仍为常数）

**vanilla 该域 final 应 d≤0 处处成立 → 幕帘是两臂把 d 抬过 0 的偏离，B1 机理上成立方向正确。且上方带（256-318）vanilla 地板值仅 -0.025：任何 ≥0.03 量级的系统性加性偏差即可翻转符号并落在 (0, 0.39] 内**——这解释了为何偏离只在海洋高空带显形（陆地该 y 域 depth 主导、|final| 大，小偏差不改号），也解释 0.39 量级（非 -64 级大错，而是小加性/小乘性系统差 + 底部带 -0.458 需 ~0.5 级抬升——两种幅度并存的形态本身是判别信号，见 §5 P3）。

### 2.3 两侧生成路径同源性核查（任务重点 3）

**结论：三路径，两种同源关系，共享错误有两个注入点。**

| 路径 | 树内容来源 | 求值语义来源 |
|---|---|---|
| C++ 生产 | 运行时 JSON：`worldgen_api.cpp:471-473`（`router->get→buildNode`）+ externalLoader `:386-391`（density_function/<dfNs>/*.json） | `density.h`（手写虚调用树）+ `density_builder.h` 解析 |
| Rust 生产 | **同一套 JSON**：`worldgen_handle.rs:147/207/215/249`（wg_dir = 调用方传入的同一 data/worldgen 树，探针口径 `versions\1.20.1\data\worldgen`，worldgen_handle.rs:120-140 契约注释） | `density.rs` + `density_builder.rs`——**文件头/行内注释自证是 C++ 的逐行移植**（density.rs:3「逐位对齐 C++ density.h」、:48「对齐 C++ YClampedGradient::clampedMap L324-330」、:176「对齐 C++ sampleImpl L411-473」） |
| Rust transpiler/DFC/GPU（env 门控，worldgen_handle.rs:221-242，默认关） | build-time 转录自同一 JSON：`generated/vanilla_density_functions.rs`（lib.rs:34）；静态抽查 initial 通道常量（0.1171875/240/256/1.5/-1.5/-0.2222）与 JSON 一致 | `dfc_backend.rs`/生成直排 + `density::transpiler_cache_2d` |

同源不对称点：
1. **JSON 是两侧唯一共享的数据源**——若 `versions\1.20.1\data\worldgen` 这份拷贝相对 vanilla jar 有转录错误，两臂忠实实现同一棵错树（共享错误注入点 ①）。
2. **Rust 求值语义是 C++ 的移植**——C++ 任何偏离 Java 的原语语义会被移植原样继承（注入点 ②）。已核对的原语（本次静态对拍未发现两臂互异）：YClampedGradient（density.h:325-330 ↔ density.rs:51-56，逐式相同）、clamp（density.h:209 ↔ density.rs:36/562）、quarter_negative（density.h:163 ↔ density.rs:43）、mul==0 短路（density.h:128 ↔ density.rs:556，数值恒等优化）、min/max 短路（density.h:129-133 ↔ density.rs:557-558，同样值恒等）、InterpolatedDF 4×4×8 网格（density.h:589-619 ↔ density.rs:261-320，结构相同）。
3. 第三路径（transpiler）是**独立**直排，恰好可作 B1c 判别对照（§4）。

## 3. B1 成立所需的共享错误形态：三个互斥子候选

### B1a：共享数据源错误（JSON 树内容 / noise_params 振幅 / biome_params 表）
两臂同读 `versions\1.20.1\data\worldgen` 同一份文件；若导出/转录时任何常量错（梯度端点、spline 值、噪声振幅、offset 表），两臂同偏 vanilla。
- 已排除面（本次静态抽查）：noise_router 顶带常量（0.1171875/-0.1171875/-0.078125/0.078125/-0.703125/240/256/-64/-40/4.0/0.64）、depth 端点（1.5/-1.5/-64/320）、factor 深海段（3.95）、sloped_cheese 结构、jagged 参数（1500/0.0）——均与 vanilla 1.20.1 已知值一致。
- 未排除面：**noise_params.json（噪声振幅表，`worldgen_api.cpp:382`/`worldgen_handle.rs:202-203` 共享）、offset/factor spline 全量、continents/erosion 噪声参数、biome_params.json**——未逐项对 jar。
- **判别方法（廉价静态探针，主会话可执行）**：从正版 1.20.1 jar 解出 `data/minecraft/worldgen/noise_settings/overworld.json` + `density_function/overworld/**` + `noise/**`（振幅），与仓库 `versions\1.20.1\data\worldgen\data\minecraft\worldgen\` 逐文件 diff。**任何 diff 即 B1a 直接候选证据；零 diff 则 B1a 整体证伪**（一次探针裁决全子候选）。

### B1b：移植共享语义差（C++ 原语偏离 Java，被 Rust 逐行移植继承）
两臂求值语义同错（对 vanilla/Java 偏）。互斥子形态（各自判别）：
- **B1b-1 插值 cell 网格域/边界**：InterpolatedDF 网格 `minY/height` 绑定、越界钳位（density.h:516-522 ↔ density.rs:300-304——注意此处有**非逐位移植痕迹**：C++ 条件含 `cx >= GX` 冗余项，Rust 只到 `gx-1`，两者最终钳位效果相同但路径不同）。若网格 y 锚点/8 格对齐错，幕帘剖面呈 **8 格台阶锯齿**。
- **B1b-2 噪声采样键/共享表错位**：DoublePerlinNoiseSampler 振幅/ octave 或 FlatCache/Cache2D 键语义错（如 flat_cache y=0 角点采样错行）。此类错全局同偏，但只在地板级负值域翻号 → 与幕帘分布吻合。
- **B1b-3 squeeze/blend 应用序差**：squeeze(0.64·interp(·)) 的乘/钳次序若与 Java 不同（如先 interp 后 0.64 时的钳位域），顶部带 -0.025 地板差最大。
- **判别方法**：三方同列 density 剖面对比——C++ `WG_DBDEBUG_X/Z`（worldgen_api.cpp:924-938）vs Rust `bin-diag/qaq1_initdensity_cost.rs`（头注释口径已是 seed 8576294172403134396 / region (200,200)）改造列输出 / `bin-diag/b1_density_probe.rs`。**两臂剖面若逐位相同但非 vanilla → B1b（共享语义错）实锤方向；两臂剖面不同 → B1 整体证伪（幕帘非共享树/语义错所致）**。这是 B1 的总判别，优先级最高。

### B1c：第三路径不对称（读数来自 transpiler/DFC/GPU 引擎而非运行时树）
若幕帘采集时 WG_TRANSPILER/WG_DFC/WG_GPU_DENSITY 任一开启，读数出自 generated 直排代码，与运行时树（B1a/b 的对象）不同源。
- **判别方法**：同列 A/B——引擎 env 开/关各采一次 WG_DBDEBUG 等价剖面（Rust 侧 `WG_DFC`/`WG_TRANSPILER` 现成，worldgen_handle.rs:221-228）；差异为零则 B1c 证伪。generated 文件静态抽查已见 initial 通道常量正确（§2.3），先验偏低但成本低。

## 4. 可证伪预测

- **P-B1-1**（B1a）：jar diff 出现 ≥1 处差异，且该项在深海 y192-318 的敏感度方向为正（抬 d）。零 diff → B1a 证伪。
- **P-B1-2**（总判别）：C++ 与 Rust 幕帘列（如 (195,z)/(198,z)）y 180-320 final 密度剖面**逐位相同**（若 B1 共享错误为真且两臂都走运行时树）；**剖面不同 → B1 整域证伪**，幕帘成因须转 B2/B3/B6。
- **P-B1-3**（子候选形态判别，建立 P-B1-2 相同之上）：剖面在 y=240/256 折点位置偏移或深度项整体缺失（近似常数 0.039/−0.05 平台）→ B1a 树内容错；剖面呈 8 格台阶锯齿 → B1b-1 网格锚错；剖面 = vanilla + 小常数加性偏移 → B1b-2/B1b-3；仅某引擎开启时出现 → B1c。
- **P-B1-4**：幕帘若真为 B1 共享错误，则偏离应在**所有低 continentalness 深海列**出现（非仅 (200,200) 4×4），且与 jagged/base_3d 噪声相位无关的空间图案（run 边界由 grad2/常量几何决定，非噪声决定）。

## 5. 主会话可执行判别探针模板（不执行）

现状可引用：`.tmp\p2full\*.blocks`（4×4 @ chunk(200,200)，seed 8576294172403134396，dll ec4a9aed）——块级已证明幕帘存在，但**无 density 数值剖面**。

1. **总判别（B1 生死探针，对应 G1+P-B1-2）**：
   - C++：`block_probe`（seed 三查后）+ env `WG_DBDEBUG=1 WG_DBDEBUG_X=195 WG_DBDEBUG_Z=<z>` 采 chunk(192? 对齐 195 所在 chunk) 多列，y 全域 %.6f 递减（worldgen_api.cpp:924-938 格式对齐 cns 先例）。
   - Rust：改 `bin-diag/qaq1_initdensity_cost.rs`（或 b1_density_probe）为同格式列输出，同 seed 同列。
   - 判读：两臂逐位 diff（预期 |diff|=0 若 B1 共享错；>0 即 B1 证伪）。
2. **B1a 静态 diff**：解包 1.20.1 官方 jar → diff `data/minecraft/worldgen/{noise_settings/overworld.json, density_function/overworld/**, noise/**}` vs `versions\1.20.1\data\worldgen\data\minecraft\worldgen\`（注意 noise_params.json 是仓库合成表，以 jar `noise/*.json` 振幅为基准）。
3. **B1c 引擎 A/B**：同列在 `WG_DFC=1` / `WG_TRANSPILER=1` / 默认 三态各采一次，互 diff。
4. **vanilla 参照（G1 缺口）**：Java DensityProbe（或 mixin RETURN dump，docs/03:95 警示 cns 反射 get(0) 坑）采同列 finalDensity + initialDensityWithoutJaggedness，格式对齐 WG_DBDEBUG。
5. 控制项：沿用 map §4 结构剥离 + §3.4 G4（granite/copper_ore 写者不在 B1 域内解释——y192-318 超 oreVein 域 `ore_vein.h:46`/`ore_vein.rs:46`，B1 只解释 stone 本体与 0<d 数值）。

## 6. idk 诚实声明

- 幕帘 granite/copper_ore 交替 run 的写者 idk（超出 B1 域，见 map §5 P1/G4）。
- 两臂幕帘列 density 是否逐位相同 idk（未采集；P-B1-2 探针前 B1 不可升格）。
- 幕帘实测上界 0.39 与 0.0390625/0.390625 的耦合是否结构性的 idk。
- vanilla 该域「无 stone」引用自 convergence 裁决，本臂未独立复验参照 .blocks（map §5 P4 保留，未做）。
- C++ 运行时 worldgenDir 实参是否恒指 `versions\1.20.1\data\worldgen` 同目录：由调用方传入（worldgen_api.cpp:345），本臂未追 Java 装载侧——若两臂实际读不同 JSON 拷贝，B1a 的「共享」前提削弱，需先核实（廉价：block_probe 日志或装载侧 grep）。
- 本产物为纯静态审查，无数据层验证轮次；retry 计数不适用（未进入验证循环）。
