# Q-AQ1 归因结论：Rust aquifer 段 35ms/chunk 的机制分解（260903-10）

- status: **confirmed**（260903-10 用户拍板；judge review-qaq1-260903-10 CONCERN 有条件通过，R1-R4 已应用）
- 验证分层：**机制 = Full（冷−暖差分 + 计数器 + 反证链 + b2 独立交叉）；量化 = Partial（R2 调和后三口径并存，见结论 1①）**
- §9.7 口径声明：载体 = 生产 fill 循环逐行镜像探针（qaq1_b1_prodfill_probe）+ 计数器（WG_AQUIFERBP/WL/SURF + GRID_ARG_SAMPLES）+ 冷路径微测（qaq1_r2_probe 双模式独立进程）；覆盖面 = seed 8576294172403134396 / region (200,200) / 8-16 chunks；可比性 = 与 Q-PD1 260903-09 stage 差分同族口径。

## 结论

1. **aquifer 段 35.07ms/chunk 分解**（量化数字为 R2 调和后口径）：
   - ① **~15.4ms：冷态 estimate_surface_height 全量扫描**——每 chunk 新建 Aquifer → surface_cache 冷 → 214 列 × 34.35 迭代 = 7342 次 initial_density 全价树采样。est 单价三口径并存（R2）：**2117ns/iter = 新鲜进程 diag 实测（生产相关口径，15.38ms/7265 iters，qaq1-r2-reconcile）**；3557ns = A 段最大抖动扫描形态（上界）；1646ns = 同进程 A/B 预热后的假冷（作废参考）。est 扫描解释 counter-free 冷态超额 **22.70ms**（qaq1_r2_probe fill 模式）中的 ~68%。
   - ② **~6-8ms：bp/wl miss 冷路径 + 杂项**（251 bp miss + 158 wl miss/chain、分配首触等）＝ 22.70 − 15.4 残差。
   - ③ 暖态 apply 内部 ~5.5ms（12 格 bp 邻域 + wl）；生产段 35.07 vs counter-free 探针冷态 33.67 的差含计数器开销与口径差（原「残差 ~2.9ms」表述按 R2 重述）。
2. **结构根因**：get_water_level_at miss（158/chunk）→ get_fluid_level 13 offset 列（横跨 5 个 x-chunk）→ est 列缓存冷 → init 树全价采样；Java 侧 est 有 chunk 级持久性（NoiseChunk 生命周期），Rust 每 chunk 丢弃。
3. **修复方向（另立优化包）**：A. est 查表化（init 树宏观粗化/per-chunk slices 复用，预期 −20ms+）；B. surface_cache 跨 chunk 持久化（次级）。
4. supersedes（原记录不改，双指针见各文件）：
   - 证据包 F5「initial_density 0.089µs/sample」——错误基线（探针 y 模式命中缓存所致），作废；本文 §结论 1① 取代。**R1 前向指针已回写 evidence-pack F4/F5。**
   - 证据包 F4 摘要漏收 t_fl 行——误导源成立，但 R3 修正：t_fl 暖态实测仅 4.44ms（非 b1 §4.3 猜的 25-30ms），漏项是**误导源而非缺口主体**；缺口主体 = 冷 est 扫描本身。
   - qaq1-b1-candidate §4「InterpolatedData 单槽抖动」机制——GRID_ARG_SAMPLES 双态增量 0 反证，作废（§4.2 凑数算术同时撤回）。**R4：qaq1-grid-thrash cmd-output 的 [判读] 行为预写模板未随实测（增量=0）更新，已作废声明（见 q-aq1-260903-10.md 更正节）；探针源码模板行已修正防再跑误导。**
   - 本文首版量化「26.1ms≈26.65ms 残差 0.6ms」——被 R2 调和取代（judge 指出选高口径未声明分歧 = N3 近亲）；现行口径见结论 1①。

## 证据链

- 冷−暖隔离：cmd-output/qaq1-b1-prodfill-260903-10.txt（T0-T5 + 计数器自检 vs F2 ✓）
- 反证：cmd-output/qaq1-grid-thrash-260903-10.txt（GRID=0）
- 闭环：cmd-output/qaq1-b1-coldpath-260903-10.txt（A=3557ns/sample；C cold 11.96/warm 4.28）+ **cmd-output/qaq1-r2-reconcile-260903-10.txt（R2 调和：fill cold 33.67/warm 10.97/excess 22.70 counter-free；diag-cold-fresh 2117ns/iter × 7265 = 15.38ms）**
- judge：.investigations/lossless-accel/review-qaq1-260903-10.md（CONCERN，R1-R4 已应用）
- 计数器：qaq1-counters / qaq1-surf-probe；过程：q-aq1-260903-10.md；候选分析：qaq1-b1-candidate-260903-10.md（§4→§5 supersedes 链）
- 工具：bin-diag qaq1_{counter,surf,apply_breakdown,b1_prodfill,grid_thrash,b1_coldpath}_probe（6 个，隔离区）
- 附带：pc_e2e_bench.rs seed 死参数修复（#20）已验证

## 新坑（错误台账候选，待 knowledge 草稿）

- N2：微测基线样本模式决定缓存命中率——「随机 y + 交替列」的 0.089µs vs est 扫描形态的 3557ns 差 40×；单点微测外推热路径成本必须复刻调用形态（与 #17 同族）。
- N3：自由参数凑数（b1 §4.2 用 158×重建×角点 凑 26ms）——量级核算每个乘数须有独立实测来源。
- N4：诊断证据摘要漏行（F4 漏 t_fl）制造「6× 缺口」假象——分解数据的**每一行**都进证据包。
- N5（b2）：多臂顺序 bench 的顺序效应制造假交互——多臂差分须 chunk 粒度交错 + 多轮。

## 未闭合遗留（不阻塞本结论）

- **b4（新课题候选）**：carver 机械成本列状态相关且反直觉——全 Air 列 carver 开启贵 22.97ms/chunk vs 实地形 10.62ms，机制未定位（uniform 路径下 WG_CARVERDIAG 死代码）；影响：A-off+carver-on 诊断配置被 +12~23ms 污染，后续 aquifer 实验一律 carver 双臂同关。详见 qaq1-b2-candidate-260903-10.md。
