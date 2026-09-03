# 草稿：10 时间线追加条（260903-14）

> 目标文件：`versions/1.20.1/docs/10-timewise-archive.md`
> 插入位置：文件末尾追加（260903-13 条之后），格式参照既有条目（标题行 + 引言行 + ✅/🔍 小节 + 📌 记录指引）。
> 状态：草稿（core.worker 产出，主会话应用）。

---

## 260903-14（实际 2026-09-03 深夜；surface_rules.rs:505 大 region panic 修复——预加载 noise key 清单缺项收口）

> 承接 260903-12 新课题登记 #2（sweep 至 ~2304-2560 chunk panic `missing noise sampler`）。过程产物 `.investigations/panic-505/`（错误台账 panic-errors.md E1-E3）；judge 意见（待返回后补登记：`<!-- judge: placeholder 260903-14 -->`）。

### 🔍 现象

- estopt 大 region sweep 在 ~2304-2560 chunk 处 panic：`surface_rules.rs:505 missing noise sampler`（260903-12 sweep 尾部原文在案）；4096 chunk sweep 无法完成。
- 仅 eroded_badlands biome 列且侵蚀度 e>0 触发 → 极低频分支，64×64 sweep 才首次命中。

### ✅ 根因（为什么错）

- overworld 预加载 noise key 静态清单（`worldgen_handle.rs` L272）缺 `minecraft:badlands_pillar_roof`；`place_badlands_pillar`（`surface_rules.rs:1372`）运行时 `get_noise` → `expect` panic。预加载集合与运行时查询集合不同步——新增 expect 型查表调用点未同步预加载来源。

### ✅ 定位（怎么发现的）

- panic 点反查调用链：surface_rules.rs:505 `expect` ← place_badlands_pillar（:1372）get_noise ← 噪声 key 来自预加载清单——清单 grep `badlands_pillar_roof` 缺失即闭合。
- 触发条件（eroded_badlands + e>0）解释「小样本全绿、大 region 必崩」；过程与三错误（E1 worldgen-data marker 路径不一致 / E2 rustc --extern 误指 cdylib / E3 Tee 目标目录后建）→ panic-errors.md 五段式台账。

### ✅ 修复

- 预加载清单补 `minecraft:badlands_pillar_roof` 一行。通用模式 → workflow-patterns 发现 #26。

### ✅ 验证（Full 层）

- 4096 chunk sweep 全程无 panic（修复前 64×64 必崩于 ~2304-2560）。
- 四臂 hash `f2b1a3932c6e589e` 零回归。
- 存档口径 3 采样 {98.9969, 99.0284, 99.0067}% vs 修复前历史 98.9520%：区间不重叠向上，散布 315 块在非确定带宽内（#10 同族判据）。

### 📌 记录指引

- 结论 → 07 篇末尾追加小节（260903-14）。
- 通用模式 → workflow-patterns 发现 #26（预加载/注册表与运行时查询集合同步 + 大 region sweep 暴露低频分支缺失）；build-tooling 发现 #15（run 存档口径照抄历史参数清单，`-PcppWorldgenDir` 必带——E1）。
- 产物：`.investigations/panic-505/`（panic-errors.md + knowledge-drafts/260903-14/）。
- 状态：修复验证完成；confirmed 留用户拍板（judge 意见待补）。
