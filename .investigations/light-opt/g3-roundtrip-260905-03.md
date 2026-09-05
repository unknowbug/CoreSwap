# G3 round-trip 实测记录（260905-03 · run1/run2 · FAIL at 严格判据）

## 实验设置
- seed 8576294172403134396（server.properties 核对 ✓）；run\world 删除后全新生成。
- run1（02:34，gate ON `-PlightRust=1`）：全新 world 生成 spawn 区域 → RCON `stop` 优雅存档。2025 chunks 存档，[LightRust] fallback = 0（全接管）。
- run2（02:37，gate ON）：同 world 重启 → Done (16.1s) → 停 20s → RCON stop。**[LightRust] lightInit ok 打印 → 至少 1 个 chunk 发生了 light() 调用（重算）**；fallback = 0。
- 快照：`.tmp/g3-260905-03/snap_light.py`（per-chunk section light 签名哈希）；对比 `cmp_roundtrip.py`。

## 结果
- chunks：before=2025 after=2025（new=0 gone=0）。
- **light-changed chunks = 123 / 2025（6.07%）→ 严格判据「重启无重算、光照不变」FAIL**。
- 空间归因（roundtrip_anatomy.py）：0/123 在 pregen 边缘；全部在距边缘 11–21 chunk 的**内部簇**（x15-36, z-24..-4，约 spawn 中心区）。

## 未解释点（诚实声明）
- run2 无 fallback 日志但 lightInit 被调用 → 重算路径是 rust 接管（非 vanilla 回退）；为何已 lightOn 的 chunk 会被重新 light() 未定位。
- 机制候选（未验证，不当公理）：①start-region 装载前沿 chunk 在邻未就绪时被 relight（但 fallback=0 与邻缺失矛盾）；②rust 隐式数组（flag1/flag2 均 omitted key）被 vanilla 读回按缺键=15 物化，邻 chunk relight 传播后显式回写 → 序列化漂移固化。
- @anchor.idk：LightStorage.enqueueSectionData 再传播语义（源不在手）仍未关闭——judge 遗留风险 #1，本 FAIL 使其升级为 G3 主阻断。

## 结论（draft，对照实验后更新）

- **G3 round-trip 严格判据（光照不变）对 vanilla 也不成立**：vanilla 对照组（gate OFF，同 seed 全新 world，同流程）首载漂移 = **17/2025（0.84%）**；rust 组 = 123/2025（6.07%，≈7×）。
- **rust 二次重启收敛**：run2→run3 = **0/2025 变化**；首载漂移是一次性现象，之后稳定。
- 每次启动都有少量 relight（rust run2/run3 均有 lightInit ok），1.20.1 存档全量**无 isLightOn 键**（vanilla/rust region 实测一致；`.tmp/net` ChunkSerializer L318 写该键与实测矛盾 → 该源树疑非 1.20.1，不作 1.20.1 一手源引用）。读侧 flag=false → relight 调度（vanilla 亦然）。
- **rust 超额漂移（123 vs 17）为真实差异信号**，候选机制（未验证，不当公理）：①mixin 早退返回跳过 vanilla POST enqueue 排序；②rust relight 重算代价大（run2 Done 16.1s vs vanilla 2.8s）暴露更多待收敛窗口。
- 运行口径（§9.7）：载体=存档 section light 签名哈希（snap_light.py）；覆盖面=单 seed 45×45 spawn 区 2025 chunks；与内存 DataLayer/客户端口径不可比。产物：`.tmp/g3-260905-03/`（run1/2/3、van-run1/2 日志 + 5 份 light 快照 + snap/cmp/anatomy 脚本）。
- @anchor.idk：LightStorage.enqueueSectionData 再传播语义未核（round-trip 收敛性已实证，其风险降级）。

## G3 判据建议（待用户拍板）
- 严格判据（光照不变）对 vanilla 亦不成立 → 不适用。
- 相对判据候选：「rust 首载漂移 ≤ vanilla 基线同量级」——当前 7× 未达标；「二次重启后 0 漂移」——已达标。
- 超额漂移是否拉起修复 = 方向决策。
