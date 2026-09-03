# off 臂 −1 扫描偏移 + 角参数 +16 修复裁决 — 260903-13

---
status: candidate
verification: partial
date: 260903-13（锚 git 提交时间戳）
supersedes: 无（修复对象是 260903-12 judge A1 发现的生产 bug + #25 第三例角参数，非推翻既有 confirmed 结论；260903-12 shared 裁决 confirmed 仍然成立）
---

## 结论

1. **off 臂 −1 扫描偏移已修**：`worldgen_handle.rs` off 臂扫描由半开区间 rev（首点 319）改为对齐 Java `NoiseChunk.computePreliminarySurfaceLevel`（forge sources NoiseChunk.java:174：`for(l=min_y+height; l>=min_y; l-=cellHeight)` → 320..-64 含两端）。
2. **角参数 +15→+16 已修（两臂共有）**：对齐 Java `MaterialRules.java:496-499` `chunkToBlockCoord(i+1)=(i+1)<<4`；dump 行 corner_params 同步。
3. **修复后 est 优化零语义差**：四臂 hash 完全一致 `f2b1a3932c6e589e`（off==shared==l2==shared+l2）——WG_EST_SHARED/L2 从「语义变化 + 提速」变为**纯性能优化**，翻默认不再是语义决策。

## 证据链

- **P0 交接验证**（修复前，HEAD 43a858e，工作树干净）：四臂 hash 与 260903-12 记录逐项一致（off `74f5dfc4eede8ef4` / shared `8bff408735f1560d` / l2==off+84.9%）→ 交接结论继承合法。
- **修复后四臂**：`cmd-output/estopt-ab-arms-p1fix-260903-13.txt`——四臂 agg hash 同值 `f2b1a3932c6e589e`。
- **Java est 角列对比**：`cmd-output/est-compare-p1fix-260903-13.txt` + `.tmp/estdump/compare_260903_13.py`——off/shared 各 **256/256 与 Java 逐值一致、0 diff**（8 missing = 预热 chunk (400..402,400) 角，Java 表无该区域列，符合预期）；Java 表 11877 条 conflicts=0。
- **敏感 chunk (201,200)**：修复前 java@+16=56 / shared@+12=48 / off=55 → 修复后两臂均 c0:48 c1:56 c2:48 c3:56，与 Java 一致（旧 off=55 为 −1 扫描伪差）。
- **Java 权威源**（Mojang 官方映射 forge-1.20.1-47.4.0-sources.jar，非反编译推断）：`.tmp/jnssrc/.../NoiseChunk.java:163-177`（preliminarySurfaceLevel = yarn estimateSurfaceHeight：坐标先 `(x>>2)<<2` 量化、扫描 320..-64 含下界）；yarn 侧 `MaterialRules.java:496-499`（四角 (i+1)<<4）。
- **可比性声明（§9.7）**：载体 = est dump 角列（与 260903-12 同一探针 WG_EST_DUMP）；覆盖面 = c0 区 64 chunk × 4 角 + 敏感 chunk 单点；与 260903-12 口径同载体同覆盖，可比。

## 修复后语义影响声明

- off 默认路径行为**有意变化**（此前为 bug 偏差）：四臂 hash 由 off `74f5dfc4…` ≠ shared `8bff4087…` 变为统一 `f2b1a3932c6e589e`——变化方向 = 向 Java 收敛（256/256 逐值验证）。
- 生产影响面：surface 角列 lerp 输入（量化敏感 chunk ~1.6-4.7%）+ off 臂 est 扫描首点；**未做全量 block_probe 回归**（声明 Partial+est 全角列组合验证；若需 Full 存档口径回归需 block_probe 逐位，见遗留）。

## 遗留

- 全量 block_probe 存档口径回归未跑（本轮验证载体 = est 角列 + 四臂 hash；如翻默认后需存档口径 Full 证据，下轮补）。
- surface_rules.rs:505 panic 课题未动（下轮立项，MUST recode-scout 前置）。
