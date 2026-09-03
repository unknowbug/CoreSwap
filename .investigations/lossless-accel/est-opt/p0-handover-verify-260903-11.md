# P0 交接验证记录（260903-11 · est 查表化优化包）

- 动作：复跑 bin-diag/qaq1_surf_probe.rs（新鲜进程，rustc 单编 .tmp/qaq1_surf_probe_p0.exe），seed 8576294172403134396 / region (200,200) / 16 chunks，与 260903-10 同口径（§9.7：载体/覆盖面/历史口径同源，可比）。
- 结果：`[timing] median=72.84ms`；`[surf] calls/chunk=214 iterations/chunk=7342 avg_iter/call=34.35`；`[wl] calls/chunk=110161 miss=2782`。
- 对比 260903-10（cmd-output/qaq1-counters / qaq1-evidence-pack）：iterations 7342、avg 34.35、miss 2782 **逐项一致**；median 67.88→72.84 在运行方差内。
- 判定：Q-AQ1 est 冷扫描量级（7342 × ~2117ns ≈ 15.5ms/chunk，占 aquifer 段 68%）**廉价独立验证通过，可继承**。下一步进入 P1 语义调研（subagent 进行中）→ P2 方案分叉。
- 纪律执行：无 carver 臂涉入（探针纯 aquifer 路径）；seed 单一未混用。
