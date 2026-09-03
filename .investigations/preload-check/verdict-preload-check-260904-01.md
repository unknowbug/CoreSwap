# verdict-preload-check-260904-01 — 启动期 noise key 机械校验 + FixHelper 解压根治

- session: 260904-01
- 状态: **candidate**（Full 层验证完成，confirmed 待用户拍板）
- 架构计划: `.investigations/000-架构设计/架构计划-260904-01-startup-check-and-fixhelper.md`

## 变更

### ① 启动期机械校验（Rust）
- `WorldgenRust/src/surface_rules.rs`：
  - `collect_noise_keys` 泛化：任何带 "noise" 字符串字段的 JSON 节点都收（原来只认 noise_threshold 类型）；
  - 新增 `collect_rule_noise_keys`：对已构建 SurfaceRule 树机械收集运行时会查 sampler 的 key；
  - 新增 `ENGINE_NOISE_KEYS` 常量（9 key）——**紧邻 place_badlands_pillar 等引擎 get_noise 调用点就近维护**（引擎路径不在 rule 树内，机械收集覆盖不到，清单是唯一事实源）。
- `WorldgenRust/src/worldgen_handle.rs`：
  - 预加载清单改为 `use surface_rules::ENGINE_NOISE_KEYS`（单一事实源，不再两处硬编码）；
  - `create_for_dim` 规则构建后新增启动期断言：`collect_rule_noise_keys(rule)` 结果 ⊆ 预加载集合，缺失即 panic 并列出缺失 key。

### ② CoreSwapFixHelper 解压根治（Java）
- `routeRel` 路由：`data` 开头 → wgDir（原版布局）；`minecraft` 开头 → `wgDir/data/` 拼接（本仓库旧布局兼容）；其余 → target。
- **根因修正（比 #15 记载更深）**：mod 资源 worldgen-data 只有 biome/（68 文件），**根本没有 noise_settings/ 等完整数据集**——marker 结构性不可能出现，解压分支从来是死的，历史全靠 `-PcppWorldgenDir` 绕过。
- 修复：`src/main/resources/worldgen-data` 整体重排 = 权威数据 `versions/1.20.1/data/worldgen`（845 文件，自带 data/ 层）+ 顶层 4 json（blocks/biome_params/biome_params_nether/noise_params）。
- 错误信息 fail-fast 补 `-PcppWorldgenDir` 绕过提示（2 处）。

## 验证（Full 层）

| 判据 | 结果 |
|---|---|
| 四臂 hash 零语义回归 | ✅ estopt_ab（新单编 exe）四臂（env 强制 00/01/10/11）hash 全部 `f2b1a3932c6e589e` 与 260903-14 confirmed 逐臂一致（cmd-output/estopt-ab-4arms-260904-01.txt）。§9.7 口径：载体=bin-diag 单编 exe；覆盖面=四臂全跑（should-fix 清偿后）；可比性=与 260903-14 同 seed 同法同哨兵值 |
| 4096 chunk sweep 无 panic | ✅ wall=128101ms，4096/4096，16 block hits/misses 与 260903-14 基线（.investigations/panic-505/cmd-output/sweep-fixed-4096-260903-14.txt）逐项相同（cmd-output/sweep-fixed-4096-260904-01.txt） |
| ① 校验生效证明（#20 死参数判据） | ✅ 删 `minecraft:calcite` → 启动即 panic exit=101，精确报 `["minecraft:calcite"]`，panic 输出已落盘（cmd-output/negtest-calcite-260904-01.txt）；恢复后 hash 复验一致 |
| ② 解压分支闭合 | ✅ 删 %TEMP%\coreswap-data → extractWorldgenDir 成功，marker 出现，849 文件齐，blocks.json 落位 target 根 |
| dll 打包一致性 | ✅ 最终构建 jar 内 native/worldgen.dll sha256 = WorldgenRust/target/release/WorldgenRust.dll（972041E6...，neg-test 往返后重打包复验） |
| gradle build | ✅ BUILD SUCCESSFUL（coreswap-1.20.1-1.0.22.jar） |

## 过程中的错误（详见 errors.md）

- E1: 两次负向测试"未触发" = **跑的是 bin-diag 旧 exe（静态链接旧代码）**——`cargo build --release` 不编译 bin-diag；教训：验证结论先核产物新鲜度（发现 #6 内容指纹判据的应用）。
- E2: collect_rule_noise_keys Cond 臂 NoiseThreshold 命中后漏 break → 死循环（estopt_ab 挂起暴露）——负向测试立功。
- E3: ② 首次修复后 marker 仍缺 → 资源集不完整（#15 记载的根因不完整），资源整体重排解决。

## 交付物

- `runtime/1.20.1/java/build/libs/coreswap-1.20.1-1.0.22.jar`（内置完整 worldgen-data + 新 dll + FixHelper 修复）
