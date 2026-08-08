# #23/#24 biome-fix：SearchTree 移植 + find 平局诊断 —— 主会话应用手册

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> 角色：anchor.worker 精确分析 subagent（只读；产出 patch 代码，不直接改 src）
> 承接：`.artifacts/8576-24blocks/biome-fix/analysis3.md`（根因定论）
> 本文件：`patch-biome-find.md` —— 对 `versions/1.20.1/cpp/worldgen/src/biome.h` 的具体修改点
> 配套产物：`searchtree.h`（完整可编译头文件）、`patch-find-diag.diff`（方案 C 诊断独立 diff）
> 日期：2026-08-09　状态：**draft**（主会话应用 + 编译 + 四套参照回归后由主会话/审查决定提升）

---

## 0. 一句话总结

把 C++ `BiomeSource::find` 从「线性遍历 + 严格 `<`（等价 Java 测试用 getValueSimple）」替换为 **vanilla 运行时实际使用的 `MultiNoiseUtil.SearchTree`**（新文件 `searchtree.h`）：非平局时结果与现线性遍历**逐位一致**（同一最近邻），平局时按 Java `TreeBranchNode` **严格大于**语义取树序遍历第一个最小距离 leaf —— 修正 #23（forest→应为 badlands 系）/#24（带顶差 1）。

**距离公式无需修改**（已逐位核实一致，见 §3）；**唯一行为差异是平局 tie-break**。

---

## 1. 修改点清单（biome.h）

| # | 位置 | 改动 |
|---|---|---|
| 1 | 头部 include（L14 后） | 新增 `#include "searchtree.h"` 与 `#include <memory>`（`std::unique_ptr`） |
| 2 | `BiomeSource` private 区（L238-239 附近） | 新增 `mutable std::unique_ptr<SearchTree<std::string>> tree_;` + `searchTree()` 懒构建方法 |
| 3 | `find()`（L220-234） | 替换为 SearchTree 查询（见 §4 完整代码） |
| 4 | private 区 | 新增 `debugFindTop()`（方案 C 诊断，env `WG_FINDTOP` / `WG_FINDDUMP` 开关） |
| 5 | —（不变） | `loadFromJson` / `temperature` / `size` / `NoiseHypercube` / `noiseToLong` / `rangeDistance` 均不动 |

保留项：`find` 签名不变（`const std::string* find(float,float,float,float,float,float) const`），`-biomeDump` / `WG_BIOMEDUMP` 输出兼容（调用方 `worldgen_api.cpp:498/741` 只取值拷贝，返回 tree 内 value 指针语义等价）。

---

## 2. 惰性构建位置

`searchTree()` 为 private 方法，**首次 `find()` 调用时构建一次**（`mutable` 缓存），之后复用。与 vanilla `MultiNoiseUtil.Entries` 构造时建树（L100）相比是延迟构建，但**树内容只依赖 entries（loadFromJson 后固定）**，延迟不改变结果。若主会话更偏好 eager，可在 `loadFromJson` 成功返回前调用一次 `searchTree()` 预热（可选，非必需）。

---

## 3. 距离公式 / tie-break 对齐结论（已核实）

### 3.1 距离公式逐位一致（无需改）

| 项 | Java（MultiNoiseUtil.java） | C++（biome.h） | 判定 |
|---|---|---|---|
| toLong | `(long)(value * 10000.0F)` L66-68 | `noiseToLong(float v){ return (long)(v*10000.0F); }` L152 | ✓ 逐位一致（float 乘法 + 截断） |
| 区间距离 | `ParameterRange.getDistance` L362-366：`l=noise-max; m=min-noise; l>0?l:max(m,0)` | `rangeDistance` L166-170：`l>0?l:(m>0?m:0)` | ✓ 等价 |
| 超立方距离 | `NoiseHypercube.getSquaredDistance` L287-295：6 维平方和 + `square(offset)` | `NoiseHypercube::getSquaredDistance` L172-180：6 维平方和 + `offset*offset` | ✓ 逐位一致 |
| SearchTree 节点距离 | `TreeNode.getSquaredDistance` L590-598：7 维平方和（第 7 维 = `[offset,offset]` vs 点 0 → |offset|²） | `searchtree.h` `Node::getSquaredDistance` | ✓ 与超立方距离数学一致（offset 维贡献 = offset²） |

**结论：距离值在 C++ 线性 find / C++ SearchTree / Java 三者完全一致；修复只改 tie-break 选择，不改距离。**

### 3.2 tie-break（根因，唯一差异）

- **C++ 现状（错）**：`if (bestDist < 0 || dist < bestDist)` 严格 `<` → 平局保留**先遍历到**的条目（biome_params.json 中 forest 在 badlands 之前）→ forest。
- **Java SearchTree（对）**：`TreeBranchNode.getResultingNode` L549 `if (l > m)` / L552 `if (l > n)` 严格 `>` → 平局**不更新** → 返回**树序遍历第一个**最小距离 leaf；树序遍历序由 `createNode` 的排序决定（L404-449，与 entries 顺序无关）。
- 移植语义：`searchtree.h` `Branch::getResultingNode` 严格照搬 L541-560。非平局时二者都返回唯一最近邻，无差异。

