# review-001：1.21.6 移植清单定稿审查（core.judge，260908-08）

- 审查对象：`port-list-260908-08.md`（Phase 2 收敛定稿）
- 证据链：w1-w4 语义 diff（均 draft/Degraded）+ `diff-structural-260908-08.txt` + `000-架构设计/架构计划-260908-08-mc-1216-port.md` + `knowledge/discovered/workflow-patterns.md`（#36/#53/#67/#13/#20 等）+ `worldgen-core/data-driven-boundary.md`
- 角色声明：只出意见，不改 status；confirmed 留人类。

## 总意见：**PASS-with-conditions**

清单总体忠实于 w1-w4：A 区工作项覆盖了 w2 的两个移植项（P5 邻居依赖表 + FULL 显式转换步）与 SPAWN barrier 运行时验证（P6，正对 w2「本结论 Degraded 未做行为探针」的遗留）、w4 的 nearMountainBiomes 跟进（C）与两个 idk（D）、w1 的 Blender @anchor.idk（P8）。B 区四项不移植的等价性论证多数站得住（temperature seaLevel 的 80=63+17 算术 w3/w4 双源核对；DimensionPadding「不实现即对齐 1.20.1」与 w4 #9 一致；carver/surface 零变化有逐字节 diff 底册支撑）。但存在 6 个须清偿的条件项（下述 C1-C6），其中 C1/C3/C5 直接命中 #53/#37「未验证预测当公理」家族与 #26 假阴性盲区。

## 通过项（抽样核对记录）

| 项 | 核对结果 |
|---|---|
| P5/P6 ↔ w2 跟进项 | 覆盖完整：w2 建议的 (a) 邻居依赖表、(b) SPAWN barrier、(c) FULL 显式转换步全部入 A 区 |
| P8 ↔ w1 裁决 3 | 一致（float hypot 不可观测级，idk 声明恰当） |
| B-Aquifer（地形形状部分） | w1 6.2 实证：第 4 邻居只喂 needsFluidTick，density 配对集合不变——「地形形状不变」论证成立 |
| B-temperature seaLevel | w3 §1.3 算术核对通过（overworld 逐位等价），「多维度化时再做」裁决合理 |
| B-DimensionPadding（对 1.20.1 目标） | w4 #9：padding=10 仅 TrialChambers、其余 NONE——「不实现即对齐 1.20.1」成立（但见 C1 的 e2e 口径矛盾） |
| D 遗留 idk | 与 w4 worker @anchor.idk 一一对应 |
| 命名/日期 | 260908-08 标签与任务目录一致，无漂移 |

## 条件项（逐条编号，清偿后才建议定稿 candidate）

### C1（MUST）P7 e2e 验收口径与 B 区「明确不移植」存在未声明的矛盾
清单总裁决是「维持 1.20.1 行为」，但 P7 的 Chunky 双臂是「Rust 接管 vs **1.21.6 vanilla**」。凡 B 区明确不移植的 1.21+ 新行为（DimensionPadding@TrialChambers、Shipwreck Y 重定位、HugeMushroom/Disk/EndSpike/Geode 4 项 feature 算法差、SPAWN barrier 差异），Rust 臂与 1.21.6 vanilla 臂**必然**出现差异。若 P7 验收不预先声明「预期差异白名单」，这些差异会被误读为移植 bug 触发返工（#13/#26 家族反向形态：已知系统差未声明，污染判读）。**要求**：P7 判据附「预期差异豁免清单」（逐项引 B 区条目），差异分解先剔豁免集再判回归。

### C2（MUST）B-Aquifer 不移植的前提是关于 Rust 侧的未验证断言
w1-w4 只 diff 了 Java。B 项写「tick 队列不复刻」——这是**对 Rust 现状的主张**，w1 只给了条件句（「若 Rust 复刻了 needsFluidTick 需移植」），无任何 worker 核对过 Rust 引擎是否输出流体 tick / needsFluidTick 面数据。同理 B-BlendingData（max_section+1「写存档才需对齐」）也依赖「Rust 不写 blending 存档字段」这一未核对前提（#36 执行体核对家族：对 Java 的裁决不能自动迁移到 Rust）。**要求**：P1 开工前各做一轮廉价核对（grep Rust 侧 aquifer/needsFluidTick/tick 写出面 + blending 序列化面），结果补进清单 B 区作为前提声明。

