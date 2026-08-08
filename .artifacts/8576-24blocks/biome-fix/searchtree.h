// searchtree.h — MultiNoiseUtil.SearchTree 的 C++ 移植（平局 tie-break 对齐 vanilla）
// ---------------------------------------------------------------------------
// Java 参考（权威，运行时实际）：
//   versions/1.20.1/data/mc_src_extract/net/minecraft/world/biome/source/util/MultiNoiseUtil.java
//     - SearchTree.create                L388-402
//     - SearchTree.createNode            L404-449   （小树排序 + 大树批量递归）
//     - SearchTree.sortTree              L451-459   （多键稳定排序）
//     - SearchTree.createNodeComparator  L461-467   （(min+max)/2，可选 abs）
//     - SearchTree.getBatchedTree        L469-487   （batch = 6^floor(log6(n-0.01))）
//     - SearchTree.getRangeLengthSum     L489-497
//     - SearchTree.getEnclosingParameters L499-518
//     - SearchTree.get                   L520-526
//     - TreeBranchNode.getResultingNode  L541-560   ★平局语义：if (l > m) / if (l > n) 严格大于
//     - TreeLeafNode.getResultingNode    L572-576   （直接返回 this）
//     - TreeNode.getSquaredDistance      L590-598   （7 维，含 offset 维；点第 7 维恒 0）
//
// 与 C++ biome.h 现有 NoiseHypercube::getSquaredDistance（6 维 + offset*offset）数学逐位一致：
//   SearchTree 节点第 7 维 = [offset,offset]，点第 7 维 = 0 → 该维距离 = |offset - 0| = |offset|，平方 = offset²。
//
// 用法（由 biome.h 的 BiomeSource 懒构建，见 patch-biome-find.md）：
//   std::vector<SearchTree<std::string>::Entry> es;
//   ... 填充 es（7 维参数区间 + biome id）...
//   auto tree = std::make_unique<SearchTree<std::string>>(std::move(es));
//   long point[7] = {t,h,c,e,d,w,0};
//   const std::string* id = tree->get(point);   // 非平局 = getValueSimple（唯一最近）；平局 = 树序遍历第一个最小
//
// 平局语义（关键）：
//   - Java TreeBranchNode 用「严格大于」更新（l > m / l > n），平局不更新 → 返回树序遍历中第一个达到最小距离的 leaf。
//   - 树序遍历序由 createNode 的排序决定（与 entries 原始顺序无关），等价 vanilla 运行时。
//   - 这与 C++ 现有 find 的线性「dist < bestDist 严格小于 → 取 entries 首个」平局语义相反（根因）。
//
// previousResult 缓存（Java ThreadLocal previousResultNode，L382/L522-524）：
//   - 默认关闭（usePrevious_=false）：每次查询 l=Long.MAX_VALUE 起算，结果确定、与查询顺序无关。
//   - 打开（setUsePrevious(true)，env WG_SEARCHTREE_CACHE）：复刻 Java 缓存语义，平局时返回「上一次查询的 leaf」。
//     注意：vanilla 运行时该缓存存活于 populateBiomes 的逐 cell 查询序列；C++ surface 逐块查询序列与其不同，
//     打开缓存会使平局结果依赖查询顺序而难以与 vanilla 参照对齐 —— 默认建议关闭，仅用于 A/B 对照实验。
//
// ---------------------------------------------------------------------------
// 2026-08-09 searchtree-fix 第 1 轮（0xC0000005 read 0x0 崩溃）：
//   主会话运行 block_probe -biomeDump 812 73 -337 崩溃（0xC0000005 read 0x0，
//   `mov rdx,[rdx]` RDX=0 空指针解引用），根因详见 .artifacts/8576-24blocks/biome-fix/searchtree-fix.md。
//   第 1 轮修订要点（消除空指针解引用路径 + 实现级 UB，平局语义与 Java 不变）：
//   ① getBatchedTree：不再 `std::move(cur)` 后 `cur.clear()` 复用（MSVC 移动后内部指针置 null 的
//      实现细节 + 递归 createNode 对 batch 引用操作组合下存在树指针失效风险），改为拷贝进 result，
//      与 Java `new TreeBranchNode(list2)`（拷贝到新数组）逐行对应。
//   ② Branch::getResultingNode：对子节点递归返回 null 做防护（Java 因树结构保证永不 null；C++ 防御，
//      不改变平局语义——正常树中该分支永不触发）。
//   ③ get：对 getResultingNode 返回 null 做防护（返回 nullptr 而非解引用）。
//   ④ makeBranch / batchRangeLength / sortBatches：对空 sub/batch 显式防护（空 vector 的 [0]
//      是 UB，MSVC 读 _Myfirst=null → 读 [0]，与崩溃指令 mov rdx,[rdx] 完全吻合）。
//
// ---------------------------------------------------------------------------
// 2026-08-09 searchtree-fix 第 2 轮（0xE06D7363 C++ 异常崩溃）：
//   第 1 轮修复版主会话重编后运行时抛 **0xE06D7363（C++ 异常）**（不再是访问违规）。
//   崩溃现场：RIP=KERNELBASE.RaiseException → VCRUNTIME CxxThrowException → block_pro+0x2A3A3，
//   栈帧链与第 1 轮崩溃（#0 block_pro+0x29B92 mov rdx,[rdx]）逐帧一致（仅因第 1 轮防御代码使各帧偏移
//   后移 ~0x2E0~0x811）→ 确证两次崩溃在**同一函数**：searchtree.h 树构建路径。
//   对拍 + 数学/穷举验证结论（本 worker 独立核实）：
//   - batch 分割（batch=6^floor(log6(n-0.01))，切分满批+尾批）与 Java L469-487 逐行等价；
//   - 对任意 n≥2：batch 恒满足 1 ≤ batch ≤ n-1（n 整数，6^k ≤ n-0.01 ⇒ n ≥ 6^k+1）→ 切分无空批、
//     节点守恒（Σ批次=n）、每批大小 < n → createNode 递归每层严格减小、必然终止；
//   - 真实数据 n=7593（biome_params.json）→ batch=1296 → 批次 [1296×5,1113] → 递归 216→36→6 全落到
//     MAX_SIMPLE 小树，无空子集。n=7 / 36 / 37 / 216 / 217 / 1296 / 1297 / 1500 / 36400 全部无空批。
//   - 因此「batch 分割算法产生空子集 → 防御 throw」在算法层不成立：防御 throw 被触发说明运行时
//     某个 sub/batch vector 实际为空或含 null 指针，即存在内存破坏/UB 使容器状态损坏。
//   - 第 2 轮算法级加固（本版，消除理论浮点边界 + 结构上杜绝空子集 + 隔离递归别名）：
//     ① getBatchedTree 的 batch 改为**确定性整数计算**（最大 6^k ≤ n-1，除法防溢出），
//       消除 Java/C++ 浮点 log/pow 在极端 n 下（n 恰为 6^k 且 k≥14，n≥7.8e10）可能把 batch 算成 n
//       导致 createNode 无限递归的理论边界风险；对 MC 实际规模（k≤5）与第 2 版浮点公式结果逐值一致。
//     ② getBatchedTree 显式结构不变式：batch ≥ 1、所有产出批次非空、Σ批次 == n、每批 < n。
//     ③ createNode 递归改传**子集的独立拷贝**（Java Arrays.asList(node.subTree) 语义等价——排序只影响
//       该子集、递归后不再读取），彻底隔离「递归原地排序修改共享 batch 引用」的别名风险。
//     ④ 保留第 1 轮全部防护（不新增 throw 点）：防御分支在正常树中永不触发，平局语义逐位不变。
//   若第 2 轮加固后仍崩，说明真根因是内存破坏（需主会话反汇编 block_pro+0x2A3A3 确认 throw 点 +
//   排查 searchtree.h 之外的 UB），而非 batch 分割算法。
#pragma once

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <memory>
#include <stdexcept>
#include <utility>
#include <vector>

