# 260910-03 vivo 冻结课题——知识库草稿（core.worker subagent 产出，主会话应用）

> 按 SUBAGENT-KNOWLEDGE-GUIDE 产出。价值门判定 + 建议归属 + 可直接粘贴的条目文本。
> 数字全部来自 `.investigations/vivo-freeze-260910-03/record.md`（用户 confirmed 冻结解决）。
> 置信度：candidate（机制闭环 + 用户 confirmed 冻结解决；正式 confirmed 授予由用户拍板）。

## 一、条目清单与价值门判定

| # | 内容 | 价值门 | 建议归属 | 载体形态 |
|---|------|--------|----------|----------|
| 1 | #27 家族第三形态：existence-only marker 跨版本陈旧缓存 → id 域错位 → 错块级联 → OOM 冻结 | **高价值（错误链条 + 可复用判据）** | `knowledge/discovered/build-tooling.md` | **#27 家族补充案例**（追加小节，不新开编号——机制与 #27 同一上位根因，去重规则：同机制不重复立条） |
| 2 | 载体偏差判据：验证载体数据通路 ≠ 生产载体时「全绿」零判别力（数据三元组） | **中高价值（判错方法/签名）** | `knowledge/discovered/workflow-patterns.md` | **新条目 #105**（#36 执行体三元组的**数据域**扩展——三元组判据新增一维，值得独立编号，家族索引挂 #36/#98） |
| 3 | 冻结类故障演进观察法：双 dump 三态区分 + 实体普查 + 视觉人证 | **中价值（可复用排查手法）** | `knowledge/discovered/workflow-patterns.md` | **简记 #106**（手法组合，无新机制，简记档） |

去重说明：条目 1 判为 #27 补充案例而非新条目的理由——#27（260907-04）已定论「marker 语义是新鲜度不是完整性」这一上位机制，本案是其**跨版本同路径形态**（#27 是增量数据缺失静默 fallback，本案是**旧版本数据整集复用**且表现面从静默升级为灾难性错块）；机制同源、签名不同，按「同机制不重复立条、新签名进补充案例」处理。若主会话/judge 认为表现面差异足够大（错块灾难 vs 静默退化），可升格为新条目 #46，文本按下方备好（标题已写两用形态，改一行即可）。

---

## 二、条目 1 草稿（→ build-tooling.md，#27 条目之后追加小节）

```markdown
### #27 家族补充案例（第三形态）: existence-only marker 跨版本陈旧缓存 → blocks.json id 域错位 982/1003 → 大规模错块级联 → 百万实体 OOM 冻结——「同 dll+权威数据干净而生产爆」先查数据缓存指纹（260910-03）

- **时间/置信度**：260910-03；candidate（机制闭环：Temp 缓存 sha 铁证 + 修复后用户 confirmed 冻结解决；**错误优先·高价值**）。
- **来源定位**：`.investigations/vivo-freeze-260910-03/record.md`；产物 `.tmp/hang-repro-260910/`。
- **现象（五段式·现象）**：1.21.6-0.1.0 vivo 生产客户端游玩 ~40s 后世界冻结（两种表现：日志戛然而止 / 级联卡顿+`Too many chained neighbor updates`×15 + `Can't keep up! 101531ms` + `OutOfMemoryError: Java heap space`@-Xmx16384m）；地形大规模错块（用户视觉人证：「灰色的草」「沙砾」「假基岩」）；实体普查 region/entities NBT = `minecraft:item ×1,031,765` + falling_block×79；Server thread 线程 dump RUNNABLE 烧 CPU 166-2340s 在 FallingBlockEntity tick 实体碰撞扫描。同 dll + 同 seed 的 dev 服/Chunky 4225/forceload 1600 全绿零异常。
- **根因（机制）**：`CoreSwapFixHelper.extractWorldgenDir()` 数据缓存 = 跨版本共享**固定 Temp 路径**（`coreswap-data`），新鲜度判据只有两个 **existence-only marker**（overworld.json + tags marker）。用户机器上 1.20.1 生产 jar 留下的旧缓存 marker 全在 → 1.21.6 jar 跳过解压 → **1.21.6 dll 读 1.20.1 blocks.json**：两版块表 1003 vs 1105 项，982 个同名块 id 错位 → 每个 chunk 大规模错块 + 无支撑重力块 → 级联坍塌 + 流体/邻居更新链 → 百万 item 实体 → O(n²) 实体碰撞 + 16GB 堆 OOM → 冻结。这是 #27「marker 语义超载」的跨版本整集复用形态：existence marker 只证「有数据」不证「是**本版本**数据」——#27 是增量缺失（静默 fallback），本案是整集陈旧（灾难性错块），同一根因的两种表现面。
- **定位（怎么发现的）**：多轮绕路后一锤定音的动作 = **比对客户端 Temp 缓存 blocks.json sha vs jar 内 blocks.json sha**（ee01b749 = 1.20.1 内容 vs 617c3dae = 1.21.6 jar 内）——一次指纹核对即破案，此前 Chunky/forceload/dump 多轮载体验证全部无效（见 workflow-patterns 载体偏差条目）。
- **修复**：`extractWorldgenDir` 加**内容指纹**（jar blocks.json vs 缓存逐字节比对，不一致整体重解压；对齐 `extractNativeDll` 已有同款逻辑）。修复 jar sha 9eae48cd…（16:44），用户实机 confirmed 冻结解决。
- **教训/判据（可复用）**：
  1. **「同一 dll + 权威数据直读全干净，生产客户端爆」= 数据缓存指纹第一嫌疑**——dev/验证载体走 `-PcppWorldgenDir` 直读权威数据，生产走 Temp 解压缓存，两者数据通路不同；dll 相同不能证明数据相同。
  2. existence-only marker 的语义边界：只证「缓存非空/上次解压过」，不证「缓存 = 当前版本资源集」——**跨版本共享缓存路径必须配内容指纹**（逐字节或 hash 比对锚点文件），extractNativeDll 早已有同款而 extractWorldgenDir 没有 = 同类判据接线不齐的欠账。
  3. 家族索引：#27（marker 单判·增量缺失形态）、#18（产物在盘≠本次生成）、#96（加载路径跟实际 run 形态走）——共同上位原则：**「数据/产物与当前版本资源集等价」必须有独立证据，existence 是最弱的一档**。
