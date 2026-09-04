# d 候选（m2）：C++ vs Rust preliminary surface（est）语义逐行对拍（draft，260904-06 fanout-writer）

> 状态：**draft**（Degraded 静态审查，只读分析，未执行任何命令；运行时旁证引自 b1 §8/§9 已落盘数据）。
> 分析者：fan-out worker d。引用全部为本次实读 file:line；读不到的源码显式声明（见 §5）。

## 0. 结论一句话

**est 本体（preliminary surface level）零语义差（高置信）**：C++（aquifer.h L145-164 + surface.h L177-194）与 Rust（worldgen_handle.rs L564-579 + aquifer 对应实现）在「步长 8、域 [320,-64] 双端含、阈值 0.390625 严格大于、密度函数 = initial_density_without_jaggedness、无命中哨兵 INT32_MAX、四角坐标 +16」六个维度逐项一致，且有运行时旁证（两臂 est 四角实测同为 24/32，b1 §9.0 R2/E5）→ **SurfaceCondC 窗口 k 两臂相同，(m2)「C++ preliminary surface 语义差」不能解释 195 列幕帘边界翻转**。**唯一真分叉在本家族的邻近输入：heightmap 填充判据**——C++ `block != air`（worldgen_api.cpp L1045，含水，对齐 Java WORLD_SURFACE_WG）vs Rust `d > 0.0`（terrain.rs L279，不含水）→ **顶块为水的开放海洋列**，两臂扫描起点 p（surface.h L736 vs surface_rules.rs L1271）与 fluid_height r（surface.h L770 vs surface_rules.rs L1291-1293）不同：Rust 从海底 stone 顶起扫、顶层 run 的 fluid_height=i32::MIN（WaterCond 恒真，surface_rules.rs L93-97），C++ 从水面起扫、r=真实水面。该差可解释 (244,-60..-54,244) 类海洋列的谓词输入翻转，但 **195 列（幕帘在 y≥214，远高于海平面，顶块是 stone 非 water）heightmap 两臂相同 → 该列差异必须指向 (m1) NOISE rock/air 阈值边界或规则树内容差，不是 preliminary surface**。

## 1. est 逐维对拍表

| 维度 | Rust | C++ | 判定 |
|---|---|---|---|
| 密度函数 | `self.init` = `initial_density_without_jaggedness`（worldgen_handle.rs L249、L73 注释） | `R["initial_density"]` ← json key `initial_density_without_jaggedness`（worldgen_api.cpp L468；aquifer 构造 L802-803；SurfaceCondC fallback 的 `initialDensityAt` 同 DF，L1101-1106） | **一致** |
| 扫描起点 | `y = min_y + noise_height` = -64+320 = **320**（worldgen_handle.rs L572） | `l = minY + height` = **320**（aquifer.h L157；surface.h L187 同） | **一致**（260903-13 judge A1 修复 319→320 后） |
| 步长 / 下界 | `y -= 8`，`while y >= min_y` **含 -64**（L573-575） | `l -= 8`，`l >= minY` **含 -64**（aquifer.h L157） | **一致**（双端含，verticalCellBlockCount=8） |
| 阈值 | `> 0.390625` 严格大于（L574） | `> 0.390625` 严格大于（aquifer.h L159；surface.h L188） | **一致** |
| 哨兵（无命中） | `est = i32::MAX`（L571） | `INT32_MAX`（aquifer.h L156；surface.h L185） | **一致** |
| 列坐标量化 | shared 默认路径走 `va.aq.estimate_surface_height`（L563-566，注释 L557-561：内部 `(x>>2)<<2`）；独立路径**不量化**（L574 直采原始 x,z） | aquifer 版**量化** `(x>>2)<<2`（aquifer.h L146-148）；SurfaceCondC 单列 fallback **不量化**（surface.h L177-194） | **角坐标下无差**：四角坐标均 16 的倍数（L582-585），量化是 no-op；且 shared 默认开、E2' 运行时证两路径 sha256 逐字节同（b1 §9.0 R3）。⚠️ 残留不对称（非角列 C++ 双版本行为不一致）仅理论性，无人消费 |
| 四角坐标 | `(cx*16, cz*16) / +16 / +16 / +16`（L582-585，260903-13 #25 修复） | 同（worldgen_api.cpp L1086-1089） | **一致** |
| 消费（SurfaceCondC） | lerp2 + floor + `block_y >= k + surface_depth - 8` **对上不封顶**（surface_rules.rs L255-270） | 完全同构（surface.h L263-280） | **一致** |
| surface_depth 输入 | `(d*2.75 + 3.0 + extra*0.25) as i32`，minecraft:surface @y0 + splitter.nextDouble（surface_rules.rs L555-570） | 同式（surface.h L379-391） | **一致** |
| **运行时旁证** | WG_EST_DUMP 四角 = 24/32（b1 §9.0 R2） | 任务背景：两臂 est 四角同为 24/32 | **数值相等** → k≈24，幕帘整体在窗口内 |

