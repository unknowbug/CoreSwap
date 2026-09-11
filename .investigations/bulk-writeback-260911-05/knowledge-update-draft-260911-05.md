# 知识库更新草稿 —— C 线（bulk section 写回）260911-05 / 目录标签 -05

> **状态：draft**。本草稿**未触碰知识库任何文件**（`knowledge/`、`versions/`、`.artifacts/` 全部零改动）——它是**编辑指令集**（目标文件 / 锚点特征串 / 可直接粘贴的追加文本 / 一行理由 / 建议编号），由**主会话应用**。
> 产出者：knowledge 子角色（`core-worker` + `core-knowledge` 作操作手册，subagent 隔离执行）。
> 产出时间：2026-09-11 22:17（Get-Date 锚）。工作块：C 线（实际 2026-09-11 20:56–22:1x）。
> 置信度：**本稿全部条目 candidate 级**（有证据）；**confirmed 只有人类能授予，本稿不自标**。
> 项目级规范：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（五段式 / 载体映射 / 价值门，优先级高于 core-knowledge 通用格式）+ `knowledge/INDEX.md`。
>
> **一手输入（全部实读，非转述）**
> - `.investigations/bulk-writeback-260911-05/record-260911-05.md`（§1 摘要 / §2 实现 / §3 E1+E2 / §4 Tier 1-4 / §5 Tier 3 降级 / §6 R8/R9 / §7 遗留 / §9 证据清单）
> - `.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`；`.artifacts/index.yaml:1362-1389`（已登记 candidate）
> - `plan-260911-05.md`（§0 修订记录 R1-R4）、`api-probe-260911-05.md`、`verify-design-draft-260911-05.md`、`scout-map.md`（旁证）
> - 源码：`versions/1.20.1/java/src/main/java/wg/bench/BulkWb.java`、`.../mixin/ChunkSectionAccessor.java`、`.../CppBridge.java`（引用处）
> - 证据：`evidence/{bulk-summaries.txt, tier3-diff.txt, tier3-slice.txt, MANIFEST-sha256.txt, cmp_wb.py, wbc-*.txt, raw-*.log}`
> - vanilla 一手源（本稿逐点实读）：`.tmp/scout-260905-08/mcsrc/` 下 `ChunkSection.java`、`PalettedContainer.java`、`PacketByteBuf.java`、`PackedIntegerArray.java`、`{Singular,Array,BiMap,IdList}Palette.java`、`LockHelper.java`、`NoiseChunkGenerator.java`、`ChunkSerializer.java`
> - 目标文件现状（追加锚点前实读）：`knowledge/discovered/{workflow-patterns,algorithm-fingerprints,compiler-idioms}.md` 末尾、`knowledge/INDEX.md` 末尾、`versions/1.20.1/docs/07-block-pipeline.md`（坑 2 段 + 文件末尾）、`versions/1.20.1/docs/10-timewise-archive.md` 末尾
> - **judge 审查（`review-260911-05.md`）截至本稿产出时不存在**（已核：目录内无 `review*` 文件）⇒ 不等它，依赖 judge 的条目见 **§7**。

---

## §0 价值门判定表

判定标准 = **只记「若再遇到，我不想重新想一遍」的东西**（SUBAGENT-KNOWLEDGE-GUIDE §〇）。

| # | 内容 | 价值门 | 处置 |
|---|---|---|---|
| 1 | **E1**：`bits==0` 单态 section 不得构造 `PackedIntegerArray`（IAE 539 次）+ 判据（位宽 0 = 无 storage；复刻 MC switch 逐 case 过构造器前置条件） | **高价值（必记）** | 五段式进**独立错误台账**（新建）+ 指纹/判据进 `algorithm-fingerprints` #22 |
| 2 | **E2**：`calculateCounts()` ≠ 增量 `setBlockState` 的三计数语义（流体重复计入；流体计数只计有随机刻的流体）+ 三计数不入存档 ⇒ 唯一可见面 = 内存态 + 判据（两套实现逐条对表；「构造器会自动算好」是危险直觉） | **高价值（必记，本块最高）** | 五段式进**错误台账** + 机制/判据进 `algorithm-fingerprints` #23 |
| 3 | **MC 编码契约指纹**：`readPacket` 逐字节契约 + `BLOCK_STATE` 位宽映射（0/1-4/5-8/≥9）+ `readLongArray` 长度不符**静默丢弃** + `computeIndex` 零布局转换 | **中价值（简记）** | `algorithm-fingerprints` 新增 **#22 简记** |
| 4 | **锁语义指纹（R9）**：`readPacket` 自带一对锁；批量导入 = 锁粒度更粗（每被替换 section 一对 ≈8.0/调用 vs 老路径每非空气块一对）+ 私有容器单次引用发布 ⇒ 无部分写窗口 | **中价值（简记）**，判据部分高价值 | **不新开号** → `compiler-idioms` **#24 补充案例**（#24 已是该代码点的锁语义主条） |
| 5 | **验证方法论（本块最易被低估）**：「零差」判据的可达性预检 + **覆盖集切片归因法**（未覆盖集也有差分 ⇒ 差分来源在改动之外）+ 口径外溢登记 | **高价值（必记）** | `workflow-patterns` 新增 **#129（最高价值·错误优先）** |
| 6 | **计划修订 R1/R2**：public 构造器 ≠ 可调用（参数类型包私有 record）；「不需要 X」的否定断言必须附验证方式 | **高价值（必记，判据）** | **不新开号** → `workflow-patterns` **#25 补充案例**（#25 = 静态调研结论失真） |
| 7 | **计划修订 R3**（Tier 3 判据不可达） | 高价值 | 已并入 #129（判据可达性预检）；**不另开号** |
| 8 | **计划修订 R4**（收益定位下调） | **低价值（不记）**：一次性收益定位；可复用部分（收益预期绑定实测天花板 / 噪声带内不主张）已被 #83/#51/#103 覆盖 | 不写 discovered；只进 docs 与时间线 |
| 9 | **合成自检覆盖生产不可达分支**（ID_LIST） | **高价值（判据）** | **不新开号** → `workflow-patterns` **#110 补充案例**（#110 = 不可达路径证明的对偶面） |
| 10 | **等价门必须与被测变更同层**（`[WG-CONTENT]` buf 层门对 Java 消费侧变更结构性不敏感） | **高价值（判据）** | **不新开号** → `workflow-patterns` **#14 补充案例**（#14 = 探针同源性，本条为数据层形态） |
| 11 | C 线结论本体（D1 落地 / Tier 1-2 零差 / 三维度 / 性能分项 / R8-R9 论证 / 遗留） | **中价值**（结论性 docs **不是**知识库核心资产，但项目主题篇承载「对 1.20.1 的验证结论」） | `versions/1.20.1/docs/07-block-pipeline.md` 追加小节 + `10-timewise-archive.md` 追加块 |
| 12 | 全空气短路、ThreadLocal epoch/stamp 去重技巧、`MAX_ID=4096` 约束、scout-map/verify-design 细节 | **低价值（不记）**（实现细节/自推可得/沿用老路径） | 不写知识库（留在 `.investigations/` 过程产物） |
| 13 | 本块对齐状态快照（"C 已落地、待 judge"） | **低价值（不记）** | 只进 `.artifacts/index.yaml`（已登记）+ 时间线状态标注 |
| 14 | 证据缺口发现：`cmp_wb.py` 运行输出未归档 | **高价值（流程）** | 本稿 §3 开放项 + §5 复核记录（不写知识库条目，属本块证据补全动作） |

---

## §1 建议新增/追加条目

> 每条给出：目标文件 / 锚点特征串 / 追加文本（可直接粘贴）/ 一行理由 / 建议编号（附冲突核对）。
> **编号冲突核对（grep 实查，2026-09-11）**：`workflow-patterns.md` 现最高 **#128**；`build-tooling.md` 现最高 **#58**；`algorithm-fingerprints.md` 现最高 **#21**；`compiler-idioms.md` 现最高 **#24**。本稿建议编号 = workflow-patterns **#129**、algorithm-fingerprints **#22 / #23**、compiler-idioms 无新号（#24 补充案例）、build-tooling **零新增** ⇒ **无冲突**。

---

### 条目 1：错误台账（E1 + E2 五段式 + 速查表）

- **目标文件（新建）**：`.investigations/bulk-writeback-260911-05/errors-260911-05.md`
  （命名对齐最近先例 `perf-reg-260910-06/errors-260910-06.md`；备选名 `bulk-writeback-260911-05-errors.md`——见 §3 开放项）
- **锚点特征串**：文件不存在 ⇒ **新建全文**（不触碰 `record-260911-05.md` §3 原文——原文保留，台账为独立成篇的正式载体，符合 SUBAGENT-KNOWLEDGE-GUIDE §三「错误台账独立成篇，不与正确结论混写」）
- **理由**：E1/E2 是本块最高价值资产（错误优先原则）；项目级指定载体 = `.investigations/<课题>/<课题>-errors.md`，需五段式 + 末尾速查表。
- **建议编号**：沿用主记录 §3 的 **E1 / E2**（同一事件不另编号，避免双编号）。

**追加文本（可直接粘贴，新建文件全文）**

