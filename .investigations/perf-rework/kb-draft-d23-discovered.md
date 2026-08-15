# kb-draft-d23-discovered —— 草稿 2/3（目标文件：knowledge/discovered/algorithm-fingerprints.md，追加到发现 #13 之后）

> 本文件是知识库 subagent 草稿，供主会话应用 + 验证签核。插入位置：algorithm-fingerprints.md 文件末尾（发现 #13 之后）追加以下条目，编号接续 #14。应用后 INDEX.md 无需新增行（algorithm-fingerprints 已在 INDEX 登记）；若按条目级登记可补「#14 边界外推/验证盲区」。

---

## 发现 #14: 边界外推遇嵌套 value 必须递归（spline 边界分支的「执行不到」类 bug）+ 单域 e2e 验证是盲区制造机

**发现时间:** 2026-08-15
**发现者:** worker（block_probe 集成立项 I5 吞吐对比 → D23 定位，perf-rework GPU 集成课题）
**来源定位:** `.investigations/perf-rework/gpu-accel-errors.md` D23 段（含最终合并版）+ `.investigations/perf-rework/i-integration-record.md` + `.investigations/perf-rework/review-003-d23-integration.md`；复现/验证数据 `cmd-output/domain-probe-D23-fixed-20260815.txt` / `cmd-output/e2e-A5-20260815-135509.txt` / `verify_p11_recursive.py`
**置信度:** candidate（GPU+sim 双修 + domain probe 全域 clean + e2e 零回归 maxDiff=3.128e-07 + 显式栈 vs 递归参照 1344 组合 0 mismatch；confirmed 待用户拍板）
**module:** re-code（发现于 perf-rework GPU 集成，规律本体为复刻算法正确性 + 验证覆盖方法）

### 观察

GPU 引擎（spline_eval 显式 while 栈）与 CPU 参照（DensityBuilder）在 e2e 验证域（x≤63, y∈[-64,-49], z≤4）逐位一致（maxDiff=3.128e-07），但在域外大坐标 chunk 域系统性错值（(784,160,-408) gpu=0.045 vs cpu=-0.458，量级级差异非浮点舍入）。根因 = **spline_eval 边界外推（coord < loc[0] / coord > loc[n-1]）对端点 value 写成 `(kind==0 ? valF : 0.0f)`——嵌套 value（kind==1）直接返回 0，未递归求值**；vanilla `Spline.apply` L259/261 的边界外推是 `value[0]+der[0]*(x-loc[0])`，端点 value 为嵌套样条时**必须递归求值**。触发条件：spline55 的 coord（continentalness@c0）= 0.060231412 **恰好 > 最后 loc 0.06** → 右边界 → 嵌套 value 返回 0 → 上层链错（参照该点应递归得 factor=4.524）。由此提炼四个跨版本/跨项目通用规律：

1. **边界分支「执行不到」类 bug 指纹**：边界外推（coord 超出 locs 范围）分支只在特定坐标域触发——单域验证（e2e 小域）永远测不到——「逐位一致」只证明**被覆盖的域**；性能/吞吐探针必须顺带做多 chunk/多 cell/多 y 层 diff 抽查。同类还有 C12（range_choice 常数分支吸收误差）——「采样点没覆盖有效路径」的假正确是通用陷阱。
2. **模拟器与 GPU 同源产物同错**：模拟器复现 GPU 错值（sim=GPU=0.045303285）＝生成器+解释器**共同逻辑 bug**（非 GPU 特有）——定位先做「GPU 特有 vs 共同逻辑」二分（sim 能复现 → 直接排除 GPU kernel/驱动层）；但 sim 只能证明「生成器产物内部一致」，**必须与第三方参照（DensityBuilder）对拍**才能发现生成器级错误。
3. **显式栈移植的返回地址/恢复点纪律**：显式栈的「返回地址（outSlot）」与「父帧恢复点（stage）」是两套状态——压帧时各设一次，回填时**只写数据槽**；任何「回填时顺带改父帧 stage」的优化破坏等待语义（跳 v1 求值 → Hermite 用 0）。
4. **对照 vanilla 逐行是最终手段**：Spline.apply 边界外推是递归求值（L259/261），不是取 0——「生成器里留的 stub/简化占位」是语义差头号嫌疑（D17 ws→0.0f 同教训），对照原版逐行才收口。

### 证据

- 决定性单点：(784,160,-408) 修复前 gpu=0.045303289 vs cpu=-0.458333333（diff 5.036e-01）→ 修复后 gpu=-0.458333343（diff 9.9e-9，`cmd-output/domain-probe-D23-fixed-20260815.txt`）
- 错误域模式：z-scan（y=160 x=784）z=-432..-412 对 / z=-408,-404 错（cz=2/3 格）；y-scan（x=784 z=-408）y=-64 对 / y∈[-56,248] 几乎全错 / y≥256 对（无地形常数分支 -0.02499）——**常数分支层吸收差异 = 假正确**（C12 同款陷阱）
- 根因证据链：sim 复现 0.045303285（与 GPU 完全一致）→ 排除 GPU kernel 特有；node[54]（roughness@c0）拆分采样 == CpuBackend 直接采样逐位一致（coord 正确）；node[22]/[33] SPLINE 大坐标域算出 0；spline55 数据（locs=[-0.19,-0.15,-0.1,0.03,0.06]）coord=0.060231412 > 0.06 触发右边界
- 修复验证：e2e maxDiff=3.128e-07 / avgDiff=1.097e-08 与基线逐位一致（零回归，`cmd-output/e2e-A5-20260815-135509.txt`）；显式栈 spline_eval_py vs 递归版 Spline.apply 参照 **1344 组合 0 mismatch**（`verify_p11_recursive.py`，覆盖边界触发域坐标）
- 候选排除（❌）：H1 角点序 / H2 cell 推导 / H3 split 数值均验证无差；中间误判（「缺 noodle_ridge_b 拆分行」「双索引错位」）被 check_split_base.py + check_two_alloc.py + check_meta_vs_splitbase.py 证伪——**对账必须基于当前生成产物**（旧 comp/spv dump 会误读索引，多花数轮）

### 如何利用

- **验证覆盖设计**：GPU/加速内核接入集成/吞吐探针时，正确性抽查 MUST 覆盖多 chunk（含 chunk 0 外）/多 cell（cy≥1、cz≥2）/多 y 层（含常数分支层——常数分支吸收差异是假正确）；「单域逐位一致」不能作为全域正确性证据
- **性能/吞吐探针默认带 diff**：只测时间不测正确性 = 只能发现慢不能发现错（本次 16/64 chunks 正是靠附带 diff 抽查才暴露 D23）
- **「GPU 特有 vs 共同逻辑」二分**：模拟器能复现 → 生成器/解释器共同 bug，先排除 GPU kernel 特有路径，再与第三方参照逐分量对拍（registry 分量探针 getRegistryEntry 采样 factor/sloped/entrances 最快）
- **显式栈移植**：返回地址与恢复点分离管理——压帧设一次、回填只写数据；边界/特殊路径用显式 stage（等边界 v0/等边界 vn），不重载普通 Hermite 状态
- **数据驱动树/表（spline 类）的边界语义**：外推端点 value 为嵌套结构时递归求值（vanilla 语义），任何「简化取 0」都是潜伏的域相关 bug——跨版本（1.18/1.19 Spline.java 同构，边界外推同为 `value[0]+der[0]*(x-loc[0])` 递归）同样适用
