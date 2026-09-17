---
status: review-only
judge: subagent
block: 260917-01
---

# judge 收尾 MUST 审查意见 — G3 漂移基底归因（260917-01）

> 审查角色：core.judge（隔离子进程）。只出意见，不改任何 status；confirmed 留人类。
> 三源核对基线：① 产物快照（criteria / record / errors / candidates×3 / cb1 / cb2 / control，全部读）② git 工作区 `git status --short`（2026-09-17 实测）③ 验证记录（cmd-output/ G17/G17b/G17c 七轮日志 + changed_set json 逐项重数）。

## 总结论：**PASS-with-conditions**

骨架合格：预登记纪律整体得到执行（含 E1 VOID 不挑臂的自证硬门生效）、三源数据互相一致（全部抽验重数吻合）、worker 诚实声明充分、fan-out 辖区划分清晰。但 **C-B4 追加预登记缺独立时序锚**（C1）与**取代记录措辞未内嵌 n=1 限定**（C2）两点须在拍板/落盘前补齐；另有两处 CONCERN 级提示（K1/K2）不阻塞。

---

## 重点审查项逐条意见

### J1 预登记合规性 — 通过，附条件 C1

- **C-B1 执行合规**：rL 读法与写死阈值一致——cb1 json 实测 rL=99.17%（G17b-L60 set 与 120 基底交集 119/120，逐项核对），落 ≥80% 分支 → PERSISTENT，无灰区，与 record §2 / .b1b4 §1 一致。自证硬门（done/lightInit/sha/hook）逐臂在日志中核实（见 J5/验证记录节）。
- **C-B2 执行合规**：字面「不等分支」确实命中（28/44，逐 chunk 清单在 cb2 json），worker 按校准收窄而非改判（见 J3），判据原文未被回改。
- **C-B4 追加预登记**：内容上判据读法写死（阈值 ≤20/≥60、交集 ≤20%/≥60%、灰区归属），且文字内嵌「本轮已验证 -PlightRust 必带，E1 教训」→ 内部证据表明其定稿晚于 E1（G17 VOID 判定）、先于 G17c run3 采集（「G17b 两臂的 world 已在 run2 后保留」的引用形态），叙事时序自洽。
- **条件 C1（时序合法性证据锚缺失）**：该目录整体 untracked（git status `??`），C-B4 追加段**无任何独立时序锚**（无 git 提交、文件无时间戳记录）。当前只能凭文本内部引用自证「追加先于采集」。**要求**：在取代记录/本目录 INDEX 说明中补一行时序锚（criteria 文件定稿时刻或首个 run3 产物时刻的对照记录）；后续追加预登记 MUST 在追加时留下可独立核验的时间标记（git add 或文件头写 Get-Date 实测时刻）。

### J2 C-B4「两臂分裂」逐臂展开 — 通过（展开未越出实质边界），附措辞条件

- 判据文字确实按「两臂共同行为」写，未预登记分裂分支——worker 自我标注准确。
- 逐臂展开审查：legacy 臂 200 changed ≥60 → 命中 M-b 分支第一支（「或」联结，48.3% 交集本落灰区段被「或」吸收，不单独结论——处理正确）；domain 臂 2 changed ≤20 且交集 1/120 ≤20% → 命中 M-a 分支。**展开未新增阈值、未改写分支语义、灰区条款未被绕过**——属于对未预登记情形的保守自然展开，非挑臂（两臂各自独立过自证硬门，数据均采信）。
- **条件 C2a**：该展开是事后裁量，取代记录与任何后续引用 MUST 显式携带「判据未预登记分裂分支，逐臂读法为事后裁量（worker/judge 均已复核）」一行——worker 已建议，本 judge 认可为必选而非可选。

### J3 .b2 §9.7 收窄 — 通过

- 逻辑成立：stable 对照实测 136/1861 = 7.3% 非零地板 → 「任一不等」分支在任何组必然触发，字面判据对本数据无检验力——这是**对照缺失下的假阳性读法识别**，收窄（系统性偏离成立、域批独有分量无证据 p≈0.22）是合法降级而非改判。§9.7 三要素（载体/覆盖面/可比性）齐备。
- worker 主动声明「7.3% 是跨臂噪声地板，≠ 单臂 run-to-run 基线（#111）」——口径区分到位。
- 小提示（不阻塞）：63.6% vs 52.5% 的 Fisher p≈0.22 在 n=44/120 下检验力有限，「无显著」宜读作「无证据」而非「无差异」——.b2 §6 措辞「域批独有分量无证据」已正确取此读法，保持即可。

### J4 §15.4 取代记录措辞 — 通过（对象指认准确），附条件 C2b

