# 时间线条目草稿 — 拟追加到 versions/1.20.1/docs/10-timewise-archive.md 末尾

> 草稿（knowledge worker 产出，待主会话应用）。格式对齐 10 篇既有条目（日期标题 + ✅/❌/🔍/⚠️ 状态标注 + 末尾「记录指引」）。
> 目标载体：`docs/10-timewise-archive.md`（时间线归口铁律：过程只进 10）。

---

## 260901-03 nether 存档写入口径 Full 化（1.0.22 dll，双 seed，candidate）

> 承接 09 篇 nether 维度课题 + `.investigations/nether-save-full/`。目标：存档级（MCA 直解）Full 口径量化 Rust nether 接管质量。dll sha256=C5AC5309F3C59A044（1.0.22 M17），区域 4×4 @(3200,3208)。

### ❌ 一、首轮 run 三场全部无效（enabled=false 未察觉）——已作废
- ReadWorldProbe 新增 nether 支持（dim 属性 + 动态 min_y/height + `_nether` 参照后缀）后跑 seed A：gen1 内存 131 差 / gen2 内存 1 差 / gen2 存档 104 差 / reconfirm 读盘 1 差——5 条「矛盾观察」跨运行不一致。
- fan-out 三候选（b1 时序 / b2 管线 / b3 非确定）分析后，b2 日志取证倒查发现铁证：**三场 run CppBridge 全部 `enabled=false`、`[Mixin] intercepted` 0 条——dll 从未加载，全部 vanilla-vs-vanilla**。原 5 条观察**已作废，被 v2 Rust run 取代**（§15.4 取代记录；facts 文件正文不删不改，待主会话回填顶部 supersedes 标注——judge #20）。
- ⚠️ b2 论据更正一笔（judge #16）：其子候选①声称「seed 从 ref 文件内读天然防错位」**与代码不符**——seed 实来自 `-D` 属性拼文件名，header 读后丢弃不校验（fail-fast 建议优先级升高）。
- 根因/教训五段式见错误台账 `nether-save-errors.md` E1-E5（cppWorldgenDir 传错一层 / header 断言凭印象 / 未查接管标志 / 论据未指认代码行 / 矛盾先查前提）。

### ✅ 二、ctypes 直连定位 cppWorldgenDir 错层（数据层证据）
- ctypes 直连 `wg_create` 单变量复现：传错层（把 CppBridge 注释里的解压布局 `…/worldgen/data/…` 当 wg_dir）返回 0，传对层（含 `data/` 的层 = `versions/1.20.1/data/worldgen`）返回非 0——机制根因坐实，b2 早期「dll 提取失败/临时目录权限」推测降为表象。

### ✅ 三、v2 真 Rust 双 seed 三口径数据（judge 全 PASS，建议 candidate）
- seed A = -2032795982907864146：内存 = 存档读回 **99.9376%**（精确同值 1047922/1048576）；MCA 直解 **99.9278%**（1047819，差 103 = cave_air 簇，精确对账）。
- seed B = 8576294172403134396：三口径精确同值 **93.5156%**（980582/1048576）。
- 口径声明（§9.7 三要素）：载体 = MCA 存档直解 + ReadWorldProbe 内存读 vs vanilla 参照（WGB2）；覆盖面 = 4×4 chunk 全高度（nether min_y=0 height=256）；**与 docs/09 的 96.44% 探针口径不可比**（载体不同）。
- Rust 真实参与证明：两 seed v2 log 均 `enabled=true` + 64 条 intercepted（目标 4×4 + feature 蔓延邻域）。

### 🔍 四、残差分类（数据直读 PASS，机制解释保持 draft）
- seed A（757 块）：矿石 feature 差 84.5% > air↔cave_air 尾随簇 13.7% > magma 1.1% > 熔岩湖边界 0.7%。
- seed B（67,994 块）：basalt deltas / 表面规则三大宗石互换 76.6%（全部落在 y≤127 噪声高度内，y≥128=100%）> soul sand valley 8.4% > 矿石 3.9% > magma 2.5% > 熔岩湖 2.0%。
- 机制归属全部 candidate 以下（residual-interpretation §4 诚实声明 Partial/Degraded）。

### 🔍 五、未闭合待查项
- 103 cave_air 簇机制（v2 下内存=读回精确同值但 MCA 多 103——新形态矛盾，b1/b3 均未闭合，judge #14 确认保持 draft 正确）。
- basalt deltas 大宗互换（B1 surface rule 条件链）、nether 矿石 features 缺口（未实现 vs 错位）——深挖优先级见 residual-interpretation §3。

### 📌 记录指引
- 错误台账 → `.investigations/nether-save-full/nether-save-errors.md`（E1-E5 五段式 + 速查表）。
- 结论 → 09 篇（或 06/07 篇，主会话定）追加小节，草稿 `knowledge-drafts/docs-appendix-nether-save.md`。
- 过程 → 本节；judge 意见 → `.investigations/nether-save-full/judge-review.md`（#16/#17/#20 修正项待主会话落实：b2 论据更正、A2 引用作废数据改写、facts 文件回填 supersedes 标注）。
- 状态：数据与口径声明 candidate（judge 建议），confirmed 留人类；机制解释 draft。
