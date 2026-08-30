# judge 审查意见：build-time transpiler shift 引用 bug 修复 + 对齐验证（收尾三源核对）

> 审查角色：core.judge（subagent，隔离执行）。
> 审查对象：2026-08-30 session 修复 build-time transpiler 的 shift 引用 bug 并重测对齐。
> 审查标准：core-judge 清单（证据完整性/落盘/三源核对/置信度合法/产物契约/噪声卡/retry cap/模块边界）。
> 审查基线：① `.investigations/macro-layer-scout/` 记录 ② git HEAD + 工作区 diff ③ 原始探针 + 生成代码 + 运行时 `density_builder.rs`。
> 结论性质：**只出审查意见，不改任何 status。confirmed 由人类授予。**

---

## 一、逐环核对结论

### 环 1 — shift 修复语义是否正确（✅ 通过）

**三源核对（transpiler 生成代码 vs 运行时 shift_df 语义）：**

| 项 | transpiler 生成代码（build/density.rs L104-108 → 生成代码） | 运行时 `density_builder.rs` L176-189 + `density.rs` shift_df | 一致 |
|---|---|---|---|
| shift_x | `noises.sample_noise("minecraft:offset", x*0.25, 0.0, z*0.25) * 4.0` | `shift_df(ns, ShiftMode::ShiftA)` → `noise.sample(pos.x*0.25, 0.0, pos.z*0.25) * 4.0`（density.rs L525） | ✅ 逐位一致 |
| shift_z | `noises.sample_noise("minecraft:offset", z*0.25, x*0.25, 0.0) * 4.0` | `shift_df(ns, ShiftMode::ShiftB)` → `noise.sample(pos.z*0.25, pos.x*0.25, 0.0) * 4.0`（density.rs L509） | ✅ 逐位一致 |
| noise | `minecraft:offset` | `get_noise_sampler("minecraft:offset")`（density_builder.rs L177/L184） | ✅ 同 noise |

- **noise 采样器等价性**：运行时 `get_noise_sampler("minecraft:offset")` 用 `random_deriver.split_str("minecraft:offset")` 派生种子；探针 NoiseSet 注册（continents_alignment.rs L27-32 / transpiler_alignment.rs L27-32）遍历 `build_noise_params_from_file`，key 带 `minecraft:` 前缀，同样 `split_str(id)` 派生种子 → **同一 noise 采样器、同一种子派生**。transpiler 生成代码 `noises.sample_noise("minecraft:offset", ...)` 查同一注册表（noise.rs L317-324）。✅
- **生成代码实证**：`grep "unresolved ref"` 生成代码 = **0**（原 55）；`grep "unhandled type"` = 0。git diff 确认所有 `0.0 /* unresolved ref minecraft:shift_x/shift_z */` 占位符被替换为正确的 `noises.sample_noise("minecraft:offset", ...)` 采样。✅
- **结论**：shift 修复语义与运行时 ShiftA/ShiftB **逐位一致**，用 `minecraft:offset` noise，种子派生一致。**通过。**

### 环 2 — continents 0.000000 是否可信（✅ 通过，附限定）

- **探针确实对比 transpiler compute_continents vs 运行时 continents 树**：continents_alignment.rs L40-41 `tree.sample(&NoisePos{...})`（运行时树）vs `compute_continents(&noises, &[], ...)`（transpiler 生成）。✅
- **n=54 真实**：6 y × 3 z × 3 x = 54 点（L37-45），`max_diff=0.000000 (n=54)`（cmd-output 记录）。✅
- **continents 是干净测试**：`continents.json` 是 `flat_cache(shifted_noise(continentalness, shift_x, shift_z, xz_scale=0.25, y_scale=0))`——含 shift 引用，但修后 transpiler 正确采样 shift offset，故 0.000000 是「已偏移 vs 已偏移」的干净对齐。✅
- **可信度**：0.000000 全对齐证明 transpiler 核心（noise/spline/shift）与运行时逐位一致。**可信。**
- **限定**：n=54 点集中在 chunk 内部 cell 边界平面（x,z∈{4,8,12}，y∈{-64,0,64,128,200,300}），未覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端 / 跨 cell 插值路径。**局部充分非全局证明**（与 docs/07 既有 multi-channel 验证同限定）。此限定不削弱「shift 修复正确」结论，但「transpiler 核心完全正确」的全局性需更大样本。

### 环 3 — final_density 0.432843 归因是否完整（✅ 通过，附限定）

