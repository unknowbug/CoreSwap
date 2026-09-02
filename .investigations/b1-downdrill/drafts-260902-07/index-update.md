# 草稿：knowledge/INDEX.md 对应分类行更新

> 载体：`knowledge/INDEX.md`「分类入口」表。只改两行（compiler-idioms / workflow-patterns），主会话应用。

---

「语言/编译器惯用法」行改为：

| 语言/编译器惯用法 | [discovered/compiler-idioms.md](discovered/compiler-idioms.md) | Java/MC 代码生成模式、浮点/整数语义、插值公式、MSVC/Windows·JVM 平台坑、跨层 id 域错位 raw block id vs state id（2026-09-01）、锚坐标换算 off-by-one below_top/above_bottom（260901-02）、JSON 布尔字段经 as_f64 读取恒 false——分型标量 API 静默语义腐蚀（发现 #8，260902-03）、跨 session raw id 标注三查——未验证标注当公理继承整链作废（发现 #9，260902-07） |

「工作流模式」行改为：

| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新）、接管单阶段后的后续阶段上下文依赖（2026-08-31）、临时产物唯一隔离区（260901-03）、cppReplace 存档口径三阶段归因法 + 同 dll 重跑非确定容差（发现 #10，260901-03）、嵌套接管管线双跑风险——内层全管线 × 外层分步拦截（发现 #11，260902-01）、静态对拍必须对拍解析产物而非输入原文——假阴性掩盖真 bug（发现 #12，260902-03）、探针坐标 bug 制造 100% 单向假象——探针输出先做 sanity check + one-step decisive probe 逐层收敛（发现 #13，260902-04）+ 测量侧先查三犯（wBiome 坐标/NoiseConfig 维度/pregen 提升 chunk）与 RegistryKey 命名空间过滤恒 false、探针零输出先查过滤/驱动条件（#13 补充案例，260902-05/06）+ 探针指标盲区（指标先从判别证据反推）与行首锚 grep 假零输出（#13 补充案例，260902-07） |
