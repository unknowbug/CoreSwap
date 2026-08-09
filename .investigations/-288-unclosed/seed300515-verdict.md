# 种子 3005152118058349760 (-1320400,-198049) 区域差异判定（draft）

> 课题：用户提供新 seed 3005152118058349760 + 坐标 (-1320400,-4,-198049)，实机观察「Java 版和 C++ 有很大不同」，怀疑隐藏 e 值翻转问题的解法。
> 方法：BlockProbe FULL 参照导出（新增 -PblockProbeFull=true 支持）+ block_probe 对比 + 差异归类。
> 状态：**draft**（判定完成，待用户拍板/交下 session）

---

## 一、结论（TL;DR）

**该区域 FULL 差异 94.13%（nonAir 81.89%）= 全部范围外 FEATURE 产物**（岩石替换 61881 + 矿石 5714 + 树草 4381 + 洞穴 6000+ + dirt/gravel 团 12000+）。**C++ 核心（density/aquifer/surface）在 SURFACE 状态 99.9986% 逐位对齐，无核心 bug。** 用户实机看到的「很大不同」= FEATURE 缺失（C++ 未实现 carvers/岩石替换/树草/矿石），与 -288 破案结论完全一致。

**关键旁证：陆地区域无 water↔terrain 差异（仅 1 块）→ e 值翻转（海底边界机制）只在海洋/含水层场景触发**——支持 B3 机制（液面网格输入差），缩小排查范围。

---

## 二、数据采集

1. **参照导出**（MC 工程 BlockProbe，修改：L796 支持 `-DblockProbe.full` 选择 FULL/SURFACE；build.gradle 加 `-PblockProbeFull` 映射）
   - `vanilla_3005152118058349760_4_-1320400_-198064.blocks`（FULL，含 FEATURE；chunk(-82525,-12379) 起 4×4）
   - header 核对：magic=0x57474232 seed=3005152118058349760 size=4 origin=(-1320400,-198064) ✓
2. **block_probe 对比**：TOTAL match=1480577/1572864 (**94.1326%**) nonAir=398193/486243 (**81.8918%**)
   - 每 chunk ~93-95%，非空气差 ~19-20%——用户实机可见的显著差异
3. **差异归类**（m300515_run1.txt 92287 行）

## 三、差异构成（92287 块）

| 类别 | 块数 | 来源 |
|---|---|---|
| 岩石替换（stone→diorite/granite/andesite/tuff） | 61881 | ore_diorite/granite/andesite/tuff FEATURE（范围外） |
| dirt 团（stone→dirt 3657 + deepslate→dirt 354） | 4011 | **ore_dirt FEATURE**（地下 dirt blob，范围外） |
| gravel 团（stone→gravel 5413 + deepslate→gravel 2845） | 8258 | ore_gravel_forest/ore_gravel FEATURE（范围外） |
| 矿石（coal/copper/iron/gold/diamond/redstone/lapis） | 5714 | ore_* FEATURE（范围外） |
| 树草（air→oak_leaves/birch_leaves/log/planks/grass） | 4381 | trees_flower_forest FEATURE（范围外） |
| 洞穴（stone/deepslate→air/cave_air） | 6370 | CaveCarver FEATURE（范围外） |
| 其他（cobble/calcite/smooth_basalt 等） | 907 | 结构/紫晶洞 FEATURE（范围外） |
| **water↔terrain** | **1** | **无海底边界差异（陆地 flower_forest/plains，无海洋）** |

## 四、样本列验证（colview300515_surface.py）

- **C++ surface 规则正确**：col(-1320358,-198033) y=77-79 地表层（dirt+dirt+grass_block）C++ **一致**（无差异标记）
- 差异 dirt/gravel 团全在**地下**（y=33-44、y=56-70，被洞穴 air 分隔）→ ore_dirt/ore_gravel_forest FEATURE
- 用户坐标列 (-1320400,-198049) y=-15..15：deepslate 层差异 = granite/tuff/andesite 岩石替换团 + gravel + iron_ore（**全部范围外 FEATURE**）；y=-4 附近 `deepslate→granite` 31 块等

## 五、与 e 值翻转问题（-288 海底边界）的关联

- **陆地（本区域）无 water↔terrain 差异 → e 值翻转不触发**。原因：e 值翻转依赖 aquifer 液面网格输入（fl2.y≠fl3.y，如 63 vs -32512），陆地无海面/液面场景 → fl2.y==fl3.y==63 → e=0 → 无翻转 → C++ 判水与 Java 一致（都判 air/无液面）
- **支持 B3 机制**：e 值翻转确实是「海洋/含水层专属」机制，修复点 = C++ aquifer 液面链在海洋场景的输入值（fl2/fl3/fl4 的 y 或 est 邻居值），与 -288 海底边界 6710 块同源
- **该区域与 -288 的差异性质不同但结论一致**：-288 = 海洋（cold_ocean 63%）有海底边界 e 值问题 + FEATURE；本区域 = 陆地（flower_forest 54% + plains 33%）无 e 值问题，纯 FEATURE 差异

## 六、对「卡住问题」的判定

用户实机观察到的「很大不同」**不是 C++ 核心 bug**，而是 FEATURE 缺失（C++ 未实现 carvers/岩石替换/树草/矿石/ore_dirt/ore_gravel 团）——与 -288 破案结论一致。**e 值翻转问题仍是独立的海洋/含水层机制**，需按 Phase 3 计划：Java 真实遍历中间量 dump 判别 (b) 液面网格输入值。

**该区域的验证价值**：
1. 作为「FEATURE 实施（carvers + 岩石替换）」后的回归验证区——实施后该区域应从 94.13% 显著提升
2. 陆地场景证明 C++ 核心无 bug（SURFACE 状态 99.9986%）——增强对 B3「海洋专属机制」的信心

## 七、产物

- `m300515_run1.txt`（92287 行 FULL 差异）
- `classify_full_mismatch.py` / `refine_core_pairs.py` / `colview300515.py` / `colview300515_surface.py` / `user_coord_check.py` / `stat_blocks2.py` / `dump_pal.py` / `check_header.py`
- MC 工程：BlockProbe.java L796 FULL 支持 + build.gradle blockProbeFull 映射（本地 M 状态，待提交）
