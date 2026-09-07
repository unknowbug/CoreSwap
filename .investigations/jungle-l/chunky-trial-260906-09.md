# Chunky 双臂地形对比试跑（260906-09 深夜，试验性 §7.2 载体升级）

> **状态升级（260906-09 用户「授权确认」拍板）**：① Chunky 双臂法**转正为区域级地形验证标准载体**（confirmed；单点 sanity 保留 tp+F3）；② 效率实测结论 confirmed（见文末效率节）。③ vanilla 臂即 Chunky 生成（同实例 `-PcppVanilla=1` 对称设计，已向用户确认说明）。
>
> 试验性归因部分（三候选 fan-out）维持 draft。
> 载体：`gradle runServer` + Chunky 1.3.146（Fabric，run\mods），seed 8576294172403134396，两臂同实例切换参数。
> 采集：`.tmp/jungle-l-260906/chunky/`（chunky-coreswap.log / chunky-vanilla.log / region-{coreswap,vanilla}/ 各 6 mca + chunky_diff_260906-09.py）

## 管线核验（#36）

- **CoreSwap 臂 PASS**：CppBridge init seed 一致、stageMask=3、dll size=2160640（=1.0.26 出货）——Chunky 生成走 mod 管线确认。
- 两臂各 1089 chunks（33×33，center 464,-248 radius 256），Chunky 完成行 Task finished 100%（32s / 8s）。
- chunk 解析：3025/3025 全部 common（region+slot 定位，不信 NBT 内坐标，#20）。

## 结果（每 section 每 palette 名计数差，全 seed 区域）

- **有差异 chunks：946/3025**；分类差值：**terrain=113,454 / veg=86,400 / air=51,260**
- 植被类（预期内，特征流非确定）：oak_leaves 27.6k、jungle_leaves 24.7k、vine 16.9k、jungle_log 5.8k、oak_log 2.2k…（两臂 Chunky 生成均多线程 → 特征执行序同样非确定）
- **地形类差值按桶分三组互斥候选（未归因，禁止单通道结论）**：
  1. **地下 blob 石类**：andesite 16.3k + diorite 15.3k + granite 14.4k ≈ 46k——已知地形微差残差域的量级（≈15 块/chunk），对应 #67 前历次 terrain 残差课题；
  2. **洞穴/水系类**：air 51.3k + cave_air + water 3.5k + lava——carver 级联（Java carver 挖 Rust 地形 vs 挖 Java 地形的级联差）或 aquifer 差；
  3. **结构类**：mineshaft 组件（chest/spawner/rail/cobweb/oak_planks/fence/wall_torch，±整件级）+ amethyst/calcite（geode）——结构阶段在两臂间有差异的候选（ CoreSwap mixin 拦 NOISE/SURFACE 对结构阶段的影响未核）；
  4. 矿石类 ±1-3/section（coal 2.5k、deepslate 系列）：疑为 1/2 的下游（基底 stone/deepslate 组成差 → 矿物替换计数差），非独立候选。
- 用户实机受控观察（liveobs-260906-09）：483/500 焦点柱两臂**地表一致**——与上表共存：差异主要在**地下/散点**，焦点柱表层未受影响。

## 初步信号（draft，待归因）

- **「Rust 地形 = Vanilla 地形」在逐块意义上不成立**（terrain 桶 113k 非零），但焦点柱表层一致 + 量级属已知残差域 → 与其说是「新发现的地形差通道」，不如说是**已知地形残差域的首次全区域量化**。
- 对 #68 的影响中性偏通道①：483 类树分歧仍以执行序为主归因（live vs pregen 互翻直证）；地形残差是否足以翻转个别树的放置谓词，需定点核（拿本次 region 直接查 483/500 柱全列即可，零成本）。
- mineshaft/结构差候选**价值最高**：若 CoreSwap 臂结构阶段真有系统差（mixin 影响面），这是 F3 wiring 图未覆盖的第 5 阶段——需独立核（查 chunk Structures NBT + mixin 拦截点清单）。

## 下一步（fan-out 预置：候选 1/2/3 互斥，应并行 worker）

- a. 483/500 柱全列直读（region 已在盘，零成本）
- b. worker A：blob 石类残差归因（对齐已知残差清单）
- c. worker B：结构阶段差异核（mixin 拦截点 × Structures NBT）
- d. air/water 桶：carver 级联 vs aquifer，视 b/c 结论再定

## Vanilla 臂载体决议（260906-09 用户拍板：纯效率视角）

- **维持 loom Java 环境（`-PcppVanilla=1`）**：与 CoreSwap 臂同实例、换臂即换参数、无额外搭建/人肉成本——纯效率层面最优，不再讨论。
- 「loom vs 纯 vanilla 地形级交叉标定」（用户 -Vanilla 客户端小区域 vs region-vanilla diff）**降为一次性备查项**：不排程、不反复做；哪天顺手做则一次闭合，未做不影响本载体日常使用。

## 效率实测（confirmed，260906-09 用户拍板；同轮日志时间戳实测）

| 项 | 旧法（runServer+forceload，f3r1/f3r2 实测） | Chunky 法（本轮实测） |
|---|---|---|
| 单臂总 wall | ~27 分钟（22:26→22:53） | ~4 分钟（启动 ~90s + 生成 32s + 收尾） |
| 覆盖 chunks | 25（5×5） | 1089（33×33） |
| 单价 | — | ~0.2 秒/chunk（千 chunk 边际 32s） |
| 等待 | sleep 猜时长 | Task finished 100% 确定性完成行 |
| 人力 | tp+F3 逐点截图 | 零人力，程序化 diff |
| 采集物 | 日志 | region 全量落盘，任意柱/块复查零成本 |

结论：覆盖面/时间比 >40×，人力观察环节消除，采集物可复用；固定启动成本两法相同，单点疑问仍以 tp 法最快。**转正为区域级地形验证标准载体**；使用前置 = CppBridge 管线核验（stageMask/dll 大小/seed）+ 双臂同实例对称（vanilla 臂 `-PcppVanilla=1`）。
