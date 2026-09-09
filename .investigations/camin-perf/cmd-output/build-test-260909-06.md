# 260909-06 build/test 原始输出摘录（judge 条件②闭合）

日期：2026-09-09 22:20-23:30（宿主 Get-Date 口径）

## 全量 workspace build（B3a + 探针扩展后）

```
PS> cargo build --offline --release
warning: `worldgen` (lib) generated 34 warnings (run `cargo fix --lib -p worldgen` to apply 2 suggestions)
    Finished `release` profile [optimized] target(s) in 1.79s
```
（workspace 全量含 versions/1.21.6/rust 薄壳 worldgen1216；exit 0）

## cargo test（WorldgenRust，release）

```
PS> cargo test --offline -p WorldgenRust --release
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; ... （7 个空套件同型行）
```
（复跑核 exit code：无 error 编译行，14/0；复核命令与结果同上）

## 执行体核验（#96/#36 三元组）

- rlib: target/release/deps/libWorldgenRust-96a33b096de64ab8.rlib，mtime 21:55:40 / 22:18:56 / 23:25:21（三次重链均 > worldgen_handle.rs mtime，#30 判据过）
- bench: rustc 单编 .tmp/camin_bench_26090906.exe（pre-B3a）→ .tmp/camin_b3a_26090906.exe（B3a）→ .tmp/camin_final_26090906.exe（CAP=2048 默认）
- hash 哨兵：on=6908dbfc9a79c40a / off=115641b86711d9cd（16×16）；32×32 on=93dc1dce21ead0ab / off=5774298dff7feea5（两轮 32×32 on 复现一致：178.5 / 179.9）
