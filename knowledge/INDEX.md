# Knowledge INDEX — 知识库总入口（分析前先查）

> 双轨结构（2026-08-08 确立）：
> - **项目特定知识**（MC 1.20.1 实现结论/对齐状态/排查过程）→ `versions/1.20.1/docs/`（01-09 主题篇 + 10 时间线），管理纪律见 AGENTS.md 三。
> - **通用可复用模式**（跨版本/跨项目：语言惯用法、工具坑、算法指纹）→ 本目录 `knowledge/`，由 core.knowledge skill 管理。

## 分类入口

| 分类 | 文件 | 说明 |
|------|------|------|
| 语言/编译器惯用法 | [discovered/compiler-idioms.md](discovered/compiler-idioms.md) | Java/MC 代码生成模式、浮点/整数语义、插值公式、MSVC/Windows·JVM 平台坑 |
| 还原工具误译及修正 | [discovered/f5-bugs.md](discovered/f5-bugs.md) | javap/反编译不可信点、反射缓存污染、修正方法 |
| 构建/工具链坑 | [discovered/build-tooling.md](discovered/build-tooling.md) | gradle daemon/env/参数解析、task UP-TO-DATE 跳过、文件同步 |
| 已确认的算法/协议指纹 | [discovered/algorithm-fingerprints.md](discovered/algorithm-fingerprints.md) | MC 密度/噪声算法特征、scale/seed 坑、key 语义 |
| 混淆/反逆向手法 | [discovered/anti-patterns.md](discovered/anti-patterns.md) | （CoreSwap 非二进制逆向，一般空置） |
| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新） |
| 预置知识 | [builtin/README.md](builtin/README.md) | 预留；RE-Framework knowledge-builtin 为汇编逆向内容，CoreSwap 不复制 |

## 写入规则（core.knowledge）

- 发现可复用模式 → 立即写入 discovered/ 对应文件（不拖）
- 每条格式：`## 发现 #N: 标题` + 发现时间/发现者/来源定位/置信度/module + 观察/证据/如何利用
- 写入后**同步更新本 INDEX**（对应分类加一行链接）
- 与 docs/ 边界：docs/ 记「对 1.20.1 的验证结论」，本目录记「可复用的通用规律」（1.18/1.19 迁移时直接查这里）
