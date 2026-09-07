# Judge 意见书 — 260907-04「tag 数据驱动化」（swe）

> 审查角色：core.judge（subagent 隔离）。只出意见，不改 status；confirmed 留给人类。
> 三源核对方式：本 judge 会话有 shell，**独立复跑**了 `cargo test -p WorldgenRust --lib --offline`（5 passed, 0 failed）、`cargo build --offline --release`（Finished；dll 2,192,896 B @2026-09-07 17:07，与交付声明一致）、`git status --short` + `git diff` + 直接读取 gitignored 路径（runtime/、tags 数据目录、.tmp）。

## 总裁定：PASS-with-conditions（建议保持 draft，条件满足后可升 candidate）

## 三源核对结果

1. **产物快照** ✓：`.artifacts/tag-datadriven-260907-04.md` 存在，内容与工作区实测一致（下述两处微差）；index.yaml 末条 `swe:tag-datadriven-260907-04` status: **draft** ✓（judge 前未升 candidate，合规）。
2. **git 工作区** ✓：diff 范围与声明完全吻合——carver.rs(+19)/feature.rs(+12)/worldgen_handle.rs(+3)/lib.rs(+1)/6 bin 各 +1~3 行（仅 EndIslands arm）/data-driven-boundary.md/index.yaml；block_tags.rs 新文件 untracked；Java 双 marker、170×2 tag JSON、.tmp 脚本在 **gitignored 路径**（`runtime/` 整目录 ignore、`.gitignore:24`、`.tmp/`），已直接读文件核验存在且内容符合声明。
3. **验证记录** ✓（本 judge 独立复跑）：cargo test 5/5 PASS（含 3 条 block_tags golden/negative）；release build Finished。注：测试输出中有 2 条 `warning[E0133]`（gpu_ffi unsafe，warning 级、test cfg、预存），不属本块改动，不阻塞。

## 逐项结论

- **A. golden 等值测试 — PASS**。三条测试真实存在且复跑通过；数据路径 = `concat!(env!("CARGO_MANIFEST_DIR"), "/../versions/1.20.1/data/worldgen")` 编译期锚定仓库相对路径，稳定（workspace 布局不变即稳）。微差①：`golden_feature_tags` 注释写「7 个 tag」实际列 6 个（artifact 说 6，代码注释笔误，不影响测试效力）。微差②：注释称 netherrack「由 golden 3 覆盖」——golden 3 测的是不存在目录，与「netherrack 无文件」走同一 `load_raw → None` 代码路径，等价性成立但表述略有跳跃（INFO 级）。
- **B. fallback 接线 — PASS**。carver：`expand_tag` true → return JSON 集；false → `fallback_overworld_replaceable`，单一出口无双重 push（out 未动时才走 fallback）。feature：同构，`expand_tag` false 时 fallback 继续向同一 out push，无重复（JSON 路径失败时 out 长度未变）。空集语义（解析成功但全缺失 → true + 空 out，不 fallback）已注释声明，与 Java RegistryEntryList 语义一致，认可。
- **C. 并发/全局状态 — PASS-with-note**。`OnceLock<RwLock<Option<Registry>>>` + `init` 在 `create_for_dim` blocks 加载后、load_carvers 之前调用，锁序一致（registry 读锁 → cache Mutex，无反向获取，无死锁路径）。**隐患（不阻塞，建议记注释/待办）**：全局单例按 `wg_dir` 变更才替换——同进程多 handle 用**不同** wg_dir 时后 init 覆盖前者，前 handle 的 expand 会读到后者的 tag 数据。当前场景（同进程各维度共用同一 worldgen-data 解压目录）安全；若未来多世界各配独立数据目录需改为 handle 持有 registry。建议在 block_tags.rs 头部补一行此边界声明。
- **D. Java 双 marker — PASS（静态审查）**。逻辑正确：任一 marker 缺失 → deleteRecursively + 重解压，正确覆盖「旧 tmp 缓存无 tags 静默跳过」失效场景；3 行改动（tagMarker 声明 + 条件合取 + 注释）无语法风险。**微瑕**：解压后只复检 `marker` 不复检 `tagMarker`——若资源 jar 本身缺 tags（打包事故），静默走 Rust fallback 无报错。建议条件③补一行 tagMarker 复检 throw。
- **E. netherrack 声明 — PASS**。`feature.rs:85` fallback 分支保留 `minecraft:netherrack → add(netherrack)`，artifact §语义要点如实声明「无 tag 文件（实测 jar MISSING）→ fallback 正常路径，非数据缺失」。诚实、与代码一致。
- **F. 6 个 bin 修复 — PASS**。diff 逐行核对：全部仅添加 `DensityFunction::EndIslands(_)` match arm（display 名/跳过/克隆），无语义改动；channel_probe 两处 arm 与声明一致。最小正确。
- **G. 降级声明 — PASS（诚实）**。artifact §验证明确声明 Java 未编译复验、mod namespace tag 未实测——与本 judge 独立观察一致（runtime/ 无构建产物可查）。§9.7 口径三要素（载体/覆盖面/可比性）已同行声明。
- **H. index 登记 — PASS**。status: draft，注释标「judge pending」，无越级。

**噪声卡核对**：本 judge 会话未初始化 .anchorlaw，噪声卡历史未核（INFO，不阻塞——本块为 swe 构造域，无 @anchor.test 声明义务）。

## 条件清单（满足后建议 candidate，confirmed 由用户拍板）

1. **（发布前 MUST）** Java 侧跑一次 gradle 编译（`compileJava` 即可）+ 一次生产 runServer 冒烟，确认双 marker 改动编译通过且旧 tmp 缓存被正确重解压（当前降级声明仍然有效，但不能带降级进发布工单）。
2. **（ SHOULD）** CoreSwapFixHelper 解压后补 `tagMarker` 复检（缺则 throw，与 marker 同等待遇），封堵「jar 打包缺 tags 静默退化」。
3. **（SHOULD）** block_tags.rs 头部补多 handle / 多 wg_dir 边界声明注释（全局单例 last-init-wins 语义）。
4. **（NICE）** 修正 `golden_feature_tags` 注释「7 个」→「6 个」。

## 推荐状态

保持 **draft**；条件 1（+2/3）完成后建议升 **candidate**。confirmed 授予权在用户。
