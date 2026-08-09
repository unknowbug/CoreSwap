# review-002：beardifier-verdict.md 审查意见（Beardifier/StructureWeightSampler 实现交付）

> 审查角色：core.judge（隔离 subprocess，只出意见，不改 status；confirmed 由宿主人类授予）
> 审查对象（三源核对）：
> - ① 交付快照：`.investigations/-288-unclosed/beardifier-verdict.md`（draft）、`.investigations/000-架构设计/架构计划-beardifier.md`（draft）、`.investigations/-288-unclosed/cmd-output/beard_m288.txt`、`.artifacts/index.yaml`
> - ② git HEAD + 工作区 diff：`beardifier.h`（新增）、`worldgen_api.cpp` / `worldgen_api.h` / `jni_bridge.cpp` / `block_probe.cpp`（修改）
> - ③ regression 记录：`cmd-output/beard244_run1.txt`（BEARD-244 原始 dump）、verdict-04.md（CellCache 等式 8/8）、review-001.md（前次 judge 意见）
> 日期：2026-08-09。状态：**意见**（不修改任何产物 status）

---

## 一、逐项结论（对照审查清单）

| # | 审查项 | 结论 | 说明 |
|---|---|---|---|
| 1 | 证据完整性（Anchorlaw §5.5 source） | ✅ **通过** | beardifier.h 3 处 + worldgen_api.cpp 1 处 `@anchor.test`，source=`probe:block_probe!BEARD244#005`（trace 类，非 static）；实跑 `scripts/scan_cpp_anchors.py`：test=4 idk=0 **invalid=0** |
| 2 | 证据落盘（spec §1.3） | ❌ **不完整** | 落盘仅：beard_m288.txt（Java 探针 dump 参照）、beard244_run1.txt（BEARD-244 参照）。**未落盘**：t_beard2.cpp/输出（8/8 对拍，在 E:\tmp）、beard_y_dist.py/beard_compare.py 及闭合分布原始输出、block_probe -288 运行输出（95.7379%→96.2606% 表格）、8576/3200 零退化输出——均只有 verdict 文字，无可引用文件 |
| 3 | 三源核对（spec §4） | ⚠️ **有差异源** | ①快照齐全但 **beardifier.h 等 4 文件均为 git 未跟踪**（`??`），源码未纳入版本控制；②工作区 diff 与 verdict 描述一致；③**差异源①**：verdict §3.1「y=50..54/64..66 Java=0 臆造占位」与落盘 beard244_run1.txt **矛盾**（这些 y 均为非零真实值，见下）；③**差异源②**：MC 工程侧（CppWorldgen.setBeardifier / CppBridge.feedBeardifier / NoiseChunkGeneratorMixin）在本地 M，**不在本仓库**——JNI 数据流契约无法三源核对 |
| 4 | 置信度合法 | ⚠️ **附条件** | 产物 status 合法（均 draft，无非人类 confirmed）✓；但验证执行信息不完整：t_beard2/block_probe 的运行命令、环境、时间未记录；无运行时证据的 BURY/BEARD_BOX/NONE 分支未标注覆盖边界（见 §四-6） |
| 5 | 产物契约 | ⚠️ **部分** | retry 声明 ✓（§四，计数 1，与历史超 cap 关系说明）；原始数据落盘 ✓（beard_m288.txt，但 verdict §六仍写「待拷入仓库」——**文本滞后**）；index.yaml 新增 **3 条**（任务描述称 4 条，见下）；被推翻结论标注：verdict-04 §三已标 B1/B3 ✓，**但 review-001 #5/#8 要求的 docs/03 L94 + 10-timewise L731「Beardifier 恒 0」旧结论作废——仍列在待办 2（知识库更新）未完成** |
| 6 | 噪声卡历史（Anchorlaw §3） | ✅ 未发现 | -288 目标未见未解决噪声卡（与 review-001 #13 一致） |
| 7 | retry cap（Anchorlaw §9.4 / spec §4.1 / §5.3） | ✅ **通过** | 声明计数 1，实现阶段无假设失败；产物有 retry 声明，未超限 |
| 8 | 模块边界（spec §1.6 / §2.5 R5） | ✅ 通过 | beardifier.h 仅引用 Java worldgen 机制（同领域），未引用其他领域模块 skill 正文 |

## 二、三源核对结果

### ① 交付快照 ↔ ② git HEAD/工作区
- 工作区 diff 与 verdict 描述一致：`beardifier.h`（24³ 表 + sample 四分支 + fastInverseSqrt）、`worldgen_api.cpp`（beardifiers map + wg_set_beardifier/clear + fillOneChunk 3a 段叠加）、`worldgen_api.h`（C API 声明）、`jni_bridge.cpp`（JNI 包装）、`block_probe.cpp`（-beard + loadBeardFile）。
- **风险**：beardifier.h / beardifier-verdict.md / beard_m288.txt / 架构计划-beardifier.md 均未 git add——三源核对第 ② 源（git HEAD）实际不含新实现，需提交后复核。
- index.yaml diff 实际新增 **3 条**（beardifier-verdict / beardifier-impl / beard-m288-evidence），任务描述「新增 4 条目」不符——若第 4 条为对拍证据（t_beard2 输出），恰印证证据落盘缺口。

