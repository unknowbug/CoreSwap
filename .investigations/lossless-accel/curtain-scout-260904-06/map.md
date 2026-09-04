# NOISE stone 幕帘（0<d≤0.39，y≈192-318）勘探地图（260904-06，recode.scout）

> status: draft（勘探产物，只含代码位置与互斥候选盘点，不下结论）
> 范围：「C++/Rust 共有 vanilla 偏离」新调查线的前置机理地图 + 探针盘点
> 前置裁决（已定，勿重查）：`.artifacts/lossless-accel/writer-verdict-260904-06.md` + `../fanout-writer-260904-06/convergence-260904-06.md`
> 约束遵守：只读勘探，未修改任何源码/数据；本文为唯一产物。

---

## 1. 幕帘机理地图：NOISE 阶段 stone 判定链（文件:行）

### 1.1 C++ 链（versions\1.20.1\cpp\worldgen\src）

| 步骤 | 位置 | 说明 |
|---|---|---|
| densityBuf 填充 | `worldgen_api.cpp:884-916` | 逐点 finalDensity 采样（`:912` 生产路径 `h->finalDensity->sample(fpos)`；`:893` GPU / `:897` DFC 旁路）+ `:915` Beardifier 加法 → `densityBuf`（`:916`） |
| **stone/air/water 判定** | `aquifer.h:70-74` | `apply(..., density)`：**`if (density > 0.0) return -1;`** → 调用方写 stone |
| 调用方（3b aquifer+oreVein 环） | `worldgen_api.cpp:1033-1048` | `:1040` aquifer.apply；`:1041` 返回 <0 且有 oreVein → oreVein.apply；`:1043` 仍 <0 → `block = stone`（`:775` 定义） |
| ore_vein 域限制 | `ore_vein.h:46` | `if (y < -60 \|\| y > 50) return -1;` —— **y 192-318 处 oreVein 必返回 -1**，幕帘中的 granite/copper_ore **不可能来自 C++ oreVein**（见 §5 待深入点 P1） |
| est 扫描（0.390625 阈值） | `aquifer.h:142-159`（注释+扫描）、`surface.h:178-190`（同语义） | 从顶向下步长 8，`initialDensityWithoutJaggedness > 0.390625`（`aquifer.h:159`、`surface.h:188`） |
| aquifer 流体决策 | `aquifer.h:76-137` | d≤0 时：默认流体 y<-54 lava / 否则 water（`:80-81`）；floodedness/spread/barrier 边际 `density+e/g/h > 0` 判 solid（`:126/:132/:137`） |
| heightmap | `worldgen_api.cpp:1045` | `block != air` 即计入（含水列也抬 heightmap——与 Rust `d>0` 判据不同，见 §2 候选 B4） |

### 1.2 Rust 链（WorldgenRust\src）

| 步骤 | 位置 | 说明 |
|---|---|---|
| 宏观密度+分类 | `terrain.rs:247-289`（`fill_chunk`） | `:273` 密度采样（宏观网格插值或逐点）→ `:276` Beardifier 加法 → `:277` `aqua.classify(x,y,z,d)` |
| **stone/air/water 判定** | `terrain.rs:222-233`（`VanillaAquifer::classify`） | **`if d > 0.0 { return BlockKind::Rock; }`**（`:223`）→ `worldgen_handle.rs:543` Rock → stone id |
| aquifer 本体 | `aquifer.rs:295`（`density > 0.0 → -1`）、`:327/:333/:338`（`density+e/f/g/h > 0` 边际）、`:457-458`（getFluidBlockY：est 与 floodedness） | 与 C++ `aquifer.h:70-137` 同构 |
| ore_vein 域限制 | `ore_vein.rs:46` | 同 C++：y>50 返回 -1 |
| est 扫描（0.390625） | `worldgen_handle.rs:571-577`（独立路径）、`surface_rules.rs:411-430`（对齐注释）、`aquifer.rs:366`（getFluidLevel 13 邻域用） | `init.sample(...) > 0.390625`，步长 8 |
| heightmap/surface 起点判据 | `terrain.rs:279`（`d > 0.0` → top）、`worldgen_handle.rs:556/610`（surface 用 heightmap） | C++ 是 `block != air`（含水），Rust 是 `d > 0.0`（不含水）——**已知分叉（convergence (d) m2 裁决），仅开放海洋列签名** |
| surface 规则树 | `surface_rules.rs`（`build_overworld_rule`，est/规则应用 `:304/:311/:1318/:1390`） | 对应 C++ `surface.h:507-667` |

