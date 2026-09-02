# 草稿：M16 知识库更新（JNI 写入路径 id 域错位）——2026-09-01 session

> 状态：draft（subagent 产出草稿，待主会话应用 + 验证签核）。
> 依据：已 judge 审查通过的 candidate 结论（2026-09-01 session）。
> 证据源：`.investigations/multiworld-port/multiworld-errors.md`（M1-M15 + 本轮）、`.investigations/multiworld-port/snapshot-CppBridge-m16fix.java`、commit `6a7337d`（引入错位的修复）。
> 自检：已过 SUBAGENT-KNOWLEDGE-GUIDE §四清单（五段式完整/根因机制层/定位含工具/速查表同步/低价值不写 docs）。

---

## A 部分：错误台账追加（载体：`.investigations/multiworld-port/multiworld-errors.md`，M15 之后、速查表之前插入）

（应用说明：M14 条目末尾追加一行标注：`> **【2026-09-01 结案】** M14 开放问题已由 M16 真根因解释——非 feature 阶段上下文问题，是 JNI 写入路径 id 域错位（见 M16）。上文「根因方向」判断作废，三方对照/指纹定位方法结论保留。`）

---

## M16.【结案】实机下界「怪异城」真根因：JNI 写入路径 block id 域错位（raw block id vs global state id）

### 现象
- 实机下界（M13 buffer 修复 + M12 STATE_BY_ID 修复之后）依然异常：chunk(-5,-3) 存档导出 **oak_leaves ×3150 + 多种 sapling + note_block** 成片出现在 nether chunk（交接 M14 开放问题）。
- 上一轮定位方向 = vanilla feature 装饰阶段 biome 上下文污染（NEXT_SESSION §3 候选三选一）——**该方向本轮被证伪**。

### 根因（机制）
- **JNI 写入路径的 id 域错位**：
  - Rust 侧 buf 中的值 = **blocks.json 域的 block 注册表 raw id**（BlockProbe 参照导出同为 `getRawId`，同域，自洽）；
  - 但 Java 侧 `CppBridge.writeChunk` 用 `Block.STATE_IDS.get(id)`（**global state id 域**）解码——raw id 当 state id 用，nether 块的高 raw id 在 state id 域错位解码成 oak_leaves×3150 等不相干方块；
  - 错位由 `6a7337d` 引入（该 commit 为修 M12 显错而把解码域改成 STATE_IDS，**只对齐了症状层，未查数据源头域**）。
- **闭环判据**：干净重生成（修复前）同 chunk 存档 oak_leaves×3150 与用户实机存档**数量完全一致**——同一确定性错位，两个环境复现同一签名。

### 定位（诊断方法，本轮三步证伪 + 一锁定）
1. **H1（feature 阶段 biome 上下文污染）证伪 ①**：scout 证明 feature 候选集 = 3×3 邻 chunk biome 容器并集 ∩ biomeSource（`ChunkGenerator#method_39787`）——结构上就是取 chunk 自身 biome，无跨维度污染通道。
2. **H1 证伪 ②（数据层）**：诊断 mixin（`-Dwg.dumpbiome`）dump 出 feature 阶段该 nether chunk 的 3×3 biome 上下文 = **纯 nether**（soul_sand_valley / nether_wastes），无任何 overworld 条目 → 「拿到主世界森林 feature 集」不成立。
3. **「Status 卡 biomes」候选证伪 ③**：Status 推进在 ChunkStatus 包装层（isAtLeast 门控），mixin cancel populateNoise 不影响推进——fill 日志中的 `status=biomes` 是 **fill 时点快照**，非最终状态；「chunk 永不重生成/调度异常」担忧解除。
4. **锁定真根因**：修复前干净重生成复现实机签名（数量逐块一致）→ 错误在**写入层且确定性**，与下游阶段随机性无关；对照 Rust buf 域（raw id）、参照导出域（getRawId）、Java 解码域（STATE_IDS）三方，**唯一异域跳点 = writeChunk 解码**。

