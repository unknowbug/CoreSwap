# Judge 审查意见 · lossless-accel 路线② FFI 工作包收尾（260903-04）

- 审查角色：core.judge（subagent，只出意见，不改 status；confirmed 留人类）
- 审查对象：`.investigations/lossless-accel/route2-ffi-260903-04.md` 收尾结论（根因 = final_density.spv 陈旧；重编全绿；C++ 引擎无重大错误、不触发转向）
- 三源核对基线：① 产物快照（route2-ffi 记录 + .bA/.bB/.bC + cmd-output 6 件）② git status/log + blob 哈希取证 ③ tri-cut3 / gpu-corner-probe2 / dfc-verify 验证记录 + 架构计划 W1-W4 判据
- 取证命令记录：`git hash-object final_density.spv.bak-pre-d23` = `71c73c96…` = `git rev-parse 9de661e:…final_density.spv`（逐字节同一 blob）

## 逐项结论

### 1. 根因判定「spv 陈旧产物（D23 修复前语义）」——PASS（证据链强于记录自述）

- **时间戳反驳已消解**：「commit 时间 ≠ 编译时间」的反驳不成立——本次有 commit 级取证：9de661e（08-15 19:22）提交的 spv blob 与 mtime 08-15 14:17 的 .bak-pre-d23 **哈希逐字节相同**，而 D23 修复（改 dfc_gen.py shader 生成器）提交于 cc58e05 08-15 19:21。即提交进库的 spv 确系修复前编译产物，不依赖 mtime 单证。
- **语义证据独立成立**：tri-cut2 旧 spv 在 seed A 已验证点 (784,160,-408) 输出 0.0453032888，与时间线 L1386 记录的 D23 修复前错误值逐位相同——「复现历史错值签名」是不依赖任何时间戳的独立证据。
- **修复闭环**：重编后 tri-cut3 双 seed 23 点 major_diff=0；6144 点 max_diff=9.18e-6（f32 ULP 级）、rounded6 96.08%。因果方向（旧 spv→错、新 spv→对）由干预实验直接证明。
- 接受该推断链，且建议主记录补记 blob 哈希取证这一最强一环（当前记录只引 mtime/commit 时间）。

### 2. 「C++ 引擎无重大错误，不触发 Rust 改写转向条款」——PASS

- 转向条款判据（架构计划：修复需动核心结构 / patch 后信心不足）未被触发：根因在构建产物不在引擎代码；重编后负坐标远端 chunk（x≈-4608）与全 y 柱全对（tri-cut3 行 14/28-34），.bB 怀疑的「负 chunk 输入失效」被 tri-cut3 直接证伪。
- 附加确认：.bA 排除的批量跨 chunk / 陈旧槽位 / push-constant 输入类，与最终根因（资产代际）不冲突，无遗留未解释失配点。

### 3. git 三源核对——PASS（1 项 CONCERN）

- 新文件清单与记录声明一致：gpu_ffi.cpp（??）、build.ps1 修改（M）、gpu_corner_probe.rs（??）、tri-cut*/gpu-corner-probe*/dfc-verify cmd-output（??）、.bA/.bB/.bC（??）、spv 修改 + .bak-pre-d23（??）。.tmp/tmp_diag_tri.cpp 被 .gitignore 覆盖（check-ignore exit 0）——记录引用的临时诊断源不入库，可接受但属易失证据（建议关键探针源归档副本）。
- **CONCERN（仓库卫生）**：仓库根出现两个游离未跟踪产物 `E:\PYTHON\CoreSwap\cpu_backend.h`、`E:\PYTHON\CoreSwap\final_density.comp`（09-03 03:52，与本轮重编同刻）——生成脚本输出落错目录的残留，与 gpu-assets 内正式产物重复，应删除或移正，防下次排查误读旧副本。

### 4. 验证记录对结论的支撑——PASS（附 2 项漏验证）

- tri-cut3（10+13=23 点、双 seed、含负坐标远端）major=0；probe2 6144 点 max_diff=9.18e-6、rounded6 96.078%——支撑「通道全绿 + f32 口径」。
- **漏验证 ①（W4 预注册第三路缺失）**：架构计划 W4 判据为「三路对拍：vs DFC oracle + **vs f64 DF 树 ≤1e-6**」——落盘输出只有 gpu_vs_dfc_oracle，**无 GPU vs f64 树的直接对比记录**（oracle 与 f64 树的关系由 260903-03 的 dfc_verify 间接桥接：ms_vs_dfc=0）。建议补跑或显式声明以「oracle→f64 树」两级链替代。
- **漏验证 ②（预注册判据放宽未走台账）**：预注册「rounded6 舍入内 0 diff」实际为 96.08%（≈3.9% 6 位舍入边界翻转）；记录以「f32 ULP 噪声、先例 96.06% 口径」解释并做了 §9.7 同行声明——解释合理，但架构计划明文「任何失败→错误台账五段式，不静默放宽口径」，lossless-accel-errors.md 中 grep 无 260903-04 条目。建议补一条五段式或显式豁免声明，否则构成「静默放宽」形式违规。

