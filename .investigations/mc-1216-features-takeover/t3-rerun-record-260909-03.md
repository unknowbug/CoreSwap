# T3 重对拍记录 — 260909-03

> 前置：T1 根因修复（registry_order）+ T2 四缺陷修复 + 新发现（WG_CA_MIN 生产 gap + tree placer 缓装缺口）。
> 载体：Chunky #26，seed -8248318472910187742，region 0,0 r=16（1089 chunks/臂，3410 common chunks）。
> 状态：**验收判据未达成**——残差已分解归因，scope 决策待用户。

## 执行体三元组核验（#36，先于归因）

- 加载文件 = `runtime/1.21.6/java/build/resources/main/native/worldgen.dll`，sha16 **02d897a3**（2,411,008 B）
  = target/release/worldgen1216.dll 本次 16:07 构建。dev classpath 加载，processResources 已重跑
  （`synced Rust dll: 2411008 bytes`）。build/libs jar 时间戳 13:26 为红鲱鱼（runServer 不经该 jar）。
- rustfeat 接管哨兵：`[Mixin] placedFeature skipped (rust takeover) count=4096` 命中（脚本内 grep 的
  是 v1 措辞 `generateFeatures skipped`，NONE-SEEN 为脚本假阴性，非接管失效）。

## 结果矩阵（terrain/veg 分类口径同 260909-02）

| 臂组合 | terrain | veg | 备注 |
|---|---|---|---|
| 噪声基线（Java-vs-Java，260909-02） | 101,529 | 35,967 | run 级非确定 |
| 噪声基线（本轮重跑） | 96,391 | 39,720 | 口径稳定 ✅ |
| 信号 v2（260909-02 修复前） | 4,857,306 | 346,025 | 信噪比 ≈48× |
| 本轮 T1+T2 修复（WG_CA_MIN 默认关） | 4,685,489 | 327,850 | 几乎未降 |
| 本轮 + WG_CA_MIN=1 | **3,654,942** | 328,803 | terrain −22%；信噪比仍 ≈36× |

## 归因链（三层）

1. **T1 p 域修复生效但区域级 veg 不动**：chunk(0,0) step9 序列全同（✅）≠ 全 region 特征内容收敛。
2. **WG_CA_MIN 生产 gap（本轮定位）**：生产路径（非 ca_min）`block_at_ext=None`/`region_col_at=None`
   → 越界读恒 -1 → Ore is_exposed_to_air / Geode 读邻 chunk 修复在生产模式 no-op。开 CA_MIN 后
   terrain 4.69M→3.65M（stone 1,277k→947k、coal 55k→40k）——方向与量级自洽。
   ⚠️ CA_MIN 性能代价 +68%（260905-10 用户拍板默认关，翻默认需重新决策）。
3. **tree placer 缓装缺口（本轮新发现，非 B7-B12 已知项）**：tree.rs 仅支持 straight/fancy/mega_jungle
   trunk + blob/fancy/bush/jungle foliage；`spruce_foliage/pine_foliage/dark_oak/giant_trunk/mega_pine/
   acacia/cherry/forking/upwards_branching/random_spread` + `three_layers_feature_size` + `pale_moss/
   attached_to_leaves` decorator 全部 skip unsupported（整树不生成）。本 region 含云杉林/巨型云杉/黑森林：
   spruce_leaves 69,256 / spruce_log 11,077 等逐项 **deterministic**（两轮 bit 级相同）= 系统性缺口非 run 噪声。
   **不在 c1-exemption-list B5-B12 内，台账需扩充。**

## 验收判据判定

- 「信噪比回落噪声量级」**未达成**（3.65M vs 0.10M）——但残差主体 = ①缓装树族（本 region 结构性存在）
  ②kelp/seagrass 边界带（b2 §二 P6 已知）③CA_MIN 关闭时的 ore 级联。继续在 b2 五缺陷框架内推进无新证据空间。
- mask 翻转（T4）被 ① 阻塞：翻转后云杉/黑森林树将整族消失（现状由 Java vanilla 臂代画）。

## 待用户决策（人工 HOOK：scope 变更）

- D1：实装 tree placer 族（估：6 placer + feature_size + 2 decorator，中大量工作）→ 走完整对拍后重提 mask。
- D2：缓装台账路径——登记 B13+（tree placer 族），T3 验收判据改为「扣除缓装族后信噪比回落噪声量级」，
  mask 翻转改为「无缓装 biome 区域先翻」或推迟。
- D3：WG_CA_MIN 是否翻默认（+68% 成本 vs ore/geode 边界语义对齐）。

## §9.7 声明

- 对比口径三要素：同 chunky 载体、同 region/seed、与 260909-02 基线直接可比；diff 脚本同源
  （diff_javafeat_rustfeat/diff_baseline_260909-02.py）。
- 原始产物：.tmp/mc1216-closeout-260909-03/{diff-signal-260909-03.txt, diff-signal-camin-260909-03.txt,
  diff-baseline-260909-03.txt, rustfeat-seq-after.log(.err), biomeorder.log}；旧 region 已改名 *-prev 保留。