### ② 代码 ↔ ③ regression
- 叠加位置核对：fillOneChunk L616-638 3a 段 `fd = finalDensity->sample + beard->sample`，写入 densityBuf；aquifer 构造（L595-607）与 apply 均在其后 → 叠加位于 aquifer 之前 ✓ 与 Java `CellCache(add(DensityInterpolator(finalDensity), Beardifier))` 语义一致（对 interpolated 结果加，非线性在插值后）✓。
- per-chunk key：`((int64_t)((uint64_t)(uint32_t)chunkX << 32)) ^ (uint32_t)chunkZ` —— 先转 uint32 保留位模式再 64 位左移，**高 32 位 = chunkX 位模式、低 32 位 = chunkZ 位模式，无符号扩展问题、无碰撞**；L622 与 L1005 两处编码一致 ✓。
- 零退化路径：无输入时 `beard=nullptr`（map 无 key 或 empty()）→ `if (beard)` 跳过 → 行为不变 ✓ 8576/3200 零退化逻辑真实。
- 线程安全：fill 时 beardifiers 只读（find），set 在 fill 前（block_probe L239 时序正确）→ 并发读安全 ✓；JNI 路径依赖 Java 侧「fill 前 set」契约（未验证）。

### 算法逐位性（对照 1.19.x 反编译 + yarn 文档 + beard244_run1.txt）
| 项 | C++ | Java 参照 | 结论 |
|---|---|---|---|
| fastInverseSqrt | `6910469410427058090LL - (l>>1)`，int64 算术右移 | 同常量 + long >> | ✅ 一致 |
| 24³ 表索引 | 构建 `arr[i*576+j*24+k]=weight(j-12,k-12,i-12)`；读 `table[k*576+i*24+j]`（k=z+12,i=x+12,j=y+12） | `STRUCTURE_WEIGHT_TABLE[k*576+i*24+j]` | ✅ 交叉验证一致 |
| pow | `std::pow(2.718281828459045, -d/16.0)` | `Math.pow(Math.E, ...)`（Math.E=2.718281828459045） | ✅ 字面量一致；⚠️ std::pow vs fdlibm pow 不保证全定义域逐位，由对拍+闭合实证支撑 |
| clampedMap/lerp | `delta=(v-os)/(oe-os)` + 分界 + `ns+delta*(ne-ns)` | getLerpProgress + clampedLerp + lerp | ✅ 一致 |
| getMagnitudeWeight | `beard_magnitude((double)x,(double)y/2.0,(double)z)` + clampedMap(d,0,6,1,0) | 1.19.x 反编译 `MathHelper.magnitude(x,(double)y/2.0,z)`（**double 除法**）+ clampedMap | ✅ 一致（非 int 除法陷阱） |
| sample 分支 | BURY+BEARD_THIN→q=p；BEARD_BOX→q=max(0,max(o-y,y-maxY))；NONE→q=0 | 反编译 `case BURY, BEARD_THIN -> p` | ✅ **与反编译一致**（此前记忆中的「BEARD_THIN→max」不成立） |
| weight 分支 | BURY→getMagnitudeWeight；BEARD_THIN+BEARD_BOX→getStructureWeight(m,q,n,p)*0.8 | 反编译 `case BEARD_THIN, BEARD_BOX -> getStructureWeight(m, q, n, p) * 0.8` | ✅ 一致 |
| junction | `d += getStructureWeight(r,l,m,l)*0.4`（r=x-sX,l=y-sgY,m=z-sZ） | 同（getSourceX/getSourceGroundY/getSourceZ） | ✅ 一致 |

### ⚠️ 验证覆盖缺口（不否定正确性，但必须标注）
- **-288 参照全 71 piece terrain=2（BEARD_THIN）**（分布实测：groundLevelDelta -1/0/1/4，terrain 全 2）。**BURY / BEARD_BOX / NONE 分支 0 个 piece 覆盖，无运行时验证**——仅代码走读 + 1.19.x 反编译参照。BEARD-244 对拍点同属 BEARD_THIN 传导区。
- BEARD_THIN 分支与 junction 循环验证充分（71 piece + 286 junction + 8/8 对拍 + 8221 块闭合）。

## 三、已知疑点核实

1. **verdict §六「参照待拷入仓库」**：**确认文本滞后**——beard_m288.txt 已在工作区（9021 字节）且 index.yaml 已登记（re-code:-288-unclosed:beard-m288-evidence），仅未 git add。verdict 文本需改为已落盘。
2. **「y=50..54/64..66 Java=0 臆造占位」声明错误**：beard244_run1.txt（落盘参照）显示 y=51..54 = **0.000100/0.000398/0.001375/0.004168**（非零）、y=64..66 = **-0.144082/-0.165489/-0.138744**（非零）；仅 y=50=0.000000。**这些是非零真实 Java 值，不是臆造**。verdict 该声明与落盘证据矛盾，且对拍未覆盖这些点（y=51..54 深水过渡带、y=64..66 沙滩边界——恰在闭合分布过渡带 y=46..51/63..67 范围内）→ **对拍范围缺口**。

