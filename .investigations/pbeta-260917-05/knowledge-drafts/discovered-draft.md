```yaml
id: pbeta-260917-05:discovered-draft
block: 260917-05
status: draft          # 草稿供主会话应用；编号/定稿归主会话
worker: subagent (core.worker)
价值门判定: 见 §0
```

# discovered 草稿（260917-05）

## 0. 价值门判定与合并决策

- 高价值（必记）：①「汇总交叉结论 MUST 回原始日志抽样验证」——本轮实证主会话机械交叉 4/5 项数字正确、1 项定性（46=ABI 切换）被 10 行日志抽样一票推翻，判错方法可复用；②「计数恒等 ≠ 集合恒等」探针口径盲区 + hash 探针须随行打印口径；③ #42 家族第三犯补充案例行。
- 建议合并：①②合并为 workflow-patterns **一条**新发现（同一根因面 = 汇总层数字对、定性错；判据同族），③为 #42 的补充案例行（一行，进 build-tooling #42 条目）。原任务设想的「探针口径不可比伪差」发现需改写：**「跨口径 hash 差集先核口径同质性」判据本身保留有效（#17/#59 家族），但本轮实例证明其逆命题同样致命——「给异常 diff 预先安一个口径解释」而不回日志核验，就是本轮 46 的错误定性来源**。合并后一条覆盖两面。

## 1. 新发现草稿（workflow-patterns，编号由主会话定，暂标 #16x）

### 发现 #16x（最高价值·错误优先）: 汇总交叉定性 MUST 回原始日志抽样——「口径解释」未经验证就是新伪差

- **发现时间/发现者**：260917-05 / core.worker subagent（pbeta-260917-05 判读）。
- **来源定位**：.investigations/pbeta-260917-05/knowledge-drafts/verdict-draft-260917-05.md §1。
- **现象**：主会话机械交叉输出「inputDiff23=46/46 为 abi=packed↔blocks9 路径切换（abi 字符串两 run 不同）」；而原始日志 pbeta05c.log 中 `abi=blocks9` 全文 **0 行**，抽样 10 个 diff chunk 三 run 均 abi=packed、hash 两两不同、emptySec 恒等——所谓「ABI 切换」在本轮从未发生。其余交叉数字（changed23=0、emptySec 全局 71871=71871、inputDiff12=529、∩changed12=123）经独立核对全部正确。
- **根因**：汇总层数字（差集大小、交集率）来自脚本复算可信，但**定性句**（「差异由 X 解释」）是叙事不是测量——「46 的全部成员 abi 字符串两 run 不同」这句若真跑过逐行比对会立即为假，说明定性在生成时未经逐行核验，且预先存在的口径解释（判据里刚写过「两口径不可比」）被当成了现成答案套在异常数据上。
- **定位**：verdict worker 按 §16.3 交接验证纪律对日志 grep `abi=blocks9`（零命中）+ 10 个 diff chunk × 3 boot 抽样比对——10 行日志一票否决。
- **教训/判据（可复用）**：
  1. 汇总交叉产物 = 数字 + 定性；**数字可复算继承，定性 MUST 抽样回原始日志/数据一手验证后才可进 verdict**（判读 worker 独立抽样是廉价一票否决，10 行即够）。
  2. 给异常 diff 安解释前先证解释的**存在前提**（本轮「ABI 切换」前提 = 日志里存在两种 abi 字符串——一个 grep 即证伪）。
  3. 「跨口径 hash 差集先核口径同质性」（#17/#59 家族）判据保留，补对偶面：**同口径 ≠ 无伪差**——本轮 46 即同口径（packed↔packed）真不稳定，最后定性为机制 open 而非套用任何现成解释。
  4. 计数型探针盲区：**计数恒等 ≠ 集合恒等**（emptySec 逐 chunk 相同不能排除「同数不同节」的空节集合差）；探针设计时计数只能作必要条件证据，且 hash 探针输出应随行打印口径要素（abi 名/长度），让日志自证口径。

## 2. #42 家族补充案例行（build-tooling #42，一行追加）

- **#42 补充案例·第三犯**（260917-05 pbeta05b）：采集驱动漏 tmpdir 固化 → JNA 落系统临时目录拒访 → lightInit threw → **整臂静默 vanilla 形态**（光照全 vanilla，非局部降级，最难看穿形态）；SELFCERT 硬门（lightInit ok 计数）正面拦截整轮 VOID。教训强化：#42「tmpdir 固化」不是可选项——凡 JNA/native 采集驱动，固化缺失的失败形态是**整臂变形而非报错退出**，SELFCERT 行为化自证是唯一可靠闸门。来源：.investigations/pbeta-260917-05/cmd-output/pbeta05b.log + pbeta05b_result.json。

## 3. 自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门先判（§0），低价值一次性数值（46/529 具体清单等）不进 discovered，只留 .investigations/
- [x] 错误优先：根因为机制层（定性未经逐行核验 + 预设解释套用），非现象复述
- [x] 定位含可复用诊断方法（grep 存在前提 + 10 行抽样）
- [x] 被推翻的定性（46=ABI 切换）保留标注不删除
- [x] 数字与原始日志/result json 一致（worker 独立核对过）
- [x] status: draft，不改任何 status；编号 #16x 留主会话定
