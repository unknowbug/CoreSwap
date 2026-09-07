# realmod 修复成果 Forge+Connector 生产环境端到端实测（260907-10）

---
编号: swe:forge-prod-e2e-260907-10
日期: 2026-09-07 22:15-22:30（Get-Date 锚定；标签 260907-10，与 260907-09 同日新块）
任务类型: 验证（生产口径）
置信度: candidate（judge 审查中；confirmed 待用户）
架构: .investigations/000-架构设计/架构计划-260907-10.md（用户已批准）
---

## 1. 目标与载体

- **对象**：260907-09 realmod 缺陷修复成果（git `cee44cf`，终版 dll sha256 `7A411E01...F6A9FD1`）在 **Forge 47.4.5 + Sinytra Connector beta.49 生产环境（SRG 命名）** 的端到端行为。
- **载体**：`runtime/forge-server`（260906-03 建成）；主线 remapped jar `coreswap-1.20.1-1.0.26.jar`（jar sha256 `B139C54E1332381BE78E6A6A06702C70B90C5C0BC45C30BC6495A43CDBF4DF02`，1431457B，构建于 09-07 22:08）+ content-test remapped jar `content-test-1.0.0.jar`（loom remapJar 产物）。
- **执行体三元组（#36/#37）**：jar 内 `native/worldgen.dll` sha256 = `7A411E01...F6A9FD1` = `target\release\worldgen.dll`（逐位一致）；生产 boot 日志 `[CppBridge] dll=... size=2178560 sha256=7a411e013f2eaa25...` 加载确认；SRG 载体确认（boot 日志含 `server-1.20.1-20230612.114412-srg.jar`）。
- **seed**：`7691421705105351955`（server.properties level-seed，三查核对一致；与 260906-03 V2 同种子）。

## 2. 验证结果

### V1 注册→写回落世界：PASS（生产 vivo）
- 三轮对照：
  - 轮1/轮2（无 env，含轮1 陈旧 jar 修正前后）：test_brick = **0**
  - 轮3（`CORESWAP_DEFAULT_BLOCK=testcontent:test_brick` + `-Dcpp.blockRegister=1`）：test_brick = **672 命中 / 144 sections**（16 个 post-Done forceload chunks，blocks 800..863）
- #80 判据：forceload 在 `Done (13.928s)` 之后触发，观察对象 = post-Done 新区块 ✔
- #81 闭环判据：行为化日志（`[WGH] env override CORESWAP_DEFAULT_BLOCK=testcontent:test_brick (was minecraft:stone/netherrack/end_stone)` 三维命中）+ A/B hash 双向变化（0 ↔ 672）✔
- 方案 C + 候选 B 生产版：`[BLOCKS-REG] testcontent:test_brick java_raw=1003 rust_id(overworld=1003 nether=1003 end=1003) writeback=testcontent:test_brick`（id 域对齐 + 按链路真实消费 id 反查还原 ✔）
- test_lamp = 0：预期（`-Dcpp.blockRegister=1` 的 1 解析为 limit=1，仅注册首个 mod 块 test_brick）。
- 注脚（judge review-260907-10-001）：轮3 boot 日志 `Done` 前仍有 3 行 unknown→AIR——spawn 预生成时段（#80：pre-Done = vanilla 生成窗口）惰性解析 miss，注册发生在 Done 前、该窗口区块不重生成，与 #80 时序自洽，不影响 PASS 判定；unknown 行在场不作判别，`[BLOCKS-REG]` 有无才是判别面。

### V2 缺口3（mod namespace settings AGG 恒等）：不重跑，§9.7 声明
- 该验证为 diag bin（diag_modns）直读数据目录载体，**不经 Java remap/SRG 管线**，生产复跑无判别力；生产侧相关证据 = boot 日志 `initNether/initEnd enabled=true` 数据加载正常。与 dev 口径的可比性边界：载具不同（diag vs vivo），不可互引量级。

### V3 缺口4（biome override 分支）：生产面收口为数据供给声明
- 生产数据目录 = mod jar 资源解压（CoreSwapFixHelper → `<tmp>/coreswap-data`），不含 testcontent biome override 数据 → vivo 无 override 触发场景；该分支修复属 Rust+数据层，无 SRG 暴露面（同 V2 逻辑）。dev 侧 A/B 双向验证维持（modns-override-260907-09.log）。

### V4 执行体三元组 + SRG 载体：PASS（见 §1）

## 3. 发现（高价值）

**生产/开发口径参数缺失第二实例（build-tooling #28「冒烟口径 ≠ 存档口径」家族）**：
- 现象：轮1/轮2 env override 行为化日志已命中（分支生效），但 test_brick 恒 0；boot 日志出现 `[BLOCKS] unknown block 'testcontent:test_brick' -> AIR (register via wg_register_block)` ×3，且无任何 `[BLOCKS-REG]` 行。
- 根因：`CppBridge.registerModBlocks()` 被 `-Dcpp.blockRegister` sysprop 门控（CppBridge.java:168，默认关）。dev 260907-09 经 gradle `-PblockRegister`→`-D` 映射自动带入；生产 run.bat 口径无该映射行 → mod 方块从未注册进 Rust registry → 惰性解析（#79 修复）按名 miss → AIR。
- 修复：`user_jvm_args.txt` 追加 `-Dcpp.blockRegister=1`（生产口径参数清单 +1 必带项）。
- 教训：跨口径（dev gradle vs 生产裸 java）参数清单按口径分组维护（#28），新门控参数加入时 MUST 同步登记生产口径携带方式；行为化日志（env 命中行）与注册行为化日志（[BLOCKS-REG] 有无）组合才是完整判据——只有前者会误判「链路已通」。

## 4. §9.7 验证可比性声明

- 载体：Forge+Connector 生产专用服（SRG）vs dev loom runServer（named）——两口径结论不互引。
- 覆盖面：V1 为 16 chunk / 8×8 forceload 区域；非全维度全 biome。V2/V3 生产面以声明收口（理由见上），dev 证据维持原状。
- 与既有口径可比性：dev 260907-09 的 570 sections（728 chunks）与本轮 672/16 chunks 量级同阶但**不可比**（chunk 数、区域、载具均不同）；test_brick 存在性判据成立即可，量级不作对齐指标。

## 5. 产物与日志索引

- 架构：`.investigations/000-架构设计/架构计划-260907-10.md`
- boot 日志：`.tmp/forge-prod-260907-10-boot.log`（轮1）/ `.tmp/forge-prod-v1-260907-10-boot.log`（轮2）/ `.tmp/forge-prod-v1b-260907-10-boot.log`（轮3 PASS）
- 扫描脚本：`.tmp/scan-region3-260907-09.py`（复用）/ `.tmp/scan-r11-260907-10.py`（定点 palette）；扫描输出落盘：`.tmp/scan-v1b-out-260907-10.log`（复扫复现 672/144，逐位一致）
- judge：`.investigations/realmod-e2e/review-260907-10-001.md`（PASS-with-conditions，三项条件已应用）
- 知识库草稿：`.investigations/realmod-e2e/knowledge-draft-260907-10.md`（subagent 产出，已应用：build-tooling #32 + #31 补充案例 + workflow-patterns #33 补充案例 + INDEX）
- 环境改动（git 外，runtime/ 整树 gitignore）：`runtime/forge-server/user_jvm_args.txt` +`-Dcpp.blockRegister=1`；`mods/` 陈旧 1.0.26 → `.stale-0906` 备份 + 新 jar + content-test jar。

## 6. 状态

- 本产物 status: **candidate**（judge 审查中）。confirmed 待用户拍板。
