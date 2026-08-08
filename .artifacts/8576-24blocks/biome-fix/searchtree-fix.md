# searchtree-fix：SearchTree 崩溃修复（0xC0000005 read 0x0）

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> 角色：anchor.worker 精确分析 subagent（只读；产出 patch，不直接改 src）
> 承接：`patch-biome-find.md`（SearchTree 移植应用）→ 主会话编译 20/20 成功 → 运行 `block_probe -biomeDump 812 73 -337` **崩溃**
> 本文件：`.artifacts/8576-24blocks/biome-fix/searchtree-fix.md` —— 根因 + 修复 patch + 自检清单
> 配套产物：修复版 `searchtree.h`（**同路径覆盖**，主会话替换 `src/searchtree.h`）、复现测试 `st_bug_test.cpp`
> 日期：2026-08-09　状态：**draft**（主会话应用 + 重编 + 回归后由主会话/审查决定提升）

---

## 0. 一句话总结

崩溃是 **SearchTree 树遍历/构建路径上的空指针解引用**（`mov rdx,[rdx]` 且 RDX=0 → 读 [0]）。原实现有三处实现级空指针/UB 风险点（`Branch::getResultingNode` 无 `leaf2`/`node` null 防护、`getBatchedTree` 的 `std::move(cur)` 后复用、`makeBranch`/`batchRangeLength`/`sortBatches` 对空容器的 `[0]` 访问），本 patch 全部消除，**平局语义与 Java 逐行不变**。

---

## 1. 崩溃现场与证据链（已核实）

### 1.1 崩溃现场（主会话两次运行一致）

```
[CORESWAP-CRASH] code=0xC0000005 addr=0x00007FF677CB9B92
[CORESWAP-CRASH] rw=read 0x0000000000000000
RAX=00000051D3B0F1D0 RBX=0000000000000000 RCX=0000016730B36750 RDX=0000000000000000
RSI=0000016730B36750 RDI=00000051D3B0F1D0 RBP=00000051D3B0F239 RSP=00000051D3B0F090
ip=24 80 00 00 00 48 8B 12 48 8B 02 48 8B 68 08 48
stack: #0 block_pro+0x29B92 #1 +0x25F2F #2 +0xF464 #3 +0x31CD7 ...
```

- `48 8B 12` = `mov rdx,[rdx]`，源 RDX=0 → 从地址 0 读 8 字节 → 空指针解引用（读 [0]）。
- RCX=RSI=0x...6750（堆，非空）→ 外层对象指针（Branch/SearchTree 的 this）；RDX=0（被解引用的空 Node* 或空 vector 的 `_Myfirst`）；RBX=0。
- 调用链：`block_probe -biomeDump` → `wg_sample_biome` → `BiomeSource::find` → `searchTree()`（首次懒构建）→ `SearchTree::get(point)`。

### 1.2 输出时序（决定性证据）

主会话运行日志（消息 322）显示 **`[BIOMEIN]` 与 `[FIND] point` 两行都已打印**：

```
[BIOMEIN] (812,73,-337) pick=(203,18,-84) sample=(812,72,-336) t=0.550060272 ... w=-0.541882336
[FIND] point t=5500 h=-946 c=161 e=-4442 d=1039 w=-5418
```

- `[BIOMEIN]` 在 `find()` 调用前打印（`wg_sample_biome` 内 WG_BIOMEDUMP 分支）→ 选点/6 维采样正常。
- `[FIND] point` 是 `debugFindTop` 遍历 entries 之后的输出（`debugFindTop` 是 `find()` 第一行）→ **debugFindTop 线性遍历完成、未崩**。
- 主会话误判「Top3 未打印」是 `Select-String 'FIND|BIOMEIN|BIOME'` 过滤掉了不含这些关键字的 Top3 行（`  #1 minecraft:...`）；不是逻辑 bug。
- **⇒ 崩溃发生在 debugFindTop 之后、`searchTree()` 构建 / `get()` 查询路径内，与调用链假设一致。**

