# panic-505 错误台账（260903-14）

## E1：gradle runServer 崩溃 `worldgen-data not found in mod resources`

- **现象**：不带 `-PcppWorldgenDir` 跑 `-PcppReplace=true -PreadWorldProbe=true`，server started 时抛 `IllegalStateException: worldgen-data not found in mod resources`（CoreSwapFixHelper.extractWorldgenDir:48），服务器立即停止。
- **根因**：jar 内资源布局为 `worldgen-data/{minecraft, blocks.json, ...}`（minecraft 直下），而 marker 检查路径是 `wgDir/data/minecraft/worldgen/noise_settings/overworld.json`（期望多一层 `data/`）——解压后 marker 永远不存在 → 二次抛异常。两条路径布局约定不同步。
- **定位**：读 CoreSwapFixHelper.java marker 路径 + `Get-ChildItem src/main/resources/worldgen-data` 对照布局；再查历史 run 日志（c4-overworld-mask011.log）确认历史 run 全部显式传 `-PcppWorldgenDir=E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen` 绕过解压。
- **修复**：run 命令补 `-PcppWorldgenDir=...`（workaround）；资源解压路径与 marker 的不一致未改（死路，列为升级点）。
- **教训**：跑存档口径 run 照抄历史 run 的完整参数清单（含 cppWorldgenDir），不要凭 build.gradle 属性列表自行裁剪——裁掉的可能是历史踩坑后的必带项。

## E2：`rustc --extern WorldgenRust=target/release/WorldgenRust.dll` 报 E0786（no .rustc section）

- **现象**：单编 bin-diag 探针时报 `found invalid metadata files for crate WorldgenRust`。
- **根因**：`--extern` 指到 cdylib（.dll）——cdylib 无 rustc metadata；单编探针必须链 rlib。
- **定位**：错误信息 + `Get-ChildItem target/release -Filter *WorldgenRust*` 确认 `libWorldgenRust.rlib` 存在。
- **修复**：`--extern WorldgenRust=target/release/libWorldgenRust.rlib`。
- **教训**：rustc 单编外部 crate 一律用 rlib；dll 只在运行时链接。

## E3：Tee-Object 目标目录未建 → sweep 首跑输出丢失

- **现象**：后台作业「成功」但日志文件 0 字节、路径不存在。
- **根因**：`New-Item` 建目录晚于作业启动，Tee-Object 静默失败。
- **定位**：作业完成后 Get-Content 报路径不存在。
- **修复**：先建目录再重跑。
- **教训**：后台作业的输出重定向目标必须先于作业存在；Tee-Object 失败不影响退出码，不能靠 exit code 判断日志落盘成功。

## 速查表（错误→根因）

| 错误 | 根因一句话 |
|---|---|
| E1 worldgen-data not found | 资源布局与 marker 路径差一层 `data/`；用 `-PcppWorldgenDir` 绕过 |
| E2 E0786 invalid metadata | --extern 误指 cdylib，应指 rlib |
| E3 日志 0 字节 | Tee 目标目录后建；Tee 失败不影响 exit code |
