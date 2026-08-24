# locFn 连续化 A/B 实验 — SERIAL 路径实现 + 测量方法（worker 交付）

> 角色：A/B 验证 worker（只改 worldgen 源码，不编译——主会话 build.ps1 编译验证）
> 课题：验证「locFn 连续化」（保留多态+直接采样+registry 共享，只改存储布局）是否是 production 并发 11× 的主导。
> 前置依据：`.investigations/wordgen-mt-scaling/production-contention-scout.md` §6（决定性 A/B 判据）。
> **状态：draft（代码未编译；主会话编译 + 运行后升级 candidate）。**

---

## 1. 改动位置（唯一改动文件）

`versions/1.20.1/cpp/worldgen/src/density.h` —— 仅 SplineDF（`class SplineDF`，原 L811-941）。**base 分支语义零变化**（env 未设 `WG_SERIAL_LOCFN` → 走原 `vector<DF>` 路径）。

改动点：
- **L822 起**：新增 SERIAL 存储成员（`serialMode` / `serialRefs` / 按类型连续池）。
- **L849**：构造器从 `= default` 改为读 `getenv("WG_SERIAL_LOCFN")` 定 `serialMode`（每个 SplineDF 含嵌套都在 wg_create 单线程构建期定死）。
- **L857 addNode**：`serialMode` 分支登记 locFn 到连续池（copy 保留 cacheId）或 BASE `vector<DF>`。
- **L911 sampleNode**：`serialMode ? sampleSerialLocFn(...) : locationFunctions[..]->sample(pos)`。
- **L948 调试块**：SERIAL 下 `locationFunctions` 为空，仅 `!serialMode` 才索引它（防越界）。
- **L964 新增 `sampleSerialLocFn`**：按 `LocFnKind` 分派到连续池实体（经基类引用调 `sample`）或 otherPool 回退。

## 2. 关键代码

```cpp
// ---- SERIAL 存储（WG_SERIAL_LOCFN=1 启用）----
enum class LocFnKind : uint8_t { FLAT_CACHE, CACHE_2D, BINOP, OTHER };
struct LocFnRef { LocFnKind kind; int32_t index; };
bool serialMode = false;
std::vector<LocFnRef> serialRefs;
std::vector<FlatCacheDF> flatCachePool;   // 连续池（copy，cacheId 保留）
std::vector<Cache2DDF> cache2dPool;
std::vector<BinaryOperation> binopPool;
std::vector<DF> otherPool;                // 未枚举类型回退（shared_ptr）

int addNode(DF locationFn, int pointCount) {
    Node nd;
    if (serialMode) {
        LocFnRef ref;
        if (auto* fc = dynamic_cast<FlatCacheDF*>(locationFn.get())) {
            flatCachePool.push_back(*fc); ref = {LocFnKind::FLAT_CACHE, (int)flatCachePool.size()-1};
        } else if (auto* c2 = dynamic_cast<Cache2DDF*>(locationFn.get())) {
            cache2dPool.push_back(*c2); ref = {LocFnKind::CACHE_2D, (int)cache2dPool.size()-1};
        } else if (auto* bo = dynamic_cast<BinaryOperation*>(locationFn.get())) {
            binopPool.push_back(*bo);     ref = {LocFnKind::BINOP, (int)binopPool.size()-1};
        } else {
            otherPool.push_back(locationFn); ref = {LocFnKind::OTHER, (int)otherPool.size()-1};
        }
        serialRefs.push_back(ref); nd.locFn = (int)serialRefs.size()-1;
    } else {
        nd.locFn = (int)locationFunctions.size();
        locationFunctions.push_back(std::move(locationFn));
    }
    nd.locBegin=(int)locations.size(); nd.subBegin=(int)subIdx.size(); nd.n=pointCount;
    nodes.push_back(nd); return (int)nodes.size()-1;
}

// sampleNode：locFn 值来源
double f = serialMode ? sampleSerialLocFn(nd.locFn, pos) : locationFunctions[nd.locFn]->sample(pos);

double sampleSerialLocFn(int idx, const NoisePos& pos) const {
    const LocFnRef& r = serialRefs[idx];
    switch (r.kind) {
        case LocFnKind::FLAT_CACHE: return static_cast<const DensityFunction&>(flatCachePool[r.index]).sample(pos);
        case LocFnKind::CACHE_2D:   return static_cast<const DensityFunction&>(cache2dPool[r.index]).sample(pos);
        case LocFnKind::BINOP:      return static_cast<const DensityFunction&>(binopPool[r.index]).sample(pos);
        case LocFnKind::OTHER:
        default:                    return otherPool[r.index]->sample(pos);
    }
}
```

