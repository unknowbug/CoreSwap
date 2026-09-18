# B6-3「light 接管面读写声明表试点」收尾 judge 审查意见（260918-06）

- 审查角色：core.judge（subagent，只出意见，不改任何 status；confirmed 留人类）
- 审查对象：`declaration-light-260918-06.md`（draft/Degraded）+ scout 图 + 比对器 + 复用证据 pbeta05c.log + git 工作区 diff
- 三源基线：① 交付文件（.investigations/b63-260918-06/ 两份 + .tmp/260918-06/b63_comparator.py）② git 工作区 diff（三死开关删除 + check_switch_mapping.py 豁免 + docs 两处失效注记，逐块读过）③ 验证记录（本轮 judge 亲手复跑比对器正/负两条命令，见 N2）

---

## N1 声明表抽查（≥5 行一手源核对）——✅ 通过

抽样 9 个站点行，逐行对照一手源（本轮 judge 直读，非转述）：

| 抽查行 | 声明四要素（读/写/门控/默认方向）核对 | 结果 |
|---|---|---|
| mixin:77 `LIGHT_RUST`（五节总门行） | `System.getProperty("coreswap.light.rust") != null`，缺省关=vanilla | 一致 ✅ |
| mixin:133 + :684（blockabi / packed 默认路，F3） | :133 `!= null`；:684 `if (!LIGHT_BLOCKABI) packedLens = wgLightCollectPacked(...)` ⇒ 缺省=packed 路；:132 注释与消费点取反并存（F3(b) 解读成立） | 一致 ✅ |
| mixin:263-266 getSectionArray | 实为 :263-267（`secs == null || secs.length < 24` 回退在 :264-267）；语义一致，行号差 1 行内 | 一致 ✅ |
| mixin:698 betadump 双门 | :698 `if (LIGHT_BETADUMP != null)` 位于 :690 `if (LIGHT_BETAPROBE)` 块内 ⇒ 双探针门同开才写文件，声明正确 | 一致 ✅ |
| mixin:743-747 + :750-751（legacy 写回 + POST） | enqueueSectionData BLOCK/SKY ×24 + setLightOn(true) + wgReleaseLightTicket，:749 注释复刻原 light() 尾部 | 一致 ✅ |
| mixin:762-766（nativeDead） | `compareAndSet(false, true)` 一次性置位、无恢复路径，「无逆-显式声明」成立 | 一致 ✅ |
| mixin:641/:644（域批降级重放不可达） | :641 注释原文「状态回退，不可达路径，loud fail」——#98 自检第 1 条的源码自证属实 | 一致 ✅ |
| mod.rs:106-109 `from_json_file` | 只读 light_data.json（`std::fs::read_to_string`）；:122-133 的 "blocks" 键是 light_data.json 内部键，**不是 blocks.json 文件**——F4 消歧正确 | 一致 ✅ |
| BulkWb:90 | `WgCompat.flag("coreswap.bulkwb", WgCompat.BULKWB_ON)` 缺省开；:91-97 四诊断门 `!= null` | 一致 ✅ |

四列齐备性（读/写、门控、默认方向、前提）逐行成立；inverse 列第三列（C1 要求的四要素之外加列）亦齐。

## N2 判据核对——✅ 通过（比对器 judge 亲手复跑）

- **C1（表完整性）**：✅ 六节站点表 + 门控总表 + reads/writes 汇总 + inverse/前提列齐备；每个写站点带一手 file:line（抽查 N1 证实）。
- **C2（正/负向）**：✅
  - 正：`python .tmp\260918-06\b63_comparator.py .investigations\pbeta-260917-05\cmd-output\pbeta05c.log` → `[OK] 实际触达 ⊆ 声明`，**rc=0**；事件计数 `[LIGHT-PATH] enter ×1587 / abi=packed ×1587 / path=legacy-rust ×1587 / chunk( ×1587`，与任务书口径（path=legacy-rust ×1587 / abi=packed ×1587，3 boot）吻合，actual 8 keys 全部 ∈ declared 17 keys。
  - 负：judge 自行注入未知值（将 `path=legacy-rust` 全文替换为 `path=legacymystery` 复制件）→ `[FAIL] UNDECLARED: light:event.value:<unknown>:legacymystery`，**rc=1** ✅（判据：注入未知值 FAIL rc=1，达成）。
