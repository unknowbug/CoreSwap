# G2 fan-out 候选 .b3：缺键/序列化语义差（H3）

- 状态：draft（置信度 candidate 倾向，验证分层 = **Degraded + 存档数据验证**：ChunkSerializer 一手源不在 .tmp/light-yarn/，序列化省略规则部分依赖 ChunkNibbleArray 语义推理 + 上游给定「缺 SkyLight 键 = 15」口径，未读 1.20.1 ChunkSerializer.java 原文）
- worker：G2 残差根因 fan-out .b3；脚本：`.tmp/light-g1/b3_key_contingency.py`、`.tmp/light-g1/b3_sky_hist.py`（parser 复用 g2_residual_map.py，缺键 fill 口径：block=0 / sky=15）

## 1. 机制分析（源码层）

### 1.1 ChunkNibbleArray 均质构造语义（一手源 .tmp/light-yarn/ChunkNibbleArray.java）

- `new ChunkNibbleArray(int defaultValue)`（L31）：**bytes = null**，仅记 defaultValue；`get()` 走 defaultValue（L64-66）。
- `isUninitialized()` = bytes==null && defaultValue==0（L172）；`isUninitialized(expected)`（L168）。
- 即 mixin 里 `wgLightNibble` 的三条路径（ServerLightingProviderMixin.java L207-213）：
  - flag=1 → `new ChunkNibbleArray(0)` = 隐式全 0（bytes==null）
  - flag=2 → `new ChunkNibbleArray(15)` = 隐式全 15（bytes==null）
  - flag=0 → 显式 2048B（bytes!=null）

### 1.2 MCA 序列化省略规则（推断，标注 ⚠️）

WorldChunk.java（一手源）**不含序列化代码**（grep 仅 4 处 light 相关，均非 serialize）；1.20.1 的存档序列化在 ChunkSerializer（yarn），本工作区无一手源。基于 ChunkNibbleArray 语义推理（⚠️ 推断）：

- 隐式数组（bytes==null）序列化时**必然走「跳过」分支**（否则须先 asByteArray() 物化，vanilla 无理由为全 0/全 15 付 2KB）：block 通道隐式全 0 跳过；sky 通道隐式全 0 与隐式全 15 均跳过（与上游给定「缺 SkyLight 键 = 15」读回口径自洽）。
- **显式数组（bytes!=null）一律写 2048B 键**，即使值全 0 或全 15。

### 1.3 H3 的不对称点在哪

Java 写回路径本身**自洽**：flag=1/2 产隐式数组 → 序列化跳过 → 读回 = fill，与 vanilla 隐式语义一致。**真正的不对称在 rust 侧 flag 判定 vs vanilla 运行时数组物化状态**：

- vanilla 光照引擎传播过的 section，其 bytes 被 asByteArray/set 物化 → 存档**显式写键**（哪怕全 0）。
- rust 若对「计算结果全 0 的 sky section」给 flag=1（隐式 0）→ 存档**跳过** → 重读 = **15**。运行时若两侧本为 0，存档对比却呈现 rust=15 vs vanilla=0 的整 section 翻转——**纯序列化语义漂移，非计算差异**。
- rust 若对全 15 section 给 flag=0 显式数据 → vanilla 本会省略而 rust 写 2048B 全 FF → 读回仍 15，无害但造成「rust 侧键多余」（数据已证实，见 §2）。

## 2. 数据验证（448 残差 chunk / 1138 残差 section 列联表）

键存在性 × 通道 × 值方向（dir = vanilla值 − rust侧fill后值，取首个差异 nibble）：

| 通道 | 键形态 | section 数 | 说明 |
|---|---|---|---|
| sky | **VPRM**（van 有键 / **rust 缺键**） | **360**（其中 276 个 van−rust=−15 整 section，ndiff=4096） | vanilla 有真实局部数据（vanMix），rust 隐式 15 |
| sky | VPRP（两侧均有键） | ~454 | 两侧显式、值真不同（±1..±15 混合，集中 x=22..37, z=−3..−24 群） |
| sky | VMRP | 1 | 边缘个例 |
| block | VPRP（两侧均有键） | ~118 | 显式混合值差异 |
| block | VPRM / VMRP | ~250 | 小差异（|d| 多 ≤4），vanilla 缺键=0 与 rust 显式 0 语义对称 |

**主导类 sky VPRM-360 的 vanilla 值分布**（b3_sky_hist.py）：**0 占 70.44%**，≤2 占 78.77%，长尾到 13；全部 ndiff=4096（vanilla 每个 nibble 均 <15）。这与「rust 运行时也算出 ≈0（flag=1），存档重读被漂移成 15」的 H3 机制高度相容——但**也无法排除 rust 真算成 15**（存档不可区分 flag=1 与 flag=2，@anchor.idk：需内存态探针才能切分）。

**键存在画像**（3377 chunks）：vanilla sky 键只出现在 Y≤7（地表附近，Y≥8 为 0/3377 = 全 15 隐式省略）；rust sky 键在 Y=8..19 恒为 931/3377 —— rust 对 vanilla 会省略的全 15 section 写了**显式**键（值均 15，fill 后无残差，无害但证实两侧物化策略不对称）。

## 3. 裁决

**倾向：UNCERTAIN**（拆分：作为「448 chunks 的唯一独立根因」= DENY；作为「主导 0↔15 整 section 翻转签名（sky VPRM-360，≥1.47M nibble 残差）的成因/放大器」= CONFIRM 候选）。

一行理由：残差中 ~810 个 section（sky VPRP ~454 + block ~368）两侧键均在且值真不同（显式混合差异，缺键语义无从作用），H3 无法独立解释；但 sky VPRM-360（vanilla 真实局部数据 vs rust 缺键=15，vanilla 值 70% 为 0）与 H3 序列化漂移机制完全相容且量级主导，需内存态探针切分「rust 运行时 =0 被存档漂移成 15」还是「rust 真算成 15」。

## 4. 关键证据清单

1. 列联表（b3_key_contingency.py）：1138 残差 section 中 sky VPRM（rust 缺键）360、其余 ~778 两侧键均存在——H3 只能触及前者的呈现形态。
2. sky VPRM-360 的 vanilla 值直方图（b3_sky_hist.py）：0 = 70.44%，全 section ndiff=4096——「两侧运行时≈一致(0)，rust 存档被隐式-0→缺键→15 漂移」相容；但存档不可区分 flag=1/2。
3. 源码：wgLightNibble flag=1/2 产 bytes==null 隐式数组（ChunkNibbleArray.java L31/L168-174），序列化按语义必被省略（推断 ⚠️，ChunkSerializer 无一手源）——Java 写回路径自洽，不对称源在 rust flag 判定 vs vanilla 数组物化状态。
