# 260908-11 错误台账（F2/F1/Chunky 块）

## E1 对拍 89% 假警报——用了 data 目录里的伪参照（测量侧错误，非回归）

- **现象**：1.21.6 P7 R 臂复跑后与 `versions/1.21.6/data/vanilla_-8248318472910187742_8_200_200.blocks` 对拍 = 89.08%、biome 14160/16384 差（ocean↔deep_ocean），疑似大回归。
- **根因**：该 data 目录参照正是 NEXT_SESSION 260908-10 已警告的伪参照（17:44 导出时 level-seed 未设 → worldSeed=-5307 污染副产品，文件名内嵌 bench seed 自证不成立）；真参照在 `.tmp/p7-260908-10/vanilla/`（17:48 双臂确认用）。
- **定位**：先查接管证据（807 intercepted ✓）→ 查 dll/工作区 diff（仅 F1 一分支）→ 用旧 R 臂交叉 diff：新 R vs 旧 R = 100.0000%，旧 R vs 伪参照 = 同一 89.08% → 测量侧定责。
- **修复**：改用 `.tmp/p7-260908-10/vanilla/` 真参照 → 99.9993%/41/biome 0，精确复现 confirmed 结论。
- **教训**：伪参照会跨块存活在「权威数据目录」里（路径权威 ≠ 内容权威）；凡对拍结果异常，第一动作 = 核对参照文件谱系（#14/#18 家族：文件名/在盘 ≠ seed/内容自证）；已 confirmed 的结论数字（99.9993%）是现成的复现锚。

## E2 `gradle runServer --nogui` 非该 task 选项（1.21.6 探针工程）

- **现象**：BUILD FAILED `Unknown command-line option '--nogui'`。
- **根因**：--nogui 非通用 CLI 选项（knowledge build-tooling #9 已有同款）；1.21.6 探针 task 未定义该 flag。
- **修复**：去掉 --nogui（BenchMod 探针自带 stop）。
- **教训**：跨版本复制命令行时逐项核对 task 定义，不凭肌肉记忆。

## E3 复用已生成世界 → 探针零命中

- **现象**：air 点集 aqdump 复跑（未删 world）：`[AQDUMP] done` 但 0 行输出、输出文件未创建。
- **根因**：aqdump 采样发生在 chunk 生成期；已生成 chunk 不会重新生成 → 零命中。
- **修复**：每次 aqdump 前删 `run\world`（同 seed 三查配套动作）。
- **教训**：生成期探针 = 一次性窗口；「在盘」≠「会重新经过探针」。

## E4 Rust 臂 aqdump 缺 `-PaqDump` 驱动 → 无 pregen/无 stop

- **现象**：`-PaqDumpRust` env 通道已配（dbg 行确认）但无 AQDUMP 输出、服务器不自动停。
- **根因**：Rust 侧 dump 只打命中点；点区域 chunk 无人加载（spawn pregen 覆盖不到时），且 BenchMod 驱动（forceload + stop）由 `-PaqDump` sysprop 触发——rust 臂也要带 `-PaqDump`。
- **修复**：`-PaqDump=true -PaqDumpPoints=... -PaqDumpOut=...` + `-PaqDumpRust=...` 同跑。
- **教训**：驱动开关与数据通道是两个开关（#20 死参数家族变体）；「env 生效（dbg 行）」≠「行为链闭合」。

## E5 gradle property → env 通道首跑未生效（无 dbg 行不可见）

- **现象**：1.20.1 build.gradle 新增 aqDumpRust env 通道，首跑无 `[AQDUMP] enabled`、无 `AQDUMP seed`。
- **根因**：首轮无 println dbg 无法区分「property 未映射 / env 未传 / 点未命中」（#32 daemon env 卫生家族）；补 dbg 行后二跑确认通道通。
- **修复**：+dbg println（对齐 1.21.6 侧已有写法）。
- **教训**：新接线通道首跑必须带行为化 dbg（#81：A=B 恒等不证通道生效）。

## E6 Chunky 臂核验查早了——bridge init 行在 Done 之后才打（#80 时序家族）

- **现象**：coreswap 臂 `Done (4.952s)` 后立即查 `[CppBridge] init` → 空 → FAIL 退出（且 java 未被杀）。
- **根因**：BenchMod/CppBridge 初始化行为惰性、在 Done 后若干秒才打印（#80：init 行时间戳不作接管生效判据家族——此处是反向：核验必须在 init 行出现后）。
- **修复**：bridge 核验改 30s 轮询等待 init 行；FAIL 分支补杀 java。
- **教训**：核验时机必须跟随被核验行的产生时机，逐秒日志轮询优于即时断言。

## E7 JNA 临时文件「拒绝访问」瞬时错误

- **现象**：一次 runServer 启动报 `UnsatisfiedLinkError: Failed to create temporary file for jnidispatch.dll: 拒绝访问`（重试即恢复）。
- **根因**：沙箱下 java.io.tmpdir 提取临时文件偶发被拒（#38 平台分层家族环境面）。
- **修复**：重跑；Chunky 驱动统一 `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=.tmp\java-tmp`（1.20.1 驱动已有先例）。
- **教训**：瞬时环境错误重跑优先于立案；驱动脚本固化 tmpdir 规避。

## 速查表

| 错误 | 一句话根因 |
|---|---|
| E1 89% 假警报 | 伪参照在权威目录里（谱系核对缺失） |
| E2 --nogui | 跨版本命令肌肉记忆 |
| E3 零命中 | 复用已生成世界 = 生成期探针窗口已关 |
| E4 rust 臂无输出 | 驱动开关与数据通道分离 |
| E5 通道静默 | 新通道无行为化 dbg |
| E6 FAIL 误报 | 核验早于 init 行产生 |
| E7 JNA 拒访 | 沙箱 tmpdir 瞬时（tmpdir 固化规避） |
