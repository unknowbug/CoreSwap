# judge 审查意见：build-time transpiler 性能优化结论链审计

> 角色：core.judge（subagent，隔离子进程）。**只出审查意见，不改任何 status。** confirmed 由人类授予。
> 审查对象：本 session「build-time transpiler 性能优化」结论链（把 density 树编译成 native 代码，避免运行时 enum match 解释）。
> 审查基线（三源核对）：① `.investigations/macro-layer-scout/`（buildtime-compile-map.md + transpiler-errors.md + cmd-output/ 各记录）；② git HEAD=`5e01772` + 工作区 diff（工作区 clean）；③ 原始探针代码（`WorldgenRust/src/bin/transpiler_*.rs` + `continents_alignment.rs`）+ 生成代码（`WorldgenRust/src/generated/vanilla_density_functions.rs`）+ `build/density.rs` + `src/noise.rs`。
> 审查标准：core.judge 清单（证据完整性/落盘/三源核对/置信度合法/产物契约/噪声卡/retry cap/模块边界）。
> 日期：2026-08-30。

---

## 0. 结论摘要（一句话）

**结论链的性能定位部分（M1-M6，cell grid 构建慢 5 倍 / 单次 fill 缓存冷 35μs / xz-only 缓存没变）内部自洽、数字可靠；但「transpiler 完整实现 + references 全 resolve」与「continents 对齐 0.0088 证明核心正确」两个环节存在重大缺陷——生成代码含 55 个未 resolve 的 `minecraft:shift_x`/`minecraft:shift_z` 引用（被替换为 0.0），这是 transpiler 的语义 bug，污染了 continents 对齐测试，也使「build-time 编译优势被无缓存抵消」的核心判断建立在有 bug 的 transpiler 上。整体置信度：candidate（性能定位部分），但 transpiler 正确性/价值判断需重大修正，confirmed 未达。**

推荐状态：**性能定位链（M1-M6）建议 candidate；transpiler 完整实现/对齐/核心判断环节建议保持 draft（需先修 shift 引用 bug + 重新对齐验证）。**

---

## 1. 逐环核对

### ① transpiler 完整实现 + CSE — ⚠️ 实现大体完整，但「references 全 resolve / unresolved: 0」是**假**（重大缺陷）

- **22 节点类型**：`build/density.rs` `gen_expr` 的 match arms 覆盖 add/mul/min/max/abs/square/cube/half_negative/quarter_negative/squeeze/clamp/y_clamped_gradient/noise/shifted_noise/shift_a/b/shift/interpolated/blend_density/alpha/offset/flat_cache/cache_2d/cache_once/cache_all_in_cell/range_choice/weird_scaled/spline/old_blended_noise/y。✅ 实现完整。
- **CSE**：`gen_spline_value` 用结构哈希（`format!("{:?}", v)`）去重嵌套 spline → `spline_helper_N`。✅ 有效。**三源核对**：CSE commit `e46c50c` 生成文件 107,592 字符（~107KB），与 `transpiler_alignment_status.txt` 的「15MB → 107KB」一致。✅
- **⚠️ 但「812 spline_helper」是错的**：实际生成文件只有 **53 个 `spline_helper_*` 函数**（CSE commit 与 HEAD 均 53）。「812」可能是 spline 点数或其他指标，非 helper 函数数。**文档数字错误（次要）。**
- **⚠️⚠️ 重大缺陷：「references 全 resolve / unresolved: 0」是假**。三源核对发现生成代码含 **55 个 `0.0 /* unresolved ref minecraft:shift_x */` 和 `0.0 /* unresolved ref minecraft:shift_z */`**（`grep "unresolved ref"` 计数 55）。根因：
  - `build/density.rs` `gen_expr` 的 String 分支（L97-107）把 `minecraft:shift_x` → `trim_start_matches("minecraft:")` → `shift_x`，然后 `registry.get("shift_x")`。但 registry 只从 `density_function/overworld` 目录收集（`collect_json`），**该目录没有 `shift_x.json`/`shift_z.json`**（`Get-ChildItem shift*.json` 为空）→ 返回 `0.0 /* unresolved ref */`。
  - **运行时 `density_builder.rs` L176-185 特殊处理 `minecraft:shift_x`/`minecraft:shift_z`**（构建 `shift_df(ShiftMode::ShiftX/ShiftZ)` 采样 offset noise），而 transpiler **没有这个特殊处理**。
  - 即：`minecraft:shift_x`/`minecraft:shift_z` 是 vanilla 内建 density 函数（不在 overworld density_function 子目录），transpiler 的 registry 无法 resolve，被静默替换为 0.0。**这是 transpiler 的语义 bug**——shifted_noise 的 shift 偏移被置零，噪声在未偏移坐标采样。
