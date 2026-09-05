# 260905-09 oak 恶化勘探中间记录（Phase 1-2，滚动更新）

## fan-out 判决进展（260905-09 晚）
- **b2（biome）birch 路径 refuted**：P1 FEATURELOG 重放 chunk(37,-16) 显示 step=9 p=20 `trees_birch_and_oak` **在执行**，但 rust 侧 birch_log/leaves 仍全零（45 棵全 oak，p≈6e-4）→「feature 没跑」排除；剩余候选 = selector 内部 RNG/分支差（b1 域）。产物 260905-09-oak-b1.md / -b2.md / -b3.md。
- **b3（地形/高度图）前提死**：b3_terrain_diff.py 实测 (37,-16)+8 邻域 9 chunk 地形逐列 diff=0/256 → 高度图传导排除。产物 260905-09-cmd-output-b3-terrain.txt。
- **c-A 意外实锤（读腿+写腿都活）**：b3 P1 副产物——van-only oak_log 集中边缘 d=0（11/22）= Java 边缘树 rust 缺失签名。
- **c-A 实施过程抓到一个接线 bug**：初版 c-A-min 只路由了 fctx.block_at（placement 谓词），树冠读/写走 OreFeatureContext::block_at（region_col_at=None 恒 -1）→ ca1 dump 零效应（out_reads=23/pending_writes=0）。修复 = OreFeatureContext 新增 block_at_ext 钩子（feature.rs）+ apply_features 注入同一闭包（worldgen_handle.rs）。修复后单点 out_reads=2558 / pending_writes=2437。
- **门控纪律**：WG_CA_MIN（默认关）；WG_CA_LOG chunk 级诊断计数。
- ca0（门关）== pfix 2193 chunk 逐位一致 → **重构零语义变化 PASS**。
- 待：ca1b（octx 修复后全 region）聚合树族 signed 判定 c-A 修复效应（预测：jungle/vine 收敛、oak 不作收敛指标——方案 §3）。
- 未闭合：birch 全零（b1 selector 域）继续追。

## 已验事实（数据层）
- 区域 #52 载体（seed 8576294172403134396, 2193 chunks, region_pfix.bin vs ab-vanilla-region）：
  - oak_leaves +29,830 高度弥散：28 chunk |diff|>500 全部正向（rust 多），仅 1 chunk 负向（v18 输出）
  - 区域级 oak_log 仅 +197 —— 与「更多树」签名不符（树叶/log 比失衡）
- 单点 chunk (37,-16)（v19，jungle/vine 全零干净样本）：
  - oak_leaves van=307 rust=1019 (+712)；oak_log van=28 rust=51；**birch_leaves van=55 rust=0**
  - 树位置整体错位：van-only logs 22 棵 / rust-only logs 45 棵，坐标完全不同
  - extra 叶 905 / missing 叶 193 / common 仅 114 → 树冠基本不重叠
  - extra 叶 Y 桶 y72-84，距最近 rust log Chebyshev d=1..3 为主（正常树冠形态）
- 交接假设核验：候选①「结构缺失改地形→树高度图不同」与 v19 签名**不完全兼容**——树 x/z 位置整体错位说明 RNG 流或 biome 输入在植被分配层已分叉，不只是 y 高度差（若仅高度差，x/z 应一致）。但 chunk (29,-16) 的 k=6/k=9 p 序列此前已验逐位一致（260905-08 交接）→ 分叉点在 seed 赋值**下游**（feature 执行内部）。

## 新数据文件
- .investigations/feature-parity/260905-09-cmd-output-v18-oak-worst.txt（区域 per-chunk 分解）
- .investigations/feature-parity/260905-09-cmd-output-v19-oak-singpoint.txt（单点空间签名）
- .investigations/feature-parity/260905-09-cmd-output/java-seedlog-chunk24-13_26-14.txt（java SEEDLOG：decorator 行不带 chunk 坐标，需按 Worker 线程分组归属；每 chunk 出现 2 个 population seed，归属机制待查）
- 采集脚本：.tmp/feature-parity-260905-08/v18_oak_worst.py、v19_oak_singPOINT.py（vanilla .mca 逐位解码：section 内 idx=y*256+z*16+x）

## fan-out 候选（互斥机制）
- b1: RNG 流下游消费分叉（vegetation feature 执行内部：count/树选择/消费序）
- b2: biome 归属差异（birch 全零签名；java 侧该 chunk 疑 forest，rust 侧待查）
- b3: 地形/高度图差异传导（结构 start 缺失 / Fix-1 邻域并集影响 heightmap）

## 环境锚点
- seed=8576294172403134396；rust dump: .tmp/feature-parity-260905-06/dumpnew/region_pfix.bin（idx=lx+lz*16+ly*256，miny=-64,h=384）
- vanilla参照: .tmp/feature-parity-260905-06/ab-vanilla-region（.mca NBT）
- java 日志: .tmp/feature-parity-260905-08/seedlog-vanilla.log（6MB，SEEDLOG mixin，-PseedLog=1 通道）
- rust 日志: .tmp/feature-parity-260905-08/rust_featurelog3.err（仅 chunk(29,-16)，48 条，18:48 = pfix dump 18:50 前的同批）
- subagent 无 shell：所有运行时采集由主会话执行命令模板
