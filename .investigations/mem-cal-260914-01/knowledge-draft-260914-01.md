# knowledge-draft-260914-01 —— 内存口径工作块知识库落盘草稿（knowledge subagent 产出，主会话应用+验证）

- 状态：draft（主会话应用后；正文状态标注 candidate，confirmed 留用户）
- 结论来源：`.investigations/mem-cal-260914-01/record-260914-01.md`（candidate，judge review-001 PASS-with-conditions，C1-C3 已应用）+ `review-001.md`
- 应用目标：① `knowledge/discovered/build-tooling.md` 尾部追加 #148/#149；② `knowledge/discovered/workflow-patterns.md` 尾部追加 #119 补充案例；③ `knowledge/INDEX.md` 尾部追加一行；④ `versions/1.21.6/docs/10-timewise-archive.md` 尾部追加 260914-01 时间线块

---

## ① knowledge/discovered/build-tooling.md 追加草稿（接 #147 之后）

```markdown
---

## 发现 #148（最高价值·错误优先）: PowerShell `$var:xxx` 是 **drive-qualified 变量**词法——`$gcl:time` 被 `$env:` 同族语法吞掉静默取空，gc log 落成 160B 空壳 + 工作目录 `,uptime` 杂散文件是此坑的副产物签名（260914-01）

- **发现时间 / 发现者 / 置信度 / module**：260914-01；主会话（VOID 臂产物核对 + PS 最小复现定因）+ judge subagent（C2 杂散文件归属核）；**candidate**（机制 PS 最小复现实证 + 修复后 r4-r6 三臂自证全过，confirmed 留人类）；build-tooling / PowerShell 脚本坑（#45/#49/#41/#147 同文件家族的**变量词法面**）。
- **来源定位**：`.investigations/mem-cal-260914-01/record-260914-01.md` §五（错误记录五段式全文）+ §六（`,uptime` 杂散文件）；judge `review-001.md` §6（VOID 留档与 err 体积差旁证）；运行台 `.tmp/mem-cal-260914-01/run_memcal_1216_260914-01.ps1`（.tmp 不入库，record 为权威记录）；VOID 臂空壳 `gc-*-r6-18220.log`（160B，盘上现存留档不删）。
- **五段式（错误优先）**：
  - **现象**：前置臂 r1/r2/r3 的 gc log 恒为 **160 字节空壳**（pauses/used 数据全缺），测量臂数据全部判 VOID；且 VOID 轮使 JVM 在工程目录落了名为 **`,uptime`** 的杂散文件（120B，`versions/1.21.6/java/,uptime`，untracked，judge C2 已清理复核零残留）。
  - **根因（机制）**：脚本里写 `$gcl:time` 本意是「变量 `$gcl` + 字面量 `:time`」拼 gc log 路径，但 PowerShell 词法把 `$gcl:time` 解析为 **drive-qualified 变量**（`$<drive>:<path>` 语法，`$env:PATH` / `$function:x` 同族）——`gcl` 被当作 PSDrive 名去解析，**不报错、静默取空** ⇒ `-Xlog:gc:file=` 收到空/非法路径，JVM 把 options 段残段（`,uptime`）当文件名落到工作目录。这不是 gc 参数写错，是**变量名接冒号的词法歧义**。
  - **定位**：读 VOID 臂产物发现 gc log 恒 160B；对照 VOID 臂 `.log.err`（~6.9KB）与 r4-r6（~7.0KB）的体积差追到 pickup 行流向（另立 #149）；PS 侧最小复现确认 `$gcl:time` 的 drive-qualified 解析行为。
  - **修复**：① gc log 路径变量改 `${gcl}` 包裹（`${gcl}:time`）或换名避开冒号歧义；② 修复后**换新标签 r4-r6 重跑**（VOID 轮 r1-r3 留档不删不覆盖，承 #146）；③ 杂散 `,uptime` 删除并复核零残留。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **PowerShell 变量名后接冒号必须 `${var}` 包裹**——`$var:xxx` 会被解析为 drive-qualified 变量（`$env:` / `$function:` / `$global:` 同族），静默取空不报错；凡「变量 + 冒号 + 后缀」拼字符串场景（路径/文件名/时间戳后缀）一律 `${var}:xxx`。
  2. **`,uptime` 类杂散文件是此坑的副产物签名**：`-Xlog:gc:file=` 类「文件路径内嵌选项段」的 JVM 参数收到畸变路径时，残段会以逗号开头落成工作目录杂散文件——看到 `,uptime` / `.0` / `.1` 类 untracked 残留先查上游脚本变量拼接，不是 JVM 随机行为。
  3. **测量装置的日志落盘产物 MUST 做「非空 + 内容合理」核验后才采数据**（与 #149 自证门配套）：空壳文件是驱动静默失败签名（#20 家族测量侧形态）。
- **家族索引**：#45（`-like` 字符类）、#49（`-File` 多值参数）、#41（管道早退截断）、#147（脚本程序化修改语法门）——同文件 PowerShell 坑家族；#20（死参数制造假判别——本条为「参数值被词法吞掉」的测量侧形态）；#144/#146（VOID 轮日志留档与杂散文件保全）。

---

## 发现 #149: JVM 参数注入类测量的**自证门判据**——`JAVA_TOOL_OPTIONS` pickup 行打 **stderr**，门禁覆盖面 = 被证通道全集（stdout+stderr）+ 注入产物非空双查；落物流向必须先实证再写门（260914-01）

- **发现时间 / 发现者 / 置信度 / module**：260914-01；主会话（第一版门只扫 stdout 假放行被抓）+ judge subagent（review-001 §1 独立复算 2 条 pickup 行）；**candidate**（修复后 r4-r6 xmxApplied=2 硬门放行，confirmed 留人类）；build-tooling / 测量有效性自证门（#118「自证行做成硬门禁」的**JVM 参数注入面**；#37「生效证据必须行为化」的通道维）。
- **来源定位**：`.investigations/mem-cal-260914-01/record-260914-01.md` §二（通道设计）/ §五（五段式根因 2）；judge `review-001.md` §1（`.log.err` 各含 2 条 pickup 行的独立复算）；VOID 臂 `.log.err`（~6.9KB）vs r4-r6（~7.0KB）体积差 = 流向旁证。
- **五段式（错误优先）**：
  - **现象**：第一版自证门只匹配 stdout 找 `-Xmx` pickup 行——`-Xmx` 实际未生效（或未实证生效）时门也放行，**自证失效假通过**；实际 pickup 通告行全在 stderr，stdout 恒零命中。
  - **根因（机制）**：`JAVA_TOOL_OPTIONS` 的 `Picked up JAVA_TOOL_OPTIONS: …` 通告行由 JVM 打到 **stderr**（JVM 既有行为，非想当然的 stdout）——门禁只扫一条流 = **漏掉被证通道**；且「pickup 行出现」与「注入产物可用」是两件事，单查一行 ≠ 双查。
  - **定位**：VOID 判 VOID 时顺带核自证面：对照两版 `.log.err` 体积差（≈2 条 pickup 行量级）+ 直接读 `.log.err` 命中 pickup 行 ⇒ 流向实证。
  - **修复**：自证门改为**同时扫 stdout+stderr** 的 pickup 行，`xmxApplied` 计数 **daemon + server 两条**（=2 才放行，防只起一半）；与 gc log 非空核验（#148 判据 3）组成双查。
  - **教训**：见判据。
- **判据（可复用）**：
  1. **自证门覆盖面 = 被证通道的全集，不是猜的一条**——`JAVA_TOOL_OPTIONS` pickup 走 stderr 是 JVM 行为；**落物流向必须先实证（跑一次读流）再写门**，凭直觉指定流 = 假放行。
  2. **JVM 参数注入类测量开工前 MUST 双查**：① pickup 行在实证过的流上出现且计数达预期条数（daemon/server 多通道场景逐条计数）；② 注入产物非空且内容合理（gc log 行数/字节数下限）。两查齐过才采数据，否则判 VOID（承 #118：mismatch 即 VOID 不许人工挑臂）。
  3. **多通道注入按条数硬门**（本块 = daemon + server 两条，=2 放行）——只查「出现过一次」对「起了一半」无区分力。
- **家族索引**：#118（自证行硬门禁——本条为 JVM 注入参数面）、#37（生效证据必须行为化——本条为「落流通道实证」维）、#97（`[FEATURE]` 日志落 stderr、stdout grep 恒零——同构的「流放错」家族先例）、#20/#148（驱动/注入静默失败签名）、#32（env 未送达家族——本门即其防线）。
```

