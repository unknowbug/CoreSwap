# 260905-08 fan-out 候选 c-A（.b5 腿 2）——block_at 越界保守拒绝 核实（core.worker）

- 角色：core.worker（fan-out 候选 c-A：block_at 越界返回 -1 vs Java ChunkRegion 邻 chunk 实况）
- 输入：knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md、260905-08-scout-tree-rng.md、-bA-stream-diff.md、-bB-set-diff.md
- 状态：**draft**（Degraded：纯静态源码对拍，沙箱无 shell，未运行任何探针）
- Java 权威源：`.tmp/scout-260905-08/mcsrc/`（yarn 1.20.1+build.10-v2）；Rust：`worldgen-core/src/worldgen_handle.rs`
- 边界：populationSeed 域未触碰；与分支 B 的 Fix-1/2/3 正交（本候选 = .b5 腿 2，独立可叠加）

## 1. Java 侧精确语义核实（一手源，逐点）

### 1.1 ChunkRegion 构造与范围 —— ❗前提修正：读区是 17×17，不是 3×3

- `ChunkRegion.java:87-107`：`chunks` 列表为平方数，`width = floor(sqrt(size))`；`lowerCorner/upperCorner` 取列表首尾。
- `ChunkStatus.java:143-157`：**FEATURES 的 taskMargin = 8** → 传入 `chunks` 为 **17×17 = 289 个 chunk**；`ChunkRegion(world, chunks, targetStatus, 1)` —— **placementRadius = 1**。
- 结论：**读（getBlockState/getTopY/getBiome）可达中心 ±8 chunk；写（setBlockState）被 isValidForSetBlock 限制在中心 ±1 chunk**（:258-287，越界写 `Util.error` + 返回 false 不落块）。
- 树族 feature 实际读写跨度：树冠半径 ≤3（mega_jungle）+ random_offset ≤8（树链不用）→ 树的读/写落点几乎全部在 ±1 chunk 内——**c-A 框架的「3×3」对树族成立，但精确语义是「17×17 读 / ±1 写」**。

### 1.2 getBlockState 对邻 chunk 返回什么

- `ChunkRegion.java:167-169`：`getBlockState(pos) → getChunk(sectionCoordX, sectionCoordZ)`（:123-125 默认 `leastStatus = ChunkStatus.EMPTY`）。
- `ChunkRegion.java:129-140`：`isChunkLoaded`（region 矩形内，:162-164）→ 取缓存 chunk，`chunk.getStatus().isAtLeast(EMPTY)`（ProtoChunk 恒真）→ **返回邻 chunk 的当前实况 BlockState**；region 外 → `create=false` 返回 null → NPE 风险（实际 feature 不会读到那么远）。
- **即：没有「保守拒绝」，region 内的越界读永远返回邻 chunk 实况字节**（terrain/已放 feature 都算）。用户问的 `isChunkUsableAsListener`：1.20.1 yarn 源**无此符号**（新版本概念）；此处对应机制 = `isChunkLoaded + status.isAtLeast(EMPTY)`。
- `getTopY`（:422-423）：`getChunk(...).sampleHeightmap(type, x&15, z&15) + 1` —— 邻 chunk heightmap 实况（ProtoChunk.setBlockState 经 `trackUpdate` 即时维护，bB 文档腿 1 已证）。

### 1.3 邻 chunk 在读的那一刻处于什么状态（时序语义）

- region 内邻 chunk 的**最低保证状态**：调度系统保证 center 跑 FEATURES 时，taskMargin=8 环内邻 chunk 已完成 **CARVERS**（noise+surface+carvers 完毕）。
- 邻 chunk 的 **FEATURES 是否已跑：不保证、取决于调度顺序**（center 的 FEATURES 不依赖邻 chunk 的 FEATURES）。主流生成顺序（由近及远波前）下，**邻 chunk 通常尚未跑 feature** → 越界读通常读到「post-carver 纯地形」；邻 chunk 先完成时读到含其 feature 的实况。
- ⚠️ **IDK-cA1（时序）**：参照 dump（ab-vanilla-region）生成时的实际调度序未验证 → Java 越界读见到的邻 chunk 是否含 feature 无法静态判定。影响：Rust 修复用「邻 chunk = 地形态」近似时可能残留少量残差（判别实验 E-3 可暴露）。
- **写入持久性**：center 的 feature 写入邻 chunk 的块**无条件持久**（ProtoChunk 落盘）→ dump 中邻 chunk 含这些块。这与读侧时序无关、确定性成立。

## 2. Rust 侧核实（worldgen_handle.rs，Fix-1 临时撤出状态，:876-882 E-B1.4b 实验）