---

## 2. 根因分析

### 2.1 直接崩溃点（三候选，均吻合 `mov rdx,[rdx]` / RDX=0）

| 候选 | 位置 | 崩溃机制 | 与现场吻合度 |
|---|---|---|---|
| A. `node->getSquaredDistance(other)` | `Branch::getResultingNode` 循环内（原 L139） | `sub` 中某元素为 null，`this=node=0` → `mov rdx,[rdx]` 读 `node->parameters[0].min` = [0] | 高（RCX=外层 Branch this 非空；RDX=node=0） |
| B. `leaf2->getSquaredDistance(other)` | `Branch::getResultingNode`（原 L142） | 子 `Branch` 在 `sub` 为空且 `alternative` 为 null 时返回 null → `leaf2=0` → [0] | 高 |
| C. `sub[0]->parameters[d]` | `makeBranch`（原 L161）/ `batchRangeLength` / `sortBatches` | `sub`/`batch` 为空 vector 时 `_Myfirst=null` → [0] | 高（MSVC 空 vector `operator[]` 读 `_Myfirst`，空时 `_Myfirst=null`，与指令完全吻合） |

三个候选都是**空 vector `[0]` 或 null Node\* 解引用**，即「树中存在空指针/空容器」。

### 2.2 为什么原实现存在这些空指针路径

逐行对比 Java `MultiNoiseUtil.SearchTree`（L404-449、L469-487、L541-560）**语义等价**，手工推演 n=7 / n=1500（1500→batch 1296→216→36→6 的递归）**树构建自洽、batch 均非空**——但原 C++ 实现有以下实现级缺陷，任一在 MSVC 运行时可产生空指针：

1. **`Branch::getResultingNode` 无 `leaf2`/`node` null 防护**（候选 A/B）：
   Java 因树结构保证 `TreeBranchNode.getResultingNode` 永不返回 null（`subTree` 数组非空），C++ 移植照抄了「无防御」的语义；一旦树出现任何空指针（下述 2/3 所致），直接解引用 → 崩溃。
2. **`getBatchedTree` 的 `std::move(cur)` 后 `cur.clear()` 再复用**（候选 C 的诱因之一）：
   `result.push_back(std::move(cur)); cur.clear();` —— MSVC 中 move 后 `cur` 的 `_Myfirst/_Mylast/_Myend` 全部置 null，`clear()`/后续 `push_back` 依赖「移动后可复用」这一实现细节；且 `cur` 是独立局部 vector，与 Java `list2 = newArrayList()` 逐轮新建不同。**非单线程必错，但为 UB 边界 + 与递归 createNode 对 batch 引用操作的组合下存在指针失效风险**，改为拷贝（Java `new TreeBranchNode(list2)` 也是拷贝到新数组）消除。
3. **`makeBranch` / `batchRangeLength` / `sortBatches` 对空容器直接 `[0]`**（候选 C）：
   `sub[0]->parameters[d]` / `batch[0]->parameters[d]` 对空 vector 是未定义行为；MSVC 空 vector `_Myfirst=null` → 读 [0]，**与崩溃指令完全一致**。

### 2.3 结论

崩溃是 SearchTree 树内空指针解引用。由于环境无 shell（无法编译运行复现），静态逐行已无法唯一锁定「哪一个候选在 n≈1500 实际触发」；本 patch 采取**工程上最稳妥的修复：同时堵住全部三个空指针路径 + 消除 move-after-use UB**，且不改变 Java 平局语义（Java 树结构保证这些防御分支永不触发，行为逐位一致）。

---

## 3. 修复 patch（精确到位置）

修复版完整文件：`.artifacts/8576-24blocks/biome-fix/searchtree.h`（覆盖前序交付版；主会话用它替换 `versions/1.20.1/cpp/worldgen/src/searchtree.h` 后重编即可）。

