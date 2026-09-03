# review-preload-check-260904-01 — judge 审查意见（原样落盘）

> judge 结论：**PASS with should-fix**（should-fix 2 项已由主会话清偿，见文末附注）

## 三源核对结果

### 源 1：产物快照
- `.artifacts/preload-check/index.yaml`：✅ 契约完整。
- verdict：✅ 与实际 diff 逐条对得上。
- errors 台账：✅ E1/E2/E3 均五段式齐全 + 速查表。

### 源 2：git 工作区 diff
- `surface_rules.rs`（+49/-11）：✅ collect_noise_keys 泛化 / collect_rule_noise_keys / ENGINE_NOISE_KEYS 与 verdict 一致。
- `worldgen_handle.rs`（+30/-4）：✅ use ENGINE_NOISE_KEYS + 启动期断言块一致。
- `CoreSwapFixHelper.java`：runtime/ 已 untrack，git 不可核对——judge 直读文件核实 routeRel 双兼容（data*→原样 / minecraft*→data/ 前缀）、marker 路径、fail-fast 提示 2 处均正确；资源重排 849 文件（845+4）实测吻合，noise_settings/overworld.json 实存。
- 附带发现：`WorldgenRust/.tmp-negtest.txt` 残留（已清偿：删除）。

### 源 3：验证记录
- sweep 4096/4096 无 panic ✅。
- estopt_ab hash 与声明一致，但当时仅 1 臂（should-fix 2）。

## 重点项
1. 断言覆盖 + 盲区诚实（引擎路径机械收集不覆盖、清单为唯一事实源）：PASS。
2. 负向测试 #20 判据：PASS（机制）/ should-fix：panic 输出补落盘。
3. 零语义回归：should-fix：补四臂或 §9.7 口径声明。
4. routeRel 双兼容 + -PcppWorldgenDir 影响：PASS（提示：旧布局目录用户需改指 data/ 层目录，fail-fast 有提示）。
5. 错误台账五段式：PASS。

## 汇总
- 推荐状态：candidate；补齐缺口 1、2 后可提交用户 confirmed；非重大转向无需重开。

## 附注（清偿记录，主会话补）
- should-fix 1：负向 panic 输出落盘 `cmd-output/negtest-calcite-260904-01.txt`（exit=101，精确报 ["minecraft:calcite"]）。
- should-fix 2：四臂 env 强制（00/01/10/11）全跑，hash 全 `f2b1a3932c6e589e`，落盘覆盖原文件；verdict 补 §9.7 三要素声明。
- 卫生项：`.tmp-negtest.txt` 已删；neg-test 往返后最终源码重打包，jar 内 dll sha256 复验 = 最新构建（972041E6...）。
