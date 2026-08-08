# Phase 4：aquifer/surface 层三块具体差异根因定位（gravel / 含水层 / 小岛）

- 范围：seed=-8248318472910187742（-8248 世界），-288 区域 chunk(-16,-16)~(-18,-15)
- 方法：纯源码/数据解读（无执行）；C++ `versions/1.20.1/cpp/worldgen/src/{surface.h,aquifer.h}` vs Java `mc_src_extract/net/minecraft/world/gen/{surfacebuilder/,chunk/}` + `.investigations/-288-reopen/cmd-output/*` + `ref_col_*.txt`
- 置信度标注：**【确定】** = 源码+数据闭合；**【推测】** = 需某数据/执行验证

---

## 0. 执行摘要

1. **gravel 染色差（(-244,50,-256) C++ gravel vs vanilla stone）不是 surface 规则翻译错，而是 aquifer 决策差的下游效应**：
   - C++ 列在 y=51..62 全为水（trace 实证 bs=32），y=50 成为「第一 solid」（q=1）→ 触发 mr9 海底段 mr3 → `STONE_DEPTH_CEILING`(stoneDepthBelow=115) false → **gravel**。
   - vanilla y=50=stone 的唯一自洽解释：**Java 的 y=58..61 在 SURFACE 阶段已是 solid（含水层判 solid 的岛）** → y=50 扫描时 q=5 → `STONE_DEPTH_FLOOR`(q≤1) false → mr9 不命中 → 保持默认 stone。y=58 stone / y=59-61 dirt 与 surface 第 4 段（mc10→mr7→DIRT）染色 + `rd=sampleRunDepth=2` 完全自洽（见 §2.3）。
2. **C++ 判水 vs Java 判 solid 的差异点在 aquifer 输入**：C++ trace 显示 y=55-57/59-62 的 `d=maxDistance(o,p)≤0` → 直接返回水（不进入 e 计算）；y=58 `d=0.64>0` 进入 e，`e=0.0`（calculateDensity 中 j==0：相邻网格点液面相同）→ 水。**Java 要判 solid，其网格点距离 d 或液面（fl.y）必须与 C++ 不同** → 指向 **aquifer 网格点坐标（`splitter.split` 派生）或液面网格（estimateSurfaceHeight/getFluidLevel）的 C++/Java 差异**，需要 Java 侧对照数据收口。
3. **(-242,-256) 小岛差、(-278,-240) 含水层差同源**：均可用「C++ aquifer e≡0（density 直接定 solid/water）、Java e 非零（正→岛 solid、负→含水层 water）」解释，且 (-278) 的 e 为负方向证明不是固定偏置。

---

## 1. 关键原始数据复核（本分析依据）

### 1.1 C++ SURFTRACE（trace_aqf_1.txt L13）
```
[SURFTRACE] (-244,50,-256) q=1 vx=115 r=63 s=-64 biome=minecraft:cold_ocean state=1->37
[SURFTRACE] (-244,49,-256) q=2 vx=114 r=63 s=-64 ... state=1->-1   (不染)
[SURFTRACE] (-244,48,-256) q=3 vx=113 ... state=1->-1
...
```
- q=从顶向下 solid 计数；vx=wy−s+1（s=−64=最低 solid 层 → vx=wy+65）；r=63=最高流体 y+1（y=62 水）。
- **y=50 是 q=1** → 该列 y=51..62 全为流体（aquifer 已判水），y=50 是第一个 solid。

