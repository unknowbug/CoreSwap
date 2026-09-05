---
编号: 000
任务: feature parity 独立课题——树木/藤蔓/矿石放置分歧定位与修复（worldgen feature 阶段 Rust↔Java 对齐）
任务类型: 机制定位 + 代码复刻修复（re-code 勘探 + swe 修复）
模式档位: 重量
状态: 已批准（260905-05 用户批准 + 两边界拍板：①tree 族解禁本课题实作；②矿石只修规则层，归因落 populationSeed 派生层即停手报用户，不碰挂起项④）
日期标签: 260905-05（实际 2026-09-05 12:01，Get-Date 实测）
依据: feature-parity-charter-260905-04.md（立项卡）+ NEXT_SESSION 260905-04
---

## 1. 全局视图

- **目标**：4×4@200 seed 8576294172403134396 域内 feature 放置 palette 级逐位一致；根因定位 + 修复 + 验证。
- **已知签名**（G2 遗产）：346/447 chunk 有 blocks 差；jungle/oak leaves、vine、jungle_log 增减（80-350 处/chunk，集中 Y=3..4 树冠段）+ stone↔coal_ore、granite↔andesite 替换层。
- **明确不做**：光照内核（G2 已证无缺陷）、D4 接管协议、永久挂起 4 项（ore_vein 域 1 / aquifer 域 2 / C++ ore 族 desync / populationSeed——⚠️ 注意与本课题「矿石替换层」的边界：本课题只对齐 1.20.1 vanilla ore feature 的 Rust 实现，不碰 C++ ore desync 挂起项，不重开 populationSeed）。

## 2. 廉价独立验证前置（交接结论验证纪律，先于一切分析）

G2 遗产结论（「翻转集中 Y=3..4 树冠段」「rust=vanilla−1~2 显式差剖面」「签名含 trees+ore 两族」）不得当公理直接续推，开工第一步 MUST 复核：
- V1：复跑/抽验 447 对比脚本（.tmp/light-g1/），确认签名分布仍复现（≥1 个 chunk 差剖面逐块核对，非只看统计）；
- V2：核对参照与 Rust 两侧 seed/坐标三查（铁律）；
- V3：「101/447 影子传播残差」候选保持推断级标注，不继承为事实。

## 3. 角色分配 & 任务拆解（依赖图）

立项卡分层判据：**先矿石/替换层（确定性更强）后树木（形状+随机派生）**。

- **Phase 0**（本文件）→ 用户批准
- **Phase 1 验证遗产**（主会话）：V1-V3 廉价验证，产出 `.investigations/feature-parity/verify-handoff-260905-05.md`
- **Phase 2 recode-scout 勘探**（subagent，MUST——机制未明禁跳单点）：
  - 勘探 1：feature 放置管线地图（NOISE→SURFACE→FEATURE 顺序、Rust 侧 feature 调度实现位置、ore placement rule 数据源 vs 代码硬编码点——对齐 data-driven-boundary.md 已标注的「feature tag 展开无数据源」升级点）
  - 勘探 2：随机派生链摸底（feature 放置的 random source：chunk-level decorator seed 派生、Java setDecoratorFeatureUniform 等价物在 Rust 的实现现状）
  - 产物：`.investigations/feature-parity/`（只读，管线地图）
- **Phase 3 分层定位**（fan-out 预备，见 §5）：
  - 3a 矿石/替换层分支：ore feature 配置（JSON 数据）+ 放置随机派生 + stone↔ore 替换规则对拍
  - 3b 树木分支：树 feature（oak/jungle）形状生成 + 藤蔓逻辑 + 随机派生对拍
- **Phase 4 修复**（swe 收敛闭环，主会话）：按定位结论改 Rust（worldgen-core），单假设线性推进
- **Phase 5 验证**（Anchorlaw）：
  - golden 逐位不变对照法（#47）适配：以 vanilla 为 golden，palette 级对比
  - 判据：4×4@200 域内 feature 差异 = 0（palette 级）；101/447 残差随柱级验证收口或显式转出
- **Phase 6 judge + 归档**

## 4. 并行执行计划

- 第一波：Phase 1（主会话）与 Phase 2 两个勘探 subagent 可并行（勘探只读不依赖 V1 结果）
- 第二波：Phase 3a/3b（若分叉成立，fan-out 并行）
- 第三波：Phase 4/5 串行

## 5. fan-out 预置

- **节点 Phase 3**：候选机制互斥分叉（预计）：a) 矿石/替换层差异（数据/派生层）vs b) 树木形状/随机派生差异——**两分支 MUST 并行 fan-out**（各派 worker 产 .bN 候选），禁止主会话逐个自推
- 触发即执行：若 Phase 2 勘探显示分叉结构不同，按实际分叉重新预置并报用户

## 6. 人工决策 HOOK 点

- H1：架构批准（本文件）✅ 260905-05 批准
- H2：Phase 3 分叉点——若 fan-out 候选互斥结构确认，报用户知悉（并行策略）✅ 两边界拍板（tree 解禁 + 矿石规则层边界）
- H3：修复方案批准（若改动涉及数据驱动边界扩张，如 feature tag 数据源落地）
- H4：confirmed 授予（judge 通过后用户拍板）

## 7. judge 步骤预置

- 节点：Phase 3 定位结论（根因定论）| MUST | 审查对象：.artifacts 快照 + git diff + 验证记录三源
- 节点：Phase 5 修复交付 | MUST | 三源核对（.artifacts + HEAD/worktree diff + golden/palette 对比记录）
- 节点：各分支 candidate 授予 | SHOULD

## 8. 知识库更新

- 结论性 docs（versions/1.20.1/docs/ 新主题篇或 06 篇追加 + 10 时间线）：**subagent 产出草稿**（core.worker，prompt 含 SUBAGENT-KNOWLEDGE-GUIDE.md 前置读取行）+ 主会话应用验证
- 通用模式 → knowledge/discovered/（同上流程）
- 过程/中间产物 → .investigations/feature-parity/（主会话可写）

## 9. 子角色介入点（全部预置）

- **scout**：Phase 2 两个勘探 subagent（recode.scout 语义，只读，产物 .investigations/ 管线地图）
- **worker**：Phase 3 分支分析/解读（fan-out 时各分支一 worker）；知识库 docs 草稿产出（Phase 6）
- **fan-out**：Phase 3 分叉 MUST（§5）
- **judge**：§7 三节点
- **knowledge**：Phase 6 结论性落盘 MUST subagent

## 10. 风险 & 回退

- R1 分叉结构与预想不符 → 回 Phase 2 补勘探（不硬套 a/b）
- R2 涉及随机派生链深层（decorator seed）→ 若撞 populationSeed 挂起项边界，立即停手报用户裁决边界
- R3 域内差异数量大（346/447）→ 分层切分保证先收确定性分支；验证域可先缩到单 chunk 再扩 4×4
- R4 .tmp/light-g1/ 脚本为临时区产物，可能不完整 → Phase 1 验证时按需重建对比脚本（palette 对比逻辑参考 judge 复算脚本）
- R5（260905-05 12:53 追加，用户拍板）：顺带小项 opacity:-1 clamp 修复（worldgen-core/src/light/mod.rs，light_data.json 解析 18 条目，#14/-8 姊妹案例）——独立小闭环，不动 light 内核其他部分，不并入 Phase 3 分叉