## 3. 静态自检（对标 scout §6 红线）

| 红线 | 实现 | 结论 |
|---|---|---|
| 保留虚调用 | `sampleSerialLocFn` 用 `static_cast<const DensityFunction&>(pool[i]).sample(pos)` 经基类引用调 `sample`（virtual 分派，无 shared_ptr deref） | ✅ |
| 直接采样（不重算 split） | 只改 locFn 存取；Hermite 二分+lerp 数学、locations/derivatives/subIdx 管线、采样递归全未动 | ✅ |
| registry 共享 + cacheId 不变 | copy 保留 cacheId（copy ctor 复制 member，不触发 `nextId++`/`updateInstanceCount`）；同 cacheId → 同 thread_local slot → **grid 每 chunk 恰建 1 次**（跨 SplineDF 引用同 registry 对象时，各 pool copy 共享同 cacheId，slot 命中） | ✅ |
| BASE 不变 | env 未设 → `serialMode=false` → addNode/sampleNode 走原 `vector<DF>` 路径；采样值逐位一致（零退化） | ✅ |
| env 门控 | `WG_SERIAL_LOCFN` 构建期读（wg_create 单线程）；BASE 默认 | ✅ |
| 不算法重写 | 区别 DFC（DFC 是数据驱动直排 + 显式栈；本文仅 locFn 存储布局） | ✅ |

**为什么 copy-保留-cacheId 是安全的**：FlatCacheDF/Cache2DDF/BinaryOperation 均为可拷贝（成员 = shared_ptr/int/double），copy ctor 不调用 `updateInstanceCount`、不分配新 `cacheId` → 副本与原件共享同一 cacheId。采样时 `slots[cacheId]`（thread_local，键为 cacheId）首次命中建 grid、后续命中；副本与原件的 `arg` 是同一 shared_ptr → 同一 grid。**同 cacheId ⇒ 同 arg ⇒ 同 grid ⇒ 每 chunk 每线程恰建 1 次**（BASE 语义保留）。多线程下各线程独立 `tlSlots`，无跨线程共享。

**`locFnSize()`** 改为 `serialMode ? serialRefs.size() : locationFunctions.size()`（WG_SPLINESTATS 诊断兼容）。

**SERIAL 覆盖范围**：scout 确认的 locFn 主导集（continents/erosion/offset/factor/jaggedness/ridges = FlatCacheDF；ridges_folded/depth = BinaryOperation add/mul 折叠）都进连续池。嵌套 SplineDF（作为 locFn 或 BinaryOperation 的 operand）因 env 全局开启 → 其**自身 locationFunctions 也自建 serial 池**（full-tree 连续化）。未枚举类型（ShiftedNoiseDF/NoiseDF/Constant/…）落 `otherPool` 回退 shared_ptr（保 BASE 正确性）。**若坐标不落在这些枚举类型，SERIAL 对该 locFn 不连续化——这是 FAST A/B 的取舍，主会话可据 scout 的 6 个 FlatCache + 2 个 BinOp 判断覆盖是否足够代表**。

## 4. 测量方法（T=1 vs T=8 放大比）

**不用 worldgen_api 新入口**：`fillOneChunkCore`（worldgen_api.cpp L742-821 WG_PHASETICK）已打印每 chunk 密度延迟 `[PTICK] chunk(x,z): density=...ms`（L1018，QPC 零污染）。测量入口复用 `conc_density_probe`（12 固定 chunk + `wg_fill_blocks_multi`）。

**准备工作**（主会话）：
1. 编译：`pwsh versions/1.20.1/cpp/build.ps1`。`conc_density_probe` 当前**不在** build.ps1 `$exes` 列表（L44-45），需加一行 `"conc_density_probe"` 进 `$exes` 后再 `pwsh build.ps1 -Target conc_density_probe`。（可选：也可用 `bench_chunks` 测吞吐，但其 measure 是 wall/N 吞吐而非每 chunk 延迟，scout 要求区分，故用 conc_density_probe。）
2. seed/worldgen dir：用既有参照目录（如 `versions/1.20.1/cpp/worldgen-data` 与一个固定 seed，同 DFC 验证用的 seed）。

