# core.judge 审查意见 — est shared 臂 Java 逐位裁决（candidate）

- 审查对象：`.artifacts/lossless-accel/est-shared-verdict-260903-12.md`（candidate）
- 勘探地图：`.investigations/lossless-accel/est-shared-java-map/java-est-chain.md`（draft）
- judge：core-judge subagent，260903-12；只出意见不改 status
- 三源核对：① verdict + scout 地图 ② 原始输出（est-compare-p13/p13b、estopt-ab-arms-p0、estopt-sweep、.tmp/estdump/*.csv）③ 代码现场（worldgen_handle.rs L517-555、aquifer.rs L343-377、ChunkNoiseSampler.java L222-240、MaterialRules.java L488-516）
- 独立复算：`.tmp/estdump/judge_recheck_26090312.py` + `judge_sign.py`（只读，全部数字逐项复现，见下）

## A. 裁决逻辑 — PASS（附 2 条 CONCERN）

复算逐项复现原始输出：java entries=11877 conflicts=0、rust 66 chunks（region 过滤后 64）、[off] 0/64、[shared] 64/64、java-missing=192、p13b 63/64 + 敏感 chunk (201,200) java@+16=56 / shared@+12=48 / off=55、x0 vs x0+16 列值差异 3/64——与 verdict 及两份 cmd-output 完全一致。

反例可能性逐一排查：

1. **Java dump 只覆盖部分列**：成立且不影响裁决方向。复算证实 Java 表列 residue（mod 16）只有 {0,4,8}×{0,4,8} 九类，**无 residue-12 列**——Rust c1-c3 角（+15 量化→+12）在 Java 表中无对应列，192 missing = 3 角 × 64 chunk，自洽（256−64=192）。因此 **shared 的等值证明严格来说只覆盖 c0 原点角列（64/64）**；c1-c3 的等值性由 p13b 敏感性间接界定（63/64 列值量化不变 + 唯一敏感 chunk 已明示差异）。verdict 措辞「共同列全量对比」与实际情况相符，且后续建议（先 +16 修正再翻默认）正确覆盖了这个缺口。**成立**。
2. **量化归并错位**：Java 表 conflicts=0（复算确认），est 是量化列的纯函数，同列归并无错位风险；p13b 的跨列对比（shared@+12 vs java@+16）被正确表述为「敏感性探测」而非等值证明，无越位。
3. **dump 时序**：est 是列的确定性纯函数（conflicts=0 佐证），dump 时序不影响值本身。
4. **off 0/64 → 系统性偏离**：实证成立，且复算发现偏离高度规整——**c0 列 off 恒为 java−1（64/64 全部 delta=−1）**。

**CONCERN-A1（机制归因缺口，非阻塞）**：c0 列上 D1（量化）与 D3（noise_height vs height，overworld 同 384）均为 no-op，**verdict 引用的 D1/D3 候选无法解释 off 在 c0 的偏离**。judge 复算给出具体新线索：off 臂扫描 `(min_y..min_y+noise_height).rev().step_by(8)` 是**半开区间 rev，首采样点 = 319**；Java 从 `k+height = 320` 起扫（320,312,…）——**off 臂扫描网格整体偏移 −1（319,311,… vs 320,312,…）**，这同时解释了「c0 也偏离」与「delta 恒为 −1 的规整性」。建议：把此线索补进 off 偏离机制段（或另立小课题验证）；这不影响「shared=修正」的裁决方向，但 off 是**当前默认臂**，−1 系统偏移是活的生产 bug 线索，不应停留在「系统性不一致」一笔带过。

**CONCERN-A2（表述精度）**：verdict L16「shared 64/64 与 Java 逐值一致」严格限于 c0 原点角列；建议正文明确「共同列=c0 原点角（其余 3 角列 Java dump 无对应列）」，避免读者高估覆盖面（§9.7 覆盖面要素的精确化）。

## B. 角参数 +15 vs +16 新发现 — PASS

- Java 侧已核源码：MaterialRules.java L496-499 四角取点 = `chunkToBlockCoord(i)/(i+1)` = `i<<4` / `(i+1)<<4`，即 chunk 原点角与 **+16** 角；+16 经 `(x>>2)<<2` 量化后仍 +16（16>>2<<2=16，复算确认）。
- Rust 侧已核源码：worldgen_handle.rs L537-538 heights4 参数确为 `cx*16+15`（两臂同）；+15 量化 = +12（复算确认）。
- scout 地图 #7 确实失真且**内部自相矛盾**：该行先写「11 角=+12（60,60 量化→60）」——60>>2<<2=60 说明 Java 11 角（+16=64…即 60 那类列）量化不变——随即又称「+15 量化后=+12 恰与 Java (i+1) 角一致」。+12 ≠ +16，结论与其自身摘录冲突。verdict 标记为「静态推断失真（#25 家族）」**成立**。
- 影响面数字（1/64 角值差、3/64 列值差 → 1.6%~4.7%）复算一致。

## C. §9.7 三要素 + seed/坐标纪律 — PASS（附 1 条 CONCERN）

- 三要素在 verdict 头部齐备：载体（Java mixin RETURN dump vs Rust WG_EST_DUMP 角值）/ 覆盖面（seed、region、共同列口径）/ 历史口径（与 260903-11 四臂 hash 同 seed 同 region 可比）。坐标语义两侧一致（同为量化列对比，坐标错位风险已由同列 key 归并排除）。
- **CONCERN-C1（seed 证据 hygiene，非阻塞）**：seed 8576294172403134396 在 estopt-mt-baseline / estopt-sweep 输出头有打印，但 **estopt-ab-arms-p0-260903-12.txt（P0 hash 复现）与三份 dump CSV 内均无 seed 字段**——seed 两侧一致目前靠 session 流程与同 session 其他输出佐证，未内嵌于本轮关键原始文件。不阻塞（数值互证强：off/shared hash 与 260903-11 逐项一致、Java 对比 64/64 规整），但建议后续 dump 工具在文件头/行头回显 seed（探针三查铁律的落盘化）。

## D. 证据链完整性 — PASS

- 原始输出全部存在且可溯源（cmd-output/ 三件 + sweep + .tmp/estdump/ 三 CSV）。
- 数量自洽：11877 条 Java、conflicts=0、64 chunks（66 行含 (400,400) 预热区，过滤正确）、192 missing 有机制解释（见 A.1）、judge 全量复算零偏差。
- 产物契约：root `.artifacts/index.yaml:738` 已登记本 verdict id。
- 附带 panic 发现证据在案：estopt-sweep-260903-12.txt 尾部 `surface_rules.rs:505 missing noise sampler`，发生于 block 8（2304 chunks）后，verdict 记「~2304-2560」略宽但属实。

## E. supersedes 表述 + panic 边界 — PASS

- supersedes 双指针合规（§15.4）：指向 260903-11「未裁决假设」快照，一行升级理由，原条目不删不改。
- panic 发现边界正确：明确「另立待查，不阻塞本裁决」，未混入本裁决证据链，未越权下根因结论（只写「疑似预加载噪声表缺项」，带待查标注）。
- 建议 1/2（先 +16 再翻默认、修正后复跑）与本审查结论一致；补充建议 3：off 臂 `.rev()` 扫描网格 −1 偏移线索（CONCERN-A1）另立验证，**因 off 是当前默认臂，其生产影响独立于翻默认决策**。

## 总体建议

- **推荐状态：维持 candidate 合理，同意上报用户 confirmed**（裁决方向「shared=修正、off=系统性偏离」由 Full 层直接对比支撑，复算零偏差）。
- confirmed 前建议顺手完成（均为文档级，不改代码）：① CONCERN-A2 一句措辞精确化；② CONCERN-A1 线索落盘（可只加一行「off 偏离机制候选：rev 半开区间扫描起点 319 vs Java 320」）。
- 翻 WG_EST_SHARED 默认不在本 verdict 授予范围，前置（+16 角修正 + 复跑）与 260903-11 review 的门控约定一致。
