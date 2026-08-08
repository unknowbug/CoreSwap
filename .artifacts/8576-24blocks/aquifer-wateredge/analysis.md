# 8576-24blocks 深板岩/水边界 12 块 mismatch 根因分析

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6 chunks（chunk x 45..50, z -27..-22）
> 参照：`versions/1.20.1/data/vanilla_8576294172403134396_6_720_-432.blocks`（SURFACE 状态）
> 范围：深板岩/水边界类 #1,2,3,7,8,9,13,16,17,21 + 可并入 #11,12,14（13 块，任务口径 12 块）
> 角色：anchor.worker 精确分析（只读，不改代码/参照）
> 日期：2026-08-09　状态：**draft**（静态对拍结论；运行时剖面验证未完成，见 §6）

---

## 0. 结论先行

**根因（候选，置信度中等偏上）**：12 块深板岩/水边界 mismatch = **aquifer.apply 的 null/water/air 三态判定在 density≈0 / 液面边界 ±1 格的翻转**，最可能来源是**块级 finalDensity 的 InterpolatedDF 插值精度与 Java CellCache 的微小差（已知 7e-6~0.12 量级 POC 现象，docs/10）在敏感边界导致符号翻转**——**不是 aquifer 逻辑 bug**（判定链与 Java 逐行一致），**不是 deepslate 床判定 bug**（deepslate 规则两版一致），**不是 -288 式结构/FEATURE 假 diff**（8576 参照是 SURFACE 状态，形态无大段结构差异）。

附：对拍发现 C++ 与 Java 的 **2 个理论不等价点**（§4），非本次 12 块根因，但建议顺手对齐。

---

## 1. mismatch 清单与机制归类

blocks.json 映射：0=air 1=stone 8=grass 9=dirt 32=water 970=deepslate（y≤8 深板岩层）。

| 块 | pos | got | vanilla | 机制类 | 说明 |
|---|---|---|---|---|---|
| #1 | 764,-31,-417 | 32 water | 970 | A | 深板岩床顶孤立 water 格（C++ 偏 water） |
| #7 | 764,-32,-416 | 32 | 970 | A | 同上 |
| #8 | 764,-31,-416 | 32 | 970 | A | 同上 |
| #9 | 764,-31,-415 | 32 | 970 | A | 同上 |
| #2 | 790,2,-432 | 970 | 32 | B | C++ 水面/水底高 1（water 少 1 格，deepslate 顶替） |
| #3 | 804,-2,-420 | 970 | 32 | B | C++ 水底高 1 |
| #13 | 810,-4,-415 | 970 | 32 | B | C++ 水底高 1 |
| #14 | 723,9,-393 | 1 stone | 32 | B | C++ 水底高 1（同 B） |
| #16 | 802,0,-372 | 0 air | 970 | C | C++ 深板岩顶低 1（air 侵入） |
| #17 | 810,-11,-355 | 970 | 0 | C | C++ 洞穴底高 1（air 被填） |
| #21 | 807,0,-347 | 970 | 0 | C | C++ 深板岩顶高 1（air 被填）；与 #16 同 y=0 互补 |
| #11 | 754,61,-403 | 9 dirt | 32 | D | 地表整列 +1（水塘边界差 1）——非 aquifer 液面型 |
| #12 | 771,41,-410 | 1 stone | 32 | D | 地表/水面 +1 型 |

机制类：
- **A（#1,7,8,9）**：深板岩床内部 1 格 C++=water。参照床顶 -32/-31 连续 deepslate，C++ 在床顶 -31（或 -32）判 aquifer=water。床顶是 density 从负（固体）到正（床顶上方）的边界格。
- **B（#2,3,13,14）**：C++ 判 null（→deepslate/stone）而 vanilla 判 water，水层薄 1 格。水面/水底边界格。
- **C（#16,17,21）**：deepslate↔air 翻转，#16 与 #21 为同 y=0 互补（一处 C++ 少 deepslate、一处多），#17 为洞穴底 air 被 deepslate 顶替。
- **D（#11,12）**：地表 stone→dirt→grass 三连段整体 +1（754/771 列，含浅水塘水面同步 +1）——属**地表高度差 1**（buildSurface 起点 / est / runDepth 分量），机制与 A/B/C 不同，任务「可并入」但非 aquifer 液面 bug。

A/B/C 共 11 块全部是 aquifer.apply 三态（null/water/air）在边界 ±1 的翻转；方向混合（C++ 偏 water、偏 null、偏 air 都有），无系统性单侧偏移。

---

## 2. 关键机制回顾（为什么 mismatch 只能来自 aquifer 三态）

