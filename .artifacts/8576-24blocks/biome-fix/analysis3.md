# 8576-24blocks biome-fix #23/#24 最终定论：find 平局 tie-break（SearchTree vs 线性遍历）

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6 chunks
> 范围：#23 (812,73,-337) C++ stone vs 参照 terracotta（badlands）；#24 (815,89,-337) C++ grass vs 参照 terracotta（badlands 带顶差 1）
> 角色：anchor.worker 精确分析 subagent（只读；不改代码，patch 建议供主会话）
> 本文件：第 3 步（最终定论），承接 `analysis.md`（第 1 步）/ `analysis2.md`（第 2 步）
> 日期：2026-08-09　状态：**draft**

---

## 0. 结论先行（定论三问一句话版）

1. **参照 terracotta 的 biome 来源**：vanilla 判定点与 C++ **同一选点 cell (203,18,-84) → block (812,72,-336)**，6 维与 C++ 一致（t=5500/hum=-946/cont=161/ero=-4442/dep=1039/w=-5418，浮点级差不改变 toLong）→ **vanilla 判定该点为 badlands 系不是靠「湿度 ≤-0.1」**（analysis2 的湿度硬约束结论**错误**，badlands humidity 实际含 `[-0.1,0.1]` 组）——**而是这组 6 维下 forest 条目与 badlands 条目的最近邻距离完全相同（平局）**，vanilla SearchTree 平局取 badlands、C++ 线性取 forest。
2. **SURFBIOME 矛盾定性**：RouterProbe 原始输出坐标是 **floor 对齐 (812,y,-340)**（非数据包 §4 声称的 (812,y,-337)），输入坐标错位 → 选点 cell 与 C++ 判定点不同 → **SURFBIOME 判 savanna/badlands 不能作为同点证据**。手动 BiomeAccess 与游戏 ChunkRegion BiomeAccess 的 seed/storage 路径等价，仅输入坐标被 RouterProbe 对齐破坏。
3. **C++ 根因**：**`biome.h:221-234` find 用线性遍历 + 严格 `<`（等价 Java 测试用 getValueSimple），而 vanilla 运行时用 `MultiNoiseUtil.SearchTree`（MultiNoiseUtil.java:379-604），平局 tie-break 不同**。判定点 t=5500 恰在 forest 温度上界 [2000,5500] 与 badlands 下界 [5500,10000] 的公共边界，产生**距离完全相等**的平局（16,406,746），C++ 取 entries 首个（forest 在 biome_params.json 中先于 badlands）→ forest；vanilla 树序遍历取 badlands → terracotta。

---

## 1. 关键更正（推翻 analysis2 的核心假设）

### 1.1 「badlands 湿度必须 ≤-0.1」是错的

analysis2 §1 断言 badlands/eroded_badlands humidity ∈ [-1.0,-0.35] ∪ [-0.35,-0.1]（toLong ≤ -1000），并据此推断「C++ 湿度 -946 差 54 是判 forest 的压倒性驱动」——**此结论错误**。

`biome_params.json`（Java BiomeParamProbe 导出的 vanilla 运行时参数表）L600-671 显示 `minecraft:badlands` 的 humidity 实际有**三组**：

| 组 | humidity range | toLong | 代表行 |
|---|---|---|---|
| A | [-1.0,-0.35] | [-10000,-3500] | L600-615 |
| B | [-0.35,-0.1] | [-3500,-1000] | L628-643 |
| C | **[-0.1,0.1]** | **[-1000,1000]** | **L656-671** |

**C++ 判定点 hum=-0.094668537 → toLong -946 落在 C 组 [-1000,1000] → 湿度命中 badlands，距离 0！**

⇒ **湿度不是硬约束。** analysis2 据此推断的「vanilla 湿度必须 ≤-0.1」「湿度差 ≥0.0047」全部作废。真正把 vanilla 与 C++ 分开的**不是 6 维值差，而是 find 平局语义**。

### 1.2 analysis2 的 compdump 坐标错位缺陷（任务已提示，此处确认）