namespace wg {

// 7 维参数区间 —— Java MultiNoiseUtil.ParameterRange 移植（getDistance L362-366）
struct STRange {
    long min;
    long max;

    // ParameterRange.getDistance(noise)：区间外距离（noise - max 与 min - noise 取正）
    long distance(long noise) const {
        long l = noise - max;
        long m = min - noise;
        return l > 0 ? l : (m > 0 ? m : 0);
    }
};

// MultiNoiseUtil.SearchTree<T> 移植（T = 携带值，本项目为 biome id std::string）
template <typename T>
class SearchTree {
public:
    static constexpr int DIM = 7;          // HYPERCUBE_DIMENSION（Java L30）
    static constexpr int MAX_SIMPLE = 6;   // MAX_NODES_FOR_SIMPLE_TREE（Java L380）

    struct Entry {
        STRange parameters[DIM];  // 7 维顺序：temperature, humidity, continentalness, erosion, depth, weirdness, offset
        T value;
    };

    explicit SearchTree(std::vector<Entry> entries) {
        if (entries.empty()) throw std::invalid_argument("SearchTree needs at least one value");
        std::vector<Node*> leaves;
        leaves.reserve(entries.size());
        for (auto& e : entries) {
            auto leaf = std::make_unique<Leaf>(e.parameters, std::move(e.value));
            leaves.push_back(leaf.get());
            owned_.push_back(std::move(leaf));
        }
        first_ = createNode(DIM, leaves);
    }