### 3.1 `Branch::getResultingNode`（原 L135-150 → 新 L150-167）

```cpp
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
```

（新增两行 `if (!node) continue;` 与 `if (!leaf2) continue;`；`if (l > m)` / `if (l > n)` 严格大于语义不变，对应 Java L549/L552。）

### 3.2 `SearchTree::get`（原 L89-93 → 新 L103-107）

```cpp
const T* get(const long (&other)[DIM]) const {
    const Leaf* leaf = first_->getResultingNode(other, usePrevious_ ? previous_ : nullptr);
    if (!leaf) return nullptr;   // 防御：树损坏时返回 null（Java 永不发生）
    if (usePrevious_) previous_ = leaf;
    return &leaf->value;
}
```

### 3.3 `getBatchedTree`（原 L203-218 → 新 L231-247）

```cpp
static std::vector<std::vector<Node*>> getBatchedTree(const std::vector<Node*>& nodes) {
    std::vector<std::vector<Node*>> result;
    if (nodes.empty()) return result;
    int batch = (int)std::pow(6.0, std::floor(std::log((double)nodes.size() - 0.01) / std::log(6.0)));
    if (batch < 1) batch = 1;
    std::vector<Node*> cur;
    cur.reserve((size_t)batch);
    for (Node* n : nodes) {
        cur.push_back(n);
        if ((int)cur.size() >= batch) {
            result.push_back(cur);   // 拷贝（Java：new TreeBranchNode(list2) 拷贝到新数组）——不再 move 后复用
            cur.clear();
        }
    }
    if (!cur.empty()) result.push_back(std::move(cur));
    return result;
}
```

（`result.push_back(std::move(cur))` → `result.push_back(cur)`：batch 指针数组拷贝进 result，语义与 Java `new TreeBranchNode(list2)`（`list2.toArray` 拷贝）一致，消除 move-after-use。）

### 3.4 `makeBranch`（原 L159-172 → 新 L178-197）

- 开头加 `if (sub.empty()) throw std::logic_error("makeBranch: empty subtree");`
- `sub[0]` 初始化改为跳过空指针元素：先找首个非空 `sub[initIdx]`，全空则抛异常；
- 遍历累加 enclosing 时 `if (!n) continue;`。

### 3.5 `batchRangeLength`（原 L221-233 → 新 L250-267）

- 空 batch 返回 0；全空指针 batch 返回 0；遍历跳过空指针。

### 3.6 `sortBatches`（原 L236-261 → 新 L270-302）

- 空 batch 视为等价（比较器返回 false）；空指针 batch 处理同 makeBranch。

> 以上 3.1/3.2 直接消除「空指针解引用崩溃」；3.3 消除 move-after-use 风险；3.4-3.6 把「空容器 `[0]`」从 UB 改为明确抛异常/等价，杜绝 `mov rdx,[rdx]`（`_Myfirst=null`）路径。

---

## 4. 自检清单

**① 树构建后所有 Node 指针有效（无局部 unique_ptr 悬垂）——✓**
- 所有 Leaf/Branch 由 `owned_`（`std::vector<std::unique_ptr<Node>>`）唯一持有；`owned_.push_back` 的 realloc 移动 unique_ptr **不改变被管理对象地址**（unique_ptr 移动只转移指针），`leaves`/`children`/`Branch::sub` 中裸指针始终指向存活对象。
- `makeBranch`/`createNode` 只拷贝裸指针，不接管所有权；无局部 unique_ptr 提前释放路径。
- `getBatchedTree` 修复版按值拷贝 batch（与 Java 一致），不存在移动后旧 buffer 复用。

**② get 递归无空解引用 ——✓**
- `Branch::getResultingNode` 对 `node`、`leaf2` 均做 null 防护；`get` 对 null leaf 返回 nullptr。
- `alternative->getSquaredDistance` 有 `alternative ?` 短路（Java `alternative == null ? ...` 等价）。