```markdown
# C 线（bulk section 写回）错误台账 —— 260911-05

> 载体：CoreSwap 项目级指定错误台账（`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md` §三），独立成篇、不与正确结论混写。
> 编号沿用主记录 `.investigations/bulk-writeback-260911-05/record-260911-05.md` §3 的 **E1 / E2**（同一事件不另编号）。
> 状态：E1 = resolved（已修复并验证）；E2 = resolved（实施期发现，已按增量语义复刻）。**均为 candidate 级**（confirmed 留人类）。
> 一手证据：`record-260911-05.md` §3 + `evidence/raw-c-{old,bulk}.log` + `evidence/bulk-summaries.txt` + `MANIFEST-sha256.txt`。

## E1：bits==0 的单态 section 不得构造 PackedIntegerArray（首跑 539 次 IAE）

**优先级**：P1（阻塞：新臂写回调用 607 → 68，首跑即大面积失败）
**状态**：resolved
**发现时间**：2026-09-11（260911-05 首轮 A/B）
**发现者**：主会话（实施首跑）
**module**：swe（Java 侧写回）
**来源定位**：`versions/1.20.1/java/src/main/java/wg/bench/BulkWb.java`（`bits==0` 独立分支 / `buildContainer` ③④段）；vanilla 锚 = `PalettedContainer.java:398-404`（`DataProvider.createData`：`bits == 0 ? new EmptyPaletteStorage(size) : new PackedIntegerArray(bits, size)`）+ `PackedIntegerArray.java:257`（`Validate.inclusiveBetween(1L, 32L, (long)elementBits)`）

### 现象
- 首轮 A/B 两臂 `[WG-CONTENT-WB]` 行数 **607（旧）vs 68（新）**；新臂日志 539 条
  `java.lang.IllegalArgumentException: The value 0 is not in the specified inclusive range of 1 to 32`（`[CppBridge] DIAG write threw`）。
- 影响：新臂 539/607 次写回调用直接抛异常 ⇒ 等价性门无法开（**结构性失败**，不是数值差）。

### 根因（机制层）
- `bits==0`（section 内只有 1 种状态）时 provider = `SINGULAR`、storage = `EmptyPaletteStorage`（**0 个 long**）——该分支**根本没有 storage 对象**，不需要打包。
- 实现无条件构造 `new PackedIntegerArray(storageBits=0, 4096)`，撞上构造器前置校验 `Validate.inclusiveBetween(1, 32, elementBits)` ⇒ IAE。
- 机制命名：**「位宽 0」在 MC 里是「无 storage」这一状态标记，不是「0 位宽的 storage」**；把 0 当普通位宽喂给位打包器，等于对「不存在的对象」调构造器。

### 定位（怎么发现的）
- 比对脚本先暴露**两臂 chunk 集不对称**（607 vs 68）这一**结构性异常**（`cmp_wb.py` 的 `only_old` / `only_bulk` 列），再翻日志拿异常原文。
- 顺序判据：**先看集合差，再看异常原文**——集合差直接指出「有一整类 section 走了异常分支」，日志原文只给「哪个参数非法」。

### 修复
- `bits==0` 走独立分支：`pb.writeVarInt(stateRawId(tl.palRaw[0]))` + `pb.writeLongArray(EMPTY_LONGS)`（0 个 long），**完全不构造打包数组**。
- 修后 607/607 调用，Tier 1/2 逐 chunk 差 0（三维持）。

### 教训（可复用判错经验）
1. **按位宽/枚举值选分支时，0 值必须单独定义语义**（0 = 无对象 / 无 storage / 无索引），不能当普通取值走通用路径。
2. **复刻 MC 的 switch 必须逐 case 过一遍目标构造器的前置条件**——本例 provider switch 的第 0 支没有 storage 对象，其余支才有 `PackedIntegerArray`。
3. **「两臂结果集规模不对称」是结构性失败的第一信号**（先于数值差），比逐行读日志更早定位到「一整类输入走了异常分支」。

## E2：ChunkSection.calculateCounts() 与增量 setBlockState 的计数语义不一致（会静默改变世界行为）

**优先级**：P0（最高——正确性风险：走构造器会静默改变生成期派生状态，且 region 对拍看不到）
**状态**：resolved（实施期发现，已按增量语义复刻；原构造器路线废弃）
**发现时间**：2026-09-11（260911-05 实施期）
**发现者**：主会话（为「能否绕过 72.5µs/section」读 `calculateCounts` 源码时发现）
**module**：swe / re-code（vanilla 语义核对）
**来源定位**：`ChunkSection.java:27-31`（构造器自动 `calculateCounts()`）/ `:60-93`（增量 `setBlockState`）/ `:111-140`（`calculateCounts`）；`NoiseChunkGenerator.java:416`（生成期走增量路径 `setBlockState(..., lock=false)`）；`ChunkSerializer.java:310-311`（三计数不入存档）

### 现象
- 起点是**性能问题**：`ChunkSection(pc, biome)` 构造器自动调 `calculateCounts()`，实测 **72,480 ns/section**，占被替换 section 成本的 **64%**（24 section/chunk）。
- 为「能否绕过这 72.5µs」去读 `calculateCounts` 源码 → 发现**语义差异**：**性能问题的解法顺带暴露了正确性问题**。

### 根因（机制层）
同一「section 计数」语义在 vanilla 内部有**两套实现**，逐条对表：

| 计数 | 增量 `setBlockState`（:70-90） | `calculateCounts`（:111-140） |
|---|---|---|
| `nonEmptyBlockCount` | 非空气 +1 | 非空气 +1，**且流体非空再 +1**（同一格重复计入） |
| `randomTickableBlockCount` | 非空气 ∧ `hasRandomTicks` | 同 |
| `nonEmptyFluidCount` | **流体非空** +1 | 仅**流体自身 `hasRandomTicks()`** 才 +1（静水 `WaterFluid.Still.hasRandomTicks()=false` ⇒ 静水 section 该计数 = 0，而增量路径 = 4096） |

- vanilla `populateNoise` 走**增量**路径（`:416`）⇒ **增量语义才是生成期 vanilla 语义**（也是老逐块路径语义）。
- 走构造器（全量重算）会让 `hasRandomFluidTicks()`（→ 流体随机刻调度）、`isEmpty()` / `nonEmptyBlockCount`（→ 客户端包 `toPacket`）偏离。
- 关键放大：**三个计数不入存档**（`ChunkSerializer.java:310-311` 只写 `block_states` / `biomes`）⇒ region 对拍（Tier 3）**看不到**它们；唯一可见面 = 生成期到下次存读之间的**内存态**。

### 定位（怎么发现的）
- 诊断方法 = **一手源逐条对表**（把两套实现的三个计数逐行并排），不是跑差分。
- 触发链 = 性能优化想绕过 `calculateCounts` → 读它 → 发现它算的是**另一套定义**。
- 验证 = Tier 2 派生字段指纹（`dh=` = 三计数逐 section 序列）逐 chunk 全等（差 0）；若走构造器路线，该门能直接检出「状态层等价但派生层不等价」形态。

### 修复
- 弃用构造器路线；新增 `ChunkSectionAccessor` mixin（4 写 + 3 读 `@Accessor`，`@Mutable` 用于 `final blockStateContainer`），**原地改旧 section 对象**：
  - 只换 `blockStateContainer`（生物群系容器原样保留，不新建 section、不调 `calculateCounts`）；
  - 三个计数在去重遍顺带算（O(distinct)），**复刻增量语义**。
- 副作用（顺带收益）：`counts_set` 由 72,480 ns/section → **116 ns/section**。

### 教训（可复用判错经验）
1. **同一语义在 vanilla 内部有两套实现时 MUST 逐条对表**（增量维护 vs 全量重算）；「哪一套是生成期语义」由**调用点**决定，不由「哪个 API 更现成」决定。
2. **「构造器会自动算好」是危险的直觉**——它算的是**另一套定义**；凡「顺手用现成 API 代替手工维护」，先核对两套实现是否定义等价。
3. **等价门必须分层**——状态层指纹（方块 id 序列）对「派生字段语义差」**结构性不敏感**；派生字段（计数序列）必须**独立成门**。本案正是先看到「状态层等价、派生层可疑」才去查语义。
4. **「不入存档的派生状态」是验证盲区**：凡判定某派生字段「不入存档」，MUST 同时声明其**唯一可见面**（本案 = 生成期内存态 → `hasRandomFluidTicks()` 调度 / `isEmpty()` / 客户端包），否则会被误读成「region 对拍过了就等于没问题」。

## 速查表（错误 → 现象签名 → 根因 → 可复用判据）

| 错误 | 现象签名 | 根因（机制） | 可复用判据 |
|---|---|---|---|
| E1 | 两臂**结果集规模不对称**（607 vs 68）+ `Validate ... 1 to 32` IAE | 位宽 0 = **无 storage**（`EmptyPaletteStorage`），被当成「0 位 storage」喂给 `PackedIntegerArray` 构造器 | ① 位宽/枚举 0 值必须单独定义语义；② 复刻 MC switch 逐 case 过构造器前置条件；③ 先看集合差再看异常原文 |
| E2 | 性能问题的解法顺带暴露语义差；状态层等价但派生层不等价 | `calculateCounts`（全量重算）与增量 `setBlockState` 的**三计数定义不同**（流体重复计入 block 计数；流体计数只计有随机刻的流体）；三计数**不入存档** ⇒ region 门看不见 | ① 同一语义两套实现 MUST 逐条对表，生成期语义由调用点决定；② 「构造器会自动算好」是危险直觉；③ 等价门必须分层；④ 不入存档的派生字段必须声明唯一可见面 |
```

---

### 条目 2：`algorithm-fingerprints` 新增 #22（编码契约指纹）

- **目标文件**：`knowledge/discovered/algorithm-fingerprints.md`
- **锚点特征串**（文件末尾，唯一）：`取代链：` + "`record-260911-05.md`" + ` §3/§4/§5 **S4**。`
- **理由**：MC 序列化编码契约 = 中价值算法/协议指纹；任何后续要碰 `PalettedContainer` 的工作（含 1.21.6 收敛、共享 Java 适配核）都要用；且 E1 的判据（位宽 0 = 无 storage）必须与契约同处一页。
- **建议编号**：**#22**（现最高 #21 ⇒ 无冲突）。

**追加文本（可直接粘贴）**