- analysis2 用 `compdump(-337)`（C++ 诊断直采 @ z=-337：t=0.549879/hum=-0.095321）推断湿度差 0.0047/0.0054——**坐标错位**：
  - 判定点实际是 biomePickCell 选点后的 **block (812,72,-336)**（数据包 §2），t=**0.550060272**、hum=**-0.094668537**（与 compdump 的 -337 值不同）
  - Java RouterProbe **B 行**是 floor 对齐坐标 **(812,72,-340)**（t=0.548046/hum=-0.096363），也不是判定点
  - **C++ vs Java 同坐标（-340）对比仅差 ~9e-6**（数据包 §3）→ 6 维采样本身 C++/Java 浮点级一致，**不存在 0.0047/0.0054 的真实差**

---

## 2. 定论一问：参照 terracotta 的 biome 来源（vanilla 判定点坐标与 6 维）

### 2.1 判定路径三方对齐（源码逐行，本次复核）

| 环节 | Java（权威） | C++ | 判定 |
|---|---|---|---|
| surface 逐块 biome | `SurfaceBuilder.buildSurface` L155 `initVerticalContext(q,vx,r,m,u,n)` → `MaterialRules.BiomeMaterialCondition`（L195）`context.biomeSupplier.get()`；`biomeSupplier = memoize(posToBiome.apply(pos.set(blockX,blockY,blockZ)))`（L464，**逐块重建**） | `surface.h:765` `biomeAtCached(m,wy,n)`（逐块 biomePickCell） | ✓ 逐块语义一致 |
| biome 取点 | `BiomeAccess.getBiome(BlockPos)` L30-64（8 邻域选点）→ `storage.getBiomeForNoiseGen(px,py,pz)`；storage=ChunkRegion → `WorldView.getBiomeForNoiseGen`（L65-68）→ **chunk.getBiomeForNoiseGen**（chunk biome 数组） | `biome.h:121-149` biomePickCell → `worldgen_api.cpp:727` → 6 维采样 @ (px<<2,py<<2,pz<<2) | ✓ 选点一致（seed=hashSeed(worldSeed) 双方一致，字节序 analysis.md 已确认） |
| chunk 数组填充 | `ChunkGenerator.populateBiomes` → `Chunk.populateBiomes` → `ChunkSection.populateBiomes` L189 `biomeSupplier.getBiome(x+j,y+k,z+l,sampler)`；`MultiNoiseBiomeSource.getBiome` L66-68 = `find(noise.sample(x,y,z))` | —（C++ 不建 chunk 数组，surface 直接采样） | 语义等价：**同一 cell 的 6 维采样坐标 = (px<<2,py<<2,pz<<2)** |
| 6 维采样 | `MultiNoiseSampler.sample` L222-235：`toBlock(x)=x<<2` → UnblendedNoisePos → 6 个 density 函数 | `worldgen_api.cpp:728-740` p.x=px<<2 等 | ✓ 一致 |
| 查找 | **`MultiNoiseUtil.SearchTree.getValue`（L146-152，运行时实际）** | `biome.h:221-234` 线性遍历 `dist < bestDist`（等价 Java **getValueSimple** L122-139，仅测试用） | ⚠️ **平局 tie-break 不同（本任务根因）** |

### 2.2 vanilla 判定点坐标与 6 维

- 判定输入 (812,73,-337)：`i=810,j=71,k=-339,l=202,m=17,n=-85,d=0.5,e=0.75,f=0.25` → 8 邻域选点（seed=hashSeed(8576)）→ **cell (203,18,-84)**（数据包 §2 C++ 实测；seed 一致 ⇒ vanilla 相同）
- 6 维 @ block (812,72,-336)（C++ 实测，数据包 §2；vanilla 同坐标浮点级 ~9e-6 差，**不改变 toLong**）：

| 分量 | 值 | toLong | 关键判定 |
|---|---|---|---|
| temperature | 0.550060272 | **5500** | forest 上界 [2000,5500] ∩ badlands 下界 [5500,10000] **公共边界** |
| humidity | -0.094668537 | **-946** | badlands C 组 [-1000,1000] ✓（analysis2 误判为不命中） |
| continentalness | 0.016117165 | 161 | 双方命中 |
| erosion | -0.444270968 | -4442 | 双方命中 |
| depth | 0.103940725 | 1039 | 双方都偏离 [0,0]/[1,1]（距离 1039） |
| weirdness | -0.541882336 | -5418 | 双方都偏离 [-10000,-9333]（距离 3915） |

### 2.3 平局计算（决定性）

**forest 条目**（biome_params.json L518 组：temp[0.2,0.55] hum[-0.1,0.1] cont[-0.11,0.3] ero[-0.7799,-0.375] depth[0,0] weird[-1.0,-0.9333]）：
- temp 5500→0，hum -946→0，cont 161→0，ero -4442→0，depth 1039→1039，weird -5418→3915
- dist² = 1039² + 3915² = **16,406,746**

