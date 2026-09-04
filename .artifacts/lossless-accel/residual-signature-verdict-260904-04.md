# residual-signature-verdict（260904-04 探针裁决轮 · 状态 candidate）

## 结论
1. **id 注读纠错**：260904-03 top 对「ref=1→9(water) n=4165」系误读——blocks.json 实测 9=dirt、32=water、34=sand、37=gravel、1=stone、2=granite。top 对真义 = stone→dirt / gravel→sand / stone→sand / gravel→dirt / water→air 等。
2. **残差真签名**：blob 状 stone/gravel/花岗岩族→dirt/sand 置换（最大连通域 615 cell），y 全高均匀（每 y≈35）。
3. **三候选封闭**：aquifer floodedness / 流面 est 翻转 / surface-carver 级联（❌❌❌，y 分布不符）。
4. **b3 伪影排除**（per-chunk 对账精确）；**b2 主导排除**（非 stone 源残差 ~41% 无法由 surface 产生；disk 跨度差 28×）；**b1 部分成立非全貌**。
5. **消融判别无效**：WG_FEATURE_SKIP 归零实验被 target 谓词状态耦合污染（skip 后 ore_granite 落点全变）；RNG 隔离成立（setDecoratorSeed 完整 setSeed）。
6. **载具差异**（详见 .investigations/lossless-accel/vehicle-verdict-260905.md）：block_probe 全程 vs mod cppReplace 残差族结构不同（ore 族 98.56% vs stone→dirt/sand 族 99.1526%），禁止互引。

## 证据源
- `.investigations/lossless-accel/fanout-residual-260904-04/`（b1/b2/b3 + phase2-probe-round-260904-04.md）
- `cmd-output/`（origin-audit / skip-ab-parse）；`.tmp/p2full/`（cmp 脚本与数据）
- blocks.json 双处反查（.tmp/p2full/blocks.json 与 versions/1.20.1/data/blocks.json 一致）

## §9.7 口径
WGB2 FULL 存档口径 4×4 @ chunk(200,200)，16 chunk×98304 cell，seed 8576294172403134396。

## 取代链（§15.4）
supersedes 07-block-pipeline.md「存档口径残差模式化(260904-03)」定性 + 10-timewise-archive L2844（取代记录已就地标注，260904-05 落盘）。
superseded_by（260905 部分取代）：`.artifacts/lossless-accel/writer-verdict-260905.md` —— 本 verdict 中「mod 载具出界 dirt 写者未定位」课题已收敛（最佳解释 = Rust features 双跑伪影），mod 载具残差课题结案重定向至 block_probe C++ 全程载具（ore 矿石族 desync）；本 verdict 的 id 纠错与三候选封闭结论不变。