**③ vector<Entry> 移动后 value 指针稳定 ——✓**
- `find` 返回 `&leaf->value`（Leaf 在 `owned_`，SearchTree 存活期间永久有效）；不指向 `entries`/`es`/构造参数 vector。
- `es`/`entries` 在 SearchTree 构造完成后析构，不影响 `Leaf::value`（值已在构造时 `std::move` 进 Leaf）。
- 调用方 `wg_sample_biome` 在 `find` 返回后立即 `id = *bid` 拷贝；`tree_`（`mutable unique_ptr<SearchTree>`）在 BiomeSource 存活期间不析构。✓

**④ 与 Java L541-560 平局语义逐行核对 ——✓**

| Java（L541-560） | C++ 修复版 | 判定 |
|---|---|---|
| `l = alternative == null ? Long.MAX_VALUE : getDistance(alternative)` L544 | `alternative ? alternative->getSquaredDistance(other) : INT64_MAX` | ✓ 逐位一致 |
| `treeLeafNode = alternative` L545 | `leaf = alternative` | ✓ |
| `for (treeNode : this.subTree)` L547 | `for (node : sub)`（+null 防护，正常不触发） | ✓ |
| `m = getDistance(treeNode, ...)` L548 | `m = node->getSquaredDistance(other)` | ✓ |
| `if (l > m)` L549 | `if (l > m)` | ✓ 严格大于 |
| `leaf2 = treeNode.getResultingNode(...)` L550 | `leaf2 = node->getResultingNode(other, leaf)`（+null 防护） | ✓ |
| `n = treeNode == leaf2 ? m : getDistance(leaf2)` L551 | `n = node == leaf2 ? m : leaf2->getSquaredDistance(other)` | ✓（`node` 为 Leaf 时 `leaf2==node` → `n=m`） |
| `if (l > n)` L552 | `if (l > n)` | ✓ 严格大于 |
| `return treeLeafNode` L559 | `return leaf` | ✓ |
| `TreeLeafNode` 返回 this L575 | `Leaf::getResultingNode` 返回 this | ✓ |

防御分支（`!node`/`!leaf2`/空容器）在 Java 树结构下永不触发，**不改变任何输入下的返回结果**。

---

## 5. 验证步骤（主会话）

1. **替换**：`.artifacts/8576-24blocks/biome-fix/searchtree.h` → `versions/1.20.1/cpp/worldgen/src/searchtree.h`，重编（CMake 增量即可，`searchtree.h` 是 biome.h 的依赖，会触发 worldgen_core 重编）。
2. **崩溃点**：`WG_FINDTOP=1 WG_BIOMEDUMP=1 block_probe 8576294172403134396 versions/1.20.1/data/worldgen versions/1.20.1/data/vanilla_8576294172403134396_6_720_-432.blocks -biomeDump 812 73 -337 -threads 1` —— 应输出 `[BIOMEIN]`、`[FIND] point`、Top3 两行（forest/badlands 距离相等 16406746）与 `[BIOME] (812,73,-337) = minecraft:badlands`（terracotta），**不再崩溃**。
3. **独立大树路径测试**：`st_bug_test.cpp`（与修复版 searchtree.h 同目录，1500+ 合成 entries + 20 万查询）编译运行，期望 `entries=… / built ok / queries ok`，退出码 0：
   ```
   call "D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
   cl /nologo /EHsc /O2 /std:c++17 /I .artifacts\8576-24blocks\biome-fix .artifacts\8576-24blocks\biome-fix\st_bug_test.cpp /Fe:%TEMP%\st_bug_test.exe
   %TEMP%\st_bug_test.exe
   ```
4. **四套参照回归**：`-288 / 3200 / 20000 / 8576`；3200 必须保持 diff=0；8576 `(812,-337)` 区域 terracotta 带恢复（badlands）。

