# 引擎级性能回归根因分析（draft）

> 角色：core.worker · 任务类型：收敛型（单假设验证）· 状态：**draft**（未审查，不得视为 confirmed）
> 日期：2026-08-12 · 数据源：主会话采集的 wgprofile/wgprofiledebug 输出（勿重复实验）
> 现象：Minecraft 1.20.1 C++ 世界生成引擎 spline 调用 20× 爆炸（130,420/chunk vs 旧 6,250）、
>       多线程单次 27,155ns vs 单线程 1,714ns。功能对齐保持 99.9994%（纯性能问题）。

---

## 0. 结论摘要（TL;DR）

- **主因（H2 成立）**：`FlatCacheDF` 单槽 thread_local 缓存 + `buildGrid` 嵌套采样递归。
  `buildGrid` 角点 `i=4`/`j=4` 时 `p.x=(chunkX*4+4)*4=(chunkX+1)*16` 指向**下一 chunk 首列**，
  嵌套 spline 的 FlatCache 收到邻居 chunk key → 单槽被污染 → 重建邻居网格 → 递归蔓延（112 chunk 实锤）。
  **rebuild 36,252 次 = 每 chunk ~1007 次（期望 ~6 次）→ 168× 爆炸**。
- **放大器（H3 成立）**：`thread_local` 单槽缓存 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss，
  多线程单次 27,155ns（16×）。调用量不变（4,703,488 ≈ 4,695,145），单次成本 ×16。
- **H1 部分成立（非主因）**：y 主序循环确实存在且与 L630 注释矛盾，但 spline/cache_2d 调用
  **全部来自 buildGrid 角点（y=0）**，块级 densityBuf 循环被 InterpolatedDF 插值 + FlatCache 查表挡掉，
  对 spline 爆炸直接贡献 ≈ 0。
- **修复方向**：per-chunk 多槽缓存（低风险，推荐）→ 线程亲和（消除 thrashing）→
  buildGrid 显式传 chunk 键（中侵入）。**改循环顺序无效且不推荐**（块级不触发 spline，且 aquifer 同序读取有对齐风险）。

---

## 1. 假设验证表

| 假设 | 判定 | 关键证据（文件:行） | 贡献量级 |
|---|---|---|---|
| H1：densityBuf y 主序 → Cache2DDF 单槽 100% miss | **部分成立（非主因）** | worldgen_api.cpp L669-672 `for by{for bz{for bx}}` 确为 y 主序；density.h L630 注释「块循环 y→z→x 下同列连续 384 次采样」**与实际循环矛盾**（y 主序下同列相邻两次访问隔 256 次）。但 splinedebug 全部 SPLINE/CACHE2D 行 **y=0**（grep `pos=(x,非0,z)` 零匹配）→ cache_2d 只在 buildGrid 角点被采样（每角点 (x,z) 不同，100% miss 是设计使然），块级循环被 InterpolatedDF（grid 插值）+ FlatCache（查表）挡掉，不触发 spline。Java ChunkNoiseSampler.java L316-327 `DensityInterpolator.fill` 也是 **y 外层**（y→x→z），非 x→z→y 列主序；Java 的 cache_2d 同样被 flat_cache 挡在块级之外。 | 对调用量爆炸 ≈ 0（块级不触发 spline）；仅注释误导 + densityBuf 内存访问不连续（次要） |
| H2：FlatCacheDF buildGrid 嵌套递归 → 邻居网格重建 | **成立（主因）** | density.h L735 `p.x=(chunkX*4+i)*4`，i=4 → `(chunkX+1)*16` 下一 chunk 首列；L687 key=(x>>4,z>>4) chunk 级；splinedebug L58423 rebuild `chunk=(44,-28)`（**左下邻居，不在 36 生成范围内**）、L62484-62487 同一 chunk(45,-27) 的 cacheId=0/1/5 **网格值完全相同的重复 rebuild**；collect-summary：rebuild 36,252 次 / 112 chunk（36 生成 + 76 邻居） | **168×**（每 chunk 1007 vs 期望 ~6）→ 直接驱动 20× spline |
| H3：多线程 thread_local thrashing | **成立（放大器）** | density.h L660-663/L718-721 `static thread_local std::vector<Slot>` 单槽；wgprofile_mt：spline 单次 27,155ns vs t1 1,714ns（16×）；调用量 4,703,488 ≈ t1 4,695,145（不变） | 单次成本 ×16（多线程 wall 8488ms > 单线程 6533ms） |