## 四、必须补齐项清单（建议 candidate 前）

1. **修正 verdict §3.1 错误声明**：引用 beard244_run1.txt 真实值（y=50..66 全列非零），删「Java=0 臆造占位」表述；如实说明对拍仅覆盖 8 点、未覆盖 y=51..54/64..66。
2. **统一对拍计数/点集**：verdict §3.1「8/8 含转负点 y=63」与 anchor L130 文本「8 点 y=55..62」矛盾——t_beard2 输出未落盘，无法核实；补齐对拍输出并统一表述。
3. **补落盘对拍证据**：t_beard2.cpp + 输出（命令 + 结果）入 `.investigations/`（spec §1.3 证据链要求），并登记 index.yaml（若「第 4 条」指此）。
4. **补落盘 block_probe 验证输出**：-288（95.7379%→96.2606%）与 8576/3200 的运行命令 + 输出摘要。
5. **澄清 12 vs 16 chunks**：beard_m288.txt 为 12 chunks（x=-18..-15, z=-16..-14，3×4），verdict §3.2 称「4×4 chunks」——给出 block_probe 实际命令；若 z=-13 行 4 chunks 无结构输入需说明。
6. **标注 BURY/BEARD_BOX/NONE 分支未覆盖**（-288 全 BEARD_THIN）——验证边界诚实声明。
7. **修正 verdict §六「待拷入仓库」**为已落盘（未提交）。
8. **完成知识库旧结论作废**：docs/03-density-functions.md L94 + 10-timewise-archive.md L731「Beardifier 恒 0」作废（review-001 #5/#8 遗留，待办 2 未完成）——收尾交付前须处理或明确移出本交付范围。
9. **MC 侧数据流核对受限声明**：CppBridge.feedBeardifier 反射字段名 / mixin populateNoise HEAD 时机 / JNI 数组长度校验缺失（jni_bridge 未校验 pieces≥pieceCount*8）——本仓库无法核对，实机验证（待办 3）前 JNI 链路无运行时实证，应在产物中声明此边界。
10. **index.yaml 条目数澄清**（实际 3 条 vs 声称 4 条）。

## 五、推荐状态

**建议 candidate（附条件）**。

- 核心结论「Beardifier（StructureWeightSampler）C++ 移植 + 接入 + 验证闭合 -288 海底边界」**方向可信**：算法核心路径（BEARD_THIN + junction）经 1.19.x 反编译逐项对照 + BEARD-244 对拍（8/8，待补落盘）+ block_probe 全量闭合 8221 块、零新增 mismatch 三重支撑；接入位置（aquifer 前叠加）、key 编码（无符号安全）、零退化路径（nullptr 跳过）代码核对无误；anchor 门禁 invalid=0。
- 主要短板集中在**证据链完整性**（对拍/验证输出未落盘、verdict 内部表述矛盾、覆盖范围声明错误）与 **BURY/BEARD_BOX/NONE 分支无运行时验证**——均不推翻核心结论，但授予 candidate 前须补齐 §四 1-5（表述/证据），6-10 可随知识库更新推进。
- **confirmed 由宿主人类拍板**（judge 不授予）。

## 六、产物引用

- `.investigations/-288-unclosed/beardifier-verdict.md`（§3.1 对拍表述、§3.2 表格、§四 retry、§六参照状态）
- `.investigations/-288-unclosed/cmd-output/beard244_run1.txt`（y=50..66 全列真实值——verdict §3.1 声明的反证）
- `.investigations/-288-unclosed/cmd-output/beard_m288.txt`（12 chunks；chunk(-16,-16) 7 pieces+28 junctions；terrain 全 2）
- `.investigations/-288-unclosed/verdict-04.md`（§二证据 2/3：9 值 + CellCache 等式 8/8；§三 B1/B3 修正）
- `.investigations/-288-unclosed/review-001.md`（#5/#8 旧结论作废遗留、#10/#11 已处理项）
- `versions/1.20.1/cpp/worldgen/src/beardifier.h`（L52-59 fastInverseSqrt、L82-96 表、L102-106 pow、L116-127 getStructureWeight、L131-160 sample）
- `versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp`（L194 map、L620-638 叠加、L997-1036 set/clear）
- `versions/1.20.1/cpp/worldgen/src/jni_bridge.cpp`（L18-31 setBeardifier，缺长度校验）
- 反编译参照：PolyhedralDev/Terra BeardGenerator.java（1.19.x，`case BURY, BEARD_THIN -> p`、`getStructureWeight(m,q,n,p)*0.8`、`(double)y/2.0`）
