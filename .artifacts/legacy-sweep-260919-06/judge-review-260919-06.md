# judge-review-260919-06 — 旧遗留清单 8 项 sweep 收尾交付审查意见

- 角色: core.judge（subagent 隔离；只出意见，不改任何 status；confirmed 留人类）
- 审查对象: 260919-06 块收尾交付（record-lowcost-260919-06.md + 全部 worker 件 + 判据/采集件）
- 三源核对: ① 产物快照（.artifacts + .investigations + .tmp，逐件实读）② git status/diff（.artifacts/index.yaml +10 行 = 仅 c1a-260919-05 条目；proposal 状态行 +4/-1；本块新目录均未入库，以文件为准——与用户预告一致）③ 验证记录（SELFCERT result.json ×2、legacy-t7-1.log 实 grep、criteria mtime 序、judge 独立复算脚本 judge-recompute.py）
- 日期: 260919-06（宿主锚）

## 总评

**PASS-with-conditions（N1–N5 binding 建议全部处理；N6–N7 登记即可）。** 逐项证据链与结论匹配，置信度标注全部合法（本块无 confirmed 越权；Degraded/机制方向/单臂 n=1 边界声明均在位）。T8（最重项）程序合规核心点全部核过：判据先于采集（mtime 18:53:24 < 采集 18:59/19:14）、预登记读法被机械执行（无事后挑读法）、双臂自证/负自证成立（result.json 实读）、**judge 独立复算 diff = 4223 / 44.6878% 与交付值逐位吻合**。最重问题 = N1：本块产物未登记进 .artifacts/index.yaml（契约缺口，非结论缺陷）。

## 逐项结论

| 项 | 结论 | 依据（judge 一手核对） |
|---|---|---|
| T1 R4 上报 | **PASS** | proposal 状态行实读 = submitted（含转交前 sha 复核 299598f46ebe1269 + 上游 eeebff9）；git diff 实证该文件为本块修改。流程性动作，无需判据，合规 |
| T2 Scope B/C | **PASS** | worker draft 在位（Degraded 声明 ✓）；backlog-map §2 引用的原定义 :82-84 三行逐字命中；用户已拍板挂起（HOOK-2），技术面结论不被本块 status 化。diffs.txt 计数 judge 未全量复算（见 N6） |
| T3 B3 余额 | **PASS** | worker draft 在位；三态表内部自洽（2 缺/4 顺带/3 作废 = 6+3 全覆盖）；用户已拍板续挂。同 N6 抽查边界 |
| T4 260914-02 open①③ | **PASS-with-concerns** | 脚本实读：正常路径唯一 stop（:58）+ WaitForExit（:62）+ refusedCount（:61）；pickHit 双重重置已删（:70-77 注释在位）；FAIL 分支 stop（:43）保留（服务器在跑，合法）。open② 按批准范围续挂 ✓。风险见 N3 |
| T5 形态审计后续 | **PASS** | 零漂移核对转抄 = 合理（backlog-map §8 已给一手 file:line）；CP-5 参数表抽查：api.rs adaptive_threads 实读 = `(logical/2).max(1) − 2 再 clamp`（A1d 注释在位）、mixin resolveMaxInflight :114-115 实读 = `max(1, logical/2−2)`——「三处同族」抽验 2/3 命中，Degraded 声明在位 ✓ |
| T6 260918-03 open①② | **PASS** | knot-static 件：一手 sources（fabric-loader）+ 字节码判读（sponge-mixin 无 sources，Degraded 显式声明 ✓）；推断等级诚实标「机制方向」，分支 1 vs 分支 3 未分辨已声明且不影响结论；§9.8 副作用（两个 .tmp 解压目录）= derived 已登记 ✓；BlobProbe.java:39/42/46 抽查实读命中。定论条件（getCause 打点一次）明确，未越权 |
| T7 palette 回退 | **PASS（程序合规）** | judge 实 grep legacy-t7-1.log：`fallback vanilla`/`packed-rc`/`legacy-fallback` = **0 行**；[LIGHT-DOMAIN] task 行 = **457**、ok=0 = **0 行**（各 task ok 之和 3703，全>0）——「0/457 任务满载」分母合法。单臂 n=1 边界 + 「未证不可能」维持 + idk 90 天线形态均在位；关账为用户拍板，judge 只核程序 → 无越权 |
| T8 域批 vs vanilla | **PASS-with-conditions（最重项）** | 全链核对见下节；FAIL 判定成立 |

## T8 专项（审查重点逐条）

