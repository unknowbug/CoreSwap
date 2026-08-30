# judge 审查意见：transpiler 4 公式修复 + 接入生产（收尾三源核对）

> 审查角色：core.judge（subagent，隔离执行）。
> 审查对象：2026-08-30 session 对 build-time transpiler 做两件事——① 修复 4 个数学公式生成错误（M12a-d）；② 接入生产（TranspilerDensity + WG_TRANSPILER 门控）。
> 审查标准：core-judge 清单（证据完整性/落盘/三源核对/置信度合法/产物契约/噪声卡/retry cap/模块边界）。
> 审查基线：① `.investigations/macro-layer-scout/` 记录（transpiler-errors.md M12 + cmd-output）② git HEAD + 工作区 diff（build/density.rs + terrain.rs + worldgen_handle.rs + 生成代码）③ 原始代码（build/density.rs + src/density.rs apply_unary/WeirdScaled::scale_value + src/terrain.rs + src/worldgen_handle.rs）+ 生成代码（vanilla_density_functions.rs）。
> 结论性质：**只出审查意见，不改任何 status。confirmed 由人类授予。**

---

## 一、逐环核对结论

### 环 1 — 4 个公式修复是否正确（✅ 通过，逐位一致）

**三源核对（transpiler 生成公式 vs 运行时 apply_unary / WeirdScaled::scale_value）：**

| 节点 | transpiler 生成（build/density.rs） | 运行时（src/density.rs） | 一致 |
|---|---|---|---|
| squeeze | L167-172：`let d = clamp(v,-1,1); d/2 - d³/24` | L44：`let d = clamp_d(x,-1,1); d/2 - d*d*d/24` | ✅ 逐位一致 |
| half_negative | L159-162：`if v>0 { v } else { 0.5*v }` | L42：`if x>0 { x } else { 0.5*x }` | ✅ 逐位一致 |
| quarter_negative | L163-166：`if v>0 { v } else { 0.25*v }` | L43：`if x>0 { x } else { 0.25*x }` | ✅ 逐位一致 |
| weird_scaled_sampler | L257-271：`scale_value` 分段阶梯 + `s*\|noise(x/s,y/s,z/s)\|` | L60-71 scale_value + L608-611 `d*noise.sample(x/d,y/d,z/d).abs()` | ✅ 逐位一致 |

- **squeeze 手算验证**：v=-1.875 时，错式 `v-v³/3` = -1.875-(-6.5918)/3 = 0.3223；对式 `clamp/2-d³/24` = -0.5+0.04167 = -0.45833。`transpiler_combine6.txt` 的 `combine.sample_combine = -0.458333` 与对式吻合。✅
- **weird_scaled rarity 映射**：transpiler L263 `rv == Some("type_2")` → Caves 分支（0.5/0.75/1.0/2.0/3.0），else → Tunnels 分支（0.75/1.0/1.5/2.0）。运行时 density_builder.rs L330-331 同映射（`type_2`→Caves，else→Tunnels）。✅
- **git diff 确认**：`f4694f9` 中 build/density.rs 的 4 处公式从错误式改为正确式（`-0.5*x`→条件式、`v-v³/3`→clamp 式、常数乘→分段阶梯）。✅
- **生成代码确认**：当前 `vanilla_density_functions.rs` L171 含 `d / 2.0 - d * d * d / 24.0`（squeeze 正确式）；`grep "unhandled type"` = 0、`grep "unresolved ref"` = 0。✅

**结论：4 个公式修复正确，与运行时逐位一致。通过。**

### 环 2 — final_density 0.000000 是否可信（⚠️ 部分通过，证据缺口 + 覆盖局限）

- **n=54 max_diff=0.000000 已落盘**：`transpiler_finaldensity_after_unaryfix.txt` = `compute_final_density(每点精确 channel) vs 运行时 final_density: max_diff=0.000000 at (-4604, 64, -4084) (n=54)`。✅ 真实记录。
- **⚠️ 该测试是「每点精确 channel」**（`transpiler_alignment.rs` L52-64）：对 54 点（6 y × 3 z × 3 x，x/z∈{4,8,12}）用 `fill_cell_corner_densities` 在**精确点**采样 channel，再 `compute_final_density` 对比运行时。**未覆盖 cell-grid 插值路径**（生产 TranspilerDensity 用的是 cell corners 采样 + 块级三线性插值）。
- **⚠️ 98304 点 max_diff=0.000000 无落盘记录**：`transpiler_prod_density.rs`（对比 td_slices vs ms_slices 全 chunk 98304 点）的输出**未写入 cmd-output**。该 claim 出现在 commit message、transpiler-errors.md、index.yaml，但**无 cmd-output 证据文件**。
- **⚠️ 与 ch0b 测试潜在矛盾**：`transpiler_ch0b_after_unaryfix.txt` 显示 corner (-4608,0,-4096) 处 `transpiler out[0]=0.055648` vs `runtime channels[0].sample=0.208410`，**diff=0.152762**（非 0）。该 corner 是 cell corner（ix=0）。n=54 测试点（ix=1,2,3）未含此 corner，故 n=54 显示 0.000000 但 ch0b 显示 0.15——**channel 采样在部分 cell corner 仍有残差**，98304 点全对齐 0.000000 存疑。