**badlands 条目**（biome_params.json L658 组：temp[0.55,1.0] hum[-0.1,0.1] cont[-0.11,0.3] ero[-0.7799,-0.375] depth[0,0] weird[-1.0,-0.9333]）：
- temp 5500→0，其余全同 → dist² = **16,406,746**

**距离完全相同 → 平局。** 双方 offset=0（offset²=0 不区分）。

- **C++**：`biome.h:228` `if (bestDist < 0 || dist < bestDist)` 严格 `<` 取首个；biome_params.json 中 forest（L238/378/516…）全部在 badlands（L600+）之前 → **取 forest**
- **vanilla**：`SearchTree.TreeBranchNode.getResultingNode`（MultiNoiseUtil.java:541-560）`if (l > m)`/`if (l > n)` 严格大于，**平局不更新 → 返回树序遍历第一个最小距离 leaf**；树序遍历序由 `createNode` 的排序决定（L404-449，与 entries 顺序无关）→ **取 badlands**（参照 terracotta 铁证）

### 2.4 判定 (a)/(b)/(其他) 结论

- **(a) 选点差：排除。** seed 双方一致（hashSeed 字节序 analysis.md 已确认）+ biomePickCell 逐行一致（analysis.md §1 步骤 1）→ 选点 cell 相同 (203,18,-84)。
- **(b) 6 维值差：排除。** 同坐标（-340）C++/Java 差仅 ~9e-6（数据包 §3），浮点级；判定点 t=0.55006 距 0.55 边界 0.00006，9e-6 差无法跨过 toLong 阈值（0.550069→5500 仍 <5501）。
- **(其他) find 平局 tie-break：命中。** 距离完全相同（§2.3）→ C++ 线性 vs vanilla SearchTree 平局行为不同 → 结果不同。
- **vanilla 判定点最可能坐标 = (812,72,-336)（block）/ cell (203,18,-84)，6 维 = C++ 实测值**（浮点级差不变 toLong）。

---

## 3. 定论二问：SURFBIOME 矛盾定性

### 3.1 原始输出坐标证据（RouterProbe 原始 stdout）

`routerprobe_812_-337.txt` 全部 B 行与 SURFBIOME 行坐标都是 **`(812,y,-340)`**，不是数据包 §4 声称的 `(812,y,-337)`：
```
B 812 64 -340 0.548046 ...
SURFBIOME 812 64 -340 minecraft:savanna
BIOME 812 64 -340 minecraft:forest
```
815 文件的 B/SURFBIOME 行同样是 `(812,y,-340)`（x 也被对齐到 812=floor(815/4)*4）。

⇒ **RouterProbe 的 B 行（floor 对齐坐标直采 6 维）与 SURFBIOME（输入 floor 对齐坐标的 BiomeAccess）都偏离了原始判定点 (812,y,-337)。**

### 3.2 手动 BiomeAccess vs 游戏 ChunkRegion BiomeAccess 路径

| 项 | 游戏实际（ChunkRegion） | RouterProbe SURFBIOME（手动） | 判定 |
|---|---|---|---|
| seed | `ChunkRegion.java:102` `new BiomeAccess(this, BiomeAccess.hashSeed(this.seed))`，seed=world.getSeed()=8576 | `new BiomeAccess(storage, BiomeAccess.hashSeed(seed))`，seed=world.getSeed()=8576 | ✓ 一致 |
| storage | ChunkRegion（WorldView）→ `WorldView.getBiomeForNoiseGen` L65-68 → `chunk.getBiomeForNoiseGen`（chunk 数组） | `(bx,by,bz)->bs2.getBiome(bx,by,bz,multiNoiseSampler)` = `find(sample(bx,by,bz))` | ✓ 语义等价（chunk 数组 = populateNoise 时 `MultiNoiseBiomeSource.getBiome` = `find(sample(cell))`，同一 cell 同 6 维） |
| 输入坐标 | surface 判定 (812,73,-337)（原始） | RouterProbe 输入 **(812,y,-340)**（被 floor 对齐） | ⚠️ **错位** |

### 3.3 矛盾解释（决定性）