- **修 shift bug 后 final_density 对齐 0.44 → 0.432843**（n=54，`transpiler_finaldensity_after_shiftfix.txt`）。**无显著变化**（0.44 → 0.432843，仅 -1.6%）。
- **归因判断**：若 shift bug 是 final_density 0.44 的主要来源，修后应显著下降；实测几乎不变 → **shift bug 不是 final_density 差异的主要来源**。剩余差异归因于「channel inner 采样」——transpiler 竖切（`fill_cell_corner_densities` 每点采样完整树）vs 运行时 Interpolated cell grid 插值语义差异。**归因方向合理。**
- **限定**：0.432843 是「每点精确 channel 采样」的对比（transpiler_alignment.rs L57-59 每点调用 `fill_cell_corner_densities_final_density`），**不是** cell grid 插值路径的对比。即该探针测的是「transpiler 竖切每点采样 vs 运行时树直接采样」的差异，**未覆盖**「transpiler cell grid 插值 vs 运行时 Interpolated cell grid 插值」的完整对齐。**「剩余差异来自 channel inner 采样」是合理推断，但未用独立探针逐位分离验证**（如对比单个 channel 的 inner 采样值）。归因**方向可信、深度未达逐位分离**。

### 环 4 — 产物契约是否满足（✅ 通过）

- **`.artifacts/index.yaml`**：新增 7 条 transpiler 条目（shift-ref-bug / shift-fix / continents-alignment / finaldensity-alignment / value-weakened / perf-no-cache / review），全部 `status: candidate`。✅
- **docs/07-block-pipeline.md**：末尾追加「2026-08-30 build-time transpiler 探索」小节（背景/修复/对齐验证/价值评估/性能定位/排除清单/域边界），标注 candidate + DRAFT 待应用。✅
- **错误台账 transpiler-errors.md**：M7/M8 更新（修复已落地 + 重测结果）+ 速查表更新。✅
- **cmd-output 修正**：`transpiler_complete.txt` / `transpiler_alignment_status.txt` 修正假表述（unresolved: 0 假 / 812 spline_helper 假，实际 53）。✅
- **新增验证记录**：`transpiler_continents_after_shiftfix.txt` / `transpiler_finaldensity_after_shiftfix.txt`。✅
- **结论**：产物契约完整满足。

### 环 5 — 置信度标注是否合法（✅ 通过）

- 所有 `.artifacts/index.yaml` 条目、docs/07 小节、错误台账均标 **candidate**（非 confirmed）。✅
- 无任何 AI 自标 confirmed 的违规。✅
- **confirmed 留给人类**：docs/07 明确「confirmed 由人类授予」。✅

### 环 6 — 是否有遗漏（judge 之前建议的 6 项）

| # | judge 建议（review-transpiler-perf.md §5） | 状态 |
|---|---|---|
| 1 | 修 shift 引用 bug（必做） | ✅ **已完成**（build/density.rs L104-108，unresolved=0） |
| 2 | 修正「references 全 resolve」表述（必做） | ✅ **已完成**（transpiler_complete.txt 修正） |
| 3 | 修正「812 spline_helper」数字（建议） | ✅ **已完成**（transpiler_alignment_status.txt 改为 53） |
| 4 | 重新评估 transpiler 价值主张（建议） | ⚠️ **部分完成**：M9 价值削弱已记录 + docs/07 四节已写；但「transpiler 加缓存 vs 直接优化 noise 采样」的**对照实验未做**，需用户拍板 transpiler 方向 |
| 5 | 补齐产物契约（必做） | ✅ **已完成**（index.yaml 7 条 + docs/07 小节） |
| 6 | 性能复测（建议） | ⚠️ **未完成**：修 shift bug 后**未复测 cell grid 构建**（41.79ms 是低估，真实差距更大）——「无缓存」主因在正确 transpiler 上是否仍成立未验证 |

**遗漏结论**：6 项中 4 项完成、2 项部分/未完成（价值对照实验、性能复测）。这两项是「建议」级（非必做），且依赖用户对 transpiler 方向的拍板，**不阻塞本次 shift 修复 + 对齐验证的收尾**，但应在下一步建议中明确。

---

## 二、三源核对表

