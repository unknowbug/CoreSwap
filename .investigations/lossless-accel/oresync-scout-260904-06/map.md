# recode.scout 勘探地图 — block_probe C++ 载具 ore 族 desync 课题立项（260904-06）

> status: draft（只读勘探；本文件只盘点候选与判别法，不下结论）
> 上游：NEXT_SESSION 课题 3（22653 mismatch / 98.56%）+ p4-reference-check-260904-06.md Facts 5（vanilla 高 y granite/copper 写者 idk）
> 角色契约：只读；产物仅本目录；候选互斥并列，分析解读归 core.worker。

---

## 0. 头号 idk 现状：vanilla @ (195,199) y185-231 granite/copper_ore/iron_ore（32 块）

### 0.1 已核死的事实（一手，本轮亲核）

| # | 事实 | 位置 |
|---|------|------|
| F1 | vanilla oreVein **结构上不可能**写 y>50：`VeinType.COPPER maxY=50 / IRON minY=-60`，apply 闭包内 `j = maxY - i; k = i - minY; k>=0 && j>=0` 才写块 | OreVeinSampler.java:46-51,78-79（mc_src_extract）；C++ 同构 ore_vein.h:33,39-41 |
| F2 | vein_toggle/ridged/gap 的 verticalRangeChoice 界 = VeinType 的 minY/maxY（i=-60, j=50），y>50 时 toggle=constant(0) | DensityFunctions.java:459-473（mc_src_extract） |
| F3 | granite 的 feature 写者只有 ore_granite_upper（uniform 64..128）与 ore_granite_lower（uniform 0..60）——**上界 128** | placed_feature/ore_granite_upper.json:14-21、ore_granite_lower.json:14-21 |
| F4 | copper 的 feature 写者 ore_copper / ore_copper_large（trapezoid -16..112）——**上界 112** | placed_feature/ore_copper.json:12-22、ore_copper_large.json:12-22 |
| F5 | **iron 的 feature 写者 ore_iron_upper 上界 384**（trapezoid 80..384，count 90）——jagged_peaks biome 显式引用 ore_iron_upper（biome/jagged_peaks.json:55）→ **vanilla 高 y iron_ore 有合法写者** | placed_feature/ore_iron_upper.json:12-22；jagged_peaks.json:44-67 |
| F6 | vanilla 1.20.1 overworld surface rule **无 granite/copper 输出**：stony_peaks→calcite|stone，jagged_peaks→stone|snow_block，frozen_peaks→ice/packed_ice，全部查证 | VanillaSurfaceRules.java:75-124,125-158（mc_src_extract） |
| F7 | jagged_peaks biome feature 列表含 ore_granite_upper/lower、ore_copper 等，但引用的 placed_feature 上界即 F3/F4 | biome/jagged_peaks.json:44-67 |

**要点**：iron_ore@y185-231 不奇怪（F5，合法）；**granite@217-229 与 copper_ore@高 y 在 1.20.1 数据里找不到任何合法写者**——已知机制全部排除后，剩下的候选集中在「参照数据解读」侧。

### 0.2 写者候选（互斥并列，各带判别法）

**候选 W-A：ref_check 列索引布局误读（参照解读伪影，#8 家族近亲）— 优先怀疑**
- 证据：`.tmp/p2full/ref_check_p4_260904-06.py:18` 的列索引公式 `idx = x_local*(H*16) + z_local*H + (y-MINY)`（x,z-major、y-minor）。而 C++ 内部 col 布局是 y-major：`(wy-minY)*256 + lz*16 + lx`（feature.h:88）。**该脚本对 vanilla blocks 导出格式的布局假设未经与导出器源码核对**。
- 机制：若导出实际为 y-major 而按 x-major 读，整列字节被打散重排——低 y 的 granite/copper blob 字节会"漂"到读出的高 y 槽位，产生 y185-231 的假高 y granite/copper（32 块恰好与 24+7+1 的 F3/F4 低 y blob 量级吻合）。
- 判别法（廉价，≤1 轮）：① 找 Java 参照导出器（BlockProbe/blocks 导出）源码核对索引公式；② 用同文件另读一列（如全部 air 的 y>300 段）看是否自洽；③ 对同一列用两种布局读，若布局 B 下 y180-319 全 air/无 granite 而 granite 集中在 y≤128，即坐实。

