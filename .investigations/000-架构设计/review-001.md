# Review-001 — core.judge Phase 3 审查：8576 主线「placeBadlandsPillar 修复」

> 审查角色：core.judge（subagent 隔离执行）。只出审查意见，不改产物 status；confirmed 只能由人类授予。
> 审查对象：surface.h placeBadlandsPillar 代码改动 + `.artifacts/8576-finaldensity/` 六件产物 + `.artifacts/index.yaml` + block_probe 验证证据。
> 审查日期：与 8576-finaldensity 产物同期。审查结论落盘本文件。

---

## 审查对象清单（复核到位）

| 对象 | 路径 | 状态 |
|---|---|---|
| 代码改动 | `versions/1.20.1/cpp/worldgen/src/surface.h` L443-445（声明）、L701-726（调用时序）、L782-819（实现）+ @anchor.test L783 | 已复核 |
| worker 产物 | `.artifacts/8576-finaldensity/` 6 件（beardifier-analysis / blender-analysis / diag810.patch / pillar-impl / surfacebuilder-analysis / 3200-degradation） | 已读全文 |
| 产物契约 | `.artifacts/index.yaml`（6 个 8576 条目全在册） | 已复核 |
| 验证证据 | `E:\tmp\bp_8576_after_pillar.txt`（修复后 99.9993% / 24 mismatch）、`E:\tmp\bp_8576_rows.csv`（修复前 820 行 mismatch 明细）、`E:\tmp\bp_3200_clean_test.txt`（99.9997% / 4 mismatch）、`E:\tmp\bp_3200_8576seed.txt`（99.9995% / 差 8）、`E:\tmp\bp_3200_after_pillar.txt`（89.8942%，与 base 完全一致） | 已复核 |
| Java 源码 | `E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\gen\surfacebuilder\SurfaceBuilder.java` L113-131/L208-234、`MaterialRules.java` L541-565、`VanillaSurfaceRules.java` L206-237、`NoiseParametersKeys.java` L54-56 | 已逐行交叉核对 |
| 协议 | `protocol/verification-protocol.md` §1（@anchor.test source 格式/验证载体要求） | 已复核 |

---

## 逐项审查结论

### 1. 证据完整性（@anchor.test source 可复现性）— ✅ 通过（附小瑕疵）

- `source="probe:block_probe!PILLAR#001"` 格式符合 verification-protocol §1（`<载体>:<工具>!<条目>#<序号>`），与既有 AQF#001-004 / SURF#001-002 / FLATCACHE#004 同风格。
- **可复现**：block_probe 以 `seed=8576294172403134396 + size=6 + origin=(720,-432)` 运行，参照 `versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks`（存在，mtime 8/7 23:23）；输出 `bp_8576_after_pillar.txt` 头完整记录 blocks 文件头（seed/size/origin/minY/height），满足「seed+坐标+参照」三要素。
- **修复前基线**：任务所称 `E:\tmp\bp_8576_full.txt`（99.9768%/820 mismatch）**文件缺失**；但 `E:\tmp\bp_8576_rows.csv`（恰 820 行 mismatch 明细，含代表列 (810,-411) y=65-118 约百行 terracotta 差异）可作基线明细，99.9768% 有 `docs/06-surface-rules.md:111`、`docs/10-timewise-archive.md:693` 双文档佐证。**基线可复核，不构成不可复现**。
- 瑕疵（建议级）：PILLAR#001 验证走的是 block_probe 常规逐位对比，无 pillar 专用追踪模式；输出文件名未显式锚定条目号。建议未来在 block_probe 增加 pillar 触发追踪（列 dump）并将 `PILLAR#001` 写入输出注释，便于精确回归。

### 2. 置信度合法 — ✅ 通过

- index.yaml 中 6 个 8576 产物 status 全部为 `candidate`；未发现非人类授予的 confirmed；无 status 违规。

### 3. 产物契约 — ✅ 通过