- SURFBIOME(812,y,-340)：`biomePickCell((812,y,-340))` 的选点 cell ≠ C++ 判定点 cell (203,18,-84)。例：z=-340 → k=-342 → n=-86（≠-85），jitter 偏移不同 → 选点 cell 在 z=-86/-85 一带（block -344/-340），而非 -84（block -336）。
- SURFBIOME @ (812,84/88/96,-340) 判 badlands、@ (812,64/72,-340) 判 savanna：都是**错位 cell 的 6 维**（t≈5498-5500 边界摆动）下的 SearchTree 结果——**与参照方块 (812,73,-337)=terracotta 不矛盾**（不同点）。
- **SURFBIOME 可信度：作为「vanilla SearchTree 在特定 6 维下判 badlands/savanna」的旁证可用；作为「同判定点 vanilla vs C++」的直接证据不可用（坐标错位）。**

---

## 4. 定论三问：C++ 根因 + 修复

### 4.1 根因（文件:行 + 精确差异）

| 项 | Java（权威，运行时实际） | C++（错误） | 位置 |
|---|---|---|---|
| 查找 | `MultiNoiseUtil.SearchTree.getValue`（L146-152）→ `tree.get`（L520-526）→ `TreeBranchNode.getResultingNode`（L541-560）**平局返回树序遍历第一个** | 线性遍历 + `dist < bestDist` 严格 `<` **平局返回 entries 首个** | Java `MultiNoiseUtil.java:379-604`；C++ `biome.h:221-234` |

- 平局触发条件：判定点 t=**5500** 恰在 forest 上界 [2000,5500] 与 badlands 下界 [5500,10000] 的公共边界（§2.3 距离完全相同 16,406,746）。
- **#23**（(812,73,-337) 整带缺失）：C++ 判 forest → badlands surface 段不触发 → terracotta 带缺失。
- **#24**（(815,89,-337) 带顶差 1）：不同 y 的选点 py 不同 → 不同 cell → 温度在 5500 边界两侧摆动，平局/非平局交替；C++ 与 vanilla 的翻转位置因平局语义差 1 格（C++ y=89、vanilla y=90）。
- **非根因（排除）**：6 维采样值（同坐标 9e-6）、选点 cell、seed、hashSeed 字节序、参数表结构、spline/float 精度、depth 偏移（历史已修）。

### 4.2 patch 建议（不修代码，供主会话）

**方案 A（首选，与 vanilla 逐位对齐）**：在 C++ 移植 `MultiNoiseUtil.SearchTree`。

```
新增 versions/1.20.1/cpp/worldgen/src/searchtree.h（或并入 biome.h）：
- struct SearchTreeNode { ParameterRange params[7]; virtual long getSquaredDistance(long(&p)[7]); }
- struct TreeLeafNode : SearchTreeNode { std::string id; }      // getResultingNode 返回 this
- struct TreeBranchNode : SearchTreeNode { std::vector<SearchTreeNode*> sub; }  // getResultingNode 按 L541-560 语义
- class SearchTree {
    TreeNode* first;
    static TreeNode* createNode(int paramNum, std::vector<TreeNode*>& subTree);  // 移植 L404-449
    static void sortTree(...)/createNodeComparator(...)/getBatchedTree(...)/getEnclosingParameters(...)  // L451-518
    const std::string* get(long t,long h,long c,long e,long d,long w);           // 移植 L520-526 + TreeNode.getSquaredDistance L590-598
  };
BiomeSource 构造（loadFromJson 后）构建 SearchTree；find() 改为 searchTree.get(...)。
```

改动要点：
- 7 维参数顺序与 Java `getParameters()`（L297-307）一致：temperature,humidity,continentalness,erosion,depth,weirdness,offset。
- 平局语义：`if (l > m)`/`if (l > n)`（严格大于）——**与 C++ 现有 `dist < bestDist` 相反方向**。
- `TreeLeafNode.getResultingNode` 直接返回 this（L572-576）。
- 排序：`createNode` 小树按 `Σ|(min+max)/2|`（L410-419），大树递归 `sortTree`（L426-448）。

**方案 B（过渡验证）**：不改结构，仅在 `find` 中复现「平局取树序第一个」——但树序依赖完整构建，无法用简单规则近似，**不推荐**（可能引入新的与 vanilla 不一致）。

**方案 C（快速探针）**：先加诊断确认（主会话执行）：
```
# 在 C++ find 中输出 6 维 toLong + 距离最小 Top3（含 forest/badlands 的距离），验证平局
# 或在 block_probe 增加 -biomeDumpTopN 输出 Top3 距离
```