- **影响**：`transpiler_complete.txt` 的「unresolved: 0, unhandled: 0（transpiler 完整）」**不成立**（unresolved 实际 55）。「transpiler 完整实现」表述需限定为「22 节点类型 + 嵌套 spline + CSE 完整，但内建 shift 引用未 resolve」。

### ② continents 对齐 0.0088 — ⚠️ **被未 resolve 的 shift 引用污染，不能证明「transpiler 核心正确」**

- **测试设计**：`continents_alignment.rs` 对比 `compute_continents`（transpiler）vs 运行时 `continents` 树，n=54 点（y∈{-64,0,64,128,200,300}，x,z∈{4,8,12}）。
- **⚠️ 污染源**：`continents.json` 是 `flat_cache(shifted_noise(continentalness, shift_x: "minecraft:shift_x", shift_z: "minecraft:shift_z", xz_scale=0.25, y_scale=0))`。transpiler 把 `shift_x`/`shift_z` 替换为 0.0（未 resolve），运行时正确采样 shift offset noise。**所以 0.0088 是「未偏移 vs 已偏移 continentalness 采样」的差异，不是「transpiler 核心 noise/spline 正确」的干净测试。**
- **为何 0.0088 小**：shift offset 通常是小量（~0.1-1 block），在 xz_scale=0.25 下对 continentalness 的扰动小，故 diff 小。**但这不能证明 transpiler 核心正确**——它只说明「shift 置零的误差在当前测试点小」。
- **结论**：② 的「transpiler 核心（noise/spline）正确」**未被该测试建立**。要证明核心正确，需先修 shift 引用 bug（resolve `minecraft:shift_x`/`shift_z`），再重测 continents 对齐。**当前 0.0088 不能作为「核心正确」的证据。**

### ③ final_density 未对齐（0.44）来自 channel inner 采样 — ⚠️ **归因不完整：0.44 是「未 resolve shift」+「channel inner 采样」的组合**

- **channel_debug.txt** 说「每点精确 channel = 竖切插值结果相同 → 差异来自 channel inner 采样」。这只排除了「插值步骤」的差异，**没有排除 channel inner 内部的未 resolve shift 引用**。
- **ch#0（BlendDensity terrain）的 channel inner 就包含未 resolve 的 shift 引用**（生成代码 `fill_cell_corner_densities_final_density` 的 out[0] 里大量 `0.0 /* unresolved ref minecraft:shift_x */`）。所以 0.44 的差异来源 = **未 resolve shift（语义 bug）+ channel inner 采样差异**，不是单一「channel inner 采样」。
- **结论**：③ 的归因**不完整**。0.44 未对齐至少部分来自 transpiler 的 shift 引用 bug，不能全部归因于「channel inner 采样 vs 运行时 Interpolated inner」。**需先修 shift bug 再重新定位 0.44 的剩余来源。**

### ④ 性能定位链（cell grid 慢 5 倍 / 单次 fill 缓存冷 35μs / xz-only 缓存没变）— ✅ 内部自洽、数字可靠，但对比非 apples-to-apples

- **数字核对**（三源一致）：
  - `transpiler_grid_calls.txt`：41.79ms/chunk, corners=1225；`transpiler_grid_compare.txt`：运行时 8.14ms（冷 14.03 - 热 5.88）。✅
  - `transpiler_fill_cost.txt`：单次 6954ns（~7μs）；`transpiler_cache_cold.txt`：缓存冷 35μs vs 热 7μs。✅
  - `transpiler_single_compare.txt`：运行时热采样 171ns。✅
  - `transpiler_xzonly_result.txt`：xz-only 缓存后 41.53ms（没变）。✅
  - 探针代码（`transpiler_grid_calls.rs`/`transpiler_fill_cost.rs`/`transpiler_fill_noise_share.rs`）与记录一致。✅
