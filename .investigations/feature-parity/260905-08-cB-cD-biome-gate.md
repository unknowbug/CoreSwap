# 260905-08 fan-out 候选 c-B/c-D —— biome 门（混合区）与并集集合差核实（core.worker）

- 角色：core.worker（fan-out 候选 c-B「biome 门在混合区偏差」/ c-D「并集集合差」）
- 输入：260905-08-scout-tree-rng.md + 260905-08-bB-set-diff.md + 260905-08-bA-stream-diff.md + 一手源逐行核实
- 状态：**draft**（Degraded 分层：纯静态对拍；沙箱无 shell，未运行任何探针/命令）
- Java 权威源：`.tmp/scout-260905-08/mcsrc/`（yarn 1.20.1+build.10-v2）；Rust：`worldgen-core/src/{biome,worldgen_handle,placement}.rs`；数据：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/biome/`
- **结论速览**：
  - **c-B：五环节静态全部通过，未发现门链错位**（access_seed ✔ / pick_cell ✔ / y 处理 ✔ / storage 等价性=surface 已验证域 ✔ / 允许集语义 ✔）。c-B 的机制前提（jitter 门判定错）**静态层面不成立**，残余风险仅剩「分类器在非 surface y 域 vs chunk 存储容器」——树门采样点在地表附近，该风险低。
  - **c-D：方向「Rust 漏」静态成立但解释不了 jungle；发现一个更强的「Rust 多」数据层机制**：`dripstone_caves.json` vegetal step 混入了 `trees_plains`/`flower_plains`/`patch_grass_plain` 等 plains 特有 feature（vanilla 1.20.1 dripstone_caves 的 vegetal 仅 glow_lichen/patch_tall_grass_2 级别，无树无花；同目录 lush_caves 列表与 vanilla 逐项一致，佐证这是 dripstone 单文件污染）。**Fix-1 并集（含 y=0 切片）把 dripstone_caves 拉进 biome_set → trees_plains 被执行 → Fix-2 门在 plains 地表 jitter 点放行 → 净多放 plains 树**——精确吻合「Fix-1 恶化 oak（+19,262→+29,708/+26,385）而 Fix-2 only ≈ 基线」的实验签名。

## 1. c-B 逐行核实（file:line）

### 1.1 access_seed 传入值 ✔
- Java `ChunkRegion.java:98` `this.seed = world.getSeed()`；`:102` `new BiomeAccess(this, BiomeAccess.hashSeed(this.seed))`；`BiomeAccess.java:22-24` `Hashing.sha256().hashLong(seed).asLong()`（Guava putLong=LE 8 字节，asLong=摘要前 8 字节 LE）。
- Rust `worldgen_handle.rs:389` `biome_access_seed: crate::biome::biome_hash_seed(seed)`；`biome.rs:302-306` sha256(seed.to_le_bytes()) 取前 8 字节 from_le_bytes —— 与 Guava 同构；且 `:249-299` sha256 常量表/填充/主循环标准实现，自审无偏移。**该 seed 已在 surface 链统一（260904-12 收口，worldgen_handle.rs:614-622 注释），全管线只有这一个 access seed**（:622/:633/:723/:850 四处调用点同一 `self.biome_access_seed`）。✔

### 1.2 biome_pick_cell 8 邻域选点 ✔（逐行）
- `biome.rs:343-372` ↔ `BiomeAccess.java:30-63`：`i=x-2`/`i>>2`/`(i&3)/4.0`、8 循环 bit 分解、扰动距离、**平局取严格小于首优**（Rust `best > v` ↔ Java `g > v`，同为「先到先得」序）——逐项一致。✔
- `biome_jitter`（`biome.rs:317-323`）↔ `method_38108`（`BiomeAccess.java:99-102`）：floorMod(l>>24,1024)/1024 → (d-0.5)*0.9 一致；`mix_seed`（:310-314）↔ SeedMixer.mixSeed 公式 `seed*(seed*6364136223846793005+1442695040888963407)+salt`（u64 回绕）一致；`biome_cell_distance`（:326-339）mix 顺序 q,r,s,q,r,s + 三次混入 l ↔ `method_38106`（:84-97）一致。✔
- y 处理：`j = y-2` 负数算术移位两侧一致（Rust i32>> 算术、Java int>> 算术）。Chunk 侧 clamp（见 1.3）不改变选点本身。✔

### 1.3 storage 语义（关键环节）——等价但依赖分类器一致（低风险，已声明）
- Java：`BiomePlacementModifier.java:35` `context.getWorld().getBiome(pos)` → `WorldView.java:45-46` → `BiomeAccess.getBiome` 选点 → `storage.getBiomeForNoiseGen(px,w,x)`；storage=ChunkRegion → `WorldView.java:65-67` default：`getChunk(SectionCoord(x), SectionCoord(z)).getBiomeForNoiseGen(...)` → `Chunk.java:422-428`：**clamp biomeY 到 chunk 范围后读已生成 section 的 4×4×4 biome 容器**（容器在 ChunkStatus.BIOME 阶段由噪声分类填充）。
- Rust：`worldgen_handle.rs:849-852` `biome_pick_cell → NoisePos{px<<2,py<<2,pz<<2} → self.biomesrc.biome(&bp)` 噪声即时分类。
- **等价条件**：即时分类（cell 角坐标）== Java 存储容器（BIOME 阶段同噪声同坐标填充）。该等价性在 **surface 域已验证**（surface 链即用此分类器且 surface 对齐）；树门采样点 y = heightmap 附近（地表），落在已验证域内 → **风险低**。`Chunk.java:426` 的 y clamp 仅在极端 y 触发，树门 y 在世界内，不触发。✔（附声明：非 surface 域未验证，登记 IDK）

### 1.4 允许集判定语义 ✔
- Java `GenerationSettings.java:88-90` `isFeatureAllowed` = `allowedFeatures`（**全部 step 的 placed feature 去重集合**）；`ChunkGenerator.java:668` `getGenerationSettings(entry)` 按采样点 biome entry 取。即判定 = **采样点 biome 的全 step feature 集合成员**，0 RNG 消费。
- Rust `worldgen_handle.rs:941-943` `biome_allows`：`features_for(&biome_at_jitter(...)).iter().any(step.any(f==fid))` = 全 step 成员判定 ✔；`features_for`（`biome.rs:491-493`）= biome JSON 的 per-step 表（空表 biome 不插入→空集 ✔ 同 Java 空表语义）；`placement.rs:311-323` Biome 臂接线 `biome_allows`/`feature_id`（fid=`indexer.step_features[k][p]`，:908），未接线时直通。✔
- fid 字符串相等 ↔ Java PlacedFeature registry entry 相等（registered by id 一一对应）等价。✔

### 1.5 c-B 判定
**门链五环节静态无错位**。若 Fix-1 恶化源于门判定错，则错位只能在运行时数据（分类器/JSON 表），而非代码逻辑——静态上 c-B 应降权。真正可静态指认的运行时数据问题见 c-D（§2.2）。

## 2. c-D 逐行核实

### 2.1 方向「Rust 漏」：成立但解释力弱
- Rust 并集 = 3×3 chunk × 16 个 xz cell（每 4×4 xz 恰采 1 点）× y∈{0,64,128}（`worldgen_handle.rs:861-873`）= 144 采样点/9chunk；Java = 每 chunk 4×4×(96 cell 全高)×9 chunk 全量 forEachValue（`ChunkGenerator.java:346-353`，bB §1.1 已核）。
- **不可能「多」**：采样点集 ⊆ 容器 cell 集（同分类器时）——「全高并集 ⊇ 切片并集」论证成立 ✔（前提=分类器一致，§1.3）。
- 「漏」的两个族：
  1. **洞窟 biome**（y=64/128 之间及 y<0 切片间隙的 lush_caves/dripstone_caves/deep_dark）——查 JSON： lush_caves vegetal 含 `rooted_azalea_tree`（azalea 叶，**不在 oak/jungle_leaves 残差签名族**）；dripstone_caves vegetal 原版无树；deep_dark 有 `trees_plains`（`deep_dark.json:78`）——deep_dark 在深板岩层，本 dump 区域 y=0 切片部分命中。→ 漏洞窟 biome 的树族效应 = azalea/deep_dark 边际，**解释不了 jungle −22k**（jungle 族 feature 只在 jungle/sparse_jungle/mangrove_swamp 地表 biome——`jungle.json:81`/`sparse_jungle.json:80`/`mangrove_swamp.json:83`，均属地表 y 带，y=64 切片+16 xz 列基本必命中存在的 jungle cell）。
  2. **海拔带 biome**（y 68~124 间隙，如 meadow `trees_meadow`）——本区域无高山，边际。
- 且实验反证：Fix-1 后 jungle −20,464→−22,295 **没有改善**——若 c-D「漏 jungle」是主因，并集落地应带来 jungle 收敛；没有 → **c-D「漏」方向对 jungle 主签名排除**（候选降权，留azalea/deep_dark小份额）。

### 2.2 ★新发现：方向「Rust 多」的数据层机制（本 worker 的核心产出）
`versions/1.20.1/data/worldgen/data/minecraft/worldgen/biome/dripstone_caves.json:82-91` vegetal step：
```
glow_lichen, patch_tall_grass_2, trees_plains, flower_plains, patch_grass_plain,
brown_mushroom_normal, red_mushroom_normal, patch_sugar_cane, patch_pumpkin
```
- vanilla 1.20.1 `dripstone_caves.json` 的 vegetal_decoration **不含任何树/花/plains patch**（只有 glow_lichen/patch_tall_grass_2 量级）；同目录 `lush_caves.json:79-89` 与 vanilla 逐项一致（含 rooted_azalea_tree 正确）——**单一文件污染特征**（疑似从 plains.json 复制覆盖）。
- **机制链（精确吻合实验签名）**：Fix-1 并集含 **y=0 切片**（`worldgen_handle.rs:867`）→ 有 dripstone_caves cell 的 chunk 其 biome_set 混入 dripstone_caves → 其（被污染的）`trees_plains` 进入 intSet → `set_decorator_seed` 后执行 → **Fix-2 门在 plains 地表 jitter 点放行（plains 允许 trees_plains）** → 净多放 plains 树/花。
  - 为什么 Fix-2 only ≈ 基线：无并集时 entries 只有 cur_biome（chunk 角 y=0 采样，:877-878），很少恰好是 dripstone_caves → 不触发。
  - 为什么 Fix-1-only 更糟（+29,708）：旧 anchor 门（==cur_biome）在**同质 plains chunk 直接放行全部新执行 feature** → 过放最大。两档实验数据（Fix-1-only 29,708 > Fix-1+2 26,385 > 基线 19,262）与「污染执行量恒定 + 门放行率 anchor≈jitter(plains地表)」自洽。
  - jungle 侧：dripstone 污染不含 jungle 族 → jungle 不动 ✔（与实验一致）。
- ⚠️ 置信度：draft。dripstone_caves 无树这一「vanilla 口径」来自项目知识（lush_caves 逐项一致佐证数据管线总体可靠），**未对 vanilla jar 原文件逐字节核对**——判别实验 E-cD-1 先行确认。

## 3. 量化预测与最小判别实验（主会话执行）

| 编号 | 实验 | 判据 |
|---|---|---|
| **E-cD-1（最便宜，先做）** | 核对 vanilla 口径：从 1.20.1 vanilla jar（或 mcwiki 数据页）解出 `data/minecraft/worldgen/biome/dripstone_caves.json` vegetal 列表比对。顺带全目录 diff 一遍 biome JSON（防同类污染）。 | 与 vanilla 不一致 → 数据污染实锤，转 E-cD-2 |
| **E-cD-2** | 修正 dripstone_caves.json（剔除 trees_plains/flower_plains/patch_grass_plain/mushroom/sugar/pumpkin 等 plains 项）→ 重生成同一确定性 dump（seed 8576294172403134396，载体 #52）→ A/B | **预测：Fix-1+2 的 oak +26,385 下降 ~5-8k（回到 ≤ 基线 +19,262 附近）；jungle 双向残差逐格不变（健全性检查）**。oak 不动 → 本机制排除，oak 恶化另寻 |
| **E-cB-1** | FEATURELOG 扩展：`worldgen_handle.rs:910` 打点行追加 `set_len/intset_len`（及 biome_set），挑 1 个「中心 plains、3×3 含 forest+dripstone cell」边界 chunk，与 Java mixin 的 setDecoratorSeed p 序列对拍 | Rust p 集多出 {trees_plains 等 plains 索引} 且该 chunk 含 dripstone cell → c-D 机制直接实锤（比 dump A/B 更便宜） |
| **E-cB-2（可选，仅当 E-cD-2 后 oak 仍异常）** | bin-diag：worst chunk (29,-16) 及一个混合 chunk，对 in_square 网格位置打印 `biome_at_jitter(x,y,z)` vs `biome_at_no_jitter`，与 Java SURFBIOME（floor 对齐口径，坐标语义先对齐——AGENTS.md 铁律 #23/#24 教训）三点交叉 | 若 jitter 判定与 Java SURFBIOME 在边缘位置系统性不一致 → c-B 复活（分类器运行时域差）；一致 → c-B 正式排除 |

## 4. 排除判据汇总

- **c-B 排除**：E-cB-1 的 p 集合按门分类器口径对拍一致 + E-cB-2 jitter 判定与 Java 交叉一致 → 排除（静态五环节已全过，运行时一致性即闭合）。
- **c-D「漏」方向排除（对 jungle 主签名）**：已静态+实验双排除（§2.1： jungle 族全为地表 biome，y=64 切片必命中；Fix-1 后 jungle 无收敛）。
- **c-D「多」（dripstone 污染）排除**：E-cD-1 证明该 JSON 与 vanilla 一致，或 E-cD-2 修后 oak 残差不动 → 排除。

## 5. 附带发现（供主会话）

1. **工作区现状**：`worldgen_handle.rs:876-882` Fix-1 处于**临时撤回态**（E-B1.4b 单变量实验，entries 只剩 cur_biome），`anchor_biome` 已删（Fix-2 已落地，:940-958）。c-D 的 E-cD-2 实验需在**恢复 Fix-1** 后做才有意义。
2. `cur_biome_id`（:857）= chunk 角 **y=0** 采样——基线（无并集）下若该点命中洞窟 biome，cur_features 即被污染（deep_dark 的 trees_plains！）；恢复 Fix-1 并集后此点并入集合不再有特殊地位，但该采样点选择本身与 Java 无对应（Java 无「中心 biome」概念），登记为已知偏差。
3. `dripstone_caves.json` 若确认污染，建议对 `data/worldgen/data/minecraft/worldgen/biome/` 全目录做一次与 vanilla jar 的机械 diff（同类污染可能不止一个文件）——数据驱动架构（AGENTS.md §四）下数据文件即代码，值得进错误台账。

## 6. 边界与置信度

- 全文 Degraded（纯静态）；未运行任何命令；status=draft；candidate 需 E-cD-1/E-cD-2 数据。
- vanilla 口径声明（§9.7 可比性）：dripstone_caves 无树的判据 = 项目知识 + lush_caves 一致性佐证，未直接读 vanilla jar（沙箱无 shell）——E-cD-1 为强制前置。
- populationSeed 域未触碰；c-B 核实全程只读。
