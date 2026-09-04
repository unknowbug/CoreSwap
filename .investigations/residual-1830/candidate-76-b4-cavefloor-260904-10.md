# candidate-76-b4 ——「腔底 default 集合 / s 语义 / 水线边界规则」机制候选分析

> 角色：core-worker（subagent 隔离，只读静态分析，无 shell，**全部结论未运行时验证**）
> 日期标签：260904-10（沿用任务方命名；日期锚由主会话 Get-Date 复核）
> 输入：`.tmp/p2full/residual76-dump-260904-10.txt`（已全文读取，95 行）、scout-residual76-260904-10.md
> 对象：零星 3 块 (198,198,18) water→dirt / (237,224,41) granite→gravel / (237,224,42) gravel→water
> 状态：**draft**（Degraded——纯静态读码）

---

## 1. 代码定位（行号全带来源）

### 1.1 Rust 列引擎（`WorldgenRust/src/surface_rules.rs` build_surface，L1187-1344）

- 扫描起点：`o = heightmap + 1`（L1259，Java L117 对齐注释）；pillar 写回（L1265-1269）。
- 列状态机（L1274-1341）：`q`=连续实心计数、`r`=首个流体 y+1（`r==MIN` 才更新，L1291-1293）、`s`=首非默认块位置（哨兵 `i32::MAX`，L1276）。
- **s-scan（L1294-1311，.b4 核心嫌疑点）**：

```rust
if s >= wy { s = i32::MAX; ... }
  let mut v = wy - 1;
  while v >= min_y - 1 {
      ...
      if st2 != air_block && st2 != water_block && st2 != lava_block {
          v -= 1; continue;      // ← 实心（含 granite/gravel/dirt/深板岩）一律当「default」继续向下
      }
      s = v + 1; break;          // 只在 air/water/lava 停
  }
```

- 规则应用集合：`if state == default_block`（L1317，default=stone，L1229）——与 Java「只对 stone 应用规则」一致。
- 规则树产 water 的位置：**海洋段**（L966-985：STONE_DEPTH_FLOOR L968 → `mc8`（water 条件，L970）→ `mc12` L973 → `mc11` L975 → water/ice，L977-979）；WaterCond 公式 L88-98（`fluid_height==MIN → 恒真`；`blockY + (add_stone_depth?sda:0) >= fluid_height + offset + surface_depth*mult`）。
- StoneDepthCond L100-104（`i<=1+offset+j+k`，与 docs/06 L43-44 已验证公式一致）。
- **ore_vein 在 surface 之前**：`worldgen_handle.rs` 步骤 2（L537-554，注释「矿脉只替换 rock 处的深部块」）→ 步骤 3 build_surface（L560/615）→ 步骤 4 carver（L619）→ 步骤 5 features（L624）。另有同源 pre-surface 导出函数 L637-687（「surface 阶段之前的区块方块 (fill_chunk 宏观 + aquifer + ore_vein)」）——判别探针可直接复用。
- C++ 侧同构：`versions/1.20.1/cpp/worldgen/src/surface.h` L772-788（同「实心继续」扫描）、L795（default 才应用）——**C++ 与 Rust 行为一致，若此差异成立则两侧同病，属「Rust/C++ vs Java」差异**。

### 1.2 Java 参照口径（docs/06-surface-rules.md）

- L62：**「s 语义：从 wy-1 向下找第一个非默认块（默认块=stone），s = v+1」**——Java 扫描在**第一个非 stone**处停（granite/gravel 也会停）。
- L63：initVerticalContext 参数序 (sda, sdb, r, x, y, z)——Rust L1315 一致。
- L94「已验证的坑」：*「s 判定集合：Java isDefaultBlock（==stone）vs C++ 早期只认 air/water/lava——非默认块（gravel 等）的处理集合必须一致」*。

### 1.3 ⚠️ 发现：doc 口径与现行代码不一致（本候选最有价值的静态发现）

- docs/06 L62/L94 说 Java 在**非 stone 即停**；但现行 Rust L1304 与 C++ surface.h L781 都是「**实心即继续，air/water/lava 才停**」——即「非默认块处理集合」**并未**按 L94 的坑修成 ==stone 语义。
- 两种可能（静态无法裁决）：① docs L94 记录的修复实际改的是别处/被回退，代码仍带此偏差；② Java 1.20.1 实际扫描条件就是「非空气非流体即继续」，docs L62 的「默认块=stone」注释不准确。本项目 web 检索未取得 vanilla SurfaceSystem 源文本（javadoc 只有类名），**需要 Java 源码/javap 一手核verify**（判别探针 P0，见 §6）。
- 可观测性修正（相对 scout D-5 的预期）：scout 曾默认「surface 期下方块必为 NOISE 输出 {stone,water,air} → 两种扫描集合等价 → 不可观测」。但 **ore_vein 在 surface 之前运行**（worldgen_handle.rs L537-554），且 vanilla 1.18+ 大矿脉的填充块含 granite/tuff（copper vein≈granite 填充、iron vein≈tuff 填充，y41 落在 vein y 域内）——**pre-surface 列内可以存在非 stone 实心** → s 集合差异**可观测**，.b4 回到候选席。