### 1.2 C++ AQF（trace_aqf_1.txt L4-12）
```
[AQF] (-244,55,-256) density=-0.043591 nearest=(o=54,p=101,q=126) d=-0.8800 bs=32
[AQF] (-244,56,-256) density=-0.053461 ... d=-0.4800 bs=32
[AQF] (-244,57,-256) density=-0.063950 ... d= 0.0000 bs=32
[AQF] (-244,58,-256) density=-0.074424 ... d= 0.6400 bs=32
[AQF-e] (-244,58,-256) density+e=-0.074424 (e=0.0000) -> FLUID
[AQF] (-244,59,-256) density=-0.084882 ... d=-0.2400 bs=32
[AQF] (-244,60,-256) density=-0.095322 ... d=-0.4800 bs=32
[AQF] (-244,61,-256) density=-0.105740 ... d=-0.7200 bs=32
[AQF] (-244,62,-256) density=-0.116134 ... d=-0.9600 bs=32
```
- **关键**：y=55-57/59-62 的 `d≤0` → aquifer.h L110 `if (d <= 0.0) return bs;` **直接返回水，不进入 e 计算**（trace 无 [AQF-e] 行印证）。
- 仅 y=58 `d=0.64>0` 进入 `calculateDensity`，`e=d*calculateDensity=0`（j==0 分支）→ density+e<0 → 水。
- d=maxDistance(o,p)=1−|p−o|/25（aquifer.h L229-232，与 Java AquiferSampler L258-260 一致【确定】）。

### 1.3 density 剖面（dump_x-244_z-256.txt [SURF]）
- y=48 +0.0256 / y=52 −0.0139 / y=56 −0.0535 / y=60 −0.0953 → C++ 列 y≈50 为水陆边界。
- cns 插值链（vanilla_density_overworld_c-16_-16_b12_0_cns.txt）idx0：y=48 +0.080098 / 49 +0.049182 / 50 +0.018266 / 51 −0.012651 / 52 −0.043567 → squeeze(0.64×idx0)≈C++ dump（phase3 已证 ≤4e-6）→ **Java 游戏实际 density 同样 y=50 solid、y≥51 水**【确定，phase3】。
- 结论：**density 层无差，差异完全在 aquifer 决策/surface 层**。

### 1.4 m288_natural_rows.txt 大面积模式
- `got=37(gravel) vanilla=1(stone)` 覆盖 chunk(-16,-16) x∈[-244,-241] z∈[-256,-248] 及 chunk(-15,-16) x∈[-240,-226]（cold_ocean + river 海底），y∈[49,52]——**系统性，非孤例/非结构**。
- (-244,58/59/60/61,-256)：got=water vs vanilla=stone/dirt/dirt/dirt（与 ref_col 一致）。

### 1.5 blocks id（blocks.json）
- 37=gravel、1=stone、32=water、970=deepslate、31=bedrock、8=grass_block、9=dirt。

---

## 2. gravel 染色差根因（重点）

### 2.1 C++ 触发路径【确定】
规则链（surface.h）：
1. `buildOverworldRule` 最终序列 L648-655：`surfaceCondC()(above_preliminary_surface) → mr9` + deepslate + bedrock。
2. mr9（L573-645）第 5 段（L639-644，STONE_DEPTH_FLOOR 段）：
   ```
   condition(stoneDepth(0,false,0,false),  // STONE_DEPTH_FLOOR: stoneDepthAbove(q) ≤ 1
       sequence(
           frozen_peaks/jagged_peaks → stone,
           warm_ocean/lukewarm_ocean/deep_lukewarm_ocean → mr2(sand),
           mr3))                             // cold_ocean → mr3
   ```
3. mr3（L482）：`sequence({condition(stoneDepth(0,false,0,true), stone), gravel})`——STONE_DEPTH_CEILING（stoneDepthBelow ≤ 1）→ stone，否则 **gravel**。
4. y=50：q=1 → STONE_DEPTH_FLOOR true → cold_ocean → mr3 → CEILING: stoneDepthBelow=115 ≤ 1 **false** → **gravel(37)**。

与 Java 对照（逐项一致【确定】）：
- Java `materialRule3`（VanillaSurfaceRules L72）= sequence(condition(STONE_DEPTH_CEILING, STONE), GRAVEL)。
- Java materialRule9 第 5 段（L263-270）= condition(STONE_DEPTH_FLOOR, sequence(frozen/jagged→STONE, warm/lukewarm→materialRule2, materialRule3))。
- StoneDepthCondition（MaterialRules.java L728-736）与 C++ StoneDepthCond（surface.h L228-234）公式逐位一致（`i ≤ 1+offset+addSurfaceDepth?runDepth:0+k`，C++ surfaceDepth≡Java runDepth=sampleRunDepth 列级常量——Java MaterialRuleContext.runDepth 在 buildSurface 中不更新，仅 initHorizontalContext 设 sampleRunDepth【确定】）。