| 核对项 | ① 记录（.investigations/） | ② git HEAD + 工作区 diff | ③ 原始代码/生成代码/运行时 | 一致 |
|---|---|---|---|---|
| shift 修复代码 | transpiler-errors.md M7「修复已落地」 | build/density.rs +10 行（L104-108） | build/density.rs L104-108 与运行时 density_builder.rs L176-189 语义一致 | ✅ |
| 生成代码 unresolved=0 | transpiler-errors.md M7「grep=0」 | 生成代码 diff：55 占位符 → 正确采样 | `grep "unresolved ref"` = 0；`grep "unhandled type"` = 0 | ✅ |
| continents 0.000000 | transpiler-errors.md M8「重测结果」 | 新增 cmd-output 记录 | continents_alignment.rs L40-41 对比 + cmd-output `max_diff=0.000000 (n=54)` | ✅ |
| final_density 0.432843 | transpiler-errors.md M7「final_density 0.432843」 | 新增 cmd-output 记录 | transpiler_alignment.rs L57-59 + cmd-output `max_diff=0.432843 (n=54)` | ✅ |
| 812→53 修正 | transpiler_alignment_status.txt 修正 | diff 确认 812→53 | 生成代码实际 53 个 spline_helper | ✅ |
| 产物契约 | index.yaml 7 条 + docs/07 小节 | diff 确认新增 | — | ✅ |
| 探针源文件 | — | `git diff` 探针 = 空（未改） | continents_alignment.rs / transpiler_alignment.rs 未变 | ✅ |

**三源一致，无差异源。**

---

## 三、审查清单结论（core-judge 8 项）

| # | 清单项 | 结论 |
|---|---|---|
| 1 | 证据完整性（@anchor.test source） | ✅ 探针可复现（seed + 坐标 + cmd-output 落盘）；验证分层 = Partial（探针，非 @anchor.test，docs/07 已声明） |
| 2 | 证据落盘 | ✅ cmd-output 验证记录 + 错误台账 + docs/07 均有可引用落盘 |
| 3 | 三源核对 | ✅ 一致，无差异源（见上表） |
| 4 | 置信度合法 | ✅ 全部 candidate，无 AI 自标 confirmed |
| 5 | 产物契约 | ✅ index.yaml 7 条 + docs/07 小节 + 错误台账更新 |
| 6 | 噪声卡历史 | ✅ 目标（transpiler 性能/对齐）无未解决噪声卡记录（该 session 为性能定位，非运行时失败累积） |
| 7 | retry cap | ✅ 本次为工程修复（修 shift bug）+ 重测，不消耗 evidence saturation 计数；无超限未声明 |
| 8 | 模块边界 | ✅ 未引用其他领域模块 skill 正文 |

---

## 四、审查意见汇总（各环节推荐状态）

| 环节 | 推荐状态 | 理由 |
|---|---|---|
| shift 引用 bug 修复（build/density.rs） | **建议 candidate** | 语义与运行时 ShiftA/ShiftB 逐位一致，unresolved=0，三源核对通过 |
| continents 对齐 0.000000（transpiler 核心正确） | **建议 candidate** | n=54 全对齐，探针真实对比；附「局部充分非全局」限定 |
| final_density 0.432843 归因（channel inner 采样） | **建议 candidate** | 修 shift bug 后无显著变化 → shift 非主因，归因方向合理；附「未逐位分离验证」限定 |
| 产物契约（index.yaml + docs/07 + 错误台账） | **建议 candidate** | 完整满足 |
| transpiler 价值评估（M9） | **保持 draft** | 对照实验未做，需用户拍板 transpiler 方向 |
| 性能复测（M1-M6 在正确 transpiler 上） | **保持 draft** | 未复测 cell grid 构建，41.79ms 是低估 |

**整体：本次 shift 修复 + 对齐验证收尾建议 candidate（confirmed 由人类授予）。**

---

## 五、下一步建议（给主会话/人类）

1. **（建议）性能复测**：修 shift bug 后复测 cell grid 构建（41.79ms 是低估，真实差距更大），确认「无缓存」主因在正确 transpiler 上仍成立（judge ⑥）。
2. **（建议）价值对照实验**：用「transpiler 加缓存 vs 直接优化 noise 采样」对照实验评估 transpiler 是否值得（judge ④），需用户拍板 transpiler 方向。
3. **（建议）final_density 差异逐位分离**：用独立探针对比单个 channel 的 inner 采样值，逐位验证「剩余差异来自 channel inner 采样」而非其他（如 noise 注册/seed 派生）。
4. **（建议）扩大 continents 对齐样本**：覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端，把「transpiler 核心正确」从局部充分提升为更全局的证明。
5. **（必做）confirmed 授予**：以上 4 项为建议级，不阻塞本次收尾；若人类认可本次 shift 修复 + 对齐验证，可授予 confirmed。

> 本意见为建议非命令；用户是最终拍板者。confirmed 由人类授予。