---

## 6. 风险与影响面

| 维度 | 评估 |
|---|---|
| 修复对象 | 仅 `searchtree.h`（SearchTree 树遍历/构建的空指针路径）；不动 biome.h / 噪声链路 / surface 规则 |
| 行为变化 | **无**（防御分支在正常树中不触发，平局语义与 Java 逐位一致；getBatchedTree 拷贝等价 Java 拷贝） |
| 性能 | `getBatchedTree` 拷贝 batch 指针（n≈1500 → 12KB/次，构建期一次性）可忽略 |
| 残留风险 | 若崩溃根因是「树构建产生了真实空指针」（候选 A/B/C 的实际触发者），防御修复**防崩但不纠错**；修复后若 `-biomeDump` 返回非 badlands 或 3200 diff≠0，需进一步用 `st_bug_test.cpp` + 真机反汇编 `block_pro+0x29B92` 定位树损坏源头（本 worker 环境无 shell，无法反汇编） |
| 状态 | **draft**（AI 不写 confirmed；由主会话编译/回归后决定提升） |

---

## 7. 产物引用

- 本文件：`.artifacts/8576-24blocks/biome-fix/searchtree-fix.md`
- 修复版头文件：`.artifacts/8576-24blocks/biome-fix/searchtree.h`（覆盖前序版）
- 独立复现测试：`.artifacts/8576-24blocks/biome-fix/st_bug_test.cpp`
- 前序：`patch-biome-find.md`（应用说明）、`analysis3.md`（平局根因定论）
- Java 参考：`versions/1.20.1/data/mc_src_extract/net/minecraft/world/biome/source/util/MultiNoiseUtil.java`（L379-604）
- 崩溃日志：项目根 `crash-coreswap-20260808-185549.txt` / `crash-coreswap-20260808-185604.txt`；`versions/1.20.1/cpp/build-msvc/bin/crash-coreswap-20260808-03*.txt`（早期 C++ 异常崩溃，同路径旁证）

---

# 第 2 轮：batch 分割根因（0xE06D7363 C++ 异常 → 第 3 版）

> 承接：第 1 轮修复版（防御 + move-after-use 消除）主会话重编通过，`block_probe -biomeDump 812 73 -337` 运行时抛 **0xE06D7363（C++ 异常）**，不再是访问违规。
> 本文件由第 2 轮 worker 追加。日期：2026-08-09　状态：**draft**

## 8. 第 2 轮崩溃现场与证据链

```
[CORESWAP-CRASH] code=0xE06D7363 addr=0x00007FF8C3511ADA
RAX=0 RBX=0x2F24FF3C8 RCX=0 RDX=0 RSI=1 RDI=0xE06D7363 RBP=4
stack: #0 KERNELBASE.dll RaiseException+0x8A  #1 VCRUNTIME140.dll CxxThrowException+0x99
       #2 block_pro+0x2A3A3  #3 +0x2644A  #4 +0xF744  #5 +0x321E7  #6 +0x2792D  #7 +0x3C4E1
```

- **0xE06D7363 = Windows 对 C++ 异常的 SEH 码**；`RaiseException ← CxxThrowException` = MSVC `throw` 语句；`block_pro+0x2A3A3` = throw 调用点。
- **栈帧链与第 1 轮崩溃逐帧对应**：第 1 轮 `#0 +0x29B92 / #1 +0x25F2F / #2 +0xF464 / #3 +0x31CD7 / #4 +0x2741D / #5 +0x3BFD1` ↔ 第 2 轮 `#2 +0x2A3A3 / #3 +0x2644A / #4 +0xF744 / #5 +0x321E7 / #6 +0x2792D / #7 +0x3C4E1`，各帧偏移仅因第 1 轮防御代码使函数变大而后移（~0x2E0~0x811）→ **两次崩溃在同一个函数（searchtree.h 树构建路径）**。
- searchtree.h 内 throw 点（rg 确认，仅 4 个）：L90 `entries.empty()`（不可能，n=7593）、L179 `makeBranch: empty subtree`、L184 `makeBranch: all subtree pointers null`、L306 `createNode: Need at least one child`。
- 主会话结论（第 2 轮任务书）：**batch 分割产生空子集 → makeBranch/createNode 收到空 → 防御 throw**，要求修「算法根因」。

