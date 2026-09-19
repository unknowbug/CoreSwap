# judge 审查意见 —— 260919-01 A1 正式判别补通道（verdict-260919-01）

- **审查角色**：core.judge（隔离子会话，只出意见不改任何 status；confirmed 留给人类）
- **审查对象**：`.artifacts/a1-260919-01/verdict-260919-01.md`（draft）+ index 条目 `swe:a1-260919-01:verdict`（draft）
- **判据**：`.investigations/a1-260919-01/criteria-260919-01.md`（预登记，时序锚 commit 6d79bc8 @2026-09-19 13:12:14——本 judge 沙箱无 shell，commit 时序未独立重验，采信 prompt 主会话锚定 + 判据文件自述）
- **三源核对**：① 交付快照（verdict + result.json + index.yaml 末条，已读）② git HEAD + 工作区 diff（**受限**：本会话禁运行命令，无法独立跑 `git status`/`git diff`；mixin 以读工作区文件全文代替，未提交状态采信 prompt 与 verdict §7 声明）③ 验证记录（两臂 log + result.json + 失败轮 C6R-L.log，已抽样核）
- **结论：PASS-with-conditions**（A1 判定方向「有显著改善面」证据充分、判别读法按预登记执行；但 verdict 正文含一处事实性复算错误（M1），须修正后方可升 candidate）

---

## 一、独立抽样复算（judge 自算数字，非转录 worker）

占比 = (fill+native)/(fill+native+wb)，与 C6R-D_result.json ratios[] 逐项对拍（**比 verdict §3 多核 2 行，并复核了 ratios 索引映射**）：

| log 行 | fill/native/wb | judge 复算占比 | ratios[] 对应值 | 判 |
|---|---|---|---|---|
| 152 | 2.734/23.695/1.574 | 26.429/28.003 = **0.943792** | ratios[0]=0.9437917 | ✅ |
| 363 | 0.250/3.448/0.230 | 3.698/3.928 = **0.941446** | ratios[1]=0.9414460 | ✅ |
| 408 | 0.303/2.761/0.193 | 3.064/3.257 = **0.940743** | ratios[2]=0.9407430 | ✅（verdict 未列，judge 补核） |
| 443 | 0.434/6.918/0.322 | 7.352/7.674 = **0.958040** | ratios[3]=0.9580401 | ✅（verdict 未列，judge 补核） |
| 475 | 0.902/3.530/0.317 | 4.432/4.749 = **0.933249** | ratios[4]=0.9332491 | ✅ |
| 505 | 0.591/6.041/3.516 | 6.632/10.148 = **0.653528** | ratios[5]=0.6535278 | ✅ |
| 9228 | 0.319/8.314/0.134 | 8.633/8.767 = **0.984715** | 主体簇（0.97–0.995）内 | ✅ |

- ratios 索引映射核对：任务行出现序 152→363→408→443→475→505 与 ratios[0..5] 一一对应、顺序无跳位。
- min/max 边界核对：ratios[] 扫描 min = **0.3207900**（数组第 61 元 = ratios[60]）、max = **0.9949984**（ratios[446]）——与 verdict §4 min=0.3208 / max=0.9950 一致 ✅。
- 计数核对（grep 实测）：`[LIGHT-DOMAIN] task` 行 **457** = `timedOut=true` **115 行** = result.json domain_task/ok/fillms 457 ✅；`degraded=[1-9]| dup=[1-9]| fallback vanilla` 全文 **0 命中** ✅（与 worker 报告一致）。
- 恒等式复核：抽样各行 avgMs ≈ 三段和（如 408 行 3.254 vs 3.257、443 行 7.670 vs 7.674）✅——**但行 2160 verdict 的偏差声明是错的，见 M1**。
- P50=0.9835 / mean=0.9774：judge 沙箱无 shell 未独立排序 457 值；采信**两套独立实现**（驱动 run_c6r_ab.py 与独立重数 recount_a1.py 不共享解析代码）同值 + judge 边界值（min/max/低尾计数）核对一致的间接验证（S4 登记方法边界）。分布形态交叉旁证：低尾 ~7 个 <0.90 值 + 主体 0.93–0.995 右偏聚集下，median(0.9835) > mean(0.9774) 方向自洽 ✅。

