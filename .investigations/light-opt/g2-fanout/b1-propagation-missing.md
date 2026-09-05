# G2 fan-out .b1 — H1「fallback chunk 收不到被接管邻居的传播更新」审查

- 候选：H1（propagation-missing）；worker：b1；状态：**draft（candidate 建议交 judge）**
- 验证分层：Partial（yarn 一手源静态审查 + 存档数据统计；无动态 trace）
- 数据：`.tmp/light-g1/{vanilla-region,rust-region}` 3377 chunks，残差 448
- 脚本：`.tmp/light-g1/b1_prop_h1.py`（输出 `.tmp/light-g1/b1_prop_h1.out.txt`）

## 1. 机制分析（yarn 一手源）

**cancel 跳过了什么**（`runtime/.../ServerLightingProviderMixin.java` L159-203 vs `.tmp/light-yarn/ServerLightingProvider.java` L171-184）：
mixin 在 light() HEAD cancel，复刻了尾部语义（setLightOn(true) + releaseLightTicket）+ 24 个 section 的 enqueueSectionData，唯一跳过的原方法语义 = PRE_UPDATE 阶段的 `super.propagateLight(chunkPos)`（L174-178）。H1 对「跳过了什么」的描述属实。

**propagateLight 的语义（关键）**——`ChunkSkyLightProvider.propagateLight(ChunkPos)`（`.tmp/light-yarn-extra/.../ChunkSkyLightProvider.java` L292-343）：
- 它 seed 的对象是**参数 chunk（本 chunk）自身**：逐 section 用自身 + 4 邻的 ChunkSkyLight（heightmap）把 sky=15 列填充，并把本 chunk 内可向外扩散的位置压入传播队列（method_51566）。
- 邻居数据是**拉取**的（`method_51589(x, z±1 / x±1)` 直接从共享 LightStorage 读邻 chunk 数据），不是等邻居推送。
- 队列扩散（doLightUpdates→method_51531/method_51530）在共享 LightStorage 上双向传播，可跨越 chunk 边界。

**因此 H1 的核心命题不成立**：vanilla 在此阶段是「拉取共享存储」语义。接管 chunk A 的 Rust 写回（enqueueSectionData）落入同一个共享 LightStorage；之后 fallback chunk B 走 vanilla light() → propagateLight(B) 时，B 自己的 seed **会把 A 已写回的数据拉进来**。「B 收不到 A 的推送更新」在 vanilla 语义下不是真实机制——push 不是必需的，pull 即可。
- 残余不确定性（@anchor.idk，specific）：若 B 的 light() 先于 A 的 Rust 写回被 light 线程处理，B seed 时读到的是 A 的初始数据；A 的后到写回是否触发对 B 的再传播，取决于 `LightStorage.enqueueSectionData/updateLight` 的再入队语义（该源不在 .tmp/light-yarn*/，本轮未核）。但即便存在，该缺陷的残差形态应是「chunk 边界面向 A 一侧的梯度衰减差」，不是整 section 翻转（见 §3）。

## 2. 数据验证

| 检验 | 结果 |
|---|---|
| proxy1 残差 × pregen 边界（fallback 只可能发生在边界：LIGHT 任务时邻 chunk 保证 INITIALIZE_LIGHT） | **残差 448 个中边界 0 个**（边界 chunk 残差率 0/254），内部 448/3123 = 14.3% |
| 分类 | flip 型（|d|=15 质量 ≥50% 且 sky 主导，或 total≥2048）= **238**；prop 型（小差值）= 210 |
| proxy2 聚类 | 仅 5 个连通片（sizes 302/128/12/4/2），**零孤立 chunk**——fallback 驱动的散点分布不成立 |
| proxy3 「自身≈vanilla 但邻 chunk 有残差」的片中心 | 存在（如 (14,-8) d=1 被 8192 级翻转邻居包围；(0,0) d=92 在 8192 环中心）——但片中心自身也是 flip 邻域，形态是「整片区域 sky 体系性偏 15」而非「干净孤岛」 |
| proxy4 翻转 section 的 Y 分布 | 前 80 个 flip chunk 的近全翻转 section 集中在 Y=4 附近（地表下方），非全高度均匀 |
| proxy5 最重 flip chunk (37,-18) 数值结构 | vanilla Y=4..6 sky 为 **heightmap 形梯度（0..14 连续值）**，rust 同 section 为**全 15**（4096/4096） |

注意 proxy5 的 rust 全 15 有歧义：可能是 rust 显式写了全 15，也可能是 SkyLight section 未序列化 + 对比脚本的缺键 fill=15 约定——两者都指向「rust 认为该柱全露天」，仍是计算层分歧，与传播无关。

## 3. H1 能否解释主导签名？

**不能。**
- 「整 section sky 0↔15 翻转 + inner 主导」是 heightmap/洞穴/遮蔽判定层的**计算分歧**（rust 把 vanilla 有梯度的封闭区域判成全 15），空间上成片、与 chunk 边界面无关、与 pregen 边界零相关。传播缺失（H1）的物理形态是：光从邻 chunk 传入不足 → 残差集中在**边界面、|d| 沿向内衰减、量级 ≤15 且多 ≤9**。proxy5 的全 15 vs 梯度结构与此完全相反。
- fallback 假设的地理前提也被证伪：neighbor-missing fallback 只可能发生在 pregen 边界，而残差 100% 在内部。

**H1 最多解释的部分**：210 个 prop 型小差值残差（如 (0,0) 92 处 |d|1-9）在类别上与传播缺失形态兼容；但其 sky 差值 face 份额仅 29.7%（边界面均匀基线 ≈23.4%，16×16 截面边框占比），**无界面富集**，所以连这部分也只是「未被证伪」而非「被支持」——更可能同属 rust 计算分歧的边缘表现。且 §1 的拉取语义分析本身也削弱了 H1 的机制前提。

## 裁决

**倾向：DENY（对 flip 主导签名）/ 对 prop 型小残差部分 UNCERTAIN-weak** —— 一行理由：**残差 448 个 0 个在 pregen 边界（fallback 只能在边界发生）+ vanilla propagateLight 是从共享存储拉取邻居数据而非等推送 + 翻转残差呈「rust 全 15 vs vanilla heightmap 梯度」的计算分歧结构，三者共同排除 H1 作为主导根因。**

建议：flip 型 238 chunk 的「封闭区域被 rust 判全露天」应另立候选（sky heightmap/maxSurfaceY/柱透明度判定的 rust 复刻分歧方向）；prop 型 210 chunk 保留为弱候选，需 LightStorage.enqueueSectionData 再传播语义源码核验后才能定。