- **C3（#98「无对象」降级）**：✅ 合规——#102 terrain_cache 形态如实登记「无对象、不凑数」，且显式声明 #102 原文未读、不对其下判定（诚实声明 3 + §③ 尾段）；三条行级不可达标注（:644 源码自证 / bin-diag 结构性 / betadump 双门）均有一手依据，且正确区分「总门缺省关」与「门开后仍不可达」（§③ 第 4 条，避免全表凑数）。⚠️ 见条件 S4：C3 的「机械检出」能力本身未演示。

## N3 跨面裁决复核（BulkWb 不并入 light 域、归 R9-b）——⚠️ 有未登记义务，见 S3

裁决方向本身合理：BulkWb 是 bulk 写面（B6 其他试点），数据域（chunk.blocks section 原地替换）与 light 面（读 section 引用 :263）存在**共享 section 数组并发交叠**。但：
1. 声明表二节 BulkWb 行与六节 writes 汇总仍写「是否入 light 声明域**由主会话裁决**」——裁决结果（不并入、归 R9-b）**未回写**声明表，表文与已做裁决漂移。
2. 并发义务归属未登记：BulkWb SENTINEL（:96-100，260913-03 R9-b 加固）语义 = **writer×writer 在飞重叠检测**（per-chunk 单写者不变量），**不覆盖 light 收集线程作为 section 读方**的交错窗口。light 读（getSectionArray 活引用 + writePacket 帧/逐格读）× bulk 原地写并发下的义务（证明无交错 / 线程时序论证 / 归入 R9-b 判据面）目前**无归属行**——这正是本声明表范式（副作用成对登记 + 依赖声明）应当捕获的形态，恰好在本试点内被漏登记。

## N4 三源一致性——✅ 基本一致，一处符号级漂移见 S2

- 声明表 vs scout 图：F3/F4 消歧方向一致（scout 图标 F3 为「疑似反转待 fan-out」，声明表已按一手复核收敛为 F3(b)），门控总表 12 行逐行对齐；scout 图范围更正（mixin 实际路径）与声明表一致。
- 声明表 vs git diff：无矛盾——本块 git 变更全部在 B6-1 死开关面（biome6oct/cppNoBatch/fjp1 三行删 + 豁免 + docs 注记），不触碰 light 面任何声明行；record-260918-06.md 项 2/项 3 与 diff 一一对应。
- 比对器 ALIASES vs 声明表键集（抽 3 键）：
  - `light:chunk.blocks:r:Mixin#wgLightCollectPacked` ✅（两处一致）
  - `light:fs:light_data.json:r:mod.rs#lightInit` ✅（两处一致）
  - `path=domain` 别名键 ⚠️ **不一致**：声明表 §② 伪代码映射到 `wgLightDomainWriteBack`，比对器映射到 `wgLightLegacyTakeover` 三键——数据域相同（⊆ 判据不受影响，positive/negative 均按域判定），但符号级漂移会在未来 domain 臂证据触达时错误归户（见 S2）。

## N5 §9.7 口径——✅ 如实、无外推；一处表述缺位见 S5

- 比对器输出 note 显式声明「覆盖面 = legacy 臂 packed 路，声明表 §9.7 口径」；证据为 260917-05 归档采集复用（3 boot，n=1 载具），judge 复跑实测 1587×1587 与声明吻合。
- 声明表 §① 第 1/5 条明确「零运行验证」「默认方向 = 静态直读，运行时实际生效值未验证」——无「⊆ 成立 ⇒ 面声明完备」的外推 ✅（负向测试只证 UNDECLARED 维，覆盖不足不反向声称为完备，口径诚实）。
- 缺位：声明表正文无独立 §9.7 等价档位行（E1/E2/E3 + 载具 n 值），口径三要素只存在于比对器 note 与 §② 上下文——建议补一行表头声明（S5）。

