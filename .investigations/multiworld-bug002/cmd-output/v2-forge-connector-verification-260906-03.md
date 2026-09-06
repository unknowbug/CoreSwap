# V2 生产环境验证记录（Forge 47.4.5 + Sinytra Connector beta.49）

日期: 260906-03（实际 2026-09-06 15:07-15:15）
载体: runtime/forge-server（本课题新建，Forge 1.20.1-47.4.5 安装器装 + Modrinth Connector-1.0.0-beta.49+1.20.1 + fabric-api-0.92.6+1.11.15+1.20.1）
mod jar: coreswap-1.20.1-1.0.24.jar（本次构建，含 mixin 维度判定重构）；dll sha256=febe913e15da136a...（init 行实录）
seed: 7691421705105351955（server.properties level-seed，与 issue #24 报告者同种子；CppBridge init 行核对一致）
生产命名环境实证: boot 日志含 server-1.20.1-20230612.114412-srg.jar（SRG 命名载体确认）

## 结果（判据 → 实测）

| 判据 | 预期 | 实测 | 判定 |
|---|---|---|---|
| 修复 jar 在生产环境 mixin APPLY | 无 MixinApplyError | 首启（含路径笔误版）曾 APPLY FAILED，修正 `world/gen/chunk/Chunk`→`world/chunk/Chunk` 后二启全绿 | PASS |
| end 维度判定 | settings=minecraft:end → 放行 | `release to vanilla: settings=minecraft:end shape=0/256`（首启 once 日志） | PASS |
| end 被误接管的旧指纹消失 | 末地 chunk 不打 populateNoise(nether) intercepted | end forceload chunk (0,0)/(7,0) 无任何 intercepted 行 | PASS |
| end 地形健康 | vanilla 末地（无 nether 床岩地板） | y1=air(void)、y55-58=end_stone（if block 探针 say 实录）；bedrock 探针未命中 | PASS |
| nether 接管不回归 | settings=minecraft:nether → 接管 | 首启 81 chunk `populateNoise(nether) intercepted` + WG-FILL readback nonair | PASS |
| overworld 接管不回归 | 新鲜 chunk 仍接管 | 二启 forceload (2000,2000) → 81 chunk `populateNoise intercepted` | PASS |
| 根因端点闭合（V2） | 「end 38% = nether 误接管」因果端点 | 生产 SRG 载体 + end 释放 + 地形 vanilla 化全链实测 | PASS |

## 降级/边界声明（§9.7 可比性三要素）

- 载体：本地 Forge+Connector 服务器（非报告者整合包）；**mod 维度（aether 等）未本地装载**——其放行与 end 走同一代码路径（settings id ∉ 接管集），机制同源但 mod 维度实体未本地复测
- 覆盖面：end/nether/overworld 三维度判定 + end 地形探针；未做全维度 region 逐方块 A/B diff（报告者口径）
- 与既有口径可比性：与 issue #24 报告者 260905 全维度 diff **不可直接比**（报告者含 mod 整合包，本环境 vanilla 三维度）——完整 A/B 验收建议报告者以修复版复测，或本地补装 aether 级 mod 后重跑

## 过程纪要

- 首启 mixin APPLY FAILED：重写时 Chunk 签名包路径笔误（refmap 无法重映射）→ 修正 + 二启绿
- overworld spawn chunks 在 CppBridge init（Done 后）之前生成 → init 前 vanilla 生成属预期边界（与 Bug 无关）
- RCON 脚本：.tmp/multiworld-bug002-260906/rcon_forceload.py / rcon_probe2.py
- 日志：runtime/forge-server/firstboot.log（首启+end 验证）/ secondboot.log（二启+探针）