**→ C++ 染 gravel 是 vanilla 规则树的忠实执行；问题不在规则翻译，而在触发规则的输入（q/vx 列形态）。**

### 2.2 vanilla y=50=stone 的机制【推测·高置信】
两个候选，仅一个自洽：
- (a) 结构岛覆盖：y=58-61 是 FEATURES 阶段放置的 structure → 与 ref 的 stone+dirt 方块吻合。**但** ChunkStatus 顺序（ChunkStatus.java L88-117：NOISE→SURFACE→L143-154 FEATURES）确定 buildSurface 在 structure pieces 之前【确定】；结构不碰 y=50，若 surface 阶段 y=50 已染 gravel，最终列 y=50 仍应 gravel → **与 ref stone 矛盾，排除**。
- (b) **Java 的 y=58-61 在 SURFACE 阶段已是 solid（含水层判 solid 的天然岛）** → 扫描 q：y=50 为 q=5 → mr9 第 5 段 STONE_DEPTH_FLOOR(q≤1) false → 不染 → 保持 stone ✓；y=58 q=4 不染（stone）✓；y=59-61 q=1/2/3 → 第 4 段 mc10(waterWithStoneDepth(-6,-1)) → STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH → mr7 → DIRT ✓（详见 §2.3）。

**(b) 为唯一自洽解释【推测·高置信】。**

### 2.3 vanilla 岛（y=58-61）染色自洽性【推测】
若 Java SURFACE 阶段列形态为：y=63 air、62 water(r=63)、61/60/59/58 solid、57..51 water、50/49/48 solid：
- y=61（q=1）：mr9 第 4 段 mc10：61+q(1)=62 ≥ r(63)−6−rd(·−1)=57−rd → 恒真 → STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH(q=1≤1+rd 真) → mr7 → **DIRT** ✓（ref y=61=dirt）
- y=60（q=2）：60+2=62 ≥ 57−rd 真；q=2≤1+rd（rd≥1）→ DIRT ✓
- y=59（q=3）：59+3=62 ≥ 57−rd 真；q=3≤1+rd（rd≥2）→ DIRT ✓
- y=58（q=4）：58+4=62 ≥ 57−rd 真；q=4≤1+rd（**rd≥3 才 DIRT**，rd≤2 → 跳过）→ 第 4 段其余不命中 → 第 5 段 STONE_DEPTH_FLOOR(q=4≤1 false) → **stone** ✓（ref y=58=stone）
- **要求 rd（sampleRunDepth）恰好 = 2**（y=59 需 rd≥2、y=58 需 rd≤2）→ **可验证预测**：C++ `ctx.surfaceDepth = sampleRunDepth(-244,-256)` 应为 2。
- 若 rd 不是 2，则需微调（比如 y=59-61 也可走第 3 段 mr8 的 mr→dirt，不影响主结论）。

### 2.4 为什么 C++ 没有这个岛【确定 + 推测】
C++ 列 y=55-62 全 bs=32（水）：
- y=55-57/59-62：`d=maxDistance(o,p)≤0` → aquifer.h L110 直接返回水（**不依赖 e**）。
- y=58：d=0.64>0 → e=d×calculateDensity；calculateDensity 在 `j=|fl.y−fl2.y|==0`（相邻网格点液面相同）时返回 0.0（aquifer.h L241-243，与 Java AquiferSampler L270-272 一致）→ e=0 → density(−0.074)<0 → 水。
- **Java 要判 y=58 solid 需要 e>+0.074（j≠0 或 barrier 大值）；要判 y=59-62 solid 需要 d>0（网格点距离与 C++ 不同）**。两个要求都指向 Java 的 **aquifer 网格点坐标（o/p/q）或液面（fl.y）与 C++ 不同**【推测】。

