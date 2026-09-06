# review-003 — CONCERN-2 回归闭环 verdict 审查（260906-04，core.judge）

> 审查对象：`.investigations/end-takeover/concern2-regression-verdict-260906-04.md` + `.tmp/concern2-260906/`
> 三源核对：① 产物快照（verdict + .tmp 目录）② git HEAD=8e2a60d + status（本块零代码改动，仅未追踪 .investigations 文件）③ 验证记录（日志/SHA 实测重算）
> 本审查只出意见，不改 status；confirmed 留给人类。

## 审查意见

1. **【通过】执行体三元组指纹** — 实测 `target/release/worldgen.dll` SHA256 = 1B5AA1DEA49445A2…（2160640B），与 verdict 声称一致；`mod-arm-server.log:191` 实证运行时加载 1.0.26 jar 内 dll 同指纹；`mod1025-server.log:191` 实证基线臂 febe913e（2152960B）。三条独立来源互证，成立。

2. **【通过】同 dll 重跑噪声基线（#51 应用正确）** — mod1025 两臂（mod1025-* / mod1025b-*）region 目录在盘，日志 mod1025-server.log + mod1025-rerun.log 在盘；verdict 正确判定「互差与跨版本信号同阶 → 存档载体不可裁决」，符合 knowledge/discovered/workflow-patterns.md 发现 #51 判据（噪声基线前置必测）。

3. **【通过】确定性载体裁决（#52 应用正确）+ SHA 实测复核** — 本 judge 独立重算 dump SHA256：dump_new = dump_new2 = dump_old = 610766E0…（overworld，含两轮重跑）；dump_nether_new = dump_nether_old = 1248DAB2…。与 verdict 声称逐位一致。dump 脚本（concern2_nether_dump.rs）确认走 `WorldgenHandle::create_for_dim` + 纯 `fill_chunk_blocks`，无 Java 装饰层，头部内嵌 seed/min_y/height，代码 diff 是唯一变量——#52 载体四要素满足。载体裁决逻辑（#51 前置排除存档载体 → #52 确定性载体裁决）链条完整、方向正确。

4. **【通过】git 状态** — HEAD=8e2a60d，工作区无已追踪代码改动（仅未追踪的 verdict/计划文档），「本块零代码改动」声明属实；新代码基线 c2330cd 在 HEAD 前两个 docs-only 提交，与 dump_new 构建口径功能等价。

5. **【问题·轻】§9.7 覆盖面要素不完整：end 维度本身未在回归覆盖内且未显式声明** — verdict 结论限定「overworld/nether 共享路径无回归」，这对 CONCERN-2 原始诉求（共享路径回归）成立；但本次改造的直接对象 end 维度本身的行为等价性不在本 verdict 覆盖内，verdict 未显式声明该覆盖面边界（§9.7 三要素之「覆盖面」）。若 end 行为已在前序工作块（end-takeover 260906-03）单独验证，verdict 应引用之；若未验证，应在「遗留」补一条明示。

6. **【问题·轻】产物契约：verdict 未登记 .artifacts/index.yaml** — 结论性 verdict 只落 .investigations/（过程载体合规），但根 `.artifacts/index.yaml` 无 concern2/260906-04 条目；按 core.artifact「结果进 .artifacts + index」契约应补登记（登记动作归主会话，本 judge 不改）。

7. **【建议】旧代码 dump 的构建溯源未落盘** — dump_old/nether_old 声称由 99b8034 worktree 单编产出（exe 体积与新码不同，旁证成立），但构建命令/时间戳记录未在 verdict 或 .tmp 留痕；#23 家族（陈旧产物假绿）正是本项目踩过的坑。建议补一行构建命令 + 时间戳记录（或引用既有 cmd-output），使「旧码」身份可复核。此项不动摇 SHA 全等结论本身（两 exe 互异 + 各自输出互等已强约束）。

8. **【建议】存档对拍数字（2206/21511/3533/23805/5012/23025）未独立重算** — ab 脚本与三臂 region 目录均在盘，理论可复核，但本 judge 未重跑（重跑成本高且不改变裁决：存档载体已被 #51 判不可裁决，这些数字仅是探索性佐证）。可接受；如需更高严谨度可留一次性复算命令在 verdict 附注。

9. **【通过】置信度与边界** — status=candidate 合法（无 confirmed 越权）；「顺带沉淀」Forge 存档级跨 run 装饰非确定性属高价值判错经验，已有 NEXT_SESSION/knowledge 联动记录，未发现未声明降级（存档对拍已诚实标注「探索性/不可裁决」）。

## 总结论

**有条件通过。** 核心裁决链（三元组指纹 → #51 噪声基线前置排除存档载体 → #52 确定性 dump SHA 双维度全等）证据完整、实测可复核、逻辑成立，「ow/nether 共享路径逐位无回归」结论成立，建议升 candidate 定稿。

条件（收尾前补齐，均非重验性）：
- C1：verdict 补 end 维度覆盖面显式声明（引用前序 end 验证或列入遗留）；
- C2：主会话补 .artifacts/index.yaml 登记；
- C3（SHOULD）：补 dump_old 构建溯源一行记录。

**建议用户在 C1/C2 补齐后确认关闭 CONCERN-2**（close 口径 = 共享路径回归闭环；end 维度行为等价性按 C1 声明的归属另行追溯）。confirmed 由用户授予。