**结论：n=54 的 0.000000 真实可信（combine + 精确点 channel 采样），但「全 chunk 98304 点 max_diff=0.000000」无落盘证据，且与 ch0b 的 0.15 残差潜在矛盾。部分通过。**

### 环 3 — 接入生产是否正确（✅ 通过，语义一致 + 门控零风险）

- **TranspilerDensity 结构**（terrain.rs L271-361）：cell grid（cell_w=4, cell_h=8）corners 采样 5 channels（`fill_cell_corner_densities_final_density`）+ 块级三线性插值 + `compute_final_density`。与 DensityMacroSampler（terrain.rs L18-77）**结构逐行一致**（同 cell 几何、同 trilerp、同 clamp）。✅
- **语义等价**：DensityMacroSampler 用 `macrolize_channels`（5 channels）+ `sample_combine`；TranspilerDensity 用 transpiled `fill_cell_corner_densities`（5 channels）+ `compute_final_density`。两者 channel 数一致（5），combine 语义对齐。✅
- **WG_TRANSPILER 门控零风险**（worldgen_handle.rs L145-156, L344-349）：env 未设时 `transpiler_density = None`，`fill_chunk` 走 else 分支用 `DensityMacroSampler`——**与改动前行为完全一致**。✅
- **泛型化正确**：`DensitySource<S: ChunkDensitySampler>` + `ChunkDensity<'a, S>` 泛型化，非 transpiler 路径 `D = DensityMacroSampler, S = DensityMacroSampler` 编译通过（工作区 clean，已提交）。✅

**结论：接入生产正确，语义与 DensityMacroSampler 一致，WG_TRANSPILER 门控零风险。通过。**

### 环 4 — 产物契约是否满足（⚠️ 部分通过，docs/07 缺失 + 2 处证据缺口）

- **`.artifacts/index.yaml`**：新增 3 条（formula-bugs / prod-integration / prod-alignment），全部 `status: candidate`。✅
- **错误台账 transpiler-errors.md**：M12a-d 五段式完整记录（现象/根因/定位/修复/教训）+ 端到端性能注记。✅
- **cmd-output**：`transpiler_finaldensity_after_unaryfix.txt`（n=54 0.000000）/ `transpiler_continents_after_unaryfix.txt`（n=54 0.000000）/ `transpiler_prodblocks_after_unaryfix.txt`（99.30%）/ `transpiler_prod_perf.txt`（1.09x）落盘。✅
- **⚠️ docs/07 未更新 M12/生产内容**：`f4694f9` 的 docs/07 diff **只有重排（reformat），无新增 M12/生产小节**。docs/07 末尾（L1020）仍写「transpiler 生产接线未完成（探针层验证，生产 fill 路径未接入优化）」——**已过时**（生产接线已完成）。commit message 声称「docs/07」但实际未记录 M12/生产。
- **⚠️ 98304 点 max_diff=0.000000 无 cmd-output**：见环 2。
- **⚠️ 94.20% vanilla FULL 无任何记录**：该 claim 只在 commit message 和任务描述出现，**仓库内无 cmd-output/docs/index.yaml 记录**。
- **⚠️ 修前 78.48% 无记录**：块级一致「修前 78.48%」无 cmd-output 记录，仅 post-fix 99.30% 有记录。

**结论：index.yaml + 错误台账 + 部分 cmd-output 满足；但 docs/07 未记录 M12/生产、98304 点/94.20%/78.48% 三处证据缺口。部分通过。**

### 环 5 — 置信度标注是否合法（✅ 通过）

- 所有 `.artifacts/index.yaml` 条目、docs/07 小节、错误台账均标 **candidate**（非 confirmed）。✅
- 无任何 AI 自标 confirmed 的违规。✅
- **confirmed 留给人类**：docs/07 明确「confirmed 由人类授予」。✅

