# 草稿：knowledge/discovered/workflow-patterns.md 发现 #10

> 用途：主会话应用——追加到 `knowledge/discovered/workflow-patterns.md` 末尾（发现 #9 之后），并同步 `knowledge/INDEX.md`（分类入口「工作流模式」行如需，沿用既有条目纪律）。当前最大编号 #9，本条为 #10。
> 状态：candidate（模式已被本轮 B1 案例三方实验验证，未走独立 judge）。

---

## 追加正文（从下面一行开始复制）

## 发现 #10: cppReplace 存档口径残差的三阶段归因法——先分阶段再定位机制

- **发现时间**：260901-03；**发现者**：nether-save-full session（B1 定论轮）；**来源定位**：`.investigations/nether-save-full/`（residual-interpretation + .b1/.b2 + judge-review）；**置信度**：candidate（三方实验数据实锤，模式层面复用性待下一案例）；**module**：workflow / Minecraft modding / 验证方法。
- **观察**：替换模式（如 cppReplace 只接管 noise+surface，Java carvers/features 仍在替换后地形上运行）下的存档口径残差是**多阶段混合产物**——直接把残差对到单一层（如 surface rule 条件链）会得出错误归因（错误 E6：B1 52k 块大宗互换一度被归因 surface rule 条件链，实为 feature 阶段产物在两种基底地形上的命中/形态差）。正确做法是**三阶段归因**：
  1. **阶段分解**：先分清 noise/surface（替换方 = Rust）与 carvers/features（存续方 = Java）各自贡献，再定位机制；
  2. **消融判别**：`WG_SKIP_SURFACE=1` 重跑——surface 关掉后残差从 93.55% 掉到 55.18% 证明 surface 是实心块主来源；且 blobs 不触发（stone 基底非 netherrack → blackstone=0）反向证明 blobs 是 feature 阶段、依赖 netherrack 基底；
  3. **纯替换方基线**：ctypes 直连 dll（或 rlib 直跑）取得纯 Rust 输出 vs 参照（本轮 77.43%，与存档口径 93.55% 载体不同不可比，§9.7）——分离「替换方自身缺口」与「存续阶段叠加产物」。
- **如何利用**：
  - **判别手段一（消融）**：单阶段开关（如 WG_SKIP_SURFACE）A/B 重跑——残差量级变化 + 依赖块（如 blackstone 依赖 netherrack 基底）是否消失，直接指认产物所属阶段；
  - **判别手段二（直连基线）**：ctypes/FFI 直连替换方库跑同区域——排除 Java 存续阶段干扰，取得替换方独立口径；与 rlib 直跑对拍可顺带验证 FFI 层确定性（本轮 cell 级 0 差异）；
  - **biome/来源分桶**：残差按 biome 列分桶——若差异 100% 落在 vanilla 某列（本轮 basalt_deltas），排除「源分配差」，收窄到「同源产物在不同基底上的表现差」；
  - **同 dll 重跑非确定性容差判据**：存续阶段（Java feature 邻块写入调度）本身非确定——同 dll 两次完整 run 相差 369 块（93.5156%→93.5508%）。**存档口径对齐指标必须声明该容差**：同 dll 重跑块级差 ≤ 百分级（千分位级百分比波动）属调度噪声，不构成实现回归判据；跨口径比较（探针 vs 存档 vs 纯 Rust）一律 §9.7 三要素声明。
  - **先消融后归因**：任何「残差 → 某层机制」的归因结论，出手前必须已有至少一个阶段消融或直连基线证据，否则降为 draft（反模式见 E6）。
- **同族模式**：发现 #4（参照数据状态三查 SURFACE vs FULL——参照阶段决定差异构成）是「对照侧」的前置；本发现是「被测侧」（替换模式运行时残差）的阶段归因，两者合用构成替换模式验证的完整口径纪律。
