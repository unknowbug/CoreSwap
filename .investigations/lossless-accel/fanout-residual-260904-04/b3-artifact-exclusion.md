# b3 候选：对比/坐标伪影排除分析（fanout-residual-260904-04）

> 状态：**draft**（core.worker 分析草稿，只读推理，未运行任何脚本；待 judge 审查）
> 输入数据（全部已读原文）：`.tmp/p2full/cmp_yhist-260904-04.txt`、`cmp_spatial-260904-04.txt`、`cmp_full2.out.txt`、`blocks.json`（id→name 映射）
> 候选假设：残差 13328/1572864（99.1526% 一致）由对比/坐标伪影造成（参照文件错位 / chunk 配对错误 / 探针坐标 bug）。参考 knowledge/discovered/workflow-patterns.md 发现 #13/#17 与 AGENTS.md 探针三查铁律。

## 1. 对账结果（伪影检查第一步）

### 1.1 per-chunk 计数 ↔ 总和对账（精确对上）

cmp_full2.out.txt L5 per-chunk 与 cmp_spatial-260904-04.txt 每 chunk `cells=` **逐值一致**：

| chunk 列 | 计数 | | chunk 列 | 计数 |
|---|---|---|---|---|
| (12,12) | 858 | | (14,12) | 633 |
| (12,13) | 1045 | | (14,13) | 868 |
| (12,14) | 1347 | | (14,14) | 959 |
| (12,15) | 964 | | (14,15) | 713 |
| (13,12) | 584 | | (15,12) | 450 |
| (13,13) | 850 | | (15,13) | 884 |
| (13,14) | 1269 | | (15,14) | 668 |
| (13,15) | 822 | | (15,15) | 414 |

行和：4214 + 3525 + 3173 + 2416 = **13328** = cmp_full2 L4 总 mismatch。✅ 对账通过。

yhist 32-band 粗带和：1041+1104+1091+1094+1081+1144+1100+1106+1116+1210+1174+1067 = **13328**。✅ y 轴覆盖 -64..319 共 384 行完整。三方（total / per-chunk / per-y / spatial）口径互洽，无丢数据、无重复计数。

### 1.2 header / seed / origin sanity

cmp_full2 L1-L3：ref 与 cpp 两侧 `seed=8576294172403134396 origin=(200,200) size=4 min_y=-64 h=384 chunks=16` 完全一致，`[sanity] header + layout match: True`。符合 AGENTS.md 探针三查铁律第 1/3 条（seed 一致、header 完整、chunk 范围在预期内、TOTAL=0.85% 合理——非 90% 级异常信号）。✅

### 1.3 blob 结构 vs 列错位签名（不相容）

若为列错位/chunk 配对错误类伪影，应观察到：
- **chunk 配对错位**（如 (12,13) 对到 (12,12)）：整 chunk 级大面积 mismatch（~10^4-10^5 cells/chunk，且相邻 chunk 数值相关）。实测最大 chunk = 1347（全 chunk 98304 cells 的 1.4%），无任何 chunk 接近整块错位量级。❌
- **列错位（x/z 偏移 1-N 列）**：mismatch 集中在地形轮廓表面（竖直方向连续长条），y 分布应强烈偏向 surface 带且与地形高度相关。实测 yhist **全 384 个 y 值近乎均匀分布**（每 y 27-46，粗带 1041-1210，极差仅 ±8%），deep(<-32) 带 2145 与 surface 带同量级——没有「轮廓状」集中。❌
- 连通域画像：各 chunk 30-110 个 comp，size-hist 以 1-10 cells 的小 blob 为主、少量 100-600 的大 blob，大 blob 也是「stone→dirt 为骨架 + gravel→sand/dirt 混入」的**成分混合体**（非单一对），无 chunk 边界 / x≡0 或 z≡15 列集中迹象（top-comp y-range 跨越 chunk 内部大段，如 (13,13) n=527 y=-32..319 横跨整个 chunk 高度）。❌
- **id 映射表错读类伪影**：应表现为单一主导对（全量 X→Y）。实测为**多对、双向**混合（stone→dirt 4165 且 sand→dirt 399、stone→gravel 355、gravel→sand 1687、water→air 520 且 air→water 170），两方向都存在，与「同一块被两套 id 表读出不同名」的单向系统偏移签名不符。❌

**判定：b3 伪影候选排除**（排除依据为数据内部自洽性 + 签名不相容；confidence: 该排除本身 candidate 级——纯静态推理，未做独立重采样，但三方对账 + 双向混合签名足以排除全部三类伪影机制）。

## 2. 反向画像：净置换方向（ref→cpp）

blocks.json 映射：0=air, 1=stone, 2=granite, 4=diorite, 6=andesite, 8=grass_block, 9=dirt, 32=water, 34=sand, 37=gravel, 43=coal_ore, 251=clay, 970=deepslate。

top20 对（覆盖 12529/13328 = 94%）分方向合计：

