# 草稿：Rust worldgen 端到端性能定位（vs Java）—— 结论 docs 草稿

> **用途**：这是给主会话的 **结论 docs 草稿**（subagent 产出，主会话应用 + 验证）。本文件是「草稿容器」，非最终落盘载体。主会话按以下「载体映射」把各段落到位到对应知识库文件，并做一致性扫描验证。
>
> 错误台账本体已完成于 `.investigations/perf-e2e/perf-e2e-errors.md`（P1-P3 五段式 + 速查表）。
>
> **载体映射**（按「记录价值门」分层）：
> - **性能定位结论（中价值）** → `versions/1.20.1/docs/07-block-pipeline.md` 末尾追加「Rust worldgen 端到端性能定位」小节（架构/采样优化方向，值得简记）。
> - **端到端修正（低价值快照 + 高价值教训）**：Java 8-9ms / Rust 44.9ms 的**当前对齐数字本身是低价值快照**（一次性数值，会随优化变化）；但「积累性差异」**教训高价值**——已写入 AGENTS.md「端到端性能对比铁律」，并在 error 台账 P3 完整沉淀；07 篇性能小节只记结论性要点，不复述一次性数值细节。
> - **过程/中间结论** → `versions/1.20.1/docs/10-timewise-archive.md` 追加 2026-08-29 条目。

---

## 一、07 篇末尾追加小节草稿（中价值 · 简记）

> 追加位置：`versions/1.20.1/docs/07-block-pipeline.md` 末尾（L783 后）。
> 状态标注：candidate（主会话应用后经 judge / 验证才可 confirmed）。

```markdown
## 2026-08-29 Rust worldgen 端到端性能定位（aquifer 是最大头，整体慢 Java 5 倍）

> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链闭合后进入性能定位。本小节记性能定位**结论与优化方向**（中价值）；错误链条（双层 Interpolated 污染 / 诊断热路径污染 / Java 基准未热）见 `.investigations/perf-e2e/perf-e2e-errors.md`（P1-P3）。

### 端到端对比（Java 充分预热）

- **Java 原版（WorldGenBench FULL 含树花植被，充分预热 JIT）**：稳定后 ~8-9ms/chunk（排除首个 298ms 冷启动）。
- **Rust（fill_chunk_blocks 无树花，清理诊断污染后）**：44.9ms/chunk → **慢 ~5 倍**。
- ⚠️ 早期「Java 60ms」是 **JIT 未热**的错误基准，据此误判 Rust 达标；真实 Java 只要 8-9ms，Rust 远未达标。**端到端必须对比充分预热的 Java**（AGENTS.md「端到端性能对比铁律」）。

### 无污染重定位：fill_chunk+surface base 29.4ms 内部构成（region 200,200 单线程）

| 组成部分 | 增量 | 占比/备注 |
|---|---|---|
| **aquifer（含水层 classify）** | **~17.5ms** | **60%（最大头）** |
| density（finalDensity 采样，含内部 Interpolated 网格首建） | ~12ms | 次大头 |
| carver 14ms / surface ~4ms | 14 / 4 | carver 属完整管线阶段 |

注意：此为本轮「无污染重定位」数字（诊断热路径污染清理后）。早前 `base_breakdown.txt`（P 污染前）的 29.8ms density / aquifer +10.5ms 为**污染态中间值**，勿再作当前性能引用。

### aquifer 内部 profile（aquifer_internal_profile，4 chunks）

| 部分 | 耗时/chunk | 占比 |
|---|---|---|
| **calculate_density** | 19.68ms | **52%（最大头）** |
| get_block_pos（3×3 邻域，有 block_positions 缓存） | 5.30ms | 14% |
| get_water_level_at | 0.89ms | 2% |
| apply 总 | 38.02ms | 占 wall 48% |

- **calculate_density 是 aquifer 内部最大头**：barrier.sample（1 个 3D Noise 节点，无 Cache2D 缓存）最多 3 次调用 + fluid 逻辑 + 算术/分支。
- get_block_pos 有 block_positions 缓存但 3×3 邻域采样仍占 14%。

### 优化方向（candidate）

1. **aquifer 的 barrier.sample 跨点缓存 / 减少 calculate_density 3 次调用 / fluid 逻辑优化**——aquifer 是 base 内最大单头（17.5ms，60%），一个 aquifer 就比 Java 全部（8-9ms）慢 2 倍。
2. **density**：单层 Interpolated 对 SplineDF 实测加速 70×（83.74→1.19ms，judge 已验证），是密度优化正解；与 AGENTS.md 铁律（SplineDF 树遍历 = 慢根源 → C2ME/DFC 直排 + 网格缓存）一致。需单层生产化验证（fill_chunk 场景跨 chunk 网格复用降每 chunk 网格首建）。
3. **carver / surface**：相对小头，后置。

### 域/边界

- 验证分层 = Partial（非逐位）；对齐率/数值为当前快照，随优化变化。
- 端到端对比必须用**充分预热的 Java 基准**（见 AGENTS.md 铁律 + error 台账 P3）。
- 数据源：`cmd-output/e2e_java_vs_rust.txt`、`cmd-output/aquifer_internal_profile.txt`；错误台账 `.investigations/perf-e2e/perf-e2e-errors.md`。
- 排除清单：❌「Interpolated 方向不可用（慢 100×）」（双层污染假象，见 P1）；❌「Rust 已接近/达标」（Java 未热错误基准所致，见 P3）。
```

