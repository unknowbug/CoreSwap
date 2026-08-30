# 草稿：.artifacts/index.yaml 追加条目（subagent 产出，主会话应用）

> **应用位置**：`.artifacts/index.yaml`——以下 yaml 文本追加到文件末尾（`entries:` 列表尾部）。追加不覆盖。

```yaml

  # === multiworld-port（Rust 多世界 Phase A/B/C，2026-08-30，commit 1102f58 + 9a3f7fa）===
  - id: 're-code:multiworld:nether-noiseheight-fix'
    path: '../.investigations/multiworld-port/multiworld-errors.md#M3'
    kind: patch
    status: candidate
    # M3 双高度修复：worldgen_handle 存 noise_height（nether 128 ≠ world height 256）+ fill_chunk 加参数（y≥noise_top 留 Air）+ 宏观采样器网格只铺噪声高度 + 13 探针 bin 补参。nether match 23.77%→74.04%（y≥128 0%→100%，超 C++ 71.97%）；overworld 95.40% 零回归。教训：参数化须贯穿 JSON→handle→fill→网格→est 全链路；C++ 已知坑清单是移植 checklist。

  - id: 're-code:multiworld:nether-determinism-fix'
    path: '../.investigations/multiworld-port/multiworld-errors.md#M4'
    kind: patch
    status: candidate
    # M4 确定性修复：biome.rs BiomeClassifier features/carvers HashMap→BTreeMap——原 all_features_lists() 每进程随机迭代序 → PlacedFeatureIndexer 编号随机 → nether features 运行间漂移 2796 块。修后两次运行逐位一致。教训：跨进程确定性要求 Registry 类容器迭代序确定（BTreeMap/Vec 排序），Rust HashMap 默认不满足。

  - id: 're-code:multiworld:nether-blocks-evidence'
    path: '../.investigations/multiworld-port/cmd-output/nether_blocks_match_v2_noiseheight.txt'
    kind: evidence
    status: candidate
    # Phase A 实证：multiworld_nether_blocks vs vanilla nether 参照（WGB2 4×4@0,0 h256）修双高度后 match 74.04%、y≥128 四带 100%；配套 v1（修前 23.77%）+ 游戏内拦截摘录 nether_ingame_intercept_20260830.txt（initNether enabled=true + populateNoise(nether) intercepted chunk(-1,-1)）。

  - id: 're-code:multiworld:ingame-nether-wiring'
    path: '../versions/1.20.1/docs/09-multi-dimension.md#八rust-多世界落地2026-08-30-phase-abc-commit-1102f58--9a3f7fa'
    kind: patch
    status: candidate
    # Phase B/C 游戏内接线：jni_bridge initDim（5 参 wg_create）+ CppBridge netherHandle/fillChunkNether/feedBeardifier 泛化 + Mixin 按维度分派（末地保护=biomeSource 缓存反射，@Shadow 够不到父类字段）+ build.gradle processResources inputs.file dll（M5：UP-TO-DATE 跳过 doFirst 同步致旧 dll → UnsatisfiedLinkError）。结论小节见 docs/09「八」；错误台账 multiworld-errors.md M1-M5。
```
