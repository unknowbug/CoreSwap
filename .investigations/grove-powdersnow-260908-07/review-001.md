# judge 审查意见：grove powder_snow 双臂对拍 verdict（260908-07）

- 审查角色：core.judge（只出意见，不改 status；confirmed 留人类）
- 三源核对：① .artifacts/grove-powdersnow-verdict-260908-07.md ② git status/diff（3e7e766 HEAD + 工作区 5 项变更）③ .investigations/grove-powdersnow-260908-07/cmd-output/* 原始输出

## 1. 数字一致性（verdict vs cmd-output）——逐项核对

| verdict 数字 | 原始输出 | 结果 |
|---|---|---|
| 雪类总量 414,830 / 403,385 | diff: [RESULT] 414830 / 403385 | ✓ |
| pw 38,528 / 35,499；both 35,488；van-only 3,040；cpp-only 11 | pw-stat: 38528/35499, both=35488, van-only=3040, cpp-only=11；自洽（35488+3040=38528 ✓；35488+11=35499 ✓） | ✓ |
| pw 柱 18,498 / 17,517，共享 17,517，vanilla 多 981 | pw-stat: van=18498 cpp=17517 shared-cols=17517；18498-17517=981 ✓；3040/981≈3.10≈「约 3.1 块/柱」✓ | ✓ |
| y 廓线 96..161 mean 115.1 / 115.4 | pw-stat: min=96 max=161 mean=115.1 / 115.4 | ✓ |
| 表面 snow only_v 30,819 / only_c 31,985 | diff by-type: snow=30819 / 31985 | ✓ |
| snow_block only_v 10,473 / only_c 883 | diff by-type: sblock=10473 / 883；总量自洽（414830-403385=11445=44324-32879 ✓） | ✓ |
| 锚点 (0,131,-7) 双臂 powder_snow | diff [ANCHOR] 行 + preverify-points.txt 双证 | ✓ |
| (26,128,25)=air、(17,126,14) 邻域无雪 弃用 | preverify + diff [ANCHOR] van=- cpp=- | ✓ |
| dll size=2178560 sha256=7a411e01…、stageMask=3、enabled=true | cpp-arm-server.log:270/269 | ✓ |
| vanilla 臂 -PcppVanilla=1 | vanilla-arm-server.log:192「[BenchMod] vanilla mode: CoreSwap worldgen disabled (A/B comparison)」 | ✓ |

**发现一处未声明的口径差**：diff 脚本 `only_v pw=3032` vs pw-stat `van-only=3040`（差 8）。机制可解释：pw-stat 按「vanilla 有 pw 且 cpp 无 pw」计（含 same-pos-diff-type=123 中 cpp 同位放 snow/snow_block 的 pw 位），diff 脚本按同位严格缺块计。verdict 引 3040（pw-stat 口径）与总量自洽，无错；但两脚本数字并存于原始证据中未声明口径差，后续读者会当矛盾抓。→ 条件 N-1。

**覆盖面表述小瑕疵**：verdict「chunks x 0..31, z -32..-1」= 32×32=1024，与同句「1089 chunks 含 margin」（33×33=1089）不自洽；且 1024（对比）与 1089（region 内含 margin）两个数并存未分层声明。→ 条件 N-3。

## 2. 置信度合法性

- status: candidate（建议），无 confirmed 越权。✓
- candidate 有 Full 级运行时证据（region NBT 直读逐块，非静态/非探针反射）支撑，验证分层标注与实际载体一致。✓
- §16.3 交接验证先做（锚点复核通过才继承上会话 draft 方向，(0,131,-7) 复现、2 坐标证伪弃用并落盘）——交接纪律执行正确。✓

## 3. §9.7 三要素 + idk 诚实性

- 三要素声明完整（载体/覆盖面/历史口径），首跑无同口径历史数据声明诚实。✓（覆盖面措辞见 N-3）
- 残差未掩盖：van-only 3040/981 柱、snow_block 单向 10,473 vs 883 均独立列为未归因 idk，且给出候选 A/B + 判别实验（双臂高度场/humidity/slope 输入对比），候选 B 与支持证据的相斥关系如实写出。✓
- verdict 正文「机制解释（candidate 置信度）」明确标注候选倾向而非定论，与 idk 节无矛盾。✓

## 4. 落盘契约

- .artifacts/index.yaml 末尾条目在位（id/path/kind/status: candidate + 关联注释）。✓
- knowledge/discovered/workflow-patterns.md #86 在位，五段式（现象→根因→定位→教训→状态），载体正确（通用可复用判据，非一次性结论）。✓
- knowledge/INDEX.md 末尾 260908-07 追加在位。✓
- 架构计划 .investigations/000-架构设计/架构计划-260908-07.md 在位（轻量档、已批准、judge/fan-out/knowledge 预置齐全；Get-Date 锚定 12:48）。✓
- git 工作区变更与声明一致（新 verdict + 两 knowledge 改动 + index + 架构计划 + 本 investigation 目录；另有 mod-compat .tmp-jar/ 未跟踪目录，与本 verdict 无关，建议提交时排除）。

## 5. 证据链：柱包含推导

- shared-cols=17517 = cpp 柱总数 → cpp-only 柱 = 0，「cpp 柱足迹 100% ⊆ vanilla」推导成立。✓
- cpp-only pw 块 = 11 与 cpp-only 柱 = 0 不矛盾（同柱不同 y 可 van 缺 cpp 有），verdict 第 28 行（块）与第 42 行（柱）分开表述、未混用单位——表述精确。✓

## 6. 双臂对称性/载体有效性

- vanilla 臂：同实例 vanilla mode（CoreSwap worldgen disabled）有日志直证。✓
- Rust 臂：CppBridge enabled=true stageMask=3，dll size/sha 与 verdict 声明一致，且 verdict 已声明 dev 构建边界（非出货 1.0.26）。✓
- seed：level.dat（preverify ✓）+ CppBridge init seed 行（cpp log:269 ✓）。**第三查 server.properties level-seed 在 cmd-output 无落盘证据**——verdict 声明「三查全过」但证据只落盘两查。→ 条件 N-2。

## 条件（PASS-with-conditions）

- **N-1**：verdict §结果表或原始证据节补一行口径声明——diff 脚本 `only_v pw=3032` 与 pw-stat `van-only=3040` 的差 8 来自 same-pos-diff-type（cpp 同位有其他雪类）的计数口径不同，verdict 采用 pw-stat 口径。
- **N-2**：补 server.properties level-seed 核对的落盘证据（cmd-output 补一行输出或引用采集命令记录）；若无法补，verdict「三查全过」改为「三查：level.dat ✓ / CppBridge init ✓ / server.properties 该轮未留痕（执行过但无落盘）」的诚实降级表述。
- **N-3**：修正 §9.7 覆盖面行：chunk 范围写准（1089 = 33×33 含 margin，与 x 0..31/z -32..-1 的 32×32=1024 对比集分层表述）。

三条均为表述/落盘补齐，不动摇 verdict 机制结论与 candidate 建议。

## 结论

**PASS-with-conditions**（条件 N-1..N-3 如上；应用后建议维持 candidate，confirmed 由用户拍板）。
