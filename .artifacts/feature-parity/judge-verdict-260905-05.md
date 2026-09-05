# Judge 审查意见：feature parity「Phase 4b patch + Phase 5 验证链」（260905-05，MUST 级）

- 审查角色：core-judge（subagent，只出意见不改 status；confirmed 留给宿主人类）
- 审查对象：工作区未提交代码 + .investigations/feature-parity 记录链 + .artifacts/feature-parity b1/b2 + .tmp/feature-parity-260905-05 受控 A/B
- 审查基线：三源核对（① git HEAD + 工作区 diff ② .investigations/.artifacts 记录 ③ 验证数据/脚本），并对可复算证据做了 **judge 独立复算**（非采信文档数字）
- 判定：**APPROVE-WITH-CONDITIONS**（建议升 candidate；confirmed 待人类拍板）

---

## 1. 三源核对结论（审查要点 a）

| 核对项 | 结果 |
|---|---|
| 工作区 diff ↔ phase4-status/apply-report 改动清单 | ✅ 一致：tree.rs 新建 + lib.rs `pub mod tree`；feature_loader/placement/carver/worldgen_handle 改动吻合 |
| worldgen_handle.rs 临时门回退声明 | ✅ 实核 diff：`WG_FEATURELOG` 走 `std::env::var(...).is_ok()`，无 feat_log_enabled! 硬门残留；`anchor_biome: Some(cur_biome_id)` 与 `generate_configured` 增 cache 实参（patch §2.4/§3.8）在位 |
| runtime/1.20.1/java/build.gradle -PfeatureLog 映射 | ✅ L195-198 实核在位 |
| features_probe.rs 参数化 | ✅ 在 diff 内（bin/features_probe.rs +18 行） |
| light_data.json 负 opacity | ✅ judge 逐条 diff 备份核验：613-629 `-1→15`、954→0；`.bak-negopacity-260905-05` 在位；light/mod.rs 注释 + 防回归单测在位 |
| **发现 S1（未登记）** | ⚠️ `.tmp/feature-parity-260905-05/working-tree.diff`（b1/b2 的证据包）是**过期快照**：622 行 vs 当前工作区 721 行，缺 features_probe.rs 参数化、light/mod.rs、lib.rs 等后续改动（快照时间 13:47:53）。b1/b2 已 DENY 故不影响其结论，但**提交前证据包须刷新** |
| **发现 S2（契约缺口）** | ⚠️ b1/b2 candidate 未登记进 `.artifacts/index.yaml`（grep 仅命中旧 b1-candidates 条目）——core.artifact 落盘契约缺口 |
| **发现 S3（最重，未登记）** | ❌ `versions/1.20.1/data/worldgen/light_data.json` 与其 .bak **均被 .gitignore:18 `data/` 忽略**（judge 实证 `git check-ignore -v`）——该文件为手工修复的数据文件（负 opacity 显式化）+ 防回归单测直读的真实数据源，**当前修复不可提交、不在版本控制内**；修复脚本 fix_neg_opacity.py 只在 .tmp（临时区，随时可清） |

## 2. 证据链合法性（审查要点 b）

- **方法论**：同 fresh world / 同 Done / 同 42-tile forceload 残差域 / region 收敛轮询 / 同停机拷贝，name 域对比（v3 口径，废弃跨 id 域）——口径成立，§9.7 三要素（载体 region / 覆盖 3009 双侧完整 chunk / 与旧 dll 117 口径可比）声明齐备。
- **judge 独立复算 ✅**：亲自重跑 `v8_ab_verdict.py`（ab-vanilla-region × ab-takeover-region）：`双侧地形完整=3009, diff chunks=1829, total=239594`，签名 = oak/jungle leaves + vine + 双向 ore 小残差（andesite/granite/diorite 各 ~1.0-1.2 万双向均衡）+ Y4-6 主导——**与 phase5-interim 记载逐位一致，无 blob 爆炸**。
- **「14.8× 回归」取代逻辑闭合 ✅**：正向对照（ab-vanilla vs g1-vanilla = 2,292,126 实例差、零 rust 参与 → 完成度伪影）+ b1/b2 双 DENY（无可达作用面 / RNG 不传播）+ 服务器考古（CppBridge init 后于 Done）三链互证，§15.4 取代记录（原结论保留 + 双指针）形式合规。
- **发现 S4（证据落盘缺口）**：⚠️ v8 判决原始输出与 cargo test 输出**均未落盘**（grep .tmp/.investigations 无 `test result:` / `239594` 原始输出）——数字只存在于 phase5-interim 正文。judge 本次复算输出已构成二次独立证据（本文件即记录），但按 Anchorlaw §1.3 主会话仍应把原始采集输出归档 `.investigations/.../cmd-output/`。
- **cargo test 复算 ✅**：judge 实跑 `cargo test --offline -p WorldgenRust --lib light::` = **2 passed / 0 failed**（含新防回归单测直读真实 light_data.json，613-629=15、954=0 断言实过）。