```markdown
---

## 发现 #22 简记: MC 1.20.1 `PalettedContainer.readPacket` 逐字节编码契约 + `BLOCK_STATE` 位宽映射指纹（含 `bits==0` 无 storage）（260911-05）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线 API 前置探针 + 实施）；**candidate**；algorithm-fingerprints / MC 序列化编码契约。
- **来源定位**：vanilla 一手源 `.tmp/scout-260905-08/mcsrc/`：`PalettedContainer.java:203-215`（`readPacket`）/ `:398-404`（`record DataProvider` 包私有 + `bits == 0 ? EmptyPaletteStorage : PackedIntegerArray`）/ `:420-430`（`BLOCK_STATE` 的 switch）/ `:465-467`（`computeIndex`）；`PacketByteBuf.java:859-867`（`writeLongArray` 带 VarInt 长度前缀）/ `:924-932`（`readLongArray` 仅在长度相符时复用传入数组）；`SingularPalette.java:67-68`、`ArrayPalette.java:85-90`、`BiMapPalette.java:74-79`、`IdListPalette.java:45-46`（空实现）；`PackedIntegerArray.java:257`（`Validate 1..32`）/ `:261`（`elementsPerLong = 64/bits`）/ `:266`（`longs = ceil(size/elementsPerLong)`）。一手产物 `.investigations/bulk-writeback-260911-05/api-probe-260911-05.md` §2/§3。
- **观察（逐字节契约）**：`readPacket(buf)` = `byte 请求位宽 i`（`:207`）→ `getCompatibleData(prev, i)`（按 i 选 provider）→ `palette.readPacket(buf)` → `buf.readLongArray(storage.getData())`（`:210`）→ 换 `this.data`。

  | 请求位宽 i | provider | palette 段字节 | storage 位宽 | storage 值域 |
  |---|---|---|---|---|
  | 0 | SINGULAR | `VarInt rawStateId` | 0（`EmptyPaletteStorage`，**无 storage 对象**） | —（写 0 个 long） |
  | 1-4 | ARRAY | `VarInt size` + size × `VarInt rawStateId` | **恒 4** | 局部 palette 索引 |
  | 5-8 | BI_MAP | 同上 | = i | 局部 palette 索引 |
  | ≥9 | ID_LIST | **无字节**（空实现） | `ceilLog2(idList.size())` | **全局 state id**（`idList.getRawId`） |

  - 实测自证（overworld/nether/end 三维一致）：`state_ids_size=24137`、`idlist_bits=15`、`max_distinct` = 7 / 7 / 2、`idlist_hits=0`。
  - **零布局转换**：`computeIndex(x,y,z) = (y<<4|z)<<4|x` 与 Rust buf 的 y-major 切片布局逐位相同 ⇒ section s 恰是连续切片 `buf[s*4096,(s+1)*4096)`。
  - ⚠️ **`readLongArray` 的静默丢弃**：`readPacket` **丢弃返回值**；`readLongArray(toArray)` 仅在 `toArray.length == VarInt 长度` 时复用传入数组，否则**新建数组并返回**（读进来的值被丢掉，storage 保持原值）——**长度不符不抛异常、不报错**（对比 `PackedIntegerArray(elementBits,size,long[])` 构造器在长度不符时抛 `InvalidLengthException`）。⇒ 自建 buf 时 storage long 数必须精确等于 `ceil(size/(64/bits))`（**推导值**：4bit → 256 longs、15bit → 1024 longs）。
- **如何利用（判据）**：
  1. 要**整段导入** `PalettedContainer` 时用公开 `readPacket`（自建 buffer），不必逐点 `set`；但 MUST 自复刻 `BLOCK_STATE` 的 switch（provider 类型**包私有**，包外不可命名）。
  2. **位宽 0 = 无 storage，不是「0 位 storage」**——不得对 0 位构造 `PackedIntegerArray`（E1：`Validate 1..32` 抛 IAE，首跑 539 次）。
  3. 复刻 MC 的 switch **逐 case 过目标构造器前置条件**（第 0 支没有 storage 对象）。
  4. 手写该 buffer 的**长度字段必须按契约对表**（palette 段有无、VarInt 前缀），错一位即「静默丢数据 + 无异常」（与 build-tooling #58 同族：单点长度笔误 → 结构级失真）。
- **家族索引**：algorithm-fingerprints #23（同案计数语义差）、compiler-idioms #6（raw id vs state id 双域——本契约 ≥9 支走 state id 域）、compiler-idioms #24 补充案例（`readPacket` 自带锁）、build-tooling #58（长度字段对表）。
```

---

### 条目 3：`algorithm-fingerprints` 新增 #23（三计数两套语义）

- **目标文件**：`knowledge/discovered/algorithm-fingerprints.md`
- **锚点特征串**：条目 2 的**同一锚点**（文件末尾 `取代链：` + "`record-260911-05.md`" + ` §3/§4/§5 **S4**。`）——**应用顺序：条目 2 先贴，再以条目 2 末尾的 `- **家族索引**：algorithm-fingerprints #23…` 行作条目 3 的锚点**（或直接连续追加两条）。
- **理由**：E2 的机制层指纹（同一语义两套实现）+ 判据是**本块最高价值**；错误链条进台账，机制/判据进 discovered 供跨版本复用（1.21.6 的 `ChunkSection` 同类风险）。
- **建议编号**：**#23**（现最高 #21，加上条目 2 的 #22 ⇒ 无冲突）。

**追加文本（可直接粘贴）**

```markdown
---

## 发现 #23: `ChunkSection` 三个派生计数在 vanilla 内部有**两套语义**——`calculateCounts()`（全量重算）≠ 增量 `setBlockState`，且三计数**不入存档**（260911-05，最高价值·错误优先）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线实施期，为绕过 72.5µs/section 读 `calculateCounts` 源码时发现）；**candidate**（Tier 2 逐 chunk 全等 + 一手源对表；confirmed 留人类）；algorithm-fingerprints / MC section 派生状态语义（完整错误链见 `.investigations/bulk-writeback-260911-05/errors-260911-05.md` E2）。
- **来源定位**：`ChunkSection.java:27-31`（构造器自动 `calculateCounts()`）/ `:60-93`（增量 `setBlockState`）/ `:111-140`（`calculateCounts`）；`NoiseChunkGenerator.java:416`（生成期走增量、`lock=false`）；`ChunkSerializer.java:310-311`（只写 `block_states`/`biomes`）；实现 `BulkWb.buildContainer` ②段 + `ChunkSectionAccessor`（4 写 + 3 读）。
- **观察（逐条对表）**：

  | 计数 | 增量 `setBlockState` | `calculateCounts` |
  |---|---|---|
  | `nonEmptyBlockCount` | 非空气 +1 | 非空气 +1 **且流体非空再 +1**（同格重复计入） |
  | `randomTickableBlockCount` | 非空气 ∧ `hasRandomTicks` | 同 |
  | `nonEmptyFluidCount` | **流体非空** +1 | 仅**流体自身 `hasRandomTicks()`**（静水 `WaterFluid.Still.hasRandomTicks()=false` ⇒ 静水 section = 0，增量 = 4096） |

  - 实测成本：构造器路线 `calculateCounts` = **72,480 ns/section**（占被替换 section 成本 64%）；自算（去重遍顺带，O(distinct)）= **116 ns/section**。
  - 生成期语义由**调用点**决定：vanilla `populateNoise` 走增量 ⇒ 增量语义才是生成期 vanilla 语义（也是老逐块路径语义）。
  - **三计数不入存档** ⇒ region 对拍看不到；唯一可见面 = 生成期到下次存读之间的**内存态**：`hasRandomFluidTicks()`（→ 流体随机刻调度）、`isEmpty()` / `nonEmptyBlockCount`（→ 客户端包 `toPacket`）。
  - 等价性验证：Tier 2 派生字段指纹（`dh=` = 三计数逐 section 序列）**607/625/625 逐 chunk 差 0**（三维持；本稿独立复算自归档抽取件）。
- **如何利用（判据）**：
  1. **同一语义在 vanilla 内部有两套实现时 MUST 逐条对表**；生成期语义按**调用点**定，不按「哪个 API 更现成」定。
  2. **「构造器会自动算好」是危险直觉**——它算的是另一套定义（走 `new ChunkSection(pc,biome)` 会静默改变 `hasRandomFluidTicks()` / 客户端包语义）。
  3. **等价门必须分层**：状态层指纹（方块 id 序列）对派生字段语义差**结构性不敏感** ⇒ 派生字段必须**独立成门**。
  4. 判定某派生字段「不入存档」时 MUST 同时声明其**唯一可见面**，否则会被误读成「region 对拍过了 = 没问题」。
- **家族索引**：compiler-idioms #24 补充案例（同 section 的锁语义）、algorithm-fingerprints #22（同案编码契约）、workflow-patterns #129（本案 Tier 3 判据降级）、workflow-patterns #14 补充案例（层不匹配的门零判别力）。
```

---

### 条目 4：`compiler-idioms` #24 补充案例（锁粒度 / 发布原子性）

- **目标文件**：`knowledge/discovered/compiler-idioms.md`
- **锚点特征串**（文件末尾，唯一）：`- **家族索引**：workflow-patterns #113（主判据与错误链）、compiler-idioms #11（诊断门控在初始化器内唯一置位——同为「初始化/持锁期语义」类简条）、#12（mixin 包约束——同为「平台语义约束」简条）。`
- **理由**：#24 主条已覆盖「`swap()` 自带锁 + 外层段锁成对契约」，**本块新增的是量化面与发布原子性判据**（同一代码点，按去重纪律**不新开号**）。
- **建议编号**：**#24 补充案例**（不新开号；compiler-idioms 现最高 #24 ⇒ 无冲突）。

**追加文本（可直接粘贴）**