## 二、逐条审查意见

### M 级

**M1（必改后方可升 candidate）：verdict §3 与 §6.4 的「行 2160 恒等式偏差 7.99ms」为错误复算，@anchor.idk 登记建立在不成立的前提上。**
原始日志 C6R-D.log:2160 实测：`avgMs=27.233 fillMs=0.487 nativeMs=8.251 wbMs=18.501`——三段和 = 0.487+8.251+18.501 = **27.239**，dev = |27.233−27.239| = **0.006ms**，**完全落在 maxdev=0.08 声明内，恒等式成立**。verdict 所写「三段和=19.239、dev≈7.99ms、该行占比 0.454」三处均与原始数据不符（19.239 来源不明，疑采样串行）；该行真实占比 = 8.738/27.239 = **0.32079 = ratios[60] = 全分布 min**。后果评估：
- 判别结论**不受影响**（该行在低尾，P50/均值分支判定不变；ratios[] 数据本身无误）；
- 但 verdict §3「一处例外」段与 §6.4 idk 项是**基于错数登记的假 idk**——按 Anchorlaw §5 idk 特异性要求，必须更正为「行 2160 恒等式成立（dev 0.006ms）；真实观察项 = timedOut 宽限任务 wb 段显著拉长（18.501ms，宽限等待口径差），对应 ratios 低尾 min=0.3208」；
- 更正同时回收 §3 表中「ratio 0.454」行与 §6.4 全段，**不得原地静默改数**（§15.4 精神：更正以附加更正记录形态落盘，原错数保留可追溯）。

### S 级

**S1：criteria §3.1 与 §3.4 存在字面冲突，驱动按 §3.4 实现——判据文本缺陷，非数据缺陷。**
§3.1 要求「逐臂（C6-L 与 C6-D 同门）`[LIGHT-DOMAIN] hook armed` ≥1」，但 §3.4 要求 legacy 臂 task 行=0，且 C6R-L2.log 实测 hook armed=**0**（result.json domain_hook=0）。驱动 run_c6r_ab.py:168 对 legacy 臂恰恰要求 `domain_hook == 0`（负证据更强：域路径不可达性正自证）。判读采信 §3.4 的臂分化语义（hook 项只对 domain 臂有意义）——legacy SELFCERT 判「全绿」成立；但判据已冻结（采集开始后禁改 #112），建议在 verdict 补一行**读法澄清附注**（不改判据原文）。

**S2：驱动自加的硬门比判据 §3.5 更严——语义偏差须知情。**
§3.5 说 fallback/degraded「非零立案如实记录」，驱动 run_c6r_ab.py:150-151 把 `fallback>0` 直接判 VOID 非零退出（fail 列表 "fallback-nonzero(new channel)"）。本轮两臂 fallback=0，无实质影响；但「更严的门」若在未来轮触发 VOID，会以判据未载明的理由断链。登记为驱动实现注释即可，不阻塞。

**S3：执行体三元组两项 + mixin 编译时间戳——judge 侧不可独立复验（沙箱无 shell），confirmed 前须主会话落盘补核。**
verdict §1 已如实声明缺项（jar sha / target-release dll 独立重算）。judge 复核可做部分：两臂 in-log dll 行（L2:2169 / D:2259）均含 `sha256=6f7fa3ae0169fb5c`，size=2474496 两侧一致，与 260917-04 前置 verdict 的 dll_sha256 全 sha 前 8 位一致 ✅；result.json 两臂 dll_sha8 一致 ✅。mixin class 13:14:05 编译含 fillMs、target/release/build-resources 三方 sha 一致——主会话称已核，judge 无法重跑，**confirmed 授予前 MUST 以主会话命令输出落盘（cmd-output 或 record 引用）为据**，当前仅为宿主口头背书。

