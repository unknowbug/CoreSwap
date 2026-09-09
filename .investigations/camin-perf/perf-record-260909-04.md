# WG_CA_MIN 性能优化记录 — 260909-04（实际 2026-09-09 17:32 起）

> 架构计划：.investigations/000-架构设计/架构计划-260909-04-camin-perf.md（用户批准）
> git 基线 2bea71c → 本块 src 改动未提交（待 judge 后提交）。
> 课题：CA_MIN 翻默认开后 features 阶段成本回收（NEXT_SESSION 260909-03 可选项 2）。

## 测量载体

- `worldgen-core/src/bin-diag/camin_bench.rs`（新增，rustc 单编 + 最新 rlib 链接，#13/#30 纪律）：
  WorldgenHandle::create + 串行 fill_chunk_blocks N×N region，纯 wall + FNV-1a 内容 hash（行为等价哨兵）。
- 环境：seed -8248318472910187742，wg=versions/1.21.6/data/worldgen，16×16=256 chunks，origin(0,0)，串行（#28）。
- 诊断：WG_CA_LOG 门控计数器（CA_OUT_READS 既有 + 新增 CA_ALL_READS 全量读 + per-feature [CA-READS] 归因 + WG_CA_CAP 临时判别）。

## 判别链（证据饱和推进，每轮新数据层证据）

| 步骤 | 实验 | 读数 | 判定 |
|---|---|---|---|
| 基线 | on/off 两臂 | on 289.1ms/chunk vs off 128.5ms（**+125%**）；hash 6908dbfc…/115641b8… | 复现（比 dump 口径 +68% 更重，区域/版本上下文不同，§9.7 声明：bench 口径 ≠ 存档 dump 口径，不可比） |
| E1（scout C1 原判：主管线不回填） | fill_chunk_blocks 回填 cache | on 297.4ms —— **零改善** | C1-as-modeled 证伪：读路径早已把前向邻 chunk 算过入缓存 |
| **E1b** | 真实形态：主管线**缓存优先命中**（条目升级 (col,heightmap) Arc 对） | on 289→**177.9ms（+125%→+35%）**；off 无回归；hash 三轮逐位不变 | ✅ 实锤：每 chunk 地形双算（reader 算第 1 遍 + 管线无条件算第 2 遍） |
| E2b（C2 clear-all 雪崩） | WG_CA_CAP 256 vs 2048 | 178.6 vs 176.5ms | C2 排除（⚠️ 260909-06 取代注记见文末：排除结论被修正） |
| WG_SKIP_FEATURES 分解 | 地形段两臂 | 119.7 vs 118.0ms（+1.4%） | 双算已消灭；残差全在 features 段（58.2 vs 13.8ms，4.2×） |
| E4b（全量读计数） | all_reads | on **853k 读/chunk** vs off 7.5k（**114×**）；44ms/853k≈51ns/读 | 单读不贵（mutex+hash 量级），是读量放大 |
| E4c（per-feature 归因） | [CA-READS] | seagrass_deep/normal ≈53%，其次 ore×4/disk/monster_room | Java 同构下扫/邻域扫描的忠实工作，非病态冗余循环 |

## 结论（candidate，待 judge + 用户 confirmed）

1. +68%/+125% 的主导根因 = **主管线与读路径地形双算**（scout C1 的真实形态：不是「不回填」而是「不读缓存」）——E1b 修复回收其大部分。
2. 残差 +35% = **ca_min 语义正确的固有成本**：真实邻值使 feature 算法不再因 -1 早退（off 臂 114× 读量差）。单读 51ns 已近下限；进一步压读（memo 化下扫）需处理写失效，语义漂移风险高，不建议。（judge 备注：「Java 原版同样支付这部分工作」为**推断非实测**——本轮未跑 Java 对照。）
3. 行为逐位不变：三轮 on 臂 hash 全等（6908dbfc9a79c40a），off 臂全等（115641b86711d9cd）。

## 改动清单（src）

- worldgen_handle.rs：① terrain_cache 值类型升级 CaTerrainEntry{col:Arc,heightmap:Arc}（新 struct，#27 判据核对：非 enum variant 变更）；② neighbor_terrain 返回 entry.col；③ fill_chunk_blocks 缓存优先命中 + 未命中回填；④ WG_CA_LOG 诊断扩展（all_reads/per-feature，门控 chunk 级，非门控时热路径仅一个 bool 分支）；⑤ WG_CA_CAP 容量判别 env（ca_cap() 单一定义，judge 条件项 1 已统一两处调用点；默认 256 不变，保留作后续容量判别）。
- blocks.rs：BlockColumn derive Clone（纯数据 Vec，平凡语义）。
- bin-diag/camin_bench.rs 新增。

## 验证记录（Phase 2.5/4）

- workspace 全量 build 绿（worldgen1216 薄壳含内）；cargo test -p WorldgenRust --release：14 passed / 0 failed。
- 行为等价门：PASS（hash 三轮逐位，on/off 各自稳定）。
- 性能门（同口径 A/B）：on 177.9 vs off 131.8ms/chunk（+35%），相对优化前 +125% 回收 ~72%。
- **降级声明（§9.7）**：本块性能读数为本地 bench 载体（串行 region、bench 口径），与 260909-03 的 223.7s/133.3s 存档 dump 口径**不可比**（载体/覆盖面/口径三要素均不同）；vs Java 端到端大样本 e2e 本轮未跑（CA_MIN 课题为 Rust 内部相对优化，Java 基线不变式由 260909-03 既有 evidence 覆盖）。

### §15.4 取代注记：E2b「CAP 256 vs 2048 差 <2%，C2 排除」被修正（260909-06）

- **supersedes**：本记录 E2b——「WG_CA_CAP 256/2048 差 <2% → clear-all 雪崩（C2）排除」；**superseded-by**：probe-260909-06.md（WG_CA_CAP 判别臂复测）。
- **推翻理由（一行）**：差异真实存在且稳定——256 vs 2048 三轮配对交错 ~10-11% wall（165.9/172.4/162.5 vs 153.7/159.2/147.1 等，方向三轮一致）；260909-04 的 <2% 是该测量口径/条件下未显形（单轮、非配对交错、机器噪声带 ±10% 淹没信号，#24/#28 口径教训）。
- **机制修正**：且 C2 的机制形态也错——不是预置的「288 miss 雪崩」（实测 miss 仅 0-8/chunk，全 region 358/322），而是 **clear-all 后对已生成过的邻列纯增重生成**（work 移位后剩的净增量），打在主管线缓存优先读路径（#100 缓存读路径被 clear 波及）。
- **判据**：① 差异接近噪声带时，「未测出」≠「不存在」——必须配对交错（#24）+ 噪声基线（#51）后才可作排除结论；② 排除类结论（「C2 排除」）在后续轮新增判别臂时 MUST 复审（本例 NEXT_SESSION judge 条件①预置即此）。
- **来源定位**：.investigations/camin-perf/probe-260909-06.md（数据表 + 判据核对节；E2b 原文位置 = 本文件判别链表）。

## IDK / 后续

- IDK-p1：+35% 残差若仍需回收，方向 = seagrass/ore 下扫的写失效感知 memo（中高风险，需独立架构评审）。
- IDK-p2：bench +125% vs dump +68% 的口径差未深究（区域上下文不同：bench 含 region 边缘外邻重算）。
- 建议切 Session（见 NEXT_SESSION 更新）：src 改动待 judge 后提交；本块已闭合，无未验证假设残留。