## 3. 风险移交项核销情况（审查要点 c）

| 项 | 状态 |
|---|---|
| §3.5/3.6 序列改变回归定界 | **部分核销，须显式转出**：A/B 只证明「整管相对旧 dll 净改善（80 vs 117/chunk）」，是**相对结论非绝对结论**——per-feature 随机序列改变未逐 feature 定界；残留 80/chunk 须归因后才能关闭 |
| R-1（decorator 同 Y HashSet 桶序） | 未核销 → 显式转出（已在树族残差签名中兑现：leaves/vine 双向差） |
| idk-7（selector/patch 占位） | 未核销 → 显式转出（与 ore 双向残差相容） |
| anchor_biome 口径 | 未核销 → 显式转出（Biome modifier 锚定口径 vs Java posToBiome jitter 未单测） |
| fancy_oak = LargeOak 口径 | 未核销 → 显式转出（验证口径待拍板） |
| S3（light_data.json 版本控制，本轮新发现） | 未登记 → 须新增移交项（见阻塞清单） |

## 4. 提交前阻塞项清单（MUST，阻塞 commit/confirmed）

1. **S3**：解决 light_data.json 版本控制——`data/` 整目录被 ignore，手工修复不可提交。三选一：① .gitignore 加显式反白名单 `!versions/1.20.1/data/worldgen/light_data.json`；② 修复脚本（fix_neg_opacity.py）迁出 .tmp 落正式位置并在 docs 登记「数据文件由 jar 提取 + 此补丁重放」流程；③ 与用户确认 data/ 目录管理策略。**在解决前该修复实质丢失风险为真实存在**（.tmp 脚本 + ignored 文件 + .bak 同 ignored）。
2. **S2**：b1/b2 补登 `.artifacts/index.yaml`。
3. **S4**：v8 判决 + cargo test 原始输出落 `.investigations/feature-parity/cmd-output/`（judge 复算记录见本文件，可作为独立第二证据引用）。
4. **S1**：刷新 `.tmp/.../working-tree.diff` 至最终提交态，或直接在 commit 信息引用最终 git diff hash。

## 5. SHOULD 项（不阻塞提交）

- phase5-interim 的 §3/§4 与取代记录同文件混排，最终归档时按时间线归口纪律拆入 10 时间线。
- 树族残差（~10-16k 级 leaves/vine）建立下轮基线数字（本判决复算的 239,594 / 1829 / 3009 即为 candidate 基线），后续 R-1/idk-7 收敛以此口径对表。

## 6. 判定理由

受控 A/B 判据经 judge 独立复算精确复现，正向对照排除伪影，取代链闭合；方法论符合 §9.7 可比性声明。四项阻塞均为**落盘/版本控制契约问题**，不动摇「工作区 Phase 4a/4b patch 无回归、净改善」的技术结论本身。因此 **APPROVE-WITH-CONDITIONS**：建议状态升 **candidate**（A/B 载体、受控双臂、可复算）；confirmed 待人类在阻塞项核销后拍板。

---
- judge 复算留痕：`.tmp/judge-recheck-diff-260905-05.diff`（当前工作区 diff 快照，721 行）；v8 复算输出见本文 §2（3009/1829/239594）；cargo test 复算：`test result: ok. 2 passed`（light:: 过滤）。
- 审查分层声明：本意见基于静态核对 + judge 侧可复算脚本重跑（python/cargo 均本会话实跑）；未重跑服务器 A/B 采集本身（采信既有 region 数据集，数据集完整性由双侧 5313 chunk 对称与复算一致性旁证）。
