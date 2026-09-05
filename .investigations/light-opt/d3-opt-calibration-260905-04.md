# D3 优化立项 · 口径复核（260905-04 · confirmed 2026-09-05 用户拍板，judge APPROVE-WITH-CONDITIONS 在先）

## 口径声明（§9.7 三要素）
- **本次内核微基准**：载体 = `worldgen-core/src/bin-diag/light_bench_real.rs`（rustc 直编，链接 rlib 96a33b09，01:35 版与 jar 内嵌 dll 同源）；数据 = **真实 blocks9**（vanilla WGB2 4×4@200 seed 8576294172403134396，取 (12..14,12..14) 3×3，u16 raw id 直放大、无 state 级 luminance 覆盖，emission 走 light_data.json 表回退）；256 chunks 批 wall，预热 8；engine scratch 复用。
- **对比历史口径**：260905-03 合成口径 6.88ms/chunk（合成 3×3 全空气+4 光源）——**与真实口径不可比**（数据分布不同），本次仅作量级对照。

## 结果
1. **真实数据内核 = 5.796 ms/chunk**（256 chunks，1.484s 总 wall）；sanity：nonzero 288497/884736，每 chunk 24-36 distinct ids，bedrock y=-64 densest —— 真实地形特征成立。
2. **交接数字 6.88ms 复核结论**：量级成立（真实口径还略低 16%），「内核 BFS 为 e2e 开销主体」归因**可继承**：5.8ms × 2025 chunks ≈ 11.7s，占 gate ON 超时（≈17.3s）主体。
3. 残余 ≈5.6s = Java 收集循环（884736 getBlockState/chunk）+ JNI 拷贝（未单独隔离，次要）。

## 方向含义（待 judge）
- 优化方向优先级：**(a) 内核 BFS 降维/分层**（真实地形 sky 直落路径占比高，均质区跳过收益可期）为 primary；(b) Java 收集缓存 secondary；(c) SIMD 暂缓（先算法级降维，后指令级）。

产物：`convert_blocks9.py` + `blocks9_real.bin`（.tmp/d3-opt-260905-04/）、`light_bench_real.rs`（bin-diag）。
