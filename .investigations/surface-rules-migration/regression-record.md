# ③ 块管线阶段 A + ④ 交叉验证 — 回归记录（regression-record）

> 载体：`.investigations/surface-rules-migration/regression-record.md`
> 记录验证命令 + 输出摘要，保证证据可引用（judge 审查 MUST 修复项）。

## 探针命令（WorldgenRust/ 下 cargo run --release --bin <name>）

| 探针 | 命令 | 验证内容 |
|---|---|---|
| beard_probe | `cargo run --release --bin beard_probe` | Beardifier 纯算法自检（BURY/junction/THIN/BOX/empty/far） |
| grass_probe | `cargo run --release --bin grass_probe` | surface rules 陆地列（grass_block+dirt） |
| blocks_cmp | `cargo run --release --bin blocks_cmp` | 完整块管线 vs vanilla 交叉验证 |

## 输出摘要

### beard_probe（Beardifier 自检，9d6960c/7a39c10 后通过）
```
[OK] from_file parsed 1 chunk: 1 piece + 1 junction
sample(10,70,10) inside BURY box = -0.111721 (BURY +0.1667 + junction -0.278)
sample(10,70,10) BURY piece-only = 0.166667
sample(10,70,10) BEARD_THIN box = -0.000575
sample(10,70,10) BEARD_BOX box = -0.556886
[OK] empty chunk recognized
[OK] far sample = 0.0
beard_probe self-check passed
```

### grass_probe（surface rules 陆地列）
```
chunk(-4,-4) col(0,0) top=98 top_block=8 (minecraft:grass_block) below=9 (minecraft:dirt)
chunk(-4,-4) col(1,0) top=77 top_block=1 (minecraft:stone) below=1 (minecraft:stone)
...（其余列 top=stone，biome 相关）
grass_probe done: found 5 land columns
```

### blocks_cmp（交叉验证，76935e9）
```
magic=0x57474232 seed=-2032795982907864146 size=4 origin=(0,0) minY=-64 height=384
Rust(surface rules) vs vanilla: match=1538270/1572864 (97.80%)  nonAir=485557/518492 (93.65%)
```

## 参照文件三查（R5 教训）

- blocks 参照：`E:\PYTHON\MC\data\vanilla_-2032795982907864146_4_0_0.blocks`
- header 核实：magic=0x57474232、seed=-2032795982907864146、size=4、origin=(0,0)、minY=-64、height=384 ✓
- block id 核实：grass_block=8、dirt=9（blocks.json）✓

## 已知简化（验证缺口，judge 指出）

- biome 温度表：blocks_cmp 用 biome_temp=0.5 简化（TempCond <0.15 分支可能不准）
- surface_heights4：用 surface_height 4 角简化（非 estimateSurfaceHeight 4 角）
- biome_at：floor 对齐简化
- 无 Beardifier（spawn 区无结构）
