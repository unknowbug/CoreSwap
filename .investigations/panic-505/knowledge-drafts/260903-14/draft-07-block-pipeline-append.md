# 草稿：07 篇追加小节（260903-14 panic 修复结论）

> 目标文件：`versions/1.20.1/docs/07-block-pipeline.md`
> 插入位置：文件末尾追加（承接「2026-09-03 est 两处偏差修复…（260903-13…）」小节之后），追加不覆盖。
> 状态：草稿（core.worker 产出，主会话应用）。

---

## 2026-09-03 surface_rules.rs:505 大 region panic 修复：overworld 预加载 noise key 清单缺项（260903-14，confirmed 待用户拍板）

> 承接 260903-12 新课题登记 #2（estopt-sweep 至 ~2304-2560 chunk 处 panic `missing noise sampler`）与本篇 260903-12 节内 ⚠️ 附注。修复 = 清单补一行；judge 意见（待返回后补充：`<!-- judge: placeholder 260903-14 -->`）。过程与错误台账 → `.investigations/panic-505/`（panic-errors.md E1-E3）。

### ✅ 根因（机制）

- overworld 预加载 noise key 静态清单（`worldgen_handle.rs` L272）缺 `minecraft:badlands_pillar_roof`；`place_badlands_pillar`（`surface_rules.rs:1372`）运行时 `get_noise` → `expect` panic。
- 触发面极窄：仅 eroded_badlands biome 列且侵蚀度 e>0 时走到该 rule → 64×64 sweep 至 ~2304-2560 chunk 才首次命中，小样本测试全绿掩盖缺失。

### ✅ 修复

预加载清单补 `minecraft:badlands_pillar_roof` 一行（与运行时查询集合同步）。

### ✅ 验证（Full 层，§9.7 口径见验证记录）

- 4096 chunk sweep 全程无 panic（修复前 64×64 sweep 必崩于 ~2304-2560）。
- 四臂 hash `f2b1a3932c6e589e` 零回归（off/shared × L2 开关）。
- 存档口径 3 采样 {98.9969, 99.0284, 99.0067}%，vs 修复前历史 98.9520%——区间不重叠向上；散布 315 块在非确定带宽内（同族判据见 workflow-patterns #10）。

> 通用模式 → workflow-patterns 发现 #26（预加载/注册表与运行时查询集合同步——expect 型查表缺失在低频分支才触发，大 region sweep 是暴露手段）；build-tooling 发现 #15（run 存档口径照抄历史参数清单，`-PcppWorldgenDir` 必带）。
