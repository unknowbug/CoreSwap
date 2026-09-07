# 柱全列直读（开工点1，260907-01）

> 状态：主会话收敛型执行，数据层直读（Full 载体：region NBT 程序化解析）。
> 载体：Chunky 双臂 region（#26 confirmed），seed 8576294172403134396。
> 脚本：`.tmp/jungle-l-260906/column_read_260907-01.py`（region+slot 定位，#20 判据）。

## 结果

### 柱 (483, -230)（Rust live Y / jungle_log 柱）

- 定位：cx=30 cz=-15 → r.0.-1.mca slot=574 lx=3 lz=10
- **全列 y=-64→319 差异 = 0**（逐位全等，含地形与植被）。

### 柱 (500, -234)（h19→h13 漂移柱）

- 定位：cx=31 cz=-15 → r.0.-1.mca slot=575 lx=4 lz=6
- 差异 21 处，**全部在 y=74–94 feature/decoration 层**：
  - vanilla 侧：整根 jungle_log y=75–94 + 树下 grass_block→dirt（y74）
  - coreswap 侧：fern（y75）+ jungle_leaves（y77-78），无树干
  - **y≤73 地形逐位全等**
- 判读：该柱差异 = 特征流非确定（#67/#68 通道①语义：树放置 run 级分歧），**非地形残差**。

## 对交接结论的廉价独立验证（STEP 1 纪律）

| 交接结论 | 验证结果 |
|---|---|
| 通道②（Rust 地形输入差参与 483 柱分歧）draft ~0.15-0.25 | **支持降级**：483 柱全列地形零差，直证地形输入一致 |
| blob 石残差 ≈15 块/chunk 是否触及焦点柱 | **不触及**：两焦点柱地形层逐位全等 |
| 500 柱「地面低 6 格」（树基高度代理推定，f3-twochannel-260906-09 §5） | **取代（§15.4）**：region 载体下地形基座一致，差异为植被层（有/无整树）。取代记录反向指针：f3-twochannel-260906-09 归档时按取代链补注「500 柱 h 定性被 column-read-260907-01 取代」（judge N-5）。 |

## 遗留

- 全区域 799/3025 chunk 有地形差（剔植被口径，采集脚本 `collect_fanout_260907-01.py` → `diff_per_chunk_260907-01.txt` / `structures_nbt_260907-01.txt`）——归因交 fan-out .b1/.b2。
