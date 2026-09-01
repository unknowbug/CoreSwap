# 草稿：docs/09-multi-dimension.md 追加小节（2026-09-07）

> 用法：主会话将以下小节追加到 `versions/1.20.1/docs/09-multi-dimension.md` 末尾（追加不覆盖；supersedes 只加注记，原行不删）。数字全部来自本 session 已 judge 审查 / worker 定稿素材，confirmed 待用户。

---

## C2 预加载表数据驱动化：nether 噪声 key 从 surface_rule JSON 构建期收集（candidate，2026-09-07，commit 709b006）

> 承接「E7 修复」节：E7 修复 = 手工补齐 nether 6 key 清单；本节 = 同一问题的架构层收尾（数据驱动化，对齐 AGENTS.md 数据驱动铁律）。

### 改动内容

- `worldgen_handle.rs` step4 预加载表数据驱动化：新增 `collect_noise_keys()`，从 surface_rule JSON 构建期自动收集 noise_threshold 引用的 noise key，预加载表不再依赖手工清单。
- **overworld 保留静态清单**：overworld 为代码规则（`build_overworld_rule`，无 JSON 源），数据驱动化不适用，静态清单保留（已验证代码规则不动）。
- **nether 静态 6 key 清单删除**（E7 手工补的清单由 JSON 收集取代）。

### 验证（存档口径，同 E7 口径）

- 3 连跑 **93.8988% 逐位同值**，无回归。
- judge C2 CONCERN 已闭环。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照（同 E7/B1 口径）；覆盖面：4×4 chunk 全高度（min_y=0, height=256）。
- 可比性：与 E7 修复后基线 93.8988% 同口径可比；本节为重构性质（行为不变验证），非改善量声明。

### 状态

- candidate；confirmed 留用户。过程 → 10 时间线 2026-09-07 条。

---

## 矿石归因定论：双重 feature 应用（candidate，2026-09-07，judge PASS 建议 candidate）

### 机制（H_B'）

- `wg_fill_blocks_multi` 内含 **carver + feature 阶段**（`worldgen_handle.rs` L442-449，`WG_SKIP_CARVER` / `WG_SKIP_FEATURES` env 门控）。
- 存档链路 mixin 只拦 **populateNoise + cancel buildSurface**，Java 侧 CARVER / FEATURES 步骤照跑 → **存档 = Rust features + Java features 双跑**。
- 修正早前结论：09 篇「SURFACE 口径残差量化」节曾写「cppReplace 架构只拦截 populateNoise + buildSurface，features 只由 Java 运行一次（无双跑通道）」——该判断对 mixin 拦截范围描述正确，但漏了 Rust 侧 `wg_fill_blocks_multi` 内含 feature 阶段这一半，双跑通道实存。**[注 2026-09-07]** 原行不删，以本节为准。

### 消融证据（seed B，4×4 @3200,3208，存档口径）

| 实验 | match | 矿石计数变化 |
|---|---|---|
| 基线 | 93.8988% | quartz 4478 / gold 1525 / magma 3814 |
| +WG_SKIP_FEATURES=1 | **94.4241%**（+5508 块） | quartz 2125（ref 1992）/ gold 739（ref 728）/ magma 1979（ref 1533） |
| +WG_SKIP_CARVER | 仅再 +370 | — |

### 结论（candidate）

1. 矿石 ~2.2× 偏高**全额归因 Rust features 双跑**（SKIP_FEATURES 后三族矿石均落回 ref 邻域）。
2. carver 双跑贡献小（+370），非主导。
3. **遗留（idk）**：overworld 同路径理论上也双跑，但 overworld 存档对齐 99.9%——是否同样双跑及为何不显形，待 X1 FEATURELOG 裁决（进行中，结论回填时间线，不在本节预写）。
4. **修复方向 judge CONCERN**：`WG_SKIP_*` 是 env 门控（进程全局），修复勿用全局默认翻转——需句柄/调用级显式 flag，避免污染其它调用方。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解 vs vanilla FULL 参照；覆盖面：4×4 chunk 全高度，seed B。
- 可比性：消融各列同为存档口径，列间可比；与 SURFACE 口径 77.49% / 纯 Rust 口径 77.43% 不可比，分列。

### 状态

- candidate（judge PASS 建议）；confirmed 留用户。

---

## soul sand valley 归因三签名（B2 定稿，2026-09-07）

### 上轮假设证伪（supersedes 注记）

> **supersedes**：本节推翻 docs/09 早前小节「soul_soil 大头疑似在 Java feature 阶段，属 B1 主导机制的正常残差」表述（原行不删，加注记）。

- **证伪证据（V1）**：Rust 管线自身 soul_soil 1363 ≈ 存档 1334——Java feature 阶段并未产出 soul_soil 大头；缺口 4140（ref 5474）**在 Rust 管线内部**（surface 层机制缺失），非 Java 侧残差。

### V2 探针三签名（180 点，三签名）

| 签名 | 现象 | 判读 |
|---|---|---|
| A | biome 足迹偏移/收窄：valley 点 Rust 判 nether_wastes，聚簇 x≥3410 边界带 | biome 判定边界带差 |
| B | soul_soil 子分支失效：entered + selector<0 仍 applied=netherrack | surface rule 子分支未生效 |
| C | floor 侧 soul_sand_layer 分支疑似缺失：组3 entered 0/60 | 候选缺失（待结构对拍确认） |

- 候选分派：**.b1a（结构差）主导**；**.b1b（噪声值偏离）idk**——缺 Java 同点对照，诚实标注未决。
- Java features 对 soul_sand 为**净回补 +587**（与矿石双跑偏高方向相反，单独记）。

### 下一步（优先级序）

1. **V3：Rust-vs-JSON rule 结构对拍**（零成本，最高优先）；
2. V4：RouterProbe 同点 selector 对比；
3. V5：biome 边界带对比。

### 口径声明（§9.7 三要素）

- 载体：V2 = Rust 管线探针 + 存档读回 + ref 三方；覆盖面：180 采样点（含边界带聚簇）。
- 可比性：V1 Rust 管线口径 vs 存档口径载体不同，分列引用；.b1b 无 Java 同点对照，不构成可比结论（idk）。

### 状态

- 三签名 = worker 定稿（candidate 级）；.b1b idk；confirmed 留用户。过程 → 10 时间线 2026-09-07 条。
