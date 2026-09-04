# B2 候选：est / without_jaggedness 扫描差能否解释 NOISE stone 幕帘（260904-06，fan-out b2 臂，core.worker）

> status: candidate-pending（worker 分析产物，待主会话收敛 + judge；不自我审查，不下最终裁决）
> 课题：幕帘（y≈192-318，vanilla 无 stone 海洋域出现 stone run + ~16 间距水口袋，C++/Rust 共有 vanilla 偏离）
> 本臂问题：est（surface_height_estimate）扫描差是成因还是只能调制形态？m2「est 本体六维零语义差」的适用域边界在哪？
> §9.7 三要素：**载体** = 静态代码审查 + 既有验证记录复核（docs/07:1116-1168、convergence (d) m2、writer-verdict），无本轮新采集数据层证据；**覆盖面** = est 计算路径（C++ aquifer.h/surface.h ↔ Rust aquifer.rs/worldgen_handle.rs/surface_rules.rs）+ est 进入 aquifer/surface 的全部入口 + 07 篇验证域口径复核；**与既有口径可比性** = 直接引用 260903-12/13 est Full 验证（Java mixin vs Rust WG_EST_DUMP，同 seed 8576294172403134396）与本线 (200,200) 4×4 域同源，但 **C++ 臂 est 在该域无直接 Java 对比**（见 §3 域边界）。
> 只读分析：未跑命令、未改源码。

---

## 0. 一句话判定

**B2（est 扫描差）不能解释幕帘 stone 本体的存在**——幕帘 stone 由 `density > 0.0` 在 est 被咨询之前就判掉（aquifer.h:74 / aquifer.rs:295），est 只影响 d≤0 格的流体位与边际 solid；且「est 偏高→高位水口袋」这一现象本身就是 mod 臂 initial_density_without_jaggedness 在高处 >0.390625 的**直接症状**，归因上游 B1。B2 最多解释口袋间距/液面形态的微调，不能解释「幕帘存在」。

---

## 1. est 如何进入 aquifer fluid 判定（任务 1）

### 1.1 est 的两个消费入口（全链盘点）

est 在每臂恰好有两条消费链，没有第三条：

1. **aquifer 流体位**：`getFluidLevel`（C++ aquifer.h:290-319 / Rust aquifer.rs:416-442）13 邻域各调 `estimateSurfaceHeight(l,m)`（aquifer.h:303 / aquifer.rs:426）→ `getFluidBlockY`（aquifer.h:329-353 / aquifer.rs:444-460）用 `surface_height_estimate` 算 `ii = est + 8 - blockY` → `f = lerp_clamp2(ii, 0..64, 1..0)` → `h = map2(f,1,0,-0.3,0.8)`、`k = map2(f,1,0,-0.8,0.4)` → `d = g - k`、`e = g - h`（aquifer.h:339-345 / aquifer.rs:450-455）→ `if e>0 → default_fl.y; else if d>0 → min(est, q)`（aquifer.h:347-351 / aquifer.rs:457-459）；以及 `getNoiseBasedFluidLevel` 直接 `min(surfaceHeightEstimate, q)`（aquifer.h:365 / aquifer.rs:471）。
2. **surface 起点判定**：SURFACE 阶段 `blockY >= estimateSurfaceHeight()`（06 篇:28/:108-109）+ 四角插值 +16 角参数（surface.h:177-194 / surface_rules.rs:411-437 / worldgen_handle.rs:582-585）。——此链属 B4/heightmap 域，m2 已裁决（convergence §已裁决 (d)），本臂不重复。

### 1.2 est 能否把「本应水」判成「本应 stone」？