### 环 6 — 是否有遗漏（⚠️ 发现 1 处性能记录矛盾 + 1 处语义近似 + 1 处覆盖局限）

| 审查点 | 结论 |
|---|---|
| 生成代码无 unhandled type | ✅ `grep "unhandled type"` = 0；`grep "unresolved ref"` = 0 |
| `cache_all_in_cell` cell 级语义 | ⚠️ transpiler 把 `cache_all_in_cell` 与 `cache_once` 同用 `transpiler_cache_3d`（(x,y,z) key，build/density.rs L240-248）。Java 中 `cache_all_in_cell` 是 cell 级缓存（cell 内共享），transpiler 用点级 (x,y,z) 缓存——**正确性保守（不产生错误值），但缓存效率低于 Java cell 级**。非正确性 bug，但语义未完全对齐 |
| 端到端性能记录矛盾 | ⚠️ **`transpiler_prod_perf.txt` = 1.09x（transpiler 慢 9%）**，但 transpiler-errors.md M12 注记 + commit message 声称「0.96-0.98x（略快 2-4%）」。**记录与声称矛盾**——声称的 0.96-0.98x 无 cmd-output 支撑（52.97/55.37 上下界不在记录中） |
| n=54 覆盖局限 | ⚠️ n=54 测试点全在 cell 边界平面（fx/fz=0），未覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端（与 prior review-transpiler-cache.md 同限定） |

**遗漏结论**：生成代码覆盖完整（无 unhandled/unresolved）；但端到端性能记录与声称矛盾（1.09x vs 0.96-0.98x）、`cache_all_in_cell` cell 级语义未完全对齐、n=54 覆盖局限。

---

## 二、三源核对表

| 核对项 | ① 记录（.investigations/） | ② git HEAD + 工作区 diff | ③ 原始代码/生成代码/运行时 | 一致 |
|---|---|---|---|---|
| squeeze 公式 | transpiler-errors.md M12a | build/density.rs L167-172 改 clamp 式 | 运行时 density.rs L44 `clamp/2-d³/24` | ✅ |
| half_negative 公式 | M12b | build/density.rs L159-162 改条件式 | 运行时 L42 `if x>0{x}else{0.5x}` | ✅ |
| quarter_negative 公式 | M12c | build/density.rs L163-166 改条件式 | 运行时 L43 `if x>0{x}else{0.25x}` | ✅ |
| weird_scaled 公式 | M12d | build/density.rs L257-271 改分段阶梯 | 运行时 L60-71 scale_value + L608-611 | ✅ |
| final_density n=54 0.000000 | `transpiler_finaldensity_after_unaryfix.txt` | 生成代码 L171 squeeze 正确式 | transpiler_alignment.rs L52-64 每点精确 channel | ✅ |
| 98304 点 max_diff=0.000000 | **无 cmd-output** | commit message 声称 | transpiler_prod_density.rs 存在但输出未落盘 | ⚠️ 证据缺口 |
| 块级一致 99.30% | `transpiler_prodblocks_after_unaryfix.txt` | — | transpiler_prod_blocks.rs | ✅ |
| 修前 78.48% | **无记录** | — | — | ⚠️ 证据缺口 |
| vs vanilla FULL 94.20% | **无记录** | commit message 声称 | — | ⚠️ 证据缺口 |
| 端到端性能 | `transpiler_prod_perf.txt` = **1.09x** | commit message 声称 0.96-0.98x | transpiler_prod_perf.rs | ⚠️ 记录矛盾 |
| WG_TRANSPILER 门控 | transpiler-errors.md M12 | worldgen_handle.rs L145-156/L344-349 | env 未设 → None → DensityMacroSampler | ✅ |
| docs/07 M12/生产 | **无** | f4694f9 docs/07 仅重排 | docs/07 L1020 仍写「生产接线未完成」 | ⚠️ 缺失 + 过时 |

**三源核对发现：4 处证据缺口（98304 点 / 78.48% / 94.20% / docs/07 M12）+ 1 处记录矛盾（性能 1.09x vs 0.96-0.98x）。**

---

## 三、审查清单结论（core-judge 8 项）

