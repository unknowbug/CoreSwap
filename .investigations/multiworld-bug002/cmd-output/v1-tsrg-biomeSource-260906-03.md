# V1 廉价验证原始输出（cmd-output 落盘，judge 第 7 条要求）

日期: 260906-03（实际 2026-09-06，主会话执行）
命令: Select-String -Path srg_to_official_1.20.1.tsrg -Pattern "biomeSource" -CaseSensitive
文件: C:\Users\NDark\.gradle\caches\forge_gradle\minecraft_user_repo\de\oceanlabs\mcp\mcp_config\1.20.1-20230612.114412\srg_to_official_1.20.1.tsrg

## 首轮输出（未收窄，命中类名噪声 MultiNoiseBiomeSourceParameterList）

	f_62137_ biomeSource
	f_254681_ biomeSource
	f_226623_ biomeSource
		2 f_226623_ biomeSource
	f_226623_ ()Lnet/minecraft/world/level/biome/BiomeSource; biomeSource
	f_197244_ biomeSource
	f_197353_ biomeSource
		1 f_197353_ biomeSource
	f_197353_ ()Lnet/minecraft/world/level/biome/BiomeSource; biomeSource

## judge 独立重跑勘误（⚠️ 引用以此为准）

- **ChunkGenerator.biomeSource 的真实条目 = `f_62137_ biomeSource`**（tsrg 行 180533，
  ChunkGenerator 类段下）——即生产（Forge 1.20.1 官方类名 + SRG 成员名）运行时
  ChunkGenerator 的 biomeSource 字段名为 f_62137_。
- `f_226623_` 属 Structure$GenerationContext（同名 biomeSource 字段，巧合）；
  f_254681_/f_197244_/f_197353_ 属其他类。此前 worker/主会话引用 f_226623_ 为误引，
  已勘误（不影响机制结论：任何类段都证明生产字段名是 SRG 形式而非 "biomeSource"）。
- 映射方向核实：srg_to_official tsrg 左列 = SRG（生产运行时名），右列 = official/Mojmap；
  该文件即 ForgeGradle 安装期生产 jar 重映射依据 → 生产命名方向断言成立。

## 判定意义

getDeclaredField("biomeSource") 在生产环境对 ChunkGenerator（运行时字段名 f_62137_）
必抛 NoSuchFieldException → NoiseChunkGeneratorMixin 旧 wgIsEnd() catch 静默 false →
末地（0/256 形状）豁免失效被 nether 句柄接管。

## 边界记录（judge 第 5 条）

- 已验证（数据层）：生产字段名 = SRG →「wgIsEnd 生产恒 false」环节成立，可动手修复。
- 待 V2（运行时日志）：「end 38% 损坏 = nether 误接管」因果端点仍是静态推断，
  根因定论（confirmed 候选）前 MUST 补 V2 拦截日志（修复后预期：末地不再出现
  populateNoise(nether) intercepted，改打 release to vanilla: settings=minecraft:end）。
- V3（catch 打日志）随旧反射路径整体删除，不再适用；「不吞异常」违规同步消除。
