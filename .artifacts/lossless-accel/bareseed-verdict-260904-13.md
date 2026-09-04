# bareseed-verdict-260904-13 — 裸 seed 收口登记（confirmed）

- 状态：**confirmed（用户拍板 260904-13）**
- 内容：Rust carver/feature `biome_at_jitter` 裸 `self.seed` → 统一 `biome_access_seed`（hashSeed，ChunkRegion.java L102 口径）
- 修复 commit：**d942e4b**（worldgen_handle.rs，260904-12）
- judge：三项 PASS（260904-12，subagent ae3b6f66）
- decisive：12 → 12（Δ0 无回归）；#20 死参数自检 = jitter biome 分类两 seed 一致、区域内无可观察效应（`.tmp/p2full/seed_pick_dump-260904-12.out.txt`）
- dll 基线历史：6B31129E → A82B7A8D（本轮已被 561AFF49 取代，见 residual9-verdict-260904-13）
- 备注：260904-12 交接中「index.yaml 登记 bareseed-fix-260904-12」实际未落 index（本轮核对发现），本文件补齐登记链。
