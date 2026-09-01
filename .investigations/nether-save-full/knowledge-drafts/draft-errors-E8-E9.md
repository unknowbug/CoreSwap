# 草稿：nether-save-errors.md 追加 E8/E9（五段式）+ 速查表行

> 用法：主会话将 E8/E9 两节追加到 `.investigations/nether-save-full/nether-save-errors.md`「未闭合待查项」之前，并在文末「错误→根因 速查表」表尾追加两行。数字全部来自素材，未编造。

---

## E8. 沙箱下 gradle runServer「failed to extract worldgen.dll」AccessDeniedException（已修复）

- **编号**：E8（环境坑，2026-09-07）
- **现象**：沙箱下 `gradle runServer` 启动失败，报 `failed to extract worldgen.dll`，异常链为 `AccessDeniedException`，写入目标为 `%TEMP%\dsh-*` 临时目录。
- **根因**：**沙箱文件权限边界**——JVM/gradle 侧原生库提取流程默认写系统 `%TEMP%`，沙箱策略对该路径拒绝写入；机制上不是 dll 本身损坏或版本不符，而是「提取目标目录不可写」，报错被包装成「failed to extract」易误判为 dll 问题。
- **定位**：读异常链中 `AccessDeniedException` 的目标路径（`%TEMP%\dsh-*`），确认拒绝发生在临时目录写入而非 dll 源读取；对照沙箱可写范围（session workspace）即定位。
- **修复**：设 `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir` 指向工作区内目录，使 JVM 全部临时文件（含 dll 提取）落到沙箱可写路径，runServer 正常启动。
- **教训**：
  1. **沙箱环境下 JVM 类工具默认临时目录不可信**：任何「提取/解包到 %TEMP%」的构建/运行流程在沙箱下优先怀疑临时目录权限，用 `java.io.tmpdir`（或等价物）重定向到工作区，而不是去排查被提取的资源文件本身。
  2. **报错文本 vs 异常链**：「failed to extract X」的表面文本指向 X，机制原因常在异常链尾部的目标路径——先读 AccessDenied 的目标再定方向。

---

## E9. WorldgenRust.dll mtime 因 fs::copy 保留时间戳不可信：显示 9/1 实为最新（已修复判定方法）

- **编号**：E9（环境坑/判错方法，2026-09-07）
- **现象**：WorldgenRust.dll 文件资源管理器/Get-ChildItem 显示 mtime 为 9/1，按时间戳判断应为旧产物，实际是最新构建——按 mtime 判新旧会得出错误结论。
- **根因**：构建/部署链使用 `fs::copy`，该调用**保留源文件时间戳**——复制产物的 mtime 反映的是源文件时间而非复制时刻；mtime 在此链路上不是「产物生成时间」的可靠信号。
- **定位**：对 dll 内容做二进制字符串探测（比对链路中新特征字符串/版本串存在于「旧 mtime」文件中），确认内容为最新 → 证明 mtime 与内容不一致，时间戳不可信。
- **修复**：判 dll 新旧改用**二进制字符串探测（内容指纹）**，不再依赖 mtime。顺带处置：bin-diag 诊断 bin 不参与默认构建（cargo 只编译 `src/bin/`），使用时**临时挪入 `src/bin/` 编译**（`init_vertical` 需 `pub` 化），用完迁回，符合临时文件唯一区纪律（AGENTS.md 八.13）。
- **教训**：
  1. **`fs::copy` 保留 mtime——凡复制链路上的产物，时间戳不代表新旧**；判产物版本用内容指纹（二进制字符串/哈希），不用文件时间戳。
  2. **诊断 bin 与正式 bin 分区**：`src/bin/` 只放随库维护 bin，一次性诊断程序放 `bin-diag/`（不参与默认构建），临时挪入编译是合法用法——勿为诊断 bin 长期污染 `src/bin/` 的全量绿。

---

## 错误→根因 速查表（追加两行）

| 错误（现象签名） | 根因 | 一句话教训 |
|---|---|---|
| E8 沙箱 gradle runServer「failed to extract worldgen.dll」AccessDeniedException | JVM 默认写 `%TEMP%\dsh-*` 提取 dll，沙箱拒绝临时目录写入；非 dll 本身问题 | 沙箱下 JVM 工具用 `-Djava.io.tmpdir` 重定向临时目录到工作区；先读异常链目标路径再定方向 |
| E9 dll mtime 显示 9/1 实为最新，按时间戳判新旧出错 | 构建链 `fs::copy` 保留源时间戳，mtime ≠ 产物生成时间 | 复制链产物判新旧用内容指纹（二进制字符串探测），不用 mtime；诊断 bin 走 bin-diag/ 临时挪入 |