1. **fillFromNoise（NOISE）**：每块 density ≤ 0 时 aquifer.apply 返回 null（保持 stone）/ water / air / lava；density > 0 返回 null（→ 默认 air/stone 由上层决定）。C++ 输入 `densityBuf = h->finalDensity->sample(fpos)`（worldgen_api.cpp:609）；Java 输入 `cacheAllInCell(add(finalDensity, Beardifier.INSTANCE))`（ChunkNoiseSampler.java:177-181，Beardifier 恒 0）。
2. **buildSurface（SURFACE）**：只对 `state == defaultBlock(stone)` 应用规则（C++ surface.h:764 / Java SurfaceBuilder L156）；deepslate 规则 `verticalGradient("deepslate", 0, 8)`（C++ surface.h:654 / Java VanillaSurfaceRules.java:283）只染 stone。
3. **因此**：got=970(deepslate) 说明 aquifer 在该格返回 null（stone）且 y≤8 → deepslate；got=32(water) 说明 aquifer 返回 water（或液面判定）；got=0(air) 说明 aquifer 返回 AIR（blockY ≥ 液面）或 density>0。**12 块全部是 aquifer 三态判定差异的落点，buildSurface/deepslate 规则本身无差异**（§3.8）。

---

## 3. 液面/床判定对拍表（Java AquiferSampler.Impl vs C++ aquifer.h）

源码：`versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java`、`NoiseChunkGenerator.java`、`ChunkNoiseSampler.java` vs `versions/1.20.1/cpp/worldgen/src/aquifer.h`、`worldgen_api.cpp`。

| # | 判定点 | Java（行） | C++（行） | 一致性 |
|---|---|---|---|---|
| 1 | fluidLevelSampler（默认液面） | `createFluidLevelSampler`：`y < min(-54,63) → FluidLevel(-54,LAVA)` 否则 `FluidLevel(63, defaultFluid=WATER)`（NoiseChunkGenerator:78-84） | `defaultFluidLevel`：`blockY < -54 → FluidLevel(-54,lava)` 否则 `FluidLevel(63,water)`（aquifer.h:320-323） | ✓ **一致**（Java 液面恒 63，非传入 y） |
| 2 | apply 18 候选 / o,p,q,r,s,t 选择 | L158-207 | aquifer.h:81-101 | ✓ 逐行一致（pack/unpack 负坐标符号扩展已对齐） |
| 3 | `fl2.getBlockState(blockY)` → `blockY>=y ? AIR : block` | FluidLevel.java:61-63 | aquifer.h:23-25 | ✓ |
| 4 | `d<=0` 早退 / water+lava 下邻判定 | L212-217 | aquifer.h:110-114 | ✓ |
| 5 | calculateDensity（barrier 噪声缓存/分段 q/lavaWater） | L263-321 | aquifer.h:235-273 | ✓（-288 验证 barrier 分量一致） |
| 6 | getFluidLevel 13 邻居 OFFSETS | CHUNK_POS_OFFSETS L99-101 | OFFSETS aquifer.h:288-291 | ✓ 逐项相同 |
| 7 | est 调用 `chunkNoiseSampler.estimateSurfaceHeight(l,m)` | L363 | `estimateSurfaceHeight(l,m)` | ✓（est 17 点两版一致，docs/04） |
| 8 | est 4 格 biome 对齐 | BiomeCoords.toBlock(fromBlock) L223-224 | `(x>>2)<<2` aquifer.h:144-145 | ✓ 负坐标一致（floor 语义） |
| 9 | est 扫描：`initialDensityWithoutJaggedness > 0.390625`，步长 8 | ChunkNoiseSampler.java:228-240 | aquifer.h:154-157（`initial_density` 分量 = JSON `initial_density_without_jaggedness`，worldgen_api.cpp:372） | ✓ 分量映射正确 |
| 10 | est 未命中返回值 | Integer.MAX_VALUE（L239） | INT32_MAX（aquifer.h:153） | ✓ |
| 11 | getFluidBlockY：method_43718（erosion<-0.225F && depth>0.9F） | VanillaBiomeParameters.java:1206-1208 | aquifer.h:332-333（double 常量） | ⚠ 阈值 float vs double 微差（§4.2） |
| 12 | getFluidBlockY：bl→clampedMap(i,0,64,1,0)；fluidFloodedness clamp；map(f,1,0,-0.3,0.8/-0.8,0.4) | L401-407 | aquifer.h:336-343 | ✓ |
| 13 | getFluidBlockY 三态返回：e>0→defaultFL.y；d>0→noiseBased；else→field_35479(-32512) | L410-417 | aquifer.h:344-350（-32512） | ✓（-32512 修复已应用） |
| 14 | getNoiseBasedFluidLevel（floorDiv/roundDownToMultiple/min(est,q)） | L421-433 | aquifer.h:352-363 | ✓ |
| 15 | getFluidBlockState（fluidLevel<=-10 → fluidType 噪声判 lava） | L435-450 | aquifer.h:365-377 | ⚠ `!= INT32_MAX` vs `!= field_35479`（§4.1，影响面≈0） |
| 16 | deepslate 规则 verticalGradient(0,8) | VanillaSurfaceRules.java:283 | surface.h:654 | ✓ |
| 17 | buildSurface 仅染 defaultState=stone | SurfaceBuilder L156 | surface.h:764 | ✓ |
| 18 | surface() above_preliminary_surface（est 4 角 + runDepth - 8） | MaterialRules.java（docs 已验证） | surface.h SurfaceCondC（docs 已验证） | ✓ |