1. **预登记先于采集**：criteria-t8.md mtime 18:53:24 < run_t8.py 18:53:51 < domain 采集 18:59:23 < vanilla 19:14:32 → 「与驱动同批定稿、先于采集」属实（#112 合规；判据与工具同批属声明明示，非事后补判据）。
2. **前置集执行**：双臂 result.json 实读——domain：done=1/move5=1/lightInit=1/hook=1/dll=cc4e39fe/seed-in-leveldat=true/region-mtime>rmtree ✓；vanilla 负自证：lightInit=0/lightrust=0/hook=0 ✓（#118 硬门）。VOID 分支在 run_t8.py 中以 `sys.exit(2)` 机械表达（#160 合规）。
3. **读法执行**：判据为存在性判据（diff>0 即 FAIL，无阈值带 #154），VERDICT FAIL = 预登记读法机械执行，**无事后挑读法**。机制归因明确留在判据层外 ✓。
4. **独立复算**：judge 自写脚本（不采信 cmp_t8.py 输出）对两份 light.json 重算：a=9450 b=9450 common=9450 new=0 gone=0，**diff=4223（44.6878%）**——与交付值逐位吻合。值均为 16 hex 字符串，比较语义正确。
5. **比较器 sample 缺陷**：实读 cmp_t8.py:16 `f"{c[0]},{c[1]}"` 对 str key 做序列切片 → "13,-1" 显示为 "1,3"。**计数/VERDICT 基于完整 str key 集合，不受影响**（复算已证）；交付侧已诚实登记不静默（#诚实声明合规）。→ N5。
6. **量级对比口径**：「44.7% ≫ 0.8% 噪声带」的 0.8% 来自 G17 旁证（vanilla 形态 run 间 drift，E1 口径），本对拍为跨执行体（E2）差异——**档位不同，0.8% 带不能作为 E2 噪声上界**，该句作为「系统性 vs 噪声」旁证说服力打折。但 criteria-t8.md:33 已预登记「量级对比交解读层」且 FAIL verdict 不依赖该对比 → 结论不受影响。→ N2。

## 置信度合法性

- 本块全部产物 status = draft（record/scope-bc/b3-balance/knot-static 各头实读）；无任何新 confirmed 授予；引用的既有 confirmed（260914-02/260918-03/池级）均标注为历史用户授予，未越权。T7「judge 确认后生效」表述 = 待本意见，程序正确。
- 验证分层标注：T2/T3/T5/T6 = Degraded（声明在位）、T7/T8 = Full 采集（运行时证据在案）、T4 open① = 静态坐实（未运行时复现，边界声明 :73 在位）——以实际执行为准，无虚标。

## 条件清单（N1..N7；N1-N5 建议处理，N6-N7 登记）

- **N1（binding，契约）**：`.artifacts/index.yaml` 未登记本块任何条目（git diff 实证 +10 行仅为 c1a-260919-05）——record/scope-bc/b3-balance 三件违反 core-artifact 落盘登记契约。收尾交付前补登记（status: draft/candidate 由主会话定，judge 建议 draft，收尾件建议 candidate 需先补 N1）。
- **N2（binding，口径）**：T8 交付文「≫0.8% 噪声带」句须补 §9.7 档位声明（G17 = E1 run 间带 vs 本对拍 = E2 跨执行体；0.8% 非 E2 上界）。FAIL verdict 本身不变。
- **N3（建议）**：T4 修复脚本 FAIL 分支（:43）stop 后无 WaitForExit/taskkill（原第二块被删时该路径的等待面一并消失）——FAIL 路径存在残留 java 进程风险；且修复后脚本未实跑回归（pickHit=2 为旧 log 复算）。下次采集首跑时验证 refusedCount 输出与进程退出。
- **N4（建议，§9.8）**：T8 采集的 in-place 副作用「server.properties seed 改写」其逆（.bak-formprobe，可寻址）未在 record 登记；world rmtree 为脚本声明内派生动作 ✓。补一行登记即可，不要求回退（#146）。
- **N5（建议）**：cmp_t8.py:16 sample 显示缺陷一行修复（key 为 str，直接打印 key 本身）；修复前后续复用该比较器时忽略 sample 行。
- **N6（登记）**：T2 diffs.txt 计数（19/8/10/1）与 T3 余额表 file:line judge 未全量复算（抽查命中：backlog-map 引用 errors:323 / verdict:56 / 架构计划:82-84 三处逐字命中）。两项均已用户拍板（挂起），残余风险低。
- **N7（登记）**：T6 sponge-mixin 侧为字节码偏移判读（无 file:line 可引）+ 版本归属为 cache 单版本弱推断——件内已诚实声明，定论条件（动态 getCause 打点）明确，维持「机制方向」等级即可。

## 知识库落盘建议（草稿候选清单，不写正文；subagent 产出后主会话应用）

1. **Sponge Mixin 设计性拒绝 mixin 类直接 classload**（IllegalClassLoadError "cannot be referenced directly"）+ Knot 包装点 + 应用层吞 cause 三段链——Fabric 探针/stats 读取通用坑，修复模式 = 计数器外移普通 holder 类（discovered/，高价值：判错签名 =「stats read failed + 顶层消息」即查 cause 链）。
2. **比较器抽样显示缺陷模式**：对 str key 做序列切片/索引 → 显示失真但计数正确；判读时「计数与显示分离核」一招（discovered/，中价值，工具坑家族）。
3. **双臂对拍判据预登记模板**：preconditions 表（含负自证硬门）+ 存在性判据无阈值带 + VOID 非零退出 + 覆盖面/外推边界四件套——可复用判据形态（workflow-patterns 或 discovered/，中价值；与 §9.7/§15.1 家族互补）。
4. **E1 run 间噪声带 ≠ E2 跨执行体上界**（N2 教训）：量级旁证引用跨档位带须声明档位差（并入既有 §9.7/口径家族条目，若有则追加不新建）。

## 建议状态（供用户拍板，非动作）

- record-lowcost-260919-06.md：补 N1/N2 后建议 **candidate**（T8 FAIL 结论与 T7 零回退计数证据链完整且经独立复算）。
- scope-bc-reassessment / b3-balance / knot-selftransform-static：**draft 维持**（Degraded/机制方向等级恰当；knot 件升 candidate 需动态面定论后）。
- confirmed：全部留人类。
