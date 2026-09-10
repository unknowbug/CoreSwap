# 260910-03 vivo 冻结课题 — core.judge 审查意见（draft）

> 审查人：core.judge（subagent，只出意见不改 status）
> 审查对象：.investigations/vivo-freeze-260910-03/record.md + CoreSwapFixHelper.java（1.21.6 / 1.20.1）+ .tmp/hang-repro-260910 原始证据 + git 工作区
> 日期标签：260910-03

## 结论：PASS-with-conditions

根因链证据闭合、修复方向正确且已双版本落地、排除清单可信；4 项条件不阻塞结案，但应随后续动作消化。

## 逐项审查

### 1. 根因链证据完整性 — 通过（含 1 处表述瑕疵）

链条：existence-only marker → 1.20.1 旧缓存被 1.21.6 命中 → blocks.json id 域错位（982/1003，缓存 sha ee01b749 vs jar 617c3dae）→ 错块 + 无支撑重力块 → 级联 + 百万 item（×1,031,765，count_entities.py 普查）→ tick 雪崩 + OOM → 冻结。各环节均有独立数据层证据，无跳跃：

- sha 铁证直接证明「缓存内容 ≠ 本 jar 数据」，这是 existence-marker 缺陷的机制级证实；
- 实体普查与线程 dump 相互印证：load dump（:478-497）Server thread cpu=166046ms RUNNABLE 于 class_1542(ItemEntity).tick → 块碰撞扫描，Render thread 同样烧在 ItemEntity tick——与百万 item 普查一致；
- 排除与确诊的因果方向（为什么 dev 服/Chunky 全干净：-PcppWorldgenDir 直读不走 Temp 缓存）自洽地解释了「复现载体分裂」，这反而是根因结论的强化证据。

**瑕疵**：record.md 概述行称「读图卡死 dump = FallingBlockEntity tick」——load dump 实际 Server/Render thread 均为 **ItemEntity**（class_1542）tick；FallingBlockEntity（class_1540）未出现在该 dump（应在 elev dumps，本次抽查未逐一核对）。不影响结论，但结案文档应修正表述。

### 2. 修复正确性 — 通过（含 1 项设计权衡需声明）

- `isDataCacheStale`：blocks.json 作 id 域锚点是充分的——块表是 Rust 侧 id 映射的入口数据，任何 id 域变化必经 blocks.json；且 existence marker 仍保留作第二道闸。逐字节比对对齐 extractNativeDll 既有模式，风格一致。
- `readJarBytes` 返回 null → 保守 true（强制重解压）合理：宁可慢不可错，且解压后 marker 缺失会显式抛错而非静默。
- 异常路径 catch → true 同理合理。
- 成本：每次启动一次 jar 内单文件读取（blocks.json 百 KB 级）+ 长度预检，可忽略；不匹配时整体重解压与原逻辑同量级。
- **设计权衡（条件 C1）**：单锚点对「blocks.json 不变、其他数据文件变（如 tags/biome params/noise settings）」的 jar 升级漏检——缓存陈旧但指纹相同。现实风险低（这些文件与块表通常联动变更，且 260907-04 tag marker 已覆盖 tags 缺失形态），但建议后续扩展为多文件/清单指纹。

### 3. 遗留风险 — 已核查，报告如下（只报告不修）

- **1.20.1 生产源码存在同款 existence-only 逻辑？——存在过，现已同款修复**：runtime/1.20.1/java/.../CoreSwapFixHelper.java:44-83 与 1.21.6 逐字同款（isDataCacheStale 已在）。双向交叉污染（1.21.6→1.20.1 反向）同样被堵住。⚠️ 注意两版本共享同一 Temp 路径 `coreswap-data`，交替运行两个版本 jar 时每次都会触发整体重解压——成本可接受但值得知晓。
- **旧存档残留（条件 C2）**：用户冻结期间生成的旧世界（错块地形 + 残留实体）在修复后加载仍会触发级联/卡顿——修复只保证新地形正确，不清洗旧世界数据。应告知用户（新开世界或清理实体）。
- **交付证据链（条件 C3）**：`runtime/` 整体被 gitignore（git status 仅见 .investigations/ 未跟踪），修复代码无版本控制痕迹，三源核对中「git HEAD + diff」一源缺失——修复进生产 jar 的证据只有 jar sha 9eae48cd + 用户行为面确认。建议归档该 jar 的验证记录（或对提取的 blocks.json 复核 sha=617c3dae）。

### 4. 排除清单 — 可信

- Rust 死锁排除：日志 2375 unique chunks 持续推进 + fillBlocks 卡帧后转空闲，是行为证据不是静态推断，充分；
- 内容差排除：diff-result.txt 定点对拍（邻居更新坐标 ±4 柱）逐点一致 + 0.015% 全域普查，且口径限制（权威数据口径 ≠ 客户端故障面）已在方法论 #1 显式声明，符合 §9.7 验证可比性声明精神；
- 光照默认关、无 client mixin：静态配置事实，可信。

### 5. 未验证面声明 — 诚实

修复后仅用户行为面确认（不再冻结），无双臂回归量化——已显式声明；1.21.6 5.15× 性能回归已单独立项并注明「冻结修复后才显性化」。符合降级声明要求。

### 6. 文档质量瑕疵（条件 C4）

- record.md 存在**两份「事件链」+ 两份「产物清单」章节**（过程追加未合并）；
- 尾部残留「当前候选机制（candidate，双臂 diff 裁决中）」段——已被根因结论取代但未按 §15.4 取代链标注，读者易误解课题仍开放。

## 条件清单

| # | 条件 | 性质 |
|---|------|------|
| C1 | isDataCacheStale 后续扩展多文件/清单指纹（blocks.json 单锚点对「块表不变、附属数据变」的升级漏检） | 建议项，不阻塞 |
| C2 | 告知用户旧存档（错块地形+残留实体）修复后加载仍会异常，建议新开世界 | 必须做（用户沟通） |
| C3 | 归档修复 jar 9eae48cd 的交付验证记录（runtime/ 无版本控制，git 三源缺一） | 必须做（证据落盘） |
| C4 | record.md 修缮：合并重复章节、候选机制段加 supersedes 标注、load dump 表述 ItemEntity 修正 | 必须做（文档） |

## 推荐状态

根因结论维持用户已 confirmed 的定论；本审查不改动任何 status。建议主会话消化 C2-C4 后归档，C1 进 1.21.6 性能回归立项的附带待办。