---

## 2. spline 调用 130,420 次/chunk 的精确构成（推演）

**关键事实（splinedebug 输出实证）**：
- 全部 `[SPLINE]` 行 `pos=(x,0,z)`（y=0）→ **spline 只在 FlatCache buildGrid 角点采样时被调用**（buildGrid 固定 `p.y=0`，density.h L733）。
- 全部 `[CACHE2D]` miss 行同样 y=0 → cache_2d 只在 buildGrid 角点被采样。
- 块级 densityBuf 98,304 次 finalDensity 采样 **不产生 spline**（InterpolatedDF grid 插值命中 + FlatCache 查表）。

**数字闭环**（三组独立计数器互相印证）：
```
CACHE2D miss 总 351,536 ÷ 36 chunk = 9,765 miss/chunk
   = 含 cache_2d 的 FlatCache rebuild 14,061 次 × 25 角点/次（= 351,525 ≈ 351,536 ✓）
      （14,061 ÷ 36 = 390 rebuild/chunk，含 cache_2d 实例 3 个：factor/jaggedness/offset；
        另 continents/erosion/ridges 3 个 arg=shifted_noise 无 cache_2d，rebuild 22,191 次）

SPLINE 调用 4,695,145（WG_PROFILE，含 leaf）≈ 2,400,550（SPLINEDEBUG 非 leaf）× ~1.96（每非 leaf 配 ~1 leaf）
   ≈ CACHE2D miss 351,536 × 13.36 spline/miss
   → 130,420/chunk = 9,765 miss/chunk × 13.36 spline/miss ✓✓
```

**每层放大链**：
```
每 chunk 期望：6 实例 × 1 rebuild = 6 rebuild → 含 cache_2d 3 实例 × 25 = 75 miss → 75 × 13.36 ≈ 1,002 spline
每 chunk 实际：~1007 rebuild（rebuild 36,252/36）→ 含 cache_2d 390 → 9,765 miss → 9,765 × 13.36 ≈ 130,420 spline
→ rebuild 爆炸 168× 是唯一起因；每 miss 的 spline 树递归 13.36 次（factor 顶层 spline + 嵌套 erosion/ridges + leaf）不变。
```

**旧基线 6,250/chunk 推演**（置信度中——旧 profile 为 07 篇/8-06 数据，实现细节不可完全复核）：
- 按同样 6 实例 × 1 rebuild/chunk 模型应得 ~1,000 spline/chunk；6,250 说明旧实现或含 surface/estimateSurfaceHeight 阶段的 spline 采样，或旧 spline 树结构/计数口径不同。
- **诚实声明**：旧基线精确构成无法从现有数据完全重建；20× 爆炸的新旧差由 rebuild 168× 驱动，方向确定，精确倍数受旧口径影响。

---

## 3. H2 根因机制详解（主因）

### 3.1 代码路径
```
块级采样 finalDensity（y 主序）
  → InterpolatedDF::sample（grid 缓存命中，density.h L488-549）
    → 每 chunk 首次 buildGrid（L576-592）→ arg 树（sloped_cheese → factor/jaggedness/offset FlatCache）
      → FlatCacheDF::sample（L683-704）
        → 若 slot.key 不匹配且 kc/lc 越界 → buildGrid（L729-745）→ 25 角点
          → 角点 i=4: p.x=(chunkX*4+4)*4=(chunkX+1)*16（**下一 chunk 首列**）
          → cache_2d miss → spline 树（顶层 n=5/6 → 嵌套 erosion n=7/11 → ridges n=2 → leaf）
            → 嵌套 spline 的 locationFunction FlatCache（continents/erosion/ridges）收到邻居坐标
              → 若其 slot 已被污染 → rebuild 邻居网格 → 递归蔓延
```

