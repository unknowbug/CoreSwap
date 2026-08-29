# WG_DFC_NOSPLIT：SplineDF 显式栈（去递归）最小实验

> 角色：验证 worker（改 worldgen 源码做最小实验，未编译——主会话 build.ps1 编译验证）
> 日期：2026-08-24 | 文件：`versions/1.20.1/cpp/worldgen/src/density.h`
> 目标：最小验证「无 split 直排」方向——只把 SplineDF `sampleNode`（递归 + locFn 虚调用）改造成
> **显式栈（去递归）**，保留 production 直接采样（无 split 预拆分），用 conc_density_probe 测
> 并发放大是否从 8.4× 降到 ~1.3×。本路径**只做「递归→显式栈」**，locFn 虚调用保留（第二步再做）。

## 一、改动位置（density.h SplineDF）

| 位置 | 内容 |
|---|---|
| L831 | 新增成员 `bool dfcNosplit = false;` |
| L848-853 | 构造函数读 `getenv("WG_DFC_NOSPLIT")`（仿 WG_SERIAL_LOCFN，构建期读） |
| L898-909 | `sample()` 分派：`dfcNosplit ? sampleNodeStack(...) : sampleNode(...)`（BASE 不变） |
| L994-1064 | 新增 `sampleNodeStack(...)`：显式栈（去递归），生产数据表 + locFn 虚调用保留 |

## 二、关键代码：sampleNodeStack（显式栈迭代）

只替换 `sampleNode` 的**递归**为显式栈；节点数据`nodes/locations/derivatives/subIdx`原样使用；
locFn 仍 `serialMode ? sampleSerialLocFn(...) : locationFunctions[locFn]->sample(pos)`（虚调用，未去）。
`outVal` 作为返回值累加器在帧间传递（标准栈机模式，同 mvp_spline_eval.cpp `spline_eval`）。

```cpp
double sampleNodeStack(int nodeId, const NoisePos& pos) const {
    struct StackFrame { int node; int stage; int k; double f; double v0; };
    // stage: 0=init; 1=middle: child k 已返回(存 v0), 待 push k+1; 2=middle: child k+1 已返回(Hermite); 3=tail: child 已返回(加导数项)
    StackFrame st[128];            // vanilla 子节点链深 ≤4 → 128 帧冗余（spline-tree-depth-scout.md）
    int sp = 0; double outVal = 0.0;
    st[sp++] = {nodeId, 0, 0, 0.0, 0.0};
    while (sp > 0) {
        StackFrame& fr = st[sp - 1];
        const Node& nd = nodes[fr.node];
        if (fr.stage == 0) {
            if (nd.n == 0) { outVal = (double)nd.fixedValue; sp--; continue; }   // 叶子
            double f = serialMode ? sampleSerialLocFn(nd.locFn, pos) : locationFunctions[nd.locFn]->sample(pos);
            const float* locs = locations.data() + nd.locBegin;
            const int* subs = subIdx.data() + nd.subBegin;
            int lo = 0, hi = nd.n;
            while (lo < hi) { int mid=(lo+hi)/2; if (f < locs[mid]) hi=mid; else lo=mid+1; }
            int i = lo - 1; fr.f = f;
            if (i < 0) { fr.k=0; fr.stage=3; st[sp++] = {subs[0], 0, 0, 0.0, 0.0}; }
            else if (i == nd.n-1) { fr.k=nd.n-1; fr.stage=3; st[sp++] = {subs[nd.n-1], 0, 0, 0.0, 0.0}; }
            else { fr.k=i; fr.stage=1; st[sp++] = {subs[i], 0, 0, 0.0, 0.0}; }
        } else if (fr.stage == 3) {                       // tail: outVal=base, r=base+d*(f-locs[k])
            float d = derivatives[nd.locBegin + fr.k];
            float loc = locations[nd.locBegin + fr.k];
            outVal += d * (fr.f - loc); sp--;
        } else if (fr.stage == 1) {                       // middle: child k 返回, 存 v0=nv, push k+1
            fr.v0 = outVal; fr.stage = 2;
            st[sp++] = {subIdx[nd.subBegin + fr.k + 1], 0, 0, 0.0, 0.0};
        } else {                                          // middle: child k+1 返回, Hermite
            double ov = outVal, nv = fr.v0;
            const float* locs = locations.data() + nd.locBegin;
            const float* ders = derivatives.data() + nd.locBegin;
            int k = fr.k; float g = locs[k], h = locs[k+1];
            double kd = (fr.f - g) / (double)(h - g);
            float l = ders[k], m = ders[k+1];
            double p = l * (h - g) - (ov - nv);
            double q = -m * (h - g) + (ov - nv);
            outVal = lerp(kd, nv, ov) + kd * (1.0 - kd) * lerp(kd, p, q);
            sp--;
        }
    }
    return outVal;
}
```

## 三、静态自检（未编译，纯对拍）

### 3.1 显式栈 == 递归（逐位）——逐分支对拍

| 递归 sampleNode 分支 | 显式栈等价路径 | 一致性 |
|---|---|---|
| 叶子 `n==0` → `fixedValue` | stage0 `nd.n==0` → `outVal=fixedValue; sp--` | ✅ 同值 |
| `i<0`：`base=sampleNode(subs[0]); r=base+d\*(f-locs[0])` (d=ders[0], locs[0]) | `fr.k=0; stage=3; push subs[0]`；child 返回→ `outVal(+)=ders[locBegin+0]\*(f-locations[locBegin+0])` | ✅ `outVal` 初值=base，`+= d*(f-loc)` = `base+d*(f-loc)` 同操作数同序 |
| `i==n-1`：`base=sampleNode(subs[n-1]); r=base+d\*(f-locs[n-1])` | `fr.k=n-1; stage=3; push subs[n-1]`；同理加导数项 | ✅ |
| `else`：`nv=sampleNode(subs[k]); ov=sampleNode(subs[k+1]); p=l\*(h-g)-(ov-nv); q=-m\*(h-g)+(ov-nv); r=lerp(kd,nv,ov)+kd\*(1-kd)\*lerp(kd,p,q)` | `stage=1: v0=nv(先取 child k)；push child k+1 → ov`；`stage=2: nv=v0, ov=outVal, 同式 p/q/kd/r` | ✅ 同操作数同式 |

