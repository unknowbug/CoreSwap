# qpd1 Java 复核（260903-09，原始输出摘要）

## run A（17:03，run\world 残留 → 无效测量）
命令：gradle runServer -PbenchProbe=1 -PbenchSeed=8576294172403134396 -PbenchSize=16 -PbenchOriginX=200 -PbenchOriginZ=200
[WorldGenBench] seed=8576294172403134396 size=16 origin=(200,200)
total=764ms avg=2.98ms min=0 max=285 → chunk 均为磁盘加载（region 200,200 昨日已生成），无效。

## run B（17:05，删 run\world 后 fresh 生成 → 有效）
命令同上；Stop-Process java + Remove-Item run\world 先行。
[WorldGenBench] seed=8576294172403134396 size=16 origin=(200,200) settings=overworld
typical FULL 27-45ms；冷启动列 x=200 65-90ms；首 chunk 754ms
total=10993ms avg=42.94140625ms min=26ms max=754ms
→ median ≈ 32ms（对照 260903-08：median 33 / total 11067 / avg 43.2）✓ 基线复现

另：runServer CLI 不能带 --nogui（build-tooling #9 已知坑复现，build.gradle 已内置 programArgs）。
JNA "拒绝访问" 栈在 main 启动早期打印但不阻断（SystemDetails 硬件检测），两日均有，非新问题。
