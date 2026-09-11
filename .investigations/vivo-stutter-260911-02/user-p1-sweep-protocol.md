# 用户实机扫描协议（P1 in-flight 限流，260911-02）

> 目标：判定「限制接管段并发度」是否救回客户端 FPS。
> 前提：预览 jar（含 P1 限流，默认 `maxinflight = 物理核-2`）。**每次只改一个变量**（只改 N，世界/视角/路线/VD 不变）。

## 0. 基线记录（每次进游戏后先做）

进游戏后按 F3 看 FPS；生成区域时记录**最低 FPS**与**主观卡顿感**（1-5 分，5=无法忍受）。
日志里核对两行（**参数是否生效**，缺失=该臂作废）：

```
[WG-INFLIGHT] max_inflight=10 logical=NN syncfill=false
[CHUNKTIME] ... inflight max=NN
```

> 若日志没有 `[WG-INFLIGHT]` ⇒ 预览 jar 没生效（装错 jar / 被旧 jar 覆盖），该臂作废。

## 1. 扫描臂（JVM 参数各加一个，其余不变）

| 臂 | JVM 参数 | 预期 |
|---|---|---|
| A | `-Dcoreswap.maxinflight=0` | 旧行为（不限，23 路）——复现基线 |
| B | `-Dcoreswap.maxinflight=12` | 每 chunk 池占用 -23%，wall 无损（**推荐起点**） |
| C | `-Dcoreswap.maxinflight=10` | 缺省值；每 chunk 更短，生成稍慢 |
| D | `-Dcoreswap.maxinflight=6` | 最保守；生成明显变慢，FPS 应最好（若机制成立） |

**建议顺序**：A → B → C → D（每条至少生成同一片新区域，如朝同一方向飞 1-2 分钟）。

## 2. 判定

- **A 卡、B/C/D 明显好转** ⇒ 机制成立，按 FPS/生成速度权衡定缺省值。
- **A 与 B/C/D 无差别** ⇒ 并发度不是客户端低帧主因 ⇒ 回 Phase 0 换方向（候选：客户端侧另有瓶颈，需 spark/帧时间剖面）。
- **B/C/D 更卡** ⇒ 车道背压反噬（生成速度成为瓶颈），改用更低侵入的方案。

## 3. 回传内容

- 每臂：`maxinflight` 值 + 最低 FPS + 主观分 + `[WG-INFLIGHT]`/`[CHUNKTIME]` 两行 + `latest.log`。
- 若某臂出现「区块不再生成/卡死」⇒ 立即停并回传（许可泄漏信号）。
