# Q-AQ1 b2 候选：35ms 差分的非 classify 污染/级联（260903-10，status: draft）

作者：fan-out worker b2。置信度：**candidate**（数据层实测 + 静态审计；confirmed 留人类）。
验证分层：Full（2×2 交错 bench + 4 组计数器运行，数据层）+ 静态审计（carver.rs / worldgen_handle.rs / terrain.rs / aquifer.rs）。
口径（§9.7）：载体 = qaq1_b2_ab_bench v2（chunk 粒度交错，3 轮×64 chunks/arm，median，n=192/arm）+ qaq1_counter_probe（16 chunks，同 seed/region）；与 Q-PD1 62ms 口径族同 seed=8576294172403134396 / region(200,200)；与 qpd1_stage_bench 结果可比（同门控机制、同 median 判据），但注意 qpd1 是六配置顺序执行、本实验是四臂 chunk 级交错（抗漂移），绝对值有 ~±10% 漂移、差分结构可比。

## 0. 一句话结论

**b2 主张被否证**：qpd1 的 aquifer 段 35.07ms **不是** carver 级联污染——qpd1 的 noore/noaqu 两臂 **carver 都被 skip**（qpd1_stage_bench.rs L50-51 累进跳过表），本实验直接测得纯 classify aquifer 成本 **A|Coff = 33.47ms ≈ 35.07 ✓**。**35ms 是真实的 classify 侧 aquifer 成本（≈67k applies/chunk × ~500ns），支持 b1。** 但实验发现一个**新的未解异常**（见 §5）：carver 在全 Air 列上反而比实地形贵 ~12ms/chunk。

## 1. 静态审计（逐行核实）

### 1.1 carver 直调 apply（b2-① 代码事实成立，量级不足）
- `carver.rs L406-410`：`get_state` → `y <= lava_level(-56)` → lava（**不触 aquifer**）；否则 `ctx.aquifer.apply(x,y,z,0.0)`——**绕过 `VanillaAquifer.skip_aquifer` 标志**（直接拿 `va.aq`，worldgen_handle L520 传 `&mut va.aq`）。✓ 机制存在。
- `aquifer.rs L234`：`apply` 首行 `if density > 0.0 { return -1; }`——carver 传 0.0 → 早退失效，每点走完整 12 格邻域（L245-255）+ get_water_level_at（L257）+ 可能 calculate_density。✓ 机制存在。
- 但 `carve_at_point`（L366-396）**先 `can_always_carve_block`（L382，replaceable 54 项 tag 展开，carver.rs L175-204）再 get_state**；且 `carve_region` 的 mask/几何循环与列内容无关。
- **量级实测**（qaq1_counter_probe，RUN A vs RUN D）：生产 bp 815,747/chunk，carver-off 时 806,672/chunk → **carver 的 aquifer.apply 仅 ≈ 756 次/chunk**（9k bp ÷ 12）。× ~500ns ≈ **≤0.4ms/chunk**。不可能贡献 ~29ms。❌

### 1.2 skip_aquifer 语义（cascade 路径核实）
- `terrain.rs L222-233`：`classify`：d>0→Rock（两配置同）；`skip_aquifer` → **直接 Air，不调 aq.apply**（L231）。
- 因此 no-aquifer 配置下 BlockColumn 地下全 Air → carver `can_always_carve_block(Air)` 失败（air 不在 replaceable 表，L178-202 核实）→ carve_at_point 早退。✓ 级联方向如假设，但由 §3 计数器知 carver 本就没有多少 apply 可省。

### 1.3 WG_SKIP_* env（b2-④ 否证）
- 全部 chunk 级读取一次：L452/473（fill_chunk_blocks）、L513/519/524。5 次 `env::var`/chunk ≈ µs 级。❌

### 1.4 macro cell-grid（b2-③ 否证）
- `fill_chunk`（terrain.rs L247+）中 aquifer 只经 `classify` 进入；cell-grid 采样器（macro_sampler/transpiler/gpu 分支，worldgen_handle L456-467）不读 aquifer 状态，ON/OFF 无行为差。❌

