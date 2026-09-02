# B1 四候选判别结论（260902-09 · confirmed，260902-09 用户拍板）

> session 260902-09 · seed 8576294172403134396 · chunk(3200..3203, 3208..3211) 4×4 · biome=basalt_deltas 单一
> judge：建议授予 candidate（条件项已闭环，见 review-260902-judge-b1.md）；**confirmed 已由用户拍板授予（260902-09）**。外推边界不变：单 seed/单 biome/4×4。

## 结论

- **C1 (d) 排除（candidate）**：NOISE 层同阶段 air 签名对拍 99.68%（4083/4096 列，y0..127）。载体=vanilla PRE surfacedump vs Rust fill_chunk_blocks 全 skip dump；覆盖面=4×4×128；与旧 13.70% 口径（阶段污染）不可比。
  - ⚠️ 口径边界：NOISE 层**材质级对比不可做**——新架构事实：Rust NOISE 层写统一 default block（id=1），材质分配在 surface 层；vanilla NOISE 已是真材质 raw id。air=0 两侧同义故 air 签名有效。
- **C2 surface 层实现差上界 ≈0.005%（candidate）**：vanilla POST vs Rust surface-only（rust→vanilla raw id 投票映射：0→0、31→79、33→96、37→118、256→5850、259→5854、849→19319）99.66% 列完全一致，26/524288 差异单元。映射为同数据自举（循环性），错票只制造假差异（保守方向）。→ (a) 系统分支差排除、(c) 随机漂移排除（差异呈 band 边缘「对」结构）。
- **C3 残余差异归因（candidate）**：主因 = NOISE 层单元格级微差（13/4096 列，rust 侧洞缘多 air）→ surface band 边缘 ±1 格平移；闭合证据 = surface 14 差异列中 13 列 ⊂ NOISE 13 差异列。1 列 only-surface（51247,51375）= blackstone 底带/lava 边缘平移（y20-24，非 air 驱动），与主因同类（边缘差）非单因严格闭合。
- **C4 (b) 本区无分叉证据**：两侧全列 biome=basalt_deltas。signature A（biome 边界带 3.7% 真差，6 维逐位独立证据链）在本区外，维持原判。
- **C5 取代链建议（待 docs 落盘，§15.4）**：历史多项残差观测（13.70% air、22.5% SURFACE、黑石/玄武岩底界差）判为**测量口径阶段污染**（cppReplace 下 Java CARVERS/FEATURES 仍跑——机制证据：WorldgenRust/src/worldgen_handle.rs L83-91/L452-463 flag+env skip 仅 Rust 内部；runtime/1.20.1/java/src/main/java/wg/bench/CppBridge.java L63-71 默认 mask 0b011；NoiseChunkGeneratorMixin L72-98 cppReplace 接管 populateNoise/buildSurface）。真实存档口径残差（~3.4%）主体归因 feature/carver 阶段链路差（26 单元 surface 种子 × 放置放大，放大系数未量化）。保留项：signature A 与 soul_soil V1 Rust surface 缺口不属污染。外推边界：单 seed/单 biome/4×4。

## 证据与工具

- dump 工具：WorldgenRust/src/bin-diag/b1_noiseonly_dump.rs、b1_surfaceonly_dump.rs（本 session 新建，含正确 chunk 坐标 3200/3208）
- 对拍脚本：.tmp/compare_noiseonly.py、compare_surface_layer.py、overlap_check.py、judge_followup.py
- 预处理输出：.tmp/surface-layer-preprocess.txt
- java noise-only 存档路线证伪记录：stageMask=7 双维度生效但存档仍含 carvers/features 产物（.tmp/cpp-noiseonly-run.log；stageMask 只控 Rust 内部阶段）

## 假阴性陷阱记录（本 session 实证两个）

1. rust 侧 [128:] 切片施加于 128 项序列 → 空序列 → 假「100% 一致」；
2. `mat=` 逗号切分散（split 后只取 parts[3]）→ 单元素序列 → 假「100% 一致」。
   防范：sanity 行强制打印序列长度与 common 数。