```

---

## 三、条目 2 草稿（→ workflow-patterns.md，新条目 #105，追加文件末尾）

```markdown
## 发现 #105: 载体偏差——验证载体与生产载体数据通路不同时，「载体全绿」对生产故障零判别力；排查 MUST 在故障载体的真实数据通路上核对（数据三元组：加载文件 + 数据指纹 + 版本）（260910-03）

- **时间/置信度/module**：260910-03；candidate（vivo 冻结课题方法论复盘，用户 confirmed 冻结解决）；workflow-patterns / 执行体三元组家族（#36 的数据域扩展，第五形态）。
- **来源定位**：`.investigations/vivo-freeze-260910-03/record.md` 方法论复盘节。
- **现象**：vivo 冻结排查中 dev 服（Chunky 4225 chunks + forceload 1600 覆盖级联区）全绿零异常，双臂 diff 0.015% 干净——但生产客户端稳定复现冻结。多轮载体验证（Chunky/forceload/dump）对根因零判别力，最后在**客户端 Temp blocks.json sha 一查即破**（缓存 = 1.20.1 内容）。
- **根因（机制）**：验证载体与生产载体的**数据通路不同**——dev 服 `-PcppWorldgenDir` 直读权威数据目录，生产客户端走 jar 解压 Temp 缓存。执行体三元组（#36：加载文件/构建产源/构建时间）只覆盖「代码执行体」，不覆盖「数据执行体」：dll 相同 ≠ 数据相同。通路分叉时，验证载体上的一切「全绿」只证明权威数据+代码干净，对生产载体上缓存层引入的故障零判别力。
- **判据（可复用）**：
  1. 生产专属故障（dev 复现不了）排查，第一动作 = 画出**生产载体的真实数据通路**（jar → 解压 → 缓存 → 加载），在通路的每一跳核对，而不是在验证载体上重复加绿。
  2. 数据三元组扩展 #36：① 实际加载的数据文件（从加载代码追路径，不从配置推断）② 数据内容指纹（加载文件 sha vs 权威源 sha）③ 版本归属（数据集是当前版本的）——三者齐才认「跑在正确数据上」。
  3. 签名：「同 dll/同代码 + dev 全绿 + 生产爆」且故障表现指向内容层（错块/错数据）→ 直接查生产缓存指纹，跳过 Chunky/forceload 类载体复现轮次（本案绕路多轮的代价即在此）。
- **家族索引**：#36（执行体三元组——本条为其数据域维度）、#98（修复依赖的数据通路必须生产口径在位）、build-tooling #27 补充案例（本条的错误实体侧）/ #96（加载路径跟实际 run 形态走）。
```

---

## 四、条目 3 草稿（→ workflow-patterns.md，简记 #106，追加文件末尾）

```markdown
### 发现 #106 简记: 客户端冻结类故障演进观察法——双 dump 三态区分 + 实体普查一锤定音 + 视觉观察当人证（260910-03）