## 2. 关键再核对：qpd1 的 aquifer 段两臂 carver 均被 skip（b2 前提错误）

qpd1_stage_bench.rs L50-51（原文核实）：
- `m_noore` skips = [OREVEIN, **SURFACE**, **CARVER**, FEATURES] —— carver 关
- `m_noaqu` skips = [AQUIFER, OREVEIN, SURFACE, **CARVER**, FEATURES] —— carver 关

→ **aquifer 段 = m_noore − m_noaqu 是 carver-off 下的纯差分**，不含 carver 直调 apply，也不含 air-列级联。证据包 F6「carver 的 getState 直调绕过 skip 标志（污染 aquifer 段）」对该差分**不适用**——carver 在两臂都不跑。

## 3. 实验：aquifer × carver 2×2（qaq1_b2_ab_bench v2）

设计：OREVEIN/SURFACE/FEATURES 四臂全 skip，只切 A(skip_aquifer)×C(skip_carver)；**chunk 粒度交错**（每 chunk 依次测 4 臂，消机器漂移——v1 顺序执行曾出现 round2 全臂 +8~13% 漂移导致的假负交互，v2 三轮臂间中位数稳定 ±0.5ms）。

结果（n=192/arm，median）：
```
m00(A on,C on)=56.46  m01(A on,C off)=45.84  m10(A off,C on)=35.34  m11(A off,C off)=12.37
A|Con = m00−m10 = 21.11   A|Coff = m01−m11 = 33.47   I(交互) = −12.35
C|Aon = m00−m01 = 10.62   C|Aoff = m10−m11 = 22.97   (违反物理约束 C|Aon≥C|Aoff!)
```

计数器交叉验证（qaq1_counter_probe，16 chunks，全段开基座）：
```
RUN A 全开:            71.00ms  bp=815,747 wl=110,161 barrier=10   /chunk
RUN B A-off:           48.22ms  bp=  8,912 wl=  1,116 barrier= 1   /chunk  → carver-only apply≈756
RUN C A-off+C-off:     17.60ms  bp=      0 wl=      0 barrier= 0   /chunk
RUN D C-off:           58.85ms  bp=806,672 wl=109,029 barrier= 9   /chunk  → classify apply≈67.2k
```
- classify 侧 apply = 806,672/12 ≈ **67.2k/chunk**（F7 的 64-68k ✓）。
- carver 直调 apply = 815,747−806,672 → **≈756 次/chunk**（F3 的「68k 全部 apply 含 carver」中 carver 份额被高估为 4k，实际 0.75k）。

## 4. 判读表（预期 → 实测 → 裁决）

| 读数 | 若 b2 主导 | 实测 | 裁决 |
|---|---|---|---|
| A\|Coff（纯 classify aquifer，无 carver） | 应显著 < 35.07（污染被剔除后剩小值） | **33.47 ≈ 35.07** | ❌ b2 否证：35ms 是真 classify 成本 → **支持 b1** |
| carver×aquifer 交互 I | 大正值（carver 工作被误归入 aquifer 段） | **−12.35（负！）** | ❌ 且暴露新异常（§5） |
| carver 直调 apply 量级 | 每雕刻点全量 apply，可达数万/chunk | **≈756 次/chunk ≈ ≤0.4ms** | ❌ b2-①② 量级不足 |
| env 读取开销 | 可测量级 | 5 次/chunk，µs 级 | ❌ b2-④ |
| macro 采样器行为差 | ON/OFF 有隐藏差 | 代码无耦合路径 | ❌ b2-③ |

## 5. 新发现（b2 范围外，建议新候选 b4）