**能，但仅限 d≤0 的边际格，且幅度受 calculateDensity 上限约束**。机制链：est 偏移 → `f` 平移（每 +8 → f 平移 0.125）→ `d/e` 符号可能翻转 → 流体位从 default(63)/noise-based 变 -32512（air，aquifer.h:352 / aquifer.rs:459）→ 邻居流体位差 `|fl.y - fl2.y|` 改变 → `calculateDensity`（aquifer.h:238-276 / aquifer.rs:379-401）的 `q`/barrier 项改变 → `e = d_dist * calcDensity`（aquifer.h:121 / aquifer.rs:326）改变 → **`density + e > 0` 边际翻转 → 返回 -1（stone）**（aquifer.h:126/:132/:137 / aquifer.rs:327/:333/:338）。

但两条硬边界：

- **幕帘 stone 本体（0<d≤0.39）不经过这条链**：`apply` 第一行 `if (density > 0.0) return -1;`（aquifer.h:74 / aquifer.rs:295）——这些格在 est 被读之前已判 stone。**est 对幕帘 stone 零贡献，无论 est 错多少。** 幕帘 stone 存在性的唯一解释域 = density 本身被抬到 >0（B1）或 classify 前置差。
- **边际翻转要求 `|density| < |e| ≤ ~2·d_dist`**（calcDensity 上限：lavaWater 恒 2.0（aquifer.h:275），一般 `2·(r+q)`，r=barrier∈[-1,1]、q 有界）——只影响 density 绝对值很小的格壳层，不能制造 y 192-318 的连续 stone run。

### 1.3 est 在海洋列若估算偏移，能否改变幕帘 run 边界？

**不能改变 run 本体边界，只能移动水口袋液面**。幕帘 run 边界 = density 过 0 位置（B1）；est 偏移改变的只是：① 口袋内 water↔air（bs = `getBlockState(blockY)`：blockY ≥ fl.y → air，aquifer.h:23-25 / aquifer.rs:FluentLevel）；② run 边缘 d≤0 且 |d| 极小的壳层格 air↔stone。**与 m2 裁决「est 不产 stone 本体」一致，本臂确认并收窄为定量边界（§1.2）。**

---

## 2. est 零语义差结论的适用域边界（任务 2）

### 2.1 前提勘误（§15.4 意义上的事实修正，非结论取代）

scout map §2-B2/P3 声称「07 篇 est 全绿只在 -288/-8248 旧域验证过，(200,200) 域未复验」——**与 docs/07:1116 矛盾**：260903-12 est shared 裁决的 Full 验证域恰是「**同 seed 8576294172403134396 同 region (200,200) 64 chunks**」（Java `ChunkNoiseSampler.estimateSurfaceHeight` mixin RETURN dump vs Rust `WG_EST_DUMP` 角值，07:1116-1117）；260903-13 修复后「Java est 角列对比 off/shared 各 **256/256 一致 0 diff**」（07:1160），敏感 chunk (201,200) 也在域内。**即 Rust est 角列在本幕帘所在区域已被 Java 直接背书。** -288/-8248 是 *更早的* C++ est 验证域（docs/04:98、docs/10:498-517），不是唯一域。

### 2.2 修正后仍然未覆盖的域（m2/07 结论的真实边界）

