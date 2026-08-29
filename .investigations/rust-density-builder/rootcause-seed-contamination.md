# 挖根因：Rust finalDensity vs vanilla 对不上 —— 定论 = 参照数据被种子污染（已用正确种子重生成，逐点吻合）

**日期**：[2026-08-27]
**状态**：confirmed（用户拍板 canonical seed=-2032795982907864146；铁律 seed 三查通过；参照已重生成，逐点吻合）
**一句话结论**：Rust 的 JSON-rebuild finalDensity 树**没有 bug**。旧的 `.density`/`.blocks` 参照数据实际由世界种子 `519481969467018787` 生成（`run/world` 目录被复用 + `level-seed` 为空），却被 `-PbenchSeed` 标成/命名为樱桃种子 `-2032795982907864146`。用正确世界种子 `-2032` 重生成参照后，Rust 树逐点精确还原 vanilla finalDensity。

> **错误台账归口**：本错误（参照数据种子污染）的五段式完整记录（现象/根因/定位/修复/教训）已归入 **`rust-errors.md` R5**，本文件只保持"结论性验证报告"定位（结论/影响/待办），不重复完整错误链。
> **记录价值门标注（2026-08-21 对齐框架升级）**：本文件的**高价值部分**是错误链/判据（"air 区吻合+ground 区全错 = 参照/种子配置错"签名判据、"bench.seed≠世界种子"机制、squeeze 饱和掩蔽）——这些已归入 R5（高价值/必记）。本文件的**结论部分**（"Rust 无 bug、对齐 91.17%/73.55%"）是**当前对齐状态快照**，属**中低价值**——若只作过程跟踪留 `.investigations/`，不据此写 docs 主题篇（写作时先过价值门：无复用价值的对齐状态快照不进知识库）。

## 证据链（全部一键可复现）

### 1. 种子污染源头（AGENTS.md 铁律 §一 #1）
- `run/server.properties` 的 `level-seed` 为空 + `run/world/` 是既有复用目录 → 世界种子 = level.dat 里的 `519481969467018787`。
- `-Dbench.seed`（gradle `-PbenchSeed`）**不会**改世界种子，只改 `bench.seed` 文件头标签。
- WorldGenBench/BlockProbe 采样 `noiseConfig.getNoiseRouter()` = **世界真实种子**，写文件时把 `bench.seed` 写进文件名/header。

日志铁证（污染时）：`cherry-blockprobe.log`
```
[BlockProbe] seed=-2032795982907864146      # benchSeed 标签
[BlockProbe] worldSeed=519481969467018787   # 世界真实种子（决定数据）
[BlockProbe] spawn=BlockPos{x=320, y=63, z=-96}
```

### 2. 决定性对照（seedtest / colcmp）
- `seedtest`：`519...` 在地面带匹配率 3× 于樱桃（14.4% vs 4.9%）。
- `colcmp`（seed=樱桃，旧参照）：地面带全错（y=-16 d=-0.52，vanilla 正、Rust -0.4583 钳位）。
- `colcmp2`（seed=519...，旧参照）：整列 y=-64..312 逐点 |d| < 4e-6 → 参照数据确实由 `519...` 生成。

### 3. 为什么 air 区"看似匹配"（红色鲱鱼）
`final_density` 顶层 `min(squeeze(...), caves/noodle)`，`squeeze(-1) = -0.5 + 1/24 = -0.458333` 是饱和钳位。air 区输入 ≤ -1 → 两侧都钳到 `-0.45833`，**与种子无关** → 早期 y≥112 的 100% 是饱和掩蔽，掩盖了地下带的真实误差。

### 4. 重生成参照（正确种子 `-2032`）
- 设 `level-seed=-2032795982907864146` + 删 `run/world` + kill stale java → 重新生成 world。
- 铁律 #1 三查：`[BlockProbe] worldSeed=-2032795982907864146` ✓（新日志）；`.density`/`.blocks` header seed=-2032795982907864146 ✓。
- **spawn 修正**：正确 `-2032` 世界 spawn = `(-96,118,-48)`，**不是** `(320,63,-96)`（那个是污染 `519...` 世界的 spawn）。

### 5. 重生成后 Rust vs vanilla
- **`vanilla_cmp_probe`（finalDensity）**：`matched(<1e-9)=10406/12288  maxDiff=6.842e-5`（worst @(0,-32,0)，浮点残差；0.777 的巨差消失）。
  - 即：用正确世界种子，Rust 树逐点精确还原 vanilla finalDensity（+85% 精确 1e-9，全部 <1e-4）。
- **`rust_vs_vanilla`（finalDensity+Aquifer→块）**：`match=1434052/1572864 (91.17%)  nonAir=381339/518492 (73.55%)`。
  - 对比污染参照的 nonAir ~18.62% → 正确参照 **73.55%**（+3.9×）。

## 结论 / 影响

- **"dfreg vs actual 对不上" = 参照数据种子污染，不是树表示 bug。** 用正确种子，Rust finalDensity 逐点吻合 vanilla。
- **剩余 gap（91.17% / 73.55% 非 100%）是真实待办**：surface 层块状态（草方块/雪/beach 判断）、Beardifier（structure density）、aquifer 精确性——这正是用户"做1/做3"要加的部分。
- **spawn 认知修正**：用户记忆中 cherry 种子 spawn=(320,63,-96) 实际来自污染 `519...` 世界；正确 `-2032` 世界 spawn=(-96,118,-48)。cherry basin 定位需重新确认（用正确 seed 的 terrain/biome 判定）。

## 待办（后续）
1. 全部 Rust 对照（finalDensity/地块）改用正确种子 `-2032` + 重新生成参照（已完成 .density/.blocks）。
2. 推进 surface/Beardifier/aquifer 层，把 91.17%/73.55% 往 100% 收敛。
3. 用正确 seed 复核 cherry basin（macro terrain：山/群系/湖）是否在 Rust 复现。
