# Voxy × CoreSwap 客户端兼容调试记录（260908-02/03 深夜，draft，主会话实测轮）

> 承接 compat-matrix-260908-02.md 的 🟢 初判 → 进入实机调试。三轮实机 + 日志取证。

## 架构事实（本轮确立，比矩阵初判更深一层）

- **CoreSwap 本质 = Fabric mod（loom 构建）**，Forge 侧全靠 Sinytra Connector 桥接（jar 根 = fabric.mod.json + coreswap.mixins.json；MANIFEST 全 Fabric-Loom 标记；无 mods.toml）。
- **生产服务端 mods 里的 Connector + fabric-api 不是 content-test 的依赖，是 CoreSwap 自身的运行前提**——realmod e2e（260907-09/10）一直在场所以从未暴露。
- **客户端安装因此是四件套**：Connector + fabric-api + coreswap.jar + voxy.jar。用户抱怨的「fabric 当标准亏」在这里落成实际成本。
- **FML 对「无桥接器的 Fabric jar」= 彻底静默跳过**（无 candidate 行、无 missing-mods.toml 警告、模组列表不显示）——排查时极易误判为「文件没放对」。

## 实机轮次

| 轮 | mods 配置 | 结果 |
|---|---|---|
| R1 | voxy only（ inadvertent） | Voxy 初始化成功；**Ingest service 炸 1400 次**（ArrayIndexOutOfBounds idx 126/len 64，WorldConversionFactory.convert:169，8 worker 全炸）——**纯 vanilla 地形上复现 = 移植版自身 1.20.1 bug**，与 CoreSwap 无关 |
| R2 | coreswap 无桥接器 | FML 静默跳过 coreswap（两轮日志零发现记录，文件属性/文件名/时间线全排查排除——#59 式「文件在但没被读」） |
| R3 | 四件套 | Connector 认领并重映射 coreswap（18.3s）；**Found valid mod file {coreswap}**；**mixin 激活实据**（23 mixins prepared + BlobFoliagePlacerMixin 实际注入）= **CoreSwap 客户端 env 激活首次验证**；**游戏内模组列表实证**（CoreSwap 1.20.1-1.0.27 / Mod ID coreswap / 状态 done，用户截图 d63a88bc）；Voxy Ingest 仍炸 126 次；用户观察：性能低于 vanilla、CPU 吃不上去 |

## 未决（下轮开工点）

1. **客户端 env 接管激活证据**：mod 装载+mixin 注入 ✓，但 `[CppBridge]`/`[BLOCKS-REG]`/stageMask 零输出——注意 R3 客户端**未带 `-Dcpp.blockRegister=1`**（#32 生产必带清单），且 CppBridge init 打印时机（boot vs world create）未核实。**接管是否真在 integrated server 生效仍是盲区**（#36 执行体三元组家族：客户端形态从未验证）。判据设计：加 `-Dcpp.blockRegister=1` 重启 + 对比世界生成速率 vs vanilla 单人。
2. **Voxy Ingest 1.20.1 bug**：纯 vanilla 复现 → 与我们无关实锤路线：R3 中 Rust 地形与 vanilla 地形异常模式相同（idx 126/len 64）即关闭「CoreSwap 引发」分支；上游报修或换 fork（ACowAdonis/voxy、KrzyszofWPL/voxy-forge-1.20.1 备选）。
3. **性能归因**：「比 vanilla 低」的主嫌 = Ingest 异常自旋（R1 在 vanilla 上也把 CPU 打到 40%）——修好 Ingest 前的性能观察无判别力（#34 族：谓词耦合）。
4. 知识库候选（下轮 subagent 草稿）：「Fabric mod 经 Connector 上 Forge 的客户端静默跳过签名」+「多 mod 调试先核 mod 身份再核行为」。

## 载体

- R1 log：attachments 0f6c9ccd（1.9MB）/ 123104ff（debug 2MB）；R3：d7e8780b / 7ac9bd7d
- jar：voxy-forge-0.2.18-beta-forge-all.jar（guchang233/voxy-forge-1.20.1 continuous，73MB）@ `.tmp/voxy/`
- 自建客户端骨架：`runtime/forge-client-test/`（Forge 47.4.5 离线启动链已通到 ModLauncher，assets 目录缺导致止步——`fetch_vanilla_libs.ps1` 已补 63 库；备用载具）

## 260908-04 增补：R4/R5 自建载具实机轮（draft，主会话实测，日志在 cmd-output/）

> 承接上文待办①（客户端接管激活证据）与待办②（Voxy Ingest 归因）。R4/R5 = 自建载具
> `runtime/forge-client-test/`（launch_client.ps1，Forge 47.4.5 离线链，四件套 mods：
> Connector-1.0.0-beta.49 + fabric-api-0.92.6 + coreswap-1.0.27 + voxy-forge-0.2.18-beta）。

### 事件1（假阴性教训）：INFO 级日志 grep 判「Connector 未认领 coreswap」= 双重假信号