| # | 未覆盖项 | 为什么 corner 验证盖不住 | 对幕帘的相关性 |
|---|---|---|---|
| U1 | **C++ 臂 est @ (200,200)** | 07:1116/1160 只对比 Java↔**Rust**；C++ est 与 Java 的直接对比记录在 -288/-8248/20000 域（docs/10:498-517） | 中——幕帘是 C++/Rust **共有**偏离，若 C++ est 在新域偏，属双臂共有候选的唯一入口；但 C++/Rust est 代码同源且 07 篇四臂 hash 同值（§2.3），风险低 |
| U2 | **内部列（非 chunk 角列）**：aquifer 13 邻域查询列 = blob 网格中心 ±{0..3}·16 偏移（aquifer.h:291-294 OFFSETS），量化 `(x>>2)<<2` 后落到任意 biome 格 | 角值 dump 只覆盖 chunk 四角列（worldgen_handle.rs:582-585）；同一 est 函数、不同输入列 | 低——同函数同量化，无机制理由只在内列错；但形式上未验证 |
| U3 | **est 高值区（y>192 的首命中点）**：若 (200,200) 角列 mod est 实际 ~48-64（海底附近），则扫描高位段 (192-318) 的 without_jaggedness 值从未被角值对比「 exercising」 | 角值对比验证的是**结果值**相等；若两侧都在低位命中，高位段数值差被掩盖 | **高**——这正是幕帘域；但注意：若 mod est 实际 >200（§4 P1 推论），则高位段已被角值对比覆盖且 = Java → est 无罪、DF 抬升实锤（B1） |
| U4 | `getFluidBlockY` 的 erosion<-0.225 && depth>0.9 分支（d=e=-1，aquifer.h:335-337 / aquifer.rs:447-448）在深海低 continentalness 域的行为 | 该分支是「内陆深湖」语义，(200,200) 深海 depth 大概率 <0.9 未走通 | 低——两臂同构，不构成共有偏离入口 |
| U5 | 四角插值特定 corner 组合（surface 侧） | m2 + 07:1159-1160 四臂 hash + Java 角列 0 diff 已覆盖 (200,200) 64 chunks | 已闭合，不列缺口 |

**(200,200) 域是否落在未验证区？——Rust est 角列：否（07:1116）；C++ est 与全部内列：形式上是。** 结论：m2「est 本体六维零语义差」**在该域对 Rust 臂有 Full 直接背书，对 C++ 臂是同源代码的外推**（候选 status 应如此限定，不是全称命题）。

### 2.3 旁证：est 路径 A/B 在本域输出零差

260903-13 翻默认验证：off/shared × L2 开关**四臂 hash 完全一致 `f2b1a3932c6e589e`**（07:1159），L2 是跨 chunk 精确值缓存（aquifer.rs:349-375，淘汰只影响命中率）——即在本域，est 计算路径选择对最终 chunk 输出**逐位无影响**。est 若要成为成因，必须是「est 函数对 Java 系统性错」而非「两臂 est 路径差」；而 Rust est 已被 Java 背书（§2.1），C++ est 同源。

---

## 3. 判定（任务 3）

### 3.1 B2 能否解释「幕帘存在」？

**不能（证伪方向，非最终裁决）**。三段独立理由：

1. **判定序**：幕帘 stone（0<d≤0.39）由 `density>0` 前置判定（aquifer.h:74 / aquifer.rs:295），est 不参与。幕帘「存在性」∈ B1 解释域（scout §1.3 推论同向：三方代码均无「0.39 stone 上界」分支）。
2. **验证域**：Rust est 在幕帘所在 (200,200) 域已被 Java Full 背书 0 diff（§2.1）；est 路径 A/B 四臂同 hash（§2.3）。共有偏离需要一个「两臂同时对 Java 错」的 est 差——目前无任何证据入口，且 C++ est 同源外推为负。
3. **高位水口袋反证 est 无辜（ symptom 归属 B1）**：口袋 water @ y≈197..277（convergence 新世界图景）要求 `fl.y > blockY`，而 default 流体位 = 63（aquifer.h:81 / aquifer.rs:298）；`e>0` 分支也只回 default（aquifer.h:347-348 / aquifer.rs:457）。唯一能产出 y>197 流体位的是 `d>0` 分支的 `min(est, q)`（aquifer.h:350/:365 / aquifer.rs:458/:471）——**⇒ mod 臂 est ≥ ~200 ⇒ initial_density_without_jaggedness 在 y≥200+ 某处 >0.390625**。vanilla 同列无高位口袋 ⇒ vanilla est < ~200（或 vanilla floodedness/margin 另差，B3 域）。**无论哪种，偏离都在 est 的输入（without_jaggedness DF 值，B1）或 floodedness 边际（B3），不在 est 扫描机制本身（B2）。** est 高只是 B1 的体温计。

### 3.2 B2 的残 余 解释力（不证伪的部分）