- **⚠️ 但对比非 apples-to-apples**：transpiler 的 `fill_cell_corner_densities` 采样完整树时，**未 resolve 的 shift 引用 = 0.0，跳过了 shift offset noise 采样**（比真实 shift 采样便宜）。所以 transpiler 的 41.79ms 是**低估**——一个修好 shift bug 的 transpiler 会更慢。**「transpiler 慢 5 倍」结论是保守的（真实差距更大）。**
- **⚠️ xz-only 缓存**：`sample_noise_xz` 是**单槽 thread_local 缓存**（keyed by `(id, x, z)`）。cell grid 迭代序 `for ix { for iz { for iy } }` 下，固定 (ix,iz) 的 iy 循环 (x,z) 恒定 → 缓存命中。但 8 个 xz-only noise 共享单槽，切换 noise 时 miss。**「没变（41.53ms）」与「xz-only noise 非主要成本」一致**（ch#0 的 3D noise + spline 主导）。✅ 合理。
- **结论**：④ 数字可靠、内部自洽，但「transpiler 慢 5 倍」是在有 shift bug（更便宜）的 transpiler 上测的，真实差距更大。**性能定位方向（无缓存是主因）成立，但需在修 shift bug 后复测。**

### ⑤ 核心判断「build-time 编译优势被无缓存抵消」— ⚠️ **前提被 shift bug 削弱，且与「noise 89% 主导」矛盾**

- **前提问题**：该判断建立在「transpiler 编译成 specialized 函数（消除 enum match）本应更快」的假设上。但：
  1. **transpiler 有 shift bug**（未 resolve → 0.0），性能对比是在有 bug 的 transpiler 上做的。
  2. **更根本的**：`tree_vs_noise.txt`（本 session 早前修正里程碑）已证明 **ch#0 corners 采样里 noise 采样占 89%，树遍历仅 11%**。transpiler 的整个价值主张是「消除 enum match 树遍历开销」，但树遍历只占 11%——**即使完美 transpiler（含缓存）也只能省 ~11% 的 ch#0 成本**。这**根本性削弱 transpiler 的价值主张**。
- **⚠️ 与 broader context 矛盾**：`noise_avx_eval.txt`/`real_avx_result.txt` 显示 noise AVX（直接优化 noise 采样）全管线仅 -1% 到 -3.2%，且 aquifer 才是全管线真瓶颈。transpiler 优化的是「树遍历」（11%），比 noise AVX 优化的「noise 采样」（89%）更偏离瓶颈。
- **结论**：⑤ 的「编译优势被无缓存抵消」**方向性成立**（无缓存确实抵消了编译收益），但**前提（编译优势本身）被 shift bug + noise 89% 主导削弱**。**transpiler 的 build-time 编译路线本身可能是在优化错误的瓶颈（树遍历 11%），而非 noise 采样（89%）。**

### ⑥ 下一步方向（深入缓存 vs 暂停 transpiler）— ⚠️ **「深入缓存」为时过早，应先修 shift bug + 重新评估价值**

- **「深入缓存」的问题**：
  1. transpiler 有 **shift 引用 bug**（未 resolve），缓存优化建立在有 bug 的代码上，先修 bug 再谈缓存。
  2. transpiler 的价值主张（消除树遍历）被「noise 89% 主导」削弱——**即使加缓存，transpiler 也只优化 11% 的树遍历**，收益有限。
  3. 更优方向（从 broader context）：noise 采样（89%）才是 ch#0 的真正大头，且全管线 aquifer 才是真瓶颈。
