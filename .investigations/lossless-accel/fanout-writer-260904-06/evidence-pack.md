# 双跑确证决定性探针证据包（260904-06）

## 结论一句话
stage-skip 修复链路（stageMask=3）下，OOB stone→dirt/sand 族**未消失**（43 HI + 11 LO），且 (195,199) 列 mod 独有 dirt 簇仍在——「Rust features 双跑伪影」假设对该写者**未获确证**。写者身份重新分叉。

## 验证前提（全部已核对）
- 现役 dll = WorldgenRust/target/release/WorldgenRust.dll，sha256 EC4A9AED…（1897984B，2026-09-03 23:47 构建），**dumpbin /exports 含 wg_set_flags/wg_get_flags**
- 双跑修复 b611fcb（2026-09-02 01:17）**早于** dll 构建 → NEXT_SESSION「现役 dll 早于 stage-skip 修复」判断**被推翻**
- Java 侧 setFlags 调用在 `CoreSwap\runtime\1.20.1\java\src\main\java\wg\bench\CppBridge.java` L79/L124（默认 mask=0b011），最近运行日志 `run\logs\latest.log`（2026-09-04 12:55）实测 `stageMask=3`
- Rust 侧门控：worldgen_handle.rs L608-626（bit0 carver / bit1 features / bit2 surface；mask=3 → Rust 只跑 NOISE+SURFACE，carver+features 跳过）
- seed 三查：latest.log `[BlockProbe] worldSeed=8576294172403134396` + 导出 header seed 一致；arm 标识 off/

## 决定性数据（confirm_doublerun_260904-06.py）
- A = `.tmp/p2full/off/vanilla_..._200_200.blocks`（mod 修复链路 FULL 4×4@(200,200)，12:56 运行）
- B = `.tmp/p2full/ns/cpp-noise-surface-260905.blocks`（C++ block_probe NOISE+SURFACE，无 biome 段格式）
- A-vs-B 总 mismatch = 89083 / 1572864（5.7%）
- **OOB stone族→dirt/sand：y>200 n=43 {granite→dirt:10, stone→dirt:12, andesite→dirt:14, gravel→sand:7}；y<-32 n=11 {gravel→sand:11}**
- y<-32 gravel→sand 集中 (244,-60..-54,244) 单列
- y>200 样本：(195,255/271,246) granite→dirt；(211,232-235,231) stone→dirt
- **ROOT 导出（12:50）与 off（12:56）sha256 相同（2ff71249…）→ 两者同链路；NEXT_SESSION INFO「root=旧链路 mod 导出」不准确（同为今日 mask=3 链路）**

## (195,199) 列三方对比（ref_col_check_260904-06.py）
- vanilla ref：dirt @ -59,-58,-57,-56,-42,184,200（共 7 处）
- mod-fixed(off)：上述 7 处 **+ 独有**：-41,-40, -27,-26,-25,-24, -10,-9, 7, **216, 229,232, 246,247,248, 262,263,264, 278,279,280, 294,295**（y≥216 呈 ~16 间距周期簇，vanilla 无）
- C++ NOISE+SURFACE：该列全 stone（上轮 verdict 已证 12/12 OOB 样本全 stone）

## 互斥候选（fan-out 依据）
- **b1：Rust SURFACE 写 OOB dirt**（mask bit2 未跳过；height 输入分叉 / surface rule 越界堆积；需解释 y≥216 周期 16 簇形态——与「surface 连续薄层」直觉冲突）
- **b2：Java vanilla features 条件性放置**（同 feature 在 Rust NOISE/SURFACE 产出的世界状态下 placement 与 vanilla 世界不同，如 lake/disk 判定输入差；需指出具体 feature 与条件差）

## 已有静态结论（上轮，需重审）
- `.investigations/lossless-accel/fanout-writer-260905/b1-rust-vein-dirt.md`：Rust NOISE 块表无 dirt/sand/gravel + ore_vein y 硬门 [-60,50] → 排除 Rust NOISE（该排除只对 NOISE 阶段，**未覆盖 SURFACE**）
- `.artifacts/lossless-accel/writer-verdict-260905.md`：双跑伪影 best-explanation（本轮证据动摇其前提）

## 纪律
- worker 无 shell：需运行验证 → 主会话执行，原始输出回传
- 本探针轮为新数据层证据，evidence saturation 计数重置