    // 等价 Java Entries.getValue(point)（L146-152）。
    // other[DIM] 第 7 维恒为 0（Java NoiseValuePoint.getNoiseValueList L313-315）。
    const T* get(const long (&other)[DIM]) const {
        const Leaf* leaf = first_->getResultingNode(other, usePrevious_ ? previous_ : nullptr);
        if (!leaf) return nullptr;   // 防御：树损坏时返回 null（Java 永不发生）
        if (usePrevious_) previous_ = leaf;
        return &leaf->value;
    }

    void setUsePrevious(bool enabled) { usePrevious_ = enabled; }

private:
    struct Leaf;  // 前向声明（Node 虚函数返回 const Leaf*）

    struct Node {
        STRange parameters[DIM];
        explicit Node(const STRange (&p)[DIM]) {
            for (int i = 0; i < DIM; i++) parameters[i] = p[i];
        }
        virtual ~Node() = default;

        // TreeNode.getSquaredDistance（L590-598）：7 维距离平方和
        long getSquaredDistance(const long (&other)[DIM]) const {
            long s = 0;
            for (int i = 0; i < DIM; i++) {
                long d = parameters[i].distance(other[i]);
                s += d * d;
            }
            return s;
        }

        virtual const Leaf* getResultingNode(const long (&other)[DIM], const Leaf* alternative) const = 0;
    };

    struct Leaf : Node {
        T value;
        Leaf(const STRange (&p)[DIM], T v) : Node(p), value(std::move(v)) {}

        // TreeLeafNode.getResultingNode（L572-576）：直接返回 this
        const Leaf* getResultingNode(const long (&)[DIM], const Leaf*) const override { return this; }
    };

    struct Branch : Node {
        std::vector<Node*> sub;

        Branch(const STRange (&enc)[DIM], std::vector<Node*> s) : Node(enc), sub(std::move(s)) {}

        // TreeBranchNode.getResultingNode（L541-560）：
        //   ★平局语义：只有严格大于（l > m / l > n）才进入/更新；平局保持当前 leaf → 返回树序遍历第一个最小距离 leaf
        const Leaf* getResultingNode(const long (&other)[DIM], const Leaf* alternative) const override {
            long l = alternative ? alternative->getSquaredDistance(other) : INT64_MAX;
            const Leaf* leaf = alternative;
            for (const Node* node : sub) {
                if (!node) continue;   // 防御：sub 含空指针（正常树永不触发，Java 同）
                long m = node->getSquaredDistance(other);
                if (l > m) {
                    const Leaf* leaf2 = node->getResultingNode(other, leaf);
                    if (!leaf2) continue;   // 防御：子节点异常返回 null（正常树永不触发，Java 同）
                    long n = node == leaf2 ? m : leaf2->getSquaredDistance(other);
                    if (l > n) {
                        l = n;
                        leaf = leaf2;
                    }
                }
            }
            return leaf;
        }
    };

