# versions/1.20.1/docs/10-timewise-archive.md 260917-03 块草稿——subagent 产出，主会话应用

> 应用位置：10-timewise-archive.md 末尾（260917-01 块追记之后）。格式对齐 260915-03/260917-01 块（状态标注 ✅/❌/🔍/❌→✅，含被排除项）。

```

## 260917-03（实际 2026-09-17 16:06 起，Get-Date 锚；K2 接管形态同配置噪声锚实测 + P-α 零成本探针 + 首轮 VOID 教训）🔍 candidate（judge PASS-with-conditions B5/B6 SHOULD 待应用；confirmed 待用户；verdict 仍 draft）

> 过程产物 `.investigations/k2-noise-anchor-260917-03/`（record + k2-noise-anchor-errors.md E1-E3 + p-alpha-result + review + cmd-output 六份日志 + changed_set json）+ `.artifacts/k2-noise-anchor-260917-03/verdict-260917-03.md`（draft）+ 计划 `.investigations/000-架构设计/架构计划-260917-03-R1载体换轨前置探针.md`（用户批准）。承接 260917-01 K2（接管形态同配置噪声锚 = R1 载体换轨前置）。执行体 dll sha256 6F7FA3AE…2337（target 与打包件开工前逐字节核对）；seed 8576294172403134396。

- ✅ **K2 噪声锚（domain 臂）**：同配置连续 3 对 run-to-run（K2-W/G17c-D3、D4/W、D5/D4）changed 均 **0/2025 = 0.0000%**；检测地板 1 chunk ≈ 0.049%。§9.7 三要素：载体 = snap_light 2025-chunk region 快照（45×45 spawn 区，light 逐 chunk 对比）；覆盖面 = 单 seed 单区域 Done+60s、n=3 对；可比性 = 与 G17c/G3 系同载体可直接比，与存档口径 / vanilla 形态 0.8% **不可比**（#111）。结论：domain 形态噪声带上界取检测地板 **≤0.05%**，低于 G17c-D3 的 0.10%——G3 的 run2→run3 残余 2 changed 属**信号非噪声**。K2-W 顺带直证 domain 重载收敛不动点（vanilla 污染 81 chunk 一次 boot 全洗回）。
- ✅ **C-1 R1 带宽度参数建议（输入，非定稿）**：带 = ≤1 chunk（0.05%，检测地板）或保守 ≤0.10%（2 chunk）灰区上限；正式重述 + 判据预登记 MUST 与采集脚本同批（#112），另步做。
- ✅ **P-α 零成本日志探针（无新 run）**：核打点节奏（`n<=8 || n%256==0` 无条件打印、不受 FormProbe.ON 门控，judge §A1 一手核对）→ 零 fallback 行 = 计数 0 → 全部有效 legacy 臂 boot（G17b-L60 ×2 + G17c-L3 ×1）fallback 总计 **0** → **M-a'-α（fallback 混合通道）排除**，M-b（循环依赖，仅存于 fallback 子路径）连带证伪；**β（静默空节瞬态读，mixin :211-216 无打点）升为唯一活着的时变输入通道候选**，F3（信 stored 未重算）并列未测（P-β/P-path 待做，非零成本）。
- ❌→✅ **E2 首轮 VOID（本块最重要过程错误，五段式见 errors 台账）**：K2-D4 首跑漏 tmpdir 修复——DSH 沙箱 TEMP 重定向 → JVM `extractWorldgenDir` AccessDeniedException → lightInit threw → **整臂静默 vanilla fallback**（SELFCERT lightInit=0/hook=0/sha=none，changed 81/2025=4% 伪装 legacy 量级）；#118 SELFCERT 硬门采集完成时即 VOID，未外泄无效结论（正面案例）。修复 = TEMP/TMP + java.io.tmpdir 固化工作区 + `gradle --stop`（#32 daemon 吞 env 成对）；三连 boot 全 PASS。→ build-tooling #42 家族补充案例。
- ❌→✅ **E3 裁决不断链**：GATE 只打印不退出 → boot1 VOID 后 boot2 已带 prev 启动，kill 留半截日志（归档 K2-D5-VOID2-partial，#144/#146 换标签）；教训回写脚本 GATE 不过即 `sys.exit(4)` 断链。→ workflow-patterns #160。E1（cmd-output 目录未建 FileNotFoundError，§八.5 家族）同台账。
- ✅ **judge 收尾（review-260917-03.md）：PASS-with-conditions，推荐 candidate**——MUST 项全过（打点节奏/判据对 .b3 预登记/§9.7 三要素/wash 轮使用/VOID 排除/台账五段式/index 无越权）；SHOULD 待应用：**B5**（[SELFCERT]/[GATE] 行本体未归档，需补录 cmd-output 或注记「分量可由日志逐项重建」）、**B6**（run_k2.py 在 .tmp 灭失即不可复现，需随课题归档副本）；B7 备注（收敛态幂等性解读边界，现文措辞已合规）。
- ❌ **被排除项**：K2-D4 首轮（VOID，沙箱 TEMP）；K2-D5 首次（VOID2-partial，链未断）；P-α 排除 M-a'-α 与 M-b（通道级证伪，非结构存在性）；G17-L60/D60 臂（VOID，vanilla 形态，#156）。
- 🔍 **遗留**：① verdict 补 B5/B6 后转 candidate、confirmed 待用户；② C-1 正式重述 + 判据预登记（与采集脚本同批）；③ P-β/P-path 分辨 β vs F3（需临时诊断打点）；④ K2 噪声锚引用保持「观测下界」措辞（单 seed 单区域 n=3）。
- 📌 记录指引：通用模式 → build-tooling #42 家族补充案例 + workflow-patterns #160 + #13 家族补充案例（subagent 草稿 + 主会话应用）；错误台账 → k2-noise-anchor-errors.md（本块独立成篇）；INDEX 尾注行同批落盘。

```