```markdown
---

### 发现 #24 补充案例（260911-05）：批量导入共享容器的**锁粒度与发布原子性**——`readPacket` 自带一对锁，且填私有容器后单次引用发布

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线 R9 并发/锁论证）；**candidate**；compiler-idioms / 锁语义（**#24 主条在「批量写」形态下的量化面**）。
- **来源定位**：`PalettedContainer.java:203-215`（`readPacket` = `lock()` / `try{…} finally{ unlock(); }`，`:204`/`:213`）；`ChunkSection.java:48-54`（lock/unlock 转发）；`NoiseChunkGenerator.java:342-346`（生成 range 内全 section 上锁）/ `:416`（持锁期 `setBlockState(..., lock=false)` = `swapUnsafe`）/ `:351-355`（统一解锁）；`LockHelper.java:20-21`/`:31-71`（`Semaphore(1)` + `ReentrantLock`，争用即 crash「Accessing … from multiple threads」）；实现 `BulkWb`（每 section 一次 `readPacket`）+ `ChunkSectionAccessor`（原地换容器 + 直写三计数）。
- **观察（锁粒度量化）**：
  - 老 CoreSwap 逐块路径：mixin 在 `populateNoise` HEAD cancel ⇒ vanilla 上锁段整体被跳过；随后**每非空气块**一次 `ChunkSection.setBlockState(x,y,z,st)`（4 参 = `lock=true`）⇒ **每非空气块一对 `LockHelper` 操作**（每 chunk 量级估计数万次，非实测计数），且逐块变异**共享的活容器**（存在数万个中间态可见窗口）。
  - bulk 路径：`readPacket` **自带一对锁** ⇒ 每**被替换** section 一对（实测 `sections_replaced` / `calls`：overworld 4856/607 ≈ **8.0**、nether 4815/625 ≈ **7.7**、end 562/625 ≈ **0.9**；上限 = 24/16/8 section per chunk）；且填的是**尚未发布的私有容器**，最后以**单次引用写**发布 ⇒ 并发读者只见「旧（全空气）」或「新（完整）」，**无部分写可见窗口**。
  - 自洽核对：`sections_replaced + air_sections_skipped = 14,568 = 607×24`（overworld：8.0 替换 + 16.0 空气短路 = 24）。
- **判据（可复用）**：① **批量导入共享容器 = 锁粒度更粗 + 发布更原子**（先建私有对象 → 单次引用发布）——这是**两个独立收益**，评估并发安全性时 MUST 分开陈述；② 复刻/接管类改动涉及「整段替换 vs 逐点写」时，锁语义核对表加两栏：**锁次数**（每块 vs 每段）与**发布原子性**（活容器逐点变异 vs 私有对象单次发布）；③ 判「新路径不比老路径弱」时，`readPacket` / `swap` 这类**自带锁**的接口按「一次调用一对锁」计，不得漏算成无锁。
- **残余风险（诚实声明）**：`sections[s]` 的容器引用非 `volatile`，跨线程可见性依赖既有 chunk 发布/同步机制（与老路径相同）；**未做**并发压力下的可见性专项验证（R9-b，登记为未验证项）。
- **家族索引**：compiler-idioms #24（主条——`swap()` 自带锁 + 「外层段锁 + `lock=false` 写」成对契约）、workflow-patterns #113（跨实现搬运锁语义主判据）、algorithm-fingerprints #22（`readPacket` 契约）、workflow-patterns #115（内容指纹门的边界：指纹取在写回之前，不覆盖写回/并发路径）。
```

---

### 条目 5：`workflow-patterns` 新增 #129（判据可达性 + 覆盖集切片归因）

- **目标文件**：`knowledge/discovered/workflow-patterns.md`
- **锚点特征串**（文件末尾，唯一）：`- **家族索引**：#127（口径换代家族主条）、#111（噪声锚）、#33（载具可比性）、#18（跨 session 数字）、#104（规模敏感性——同属「倍数随样本属性变」家族）。`
- **理由**：本块最易被低估的一条——「零差」判据在 live-server 载体上结构性不可达，且给出了**通用归因方法**（覆盖集切片）；同时是历史 region 对拍保真度数字的**口径外溢**登记载体。
- **建议编号**：**#129**（现最高 #128 ⇒ 无冲突）。
- **去重核对**：#51（噪声基线前置必测）与 #111（噪声锚取同配置基线）覆盖「先测噪声底」，**未覆盖**「判据阈值可达性预检」与「覆盖集切片归因法」⇒ 新开号成立；#112（判据读法预登记）为**读法**域，本条为**阈值可达性**域。

**追加文本（可直接粘贴）**

```markdown
---

## 发现 #129（最高价值·错误优先）: 「零差」类判据的可达性预检 + 覆盖集切片归因法——噪声底高于判据要求时判据结构性不可达（260911-05）

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线 Tier 3 对拍降级轮）；**candidate**（judge 待审；confirmed 留人类）；workflow-patterns / 验证判据可达性（**#51/#111 的判据可达性面**）。
- **来源定位**：`.investigations/bulk-writeback-260911-05/record-260911-05.md` §5；证据 `evidence/tier3-diff.txt`（全域 `blocks=313,589,760 diff=86,841 = 0.0277%`、`sections same/diff = 74822/1738`）+ `evidence/tier3-slice.txt`（切片 A/B）+ `evidence/tier3_slice.py`（exec 复用 B1 已修正解析器 `diff_arms_fixed.py` 前缀源码，零复制 ⇒ 口径可比）。
- **观察**：预登记判据「两臂 region 文件逐块差 = 0」在本 harness **不可达**——全域差分 0.0277%；按 `[WG-CONTENT-WB]` 的 chunk 坐标**切成两半**：
  - 切片 A（**走过** bulk 写回的 607 chunk）：20,738 / 59,670,528 = **0.0348%**
  - 切片 B（**从未走过**本代码路径的 2,583 chunk）：66,103 / 253,919,232 = **0.0260%**
  ⇒ **未覆盖集也有差分** ⇒ 残差来源在改动之外（跨 run 固有抖动），不是写回差异。
- **根因（机制）**：live-server 载体在生成期间**仍在跑随机刻/装饰**，各 run 的「机会数」不同 ⇒ 差分集中在 FEATURES 阶段产物与随机刻可改写方块，且 top 对**双向对称**（`sculk↔deepslate` 9111/6623、`kelp_plant↔water` 2944/2905、`oak_leaves↔air` 4061/3551 等）——**对称双向变化是「随机/时序抖动」签名**，单向集中变化才优先怀疑实现差异。
- **判据（MUST）**：
  1. **预登记任何「零差 / ≤阈值」判据前，先测该载体的噪声底**；噪声底 > 判据要求 ⇒ 判据**结构性不可达**，MUST 当场降级（本案 = noise-floor-limited「不可判」）并**改派到噪声无关的载体**（本案改由 Tier 1「写回后读回」指纹承载）；
  2. **归因方法 = 用被测代码路径的覆盖集切片**：若「未覆盖集」也有同量级差分 ⇒ 差分来源在改动之外，**禁止**归因给被测改动；切片工具 MUST 与主判据**同一份解析器**（否则切片与全域不可比）；
  3. 降级声明 MUST 写明「判据**不可达**」而非「差异小到可接受」——两者是不同结论；
  4. 判「噪声底」时须同时给出**全域/覆盖/未覆盖三档**，单给全域无法区分「残差来自改动」与「残差来自环境」。
- **口径外溢（登记，不改旧结论）**：本工程历史上基于 live-server region 对拍的**保真度数字**（含 B1/E5 复算的 0.0133-0.0198%、历史代理基线 0.0180%）**同受该抖动限制**，不可当**绝对**保真度读（⚠️ 严格说：0.026% 抖动底是在**本块载体/区域集**上测得，跨 region 集按 §9.7 载体要素**不作精确阈值迁移**，但量级限制成立）。这些数字作为**同仪器相对比较**的用法（如 B1 的「≤历史基线」论证）不受影响——受影响的是把绝对值读成「实现差异有多大」。
- **家族索引**：#51（噪声基线前置必测——本条补「阈值可达性」判定）、#111（噪声锚应取同配置 run-to-run）、#115（噪声无关的内容指纹门——本条为 region 层的替代载体面）、#112（判据读法预登记——本条为**阈值可达性**预登记）、#49（跨 run 完成度伪差）、algorithm-fingerprints #21（维度确定性指纹——end = 0 是「可升位级门」的对照面）。
```

---

### 条目 6：`workflow-patterns` #25 补充案例（计划期技术断言的静态失真）

- **目标文件**：`knowledge/discovered/workflow-patterns.md`
- **锚点特征串**：条目 5 追加文本末尾的 `- **家族索引**：#51（噪声基线前置必测——本条补「阈值可达性」判定）…algorithm-fingerprints #21（维度确定性指纹——end = 0 是「可升位级门」的对照面）。` 行（**应用顺序：条目 5 先贴**）；或按 §1 顺序连续追加。
- **理由**：#25 主条 = 「静态调研结论失真」，R1/R2 正是**计划文件域**的同一失真形态（判据可直接复用），按去重纪律**不新开号**。
- **建议编号**：**#25 补充案例**（不新开号；workflow-patterns 新号只有 #129 ⇒ 无冲突）。

**追加文本（可直接粘贴）**

```markdown
---

### 发现 #25 补充案例（260911-05）：计划期技术断言的静态失真两形态——「public 构造器 ≠ 可调用」与「不需要 X」的无验证断言

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线计划 §0 修订 R1/R2）；**candidate**；workflow-patterns / 计划期断言核对（**#25 主判据在「计划文件」域的形态**）。
- **来源定位**：`.investigations/bulk-writeback-260911-05/plan-260911-05.md` §0（R1/R2 修订记录）+ `api-probe-260911-05.md` §1/§6；实现侧 `BulkWb.java` 类注释 + `ChunkSectionAccessor.java`。
- **观察（两形态）**：
  1. **R1「API 可用性」**：计划 §4.2 原案用 5 参 `PalettedContainer` 构造器——构造器**本身 public**，但第 3 参类型 `PalettedContainer.DataProvider` 是**包私有 record**（`PalettedContainer.java:398`，无修饰符）⇒ 包外**无法命名该类型**，调用点写不出来。API 前置探针一轮否决，改走公开 `readPacket(PacketByteBuf)`（反而更优：storage longs 整段自建，连 per-position 容器写都省掉）。
  2. **R2「不需要 X」**：探针结论「**无需 mixin**」被实施期推翻——需要 1 个 accessor mixin，理由是**两个独立维度**：① 性能（构造器自动 `calculateCounts()` 实测 72,480 ns/section = 被替换 section 成本的 64%）② 语义（`calculateCounts` 与增量 `setBlockState` 的三计数定义不一致，见 algorithm-fingerprints #23）。而计划 §8 的子角色预置还据此写了「无需 mixin」。
- **根因（机制）**：计划期的技术断言是**静态结论**，其「真」依赖于**未被检查的维度**——R1 依赖「类型可见性」维度（读「构造器是 public」会漏掉**参数类型**的修饰符），R2 依赖「成本/语义」维度（「整段替换不需要 mixin」只覆盖机制可行性，不覆盖**成本与语义保真**）。
- **判据（可复用）**：
  1. 计划里凡出现「用 API X」的选型，MUST 把判据落到**调用点能否书写**这一编译级事实（参数类型/返回类型的**访问修饰符**一手核）；「构造器/方法本身 public」**不构成**可用性证据；
  2. 计划里凡出现「**不需要 X**」的否定断言，MUST 附**验证方式 + 触发条件**（怎么证、在什么条件下会翻）；给不出验证方式的否定断言 = 未验证假设，实施期必须重新核；
  3. 计划期「工作量/收益」估计 MUST 标注**未预见项的位置**（本案工作量从估 120-180 行 → 实际 ~400 行，低估项 = ① 计数语义复刻 ② Tier 1/2 指纹门载体 ③ 合成自检——三项在计划里都没出现）。
- **附（R4，定位下调）**：收益预期在计划期已有实测天花板（写回占 chunk 时间 ~6.2%），实施后进一步下调为「**可维护性为主、性能为次要附带**」，端到端落在 ±10% 机器噪声带内**不主张**——即计划期收益预期应绑定实测天花板，实施后按实测**下调**而非上调（噪声带判据见 #103/#51，本条不另开号）。
- **家族索引**：#25（主条——静态调研结论失真的生产路径可达性/常量源头两例）、#94（任务简报字段名/分类词不进一手源核对不可用）、#95（scout 结论的事实前提与结论分开验证）、#42（静态机制断言未实测当公理）、#110（可达性证明三件套）、#112（判据预登记）。
```

