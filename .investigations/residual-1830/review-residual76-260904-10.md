# review-residual76-260904-10.md —— core.judge 审查意见（残留 76 surface 微族：根因 + 修复申请 candidate）

> 角色：core.judge（subagent 隔离，只出意见不改 status）
> 审查对象：残留 76「surface 微族」根因结论（BiomeAccess zoom 缺失）+ 修复（commit ab56706）+ decisive 验证申请 candidate
> 三源核对方式：judge 无 shell——① 产物/证据读盘核对 ② 代码应用版直接读现行文件（worldgen_handle.rs / biome.rs / C++ biome.h/surface.h/worldgen_api.cpp）③ 验证记录读 verify 脚本 + 探针原始输出（decisive 运行输出本体未落盘，见 C-2）

---

## 分项结论

### ① 根因结论（主族 73 块 = surface biome 输入缺 BiomeAccess zoom）—— **PASS**

- Java 一手证据链成立：cmd-output/java-sscan-source-SurfaceBuilder-1.20.1.txt L108（`biomeAccess::getBiome` 传入 MaterialRuleContext）+ L119（列级 biome 也走 `biomeAccess.getBiome`）；claim 引 ChunkRegion L102 hashSeed，与 surface_rules.rs L613-619 注释口径一致。
- 修复前行为实锤：fill_chunk_blocks 旧 `biome_at`（worldgen_handle.rs L609-612）= `(x>>2)<<2` 单元直读、无 jitter 选点——与 Java 生产路径确有机制差。
- 签名吻合（workflow-patterns #15）：dump 76 块 dtop 全 0 / biome 通道双侧同 / 成簇 patch 边界带 / 双向互翻（gravel↔sand），与「BiomeAccess 8 邻域选点在 biome 边界带（deep_ocean↔deep_lukewarm，dump L11-12/18/25 raw[20]↔raw[29] 交界）两侧选点不同 → sand/gravel 分支翻转」形状一致。b2 候选（噪声 patch）经一手核伪：海洋段 sand/gravel 选择器纯 biome 等值匹配无噪声条件（candidate-76-b2 §2.1 逐行定位 L1020-1037），排除合理。b4 候选诚实降级为「零星 3 块不归 s 语义」，处理得当。
- fan-out 纪律合规：scout 预置 5 候选分叉（§5）→ .b2/.b4 并行 worker 候选落盘 → 收敛。

### ② 修复（ab56706：biome_at_surface + picked-cell packed 缓存 key）—— **PASS（附 C-4 半修 CONCERN）**

- 代码应用版核对：worldgen_handle.rs L50 `biome_access_seed` 字段 + L389 `biome_hash_seed(seed)` 构造 + L620-624 `biome_at_surface`（biome_pick_cell(self.biome_access_seed,...)）+ L629-636 仅接 build_surface；packed key L632-635 带 u32 截断防负坐标符号扩展，注释声明对齐 C++ biomeCellKey——与 worldgen_api.cpp L1080-1099 接线同构（L359 h->biomeAccessSeed = biomeHashSeed(seed)）。
- biome.rs L196-299：sha256 手写实现 + biome_hash_seed（le bytes → 前 8 字节 le asLong）+ SeedMixer/jitter/cell_distance/biome_pick_cell，与 Java BiomeAccess 语义逐项同构；C++ biome.h L79 biomeHashSeed 同源参照在盘。
- 注释 L618-619 明确声明 carver/feature seed 口径疑点「本轮无残差证据不动」（#36 覆盖面声明）——诚实标注合规。

### ③ decisive 验证（76→12，99.999%）—— **PASS（附 C-2 证据落盘 CONCERN）**

- verify_surfacefix_260904-10.py：seed 三查 assert（ref/prev/new 同 seed=8576294172403134396）+ 三方对比（baseline ref↔off-aquifix-09 / new ref↔off-surfacefix-10）+ 76 样本 spot check——脚本设计正确（独立 diff 复算，非复用结论）。
- 双臂产物在盘：ref/ 与 off-surfacefix-260904-10/{blocks, blocks.json}，dump 头部四查过（L1：seed/origin/size 双侧一致）。
- judge 未独立复算（无 shell）——命令模板见「补强要求 R1」。decisive 结论暂以「脚本+输入在盘、输出在会话记录」背书，落盘后升实。

### ④ 剩余 12 块归因 —— **PASS（归因诚实）**

- ore_vein 域：res76-probe CSV L10-23：(237,41,224) CS pre=stone post=gravel，vanilla y41=granite——ore_vein 漏放直接可读；且 b4 候选 §2.1 已独立论证「surface 只对 stone 动手，granite 必归 ore_vein/feature 域」，双源一致。
- aquifer 域：(198,18,198) pre=stone post=dirt、y19 water——pre-surface 固/液差一格、surface 忠实级联（b4 §2.3），CSV L24-30 剖面吻合。
- gravel→sand 残 9「未立案」：claim 中明确单列，未混入主结论——诚实度 PASS（但见 C-3）。

