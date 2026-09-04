# b2 候选草稿：surface rule / disk 特征差 vs FULL 残差签名

- 候选：.b2（fan-out residual-260904-04）
- 置信度：**draft**（Degraded = 纯静态审查，无运行时探针）
- 输入数据：`.tmp/p2full/cmp_yhist-260904-04.txt`、`cmp_spatial-260904-04.txt`、`cmp_full2.out.txt`（seed 8576294172403134396，4×4 @ (200,200)，残差 13328/1572864，99.1526% 一致——口径：存档 FULL 对比）
- 代码证据：`versions/1.20.1/cpp/worldgen/src/surface.h`（859 行全读）、`feature.h` L380-457（DiskFeature）、`WorldgenRust/src/surface_rules.rs`（L1-1065 读）

## 结论（先行）

**基本不相容 / 主导残差可排除 surface rule 与 disk 特征作为根因**（可解释比例估算 <35%，且结构上无法覆盖残差的最关键签名）。逐签名论证如下。

## 1. 相容性论证（逐带定量核算）

### 1.1 surface rule 的作用域硬约束（C++ surface.h buildSurface L757-807）

三条结构性约束，任一独立即可排除大块残差：

- **约束 A（只动 default 块）**：`if (state == defaultBlock)`（L795）——surface rule **只替换 stone**。残差中 ref 侧非 stone 的 cell（gravel→sand 1687+898+653+136…、gravel→dirt、water↔air ~816、sand→dirt 363、sand→clay 等）合计约 5500+ cell，**结构上不可能由 surface rule 分歧产生**（Java 侧同样只对 default 块 apply）。即 ≥41% 的残差与 surface rule 无关，无论实现多么错误。
- **约束 B（深部规则集为空）**：y<-32 深部能命中的规则只剩 bedrock_floor（y≤-59，产物 bedrock）与 deepslate（vertical_gradient 0/8，产物 deepslate）——**都不产 dirt/sand/gravel**。而深部残差 y[-64,-32)=1041 + y[-32,0)=1104 ≈ 2145 cell，其中 stone→dirt(deep) 298 + stone→sand(deep) 109 + gravel→sand/dirt(deep) 267。关键判别带 **y=-64..-59**：残差 38+39+37+35+37+32=218 cell，此带唯一适用的规则是 bedrock_floor，不可能产 stone→dirt/sand——surface rule 假设在此带被**直接证伪**。
- **约束 C（y 局部性）**：若 est（above_preliminary_surface）或 stone_depth 差是根因，残差必集中在地形表面 est 窗口（±surface_depth≈3-8，即单一带状 y 区间）。实测 yhist **全 384 个 y 每层 27-46 均匀（≈34.7/y）**，无任何峰——与一切 y 局部化机制（surface rule、disk）不相容。

### 1.2 disk 特征的作用域约束（feature.h L428-449）

- disk = 以 placement y 为中心 half_height±（典型 1-4）的椭圆柱，**垂直跨度 ≤ ~9、水平半径 2-6**；且 C++ placement 高度图取 y 语义（placement.h L209，+1 使残差变差的历史实测）。它无法产生**跨 y -64..191（跨度 255）的 3D 连通 blob（最大 615 cell）**——blob 的 y 跨度是 disk 垂直跨度的 ~28 倍。
- 16/16 chunk 全有残差且每 y 密度均匀 ≈35：disk 是稀疏 feature（每 chunk 个位数），不可能铺满所有 y 带。
- **y>250 带（192-256: 1116+1210，256-319: 1174+1067）的 stone→dirt**（surface 带内 n=1362，且 y=288-319 带 1067 cell）：disk 落点为水底/湖底高度图，不可能出现在 y 250+ 的实体地形带；surface rule 若在此带触发则同时应受约束 B 反证深部。

### 1.3 定量汇总

| 残差子集 | cell 数（约） | surface rule 可解释？ | disk 可解释？ |
|---|---|---|---|
| ref 非 stone 源（gravel/sand→*, water↔air） | ~5500 (41%) | ❌ 约束 A | water↔air ❌；gravel→sand/dirt 部分可能 |
| y≤-59 带（218） | 218 (1.6%) | ❌ 约束 B | ❌ |
| 全 y<0 带（2145） | 2145 (16%) | ❌ 约束 B | ❌（深部湖底磁盘除外，极小） |
| y>250 带（2200+） | ~2200 (17%) | 仅地表附近 | ❌ |
| blob 跨 y -64..191 | 615+ | ❌ 约束 C | ❌（跨度 28×） |
| 地表附近 stone→dirt/sand（y 0-191） | ~4000 (30%) | 结构上可能但 yhist 均匀性否定 | 否 |

