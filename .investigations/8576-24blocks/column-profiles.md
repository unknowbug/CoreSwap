# 8576 剩余 24 mismatch — 列剖面形态快照

> recode.scout 只读勘探产物（不修改任何代码/参照文件）
> seed=8576294172403134396，区域 720,-432 6×6 chunks（chunk 45..50 × -27..-22）
> 参照：vanilla_8576294172403134396_6_720_-432.blocks（read_col2.py）
> C++：block_probe.exe -blockDump（内部 ID 与 blocks.json 映射一致：0=air 1=stone 8=grass 9=dirt 32=water 425/426/494=terracotta 系 970=deepslate）
> 注：首次并行 read_col2 曾读到不一致数据（疑似文件当时未就绪）；已重跑全部列确认稳定，所有数据以本文件为准。

---

## 一、深板岩/水边界类（#1-3,7-9,13,16,17,21，12 块，savanna）

### 代表列 1：(764,-417) — mismatch #1 (764,-31,-417) got=water vanilla=deepslate

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| -35 | deepslate | 970 | ✓ |
| -34 | air | 0 | ✓ |
| -33 | air | 0 | ✓ |
| -32 | deepslate | 970 | ✓ |
| **-31** | **deepslate** | **32 water** | ✗ **#1** |
| -30 | deepslate | 970 | ✓ |
| -25..-2 | deepslate | 970 | ✓ |
| -1..10 | water | 32 | ✓ |

形态：整列参照与 C++ 几乎全同，仅 -31 一格 C++ 是 water、参照是 deepslate——**深板岩床内部孤立 water 格**（参照床顶 -32，C++ 在 -31 误放一格水）。

### 代表列 2：(790,-432) — mismatch #2 (790,2,-432) got=deepslate vanilla=water

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| -6 | air | 0 | ✓ |
| -5..-2 | deepslate | 970 | ✓ |
| -1 | air | 0 | ✓ |
| 0 | deepslate | 970 | ✓ |
| 1 | water | 32 | ✓ |
| **2** | **water** | **970 deepslate** | ✗ **#2** |
| 3..4 | stone | 1 | ✓ |
| 5 | deepslate | 970 | ✓ |
| 6..10 | stone | 1 | ✓ |

形态：水层参照 1..2 两格，C++ 只有 y=1 一格，y=2 被 deepslate 顶替——**C++ 水面/水底边界高 1 格**（水层薄 1）。

### 补充列（同机制）
- **(804,-420)** #3 (y=-2)：参照 water -2..2，C++ -2 deepslate、-1..2 water → C++ 水底高 1。
- **(810,-415)** #13 (y=-4)：参照 water -4..1，C++ -4 deepslate、-3..1 water → C++ 水底高 1。
- **(802,-372)** #16 (y=0)：参照 0..1 deepslate，C++ 0 air、1 deepslate → C++ 深板岩顶低 1（air 侵入）。
- **(807,-347)** #21 (y=0)：参照 0 air、1..5 deepslate，C++ 0 deepslate → C++ 深板岩顶高 1（air 被填）。
- **(810,-355)** #17 (y=-11)：参照 -11 air（洞底）、-10..5 deepslate，C++ -11 deepslate → C++ 洞穴底高 1（air 被填）。
- **(764,-416)/(764,-415)** #7,8,9：同 (764,-417)，C++ 在深板岩床 -32/-31 误放 water。

**类内一致性**：#2/#3/#13 是「C++ 水面/水底 +1（水少 1 格）」，#16/#17/#21 是「C++ 深板岩/空气边界 ±1（air 被填或 air 侵入）」，#1/#7/#8/#9 是「深板岩床内部多 water 格」。共同点：**全部发生在 deepslate↔water↔air 的边界 ±1 格内，无大段差异**。观察 #16 (802,0) 与 #21 (807,0) 为互补翻转（同一 y=0，一处 C++ 少 deepslate、一处多）。

疑似机制方向：aquifer 液面 / 河床深度 / deepslate 床顶判定存在 ±1 偏差（边界翻转）。

---

## 二、地表三连错位类（#4-6,10-11,18-20 及 #12，#14，9 块，savanna）

