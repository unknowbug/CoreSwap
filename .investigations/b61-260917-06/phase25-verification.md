# B6-1 Phase 2.5 验证记录：行为化哨兵（260917-06）

```yaml
status: draft
block: 260917-06
role: 主会话（验证执行）
layer:
  static: Degraded（T1/T2/T3/T4 静态抽取与对账）
  behavioral: Partial（两个哨兵实跑：JVM 参数发射面直读）
  NOT_DONE: Full（未做「补映射后端到端行为等价」验证——属 T5 之后）
```

## 1. 验证方法（可复用）

**问题**：静态对账说「`-PsurfaceDumpDim` 无效」——但静态断言不足以定案（#42/#25 家族：静态机制断言未实测当公理）。需要**行为化直证**。

**方法**：用 Gradle **init script**（`-I`）在 `runServer` 的 `doFirst` 里打印 `t.jvmArgs` —— 这是 JVM 参数的**发射点本体**，比读 `build.gradle` 源码更强（源码说"应该发射什么"，jvmArgs 是"实际发射了什么"）。

脚本：`.tmp/b61-260917-06/print_vmargs.gradle`

## 2. 哨兵 A（DEFECT 侧）：`-PsurfaceDump -PsurfaceDumpDim=minecraft:the_nether`

**实测输出（JVM 参数发射面原文）**：
```
===B61-VMARGS-BEGIN===
VMARG: -Dsurfacedump.probe=1
VMARG: -Dsurfacedump.out=E:\PYTHON\CoreSwap\.tmp\b61-260917-06\sd.txt
===B61-VMARGS-END===
```

**判定**：`-Dsurfacedump.dim` **缺席**（未发射）⇒ 传 `-PsurfaceDumpDim=...` 静默无效。**DEFECT-MISSING-MAPPING 实锤**（从静态推断升为行为化直证）。

**旁证**：同族 `chunkX/chunkZ/size/out/outPost` 均有映射 ⇒ 用户自然会预期 `dim` 也可用 ⇒ 缺陷面成立。

## 3. 哨兵 B（SCOPED 侧）：`-PheightProbe`（height.x/z 族的对照）

**实测输出**：
```
===B61-VMARGS-BEGIN===
VMARG: -Dheight.probe=true
===B61-VMARGS-END===
```

**判定**：该族**仅发射父门**，无任何子参数映射 ⇒ 与「族内零 `-P` 先例 ⇒ SCOPED」的裁决规则一致——`height.x/z` 判 SCOPED-DIRECT-D 成立（用户预期走 `-D` 直传）。

## 4. 两个哨兵的判别力结论

| 族 | 族内子参数映射数 | 实测发射 | 裁决 | 哨兵是否支持 |
|---|---|---|---|---|
| surfaceDump | 5/6（dim 缺） | 仅 probe + out | **DEFECT** | ✅ 支持 |
| heightProbe | 0/2 | 仅父门 | **SCOPED** | ✅ 支持 |

⇒ **裁决规则（族内对称性判据）经行为化验证具备判别力**：它不是「有 javadoc 写 -D 就算旁路」也不是「没映射就算缺陷」，而是用**族内先例**区分二者，两个方向的哨兵各证一侧。

## 5. 覆盖面与未覆盖面（§9.7）

- **覆盖面**：2 个哨兵（1 DEFECT 类 + 1 SCOPED 类）；`jvmArgs` 发射面为**权威发射点**（强于读源码）。
- **未覆盖面**：① 其余 26 项未逐个哨兵（成本；按同族规则外推，**属外推非实证**）；② 未验证「补映射后行为是否如预期」（T5 之后的前置）；③ 未跑端到端 runServer 生成（本次哨兵只到 JVM 参数发射，未等世界生成——`sentinel-surfacedump.log` 的 boot 已 Done 3.651s，但 probe 未产出属预期：probe 需特定 chunk 触发）。
- **降级声明**：本验证为 **Partial**（JVM 参数发射面直读 = 单点行为化），**不得称 Full**；「补映射后生效」的 Full 验证属后续。

## 6. 诚实声明

- 哨兵 A/B 输出为**原始日志粘贴**（见 `.investigations/b61-260917-06/cmd-output/vmargs-sentinel.log`）；
- init script 属临时件（`.tmp/`，§八.13 唯一临时区纪律），不入库；
- 本次验证**未改动任何生产代码**（只加临时 init script + 实跑）。