    Node* first_ = nullptr;
    bool usePrevious_ = false;
    mutable const Leaf* previous_ = nullptr;
    std::vector<std::unique_ptr<Node>> owned_;

    // TreeBranchNode(list)（L531-538）：parameters = getEnclosingParameters(list)
    // 所有 Node 由 owned_ 唯一持有（unique_ptr），树构建完成后所有指针稳定有效；
    // makeBranch 只把裸指针拷贝进 Branch::sub / children，不接管所有权，无悬垂。
    Node* makeBranch(std::vector<Node*> sub) {
        if (sub.empty()) throw std::logic_error("makeBranch: empty subtree");   // 防御（Java getEnclosingParameters 同样要求非空）
        STRange enc[DIM];
        // 防御：跳过空指针元素（Java 树结构保证非 null；C++ 防损坏防护）
        size_t initIdx = 0;
        while (initIdx < sub.size() && !sub[initIdx]) initIdx++;
        if (initIdx == sub.size()) throw std::logic_error("makeBranch: all subtree pointers null");
        for (int d = 0; d < DIM; d++) enc[d] = sub[initIdx]->parameters[d];
        for (const Node* n : sub) {
            if (!n) continue;
            for (int d = 0; d < DIM; d++) {
                if (n->parameters[d].min < enc[d].min) enc[d].min = n->parameters[d].min;
                if (n->parameters[d].max > enc[d].max) enc[d].max = n->parameters[d].max;
            }
        }
        auto b = std::make_unique<Branch>(enc, std::move(sub));
        Node* p = b.get();
        owned_.push_back(std::move(b));
        return p;
    }

    // createNode 小树排序 key（L410-419）：Σ |(min+max)/2|
    static long sumAbsMid(const Node* n, int pn) {
        long s = 0;
        for (int j = 0; j < pn; j++) s += std::llabs((n->parameters[j].min + n->parameters[j].max) / 2L);
        return s;
    }

    // createNodeComparator（L461-467）：(min+max)/2，可选取绝对值
    static long paramMid(const Node* n, int cp, bool absv) {
        long v = (n->parameters[cp].min + n->parameters[cp].max) / 2L;
        return absv ? std::llabs(v) : v;
    }

    // sortTree（L451-459）：多键稳定排序，键序 (cp+0)%pn, (cp+1)%pn, ...
    // Java List.sort（Timsort 稳定）→ C++ std::stable_sort 对应
    static void sortTree(std::vector<Node*>& v, int pn, int cp, bool absv) {
        std::stable_sort(v.begin(), v.end(), [pn, cp, absv](const Node* a, const Node* b) {
            for (int i = 0; i < pn; i++) {
                long ka = paramMid(a, (cp + i) % pn, absv);
                long kb = paramMid(b, (cp + i) % pn, absv);
                if (ka != kb) return ka < kb;
            }
            return false;  // 全键相等：保持原顺序（与 Java 稳定排序一致）
        });
    }

