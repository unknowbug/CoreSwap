# GPU 算子（wg_fill_density 网格角点）验证——绕开 CPU 并发慢的候选方案（2026-08-16）

> 状态：draft | 关联 density-latency-rootcause.md（CPU density 并发慢 11×）
> 结论前置：GPU 算子（网格角点 768 点/chunk）**已实测 22-39×**，maxDiff 1e-6~8e-6（精度达标）——但**接入 fillOneChunkCore 被 D24/D25 判定不可行**（见下方「⚠️ 差点重蹈 D24/D25——回退教训」段）。

## GPU 算子已验证数据（throughput-I5-20260815-185504.txt）

```
# chunks | points | CPU(ms) | GPU(ms) | CPU pts/s | GPU pts/s | speedup
    1 |   768 |  8058 |  362 |  95 | 2121 | 22.26x (maxDiff=1.06e-06)
    4 |  3072 | 26868 | 1108 | 114 | 2773 | 24.26x (maxDiff=2.86e-06)
   16 | 12288 | 155017 | 4498 |  79 | 2732 | 34.46x (maxDiff=4.42e-06)
   64 | 49152 | 630592 | 16051 |  78 | 3062 | 39.29x (maxDiff=8.26e-06)
```
- **GPU 网格角点 22-39×**，精度 1e-6~8e-6（final_density，seed 8576）
- **GPU 吞吐**：2121-3062 pts/s（rtx4060 Laptop）；**CPU 吞吐**：78-114 pts/s
- **GPU 管线**：splitTotal=8672 perSample=352 splineBindBase=6，一次性编译 ~70-100s

## 关键理解（CPU 并发慢 vs GPU 算子）

### CPU 侧（fillOneChunkCore density 阶段，L790）
- 每 chunk **98304 点** × `finalDensity->sample`（树遍历 + thread_local 网格缓存）
- 并发下 density 慢 **11×**（34→400ms）；调用次数不变 → **内存延迟竞争**（L3/CCX 分片）

### GPU 算子侧（wg_fill_density 网格角点，L597-626）
- 每 chunk **768 点**（SX=4/SY=48/SZ=4，间隔 4/8/4）≈ 98304 的 1/128
- GPU 算 768 角点 → 22-39×；**CPU 只做插值**（97824 点三线性，不打 spline/noise 树）

### ⚠️ 关键差异（throughput CPU vs fillOneChunkCore CPU）
- throughput 的 CPU 模式：**768 点却要 8s**（10.5ms/点）——因为**每次无缓存**重建网格/树遍历
- fillOneChunkCore density：**98304 点 34ms**（0.35μs/点）——有 thread_local 缓存
- **两个 CPU 模式差距 3 万倍**（缓存有无）——throughput 的 CPU 基线不代表生产 fillOneChunkCore 的 CPU 成本

## 结论与待验证

- **GPU 算子（网格角点）本身已验证**（22-39×，精度达标）——这是「正确方向未实施」的方案
- **待验证**：接入 fillOneChunkCore（density 阶段改「GPU 网格角点 + CPU 插值」）后，**CPU 的 L3 并发竞争是否被绕开**（因为 CPU 只做插值 + 网格角点 1/128 工作量，且 GPU 计算与 CPU 竞争独立）
- **风险**：D24 已证明「逐 block 完整树 GPU 化」不可行（98304 点 × 8672 floats 上传 = 3.4GB/chunk 带宽死局）；**网格角点模式分 1/128 数据量**，无此风险

## 下一步（用户已选「先验证 GPU 算子」）

接入 fillOneChunkCore density 阶段 = 实代码改动（把 L790 `finalDensity->sample(fpos)` 改为「GPU 网格角点 + CPU 插值」）。最小验证 = 先在 GPU 模式下跑完整 chunk，确认：
1. GPU 网格角点结果正确（对齐 8576 99.9994% 零退化）
2. 并发下 GPU 是否绕开 CPU 竞争（看 density 阶段是否不再 11×）