- `:932-939` `block_at_col`：`lx/lz` 出 0..16 或 Y 越界 → **返回 -1**；注释 `:930` 自认「已知语义偏差（judge C-3）」。属实 ✅。
- -1 的消费点：`block_predicate_filter` / `would_survive`（tree checked 链）/ 谓词匹配 —— 非 air/非目标块 → **放置点被整树拒绝**（不是裁剪，是整次 generate 失败或该候选点跳过）。
- 写侧：`octx.col` 是单 chunk `BlockColumn`，**越界块物理上无处可写 → 跨 chunk 树冠被静默裁剪**（邻 chunk 后续独立生成，永远看不到）。
- 与 Java 的双重差：① 读被拒（Java 读地形实况多半通过）→ Rust **少整树**；② 写被裁（Java 持久跨块）→ Rust 邻 chunk **少树冠块**。两个方向都是 rust<java。

## 3. 对实验数据（Fix-1 恶化）的归因判断

- Fix-1-only oak +19,262→+29,708 恶化：**主归因不是 c-A**。c-A 机制只会让新执行的 feature 被拒绝/裁剪（rust 计数不动），解释不了 rust 侧**增量 +10k**。增量的机制 = 新执行的「邻 biome feature」在 Rust Biome 门（对齐采样==anchor，Fix-1-only 时仍是旧门）下**过放**（如 plains feature 在 forest chunk 放行——Java 允许集判定拒绝）→ 这是 .b2b 域，与 bB 文档 §2 预测一致（oak 过放主导）。
- c-A 与 Fix-1 的真实耦合：并集扩大执行集后，**边界 chunk 的放置点更密、更靠边** → c-A 的拒绝/裁剪面同比例放大 → jungle/vine 的「rust 少」被 Fix-1 部分掩盖（Fix-1+2 的 jungle −22,295 只比基线 −20,464 好一点：过放补了正、拒绝裁剪压了负）。
- **量化预测（c-A 修复后，其余不变）**：
  - jungle_leaves：收敛（负残差绝对值下降）——边界 chunk Java 放的树/冠 Rust 补上。
  - vine：同向收敛（vine 载点 = jungle leaves）。
  - oak：**恶化或不动**（Java 边缘树补上 → rust−java 更正）；oak 真正收敛要等 .b2b 过放修掉（Fix-2）之后。**判 c-A 时 oak 不作收敛指标**。
  - 预测量级：中（单边界 chunk 边缘带面积占比 ~45%（d≤2），但整树拒绝只发生在 trunk 落点 + 冠跨界的 chunk；估计占树族残差的 10-30%，静态无法更精）。

## 4. 实现方案评估（Rust 单 chunk 管线）

关键洞察（由 §1.3）：**Java 的时序保证是「邻 chunk = post-carver 地形态」，feature 不递归**。所以不需要 3×3 递归 feature 管线！拆成两个独立、都可在单 chunk 管线内落地的机制：

### c-A-min（读侧，推荐先做）
- `fill_chunk_blocks` 里，features 阶段前为 3×3 邻 chunk 各生成一列**地形列**（noise+surface+carver，无 feature——即现有管线跳过 apply_features），建 48×48 region 列缓存；`block_at_col` 越界改读缓存；`ocean_floor`/`world_surface` 同步为 48×48（placement.rs:301/:330/:357 的邻域 heightmap 直通一并修复）。
- 成本：地形列生成 ×9/（region 扫描时缓存复用可摊回 ×1，见下）。`carver` 阶段是否需要邻 chunk（carver 本身跨 chunk 挖）——现状 Rust carver 已单 chunk 近似，保持，IDK 登记。
- cache 策略：region 级生成器持有 `HashMap<(cx,cz), BlockColumn>`，chunk N 用完的列留作 N+1 的邻列 → 摊销后每 chunk 只多算 ~1 次地形列。**必须做缓存，否则 ×9 地形成本不可接受**。

### c-A-write（写侧，第二步）
- 跨 chunk 写缓冲：`pending_cross: HashMap<(cx,cz), Vec<(lx,wy,lz,BlockId)>>`（`octx.pending_cross` 字段已预留占位，:971 当前 None）；放置出口检测 lx/lz 出界 → 写 pending 而非丢弃；目标 chunk 的 features 阶段开始前（地形列生成后）先 overlay 归属自己的 pending 写入。
- 时序语义对齐：overlay 只模拟「邻 chunk 先于本 chunk 生成时留下的块」（Java 写持久语义）；「本 chunk 先生成的块出现在已生成邻 chunk」在 Rust dump 管线里需要全局重写——**dump 载体是逐 chunk 顺序生成的，建议按 chunk 顺序生成时后写前不可达（放弃），只做 pending overlay，即覆盖 Java 写入的多数场景**；IDK-cA2 登记顺序残余差。
- 工作量：c-A-min ≈ 150-250 行（region 列缓存 + block_at 改路由 + heightmap 扩展）；c-A-write ≈ 80-150 行（pending 结构 + overlay + 出口改写）。合计 1-2 个工作日含 A/B 验证。风险点：BlockColumn 的 carver 后状态快照内存（48×48×384 列 ≈ 1.7MB/chunk，region 缓存 9 列 ~16MB，可接受）。

