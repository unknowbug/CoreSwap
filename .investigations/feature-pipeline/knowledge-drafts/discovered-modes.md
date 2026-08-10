# knowledge 草稿：FEATURE/随机/工具 通用可复用模式（2026-08-10 FEATURE 实施发现）

> 目标文件：`knowledge/discovered/algorithm-fingerprints.md`（分类「已确认的算法/协议指纹」）
> 编号：接现有最大号 #5 之后 → **#6 ~ #9**
> 状态：draft（未写入 knowledge/，未更新 INDEX —— 待审查后由 core.knowledge 落盘）
> 来源：`.investigations/feature-pipeline/`（phase2/3/3.5/4 cmd-output + pipeline-map.md + block_probe.cpp 证据）

---

## 发现 #6: Java stream.flatMap 惰性 → positions 链必须深度优先（BFS 消费 RNG 错序）

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 3 ore 定位）
**来源定位:** MC 1.20.1 `feature/PlacedFeature.java:44-63`（generate）+ `.investigations/feature-pipeline/pipeline-map.md` §2.2
**置信度:** confirmed（2026-08-10 已验证；-288 FULL 96.67%→96.59%、granite 56.2%→修复后 phase3.5 88.3%）
**module:** re-code

### 观察
Java `PlacedFeature.generate` 用 `Stream.flatMap` 串联 placement modifiers：
```
Stream<BlockPos> stream = Stream.of(pos);
for (PlacementModifier pm : placementModifiers)
    stream = stream.flatMap(p -> pm.getPositions(context, random, p));
```
`flatMap` 是**惰性**的：对当前位置 p 立即完整执行该 modifier 的 getPositions 得到子位置流，再递归展开到叶子，然后才处理下一个兄弟位置 —— 即 **深度优先**。且所有 placement modifier 与 feature 内部共用**同一个 `chunkRandom`（已 setDecoratorSeed）RNG 流**。因此 positions 链的遍历顺序直接决定 RNG 消费顺序 → 决定每个位置。

C++ 初版用「vector 先收集全部位置、再逐层展开」的 **BFS**：同一个 count/in_square/height_range 链，BFS 与 DFS 的 RNG 消费顺序不同 → 位置全部错 → ore 球体位置不重合（granite 匹配率仅 56.2%）。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase3_ore_result.txt`：`root cause = BFS vs DFS positions chain`；-288 FULL 96.67%、300515 96.59%、granite 56.2%
- `pipeline-map.md` §2.2（PlacedFeature.generate flatMap 语义 + 共用 RNG 流）+ §2.4（PlacedFeatureIndexer 决定 setDecoratorSeed 的 index）
- 修复后 phase35_crosschunk_result.txt：granite 88.3%、diorite 85.7%、tuff 87.8%、dirt 92.7%

### 如何利用
- 复刻 positions 链：用**递归/栈式 DFS**（当前位置 → 立即跑完整个 modifier 链到叶子 → 下一个），不要「收集再展开」BFS
- 跨版本通用：1.18/1.19/1.20.x 的 PlacedFeature.generate 都是同一 flatMap 惰性模式；凡「一个 modifier 输出 0..n 个位置、下一个 modifier 再变换」的链都是 DFS
- RNG 必须贯穿 positions 链与 feature.generate 全程同一个流，中途不得重建/重播种

---

## 发现 #7: carver 随机基类是 CheckedRandom（48 位 LCG），不是 Xoroshiro——FEATURES 才是 Xoroshiro

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 2 carvers）
**来源定位:** MC 1.20.1 `util/math/random/CheckedRandom.java` / `ChunkRandom.java setCarverSeed L87-93` / `carver/CaveCarver.java carveTunnels L124-219`；C++ `versions/1.20.1/cpp/worldgen/src/chunkrandom.h`、`carver.h`
**置信度:** confirmed（2026-08-10 已验证；挖洞重合 12%→69%，-288 FULL 93.4462%→93.9442%）
**module:** re-code

### 观察
MC 1.18+ 生成两阶段 RNG 基类**不同**，混用即全错：
- **CARVERS 阶段**：`ChunkRandom(new CheckedRandom(RandomSeed.getSeed()))` —— CheckedRandom 是 **48 位 LCG**：`seed = (seed * 0x5DEECE66D + 0xB) & ((1<<48)-1)`；`next(bits) = seed >>> (48-bits)`（Java `next(int bits)` 语义，高 32 位返回）。`setCarverSeed(worldSeed)` 走 LCG 递推（`nextLong()` = 两次 next(32) 有符号拼接，见 MC-239059）。
- **递归洞穴分支**：`carveTunnels/carveRavine` 内部 `Random.create(seed)` **默认也是 CheckedRandom**（LCG）——不是 Xoroshiro。
- **FEATURES 阶段**：`ChunkRandom(new Xoroshiro128PlusPlusRandom(...))` —— 与 carver 完全不同，C++ `random.h` 已有。

C++ 曾把 carver 递归分支误用 XoroshiroRandom → 漂移序列全错 → 洞穴挖洞位置不重合（仅 12%，2042/16668）；改回 CheckedRandom 后重合 69%（11929/17573）。挖洞位置一旦错位，后续一切依赖洞穴形状的方块（含水层、FEATURE 的 carving_mask、underwater_magma）全部连锁错位。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase2_carvers_result.txt`：根因「carveTunnels/carveRavine 内部 Random.create(seed) = CheckedRandom（48 位 LCG）；C++ 曾误用 XoroshiroRandom → 漂移序列全错 → 挖洞位置不重合（仅 12%）」；修复后 69%；SURFACE 模式零退化 8576 99.9994%/3200 99.9997%
- `chunkrandom.h` 头部注释：CheckedRandom 48 位 LCG 常量（MULTIPLIER=0x5DEECE66D、INCREMENT=0xB、SEED_MASK=(1<<48)-1）；setCarverSeed 语义 + MC-239059 有符号拼接
- `pipeline-map.md` 附录 A：CheckedRandom（LCG）next(bits)=seed>>>(48-bits)；Xoroshiro128PlusPlus 是 FEATURES；`Random.create(seed)` 默认 Xoroshiro **但 carver 隧道分支例外是 CheckedRandom**

