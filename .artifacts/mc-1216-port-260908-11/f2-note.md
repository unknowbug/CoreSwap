# F2 探针修复说明（260908-11，载体在 gitignored runtime/）

- 改动：`runtime/1.20.1/java/src/main/java/wg/bench/mixin/AquiferDumpProbeMixin.java` 与 `runtime/1.21.6/java/...` 同名文件 `wgNoiseBasedFluidLevel`：`return Math.min(surfaceHeightEstimate, prnd)` → `return Math.min(surfaceHeightEstimate, base + prnd)`（对齐 vanilla `q = base + prnd; return Math.min(surfaceHeightEstimate, q)`，1.20.1/1.21.6 同语义，mc_src_extract AquiferSampler.java:447-452 一手核对）。
- 验证：静态行级对齐 + 运行时 sane（多轮链级 dump，fl 值与 vanilla 缓存真值一致）+ Rust 跨执行体旁证；spread-replay 字段级手算未达成（降级声明，见 verdict-f1f2-chunky-260908-11.md §F2）。
- 附带：`runtime/1.20.1/java/build.gradle` 新增 aqDumpRust env 通道（对齐 1.21.6 侧；含 dbg 行）。