## ② knowledge/discovered/workflow-patterns.md 尾部追加草稿（#119 补充案例，接 #146 之后）

```markdown
---

> **#119 补充案例（260914-01，固定 -Xmx2G 后残余摆动带量化 + n≥3 极差带判据）**：固定 `-Xmx2G` 后同配置串行 n=3（同 seed/同 256 chunk region/同 dll），peakPriv 极差 **~110.9MB（~6.7%）**、peakWS ~7.0%、GC 停顿后 used 稳态 ~4.4%（~22MB）——**#119 的 1.7× 摆动收窄到 ~7% 但未归零**（G1 堆高水位机制仍在，峰值进程内存不随 -Xmx 钉死）。判据（内存回归口径，candidate）：① 载体必须固定 `-Xmx`（pickup 行双通道自证可行，→ build-tooling #149）；② **n≥3 同配置串行 run 取 peakPriv 极差带作本底噪声带**（带绑定 §9.7 三要素，换 -Xmx/region/seed 须重测，不得跨口径引用；极差随 n 单调增长，n=3 是小样本估计）；③ **信号判定操作定义**：单次 run 超带只作线索，**连续 n≥2 同向超带才立信号课题**；判据**首次使用后 MUST 用首用 run 再校准带**（升 n=5 或按新极差重算）；④ live-set 代理用 GC 停顿后 used 稳态值，不用峰值进程内存；⑤ ±100MB 内的单 run 峰值变化不构成回归/优化证据。VOID 轮教训（r1-r3 空壳 gc log + 门扫错流）→ build-tooling **#148/#149**。来源：`.investigations/mem-cal-260914-01/record-260914-01.md`（judge review-001 PASS-with-conditions，C1-C3 已应用；confirmed 留用户）。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260914-01 块。
```

