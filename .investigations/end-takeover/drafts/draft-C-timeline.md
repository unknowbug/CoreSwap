# draft-C — 10-timewise-archive.md 追加草稿（260906-04 end 接管开发块）

> 目标文件：`versions/1.20.1/docs/10-timewise-archive.md`（末尾追加，格式对齐既有条目）。
> 素材：`.investigations/end-takeover/`（00-recon-verdict / 01-rust-status / 02-end-biome-rules / 260906-04-errors / review-002-final）。
> 状态：draft（subagent 草稿，待主会话应用）。

---

## 260906-04（实际 2026-09-06 15:43 起 Get-Date 锚定：end 接管开发块——摸底 → 轻量架构 → G1-G3 开发 → 对拍 → 生产验证）✅ 修复完成 judge 通过（candidate，judge 推荐可 confirmed，待用户拍板）

> 过程产物 `.investigations/end-takeover/`（00 摸底 verdict / 01 rust 现状 / 02 biome 规则 / 03 资源闭包 / 260906-04-errors.md 五段式台账）；通用模式 → workflow-patterns #56-#59、compiler-idioms #18/#19（草稿）。

- ✅ **摸底（scout + worker subagent，Partial 静态审查）**：NEXT_SESSION「create_for_dim 硬编码 overworld」方向描述经廉价验证判定**已过时**（260905-01 拆分后已全参数化，nether 实证跑通）——缺口定性中小，轻量架构 3 要点（Rust 内核 / Java 接管 / 验证闭环）经用户批准。硬缺口 = `minecraft:end_islands` DF 节点（含 SimplexNoiseSampler ~100 行新写）+ EndBiomeSource 位置判定分类器（非 MultiNoise，假设已验证：只读 erosion，阈值三段 + 中心 4096L，02 篇规则提取齐全）。
- ✅ **G1 Rust 内核**：SimplexNoiseSampler（复用 noise.rs GRADIENTS 表）+ EndIslands（CheckedRandom(seed)+skip(17292)，worldSeed 直传无 split）+ density_builder 注册 `minecraft:end_islands`；EndBiomeSource 分类器 + create_for_dim 按 settingsName=end 路由。
- 🔍→✅ **cell 4×8 硬编码发现**：开发/对拍中定位到 end 路径 cell 划分存在 4×8 硬编码，修复（with_cells 共享路径恒等变换，judge PASS-3 静态复核）；同批错误链见台账 E1-E3（catch-all 顺序 / wrapping 溢出 / 空列哨兵 MIN——五段式全记录）。
- ✅ **两轮 A/B 逐位对拍 35520 → 18529 → 0**：首轮全量 A/B 差异块 35520 → cell 修复后 18529 → 对拍比对方法更正（Java `%.17g` vs Rust `{:.17e}` 字符串比对假阴性，数值实际全等，改数值化比较）后 ab-final **diff_chunks=0/36，total_mismatch=0**（palette↔id 逐块全等，Full 判据达成）。
- ✅ **生产验证坑（forceload/DIM1）**：forceload 端到端验收需在 end 维度（DIM1 路径）执行——主世界 forceload 不触达 end chunk（E5：无 populateNoise(end) 日志即此坑暴露面）；实侧维度 chunk 形状 0/256 与 noise.height=128 的域混淆一并修正（zeroShape 改名 + settings id 双闸，workflow-patterns #58）。
- ✅ **judge 终审（review-002-final，MUST 级）**：推荐 confirmed——三源核对 7 项全 ✅（双 seed 逐行全等 / seed 三查 / ab-final 逐位 / 执行体三元组 sha256 独立重算 / git diff 抽查 / 工作区干净）；**哈希更正声明：终版 dll sha256 = 1B5AA1DEA49445A2…（2160640B），早期记录 96AD0411 为 cell 修复前旧 dll，防记录污染**。CONCERN：overworld/nether 既有 A/B 回归建议补跑（非阻塞）；y≥128 上半未入判据（vanilla 全空气，口径已限定声明）；D3 SHOULD judge 被 MUST 覆盖（记台账）。
- 🔍 **open**：confirmed 待用户拍板；共享路径（with_cells/wrapping/空列哨兵）运行时回归补证建议待执行。

---
