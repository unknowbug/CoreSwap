# 草稿：versions/1.20.1/docs/10-timewise-archive.md 追加「260903-12」节（subagent 产出，主会话应用）

> **应用位置**：`versions/1.20.1/docs/10-timewise-archive.md` 文件末尾（当前 2701 行，「260903-11」节之后）追加本节。追加不覆盖。
> 日期锚：git 提交簇 260903-12（367de35@2026-09-03 20:43 之后至本 session 提交），实际 2026-09-03 晚。

---

## 260903-12（est shared 臂 Java 逐位裁决 + L2 翻默认三件套 + gpu-merge 重议——四臂课题收口轮）

> 承接 260903-11「未闭合」三项（shared 裁决 / 翻默认前置 / e2e 观察）；产物 `.artifacts/lossless-accel/{est-shared-verdict,est-l2-defaultflip-p2,gpu-merge-revisit}-260903-12.md`（均 candidate）；过程 `.investigations/lossless-accel/` + `cmd-output/*260903-12*`。

### ✅ P0 交接验证：四臂 hash 复现（§15.3 廉价独立验证先行）
复跑四臂 A/B（estopt-ab-arms-p0）：off `74f5dfc4` / shared `8bff4087` / l2==off + 命中 84.9%，与 260903-11 逐项一致 → 交接结论可继承。环境四查过（删 run\world、seed 8576294172403134396 三处一致、WG_* 默认关、dump 门控不影响 hash）。
**为什么**：260903-11 的 shared hash 变化只是「待裁决假设」，不验证不续推（M14/M11 纪律）。

### 🔍 scout：Java est 链勘探（subagent 隔离）
勘探产物 `.investigations/lossless-accel/est-shared-java-map/java-est-chain.md`（Java est 调用面 + mixin RETURN dump 路线）。
**为什么**：裁决 Java 语义需先摸清 est 调用链与可靠 dump 位置（#2：机制未明先勘探）。

### ✅ Java est dump 探针搭建 + 三方对比裁决（P1，Full 层）
Java 侧 EstDumpProbeMixin（RETURN dump）+ Rust WG_EST_DUMP 同 seed 同 region 对比：共同列（c0 原点角，64 chunk）**shared 64/64 与 Java 逐值一致；off 0/64 全偏（judge 复算 delta 恒 −1）**；角列敏感性 63/64（唯一敏感 chunk (201,200)：java@+16=56 / shared@+12=48 / off=55）。**裁决：shared=修正既有 est 错位，off=系统性偏离**（supersedes 260903-11「未裁决假设」）。
**为什么**：翻默认前置 = shared 语义必须 Java 逐位背书；共同列等值证明 + 敏感性探测双口径闭合（judge A2 精确化覆盖面表述）。

### ✅ 翻默认前置三件套（P2，Full 层）
① Mutex 争用基线（estopt_mt_bench T=1/2/4/8 交错双跑）：L2 加速比 2.55→3.12× 随线程不降反升，无争用退化（双跑偏差 <3%，judge B 修正「<2%」表述）；② 大 region sweep（64×64）：命中稳定 92±1%、evictions=0，触顶投影 ~7600+ chunk（judge B 复算修正 inserts ~40k / ~17 条每 chunk），typical region 远未触顶；③ e2e l2 stats 落盘（judge C1 清偿）。
**为什么**：260903-11 judge 预置的三项翻默认门控逐项清偿，量化声明全部可溯源。

### ✅ P2.4 剩余差归因：微测外推生产无效（#21 量化实锤）
est_price_probe 同代码 hot 60ns/iter vs cold 5.7µs/iter（形态差 ~95×）；跨 session 生产隐含单价 8.5/9.9µs 稳定 → 次级效应候选不构成互斥候选，fan-out 免触发（judge 认可作为收敛判定依据）。
**为什么**：e2e 收益与微测上界的 ~1.5× 剩余差要给 Partial 解释并声明，不能留缺口（§9.7）。

### ✅ gpu-batch-merge 重议（P3.1，决策建议）
est L2 落地后新基线：Rust l2 单线程 27.69 ms/chunk vs Java FULL ~33（**跨 bench 近似比较**，judge D 标注；保守口径 T=1 35.8 同量级）→ 立项目标（追平 Java）已被无损路径消除，8 线程 ~4.5 ≈ 7× Java（量级判断）。**建议降级/搁置 gpu-batch-merge**，待用户拍板。
**为什么**：GPU 路径 dispatch/readback 成本（369ms，260903-08）在小批量下是负收益；目标消失则工作包失去存在依据。

### ✅ 两轮 judge 审查（review-est-shared / review-p2-p3-final）
① shared 裁决：PASS + 3 CONCERN（A1 off 臂 −1 扫描偏移线索 / A2 覆盖面措辞 / C1 dump 缺 seed 头），复算零偏差，同意上报 confirmed；② P2/P3：无 BLOCK，judge B 数值修正（inserts/触顶投影）+ judge D 跨 harness 可比性标注，修正后可推荐 candidate→confirmed。
**为什么**：confirmed 前 MUST judge（AGENTS 强制触发点）；judge 复算产出 A1 新机制线索（见下新课题）。

### 🔍 新课题登记（本 session 新增，均另立验证，不阻塞上述 candidate）
1. **off 臂 −1 扫描偏移 bug（judge A1，生产 bug 线索）**：off 臂 `(min_y..min_y+noise_height).rev().step_by(8)` 半开区间 rev 首采样点 = 319，Java 从 320 起扫（319,311,… vs 320,312,…）——同时解释「c0 也偏离」与「delta 恒 −1」的规整性；**off 是当前默认臂，生产影响独立于翻默认决策**。
2. **surface_rules.rs:505 大 region panic**：`fill_chunk_blocks` 在 64×64 sweep 至 ~2304-2560 chunk 处 panic `missing noise sampler`（estopt-sweep 尾部原文在案）——疑似预加载噪声表缺项在特定 biome/区域触发，生产稳定性课题（数据截止于此，4096 chunk 全程未完成）。
3. **角参数 +15→+16 修正待办**：Rust 两臂 heights4 参数 `cx*16+15`（量化后 +12）≠ Java SURFACE 四角 +16——完全对齐需改 +16（两臂，独立小包），**翻 shared 默认的前置条件**。
4. **shared 翻默认待用户拍板**：前置四项 ✅（零回归/无争用/淘汰无风险/stats 落盘）+ 建议与 +16 修正联动一次到位。

### 📌 记录指引
- 通用模式 → workflow-patterns #25 补充案例（静态「恰好一致」断言必须显式算术）+ #21 补充案例（hot/cold 95× + 单价稳定性作收敛判据）；build-tooling 发现 #13（GRADLE_USER_HOME 复发）/ #14（watchdog 强杀 + dump 内嵌 seed 头）。
- 产物：三份 candidate artifact + index.yaml 登记；judge 意见两份；cmd-output 六件（est-compare-p13/p13b、estopt-ab-arms-p0、estopt-mt-baseline、estopt-sweep、est-price-p24）。
- 状态：三份产物均 candidate（两轮 judge 建议 confirmed 前清偿文档级修正），confirmed 留用户；翻默认动作不在本 session 执行范围。
