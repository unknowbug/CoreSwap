# Judge 审查意见 — candidate c-B BeehiveTreeDecorator 实装（260905-10）

- 审查角色：core-judge / anchor-judge（subagent 隔离）
- 审查对象：`.artifacts/feature-parity/candidate-beehive-260905-10.md`（FEA-10，status: draft）
- 三源核对：① 产物快照 ✓ ② git 工作区 diff（tree.rs / placement.rs / index.yaml）+ git status ✓ ③ 验证记录（bA/bB/interim + cmd-output + region bin hash）✓
- **结论：APPROVE-WITH-CONDITIONS**（不修改 status；candidate 授予待条件核销 + 用户拍板；confirmed 留给人类）

---

## 一、逐消费点对拍（重点 1）

Java 权威：`.tmp/scout-260905-08/mcsrc/.../BeehiveTreeDecorator.java:41-75`；Rust：`worldgen-core/src/tree.rs:401-435`。

| 消费点 | Java | Rust | 判定 |
|---|---|---|---|
| nextFloat 门 | L43 `!(nextFloat >= p)`，恒 1 次 | tree.rs:403 `!(next_float() >= p)` 同构 | ✅ 一致 |
| i 推导（leaves 非空） | L46-47 `max(leaves[0].y-1, logs[0].y+1)` | tree.rs:406-410 用 y 极值近似（max(leaves y)-1、min(trunk y)+1），idk-bee1 已声明 | ⚠️ 近似，已声明 |
| i 推导（leaves 空） | L48 `min(logs[0].y+1+nextInt(3), logs[last].y)` | tree.rs:412-415 同构（min lo+1+nextInt(3), hi）；trunk 空时 return 发生在 nextInt(3) 之前，与 Java get(0) 抛异常前不消费等价 | ✅（空集病理分支见意见 3） |
| 候选枚举 | L49-52 每根 y==i log × GENERATE_DIRECTIONS，0 消费 | tree.rs:422-423 同结构，0 消费 | ⚠️ **方向集不一致（见意见 1，必须修）** |
| findFirst 单放置 | L55 shuffle（自有 Random，非世界流）→ filter(isAir(pos) && isAir(pos.offset(SOUTH))) → findFirst | tree.rs:425 空气+南侧空气判定同构；确定序取首候选（idk-bee2 已声明 shuffle 非确定序） | ✅ 流等价（意见 1 边缘形态除外） |
| 蜂数消费 | L59-64 `2+nextInt(2)` + ix×`nextInt(599)`，仅放置时消费 | tree.rs:428-429 同构，break 'outer 单放置 | ✅ 一致 |

### 意见 1（必须修，MUST）——Beehive 候选方向集与 Java 不一致，含一个未声明的 RNG 流分歧边缘形态

- Java `GENERATE_DIRECTIONS`（BeehiveTreeDecorator.java:24-28）= `Direction.Type.HORIZONTAL.stream()` 去 `SOUTH.getOpposite()`（=NORTH）。Direction.java:499 实证 `Type.HORIZONTAL` 序 = **NORTH, EAST, SOUTH, WEST** → 去 NORTH 后候选 = **{E, S, W}（3 向）**。
- Rust tree.rs:423 复用 `DIR_HORIZONTAL_JAVA_ORDER_VINE`（tree.rs:447）= **W,E,N,S（4 向）**——**多出 NORTH (0,-1)**，且 tree.rs:417 注释「HORIZONTAL 流序 N,W,S,E 去 NORTH → W,S,E」对 Java 流序的陈述本身错误（实际 N,E,S,W → E,S,W）。
- 影响：候选枚举 0 消费，常规情形流不受影响——但当且仅当某 log 的 E/S/W 三向全非空气而 N 向空气+南空气时，**rust 放置并消费 `2+nextInt(2)+ix×nextInt(599)`，java 不放置 0 消费** → RNG 流分歧（与产物「流不受影响」的断言在该边缘形态下不成立）。概率极低（bee_nest signed 0 旁证本轮 dump 未触发），但机制存在且未在 IDK 清单声明。
- 修复建议：Beehive 用独立 3 向常量 `[(1,0),(0,1),(-1,0)]`（E,S,W），或复用 4 向后 `if (dx,dz)==(0,-1) continue;`；同时修正 tree.rs:417 注释。修复不改常规消费数，region 结果预计不变（可只静态复核 + 抽 chunk 复跑）。