### ⑤ P0 附带发现（docs/06 L62/L94 s-scan 口径失准）—— **PASS**

- 一手源码实锤：SurfaceBuilder.java L181-183 `isDefaultBlock = !isAir && fluidState.isEmpty()`（非空非流体，非 ==stone）；L144-151 s-scan 在首个非 default 处停。Rust L1304（实心继续）/C++ surface.h L781 同「实心集合」——两侧本就与 Java 一致，docs/06 需修正而非改码。结论与一手源逐行吻合，判「docs 失准」成立。

### ⑥ open 疑点（carver/feature 裸 seed）—— **CONCERN（风险评级：中，非本课题阻塞）**

- 已核实：Rust worldgen_handle.rs L722/L849 `biome_pick_cell(self.seed, ...)` 用裸 seed；**C++ 侧全部 pick 站点（worldgen_api.cpp L646/1060/1082/1498）均用 biomeAccessSeed（hashed）**——这是 Rust↔C++ 实装分歧，不只是「与 Java 口径待核」。Java 侧 carver/feature 同样从 ChunkRegion.getBiomeAccess() 取 biome，大概率同 hashSeed。
- 判级理由：本 seed/区域 12 块残差无 carver 域签名支撑（b4 勘探未命中），当前观测面小；但一旦换 seed/区域暴露 carver 域 biome 依赖差异，回溯成本高。建议立「待办」而非「不修定论」，下轮换 seed 回归时低成本 A/B（裸 vs hashed）。

---

## 落盘契约 / 置信度 / 流程检查

- **C-1 产物契约缺口（FAIL→候选授予前 MUST 补）**：本课题根因结论**无任何 verdict 产物落盘**——.artifacts/index.yaml 无 residual-76 条目（grep surfacefix/residual76 = 0 命中）、.investigations/residual-1830/ 只有 scout + .b2/.b4 候选（均 draft），无 verdict md。结论目前只存在于代码注释 + 会话记录，违反「产物不得只留在对话里」（core.artifact）。code 注释引用的「残留 76 verdict」实际不存在（悬空引用）。
- **C-2 decisive 运行输出未落盘**：76→12 输出只在会话记录；`.tmp/p2full/` 无对应 cmd-output 快照。违反证据可引用性（Anchorlaw §5.5 / spec §1.3）。
- **C-3 残 9 未立案可接受但须登记**：应作为 NEXT_SESSION/台账待办条目存在，而非仅 claim 文字。
- **C-4 半修一致性**：biome_at_surface 只接 build_surface 是**有意的覆盖面限定**（注释已声明），本身合规；但与 ⑥ 合并看，Rust carver/feature 与 C++ 分歧是实装漂移信号，建议在 verdict 的「覆盖面」节显式写「surface 域已修，carver/feature 域口径待核」。
- **C-5 sha256 无 known-answer 测试**：decisive 端到端通过只能证明「该 seed 下 hashSeed 未见错」——sha256/putLong 换端序类错误在同一 seed 上可能被 8 邻域选点鲁棒性掩盖。candidate 可授予（行为证据足），confirmed 前建议补 KAT（对拍 C++ biomeHashSeed 或 Guava 已知向量，任一 seed 即可 + 负数 seed 一个）。
- **retry cap / 模块边界 / 验证者分离**：单轮修复不涉超限；scout/worker 角色标注齐全、无跨模块 skill 正文引用；decisive 由主会话执行（符合 subagent 无 shell 契约）——均 PASS。

## 推荐状态

- **根因结论 + 修复：建议授予 candidate**（decisive 行为证据充分、机制链一手核实、候选排除有据）——**前置条件 = C-1/C-2 补齐**（verdict md + .artifacts 条目 + decisive 输出落盘；无此二项只能保持 draft）。
- **confirmed 补强清单**：R1 judge 独立复算（下）落盘；C-5 KAT；P0 docs/06 修正落盘（subagent 草稿流程）；C-3 残 9 立项登记；⑥ carver seed 疑点至少完成一次跨域 A/B 探针或正式 @anchor.idk 挂账。

## R1 judge 独立复算命令模板（主会话执行，输出落 .investigations/residual-1830/cmd-output/）

```powershell
$env:PYTHONIOENCODING="utf-8"
python .tmp/p2full/verify_surfacefix_260904-10.py *> .investigations/residual-1830/cmd-output/verify_surfacefix_rerun_judge.txt
```

判据：residual: 76 -> 12 复现、spot checks 中主族代表点 new 列与 ref 一致（(237,41,224)/(237,42,224)/(198,18,198) 保持差异属预期，见 ④）。
