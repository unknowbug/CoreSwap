---
version: 1.0.27
status: published
date: 2026-09-07 22:50（Get-Date 锚定）
---
## 1. 构建产物

- jar: `E:\PYTHON\CoreSwap\runtime\1.20.1\java\build\libs\coreswap-1.20.1-1.0.27.jar`
- jar sha256: `E7D7EFCDFDA4E00988E6D654AF5EAF53D59E642FCA812763CC32B756621ADF44`
- dll sha256（jar 内 worldgen.dll = `target/release/worldgen.dll`，主工作区核验一致）: `7A411E013F2EAA25BCD96E5BB06999F84D252CA95AFD3D7E763DCD250F6A9FD1`（2178560 B；即 260907-09 起的现役基线，替换 1B5AA1DE）
- 源 commit: `be27d68`（realmod 修复代码 = cee44cf，文档 = 0eb3bd7/be27d68）

## 2. 验证记录（1.0.26 → 1.0.27 增量全链 confirmed）

| 判据 | 结论 | 证据 |
|---|---|---|
| 缺陷一修复（default_block 创建期解析 miss 运行期注册 mod 方块 → 整片 AIR） | dev + 生产双口径修复：SurfaceBuilder `default_block_name` 惰性按名解析（4 消费点） | `.artifacts/realmod-e2e-260907-09.md`（confirmed）+ 生产 V1 PASS |
| 缺陷二修复（biome.rs skip 规则使 vanilla biome 同名 override 恒被跳过） | mod 目录 override 分支修复 + 行为化日志；override 臂 AGG ≠ baseline 双向回归 | `.artifacts/realmod-e2e-260907-09.md` + `.tmp/modns-override-260907-09.log` |
| N5：writeback 反查按 rust_id | `[BLOCKS-REG] java_raw=1003 rust_id(overworld=1003 nether=1003 end=1003) writeback=testcontent:test_brick` | `.tmp/forge-prod-v1b-260907-10-boot.log` |
| **生产环境 e2e（本版新增判据面）** | Forge 47.4.5 + Connector beta.49 专用 SRG 服务端：mod 方块写回落世界 PASS（test_brick 672/144 sections，16 post-Done forceload chunks）；执行体三元组 judge 独立重算逐位一致；`server-...-srg.jar` 载体确认 | `.artifacts/forge-prod-e2e-260907-10.md`（confirmed，judge review-260907-10-001 PASS-with-conditions 条件已应用） |
| 执行体三元组（本票） | target dll = 发版 jar 内 dll sha256 一致（主会话核验，同基线 dll 无重构建） | 本文件 §1 |
| cargo test | 7/7 | cee44cf 记录 |
| 1.0.26 既有判据 | nether/end 接管、BUG-002 修复等不回归——本版 dll 相对 1.0.26 的改动面 = worldgen-core 三文件（realmod 缺陷一/二修复 + N5），judge 审查覆盖 | cee44cf + review-260907-09-001 / review-260907-10-001 |

## 3. Release notes（草案）

### English

**Real mod compatibility hardening — production-verified.**

- Fixed: mod-registered blocks resolving to AIR when used as surface default_block (creation-time lookup now lazy per-chunk name resolution)
- Fixed: mod data-pack biome overrides were silently skipped by the takeover skip rule
- Block write-back now resolves by runtime block id (id-domain aligned across Java/Rust)
- New verification tier: end-to-end test on a dedicated Forge 1.20.1 + Sinytra Connector production server (SRG remapped) — real content mod blocks verified landing in freshly generated chunks
- Maintenance: production launch now documents required `-Dcpp.blockRegister` parameter

### 中文

**真实 mod 兼容加固——生产环境实测版。**

- 修复：mod 注册方块作为表面 default_block 时整片解析为 AIR（创建期解析改为逐 chunk 惰性按名解析）
- 修复：mod 数据包 biome override 被接管 skip 规则静默跳过
- 方块写回改为按运行时 block id 反查（Java/Rust id 域对齐）
- 新增验证层级：Forge 1.20.1 + Sinytra Connector 专用生产服（SRG 重映射）端到端实测——真实内容 mod 方块落世界验证通过
- 维护：生产启动参数 `-Dcpp.blockRegister` 必带项已文档化

## 4. 发布动作清单（Maint 执行）

- tag：`v1.0.27`
- 标题建议：`CoreSwap 1.0.27 — Real mod compatibility, production-verified` / 中文 `CoreSwap 1.0.27 —— 真实 mod 兼容生产实测版`
- notes：用 §3
- asset：`coreswap-1.20.1-1.0.27.jar`（原文件名）
- 发布后：① status 镜像回写（260907-10 提醒：1.0.26 的回执写在了工单 head，status 镜像目录仍缺——本票请按契约建 `status\` 镜像）② 本票 status → published

## 5. 已知边界 / 降级声明

- 生产 e2e 覆盖面：1 seed × 16 chunks × overworld（§9.7 口径声明：与 dev 口径不可比）；nether/end 的 mod env vivo 未单独跑（接管机制与 1.0.26 同基线无改动）
- V2/V3（mod namespace AGG / biome override 生产复跑）按 §9.7 声明收口：验证路径不经生产特有层，复跑判别力为零（workflow-patterns #33 补充案例）
- mod 维度自带 biome_params 场景未实测（遗留 idk，README 边界注记同 1.0.26）
- GPU 管线为下一阶段独立课题（用户 260907-10 拍板：先发本版定格基线，再启动 GPU 接入；GPU 接入后验证基线全面重置为本版）