## 5. 最小判别实验（主会话可执行，无需先实现 c-A）

**E-cA1（边缘带富集，先用现有 dump，最便宜）**：
1. 载体：现有确定性 dump（seed 8576294172403134396）——Java 参照 dump vs Rust **Fix-1+2** dump（恶化归因首选）与基线 dump（复算）。
2. 提取树族块（oak_leaves/log、jungle_leaves/log、vine），逐 chunk 计算残差块的边缘距离 `d = min(lx, 15-lx, lz, 15-lz)`（0..7）。
3. 计算富集因子 `E = ρ(d≤2) / ρ(3≤d≤7)`，ρ = 残差块数 / 该环带柱数（柱数/chunk：d0=60, d1=56, d2=52, d3=48…d7=4；d≤2=168 柱，3..7=88 柱）。
   - **判据：E ≥ 2.0 → c-A 支持实锤；1.3 ≤ E < 2.0 → c-A 次要因素；E < 1.3 → c-A 排除主导地位**。
   - 分层对照：只取「3×3 biome 并集 ≠ 中心 biome」的真边界 chunk 重算 E（c-A 的效应应集中在边界 chunk；全 chunk 混算会稀释）。天然基线：树冠半径本身就造成边缘残差，E 的解读必须与「中心 chunk 同指标」对比——真边界 chunk 的 E 显著高于同质 chunk 的 E 才算 c-A 信号。
4. **E-cA2（写裁剪镜像配对，c-A 写腿的决定性判据）**：对 Rust dump 中「d==0 且缺失（Java 有 rust 无）」的 leaves 簇，在 Java dump 的**相邻 chunk** 共享边界 5 格内找 log/trunk，且 Rust 相邻 chunk 同位置也有该 log（= 树干两侧都生成了，只有跨界冠被裁）。配对命中率 >50% → 写裁剪腿实锤；<20% → 写腿贡献可忽略。
5. **E-cA3（Fix-1 恶化归因反证）**：基线 vs Fix-1-only 两个 Rust dump 互 diff，取「rust 新增块」做 E-cA1 同款环带统计——**预测：新增块富集于 chunk 内部/非边缘 → 证实恶化来自 .b2b 过放而非 c-A**（c-A 若是恶化源，新增块应集中在边缘带，且应是「Java 有 rust 无」的镜像缺失而非 rust 净增）。
6. 全程同一确定性 dump 载体（#52），禁跨 run（#51）。

## 6. 排除判据汇总

| 条件 | 结论 |
|---|---|
| E-cA1：真边界 chunk 的 E < 1.3（且与同质 chunk 无显著差） | c-A 排除主导地位（读腿） |
| E-cA2：镜像配对命中率 < 20% | c-A 写腿排除 |
| 未来 c-A-min 实装后 A/B：树族三签名残差变化 < 5% | c-A 降级为噪声级 |
| E-cA3：Fix-1 新增块富集在边缘带 | ❌ 反证本报告 §3 归因（c-A 反成 Fix-1 恶化源），需重审 |

## 7. 边界声明

- 全文 Degraded（静态）；未运行任何命令/探针；status=draft，candidate 需 E-cA1/E-cA2 数据。
- 未验证项：① IDK-cA1——Java 参照 dump 生成时邻 chunk feature 时序（决定 c-A-min 用「地形态近似」的残留量）；② Rust carver 是否需要 region 化（本方案保持单 chunk 近似）；③ 树族残差中 c-A 份额的静态上界只能给 10-30% 粗估。
- 与分支 B 结论的关系：本报告确认 bB §1.4 腿 2 的「保守拒绝」属实，但补充两点修正：(a) Java 读区实际 17×17（写 ±1），树族域 3×3 近似成立；(b) **bB §1.4 腿 2「跨 chunk 写入，Java 能判定并放置」只对「center 的写进邻 chunk」成立，读到的邻 chunk 多为 pre-feature 地形**——bB 文档该处「邻 chunk 实况」的表述在此精确化。
