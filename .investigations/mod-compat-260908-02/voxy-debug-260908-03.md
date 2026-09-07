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
