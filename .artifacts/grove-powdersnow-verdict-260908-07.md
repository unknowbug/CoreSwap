# grove powder_snow 双臂对拍 verdict（260908-07）

- **status**: **confirmed（用户拍板 260908-07 · 实际 2026-09-08 13:09 授权确认；judge PASS-with-conditions N-1~N-3 已应用）**
- **验证分层**: Full（region 存档 NBT 直读逐块对比，非静态审查/非探针反射）
- **supersedes**: 无（继承上会话 draft 判定「grove 细雪 = vanilla 陷阱机制非 bug」，本轮以同坐标 Java 对照补齐其缺失证据；原 draft 无独立编号，继承关系在此声明）
- **课题**: grove 生物群系细雪（powder_snow）生成差异是 vanilla 陷阱机制，还是 Rust surface 规则 bug？

## §9.7 验证可比性声明（三要素）

| 要素 | 声明 |
|---|---|
| 载体 | Chunky 1.3.146 预生成 → gradle runServer（loom, Fabric 1.20.1）双臂存档 region 直读，Python NBT block_states 逐块提取 |
| 覆盖面 | region r.0.-1 全量（chunks x 0..31, z -32..-1；1089 chunks 含 margin，square center (256,-256) radius 256），1024 chunks 雪类方块（powder_snow/snow/snow_block）全量逐块对比 |
| 历史口径可比性 | 无历史同口径可比数据（首跑）；上会话 F3 目测坐标仅作锚点复核用，不构成同口径对比 |

> **对比集口径（judge N-3 修正）**：对比集 = region r.0.-1 内 chunks x 0..31, z -32..-1（**1024 chunks**）；Chunky 任务覆盖 33×33=1089 chunks（含 margin，margin chunks 不在本对比集内）。两数分层使用，1024 为对比覆盖面。

## 对照条件

- seed = -546755292641445454，双臂同 seed；seed 三查：level.dat WorldGenSettings.seed 直读 ✓（.tmp/powdersnow-preverify-260908-07.py 输出落盘）+ CppBridge init 行 ✓（cpp-arm-server.log）+ server.properties level-seed 行 ✓（本轮启动前 Select-String 输出，会话内直证、未落盘 cmd-output——judge N-2 降级声明：第三查为会话内直读，证据强度与前两查同源但无独立落盘文件）。
- vanilla 臂：-PcppVanilla=1（同实例对称设计，标准载体决议 260906-09 用户拍板）。
- Rust 臂：默认参数；CppBridge 管线核验 PASS（init seed ✓ enabled=true stageMask=3；dll size=2178560 sha256=7a411e013f2eaa25…，为当前 dev 构建——注意与出货 1.0.26 dll size=2160640 不同，本 verdict 仅对 dev 构建负责）。
- 两臂 stop 走 RCON，region 文件落盘 `.tmp/grove-powdersnow-260908-07/region-{vanilla,cpp}/r.0.-1.mca`。

## 结果（实测）

| 指标 | vanilla | cpp | 备注 |
|---|---|---|---|
| 雪类总量 | 414,830 | 403,385 | 差 ~2.8% |
| powder_snow | 38,528 | 35,499 | both=35,488；van-only=3,040（981 柱，约 3.1 块/柱）；cpp-only=11（注：diff 脚本 only_v pw=3032 与 pw-stat van-only=3040 差 8 = same-pos-diff-type 计数口径不同，本表取 pw-stat 口径——judge N-1 注记） |
| pw 柱足迹 | 18,498 | 17,517 | cpp 17,517 柱 100% 与 vanilla 共享（17517/17517）；vanilla 多 981 边缘柱 |
| pw y 廓线 | y 96..161, mean 115.1 | y 96..161, mean 115.4 | 一致（均值差 0.3） |
| 表面 snow 层 | only_v 30,819 | only_c 31,985 | 双向近似平衡，符合已知地形微差残差域（~15 块/chunk 量级级联） |
| snow_block | only_v 10,473 | only_c 883 | 单向偏 vanilla 多，与 pw 残差同向 |

锚点核查：(0,131,-7)（上会话 F3 现场细雪点）双臂均为 minecraft:powder_snow ✓。
交接坐标复核：(26,128,25)=air（邻域坡面 snow/snow_block，疑眼位坐标）；(17,126,14) 邻域无雪——F3 瞬时快照无落盘，弃用。

## Verdict（candidate）

**grove 细雪 = vanilla 陷阱机制（powder snow trap feature），非 Rust surface 规则 bug。**

支持证据：
1. **同坐标同 seed 下 cpp 柱足迹 100% 被 vanilla 包含**（17517/17517，cpp-only 仅 11 块）——不存在「Rust 多生成」的机制性偏移；差异方向单一为 vanilla 多出 981 个 pw 边缘柱。
2. **y 廓线一致**（同范围同均值 ±0.3）——pw 的纵向放置逻辑（surface 规则的 depth/drop 参数语义）双臂一致。
3. **锚点 (0,131,-7) 双臂均为 powder_snow**——上会话 F3 现场差异点在正确 seed 对照下不复现，原「差异现场」不成立。

机制解释（candidate 置信度）：van-only pw 集中于边缘柱（981 柱 × ~3.1 块/柱）与 snow_block 单向偏差（only_v 10,473 vs only_c 883）同向，模式符合**地形高度微差级联**：pw 陷阱 feature 依赖表面高度/坡度条件触发，Rust 与 vanilla 在 grove 高度场上的微小残差（已知 ~15 块/chunk 量级域）使部分边缘柱越/不越触发阈值。即差异是上游地形残差的下游表现，非 pw/surface 规则本体错误。

## @anchor.idk 残差诚实声明（未归因，列为后续候选）

以下残留**未归因**，不得随本 verdict 一并视为已解释：
- van-only 3,040 pw / 981 边缘柱——候选 A：地形残差级联（与表面 snow 双向平衡一致，当前倾向）；候选 B：pw/snow_block surface 规则阈值实现差（与 cpp 柱 100% 包含、y 廓线一致两证据相斥，但未做规则级排除）。
- snow_block 单向偏 vanilla 多（only_v 10,473 vs only_c 883）——量级远超表面 snow 的双向平衡域，同向于 pw 残差，但独立归因未做。
- 判别实验（后续候选）：对 van-only 柱取样，输出双臂该柱的 surface 高度场/humidity/slope 输入对比——若输入差在残差域内则归因 A 坐实；若输入一致而输出差则升级为规则 bug 重开。

## 前置依赖与边界

- 继承上会话 draft「vanilla 陷阱机制非 bug」：本轮以同坐标 Java 对照补齐其证据缺口（§16.3 交接验证：锚点复核通过后才继承方向）。
- 本 verdict 针对 dev 构建 dll（size=2178560），出货构建一致性未在本轮覆盖。

## 原始证据

- `.investigations/grove-powdersnow-260908-07/cmd-output/`：diff-snowclass-r0.-1.txt / pw-stat-r0.-1.txt / preverify-points.txt / inventory-rust-side.txt / vanilla-arm-server.log / cpp-arm-server.log
- region：`.tmp/grove-powdersnow-260908-07/region-{vanilla,cpp}/r.0.-1.mca`
- 脚本：`.tmp/grove-diff-260908-07.py` / `.tmp/grove-pwstat-260908-07.py` / `.tmp/powdersnow-{preverify,scan,inventory}-260908-07.py` / `.tmp/grove-rcon-260908-07.py`
