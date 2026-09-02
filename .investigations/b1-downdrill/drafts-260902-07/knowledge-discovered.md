# 草稿：knowledge/discovered 两条

> 载体 1：`knowledge/discovered/compiler-idioms.md` 末尾追加「发现 #9」。
> 载体 2：`knowledge/discovered/workflow-patterns.md`——发现 #13 末尾「补充案例（260902-05/06…）」之后追加新「补充案例（260902-07）」小节（现有格式 = #13 下挂补充案例小节，非新条目；两条案例同属 #13「探针输出先 sanity check / 零输出先查自身」判据家族，并入不另立 #14）。主会话应用。

---

## （compiler-idioms.md 追加）

## 发现 #9: 跨 session raw id 标注三查——未验证的 id→方块标注当公理继承，整条机制链作废（260902-07）

- **发现时间**：260902-07；**发现者**：core.worker 草稿（b1-downdrill H1 环1 证伪复盘）+ 主会话应用；**来源定位**：`.investigations/b1-downdrill/b1-errors.md` E-B1-9 + `facts-260902-06/07.md`；**置信度**：candidate（数据层实锤：LAUIDMAP 权威映射一轮推翻定案候选，judge PASS，confirmed 待用户拍板）；**module**：workflow / re-code（MC 注册表 / 交接纪律）。
- **观察**：MC 探针输出 raw block/state id（如 COLPROF `99|0->19319`）时，若首次出现未同时输出 id→方块映射，session 内凭数值直觉赋语义（「19319 大数≈新块≈lava」「5854≈netherrack」）后写入事实链，下一 session 即当公理直接续推——实测 Java STATE_IDS 权威映射：**19319=blackstone、5854=basalt**（lava=96、netherrack=5850），「熔岩海缺失」机制链环 1 整环证伪、修复方案作废。标注从「未验证解释」升格为「事实」只发生在文档传递里，不在数据里。
- **证据**：`[LAUIDMAP]`（探针启动时遍历 Registries.BLOCK/STATE_IDS 打印映射）vanilla 轮一轮实锤；LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零；COLPROF 10 列 diff 真相 = V 黑石底(y=99) vs C 玄武岩底(y=100~104)，两侧均实心。验证成本一轮，此前整条五环机制链与回归判据全部改写。
- **如何利用**：
  - **标注三查（与 seed 三查同级的开工检查项）**：① 探针输出的 raw id/枚举/魔法数**首次解释前必须先建立 id→语义映射**（探针打印 LAUIDMAP 类映射，禁止数值范围直觉命名）；② 标注跨 session 传递 MUST 带「已验证/未验证」标记，未验证标注续用前先做廉价独立验证（≤ 一轮，§16.3 宿主交接验证）；③ 机制链逐环追问「这一环的输入标注是谁验证的」——环 1 错则整链作废，越早核实越便宜。
  - **判据**：「机制链自洽 + 量级对得上」不能替代输入标注核实——自洽的链条建在错误标注上时全链同样自洽（本例黑石/玄武岩材质差同样能解释转换面漂移现象）。
  - 交叉引用：workflow-patterns 发现 #13（测量侧先查三犯——本发现是「标注/解释侧」的对应纪律）；AGENTS.md「交接结论验证纪律」（M14/M11 复盘）在 id 域的具体化。

---

## （workflow-patterns.md 发现 #13 末尾追加）

### 补充案例（260902-07，指标盲区 + 行首锚假零输出）

同判据家族两例（b1-downdrill lavaAudit 课题，详见 b1-errors.md E-B1-10/11）：

1. **探针指标盲区**：lavaAudit v1 只记 above=lava 转换，恰好测不到判别「熔岩海缺失」所需的 below=lava 面向（air→lava 转换面）——v1 输出「99.4% 一致」对核心命题零判别力，是「测了且一致」的假安心；v2 加记 lavaSurfY 后一轮实锤两侧 air→lava 面向均为零，直接推翻待测现象的存在。**判据扩展**：设计探针指标先写「要判别命题 P，最小充分证据是什么」再检查指标覆盖，指标名对口 ≠ 覆盖证明（与发现 #12 对拍对象错级同族：测量设计与判别目标脱节）。
2. **行首锚 grep 假零输出**：`^\[LAVAAUDIT\]` 对带日志框架前缀（时间戳/线程名）的 log 行恒零命中 → 误判「探针零输出」。与 260902-06 RegistryKey 命名空间前缀（本发现 #13 补充案例）同属「过滤条件把全部行静默滤掉」家族。**判据扩展**：「零命中」先打印一行原文核对行格式，`^` 行首锚对 log 行默认不可用。

---

## （INDEX.md 对应分类行更新 → 见 index-update.md）
