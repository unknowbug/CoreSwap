# judge 审查意见 — 工作块 260913-06（BULKWB 翻转后 nt/en 默认路径行为门 + sentinel 跨 region）

审查角色：core.judge（隔离 subagent，只读）。审查对象：`record-260913-06.md` + cmd-output/ 归档件 + .tmp 运行台脚本 + d4i-260913-01 基线 MANIFEST。本意见不改变任何 status；confirmed 留给人类。

## 逐项判定

### J1（#141 两断言独立取证 + 排序域口径）— PASS
- 断言①（路径切换自证）：归档日志逐字核实。`1216-postflip-nt-default.log:3240-3241` = `[WG-BULKWB] calls=625` / `[WG-PERBLOCK] calls=0`；en 臂同（:3240-3241）。正/负成对、独立于断言②。seed=417950215108767439 与记录一致（log:92）。
- 断言②（产物不变）：fp 三层全 64 位 sha + 尺寸，三方一致（record 表 ↔ fp-sha-260913-06.txt ↔ MANIFEST-sha256-260913-06.txt 第 5-10 行）。
- 排序域声明：record §P2 明示「§9.7 同载体同 seed 同 region 排序域」，且运行台 `Sort-Object` 落盘（run_1216_arm:145-147）与声明一致，无夸大。end 维确定性指纹（#21）论证成立。

### J2（P2 结论 + 蕴含链完整性）— PASS
- 「逐字节同一」对照 d4i-260913-01 MANIFEST 属实：nt 三件 `df9f56a9…/86237B`、`4acf453e…/35951B`、`4f02091a…/50286B` = MANIFEST 第 40-42 行 fp-1.21.6-nt-default（亦=nt-bulk/nt-perblock）；en 三件 = 第 31-33 行 fp-1.21.6-en-default。
- 蕴含链：overworld 断言①由 260913-01 postflip-ow-default（confirmed 前块）承载 + 本块 nt/en 断言①实测 → 三维路径切换齐备；断言②三维 fp 全同 → 「翻转后 nt/en 默认路径 = bulk 且产物不变」成立。record §覆盖面已声明 P2 只覆盖 1.21.6、ow 未复跑——边界诚实。
- 证据链小缺口见「条件 C1」：[result] 汇总行未归档，content=625/wb=625/intercept=625/carrier=OK/零 throws/零 EntryMissing/boot 36.2s/gen 10s/FINISHED 无法从归档件逐字复现（calls=625/perblock=0 与 fp 可复现，为主要判据，故不降级）。

### J3（P3 四臂判据 + region/forceload 处理）— PASS
- 四臂 armed=1 全部逐字核实（各 r1/r2 log 的 `[WG-BULKWB-SENTINEL] armed:` 行）；`Accessing chunk sections from multiple threads`（crash 判据）四臂 0 命中。
- wb 行数直数复现：1201-r1=576、1201-r2=529、1216-r1=578−2(armed/init 非计数行,实=576)、1216-r2=531−2(=529)——与 record 表一致（对 1216 两臂，armed 行恰含 "WG-CONTENT-WB" 无关…注：armed 行不含该子串，578/531 计数含 init 行系 grep 模式所致；记录值 576/529 与 WB 行数吻合）。
- region 坐标核实：r1 chunk 124-143 邻域 ↔ [2048,2303]²/256 chunks；r2 chunk -196..-178 邻域 ↔ [-3072,-2848]²/15×15=225。r2 首跑 289>256 失败→缩 225 重跑已如实记录（record §过程错误 1），重跑后判据全过，不影响结论。
- 疑点见「条件 C2」：suspExc=3 无法从四份归档 log 复现（`Exception|Mixin apply failed|NoClassDefFoundError` 四臂均 0 命中）。

### J4（§9.7 覆盖面边界声明）— PASS
- 执行体范围（P2 仅 1.21.6）、region 范围（每组合 2 region、不声称全 region）、seed 范围（未测更多 seed）、排序域（#142 口径窄化、原生顺序不在判据域）四要素均显式声明。充分。

### J5（两处过程错误五段式）— PASS（附小缺口）
- 错误 1（forceload 289>256）：现象/根因（#20 死参数家族）/定位（零 armed=驱动未生效签名）/修复（225 重跑）/教训（前置核算）五段齐备。
- 错误 2（脚本两次语法翻车 + `[scriptblock]::Create` 吞错假 OK）：五段齐备，`Language.Parser::ParseFile` 实核的教训可复用。
- 小缺口：289 失败首跑的原始日志/转录未归档（叙述记录，无证据件）；建议后续失败跑也留档（SHOULD，不阻断）。

### J6（脚本变更登记 vs 实际 diff）— PASS
- `run_1216_arm_260913-06.ps1` vs `d4i-260913-01/run_1216_arm_v2.ps1`：逐行核对，diff 恰为登记的 4 条（out 目录 / 默认 Props=-Dcoreswap.bulkwblog=1 / fp 全 64 位 sha+尺寸 / fp-sha 文件追加），无未登记改动。
- `run_sentinel_{1201,1216}_260913-06.ps1` vs `sentinel-260913-05/` 母本：diff 恰为登记的 2 条（out 目录 / +RTag 入标签与文件名），无未登记改动。

## 总 verdict：PASS-with-conditions

推荐：record 可升 candidate（核心判据 chains 全部三源核实）；满足以下条件后交用户 confirmed：
- **C1（MUST）**：补归档 P2 两臂 + P3 四臂的 `[result]` 汇总转录（run_1216_arm 的 content/wb/intercept/carrier/exceptions/boot/gen/FINISHED 行与 sentinel 的 armed/wb/suspExc 行）到 cmd-output/，或在 record 中改写为仅引用可复现的归档行。record 产物清单现声称「arm 汇总（job 输出转录 cmd-output/）」与实际归档内容不符。
- **C2（MUST，澄清或修正）**：P3 「suspExc=3/4 均 WMI/COM 良性」无法从四份归档 log 复现（0 命中）。请指认来源（哪一臂哪几行）或修正记录（若实为 0，直接写 0 更干净）；不影响 armed/wb/crash 主判据。
- **C3（SHOULD）**：289 失败首跑留原始证据件；MANIFEST-260913-06 可补 3 个 .ps1 的 sha 以便 J6 类核对免人工 diff。

## 无法核查的面（显式声明）
- git HEAD/工作区 diff（无 shell 权限）——三源核对中「git 源」未做，仅做了快照 + 日志 + 前块基线两源半。
- 运行时行为本身（日志只读转述；未重放任何 server 运行）。
- dll 盘上 hash 实核（record 声称「盘上 hash 实核」，judge 无 shell 无法重算；in-log `sha256=abd7d889…` 自证已核实于两份 P2 log:93）。
- .tmp 归档件与 cmd-output 归档件逐字节一致性（未重算 sha；MANIFEST 自洽性已核）。