### 如何利用
- 复刻 carver 前先确认 RNG 基类：**carver = LCG（CheckedRandom），feature = Xoroshiro128PlusPlus**，两套必须分别实现且绝不混用
- 递归子分支 `Random.create(seed)` 的默认实现会随上下文不同（carver 内是 CheckedRandom）——不要按「默认 = Xoroshiro」一刀切，逐调用点核对
- 挖洞位置错位是「用错 RNG」的强信号：若洞穴与参照不重合且洞穴数量量级匹配（挖洞总量接近但位置不对），先查 RNG 基类，再查 MathHelper.sin/cos 查表（65536 项，非 std::sin）

---

## 发现 #8: 两阶段 FEATURE + pendingCross 跨 chunk 写入——复刻「后写覆盖」语义需区域缓存 + 待应用队列

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 3.5 cross-chunk）
**来源定位:** MC 1.20.1 `ChunkGenerator.generateFeatures`（按 chunk 序遍历）+ `OreFeature.generateVeinPart`（椭球跨 chunk 边界）；C++ `versions/1.20.1/cpp/worldgen/src/block_probe.cpp`（两阶段 `wg_fill_blocks_multi_phase`）
**置信度:** confirmed（2026-08-10 已验证；-288 FULL 96.67%→97.8464%、300515 96.59%→98.0948%）
**module:** re-code

### 观察
Java 世界按 **chunk 序**生成：chunk A 先生成时，跨 chunk feature（如 size 大的球体 ore 椭球、紫晶洞、树冠）的方块会**直接写入相邻 chunk B 的区域**；之后 chunk B 自己生成 feature 时再写一遍 → 语义是 **「后写覆盖」（last-write-wins）**，且 A 的跨 chunk 方块在 B 生成时是可见的（B 的 feature 判定/放置会读到它们）。

C++ `fillOneChunk` 是单 chunk 独立生成、输出即 memcpy 走：处理 B 时 A 的方块已不在内存 → 既看不到 A 的跨 chunk 写入，也无法产生「A 先写、B 后写覆盖」。复刻方案（phase3.5 采用）：**两阶段生成** —— ① surface+carvers 全部 chunk 先生成并缓存 regionCols；② features 阶段逐 chunk 串行，把跨 chunk 写入暂存为 pendingCross 队列，等相邻 chunk 生成后按「后写覆盖」应用。块级提升：-288 FULL 96.67%→97.8464%（nonAir 93.6490%），granite 56.2%→88.3%、dirt 92.7%。