**跑 4 组**（同一可执行，env 切换；全程禁 WG_PROFILE/WG_STAGETIMER——并发污染）：

```pwsh
# BASE   T=1
$env:WG_PHASETICK=1;  conc_density_probe.exe <seed> <worldgendir> 1
# BASE   T=8
$env:WG_PHASETICK=1;  conc_density_probe.exe <seed> <worldgendir> 8
# SERIAL T=1
$env:WG_PHASETICK=1; $env:WG_SERIAL_LOCFN=1; conc_density_probe.exe <seed> <worldgendir> 1
# SERIAL T=8
$env:WG_PHASETICK=1; $env:WG_SERIAL_LOCFN=1; conc_density_probe.exe <seed> <worldgendir> 8
```

**计算**（12 个 chunk 的 `density=` ms）：
- 每 run 取 12 个 [PTICK] `density` 值 → **中位数**（抗单 chunk 抖动）。
- `每样本成本(single)` = 中位数(T1) / totalPoints。主世界 totalPoints=98304（`noiseHeight*256`；改下界/高度时调整，bench 见 worldgen_api.cpp L767）。
- `放大比 = 中位数(T8 density) / 中位数(T1 density)`。

**判据**（scout §6 原文）：
- SERIAL 放大比显著**低于** BASE（向 DFC 1.3× 靠拢）→ **A 主导**，Plan A 是真主修复，值得落地。
- SERIAL 放大比与 BASE **基本持平**（仍 ~8-11×，仅绝对耗时微降）→ **A 非主导**，Plan A 收益有限，**不做**，转向 B/C（依赖链 / I-cache）。
- 顺带：若 SERIAL 的 T1 单样本也明显变快（>10%）→ locFn 存取本身（单线程）是热点，佐证 A 有独立价值（但放大比判据才是「是否修 11×」的最终依据）。

**测量纪律**（AGENTS.md 八）：只用 WG_PHASETICK；不用 WG_PROFILE/WG_STAGETIMER（每采样 steady+原子 → 并发污染）；吞吐（wall/N）与每 chunk 延迟（[PTICK] density）分开看——放大比用延迟；线程池正确性已由 0a781e1 修复（worker 就绪/通知竞争）。

## 5. 风险 / 边界

1. **`SplineDF` 自嵌套**：未给 SplineDF 自身建 typed pool（`std::vector<SplineDF>` 成员非法——不完整类型）。直接作为 locFn 的嵌套 SplineDF 落 `otherPool`（shared_ptr）。其内部 locFn 池仍因 env 全局开启而连续化（full-tree 连续），仅该嵌套层对象本身经 shared_ptr 访问。**对 A/B 判定无碍**（scout 的 B=依赖链本来就不被连续化修复）。
2. **copy 引入额外对象**：连续池持 locFn 副本（同 cacheId）。内存极小（每类型池只放 ~6 个 distinct 对象），采样值逐位一致。若主会话想更严格，可在 addNode 用 `unordered_map<const DensityFunction*, LocFnRef>` 按指针身份去重复用池槽（本实现未做，FAST 取舍）。
3. **未编译**：主会话须 `build.ps1` 编译验证通过 + 跑一轮 `WG_SERIAL_LOCFN=1` 的 block_probe/density 对齐（BK-001 零退化）再进入测量。density.h `getenv`/`dynamic_cast`/`uint8_t` 依赖 `<cstdlib>`/RTTI/`<cstdint>`（均已在，见 L10/L2；本文件已用 dynamic_cast）。
4. **anchor 扫描**：`python scripts/scan_cpp_anchors.py versions/1.20.1/cpp/worldgen/src` 结果 `invalid=0`（本次改动未增删 anchor；L910 `@anchor.test` source 仍有效）。

---
> 反编译？无（纯 C++ 存储布局改动）。confidence: draft（主会话编译 + 运行后升级 candidate）。人工 `confirmed` 由主/用户授予。
