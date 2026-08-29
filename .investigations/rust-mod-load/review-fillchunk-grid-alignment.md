# Review — candidate「fill_chunk 宏观采样对齐 Java Interpolated 网格架构」立项审查

- 审查角色：core.judge（subagent 隔离）
- 审查对象：性能优化方向 candidate——是否值得立项「宏观网格采样对齐 Java（~1225 网格点 vs Rust 逐点 98304，80×）」以解决 Rust 慢 5 倍
- 审查日期：2026-08-29
- **本审查只出意见，不改任何 status；confirmed 由人类授予**

---

## 一、数据源核对 / 三源核对

| 数据源 | 内容 | 状态 |
|---|---|---|
| `.investigations/rust-mod-load/cmd-output/aquifer_internal_precise.txt` | aquifer 内部无污染精确定位 | ✅ 已读 |
| `.investigations/rust-mod-load/cmd-output/grid_sampling_correction.txt` | 网格探针 2000ms 记录 | ✅ 已读 |
| `.investigations/rust-mod-load/cmd-output/e2e_java_vs_rust.txt` | 端到端 44.9ms vs 8-9ms | ✅ 已读 |
| `WorldgenRust/src/bin/grid_sampling_bench.rs` | 网格探针源码 | ✅ 已读 |
| `WorldgenRust/src/terrain.rs` | fill_chunk 逐点采样实现 | ✅ 已读 |
| `WorldgenRust/src/density.rs` | InterpolatedData (CELL_X/Y/Z, build_grid) | ✅ 已读 |
| `WorldgenRust/src/aquifer.rs` | Aquifer.apply / calculate_density / get_fluid_level | ✅ 已读 |
| `WorldgenRust/src/bin/density_interp_diag.rs` | 双层污染诊断探针源码 | ✅ 已读 |
| `WorldgenRust/src/bin/density_interp_single.rs` | 单层对照探针源码 | ✅ 已读 |
| `WorldgenRust/src/bin/density_interp_bench.rs` | 原探针（报告 100×/44%）| ✅ 已读（源码）|
| `.investigations/rust-mod-load/cmd-output/fillchunk_profile.txt` | fill_chunk 分段剖面 | ✅ 已读 |
| `.investigations/rust-mod-load/cmd-output/aquifer_internal_profile.txt` | aquifer 分段剖面 | ✅ 已读 |
| `.investigations/rust-mod-load/cmd-output/density_tree_profile.txt` | finalDensity 树结构 | ✅ 已读 |
| `.investigations/rust-mod-load/cmd-output/base_breakdown.txt` | base 三段细分 | ✅ 已读 |
| `.investigations/rust-mod-load/review-density-candidate.md` | **前次 judge 审查（2026-08-29，含运行复现数据）** | ✅ 已读 |

**三源核对（spec §4）**：
- git HEAD/工作区 diff：`grid_sampling_bench.rs` 与全部 70 个 src/bin 探针**已 tracked**；`.investigations/rust-mod-load/` 25 文件已 tracked。审查基于工作区当前文件，源码与应用版一致（无 git 滞后）。
- **关键**：candidate 依据的 `grid_sampling_correction.txt`（2000ms 不可行）与**前次 judge `review-density-candidate.md` 的运行复现数据冲突**——前次审查已实测证明「2000ms = 双层污染，非方向不可行」。本 candidate 未吸收该已落盘结论，重复了已被推翻的测量。

---

## 二、逐项核对

### ① 「采样次数差 ~80× 是根本」—— **结论成立但被错误应用（mis-applied）**

- 成立部分：Java final_density 树用 Interpolated 网格（~1225 点，CELL 4x4x8）替代逐点 98304，将 **density 树采样** 从 98304 次降到 ~1225 次——80× 采样差存在。
- **但**：① Rust final_density 树**内部已含 5 个 Interpolated 通道**（density_tree_profile.txt：`Interpolated 5`），**内层网格缓存已存在**；density 每 chunk 实测仅 **~14ms**（fillchunk_profile），**不是**慢 5 倍主因。② **aquifer 是逐块独立采样器，Java 同样不插值**——「~80×」**根本不适用 aquifer**。
- ⇒ **致命**：candidate 把「80× 采样差」当作「Rust 慢 5 倍的根本」，但 base 最慢的是 **aquifer 17.5-40ms（43-64%）**，而 aquifer 恰恰是 grid-interpolation 覆盖不到的逐块逻辑。**「根本」归因错误。**

### ② grid_sampling_bench 2000ms 是探针缺陷（雪崩）—— **确认是缺陷，非 Rust 网格采样不可行**

- 决定性证据来自**前次 judge 的运行复现**（review-density-candidate.md + density_interp_correction.txt）：
  - 内层 Interpolated 的 grid 采样：裸树每 chunk **6029 次**；外层再包 Interpolated 后**膨胀到每 chunk 175 万次（291×）**——雪崩机制确认（外层 build_grid 网格点跨 chunk 边界，反复清空内层 chunk 网格）。
  - **单层 Interpolated 包装纯 SplineDF（sloped_cheese，无缓存污染）= 加速 70×**（83.74ms → 1.19ms）。
- ⇒ **2000ms 是双层 Interpolated 包装缺陷，不是「网格采样在 Rust context 不可行」**。正确架构下单层 Interpolated 高度有效。candidate 的 `grid_sampling_bench` 复现了同一双层缺陷（直接 `df.sample` 外层网格点，df 内部已含 interpolated）→ 得到 320× 慢。

