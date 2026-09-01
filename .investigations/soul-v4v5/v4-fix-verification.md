# V4 修复与回归验证（布尔字段解析 bug，260902-03，draft）

## 根因（承 worker 裁决 .artifacts/.b2-soul/v4-eval-conflict.md）
- `parse_surface_cond` 用 `as_f64()` 读 JSON 布尔字段 `add_surface_depth` / `add_stone_depth`——`JsonValue::Bool` 走 `as_f64()` 返回 None → **恒解析为 false**（原 surface_rules.rs 三处：y_above / stone_depth / water）。
- soul 分支 ceiling 条件 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1+0+0` → 分支未进 → 穿透 [7] 兜底 netherrack。V3「soul_soil 兜底」推读本身正确，错在解析产物树；V3 静态对拍「参数全对拍」对拍的是 JSON 原文而非解析产物，被假阴性掩盖（教训入产物 §4）。

## 修复（surface_rules.rs）
- 新增 `SurfaceBuilder::parse_bool_field`：`as_bool().or_else(|| as_f64().map(|f| f != 0.0)).unwrap_or(false)`（兼容数字 0/1 与缺省）。
- 三处调用替换：y_above `add_stone_depth` / stone_depth `add_surface_depth` / water `add_stone_depth`。

## 验证链
1. **树复现**（soul_tree_repro）：修复前解析树全 asd=false（假阴性 8 处 JSON-false 掩盖）；修复后 5 处翻 true（soul ceiling/floor StoneDepth、gravel patch y_above 30/35、basalt floor stone_depth），其余 8 处保持 false=JSON 原值。产物 `cmd-output/soul-tree-repro-postfix.txt`。
2. **生产 ctx dump**（soul_ctx_dump，180 点）：netherrack 103→71，新增 soul_soil=18 / soul_sand=14；定点 3260,1,3200（sdb=2, sd=3, selector<0）applied 256→**258(soul_soil)**，与 V3 语义推演逐位一致。产物 `cmd-output/soul-ctx-dump-postfix.stderr.txt`。
3. **nether 存档全量回归**（run4 模板照抄，seed B 4×4@3200,3208，ReadWorldProbe 存档口径）：
   | 轮 | 修复前 | 修复后 |
   |---|---|---|
   | 对齐率 | 94.4241%（990108/1048576） | **run1 96.6215%（1013150/1048576）/ run2 96.5866%（1012784/1048576）** |
   - +2.20pp；run1/run2 差 366 块，在已知同 dll 重跑非确定容差带内（workflow-patterns 发现 #10，2330 块带宽）。可比性：同参照（FULL ref seed B）、同 region、同口径，与 94.42% 历史值可比。
4. **soul 族 per-id 佐证**（.tmp/soul_per_id.py，save MCA vs FULL ref）：
   - soul_soil：修复前 1334 → **5771**（ref 5474，+297）；soul_sand：1471→2494（ref 2457，+37）——soul 族闭合至 ref 邻域。
   - quartz 2095（ref 1992）/ gold 711（728）/ magma 1543（1533）/ gravel 674（674）——矿石/杂项近邻域。
   - basalt −1736 / blackstone −434：B1 surface 残差家族（已知遗留，非本次范围）。修复前后对照（同 seed/region/口径）：basalt save-ref 修复前 −3631（NEXT_SESSION 260902-02）→ 修复后 −1736——asd 翻转无新负迁移，B1 家族反而收敛。

## 附带修复影响面
- nether gravel patch 高度带（y_above asd）整体修正；nether_wastes 签名 C「soul_sand_layer 分支 entered 0/60」同 bug 源（L579 add_surface_depth），随本修复闭合（soul_sand 2494 vs ref 2457 佐证）。
- 3275,2,3201 系 bedrock_floor 先中，从签名 B 证据集剔除。
- overworld 不受影响（代码规则树，不走 parse_surface_cond）。

## 状态
- 结论候选：根因（布尔解析 bug）+ 修复有效性——待 judge。
- 残差（idk/遗留）：basalt/blackstone B1 家族；366 块非确定带宽；V5（biome 边界带 vs vanilla 足迹）——修复后残差图需重导（readWorldProbe mismatch 全集），本次未做，残差已降至 ~3.4%。
