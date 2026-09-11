# C（Rust bulk section 写回）验证设计草稿（260911-05）

> 状态：**draft**（主会话草稿，待 Phase 0 计划批准后并入 plan）。与 scout-map.md 正交：本文件只管「怎么证明它等价 / 值不值得」，不预判实现形态。

## 0. 为什么 C 的验证必须比 A1a 更强

A1a（写回跳空气）改动**语义零变化**（只是少写 air），验证只需行为门（指纹一致）。C 改动**写回机制本身**（逐块 `setBlockState` → 整段替换），可能改变：

- section 的 palette 编码/位宽（同一组方块的 palette 顺序、`nonEmptyBlockCount`）
- section 的**派生状态**（随机刻标记、block entity 触发、光照重算范围）
- 高度图填充时机与来源（原来由 `setBlockState` 触发的 `trackUpdate` 副作用）
- 跨 section 边界行为（`ChunkSection` 之外的 chunk 级状态）

⇒ **判据必须从「百分比差异 ≤ 噪声」升级为「逐位零差」**（噪声无关的强判据），并把「零差」拆成两层：① 逻辑层（Java 读回同一点得同一 state）② 编码层（section 的 palette/data 字节级一致，若关心存档体积与序列化）。

## 1. 三层验证（由强到弱，全部可离线）

### Tier 1 — 逻辑层逐 chunk 指纹（主判据，噪声无关）
- 在 Java 侧、写回**之后**，对每个 chunk 的**全部 section** 读回并算 FNV-1a 64（复用 A1b 的 `wgBufHash` 形态，但输入改为 `chunk.getSection(i).getBlockState(x,y,z).getRawId()` 遍历 98,304 点），打印 `[WG-CONTENT-WB] chunk(x,z) hash=<hex>`。
- **A/B**：同构建态单变量（旧逐块路径 `-Dcoreswap.bulkwb=0` ↔ 新 bulk 路径），同 seed 同 region。
- **判据**：两臂**逐 chunk hash 全等**（不是「差异小」）。任一 chunk 不等 = FAIL，且可直接定位到 chunk 坐标做单点 diff。
- 成本：读回 98,304 点/chunk 是热路径开销 ⇒ **仅诊断开关打开时执行**（默认关，与 MIXLOG 同族纪律），且 A/B 两臂对称开启（避免口径不对等，见 index.yaml:1223 的教训）。

### Tier 2 — 编码层 section 字节一致（存档/序列化等价）
- 用 accessor mixin 读 section 的 `PalettedContainer`（palette 列表 + 位打包 data + `nonEmptyBlockCount`），算字节级 hash。
- **判据**：与 Tier 1 同臂，palette 顺序与位宽**允许不同**（编码不是语义），但**解码后的 block 序列必须全等**（由 Tier 1 覆盖）；本层只用于**发现意外副作用**（如 `nonEmptyBlockCount` 未更新）——所以判据是「派生字段正确」而非「字节相同」。
- 若 Tier 1 全等而本层派生字段不等 ⇒ 视为 **FAIL**（下游 `isEmpty`/光照会用到）。

### Tier 3 — 存档对拍（端到端，用**已修好的**工具）
- 两臂各自生成同一 region → 用 `.investigations/e5-recompute-260911-05/diff_arms_fixed.py`（tag7 已修）对拍。
- **判据**：`diff = 0`（**不是** ≤ 噪声基线）——因为这是同 seed 同实现同路径，唯一变量 = 写回机制，理论上应逐块相同。
- 历史代理基线的教训（0.0180%）在此**不适用**：那是跨实现/跨 run 的噪声，本层是同实现的单变量。
- 覆盖面声明（§9.7）：三维持（overworld/nether/end）；region 集与 Tier 1 同一批；不可与 260910-05 的旧口径数字互引。

### Tier 4 — 性能 A/B（值不值得）
- 同构建态单变量：wall、进程 CPU 核数、`ChunkTiming.write` 分项（1.21.6 有分项，1.20.1 需补或退化为 wall）。
- **判据**：写回分项必须显著下降（预期 = 从 O(blocks) 降到 O(sections)）；wall 变化落在 ±10% 噪声带内也**可接受**（C 的主要收益是**可移植性**与**去掉 per-version 热循环**，不是 wall）。
- 若写回分项**未**显著下降 ⇒ 实现未真正 bulk 化（回归实现层，不是「优化无效」）。

## 2. 回退与开关纪律

- 新路径 MUST 留同构建态回退开关（`-Dcoreswap.bulkwb=0` → 旧逐块路径），使 Tier 1/3/4 的 A/B 是**同 jar 单变量**，且出问题可即时降级出货。
- 旧路径**不得删除**（至少保留一个版本周期），并在 10 时间线登记保留期限。

## 3. 不可用/需谨慎的既有载体

- `[WG-CONTENT]`（buf 层指纹，读的是 **Rust 输出缓冲**）在 C 下**语义变化**：它证明的是「Rust 输出不变」，而 C 的变量在 **Java 应用层** ⇒ C 需要的是**写回后**的指纹（Tier 1），不能拿 buf 层指纹充当 C 的等价证据。
- 自比 0 差 = **恒真对照**（B-C7 教训），不得作为 C 的正确性证据。
- 逐点小样本「一致」在解析失步下会**成对误读成同一值**（假一致，旧读法 9/45 vs 规范 50/55）⇒ 小样本一致不能替代全域普查。

## 4. 待 scout 回答后才能定的项

- Tier 2 的 accessor 是否可行（`ChunkSection`/`PalettedContainer` 字段可见性）
- 高度图由谁填充、bulk 路径是否要显式接管（若原来依赖 `setBlockState` 副作用）
- 1.20.1 侧是否有 `ChunkTiming` 分项可作 Tier 4 的写回成本尺
