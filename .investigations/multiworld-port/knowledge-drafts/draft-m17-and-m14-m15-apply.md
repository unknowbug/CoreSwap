# 草稿：M17 台账条目 + M14/M15 草稿应用核对 + compiler-idioms 新发现 —— 260901-02 session

> 状态：draft（knowledge 落盘 subagent 产出，待主会话应用 + 验证签核）。
> 依据：`.investigations/multiworld-port/m17-bedrock-band-summary.md`（M17 修复结论摘要，candidate 待 judge + 用户确认）。
> 自检：已过 `knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md` §四清单（五段式完整/根因机制层/定位含工具/速查表同步/数字来自主会话实测记录）。
> 应用方式：A 部分 → `multiworld-errors.md` M16 之后、速查表之前插入正文 + 速查表末尾加行；B 部分 → 无需应用（核对结论，见节内说明）；C 部分 → `knowledge/discovered/compiler-idioms.md` 末尾追加发现 #7 + INDEX.md 对应行说明列更新。
> **本文件不改任何正式文件。**

---

## A 部分：错误台账 M17 条目（载体：`.investigations/multiworld-port/multiworld-errors.md`）

（应用说明：正文插入在 M16 条目之后、`## 附：错误 → 根因 速查表` 之前；速查表行加在表末尾。）

---

## M17. nether bedrock roof 随机带 4011 块残差：`below_top` 锚换算 off-by-one（`min_y+height-v` → `min_y+height-1-v`）

### 现象
- M13 修复 `vertical_gradient` 解析缺失后，nether 块级对齐达 96.0568%，但残留**结构性微差**：bedrock roof 随机带（y=123..126）4011 块残差（M13 后混淆对遗留：roof 主层已对、随机层微差）。
- 诊断 bin（`WorldgenRust/src/bin/nether_bedrock_band.rs`，4×4@0,0 seed -8248）per-y vanilla/rust bedrock 计数对比：vanilla 概率序列 [123]=0.2、[124]=0.4、[125]=0.6、[126]=0.8、[127]=1.0（满层）；Rust **同一模式整体 +1 层**（[123]=0、…、[127]=0.8）。

### 根因（机制）
- `parse_anchor_abs_y` 的 `below_top` 锚换算 off-by-one：
  - 旧：`min_y + height - v`（nether：128-v → v=5 时 true_y=123/false_y=128）
  - 新：`min_y + height - 1 - v`（Java 顶块 y = min_y+height-1 = 127 起 → true_y=122/false_y=127）
- Java 的 `VerticalGradient` 锚 `above_bottom(0)` / `below_top(0)` 语义：顶块坐标是 **min_y+height-1**（闭区间端点），Rust 侧换算漏了 `-1` → 整条 vertical_gradient 判定的 y 基准整体平移一层，随机带概率层全部错位。
- **overworld 为什么没暴露**：overworld deepslate 梯度用**绝对锚**（absolute y），不走 `below_top` 换算路径——全绝对锚的维度掩盖了这个换算 bug，直到第一个依赖 `below_top` 的维度（nether roof）才暴露。

### 定位（诊断方法）
1. **随机带概率序列逐层对比**（诊断 bin `nether_bedrock_band.rs`）：per-y 统计两侧 bedrock 出现概率——vanilla 0.2/0.4/0.6/0.8/1.0 vs Rust +1 层平移，**确定性平移签名**直接指向锚 y 基准错位（若是随机流/种子错，概率序列形状不会保持一致平移）。
2. 修复后复跑同 bin：每层 **van_only=rust_only=0 逐位吻合** → splitter 种子派生正确，纯锚换算 bug，一维一因闭环。

### 修复
- `WorldgenRust/src/surface_rules.rs` L944：`parse_anchor_abs_y` 的 `below_top` 分支 `min_y + height - v` → `min_y + height - 1 - v`。
- 效果（同工具同区域同 seed，前后可比）：
  - 随机带逐位吻合（van_only=rust_only=0）；
  - 全量回归 `multiworld_nether_blocks` TOTAL **96.0568% → 96.4428%**；y96..127 **94.0% → 97.12%**。
- 状态：cargo release 编译通过；未实机验证（建议用户实机验收时一并观察下界顶部床岩）。

### 教训（可复用判错经验）
- **「随机带概率序列逐层对比」是定位锚错位的利器**：per-y 概率序列是 vertical_gradient 的指纹——序列形状一致但整体平移 = 确定性平移签名，一眼锁定锚 y 基准；与随机流错误（形状破坏）可明确区分。
- **数据驱动 JSON 规则的锚换算必须用「非绝对锚的维度」实测验证**：overworld 全绝对锚使 `below_top` 路径从未被执行过——单维度验证通过的换算代码 ≠ 换算正确，只是未被覆盖；新维度接入时对每个锚类型（above_bottom/below_top/absolute）至少一条实测用例（与 M3「参数化须贯穿全链路」同族：覆盖缺口 = 隐性 bug 存量）。
- **闭区间端点 off-by-one 家族又一条**：`min_y+height` 是开区间端，顶块是 `min_y+height-1`——凡「从顶/底数第 N 层」换算，先把端点语义（inclusive/exclusive）与 Java 源码核对再写公式。

---

## A-2：速查表新增 1 行（加在 `## 附：错误 → 根因 速查表` 表末尾，M16 行之后）

