# 每 chunk 并发下慢 7.5 倍——MTTRACE 铁证（2026-08-16 复测）

> 状态：draft（主会话临时排查记录）| 结论性 docs 落盘待 subagent
> 上一步结论修正：notify bug 修复消除串行假象，但「每 chunk 并发下慢」是**独立且真实**的问题

## 关键发现（MTTRACE 复测，`bench-A-recheck-8x8-20260816.txt` + WG_MTTRACE 4x4 16chunks T=8）

### 1. notify 修复已验证：8 worker 真并行（无串行假象）
WG_MTTRACE 第一批 8 chunk：
```
[MT] chunk(-2,-2) by=576 enter=23134 exit=23604 dur=470
[MT] chunk(0,-2) by=7944 enter=23134 exit=23605 dur=471
[MT] chunk(-1,-2) by=48904 enter=23134 exit=23605 dur=471
[MT] chunk(1,-2) by=47168 enter=23134 exit=23605 dur=471
...（8 个 chunk 全部 enter=23134，exit 相差仅 6ms）
```
→ **8 worker 同时进入、几乎同时退出 = 真并行**（done_by 分散 12 线程，WG_TASKTIME 交叉印证）。

### 2. 但每 chunk 并发下慢 7.5 倍（真实，非 fprintf 伪影）
- T=1（单线程，64-chunk）：每 chunk **62.89ms**
- T=8（并发）：每 chunk dur **~470ms**（第一批 8 个 dur=470-477）
- **ratio ≈ 7.5×**——与 scout-map L123「并发下每 chunk dur 530ms vs 单线程 71ms（7.5 倍）」一致
- **排除 fprintf 锁竞争伪影**：8 个 chunk 同时 enter（23134）+ 几乎同时 exit（23605-23611），说明 exit 时间戳未被 fprintf 锁拉偏（若 fprintf 竞争强会错开 exit）——dur 值真实反映墙钟

### 3. 修正上一轮结论
- 上一轮「bench-notifyfix-8x8：T=1 98.02 / T=8 89.88 不再反降」**基于 T=1=98.02 异常高基线**得出，**作废**（T=1 应为 ~62.89ms；98.02 是环境噪声）
- **修复后真实形态**：T=1 62.89 / T=8 75.13 / T=12 84.61 / T=22 83.11——**仍反降 +19~34%**
- **notify bug 修复（消除串行假象）≠ 解决反降**：反降主因是「每 chunk 并发下慢」（共享资源竞争，C7 带宽已排除 1-2%，剩共享内存延迟 QoS / 缓存 / TLB / 锁）

## 遗留方向（第二阶段课题，未定位根因）

「每 chunk 并发下慢 7.5×」根因待定位（之前 C1-C7 排查在 notify bug 串行假象下进行，需在「真并行」状态下重审）：
- C2 睿频/SMT（频率归一化后仍反降 +14~25% → 已排除为主因）
- C3 LLC 容量（8T 活动集 10.4MB < 16.5MB LLC → 对 8T 已排除）
- C7 带宽（540MB/s = DDR 1-2% → 已排除为主因）
- **剩余真凶方向**：共享内存延迟 QoS（并发依赖链 miss 延迟放大）+ 未排查的每 chunk 内部竞争（thread_local 缓存行共享 / 大表随机访问跨核 L3 miss / spline 表驻留热点）

## 结论

> notify bug 是真 bug（串行假象），但**不是反降主因**。「每 chunk 并发下慢 7.5×」独立且真实（MTTRACE 铁证），是反降的真正来源——需在真并行状态下重审 C1-C10 候选（之前都受串行假象干扰）。