---

### 条目 7：`workflow-patterns` #110 补充案例（不可达分支合成压测）

- **目标文件**：`knowledge/discovered/workflow-patterns.md`
- **锚点特征串**：条目 6 追加文本末尾的 `- **家族索引**：#25（主条——静态调研结论失真的生产路径可达性/常量源头两例）…、#112（判据预登记）。` 行（应用顺序：条目 6 先贴）。
- **理由**：#110 主条证明「路径不可达」，本条是其**对偶面**：不可达分支仍须**合成压测**并在结论里显式声明，否则「全绿但未覆盖」。
- **建议编号**：**#110 补充案例**（不新开号）。

**追加文本（可直接粘贴）**

```markdown
---

### 发现 #110 补充案例（260911-05）：生产不可达的编码分支 MUST 用合成输入压测并显式声明——「全绿但未覆盖」

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线编码四支自检）；**candidate**；workflow-patterns / 可达性证明的**对偶面**（#110 主条证明「路径不可达」，本条要求「不可达分支仍须覆盖」）。
- **来源定位**：`.investigations/bulk-writeback-260911-05/record-260911-05.md` §4「编码分支自检」；实现 `BulkWb.selfTest()`（`-Dcoreswap.bulkwbtest=1`）；证据 `evidence/bulk-summaries.txt`（四行 `[WG-BULKWB-TEST] PASS` + `all branches PASS`）。
- **观察**：自然生成 `max_distinct` = 7（overworld）/ 7（nether）/ 2（end）、`idlist_hits=0`（三维持）⇒ 编码四支里的 **ID_LIST 支在生产数据上不可达**；若不刻意压测，该分支「全绿但未覆盖」（编译过 + 无异常 = 零证据）。做法 = 合成 buf 四例（distinct = 1 / 3 / 20 / **300**）逐位读回 4096 位置比对 `CppBridge.stateById`，四支全绿。
- **判据（可复用）**：① 凡判定某分支在生产数据上**不可达**，MUST 同时给出「合成输入覆盖」证据，并在结论里**显式声明不可达 + 已合成覆盖**（不得只写「四支全绿」而不提可达性）；② 合成例按**分支边界**取（SINGULAR / ARRAY / BI_MAP / ID_LIST 各一，末支取 >256 distinct 刚好越界），不是随便取样；③ 自检开关默认关（生产零成本），但结论引用时必须声明「该证据来自**合成自检**，不是生产观测」。
- **家族索引**：#110（主条——不可达路径证明三件套）、#52（确定性 dump 载体——同属「把不可判变可判」的手段面）、#89（结构性难采样的降级声明）。
```

---

### 条目 8：`workflow-patterns` #14 补充案例（等价门必须同层）

- **目标文件**：`knowledge/discovered/workflow-patterns.md`
- **锚点特征串**：条目 7 追加文本末尾的 `- **家族索引**：#110（主条——不可达路径证明三件套）、#52（确定性 dump 载体——同属「把不可判变可判」的手段面）、#89（结构性难采样的降级声明）。` 行（应用顺序：条目 7 先贴）。
- **理由**：#14 主条 = 探针**阶段**同源性；本条是**数据层**同源性（buf 层门对 Java 消费侧变更零判别力），判据可复用且计划 §8 已预置该条目。
- **建议编号**：**#14 补充案例**（不新开号）。

**追加文本（可直接粘贴）**

```markdown
---

### 发现 #14 补充案例（260911-05）：等价门必须与被测变更**同层**——`[WG-CONTENT]`（Rust buf 层指纹）对 Java 侧写回变更结构性不敏感

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线验证设计）；**candidate**；workflow-patterns / 探针同源性（#14 主条为「**阶段**同源」，本条为「**数据层**同源」）。
- **来源定位**：`.investigations/bulk-writeback-260911-05/verify-design-draft-260911-05.md` §3 + `plan-260911-05.md` §2.2；实现锚 = `CppBridge` 的 `[WG-CONTENT]`（hash **Rust 输出的 buf**）与 `BulkWb.readbackHash`（hash **写回后** section 读回）。
- **观察**：C 的变量在 **Java 侧消费**（写回机制），而 `[WG-CONTENT]` 指纹算的是 **Rust 输出的 `buf`**——buf 不变则 hash **必然**相同 ⇒ 该门对本次变更**不可能**检出回归（1.21.6 侧该门还整体缺失）。C 的等价性只能由「写回**之后**读回」的 Tier 1 门承载。
- **判据（可复用）**：① 设计/引用行为门时先问「**被测变量在哪一层**」——门观测的数据层必须覆盖被测变更所在的层，否则该门对该变更**零判别力**（「门全绿」不构成证据）；② 同一载体里可以有多层指纹，**层名必须写进结论**（本案 `[WG-CONTENT]` = buf 层 / `[WG-CONTENT-WB]` = 写回后读回层），避免下游把 buf 层绿读成消费层绿；③ 与 #36（执行体同源）互补：**执行体同源 ≠ 数据层同源**。
- **家族索引**：#14（主条——探针阶段同源性）、#36（验证探针与生产执行体不同源）、#105（载体偏差——数据通路）、#46（跨实现缺键语义不对称）、#115（内容指纹门的取点边界）。
```

---

### 条目 9：`versions/1.20.1/docs/07-block-pipeline.md` 追加小节（+ 坑 2 补充指针）

- **目标文件**：`versions/1.20.1/docs/07-block-pipeline.md`
- **锚点特征串**：
  - (a) 文件末尾（唯一）：`（237,41,224)）——永久挂起，归因保持 candidate、坐标不删；光照/流体课题可引用为已知差异源；多世界新 seed 流下此族为残差放大候选（详注见 11 篇）。`
  - (b) 坑 2 段末尾（唯一）：`- 不要反射改计数（运行时字段是混淆名，且 setBlockState 更干净）`
- **理由**：07 篇主题 = 块级流水线 + 「Java 侧写入路径（`CppBridge.fillChunk` / `writeChunk`）的坑」，C 线正是该路径的机制升级 ⇒ **主题篇归口正确**（不散落到 09 篇；三维度覆盖写在 07 表内即可）。坑 2 的「不要反射改计数」需**追加限定指针**（只增不删，不构成取代）。
- **建议编号**：docs 无编号；小节标题自带日期+块标签。

**追加文本 A（可直接粘贴到文件末尾）**