## 2. (237,224) y41/42 成对翻转机制推演

签名（dump L13-14）：ref y41=granite / CS y41=gravel；ref y42=gravel / CS y42=water。列顶双侧同 62（dtop=0），水深双侧同 20，biome 双侧同 raw[20]。

### 2.1 关键前置：granite 不可能是 surface 规则产物，且可先于 surface 存在

- 规则应用集合 = stone only（Rust L1317 / Java 同）→ surface 不会「把 granite 改成 gravel」也不会产 granite；ref 的 granite 只能来自 **ore_vein 填充（pre-surface，copper/iron vein 填充块）或 feature blob（post-surface）**。
- → y41 差（granite vs gravel）必为 **pre/post-surface 之外的阶段差**：要么 CS ore_vein 在 (237,y41,224) 漏放 granite 填充（pre-surface 差 → CS 该位是 stone → surface 走 gravel patch → gravel，而 vanilla 该位 granite 非默认被 surface 跳过），要么两侧 ore_vein 都没放、ref 的 granite 是 feature blob 放的而 CS feature 漏放（post-surface 差）。两者都不是 s 语义本身。

### 2.2 y42 差（ref gravel vs CS water）的两条机制线

- **线 1（surface 分支差，非 s 语义）**：若 pre-surface 两侧 y42 同为 stone，则 ref 落 gravel patch 分支、CS 落海洋段 water 分支（L977-979 的 `b("water")`——**surface 规则本身可以产 water**）。分支分流条件 = `mc8`（water(-1,0)：`wy >= r-1+0`，r 双侧同 43 → 42≥42 双侧真）/ `mc12` / `mc11`。r、biome、列顶双侧全同 → 剩余可差输入只有 `mc11`/`mc12` 的真实身份（若其中之一是 abovePreliminarySurface/est 4 角插值类条件，则 est 在该列差一格即可翻转）。**此线不需要 s 差异**——s 只进 stone_depth_below（vx），本列 stone run 只有一块厚（y42），分支条件不含 vx。
- **线 2（s 集合差级联，本候选 .b4 本体）**：若 pre-surface 两侧 y41 都是 granite，则 vanilla 在 y42 扫描「y41 是非 stone → 停，s=42」，CS「y41 实心 → 继续，s 更深」→ **vx（stone_depth_below）两侧差若干**。但本列 y42 是列内唯一实心块，其后的分支条件（water/StoneDepth-floor/gravel patch 噪声）均不消费 vx；**本列内 s 集合差无可观测后果**。线 2 若要成立，必须落在「stone run 跨过 granite/tuff 且下方规则用 stone_depth_below（CEILING 类 / badlands secondary）」的列——本对不是该形状。
- **判别性陈述**：线 1 与线 2 对 y42 的区分 = pre-surface y41 是否 granite（P2 探针）；但无论哪条线，**y41 差独立归因于 ore_vein/feature 域**（§2.1），.b4「腔底 default 集合」无法整体解释这一对。

### 2.3 (198,198,18) water→dirt

- y18 深层腔内：ref=water（aquifer 流体），CS=dirt。dirt 只能由 surface 规则在 **stone** 上产出（CS）→ 说明 surface 期 CS 该位是实心、ref 该位是流体 → **pre-surface（aquifer）固/液边界差一格**，surface 只是忠实级联（把 CS 的石头染成 dirt）。与 aquifer splitter 修复族同域（f41555d 后残差），不指向 s 语义。
- 附注：该块 y18 低于海洋水柱（wd=22 → 开放水 y41..62），深层水只能来自 aquifer 腔；r 语义上腔内流体在「先遇 air 重置 r=MIN」（L1287-1289）后会重新抬 r——两侧逻辑一致（air→q=0,r=MIN 同构），非候选点。

## 3. 可排除性评估（诚实分级）

| 子项 | 判定 | 依据 |
|---|---|---|
| s=v+1 数值语义（哨兵/重扫条件/世界底） | **排除**（静态） | Rust L1276/1295-1310 与 docs L62-63 公式逐项同构：哨兵 MAX↔MIN 等价（`s>=wy` 重扫）、世界底 air 化等价（L1299-1301 ↔ Java o<min 退出 s=min）、vx=wy-s+1 一致 |
| isDefaultBlock 集合（==stone vs 实心集合） | **活候选但非本 3 块根因** | 代码差异实存（§1.3）；但 (237,224) 列内 vx 不被任何分支消费 → 本对不可由它解释；(198,198,18) 是 aquifer 域。**全局影响待 P1 源码核实 + 探针** |
| 水线边界（water(-1,0)/STONE_DEPTH_FLOOR 段） | **活（线 1）** | y42 双侧分支分流（water vs gravel）只能来自 mc11/mc12 类条件输入差或 r 差——r 已排除（wd 双侧同 20），est 类条件差是剩余嫌疑 |
| .b4 整体解释零星 3 块 | **不成立** | y41 granite 归 ore_vein/feature 域（surface 只对 stone 动手，§2.1）；y18 归 aquifer 域（§2.3） |