### 代表列 1：(743,-406) — mismatch #4 (y=68) #5 (y=71) #6 (y=72)

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| 55..67 | stone | stone | ✓ |
| **68** | **dirt** | **stone** | ✗ #4 |
| 69..70 | dirt | dirt | ✓ |
| **71** | **grass** | **dirt** | ✗ #5 |
| **72** | **air** | **grass** | ✗ #6 |
| 73+ | air | air | ✓ |

形态：参照 stone 顶 67 / dirt 68-70 / grass 71；C++ stone 顶 68 / dirt 69-71 / grass 72。**C++ 整列地表层整体高 1 格**（stone→dirt→grass 三连段同步 +1）。

### 代表列 2：(800,-363) — mismatch #18 (y=72) #19 (y=76) #20 (y=77)

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| 55..71 | stone | stone | ✓ |
| **72** | **dirt** | **stone** | ✗ #18 |
| 73..75 | dirt | dirt | ✓ |
| **76** | **grass** | **dirt** | ✗ #19 |
| **77** | **air** | **grass** | ✗ #20 |
| 78+ | air | air | ✓ |

形态：参照 stone 顶 71 / dirt 72-75 / grass 76；C++ stone 顶 72 / dirt 73-76 / grass 77。**同样整体 +1**。

### 补充列
- **(754,-403)** #10 (y=56) #11 (y=61)：参照 stone 54-55 / dirt 56-60 / water 61-62 / air 63+；C++ stone 54-56 / dirt 57-61 / water 62 / air 63+ → 地表整体 +1（61 的水被 dirt 顶替，62 水保留）。水塘也同步 +1。
- **(771,-410)** #12 (y=41)：参照 stone 35-40 / water 41+；C++ stone 38-41 / water 42+ → C++ 水面 +1。
- **(723,-393)** #14 (y=9)：参照 water 5..9 / stone 10+；C++ water 6..8 / stone 9+ → 与 #2 同向（C++ 水底高 1），非地表 +1 型。

**类内一致性**：743 / 800 / 754 / 771 四列全部为**地表整列 +1**（stone/dirt/grass/水面同步抬 1），高度一致。723 列为水底 +1（与深板岩类 #2 同机制）。

疑似机制方向：地表高度（est / runDepth / surface 分量）在 savanna 整体差 1，导致 stone→dirt→grass 三连段同步偏移。

---

## 三、forest terracotta 类（#23,24，2 块）

### 代表列 1：(812,-337) — mismatch #23 (y=73) got=stone vanilla=terracotta

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| 60..72 | stone | stone | ✓ |
| **73** | **terracotta** | **stone** | ✗ #23 |
| 74..82 | stone | stone | ✓ |
| 83..85 | dirt | dirt | ✓ |
| 86 | grass | grass | ✓ |
| 87+ | air | air | ✓ |

形态：参照在 stone 层深处 y=73 有一层 terracotta（badlands 深层 terracotta 带），C++ **整层缺失**。地表（83..86）完全一致。
参照 blocks 文件 biome 段 (812,-337) z*16+x = **badlands**；C++ -biomeDump = **forest**。

### 代表列 2：(815,-337) — mismatch #24 (y=89) got=grass vanilla=terracotta

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| 60..83 | stone | stone | ✓ |
| 84 | orange_terracotta(426) | 426 | ✓ |
| 85 | stone | stone | ✓ |
| 86 | dirt | dirt | ✓ |
| 87 | white_terracotta(425) | 425 | ✓ |
| 88 | terracotta(494) | 494 | ✓ |
| **89** | **terracotta(494)** | **grass** | ✗ #24 |
| 90 | air | air | ✓ |

形态：参照 terracotta 带 87(white)+88+89 三层；C++ 87+88 两层、**89 被 grass 顶替** → terracotta 带顶差 1 格（短一层）。
参照 blocks 文件 biome 段 (815,-337) z*16+x = forest（与 terracotta 带并存，疑 biome 段记录问题）；C++ -biomeDump = forest。

**类内一致性**：两列均为 badlands 深层 terracotta 带机制，但形态不同：812 为整带缺失（biome 判定差：C++ forest vs 参照 badlands），815 为带顶差 1（terracotta 带高度/表面规则差 1）。