### 意见 2（通过）——leaves 空/非空分支语义

- `leaves_set.is_empty()` ↔ Java `list.isEmpty()`（L46）同构；leaves 空时 rust 多消费 nextInt(3)（tree.rs:415）与 Java 一致。✅

### 意见 3（备注，不阻塞）——空集病理分支

- leaves 非空 + trunk 空：rust `unwrap_or(0)` 得 i=max(a-1,1)（tree.rs:408-410），Java 会在 `list2.get(0)` 抛 IndexOutOfBounds（L47）。防御性偏离，实际树配置不可能 trunk 空，仅记录。
- trunk 空 + leaves 空：rust return 于 nextInt(3) 前（tree.rs:412），Java 抛异常前同样 0 消费——语义等价（均异常路径，rust 静默）。✅

### 意见 4（备注）——「确定序」表述

- trunk_set 为 `Vec`（push 序=生成序，tree.rs 签名 `&Vec<[i32;3]>`），rust 候选序确定 ✓，与产物「确定序首候选」一致；java shuffle 自有 Random 非确定 → 放置点本就无逐位对齐目标，idk-bee2 覆盖。无需动作。

---

## 二、region 三臂与口径（重点 3）

### 意见 5（通过）——三臂数据与 hash

- `Get-FileHash`：region_bee_ca.bin = region_bee.bin = `4D216088…48FD`（默认态=CA-on，与「WG_CA_MIN 默认开」实证自洽）；region_bee_only.bin = `19BA41F4…53E474`。三 bin 均 431,178,924 字节、seed 断言 8576294172403134396（v25 脚本 L18 assert）。
- v25/v25b 脚本 FAMS 含 bee_nest（v25_bee_region.py:33），产物「bee_nest 0/0 两侧一致」= signed 总差 0，与 gate 0.002 稀释一致。⚠️ 表述建议：signed 0 可能掩盖「两侧各 ~N 个但位置错开」（idk-bee2 域），「两侧一致」措辞过强，建议改「signed 差 0（位置级不保证，见 idk-bee2）」。
- **§9.7 口径三要素**：载体（idk7_region_dump 同源 bin）✓、覆盖面（2193 chunks signed 全量）✓、可比性（v18 同口径，脚本头注明）✓——齐。

### 意见 6（条件，SHOULD）——臂数字原始输出未落盘

- +30807/-12055/+1972/-973 等数字仅存在于 v25/v25b 脚本 `print()` 输出，`.investigations/feature-parity/260905-10-cmd-output/` 与 .tmp 均无对应 stdout 落盘（全仓 grep 无命中）。脚本 + bin + hash 在，可复现，但按「原始输出落盘」契约应补：重跑 v25/v25b 把 stdout 存 cmd-output（成本一轮，无需重采）。

---

## 三、WG_TREEDIAG 门控（重点 4）

### 意见 7（通过）——默认关 + 热路径

- placement.rs:281-286：`OnceLock<bool>` + `std::env::var("WG_TREEDIAG").is_ok()` 显式判存在 → **unset 默认关** ✓（正确避开 env_enabled 默认开语义，注释自警）。热路径 = 每 get_positions 一次 OnceLock get（原子读），非每点 env 查询——可接受，符合「诊断门控 chunk/调用级一次」纪律（非严格零成本，量级无害）。
- Rust 侧 [BEE-MISS] 打在 Unsupported 分支（tree.rs:436-438）语义为「任何 unsupported decorator 0 消费」，名称易误读为 beehive 专属——minor，命名可改 [DECOR-MISS]。
- Java 侧 mixin（BeehiveDecoratorMixin.java:20）`Boolean.getBoolean("wg.treediag") || env != null` 默认关 ✓；TrunkPlacerMixin/Count/Square mixin、build.gradle、coreswap.mixins.json 均在盘（runtime/1.20.1/java，mtime 2026/9/5 19:46-20:59）。

---

## 四、三源一致性（重点 2/5）

### 意见 8（条件，声明义务）——runtime 侧 mixin 不可由 git diff 核对

