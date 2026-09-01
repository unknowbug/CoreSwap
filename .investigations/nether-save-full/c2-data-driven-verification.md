# C2 预加载表数据驱动化 — 验证记录（candidate，2026-09-07）

## 改动
- `WorldgenRust/src/surface_rules.rs`：新增 `collect_noise_keys()`——遍历 surface_rule JSON（sequence/if_true/then_run/invert 递归），收集全部 noise_threshold 引用的 noise key（去重）。
- `WorldgenRust/src/worldgen_handle.rs` step4：预加载 key 分两路——
  - 基础 3 key（surface / surface_secondary / clay_bands_offset，SurfaceBuilder 引擎无条件用）；
  - overworld：保留静态清单（代码规则无 JSON 数据源，8 key）；非 overworld：从 `settings.surface_rule` JSON 构建期动态收集（judge C2 CONCERN 闭环）。
  - 静态 nether 6 key 清单删除（由 JSON 收集取代）。

## 静态验证
- nether.json surface_rule 中 noise_threshold 引用恰好 6 key = 旧硬编码 6 key（patch / nether_state_selector / netherrack / nether_wart / soul_sand_layer / gravel_layer）——收集覆盖完备（dump 对照 `.tmp/nether-surface-rule.txt` + judge 上轮独立反查）。
- cargo check / cargo build --release 绿（无新警告）。

## 运行时回归（存档口径，seed B=8576294172403134396，4×4 @3200,3208，FULL 参照 hash=1DDE3B09）
| run | match | nonAir | initNether | SURFACE-WARN |
|-----|-------|--------|-----------|--------------|
| 1 | 984600/1048576 = **93.8988%** | 424900/488508 | enabled=true ✓ | 0 ✓ |
| 2 | 984600/1048576 = **93.8988%** | 同上 | enabled=true ✓ | 0 ✓ |
| 3 | 984600/1048576 = **93.8988%** | 同上 | enabled=true ✓ | 0 ✓ |

- 3 次 run 逐位同值（本轮零非确定性波动）；93.8988% ∈ 上轮修复后采样集 {93.8988, 93.6767, 93.6765}，与修复前区间 [93.5156, 93.5508] 不重叠——**C2 行为与硬编码清单一致，无回归**。
- 判据遵从 docs/09「区间不重叠 + ≥3 采样」。
- log：`cmd-output/c2-regression-run{1,2,3}.log`。

## 环境新坑（本次踩坑）
- 沙箱下 gradle runServer 崩溃：`failed to extract worldgen.dll` → `AccessDeniedException` 写 `%TEMP%\dsh-*\coreswap-native`（DSH 临时区不可写）。
- 修法：`$env:JAVA_TOOL_OPTIONS='-Djava.io.tmpdir=E:\PYTHON\CoreSwap\.tmp\java-tmp'`（tmpdir 指向工作区）。
- 另：`WorldgenRust.dll` 的 mtime 因 `fs::copy` 保留时间戳不可信（显示 9/1 实为最新），验证 dll 新旧用二进制字符串探测（如 C1 的 warn 文案）。

## 状态
- candidate（行为一致性 + 静态覆盖完备 + 运行时判据全过）；confirmed 待用户。
