# 8576-24blocks 修复后回归记录（2026-08-08 深夜，主会话执行）

> judge 审查指出「最终回归证据零落盘」——本文件补齐。所有命令均为本 session 实际运行。

## 1. #23/#24 修复闭环证据（anchor SURFBIOME#003 source）

`block_probe -biomeDump 812 73 -337`（修复后，bash-29）：

```
[CORESWAP] crash handler installed
[BIOME] (812,73,-337) = minecraft:badlands
```

- 修复前 C++ 判 forest（bash-4 记录：812/815 列 y=73/89/100 全 forest）
- 修复后 = badlands（与参照 terracotta 带一致）→ **@anchor.test SURFBIOME#003 source 有运行记录支撑**

## 2. 修复链回归（bash-35/36，stash 实验分隔）

| 参照 | 修复前（8/8 HEAD，git stash 验证） | 修复后（SearchTree+顺手对齐） | 判定 |
|---|---|---|---|
| 8576（99.9993% 基线） | 99.9993%（24 mismatch） | **99.9994%（22 mismatch）** | 提升（#23/#24 修复）✓ |
| 3200 干净参照 | 99.9997%（4） | **99.9997%（4）** | 零退化 ✓（主世界铁律） |
| 20000,20000 | 99.9989%（18，stash 实验确认 = 8/8 HEAD 已有） | 99.9989%（18） | 零变化（基线修正：8/7 深夜记录的 99.9997% 已过时）✓ |
| -288,-256（结案参照） | 95.7376%（结构假 diff 基线） | 95.7376% | 零变化 ✓ |

- stash 实验（bash-34）：`git stash push` 后编译跑 20000 = 99.9989%（18 块）——**证明 20000 的 18 块非今日改动引入**（river/taiga 边界插值差，并入 8576 21 块课题）
- 顺手对齐单独回归（bash-10）：8576 99.9993% + 3200 99.9997%（零退化）——在 SearchTree 修复前已单独验证

## 3. 门禁

- `scan_cpp_anchors.py` invalid=0（bash 两次运行，含 searchtree.h @anchor SURFBIOME#003）

## 4. 搜索树移植验证链（3 版迭代）

| 版本 | 现象 | 根因 | 修复 |
|---|---|---|---|
| v1（worker） | 0xC0000005 read [0]（mov rdx,[rdx] RDX=0） | Node 树生命周期/悬垂 | worker 修复版（防御分支） |
| v2（worker） | 0xE06D7363 C++ 异常 | makeBranch/createNode 空子集 | worker 修复版（batch 分割） |
| v3（主会话） | 0xE06D7363 异常 + WG_STDIAG 诊断 | **MSVC long=32 位：`long bestCost = INT64_MAX` 截断 -1 → `bestCost > cost` 恒 false → bestBatches 空** | 全链路 long long（64 位）✓ 修复 |

- 诊断定位路径：WG_STDIAG 打印 `bestCost=-1` → int64_test.cpp 复现 MSVC long 截断 → 全量 64 位化
- 修复后 (812,73,-337) = badlands ✓（第 1 节）

## 5. 残余 mismatch 记录（非本任务修复范围）

- 8576 剩余 22 块 = 21 块 finalDensity 边界翻转（插值精度差，candidate 待立项）+ river 1 块（同机制）
- 20000 18 块（river/taiga 边界，同机制，并入课题）
- -288 结案（结构假 diff，95.74% 基线）

## 6. 未验证事项（诚实声明）

- Java DensityProbe 同点高精度对比未跑（21 块课题立项前置，需 gradle 探针 + CppBridge 禁用）
- RouterProbe SURFBIOME 与游戏实际 biome 矛盾（手动 BiomeAccess seed 路径存疑）未进一步验证——#23/#24 已由修复闭环证实（参照 terracotta = 权威），SURFBIOME 矛盾不阻塞
