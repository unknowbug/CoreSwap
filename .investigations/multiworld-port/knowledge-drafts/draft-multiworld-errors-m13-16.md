# 草稿：multiworld-errors.md M14/M15 追加 + discovered 新条目 + M13 修正

> 产出者：knowledge 落盘 subagent（2026-08-31，session 尾声批次）。status: candidate。
> 素材来源：主会话 session 尾声实测记录（F3 截图 / 三方 chunk 对照 / CountingAlloc 现象）。
> （注：原批次含 M16 思维链元教训，用户明确取消——模型层面问题、换 session 唯一解，不产出。）
>
> **主会话应用方式**：
> ① 「A 部分」整段插入 `multiworld-errors.md` 的 **M13 节之后、`## 附：错误 → 根因 速查表` 之前**；
> ② 「B 部分」表格行追加到速查表**末尾**；
> ③ 「C 部分」按内附说明应用到 `knowledge/discovered/workflow-patterns.md` + `knowledge/INDEX.md`；
> ④ 「D 部分」对 M13 节做一处定点 edit（补一句效果边界说明）。

---

# A 部分：multiworld-errors.md 追加正文（M14/M15 两条，插 M13 之后、速查表之前）

---

## M14.【开放问题·本轮核心】实机下界「怪异城+虚空」：feature 装饰阶段的 biome 上下文错乱

### 现象
- 实机下界（M13 buffer 修复 + M12 STATE_BY_ID 修复之后）依然异常——用户实机验证（F3 截图）：**Biome=soul_sand_valley 判定正确**，但地形大片缺失（视觉真空 + 物理坠落）。
- chunk(-5,-3) 存档导出统计：**oak_leaves ×3150 + jungle/acacia/dark_oak_sapling + note_block** 成片出现在下界 chunk——**主世界森林的树 feature 块**。用户称为「怪异下界城」的实体就是这些错乱块。

### 根因（机制，定位到方向、机制待查）
- **三方对照（同 seed=-2032795982907864146 同 chunk）**：vanilla 导出 vs Rust fill **高度一致**（bedrock/blackstone/涂布/空腔大格局对齐，残差仅 soul_soil/soul_sand 涂布分布与 cave_air 分布微差）；**存档数据与两者都不同**（橡树海洋）→ 错乱块不来自 Rust fill，来自 **vanilla 的 feature 装饰阶段**（在 Rust fill 之后由 vanilla 运行）。
- 机制方向：mixin 接管 populateNoise 后，**vanilla 的 applyBiomeDecoration 对该下界 chunk 使用的 biome/feature 上下文被污染**——拿到了主世界森林的 feature 集（橡树树叶 = 树 feature 产物）。
- 候选（下轮审计 applyBiomeDecoration 的 biome 上下文）：① chunk 的 biome 属性在 fill 后未刷新；② NoiseConfig 的 climate 采样状态；③ feature 阶段的 biome source 依赖。

### 定位（诊断方法）
1. **三方对照隔离产物归属**：vanilla 导出 / Rust fill / 实机存档三方同 seed 同 chunk 对比——存档与 vanilla、Rust fill **都**不同 → 错乱块必在「vanilla 后续阶段」，排除 Rust fill 自身。这是「mod 接管后异常归因」的关键一步：先把嫌疑从自己人身上洗掉。
2. **存档块统计找指纹**：oak_leaves ×3150 + 多种 sapling 是「森林 feature 集」的签名性组合，直接指向 feature 装饰阶段拿错 biome，而非噪声/表面层。
3. F3 biome 判定正确 + 地形错乱并存 → 排除「biome 判定算法错」，锁定「biome 上下文传递链」。

### 修复
- **未修**。下轮开工点：审计 applyBiomeDecoration 的 biome 上下文（候选三选一消融验证）。