### 4.3 影响面（3200 铁律风险）

| 维度 | 评估 |
|---|---|
| 修复对象 | `biome.h:221-234` find（全局所有 biome 判定）——**不动 temperature/vegetation 噪声链路**（2D 分量值不受影响） |
| 受影响点 | 仅「距离完全相等」的平局点（multi-biome hypercube 边界，如 t=5500 这种恰好落在相邻 biome 区间公共边界的点）翻转；远离边界的点**不受影响**（距离唯一胜者不变） |
| 3200 当前 100% | 说明 3200 区域要么无平局点、要么当前平局结果已与 vanilla 一致。若存在 C++ 误判的平局点，3200 不可能 100% → 修复方向是「向 vanilla 对齐」（正确性提升），不会把已正确的判定改错 |
| 风险 | **中**：find 全局参与 + 3200 是铁律参照 → 修复必须全量回归 `-288 / 3200 / 20000 / 8576` 四套参照，**3200 必须保持 diff=0**；若出现平局点翻转需逐点确认对齐 vanilla |
| 附带 | 修复后 8576 参照 (812,-337) 区域 terracotta 带应恢复；注意 `-biomeDump`/`WG_BIOMEDUMP` 输出也会随之改变（同一 find） |

---

## 5. 置信度与局限

- **高置信（平局根因）**：§2.3 距离计算精确相同（16,406,746）；forest/badlands 条目参数逐条核对；C++ 线性 `<` 与 Java SearchTree 严格 `>` 平局语义差异直接源自源码。
- **高置信（排除 6 维/选点差）**：同坐标 9e-6（数据包 §3）、seed/字节序一致（analysis.md）。
- **中高置信（vanilla 平局取 badlands）**：参照 terracotta 铁证 + 温度边界分析自洽；但**未运行时验证** vanilla SearchTree @ (812,72,-336) 的实际返回值（RouterProbe 无该坐标同点数据）。
- **局限**：
  1. 无 Java 运行时同点（(812,72,-336)）SearchTree 输出，vanilla 取 badlands 是「参照铁证 + 平局必然」的反推。
  2. SearchTree 树序遍历具体落到哪个 leaf 需移植后实测确认；若 vanilla 实际取 savanna 之外的第三个 biome 系（不可能——terracotta 需 badlands 系），本结论需修订。
  3. RouterProbe SURFBIOME 的 floor 对齐细节从原始输出推断（未见源码），若有偏差需主会话核对 RouterProbe.java。
- 状态：**draft**（移植 SearchTree 实测后可由主会话/审查提升；AI 不写 confirmed）。

---

## 6. 产物引用

- 本文件：`.artifacts/8576-24blocks/biome-fix/analysis3.md`
- 前序：`analysis.md`（选点/公式/参数表逐行对拍）、`analysis2.md`（温度链路，**其中「badlands 湿度必须 ≤-0.1」「湿度差 0.0047/0.0054」结论被本文件 §1 推翻**）
- 数据包：`.investigations/8576-24blocks/biome-fix-datapack.md`（§2 C++ 判定输入、§3 B 行、§4 SURFBIOME 原始、§5 SurfaceBuilder 对齐）
- RouterProbe 原始：`.investigations/8576-24blocks/routerprobe/routerprobe_812_-337.txt`、`routerprobe_815_-337.txt`（**B/SURFBIOME 行坐标 = floor 对齐 -340**）
- 参数表：`versions/1.20.1/data/biome_params.json`（forest L238-541 含 [0.2,0.55] 组；badlands L600-671 含 humidity [-0.1,0.1] C 组；savanna L462-513 等）
- 源码：Java `BiomeAccess.java`、`ChunkRegion.java`（L102）、`WorldView.java`（L65-68）、`Chunk.java`（L422/437-448）、`ChunkSection.java`（L182-195）、`MultiNoiseBiomeSource.java`（L66-68）、`MultiNoiseUtil.java`（L146-152、L213-235、L287-295、L362-366、L379-604）、`MaterialRules.java`（L195、L462-469）、`SurfaceBuilder.java`（L107-109、L119、L155）；C++ `biome.h`（L76-84、L103-149、L166-180、L221-234）、`worldgen_api.cpp`（L475-503、L720-743）、`surface.h`（L691-702、L765）
