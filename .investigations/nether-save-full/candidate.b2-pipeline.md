# candidate b2：测量/对比管线自身有 bug（nether-save-full seed A）

status: draft
worker: fan-out b2（只读静态分析 + 日志取证，无 shell）
date: 260901-03

## 结论倾向

**b2 成立，但真正的管线缺陷不在最初列的三个位置，而在实验协议层——比预想严重得多：**

**本次 seed A 三连跑（gen1 20:09 / gen2 20:13 / reconfirm 20:13:47）中 CppBridge 全部 enabled=false，dll 一次都没加载成功，任何一次运行都没有测到 Rust/C++ 输出。** 日志铁证（.tmp/rust_nether_save_seedA.log L240-242、.tmp/rust_nether_save_seedA_gen.log L235-236）：

```
[20:09:38] [CppBridge] init seed=... enabled=false
[20:09:38] [CppBridge] dll info failed: java.lang.RuntimeException: failed to extract worldgen.dll
[20:09:38] [CppBridge] initNether seed=... enabled=false
```

gen2 log 同样有 `enabled=false` + `failed to extract worldgen.dll`；且**两个 log 里都没有任何一条 `[Mixin] populateNoise(nether) intercepted`**（grep 全文 0 命中）。另外进程启动早期有 `java.lang.UnsatisfiedLinkError: Failed to create temporary file for jnidispatch.dll: 拒绝访问`（gen1 log L82/L136/L183）——临时目录权限问题，很可能同时是 worldgen.dll 提取失败的根因。

由此，事实文件里的核心前提「gen1/gen2 = Rust gen（cppReplace 同跑）」**是假的**：三次运行中 populateNoise 从未被拦截，所有 chunk 要么从盘加载（旧残留），要么 vanilla 现生成。观察到的 cave_air / gold 全部来自 **DIM-1 里上一批实验留下的 stale 存档残留**（vanilla 现生成不可能产出 cave_air——nether 无 carver，且 vanilla 参照同位置=air）。所谓「同参数跨运行不同（M4 家族嫌疑）」在 seed A 本场**不成立**——根本没有同参数：三次都在读同一份不断被停服 resave 演化的旧盘数据。

## 子候选逐一裁决

### ① ReadWorldProbe.java nether 改动有缺陷 — ❌ 排除（代码审查逐项核对）

- header 跳过：readInt+readLong+readInt×5 = WGB2 写入端（BlockProbe.java L476-482：magic/seed/size/originX/originY/worldMinY/worldHeight）逐字段对齐 ✓
- 逐 chunk 布局：read wx,wz → height×256 个 readUnsignedShort → 256 个 readUTF（BlockProbe L918-934 写入序一致）✓
- 索引换算：`by=k/256, z=(k%256)/16, x=k%16` 与写入端 y 外层/z 中层/x 内层一致 ✓；`minY+by` 动态正确（nether minY=0/height=256，log L243 已打印确认）✓
- nether 参照文件名 `_nether` 后缀与 BlockProbe L457-458 一致 ✓；seed/origin 从 ref 文件内读（非命令行拼），天然防 seed 错位 ✓
- biome readUTF 段（y=100，nether 高度内合法）正确跳过 ✓
- 逻辑上能产生的错误模式是**系统性/全局性**（索引错位→海量差），不可能产生「单 chunk y70-71 一簇 130 块 air↔cave_air」。❌ 无法解释任何观察。

### ③ compare_save_region.py / parse_mca_chunk.py 解析 bug — ❌ 基本排除

- 代码审查：palette/data 展开、`bits=max(4,(len-1).bit_length())`、无 data 段时整段取 palette[0]、Y 越界裁剪，均正确；ref 侧符号读 `>%dh`（short 有符号，raw id < 32768 无影响）。
- **决定性旁证：seed B 同一工具链「gen 内存级 = 存档 MCA = 1014474 精确同值」**——MCA 解析路径与 ReadWorldProbe 在 seed B 上互相印证一致。若解析器有 bug，seed B 不可能严丝合缝。❌ 无法解释 seed A 的 chunk 级矛盾。

### ② reconfirm 时 chunk 被 vanilla 重生成 — ⚠️ 机制方向对、表述错、单靠它不自洽

任务书里的推演「chunk(203,200) 重生成 → vanilla 输出 → 与参照完美匹配 → 总差=1」**能复现观察 4 的形状**，但按原表述审查证据链**不自洽**：

- 生成条件：盘上有 status=full 的 chunk 时 `getChunk(FULL,true)` 直接加载不重生成；重生成只在 chunk 缺失/状态不足时发生。而 20:15 后独立解析证明同一 r.6.6.mca 里该 chunk 存在（cave_air 版）。
- 若 reconfirm 真重生成并停服 resave，MCA 应被 air 版覆盖 → 20:15 后的解析应看到 air，实际看到 cave_air。**矛盾**。
- **结合新证据后机制被重构**：不是「reconfirm 那一次 CppBridge 禁用」，而是**三次运行全部禁用**。因此正确的图景是：cave_air/gold = 旧 Rust 会话的盘上残留；「总差=1」的 run 是**该 chunk 当次没从盘上读到（残留缺失/未被加载）→ vanilla 现生成 → air → 与参照匹配**。残余矛盾（gen2 内存读 air vs 其停服后 MCA 解析读 cave_air）指向 **compare 所用 mca 副本可能是 stale 拷贝或路径不符**（compare_nether_seedA.txt 只打印统计不打印路径，无法从现有产物确认它读的是哪个文件）——这一条必须由探针 P1 落实，本 worker 不下结论。

## 能解释的观察