**结论：两个候选机制的可行交集 ≤ 剩余的一小部分，且被 yhist 全带均匀 + blob 跨度两个签名同时否定为主因。**

## 2. 代码覆盖检查（C++ surface.h vs Rust surface_rules.rs）

- 规则树结构（mc1-mc18、mr-mr9、bedrock_floor/deepslate 终段）逐段对齐；StoneDepth 的 k 映射（不 clamp + (int) 向零截断）两侧一致。
- **发现一处 Rust↔C++ 静态分歧（次级观察，不足以解释主签名）**：windswept_hills / windswept_savanna / windswept_gravelly_hills / old_growth_taiga 的 surface 噪声阈值——C++ 用原始值 `1.0 / 1.75 / 2.0 / -0.95`（surface.h L519/541/544-546/587-588），Rust 一律 `/8.25`（surface_rules.rs L669/726/734-742/837/841）。二者必有一侧对 Java 语义翻译错误（只影响 windswept 系地表 stone/coarse_dirt 判定，仍是地表局部，无法解释全 y 均匀）。
- C++ DiskFeature L443 注释对 Java placeBlock「找到即 break」语义存疑（一行注释自我反问），gravel→sand/dirt 子集（~4100 cell，41% 的 ref-gravel 源）**唯一可能**与 disk/badlands 分歧相关的部分——但这不能解释 water↔air 与深部 blob。
- water→air(339)/air→water(116) 指向含水层（aquifer）/carver/湖泊流体阶段，非 surface/disk。

## 3. biome 核对（本区域）

- @anchor.idk("chunk(200,200)-region 16 chunk 的 biome 名称未确认：.tmp/p2full blocks 文件（WGB2）含 per-chunk biome 字符串段且 cmp_full2.py 有 biome diff 检查但输出无 biome 差异行（两侧 biome 大概率一致），本 subagent 沙箱无 shell 无法运行 python 提取名称")——主会话可执行：`python .tmp/p2full/cmp_full2.py` 前先加打印 biomes，或用 cmp_full2.py 的解析函数 dump `a['chunks']` 的 biome 字段。
- biome 一致性本身已由 cmp_full2 输出旁证（脚本会对 biome 差异打印，实际无）——即使 biome 有错也不改变 §1 的结构排除论证（约束 A/B/C 与 biome 无关）。

## 4. 判别实验建议（主会话可执行探针模板）

1. **band 排除实验（最廉价，先做）**：对 C++ 侧单独关闭 surface rule 应用（buildSurface 后 col 保持 NOISE 阶段 stone）重跑 4×4 FULL 对比——若残差几乎不变，证明残差不在 surface 阶段（预期成立）；同理对 feature 阶段（disk 关闭）做 A/B。
2. **SURFTRACE 定点**：`WG_SURFACE_TRACE=<x,z>`（surface.h 已有 wg_surfaceTrace 钩子）采一个深部残差点（从 cmp_spatial 选 chunk(12,15) y=-64..-49 comp 内坐标），观察 rule->apply 是否被调用及命中哪条规则——预期「非 default 跳过」或无命中，直接证伪 surface rule 源。
3. **阶段剥离对比**：导出 CARVERS 后 / FEATURE 前的中间 blocks 快照对比（若探针支持阶段导出），把残差定位到生成阶段（与 -288 管线勘探法一致）。
4. **水/空气对专项**：对 water↔air 339/116 的坐标跑 aquifer 探针（density_probe/aquifer 对拍），这是独立于 b2 的候选方向。

## 5. 附：Rust↔C++ 阈值分歧清单（静态，待独立验证）

| biome | C++ surface.h | Rust surface_rules.rs |
|---|---|---|
| windswept_hills | 1.0 | 1.0/8.25 |
| windswept_savanna | 1.75 | 1.75/8.25（mr7）、1.75/8.25 与 -0.5/8.25（mr8） |
| windswept_gravelly_hills | 2.0 / 1.0 / -1.0 | 2.0/8.25 / 1.0/8.25 / -1.0/8.25 |
| old_growth_taiga | 1.75 / -0.95 | 1.75/8.25 / -0.95/8.25 |

（注：本对比基于 FULL 残差数据为 Java ref vs C++ 侧；Rust 侧规则树差异是旁支发现，供后续候选使用。）

——draft，未做任何运行时验证（Degraded 静态审查声明）。