### ③ 正确对齐 Java 网格架构的可行性 —— **可行，但 key 是「每密度子树单层」，不是「final_density 外层再包一层」**

- 现有 `InterpolatedData`（density.rs L220-315）已实现正确 per-chunk 网格（CELL_X=4/Y=8/Z=4, build_grid gx=5/gy=49/gz=5=1225 点）+ 三线性插值。**基础设施已就位**，无需新建。
- 已接入：`minecraft:interpolated` → `DensityFunction::Interpolated`（density_builder.rs L285）。
- fill_chunk 当前用 `VanillaDensity{df:&tree}` 直逐点采样（terrain.rs L30-32, L75）；tree 内部已含 5 个 Interpolated → **已获得内部网格缓存**。再包外层 Interpolated = 冗余双层 = 雪崩（100×/291× 已证）。
- ⇒ **正确路径**：对 final_density 内**各纯 SplineDF 子树**（如 sloped_cheese）应用**单层** Interpolated 网格缓存（70× 加速来源），而非宏观再包一层。

### ④ 正确性风险（三线性 vs 精确）—— **「44% 差异」是双层污染，非插值本身破坏**

- 前次审查结论：「44% 差异」= 双层二次插值偏差 + 对插值语义的误解。**MC 密度本质是插值的**（final_density 内部已 4x4x8 interpolated），「精确逐点」在含 5 个 Interpolated 的树上**本来就是插值混合**。
- 单层插值误差是固有精度产物——对齐目标是**匹配 Java 自身的网格/三线性语义**，不是「精确 vs 插值」。正确单层配置的对齐应匹配 Java 自身插值误差。
- ⇒ 正确性风险真实但**可控**，属于「对齐 Java 网格坐标/插值语义」的验证问题，非「插值不可用」。

### ⑤ 是否值得立项 —— **该具体 candidate（宏观网格采样）不值得按此框架立项**

- 它瞄准 density 树（~14ms，已基本插值化），却**回避真正的最大成本 aquifer（16-40ms）**= 治标不治本。
- **更优替代**（按收益排序）：
  1. **直接优化 aquifer 每点开销**——最大头（43-64%）：get_block_pos 3×3 邻域 18 次/点 / calculate_density 至 3 次/点 / get_fluid_level。这是 Java 同样逐块的开销，是 5× 差距核心，网格采样不覆盖。
  2. **单层 Interpolated 应用于 final_density 内纯 SplineDF 子树**（sloped_cheese 70×）——真实但收窄密度段（~14ms）。
  3. **DFC 直排**（AGENTS.md 铁律：SplineDF 树遍历=慢根源，方向=C2ME 式 DFC 直排 + 网格缓存）。

---

## 三、审查意见

### 置信度
- **「80× 采样差 = 根本」**：❌ **不成立**——归因错误，aquifer（Java 同样逐块、不被插值）才是最大成本，该方向治不了 5×。
- **「2000ms = 网格不可行 → 放弃宏观采样」**：❌ **站不住**——2000ms 是双层污染（前次运行数据证 291× 膨胀 + 单层 70× 有效）。
- **「正确对齐 Java 网格架构可行」**：✅ 可行，但必须**每密度子树单层**，现有 InterpolatedData 可复用，禁止 final_density 外层再包。

### 立项建议
- **不按「宏观网格采样对齐」立项**（范围错，治标不治本，且基于已推翻的测量）。
- **建议立项方向**（高收益 + 证据充分）：
  1. **aquifer 每点开销优化**（首推，最大头，Java 同样成本，是 5× 真正核心）；
  2. **单层 Interpolated 应用于纯 SplineDF 子树**（70× 实测，收窄 density ~14ms）；
  3. 后续评估 **DFC 直排**（AGENTS.md 铁律方向）。

### 设计要点（若坚持做密度段网格化，避免雪崩的关键）
1. **绝不**在 final_density 外层再包 Interpolated（=双层雪崩，291×）。
2. 对**单独的纯 SplineDF 密度函数**（sloped_cheese 类）用 InterpolatedData 单层包装，复用现有 build_grid/三线性。
3. **跨 chunk 网格复用**验证（每 chunk 网格首建 ~14ms 能否跨 chunk 复用降耗，前次 judge 遗留待办）。
4. 对齐验证：确认网格交点坐标（`ix*4/iy*8/iz*4` floor_div chunk 归组 + 边界 clamp）与 Java 网格完全一致，避免坐标错位（#23/#24 教训）。
5. 测量纪律：单线程 + WG 门控 chunk 级判断，规避测量污染铁律（诊断不逐点执行）。

### 三源核对结论
- 数据记录（grid_sampling_correction/aquifer_internal_precise/e2e）↔ 探针源码 ↔ fill_chunk/density/aquifer 实现：全部对齐。
- **但 candidate 与已落盘的 `review-density-candidate.md` 结论冲突**——前次 judge 已证 2000ms=双层污染、70× 单层有效，本 candidate 未吸收，重复了被推翻判据。**审查基线不一致**（candidate 载体 grid_sampling_correction 未记录「双层污染」背景）。

### 推荐状态
- **保持 draft，不立项当前 framing**。方向修正为：a) aquifer 每点开销优化（主）；b) 单层 Interpolated 于 SplineDF 子树（次）；c) 补记 grid_sampling 2000ms = 双层污染的背景到记录，避免再次误判。

---

*本意见为建议非命令，confirmed 由人类授予。*
