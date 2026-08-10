# deprecated-vegetation —— 树花植被 Feature 实现（已废弃，非必要勿动）

> **状态：已废弃（2026-08-10 用户拍板）—— 本文件夹内容仅供参考，不参与编译、不接入调度。**
> **规则：后续 session 非必要直接无视本文件夹内容。** 不要因为本文件夹存在而实现/恢复树花植被。

## 为什么废弃

用户拍板（2026-08-10）：**Feature 影响的树花植被这些装饰不做**。原因：

1. **细节版本改动太多**——树/花/草植被在 MC 版本间差异大（1.20 → 1.21 大量变动），逐位对齐成本不可接受
2. **MOD 特别容易碰到的位置**——实机 Mod 装饰主要挂 FEATURES 阶段，C++ 全接管会丢 Mod 花/草/树；兼容工作量不可接受

## 内容

从 `feature.h` 剪出的树花植被实现（原文件已移除这些段）：

| 符号 | 原 Java 参照 | 说明 |
|---|---|---|
| `SimpleBlockFeatureConfig` / `SimpleBlockFeature` | SimpleBlockFeature.java | 单方块放置（noise_provider 简化） |
| `RandomPatchFeatureConfig` / `RandomPatchFeature` | RandomPatchFeature.java | 随机斑块（花/草） |
| `TreeFeatureConfig` / `TreeFeature` | TreeFeature.java + StraightTrunkPlacer + BlobFoliagePlacer | oak/birch 直树，fancy_oak 简化 |
| `RandomSelectorFeatureConfig` / `RandomSelectorFeature` | random_selector | trees_flower_forest 等随机选择 |

## 历史事实（防重走弯路）

- 这些代码**曾实现并接入**（2026-08-10 FEATURE Phase 5），但验证未达标：
  - 树只放 40%（canGenerate 失败率高：origin ground 检查/树干空间检查失败）
  - **300515 花爆炸**：dandelion C++ 533 vs 参照 11 —— 树未实现 → 树冠区被当 air 放花
- 随后**代码层禁用**（feature_loader.h generateOther 对 flower/random_patch/simple_block/tree return false；worldgen_api.cpp random_selector return false）
- 2026-08-10 深夜重申不做 → 本 session 将实现代码迁移到此文件夹，主代码彻底移除接入点
- **禁用后基线**（实测）：8576 SURFACE 99.9994% / 3200 SURFACE 99.9997% / -288 FULL 97.8460% / 300515 FULL 98.0975% —— 树花不影响基线（参照的树/花方块 = 已知预期差异）

## 依赖与恢复

- 依赖 `feature.h` 的 `OreFeatureContext` / `BlockRegistry` / `json.h` 等（剪出时保持原状，如需单独编译需自行补依赖）
- 若未来要恢复：git 历史 c04768e 前的 feature.h 有完整版本；恢复需重新接入 feature_loader.h 分发 + worldgen_api.cpp 调度 + placement.h 植被 modifier，并重跑 Java 对拍