- 6 件产物全部落盘且全部在 `.artifacts/index.yaml` 注册（id 均带 `re-code:8576-finaldensity:` 前缀，path 正确）。
- 注（决策开放）：本审查文件 review-001.md 未入 index——审查产物是否入契约由 maintainer 定，不阻塞。

### 4. 噪声卡历史 — ⚠️ 部分未解决（建议级，非 blocker）

- **已关闭**：「8576 剩余 826 块 = terracotta 带边缘」（`06-surface-rules.md:131`）→ pillar 修复后代表列 (810,-411) 修复前约百行 mismatch（csv L47-809）→ 修复后 0 行，chunk(50,-26) 仅剩 1 处 savanna 差异。目标卡关闭。
- **仍开放（正交）**：「参照深层 terracotta 带（y=-32 单层/带）来源未明，假 diff 候选」（`06-surface-rules.md:132`、`10-timewise-archive.md:500-504`）。本次产物未声明关闭；但修复后 24 mismatch 中无 y=-32 terracotta 差异 → 该卡在当前参照下已无 observable mismatch。**建议后续归档显式关闭或标注复核结果**。
- **新增残留（未立项）**：24 mismatch 中 2 块 biome=forest 的 vanilla=494 terracotta（pos(812,73,-337)/(815,89,-337)，C++ stone/grass）、~20 块 savanna 水/深板岩/表层差异、river 及洞穴类各 1-3 块——均在 pillar 声明范围外，产物诚实标注「不在本改动范围」。建议下一个 8576 卡从这些立项。

### 5. retry cap — ✅ 通过（不超限，附观察）

- 目标「8576 terracotta 带」的假设演进链：est 边界（晚6，修复后 8576 略降 99.60→99.576，关闭）→ finalDensity 微差（0.06@y60，组件差定位）→ squeeze/densityBuf（WG_SURFDUMP 验证 C++ 自洽，关闭）→ Beardifier（否证）→ Blender（否证）→ pillar（实现+验证）。
- 6 个候选均为**不同具体假设**，每个在被源码证据否证/关闭后才进入下一个；无同一假设反复 Lift→Verify 的情况，单候选验证次数 ≤2。**≤3 上限未超**。
- 观察（建议级）：候选链 6 跳偏长。建议对同一目标立项前先做候选收敛评审（并列候选一次验证多个），降低链式推进开销。

### 6. 模块边界 — ✅ 通过

- 产物引用范围：mc_src_extract Java 源码（SurfaceBuilder/MaterialRules/VanillaSurfaceRules/Blender/ChunkNoiseSampler/StructureWeightSampler/NoiseParametersKeys）、项目 docs（06/10-timewise）、diag810.patch、block_probe 输出。grep 产物目录无 skill/模块 正文引用。边界干净。

### 7. 代码正确性（judge 附加）— ✅ 通过（与 Java 源码逐位一致）

对 surface.h L782-819 与 `SurfaceBuilder.java` L208-234、buildSurface L113-131 逐行核对：

| 项 | Java（行号） | C++（surface.h） | 结论 |
|---|---|---|---|
| e 公式 | L210 `min(\|badlands_surface(x,0,z)*8.25\|, badlands_pillar(x*.2,0,z*.2)*15.0)`（pillar 项无 abs） | L791-792 | ✅ 一致 |
| h/i/j | L214-216 `h=\|roof(x*.75,z*.75)*1.5\|; i=64+min(e²*2.5, ceil(h*50)+24); j=floor(i)` | L795-798 `(int)std::floor` | ✅ 一致 |
| 触发判定 | L211 `!(e<=0)`、L217 `surfaceY<=j` | L793、L799（取反早退，等价） | ✅ 一致 |
| 校验循环 | L218-227 stone break / water return | L802-806 | ✅ 一致 |
| 填充循环 | L229-231 `while air → setState(defaultState)` | L809-816 | ✅ 一致 |
| 越界语义 | getState 越界→AIR、setState 越界无效 | L803/L810 读 AIR、L812 跳过写 | ✅ 一致 |
| heightmap 抬升 | setState→trackUpdate 首填 y=j→j+1 | L818 `max(columnHeight,j+1)`（filled 前提） | ✅ 一致 |
| 时序 | L117 o=heightmap+1 → L119 biome 采样 → L121 pillar → L124 p 重采样 → L131 主循环 | L707/L711/L714/L717/L726 | ✅ 一致 |
| 噪声参数 | NoiseParametersKeys：PILLAR -2{1,1,1,1}、PILLAR_ROOF -8{1}、SURFACE -6{1,1,1} | worldgen_api.cpp L112-114 | ✅ 一致 |