**carver 机械成本列状态相关，方向反直觉，机制未定位（@anchor.idk）**：
- 全 Air 列（A-off）上 carver 开启比关闭贵 **22.97ms/chunk**（bench）/ 30.6ms（counter B−C）；实地形（A-on）上只贵 **10.62ms**（bench）/ 12.15ms（counter A−D）。两独立仪器一致。
- 静态上 carver-on-air 每点工作严格 ≤ carver-on-stone（air 在 replaceable 扫描中必败早退，stone idx0 命中+apply），计数器又证明两配置 aquifer 调用都只有 ~756 次 → **~12-23ms 的 carver 机械成本在 289 邻居 seed/should_carve + 隧道几何 + CarvingMask 位运算里，且为何 Air 列更贵无法用现有代码解释**。
- 候选解释（未验证）：① mask-miss 的 `can_always_carve_block` 54 项线性扫描（`iter().any`，失败时全扫 ~200ns/点）在 Air 列全量触发、实地形被「d≤0 → classify Air 洞穴占地下 ~69%」部分抵消——但无法解释方向；② 生产走 uniform 路径（worldgen_handle L621-638），WG_CARVERDIAG（L641+）在 uniform 路径**死代码**，无现成内部分解工具。
- 影响：不污染 qpd1 各段（carver 段两臂 A 都开）；但**任何「A-off 且 carver-on」的诊断配置**（如 RUN B 型单变量实验）会被 +12~23ms 污染。**后续 aquifer 归因实验一律把 carver 双臂同时关掉**（本实验 A|Coff 口径）。

## 6. 对 G1 的回传

- 35.07ms ≈ 纯 classify aquifer 成本（67.2k applies × ~500ns）。diag 微测内部 ~90ns/apply 与生产 ~500ns/apply 的 **6× 单价差是主要矛盾 → b1 接手**。
- b2 的四个子机制全部否证（或量级不足 1ms）。
- 交给 b1 的线索：生产 apply 与 diag apply_breakdown（F4）的载体差——生产走 `Aquifer::apply`（共享 `self`、含 surface_cache/Aquifer 实例字段、&mut 独占）vs diag 独立构建；另有 macro cell-grid 插值 d 值分布（生产 classify 输入 d 是插值后值，diag 是树直采）可能影响 `d<=0.0`（L260）与三个 `calculate_density` 分支的命中率——F2 的 wl 110k/67k ≈ 1.64 次/apply 说明大部分 apply 走到 fl3（L263），即普遍进入第一个 calculate_density。

## 7. 复现命令（pwsh，bin 已构建于 target/release）

```pwsh
# 2×2 交错 bench（~90s）
& E:\PYTHON\CoreSwap\WorldgenRust\target\release\qaq1_b2_ab_bench.exe
# 计数器四象限（每 ~15s）
$exe='E:\PYTHON\CoreSwap\WorldgenRust\target\release\qaq1_counter_probe.exe'
& $exe                                            # RUN A 全开
$env:WG_SKIP_AQUIFER='1';           & $exe; Remove-Item Env:WG_SKIP_AQUIFER   # RUN B
$env:WG_SKIP_AQUIFER='1'; $env:WG_SKIP_CARVER='1'; & $exe; Remove-Item Env:WG_SKIP_AQUIFER,Env:WG_SKIP_CARVER  # RUN C
$env:WG_SKIP_CARVER='1';            & $exe; Remove-Item Env:WG_SKIP_CARVER    # RUN D
```
源码：`WorldgenRust/src/bin-diag/qaq1_b2_ab_bench.rs`（构建方式：临时挪入 `src/bin/` → `cargo build --release --bin qaq1_b2_ab_bench` → 挪回，bin-diag 纪律）。

## 8. 错误/教训记录（本候选过程）

1. **v1 顺序四臂 bench 出现物理不可能的负交互（air 列 carver 比实地形贵 26ms）→ 差分实验必须抗漂移**：round2 全臂 +8~13% 机器漂移下，臂间顺序执行的大差分不可信；chunk 粒度交错（每 chunk 轮询 4 臂）后三轮稳定。与「测量/探针污染铁律」同族：多臂差分 bench 的顺序效应是新的污染源。
2. **fan-out 输入的 F6 前提未先核 qpd1 源码跳过表**——「aquifer 段含 carver 污染」的前提与 qpd1_stage_bench L50-51 的实际累进跳过表矛盾。教训：接手差分结论先读 bench 的 skip 组合表（一行核对，成本远低于一轮实验）。