| 组 | ref 侧损失 | cpp 侧净得 |
|---|---|---|
| 石族（stone 6502 / gravel 3482 / granite 654 / diorite 524 / andesite 359 / coal_ore 158 / deepslate 116） | **≈11795** | — |
| dirt | — | 6898（←stone 4165/gravel 1359/granite 439/diorite 331/andesite 205/sand→dirt 399） |
| sand | — | 3830（←gravel 1687/stone 1581/granite 215/diorite 193/andesite 154） |
| water | −520（water→air） | +387（←air 170、←stone 217）→ 净 **−133** |
| air | −170 | +726（←water 520、←deepslate 116、←stone 90）→ 净 **+556** |

**净方向画像**：
1. **主通道 = cpp 把 ref 的 stone/gravel/granite/diorite/andesite（石族）替换成 dirt/sand（净 +~10.7k vs 石族 −~11.8k，94% 覆盖内）**——即 **C++ 侧 surface 规则把本应保留为 stone（或 Java 侧应放 gravel/disk 的位置）多执行了 dirt/sand 置换**；ref 侧的 gravel→sand/dirt 说明 Java 放 gravel 的位置 C++ 放成 sand/dirt（disk/表面染色目标块或随机流错位）。
2. 次通道：water 净 −133（water→air 520 vs 反向 287），air 净 +556——洞穴带局部液面/空气边缘效应，量级小（<5%），非主矛盾。
3. 少量 ore 丢失（coal_ore→stone 158）与 clay 换入（37→251, 81）。

**机制约束（供收敛方使用）**：净方向 = cpp 多替换石族→dirt/sand ⇒ **指向 C++ 侧 surface builder 多触发（或随机源错位导致阈值判定偏移），以及 gravel/sand/dirt 染色分支的目标块选择或判定顺序与 Java 不一致**——**不指向 aquifer floodedness 流体判定缺失**（那会给出 water 净大幅单向减少 + stone→water 主导，实测 water 净仅 −133 且 stone→water 仅 217，非主对）。Java 侧特征丢失（如 disk 放置缺失）方向为 ref→cpp 出现 stone，与实测主方向（ref=stone→cpp=dirt/sand）相反，同样排除。

## 3. id 误读历史核对（§15.4 取代条目清单）

blocks.json 实测：`minecraft:dirt: 9`、`minecraft:water: 32`。上轮记录「ref=1→9(water)」**实为 stone→dirt**，water 的 id 是 32。误读出处（grep 已核实，仅存在于 10-timewise-archive.md）：

需按 §15.4 结论取代链（supersedes 双指针 + 一行推翻理由，原文不删不改）处理的位置：

| 位置 | 原文内容 | 错误点 |
|---|---|---|
| `versions/1.20.1/docs/10-timewise-archive.md` **L1200** | 「top 互换对（ref→cpp）：**1(stone)→9(water)** n=4165 …主体为 stone/deepslate 族 ↔ water/air 互换，**aquifer/深板岩族特征**（液面判定与深板岩分层交界处方块身份互换…）」 | id=9 是 dirt 非 water；整句「↔ water/air 互换 / aquifer 特征」签名判断随 id 误读失效 |
| `10-timewise-archive.md` **L1206**（下轮候选 1） | 「aquifer floodedness 判定输入/语义差（**stone→water 单向占大头**，指向流体判定未触发…）」 | 不存在 stone→water 单向大头（实际 4165 是 stone→**dirt**；stone→water 仅 217）；候选 1 的先验依据失效 |
| `10-timewise-archive.md` **L2844** | 「**stone↔water 等互换模式**（ref=1→9 n=4165 等 top 对）…机制候选：aquifer floodedness/流面高度/表面级联」 | 同一误读的时间线副本；「stone↔water」应更正为「stone→dirt（表面置换差）」 |
| `10-timewise-archive.md` **L1201/L1208** | est 无关性结论 + 下轮「残差 y 分布直方图先行」 | est 无关结论本身不依赖 id（仍有效，无需取代）；仅 L1206-1208 的机制候选 1 需随之降权/取代 |

`versions/1.20.1/docs/07-block-pipeline.md`：grep 全文无「ref=1」或残差 id 对记录（其 aquifer 小节为 -288 课题与性能课题，与本误读无关）——**07 篇无需取代条目**。

**取代要点（供主会话应用）**：supersedes 指向上述 10 篇条目；一行推翻理由 = 「id 映射误读：9=dirt 非 water（blocks.json 实测）；残差主签名应为 stone/gravel 族→dirt/sand 的 surface 置换差，非 aquifer 流体互换；净方向指向 C++ surface 规则多触发/染色分支目标块差，aquifer floodedness 候选降权」。

## 4. 结论

1. **对账全通过**：total=13328 = per-chunk 16 项和 = yhist 粗带和；两侧 header/seed/origin 一致 → 数据本身可信。
2. **b3 伪影判定：排除**。chunk 配对错误（量级不符）、列错位（y 分布均匀无轮廓集中）、id 表错读（多对双向混合）三类签名均与实测不相容。
3. **净置换方向**：cpp 比 ref 多 dirt/sand（+~10.7k），石族少（−~11.8k）→ 指向 C++ surface 规则多触发/染色目标块差；water 净仅 −133，aquifer floodedness 方向不成立。
4. **需取代条目**：10-timewise-archive.md L1200、L1206、L2844（L1201 est 无关结论保留）；07 篇无。
5. 置信度：**draft**（静态推理 + 数据内部对账，无独立重采样运行；judge 审查后可议 candidate）。
