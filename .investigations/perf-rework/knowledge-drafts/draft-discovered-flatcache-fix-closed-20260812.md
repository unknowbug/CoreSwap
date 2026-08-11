# discovered/algorithm-fingerprints.md 发现 #10 补丁草稿（2026-08-12 修复闭环补录）

> **目标路径**：`knowledge/discovered/algorithm-fingerprints.md`
> **范围**：发现 #10（当前 L235-275，UTF-8 编码）。
> **应用方式（主会话）**：按下方「修正块」逐项就地修改（每块给出定位 marker → 替换/追加的新文本）。核心指纹结论不变；本次补录 = ① 修复方案更新（H2 指纹的「修复 = per-chunk 多槽缓存或显式传入当前 chunk 键」→「**已实施：当前 chunk 上下文绑定 + 越界直算**」，并记录 16 槽 LRU 弃用教训）② 置信度/证据段补修复后数据。
> **草稿状态**：draft（主会话应用 + 用户已验收；数字与 `.investigations/perf-rework/cmd-output/` 落盘文件核对一致）。

---

## 修正块 1：置信度行补「修复闭环」（L240）

**定位 marker**：
```markdown
**置信度:** confirmed（机制已 WG_PROFILE/WG_SPLINEDEBUG 实测坐实；2026-08-12 根因定论经 judge 通过 + 用户拍板；修复验证待 Phase 2 闭环后补数据）
```

**替换为**：
```markdown
**置信度:** confirmed（机制已 WG_PROFILE/WG_SPLINEDEBUG 实测坐实；2026-08-12 根因定论经 judge 通过 + 用户拍板；2026-08-12 修复闭环验证达标（rebuild 216=6.0/chunk、覆盖 36）+ judge 通过 + 用户验收）
```

## 修正块 2：「H2 指纹信号」段修复方向更新（L260，整段替换）

**定位 marker**：
```markdown
**H2 指纹信号**：缓存 key 由采样坐标派生（chunk 级），而采样点存在**越出当前上下文范围的角点**（buildGrid i=4/j=4）时，单槽缓存必然收到非本 chunk key → miss + 重建 + 递归蔓延。排查特征 = 重建计数的 chunk 覆盖**超出生成范围**（112 = 36+76 邻居）。修复方向 = per-chunk 多槽缓存（采样值逐位不变，不破坏 BK-001 对齐）或显式传入当前 chunk 键；**改循环顺序无效**（块级不触发 spline，H1 非主因）。
```

**替换为**：
```markdown
**H2 指纹信号**：缓存 key 由采样坐标派生（chunk 级），而采样点存在**越出当前上下文范围的角点**（buildGrid i=4/j=4）时，单槽缓存必然收到非本 chunk key → miss + 重建 + 递归蔓延。排查特征 = 重建计数的 chunk 覆盖**超出生成范围**（112 = 36+76 邻居）。**修复（2026-08-12 已实施并闭环）= 当前 chunk 上下文绑定**：thread_local `g_curChunkX/Z` 绑定当前生成 chunk（fillOneChunkCore 入口 RAII 设置、返回恢复 `INT32_MIN`），k/l 相对 startBiomeX 计算，越界 → `delegate.sample(pos)` **直算不重建**——即 Java per-chunk 实例语义（ChunkNoiseSampler.java L836-881：构造时预计算 25 角点、之后纯查表、永不构建邻居网格）的 C++ 模拟。**关键教训：per-chunk 多槽 LRU 不足以根除**（初版 16 槽 LRU 仍为 pos 推导的邻居 key 构建网格，rebuild 仅 36,252→7,318，覆盖仍 112）——必须消除「越界→重建」语义本身；**改循环顺序无效**（块级不触发 spline，H1 非主因）。
```

## 修正块 3：「证据」段追加修复后数据（L264 行后追加）

**定位 marker**（L264 整行）：
```markdown
- WG_PROFILE/WG_SPLINEDEBUG（2026-08-12，单线程 -threads 1 精确统计）：spline 4,695,145 次（130,420/chunk，旧 6,250 → **20×**）；FlatCache rebuild **36,252** 次 / 112 chunk（每 chunk ~1007，期望 ~6 → **168×**）；CACHE2D miss 351,536 次（4 个 cacheId，= 14,061 rebuild × 25 角点 ✓）；spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×** thrashing）；放大链 = rebuild 168× × 13.36 spline/miss ✓
```

**替换为**（保留原行 + 追加修复后两行）：
```markdown
- WG_PROFILE/WG_SPLINEDEBUG（2026-08-12，单线程 -threads 1 精确统计）：spline 4,695,145 次（130,420/chunk，旧 6,250 → **20×**）；FlatCache rebuild **36,252** 次 / 112 chunk（每 chunk ~1007，期望 ~6 → **168×**）；CACHE2D miss 351,536 次（4 个 cacheId，= 14,061 rebuild × 25 角点 ✓）；spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×** thrashing）；放大链 = rebuild 168× × 13.36 spline/miss ✓
- 修复后验证（2026-08-12 终版 ctx，数据 `.investigations/perf-rework/cmd-output/`）：FLATCACHE rebuild **216 = 6.0/chunk**（期望 ~6 完全达标）、覆盖 **36**（蔓延根除）；CACHE2D miss **23,117**（旧 351,536）；SPLINE **3,032/chunk**（SPLINEDEBUG 非 leaf 口径，旧 66,682，回旧基线 6,250 水平；WG_PROFILE 全量 spline.sample **5,906/chunk**）；单线程 wall 6,533→**2,910ms**（2.2×）；bench 单线程 **62.38ms/chunk**（旧 ~181，3×）；8576 **99.9994%** / 3200 **99.9997%** 零退化
- 初版 16 槽 LRU 对照（已弃用）：rebuild 36,252→7,318（203/chunk）、覆盖仍 112（splinedebug_8576_t1_fixed.txt）——多槽只降频率不除「越界→重建」语义
```

## 修正块 4：「如何利用」段越界角点指纹用法更新（L274，整行替换）

**定位 marker**：
```markdown
- **越界角点指纹（H2，2026-08-12 新增）**：缓存 key 由采样坐标派生（如 `(x>>4,z>>4)` chunk 级）且采样点可能越出当前上下文范围（buildGrid 角点 i=4 → 下一 chunk 首列）时，单槽缓存必然被邻居 key 污染 → 递归重建蔓延。排查 = 检查重建计数的 chunk 覆盖是否超出生成范围（本次 112 chunk = 36 生成 + 76 邻居实锤）；修复 = per-chunk 多槽缓存（采样值逐位不变，对齐零退化）或显式传入当前 chunk 键
```

**替换为**：
```markdown
- **越界角点指纹（H2，2026-08-12 定论，修复已闭环）**：缓存 key 由采样坐标派生（如 `(x>>4,z>>4)` chunk 级）且采样点可能越出当前上下文范围（buildGrid 角点 i=4 → 下一 chunk 首列）时，单槽缓存必然被邻居 key 污染 → 递归重建蔓延。排查 = 检查重建计数的 chunk 覆盖是否超出生成范围（本次 112 chunk = 36 生成 + 76 邻居实锤）；**修复 = 当前 chunk 上下文绑定**（thread_local 显式传入当前 chunk 键 + 越界直算不重建，模拟 Java per-chunk 实例语义；实测 rebuild 216=6.0/chunk、覆盖 36）——**多槽 LRU 不够**（本次初版 16 槽 LRU 仍为邻居 key 建网格、覆盖仍 112），必须消除「越界→重建」语义
```
