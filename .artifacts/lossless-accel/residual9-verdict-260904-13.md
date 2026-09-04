# residual9-verdict-260904-13 — 残 9 gravel→sand 收口（H2 定点分类根因 + 修复 + decisive）

- 状态：**confirmed（用户拍板 260904-13）**；judge 同意 candidate 在前
- judge：同意 candidate（复跑 verify 复现 12→3；附 3 项非阻断补正——idk 注释已加、commit 待落、脚本哈希笔误已更）
- 日期：260904-13（实际 2026-09-04 22:24-23:30，Get-Date 锚定）
- 前置三元组 ✓：开工时现役 dll=A82B7A8D（与 260904-12 交接一致）；git @ aab1181 干净
- 课题链：residual76（confirmed，49→9）→ residual12（探针 A，H2 唯一存活）→ 本轮收口

## 1. 根因（三源交叉，candidate 级）

**Java MultiNoiseUtil 分类器在 1e-4 定点 long 域运算**（`MultiNoiseUtil.toLong(v) = (long)(v * 10000F)`，输入先过 float；C++ biome.h L155 `noiseToLong` 已同构复刻）。**Rust biome.rs 用 f64 全精度算距离且未量化输入 → 定点域的精确平局在 f64 下变成 ~1e-9 假严格差 → 刀锋点分类翻转**。

证据链（残 9 簇 pick cell (59,9,62)，同 seed 同 6d 输入 t=0.200031787 d=-0.007196…）：
1. **Java 臂 ground truth**（Biome6Probe mnDump，反射直读 NoiseValuePoint）：raw longs `t=2000 h=494 c=-5097 e=31 d=-71 w=-5468` → dist(deep_ocean)=dist(deep_lukewarm)=**5041，精确平局** → 树序取先 = deep_ocean（参数表 line 12 < line 16）。storage 佐证：cellDump chunk(14,15) cellY=9 残差簇 cell(12,8)/(0,8)=deep_ocean ✓（`.tmp/p2full/res13-mndump4-260904-13.log` / `res13-biome6-java-260904-13.log`）
2. **C++ 臂**：WG_BIOMEDUMP 同 pick 同 6d → deep_ocean（i64 定点域同样平局取先）（`.tmp/p2full/res13-cpp-biome-260904-13.txt`）
3. **Rust（修复前）**：f64 → dist(lukewarm)=5.178618e-5 < dist(ocean)=5.178719e-5（差 1.01e-9）→ deep_lukewarm ✗

§9.7 声明：mnDump 载体=NoiseValuePoint 直读（覆盖该 cell 一点）；decisive 载体=gradle blockProbe FULL 口径 4×4@200,200；与残 76 存档写入口径（82.16%）不可比。

## 2. 修复（WorldgenRust/src/biome.rs）

- `noise_to_long(v: f64) = ((v as f32) * 10000.0) as i64`（C++ L155 同构，含 float 中转）
- SearchTreeNode 携带量化 `lparams: Vec<[i64;2]>`（NAN 空范围 → `[i64::MAX, i64::MIN]` 哨兵），叶子/enclosing 距离改 i64 运算
- 平局裁决：`dist == best && idx < best.idx` 按参数表行序取先（Java/C++「树序第一个」在残 9 点的等价近似；全量 decisive 未引入新残差）
- 修复后 dll sha256 = **561AFF49…**（idk 注释终版；19D21219 为其前序构建，行为等价）；decisive fix13d（23:27）以 561AFF49 复跑，12→3 复现一致
- judge 补正已落实：biome.rs get_resulting_node 加 @anchor.idk（平局 idx 序近似 + 跨 seed 潜在分歧源标记）；verify 脚本哈希笔误更正

## 3. decisive（decisive 12→3，-9）

- 命令：gradle runServer `-PblockProbe=true -PblockProbeFull=true -PcppReplace=true -PcppLib=<新dll> -PcppWorldgenDir=versions\1.20.1\data\worldgen -PbenchSeed=8576294172403134396 -PbenchSize=4 -PbenchOriginX=200 -PbenchOriginZ=200`（seed 三查 ✓ 三方一致；world 删除重导；pregen FULL 35s 在场）
- 输出：`.tmp/p2full/off-fix13-260904-13/`；对比脚本 `verify_fix13_260904-13.py`
- **ref<->off-fix13：mism=3（water→dirt 1 / granite→gravel 1 / gravel→water 1）match=100.000%**；残 9 gravel→sand 全灭
- 剩余 3 块与上轮归因一致（ore_vein 1 + aquifer 2），独立课题不混判

## 4. 排错记录（错误优先，本轮三坑）

1. **BIOME6 直采 router 6d（t=0.1121）与分类输入（t=0.200032）完全不同**：Biome6Probe 直采用 quart 坐标语义，生产 biome 链在 (px<<2) 块坐标语义上采样 DFs——打印坐标≠采样坐标（#17 坐标钉死律再现实例）；本轮以生产语义 mnDump 直读 NoiseValuePoint 破局。
2. **`-PblockProbe.full=true` 静默不生效**：gradle 映射名实为 `blockProbeFull`（#8 家族三犯形态）；症状 = 残差 12→78107（carver 邻域缺失假回归），修正后 12→3。判错经验：gradle 新 -P 参数首次使用必须核对 build.gradle 映射行 + 日志行为化证据（pregen 行在场）。
3. **NoiseValuePoint 无 double getter**：字段是 long 定点（temperatureNoise() 等），先反射 dump 方法表再取 getter。
- A/B 隔离插曲：首次 78107 疑似修复回归 → 旧 dll 同命令 78116 → 证实环境口径问题而非回归（且 78116−78107=9 恰为修复点，已闭环）。

## 5. 待办顺延

1. ore_vein 1 + aquifer 2（归因已明，各自立案）
2. bareseed 收口 confirmed 仍待用户拍板（judge 材料齐备）
3. H1 zoom pick 全局性（C++ 对照臂本轮已间接覆盖残 9 簇 pick 一致；全局普查仍 open，低优先）