est 机制内的真实自由度只剩：① U1（C++ est 新域未直接验）；② U2（内列未验）；③ U4 分支。它们最多解释**口袋液面 ±8/±16 的位置微差与 |d| 极小壳层的个别格翻转**，与「run 数量/范围/幕帘存在」差 2 个量级以上。B2 应裁为：**排除出「幕帘成因」候选集，降级为「口袋形态调制项」+ C++ est 域复验尾巴（低成本补采即可闭合 U1/U2）**。

### 3.3 可证伪预测（供主会话/judge 攻击）

- **P-B2-1（est 三方相等预测）**：在 chunk(200..203, 200..203) 角列 + 至少一个幕帘口袋内列，C++ WG_ESTDUMP = Rust WG_EST_DUMP = Java RouterProbe ESH（同 seed 三查后）。若 Rust=Java（已知）而 C++ ≠ Java 且差 ≥8 → B2 复活（U1 实锤）；若三方相等 → B2 对本课题彻底出局。
- **P-B2-2（est 扰动不动幕帘本体）**：对 mod 臂把 est 强制改为 vanilla 值（或把 0.390625 阈值扰动 ±0.05 重生成），stone run 的 y 范围与 run 数量不变，只有水口袋液面 y 与个别壳层格变。若 run 本体随 est 改变 → 本报告判定错，B2 复活。
- **P-B2-3（高位 without_jaggedness 抬升预测，反向攻 B1）**：幕帘列 `initial_density_without_jaggedness` 在 y≈200-320 存在 >0.390625 的采样点（mod 臂），vanilla 同列同 DF 全 ≤0.390625。若 mod 臂该 DF 在高位全 ≤0.390625 但口袋仍在 y>197 → 本报告 §3.1-3 的 min(est,q) 推理链有错，需回查（例如 blob 网格 q 计算或 bs 判定另有入口）。

### 3.4 判别探针模板（供主会话执行，本臂不执行）

```
# E-B2-1 est 三方 dump（闭合 U1/U2，≈P-B2-1）
# 前置：seed 三查（AGENTS 探针铁律；seed=8576294172403134396）
C++ : block_probe + WG_ESTDUMP=<out>（worldgen_api.cpp:1090-1093），范围 chunk(199..204,199..204)
Rust: WG_EST_DUMP=<out>（worldgen_handle.rs:587-601），同范围（注意：现只 dump chunk 四角，
      内列需小改探针或复用 va.aq.estimate_surface_height 直采——主会话决策，本臂不动码）
Java: RouterProbe ESH 同点（docs/10:718 先例；或 cns mixin RETURN dump，docs/07:1116 先例）
比对口径：全部量化列 (x>>2)<<2 对齐后逐值 diff；输出差列清单 + 各列 biome/continentalness。

# E-B2-2 without_jaggedness 高位剖面（判 P-B2-3，实为 B1 输入证据）
C++ : WG_SURFDUMP(_X=..,_Z=..)（worldgen_api.cpp:962-999，含 initialDensity 分量）@ 幕帘列 (195,z)/(198,z) 及 vanilla 无幕帘邻列
Rust: bin-diag/qaq1_initdensity_cost.rs 口径改造（scout §3.2：seed/region 已对）输出 y=320..-64 剖面
Java: DensityProbe/mixin dump（缺口 G1，需补采；cns 反射坑见 docs/03:95）
判据：mod 两臂高位是否存在 >0.390625；vanilla 同列对照。

# E-B2-3 est 扰动 A/B（判 P-B2-2）
Rust: WG_EST_SHARED=0 vs 1（已有 07:1159 四臂同 hash 可直接引用）+ 阈值扰动需临时改码（禁——
      本臂只读；扰动实验列为「需改码」项交主会话/swe 决策，或用 WG_ESTDUMP 数值反证替代）
C++ : 同理暂无阈值门控，只做 E-B2-1 数值对比。

# E-B2-4 口袋液面归因（B2↔B3 分界）
C++ : WG_AQFDUMP + [AQF-IN]/[AQF]/[AQF-e]（aquifer.h:71-124）@ 幕帘口袋列 y 190-290
Rust: bin-diag/aquifer_probe.rs / aquifer_apply_breakdown.rs
判据：口袋列 fl.y 是否 = min(est,q)（B1 症状）还是 d/e 符号翻转所致（B3）；
      [AQF-e] 行 density+e 值量级对照 §1.2 边界（|d|<|e| 才可能边际翻 stone）。
```