| nether bedrock roof 随机带 4011 块残差：per-y 概率序列 vanilla 0.2/0.4/0.6/0.8/1.0 vs Rust 整体 +1 层平移（M17） | `parse_anchor_abs_y` 的 `below_top` 锚换算 off-by-one：`min_y+height-v` 应为 `min_y+height-1-v`（Java 顶块 = min_y+height-1，闭区间端点）；overworld 全绝对锚使该换算路径从未被覆盖，nether 首个 `below_top` 使用方暴露 bug；修后逐位吻合（van_only=rust_only=0），TOTAL 96.0568%→96.4428%、y96..127 94.0→97.12% | **随机带概率序列逐层对比 = 锚错位的确定性平移签名**（形状一致整体平移 → 锚 y 基准错；形状破坏 → 随机流错）；**锚换算要用非绝对锚的维度实测验证**（单维度全绝对锚通过 ≠ 换算正确）；闭区间端点家族：从顶/底数第 N 层换算先核对 inclusive/exclusive 语义 |

---

## B 部分：M14/M15 草稿应用核对（结论：无待应用内容，本节仅记录核对过程）

1. **指定草稿不存在**：`knowledge-drafts/draft-multiworld-errors-m13-16.md` 未找到；`knowledge-drafts/` 下唯一相近文件为 `draft-m16-id-domain-mismatch.md`。
2. **该文件不含 M14/M15 部分**：draft-m16-id-domain-mismatch.md 只含 M16（条目 + 速查表行 + compiler-idioms 发现 #6 + workflow-patterns #8 更新）——M16 部分已应用到现行台账（M16 五段式、M14 末尾结案标注、速查表 M16 行均在位），按任务要求**过滤跳过，不重复写**。
3. **M14/M15 已在现行台账成型**：`multiworld-errors.md` 已含完整 M14 条目（含 2026-09-01 结案标注「真根因由 M16 解释，根因方向作废、方法结论保留」）与 M15 条目（CountingAlloc 全局原子计数序列化）及对应速查表行——**无遗留的 M14/M15 草稿待整理/追加**。
4. **处置结论**：B 部分为纯核对记录，主会话**无需对正式文件做任何 M14/M15 相关改动**；若后续发现其它目录存在含 M14/M15 段的旧草稿，以其内容与现行台账比对后按「追加不覆盖」处理（本次未发现）。

---

## C 部分：compiler-idioms.md 新发现草稿（发现 #7）

（应用说明：追加到 `knowledge/discovered/compiler-idioms.md` 末尾；INDEX.md「语言/编译器惯用法」行说明列末尾追加「、锚坐标换算 off-by-one below_top/above_bottom（260901-02）」。）

## 发现 #7: 锚坐标换算 off-by-one（below_top 类顶块相对锚 = min_y+height-1-v）——数据驱动规则锚公式的维度覆盖判据

**发现时间:** 260901-02
**发现者:** worker（multiworld-port M17）
**来源定位:** `.investigations/multiworld-port/multiworld-errors.md` M17 + `.investigations/multiworld-port/m17-bedrock-band-summary.md`（修复位置 `WorldgenRust/src/surface_rules.rs` L944）
**置信度:** candidate（修复后逐位吻合：van_only=rust_only=0；TOTAL 96.0568%→96.4428%，同工具同区域同 seed 前后可比；confirmed 待用户拍板）
**module:** re-code / swe（数据驱动规则解析 / 坐标域换算）

### 观察
MC worldgen 的相对锚（`above_bottom(N)` / `below_top(N)`）换算：**顶块 y = min_y+height-1（闭区间端点），不是 min_y+height**——`below_top(v)` 正确公式为 `min_y + height - 1 - v`。写漏 `-1` 会使整条 vertical_gradient 判定的 y 基准整体平移一层，随机带概率层全部错位。此类 bug 会被「全绝对锚的维度」长期掩盖（overworld deepslate 用 absolute 锚，`below_top` 路径从未被执行），直到第一个依赖相对锚的维度（nether bedrock roof）才暴露。

### 证据
- vanilla nether bedrock roof 概率序列（4×4@0,0 seed -8248，诊断 bin `nether_bedrock_band.rs` per-y 计数）：[123]=0.2、[124]=0.4、[125]=0.6、[126]=0.8、[127]=1.0；Rust 修复前同形状**整体 +1 层**（[123]=0…[127]=0.8）——确定性平移签名。
- 修复（`min_y+height-v` → `min_y+height-1-v`）后逐位吻合（每层 van_only=rust_only=0）→ splitter 种子派生正确，纯锚换算 bug；全量回归 TOTAL 96.0568%→96.4428%、y96..127 94.0→97.12%。

### 如何利用
- **公式**：`above_bottom(v) = min_y + v`；`below_top(v) = min_y + height - 1 - v`——凡「从顶/底数第 N 层」的换算，先把端点语义（inclusive/exclusive）与 Java 源码核对再写（与 M3「锚 height 用逻辑生成高度不混用 world_height」同族：锚换算两个独立坑 = 高度基准 + 端点 off-by-one）。
- **签名**：per-y 概率/计数序列形状一致但整体平移 = 锚 y 基准错位；形状破坏才是随机流/种子错——诊断 bin 按层统计即可单轮定位。
- **覆盖判据**：数据驱动 JSON 规则的每个锚类型（absolute/above_bottom/below_top）至少要有一条**非绝对锚维度**的实测用例——单维度（全绝对锚）验证通过 ≠ 换算正确，只是未被覆盖。

---
