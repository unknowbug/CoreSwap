# 修复设计：FlatCache per-chunk 上下文绑定 + Cache2D 16 槽 LRU（Phase 2）

> 依据：root-cause-draft.md（H2 主因 + H3 放大器，用户拍板 + judge 通过）+ Java 参照源码逐行核对
> 目标：消除 rebuild 168× 爆炸 + 缓解多线程 thrashing；**采样值逐位不变（BK-001 零退化铁律）**
> 状态：draft（实施已闭环验证：rebuild 216=6/chunk、蔓延根除、双种子零退化；judge 审查 review-fix-delivery.md 主结论通过）
> **实现演进注记（2026-08-12）**：初版设计为 FlatCacheDF/Cache2DDF 双 16 槽 LRU（见 §3 保留），实测 rebuild 36,252→7,318（5× 降）但**未消除蔓延**（rebuild 203/chunk vs 期望 6，覆盖仍 112 chunk）→ 关键洞察：16 槽 LRU 仍会为邻居 key 构建网格。改用**当前生成 chunk 上下文绑定 + 单槽**（Java per-chunk 实例语义完全对齐，§4），实测 rebuild 216=6.0/chunk（期望值）、覆盖 36（蔓延根除）。Cache2DDF 保留 16 槽 LRU（角点共享列可命中，无蔓延风险）。

## 一、Java 参照语义（ChunkNoiseSampler.java，权威）

### FlatCache（L836-881）
```java
class FlatCache implements Wrapper {
    final double[][] cache;  // [horizontalBiomeEnd+1][+1] = 5×5
    FlatCache(delegate, sample) {
        // 构造时一次性预计算 25 角点（y=0）：startBiomeX+i, startBiomeZ+j → BiomeCoords.toBlock
        for i,j in 0..4: cache[i][j] = delegate.sample(new UnblendedNoisePos(blockX, 0, blockZ));
    }
    sample(pos) {
        k = BiomeCoords.fromBlock(pos.blockX()) - startBiomeX;  // (x>>2) - chunkX*4
        l = BiomeCoords.fromBlock(pos.blockZ()) - startBiomeZ;
        return (k,l) ∈ [0,5) ? cache[k][l] : delegate.sample(pos);  // 越界直算，不重建
    }
}
```
**关键事实**：
1. FlatCache 是 **per-chunk 实例**（ChunkNoiseSampler 每 chunk 一个），网格绑定 chunk 生命周期 → 无跨 chunk 污染
2. **无 rebuild 机制**：构造时建一次，之后永远查表；越界 → delegate.sample 直算（不缓存、不重建）
3. 边界共享：x = chunkX*16+16（下一 chunk 首列）→ k=4 ∈ [0,5) → 命中本 chunk 网格（防嵌套递归）
4. **嵌套 spline 的 locationFunction FlatCache 也是同一 chunk 的实例** → 嵌套采样永远命中自己的网格（或越界直算），**绝不触发邻居网格构建**

### Cache2D（L557-595）
```java
class Cache2D implements Wrapper {
    long lastSamplingColumnPos = MARKER;  // 单槽
    double lastSamplingResult;
    sample(pos) { key = ChunkPos.toLong(x, z); return key==last ? last : {last=key; last=delegate.sample(pos);} }
}
```
- Java Cache2D 也是 per-chunk 实例 + 单槽；**但块级循环被 flat_cache 挡掉**（H1 已证：spline 全来自 buildGrid 角点 y=0），仅 buildGrid 25 角点采样它——25 角点 (x,z) 各不同 → 单槽全 miss 是设计行为，Java 每 chunk 只发生 1 次（FlatCache 构造时 25 角点）

## 二、C++ 现状 vs Java 差异（问题本质）

| 维度 | Java | C++ 现状 |
|---|---|---|
| 实例生命周期 | per-chunk（每 chunk 新建 ChunkNoiseSampler） | 全局单例 DensityFunction 树（wg_create 一次），所有 chunk 共享 |
| 网格构建 | 构造时一次性 25 角点 | lazy：首次 sample 该 chunk 时 buildGrid |
| 缓存容器 | 对象字段（绑定实例） | thread_local 单槽（每线程 1 槽） |
| 越界处理 | delegate.sample 直算 | **rebuild pos 推导的新 chunk 网格** |
| 跨 chunk | 无（每 chunk 独立实例） | 单槽被污染 → 每 chunk 首访 rebuild |

**根因链（H2）**：嵌套 spline 的 FlatCache（continents/erosion/ridges 的 locationFunction）收到 buildGrid 角点 i=4 的邻居坐标 → C++ 越界时 **rebuild 邻居网格**（而非 Java 的直算）→ 邻居 buildGrid 又产出更远角点 → 递归蔓延 112 chunk → rebuild 36,252 = 168×。

