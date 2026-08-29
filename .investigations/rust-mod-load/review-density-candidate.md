# Review — density 优化方向修正结论（judge 审查意见）

- 审查角色：core.judge（subagent 隔离）
- 审查对象：candidate「density 方向修正」——Interpolated 优化方向放弃 + 重新定位 fill_chunk 内部
- 审查日期：2026-08-29
- **本审查只出意见，不改任何 status；confirmed 由人类授予**

---

## 一、数据源核对

| 数据源 | 内容 | 状态 |
|---|---|---|
| `.investigations/rust-mod-load/cmd-output/density_interp_correction.txt` | 方向修正记录（candidate 载体） | ✅ 已读 |
| `WorldgenRust/src/bin/density_interp_bench.rs` | 原探针 | ✅ 已读 + 已运行复现 |
| `WorldgenRust/src/bin/density_interp_diag.rs` | 本审查新增诊断探针 | ✅ 已建 + 已运行 |
| `WorldgenRust/src/bin/density_interp_single.rs` | 本审查新增单层对照探针 | ✅ 已建 + 已运行 |
| `WorldgenRust/src/density.rs` | Interpolated 实现（L220-314 + 内部缓存） | ✅ 已读 |
| `WorldgenRust/src/terrain.rs` / `worldgen_handle.rs` | fill_chunk / fill_chunk_blocks | ✅ 已读 |
| `WorldgenRust/src/density_builder.rs` | build_node（final_density 含 interpolated）| ✅ 已读 |
| `overworld.json` final_density 定义 | 内含 `minecraft:interpolated` | ✅ 已确认 |
| base_breakdown.txt / pipeline_breakdown.txt | 前次测量记录 | ✅ 已读 + 复现对比 |

**三源核对**：
- git HEAD/工作区 diff：探针 3 个文件（原 + 本审查 2 个诊断）+ correction.txt 均 **untracked（未提交）**——candidate 证据尚未落 git。审查基于工作区当前文件，源码与应用版一致（无 git 滞后问题，但证据未进版本库）。
- 复现：`density_interp_bench` 可运行，复现数值 6.7ms/628ms/130113-294912 **与原记录 6.4/632/130113 一致**。

---

## 二、逐项核对

### ① density_interp_bench 测量方法有效性 —— **有效但测量基准有缺陷**

- 预热（20 轮）+ 3 次均值 + 点数 98304：方法学正确。
- **缺陷**：`raw = build_node(final_density)` 树**内部已含 `minecraft:interpolated` 节点**（overworld.json 确认），探针又在 raw 外层再包 `Interpolated(new(raw,...))` → **双层 Interpolated**。
- 诊断探针证明：内层 Interpolated 的 grid 采样在裸树 = 每 chunk 6029 次；外层再包后膨胀到**每 chunk 175 万次**（112M/64，291×）。机制 = 外层 build_grid 的网格点跨越多个 chunk，反复清空内层 chunk 网格 → 雪崩重建。
- ⇒ **「裸 6.7ms」是内部 Interpolated 缓存命中后的纯逐点采样**；「Intper 628ms」是双层包装的低效，**不代表单层 Interpolated 性能**。

### ② 「裸 6.4ms 可信、与 29.8ms 矛盾合理」 —— **部分成立，但把密度低估了**

- 复现：裸单 chunk 6.7ms（预热后缓存命中）、raw 多 chunk 14ms/chunk（含内部 Interpolated 网格首建）、fill_chunk 纯 density（全 WG_SKIP）31.4ms/chunk。
- base_breakdown 的 29.8ms 与本次复现 31.4ms **吻合**（base 数字可信）。
- **关键偏差**：candidate 把「29.8ms - 6.4ms ≈ 23ms」全归为 fill_chunk 其他开销，但诊断显示 **生产 per-chunk density 真实成本约 14ms（内部 Interpolated 网格首建主导）**，不是 6.4ms。fill_chunk 的「其他」仅约 17ms。
- ⇒ 方向对（base「纯 density」不是纯采样），但 **6.4ms 是缓存命中下界，不是生产密度上界**；真实 density per-chunk ≈14ms，仍是 fill_chunk 最大单项（14/31.4 ≈ 45%）。

### ③ 之前 base「density 29.8ms」测量缺陷 —— **WG_SKIP_AQUIFER 不能分离纯采样（确认）**

