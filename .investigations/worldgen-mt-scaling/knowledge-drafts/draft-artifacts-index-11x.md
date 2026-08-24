# 草稿 · .artifacts/index.yaml 追加条目 —— 11× 争用归因（latency QoS）

> **用途**：把以下条目追加到 `.artifacts/index.yaml` 的 `entries:` 列表末尾（主会话应用）。
> **状态约定**：排除链为 production 模型实测确证级，但按 RE-Framework 置信度状态机**一律标 `candidate`**（`confirmed` 只由人类拍板授予）。latency QoS 归因本身是**推断（@anchor.idk）**，candidate。
> **schema**：对齐 `.artifacts/index.yaml`（`- id / path / kind / status`）。

```yaml
  - id: 're-code:worldgen-mt-scaling:11x-contention-log'
    path: '../.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md'
    kind: analysis
    status: candidate
    # 结论摘要：production density 并发 11× 争用完整排查过程日志（DFC 失败定论→SERIAL/NOSPLIT/DEVIRT/spline-only/warm/WG_FLAT_TOP 排除链→interp/noodle 采样内部→latency QoS 归因→M3 探针 bug）。
    # 主会话备注：本条目为逐项排除链 + 探针 bug 记录（过程/数据/教训），非单一结论；结论性归因见下方 latency-qos 条目。

  - id: 're-code:worldgen-mt-scaling:latency-qos-attribution'
    path: '../.investigations/worldgen-mt-scaling/interp-memory-access.md'
    kind: analysis
    status: candidate
    # 结论摘要：11× 争用归因 = 长串行依赖链 + 内存子系统 latency QoS（每级 load 结果喂下一级，8 线程灌入共享内存子系统排队 → 每级延迟非线性膨胀）。
    # 排除链（production 模型确证级，同探针 conc_density_probe）：存储(SERIAL 10.25×)/递归(NOSPLIT 9.9×)/locFn 虚分派(DEVIRT 10.05×)/buildGrid(warm 10.10×)/顶层 min/squeeze/mul 虚分派(WG_FLAT_TOP 10.55×)/spline(spline-only 1.62×) 均非 11×；非带宽(C7 DDR 1-2%)/非 SMT(T=8≤12 物理核)/非写乒乓(全只读)。
    # 置信度：排除链 = candidate（production 模型实测，等人类确认）；latency QoS 归因 = candidate/推断（@anchor.idk，需 M3 干净验证）。

  - id: 're-code:worldgen-mt-scaling:wg-flat-top-bitwise'
    path: '../.investigations/worldgen-mt-scaling/wrapper-chain-measurement.md#8'
    kind: evidence
    status: candidate
    # 结论摘要：WG_FLAT_TOP（去 min/squeeze/mul 虚分派 4→2）与生产逐位一致（block_probe -save out_prod.bin vs out_flat.bin SHA256 identical=True）。
    # 性能结果：10.55× ≈ 生产 10.32× → 虚分派数无碍（负面结论，排除项）。逐位一致保证此负面结论可信（改生产路径后 block_probe 对拍纪律）。

  - id: 're-code:worldgen-mt-scaling:m3-probe-bug'
    path: '../.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md#15'
    kind: errors
    status: candidate
    # 结论摘要：M3 interp-only 探针（wg_sample_interp）自身 bug —— hit 慢 850× vs production（291μs vs 0.34μs/点）；单点即触发 buildGrid 怪物树 27.9ms。
    # 候选根因：thread_local slots 每采样 resize/allocator / 坐标跨 256 cell / g_curChunk 引入路径。需 perf（VTune/ETW）定位；非 11× 机制。latency QoS 归因因此未直接验证。

  - id: 're-code:worldgen-mt-scaling:11x-errors-mt12-mt17'
    path: '../.investigations/worldgen-mt-scaling/knowledge-drafts/draft-mt-errors-11x.md'
    kind: errors
    status: candidate
    # 结论摘要：本 session 新增 6 条错误（MT12-MT17），五段式完整记录：① SERIAL static_cast<const DensityFunction&> 强制虚调用（SERIAL 未去虚分派）
    # ② 探针 scattered 坐标失真（grid 重建主导）③ conc_sample_probe(std::thread) vs conc_density_probe(wg_worker pool) 线程模型混淆
    # ④ scout 静态误判 buildGrid 深链主导（warm 实测推翻）⑤ wg_sample_density 单点无 grid 缓存（std::thread 超时）⑥ 改生产路径后须 block_probe 对拍。
    # 主会话备注：应用本条目时同步把 ①-⑥ 追加到 mt-scaling-errors.md（MT12-MT17）+ 速查表。

  - id: 're-code:worldgen-mt-scaling:11x-timewise-20260823-24'
    path: '../.investigations/worldgen-mt-scaling/knowledge-drafts/draft-10-timewise-20260823-24-11x.md'
    kind: knowledge
    status: candidate
    # 结论摘要：2026-08-23/24 的 11× 争用定位完整过程时间线草稿（DFC 失败定论→locFn A/B→递归→虚分派→wrapper 链隔离→warm→WG_FLAT_TOP→latency QoS 收敛→M3 探针 bug），
    # 每步含为什么+数据+教训，状态标注 ✅/❌/🔍。主会话应用后并入 versions/1.20.1/docs/10-timewise-archive.md。
```

> **主会话应用注意**：
> 1. 这些条目追加到 `.artifacts/index.yaml` `entries:` 末尾；`status` 统一 `candidate`（等人类 confirmed；latency QoS 归因需 M3 验证后再升）。
> 2. `path` 相对 `.artifacts/`（照现有约定，`../.investigations/...`）。
> 3. 应用后跑 `ref_merge_index`（若用 merge 机制）或直接按 schema 追加；确认无重复 id。
> 4. 错误台账条目（11x-errors-mt12-mt17）应用时同时合并到 `mt-scaling-errors.md`。