**判读**：est 扫描域内任何一点若两臂 DF 采样值不同，属于 (m1) 密度函数实现差的下游，不属于 est 语义差。est 语义框架（本任务五维：步长/边界/阈值/d>0 vs d>阈值/流体列处理）中前四维**零差**。

## 2. 「d>0 vs d>阈值」与「含流体列处理」——真分叉点：heightmap 填充判据

est 本身不看流体；但 surface 扫描的**起点 heightmap** 两臂判据不同：

- **C++**：`if (block != air && wy > heightmap[bz*16+bx]) heightmap[...] = wy;`（worldgen_api.cpp L1045）→ 最高**非空气**块，**含水**（水面即 heightmap）→ 对齐 Java WORLD_SURFACE_WG（docs/07-block-pipeline.md L28「最高非空气块」）。
- **Rust**：`if top == i32::MIN && d > 0.0 { top = y; }`（terrain.rs L279，自顶向下首个 `d>0` solid，aquifer 分类前判据、**不含水**；L281 存入 surface_height；无 solid = i32::MIN，L239）。
- 消费同构：`p = column_h + 1`（surface.h L736/L748 vs surface_rules.rs L1259/L1271）；q/r/s 状态机逐行同构（surface.h L752-808 vs surface_rules.rs L1274-1341：air 清 q/r、fluid 置 r 保持 q、stone q++）。

**分叉后果（仅「顶块为水的列」触发）**：
1. 扫描起点：C++ p≈水面+1，Rust p≈海底 stone 顶+1 → 水柱内 C++ 会访问任何 stone（含幕帘穿水段），Rust 不访问。
2. fluid_height r：Rust 顶层 stone run 在遇到下方流体前 r=i32::MIN → `WaterCond` 恒真（surface_rules.rs L93-97），C++ 同 run r=真实水面 → 水下条件规则（mc10 族）判定翻转。E3 旁证：mod 侧 (244,-58) `fluid_height=MIN, sda=48`（b1 §9.2）——sda=48 = 从海底顶起跨流体累计（fluid 不重置 q，surface_rules.rs L1290-1293），正是「Rust 起点在海底」的签名。
3. SteepCond 邻居 heightmap：海洋列两臂邻值不同（surface.h L255-260 vs surface_rules.rs L248-251），坡度规则输入差。

**触发域界定（关键收窄）**：aquifer 水口袋上方有 stone 时，两臂 heightmap 都= stone 顶（max 非 air 不受下方水影响）→ **只有「最高非空气块=水」的开放海洋列**分叉。(244) 列（deep_lukewarm_ocean，深海）命中；(195,199) 列幕帘在 y≥214（海平面 ~63 之上），顶块是 stone → **不命中，两臂 heightmap/起点一致**。

## 3. 五段式证据链

