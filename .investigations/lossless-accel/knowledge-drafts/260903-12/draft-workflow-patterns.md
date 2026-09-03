# 草稿：knowledge/discovered/workflow-patterns.md 追加（subagent 产出，主会话应用）

> **应用位置**：`knowledge/discovered/workflow-patterns.md`——「## 发现 #25」末尾追加两条「补充案例」小节（追加不覆盖）。写后同步 INDEX.md「工作流模式」行说明（无需新行）。
> 现有编号核对：workflow-patterns.md 当前最大编号 **#25**；本次均为 #25 / #21 的补充案例，**不新增编号**。
> 来源 session：260903-12（实际 2026-09-03 晚，锚 git 367de35 20:43 前后）；证据：`.artifacts/lossless-accel/est-shared-verdict-260903-12.md` + `.investigations/lossless-accel/review-est-shared-260903-12.md` + `.investigations/lossless-accel/cmd-output/est-price-p24-260903-12.txt` / `estopt-mt-baseline-260903-12.txt`。

---

### 补充案例（260903-12，#25 第三例：静态地图「恰好一致」断言失真——量化算术必须显式做）

- **现象**：scout 地图（est-shared-java-map/java-est-chain.md）#7 断言「Rust heights4 传 +15，量化后恰与 Java (i+1) 角一致」——据此本应触发的 +16 角修正被推迟了一整轮；实际 `+15 >> 2 << 2 = +12 ≠ +16 = +16 >> 2 << 2`，被 P1 运行时逐值对比（est-compare-p13b：敏感 chunk (201,200) java@+16=56 vs shared@+12=48）抓出。judge 复算另发现该断言与地图自身摘录内部自相矛盾（同段先写「60 量化→60 量化不变」，随即又称 +15 量化后一致）。
- **根因（机制）**：静态对拍清单里的「恰好一致」结论靠**直觉合并**产出——两侧参数（+15 vs +16）肉眼「差不多」，量化函数（`(x>>2)<<2`）的等价类边界（15 与 16 分属 +12/+16 两类）没有做显式算术就判了等价。这是 #25 案例①（引用行号语义错位）/②（常量记忆错）之外的第三种失真形态：**函数等价性靠直觉而非计算**。
- **定位**：P1 Java mixin RETURN dump vs Rust WG_EST_DUMP 逐值对比——运行时抓出 +12≠+16；judge 复算（16>>2<<2=16、15>>2<<2=12）一击确认。
- **修复**：verdict 登记「角参数 +15→+16 修正」待办（两臂共有，独立小包）；scout 地图不回改（失真记录保留，§15.4 精神）。
- **教训**：**静态对拍清单里的任何「恰好一致/恰好相同」断言 MUST 做显式算术**（把量化/取整/映射函数套进去逐值算），不能靠直觉合并「差不多的参数」；「一致」断言与自身摘录冲突是失真的廉价检测信号。同族：#25 案例①②、#17（打印坐标≠采样坐标）——同属「静态调研产出消费前先验证」，本条补「函数等价性维度」。

---

### 补充案例（260903-12，#21 第二次量化实锤：hot/cold 形态差 ~95× + 单价稳定性作 fan-out 免触发判据）

- **现象**：同一段 est 扫描代码（est_price_probe，同 seed 同树），hot 模式（单列重复采样）65/57 ns/iter vs cold 生产形态（顺序新列）5751/5721 ns/iter——**形态差 ~88-100×（judge 复算，取 ~95×）**。同时，跨 session 生产隐含 est 单价稳定：260903-11 `48ms/(7342−1715 iter)≈8.5µs/iter`；本 session `55.6ms/5627 iter≈9.9µs/iter`。
- **如何利用（两条）**：
  1. **「微测外推生产无效」的量化实锤**：#21 主案例差 40×、260903-11 补充案例（working set）差 ~5×、本例同代码 hot/cold 形态差 ~95×——三次独立测得同一结论，量级 40-100×。判据强化：任何微测数字引用到生产前，先问采样形态（遍历方向/列切换/冷热/生命周期）与 working set 是否同构；**「微测 × N ≈ 生产实测」不自洽时永远怀疑形态失配而非实现异常**（与 AGENTS「阶段耗时×批次≈wall」自洽判据同源）。
  2. **跨 session 单价稳定性可作收敛判定依据（fan-out 免触发）**：剩余差归因时，若「生产隐含单价跨 session 稳定」（8.5 vs 9.9µs，±16%），则主机制成本主导且无漂移，次级效应候选（b2 类）不构成与主机制并存的互斥候选 → core.fanout 触发条件（≥2 互斥候选）不满足，**免 fan-out**。本例 judge 复核认可（review-p2-p3-final §C：「作为收敛判定依据使用而非独立证实，符合触发条件」）。注意边界：该判据只免 fan-out，不构成对主机制的独立证实——结论仍需生产实测支撑。
- **证据**：`.investigations/lossless-accel/cmd-output/est-price-p24-260903-12.txt`（hot 65/57ns、cold 5751/5721ns 原始输出）；单价核算见 `.artifacts/lossless-accel/est-l2-defaultflip-p2-260903-12.md` P2.4。