### 教训（⚠️ 重点：开放问题 + 可复用判错经验）
- **接管世界生成的一个阶段（populateNoise）后，后续 vanilla 阶段（feature 装饰）的上下文依赖会暴露**——mod 接管世界生成必须审计**全管线的阶段依赖**，不只是被替换的那个阶段（已立 discovered 条目，见本草稿 C 部分）。
- **三方对照是 mod 接管场景的归因利器**：vanilla 导出（基线）+ 自家输出（被怀疑方）+ 运行时实况（集成结果）三方对齐情况直接分派责任——「实况与两个静态产物都不同」= 中间被第三方阶段改写。
- **视觉+物理直觉胜过长推理链**：视觉真空 + 物理坠落两个体感现象 = 数据层真空实锤（用户的直觉判断比推理链先到）。

---

## M15. 诊断工具的性能陷阱：CountingAlloc 全局原子计数在多线程 fill 下灾难性慢

### 现象
- 为内存诊断给 dll 挂全局分配计数器（GlobalAlloc 包装 + 全局 AtomicI64 计数）后，删档重生成的 chunk 生成**灾难性变慢**——用户感知「卡死」，被迫强杀进程。

### 根因（机制）
- 全局 AtomicI64 的 alloc/dealloc 每次 RMW（read-modify-write）——**23 线程并发 fill 的分配密集场景下成为序列化点**：每次内存分配都要抢同一把原子「锁」，多线程并行度被分配计数归一。
- 本质与「测量/探针污染铁律」（AGENTS §四，WG_PROFILE/WG_STAGETIMER 家族）同族：**诊断工具改变被诊断系统的行为**——但层级更深，这次污染的不是计时精度，是执行并行度本身。

### 定位（诊断方法）
- 挂计数器前后行为对比：计数器一挂即「卡死」、一移除即恢复 → 变量唯一，直接归因（诊断代码在热路径的每分配执行 + 全局共享 = 唯一嫌疑）。

### 修复
- 已移除计数器（性能恢复）；结论记录在案。

### 教训（可复用判错经验）
- **全局分配计数器 = 分配序列化**，多线程高频分配场景**禁用**（或 thread_local 计数 + 定期汇总——把 RMW 竞争从每次分配降为每汇总周期一次）。
- **「诊断工具改变被诊断系统的行为」的极端形态**：不只污染测量值，直接改变执行特性（并行度归一/卡死）——热路径诊断代码必须 chunk 级判断一次或编译期 feature gate（与「端到端性能对比铁律」的诊断门控条款同族）。

---

# B 部分：速查表追加（2 行，插表末）

| 实机下界「怪异城」：soul_sand_valley 判定正确但地形缺失，存档 chunk 出现 oak_leaves×3150+多 sapling（M14【开放问题】） | mixin 接管 populateNoise 后，vanilla feature 装饰阶段（applyBiomeDecoration）对该 chunk 的 **biome/feature 上下文被污染**（拿到主世界森林 feature 集）；三方对照（vanilla 导出 vs Rust fill 高度一致 vs 存档两者皆不同）锁死错乱块来自 vanilla 后续阶段；机制待查（chunk biome 属性未刷新 / NoiseConfig climate 状态 / feature 阶段 biome source） | **接管一个阶段后，后续阶段的上下文依赖会暴露**——mod 接管世界生成必须审计全管线阶段依赖（discovered 发现 #8）；**三方对照归因**：实况与两个静态产物都不同 = 中间被第三方阶段改写；视觉+物理直觉（真空+坠落）优先于长推理链 |
| 挂全局分配计数器后多线程 fill 灾难性变慢「卡死」（M15） | 全局 AtomicI64 每次 alloc/dealloc RMW，23 线程分配密集场景成为**序列化点**——并行度被分配计数归一；已移除恢复 | **全局分配计数器 = 分配序列化，多线程高频分配禁用**（或 thread_local 计数+定期汇总）；「诊断工具改变被诊断系统」的极端形态：污染的不是测量值而是执行并行度——热路径诊断必须 chunk 级一次判断或编译期 gate |

---

# C 部分：discovered 新条目（通用模式，跨项目价值）