| # | 清单项 | 结论 |
|---|---|---|
| 1 | 证据完整性（@anchor.test source） | ✅ 探针可复现（seed + 坐标 + cmd-output 落盘）；验证分层 = Partial（探针，非 @anchor.test，docs/07 已声明） |
| 2 | 证据落盘 | ⚠️ n=54/99.30%/perf 已落盘；但 98304 点 / 94.20% / 78.48% 三处无 cmd-output |
| 3 | 三源核对 | ⚠️ 4 处证据缺口 + 1 处记录矛盾（性能 1.09x vs 0.96-0.98x） |
| 4 | 置信度合法 | ✅ 全部 candidate，无 AI 自标 confirmed |
| 5 | 产物契约 | ⚠️ index.yaml + 错误台账满足；docs/07 未记录 M12/生产（仅重排） |
| 6 | 噪声卡历史 | ✅ 目标（transpiler 性能/对齐）无未解决噪声卡记录 |
| 7 | retry cap | ✅ 本次为工程修复（修公式 + 接入生产）+ 重测，不消耗 evidence saturation 计数；无超限未声明 |
| 8 | 模块边界 | ✅ 未引用其他领域模块 skill 正文 |

---

## 四、审查意见汇总（各环节推荐状态）

| 环节 | 推荐状态 | 理由 |
|---|---|---|
| 4 个公式修复（M12a-d） | **建议 candidate** | 与运行时 apply_unary/WeirdScaled::scale_value 逐位一致，手算验证 + git diff + 生成代码三源确认 |
| final_density n=54 0.000000 | **建议 candidate（附覆盖局限）** | 真实落盘（每点精确 channel 测试）；但未覆盖 cell-grid 插值路径，且 ch0b 显示部分 cell corner 有 0.15 残差 |
| 98304 点 max_diff=0.000000 | **保持 draft（证据缺口）** | 无 cmd-output 落盘；与 ch0b 0.15 残差潜在矛盾，需补记录 |
| 接入生产（TranspilerDensity + WG_TRANSPILER） | **建议 candidate** | 语义与 DensityMacroSampler 一致，门控零风险（env 未设行为不变） |
| 产物契约（index.yaml + 错误台账） | **建议 candidate** | index.yaml + 错误台账满足 |
| docs/07 M12/生产小节 | **保持 draft（缺失 + 过时）** | f4694f9 仅重排 docs/07，未记录 M12/生产；L1020 仍写「生产接线未完成」已过时 |
| 端到端性能（0.96-0.98x） | **保持 draft（记录矛盾）** | cmd-output 实测 1.09x（慢 9%），声称 0.96-0.98x（快 2-4%）无记录支撑 |
| vs vanilla FULL 94.20% | **保持 draft（证据缺口）** | 仓库内无任何记录 |

**整体：4 公式修复 + 接入生产方向正确，建议 candidate（confirmed 由人类授予）；但需补 3 处证据（98304 点 / 94.20% / 78.48%）、修正性能记录矛盾（1.09x vs 0.96-0.98x）、补 docs/07 M12/生产小节。**

---

## 五、下一步建议（给主会话/人类）

1. **（必做）补 98304 点 max_diff 落盘**：运行 `transpiler_prod_density.rs` 并落盘 cmd-output。当前「全 chunk 98304 点 max_diff=0.000000」无证据，且 ch0b 显示部分 cell corner 有 0.15 残差——需确认全 chunk 是否真为 0.000000。
2. **（必做）修正性能记录矛盾**：`transpiler_prod_perf.txt` 实测 1.09x（transpiler 慢 9%），但 transpiler-errors.md M12 注记 + commit message 声称 0.96-0.98x（快 2-4%）。需澄清：是多次运行取范围（0.96-0.98x）还是单次 1.09x？若为单次，应修正声称；若为多次，应补落盘多次运行记录。
3. **（必做）补 docs/07 M12/生产小节**：docs/07 未记录 4 公式修复 + 生产接线，且 L1020「生产接线未完成」已过时。需追加 M12/生产小节并更新状态。
4. **（必做）补 94.20% vanilla FULL 记录**：该 claim 无任何落盘。需运行 transpiler vs vanilla FULL 对比并落盘 cmd-output。
5. **（建议）补 78.48% 修前基线**：块级一致「修前 78.48%」无记录，仅 post-fix 99.30% 有记录。
6. **（建议）`cache_all_in_cell` cell 级语义**：transpiler 用点级 (x,y,z) 缓存（正确性保守），Java 是 cell 级缓存——语义未完全对齐，但非正确性 bug。可评估是否需对齐 cell 级缓存效率。
7. **（建议）扩大对齐样本**：覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端，把「transpiler 核心正确」从局部充分提升为更全局证明。
8. **（必做）confirmed 授予**：以上 1-4 为必做项，处理后再授予 confirmed；若人类认可本次 4 公式修复 + 生产接入方向，可先授予 candidate。

> 本意见为建议非命令；用户是最终拍板者。confirmed 由人类授予。
