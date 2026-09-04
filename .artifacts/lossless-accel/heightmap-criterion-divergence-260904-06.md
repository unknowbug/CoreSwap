# 海洋列 heightmap 填充判据分叉：静态坐实（draft，260904-06）

> **status**: draft（静态对拍 Partial/Degraded 证据 + verdict-260904-06 (d) 运行时签名旁证；升 candidate 待 judge）
> **§9.7 口径**: 载体 = C++/Rust 源码静态对拍 + writer-verdict-260904-06 (d) 节的 (244) 列运行时签名（sda=48/fluid_height=MIN）；覆盖面 = 判据语句单点 + 海洋列触发域推理；与 (d) 节口径可比（同源同列集）。
> **继承验证**: 本条继承 convergence-260904-06 (d) m2 的「heightmap 填充判据分叉」假设——本轮已按 M14/M11 纪律做廉价独立验证（双侧源码定位），验证通过。

## 结论（draft）

1. **C++ 侧**：`versions\1.20.1\cpp\worldgen\src\worldgen_api.cpp:1045`
   `if (block != air && wy > heightmap[...]) heightmap[...] = wy;`
   判据 = **放置块非 air**。NOISE 阶段 aquifer 在密度≤0 处放水 → **水面计入 heightmap**（WORLD_SURFACE_WG 顶到水面）。
2. **Rust 侧**：`WorldgenRust\src\terrain.rs:279`
   `if top == i32::MIN && d > 0.0 { top = y; }`
   判据 = **密度 > 0**。水柱（d≤0，含海洋水体）**不计入** surface_height → 海洋列 heightmap 偏低（落到海底 stone 顶而非水面）。
3. **vanilla 语义（Java 源已核，2026-09-04，E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\Heightmap.java）**：L24 `NOT_AIR = state -> !state.isAir()`；L143 `WORLD_SURFACE_WG(..., Heightmap.NOT_AIR)` → **水计入**。按此 **C++ 对、Rust 偏离 vanilla**——与 verdict (244) 列「mod sand 疑为正确值、C++ 写 gravel 偏离」的 d 节推理方向存在张力（(244) 列归因待新线三方对比最终裁决）。
4. **触发域**：仅「列顶块=水/流体」的开放海洋列（水体顶高于海底）；陆地列 d>0 顶=非空顶，两侧一致——与 (d) 节「195 列不受影响」吻合。

## 影响
- Rust surface 扫描起点 o = heightmap+1 偏低 → OceanFloor/水下列 surface 规则、fluid_height、steep 谓词输入差。
- 同族点位：`terrain.rs:279` 是 surface_height 唯一填充点（`worldgen_handle.rs:556/681` 均消费它）。

## 验证分层声明
Partial/Degraded：静态对拍（本产物）+ 运行时签名旁证（d 节）；**无独立运行时 A/B**。修复方向登记：Rust `terrain.rs` 判据改为「放置块非 air」（含 aquifer 水/气输出语义对齐），预计一处小改——按纪律本轮零代码改动，修复另出微计划。

## 附：已排除
- C++ heightmap 索引转置（`surface_rules.rs:119` 注释疑点）：写 `bz*16+bx`（api L1045）↔ 读 `z*16+x`（surface.h L734）一致，不成立。
