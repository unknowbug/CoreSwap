# P-α 廉价探针结果（260917-03，draft）

> 探针设计来源：`.investigations/g3-drift-basis-260917-01/candidates/.b3-mechanism-candidates-draft.md` §1 M-a'-α 判别探针 1（预置）。
> 执行：主会话（零成本日志统计，无新 run）。日期锚 = Get-Date 2026-09-17 16:06。

## 方法

统计有效臂日志中 `[LightRust] fallback vanilla xN` 行（mixin ServerLightingProviderMixin.java:149-159 打点：
`n <= 8 || (n % 256) == 0` 打印——**n=1..8 无条件打印，无节流盲区**；fallback 发生则 x1 必出现）。
grep 命令行数与日志原文恒等式自检：0 行命中 = 计数 0（无可丢失通道，stdout/stderr 均并入日志，run 脚本 `stderr=subprocess.STDOUT`）。

## 结果

| 日志 | 臂形态 | boot 数 | fallback 行数 | 有效性 |
|---|---|---|---|---|
| G17-L60 / G17-D60 | 漏 -PlightRust = vanilla 光照 | 2+2 | 0 | **VOID**（#156，无信息量） |
| G17b-L60 | legacy 接管 | 2 | **0** | ✅（judge 已核 lightInit ok ×2） |
| G17b-D60 | domain 接管 | 2 | **0** | ✅ |
| G17c-L3 | legacy run3 | 1 | **0** | ✅ |
| G17c-D3 | domain run3 | 1 | **0** | ✅ hook armed + task 行在 |

**fallback 总计 = 0**（全部有效 legacy 臂 boot：G17b-L60 ×2 + G17c-L3 ×1）。

## 判读（按 .b3 预登记判据）

- 「fallback 集 ∩ changed 集显著 → α 主嫌疑；**fallback≈0 → α 排除，转 β**」→ **α（fallback 混合通道）排除**（在已观测的 run2→run3 漂移窗内 fallback 未发生，交集恒空）。
- 连带：M-b（循环依赖，仅存于 vanilla fallback 子路径的限定形态）的生死判据 = fallback 计数 → **恒 0 ⇒ M-b 作为 run2→run3 漂移的通道证伪**（结构排除 Rust 内核通道 F1 + 实测排除 fallback 通道）。
- **β（静默空节瞬态读，`sec.isEmpty()→AIR`，mixin :211-216 无打点不回退）升为唯一活着的时变输入通道候选**；F3「信 stored 未重算」仍为并列未测通道（P-path 才能分辨）。

## 边界与诚实声明

- 覆盖面：仅本轮 G17b/G17c 的 5 个有效 boot、单 seed、n=1 run 链；「fallback 在其他时序/其他 seed 下也 ≈0」未证。
- β 与 F3 的分辨需 P-β/P-path（需临时诊断打点，非零成本，本轮不做）。
- 本结果为 draft；candidate 需 judge SHOULD 审查。