## 4. 与主族 73 块（.b2 噪声 patch）的关系

- 主族 gravel↔sand 双向互翻（49+24）全在 y35-40 开放水域床面，列顶/biome/r 全同——若线 1（mc11/est 类条件差）成立，则主族与 y42 同根（**分支输入差**而非噪声阈值差），可与 .b2 竞争/合并；此点建议 judge 一并对照（本 worker 不越界下结论）。

## 5. 未验证声明

- 本产物纯静态（Degraded）：未跑任何探针/对比；§2 的机制推演是基于签名与代码读码的**候选解释**，非结论。
- Java 1.20.1 SurfaceSystem 扫描条件 ==stone 的口径当前仅由 docs/06 L62/L94（二手转述）支撑，未取得一手源码行。

## 6. 判别探针设计（交主会话执行）

### P0【静态一手核verify，最先做】Java SurfaceSystem.buildSurface 扫描条件
```
# 在 Java 参照源码（yarn/mojang sources jar 或 javap -c SurfaceSystem）中定位 buildSurface 列循环，
# 提取 s-scan 内层 while/for 的条件表达式（是否 getBlockState(...).is(defaultBlock) / ==stone），
# 落盘 .investigations/residual-1830/cmd-output/java-sscan-source.txt
```
- 判据：条件确为「==defaultBlock 才继续」→ Rust L1304/C++ surface.h L781 为实锤偏差（.b4 升级，需另行立项评估影响面——注意影响面评估禁止归零式 A/B，workflow-patterns #34）；条件为「非空非流体即继续」→ docs/06 L62/L94 表述需修正，.b4 default-set 支项降级关闭。

### P1【Rust pre-surface 剖面，零重导可先查】双列三检查点 dump
复用 worldgen_handle.rs L637-687 现成 pre-surface 导出（fill_chunk+aquifer+ore_vein，surface 之前）+ [SOUL-CTX] 点集门控模式（surface_rules.rs L1251/1319-1333，dump_points 驱动）：
```
# 1) pre-surface：用 L637 起的 pre-surface 路径导 (237,224) 与 (198,198) 列 y12..46 方块态
# 2) post-surface / post-feature：WG_AQDUMP 同款点文件门控（aqdump_hit 模式，先判 env 存在性——
#    investigate-260904-09.md 错误#1），points.txt 写：
#    237,42,224
#    237,41,224
#    198,18,198
#    （额外补 y38..46 全剖面点更佳）
WG_AQDUMP=.tmp/p2full/b4-points.txt <常规 driver 命令，seed=8576294172403134396>
# stderr 落 .tmp/p2full/b4-soulctx-260904-10.txt
```
- 判据：① CS pre-surface y41 是否 granite/实心非 stone——是 → ore_vein 域差实锤（y41 差归 ore_vein 课题）；否（stone）→ feature blob 域差（y41 归 feature 课题）。② y42 surface 前后态：pre=stone、post=water → 线 1 surface 分支差实锤，[SOUL-CTX] 的 biome/sda/sdb/fluid_height 直接给出 mc11/mc12 哪个条件翻掉。③ (198,18) pre-surface 是否实心 → aquifer 边界差实锤。
- Java 侧对照（如需 mc11/mc12 逐条件值）：克隆 AquiferDumpProbeMixin 模式（.tmp/aqdump/）对同列做 surface decision 捕获——二轮再做，先看 P1 是否已可裁决。

## 7. 来源清单

- `.tmp/p2full/residual76-dump-260904-10.txt` L13-14/83-94（3 块签名/成对计数）
- `WorldgenRust/src/surface_rules.rs` L88-98/100-104/865-1038/1187-1344（列引擎+规则树+诊断门控）
- `WorldgenRust/src/worldgen_handle.rs` L2/493/537-560/615-687（管线顺序 + pre-surface 导出函数）
- `versions/1.20.1/cpp/worldgen/src/surface.h` L750-808（C++ 同构）
- `versions/1.20.1/docs/06-surface-rules.md` L54-64/91-95/161-189（s 语义口径 + 已验证坑 + Beardifier 级联先例）
- `.investigations/residual-1830/scout-residual76-260904-10.md` §2.2 D-5 / §5 .b4、`investigate-260904-09.md` L9/19（WG_AQDUMP 模式与 env 存在性坑）
- web 检索（未获一手源码，仅确认 javadoc 存在）：[SurfaceSystem javadoc](https://aldak0.ru/javadoc/1.21.1-21.1.x/net/minecraft/world/level/levelgen/SurfaceSystem.html)、[yarn SurfaceBuilder](https://maven.fabricmc.net/docs/yarn-1.21+build.1/net/minecraft/world/gen/surfacebuilder/SurfaceBuilder.html)
