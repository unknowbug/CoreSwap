# B6-1 延伸发现：union 口径掩盖单版本缺口（260918-01 第二轮）

```yaml
status: draft
block: 260918-01
role: 主会话（延伸核查）
trigger: 收尾后核查遗留项 #3（1.21.6 侧是否同病）
layer: Degraded（静态抽取；未实跑 1.21.6 boot）
```

## 1. 触发与结论

**触发**：B6-1 交付后核查「1.21.6 侧是否同病」（原遗留项 #3）。

**结论（两步递进）**：
1. **1.21.6 同病，且更严重**——per-version 口径下 **1.21.6 有 21 个缺口**，1.20.1 有 11 个（其中 1.20.1 的 11 个已定性为合法旁路 + 1 UNRESOLVED）。
2. **本块自己的门禁存在口径缺陷**：原实现对两版取 **union** 后算 ORPHAN，导致 **1.20.1 已声明而 1.21.6 未声明的开关被静默掩盖**——1.21.6 的 21 个缺口里，有 **10 个**正是本块刚在 1.20.1 修好的同名开关（被 union 掩盖）。**已修**（见 §3）。

## 2. 1.21.6 缺口实测（per-version，含共享 carrier）

| 版本 | 声明面 | 本版+共享消费面 | 缺口 |
|---|---|---|---|
| 1.20.1 | 128 | 136 | **11** |
| 1.21.6 | 106 | 124 | **21** |

**1.21.6 缺口清单（21 项）**：
```
bench.threads / bench.worldgen / blobProbe.chunkX / blobProbe.chunkZ / blobProbe.dim /
blobProbe.size / chunkRandom.seed / chunkRandom.seed288 / colDump.targets / colprof.x /
colprof.z / coreswap.bulkwblog / coreswap.bulkwbsentinel / coreswap.bulkwbtest /
coreswap.stallwatch / coreswap.wbcheck / coreswap.wbcontent / height.x / height.z /
java.io.tmpdir / surfacedump.dim
```

**其中 10 项 = 本块已在 1.20.1 修复的同名缺陷**（`blobProbe.{chunkX,chunkZ,size,dim}` / `colprof.{x,z}` / `surfacedump.dim` / `coreswap.{bulkwblog,wbcontent,bulkwbtest,bulkwbsentinel}` 中的大部分）；另 **`coreswap.stallwatch` 是 1.21.6 独有缺口**（1.20.1 已声明）。

⇒ 这说明缺陷**不是 1.20.1 的偶发**，而是**两版共有的系统性映射维护债**；且 1.21.6 落后更多（声明面 106 vs 128）。

## 3. 门禁口径修复（本块自己产物的缺陷，错误优先留痕）

**缺陷**：`scripts/check_switch_mapping.py` 原实现把两版 `build.gradle` 的 `-D` 名取**并集**（M1），消费面也取并集（M2），ORPHAN = M2∖M1 —— **单版本缺口在并集中消失**。

- **实例**：`blobProbe.chunkX` 在 1.20.1 已声明（本块修复）⇒ 并集 M1 含它 ⇒ ORPHAN 不含它 ⇒ **1.21.6 缺它这件事看不见**。
- **家族**：#105（载体偏差——「载体全绿对真实故障零判别力」）+ #163（计数门 vs 内容门）同族：**聚合口径掩盖分项缺陷**。

**修复**：脚本新增 **per-version 缺口段**（按版本分别算 `consumed_ver ∪ consumed_shared ∖ declared_ver`），并：
- 输出「`-- per-version 缺口（union 口径掩盖的单版本缺陷）--`」段；
- `--strict` 语义升级 = **DEAD 非空 或 任一版本有缺口** 即非零退出；
- 打印提示「union 口径的 ORPHAN 会掩盖单版本缺口——请以 per-version 段为准」。

## 4. 未处置（诚实边界）

1. **1.21.6 的 21 个缺口未修**——本块范围为「建立机制 + 修 1.20.1 实证缺陷」；1.21.6 补映射**属独立工作**（需先按族内对称性判据逐项定性，且 1.21.6 是另一条产物线的构建面）。
2. **未实跑 1.21.6 boot 验证**（本核查为静态抽取，Degraded）；1.21.6 的发射面未用哨兵验过。
3. **1.21.6 侧的「合法旁路」定性未做**——21 项中应有一部分（`bench.*` / `height.*` / `chunkRandom.seed` / `java.io.tmpdir`）与 1.20.1 同属旁路，**不得直接按 21 个缺陷计**；须走同款 T4 定性流程。
4. `coreswap.stallwatch`（1.21.6 独有缺口）：1.20.1 有映射（`:165`）而 1.21.6 无 —— 需核 1.21.6 是否有该消费点，若有则属**新发现的单版本缺陷**。

## 5. 判据沉淀（可复用）

- **判据（MUST）**：凡「跨多个载体/版本聚合的对账门」，MUST **同时给出 per-carrier 视角**——聚合视角（union/intersection）会掩盖单载体缺口，且掩盖方向与「哪个载体先被修好」耦合（先修的载体**制造**对后修载体的掩盖）。
- **判据（MUST）**：`--strict` 类门禁的**判据域**须与「用户可踩到的路径」对齐——用户可能只跑其中一版，故任单版缺口都应是失败。
- **反模式**：「并集口径 + 报告绿」= 与 #163 silently-green 门同构的**聚合形态**。