### 1.3 vanilla Java 对应逻辑（versions\1.20.1 下无 Java 源，引用线索在 docs）

- **stone 判定同构**：vanilla 亦是 `finalDensity > 0 → default_block(stone)`，否则 aquifer fluid（docs/01-architecture.md:39 `ChunkNoiseSampler` ↔ `density.h`+`worldgen_api.cpp`；docs/04-aquifer.md:63-68 `estimateSurfaceHeight` 引用）。
- est 阈值 0.390625 为 1.20.1 硬编码（docs/03-density-functions.md:77）；Java 位置引用 `ChunkNoiseSampler.estimateSurfaceHeight`（docs/10-timewise-archive.md:484）、`MaterialRules.java:488-516/567-572`（docs/10:682-684）、`NoiseChunk.java:174`（worldgen_handle.rs:568 注释）、`AquiferSampler.Impl.apply`（aquifer.h:68 注释）。
- Java 源/探针载体在仓库外部（gradle 探针工程，只读引用；AGENTS §五）。
- **关键推论（勘探级观察，非结论）**：三方的 stone 边界都是 `d > 0`，**没有任何一方的代码里存在「0.39 stone 上界」**——0.390625 只出现在 est 扫描（决定 est → aquifer fluid_level + surface 起点）。「0<d≤0.39」是测量口径对幕帘区 density 数值的刻画，不是某段分支条件。因此「vanilla y≥201 无 stone」意味着 vanilla 该区域 d≤0，幕帘偏离必须由「C++/Rust 密度数值在该区域被抬到 >0」或「d≤0 但 aquifer 误回 solid」解释——分别对应 §2 候选 B1/B3。

## 2. 分叉候选盘点（互斥并列，不下结论；worker/fan-out 裁决）

> 幕帘是复合结构（stone run + ~16 间距 aquifer 水口袋，convergence 新世界图景）。候选按「解释什么」分层：B1/B2/B3 解释 stone 本体与口袋，B4 解释作画边界，B5 是对比控制项。

- **B1 final/initial density 数值差（树内容或采样路径）** —— 幕帘 stone 本体的首选解释域。
  - 位置：C++ `density.h`（插值/样条）、`density_builder.h`（buildNode）+ `worldgen_api.cpp:912`；Rust `generated/vanilla_density_functions.rs`（transpiler 直排）+ `density.rs` + `terrain.rs:273`。两侧树内容同源自同一套 JSON→代码转录，**共享转录错误可同时命中两臂**。
  - 判别探针：三方同列 density 剖面对比（C++ `WG_DBDEBUG` vs Rust `b1_density_probe`/`debug_density` vs Java DensityProbe cns 反射 dump）@ 幕帘列 y 180-320；现有 cns dump（`versions/1.20.1/data/vanilla_density_*_cns.txt`）均不在 (200,200) 区域（§3 缺口 G1）。
- **B2 initial_density_without_jaggedness / est 扫描差** —— 不产 stone 本体（est 只影响 fluid_level 与 surface 起点），但决定幕帘上水口袋的 ~16 间距形态。
  - 位置：C++ `aquifer.h:142-159`、`surface.h:178-190`；Rust `worldgen_handle.rs:571-577`、`surface_rules.rs:411-430`、`aquifer.rs:360-366`。07 篇 L1116 记录 est 两侧曾在 -288/-8248 域全绿——**但 (200,200)/8576 域未复验**。
  - 判别探针：`WG_ESTDUMP`（C++ `worldgen_api.cpp:1090`）vs `WG_EST_DUMP`（Rust `worldgen_handle.rs:587`）@ chunk(200,200) 四角 + Java RouterProbe ESH 同点。