1. gen1 内存 vs gen2 内存不同（131 vs 1 差）——不是 M4 非确定性，是两次运行「残留盘加载 vs vanilla 现生成」的差别（CppBridge 全程未启用，前提即错）。
2. 三次运行都有 quartz→gold 单点差——盘上残留的 Rust 矿石产物在 (3200,13,3208)，跨运行持久。
3. cave_air 的出现/消失——同上，盘加载（残留）vs 现生成（vanilla=air）。
4. seed B 正常——seed B 区域无残留干扰（或残留与本轮输出一致），工具链本身无恙。

## 不能解释的观察

1. gen2 内存（air）vs gen2 停服后 MCA（cave_air）：若 gen2 真读了残留 cavel 版本应见差、读了 air 版本 resave 后 MCA 应为 air——两个方向都推不出「MCA=cave_air」。最可能解释是 compare 输入文件不是 gen2 停服后的那份 mca（stale 拷贝/路径错），**待 P1 验证，未证实前不下结论**。
2. 旧残留（cave_air+gold 版）最初由哪次会话写入——需 MCA chunk Timestamp + DataVersion 取证。

## 建议主会话执行的决定性廉价探针（均近零成本）

- **P1（MCA 取证，零新运行）**：扩展 dump_chunk_203_200.py，打印 r.6.6.mca 中 chunk(203,200) 与 (200,200) 的 NBT `Timestamp`（epoch 秒）、`DataVersion`、`Status`，对照三个墙钟锚点（gen1 停服 20:09:45 / gen2 停服 20:13:21 / reconfirm 停服 ~20:13:5x）与**实际被 compare 的那个 mca 文件的 mtime+完整路径**。一次定位：cave_air 版最后由谁写入、compare 读的是不是 reconfirm 输出的那份文件。
- **P2（协议修复门，重跑前强制）**：① 修 dll 提取失败（先查 java.io.tmpdir 权限，JNA jnidispatch「拒绝访问」同源嫌疑）；② 重跑验收判据写死：log 必须含 `enabled=true` + 恰好 16 条 `populateNoise(nether) intercepted`，否则该 run 作废；③ 每次运行间清 `world/region` **和 `world/DIM-1/region`**（nether 残留正是本场混乱之源），并核对 DIM-1 mtime 确已更新。

## 前提修正后的再评估（260901-03 追加，原结论不改）

**新数据层证据（主会话确认）**：所有 gen run 的 CppBridge 均 enabled=false 的根因已定位——**cppWorldgenDir 传错一层目录 → wg_create 返回 0**（此前本文档推测的「dll 提取失败/临时目录权限」是表象之一，机制根因是路径传参错）。即 gen1/gen2/reconfirm 的世界**全部是 vanilla 生成的，没有任何一次 run 真正经过 Rust/mixin 拦截**（日志 [Mixin] 计数=0、initNether enabled=false）。

在此前提下对 b2 各子候选的再评估：

| 子候选 | 原裁决 | 新前提下 | 说明 |
|---|---|---|---|
| ① ReadWorldProbe nether 改动 | ❌ 排除 | **仍 ❌，进一步坐实** | 三次运行全走 vanilla 路径，ReadWorldProbe 的 header 跳过/索引换算/文件名/biome 段与 vanilla 读回路径完全同构；seed B「内存=存档精确同值」继续背书。若它有缺陷，差异应是系统性而非单 chunk 单簇。 |
| ② reconfirm 时 chunk 被 vanilla 重生成 | ⚠️ 机制重构 | **升级为主要成立机制的载体** | 新前提下「vanilla 重生成」不再只是 reconfirm 一环，而是**三场运行的统一状态**：凡盘上无残留的 chunk 全部 vanilla 现生成（=参照，完美匹配）；凡读到 stale 残留的 chunk 才出 cave_air/gold 差。「总差=1 的 run」= 恰好只有 (200,200) 有 gold 残留、(203,200) 无残留被现生成。原证据链矛盾（重生成+resave 应覆盖 MCA 为 air vs 20:15 后解析 cave_air）仍未闭合，仍指向 compare 输入文件非 reconfirm 输出（待 P1）。 |
| ③ compare/parse 解析 bug | ❌ 基本排除 | **仍 ❌** | seed B 交叉验证不受前提修正影响（seed B 亦为 vanilla 路径，工具链同构）。 |
| （新增）实验协议层缺陷 | 本文档主结论 | **确认为 b2 的真缺陷，根因收窄** | 缺陷从「dll 提取失败」收窄为「cppWorldgenDir 传错一层 → wg_create=0 → 静默降级 vanilla」。**流程教训**：CppBridge init 失败仅打 log 不 fail-fast，导致三场「Rust 验证」实验在无 Rust 参与下跑完并产出 5 条矛盾观察——静默降级是测量管线的 P1 级缺陷（与 AGENTS.md「DensityProbe 禁 CppBridge 否则参照被污染」同族的反向案例：需要 CppBridge 时静默不可用同样致命）。 |

**修正后的结论倾向（不变式）**：b2 成立的定性不变，但范围收窄——ReadWorldProbe/compare/parse 三个代码组件均无缺陷；b2 的真缺陷 = **实验协议层（CppBridge 静默降级 + DIM-1 残留未清 + compare 输入文件溯源缺失）**。观察 1-3 的解释力不变；观察 4（gen2 内存 vs gen2 存档）仍是唯一未闭合项，P1 探针（MCA Timestamp/文件 mtime 取证）优先级进一步升高——新前提下唯一能写出 cave_air 版本的写入者只剩「更早会话的残留未被 compare 路径反映」这一条路。

探针修订：P2-① 从「查 java.io.tmpdir 权限」改为「修 cppWorldgenDir 传参（wg_create 返回值 fail-fast）」；P2-② 验收判据不变且更关键（enabled=true + 16 条 intercepted 现在是唯一能证明 Rust 真正参与的判据）。产物保持 status: draft。
