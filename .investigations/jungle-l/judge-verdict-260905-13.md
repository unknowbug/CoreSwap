# judge 裁定归档（260905-13 jungle_l 工作块）+ CONDITIONS 核销

> 审查者：core.judge subagent（96c47f6d）；本文件 = 主会话归档（过程性记录）。

## 裁定：APPROVE-WITH-CONDITIONS

- J1 修复逐行核对通过（y 语义 i-3+l/2、循环结构、push 位置 ↔ MegaJungleTrunkPlacer.java:45-46 一致）；消费条件逐点一致（canReplace 门两侧等效，无流相位系统性差）。
- .b2 C5 降级 / .b3 双重排除 / J5 反向预测均判据自洽。
- J3 排除可推荐 candidate（用户拍板 confirmed）；其余保持 draft。

## CONDITIONS 核销

- **C1（未核销，转下轮）**：vine +1972→−16388 重排解读保持 draft，待 ①单棵 mega RNG 流 trace ②TREESET mega 枝干 log 清点 ③.b4 上游差定界。**修复代码本身保留**（语义正确 + judge 逐行核对），不回滚。
- **C2（本轮核销）**：region 复测 §9.7 口径声明——载体 = idk7_region_dump 纯实现 bin（region_j1fix_full.bin，2193 chunks，x[-2,40] z[-26,24] miny=-64 h=384，seed 8576294172403134396）；覆盖面 = 全 2193 chunk 树族 8 id 计数（sv≥400 过滤后同样本）；可比性 = 与 v25（260905-10 bee+CA 基线）同脚本族同 vanilla 参照（ab-vanilla-region）同 id 级口径，**可比**；与 features_probe 6×6 存档口径**不可比**。
- **C3（本轮核销）**：mega 放置成功性差（Rust 1 棵成功 vs Java THJ 27/28 两棵全失败）已登记入 J5 定界清单（b3-shortcircuit-trace.md §五 J5 项 + 本文记录），候选 = get_top_position/域守卫④⑥差。
