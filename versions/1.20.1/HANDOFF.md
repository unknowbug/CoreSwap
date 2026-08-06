
## 2026-08-06 晚 · 多维度引擎（下界跑通 72%）

- wg_create 纯数据驱动（settingsName/biomeParamsFile/worldHeight——引擎从 JSON 读 minY/noiseHeight/aquifersEnabled；dimFor 硬编码表已删）
- surface_rule JSON 解析器（下界全节点）→ 下界 surface 数据驱动；VerticalGradient 反锚序（bedrock_roof）
- 下界 72% 全匹配 / 75.4% 非 air（y 循环 noiseHeight 修复 + out 越界修复）
- 剩余 28% 差异 = density（y 48-80 微负——base_3d_noise y 方向采样差，参数已确认读对；下一步 base_3d_noise 分量对比）
- 工具：DensityProbe（Java 导 vanilla density）/ got_export -densityDump / wg_sample_density API / BlockProbe 维度化
- 坑：runDepth 洞内重置破坏主世界（回滚）；nether base_3d_noise 参数不在注册表（old_blended_noise 内联）；下界 y_scale 0.375≠主世界 0.125
- PR 待处理：#2（mod id，已修 1.0.3 重复）、#3（Forge+Connector 兼容——新方向）
- docs/09-multi-dimension.md（多维度知识库，新增不覆盖）