### 证据
- `.investigations/feature-pipeline/cmd-output/phase35_crosschunk_result.txt`：`two-phase (surface+carvers store regionCols -> features pendingCross apply A-overwrites-B)`；-288 FULL 97.8464%、300515 98.0948%；granite 88.3% diorite 85.7% tuff 87.8% dirt 92.7%
- `block_probe.cpp L243-246`：`WG_GEN_MODE=full` 时两阶段 `wg_fill_blocks_multi_phase(h,...,1)` + `(...,2)`（阶段 1 surface+carvers 存 regionCols；阶段 2 features 串行跨 chunk 写）
- `pipeline-map.md` §6.1：Java CARVERS/FEATURES 独立 ChunkStatus、跨 chunk 邻域读取（generateFeatures 读 3×3）——单 chunk 生成会丢邻域越界部分

### 如何利用
- 凡复刻「世界按 chunk 序 + feature 可跨界写」的生成器（MC 1.18+ 均如此），**MUST 两阶段**：先无 FEATURE 阶段全量落 regionCols，再 FEATURE 阶段串行逐 chunk 处理，跨 chunk 写入走待应用队列
- 后写覆盖语义 = 以「最后生成该位置的 chunk 的写入」为准，不能各 chunk 独立求解（独立求解会丢 A 写入 B 的块或产生重复）
- 区域缓存只需覆盖 feature 的跨界半径（ore 椭球/geode 小；树冠/大 feature 大），先按 3×3 邻域缓存 + pending 队列即可复现绝大多数跨界
- 判定「跨 chunk 差异」前先确认参照是 FULL（含 FEATURE）还是 SURFACE——FULL 才有此语义（见 workflow-patterns 发现 #4）

---

## 发现 #9: blocks 参照文件每 chunk 后跟 256 项 biome 段（2+len 结构）——读取脚本 MUST 跳过，否则 chunk 坐标错位（int16 溢出假象）

**发现时间:** 2026-08-10
**发现者:** worker（FEATURE Phase 2 参照读取修复）
**来源定位:** C++ `versions/1.20.1/cpp/worldgen/src/block_probe.cpp L194-235`（参照 blocks 文件解析）
**置信度:** confirmed（2026-08-10 已验证；修复后 SURFACE 零退化 + carver 闭合）
**module:** swe

### 观察
BlockProbe 导出的 vanilla 参照 blocks 文件（大端）chunk 记录格式为：
```
header: magic(4) + seed(8) + size(4) + originX(4) + originZ(4) + minY(4) + height(4)
每 chunk:
  cx(4) + cz(4)
  BPC = 16*16*height 个 uint16 方块状态
  biome 段：256 项（16×16），每项 = 2 字节长度前缀(blen) + blen 字节 UTF 字符串（writeUTF）
```
**关键坑**：每 chunk 的方块数据后**紧跟 256 项 biome 段**（-288/300515 参照一直含 biome 段）。若读取脚本不跳过（或按 `blen<128` 截断读），后续 chunk 的 `cx/cz` 实际读到的是 biome 名 UTF 字节拼接出的值——坐标瞬间变成超大/负值，表现为「**int16 坐标溢出假象**」，实际是**流错位**：后续所有 chunk 的坐标与方块数据全部错位，对比结果无意义（可能伴随栈越界写，如 `buf[8]` 容纳 blen 超 8 字节的 biome 名）。

### 证据
- `block_probe.cpp L199` 注释：`biome 段字符串 blen < 128，必须 ≥ 128（曾用 8 导致栈越界写）`
- `block_probe.cpp L226-231`：显式跳过 256 项 biome 段（每项先读 2 字节长度，再读 blen 字节）；L230 `writeUTF 长度无上界（biome 名 ≤ 64，但安全读全部）`
- `.investigations/feature-pipeline/cmd-output/phase2_carvers_result.txt` 关键修复链 #1：`block_probe biome 段跳过 bug（blen<128 截断）→ 参照读取错误 → 修复（参照一直含 biome 段）`

### 如何利用
- 解析此类参照 blocks 文件：读方块数据后 **MUST 按 256 项 ×（2 字节长度 + blen 字节）跳过 biome 段**，再读下一 chunk
- 长度前缀是大端 uint16；blen 用「读 2 字节得长度 → 读满长度字节」安全读，**不要假设长度上界**（栈缓冲 ≥ 128 或按长度动态分配）
- 诊断信号：chunk 坐标读出来是异常值/负值（看似 int16 溢出）而 header 的 origin/size 正常时——先怀疑**流错位**（漏跳/错跳变长字段），不是坐标本身溢出
- 用参照文件做对比的脚本（任何语言）都要按此格式解析；跨版本格式若加字段同样适用「未知变长段 MUST 先确认结构再跳」