### 修复
- `CppBridge.writeChunk` 解码改回 raw id 域：`Registries.BLOCK.get(id).getDefaultState()`（快照：`.investigations/multiworld-port/snapshot-CppBridge-m16fix.java`；runtime/ 已 untrack 无 git diff）。
- 注意与 M12 的区分：M12 修的是「STATE_BY_ID **查询**路径」的域（当时结论对查询成立）；本次错的是「**存档写入**路径」——两条路径数据源头域不同，不能共享同一解码函数。

### 验证（Partial 声明）
- **验证分层 = Partial（单 chunk 存档级，非 Full）**。
- 修复后存档 vs vanilla **FULL** 参照逐位：nether chunk(-5,-3) **82.16%**、overworld chunk(10,10) **87.75%**；残差 = soul_soil/cave_air/air/石系边界微结构——归因于参照含 carvers/features 而我方只替换 NOISE+SURFACE，及 cave_air/air 语义差，非写入层错误。
- **⚠️ 口径分流（必读）**：82.16% / 87.75% 是「**存档写入路径正确性**」口径（存档 vs vanilla FULL 参照）；**不延续**旧 96.06% 的「**探针（Rust 产物 raw id 域）对齐**」口径——两者比较对象、id 域、阶段覆盖均不同，禁止跨口径比大小或拼趋势。

### 教训（可复用判错经验）
1. **「id 域要三查」**：blocks.json / 参照导出 / Rust buf 同为 raw id 域，Java 写入用 STATE_IDS 即错——跨层传 id 必须显式声明域（raw block id vs global state id），是「seed/坐标三查」（AGENTS 探针采集核对铁律）的 **id 域版本**。
2. **「探针对齐率高 ≠ 存档正确」**：探针对比在 raw id 域自洽，掩盖了 JNI 写入解码层的域错位——「纯函数 vs 生产插值」工具语义陷阱（M12 补遗二）的同族新形态：**验证工具的域自洽 ≠ 生产路径的域正确**。
3. **6a7337d 教训延伸**：修 bug 时「改成什么域」要先查**数据源头的域**，而不是只对齐症状层——症状层修复可能把错误换一个域延续下去（M12→M16 两跳实锤）。
4. **overworld「零回归」是探针口径假象**：6a7337d 后 overworld 存档层从未被验证（spawn 区由 vanilla 在 SERVER_STARTED 前代生成，掩盖写入层错误）——「未报错/无差异反馈」≠「路径被验证」。

---

## B 部分：速查表追加（同台账文件末尾「错误 → 根因 速查表」加一行）

| 实机下界「怪异城」：nether 存档 oak_leaves×3150+多 sapling+note_block，数量跨环境精确复现（M16【结案】） | JNI 写入路径 **block id 域错位**：Rust buf = raw block id 域（与 blocks.json/参照导出同域），writeChunk 用 `STATE_IDS.get(id)`（state id 域）错位解码（6a7337d 引入，只对齐症状层未查数据源头域）；feature 阶段 biome 污染（H1）与 Status 卡 biomes（H2）均被数据证伪 | **id 域三查**（blocks.json/参照导出/Rust buf/Java 解码四点同域核对），跨层传 id 必须显式声明域；「不相干方块成片 + 数量精确复现」= 写入层确定性错误签名（判据见 discovered 发现 #6）；**探针对齐 ≠ 存档正确**（验证口径必须覆盖生产写入路径） |

---

## C 部分：discovered 新条目（归口判断：**`knowledge/discovered/compiler-idioms.md`**，追加为「发现 #6」——该文件当前最大 #5）

> 归口理由：id 域错位是跨层（Rust→JNI→Java）数据表示惯用法错误，属「跨模块通用模式/惯用法」类，非算法指纹（algorithm-fingerports 收 MC 算法特征）、非工作流（workflow-patterns 收流程模式）。同时在本条目下交叉引用 workflow-patterns 发现 #8。

（应用说明：在 `compiler-idioms.md` 末尾追加；INDEX.md「语言/编译器惯用法」行说明列末尾追加「、跨层 id 域错位 raw block id vs state id（2026-09-01）」。）

## 发现 #6: 跨层 id 域错位（raw block id vs global state id）——Minecraft mod 写入存档的判据

