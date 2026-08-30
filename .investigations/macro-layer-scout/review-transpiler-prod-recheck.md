# judge 审查意见：build-time transpiler 补证后收尾三源复审（review-transpiler-prod-recheck）

> 审查角色：core.judge + anchor.judge（subagent，隔离执行）。
> 审查对象：2026-08-30 session 对上一轮 judge（`review-transpiler-prod.md`）指出的证据缺口做补证后的**收尾三源复审**。上一轮给 8 个环节的推荐状态中，4 项为 draft（98304 点 / 性能矛盾 / docs/07 / vanilla FULL），需复核是否被 commit `649a2b2` 真正补齐、能否升级 candidate，并检查是否引入新问题。
> 审查基线（三源）：① 产物快照/记录（transpiler-errors.md + cmd-output/ + .artifacts/index.yaml + docs/07）② git（log / `git show 649a2b2` / 工作区 clean，HEAD=`649a2b2`）③ 原始代码（build/density.rs + src/density.rs + src/terrain.rs + src/worldgen_handle.rs + src/bin/* 探针 + generated/vanilla_density_functions.rs）。
> 审查方式：**纯静态文件审查，未运行任何命令/编译**（subagent 无 shell）。验证分层标注以实际执行为准——本次无法复跑探针，只能核对「探针源码是否真做某计算」+「cmd-output 是否真实」。这是复审的固有边界：结论基于探针源码逻辑正确性推断，非运行时重验证。
> 结论性质：**只出审查意见，不改任何 status。confirmed 由人类授予。**

---

## 〇、git 基线

- `git log --oneline -1` = `649a2b2 docs(density): transpiler M12 evidence gaps + perf correction + cache id isolation`
- `git status --short` = 空（**工作区 clean，已提交**）。
- `git show 649a2b2 --stat`：15 files，+388/-63。含：4 个 cmd-output（98304 / vanilla_full / perf_multi / baseline）+ slices_ch0 系列 4 个 + `transpiler_slices_ch0.rs` 新探针 + docs/07 M12/生产小节 + index.yaml 39 行 + transpiler-errors.md 注记修正 + build/density.rs cache id 隔离 + 生成代码重生成。

**结论：git 三源基线成立——审查对象确已提交且 worktree 与 commit 一致，非「快照滞后于工作区」场景。**

---

## 一、逐环节复核结论（上轮 draft 项是否已补齐）

### 环 1 — 98304 点 max_diff=0.000000（上轮 draft → **补齐，建议候选人 upgrade candidate**，附 2 处精度/真实性限定）

**关键澄清（直接回应任务预设）：上一轮 draft 的理由是「无 cmd-output 落盘」。此缺口已被 `649a2b2` 用 `transpiler_prod_density_98304.txt` 补齐，且该文件内容与任务描述的担忧点不同——它不是「只记录分布统计」，探针源码 `transpiler_prod_density.rs` L51-63 实为全量逐点 max_diff 遍历：**

```rust
for y in -64..320 { for z in 0..16 { for x in 0..16 {
    let a = td_slices.sample(&pos); let b = ms_slices.sample(&pos);
    ...
    let d = (a-b).abs(); if d > max_diff { max_diff = d; } n += 1;
}}}
println!("...max_diff={:.6}", max_diff);
```

- **探针确实遍历全部 98304 点**（384 y × 16 x × 16 z）计算逐点 |Δ|，`max_diff` 来自逐点比较，不是分布统计直接相等推导。文件里的「分布（均值/正 block 数）」是附加展示列，`max_diff=0.000000` 是逐点算出的硬结果。**这与上一轮「无落盘」的缺口不同——现在既有落盘、探针源码又在仓库内且可复现。**
- 98304 点取的是**每个 block 位置**（非 cell 边界），比 n=54（全在 cell 边界平面 fx/fz=0）覆盖面更广——**也顺带缓解了上一轮「n=54 覆盖局限」**（cell 内部任意点 + 全高度都被覆盖）。这是补证的实际增量。
- **⚠️ 精度限定（info 级）**：`max_diff={:.6}` 只显示 6 位小数，0.000000 意味着实际 max < 5×10⁻⁷，**不保证逐位 bit 级 0**。且同批 `transpiler_slices_ch0_after_bnfix.txt` 显示 1225 点里 `diff_pts=105/1225`（即 105 点 |Δ|∈(1e-9, 5e-7)），证明残留的是**浮点噪声级差异（~1e-7），非结构性差异**。故「逐位对齐 / 0.000000 = bit 级一致」的措辞**略微过强**——精确表述应是「全 chunk 98304 点 max_diff <5e-7（浮点残差内），无结构性差异」。
- **⚠️ 真实性佐证链（should-fix）**：cmd-output 文件本身只印了 max_diff 值，未印逐点 diff 数组/argmax 坐标，**独立复审者无法仅从 txt 重推导逐点比较**——需依赖探针源码可信。建议在 txt 或探针里补「max_diff 所在坐标 + 最大若干 diff 点列表」，使逐点遍历可复核。

**判定：98304 点证据已补齐且确为逐点遍历，建议 upgrade candidate；精度措辞从「逐位/0.000000」改为「<5e-7 浮点残差内」应记为 debt/下轮修正。**

### 环 2 — ch0b 残差矛盾（上轮潜在矛盾 → **已澄清，建议 upgrade candidate**）

- 上轮疑虑：`transpiler_ch0b_after_unaryfix.txt` 显示 corner(-4608,0,-4096) 处 diff=0.152762，与「全对齐 0.000000」矛盾。
- **澄清链**：`transpiler_ch0b_after_unaryfix.txt` 是**在 M12 四个公式修复之前**（f4694f9 前）拍的快照——那时只有 unary 修复，squeeze/half_negative/quarter_negative/weird_scaled 4 个公式仍错，故 corner 有 0.15 残差是**公式 bug 未修完的中间态**。
- **补齐证据**：新增 `transpiler_slices_ch0.rs` 探针（committed）对 **5×5×49=1225 个 cell corner** 直接对拍 td_slices vs ms_slices——**该网格含 corner(-4608,0,-4096)**（cx=-288, cz=-256, ix=0/iz=0/iy=8 → x=-4608, y=0, z=-4096）。`transpiler_slices_ch0_after_bnfix.txt` 显示**时序 A/B 均 max_diff=0.000000**。即补 `set_blended_noise` + M12 公式修复后，含 ch0b corner 的全部 1225 个 cell corner 都对齐到 <5e-7。
- **根因澄清**（committed docs/07 排除清单已诚实记录）：134/1225 diff 的根因是**探针漏 `set_blended_noise`**（ch0 内 `sample_blended_noise` 返回 0），不是 cache id 污染。cache id 隔离是**防御性**修复。
- **⚠️ diff_pts=105/1225（info）**：同环 1，105 点是浮点噪声级（<5e-7 but >1e-9），非结构差异。**「0.000000」= 6 dp 舍入，非 bit 级 0。**

**判定：ch0b 残差矛盾已澄清——0.15 是预 M12 公式修复的中间态，补证后同 1225 点网格对齐 <5e-7。建议 upgrade candidate。**

### 环 3 — 端到端性能记录一致性（上轮 draft「记录矛盾」→ **补齐，建议 upgrade candidate**，附 provenance 限定）

- 上轮矛盾：`transpiler_prod_perf.txt`=1.09x（单次）vs 声称 0.96-0.98x（快 2-4%）。
- **补齐**：`transpiler_prod_perf_multi.txt` 记录 **5 次运行** 0.96/1.05/0.98/1.01/1.00x，均值 ~1.00x。transpiler-errors.md 端到端性能注记同步修正（初次声称 0.96-0.98x 与 1.09x **均为单次噪声**，5-run 均值持平）。
- **当前仓库内无任何残留 0.96-0.98x / 1.09x 的单次声称**（grep 确认）：old claim 已从 transpiler-errors.md 替换，docs/07 M12 节写 5-run 持平。**三源一致。**
- **⚠️ provenance 限定（should-fix）**：`transpiler_prod_perf.rs` 探针源码每次调用只印**一行** `transpiler/基线: X.XXx`（单次 wall，内部 3 次循环取均值）。因此 `transpiler_prod_perf_multi.txt` 的 5 行（run1-5）**不是单个 committed 探针一次运行产出**，而是**同一二进制重复运行 5 次、手工汇总**的产物。数字与结论自洽（±5% 噪声内持平），也符合「多次运行取范围/均值」的铁律，但**无 committed 的 5 次循环包装脚本**，无法复现「一次跑出 5 行」的原始动作。建议补一个 5-iteration 包装或在该 txt 顶部注明「= transpiler_prod_perf.rs ×5 运行手动汇总」。

**判定：性能记录矛盾已通过 5-run 多次测量修正，且三源一致。建议 upgrade candidate（附多 run 探针 provenance 应补记）。**

### 环 4 — docs/07 M12/生产落盘质量（上轮 draft「缺失+过时」→ **补齐，建议 upgrade candidate**，附 2 处文档卫生问题）

- **补齐**：`649a2b2` 在 docs/07 末尾追加「2026-08-30 transpiler 4 公式修复 + 生产接线（M12）」节（L1025-1065），含 M12 四公式、生产接线实现 + 零风险门控、验证数字、探针教训、已排除假说清单、域/边界。**为 append 不覆盖**（既有 M7-M11 节保留）。
- **L1020 过时状态已修正**：`status：...生产接线已完成（...下节...）`（原「生产接线未完成」→「已完成」）。
- **与新 commit 内容一致性**：docs/07 M12 节引用的所有 cmd-output（finaldensity_after_unaryfix / prod_density_98304 / slices_ch0_after_bnfix / prodblocks_after_unaryfix / prod_vanilla_full / macrosampler_baseline / perf_multi）均存在且内容与本节数字吻合。生成代码 `vanilla_density_functions.rs` L171 含 squeeze 正确式 `d/2.0 - d*d*d/24.0`，`grep "unhandled type"/"unresolved ref"`=0。**三源一致。**
- **⚠️ 文档卫生问题（should-fix）**：
  1. **L1027 仍带 `[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]` 标记**——内容已由主会话应用进 commit 且 docs 已被上一轮 judge 审过，该 DRAFT 标记未移除/未更新为「已应用」。会导致读者误以为本节未定稿。
  2. **同文件内新旧 status 矛盾**：L923（M7 shift-fix 节）、L979（M11 cache-fix 节）仍写「生产接线未完成」，而 L1020 + 新 M12 节写「已完成」。虽是历史快照（append 不覆盖原则允许），但**同一 docs/07 里并存「未完成」与「已完成」**，读者需自行判断哪是当前态。建议旧节加一行「⚠️ 后续已接线生产，见本文 M12 节」，消除歧义。

**判定：docs/07 M12/生产小节已补、L1020 已修正、append 不覆盖、内容与 commit 一致。建议 upgrade candidate（附 2 处 should-fix 文档卫生项）。**

### 环 5 — vanilla FULL 94.20%（上轮 draft「无记录」→ **补齐，建议 upgrade candidate**，附解读限定）

- **补齐**：`transpiler_prod_vanilla_full.txt` = `match=1481583/1572864 (94.20%) nonAir=416210/501102 (83.06%)`；`macrosampler_prod_vanilla_full_baseline.txt` = `95.40% nonAir 86.89%`。两者为同格式（「WorldgenHandle vs vanilla FULL」探针，seed -8248，4×4 origin -288,-256，含 carver+features），transpiler（WG_TRANSPILER）vs 基线（不含）。**落盘齐全，格式与既存 `handle_probe.rs`/`features_probe.rs` 探针输出一致。**
- **⚠️ 解读限定（should-fix/info）**：94.20% < 基线 95.40%（差距 1.2pt），且 nonAir 83.06% < 86.89%。这**与「transpiler 与基线 density 对齐 <5e-7」不矛盾，但说明 transpiler 路径的块级输出并非与基线逐位一致**——环 1 的块级一致为 **99.30%（非 100%）**，即 ~0.7% 块因密度近 0 边界的浮点残差翻转分类，经 carver/features 级联放大成 FULL 层面 ~1.2pt 差距。**这是「transpiler 相对基线略偏 vanilla 更远」的真实观察，不是回退到错误，但应诚实记录**（transpiler 不是「与基线一致且同样好」，而是「与基线 density 对齐到浮点残差、块级多数一致、FULL 略低」）。
- 上一轮「78.48% 修前基线无记录」也已处理：transpiler-errors.md M11 注记（L344）**已显式声明降级**——「修前 78.48% 为会话内实测回显、无 cmd-output … 标注降级（Partial/Degraded）」，仅修后 99.30% 有落盘。**诚实声明满足，缺口按「已声明不可复现」关闭。**

**判定：94.20% 已落盘成对基线。建议 upgrade candidate（附「block 级 99.30%、FULL 略低基线 1.2pt、非逐位一致」的解读限定）。**

---

## 二、三源核对表

| 核对项 | ① 记录（cmd-output/index/errors） | ② git 649a2b2 + 工作区 | ③ 原始代码/生成代码 | 一致 |
|---|---|---|---|---|
| 98304 点 max_diff=0.000000 | `transpiler_prod_density_98304.txt` 已落盘 | 该 txt 在 649a2b2 新增，worktree clean | `transpiler_prod_density.rs` L51-63 逐点遍历算 max_diff | ✅（探针源码证明逐点；⚠️ 仅 6dp，残留 1e-7 噪声） |
| 块级一致 99.30% | `transpiler_prodblocks_after_unaryfix.txt`（f4694f9） | 探针在仓库 | `transpiler_prod_blocks.rs` | ✅（0.7% 残留 = 密度近 0 翻转） |
| ch0b 残差澄清 | `transpiler_slices_ch0_after_bnfix.txt` 时序AB 0.000000 | 该文件 + `transpiler_slices_ch0.rs` 在 649a2b2 新增 | 探针覆盖含 corner(-4608,0,-4096) 的 1225 网格 | ✅（0.15 = 预 M12 修复中间态） |
| 性能 5-run 持平 | `transpiler_prod_perf_multi.txt` 5 行 | 该 txt 在 649a2b2 新增 | `transpiler_prod_perf.rs` 单次一行 | ✅ 数值自洽；⚠️ 多 run 无 committed 包装，provenance 弱 |
| vanilla FULL 94.20% vs 基线 95.40% | 两个 txt 成对落盘 | 均在 649a2b2 新增 | 格式匹配 handle_probe/features_probe | ✅（⚠️ transpiler < 基线 1.2pt，非逐位同优） |
| docs/07 M12/生产 | M12 节 L1025-1065 | 649a2b2 append + L1020 修正 | 内容与 cmd-output/生成代码一致 | ✅（⚠️ L1027 残留 DRAFT 标记；旧节 L923/L979 未完成与 L1020 已完成并存） |
| 4 公式修复（环 base） | transpiler-errors.md M12 | f4694f9（已 prior judge 通过） | build/density.rs L161-173/L259-273 + generated L171 | ✅（本复审复跑核对仍一致） |
| WG_TRANSPILER 门控（环 base） | docs/07 + errors M12 | 已提交 | worldgen_handle.rs L147-158/L344-349 env 未设→None | ✅ |
| cache id 隔离（新增） | docs/07 防御性 + errors | 649a2b2 build/density.rs 0→1M | build L20-23 + density.rs C2D_CACHE L354/L387 支持 resize | ✅ 防御性修复，正确性保守 |

**三源核对结论：上一轮 4 处 draft 证据缺口（98304 / 性能矛盾 / docs/07 / vanilla FULL）全部已在 649a2b2 补齐落盘，且与 git 代码、探针源码三向一致。无「快照滞后于工作区」问题。**

---

## 三、core-judge 8 项清单结论

| # | 清单项 | 结论 |
|---|---|---|
| 1 | **证据完整性（@anchor.test source）** | ✅ 探针可复现（seed=-8248318472910187742 + chunk 坐标 + cmd-output 落盘）；验证分层 = Partial（探针，非 @anchor.test，docs/07 已声明）。本复审为静态核对，未复跑 |
| 2 | **证据落盘** | ✅ 4 处缺口已补齐：98304 / vanillafull+baseline / perf_multi / slices_ch0 系列均落盘。⚠️ perf_multi 无 committed 5-run 包装、98304 txt 未印逐点 diff 数组（should-fix） |
| 3 | **三源核对** | ✅ 8 处核对项全部一致，无快照滞后。上轮 4 处矛盾/缺口已解决 |
| 4 | **置信度合法** | ✅ 全部 candidate，index.yaml 无 AI 自标 confirmed；confirmed 留给人类 |
| 5 | **产物契约** | ✅ index.yaml 5 条新条目（prod-density-98304 / vanilla-full / perf-multi / cacheid-isolation / probe-noiseset-ln）均 candidate 合法。⚠️ 0-byte 占位 `transpiler_slices_ch0.txt` 被提交（should-fix）；docs/07 L1027 残留 DRAFT 标记 |
| 6 | **噪声卡历史** | ✅ transpiler 性能/对齐目标无未解决噪声卡记录 |
| 7 | **retry cap** | ✅ 本 commit 为证据补齐 + 防御性修复 + 文档落盘，属工程修复/补证，不消耗 evidence saturation 计数；无超限未声明 |
| 8 | **模块边界** | ✅ 未引用其他领域模块 skill 正文 |

---

## 四、审查意见汇总（各环节推荐状态）

| 环节 | 上轮状态 | 本轮推荐 | 理由 |
|---|---|---|---|
| 4 公式修复（M12a-d） | 建议 candidate | **建议 candidate**（维持） | f4694f9 已审，三源仍一致，本复审复跑核对无新问题 |
| final_density n=54 0.000000 | 建议 candidate（附局限） | **建议 candidate**（维持） | 98304 已补，覆盖局限缓解；措辞改为「<5e-7」 |
| **98304 点 max_diff=0.000000** | **保持 draft** | **建议 candidate** | 缺口已落盘 + 探针源码证明逐点遍历；附「6dp/浮点残差」精度限定 |
| **接入生产** | 建议 candidate | **建议 candidate**（维持） | 门控零风险，泛型化编译通过（clean submitted） |
| **docs/07 M12/生产小节** | **保持 draft** | **建议 candidate** | M12 节 + L1020 修正 + append 不覆盖；⚠️ L1027 DRAFT 标记、旧节 status 矛盾（should-fix） |
| **端到端性能** | **保持 draft** | **建议 candidate** | 5-run 均值 1.00x 取代单次 1.09x/0.96-0.98x，三源一致；⚠️ provenance 弱（should-fix） |
| **vanilla FULL 94.20%** | **保持 draft** | **建议 candidate** | 成对基线落盘；⚠️ 需附「块级 99.30%、FULL 略低基线 1.2pt、非逐位」解读 |
| 产物契约 / index.yaml | 建议 candidate | **建议 candidate**（维持） | 5 条新条目 candidate 合法 |

**整体结论：上一轮指出的 4 个 draft 证据缺口（98304 / 性能矛盾 / docs/07 M12 / vanilla FULL）已全部被 `649a2b2` 补齐落盘且三源一致，各环节可统一升级到 candidate（confirmed 由人类授予）。无阻断性（blocking）问题。新增的文档卫生/证据展示项为 should-fix/info，不阻塞提交。**

---

## 五、下一步建议（给主会话/人类）

**必做/应做（should-fix，不阻塞 candidate）：**
1. **修正「逐位对齐 / 0.000000 = bit 级一致」措辞**：证据证明是「max_diff <5e-7、残留 1e-7 浮点噪声（slices 105/1225 diff 为证）、块级 99.30%（非 100%）」。三处 docs（transpiler-errors.md / docs/07 / index.yaml 摘要）的「逐位一致/0.000000」应改为「<5e-7 浮点残差内对齐」，避免读者误以为 bit-exact。此点**对判据有实质意义**：0.7% 块翻转 + FULL 低 1.2pt 说明 transpiler 并非「与基线逐位同优」。
2. **移除 docs/07 M12 节 L1027 的 `[DRAFT — knowledge subagent 产出草稿，待主会话应用 + 验证]` 残留标记**，更新为「已应用（649a2b2）」。
3. **消除 docs/07 同文件 status 矛盾**：旧节 L923/L979「生产接线未完成」加一行前向注记 →「后续已接线，见 M12 节」。
4. **补 perf_multi provenance**：`transpiler_prod_perf_multi.txt` 顶部注明「= `transpiler_prod_perf.rs` ×5 次运行手动汇总」，或补一个 5-iteration 包装脚本，使「一次出 5 行」可复现。
5. **补 98304 逐点 diff 可复核性**：探针增加 max_diff 所在坐标 + 最大若干 diff 点输出，使逐点遍历可从 txt 复核（当前只能靠源码推断）。
6. **删除/填充 0-byte 占位 `transpiler_slices_ch0.txt`**（committed 空文件）。

**建议（info，可忽略）：**
7. **cache id 隔离的 C2D_CACHE resize 开销**：transpiler id 从 1M 起，`C2D_CACHE` Vec resize 到 1M+ 槽（~8MB/thread，惰性 Box）。功能正确，但可评估是否值得（防御性已达成），或改用独立 cache 数组降低 footprint。

**确认步骤（人类拍板前）：**
8. 以上 1-6 为建议修正不阻塞；若人类认可本次「4 公式修复 + 接入生产 + 证据补齐」方向，可对 8 环节统一授予 candidate。**confirmed 由人类裁决。**

> 本意见为建议非命令；用户是最终拍板者。confirmed 由人类授予。
> 审查边界声明：本次为静态复审，未复跑任何探针/编译——「98304 逐点」「slices 0.000000」「5-run 性能」等数字的正确性依赖探针源码逻辑被证实，但未做运行时重验证（subagent 无 shell）。如需运行时确证，须由主会话复跑对应探针（见建议 5）。