- **被取代对象指认准确**：① 260915-03「G3 drift 由时机形态主导」——C-B1 rL=99.2% 直接证伪「快照窗/停服时机」轴（60s 与 20s changed 集基本不变），推翻成立；② 260916-01 record §4 发现 1 的「快照窗完成时序边界」**候选机制**——同证据链证伪，且原文以「候选」身份存在、连带回炉定位准确（原结论不删不改、双指针登记符合 §15.4 形态）。
- **新结论证据边界**：「legacy per-chunk 路径轮次级不收敛 / 域批一次重载到不动点 / 现象为光照接管特有」——核心事实（120 基底复现 99.2%、run3 分裂 200 vs 2、spawn 箱 100% 聚集、vanilla 臂 0.8%）均有落盘数据支撑（本 judge 已重数）。但 **n=1、单 seed、单 dll、单载具、run3「逐轮换血」仅一个样本**——worker 已在 §6/§7 诚实标注且 candidate 封顶。
- **条件 C2b**：.b1b4 §4 建议措辞的「新结论」行未内嵌上述限定——限定只存在于草稿 §6/§7，若按现措辞直接誊入取代记录，读者在引用点看不到边界。**要求**：取代记录正式文本的新结论行 MUST 内嵌「（n=1 单 seed 单载具，candidate）」限定；「5×5 全帧重算覆盖陈旧历史」等机制推演（无直接探针）MUST 不进入取代记录正文（可留在候选草稿）。

### J5 E1 教训处置 — 通过

- **VOID 处置合规**：G17 两臂 lightInit=0、零 [LightRust] 行（G17-L60.log / G17-D60.log 实测核对，两臂均无 lightInit/hook 行）→ 按 #118 硬门整臂 VOID，未挑臂；换标签 G17b-* 重跑并显式补 -PlightRust（G17b 日志 lightInit ok ×2 + hook armed 核实）；根因链（build.gradle `findProperty('lightRust')` → `-Dcoreswap.light.rust=1`，本 judge 实测命中）与 #90/#8 家族归类合理。
- **VOID 数据作旁证合法**：口径已声明（vanilla 光照、60s settle、单对照、n=1、跨形态复用是近似——.b1b4 §1/§7.6、errors E1 教训③ 均有标注），仅作「基底为光照接管特有」的方向性旁证，未进入任何判据读法。合法。
- 提示：E1 属于「复测口径失传」新形态（#90 家族变体），知识库落盘时建议单列该形态（待办已在 record §6，此处确认价值门通过）。

### J6 .b3 机制候选静态推演 — 通过（抽验全部相符）

一手源码抽验（工作树 = 未提交 CP-1 改动版，与 .b3 核对时同源）：
- **mixin :211-216**：空节 `sec.isEmpty()` → `Arrays.fill(blocks9, k, k+4096, 0)` + continue——**确实无打点、无计数、不回退**（β「完全静默」描述准确）。
- **mixin :637-657**：collect 失败 → `wgLightFallback("neighbor-missing-or-height")` + return false；rc≠0 → fallback——失败即回退、不 cancel（cir 未置 → vanilla light() 续走），F2 相符。
- **mixin :677-683**：native throw → fallback + return false，相符；:694-697 LIGHT_DOMAIN 预检失败 → WG_DOMAIN_INLINE 计数后走内联路径，与 §2「回退面结构不同」相符。
- **mod.rs:182-192**：`light_compute(engine, blocks9, out_block, out_sky, out_flags)`——签名确实**无任何 stored light 输入**，scratch 为 per-call 局部（F1 在签名层成立，.b3 也诚实限定「签名级一手事实」）。
- M-b 结构性削弱（F1 排除 Rust 内核循环依赖、限定到 vanilla 回退子路径）逻辑自洽；M-c 定位为空间定位层、不越权解释逐 run 变异——候选树纪律良好。@anchor.idk ×2 使用合规（具体、指明影响面）。

### J7 CP-1 决策输入中立性 — 通过

- .b3 §3 明确「决策输入非决定」，R1（载体换轨）/ R2（载体换型）双方向并列、各附前置；「无论 R1/R2：α/β 通道修复是前置」的独立约束与 CP-1 去留解耦——未隐性拍板。
- 保留 .b1 收益面（§3.1）与转其他候选的前置（§3.3）双向陈述对称；「收敛实证目前只属于 domain 形态」是实测事实陈述而非推荐。
- C-1 重述两方向（R1 噪声地板带 + domain 载体 / R2 确定性 dump 换型）与证据相容：2.5% 阈值在 legacy 臂结构性不可达（5.93→6.27→9.88 逐轮上升，三轮数据在案）成立；R1 依赖的「噪声地板带」尚无接管形态同配置实测（#111，.b1b4 §7.6 已声明）——**采纳 R1 前须补接管形态噪声锚实测**，此为决策时输入而非本块缺陷。

### J8 数字一致性 — 通过（抽验 6 处，全部吻合）