## 应用说明

- 写入 `knowledge/discovered/workflow-patterns.md`（追加「## 发现 #8」，该文件当前最大为 #7）；
- `knowledge/INDEX.md` 分类入口「工作流模式」行的说明列末尾追加「、接管单阶段后的后续阶段上下文依赖（2026-08-31）」。

## 发现 #8: 接管世界生成单阶段后的后续阶段上下文依赖（Minecraft modding 通用）

- **发现时间**：2026-08-31；**发现者**：multiworld-port session（M14）；**来源定位**：`.investigations/multiworld-port/multiworld-errors.md` M14；**置信度**：candidate（现象三方对照实锤，机制方向待查）；**module**：workflow / Minecraft modding。
- **观察**：mixin/注入接管世界生成管线的一个阶段（如 populateNoise/NOISE）后，后续阶段（feature 装饰 applyBiomeDecoration / SURFACE / lighting）对被接管阶段的**上下文依赖**会暴露——本例：Rust fill 的下界地形与 vanilla 高度一致，但 vanilla 后续 feature 装饰拿到的 biome/feature 上下文被污染（主世界森林的树 feature 铺满下界 chunk）。
- **证据**：三方对照（vanilla 导出 vs Rust fill 一致 vs 实机存档橡树海洋）锁死错乱块来自 vanilla feature 阶段而非自家 fill；F3 biome 判定正确排除判定算法，锁定上下文传递链。
- **如何利用**：
  - **审计清单**：被接管阶段之后的**每个 vanilla 阶段**，其输入依赖是否仍满足——biome 上下文（chunk biome 属性在 fill 后是否刷新）、NoiseConfig 状态（climate 采样）、chunk Status 推进（**Status 不推进会导致 chunk 永不重生成**）、高度图依赖。
  - mod 接管世界生成的验收不能只验「被替换阶段的输出正确」，必须端到端验收运行时存档（实况含全部后续阶段产物）——单阶段对拍全绿 ≠ 集成正确。
  - 同族风险：任何「替换框架管线一段」的 mod 模式（不只是 worldgen：事件接管、渲染 pass 替换）都要问「下游阶段吃我什么状态」。

## INDEX.md 应用后该行变为

| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新）、接管单阶段后的后续阶段上下文依赖（2026-08-31） |

---

# D 部分：M13 修正补遗（定点 edit）

**位置**：`multiworld-errors.md` M13 节「### 效果」小节末尾追加一句：

> - ⚠️ **效果边界（2026-08-31 补充）**：buffer 修复后实机下界的结构性块（bedrock/blackstone/netherrack）与 vanilla 对齐 ✓，但「怪异城」（feature 错乱，→M14）与「全 air chunk」（JNI buffer 越界，已修）是**三个独立问题**——M13 修复只解决了第一个问题的解析缺失部分，最终下界虚空问题的完整闭环见 M14。

---

# 自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：M14（错误链条+判错方法，高价值必记）/ M15（反模式+环境坑，高价值必记）/ discovered #8（跨项目通用模式，中高价值）——无低价值条目
- [x] 三条新错误均五段式完整，根因为机制层面非现象复述
- [x] 定位含诊断方法（三方对照 / 变量唯一归因）
- [x] 判错经验已沉淀（可复用判据入教训段+速查表）
- [x] 被排除假说标注（M14 排除 biome 判定算法错 ❌，保留在正文）
- [x] 载体正确：错误→multiworld-errors.md 台账；通用模式→discovered/workflow-patterns.md（发现 #8 格式对齐现有条目）；INDEX 同步行已备
- [x] 速查表追加 2 行（M14/M15）
- [x] 数字来自主会话提供的实测记录（oak_leaves×3150、chunk(-5,-3)、seed、23 线程），无编造
- [x] 格式与目标文件末尾现状对齐（M13 节之后插正文、速查表末尾加行、发现 #8 接 #7 序号）
- [x] M16 按用户指示取消，未产出
