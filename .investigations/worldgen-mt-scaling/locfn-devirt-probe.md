# ① locFn「真·去虚分派」探针 — devirtualize 最小实验（worker，不编译）

> 角色：验证 worker（只改 worldgen 源码最小实验，不编译——主会话 build.ps1 编译）。
> **背景更正（用户核实源码确认 — 修正我的前提）**：SERIAL 的 `sampleSerialLocFn` 从未去掉虚分派——它 kind-switch 后 `static_cast<const DensityFunction&>(pool[i]).sample(pos)` **转回基类引用调 `.sample`，仍走 vtable**。所以 SERIAL A/B 的 10.25× = BASE 10.03× 持平**只能证明「locFn 存储连续化 + 去 shared_ptr deref」不是争用，不能证明「虚分派不是争用」**。「去虚调用」**从未测过**，是全新实验。
> **状态：draft（代码已改，未编译——主会话编译 + 运行后升级 candidate）。** anchor 扫描 invalid=0。

---

## 1. 修正后的真实格局

| 实验 | 改了什么 | 虚分派？ | 并发放大 | 结论 |
|---|---|---|---|---|
| BASE | 原样 | 保留 | 10.38× | 基线 |
| SERIAL（改前） | locFn **存储**连续化（去 deref） | **仍保留**（static_cast 回基类引用） | 10.25× | **存储非争用**（≠ 虚分派非争用） |
| NOSPLIT | **递归**→显式栈 | 仍保留 | 9.9× | 递归非争用 |
| **DEVIRT（本实验）** | locFn 虚分派 **devirtualize**（具体类型直接调） | **去掉** | **待测** | **真·待测（全新）** |

**关键**：`static_cast<const DensityFunction&>(pool[i]).sample(pos)` 的 `static_cast` 把具体类型转换成**基类引用**，编译器随后只能走 vtable 虚调用（它不知道动态类型）。**去掉这个 static_cast 前缀**，`pool[i].sample(pos)` 的静态类型是具体类（`FlatCacheDF&`），而**池元素是值对象**（`std::vector<FlatCacheDF>`），动态类型 = 静态类型 = 具体类 → **编译器可 devirtualize 成直接调用**。这才是「去虚分派」。

---

## 2. 改动（density.h，`SplineDF::sampleSerialLocFn`，min diff）

**只改 3 个 case：去掉 `static_cast<const DensityFunction&>` 前缀，直接对具体类型池元素调 `.sample()`。** OTHER 保留（无确定类型→shared_ptr->sample）。env 门控复用 `WG_SERIAL_LOCFN`（不加新门控）；BASE 路径（L920/L1013 的 `!serialMode` 分支）**未动**。

```cpp
double sampleSerialLocFn(int idx, const NoisePos& pos) const {
    const LocFnRef& r = serialRefs[idx];
    switch (r.kind) {
        case LocFnKind::FLAT_CACHE:
            return flatCachePool[r.index].sample(pos);      // 改：去掉 static_cast，具体类型直接调
        case LocFnKind::CACHE_2D:
            return cache2dPool[r.index].sample(pos);
        case LocFnKind::BINOP:
            return binopPool[r.index].sample(pos);
        case LocFnKind::OTHER:
        default:
            return otherPool[r.index]->sample(pos);          // 保留（无确定类型）
    }
}
```

---

## 3. 静态 devirtualize 分析（是否会真去虚调用）

| 检查点 | 结果 |
|---|---|
| 池元素静态类型 | `flatCachePool` = `std::vector<FlatCacheDF>`（by-value）；`flatCachePool[i]` → `FlatCacheDF&`（const 方法下 `const FlatCacheDF&`）。**具体类，非基类/引用** | ✅ |
| 池元素动态类型 | 向量元素是一个真实 `FlatCacheDF` 对象（构造为 FlatCacheDF），非基类子对象 → **动态类型 = FlatCacheDF** | ✅ |
| `sample` 是否 virtual override | `FlatCacheDF::sample(const NoisePos&) const override`（L742，virtual，非 final） | 是（但值对象调用可 devirt） |
| devirtualize 保证性 | 类值对象上调 virtual：规范保证「glvalue 静态类型为具体类 → 动态类型=静态类型 → 调用解析到该类的 final overrider」→ 编译期可解析 = **语义上保证 devirtualize**（与 final 无关） | ✅ |
| MSVC /O2 行为 | MSVC 对静态类型已知的具体类值/虚调用做 devirtualize（标准优化）；flatCachePool 是具名成员、元素类型显式 → /O2 应直接调用 | 应 ✅ |
| const 方法内 | `sampleSerialLocFn` 是 `const`，`flatCachePool[r.index]` 为 `const FlatCacheDF&`，`sample` 为 `const` → 调 const 成员，devirt 不变 | ✅ |
| OTHER 回退 | `otherPool[r.index]` = `shared_ptr<DensityFunction>`（基类指针）→ 无法 devirt，走虚调用 | 保留（非主导集） |