- **时间/置信度/module**：260910-03；candidate；workflow-patterns / 排查手法。
- **手法**：客户端冻结先抓**线程 dump ×2（间隔 ~60s）**区分三态：① **死锁** = 两帧完全同点钉死；② **进展崩塌** = 帧在移动但推进极慢（本案 2375 unique chunks ~21/s 推进 = 排除 Rust 死锁的直接证据）；③ **O(n²) 烧 CPU** = RUNNABLE + CPU 累计暴涨（本案 Server thread 166s→2340s 在 FallingBlockEntity tick 实体碰撞）。实体爆炸的一锤定音工具 = **region entities NBT 普查**（本案 `minecraft:item ×1,031,765`；注意：服务器卡死时自动保存不执行，region/entities 可能 0 字节——实体证据路径会断，需在读图/游玩中的存档上取）。用户视觉观察（「灰色的草」「基岩」）是错块类故障的关键人证线索，应当证据用不当噪音丢。
- **证据**：`.investigations/vivo-freeze-260910-03/record.md` 现场证据节；`.tmp/hang-repro-260910/client-threaddump-*.txt` + count_entities.py。
- **家族索引**：#85（关服瞬间批量异常时序排除）、#36（dump 状态 = 执行体观察面）。
```

---

## 五、INDEX.md 追加行草稿（主会话应用条目后同步）

```markdown
> 260910-03 追加：build-tooling 追加 **#27 家族补充案例（第三形态·错误优先）**（existence-only marker 跨版本陈旧缓存——1.21.6 dll 读 1.20.1 blocks.json，982/1003 id 域错位 → 大规模错块 → 重力/流体级联 → 百万 item 实体 → O(n²) 碰撞 + 16GB OOM 冻结；判据 = 「同 dll+权威数据干净而生产爆」先查数据缓存指纹，跨版本共享缓存路径必须配内容指纹）+ workflow-patterns 新增 **发现 #105**（载体偏差——dev 直读权威数据 vs 生产 Temp 解压缓存通路分叉时载体全绿零判别力；数据三元组 = 加载文件+数据指纹+版本，#36 数据域第五形态）+ **发现 #106 简记**（冻结三态观察法：双 dump×2 间隔 60s 区分死锁/进展崩塌/O(n²) 烧 CPU；实体 NBT 普查一锤定音；卡死服务器 region/entities 可能 0 字节断证据路径）。来源：.investigations/vivo-freeze-260910-03/record.md（用户 confirmed 冻结解决；遗留 1.21.6 性能回归 5.15× 下轮立项）。
```

## 六、自检清单（GUIDE §四）

- [x] 价值门：条目 1/2 高/中高价值详记，条目 3 中价值简记；无低价值内容写入
- [x] 条目 1 五段式完整（现象/根因/定位/修复/教训），无「只记修复」
- [x] 根因为机制层（marker 语义超载 / 数据通路分叉），非现象复述
- [x] 定位含可复用诊断动作（缓存 sha 指纹比对、双 dump 三态、实体普查）
- [x] 被排除假说保留（record.md 已排除清单引用在案，条目内引用 Rust 死锁排除证据）
- [x] 载体正确（错误链条→#27 补充案例 in build-tooling；判错方法→workflow-patterns #105；手法简记 #106）
- [x] 数字全部来自 record.md（982/1003、1,031,765、sha ee01b749/617c3dae、9eae48cd、101531ms、166s/2340s、0.015%），无编造
- [x] 格式与两目标文件末尾现状对齐（build-tooling 尾部 #45、workflow-patterns 尾部 #104，编号衔接 #46 预留 / #105、#106）
- [x] INDEX.md 追加行格式与既有 26090XXX 追加行对齐

## 七、主会话应用清单（供核对）

1. `knowledge/discovered/build-tooling.md`：在 #27 条目之后（或文件末尾，与 #27 邻近即可）粘贴条目 1 小节。
2. `knowledge/discovered/workflow-patterns.md`：文件末尾追加条目 2（#105）+ 条目 3（#106）。
3. `knowledge/INDEX.md`：末尾追加第五节追加行。
4. 时间线：`versions/1.21.6/docs/10-timewise-archive.md` 260910-03 块由主会话按常规流程处理（本草稿不含，record.md 已有过程全文）。
5. 遗留课题提醒：1.21.6 性能回归 5.15×（vanilla 52s vs coreswap 268s / 4225 chunks）用户已拍板下轮立项——勿随本课题销案。
```
