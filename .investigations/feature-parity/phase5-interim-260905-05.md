# Phase 5 运行时验证中间记录（260905-05，feature parity，主会话）

状态：draft（进行中）

## 1. 导出管线重建（phase4-status 开工点 1 完成）

- 首选载体落地：vanilla 参照不重导（.tmp/light-g1/vanilla-region），只重导 Rust 侧。
- 执行体三元组核验 ✅（知识库 #36）：target/release/worldgen.dll 新鲜（D97995B9…）= build/resources 副本 hash 一致；日志 `[coreswap] 已同步 Rust dll: 2056704 bytes` + `[LightRust] lightInit ok` + `[CppBridge] init seed=8576294172403134396`；Done 18.3s。
- seed 双核 ✅：server.properties level-seed + world/level.dat 实测 = 8576294172403134396（charter 目标）。
- **导出坑（本轮新发现）**：①Done 后 2s 停服 → spawn 排队 chunk 存成整柱 air（213 个）；②空转等待不排队列（生成需求驱动）；③老导出的 3377 覆盖来自 rcon **forceload** 定向预生成（#14/#18 的 pregen 手段考古落单：`forceload add` 8×8 chunk tile 循环 + region 体积收敛轮询 + rcon 每条独立连接 180s 超时——forceload 单条同步执行会打断 10s 默认超时的共享连接）。脚本：.tmp/feature-parity-260905-05/export_rust_region.ps1 + pregen_forceload.py。最终导出 5313 chunks / 12 region。

## 2. 可比性口径（§9.7 三要素声明）

- 空柱洞甄别：vanilla g1 与旧 rust g1 均有 2044 个整柱 air（预生成外圈常态，坐标完全一致）——**3377 全域统计口径有 60% 是空对空，不可比**。
- 可比集 = 双侧地形完整（solid ≥1000 块）交集 = **1333 chunk**。本轮所有数字均此口径。

## 3. 隔离定责（对照基线归因法 #16）

| 口径（1333 可比 chunk） | 差异实例 | 主签名 |
|---|---|---|
| vanilla vs 旧 dll（g1，改动前） | 155,470 | leaves/vine/log（Y4-6，10k 级）+ 中等 blob——**= G2 遗产签名复现**（phase4-status 记的 655 差 chunk 同口径吻合） |
| vanilla vs 新 dll（工作区改动后） | 2,298,206（**14.8×**） | **blob 爆炸**：tuff/andesite/diorite/granite rust 多 ~786k，Y -1..4 全带均匀偏高 |

**定责**：工作区 Phase 4a/4b 改动在 blob 置换层引入回归（phase4-status 风险移交项「不止单调改善」实锤）。树族签名量级新旧相近（oak/jungle leaves 各 ~10-16k），树部分未见爆炸。

## 4. 定量切割（v4_blob_cut.py，andesite 代表）

- 旧 dll：739.7/chunk vs vanilla 738.8，count-equal 1112/1333（83%）→ 近齐。
- 新 dll：881.4/chunk（**+19%**），count-equal 685/1333（51%）；Y 域不变（-1..4），全带均匀偏高 15-19%，无单带尖峰。
- 切割结论：**blob 数量/个数层语义变，非高度分布、非位置序列**。互斥候选 ≥2 → fan-out b1（count/IntProvider 采样）/ b2（modifier 链消耗 + discard_chance 层）。

## 5. 待办

- b1/b2 候选回 → 收敛 → 修（Phase 4c）→ 重导重验（判据：blob 差回旧 dll 155k 量级 + 树族归零方向）→ judge MUST → 提交。

---

## ⚠️ §3-§4 结论被取代（260905-05 晚些，supersedes 上文「隔离定责/定量切割」节）

**取代记录**：上文「工作区改动引入 14.8× blob 回归」的结论**被推翻**。后续证据链：
1. b1（DENY，.artifacts/feature-parity/b1-candidate-260905-05.md）：count/IntProvider 均为 JSON 常量，假设无作用面；机制算术（+142.6 块 ≈ +2.23 blob vs 尝试 2.17/chunk）不支持。
2. b2（基本 DENY，b2-candidate-260905-05.md）：discard/count 子机制均排除；set_decorator_seed 每 feature 独立播种，patch RNG 改动不传播。
3. 服务器侧考古（WG_FEATURELOG 硬开实验 + 纯 Rust features_probe 载体对照）：
   - **[CppBridge] init 发生在 Done 之后** → 启动区（pre-Done）= vanilla 装饰；接管只覆盖 init 后生成的 chunk。
   - 服务器 run 的 chunk 完成度随停止时机剧烈变化（同位置不同 run andesite 551↔828）。
   - g1 vanilla 参照有 2044 空柱 + 完成度存疑；recheck/vanilla_*.blocks 参照 id 域错位（无 andesite=6/diorite=4——C++ compact id 域，#9 家族），不可与现 blocks.json 域混用。
4. **修正后的结论（candidate）**：「新旧 dll 15× 差异」主要是**不同完成度/装饰状态数据的伪对比**（#18 家族变体）；工作区改动是否真回归，须以同方法论受控 A/B 定案。b1/b2 的静态对拍结论（常量 count、RNG 隔离）仍有效，反向支持「改动不应对 andesite 产生大面积影响」。

**定案实验（进行中）**：同方法论受控 A/B——A=cppVanilla（纯 vanilla）vs B=接管模式，同 fresh world/同 forceload 域/同等待窗口；再按需 B'（stash 工作区改动后的 old dll）。判据全部用 name 域 region 对比（v3 口径），废弃跨 id 域对照。

## ✅ 定案实验结果（260905-05，受控 A/B 完成）

- 双臂同方法论（fresh world / Done / 42 tile forceload 残差域 / region 收敛轮询 / stop 后拷贝）：ab-vanilla-region / ab-takeover-region，各 5313 chunks。
- **旁证（先于判决）**：ab-vanilla vs g1-vanilla = 2,292,126 实例差异（stone/deepslate 少 + tuff/andesite/diorite/granite 多）——与所谓「新 dll 回归」签名完全一致，而该臂零 rust 参与 → **伪影定案**：g1 vanilla 参照缺 blob 装饰层，此前「回归」= 完成度伪差。
- **判决对比（ab-vanilla vs ab-takeover，双侧地形完整 3009 chunk）**：
  - diff = **239,594 实例 / 1829 chunk = 80/chunk**（旧 dll 口径 155,470/1333 = 117/chunk，**净改善 ~32%**）。
  - 主签名 = 树族（oak/jungle leaves、vine，Y4-6 主导）+ 双向 ore 小残差（andesite/granite/diorite 各 1-1.2 万，双向均衡）——符合 idk-7（selector/patch 占位）与 R-1（HashSet 桶序）已知偏差源预期，无 blob 爆炸。
- **结论（candidate，待 judge + 用户拍板 confirmed）**：工作区 Phase 4a/4b patch 无回归、净改善；「14.8× 回归」结论作废（上文取代记录有效）。剩余树族残差 = 已登记 idk 的显式转出项。
- 判决脚本：.tmp/feature-parity-260905-05/v8_ab_verdict.py（含 v7 完成度伪差量化）。