- `git status`：仅 index.yaml / placement.rs / tree.rs 修改 + 产物/调查新增；**无任何 runtime 侧文件**——runtime/ 已被 commit 0fc44d3 untrack（local-only）。即 mixin（TrunkPlacerMixin/BeehiveDecoratorMixin/Square/Count）+ build.gradle + coreswap.mixins.json 的三源核对只能对盘上文件做，git diff 覆盖不到。candidate 应补一行声明「runtime 侧证据为本地未跟踪，git diff 不含，验证可复现性依赖本地 run 目标」，防后续 session 误判。
- tree.rs/placement.rs diff 与 candidate 声明一致：Beehive 变体（:311）+ parse（:325-326）+ generate（:401-435）+ [TH]/[CNT]/[SQ] 打点；无夹带改动。✅

### 意见 9（条件，§15.4 supersedes）——WG_CA_MIN 口径修正链未闭合

- 本轮实证：worldgen_handle.rs:876 `env_enabled("WG_CA_MIN")` = unset→true（默认开），推翻 FEA-9 title 的「WG_CA_MIN 默认关」。但：
  1. FEA-10 index 条目与产物只有「口径修正」文字，**无显式 supersedes 指针**（应为「本条目 supersedes FEA-9 title/verdict 中『WG_CA_MIN 默认关』表述」双指针格式）；
  2. worldgen_handle.rs:105、:875 两处注释仍写「默认关待验证」——与本轮实证矛盾，应随本批更新；
  3. FEA-9 拍板语境是「默认关 + 翻默认另需端到端性能对比」——现状默认开与该决定存在张力，**是否接受默认开为长期态需用户显式裁决**（本 judge 仅指出，不裁）。

### 意见 10（通过）——index/standing

- FEA-10 条目已入 index.yaml（status: draft，verdict 如实标注 judge 待审）✓。standing 无强制新增项；可选：将「WG_CA_MIN 默认开」作为 standing 提示条（与意见 9 合并处理即可）。

---

## 五、决定性证据复核

- `.investigations/feature-parity/260905-10-cmd-output/rust-treediag-chunk37--16-beefix.txt` L692-694：`[SQ] 602,-244` → `[TH] 6 @ (602,72,-244)` → `trees_birch_and_oak placed` —— 与产物声明的 java 树1 `SQ(602,-244) → THJ=6 → [BEE]` 重合 ✓（java 侧 [BEE] 打点在 treediag-vanilla.log 有实证，95MB 已抽样确认存在）。
- 修复前后 [SQ]/[TH] 序列逐行 diff：前 480 行全同，L481 起分叉——与「修复引入每树 1 次 nextFloat，分叉自 decorator 后传播」的机制时序一致 ✓。
- bA（否证）/bB（排除 + 范围外发现 beehive 缺抽）记录完整、分层标注诚实（bB 自标 Degraded 静态）。✅

---

## 六、总裁定

**APPROVE-WITH-CONDITIONS**：机制主线（beehive 恒 1 次 nextFloat 缺抽 → 树2 起流漂移）证据链闭合、决定性数据可复核、降级/IDK 声明诚实。条件（candidate 授予前）：

| # | 条件 | 级别 |
|---|---|---|
| C1 | Beehive 候选方向集改 3 向 {E,S,W}（独立常量或滤 NORTH），修正 tree.rs:417 错误注释；静态复核 + 抽 chunk 复跑确认流不变 | MUST |
| C2 | 补 v25/v25b stdout 落盘 cmd-output（臂数字原始输出） | SHOULD |
| C3 | FEA-10 条目补 §15.4 supersedes 指针（FEA-9「默认关」表述）；更新 worldgen_handle.rs:105/:875 过时注释；「默认开长期态」提请用户裁决 | SHOULD（第 3 点 MUST-ask-user） |
| C4 | candidate 补声明：runtime 侧 mixin 为 untracked local-only，git diff 不覆盖 | SHOULD |

建议状态：核销 C1（+C3 文字项）后可升 **candidate**；confirmed 由用户拍板。残余（jungle_l -12055、树3+ 状态依赖短路、R-1 HashSet 序）已在产物如实持有，不阻塞本 candidate。
