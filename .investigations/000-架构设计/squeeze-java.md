---
编号: 001
任务: 确认 Java 1.20.1 SQUEEZE 公式（scout 产物）
阶段: Phase 1 勘探
状态: candidate
来源: scout subagent（read_only_task，隔离勘探）
---

## 结论

Java 1.20.1 `DensityFunctionTypes.UnaryOperation.apply(Type, double)` SQUEEZE 分支（DensityFunctionTypes.java 1161-1164 行）：

```java
case SQUEEZE -> {
    double d = MathHelper.clamp(density, -1.0, 1.0);
    yield d / 2.0 - d * d * d / 24.0;
}
```

数学：`squeeze(x) = clamp(x,-1,1)/2 − clamp(x,-1,1)³/24`

**与 C++（density.h:154 `d = clampD(x,-1,1); return d/2 - d*d*d/24;`）逐位一致** → **squeeze 排除为根因**（2 倍差是我手算遗漏 squeeze 的 /2：0.64×(-0.1188)=-0.076 → squeeze(-0.076)=-0.038 = densityDump ✓ 链路自洽）。

## 附带发现

- `data/mc_src_extract/` = MC 1.20.1 完整 yarn 源码提取（scout 定位，后续验证直接读，不用跑 gradle）
- 1.20.1 无独立 Squeeze record（1.19.4+ 重构为 UnaryOperation.Type.SQUEEZE 枚举）

## 证据

- 源码：`data/mc_src_extract/net/minecraft/world/gen/densityfunction/DensityFunctionTypes.java` L1161-1164
- loom sources jar：`E:\PYTHON\MC\versions\1.20.1\java\.gradle\loom-cache\minecraftMaven\...minecraft-merged-...-sources.jar`（同源）
