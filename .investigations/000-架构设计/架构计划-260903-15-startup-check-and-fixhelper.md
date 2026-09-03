---
编号: 000
任务: 启动期 noise key 机械校验（#26 判据 1）+ CoreSwapFixHelper 解压布局根治（#15），完成后编译交付用户实测
任务类型: 防御性工程（swe 域，收敛闭环）
模式档位: 轻量
状态: 待批准
session: 260903-15
---

## 范围（含明确不做什么）

**做**：
1. **① 启动期机械校验**（`WorldgenRust/src/worldgen_handle.rs`）：
   - 把 overworld 预加载静态清单（L272）提为单一事实源常量 `OVERWORLD_NOISE_KEYS`；
   - `create()` 末尾新增校验步：对 `build_overworld_rule()` 产出的规则树收集全部运行时引用 noise key，断言 ⊆ 已预加载 sampler 集合，缺失即 fail-fast panic（带缺失 key 清单）——把 505 类 expect panic 从「低频分支运行时崩溃」提前到「启动即报」；
   - 非 overworld 侧：`collect_noise_keys` 扩展为收 surface rule JSON 中**任意节点的 noise 引用字段**（不再只 `noise_threshold`），收完同样走 ⊆ 校验（同一次循环即可，动态收集后预加载，天然闭合）。
2. **② CoreSwapFixHelper 解压布局根治**（`runtime/1.20.1/java/src/main/java/wg/bench/CoreSwapFixHelper.java`）：
   - 根因：资源布局 `worldgen-data/minecraft/...`（无 `data/` 层），而 `extractFromJar` 只把 `rel.startsWith("data")` 的条目路由进 wgDir——`minecraft/...` 条目全被丢到 `target`（coreswap-data 根），marker `wgDir/data/minecraft/worldgen/noise_settings/overworld.json` 永远不出现 → IllegalStateException；
   - 修复：路由规则改为「`data/` 开头 → wgDir；`minecraft/` 开头 → wgDir/data/ 前缀拼接」，jar 与 dev classpath 两条分支同改；错误信息 fail-fast 提示带 `-PcppWorldgenDir` 绕过项。
3. **③ 编译交付**：`cargo build --release` + worldgen.dll 同步 java resources（build 链路既有流程），Java 侧 gradle 编译产物交付用户实测。

**不做**：gpu-batch-merge 决策（另行拍板）；idk 类课题（55→33 漂移等）；SurfaceBuilder 规则逻辑任何语义改动；资源布局重排（选代码路由修复，不动 resources 目录）。

## 任务拆解

| # | 子任务 | 预期产物 |
|---|---|---|
| 1 | ① Rust 启动期校验 + collect_noise_keys 泛化 | worldgen_handle.rs / surface_rules.rs diff |
| 2 | ② FixHelper 路由修复 | CoreSwapFixHelper.java diff |
| 3 | 回归：cargo test + 现有探针/基准快速冒烟（seed 8576294172403134396 小 sweep 无 panic、hash 不变） | cmd-output 落盘 |
| 4 | ③ 构建 + dll 同步 resources + gradle assemble 交付 | 产物路径回报用户 |

## 验证方式

- Full 层回归：四臂 hash 必须仍 `f2b1a3932c6e589e`（零语义回归判据）；4096 sweep 本轮不重跑（①②均非生成语义改动，小 sweep + hash 足够；若 hash 变即语义回归，立即停）。
- ① 校验生效证明：人为从清单删一 key 跑启动 → 必须启动期 panic 报缺失（自变量真被改变，#20 判据）。
- ② 修复证明：删除 tmp 缓存 `coreswap-data` 后走解压分支启动 → marker 出现、server 正常起（复现 build-tooling #15 现象并闭合）。

## judge 预置

- 收尾交付 MUST judge（三源核对：artifacts 快照 + git diff + 回归记录）。

## fan-out 预置

- 无分叉预期（单假设防御性工程）；若 ① 校验实现出现 ≥2 互斥方案分歧再评。

## 知识库更新

- 结论性落盘：subagent 产出草稿（workflow-patterns #26 判据 1 落地案例 / build-tooling #15 根治记录）+ 主会话应用验证。

## 子角色介入点

- scout: 否（现场已勘明，无机制未明）
- worker: 收敛编码主会话直接做（swe 域 v0.8 收敛门）；收尾 judge 用 subagent
- fan-out: 无
- judge: 收尾交付 MUST（subagent，三源核对）
- knowledge: 结论性 docs 草稿 subagent 产出 + 主会话应用