## ③ knowledge/INDEX.md 尾部追加行草稿（接 260913-06 行之后）

```markdown

> 260914-01 追加：workflow-patterns **#119 补充案例**（固定 -Xmx2G 落地——残余摆动带 ~7% 未归零（1.7× 收窄非消除），内存回归判据 = n≥3 peakPriv 极差带作本底噪声 + 连续 n≥2 超带才立信号 + 带绑定载体口径、首用后再校准）；build-tooling 新增**发现 #148（错误优先）**（PowerShell `$var:` drive-qualified 词法陷阱——`$gcl:time` 静默取空致 gc log 160B 空壳，`,uptime` 杂散文件为其副产物签名，变量接冒号必须 `${var}`）+ **发现 #149**（JVM 参数注入自证门判据——`JAVA_TOOL_OPTIONS` pickup 行打 stderr，门禁覆盖被证通道全集 stdout+stderr + 注入产物非空双查，落物流向先实证再写门）。来源：`.investigations/mem-cal-260914-01/`（record + judge PASS-with-conditions）。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260914-01 块。
```

## ④ versions/1.21.6/docs/10-timewise-archive.md 尾部 260914-01 时间线块草稿（接 260913-06 块之后）

```markdown

## 260914-01（实际 2026-09-14，Get-Date 锚开工时点）：内存口径测量——固定 -Xmx2G 下 run 间摆动带（臂 r4/r5/r6）—— **candidate**（judge PASS-with-conditions 条件已应用；confirmed 待用户授予）

> 过程产物 `.investigations/mem-cal-260914-01/`（`record-260914-01.md` 主记录 + `review-001.md` judge 三源核对 + 本草稿 `knowledge-draft-260914-01.md`）+ `.tmp/mem-cal-260914-01/`（mem-poll csv / gc log / 运行台脚本，不入库在盘可核）。上游：#119（260910-08，未设 -Xmx 时峰值摆动 1.7×，内存回归 MUST 固定 -Xmx）；本块 = 其落地第一步。通用模式 → workflow-patterns **#119 补充案例** + build-tooling **#148/#149**。

- ✅ **核心结论**：固定 `-Xmx2G` 后 n=3 摆动带 = peakPriv 极差 **~110.9MB（~6.7%）** / peakWS ~7.0% / GC 停顿后 used 稳态 ~4.4%——1.7× 摆动**收窄到 ~7% 未归零**；内存回归判据草案 = n≥3 peakPriv 极差带作本底噪声，**连续 n≥2 同向超带才立信号**，带绑定 §9.7 载体口径、首用后再校准；回归主通道 peakPriv（含 native+堆 committed，peakWS 为辅）。
- ❌→修正 **前置臂 r1/r2/r3 VOID（错误优先）**：① `$gcl:time` 被 PowerShell 解析为 drive-qualified 变量（`$env:` 同族）静默取空 → gc log 160B 空壳 + 工作目录 `,uptime` 杂散文件（judge C2 已清理）——修复 = `${gcl}` 包裹（→ #148）；② 自证门只扫 stdout，`JAVA_TOOL_OPTIONS` pickup 行实际打 stderr → 假放行——修复 = stdout+stderr 全流扫描 + xmxApplied=2 硬门（→ #149）。VOID 轮留档不删不覆盖（#146），修复后换标签 r4-r6 重跑。
- ✅ **judge（隔离 subagent，三源核对）= PASS-with-conditions，条件已应用**：数字零偏差（judge 独立重算 gc log/csv 逐格吻合）；C1 = 信号判定操作定义补入判据；C2 = 杂散文件清理复核；C3 = GC 计数口径注记（pauses 只计 Pause Young 行，全停顿口径 69/69/70）。
- ⚠️ **§9.7 / Degraded**：载体 = 固定 -Xmx2G + 256 chunk region forceload + 同 seed 同 dll（`abd7d889…`）串行；覆盖面 = 单 seed 单 region overworld n=3（极差随 n 单调增长，小样本）；与 #119 未固定 -Xmx 口径**不可比**；Degraded = 无 Full GC / 无 jcmd 直读 live-set（只有 GC 停顿后 used 代理）、5s 采样粒度峰值归因未知、peakPriv 构成未分解。
- 状态：**candidate**，confirmed 待用户授予。
```

---

（草稿完——以上均为文本草稿，未改动 knowledge/、docs/、index.yaml 任何文件。）