### 2.5 待验证点（Java 侧）
1. Java 在 (-244,55..62,-256) 的 computeSubstance 输入：o/p/q、d、fl.y/fl2.y、calculateDensity 的 j/e——与 C++ trace 逐项对比。
   - o/p/q 一致 → 液面网格差（estimateSurfaceHeight/getFluidLevel）
   - o/p/q 不一致 → 网格点坐标差（getBlockPos 的 splitter.split(x,y,z) 派生，aquifer.h L217-227 vs AquiferSampler L177-183）
2. C++ estimateSurfaceHeight per-chunk 缓存（aquifer.h L142-161，CACHE_OFF_X/Z、cacheCx/cacheCz）在 13 邻居跨 chunk（±48 格 → 5×5 chunk）时是否越界/未更新 → est 错 → 液面错 → e 错。
3. 验证 rd=sampleRunDepth(-244,-256)==2 预测。

---

## 3. 含水层差方向（(-278,-240) y=15 C++ stone vs vanilla water）

### 3.1 数据
- dump_x-278_z-240.txt [SURF]：y=8 +0.0427 / 12 +0.0557 / 16 +0.0687 / 20 +0.0685 / 24 +0.0684——**y=12..24 全为正**。
- C++（e=0）：density>0 → solid → y=15 stone ✓（与 got=stone 一致）。
- vanilla 参照：y=0-3 deepslate、4-14 stone、**15-19 water**、20-22 stone、23 air（洞穴）、24-30 stone。

### 3.2 判定
- vanilla y=15-19 是水，而 C++ density 在该段为正 → **Java 需要 density+e<0，即 e<−0.068（强负）**；y=20-22 又是 stone → e 在 y=20+ 转正/足够小。
- C++ e≡0（计算路径上 j==0 或未进入 e 计算）→ 全 solid。
- **方向判定：这是 aquifer e 值差异（C++ 缺失/恒 0 vs Java 有负 e 的含水层区域）**，不是 surface 规则问题【推测·高置信】。
- 与 §2 的 e 差同源（那边 Java e>0，这边 Java e<0）→ 证明 e 是**区域化符号值**，不是固定偏置【确定推理】。

### 3.3 下一步验证
- Java 侧 (-278,-240) 的 y=8..24 aquifer dump（o/p/q/d/fl.y/e 逐层），确认 Java e<0 的来源（j≠0？barrier 负？d 路径不同？）。
- 洞穴 air（y=23）：C++ 未判（solid），vanilla air——同样属于 aquifer/洞穴密度区域差异，一并验证。

---

## 4. (-242,-256) 小岛差方向

### 4.1 数据
- dump_x-242_z-256.txt：y=48 +0.0264 / 52 −0.0137 / 56 −0.0539 / 60 −0.0946。
- vanilla 参照：y=50 stone、51-55 water、**56-62 岛（stone+dirt+grass_block）**、63 air（岛露出水面，y=62 grass_block 顶）。
- mismatch：(-242,56,-256) C++ got=32 water vs vanilla=1 stone。

### 4.2 判定
- C++（e=0）：y=56 density=−0.0539<0 → water；岛缺失（y=56-62 全水）。
- vanilla：y=56-62 判 solid（需 Java e>+0.0539 等）→ 岛存在；surface 染色：y=62 grass（mr9 第 3 段 mr8 fallback mr：mc9=water(0,0) 62≥r(56) 真 → grass_block）、y=61-59 dirt（第 4 段 mc10→mr7）、y=58-56 stone（q 高不染）——与 ref 列自洽【推测】。
- **方向判定：与 (-244,-256) 完全同机制——C++ aquifer 判水（e 缺失）→ 岛缺失 → 下游 surface 行为连锁**。

---

## 5. 根因汇总与置信度