```markdown

## 2026-09-11 C 线：Java 侧 bulk section 写回（D1，Rust 零改动）— candidate（judge 待审 / confirmed 留用户）

> 载体与依据：`.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`（candidate）+ `.investigations/bulk-writeback-260911-05/record-260911-05.md`（§3 两个错误 / §4 Tier 1-4 / §5 Tier 3 降级 / §6 R8/R9）+ `plan-260911-05.md` §0（修订 R1-R4）+ `evidence/`（MANIFEST sha 清单）。提交 `8dd9e71`。通用模式 → knowledge/discovered：algorithm-fingerprints #22/#23、compiler-idioms #24 补充案例、workflow-patterns #129 / #25 补充案例 / #110 补充案例 / #14 补充案例（subagent 草稿 → 主会话应用）。

### 形态（D1 落地）
- 把「Rust 填好的扁平 `int[]`（y-major raw block id）」一次性转成 section 级 `PalettedContainer`，取代**每 chunk 98,304 次 `ChunkSection.setBlockState`**；调用点与时机不变（NOISE 阶段接管点内、高度图之前），**Rust/JNI/数据驱动边界零改动**。
- 路线 = 公开 `PalettedContainer.readPacket(PacketByteBuf)`（自建 buffer；原案 5 参构造器因第 3 参 `DataProvider` 是包私有 record 而不可调用）+ `ChunkSectionAccessor` mixin（4 写 + 3 读）**原地换容器 + 直写三计数**（不新建 section、不调 `calculateCounts`、生物群系容器原样保留）。
- 回退开关 `-Dcoreswap.bulkwb=0`（旧逐块路径**保留不删**，即时降级 + A/B 单变量）；诊断 `-Dcoreswap.bulkwblog` / `-Dcoreswap.wbcontent` / `-Dcoreswap.bulkwbtest`（默认全关，生产零成本）。

### 等价性（Full，噪声无关；§9.7 三要素）
- **载体**：同 JVM 内、写回**之后**读回全部 section 算 FNV-1a 64（`[WG-CONTENT-WB] hash=`）+ 三计数序列指纹（`dh=`）。
- **覆盖面**：写回调用 607（overworld）/ 625（nether）/ 625（end），各 24/16/8 section × 4096 位置（overworld = 59,670,528 位置）。
- **可比性**：同构建态、同 dll（`838e89794a54e19d`）、同 seed/区域、背靠背两臂，**唯一变量 = `-Dcoreswap.bulkwb`**。
- **结果**：Tier 1（状态层）与 Tier 2（派生层）**逐 chunk 差均为 0**，两臂 chunk 集相同（`only_old=0` / `only_bulk=0`）、零异常；编码四支 SINGULAR/ARRAY/BI_MAP/**ID_LIST** 合成自检逐位读回全绿（自然生成 `max_distinct=7` / `idlist_hits=0` ⇒ ID_LIST 生产不可达，必须刻意压测）。
- ⚠️ `[WG-CONTENT]`（Rust **buf** 层指纹）**与本门不同层**：buf 不变则 hash 必相同 ⇒ 对 Java 侧消费的变更**不敏感**，C 的等价性只能由本门承载（判据 → workflow-patterns #14 补充案例）。

### 两个实施期错误（最高价值，五段式见错误台账）
1. **E1**：`bits==0` 的单态 section **不得构造 `PackedIntegerArray`**（`Validate 1..32` 抛 IAE，首跑 539 次）——位宽 0 = **无 storage**，不是「0 位 storage」；复刻 MC 的 switch 必须逐 case 过构造器前置条件。
2. **E2**：`calculateCounts()` 与增量 `setBlockState` 的**计数语义不一致**（流体重复计入 `nonEmptyBlockCount`；`nonEmptyFluidCount` 只计有随机刻的流体）⇒ 走构造器会**静默改变** `hasRandomFluidTicks()` / 客户端包语义；已改为复刻**增量**语义（生成期 vanilla 语义由调用点决定）。三计数**不入存档**（`ChunkSerializer:310-311`）⇒ region 对拍看不到，唯一可见面 = 生成期内存态。

### 性能（Partial：分项计时，跨 run 只作趋势）

| 维度 | 旧逐块 ns/chunk | bulk ns/chunk | 变化 |
|---|---|---|---|
| overworld | 2,151,591 | **652,202** | −69.7% |
| nether | 2,657,477 | **859,626** | −67.7% |
| end | 2,809,444 | **804,970** | −71.4% |

- 分项（定稿轮）：`build=75,666 ns/section`（含 `readPacket=5,963`）、`counts_set=116 ns/section`（原 `calculateCounts` 72,480 ns/section）；`sections_replaced=4856` / `air_sections_skipped=9712`（= 607×24）。
- ⚠️ **价值定位**：**可维护性/可移植性为主**（消掉逐块循环与 per-version 诊断，为共享 Java 适配核与 1.21.6 收敛铺路）；性能为次要附带，写回仅占 chunk 时间约 2-6% ⇒ **端到端落在 ±10% 机器噪声带内，不予主张**（本 run wall/CPU 因指纹门重载不可用于端到端）。

### Tier 3 降级声明（本块重要修正）
- 原判据「两臂 region 文件逐块差 = 0」**在本 harness 不可达**：全域 `blocks=313,589,760 diff=86,841 = 0.0277%`；**切片证据**（按 `[WG-CONTENT-WB]` chunk 坐标切）——走过写回的 607 chunk = **0.0348%**，**从未走过本代码路径**的 2,583 chunk = **0.0260%** ⇒ 残差来源在写回之外（跨 run 固有抖动：FEATURES 阶段产物 + 随机刻可改写方块的**双向对称**变化）。
- **改判**：region 层降级为「**不可判**（noise-floor-limited）」；写回等价性由 Tier 1 承载。**口径外溢登记**：本工程历史上基于 live-server region 对拍的保真度数字（含 B1/E5 复算的 0.0133-0.0198%、历史代理基线 0.0180%）**同受该抖动限制**，不可当**绝对**保真度读；其作为**同仪器相对比较**的用法不受影响。判据沉淀 → workflow-patterns #129。

### R8/R9 论证（judge 前置）
- **R9（锁语义）**：新路径**不比老路径弱且在锁粒度/原子性上更强**——`readPacket` 自带一对锁（每**被替换** section 一对，实测 ≈8.0/调用 overworld，上限 24/chunk）vs 老路径**每非空气块**一对（数万次/chunk）；填**私有容器**后单次引用发布 ⇒ **无部分写可见窗口**。残余风险：容器引用非 `volatile`，可见性依赖既有 chunk 发布机制（与老路径同）；**未做**并发压力专项（R9-b）。
- **R8（`BelowZeroRetrogen`）**：1.20.1 该 flag 仅由**旧存档 chunk NBT** 携带（`ChunkSerializer:185/281-284`）；本 harness 每臂删 world 重新生成 ⇒ flag 恒不置；且 C 不动生物群系容器、不动 FEATURES 阶段 ⇒ 即便触发也无新增暴露面。

### 遗留 / 未覆盖
- R9-b 并发可见性专项未做；ID_LIST 仅合成覆盖（生产不可达）；`MAX_ID=4096` 沿用老路径同款约束；**sync 形态未跑**（本块只 async）；端到端 wall 不主张；共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）。
```

**追加文本 B（可直接粘贴到坑 2 段末尾，紧跟 `- 不要反射改计数（运行时字段是混淆名，且 setBlockState 更干净）` 之后）**

```markdown

> ⚠️ **补充（260911-05，C 线 bulk section 写回）**：本条结论**不推翻**——「写路径必须维护三计数」仍成立，且旧逐块路径保留为回退（`-Dcoreswap.bulkwb=0`）。C 线把机制升级为「**整段导入 + 原地换容器 + 经 mixin `@Accessor` 显式直写三计数**」：不再逐块 `setBlockState`，但**必须复刻同一套增量语义**（`calculateCounts` 是另一套定义，见本篇新增小节 + algorithm-fingerprints #23）；`@Accessor` 是**编译期 mixin**，不是本条禁止的「运行时反射改计数」（混淆名问题不存在）。判据不变：**任何绕过 `setBlockState` 的写路径都必须显式维护三个派生计数**。
```

---

### 条目 10：`versions/1.20.1/docs/10-timewise-archive.md` 追加块

- **目标文件**：`versions/1.20.1/docs/10-timewise-archive.md`
- **锚点特征串**（文件末尾，唯一）：`工单建议的 Release 标题未按仓库惯例命名由 Maint 纠正（规则已入 Maint 侧 AGENTS §5）。1.0.29 生命周期闭环。`
- **理由**：过程/中间结论/被推翻假说/工具演进归口时间线（07 篇只留结论与判据）；本块含「Tier 3 判据降级」与「两个错误」两类过程资产。
- **建议编号**：时间线块标题 = **260911-06**（**与同日 E5 复算块同号 `260911-05` 的冲突见 §3 开放项**；目录名不改）。

**追加文本（可直接粘贴）**

```markdown

---

## 260911-06（C 线 bulk section 写回；目录标签沿用 -05，实际 2026-09-11 20:56–22:1x）🔍 candidate（judge 未产出 / 用户未 confirmed）

> 过程产物 `.investigations/bulk-writeback-260911-05/`（`record-260911-05.md` §1-§9 / `plan-260911-05.md` §0 修订 R1-R4 / `api-probe-260911-05.md` / `scout-map.md` / `verify-design-draft-260911-05.md` / `errors-260911-05.md`（E1/E2 五段式 + 速查表）/ `evidence/`（六臂原始日志 + `wbc-*.txt` 抽取 + `bulk-summaries.txt` + `tier3-diff.txt` / `tier3-slice.txt` + 判据脚本 + `MANIFEST-sha256.txt` 40 行 + `impl-diff-260911-05.txt`））；判决 `.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`（candidate）+ `.artifacts/index.yaml`（`swe:bulk-writeback-260911-05:verdict`）；提交 `8dd9e71`（Java 实现；**Rust 零改动**，dll `838e89794a54e19d`）；通用模式 → algorithm-fingerprints #22/#23、compiler-idioms #24 补充案例、workflow-patterns #129 / #25 补充案例 / #110 补充案例 / #14 补充案例（subagent 草稿 → 主会话应用）。

- ✅ **D1 落地（Java 侧 bulk section 写回，Rust 零改动）**：路线 = 公开 `readPacket(PacketByteBuf)`（原案 5 参构造器因第 3 参 `PalettedContainer.DataProvider` 是包私有 record 而不可调用）+ `ChunkSectionAccessor` mixin（4 写 + 3 读，`@Mutable`）**原地换容器 + 直写三计数**（不新建 section、不调 `calculateCounts`、生物群系容器原样保留）；消掉每 chunk 98,304 次 `setBlockState`；回退开关 `-Dcoreswap.bulkwb=0`（旧路径保留不删）。
- ✅ **等价性（决定性，噪声无关）**：Tier 1 写回后读回 FNV-1a 64 + Tier 2 三计数序列指纹，**607/625/625 逐 chunk 差均为 0**、两臂 chunk 集相同、零异常（覆盖 59,670,528 位置）；编码四支 SINGULAR/ARRAY/BI_MAP/**ID_LIST** 合成自检逐位读回全绿（自然生成 `max_distinct=7` ⇒ ID_LIST 生产不可达，刻意压测）。本稿**独立复算**自归档 `wbc-*.txt`（607/625/625 行、`only_old`/`only_bulk` = 0、hash|dh 值差 0）复核一致。
- ✅ **性能（同 run 同仪器单变量）**：写回分项 overworld 2,151,591→**652,202** ns/chunk（−69.7%）、nether 2,657,477→**859,626**（−67.7%）、end 2,809,444→**804,970**（−71.4%）；`counts_set` 由 `calculateCounts` 的 72,480 ns/section → 116 ns/section。**价值定位下调**：可维护性/可移植性为主，性能次要附带（写回占 chunk ~2-6% ⇒ 端到端在 ±10% 噪声带内**不主张**）。
- ❌ **E1（首跑 539 次 IAE）**：`bits==0` 单态 section 不得构造 `PackedIntegerArray`（`Validate 1..32`）——位宽 0 = **无 storage**；判据 = 复刻 MC switch 逐 case 过构造器前置条件；定位 = **先看两臂集合差（607 vs 68）再看异常原文**。
- ❌ **E2（最高价值：性能问题的解法暴露正确性问题）**：`calculateCounts()` 与增量 `setBlockState` 的计数语义**不一致**（流体重复计入 `nonEmptyBlockCount`；`nonEmptyFluidCount` 只计有随机刻的流体）⇒ 走构造器会静默改变 `hasRandomFluidTicks()` / 客户端包语义；生成期语义由**调用点**决定（vanilla 走增量）⇒ 改为复刻增量语义。三计数**不入存档**（`ChunkSerializer:310-311`）⇒ region 对拍看不到，唯一可见面 = 生成期内存态。
- ⚠️ **Tier 3 判据降级（重要修正）**：原「region 逐块差 = 0」在本 harness **不可达**——全域 0.0277%，**切片**后走过写回的 607 chunk = 0.0348%、**从未走过本代码路径**的 2,583 chunk = **0.0260%** ⇒ 残差为跨 run 固有抖动（FEATURES 产物 + 随机刻可改写方块的双向对称变化）；region 层改判「不可判」，等价性由 Tier 1 承载。**口径外溢登记**：历史 live-server region 对拍保真度数字（含 B1/E5 复算 0.0133-0.0198%）同受该抖动限制，**不可当绝对保真度读**（相对比较用法不受影响）。
- ✅ **R8/R9 论证**：R9 = 新路径不比老路径弱且在锁粒度/原子性上更强（`readPacket` 自带一对锁，每被替换 section 一对 ≈8.0/调用（上限 24/chunk）vs 老路径每非空气块一对；填私有容器后单次引用发布 ⇒ 无部分写可见窗口）；残余 = 引用非 `volatile`，未做并发压力专项（R9-b）。R8 = `BelowZeroRetrogen` 仅由旧存档 NBT 携带，本 harness 每臂删 world ⇒ flag 恒不置，且 C 不动生物群系/FEATURES ⇒ 无新增暴露面。
- 🔍 **open（未核/降级/边界）**：① R9-b 并发可见性专项未做；② ID_LIST 仅合成覆盖（生产不可达）；③ `MAX_ID=4096` 沿用老路径同款约束；④ **sync 形态未跑**（本块只 async）；⑤ 端到端 wall 不主张（噪声带内）；⑥ 共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）；⑦ **Tier 1/2 判据脚本的运行输出未落盘**（`cmp_wb.py` 与原始日志均在库，可复跑；本稿已用归档抽取件独立复算 PASS，建议补存一次输出）；⑧ **流程**：judge **未产出**（`review-260911-05.md` 不存在）、confirmed 未授予；⑨ **块号**：与同日 E5 复算块（19:37-20:52）同用 `260911-05` 标签，本行按 `-06` 记、目录名不改（见 `.investigations/bulk-writeback-260911-05/knowledge-update-draft-260911-05.md` §3）。
```

