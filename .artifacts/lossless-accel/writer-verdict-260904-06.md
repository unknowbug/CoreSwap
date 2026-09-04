# 写者裁决 v2：OOB dirt/sand 写者 = Rust SURFACE（candidate，260904-06）

> **supersedes**: `writer-verdict-260905.md`（双跑伪影 best-explanation）
> **superseded-by**: （无）
> **推翻理由一行**: stageMask=3 修复链路运行时重导后 OOB dirt 未消失、E1 跳过 Rust SURFACE 后 (195,199) 列 dirt/sand/gravel 全清空——写者是 Rust SURFACE 而非双跑伪影。
> **status**: candidate（judge PASS-with-conditions，review-judge-260904-06-writer.md，C1-C6 补正已应用；confirmed 留人类）
> **§9.7 口径**: 载体 = mod cppReplace 链（dll ec4a9aed，stageMask=3）block_probe FULL 导出 vs C++ NOISE+SURFACE 导出；覆盖面 = 4×4 @ chunk(200,200) seed 8576294172403134396；与 260905 轮口径可比（同域同 seed，执行体状态不同已在文中声明）。

## 结论
1. **写者身份（candidate）**：mod 臂 OOB dirt/sand/gravel 由 Rust `build_surface` 写入——结构消去链（NOISE/ore_vein/aquifer 块表无此三类、features 被 bit1 跳过、SURFACE 在跑）+ E1 运行时确证（WG_SKIP_SURFACE=1 后列清空）+ E2' 排除 est 路径混淆（shared 开/关 sha256 逐字节相同）。
2. **双跑伪影假设结案**：b611fcb（09-02）早于现役 dll 构建（09-03 23:47），dll 含 wg_set_flags 且日志 stageMask=3——NEXT_SESSION「现役 dll 早于修复」前提被推翻（廉价验证：dumpbin + git 时间戳）。
3. **机制现状（draft/idk）**：NOISE「stone 幕帘」（y≈192-318，0<d≤0.39）上两臂 surface 各自作画，43/11 = 边界微差；col(198) 周期 dirt 由 ~16 间距 aquifer 水口袋驱动，col(199) mod 侧触发输入 idk；E3 钩子 195 列零输出 idk（R8：chunk 未生成已证伪）；(244) 列 = C++ 参照自身 surface 规则分支缺陷（同 biome 下 C++ 写 gravel 偏离 vanilla，mod sand 疑为正确值）+ d 候选：heightmap 填充判据分叉（C++ 含水 vs Rust 不含水，仅开放海洋列）。
4. **移交新调查线**：「NOISE stone 幕帘（0<d≤0.39）的 C++/Rust 共有 vanilla 偏离」——范围含海洋列 heightmap 判据分叉，对比须控制 structures 混杂（b2 §3.2 警示）。

## 证据链
完整过程与原始数据：`.investigations/lossless-accel/fanout-writer-260904-06/`（evidence-pack / b1 §1-§10 含 R1-R8 / b2 / d / convergence / cmd-output 七件 + review-judge-260904-06-writer.md）。