## 三、修复方案：per-chunk 多槽缓存（主修复）

### 3.1 核心思路
用 **每实例 thread_local 小容量多槽（CAP=16）LRU** 模拟 Java 的 per-chunk 实例缓存：
- 缓存**最近访问的 16 个 chunk 网格**（FlatCache）/ 16 列（Cache2D），按 chunk 键索引
- sample 命中（key 匹配 或 k/l 界内边界共享）→ 查表；未命中 → 替换 LRU 槽 + buildGrid
- 效果：每个 chunk 网格**只构建一次**（首次访问），后续访问（含嵌套递归回访）全部命中；跨 chunk 迁移（多线程）时线程恢复处理已缓存 chunk 也命中

### 3.2 为什么 16 槽足够
- 单线程顺序生成：当前 chunk C + 嵌套触及的 5×5 邻域（最多 25 chunk）——16 槽覆盖主要局部性，LRU 替换只淘汰最久未用
- 多线程（24 线程 × 16 槽 = 384 chunk 覆盖 >> 112 邻居）→ 跨线程迁移后回访命中 → **同时缓解 H3 thrashing**
- 内存：16 槽 × 25 double ≈ 3.2KB/实例/线程 × 6 实例 × 24 线程 ≈ 460KB（可接受）

### 3.3 具体改动（density.h）

**FlatCacheDF**：
- `Slot` 从单槽 → `std::array<SubSlot, CAP>`（key/cx/cz/grid/stamp）
- `sample(pos)`：
  1. 计算 key=(x>>4, z>>4)；遍历 16 槽找「key 匹配 或 (k,l) ∈ [0,5) 边界共享」→ 命中：stamp=tick++，返回 grid[l*5+k]（k/l 用该槽的 cx/cz）
  2. 未命中 → 找 stamp 最小（最久未用）或空槽 → 替换为 pos 的 chunk + buildGrid → stamp=tick++ → 返回
  3. **保留现有 k=4 边界命中语义**（density.h L700-702）：命中槽后 k/l 界内返回 grid，否则 delegate.sample 直算（Java 越界语义）
- `buildGrid` 不变（25 角点、y=0、锚点不变）
- thread_local 容器不变（每线程独立，无锁）

**Cache2DDF**：
- `Slot` 单槽 → `std::array<SubSlot, CAP>`（key/value/stamp）
- `sample(pos)`：遍历找 key 匹配 → 命中返回；未命中 → 替换 LRU 槽 + arg->sample(pos)
- 注意：key=(x,z) 块级（非 chunk 级）——25 角点各不同列 → 命中率由「同列回访」决定（嵌套递归中同角点坐标重复采样时命中）

### 3.4 不改动的部分
- InterpolatedDF：chunk 级 key 单槽在单线程下正常（interpGrid.fill=238 ≈ 6.6/chunk = 每实例每 chunk 1 次）；多线程 thrashing 由 FlatCache/Cache2D 多槽覆盖后大幅缓解（若验证不达标再评估）
- surface.h 同类 thread_local：成本低（噪声采样），本次不动
- densityBuf 循环顺序：**不改**（H1 非主因，aquifer 同序读取有对齐风险）
- 采样值/插值公式：**完全不动**（纯缓存路径改造）

## 四、验证计划（BK-001 零退化铁律）

1. **对齐回归**：block_probe 8576/3200 SURFACE → TOTAL ≥ 99.9994%/99.9997%（与修复前逐位一致）
2. **性能复测**（08-12 同口径）：
   - 单线程 WG_SPLINEDEBUG：rebuild 从 36,252 → 期望 ~216（6 实例 × 36 chunk）+ 邻居蔓延少量；spline 从 130,420/chunk → 期望回落到 ~2,000-10,000/chunk
   - 单线程 WG_PROFILE：spline 单次 ~1,714ns 保持；wall 从 6,533ms → 期望 < 1s（36 chunk）
   - 多线程 WG_PROFILE：spline 单次 27,155ns → 期望大幅回落（thrashing 缓解）；wall 多线程 < 单线程
   - bench_chunks 吞吐：回 ~30ms/chunk 目标
3. **scan 门禁**：python scripts\scan_cpp_anchors.py → invalid=0

## 五、风险与回退
- LRU 替换策略 bug → 采样值错 → 8576/3200 回归即时暴露，回退 = 恢复单槽实现（git diff 可逆）
- 16 槽不够 → rebuild 仍偏高 → CAP 提到 32 或按「当前 chunk + 5×5 邻域」精确覆盖
- 多线程仍 thrashing（InterpolatedDF 贡献）→ Phase 2 后续加线程亲和（root-cause 方案 2）