**结论**：aquifer 判定链 18 项静态对拍全部对齐（2 个理论不等价点见 §4）。docs/04 与 10-timewise 已用 Java RouterProbe/cns 实测确认 est、barrier/erosion/depth/fluidFloodedness、角点、插值（interp0/1 差 7e-6 量级）在 -288 全部一致。

---

## 4. 对拍发现的 2 个理论不等价点（非 12 块根因）

### 4.1 getFluidBlockState 无效液面检查用错常量
- Java：`fluidLevel != DimensionType.field_35479`（-32512）（AquiferSampler.java:437）
- C++：`fluidLevel != INT32_MAX`（aquifer.h:367）
- 后果：当 getFluidBlockY 返回 -32512（无效液面）时，C++ 会**误进入 lava 判定分支**（若 fluidType 噪声 |d|>0.3 则把 block 置 lava）。但 FluidLevel(-32512, …).getBlockState(blockY) 因 `blockY >= -32512` 恒真 → **恒返回 AIR**，block 字段是 water 还是 lava 不影响结果 → **对块判定影响面≈0**。属代码不等价但不产生本类 mismatch。

### 4.2 method_43718 阈值 float vs double
- Java：`erosion < -0.225F && depth > 0.9F`（float 常量）
- C++：`erosionDF->sample(pos) < -0.225 && depthDF->sample(pos) > 0.9`（double 常量）
- 后果：阈值区间差约 6e-9 / 4.6e-8，仅当 erosion/depth 恰好落在该窄区间时判定翻转。概率极低，非 12 块根因，但建议对齐 Java 的 float 语义（用 `-0.225f`/`0.9f` 常量）。

---

## 5. 根因判定证据链

### 5.1 排除「结构/FEATURE 假 diff」（对照 -288 结案）
- -288 结案（NEXT_SESSION 四 / 10-timewise L711-744）：95.72% 差 = **ocean ruin/沉船 + 矿脉 + 树草 FEATURE 假 diff**；参照 -288 是 **FULL 状态**（19:39 导出，含结构方块）；island 是 4×16 规则矩形结构段，density 差 0.12 翻转 aquifer 判定。
- **8576 参照是 SURFACE 状态**（10-timewise L328：`3200/20000/8576 SURFACE（方块对比用）`）→ **不含结构/FEATURE 方块**。
- 12 块形态全部为**边界 ±1 单格**，无大段结构矩形（区别于 -288 的整段岛缺失）；savanna/river/forest 无 ocean ruin 类结构块证据（有 village 可能但 SURFACE 参照不含其方块）。
- **排除结构假 diff**。

### 5.2 排除 seed 污染
- 参照文件头 seed=8576294172403134396 = 命令行 seed（已由 subagent 读取 blocks 头部确认）；docs/10 的「3200 污染教训」针对 `vanilla_-8248…`（8/8 00:02 被重导），本参照为 8/7 23:23 导出、文件名 seed=8576 与实际一致。

### 5.3 排除 deepslate 床判定 bug
- deepslate 规则（§3.16/17）两版一致；buildSurface 只染 stone。got=970 只可能是「aquifer 返回 null」的结果，非规则差异。