- `f` 均 `double`（locFn sample 返回）；`locs/ders` 均 `float`；`kd=(f-g)/(double)(h-g)` 同式。
- FP 操作序一致：子值先算（push 后返回）再组合，与递归「先递归后组合」同序 ⇒ 逐位一致。
- `outVal` 帧间传递：任何帧 pop 时 `outVal` 恒为该帧返回值（叶子→fixedValue；tail→加导数项；middle→Hermite），父帧消费，标准栈机正确。

### 3.2 无 split（区别于 DFC CpuBackend）

locFn 走 `locationFunctions[locFn]->sample(pos)` / `sampleSerialLocFn(...)` 直接采样，
FlatCacheDF/Cache2DDF/BinaryOperation 的 `sample()` 内含其自身 grid/noise 直接计算，**无任何 split 预拆分**。单点应与 production 持平。

### 3.3 BASE 不变

`dfcNosplit` 默认 false → `sample()` 走原 `sampleNode(root,pos)`；数据表/member/`addLeaf`/`addNode` 未动。仅新增 env 门控分支，BAS 路径零改动。

### 3.4 门禁

`python scripts/scan_cpp_anchors.py versions/1.20.1/cpp/worldgen/src` → **invalid=0**（7 anchors: test=5 idk=2）。
新函数标 `@anchor.idk`（一致性仅静态对拍，**未运行验证**；主会话 A/B 实测 maxDiff=0 后升级 @anchor.test）。

## 四、关键红线核对

- ✅ **无 split**：直接采样，single point 保持 production 快。
- ✅ **最小**：先只「递归→显式栈」，locFn 虚调用保留（去虚调用=第二步）。
- ✅ **保正确性**：显式栈==递归（上面 3.1 逐分支对拍）。待主会话实测 maxDiff=0 兜底。
- ✅ **测量纪律**：WG_PHASETICK（干净），禁 WG_PROFILE/WG_STAGETIMER（详见 AGENTS.md）。

## 五、测量方法（主会话执行；conc_density_probe）

固定 12 chunk（cxs -6..-4, czs -6..-4，由 conc_density_probe.cpp 内定），`wg_fill_blocks_multi` + `WG_PHASETICK`；
每 chunk 一行 `[PTICK] chunk(x,z): density=%.2fms ...`，取 12 个 density 的**中位数**为单 chunk density。

```powershell
# BASE（WG_DFC_NOSPLIT 不设）
$env:WG_PHASETICK="1"
WG_PHASETICK=1 conc_density_probe <seed> <worldgen_dir> 1     # T=1 → 12 density 中位数 → d1_base
WG_PHASETICK=1 conc_density_probe <seed> <worldgen_dir> 8     # T=8 → 12 density 中位数 → d8_base
# amp_base = d8_base / d1_base   （预期 ~8.4x）

# WG_DFC_NOSPLIT
$env:WG_DFC_NOSPLIT="1"; $env:WG_PHASETICK="1"
conc_density_probe <seed> <worldgen_dir> 1                     # T=1 → d1_nosplit
conc_density_probe <seed> <worldgen_dir> 8                     # T=8 → d8_nosplit
# amp_nosplit = d8_nosplit / d1_nosplit   （若显著降 → 递归有贡献；若持平 → 争用不在递归）
```

**判据**：`amp_nosplit` 若从 `amp_base`（~8.4×）显著降至 ~3× → 递归是争用贡献之一，方向可行；
若 `amp_nosplit ≈ amp_base` → 争用不在递归（在 locFn 虚调用/寻址），需第二步「去虚调用」。

**注意**：
- `WG_DFC_NOSPLIT` 在 **`wg_create`（SplineDF 构造）时读 env**，须在**启动探针前**设好（同 WG_SERIAL_LOCFN 语义）。BASE 需 `Remove-Item Env:WG_DFC_NOSPLIT`（或 `$env:WG_DFC_NOSPLIT=$null`）。
- **勿与 WG_DFC_CPU 同开**（正交门控；WG_DFC_CPU 走 dfcBackend 全直排+split；本路径走 production finalDensity 内部）。
- 多轮取 min/median 防调度噪声；同 seed/同世界目录/同 chunk 批。
- 正确性 A/B：跑 block_probe（WG_DFC_NOSPLIT=1 vs 不设，同 seed/chunk）比对 densityBuf，期望 maxDiff=0；或仿 dfc_fill_compare.cpp 做 WG_DFC_NOSPLIT 版 fill-compare。

## 六、风险/边界

- 栈深固定 128 帧：vanilla 子节点链 ≤4，安全；若跨到更深 spline（非 vanilla/amplified），需扩帧或改动态栈（当前为最小实验，只覆盖 vanilla）。
- `sampleNodeStack` 无 `wg_splineDebug` 打印（仅诊断模式差异，采样路径相同；默认/env 门控下无影响）。
- anchor 状态：@anchor.idk（未验证）；主会话确认 maxDiff=0 后升级 @anchor.test 并加真实 probe source。