---

## 4. 完整代码（替换/新增，可直接抄入 biome.h）

### 4.1 include（头部）

```cpp
#include "json.h"
#include "searchtree.h"   // 新增：MultiNoiseUtil.SearchTree 移植
#include <memory>         // 新增：std::unique_ptr
```

### 4.2 find() 替换（L220-234 → 下方代码）

```cpp
    // 六维噪声值 → 最近 biome id（等价 vanilla MultiNoiseUtil.SearchTree.getValue，L146-152）
    // 非平局 = getValueSimple（唯一最近邻，与旧线性 find 结果一致）；平局 = 树序遍历第一个最小距离 leaf（对齐 vanilla）
    const std::string* find(float temp, float hum, float cont, float ero, float depth, float weird) const {
        long t = noiseToLong(temp), h = noiseToLong(hum), c = noiseToLong(cont);
        long e = noiseToLong(ero), d = noiseToLong(depth), w = noiseToLong(weird);
        debugFindTop(t, h, c, e, d, w);   // 方案 C 诊断（env 开关，不改结果）
        long point[SearchTree<std::string>::DIM] = {t, h, c, e, d, w, 0L};
        return searchTree().get(point);
    }
```

### 4.3 private 区新增（在 `std::vector<BiomeEntry> entries;` 附近）

```cpp
    mutable std::unique_ptr<SearchTree<std::string>> tree_;   // 首次 find 懒构建

    // 懒构建 SearchTree（树内容只依赖 entries，构建一次后复用）
    const SearchTree<std::string>& searchTree() const {
        if (!tree_) {
            std::vector<SearchTree<std::string>::Entry> es;
            es.reserve(entries.size());
            for (const auto& e : entries) {
                SearchTree<std::string>::Entry entry;
                entry.parameters[0] = STRange{e.cube.tempMin,    e.cube.tempMax};
                entry.parameters[1] = STRange{e.cube.humMin,     e.cube.humMax};
                entry.parameters[2] = STRange{e.cube.contMin,    e.cube.contMax};
                entry.parameters[3] = STRange{e.cube.eroMin,     e.cube.eroMax};
                entry.parameters[4] = STRange{e.cube.depthMin,   e.cube.depthMax};
                entry.parameters[5] = STRange{e.cube.weirdMin,   e.cube.weirdMax};
                entry.parameters[6] = STRange{e.cube.offset,     e.cube.offset};   // 第 7 维 [offset,offset]，点第 7 维恒 0
                entry.value = e.id;
                es.push_back(std::move(entry));
            }
            tree_ = std::make_unique<SearchTree<std::string>>(std::move(es));
            // 默认关闭 previousResult 缓存（确定性，平局=树序遍历第一个）；WG_SEARCHTREE_CACHE=1 复刻 Java 缓存语义（A/B 对照用）
            tree_->setUsePrevious(getenv("WG_SEARCHTREE_CACHE") != nullptr);
        }
        return *tree_;
    }
```

### 4.4 方案 C 诊断（private 方法）

```cpp
    // 方案 C 诊断：验证平局。WG_FINDTOP=任意值 → 打印 Top3 距离+id（含平局标记）；WG_FINDDUMP=任意值 → 打印全量距离。
    // 不改变 find 结果；仅当 env 存在时走线性遍历开销（诊断用）。
    void debugFindTop(long t, long h, long c, long e, long d, long w) const {
        static const bool top  = getenv("WG_FINDTOP")  != nullptr;
        static const bool dump = getenv("WG_FINDDUMP") != nullptr;
        if (!top && !dump) return;
        struct Hit { long dist; const std::string* id; };
        Hit top3[3] = {{INT64_MAX, nullptr}, {INT64_MAX, nullptr}, {INT64_MAX, nullptr}};
        std::vector<Hit> all;
        if (dump) all.reserve(entries.size());
        for (const auto& entry : entries) {
            long dist = entry.cube.getSquaredDistance(t, h, c, e, d, w);
            if (dump) all.push_back({dist, &entry.id});
            for (int i = 0; i < 3; i++) {   // 稳定 Top3（相等保留先出现）
                if (!top3[i].id || dist < top3[i].dist) {
                    for (int j = 2; j > i; j--) top3[j] = top3[j - 1];
                    top3[i] = {dist, &entry.id};
                    break;
                }
            }
        }
        std::fprintf(stderr, "[FIND] point t=%ld h=%ld c=%ld e=%ld d=%ld w=%ld\n", t, h, c, e, d, w);
        if (top) {
            for (int i = 0; i < 3 && top3[i].id; i++) {
                const char* tie = (i > 0 && top3[i].dist == top3[0].dist) ? "  <== TIE with #1" : "";
                std::fprintf(stderr, "  #%d %-36s dist=%lld%s\n", i + 1, top3[i].id->c_str(), (long long)top3[i].dist, tie);
            }
        }
        if (dump) {
            std::sort(all.begin(), all.end(), [](const Hit& a, const Hit& b) { return a.dist < b.dist; });
            for (const auto& hit : all)
                std::fprintf(stderr, "  %-36s dist=%lld\n", hit.id->c_str(), (long long)hit.dist);
        }
    }
```

