# judge 审查意见 review-001（260905-02 workspace 拆分行为回归 candidate）

审查角色：core.judge（subagent，只出意见不改 status）。三源核对 + 独立重跑。

## 三源核对结果

1. **证据快照**：regression-260905-02.md 在案，含实验矩阵、口径声明、解码表、执行环境与坑记录。✅
2. **git 状态**：HEAD = f506909（dadc4f4 拆分 + gitignore 修正），与主张一致。dadc4f4 --stat 核对：纯结构迁移（git mv 重命名 + 3 个新 Cargo.toml + jni_bridge.rs 一处路径改动 + .gitignore），无算法源码改动 → 单变量前提成立。✅
3. **验证记录**：.tmp/regress-260905-02/ 5 份导出 + cmp_blocks.py + decode_diffs.py 齐全。judge 独立重跑全部对比：

| 对比 | 重跑结果 | 与记录一致 |
|---|---|---|
| vanilla vs rust | 3 diff bytes @ 42229/2079781/2080293 | ✅ |
| rust vs rust2 | 0 diff | ✅ |
| vanilla vs vanilla2 | 0 diff | ✅ |
| olddll vs vanilla | 3 diff bytes 同偏移 | ✅ |
| **rust vs olddll（judge 补做的直接对比）** | **0 diff** | ✅（超出记录的推断链） |
| decode_diffs.py | 3 块 world 坐标/块名与记录逐字一致，解析 assert p==len 通过（biome 段到 EOF 无失步） | ✅ |

## 逐项审查

1. **实验设计**：通过。A/B 对照单变量（dadc4f4 为纯结构拆分，见上）；确定性检验双侧充分（vanilla 重跑 0 diff + rust 重跑 0 diff，排除了环境噪声导致的假 0/假 3）；旧 dll 从拆分前源码独立 worktree 构建，排除了「用新源码冒充旧 dll」的可能。**补充强度**：记录中 (a) 的「逐位一致」是由「两者 vs vanilla 差异集合相同」推断的（严格说差集相同不排除差值不同），但 judge 补做 rust-vs-olddll 直接对比 = 0 diff，结论实际成立且比记录的论证更强——建议在记录中把这条直接对比补入矩阵。
2. **§9.7 口径三要素**：完备。载体（blockProbe FULL 存档写入口径逐字节）/覆盖面（单 seed、4×4@200、overworld、含 carver 邻域预生成）/可比性（同构但显式声明了历史口径差异并援引发现 #36）三要素齐备，且诚实声明了 100.000% 历史口径不可直接续推——这是 §9.7 的正确用法。
3. **差异块解码可信**：通过。decode_diffs.py 有 `assert p == len(data)`（biome UTF 段逐条读取至 EOF 无失步即通过），重跑通过；id→块名映射用 blocks.json 反查；3 块世界坐标解码（198,18,198)/(237,41,224) 落在 4×4@200 范围内，自洽。
4. **结论 (b) 不拉起**：合规。残差签名（单块 water/dirt、granite/gravel 互换）落在既有永久挂起域（aquifer/ore 族），按 260904-15 拍板不重新拉起——judge 认同；本审查未复核 260904-15 拍板文本本身（超本轮范围），假定其为有效 human hook 记录。
5. **替代解释排查**：
   - vanilla 侧环境变化：#4 vanilla 重跑 0 diff 排除（同 session 内 vanilla 自身稳定）。
   - dll 打包链污染：#5 旧 dll 是拆分前源码独立构建且与新 dll 逐位一致，打包链若被污染则两者不可能一致；build.gradle 从 target/release/worldgen.dll 打包（新路径）在记录坑段落有提及。基本排除。
   - 残余小缺口：旧 dll 构建用的是 worktree @291bda6 = 拆分前最后一个 commit，但若拆分前工作区有未提交改动，则「拆分前源码」以 291bda6 为锚——结论 (a) 的严格表述应为「与 291bda6 构建一致」。记录未声明这一点，属轻微不严谨，不动摇结论（291bda6 即拆分基线 commit，无未提交改动证据缺失记录）。

## 需补实验

无 MUST 项。两个 SHOULD 级低成本补充：
1. 把 rust-vs-olddll = 0 直接对比结果补进 regression-260905-02.md 实验矩阵（judge 已代跑，数字见上；补录即可，无需重跑）。
2. 确认 291bda6 时点工作区无未提交源码改动（git 记录层面核对一次），把「旧 dll = 291bda6 构建基线」写明。

## 推荐

**APPROVE**（附上述 2 项 SHOULD 补录）。建议拆分升 **candidate**（confirmed 留人类拍板）。结论 (c) 的口径解读正确：NEXT_SESSION「预期 100% match」= 拆分前后一致（已证），历史 100.000% 口径不作为本轮验收判据——与 §9.7 一致。
