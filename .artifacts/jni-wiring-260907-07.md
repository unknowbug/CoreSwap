# mod 侧 JNI 方块注册接线（方案 C 冒烟）交付 — 260907-07

状态: **candidate**（待 judge + 用户拍板）· 验证分层: Full（runServer 端到端实测）
日期锚: 2026-09-07（Get-Date 18:59 开工）· HEAD 基线: c9dffbb

## 范围
方案 C（用户 260907-07 拍板）：只通注册链路 + 冒烟验证；**id 空间错位的写回修复单独开卡，本步未实施**。

## 改动清单
1. `versions/1.20.1/rust/src/jni_bridge.rs`：新增 `Java_wg_CppWorldgen_registerBlock`（调 wg_register_block C ABI；handle==0 → -1；name 内嵌 NUL → JavaException；judge N6 注：JNI 层 name null 走 get_string 抛异常路径，api 层 null→-1 契约在 JNI 侧不可达）。
2. `runtime/1.20.1/java/src/main/java/wg/CppWorldgen.java`：native 声明 `registerBlock(long, String)`（注释声明 Rust 动态 id ≠ Java raw id）。
3. `runtime/1.20.1/java/src/main/java/wg/bench/CppBridge.java`：`registerModBlocks()`——`-Dcpp.blockRegister` 门控（**默认关，对已 confirmed 对齐态零扰动**）；遍历 Registries.BLOCK 非 minecraft 方块注册到三句柄并逐条打印 java_raw/rust_id；无 mod 方块时合成名回退（testmod:probe_block_N，冒烟专用）。
4. `runtime/1.20.1/java/src/main/java/wg/bench/BenchMod.java`：initEnd 后调用（创建期注册时序约定）。
5. `runtime/1.20.1/java/build.gradle`：`-PcppBlockRegister` → `-Dcpp.blockRegister` 映射行（发现 #8/#25 家族：逐行显式映射）。

## 回滚残留面修复（260907-06 事故影响面，用户本轮点名）
- `gradle.properties`：`org.gradle.java.home` 指向已删除 `E:/python/MC/tools/jdk17` → 改 `D:/Program Files/Java/jdk-17.0.12`（构建曾因此直接失败——残留影响面实锤）。
- `run_rust_client.ps1`：2 处 JAVA_HOME 同源 MC 路径 → 同上。
- `build.gradle`：2 处 hs_err ErrorFile MC 路径 → CoreSwap run 目录。
- `JniProbe.java` / `ReadWorldProbe.java`：bench.worldgen / bench.out 默认值 MC 路径 → CoreSwap 数据目录。
- dll sha 基线切换：新基线 = 接线后 **C7AD570043135407…**（2170368B）；接线前回滚重建基线 DC8FF3A2E06DBE2D…（与 NEXT_SESSION 记录逐位一致，交叉验证通过）；D9085130 永久弃用。

## 验证记录（§9.7 声明：载体 = runServer 实机 + cargo test；覆盖 = JNI 注册链路三维度；与存档级对齐口径不可比）
- cargo test workspace 全量：6/6 passed，0 failed。
- dumpbin 符号：worldgen.dll 含 registerBlock JNI 符号（二进制检索确认）。
- gradle compileJava：BUILD SUCCESSFUL（JDK17 修复后）。
- runServer 冒烟（seed 8576294172403134396，cppReplace=1）：
  - `testmod:probe_block_1/2/3` → rust_id **1003/1004/1005**（overworld/nether/end 三句柄一致，幂等重入返回同 id）。
  - Rust stderr `[BLOCKS] registered` 与 Java stdout `[BLOCKS-REG]` 双侧日志齐备。
  - **id 空间错位数据层证据成立**：Rust 动态 id 从 blocks.json max(1002)+1 起；Java 注册表 mod raw id 从 vanilla 全表后起；合成名 java_raw=N/A。CppBridge:421 `Registries.BLOCK.get(id)` 写回对 mod 方块**无对齐保证**（隐式同空间假设不成立；judge N3：两空间起点接近，不排除偶合碰撞，非「必错」）。
- 命令输出补落盘（judge N7）：`.tmp/verification-260907-07.log`（cargo test 6/6 + compileJava BUILD SUCCESSFUL + dll 符号检索 + sha 复算）。
- 日志：`.tmp/blockreg-smoke-260907-07.log` / `.tmp/blockreg-smoke2-260907-07.log`。

## 已知边界 / idk
- @anchor.idk: SERVER_STARTED 与 spawn chunk prepare 的先后时序未运行时验证（本冒烟注册发生在 Done 之后仍成功，未触发生成线程竞争——但非时序证明）。
- @anchor.idk: 真实内容 mod 方块（非合成名）路径未实测——本环境无内容 mod，真实 mod 数据命名空间解析（缺口 3）与同名 biome override（缺口 4）仍未实测。
- @anchor.idk: `minecraft:empty` 未注册名告警在 init 期出现（vanilla 注册表含此名）——既有行为，与本次接线无关，未查。
- 写回错位修复（候选 B：wg_register_block_id 显式 id 映射）单独开卡，动 confirmed api.rs 需 judge + 用户拍板。

## 预置介入点核对
- scout ✅（jni-wiring-scout-260907-07.md，路径全限 CoreSwap 内）· judge：本交付 MUST · knowledge：subagent 草稿待产出 · fan-out：未触发（单假设 + 设计分叉走人工 HOOK）