- **B3 aquifer fluid 判定差（d≤0 时误回 solid，或水口袋位置差）**。
  - 位置：C++ `aquifer.h:70-137`（`density+e/g/h>0` 边际、`getFluidBlockY :328-377`、`getWaterLevel :278+`）vs Rust `aquifer.rs:295-338`、`:457-458`。若两侧共享同一处转录差（如 floodedness/spread 噪声参数或 blob 网格），可同时偏。
  - 判别探针：C++ `WG_AQFDUMP`/`[AQF]/[AQF-e]/[AQF-IN]`（`aquifer.h:70-124`）vs Rust `bin-diag/aquifer_probe.rs`、`bin/aquifer_bp_probe.rs`、`bin/aquifer_wl_probe.rs`；对比水口袋 (o,p,q) 与 y 序列。
- **B4 surface 规则树内容差 + heightmap 填充判据分叉** —— 解释两臂幕帘上作画边界微差（43/11）与 (244) 列 sand/gravel；不解释 stone 本体。
  - 位置：C++ `surface.h:507-667`（build_overworld_rule）+ `worldgen_api.cpp:1045`（heightmap 判 `block != air`）；Rust `surface_rules.rs` + `terrain.rs:279`（判 `d > 0.0`）。后者已有 m2 裁决（convergence (d)：仅顶块=水的开放海洋列两臂起点不同）。
  - 判别探针：`WG_SURFTRACE(_X/_Z)`（`worldgen_api.cpp:59`）vs Rust `WG_SOUL_CTX_DUMP`（`surface_rules.rs:470`）+ soul_selector_probe 单列规则轨迹。
- **B5 structures/Beardifier 混杂（judge C4 警示的正式化，控制项而非成因候选）**。
  - 证据行：**C++ 臂跳过 structures**（`feature_loader.h:5`「structure 部分跳过」、`worldgen_api.cpp:1585`）；**Rust 臂 features 可跳**（`worldgen_handle.rs:620`）且 feature_loader 同构；**唯一可能写结构的是 vanilla 参照臂**。
  - 方向性排除（廉价验证已隐含）：幕帘 y 192-318 的 vanilla「无 stone」不能由「C++/Rust 缺结构 beardifier」解释——beard 只加正密度（`beardifier.h:98-99` structureWeight≥0），vanilla 有结构 beard 仍无 stone；但 (195,199) 深 dirt @ -59..-42 可由结构写入，三方对比 MUST 剥离（§4）。
- **B6 采集错位类（工具性核对项，非机制）**：seed/坐标语义三查（AGENTS 探针铁律；三套坐标语义不同：RouterProbe floor 对齐 / -biomeDump (px<<2) / WG_COMPDUMP 原始块坐标）。任何三方数值对比第一动作。

## 3. 探针盘点

### 3.1 现有可用（C++，block_probe/worldgen_api，env 门控默认关）

| 探针 | 位置 | 幕帘用途 |
|---|---|---|
| `WG_DBDEBUG(_X/_Z)` | worldgen_api.cpp:924-938 | **指定列 densityBuf 全剖面**（y 递减 %.6f，含 beard，不经 aquifer/surface）——幕帘 d 值直接证据 |
| `WG_COMPDUMP(_X/_Z)` | worldgen_api.cpp:940-960 | 指定列全部 router 组件（barrier/fluid/vein 等，无插值）——B3 输入侧 |
| `WG_SURFDUMP(_X/_Z)[_Y]` / `WG_SURFDUMP_SCAN` | worldgen_api.cpp:962-999 | initialDensity+finalDensity 剖面、est 值、分量（base_3d_noise/factor/depth/jaggedness 等）、y=31 正密度列扫描 |
| `WG_NOODLEDUMP(_X/_Z)` | worldgen_api.cpp:1004-1032 | noodle/cheese/layer 树 raw+树值列剖面 |
| `WG_ESTDUMP[_X/_Z]` | worldgen_api.cpp:1090-1093 | est 值 dump（B2） |
| `WG_AQFDUMP` + `[AQF-IN]/[AQF]/[AQF-e]` | aquifer.h:70-124 + worldgen_api.cpp:59 | aquifer 逐块判定轨迹（B3）；`WG_AQF_YMIN/YMAX` 限域 |
| `-biomeDump` / `-blockDump` / `-compXY` | block_probe.cpp:136/120/148 | 列 biome/单点 block/分量对比 |
| `WG_BIOMEDUMP` / `WG_FINDTOP` / `WG_FINDDUMP` / `WG_SEARCHTREE_CACHE` | AGENTS §五 | biome 判定/平局/SearchTree A/B |
| `WG_SURFTRACE(_X/_Z)` | worldgen_api.cpp:59 | surface 规则轨迹（B4） |
| `WG_CARVERLOG` / `WG_FEATURELOG` / `WG_FEATURE_SKIP` / `WG_CARVER_SKIP` | worldgen_api.cpp:1556-1623 | 结构/特征写者排查（§4） |
| 旧反射 dump 参照 | `versions/1.20.1/data/vanilla_density_*_cns.txt`（8 个文件，区域 c-18_-16 / c45_-26 / c50_33 / c1250_1250 等） | vanilla 密度列剖面**先例格式**；**无一在 (200,200) 区域** |

