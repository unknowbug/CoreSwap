# 主题篇追加小节草稿 — 「nether 存档写入口径 Full 化（1.0.22 dll，双 seed）」

> 草稿（knowledge worker 产出，待主会话应用 + 验证）。
> **目标篇目**：`versions/1.20.1/docs/09-multi-dimension.md`（nether 维度课题归口篇，含 M3/M7/M13 nether 表面规则/熔岩/双高度既有小节，本节数据与其残差家族直接衔接）。备选：若主会话按「存档写入管线」归类，可落 07-block-pipeline.md，但 09 篇与既有 nether surface rule 遗留课题上下文最连续，**推荐 09**。
> 纪律：追加不覆盖；本节所有结论 **status: candidate**（judge-review #1-#15 建议），confirmed 留人类授予；机制解释部分保持 draft。

---

## nether 存档写入口径 Full 化（1.0.22 dll，双 seed）（candidate，260901-03）

> 载体：Rust nether 接管 gen（cppReplace）→ 存档落盘 → MCA 直解（compare_save_region.py）vs vanilla BlockProbe 参照（WGB2）+ ReadWorldProbe 内存读交叉验证。dll sha256=C5AC5309F3C59A044（1.0.22 M17）。
> **口径声明（v0.20 §9.7 三要素）**：① 载体 = MCA 存档直解 + 内存读，vs vanilla 参照；② 覆盖面 = 4×4 chunks @(3200,3208) 全高度（nether min_y=0，height=256，动态读取）；③ **与 96.44% 探针口径（docs/09 既有数字）不可比**——载体不同（存档/内存 vs 探针直采），数值禁止直接互比。seed 三查：server.properties ↔ level.dat ↔ ref header 全同值。
> 过程错误（首轮三场 run enabled=false 全作废、cppWorldgenDir 错层等）见 `.investigations/nether-save-full/nether-save-errors.md`（独立台账，不在此重复）。

### 双 seed 三口径数字表（candidate）

| seed | 内存读（ReadWorldProbe） | 存档读回（reconfirm，从盘读） | MCA 直解（compare_save_region） | 残差块数 |
|---|---|---|---|---|
| A = -2032795982907864146 | 99.9376%（1047922/1048576） | 99.9376%（**与内存精确同值**） | 99.9278%（1047819，差 103 = cave_air 簇） | 757（MCA）/ 655（内存） |
| B = 8576294172403134396 | 93.5156%（980582/1048576） | 93.5156%（精确同值） | 93.5156%（精确同值） | 67,994 |

- Rust 真实参与证明：v2 log `enabled=true` + 64 条 `populateNoise(nether) intercepted`（4×4 目标 + feature 蔓延邻域）。**验收判据：enabled 标志 + intercepted 覆盖目标 chunk，缺一 run 作废**（首轮教训，见错误台账 E1/E3）。
- seed B 残差全部落在 y≤127（layerMatch：y0..31=82% / 32..63=88% / 64..95=88% / 96..127=90%；**y≥128=100%**）——与「noise_height=128、y≥128 留 air」（09 篇 M3 教训）自洽；本次 run buildSurface 被 Mixin skip，存档表层全来自 Rust 生成，残差 = vanilla surface rule 输出 vs Rust 生成的差异。

### 残差机制分类占比（分类 = 数据直读，candidate；机制解释 = draft）

| seed | 类别 | 块数 | 占比 | 机制候选（draft） |
|---|---|---|---|---|
| A | A1 nether 矿石 feature 差（quartz/gold/debris，方向全为「vanilla 有矿→存档无矿」） | 640 | 84.5% | Rust nether ore feature 未放置或错位（与 B4 同家族；09 篇已知缺口「fill_chunk_blocks nether 逻辑差异化」的存档级量化） |
| A | A2 air↔cave_air 尾随簇（单 chunk(203,200) y70-72） | 104 | 13.7% | **未闭合**（见下） |
| A | A3/A4 magma 点差 / 熔岩湖边界点差 | 13 | 1.8% | seed B 大类的缩微版 |
| B | B1 basalt deltas 三大宗石互换（basalt↔blackstone↔netherrack，成片双向） | 52,078 | 76.6% | surface rule 条件链系统性偏差（biome 判定 / noise 阈值 / Hole 语义 `surface_depth<=0` vs `stoneDepthAbove<=0` 的下游表现之一，未验证） |
| B | B2 soul sand valley 涂布边界 | 5,720 | 8.4% | 吻合 09 篇已知遗留（y=1..2 soul_sand_valley 表面残差），块数放大 |
| B | B4 矿石（与 A1 同家族） | 2,629 | 3.9% | 同 A1 |
| B | B5 magma / B3 熔岩湖边界 | 3,069 | 4.5% | magma：underwater_magma/邻接判定归属未定；湖边界：M7 seaLevel 机制已修、边界条件残差（已知遗留） |
| B | 未分类（top15 以下散点） | 4,498 | 6.6% | — |

### 未闭合待查项（全部 draft/待查）

1. **103 cave_air 簇机制**：v2 下 seed A 内存 = 存档读回**精确同值**（无 cave_air），MCA 直解却多 103 块 air→cave_air（同 chunk 同簇，y69=0/70=4/71=23/72=53）——「同一次落盘、两种读取口径不同」的新形态矛盾；Rust 全代码零写 cave_air，b1（时序）/b3（非确定）候选均未闭合。探针方向：M4（biome.rs BTreeMap）复核、禁 carvers/features 重跑定位尾随写入者、save 前后 hook dump。
2. **basalt deltas 大宗互换（B1）**：76.6% 大头，层位/形状已锁定（y≤127、按区域成片）——surface rule 单列对拍 + 按 biome 分桶可定位。
3. **矿石 features 缺口（A1+B4，3,269 块）**：「未实现」vs「放置错位」归属未定——feature 阶段 A/B diff 出 Rust 实际矿位 vs 参照矿位集合即可裁决（若错位，按发现 #6 查 PlacedFeatureIndexer 编号链）。

### 下一步深挖优先级（块数 × 可定位性，residual-interpretation §3）

1. **B1 表面规则大宗互换**（52,078，可定位性高：biome 分桶 + 单列 surface rule 逐步对拍）
2. **A1+B4 nether 矿石 feature**（3,269，中高：一次探针双 seed 受益，A1 占 seed A 残差 84.5%）
3. **B2 soul sand valley**（5,720，中高：已知遗留，限 y∈[0,4] 切片 diff）
4. **B3 熔岩湖边界**（1,375，中：y 直方图看是否聚 sea_level 附近）
5. **A2 cave_air 簇**（104，价值在排除 M4 家族复发而非块数）
6. **B5 magma**（1,694，中低：与 #1 同脚本分桶）

> 状态：数据、口径声明、分类占比 = **candidate**（judge-review #1-4/#15 建议，260901-03）；机制解释与待查项 = draft；**confirmed 留人类**。
> 关联：`.investigations/nether-save-full/`（facts / .b1-.b3 / residual-interpretation / judge-review / nether-save-errors.md）。
