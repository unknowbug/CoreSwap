# b3 决定性实验设计 — 单树 RNG 消费计数 trace（J3 短路族收口）

- 归属：.b3（jungle_l / J3）/ 状态：draft（设计，无 shell 执行——运行由主会话执行，原始输出回传解读）
- 目标：用「每棵 jungle_tree 的 blob 角 nextInt(2) 消费次数」trace 直接判 J3 成立/排除/降级。

## 一、采集方式

### Rust 侧（现有钩子自查 + 最小补点）

- 现有：`WG_TREEDIAG` env 门控（placement.rs:281-303，tree.rs:438-439/541-542）——已有 `[TH]`（getHeight）、`[CNT]`、`[SQ]`、`[BEE-MISS]`。
- **缺角消费点**：`is_position_invalid` Blob/Bush 角分支（tree.rs:247-258）无打点。补丁模板（一行，同 env 门控，位置在 next_int_bound 调用前）：
  ```rust
  if crate::placement::treediag_enabled() { eprintln!("[CORN-BLOB] y={} r={} @({},{},{})", y, r, cx+dx, tree_node_y+y, cz+dz); }
  ```
  （Bush 同构 `[CORN-BUSH]`；放在 `ax==r&&az==r` 命中后、`next_int_bound(2)` 之前，一行=一次消费）
- 运行：`WG_TREEDIAG=1` + features_probe / fill driver，目标 chunk 与 Java 侧同 chunk（wgdiag_firstrun.ps1 已用 (29,-16)，seed 8576294172403134396，log = `.tmp\jungle-l-260905-13\wgdiag-firstrun.log`）。

### Java 侧（WgDiag 通道）

- 现有 mixin：TrunkPlacerMixin `[THJ]` / ChunkRandomSeedLogMixin population 行 / CountPlacement `[CNT]` / Square `[SQ]` / Beehive `[BEE]`（runtime/1.20.1/java，local-only）。
- **缺角消费点**：新增 `BlobFoliagePlacerMixin`（@Inject 到 `isInvalidForLeaves` HEAD 或 Tail，@At HEAD + 参数 dx==radius&&dz==radius&&isClientVisible 时打印 `[CORN-BLOB] dx,y,dz`）。同 WgDiag curAllowed() 过滤（WG_DIAGCHUNK=29,-16），运行命令照抄 wgdiag_firstrun.ps1（-Ptreediag=1 -Pdiagchunk=29,-16）。
- 注意：fancy/LargeOak 无角消费，mixin 只挂 BlobFoliagePlacer（+可选 BushFoliagePlacer），不污染其他 placer。

### 判读输入

- 两侧各得目标 chunk 内每棵 jungle_tree（以 [TH]/[THJ] 分树）的 `[CORN-BLOB]` 行数序列（层序敏感）。
- 预测基线（静态推导，.b3 §1.1）：jungle_tree 每棵 16 行（4 层×4 角，j 序 [1,1,2,2]）；jungle_bush 每棵 8 行（j 序 [2,1]）。

## 二、判读判据（三行）

1. **成立（J3 实差）**：同一棵树（[TH]/[THJ] 同高同位）两侧 `[CORN-BLOB]` 行数或层分布不同（如 Rust 16 vs Java 12——y==0 层语义差）→ 消费漂移实锤，量化每树漂移 d，转入修复（对齐短路/y 条件）+ region 重测归因。
2. **排除（J3 关闭）**：两侧每树行数与层分布逐树一致（含 16/8 基线）且树间序一致 → 短路族在 jungle 域零贡献，J3 核销（记录取代：残余标签移除 blob 角项，转 J1/J2）。
3. **降级（不可判读）**：任一侧目标 chunk 无 jungle_tree（[TH]/[THJ] 树数 0 或两侧树数不同——上游 J1/selector 已漂移）→ 单树对拍不可达，J3 保持低先验持有，**先修 J1（确定性缺失）后重采**再判；此时判读权移交 J1 修复后的复测轮。

## 三、执行成本与顺序

- Rust 补丁 2 行 + Java mixin 1 文件；采集复用 wgdiag_firstrun.ps1 骨架（三查：seed 三处 / 坐标口径 / 参照完整——按 AGENTS 探针三查铁律）。
- 顺序建议：与 J1 修复后的重采**共用同一次采集**（一次 gradle run + 一次 Rust run 出双判据数据），不单独立项。