**行为级验证（最强证据）**：修复后 6×6 区域 TOTAL 99.9993%（3538920/3538944）、24 mismatch；代表列 (810,-411) 修复前约百行 terracotta mismatch → 修复后 0；chunk(50,-26) 从 820 块量级收敛到 1 块（savanna 邻域列，非 pillar 列）。3200 无退化（`bp_3200_clean_test.txt` 99.9997%；`bp_3200_after_pillar.txt` 89.8942% 与 base 完全相同 = pillar 对该参照零数值影响）。

**次要差异（建议级，非 blocker）**：`SteepCond`（surface.h L250-258）读 `*ctx.columnHeightmap`（buildSurface 传入的 pillar 前 const 快照）；Java `SteepSlopePredicate`（MaterialRules.java L553-561）在规则求值时**实时** `chunk.sampleHeightmap`（pillar 抬升后）。badlands 段规则树不含 steep（VanillaSurfaceRules L206-237 无 steepSlope）→ 对 pillar 目标零影响；但非 badlands 邻域列（若走含 steep 的段）存在理论差异。当前 24 mismatch 无此形态。**建议列为后续关闭项**。

**编译状态**：pillar-impl.md 注明 worker 环境无法编译（权限拦截）；父代理后续已编译+运行（bp_8576_after_pillar 即 pillar 版 block_probe 输出），编译缺口已由验证证据填补。

---

## 推荐状态

| 产物 | 当前 | 推荐 |
|---|---|---|
| 全部 6 件（surfacebuilder-analysis / beardifier-analysis / blender-analysis / 3200-degradation / pillar-impl / diag810.patch） | candidate | **保持 candidate**（不写 confirmed） |
| 补充建议 | — | surfacebuilder-analysis、3200-degradation、beardifier-analysis、blender-analysis 证据已闭环，**建议用户考虑授予 confirmed**（尤其 3200-degradation 已由 bp_3200_8576seed=99.9995% + bp_3200_clean_test=99.9997% 实证闭环） |
| 代码改动 | — | 建议用户拍板是否合入（本审查不代劳） |

---

## 阻塞项

**无硬阻塞项。**

## 建议下一步（按优先级）

1. **基线文件补齐**：`E:\tmp\bp_8576_full.txt` 缺失，归档时用 `bp_8576_rows.csv`（820 行）+ docs 数值锚定「修复前 99.9768%/820 mismatch」，避免基线口径漂移。
2. **显式关闭 y=-32 深层 terracotta 噪声卡**（`06-surface-rules.md:132`）：当前参照下已无 observable mismatch，建议标注「复核：已被更早修复消除或属假 diff」。
3. **SteepCond 与 Java 实时 heightmap 对齐**（surface.h L250-258 ↔ MaterialRules L553-561）：理论差异项，纳入后续关闭清单。
4. **剩余 24 mismatch 立项**：forest terracotta ×2、savanna 水/深板岩 ~20、river/洞穴类，作为下一个 8576 噪声卡候选（并套用「候选收敛评审」）。
5. **block_probe 增加 PILLAR 专用追踪**（pillar 列 dump + 输出锚定 `PILLAR#001`），提升未来回归精度。
6. **决策开放项**：review-001 是否入 index.yaml 由 maintainer 定。

---

*审查声明：本文件仅记录审查意见与建议，不构成对任何产物 status 的授予/变更；confirmed 只能由人类（用户）授予。*
