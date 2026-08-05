# CoreSwap AI 模块（占位）

实体 AI / 寻路核心的 C++ 模块。**尚未开工**——在 worldgen 闭环（JNI 桥 → 方块层 → 可安装 mod）之后启动。

计划（复用 worldgen 的方法论）：
1. POC 先行：逐位复刻 vanilla 寻路行为（A* 变体 + 世界碰撞查询）+ JNI 加速验证
2. 行为一致性验证（同 seed 同场景路径对比）
3. 产品化集成