### 3.2 现有可用（Rust，WorldgenRust\src）

| 探针 | 位置 | 幕帘用途 |
|---|---|---|
| `WG_SKIP_SURFACE/CARVER/FEATURES/AQUIFER/OREVEIN`、`wg_set_flags` 位门 | worldgen_handle.rs:535/609/615/620、terrain.rs:231 | 分层剥离（写者消去链，verdict 已用） |
| `WG_EST_SHARED` / `WG_EST_DUMP` | worldgen_handle.rs:563-601 | est 路径 A/B + 四角 dump（B2） |
| `WG_TRANSPILER` / `WG_DFC` / `WG_GPU_DENSITY` | worldgen_handle.rs:221-236 | 密度引擎 A/B（隔离 B1 的「引擎 vs 树内容」） |
| `bin-diag/b1_noiseonly_dump.rs` / `b1_surfaceonly_dump.rs` / `b1_column_trace.rs`（`WG_B1_COLS/WG_B1_YMAX`） / `b1_density_probe.rs`（`WG_SEED`） | bin-diag 目录 | **NOISE-only 臂列剖面**——幕帘本体直接取证 |
| `bin-diag/aquifer_probe.rs` / `bin/aquifer_bp_probe.rs` / `bin/aquifer_wl_probe.rs` / `bin-diag/aquifer_apply_breakdown.rs` | — | aquifer 判定/水口袋（B3） |
| `bin-diag/qaq1_initdensity_cost.rs`（头注释：口径 = seed 8576294172403134396 / region (200,200)，initial_density_without_jaggedness） | bin-diag/qaq1_initdensity_cost.rs:3 | **已有正确口径的 init 密度采样基础**，可改造为剖面输出 |
| `bin-diag/rust_vs_vanilla.rs` / `banddiff.rs` / `gapdecomp.rs` / `densityprofile.rs` | bin-diag | 逐点 density→block 分类对比 |
| `WG_FEATURELOG`（`[FEATURE]`）、`bin-diag/b1_selector_dump.rs`（`[R-ORIGIN]`） | worldgen_handle.rs:623-625、b1_selector_dump.rs:138 | feature/selector 来源追踪 |
| `bin/debug_density.rs` / `bin/fill_density_probe.rs` / `bin/chunk_block_probe.rs` | bin | 单点/整 chunk SOLID-air-water 剖面 |

### 3.3 现有可用（vanilla Java 侧，外部 gradle 探针工程，只读引用）

- DensityProbe / RouterProbe / BlockProbe（AGENTS §五：Partial 分层）；docs/10:718 记录 RouterProbe ESH + BlockProbe EstDiagN 先例（-288 域）。
- cns 密度反射 dump 格式（data\*.cns.txt）——docs/03:95 警示：cns 反射有坑（interpolators get(0) 非 finalDensity），**用 mixin RETURN dump 更稳**（docs/07:1116 est 验证先例）。

### 3.4 缺口（幕帘三方列剖面还缺什么）