### 5.4 指向「插值精度 → 边界翻转」
- docs/10（L453）：`8576 98.67%→99.60%：密度角点全部对齐 0 差；剩余 0.4% = InterpolatedDF 插值差（非角点，已知 POC 现象，y60 险峻地形翻转）`——**已知 C++ 块级插值存在微小误差**。
- NEXT_SESSION（L67-68）：插值 interp0/1 C++ vs Java cns 链差 7e-6 量级（-288 点）；L70：**density 差 0.12 即可翻转 aquifer 判定**（-288 结构点证据，量级参考）。
- 8576 角点全对齐、块级插值点存在微差 → 在 density≈0 / 液面边界（深板岩床顶、水面/水底、洞穴底）恰好翻转向量混合（C++ 偏 water/null/air 都有）→ **与插值误差模式自洽**。
- 现剩余 mismatch 极稀疏（24/3,538,944 = 0.0007%，非 air 12/1,201,934 = 0.001%），符合「只在敏感边界偶尔翻转」的插值精度特征。

---

## 6. 未验证项（运行时验证未执行）

本 worker 只读沙箱拦截 exe 调用（read_only_task 实测 `blocked: read-only subagents can run only permission-classified foreground read-only commands`），以下**决定性验证需主会话/带运行权限 agent 补跑**：

1. **seed/匹配率确认**：
   ```
   & versions\1.20.1\cpp\build-msvc\bin\block_probe.exe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks -threads 8
   ```
   （应打印 `blocks file: … seed=8576294172403134396 …` 与 TOTAL≈99.9993%）
2. **12 列 WG_SURFDUMP 剖面**（关键：-40..5 的 initialDensity/finalDensity 符号 + estimateSurfaceHeight），模板：
   ```powershell
   $env:WG_SURFDUMP="1"; $env:WG_SURFDUMP_X="764"; $env:WG_SURFDUMP_Z="-417"
   & versions\1.20.1\cpp\build-msvc\bin\block_probe.exe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks -blockDump 764 -31 -417
   Remove-Item Env:WG_SURFDUMP,Env:WG_SURFDUMP_X,Env:WG_SURFDUMP_Z -ErrorAction SilentlyContinue
   ```
   列：764,-417｜790,-432｜804,-420｜764,-416｜810,-415｜802,-372｜810,-355｜807,-347｜754,-403｜771,-410｜723,-393｜733,-382
3. **判定判据**：若某格 C++ finalDensity 与 vanilla（需 Java DensityProbe/cns 同点采样）差 ∈ (0,0.12) 且符号翻转 → 坐实「插值精度边界翻转」；若 density 逐位一致但 aquifer 返回不同 → 升级为 aquifer 真 bug（回到 §3 逐点 Diag）。

---

## 7. 修复方向（若需逐位对齐）

1. **首要**：追 InterpolatedDF 块级三线性插值 vs Java CellCache 的浮点顺序/权重差（NEXT_SESSION L114：Java 8 interpolators vs C++ 6 实例，差 2 个 noodle_ridge 类——需复核 8 个插值器逐一对应）。这是根因候选，工程量中等，收益 = 消除边界翻转类剩余 mismatch。
2. **顺手对齐**（§4，不改变块判定但消除隐患）：
   - aquifer.h:367 `fluidLevel != INT32_MAX` → `!= -32512`（对齐 field_35479）。
   - aquifer.h:332-333 method_43718 阈值用 float 常量（`-0.225f`/`0.9f`）。
3. **D 类（#11,12）单独立项**：地表 stone→dirt→grass 三连段整体 +1 → 查 est 4 角插值 / runDepth / buildSurface 起点，与 aquifer 无直接关系。

---

## 8. 置信度

**draft**（按 §15.4 状态机：AI 绝不写 confirmed）
- 确定性高：aquifer 判定链 18 项静态对拍一致；deepslate 规则一致；参照 SURFACE 状态排除结构假 diff；seed 无污染。
- 待运行时确认：块级 density 符号翻转的直接证据（§6 判据）——未取得，故不能升级 candidate。
- 若运行时验证推翻「插值精度」→ 首查 §4.1/4.2 两点是否在目标格触发，其次重查 §3 逐点 Diag。

---

## 9. 产物引用

- 本文件：`.artifacts/8576-24blocks/aquifer-wateredge/analysis.md`
- 明细：`.investigations/8576-24blocks/mismatch-list.md`
- 列剖面：`.investigations/8576-24blocks/column-profiles.md`
- C++ 源码：`versions/1.20.1/cpp/worldgen/src/aquifer.h`、`surface.h`、`worldgen_api.cpp`、`density.h`
- Java 源码：`versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java`、`ChunkNoiseSampler.java`、`NoiseChunkGenerator.java`、`net/minecraft/world/biome/source/util/VanillaBiomeParameters.java`、`net/minecraft/world/gen/surfacebuilder/VanillaSurfaceRules.java`
- 结案记录：`NEXT_SESSION.md` 四、`versions/1.20.1/docs/04-aquifer.md`、`versions/1.20.1/docs/10-timewise-archive.md`
