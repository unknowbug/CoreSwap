# 旧遗留四项零成本收口处置记录（260920-04）

- status: confirmed（2026-09-20 用户在方案 A 选项中明确批准四项处置；本记录为处置登记载体，主会话应用）
- 性质：纯文档收口，零改码零采集零编译。
- 溯源依据：`.investigations/legacy-sweep-260919-06/backlog-map.md`（260919-06 scout 溯源地图，file:line 逐一实读核对）+ 本块复核。

## 1. 260914-02 open②（r1 首跑预热假设独立验证）→ **关账**

- 首证：`.investigations/ns-ab-260914-02/record-260914-02.md:108`「r1 首跑预热假设未做独立验证（如额外 warmup 臂）——不影响主判据（保守读法两路一致），留待下批可选」。
- 关账理由：① 主结论已 confirmed（record:3，2026-09-14 用户授予），open② 显式不影响主判据；② 同族判据已由 #103/#24 补充案例 + #150 以「交错配对必要性」形态沉淀，warmup 臂重采的边际信息量趋零；③ 重采成本 = 中（需加臂重跑）。
- 处置：关账（不闭合原假设，登记「永久挂起」；若未来出现与预热机制相关的新形态证据可凭本条重开）。

## 2. C-4 灰区二值化 → **维持 verdict 现状关账**

- 首证链：`.investigations/000-架构设计/架构计划-260919-04-C1前置G3收敛性预研.md:11`（verdict-260919-03 confirmed 裁定「C-4 灰区保留」）+ `.investigations/c1a-260919-05/record-260919-05.md`（C-1a J4 性能判据未满足，用户拍板保留缺省生效）。
- 关账理由：verdict 已 confirmed 且裁定「保留」；二值化属 C-1a 链延伸，其性能判据（J4）未满足、缺省生效为用户既定拍板；现无新形态证据（CP-2/CP-4 同族「降后复议条件」未触发）。
- 处置：关账。复议条件 = C-1a/域批链出现新形态证据。

## 3. idk-C1a-1（C-1a RSS 未采样）→ **转正式 idk 90 天线**

- 首证：`.investigations/c1a-260919-05/record-260919-05.md:49`（J6：RSS 未采样，净额预期为负，实测缺位）+ judge-review-260919-05.md:48（已显式声明「不作为采纳决策依据」——合规 idk 登记而非静默略过）。
- 处置：维持正式 idk 登记（idk-C1a-1），起 90 天升级线（自 260919-05 首证起算）；激活条件 = 出现需要 RSS 证据的采纳/回退决策，届时先建 #149 -Xmx RSS 采样配方再跑臂。

## 4. Forge+Connector AW（AW 在该载体应用未验证）→ **转正式 idk/待验证登记**

- 首证：`knowledge/discovered/workflow-patterns.md:3357`（AW 为 remap 安全替代；已知边界：Forge+Connector 载体对 AW 的应用未验证，未加宽将 loud IllegalAccessError，非静默、可判别）。
- 处置：登记为 idk（idk-aw-forge-1）。验证设计现成（Forge 生产 run 一次，失败签名 = loud IllegalAccessError）；激活条件 = 下一次 Forge+Connector 载体采集顺手覆盖，不为此单独跑 run（单独成本中高、当前无其它 Forge 采集需求）。

## 汇总

| 项 | 处置 | 状态 |
|---|---|---|
| 260914-02 open② | 关账（永久挂起，重开条件已记） | closed |
| C-4 灰区二值化 | 维持 verdict 关账（复议条件已记） | closed |
| idk-C1a-1 | 正式 idk + 90 天线 | idk open |
| Forge+Connector AW | idk（idk-aw-forge-1），随下次 Forge 采集顺手验证 | idk open |
