# aquifer→blob 残差课题 · 探针裁决轮中间记录（260904-04 主会话）

> 状态：draft / 中间产物（非结论）。上游：fanout-residual-260904-04/b1/b2/b3 三候选草稿。

## 已执行实验与结果

### 实验① origin 审计（b1 建议模板，WG_FEATURELOG=1 + [ORIGIN] 扩展）
- 修改：worldgen_api.cpp genFn 内 [ORIGIN] 过滤扩为 ore_granite|ore_dirt|ore_gravel|disk_（本次构建现役）。
- 结果（origin-audit-parse-260904-04.txt）：
  - ore_dirt：16 chunk × 7 = 112 次，origin y ∈ [0,158]，**零出界** → height provider 误用/第二 YOffset 缺 -1 假设不成立
  - ore_gravel：16 × 14 = 224 次，y ∈ [-61,316]（全量程 ✅ 与 JSON above_bottom 0..below_top 0 一致）
  - disk_sand：**0 次执行**（期望 48）→ 本区域无水面 disk（内陆），sand 残差非 disk 来源
- 判据 A（出界实锤）未触发；判据 B（次数偏差）未触发。

### 硬缺口推理（vein 展宽核算）
- feature.h:141-147：vein y 起点 = origin ± nextInt(3)-2（±2）；半径 ≤ ~2.5（size 33）→ 纵向展宽 ≤ ±5。
- origin ≤158 + 5 = 163 ≪ 200 → **y>200 的 stone→dirt（1362 cell）确定非 ore_dirt 所写**；y<-32（298）同理。
- 出界 dirt 写入者仍未定位（候选：surface rule windswept 分歧（b2 旁支）、植被/其他 feature、pendingCross、mod 路径独有阶段）。

### 实验③ feature 归零（WG_FEATURE_SKIP=ore_dirt,ore_gravel,disk_）
- 门控已加 worldgen_api.cpp（setDecoratorSeed 保留，generate 跳过）。
- **发现 0（载具差异，重大）**：block_probe -features 全程路径 vs vanilla = **22653 mismatch（98.56%）**，
  残差族与 mod cppReplace 路径（13328 / 99.1526%，stone→dirt/sand 族）**完全不同**：
  主导对为 tuff→deepslate 1496、andesite→stone 1270、granite→stone 1175、air→deepslate 1171、stone→coal_ore 974、
  tall_seagrass→water 864 —— **ore/矿石族 desync 主导**（Java 有 granite/andesite/红石矿 blob 处 C++ 缺/错位）。
  → **block_probe FULL 路径 ≠ mod 路径的 feature 阶段行为**；两载具残差不可互相引用（§9.7 可比性）。
- **发现 1（消融不干净）**：skip 后 mismatch 22653→30940，新增 8447 cell 含 air→polished_granite 3677
  → ore_granite 落点被 skip 改变。机制：ore_dirt/gravel 先写 dirt/gravel，后续 ore 的 target 谓词
  （base_stone_overworld）读到已换块 → 跳过前序 feature 会级联改变后续 feature 落点。
  setDecoratorSeed 为完整 setSeed（chunkrandom.h:151-154），RNG 隔离成立；耦合在**方块状态经 target 谓词传递**。
  → **skip 归零法对本课题无效**（特征间经谓词耦合），需改用「加法消融」（只开单一 feature）或定点写者追溯。

## 对三候选的更新判定
- b3（伪影）：排除（不变，mod 路径数据对账自洽）。
- b2（surface/disk 主导）：主导排除（不变）；windswept /8.25 阈值分歧仍为独立待查项。
- b1（feature blob 放置差）：**部分成立但非全貌**——mod 路径的 stone→dirt/sand 族与 block_probe 路径的 ore 族
  desync 是两个不同载具上的不同残差族；b1 的 ore blob 机制解释 block_probe 族更贴，mod 族出界 dirt 仍未定位。

## 新分叉（下一步需裁决）
1. mod cppReplace 路径与 block_probe -features 路径在 feature 阶段的集成差异（哪条是生产语义？mod 路径 feature 由谁执行？）
2. mod 路径出界 dirt（y>200 / y<-32，~1660 cell）的写入者定位（定点 setBlock 日志回溯）

## 载具/工具纪要（本轮坑）
- block_probe argv[3] = 已存在的 vanilla 参照（fopen 失败不 throw 但目录错会 throw）；wgDir 必须含 data/minecraft/（=versions\1.20.1\data\worldgen）；FULL 需 `-features`；导出用 `-save`（**无 biome 段格式**，3145888B，与 Java 导出 3243362B 不同——cmp 脚本按长度 ≥3243362 判格式）。
- wgDir 传错（...\data 而非 ...\data\worldgen）→ 未捕获 C++ 异常 0xE06D7363 / 退出码 -1073740791，crash handler 只给栈无消息——目录错误信号。
- cmd-output 目录不存在时 `2>` 重定向失败 → probe 未跑即 OpenError（先 New-Item）。
