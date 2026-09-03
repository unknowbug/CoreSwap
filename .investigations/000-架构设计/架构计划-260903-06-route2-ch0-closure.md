---
编号: 000
任务: 路线② X2 后续——ch0 跨语言残差闭合（GPU ch0 vs C++ CPU oracle）→ 解锁 WG_GPU_CHANNELS → 端到端 P4
任务类型: 验证（数据层对拍）+ swe（transpiler cache_2d 修复 + fallback 改绑）
模式档位: 轻量（≤3 主线要点，机制已明，前包 260903-05 已钉定形态）
状态: 批准（260903-06，实际 2026-09-03 14:29，git 锚 80c9e95）
编号来源: 前包 260903-05（80c9e95）→ 本包 260903-06
---

## 范围（含明确不做什么）

- 做四件事：
  1. **P-A**：GPU ch0 vs C++ CPU 侧 ch0 dump（got_export densityDump / C++ production 链）数值复核——Rust transpiler oracle 已证不可靠（cache_2d 列常量化），C++ CPU 侧为唯一可信 oracle；闭合 ch0 残差 0.03-0.23 归因 → 解锁 WG_GPU_CHANNELS
  2. **P-B**：transpiler cache_2d 修复（仅可证 y 无关子树缓存）+ GpuChannelDensity CPU fallback 改绑 DfcDensity；门禁 scan invalid=0 + cargo 绿
  3. **P-C**：P4 端到端 vs Java ≥256 chunks（WG_GPU_CHANNELS 开/关 A/B，零退化铁律 + §9.7 三要素预声明，judge D2 口径）+ 0.61× 无探针整批 wall 复测
  4. **P-D**：知识库落盘（subagent 草稿）+ 收尾 judge（MUST，三源核对）
- 不做：N1 取证、H3 ×16 重测、glslc 原子更新判据落地（欠账维持范围外，下轮再排）；不动 macro_sampler 生产默认路径。

## 继承结论（已核验可继承）

- GPU 通道含 ch0 = Java/C++ 语义（bB 五要素等价 + final e2e 3.1e-7，judge 过）——但 P-A 仍以数据层复核钉死（ch0 是通道级首验）
- Rust transpiler ch0 缺陷根因 = density.rs:335 cache_2d 闭包 y=0 求值整列复用（.bA，candidate）
- macro vs GPU ch0 残差 0.03-0.23 未闭合（idk 候选）——P-A 顺带归因（macro 走真实树仍与 C++ 有差的原因）

## 验证方式

- P-A：got_export/densityDump 采集（主会话）→ worker 解读（subagent）；判据 = GPU ch0 vs C++ CPU ch0 major_diff(>1e-4)=0（f32 口径，§9.7 三要素声明）
- P-B：scan_cpp_anchors.py invalid=0 + cargo check/build + ch1-4 对拍不回归
- P-C：WorldGenBench ≥256 chunks 稳定中位数，GPU 关门控与主线一致（零退化）；0.61× 复测 = 无探针整批 wall + 调用计数（judge 修正的两步走）

## 子角色介入点（预置，执行只核对不补排）

- scout: 否（机制已明）
- worker: P-A 数据解读（subagent）；P-D 知识库草稿（subagent，prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md）
- fan-out: P-A 若归因分叉 ≥2 互斥机制候选 → MUST fan-out .bN（禁止主会话自推）
- judge: P-A 闭合 candidate 授予 SHOULD；P-C 端到端结论 MUST；收尾交付 MUST（三源核对）
- knowledge: P-D（结论进 docs/10 时间线/discovered，subagent 产出）

## 风险 & 回退

- C++ CPU 侧无现成 ch0 单独出口 → 用 got_export densityDump（分量级无插值，既有工具）或临时诊断出口（bin-diag 隔离）
- ch0 残差归因到 macro 侧（Rust 真实树 vs C++ 差异）→ 影响面 = Rust 生产 macro 路径通道级精度，final 层 99.99% 掩盖中——如实落盘 + judge 评估是否阻塞 WG_GPU_CHANNELS（GPU 侧本身经 C++ oracle 复核可信即可启用）
