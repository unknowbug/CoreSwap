# OOB 残差课题重推导裁决：无残差（幻影课题）（confirmed，260904-06）

> **supersedes**: `writer-verdict-260905.md`（双跑伪影 best-explanation）+ `writer-verdict-260904-06.md`（写者=Rust SURFACE）+ `curtain-verdict-260904-06.md`（幕帘=vanilla 机制）——三者核心证据均为布局误读伪影
> **superseded-by**: （无）
> **推翻理由一行**: 正确布局（blocks.h:69 y-major）重读全部臂导出：mod↔vanilla FULL OOB 区（y>200|y<-32）0 diff、surface 写者 y>200 贡献 0、cppNS y>200 0 diff——「OOB dirt 写者/幕帘/双跑伪影」均不存在，系 x-major 列读脚本制造的幻影数据（incident-layout-260904-06.md）。
> **status**: confirmed（judge PASS 260904-08 review-oob-redo-260904-08.md 独立复算全吻合；用户实机 A/B 后拍板 confirmed 260904-08）
> **§9.7 口径**: 载体 = coltool.py（y-major 权威实现，含 header/sanity 自检）对既有臂导出逐字节重读；覆盖面 = seed 8576294172403134396 / 4×4 @ chunk(200,200) 全部 98304×16 块位 × 5 臂（ref/mod/e1/e2/cppNS）；与既有口径同域同 seed，但读数变换已修正（其可比性以 incident 记录为准）。

## 裁决

1. **无 OOB 残差（candidate）**：mod（Rust dll EC4A9AED，stageMask=3 FULL）与 vanilla 参照在 OOB 区逐位一致——四×4 区域内「出界 dirt/sand 写者」不存在。原课题（跨 260905/260904-06 两轮）结案：**幻影课题**。
2. **「写者 = Rust SURFACE」结论失效**：E1 消融（skip surface）在正确布局下 y>200 差异为 0——无对象。E1 的 y<0 大量差异（deepslate/bedrock surface 规则）为预期行为，与残差无关。
3. **布局无关硬数据存活**：WG_DBDEBUG (195,199) 密度全负（y192-318 = −0.02/−0.46）仍有效；aquifer 三方 17 项对拍、est 六维零差、heightmap 判据分叉（terrain.rs:279 vs worldgen_api.cpp:1045 + Heightmap.java:24）等静态对拍不受布局事故影响。
4. **真残差主场移回 ore 族 desync**（block_probe 引擎内 TOTAL ~95%，引擎内计数布局无关、真实存在）：22653 mismatch 的族分解坐标归因需以 coltool 重算；cmp_spatial_260904-04.py 同布局嫌疑连坐。高 y granite/copper idk 随幻影数据一并撤销（正确布局下无此信号）。

## 重推导数据（本裁决一手）
- `.tmp/p2full/rederive_e1_260904-06.py` / `rederive_e1b_260904-06.py` 输出（E1 全列对/ y>200 分解 / mod-vs-ref / cppNS-vs-ref）
- `.tmp/p2full/coltool.py`（布局权威实现）+ `recheck_y_major_260904-06.py`（列 (195,199)：ref/cppNS y180-319 全 air；全列 diff 仅 22 且全为低 y 预期差）

## 影响的裁决链（全部降级/作废）
- writer-verdict-260905（candidate）→ 作废
- writer-verdict-260904-06（candidate）→ 作废
- curtain-verdict-260904-06（candidate，judge PASS-with-conditions 亦基于污染脚本）→ 作废
- heightmap-criterion-divergence-260904-06（draft）→ **保留**（静态对拍，布局无关）
- b1/b2/b3 静态对拍产物 → 排除类结论维持，机制性 idk 中凡引用列剖面的需重算
