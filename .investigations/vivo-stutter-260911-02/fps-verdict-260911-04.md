# FPS 数据判读与缺省值拍板记录（260911-04）

## 输入

- **用户拍板（2026-09-11 18:38 前后，对话直接确认）**：
  - 实机验证已完成：「EXEC 模式完美解决问题」。
  - 线程设置确认：**维持既有设定「物理核 − 2」为最佳，不再测试**。
  - 决策：exec 模式**转正**（缺省开），直接发 release 工单。

## 判读

- 池宽语义核对：现实现缺省 = `logical/2 − 2`，javadoc 明示与「物理核 − 2」同源（SMT2 下等价，引擎 `adaptive_threads` 同口径）——**用户拍板的设定即现有缺省，池宽零代码改动**。
- maxinflight：exec 转正后 P1 信号量保留为回退路径（`-Dcoreswap.exec=0`），`maxinflight` 缺省值决策随 exec 转正自然关闭（仅回退路径使用，维持 `logical/2−2` 同源缺省）。
- judge 条件遗留核销：
  - C1（wall 差单 run 趋势）：用户实机验证已接受效果，wall 定量裁决不再需要。
  - C3（出单前声明：低核数机器池宽、多维度共用单池、maxWs +2.1GB）：转入工单 §5 已知边界声明。
  - perfprofile 临时件：本块移除（`cf9fa54`），重编 final jar 后重算 sha（非转录）。
  - C4（分派优先级 javadoc）：已在 260911-03 应用。

## 产物（final jar，1.0.29 新版本号——用户拍板）

- 源 commit：`cf9fa54`（perfprofile 移除 + version bump；exec 实现链 `0ecac5c`→`d20aac1`，Rust 零改动）
- jar：`versions/1.20.1/java/build/libs/coreswap-1.20.1-1.0.29.jar`（重编 2026-09-11 18:55，`--rerun-tasks` 全量）
- **三元组 MATCH**：jar sha256 = `b057fda216012647a0e0ea6bbe0d1b4967a5458950ddc5f208dc85f48c312239`；jar 内 `native/worldgen.dll` = `dd3b645f2c79d2cb54619e0ecb95f9b913b3fe02f30d74eba9ea5ee92886765d` = `target/release/worldgen.dll`（12:53 刷，当日核）
- 时间先后关系（#121）：源 commit `cf9fa54` → 重编 18:40/18:55 > A/B 窗口（260911-03 同日 12:xx-13:xx）> 引擎 dll 12:53；sha 三元组重算于重编后。

## 状态

- 判读 + 拍板建议：candidate（依据 = 用户实机确认 + 260911-03 三臂 A/B + 行为门 4140 指纹 diff=0）
- judge：MUST（缺省值拍板 = 重大方向 + 工单出单）——见 `.artifacts/releases/judge-verdict-260911-04.md`
- confirmed：**用户已拍板**（「EXEC 模式完美解决问题可以发 release 了」）——AI 侧仅记录，不授予标签
