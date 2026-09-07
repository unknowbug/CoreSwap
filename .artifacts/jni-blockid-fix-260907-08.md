# mod 方块 id 错位写回修复（候选 B：wg_register_block_id 显式 id 对齐）交付 — 260907-08

状态: **candidate**（待 judge + 用户拍板）· 验证分层: Full（cargo test 全量 + runServer 实机冒烟）
日期锚: 260907-08（Get-Date 2026-09-07 19:35 开工）· HEAD 基线: a3df073（交付时）

## 范围

NEXT_SESSION 260907-07 开工点 1：修复 mod 方块注册 rust_id 与 Java raw id 域错位导致写回（`Registries.BLOCK.get(id)`）无对齐保证的问题。方案 = 源码核对后的**简化候选 B**：显式 id 注册使 Rust 内部 id 与 Java raw id **同域**，写回直查即对齐——**无需映射表**（映射表引入第二真相源，显式 id 消除映射层；对计划的小偏差，已声明）。

## 改动清单

1. `worldgen-core/src/blocks.rs`：`BlockRegistry::register_with_id(name, id)`——显式 id 注册；语义：同名幂等（不一致告警保留既有）/ id 被占拒绝 -1 / 越界拒绝 / 成功推进 `next_id`（显式 id 后动态分配不回退碰撞）。新增测试 `register_with_id_semantics`。
2. `worldgen-core/src/worldgen_handle.rs`：`register_block_with_id` 包装。
3. `worldgen-core/src/api.rs`：新 ABI `wg_register_block_id(handle, name, java_raw_id)`——**旧 `wg_register_block` 保留不动**（合成名冒烟 + ABI 兼容，零回退成本）。
4. `versions/1.20.1/rust/src/jni_bridge.rs`：`Java_wg_CppWorldgen_registerBlockId` JNI 接线。
5. `runtime/1.20.1/java/.../wg/CppWorldgen.java`：native 声明 `registerBlockId(long, String, int)`。
6. `runtime/1.20.1/java/.../bench/CppBridge.java`：`registerModBlocks()` 有 java_raw 的方块走 `registerBlockId`（候选 B 路径）+ 逐条打印 `java_raw ↔ rust_id ↔ writeback` 对齐自证；合成名回退扩展为 aligned（显式 id，模拟真实 mod raw id 位置 = `Registries.BLOCK.size()`）+ dynamic（旧路径，ABI 兼容哨兵）双探针。

## 验证记录（§9.7：载体 = cargo test workspace 全量 + runServer 实机；覆盖 = 显式 id 注册链路；与存档级对齐口径不可比）

- cargo test workspace 全量：**7/7 passed**（含新增 register_with_id_semantics：对齐/冲突拒绝/越界拒绝/next_id 推进/动态与显式互不干扰）。
- dll：`target\release\worldgen.dll` 2177024B @2026-09-07，sha256 = **87C35BD6520A1455C792E185DF10317DA1937172DABB36A98F3FED3D2424C776**（本块新基线，替换上块 C7AD…；记录日 2026-09-07 + 构建命令 `cargo build --offline -p worldgen --release` + HEAD a3df073 上下文齐备，#10 纪律）。注：冒烟 round4 跑在 judge 前 dll（230B45A9…）上；judge N3/N4 条件（共享 const + 注释）应用后重建，改动为行为恒等的注释/字面量提取（Rust 编译器宏展开层），cargo test 7/7 复绿，符号哨兵 FOUND——冒烟结论对该 dll 有效，#76 同代码双臂恒等判据注释级适用。符号哨兵 registerBlockId/wg_register_block_id 双 FOUND（#23 假绿防护）。
- runServer 冒烟 round 4（seed 8576294172403134396，-PcppReplace -PcppBlockRegister=1，-PcppLib 直载）：
  - `testmod:aligned_probe java_raw=1003 rust_id(overworld=1003 nether=1003 end=1003)`——**id 域对齐断言 PASS**（1003 = `Registries.BLOCK.size()`，即真实 mod 首块 raw id 位置；三句柄一致）。
  - `testmod:probe_block_1 rust_id=1004`——**next_id 推进实机 PASS**（动态分配不碰撞显式 1003）。
  - writeback=minecraft:air 属预期（无 mod 环境 1003 处无注册表条目；有 mod 时该处即 mod 方块——本机制断言为 rust_id==java_raw，Java 侧 `Registries.BLOCK.get(rust_id)` 直查命中）。
- 验证输出：`.tmp/verification-260907-08.log`；冒烟日志 `.tmp/blockreg-smoke4-260907-08.log`（round 1-3 失败轮留档）。

## 失败轮记录（错误优先）

- round 1：aligned 探针用真实 vanilla 块 calcite（raw=910）→ Rust **正确拒绝**（910 已属 blocks.json `minecraft:calcite`）——冲突拒绝语义按设计工作，但探针设计缺陷：真实 mod raw id 在 vanilla 表之后，必须用 `Registries.BLOCK.size()` 模拟位置。
- round 2：compileJava FAILED（变量作用域 name/ids 重复声明）→ 改名修复。
- round 3：`-PcppWorldgenDir` 指向 `versions\1.20.1\data` 布局缺 wg_create 根级 settings 文件 → handle=0 全 -1（`enabled=false` 是判别签名）；改走 jar 内 worldgen-data 解压路径（与 260907-07 冒烟口径一致）后通过。**注意**：#15 的「-PcppWorldgenDir 必带」是存档口径项，注册冒烟不适用。

## 已知边界 / idk

- @anchor.idk: 真实内容 mod 端到端（缺口 3 mod 命名空间数据路径 / 缺口 4 同名 biome override）仍未实测——本环境无内容 mod，维持上块 idk。
- @anchor.idk: aligned 探针的 writeback 反查在无 mod 环境返回 air，非「写回落世界」级证据；落世界级验证依赖真实 mod（同缺口 3）。judge N5：对齐自证打印为 `get(javaRaw)` 而非 `get(rust_id)`，写回侧 rust_id 反查未独立证明（当前靠 rust_id 数值人工比对）——真实 mod 实测时改按 rid 反查。
- @anchor.idk: register_with_id 的 map insert 与 names[id] 写入非原子，依赖「创建期单线程」时序契约（judge N4，注释已显式声明）。
- 旧 `wg_register_block` 动态 id 路径保留（合成名冒烟专用）；其写回错位风险仍在（合成名无 java_raw，无写回语义，不受影响）。
- **Java 侧无 git 基线**（judge N2）：CppWorldgen.java / CppBridge.java 位于 gitignored `/runtime/`，无版本可审计，judge 直读核对一致——仓库既有策略，非本卡违规。

## judge 审查（260907-08）

- 判定：**PASS-with-conditions**（subagent core.judge，三源核对）。
- 条件应用：N1 index.yaml 补登（见下）✅ / N2 Java 无 git 基线声明（本节）✅ / N3 16384 → ID_TABLE_CAPACITY 共享 const ✅ / N4 创建期契约注释 ✅ / N5 rid 反查改进挂真实 mod 实测卡 ✅。
- N3/N4 应用后：cargo test 7/7 复绿 + dll 重建（sha 基线切换见验证记录节）。

## 预置介入点核对

- scout：否（机制已明，源码核对即前置）✅ · fan-out：未触发（单假设）✅ · judge：本交付 MUST · knowledge：subagent 草稿（进行中）· HOOK：计划批准 ✅（用户「批准，发 todo」），新函数路径在计划内预置。
