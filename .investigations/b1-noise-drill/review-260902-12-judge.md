# judge 审查意见 — B1 NOISE 微差下钻 verdict 260902-11（260902-12）

> 审查对象：.artifacts/b1-noise-drill/noise-drill-verdict-260902-11.md（candidate 申请）
> 三源核对：① 产物快照 verdict + index.yaml ✓；② git：全部本 session 新增，lib/dll 零改动 ✓（未提交）；③ 探针源码/坐标/seed/sanity 与 verdict 一致 ✓，census 桶标签偏移已诚实标注 ✓。

## CONCERN

- **C1（最重要）｜Java↔Rust 数值差从未直接测量**：全部量化证据均为 Rust 单侧 sample_density_exact；scout §4 末与架构计划 Phase 2.5 预定的两侧配对 dump（Java DensityProbe）未执行。「~1e-6 级 FP 求值序微差」是推断非实测。
- **C2｜量级分流判据自洽性缺口**：scout 二分判据（|d|~1e-9→舍入类 vs ~0.1→角点值错）之间，实测 |d|∈[3.7e-8, 2.27e-5] 居间；第 13 格符号翻转要求 Δd≥2.27e-5，与「仅需 ~1e-6」表述矛盾（仅对 |d|≤1e-6 格成立）；2.27e-5 是否在求值序重结合累积范围未做 op×ulp 估算。
- **C3｜「非结构性错误」措辞越出证据**：A3 家族含「old_blended_noise 内部 ~1e-6 级系统差」，与随机舍入差在当前证据下不可区分。建议措辞改「非符号级/网格级结构错误；A1 vs A3 不可区分」。
- **C4｜A4 排除有一处未验前提**：「C++ 非同构 AVX 路径」（C++ noise 无 SIMD）未引用证据；建议静态核对一行或降格声明。

## NOTE

- N1：census 阈值放宽已显式披露，非隐瞒式 cherry-pick；但 1e-5 为后验阈值，建议标注「用于封闭性统计而非判据」。
- N2：「71 擦边格未翻转 = 插值平滑」是 plausible 非已验证，建议标注候选解释。
- N3：「无可感知影响」无证据，建议改显式 idk 或删除。
- N4：scout「x 等差 9」被全量否定、主会话修正——过程修正如实记录，好样板。

## 结论

**建议授予 candidate（有保留）**：机制类收敛证据链扎实；C1+C2 要求措辞限定为「机制类 = FP 求值序微差类（A1/A3 不可区分），量级为推断」——修 C3 措辞后即可授予，不必补探针。

## 用户拍板（260902-12）

1. C3 措辞修正采纳 → candidate 授予；
2. 不补 Java 配对采样（接受机制类收敛）；
3. A1/A3 不再下钻，以 99.9992% 封顶结案 → **confirmed 回写**（verdict + index.yaml 已应用）。

## 处置记录

- C1/C2/C3/N1/N2/N3 全部应用进 verdict 正文；C4 补 idk 标注（未静态核验）。judge 原始意见全文（本文件前半）由主会话转录落盘。