| # | 结论 | 置信度 |
|---|---|---|
| 1 | C++ gravel 染色的直接规则路径：y=50(q=1, stoneDepthBelow=115) → mr9 第 5 段 → mr3 → STONE_DEPTH_CEILING false → gravel(37) | **【确定】**（surface.h L639-644/L482 + trace） |
| 2 | C++ 规则树与 Java VanillaSurfaceRules 逐项一致（含 StoneDepth/Water 条件、runDepth=sampleRunDepth 语义） | **【确定】**（surface.h vs MaterialRules.java 对照） |
| 3 | C++ 列 y=51-62 全为水（AQF bs=32；y=55-57/59-62 因 d≤0 直接返回水）→ y=50 是 q=1 | **【确定】**（trace_aqf_1.txt） |
| 4 | vanilla y=50=stone 的机制 = y=58-61 在 Java SURFACE 阶段已是 solid（含水层判 solid 岛）→ q=5 → STONE_DEPTH_FLOOR false → 不染；结构岛假设被 ChunkStatus 顺序排除 | **【推测·高置信】**（ref_col + m288 + ChunkStatus.java L88-154） |
| 5 | C++ e=0 的直接原因 = calculateDensity 的 j==0（相邻网格点液面相同）→ 返回 0（aquifer.h L241-243，与 Java L270-272 公式一致） | **【确定】**（trace AQF-e + 源码） |
| 6 | Java 判 solid 的 aquifer 输入差异（网格点坐标 getBlockPos / splitter 派生，或液面网格 estimateSurfaceHeight/getFluidLevel）→ 需 Java 侧对照数据收口 | **【推测】**（待验证） |
| 7 | (-278,-240) 含水层差：C++ e≡0 → solid；Java 需 e 强负 → water（方向相反证明是区域化 e 值差） | **【推测·高置信】** |
| 8 | (-242,-256) 小岛差：与 #1-#6 同机制（C++ 判水 → 岛缺失） | **【推测·高置信】** |
| 9 | rd=sampleRunDepth(-244,-256)==2 预测（vanilla y=58 stone / 59-61 dirt 的染色边界） | **【推测·可验证】** |

---

## 6. 修复方向（仅定位，不修代码）

- **首要**：对齐 aquifer 网格点/液面链。重点排查：
  1. `getBlockPos` 的 `splitter->split(x,y,z)` 与 Java `randomDeriver.split(x,y,z)` 在负坐标/跨 chunk 的种子派生是否逐位一致（aquifer.h L217-227 vs AquiferSampler.java L177-183）；
  2. `estimateSurfaceHeight` per-chunk 缓存（aquifer.h L142-161）对 13 邻居跨 chunk（±48 格）的覆盖/更新（CACHE_DIM=32、CACHE_OFF_X=12、CACHE_OFF_Z=4 的边界），est 错 → 液面错 → e 错；
  3. `getFluidLevel`/`getFluidBlockY`（aquifer.h L286-363 vs AquiferSampler.java L353-433）的 13 邻居扫描与 est 输入。
- **决定性对照实验**：在 Java 侧补打 (-244,55..62,-256) 的 computeSubstance 中间量（o/p/q/d、fl.y/fl2.y、j/e）与 C++ trace 逐项对比——一次即可区分「网格点坐标差」vs「液面网格差」。
- surface 层本身无 bug（翻译正确）；gravel/小岛/含水层三类 mismatch 均为 aquifer 上游差的下游表现。

---

## 7. 影响架构的变化

> 标注「架构变更建议」，交主会话裁决（本角色不实施）。

- 无架构级变更建议。当前差异定位在 **aquifer 内部输入链（网格点随机派生/est 缓存/液面）**，属实现修正而非架构改动。
- 若确认 splitter 派生差异：涉及 `XoroshiroRandom::Splitter` 在负坐标派生路径的复用（aquifer splitter 与 density splitter 为同一对象，需确认派生键一致）。
- 参照列与 cns 数据可作为 aquifer 决策回归的固定测试样本（三列 + 三段 y 区间）。