---

## 4. 诚实声明 idk / 限制

- **idk-1**：C++ 臂 est 在 (200,200) 域与 Java 的直接对比数据不存在于仓库（07 篇只做 Java↔Rust）——U1 是本判定唯一的实质敞口，闭合成本 = 一次 E-B2-1 采集。
- **idk-2**：幕帘列 mod 臂 est 实际值未测（§3.1-3 的「est ≥ ~200」是从水口袋存在反推的必然结论，逻辑上强；但 est 具体值/q 值/fl.y 逐口袋对照未采，E-B2-4 未跑）。
- **idk-3**：vanilla 侧 est/without_jaggedness 在 (200,200) 完全无数据（scout 缺口 G1 未补）——P-B2-3 的 vanilla 半边目前不可判。
- **idk-4**：scout map P3 表述（「(200,200) 域未复验」）与 docs/07:1116 冲突，本臂以 docs 正文 + git 可溯验证记录为准做勘误；若 07:1116 的「region (200,200)」与幕帘 chunk(200,200) 非同一坐标系口径（chunk vs region 起点），需主会话一次核对（廉价：读 260903-12 verdict 头部 §9.7）。
  - **✅ idk-4 已闭合（主会话，260904-06）**：`est-shared-verdict-260903-12.md:7` §9.7 明载「覆盖面 = seed 8576294172403134396、**chunk region (200,200) 8×8=64 chunks**、每 chunk 4 角列」——即 chunk 坐标 (200,200) 起的 8×8，幕帘 4×4 @ chunk(200,200) **完全落在验证域内**。U1 敞口确认为「仅 C++ 臂 est 无 Java 直接对比」（Rust 臂已背书）。
- 本产物无新数据层证据（纯静态审查 + 记录复核，Degraded 分层），按 §9.4 不消耗也不重置 evidence saturation 计数；所有「判定」为 worker 意见，status 停在 candidate-pending，confirmed/淘汰由主会话收敛 + judge + 用户拍板。

## 5. 引用清单（文件:行）

- C++：`versions/1.20.1/cpp/worldgen/src/aquifer.h:74,80-82,121,126,132,137,142-164,238-276,290-319,329-353,355-366`；`surface.h:177-194`
- Rust：`WorldgenRust/src/aquifer.rs:294-341,343-377,379-401,416-442,444-460,462-472`；`worldgen_handle.rs:556-611`（off 臂扫描 `571-577`、四角 `582-585`、est dump `587-601`）；`surface_rules.rs:411-437`
- 验证记录：`versions/1.20.1/docs/07-block-pipeline.md:1110-1168`（est Full 验证域 = seed 8576294172403134396 / region (200,200) / 64 chunks；256/256 0 diff；四臂 hash f2b1a3932c6e589e）；`docs/04-aquifer.md:63-100`；`docs/06-surface-rules.md:66-121`；`docs/10-timewise-archive.md:475-517,682-684,718`
- 前置裁决：`.artifacts/lossless-accel/writer-verdict-260904-06.md`；`.investigations/lossless-accel/fanout-writer-260904-06/convergence-260904-06.md`（(d) m2）
- 勘探地图：`.investigations/lossless-accel/curtain-scout-260904-06/map.md`（§1.3/§2-B2/P3——P3 表述本臂勘误，见 §2.1/idk-4）
- 知识库：`knowledge/discovered/compiler-idioms.md` 发现 #10（est 扫描 off-by-one/角 +16，已修复闭合）+ 易错点清单（est 4 角插值）