---

### 条目 11：`knowledge/INDEX.md` 追加块（同步索引）

- **目标文件**：`knowledge/INDEX.md`
- **锚点特征串**（文件末尾，唯一）：`来源：` + "`.investigations/e5-recompute-260911-05/record-260911-05.md`" + `（**candidate**，judge PASS-with-conditions 条件已应用；confirmed 留人类）。`
- **理由**：core-knowledge 硬性要求「写入后同步更新 INDEX.md」；按近期惯例（260909-02 起）**追加块承载**，**不动**分类入口表（该表自 260904-06 起已不再逐块同步）。
- **建议编号**：追加块标题 = `> 260911-06 追加（C 线 bulk section 写回；目录标签沿用 -05）`。

**追加文本（可直接粘贴）**

```markdown

> 260911-06 追加（C 线 bulk section 写回；目录标签沿用 -05）：**algorithm-fingerprints 新增 #22 简记 / #23**（#22 = MC 1.20.1 `PalettedContainer.readPacket` 逐字节编码契约 + `BLOCK_STATE` 位宽映射（bits 0 = 无 storage / 1-4 恒 4bit / 5-8 请求位宽 / ≥9 全局 state id）+ `readLongArray` 长度不符**静默丢弃**（返回值被丢）+ 零布局转换（`computeIndex` 同构）；#23 = `ChunkSection` 三计数**两套语义**（`calculateCounts` ≠ 增量 `setBlockState`；流体重复计入 block 计数；流体计数只计有随机刻的流体）+ 三计数**不入存档** ⇒ 唯一可见面 = 生成期内存态，判据 = 同一语义两套实现逐条对表 / 「构造器会自动算好」是危险直觉 / 等价门必须分层）；**compiler-idioms #24 补充案例**（批量导入共享容器 = 锁粒度更粗 + 发布更原子——`readPacket` 自带一对锁，每被替换 section 一对 ≈8.0/调用 vs 老路径每非空气块一对；填私有容器后单次引用发布 ⇒ 无部分写窗口）；**workflow-patterns 新增 #129（最高价值·错误优先）**（「零差」类判据的**可达性预检** + **覆盖集切片归因法**——live-server region 对拍存在跨 run 固有抖动（全域 0.0277%；走过写回 0.0348% vs 未走过 0.0260%）⇒ 「逐块差 = 0」结构性不可达；未覆盖集也有差分 ⇒ 差分来源在改动之外；附**口径外溢登记**：历史 live-server region 对拍保真度数字不可当绝对保真度读）+ **#25 补充案例**（计划期技术断言静态失真两形态：public 构造器 ≠ 可调用（参数类型包私有）/ 「不需要 X」必须附验证方式；附工作量低估三项与收益定位下调）+ **#110 补充案例**（生产不可达的编码分支 MUST 合成压测并显式声明，否则「全绿但未覆盖」）+ **#14 补充案例**（等价门必须与被测变更**同层**——`[WG-CONTENT]`（Rust buf 层）对 Java 侧写回变更结构性不敏感）。来源：`.investigations/bulk-writeback-260911-05/{record-260911-05.md,errors-260911-05.md,plan §0 R1-R4,api-probe}` + `.artifacts/bulk-writeback-260911-05/verdict-260911-05.md`（**candidate**，judge 待审；confirmed 留人类）。时间线 → `versions/1.20.1/docs/10-timewise-archive.md` 260911-06 块。
```

---

### 条目 12（跨块·可选）：`e5-recompute-260911-05/record-260911-05.md` 追加限定指针（口径外溢）

- **目标文件**：`.investigations/e5-recompute-260911-05/record-260911-05.md`
- **锚点特征串**（文件末尾，唯一）：`4. 知识库数值更新（S4）须经 subagent 草稿后应用（框架强制）。`
- **理由**：C 线 §5 明确要求「口径外溢须登记为 B1 的限定说明」——B1/E5 的保真度数字的**载体**就是该记录，指针落在数字所在处最有效。**只追加、不改原文**（§15.4：原结论正文一字不改）。
- **建议编号**：无限定指针（追加式注记）。
- ⚠️ **跨块动作**：涉及另一工作块的记录 ⇒ 需主会话/人类确认（见 §3 开放项 4）。

**追加文本（可直接粘贴）**

```markdown

---

> ⚠️ **限定指针（260911-05 C 线追加，原文不删不改）**：本记录的复算数字（0.0133-0.0198% 等）与历史代理基线（0.0180%）**同受 live-server region 对拍的跨 run 固有抖动限制**——C 线实测：全域差分 0.0277%，**从未走过被测代码路径**的 chunk 也有 **0.0260%**（切片证据，`.investigations/bulk-writeback-260911-05/evidence/tier3-slice.txt`）。⇒ 这些百分比**不可当绝对保真度读**（⚠️ 抖动底 0.026% 是在 **C 线载体/区域集**上测得，按 §9.7 载体要素**不作精确阈值跨区迁移**，但量级限制成立）；本记录中作为**同仪器相对比较**的用法（「≤ 历史基线」论证）**不受影响**。限定链：`.investigations/bulk-writeback-260911-05/record-260911-05.md` §5「口径外溢」。
```

---

## §2 归口说明

| 内容 | 载体 | 依据 |
|---|---|---|
| E1 / E2 五段式 + 速查表 | `.investigations/bulk-writeback-260911-05/errors-260911-05.md`（**新建，独立成篇**） | 项目级指定错误台账载体（SUBAGENT-KNOWLEDGE-GUIDE §三）；错误优先原则 |
| 编码契约指纹（#22）、三计数两套语义（#23） | `knowledge/discovered/algorithm-fingerprints.md` | 通用可复用（跨版本/跨项目）中/高价值；1.21.6 收敛与共享适配核会再遇 |
| 锁粒度/发布原子性（#24 补充案例） | `knowledge/discovered/compiler-idioms.md` | 同代码点已有主条 ⇒ 补充案例（去重纪律） |
| 判据可达性 + 切片归因（#129）、计划期断言（#25 补充）、不可达分支压测（#110 补充）、同层门（#14 补充） | `knowledge/discovered/workflow-patterns.md` | 判错方法/判据类高价值；已有近邻主条者用「补充案例」承载 |
| INDEX 同步 | `knowledge/INDEX.md`（末尾追加块） | core-knowledge 硬性要求；近期惯例 = 追加块，不动分类入口表 |
| C 线结论本体（形态/等价性/性能/降级声明/R8-R9/遗留） | `versions/1.20.1/docs/07-block-pipeline.md`（末尾追加小节 + 坑 2 追加限定指针） | 07 篇主题 = 块级流水线 + Java 写入路径的坑；**结论进主题篇** |
| 过程/中间结论/被推翻假说（Tier 3 降级、E1/E2 现象链、R1-R4 修订） | `versions/1.20.1/docs/10-timewise-archive.md`（末尾追加 260911-06 块） | 时间线只进 10，不散落主题篇 |
| 口径外溢限定指针 | `.investigations/e5-recompute-260911-05/record-260911-05.md`（末尾追加，跨块可选） | 指针落在数字所在载体；§15.4 只追加不改写 |
| **不记（低价值）** | 不写任何知识库载体 | 见 §0 第 8/12/13 行：R4 一次性收益定位、实现细节（全空气短路 / ThreadLocal epoch 技巧 / `MAX_ID` 约束）、当前对齐状态快照——留在 `.investigations/` 过程产物与 `.artifacts/index.yaml` |
| **不写 09-multi-dimension.md** | — | C 的机制不属于多维度主题；三维度覆盖只是**覆盖面声明**，写在 07 小节表内即可（防散落/堆积） |
| **不改 01-architecture.md** | — | 可维护性/共享适配核的论证属后续「共享核抽取」立项内容；本块不预占（如需一行指针由主会话定，建议不加） |

---

## §3 必须由人类/主会话决定的开放项