**S4：P50/均值统计量的 judge 独立性边界。**
见 §一末条：双独立实现一致 + judge 边界值核对；完整排序未由 judge 复跑。可接受，登记方法即可。

**S5：mixin 打点语义核对（纯观测性）——通过，附一处口径说明。**
wgLightDomainTaskRun（mixin :579-670）核：① fill 段 = t0 起点至 JNI 前实测**含** centers 排序/handle ensure/blocks25 分配/assemble 全程（:582-618），与判据 §2「fill = assemble 拼帧段」措辞基本一致但比字面「拼帧」略宽（含一次性分配），不影响占比判别（判据 §5 已声明占比只看 fill+native 对全任务比）；② LIGHT_TIMING 门控为整块：`_t1/_t2`（:620-622）、fillNs/nativeNs 赋值（:626-629）、`_tw0`（:633）、打印字段（:663-667）均在门内，门关时打印行无计时字段、无行为分支依赖计时值——纯观测成立；③ 成功行（out!=null）带 fillMs=/nativeMs=/wbMs=、fail≠null 行（out=null）不带——与判据 §3.4「fail≠null 行不带计时字段 = 合法形态」措辞一致 ✅；④ LIGHT_TIMING 开启时每 task 增 3 次 nanoTime + 格式化，观测开销与域批频度（≈per-chunk/9）相称，符合「chunk 级一次、不进热路径」纪律。

**S6：验收面其余各项——通过。**
- 判据符合性：VOID 断链（驱动非零退出实现，:194-196）、world 身份门（gzip level.dat seed 直证 + region mtime > rmtree epoch，result.json 两臂同值全过）、fillms 门（457/457）、#156 开关口径（驱动 :33-35 逐臂 `-PlightRust/-Pformprobe/-Pformdrive/-PlightTiming` + domain 加 `-Pdomainbatch`）、#112 读法（P50 主读 + 同侧性核对后取 ≥30% 分支）——逐项符合预登记 ✅。
- legacy 臂：LIGHTPROBE-T 18 行（L2:1010–16952，chunks 200→3600 步进 200，collectAvgMs 0.486→0.219 / nativeAvgMs 3.402→3.650）与 260917-04 C6-L 末行（0.317/5.674）同形态量级 ✅；domain 路径 0 行 ✅。
- §9.7/§1.3：三要素同行 + E1/E2 档位 + S 集声明完整；E1 量级对照（0.52 vs 0.98 ms/chunk，跨轮 ~2× 判轮间波动）方向合理且未隐瞒差异 ✅。numeric/qualitative/anchor 三级处置与 #162 一致 ✅。
- §9.8 副作用逆：失败轮 C6R-L 换标签留档核实（C6R-L.log 6823 行，13:17:44 日志截断式中断，与「宿主崩溃带走进程树」声明一致；未覆盖删除 ✅）；rmtree=生成产物等价声明在案 ✅。
- index.yaml 末条 `swe:a1-260919-01:verdict`（draft，:1866-1874）落盘契约 ✅；status=draft 合法，无越权 confirmed ✅。
- verdict §6 边界/idk 其余各项（字段语义 idk、timedOut 入 ratios 读法、n=1 覆盖面）如实 ✅（唯 §6.4 见 M1）。
- retry cap：本块为单轮采集+判别，无逆向假设验证轮次累积，不触发 §9.4 ✅。

## 三、推荐

- **判定建议**：A1 正式判别结论「有显著改善面（P50=0.9835 ≥ 30% 分支，n=457）」**建议升 candidate**——以完成 M1 更正为前提；S1/S2 补附注、S3 主会话补核落盘后一并处理。
- **confirmed**：留待用户拍板（本 judge 不授予）。
- 判据前置集核对：前置 = verdict-260917-04 §3（confirmed）降级声明，已核实其指向「新增域批路径计时行后重采」——本块即该补通道执行，前置未被取代或失效，判据未 suspended ✅。
- 噪声卡历史：本轮无未解决噪声卡引用；timedOut=115 沿 260917-04 前例归因登记，如实。

—— core.judge（subagent，2026-09-19；本意见仅为审查建议，不改任何产物 status）
