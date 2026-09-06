# CONCERN-2 回归闭环 verdict（260906-04 · review-002 CONCERN-2 跟进）

> 状态：**confirmed（用户拍板关闭 260906-05）**；judge：review-003 有条件通过（C1/C2/C3 已补齐）
> 验证分层：Full（确定性载体 SHA 对拍 + 双臂存档对拍 + 同 dll 重跑基线）
> 口径声明（§9.7）：见各节

## 结论：CONCERN-2 关闭——end 接管改造（with_cells / wrapping / 空列哨兵 / cell 参数化）在 overworld/nether 共享路径上**逐位无回归**

## 证据链

1. **执行体三元组核验**（#36）：target/release dll = 1.0.26 jar 内 dll = mods jar（6B9EF26F）内 dll = **1B5AA1DEA49445A2…**（2160640B）；运行时 CppBridge 日志自报一致。基线臂 1.0.25 dll = **febe913e**（日志确认加载）。
2. **存档级双臂对拍（探索性，后判定不可裁决）**：seed 7691421705105351955，chunk(0..5,0..5)，vanilla / 1.0.25 / 1.0.26 三臂 forceload 生成。
   - 1.0.25 vs 1.0.26：ow 2206 / nether 21511 mismatch
   - vanilla vs 1.0.25：ow 3533 / nether 23805（既有非逐位对齐口径）
3. **同 dll 重跑确定性自检**（1.0.25 两轮）：ow 5012 / nether 23025 mismatch——**噪声基线与跨版本信号同阶（#51 命中），存档级跨 run 载体无法裁量此回归**；OW 差异 top 全为 spruce/pine 树（Java 装饰层），非确定性源在 Java 侧跨 run 装饰顺序。
4. **确定性载体裁决（#52 载体，决定性）**：`idk7_region_dump`（overworld）+ nether 变体 `concern2_nether_dump`（create_for_dim nether.json/256），纯 Rust fill_chunk_blocks（无 Java 装饰），chunk(0..5,0..5)：
   - Rust 侧跨 run 确定性 ✅（同代码两轮 SHA 全等 610766E0…）
   - 旧代码（99b8034，worktree 单编）vs 新代码（HEAD c2330cd）：overworld **610766E0… 全等**、nether **1248DAB2… 全等**

## 覆盖面声明（C1，review-003 补）

- 本 verdict 覆盖 = **overworld/nether 共享路径**（noise/surface 等 Rust 侧管线）。**end 维度本身不在本回归核查覆盖内**——end 行为等价性由前序 end 接管验证线（.investigations/end-takeover/，260906-04 之前各 verdict + judge review-002）另行追溯，本结论不构成 end 维度逐位证明。

## 判定

- end 改造对共享维度 Rust 噪声/表面阶段影响 = **零**（逐位 SHA 全等，双维度）。
- CONCERN-2 的「静态恒等变换补运行时证据」诉求达成，且升级为确定性载体证据（强于原建议的存档 A/B——后者被 #51 证明不可裁决）。
- 顺带沉淀：**Forge+本环境存档级跨 run 装饰非确定性**是回归核查的系统性障碍，后续回归一律走 Rust 确定性 dump 载体（或覆盖装饰层时需 chunk 配对差分 + 噪声基线前置）。

## 载体与脚本

- 三臂 region 基线：.tmp/concern2-260906/{van,mod,mod1025*}-{ow,nether}/
- 存档对拍：concern2_ab.py / ab2 / ab3 / ab4.py（复用 .tmp/end-takeover-260906/ab_end.py 解析）
- 确定性 dump：idk7_region_dump.rs（旧）、concern2_nether_dump.rs（新增，nether 变体）、dump_{new,old,nether_*}.bin + SHA
- **dump_old 构建溯源（C3，review-003 补）**：旧代码 99b8034 经 `git worktree add` + 标准单编流程构建（`cargo build --offline -p WorldgenRust --release --lib` + `rustc --edition 2021 -O`，见 NEXT_SESSION 260906-04 纪律要点）；当时未留命令日志，以产物 mtime 为溯源代理（dump_new 17:31:52 / dump_old 17:32:19，同分钟级连续采集），且 judge 已独立重算 SHA 与 verdict 记载吻合——溯源声明为 Partial（方法确定、命令原文无留痕）。
- 服务端日志：van-arm-server.log / mod-arm-server.log / mod1025-server.log / mod1025-rerun.log

## 遗留

- B 子任务（J5 T5 重做）另线进行。
- 若希望连 Java 装饰层一起回归核查，需另立课题解决装饰层非确定性（超出本 CONCERN 范围）。
