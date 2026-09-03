# preload-check 错误台账（260904-01）

## E1: 负向测试跑在旧 exe 上——两次"未触发 panic"均无效
- **现象**：删 `badlands_pillar_roof` 后 estopt_ab 正常出 hash 不 panic；换删 calcite 仍不 panic。
- **根因**：estopt_ab 在 `src/bin-diag/`，`cargo build --release` 不编译 bin-diag——`target/release/estopt_ab.exe` 是 2026-09-03 22:21 的旧产物（静态链接旧 lib 代码）。改源码重编 lib 对旧 exe 完全无效。
- **定位**：`Get-Item exe | LastWriteTime` 发现时间戳早于本次改动。
- **修复**：按 bin-diag 单编纪律 `rustc --edition 2024 src\bin-diag\estopt_ab.rs -O --extern WorldgenRust=target\release\libWorldgenRust.rlib -L target\release\deps -o target\release\estopt_ab.exe`。
- **教训**：**验证结论先核二进制产物新鲜度**（build-tooling #6 内容指纹判据的变体：时间戳/哈希核产物，不信"我编译过了"）；bin-diag 探针每次用前必须重单编。

## E2: collect_rule_noise_keys 死循环
- **现象**：修好 E1 后新 exe 启动挂起（>300s 无输出）。
- **根因**：Cond 臂 `loop { match c { NoiseThreshold => push（无 break/无前进）... } }`——命中 NoiseThreshold 后 c 不变、不 break，死循环。
- **定位**：读代码 + 挂起点在 create() 断言块。
- **修复**：NoiseThreshold 臂 push 后 `break`。
- **教训**：状态机循环每臂必须「前进或退出」；负向测试意外暴露此 bug——**若两次负向测试"通过"了（E1 的假阴性）此 bug 会带进生产**，假阴性比假阳性更危险。

## E3: ② 首次修复后 marker 仍缺——#15 根因记载不完整
- **现象**：路由修复（minecraft→wgDir/data）后解压成功但 `noise_settings/overworld.json` 仍不存在，`worldgen` 下只有 `biome/`。
- **根因**：mod 资源 `worldgen-data/` 只有 68 个文件（仅 biome + 顶层 json），**完整数据集（noise_settings/density_function/... 845 文件）从未打包进 jar**——解压分支结构性死路，历史全靠 `-PcppWorldgenDir` 绕过。build-tooling #15 只记了「布局不一致」，漏了「资源集不完整」这一主因。
- **定位**：解压产物逐层列目录 + 对照权威数据目录 `versions/1.20.1/data/worldgen`（845 文件）。
- **修复**：resources/worldgen-data 整体重排 = 权威 worldgen（自带 data/ 层）+ 顶层 4 json；重打包后删缓存实测 marker 出现。
- **教训**：**「绕过项永远在用」的分支 = 死分支信号**——修 root cause 前先确认分支的输入数据本身是否存在；文档记载的根因要验证到"能闭合"为止，不能到"能解释"为止。

## 错误→根因速查表
| 错误签名 | 先查 |
|---|---|
| 改了源码但探针行为不变 | bin-diag exe 是旧产物（E1）——核 LastWriteTime，单编 |
| 新启动断言/校验导致启动挂起 | 递归收集循环某臂无 break/无前进（E2） |
| 修了路由/布局仍缺文件 | 输入数据集本身不完整（E3）——对照权威源清点文件数 |