- **「暂停 transpiler」更合理**，但决策依据应修正为：**transpiler 优化的是树遍历（11%），不是 noise 采样（89%）；且 transpiler 有 shift 引用 bug 未对齐。** 在修 shift bug + 重新对齐 + 重新评估「transpiler 是否值得」之前，不应投入缓存优化。
- **建议**：先修 transpiler 的 shift 引用 bug（resolve `minecraft:shift_x`/`shift_z`，对齐运行时 `density_builder.rs` L176-185），重测 continents 对齐（验证核心正确性）与 final_density 对齐（分离 shift bug 与 channel inner 采样差异），再决定「深入缓存 vs 暂停」。**在修 bug 前，缓存优化方向不成立。**

---

## 2. 三源核对表（不一致项汇总）

| 环节 | .investigations 记录 | git HEAD/工作区 | cmd-output/生成代码 | 一致性 |
|---|---|---|---|---|
| ① 22 节点类型 | transpiler_complete "22 全实现" | build/density.rs match arms 覆盖 | 生成代码编译通过 | ✅ |
| ① references 全 resolve | transpiler_complete "unresolved: 0" | build/density.rs 无 shift_x 特殊处理 | **生成代码 55 个 unresolved ref** | ⚠️ **假** |
| ① CSE 107KB | transpiler_alignment_status "15MB→107KB" | e46c50c 生成 107,592 字符 | 107KB ✅ | ✅（但「812 spline_helper」错，实际 53） |
| ② continents 0.0088 | continents_alignment "核心正确" | continents.json 用 shift_x/shift_z | **transpiler 未 resolve shift → 0.0** | ⚠️ 污染 |
| ③ final_density 0.44 | channel_debug "channel inner 采样" | 生成 out[0] 含 unresolved shift | 0.44 含 shift bug 贡献 | ⚠️ 归因不完整 |
| ④ 性能数字 | transpiler_* 记录 | 探针代码一致 | 41.79/8.14/7μs/35μs/41.53ms | ✅（但非 apples-to-apples） |
| ⑤ 编译优势被抵消 | transpiler-errors M5 | — | 前提被 shift bug + noise 89% 削弱 | ⚠️ 需修正 |
| ⑥ 深入缓存 | transpiler-errors 修复方向 | — | 缓存建立在有 bug 代码上 | ⚠️ 为时过早 |

---

## 3. judge 审查清单结论

1. **证据完整性（@anchor.test source）**：本 session 探针（transpiler_*/continents_alignment）均为独立可编译 Rust 探针，非 `@anchor.test` 标注函数；探针代码可复现，属可接受。**未发现伪造证据。**
2. **证据落盘**：原始 cmd-output 落盘于 `.investigations/macro-layer-scout/cmd-output/`（探针级，合规）；**但 transpiler 结论级落盘缺**——`docs/07` 与 `10-timewise` **均无 transpiler 条目**（grep 为空），`.artifacts/index.yaml` **无 transpiler 条目**（只有 macro-layer-scout 的 multi-channel/noise/aquifer 条目，无 build-time transpiler）。**证据链未完整落盘。**
3. **三源核对**：见上表。① references 全 resolve 是**假**（55 unresolved）；② continents 0.0088 被污染；③ 0.44 归因不完整；④ 数字可靠但非 apples-to-apples。
4. **置信度合法**：产物标 draft/candidate 合法，未发现 AI 自标 confirmed。但「transpiler 完整实现」「references 全 resolve」「核心正确」等表述**证据不足以支撑其强度**，属置信度标注偏乐观。
5. **产物契约**：**不满足**——transpiler 结论未进 docs/07 或 10-timewise，`.artifacts/index.yaml` 无 transpiler 条目。
6. **噪声卡历史**：目标（transpiler 性能/对齐）无未解决噪声卡记录（该 session 为性能定位，非运行时失败累积）。
7. **retry cap**：性能定位链（M1-M6）有多次方向修正，但多为「新数据层证据」（新探针/计数）驱动，非「无证据空转」；**未发现连续 3 轮无新证据的违规**。
8. **模块边界**：无跨模块 skill 正文引用违规。

---

## 4. 审查意见汇总

