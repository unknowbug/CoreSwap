# 草稿：追加到 versions/1.20.1/docs/09-multi-dimension.md 的小节（draft，未应用）

> 本文件为 subagent 产出草稿，主会话审后应用。追加位置：09 篇「nether_state_selector 预加载表修复」节之后。
> 置信度：**candidate**（待 judge + 用户拍板 confirmed）。验证分层：**Partial**（SURFACE 参照口径 = BlockProbe 无 carvers/features，端到端存档口径分列声明）。
> 勘误注记：本节与前节 noise JSON 路径应为 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise/*.json`（前节草稿写漏 worldgen/data 两级，应用时以本节为准修正）。

---

## SURFACE 口径残差量化：Rust surface 层自身残差 = 22.5%，主导形态 = basalt/blackstone 位放 netherrack（candidate，260901-04）

> 承接「B1 定论」节 judge WARN-4 待排除备择：「Rust surface 薄带残差在 52k 中的量级未单独量化」——本节以 SURFACE 参照口径闭合（计划项 2）。

### 采集与口径

- vanilla SURFACE 参照：BlockProbe 默认口径（无 carvers/features；`-PblockProbe -PblockProbeDimension=nether`，**不带** blockProbe.full），seed B = 8576294172403134396，4×4 @3200,3208。export log = `.investigations/nether-save-full/cmd-output/b2-surface-ref-export.log`（`BlockProbe worldSeed=` 核对一致）。
- FULL 参照已备份 `.blocks.full`，hash 不同确认口径切换生效（SURFACE 270D6E97… vs FULL 1DDE3B09…）。
- 对比脚本：`.tmp/b2_surface_residual.py`；纯 Rust 侧 = rlib dump（`.tmp/b1-rlib-blocks.bin`）。

### 数据（数据直读）

| 对比 | 数字 | 判读 |
|---|---|---|
| SURFACE 参照 vs FULL 参照 | diff 仅 21,296/1,048,576（**97.9691% identical**） | 本 4×4 区域 features 贡献 ~2%；黑石/玄武岩大宗主体在 surface rules 层（SURFACE 参照 basalt = 173,073 vs FULL 172,704，几乎不变） |
| SURFACE 参照 vs 纯 Rust rlib dump | **77.4857%**（match = 812496/1048576） | Rust surface 层自身残差 = 22.5%（SURFACE 口径） |
| 分族 | solid_solid 233,197 / ref_solid_rust_air 2,871 / ref_air_rust_solid 12 | 残差几乎全是实心块互换，非实空差 |
| top mismatch | basalt→netherrack 157,658 / blackstone→netherrack 35,031 / cave_air→netherrack 15,678 | 主导形态 = vanilla surface 放 basalt/blackstone 处 Rust 放 netherrack |

### 结论（candidate）

1. **Rust surface 层自身残差 = 22.5%**（SURFACE 口径），量化了 B1 定论中「薄带残差」的真实量级——不是薄带，而是 surface 层大宗差异（basalt/blackstone→netherrack 为主）。
2. **存档口径 93.8988% 说明 Java features 在 Rust 基底上补齐了其中大部分**——与 B1 定论「feature 产物 × 两种基底地形差」自洽：同一套 Java feature 在 vanilla 基底与 Rust 基底（netherrack 化的表面）上命中/形态不同，端到端差异被大幅压缩。
3. **judge WARN-4 备择「Rust 已实现 feature 与 Java feature 并存重复放置」可排除**：cppReplace 架构只拦截 populateNoise + buildSurface，features 只由 Java 运行一次（架构事实，无双跑通道）；且 SURFACE 口径 Rust 侧残差形态（basalt/blackstone→netherrack）与存档口径收敛方向一致。
4. **⚠️ 外推限制**：本区域 FULL−SURFACE 差仅 ~2% 是 4×4 局部观察（basalt deltas 宗石恰好 surface 主导），**勿外推为全局 features 贡献占比**。

### 口径声明（§9.7 三要素）

- 载体：SURFACE 参照 = BlockProbe 默认口径（无 carvers/features）vs 纯 Rust rlib dump。
- 覆盖面：4×4 chunk 全高度（min_y=0, height=256），seed B = 8576294172403134396。
- 可比性：**77.4857%（SURFACE 口径）与 93.8988%（存档口径）载体不同不可比**，分列；与 B1 定论节纯 Rust 口径 77.43%（FULL 参照）亦不同载体，分列——77.4857% vs 77.43% 数值接近纯属本区域 features 占比低的巧合，不构成口径可合并的证据。

### 状态

- 置信度 candidate；confirmed 留用户。过程 → 10 时间线 260901-04 条（如需可由主会话合并应用）。