- `WG_SKIP_AQUIFER` 仅在 `classify` 里跳过 `aq.apply`（terrain.rs L42），但 fill_chunk 仍执行 98304×`dense.sample` + 98304×`beard.sample` + 256×`biome.biome()`（每列 6 DF）+ classify 调用 + 循环。
- ⇒ **「纯 density 29.8ms」含 biome(256×6)+beard+classify+循环开销，非纯采样**——candidate 此诊断正确。

### ④ 结论逻辑自洽性（6.4 纯采样 + 其他 ≈ 29.8ms） —— **数字不自洽，方向部分对**

- 算术：29.8 - 6.7 = 23.1ms 归「其他」；但 `其他` 实测仅 ~17ms，density 自身贡献 ~14ms（网格首建）。
- ⇒ 6.4ms 非真实生产密度成本，candidate 的「density 不慢」结论**被测量下界误导**。

### ⑤ 遗漏 —— **核心问题：Interpolated 方向被错误否定**

- **决定性新证据**：单层 Interpolated 包装纯 SplineDF（sloped_cheese，无缓存污染）= **加速 70×**（83.74ms → 1.19ms）。
- ⇒ **「Interpolated 慢 100× → 放弃」的 100× 是双层污染假象**，非方向问题。真正该放弃的是「在 final_density 外层再包一层 Interpolated」（本就冗余双层插值），而**单层 Interpolated 对 SplineDF 高度有效**，与 AGENTS.md 铁律（SplineDF 树遍历 = 慢根源，方向 = C2ME/DFC 直排 + 网格缓存）一致。
- 「44% 差异」= 双层二次插值偏差 + 对插值语义的误解：MC 密度本就该插值（final_density 内部即 interpolated），单层插值误差是固有精度产物，非「不可用」。

---

## 三、审查意见

### 置信度评估
- **「base density 29.8ms ≠ 纯采样（含 biome/beard/classify/网格首建）」**：✅ **成立**（WG_SKIP_AQUIFER 分离缺陷确认 + base 数字复现吻合）。建议 candidate。
- **「重新定位 fill_chunk 内部慢点（biome/surface_height/aquifer）」**：方向合理但**证据不足**——尚未做 fill_chunk 内部精确剖面，且 `其他` 仅 ~17ms 而 density 网格首建 ~14ms，**density 仍是 fill_chunk 最大单项**，过早断言「慢点在 biome/surface」缺乏实测。
- **「Interpolated 优化方向错误 → 放弃」**：❌ **站不住**。100× 慢 = 双层污染（诊断证明内层采样膨胀 291×）；单层对照实测加速 70×。据此放弃 Interpolated 是**基于污染测量的错误否定**。

### 更优的下一步（fill_chunk 内部剖面怎么拆）
1. **先修测量基准**：评估 Interpolated 时用**单层**包装（包纯 SplineDF 子树，如 sloped_cheese），不要包已含 interpolated 的 final_density 外层。
2. **fill_chunk 精确定性剖面**：在 fill_chunk 内分段计时——`dense.sample`（含内部网格首建）/ `beard.sample` / `classify`（含或跳 aquifer）/ `biome.biome`（每列）/ 数组初始化+循环。用 WG 门控计时探针（非并发，单线程，规避测量污染铁律）。
3. **验证单层 Interpolated 生产化**：若单层 70× 在 fill_chunk 场景可复现（每 chunk 网格首建 14ms 是否能用跨 chunk 复用降低），则 Interpolated（或 DFC 直排）才是 density 优化的正解，方向与 AGENTS.md 铁律一致，不应放弃。

### 三源核对结论
- data 记录（correction.txt）↔ 探针源码 ↔ 本次复现/新增诊断：全部对齐，但揭示 correction.txt 的核心数字（Interpolated 慢 100×）基于**有缺陷的双层基准**。
- ⚠️ 探针与记录均 **untracked**，建议评审通过后补提交，且修正记录应补记「双层污染」这一关键背景。

### 推荐状态
- 建议：**保持 draft，暂不升 candidate**（因「Interpolated 放弃」这一结论性论断被污染测量推翻——按置信度状态机，含错误支撑的候选不宜升 candidate）。
- 修正方向：撤销「Interpolated 慢 100×」作为放弃理由；保留「base 非纯采样 → fill_chunk 内部需精确定位」；新增「单层 Interpolated 70× 加速是密度优化正解」作为 candidate 修正。