    // batch = 最大 6^k 使 6^k <= n-1 —— Java L472 `(int)Math.pow(6, floor(log6(n-0.01)))` 的整数等价。
    // 数学等价：6^floor(log6(n-0.01)) 是满足 6^k <= n-0.01 < 6^(k+1) 的 6^k；因 n、6^k 均整数，
    // 6^k <= n-0.01 ⇔ 6^k <= n-1。用整数除法避免 b*6 溢出：6^k <= n-1 ⇔ 6^(k-1) <= (n-1)/6。
    // 结构性保证（对任意 n≥2）：batch ≥ 1 且 batch ≤ n-1。
    //   - batch ≤ n-1 ⇒ getBatchedTree 至少产出 1 批且每批大小 < n ⇒ createNode 递归每层严格减小、终止；
    //   - batch ≥ 1 ⇒ 切分永远有可取满的批，绝无空批。
    // 对比浮点版：MC 实际规模（n≤6^5，batch≤1296）两者逐值一致；n 恰为 6^k 且 k≥14 时浮点
    // log/pow 的舍入方向可能把 batch 算成 n（无限递归），整数版从结构上排除。
    static int computeBatch(size_t n) {
        long long b = 1;
        while (b <= (long long)((n - 1) / 6)) b *= 6;   // b 用 long long：对任意 n 无 int 溢出（MC 实际 n≤7593 → b≤1296）
        return (int)b;   // 对 n≤2^31-1（Java List 上限）b≤6^11=362,797,056（int 安全）；返回 int 与 Java (int) 语义一致
    }

    // getBatchedTree（L469-487）：batch = computeBatch(n)，按顺序每 batch 个切一批。
    // 结构不变式（getBatchedTree 语义保证，Java 相同）：
    //   - nodes 空 → 返回空（调用方 createNode 保证非空，Java 对空列表同样返回空列表）；
    //   - 否则 result 非空；每个 batch 非空（满批 + 非空尾批）；Σbatch_size == nodes.size()；
    //     每个 batch_size <= batch <= n-1 < n（递归严格减小）。
    // 2026-08-09 fix：不再 `std::move(cur)` 后 `cur.clear()` 复用（MSVC 移动后内部指针置 null 的
    //   实现细节 + 递归 createNode 对 batch 引用操作组合下存在树指针失效风险），改为拷贝进 result，
    //   与 Java `new TreeBranchNode(list2)`（拷贝到新数组）逐行对应。cur 仅 push/clear 复用自身 buffer。
    static std::vector<std::vector<Node*>> getBatchedTree(const std::vector<Node*>& nodes) {
        std::vector<std::vector<Node*>> result;
        if (nodes.empty()) return result;   // 防御（Java getBatchedTree 对空列表同样返回空列表）
        const int batch = computeBatch(nodes.size());
        result.reserve((nodes.size() + (size_t)batch - 1) / (size_t)batch);
        std::vector<Node*> cur;
        cur.reserve((size_t)batch);
        for (Node* n : nodes) {
            cur.push_back(n);
            if (cur.size() >= (size_t)batch) {
                result.push_back(cur);   // 拷贝（Java：new TreeBranchNode(list2) 拷贝到新数组）
                cur.clear();
            }
        }
        if (!cur.empty()) result.push_back(std::move(cur));
        return result;
    }

    // getRangeLengthSum（L489-497）：batch 的 enclosing 参数跨距和 Σ|max-min|
    static long batchRangeLength(const std::vector<Node*>& batch) {
        if (batch.empty()) return 0;   // 防御：空 batch 贡献 0（Java getRangeLengthSum 对空参数返回 0）
        STRange enc[DIM];
        size_t initIdx = 0;
        while (initIdx < batch.size() && !batch[initIdx]) initIdx++;
        if (initIdx == batch.size()) return 0;   // 防御：全空指针 batch
        for (int d = 0; d < DIM; d++) enc[d] = batch[initIdx]->parameters[d];
        for (const Node* n : batch) {
            if (!n) continue;
            for (int d = 0; d < DIM; d++) {
                if (n->parameters[d].min < enc[d].min) enc[d].min = n->parameters[d].min;
                if (n->parameters[d].max > enc[d].max) enc[d].max = n->parameters[d].max;
            }
        }
        long s = 0;
        for (int d = 0; d < DIM; d++) s += std::llabs(enc[d].max - enc[d].min);
        return s;
    }