疑似机制方向：
- #23：**biome 判定差异**（C++ 判 forest，vanilla 为 badlands → 深层 terracotta 带规则不触发）。
- #24：badlands terracotta 带顶高度/地表替换规则差 1（grass 多盖一层）。

---

## 四、river 类（#15，1 块）

### 代表列：(733,-382) — mismatch #15 (y=26) got=stone vanilla=water

| y | 参照 | C++ | 一致 |
|---|---|---|---|
| 10..13 | stone | stone | ✓ |
| 14..25 | water | water | ✓ |
| **26** | **water** | **stone** | ✗ #15 |
| 27..40 | stone | stone | ✓ |
| 41..42 | gravel | — | (未采样) |
| 43..62 | water | water | ✓ |
| 63+ | air | air | ✓ |

形态：参照 river 主水体 14..26（13 格），C++ 水体 14..25（12 格），y=26 被 stone 顶替 → **C++ 河床/水底高 1 格**。上层水体 43..62 与 gravel 层形态未测，但从主水体看是水底 +1 型。
参照 blocks 文件 biome 段 z*16+x = river；C++ -biomeDump = river（biome 判定一致）。

疑似机制方向：river 河床深度判定差 1（与深板岩类 #2/#3/#13 的「C++ 水底高 1」同形态）。

---

## 差异形态归纳表

| 类 | 块数 | 形态 | 疑似机制方向 |
|---|---|---|---|
| 深板岩/水边界 | 12 (#1-3,7-9,13,16,17,21) | 全部在 deepslate↔water↔air 边界 ±1 格：a) C++ 水底/水面 +1（#2,3,13）；b) C++ 深板岩↔air 翻转 ±1（#16,17,21，#16/#21 互补）；c) 深板岩床内孤立 water 格（#1,7,8,9） | aquifer 液面 / 河床深度 / deepslate 床顶判定 ±1 |
| 地表三连错位 | 9 (#4-6,10,11,12,18,19,20) | 743/800/754/771 四列 stone→dirt→grass→air 三连段**整体 +1**（含水面/水塘同步 +1） | 地表高度（est/runDepth/surface 分量）差 1 |
| 地表/水底边界 | (#14) | C++ 水底 +1（stone 顶高 1） | 同深板岩类水底 +1 机制 |
| forest terracotta | 2 (#23,24) | #23：badlands 深层 terracotta 带整层缺失（C++ biome=forest vs 参照=badlands）；#24：terracotta 带顶差 1（grass 多盖一层） | #23 biome 判定差异；#24 terracotta 带高度/表面规则差 1 |
| river | 1 (#15) | C++ 河床/水底高 1（主水体短 1 格） | river 河床深度判定差 1 |

## 跨类观察（重要）

1. **「+1 地表」与「+1 水底」是两种独立机制**：地表三连类是地表层整体抬 1（743/800/754/771 完全同步）；水底类（#2/#3/#13/#14/#15）是水底边界抬 1。两者在同一 seed 中共存。
2. **#16 (802,0) 与 #21 (807,0) 为互补翻转**：同一 y=0 高度，一处 C++ 少 deepslate（got=0 vs 970）、一处多（got=970 vs 0），疑同一 deepslate 床顶边界在两侧各偏 1。
3. **biome 段两个索引（x*16+z / z*16+x）结果不一致**（如 812: forest/badlands、733: savanna/river），需在 Phase 2 确认 blocks 文件 biome 段正确索引，避免 biome 判定结论被带偏。
4. 所有 24 块均无大段错位，全部为边界 ±1 或整层缺失类，与整体 99.9993% 匹配率相符。

## 产物引用

- 本文件：`.investigations/8576-24blocks/column-profiles.md`
- 明细：`.investigations/8576-24blocks/mismatch-list.md`
- 参照读取脚本：`versions/1.20.1/data/read_col2.py`、`read_biome2.py`
- C++ 采样：`versions/1.20.1/cpp/build-msvc/bin/block_probe.exe -blockDump/-biomeDump`
- 内部 ID 编码：`versions/1.20.1/data/blocks.json`