## 9. 逐行对拍 + 数学/穷举验证（本 worker 独立核实）

### 9.1 逐行对拍结论

`getBatchedTree`（L469-487）、`getRangeLengthSum`（L489-497）、`createNode`（L404-449）、`sortTree`（L451-459）、`createNodeComparator`（L461-467）与第 2 版 C++ 实现**逐行等价**：

| 环节 | Java | 第 2 版 C++ | 判定 |
|---|---|---|---|
| batch 公式 | `(int)Math.pow(6, floor(log6(n-0.01)))` L472 | `(int)std::pow(6, floor(log(n-0.01)/log6))` | ✓ 同公式 |
| 切分 | `list2.add; if (size>=i) 产出; list2=new` L474-479 | `cur.push; if (size>=batch) result.push_back(cur); cur.clear()` | ✓ 等价（第 1 轮已改拷贝） |
| 尾批 | `if (!list2.isEmpty()) 产出` L482-484 | `if (!cur.empty()) result.push_back` | ✓ 非空才产出 |
| 跨距和 | `getRangeLengthSum(treeBranchNode.parameters)` L431-433 | `batchRangeLength(batch)`（enclosing 7 维） | ✓ 等价 |
| 选划分 | `if (l > m)` 严格大于，平局保留第一个 j L435 | `if (bestCost > cost)` | ✓ 等价 |
| batch 排序 | `sortTree(list, pn, i, true)` 用 TreeBranchNode.parameters 中点 L442 | `sortBatches(bestBatches, pn, bestParam, true)` 用 batch enclosing 中点 | ✓ 等价 |
| 递归 | `Arrays.asList(node.subTree)` 传 TreeBranchNode 内数组 L445 | `createNode(pn, b)` 传 batch vector 引用 | ✓ 等价（第 3 版改独立拷贝，见 §10） |

### 9.2 数学证明（覆盖任意 n≥2，比穷举更强）

`batch = 6^floor(log6(n-0.01))`，对整数 n≥2：

1. **batch ≥ 1**：n≥2 → n-0.01 ≥ 1.99 → log6 ≥ 0.384 > 0 → floor ≥ 0 → batch ≥ 1。无 batch=0。
2. **batch ≤ n-1 < n（关键）**：6^k ≤ n-0.01 ⇒ 因 n、6^k 均整数 ⇒ 6^k ≤ n-1 ⇒ batch ≤ n-1。**batch 永不等于 n**（n=6^k 时 log6(6^k-0.01) < k → batch=6^(k-1)，`-0.01` 偏移恰压掉「batch==n 无限递归」）。
3. **无空子集**：满批（size==batch≥1）产出 + 尾批非空才产出；余数 r=n mod batch，r=0 无尾批、r≥1 尾批非空。
4. **节点守恒**：Σ批次大小 == n。
5. **递归严格减小**：每批大小 ≤ batch ≤ n-1 < n → createNode 递归每层严格减小，最终到 ≤6 走 MAX_SIMPLE 小树，必然终止。

### 9.3 真实数据与特殊值

`biome_params.json` 实测 **n=7593**（文件 7595 行 − `[`/`]` 框架，本 worker 只读环境 Select-String 行号 1/7595 确认）：
- batch = 6^4 = **1296**，批次 **[1296×5, 1113]**（守恒 ✓ 无空 ✓）。
- 递归链：1296→batch 216→[216×6]→36→[36×6]→6→小树；1113→batch 216→[216×5,33]→33→batch 6→[6×5,3]→小树。全部落到 ≤6，无空子集。

