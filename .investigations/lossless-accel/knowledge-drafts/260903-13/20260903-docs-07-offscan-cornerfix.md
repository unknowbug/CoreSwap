# 草稿：07 主题篇「est 优化收口」节追加小节（260903-13）

目标落盘位置：`versions/1.20.1/docs/07-block-pipeline.md`，追加到「## 2026-09-03 est 优化收口（追加小节草稿，260903-12）」小节**之后**（追加不覆盖，不在主题篇新建时间线式章节）。

---

## 2026-09-03 est 两处偏差修复：off 臂扫描偏移 + 角参数 +15→+16（260903-13，四臂同 hash）

> 承接上节「est 优化收口」遗留的新课题 #1（off 臂 −1 扫描偏移）与 #3（角参数 +16 修正待办）。修复 commit 3e2e67d；judge review 见 `.investigations/lossless-accel/review-offscan-cornerfix-260903-13.md`（PASS）；结论 `.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md`。

### ✅ 修复一：off 臂扫描首点 off-by-one（半开区间 rev）

- **根因（机制）**：Rust `(min_y..min_y+noise_height).rev().step_by(8)` 是半开区间 rev——首采样点 = **319**；Java `NoiseChunk.computePreliminarySurfaceLevel`（forge official sources NoiseChunk.java:174）为 `for(l=minY+height; l>=minY; l-=cellHeight)`——**320..-64 含两端**。半开区间 rev 使首点差 1、下界差 step（编译器惯用法条目，见 knowledge/discovered/compiler-idioms.md 发现 #10）。
- **修复**：扫描改为闭区间含两端对齐 Java（首点 320、下界 -64 均含）。

### ✅ 修复二：角参数 +15→+16

- **根因（机制）**：Java `MaterialRules.java:496-499` SURFACE 四角用 `chunkToBlockCoord(i+1) = (i+1)<<4` 即 **+16**；Rust 两臂曾用 `cx*16+15`（经量化 +12 ≠ +16）——系 workflow-patterns #25「静态调研结论失真」第三例的实例修复（上节已登记为待办）。
- **修复**：两臂（off/shared）heights4 角参数统一改 +16。

### ✅ 验证（Full 层，§9.7 口径见 verdict 头部）

- **修复后两臂四臂 hash 完全一致 `f2b1a3932c6e589e`**（off/shared × L2 开关，64 chunks A/B）。
- Java est 角列对比：off / shared 各 **256/256 一致，0 diff**；敏感 chunk (201,200) 角值一致（java@+16=56）。
- 状态：candidate（judge PASS 建议）；**est 优化语义零差达成 → 翻默认（WG_EST_L2 / WG_EST_SHARED）由语义裁决问题变为纯性能决策**。

> 过程 → 10 时间线 260903-13 条；编译器惯用法 → compiler-idioms 发现 #10。