1. **块号冲突（同日两块同号 `260911-05`）**：`e5-recompute-260911-05`（19:37–20:52，B1/E5 复算）与 `bulk-writeback-260911-05`（20:56–22:1x，C 线）**同日同号**。本稿文档标签按 **`260911-06`** 记 C 线，**目录名不改**（避免改名返工，AGENTS.md §三.5 命名纪律的成本权衡）。→ 需主会话确认：接受 `-06` 文档标签 / 或统一按 `-05`（两线同块）并在时间线合并叙述。
2. **错误台账载体命名与去重**：新建 `errors-260911-05.md`（本稿建议）vs `bulk-writeback-260911-05-errors.md`；且需确认**与 `record-260911-05.md` §3 的关系**——本稿建议 **record 原文一字不改**（保留原始记录），台账作为正式载体（§三「独立成篇」）；若主会话判定重复成本过高，**最低要求 = E1/E2 的可复用判据必须落盘**（不得只留在 record §3）。
3. **依赖 judge 结论的条目（见 §7）**：Tier 3 降级是否被 judge 认可、E2 的 P0 定级、#129 的降级措辞与「口径外溢」表述范围 —— judge 返回后需复核。
4. **跨块动作（条目 12）**：是否在 `e5-recompute-260911-05/record-260911-05.md` 追加口径外溢限定指针（涉及另一工作块记录）。
5. **R1 的「javac 拒」表述精度**：`api-probe-260911-05.md` §1 声明的核对方式 = **源码级核对**（读 `PalettedContainer.java:398` 无修饰符 record），文中引用的 `DataProvider is not public in PalettedContainer; cannot be accessed from outside package` 是 **javac 语义描述**，**归档里没有编译运行记录**。→ 建议措辞统一为「源码级核对 + Java 语言规则（包外不可命名包私有类型）」，或补一次一行编译探针；**不得**写成「实测 javac 报错」（anti-hallucination）。
6. **推导值标注**：`#22` 中 storage long 数（4bit → 256 / 15bit → 1024）系由 `PackedIntegerArray.java:261/266` 公式**推导**（非实测打印）⇒ 已标注「推导值」，引用时请保留标注。
7. **07 篇追加小节的取代关系**：本稿把坑 2 的补充按「**追加限定指针**」处理（不构成取代）；若主会话认为需要 §15.4 取代链（supersedes 双指针），请按该格式补双向指针。
8. **`algorithm-fingerprints` #23 的载体归属**：本稿放 `algorithm-fingerprints`（机制指纹）；若主会话倾向「判据为主 → workflow-patterns」，可改为 workflow-patterns 条目（但**编号需重核**，现最高 #128 + #129 已占用）。
9. **是否补存 Tier 1/2 判据脚本输出**（证据补全）：`cmp_wb.py` 在库、原始日志在库，本稿已用归档抽取件独立复算 PASS；建议主会话复跑一次把输出存 `evidence/cmp-wb-260911-05.txt`（低成本、消除 judge 三源核对时的证据缺口质疑）。

---

## §4 应用检查清单（主会话应用后自检）

- [ ] **只增不删（§15.4）**：逐条确认目标文件原有内容**零改动**——所有动作均为「文件末尾追加」或「指定锚点后插入」；被限定/被补充的旧结论（07 篇坑 2、#24 主条、#110/#25/#14 主条、E5 记录数字）**正文一字未改**。
- [ ] **编号无冲突**：`workflow-patterns` 新号仅 **#129**（原最高 #128）✓；`algorithm-fingerprints` 新号 **#22 / #23**（原最高 #21）✓；`compiler-idioms` **无新号**（#24 补充案例）✓；`build-tooling` **零新增** ✓；`knowledge/INDEX.md` 追加块标题 `260911-06`（与已有 `260911-05` 块不重号）✓。
- [ ] **INDEX 同步**：`knowledge/INDEX.md` 末尾追加块已贴；分类入口表**按惯例未动**（如主会话要动表，须逐行只追加）。
- [ ] **数值零编造、可回一手证据**：607/625/625、59,670,528、−69.7/−67.7/−71.4%、2,151,591→652,202、72,480→116 ns/section、0.0348/0.0260/0.0277%、`max_distinct` 7/7/2、`idlist_bits=15`、`state_ids_size=24137`、`sections_replaced` 4856/4815/562、`air_sections_skipped` 9712/5185/4438 —— 全部可回 `evidence/bulk-summaries.txt` / `tier3-slice.txt` / `tier3-diff.txt` / `record` §4；**推导值（256/1024 longs）已标注**；**无实测计数的项（老路径锁次数量级）已标「估计」**。
- [ ] **§9.7 三要素**：07 追加小节（载体/覆盖面/可比性三行）、时间线块、#129/#23/#22 均带载体与覆盖面声明。
- [ ] **置信度合法**：本稿全部条目 `candidate`；**无任何 confirmed 自标**。
- [ ] **格式对齐目标文件末尾现状**：`## 发现 #N` / `### 发现 #N 补充案例` 标题形态与近邻条目一致；元数据行（发现时间/发现者/置信度/module）齐备。
- [ ] **错误台账**：五段式齐备（现象/根因/定位/修复/教训）、末尾速查表已加两行、E1/E2 编号与 record §3 一致。
- [ ] **时间线归口**：新增内容**全部**在 10 篇（过程）或 07 篇（结论），**未**在 07 篇新建时间线式章节。
- [ ] **judge 复核**：§7 依赖 judge 的条目在 judge 返回后逐条复核，必要时按 §15.4 记取代/限定。

---

## §5 本稿独立复核记录（哪些是本稿自做的、哪些是转引的）

**本稿自做（一手复核）**
1. **Tier 1/2 独立复算**（不依赖 `cmp_wb.py` 的运行输出）：从归档抽取件 `evidence/wbc-{c,c-nether,c-end}-{old,bulk}.txt` 解析 `hash=`/`dh=`，结果 = **607/625/625 行、`only_old=0`、`only_bulk=0`、值差 0**（三维持）⇒ 与 record §4 结论一致。
2. **切片百分比独立复算**：`20738/59670528 = 0.0348%`、`66103/253919232 = 0.0260%`、`86841/313589760 = 0.0277%`；且 `59,670,528 + 253,919,232 = 313,589,760`（= `tier3-diff.txt` 全域分母）、未覆盖 chunk = `3190 − 607 = 2583` ✓。
3. **vanilla 源码逐点实读**（本稿自行打开，非转述）：`ChunkSection.java:21-31 / 56-93 / 111-140`、`PalettedContainer.java:203-215 / 398-404 / 420-430 / 465-467`、`PacketByteBuf.java:859-867 / 924-932`、`PackedIntegerArray.java:256-276`、`SingularPalette:67-68`、`ArrayPalette:85-90`、`BiMapPalette:74-79`、`IdListPalette:45-46`、`LockHelper.java:17-71`、`NoiseChunkGenerator.java:335-355 / 414-419`、`ChunkSerializer.java:300-324`。
4. **目标文件锚点实读**：`knowledge/INDEX.md` 末尾、三个 `discovered/*.md` 末尾、`docs/07` 坑 2 段与末尾、`docs/10` 末尾、`e5-recompute` record 末尾 —— 锚点特征串均为**文件内唯一出现**（已核对）。
5. **证据缺口发现**：`evidence/` 中**没有** `cmp_wb.py` 的运行输出文件（无 `cmp-wb*.txt`）⇒ Tier 1/2 的「PASS」打印结论未归档（原始日志与脚本在库、可复跑）。已用自做复算（第 1 项）替代验证；建议补存（§3 开放项 9）。
6. **编号冲突核对**：grep 实查四个 discovered 文件的现有最高号（#128 / #58 / #21 / #24），本稿建议编号无冲突。

**转引（未独立复算，诚实声明）**
- 性能数字（Tier 4 各 ns/chunk、分项）与 `[WG-BULKWB-TEST]` 四支 PASS：直接引用 `evidence/bulk-summaries.txt`（原始日志在库），本稿**未重跑** JVM/Chunky。
- `calculateCounts` 72,480 ns/section 与 `state_ids_size=24137`/`idlist_bits=15`：引用 `evidence/bulk-summaries.txt`（自证行）与 record §4。
- R9 中「老路径每非空气块一对锁（每 chunk 数万次）」为 record §6 的**量级估计**（非实测计数）——本稿保留「估计」标注，未升级为实测。
- 本稿**未触碰** `knowledge/`、`versions/`、`.artifacts/` 任何文件（仅读取）；未运行任何构建/世界生成。

---

## §6 声明

- **本草稿未触碰知识库任何文件**：`knowledge/`、`versions/`、`.artifacts/` 全部**零写入**（只读核对）。本文件是**唯一产出**：`.investigations/bulk-writeback-260911-05/knowledge-update-draft-260911-05.md`。
- **状态 = draft**；本稿全部建议条目为 **candidate 级**（有证据），**confirmed 只有人类能授予**。
- **只增不删**：涉及被补充/被限定的旧结论（07 篇坑 2、compiler-idioms #24、workflow-patterns #14/#25/#110、E5 记录数字）**一律只追加指针或补充案例，原文一字不改**。
- **judge 结论未产出**（`review-260911-05.md` 不存在，产出时点已核）⇒ 依赖 judge 的条目见 **§7**，请主会话在 judge 返回后复核。

---

## §7 judge 结论相关待办（供主会话在 judge 返回后复核）

| 本稿条目 | 依赖 judge 的点 | judge 返回后的动作 |
|---|---|---|
| 条目 1（错误台账 E1/E2） | E2 的 **P0 定级**与「已 resolved」定性是否被 judge 认可 | 若 judge 提出定级/机制修正 → 只追加修正注记（不改 E2 正文），并按 §15.4 记取代链 |
| 条目 5（#129） | **Tier 3 降级（noise-floor-limited）**是否被 judge 认可；「口径外溢」表述范围（是否扩大到全部历史 region 对拍数字）是否被接受 | 若 judge 要求收窄/扩大 → 只追加限定或补指针；判据本体（可达性预检 + 切片归因）不受影响 |
| 条目 5/11（口径外溢） | 抖动底 0.026% 的**跨区迁移**边界（本稿已按 §9.7 声明「不作精确阈值迁移」）是否被 judge 认为足够 | 若要求更强的载体声明 → 补 §9.7 三要素行 |
| 条目 4（#24 补充案例 / R9） | **R9-b 并发可见性专项未做**是否被判为阻塞 candidate 的条件 | 若被判阻塞 → 在条目与 docs 小节加「未验证项」强化标注（不改机制结论） |
| 条目 9/10（07 篇 + 时间线） | docs 小节的「结论 vs 过程」归口是否符合 judge 预期；遗留清单是否需补项 | 按 judge 条件追加（只增不删） |
| 条目 6（#25 补充案例 / R1-R4） | R1「javac 拒」表述精度（§3 开放项 5）是否被 judge 要求补编译证据 | 若要求 → 补一次一行编译探针并把原始输出落盘，再改措辞 |
| 全部条目 | judge 可能要求**补存** Tier 1/2 判据脚本输出（§5 第 5 项证据缺口） | 复跑 `cmp_wb.py` → 存 `evidence/cmp-wb-260911-05.txt` → 在条目 9/10 的引用处补该证据路径 |