特殊值推演（batch → 批次序列）：n=7→6→[6,1]；n=36→6→[6×6]；n=37→36→[36,1]；n=216→36→[36×6]；n=217→216→[216,1]；n=1296→216→[216×6]；n=1297→1296→[1296,1]；n=1500→1296→[1296,204]；n=36400→7776→[7776×4,5296]。**全部无空批、守恒、递归终止。**

### 9.4 浮点边界（唯一理论风险，整数版消除）

`log6(n-0.01)` 距最近整数的距离随 n 指数缩小：n=46656（6^6）距 6 为 1.2e-7（余量 >10^8×ulp，安全）；理论风险只在 n 恰为 6^k 且 k≥14（n≥7.8e10）时依赖 libm 舍入方向，可能把 batch 算成 n → 无限递归。**MC 1.20.1 实际 n=7593（k≤5）完全安全；Java int 上限（n≤2^31-1）对应 k≤11，仍安全。**

### 9.5 结论（诚实修正主会话假设）

- **batch 分割算法本身不产生空子集**（§9.2 数学证明 + §9.3 实测 + §9.4 边界分析）。
- **防御 throw 被触发 ⇒ 运行时某个 sub/batch vector 实际为空或含 null 指针 ⇒ 存在内存破坏/UB 使容器状态损坏**，而不是「batch 算法产生空子集」。
- 已识别的 UB：第 1 轮 `getBatchedTree` 的 `std::move(cur)` 后 `cur.clear()`+复用（第 2 版已修）；残余风险集中于「递归 createNode 对 batch 引用（bestBatches 元素）原地排序的别名组合」与「浮点 batch 的理论边界」——第 3 版分别用**独立拷贝**与**确定性整数 batch** 消除（§10）。

## 10. 第 3 版改动点（算法级加固，非只加防御）

| # | 位置 | 第 2 版 → 第 3 版 | 目的 |
|---|---|---|---|
| ① | `computeBatch`（新增） | batch 从浮点 `6^floor(log6(n-0.01))` → **确定性整数算法**：`b=1; while (b ≤ (n-1)/6) b*=6;`（long long 防溢出）。数学等价（对 n≤2^31-1，b≤6^11=362M int 安全），逐值一致。 | 消除浮点 log/pow 在极端 n 下 batch==n 无限递归的理论边界风险；把「batch ≤ n-1」变成结构保证 |
| ② | `getBatchedTree` | 用 `computeBatch` + `result.reserve` 预分配；结构不变式注释（batch≥1、批次非空、Σ=n、每批<n）。保持第 1 轮拷贝语义（无 move-after-use）。 | 结构上杜绝空子集；预分配减少 realloc |
| ③ | `createNode` 递归 | `for (auto& b : bestBatches) createNode(pn, b)` → `std::vector<Node*> childSet(b); createNode(pn, childSet)`（传**子集独立拷贝**）。 | 隔离「递归原地排序修改共享 batch 引用」的别名风险；结果与 Java `Arrays.asList(node.subTree)` 语义等价（排序只影响该子集、递归后不再读取） |
| ④ | 其余 | 全部保留第 1 轮修复（null 防护、空容器防御、拷贝语义、unique_ptr 所有权）。**未新增 throw 点**。 | 防御分支在正常树中永不触发，平局语义逐位不变 |

**行为一致性**：computeBatch 对 MC 实际规模与第 2 版浮点公式逐值一致；childSet 拷贝的初始顺序与 b 相同、排序逻辑相同 → 每个 batch 的最终树与第 2 版/Java 完全一致；`Branch::getResultingNode` 未改动 → 平局语义与 Java L541-560 逐位不变。

## 11. 自检清单（第 3 版）

