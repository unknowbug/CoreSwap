# 极端坐标 FP 微差应力测试 verdict（candidate → 用户已拍板封存）

> 课题：世界边界（±30M）极限坐标下，实现 vs vanilla 的差异是否演变为「地形颠覆」
> 口径（§9.7）：载体 = BlockProbe WGB2 端到端逐位对比（vanilla Java vs Rust dll 接管 NOISE/SURFACE，carver/feature Java 照跑，dll sha256=68d7f401…，与 B1/260902-10 定案同一构建）；覆盖面 = 2 seed × 2 极限角 × 4×4 chunk（每区 1,572,864 块）+ 1 普通坐标对照；可比性 = 与 260902-10 confirmed 同链路，overworld min_y=-64/height=384。
> seeds：+7159221168429822337 / −7159221168429822337（signed int64）；极限原点（块坐标，chunk 对齐）：+29999936（chunk 1874996..1874999，含世界最后合法 chunk）/ −30000000（chunk −1875000..−1874997）。

## 结论（candidate，用户 260902 拍板封存）

1. **无地形颠覆（PASS，预登记参考线 95% 全过，实际最差 98.59%）**：
   - ① +seed×(+30M,+30M)：**98.9634%**（失配 16,304）；表面高度漂移 0/4096 列
   - ② +seed×(−30M,−30M)：**99.0255%**（15,327）；0/4096
   - ③ −seed×(+30M,+30M)：**98.8536%**（18,031）；0/4096
   - ④ −seed×(−30M,−30M)：**99.8541%**（2,295）；592/4096 列漂移，幅度仅 ±1~3 格
2. **负坐标极限未炸**（用户重点关注项）：区②④与正极限形态对称，④为四区最佳——floorDiv/负轴路径无结构崩坏。
3. **极限坐标不放大差异（对照归因）**：普通坐标对照（+seed×chunk 200,200）一致率 **98.5914%**，低于全部极限区——区①②③的失配主体是「泥土带」系统差（stone/sand→dirt，y≈22-53 水平带，单簇 1.2 万~1.8 万块），该带在普通坐标同样存在，**非坐标极端化引起**；区④（无该带的岩质区）只剩 466 个 FP 擦边小散簇（最大 522 块，B1 同族量级）。
4. **FP 微差课题封存**：边界坐标下 FP 求值序微差不演变为结构崩坏；后续无需再投入（与性能优化轨道正交，B1 定案维持）。

## ⚠️ 醒目标注 —— 泥土带系统差（遗留项，仅记录不下钻，用户拍板）

> **【遗留 · 未归因 · 与极限坐标无关】** vanilla 在 y≈22-53 地下带为 stone/sand，Rust dll 侧为 **dirt**（区①②③ + 普通坐标对照均复现，单簇 1.2万~1.8万块，占 ~1%）。疑似 biome 驱动 surface rule 家族差异（B1 signature A「biome 真差」远亲），**未定位根因、未修复**。影响评估：地表不可见（表面高度零漂移），仅地下材质带错位。若未来做逐位 100% 对齐或依赖地下方块的功能，**此项是当前最大已知失配源**——见各 cmp_*.out.txt 的 (ref,cpp) id 对分布。

## 证据链

| 采样 | 一致率 | 失配 | 形态 | 明细 |
|---|---|---|---|---|
| ① | 98.9634% | 16,304 | 泥土带 y42-53 | .tmp/extreme/cmp_run1_Ppos.out.txt |
| ② | 99.0255% | 15,327 | 泥土带 y32-47 | cmp_run2_Pneg.out.txt |
| ③ | 98.8536% | 18,031 | 泥土带 y26-46 | cmp_run3_Npos.out.txt |
| ④ | 99.8541% | 2,295 | FP 散簇 466 个 | cmp_run4_Nneg.out.txt |
| 对照 | 98.5914% | 22,156 | 泥土带 y22-49 | cmp_ctl_Pmid.out.txt |

- 采集日志：.tmp/extreme/log_run2b/3/4/5/6/7/8/9/10_*.log（每次 worldSeed 与 server.properties 双核对；每跑删 run\world 防缓存 chunk 污染；ref 跑无 CppBridge、cpp 跑 16/16 chunk populateNoise intercepted + dll sha 68d7f401 核对）
- 探针/脚本：.tmp/extreme/cmp_region.py（WGB2 解析：32B 头 + 每 chunk [wx,wz + 256×height u16 + 256 writeUTF biome]）
- 环境：gradle 沙箱坑复现（GRADLE_USER_HOME 指 .gradle-home，build-tooling #7）；cppWorldgenDir 必须显式传 jar 外路径，否则 `worldgen-data not found` 崩

## 外推边界

overworld / 2 seed / 4×4 每区 / ±30M 角点；nether 未重测（B1 口径已 confirmed）；泥土带根因 idk。

---

## [补注 260902 · 用户线索，未验证]

用户提出：印象中 MC 官方说过 Java 版同 seed 生成两次世界也可能有万分之几的方块不同（vanilla 自身非确定性）。与本仓既有记录 workflow-patterns #10「同 dll 重跑非确定容差」同族。含义：对微小残差的「零容差」期待可能本身不成立。注：这不改变本 verdict 依据——泥土带 ~1% 为确定性系统差（跨运行稳定复现、同 seed 同形态），量级远超万分之几，非该非确定性可解释。

## [judge 审查 260902]

通过（意见全文：`.investigations/extreme-coord-stress/judge-opinion-260902.md`）；C1（index.yaml 登记）/C2（派生统计复现：`cmd-output/derive_stats.out.txt`）均已闭环。用户拍板：**课题封存**；泥土带系统差醒目标注、仅记录不下钻。