| 段 | 内容 |
|---|---|
| **现象** | 两臂 est 四角同为 24/32，但同一 stone 幕帘 run 顶两臂各自刷不同 dirt 集（43 点级边界微差）；(244) 深海列 ref=gravel / mod=sand |
| **根因（机制，候选）** | est 语义六维逐项一致（§1 表）→ SurfaceCondC 窗口无差；真分叉 = heightmap 判据 `block!=air`（C++，含水）vs `d>0.0`（Rust，不含水）→ 仅开放海洋列的扫描起点/fluid_height/steep 输入差（§2）。195 列差异**不由本家族解释** |
| **定位** | grep preliminary/surface_height/heightmap 定位 C++ 两处 est（aquifer.h/surface.h）+ sh4 装配（worldgen_api.cpp L1085-1099）+ heightmap 填充（L1045）；Rust est/heightmap/SurfaceCondC 逐行实读对拍 |
| **修复方向（未做）** | 若要消海洋列分叉：Rust surface_height 改「首个非空气」（含水）语义，或单独维护 WORLD_SURFACE_WG 语义 heightmap 供 surface 用（注意 terrain.rs L283 biome 采样 by 也消费 top，需评估连带） |
| **教训** | 「preliminary surface」家族要拆两层：est 层（阈值扫描，两臂已同）与 heightmap 层（扫描起点判据，两臂不同）——「est 四角相等」不等于「surface 输入全相等」 |

## 4. 对裁决目标（m2）的正面回答

- **步长/起止边界/阈值/sentinel/DF 选择**：C++ 与 Rust **无语义差**（§1）。
- **d>0 vs d>阈值**：est 都用 0.390625；差在 **heightmap 填充判据**（C++ block≠air vs Rust d>0）——真分叉，但域收窄到「顶块为水的列」。
- **含流体列处理**：q/r/s 状态机同构；差源于上述 heightmap 判据（Rust 顶层 run fluid_height=MIN vs C++ 真实水面）。
- **能否解释 (244) gravel/sand**：可贡献谓词输入差（fluid_height/WaterCond/steep），但 E3 显示 mod 在 (244,-58) applied=无条件 gradient 段（biome 自洽 mr2=sand），ref 侧 gravel 更像规则树分支/biome 采样差（b1 §9.4-(m3)/(c)），**preliminary surface 不是其主因**。
- **能否解释 195 幕帘边界翻转**：**不能**（该列 heightmap 两臂同）→ 指向 (m1) NOISE d≈0/d 分类边界，或 C++ 代码规则树 vs Rust `build_overworld_rule` 的**规则树内容差**（知识库「数据驱动边界」候选；本次未逐节点对拍两棵树，idk）。**若幕帘翻转点全部位于开放海洋列则结论反转，需 (b) E4-lite 列剖面数据裁决**。
- 附加发现：C++ 内部 est 双实现量化行为不一致（aquifer 版量化 / SurfaceCondC fallback 不量化）——当前无人消费 fallback（sh4 恒 size 4），仅登记。

## 5. 诚实声明（未读/读不到）

- **Java 源码未直接读到**：`E:\PYTHON\MC\versions\1.20.1` 与 `E:\PYTHON\MC\data\mc_src_extract` 均无 `NoiseChunk.java`（glob 两查为空）。Java 语义引自项目已验证记录：computePreliminarySurfaceLevel = `for(l=minY+height; l>=minY; l-=cellHeight)` 320..-64 双端含（docs/07 L1149、10-timewise-archive L2758，注 forge sources NoiseChunk.java:174）、四角 `(i+1)<<4`（MaterialRules.java:496-499）、WORLD_SURFACE_WG=最高非空气块（docs/07 L28）——**本判据的 Java 对齐性为二手引用（candidate 级），非本次一手实读**。
- Rust `self.init` 是否含 jaggedness：已读 L73/L249 确认为 `initial_density_without_jaggedness`（解除 b1 §2.2 的 idk）。
- 两棵 surface 规则树（C++ `buildOverworldRule` 代码 vs Rust `build_overworld_rule`）未逐节点对拍——超出 m2 范围，idk。

## 6. 自检清单

- [x] 全部引用带 file:line，本次实读；Java 引用显式标注二手来源
- [x] 五个裁决维度逐一回答，未回避
- [x] 运行时旁证（est=24/32、E2' sha256、E3 fluid_height=MIN/sda=48）引自 b1 已落盘数据并注明
- [x] 分叉触发域已收窄（开放海洋列 vs 幕帘列），给出可否证预测
- [x] 未执行任何命令；置信度 draft，confirmed 留给人类