- **G1**：seed 8576294172403134396 / chunk(200,200) 区域的 **vanilla density 列剖面**（finalDensity + initialDensityWithoutJaggedness 各一），现有 cns 文件均不在该区域——需 Java DensityProbe（或 mixin dump）补采，格式对齐 WG_DBDEBUG 以便逐行 diff。
- **G2**：幕帘列（如 (195,z)/(198,z)/(199,z)）的 **aquifer 水口袋三方位置表**：mod 臂已有（E4-lite），C++ 可 WG_AQFDUMP 采，**vanilla 侧无 aquifer 事件 dump 先例**——需 Java 侧补（BlockProbe 逐块 water y 表即可，不必反射 aquifer 内部）。
- **G3**：**vanilla 结构落点清单**（chunk(200,200) 4×4 内 structure placements）——见 §4。
- **G4**：幕帘中 **granite/copper_ore 交替 run 的写者判定探针**（C++/Rust oreVein y≤50 上限排除了两臂本地成因，见 §5 P1）；消去顺序建议：WG_SKIP_OREVEIN A/B（Rust 现成）+ C++ -features 门控 A/B。
- **G5**：C++ 侧无 `d>0` 幕帘列自动扫描器（Rust 有 bin-diag 系列）；WG_SURFDUMP_SCAN 只扫 y=31（worldgen_api.cpp:990）——幕帘域需 y≈192-318 的扫描（小改探针或脚本消费 WG_DBDEBUG 输出，主会话执行）。

## 4. structures 混杂控制方案（可操作步骤，不执行）

前提证据：C++/Rust 臂均无结构写者（`feature_loader.h:5`、`worldgen_api.cpp:1585`、Rust feature_loader 同构 + verdict E1 stageMask 语义）；mod 臂 features bit1 跳过。**混杂只可能进入 vanilla 参照臂**。

1. **列出候选结构集合**（deep ocean + 深 dirt 签名指向）：fossil（dirt/bone_block/coal_ore，可达深部）、trail_ruins（dirt/gravel/stone_bricks/suspicious_gravel/sand）、ocean_ruins（stone_bricks/sandstone/gravel/magma/chest 位）。三者都有深 dirt/gravel 词汇表，与 (195,199) 深 dirt @ -59..-42 相容。
2. **取 vanilla 落点**：在 Java 探针工程用 structure placement 查询（或对参照 .blocks 逐列做「仅 vanilla 有」差集后按方块词汇表反推）确认 chunk(200,200)±邻域是否确有 placement——**无落点则 C4 警示自动解除，direct diff 合法**。
3. **词汇表归因**：对三方列剖面逐 y 差异块，先匹配 §4.1 结构词汇表；匹配的标 `structures`，不进入 NOISE/surface 归因。
4. **深度/形态判据**：幕帘 stone run 是「stone+含水层水口袋 ~16 间距」的水平连续形态；结构写入是离散团块（3-8 格团）+ 含其伴生块——形态不符的结构词汇块单独列 idk，不强行归因。
5. **主对比口径**：NOISE 幕帘本体的机制判定以 **C++↔Rust 双臂 diff**（双方都无结构写者，差异纯 NOISE/aquifer/surface）为主；vanilla 仅参与两个已控混杂的判定——① y≥201 无 stone 的存在性（stone 不在任何候选结构词汇表，混杂不侵蚀该判定）② 结构剥离后的剩余差。
6. **复核门**：若 vanilla 臂出现幕帘区间内的 dirt/gravel 之外的差异（如 deepslate/id 970 类），先过 G3 结构清单再归因。

## 5. 待深入点清单（交 worker/fan-out）

- **P1**：幕帘 granite/copper_ore 交替 run 写者 idk（§3.4 G4）——y 192-318 超 oreVein 域（ore_vein.h:46 / ore_vein.rs:46），候选 = mod 臂 feature 写入或 aquifer barrier 之外的第三写者；不排除 mod 臂专用。
- **P2**：B1 分叉后子分叉（树内容转录差 vs 插值网格 vs blend/jaggedness 应用）——符合 fan-out 触发（≥2 互斥子候选），建议并行 .bN。
- **P3**：est 两臂 (200,200) 域复验（G1/G2 采集后即可做，成本低，07 篇结论只在旧域验证过——交接纪律：不把旧域全绿当公理续推）。
- **P4**：vanilla「无 stone」判定的自证（该结论源自 mod 臂/参照对比，worker 应先独立复验参照 .blocks 该区间确为全 air/water，防 M11 类参照误读）。

## 6. 混淆评估

不适用（本线为自有 C++/Rust 源码 + docs 引用，无字节码输入）。vanilla Java 源获取走外部 gradle 探针工程（只读），不进本仓库。