> 注：若主会话希望诊断输出与 `WG_BIOMEDUMP`（worldgen_api.cpp:493）协同，可在 `WG_BIOMEDUMP` 分支后追加调用 `debugFindTop`；但 `debugFindTop` 独立于 `WG_BIOMEDUMP`，单独设置即可。

---

## 5. 验证步骤（主会话）

1. **编译**：`searchtree.h` 放入 `versions/1.20.1/cpp/worldgen/src/`，biome.h 按 §4 修改，编译。
2. **诊断确认平局**：`WG_FINDTOP=1 block_probe (812,73,-337)`（或任意单点 probe）应看到：
   ```
   [FIND] point t=5500 h=-946 c=161 e=-4442 d=1039 w=-5418
     #1 minecraft:forest        dist=16406746
     #2 minecraft:badlands      dist=16406746  <== TIE with #1
   ```
   若 Top2 距离相等即验证根因；随后确认 SearchTree 返回 badlands（而非 forest）。
3. **四套参照全量回归**：`-288 / 3200 / 20000 / 8576`。**3200 必须保持 diff=0**（若出现平局点翻转，逐点对照 vanilla 参照确认）。
4. **8576 目标**：`(812,73,-337)` 从 stone 恢复为 terracotta（494），`(815,89,-337)` grass→terracotta、`(816,90,-337)` 对齐 vanilla。
5. **A/B 缓存实验（可选）**：`WG_SEARCHTREE_CACHE=1` 对比平局点行为；默认关闭。

---

## 6. 风险与影响面（3200 铁律）

| 维度 | 评估 |
|---|---|
| 修复对象 | 仅 `biome.h` find 查找路径（`BiomeSource::find`）；**不动** temperature/vegetation 噪声链路、选点（biomePickCell）、2D 分量值、surface 规则 |
| 受影响点 | 仅「距离完全相等」的平局点翻转（如 t=5500 落在 forest/badlands 公共边界）；远离边界点距离唯一胜者不变，**非平局与旧结果逐位一致** |
| 3200 当前 100% | 若 3200 无平局点 → 修复后仍 100%；若存在 C++ 误判平局点 → 修复使其向 vanilla 对齐（正确性提升）。**修复方向是向 vanilla 对齐，不会把已正确判定改错**，但必须全量回归验证 |
| `-biomeDump` / `WG_BIOMEDUMP` | 同一 find 输出随之改变（平局点）——预期行为，8576 参照将恢复 terracotta 带 |
| 性能 | 树构建一次 O(n·log n)（n≈千级），查询 O(log n) 远优于线性 O(n)；对 surface 逐块调用是净收益 |
| 线程 | `find`/`searchTree` 现为 const + mutable 懒构建，若未来多线程并发调用有 data race；当前 C++ 复刻单线程生成，无影响（文档记录即可） |

### 6.1 额外风险/验证项（本任务范围外，主会话应核实）

- **biome_params.json 条目数**：本 worker 观察到该文件 **1500+ 行**（13 个 weirdness 区间 × 各 temp/hum/cont/ero 组合 × depth 0/1 + ocean/cave），明显多于常见记忆中的 overworld preset 量级。SearchTree 移植对**任意 n 都正确**（算法通用），但若 biome_params.json 与 vanilla `VanillaBiomeParameters.writeOverworldBiomeParameters` 的实际输出**条目集合或顺序不一致**，则 C++ 的最近邻搜索空间与 vanilla 存在独立差异（非 tie-break）。建议主会话用 Java 运行时 dump 与 biome_params.json 做一次条目级 diff 确认（可在修复合入前先做，也可作为独立 issue）。
- **entries 顺序**：SearchTree 构建使用稳定排序，在「两个 entry 排序 key 完全相等」时保留输入顺序；C++ 输入顺序 = biome_params.json 顺序。若与 vanilla preset 顺序不同且存在 key 全等的相邻条目，平局结果可能微差（概率极低；本任务判定点不受影响——forest/badlands 的 key 不同）。

---

## 7. 产物引用

- 本文件：`.artifacts/8576-24blocks/biome-fix/patch-biome-find.md`
- 头文件：`.artifacts/8576-24blocks/biome-fix/searchtree.h`
- 方案 C 独立 diff：`.artifacts/8576-24blocks/biome-fix/patch-find-diag.diff`
- 根因定论：`.artifacts/8576-24blocks/biome-fix/analysis3.md`
- 运行时数据包：`.investigations/8576-24blocks/biome-fix-datapack.md`
- Java 参考：`versions/1.20.1/data/mc_src_extract/net/minecraft/world/biome/source/util/MultiNoiseUtil.java`（L379-604）
- 参数表：`versions/1.20.1/data/biome_params.json`
