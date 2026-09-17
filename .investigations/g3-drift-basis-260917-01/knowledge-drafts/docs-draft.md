# 草稿：docs 落盘（260917-01/02 收尾，subagent 产出，主会话应用）

> **应用位置与查重声明（主会话应用前必读）**：
> 1. `versions/1.20.1/docs/12-lighting.md`：末尾（「D-hm」行之后，文件现 156 行）追加 §1 小节。追加不覆盖。
> 2. `versions/1.20.1/docs/10-timewise-archive.md`：**查重命中**——该文件 3282-3296 行已存在完整 260917-01 块（三探针裁决链 / E1 VOID / run3 分裂 / judge 条件，格式与本要求一致），**不另立新块**。只做 §2 的一条**追记行**，追加到该块末尾（现 3296 行「📌 记录指引」行之后），更新 confirmed 状态与落盘指针。追加不覆盖。

---

## §1 12-lighting.md 末尾追加文本

## G3 漂移基底归因（260917-01，confirmed）

> 正式裁决：`.artifacts/g3-drift-basis-260917-01/verdict-260917-01.md`（judge PASS-with-conditions，C1/C2a/C2b 已应用；用户拍板 confirmed 260917-02）。域边界：seed 8576294172403134396 × dll 6F7FA3AE…2337 × snap_light.py 载具 × spawn 邻域箱 × Done+60s 口径；不外推其他 seed/载具/维度。

### 结论

G3 drift 基底（spawn 邻域 120 chunk，两臂形态无关同集，100% 聚集 x[15,36]×z[-24,-4]、边缘环 0%）由 **legacy per-chunk 光照路径的轮次级不收敛**主导：

- settle 持久性 rL = 99.2%（119/120，60s 与 20s changed 集基本不变）——非快照窗/停服时机效应；
- run3 legacy 仍 200 changed（9.88%）且漂移集换血（基底交集仅 48.3%），无不动点（5.93→6.27→9.88% 逐轮上升）；
- **域批路径一次重载收敛至不动点**（run3 changed 2，0.10%≈噪声地板）；
- 现象为光照接管形态特有（vanilla 光照同协议 ~0.8%，VOID 臂旁证，口径已声明）。

「域批有害」方向撤销，改「域批收敛性优于 legacy per-chunk（本载具口径）」；C-1 的 2.5% 阈值系随 G3 round-trip 载体一并回炉（round-trip 在两形态上量的是收敛行为不是质量，→ workflow-patterns #157）。

**限定（judge C2a/C2b，内嵌正文）**：判据未预登记分裂分支，逐臂读法为事后裁量（worker/judge 均已复核，→ workflow-patterns #159）；n=1 单 seed 单载具。「域批 5×5 全帧重算覆盖 legacy 陈旧历史」等机制解释为静态推演、无直接探针（P-α/P-β/P-path 未执行），**不属本结论内容**，仅存候选草稿（candidates/.b3 §1-§2）。

### §15.4 取代声明（supersedes 双指针，原文不删不改）

- **取代** 260915-03「G3 drift ~5.83% 由时机形态主导」（推翻理由一行：C-B1 settle rL=99.2% 证伪「快照窗/停服时机」轴；C-B4 run3 证漂移为 legacy 路径轮次级不收敛，非时机一次性效应）；
- **连带取代** 260916-01 record §4 候选机制「Done+20s 快照窗内完成时序边界」（同证据链）；
- 取代记录正式文本：verdict-260917-01.md §1（含时序锚 §1.1：criteria 定稿 14:10:40 < run_g3_run3.py 14:11:12 < 首臂日志 14:13:02）；原结论正文均不改。

### 证据指针

`.investigations/g3-drift-basis-260917-01/`（criteria C-B1/C-B4 条、cb1-set-compare / cb2-offline-result / control-calibration json、cmd-output/G17b-\*/G17c-\*、candidates/×3、judge-review-260917-01.md、g3-drift-errors.md E1/E2）。CP-1 重评决策输入（R1 载体换轨 / R2 载体换型双向并列，R1 前置 = 接管形态同配置噪声锚实测，K2）→ verdict §3。

---

## §2 10-timewise-archive.md 260917-01 块末尾追记行（追加到「📌 记录指引」行之后）

- ✅ **追记（260917-02）**：用户拍板 **confirmed**（§15.4 取代记录 + 归因结论）；CP-1 = **R1 保留 .b1 + 载体换轨**（R1 前置 = 接管形态同配置噪声锚实测，K2 升决策前置）。正式裁决 → `.artifacts/g3-drift-basis-260917-01/verdict-260917-01.md`（含时序锚 C1 + 限定 C2a/C2b 内嵌）；结论落盘 → 12 篇「G3 漂移基底归因（confirmed）」小节；workflow-patterns 新增 #159（分裂逐臂展开三条件）+ build-tooling #8/#118 家族补充案例（复测口径失传指针 → #156）同批落盘。
