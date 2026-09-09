# T1 根因修复记录 — 1.21.6 biome_registry_order（260909-03）

> 会话：mc-1216-features-takeover 续块。计划：架构计划-260909-03-features-takeover-rootfix.md。
> 状态：T1 完成（candidate 级证据，最终对拍归 T3）。

## T0 廉价独立验证（通过）

- probe-rustfeat-err-snapshot.log :13/:65/:78 = 三维度 registry_order 缺失实录；
- biome.rs:405-418 缺失回退路径 + :550-570 all_features_lists 排序消费点在位；
- glob 全仓仅 1.20.1 有该文件，1.21.6 data 目录缺失。
- 结论：根因方向（字典序回退 → PlacedFeatureIndexer p 域错位）验证通过，继承。

## 修复动作

1. 导出：复用 runtime/1.21.6/java 的 `BiomeSourceLogMixin`（`wg.seedlog` → `[BIOMELOG]`），
   `-PseedLog=1`（build.gradle:84）；脚本 `.tmp/mc1216-closeout-260909-03/biome_order_export_260909-03.ps1`，
   存档口径全参数（#15）。产出 **54 项 overworld 枚举序** → `versions/1.21.6/data/worldgen/biome_registry_order.json`。
2. 覆盖面核对：1.21.6 biome json 共 65；54 在序内，表外 11 = nether 5 + end 5 + the_void——与 1.20.1 文件构成同构
   （仅 overworld 序，55 项）。**禁止照抄 1.20.1 的要求达成**（cherry_grove 等 1.21 新 biome 已在手导序内）。
3. Rust 侧零代码改动（load_registry_order 运行时按 wg_dir 读取）。

## 验证（Full/Partial 探针）

- 短跑探针 `rustfeat_seq_probe_260909-03.ps1`（rustfeat 臂口径 mask=1 + `-PfeatureLog=1` + RCON forceload 触发
  chunk(0,0)——**踩坑补记：boot spawn 区不走该序列探针路径，[FEATURE] 行必须 forceload 触发**；WG_FEATURELOG 输出
  在 stderr → `$log.err`）。
- registry_order missing 行：**0**。
- chunk(0,0) step=9（k=9 植被段）13 项 (p,fid) 与 javafeat 快照（probe-javafeat-log-snapshot.log）**逐项全同**：
  p={0,50,57,68,73,74,82,88,89,101,102,103,105}，fid 顺序一一对应；修复前 p 全体错位
  （{0,28,53,71,76,77,82,87,89,94,97,101,102}）。

## §9.7 声明

- 本结论载体 = 单 chunk (k,p,fid) 序列探针（chunk(0,0)），覆盖面 = step9 植被段 p 域；区域级块对拍归 T3。
- nether/end 序未对齐（registry_order 表外落字典序追加）——沿用 260905-08 已声明近似，本轮区域对拍仅 overworld；
  风险单列，不在本课题闭合范围。
- 局限：Java 侧 [JFEATURE] 行无 fid 列，fid 对应关系以「13 项计数 + p 逐项相等 + 修复前 13 fid 集一致」间接锚定。
