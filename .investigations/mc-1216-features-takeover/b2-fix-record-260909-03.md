# T2 b2 修复记录 — 260909-03

> 计划：架构计划-260909-03-features-takeover-rootfix.md。状态：4 项已修 + 编译绿 + step9 回归哨兵通过；第 5 项证伪勘误。
> 修改文件：`worldgen-core/src/feature.rs`、`worldgen-core/src/tree.rs`（薄壳零改动）。

## ① Disk 列内 break（PROVEN → 已修）

- feature.rs DiskFeature::generate：去掉列内首层命中 `break`，对齐 Java DiskFeature.java:44-52
  「target 匹配的每一层都替换」（half_height 层语义）。
- 附带核对：Java 每放置消费 1 次 provider RNG，但 vanilla disk 全用 SimpleBlockStateProvider（0 RNG）——RNG 序无差（b2 已核对，维持）。

## ② Geode isAir 不含 cave_air（PROVEN → 已修）

- feature.rs GeodeFeature::generate（原 :2006）：`st == AIR` 扩为 AIR ∨ cave_air ∨ void_air，
  对齐 Java GeodeFeature.java:62-63 `isAir()` 全族。
- §1.2B（CheckedRandom vs LegacyRandom 噪声流等价性）**未静态裁决**——交 T3 对拍 geode 家族差实证（amethyst/smooth_basalt 量级）。

## ③ Ore isExposedToAir 邻 chunk -1（PROVEN → 已修）

- feature.rs is_exposed_to_air：原 `local_idx` 出界恒 -1 → 永不 discard；改走 `ctx.block_at`
  全路由（local → block_at_ext → region_col_at），对齐 Java ChunkSectionCache 真实读（含邻 chunk section）。
- 选型：单案收敛（block_at 是现成忠实通道），fan-out 不触发（无互斥候选）。

## ④ Lake isSolid 排除树叶（PROVEN → 已修）

- tree.rs 新增 `is_solid_lake`（仅排除流体/藤/发光地衣，树叶算 solid，对齐 Material#isSolid
  语义，LakeFeature.java:80/:122）；feature.rs lake 两处调用点（校验 pass u<4 + barrier pass）切换。
- 范围限定：dungeon 等其他 is_solid_id 调用点**不扩散修改**（本课题未 PROVEN，维持通用近似声明）。

## ⑤ emerald_ore catch-all —— 证伪（§15.4 勘误）

- b2 §1.5 主张「feature_loader 无 emerald_ore 分支 → catch-all 直接损失」。**证伪**：
  - configured_feature/ore_emerald.json `type` = `minecraft:ore`（已在 :118 分发内）；
  - 旧快照实录：`fid=minecraft:ore_emerald` 正常调用 + 66 次 `ore_emerald placed at`；
  - unknown type 全集（snapshot unique 提取）不含任何 emerald 条目。
- 根因：b2 报告把 **placed feature id `emerald_ore`** 与 **configured feature `ore_emerald`（type ore）** 混同
  （发现 #90 转抄漂移 / #94 命名失真家族形态）。原记录不改，本节即取代记录（§15.4 双指针）。
- 影响：b2 §五「下游 RNG 流错位让渡 b1」随勘误失效（judge C2 已先勘误 RNG 部分）。

## 验证（§9.7）

- 构建：`cargo build --offline --release` workspace 全量绿（#27）；产物 target/release/worldgen1216.dll 16:07:24。
- step9 回归哨兵（短跑探针 + forceload）：修复后 chunk(0,0) step=9 13 项 (p,fid) 与 Java 全同——p 域未扰动。
- P1-P4/P6 族级判别不再单独跑短探针——T3 区域级重对拍统一承担（信噪比回落 = 联合验收；若残留，按 diff top name 归族复核）。
- 载体：seed -8248318472910187742，region 0,0 r=16（1089 chunks/臂）；与既有口径可比性：同 chunky 载体同 region，
  可与 260909-02 基线直接比。
