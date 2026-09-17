# K2 噪声锚课题错误台账（260917-03）

格式：五段式（现象/根因/定位/修复/教训），错误→根因速查表见末尾。

## E1 cmd-output 目录未预建 → 脚本 FileNotFoundError

- **现象**：`run_k2.py` 首跑 `io.open(logfile,"w")` 抛 `FileNotFoundError`，链 SKIP。
- **根因**：新建课题目录只建了根目录，`cmd-output/` 未建即写（§八.5「Move-Item 目标父目录不存在静默失败」同族——写侧形态）。
- **定位**：traceback 直接指向 open 行。
- **修复**：`New-Item -ItemType Directory -Force ...cmd-output`。
- **教训**：新课题脚本落盘路径的父目录随脚本创建（或脚本内 makedirs）。

## E2（高价值）DSH 沙箱 TEMP 重定向 → lightInit threw → 整臂静默 vanilla fallback（K2-D4 首轮 VOID）

- **现象**：K2-D4 SELFCERT `lightInit_ok=0, fallback=1, hook=0, sha=none`，GATE VOID；changed=81/2025=4%（实为 vanilla 光照形态跑了一轮）。服务端随后 `Encountered an unexpected exception` 崩溃。
- **根因**：DSH 沙箱把 TEMP/TMP 重定向到宿主 `C:\Users\NDark\AppData\Local\Temp\dsh-jks3Sq`，JVM 内 `CoreSwapFixHelper.extractWorldgenDir` 建目录 `AccessDeniedException` → `ExceptionInInitializerError` → `wgLightEnsureInit` 捕获 → `lightInitFailed` 永久 fallback vanilla。同轮 JNA `jnidispatch.dll` 提取同样被拒。**接管失败是静默的（只有 x1 fallback 行 + 崩溃栈），不细看自证行就会拿 vanilla 形态数据当 domain 噪声锚**。
- **定位**：grep 日志 `LightRust|fallback` → 行 222 `[LightRust] lightInit threw` → 堆栈 `AccessDeniedException: ...Temp\dsh-jks3Sq\coreswap-data`。#118 SELFCERT 硬门在采集完成时即打 VOID，未外泄无效结论（硬门按设计工作）。
- **修复**：脚本内固化 `TEMP/TMP` + `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=` 指向工作区 `.tmp/k2-260917-03/jtmp`，并先 `gradle --stop`（防旧 daemon 复用吞 env，#32 家族）。修复后三连 boot 全 PASS。
- **教训**：① #42 JNA tmpdir 家族新形态——**沙箱会话跑 runServer 必须固化 tmpdir 到工作区**，判别签名 = `AccessDeniedException` 指向 `Temp\dsh-*` 目录；② 自证硬门必须包含「接管形态」证据（lightInit/hook/sha 三件），单看 changed 数会误读（4% 恰在 G17b legacy 量级，极易被当有效臂）；③ daemon env 残留与客户端 env 修正必须成对处理。

## E3 job kill 中断 boot2 → 半截日志（K2-D5-VOID2-partial）

- **现象**：kill pwsh-5 时 boot2 正在跑，留下无 SELFCERT 行的不完整日志。
- **根因**：boot1 VOID 后未及时 kill（发现延迟），链第二 boot 已启动。
- **定位**：日志无 `[SELFCERT]`/`[GATE]` 行 + java 进程清单核对。
- **修复**：日志归档改名 `K2-D5-VOID2-partial.log`，world 无 lock 残留后重跑。
- **教训**：VOID 裁决要在下一 boot 启动前做出（脚本内 GATE 不过即 sys.exit 非零，链即断——本次脚本 GATE 只打印不退出，是 E3 的前置缺陷）。

## 错误→根因速查表

| 错误 | 一句话根因 |
|---|---|
| E1 | 父目录未建即写 |
| E2 | 沙箱 TEMP 重定向 → 资源提取拒访 → 接管静默 fallback（tmpdir 未固化） |
| E3 | VOID 裁决不中断链 + kill 留半截日志 |
