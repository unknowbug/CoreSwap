# 10-timewise-archive.md 追加段草稿（2026-08-12 根因定论）

> **目标路径**：`versions/1.20.1/docs/10-timewise-archive.md`
> **应用方式（主会话）**：将下方 `---` 分隔线之后、`## 2026-08-12 ...` 起的内容，**原样追加到该文件末尾**（文档铁律：追加不覆盖）。当前文件末尾为 2026-08-11 条目最后一行「- ✅ **RQ-006（C++ 有损加速...）**：仅评估+用户逐项拍板后实施，不默认开（边界内待议）。」，在它之后追加。
> **状态标注**：✅ 根因定论 / 🔍 修复中（Phase 2）——与 10 篇既有条目风格一致。
> **草稿状态**：draft（主会话应用；根因已过 judge + 用户拍板，修复验证待闭环）。

---

## 2026-08-12：性能回归根因定论（H1/H2/H3 假设验证 + judge 通过 + 用户拍板）（✅ 根因定论 / 🔍 修复中 Phase 2）

> 承接 2026-08-11 条目（性能回归根因 candidate 未结案）。2026-08-12 主会话采集新数据（wgprofile_t1/mt + splinedebug 537MB，36 chunks 6×6，seed 8576294172403134396），H1/H2/H3 假设全部验证，根因定论过 judge 审查并经用户拍板确认。完整分析落盘 `.investigations/perf-rework/root-cause-draft.md`（analysis, candidate）+ `review-rootcause.md`（review, candidate），已登记 `.artifacts/index.yaml`。本条保留验证链与定论过程。

### 数据采集（2026-08-12 主会话，勿重复实验）

- 命令：`block_probe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks`（36 chunks 6×6）；MSVC 强制重编，TOTAL 99.9994% 对齐保持（纯性能问题，无功能退化）。
- 原始输出：`cmd-output/wgprofile_8576_t1.txt`、`wgprofile_8576_mt.txt`、`splinedebug_8576_t1.txt`（537MB）；摘要 `cmd-output/collect-summary.md`。

### ✅ 假设验证（三组独立计数器数字闭环）

- **H1（y 主序 → Cache2DDF 单槽 100% miss）：部分成立（非主因）**。y 主序循环属实（worldgen_api.cpp L669-672 `for by{for bz{for bx}}`）且与 density.h L630 注释「同列连续 384 次采样」矛盾；但 splinedebug 全部 SPLINE/CACHE2D 行 **y=0**（grep `pos=(x,非0,z)` 零匹配）→ spline 只在 buildGrid 角点被采样，块级 densityBuf 98,304 次采样被 InterpolatedDF 插值 + FlatCache 查表挡掉（0 次 spline）→ 对爆炸贡献 ≈ 0。改循环顺序无效且不推荐（aquifer 同序读取有对齐风险）。
- **H2（FlatCacheDF 单槽 + buildGrid 嵌套递归 → 邻居网格重建）：成立（主因）**。density.h L735 `p.x=(chunkX*4+i)*4`，i=4 → `(chunkX+1)*16` = **下一 chunk 首列** → 嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（L687 key=(x>>4,z>>4)）→ 单槽被污染 → 重建邻居网格 → **递归蔓延 112 chunk**（36 生成 + 76 邻居，含左下对角 (44,-28)）。**rebuild 36,252 = 每 chunk ~1007 vs 期望 ~6 → 168× 爆炸** → 直接驱动 spline 调用 **20×**（4,695,145 = 130,420/chunk vs 旧 6,250）。
- **H3（多线程 thread_local thrashing）：成立（放大器）**。单槽 thread_local（density.h L660-663/L718-721）+ 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss。spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×**）；调用量不变（4,703,488 ≈ 4,695,145）；wall mt 8488ms > t1 6533ms（并行反而更慢）。
- **数字闭环**（三组独立计数器互相印证）：CACHE2D miss 351,536 = 14,061 rebuild × 25 角点 ✓；spline 4,695,145 ≈ 2,400,550（SPLINEDEBUG 非 leaf）× 1.96 ✓ ≈ 351,536 miss × 13.36 spline/miss ✓；130,420/chunk = 9,765 miss/chunk × 13.36 ✓；36,252 ÷ 36 = 1,007 ✓。
- **08-11 vs 08-12 数据口径**：08-11（rebuild 438,092 / 单次 20,598ns）为多线程 thrashing 环境粗计数器；08-12（36,252 / 1,714ns）为单线程精确统计。不构成矛盾，放大链实为「rebuild 168× × 13.36 spline/miss」。

### ✅ judge 审查通过（review-rootcause.md）

- **主结论通过**：H2 主因（FlatCacheDF 单槽 + buildGrid 角点 i=4 越界 → 嵌套递归蔓延 112 chunk，rebuild 168×）、H3 放大器（thread_local thrashing 16×）、H1 非主因（y 主序注释矛盾已实证不触发 spline），机制与代码一致，数字闭环可复核，置信度标注合法，修复方向（per-chunk 多槽缓存）不破坏 BK-001（采样值逐位不变）。
- **7 项修正/澄清建议**（已处理或已声明）：① CACHE2D 第 4 个 cacheId 来源（spline locationFunction 可能为 Cache2D，列入 root-cause §6 不确定点）② 08-11 vs 08-12 数据差异（口径说明已补入 07 篇 + 发现 #10 修正）③ collect-summary Java 循环顺序断言修正（root-cause §4.1 独立核对为 y 外层，非 x→z→y）④ index.yaml 登记 root-cause-draft/review-rootcause（本次完成）⑤ retry 记录缺失（H1/H2/H3 单轮验证 + 数字闭环已声明）⑥ 噪声卡历史无法核对（工作区无 noise_cards.json，留档）⑦ wall 时间 6448.0 vs 6533.3 来源注明（取 collect-summary）。

### ✅ 用户拍板 + 修复启动

- ✅ **根因定论（用户拍板确认）**：H2 主因（FlatCacheDF 单槽缓存 + buildGrid 角点越界 → 嵌套 FlatCache 邻居 key 污染 → 递归蔓延，rebuild 168× → spline 20×）+ H3 放大器（thread_local thrashing 单次 ×16）+ H1 非主因（块级不触发 spline）。
- 🔍 **Phase 2 修复中**：**per-chunk 多槽缓存**（主修复，低风险，采样值逐位不变，不破坏 BK-001 对齐；保留 k=4 边界命中语义 density.h L700-702）→ **线程亲和恢复**（后续，消除 thrashing）；改循环顺序不推荐（H1 非主因）。修复验证待闭环（以 08-12 同口径计数器复测 rebuild/spline 回落）。
