# T1 一手核：size=1 批 blocks25 窗口组成（idk-K2b）——260920-03

## 结论（candidate 建议，confirmed 留用户）

**idk-K2b 闭合（Java + Rust 双面一手源核读，零采集）**；同时**证伪** judge N2 载荷假设的后半句：

1. **恒定零填充 = 证实**：size=1 批 blocks25 的 5×5 槽中，仅 (0..2)×(0..2) 9 槽被中心 blocks9 覆盖，其余 16 槽恒 0，且**零槽不进 size=1 中心的输出段**。
2. **「窗口 = 中心 chunk」= 证伪**：窗口 = **中心 + 8 邻共 3×3 chunk 的 blocks9 快照**，非仅中心 chunk 16×16×384。

## 一手证据（file:line）

Java 侧 `versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java`：
- :786 `int[] blocks25 = new int[WG_BLOCKS25_LEN]` —— 零初始化（恒定零填充的机械来源）。
- :789-803 拼装循环：`dst = ((kz+dz9)*5 + (kx+dx9)) * 98304`；k=0（size=1）→ 仅 5×5 的 (0..2,0..2) 9 槽被该中心 b9 覆盖。
- :805-806 注释 + 内核对侧实证（下条）：未覆盖边缘槽不进任何中心输出。
- :816-817 `e3aIn` 仅 blocks25 路（LIGHT_PACKED=off 臂）赋值 —— 260920-01/02 的 blk25 hash 数据确系本路产物。
- 快照时点：b9 在**提交线程 light() 入口**采集（:579 `wgLightCollectBlocks` → `LightDomainBatch.java:119` `st.blocks9s.put(centerPos, blocks9Snapshot)`）。

Rust 侧 `worldgen-core/src/light/mod.rs`：
- :400-405 `light_compute_domain` → :433 impl；:446-468 全域 fill 一趟共享（25 chunk）。
- :474-519 逐中心窗准备：`for k in 0..9`，k=0 → `x0=z0=0`，opacity/col_max **子拷仅 3×3 chunk 域**（:491-504），种子回填按 `x∈[x0,x0+DOM)` 过滤（:513）——**零槽（列/行 3-4）结构性地不进入 k=0 窗**，BFS/export 窗内封闭。
- 载体身份链（judge C1 补）：Java `wg.CppWorldgen.lightComputeDomain`（Mixin:810）→ `versions/1.20.1/rust/src/jni_bridge.rs:439` `Java_wg_CppWorldgen_lightComputeDomain` → :472 `light::light_compute_domain`——Rust 侧读数即 Java 路径实际内核，链条闭合。
- 佐证：:866-900 `light_compute_domain_bitwise_equivalence` 等价门（G3：fill/种子均本帧快照确定函数）。

## 对既有结论的修正（§15.4 取代面，原文不改）

- **k2-worker.md :34/:70 条件推论收窄**：「size=1 批 9/9 全异 ⇒ 差异压到中心 chunk 自身 16×16×384 内」**不成立**——窗口含 8 邻快照，差异可位于 3×3 邻域**任一 chunk** 的内容（C1a）/快照时点（C1b，采集时点 = 各 chunk 提交时刻，天然非同时）。
- **保留成立面**：「批分组/同批拼装顺序」排除仍有效（size=1 批无同批共享、零填充不进输出）；C2 零填充轮廓差排除结论不受影响（其排除的是填充轮廓差，本核确认填充恒定）。
- C1a vs C1b 仍 hash 层不可分离；分离需逐元素 dump 新采集（另行立项），但**判读域从「中心 chunk」扩为「3×3 邻域快照」**。

## idk 台账动作

- **idk-K2b → 闭合**（双面一手源）。
- 新 idk 不开：C1a/C1b 分离本就在「需新采集」清单（k2-worker §5 建议），非新缺口。

## 降级声明

- 本核为**静态一手源核读**（Degraded 层：无运行时 trace）；但被核对象为装配代码路径（非数值敏感函数），且零填充「不进输出」有 Rust 等价门测试佐证，声明降级足够支撑候选级。