**结论（candidate）**：三个 case 的 devirtualize **语义上有保证**（值对象 + 具体类静态类型），/O2 下应为直接调用。**唯一需实测确认**：MSVC 是否真生成直接调用（避免「编译器保守仍走虚调用」导致假阴性）。若测定无变化且怀疑 MSVC 未 devirt，**备选**：给三个类的 `sample` override 加 `final`（强制 devirt）——但那会动类定义，先留作兜底（仅 DEVIRT 无效时才考虑，且需确认无类从 FlatCacheDF/Cache2DDF/BinaryOperation 派生）。

**注意（BINOP 内层仍是虚调用）**：`binopPool[i].sample(pos)` devirt 的是 **BinaryOperation::sample 这层**；其内部 `a->sample`/`b->sample`（`DF=shared_ptr<DensityFunction>`）仍是虚调用。这是**正确隔离**（只去「locFn 分派层」的虚调用，不动 locFn 内部）。

---

## 4. 测量方法（主会话）

### 4.1 编译
```pwsh
pwsh versions/1.20.1/cpp/build.ps1 -Target conc_density_probe
```

### 4.2 4 组（WG_PHASETICK，禁 WG_PROFILE/WG_STAGETIMER）
```pwsh
# BASE   T=1 / T=8
$env:WG_PHASETICK=1; conc_density_probe.exe <seed> <dir> 1
$env:WG_PHASETICK=1; conc_density_probe.exe <seed> <dir> 8
# DEVIRT（WG_SERIAL_LOCFN=1，改后 = 连续池 + devirtualize）
$env:WG_PHASETICK=1; $env:WG_SERIAL_LOCFN=1; conc_density_probe.exe <seed> <dir> 1
$env:WG_PHASETICK=1; $env:WG_SERIAL_LOCFN=1; conc_density_probe.exe <seed> <dir> 8
```

### 4.3 计算
- 每 run 取 12 个 `[PTICK] chunk(x,z): density=...ms` 的**中位数**（填 12 固定 chunk；同 run 内单 chunk 抖动用中位数抗）。
- `放大比 = median(T8 density) / median(T1 density)`。
- 顺带对比 per-chunk 绝对耗时（T1）——devirt 若同时明显快（单片森林），佐证 locFn 存取单线程只是热点但非 11× 主因。

### 4.4 判据（对比基准：改前 SERIAL 10.25×（连续池+虚分派）、BASE 10.38×）
- DEVIRT（连续池+devirtualize）**显著 < 10.25×**（向 DFC 的 1.3× 靠拢）→ **虚分派是争用贡献** → 方向可行（可再扩到 wrapper 链 / 更多 locFn 类型 devirt）。
- DEVIRT **≈ 10.25×**（仅连续池+devirt 无变化）→ **虚分派非争用** → 确认①排除，转向 **② wrapper 链 + 寻址**（用我上轮交付的 `wg_sample_spline` 探针：直接采样单个 SplineDF 绕过顶部 wrapper，测纯 spline vs whole tree）。
- **注**：DEVIRT 与 BASE 的 T1 单线程对比也能验证 devirt 是否真发生（若同样 /O2 下 DEVIRT 的 T1 单样本比 BASE 快一个可测的常数 → devirt 生效且 locFn 存取是单线程热点；若 T1 完全持平 → 需怀疑 MSVC 未 devirt，走 §3 final 备选）。

---

## 5. 风险 / 边界

1. **MSVC 未 devirt（假阴性风险）**：若 DEVIRT 测定与 SERIAL 持平且 T1 也持平 → 先怀疑 MSVC 保守。兜底 = `final`（改类定义，需确认无派生）。**先看 T1 对比定位**（devirt 生效与否在 T1 就有反映）。
2. **正交门控**：`WG_SERIAL_LOCFN` / `WG_DFC_NOSPLIT` / `WG_DFC_CPU` 勿同开（测量彼此污染）。DEVIRT 只与 SERIAL 同门控。
3. **正确性可牺牲**：争用诊断专用，非生产路径。但改法保证 BASE 零变化（`!serialMode` 分支未动）；SERIAL 模式采样值逐位一致（无算法改动，仅静态分派形式）。
4. **未编译**：主会话 build.ps1 编译 + 冒烟（density 模式与改前一致）后再进 §4 测量。
5. **anchor**：本次改动未增删 @anchor（scan 实测 invalid=0）。

---

## 6. 一句话总结

**修正**：SERIAL 从未去虚分派（static_cast 回基类引用仍走 vtable）；「去虚调用」是全新实验。**改法**：`sampleSerialLocFn` 3 个 case 去掉 `static_cast<const DensityFunction&>` 前缀，直接 `pool[i].sample()`（池元素为具体类值对象 → 编译器 devirtualize）。**判据**：DEVIRT（连续池+devirt）若 < 10.25× → 虚分派是争用贡献；若 ≈ 10.25× → ①排除，转向 ②（已备好 `wg_sample_spline` 探针）。

> **confidence: draft**（代码已改未编译）。主会话编译 + 跑 §4 后升级 candidate；`confirmed` 由用户拍板。