**发现时间:** 2026-09-01
**发现者:** worker（multiworld-port M16）
**来源定位:** `.investigations/multiworld-port/multiworld-errors.md` M16 + `.investigations/multiworld-port/snapshot-CppBridge-m16fix.java`
**置信度:** candidate（闭环判据实锤：修复前重生成与实机存档错位数量完全一致 + 修复后存档级 Partial 验证通过；confirmed 待用户拍板）
**module:** re-code / swe（JNI/FFI 跨语言边界）

### 观察
Minecraft 存在两套块 id 域：**raw block id**（`Registries.BLOCK.getRawId`，block 注册表序）与 **global state id**（`Block.STATE_IDS`，blockstate 展平序）。跨层（如 Rust 产物 → JNI → Java 写入存档）传 id 时，若中间某一跳换了域而无声明，低 id 区（经典块，两域前段近似重合）恰好命中、高 id 区（nether/新块）全面错位——错误信号是「不相干方块成片」而非崩溃。

### 证据
- 现象签名：nether 存档 chunk 出现 oak_leaves×3150 + 多 sapling + note_block（主世界森林 feature 组合），重生成间数量精确复现（`.investigations/multiworld-port/multiworld-errors.md` M14/M16）。
- 排除链：feature 阶段 3×3 biome 上下文 dump 为纯 nether（-Dwg.dumpbiome）→ 排除 feature 污染；Status 推进由包装层门控不受 mixin cancel 影响 → 排除调度问题；唯一异域跳点 = writeChunk 解码（Rust buf 与 blocks.json/参照导出同为 raw id 域）。
- 修复闭环：`Registries.BLOCK.get(id).getDefaultState()`（raw id 域）替换 `STATE_IDS.get(id)` 后，存档 vs vanilla FULL 参照 nether 82.16% / overworld 87.75%（存档写入口径，残差为 carvers/features 覆盖差）。

### 如何利用
- **每跳「域声明」**：JNI/FFI 传 id 的每一跳（导出/传输/解码写入）都要显式声明所用域；参照导出域与写入解码域必须同源核对——这是「seed/坐标三查」铁律的 id 域版本。
- **判据（写入层确定性错误的签名）**：
  1. 块名直方图签名：**橡树叶 + 多种 sapling + note_block 混入 = 错位解码签名**，非 feature 签名（feature 不会以 note_block 成片混入）；
  2. 同代码重生成数量精确复现 = 写入层确定性错误，排除下游阶段随机性；
  3. 排查顺序：**写入路径 id 域 → 下游阶段上下文 → 判定算法**（本例前两轮反着走，多耗一轮）。
- 交叉引用：接管类 mod 的下游阶段审计清单见 workflow-patterns 发现 #8（本例最终根因不是它，但 #8 仍是接管类 mod 的有效检查清单）。

---

## D 部分：workflow-patterns 发现 #8 状态更新（追加到该条目末尾，不改正文）

> **【2026-09-01 更新】** 发现 #8 的原始案例（M14「怪异城」）最终根因已由 M16 定案为 **JNI 写入路径 id 域错位**（见 compiler-idioms.md 发现 #6）——非 feature 阶段上下文问题。本条目的「审计清单」价值保留（下游阶段吃我什么状态的检查方法仍有效）；置信度从 candidate 降为「检查清单有效性：candidate（未在本例定案，但检查方法独立可复用）」。本例根因见 M16。

（应用说明：同时在 INDEX.md「工作流模式」行说明列「接管单阶段后的后续阶段上下文依赖（2026-08-31）」后追加「（本例根因后由 M16 定案为 id 域错位，见 compiler-idioms #6）」。）

---

## E 部分：judge 4 项 CONCERN 及处置（本节为应用说明，随主会话应用一并落盘）

