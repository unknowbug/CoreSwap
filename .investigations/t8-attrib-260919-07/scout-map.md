# T8 归因勘探地图 — scout-map（260919-07，draft，只读勘探产物）

- 角色: scout（re-code/recode-scout 形态，静态只读；本文件 = 唯一写产物）
- 课题: T8 domain 臂 vs vanilla 臂 light-hash diff = 4223/9450（44.69%）机制归因（.b1 域批算法差 / .b2 settled 时序差 / .b3 snap 载具覆盖差）
- 纪律: 零采集、零修改既有文件；原始事实（F）与推断（I）分开标注；置信度随条目标注。

---

## 1. 两臂数据形态盘点

### 1.1 载体与采集链（F，源 = run_t8.py + criteria-t8.md）

- 两臂同 seed `8576294172403134396`、同 dll sha8 `cc4e39fe`、同 workload（FP-DRV 虚拟玩家驱动，`ev=move seq=5` 为完成信号）+ Done 后 settle 60s → rcon stop → **停服后**对 `world/region` 做 snap。
- snap 载具 = `.tmp/g3-260905-03/snap_light.py`（两臂同一脚本同一次执行形态）：解析每臂 region/*.mca 的 chunk NBT `sections`，对每个 section 取 `(Y, sha256(BlockLight)[:12] or "-", sha256(SkyLight)[:12] or "-")`，排序后整体 sha256 → **per-chunk 16 hex hash**；key = `"cx,cz"` 世界块坐标。
- 快照时点（F，log 实读）：domain 臂 Done@18:55:14，最后 [LIGHT-DOMAIN] task@18:58:00，stop@18:58:46（move5@18:57:44 + 60s settle）；vanilla 臂 Done@19:10:22，move5@19:12:52，stop@19:13:55。**两臂都是停服后落盘 region → snap 读的是关服时已写盘的 light 数据**。

### 1.2 SELFCERT / world 身份（F，result.json 实读）

| 字段 | domain-r1 | vanilla-r1 |
|---|---|---|
| done / drv_move5 | 1 / 1 | 1 / 1 |
| lightInit_ok | 1 | 0（负自证 #118 通过） |
| lightrust_lines / domain_hook | 1 / 1 | 0 / 0 |
| dll_sha8 | cc4e39fe | cc4e39fe |
| seed_in_leveldat | true | true |
| region_files | 18 | 18 |
| region_min_mtime_gt_rmtree | true | true |

### 1.3 覆盖面（F）

- 两臂 snap 各 **9450 chunk**，key 集合**完全相同**（common=9450，new=0，gone=0）→ 覆盖面一致，.b3 的「缺键语义差」在 chunk 级不成立（见 §3）。
- chunk x 范围 3..342、z 范围（diff 子集观测）-25..212（FP-DRV spawn 邻域驱动箱）。
- domain 臂运行 log 内 [LIGHT-DOMAIN] task 行 457 条，其中 **timedOut=true 115 条**（ok 均 >0，无 ok=0 行）；vanilla 臂 0 条 domain 行（负自证一致）。（F）

### 1.4 snap 哈希的信息论边界（F，关键）

- snap_light.py 只落盘 **最终 16 hex hash**，**不落盘 per-section 签名、不落盘 light 字节本体** → 既有 diff 数据**无法**回答：|Δlight| 量级（多少级差）、差在哪个 y/section、BlockLight vs SkyLight 哪个通道差、缺键 vs 值差（hash "-" 与否被混入总 hash，无法分离）。
- 附带事实（F，代码直读 snap_light.py:23）：`each_chunk_raw` 循环体对 file 级 hash 有一个死代码计算（未入 snap），不影响输出；:88 chunk 坐标 = `rx*32+(i&31), rz*32+(i>>5)`，与 region 索引惯例一致。

---

## 2. diff 的已知统计（cmp_t8.py 输出了什么 / 没输出什么）

**已输出（F）**：chunks a/b = 9450/9450；common = 9450；new = 0；gone = 0；**diff = 4223（44.6878%）**；sample 行（有缺陷）；VERDICT: FAIL。
- scout 独立复算（本 session，ConvertFrom-Json 全量逐 key 比对）：**diff = 4223 逐位复现** ✓（置信度：高——三重一致：cmp_t8.py、judge-recompute judge 复算、本次复算）。
- diff chunk 坐标范围（本次新算，F）：x 13..332，z -25..212 —— **全驱动箱弥散**，非边缘窄带（仅此一维直读，未做连通域）。

**已知缺陷（F，judge N5）**：cmp_t8.py:16 sample 行对 str key 做序列切片 → 抽样坐标显示失真（如 "13,-1" 显示为 "1,3"）；计数与 VERDICT 不受影响。

**没输出 / 既有数据不可得（F→I）**：
1. |Δ| 量级直方图 —— **不可得**（hash 无量纲，见 §1.4）。
2. diff section 的 y 分布 —— **不可得**（同上；除非按 §5 模板扩展重解析 mca，但那是新采集边界的只读重解析，主会话自行决策是否越出「零采集」——模板默认不做，仅给注释位）。
3. 空间连通域 / 成簇度 —— **未做**（本次模板补齐，§5）。
4. BlockLight/SkyLight 通道分离 —— **不可得**（通道覆盖形态 #188 关注面；同受 §1.4 限制）。
5. run 间重复臂（n=1）—— 不存在，E1 摆动面无本对拍自有数据（仅 G17 旁证 0.8%，E1 档，judge N2 明示不作 E2 上界）。

---

## 3. 三候选证据分账（让渡清单：只标归属建议 + 已知事实，不解释机制）

### .b1 域批算法差（域批 fill pass vs vanilla per-chunk 传播语义差）
- 已知事实：domain 臂 457 task、115 task timedOut=true（ok 仍 >0）；sealed 计数递增至 342；graceMs=10000（log 头）。（F）
- 已知事实：diff 弥散全箱 44.7%，两臂 chunk 集一致。（F）
- 归属建议：承接「系统性、大面积、与执行体强绑定」类证据；若 §5 分布分析显示 diff 呈地形相关成簇（如洞窟/水面），偏向承接。（I，低置信，待分布结果）

### .b2 settled 时序差（采集时点未 settled / tick 瞬态 / 重载不收敛）
- 已知事实：两臂均 settle 60s 且停服后 snap；domain 最后一次 task 在 stop 前 46s。（F）
- 已知事实：domain 臂存在 timedOut=true 任务 115/457 —— 「超时但 ok>0」的语义（任务结果是否终态）是 .b2 的核心待判事实。（F）
- 归属建议：若 domain 臂重跑 n=2 diff ≈ 0 而 vs vanilla 仍 diff，则瞬态面被剥离、证据流走 .b1/.b3；若 n=2 自身 diff 大，.b2 升主嫌。（I，归属条件已给，机制让渡）

### .b3 snap 载具覆盖差（两臂 snap 覆盖/坐标/缺键语义差）
- 已知事实：**两臂同一 snap 脚本、停服后同一 world 布局语义、chunk 集完全一致（new=gone=0）**；seed/dll/region 数均一致。（F）
- 已知事实：snap 坐标推导（rx*32+…）与 18 个 region 文件规模自洽。（F）
- 归属建议：chunk 级覆盖差已被数据排除（低概率承载 44.7%）；残余面只剩「section 缺键（'-'）语义在两臂 NBT 序列化差异下混入 hash」——该子面被 §1.4 信息论边界遮蔽，**现状既不可证实也不可证伪**，需通道/缺键分离才可出清。（I，条件化归属）

> 分账一句话：.b3 chunk 级已被覆盖面一致事实压缩到「section 缺键语义」残余子面；.b2 挂在 115×timedOut 语义与 n=1 无重复臂两个未判事实上；.b1 承接默认剩余证据但同样未有任何直接证据指向 —— 三者当前**均无决定性证据**，让渡给 worker/fan-out。

---

## 4. 风险与判据注意

1. **world 身份链 #161**：两臂 result.json 的 seed_in_leveldat=true + region mtime>rmtree 已核（§1.2）；分布分析若复用 snap json 无需再碰 world；若任何后续动作重开 region 文件，必须重新走 #161 链（region 已被后续 run 覆写的可能存在——`.tmp` snap json 才是稳定证据体）。（F+I）
2. **判据读法不可改**：criteria-t8.md 为预登记存在性判据（diff>0 即 FAIL，无阈值带 #154）；本轮 FAIL verdict 已成立且经 judge 独立复算，归因工作**不得回改判据读法或翻案 verdict**。（F）
3. **通道覆盖形态 #188**：BlockLight/SkyLight 通道分离是登记在案的覆盖形态关注面；既有数据无法分通道（§1.4）——分布分析结论**必须声明该盲区**，不得以 chunk 级 hash 差冒充通道级结论（§9.7 无效声明清单）。（F）
4. **口径声明**：本对拍 = E2 跨执行体；E1 0.8% 带不作 E2 上界（judge N2）；n=1 不外推其他 seed/维度/驱动方式。（F）
5. **统计口径风险（I）**：44.69% 是「hash 不同的 chunk 占比」，**不是**「光照值差 chunk 占比」的上界或下界的直接解读——1 bit 级差与满级差在同 hash 比较下等价，任何百分比外推都须先过此声明。

---

## 5. 主会话可执行的分布分析命令模板（零采集，只读既有文件）

```python
# -*- coding: utf-8 -*-
# t8_diff_dist.py — T8 diff 空间成簇分析（只读 t8-*-r1-light.json，零采集）
# 用法: python t8_diff_dist.py
# 输出: diff chunk 散点摘要 / 4-连通域成簇统计 / 坐标直方（x,z 各 20 桶）
# 声明: 既有数据为 per-chunk 16hex hash，无量纲、无 y、无通道 —— 本模板不做也不得解读
#       |d| 量级、y 剖面、BlockLight/SkyLight 通道分离（数据不存在，#188 盲区）。
import json, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

D   = r"E:\PYTHON\CoreSwap\.tmp\legacy-sweep-260919-06"
FA  = D + r"\t8-domain-r1-light.json"
FB  = D + r"\t8-vanilla-r1-light.json"

a = json.load(open(FA, encoding="utf-8"))
b = json.load(open(FB, encoding="utf-8"))
assert set(a) == set(b), "key set drifted; re-check coverage before any analysis"
diff = {(int(k.split(',')[0]), int(k.split(',')[1])) for k in a if a[k] != b[k]}
allp = {(int(k.split(',')[0]), int(k.split(',')[1])) for k in a}
print(f"chunks={len(allp)} diff={len(diff)} ({100.0*len(diff)/len(allp):.4f}%)")

xs = [p[0] for p in diff]; zs = [p[1] for p in diff]
print(f"diff x range {min(xs)}..{max(xs)}  z range {min(zs)}..{max(zs)}")

# --- 4-连通域成簇（chunk 粒度） ---
seen, comps = set(), []
for p in diff:
    if p in seen: continue
    stack, comp = [p], 0
    seen.add(p)
    while stack:
        c = stack.pop(); comp += 1
        for dx, dz in ((1,0),(-1,0),(0,1),(0,-1)):
            n = (c[0]+dx, c[1]+dz)
            if n in diff and n not in seen:
                seen.add(n); stack.append(n)
    comps.append(comp)
comps.sort(reverse=True)
print(f"components={len(comps)} max={comps[0]} top5={comps[:5]} "
      f"singletons={sum(1 for c in comps if c==1)}")
# 读法提示（让渡 worker）: 大连通域 => 空间相关(地形/传播面)证据偏向 .b1/.b2；
# 高 singleton 比 => 离散点状差（section 缺键类随机语义）偏向 .b3 残余子面。仅分账提示。

# --- 边缘直方（20 桶，观感级） ---
def hist(vals, lo, hi, n=20, tag=""):
    w = (hi - lo + 1) / n
    h = [0]*n
    for v in vals: h[min(int((v-lo)/w), n-1)] += 1
    print(tag, " ".join(f"{lo+i*w}:{c}" for i, c in enumerate(h) if c))
hist(xs, min(p[0] for p in allp), max(p[0] for p in allp), tag="x-hist")
hist(zs, min(p[1] for p in allp), max(p[1] for p in allp), tag="z-hist")

# --- 如需 y 剖面/通道分离: 现有 json 不含该信息（snap_light.py 只存总 hash）。
#     任何此类分析 = 重新解析两臂已停服 world 的 region mca（或重采），属新动作，
#     须主会话按 #161 重走 world 身份链后另行决策；本模板不包含。
```

- 可执行性自检（本 session 实证）：`FA`/`FB` 均存在（289305 B ×2），`set(a)==set(b)` 断言当前成立（9450=9450），diff=4223 复现 → 模板**可直接跑**。连通域与直方段为本 session 未实跑部分（模板草案），逻辑为标准 BFS/分桶，风险低。（I，标注未实跑段）

---

## 6. 勘探产物与置信度汇总

- 本图所有 F 条目 = 文件直读/本 session 复算（高置信）；I 条目 = 归属建议与读法提示（低/中置信，待 worker）。
- 未做：连通域实跑、任何 mca 重解析、任何采集 —— 均按纪律让渡主会话。
