# C1 CONCERN 判据措辞修正草稿（knowledge 草稿，status 不改，待主会话应用）

> 产出：core.worker 知识库角色（subagent），2026-09-08。
> C1 原文（`.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md` L16/L45；任务给的 `.investigations/nether-save-full/review-judge-20260908.md` 不存在，实际 judge 文档在 .artifacts 下，与 NEXT_SESSION L21 引用一致）：
> 「CONCERN-C1（判据措辞）：设计文档判据『≥3 采样区间一致』——实测三轮是同一 seed 同一 region（198..205）重复三次，验证的是确定性/可复现性，不是 3 个独立采样区间……建议改判据措辞为『同 region 三次复跑一致』或补跑 2 个不同 region。」

**修改理由（引 C1）**：「≥3 采样区间」判据实测为同一 region（seed B，nether 4×4@3200,3208，region 198..205）3 次复跑，验证的是**确定性/可复现性**而非**区域覆盖面**——措辞与实测不符，按 judge C1 统一回写为「同 region 3 次复跑确定性 + ore per-id 消融值佐证（quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = SKIP_FEATURES 消融值，ref 邻域 1992/728/1533）」；历史对齐数字保留 seed+region+口径三要素，只改判据措辞不动其他内容。

---

## 文件 1：`E:\PYTHON\CoreSwap\NEXT_SESSION.md`

**对 1.1**（L41，纪律条——「按 C1 修正」的待办语义改为已定稿措辞）：

old:
```
- 历史对齐数字引用必须带 seed+region+口径三要素（沿袭 X1）；「≥3 采样区间」措辞按 C1 修正。
```
new:
```
- 历史对齐数字引用必须带 seed+region+口径三要素（沿袭 X1）；回归判据措辞统一为「同 region 3 次复跑确定性 + ore per-id 消融值佐证」（C1 已回写，禁用「≥3 采样区间」——同 region 复跑验证的是确定性，非覆盖面）。
```

（L21 CONCERN 在案记录、L35 工作清单 todo 项不动——status/待办状态归主会话管理。）

## 文件 2：`E:\PYTHON\CoreSwap\versions\1.20.1\docs\09-multi-dimension.md`

**对 2.1**（L448，回归验证表「修复后」行——3 轮 run 的定性改为同 region 复跑）：

old:
```
| 修复后（3 轮全新 run） | **全部 94.4241%**（990108/1048576） | 与消融轮 SKIP_FEATURES 值逐位一致 = 双跑通道闭合 |
```
new:
```
| 修复后（同 region 3 次复跑，seed B 4×4@3200,3208，存档口径） | **全部 94.4241%**（990108/1048576） | 同 region 复跑零散布 = 确定性/可复现（非多 region 覆盖面）；与消融轮 SKIP_FEATURES 值逐位一致 = 双跑通道闭合 |
```

**对 2.2**（L451，判据行——核心修正点）：

old:
```
- 判据：修复后值 94.4241% 与此前手工 `WG_SKIP_FEATURES=1` 消融值**逐位相同**，且 3 轮全新 run 无散布——比「区间不重叠」判据更强（重复了消融实验的因果链）。
```
new:
```
- 判据（C1 措辞修正）：**同 region 3 次复跑确定性**（seed B，nether 4×4@3200,3208 同一 region，3 次全新 run 全部 94.4241% 零散布——验证的是确定性/可复现性，非多 region 覆盖面）+ **ore per-id 消融值佐证**（quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = SKIP_FEATURES 消融值，ref 邻域 1992/728/1533）——修复后值与手工 `WG_SKIP_FEATURES=1` 消融值逐位相同（重复了消融实验的因果链）。
```

## 文件 3：`E:\PYTHON\CoreSwap\versions\1.20.1\docs\10-timewise-archive.md`

**对 3.1**（L2402，时间线 2026-09-08 条回归行——同步措辞；追加不覆盖原则不受影响，此为同日条内措辞修正）：

old:
```
- 回归：3 轮全新 run 全部 **94.4241%**（990108/1048576，seed B，nether 4×4@3200,3208，FULL 参照，ReadWorldProbe 存档口径；修复前 93.8988%）；ore per-id quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = **SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533）——与消融实验因果链重复，比区间判据更强。
```
new:
```
- 回归（C1 措辞修正）：同 region（seed B，nether 4×4@3200,3208）3 次复跑全部 **94.4241%**（990108/1048576，FULL 参照，ReadWorldProbe 存档口径；修复前 93.8988%）——验证确定性非覆盖面；ore per-id quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = **SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533）——与消融实验因果链重复。
```

---

## 明确不改的项（边界声明）

- `.investigations/nether-save-full/design-wg-set-flags-20260908.md` L32（「≥3 采样区间一致」原文出处）：**不改**——.investigations 为过程性历史记录，C1 的修正对象是正式 docs/交接措辞；原判据措辞保留即保留「为什么错」的证据链（错误优先原则）。
- `.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md`：judge 意见原文，永不改写。
- NEXT_SESSION L21（CONCERN 在案）/ L35（工作清单 todo）：状态类内容归主会话应用时处置。
- docs/09 表中「修复前（消融轮）」「ore per-id 直接佐证」两行：数字与判读已符合 C1 要求口径，不动。

## 自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：本条为判错/措辞修正（高价值——判据措辞错误会误判覆盖面），载体正确（正式 docs + 交接纪律条）
- [x] 只改判据措辞，数字与三要素（seed B / region 4×4@3200,3208 / 存档口径）全部保留
- [x] 数字来自 NEXT_SESSION.md L17-18 与 judge 文档实测记录，无编造
- [x] old→new 为逐字精确文本对，主会话可直接 edit 应用