| # | CONCERN | 处置（已体现于上） |
|---|---------|------------------|
| 1 | M14「feature 阶段上下文污染」方向性结论被推翻，discovered #8 与速查表 M14 行存在误导风险 | A 部分 M14 结案标注 + D 部分 #8 状态更新 + B 部分速查表 M16 行含排除链（定位作废、方法保留） |
| 2 | 82.16%/87.75% 与旧 96.06% 数字口径不同，直接对比会得出「对齐率暴跌」的错误结论 | M16 验证段显式「口径分流」声明（存档写入口径 vs 探针 raw id 域口径，禁止跨口径比较） |
| 3 | 验证覆盖只有 2 个单 chunk，非 Full | 显式 Partial 声明 + 残差归因（carvers/features 覆盖差 + cave_air/air 语义差）；Full 化列入下轮清单（批量 chunk 存档级回归） |
| 4 | overworld「零回归」从未在存档层验证（探针口径假象） | M16 教训 #4 沉淀 + NEXT_SESSION 下轮清单加入「overworld 存档层回归验证」 |

---

## F 部分：NEXT_SESSION.md 更新草稿（替换 §3 + 下轮工作清单）

（应用说明：整节替换 `### 3. 🔴 实机下界...` 与 `## 🟢 下轮工作清单`；其余节按最新状态核对后保留——本 subagent 只产出这两节替换文本。）

### 3. ✅ 实机下界「虚空+怪异城」——已结案（M16，待用户实机验收）
**真根因**：JNI 写入路径 block id 域错位——Rust buf = raw block id 域（与 blocks.json/参照导出同域），`CppBridge.writeChunk` 用 `Block.STATE_IDS.get(id)`（state id 域）错位解码（6a7337d 引入）。nether 存档 oak_leaves×3150+sapling+note_block = 错位解码签名；修复前重生成与实机存档错位数量完全一致（闭环判据）。上一轮「feature 阶段 biome 上下文污染」方向已被数据证伪（3×3 biome dump 纯 nether）；「Status 卡 biomes」亦证伪（Status 推进由包装层门控，fill 日志 status 是时点快照）。
**修复**：writeChunk 改回 raw id 域 `Registries.BLOCK.get(id).getDefaultState()`（快照 `.investigations/multiworld-port/snapshot-CppBridge-m16fix.java`）。
**验证**：Partial（单 chunk 存档级）——修复后存档 vs vanilla FULL 参照：nether(-5,-3) 82.16% / overworld(10,10) 87.75%，残差 = carvers/features 覆盖差 + cave_air/air 语义差。**口径注意：这是「存档写入」口径，与旧 96.06%「探针 raw id 域」口径不可比。**
**待办**：用户实机验收（`run_rust_client.ps1` 进下界：应无橡树叶/sapling 成片、无坠落）；Full 化批量 chunk 存档回归。

## 🟢 下轮工作清单（按优先级）

1. **用户实机验收 M16 修复**：下界无橡树叶/坠落；顺带确认 overworld 正常。
2. **overworld 存档层回归验证**（M16 教训 #4：6a7337d 后从未在存档层验证过，spawn 区被 vanilla 前代生成掩盖）——批量 chunk 存档级对比，顺带把 M16 验证 Full 化。
3. **bedrock 随机带**（123..126 微差 4011 块）：vertical_gradient 随机层 lerp 阈值精修。
4. **内存「差点爆」定位**：删档全量重生成的内存高峰 vs 泄漏（需任务管理器 java 提交大小数字）。
5. **jar 的 dll 同步核对**：latest jar 内 dll 是否含 M16 修复版本（02:03+ 判据需更新为最新构建时间）。
6. 知识库草稿应用：knowledge-drafts/draft-m16-id-domain-mismatch.md（本文档——台账 M16 + 速查表 + compiler-idioms #6 + workflow-patterns #8 更新 + INDEX + 本 NEXT_SESSION 替换）。

---

## 附：本草稿应用清单（主会话操作备忘）

1. `.investigations/multiworld-port/multiworld-errors.md`：M14 末尾结案标注 + M16 五段式插入（M15 后）+ 速查表加 M16 行。
2. `knowledge/discovered/compiler-idioms.md`：追加发现 #6。
3. `knowledge/discovered/workflow-patterns.md`：发现 #8 末尾追加更新块。
4. `knowledge/INDEX.md`：compiler-idioms 行与 workflow-patterns 行说明列更新。
5. `NEXT_SESSION.md`：§3 与下轮清单替换（F 部分）。