| 环节 | 审查结论 | 推荐状态 |
|---|---|---|
| ① transpiler 完整实现 + CSE | ⚠️ 22 节点 + CSE 完整（107KB 编译通过）；**但「references 全 resolve」假（55 unresolved shift 引用）**；「812 spline_helper」错（实际 53） | 建议 draft（需修 shift bug + 修正文档） |
| ② continents 对齐 0.0088 | ⚠️ **被未 resolve shift 污染，不能证明核心正确** | 建议 draft（修 shift bug 后重测） |
| ③ final_density 0.44 | ⚠️ **归因不完整：0.44 = shift bug + channel inner 采样** | 建议 draft（修 shift bug 后重新定位） |
| ④ 性能定位链 | ✅ 数字可靠、内部自洽；但非 apples-to-apples（shift=0.0 更便宜，真实差距更大） | 建议 candidate（附「修 shift bug 后复测」限定） |
| ⑤ 编译优势被无缓存抵消 | ⚠️ 方向成立，但前提被 shift bug + noise 89% 主导削弱 | 建议 draft（需修正价值判断） |
| ⑥ 深入缓存 vs 暂停 | ⚠️ 「深入缓存」为时过早（先修 shift bug + 重新评估价值）；「暂停」更合理 | 建议 draft（先修 bug 再决策） |

**整体置信度：性能定位链（M1-M6）candidate；transpiler 正确性/价值判断需重大修正，confirmed 未达。**

---

## 5. 下一步建议（给主会话/人类）

1. **修 transpiler shift 引用 bug（必做，最高优先）**：`build/density.rs` 需特殊处理 `minecraft:shift_x`/`minecraft:shift_z`（对齐运行时 `density_builder.rs` L176-185 的 `shift_df(ShiftMode::ShiftX/ShiftZ)`），不能静默替换为 0.0。修后重测 continents 对齐（验证核心正确性）与 final_density 对齐（分离 shift bug 与 channel inner 采样差异）。
2. **修正「references 全 resolve」表述（必做）**：`transpiler_complete.txt` 的「unresolved: 0」是假，实际 55。修正为「22 节点类型 + 嵌套 spline + CSE 完整，但内建 shift 引用未 resolve」。
3. **修正「812 spline_helper」数字（建议）**：实际 53 个 helper 函数。
4. **重新评估 transpiler 价值主张（建议）**：`tree_vs_noise.txt` 已证明 ch#0 里 noise 采样 89%、树遍历 11%——transpiler 优化的是树遍历（11%），不是 noise 采样（89%）。**在修 shift bug + 重新对齐后，用「transpiler 加缓存 vs 直接优化 noise 采样」的对照实验评估 transpiler 是否值得**，而非默认「深入缓存」。
5. **补齐产物契约（必做）**：transpiler 结论 → 派 knowledge subagent 产出 docs/07 或 10-timewise 草稿；补 `.artifacts/index.yaml` 条目（transpiler 完整实现 / CSE / continents 对齐 / final_density 对齐 / 性能定位链 / shift bug）。
6. **性能复测（建议）**：修 shift bug 后复测 cell grid 构建（41.79ms 是低估，真实差距更大），确认「无缓存」主因在正确 transpiler 上仍成立。

---

## 6. 关键发现（本审查最重要的 3 点）

1. **transpiler 有 shift 引用 bug**：生成代码含 55 个 `0.0 /* unresolved ref minecraft:shift_x/shift_z */`，`minecraft:shift_x`/`shift_z` 是 vanilla 内建函数，transpiler 的 registry（只收 overworld density_function 目录）无法 resolve，被静默置零。运行时 `density_builder.rs` 正确特殊处理。**这使「references 全 resolve」「transpiler 完整实现」表述为假，并污染 continents 对齐测试。**
2. **continents 0.0088 不能证明核心正确**：continents 用 shift_x/shift_z，transpiler 置零 → 0.0088 是「未偏移 vs 已偏移」差异，非干净的核心正确性测试。
3. **transpiler 价值主张被「noise 89% 主导」削弱**：`tree_vs_noise.txt` 证明 ch#0 里 noise 采样 89%、树遍历 11%——transpiler 优化的是树遍历（11%），不是 noise 采样（89%）。**「深入缓存」前应先修 shift bug + 重新评估 transpiler 是否值得，而非默认继续。**

> 本意见为建议非命令；用户是最终拍板者。confirmed 由人类授予。