    // Java L442 sortTree(list, pn, i, true)：对 TreeBranchNode 列表排序 → C++ 对 batch 列表排序（按 enclosing 中点）
    static void sortBatches(std::vector<std::vector<Node*>>& batches, int pn, int cp, bool absv) {
        std::stable_sort(batches.begin(), batches.end(), [pn, cp, absv](const std::vector<Node*>& a, const std::vector<Node*>& b) {
            if (a.empty() || b.empty()) return false;   // 防御：空 batch 视为等价（Java 不会出现空 batch）
            STRange ea[DIM], eb[DIM];
            size_t ia = 0, ib = 0;
            while (ia < a.size() && !a[ia]) ia++;
            while (ib < b.size() && !b[ib]) ib++;
            if (ia == a.size() || ib == b.size()) return false;   // 防御：全空指针 batch 视为等价
            for (int d = 0; d < DIM; d++) { ea[d] = a[ia]->parameters[d]; eb[d] = b[ib]->parameters[d]; }
            for (const Node* n : a) {
                if (!n) continue;
                for (int d = 0; d < DIM; d++) {
                    if (n->parameters[d].min < ea[d].min) ea[d].min = n->parameters[d].min;
                    if (n->parameters[d].max > ea[d].max) ea[d].max = n->parameters[d].max;
                }
            }
            for (const Node* n : b) {
                if (!n) continue;
                for (int d = 0; d < DIM; d++) {
                    if (n->parameters[d].min < eb[d].min) eb[d].min = n->parameters[d].min;
                    if (n->parameters[d].max > eb[d].max) eb[d].max = n->parameters[d].max;
                }
            }
            for (int i = 0; i < pn; i++) {
                int idx = (cp + i) % pn;
                long ka = (ea[idx].min + ea[idx].max) / 2L;
                long kb = (eb[idx].min + eb[idx].max) / 2L;
                if (absv) { ka = std::llabs(ka); kb = std::llabs(kb); }
                if (ka != kb) return ka < kb;
            }
            return false;
        });
    }

    // createNode（L404-449）
    Node* createNode(int pn, std::vector<Node*>& subTree) {
        if (subTree.empty()) throw std::logic_error("Need at least one child to build a node");
        if (subTree.size() == 1) return subTree[0];
        if (subTree.size() <= MAX_SIMPLE) {
            // L410-419：小树按 Σ|(min+max)/2| 稳定排序
            std::stable_sort(subTree.begin(), subTree.end(), [pn](const Node* a, const Node* b) {
                long ka = sumAbsMid(a, pn), kb = sumAbsMid(b, pn);
                return ka < kb;
            });
            return makeBranch(subTree);
        }
        // L421-448：大树 —— 对 7 个参数分别排序+batch，选跨距和最小的划分（平局保留第一个 j，严格 >）
        long bestCost = INT64_MAX;
        int bestParam = -1;
        std::vector<std::vector<Node*>> bestBatches;
        for (int j = 0; j < pn; j++) {
            sortTree(subTree, pn, j, false);
            std::vector<std::vector<Node*>> batches = getBatchedTree(subTree);
            long cost = 0;
            for (const auto& b : batches) cost += batchRangeLength(b);
            if (bestCost > cost) {  // Java l > m（L435）严格大于，平局保留第一个 j
                bestCost = cost;
                bestParam = j;
                bestBatches = std::move(batches);
            }
        }
        sortBatches(bestBatches, pn, bestParam, true);
        std::vector<Node*> children;
        children.reserve(bestBatches.size());
        for (const auto& b : bestBatches) {
            // 递归 createNode 会对子集原地排序（Java Arrays.asList(node.subTree) 语义等价——排序只影响该
            // 子集、递归后不再读取）；传 b 的独立拷贝，使排序不触碰 bestBatches 原始 batch（隔离别名）。
            // 结构不变式（getBatchedTree 保证）：b 非空，且 b.size() < subTree.size() → 递归每层严格减小、终止。
            std::vector<Node*> childSet(b);
            children.push_back(createNode(pn, childSet));
        }
        return makeBranch(children);
    }
};

} // namespace wg