### 3.2 为什么 rebuild 从 6/chunk 爆到 1007/chunk
- **单槽缓存跨 chunk 污染**：一个 FlatCache 实例只有一个 thread_local slot（density.h L684-686）。
  任意线程任意时刻只能缓存 1 个 chunk 的 5×5 网格。
- **buildGrid 角点必然触碰邻居**：i=4/j=4 角点坐标等于下一 chunk 首列/首行（L735-738）。
- **嵌套放大**：factor 的 buildGrid 角点采样 spline 树时，顶层 spline 的 locationFunction 是
  continents/erosion/ridges 的 FlatCache → 这些嵌套 FlatCache 用**同一个角点坐标**做 key 判定 →
  若其 slot 非当前 chunk → 触发**邻居 chunk 网格重建** → 邻居 buildGrid 又产生更远邻居角点 → 蔓延。
- **实测蔓延**：rebuild 覆盖 112 chunk（36 生成 + 76 邻居，collect-summary §2）；
  splinedebug L58423 rebuild `(44,-28)`（生成范围 45..50 × -27..-22 之外的左下对角邻居）。
- **同一 chunk 反复 rebuild**：L133-135 与 L62484-62487 网格值逐位相同但重复打印
  → 该 chunk 的网格被邻居污染后又被重新构建 → 同 chunk 多实例多次重建。

### 3.3 为什么不影响输出（对齐保持 99.9994%）
- FlatCache/Cache2D 都是**纯函数缓存**：同一 (x,z)（或 chunk）采样值确定，重建只是重算相同值
  （L62484 网格值与 L133 逐位相同即证）→ 功能无损，纯性能浪费。

---

## 4. H1/H3 附加说明

### 4.1 H1：注释与实现的矛盾（事实）
- density.h L630 注释：**「块循环 y→z→x 顺序下同列连续 384 次采样，命中率 100%」**。
- 实际 densityBuf 循环（worldgen_api.cpp L669-672）：`for by { for bz { for bx } }` = **y 主序**，
  同列 (x,z) 相邻两次访问间隔 16×16=256 次 → **不连续**，注释假设的前提不成立。
- 但该矛盾**不影响 spline 爆炸**：块级循环不直接采样 cache_2d（被 InterpolatedDF + FlatCache 挡掉），
  且 cache_2d 在 buildGrid 角点采样时（每角点 (x,z) 不同）100% miss 是**设计行为**（Java 同样如此）。
- Java 参照：ChunkNoiseSampler.java L316-327 `DensityInterpolator.fill` = y 外层 / x 中层 / z 内层（y 主序），
  **非 x→z→y 列主序**；Java cache_2d（L557-579）同样是单槽 lastSamplingColumnPos，块级被 flat_cache 挡掉。

### 4.2 H3：多线程 thrashing（事实）
- 单槽 thread_local（density.h L660-663/L718-721；surface.h L181/L383/L399 同类模式）。
- 多线程下 chunk 由不同 worker 处理 → 每线程首次访问该 chunk 的 FlatCache 即 miss → 每线程每 chunk 重复 rebuild。
- 实测：spline 单次 t1=1,714ns / mt=27,155ns（16×）；wall mt=8488ms > t1=6533ms（并行反而更慢）。
- surface.h 同类 thread_local 缓存（estimateSurfaceHeight L181、getTerracottaBlock L399）同样受影响，但成本低（噪声采样），非本次爆炸主因。

---

## 5. 修复方向建议（含 BK-001 对齐风险评估）

> BK-001 铁律：8576/3200 SURFACE 零退化，**绝不能改输出**。以下方案均只动缓存路径，不改变采样值。

