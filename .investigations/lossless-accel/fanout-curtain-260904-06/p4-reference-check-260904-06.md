# P4 参照复验：幕帘课题前提坍塌（260904-06，主会话 decisive probe 记录）

> status: draft（数据层新证据，Facts 全部一手实测；解释框架待 worker/judge 重构）
> 触发：P-B1-2 判别探针执行中发现 C++ 生产密度为负 → 顺藤查参照 → P4 命中

## Facts（实测，一手）

1. **C++ 生产 densityBuf @ (195,199)（WG_DBDEBUG，`.tmp/p2full/cpp-dbdebug-195-199-260904-06.txt`）**：
   y256-318 = **−0.02**（b1 理论地板 −0.025 同量级），y192-255 = −0.46，y180-319 **全负**。
   → **「幕帘 = 0<d」前提在 C++ 臂证伪**。C++ 导出的幕帘 stone 不来自 d>0，只能来自 aquifer barrier margin（`density+e>0` 翻转，aquifer.h:121-137）——|d|=0.02 正是 margin 高发区。
2. **点采 vs 生产一致**：density_probe -dfDump final_density @ (195,199) y192-232 = −0.458 恒定（≈生产 −0.46）——插值语义无异常。
3. **P4 命中：vanilla 参照列 (195,199) y180-319 含 131 个 stone 族块**（stone 99 + granite 24 + copper_ore 7 + iron_ore 1（judge C2 措辞补正：族总数 131，「99」仅 stone 单项），`.tmp/p2full/ref_check_p4_260904-06.py` 输出）——**「vanilla y≥201 无 stone」为参照误读，writer-verdict-260904-06「真根因上移 NOISE」推理前提失效**。
4. **幕帘在三方都存在**：vanilla ref / C++ 导出 / mod 导出同列同带均有 stone 幕帘 → **幕帘 = vanilla 正常机制产物，非 C++/Rust 共有偏离**。 aquifer 高位水口袋（~16 间距）+ barrier margin stone 在 vanilla 同构存在（b3 已证 aquifer 三方 17 项零偏离——自洽）。
5. **真实残差现形（本列 y180-319，32 块全部同型）**：`C++ stone ← vanilla granite/copper_ore/iron_ore`——**C++ 缺高 y granite/copper 写者**（y 185-231，超出 oreVein y≤50 硬门 ore_vein.h:46；也超出 b3 亲核的 feature 上界 granite 128 / copper 112——vanilla y 217-229 granite 的写者身份成为新 idk）。
6. 列剖面另见 grass/water 交替（y 67-214 段，两臂逐位一致）——真实地形或 id 映射问题未查（新 idk，#8 raw id 家族嫌疑）。

## 结论（draft）

- **writer-verdict-260904-06 结论 3「真根因上移 NOISE（幕帘 = 共有 vanilla 偏离）」应被取代**（§15.4，待正式取代记录）：幕帘是 vanilla 机制，课题剩余 = ① C++ 高 y granite/copper 写者缺失（本列 32 块，或即 ore 族 desync 课题 3 的一角）② 260905/260904-06 各臂对比里哪些差异真的存在需重算（此前以错误参照前提归因）。
- b1 臂「B1 共享密度抬升」对**幕帘存在性**失去对象（幕帘不是偏离）；但 −0.02 带 + margin 机制与 b3 预测闭环。
- rust 臂数值剖面探针（Rust 侧 dump）暂缓——判别对象已消失。

## 下一步建议
1. 正式取代记录 + 本列残差并入「block_probe C++ 载具 ore 族 desync」课题（NEXT_SESSION 课题 3，tuff/andesite/granite/stone→coal_ore 族分解已有 22653 mismatch 底账）。
2. worker 重解读 260904-06 writer verdict 受影响结论（哪些归因需要重算）。
3. granite@y217 写者 idk：查 vanilla 1.20.1 高 y granite 真实来源（aquifer? surface rule on peaks? ore vein height 实测口径）。

## 探针产物
- `.tmp/p2full/cpp-dbdebug-195-199-260904-06.txt`（生产密度列 dump）
- `.tmp/p2full/cpp-fdcol-195-199.txt` / `cpp-inicol-195-199.txt`（点采 %.17g）
- `.tmp/p2full/{ref_check,ref_terrain,xcheck}_p4_260904-06.py` + 输出
- `.tmp/p2full/{fd,ini}_extract.json`（DF 节点抽取）