## N6 停滞/历史核对（负向测试首轮假阴性错误链）——⚠️ 未落盘，见 S1

- 比对器现版含 `KNOWN_VALUES` 未知值检测（:59/:66-67，注释「未知事件值 = UNDECLARED（负向测试判据）」）——这是修复后的形态，判据有效（N2 负向实测 rc=1）。
- 但**首轮假阴性（未知值零触达 → 误判 PASS）的错误链在本块无任何落盘记录**：record-260918-06.md（4 项，无 b63 比对器段）、.investigations/b63-260918-06/（仅声明表 + scout 图两文件）、.tmp/260918-06/（仅 comparator + fake_with_unknown.log 注入件 + util_disasm.txt，注入件存在说明负测跑过，但无记录文件）。按项目「错误优先原则」（现象→根因→定位→修复→教训 五段式），此错误链 MUST 补录——「负向测试首轮假阴性」属高价值判错经验（与 knowledge #12/#27/#172 同家族：不能失败的 FAIL 判据 = silently-green 门的负测版）。

---

## 其他 judge 清单项快核

- status 合法：声明表 draft / scout draft，无 confirmed 越权 ✅。
- 产物契约：.investigations 落盘齐（draft 思维链载体合规）；.artifacts/index.yaml 无 b63 条目——试点 draft 阶段可接受，**confirmed 前若升 candidate 应补 .artifacts 登记**（建议项，不设条件）。
- 证据饱和 / 噪声卡 / 模块边界 / §9.8：本轮零运行验证（Degraded 全单声明），无 in-place 副作用新增（复用既有 pbeta05c 日志，只读）✅；比对器为 .tmp 一次性区 ✅；无跨模块 skill 正文引用 ✅。
- git diff 附带核（N4 之外）：check_switch_mapping.py 的 MC_ENGINE_CONSUMERS 豁免带一手核实指针（record 项 3 javap）✅；docs 两处失效注记与 build.gradle 删行同步 ✅。

---

## 总判定：**PASS-with-conditions**（5 项条件，均不推翻 C1/C2 结论）

- **S1（N6，MUST 补录）**：比对器负向测试首轮假阴性（未知值零触达误判 PASS）错误链未落盘——按错误优先原则在 .investigations 层补一段五段式记录（现象/根因/定位/修复/教训），并家族索引到 #172（silently-green 门的负测对偶）。
- **S2（N4）**：比对器 `path=domain` 别名键（wgLightLegacyTakeover）与声明表 §②（wgLightDomainWriteBack）符号级不一致——对齐为 DomainWriteBack 键（降级重放属另一事件形态，应单独别名），或在比对器加注说明。
- **S3（N3）**：BulkWb 裁决结果回写声明表（替换「由主会话裁决」字样）；显式登记「light 读 × bulk 原地写共享 section 数组」并发验证义务的归属（R9-b 判据面或 B6 后续块），注明 BulkWb SENTINEL 只覆盖 writer×writer、不含 light reader。
- **S4（C3）**：C3 按「判据存在、无凑数」支路成立，但「机械检出不可达前提」能力未演示（三条不可达标注为人工静态标注，比对器无对应检出线）——登记为下块待办（比对器增 `前提不可达` 机械判定或显式降级声明）。
- **S5（N5）**：声明表表头补一行 §9.7 口径声明（等价档位 E1 同构建态复用采集 / 载具 n=1 / 3 boot / 覆盖面=legacy 臂 packed 路），与比对器 note 对齐。

推荐状态：**保持 draft**——S1/S3 为收尾补录动作，完成并经用户复核后可建议升 candidate；confirmed 留用户拍板。