- **现象**：首判「coreswap 未被 Connector 认领」，依据两条：① INFO 级日志 grep「Found valid mod/coreswap」零命中；② 「Dependency resolution found 1 candidates」。换 1.0.26（服务端实锤可认领）重跑仍 1 candidates，险些触发 fan-out。
- **根因**：① Connector 的认领行 `Found valid mod file coreswap-..._mapped_srg_...jar with {coreswap}` 本身是 **DEBUG 级**——用 INFO 级 grep 查一个只在 DEBUG 出现的行，假阴性是结构性的；② 「1 candidates」是依赖解析计数，与认领**无因果**（红鲱鱼）。
- **定位**：开 debug 级日志（`-Dforge.logging.console.level=debug`）后认领行立现；辅证 = `.connector` 缓存里 1.0.26/1.0.27 的 `_mapped_srg` 重映射 jar 都在（10:35/10:39 时间戳）= 装载/重映射直证。
- **修复/排除**：两版 jar（sha 不同，大小同 1431457）均被正常认领 + remap + mixin 注入（`Preparing coreswap.mixins.json (23)` + TrunkPlacerMixin 实际注入行）。「未认领」分支关闭，❌ 排除。
- **教训**：grep 日志前先核目标行的日志级别；旁证计数与结论无因果链时不得作独立判据。详见 knowledge/discovered/workflow-patterns.md 发现 #84。
- **置信度/验证分层**：candidate；本轮已验证事实。

### 事件2（待办①闭合）：integrated server 形态接管激活首次实锤（#36 执行体三元组补面）

- **现象**：主菜单阶段零 `[CppBridge]`/`[BLOCKS-REG]` 输出；用户在客户端建世界后日志命中：`[CppBridge] init seed=... enabled=true stageMask=3`、`initNether/initEnd enabled=true`、`[BLOCKS-REG] done count=0`（无 mod 内容块时 count=0 属预期）、`[BenchMod] CoreSwap replace mode: C++ worldgen active`。
- **根因（零输出的解释）**：一手源码核对 `runtime/1.20.1/java/src/main/java/wg/bench/BenchMod.java`——`CppBridge.init` 挂 `ServerLifecycleEvents.SERVER_STARTED`，注释明示 integrated server 也加载 → **主菜单零 CppBridge 输出是预期行为**。
- **结论**：客户端 integrated server 形态下 Rust worldgen 接管激活（stageMask=3 生产形态口径）首次实锤。附：`-Dcpp.blockRegister=1` 已入 launch_client.ps1（#32 必带清单客户端口径同步）。
- **置信度/验证分层**：candidate（行为化日志命中；vanilla 对照生成速率对比属待办③，暂缓）。

### 事件3（待办②分支关闭）：R5 复现 Ingest 异常模式与 R1 逐字相同 → 「CoreSwap 引发」关闭

- **现象**：R5 用户进世界移动后，8 个 Voxy worker 全炸 `Ingest service: ArrayIndexOutOfBoundsException Index 126 out of bounds for length 64, WorldConversionFactory.convert:169`——与 R1 纯 vanilla 地形异常**逐字相同**。
- **结论**：上文待办②判据命中 → 「CoreSwap 引发」分支关闭（candidate 级：移植版自身 1.20.1 bug，上游报修或换 fork，备选见上文待办②）。

### 事件4（骨架伪影）：非法 `--uuid 0` → Voxy 静默半初始化，伪装成「Ingest 不复现」

- **现象/根因**：R4 中 Voxy `async init failed`（`User.m_240411_()` NPE），LOD/Ingest 管线整体不起，一度误判「Ingest 不复现」——不是 bug 消失，是宿主管线没跑。
- **修复**：换合法格式 UUID（`12345678-abcd-3ef0-9cba-1234567890ab`）后管线起，R5 复现事件3。详见 build-tooling.md 发现 #33。

### 事件5（工具坑，简记）

Move-Item 目标父目录不存在时把源文件静默改名成目标路径（无 SilentlyContinue 也静默）——1.0.27 jar 一度「消失」成 `.tmp\coreswap-1027-client-test` 无扩展名文件。AGENTS.md 八.5 的扩展形态。详见 build-tooling.md 发现 #34 简记。

### 性能归因（待办③维持暂缓）

Ingest 异常自旋活跃（8 worker 持续炸），性能观察无判别力（#34 族），维持暂缓声明。

### 轮次表（续上文）

| 轮 | 载具 | 结果 |
|---|---|---|
| R4 | 自建 forge-client-test，四件套（uuid=0 非法） | 装载链全通但误判未认领（事件1）；Voxy init 失败伪装「Ingest 不复现」（事件4） |
| R5 | 同上，uuid 修正 + debug 日志 | 认领/remap/mixin 直证；建世界后接管激活实锤（事件2）；Ingest 8 worker 全炸与 R1 逐字同（事件3） |
