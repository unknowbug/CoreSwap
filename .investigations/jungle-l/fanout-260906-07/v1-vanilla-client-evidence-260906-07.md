# V1：-Vanilla 客户端复验裁决证据（260906-07，draft→candidate 材料）

> 载体：用户实跑 `run_rust_client.ps1 -Vanilla` 客户端（loom + fabric-loader 0.15.11 + fabric-api 0.92.0，无 Rust dll，worldgen = 纯 vanilla），旁观模式，F3 截图 4 张（含 /seed）。
> seed 三查：截图聊天框 `Seed:[8576294172403134396]` ✓（与 pregen 基线/交叉表同 seed）。
> 观察：两分歧点用户均站在树干正中，Targeted Block = jungle_log。

## 截图事实

1. (469,71,-230)：XYZ 469.5/71.0/-229.5，Targeted Block **469,72,-230 minecraft:jungle_log** → **Y**。
2. (505,72,-261)：XYZ 505.5/72.0/-260.5，Targeted Block **505,73,-261 minecraft:jungle_log** → **Y**。

## 四环境对照（两分歧点）

| 点位 | 实机(modded, 260906-05) | pregen(runServer 批量, 260906-06) | **-Vanilla 客户端(本次)** | rust ca_min=on |
|---|---|---|---|---|
| (469,71,-230) | Y | **N** | **Y** | Y(h19) |
| (505,72,-261) | **N** | Y(h30) | **Y** | Y(h30) |

## 推论（收敛材料）

1. **B1/R1 执行序依赖获运行时直证**：`-Vanilla` 客户端与 pregen **同 seed、同载体家族、同 vanilla worldgen 代码**，唯一差异 = 加载方式（客户端集成服渐进加载 vs 服务端一次性 forceload 批量）→ (469) 一侧有一侧无。**同代码同 seed 不同执行序 → 不同 mega 集**，「不存在唯一 Java 基线」从推论（j5-baseline 结论 B，candidate）升级为运行时实锤。
2. 方向与 E0 矩阵吻合：chunk (29,-15) 在 pregen 中 order=#7 早跑、东侧邻（(30,-15) #13、(30,-16) #14 等）未完成 → (469) 无树；客户端渐进加载下邻块特征先在 → 有树。
3. **实机 505=N 成为孤例**（其余三方全 Y）：归 .b2（实机 mods/Connector 改行为）或 .b3（当时观察误差，未开旁观目测）——实机侧 mods 清单仍缺（.b2 E1 未执行）。
4. rust ca_min=on 在两点与 `-Vanilla` 一致——260906-05「ca_min=on 更接近实机」直觉获旁证；**口径声明（§9.7）**：跨执行序，不作量级结论。
5. 判别签名升级：mega 集对执行序的敏感面**包含单点有无级翻转**（469），不只是位移。

## 边界与诚实声明

- 客户端为单人集成服，其 chunk 加载序本身无日志——「渐进序 ≠ 批量序」由结果反推（构造性），受控版 = .b1 E2（同服务器分批 forceload 重跑）仍可作闭环确认实验。
- 实机原始 11 点真值（260906-05）观察条件未确认是否旁观——精度待降级标注（本轮用户未开旁观即看错一次，B3 权重回调：非弱）。
- 本证据不改 .b1/.b2/.b3 status；candidate 授予待 judge。
