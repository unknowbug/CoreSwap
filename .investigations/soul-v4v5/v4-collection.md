# V4 采集记录（生产链 soul 分支 ctx dump，260902-03）

## 方法
- patch：`surface_rules.rs` build_surface 内 env 门控点级 ctx dump（`WG_SOUL_CTX_DUMP`，OnceLock 进程级点集 + chunk 级门控，未配置零热路径成本）。
- 驱动：`WorldgenRust/src/bin-diag/soul_ctx_dump.rs`（临时挪 src/bin 编译，E9 流程）——create_for_dim(nether, seed B) + 逐 chunk 生产 fill_chunk_blocks，dump 生产 ctx。
- 点集：`.tmp/soul-mismatch-points.txt`（180 点，V2 签名 B/C mismatch 点），180/180 全命中。
- 产物：`cmd-output/soul-ctx-dump.stderr.txt`（181 行 [SOUL-CTX]）+ stdout。

## 采集层初步对照（非结论，交 worker 解读）
- 抽样点（3260,1,3200 / 3260,3,3200 / 3275,2,3201）：probe CSV（soul-selector-probe.csv）与生产 dump 的
  biome / stone_depth_above / stone_depth_below / surface_depth / selector **逐项全同**。
- probe stderr 的整规则 apply（外部重组 ctx）与生产 apply 结果**一致**（netherrack / netherrack / id=31）。
- → 「probe 复算输入 ≠ 生产 ctx」候选（V3 §2 候选①②的输入差分支）被采集数据否定。
- 新矛盾：biome=soul_sand_valley ∧ ceiling_ok（sdb≤1+0+surface_depth）∧ selector<0 → applied=netherrack(256)，
  与 V3「进 soul 分支必得 soul_soil 兜底」的结构推演冲突 → 矛盾在规则树求值/解析层
  （候选：V3 静态误读 JSON / parser 产物树 ≠ JSON 语义 / 求值语义差）。
- 附：3275,2,3201 applied=id=31 疑为 bedrock_floor（y=2 ∈ above_bottom 0..5）先中，非 soul 判定点。

## 附：block id 映射（versions/1.20.1/data/blocks.json 核对，260902-03）
air=0 / stone=1 / bedrock=31 / lava=33 / gravel=37 / netherrack=256 / soul_sand=257 / soul_soil=258 / blackstone=849

## 附：180 点生产 dump 分布（主会话统计，260902-03）
- biome：nether_wastes=90 / soul_sand_valley=90（**生产侧 biome 一半点判为 nether_wastes**——与签名 C「组3 entered 0/60」家族吻合）
- applied：256(netherrack)=103 / 31(bedrock)=77
- ceiling_ok=true 共 115；selector<0 共 93

## 待 worker 裁决
1. 上述矛盾的机制定论（读 nether.json [4] 分支原文 + parse_surface_rule/parse_surface_cond 产物语义）。
2. probe CSV 全量 180 点 vs dump 全量对照（不只抽样）——确认无任何输入差残点。