---

## 二、10 时间线追加条目草稿（中价值 · 简记）

> 追加位置：`versions/1.20.1/docs/10-timewise-archive.md` 末尾（L2160 后）。

```markdown
## 2026-08-29 Rust worldgen 端到端性能定位（aquifer 最大头，慢 Java 5 倍）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节 + `.investigations/perf-e2e/` + `perf-e2e-errors.md` 错误台账（P1-P3）。

### 🔍 一、density 方向修正（11f478f，judge 推翻）
- `density_tree_profile`：finalDensity 3710 节点（无指数膨胀，Spline 仅 9；Wrapping 727 + ShiftDF 708 占 ~40%）。
- 原判「裸 density 6.4ms / Interpolated 632ms → 放弃」被 judge 推翻（`review-density-candidate.md`）：632ms 是**双层 Interpolated 污染**（final_density 内部已含 interpolated，外层再包一层 → 内层 mesh 跨 chunk 雪崩重建 291×=175 万次/chunk）。
- judge 实测单层 Interpolated 对 SplineDF = **70× 加速**（83.74→1.19ms），Interpolated 不应放弃，反是密度优化正解。

### 🔍 二、fill_chunk 内部定位（597e8d5）
- classify(aquifer) 43-64% 是 fill_chunk 最大头（有污染但相对占比可信）。

### ❌ 三、诊断代码热路径污染（P2，d9ff1e2）
- AQPROF atomic load / `Instant::now` ×3 / env::var 每点执行（98304 次/chunk）→ **27% 退化（61.5→44.9ms）**，用户提醒「断点污染」坑后迁移到 chunk 级门控。

### ❌ 四、端到端基准重大修正（P3）
- 早期「Java 60ms」是 **JIT 未热**错误基准 → 误判 Rust 达标（「积累性差异」担忧）。
- 充分预热后 **Java FULL 只要 ~8-9ms/chunk**，Rust 44.9ms **慢 ~5 倍**。

### ✅ 五、无污染重定位（本轮）
- base（fill_chunk+surface）29.4ms：**aquifer 增量 ~17.5ms（60%）> density ~12ms > carver 14ms > surface ~4ms**。
- 一个 aquifer（17.5ms）就比 Java 全部（8-9ms）慢 2 倍——优化应聚焦 aquifer。
- aquifer 内部：calculate_density **52%**（barrier.sample 无 Cache2D 缓存 + fluid 逻辑 + 最多 3 次调用）、get_block_pos 3×3 邻域 14%、get_water_level_at 2%。

### 🧰 六、铁律沉淀（用户拍板）
- AGENTS.md 新增「端到端性能对比铁律」（必须端到端对比充分预热的 Java；诊断代码不能放热路径每点执行）。

### 📌 记录指引
- 错误台账：`.investigations/perf-e2e/perf-e2e-errors.md`（P1-P3 五段式 + 速查表）。
- 结论：07 篇「Rust worldgen 端到端性能定位」小节（中价值）。
- 过程：本节 + `.investigations/perf-e2e/`。
- **域边界**：端到端数字 = Partial 快照（随优化变化）；优化方向（aquifer barrier 缓存 / 单层 Interpolated）/ DFC = candidate 待立项验证。
```

---

## 三、主会话应用清单（自检）

- [ ] 07 篇末尾追加「Rust worldgen 端到端性能定位」小节（§一 草稿，中价值简记）。
- [ ] 10 时间线末尾追加 2026-08-29 条目（§二 草稿）。
- [ ] 端到端一次性数值（Java 8-9ms / Rust 44.9ms）在 07 篇**只记结论性要点**，不复述会随优化变化的快照细节（低价值快照不写 docs 细则）。
- [ ] 「积累性差异 / 端到端对比 / 诊断热路径污染」教训已由 AGENTS.md 铁律 + P1-P3 错误台账承载，主题篇不再重复长链条。
- [ ] 应用后跑一致性扫描（确认无时间线式章节误入主题篇；数字与 cmd-output 记录一致）。
- [ ] 结论 candidate → 主会话验证 / judge 审查 → 用户拍板 confirmed 后方可标 confirmed（主会话职责，非本草稿）。
