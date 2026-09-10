# vivo-verify-260910-02 — 执行记录（实际 2026-09-10 14:15-）

## Step 1 前置核验（执行体三元组，#96/#23）

- git HEAD = f2daa5c（2026-09-10 12:58）；B3a 源码 e91c7f6 = 2026-09-09 23:32。
- **陈旧警报**：target/release 两个 dll 原 mtime = 2026-09-09 22:39:45（早于 e91c7f6 23:32）→ 现役 dll 不含 B3a/CAP2048，强制 rebuild（#23 家族：mtime/Finished 均不可单独信）。
- rebuild：`cargo build --offline -p worldgen --release`（14:22:20）+ `-p worldgen1216 --release`（14:22:41，#39 双薄壳）。
- 产物指纹：
  - worldgen.dll    sha256 = D390CE604F4C3CE76CFB92EE464108680F63EB5C3FD5B2531F1BB859DC9CC781
  - worldgen1216.dll sha256 = 5E30187A5782A3B11334647C94B327AE0C8504DCDCCB10ED16B10E0C44A4FCFB
- **内容哨兵（双过）**：二进制内含 `CA-MEMO-OFF` / `CA-NT`（e91c7f6 新增 WG_CA_MEMODIAG 扩展标签）→ dll 确含 B3a 代码。

## Step 2 实机运行（judge 条件 1 补写，原始日志 = 同目录 vivo-run-260910-02.log）

- 启动：`gradle runServer -PcppReplace`（GRADLE_USER_HOME=E:\PYTHON\CoreSwap\.gradle-home），workdir runtime\1.21.6\java，全量日志 `*>` 本目录 vivo-run-260910-02.log（#41）。旧 world 先归档改名 world-prev-260910-02（fresh gen，#19/#88）。
- 时间线：起服 14:26 前后 → `Done (4.831s)` 14:26:38 → [CppBridge] init 三句柄 14:26:40（#80：init 晚于 Done，轮询判读）→ RCON forceload 25,25 与 -25,-25 两区 14:28:12/14:28:21（post-Done 验证面）→ 生成 242 chunks（populateNoise intercepted / buildSurface skipped 各 242，placedFeature skipped=0 = stageMask=3 口径自洽）→ RCON stop 14:29:14 → Stopping server ×1 → `BUILD SUCCESSFUL in 3m 28s`。
- 退出结论：**exit code 0**（后台 job pwsh-1 实测结算 `completed, exit code 0`——非由 BUILD SUCCESSFUL 推得）。
- RCON 工具：.tmp\rcon_cli.py（#29 合规 4 字节 type）。