**候选 W-B：参照文件本身非 vanilla 1.20.1（seed/版本/状态混入，三查家族）**
- 证据：参照文件 `vanilla_8576294172403134396_4_200_200.blocks`（ref_check 脚本:3）；按 AGENTS.md 探针三查，seed/导出状态（SURFACE vs FULL）需与课题底账一致；10 篇 L936 有「参照状态三查」前科。
- 判别法：核对导出日志/文件 header 的 seed 与 8576 底账一致；确认该文件导出自无数据包的原版 1.20.1（无 mod/neoforge 混入）。

**候选 W-C：本地 data JSON ≠ 官方 jar 数据（数据源差）**
- 证据：F3/F4 基于 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/placed_feature/`（仓库副本）。若该副本与官方 1.20.1 jar 内 JSON 有漂移（历史上 id 注读误读有前科，10 篇 L1201/L1223），上界结论不可靠。
- 判别法：从 `runtime/1.20.1` 官方 jar 直接解压 `data/minecraft/worldgen/placed_feature/ore_*.json` diff（主会话执行，scout 无命令权）。

**候选 W-D：未知 vanilla 写者（biome 装饰/其它 feature 类型）**
- 现状：F1-F7 已排除 oreVein、surface rule、已知 ore feature 上界；剩余可能 = 非 ore 型 feature（geode 无 granite、spring 无）或对 1.20.1 语义记忆缺口。**列为兜底候选**，仅当 W-A/B/C 全部排除才立 worker 详查。
- 判别法：若 W-A 排除（布局核对无误），对 (195,199) 邻域做 vanilla 存档实地核对（F3 看真实方块 + /locate 特征）。

---

## 1. ore/vein 管线地图（C++ 侧 + vanilla 参照）

### 1.1 NOISE 阶段（密度/aquifer/oreVein）

```
fillOneChunkCore（worldgen_api.cpp）
  ├─ densityBuf = finalDensity->sample（density_builder.h 树，JSON noise_settings/overworld.json）
  ├─ ChainedBlockSource：aquifer.apply（aquifer.h:121-137，density+e>0 → barrier margin stone）
  │    └─ <0 → oreVein.apply（ore_vein.h:29-55）
  │         ├─ y 预检查 y∈[-60,50]（ore_vein.h:33；Java 无显式预检但 VeinType 界等价，OreVeinSampler.java:49-51）
  │         ├─ veinToggle>0→COPPER(granite/copper_ore/raw_copper, y0..50)；≤0→IRON(tuff/deepslate_iron_ore, y-60..-8)
  │         ├─ lerpClamp(min(j,k),0,20,-0.2,0)+e<0.4 → -1（ore_vein.h:43-44）
  │         ├─ splitter.split(x,y,z)（"minecraft:ore" 派生，02 篇 L58）；nextFloat>0.7 → -1
  │         ├─ veinRidged>=0 → -1（ore_vein.h:47，apply 开头语义）
  │         └─ 90/10 ore/stone（raw 2%）：ore_vein.h:49-52
  └─ vein 三分量 = noise_router 内联（DensityFunctions.java:459-473 verticalRangeChoice(-60..50)；
       C++ density_builder.h 动态构造，05 篇 L13-16/03 篇 L70；插值器 idx5-7 属 OreVeinSampler 不在 finalDensity 树，03 篇 L96）
```

### 1.2 FEATURES 阶段（ore blob 家族 = 课题主嫌）

```
applyCarversAndFeatures（07 篇 L399：GenerationStep.Feature ordinal，underground_ores=6）
  ├─ setDecoratorSeed(l,p,k)（populationSeed——Rust 侧 1/13 匹配未闭合，11 篇 L33；C++ chunkrandom.h:151-154 setSeed）
  ├─ PlacedFeature.generate：modifiers 惰性链（深度优先！placement.h:324-339——历史上广度优先 bug 致 granite 56%，07 篇 L433-436）
  │    ├─ count / rarity_filter / in_square（placement.h:141-180）
  │    ├─ height_range：HeightProvider uniform/trapezoid（placement.h:183-193 + carver.h:97-118；
  │    │    JSON 锚点 absolute/above_bottom/below_top，carver.h:86-92）
  │    └─ biome 过滤（placement.h:218-231，C++ 简化保留位置——**与 Java posToBiome 判定差异是已知简化点**）
  ├─ OreFeature.generate（feature.h:130-157，Math.sin 标准库 + OCEAN_FLOOR_WG 门 feature.h:151）
  │    └─ generateVeinPart（feature.h:160-246，MathHelper.sin 查表 feature.h:176；target 谓词 base_stone_overworld
  │         展开 feature.h:316-337——**谓词读到的方块状态耦合前序 feature（消融不干净，phase2-probe-round-260904-04.md:27-31）**）
  └─ 跨 chunk 两阶段 regionCols/pendingCross（feature.h:75-79,112-123；granite 88.3% 那轮的基础，07 篇 L447-452）
