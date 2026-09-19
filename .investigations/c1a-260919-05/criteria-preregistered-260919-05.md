---
id: c1a:criteria-260919-05
block: 260919-05
status: draft          # 预登记判据（#112：与采集同批，读法写死，事后不得挑读法）
form: C-1a 形态 S1 = 9×3×3 窗最小切入（用户 260919-05 拍板；全域 fill 一趟共享 + 9 中心 3×3 窗 BFS+export）
upstream: .investigations/c1-pre-260919-04/design-c1-260919-04.md §4 判据草案（本档按 S1 形态裁剪）
---

# C-1a 实现轮预登记判据（260919-05）

## 判据（读法预写死；凡未列出的读法一律不可采用）

| # | 判据 | 判定（写死） | preconditions（§15.1：key / expected / check） |
|---|---|---|---|
| J1 | 位等价门 | `light_compute_domain_bitwise_equivalence` 双 LCG 种子 PASS（域批共享 fill 路 vs per-chunk 路 9 中心逐位等价）= 满足；任一 assert FAIL = 不满足 | key=cargo test 退出码 / expected=0 / check=本块运行日志 |
| J2 | 全量单测 | `cargo test --offline -p WorldgenRust --lib --release` 16/16 PASS | key=退出码 / expected=0 / check=同上 |
| J3 | 行为门 | domain-on 臂 ratioP50 ≥ 0.95 × 同轮 domain-off 臂 | key=run_phase json ratio 字段 / expected=≥0.95 / check=采集落盘 json |
| J4 | 性能主判据 | ① fill 段（fill_us P50，全域共享一趟口径）**与 off 臂 9 中心 fill 求和口径不可直接比**——主判据改用：nativeMs P50(on) ≤ 0.90 × nativeMs P50(off)；② 旁证：subcopy_us P50 < off 臂 subcopy_us P50（b9 拷贝消失面） | key=run_phase json nativeMs/subcopy P50 / expected=≤0.90× / 残缺率≤5%（#190）双档门 |
| J5 | G3 收敛性门 | 实现后重载 settle 复测 changed ≈ 噪声地板（verdict-260919-02 同口径：changed 占比 ≤ ~0.5% 且不逐轮上升） | key=重载轮 changed 计数 / expected≈地板 / check=采集日志；同轮同 dll / seed 双证（#161 world 身份门 + #156 全使能） |
| J6 | RSS 登记（S4 降登记级） | 实测同臂进程 RSS 记录在案，无异常（>2× off 臂）即转预登记 idk 关闭 | key=RSS 采样 / expected=<2× off / check=采样记录 + 口径声明（#149：固定 -Xmx） |

## 灰区归属（预写死，#154）

- nativeMs 比值落 (0.90, 0.95] = **灰区**：报「方向为正未达判据」，不得判 PASS；落 ≥0.95 = FAIL。
- 残缺率 >5% 的臂 = VOID（#190），复采换标签（#144-146：先归档旧日志）。

## 执行体三元组（#96）

- 判定臂 dll = `target/release/worldgen.dll` sha256 `CC4E39FE850B11003715EA849E23A6EE37036E58A80DFD68DD8C5CB9BE9154AE`（260919-05 本块构建，含 C-1a）；实际加载文件 sha in-log 核对（#96 dev-run 形态 = build/resources/main/native/worldgen.dll）。