**① batch 分割对任意 n≥2 无空子集 —— ✓**
- computeBatch：`b≥1` 且 `b≤n-1`（数学证明，n 为整数）；getBatchedTree 满批 + 非空尾批，Σ批次==n。

**② createNode 递归终止（每层子集严格减小） —— ✓**
- 每批 ≤ batch ≤ n-1 < n = 当前子集大小 → 递归严格减小；最终 ≤6 走 MAX_SIMPLE 小树 → makeBranch，必然终止。无 batch==n 无限递归路径（整数算法结构排除）。

**③ 平局语义仍与 Java L541-560 一致 —— ✓**
- `Branch::getResultingNode` 逐行未动：`if (l > m)` L549 / `if (l > n)` L552 严格大于，平局不更新 → 树序遍历第一个最小距离 leaf；`Leaf` 返回 this L575。防御分支（null 检查）正常树永不触发。

**④ 无 move-after-use / 悬垂（第 1 轮已修项保持） —— ✓**
- getBatchedTree：`result.push_back(cur)`（拷贝）→ `cur.clear()`（复用自身 buffer）→ 尾批 `std::move(cur)`（move 后 cur 不再用）。
- createNode：`bestBatches = std::move(batches)`（move 后 batches 不再用）；递归 `childSet` 拷贝后 createNode 修改 childSet（childSet 之后不再用）。
- makeBranch 按值拷贝；`owned_` unique_ptr 移动不改变被管理对象地址；`children.reserve` 预分配无 realloc 期间引用失效。

**⑤ 新代码语法（无 clangd 环境，人工核） —— ✓**
- 新增函数/语句均为标准 C++17；整体结构与第 2 版一致（第 2 版已编译通过）。

## 12. 验证步骤（主会话）

1. **替换**：`.artifacts/8576-24blocks/biome-fix/searchtree.h` → `versions/1.20.1/cpp/worldgen/src/searchtree.h`，重编（CMake 增量）。
2. **崩溃点回归**：`WG_FINDTOP=1 WG_BIOMEDUMP=1 block_probe 8576294172403134396 versions/1.20.1/data/worldgen versions/1.20.1/data/vanilla_8576294172403134396_6_720_-432.blocks -biomeDump 812 73 -337 -threads 1` —— 期望不再抛 0xE06D7363，输出 `[BIOME] (812,73,-337) = minecraft:badlands`（terracotta）。
3. **独立测试**：`st_bug_test.cpp`（同目录，引用第 3 版 searchtree.h）编译运行，期望 `entries=… / built ok / queries ok`。
4. **四套参照回归**：`-288 / 3200 / 20000 / 8576`；3200 保持 diff=0。

## 13. 风险与诚实说明

| 维度 | 评估 |
|---|---|
| 修复对象 | 仅 `searchtree.h`：batch 计算（确定性整数）+ 递归隔离（独立拷贝）+ 结构不变式；不动 biome.h / 噪声 / surface |
| 行为变化 | **无**（MC 实际规模下 computeBatch 与浮点公式逐值一致；childSet 拷贝语义等价；平局语义不变） |
| 对主会话假设的修正 | **batch 分割算法数学上无空子集**（§9.2 证明）；第 2 轮崩溃的直接机制是防御 throw 被触发（内存破坏/UB 使容器变空），非「算法产生空子集」 |
| 若第 3 版仍崩 | 真根因是内存破坏：需主会话 ① 反汇编 `block_pro+0x2A3A3` 确认 throw 点与 `e.what()` 字符串（异常消息定位具体 throw）；② 排查 searchtree.h 之外的 UB（density_builder/json/xoroshiro 的 runtime_error、`st_bug_test.cpp` 的 36400 规模复现）；③ 用 AddressSanitizer/`_CrtSetBreakAlloc` 定位堆破坏。本 worker 无 shell/反汇编权限，无法进一步 |
| 状态 | **draft**（AI 不写 confirmed；主会话重编 + 回归后决定提升） |