```

### 1.3 id 映射（raw id vs state id，#8 家族风险点）

- `blocks.id(name)` 返回 raw block id（feature.h:373）；ore JSON 的 `state.Name` 无 Properties 处理（copper_ore 等 default 态，当前数据无多态 ore，风险低）。
- 对照表 `data/blocks.json`；**历史前科**：10 篇 L1201/L1223 同一份 blocks.json 曾发生两次 id 注读误读（9=dirt 非 water、2=granite）——任何"id 数字→名"结论必须经 blocks.json 亲核，不凭记忆。
- ref_check 脚本的 `id2name` 未命中时**原样打印数字**（ref_check_p4_260904-06.py:20 `id2name.get(v,v)`）——若 vanilla 导出用了不同 blocks.json，高 y "granite" 可能是**未映射 id 被误读**（并入候选 W-A/W-B 判别）。

---

## 2. 22653 mismatch 族分解底账（既有数据位置）

| 产物 | 内容 | 路径 |
|------|------|------|
| phase2-probe-round-260904-04.md | **发现 0**：block_probe -features vs vanilla = 22653 / 98.56%；族：tuff→deepslate 1496、andesite→stone 1270、granite→stone 1175、air→deepslate 1171、stone→coal_ore 974、tall_seagrass→water 864；**发现 1**：skip 消融不干净（22653→30940，air→polished_granite 3677），特征间经 target 谓词耦合 → **skip 归零法无效，需加法消融/定点写者追溯** | .investigations/lossless-accel/fanout-residual-260904-04/phase2-probe-round-260904-04.md:22-31 |
| skip-ab-parse | baseline=22653 skip=30940 原始输出 | 同目录 cmd-output/skip-ab-parse-260904-04.txt |
| origin-audit | ore_dirt y∈[0,158] 零出界 / ore_gravel 全量程 / disk_sand 0 次 | 同目录 cmd-output/origin-audit-parse-260904-04.txt |
| b1/b2/b3 三候选 | b1 feature blob（部分成立）/ b2 surface（主导排除）/ b3 伪影（排除） | 同目录 b1-feature-blob.md / b2-surface-disk.md / b3-artifact-exclusion.md |
| p4 参照复验 | 高 y granite idk 诞生 + 「vanilla y≥201 无 stone」参照误读 | .investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md |
| 原始参照/探针数据 | cpp-dbdebug / ref_check 脚本与输出 / blocks.json | .tmp/p2full/*260904-06* |
| review 三源核对 | writer-verdict 结论取代背景 | .investigations/lossless-accel/review-260905-writer-vehicle-verdicts.md |

各族判别探针现状：**尚无 per-族判别探针**——已有的是全局 WG_FEATURELOG/[ORIGIN]/[OREPLACE]（feature.h:230、worldgen_api.cpp genFn 过滤）与 WG_FEATURE_SKIP 门控（已被证明耦合污染）。加法消融（只开单一 placed_feature）工具未建。

---

## 3. 分叉候选盘点（按族，互斥并列）

> 通用背景：block_probe -features 载具 ≠ mod cppReplace 载具（phase2:26，§9.7 不可互引）；Rust 侧 populationSeed 1/13 是另一未闭合点（11 篇 L33）——课题内 MUST 声明载具口径。

### 3.1 族：granite→stone 1175（+ 高 y 32 块同族）
- **G-A** C++ ore_granite placed_feature 未放置/落点错位（随机序列 desync，与 Rust 1/13 同家族）→ 探针：WG_FEATURELOG 只开 ore_granite，统计 C++ 落点数/y 分布 vs vanilla 同 chunk granite 计数（数量对齐判位置错，数量缺判未放置）。
- **G-B** ore_granite 落点对但 target 谓词失败（C++ 该处非 stone/tag 差异——注意 block_probe 载具 surface/carver 前置状态与 vanilla 的差）→ 探针：[OREPLACE] 扩展打印 target.test 失败时的实际方块 id 分布。
- **G-C** 参照侧解读伪影（W-A/W-B/W-C，§0.2）——高 y 32 块若坐实伪影，granite 族 mismatch 数字本身需重算（对比脚本可能同布局 bug）→ 探针：候选 W-A 判别法，且**检查 22653 对比脚本是否用了同一布局假设**。

### 3.2 族：tuff→deepslate 1496（最大族）
- **T-A** ore_vein IRON 脉 tuff 写者 y 界或 toggle 方向差（copper/iron 归属 d>0 分界）→ 探针：ore_probe.exe（08 篇 L48）对 mismatch 密集 chunk 采样三件套+apply 决策 vs Java OreProbe。
- **T-B** ore_tuff/diorite 类 placed_feature desync（deepslate_ore_replaceables 谓词耦合，feature.h:324-326）→ 探针：同 G-A 加法消融。
- **T-C** deepslate 过渡带 y 判定差（surface/噪声明暗线）→ 探针：对 mismatch cell 标注 y 直方图，看是否集中 y=0 过渡带。

### 3.3 族：andesite→stone 1270 / granite→stone 1175（stone 侧）
- **A-A** vanilla 有 blob（ore_andesite/ore_granite）而 C++ 同位置未写 → G-A/G-B 探针复用。
- **A-B** C++ 写了 stone（oreVein copper 脉 stone=granite、iron 脉 stone=tuff——**stone→granite 反向也应在列**）而 vanilla 无 → 探针：反向对（C++ granite→vanilla stone）计数，若与正向同量级则疑 oreVein desync 而非 feature 缺失。

### 3.4 族：stone→coal_ore 974
- **C-A** ore_coal_upper（count 90，trapezoid 90..?) / ore_coal_lower placed desync（同 G-A 家族）→ 探针：coal 两 placed_feature 的 JSON 上界亲核 + 加法消融。
- **C-B** ore_coal_buried 的 discard_on_air_exposure 分支差（isExposedToAir feature.h:255-266，邻域读跨 chunk 时 -1 语义）→ 探针：mismatch cell 是否贴空气。

### 3.5 族：高 y granite/copper（p4 新 32 块）
- **H-A** 参照伪影（W-A 布局 / W-B 文件 / W-C 数据源）——判别见 §0.2；**这是立项后第一探针**（成本最低，且若坐实则课题部分底账要重算）。
- **H-B** iron 合法（F5）但 granite/copper 需 W-D 兜底详查。
- 注意：**iron@高y 与 granite@高y 应分开判**——iron 有合法写者，混在一族会污染判别。

### 3.6 交叉风险（进课题计划）
1. 22653 对比脚本与 ref_check 是否共享同一列布局假设——若共享，H-A 坐实时全族数字连坐重算（先于一切 worker 分析）。
2. feature 间谓词耦合（phase2:27-31）→ 课题探针一律用加法消融，禁 skip 消融。
3. 载具声明（§9.7）：全部结论限 block_probe -features 载具 + 参照文件 8576294172403134396 同口径。
4. biome 过滤 C++ 简化（placement.h:218-231）是结构差嫌疑点——若加法消融显示落点数对但位置系统性偏，先查此简化。

---

## 4. 待深入点清单（交主会话排程）

| # | 事项 | 建议 |
|---|------|------|
| 1 | H-A 判别（列布局/seed/数据源三查） | 立项后第一动作，主会话执行（scout 无命令权） |
| 2 | 22653 对比脚本布局审计 | 同上，与 1 合并 |
| 3 | coal/copper/andesite 各 placed_feature JSON 上界全量亲核 | worker 一轮（本轮只亲核 granite/copper/iron/tuff） |
| 4 | 加法消融工具（只开单一 placed_feature 的 WG_FEATURE_ONLY 门控） | 工程项，swe 闭环 |
| 5 | W-D 兜底（vanilla 存档实地核对） | 仅 1-3 排除后 |

## 5. 混淆评估

不适用（Java 源码参照 = yarn sources，无混淆；C++ 为本仓库自研）。