### C3（MUST）P3「预期仅键表差异」是未验证预测当公理——#53 家族命中
「w1 证实 density 数学零变」≠「1.21.6 density_function/noise JSON 仅键表差异」：w1 明确把 noise 参数数值归「数据侧另案核对」，w4 记录了 ORE_DIAMEDIUM_MEDIUM 等数据级新增，且 w1-w4 均未 diff 过 **1.21.6 的 worldgen JSON 数据本体**（新 DF 类型、codec 字段演化都无证据排除）。把「预期仅键表差异」写进定稿清单即把预测当公理（#53/#37）。**要求**：① P3 验收 = transpiler 输出产物树 vs 1.21.6 JSON 的 parse 产物对拍（#12：对拍解析产物，非输入原文），不得以「跑通即毕」收口；② 预先定义「不支持的 DF 新类型」处理策略 = fail-fast 报错（#56：禁静默 fallback/落 catch-all），并把 1.21.6 DF 类型集 ⊆ transpiler 支持集作为启动期断言目标（#26/#27 判据）。

### C4（MUST）Degraded→运行时闭环映射缺失
w1-w4 全部为 Degraded，但清单未声明哪些「零变化/逐位等价」静态结论由哪个 Phase 4 载体闭环。**要求**：清单补一张映射表——(a) 由 P7 e2e 端到端兜底的（density/noise/surface/carver 数学零变化）；(b) MUST 加专项运行时探针的：aquifer 地形形状抽验（w1 裁决 5 自身建议）、Blender blending 路径（若 1.21.6 验证含老世界混合则必测，否则显式声明不启用）、temperature/冰冻海洋、SPAWN barrier（P6 已列）；(c) 显式留 @anchor.idk 的（P8 之外补 aquifer needsFluidTick——它当前是「不移植」决定而非 idk，与 C2 合并清偿）。缺此表，「零必修项」的总裁决在 Phase 4 无判定闭合路径。

### C5（MUST）风险回退缺失 + 一处代码改动被「数据级吸收」掩盖
清单无任何 R/回退节（架构计划 R1-R4 未被操作化）。已知兼容缺口至少列四项：
1. **1.21.6 biome JSON carvers 单列化**（w4 #5：Carver 枚举删除）——Rust `load_carvers` 现按 1.20.1 格式解析，这**必须改 Rust 解析代码**，但 C 节把它归入「随 P2 进数据集」，代码面被掩盖（#53 同型：把代码改动记成数据改动，直接威胁 T1 口径，见 C6）；
2. SpawnEntry Pool 化 / MapCodec 演化对 Rust feature/carver JSON 解析器的兼容性未评估（#12 假阴性盲区：JSON 原文看着兼容 ≠ 解析器读对了）；
3. biome_params.json 需新增 PALE_GARDEN 行、blocks.json 新块（pale oak 系/leaf_litter/dry grass）——P2 隐含覆盖但未列核对项；
4. extract_tags.py 对 1.21.6 tag 目录结构的适配未验证（scout 1c 结论当公理风险）。
回退条款照抄架构 R4（1.20.1 薄壳冻结 + 确定性 dump 哨兵，#76 同代码双臂法可复用）写入清单。

### C6（SHOULD）T1 口径不可操作且有系统性偏差
现口径把 P3 生成物计为「代码侧」、P4（Java 探针工程）计为「数据侧」：机器生成行数会污染「引擎代码改动」度量，Java 探针工程根本不是「数据」。**要求**：改三列分账——① worldgen-core 手写引擎改动行数（`generated/`、`bin-diag/` 除外，理想 ≈0 但 carvers 解析器改动应如实计入）；② 数据 JSON 行数；③ 探针工程行数（单列不进比值）。同时「理想『换版本零引擎改动』已被 w1-w4 证实为可达」措辞过强：w1-w4 证实的是 **Java 侧差异面**小，Rust 侧解析器改动（C5-1）尚未计入——降为「预计接近可达，待 P3/P5 实测」。T2 可用，但须把 scout-1c 理想清单的具体文件钉死为引用对象，悬空引用/历史路径残骸按架构计划单独出账。

### C7（SHOULD，备忘）FittestPositionFinder /locate 公式差未入清单
w4 #2 记录 1.21.6 改了 `/locate biome` 适应度公式（不进热路径）。若 P4 探针迁移或 P7 验证流程用到 `/locate`，需声明该口径差或避用；不用则维持现状，仅备忘防踩。

## 裁决建议

清单本体结构合格、与证据链追踪性良好，**不建议直接进入 Phase 3 实现**；C1-C5 清偿后（预计一轮文档修订 + 两轮廉价核对）可升 candidate 定稿开工。C2/C3 的核对动作均为 ≤一轮的 grep/静态对拍成本，符合交接结论验证纪律。

- 推荐状态：移植清单保持 **draft**（定稿候选），条件清偿后建议 candidate。
- 审查人：core.judge（subagent，隔离）；本意见不含任何 status 修改。