---

## ⚠️ 差点重蹈 D24/D25——回退教训（2026-08-16，必须记录）

> 状态：draft | 这条是本文件最重要的记录——**我差点在已被 D24/D25 决定性排除的方向上验证，靠查 gpu-accel-errors.md 才避免**。

### 现象
- 我把 fillOneChunkCore 的 GPU 分支**改成「GPU 直算 768 网格角点 + CPU 三线性插值到 98304」**（约 80 行改动），编译成功（bench_chunks 25s），GPU 引擎上线（pipeline ready），正在跑测试。
- 测试跑到一半我停下来重查 gpu-accel-errors.md D24/D25，才发现**这个「角点 + CPU 插值」正是 D25 判定 maxDiff=8.7e-2 不可行的方案 C**——继续跑只会得到「对齐失败」的负面结果，浪费 GPU 100s + 时间。

### 根因（机制层面）
- **「GPU 算角点 + CPU 插值」= 插值近似非线性 density**。final_density = `min(squeeze(InterpolatedDF 网格插值), ...)`，squeeze 是非线性；**对非线性函数做角点间三线性插值会产生误差**。
- D25 已 sim 验证：方案 B（完整树网格插值）误差 **5e-2**；方案 C（interp 内容树角点 + CPU 插值）误差 **8.7e-2**——**均不可行**。
- D25 唯一正确形式 = **`interp_N`（8 角点 delegate + 插值）maxDiff=0**——因为它插值的是 **InterpolatedDF 的线性密度函数**（不是完整非线性 finalDensity 树）。
- **我的误解**：wg_fill_density 的 maxDiff=1e-6 是「GPU **直算完整 finalDensity 树**在 768 角点」（GPU shader 跑完整树），**不含 CPU 插值**；我**加了 CPU 插值**就引出了插值误差——**把「GPU 直算角点（精确）」误当成「GPU 角点 + CPU 插值（有误差）」**。

### 定位（诊断方法）
1. 我跑 GPU 网格角点测试时，**凭记忆以为「GPU 角点 22-39× + 插值」可行**（D24 末尾写「正确方向 = GPU 只算网格角点 + CPU 三线性插值」——**这句话本身有歧义**，让我误以为 CPU 插值可行）。
2. 重查 gpu-accel-errors.md D24/D25 规格才发现：**D25 已 sim 决定性排除「角点 + CPU 插值」（8.7e-2）**；D24 的「正确方向」措辞没提插值误差风险，误导了我。
3. **教训来源**：项目自己的知识库（D24/D25 负面结论）+ 「负面结论深化比重复尝试有价值」（D25 L656）。

### 修复
- **回退** fillOneChunkCore 的 GPU 改动（`git checkout -- worldgen_api.cpp`）——源码回到 D24 干净的 CPU 默认路径（零退化铁律不受影响）。
- **不接入** GPU 角点插值方案——它被 D24/D25 排除。

### 教训（可复用判错经验）
1. **改动生产代码前，先查项目自己的知识库**（gpu-accel-errors.md D24/D25 有无排除该方向）——**本项目已有大量「被排除的负面结论」，改代码前必须 grep 排除链**，否则重蹈覆辙。
2. **区分「GPU 直算角点（精确）」vs「GPU 角点 + CPU 插值（有误差）」**——前者 maxDiff 1e-6，后者是 D25 判不可行的方案（8.7e-2）。**「+ 插值」这一步是误差来源**（非线性 density）。
3. **「正确方向」措辞可能省略风险**——D24 末「GPU 只算网格角点 + CPU 三线性插值」没提插值误差，需回查 D25 的 sim 排除才能确认不可行。
4. **先停实验再汇报**：我跑了一半主动停（含 GPU 100s 初始化）去查知识库——**「先查清楚再跑」比「跑了再说」省 GPU 100s + 时间**。