1. rL 99.2%：G17b-L60 set ∩ 120 基底 = 119 → 119/120 = 99.17% ✓（cb1 json 与 record/.b1b4 一致）。
2. cb2 三组：28/44=63.6%、63/120=52.5%、136/1861=7.3% ✓（cb2 json = control json = .b2 §1 = record §2 四处一致）。
3. changed 计数逐文件重数：G17-L60=17、G17-D60=16、G17b-L60=127、G17b-D60=166、G17c-L3=200、G17c-D3=2 ✓（changed_set json 逐一 ConvertFrom-Json 重数）。
4. 百分比对 2025（45×45）换算：5.93/8.10/6.27/8.20/9.88/0.10/0.8% 全部 ✓。
5. run3 基底交集：G17c-L3 base_intersection 重数 = 58 → 58/120 = 48.3% ✓；G17c-D3 交集 1 ≤20% ✓。
6. 域臂 timeout：G17b-D60.log 末值 timeout=23、G17c-D3 timeout=23/degraded=0，与 record 补充数据行一致 ✓。
- 验证记录抽验：G17 两臂日志 0 lightInit/0 hook（VOID 实证）；G17b-L60/G17b-D60/G17c-L3/G17c-D3 均有 `lightInit ok`、sha256=6f7fa3ae0169fb5c…、Done 计数与采集矩阵吻合（G17 两臂各 2 boot、G17b 两臂各 2 boot、G17c 各 1 boot）；G17b/G17c domain 臂 `[LIGHT-DOMAIN] hook armed` + sealed≥1 ✓；G17 两臂 fallback=0（无 fallback 行）✓。执行体三元组自证在全臂成立。

## 三源核对与框架纪律

- **git 工作区**：`status --short` = 6 个已修改实现文件（CppWorldgen / build.gradle / ServerLightingProviderMixin / jni_bridge.rs / light/mod.rs / LightDomainBatch 新增）+ 3 个 untracked 调查目录。**与「本块零实现改动」声称相容**：无任何文件能归属到 260917-01 本块（本块产物全部在 .investigations/g3-drift-basis-260917-01/ 与 .tmp/cp1-260917-01/，后者 untracked 不入库，record §5 已声明）；已修改实现文件均属 260916-01 CP-1 遗留工作树，且是本块全部探针与静态核对的一手源（mixin/mod.rs 行号抽验基于同一工作树，源一致）。注意：CP-1 遗留工作树未提交，本块结论对其有因果依赖（dll 6F7FA3AE 即其构建产物），提交时须一并处置（record §6 待办已列）。
- **产物契约**：本块结论尚在 .investigations/（过程域），无 .artifacts/ 结论性产物——当前处于 draft 收尾审查点，合规；升级 candidate/落结论时须按 core.artifact 登记 + 知识库更新走 subagent 草稿制（record §6 已预置）。
- **retry cap / 噪声卡**：无超限迹象（E1 一轮定位修复即回数据层；E2 属工具缺陷非验证轮）；引用 #49/#51/#65-#67/#80/#90/#111/#118/#144/#146 等历史模式均标注，无未解决噪声卡阻塞本块。
- **status 纪律**：全部产物 draft；.b1b4 出现「candidate 级」字样均为**推荐提升**措辞且注明待 judge+用户——无违规自授。本 judge 建议：.b2 收窄结论与 .b1b4 综合（含 C2a/C2b 修订后）可推荐 candidate；三机制候选（M-a'-α/β、M-c）保持 draft 待 P-α/P-β/P-path 探针链。

## 编号条件与 CONCERN 汇总

- **C1（条件）**：C-B4 追加预登记补独立时序锚；后续追加预登记 MUST 留可核验时间标记。
- **C2a（条件）**：取代记录/引用 MUST 携带「分裂分支为事后逐臂裁量」声明行。
- **C2b（条件）**：取代记录正式文本新结论行 MUST 内嵌 n=1/单 seed/单载具 + candidate 限定；机制推演（5×5 覆盖等）不得进入取代记录正文。
- **K1（CONCERN，不阻塞）**：run3 判据数字（200/2）目前只存在于 changed_set3.json（其内含 changed3 列表，数据完好），但 record §1 所述「离线补算」的计算过程无独立落盘脚本输出可溯源——建议把补算脚本或其输出摘要补入 cmd-output/（原始 light_after3 快照在案，可复算，风险低）。
- **K2（CONCERN，不阻塞）**：VOID 0.8% 噪声锚与 stable 7.3% 跨臂地板均为近似口径（vanilla/跨臂，非接管形态单臂基线）——R1 方向采纳前 MUST 补接管形态同配置噪声锚实测（.b1b4 §7.6/#111 已自declared，此处升为决策前置提醒）。

## 推荐状态

- criteria / record / errors / 三份 candidate 草稿：**保持 draft → 建议 candidate**（C1/C2a/C2b 修订完成后）。
- 260915-03 §15.4 取代记录：**措辞修订（C2a/C2b）后可提交用户拍板**；confirmed 一律留人类。