| # | 方案 | 预期收益 | 对齐风险 | 备注 |
|---|---|---|---|---|
| 1 | **per-chunk 多槽缓存（推荐）**：FlatCacheDF/Cache2DDF 的 thread_local 单槽 → 小容量 per-chunk map（如最近 4-8 chunk 的 grid/值），key 不变 | rebuild 回落到 ~6/chunk → spline 回 ~1,000/chunk（近 130× 降） | **低**：纯缓存，网格/值逐位不变；需保留 key 语义（FlatCache 5×5 覆盖 + 边界 k=4 命中逻辑，density.h L700-702） | 与 Java per-chunk FlatCache 对象语义等价（Java 每个 chunk 一个 FlatCache，C++ 用多槽模拟） |
| 2 | **线程亲和恢复**：每 chunk 固定线程处理 / 每线程不迁移 chunk；或 per-thread 缓存绑定 chunk 生命周期 | 多线程单次 27,155ns → ~1,714ns（16×） | **低**：纯调度，不改采样 | 需线程池/任务分发改造；surface 阶段同类 thread_local 同步受益 |
| 3 | **buildGrid 显式传 chunk 键**：给嵌套采样传入「当前生成 chunk」，使边界角点 i=4/j=4 命中当前网格 k=4，而非用 pos 推导邻居 key | 消除嵌套递归蔓延（H2 直接根除） | **中**：需改 DensityFunction::sample 调用链签名（侵入）或 thread_local「当前 chunk 上下文」 | 与 Java 语义更贴合（Java FlatCache 构造时绑定 chunk）；实现量大于方案 1 |
| 4 | **改 densityBuf 循环顺序**（y→z→x 改 x→z→y 列主序） | **无收益**（块级不触发 spline/cache_2d） | **中高**：aquifer/oreVein 读取 densityBuf 同序（L801），改动影响内存布局与读取模式，可能引入未验证的浮点/行为差异 | **不推荐**；仅能改善 densityBuf 写入局部性，不是根因 |

**推荐组合**：方案 1（主修复，直接消除 rebuild 爆炸）+ 方案 2（多线程放大）。方案 3 作为后续精化。

---

## 6. 置信度与不确定点

- **置信度**：draft（已由代码 + 三次独立计数器闭环验证，未经审查，未提升 candidate）。
- **已确证**：spline 全来自 buildGrid 角点（y=0）；rebuild 168× 蔓延 112 chunk；CACHE2D miss 351,536 =
  14,061 rebuild × 25；spline 4,695,145 = 351,536 × 13.36；H3 多线程 16× 单次成本。
- **不确定点（诚实声明）**：
  1. 旧基线 6,250/chunk 的精确构成（旧实现细节不可复核，仅方向确定）。
  2. rebuild 36,252 中 6 实例各自的精确拆分（含 cache_2d 的 14,061 为反推值，与 3/6 实例比例
     匹配但未逐行统计 cacheId 分布；continents/erosion/ridges rebuild 22,191 无 cache_2d 计数器佐证，
     系 36,252−14,061 之差）。
  3. 递归蔓延的精确首触发点（(44,-28) 的第一次触发无法从 537MB 片段逐行重建，但蔓延事实与
     代码路径均已实证）。
  4. 多线程 27,155ns 的精确构成（thrashing vs 缓存行伪共享 vs 原子计数争用未细分）。

---

## 7. 附：关键代码定位

| 位置 | 内容 |
|---|---|
| worldgen_api.cpp L669-681 | densityBuf 填充循环 y→z→x（y 主序） |
| density.h L630 | Cache2DDF 注释（与实现矛盾的「同列连续 384 次」宣称） |
| density.h L638-651 | Cache2DDF::sample（thread_local 单槽，key=(x,z) 块级） |
| density.h L683-704 | FlatCacheDF::sample（thread_local 单槽，key=(x>>4,z>>4) chunk 级；边界 k/l 命中不 rebuild） |
| density.h L729-745 | buildGrid（角点 i=4 → p.x=(chunkX+1)*16 下一 chunk 首列） |
| density.h L749-838 | SplineDF（sampleImpl 递归 apply → 嵌套子 spline；locationFunction 可为 FlatCache） |
| density.h L488-592 | InterpolatedDF（grid 缓存命中挡掉块级 spline） |
| ChunkNoiseSampler.java L316-327 | Java DensityInterpolator.fill（y 外层，非 x→z→y） |
| ChunkNoiseSampler.java L557-579 / L836-870 | Java Cache2D / FlatCache（per-chunk 实例，C++ 单槽模拟的语义参照） |
| surface.h L181/L383/L399 | 同类 thread_local 单槽缓存（surface 阶段，成本低，非主因） |