### 5. 附加 a) 置信度与取代链——PASS（附格式建议）

- 根因结论建议 **candidate**（有运行时干预实验 + commit 级取证）；不建议 confirmed（留人类）。三事实中 ①create 74s、③f32 口径可 candidate；②Mutex 0.61× 见第 6 项。
- Degraded 静态审查的 bA/bB 被运行时实验取代：主记录「候选归属（§15.4 取代记录）」段已逐候选写明证实/证伪/保留，取代方向清晰。**格式缺口**：按 §15.4 应为 supersedes 双指针——.bA/.bB 产物文件本体未回写 supersedes 指针（status 仍 draft 且无指向取代记录的一行），主记录单端指针。建议主会话应用时在 .bA/.bB 头部各补一行 supersedes 指针（内容不改）。

### 6. 附加 b) sampleInterpGrid y=320 grid[49] 越界读——PASS

- 处理正确：bB 明确「真实但不解释本 diff」，主记录归为独立缺陷另立修复项、并注明「本次数据未触发」（y=320 行新旧 spv 下均一致，与 clamp 分支主导的解释自洽）。未混入根因。
- 建议：该缺陷目前只存在于记录文字，未见进入任何修复项清单/台账——落一个待办条目防丢失。

### 7. 附加 c) 三事实解读是否过度声明——CONCERN（仅事实②）

- ① create #1=75.0s/#2=74.3s（首轮 63.7/66.1s 同向）→「无跨 handle 缓存」证据充分，无过度声明。
- ③ f32 口径：§9.7 三要素同行声明到位（载体=角点批量 fill、覆盖=8 chunk×全 y、与 oracle 同源可比），且与历史 96.06% 口径做了可比性对照——合规。
- ② 「Mutex 0.61× = GPU dispatch 异步流水」：**机制归因略超现有证据**。0.61× 只证明「Mutex 未串行化吞吐」，「异步流水」是对机制的候选解释——记录自己也承认正确性「由 fill 内 readback 同步保证」是推断（无 per-call readback 计时或 queue depth 证据区分「异步化」vs「驱动队列排队」vs「测量口径」）。首轮记录曾诚实标「待判」，终值段改为断言语气。建议降为「候选解释（candidate 机制、draft 证据）」，或在生产接线前补一次微验证（如双线程 per-fill 计时分布）。不影响本包收尾结论。

### 8. 附加 d) 对照架构计划 W1-W4——PASS（W1-W3 齐，W4 见第 4 项两条漏验证）

- W1 ✅（dfc-verify 重跑与 260903-03 一致：0.92% 残差、ms_vs_dfc=0、dfc 658.5 ms/chunk，落盘可查）；W2 ✅（shim+build.ps1 -Ffi，产物在 git status 可见）；W3 ✅（bin-diag 隔离区合规）；W4 主体达成，两条缺口如上。
- W5（S6 台账清偿）声明已完成，本次抽查 errors.md 无 260903-04 新条目与「重编 spv 教训」——**本轮最大的可复用教训（gpu-assets 二进制产物与生成器代码可跨版本失配、重编即愈）未进台账/知识库**，恰是「资产代际失配」类可复用判据，建议随 docs 落盘批补记。

### 9. 产物契约与记录卫生——CONCERN

- `.artifacts/lossless-accel/` 无 index.yaml，根 `.artifacts/index.yaml` 也未登记 route2-tricut.* 三件——core.artifact 落盘契约未闭环（产物文件本身齐备，不驳回，但应用时须补登记）。
- 主记录 fan-out 段 `.bB` 条目重复两行（一行 ✅ 结论、一行「进行中」残留），应清理，防后续误读。

## 总评与建议置信度

- 收尾结论「根因 = spv 陈旧产物、重编全绿、C++ 引擎无重大错误、不触发转向条款」：**建议 candidate**（证据链强：blob 哈希 + 历史错值复现 + 重编干预实验三重独立）。confirmed 留人类拍板。
- 事实①③：candidate；事实②机制解释：draft（候选解释）。
- 应用前建议动作清单（不阻塞 candidate）：补记 blob 哈希取证 → 根因/重编教训进错误台账（subagent 草稿）→ .bA/.bB 补 supersedes 回指针 → artifacts index 登记 → 清理仓库根游离 cpu_backend.h/final_density.comp → sampleInterpGrid OOB 立修复项待办。
